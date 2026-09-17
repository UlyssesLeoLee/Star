//! `risk-engine` — AI Worktree Graph Canvas Risk Engine crate.
//!
//! Per ULYS-57.2 T4 (2026-09-16):
//! - `RiskEngine` Trait + 6 方法 (per DD-WORKTREE-CANVAS-001 §13)
//! - V1 (Git Status) + V2 (Diff Overlap) + V3 (Symbol Overlap) 分层 (per DD §35)
//! - 11 Risk 类型 emit (per DD §13 + spec §2.2)
//! - `RiskEdgeMetadata` 6 字段 (per spec §2.2)
//! - RiskEngine ↔ HealthEngine 集成 (Risk 变化触发 Health 重算, per INV-WC-02/03)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 `RiskError` 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//! - #13 a/d W/T/M 100% 覆盖 (worktree_conflict T)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod config;
pub mod engine;
pub mod error;
pub mod risk;
pub mod v1_git_status;
pub mod v2_diff_overlap;
pub mod v3_symbol_overlap;

pub use config::RiskEngineConfig;
pub use engine::{ConflictPrediction, InMemoryRiskEngine, RiskEngine};
pub use error::RiskError;
pub use risk::{Risk, RiskEdgeMetadata, RiskType};
