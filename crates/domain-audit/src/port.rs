//! Audit 端口

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{AIAuditMetadata, AuditEvent};
use crate::error::AuditError;
use crate::value_object::{AuditAction, AuditEventId, TenantId, UserId};

/// `RecordAuditEventCommand`(追加一条审计事件)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAuditEventCommand {
    pub tenant_id: TenantId,
    pub actor_id: UserId,
    pub action: AuditAction,
    pub target_type: String,
    pub target_id: uuid::Uuid,
    pub payload_json: serde_json::Value,
}

/// `RecordAIAuditMetadataCommand`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAIAuditMetadataCommand {
    pub audit_event_id: AuditEventId,
    pub tenant_id: TenantId,
    pub agent_session_id: uuid::Uuid,
    pub worktree_id: Option<uuid::Uuid>,
    pub prompt_hash: String,
    pub response_hash: String,
    pub retention_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAuditEventQuery {
    pub tenant_id: TenantId,
    pub actor_id: Option<UserId>,
    pub target_type: Option<String>,
    pub target_id: Option<uuid::Uuid>,
    pub action: Option<AuditAction>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListAuditEventQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            actor_id: None,
            target_type: None,
            target_id: None,
            action: None,
            from: None,
            to: None,
            limit: 100,
            offset: 0,
        }
    }
}

#[async_trait]
pub trait AuditCommandPort: Send + Sync {
    /// 追加审计事件(append-only,INV-AUD-01 强制)
    async fn record_event(
        &self,
        cmd: RecordAuditEventCommand,
        actor: ActorContext,
    ) -> Result<AuditEvent, AuditError>;
    /// 追加 AI 审计元数据
    async fn record_ai_metadata(
        &self,
        cmd: RecordAIAuditMetadataCommand,
        actor: ActorContext,
    ) -> Result<AIAuditMetadata, AuditError>;
}

#[async_trait]
pub trait AuditQueryPort: Send + Sync {
    async fn get_event(
        &self,
        id: AuditEventId,
        viewer: ActorContext,
    ) -> Result<AuditEvent, AuditError>;
    async fn list_events(
        &self,
        q: ListAuditEventQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AuditEvent>, AuditError>;
    async fn count_events(
        &self,
        q: ListAuditEventQuery,
        viewer: ActorContext,
    ) -> Result<u64, AuditError>;
    async fn get_ai_metadata(
        &self,
        audit_event_id: AuditEventId,
        viewer: ActorContext,
    ) -> Result<AIAuditMetadata, AuditError>;
}
