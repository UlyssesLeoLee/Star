//! `relationship-engine` — AI Worktree Graph Canvas Relationship Engine crate.
//!
//! Per ULYS-57.2 T7 (2026-09-16):
//! - `RelationshipEngine` Trait + 13 Edge ops + 4 高风险 Edge metadata (per DD §13, INV-WC-04)
//! - Edge CRUD (create / read / update / delete / merge / supersede)
//! - Edge 查询 (1-hop / 2-hop / 3-hop, per DD §37)
//!
//! 守门:
//! - #1 v25 cargo test 单 crate 实证
//! - #6 v2 `RelationshipError` 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]

pub mod edge_ops;
pub mod engine;
pub mod error;
pub mod n_hop;

pub use engine::{InMemoryRelationshipEngine, RelationshipEngine};
pub use error::RelationshipError;
pub use n_hop::NHopQuery;
