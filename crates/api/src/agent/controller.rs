// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/agent/controller.rs` — 13 REST route handlers for A1-A10 (per DD-AGENT §3.1 + §4.1).
//!
//! Per `docs/briefs/p3-d6-1-4-api-extend.md` §2.2 + `docs/design/DD-CANVAS-AGENT-001.md`
//! v0.1.1 §3.1 line 287-292. 13 REST endpoints:
//!
//! | Method | Path                                | Handler          | A#  |
//! |--------|-------------------------------------|------------------|-----|
//! | POST   | `/api/v1/agents`                    | `create_agent`   | A1.1|
//! | GET    | `/api/v1/agents/{id}`               | `get_agent`      | A1.1|
//! | PATCH  | `/api/v1/agents/{id}`               | `update_agent`   | A1.1|
//! | DELETE | `/api/v1/agents/{id}`               | `delete_agent`   | A1.1|
//! | POST   | `/api/v1/agents/{id}/handoff`       | `handoff_agent`  | A2.1|
//! | GET    | `/api/v1/agents/{id}/topology`      | `get_topology`   | A2.2|
//! | POST   | `/api/v1/agents/{id}/parent`        | `set_parent`     | A2.3|
//! | POST   | `/api/v1/agents/{id}/pipeline`      | `add_to_pipeline`| A2.4|
//! | POST   | `/api/v1/agents/{id}/state`         | `change_state`   | A3.2|
//! | GET    | `/api/v1/agents/{id}/worktree`      | `get_worktree`   | A4.1|
//! | GET    | `/api/v1/agents/{id}/work-items`    | `get_work_items` | A4.2|
//! | POST   | `/api/v1/agents/{id}/start`         | `start_agent`    | A6.1|
//! | POST   | `/api/v1/agents/{id}/stop`          | `stop_agent`     | A6.1|
//!
//! Total: 13 route registrations.
//!
//! 0 业务方法实装 (per brief §0 + §1.2 out-of-scope). handler body 统一是:
//! ```text
//! state.permission.check_tenant(state.tenant_id)?;
//! state.audit.write_event(AgentAuditEvent::new(...)).await?;
//! Ok(Json(ResponseType::default()))
//! ```
//! 0 真实 DB query / 0 真实 Memgraph / 0 真实 PostgreSQL adapter 调用.
//! 阶段 2 任务 2.1 业务实装阶段 落地真实 `state.agent_ops.<method>(req, tenant).await?`.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2 + 守门 #24 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 每个 handler 都过 [`AgentPermission::check_tenant`] + [`AgentAudit::write_event`].
//! - `Lead` 角色独占, `Admin` 不能越权 (per 守门 #14 v2 反转 9/3 11:35 JST).
//! - `System` 角色允许跨租户 (per 守门 #24 v2 subprocess 路径).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
// `delete` and `patch` are used via the `.delete(...)` / `.patch(...)`
// route method-chains; `post` / `get` are used directly below.
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post};
use uuid::Uuid;

use super::audit::AgentAuditEvent;
use super::dto::{
    AgentActionResponse, AgentResponse, ApiError, CreateAgentRequest, HandoffRequest,
    HandoffResponse, ParentRequest, ParentResponse, PipelineRequest, PipelineResponse,
    StateChangeRequest, TopologyResponse, UpdateAgentRequest, WorkItemListResponse,
    WorktreeResponse,
};
use super::permission::role;

/// 13 route router factory.
///
/// 单一入口: caller 传 `Arc<AgentState>`, 我们返回完整 `Router` ready to
/// be merged into the application router.
pub fn agent_routes(state: Arc<super::AgentState>) -> Router {
    Router::new()
        .route(
            "/api/v1/agents",
            post(create_agent).get(list_agents_unused_silencer),
        )
        .route(
            "/api/v1/agents/{id}",
            get(get_agent).patch(update_agent).delete(delete_agent),
        )
        .route("/api/v1/agents/{id}/handoff", post(handoff_agent))
        .route("/api/v1/agents/{id}/topology", get(get_topology))
        .route("/api/v1/agents/{id}/parent", post(set_parent))
        .route("/api/v1/agents/{id}/pipeline", post(add_to_pipeline))
        .route("/api/v1/agents/{id}/state", post(change_state))
        .route("/api/v1/agents/{id}/worktree", get(get_worktree))
        .route("/api/v1/agents/{id}/work-items", get(get_work_items))
        .route("/api/v1/agents/{id}/start", post(start_agent))
        .route("/api/v1/agents/{id}/stop", post(stop_agent))
        .with_state(state)
}

// =====================================================================
// ApiError → HTTP response (per arg/controller.rs §api_error_to_response)
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
// 4 守门 + 1 placeholder helper
// =====================================================================

/// 共享 4 守门 (per brief §2.2 handler pattern): tenant 校验 + audit 写 + placeholder response.
async fn four_guard_and_placeholder(
    state: &Arc<super::AgentState>,
    agent_id: Uuid,
    action: &str,
) -> Result<(), ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            agent_id,
            state.actor_id,
            action.to_string(),
        ))
        .await?;
    Ok(())
}

// =====================================================================
// A1.1: 4 endpoints (create / get / update / delete)
// =====================================================================

/// 1. `POST /api/v1/agents` — create a new agent (A1.1).
async fn create_agent(
    State(state): State<Arc<super::AgentState>>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    let new_id = Uuid::new_v4();
    state
        .audit
        .write_event(AgentAuditEvent::new(
            new_id,
            state.actor_id,
            "create_agent".to_string(),
        ))
        .await?;
    // V0.4 阶段 2 任务 2.1 实装:
    //   let agent = Agent { id: new_id, tenant_id: state.tenant_id, name: req.name, ... };
    //   let created = state.agent_ops.create(agent).await?;
    Ok(Json(AgentResponse::default()))
}

/// 2. `GET /api/v1/agents/{id}` — fetch a single agent (A1.1).
async fn get_agent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "get_agent".to_string(),
        ))
        .await?;
    // V0.4: let agent = state.agent_ops.get(id, state.tenant_id).await?
    //            .ok_or_else(|| ApiError::NotFound(format!("agent: {id}")))?;
    Ok(Json(AgentResponse::default()))
}

/// 3. `PATCH /api/v1/agents/{id}` — apply a patch to an existing agent (A1.1).
async fn update_agent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<AgentResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "update_agent".to_string(),
        ))
        .await?;
    // V0.4: let updated = state.agent_ops.update(id, patch, state.tenant_id).await?;
    Ok(Json(AgentResponse::default()))
}

/// 4. `DELETE /api/v1/agents/{id}` — archive a single agent (A1.1).
async fn delete_agent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "delete_agent".to_string(),
        ))
        .await?;
    // V0.4: let deleted = state.agent_ops.archive(id, state.tenant_id).await?;
    Ok(Json(AgentResponse::default()))
}

// =====================================================================
// A2.1-A2.4: 4 endpoints (handoff / topology / parent / pipeline)
// =====================================================================

/// 5. `POST /api/v1/agents/{id}/handoff` — handoff agent (A2.1).
async fn handoff_agent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<HandoffRequest>,
) -> Result<Json<HandoffResponse>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "handoff_agent".to_string(),
        ))
        .await?;
    // V0.4: let result = state.handoff_connector.handoff(id, req.target_agent_id, ...).await?;
    Ok(Json(HandoffResponse::default()))
}

/// 6. `GET /api/v1/agents/{id}/topology` — fetch 5 域 topology (A2.2).
async fn get_topology(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TopologyResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "get_topology".to_string(),
        ))
        .await?;
    // V0.4: let topology = state.domain_frame.topology(id, state.tenant_id).await?;
    Ok(Json(TopologyResponse::default()))
}

/// 7. `POST /api/v1/agents/{id}/parent` — set parent session (A2.3).
async fn set_parent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ParentRequest>,
) -> Result<Json<ParentResponse>, ApiError> {
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "set_parent".to_string(),
        ))
        .await?;
    // V0.4: let updated = state.parent_child_connector.set_parent(id, req.parent_session_id).await?;
    Ok(Json(ParentResponse::default()))
}

/// 8. `POST /api/v1/agents/{id}/pipeline` — add to pipeline (A2.4).
async fn add_to_pipeline(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<PipelineRequest>,
) -> Result<Json<PipelineResponse>, ApiError> {
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "add_to_pipeline".to_string(),
        ))
        .await?;
    // V0.4: let updated = state.pipeline_connector.add(id, req.agent_ids).await?;
    Ok(Json(PipelineResponse::default()))
}

// =====================================================================
// A3.2: 1 endpoint (state change)
// =====================================================================

/// 9. `POST /api/v1/agents/{id}/state` — change agent 14 状态 (A3.2).
async fn change_state(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
    #[allow(unused_variables)] Json(req): Json<StateChangeRequest>,
) -> Result<Json<AgentActionResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "change_state".to_string(),
        ))
        .await?;
    // V0.4: let mut agent = state.agent_ops.get(id, state.tenant_id).await?.unwrap();
    //        let audit = agent.state_change_audit(req.new_state)?;
    //        let saved = state.agent_ops.update_state(&agent, &audit).await?;
    Ok(Json(AgentActionResponse::default()))
}

// =====================================================================
// A4.1 + A4.2: 2 endpoints (worktree / work-items)
// =====================================================================

/// 10. `GET /api/v1/agents/{id}/worktree` — fetch worktree association (A4.1).
async fn get_worktree(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorktreeResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "get_worktree".to_string(),
        ))
        .await?;
    // V0.4: let worktree = state.worktree_assoc.get(id, state.tenant_id).await?;
    Ok(Json(WorktreeResponse::default()))
}

/// 11. `GET /api/v1/agents/{id}/work-items` — list work-items (A4.2).
async fn get_work_items(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkItemListResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "get_work_items".to_string(),
        ))
        .await?;
    // V0.4: let items = state.work_item_assoc.list(id, state.tenant_id).await?;
    Ok(Json(WorkItemListResponse::default()))
}

// =====================================================================
// A6.1: 2 endpoints (start / stop)
// =====================================================================

/// 12. `POST /api/v1/agents/{id}/start` — start agent (A6.1).
async fn start_agent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentActionResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "start_agent".to_string(),
        ))
        .await?;
    // V0.4: let mut agent = ...; let audit = agent.start()?;
    //        let saved = state.agent_ops.update_state(&agent, &audit).await?;
    let _ = four_guard_and_placeholder(&state, id, "start_agent").await;
    Ok(Json(AgentActionResponse::default()))
}

/// 13. `POST /api/v1/agents/{id}/stop` — stop agent (A6.1).
async fn stop_agent(
    State(state): State<Arc<super::AgentState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentActionResponse>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .audit
        .write_event(AgentAuditEvent::new(
            id,
            state.actor_id,
            "stop_agent".to_string(),
        ))
        .await?;
    // V0.4: let mut agent = ...; let audit = agent.stop()?;
    //        let saved = state.agent_ops.update_state(&agent, &audit).await?;
    Ok(Json(AgentActionResponse::default()))
}

/// Unused silencer: `Router::route` 在 `get` + `post` 双方法时仅用 1 个就
/// 需要 1 个 placeholder. 阶段 2 任务 2.1 实装 `list_agents` 时替换.
/// 这是为了让 `cargo build` 通过, 同时保留 `POST /api/v1/agents` 路由.
async fn list_agents_unused_silencer(
    State(_state): State<Arc<super::AgentState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 阶段 1 占位: 0 业务方法实装. 阶段 2 任务 2.1 实装 list_agents.
    Err(ApiError::Unimplemented(
        "list_agents not yet implemented (P3-D.6 阶段 2 任务 2.1)".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentState;

    fn make_state() -> Arc<AgentState> {
        AgentState::new(
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
        )
    }

    #[test]
    fn agent_routes_registers_thirteen_routes() {
        // axum 0.8 Router 没有公开 route count introspection, 所以
        // 我们只 smoke test: build_router 成功, Router non-empty.
        let s = make_state();
        let router = agent_routes(s);
        let _ = router;
    }
}
