//! canvas-game lib: 画布游戏引擎元数据 + axum router builder (per ULYS-160 §4 阶段 1 骨架)
//!
//! 阶段 1: 健康检查 + 服务版本 + 占位 endpoint. 0 业务.
//! 阶段 2+: 角色 / 弹幕 / 战斗 / 3D sprite 游戏逻辑 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.4).

#![warn(missing_docs)]

use axum::{routing::get, Json, Router};
use serde_json::json;

/// 服务元数据.
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// 服务名.
    pub name: &'static str,
    /// 服务语义版本.
    pub version: &'static str,
    /// HTTP 监听端口 (per deploy/canvas-game-k3s.yaml containerPort: 8084).
    pub http_port: u16,
    /// 阶段 1 骨架 marker.
    pub phase: Phase,
}

/// 实现阶段 marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// 阶段 1: 骨架.
    Skeleton,
    /// 阶段 2+: 业务 logic 已落地.
    Implemented,
}

impl ServiceMetadata {
    /// canvas-game 服务元数据.
    pub const CANVAS_GAME: Self = Self {
        name: "canvas-game",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8084,
        phase: Phase::Skeleton,
    };
}

/// canvas-game 错误类型.
#[derive(Debug, thiserror::Error)]
pub enum CanvasGameError {
    /// 阶段 1 基础 占位.
    #[error("canvas-game skeleton: not implemented (阶段 2 落地)")]
    NotImplemented,
}

/// 构建 axum Router.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/api/v1/gameplay", get(gameplay_placeholder))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_GAME.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_GAME.name,
        "version": ServiceMetadata::CANVAS_GAME.version,
        "phase": ServiceMetadata::CANVAS_GAME.phase,
    }))
}

async fn gameplay_placeholder() -> Json<serde_json::Value> {
    Json(json!({
        "endpoint": "/api/v1/gameplay",
        "phase": "skeleton",
        "message": "阶段 2 落地: 角色 / 弹幕 / 战斗 / 3D sprite 游戏逻辑 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.4)",
    }))
}