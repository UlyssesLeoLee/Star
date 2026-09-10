// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff` — STAR BFF (Backend for Frontend) workspace.
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 301-307 + `docs/briefs/p3-d6-1-5-bff-skeleton.md` v0.1 §0.
//! BFF 跟 `crates/api` 平级, 走 envoy 独立 deployment (per 9/1 13:05 JST 偏好),
//! 通过 `svc://` 引用 `crates/api` 业务 svc.
//!
//! 1 sub-module (本任务 1.5):
//! - [`collaboration`] — 5 REST + 4 WSS endpoint (A12.1-A12.5) + 共享
//!   `CollaborationState` + permission + audit + WSS broadcast hub
//!
//! 0 业务方法实装 (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标), 留 P3-D.6 阶段 2
//! 任务 2.3 A12 业务实装. 0 真实 WSS 业务逻辑 (留 P3-D.6 阶段 3 集成 任务 3.2:
//! 0 CRDT 0 NATS 0 真实 broadcast subscribe).

#![warn(missing_docs)]

pub mod collaboration;

// Re-exports for the convenience of the application layer.
pub use collaboration::{build_router, CollaborationState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bff_module_exports_collaboration_submodule() {
        // Smoke test: `collaboration` module is publicly reachable.
        let _ = std::any::type_name::<collaboration::CollaborationState>();
    }

    #[test]
    fn bff_module_re_exports_collaboration_state() {
        // Smoke test: `CollaborationState` re-export is callable.
        let _: fn() -> collaboration::PermissionLevel = || collaboration::PermissionLevel::View;
    }
}
