// SPDX-License-Identifier: MIT OR Apache-2.0
//! worktree 路由真实接线 (per Phase I Batch 1)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #7-8):
//! - `create`   ← `star-mcp/src/tools/create_worktree.rs::invoke`
//! - `get_by_id` ← `star-mcp/src/tools/get_worktree.rs::invoke`

use std::sync::{Arc, OnceLock};

use axum::{
    extract::Path,
    Json,
};
use domain_worktree::{
    ActorContext, CreateWorktreeCommand, InMemoryWorktreeService, ProjectId, RepositoryId,
    RuntimeId, TenantId, UserId, WorkItemId, WorktreeCommandPort, WorktreeId,
    WorktreeQueryPort,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory worktree service
fn service() -> &'static Arc<InMemoryWorktreeService> {
    static SVC: OnceLock<Arc<InMemoryWorktreeService>> = OnceLock::new();
    SVC.get_or_init(|| Arc::new(InMemoryWorktreeService::new()))
}

/// `GET /api/v1/worktrees/{id}`
pub async fn get_by_id(
    Path(id): Path<String>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let uuid = Uuid::parse_str(&id).map_err(|e| {
        RestError::validation(
            format!("invalid worktree_id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    let actor = ActorContext::default().with_role("developer");
    let wt = service()
        .get_by_id(WorktreeId::from(uuid), &actor)
        .await?;
    Ok(Json(RestResponse::ok(json!({
        "worktree": {
            "id": wt.id.to_string(),
            "tenant_id": wt.tenant_id.to_string(),
            "project_id": wt.project_id.to_string(),
            "repository_id": wt.repository_id.to_string(),
            "work_item_id": wt.work_item_id.to_string(),
            "runtime_id": wt.runtime_id.to_string(),
            "branch": wt.branch,
            "base_branch": wt.base_branch,
            "status": wt.status.as_str(),
            "owner_user_id": wt.owner_user_id.to_string(),
            "version": wt.version,
            "created_at": wt.created_at.to_rfc3339(),
            "updated_at": wt.updated_at.to_rfc3339(),
        }
    }))))
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    /// 关联的 WorkItem ID (UUID) — 必填
    pub work_item_id: String,
    /// 分支名 — 可选, 默认 `feature/{work_item_id}`
    pub branch_name: Option<String>,
    /// Agent session ID — 可选 (P0 简化: 仅记录, 不绑定)
    pub agent_session_id: Option<String>,
}

/// `POST /api/v1/worktrees`
///
/// 接受 JSON body (work_item_id + branch_name + agent_session_id), 跟 star-mcp `create_worktree` 范式一致.
/// query param `?issue_id=...` 兼容性: 客户端可直接传 work_item_id 字段 (跟 MCP issue_id 兼容).
pub async fn create(
    Json(body): Json<CreateBody>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    let issue_id = body.work_item_id;
    let branch = body
        .branch_name
        .unwrap_or_else(|| format!("feature/{issue_id}"));

    let actor = ActorContext::default().with_role("developer");
    let tenant_id = TenantId::from(actor.tenant_id);
    let project_id = ProjectId::new();
    let work_item_uuid =
        Uuid::parse_str(&issue_id).unwrap_or_else(|_| Uuid::new_v4());
    let work_item_id = WorkItemId::from(work_item_uuid);
    let repository_id = RepositoryId::new();
    let runtime_id = RuntimeId::new();

    let cmd = CreateWorktreeCommand {
        tenant_id,
        project_id,
        work_item_id,
        repository_id,
        branch: branch.clone(),
        base_branch: "main".to_string(),
        runtime_id,
        owner_user_id: UserId::from(actor.user_id),
    };
    let wt = service().create_worktree(cmd, &actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "worktree": {
            "id": wt.id.to_string(),
            "tenant_id": wt.tenant_id.to_string(),
            "project_id": wt.project_id.to_string(),
            "repository_id": wt.repository_id.to_string(),
            "work_item_id": wt.work_item_id.to_string(),
            "runtime_id": wt.runtime_id.to_string(),
            "branch": wt.branch,
            "base_branch": wt.base_branch,
            "status": wt.status.as_str(),
            "owner_user_id": wt.owner_user_id.to_string(),
            "version": wt.version,
            "created_at": wt.created_at.to_rfc3339(),
            "updated_at": wt.updated_at.to_rfc3339(),
        }
    }))))
}
