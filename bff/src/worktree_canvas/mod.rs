// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/mod.rs` — AI Worktree Graph Canvas BFF module
//! (per ULYS-57.4 T13, 2026-09-17, per DD-WORKTREE-CANVAS-001 §20-§21 +
//! WORKTREE-CANVAS-IMPL-PLAN-001 §4.13).
//!
//! 11 sub-module:
//! - [`mod.rs`]         — `WorktreeCanvasState` 共享状态 (10 字段) + build_router
//! - [`dto`]            — Request/Response/SSE Event/WebSocket 15 类型 + API Error
//! - [`permission`]     — RBAC 5 角色 + 3 档权限 + 18 Action 矩阵 (per DD §44)
//! - [`audit`]          — 100% audit middleware (per 守门 #13 d)
//! - [`idempotency`]    — Idempotency key cache (24h TTL, per FR-ACTION-006)
//! - [`sse`]            — 15 SSE 端点 (per IMPL-PLAN §4.13.3 + spec §4.5)
//! - [`ws`]             — 1 WebSocket 端点 (per IMPL-PLAN §4.13.4 + DD §21)
//! - [`controller`]     — 11 REST 端点 (per IMPL-PLAN §4.13.2 + DD §20)
//! - [`middleware`]     — RBAC + Idempotency + Audit 包装 (per IMPL-PLAN §4.13.5)
//! - [`openapi`]        — OpenAPI 3.0 spec (per AC-Q-6 + IMPL-PLAN §4.13)
//! - `tests`            — integration smoke tests
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #6 v2 + 守门 #7 + 守门 #13 + 守门 #14 v2 + 守门 #19 v19):
//! - 0 `unsafe` blocks.
//! - feature-gated `--features worktree_canvas` (守门 #19 v19 不破坏 V0.1 collab).
//! - 6-field API Error (per 守门 #6 v2).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2).

use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use uuid::Uuid;

pub mod audit;
pub mod controller;
pub mod dto;
pub mod idempotency;
pub mod middleware;
pub mod openapi;
pub mod permission;
pub mod sse;
pub mod ws;

pub use audit::{WorktreeAudit, WorktreeAuditEvent};
pub use controller::{bff_health, rest_routes};
pub use dto::{
    ActionRequest, ActionResponse, AgentSummary, BranchName, CommitSha, ConfirmDialog, CreateWorktreeRequest,
    HealthDeduction, HealthScoreDto, HealthSummaryDto, ImpactDto, ListWorktreesResponse,
    NlQueryRequest, NlQueryResponse, RepositoryDto, RiskResponse, SearchRequest, SearchResponse,
    TaskSummary, WorktreeApiError, WorktreeFieldChange, WorktreeGraphResponse, WorktreeResponse,
    WorktreeSseEvent, WsClientMessage, WsServerMessage, BffHealthDto, GraphEdgeDto, GraphNodeDto,
};
pub use idempotency::{IdempotencyCache, IdempotencyRecord};
pub use permission::{
    ActionDanger, PermissionLevel, RoleSet, WorktreeAction, WorktreePermission,
};
pub use sse::{sse_routes, WorktreeSseHub, SSE_KEEPALIVE_INTERVAL, SSE_ROUTE_COUNT};
pub use ws::{ws_routes, WS_HEARTBEAT_INTERVAL};

use permission::WorktreePermission as _WorktreePermission;

/// 共享状态 (per IMPL-PLAN §4.13 brief, 跨 11 REST + 15 SSE + 1 WS route 共享).
///
/// **10 字段** (per IMPL-PLAN §4.13.5):
/// 1. `permission`   — RBAC 守门
/// 2. `audit`        — 100% audit middleware
/// 3. `idempotency`  — Idempotency key cache
/// 4. `sse_hub`      — SSE broadcast hub
/// 5. `tenant_id`    — RLS 13 类 (per 守门 #13 b)
/// 6. `actor_id`     — 当前 user/actor (per 守门 #10)
/// 7. `started_at`   — BFF 实例启动时间 (per bff_health uptime)
/// 8. `repo_count`   — 占位 repo 数 (阶段 2 业务实装接 graph-core)
/// 9. `worktree_count` — 占位 worktree 数
/// 10. `placeholder`  — 阶段 2 业务实装接入 `graph-core::GraphRepository` 时移除
#[derive(Debug, Clone)]
pub struct WorktreeCanvasState {
    pub permission: Arc<WorktreePermission>,
    pub audit: Arc<WorktreeAudit>,
    pub idempotency: Arc<IdempotencyCache>,
    pub sse_hub: Arc<WorktreeSseHub>,
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub started_at: Instant,
    pub repo_count: u32,
    pub worktree_count: u32,
    pub placeholder: Option<()>,
}

impl WorktreeCanvasState {
    /// 构造一个新的 state (公开 API).
    pub fn new(
        tenant_id: Uuid,
        actor_id: Uuid,
        roles: RoleSet,
        canvas_level: PermissionLevel,
    ) -> Result<Self, WorktreeApiError> {
        if tenant_id.is_nil() {
            return Err(WorktreeApiError::bad_request(
                "tenant_id is nil",
                "WorktreeCanvasState::new",
            ));
        }
        let permission = Arc::new(WorktreePermission::new(tenant_id, roles, canvas_level));
        Ok(Self {
            permission,
            audit: Arc::new(WorktreeAudit::new()),
            idempotency: Arc::new(IdempotencyCache::new()),
            sse_hub: Arc::new(WorktreeSseHub::new()),
            tenant_id,
            actor_id,
            started_at: Instant::now(),
            repo_count: 0,
            worktree_count: 0,
            placeholder: None,
        })
    }

    /// From-parts constructor (测试用, 接受 pre-built components).
    pub fn from_parts(
        permission: WorktreePermission,
        audit: Arc<WorktreeAudit>,
        idempotency: Arc<IdempotencyCache>,
        sse_hub: Arc<WorktreeSseHub>,
        actor_id: Uuid,
    ) -> Result<Self, WorktreeApiError> {
        let tenant_id = permission.current_tenant();
        if tenant_id.is_nil() {
            return Err(WorktreeApiError::bad_request(
                "tenant_id is nil",
                "WorktreeCanvasState::from_parts",
            ));
        }
        Ok(Self {
            permission: Arc::new(permission),
            audit,
            idempotency,
            sse_hub,
            tenant_id,
            actor_id,
            started_at: Instant::now(),
            repo_count: 0,
            worktree_count: 0,
            placeholder: None,
        })
    }
}

/// Top-level router factory. Mounts 11 REST + 16 SSE (15 events + 1 health) + 1 WS routes.
pub fn build_router(state: Arc<WorktreeCanvasState>) -> Router {
    rest_routes(state.clone())
        .merge(sse_routes(state.clone()))
        .merge(ws_routes(state.clone()))
        // BFF instance-level health (per brief T13 守门 #5 + 守门 #6 PowerShell + Unix path compat).
        // 单独 Router, 不带 state (handler 不依赖 state).
        .merge(Router::new().route(
            "/v1/worktree-canvas/bff-health",
            axum::routing::get(controller::bff_health),
        ))
}

// =====================================================================
// Sub-module re-exports
// =====================================================================

pub use dto::{
    AgentId, BranchId, IssueId, PRId, RepoId, RiskScore, TaskId, TestId, UserId, WorktreeId,
};
pub use graph_core::state::{
    AgentResultState, AgentStatus, HumanState, IssueState, MachineState, MergeStrategy, PRState,
    SymbolKind, TaskStatus, TestState, TestStatus, TokenUsage,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }
    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    #[test]
    fn worktree_canvas_state_has_ten_fields() {
        let state = WorktreeCanvasState::new(tenant(), actor(), RoleSet::new(), PermissionLevel::View).unwrap();
        // 10 字段 (per mod doc).
        let fields = (
            &state.permission,
            &state.audit,
            &state.idempotency,
            &state.sse_hub,
            state.tenant_id,
            state.actor_id,
            state.started_at,
            state.repo_count,
            state.worktree_count,
            state.placeholder,
        );
        let _ = fields; // smoke
    }

    #[test]
    fn new_rejects_nil_tenant_id() {
        let r = WorktreeCanvasState::new(Uuid::nil(), actor(), RoleSet::new(), PermissionLevel::View);
        assert!(r.is_err());
    }

    #[test]
    fn build_router_smoke() {
        let state = Arc::new(
            WorktreeCanvasState::new(tenant(), actor(), RoleSet::new(), PermissionLevel::View).unwrap(),
        );
        let _ = build_router(state);
    }
}
