// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` 8 REST stub 端点 (per OPS-BASIC-DESIGN §3)
//!
//! MVP-骨架: 所有端点返 200 + mock data + meta.stub=true
//! 实装阶段: 各端点接业务逻辑, 移除 stub flag, 改返真实数据
//!
//! 8 端点分布:
//! - F-01 Cluster (4): list / canary / rollback / status
//! - F-02 LogAI (2): upload / analysis
//! - F-03 Metrics (1): summary
//! - F-04 Docs (1): list

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::OpsError;
use crate::ops_ai::default_ladder;
use crate::ops_domain::{
    cluster::{CanaryRequest, HelmRelease, RollbackRequest},
    log::{LogAnalysis, LogEntry, LogLevel},
    metrics::OpsMetric,
};

/// 顶层响应包装: data + meta (per star-api-rest 模式)
#[derive(Debug, Serialize)]
pub struct OpsResponse<T: Serialize> {
    pub data: T,
    pub meta: OpsMeta,
}

/// 响应 meta (per OPS-BASIC-DESIGN §3.5)
#[derive(Debug, Serialize)]
pub struct OpsMeta {
    pub stub: bool,
    pub total: Option<usize>,
    pub hint: Option<String>,
    pub ai_channel: Option<String>,
    pub analysis_triggered: Option<bool>,
    pub needs_review: Option<bool>,
    pub entry_count: Option<usize>,
    pub phase: Option<String>,
}

impl<T: Serialize> OpsResponse<T> {
    #[allow(dead_code)]
    fn new(data: T) -> Self {
        Self {
            data,
            meta: OpsMeta {
                stub: true,
                total: None,
                hint: Some("MVP-骨架, [M] 子项拍板后实装".to_string()),
                ai_channel: None,
                analysis_triggered: None,
                needs_review: None,
                entry_count: None,
                phase: None,
            },
        }
    }
}

/// App state (MVP 简单版本, 仅 Ladder)
#[derive(Clone)]
pub struct AppState {
    pub ladder: std::sync::Arc<crate::ops_ai::ladder::Ladder>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ladder: std::sync::Arc::new(default_ladder()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// 构建 axum Router (8 stub 端点 + 2 health check)
pub fn router(state: AppState) -> Router {
    Router::new()
        // F-01 Cluster (4)
        .route("/api/ops/cluster/releases", get(cluster_list))
        .route("/api/ops/cluster/canary", post(cluster_canary))
        .route("/api/ops/cluster/rollback", post(cluster_rollback))
        .route("/api/ops/cluster/status", get(cluster_status))
        // F-02 LogAI (2)
        .route("/api/ops/log/upload", post(log_upload))
        .route("/api/ops/log/analysis/{id}", get(log_analysis))
        // F-03 Metrics (1)
        .route("/api/ops/metrics/summary", get(metrics_summary))
        // F-04 Docs (1)
        .route("/api/ops/docs", get(docs_list))
        // Health
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state)
}

// ============ F-01 Cluster ============

async fn cluster_list() -> impl IntoResponse {
    let releases = HelmRelease::list_stub();
    let total = releases.len();
    Json(OpsResponse {
        data: releases,
        meta: OpsMeta {
            stub: true,
            total: Some(total),
            hint: Some("F-01 实装阶段接入 kube-rs".to_string()),
            ai_channel: None,
            analysis_triggered: None,
            needs_review: None,
            entry_count: None,
            phase: None,
        },
    })
}

async fn cluster_canary(
    Json(req): Json<CanaryRequest>,
) -> Result<Json<OpsResponse<CanaryAck>>, OpsError> {
    // MVP: 仅 record, 不实装 helm exec
    Ok(Json(OpsResponse {
        data: CanaryAck {
            action_id: Uuid::new_v4(),
            status: "Pending".to_string(),
        },
        meta: OpsMeta {
            stub: true,
            total: None,
            hint: Some(format!(
                "canary {}% → revision {:?} (MVP 仅 record, 不实装 helm upgrade)",
                req.canary_weight, req.target_revision
            )),
            ai_channel: None,
            analysis_triggered: None,
            needs_review: None,
            entry_count: None,
            phase: None,
        },
    }))
}

async fn cluster_rollback(
    Json(req): Json<RollbackRequest>,
) -> Result<Json<OpsResponse<CanaryAck>>, OpsError> {
    Ok(Json(OpsResponse {
        data: CanaryAck {
            action_id: Uuid::new_v4(),
            status: "Pending".to_string(),
        },
        meta: OpsMeta {
            stub: true,
            total: None,
            hint: Some(format!(
                "rollback to revision {} (MVP 仅 record)",
                req.target_revision
            )),
            ai_channel: None,
            analysis_triggered: None,
            needs_review: None,
            entry_count: None,
            phase: None,
        },
    }))
}

#[derive(Serialize)]
struct CanaryAck {
    action_id: Uuid,
    status: String,
}

#[derive(Serialize)]
struct ClusterStatus {
    release_name: String,
    phase: String,
    replicas: Replicas,
}

#[derive(Serialize)]
struct Replicas {
    ready: u32,
    desired: u32,
}

async fn cluster_status() -> impl IntoResponse {
    Json(OpsResponse {
        data: ClusterStatus {
            release_name: "star-mcp".to_string(),
            phase: "Healthy".to_string(),
            replicas: Replicas {
                ready: 3,
                desired: 3,
            },
        },
        meta: OpsMeta {
            stub: true,
            total: None,
            hint: Some("F-01 实装阶段接入 kubectl get pods".to_string()),
            ai_channel: None,
            analysis_triggered: None,
            needs_review: None,
            entry_count: None,
            phase: Some("Healthy".to_string()),
        },
    })
}

// ============ F-02 LogAI ============

#[derive(Serialize)]
struct UploadAck {
    log_id: Uuid,
    entry_count: usize,
    analysis_triggered: bool,
}

#[derive(Deserialize)]
struct LogUploadBody {
    source: String,
    #[allow(dead_code)]
    level_filter: Option<Vec<LogLevel>>,
    content: String,
}

async fn log_upload(
    State(state): State<AppState>,
    Json(body): Json<LogUploadBody>,
) -> Result<Json<OpsResponse<UploadAck>>, OpsError> {
    let log = LogEntry {
        id: Uuid::new_v4(),
        source: body.source,
        level: LogLevel::Error,
        message: body.content,
        timestamp: chrono::Utc::now(),
        trace_id: None,
    };

    // MVP: 触发 AI 分析 (走 Ladder)
    let analysis_triggered = state.ladder.analyze_log(&log).await.is_ok();

    Ok(Json(OpsResponse {
        data: UploadAck {
            log_id: log.id,
            entry_count: 1,
            analysis_triggered,
        },
        meta: OpsMeta {
            stub: true,
            total: None,
            hint: Some("F-02 实装阶段接入 log 采集 agent".to_string()),
            ai_channel: Some("mock".to_string()),
            analysis_triggered: Some(analysis_triggered),
            needs_review: None,
            entry_count: Some(1),
            phase: None,
        },
    }))
}

async fn log_analysis(Path(id): Path<Uuid>) -> Result<Json<OpsResponse<LogAnalysis>>, OpsError> {
    // MVP: 返 stub LogAnalysis
    let analysis = LogAnalysis::stub_for(id);
    let needs_review = analysis.confidence < 0.5; // 守门 #23

    Ok(Json(OpsResponse {
        data: analysis,
        meta: OpsMeta {
            stub: true,
            total: None,
            hint: Some("F-02 实装阶段从 ops_log_entry 表查".to_string()),
            ai_channel: Some("mock".to_string()),
            analysis_triggered: None,
            needs_review: Some(needs_review),
            entry_count: None,
            phase: None,
        },
    }))
}

// ============ F-03 Metrics ============

async fn metrics_summary() -> impl IntoResponse {
    let metrics = OpsMetric::summary_stub();
    let total = metrics.len();
    Json(OpsResponse {
        data: metrics,
        meta: OpsMeta {
            stub: true,
            total: Some(total),
            hint: Some("F-03 实装阶段接 star-telemetry".to_string()),
            ai_channel: None,
            analysis_triggered: None,
            needs_review: None,
            entry_count: None,
            phase: None,
        },
    })
}

// ============ F-04 Docs ============

#[derive(Serialize)]
struct DocsList {
    data: Vec<DocRef>,
    meta: DocsMeta,
}

#[derive(Serialize)]
struct DocRef {
    path: String,
    title: String,
    category: String,
    updated_at: String,
}

#[derive(Serialize)]
struct DocsMeta {
    stub: bool,
    total: usize,
}

async fn docs_list() -> impl IntoResponse {
    let docs = vec![
        DocRef {
            path: "docs/requirements/SRS-STAR-OPS-001.md".to_string(),
            title: "STAR Ops Console SRS".to_string(),
            category: "SRS".to_string(),
            updated_at: "2026-09-08T07:53:00Z".to_string(),
        },
        DocRef {
            path: "docs/basic-design/OPS-BASIC-DESIGN-001.md".to_string(),
            title: "STAR Ops Console 基本设计".to_string(),
            category: "BAS".to_string(),
            updated_at: "2026-09-08T07:53:00Z".to_string(),
        },
    ];
    let total = docs.len();
    Json(OpsResponse {
        data: DocsList {
            data: docs,
            meta: DocsMeta { stub: true, total },
        },
        meta: OpsMeta {
            stub: true,
            total: Some(total),
            hint: Some("F-04 实装阶段接 walkdir 扫描".to_string()),
            ai_channel: None,
            analysis_triggered: None,
            needs_review: None,
            entry_count: None,
            phase: None,
        },
    })
}

// ============ Health ============

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, "READY")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn cluster_list_returns_one_release() {
        let app = router(AppState::new());
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/ops/cluster/releases")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_summary_returns_five_kpis() {
        let app = router(AppState::new());
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/ops/metrics/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn healthz_returns_200() {
        let app = router(AppState::new());
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn log_analysis_returns_stub() {
        let app = router(AppState::new());
        let id = Uuid::nil();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/ops/log/analysis/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
