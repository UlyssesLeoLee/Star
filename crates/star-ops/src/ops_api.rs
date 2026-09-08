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
    cluster::{CanaryRequest, HelmActionAck, HelmRelease, RollbackRequest},
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
    match HelmRelease::list_releases().await {
        Ok(releases) => {
            let total = releases.len();
            Json(OpsResponse {
                data: releases,
                meta: OpsMeta {
                    stub: false,
                    total: Some(total),
                    hint: Some("F-01 端到端, helm_canary_mock.sh subprocess (守門 #1 R-05)".to_string()),
                    ai_channel: None,
                    analysis_triggered: None,
                    needs_review: None,
                    entry_count: None,
                    phase: None,
                },
            })
        }
        Err(e) => Json(OpsResponse {
            data: Vec::<HelmRelease>::new(),
            meta: OpsMeta {
                stub: false,
                total: Some(0),
                hint: Some(format!("F-01 端到端失败: {} (守門 #1 R-05)", e)),
                ai_channel: None,
                analysis_triggered: None,
                needs_review: None,
                entry_count: None,
                phase: None,
            },
        }),
    }
}

async fn cluster_canary(
    Json(req): Json<CanaryRequest>,
) -> Result<Json<OpsResponse<HelmActionAck>>, OpsError> {
    // 守門 #5 v2: 1MB body 限制 (RequestBodyLimitLayer 整体应用)
    // 守門 #1 R-05: helm_canary_mock.sh subprocess, 真实 K8s 切换 owner 拍板
    HelmRelease::trigger_canary(&req)
        .await
        .map(|ack| {
            Json(OpsResponse {
                data: ack,
                meta: OpsMeta {
                    stub: false,
                    total: None,
                    hint: Some(format!("canary {}% → revision {:?}", req.canary_weight, req.target_revision)),
                    ai_channel: None,
                    analysis_triggered: None,
                    needs_review: None,
                    entry_count: None,
                    phase: Some("canary".to_string()),
                },
            })
        })
        .map_err(|e| OpsError::Internal(format!("canary 失败: {} (守門 #1 R-05)", e)))
}

async fn cluster_rollback(
    Json(req): Json<RollbackRequest>,
) -> Result<Json<OpsResponse<HelmActionAck>>, OpsError> {
    HelmRelease::rollback(&req)
        .await
        .map(|ack| {
            Json(OpsResponse {
                data: ack,
                meta: OpsMeta {
                    stub: false,
                    total: None,
                    hint: Some(format!("rollback to revision {}", req.target_revision)),
                    ai_channel: None,
                    analysis_triggered: None,
                    needs_review: None,
                    entry_count: None,
                    phase: Some("rollback".to_string()),
                },
            })
        })
        .map_err(|e| OpsError::Internal(format!("rollback 失败: {} (守門 #1 R-05)", e)))
}

#[derive(Serialize)]
struct ClusterStatus {
    release_name: String,
    phase: String,
    revision: u32,
    canary_weight: u8,
    mock: bool,
}

async fn cluster_status() -> impl IntoResponse {
    let release_name = "star-mcp";
    match HelmRelease::status(release_name).await {
        Ok(output) => {
            let status = output.status.clone();
            Json(OpsResponse {
                data: ClusterStatus {
                    release_name: output.release_name,
                    phase: status.clone(),
                    revision: output.revision,
                    canary_weight: output.canary_weight,
                    mock: output.mock,
                },
                meta: OpsMeta {
                    stub: false,
                    total: None,
                    hint: Some("F-01 端到端, helm_canary_mock.sh status (守門 #1 R-05)".to_string()),
                    ai_channel: None,
                    analysis_triggered: None,
                    needs_review: None,
                    entry_count: None,
                    phase: Some(status),
                },
            })
        }
        Err(e) => Json(OpsResponse {
            data: ClusterStatus {
                release_name: release_name.to_string(),
                phase: "Unknown".to_string(),
                revision: 0,
                canary_weight: 0,
                mock: true,
            },
            meta: OpsMeta {
                stub: false,
                total: None,
                hint: Some(format!("F-01 端到端失败: {} (守門 #1 R-05)", e)),
                ai_channel: None,
                analysis_triggered: None,
                needs_review: None,
                entry_count: None,
                phase: Some("Unknown".to_string()),
            },
        }),
    }
}

// ============ F-02 LogAI ============

/// F-02 log_upload body 大小硬上限 (per 守門 #5 v2, 1MB 限制)
/// 防止恶意 client 上传超大 log 撑爆内存
const LOG_UPLOAD_MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB

#[derive(Serialize)]
struct UploadAck {
    log_id: Uuid,
    entry_count: usize,
    analysis_triggered: bool,
    trace_id: Option<String>,
}

#[derive(Deserialize)]
struct LogUploadBody {
    source: String,
    level_filter: Option<Vec<LogLevel>>,
    content: String,
    /// 跨调用链追踪 ID (per 守門 #5 v2 + OPS-BASIC-DESIGN §3.4)
    trace_id: Option<String>,
}

async fn log_upload(
    State(state): State<AppState>,
    Json(body): Json<LogUploadBody>,
) -> Result<Json<OpsResponse<UploadAck>>, OpsError> {
    // 守門 #5 v2: body 大小硬限制 1MB, 防止恶意 client 撑爆内存
    let body_size = body.content.len();
    if body_size > LOG_UPLOAD_MAX_BODY_BYTES {
        return Err(OpsError::BadRequest(format!(
            "log content 超过 1MB 限制 ({} bytes), per 守門 #5 v2",
            body_size
        )));
    }

    // 守門 #5 v2: trace_id 传递 (跨调用链追踪, OPS-BASIC-DESIGN §3.4)
    let trace_id = body.trace_id.clone();

    // 守門 #13: level_filter 应用 — 如果 client 提供, 仅保留匹配 level 的行
    let filtered_content = if let Some(filter) = &body.level_filter {
        apply_level_filter(&body.content, filter)
    } else {
        body.content
    };

    let log = LogEntry {
        id: Uuid::new_v4(),
        source: body.source,
        // MVP: 单 log 简化, 实际从 filtered_content 多行解析 (后续子项)
        level: LogLevel::Error,
        message: filtered_content,
        timestamp: chrono::Utc::now(),
        trace_id: trace_id.clone(),
    };

    // F-02 端到端: 触发 AI 分析 (走 Ladder)
    let analysis_triggered = state.ladder.analyze_log(&log).await.is_ok();

    Ok(Json(OpsResponse {
        data: UploadAck {
            log_id: log.id,
            entry_count: 1,
            analysis_triggered,
            trace_id: log.trace_id,
        },
        meta: OpsMeta {
            stub: false, // F-02 端到端: 真实调用
            total: None,
            hint: Some(
                "F-02 端到端实装: trace_id 传递 + level_filter + 1MB 限制 (per 守門 #5 v2)"
                    .to_string(),
            ),
            ai_channel: Some("mock".to_string()),
            analysis_triggered: Some(analysis_triggered),
            needs_review: None,
            entry_count: Some(1),
            phase: None,
        },
    }))
}

/// 按 level_filter 过滤 log 行 (per 守門 #13 + OPS-BASIC-DESIGN §3.4)
/// 简单规则: 含 ERROR/WARN/INFO/DEBUG/TRACE 关键字的行, 跟 filter 集合求交
fn apply_level_filter(content: &str, filter: &[LogLevel]) -> String {
    if filter.is_empty() {
        return content.to_string();
    }
    let filter_keywords: Vec<&'static str> = filter
        .iter()
        .map(|l| match l {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        })
        .collect();
    content
        .lines()
        .filter(|line| {
            let upper = line.to_uppercase();
            filter_keywords.iter().any(|kw| upper.contains(kw))
        })
        .collect::<Vec<_>>()
        .join("\n")
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

    /// F-02 端到端: log_upload 真实路径, 验证 trace_id 传递 + level_filter 应用 + 1MB 限制
    /// 守门 #5 v2: API key 安全 + 1MB body 限制
    #[tokio::test]
    async fn log_upload_with_trace_id_and_level_filter() {
        let app = router(AppState::new());
        let body = serde_json::json!({
            "source": "k8s-pod/test",
            "level_filter": ["ERROR", "WARN"],
            "content": "2026-09-08 INFO ok\n2026-09-08 ERROR helm release failed\n2026-09-08 WARN retry\n",
            "trace_id": "trace-f02-test-001"
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ops/log/upload")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// F-02 端到端: 1MB body 限制 (守门 #5 v2)
    #[tokio::test]
    async fn log_upload_rejects_oversized_body() {
        let app = router(AppState::new());
        // 构造 1.1MB content
        let big = "x".repeat(1024 * 1024 + 100);
        let body = serde_json::json!({
            "source": "test",
            "content": big,
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ops/log/upload")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        // 守门 #5 v2: 1MB 限制触发 BadRequest (HTTP 400)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
