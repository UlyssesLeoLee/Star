// SPDX-License-Identifier: MIT OR Apache-2.0
//! merge_request 路由真实接线 (per Phase I Batch 2; 2026-11-19 批次 B 真实化)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #14):
//! - `create` ← `star-mcp/src/tools/create_merge_request.rs::invoke`
//!
//! 批次 B 变更 (per F06 §6 "有名无实"治理):
//! - 身份: `AuthenticatedUser` extractor (JWT 验签 + 设备代际) → `actor_from_auth`,
//!   不再 `ActorContext::default()` 全零假身份
//! - service: 生产走 `ScmApiState` 注入的**单一共享实例** (MR/Review/Pipeline 同源);
//!   `build_router()` 兼容路径用 `service()` OnceLock (reset_all_state 复用)
//! - 新增 `GET /merge-requests` (INV-SCM-04 project-scoped 可见性)
//! - 新增 `POST /merge-requests/:id/transition` (domain 状态机白名单)

use std::sync::{Arc, OnceLock};

use axum::extract::{Path, Query, State};
use axum::routing::post;
use axum::{Json, Router};
use domain_scm::{
    ActorContext, CreateMRInput, InMemoryScmService, PullRequestId, PullRequestState,
    RecordPullRequestTransitionCommand, RepositoryId, ScmCommandPort, TenantId,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::oauth::middleware::AuthenticatedUser;
use crate::error::RestError;
use crate::response::RestResponse;

use super::scm_state::{actor_from_auth, ScmApiState};

/// 兼容路径共享 service (仅 `build_router()` stub-auth 骨架用;
/// 生产 `build_scm_router` 一律注入 ScmApiState.service)
pub(crate) fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(InMemoryScmService::new_for_test)
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    /// MR 标题 — 必填
    pub title: String,
    /// 源分支 (head) — 必填
    pub head: String,
    /// 目标分支 (base) — 必填
    pub base: String,
    /// 仓库 ID (UUID) — 必填
    pub repository_id: String,
    /// MR 描述 — 可选
    pub description: Option<String>,
}

/// `POST /api/v1/merge-requests` — 生产版 (ScmApiState 注入 + 真实身份)
pub async fn create(
    State(state): State<ScmApiState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    create_impl(&state.service, actor_from_auth(&auth), body).await
}

/// `POST /api/v1/merge-requests` — 兼容版 (build_router stub-auth 预览骨架,
/// 保持 Phase I Batch 2 的无鉴权行为; 生产流量一律走 build_scm_router)
pub async fn create_compat(
    Json(body): Json<CreateBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    create_impl(service(), ActorContext::default(), body).await
}

async fn create_impl(
    svc: &InMemoryScmService,
    actor: ActorContext,
    body: CreateBody,
) -> Result<Json<RestResponse<Value>>, RestError> {
    if body.title.is_empty() {
        return Err(RestError::validation(
            "title is required".to_string(),
            "Provide a non-empty title",
        ));
    }
    if body.base.is_empty() || body.head.is_empty() {
        return Err(RestError::validation(
            "base and head branches are required".to_string(),
            "Provide non-empty base and head",
        ));
    }
    let repository_uuid = Uuid::parse_str(&body.repository_id).map_err(|e| {
        RestError::validation(
            format!("invalid repository_id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    let repository_id = RepositoryId::from(repository_uuid);

    let input = CreateMRInput {
        tenant_id: TenantId::from(actor.tenant_id),
        repository_id,
        title: body.title,
        description: body.description,
        base: body.base,
        head: body.head,
    };
    let pr = svc.create_mr(input, actor).await?;
    Ok(Json(RestResponse::ok(json!({ "mr": pr_json(&pr) }))))
}

fn pr_json(pr: &domain_scm::PullRequest) -> Value {
    json!({
        "id": pr.id.to_string(),
        "title": pr.title,
        "description": pr.description,
        "state": pr.state.as_str(),
        "source_branch": pr.source_branch,
        "target_branch": pr.target_branch,
        "mergeable": pr.mergeable,
        "repository_id": pr.repository_id.to_string(),
        "external_id": pr.external_id,
        "author_user_id": pr.author_user_id.to_string(),
        "created_at": pr.created_at.to_rfc3339(),
        "updated_at": pr.updated_at.to_rfc3339(),
        "lock_version": pr.lock_version,
    })
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// 可选: 仅列该仓库的 MR
    pub repository_id: Option<String>,
}

/// `GET /api/v1/merge-requests?repository_id=<uuid>` — 生产版
///
/// INV-SCM-04: 带 repository_id 先做 project-scoped 校验 (不可见 → 422 SCM_NOT_FOUND);
/// 无参列表仅返回 actor 可访问 project 的仓库下的 MR。
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
    // 可见仓库集合 = tenant 过滤 + INV-SCM-04 project 成员资格 (domain helper)
    let visible = svc.list_repositories(actor.clone()).await;
    let repo_filter = match params.repository_id {
        Some(raw) => {
            let uuid = Uuid::parse_str(&raw).map_err(|e| {
                RestError::validation(
                    format!("invalid repository_id UUID: {e}"),
                    "Provide a valid UUID",
                )
            })?;
            let id = RepositoryId::from(uuid);
            if !visible.iter().any(|r| r.id == id) {
                return Err(RestError::from(domain_scm::ScmError::NotFound(id)));
            }
            Some(id)
        }
        None => None,
    };
    let mut mrs = svc.list_pull_requests(actor).await;
    mrs.retain(|pr| match repo_filter {
        Some(id) => pr.repository_id == id,
        None => visible.iter().any(|r| r.id == pr.repository_id),
    });
    let items: Vec<Value> = mrs.iter().map(pr_json).collect();
    Ok(Json(RestResponse::ok(json!({ "merge_requests": items }))))
}

#[derive(Debug, Deserialize)]
pub struct TransitionBody {
    /// 目标状态 (domain 状态机白名单裁决, e.g. OPEN / MERGED / CLOSED)
    pub to_state: String,
}

/// `POST /api/v1/merge-requests/:id/transition` — 生产版
pub async fn transition(
    State(state): State<ScmApiState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(body): Json<TransitionBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    transition_impl(&state.service, actor_from_auth(&auth), id, body).await
}

async fn transition_impl(
    svc: &InMemoryScmService,
    actor: ActorContext,
    id: Uuid,
    body: TransitionBody,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let new_state = match body.to_state.as_str() {
        "DRAFT" | "Draft" => PullRequestState::Draft,
        "OPEN" | "Open" => PullRequestState::Open,
        "REVIEWING" | "Reviewing" => PullRequestState::Reviewing,
        "CHANGES_REQUESTED" | "ChangesRequested" => PullRequestState::ChangesRequested,
        "APPROVED" | "Approved" => PullRequestState::Approved,
        "MERGEABLE" | "Mergeable" => PullRequestState::Mergeable,
        "MERGED" | "Merged" => PullRequestState::Merged,
        "CLOSED" | "Closed" => PullRequestState::Closed,
        other => return Err(RestError::validation(
            format!("unknown to_state: {other}"),
            "Use one of DRAFT/OPEN/REVIEWING/CHANGES_REQUESTED/APPROVED/MERGEABLE/MERGED/CLOSED",
        )),
    };
    let cmd = RecordPullRequestTransitionCommand {
        pr_id: PullRequestId::from_uuid(id),
        new_state,
        actor,
    };
    let pr = svc.transition_pull_request(cmd).await?;
    Ok(Json(RestResponse::ok(json!({ "mr": pr_json(&pr) }))))
}

/// 生产 router: 共享 ScmApiState + 真实身份 (由 build_supplemental_routers 挂载)
pub(crate) fn router(state: ScmApiState) -> Router {
    Router::new()
        .route("/api/v1/merge-requests", post(create).get(list))
        .route("/api/v1/merge-requests/{id}/transition", post(transition))
        .with_state(state)
}
