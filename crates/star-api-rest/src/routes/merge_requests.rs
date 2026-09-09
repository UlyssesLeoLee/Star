// SPDX-License-Identifier: MIT OR Apache-2.0
//! merge_request 路由真实接线 (per Phase I Batch 2)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #14):
//! - `create` ← `star-mcp/src/tools/create_merge_request.rs::invoke`

use std::sync::{Arc, OnceLock};

use axum::Json;
use domain_scm::{ActorContext, CreateMRInput, InMemoryScmService, RepositoryId, TenantId};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory SCM service
pub(crate) fn service() -> &'static Arc<InMemoryScmService> {
    static SVC: OnceLock<Arc<InMemoryScmService>> = OnceLock::new();
    SVC.get_or_init(|| InMemoryScmService::new_for_test())
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

/// `POST /api/v1/merge-requests`
pub async fn create(Json(body): Json<CreateBody>) -> Result<Json<RestResponse<Value>>, RestError> {
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

    let actor = ActorContext::default().with_role("project_admin");
    let input = CreateMRInput {
        tenant_id: TenantId::from(actor.tenant_id),
        repository_id,
        title: body.title,
        description: body.description,
        base: body.base,
        head: body.head,
    };
    let pr = service().create_mr(input, actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "mr": {
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
        }
    }))))
}
