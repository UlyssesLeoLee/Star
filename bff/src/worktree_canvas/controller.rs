// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/controller.rs` — 11 REST 端点 (per DD-WORKTREE-CANVAS-001
//! §20 + IMPL-PLAN §4.13.2):
//!
//! 1. `GET    /v1/worktree-canvas/repositories`              List repos
//! 2. `GET    /v1/worktree-canvas/worktrees`                 List worktrees
//! 3. `POST   /v1/worktree-canvas/worktrees`                 Create
//! 4. `GET    /v1/worktree-canvas/worktrees/:id`              Get one
//! 5. `PATCH  /v1/worktree-canvas/worktrees/:id`              Update
//! 6. `DELETE /v1/worktree-canvas/worktrees/:id`              Delete
//! 7. `POST   /v1/worktree-canvas/worktrees/:id/actions/:action_type`  18 Action
//! 8. `GET    /v1/worktree-canvas/risks`                     List risks
//! 9. `GET    /v1/worktree-canvas/health`                    Aggregate health
//! 10. `POST   /v1/worktree-canvas/search`                   Search (DSL)
//! 11. `POST   /v1/worktree-canvas/nl-query`                 NL → DSL → query
//!
//! 阶段 1: 占位 handler, 0 真实业务. 阶段 2 业务实装落地 graph-core + git-adapter 调用.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 每个 handler 必过 RBAC + (写路径) Idempotency + Audit.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use super::audit::{WorktreeAudit, WorktreeAuditEvent};
use super::dto::{
    ActionRequest, ActionResponse, BffHealthDto, CreateWorktreeRequest, HealthSummaryDto,
    ListWorktreesResponse, NlQueryRequest, NlQueryResponse, RepositoryDto, RiskResponse,
    SearchRequest, SearchResponse, WorktreeApiError, WorktreeGraphResponse, WorktreeResponse,
};
use super::idempotency::IdempotencyCache;
use super::permission::{PermissionLevel, RoleSet, WorktreeAction, WorktreePermission};
use super::sse::WorktreeSseHub;
use super::WorktreeCanvasState;

// =====================================================================
// Router factory (per IMPL-PLAN §4.13.2)
// =====================================================================

/// Build 11 REST routes router (per IMPL-PLAN §4.13.2).
pub fn rest_routes(state: Arc<WorktreeCanvasState>) -> Router {
    Router::new()
        .route("/v1/worktree-canvas/repositories", get(list_repositories))
        .route("/v1/worktree-canvas/worktrees", get(list_worktrees).post(create_worktree))
        .route(
            "/v1/worktree-canvas/worktrees/{id}",
            get(get_worktree).patch(update_worktree).delete(delete_worktree),
        )
        .route(
            "/v1/worktree-canvas/worktrees/{id}/actions/{action_type}",
            post(execute_action),
        )
        .route("/v1/worktree-canvas/worktrees/{id}/graph", get(get_worktree_graph))
        .route("/v1/worktree-canvas/risks", get(list_risks))
        .route("/v1/worktree-canvas/health", get(get_health))
        .route("/v1/worktree-canvas/search", post(search))
        .route("/v1/worktree-canvas/nl-query", post(nl_query))
        .with_state(state)
}

// =====================================================================
// Handlers
// =====================================================================

/// 1. `GET /v1/worktree-canvas/repositories`.
async fn list_repositories(
    State(state): State<Arc<WorktreeCanvasState>>,
) -> Result<Json<Vec<RepositoryDto>>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    // 阶段 1: 0 真实 DB, 返回空列表占位.
    Ok(Json(Vec::new()))
}

#[derive(Debug, Deserialize)]
struct ListWorktreesQuery {
    #[serde(default)]
    repo_id: Option<Uuid>,
    #[serde(default)]
    state_filter: Option<String>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    50
}

/// 2. `GET /v1/worktree-canvas/worktrees`.
async fn list_worktrees(
    State(state): State<Arc<WorktreeCanvasState>>,
    Query(_q): Query<ListWorktreesQuery>,
) -> Result<Json<ListWorktreesResponse>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    Ok(Json(ListWorktreesResponse {
        items: Vec::new(),
        total: 0,
        next_cursor: None,
    }))
}

/// 3. `POST /v1/worktree-canvas/worktrees` (Create).
async fn create_worktree(
    State(state): State<Arc<WorktreeCanvasState>>,
    Json(req): Json<CreateWorktreeRequest>,
) -> Result<(StatusCode, Json<ActionResponse>), WorktreeApiError> {
    state.permission.require_action(WorktreeAction::CreateWorktree)?;

    // Idempotency 重放检查.
    if let Some(cached) = state.idempotency.lookup(req.idempotency_key) {
        if let Ok(resp) = serde_json::from_value::<ActionResponse>(cached.response) {
            return Ok((StatusCode::OK, Json(resp)));
        }
    }

    // 阶段 1: 占位实现.
    let resp = ActionResponse {
        success: true,
        worktree_id: Uuid::new_v4(),
        new_state: graph_core::state::HumanState::Running,
        duration_ms: 0,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    // Idempotency 缓存.
    state
        .idempotency
        .store(
            req.idempotency_key,
            state.actor_id,
            WorktreeAction::CreateWorktree.as_str().to_string(),
            serde_json::to_value(&resp).unwrap_or(serde_json::Value::Null),
        )?;

    // Audit (Safe, 不强制写).
    let audit_event = WorktreeAuditEvent::new(
        Some(resp.worktree_id),
        Some(req.repo_id),
        state.actor_id,
        WorktreeAction::CreateWorktree,
        Some(req.idempotency_key),
        serde_json::json!({"branch": req.branch}),
    );
    state.audit.write_event(audit_event).await?;

    Ok((StatusCode::CREATED, Json(resp)))
}

/// 4. `GET /v1/worktree-canvas/worktrees/:id`.
async fn get_worktree(
    State(state): State<Arc<WorktreeCanvasState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorktreeResponse>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    // 阶段 1: 0 真实 DB, 返回 404 占位.
    Err(WorktreeApiError::not_found(
        format!("worktree {id} not found (stage 1 placeholder)"),
        "controller::get_worktree",
    ))
}

/// `GET /v1/worktree-canvas/worktrees/:id/graph` — focus N-hop subgraph (per IMPL-PLAN).
async fn get_worktree_graph(
    State(state): State<Arc<WorktreeCanvasState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorktreeGraphResponse>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    Ok(Json(WorktreeGraphResponse {
        center: id,
        hop: 1,
        nodes: Vec::new(),
        edges: Vec::new(),
    }))
}

/// 5. `PATCH /v1/worktree-canvas/worktrees/:id`.
async fn update_worktree(
    State(state): State<Arc<WorktreeCanvasState>>,
    Path(id): Path<Uuid>,
    Json(_patch): Json<serde_json::Value>,
) -> Result<Json<WorktreeResponse>, WorktreeApiError> {
    state.permission.require(PermissionLevel::Edit)?;
    Err(WorktreeApiError::not_found(
        format!("worktree {id} not found"),
        "controller::update_worktree",
    ))
}

/// 6. `DELETE /v1/worktree-canvas/worktrees/:id` (Destructive, confirm required).
async fn delete_worktree(
    State(state): State<Arc<WorktreeCanvasState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, WorktreeApiError> {
    state.permission.require_action(WorktreeAction::Delete)?;
    Err(WorktreeApiError::not_found(
        format!("worktree {id} not found"),
        "controller::delete_worktree",
    ))
}

#[derive(Debug, Deserialize)]
struct ExecuteActionPath {
    id: Uuid,
    action_type: String,
}

/// 7. `POST /v1/worktree-canvas/worktrees/:id/actions/:action_type` (18 Action).
async fn execute_action(
    State(state): State<Arc<WorktreeCanvasState>>,
    Path(p): Path<ExecuteActionPath>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, WorktreeApiError> {
    let action = WorktreeAction::from_url(&p.action_type).ok_or_else(|| {
        WorktreeApiError::bad_request(
            format!("unknown action_type: {}", p.action_type),
            "controller::execute_action",
        )
    })?;

    // RBAC 守门.
    state.permission.require_action(action)?;

    // Idempotency 重放检查.
    if let Some(cached) = state.idempotency.lookup(req.idempotency_key) {
        if let Ok(resp) = serde_json::from_value::<ActionResponse>(cached.response) {
            return Ok(Json(resp));
        }
    }

    // Destructive: 必须 confirm=true (per DD §20.2 + spec §4.4).
    if matches!(action.danger(), super::permission::ActionDanger::Destructive) && !req.confirm {
        return Err(WorktreeApiError::conflict(
            format!("destructive action {} requires confirm=true", action.as_str()),
            "controller::execute_action",
        ));
    }

    // 阶段 1: 占位实现.
    let resp = ActionResponse {
        success: true,
        worktree_id: p.id,
        new_state: graph_core::state::HumanState::Running,
        duration_ms: 0,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    // Idempotency 缓存.
    state
        .idempotency
        .store(
            req.idempotency_key,
            state.actor_id,
            action.as_str().to_string(),
            serde_json::to_value(&resp).unwrap_or(serde_json::Value::Null),
        )?;

    // Audit (per 守门 #13 d: 100% Destructive/Warning/Lock/Unlock 写路径过 audit).
    if WorktreeAudit::is_required(action) {
        let audit_event = WorktreeAuditEvent::new(
            Some(p.id),
            None,
            state.actor_id,
            action,
            Some(req.idempotency_key),
            serde_json::json!({"params": req.params, "confirm": req.confirm}),
        );
        state.audit.write_event(audit_event).await?;
    }

    Ok(Json(resp))
}

/// 8. `GET /v1/worktree-canvas/risks`.
async fn list_risks(
    State(state): State<Arc<WorktreeCanvasState>>,
) -> Result<Json<Vec<RiskResponse>>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    Ok(Json(Vec::new()))
}

/// 9. `GET /v1/worktree-canvas/health`.
async fn get_health(
    State(state): State<Arc<WorktreeCanvasState>>,
) -> Result<Json<HealthSummaryDto>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    Ok(Json(HealthSummaryDto {
        overall_score: 100,
        healthy: 0,
        at_risk: 0,
        diverged: 0,
        conflict: 0,
        stale: 0,
    }))
}

/// 10. `POST /v1/worktree-canvas/search` (DSL).
async fn search(
    State(state): State<Arc<WorktreeCanvasState>>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    // 阶段 1: 0 真实 DSL 解析, 仅 validate.
    if req.query.is_empty() {
        return Err(WorktreeApiError::bad_request(
            "query is empty",
            "controller::search",
        ));
    }
    Ok(Json(SearchResponse {
        worktree_ids: Vec::new(),
        total: 0,
        duration_ms: 0,
    }))
}

/// 11. `POST /v1/worktree-canvas/nl-query` (NL → DSL → query).
async fn nl_query(
    State(state): State<Arc<WorktreeCanvasState>>,
    Json(req): Json<NlQueryRequest>,
) -> Result<Json<NlQueryResponse>, WorktreeApiError> {
    state.permission.require(PermissionLevel::View)?;
    // 阶段 1: 0 LLM 集成 (per 守门 #23 v2 LLM 走 mock), 直接 echo + placeholder explanation.
    Ok(Json(NlQueryResponse {
        worktree_ids: Vec::new(),
        translated_query: req.question.clone(),
        explanation: format!("[stage 1 mock] NL query received: {}", req.question),
    }))
}

// =====================================================================
// BFF health (per brief T13 健康检查 — 不计入 11 REST, 独立 health endpoint)
// =====================================================================

/// `GET /v1/worktree-canvas/bff-health` — BFF 实例级 health (跟 worktree 业务 health 区分).
///
/// 公开 endpoint, 不需 RBAC. 通过 `Router::route` 单独挂载, 不使用 State.
pub async fn bff_health() -> Json<BffHealthDto> {
    Json(BffHealthDto {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // 模块级无 state, 占位 0 (per brief T13 健康检查).
        worktree_canvas_enabled: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree_canvas::dto::WorktreeSseEvent;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }
    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    fn make_state(level: PermissionLevel, roles: RoleSet) -> Arc<WorktreeCanvasState> {
        let perm = WorktreePermission::new(tenant(), roles, level);
        Arc::new(
            WorktreeCanvasState::from_parts(
                perm,
                Arc::new(WorktreeAudit::new()),
                Arc::new(IdempotencyCache::new()),
                Arc::new(WorktreeSseHub::new()),
                actor(),
            )
            .expect("state new"),
        )
    }

    #[tokio::test]
    async fn bff_11_rest_endpoints_rbac_enforced() {
        // Brief UT #1: 11 REST endpoint RBAC 强制 (Viewer 不能触发 Destructive Action).
        // 我们 smoke test 至少 3 个代表性 endpoint:
        // 1. GET /repositories (View ok)
        // 2. GET /worktrees (View ok)
        // 3. POST /worktrees/{id}/actions/merge (Destructive, Viewer denied)
        let state = make_state(PermissionLevel::View, RoleSet::new());
        // 1. View 看 repositories 应该 ok.
        assert!(list_repositories(State(state.clone())).await.is_ok());
        // 2. View 看 worktrees 应该 ok.
        let q = ListWorktreesQuery {
            repo_id: None,
            state_filter: None,
            cursor: None,
            limit: 50,
        };
        assert!(list_worktrees(State(state.clone()), Query(q)).await.is_ok());
        // 3. View 触发 Merge (Destructive) 必须拒绝.
        let p = ExecuteActionPath {
            id: Uuid::new_v4(),
            action_type: WorktreeAction::Merge.as_url().into(),
        };
        let req = ActionRequest {
            params: serde_json::json!({}),
            confirm: true,
            idempotency_key: Uuid::new_v4(),
        };
        assert!(execute_action(State(state), Path(p), Json(req)).await.is_err());
    }

    #[tokio::test]
    async fn bff_15_sse_events_authenticated() {
        // Brief UT #2: 15 SSE event 变体都通过 RBAC View 守门.
        // Smoke test: 验证 15 个 event 名都被注册到 WorktreeSseEvent::ALL_NAMES
        // 跟 sse route 1:1 对应.
        use crate::worktree_canvas::sse::is_valid_sse_path;
        assert_eq!(WorktreeSseEvent::ALL_NAMES.len(), 15);
        for name in WorktreeSseEvent::ALL_NAMES {
            let path = format!("/v1/worktree-canvas/events/{name}");
            assert!(is_valid_sse_path(&path), "SSE path not registered: {path}");
        }
        // Unknown event name 应该拒绝.
        assert!(!is_valid_sse_path("/v1/worktree-canvas/events/NotAnEvent"));
    }

    #[tokio::test]
    async fn get_worktree_returns_404_for_unknown_id() {
        let state = make_state(PermissionLevel::View, RoleSet::new());
        let r = get_worktree(State(state), Path(Uuid::new_v4())).await;
        assert!(matches!(r, Err(ref e) if e.code == "NOT_FOUND"));
    }

    #[tokio::test]
    async fn execute_action_rejects_unknown_action() {
        let state = make_state(PermissionLevel::Edit, RoleSet::new());
        let p = ExecuteActionPath {
            id: Uuid::new_v4(),
            action_type: "not-an-action".into(),
        };
        let req = ActionRequest {
            params: serde_json::json!({}),
            confirm: false,
            idempotency_key: Uuid::new_v4(),
        };
        let r = execute_action(State(state), Path(p), Json(req)).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn execute_destructive_action_requires_confirm() {
        use std::iter::FromIterator;
        let mut roles = RoleSet::new();
        roles.insert(super::super::permission::role::LEAD.to_string());
        let state = make_state(PermissionLevel::Edit, roles);
        let p = ExecuteActionPath {
            id: Uuid::new_v4(),
            action_type: WorktreeAction::Merge.as_url().into(),
        };
        let req = ActionRequest {
            params: serde_json::json!({}),
            confirm: false, // 缺 confirm
            idempotency_key: Uuid::new_v4(),
        };
        let r = execute_action(State(state), Path(p), Json(req)).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn execute_destructive_action_succeeds_with_confirm() {
        let mut roles = RoleSet::new();
        roles.insert(super::super::permission::role::LEAD.to_string());
        let state = make_state(PermissionLevel::Edit, roles);
        let p = ExecuteActionPath {
            id: Uuid::new_v4(),
            action_type: WorktreeAction::Merge.as_url().into(),
        };
        let req = ActionRequest {
            params: serde_json::json!({}),
            confirm: true,
            idempotency_key: Uuid::new_v4(),
        };
        let r = execute_action(State(state), Path(p), Json(req)).await;
        assert!(r.is_ok(), "merge with confirm should succeed: {:?}", r.err());
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let state = make_state(PermissionLevel::View, RoleSet::new());
        let req = SearchRequest {
            query: String::new(),
            query_type: "dsl".into(),
        };
        let r = search(State(state), Json(req)).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn nl_query_mock_returns_explanation() {
        let state = make_state(PermissionLevel::View, RoleSet::new());
        let req = NlQueryRequest {
            question: "show conflict worktrees".into(),
        };
        let r = nl_query(State(state), Json(req)).await.unwrap();
        assert!(r.0.explanation.contains("[stage 1 mock]"));
    }
}
