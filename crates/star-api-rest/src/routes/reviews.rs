// SPDX-License-Identifier: MIT OR Apache-2.0
//! review 路由真实接线 (per Phase I Batch 2; 2026-11-19 批次 B 真实化)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #15):
//! - `request` ← `star-mcp/src/tools/request_review.rs::invoke`
//!
//! 批次 B 变更 (per F06 §6 "有名无实"治理):
//! - 身份: `AuthenticatedUser` → `actor_from_auth` (不再 default 假身份)
//! - service: 与 merge_requests / pipelines **同一共享实例** (ScmApiState 注入),
//!   request_review 能看见 create_mr 写入的 PR
//! - 新增 `GET /reviews?mr_id=<uuid>` (列某 PR 的 review, INV-SCM-04)
//! - 新增 `POST /reviews/:id/decision` (仅 reviewer 本人裁决, domain decide_review)

use std::sync::{Arc, OnceLock};

use axum::extract::{Path, Query, State};
use axum::routing::post;
use axum::{Json, Router};
use domain_scm::{
    ActorContext, InMemoryScmService, PullRequestId, ReviewDecisionInput, ReviewId, ReviewState,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::oauth::middleware::AuthenticatedUser;
use crate::error::RestError;
use crate::response::RestResponse;

use super::scm_state::{actor_from_auth, ScmApiState};

/// 兼容路径共享 service (仅 `build_router()` stub-auth 骨架用)
pub(crate) fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(InMemoryScmService::new_for_test)
}

#[derive(Debug, Deserialize)]
pub struct RequestBody {
    /// MR ID (UUID) — 必填
    pub mr_id: String,
    /// 评审人 user ID 列表 — 可选
    pub reviewers: Option<Vec<String>>,
}

/// `POST /api/v1/reviews` — 生产版 (ScmApiState 注入 + 真实身份)
pub async fn request(
    State(state): State<ScmApiState>,
    auth: AuthenticatedUser,
    Json(body): Json<RequestBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    request_impl(&state.service, actor_from_auth(&auth), body).await
}

/// `POST /api/v1/reviews` — 兼容版 (build_router stub-auth 预览骨架,
/// 保持 Phase I Batch 2 的无鉴权行为; 生产流量一律走 build_scm_router)
pub async fn request_compat(
    Json(body): Json<RequestBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    request_impl(service(), ActorContext::default(), body).await
}

async fn request_impl(
    svc: &InMemoryScmService,
    actor: ActorContext,
    body: RequestBody,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let pr_uuid = Uuid::parse_str(&body.mr_id).map_err(|e| {
        RestError::validation(format!("invalid mr_id UUID: {e}"), "Provide a valid UUID")
    })?;
    let pr_id = PullRequestId::from_uuid(pr_uuid);
    let reviewers = body.reviewers.unwrap_or_default();

    let result = svc.request_review(pr_id, reviewers, actor).await?;
    let reviews: Vec<Value> = result.reviews.iter().map(review_json).collect();
    Ok(Json(RestResponse::ok(json!({
        "review": {
            "mr_id": body.mr_id,
            "state": result.state,
            "reviewers": result.reviewers,
            "reviews": reviews,
        }
    }))))
}

fn review_json(r: &domain_scm::Review) -> Value {
    json!({
        "id": r.id.to_string(),
        "pull_request_id": r.pull_request_id.to_string(),
        "reviewer_user_id": r.reviewer_user_id.to_string(),
        "state": r.state.as_api_str(),
        "body": r.body,
        "submitted_at": r.submitted_at.to_rfc3339(),
        "created_at": r.created_at.to_rfc3339(),
        "lock_version": r.lock_version,
    })
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// MR ID (UUID) — 必填
    pub mr_id: String,
}

/// `GET /api/v1/reviews?mr_id=<uuid>` — 生产版 (列某 PR 的 review)
pub async fn list(
    State(state): State<ScmApiState>,
    auth: AuthenticatedUser,
    Query(params): Query<ListQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    list_impl(&state.service, actor_from_auth(&auth), params).await
}

async fn list_impl(
    svc: &InMemoryScmService,
    actor: ActorContext,
    params: ListQuery,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let pr_uuid = Uuid::parse_str(&params.mr_id).map_err(|e| {
        RestError::validation(format!("invalid mr_id UUID: {e}"), "Provide a valid UUID")
    })?;
    let pr_id = PullRequestId::from_uuid(pr_uuid);
    let reviews = svc.list_reviews_for_pr(pr_id, actor).await?;
    let items: Vec<Value> = reviews.iter().map(review_json).collect();
    Ok(Json(RestResponse::ok(json!({ "reviews": items }))))
}

#[derive(Debug, Deserialize)]
pub struct DecisionBody {
    /// 裁决状态: Approved / ChangesRequested / Commented
    pub state: String,
    /// 评审内容 — 可选
    pub body: Option<String>,
}

/// `POST /api/v1/reviews/:id/decision` — 生产版 (仅 reviewer 本人)
pub async fn decide(
    State(state): State<ScmApiState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(body): Json<DecisionBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    decide_impl(&state.service, actor_from_auth(&auth), id, body).await
}

async fn decide_impl(
    svc: &InMemoryScmService,
    actor: ActorContext,
    id: Uuid,
    body: DecisionBody,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let state = match body.state.as_str() {
        "Approved" | "APPROVED" => ReviewState::Approved,
        "ChangesRequested" | "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
        "Commented" | "COMMENTED" => ReviewState::Commented,
        other => {
            return Err(RestError::validation(
                format!("unknown decision state: {other}"),
                "Use one of Approved/ChangesRequested/Commented",
            ))
        }
    };
    let input = ReviewDecisionInput {
        state,
        body: body.body,
    };
    let review = svc
        .decide_review(ReviewId::from_uuid(id), input, actor)
        .await?;
    Ok(Json(RestResponse::ok(
        json!({ "review": review_json(&review) }),
    )))
}

/// 生产 router: 共享 ScmApiState + 真实身份 (由 build_supplemental_routers 挂载)
pub(crate) fn router(state: ScmApiState) -> Router {
    Router::new()
        .route("/api/v1/reviews", post(request).get(list))
        .route("/api/v1/reviews/{id}/decision", post(decide))
        .with_state(state)
}
