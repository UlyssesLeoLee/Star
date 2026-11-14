//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/Argo v0.4 `crates/argo/src/model.rs`
//! (AGPL-3.0) — 权利人 2026-11-14 拍板按 Star 的 MIT 许可复用
//! (per REUSE-MAP-COLLAB-SANDBOX-001 §3 权利人决定).
//! Source: E:\Argo @ 2941596 (dev), https://github.com/UlyssesLeoLee/Argo
//! ═══════════════════════════════════════════════════════════════════

//! argo-core — Argo 多机 Agent 数据模型内核 (Star)
//!
//! Controller/Worker 多机任务调度模型: NewTask/TaskRow/NodeRow/WorkerNode/
//! Event 流/Invocation (支持 claude/codex/hermes/orca/minimax/multica/star 7 CLI)
#![allow(missing_docs, unreachable_pub)]

pub mod model;

pub use model::*;
