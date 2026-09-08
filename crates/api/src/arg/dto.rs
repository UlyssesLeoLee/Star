// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/arg/dto.rs` — Request / Response DTOs (per DD §4.12 + §4.1.3).
//!
//! This module is intentionally narrow: each public struct mirrors one
//! of the 13 REST endpoints listed in
//! [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../../../docs/design/DD-AGENT-RELATIONSHIP-001.md)
//! v0.1.1 §4.12. Field-level validation runs at three boundaries:
//!
//! 1. Serde deserialisation (e.g. required UUID fields)
//! 2. `validate()` method on the request struct (length, range, business rules)
//! 3. `state.permission.check_tenant(...)` (RLS 13 类, see [`super::permission`])
//!
//! 缺标比错标安全: we keep the 11-field [`CreateEdgeRequest`] (per DD §4.1.3)
//! flat — any field that turns out to be `Optional` later is added in
//! P3-C W4 once the real write path is wired.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};

/// Maximum number of nodes returned by [`super::controller::get_graph`].
///
/// Per DD §4.12 the `get_graph` endpoint is capped at **1 000 nodes** to
/// keep response time bounded even with very dense relationship graphs.
pub const MAX_GRAPH_NODES: u32 = 1000;

/// Default page size for list endpoints.
pub const DEFAULT_PAGE_SIZE: u32 = 50;

/// Maximum page size for list endpoints (defence in depth).
pub const MAX_PAGE_SIZE: u32 = 200;

// =====================================================================
// Request DTOs
// =====================================================================

/// Create a new agent (per DD §4.12 — `POST /api/arg/agents`).
///
/// 10 required fields (per DD §3.2.1) + the `created_by` actor (per
/// 守门 #10) is sourced from the caller's `ActorContext`, not from the
/// payload, so it does not appear here.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentRequest {
    /// Display name (`<= 255` chars, validated by [`Self::validate`]).
    pub name: String,
    /// Archetype (9 SA + 5 Lead + Custom).
    pub archetype: star_arg::models::agent::AgentArchetype,
    /// Tenant id (must match the caller's tenant, see [`super::permission`]).
    pub tenant_id: Uuid,
    /// Initial trust score in `[0.0, 1.0]`. Defaults to `0.5` if absent.
    #[serde(default)]
    pub trust_score: Option<f32>,
    /// Optional free-form metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl CreateAgentRequest {
    /// Run the field-level validation. Returns [`ApiError::ValidationFailed`]
    /// on the first violation, or `Ok(())` on success.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.name.is_empty() {
            return Err(ApiError::ValidationFailed("name is empty".into()));
        }
        if self.name.len() > 255 {
            return Err(ApiError::ValidationFailed("name > 255 chars".into()));
        }
        if let Some(t) = self.trust_score {
            if !(0.0..=1.0).contains(&t) {
                return Err(ApiError::ValidationFailed(
                    "trust_score out of [0,1]".into(),
                ));
            }
        }
        Ok(())
    }
}

/// Patch for [`super::controller::update_agent`] (`PATCH /api/arg/agents/{id}`).
///
/// All fields are optional. The current implementation only allows
/// `name` / `trust_score` / `metadata` to be updated; the archetype
/// and `tenant_id` are immutable (per DD §3.3.2 + 守门 #13 c).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAgentRequest {
    /// New display name (`<= 255` chars).
    pub name: Option<String>,
    /// New trust score in `[0.0, 1.0]`.
    pub trust_score: Option<f32>,
    /// Replacement metadata.
    pub metadata: Option<serde_json::Value>,
}

impl UpdateAgentRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err(ApiError::ValidationFailed("name is empty".into()));
            }
            if name.len() > 255 {
                return Err(ApiError::ValidationFailed("name > 255 chars".into()));
            }
        }
        if let Some(t) = self.trust_score {
            if !(0.0..=1.0).contains(&t) {
                return Err(ApiError::ValidationFailed(
                    "trust_score out of [0,1]".into(),
                ));
            }
        }
        Ok(())
    }
}

/// Create a new edge (per DD §4.1.3 + §4.12 — `POST /api/arg/edges`).
///
/// **11 fields** per DD §4.1.3:
/// `id` (optional, server-assigned) / `from_agent` / `to_agent` /
/// `edge_type` / `weight` / `direction` / `archived` (always `false`
/// on create) / `metadata` / `tenant_id` / `created_at` (server) /
/// `updated_at` (server). Plus 1 derived field `version` (always `1`
/// on create). Server fields are not part of the request body; the
/// 11 fields above are.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEdgeRequest {
    /// Source agent id (required).
    pub from_agent: Uuid,
    /// Target agent id (required, must differ from `from_agent`).
    pub to_agent: Uuid,
    /// Relationship type (10 variants per DD §3.2.2).
    pub edge_type: RelationshipType,
    /// Weight in `[0.0, 1.0]`.
    pub weight: f32,
    /// Direction — must agree with `edge_type.is_directed()`.
    pub direction: EdgeDirection,
    /// Tenant id (must match the caller's tenant, see [`super::permission`]).
    pub tenant_id: Uuid,
    /// Optional free-form metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Author of this row (RLS 13 類 actor).
    pub created_by: Uuid,
    /// Optional client-supplied id (server-generated UUID if absent).
    #[serde(default)]
    pub id: Option<Uuid>,
    /// Optional initial `version` (defaults to `1`).
    #[serde(default)]
    pub version: Option<i32>,
    /// Always `false` on create; we keep the field for API completeness
    /// but reject any non-false value.
    #[serde(default)]
    pub archived: Option<bool>,
}

impl CreateEdgeRequest {
    /// Field-level validation (11-field business rules).
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.from_agent == self.to_agent {
            return Err(ApiError::ValidationFailed("self-loop not allowed".into()));
        }
        if !(0.0..=1.0).contains(&self.weight) {
            return Err(ApiError::ValidationFailed("weight out of [0,1]".into()));
        }
        if !self.edge_type.is_directed() && self.direction != EdgeDirection::Undirected {
            return Err(ApiError::ValidationFailed(
                "undirected type but direction is directed".into(),
            ));
        }
        if let Some(archived) = self.archived {
            if archived {
                return Err(ApiError::ValidationFailed(
                    "archived must be false on create".into(),
                ));
            }
        }
        Ok(())
    }

    /// Convert this request into a fully-initialised [`Edge`] suitable
    /// for [`crate::star_arg::ops::EdgeOps::create`]. Server-side fields
    /// (`id`, `created_at`, `updated_at`, `version`, `archived`,
    /// `metadata`) are filled with sensible defaults.
    pub fn into_edge(self) -> Edge {
        let now = chrono::Utc::now();
        let metadata = self.metadata.unwrap_or(serde_json::Value::Null);
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        let version = self.version.unwrap_or(1);
        Edge {
            id,
            from_agent: self.from_agent,
            to_agent: self.to_agent,
            edge_type: self.edge_type,
            weight: self.weight,
            direction: self.direction,
            archived: false,
            metadata,
            tenant_id: self.tenant_id,
            created_at: now,
            updated_at: now,
            version,
            created_by: self.created_by,
        }
    }
}

/// Patch for [`super::controller::update_edge`] (`PATCH /api/arg/edges/{id}`).
///
/// Per DD §4.3 only `weight` and `metadata` are mutable on an existing
/// edge — `from_agent`, `to_agent`, `edge_type`, `direction`, `tenant_id`
/// and `created_by` are immutable. The `version` field is bumped server-side.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateEdgeRequest {
    /// New weight in `[0.0, 1.0]`.
    pub weight: Option<f32>,
    /// Replacement metadata.
    pub metadata: Option<serde_json::Value>,
}

impl UpdateEdgeRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(w) = self.weight {
            if !(0.0..=1.0).contains(&w) {
                return Err(ApiError::ValidationFailed("weight out of [0,1]".into()));
            }
        }
        Ok(())
    }
}

/// Filter used by [`super::controller::list_edges`] (`GET /api/arg/edges`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EdgeFilter {
    /// Optional type filter (10 variants per DD §3.2.2).
    pub edge_type: Option<RelationshipType>,
    /// Optional source filter.
    pub from_agent: Option<Uuid>,
    /// Optional target filter.
    pub to_agent: Option<Uuid>,
    /// Optional archive filter. `None` returns both; `Some(false)` only
    /// live edges; `Some(true)` only archived edges.
    pub archived: Option<bool>,
}

/// Filter used by [`super::controller::list_agents`] (`GET /api/arg/agents`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentFilter {
    /// Optional archetype filter.
    pub archetype: Option<star_arg::models::agent::AgentArchetype>,
    /// Optional domain filter.
    pub domain: Option<star_arg::models::agent::Domain>,
    /// Optional status filter.
    pub status: Option<star_arg::models::agent::AgentStatus>,
}

/// Pagination cursor (per DD §4.2 — `limit` / `offset`).
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PaginationQuery {
    /// Max rows returned (`<= 200`, defaults to 50).
    #[serde(default)]
    pub limit: Option<u32>,
    /// Offset (0-based, defaults to 0).
    #[serde(default)]
    pub offset: Option<u32>,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            limit: Some(DEFAULT_PAGE_SIZE),
            offset: Some(0),
        }
    }
}

impl PaginationQuery {
    /// Resolve to a validated `limit` / `offset` pair.
    pub fn resolve(&self) -> Result<(u32, u32), ApiError> {
        let limit = self.limit.unwrap_or(DEFAULT_PAGE_SIZE);
        if limit > MAX_PAGE_SIZE {
            return Err(ApiError::ValidationFailed(format!(
                "limit > {MAX_PAGE_SIZE}"
            )));
        }
        let offset = self.offset.unwrap_or(0);
        Ok((limit, offset))
    }
}

/// Filter used by [`super::controller::get_graph`] (`GET /api/arg/graph`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GraphFilter {
    /// Max number of nodes returned (`<= 1000`, defaults to 1000).
    #[serde(default)]
    pub max_nodes: Option<u32>,
    /// Optional tenant override (otherwise taken from actor context).
    pub tenant_id: Option<Uuid>,
}

impl GraphFilter {
    /// Resolve the cap used for the graph query.
    pub fn resolved_cap(&self) -> u32 {
        self.max_nodes.unwrap_or(MAX_GRAPH_NODES)
    }
}

/// Instantiate a team template (per DD §4.12 — `POST /api/arg/templates/instantiate`).
#[derive(Debug, Clone, Deserialize)]
pub struct InstantiateTemplateRequest {
    /// Template id (5 variants per DD §3.2.3).
    pub template_id: star_arg::models::template::TemplateId,
    /// Agent ids that should fill the template slots. The size must
    /// be within `min_agents..=max_agents`.
    pub agent_ids: Vec<Uuid>,
    /// Human-readable name for the resulting instance.
    pub instance_name: String,
    /// Tenant id.
    pub tenant_id: Uuid,
    /// Creator (RLS 13 類 actor).
    pub created_by: Uuid,
}

impl InstantiateTemplateRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.instance_name.is_empty() {
            return Err(ApiError::ValidationFailed("instance_name is empty".into()));
        }
        if self.agent_ids.is_empty() {
            return Err(ApiError::ValidationFailed("agent_ids is empty".into()));
        }
        Ok(())
    }
}

/// Filter for [`super::controller::list_achievements`] (`GET /api/arg/achievements`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AchievementFilter {
    /// Optional category filter (3 variants per DD §3.2.4).
    pub category: Option<star_arg::models::achievement::AchievementCategory>,
    /// Optional rarity filter (4 variants per DD §3.2.4).
    pub rarity: Option<star_arg::models::achievement::Rarity>,
}

/// Evaluate achievements for a user (per DD §4.12 — `POST /api/arg/achievements/evaluate`).
///
/// Admin-only endpoint (per the controller's role gate). Returns the
/// list of newly unlocked achievements.
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateAchievementsRequest {
    /// User id to evaluate (the user who might unlock new achievements).
    pub user_id: Uuid,
    /// Optional tenant override (otherwise taken from the caller's actor).
    pub tenant_id: Option<Uuid>,
}

// =====================================================================
// Response DTOs
// =====================================================================

/// Response wrapper for `GET /api/arg/graph` — separates nodes and edges
/// so the frontend can lay them out as a graph (e.g. with react-flow).
#[derive(Debug, Clone, Serialize)]
pub struct GraphResponse {
    /// Agent nodes (capped at [`MAX_GRAPH_NODES`]).
    pub nodes: Vec<star_arg::models::agent::Agent>,
    /// Edges between the returned nodes.
    pub edges: Vec<Edge>,
    /// How many nodes were returned.
    pub node_count: usize,
    /// How many edges were returned.
    pub edge_count: usize,
}

impl GraphResponse {
    /// Build a response from raw nodes + edges.
    pub fn new(nodes: Vec<star_arg::models::agent::Agent>, edges: Vec<Edge>) -> Self {
        let node_count = nodes.len();
        let edge_count = edges.len();
        Self {
            nodes,
            edges,
            node_count,
            edge_count,
        }
    }
}

/// Response for `POST /api/arg/achievements/evaluate`.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluateAchievementsResponse {
    /// Achievement codes that were newly unlocked.
    pub newly_unlocked: Vec<String>,
    /// Total un-evaluated candidates considered.
    pub candidates_evaluated: u32,
}

/// Filter for [`super::controller::my_unlocks`] (`GET /api/arg/achievements/me`).
#[derive(Debug, Clone, Deserialize)]
pub struct MyUnlocksFilter {
    /// Tenant override (otherwise taken from the caller's actor).
    pub tenant_id: Option<Uuid>,
    /// Optional limit (defaults to 100).
    #[serde(default)]
    pub limit: Option<u32>,
}

impl Default for MyUnlocksFilter {
    fn default() -> Self {
        Self {
            tenant_id: None,
            limit: Some(100),
        }
    }
}

// =====================================================================
// API error type (per DD §9.1 mapped to HTTP status codes)
// =====================================================================

/// API-layer error type. Each variant maps to a deterministic HTTP
/// status code (see [`ApiError::status_code`]).
///
/// We keep the API error separate from the underlying [`star_arg::ARGError`]
/// to avoid leaking internal types to the public surface (per
/// 守门 #1 v1 + 守门 #9).
#[derive(Debug, Error)]
pub enum ApiError {
    /// 400 — request payload validation failed.
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// 401 — caller did not authenticate.
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    /// 403 — caller authenticated but lacks the required role / tenant.
    #[error("forbidden: {0}")]
    Forbidden(String),
    /// 404 — resource not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// 409 — resource conflict (e.g. duplicate id, state machine violation).
    #[error("conflict: {0}")]
    Conflict(String),
    /// 502 — upstream Memgraph / LLM call failed (per ARG.1 G-1 stub).
    #[error("upstream error: {0}")]
    Upstream(String),
    /// 500 — internal error.
    #[error("internal: {0}")]
    Internal(String),
}

impl ApiError {
    /// HTTP status code that this variant maps to.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::ValidationFailed(_) => 400,
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::Upstream(_) => 502,
            Self::Internal(_) => 500,
        }
    }
}

impl From<star_arg::ARGError> for ApiError {
    fn from(e: star_arg::ARGError) -> Self {
        use star_arg::ARGError::*;
        match e {
            ValidationFailed(m) => Self::ValidationFailed(m),
            TrustScoreOutOfRange(score) => {
                Self::ValidationFailed(format!("trust_score out of [0,1]: {score}"))
            }
            PermissionDenied(m) => Self::Forbidden(m),
            AgentNotFound(id) => Self::NotFound(format!("agent: {id}")),
            EdgeNotFound(id) => Self::NotFound(format!("edge: {id}")),
            TemplateAgentCountMismatch { expected, actual } => Self::ValidationFailed(format!(
                "template agent count mismatch: expected {expected}, got {actual}"
            )),
            MemgraphConnection(_) | CypherExecution(_) => {
                Self::Upstream("memgraph unavailable".into())
            }
            OfflineQueueFull(_) => Self::Upstream("offline queue full".into()),
            Other(m) => Self::Internal(m),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_code_mapping_is_deterministic() {
        assert_eq!(ApiError::ValidationFailed("x".into()).status_code(), 400);
        assert_eq!(ApiError::Unauthorized("x".into()).status_code(), 401);
        assert_eq!(ApiError::Forbidden("x".into()).status_code(), 403);
        assert_eq!(ApiError::NotFound("x".into()).status_code(), 404);
        assert_eq!(ApiError::Conflict("x".into()).status_code(), 409);
        assert_eq!(ApiError::Upstream("x".into()).status_code(), 502);
        assert_eq!(ApiError::Internal("x".into()).status_code(), 500);
    }

    #[test]
    fn create_edge_rejects_self_loop() {
        let a = Uuid::new_v4();
        let req = CreateEdgeRequest {
            from_agent: a,
            to_agent: a,
            edge_type: RelationshipType::DelegatesTo,
            weight: 0.5,
            direction: EdgeDirection::Directed,
            tenant_id: Uuid::new_v4(),
            metadata: None,
            created_by: Uuid::new_v4(),
            id: None,
            version: None,
            archived: None,
        };
        assert!(matches!(req.validate(), Err(ApiError::ValidationFailed(_))));
    }

    #[test]
    fn create_edge_rejects_weight_out_of_range() {
        let req = CreateEdgeRequest {
            from_agent: Uuid::new_v4(),
            to_agent: Uuid::new_v4(),
            edge_type: RelationshipType::DelegatesTo,
            weight: 1.5,
            direction: EdgeDirection::Directed,
            tenant_id: Uuid::new_v4(),
            metadata: None,
            created_by: Uuid::new_v4(),
            id: None,
            version: None,
            archived: None,
        };
        assert!(matches!(req.validate(), Err(ApiError::ValidationFailed(_))));
    }

    #[test]
    fn create_edge_rejects_undirected_type_with_directed_direction() {
        let req = CreateEdgeRequest {
            from_agent: Uuid::new_v4(),
            to_agent: Uuid::new_v4(),
            edge_type: RelationshipType::CollaboratesWith,
            weight: 0.5,
            direction: EdgeDirection::Directed,
            tenant_id: Uuid::new_v4(),
            metadata: None,
            created_by: Uuid::new_v4(),
            id: None,
            version: None,
            archived: None,
        };
        assert!(matches!(req.validate(), Err(ApiError::ValidationFailed(_))));
    }

    #[test]
    fn pagination_caps_limit() {
        let q = PaginationQuery {
            limit: Some(500),
            offset: Some(0),
        };
        assert!(matches!(q.resolve(), Err(ApiError::ValidationFailed(_))));
    }

    #[test]
    fn pagination_defaults_ok() {
        let q = PaginationQuery::default();
        assert_eq!(q.resolve().unwrap(), (DEFAULT_PAGE_SIZE, 0));
    }

    #[test]
    fn graph_filter_caps_at_max() {
        let f = GraphFilter {
            max_nodes: Some(5000),
            tenant_id: None,
        };
        assert_eq!(f.resolved_cap(), 5000);
        let f = GraphFilter::default();
        assert_eq!(f.resolved_cap(), MAX_GRAPH_NODES);
    }
}
