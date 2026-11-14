// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::module",kind:"module"})
MERGE (port:Type {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::CollaborationInboxStore",kind:"trait"})
MERGE (pg:Type {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::PgCollaborationInboxStore",kind:"struct"})
MERGE (outcome:Type {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::ApplyOutcome",kind:"enum"})
MERGE (error:Type {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::CollaborationInboxError",kind:"enum"})
MERGE (apply:Symbol {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::CollaborationInboxStore.apply",kind:"method"})
MERGE (checkpoint:Symbol {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::CollaborationInboxStore.checkpoint",kind:"method"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(port),(module)-[:DEFINES]->(pg),(module)-[:DEFINES]->(outcome),(module)-[:DEFINES]->(error)
MERGE (pg)-[:IMPLEMENTS]->(port)
MERGE (port)-[:HAS_METHOD]->(apply),(port)-[:HAS_METHOD]->(checkpoint)
MERGE (apply)-[:RETURNS_TYPE]->(outcome)
MERGE (apply)-[:CALLS]->(:Database {id:"public.commit_device_revoke_event_v1",kind:"stored_function"})
MERGE (checkpoint)-[:CALLS]->(:Database {id:"public.read_collaboration_inbox_checkpoint",kind:"stored_function"})
MERGE (apply)-[:WRITES]->(:Table {name:"collaboration_device_inbox",kind:"table"})
MERGE (apply)-[:WRITES]->(:Table {name:"collaboration_device_projection",kind:"table"})
MERGE (apply)-[:WRITES]->(:Table {name:"collaboration_inbox_checkpoint",kind:"table"})
@endcypher
*/
//! Collaboration owner receiver: applies Identity Outbox events through its own
//! idempotent Inbox, updates the local invalidation projection, and advances
//! the durable cursor only inside the same committed transaction (review 合约
//! #2/#4/#5). Gaps, stale cursors and digest conflicts fail closed.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::contract::{
    decide_apply, ApplyDecision, InboxCursor, ReceivedEvent, StoredInboxRow, SuspensionReason,
    PRODUCER_ID,
};

/// Result of one durable inbox apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// New event applied; projection updated and cursor advanced in the same
    /// committed transaction.
    Committed {
        receipt_id: Uuid,
        committed_at: DateTime<Utc>,
        next_expected_sequence: i64,
    },
    /// Same identity + same digest replay → stable existing receipt returned;
    /// nothing was overwritten (review 合约 #4).
    ExactReplay {
        receipt_id: Uuid,
        committed_at: DateTime<Utc>,
    },
    /// Same identity, different digest → isolated incident recorded, alarm
    /// raised, nothing overwritten (review 合约 #4).
    DigestConflict,
    /// A sequence gap was detected → incident recorded, consumer checkpoint
    /// suspended; the projection must be reconciled with the Identity owner
    /// before any further apply (review 合约 #5).
    SequenceGap { next_expected_sequence: i64 },
    /// The event is behind the durable cursor and no stored receipt exists →
    /// retention gap; fail closed with a suspended checkpoint (review 合约 #5).
    StaleCursor { next_expected_sequence: i64 },
    /// Envelope rejected by the candidate contract (unknown type/version) →
    /// incident recorded, cursor untouched (review 合约 #6).
    Rejected,
    /// The consumer checkpoint was already suspended → fail closed without
    /// touching the projection (review 合约 #5).
    Suspended { next_expected_sequence: i64 },
}

#[derive(Debug, thiserror::Error)]
pub enum CollaborationInboxError {
    #[error("collaboration inbox apply failed: {0}")]
    Apply(String),
}

/// Collaboration owner persistence contract for the durable receiver.
#[async_trait]
pub trait CollaborationInboxStore: Send + Sync {
    /// Apply one received event inside one local owner transaction. The cursor
    /// is advanced by the store only as part of that committed transaction;
    /// any failure leaves the checkpoint untouched and the event replayable.
    async fn apply(
        &self,
        received: &ReceivedEvent,
    ) -> Result<ApplyOutcome, CollaborationInboxError>;

    /// Read the durable checkpoint bound to this transport profile. `None`
    /// means the consumer has not yet established a cursor (first delivery).
    async fn checkpoint(
        &self,
        tenant_id: Uuid,
        stream_id: Uuid,
        transport_provider_id: &str,
        transport_profile_id: Uuid,
    ) -> Result<Option<InboxCursor>, CollaborationInboxError>;
}

/// PostgreSQL implementation backed by `commit_device_revoke_event_v1`, which
/// commits the Inbox receipt, the projection update and the cursor advance
/// atomically (review 合约 #4/#5).
pub struct PgCollaborationInboxStore {
    pool: PgPool,
}

impl PgCollaborationInboxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct CommitOutcomeRow {
    outcome: String,
    receipt_id: Option<Uuid>,
    committed_at: Option<DateTime<Utc>>,
    next_expected_sequence: Option<i64>,
}

#[async_trait]
impl CollaborationInboxStore for PgCollaborationInboxStore {
    async fn apply(
        &self,
        received: &ReceivedEvent,
    ) -> Result<ApplyOutcome, CollaborationInboxError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(apply_error("begin collaboration inbox apply"))?;
        set_collaboration_runtime_scope(&mut tx, received.event.tenant_id).await?;
        let row = sqlx::query_as::<_, CommitOutcomeRow>(
            "SELECT * FROM public.commit_device_revoke_event_v1($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)",
        )
        .bind(received.event.tenant_id)
        .bind(PRODUCER_ID)
        .bind(received.event.stream_id)
        .bind(received.event.source_sequence)
        .bind(received.event.event_id)
        .bind(&received.event.event_type)
        .bind(received.event.schema_version)
        .bind(received.event.actor_id)
        .bind(received.event.device_id)
        .bind(received.event.device_generation)
        .bind(received.event.occurred_at)
        .bind(received.event.correlation_id)
        .bind(received.event_digest.as_slice())
        .bind(&received.transport_provider_id)
        .bind(received.transport_profile_id)
        .bind(cursor_handle_for(received).to_vec())
        .fetch_one(&mut *tx)
        .await
        .map_err(apply_error("commit device revoke event"))?;
        tx.commit()
            .await
            .map_err(apply_error("commit collaboration inbox apply"))?;
        map_outcome(row)
    }

    async fn checkpoint(
        &self,
        tenant_id: Uuid,
        stream_id: Uuid,
        transport_provider_id: &str,
        transport_profile_id: Uuid,
    ) -> Result<Option<InboxCursor>, CollaborationInboxError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(apply_error("begin collaboration checkpoint read"))?;
        set_collaboration_runtime_scope(&mut tx, tenant_id).await?;
        let row: Option<(
            Vec<u8>,
            i64,
            Option<String>,
            Option<DateTime<Utc>>,
            DateTime<Utc>,
        )> =
            sqlx::query_as("SELECT * FROM public.read_collaboration_inbox_checkpoint($1,$2,$3,$4)")
                .bind(tenant_id)
                .bind(stream_id)
                .bind(transport_provider_id)
                .bind(transport_profile_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(apply_error("read collaboration checkpoint"))?;
        tx.commit()
            .await
            .map_err(apply_error("commit collaboration checkpoint read"))?;
        Ok(row.map(
            |(
                cursor_handle,
                next_expected_sequence,
                suspended_reason,
                _suspended_at,
                _updated_at,
            )| {
                InboxCursor {
                    stream_id,
                    transport_provider_id: transport_provider_id.to_string(),
                    transport_profile_id,
                    cursor_handle,
                    next_expected_sequence,
                    suspension: suspended_reason.and_then(|reason| match reason.as_str() {
                        "sequence_gap" => Some(SuspensionReason::SequenceGap),
                        "stale_cursor" => Some(SuspensionReason::StaleCursor),
                        "retention_gap" => Some(SuspensionReason::RetentionGap),
                        _ => None,
                    }),
                }
            },
        ))
    }
}

fn map_outcome(row: CommitOutcomeRow) -> Result<ApplyOutcome, CollaborationInboxError> {
    match row.outcome.as_str() {
        "committed" => Ok(ApplyOutcome::Committed {
            receipt_id: row.receipt_id.ok_or_else(|| {
                CollaborationInboxError::Apply("committed outcome without receipt".to_string())
            })?,
            committed_at: row.committed_at.ok_or_else(|| {
                CollaborationInboxError::Apply("committed outcome without timestamp".to_string())
            })?,
            next_expected_sequence: row.next_expected_sequence.unwrap_or(1),
        }),
        "exact_replay" => Ok(ApplyOutcome::ExactReplay {
            receipt_id: row.receipt_id.ok_or_else(|| {
                CollaborationInboxError::Apply("exact_replay outcome without receipt".to_string())
            })?,
            committed_at: row.committed_at.ok_or_else(|| {
                CollaborationInboxError::Apply("exact_replay outcome without timestamp".to_string())
            })?,
        }),
        "conflict" => Ok(ApplyOutcome::DigestConflict),
        "sequence_gap" => Ok(ApplyOutcome::SequenceGap {
            next_expected_sequence: row.next_expected_sequence.unwrap_or(1),
        }),
        "stale_cursor" => Ok(ApplyOutcome::StaleCursor {
            next_expected_sequence: row.next_expected_sequence.unwrap_or(1),
        }),
        "rejected" => Ok(ApplyOutcome::Rejected),
        "suspended" => Ok(ApplyOutcome::Suspended {
            next_expected_sequence: row.next_expected_sequence.unwrap_or(1),
        }),
        other => Err(CollaborationInboxError::Apply(format!(
            "unknown commit outcome {other:?}"
        ))),
    }
}

async fn set_collaboration_runtime_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), CollaborationInboxError> {
    // The deployment login receives star_collaboration_runtime through explicit
    // membership; the runtime role can only EXECUTE the three owner functions.
    sqlx::query("SET LOCAL ROLE star_collaboration_runtime")
        .execute(&mut **tx)
        .await
        .map_err(apply_error("set collaboration runtime role"))?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(apply_error("set collaboration tenant scope"))?;
    Ok(())
}

fn apply_error(operation: &'static str) -> impl FnOnce(sqlx::Error) -> CollaborationInboxError {
    move |error| CollaborationInboxError::Apply(format!("{operation}: {error}"))
}

/// Deterministic opaque cursor handle bound to the stream + transport profile:
/// reproducible after a restart so the durable checkpoint can be resumed, but
/// never interpretable by a different provider/profile (review 合约 #5/#7).
fn cursor_handle_for(received: &ReceivedEvent) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"collab-inbox-cursor|v1|");
    hasher.update(received.event.stream_id.as_bytes());
    hasher.update(b"|");
    hasher.update(received.transport_profile_id.as_bytes());
    hasher.finalize().into()
}

/// In-memory receiver used by unit tests; mirrors the SQL function's outcome
/// matrix through the shared `decide_apply` policy. State is interior-mutable
/// so the async trait's `&self` receiver can still commit writes durably
/// between calls.
#[derive(Debug, Default)]
pub struct InMemoryCollaborationInbox {
    state: std::sync::Mutex<InMemoryInboxState>,
}

#[derive(Debug, Default, Clone)]
struct InMemoryInboxState {
    rows: Vec<StoredInboxRow>,
    projection: Vec<(Uuid, Uuid, i64)>,
    cursors: Vec<InboxCursor>,
    incidents: Vec<(String, Uuid)>,
}

impl InMemoryCollaborationInbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stored_rows(&self) -> Vec<StoredInboxRow> {
        self.state.lock().expect("inbox state").rows.clone()
    }

    pub fn projection(&self) -> Vec<(Uuid, Uuid, i64)> {
        self.state.lock().expect("inbox state").projection.clone()
    }

    pub fn incidents(&self) -> Vec<(String, Uuid)> {
        self.state.lock().expect("inbox state").incidents.clone()
    }
}

#[async_trait]
impl CollaborationInboxStore for InMemoryCollaborationInbox {
    async fn apply(
        &self,
        received: &ReceivedEvent,
    ) -> Result<ApplyOutcome, CollaborationInboxError> {
        let mut state = self.state.lock().expect("inbox state");
        let existing = state
            .rows
            .iter()
            .find(|row| {
                row.event.stream_id == received.event.stream_id
                    && row.event.source_sequence == received.event.source_sequence
                    && row.event.event_id == received.event.event_id
            })
            .cloned();
        let checkpoint = state
            .cursors
            .iter()
            .find(|cursor| {
                cursor.stream_id == received.event.stream_id
                    && cursor.transport_profile_id == received.transport_profile_id
            })
            .cloned();
        match decide_apply(received, existing.as_ref(), checkpoint.as_ref()) {
            ApplyDecision::Reject { .. } => {
                state.incidents.push((
                    "unknown_schema_version".to_string(),
                    received.event.event_id,
                ));
                Ok(ApplyOutcome::Rejected)
            }
            ApplyDecision::Suspended {
                next_expected_sequence,
            } => Ok(ApplyOutcome::Suspended {
                next_expected_sequence,
            }),
            ApplyDecision::SequenceGap {
                next_expected_sequence,
            } => {
                state
                    .incidents
                    .push(("sequence_gap".to_string(), received.event.event_id));
                let mut cursor = checkpoint.unwrap_or(InboxCursor {
                    stream_id: received.event.stream_id,
                    transport_provider_id: received.transport_provider_id.clone(),
                    transport_profile_id: received.transport_profile_id,
                    cursor_handle: Vec::new(),
                    next_expected_sequence,
                    suspension: None,
                });
                cursor.suspension = Some(SuspensionReason::SequenceGap);
                state.cursors.retain(|c| {
                    !(c.stream_id == cursor.stream_id
                        && c.transport_profile_id == cursor.transport_profile_id)
                });
                state.cursors.push(cursor);
                Ok(ApplyOutcome::SequenceGap {
                    next_expected_sequence,
                })
            }
            ApplyDecision::StaleCursor {
                next_expected_sequence,
            } => {
                state
                    .incidents
                    .push(("stale_cursor".to_string(), received.event.event_id));
                let mut cursor = checkpoint.unwrap_or(InboxCursor {
                    stream_id: received.event.stream_id,
                    transport_provider_id: received.transport_provider_id.clone(),
                    transport_profile_id: received.transport_profile_id,
                    cursor_handle: Vec::new(),
                    next_expected_sequence,
                    suspension: None,
                });
                cursor.suspension = Some(SuspensionReason::StaleCursor);
                state.cursors.retain(|c| {
                    !(c.stream_id == cursor.stream_id
                        && c.transport_profile_id == cursor.transport_profile_id)
                });
                state.cursors.push(cursor);
                Ok(ApplyOutcome::StaleCursor {
                    next_expected_sequence,
                })
            }
            ApplyDecision::ExactReplay { receipt_id } => Ok(ApplyOutcome::ExactReplay {
                receipt_id,
                committed_at: received.event.occurred_at,
            }),
            ApplyDecision::DigestConflict { .. } => {
                state
                    .incidents
                    .push(("digest_conflict".to_string(), received.event.event_id));
                Ok(ApplyOutcome::DigestConflict)
            }
            ApplyDecision::Apply {
                next_expected_sequence,
            } => {
                let row = StoredInboxRow {
                    event: received.event.clone(),
                    event_digest: received.event_digest,
                    receipt_id: Uuid::new_v4(),
                };
                let receipt_id = row.receipt_id;
                state.rows.push(row);
                state.projection.push((
                    received.event.actor_id,
                    received.event.device_id,
                    received.event.device_generation,
                ));
                let mut cursor = checkpoint.unwrap_or(InboxCursor {
                    stream_id: received.event.stream_id,
                    transport_provider_id: received.transport_provider_id.clone(),
                    transport_profile_id: received.transport_profile_id,
                    cursor_handle: Vec::new(),
                    next_expected_sequence: 1,
                    suspension: None,
                });
                // Cursor advance happens only here, together with the inbox row
                // and projection — mirroring the single committed transaction.
                cursor.next_expected_sequence = next_expected_sequence;
                cursor.suspension = None;
                state.cursors.retain(|c| {
                    !(c.stream_id == cursor.stream_id
                        && c.transport_profile_id == cursor.transport_profile_id)
                });
                state.cursors.push(cursor);
                Ok(ApplyOutcome::Committed {
                    receipt_id,
                    committed_at: received.event.occurred_at,
                    next_expected_sequence,
                })
            }
        }
    }

    async fn checkpoint(
        &self,
        _tenant_id: Uuid,
        stream_id: Uuid,
        transport_provider_id: &str,
        transport_profile_id: Uuid,
    ) -> Result<Option<InboxCursor>, CollaborationInboxError> {
        let state = self.state.lock().expect("inbox state");
        Ok(state
            .cursors
            .iter()
            .find(|cursor| {
                cursor.stream_id == stream_id
                    && cursor.transport_provider_id == transport_provider_id
                    && cursor.transport_profile_id == transport_profile_id
            })
            .cloned())
    }
}

/// The receiver-side durable reader for relay iteration: fetch the next
/// pending events after the given sequence, bounded (review 合约 #5/#7 —
/// bounded replay, never an unbounded scan).
#[derive(Debug, Clone, Copy)]
pub struct BoundedReplay {
    /// Maximum events per replay batch.
    pub max_events: usize,
}

impl Default for BoundedReplay {
    fn default() -> Self {
        Self { max_events: 64 }
    }
}

impl BoundedReplay {
    /// Bound a replay request so a broken source cannot force an unbounded
    /// drain; returns the capped count.
    pub fn cap(&self, requested: usize) -> usize {
        requested.min(self.max_events).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_outbox::contract::{
        owner_stream_id, DeviceRevocationEvent, CONSUMER_ID, EVENT_DIGEST_LEN,
        EVENT_TYPE_DEVICE_REVOKED, SCHEMA_VERSION,
    };
    use chrono::{TimeZone, Utc};

    fn received_event(sequence: i64, generation: i64, profile: u128) -> ReceivedEvent {
        let tenant = Uuid::from_u128(0xA1);
        let actor = Uuid::from_u128(0xA2);
        let event = DeviceRevocationEvent {
            event_id: Uuid::from_u128(0xA3),
            event_type: EVENT_TYPE_DEVICE_REVOKED.to_string(),
            schema_version: SCHEMA_VERSION,
            tenant_id: tenant,
            actor_id: actor,
            device_id: Uuid::from_u128(0xA4),
            device_generation: generation,
            stream_id: owner_stream_id(tenant, actor),
            source_sequence: sequence,
            occurred_at: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            correlation_id: Uuid::from_u128(0xA5),
        };
        let digest = event.event_digest();
        ReceivedEvent {
            event,
            event_digest: digest,
            transport_provider_id: "pg-native".to_string(),
            transport_profile_id: Uuid::from_u128(profile),
        }
    }

    #[tokio::test]
    async fn idempotent_replay_returns_existing_receipt_without_duplicate_projection() {
        let inbox = InMemoryCollaborationInbox::new();
        let first = inbox.apply(&received_event(1, 2, 0xB1)).await.unwrap();
        let ApplyOutcome::Committed { receipt_id, .. } = first else {
            panic!("first apply must commit");
        };
        let replay = inbox.apply(&received_event(1, 2, 0xB1)).await.unwrap();
        assert!(matches!(
            replay,
            ApplyOutcome::ExactReplay { receipt_id: replay_receipt, .. } if replay_receipt == receipt_id
        ));
        assert_eq!(inbox.stored_rows().len(), 1, "no duplicate inbox row");
        assert_eq!(inbox.projection().len(), 1, "no duplicate projection row");
        assert_eq!(
            inbox.incidents().len(),
            0,
            "exact replay is not an incident"
        );
    }

    #[tokio::test]
    async fn digest_mismatch_on_same_identity_is_isolated_and_alarmed() {
        let inbox = InMemoryCollaborationInbox::new();
        let first = inbox.apply(&received_event(1, 2, 0xB2)).await.unwrap();
        assert!(matches!(first, ApplyOutcome::Committed { .. }));
        // Same producer + stream + sequence + event_id, but different digest:
        // simulate by mutating the generation (identity fields change → digest
        // changes) while keeping the event id.
        let mut tampered = received_event(1, 2, 0xB2);
        tampered.event.device_generation = 99;
        let outcome = inbox.apply(&tampered).await.unwrap();
        assert_eq!(outcome, ApplyOutcome::DigestConflict);
        assert_eq!(inbox.stored_rows().len(), 1, "conflict must not overwrite");
        assert_eq!(inbox.incidents().len(), 1);
        assert_eq!(inbox.incidents()[0].0, "digest_conflict");
        // The stored projection still reflects the original digest's generation.
        assert_eq!(
            inbox.projection(),
            vec![(Uuid::from_u128(0xA2), Uuid::from_u128(0xA4), 2)]
        );
    }

    #[tokio::test]
    async fn sequence_gap_detection_fails_closed_and_suspends_consumer() {
        let inbox = InMemoryCollaborationInbox::new();
        let first = inbox.apply(&received_event(1, 2, 0xB3)).await.unwrap();
        assert!(matches!(first, ApplyOutcome::Committed { .. }));
        // Event 3 arrives while the cursor expects 2: the gap must fail closed.
        let outcome = inbox.apply(&received_event(3, 4, 0xB3)).await.unwrap();
        assert_eq!(
            outcome,
            ApplyOutcome::SequenceGap {
                next_expected_sequence: 2
            }
        );
        assert_eq!(inbox.incidents().len(), 1);
        assert_eq!(inbox.incidents()[0].0, "sequence_gap");
        assert_eq!(inbox.stored_rows().len(), 1, "gap event must not be stored");
        // After suspension, further applies fail closed with Suspended.
        let suspended = inbox.apply(&received_event(2, 3, 0xB3)).await.unwrap();
        assert!(matches!(suspended, ApplyOutcome::Suspended { .. }));
        assert_eq!(
            inbox.stored_rows().len(),
            1,
            "suspended consumer stores nothing"
        );
    }

    #[tokio::test]
    async fn cursor_advances_only_with_committed_apply() {
        let inbox = InMemoryCollaborationInbox::new();
        for seq in 1..=3 {
            let outcome = inbox
                .apply(&received_event(seq, seq + 1, 0xB4))
                .await
                .unwrap();
            assert!(matches!(outcome, ApplyOutcome::Committed { .. }));
        }
        let cursor = inbox
            .checkpoint(
                Uuid::from_u128(0xA1),
                owner_stream_id(Uuid::from_u128(0xA1), Uuid::from_u128(0xA2)),
                "pg-native",
                Uuid::from_u128(0xB4),
            )
            .await
            .unwrap()
            .expect("cursor exists after applies");
        assert_eq!(cursor.next_expected_sequence, 4);
    }

    #[tokio::test]
    async fn unknown_schema_version_is_rejected_without_cursor_movement() {
        let inbox = InMemoryCollaborationInbox::new();
        let mut unknown = received_event(1, 2, 0xB5);
        unknown.event.schema_version = 99;
        let outcome = inbox.apply(&unknown).await.unwrap();
        assert_eq!(outcome, ApplyOutcome::Rejected);
        assert_eq!(inbox.stored_rows().len(), 0);
        assert_eq!(inbox.incidents().len(), 1);
        assert_eq!(inbox.incidents()[0].0, "unknown_schema_version");
        assert!(
            inbox
                .checkpoint(
                    Uuid::from_u128(0xA1),
                    owner_stream_id(Uuid::from_u128(0xA1), Uuid::from_u128(0xA2)),
                    "pg-native",
                    Uuid::from_u128(0xB5),
                )
                .await
                .unwrap()
                .is_none(),
            "rejected events must not create or advance a cursor"
        );
    }

    #[tokio::test]
    async fn event_behind_cursor_without_stored_receipt_is_retention_gap() {
        let inbox = InMemoryCollaborationInbox::new();
        let first = inbox.apply(&received_event(1, 2, 0xB6)).await.unwrap();
        assert!(matches!(first, ApplyOutcome::Committed { .. }));
        let second = inbox.apply(&received_event(2, 3, 0xB6)).await.unwrap();
        assert!(matches!(second, ApplyOutcome::Committed { .. }));
        // Now deliver sequence 1 again but with a different event_id (so no
        // stored row matches): behind cursor + no receipt → retention gap.
        let mut behind = received_event(1, 2, 0xB6);
        behind.event.event_id = Uuid::from_u128(0xC7);
        let outcome = inbox.apply(&behind).await.unwrap();
        assert!(matches!(outcome, ApplyOutcome::StaleCursor { .. }));
        assert_eq!(inbox.incidents().len(), 1);
        assert_eq!(inbox.incidents()[0].0, "stale_cursor");
        assert_eq!(inbox.stored_rows().len(), 2);
    }

    #[test]
    fn bounded_replay_caps_unbounded_requests() {
        let policy = BoundedReplay { max_events: 64 };
        assert_eq!(policy.cap(1), 1);
        assert_eq!(policy.cap(64), 64);
        assert_eq!(policy.cap(usize::MAX), 64);
        assert_eq!(policy.cap(0), 1);
    }

    #[test]
    fn digest_length_constant_matches_ddl_check() {
        assert_eq!(EVENT_DIGEST_LEN, 32);
        assert_eq!(CONSUMER_ID, "collab.device-auth.v1");
        assert_eq!(PRODUCER_ID, "identity.device-revoke.v1");
    }
}
