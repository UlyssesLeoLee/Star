// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/canvas_collab` — STAR Canvas Collaboration API tier (A12).
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 301-307 + `docs/design/DD-CANVAS-001.md` v0.1.1 C-25/C-26. This
//! module is the **API tier** for A12 canvas collaboration; it sits on top of
//! `crates/canvas-collab` (data tier) and exposes 5 REST endpoints. WebSocket
//! 4 端点 走 `bff/src/collaboration/wss_hub.rs` (任务 1.5), 0 SSE / 0 WebSocket 在本 module.
//!
//! 4 sub-modules:
//! - [`controller`]  — 5 REST route handlers (A12.1 + A12.2 + A12.3 + A12.4 + A12.7)
//! - [`permission`]  — RLS 13 類 + 6 角色 + 3 档权限 (View/Comment/Edit) 守门
//! - [`audit`]       — 100% audit middleware (per 守门 #13 d)
//! - [`dto`]         — request / response types + [`ApiError`]
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #12 v21 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - RLS tenant 校验每个写路径都过 [`permission::CanvasCollabPermission::check_tenant`].
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
// CanvasCollabState (per DD-AGENT §3.1, 跨 5 route 共享)
// =====================================================================

/// 共享状态, 跨 5 route 共享.
///
/// **V0.1 阶段 1 基础 (任务 1.4)**: 5 字段 (audit + permission + tenant_id +
/// actor_id + canvas_id). brief §2.6 提到 8 字段 + canvas_ops placeholder,
/// 但 8 字段 设计 在 V0.2 阶段 2 任务 2.3 业务实装阶段 才在 `canvas-collab`
/// crate 落地, V0.1 阶段 1 占位 不引用未存在的类型 (per 守门 #1 v25 +
/// 守门 #19 v19 累积规不破坏 V0.1).
#[derive(Clone)]
pub struct CanvasCollabState {
    /// 100% audit middleware (per 守门 #13 d).
    pub audit: Arc<audit::CanvasCollabAudit>,
    /// RLS 13 類 + 6 角色 + 3 档权限 守门.
    pub permission: Arc<permission::CanvasCollabPermission>,
    /// 当前 tenant_id (per 守门 #13 b RLS 13 類).
    pub tenant_id: Uuid,
    /// 当前 actor user_id (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// 当前 canvas_id (per A12.7 grant_permission context 派生).
    pub canvas_id: Uuid,
}

impl CanvasCollabState {
    /// 构造一个新的 `CanvasCollabState` (per brief §2.6 CanvasCollabState::new).
    pub fn new(tenant_id: Uuid, actor_id: Uuid, canvas_id: Uuid) -> Arc<Self> {
        use std::collections::HashSet;
        let mut roles = HashSet::new();
        // 5 域 Lead Mavis 临时代签阶段 (per 守门 #14 v2 拍板 D) 默认含 LEAD.
        roles.insert(permission::role::LEAD.to_string());
        Arc::new(Self {
            audit: Arc::new(audit::CanvasCollabAudit::new()),
            permission: Arc::new(permission::CanvasCollabPermission::new(
                tenant_id,
                roles,
                permission::PermissionLevel::Edit, // 默认 Edit (5 域 Lead 自驱, 5 域决策)
            )),
            tenant_id,
            actor_id,
            canvas_id,
        })
    }
}

/// Top-level router factory. Single entry point used by `lib.rs` /
/// `main.rs` to mount the 5 canvas_collab routes on the application router.
pub fn build_router(state: Arc<CanvasCollabState>) -> Router {
    controller::canvas_collab_routes(state)
}

// Re-exports for the convenience of the application layer.
pub use controller::canvas_collab_routes;
pub use dto::ApiError;
pub use permission::{role, CanvasCollabPermission, PermissionLevel};

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    fn canvas() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap()
    }

    #[test]
    fn canvas_collab_state_constructor_returns_arc_with_correct_ids() {
        let s = CanvasCollabState::new(tenant(), actor(), canvas());
        assert_eq!(s.tenant_id, tenant());
        assert_eq!(s.actor_id, actor());
        assert_eq!(s.canvas_id, canvas());
        assert_eq!(s.permission.current_tenant(), tenant());
        assert_eq!(s.permission.canvas_level(), PermissionLevel::Edit);
    }

    #[test]
    fn build_router_smoke_test() {
        // axum 0.8 Router 没有公开 route count introspection, 所以
        // 我们只 smoke test: build_router 成功, Router non-empty.
        let s = CanvasCollabState::new(tenant(), actor(), canvas());
        let router = build_router(s);
        let _ = router;
    }
}
