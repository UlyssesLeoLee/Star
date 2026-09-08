// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/arg/controller.rs` — 14 routes (13 REST + 1 WebSocket).
//!
//! Per [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../../../docs/design/DD-AGENT-RELATIONSHIP-001.md)
//! v0.1.1 §4.12. Routes:
//!
//! | Method | Path                                   | Handler              |
//! |--------|----------------------------------------|----------------------|
//! | POST   | `/api/arg/agents`                      | `create_agent`       |
//! | GET    | `/api/arg/agents`                      | `list_agents`        |
//! | GET    | `/api/arg/agents/{id}`                 | `get_agent`          |
//! | PATCH  | `/api/arg/agents/{id}`                 | `update_agent`       |
//! | POST   | `/api/arg/edges`                       | `create_edge`        |
//! | GET    | `/api/arg/edges`                       | `list_edges`         |
//! | GET    | `/api/arg/edges/{id}`                  | `get_edge`           |
//! | PATCH  | `/api/arg/edges/{id}`                  | `update_edge`        |
//! | DELETE | `/api/arg/edges/{id}`                  | `archive_edge`       |
//! | GET    | `/api/arg/graph`                       | `get_graph`          |
//! | POST   | `/api/arg/templates/instantiate`       | `instantiate_template` |
//! | GET    | `/api/arg/achievements`                | `list_achievements`  |
//! | GET    | `/api/arg/achievements/me`             | `my_unlocks`         |
//! | POST   | `/api/arg/achievements/evaluate`       | `evaluate_achievements` |
//! | GET    | `/ws/arg/events`                       | `sse_hub` (WS upgrade) |
//!
//! Total: 14 route registrations (13 REST + 1 WebSocket).
//!
//! 守门合规:
//! - 守门 #7 `unsafe_code = "forbid"` — no `unsafe` blocks.
//! - 守门 #13 (W/T/M) — RLS tenant 校验每个写路径都过 [`crate::arg::permission::ARGPermission::check_tenant`].
//! - 守门 #14 v2 (5 域 Lead Mavis 临时代签) — `Lead` 角色独占, `Admin` 不能越权.
//! - 守门 #24 v2 (subprocess 路径) — `System` 角色允许跨租户 (L0/L1 自动服务).

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

// Method-routing helpers (`post` / `patch` / `delete`) are imported
// individually for each route method-chain to keep the macro-expanded
// `unused_imports` lint quiet while still being a single import block.
#[allow(unused_imports)]
use axum::routing::{delete, patch, post};
use serde::Serialize;
use uuid::Uuid;

use star_arg::models::edge::Edge;
use star_arg::ops::event_writer::EventWriter;

use super::dto::{
    AchievementFilter, AgentFilter, ApiError, CreateAgentRequest, CreateEdgeRequest, EdgeFilter,
    EvaluateAchievementsRequest, EvaluateAchievementsResponse, GraphFilter, GraphResponse,
    InstantiateTemplateRequest, MyUnlocksFilter, PaginationQuery, UpdateAgentRequest,
    UpdateEdgeRequest, MAX_GRAPH_NODES,
};
use super::permission::{role, ARGPermission};
use super::sse_hub::{sse_hub as ws_sse_hub, ARGSseEvent};

/// 14 route router factory.
///
/// 单一入口: caller 传 `Arc<ARGState>`, 我们返回完整 `Router` ready to
/// be merged into the application router (per brief AC-2).
pub fn arg_routes(state: Arc<super::ARGState>) -> Router {
    Router::new()
        .route("/api/arg/agents", post(create_agent).get(list_agents))
        .route("/api/arg/agents/{id}", get(get_agent).patch(update_agent))
        .route("/api/arg/edges", post(create_edge).get(list_edges))
        .route(
            "/api/arg/edges/{id}",
            get(get_edge).patch(update_edge).delete(archive_edge),
        )
        .route("/api/arg/graph", get(get_graph))
        .route("/api/arg/templates/instantiate", post(instantiate_template))
        .route("/api/arg/achievements", get(list_achievements))
        .route("/api/arg/achievements/me", get(my_unlocks))
        .route(
            "/api/arg/achievements/evaluate",
            post(evaluate_achievements),
        )
        .route("/ws/arg/events", get(ws_sse_hub))
        .with_state(state)
}

// =====================================================================
// 5 helper: shared API error → HTTP response mapping
// =====================================================================

/// Convert [`ApiError`] into an axum response. We use a JSON body so
/// the frontend can show a typed error message; the status code is
/// taken from [`ApiError::status_code`].
fn api_error_to_response(e: ApiError) -> (StatusCode, Json<serde_json::Value>) {
    let status = StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = serde_json::json!({
        "error": {
            "code": match &e {
                ApiError::ValidationFailed(_) => "validation_failed",
                ApiError::Unauthorized(_) => "unauthorized",
                ApiError::Forbidden(_) => "forbidden",
                ApiError::NotFound(_) => "not_found",
                ApiError::Conflict(_) => "conflict",
                ApiError::Upstream(_) => "upstream_unavailable",
                ApiError::Internal(_) => "internal",
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
// Agent endpoints (4: create / list / get / update)
// =====================================================================

/// 1. `POST /api/arg/agents` — create a new agent.
async fn create_agent(
    State(state): State<Arc<super::ARGState>>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    let agent = star_arg::models::agent::Agent::new(
        req.name,
        req.archetype,
        req.tenant_id,
        state.tenant_id_for_actor(),
    );
    let created = state.agent_node_ops.create(agent).await?;
    Ok(Json(
        serde_json::to_value(&created).unwrap_or(serde_json::Value::Null),
    ))
}

/// 2. `GET /api/arg/agents` — list agents, filtered + paginated.
async fn list_agents(
    State(state): State<Arc<super::ARGState>>,
    Query(filter): Query<AgentFilter>,
    Query(page): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (limit, offset) = page.resolve()?;
    state.permission.check_tenant(state.tenant_id)?;
    let ops_filter = star_arg::ops::agent_node::AgentFilter {
        archetype: filter.archetype,
        domain: filter.domain,
        status: filter.status,
    };
    let agents = state
        .agent_node_ops
        .list(ops_filter, default_pagination(limit, offset))
        .await?;
    Ok(Json(
        serde_json::to_value(&agents).unwrap_or(serde_json::Value::Null),
    ))
}

/// 3. `GET /api/arg/agents/{id}` — fetch a single agent.
async fn get_agent(
    State(state): State<Arc<super::ARGState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    let agent = state
        .agent_node_ops
        .get(id, state.tenant_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("agent: {id}")))?;
    Ok(Json(
        serde_json::to_value(&agent).unwrap_or(serde_json::Value::Null),
    ))
}

/// 4. `PATCH /api/arg/agents/{id}` — apply a patch to an existing agent.
async fn update_agent(
    State(state): State<Arc<super::ARGState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    let patch = star_arg::ops::agent_node::AgentPatch {
        name: req.name,
        trust_score: req.trust_score,
        metadata: req.metadata,
        status: None,
    };
    let updated = state
        .agent_node_ops
        .update(id, patch, state.tenant_id_for_actor())
        .await?;
    Ok(Json(
        serde_json::to_value(&updated).unwrap_or(serde_json::Value::Null),
    ))
}

// =====================================================================
// Edge endpoints (5: create / list / get / update / archive)
// =====================================================================

/// 5. `POST /api/arg/edges` — create a new edge.
async fn create_edge(
    State(state): State<Arc<super::ARGState>>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    let edge = req.into_edge();
    let created = state.edge_ops.create(edge.clone()).await?;
    // Broadcast an SSE event (best-effort, do not fail the request).
    state.sse_hub.publish(ARGSseEvent::EdgeCreated {
        edge: created.clone(),
    });
    Ok(Json(
        serde_json::to_value(&created).unwrap_or(serde_json::Value::Null),
    ))
}

/// 6. `GET /api/arg/edges` — list edges, filtered.
async fn list_edges(
    State(state): State<Arc<super::ARGState>>,
    Query(filter): Query<EdgeFilter>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    let ops_filter = star_arg::ops::edge_ops::EdgeFilter {
        edge_type: filter.edge_type,
        from_agent: filter.from_agent,
        to_agent: filter.to_agent,
    };
    let _ = filter.archived; // filter by archived is not in ops::EdgeFilter yet (per ARG.1)
    let edges = state.edge_ops.list(ops_filter).await?;
    Ok(Json(
        serde_json::to_value(&edges).unwrap_or(serde_json::Value::Null),
    ))
}

/// 7. `GET /api/arg/edges/{id}` — fetch a single edge.
async fn get_edge(
    State(state): State<Arc<super::ARGState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    let edge = state
        .edge_ops
        .get(id, state.tenant_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("edge: {id}")))?;
    Ok(Json(
        serde_json::to_value(&edge).unwrap_or(serde_json::Value::Null),
    ))
}

/// 8. `PATCH /api/arg/edges/{id}` — apply a weight / metadata patch.
async fn update_edge(
    State(state): State<Arc<super::ARGState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEdgeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    let patch = star_arg::ops::edge_ops::EdgePatch {
        weight: req.weight,
        metadata: req.metadata,
    };
    let updated = state
        .edge_ops
        .update(id, patch, state.tenant_id_for_actor())
        .await?;
    state.sse_hub.publish(ARGSseEvent::EdgeChanged {
        edge: updated.clone(),
    });
    Ok(Json(
        serde_json::to_value(&updated).unwrap_or(serde_json::Value::Null),
    ))
}

/// 9. `DELETE /api/arg/edges/{id}` — soft-archive an edge (`archived = true`).
async fn archive_edge(
    State(state): State<Arc<super::ARGState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::SYSTEM, role::LEAD])?;
    state
        .edge_ops
        .archive(id, state.tenant_id_for_actor())
        .await?;
    state.sse_hub.publish(ARGSseEvent::EdgeArchived {
        id,
        tenant_id: state.tenant_id,
    });
    Ok(StatusCode::NO_CONTENT)
}

// =====================================================================
// Graph endpoint (1: get_graph)
// =====================================================================

/// 10. `GET /api/arg/graph` — return up to [`MAX_GRAPH_NODES`] nodes,
///     with the edges between them.
///
/// Per ARG.1 G-1 the underlying list is a stub; the controller returns
/// an empty graph (200 OK) and the frontend can still layout the canvas.
async fn get_graph(
    State(state): State<Arc<super::ARGState>>,
    Query(filter): Query<GraphFilter>,
) -> Result<Json<GraphResponse>, ApiError> {
    let tenant = filter.tenant_id.unwrap_or(state.tenant_id);
    state.permission.check_tenant(tenant)?;
    let cap = filter.resolved_cap();
    if cap > MAX_GRAPH_NODES {
        return Err(ApiError::ValidationFailed(format!(
            "max_nodes > {MAX_GRAPH_NODES}"
        )));
    }
    // G-1 stub: empty graph with a 200 OK. Real impl lands in P3-C W1.
    let response = GraphResponse::new(vec![], vec![]);
    Ok(Json(response))
}

// =====================================================================
// Template endpoint (1: instantiate_template)
// =====================================================================

/// 11. `POST /api/arg/templates/instantiate` — materialise a [`star_arg::models::template::TeamTemplate`].
async fn instantiate_template(
    State(state): State<Arc<super::ARGState>>,
    Json(req): Json<InstantiateTemplateRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    req.validate()?;
    state.permission.check_tenant(req.tenant_id)?;
    state
        .permission
        .require_any_role(&[role::EDITOR, role::ADMIN, role::LEAD, role::SYSTEM])?;
    let instance = state
        .template_ops
        .instantiate(
            req.template_id,
            req.agent_ids,
            req.instance_name,
            req.tenant_id,
            req.created_by,
        )
        .await?;
    Ok(Json(
        serde_json::to_value(&instance).unwrap_or(serde_json::Value::Null),
    ))
}

// =====================================================================
// Achievement endpoints (3: list / my_unlocks / evaluate)
// =====================================================================

/// 12. `GET /api/arg/achievements` — list all 20 achievement definitions
///     (per DD §3.2.4). Optional category / rarity filters are applied.
async fn list_achievements(
    State(state): State<Arc<super::ARGState>>,
    Query(filter): Query<AchievementFilter>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    let all = star_arg::models::achievement::all_achievements();
    let filtered: Vec<_> = all
        .into_iter()
        .filter(|a| match filter.category {
            Some(c) => a.category == c,
            None => true,
        })
        .filter(|a| match filter.rarity {
            Some(r) => a.rarity == r,
            None => true,
        })
        .collect();
    Ok(Json(
        serde_json::to_value(&filtered).unwrap_or(serde_json::Value::Null),
    ))
}

/// 13. `GET /api/arg/achievements/me` — current user's unlocks.
///
/// Per ARG.1, the unlock history is held by the [`star_arg::ops::EventWriter`]
/// (in-process); the controller returns the events that are
/// `ARGEvent::AchievementUnlocked` and match the actor's `tenant_id`.
async fn my_unlocks(
    State(state): State<Arc<super::ARGState>>,
    Query(filter): Query<MyUnlocksFilter>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tenant = filter.tenant_id.unwrap_or(state.tenant_id);
    state.permission.check_tenant(tenant)?;
    let limit = filter.limit.unwrap_or(100);
    let unlocks: Vec<_> = state
        .event_writer
        .snapshot()
        .into_iter()
        .filter_map(|e| match e {
            star_arg::models::ARGEvent::AchievementUnlocked(u) if u.tenant_id == tenant => Some(u),
            _ => None,
        })
        .take(limit as usize)
        .collect();
    Ok(Json(
        serde_json::to_value(&unlocks).unwrap_or(serde_json::Value::Null),
    ))
}

/// 14. `POST /api/arg/achievements/evaluate` — admin-only re-evaluation
///     of the achievement engine. The endpoint returns the list of newly
///     unlocked codes (empty for the G-1 stub).
async fn evaluate_achievements(
    State(state): State<Arc<super::ARGState>>,
    Json(req): Json<EvaluateAchievementsRequest>,
) -> Result<Json<EvaluateAchievementsResponse>, ApiError> {
    let tenant = req.tenant_id.unwrap_or(state.tenant_id);
    state.permission.check_tenant(tenant)?;
    // Per DD §4.12, `evaluate` is admin-only.
    state.permission.require_role(role::ADMIN)?;
    let response = EvaluateAchievementsResponse {
        newly_unlocked: vec![],
        candidates_evaluated: 0,
    };
    Ok(Json(response))
}

// =====================================================================
// Shared helpers
// =====================================================================

fn default_pagination(limit: u32, offset: u32) -> star_arg::ops::agent_node::Pagination {
    star_arg::ops::agent_node::Pagination { limit, offset }
}

// Reference: keep `Edge` and `EventWriter` reachable for future edits
// (the broadcast helper signature uses these types).
#[allow(dead_code)]
fn _types_reachable() {
    let _e: Option<Edge> = None;
    let _w: Option<EventWriter> = None;
    let _p: Option<ARGPermission> = None;
    // Touch Serialize so the import does not get pruned on a future edit.
    fn _assert_serialize<T: Serialize>() {}
    _assert_serialize::<Edge>();
}
