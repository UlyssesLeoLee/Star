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
//! 实现走 `worktree-service::scan_external_worktrees` (per FR-ORCA-011 AC-1).

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use graph_core::types::RepoId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
