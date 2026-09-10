// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/collaboration/controller.rs` — 5 REST + 4 WSS route handlers for A12 (per DD-AGENT §3.1 + §4.14 + brief §2.4 + §2.5).
//!
//! Per `docs/briefs/p3-d6-1-5-bff-skeleton.md` §2.4 + §2.5. 5 REST endpoints + 4 WSS endpoints:
//!
//! | Method | Path                                          | Handler           | A#    |
//! |--------|-----------------------------------------------|-------------------|-------|
//! | POST   | `/bff/v1/canvas/elements`                     | `create_element`  | A12.1 |
//! | PATCH  | `/bff/v1/canvas/elements/{id}`                | `update_element`  | A12.3 |
//! | POST   | `/bff/v1/canvas/elements/{id}/comments`       | `add_comment`     | A12.5 |
//! | GET    | `/bff/v1/canvas/{id}/presence`                | `get_presence`    | A12.2 |
//! | POST   | `/bff/v1/canvas/{id}/follow`                  | `set_follow`      | A12.4 |
//! | GET    | `/bff/v1/ws/canvas/elements`                  | `wss_canvas_elements`   | A12.1 |
//! | GET    | `/bff/v1/ws/canvas/presence`                  | `wss_canvas_presence`   | A12.2 |
//! | GET    | `/bff/v1/ws/canvas/comments`                  | `wss_canvas_comments`   | A12.5 |
//! | GET    | `/bff/v1/ws/canvas/follow`                    | `wss_canvas_follow`     | A12.4 |
//!
//! Total: 9 route registrations (5 REST + 4 WSS).
//!
//! 0 业务方法实装 (per brief §0 + §1.2 out-of-scope). 0 CRDT 0 NATS 0 真实
//! broadcast subscribe (留 P3-D.6 阶段 2/3 续做). 阶段 2 任务 2.3 业务实装阶段
//! 落地真实 `state.canvas_ops.<method>(req, tenant).await?`.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2 + 守门 #24 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 每个 handler 都过 [`CollaborationPermission::check_tenant`] +
//!   [`CollaborationPermission::require`] (3 档权限) + [`CollaborationAudit::write_event`].
//! - 平台管理员 (`Admin` / `System`) 跨租户访问 (per 守门 #24 v2 subprocess 路径).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use uuid::Uuid;

use super::audit::CollaborationAuditEvent;
use super::dto::{
    AddCommentRequest, ApiError, CommentResponse, CreateElementRequest, ElementResponse,
    FollowResponse, PresenceListResponse, SetFollowRequest, UpdateElementRequest,
};
use super::permission::{role, PermissionLevel};
use super::wss_hub::{
    wss_canvas_comments, wss_canvas_elements, wss_canvas_follow, wss_canvas_presence,
};

// =====================================================================
// 9 route router factory (5 REST + 4 WSS)
// =====================================================================

/// 9 route router factory (5 REST + 4 WSS, per brief §2.4 + §2.5).
///
/// 单一入口: caller 传 `Arc<CollaborationState>`, 我们返回完整 `Router` ready to
/// be merged into the application router (per envoy 独立 deployment per
/// 9/1 13:05 JST 偏好, 业务 svc 通过 `svc://bff` 引用).
pub fn collaboration_routes(state: Arc<super::CollaborationState>) -> Router {
    // 5 REST 路由 (per brief §2.4 表格 1-5)
    let rest = Router::new()
        .route("/bff/v1/canvas/elements", post(create_element))
        .route("/bff/v1/canvas/elements/{id}", patch(update_element))
        .route("/bff/v1/canvas/elements/{id}/comments", post(add_comment))
        .route("/bff/v1/canvas/{id}/presence", get(get_presence))
        .route("/bff/v1/canvas/{id}/follow", post(set_follow))
        .with_state(state.clone());

    // 4 WSS 路由 (per brief §2.5 表格 1-4)
    let wss = Router::new()
        .route("/bff/v1/ws/canvas/elements", get(wss_canvas_elements))
        .route("/bff/v1/ws/canvas/presence", get(wss_canvas_presence))
        .route("/bff/v1/ws/canvas/comments", get(wss_canvas_comments))
        .route("/bff/v1/ws/canvas/follow", get(wss_canvas_follow))
        .with_state(state);

    rest.merge(wss)
}

// =====================================================================
// ApiError → HTTP response
// =====================================================================

/// Convert [`ApiError`] into an axum response. Mirrors the helper in
/// `crates/api/src/canvas_collab/controller.rs` (per 守门 #19 v19 累积规,
/// BFF 跟 API tier 同形但 0 跨层 leak).
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

/// 1. `POST /bff/v1/canvas/elements` — create a new canvas element (A12.1).
async fn create_element(
    State(state): State<Arc<super::CollaborationState>>,
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
        .write_event(CollaborationAuditEvent::new(
            req.canvas_id,
            state.actor_id,
            "create_element".to_string(),
            serde_json::json!({"element_id": new_id, "kind": req.kind}),
        ))
        .await?;
    // V0.2 阶段 2 任务 2.3 实装: let element = CanvasElementBackend { id: new_id, ... };
    //        let created = state.canvas_ops.create_element(element).await?;
    Ok(Json(ElementResponse::default()))
}

// =====================================================================
// A12.3: update element
// =====================================================================

/// 2. `PATCH /bff/v1/canvas/elements/{id}` — update an existing element (A12.3).
async fn update_element(
    State(state): State<Arc<super::CollaborationState>>,
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
        .write_event(CollaborationAuditEvent::new(
            state.canvas_id,
            state.actor_id,
            "update_element".to_string(),
            serde_json::json!({"element_id": _id, "patch": req}),
        ))
        .await?;
    // V0.2 阶段 2 任务 2.3 实装: let updated = state.canvas_ops.update_element(_id, patch, state.tenant_id).await?;
    Ok(Json(ElementResponse::default()))
}

// =====================================================================
// A12.5: add comment
// =====================================================================

/// 3. `POST /bff/v1/canvas/elements/{id}/comments` — add a comment (A12.5).
async fn add_comment(
    State(state): State<Arc<super::CollaborationState>>,
    Path(element_id): Path<Uuid>,
    Json(req): Json<AddCommentRequest>,
) -> Result<Json<CommentResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    // 评论权限: 至少 `Comment` 等级 (per A12.7 派生).
    state.permission.require(PermissionLevel::Comment)?;
    state
        .audit
        .write_event(CollaborationAuditEvent::new(
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
    // V0.2 阶段 2 任务 2.3 实装: let comment = CanvasComment { ... };
    //        let created = state.canvas_ops.add_comment(comment).await?;
    Ok(Json(CommentResponse::default()))
}

// =====================================================================
// A12.2: get presence
// =====================================================================

/// 4. `GET /bff/v1/canvas/{id}/presence` — list active cursors (A12.2).
async fn get_presence(
    State(state): State<Arc<super::CollaborationState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<PresenceListResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state.permission.require(PermissionLevel::View)?;
    state
        .audit
        .write_event(CollaborationAuditEvent::new(
            canvas_id,
            state.actor_id,
            "get_presence".to_string(),
            serde_json::json!({}),
        ))
        .await?;
    // V0.2 阶段 2 任务 2.3 实装: let cursors = state.canvas_ops.presence_list(canvas_id, state.tenant_id).await?;
    Ok(Json(PresenceListResponse::default()))
}

// =====================================================================
// A12.4: set follow
// =====================================================================

/// 5. `POST /bff/v1/canvas/{id}/follow` — start/stop follow mode (A12.4).
async fn set_follow(
    State(state): State<Arc<super::CollaborationState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<SetFollowRequest>,
) -> Result<Json<FollowResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    // Follow 模式: 至少 `View` 等级 (per A12.4 派生).
    state.permission.require(PermissionLevel::View)?;
    let action = if req.enabled {
        "follow.start"
    } else {
        "follow.stop"
    };
    state
        .audit
        .write_event(CollaborationAuditEvent::new(
            canvas_id,
            state.actor_id,
            action.to_string(),
            serde_json::json!({
                "followee_id": req.followee_id,
                "enabled": req.enabled,
            }),
        ))
        .await?;
    // V0.2 阶段 2 任务 2.3 实装: state.canvas_ops.set_follow(canvas_id, state.actor_id, req.followee_id, req.enabled).await?;
    Ok(Json(FollowResponse {
        canvas_id,
        follower_id: state.actor_id,
        followee_id: req.followee_id,
        enabled: req.enabled,
        at: chrono::Utc::now(),
    }))
}

#[cfg(test)]
mod tests {
    use super::super::CollaborationState;
    use super::*;
    use crate::collaboration::audit::CollaborationAudit;
    use crate::collaboration::permission::{CollaborationPermission, PermissionLevel};
    use crate::collaboration::wss_hub::CollaborationWssHub;
    use std::collections::HashSet;

    fn make_state() -> Arc<CollaborationState> {
        let mut roles = HashSet::new();
        roles.insert(role::LEAD.to_string());
        let permission = Arc::new(CollaborationPermission::new(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            roles,
            PermissionLevel::Edit,
        ));
        let audit = Arc::new(CollaborationAudit::new());
        let wss_hub = Arc::new(CollaborationWssHub::new());
        CollaborationState::new(
            audit,
            permission,
            wss_hub,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap(),
        )
    }

    #[test]
    fn collaboration_routes_registers_nine_routes() {
        // axum 0.8 Router 没有公开 route count introspection, 所以
        // 我们只 smoke test: build_router 成功, Router non-empty.
        let s = make_state();
        let router = collaboration_routes(s);
        let _ = router;
    }
}
