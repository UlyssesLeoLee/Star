//! `edge.rs` — 13 Edge 类型 enum + struct (per DD §8)
//!
//! Per INV-WC-04: 13 Edge 类型严格 13 个, 不允许多/少
//!
//! 13 Edge 类型清单 (per DD §8.1):
//!  1. BASED_ON         (Worktree → Commit)         DD §8.1.1
//!  2. USES_BRANCH      (Worktree → Branch)         DD §8.1.2
//!  3. DERIVED_FROM     (Worktree → Worktree)       DD §8.1.3
//!  4. WORKS_ON         (AgentSession → Worktree)   DD §8.1.4
//!  5. IMPLEMENTED_IN   (Task → Worktree)           DD §8.1.5
//!  6. MODIFIES         (Worktree → File)           DD §8.1.6
//!  7. MODIFIES_SYMBOL  (Worktree → Symbol, P2)     DD §8.1.7
//!  8. CONFLICTS_WITH   (Worktree → Worktree)       DD §8.1.8
//!  9. OVERLAPS_WITH    (Worktree → Worktree)       DD §8.1.9
//! 10. DEPENDS_ON       (Worktree → Worktree)       DD §8.1.10
//! 11. BLOCKS           (Worktree → Worktree)       DD §8.1.11
//! 12. SUPERSEDES       (Worktree → Worktree)       DD §8.1.12
//! 13. MERGED_INTO      (Worktree → Branch)         DD §8.1.13

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{HumanState, MergeStrategy};
use crate::types::{AgentId, BranchId, RiskScore, TaskId, UserId, WorktreeId};

/// Tagged enum dispatching 13 Edge 类型 (per INV-WC-04)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EdgePayload {
    /// (Worktree → Commit)
    BasedOn(BasedOnEdge),
    /// (Worktree → Branch)
    UsesBranch(UsesBranchEdge),
    /// (Worktree → Worktree)
    DerivedFrom(DerivedFromEdge),
    /// (AgentSession → Worktree)
    WorksOn(WorksOnEdge),
    /// (Task → Worktree)
    ImplementedIn(ImplementedInEdge),
    /// (Worktree → File)
    Modifies(ModifiesEdge),
    /// (Worktree → Symbol, P2)
    ModifiesSymbol(ModifiesSymbolEdge),
    /// (Worktree → Worktree)
    ConflictsWith(ConflictsWithEdge),
    /// (Worktree → Worktree)
    OverlapsWith(OverlapsWithEdge),
    /// (Worktree → Worktree)
    DependsOn(DependsOnEdge),
    /// (Worktree → Worktree)
    Blocks(BlocksEdge),
    /// (Worktree → Worktree)
    Supersedes(SupersedesEdge),
    /// (Worktree → Branch)
    MergedInto(MergedIntoEdge),
}

/// 13 Edge 类型 kind 标签 (per INV-WC-04 enum completeness check)
///
/// 跟 `EdgePayload` variant 一一对应; UT `edge_13_types_enum_complete` 枚举校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// BASED_ON (Worktree → Commit)
    BasedOn,
    /// USES_BRANCH (Worktree → Branch)
    UsesBranch,
    /// DERIVED_FROM (Worktree → Worktree)
    DerivedFrom,
    /// WORKS_ON (AgentSession → Worktree)
    WorksOn,
    /// IMPLEMENTED_IN (Task → Worktree)
    ImplementedIn,
    /// MODIFIES (Worktree → File)
    Modifies,
    /// MODIFIES_SYMBOL (Worktree → Symbol, P2)
    ModifiesSymbol,
    /// CONFLICTS_WITH (Worktree → Worktree)
    ConflictsWith,
    /// OVERLAPS_WITH (Worktree → Worktree)
    OverlapsWith,
    /// DEPENDS_ON (Worktree → Worktree)
    DependsOn,
    /// BLOCKS (Worktree → Worktree)
    Blocks,
    /// SUPERSEDES (Worktree → Worktree)
    Supersedes,
    /// MERGED_INTO (Worktree → Branch)
    MergedInto,
}

impl EdgeKind {
    /// `EdgePayload` variant 数量 — INV-WC-04 守门用
    pub const COUNT: usize = 13;

    /// 所有 13 个 kind (按 enum 顺序)
    pub const ALL: [EdgeKind; Self::COUNT] = [
        Self::BasedOn,
        Self::UsesBranch,
        Self::DerivedFrom,
        Self::WorksOn,
        Self::ImplementedIn,
        Self::Modifies,
        Self::ModifiesSymbol,
        Self::ConflictsWith,
        Self::OverlapsWith,
        Self::DependsOn,
        Self::Blocks,
        Self::Supersedes,
        Self::MergedInto,
    ];
}

impl EdgePayload {
    /// 提取对应的 `EdgeKind`.
    pub fn kind(&self) -> EdgeKind {
        match self {
            Self::BasedOn(_) => EdgeKind::BasedOn,
            Self::UsesBranch(_) => EdgeKind::UsesBranch,
            Self::DerivedFrom(_) => EdgeKind::DerivedFrom,
            Self::WorksOn(_) => EdgeKind::WorksOn,
            Self::ImplementedIn(_) => EdgeKind::ImplementedIn,
            Self::Modifies(_) => EdgeKind::Modifies,
            Self::ModifiesSymbol(_) => EdgeKind::ModifiesSymbol,
            Self::ConflictsWith(_) => EdgeKind::ConflictsWith,
            Self::OverlapsWith(_) => EdgeKind::OverlapsWith,
            Self::DependsOn(_) => EdgeKind::DependsOn,
            Self::Blocks(_) => EdgeKind::Blocks,
            Self::Supersedes(_) => EdgeKind::Supersedes,
            Self::MergedInto(_) => EdgeKind::MergedInto,
        }
    }
}

// =========================================================================
// 13 Edge struct (per DD §8.1)
// =========================================================================

/// 1. BASED_ON (Worktree → Commit) — per DD §8.1.1
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BasedOnEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target commit SHA
    pub to: String,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// 2. USES_BRANCH (Worktree → Branch) — per DD §8.1.2
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsesBranchEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Branch
    pub to: BranchId,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// 3. DERIVED_FROM (Worktree → Worktree) — per DD §8.1.3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DerivedFromEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target (parent) Worktree
    pub to: WorktreeId,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Merge base SHA (between source and target)
    pub merge_base_sha: Option<String>,
}

/// 4. WORKS_ON (AgentSession → Worktree) — per DD §8.1.4
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorksOnEdge {
    /// Source AgentSession
    pub from: AgentId,
    /// Target Worktree
    pub to: WorktreeId,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Ended at (None = still working)
    pub ended_at: Option<DateTime<Utc>>,
}

/// 5. IMPLEMENTED_IN (Task → Worktree) — per DD §8.1.5
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImplementedInEdge {
    /// Source Task
    pub from: TaskId,
    /// Target Worktree
    pub to: WorktreeId,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// 6. MODIFIES (Worktree → File) — per DD §8.1.6
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifiesEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target File UUID
    pub to: Uuid,
    /// Lines added
    pub lines_added: u32,
    /// Lines removed
    pub lines_removed: u32,
    /// Commit SHA
    pub commit_sha: String,
}

/// 7. MODIFIES_SYMBOL (Worktree → Symbol, P2) — per DD §8.1.7
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifiesSymbolEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Symbol UUID
    pub to: Uuid,
    /// Lines added
    pub lines_added: u32,
    /// Lines removed
    pub lines_removed: u32,
    /// Commit SHA
    pub commit_sha: String,
}

/// 8. CONFLICTS_WITH (Worktree → Worktree) — per DD §8.1.8
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictsWithEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Worktree
    pub to: WorktreeId,
    /// Risk score (0-1)
    pub risk_score: RiskScore,
    /// Shared files (paths)
    pub shared_files: Vec<String>,
    /// Shared symbols (qualified names, P2 / V3)
    pub shared_symbols: Vec<String>,
    /// Detected at
    pub detected_at: DateTime<Utc>,
    /// Reason
    pub reason: String,
    /// Confidence (0-1)
    pub confidence: f32,
}

/// 9. OVERLAPS_WITH (Worktree → Worktree) — per DD §8.1.9
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverlapsWithEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Worktree
    pub to: WorktreeId,
    /// Overlap score (0-1)
    pub overlap_score: f32,
    /// Shared files
    pub shared_files: Vec<String>,
    /// Detected at
    pub detected_at: DateTime<Utc>,
}

/// 10. DEPENDS_ON (Worktree → Worktree) — per DD §8.1.10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependsOnEdge {
    /// Source Worktree (A — depends on B)
    pub from: WorktreeId,
    /// Target Worktree (B — A depends on B)
    pub to: WorktreeId,
    /// Required state (default Merged)
    pub required_state: HumanState,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Created by user
    pub created_by: UserId,
}

/// 11. BLOCKS (Worktree → Worktree) — per DD §8.1.11
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlocksEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Worktree
    pub to: WorktreeId,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Reason
    pub reason: String,
}

/// 12. SUPERSEDES (Worktree → Worktree) — per DD §8.1.12
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupersedesEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Worktree
    pub to: WorktreeId,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Reason
    pub reason: String,
}

/// 13. MERGED_INTO (Worktree → Branch) — per DD §8.1.13
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MergedIntoEdge {
    /// Source Worktree
    pub from: WorktreeId,
    /// Target Branch
    pub to: BranchId,
    /// Merged at
    pub merged_at: DateTime<Utc>,
    /// Merge commit SHA
    pub commit_sha: String,
    /// Merge strategy
    pub merge_strategy: MergeStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kind_count_is_thirteen() {
        assert_eq!(EdgeKind::COUNT, 13);
        assert_eq!(EdgeKind::ALL.len(), 13);
    }

    #[test]
    fn all_edge_kinds_unique() {
        let kinds = EdgeKind::ALL;
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "duplicate edge kind at index {} vs {}", i, j);
                }
            }
        }
    }

    #[test]
    fn edge_payload_kind_matches_variant() {
        let wt_from = WorktreeId::new_v4();
        let wt_to = WorktreeId::new_v4();
        let edge = EdgePayload::DependsOn(DependsOnEdge {
            from: wt_from,
            to: wt_to,
            required_state: HumanState::Merged,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        });
        assert_eq!(edge.kind(), EdgeKind::DependsOn);
    }
}
