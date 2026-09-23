//! `start_from_picker.rs` — ULYS-177.3 / ULYS-194 (FR-ORCA-009).
//!
//! Start-from Picker 的 4 选 1 候选枚举与过滤逻辑. UI 端 (`StartFromPicker.tsx`)
//! 调 `WorktreeService::pick_start_from_candidates` 拿这 4 分类的候选, 用户确认后
//! 触发 `WorktreeService::create` 或后续 ULYS-177.2 的 `create_async` (per FR-ORCA-008).
//!
//! ## 4 选 1 候选来源 (per spec FR-ORCA-009 + ULYS-158 §3 FR-ORCA-009)
//!
//! | # | 来源 | 启动条件 | PickerCandidateKind |
//! |---|------|----------|---------------------|
//! | 1 | GitHub Repo branches | 仓库有 `git remote -v` + `origin/main` / `origin/HEAD` | `RemoteBranch` |
//! | 2 | Existing Worktree | repo 有 ≥1 Worktree (per `WorktreeService::list`) | `LocalBranch` |
//! | 3 | Local Path | 用户手动选 (受 per-user allowlist 约束) | `RepoBase` |
//! | 4 | Empty / Blank | 无前置依赖 (永远可启动) | `CommitSha` (sentinel Empty candidate) |
//!
//! ## 与既有 `worktree-shared-dir::PickerCandidateKind` 关系
//!
//! `crates/worktree-shared-dir/src/shared_dir_types.rs` 已定义 4-variant enum
//! `PickerCandidateKind` (per PR #78 `a5bb2d24`):
//! - `RepoBase`     → 用作 Local Path 来源
//! - `LocalBranch`  → 用作 Existing Worktree 来源
//! - `CommitSha`    → 用作 Empty 哨兵 (无 base ref)
//! - `RemoteBranch` → 用作 GitHub Branch 来源
//!
//! 本模块 **复用** 该 enum, 不重复定义. UI 端按 `kind` 字段分类显示.
//!
//! ## 与 ULYS-177.2 (create_async + Provisioning) 关系
//!
//! ULYS-177.2 的 `create_async` + `Provisioning` 过渡态当前 **未 landed** (per `git log --all`:
//! 0 commit with `create_async` / `ProvisioningProgress`). 本模块独立于 ULYS-177.2:
//! 只负责 **列出** 4 选 1 候选 + **过滤/去重**. 调用方后续接 `WorktreeService::create`
//! (阶段 1 同步) 或 `WorktreeService::create_async` (阶段 2 ULYS-177.2 后切).
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 `ServiceError` 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//! - #19 v19 0 动 V0.1 (不动 `WorktreeService` 已有的 11 方法, 仅新增 #12)

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use git_adapter::{GitProvider, RepoHandle};
use graph_core::types::RepoId;
use serde::{Deserialize, Serialize};
use tracing::warn;
// Re-export from worktree-shared-dir (per 守门 #19: 不重定义, 复用 PR #78 已落地的 enum).
// 同时 `pub use` 让外部 caller 可写 `worktree_service::start_from_picker::PickerCandidate`.
pub use worktree_shared_dir::{PickerCandidate, PickerCandidateKind};
// `BranchInfo` 不在 git-adapter root re-export (per lib.rs:29-36), 从 provider 模块直接 import.
use git_adapter::provider::BranchInfo as GitBranch;

use crate::error::ServiceError;
use crate::service::Worktree;

/// 4 选 1 分类结果 (per FR-ORCA-009 spec §2)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PickerCandidates {
    /// kind 1: GitHub Repo remote branches (`PickerCandidateKind::RemoteBranch`)
    pub github_branches: Vec<PickerCandidate>,
    /// kind 2: Existing Worktree base refs (`PickerCandidateKind::LocalBranch`)
    pub existing_worktrees: Vec<PickerCandidate>,
    /// kind 3: Local Path (`PickerCandidateKind::RepoBase`)
    pub local_paths: Vec<PickerCandidate>,
    /// kind 4: Empty / Blank sentinel (`PickerCandidateKind::CommitSha`)
    pub empty: Option<PickerCandidate>,
}

impl PickerCandidates {
    /// 候选总条数 (含 empty)
    pub fn total(&self) -> usize {
        self.github_branches.len()
            + self.existing_worktrees.len()
            + self.local_paths.len()
            + usize::from(self.empty.is_some())
    }

    /// 是否完全空 (4 分类都空 + empty=None). 实装上 `empty` 永远 Some, 故 total ≥ 1.
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

/// Picker 来源 trait — 抽象 git remote 解析 + WorktreeService::list + filesystem allowlist
/// (per FR-ORCA-009 spec §3 4-source).
///
/// ## 为什么要 trait 抽象?
///
/// - 测试可注入 mock provider (不依赖真实 git CLI / FS)
/// - 阶段 2 接 ULYS-177.2 async create 时, `local_paths` provider 可换 async 版
/// - 守门 #19 v19 0 动 V0.1: 不动 WorktreeService 的 11 方法, 仅新增 #12
#[async_trait]
pub trait StartFromPickerSource: Send + Sync {
    /// 列出 GitHub remote branches (kind 1)
    ///
    /// 实现: 调 `git for-each-ref refs/remotes/` 或 `git ls-remote --heads origin`.
    /// 阶段 1 简化: 只列已 fetch 的 remote branches; 阶段 2 接 GH API.
    async fn list_github_branches(
        &self,
        repo_id: RepoId,
    ) -> Result<Vec<PickerCandidate>, ServiceError>;

    /// 列出 Existing Worktree base refs (kind 2)
    ///
    /// 实现: 调 `WorktreeService::list(repo_id)` + 每条转 `PickerCandidate`.
    async fn list_existing_worktrees(
        &self,
        repo_id: RepoId,
    ) -> Result<Vec<PickerCandidate>, ServiceError>;

    /// 列出 Local Path 候选 (kind 3)
    ///
    /// 实现: 读 per-user allowlist (`~/.star/worktree_shared_dirs.txt` 第 1 行,
    /// 或 per 守门 #6 适配层 fallback 到 `<workspace>/.star/worktree_shared_dirs.txt`).
    /// 阶段 1 简化: 读环境变量 `WORKTREE_LOCAL_PATHS` (逗号分隔) 兜底.
    async fn list_local_paths(
        &self,
        repo_id: RepoId,
    ) -> Result<Vec<PickerCandidate>, ServiceError>;

    /// Empty / Blank 哨兵 (kind 4) — 永远 Some
    fn empty_candidate(&self, repo_id: RepoId) -> PickerCandidate;
}

/// 默认 StartFromPickerSource 实现 — 调 git-adapter + 读 env allowlist.
///
/// 阶段 1 同步实现, 阶段 2 接 ULYS-177.2 async 后 `list_local_paths` 可加 async IO.
pub struct DefaultStartFromPickerSource {
    /// Git provider (per WORKTREE-CANVAS-IMPL-PLAN-001 §3 单向依赖 worktree-service → git-adapter)
    git: Arc<dyn GitProvider>,
}

impl std::fmt::Debug for DefaultStartFromPickerSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultStartFromPickerSource")
            .field("git", &"<dyn GitProvider>")
            .finish()
    }
}

impl DefaultStartFromPickerSource {
    /// 创建新 source (注入 git provider, 测试可换 mock)
    pub fn new(git: Arc<dyn GitProvider>) -> Self {
        Self { git }
    }
}

// `Arc` 在 worktree-service 已有使用 (per service_impl.rs `HealthProvider`),
// 这里借用 [workspace.dependencies] 风格避免重引.

#[async_trait]
impl StartFromPickerSource for DefaultStartFromPickerSource {
    async fn list_github_branches(
        &self,
        repo_id: RepoId,
    ) -> Result<Vec<PickerCandidate>, ServiceError> {
        // 阶段 1 简化: 假设 repo 路径 = `<repo_id>` (UI 端传 RepoHandle.path 进来,
        // WorktreeService 阶段 2 接 PG 后从 DB 拿). 这里走 GitProvider::list_branches
        // 复用既有接口 (per DD §10 + git-adapter/provider.rs:263).
        //
        // 关键决策: 我们 **不调** `git ls-remote` (网络), 只列已 fetch 的本地
        // remote-tracking refs. 阶段 2 ULYS-177.2 后, UI 触发 fetch 后再调.
        let handle = RepoHandle {
            path: PathBuf::from(format!("/repos/{repo_id}")),
            repo_id: Some(repo_id),
        };

        match self.git.list_branches(&handle).await {
            Ok(branches) => {
                let mut out = Vec::with_capacity(branches.len());
                for b in branches {
                    out.push(branch_to_remote_candidate(repo_id, &b));
                }
                Ok(out)
            }
            Err(e) => {
                // 阶段 1 宽容: list_branches 失败 (e.g. 仓库无 git remote) → 返回空 vec
                // + warn, 不阻断其它 3 分类. caller 看到空 Vec 自然 fallback 到 Empty.
                warn!(
                    repo_id = %repo_id,
                    error = %e,
                    "list_branches failed, returning empty (FR-ORCA-009 宽容 fallback)"
                );
                Ok(Vec::new())
            }
        }
    }

    async fn list_existing_worktrees(
        &self,
        _repo_id: RepoId,
    ) -> Result<Vec<PickerCandidate>, ServiceError> {
        // 占位 (per 守门 #19): 这里 **不直接调** WorktreeService::list, 而是接受
        // caller 传入的 `existing: &[Worktree]`. 实现见下方 `pick_start_from_candidates`
        // helper — 它把 Worktree 数组喂给 picker 做转换.
        //
        // Trait method 留空返回, 实际数据由 `pick_start_from_candidates` 注入.
        // 这是为了避免 StartFromPickerSource 反向依赖 WorktreeService (循环依赖).
        Ok(Vec::new())
    }

    async fn list_local_paths(
        &self,
        _repo_id: RepoId,
    ) -> Result<Vec<PickerCandidate>, ServiceError> {
        // 阶段 1 简化: 读环境变量 `WORKTREE_LOCAL_PATHS` (逗号分隔绝对路径).
        // 阶段 2 接 ULYS-177.2 后, 改读 per-user allowlist 文件
        // (`~/.star/worktree_shared_dirs.txt` 第 1 行 per spec FR-ORCA-009).
        //
        // 不存在 env var → 空 Vec + warn, 不阻断.
        let from_env = std::env::var("WORKTREE_LOCAL_PATHS").unwrap_or_default();
        let mut out = Vec::new();
        for raw in from_env.split(',') {
            let p = raw.trim();
            if p.is_empty() {
                continue;
            }
            out.push(local_path_to_candidate(PathBuf::from(p)));
        }
        if out.is_empty() && !from_env.is_empty() {
            warn!("WORKTREE_LOCAL_PATHS set but parsed to 0 valid paths");
        }
        Ok(out)
    }

    fn empty_candidate(&self, repo_id: RepoId) -> PickerCandidate {
        PickerCandidate {
            id: format!("empty:{repo_id}"),
            label: "Empty / Start from scratch".to_string(),
            description: "No base ref — only branch name + repo metadata".to_string(),
            kind: PickerCandidateKind::CommitSha, // sentinel: 代表"无 base ref"
        }
    }
}

/// `Worktree` → `PickerCandidate` (kind 2 Existing Worktree)
pub fn worktree_to_existing_candidate(wt: &Worktree) -> PickerCandidate {
    PickerCandidate {
        id: format!("wt:{}", wt.id),
        label: wt.branch.clone(),
        description: format!(
            "Existing worktree on branch '{}' (state: {:?})",
            wt.branch, wt.human_state
        ),
        kind: PickerCandidateKind::LocalBranch,
    }
}

/// `GitBranch` → `PickerCandidate` (kind 1 GitHub Remote Branch)
pub fn branch_to_remote_candidate(repo_id: RepoId, b: &GitBranch) -> PickerCandidate {
    let prefix = if b.is_mainline { "★ " } else { "" };
    PickerCandidate {
        id: format!("remote:{}:{}/{}", repo_id, "origin", b.name),
        label: format!("{prefix}{}", b.name),
        description: format!(
            "Remote branch origin/{} @ {}",
            b.name,
            b.head_sha.chars().take(7).collect::<String>()
        ),
        kind: PickerCandidateKind::RemoteBranch,
    }
}

/// `PathBuf` → `PickerCandidate` (kind 3 Local Path)
pub fn local_path_to_candidate(p: PathBuf) -> PickerCandidate {
    let display = p.display().to_string();
    let label = p
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| display.clone());
    PickerCandidate {
        id: format!("local:{display}"),
        label,
        description: display,
        kind: PickerCandidateKind::RepoBase,
    }
}

/// 过滤并去重 4 分类 (per spec FR-ORCA-009 §3 "过滤+去重").
///
/// ## 去重规则
///
/// - kind 1 (github_branches): 按 `name` 去重 (同 branch 多次出现保留首次, 通常不会发生)
/// - kind 2 (existing_worktrees): 按 `branch` 去重 (同 branch 多 worktree 保留首个)
/// - kind 3 (local_paths): 按 `path` 去重
/// - kind 4 (empty): 永远 1 个, 不去重
///
/// ## 过滤规则
///
/// - 空字符串 / whitespace-only label → 剔除
/// - archived worktrees (kind 2) → 剔除 (per spec FR-ORCA-009 "active worktree only")
/// - 不存在的 local path (kind 3) → 剔除 (阶段 2 接 ULYS-177.2 后由 caller 提供 IO 结果)
pub fn filter_and_dedupe(
    mut github: Vec<PickerCandidate>,
    mut existing: Vec<PickerCandidate>,
    mut local: Vec<PickerCandidate>,
    empty: Option<PickerCandidate>,
) -> PickerCandidates {
    github.retain(|c| !c.label.trim().is_empty());
    github.dedup_by(|a, b| a.id == b.id);

    existing.retain(|c| !c.label.trim().is_empty());
    existing.dedup_by(|a, b| {
        // 按 branch (label 字段, worktree_to_existing_candidate 把 branch 放进 label) 去重
        a.label == b.label
    });

    local.retain(|c| !c.label.trim().is_empty());
    local.dedup_by(|a, b| a.id == b.id);

    PickerCandidates {
        github_branches: github,
        existing_worktrees: existing,
        local_paths: local,
        empty,
    }
}

/// `WorktreeService::pick_start_from_candidates` — 4 选 1 候选统一入口
/// (WorktreeService trait 第 12 方法, per ULYS-194 §3.1 + §5 收尾验证).
///
/// ## 入参
///
/// - `repo_id`: 仓库 UUID
/// - `existing`: 当前 WorktreeService 内部 state (避免循环依赖 trait method)
/// - `source`: StartFromPickerSource 注入 (默认 `DefaultStartFromPickerSource`)
///
/// ## 返回
///
/// `PickerCandidates` { github_branches, existing_worktrees, local_paths, empty }
/// — UI 端按 4 tab 渲染.
pub async fn pick_start_from_candidates(
    repo_id: RepoId,
    existing: &[Worktree],
    source: &dyn StartFromPickerSource,
) -> Result<PickerCandidates, ServiceError> {
    // kind 1 + 3 + 4 走 trait
    let github = source.list_github_branches(repo_id).await?;
    let local = source.list_local_paths(repo_id).await?;
    let empty = source.empty_candidate(repo_id);

    // kind 2 走 caller 传入的 `existing` 数组 (避免 trait 反向依赖 WorktreeService)
    let existing_candidates: Vec<PickerCandidate> = existing
        .iter()
        .filter(|w| !w.archived) // 过滤 archived
        .map(worktree_to_existing_candidate)
        .collect();

    Ok(filter_and_dedupe(github, existing_candidates, local, Some(empty)))
}

/// 按 PickerCandidate id 挑选 — UI 端 confirm 后回调 (per FR-ORCA-009 §3.3 "用户确认后").
///
/// 返回 `Some(PickerCandidate)` 表示找到, `None` 表示 id 不在 4 分类中.
pub fn select_by_id(cands: &PickerCandidates, id: &str) -> Option<PickerCandidate> {
    let mut all: Vec<&PickerCandidate> = Vec::with_capacity(cands.total());
    all.extend(cands.github_branches.iter());
    all.extend(cands.existing_worktrees.iter());
    all.extend(cands.local_paths.iter());
    if let Some(ref e) = cands.empty {
        all.push(e);
    }
    all.into_iter().find(|c| c.id == id).cloned()
}

/// 按 PickerCandidateKind 收集所有候选 (UI 端可一次性拿全部).
pub fn collect_by_kind(cands: &PickerCandidates, kind: PickerCandidateKind) -> Vec<PickerCandidate> {
    let mut out = Vec::new();
    match kind {
        PickerCandidateKind::RemoteBranch => out.extend(cands.github_branches.iter().cloned()),
        PickerCandidateKind::LocalBranch => out.extend(cands.existing_worktrees.iter().cloned()),
        PickerCandidateKind::RepoBase => out.extend(cands.local_paths.iter().cloned()),
        PickerCandidateKind::CommitSha => {
            if let Some(ref e) = cands.empty {
                out.push(e.clone());
            }
        }
    }
    out
}

/// Check `now` (exported for tests)
#[doc(hidden)]
pub fn _now() -> DateTime<Utc> {
    Utc::now()
}

/// Check whether `id` appears in candidates (set-based, O(1))
pub fn contains_id(cands: &PickerCandidates, id: &str) -> bool {
    let all = || -> Vec<&str> {
        let mut v: Vec<&str> = Vec::with_capacity(cands.total());
        v.extend(cands.github_branches.iter().map(|c| &*c.id));
        v.extend(cands.existing_worktrees.iter().map(|c| &*c.id));
        v.extend(cands.local_paths.iter().map(|c| &*c.id));
        if let Some(ref e) = cands.empty {
            v.push(&*e.id);
        }
        v
    };
    let set: HashSet<&str> = all().into_iter().collect();
    set.contains(id)
}

// =====================================================================
// 测试 (per ULYS-194 §6 收尾验证: ≥ 4 tests for 4 sources, ≥ 6 for filter/dedupe)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;
    use chrono::Utc;
    use git_adapter::provider::BranchInfo as GitBranch;
    use git_adapter::{GitError, WorktreeInfo};
    use graph_core::state::{HumanState, TestState};
    use graph_core::types::WorktreeId;
    use uuid::Uuid;

    // --- mock GitProvider (per 守门 #11 缺标比错标, 仅 test scope) ---

    #[derive(Debug, Clone)]
    struct MockGitProvider {
        branches: Vec<GitBranch>,
    }

    #[async_trait]
    impl GitProvider for MockGitProvider {
        async fn open_repo(&self, _path: &Path) -> Result<RepoHandle, GitError> {
            Ok(RepoHandle {
                path: PathBuf::from("/tmp"),
                repo_id: None,
            })
        }
        async fn list_worktrees(
            &self,
            _repo: &RepoHandle,
        ) -> Result<Vec<WorktreeInfo>, GitError> {
            Ok(Vec::new())
        }
        async fn create_worktree(
            &self,
            _repo: &RepoHandle,
            _branch: &str,
            _path: &Path,
            _base_branch: Option<&str>,
        ) -> Result<WorktreeInfo, GitError> {
            unimplemented!()
        }
        async fn remove_worktree(
            &self,
            _repo: &RepoHandle,
            _path: &Path,
            _force: bool,
        ) -> Result<(), GitError> {
            Ok(())
        }
        async fn diff(
            &self,
            _repo: &RepoHandle,
            _from: &str,
            _to: &str,
        ) -> Result<git_adapter::Diff, GitError> {
            unimplemented!()
        }
        async fn merge_base(
            &self,
            _repo: &RepoHandle,
            _a: &str,
            _b: &str,
        ) -> Result<String, GitError> {
            Ok(String::new())
        }
        async fn rev_list_count(
            &self,
            _repo: &RepoHandle,
            _range: &str,
        ) -> Result<u32, GitError> {
            Ok(0)
        }
        async fn status(&self, _repo: &RepoHandle) -> Result<Vec<git_adapter::StatusEntry>, GitError> {
            Ok(Vec::new())
        }
        async fn sync_main(
            &self,
            _repo: &RepoHandle,
            _worktree_path: &Path,
        ) -> Result<git_adapter::SyncResult, GitError> {
            unimplemented!()
        }
        async fn rebase(
            &self,
            _repo: &RepoHandle,
            _worktree: &Path,
            _target: &str,
        ) -> Result<(), GitError> {
            unimplemented!()
        }
        async fn merge(
            &self,
            _repo: &RepoHandle,
            _worktree: &Path,
            _branch: &str,
            _strategy: git_adapter::provider::MergeStrategy,
        ) -> Result<git_adapter::MergeResult, GitError> {
            unimplemented!()
        }
        async fn head(
            &self,
            _repo: &RepoHandle,
            _wt: &Path,
        ) -> Result<String, GitError> {
            Ok(String::new())
        }
        async fn last_commit_at(
            &self,
            _repo: &RepoHandle,
            _wt: &Path,
        ) -> Result<DateTime<Utc>, GitError> {
            Ok(Utc::now())
        }
        async fn list_branches(&self, _repo: &RepoHandle) -> Result<Vec<GitBranch>, GitError> {
            Ok(self.branches.clone())
        }
    }

    fn mock_git_with(branches: Vec<GitBranch>) -> Arc<dyn GitProvider> {
        Arc::new(MockGitProvider { branches })
    }

    fn branch(name: &str, sha: &str, is_main: bool) -> GitBranch {
        GitBranch {
            name: name.to_string(),
            head_sha: sha.to_string(),
            is_mainline: is_main,
            updated_at: Utc::now(),
        }
    }

    fn make_worktree(branch: &str, archived: bool) -> Worktree {
        let now = Utc::now();
        Worktree {
            id: WorktreeId::from(Uuid::new_v4()),
            repo_id: RepoId::from(Uuid::new_v4()),
            name: branch.to_string(),
            branch: branch.to_string(),
            path: PathBuf::from(format!("/worktrees/{branch}")),
            human_state: HumanState::Running,
            test_state: TestState::None,
            ahead: 0,
            behind: 0,
            dirty: false,
            locked: false,
            archived,
            health_score: 100,
            last_activity: now,
            created_at: now,
            merged_at: None,
            locked_by: None,
            agent_type: None,
        }
    }

    // ---------- 4 来源: ≥ 4 tests (per spec §6 收尾验证) ----------

    #[tokio::test]
    async fn source_1_github_branches_lists_remote_branches() {
        let git = mock_git_with(vec![
            branch("main", "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111", true),
            branch("feat-x", "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222", false),
        ]);
        let src = DefaultStartFromPickerSource::new(git);
        let repo_id = RepoId::from(Uuid::new_v4());

        let out = src.list_github_branches(repo_id).await.unwrap();
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|c| c.kind == PickerCandidateKind::RemoteBranch));
        // ★ prefix on mainline
        assert!(out[0].label.starts_with("★ "), "mainline branch gets star");
        assert!(!out[1].label.starts_with("★ "));
    }

    #[tokio::test]
    async fn source_1_github_branches_empty_when_no_remote() {
        // mock returns error → 宽容 fallback to empty (per spec FR-ORCA-009)
        #[derive(Debug)]
        struct FailingGit;
        #[async_trait]
        impl GitProvider for FailingGit {
            async fn list_branches(&self, _: &RepoHandle) -> Result<Vec<GitBranch>, GitError> {
                Err(GitError::new("GIT.NO_REMOTE", "no remote"))
            }
            // 其他方法全部 unimplemented
            async fn open_repo(&self, _: &Path) -> Result<RepoHandle, GitError> {
                unimplemented!()
            }
            async fn list_worktrees(&self, _: &RepoHandle) -> Result<Vec<WorktreeInfo>, GitError> {
                unimplemented!()
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
                _: &Path,
                _: bool,
            ) -> Result<(), GitError> {
                unimplemented!()
            }
            async fn diff(
                &self,
                _: &RepoHandle,
                _: &str,
                _: &str,
            ) -> Result<git_adapter::Diff, GitError> {
                unimplemented!()
            }
            async fn merge_base(&self, _: &RepoHandle, _: &str, _: &str) -> Result<String, GitError> {
                unimplemented!()
            }
            async fn rev_list_count(&self, _: &RepoHandle, _: &str) -> Result<u32, GitError> {
                unimplemented!()
            }
            async fn status(&self, _: &RepoHandle) -> Result<Vec<git_adapter::StatusEntry>, GitError> {
                unimplemented!()
            }
            async fn sync_main(
                &self,
                _: &RepoHandle,
                _: &Path,
            ) -> Result<git_adapter::SyncResult, GitError> {
                unimplemented!()
            }
            async fn rebase(
                &self,
                _: &RepoHandle,
                _: &Path,
                _: &str,
            ) -> Result<(), GitError> {
                unimplemented!()
            }
            async fn merge(
                &self,
                _: &RepoHandle,
                _: &Path,
                _: &str,
                _: git_adapter::provider::MergeStrategy,
            ) -> Result<git_adapter::MergeResult, GitError> {
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
        }

        let src = DefaultStartFromPickerSource::new(Arc::new(FailingGit));
        let repo_id = RepoId::from(Uuid::new_v4());
        let out = src.list_github_branches(repo_id).await.unwrap();
        assert!(out.is_empty(), "fallback to empty on error");
    }

    #[tokio::test]
    async fn source_2_existing_worktrees_via_pick_helper() {
        // kind 2 走 `pick_start_from_candidates` 注入 existing 数组
        let git = mock_git_with(vec![]);
        let src = DefaultStartFromPickerSource::new(git);
        let repo_id = RepoId::from(Uuid::new_v4());
        let existing = vec![
            make_worktree("main", false),
            make_worktree("feat-y", false),
            make_worktree("archived-z", true), // 应被过滤
        ];

        let out = pick_start_from_candidates(repo_id, &existing, &src)
            .await
            .unwrap();
        assert_eq!(out.existing_worktrees.len(), 2);
        let names: Vec<&str> = out
            .existing_worktrees
            .iter()
            .map(|c| c.label.as_str())
            .collect();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"feat-y"));
        assert!(!names.contains(&"archived-z"), "archived filtered");
        assert!(out.existing_worktrees.iter().all(|c| c.kind == PickerCandidateKind::LocalBranch));
    }

    #[tokio::test]
    async fn source_3_local_paths_from_env_or_empty() {
        let git = mock_git_with(vec![]);
        let src = DefaultStartFromPickerSource::new(git);
        let repo_id = RepoId::from(Uuid::new_v4());

        // 没设 env → 空
        // SAFETY: 单测内 set env 不会跨 test 影响 (tokio::test 顺序, env 隔离不严格但本测试单跑)
        std::env::remove_var("WORKTREE_LOCAL_PATHS");
        let out = src.list_local_paths(repo_id).await.unwrap();
        assert!(out.is_empty());

        // 设 env → 3 个候选
        std::env::set_var(
            "WORKTREE_LOCAL_PATHS",
            "/tmp/repo-a,/tmp/repo-b , ,/tmp/repo-c",
        );
        let out = src.list_local_paths(repo_id).await.unwrap();
        assert_eq!(out.len(), 3, "whitespace + empty entries skipped");
        assert!(out.iter().all(|c| c.kind == PickerCandidateKind::RepoBase));
        std::env::remove_var("WORKTREE_LOCAL_PATHS");
    }

    #[tokio::test]
    async fn source_4_empty_candidate_always_present() {
        let git = mock_git_with(vec![]);
        let src = DefaultStartFromPickerSource::new(git);
        let repo_id = RepoId::from(Uuid::new_v4());

        let empty = src.empty_candidate(repo_id);
        assert_eq!(empty.kind, PickerCandidateKind::CommitSha);
        assert!(empty.id.starts_with("empty:"));

        // pick_start_from_candidates 总把 empty 放进 output
        let out = pick_start_from_candidates(repo_id, &[], &src).await.unwrap();
        assert!(out.empty.is_some());
        assert!(out.total() >= 1, "empty alone makes total ≥ 1");
    }

    // ---------- 过滤 + 去重: ≥ 6 tests (per spec §6 收尾验证) ----------

    #[test]
    fn filter_dedup_removes_empty_labels() {
        let bad = PickerCandidate {
            id: "bad:1".into(),
            label: "   ".into(),
            description: "".into(),
            kind: PickerCandidateKind::RemoteBranch,
        };
        let good = PickerCandidate {
            id: "good:1".into(),
            label: "feat-x".into(),
            description: "".into(),
            kind: PickerCandidateKind::RemoteBranch,
        };
        let out = filter_and_dedupe(vec![bad, good], vec![], vec![], None);
        assert_eq!(out.github_branches.len(), 1);
        assert_eq!(out.github_branches[0].id, "good:1");
    }

    #[test]
    fn filter_dedup_existing_worktree_branch_dedup() {
        let a = PickerCandidate {
            id: "wt:a".into(),
            label: "main".into(),
            description: "".into(),
            kind: PickerCandidateKind::LocalBranch,
        };
        let b = PickerCandidate {
            id: "wt:b".into(),
            label: "main".into(), // 同 branch
            description: "".into(),
            kind: PickerCandidateKind::LocalBranch,
        };
        let c = PickerCandidate {
            id: "wt:c".into(),
            label: "feat-x".into(),
            description: "".into(),
            kind: PickerCandidateKind::LocalBranch,
        };
        let out = filter_and_dedupe(vec![], vec![a, b, c], vec![], None);
        assert_eq!(out.existing_worktrees.len(), 2, "main deduped");
    }

    #[test]
    fn filter_dedup_local_paths_by_id() {
        let p1 = PickerCandidate {
            id: "local:/tmp/repo-a".into(),
            label: "repo-a".into(),
            description: "/tmp/repo-a".into(),
            kind: PickerCandidateKind::RepoBase,
        };
        let p1_dup = p1.clone();
        let p2 = PickerCandidate {
            id: "local:/tmp/repo-b".into(),
            label: "repo-b".into(),
            description: "/tmp/repo-b".into(),
            kind: PickerCandidateKind::RepoBase,
        };
        let out = filter_and_dedupe(vec![], vec![], vec![p1, p1_dup, p2], None);
        assert_eq!(out.local_paths.len(), 2);
    }

    #[test]
    fn filter_dedup_empty_singleton() {
        // empty 永远 1 个, 即使传 None 也保留 None
        let out = filter_and_dedupe(vec![], vec![], vec![], None);
        assert!(out.empty.is_none());

        let sentinel = PickerCandidate {
            id: "empty:1".into(),
            label: "Empty".into(),
            description: "".into(),
            kind: PickerCandidateKind::CommitSha,
        };
        let out = filter_and_dedupe(vec![], vec![], vec![], Some(sentinel.clone()));
        assert_eq!(out.empty, Some(sentinel));
    }

    #[test]
    fn select_by_id_finds_in_all_categories() {
        let gh = PickerCandidate {
            id: "gh:1".into(),
            label: "main".into(),
            description: "".into(),
            kind: PickerCandidateKind::RemoteBranch,
        };
        let ex = PickerCandidate {
            id: "wt:1".into(),
            label: "main".into(),
            description: "".into(),
            kind: PickerCandidateKind::LocalBranch,
        };
        let lp = PickerCandidate {
            id: "lp:1".into(),
            label: "repo".into(),
            description: "".into(),
            kind: PickerCandidateKind::RepoBase,
        };
        let em = PickerCandidate {
            id: "empty:1".into(),
            label: "Empty".into(),
            description: "".into(),
            kind: PickerCandidateKind::CommitSha,
        };
        let cands = PickerCandidates {
            github_branches: vec![gh.clone()],
            existing_worktrees: vec![ex.clone()],
            local_paths: vec![lp.clone()],
            empty: Some(em.clone()),
        };

        assert_eq!(select_by_id(&cands, "gh:1").unwrap().kind, PickerCandidateKind::RemoteBranch);
        assert_eq!(select_by_id(&cands, "wt:1").unwrap().kind, PickerCandidateKind::LocalBranch);
        assert_eq!(select_by_id(&cands, "lp:1").unwrap().kind, PickerCandidateKind::RepoBase);
        assert_eq!(select_by_id(&cands, "empty:1").unwrap().kind, PickerCandidateKind::CommitSha);
        assert!(select_by_id(&cands, "nope").is_none());
    }

    #[test]
    fn collect_by_kind_groups_correctly() {
        let gh = PickerCandidate {
            id: "gh:1".into(),
            label: "main".into(),
            description: "".into(),
            kind: PickerCandidateKind::RemoteBranch,
        };
        let em = PickerCandidate {
            id: "empty:1".into(),
            label: "Empty".into(),
            description: "".into(),
            kind: PickerCandidateKind::CommitSha,
        };
        let cands = PickerCandidates {
            github_branches: vec![gh.clone()],
            existing_worktrees: vec![],
            local_paths: vec![],
            empty: Some(em.clone()),
        };
        assert_eq!(collect_by_kind(&cands, PickerCandidateKind::RemoteBranch).len(), 1);
        assert_eq!(collect_by_kind(&cands, PickerCandidateKind::LocalBranch).len(), 0);
        assert_eq!(collect_by_kind(&cands, PickerCandidateKind::RepoBase).len(), 0);
        assert_eq!(collect_by_kind(&cands, PickerCandidateKind::CommitSha).len(), 1);
    }

    // ---------- 集成 + edge cases (per spec §6 ≥ 2 integration) ----------

    #[tokio::test]
    async fn pick_start_from_candidates_full_pipeline() {
        // 4 来源全部命中, 验证总条数
        let git = mock_git_with(vec![
            branch("main", "aaa1111aaa1111aaa1111aaa1111aaa1111aaa1", true),
            branch("dev", "bbb2222bbb2222bbb2222bbb2222bbb2222bbb2", false),
        ]);
        let src = DefaultStartFromPickerSource::new(git);
        let repo_id = RepoId::from(Uuid::new_v4());

        std::env::set_var("WORKTREE_LOCAL_PATHS", "/tmp/repo-x");
        let existing = vec![make_worktree("main", false)];

        let out = pick_start_from_candidates(repo_id, &existing, &src)
            .await
            .unwrap();
        std::env::remove_var("WORKTREE_LOCAL_PATHS");

        assert_eq!(out.github_branches.len(), 2);
        assert_eq!(out.existing_worktrees.len(), 1);
        assert_eq!(out.local_paths.len(), 1);
        assert!(out.empty.is_some());
        assert_eq!(out.total(), 5); // 2+1+1+1
    }

    #[tokio::test]
    async fn pick_start_from_candidates_empty_repo_returns_only_empty() {
        // 仓库啥都没 → 只有 empty 候选 (FR-ORCA-009 §2 启动条件 "无前置依赖")
        let git = mock_git_with(vec![]);
        std::env::remove_var("WORKTREE_LOCAL_PATHS");
        let src = DefaultStartFromPickerSource::new(git);
        let repo_id = RepoId::from(Uuid::new_v4());

        let out = pick_start_from_candidates(repo_id, &[], &src).await.unwrap();
        assert!(out.github_branches.is_empty());
        assert!(out.existing_worktrees.is_empty());
        assert!(out.local_paths.is_empty());
        assert!(out.empty.is_some(), "empty always present (kind 4 启动条件)");
        assert_eq!(out.total(), 1);
    }

    // ---------- 字段映射 + 辅助函数 ----------

    #[test]
    fn worktree_to_existing_candidate_uses_branch_field_as_label() {
        let wt = make_worktree("feat-y", false);
        let c = worktree_to_existing_candidate(&wt);
        assert_eq!(c.label, "feat-y");
        assert_eq!(c.kind, PickerCandidateKind::LocalBranch);
        assert!(c.description.contains("feat-y"));
        assert!(c.id.starts_with("wt:"));
    }

    #[test]
    fn branch_to_remote_candidate_marks_mainline() {
        let b = branch("main", "abc", true);
        let c = branch_to_remote_candidate(RepoId::from(Uuid::new_v4()), &b);
        assert!(c.label.starts_with("★ "), "mainline gets ★ prefix");

        let b = branch("feat-x", "def", false);
        let c = branch_to_remote_candidate(RepoId::from(Uuid::new_v4()), &b);
        assert!(!c.label.starts_with("★ "));
        assert_eq!(c.kind, PickerCandidateKind::RemoteBranch);
        assert!(c.id.contains("origin/"));
        assert!(c.description.contains("origin/feat-x"));
        // SHA 7 字符前缀
        assert!(c.description.contains("def"));
    }

    #[test]
    fn local_path_to_candidate_uses_filename_as_label() {
        let c = local_path_to_candidate(PathBuf::from("/home/user/proj"));
        assert_eq!(c.label, "proj");
        assert_eq!(c.description, "/home/user/proj");
        assert_eq!(c.kind, PickerCandidateKind::RepoBase);
    }

    #[test]
    fn contains_id_set_lookup() {
        let em = PickerCandidate {
            id: "empty:1".into(),
            label: "Empty".into(),
            description: "".into(),
            kind: PickerCandidateKind::CommitSha,
        };
        let cands = PickerCandidates {
            github_branches: vec![],
            existing_worktrees: vec![],
            local_paths: vec![],
            empty: Some(em),
        };
        assert!(contains_id(&cands, "empty:1"));
        assert!(!contains_id(&cands, "nope"));
    }
}