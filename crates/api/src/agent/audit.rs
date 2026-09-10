// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/agent/audit.rs` — 100% audit middleware (per 守门 #13 d Transaction 100% audit + DD-AGENT §3.3 + §4.7).
//!
//! Per brief §2.4: 5 字段 `AgentAuditEvent` + `AgentAudit` sink placeholder + `write_event` 异步方法.
//! 阶段 1 占位: `write_event` 仅 log 一行, 不真正写 WORM append-only audit_event 表
//! (per ADR-0043 + 守门 #13 b). 阶段 2 业务实装阶段 (任务 2.1-2.5) 落地真实
//! `AuditEventSink` 写 PostgreSQL / Memgraph `audit_event` 表.
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #13 d + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - 100% 写路径过 `AgentAudit::write_event` (per 守门 #13 d).
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::ApiError;

/// 5 字段 `AgentAuditEvent` (per brief §2.4 + DD-AGENT §4.7 派生).
///
/// 跟 `arg/sse_hub::ARGSseEvent` 6 事件类型不同 — `AgentAuditEvent` 是
/// append-only audit log, 1 event type, 5 字段. 阶段 2 业务实装阶段
/// 落地真实 WORM audit_event 表 (per ADR-0043).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAuditEvent {
    /// 全局唯一 ID.
    pub id: Uuid,
    /// 关联 agent ID.
    pub agent_id: Uuid,
    /// 操作者 user/actor ID (per 守门 #10 author = current actor).
    pub actor_id: Uuid,
    /// 操作动作 (e.g. "create_agent" / "update_agent" / "handoff" / "start" / "stop" / "restart").
    pub action: String,
    /// 操作时间 (UTC, per ADR-0043 WORM append-only).
    pub at: DateTime<Utc>,
}

impl AgentAuditEvent {
    /// 构造一个新的 `AgentAuditEvent`.
    pub fn new(agent_id: Uuid, actor_id: Uuid, action: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            actor_id,
            action: action.into(),
            at: Utc::now(),
        }
    }
}

/// Audit sink placeholder.
///
/// 阶段 1 占位: 0 业务, 仅构造 + `write_event` 返回 `Ok(())`. 阶段 2 续做
/// 时换成真实 `AuditEventSink` 写 PostgreSQL `audit_event` 表 (per ADR-0043 +
/// 守门 #13 d Transaction 100% audit + 守门 #13 b 物理删除禁止).
#[derive(Debug, Clone, Default)]
pub struct AgentAudit {
    /// Placeholder for the future sink (e.g. `Arc<dyn AuditEventSink>`).
    /// Currently unused; reserved for phase 2.
    _placeholder: (),
}

impl AgentAudit {
    /// 构造一个新的 audit sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// 异步写 audit event (per 守门 #13 d 100% audit).
    ///
    /// 阶段 1 占位: 仅检查 event 字段非空, 返回 `Ok(())`. 阶段 2 业务实装阶段
    /// 落地真实 WORM append-only `audit_event` 表写.
    pub async fn write_event(&self, event: AgentAuditEvent) -> Result<(), ApiError> {
        if event.action.is_empty() {
            return Err(ApiError::BadRequest("audit event action is empty".into()));
        }
        if event.agent_id.is_nil() {
            return Err(ApiError::BadRequest("audit event agent_id is nil".into()));
        }
        // 阶段 1: 占位, 0 真实 DB 写. 阶段 2 任务 2.1+ 落地.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_new_fills_id_and_at() {
        let agent_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();
        let e = AgentAuditEvent::new(agent_id, actor_id, "create_agent");
        assert_eq!(e.agent_id, agent_id);
        assert_eq!(e.actor_id, actor_id);
        assert_eq!(e.action, "create_agent");
        assert_ne!(e.id, Uuid::nil());
    }

    #[tokio::test]
    async fn write_event_accepts_well_formed_event() {
        let audit = AgentAudit::new();
        let event = AgentAuditEvent::new(Uuid::new_v4(), Uuid::new_v4(), "start");
        assert!(audit.write_event(event).await.is_ok());
    }

    #[tokio::test]
    async fn write_event_rejects_empty_action() {
        let audit = AgentAudit::new();
        let event = AgentAuditEvent::new(Uuid::new_v4(), Uuid::new_v4(), "");
        assert!(matches!(
            audit.write_event(event).await,
            Err(ApiError::BadRequest(_))
        ));
    }
}
