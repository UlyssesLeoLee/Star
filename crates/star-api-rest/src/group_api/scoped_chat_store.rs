//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"scoped_chat_store.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"scoped_chat_store",type:"module",language:"rust"}),
//!   (store:Class {name:"PgScopedChatWorkflow",type:"class",language:"rust"}),
//!   (protector:Class {name:"TranscriptBodyProtector",type:"class",language:"rust"}),
//!   (context:Class {name:"TranscriptProtectionContext",type:"class",language:"rust"}),
//!   (protected:Class {name:"ProtectedTranscriptBody",type:"class",language:"rust"}),
//!   (idem:Class {name:"StoredChatIdempotency",type:"class",language:"rust"}),
//!   (max_targets:Variable {name:"MAX_CHAT_TARGETS",type:"variable",language:"rust"}),
//!   (max_refs:Variable {name:"MAX_CHAT_ENTITY_REFS",type:"variable",language:"rust"}),
//!   (max_bytes:Variable {name:"MAX_CHAT_MESSAGE_BYTES",type:"variable",language:"rust"}),
//!   (new:Function {name:"PgScopedChatWorkflow::new",type:"function",language:"rust"}),
//!   (submit:Function {name:"ScopedChatWorkflow::submit",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(store),(m)-[:CONTAINS]->(protector),
//!   (m)-[:CONTAINS]->(context),(m)-[:CONTAINS]->(protected),(m)-[:CONTAINS]->(idem),
//!   (m)-[:CONTAINS]->(max_targets),(m)-[:CONTAINS]->(max_refs),(m)-[:CONTAINS]->(max_bytes),
//!   (store)-[:HAS_METHOD]->(new),(store)-[:HAS_METHOD]->(submit),(store)-[:USES]->(protector),
//!   (submit)-[:USES]->(idem),(submit)-[:USES]->(max_targets),
//!   (submit)-[:USES]->(max_refs),(submit)-[:USES]->(max_bytes);

use async_trait::async_trait;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use super::scoped_chat::{
    ScopedChatCommand, ScopedChatReceipt, ScopedChatScope, ScopedChatWorkflow,
    ScopedChatWorkflowError,
};

const MAX_CHAT_TARGETS: usize = 20;
const MAX_CHAT_ENTITY_REFS: usize = 100;
const MAX_CHAT_MESSAGE_BYTES: usize = 32 * 1024;

#[derive(Clone)]
pub struct TranscriptProtectionContext {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub target_worktree_ids: Vec<Uuid>,
}

pub struct ProtectedTranscriptBody {
    pub ciphertext: Vec<u8>,
    pub wrapped_data_key: Vec<u8>,
    pub key_id: String,
    pub retention_seconds: i64,
}

/// Implementations must use authenticated envelope encryption, bind the supplied context as
/// associated data, resolve the strictest applicable Project retention policy across targets,
/// and never log plaintext. The wrapped key must be independently rotatable and destroyable.
#[async_trait]
pub trait TranscriptBodyProtector: Send + Sync {
    async fn protect(
        &self,
        context: TranscriptProtectionContext,
        plaintext: &str,
    ) -> Result<ProtectedTranscriptBody, ()>;
}

#[derive(Clone)]
pub struct PgScopedChatWorkflow {
    pool: PgPool,
    body_protector: Arc<dyn TranscriptBodyProtector>,
}

#[derive(FromRow)]
struct StoredChatIdempotency {
    request_hash: Vec<u8>,
    response_body: Value,
    active: bool,
}

impl PgScopedChatWorkflow {
    pub fn new(pool: PgPool, body_protector: Arc<dyn TranscriptBodyProtector>) -> Self {
        Self {
            pool,
            body_protector,
        }
    }
}

#[async_trait]
impl ScopedChatWorkflow for PgScopedChatWorkflow {
    async fn submit(
        &self,
        command: ScopedChatCommand,
    ) -> Result<ScopedChatReceipt, ScopedChatWorkflowError> {
        let mut target_ids = command.target_worktree_ids.clone();
        target_ids.sort_unstable();
        target_ids.dedup();
        let mut authorized_ids = command
            .authorized_targets
            .iter()
            .map(|target| target.worktree_id)
            .collect::<Vec<_>>();
        authorized_ids.sort_unstable();
        authorized_ids.dedup();
        let invalid_scope_targets = match command.scope {
            ScopedChatScope::Worktree => target_ids != [command.current_worktree_id],
            ScopedChatScope::Global => target_ids.is_empty() || target_ids.len() > MAX_CHAT_TARGETS,
        };
        let invalid_entity_refs = command.entity_refs.len() > MAX_CHAT_ENTITY_REFS
            || command.entity_refs.iter().any(|entity_ref| {
                entity_ref.ref_type.trim().is_empty()
                    || entity_ref.ref_type.len() > 100
                    || !target_ids.contains(&entity_ref.worktree_id)
            });
        if command.message.trim().is_empty()
            || command.message.len() > MAX_CHAT_MESSAGE_BYTES
            || invalid_entity_refs
            || invalid_scope_targets
            || target_ids != command.target_worktree_ids
            || target_ids != authorized_ids
            || authorized_ids.len() != command.authorized_targets.len()
        {
            return Err(ScopedChatWorkflowError::Internal);
        }

        let session_id = command.session_id.unwrap_or_else(Uuid::new_v4);
        let run_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let payload_id = Uuid::new_v4();
        let receipt = ScopedChatReceipt {
            session_id,
            run_id,
            scope: command.scope,
            target_worktree_ids: target_ids.clone(),
            correlation_id: command.correlation_id,
            status: "queued".to_owned(),
        };
        let response_body =
            serde_json::to_value(&receipt).map_err(|_| ScopedChatWorkflowError::Internal)?;
        let request_hash = command.request_fingerprint.to_vec();
        let scope = match command.scope {
            ScopedChatScope::Worktree => "WORKTREE",
            ScopedChatScope::Global => "GLOBAL",
        };
        let entity_refs_json = serde_json::to_value(&command.entity_refs)
            .map_err(|_| ScopedChatWorkflowError::Internal)?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.actor_id', $2, true)",
        )
        .bind(command.tenant_id.to_string())
        .bind(command.actor_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?;

        let inserted = sqlx::query(
            r#"
            INSERT INTO multica.group_chat_idempotency (
                tenant_id, actor_id, idempotency_key, request_hash, response_body,
                retention_period, expires_at
            ) VALUES ($1, $2, $3, $4, $5, INTERVAL '30 days', now() + INTERVAL '30 days')
            ON CONFLICT (tenant_id, actor_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(command.tenant_id)
        .bind(command.actor_id)
        .bind(command.idempotency_key)
        .bind(&request_hash)
        .bind(&response_body)
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?
        .rows_affected()
            == 1;

        if !inserted {
            let existing = sqlx::query_as::<_, StoredChatIdempotency>(
                r#"
                SELECT request_hash, response_body, expires_at > now() AS active
                FROM multica.group_chat_idempotency
                WHERE tenant_id = $1 AND actor_id = $2 AND idempotency_key = $3
                FOR UPDATE
                "#,
            )
            .bind(command.tenant_id)
            .bind(command.actor_id)
            .bind(command.idempotency_key)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?
            .ok_or(ScopedChatWorkflowError::Unavailable)?;

            if existing.active {
                if existing.request_hash != request_hash {
                    return Err(ScopedChatWorkflowError::Conflict);
                }
                let stored_receipt = serde_json::from_value(existing.response_body)
                    .map_err(|_| ScopedChatWorkflowError::Internal)?;
                tx.commit()
                    .await
                    .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
                return Ok(stored_receipt);
            }

            sqlx::query(
                r#"
                UPDATE multica.group_chat_idempotency
                SET request_hash = $4,
                    response_body = $5,
                    retention_period = INTERVAL '30 days',
                    expires_at = now() + INTERVAL '30 days',
                    created_at = now()
                WHERE tenant_id = $1 AND actor_id = $2 AND idempotency_key = $3
                "#,
            )
            .bind(command.tenant_id)
            .bind(command.actor_id)
            .bind(command.idempotency_key)
            .bind(&request_hash)
            .bind(&response_body)
            .execute(&mut *tx)
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
        }

        let protected_body = self
            .body_protector
            .protect(
                TranscriptProtectionContext {
                    tenant_id: command.tenant_id,
                    actor_id: command.actor_id,
                    session_id,
                    message_id,
                    target_worktree_ids: target_ids.clone(),
                },
                &command.message,
            )
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
        if protected_body.ciphertext.is_empty()
            || protected_body.wrapped_data_key.is_empty()
            || protected_body.key_id.trim().is_empty()
            || protected_body.key_id.len() > 200
            || protected_body.retention_seconds <= 0
        {
            return Err(ScopedChatWorkflowError::Unavailable);
        }

        sqlx::query(
            r#"
            INSERT INTO multica.group_chat_message_payload (
                payload_id, tenant_id, actor_id, ciphertext, wrapped_data_key, key_id,
                retention_period, expires_at, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                make_interval(secs => $7::double precision),
                now() + make_interval(secs => $7::double precision), now()
            )
            "#,
        )
        .bind(payload_id)
        .bind(command.tenant_id)
        .bind(command.actor_id)
        .bind(&protected_body.ciphertext)
        .bind(&protected_body.wrapped_data_key)
        .bind(&protected_body.key_id)
        .bind(protected_body.retention_seconds as f64)
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?;

        if command.session_id.is_some() {
            sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT session_id
                FROM multica.group_chat_session
                WHERE tenant_id = $1
                  AND actor_id = $2
                  AND session_id = $3
                  AND origin_worktree_id = $4
                FOR SHARE
                "#,
            )
            .bind(command.tenant_id)
            .bind(command.actor_id)
            .bind(session_id)
            .bind(command.current_worktree_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?
            .ok_or(ScopedChatWorkflowError::Conflict)?;
        } else {
            let thread_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO multica.group_chat_session (
                    session_id, tenant_id, actor_id, origin_worktree_id,
                    langgraph_thread_id, created_at
                ) VALUES ($1, $2, $3, $4, $5, now())
                "#,
            )
            .bind(session_id)
            .bind(command.tenant_id)
            .bind(command.actor_id)
            .bind(command.current_worktree_id)
            .bind(thread_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
        }

        if command.session_id.is_none() {
            sqlx::query(
                r#"
                INSERT INTO multica.group_chat_audit_event (
                    event_id, tenant_id, actor_id, session_id, event_type,
                    correlation_id, details, occurred_at
                ) VALUES ($1, $2, $3, $4, 'session.created', $5, '{}'::jsonb, now())
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(command.tenant_id)
            .bind(command.actor_id)
            .bind(session_id)
            .bind(command.correlation_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
        }

        sqlx::query(
            r#"
            INSERT INTO multica.group_chat_run (
                run_id, tenant_id, actor_id, session_id, origin_worktree_id,
                scope, target_worktree_ids, entity_refs, state, correlation_id,
                idempotency_key, request_hash, retention_period, expires_at, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, 'queued', $9, $10, $11,
                INTERVAL '30 days', now() + INTERVAL '30 days', now()
            )
            "#,
        )
        .bind(run_id)
        .bind(command.tenant_id)
        .bind(command.actor_id)
        .bind(session_id)
        .bind(command.current_worktree_id)
        .bind(scope)
        .bind(&target_ids)
        .bind(&entity_refs_json)
        .bind(command.correlation_id)
        .bind(command.idempotency_key)
        .bind(&request_hash)
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?;

        sqlx::query(
            r#"
            INSERT INTO multica.group_chat_message (
                message_id, tenant_id, actor_id, session_id, run_id,
                payload_id, role, scope, target_worktree_ids, entity_refs,
                correlation_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, 'user', $7, $8, $9, $10, now())
            "#,
        )
        .bind(message_id)
        .bind(command.tenant_id)
        .bind(command.actor_id)
        .bind(session_id)
        .bind(run_id)
        .bind(payload_id)
        .bind(scope)
        .bind(&target_ids)
        .bind(&entity_refs_json)
        .bind(command.correlation_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?;

        sqlx::query(
            r#"
            INSERT INTO multica.group_chat_dispatch_event (
                event_id, tenant_id, actor_id, run_id, event_type,
                schema_version, correlation_id, payload, created_at
            ) VALUES ($1, $2, $3, $4, 'group_chat.run.requested', 1, $5, $6, now())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(command.tenant_id)
        .bind(command.actor_id)
        .bind(run_id)
        .bind(command.correlation_id)
        .bind(json!({ "run_id": run_id }))
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?;

        sqlx::query(
            r#"
            INSERT INTO multica.group_chat_audit_event (
                event_id, tenant_id, actor_id, session_id, run_id, event_type,
                correlation_id, details, occurred_at
            ) VALUES ($1, $2, $3, $4, $5, 'message.accepted', $6, $7, now()),
                     ($8, $2, $3, $4, $5, 'run.queued', $6, $9, now())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(command.tenant_id)
        .bind(command.actor_id)
        .bind(session_id)
        .bind(run_id)
        .bind(command.correlation_id)
        .bind(json!({ "message_role": "user", "scope": scope }))
        .bind(Uuid::new_v4())
        .bind(json!({ "target_count": target_ids.len() }))
        .execute(&mut *tx)
        .await
        .map_err(|_| ScopedChatWorkflowError::Unavailable)?;

        // The message, run intent, immutable dispatch event and idempotent receipt become
        // visible together. A separate worker must consume the event before L0 executes.
        tx.commit()
            .await
            .map_err(|_| ScopedChatWorkflowError::Unavailable)?;
        Ok(receipt)
    }
}
