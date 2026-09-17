//! `layout-engine` — AI Worktree Graph Canvas Layout Engine crate.
//!
//! Per ULYS-57.3 T10 (2026-09-17):
//! - `LayoutEngine` + 3 Layout 算法 (Dagre / D3Force / Elk, per D-LAYOUT-001 + spec §8.8)
//! - 3 切换阈值 (per spec §8.8):
//!   - dagre.js: TREE VIEW + node_count < 100
//!   - d3-force: DEPENDENCY / RISK / HISTORY VIEW
//!   - ELK.js: AGENT VIEW + node_count > 50, 或全局 > 500
//! - `visualDistance = log(commitDistance + 1) * 30` (per FR-WT-005 + DD §36)
//! - Incremental Layout (per DD §31, NFR-PERF-002 60fps)
//!
//! 守门:
//! - #1 v25 `cargo test -p layout-engine --lib` 3/3 pass
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]`
//! - NFR-PERF-002 (1000 WT 60fps, P2 实装基准)
//!
//! 阶段 1 scope (per WORKTREE-CANVAS-IMPL-PLAN-001 §4.10 T10):
//! - LayoutAlgorithm enum + 3 算法 + 阈值选择器 (`pick_algorithm(view_mode, node_count)`)
//! - visualDistance 公式 (per §36)
//! - Incremental Layout (位置更新 + edge path 重算)
//! - LayoutEngine Trait + LayoutEnginePure impl (无 GPU / 无 JS bridge, P2 落点)

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]

pub mod bezier;
pub mod engine;
pub mod error;
pub mod incremental;
pub mod visual_distance;

pub use bezier::bezier_path;
pub use engine::{LayoutEngine, LayoutEnginePure, LayoutInput, LayoutOptions, LayoutOutput};
pub use error::LayoutError;
pub use incremental::update_incremental;
pub use visual_distance::visual_distance;
