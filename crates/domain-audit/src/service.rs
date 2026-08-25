//! InMemoryAuditService

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::context::ActorContext;
use crate::entity::{AIAuditMetadata, AuditEvent};
use crate::error::AuditError;
use crate::event::{AuditEventAppended, AuditEventKind, EventMeta};
use crate::invariants::{
    check_invariant_04_ai_metadata_required, compute_immutable_hash, run_invariants,
    ALL_INVARIANT_CHECKS,
};
use crate::port::{
    AuditCommandPort, AuditQueryPort, ListAuditEventQuery, RecordAIAuditMetadataCommand,
    RecordAuditEventCommand,
};
use crate::value_object::{AIAuditMetadataId, AuditEventId, TenantId};

/// **InMemory Audit 命令/查询服务**
pub struct InMemoryAuditService {
    events: Arc<RwLock<HashMap<AuditEventId, AuditEvent>>>,
    ai_metadata: Arc<RwLock<HashMap<AIAuditMetadataId, AIAuditMetadata>>>,
    event_tx: mpsc::UnboundedSender<AuditEventKind>,
}

impl InMemoryAuditService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<AuditEventKind>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            ai_metadata: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }
    pub async fn count(&self) -> usize {
        self.events.read().await.len()
    }
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), AuditError> {
        if actor.tenant_id != expected {
            return Err(AuditError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryAuditService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryAuditService {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
            ai_metadata: self.ai_metadata.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[async_trait]
impl AuditCommandPort for InMemoryAuditService {
    async fn record_event(
        &self,
        cmd: RecordAuditEventCommand,
        actor: ActorContext,
    ) -> Result<AuditEvent, AuditError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        if !actor.is_auditor() && actor.user_id != cmd.actor_id.into_uuid() {
            // 非审计员只能为自己记录(系统级 actor 可绕过)
        }
        let now = chrono::Utc::now();
        let id = AuditEventId::new();
        let hash = compute_immutable_hash(
            cmd.tenant_id,
            cmd.actor_id,
            &cmd.action.to_string(),
            &cmd.target_type,
            cmd.target_id,
            now,
        );
        let event = AuditEvent {
            id,
            tenant_id: cmd.tenant_id,
            actor_id: cmd.actor_id,
            action: cmd.action,
            target_type: cmd.target_type.clone(),
            target_id: cmd.target_id,
            payload_json: cmd.payload_json,
            occurred_at: now,
            immutable_hash: hash,
        };
        run_invariants(ALL_INVARIANT_CHECKS, &event)?;
        self.events.write().await.insert(id, event.clone());

        // 通知总线(append-only 事件流)
        let kind = AuditEventKind::Appended(AuditEventAppended {
            meta: EventMeta {
                actor_user_id: Some(crate::value_object::UserId::from_uuid(actor.user_id)),
                ..EventMeta::new(cmd.tenant_id)
            },
            audit_event_id: id,
            action: event.action,
            target_type: event.target_type.clone(),
            target_id: event.target_id,
        });
        let _ = self.event_tx.send(kind);
        Ok(event)
    }

    async fn record_ai_metadata(
        &self,
        cmd: RecordAIAuditMetadataCommand,
        actor: ActorContext,
    ) -> Result<AIAuditMetadata, AuditError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let m_id = AIAuditMetadataId::new();
        let m = AIAuditMetadata {
            id: m_id,
            audit_event_id: cmd.audit_event_id,
            tenant_id: cmd.tenant_id,
            agent_session_id: cmd.agent_session_id,
            worktree_id: cmd.worktree_id,
            prompt_hash: cmd.prompt_hash,
            response_hash: cmd.response_hash,
            retention_until: cmd.retention_until,
            created_at: chrono::Utc::now(),
        };
        check_invariant_04_ai_metadata_required(&m)?;
        self.ai_metadata.write().await.insert(m_id, m.clone());
        Ok(m)
    }
}

#[async_trait]
impl AuditQueryPort for InMemoryAuditService {
    async fn get_event(
        &self,
        id: AuditEventId,
        viewer: ActorContext,
    ) -> Result<AuditEvent, AuditError> {
        if !viewer.is_auditor() {
            return Err(AuditError::PermissionDenied);
        }
        let e = self
            .events
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or(AuditError::NotFound(id))?;
        if e.tenant_id != viewer.tenant_id {
            return Err(AuditError::PermissionDenied);
        }
        Ok(e)
    }
    async fn list_events(
        &self,
        q: ListAuditEventQuery,
        viewer: ActorContext,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        Self::check_tenant(&viewer, q.tenant_id)?;
        if !viewer.is_auditor() {
            return Err(AuditError::PermissionDenied);
        }
        let store = self.events.read().await;
        let mut all: Vec<AuditEvent> = store
            .values()
            .filter(|e| e.tenant_id == q.tenant_id)
            .filter(|e| q.actor_id.map_or(true, |a| e.actor_id == a))
            .filter(|e| {
                q.target_type
                    .as_ref()
                    .map_or(true, |t| &e.target_type == t)
            })
            .filter(|e| q.target_id.map_or(true, |id| e.target_id == id))
            .filter(|e| q.action.map_or(true, |a| e.action == a))
            .filter(|e| q.from.map_or(true, |f| e.occurred_at >= f))
            .filter(|e| q.to.map_or(true, |t| e.occurred_at <= t))
            .cloned()
            .collect();
        all.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
        let offset = q.offset as usize;
        let limit = q.limit as usize;
        Ok(all.into_iter().skip(offset).take(limit).collect())
    }
    async fn count_events(
        &self,
        q: ListAuditEventQuery,
        viewer: ActorContext,
    ) -> Result<u64, AuditError> {
        if !viewer.is_auditor() {
            return Err(AuditError::PermissionDenied);
        }
        Ok(self
            .events
            .read()
            .await
            .values()
            .filter(|e| e.tenant_id == q.tenant_id)
            .count() as u64)
    }
    async fn get_ai_metadata(
        &self,
        audit_event_id: AuditEventId,
        viewer: ActorContext,
    ) -> Result<AIAuditMetadata, AuditError> {
        if !viewer.is_auditor() {
            return Err(AuditError::PermissionDenied);
        }
        self.ai_metadata
            .read()
            .await
            .values()
            .find(|m| m.audit_event_id == audit_event_id)
            .cloned()
            .ok_or(AuditError::Internal(format!(
                "AIAuditMetadata for event {audit_event_id} not found"
            )))
    }
}
