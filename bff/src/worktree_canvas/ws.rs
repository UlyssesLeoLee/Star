// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/ws.rs` — 1 WebSocket 端点 (per DD-WORKTREE-CANVAS-001 §21
//! + IMPL-PLAN §4.13.4 `/v1/worktree-canvas/realtime`).
//!
//! UI ↔ BFF realtime: 客户端订阅 / ping, 服务端 fanout 15 SSE event + 心跳 30s
//! (per IMPL-PLAN §4.13.4 心跳 30s).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - RBAC: View 即可连接 (per DD §44).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use super::dto::{WorktreeApiError, WorktreeSseEvent, WsClientMessage, WsServerMessage};
use super::WorktreeCanvasState;

/// WebSocket 心跳 interval (per IMPL-PLAN §4.13.4).
pub const WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Build the WebSocket routes router.
pub fn ws_routes(state: Arc<WorktreeCanvasState>) -> Router {
    Router::new()
        .route("/v1/worktree-canvas/realtime", get(ws_handler))
        .with_state(state)
}

/// WebSocket handler (per IMPL-PLAN §4.13.4 + DD §21).
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WorktreeCanvasState>>,
) -> Result<impl IntoResponse, WorktreeApiError> {
    // RBAC: View 即可 (per DD §44).
    state.permission.require(super::permission::PermissionLevel::View)?;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

/// Per-connection WebSocket handler (per spec §4.5 wire protocol).
async fn handle_socket(socket: WebSocket, state: Arc<WorktreeCanvasState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.sse_hub.subscribe();

    loop {
        tokio::select! {
            // 1. 客户端消息.
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsClientMessage>(&text) {
                            Ok(WsClientMessage::Subscribe { events }) => {
                                let ack = WsServerMessage::Subscribed { events };
                                let json = serde_json::to_string(&ack).unwrap_or_default();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(WsClientMessage::Unsubscribe { .. }) => {
                                // 阶段 1: 0 业务 filter, 0 实际 unsubscribe 跟踪.
                                // 阶段 2 业务实装时 per-connection filter.
                            }
                            Ok(WsClientMessage::Ping) => {
                                let pong = WsServerMessage::Pong;
                                let json = serde_json::to_string(&pong).unwrap_or_default();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let err = WsServerMessage::Error {
                                    code: "BAD_MESSAGE".into(),
                                    message: e.to_string(),
                                };
                                let json = serde_json::to_string(&err).unwrap_or_default();
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => { /* binary / pong ignored */ }
                    Some(Err(_)) => break,
                }
            }

            // 2. SSE event fanout (per spec §4.5 worktree events).
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        let msg = WsServerMessage::Event { payload: ev };
                        let json = serde_json::to_string(&msg).unwrap_or_default();
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }

            // 3. 心跳 (per IMPL-PLAN §4.13.4).
            _ = tokio::time::sleep(WS_HEARTBEAT_INTERVAL) => {
                let pong = WsServerMessage::Pong;
                let json = serde_json::to_string(&pong).unwrap_or_default();
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree_canvas::dto::WorktreeApiError;
    use crate::worktree_canvas::permission::RoleSet;
    use crate::worktree_canvas::permission::WorktreePermission;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn ws_heartbeat_interval_is_30_seconds() {
        // IMPL-PLAN §4.13.4 心跳 30s.
        assert_eq!(WS_HEARTBEAT_INTERVAL, Duration::from_secs(30));
    }

    #[test]
    fn ws_routes_creates_router() {
        // 烟雾测试: ws_routes 接受 state, 返回 Router (不 panic).
        let state = Arc::new(
            WorktreeCanvasState::new(
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
                RoleSet::new(),
                super::super::permission::PermissionLevel::View,
            )
            .unwrap_or_else(|e: WorktreeApiError| panic!("state new failed: {e:?}")),
        );
        let _ = ws_routes(state);
    }
}
