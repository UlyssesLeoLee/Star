// SPDX-License-Identifier: MIT OR Apache-2.0
//! workspace 路由真实接线 (per Phase I Batch 1)
//!
//! 范式来源 (per `STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.1 表 #6):
//! - `get_by_id` ← `star-mcp/src/tools/get_workspace.rs::invoke`

use std::sync::{Arc, OnceLock};

use axum::{extract::Path, Json};
use domain_workspace::{ActorContext, InMemoryWorkspaceService, WorkspaceId, WorkspaceQueryPort};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::RestError;
use crate::response::RestResponse;

/// 全 handler 共享的 in-memory workspace service
pub(crate) fn service() -> &'static Arc<InMemoryWorkspaceService> {
    static SVC: OnceLock<Arc<InMemoryWorkspaceService>> = OnceLock::new();
    SVC.get_or_init(InMemoryWorkspaceService::new_for_test)
}

/// `GET /api/v1/workspaces/{id}`
pub async fn get_by_id(Path(id): Path<String>) -> Result<Json<RestResponse<Value>>, RestError> {
    let uuid = Uuid::parse_str(&id).map_err(|e| {
        RestError::validation(
            format!("invalid workspace_id UUID: {e}"),
            "Provide a valid UUID",
        )
    })?;
    // nil-tenant actor 走 service.get_by_id → 跨 tenant 拒绝 → 404 (跟 star-mcp `get_workspace` 简化模式)
    let actor = ActorContext::default();
    let ws = service().get_by_id(WorkspaceId::from(uuid), actor).await?;
    Ok(Json(RestResponse::ok(json!({
        "workspace": {
            "id": ws.id.to_string(),
            "tenant_id": ws.tenant_id.to_string(),
            "workspace_key": ws.workspace_key,
            "name": ws.name,
            "description": ws.description,
            "version": ws.version,
            "created_at": ws.created_at.to_rfc3339(),
            "updated_at": ws.updated_at.to_rfc3339(),
        }
    }))))
}
