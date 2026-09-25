//! `terminal-stack` — Terminal Stack backend (per ULYS-200 + FR-ORCA-036/039 §12 v1.0)
//!
//! **目的**: 画布内嵌终端栈的 backend 数据结构层 (per v1.0 §19 风险/已知缺口第 3 项
//! "Ghostty-class WebGL 终端栈需独立选型" 拍板 = 候选 B 后, 在 Rust 侧实装 scrollback 持久化 + split pane 数据结构).
//!
//! **MVP v0 范围 (本 issue ULYS-200)**:
//! - `ScrollbackBuffer` (FR-ORCA-036): 内存 ring buffer + JSON round-trip
//! - `SplitTree` (附加需求 per spec §12): 树状分裂数据结构 + JSON round-trip
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - SQLite WAL 持久化层 (per `cli_session_registry` 模式扩展)
//! - 与 `cli_session.scrollback_bytes` 集成
//! - Restart recovery (从 disk replay 到 in-memory)
//! - ANSI escape sequence 解析 (per FR-ORCA-039 TUI Transcript Capture)
//! - 前端 UI 渲染 + xterm.js addon (per ULYS-200 description "附加需求")
//! - WebSocket 协议层 (frontend 已有 XtermViewer.tsx, 381 lines)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod persistence;
pub mod recovery;
pub mod scrollback_buffer;
pub mod split_pane;
pub mod ws;

pub use scrollback_buffer::{
    ScrollbackBuffer, ScrollbackError as BufferError, ScrollbackLine, ScrollbackSource,
};
pub use split_pane::{
    PaneNode, SplitDirection, SplitError, SplitNode, SplitPane, SplitTree, SplitTreeKind,
};
