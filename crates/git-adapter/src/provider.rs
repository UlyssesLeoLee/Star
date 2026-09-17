//! `provider.rs` — `GitProvider` Trait + 14 方法 (per DD-WORKTREE-CANVAS-001 §10)
//!
//! Per ULYS-57.1 T2.1 + IMPL-PLAN §4.2:
//! - 14 方法 (per D-GIT-001 + NFR-REL-001 Git Source of Truth 不可变)
//!
//! 方法清单 (per DD §10 + 守门 #1 v25):
//!  1. `open_repo`           (打开 Repo)
//!  2. `list_worktrees`      (拉 Worktree 列表)
//!  3. `create_worktree`     (建 Worktree)
//!  4. `remove_worktree`     (删 Worktree)
//!  5. `diff`                (Diff)
//!  6. `merge_base`          (Merge base SHA)
//!  7. `rev_list_count`      (commit 计数, ahead/behind 计算)
//!  8. `status`              (porcelain status)
//!  9. `sync_main`           (Fetch + Rebase)
//! 10. `rebase`              (Rebase onto target)
//! 11. `merge`               (Merge branch into current)
//! 12. `head`                (Get HEAD SHA)
//! 13. `last_commit_at`      (Last commit time)
//! 14. `list_branches`       (List branches — NFR-REL-001 补, 用于 ahead/behind / USES_BRANCH edge)
//!
//! 跨域引用 (per graph-core `state.rs`):
//! - `MergeStrategy` 用 graph_core::state::MergeStrategy

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// MergeStrategy 复 graph-core (per 跨域依赖, 单向: git-adapter 依赖 graph-core)
pub use graph_core::state::MergeStrategy;

/// Repo handle (per DD §10)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoHandle {
    /// Repo 路径
    pub path: PathBuf,
    /// 关联的 RepoId (跨 crate 引用, per 守门 #11)
    pub repo_id: Option<graph_core::types::RepoId>,
}

/// Worktree info (per DD §10 + 守门 #NFR-REL-001 Git 不可变: 一次性 snapshot)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorktreeInfo {
    /// Worktree 路径
    pub path: PathBuf,
    /// Worktree name
    pub name: String,
    /// Branch name
    pub branch: String,
    /// HEAD SHA
    pub head: String,
    /// Ahead of main
    pub ahead: u32,
    /// Behind main
    pub behind: u32,
    /// Working tree dirty?
    pub dirty: bool,
    /// Last commit time
    pub last_commit_at: Option<DateTime<Utc>>,
}

/// Diff result (per DD §10)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diff {
    /// Per-file diff
    pub files: Vec<DiffFile>,
    /// Total lines added
    pub total_added: u32,
    /// Total lines removed
    pub total_removed: u32,
}

/// Per-file diff
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffFile {
    /// File path
    pub path: PathBuf,
    /// Added line numbers
    pub added_lines: Vec<u32>,
    /// Removed line numbers
    pub removed_lines: Vec<u32>,
    /// Hunks (line ranges)
    pub hunks: Vec<DiffHunk>,
}

/// Diff hunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffHunk {
    /// Old start line
    pub old_start: u32,
    /// Old line count
    pub old_lines: u32,
    /// New start line
    pub new_start: u32,
    /// New line count
    pub new_lines: u32,
}

/// Status entry (per DD §10 porcelain status)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusEntry {
    /// File path
    pub path: PathBuf,
    /// Status kind (per 4 档分类 — `git_status_dirty_state_4_classification` UT)
    pub kind: StatusKind,
}

/// 4 档 dirty state 分类 (per DD §10 + 守门 NFR-REL-001)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusKind {
    /// Working tree modified
    Modified,
    /// Working tree staged
    Staged,
    /// Untracked file
    Untracked,
    /// Conflicting (merge in progress)
    Conflicting,
}

/// Sync result (per DD §10)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncResult {
    /// Fetched commit count
    pub fetched_commits: u32,
    /// Rebased commit count
    pub rebased_commits: u32,
    /// Conflicts (path / error)
    pub conflicts: Vec<ConflictInfo>,
}

/// Merge result (per DD §10)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MergeResult {
    /// Success?
    pub success: bool,
    /// Merge commit SHA (None on failure)
    pub merge_commit: Option<String>,
    /// Conflicts
    pub conflicts: Vec<ConflictInfo>,
    /// Strategy used
    pub strategy_used: MergeStrategy,
}

/// Conflict info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictInfo {
    /// File path / commit SHA
    pub path: String,
    /// Error description
    pub error: String,
}

/// Branch info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// HEAD SHA
    pub head_sha: String,
    /// Is mainline?
    pub is_mainline: bool,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

/// `GitProvider` Trait — 14 方法 (per DD §10 + IMPL-PLAN T2.1)
#[async_trait]
pub trait GitProvider: Send + Sync {
    /// 1. Open a repository
    async fn open_repo(&self, path: &Path) -> Result<RepoHandle, crate::error::GitError>;

    /// 2. Get Worktree list
    async fn list_worktrees(
        &self,
        repo: &RepoHandle,
    ) -> Result<Vec<WorktreeInfo>, crate::error::GitError>;

    /// 3. Create Worktree
    async fn create_worktree(
        &self,
        repo: &RepoHandle,
        branch: &str,
        path: &Path,
        base_branch: Option<&str>,
    ) -> Result<WorktreeInfo, crate::error::GitError>;

    /// 4. Remove Worktree
    async fn remove_worktree(
        &self,
        repo: &RepoHandle,
        path: &Path,
        force: bool,
    ) -> Result<(), crate::error::GitError>;

    /// 5. Get diff
    async fn diff(
        &self,
        repo: &RepoHandle,
        from: &str,
        to: &str,
    ) -> Result<Diff, crate::error::GitError>;

    /// 6. Get merge base
    async fn merge_base(
        &self,
        repo: &RepoHandle,
        a: &str,
        b: &str,
    ) -> Result<String, crate::error::GitError>;

    /// 7. Count commits in range
    async fn rev_list_count(
        &self,
        repo: &RepoHandle,
        range: &str,
    ) -> Result<u32, crate::error::GitError>;

    /// 8. Get status (porcelain)
    async fn status(&self, repo: &RepoHandle) -> Result<Vec<StatusEntry>, crate::error::GitError>;

    /// 9. Fetch + Rebase onto main
    async fn sync_main(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
    ) -> Result<SyncResult, crate::error::GitError>;

    /// 10. Rebase onto target
    async fn rebase(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
        target: &str,
    ) -> Result<(), crate::error::GitError>;

    /// 11. Merge branch into current
    async fn merge(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
        branch: &str,
        strategy: MergeStrategy,
    ) -> Result<MergeResult, crate::error::GitError>;

    /// 12. Get HEAD SHA
    async fn head(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
    ) -> Result<String, crate::error::GitError>;

    /// 13. Get last commit time
    async fn last_commit_at(
        &self,
        repo: &RepoHandle,
        worktree_path: &Path,
    ) -> Result<DateTime<Utc>, crate::error::GitError>;

    /// 14. List branches (NFR-REL-001 补, 用于 ahead/behind / USES_BRANCH edge)
    async fn list_branches(
        &self,
        repo: &RepoHandle,
    ) -> Result<Vec<BranchInfo>, crate::error::GitError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_kind_serializes_snake_case() {
        let j = serde_json::to_string(&StatusKind::Modified).unwrap();
        assert_eq!(j, "\"modified\"");
    }

    #[test]
    fn repo_handle_includes_repo_id() {
        let h = RepoHandle {
            path: PathBuf::from("/tmp/x"),
            repo_id: Some(graph_core::types::RepoId::new_v4()),
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains("repo_id"));
    }

    #[test]
    fn worktree_info_has_required_fields() {
        let info = WorktreeInfo {
            path: PathBuf::from("/tmp/wt"),
            name: "wt-a".into(),
            branch: "feat/a".into(),
            head: "abc".into(),
            ahead: 2,
            behind: 3,
            dirty: false,
            last_commit_at: None,
        };
        let j = serde_json::to_string(&info).unwrap();
        let parsed: WorktreeInfo = serde_json::from_str(&j).unwrap();
        assert_eq!(parsed.ahead, 2);
        assert_eq!(parsed.behind, 3);
    }

    #[test]
    fn merge_result_carries_strategy_used() {
        let m = MergeResult {
            success: true,
            merge_commit: Some("deadbeef".into()),
            conflicts: vec![],
            strategy_used: MergeStrategy::Merge,
        };
        let j = serde_json::to_string(&m).unwrap();
        assert!(j.contains("\"strategy_used\":\"merge\""));
    }
}
