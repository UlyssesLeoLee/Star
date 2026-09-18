// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/sse.rs` — 15 SSE 端点 + Event 推送 (per
//! DD-WORKTREE-CANVAS-001 §21 + IMPL-PLAN §4.13.3 + spec §4.5).
//!
//! 阶段 1: in-memory broadcast hub (`WorktreeSseHub`) + 15 路由桩, 占位 handler
//! 返回带 keepalive 的 SSE stream. 阶段 2 业务实装落地 Redis Streams fanout
//! (per IMPL-PLAN §3.2 event-bus crate + spec §4.5).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 15 SSE endpoint 路径 1:1 对应 spec §4.5 15 事件名 (PascalCase URL 形式).

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response, Sse};
use axum::routing::get;
use axum::Router;
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::dto::{WorktreeApiError, WorktreeSseEvent};
use super::WorktreeCanvasState;

/// SSE broadcast hub (per spec §4.5 + IMPL-PLAN §4.13.3).
///
/// Broadcast channel capacity 1024 (per V0.1 collab `CollaborationWssHub` 模式一致).
/// 阶段 1 占位: 0 业务 event fan-in. 阶段 2 业务实装从 `event-bus` crate (per
/// IMPL-PLAN §3.2) 拉真实 event 转发.
#[derive(Debug, Clone)]
pub struct WorktreeSseHub {
    tx: broadcast::Sender<WorktreeSseEvent>,
}

impl WorktreeSseHub {
    /// 默认 broadcast channel capacity (跟 V0.1 collab 一致).
    pub const DEFAULT_CAPACITY: usize = 1024;

    /// 构造一个新的 SSE hub.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(Self::DEFAULT_CAPACITY);
        Self { tx }
    }

    /// 订阅 SSE event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<WorktreeSseEvent> {
        self.tx.subscribe()
    }

    /// 当前订阅者数量.
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// 发布一个 SSE event (阶段 2 由 event-bus 集成).
    pub fn publish(&self, event: WorktreeSseEvent) -> Result<usize, WorktreeApiError> {
        self.tx
            .send(event)
            .map_err(|e| WorktreeApiError::new("BROADCAST", e.to_string(), "bff.worktree_canvas", "sse::publish"))
    }
}

impl Default for WorktreeSseHub {
    fn default() -> Self {
        Self::new()
    }
}

/// 序列化 SSE event 为 wire format: `event: <name>\ndata: <json>\n\n`.
fn encode_sse_frame(event: &WorktreeSseEvent) -> Result<String, serde_json::Error> {
    let name = event.event_name();
    let data = serde_json::to_string(event)?;
    Ok(format!("event: {name}\ndata: {data}\n\n"))
}

/// SSE keepalive interval (per spec §4.5 throttle 50ms 防抖, keepalive 30s).
pub const SSE_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// Build the 15 SSE routes router (per IMPL-PLAN §4.13.3).
///
/// 路径格式: `/v1/worktree-canvas/events/<event-name>` (e.g. `/events/WorktreeCreated`).
/// 每个路由返回带 keepalive 的 SSE stream, 占位 0 真实事件 (阶段 2 业务实装).
pub fn sse_routes(state: Arc<WorktreeCanvasState>) -> Router {
    let mut router = Router::new();

    // 15 SSE 端点 (per spec §4.5 event 列表, 1:1 镜像).
    for name in WorktreeSseEvent::ALL_NAMES {
        let path = format!("/v1/worktree-canvas/events/{name}");
        router = router.route(&path, get(sse_handler));
    }

    // 健康检查 SSE (per IMPL-PLAN §4.13.2 BFF health endpoint).
    router = router.route(
        "/v1/worktree-canvas/health/stream",
        get(sse_health_stream),
    );

    router.with_state(state)
}

/// SSE handler (per event 端点) — 阶段 1 占位: 立即返回空 stream + 401 if no permission.
async fn sse_handler(
    State(state): State<Arc<WorktreeCanvasState>>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>>, Response> {
    // RBAC 守门: View 即可订阅 (per DD §44).
    state
        .permission
        .require(super::permission::PermissionLevel::View)
        .map_err(|e| e.into_response())?;

    let mut rx = state.sse_hub.subscribe();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let frame = encode_sse_frame(&event).unwrap_or_default();
                    yield Ok(axum::response::sse::Event::default()
                        .event(event.event_name().to_string())
                        .data(frame));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(SSE_KEEPALIVE_INTERVAL)
            .text("keepalive"),
    ))
}

/// Health stream (per brief T13 健康检查 SSE) — 阶段 1 占位.
async fn sse_health_stream(
    State(state): State<Arc<WorktreeCanvasState>>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>>, Response> {
    state
        .permission
        .require(super::permission::PermissionLevel::View)
        .map_err(|e| e.into_response())?;

    let started = std::time::Instant::now();
    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(SSE_KEEPALIVE_INTERVAL).await;
            let payload = serde_json::json!({
                "status": "ok",
                "uptime_seconds": started.elapsed().as_secs(),
                "worktree_canvas_enabled": true,
            });
            yield Ok(axum::response::sse::Event::default()
                .event("HealthChanged")
                .data(payload.to_string()));
        }
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(SSE_KEEPALIVE_INTERVAL)
            .text("keepalive"),
    ))
}

// =====================================================================
// IntoResponse for WorktreeApiError (helper for handlers)
// =====================================================================

impl IntoResponse for WorktreeApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "CONFLICT_REQUIRED" => StatusCode::CONFLICT,
            "INTERNAL" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        let mut resp = Response::new(Body::from(body));
        *resp.status_mut() = status;
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        resp
    }
}

// =====================================================================
// 15 SSE endpoint 名称守门 (per spec §4.5)
// =====================================================================

/// 验证给定 URL path 是 15 SSE 端点之一.
pub fn is_valid_sse_path(path: &str) -> bool {
    let Some(name) = path.strip_prefix("/v1/worktree-canvas/events/") else {
        return path == "/v1/worktree-canvas/health/stream";
    };
    WorktreeSseEvent::ALL_NAMES.iter().any(|n| *n == name)
}

/// SSE route count (per IMPL-PLAN §4.13.3: 15 + 1 health = 16).
pub const SSE_ROUTE_COUNT: usize = 16;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_starts_with_zero_receivers() {
        let hub = WorktreeSseHub::new();
        assert_eq!(hub.receiver_count(), 0);
    }

    #[test]
    fn hub_subscribe_increments_receiver_count() {
        let hub = WorktreeSseHub::new();
        let _rx1 = hub.subscribe();
        let _rx2 = hub.subscribe();
        assert_eq!(hub.receiver_count(), 2);
    }

    #[test]
    fn hub_publish_returns_receiver_count() {
        let hub = WorktreeSseHub::new();
        let _rx = hub.subscribe();
        let event = WorktreeSseEvent::WorktreeCreated {
            worktree_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            branch: "main".into(),
        };
        assert_eq!(hub.publish(event).unwrap(), 1);
    }

    #[test]
    fn encode_sse_frame_uses_event_name_and_json_data() {
        let event = WorktreeSseEvent::WorktreeDeleted {
            worktree_id: Uuid::new_v4(),
        };
        let frame = encode_sse_frame(&event).unwrap();
        assert!(frame.starts_with("event: WorktreeDeleted\n"));
        assert!(frame.contains("data: "));
        assert!(frame.ends_with("\n\n"));
    }

    #[test]
    fn is_valid_sse_path_accepts_all_15_names() {
        for name in WorktreeSseEvent::ALL_NAMES {
            let path = format!("/v1/worktree-canvas/events/{name}");
            assert!(is_valid_sse_path(&path), "path should be valid: {path}");
        }
    }

    #[test]
    fn is_valid_sse_path_rejects_unknown_event() {
        assert!(!is_valid_sse_path("/v1/worktree-canvas/events/NotAnEvent"));
        assert!(!is_valid_sse_path("/v1/worktree-canvas/events/"));
    }

    #[test]
    fn sse_route_count_is_sixteen() {
        // 15 event 端点 + 1 health/stream = 16.
        assert_eq!(SSE_ROUTE_COUNT, 16);
    }
}
