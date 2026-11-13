/*
@cypher schema=1 source_sha256=af64630bd48121214b5c358663af485129b21f905c0582bc2ea588c3615a5663
MERGE (self:File {path:"crates/canvas-realtime/src/lib.rs"})
MERGE (metadata:Type {id:"crates/canvas-realtime/src/lib.rs::ServiceMetadata"})
MERGE (phase:Type {id:"crates/canvas-realtime/src/lib.rs::Phase"})
MERGE (error:Type {id:"crates/canvas-realtime/src/lib.rs::CanvasRealtimeError"})
MERGE (router:Symbol {id:"crates/canvas-realtime/src/lib.rs::router",kind:"function"})
MERGE (healthz:Symbol {id:"crates/canvas-realtime/src/lib.rs::healthz",kind:"function"})
MERGE (ready:Symbol {id:"crates/canvas-realtime/src/lib.rs::ready",kind:"function"})
MERGE (version:Symbol {id:"crates/canvas-realtime/src/lib.rs::version",kind:"function"})
MERGE (ws_handler:Symbol {id:"crates/canvas-realtime/src/lib.rs::ws_handler",kind:"function"})
MERGE (handle_socket:Symbol {id:"crates/canvas-realtime/src/lib.rs::handle_socket",kind:"function"})
MERGE (self)-[:DEFINES]->(metadata)
MERGE (self)-[:DEFINES]->(phase)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(healthz)
MERGE (self)-[:DEFINES]->(ready)
MERGE (self)-[:DEFINES]->(version)
MERGE (self)-[:DEFINES]->(ws_handler)
MERGE (self)-[:DEFINES]->(handle_socket)
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/healthz",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/ready",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/version",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/ws",method:"GET"})
MERGE (router)-[:CALLS]->(healthz)
MERGE (router)-[:CALLS]->(ready)
MERGE (router)-[:CALLS]->(version)
MERGE (router)-[:CALLS]->(ws_handler)
MERGE (ws_handler)-[:CALLS]->(handle_socket)
MERGE (ready)-[:USES_TYPE]->(metadata)
MERGE (version)-[:USES_TYPE]->(metadata)
@endcypher
*/
//! Star Canvas realtime service metadata and Axum router.
//!
//! Provides health, version, and WebSocket collaboration endpoints.

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
    /// HTTP listener port (per deploy/star-canvas-services-k3s.yaml containerPort: 8082).
    pub http_port: u16,
    /// WebSocket listener port (per deploy/star-canvas-services-k3s.yaml containerPort: 8083).
    pub ws_port: u16,
    /// Current service implementation phase.
    pub phase: Phase,
}

/// 实现阶段 marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// Service scaffold with HTTP metadata and WebSocket collaboration endpoints.
    Skeleton,
    /// Product collaboration protocol has been implemented.
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
    /// Product collaboration operation is not implemented.
    #[error("canvas-realtime product collaboration operation is not implemented")]
    NotImplemented,
}

/// Build the HTTP and WebSocket routes for this service.
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

/// Upgrade an HTTP request to a WebSocket connection.
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle messages on an established WebSocket connection.
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
