// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/canvas_collab/audit.rs` — 100% audit middleware for canvas_collab (per 守门 #13 d + DD-AGENT §4.14 + DD-001 C-26).
//!
//! Per brief §2.9: 6 字段 `CanvasCollabAuditEvent` + `CanvasCollabAudit` sink
//! placeholder + `write_event` 异步方法. 阶段 1 占位: 0 真实 WORM 写, 阶段 2
//! 任务 2.3 业务实装阶段 落地真实 `AuditEventSink` 写 PostgreSQL `audit_event` 表
//! (per ADR-0043 + 守门 #13 b 物理删除禁止 + 守门 #13 c SCD Type 2).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 d + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 100% 写路径过 `CanvasCollabAudit::write_event` (per 守门 #13 d).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::ApiError;

/// 6 字段 `CanvasCollabAuditEvent` (per brief §2.9 + DD-AGENT §4.14 + DD-001 C-26 派生).
///
/// 跟 agent/audit.rs `AgentAuditEvent` 5 字段不同, 这边多 1 个 `metadata: serde_json::Value`
/// 字段 (per A12.5 @ 提醒 + payload snapshot 派生).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasCollabAuditEvent {
    /// 全局唯一 ID.
    pub id: Uuid,
    /// 关联 canvas ID.
    pub canvas_id: Uuid,
    /// 操作者 user/actor ID (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// 操作动作 (e.g. "create_element" / "update_element" / "add_comment" / "grant_permission" / "list_presence").
    pub action: String,
    /// 操作时间 (UTC, per ADR-0043 WORM append-only).
    pub at: DateTime<Utc>,
    /// 任意 JSON metadata (per A12.5 @ 提醒 + payload snapshot).
    pub metadata: serde_json::Value,
}

impl CanvasCollabAuditEvent {
    /// 构造一个新的 `CanvasCollabAuditEvent`.
    pub fn new(
        canvas_id: Uuid,
        actor_id: Uuid,
        action: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            canvas_id,
            actor_id,
            action: action.into(),
            at: Utc::now(),
            metadata,
        }
    }
}

/// Audit sink placeholder.
///
/// 阶段 1 占位: 0 业务, 仅构造 + `write_event` 返回 `Ok(())`. 阶段 2 续做
/// 时换成真实 `AuditEventSink` 写 PostgreSQL `audit_event` 表.
#[derive(Debug, Clone, Default)]
pub struct CanvasCollabAudit {
    /// Placeholder for the future sink.
    _placeholder: (),
}

impl CanvasCollabAudit {
    /// 构造一个新的 audit sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// 异步写 audit event (per 守门 #13 d 100% audit).
    ///
    /// 阶段 1 占位: 仅检查 event 字段非空, 返回 `Ok(())`. 阶段 2 任务 2.3
    /// 业务实装阶段 落地真实 WORM append-only `audit_event` 表写.
    pub async fn write_event(&self, event: CanvasCollabAuditEvent) -> Result<(), ApiError> {
        if event.action.is_empty() {
            return Err(ApiError::BadRequest("audit event action is empty".into()));
        }
        if event.canvas_id.is_nil() {
            return Err(ApiError::BadRequest("audit event canvas_id is nil".into()));
        }
        // 阶段 1: 占位, 0 真实 DB 写. 阶段 2 任务 2.3 落地.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_new_fills_id_and_at() {
        let canvas_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();
        let e = CanvasCollabAuditEvent::new(
            canvas_id,
            actor_id,
            "create_element",
            serde_json::json!({}),
        );
        assert_eq!(e.canvas_id, canvas_id);
        assert_eq!(e.actor_id, actor_id);
        assert_eq!(e.action, "create_element");
        assert_ne!(e.id, Uuid::nil());
    }

    #[tokio::test]
    async fn write_event_accepts_well_formed_event() {
        let audit = CanvasCollabAudit::new();
        let event = CanvasCollabAuditEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "add_comment",
            serde_json::json!({"comment_id": "abc"}),
        );
        assert!(audit.write_event(event).await.is_ok());
    }

    #[tokio::test]
    async fn write_event_rejects_empty_action() {
        let audit = CanvasCollabAudit::new();
        let event = CanvasCollabAuditEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "",
            serde_json::Value::Null,
        );
        assert!(matches!(
            audit.write_event(event).await,
            Err(ApiError::BadRequest(_))
        ));
    }
}
