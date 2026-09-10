// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/agent/dto.rs` — Request / Response DTOs for A1-A10 agent API (per DD-AGENT §3.1 + §4.1).
//!
//! Per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.4 + `docs/briefs/p3-d6-1-4-api-extend.md` §2.5.
//! 13 Request/Response DTOs + 1 `ApiError` enum 5-variant (跟 arg/dto.rs 6-variant 不同, 5-variant
//! per brief §2.5 + 守门 #14 v4 "actor = `架构师 (Mavis 接手 agent per DEC-008)`").
//!
//! 阶段 1 基础: 字段定义 + validate(), 0 业务方法实装 (留阶段 2 任务 2.1-2.5).
//! 阶段 2 才走 `state.agent_ops.create(req.into(), state.tenant_id).await?` 等真实调用.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #14 v2 + 守门 #19 v19):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 字段类型跟 `agent_domain::Agent` 14 字段 1:1 对齐 (per DD §4.14.1 + 守门 #19 v19 累积规).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// API error type (5 variants per brief §2.5)
// =====================================================================

/// API-layer error type. 5 variants (per brief §2.5 + 守门 #11 缺标比错标,
/// 比 `arg/dto.rs` 6-variant 少 `Conflict` + `Unauthorized` 2 个, 留阶段 2 续做).
#[derive(Debug, Error)]
pub enum ApiError {
    /// 400 — request payload validation failed.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// 403 — caller authenticated but lacks the required role / tenant.
    #[error("forbidden: {0}")]
    Forbidden(String),
    /// 404 — resource not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// 500 — internal error.
    #[error("internal: {0}")]
    Internal(String),
    /// 501 — not implemented (per brief §0 + §1.2 阶段 1 占位, 阶段 2 实装业务方法).
    #[error("not implemented: {0}")]
    Unimplemented(String),
}

impl ApiError {
    /// HTTP status code that this variant maps to.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Internal(_) => 500,
            Self::Unimplemented(_) => 501,
        }
    }
}

// =====================================================================
// Request DTOs
// =====================================================================

/// Create a new agent (per A1.1 — `POST /api/v1/agents`).
///
/// 10 字段 per `agent_domain::Agent` 14 字段 (per DD §4.14.1, 跳过 server 字段
/// `id` / `started_at` / `tenant_id` 由 server 注入 + `version` 默认 1):
/// - `name` / `avatar_url` / `role` / `kind` / `domain`
/// - `status` (default = `Initializing`) / `token_usage` (default 0) / `token_budget` (default 1.2M)
/// - `parent_session_id` (optional) / `pipeline_agent_ids` (default empty)
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentRequest {
    /// Display name (`<= 255` chars, validated by [`Self::validate`]).
    pub name: String,
    /// Optional avatar URL.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Agent role (Supervisor / Worker / Reviewer).
    pub role: agent_domain::AgentRole,
    /// Agent kind (9 SA + Custom).
    pub kind: agent_domain::AgentKind,
    /// 5 域 (None = 跨域 Agent, per DD §4.14.1 派生).
    #[serde(default)]
    pub domain: Option<agent_domain::Domain>,
    /// 14 状态机 初始状态 (defaults to `Initializing` per `Agent::default()`).
    #[serde(default)]
    pub status: Option<agent_domain::AgentState>,
    /// token 累计使用量 (default 0).
    #[serde(default)]
    pub token_usage: Option<u64>,
    /// token 预算 (default 1.2M / SRE·周 per STAR-OLU-001 §6).
    #[serde(default)]
    pub token_budget: Option<u64>,
    /// Optional parent session ID (per A2.3 派生).
    #[serde(default)]
    pub parent_session_id: Option<Uuid>,
    /// Pipeline agent IDs (per A2.4 派生).
    #[serde(default)]
    pub pipeline_agent_ids: Vec<Uuid>,
}

impl CreateAgentRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.name.is_empty() {
            return Err(ApiError::BadRequest("name is empty".into()));
        }
        if self.name.len() > 255 {
            return Err(ApiError::BadRequest("name > 255 chars".into()));
        }
        Ok(())
    }
}

/// Patch for [`super::controller::update_agent`] (`PATCH /api/v1/agents/{id}`).
///
/// 8 字段 (per A1.1 partial update 派生). `id` / `tenant_id` / `version` / `started_at`
/// 不可改, `token_usage` 也不可改 (per STAR-OLU-001 §6 token 预算仅 server 内部 bump).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAgentRequest {
    /// New display name.
    pub name: Option<String>,
    /// New avatar URL.
    pub avatar_url: Option<String>,
    /// New role.
    pub role: Option<agent_domain::AgentRole>,
    /// New kind.
    pub kind: Option<agent_domain::AgentKind>,
    /// New domain.
    pub domain: Option<Option<agent_domain::Domain>>,
    /// New status (state machine transition).
    pub status: Option<agent_domain::AgentState>,
    /// New token budget.
    pub token_budget: Option<u64>,
    /// New pipeline agent IDs.
    pub pipeline_agent_ids: Option<Vec<Uuid>>,
}

impl UpdateAgentRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err(ApiError::BadRequest("name is empty".into()));
            }
            if name.len() > 255 {
                return Err(ApiError::BadRequest("name > 255 chars".into()));
            }
        }
        Ok(())
    }
}

/// A2.1 handoff — `POST /api/v1/agents/{id}/handoff`.
#[derive(Debug, Clone, Deserialize)]
pub struct HandoffRequest {
    /// 目标 agent ID (per A2.1 handoff 派生).
    pub target_agent_id: Uuid,
    /// handoff reason (per DD §4.14.2 audit 派生).
    #[serde(default)]
    pub reason: Option<String>,
    /// tenant_id (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

impl HandoffRequest {
    /// Field-level validation.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.target_agent_id.is_nil() {
            return Err(ApiError::BadRequest("target_agent_id is nil".into()));
        }
        if self.tenant_id.is_nil() {
            return Err(ApiError::BadRequest("tenant_id is nil".into()));
        }
        Ok(())
    }
}

/// A2.3 set parent — `POST /api/v1/agents/{id}/parent`.
#[derive(Debug, Clone, Deserialize)]
pub struct ParentRequest {
    /// Parent session ID (None = detach, per A2.3 派生).
    pub parent_session_id: Option<Uuid>,
    /// tenant_id (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

/// A2.4 add to pipeline — `POST /api/v1/agents/{id}/pipeline`.
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineRequest {
    /// Pipeline agent IDs to add (per A2.4 派生).
    pub agent_ids: Vec<Uuid>,
    /// tenant_id (per 守门 #13 a 100% RLS 13 类).
    pub tenant_id: Uuid,
}

/// A3.2 state change — `POST /api/v1/agents/{id}/state`.
#[derive(Debug, Clone, Deserialize)]
pub struct StateChangeRequest {
    /// 目标 14 状态 (per DD §5.2 状态机).
    pub new_state: agent_domain::AgentState,
    /// Optional reason (per audit 派生).
    #[serde(default)]
    pub reason: Option<String>,
}

// =====================================================================
// Response DTOs
// =====================================================================

/// Response for create / get / update agent — 14 字段 1:1 跟 `agent_domain::Agent`.
#[derive(Debug, Clone, Serialize)]
pub struct AgentResponse {
    /// Global unique ID.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional avatar URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Role: Supervisor / Worker / Reviewer.
    pub role: agent_domain::AgentRole,
    /// Kind: 9 SA + Custom.
    pub kind: agent_domain::AgentKind,
    /// 5 域.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<agent_domain::Domain>,
    /// 14 状态机.
    pub status: agent_domain::AgentState,
    /// token 累计使用量.
    pub token_usage: u64,
    /// token 预算.
    pub token_budget: u64,
    /// Parent session ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<Uuid>,
    /// Pipeline agent IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pipeline_agent_ids: Vec<Uuid>,
    /// Start time (UTC).
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Tenant ID (per 守门 #13 a RLS 13 类).
    pub tenant_id: Uuid,
    /// SCD Type 2 版本.
    pub version: u32,
}

impl Default for AgentResponse {
    fn default() -> Self {
        let agent = agent_domain::Agent::default();
        Self::from_agent(agent)
    }
}

impl AgentResponse {
    /// Build an `AgentResponse` from a domain `Agent` value.
    pub fn from_agent(a: agent_domain::Agent) -> Self {
        Self {
            id: a.id,
            name: a.name,
            avatar_url: a.avatar_url,
            role: a.role,
            kind: a.kind,
            domain: a.domain,
            status: a.status,
            token_usage: a.token_usage,
            token_budget: a.token_budget,
            parent_session_id: a.parent_session_id,
            pipeline_agent_ids: a.pipeline_agent_ids,
            started_at: a.started_at,
            tenant_id: a.tenant_id,
            version: a.version,
        }
    }
}

/// A2.1 handoff response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct HandoffResponse {
    /// handoff 后的 agent 状态.
    pub agent: AgentResponse,
    /// handoff 目标 agent.
    pub target_agent_id: Uuid,
    /// handoff 时间 (UTC).
    pub at: chrono::DateTime<chrono::Utc>,
}

/// A2.2 topology response — 5 域 Frame info (per DD §3.3.2 派生).
#[derive(Debug, Clone, Serialize, Default)]
pub struct TopologyResponse {
    /// Agent ID (per A2.2).
    pub agent_id: Uuid,
    /// 5 域 拓扑: 域 → 同域 agent IDs.
    pub domain_topology: std::collections::HashMap<agent_domain::Domain, Vec<Uuid>>,
    /// 跨域 handoff 边.
    pub cross_domain_edges: Vec<Uuid>,
}

/// A2.3 parent response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ParentResponse {
    /// Agent ID.
    pub agent_id: Uuid,
    /// Parent session ID (None = detached).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<Uuid>,
}

/// A2.4 pipeline response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PipelineResponse {
    /// Agent ID.
    pub agent_id: Uuid,
    /// Pipeline agent IDs after add.
    pub pipeline_agent_ids: Vec<Uuid>,
}

/// A4.1 worktree association response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct WorktreeResponse {
    /// Agent ID.
    pub agent_id: Uuid,
    /// Worktree ID (per A4.1 派生, 0..1 mapping).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_id: Option<Uuid>,
    /// Worktree 路径.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
}

/// A4.2 work-items list response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct WorkItemListResponse {
    /// Agent ID.
    pub agent_id: Uuid,
    /// Work-item IDs (per A4.2 1:N mapping).
    pub work_item_ids: Vec<Uuid>,
    /// Total count.
    pub count: usize,
}

/// A6.1 start / stop / restart 共用 response.
#[derive(Debug, Clone, Serialize, Default)]
pub struct AgentActionResponse {
    /// Agent ID.
    pub agent_id: Uuid,
    /// 当前 14 状态.
    pub status: agent_domain::AgentState,
    /// 动作类型 ("start" / "stop" / "restart" / "state_change").
    pub action: String,
    /// SCD Type 2 版本 (bumped).
    pub version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_code_mapping_is_deterministic() {
        assert_eq!(ApiError::BadRequest("x".into()).status_code(), 400);
        assert_eq!(ApiError::Forbidden("x".into()).status_code(), 403);
        assert_eq!(ApiError::NotFound("x".into()).status_code(), 404);
        assert_eq!(ApiError::Internal("x".into()).status_code(), 500);
        assert_eq!(ApiError::Unimplemented("x".into()).status_code(), 501);
    }

    #[test]
    fn create_agent_rejects_empty_name() {
        let req = CreateAgentRequest {
            name: String::new(),
            avatar_url: None,
            role: agent_domain::AgentRole::Worker,
            kind: agent_domain::AgentKind::Custom,
            domain: None,
            status: None,
            token_usage: None,
            token_budget: None,
            parent_session_id: None,
            pipeline_agent_ids: vec![],
        };
        assert!(matches!(req.validate(), Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn agent_response_from_default_agent_roundtrip_fields() {
        let resp = AgentResponse::default();
        // 14 字段都填了 (除 2 个 Option 默认 None)
        assert_eq!(resp.id, Uuid::nil());
        assert_eq!(resp.token_budget, 1_200_000);
        assert_eq!(resp.version, 1);
    }

    #[test]
    fn handoff_request_rejects_nil_tenant() {
        let req = HandoffRequest {
            target_agent_id: Uuid::new_v4(),
            reason: None,
            tenant_id: Uuid::nil(),
        };
        assert!(matches!(req.validate(), Err(ApiError::BadRequest(_))));
    }
}
