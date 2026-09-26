//! domain-canvas lib: 业务域画布元数据 + axum router builder (per ULYS-160 §2 阶段 1 骨架)
//!
//! 阶段 1: 健康检查 + 服务版本 + 占位 endpoint. 0 业务.
//! 阶段 2+: 26 表 W/T/M + 5 角色 业务逻辑 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.2).

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
    /// HTTP 监听端口 (per deploy/canvas-game-k3s.yaml containerPort: 8081).
    pub http_port: u16,
    /// 阶段 1 骨架 marker.
    pub phase: Phase,
}

/// 实现阶段 marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// 阶段 1: 骨架, 仅健康检查 + 元数据 + 占位 endpoint.
    Skeleton,
    /// 阶段 2+: 业务 logic 已落地.
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
    #[error("domain-canvas skeleton: not implemented (阶段 2 落地)")]
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
        "message": "阶段 2 落地: 26 表 W/T/M + 5 角色 CRUD (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.2)",
    }))
}