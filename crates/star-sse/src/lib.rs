// SPDX-License-Identifier: MIT OR Apache-2.0
//! crates/star-sse — SSE server (per spec/services/02-sse-streaming-spec.md §3)
//!
//! 提供 SSE (Server-Sent Events) 推送：EventRouter 维护事件存储 + Last-Event-ID
//! 重连；SseEndpoint 提供 handler 框架 (鉴权 + 30s heartbeat)。
//!
//! Phase F 阶段：in-memory 存储 (TODO: Phase F+ 接 redis)。

pub mod event_router;
pub mod sse_endpoint;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 事件类型 (per spec/services/02 §2 — 5 域事件归类)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    /// MergeRequest 域: PR/MR 状态变更
    MergeRequest,
    /// Pipeline 域: CI/CD 流水线状态
    Pipeline,
    /// Agent 域: AI 代理状态变更
    AgentState,
    /// Worktree 域: worktree 切换/分支变化
    WorktreeChange,
}

/// 事件载荷 (per spec/services/02 §3 — 通用 envelope)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 唯一事件 ID (monotonic id 格式 `evt-<n>`)
    pub id: String,
    /// 事件类型
    pub event_type: EventType,
    /// 事件源 (e.g. "scm:github", "agent:builder")
    pub source: String,
    /// Unix timestamp (秒)
    pub timestamp: i64,
    /// 业务数据
    pub data: serde_json::Value,
}

/// SSE 错误 (per spec/services/02 §6 — 4 类错误)
#[derive(Debug, Error)]
pub enum SseError {
    /// 鉴权失败 (token invalid/expired)
    #[error("auth: {0}")]
    Auth(String),
    /// 资源不存在 (e.g. event id 未找到)
    #[error("not_found: {0}")]
    NotFound(String),
    /// 网络错误 (客户端断开/超时)
    #[error("network: {0}")]
    Network(String),
    /// 内部错误 (panic/序列化等)
    #[error("internal: {0}")]
    Internal(String),
}

pub use event_router::EventRouter;
pub use sse_endpoint::SseEndpoint;
