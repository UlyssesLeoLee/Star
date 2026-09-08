// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/arg/sse_hub.rs` — WebSocket / SSE fan-out hub (per DD §4.12 + §5).
//!
//! 6 事件类型 (per brief §2.1 A + DD §11.1 WebSocket 推送 NFR):
//! 1. `edge.changed`               — 边被 update (PATCH)
//! 2. `edge.created`               — 边被 create (POST)
//! 3. `edge.archived`              — 边被 archive (DELETE)
//! 4. `achievement.unlocked`       — 成就解锁
//! 5. `dispatch.route.changed`     — 5 域 Lead 决策
//! 6. `agent.trust_score.changed`  — agent 信任度变化
//!
//! 实现要点:
//! - 用 `tokio::sync::broadcast::Sender<ARGSseEvent>` 走 fan-out
//!   (per DD §11.1 IT-09 WebSocket 推送 < 100ms, 10 并发)
//! - WebSocket 升级走 `axum::extract::ws::WebSocketUpgrade` extractor
//!   (per brief AC-2, 不是普通 `get()` handler)
//! - `broadcast` 容量 256, 慢消费者掉线时返回 `RecvError::Lagged`
//! - 跨 session 跨 sub-agent 共享同一个 hub (per 守门 #24 v2)

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

#[allow(unused_imports)]
use super::dto::ApiError;

/// `ARGSseEvent` — 6 事件类型 (per brief §2.1 A + DD §11.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ARGSseEvent {
    /// 边被 update (PATCH /api/arg/edges/{id}).
    EdgeChanged {
        /// The updated edge.
        edge: star_arg::models::edge::Edge,
    },
    /// 边被 create (POST /api/arg/edges).
    EdgeCreated {
        /// The newly created edge.
        edge: star_arg::models::edge::Edge,
    },
    /// 边被 archive (DELETE /api/arg/edges/{id}).
    EdgeArchived {
        /// The archived edge id.
        id: Uuid,
        /// Tenant id (for the client to filter).
        tenant_id: Uuid,
    },
    /// 成就解锁 (POST /api/arg/achievements/evaluate).
    AchievementUnlocked {
        /// Achievement code (e.g. `TOP-001-MESH-5DOMAIN`).
        code: String,
        /// User id that unlocked the achievement.
        user_id: Uuid,
        /// Tenant id.
        tenant_id: Uuid,
    },
    /// 5 域 Lead dispatch 决策 (per 守门 #3 反转 + 守门 #14 v2).
    DispatchRouteChanged {
        /// Current agent id.
        current_agent_id: Uuid,
        /// New delegate list.
        delegates: Vec<Uuid>,
        /// Tenant id.
        tenant_id: Uuid,
    },
    /// Agent 信任度变化 (per DD §3.3.3 + 守门 #11).
    AgentTrustScoreChanged {
        /// The affected agent.
        agent_id: Uuid,
        /// Score before the change.
        before: f32,
        /// Score after the change.
        after: f32,
        /// Tenant id.
        tenant_id: Uuid,
    },
}

impl ARGSseEvent {
    /// Stable string code used in the SSE `event:` field.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::EdgeChanged { .. } => "edge.changed",
            Self::EdgeCreated { .. } => "edge.created",
            Self::EdgeArchived { .. } => "edge.archived",
            Self::AchievementUnlocked { .. } => "achievement.unlocked",
            Self::DispatchRouteChanged { .. } => "dispatch.route.changed",
            Self::AgentTrustScoreChanged { .. } => "agent.trust_score.changed",
        }
    }
}

/// Default broadcast channel capacity.
///
/// 256 是 per DD §11.1 "WebSocket 推送 P95 < 100ms, 10 并发" 推算的
/// 缓冲上限; 超过会被 `RecvError::Lagged` 丢, 慢消费者重连即可.
pub const BROADCAST_CAPACITY: usize = 256;

/// Hub backing the `/ws/arg/events` endpoint.
///
/// 单实例 (`Arc<ARGSSEHub>`) 由 [`crate::star_arg::mod::ARGState`] 持有; 每个
/// WS 连接都从 hub 拿一个新的 `broadcast::Receiver` 订阅.
#[derive(Debug, Clone)]
pub struct ARGSSEHub {
    tx: broadcast::Sender<ARGSseEvent>,
}

impl ARGSSEHub {
    /// Build a new hub with the default capacity (256).
    pub fn new() -> Self {
        Self::with_capacity(BROADCAST_CAPACITY)
    }

    /// Build a new hub with a custom capacity (used by tests).
    pub fn with_capacity(cap: usize) -> Self {
        let (tx, _rx) = broadcast::channel(cap);
        Self { tx }
    }

    /// Number of currently subscribed receivers.
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Subscribe a new consumer. Returns a `broadcast::Receiver` that
    /// receives every event sent to the hub after the call.
    pub fn subscribe(&self) -> broadcast::Receiver<ARGSseEvent> {
        self.tx.subscribe()
    }

    /// Publish an event to all current subscribers. Returns the number
    /// of receivers that got the event (excluding lagged ones).
    pub fn publish(&self, event: ARGSseEvent) -> usize {
        self.tx.send(event).unwrap_or(0)
    }
}

impl Default for ARGSSEHub {
    fn default() -> Self {
        Self::new()
    }
}

/// 14. WS handler — `GET /ws/arg/events`.
///
/// Per brief AC-2 we MUST use `axum::extract::ws::WebSocketUpgrade`
/// (not a plain `get()` handler). The state parameter carries the
/// `Arc<ARGSSEHub>` (and full `ARGState` for `tenant_id` filtering).
///
/// The handler is **infallible** (returns `impl IntoResponse`) so it
/// matches axum's `Handler` trait. Permission failures are returned
/// as an HTTP 403 JSON response (per DD §4.12 RLS 13 類).
pub async fn sse_hub(
    ws: WebSocketUpgrade,
    State(state): State<Arc<super::ARGState>>,
) -> impl IntoResponse {
    // RLS 13 類: WS connect 仍走 permission 守门, 没有 actor 上下文就
    // 拒绝 (per 守门 #14 v2 拍板 D, Mavis 临时代签阶段 tenant 必须存在).
    if state.tenant_id.is_nil() {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({
                "error": {
                    "code": "forbidden",
                    "message": "WS connect requires non-nil tenant_id",
                }
            })),
        )
            .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
        .into_response()
}

/// Per-connection task: subscribe to the hub, forward events to the
/// client as text frames; close on either side erroring out.
async fn handle_socket(mut socket: WebSocket, state: Arc<super::ARGState>) {
    let mut rx = state.sse_hub.subscribe();
    let tenant = state.tenant_id;

    // Send a welcome frame so the client can confirm the connection.
    let welcome = serde_json::json!({
        "type": "hello",
        "tenant_id": tenant,
        "subscribers": state.sse_hub.receiver_count(),
    });
    if socket
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            // Forward broadcast events to the client.
            event = rx.recv() => {
                match event {
                    Ok(evt) => {
                        // Per-tenant filter: skip events from other tenants.
                        if let Some(evt_tenant) = event_tenant(&evt) {
                            if evt_tenant != tenant {
                                continue;
                            }
                        }
                        let payload = serde_json::to_string(&evt)
                            .unwrap_or_else(|_| "{}".to_string());
                        if socket
                            .send(Message::Text(payload.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Slow consumer: send a sentinel and continue.
                        let lag = serde_json::json!({
                            "type": "lagged",
                            "message": "subscriber lagged, some events dropped"
                        });
                        if socket
                            .send(Message::Text(lag.to_string().into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            // Drain client frames so the socket stays alive (axum
            // requires the server to call `recv` to process pings).
            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        if socket.send(Message::Pong(p)).await.is_err() {
                            break;
                        }
                    }
                    _ => { /* ignore other client frames */ }
                }
            }
        }
    }
}

/// Extract the tenant id from an event, if the variant carries one.
fn event_tenant(event: &ARGSseEvent) -> Option<Uuid> {
    match event {
        ARGSseEvent::EdgeChanged { edge } => Some(edge.tenant_id),
        ARGSseEvent::EdgeCreated { edge } => Some(edge.tenant_id),
        ARGSseEvent::EdgeArchived { tenant_id, .. } => Some(*tenant_id),
        ARGSseEvent::AchievementUnlocked { tenant_id, .. } => Some(*tenant_id),
        ARGSseEvent::DispatchRouteChanged { tenant_id, .. } => Some(*tenant_id),
        ARGSseEvent::AgentTrustScoreChanged { tenant_id, .. } => Some(*tenant_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use star_arg::ops::event_writer::EventWriter;

    fn empty_state() -> Arc<super::super::ARGState> {
        let client = std::sync::Arc::new(star_arg::client::MemgraphClient::new(
            "bolt://localhost:7687".into(),
            "user".into(),
            "pw".into(),
            1,
        ));
        let event_writer = std::sync::Arc::new(EventWriter::new());
        let agent_ops = std::sync::Arc::new(star_arg::ops::AgentNodeOps::new(
            client.clone(),
            event_writer.clone(),
        ));
        let edge_ops = std::sync::Arc::new(star_arg::ops::EdgeOps::new(
            client.clone(),
            event_writer.clone(),
        ));
        let template_ops = std::sync::Arc::new(star_arg::ops::TemplateOps::new(
            client,
            edge_ops.clone(),
            event_writer.clone(),
        ));
        let achievement_writer = EventWriter::new();
        let achievement_ops =
            std::sync::Arc::new(star_arg::ops::AchievementOps::new(achievement_writer));
        let hub = ARGSSEHub::new();
        let permission = std::sync::Arc::new(super::super::permission::ARGPermission::new(
            Uuid::nil(),
            std::collections::HashSet::new(),
        ));
        super::super::ARGState::new(
            agent_ops,
            edge_ops,
            template_ops,
            achievement_ops,
            event_writer,
            std::sync::Arc::new(hub),
            permission,
            Uuid::nil(),
        )
    }

    #[test]
    fn hub_subscribe_and_publish() {
        let hub = ARGSSEHub::new();
        let mut rx = hub.subscribe();
        let n = hub.publish(ARGSseEvent::EdgeArchived {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
        });
        assert_eq!(n, 1);
        let evt = rx.try_recv().expect("event delivered");
        assert!(matches!(evt, ARGSseEvent::EdgeArchived { .. }));
    }

    #[test]
    fn hub_receiver_count_grows_with_subscribers() {
        let hub = ARGSSEHub::new();
        assert_eq!(hub.receiver_count(), 0);
        let _a = hub.subscribe();
        assert_eq!(hub.receiver_count(), 1);
        let _b = hub.subscribe();
        assert_eq!(hub.receiver_count(), 2);
    }

    #[test]
    fn event_kind_is_stable() {
        let evt = ARGSseEvent::DispatchRouteChanged {
            current_agent_id: Uuid::new_v4(),
            delegates: vec![],
            tenant_id: Uuid::nil(),
        };
        assert_eq!(evt.kind(), "dispatch.route.changed");
    }

    // Suppress the dead_code warning on `empty_state`; it is wired up by
    // the WS handler at runtime and a future test that drives a real
    // axum oneshot server.
    #[allow(dead_code)]
    fn _state_constructor_compiles() -> Arc<super::super::ARGState> {
        empty_state()
    }
}
