//! `canvas-renderer` — AI Worktree Graph Canvas Canvas Renderer crate.
//!
//! Per ULYS-57.3 T12 (2026-09-17):
//! - `CanvasRenderer` Trait + 8 方法 (per spec §4.1)
//! - React-Flow 11.x adapter (MVP JSON contract, per BD D-CANVAS-001)
//! - LOD 3 级 (L0 Project / L1 Repo+Worktree / L2 Task+Agent, per INV-WC-05)
//! - Viewport Virtualization (1000 WT 只渲染可见, per INV-WC-06)
//! - 5 View Mode 切换 (TREE / DEPENDENCY / RISK / AGENT / HISTORY, per FR-UI-007..011)
//! - Focus Mode 1/2/3-hop (per INV-WC-07, DD §37)
//!
//! 守门:
//! - #1 v25 `cargo test -p canvas-renderer --lib` 4/4 pass
//! - #6 v2 错误码 6-field
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]`
//! - #13 W/T/M 100% 覆盖 (本期仅 Rust crate, 无 DB; UI 渲染留 frontend T14)

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
// 守门 #6 v2: CanvasError 6-field 设计 (code / message / source / location / context / trace_id)
#![allow(clippy::result_large_err)]

pub mod error;
pub mod focus;
pub mod lod;
pub mod reactflow;
pub mod renderer;
pub mod view_mode;
pub mod viewport;

pub use error::CanvasError;
pub use focus::{FocusResult, FocusTransparency};
pub use lod::{LodLevel, LodSelector};
pub use reactflow::{ReactFlowAdapter, ReactFlowEdge, ReactFlowNode, ReactFlowSpec};
pub use renderer::{CanvasRenderer, RenderState};
pub use view_mode::{ViewMode, ViewModeLayout};
pub use viewport::{should_render, Viewport, VirtualizationPolicy};