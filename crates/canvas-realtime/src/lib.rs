//! canvas-realtime lib: Yjs CRDT 后端元数据 + axum router builder (per ULYS-160 §3 阶段 1 骨架)
//!
//! 阶段 1: 健康检查 + 服务版本 + HTTP :8082 + WSS :8083 echo endpoint. 0 Yjs 协议.
//! 阶段 2+: 落地 Yjs/yrs CRDT 集成 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.3).

#![warn(missing_docs)]

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;

/// 服务元数据.
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// 服务名.
    pub name: &'static str,
    /// 服务语义版本.
    pub version: &'static str,
    /// HTTP 监听端口 (per deploy/canvas-game-k3s.yaml containerPort: 8082).
    pub http_port: u16,
    /// WebSocket 监听端口 (per deploy/canvas-game-k3s.yaml containerPort: 8083).
    pub ws_port: u16,
    /// 阶段 1 骨架 marker.
    pub phase: Phase,
}

/// 实现阶段 marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// 阶段 1: 骨架, HTTP + WSS echo.
    Skeleton,
    /// 阶段 2+: Yjs/yrs CRDT 已落地.
    Implemented,
}

impl ServiceMetadata {
    /// canvas-realtime 服务元数据.
    pub const CANVAS_REALTIME: Self = Self {
        name: "canvas-realtime",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8082,
        ws_port: 8083,
        phase: Phase::Skeleton,
    };
}

/// canvas-realtime 错误类型.
#[derive(Debug, thiserror::Error)]
pub enum CanvasRealtimeError {
    /// 阶段 1 基础 占位.
    #[error("canvas-realtime skeleton: not implemented (阶段 2 落地)")]
    NotImplemented,
}

/// 构建 HTTP axum Router (阶段 1: 健康检查 + 元数据 endpoint + WSS echo).
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/ws", get(ws_handler))
}

/// HTTP 健康检查 endpoint.
async fn healthz() -> &'static str {
    "ok"
}

/// HTTP readiness endpoint.
async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_REALTIME.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

/// HTTP version endpoint.
async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_REALTIME.name,
        "version": ServiceMetadata::CANVAS_REALTIME.version,
        "phase": ServiceMetadata::CANVAS_REALTIME.phase,
    }))
}

/// WebSocket 升级 handler (阶段 1: echo; 阶段 2+: Yjs/yrs CRDT 集成).
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// WebSocket 连接处理 (阶段 1: echo; 阶段 2+: Yjs CRDT sync).
async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(t)) => {
                if socket.send(Message::Text(t)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Binary(b)) => {
                if socket.send(Message::Binary(b)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}