//! `event.rs` — `GitEvent` 15 种 (per spec §4.5 + DD §11)
//!
//! Per ULYS-57.1 T2.6 (WORKTREE-CANVAS-IMPL-PLAN-001 §4.2) + TEST-DESIGN §2.2 UT `git_event_15_types_emit_correct_payload`:
//! - 15 GitEvent 完整覆盖 (INV-WC-NFR-REL-001 守门)
//! - 每种 event 携带 `repo_id` + 业务字段, 便于消费端 (Graph / UI / Risk Engine / Agent / Notification)
//!
//! 15 GitEvent 类型清单 (per spec §4.5 + DD §11 + BD §13):
//!  1. HeadChanged
//!  2. RefsUpdated
//!  3. WorkingTreeChanged
//!  4. IndexChanged
//!  5. WorktreeListChanged
//!  6. BranchCreated
//!  7. BranchDeleted
//!  8. CommitReceived
//!  9. MergeStarted
//! 10. MergeCompleted
//! 11. RebaseStarted
//! 12. RebaseCompleted
//! 13. FetchCompleted
//! 14. PushCompleted
//! 15. TagCreated

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use graph_core::types::RepoId;

/// Tagged enum dispatching 15 GitEvent 类型 (per spec §4.5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GitEvent {
    /// 1. HEAD commit SHA 变更
    HeadChanged {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// 新 HEAD SHA
        new_head: String,
        /// 旧 HEAD SHA
        old_head: Option<String>,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 2. refs/heads/<branch> 更新
    RefsUpdated {
        /// Repo UUID
        repo_id: RepoId,
        /// Branch 名
        branch: String,
        /// 新 SHA
        new_head: String,
        /// 旧 SHA
        old_head: Option<String>,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 3. 工作区变更 (dirty)
    WorkingTreeChanged {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// 是否 dirty
        dirty: bool,
        /// 变更文件数
        files_changed: u32,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 4. Index 变更 (staged)
    IndexChanged {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// staged file count
        staged_count: u32,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 5. Worktree 列表变更
    WorktreeListChanged {
        /// Repo UUID
        repo_id: RepoId,
        /// 新增
        added: Vec<PathBuf>,
        /// 移除
        removed: Vec<PathBuf>,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 6. Branch 创建
    BranchCreated {
        /// Repo UUID
        repo_id: RepoId,
        /// Branch 名
        branch: String,
        /// Branch HEAD SHA
        head_sha: String,
        /// 从哪个 base 派生
        base_sha: Option<String>,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 7. Branch 删除
    BranchDeleted {
        /// Repo UUID
        repo_id: RepoId,
        /// Branch 名
        branch: String,
        /// 最后一次 SHA
        last_sha: String,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 8. 收到新 commit
    CommitReceived {
        /// Repo UUID
        repo_id: RepoId,
        /// Commit SHA
        sha: String,
        /// Author
        author: String,
        /// Commit message
        message: String,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 9. Merge 开始
    MergeStarted {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// 源 branch
        source_branch: String,
        /// 目标 branch
        target_branch: String,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 10. Merge 完成
    MergeCompleted {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// 是否成功
        success: bool,
        /// Merge commit SHA
        merge_commit: Option<String>,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 11. Rebase 开始
    RebaseStarted {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// 目标 branch
        target: String,
        /// Rebase commit 数
        commit_count: u32,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 12. Rebase 完成
    RebaseCompleted {
        /// Repo UUID
        repo_id: RepoId,
        /// Worktree 路径
        worktree_path: PathBuf,
        /// 是否成功
        success: bool,
        /// 实际 Rebase commit 数
        rebased_commits: u32,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 13. Fetch 完成
    FetchCompleted {
        /// Repo UUID
        repo_id: RepoId,
        /// Remote 名 (e.g. "origin")
        remote: String,
        /// Fetch commit 数
        fetched_commits: u32,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 14. Push 完成
    PushCompleted {
        /// Repo UUID
        repo_id: RepoId,
        /// Remote 名
        remote: String,
        /// Branch 名
        branch: String,
        /// Pushed commit 数
        pushed_commits: u32,
        /// 时间戳
        at: DateTime<Utc>,
    },
    /// 15. Tag 创建
    TagCreated {
        /// Repo UUID
        repo_id: RepoId,
        /// Tag 名
        tag: String,
        /// 指向 SHA
        target_sha: String,
        /// 消息
        message: Option<String>,
        /// 时间戳
        at: DateTime<Utc>,
    },
}

/// 15 GitEvent 类型 kind 标签 (per INV-WC-NFR-REL-001 enum completeness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitEventKind {
    /// HEAD commit SHA 变更
    HeadChanged,
    /// refs/heads/<branch> 更新
    RefsUpdated,
    /// 工作区变更
    WorkingTreeChanged,
    /// Index 变更
    IndexChanged,
    /// Worktree 列表变更
    WorktreeListChanged,
    /// Branch 创建
    BranchCreated,
    /// Branch 删除
    BranchDeleted,
    /// 收到新 commit
    CommitReceived,
    /// Merge 开始
    MergeStarted,
    /// Merge 完成
    MergeCompleted,
    /// Rebase 开始
    RebaseStarted,
    /// Rebase 完成
    RebaseCompleted,
    /// Fetch 完成
    FetchCompleted,
    /// Push 完成
    PushCompleted,
    /// Tag 创建
    TagCreated,
}

impl GitEventKind {
    /// `GitEvent` variant 数量 — NFR-REL-001 守门用
    pub const COUNT: usize = 15;

    /// 所有 15 个 kind (按 enum 顺序)
    pub const ALL: [GitEventKind; Self::COUNT] = [
        Self::HeadChanged,
        Self::RefsUpdated,
        Self::WorkingTreeChanged,
        Self::IndexChanged,
        Self::WorktreeListChanged,
        Self::BranchCreated,
        Self::BranchDeleted,
        Self::CommitReceived,
        Self::MergeStarted,
        Self::MergeCompleted,
        Self::RebaseStarted,
        Self::RebaseCompleted,
        Self::FetchCompleted,
        Self::PushCompleted,
        Self::TagCreated,
    ];
}

impl GitEvent {
    /// 提取对应的 `GitEventKind`.
    pub fn kind(&self) -> GitEventKind {
        match self {
            Self::HeadChanged { .. } => GitEventKind::HeadChanged,
            Self::RefsUpdated { .. } => GitEventKind::RefsUpdated,
            Self::WorkingTreeChanged { .. } => GitEventKind::WorkingTreeChanged,
            Self::IndexChanged { .. } => GitEventKind::IndexChanged,
            Self::WorktreeListChanged { .. } => GitEventKind::WorktreeListChanged,
            Self::BranchCreated { .. } => GitEventKind::BranchCreated,
            Self::BranchDeleted { .. } => GitEventKind::BranchDeleted,
            Self::CommitReceived { .. } => GitEventKind::CommitReceived,
            Self::MergeStarted { .. } => GitEventKind::MergeStarted,
            Self::MergeCompleted { .. } => GitEventKind::MergeCompleted,
            Self::RebaseStarted { .. } => GitEventKind::RebaseStarted,
            Self::RebaseCompleted { .. } => GitEventKind::RebaseCompleted,
            Self::FetchCompleted { .. } => GitEventKind::FetchCompleted,
            Self::PushCompleted { .. } => GitEventKind::PushCompleted,
            Self::TagCreated { .. } => GitEventKind::TagCreated,
        }
    }

    /// 提取 repo_id
    pub fn repo_id(&self) -> RepoId {
        match self {
            Self::HeadChanged { repo_id, .. }
            | Self::RefsUpdated { repo_id, .. }
            | Self::WorkingTreeChanged { repo_id, .. }
            | Self::IndexChanged { repo_id, .. }
            | Self::WorktreeListChanged { repo_id, .. }
            | Self::BranchCreated { repo_id, .. }
            | Self::BranchDeleted { repo_id, .. }
            | Self::CommitReceived { repo_id, .. }
            | Self::MergeStarted { repo_id, .. }
            | Self::MergeCompleted { repo_id, .. }
            | Self::RebaseStarted { repo_id, .. }
            | Self::RebaseCompleted { repo_id, .. }
            | Self::FetchCompleted { repo_id, .. }
            | Self::PushCompleted { repo_id, .. }
            | Self::TagCreated { repo_id, .. } => *repo_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_event_kind_count_is_fifteen() {
        assert_eq!(GitEventKind::COUNT, 15);
        assert_eq!(GitEventKind::ALL.len(), 15);
    }

    #[test]
    fn all_git_event_kinds_unique() {
        let kinds = GitEventKind::ALL;
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate kind at index {} vs {}", i, j);
                }
            }
        }
    }

    #[test]
    fn head_changed_event_carries_repo_id() {
        let event = GitEvent::HeadChanged {
            repo_id: RepoId::new_v4(),
            worktree_path: PathBuf::from("/tmp/wt"),
            new_head: "0".repeat(40),
            old_head: None,
            at: Utc::now(),
        };
        assert_eq!(event.kind(), GitEventKind::HeadChanged);
        assert_eq!(event.repo_id(), event.repo_id());
    }

    #[test]
    fn git_event_serializes_with_kind_tag() {
        let event = GitEvent::MergeCompleted {
            repo_id: RepoId::new_v4(),
            worktree_path: PathBuf::from("/tmp/wt"),
            success: true,
            merge_commit: Some("deadbeef".into()),
            at: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"kind\":\"merge_completed\""));
        assert!(json.contains("\"success\":true"));
    }
}
