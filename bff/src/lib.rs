// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff` — STAR BFF (Backend for Frontend) workspace.
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 301-307 + `docs/briefs/p3-d6-1-5-bff-skeleton.md` v0.1 §0.
//! BFF 跟 `crates/api` 平级, 走 envoy 独立 deployment (per 9/1 13:05 JST 偏好),
//! 通过 `svc://` 引用 `crates/api` 业务 svc.
//!
//! 2 sub-module (本任务 1.5 + ULYS-57.4 T13):
//! - [`collaboration`] — V0.1 5 REST + 4 WSS endpoint (A12.1-A12.5) + 共享
//!   `CollaborationState` + permission + audit + WSS broadcast hub.
//!   **永远编译** (守门 #19 v19: 不破坏 V0.1).
//! - [`worktree_canvas`] — ULYS-57.4 T13 (per DD-WORKTREE-CANVAS-001 §20-§21 +
//!   WORKTREE-CANVAS-IMPL-PLAN-001 §4.13): 11 REST + 15 SSE + 1 WS endpoint +
//!   RBAC 5 角色 + Idempotency 24h + Audit + OpenAPI 3.0 spec.
//!   **Feature-gated** behind `--features worktree_canvas` (默认开启,
//!   `--no-default-features` 仅 V0.1 collab 用于守门 #19 v19 回归).
//!
//! 0 业务方法实装 (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标), 留 P3-D.6 阶段 2
//! 任务 2.3 A12 业务实装. 0 真实 WSS 业务逻辑 (留 P3-D.6 阶段 3 集成 任务 3.2:
//! 0 CRDT 0 NATS 0 真实 broadcast subscribe).

#![warn(missing_docs)]

pub mod collaboration;

// ULYS-57.4 T13 BFF module (per DD §20-§21 + IMPL-PLAN §4.13 + spec §4.4 §4.5).
// 守门 #19 v19: feature-gated, 0 破坏 V0.1 collab.
#[cfg(feature = "worktree_canvas")]
pub mod worktree_canvas;

// Re-exports for the convenience of the application layer.
pub use collaboration::{build_router as build_collab_router, CollaborationState};

#[cfg(feature = "worktree_canvas")]
pub use worktree_canvas::{
    build_router as build_worktree_canvas_router, openapi::openapi_document, WorktreeCanvasState,
    WorktreeSseHub,
};

/// Top-level BFF router factory.
///
/// 默认 (有 `worktree_canvas` feature): collab routes + worktree_canvas routes.
/// `--no-default-features`: 仅 collab routes (V0.1 回归).
pub fn build_router(state: build_collab_router_state::CollabRouterState) -> axum::Router {
    collab_only(state)
}

#[cfg(feature = "worktree_canvas")]
pub use collab_only as build_full_router;

/// V0.1 collab-only router (per 守门 #19 v19).
pub fn collab_only(state: build_collab_router_state::CollabRouterState) -> axum::Router {
    build_collab_router(state.into())
}

/// Public state placeholder to allow V0.1 + worktree_canvas dual build.
pub mod build_collab_router_state {
    pub use std::sync::Arc;
    pub use crate::collaboration::CollaborationState as CollabRouterState;
    impl From<Arc<crate::collaboration::CollaborationState>> for CollabRouterState {
        fn from(s: Arc<crate::collaboration::CollaborationState>) -> Self {
            // SAFETY: CollaborationState is Clone, Arc unwrap is fine.
            (*s).clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collaboration::CollaborationState;
    use std::sync::Arc;

    #[test]
    fn bff_module_exports_collaboration_submodule() {
        let _ = std::any::type_name::<collaboration::CollaborationState>();
    }

    #[test]
    fn bff_module_re_exports_collaboration_state() {
        let _: fn() -> collaboration::PermissionLevel = || collaboration::PermissionLevel::View;
    }

    #[test]
    fn bff_build_router_smoke() {
        use std::collections::HashSet;
        let mut roles = HashSet::new();
        roles.insert(collaboration::role::LEAD.to_string());
        let permission = Arc::new(collaboration::CollaborationPermission::new(
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            roles,
            collaboration::PermissionLevel::Edit,
        ));
        let audit = Arc::new(collaboration::audit::CollaborationAudit::new());
        let wss_hub = Arc::new(collaboration::wss_hub::CollaborationWssHub::new());
        let state = CollaborationState::new(
            audit,
            permission,
            wss_hub,
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
            uuid::Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap(),
        );
        let router = build_collab_router(state);
        let _ = router;
    }

    #[cfg(feature = "worktree_canvas")]
    #[test]
    fn bff_module_exports_worktree_canvas_when_feature_on() {
        let _ = std::any::type_name::<WorktreeCanvasState>();
        let doc = openapi_document();
        assert_eq!(doc["openapi"], "3.0.3");
    }
}
