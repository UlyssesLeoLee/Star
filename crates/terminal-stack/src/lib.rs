//! `terminal-stack` — Terminal Stack backend (per ULYS-200 + ULYS-220 P1-A + ULYS-222 P1-C + FR-ORCA-036/039 §12 v1.0)
//!
//! **目的**: 画布内嵌终端栈的 backend 数据结构层 + SQLite WAL 持久化 + WebSocket 协议层.
//!
//! **MVP v0 范围**:
//! - ULYS-200: `ScrollbackBuffer` (FR-ORCA-036) + `SplitTree` (§12 附加) 内存 + JSON
//! - ULYS-220 P1-A: `TerminalStackPersistence` SQLite WAL 持久化层
//! - ULYS-222 P1-C: WebSocket 协议层 (`ws` 子模块) + hub pub/sub + handler
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - ULYS-221 P1-B Restart recovery 算法 (per `terminal_stack_pane` / `terminal_stack_scrollback_line` 表)
//! - ULYS-223 P1-D 前端 UI + xterm.js addon
//! - ANSI escape sequence 解析 (per FR-ORCA-039)
//! - PTY 进程管理 (per description "实装期走原型对比")
//! - HTTP 路由注册 (`/v1/remote/terminal/{runtime_id}` 在 P1-D 集成阶段添加)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies] 或 per-crate (rusqlite 同款)
//! - #DB-13 W/T/M 派生 (per cli_session_registry 守门 同款)
//! - #13 L0 协调派生 (per in-process pub/sub + Mailbox 模式; ws::hub 模块)

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod persistence;
pub mod scrollback_buffer;
pub mod split_pane;
pub mod ws;

pub use persistence::{TerminalStackPersistence, TerminalStackPersistenceError};
pub use scrollback_buffer::{
    ScrollbackBuffer, ScrollbackError as BufferError, ScrollbackLine, ScrollbackSource,
};
pub use split_pane::{
    PaneNode, SplitDirection, SplitError, SplitNode, SplitPane, SplitTree, SplitTreeKind,
};
// ULYS-222 P1-C re-exports
pub use ws::handler::{ws_handler, WsTerminalSession};
pub use ws::hub::{WsHub, WsHubConfig, WsPaneStream};
pub use ws::integration::{WsRecoverySource, WsSnapshotLoader};
pub use ws::protocol::{
    ClientMessage, PtyExitReason, ServerMessage, SplitUpdate, WsProtocolError, WsSnapshot,
};
