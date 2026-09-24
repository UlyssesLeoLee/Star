//! `terminal-stack::ws` — WebSocket Protocol Layer (per ULYS-222 P1-C)
//!
//! **目的 (per ULYS-200.3 description §2)**: server 端 WebSocket 协议层, 接收前端
//! xterm.js / `@xterm/addon-fit` 请求并推送 scrollback 数据.
//!
//! **子模块**:
//! - [`protocol`]: JSON 消息 schema (ClientMessage / ServerMessage) + 错误类型
//! - [`hub`]: in-process pub/sub hub per pane (per 守门 #13 L0 协调派生 — Mailbox 模式)
//! - [`handler`]: axum WS endpoint (`WebSocketUpgrade` extractor) + 事件循环
//! - [`integration`]: 持久化层 (P1-A) 适配 + P1-B recovery seam (trait abstraction)
//!
//! **协议契约** (per `frontend/src/components/remote/XtermViewer.tsx` line 134/149/150):
//! - 客户端 outbound JSON: `{type: "stdin", data}` / `{type: "resize", cols, rows}`
//! - 服务端 inbound JSON envelope: 详见 [`protocol::ServerMessage`]
//!
//! **不在 P1-C 范围**:
//! - ❌ HTTP 路由注册 (`crates/api/arg/sse_hub` 同款 axum::Router 集成在 P1-D 实装)
//! - ❌ Restart recovery 算法 (per ULYS-221 P1-B; P1-C 通过 [`integration::WsSnapshotLoader`] seam 对接)
//! - ❌ Persistence 层本身 (per ULYS-220 P1-A; P1-C 直接消费 [`crate::persistence::TerminalStackPersistence`])
//! - ❌ PTY 进程管理 (per ULYS-200 description §5 "实装期走原型对比")
//! - ❌ 前端 xterm.js addon (per ULYS-223 P1-D)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies] 或 per-crate (axum 0.8 已 per-crate 在 `crates/api` 守门 #48 锁定)
//! - #13 L0 协调派生 (in-process pub/sub + Mailbox)

pub mod handler;
pub mod hub;
pub mod integration;
pub mod protocol;
