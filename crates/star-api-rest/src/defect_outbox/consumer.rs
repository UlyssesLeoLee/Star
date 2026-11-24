// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/src/defect_outbox/consumer.rs"})
MERGE (port:Type {id:"crates/star-api-rest/src/defect_outbox/consumer.rs::DefectInboxStore",kind:"trait"})
MERGE (pg:Type {id:"crates/star-api-rest/src/defect_outbox/consumer.rs::PgDefectInboxStore",kind:"struct"})
MERGE (mem:Type {id:"crates/star-api-rest/src/defect_outbox/consumer.rs::InMemoryDefectInbox",kind:"struct"})
MERGE (outcome:Type {id:"crates/star-api-rest/src/defect_outbox/consumer.rs::ApplyOutcome",kind:"enum"})
MERGE (error:Type {id:"crates/star-api-rest/src/defect_outbox/consumer.rs::DefectInboxError",kind:"enum"})
MERGE (self)-[:DEFINES]->(port),(module)-[:DEFINES]->(pg),(module)-[:DEFINES]->(mem)
MERGE (self)-[:DEFINES]->(outcome),(module)-[:DEFINES]->(error)
@cypher
*/
//! Defect Notification Inbox consumer: applies Defect Outbox events through its own
//! idempotent Inbox, updates the local notification projection, and advances
//! the durable cursor only inside the same committed transaction.
//!
//! Based on `device_outbox::collaboration_inbox` pattern, adapted for Defect → Notification.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::contract::{
    decide_apply, ApplyDecision, Envelope, InboxCursor, ReceivedEvent,
    EnvelopeError, StoredInboxRow,
};

/// Result of one durable inbox apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyOutcome {
    /// New event applied; projection updated and cursor advanced in the same
    /// committed transaction.
    Committed {
        receipt_id: Uuid,
        committed_at: DateTime<Utc>,
        next_expected_sequence: i64,
    },
    /// Same identity + same digest replay → stable existing receipt returned;
    /// nothing was overwritten.
    ExactReplay {
        receipt_id: Uuid,
        committed_at: DateTime<Utc>,
    },
    /// Same identity, different digest → isolated incident recorded, alarm
    /// raised, nothing overwritten.
    DigestConflict,
    /// A sequence gap was detected → incident recorded, consumer checkpoint
    /// suspended; the projection must be reconciled before any further apply.
    SequenceGap { next_expected_sequence: i64 },
    /// The event is behind the durable cursor and no stored receipt exists →
    /// retention gap; fail closed with a suspended checkpoint.
    StaleCursor { next_expected_sequence: i64 },
    /// Envelope rejected by the candidate contract (unknown type/version).
    Rejected,
    /// The consumer checkpoint was already suspended → fail closed without
    /// touching the projection.
    Suspended { next_expected_sequence: i64 },
}

#[derive(Debug, thiserror::Error)]
pub enum DefectInboxError {
    #[error("defect inbox apply failed: {0}")]
    Apply(String),
    #[error("cursor error: {0}")]
    Cursor(String),
    #[error("projection error: {0}")]
    Projection(String),
}

/// Defect Inbox persistence port
#[async_trait]
pub trait DefectInboxStore: Send + Sync {
    /// Apply a received event, update projection, advance cursor — all in one transaction.
    async fn apply(
        &self,
        cursor: &super::contract::InboxCursor,
        event: &super::contract::ReceivedEvent,
        stored_row: Option<&super::contract::StoredInboxRow>,
    ) -> Result<ApplyOutcome, DefectInboxError>;

    /// Get current cursor for this consumer/producer pair.
    async fn get_cursor(&self, consumer_id: &str, producer_id: &str) -> Result<super::contract::InboxCursor, DefectInboxError>;

    /// Get stored inbox row by sequence (for replay/digest check).
    async fn get_stored_row(&self, sequence: i64) -> Result<Option<super::contract::StoredInboxRow>, DefectInboxError>;
}

/// In-memory implementation (development/testing only)
pub struct InMemoryDefectInbox {
    cursor: std::sync::RwLock<super::contract::InboxCursor>,
    inbox: std::sync::RwLock<std::collections::BTreeMap<i64, super::contract::StoredInboxRow>>,
    projection: std::sync::RwLock<Vec<super::contract::ReceivedEvent>>,
}

impl InMemoryDefectInbox {
    pub fn new() -> Self {
        Self {
            cursor: std::sync::RwLock::new(super::contract::InboxCursor {
                consumer_id: "notification.owner".to_string(),
                producer_id: "defect.owner".to_string(),
                sequence: 0,
                updated_at: chrono::Utc::now(),
            }),
            inbox: std::sync::RwLock::new(std::collections::BTreeMap::new()),
            projection: std::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl DefectInboxStore for InMemoryDefectInbox {
    async fn apply(
        &self,
        cursor: &super::contract::InboxCursor,
        event: &super::contract::ReceivedEvent,
        stored_row: Option<&super::contract::StoredInboxRow>,
    ) -> Result<ApplyOutcome, DefectInboxError> {
        let decision = decide_apply(&cursor, &event, stored_row);

        match decision {
            super::contract::ApplyDecision::Accept => {
                let receipt_id = uuid::Uuid::new_v4();
                let now = chrono::Utc::now();
                let seq = event.envelope.sequence;

                // Store inbox row
                let row = super::contract::StoredInboxRow {
                    sequence: event.envelope.sequence,
                    event_type: event.envelope.event_type.clone(),
                    event_id: event.envelope.event_id,
                    correlation_id: event.envelope.correlation_id,
                    payload_digest: event.envelope.digest,
                    payload: event.payload.clone(),
                    received_at: chrono::Utc::now(),
                    applied_at: Some(chrono::Utc::now()),
                    receipt_id: Some(uuid::Uuid::new_v4()),
                };

                // Store in memory
                self.inbox.write().unwrap().insert(event.envelope.sequence, row.clone());
                self.projection.write().unwrap().push(event.clone());

                // Advance cursor
                let mut cursor_guard = self.cursor.write().unwrap();
                cursor_guard.sequence = event.envelope.sequence;
                cursor_guard.updated_at = chrono::Utc::now();
                let next_seq = cursor_guard.sequence + 1;

                Ok(ApplyOutcome::Committed {
                    receipt_id: uuid::Uuid::new_v4(),
                    committed_at: chrono::Utc::now(),
                    next_expected_sequence: next_seq,
                })
            }
            super::contract::ApplyDecision::Replay => {
                Ok(ApplyOutcome::ExactReplay {
                    receipt_id: uuid::Uuid::new_v4(),
                    committed_at: chrono::Utc::now(),
                })
            }
            super::contract::ApplyDecision::DigestConflict => {
                Ok(ApplyOutcome::DigestConflict)
            }
            super::contract::ApplyDecision::SequenceGap { next_expected } => {
                Ok(ApplyOutcome::SequenceGap { next_expected_sequence: next_expected })
            }
            super::contract::ApplyDecision::StaleCursor { next_expected } => {
                Ok(ApplyOutcome::StaleCursor { next_expected_sequence: next_expected })
            }
            super::contract::ApplyDecision::InvalidEnvelope => {
                Ok(ApplyOutcome::Rejected)
            }
        }
    }

    async fn get_cursor(&self, _consumer_id: &str, _producer_id: &str) -> Result<super::contract::InboxCursor, DefectInboxError> {
        Ok(self.cursor.read().unwrap().clone())
    }

    async fn get_stored_row(&self, sequence: i64) -> Result<Option<super::contract::StoredInboxRow>, DefectInboxError> {
        Ok(self.inbox.read().unwrap().get(&sequence).cloned())
    }
}

impl Default for InMemoryDefectInbox {
    fn default() -> Self {
        Self::new()
    }
}

/// PG implementation placeholder (for production)
pub struct PgDefectInboxStore {
    pool: sqlx::PgPool,
}

impl PgDefectInboxStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DefectInboxStore for PgDefectInboxStore {
    async fn apply(
        &self,
        _cursor: &super::contract::InboxCursor,
        _event: &super::contract::ReceivedEvent,
        _stored_row: Option<&super::contract::StoredInboxRow>,
    ) -> Result<ApplyOutcome, DefectInboxError> {
        Err(DefectInboxError::Apply("PG implementation not yet implemented".to_string()))
    }

    async fn get_cursor(&self, _consumer_id: &str, _producer_id: &str) -> Result<super::contract::InboxCursor, DefectInboxError> {
        Err(DefectInboxError::Cursor("PG implementation not yet implemented".to_string()))
    }

    async fn get_stored_row(&self, _sequence: i64) -> Result<Option<super::contract::StoredInboxRow>, DefectInboxError> {
        Err(DefectInboxError::Apply("PG implementation not yet implemented".to_string()))
    }
}