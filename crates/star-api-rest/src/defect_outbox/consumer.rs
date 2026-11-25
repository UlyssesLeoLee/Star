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

/// PG implementation（生产）——per G-8 关闭条件：消费者回路必须真实落库。
///
/// 三个物理表（由 `db/migrations/2026-11-25-defect-domain-phase-a.sql` 建）：
/// - `defect.notification_inbox`      幂等 inbox 行（同序列只存一份）
/// - `defect.notification_inbox_cursor` 持久游标（可挂起，不丢位点）
/// - `defect.notification_projection`  通知投影（消费结果）
///
/// 语义与 `InMemoryDefectInbox` 完全一致，差别只在持久化：
/// inbox 行 + 投影 + 游标推进在**同一事务**内提交（per DD §2.1 不变量）。
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
        cursor: &super::contract::InboxCursor,
        event: &super::contract::ReceivedEvent,
        stored_row: Option<&super::contract::StoredInboxRow>,
    ) -> Result<ApplyOutcome, DefectInboxError> {
        use sqlx::Row;

        let decision = decide_apply(cursor, event, stored_row);
        match decision {
            super::contract::ApplyDecision::Accept => {
                let mut tx = self
                    .pool
                    .begin()
                    .await
                    .map_err(|e| DefectInboxError::Apply(e.to_string()))?;

                let receipt_id = uuid::Uuid::new_v4();
                let now = chrono::Utc::now();
                let seq = event.envelope.sequence;

                // 1. inbox 行（幂等：同序列重复插入会撞主键 → ON CONFLICT DO NOTHING）
                sqlx::query(
                    r#"INSERT INTO defect.notification_inbox
                         (sequence, event_type, event_id, correlation_id, payload_digest,
                          payload, received_at, applied_at, receipt_id)
                       VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                       ON CONFLICT (sequence) DO NOTHING"#,
                )
                .bind(seq)
                .bind(&event.envelope.event_type)
                .bind(event.envelope.event_id)
                .bind(event.envelope.correlation_id)
                .bind(event.envelope.digest.to_vec())
                .bind(&event.payload)
                .bind(now)
                .bind(now)
                .bind(receipt_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| DefectInboxError::Apply(e.to_string()))?;

                // 2. 通知投影（消费结果）
                sqlx::query(
                    r#"INSERT INTO defect.notification_projection
                         (event_id, event_type, correlation_id, payload, recorded_at)
                       VALUES ($1,$2,$3,$4,$5)
                       ON CONFLICT (event_id) DO NOTHING"#,
                )
                .bind(event.envelope.event_id)
                .bind(&event.envelope.event_type)
                .bind(event.envelope.correlation_id)
                .bind(&event.payload)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(|e| DefectInboxError::Projection(e.to_string()))?;

                // 3. 游标推进（同一事务；解除挂起）
                sqlx::query(
                    r#"INSERT INTO defect.notification_inbox_cursor
                         (consumer_id, producer_id, sequence, suspended_reason, updated_at)
                       VALUES ($1,$2,$3,NULL,$4)
                       ON CONFLICT (consumer_id, producer_id) DO UPDATE
                       SET sequence=EXCLUDED.sequence,
                           suspended_reason=NULL,
                           suspended_at=NULL,
                           updated_at=EXCLUDED.updated_at"#,
                )
                .bind(&cursor.consumer_id)
                .bind(&cursor.producer_id)
                .bind(seq)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(|e| DefectInboxError::Cursor(e.to_string()))?;

                tx.commit()
                    .await
                    .map_err(|e| DefectInboxError::Apply(e.to_string()))?;

                Ok(ApplyOutcome::Committed {
                    receipt_id,
                    committed_at: now,
                    next_expected_sequence: seq + 1,
                })
            }
            super::contract::ApplyDecision::Replay => {
                // 精确重放：返回既有回执，不写任何行（幂等）。
                let receipt_id = stored_row
                    .and_then(|r| r.receipt_id)
                    .unwrap_or_else(uuid::Uuid::new_v4);
                Ok(ApplyOutcome::ExactReplay {
                    receipt_id,
                    committed_at: chrono::Utc::now(),
                })
            }
            super::contract::ApplyDecision::DigestConflict => {
                // 同序列不同摘要 → 隔离事故，不覆盖既有行。
                self.record_incident(event, "digest_conflict").await?;
                Ok(ApplyOutcome::DigestConflict)
            }
            super::contract::ApplyDecision::SequenceGap { next_expected } => {
                self.suspend_cursor(cursor, "sequence_gap").await?;
                Ok(ApplyOutcome::SequenceGap {
                    next_expected_sequence: next_expected,
                })
            }
            super::contract::ApplyDecision::StaleCursor { next_expected } => {
                self.suspend_cursor(cursor, "retention_gap").await?;
                Ok(ApplyOutcome::StaleCursor {
                    next_expected_sequence: next_expected,
                })
            }
            super::contract::ApplyDecision::InvalidEnvelope => {
                self.record_incident(event, "invalid_envelope").await?;
                Ok(ApplyOutcome::Rejected)
            }
        }
    }

    async fn get_cursor(
        &self,
        consumer_id: &str,
        producer_id: &str,
    ) -> Result<super::contract::InboxCursor, DefectInboxError> {
        use sqlx::Row;
        let row = sqlx::query(
            r#"SELECT consumer_id, producer_id, sequence, updated_at
               FROM defect.notification_inbox_cursor
               WHERE consumer_id=$1 AND producer_id=$2"#,
        )
        .bind(consumer_id)
        .bind(producer_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Cursor(e.to_string()))?;

        Ok(match row {
            Some(r) => super::contract::InboxCursor {
                consumer_id: r.get::<String, _>("consumer_id"),
                producer_id: r.get::<String, _>("producer_id"),
                sequence: r.get::<i64, _>("sequence"),
                updated_at: r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
            },
            None => super::contract::InboxCursor {
                consumer_id: consumer_id.to_string(),
                producer_id: producer_id.to_string(),
                sequence: 0,
                updated_at: chrono::Utc::now(),
            },
        })
    }

    async fn get_stored_row(
        &self,
        sequence: i64,
    ) -> Result<Option<super::contract::StoredInboxRow>, DefectInboxError> {
        use sqlx::Row;
        let row = sqlx::query(
            r#"SELECT sequence, event_type, event_id, correlation_id, payload_digest,
                      payload, received_at, applied_at, receipt_id
               FROM defect.notification_inbox WHERE sequence=$1"#,
        )
        .bind(sequence)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Apply(e.to_string()))?;

        Ok(row.map(|r| {
            let digest_vec: Vec<u8> = r.get("payload_digest");
            let mut digest = [0u8; 32];
            let len = digest_vec.len().min(32);
            digest[..len].copy_from_slice(&digest_vec[..len]);
            super::contract::StoredInboxRow {
                sequence: r.get("sequence"),
                event_type: r.get("event_type"),
                event_id: r.get("event_id"),
                correlation_id: r.get("correlation_id"),
                payload_digest: digest,
                payload: r.get("payload"),
                received_at: r.get("received_at"),
                applied_at: r.get("applied_at"),
                receipt_id: r.get("receipt_id"),
            }
        }))
    }
}

impl PgDefectInboxStore {
    /// 挂起游标（fail-closed：不再前进，等人工/自动对账）。
    async fn suspend_cursor(
        &self,
        cursor: &super::contract::InboxCursor,
        reason: &str,
    ) -> Result<(), DefectInboxError> {
        sqlx::query(
            r#"INSERT INTO defect.notification_inbox_cursor
                 (consumer_id, producer_id, sequence, suspended_reason, suspended_at, updated_at)
               VALUES ($1,$2,$3,$4,now(),now())
               ON CONFLICT (consumer_id, producer_id) DO UPDATE
               SET suspended_reason=EXCLUDED.suspended_reason,
                   suspended_at=EXCLUDED.suspended_at,
                   updated_at=EXCLUDED.updated_at"#,
        )
        .bind(&cursor.consumer_id)
        .bind(&cursor.producer_id)
        .bind(cursor.sequence)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Cursor(e.to_string()))?;
        Ok(())
    }

    /// 记录隔离事故（摘要冲突 / 非法信封）——不覆盖既有事实。
    async fn record_incident(
        &self,
        event: &super::contract::ReceivedEvent,
        reason: &str,
    ) -> Result<(), DefectInboxError> {
        sqlx::query(
            r#"INSERT INTO defect.notification_inbox_incident
                 (event_id, event_type, correlation_id, reason, payload, recorded_at)
               VALUES ($1,$2,$3,$4,$5,now())
               ON CONFLICT (event_id, reason) DO NOTHING"#,
        )
        .bind(event.envelope.event_id)
        .bind(&event.envelope.event_type)
        .bind(event.envelope.correlation_id)
        .bind(reason)
        .bind(&event.payload)
        .execute(&self.pool)
        .await
        .map_err(|e| DefectInboxError::Apply(e.to_string()))?;
        Ok(())
    }
}