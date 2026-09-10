// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/collaboration` — STAR BFF Canvas Collaboration tier (A12) (per DD-AGENT §3.1 + brief §0).
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 301-307 + `docs/briefs/p3-d6-1-5-bff-skeleton.md` §0.
//! This module is the **BFF tier** for A12 canvas collaboration; it sits
//! **alongside** `crates/api/src/canvas_collab` (BFF 跟 API 平级 0 反向依赖,
//! per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标) and exposes 5 REST endpoints
//! + 4 WSS endpoints (per A12.1-A12.5).
//!
//! 6 sub-modules (per DD-AGENT §3.1 line 301-307):
//! - [`mod.rs`]     — `CollaborationState` shared across 5 REST + 4 WSS routes
//! - [`controller`] — 5 REST route handlers (A12.1 + A12.3 + A12.5 + A12.2 + A12.4)
//! - [`wss_hub`]    — 4 WSS route handlers + `CollaborationWssHub` broadcast channel
//! - [`permission`] — RLS 13 類 + 6 角色 + 3 档权限 (View/Comment/Edit) 守门
//! - [`audit`]      — 100% audit middleware (per 守门 #13 d)
//! - [`dto`]        — request / response / WSS message types + [`ApiError`]
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #12 v21 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - RLS tenant 校验每个写路径都过 [`permission::CollaborationPermission::check_tenant`].
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).
//! - docs 同步走 `docs/automation-design.md` §4.34.4 + `scripts/automation/registry.md` (跨 session 续).

use std::sync::Arc;

use axum::Router;
use canvas_collab::PresenceCursor;
use uuid::Uuid;

pub mod audit;
pub mod controller;
pub mod dto;
pub mod permission;
pub mod wss_hub;

// =====================================================================
// CollaborationState (per DD-AGENT §3.1, 跨 5 REST + 4 WSS route 共享)
// =====================================================================

/// 共享状态, 跨 5 REST + 4 WSS route 共享 (per brief §2.3).
///
/// **V0.1 阶段 1 基础 (任务 1.5)**: 7 字段 (audit + permission + wss_hub +
/// tenant_id + actor_id + canvas_id + presence_placeholder). brief §2.3 提到
/// 8-10 字段 含 `canvas_ops: Arc<CanvasOps>`, 但 `canvas_collab::CanvasOps`
/// 类型在 V0.2 阶段 2 任务 2.3 业务实装阶段 才在 `canvas-collab` crate 落地
/// (跟 V0.1 `crates/api/src/canvas_collab/mod.rs` 5 字段 0 引用未存在类型 模式一致,
/// per 守门 #1 v25 + 守门 #19 v19 累积规不破坏 V0.1).
#[derive(Clone)]
pub struct CollaborationState {
    /// 100% audit middleware (per 守门 #13 d).
    pub audit: Arc<audit::CollaborationAudit>,
    /// RLS 13 類 + 6 角色 + 3 档权限 守门 (BFF middleware 强制 per 9/1 13:05 JST 偏好).
    pub permission: Arc<permission::CollaborationPermission>,
    /// WSS fan-out broadcast hub (per A12.1-A12.4 4 WSS endpoint, 阶段 1 占位 0 业务).
    pub wss_hub: Arc<wss_hub::CollaborationWssHub>,
    /// 当前 tenant_id (per 守门 #13 b RLS 13 類).
    pub tenant_id: Uuid,
    /// 当前 actor user_id (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// 当前 canvas_id (per A12.7 grant_permission context 派生).
    pub canvas_id: Uuid,
    /// Presence placeholder (V0.2 阶段 2 任务 2.3 落地 presence list cache,
    /// 阶段 1 占位 `None`, 0 业务方法).
    pub presence_placeholder: Option<PresenceCursor>,
}

impl CollaborationState {
    /// 构造一个新的 `CollaborationState` (per brief §2.3 `CollaborationState::new`).
    pub fn new(
        audit: Arc<audit::CollaborationAudit>,
        permission: Arc<permission::CollaborationPermission>,
        wss_hub: Arc<wss_hub::CollaborationWssHub>,
        tenant_id: Uuid,
        actor_id: Uuid,
        canvas_id: Uuid,
    ) -> Arc<Self> {
        Arc::new(Self {
            audit,
            permission,
            wss_hub,
            tenant_id,
            actor_id,
            canvas_id,
            presence_placeholder: None,
        })
    }
}

/// Top-level router factory. Single entry point used by `lib.rs` / `main.rs`
/// to mount the 5 REST + 4 WSS routes on the application router.
pub fn build_router(state: Arc<CollaborationState>) -> Router {
    controller::collaboration_routes(state)
}

// Re-exports for the convenience of the application layer.
pub use controller::collaboration_routes;
pub use dto::{ApiError, CollaborationWssEvent};
pub use permission::{role, CollaborationPermission, PermissionLevel};
pub use wss_hub::CollaborationWssHub;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
    fn collaboration_state_constructor_returns_arc_with_correct_ids() {
        let mut roles = HashSet::new();
        roles.insert(permission::role::LEAD.to_string());
        let permission = Arc::new(permission::CollaborationPermission::new(
            tenant(),
            roles,
            permission::PermissionLevel::Edit,
        ));
        let audit = Arc::new(audit::CollaborationAudit::new());
        let wss_hub = Arc::new(wss_hub::CollaborationWssHub::new());
        let s = CollaborationState::new(audit, permission, wss_hub, tenant(), actor(), canvas());
        assert_eq!(s.tenant_id, tenant());
        assert_eq!(s.actor_id, actor());
        assert_eq!(s.canvas_id, canvas());
        assert_eq!(s.permission.current_tenant(), tenant());
        assert_eq!(s.permission.canvas_level(), PermissionLevel::Edit);
        assert!(s.presence_placeholder.is_none());
    }

    #[test]
    fn build_router_smoke_test() {
        // axum 0.8 Router 没有公开 route count introspection, 所以
        // 我们只 smoke test: build_router 成功, Router non-empty.
        let mut roles = HashSet::new();
        roles.insert(permission::role::LEAD.to_string());
        let permission = Arc::new(permission::CollaborationPermission::new(
            tenant(),
            roles,
            permission::PermissionLevel::Edit,
        ));
        let audit = Arc::new(audit::CollaborationAudit::new());
        let wss_hub = Arc::new(wss_hub::CollaborationWssHub::new());
        let s = CollaborationState::new(audit, permission, wss_hub, tenant(), actor(), canvas());
        let router = build_router(s);
        let _ = router;
    }

    #[test]
    fn re_exports_are_accessible() {
        // Smoke test: re-exports compile and are reachable.
        let _: fn() -> wss_hub::CollaborationWssHub = wss_hub::CollaborationWssHub::new;
        let _ = ApiError::NotFound("x".into());
        let _ = role::ADMIN;
    }
}
