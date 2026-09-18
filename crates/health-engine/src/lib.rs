//! `health-engine` — AI Worktree Graph Canvas Health Engine crate.
//!
//! Per ULYS-57.2 T5 (2026-09-16):
//! - `HealthEngine` Trait + 4 方法 (per DD-WORKTREE-CANVAS-001 §14)
//! - 13 因素加权扣分 (per DD §34, INV-WC-02, D-HEALTH-001)
//! - 3 Health 阈值 (GREEN ≥80 / YELLOW 50-79 / RED <50, per BD §7.3)
//! - Health Score 扣分明细可展开 (per FR-WT-028, R8 Explainable State)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod engine;
pub mod error;
pub mod score;

pub use engine::{HealthEngine, InMemoryHealthEngine};
pub use error::HealthError;
pub use score::{
    compute, Deduction, HealthContext, HealthLevel, HealthScore, HealthScorePoint, Recommendation,
    WorktreeHealthInput,
};
