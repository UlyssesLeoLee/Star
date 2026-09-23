//! `terminal-stack` — Terminal Stack backend (per ULYS-200 + ULYS-220 P1-A + ULYS-221 P1-B + FR-ORCA-036/039 §12 v1.0)
//!
//! **目的**: 画布内嵌终端栈的 backend 数据结构层 + SQLite WAL 持久化 + restart recovery 算法.
//!
//! **MVP v0 范围**:
//! - ULYS-200: `ScrollbackBuffer` (FR-ORCA-036) + `SplitTree` (§12 附加) 内存 + JSON
//! - ULYS-220 P1-A: `TerminalStackPersistence` SQLite WAL 持久化层
//! - ULYS-221 P1-B: `recover_scrollback()` restart recovery 算法 + `RecoveryRequest` / `RecoveryResponse`
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - ULYS-222 P1-C WebSocket 协议层
//! - ULYS-223 P1-D 前端 UI + xterm.js addon
//! - ANSI escape sequence 解析 (per FR-ORCA-039)
//! - PTY 进程管理 (per description "实装期走原型对比")
//! - 跨 pane 拆分场景恢复 (per `pane_id` 显式注入路径留 P2)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies] 或 per-crate (rusqlite 同款)
//! - #DB-13 W/T/M 派生 (per cli_session_registry 守门 同款)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod persistence;
pub mod recovery;
pub mod scrollback_buffer;
pub mod split_pane;

pub use persistence::{TerminalStackPersistence, TerminalStackPersistenceError};
pub use recovery::{recover_scrollback, RecoveryError, RecoveryRequest, RecoveryResponse};
pub use scrollback_buffer::{
    ScrollbackBuffer, ScrollbackError as BufferError, ScrollbackLine, ScrollbackSource,
};
pub use split_pane::{
    PaneNode, SplitDirection, SplitError, SplitNode, SplitPane, SplitTree, SplitTreeKind,
};
