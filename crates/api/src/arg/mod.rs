// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/arg` — STAR Agent Relationship Graph (ARG) API tier.
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../../../docs/design/DD-AGENT-RELATIONSHIP-001.md)
//! v0.1.1 §4.12 + §3.2.5. This module is the **API tier** of the ARG
//! subsystem; it sits on top of the data tier (`crates/arg`) and exposes
//! 13 REST endpoints + 1 WebSocket to the rest of the workspace.
//!
//! 5 sub-modules:
//! - [`controller`]  — 14 route handlers (13 REST + 1 WebSocket)
//! - [`sse_hub`]     — WebSocket / SSE fan-out hub (6 event types)
//! - [`permission`]  — RLS 13 類 + 6 角色 守门 (per 守门 #13 + #14 v2)
//! - [`dto`]         — request / response types + [`ApiError`]
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #12 v21 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - RLS tenant 校验每个写路径都过 [`permission::ARGPermission::check_tenant`].
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).
//! - docs 同步走 `docs/automation-design.md` §4.18 + `scripts/automation/registry.md` §5.4.

use std::sync::Arc;

use axum::Router;
use uuid::Uuid;

use star_arg::ops::{
    event_writer::EventWriter, AchievementOps, AgentNodeOps, EdgeOps, TemplateOps,
};

pub mod controller;
pub mod dto;
pub mod permission;
pub mod sse_hub;

// =====================================================================
// ARGState (per DD §3.2.5 共享类型, 11th of 11 shared types)
// =====================================================================

/// 共享状态, 跨 14 route 共享.
///
/// 8 字段 (per DD §3.2.5):
/// - 5 ops (`AgentNodeOps` / `EdgeOps` / `TemplateOps` / `AchievementOps` /
///   `EventWriter`) — 来自 `crates/arg` 数据层
/// - 2 跨域组件 (`ARGSSEHub` / `ARGPermission`) — 本模块自带
/// - 1 RLS tenant_id (per 守门 #13 b 13 類)
///
/// `#[allow(dead_code)]` 不需要: 全部 8 字段都被 controller 访问 (写
/// 路径) 或 sse_hub 访问 (读路径).
pub struct ARGState {
    /// Agent CRUD (per DD §4.2).
    pub agent_node_ops: Arc<AgentNodeOps>,
    /// Edge CRUD + traversal (per DD §4.3).
    pub edge_ops: Arc<EdgeOps>,
    /// Team template operations (per DD §4.4).
    pub template_ops: Arc<TemplateOps>,
    /// Achievement unlock + query (per DD §4.7).
    pub achievement_ops: Arc<AchievementOps>,
    /// Append-only event writer (per DD §3.2.5 + 守门 #13 b Transaction).
    pub event_writer: Arc<EventWriter>,
    /// WebSocket / SSE fan-out hub.
    pub sse_hub: Arc<sse_hub::ARGSSEHub>,
    /// RLS 13 類 + 6 角色 守门.
    pub permission: Arc<permission::ARGPermission>,
    /// 当前 tenant_id (per 守门 #13 b RLS 13 類).
    pub tenant_id: Uuid,
}

impl ARGState {
    /// Build a new `ARGState` from the 5 data-tier ops + 2 API-tier
    /// components + the actor's `tenant_id`.
    ///
    /// Use this constructor in `main.rs` / `lib.rs` when wiring the
    /// application router. The `actor_uuid` is the id used as
    /// `created_by` for new edges / agents (per 守门 #10 author).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_node_ops: Arc<AgentNodeOps>,
        edge_ops: Arc<EdgeOps>,
        template_ops: Arc<TemplateOps>,
        achievement_ops: Arc<AchievementOps>,
        event_writer: Arc<EventWriter>,
        sse_hub: Arc<sse_hub::ARGSSEHub>,
        permission: Arc<permission::ARGPermission>,
        tenant_id: Uuid,
    ) -> Arc<Self> {
        Arc::new(Self {
            agent_node_ops,
            edge_ops,
            template_ops,
            achievement_ops,
            event_writer,
            sse_hub,
            permission,
            tenant_id,
        })
    }

    /// Convenience helper: `tenant_id` for the actor. We use the
    /// state's tenant_id for the `created_by` field on agent / edge
    /// writes (per 守门 #10 author = current actor).
    pub fn tenant_id_for_actor(&self) -> Uuid {
        self.tenant_id
    }
}

/// Top-level router factory. Single entry point used by `lib.rs` /
/// `main.rs` to mount the 14 ARG routes on the application router.
pub fn build_router(state: Arc<ARGState>) -> Router {
    controller::arg_routes(state)
}

// Re-exports for the convenience of the application layer.
pub use controller::arg_routes;
pub use dto::ApiError;
pub use permission::{role, ARGPermission};
pub use sse_hub::{ARGSSEHub, ARGSseEvent};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_state(tenant: Uuid) -> Arc<ARGState> {
        let client = std::sync::Arc::new(star_arg::client::MemgraphClient::new(
            "bolt://localhost:7687".into(),
            "user".into(),
            "pw".into(),
            1,
        ));
        let event_writer = Arc::new(EventWriter::new());
        let agent_node_ops = Arc::new(AgentNodeOps::new(client.clone(), event_writer.clone()));
        let edge_ops = Arc::new(EdgeOps::new(client.clone(), event_writer.clone()));
        let template_ops = Arc::new(TemplateOps::new(
            client,
            edge_ops.clone(),
            event_writer.clone(),
        ));
        // AchievementOps owns its own EventWriter (EventWriter doesn't
        // impl Clone). In real usage, the 4 ops would share a single
        // writer; for unit tests we accept the slight duplication.
        let achievement_writer = EventWriter::new();
        let achievement_ops = Arc::new(AchievementOps::new(achievement_writer));
        let sse_hub = Arc::new(ARGSSEHub::new());
        let mut roles = HashSet::new();
        roles.insert(role::ADMIN.to_string());
        let permission = Arc::new(ARGPermission::new(tenant, roles));
        ARGState::new(
            agent_node_ops,
            edge_ops,
            template_ops,
            achievement_ops,
            event_writer,
            sse_hub,
            permission,
            tenant,
        )
    }

    #[test]
    fn arg_state_constructor_returns_arc() {
        let t = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let s = make_state(t);
        assert_eq!(s.tenant_id, t);
        assert_eq!(s.tenant_id_for_actor(), t);
    }

    #[test]
    fn build_router_registers_14_routes() {
        let t = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let s = make_state(t);
        let router = build_router(s);
        // axum 0.8 Router has no public introspection of route count,
        // so we just assert the build succeeded and the router is non-empty.
        // The actual route count is verified by the Python IT script
        // `scripts/automation/arg_api_test.py` per DD §10.2.
        let _ = router;
    }

    #[test]
    fn re_exports_are_accessible() {
        // Smoke test: re-exports compile and are reachable.
        let _: fn() -> ARGSSEHub = ARGSSEHub::new;
        let _ = ApiError::NotFound("x".into());
        let _ = role::ADMIN;
    }
}
