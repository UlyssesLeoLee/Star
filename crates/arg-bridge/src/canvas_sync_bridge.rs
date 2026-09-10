// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canvas UI sync bridge (P3-D.6 阶段 1 基础 任务 1.2 新增, per DD-CANVAS-AGENT-001 v0.1 §3.1).
//!
//! 桥接 `star-arg` (ARG data tier) + `star-arg-bridge` (ARG bridge tier) +
//! `star-arg-effect` (ARG effect tier) 跟 `crates/canvas-collab/` (A12 多人编辑, 阶段 1 任务 1.3 新建)
//! + `frontend/canvas` (UI 订阅端).
//!
//! 阶段 1 基础 **仅 0 业务方法骨架** (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.2):
//! - 仅 `struct` / `enum` 字段定义 + `Default` 实现
//! - 0 API 0 endpoint 0 schema 留阶段 3 集成 任务 3.2 落地 WSS 推送 / SSE 广播 / 5 WSS 协议
//!
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).
//! 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).

#![warn(missing_docs)]

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use star_arg::client::MemgraphClient;
use star_arg::models::event::ARGEvent;

use crate::period_flush::PeriodFlushWorker;
use crate::protocol::BridgeEnvelopeKind;

/// Canvas UI sync bridge 错误类型 (per 守门 #11 缺标比错标, 显式定义, 0 业务方法).
///
/// 阶段 1 基础 占位, 阶段 3 集成 任务 3.2 落地 WSS 连接错误 / SSE 广播错误
/// / canvas-collab 协议错误 4 类.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasSyncBridgeError {
    /// 阶段 1 基础 占位, 阶段 3 集成 落地 WSS 连接错误
    NotImplemented,
}

/// Canvas UI sync 事件 (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.1 + 11 共享类型扩展, 0 业务方法).
///
/// 阶段 1 基础 仅 `enum` 字段定义, 阶段 3 集成 任务 3.2 落地 WSS 推送.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanvasSyncEvent {
    /// ARG 边变化 (来自 `MemgraphEventListener`, per DD §4.10)
    EdgeChanged,
    /// agent 状态变化 (来自 `PeriodFlushWorker`, per BD §7.2)
    AgentStatusChanged,
    /// 成就解锁 (来自 `arg-effect` `achievement_engine`, per DD §4.12)
    AchievementUnlocked,
    /// 信任度变化 (来自 `arg-effect` `trust_engine`, per DD §4.13)
    TrustScoreChanged,
    /// 调度路由变化 (来自 `arg-effect` `dispatch_router`, per DD §4.14)
    DispatchRouteChanged,
    /// canvas-collab 多人编辑 WSS 广播 (阶段 1 任务 1.3 + 阶段 3 任务 3.2)
    CanvasMultiUserEvent,
}

/// Canvas UI sync 事件 payload (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.2.5 11 共享类型扩展, 0 业务方法).
///
/// 阶段 1 基础 仅字段定义, 阶段 3 集成 任务 3.2 落地 WSS 推送.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSyncEventPayload {
    /// Event id (UUIDv4)
    pub event_id: Uuid,
    /// Event 类型
    pub event_type: CanvasSyncEvent,
    /// 触发 actor id (前端用户 / 后端 agent / `None` = 系统事件)
    pub actor_id: Option<Uuid>,
    /// Event 时间戳 (UTC)
    pub timestamp: DateTime<Utc>,
    /// Event 业务 payload (JSON, 由 `CanvasSyncEvent` 决定 schema)
    pub payload: serde_json::Value,
    /// Event schema 版本 (per 守门 #13 c SCD Type 2 派生规)
    pub version: i32,
}

impl Default for CanvasSyncEventPayload {
    fn default() -> Self {
        Self {
            event_id: Uuid::nil(),
            event_type: CanvasSyncEvent::EdgeChanged,
            actor_id: None,
            timestamp: Utc::now(),
            payload: serde_json::Value::Null,
            version: 1,
        }
    }
}

/// Canvas UI sync bridge 配置 (per DD-CANVAS-AGENT-001 v0.1 §3.1, 0 业务方法).
///
/// 阶段 1 基础 仅字段定义 + `Default`, 阶段 3 集成 任务 3.2 落地配置加载.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSyncBridgeConfig {
    /// WSS 推送 endpoint (canvas-collab 订阅, 阶段 1 任务 1.3 + 阶段 3 任务 3.2)
    pub wss_endpoint: String,
    /// canvas-collab BFF endpoint (HTTP 业务调用)
    pub canvas_collab_endpoint: String,
    /// 节流窗口 (毫秒, 默认 50ms = 20 events/s, per arch 2026-09-03-arg §6)
    pub throttle_ms: u64,
    /// 批量推送大小 (默认 100, 阶段 3 集成 任务 3.2 落地)
    pub batch_size: usize,
    /// canvas-collab 鉴权 token (env-bound, 守门 #5 env-safety, 不打印)
    pub auth_token: String,
}

impl Default for CanvasSyncBridgeConfig {
    fn default() -> Self {
        Self {
            wss_endpoint: "wss://canvas-collab/canvases/[id]".to_string(),
            canvas_collab_endpoint: "http://canvas-collab-bff/canvases".to_string(),
            throttle_ms: 50,
            batch_size: 100,
            auth_token: String::new(),
        }
    }
}

/// Canvas UI sync bridge 骨架 (per P3-D.5 DD-CANVAS-AGENT-001 v0.1 §3.1, 0 业务方法).
///
/// 阶段 1 基础 **仅字段定义 + `new` 构造**, 阶段 3 集成 任务 3.2 落地 WSS 推送 / SSE 广播 / 5 WSS 协议.
///
/// 字段类型:
/// - `memgraph_client`: `Arc<MemgraphClient>` 共享 ARG data tier 客户端 (跟 `period_flush.rs` 一致)
/// - `sync_protocol`: 协议分发表 (5 ARG effect 维度, per `protocol.rs` `BridgeEnvelopeKind`)
/// - `period_flush_worker`: `Arc<PeriodFlushWorker>` 共享 flush worker (不直接持有所有权)
/// - `event_queue`: 待推送事件 buffer (阶段 3 集成 任务 3.2 flush)
#[derive(Debug, Clone)]
pub struct CanvasSyncBridge {
    /// Bridge 配置
    pub config: CanvasSyncBridgeConfig,
    /// Memgraph 客户端 (可选, 阶段 3 集成 时初始化, `Arc` 共享 跟 `period_flush.rs` 一致)
    pub memgraph_client: Option<Arc<MemgraphClient>>,
    /// ARG 事件类型 → 推送策略 (per `BridgeEnvelopeKind` 5 维度)
    pub sync_protocol: Option<BridgeEnvelopeKind>,
    /// 周期 flush worker (可选, 阶段 3 集成 时初始化, `Arc` 共享 跟 `period_flush.rs` 一致)
    pub period_flush_worker: Option<Arc<PeriodFlushWorker>>,
    /// 待推送事件 buffer (阶段 3 集成 任务 3.2 flush)
    pub event_queue: Vec<CanvasSyncEventPayload>,
}

impl CanvasSyncBridge {
    /// Canvas UI sync bridge 构造函数 (0 业务逻辑, 仅字段预填, 阶段 3 集成 任务 3.2 落地).
    ///
    /// 阶段 1 基础: 3 个可选字段 = `None`, `event_queue` = 空, 仅 `config` 透传.
    pub fn new(config: CanvasSyncBridgeConfig) -> Self {
        Self {
            config,
            memgraph_client: None,
            sync_protocol: None,
            period_flush_worker: None,
            event_queue: Vec::new(),
        }
    }

    /// 引用 `ARGEvent` 验证 use 路径 (per 守门 #1 禁回溯叙事: 0 业务方法, 仅类型引用)
    #[allow(dead_code)]
    fn _event_type_ref() -> Option<ARGEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_sync_event_default() {
        let payload = CanvasSyncEventPayload::default();
        assert_eq!(payload.event_id, Uuid::nil());
        assert_eq!(payload.event_type, CanvasSyncEvent::EdgeChanged);
        assert_eq!(payload.version, 1);
        assert!(payload.actor_id.is_none());
    }

    #[test]
    fn test_canvas_sync_bridge_config_default() {
        let config = CanvasSyncBridgeConfig::default();
        assert_eq!(config.throttle_ms, 50);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.wss_endpoint, "wss://canvas-collab/canvases/[id]");
        assert!(config.auth_token.is_empty());
    }

    #[test]
    fn test_canvas_sync_bridge_new() {
        let config = CanvasSyncBridgeConfig::default();
        let bridge = CanvasSyncBridge::new(config.clone());
        assert_eq!(bridge.config.throttle_ms, 50);
        assert_eq!(bridge.config.batch_size, 100);
        assert_eq!(bridge.event_queue.len(), 0);
        assert!(bridge.memgraph_client.is_none());
        assert!(bridge.sync_protocol.is_none());
        assert!(bridge.period_flush_worker.is_none());
    }
}
