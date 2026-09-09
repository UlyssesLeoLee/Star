// SPDX-License-Identifier: MIT OR Apache-2.0
//! review 路由真实接线 (per Phase I Batch 2)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #15):
//! - `request` ← `star-mcp/src/tools/request_review.rs::invoke`

use std::sync::{Arc, OnceLock};

use axum::Json;
use domain_scm::{ActorContext, InMemoryScmService, PullRequestId, ScmCommandPort};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory SCM service
fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryScmService::new_for_test())
}

#[derive(Debug, Deserialize)]
pub struct RequestBody {
    /// MR ID (UUID) — 必填
    pub mr_id: String,
    /// 评审人 user ID 列表 — 可选
    pub reviewers: Option<Vec<String>>,
}

fn review_state_str(s: &domain_scm::ReviewState) -> &'static str {
    match s {
        domain_scm::ReviewState::Pending => "PENDING",
        domain_scm::ReviewState::Approved => "APPROVED",
        domain_scm::ReviewState::ChangesRequested => "CHANGES_REQUESTED",
        domain_scm::ReviewState::Commented => "COMMENTED",
    }
}

/// `POST /api/v1/reviews`
pub async fn request(
    Json(body): Json<RequestBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let pr_uuid = Uuid::parse_str(&body.mr_id).map_err(|e| {
        RestError::validation(
            format!("invalid mr_id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    let pr_id = PullRequestId::from_uuid(pr_uuid);
    let reviewers = body.reviewers.unwrap_or_default();

    let actor = ActorContext::default().with_role("developer");
    let result = service().request_review(pr_id, reviewers, actor).await?;
    let reviews: Vec<Value> = result
        .reviews
        .iter()
        .map(|r| {
            json!({
                "id": r.id.to_string(),
                "reviewer_user_id": r.reviewer_user_id.to_string(),
                "state": review_state_str(&r.state),
                "submitted_at": r.submitted_at.to_rfc3339(),
            })
        })
        .collect();
    Ok(Json(RestResponse::ok(json!({
        "review": {
            "id": format!("REV-{}", pr_id),
            "mr_id": body.mr_id,
            "state": result.state,
            "reviewers": result.reviewers,
            "reviews": reviews,
        }
    }))))
}
