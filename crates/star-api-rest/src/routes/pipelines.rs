// SPDX-License-Identifier: MIT OR Apache-2.0
//! pipeline 路由真实接线 (per Phase I Batch 2; 2026-11-19 批次 B 真实化)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #16):
//! - `get_status` ← `star-mcp/src/tools/get_pipeline_status.rs::invoke`
//!
//! 批次 B 变更 (per F06 §6 "有名无实"治理):
//! - 身份: `AuthenticatedUser` → `actor_from_auth` (不再 default 假身份)
//! - service: 与 merge_requests / reviews **同一共享实例** (ScmApiState 注入),
//!   register_pipeline 写入的 pipeline 经 external_id 索引可查

use std::sync::{Arc, OnceLock};

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use domain_scm::{ActorContext, InMemoryScmService};
use serde_json::{json, Value};

use crate::auth::oauth::middleware::AuthenticatedUser;
use crate::error::RestError;
use crate::response::RestResponse;

use super::scm_state::{actor_from_auth, ScmApiState};

/// 兼容路径共享 service (仅 `build_router()` stub-auth 骨架用)
pub(crate) fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(InMemoryScmService::new_for_test)
}

/// `GET /api/v1/pipelines/{id}` (id = external_id, e.g. "PIPE-xxx") — 生产版
pub async fn get_status(
    State(state): State<ScmApiState>,
    auth: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    get_status_impl(&state.service, actor_from_auth(&auth), id).await
}

/// `GET /api/v1/pipelines/{id}` — 兼容版 (build_router stub-auth 预览骨架,
/// 保持 Phase I Batch 2 的无鉴权行为; 生产流量一律走 build_scm_router)
pub async fn get_status_compat(
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    get_status_impl(service(), ActorContext::default(), id).await
}

async fn get_status_impl(
    svc: &InMemoryScmService,
    actor: ActorContext,
    id: String,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if id.is_empty() {
        return Err(RestError::validation(
            "pipeline_run_id is required".to_string(),
            "Provide a non-empty id",
        ));
    }
    let pipeline = svc.find_pipeline_by_external_id(&id, actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "pipeline": {
            "id": pipeline.id.to_string(),
            "external_id": pipeline.external_id,
            "status": pipeline.status.as_api_str(),
            "head_sha": pipeline.head_sha,
            "url": pipeline.url,
            "started_at": pipeline.started_at.map(|t| t.to_rfc3339()),
            "finished_at": pipeline.finished_at.map(|t| t.to_rfc3339()),
            "created_at": pipeline.created_at.to_rfc3339(),
            "lock_version": pipeline.lock_version,
        }
    }))))
}

/// 生产 router: 共享 ScmApiState + 真实身份 (由 build_supplemental_routers 挂载)
pub(crate) fn router(state: ScmApiState) -> Router {
    Router::new()
        .route("/api/v1/pipelines/{id}", get(get_status))
        .with_state(state)
}
