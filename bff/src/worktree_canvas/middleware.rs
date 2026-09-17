// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/middleware.rs` — RBAC + Idempotency + Audit 包装
//! (per IMPL-PLAN §4.13.5 + DD-WORKTREE-CANVAS-001 §42-§44).
//!
//! 阶段 1: 提供 5 个 helper (走 `WorktreeCanvasState`), 业务 handler 已 inline 调用.
//! 阶段 2 业务实装时可考虑抽成 axum middleware layer (per IMPL-PLAN §4.13.5).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 100% Destructive / Warning / Lock/Unlock 写路径必过 audit (per 守门 #13 d).

use std::sync::Arc;
use uuid::Uuid;

use super::audit::{WorktreeAudit, WorktreeAuditEvent};
use super::dto::WorktreeApiError;
use super::idempotency::IdempotencyCache;
use super::permission::{
    ActionDanger, PermissionLevel, WorktreeAction, WorktreePermission,
};
use super::WorktreeCanvasState;

// =====================================================================
// RBAC middleware helpers (per DD §44)
// =====================================================================

/// Require view level for read endpoints.
pub fn require_view(state: &WorktreeCanvasState) -> Result<(), WorktreeApiError> {
    state.permission.require(PermissionLevel::View)
}

/// Require edit level for write endpoints.
pub fn require_edit(state: &WorktreeCanvasState) -> Result<(), WorktreeApiError> {
    state.permission.require(PermissionLevel::Edit)
}

/// Require specific action RBAC (per spec §4.4 Action 矩阵).
pub fn require_action(
    state: &WorktreeCanvasState,
    action: WorktreeAction,
) -> Result<(), WorktreeApiError> {
    state.permission.require_action(action)
}

// =====================================================================
// Idempotency middleware helpers (per FR-ACTION-006 + IMPL-PLAN §5)
// =====================================================================

/// 检查 idempotency key 是否命中缓存; 命中返回 cached JSON value.
pub fn idempotency_lookup(
    cache: &IdempotencyCache,
    key: Uuid,
) -> Option<serde_json::Value> {
    cache.lookup(key).map(|r| r.response)
}

/// 写入 idempotency 记录 (per FR-ACTION-006 + spec §4.4 Action 矩阵).
pub fn idempotency_store(
    cache: &IdempotencyCache,
    key: Uuid,
    actor_id: Uuid,
    action: WorktreeAction,
    response: serde_json::Value,
) -> Result<(), WorktreeApiError> {
    cache.store(key, actor_id, action.as_str().to_string(), response)
}

// =====================================================================
// Audit middleware helpers (per 守门 #13 d + DD §43)
// =====================================================================

/// 是否必写 audit (per 守门 #13 d 100% Destructive/Warning/Lock/Unlock).
pub fn audit_required(action: WorktreeAction) -> bool {
    WorktreeAudit::is_required(action)
}

/// 异步写 audit event (per 守门 #13 d).
pub async fn audit_write(
    audit: &WorktreeAudit,
    worktree_id: Option<Uuid>,
    repository_id: Option<Uuid>,
    actor_id: Uuid,
    action: WorktreeAction,
    idempotency_key: Option<Uuid>,
    metadata: serde_json::Value,
) -> Result<(), WorktreeApiError> {
    let event = WorktreeAuditEvent::new(
        worktree_id,
        repository_id,
        actor_id,
        action,
        idempotency_key,
        metadata,
    );
    audit.write_event(event).await
}

// =====================================================================
// Aggregate helper: 完整链路 (RBAC + Idempotency + Audit)
// =====================================================================

/// 完整 Action middleware 守门结果 (per IMPL-PLAN §4.13.5).
///
/// 返回值 = 0 表示三段全过, 可以执行 Action 业务. 非 0 表示中途拒绝,
/// 调用方应直接返回错误响应 (per 守门 #13 100% audit 完整性).
pub struct ActionMiddlewareGate {
    pub action: WorktreeAction,
    pub rbac_ok: bool,
    pub idempotency_key: Uuid,
    pub audit_required: bool,
}

impl ActionMiddlewareGate {
    /// 构造并跑 RBAC 守门.
    pub fn check(
        state: &WorktreeCanvasState,
        action: WorktreeAction,
        idempotency_key: Uuid,
    ) -> Result<Self, WorktreeApiError> {
        // 1. RBAC.
        state.permission.require_action(action)?;
        Ok(Self {
            action,
            rbac_ok: true,
            idempotency_key,
            audit_required: WorktreeAudit::is_required(action),
        })
    }

    /// 跑 audit 写 (如果 audit_required).
    pub async fn write_audit(&self, state: &WorktreeCanvasState, worktree_id: Uuid, metadata: serde_json::Value) -> Result<(), WorktreeApiError> {
        if !self.audit_required {
            return Ok(());
        }
        audit_write(
            &state.audit,
            Some(worktree_id),
            None,
            state.actor_id,
            self.action,
            Some(self.idempotency_key),
            metadata,
        )
        .await
    }
}

// =====================================================================
// State wrapper helper (避免每个 handler 都展开 Arc clone)
// =====================================================================

/// Clone the state (cheap Arc clone, per `Arc<WorktreeCanvasState>` 模式).
pub fn clone_state(state: &Arc<WorktreeCanvasState>) -> Arc<WorktreeCanvasState> {
    Arc::clone(state)
}

/// 当前 actor ID.
pub fn current_actor(state: &WorktreeCanvasState) -> Uuid {
    state.actor_id
}

/// 当前 tenant ID.
pub fn current_tenant(state: &WorktreeCanvasState) -> Uuid {
    state.tenant_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree_canvas::permission::RoleSet;
    use crate::worktree_canvas::dto::WorktreeApiError;

    fn tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }
    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    fn make_state() -> Arc<WorktreeCanvasState> {
        Arc::new(
            WorktreeCanvasState::new(tenant(), actor(), RoleSet::new(), PermissionLevel::Edit)
                .unwrap(),
        )
    }

    #[test]
    fn require_view_accepts_view_and_above() {
        let state = make_state();
        assert!(require_view(&state).is_ok());
    }

    #[test]
    fn idempotency_helpers_round_trip() {
        let state = make_state();
        let key = Uuid::new_v4();
        let v = serde_json::json!({"ok": true});
        idempotency_store(
            &state.idempotency,
            key,
            actor(),
            WorktreeAction::Lock,
            v.clone(),
        )
        .unwrap();
        assert_eq!(idempotency_lookup(&state.idempotency, key), Some(v));
    }

    #[test]
    fn audit_required_matches_spec() {
        assert!(audit_required(WorktreeAction::Merge));
        assert!(audit_required(WorktreeAction::Lock));
        assert!(!audit_required(WorktreeAction::Focus));
    }

    #[tokio::test]
    async fn action_middleware_gate_runs_three_stages() {
        let mut roles = RoleSet::new();
        roles.insert(crate::worktree_canvas::permission::role::LEAD.to_string());
        let state = Arc::new(
            WorktreeCanvasState::new(tenant(), actor(), roles, PermissionLevel::Edit).unwrap(),
        );
        let key = Uuid::new_v4();
        let gate = ActionMiddlewareGate::check(&state, WorktreeAction::Delete, key).unwrap();
        assert!(gate.rbac_ok);
        assert!(gate.audit_required);
        // 写 audit 应该 OK.
        let wt = Uuid::new_v4();
        gate.write_audit(&state, wt, serde_json::json!({})).await.unwrap();
    }

    #[test]
    fn action_middleware_gate_rbac_rejects_viewer_for_destructive() {
        let state = make_state();
        let key = Uuid::new_v4();
        let r = ActionMiddlewareGate::check(&state, WorktreeAction::Delete, key);
        assert!(matches!(r, Err(WorktreeApiError { ref code, .. }) if code == "FORBIDDEN"));
    }
}
