//! `v3_symbol_overlap.rs` — V3 Symbol Overlap 层 (per DD §13 + §35)
//!
//! 检测 (需 tree-sitter AST, per star-treesitter):
//! - `SymbolOverlap` — 跨 WT 共享 symbol

use chrono::Utc;

use graph_core::types::{RiskScore, WorktreeId};

use crate::config::RiskEngineConfig;
use crate::risk::{Risk, RiskType};

/// 输入 (per DD §13)
#[derive(Debug, Clone)]
pub struct V3Input {
    /// Worktree A
    pub worktree_a: WorktreeId,
    /// Worktree B
    pub worktree_b: WorktreeId,
    /// 共享 symbols (qualified names)
    pub shared_symbols: Vec<String>,
}

/// V3 风险检测 (per DD §35)
pub fn detect(input: &V3Input, config: &RiskEngineConfig) -> Vec<Risk> {
    if !config.v3_enabled {
        return Vec::new();
    }

    let mut out = Vec::new();
    let now = Utc::now();

    if !input.shared_symbols.is_empty() {
        let score = (input.shared_symbols.len() as f32 / 10.0).min(1.0);
        out.push(Risk {
            id: uuid::Uuid::new_v4(),
            worktree_id: input.worktree_a,
            risk_type: RiskType::SymbolOverlap,
            risk_score: RiskScore::new(score),
            reason: format!("{} shared symbols", input.shared_symbols.len()),
            detected_at: now,
            resolved_at: None,
            related_worktrees: vec![input.worktree_b],
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v3_disabled_returns_empty_by_default() {
        // V3 默认 v3_enabled=false, 即使有 shared_symbols 也返回空
        let config = RiskEngineConfig::default();
        let input = V3Input {
            worktree_a: WorktreeId::new_v4(),
            worktree_b: WorktreeId::new_v4(),
            shared_symbols: vec!["Foo::bar".into()],
        };
        let risks = detect(&input, &config);
        assert!(risks.is_empty());
    }

    #[test]
    fn v3_enabled_detects_symbol_overlap() {
        let mut config = RiskEngineConfig::default();
        config.v3_enabled = true;
        let input = V3Input {
            worktree_a: WorktreeId::new_v4(),
            worktree_b: WorktreeId::new_v4(),
            shared_symbols: vec!["Foo::bar".into(), "Baz::qux".into()],
        };
        let risks = detect(&input, &config);
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].risk_type, RiskType::SymbolOverlap);
    }
}
