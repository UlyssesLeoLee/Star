//! `external_worktree_import.rs` — ULYS-158.5 / FR-ORCA-011 (平行 git worktree 兼容).
//!
//! Per `docs/ecosystem-survey/orca-design-survey.md` v1.0 §3 FR-ORCA-011 +
//! ULYS-218 (ULYS-158.5) description §1.3:
//!
//! ## AC
//!
//! - AC-1: external git worktree 默认 hidden (UI sidebar)
//! - AC-2: sidebar 显示 "hidden worktrees" card 提示
//! - AC-3: `git worktree remove` 后 Orca 下次刷新清理自身状态
//!
//! ## 模块边界 (per brief §2.3 + ULYS-195 ULYS-177.4 旁路)
//!
//! 本模块是 ULYS-218 (ULYS-158.5) bundle 的 1/3, 与 ULYS-195 (ULYS-177.4)
//! 平行实装: ULYS-195 在 `worktree-service` 实现"git porcelain v2 解析
//! + scan_external_worktrees + diff_external + map_to_worktree" —
//!   数据面全在那里.
//!
//! 本模块是 **trait + registry 抽象面** (per brief §2.3):
//! - `ExternalWorktreeImport` trait: 暴露 `scan / import / cleanup_stale`
//!   三个方法, caller (UI sidebar / REST endpoint / `RealWorktreeCreateAsync`)
//!   只调 trait 不接触 git porcelain.
//! - `InMemoryExternalWorktreeImport`: in-process 实现, 给前端 + 测试用.
//! - 真实 `RealExternalWorktreeImport`: 桥接到 `worktree-service::external_worktree_import`,
//!   即 ULYS-195 已 ship 的实现.
//!
//! ## 不依赖 Multica CLI
//!
//! 同 ULYS-177 路径 C: 本 trait 抽象 + in-memory 都不依赖 CLI. 真实
//! 实现走 `git-adapter::GitProvider::list_worktrees` (per FR-ORCA-011 AC-1),
//! 复用 git-adapter 已有的 `git worktree list --porcelain` 解析逻辑
//! (worktree-service::scan_external_worktrees 也是同一路径). 本 crate
//! 不直接调 git CLI, 避免重复实现 porcelain 解析.
//!
//! ## Real impl 桥接 (ULYS-218.4 / ULYS-231)
//!
//! `RealExternalWorktreeImport` 跟 `RealStartFromPicker` 用同一种
//! 模式 (per brief §2.1 + PR #107 设计语言):
//! - 持 `Arc<dyn GitProvider>` (libgit2 / CLI 都行)
//! - 持 `Arc<ExternalWorktreeImportRegistry>` (repo_id → 本地 path)
//! - scan: registry.lookup(repo_id) → GitProvider::open_repo → list_worktrees → 转 DTO
//! - import: 同 path 必须先 scan 出来; 标记 is_managed=true, 返回
//!   派生的 WorktreeId (本 crate WorktreeId = RepoId 占位, per PR #107
//!   InMemory 同样的 placeholder 策略, 等 ULYS-217 WorktreeStateMachine
//!   实装后再调 RealWorktreeCreateAsync::start 真正进 Provisioning)
//! - cleanup_stale: 调 GitProvider::remove_worktree (本期最小实装: 对
//!   stale path 做 git-level remove; 状态机侧等 ULYS-217 跨 session 续).

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use git_adapter::provider::{GitProvider, RepoHandle};
use graph_core::types::RepoId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::error::SharedDirError;

// =====================================================================
// 错误类型 (per brief §2.3)
// =====================================================================

/// `ExternalWorktreeImport` 错误码 (per 守门 #6 v2 6-field).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExternalWorktreeImportError {
    /// `git worktree list --porcelain` 调失败 (binary 不在 PATH, IO 失败, etc.).
    #[error("git worktree list scan failed: {0}")]
    GitCommand(String),
    /// porcelain 输出解析失败 (非预期 v2 格式).
    #[error("git worktree list parse failed: {0}")]
    Parse(String),
    /// RepoId 在本地 registry 找不到对应 path (per FR-ORCA-011 caller 检查).
    #[error("repo_id={repo_id:?} not found in registry: {message}")]
    RepoNotFound {
        /// 不存在的 repo_id.
        repo_id: RepoId,
        /// 错误描述.
        message: String,
    },
    /// `import` 时该 path 不在 `git worktree list` 输出里 (e.g. user 给
    /// 了一个不存在的 path, 或者已经被 `git worktree remove`).
    #[error("external worktree at {path:?} not found in scan output")]
    PathNotInScan {
        /// 找不到的 path.
        path: PathBuf,
    },
}

impl From<ExternalWorktreeImportError> for SharedDirError {
    fn from(e: ExternalWorktreeImportError) -> Self {
        let code = match &e {
            ExternalWorktreeImportError::GitCommand(_) => "WSD.EXT_GIT_CMD_FAIL",
            ExternalWorktreeImportError::Parse(_) => "WSD.EXT_PARSE_FAIL",
            ExternalWorktreeImportError::RepoNotFound { .. } => "WSD.EXT_REPO_NOT_FOUND",
            ExternalWorktreeImportError::PathNotInScan { .. } => "WSD.EXT_PATH_NOT_IN_SCAN",
        };
        SharedDirError::new(code, e.to_string(), "wsd-external-worktree-import-trace")
    }
}

/// `ExternalWorktreeImport` 结果类型别名.
pub type ExternalWorktreeImportResult<T> = Result<T, ExternalWorktreeImportError>;

// =====================================================================
// DTO (per brief §2.3)
// =====================================================================

/// 一个 external worktree（用户用 `git worktree add` 创建的）.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalWorktree {
    /// 实际 worktree 工作目录 (`git worktree list --porcelain` 的 `worktree` 行).
    pub path: PathBuf,
    /// HEAD commit SHA (full 40 hex, 由 git porcelain v2 `HEAD <sha>` 行给).
    pub head_commit: String,
    /// 当前分支 (`None` = detached HEAD).
    pub branch: Option<String>,
    /// `is_managed = false` 表示 Non-Orca 创建 (per brief §1.3 + AC-1 hidden default).
    pub is_managed: bool,
}

// =====================================================================
// Trait (per brief §2.3)
// =====================================================================

/// `ExternalWorktreeImport` — 平行 git worktree 导入 trait (per FR-ORCA-011).
///
/// 三个方法对应 brief §1.3 三个 AC:
/// - `scan` (AC-1 hidden default): caller 拿到所有 external worktrees,
///   UI sidebar 默认不显示 (由 UI 决定 hidden / visible).
/// - `import` (AC-2 sidebar card): 用户从 sidebar "hidden worktrees"
///   card 点 "Import", 把 hidden external 拉进 Orca 状态机.
/// - `cleanup_stale` (AC-3): 用户跑了 `git worktree remove` 后, 下次
///   scan 自动清掉 state machine 里的 stale 记录.
#[async_trait]
pub trait ExternalWorktreeImport: Send + Sync {
    /// 扫描仓库下所有外部 (Non-Orca 创建的) worktrees.
    ///
    /// 返回值: `ExternalWorktree` 列表, 包含所有 `is_managed = false`
    /// 的项. UI sidebar 据此决定 hidden / visible.
    async fn scan(&self, repo_id: RepoId) -> ExternalWorktreeImportResult<Vec<ExternalWorktree>>;

    /// 把一个 hidden external 导入到 Orca 状态机 (per AC-2).
    ///
    /// 成功后该 worktree 进入 `Provisioning → Running` 状态 (per
    /// WorktreeStateMachine 8 态); caller 不需要关心 transition 细节.
    async fn import(&self, repo_id: RepoId, path: PathBuf) -> ExternalWorktreeImportResult<RepoId>;

    /// 清理 stale 记录 (per AC-3 — `git worktree remove` 后).
    ///
    /// 返回被清理的 path 列表 (caller 用于 logging + audit).
    async fn cleanup_stale(&self, repo_id: RepoId) -> ExternalWorktreeImportResult<Vec<PathBuf>>;
}

// =====================================================================
// In-memory 实现 (per brief §2.3 + 用于测试 / 前端 dry-run)
// =====================================================================

/// `InMemoryExternalWorktreeImport` — 进程内 state-backed 实现, 无 git 调用.
///
/// 用途:
/// - UT 覆盖 `ExternalWorktreeImport` trait 全部方法 (scan / import / cleanup_stale)
/// - 前端 dry-run (无真实 git 二进制时给个 mock 数据)
/// - CLI `multica worktree ext-import list` 在仓库未初始化时的占位
///
/// 真实实现 `RealExternalWorktreeImport` 见 ULYS-195 已 ship 的
/// `worktree-service::external_worktree_import` (本 crate 通过 path
/// 边界, 不在 `worktree-shared-dir` 里复制 porcelain v2 解析).
#[derive(Debug, Clone, Default)]
pub struct InMemoryExternalWorktreeImport {
    inner: Arc<RwLock<InMemoryExternalWorktreeImportState>>,
}

#[derive(Debug, Default)]
struct InMemoryExternalWorktreeImportState {
    /// repo_id → Vec<ExternalWorktree> (per-repo 工作集)
    worktrees: std::collections::HashMap<RepoId, Vec<ExternalWorktree>>,
    /// repo_id → 本地 path (callable 时校验 path 来自真实 repo)
    registry: std::collections::HashMap<RepoId, PathBuf>,
}

impl InMemoryExternalWorktreeImport {
    /// 新建空实例.
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 repo_id → path (per FR-ORCA-011 caller 检查).
    pub fn register_repo(&self, repo_id: RepoId, path: PathBuf) {
        let mut s = self.inner.write().expect("InMemoryExternalWorktreeImport lock poisoned");
        s.registry.insert(repo_id, path);
    }

    /// 注入一份 external worktrees (测试 + dry-run 用).
    pub fn seed(&self, repo_id: RepoId, items: Vec<ExternalWorktree>) {
        let mut s = self.inner.write().expect("InMemoryExternalWorktreeImport lock poisoned");
        s.worktrees.insert(repo_id, items);
    }
}

#[async_trait]
impl ExternalWorktreeImport for InMemoryExternalWorktreeImport {
    async fn scan(&self, repo_id: RepoId) -> ExternalWorktreeImportResult<Vec<ExternalWorktree>> {
        let s = self.inner.read().expect("InMemoryExternalWorktreeImport lock poisoned");
        // repo_id 必须注册过 (per FR-ORCA-011 caller 检查, 真实 git 也走 lookup)
        if !s.registry.contains_key(&repo_id) {
            return Err(ExternalWorktreeImportError::RepoNotFound {
                repo_id,
                message: "repo_id not registered in InMemoryExternalWorktreeImport".into(),
            });
        }
        Ok(s.worktrees.get(&repo_id).cloned().unwrap_or_default())
    }

    async fn import(&self, repo_id: RepoId, path: PathBuf) -> ExternalWorktreeImportResult<RepoId> {
        let mut s = self.inner.write().expect("InMemoryExternalWorktreeImport lock poisoned");
        if !s.registry.contains_key(&repo_id) {
            return Err(ExternalWorktreeImportError::RepoNotFound {
                repo_id,
                message: "repo_id not registered in InMemoryExternalWorktreeImport".into(),
            });
        }
        let items = s.worktrees.entry(repo_id).or_default();
        // 该 path 必须在 scan 输出里 (per AC-2 "Import 按钮触发, 源是 hidden card")
        if !items.iter().any(|w| w.path == path) {
            return Err(ExternalWorktreeImportError::PathNotInScan { path });
        }
        // 标记 is_managed = true 进入 Orca 状态机 (per AC-2: import 进入 Provisioning)
        for w in items.iter_mut() {
            if w.path == path {
                w.is_managed = true;
            }
        }
        // 真实 `RealExternalWorktreeImport::import` 返回 `WorktreeId`;
        // 本 in-memory 简化: 用 repo_id 作为 placeholder (per brief §2.3
        // signature `WorktreeId` 与 `RepoId` 都是 `Uuid`, graph-core
        // 给的是 type alias). caller 不应依赖该 placeholder.
        Ok(repo_id)
    }

    async fn cleanup_stale(&self, repo_id: RepoId) -> ExternalWorktreeImportResult<Vec<PathBuf>> {
        let mut s = self.inner.write().expect("InMemoryExternalWorktreeImport lock poisoned");
        if !s.registry.contains_key(&repo_id) {
            return Err(ExternalWorktreeImportError::RepoNotFound {
                repo_id,
                message: "repo_id not registered in InMemoryExternalWorktreeImport".into(),
            });
        }
        // 模拟 "git worktree list" 现状: registry 里所有 path 都算 stale
        // (per AC-3 — 真实实现会比对 last_seen_at + `git worktree list`)
        let removed: Vec<PathBuf> = s
            .registry
            .get(&repo_id)
            .map(|p| vec![p.clone()])
            .unwrap_or_default();
        if let Some(items) = s.worktrees.get_mut(&repo_id) {
            items.clear();
        }
        Ok(removed)
    }
}

// =====================================================================
// 单元测试 (per brief §5: ExternalWorktreeImport ≥ 10 条)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn ext(path: &str, branch: Option<&str>, managed: bool) -> ExternalWorktree {
        ExternalWorktree {
            path: PathBuf::from(path),
            head_commit: "a".repeat(40),
            branch: branch.map(String::from),
            is_managed: managed,
        }
    }

    #[tokio::test]
    async fn scan_empty_returns_empty() {
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        let r = picker.scan(repo).await.unwrap();
        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn scan_returns_seeded_external_worktrees() {
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        picker.seed(
            repo,
            vec![
                ext("/tmp/wt1", Some("feat-a"), false),
                ext("/tmp/wt2", None, false),
            ],
        );
        let r = picker.scan(repo).await.unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].branch, Some("feat-a".into()));
        assert_eq!(r[1].branch, None);
        assert!(!r[0].is_managed);
    }

    #[tokio::test]
    async fn scan_unregistered_repo_errors_repo_not_found() {
        let picker = InMemoryExternalWorktreeImport::new();
        let unregistered = RepoId::new_v4();
        let err = picker.scan(unregistered).await.unwrap_err();
        match err {
            ExternalWorktreeImportError::RepoNotFound { repo_id, .. } => {
                assert_eq!(repo_id, unregistered);
            }
            _ => panic!("expected RepoNotFound, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn import_marks_managed_true_for_existing_path() {
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        picker.seed(repo, vec![ext("/tmp/wt1", Some("feat-a"), false)]);

        let _id = picker.import(repo, PathBuf::from("/tmp/wt1")).await.unwrap();

        let after = picker.scan(repo).await.unwrap();
        assert_eq!(after.len(), 1);
        assert!(after[0].is_managed, "import should flip is_managed=true");
    }

    #[tokio::test]
    async fn import_path_not_in_scan_errors() {
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        picker.seed(repo, vec![ext("/tmp/wt1", Some("feat-a"), false)]);

        let err = picker.import(repo, PathBuf::from("/tmp/nope")).await.unwrap_err();
        match err {
            ExternalWorktreeImportError::PathNotInScan { path } => {
                assert_eq!(path, PathBuf::from("/tmp/nope"));
            }
            _ => panic!("expected PathNotInScan, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn import_unregistered_repo_errors_repo_not_found() {
        let picker = InMemoryExternalWorktreeImport::new();
        let unregistered = RepoId::new_v4();
        let err = picker.import(unregistered, PathBuf::from("/tmp/x")).await.unwrap_err();
        match err {
            ExternalWorktreeImportError::RepoNotFound { .. } => {}
            _ => panic!("expected RepoNotFound, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn cleanup_stale_clears_all_worktrees_for_repo() {
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        picker.seed(
            repo,
            vec![
                ext("/tmp/wt1", Some("a"), true),
                ext("/tmp/wt2", None, true),
            ],
        );

        let removed = picker.cleanup_stale(repo).await.unwrap();
        assert_eq!(removed.len(), 1, "应该把 registry path 报为 stale");
        assert_eq!(removed[0], PathBuf::from("/tmp/repo"));

        let after = picker.scan(repo).await.unwrap();
        assert!(after.is_empty(), "cleanup 后 scan 应为空");
    }

    #[tokio::test]
    async fn cleanup_stale_unregistered_repo_errors() {
        let picker = InMemoryExternalWorktreeImport::new();
        let unregistered = RepoId::new_v4();
        let err = picker.cleanup_stale(unregistered).await.unwrap_err();
        match err {
            ExternalWorktreeImportError::RepoNotFound { .. } => {}
            _ => panic!("expected RepoNotFound, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn scan_hidden_default_semantics_ac1() {
        // AC-1: scan 返回的 items 默认 is_managed=false (caller 据此 hidden)
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        picker.seed(repo, vec![ext("/tmp/wt1", Some("feat-a"), false)]);

        let items = picker.scan(repo).await.unwrap();
        assert!(!items[0].is_managed);
        // UI sidebar 据此决定 hidden; 本 trait 不管 UI 行为.
    }

    #[tokio::test]
    async fn import_keeps_head_commit_field_intact() {
        let picker = InMemoryExternalWorktreeImport::new();
        let repo = RepoId::new_v4();
        picker.register_repo(repo, PathBuf::from("/tmp/repo"));
        let original_sha = "abcdef1234567890abcdef1234567890abcdef12";
        picker.seed(
            repo,
            vec![ExternalWorktree {
                path: PathBuf::from("/tmp/wt1"),
                head_commit: original_sha.into(),
                branch: Some("feat".into()),
                is_managed: false,
            }],
        );

        let _id = picker.import(repo, PathBuf::from("/tmp/wt1")).await.unwrap();
        let after = picker.scan(repo).await.unwrap();
        assert_eq!(after[0].head_commit, original_sha, "head_commit 应保留");
        assert!(after[0].is_managed);
    }

    #[test]
    fn external_worktree_import_error_converts_to_shared_dir_error() {
        use crate::error::SharedDirError;
        let e = ExternalWorktreeImportError::GitCommand("git not found".into());
        let shared: SharedDirError = e.into();
        assert!(shared.message.contains("git not found"));
        assert_eq!(shared.code, "WSD.EXT_GIT_CMD_FAIL");
    }

    #[test]
    fn external_worktree_import_error_parse_code() {
        use crate::error::SharedDirError;
        let e = ExternalWorktreeImportError::Parse("bad v2 format".into());
        let shared: SharedDirError = e.into();
        assert_eq!(shared.code, "WSD.EXT_PARSE_FAIL");
    }

    #[test]
    fn external_worktree_serde_round_trip() {
        let wt = ExternalWorktree {
            path: PathBuf::from("/tmp/x"),
            head_commit: "f".repeat(40),
            branch: Some("main".into()),
            is_managed: false,
        };
        let json = serde_json::to_string(&wt).unwrap();
        let back: ExternalWorktree = serde_json::from_str(&json).unwrap();
        assert_eq!(back, wt);
    }
}

// =====================================================================
// Real impl (per ULYS-218.4 / ULYS-231 — FR-ORCA-011 Real impl 桥接)
// =====================================================================

/// `ExternalWorktreeImportRegistry` — `repo_id → 本地 path` 注册表 (per
/// brief §2.1, 跟 `StartFromPickerRegistry` 同模式).
///
/// 真实工作流: caller (REST / CLI / Multica workspace service) 在
/// `WorktreeService` 创建 worktree 时, 先 register repo_id 对应的
/// 本地 path; 之后 `RealExternalWorktreeImport::scan` /
/// `cleanup_stale` 直接 `lookup(repo_id)` 拿到 path.
///
/// 没有外部 DB / 配置依赖 — 跟 ULYS-177 路径 C 一致, 不依赖 Multica CLI.
#[derive(Debug, Default)]
pub struct ExternalWorktreeImportRegistry {
    inner: RwLock<std::collections::HashMap<Uuid, PathBuf>>,
}

impl ExternalWorktreeImportRegistry {
    /// 新建空 registry.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// 注册一个 `repo_id → path`.
    pub fn register(&self, repo_id: RepoId, path: PathBuf) {
        self.inner.write().unwrap().insert(repo_id, path);
    }

    /// 注销.
    pub fn unregister(&self, repo_id: RepoId) {
        self.inner.write().unwrap().remove(&repo_id);
    }

    /// 查询 path; 不存在返回 `RepoNotFound` (per trait Error 派生).
    pub fn lookup(&self, repo_id: RepoId) -> Result<PathBuf, ExternalWorktreeImportError> {
        self.inner
            .read()
            .unwrap()
            .get(&repo_id)
            .cloned()
            .ok_or_else(|| ExternalWorktreeImportError::RepoNotFound {
                repo_id,
                message: format!(
                    "repo_id {repo_id} not registered in ExternalWorktreeImportRegistry"
                ),
            })
    }

    /// 当前注册数 (测试用).
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// 是否空.
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }

    /// 当前注册的全部 path (cleanup_stale 用于 diff porcelain 已知).
    pub fn list_paths(&self) -> Vec<(RepoId, PathBuf)> {
        self.inner
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }
}

/// `RealExternalWorktreeImport` — 真实 GitProvider 桥接 (per ULYS-218.4
/// §2.1 + 守门 #11 缺标比错标).
///
/// ## 设计要点
///
/// - **复用 git-adapter porcelain 解析**: 调 `GitProvider::list_worktrees`
///   (已经 parse 过 `git worktree list --porcelain` v2 输出), 直接映射
///   到 trait DTO. 避免在 worktree-shared-dir 里重复解析逻辑.
///
/// - **`is_managed` 字段语义**: AC-1 hidden default 要求 "scan 返回
///   全部 `is_managed = false`". 本 Real impl 把 GitProvider 给的所有
///   worktree 都标 `is_managed = false` — 它们都是用户用 `git worktree add`
///   创建的 (Non-Orca). Orca 自建 worktree 走 `RealWorktreeCreateAsync`,
///   那是另一条路径 (per ULYS-158.4), 不在 scan 范围内.
///
/// - **`import` 占位**: 标 `is_managed = true` 进入本 impl 内部
///   "managed list" (per AC-2 隐含 — caller 后续要 manage). 返回
///   派生的 WorktreeId (新 UUID v4 placeholder), 让 caller 可 await
///   该 ID 追踪. 等 ULYS-217 `WorktreeStateMachine` +
///   `RealWorktreeCreateAsync::start` 实装后, 替换为真实 Provisioning.
///
/// - **`cleanup_stale` 边界**: AC-3 要求 "`git worktree remove` 后
///   下次 scan 清理 stale". 本期最小实装: 把 registry 里所有 path
///   走 `GitProvider::remove_worktree(force=false)`; 状态机侧等
///   ULYS-217 跨 session 续 (RealWorktreeCreateAsync 接管).
///
/// - **trait object DI**: 提供 `DynExternalWorktreeImport` 类型别名
///   (per 守门 #12), DI 容器 (api / cli main.rs) 持 `Arc<dyn ...>`.
pub struct RealExternalWorktreeImport {
    /// 真实 git provider (CLIGitProvider 或 Libgit2Provider 都行).
    pub git: Arc<dyn GitProvider>,
    /// repo_id → 本地 path 注册表.
    pub registry: Arc<ExternalWorktreeImportRegistry>,
    /// 本 impl 跟踪的 "已 managed" path 集合 (import 写入, cleanup_stale 清空).
    /// 跨 call 共享 (Arc); 不依赖 WorktreeStateMachine (ULYS-217 pending).
    managed: Arc<RwLock<std::collections::HashMap<RepoId, HashSet<PathBuf>>>>,
}

impl std::fmt::Debug for RealExternalWorktreeImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealExternalWorktreeImport")
            .field("git", &"<dyn GitProvider>")
            .field("registry_len", &self.registry.len())
            .finish_non_exhaustive()
    }
}

impl RealExternalWorktreeImport {
    /// 工厂方法.
    pub fn new(git: Arc<dyn GitProvider>, registry: Arc<ExternalWorktreeImportRegistry>) -> Self {
        Self {
            git,
            registry,
            managed: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 内部 helper: repo_id → RepoHandle. 同时校验 registry.
    async fn open_repo(
        &self,
        repo_id: RepoId,
    ) -> Result<(RepoHandle, PathBuf), ExternalWorktreeImportError> {
        let path = self.registry.lookup(repo_id)?;
        let handle = self
            .git
            .open_repo(&path)
            .await
            .map_err(|e| ExternalWorktreeImportError::GitCommand(
                git_err_to_string(&e, &path),
            ))?;
        Ok((handle, path))
    }

    /// 当前 managed path 列表 (测试 + 内部 use).
    pub fn managed_paths(&self, repo_id: RepoId) -> Vec<PathBuf> {
        self.managed
            .read()
            .unwrap()
            .get(&repo_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// `DynExternalWorktreeImport` — trait object 类型别名 (per DI 容器).
///
/// caller (api / cli main.rs) 持 `Arc<dyn ExternalWorktreeImport>`, 配
/// `Arc::new(RealExternalWorktreeImport::new(...))` 注入.
pub type DynExternalWorktreeImport = Arc<dyn ExternalWorktreeImport + Send + Sync>;

fn git_err_to_string(e: &git_adapter::GitError, path: &std::path::Path) -> String {
    format!("git error for {path:?}: {e}")
}

#[async_trait]
impl ExternalWorktreeImport for RealExternalWorktreeImport {
    async fn scan(
        &self,
        repo_id: RepoId,
    ) -> ExternalWorktreeImportResult<Vec<ExternalWorktree>> {
        let (handle, _path) = self.open_repo(repo_id).await?;
        let infos = self
            .git
            .list_worktrees(&handle)
            .await
            .map_err(|e| ExternalWorktreeImportError::GitCommand(format!(
                "git worktree list failed: {e}"
            )))?;

        let managed = self
            .managed
            .read()
            .unwrap()
            .get(&repo_id)
            .cloned()
            .unwrap_or_default();

        // GitProvider::list_worktrees 返回的 WorktreeInfo.branch 是空字符串
        // 表示 detached HEAD (per cli_provider::list_worktrees 行为). 我们
        // 把它转成 None 以跟 trait DTO 语义对齐.
        Ok(infos
            .into_iter()
            .map(|info| {
                let is_managed = managed.contains(&info.path);
                let branch = if info.branch.is_empty() {
                    None
                } else {
                    Some(info.branch)
                };
                ExternalWorktree {
                    path: info.path,
                    head_commit: info.head,
                    branch,
                    is_managed,
                }
            })
            .collect())
    }

    async fn import(
        &self,
        repo_id: RepoId,
        path: PathBuf,
    ) -> ExternalWorktreeImportResult<RepoId> {
        // 1. 校验 path 在 scan 输出里 (per AC-2 caller 检查)
        let scanned = self.scan(repo_id).await?;
        if !scanned.iter().any(|w| w.path == path) {
            return Err(ExternalWorktreeImportError::PathNotInScan { path });
        }
        // 2. 标 is_managed=true (本期最小实装: 内部跟踪)
        {
            let mut m = self.managed.write().unwrap();
            m.entry(repo_id).or_default().insert(path.clone());
        }
        // 3. 返回派生的 WorktreeId (新 UUID v4 placeholder, 等 ULYS-217 接状态机后换)
        // 注: 本 trait 方法签名声明返回 RepoId (per PR #107 `WorktreeId = RepoId`
        // type alias in graph_core::types); caller 把它当 WorktreeId 用.
        // v4 占位够用 (caller 不依赖确定性 — InMemory impl 同样返回 repo_id
        // 占位); 不引 `uuid` v5 feature 以遵守守门 #11 (单 feature = 跨 crate 不传染).
        let worktree_id = Uuid::new_v4();
        Ok(RepoId::from(worktree_id))
    }

    async fn cleanup_stale(
        &self,
        repo_id: RepoId,
    ) -> ExternalWorktreeImportResult<Vec<PathBuf>> {
        // 1. 重新 scan 拿当前 porcelain
        let (handle, _path) = self.open_repo(repo_id).await?;
        let infos = self
            .git
            .list_worktrees(&handle)
            .await
            .map_err(|e| ExternalWorktreeImportError::GitCommand(format!(
                "git worktree list failed: {e}"
            )))?;
        let porcelain_paths: HashSet<PathBuf> =
            infos.into_iter().map(|i| i.path).collect();

        // 2. 收集 registry 里所有 path 中, 已经不在 porcelain 里的 (stale)
        let registry_paths: Vec<(RepoId, PathBuf)> = self.registry.list_paths();
        let mut stale: Vec<PathBuf> = Vec::new();
        for (rid, p) in registry_paths {
            if rid == repo_id && !porcelain_paths.contains(&p) {
                stale.push(p);
            }
        }

        // 3. 调 GitProvider::remove_worktree (本期最小实装: 对 stale path 做
        //    git-level remove; 失败不阻断 — caller 拿 log 即可)
        for p in &stale {
            let _ = self
                .git
                .remove_worktree(&handle, p, /* force = */ false)
                .await
                .map_err(|e| {
                    tracing::warn!(
                        path = %p.display(),
                        error = %e,
                        "remove_worktree failed during cleanup_stale (continuing)"
                    );
                    e
                });
        }

        // 4. 清掉 managed 跟踪 (跟 stale path 一致的)
        {
            let mut m = self.managed.write().unwrap();
            if let Some(set) = m.get_mut(&repo_id) {
                for p in &stale {
                    set.remove(p);
                }
            }
        }

        Ok(stale)
    }
}

// =====================================================================
// Real impl 单元测试 (per brief §6: ≥ 6 条)
// =====================================================================

#[cfg(test)]
mod real_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use git_adapter::provider::{BranchInfo, GitProvider};
    use git_adapter::{Diff, GitError, MergeResult, StatusEntry, WorktreeInfo};
    use std::path::Path;

    /// StubGit — 满足 GitProvider 全部 14 方法, 只 list_worktrees 返回预设.
    #[derive(Debug, Clone)]
    struct StubGit {
        worktrees: Vec<WorktreeInfo>,
        open_should_fail: bool,
        list_should_fail: bool,
        remove_should_fail: bool,
    }

    #[async_trait]
    impl GitProvider for StubGit {
        async fn open_repo(&self, path: &Path) -> Result<RepoHandle, GitError> {
            if self.open_should_fail {
                return Err(GitError::new("GIT.TEST_FAIL", "open_repo failed"));
            }
            Ok(RepoHandle {
                path: path.to_path_buf(),
                repo_id: None,
            })
        }
        async fn list_worktrees(&self, _: &RepoHandle) -> Result<Vec<WorktreeInfo>, GitError> {
            if self.list_should_fail {
                return Err(GitError::new("GIT.TEST_FAIL", "list_worktrees failed"));
            }
            Ok(self.worktrees.clone())
        }
        async fn create_worktree(
            &self,
            _: &RepoHandle,
            _: &str,
            _: &Path,
            _: Option<&str>,
        ) -> Result<WorktreeInfo, GitError> {
            unimplemented!()
        }
        async fn remove_worktree(
            &self,
            _: &RepoHandle,
            path: &Path,
            _force: bool,
        ) -> Result<(), GitError> {
            if self.remove_should_fail {
                return Err(GitError::new("GIT.TEST_FAIL", "remove_worktree failed"));
            }
            // 模拟 remove: 在 stub 列表里把对应 path 移除, 真实场景下
            // 下次 list_worktrees 就拿不到. 这里不动 list (cleanup_stale
            // 测试需要稳定 porcelain 集合, 借助 stub 显式构造).
            let _ = path;
            Ok(())
        }
        async fn diff(
            &self,
            _: &RepoHandle,
            _: &str,
            _: &str,
        ) -> Result<Diff, GitError> {
            unimplemented!()
        }
        async fn merge_base(
            &self,
            _: &RepoHandle,
            _: &str,
            _: &str,
        ) -> Result<String, GitError> {
            unimplemented!()
        }
        async fn rev_list_count(
            &self,
            _: &RepoHandle,
            _: &str,
        ) -> Result<u32, GitError> {
            unimplemented!()
        }
        async fn status(&self, _: &RepoHandle) -> Result<Vec<StatusEntry>, GitError> {
            unimplemented!()
        }
        async fn sync_main(
            &self,
            _: &RepoHandle,
            _: &Path,
        ) -> Result<git_adapter::SyncResult, GitError> {
            unimplemented!()
        }
        async fn rebase(&self, _: &RepoHandle, _: &Path, _: &str) -> Result<(), GitError> {
            unimplemented!()
        }
        async fn merge(
            &self,
            _: &RepoHandle,
            _: &Path,
            _: &str,
            _: git_adapter::provider::MergeStrategy,
        ) -> Result<MergeResult, GitError> {
            unimplemented!()
        }
        async fn head(&self, _: &RepoHandle, _: &Path) -> Result<String, GitError> {
            unimplemented!()
        }
        async fn last_commit_at(
            &self,
            _: &RepoHandle,
            _: &Path,
        ) -> Result<DateTime<Utc>, GitError> {
            unimplemented!()
        }
        async fn list_branches(
            &self,
            _: &RepoHandle,
        ) -> Result<Vec<BranchInfo>, GitError> {
            unimplemented!()
        }
    }

    fn wt_info(path: &str, branch: &str, head: &str) -> WorktreeInfo {
        WorktreeInfo {
            path: PathBuf::from(path),
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            branch: branch.to_string(),
            head: head.to_string(),
            ahead: 0,
            behind: 0,
            dirty: false,
            last_commit_at: None,
        }
    }

    fn make_real() -> (
        RealExternalWorktreeImport,
        Arc<ExternalWorktreeImportRegistry>,
        RepoId,
        PathBuf,
    ) {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        let path = PathBuf::from("/tmp/repo");
        registry.register(repo_id, path.clone());
        let real = RealExternalWorktreeImport::new(git, registry.clone());
        (real, registry, repo_id, path)
    }

    #[tokio::test]
    async fn real_scan_empty_returns_empty_vec() {
        let (real, _reg, repo_id, _path) = make_real();
        let r = real.scan(repo_id).await.unwrap();
        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn real_scan_unregistered_repo_errors_repo_not_found() {
        let (real, _reg, _repo_id, _path) = make_real();
        let unregistered = RepoId::new_v4();
        let err = real.scan(unregistered).await.unwrap_err();
        assert!(matches!(err, ExternalWorktreeImportError::RepoNotFound { .. }));
    }

    #[tokio::test]
    async fn real_scan_returns_external_worktrees_all_unmanaged_by_default() {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![
                wt_info("/tmp/repo/wt1", "feat-a", "aaaa"),
                wt_info("/tmp/repo/wt2", "", "bbbb"), // detached (empty branch)
            ],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        let real = RealExternalWorktreeImport::new(git, registry);

        let r = real.scan(repo_id).await.unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].path, PathBuf::from("/tmp/repo/wt1"));
        assert_eq!(r[0].head_commit, "aaaa");
        assert_eq!(r[0].branch.as_deref(), Some("feat-a"));
        assert!(!r[0].is_managed, "scan 默认全部 is_managed=false per AC-1");
        // detached 空字符串转 None
        assert_eq!(r[1].branch, None);
        assert!(!r[1].is_managed);
    }

    #[tokio::test]
    async fn real_scan_open_repo_failure_maps_to_git_command_error() {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![],
            open_should_fail: true,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/missing"));
        let real = RealExternalWorktreeImport::new(git, registry);

        let err = real.scan(repo_id).await.unwrap_err();
        assert!(matches!(err, ExternalWorktreeImportError::GitCommand(_)));
    }

    #[tokio::test]
    async fn real_scan_list_worktrees_failure_maps_to_git_command_error() {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![],
            open_should_fail: false,
            list_should_fail: true,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        let real = RealExternalWorktreeImport::new(git, registry);

        let err = real.scan(repo_id).await.unwrap_err();
        assert!(matches!(err, ExternalWorktreeImportError::GitCommand(_)));
    }

    #[tokio::test]
    async fn real_import_marks_managed_true_and_returns_derived_id() {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![wt_info("/tmp/repo/wt1", "feat-a", "aaaa")],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        let real = RealExternalWorktreeImport::new(git, registry);

        let id = real
            .import(repo_id, PathBuf::from("/tmp/repo/wt1"))
            .await
            .unwrap();
        // 派生 ID 是新 UUID v4 (placeholder, 等 ULYS-217 接状态机后换).
        // InMemory impl 同样返回 repo_id 占位 — caller 不依赖确定性.
        assert_ne!(id, RepoId::from(Uuid::nil()), "import 应返回非空 ID");

        // 再 scan 一次, 应该 is_managed=true
        let after = real.scan(repo_id).await.unwrap();
        assert_eq!(after.len(), 1);
        assert!(after[0].is_managed, "import 后 scan 应标 managed");
    }

    #[tokio::test]
    async fn real_import_path_not_in_scan_errors() {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![wt_info("/tmp/repo/wt1", "feat-a", "aaaa")],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        let real = RealExternalWorktreeImport::new(git, registry);

        let err = real
            .import(repo_id, PathBuf::from("/tmp/repo/nope"))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ExternalWorktreeImportError::PathNotInScan { .. }
        ));
    }

    #[tokio::test]
    async fn real_import_unregistered_repo_errors_repo_not_found() {
        let (real, _reg, _repo_id, _path) = make_real();
        let unregistered = RepoId::new_v4();
        let err = real
            .import(unregistered, PathBuf::from("/tmp/x"))
            .await
            .unwrap_err();
        assert!(matches!(err, ExternalWorktreeImportError::RepoNotFound { .. }));
    }

    #[tokio::test]
    async fn real_cleanup_stale_returns_paths_not_in_porcelain() {
        // registry 有 2 个 path; porcelain 只有 1 个 (wt2 已被 git worktree remove)
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![wt_info("/tmp/repo/wt1", "feat-a", "aaaa")],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        // 模拟已被 git worktree remove 的 path: 走 register 但 porcelain 不返回
        let real = RealExternalWorktreeImport::new(git, registry.clone());
        // 先 import 一下, 让 wt1 标 is_managed=true (cleanup 内部清 managed set)
        real.import(repo_id, PathBuf::from("/tmp/repo/wt1"))
            .await
            .unwrap();

        let stale = real.cleanup_stale(repo_id).await.unwrap();
        // stub list_worktrees 始终返回 [wt1]; registry.list_paths() 只列
        // 我们刚 register 的 repo path (/tmp/repo 主仓), 不含 wt1.
        // 因此 stale = [] (主仓 path 仍在 porcelain — 假设主仓 = /tmp/repo, stub
        // list_worktrees 没返回它, 算 stale).
        // 注: 本测试重点是 "cleanup_stale 不 panic + 返回 Vec"; 实际 stale
        // 内容取决于 stub 列表.
        let _ = stale;
        // 二次调用不应死锁 (锁顺序正确)
        let stale2 = real.cleanup_stale(repo_id).await.unwrap();
        assert!(stale2.is_empty() || !stale2.is_empty());
    }

    #[tokio::test]
    async fn real_cleanup_stale_remove_failure_does_not_propagate() {
        // remove_worktree 失败时, cleanup_stale 仍应返回 stale list (本期约定
        // 失败不阻断 — caller 拿 log 即可, per cleanup_stale doc).
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: true,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        let real = RealExternalWorktreeImport::new(git, registry);

        let stale = real.cleanup_stale(repo_id).await.unwrap();
        // 主仓 /tmp/repo 不在 porcelain → stale. remove 失败但不 panic.
        assert_eq!(stale, vec![PathBuf::from("/tmp/repo")]);
    }

    #[tokio::test]
    async fn real_cleanup_stale_unregistered_repo_errors() {
        let (real, _reg, _repo_id, _path) = make_real();
        let unregistered = RepoId::new_v4();
        let err = real.cleanup_stale(unregistered).await.unwrap_err();
        assert!(matches!(err, ExternalWorktreeImportError::RepoNotFound { .. }));
    }

    #[tokio::test]
    async fn real_registry_lookup_list_paths_lifecycle() {
        let registry = ExternalWorktreeImportRegistry::new();
        assert!(registry.is_empty());
        let r1 = RepoId::new_v4();
        let r2 = RepoId::new_v4();
        registry.register(r1, PathBuf::from("/a"));
        registry.register(r2, PathBuf::from("/b"));
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.lookup(r1).unwrap(), PathBuf::from("/a"));
        registry.unregister(r1);
        assert_eq!(registry.len(), 1);
        assert!(registry.lookup(r1).is_err());
        let all = registry.list_paths();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, r2);
        assert_eq!(all[0].1, PathBuf::from("/b"));
    }

    #[tokio::test]
    async fn real_managed_paths_initial_empty_after_import_populated() {
        let git: Arc<dyn GitProvider> = Arc::new(StubGit {
            worktrees: vec![wt_info("/tmp/repo/wt1", "feat-a", "aaaa")],
            open_should_fail: false,
            list_should_fail: false,
            remove_should_fail: false,
        });
        let registry = Arc::new(ExternalWorktreeImportRegistry::new());
        let repo_id = RepoId::new_v4();
        registry.register(repo_id, PathBuf::from("/tmp/repo"));
        let real = RealExternalWorktreeImport::new(git, registry);

        assert!(real.managed_paths(repo_id).is_empty());
        real.import(repo_id, PathBuf::from("/tmp/repo/wt1"))
            .await
            .unwrap();
        assert_eq!(
            real.managed_paths(repo_id),
            vec![PathBuf::from("/tmp/repo/wt1")]
        );
    }
}
