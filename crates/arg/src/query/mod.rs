// SPDX-License-Identifier: MIT OR Apache-2.0
//! Query templates (per DD-AGENT-RELATIONSHIP-001 §3.1).
//!
//! Three submodules holding the 8 拓扑成就 Cypher (already closed at
//! DD v0.1, `G-6` ✅), the 7 行为成就 event pattern placeholders
//! (P3-E ARG.8) and the 5 产出成就 aggregate-metric placeholders
//! (P3-E ARG.8).

/// 7 协作行为 event pattern placeholders (per DD §3.1, P3-E ARG.8).
pub mod behavior;
/// 5 产出质量 aggregate-metric placeholders (per DD §3.1, P3-E ARG.8).
pub mod output;
/// 8 拓扑成就 Cypher templates (per DD §6).
pub mod topology;

pub use behavior::all_behavior_patterns;
pub use output::all_output_metrics;
pub use topology::all_topology_cyphers;
