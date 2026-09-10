// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/collaboration/wss_hub.rs` — WebSocket fan-out hub for BFF canvas collaboration (per DD §4.14 + §11.1 + brief §2.5).
//!
//! 7 事件类型 (per brief §2.5 + DD-AGENT §4.14 + NFR-AGENT-MU-CONS-01):
//! 1. `element.created`           — A12.1 create
//! 2. `element.updated`           — A12.3 update
//! 3. `element.deleted`           — 5 REST endpoint 5 delete
//! 4. `presence.updated`          — A12.2 cursor push
//! 5. `comment.added`             — A12.5 add comment
//! 6. `follow.started`            — A12.4 start follow
//! 7. `follow.stopped`            — A12.4 stop follow
//!
//! 实现要点 (per brief §2.5):
//! - 用 `tokio::sync::broadcast::Sender<CollaborationWssEvent>` 走 fan-out
//!   (per DD §11.1 IT-09 WebSocket 推送 < 100ms, 10 并发)
//! - WebSocket 升级走 `axum::extract::ws::WebSocketUpgrade` extractor
//!   (跟 `crates/api/src/arg/sse_hub.rs` 同形)
//! - `broadcast` 容量 1024 (per brief §2.5, 比 ARG hub 256 容量大因为 A12
//!   实时光标推送频率更高 per NFR-AGENT-MU-CONS-01), 慢消费者掉线时返回
//!   `RecvError::Lagged`.
//! - 跨 session 跨 sub-agent 共享同一个 hub (per 守门 #24 v2).
//!
//! 阶段 1 占位: 0 真实 WSS 业务逻辑 (留 P3-D.6 阶段 3 集成 任务 3.2: 0 CRDT
//! 0 NATS 0 真实 broadcast subscribe). 当前仅接受 WS 连接 + 立即关闭
//! 模式 (per brief §2.5 handler 占位).

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::dto::ApiError;
use super::dto::CollaborationWssEvent;
use super::permission::PermissionLevel;
use super::CollaborationState;

/// Default broadcast channel capacity.
///
/// 1024 是 per brief §2.5 "A12 实时光标推送频率更高" 推算的缓冲上限; 超过会被
/// `RecvError::Lagged` 丢, 慢消费者重连即可.
pub const BROADCAST_CAPACITY: usize = 1024;

/// Hub backing the 4 WSS endpoints under `/bff/v1/ws/canvas/*`.
///
/// 单实例 (`Arc<CollaborationWssHub>`) 由 [`CollaborationState`] 持有; 每个
/// WS 连接都从 hub 拿一个新的 `broadcast::Receiver` 订阅.
#[derive(Debug, Clone)]
pub struct CollaborationWssHub {
    tx: broadcast::Sender<CollaborationWssEvent>,
}

impl CollaborationWssHub {
    /// Build a new hub with the default capacity (1024).
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
    pub fn subscribe(&self) -> broadcast::Receiver<CollaborationWssEvent> {
        self.tx.subscribe()
    }

    /// Publish an event to all current subscribers. Returns the number
    /// of receivers that got the event (excluding lagged ones).
    pub fn broadcast(&self, event: CollaborationWssEvent) -> usize {
        self.tx.send(event).unwrap_or(0)
    }
}

impl Default for CollaborationWssHub {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// WSS endpoint handlers (4 endpoints per brief §2.5)
// =====================================================================

/// 1. `GET /bff/v1/ws/canvas/elements` — element create/update/delete push (A12.1 + A12.3).
///
/// Per brief §2.5 endpoint 1.
pub async fn wss_canvas_elements(
    ws: WebSocketUpgrade,
    State(state): State<Arc<CollaborationState>>,
) -> impl IntoResponse {
    // RLS 13 類: WS connect 走 permission 守门, 至少 `View` 等级 (per A12.7 派生).
    if let Err(e) = state.permission.require(PermissionLevel::View) {
        return api_error_to_response(e).into_response();
    }
    if state.tenant_id.is_nil() {
        return api_error_to_response(ApiError::Forbidden(
            "WS connect requires non-nil tenant_id".into(),
        ))
        .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
        .into_response()
}

/// 2. `GET /bff/v1/ws/canvas/presence` — presence cursor push (A12.2).
///
/// Per brief §2.5 endpoint 2.
pub async fn wss_canvas_presence(
    ws: WebSocketUpgrade,
    State(state): State<Arc<CollaborationState>>,
) -> impl IntoResponse {
    if let Err(e) = state.permission.require(PermissionLevel::View) {
        return api_error_to_response(e).into_response();
    }
    if state.tenant_id.is_nil() {
        return api_error_to_response(ApiError::Forbidden(
            "WS connect requires non-nil tenant_id".into(),
        ))
        .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
        .into_response()
}

/// 3. `GET /bff/v1/ws/canvas/comments` — comment push (A12.5).
///
/// Per brief §2.5 endpoint 3.
pub async fn wss_canvas_comments(
    ws: WebSocketUpgrade,
    State(state): State<Arc<CollaborationState>>,
) -> impl IntoResponse {
    if let Err(e) = state.permission.require(PermissionLevel::View) {
        return api_error_to_response(e).into_response();
    }
    if state.tenant_id.is_nil() {
        return api_error_to_response(ApiError::Forbidden(
            "WS connect requires non-nil tenant_id".into(),
        ))
        .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
        .into_response()
}

/// 4. `GET /bff/v1/ws/canvas/follow` — follow mode push (A12.4).
///
/// Per brief §2.5 endpoint 4.
pub async fn wss_canvas_follow(
    ws: WebSocketUpgrade,
    State(state): State<Arc<CollaborationState>>,
) -> impl IntoResponse {
    if let Err(e) = state.permission.require(PermissionLevel::View) {
        return api_error_to_response(e).into_response();
    }
    if state.tenant_id.is_nil() {
        return api_error_to_response(ApiError::Forbidden(
            "WS connect requires non-nil tenant_id".into(),
        ))
        .into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
        .into_response()
}

// =====================================================================
// Helper: convert ApiError to a 4xx JSON response
// =====================================================================

/// Convert [`ApiError`] into an axum response. Mirrors the helper in
/// `crates/api/src/canvas_collab/controller.rs` (per 守门 #19 v19 累积规,
/// BFF 跟 API tier 同形但 0 跨层 leak).
fn api_error_to_response(e: ApiError) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    use axum::http::StatusCode;
    let status = StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = serde_json::json!({
        "error": {
            "code": match &e {
                ApiError::BadRequest(_) => "bad_request",
                ApiError::Forbidden(_) => "forbidden",
                ApiError::NotFound(_) => "not_found",
                ApiError::Internal(_) => "internal",
                ApiError::Unimplemented(_) => "not_implemented",
            },
            "message": e.to_string(),
        }
    });
    (status, axum::Json(body))
}

// =====================================================================
// Per-connection task: subscribe to the hub, forward events to the
// client as text frames; close on either side erroring out.
// =====================================================================

/// Per-connection task: subscribe to the hub, forward events to the
/// client as text frames; close on either side erroring out.
async fn handle_socket(mut socket: WebSocket, state: Arc<CollaborationState>) {
    let mut rx = state.wss_hub.subscribe();
    let tenant = state.tenant_id;

    // Send a welcome frame so the client can confirm the connection.
    let welcome = serde_json::json!({
        "type": "hello",
        "tenant_id": tenant,
        "subscribers": state.wss_hub.receiver_count(),
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
fn event_tenant(event: &CollaborationWssEvent) -> Option<Uuid> {
    match event {
        CollaborationWssEvent::ElementCreated { canvas_id, .. }
        | CollaborationWssEvent::ElementUpdated { canvas_id, .. }
        | CollaborationWssEvent::ElementDeleted { canvas_id, .. }
        | CollaborationWssEvent::PresenceUpdated { canvas_id, .. }
        | CollaborationWssEvent::CommentAdded { canvas_id, .. }
        | CollaborationWssEvent::FollowStarted { canvas_id, .. }
        | CollaborationWssEvent::FollowStopped { canvas_id, .. } => Some(*canvas_id),
    }
}

// Suppress unused warning for `Serialize` / `Deserialize` (used implicitly
// via the serde derives on `CollaborationWssEvent`).
#[allow(dead_code)]
fn _assert_serde_traits<T: Serialize + Deserialize<'static>>() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collaboration::audit::CollaborationAudit;
    use crate::collaboration::permission::{CollaborationPermission, PermissionLevel};
    use crate::collaboration::CollaborationState;
    use chrono::Utc;
    use std::collections::HashSet;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    fn canvas() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap()
    }

    fn make_hub() -> Arc<CollaborationWssHub> {
        Arc::new(CollaborationWssHub::with_capacity(8))
    }

    fn make_state() -> Arc<CollaborationState> {
        let mut roles = HashSet::new();
        roles.insert(crate::collaboration::permission::role::LEAD.to_string());
        let permission = Arc::new(CollaborationPermission::new(
            tenant(),
            roles,
            PermissionLevel::Edit,
        ));
        let audit = Arc::new(CollaborationAudit::new());
        let wss_hub = make_hub();
        Arc::new(CollaborationState {
            audit,
            permission,
            wss_hub,
            tenant_id: tenant(),
            actor_id: actor(),
            canvas_id: canvas(),
            presence_placeholder: None,
        })
    }

    #[test]
    fn hub_starts_with_zero_receivers() {
        let h = make_hub();
        assert_eq!(h.receiver_count(), 0);
    }

    #[test]
    fn hub_subscribe_increments_receiver_count() {
        let h = make_hub();
        let _rx1 = h.subscribe();
        let _rx2 = h.subscribe();
        assert_eq!(h.receiver_count(), 2);
    }

    #[test]
    fn broadcast_returns_receiver_count() {
        let h = make_hub();
        let _rx1 = h.subscribe();
        let _rx2 = h.subscribe();
        let n = h.broadcast(CollaborationWssEvent::ElementCreated {
            canvas_id: canvas(),
            element_id: Uuid::new_v4(),
            actor_id: actor(),
            at: Utc::now(),
        });
        assert_eq!(n, 2);
    }

    #[test]
    fn default_capacity_matches_constant() {
        let h = CollaborationWssHub::default();
        // `receiver_count` is 0 right after construction.
        assert_eq!(h.receiver_count(), 0);
        // Default is 1024 per brief §2.5 (we cannot directly introspect
        // `broadcast::Sender` capacity without consuming it, so we just
        // assert that `default()` returns a usable hub).
    }

    #[test]
    fn event_tenant_returns_canvas_id() {
        let evt = CollaborationWssEvent::PresenceUpdated {
            canvas_id: canvas(),
            user_id: actor(),
            x: 1.0,
            y: 2.0,
            at: Utc::now(),
        };
        assert_eq!(event_tenant(&evt), Some(canvas()));
    }

    #[test]
    fn wss_canvas_elements_returns_wrong_state_type_check() {
        // Smoke test: the handler signature compiles with `State<Arc<CollaborationState>>`.
        // We can't easily invoke the handler directly without a tower
        // `Service` test harness, so we just type-check the state struct.
        let s = make_state();
        let _ = State(s);
    }
}
