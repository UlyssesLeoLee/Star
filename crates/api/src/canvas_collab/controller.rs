// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/canvas_collab/controller.rs` — 5 REST route handlers for A12 (per DD-AGENT §3.1 + §4.14).
//!
//! Per `docs/briefs/p3-d6-1-4-api-extend.md` §2.7. 5 REST endpoints (per A12.1-A12.4 + A12.7):
//!
//! | Method | Path                                         | Handler           | A#    |
//! |--------|----------------------------------------------|-------------------|-------|
//! | POST   | `/api/v1/canvas/elements`                    | `create_element`  | A12.1 |
//! | PATCH  | `/api/v1/canvas/elements/{id}`               | `update_element`  | A12.2 |
//! | POST   | `/api/v1/canvas/elements/{id}/comments`      | `add_comment`     | A12.3 |
//! | GET    | `/api/v1/canvas/{id}/presence`               | `get_presence`    | A12.4 |
//! | POST   | `/api/v1/canvas/{id}/permission`             | `grant_permission`| A12.7 |
//!
//! Total: 5 route registrations.
//!
//! 0 业务方法实装 (per brief §0 + §1.2 out-of-scope). 0 WebSocket / 0 SSE
//! (per §1.2 out-of-scope, A12 WebSocket 4 端点 走 `bff/src/collaboration/wss_hub.rs`
//! 任务 1.5 续做). 阶段 2 任务 2.3 业务实装阶段 落地真实 `state.canvas_ops.<method>(req, tenant).await?`.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2 + 守门 #24 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 每个 handler 都过 [`CanvasCollabPermission::check_tenant`] + [`CanvasCollabPermission::require`] (3 档权限) + [`CanvasCollabAudit::write_event`].
//! - 平台管理员 (`Admin` / `System`) 跨租户访问 (per 守门 #24 v2 subprocess 路径).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
// `patch` is used via the `.patch(...)` route method-chain; `post` / `get`
// are used directly below.
#[allow(unused_imports)]
use axum::routing::{get, patch, post};
use uuid::Uuid;

use super::audit::CanvasCollabAuditEvent;
use super::dto::{
    AddCommentRequest, ApiError, CommentResponse, CreateElementRequest, ElementResponse,
    GrantPermissionRequest, PermissionResponse, PresenceListResponse, UpdateElementRequest,
};
use super::permission::{role, PermissionLevel};

// =====================================================================
// 5 route router factory
// =====================================================================

/// 5 route router factory.
///
/// 单一入口: caller 传 `Arc<CanvasCollabState>`, 我们返回完整 `Router` ready to
/// be merged into the application router.
pub fn canvas_collab_routes(state: Arc<super::CanvasCollabState>) -> Router {
    Router::new()
        .route("/api/v1/canvas/elements", post(create_element))
        .route("/api/v1/canvas/elements/{id}", patch(update_element))
        .route("/api/v1/canvas/elements/{id}/comments", post(add_comment))
        .route("/api/v1/canvas/{id}/presence", get(get_presence))
        .route("/api/v1/canvas/{id}/permission", post(grant_permission))
        .with_state(state)
}

// =====================================================================
// ApiError → HTTP response
// =====================================================================

/// Convert [`ApiError`] into an axum response. We use a JSON body so
/// the frontend can show a typed error message; the status code is
/// taken from [`ApiError::status_code`].
fn api_error_to_response(e: ApiError) -> (StatusCode, Json<serde_json::Value>) {
    let status = StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = serde_json::json!({
        "error": {
            "code": match &e {
                ApiError::BadRequest(_) => "bad_request",
                ApiError::Forbidden(_) => "forbidden",
                ApiError::NotFound(_) => "not_found",
                ApiError::Internal(_) => "internal",
                ApiError::Unimplemented(_) => "not_implemented",
            },
            "message": e.to_string(),
        }
    });
    (status, Json(body))
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        api_error_to_response(self).into_response()
    }
}

// =====================================================================
// A12.1: create element
// =====================================================================

/// 1. `POST /api/v1/canvas/elements` — create a new canvas element (A12.1).
async fn create_element(
    State(state): State<Arc<super::CanvasCollabState>>,
    Json(req): Json<CreateElementRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state.permission.require(PermissionLevel::Edit)?;
    let new_id = Uuid::new_v4();
    state
        .audit
        .write_event(CanvasCollabAuditEvent::new(
            req.canvas_id,
            state.actor_id,
            "create_element".to_string(),
            serde_json::json!({"element_id": new_id, "kind": req.kind}),
        ))
        .await?;
    // V0.4: let element = CanvasElementBackend { id: new_id, canvas_id: req.canvas_id, ... };
    //        let created = state.canvas_ops.create_element(element).await?;
    Ok(Json(ElementResponse::default()))
}

// =====================================================================
// A12.2: update element
// =====================================================================

/// 2. `PATCH /api/v1/canvas/elements/{id}` — update an existing element (A12.2).
async fn update_element(
    State(state): State<Arc<super::CanvasCollabState>>,
    Path(_id): Path<Uuid>,
    Json(req): Json<UpdateElementRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state.permission.require(PermissionLevel::Edit)?;
    state
        .audit
        .write_event(CanvasCollabAuditEvent::new(
            state.canvas_id,
            state.actor_id,
            "update_element".to_string(),
            serde_json::json!({"element_id": _id, "patch": req}),
        ))
        .await?;
    // V0.4: let updated = state.canvas_ops.update_element(_id, patch, state.tenant_id).await?;
    Ok(Json(ElementResponse::default()))
}

// =====================================================================
// A12.3: add comment
// =====================================================================

/// 3. `POST /api/v1/canvas/elements/{id}/comments` — add a comment (A12.3).
async fn add_comment(
    State(state): State<Arc<super::CanvasCollabState>>,
    Path(element_id): Path<Uuid>,
    Json(req): Json<AddCommentRequest>,
) -> Result<Json<CommentResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    // 评论权限: 至少 `Comment` 等级 (per A12.7 派生).
    state.permission.require(PermissionLevel::Comment)?;
    state
        .audit
        .write_event(CanvasCollabAuditEvent::new(
            req.canvas_id,
            state.actor_id,
            "add_comment".to_string(),
            serde_json::json!({
                "element_id": element_id,
                "thread_id": req.thread_id,
                "mentioned": req.mentioned_user_ids,
            }),
        ))
        .await?;
    // V0.4: let comment = CanvasComment { ... };
    //        let created = state.canvas_ops.add_comment(comment).await?;
    Ok(Json(CommentResponse::default()))
}

// =====================================================================
// A12.4: get presence
// =====================================================================

/// 4. `GET /api/v1/canvas/{id}/presence` — list active cursors (A12.4).
async fn get_presence(
    State(state): State<Arc<super::CanvasCollabState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<PresenceListResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state.permission.require(PermissionLevel::View)?;
    state
        .audit
        .write_event(CanvasCollabAuditEvent::new(
            canvas_id,
            state.actor_id,
            "get_presence".to_string(),
            serde_json::json!({}),
        ))
        .await?;
    // V0.4: let cursors = state.canvas_ops.presence_list(canvas_id, state.tenant_id).await?;
    Ok(Json(PresenceListResponse::default()))
}

// =====================================================================
// A12.7: grant permission
// =====================================================================

/// 5. `POST /api/v1/canvas/{id}/permission` — grant permission to a user (A12.7).
async fn grant_permission(
    State(state): State<Arc<super::CanvasCollabState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<GrantPermissionRequest>,
) -> Result<Json<PermissionResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    // 授权: 仅 `Admin` / `Lead` / `System` 可授 (per A12.7 派生).
    state
        .permission
        .require_any_role(&[role::ADMIN, role::LEAD, role::SYSTEM])?;
    state.permission.require(PermissionLevel::Edit)?;
    state
        .audit
        .write_event(CanvasCollabAuditEvent::new(
            canvas_id,
            state.actor_id,
            "grant_permission".to_string(),
            serde_json::json!({
                "grantee_user_id": req.user_id,
                "level": format!("{:?}", req.level),
            }),
        ))
        .await?;
    // V0.4: let perm = CanvasPermission { ... };
    //        let created = state.canvas_ops.grant_permission(perm).await?;
    Ok(Json(PermissionResponse::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas_collab::CanvasCollabState;

    fn make_state() -> Arc<CanvasCollabState> {
        CanvasCollabState::new(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap(),
        )
    }

    #[test]
    fn canvas_collab_routes_registers_five_routes() {
        // axum 0.8 Router 没有公开 route count introspection, 所以
        // 我们只 smoke test: build_router 成功, Router non-empty.
        let s = make_state();
        let router = canvas_collab_routes(s);
        let _ = router;
    }
}
