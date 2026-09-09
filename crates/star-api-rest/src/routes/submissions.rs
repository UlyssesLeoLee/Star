// SPDX-License-Identifier: MIT OR Apache-2.0
//! submission 路由真实接线 (per Phase I Batch 2)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #18):
//! - `submit` ← `star-mcp/src/tools/submit.rs::invoke` (走 list_results 真实 service, P0 简化)
//!
//! 已知缺口 (per 独立 WBS §3 #2): 该 MCP 工具自身文档承认 step 6-12 是"简化 mock",
//! 照抄后 /api/v1/submissions 依然不是 100% 真实, 这是上游既有设计决策.

use std::sync::{Arc, OnceLock};

use axum::Json;
use domain_validation::{
    ActorContext, InMemoryValidationService, ListValidationQuery, TenantId, ValidationKind,
    ValidationQueryPort, ValidationStatus,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory validation service
fn service() -> &'static Arc<InMemoryValidationService> {
    static SVC: OnceLock<Arc<InMemoryValidationService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryValidationService::new_for_test())
}

#[derive(Debug, Deserialize)]
pub struct SubmitBody {
    /// Worktree ID (UUID) — 必填
    pub worktree_id: String,
    /// WorkItem ID (UUID) — 可选
    pub work_item_id: Option<String>,
}

fn status_str(s: ValidationStatus) -> &'static str {
    match s {
        ValidationStatus::Pending => "PENDING",
        ValidationStatus::Running => "RUNNING",
        ValidationStatus::Passed => "PASSED",
        ValidationStatus::Failed => "FAILED",
        ValidationStatus::Skipped => "SKIPPED",
    }
}

/// `POST /api/v1/submissions`
pub async fn submit(
    Json(body): Json<SubmitBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let worktree_uuid = Uuid::parse_str(&body.worktree_id).map_err(|e| {
        RestError::validation(
            format!("invalid worktree_id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    let worktree_id = domain_validation::WorktreeId::from_uuid(worktree_uuid);

    let actor = ActorContext::nil_actor_with_tenant(Uuid::nil()).with_role("service_internal");
    let tenant_id = TenantId(actor.tenant_id);
    let q = ListValidationQuery {
        tenant_id,
        work_item_id: None,
        worktree_id: Some(worktree_id),
        kind: Some(ValidationKind::Build),
        status: None,
        limit: 10,
        offset: 0,
    };
    let results = service().list_results(q, actor).await?;
    let items: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "id": r.id.to_string(),
                "kind": r.kind.as_str(),
                "status": status_str(r.status),
                "worktree_id": r.worktree_id.map(|x| x.to_string()),
                "work_item_id": r.work_item_id.map(|x| x.to_string()),
            })
        })
        .collect();
    Ok(Json(RestResponse::ok(json!({
        "submission": {
            "worktree_id": body.worktree_id,
            "work_item_id": body.work_item_id,
            "validations": items,
            "total": items.len(),
            "note": "Phase I Batch 2 简化: 走 list_results 真实 service 路径, step 6-12 简化为只读 list, 完整 submit 流程留 Phase II 持久化时 (per 独立 WBS §3 #2)",
        }
    }))))
}
