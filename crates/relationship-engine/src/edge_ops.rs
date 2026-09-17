//! `edge_ops.rs` — 13 Edge 类型 validation + 4 高风险 metadata (per DD §13 + INV-WC-04)

use serde::{Deserialize, Serialize};

use graph_core::edge::EdgeKind;

use crate::error::RelationshipError;

/// 4 高风险 Edge 类型 (per DD §13 高风险元数据要求)
/// - CONFLICTS_WITH
/// - OVERLAPS_WITH
/// - DEPENDS_ON
/// - BLOCKS
///
/// 高风险 edge 必须携带 6 字段 metadata (risk_score / reason / etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskKind {
    /// CONFLICTS_WITH
    ConflictsWith,
    /// OVERLAPS_WITH
    OverlapsWith,
    /// DEPENDS_ON
    DependsOn,
    /// BLOCKS
    Blocks,
}

impl HighRiskKind {
    /// 总数 — INV-WC-04 守门
    pub const COUNT: usize = 4;

    /// 全部 4 个高风险 kind
    pub const ALL: [HighRiskKind; Self::COUNT] = [
        Self::ConflictsWith,
        Self::OverlapsWith,
        Self::DependsOn,
        Self::Blocks,
    ];
}

impl HighRiskKind {
    /// 从 EdgeKind 转换 (失败 = 不是高风险)
    pub fn from_edge_kind(kind: EdgeKind) -> Option<Self> {
        match kind {
            EdgeKind::ConflictsWith => Some(Self::ConflictsWith),
            EdgeKind::OverlapsWith => Some(Self::OverlapsWith),
            EdgeKind::DependsOn => Some(Self::DependsOn),
            EdgeKind::Blocks => Some(Self::Blocks),
            _ => None,
        }
    }

    /// 转换回 EdgeKind
    pub fn to_edge_kind(&self) -> EdgeKind {
        match self {
            Self::ConflictsWith => EdgeKind::ConflictsWith,
            Self::OverlapsWith => EdgeKind::OverlapsWith,
            Self::DependsOn => EdgeKind::DependsOn,
            Self::Blocks => EdgeKind::Blocks,
        }
    }
}

/// 高风险 Edge metadata (per DD §13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighRiskMetadata {
    /// Risk score (0-1)
    pub risk_score: f32,
    /// 共享 files (paths)
    pub shared_files: Vec<String>,
    /// 共享 symbols (qualified names)
    pub shared_symbols: Vec<String>,
    /// 检测时间
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// Reason (人类可读)
    pub reason: String,
    /// Confidence (0-1)
    pub confidence: f32,
}

/// Edge 创建参数 (per DD §13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCreateRequest {
    /// Edge kind
    pub kind: EdgeKind,
    /// From node ID
    pub from_id: uuid::Uuid,
    /// To node ID
    pub to_id: uuid::Uuid,
    /// 高风险 metadata (可选)
    pub high_risk_metadata: Option<HighRiskMetadata>,
}

/// 验证 13 Edge 类型合法 (per DD §13 + INV-WC-04)
pub fn validate_kind(kind: EdgeKind) -> Result<(), RelationshipError> {
    // 全部 13 种 enum 值都合法 — 编译时保证
    // 此函数仅用于运行时校验来自外部的输入 (e.g. JSON 解析)
    match kind {
        EdgeKind::BasedOn
        | EdgeKind::UsesBranch
        | EdgeKind::DerivedFrom
        | EdgeKind::WorksOn
        | EdgeKind::ImplementedIn
        | EdgeKind::Modifies
        | EdgeKind::ModifiesSymbol
        | EdgeKind::ConflictsWith
        | EdgeKind::OverlapsWith
        | EdgeKind::DependsOn
        | EdgeKind::Blocks
        | EdgeKind::Supersedes
        | EdgeKind::MergedInto => Ok(()),
        // _ => Err(RelationshipError::invalid_kind(format!("{:?}", kind), "trace")),
        // 当前 enum 是穷尽枚举, 不可达
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kind_validation_13_kinds() {
        // Per TEST-DESIGN §2.2: edge_kind_validation_13_kinds
        use graph_core::edge::EdgeKind;
        let all = [
            EdgeKind::BasedOn,
            EdgeKind::UsesBranch,
            EdgeKind::DerivedFrom,
            EdgeKind::WorksOn,
            EdgeKind::ImplementedIn,
            EdgeKind::Modifies,
            EdgeKind::ModifiesSymbol,
            EdgeKind::ConflictsWith,
            EdgeKind::OverlapsWith,
            EdgeKind::DependsOn,
            EdgeKind::Blocks,
            EdgeKind::Supersedes,
            EdgeKind::MergedInto,
        ];
        assert_eq!(all.len(), 13);
        for k in all {
            assert!(validate_kind(k).is_ok());
        }
    }

    #[test]
    fn high_risk_4_metadata_complete() {
        // Per TEST-DESIGN §2.2: edge_ops_crud_4_high_risk_metadata
        use graph_core::edge::EdgeKind;
        assert_eq!(HighRiskKind::COUNT, 4);
        // 4 高风险 EdgeKind → 4 HighRiskKind 映射
        let mappings = [
            (EdgeKind::ConflictsWith, HighRiskKind::ConflictsWith),
            (EdgeKind::OverlapsWith, HighRiskKind::OverlapsWith),
            (EdgeKind::DependsOn, HighRiskKind::DependsOn),
            (EdgeKind::Blocks, HighRiskKind::Blocks),
        ];
        for (ek, hrk) in mappings {
            assert_eq!(HighRiskKind::from_edge_kind(ek), Some(hrk));
            assert_eq!(hrk.to_edge_kind(), ek);
        }

        // 非高风险
        assert_eq!(HighRiskKind::from_edge_kind(EdgeKind::BasedOn), None);
        assert_eq!(HighRiskKind::from_edge_kind(EdgeKind::Supersedes), None);

        // 验证 6 字段 metadata
        let m = HighRiskMetadata {
            risk_score: 0.5,
            shared_files: vec!["x.rs".into()],
            shared_symbols: vec!["Foo::bar".into()],
            detected_at: chrono::Utc::now(),
            reason: "test".into(),
            confidence: 0.9,
        };
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
