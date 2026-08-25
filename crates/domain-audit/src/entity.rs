//! Audit 域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{AIAuditMetadataId, AuditAction, AuditEventId, TenantId, UserId};

/// **AuditEvent**(审计事件,append-only,不可 UPDATE/DELETE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 主键
    pub id: AuditEventId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 触发者 User ID
    pub actor_id: UserId,
    /// 动作
    pub action: AuditAction,
    /// 目标类型(`User` / `WorkItem` / `Tenant` 等)
    pub target_type: String,
    /// 目标 ID
    pub target_id: uuid::Uuid,
    /// 负载 JSON
    pub payload_json: serde_json::Value,
    /// 发生时间
    pub occurred_at: DateTime<Utc>,
    /// 不可变哈希(`sha256(id|tenant|actor|action|target|occurred_at)`)
    pub immutable_hash: String,
}

impl AuditEvent {
    pub const FIELD_COUNT: usize = 9;
}

/// **AIAuditMetadata**(AI 审计元数据,关联到 AuditEvent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAuditMetadata {
    pub id: AIAuditMetadataId,
    pub audit_event_id: AuditEventId,
    pub tenant_id: TenantId,
    /// Agent 会话 ID
    pub agent_session_id: uuid::Uuid,
    /// Worktree ID(可空)
    pub worktree_id: Option<uuid::Uuid>,
    /// Prompt 哈希(不存明文)
    pub prompt_hash: String,
    /// Response 哈希
    pub response_hash: String,
    /// 保留期限(None 表示永久)
    pub retention_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AIAuditMetadata {
    pub const FIELD_COUNT: usize = 9;
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.retention_until.map_or(false, |r| r < now)
    }
}
