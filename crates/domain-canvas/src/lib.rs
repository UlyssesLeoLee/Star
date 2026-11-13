/*
@cypher schema=1 source_sha256=2bf5cc788f4f280660769dea8ecefa172bdeab5eb0169ef511ffec0c218f0c58
MERGE (self:File {path:"crates/domain-canvas/src/lib.rs"})
MERGE (metadata:Type {id:"crates/domain-canvas/src/lib.rs::ServiceMetadata"})
MERGE (phase:Type {id:"crates/domain-canvas/src/lib.rs::Phase"})
MERGE (error:Type {id:"crates/domain-canvas/src/lib.rs::DomainCanvasError"})
MERGE (router:Symbol {id:"crates/domain-canvas/src/lib.rs::router",kind:"function"})
MERGE (healthz:Symbol {id:"crates/domain-canvas/src/lib.rs::healthz",kind:"function"})
MERGE (ready:Symbol {id:"crates/domain-canvas/src/lib.rs::ready",kind:"function"})
MERGE (version:Symbol {id:"crates/domain-canvas/src/lib.rs::version",kind:"function"})
MERGE (canvas:Symbol {id:"crates/domain-canvas/src/lib.rs::canvas_placeholder",kind:"function"})
MERGE (self)-[:DEFINES]->(metadata)
MERGE (self)-[:DEFINES]->(phase)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(healthz)
MERGE (self)-[:DEFINES]->(ready)
MERGE (self)-[:DEFINES]->(version)
MERGE (self)-[:DEFINES]->(canvas)
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/healthz",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/ready",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/version",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/api/v1/canvas",method:"GET"})
MERGE (router)-[:CALLS]->(healthz)
MERGE (router)-[:CALLS]->(ready)
MERGE (router)-[:CALLS]->(version)
MERGE (router)-[:CALLS]->(canvas)
MERGE (ready)-[:USES_TYPE]->(metadata)
MERGE (version)-[:USES_TYPE]->(metadata)
@endcypher
*/
//! Star Canvas domain service metadata and Axum router.
//!
//! Provides health, version, and the current Canvas API scaffold.

#![warn(missing_docs)]

use axum::{routing::get, Json, Router};
use serde_json::json;

/// 服务元数据 (per 阶段 1 部署到 GHCR 后 K8s liveness probe 用).
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// 服务名.
    pub name: &'static str,
    /// 服务语义版本.
    pub version: &'static str,
    /// HTTP listener port (per deploy/star-canvas-services-k3s.yaml containerPort: 8081).
    pub http_port: u16,
    /// Current service implementation phase.
    pub phase: Phase,
}

/// 实现阶段 marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// Service scaffold with health and metadata endpoints.
    Skeleton,
    /// Product operations have been implemented.
    Implemented,
}

impl ServiceMetadata {
    /// domain-canvas 服务元数据.
    pub const DOMAIN_CANVAS: Self = Self {
        name: "domain-canvas",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8081,
        phase: Phase::Skeleton,
    };
}

/// domain-canvas 错误类型 (per 守门 #11).
#[derive(Debug, thiserror::Error)]
pub enum DomainCanvasError {
    /// 阶段 1 基础 占位.
    #[error("domain-canvas product operations are not implemented")]
    NotImplemented,
}

/// 构建 axum Router.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/api/v1/canvas", get(canvas_placeholder))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::DOMAIN_CANVAS.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::DOMAIN_CANVAS.name,
        "version": ServiceMetadata::DOMAIN_CANVAS.version,
        "phase": ServiceMetadata::DOMAIN_CANVAS.phase,
    }))
}

async fn canvas_placeholder() -> Json<serde_json::Value> {
    Json(json!({
        "endpoint": "/api/v1/canvas",
        "phase": "skeleton",
        "message": "Canvas domain API scaffold; product operations are not yet implemented.",
    }))
}
