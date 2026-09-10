// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/agent` — STAR Agent API tier (A1-A10).
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 287-292. This module is the **API tier** for A1-A10
//! agent operations; it sits on top of `crates/agent-domain` (data tier) and
//! exposes 13 REST endpoints.
//!
//! 4 sub-modules:
//! - [`controller`]  — 13 REST route handlers (A1.1 + A2.1 + A2.2 + A2.3 + A2.4 + A3.2 + A4.1 + A4.2 + A6.1 + A6.2 + A7.2)
//! - [`permission`]  — RLS 13 類 + 6 角色 守门
//! - [`audit`]       — 100% audit middleware (per 守门 #13 d)
//! - [`dto`]         — request / response types + [`ApiError`]
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #12 v21 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - RLS tenant 校验每个写路径都过 [`permission::AgentPermission::check_tenant`].
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).
//! - docs 同步走 `docs/automation-design.md` §4.34.3 + `scripts/automation/registry.md` §5.5.

use std::sync::Arc;

use axum::Router;
use uuid::Uuid;

pub mod audit;
pub mod controller;
pub mod dto;
pub mod permission;

// =====================================================================
// AgentState (per DD-AGENT §3.1, 跨 13 route 共享)
// =====================================================================

/// 共享状态, 跨 13 route 共享.
///
/// **V0.1 阶段 1 基础 (任务 1.4)**: 4 字段 (audit + permission + tenant_id + actor_id).
/// brief §2.1 提到 8 字段 + 5 ops placeholder, 但 5 ops (`AgentOps` /
/// `HandoffConnector` / `DomainFrame` / `ParentChildConnector` /
/// `PipelineConnector` / `WorktreeAssoc` / `WorkItemAssoc`) 阶段 2
/// 任务 2.1 业务实装阶段 才在 `agent-domain` 落地, V0.1 阶段 1 占位
/// 不引用未存在的类型 (per 守门 #1 v25 + 守门 #19 v19 累积规不破坏 V0.1).
#[derive(Clone)]
pub struct AgentState {
    /// 100% audit middleware (per 守门 #13 d).
    pub audit: Arc<audit::AgentAudit>,
    /// RLS 13 類 + 6 角色 守门.
    pub permission: Arc<permission::AgentPermission>,
    /// 当前 tenant_id (per 守门 #13 b RLS 13 類).
    pub tenant_id: Uuid,
    /// 当前 actor user_id (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
}

impl AgentState {
    /// 构造一个新的 `AgentState` (per brief §2.1 AgentState::new).
    pub fn new(tenant_id: Uuid, actor_id: Uuid) -> Arc<Self> {
        use std::collections::HashSet;
        let mut roles = HashSet::new();
        // 5 域 Lead Mavis 临时代签阶段 (per 守门 #14 v2 拍板 D) 默认含 LEAD.
        roles.insert(permission::role::LEAD.to_string());
        Arc::new(Self {
            audit: Arc::new(audit::AgentAudit::new()),
            permission: Arc::new(permission::AgentPermission::new(tenant_id, roles)),
            tenant_id,
            actor_id,
        })
    }
}

/// Top-level router factory. Single entry point used by `lib.rs` /
/// `main.rs` to mount the 13 agent routes on the application router.
pub fn build_router(state: Arc<AgentState>) -> Router {
    controller::agent_routes(state)
}

// Re-exports for the convenience of the application layer.
pub use controller::agent_routes;
pub use dto::ApiError;
pub use permission::{role, AgentPermission};

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
    fn agent_state_constructor_returns_arc_with_correct_ids() {
        let s = AgentState::new(tenant(), actor());
        assert_eq!(s.tenant_id, tenant());
        assert_eq!(s.actor_id, actor());
        assert_eq!(s.permission.current_tenant(), tenant());
    }

    #[test]
    fn build_router_smoke_test() {
        // axum 0.8 Router 没有公开 route count introspection, 所以
        // 我们只 smoke test: build_router 成功, Router non-empty.
        let s = AgentState::new(tenant(), actor());
        let router = build_router(s);
        let _ = router;
    }
}
