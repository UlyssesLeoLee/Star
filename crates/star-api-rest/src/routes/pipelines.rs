// SPDX-License-Identifier: MIT OR Apache-2.0
//! pipeline 路由真实接线 (per Phase I Batch 2)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #16):
//! - `get_status` ← `star-mcp/src/tools/get_pipeline_status.rs::invoke`

use std::sync::{Arc, OnceLock};

use axum::{extract::Path, Json};
use domain_scm::{ActorContext, InMemoryScmService};
use serde_json::{json, Value};

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory SCM service
pub(crate) fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryScmService::new_for_test())
}

fn pipeline_status_str(s: domain_scm::PipelineStatus) -> &'static str {
    match s {
        domain_scm::PipelineStatus::Pending => "PENDING",
        domain_scm::PipelineStatus::Running => "RUNNING",
        domain_scm::PipelineStatus::Success => "SUCCESS",
        domain_scm::PipelineStatus::Failed => "FAILED",
        domain_scm::PipelineStatus::Canceled => "CANCELED",
    }
}

/// `GET /api/v1/pipelines/{id}` (id = external_id, e.g. "PIPE-xxx")
pub async fn get_status(Path(id): Path<String>) -> Result<Json<RestResponse<Value>>, RestError> {
    if id.is_empty() {
        return Err(RestError::validation(
            "pipeline_run_id is required".to_string(),
            "Provide a non-empty id",
        ));
    }
    let actor = ActorContext::default().with_role("developer");
    let pipeline = service().find_pipeline_by_external_id(&id, actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "pipeline": {
            "id": pipeline.id.to_string(),
            "external_id": pipeline.external_id,
            "status": pipeline_status_str(pipeline.status),
            "head_sha": pipeline.head_sha,
            "url": pipeline.url,
            "started_at": pipeline.started_at.map(|t| t.to_rfc3339()),
            "finished_at": pipeline.finished_at.map(|t| t.to_rfc3339()),
            "created_at": pipeline.created_at.to_rfc3339(),
            "lock_version": pipeline.lock_version,
        }
    }))))
}
