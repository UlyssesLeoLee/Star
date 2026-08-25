//! Audit 域事件(append-only 事件流)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{AuditAction, AuditEventId, TenantId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    pub event_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: Option<UserId>,
}

impl EventMeta {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            tenant_id,
            occurred_at: Utc::now(),
            actor_user_id: None,
        }
    }
}

/// 内部事件:AuditEvent 已成功追加
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventAppended {
    pub meta: EventMeta,
    pub audit_event_id: AuditEventId,
    pub action: AuditAction,
    pub target_type: String,
    pub target_id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEventKind {
    Appended(AuditEventAppended),
}

impl AuditEventKind {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::Appended(_) => "star.events.audit.event.appended.v1",
        }
    }
}
