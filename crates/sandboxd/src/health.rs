/*
@cypher schema=1 source_sha256=aca2ffbe5108ec3cf5885cf2dd5c733f82c50391d8821da290765d5f643d3e54
MERGE (self:File {path:"crates/sandboxd/src/health.rs"})
@endcypher
*/
//! `sandboxd` health 模块 (per FR-6.3, BD-002 §6.5)
//!
//! `/healthz` 返回 `{status, uptime_sec, sessions_active, sessions_total, version}`,
//! 供 systemd / k8s liveness probe + 调试 (per FR-6.3)。

use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::metrics::SandboxdMetrics;
use crate::orchestrator::Orchestrator;

/// axum 共享状态 (healthz + metrics handler)
#[derive(Clone)]
pub struct AppState {
    /// daemon 启动时刻 (uptime 计算)
    pub started_at: Instant,
    /// orchestrator (active/total 计数)
    pub orchestrator: Arc<Orchestrator>,
    /// Prometheus 指标
    pub metrics: Arc<SandboxdMetrics>,
    /// 版本号
    pub version: &'static str,
}

/// 构造 axum Router (/healthz + /metrics, per FR-6.1 + FR-6.3)
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .with_state(state)
}

/// /healthz handler (per FR-6.3 响应字段)
async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.orchestrator.health_status();
    let code = if status.healthy {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(json!({
            "status": if status.healthy { "ok" } else { "degraded" },
            "uptime_sec": state.started_at.elapsed().as_secs(),
            "sessions_active": state.orchestrator.active_count(),
            "sessions_total": state.orchestrator.total_count(),
            "version": state.version,
        })),
    )
}

/// /metrics handler (Prometheus 文本格式, per FR-6.1)
async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics.render() {
        Ok(body) => (
            axum::http::StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4",
            )],
            body,
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; charset=utf-8",
            )],
            format!("metrics render failed: {e}"),
        ),
    }
}
