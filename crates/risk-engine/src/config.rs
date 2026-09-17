//! `config.rs` — `RiskEngineConfig` (per DD §13 + spec §2.2)

use serde::{Deserialize, Serialize};

/// RiskEngine 配置 (per DD §13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEngineConfig {
    /// V1 enabled (Git Status + File Overlap)
    pub v1_enabled: bool,
    /// V2 enabled (Diff Overlap + Line Overlap)
    pub v2_enabled: bool,
    /// V3 enabled (Symbol Overlap + AST)
    pub v3_enabled: bool,
    /// Stale 阈值 (天), 默认 7
    pub stale_threshold_days: u32,
    /// Divergence ahead 阈值, 默认 5
    pub divergence_ahead_threshold: u32,
    /// Divergence behind 阈值, 默认 5
    pub divergence_behind_threshold: u32,
}

impl Default for RiskEngineConfig {
    fn default() -> Self {
        Self {
            v1_enabled: true,
            v2_enabled: true,
            v3_enabled: false, // V3 需 tree-sitter, 阶段 1 默认关闭
            stale_threshold_days: 7,
            divergence_ahead_threshold: 5,
            divergence_behind_threshold: 5,
        }
    }
}
