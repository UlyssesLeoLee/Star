// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=fb25f6e8d8ab9d0b60c1949d0f86c7a13d5b97f03fb7d45b46ac451eec5bbdb4
MERGE (self:File {path:"crates/star-api-rest/src/device_outbox/contract.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::module",kind:"module"})
MERGE (event:Type {id:"crates/star-api-rest/src/device_outbox/contract.rs::DeviceRevocationEvent",kind:"struct"})
MERGE (cursor:Type {id:"crates/star-api-rest/src/device_outbox/contract.rs::InboxCursor",kind:"struct"})
MERGE (stored:Type {id:"crates/star-api-rest/src/device_outbox/contract.rs::StoredInboxRow",kind:"struct"})
MERGE (decision:Type {id:"crates/star-api-rest/src/device_outbox/contract.rs::ApplyDecision",kind:"enum"})
MERGE (error:Type {id:"crates/star-api-rest/src/device_outbox/contract.rs::EnvelopeError",kind:"enum"})
MERGE (digest:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::event_digest",kind:"function"})
MERGE (streamId:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::owner_stream_id",kind:"function"})
MERGE (validate:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::validate_envelope",kind:"function"})
MERGE (decide:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::decide_apply",kind:"function"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(event),(module)-[:DEFINES]->(cursor),(module)-[:DEFINES]->(stored),(module)-[:DEFINES]->(decision),(module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(digest),(module)-[:DEFINES]->(streamId),(module)-[:DEFINES]->(validate),(module)-[:DEFINES]->(decide)
MERGE (digest)-[:USES_TYPE]->(event),(streamId)-[:RETURNS_TYPE]->(cursor)
MERGE (validate)-[:USES_TYPE]->(event),(decide)-[:USES_TYPE]->(event),(decide)-[:USES_TYPE]->(stored),(decide)-[:USES_TYPE]->(cursor),(decide)-[:RETURNS_TYPE]->(decision)
@endcypher
*/
//! Pure domain contract for the durable Identity Outbox → Collaboration Inbox chain
//! (PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106 推荐 owner 合约 #3–#5).
//!
//! The event envelope carries metadata only: stable `event_id`, `event_type`,
//! `schema_version`, `tenant_id`, opaque `actor_id`/`device_id`, the new
//! `device_generation`, `stream_id`, owner-monotonic `source_sequence`,
//! `occurred_at` and a `correlation_id`. No token, device private key/proof,
//! body or plaintext DEK may ever enter an event.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Stable producer identity for the Identity/Auth owner revocation stream.
pub const PRODUCER_ID: &str = "identity.device-revoke.v1";
/// Stable consumer identity of the Collaboration owner's durable Inbox.
pub const CONSUMER_ID: &str = "collab.device-auth.v1";
/// Only known event type in the frozen-enough candidate contract.
pub const EVENT_TYPE_DEVICE_REVOKED: &str = "device.revoked";
/// Only known schema version of the candidate contract.
pub const SCHEMA_VERSION: i32 = 1;
/// SHA-256 digest length in bytes.
pub const EVENT_DIGEST_LEN: usize = 32;
/// Upper bound on the opaque cursor handle carried in a checkpoint.
pub const MAX_CURSOR_HANDLE_LEN: usize = 128;

/// Metadata-only device revocation event emitted by the Identity owner and
/// applied by the Collaboration owner's Inbox. Review 合约 #3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRevocationEvent {
    /// Stable event identity; unique per producer stream.
    pub event_id: Uuid,
    /// `device.revoked` in schema v1; unknown values fail closed.
    pub event_type: String,
    /// Envelope schema version; unknown versions fail closed.
    pub schema_version: i32,
    /// Tenant scope; must match the receiver's active tenant scope.
    pub tenant_id: Uuid,
    /// Opaque actor (device owner) identifier.
    pub actor_id: Uuid,
    /// Opaque device identifier.
    pub device_id: Uuid,
    /// New authoritative device generation after the revocation.
    pub device_generation: i64,
    /// Owner-local stream the event belongs to.
    pub stream_id: Uuid,
    /// Owner-internal monotonic sequence within the stream.
    pub source_sequence: i64,
    /// When the revocation fact occurred in the owner transaction.
    pub occurred_at: DateTime<Utc>,
    /// Correlation / idempotency identity of the producing operation.
    pub correlation_id: Uuid,
}

impl DeviceRevocationEvent {
    /// Canonical SHA-256 digest over the full metadata envelope (review 合约 #4:
    /// receivers pin the digest and isolate same-identity/different-digest replays).
    pub fn event_digest(&self) -> [u8; EVENT_DIGEST_LEN] {
        event_digest(self)
    }
}

/// Canonical envelope digest. Field values are domain-separated; the digest
/// covers every identity and content field of the event so that any mismatch
/// between two deliveries of "the same" event is detectable.
pub fn event_digest(event: &DeviceRevocationEvent) -> [u8; EVENT_DIGEST_LEN] {
    let mut hasher = Sha256::new();
    hasher.update(b"identity-device-revoke|v1|");
    hasher.update(event.producer_seed());
    hasher.update(event.event_type.as_bytes());
    hasher.update(b"|v");
    hasher.update(event.schema_version.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(event.tenant_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.actor_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.device_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.device_generation.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(event.stream_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.source_sequence.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(event.event_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.correlation_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.occurred_at.to_rfc3339().as_bytes());
    hasher.finalize().into()
}

impl DeviceRevocationEvent {
    fn producer_seed(&self) -> &'static [u8] {
        // The producer identity is fixed by contract; kept as a domain-separated
        // constant so the digest never depends on caller-supplied producer text.
        PRODUCER_ID.as_bytes()
    }
}

/// Deterministic owner-local stream identifier for a (tenant, actor) pair.
/// Derived from a domain-separated SHA-256 so every Identity owner operation
/// and every consumer agrees on the same stream id without a shared registry.
pub fn owner_stream_id(tenant_id: Uuid, actor_id: Uuid) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(b"identity.device-revoke.stream.v1|");
    hasher.update(tenant_id.as_bytes());
    hasher.update(b"|");
    hasher.update(actor_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

/// Envelope validation failures; every one of them fails closed (review 合约 #6).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EnvelopeError {
    #[error("unknown event type {0:?}")]
    UnknownEventType(String),
    #[error("unknown schema version {0}")]
    UnknownSchemaVersion(i32),
    #[error("invalid envelope field: {0}")]
    InvalidField(&'static str),
}

/// Validate the candidate contract fields; unknown type/version and structural
/// invalidity are rejected — never silently accepted.
pub fn validate_envelope(event: &DeviceRevocationEvent) -> Result<(), EnvelopeError> {
    if event.event_type != EVENT_TYPE_DEVICE_REVOKED {
        return Err(EnvelopeError::UnknownEventType(event.event_type.clone()));
    }
    if event.schema_version != SCHEMA_VERSION {
        return Err(EnvelopeError::UnknownSchemaVersion(event.schema_version));
    }
    if event.tenant_id.is_nil() {
        return Err(EnvelopeError::InvalidField("tenant_id"));
    }
    if event.actor_id.is_nil() {
        return Err(EnvelopeError::InvalidField("actor_id"));
    }
    if event.device_id.is_nil() {
        return Err(EnvelopeError::InvalidField("device_id"));
    }
    if event.stream_id.is_nil() {
        return Err(EnvelopeError::InvalidField("stream_id"));
    }
    if event.event_id.is_nil() {
        return Err(EnvelopeError::InvalidField("event_id"));
    }
    if event.correlation_id.is_nil() {
        return Err(EnvelopeError::InvalidField("correlation_id"));
    }
    if event.device_generation <= 0 {
        return Err(EnvelopeError::InvalidField("device_generation"));
    }
    if event.source_sequence <= 0 {
        return Err(EnvelopeError::InvalidField("source_sequence"));
    }
    Ok(())
}

/// Consumer checkpoint: an opaque, transport-profile-bound recovery handle
/// (review 合约 #5). Cursors must never be compared or reused across
/// providers/profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxCursor {
    pub stream_id: Uuid,
    pub transport_provider_id: String,
    pub transport_profile_id: Uuid,
    /// Opaque bytes owned by the transport/profile pair; never interpreted by
    /// another provider.
    pub cursor_handle: Vec<u8>,
    /// Next source sequence the consumer expects; applies are only accepted at
    /// exactly this sequence, so gaps fail closed instead of silently skipping.
    pub next_expected_sequence: i64,
    /// Why the consumer is currently suspended, if it is (review 合约 #5).
    pub suspension: Option<SuspensionReason>,
}

impl InboxCursor {
    /// The cursor may only be used while the transport provider and profile
    /// both match the ones it was created for (review 合约 #5/#7).
    pub fn is_bound_to(&self, transport_provider_id: &str, transport_profile_id: Uuid) -> bool {
        self.transport_provider_id == transport_provider_id
            && self.transport_profile_id == transport_profile_id
    }

    /// A cursor handle is only valid when it is bound to a concrete profile.
    pub fn handle_len_ok(&self) -> bool {
        self.cursor_handle.len() <= MAX_CURSOR_HANDLE_LEN
    }
}

/// A row the receiver already stored for the same (producer, stream, sequence,
/// event_id) identity — used for exact-replay detection and digest isolation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredInboxRow {
    pub event: DeviceRevocationEvent,
    pub event_digest: [u8; EVENT_DIGEST_LEN],
    pub receipt_id: Uuid,
}

impl StoredInboxRow {
    /// Exact replay (review 合约 #4): same identity and same digest → the
    /// stored receipt is returned and nothing is overwritten.
    pub fn is_exact_replay_of(&self, received: &ReceivedEvent) -> bool {
        self.event == received.event && self.event_digest == received.event_digest
    }

    /// Same identity, different digest: must be isolated and alarmed, never
    /// overwritten (review 合约 #4).
    pub fn digest_conflicts_with(&self, received: &ReceivedEvent) -> bool {
        self.event.event_id == received.event.event_id
            && self.event.stream_id == received.event.stream_id
            && self.event.source_sequence == received.event.source_sequence
            && self.event_digest != received.event_digest
    }
}

/// An event as handed to the receiver over one transport profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedEvent {
    pub event: DeviceRevocationEvent,
    /// Digest computed from the received envelope by the receiver itself; the
    /// producer never ships the digest, so a digest mismatch detects both
    /// corruption and identity confusion.
    pub event_digest: [u8; EVENT_DIGEST_LEN],
    pub transport_provider_id: String,
    pub transport_profile_id: Uuid,
}

/// Receiver-side decision for one received event, computed before any write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyDecision {
    /// Envelope fails the candidate contract → reject, alarm, never apply.
    Reject { reason: EnvelopeError },
    /// The consumer checkpoint is suspended (gap/stale cursor) → fail closed
    /// until reconciliation; never treat absence of progress as authorization.
    Suspended { next_expected_sequence: i64 },
    /// `source_sequence > next_expected_sequence` → gap detected (review 合约 #5):
    /// pause the projection, record an incident, do not advance.
    SequenceGap { next_expected_sequence: i64 },
    /// `source_sequence < next_expected_sequence` with no stored row → retention
    /// gap: the missing fact can no longer be verified → fail closed.
    StaleCursor { next_expected_sequence: i64 },
    /// Same identity, same digest → return the stored receipt, overwrite nothing.
    ExactReplay { receipt_id: Uuid },
    /// Same identity, different digest → isolate and alarm, never overwrite.
    DigestConflict { receipt_id: Uuid },
    /// New event at exactly the expected sequence → apply, then advance the
    /// cursor inside the same committed transaction.
    Apply { next_expected_sequence: i64 },
}

/// Pure apply policy shared by the PostgreSQL adapter and the in-memory test
/// double. The SQL function `commit_device_revoke_event_v1` mirrors this logic
/// so the durable path and the unit path cannot drift on the outcome matrix.
pub fn decide_apply(
    received: &ReceivedEvent,
    existing: Option<&StoredInboxRow>,
    checkpoint: Option<&InboxCursor>,
) -> ApplyDecision {
    if let Err(reason) = validate_envelope(&received.event) {
        return ApplyDecision::Reject { reason };
    }
    let expected = checkpoint
        .map(|cursor| cursor.next_expected_sequence)
        .unwrap_or(1);
    if let Some(cursor) = checkpoint {
        if !cursor.handle_len_ok() {
            return ApplyDecision::Reject {
                reason: EnvelopeError::InvalidField("cursor_handle"),
            };
        }
        if cursor.suspension.is_some() {
            return ApplyDecision::Suspended {
                next_expected_sequence: expected,
            };
        }
    }
    if let Some(row) = existing {
        if row.is_exact_replay_of(received) {
            return ApplyDecision::ExactReplay {
                receipt_id: row.receipt_id,
            };
        }
        if row.digest_conflicts_with(received) {
            return ApplyDecision::DigestConflict {
                receipt_id: row.receipt_id,
            };
        }
        // Same identity, different content but same digest is impossible for
        // SHA-256-collision-free inputs; treat any non-exact replay of an
        // existing identity as a conflict (fail closed, no overwrite).
        return ApplyDecision::DigestConflict {
            receipt_id: row.receipt_id,
        };
    }
    if received.event.source_sequence > expected {
        return ApplyDecision::SequenceGap {
            next_expected_sequence: expected,
        };
    }
    if received.event.source_sequence < expected {
        return ApplyDecision::StaleCursor {
            next_expected_sequence: expected,
        };
    }
    ApplyDecision::Apply {
        next_expected_sequence: expected + 1,
    }
}

/// Why a consumer checkpoint is suspended (review 合约 #5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspensionReason {
    SequenceGap,
    StaleCursor,
    RetentionGap,
}

impl SuspensionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            SuspensionReason::SequenceGap => "sequence_gap",
            SuspensionReason::StaleCursor => "stale_cursor",
            SuspensionReason::RetentionGap => "retention_gap",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn event(source_sequence: i64, generation: i64) -> DeviceRevocationEvent {
        let tenant = Uuid::from_u128(0x11);
        let actor = Uuid::from_u128(0x22);
        DeviceRevocationEvent {
            event_id: Uuid::from_u128(0x33),
            event_type: EVENT_TYPE_DEVICE_REVOKED.to_string(),
            schema_version: SCHEMA_VERSION,
            tenant_id: tenant,
            actor_id: actor,
            device_id: Uuid::from_u128(0x44),
            device_generation: generation,
            stream_id: owner_stream_id(tenant, actor),
            source_sequence,
            occurred_at: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            correlation_id: Uuid::from_u128(0x55),
        }
    }

    fn received(seq: i64) -> ReceivedEvent {
        let event = event(seq, 2);
        let digest = event.event_digest();
        ReceivedEvent {
            event,
            event_digest: digest,
            transport_provider_id: "pg-native".to_string(),
            transport_profile_id: Uuid::from_u128(0x66),
        }
    }

    #[test]
    fn digest_is_stable_and_covers_identity_fields() {
        let event = event(1, 2);
        assert_eq!(event.event_digest(), event_digest(&event));
        let mut changed = event.clone();
        changed.device_generation = 3;
        assert_ne!(event.event_digest(), changed.event_digest());
        let mut changed = event.clone();
        changed.device_id = Uuid::from_u128(0x77);
        assert_ne!(event.event_digest(), changed.event_digest());
    }

    #[test]
    fn owner_stream_id_is_deterministic_and_owner_scoped() {
        let tenant = Uuid::from_u128(0x1);
        let actor = Uuid::from_u128(0x2);
        assert_eq!(
            owner_stream_id(tenant, actor),
            owner_stream_id(tenant, actor)
        );
        assert_ne!(
            owner_stream_id(tenant, actor),
            owner_stream_id(tenant, Uuid::from_u128(0x3))
        );
        assert_ne!(
            owner_stream_id(tenant, actor),
            owner_stream_id(Uuid::from_u128(0x9), actor)
        );
    }

    #[test]
    fn unknown_event_type_and_schema_version_fail_closed() {
        let mut unknown_type = event(1, 2);
        unknown_type.event_type = "device.enrolled".to_string();
        assert!(matches!(
            validate_envelope(&unknown_type),
            Err(EnvelopeError::UnknownEventType(_))
        ));
        let mut unknown_version = event(1, 2);
        unknown_version.schema_version = 2;
        assert!(matches!(
            validate_envelope(&unknown_version),
            Err(EnvelopeError::UnknownSchemaVersion(_))
        ));
    }

    #[test]
    fn first_event_at_sequence_one_is_applied_and_advances_cursor() {
        let received = received(1);
        let decision = decide_apply(&received, None, None);
        assert_eq!(
            decision,
            ApplyDecision::Apply {
                next_expected_sequence: 2
            }
        );
    }

    #[test]
    fn cursor_is_profile_bound_and_rejects_cross_profile_reuse() {
        let cursor = InboxCursor {
            stream_id: Uuid::from_u128(0x1),
            transport_provider_id: "pg-native".to_string(),
            transport_profile_id: Uuid::from_u128(0x2),
            cursor_handle: vec![0xAB; 16],
            next_expected_sequence: 3,
            suspension: None,
        };
        assert!(cursor.is_bound_to("pg-native", Uuid::from_u128(0x2)));
        assert!(!cursor.is_bound_to("pg-native", Uuid::from_u128(0x3)));
        assert!(!cursor.is_bound_to("kafka", Uuid::from_u128(0x2)));
    }

    #[test]
    fn cursor_handle_is_bounded() {
        let mut cursor = InboxCursor {
            stream_id: Uuid::from_u128(0x1),
            transport_provider_id: "pg-native".to_string(),
            transport_profile_id: Uuid::from_u128(0x2),
            cursor_handle: vec![0; MAX_CURSOR_HANDLE_LEN],
            next_expected_sequence: 3,
            suspension: None,
        };
        assert!(cursor.handle_len_ok());
        cursor.cursor_handle.push(0);
        assert!(!cursor.handle_len_ok());
    }
}
