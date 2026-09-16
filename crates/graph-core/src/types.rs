//! ID newtypes + score/value-object — graph-core
//!
//! Per DD-WORKTREE-CANVAS-001 §7 + §8 + §2.1 + §2.2 + §2.3 + §2.4:
//! - `RepoId` / `WorktreeId` / `BranchId` / `AgentId` / `TaskId` / `UserId` /
//!   `PRId` / `TestId` / `IssueId` 9 类 newtype UUID
//! - `RiskScore` 0-1 浮点 (CONFLICTS_WITH / OVERLAPS_WITH)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Repository UUID (per DD §2.2 `RepositoryNode.id`)
pub type RepoId = Uuid;

/// Worktree UUID (per DD §2.1 `Worktree.id`)
pub type WorktreeId = Uuid;

/// Branch UUID (per DD §2.1 path — Mainline 也是 BranchId)
pub type BranchId = Uuid;

/// Agent UUID (per DD §2.3 `AgentSessionNode.id`)
pub type AgentId = Uuid;

/// Task UUID (per DD §2.4 `TaskNode.id`)
pub type TaskId = Uuid;

/// User UUID (per DD §2.1 / §2.3 / §2.4 assignee / created_by)
pub type UserId = Uuid;

/// Pull Request UUID (per DD §7.3 `PullRequestNode.id`)
pub type PRId = Uuid;

/// Test run UUID (per DD §7.6 `TestRunNode.id`)
pub type TestId = Uuid;

/// Issue UUID (per DD §7.7 `IssueNode.id`)
pub type IssueId = Uuid;

/// Risk score 0.0..=1.0 (per DD §8.1.8 CONFLICTS_WITH `risk_score`)
///
/// Stored as `f32` for compactness; serialized to JSON as `f32` with `value` field.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RiskScore(pub f32);

impl RiskScore {
    /// Construct from f32 in range 0.0..=1.0 (clamped).
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Inner value.
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for RiskScore {
    fn default() -> Self {
        Self(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_score_clamps_out_of_range() {
        assert_eq!(RiskScore::new(-0.5).value(), 0.0);
        assert_eq!(RiskScore::new(1.5).value(), 1.0);
        assert_eq!(RiskScore::new(0.42).value(), 0.42);
    }

    #[test]
    fn id_newtypes_are_uuid_backed() {
        let id: WorktreeId = Uuid::new_v4();
        let _: RepoId = Uuid::new_v4();
        assert_ne!(id, Uuid::nil());
    }
}
