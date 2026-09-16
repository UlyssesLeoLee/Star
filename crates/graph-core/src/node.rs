//! `node.rs` — 11 Node 类型 enum + struct (per DD §7 + §2.1..§2.4)
//!
//! Per INV-WC-05: 11 Node 类型严格 11 个, 不允许多/少
//!
//! 11 Node 类型清单 (per SRS §7 + DD §2.1..§2.4 + §7.2..§7.7):
//! 1. RepositoryNode (DD §2.2)
//! 2. BranchNode (DD §2.1 path — Mainline = BranchNode)
//! 3. WorktreeNode (DD §2.1)
//! 4. CommitNode (DD §7.2)
//! 5. TaskNode (DD §2.4)
//! 6. AgentSessionNode (DD §2.3)
//! 7. PullRequestNode (DD §7.3)
//! 8. FileNode (DD §7.4)
//! 9. SymbolNode (DD §7.5)
//! 10. TestRunNode (DD §7.6)
//! 11. IssueNode (DD §7.7)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::state::{
    AgentResultState, AgentStatus, IssueState, PRState, SymbolKind, TaskStatus, TestStatus,
    TokenUsage,
};
use crate::types::{AgentId, IssueId, PRId, RepoId, TaskId, TestId, UserId, WorktreeId};

/// Tagged enum dispatching 11 Node 类型 (per INV-WC-05)
///
/// 序列化形态: `{"kind": "repository" | "branch" | ..., "data": {...}}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodePayload {
    /// Repository (DD §2.2)
    Repository(RepositoryNode),
    /// Branch (DD §2.1 path — Mainline 也是 BranchNode)
    Branch(BranchNode),
    /// Worktree (DD §2.1)
    Worktree(WorktreeNode),
    /// Commit (DD §7.2)
    Commit(CommitNode),
    /// Task (DD §2.4)
    Task(TaskNode),
    /// AgentSession (DD §2.3)
    AgentSession(AgentSessionNode),
    /// PullRequest (DD §7.3)
    PullRequest(PullRequestNode),
    /// File (DD §7.4)
    File(FileNode),
    /// Symbol (DD §7.5)
    Symbol(SymbolNode),
    /// TestRun (DD §7.6)
    TestRun(TestRunNode),
    /// Issue (DD §7.7)
    Issue(IssueNode),
}

/// 11 Node 类型 kind 标签 (per INV-WC-05 enum completeness check).
///
/// 跟 `NodePayload` variant 一一对应; UT `node_11_types_enum_complete` 枚举校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Repository Node
    Repository,
    /// Branch Node (含 Mainline)
    Branch,
    /// Worktree Node
    Worktree,
    /// Commit Node
    Commit,
    /// Task Node
    Task,
    /// AgentSession Node
    AgentSession,
    /// PullRequest Node
    PullRequest,
    /// File Node
    File,
    /// Symbol Node
    Symbol,
    /// TestRun Node
    TestRun,
    /// Issue Node
    Issue,
}

impl NodeKind {
    /// `NodePayload` variant 数量 — INV-WC-05 守门用
    pub const COUNT: usize = 11;

    /// 所有 11 个 kind (按 enum 顺序)
    pub const ALL: [NodeKind; Self::COUNT] = [
        Self::Repository,
        Self::Branch,
        Self::Worktree,
        Self::Commit,
        Self::Task,
        Self::AgentSession,
        Self::PullRequest,
        Self::File,
        Self::Symbol,
        Self::TestRun,
        Self::Issue,
    ];
}

impl NodePayload {
    /// 提取对应的 `NodeKind`.
    pub fn kind(&self) -> NodeKind {
        match self {
            Self::Repository(_) => NodeKind::Repository,
            Self::Branch(_) => NodeKind::Branch,
            Self::Worktree(_) => NodeKind::Worktree,
            Self::Commit(_) => NodeKind::Commit,
            Self::Task(_) => NodeKind::Task,
            Self::AgentSession(_) => NodeKind::AgentSession,
            Self::PullRequest(_) => NodeKind::PullRequest,
            Self::File(_) => NodeKind::File,
            Self::Symbol(_) => NodeKind::Symbol,
            Self::TestRun(_) => NodeKind::TestRun,
            Self::Issue(_) => NodeKind::Issue,
        }
    }

    /// 提取 RepoId (用于 GraphRepository 边界筛选).
    pub fn repo_id(&self) -> Option<RepoId> {
        match self {
            Self::Repository(n) => Some(n.id),
            Self::Branch(n) => Some(n.repo_id),
            Self::Worktree(n) => Some(n.repo_id),
            Self::Commit(n) => Some(n.repo_id),
            Self::Task(n) => Some(n.repo_id),
            Self::AgentSession(n) => Some(n.repo_id),
            Self::PullRequest(n) => Some(n.repo_id),
            Self::File(n) => Some(n.repo_id),
            Self::Symbol(_) => None,
            Self::TestRun(_) => None,
            Self::Issue(n) => Some(n.repo_id),
        }
    }
}

// =========================================================================
// 11 Node struct (per DD §2.1..§2.4 + §7.2..§7.7)
// =========================================================================

/// Repository Node (per DD §2.2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryNode {
    /// Repo UUID
    pub id: RepoId,
    /// Repository name (e.g. "Star")
    pub name: String,
    /// Repository URL (git remote)
    pub url: String,
    /// Default branch (e.g. "main")
    pub default_branch: String,
    /// Active Worktree count (per L0 zoom)
    pub active_worktree_count: u32,
    /// Risk count
    pub risk_count: u32,
    /// Ready count
    pub ready_count: u32,
    /// Merged count
    pub merged_count: u32,
    /// Last commit time
    pub last_commit_at: DateTime<Utc>,
}

/// Branch Node (per DD §2.1 path; Mainline 也是 BranchNode)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BranchNode {
    /// Branch UUID
    pub id: crate::types::BranchId,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Branch name (e.g. "main", "feature/payment")
    pub name: String,
    /// 是否 Mainline (per DD §6.1 L1 zoom)
    pub is_mainline: bool,
    /// Latest commit SHA
    pub head_sha: String,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// Worktree Node (per DD §2.1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorktreeNode {
    /// Worktree UUID
    pub id: WorktreeId,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Worktree name
    pub name: String,
    /// Branch name
    pub branch: String,
    /// Worktree 本地路径
    pub path: PathBuf,
    /// HEAD commit SHA
    pub head_sha: String,
    /// Ahead of main
    pub ahead: u32,
    /// Behind main
    pub behind: u32,
    /// Working tree dirty?
    pub dirty: bool,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Created time
    pub created_at: DateTime<Utc>,
    /// Merged time (None = 未合并)
    pub merged_at: Option<DateTime<Utc>>,
}

/// Commit Node (per DD §7.2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitNode {
    /// Commit SHA
    pub sha: String,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Author name
    pub author: String,
    /// Author email
    pub author_email: String,
    /// Committer name
    pub committer: String,
    /// Commit message
    pub message: String,
    /// Parent SHAs
    pub parents: Vec<String>,
    /// Tree SHA
    pub tree_sha: String,
    /// Authored at
    pub authored_at: DateTime<Utc>,
    /// Committed at
    pub committed_at: DateTime<Utc>,
    /// Worktree BASED_ON 来源
    pub worktree_id: Option<WorktreeId>,
}

/// Task Node (per DD §2.4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskNode {
    /// Task UUID
    pub id: TaskId,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Task title
    pub title: String,
    /// Task description
    pub description: Option<String>,
    /// Status
    pub status: TaskStatus,
    /// Assignee user
    pub assignee_id: Option<UserId>,
    /// Worktrees implementing this task
    pub worktree_ids: Vec<WorktreeId>,
    /// Due date
    pub due_date: Option<DateTime<Utc>>,
    /// Created time
    pub created_at: DateTime<Utc>,
}

/// AgentSession Node (per DD §2.3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSessionNode {
    /// Agent UUID
    pub id: AgentId,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Agent type (e.g. "claude-code", "codex")
    pub agent_type: String,
    /// Model (e.g. "claude-sonnet-4")
    pub model: String,
    /// Agent status (per SRS-STAR-AGENT-RUNTIME-001 14 状态)
    pub status: AgentStatus,
    /// Current task
    pub task_id: Option<TaskId>,
    /// Current worktree (WORKS_ON)
    pub worktree_id: Option<WorktreeId>,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Tool call count
    pub tool_calls: u32,
    /// Result state
    pub result_state: AgentResultState,
}

/// PullRequest Node (per DD §7.3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequestNode {
    /// PR UUID
    pub id: PRId,
    /// Repository UUID
    pub repo_id: RepoId,
    /// PR number
    pub number: u32,
    /// Title
    pub title: String,
    /// Body
    pub body: String,
    /// State (open / merged / closed)
    pub state: PRState,
    /// Source branch
    pub source_branch: String,
    /// Target branch
    pub target_branch: String,
    /// Worktree reviewed (REVIEWS)
    pub worktree_id: Option<WorktreeId>,
    /// Review count
    pub review_count: u32,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Merged at
    pub merged_at: Option<DateTime<Utc>>,
}

/// File Node (per DD §7.4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileNode {
    /// File UUID
    pub id: Uuid,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Path relative to repo root
    pub path: PathBuf,
    /// Size bytes
    pub size_bytes: u64,
    /// Line count
    pub lines: u32,
    /// Language (e.g. "rust", "typescript")
    pub language: String,
}

/// Symbol Node (per DD §7.5, P2 zoom L5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolNode {
    /// Symbol UUID
    pub id: Uuid,
    /// File UUID
    pub file_id: Uuid,
    /// Qualified name (e.g. "crate::module::function")
    pub qualified_name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Line start
    pub line_start: u32,
    /// Line end
    pub line_end: u32,
    /// Signature
    pub signature: Option<String>,
}

/// TestRun Node (per DD §7.6)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestRunNode {
    /// Test run UUID
    pub id: TestId,
    /// Worktree UUID (VALIDATES)
    pub worktree_id: WorktreeId,
    /// Test framework (e.g. "cargo test", "jest")
    pub framework: String,
    /// Status
    pub status: TestStatus,
    /// Total tests
    pub total: u32,
    /// Passed
    pub passed: u32,
    /// Failed
    pub failed: u32,
    /// Skipped
    pub skipped: u32,
    /// Duration milliseconds
    pub duration_ms: u64,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Finished at
    pub finished_at: Option<DateTime<Utc>>,
}

/// Issue Node (per DD §7.7)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IssueNode {
    /// Issue UUID
    pub id: IssueId,
    /// Repository UUID
    pub repo_id: RepoId,
    /// Issue number
    pub number: u32,
    /// Title
    pub title: String,
    /// Body
    pub body: String,
    /// State
    pub state: IssueState,
    /// Labels
    pub labels: Vec<String>,
    /// Assignee
    pub assignee_id: Option<UserId>,
    /// Related worktrees
    pub worktree_ids: Vec<WorktreeId>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BranchId, WorktreeId};

    fn count_node_kinds() -> usize {
        NodeKind::ALL.len()
    }

    #[test]
    fn node_kind_count_is_eleven() {
        assert_eq!(NodeKind::COUNT, 11);
        assert_eq!(count_node_kinds(), 11);
    }

    #[test]
    fn all_kinds_unique() {
        let kinds = NodeKind::ALL;
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate kind at index {} vs {}", i, j);
                }
            }
        }
    }

    #[test]
    fn node_payload_kind_matches_variant() {
        let repo = RepositoryNode {
            id: RepoId::new_v4(),
            name: "Star".into(),
            url: "git@github.com:Ulysses/Star.git".into(),
            default_branch: "main".into(),
            active_worktree_count: 0,
            risk_count: 0,
            ready_count: 0,
            merged_count: 0,
            last_commit_at: Utc::now(),
        };
        let payload = NodePayload::Repository(repo);
        assert_eq!(payload.kind(), NodeKind::Repository);
    }

    #[test]
    fn worktree_node_roundtrip_serde() {
        let node = WorktreeNode {
            id: WorktreeId::new_v4(),
            repo_id: RepoId::new_v4(),
            name: "wt-a".into(),
            branch: "feat/a".into(),
            path: PathBuf::from("/tmp/wt-a"),
            head_sha: "abc123".into(),
            ahead: 2,
            behind: 0,
            dirty: false,
            last_activity: Utc::now(),
            created_at: Utc::now(),
            merged_at: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: WorktreeNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, parsed);
    }

    #[test]
    fn branch_node_marks_mainline() {
        let _branch = BranchNode {
            id: BranchId::new_v4(),
            repo_id: RepoId::new_v4(),
            name: "main".into(),
            is_mainline: true,
            head_sha: "0".repeat(40),
            updated_at: Utc::now(),
        };
    }
}
