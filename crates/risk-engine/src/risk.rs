//! `risk.rs` — Risk 数据模型 (per DD §13 + spec §2.2)
//!
//! - 11 Risk 类型 (per spec §2.2)
//! - `RiskEdgeMetadata` 6 字段 (per spec §2.2)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use graph_core::types::{RiskScore, WorktreeId};

/// 11 Risk 类型 (per DD §13 + spec §2.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskType {
    /// Merge conflict risk
    MergeConflict,
    /// File overlap risk
    FileOverlap,
    /// Symbol overlap risk
    SymbolOverlap,
    /// Stale risk (long inactivity)
    Stale,
    /// Divergence risk (ahead/behind 过大)
    Divergence,
    /// Dependency risk (unresolved deps)
    Dependency,
    /// Test failure risk
    TestFailure,
    /// Build failure risk
    BuildFailure,
    /// Agent incomplete risk
    AgentIncomplete,
    /// Review missing risk
    ReviewMissing,
    /// Superseded risk
    Superseded,
}

impl RiskType {
    /// 类型总数 — INV-WC-03 守门
    pub const COUNT: usize = 11;

    /// 全部 11 个 type
    pub const ALL: [RiskType; Self::COUNT] = [
        Self::MergeConflict,
        Self::FileOverlap,
        Self::SymbolOverlap,
        Self::Stale,
        Self::Divergence,
        Self::Dependency,
        Self::TestFailure,
        Self::BuildFailure,
        Self::AgentIncomplete,
        Self::ReviewMissing,
        Self::Superseded,
    ];
}

/// Risk 事件 (per DD §13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    /// Risk UUID
    pub id: Uuid,
    /// Worktree UUID
    pub worktree_id: WorktreeId,
    /// Risk type
    pub risk_type: RiskType,
    /// Risk score (0-1)
    pub risk_score: RiskScore,
    /// 人类可读 reason
    pub reason: String,
    /// Detected at
    pub detected_at: DateTime<Utc>,
    /// Resolved at
    pub resolved_at: Option<DateTime<Utc>>,
    /// 关联的其他 Worktree (per spec §2.2)
    pub related_worktrees: Vec<WorktreeId>,
}

/// Risk Edge Metadata 6 字段 (per spec §2.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEdgeMetadata {
    /// Risk score (0-1)
    pub risk_score: RiskScore,
    /// Shared files (paths)
    pub shared_files: Vec<String>,
    /// Shared symbols (qualified names)
    pub shared_symbols: Vec<String>,
    /// Detected at
    pub detected_at: DateTime<Utc>,
    /// Reason (人类可读)
    pub reason: String,
    /// Confidence (0-1)
    pub confidence: f32,
}

impl RiskEdgeMetadata {
    /// 构造空 metadata
    pub fn empty() -> Self {
        Self {
            risk_score: RiskScore::new(0.0),
            shared_files: Vec::new(),
            shared_symbols: Vec::new(),
            detected_at: Utc::now(),
            reason: String::new(),
            confidence: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_type_11_variants_enum_complete() {
        // Per TEST-DESIGN §2.2: 11 risk 类型枚举完整
        assert_eq!(RiskType::ALL.len(), 11);
        // 全部唯一
        for (i, a) in RiskType::ALL.iter().enumerate() {
            for (j, b) in RiskType::ALL.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn risk_edge_metadata_six_fields() {
        let m = RiskEdgeMetadata::empty();
        // 6 字段: risk_score, shared_files, shared_symbols, detected_at, reason, confidence
        let _ = (
            m.risk_score,
            m.shared_files,
            m.shared_symbols,
            m.detected_at,
            m.reason,
            m.confidence,
        );
    }
}
