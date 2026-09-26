//! canvas-engine lib: 共享元数据 + axum router builder (per ULYS-160 §1 阶段 1 骨架)
//!
//! 阶段 1: 仅返回服务元数据 (name / version / ports / phase marker), 0 业务.
//! 阶段 2+: 落地 5 domain 共享 API + 平台能力 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.1).
//!
//! 守门 #14 v4 Mavis 临时代签 5 域 Lead 决策 per 9/3 11:35 JST 反转.
//! 真人到位追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).
//! 守门 #7 0 unsafe (per workspace lint unsafe_code = "forbid").
//! 守门 #11 缺标比错标 (NotImplemented 占位, 阶段 2 落地).

#![warn(missing_docs)]

use axum::{routing::get, Json, Router};
use serde_json::json;

/// 服务元数据 (per 阶段 1 部署到 GHCR 后 K8s liveness probe 用).
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// 服务名 (K8s deployment label app: canvas-engine).
    pub name: &'static str,
    /// 服务语义版本 (per Cargo.toml version).
    pub version: &'static str,
    /// HTTP 监听端口 (per deploy/canvas-game-k3s.yaml containerPort).
    pub http_port: u16,
    /// 阶段 1 骨架 marker (阶段 2+ 落地业务后改 `Phase::Implemented`).
    pub phase: Phase,
}

/// 实现阶段 marker (per 守门 #11 缺标比错标).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// 阶段 1: 骨架, 仅健康检查 + 元数据 + 占位 endpoint.
    Skeleton,
    /// 阶段 2+: 业务 logic 已落地 (per SRS / BD / DD 文档).
    Implemented,
}

impl ServiceMetadata {
    /// canvas-engine 服务元数据 (per deploy/canvas-game-k3s.yaml containerPort: 8080).
    pub const CANVAS_ENGINE: Self = Self {
        name: "canvas-engine",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8080,
        phase: Phase::Skeleton,
    };
}

/// canvas-engine 错误类型 (per 守门 #11).
#[derive(Debug, thiserror::Error)]
pub enum CanvasEngineError {
    /// 阶段 1 基础 占位.
    #[error("canvas-engine skeleton: not implemented (阶段 2 落地)")]
    NotImplemented,
}

/// 构建 axum Router (阶段 1: healthz + /ready + /version).
///
/// 阶段 2+: 加 /api/v1/canvas/* 业务 endpoint.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "version": ServiceMetadata::CANVAS_ENGINE.version,
        "phase": ServiceMetadata::CANVAS_ENGINE.phase,
    }))
}