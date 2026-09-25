//! `external_worktree_import.rs` — ULYS-177.4 / ULYS-195 (FR-ORCA-011).
//!
//! 检测用户用原生 `git worktree add` 在仓库下创建、未经过
//! `WorktreeService::create` 的 worktree, 并把它们导入到
//! `WorktreeService` 的 in-memory + (阶段 2) DB projection 里.
//!
//! ## 触发场景
//!
//! 1. `app_startup` hook — 启动时对每个 repo 调 `import_all`
//! 2. CLI `multica worktree list` — 列前先扫
//! 3. Frontend `/api/v1/worktree-import/scan?repo_id=X`
//! 4. 定时 cron (每 5 分钟防漏检)
//!
//! ## 冲突处理
//!
//! - 默认: 同 branch 已存在则 `skip` + `tracing::warn!`
//! - `--force` (per FR-ORCA-011 AC): 更新内部 Worktree.branch 指向新 path
//!
//! ## 模块边界
//!
//! - 本模块定义: `ExternalWorktree` 结构, `scan_external_worktrees` (调 git),
//!   `parse_porcelain`, `diff_external`, `map_to_worktree` (字段映射 helper).
//! - 真正的"导入"逻辑 (写 WorktreeService 内部状态) 通过 `WorktreeService::import_external_worktrees`
//!   新增 trait 方法 (方法 #12 per ULYS-195 §5) 在 `service_impl.rs` 实现.

use std::path::{Path, PathBuf};

use chrono::Utc;
use graph_core::state::{HumanState, TestState};
use graph_core::types::{RepoId, UserId, WorktreeId};
use uuid::Uuid;

use crate::error::ServiceError;
use crate::service::Worktree;

/// 一个外部 worktree（用户用 `git worktree add` 创建的）.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalWorktree {
    /// `.git/worktrees/<name>` 路径
    pub git_path: PathBuf,
    /// 实际 worktree 工作目录
    pub worktree_path: PathBuf,
    /// HEAD commit SHA (full 40 hex chars)
    pub head_commit: String,
    /// 当前分支 (`None` = detached HEAD)
    pub branch: Option<String>,
    /// `.git/worktrees/<name>/locked` 内容（`None` = 未锁）
    pub locked: Option<String>,
    /// `git worktree prune` 标记 (porcelain v2 `prunable git reason...`)
    pub prunable: bool,
}

/// Import 错误 (per FR-ORCA-011 AC).
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// 调 `git worktree list` 失败 (binary 不在 PATH, IO 失败, etc.)
    #[error("git worktree list failed: {0}")]
    GitCommand(String),
    /// porcelain 输出解析失败 (非预期 v2 格式)
    #[error("failed to parse porcelain output: {0}")]
    Parse(String),
    /// IO 错误
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl ImportError {
    /// 转成 ServiceError (WT.GIT_FAIL 或 WT.IO_FAIL).
    pub fn into_service_error(self, trace_id: impl Into<String>) -> ServiceError {
        let trace_id = trace_id.into();
        match self {
            ImportError::GitCommand(msg) => ServiceError::new("WT.GIT_FAIL", msg, trace_id)
                .with_location(
                    "external_worktree_import.rs:scan_external_worktrees".to_string(),
                ),
            ImportError::Parse(msg) => {
                ServiceError::new("WT.GIT_FAIL", format!("porcelain parse failed: {msg}"), trace_id)
                    .with_location("external_worktree_import.rs:parse_porcelain".to_string())
            }
            ImportError::Io(e) => ServiceError::new("WT.IO_FAIL", e.to_string(), trace_id)
                .with_location("external_worktree_import.rs".to_string()),
        }
    }
}

/// 扫描一个 repo 下的全部 `git worktree list --porcelain` 条目
/// (FR-ORCA-011 AC-1).
///
/// 调 git CLI, 解析 porcelain v2 输出 (per git 官方文档, 稳定于 git 2.34+).
pub async fn scan_external_worktrees(repo_path: &Path) -> Result<Vec<ExternalWorktree>, ImportError> {
    let output = tokio::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .await
        .map_err(|e| ImportError::GitCommand(format!("spawn failed: {e}")))?;

    if !output.status.success() {
        return Err(ImportError::GitCommand(format!(
            "exit={:?} stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_porcelain(&stdout)
}

/// 解析 `git worktree list --porcelain` 输出.
///
/// ## porcelain v2 格式 (per `git-worktree` manpage, 2.34+):
///
/// ```text
/// worktree /path/to/wt1
/// HEAD abc123...
/// branch refs/heads/feat-x
///
/// worktree /path/to/wt2
/// HEAD def789...
/// detached
///
/// worktree /path/to/wt3
/// HEAD 012abc...
/// branch refs/heads/main
/// locked reason: rebasing in progress
/// prunable git: error: unable to rmdir 'wt3'
/// ```
///
/// 每条 entry 由 `\n\n` 分隔; 每个 entry 的首行必为 `worktree <path>`.
pub fn parse_porcelain(stdout: &str) -> Result<Vec<ExternalWorktree>, ImportError> {
    // 平台兼容: Windows 上 git CLI 可能输出 `\r\n` 行尾 (尤其 `core.autocrlf=true` 配置下),
    // `\n\n` split 会把 CRLF 边界粘合为单一 block, 丢失中间的 worktree.
    // 归一化 `\r\n` → `\n` 后再 split (per ULYS-195 stage 2 review finding #1).
    let normalized = stdout.replace("\r\n", "\n");
    let mut out = Vec::new();
    for block in normalized.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let mut wt_path: Option<PathBuf> = None;
        let mut head: Option<String> = None;
        let mut branch: Option<String> = None;
        let mut locked: Option<String> = None;
        let mut prunable = false;

        for line in block.lines() {
            if let Some(rest) = line.strip_prefix("worktree ") {
                wt_path = Some(PathBuf::from(rest));
            } else if let Some(rest) = line.strip_prefix("HEAD ") {
                head = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("branch ") {
                // refs/heads/feat-x → feat-x
                let name = rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string();
                branch = Some(name);
            } else if line == "detached" {
                branch = None;
            } else if line == "locked" || line.starts_with("locked ") {
                // `locked` 或 `locked <reason>`
                let rest = line.strip_prefix("locked ").unwrap_or("");
                locked = Some(rest.to_string());
            } else if line.starts_with("prunable") {
                prunable = true;
            }
        }

        let worktree_path = wt_path.ok_or_else(|| {
            ImportError::Parse(format!("missing 'worktree' line in block: {block}"))
        })?;
        let head_commit =
            head.ok_or_else(|| ImportError::Parse(format!("missing 'HEAD' line in block: {block}")))?;

        // git_path 推导: `.git/worktrees/<basename(worktree_path)>`
        let basename = worktree_path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string());
        let git_path = PathBuf::from(format!(".git/worktrees/{basename}"));

        out.push(ExternalWorktree {
            git_path,
            worktree_path,
            head_commit,
            branch,
            locked,
            prunable,
        });
    }

    Ok(out)
}

/// 找出 **不在** `existing` 内的外部条目 (按 `worktree_path` 完全相等判断).
pub fn diff_external(external: &[ExternalWorktree], existing: &[Worktree]) -> Vec<ExternalWorktree> {
    let known: std::collections::HashSet<PathBuf> =
        existing.iter().map(|w| w.path.clone()).collect();
    external
        .iter()
        .filter(|e| !known.contains(&e.worktree_path))
        .cloned()
        .collect()
}

/// `ExternalWorktree` → `Worktree` 字段映射 (FR-ORCA-011 AC-2).
///
/// - `branch` (or detached commit short SHA) → branch
/// - `worktree_path` → path
/// - 默认 `human_state: Running` (git 层 OK)
pub fn map_to_worktree(repo_id: RepoId, ext: &ExternalWorktree) -> Worktree {
    let now = Utc::now();
    let branch = ext.branch.clone().unwrap_or_else(|| {
        ext.head_commit.chars().take(7).collect::<String>()
    });
    let name = ext
        .worktree_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| branch.clone());

    Worktree {
        id: WorktreeId::from(Uuid::new_v4()),
        repo_id,
        name,
        branch,
        path: ext.worktree_path.clone(),
        human_state: HumanState::Running,
        test_state: TestState::None,
        ahead: 0,
        behind: 0,
        dirty: false,
        locked: ext.locked.is_some(),
        archived: false,
        health_score: 100,
        last_activity: now,
        created_at: now,
        merged_at: None,
        locked_by: if ext.locked.is_some() {
            Some(UserId::from(Uuid::new_v4()))
        } else {
            None
        },
        agent_type: Some("external_import".to_string()),
    }
}

/// `ImportOutcome` — `import_external_worktrees` 服务的细粒度返回
/// (per FR-ORCA-011 AC-3): caller 可区分 imported / skipped / updated。
///
/// `Worktree` 不 derive `PartialEq` (per stage 1 service 设计), 故
/// `ImportOutcome` 只 manual / manual auto — caller 用 `total_changed` 判定.
#[derive(Debug, Clone, Default)]
pub struct ImportOutcome {
    /// 新插入的 (not in WorktreeService 内部状态).
    pub imported: Vec<Worktree>,
    /// 同 branch 已存在, 因未 force 而跳过.
    pub skipped: Vec<String>,
    /// force=true 时, 已更新 path/branch 的内部 Worktree.
    pub updated: Vec<Worktree>,
}

impl ImportOutcome {
    /// 总成功数 (imported + updated).
    pub fn total_changed(&self) -> usize {
        self.imported.len() + self.updated.len()
    }
}

// =====================================================================
// 阶段 2: 触发场景 helper + post-import hook (per ULYS-195 §3.3 + §6)
// =====================================================================
//
// `WorktreeService::import_external_worktrees` 已是单次 trait 方法; 阶段 2
// 抽出三件辅助:
//   1. `scan_for_repo` / `scan_all_repos` — 给 4 触发场景 (app_startup /
//      CLI `multica worktree list` / UI `/api/v1/worktree-import/scan?repo_id=X` /
//      cron 每 5 分钟) 的统一 entry point
//   2. `PostImportHook` trait — 让 `InMemoryWorktreeService` 在 import 成功后
//      回调 (e.g. SharedDirResolver 重算共享目录 symlinks)
//   3. `NoopPostImportHook` — 默认空 hook, 保持 stage 1 行为不破坏测试
//
// 故意**不**实装:
//   - PG-backed persistence: 走 stage 2 PR 拆分, 不在本 PR 范围 (per D-Boy 9/24 拍板)
//   - cron infra: 仓库暂无 scheduler, 留 P1 followup
//   - CLI/UI/REST 接线: 跨 crate, 走 stage 2 PR 拆分

/// 单 repo 导入入口 (per ULYS-195 §6 trigger scenario helper).
///
/// 给 CLI / UI / cron / app_startup 4 类 caller 的统一入口 — 内部直接转
/// `WorktreeService::import_external_worktrees` trait 方法, 强制走 service
/// 抽象层 (不绕过 trait 直接 scan + diff).
///
/// `repo_path` 是 git 仓库根目录 (即 `git rev-parse --show-toplevel` 的输出).
pub async fn scan_for_repo<S: crate::service::WorktreeService + ?Sized>(
    service: &S,
    repo_id: RepoId,
    repo_path: &Path,
    force: bool,
) -> Result<ImportOutcome, ServiceError> {
    service
        .import_external_worktrees(repo_id, repo_path, force)
        .await
}

/// `RepoDescriptor` — `scan_all_repos` 接受的输入 (per ULYS-195 §3.3 cron
/// 定时扫所有 repo 防漏检).
///
/// 仓库路径就是 git 根; 不引入额外 metadata (per FR-ORCA-011 AC-3 "批量
/// 扫描 + 导入" 简化为 (repo_id, repo_path) 二元组).
#[derive(Debug, Clone)]
pub struct RepoDescriptor {
    /// Repo UUID (per graph-core `RepoId`)
    pub repo_id: RepoId,
    /// git 仓库根路径 (per `git rev-parse --show-toplevel`)
    pub repo_path: PathBuf,
}

/// 批量扫描入口 (per ULYS-195 §3.3 + AC-3).
///
/// 对每个 repo 调 `scan_for_repo`, 错误不阻断后续 repo — 汇总到
/// `Vec<(RepoId, ServiceError)>`. 任一 repo 成功 = 全部 `ImportOutcome` 拼成
/// 一个 vector 返回.
///
/// caller (e.g. cron / app_startup) 可逐 repo 看 errors, 不需要整个批次
/// 失败即重试全部.
pub async fn scan_all_repos<S: crate::service::WorktreeService + ?Sized>(
    service: &S,
    repos: &[RepoDescriptor],
    force: bool,
) -> (Vec<(RepoId, ImportOutcome)>, Vec<(RepoId, ServiceError)>) {
    let mut outcomes = Vec::with_capacity(repos.len());
    let mut errors = Vec::new();
    for repo in repos {
        match scan_for_repo(service, repo.repo_id, &repo.repo_path, force).await {
            Ok(o) => outcomes.push((repo.repo_id, o)),
            Err(e) => errors.push((repo.repo_id, e)),
        }
    }
    (outcomes, errors)
}

/// `PostImportHook` — import 成功后的回调 (per ULYS-195 §3 软依赖).
///
/// 默认 impl (`NoopPostImportHook`) 是空 hook; 实装 crate 可注入更复杂
/// 逻辑. 例如 `SharedDirResolverHook` 在 import 成功后调
/// `shared_dir_resolver.resolve(repo_id)` 重算共享目录 symlinks.
///
/// 故意**异步**: 避免阻塞 import 主路径, hook 失败不阻断 import 结果
/// (只 warn log). 这是 stage 2 解耦的关键.
#[async_trait::async_trait]
pub trait PostImportHook: Send + Sync {
    /// Import 完成后回调 (per outcome 全部 imported+updated 之后).
    ///
    /// `repo_id` — 触发 import 的 repo
    /// `imported` — 本轮新 insert 的 Worktree (可能空)
    /// `updated` — force=true 时被更新的 Worktree (可能空)
    /// `skipped` — branch 冲突被 skip 的 branch 名 (可能空)
    async fn on_import_complete(
        &self,
        repo_id: RepoId,
        imported: &[Worktree],
        updated: &[Worktree],
        skipped: &[String],
    );
}

/// 默认空 hook — `InMemoryWorktreeService::new()` 用这个, 不破坏 stage 1 测试。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPostImportHook;

#[async_trait::async_trait]
impl PostImportHook for NoopPostImportHook {
    async fn on_import_complete(
        &self,
        _repo_id: RepoId,
        _imported: &[Worktree],
        _updated: &[Worktree],
        _skipped: &[String],
    ) {
        // noop — 不发事件, 不写 DB, 不调 resolver
    }
}

// =====================================================================
// 测试 (per ULYS-195 §3 #5 实装收尾验证)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryWorktreeService;

    #[test]
    fn parse_porcelain_single_with_lock() {
        let input = "worktree /tmp/wt1\nHEAD abc123def456789012345678901234567890abcd\nbranch refs/heads/feat-x\nlocked reason: rebasing in progress\n\n";
        let out = parse_porcelain(input).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].worktree_path, PathBuf::from("/tmp/wt1"));
        assert_eq!(out[0].head_commit, "abc123def456789012345678901234567890abcd");
        assert_eq!(out[0].branch.as_deref(), Some("feat-x"));
        assert_eq!(out[0].locked.as_deref(), Some("reason: rebasing in progress"));
        assert!(!out[0].prunable);
    }

    #[test]
    fn parse_porcelain_detached_head() {
        let input =
            "worktree /tmp/wt-detached\nHEAD 0123456789abcdef0123456789abcdef01234567\ndetached\n\n";
        let out = parse_porcelain(input).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].branch, None);
        assert!(!out[0].prunable);
    }

    #[test]
    fn parse_porcelain_multiple_and_prunable() {
        let input = "worktree /tmp/wt1\nHEAD aaaa\nbranch refs/heads/main\n\nworktree /tmp/wt2\nHEAD bbbb\nbranch refs/heads/dev\nlocked\n\nworktree /tmp/wt3\nHEAD cccc\nbranch refs/heads/feat\nprunable git: error: unable to rmdir 'wt3'\n\n";
        let out = parse_porcelain(input).unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].branch.as_deref(), Some("main"));
        assert_eq!(out[1].branch.as_deref(), Some("dev"));
        assert_eq!(out[1].locked.as_deref(), Some(""));
        assert!(out[2].prunable);
    }

    #[test]
    fn parse_porcelain_empty() {
        let out = parse_porcelain("").unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn parse_porcelain_missing_head_errors() {
        let input = "worktree /tmp/wt\nbranch refs/heads/main\n\n";
        let err = parse_porcelain(input).unwrap_err();
        assert!(matches!(err, ImportError::Parse(_)));
    }

    #[test]
    fn diff_external_filters_known() {
        let ext = vec![
            ExternalWorktree {
                git_path: PathBuf::from(".git/worktrees/a"),
                worktree_path: PathBuf::from("/tmp/a"),
                head_commit: "aaa".into(),
                branch: Some("feat-a".into()),
                locked: None,
                prunable: false,
            },
            ExternalWorktree {
                git_path: PathBuf::from(".git/worktrees/b"),
                worktree_path: PathBuf::from("/tmp/b"),
                head_commit: "bbb".into(),
                branch: Some("feat-b".into()),
                locked: None,
                prunable: false,
            },
        ];
        let existing = vec![Worktree {
            id: WorktreeId::from(Uuid::new_v4()),
            repo_id: Uuid::new_v4(),
            name: "a".into(),
            branch: "feat-a".into(),
            path: PathBuf::from("/tmp/a"),
            human_state: HumanState::Running,
            test_state: TestState::None,
            ahead: 0,
            behind: 0,
            dirty: false,
            locked: false,
            archived: false,
            health_score: 100,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
            locked_by: None,
            agent_type: None,
        }];
        let diff = diff_external(&ext, &existing);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].branch.as_deref(), Some("feat-b"));
    }

    #[test]
    fn map_to_worktree_default_running() {
        let ext = ExternalWorktree {
            git_path: PathBuf::from(".git/worktrees/x"),
            worktree_path: PathBuf::from("/tmp/x"),
            head_commit: "abcdef1234567890abcdef1234567890abcdef12".into(),
            branch: Some("feat-x".into()),
            locked: None,
            prunable: false,
        };
        let repo_id = Uuid::new_v4();
        let wt = map_to_worktree(repo_id, &ext);
        assert_eq!(wt.repo_id, repo_id);
        assert_eq!(wt.branch, "feat-x");
        assert_eq!(wt.path, PathBuf::from("/tmp/x"));
        assert_eq!(wt.human_state, HumanState::Running);
        assert_eq!(wt.test_state, TestState::None);
        assert!(!wt.locked);
        assert_eq!(wt.health_score, 100);
        assert_eq!(wt.agent_type.as_deref(), Some("external_import"));
    }

    #[test]
    fn map_to_worktree_detached_uses_commit_short() {
        let ext = ExternalWorktree {
            git_path: PathBuf::from(".git/worktrees/y"),
            worktree_path: PathBuf::from("/tmp/y"),
            head_commit: "abcdef1234567890abcdef1234567890abcdef12".into(),
            branch: None,
            locked: None,
            prunable: false,
        };
        let wt = map_to_worktree(Uuid::new_v4(), &ext);
        assert_eq!(wt.branch, "abcdef1");
    }

    #[test]
    fn map_to_worktree_locked_sets_locked_owner() {
        let ext = ExternalWorktree {
            git_path: PathBuf::from(".git/worktrees/z"),
            worktree_path: PathBuf::from("/tmp/z"),
            head_commit: "0000000000000000000000000000000000000000".into(),
            branch: Some("feat-z".into()),
            locked: Some("reason: rebasing".into()),
            prunable: false,
        };
        let wt = map_to_worktree(Uuid::new_v4(), &ext);
        assert!(wt.locked);
        assert!(wt.locked_by.is_some());
    }

    #[test]
    fn import_outcome_total_changed_counts_imported_plus_updated() {
        let outcome = ImportOutcome {
            imported: vec![],
            skipped: vec!["feat-x".into()],
            updated: vec![],
        };
        assert_eq!(outcome.total_changed(), 0);

        let wt = Worktree {
            id: WorktreeId::from(Uuid::new_v4()),
            repo_id: Uuid::new_v4(),
            name: "x".into(),
            branch: "feat-x".into(),
            path: PathBuf::from("/tmp/x"),
            human_state: HumanState::Running,
            test_state: TestState::None,
            ahead: 0,
            behind: 0,
            dirty: false,
            locked: false,
            archived: false,
            health_score: 100,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
            locked_by: None,
            agent_type: None,
        };
        let outcome = ImportOutcome {
            imported: vec![wt.clone()],
            skipped: vec![],
            updated: vec![wt],
        };
        assert_eq!(outcome.total_changed(), 2);
    }

    // --- 阶段 2 stage 2 review finding #1: CRLF 平台兼容 (Windows git 输出 `\r\n`) ---

    #[test]
    fn parse_porcelain_crlf_line_endings_windows_git() {
        // 模拟 Windows 上 `git worktree list --porcelain` 用 CRLF 行尾的输出.
        // 2 个 worktree + CRLF: 必须解析出 2 条, 而不是 1 条 (粘合 bug).
        let input = "worktree /tmp/wt1\r\nHEAD abc123\r\nbranch refs/heads/feat-x\r\n\r\nworktree /tmp/wt2\r\nHEAD def456\r\nbranch refs/heads/main\r\n\r\n";
        let out = parse_porcelain(input).unwrap();
        assert_eq!(out.len(), 2, "CRLF 不能粘合 2 个 worktree");
        assert_eq!(out[0].branch.as_deref(), Some("feat-x"));
        assert_eq!(out[1].branch.as_deref(), Some("main"));
    }

    #[test]
    fn parse_porcelain_mixed_crlf_and_lf() {
        // 混合行尾: 头 1 个 worktree 用 LF, 第 2 个用 CRLF.
        let input = "worktree /tmp/wt1\nHEAD abc123\nbranch refs/heads/feat-x\n\nworktree /tmp/wt2\r\nHEAD def456\r\nbranch refs/heads/main\r\n\r\n";
        let out = parse_porcelain(input).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].branch.as_deref(), Some("feat-x"));
        assert_eq!(out[1].branch.as_deref(), Some("main"));
    }

    // --- 阶段 2 stage 2: scan_for_repo / scan_all_repos trigger helpers ---

    #[tokio::test]
    async fn scan_for_repo_calls_import_external_worktrees() {
        // 不需要真 git: 直接用 service trait + 调 scan_for_repo
        // 验证它就是 import_external_worktrees 的 thin wrapper.
        // 这里 mock 通过 InMemoryWorktreeService + 走空 path (会失败,
        // 但我们要确认 scan_for_repo 真的把参数透传过去).
        let svc = InMemoryWorktreeService::new();
        let repo_id = RepoId::new_v4();
        let err = scan_for_repo(&svc, repo_id, Path::new("/nonexistent/repo"), false)
            .await
            .expect_err("nonexistent repo should error");
        // 错误来自 git CLI spawn fail (Windows 上 /nonexistent 不存在
        // 导致 spawn 失败或 git 命令非零退出).
        assert!(
            err.code == "WT.GIT_FAIL" || err.code == "WT.IO_FAIL",
            "expected WT.GIT_FAIL or WT.IO_FAIL, got {}",
            err.code
        );
    }

    #[tokio::test]
    async fn scan_all_repos_continues_on_error() {
        // scan_all_repos 必须不阻断: 一个 repo 错误不影响后续 repo.
        let svc = InMemoryWorktreeService::new();
        let repos = vec![
            RepoDescriptor {
                repo_id: RepoId::new_v4(),
                repo_path: PathBuf::from("/nonexistent/repo-1"),
            },
            RepoDescriptor {
                repo_id: RepoId::new_v4(),
                repo_path: PathBuf::from("/nonexistent/repo-2"),
            },
        ];
        let (outcomes, errors) = scan_all_repos(&svc, &repos, false).await;
        assert!(outcomes.is_empty(), "no successes for nonexistent paths");
        assert_eq!(errors.len(), 2, "both repos should error independently");
    }

    #[tokio::test]
    async fn noop_post_import_hook_runs_without_panic() {
        // Noop hook 调一次不应 panic (给 stage 1 InMemoryWorktreeService 用)
        let hook = NoopPostImportHook;
        let repo_id = RepoId::new_v4();
        let imported: Vec<Worktree> = vec![];
        let updated: Vec<Worktree> = vec![];
        let skipped: Vec<String> = vec![];
        hook.on_import_complete(repo_id, &imported, &updated, &skipped)
            .await;
        // 没 panic = pass
    }
}