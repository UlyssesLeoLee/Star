// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/audit.rs` — 100% audit middleware for AI Worktree
//! Graph Canvas BFF (per 守门 #13 d + DD-WORKTREE-CANVAS-001 §43 + IMPL-PLAN §4.13.5).
//!
//! Per spec §4.4 Action 矩阵 + DD §43: **100% Destructive + Warning + Lock/Unlock
//! 写路径过 audit**, Safe Action 可选. 阶段 1 占位: 0 真实 WORM 写, 阶段 2 业务实装
//! 阶段 落地真实 `AuditEventSink` 写 PostgreSQL `worktree_action_audit` 表
//! (per IMPL-PLAN §5 W/T/M 100% 覆盖: T 类 audit 表).
//!
//! 跟 V0.1 `bff/src/collaboration/audit.rs` 同形 (6 字段 + 占位 sink), 独立定义避免
//! cross-tier 紧耦合 (per 守门 #9 #1 禁跨层 leak + 守门 #11 缺标比错标).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 d + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - 100% Destructive / Warning / Lock/Unlock 写路径必过 `WorktreeAudit::write_event`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::WorktreeApiError;
use super::permission::{ActionDanger, WorktreeAction};

/// 6 字段 `WorktreeAuditEvent` (per brief T13 RBAC + DD §43 + IMPL-PLAN §4.13.5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeAuditEvent {
    /// 全局唯一 ID.
    pub id: Uuid,
    /// 关联 worktree ID (None for repo-level actions).
    pub worktree_id: Option<Uuid>,
    /// 关联 repository ID (per DD §43).
    pub repository_id: Option<Uuid>,
    /// 操作者 user/actor ID (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// Action 名 (e.g. "Merge" / "Delete" / "Lock").
    pub action: String,
    /// 危险级别 (per spec §4.4 矩阵).
    pub danger: ActionDanger,
    /// 操作时间 (UTC).
    pub at: DateTime<Utc>,
    /// idempotency key (None for read-only).
    pub idempotency_key: Option<Uuid>,
    /// 任意 JSON metadata (action params snapshot + actor IP + user agent).
    pub metadata: serde_json::Value,
}

impl WorktreeAuditEvent {
    /// 构造一个新的 audit event.
    pub fn new(
        worktree_id: Option<Uuid>,
        repository_id: Option<Uuid>,
        actor_id: Uuid,
        action: WorktreeAction,
        idempotency_key: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            worktree_id,
            repository_id,
            actor_id,
            action: action.as_str().to_string(),
            danger: action.danger(),
            at: Utc::now(),
            idempotency_key,
            metadata,
        }
    }

    /// 6 字段守门 (per 守门 #6 v2 派生): 所有必填字段非空 / 非 nil.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.action.is_empty() {
            return Err("action is empty");
        }
        if self.actor_id.is_nil() {
            return Err("actor_id is nil");
        }
        if self.id.is_nil() {
            return Err("id is nil");
        }
        Ok(())
    }
}

/// Audit sink placeholder (per brief T13 + IMPL-PLAN §4.13.5).
///
/// 阶段 1: 仅 validate + 0 真实 DB 写. 阶段 2 业务实装落地 `worktree_action_audit` 表.
#[derive(Debug, Clone, Default)]
pub struct WorktreeAudit {
    /// Placeholder for future sink.
    _placeholder: (),
}

impl WorktreeAudit {
    /// 构造一个新的 audit sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// 异步写 audit event (per 守门 #13 d 100% audit).
    ///
    /// 阶段 1: 仅 validate event 字段, 返回 `Ok(())`. 阶段 2 业务实装落地真实
    /// WORM append-only `worktree_action_audit` 表写.
    pub async fn write_event(&self, event: WorktreeAuditEvent) -> Result<(), WorktreeApiError> {
        event.validate().map_err(|e| {
            WorktreeApiError::bad_request(e, "audit::write_event::validate")
        })?;
        // 阶段 1: 占位 0 真实 DB 写. 阶段 2 业务实装落地.
        Ok(())
    }

    /// 守门 #13 d 验证 helper: 哪些 action 必须写 audit.
    pub fn is_required(action: WorktreeAction) -> bool {
        matches!(action.danger(), ActionDanger::Warning | ActionDanger::Destructive)
            || matches!(action, WorktreeAction::Lock | WorktreeAction::Unlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    #[test]
    fn audit_event_new_fills_id_and_at() {
        let wt = Uuid::new_v4();
        let repo = Uuid::new_v4();
        let e = WorktreeAuditEvent::new(
            Some(wt),
            Some(repo),
            actor(),
            WorktreeAction::Merge,
            Some(Uuid::new_v4()),
            serde_json::json!({"confirm": true}),
        );
        assert_eq!(e.worktree_id, Some(wt));
        assert_eq!(e.repository_id, Some(repo));
        assert_eq!(e.actor_id, actor());
        assert_eq!(e.action, "Merge");
        assert_eq!(e.danger, ActionDanger::Destructive);
        assert!(!e.id.is_nil());
    }

    #[test]
    fn audit_event_validate_rejects_empty_action() {
        let mut e = WorktreeAuditEvent::new(
            None,
            None,
            actor(),
            WorktreeAction::Focus,
            None,
            serde_json::json!({}),
        );
        e.action = String::new();
        assert!(e.validate().is_err());
    }

    #[test]
    fn audit_event_validate_rejects_nil_actor() {
        let mut e = WorktreeAuditEvent::new(
            None,
            None,
            actor(),
            WorktreeAction::Focus,
            None,
            serde_json::json!({}),
        );
        e.actor_id = Uuid::nil();
        assert!(e.validate().is_err());
    }

    #[tokio::test]
    async fn write_event_accepts_well_formed() {
        let audit = WorktreeAudit::new();
        let event = WorktreeAuditEvent::new(
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
            actor(),
            WorktreeAction::Delete,
            Some(Uuid::new_v4()),
            serde_json::json!({}),
        );
        assert!(audit.write_event(event).await.is_ok());
    }

    #[test]
    fn is_required_audit_matches_danger() {
        // Destructive + Warning + Lock + Unlock 必写, Safe 可选.
        assert!(WorktreeAudit::is_required(WorktreeAction::Merge)); // Destructive
        assert!(WorktreeAudit::is_required(WorktreeAction::Delete)); // Destructive
        assert!(WorktreeAudit::is_required(WorktreeAction::SyncMain)); // Warning
        assert!(WorktreeAudit::is_required(WorktreeAction::Lock)); // Safe but required
        assert!(WorktreeAudit::is_required(WorktreeAction::Unlock)); // Safe but required
        // 其他 Safe: 不必写 (e.g. Focus, OpenWorktree).
        assert!(!WorktreeAudit::is_required(WorktreeAction::Focus));
    }
}
