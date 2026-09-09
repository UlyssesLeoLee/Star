// SPDX-License-Identifier: MIT OR Apache-2.0
//! validation 路由真实接线 (per Phase I Batch 2)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #17):
//! - `run` ← `star-mcp/src/tools/run_validation.rs::invoke` (走 list_results 真实 service)

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
pub(crate) fn service() -> &'static Arc<InMemoryValidationService> {
    static SVC: OnceLock<Arc<InMemoryValidationService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryValidationService::new_for_test())
}

#[derive(Debug, Deserialize)]
pub struct RunBody {
    /// Worktree ID (UUID) — 可选
    pub worktree_id: Option<String>,
    /// 校验类型列表 — 可选, 缺省 7 类 SOW
    pub kinds: Option<Vec<String>>,
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

fn parse_kind(s: &str) -> Option<ValidationKind> {
    match s {
        "BUILD" => Some(ValidationKind::Build),
        "UNIT_TEST" => Some(ValidationKind::UnitTest),
        "INTEGRATION_TEST" => Some(ValidationKind::IntegrationTest),
        "LINT" => Some(ValidationKind::Lint),
        "FORMAT" => Some(ValidationKind::Format),
        "STATIC_ANALYSIS" => Some(ValidationKind::StaticAnalysis),
        "SECURITY_CHECK" => Some(ValidationKind::SecurityCheck),
        "ACCEPTANCE_CHECK" => Some(ValidationKind::AcceptanceCheck),
        "REVIEW" => Some(ValidationKind::Review),
        "CUSTOM_VALIDATION" => Some(ValidationKind::CustomValidation),
        _ => None,
    }
}

/// `POST /api/v1/validations`
pub async fn run(Json(body): Json<RunBody>) -> Result<Json<RestResponse<Value>>, RestError> {
    let worktree_id = body
        .worktree_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(domain_validation::WorktreeId::from_uuid);
    let kinds: Vec<ValidationKind> = body
        .kinds
        .unwrap_or_default()
        .iter()
        .filter_map(|s| parse_kind(s))
        .collect();
    let kinds = if kinds.is_empty() {
        ValidationKind::SOW_REQUIRED.to_vec()
    } else {
        kinds
    };

    let actor = ActorContext::nil_actor_with_tenant(Uuid::nil()).with_role("service_internal");
    let tenant_id = TenantId(actor.tenant_id);
    let kind_filter = kinds.first().copied();
    let q = ListValidationQuery {
        tenant_id,
        work_item_id: None,
        worktree_id,
        kind: kind_filter,
        status: None,
        limit: 10,
        offset: 0,
    };
    let results = service().list_results(q, actor).await?;

    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut failed_tests: Vec<String> = Vec::new();
    let mut items_json: Vec<Value> = Vec::with_capacity(results.len());
    for r in &results {
        match r.status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::Failed => {
                failed += 1;
                failed_tests.push(format!(
                    "{}/{}/{}",
                    r.kind,
                    r.work_item_id.map(|x| x.to_string()).unwrap_or_default(),
                    r.worktree_id.map(|x| x.to_string()).unwrap_or_default(),
                ));
            }
            ValidationStatus::Skipped => skipped += 1,
            _ => {}
        }
        items_json.push(json!({
            "id": r.id.to_string(),
            "kind": r.kind.as_str(),
            "status": status_str(r.status),
            "work_item_id": r.work_item_id.map(|x| x.to_string()),
            "worktree_id": r.worktree_id.map(|x| x.to_string()),
            "log_excerpt_ref": r.log_excerpt_ref,
            "is_ai_complete_claim": r.is_ai_complete_claim,
        }));
    }
    Ok(Json(RestResponse::ok(json!({
        "validation": {
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "failed_tests": failed_tests,
            "items": items_json,
            "kinds": kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
        }
    }))))
}
