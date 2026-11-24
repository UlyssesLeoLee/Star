// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/src/defect_outbox/contract.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/defect_outbox/contract.rs::contract",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
*/
//! Defect Outbox contract: envelope format, event types, validation, and apply decision.
//!
//! Based on `device_outbox::contract` pattern, adapted for Defect domain events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Producer identity (Defect owner service)
pub const PRODUCER_ID: &str = "defect.owner";

/// Consumer identity (Notification owner)
pub const CONSUMER_ID: &str = "notification.owner";

/// Schema version for envelope compatibility
pub const SCHEMA_VERSION: u8 = 1;

/// Event digest length (SHA-256)
pub const EVENT_DIGEST_LEN: usize = 32;

/// Event types emitted by Defect owner
pub const EVENT_TYPE_DEFECT_CREATED: &str = "defect.created";
pub const EVENT_TYPE_DEFECT_TRANSITIONED: &str = "defect.transitioned";
pub const EVENT_TYPE_VERIFICATION_SUBMITTED: &str = "defect.verification_submitted";
pub const EVENT_TYPE_VERIFICATION_PASSED: &str = "defect.verification_passed";
pub const EVENT_TYPE_VERIFICATION_FAILED: &str = "defect.verification_failed";

/// Envelope error codes
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EnvelopeError {
    #[error("unsupported schema version: {0}")]
    UnsupportedSchemaVersion(u8),
    #[error("unknown event type: {0}")]
    UnknownEventType(String),
    #[error("malformed payload: {0}")]
    MalformedPayload(String),
    #[error("digest mismatch: expected {expected:?}, got {actual:?}")]
    DigestMismatch { expected: [u8; EVENT_DIGEST_LEN], actual: [u8; EVENT_DIGEST_LEN] },
}

/// Inbox cursor for durable checkpoint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxCursor {
    pub consumer_id: String,
    pub producer_id: String,
    pub sequence: i64,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Received event from outbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedEvent {
    pub envelope: Envelope,
    pub payload: Value,
}

/// Envelope wrapping an outbox event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub schema_version: u8,
    pub producer_id: String,
    pub event_type: String,
    pub sequence: i64,
    pub event_id: Uuid,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub digest: [u8; EVENT_DIGEST_LEN],
    pub payload: Value,
}

/// Stored inbox row (persisted in Inbox table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredInboxRow {
    pub sequence: i64,
    pub event_type: String,
    pub event_id: Uuid,
    pub correlation_id: Uuid,
    pub payload_digest: [u8; EVENT_DIGEST_LEN],
    pub payload: Value,
    pub received_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
    pub receipt_id: Option<uuid::Uuid>,
}

/// Defect-specific outbox events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DefectOutboxEvent {
    #[serde(rename = "defect.created")]
    DefectCreated {
        defect_id: Uuid,
        tenant_id: Uuid,
        project_id: Uuid,
        title: String,
        severity: String,
        priority: String,
    },
    #[serde(rename = "defect.transitioned")]
    DefectTransitioned {
        defect_id: Uuid,
        from: String,
        to: String,
        version: u32,
    },
    #[serde(rename = "defect.verification_submitted")]
    VerificationSubmitted {
        defect_id: Uuid,
        verification_id: Uuid,
        result: String,
    },
    #[serde(rename = "defect.verification_passed")]
    VerificationPassed {
        defect_id: Uuid,
        fix_version: String,
        result: String,
        verification_id: Uuid,
    },
    #[serde(rename = "defect.verification_failed")]
    VerificationFailed {
        defect_id: Uuid,
        fix_version: String,
        result: String,
        verification_id: Uuid,
    },
}

/// Suspension reason for inbox checkpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuspensionReason {
    SequenceGap,
    DigestConflict,
    StaleCursor,
    InvalidEnvelope,
}

/// Apply decision from contract validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyDecision {
    Accept,
    Replay,
    DigestConflict,
    SequenceGap { next_expected: i64 },
    StaleCursor { next_expected: i64 },
    InvalidEnvelope,
}

/// Compute SHA-256 digest of payload
pub fn event_digest(payload: &serde_json::Value) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(payload).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    arr
}

/// Validate envelope structure and digest
pub fn validate_envelope(envelope: &Envelope, payload: &Value) -> Result<(), EnvelopeError> {
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(EnvelopeError::UnsupportedSchemaVersion(envelope.schema_version));
    }
    match envelope.event_type.as_str() {
        EVENT_TYPE_DEFECT_CREATED
        | EVENT_TYPE_DEFECT_TRANSITIONED
        | EVENT_TYPE_VERIFICATION_SUBMITTED
        | EVENT_TYPE_VERIFICATION_PASSED
        | EVENT_TYPE_VERIFICATION_FAILED => {}
        _ => return Err(EnvelopeError::UnknownEventType(envelope.event_type.clone())),
    }
    let expected_digest = event_digest(payload);
    if envelope.digest != expected_digest {
        return Err(EnvelopeError::DigestMismatch {
            expected: expected_digest,
            actual: envelope.digest,
        });
    }
    Ok(())
}

/// Compute owner stream ID for relay/consumer
pub fn owner_stream_id(producer_id: &str, consumer_id: &str, tenant_id: Uuid) -> String {
    format!("{}:{}:{}", producer_id, consumer_id, tenant_id)
}

/// Decide whether to apply a received event
pub fn decide_apply(
    cursor: &InboxCursor,
    event: &ReceivedEvent,
    stored_row: Option<&StoredInboxRow>,
) -> ApplyDecision {
    // 1. Sequence check
    if event.envelope.sequence <= cursor.sequence {
        if let Some(stored) = stored_row {
            if stored.event_id == event.envelope.event_id && stored.payload_digest == event.envelope.digest {
                return ApplyDecision::Replay;
            }
            return ApplyDecision::DigestConflict;
        }
        return ApplyDecision::StaleCursor { next_expected: cursor.sequence + 1 };
    }

    // 2. Gap check
    if event.envelope.sequence > cursor.sequence + 1 {
        return ApplyDecision::SequenceGap { next_expected: cursor.sequence + 1 };
    }

    // 3. Digest conflict with stored row
    if let Some(stored) = stored_row {
        if stored.event_id == event.envelope.event_id && stored.payload_digest != event.envelope.digest {
            return ApplyDecision::DigestConflict;
        }
    }

    ApplyDecision::Accept
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_digest() {
        let payload = json!({"defect_id": "00000000-0000-0000-0000-000000000001"});
        let digest = event_digest(&payload);
        assert_eq!(digest.len(), 32);
    }

    #[test]
    fn test_validate_envelope_ok() {
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            producer_id: PRODUCER_ID.to_string(),
            event_type: EVENT_TYPE_DEFECT_CREATED.to_string(),
            sequence: 1,
            event_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            digest: [0u8; 32], // placeholder
            payload: json!({"defect_id": "00000000-0000-0000-0000-000000000001"}),
        };
        let payload = json!({"defect_id": "00000000-0000-0000-0000-000000000001"});
        let envelope_with_digest = Envelope {
            digest: event_digest(&payload),
            ..envelope
        };
        assert!(validate_envelope(&envelope_with_digest, &payload).is_ok());
    }

    #[test]
    fn test_validate_envelope_bad_digest() {
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            producer_id: PRODUCER_ID.to_string(),
            event_type: EVENT_TYPE_DEFECT_CREATED.to_string(),
            sequence: 1,
            event_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            digest: [0u8; 32],
            payload: json!({"defect_id": "00000000-0000-0000-0000-000000000001"}),
        };
        let payload = json!({"defect_id": "00000000-0000-0000-0000-000000000001"});
        assert!(matches!(validate_envelope(&envelope, &payload), Err(EnvelopeError::DigestMismatch { .. })));
    }

    #[test]
    fn test_validate_envelope_unknown_type() {
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            producer_id: PRODUCER_ID.to_string(),
            event_type: "unknown.event".to_string(),
            sequence: 1,
            event_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            digest: event_digest(&json!({})),
            payload: json!({}),
        };
        assert!(matches!(validate_envelope(&envelope, &json!({})), Err(EnvelopeError::UnknownEventType(_))));
    }

    #[test]
    fn test_decide_apply_accept() {
        let cursor = InboxCursor {
            consumer_id: CONSUMER_ID.to_string(),
            producer_id: PRODUCER_ID.to_string(),
            sequence: 0,
            updated_at: Utc::now(),
        };
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            producer_id: PRODUCER_ID.to_string(),
            event_type: EVENT_TYPE_DEFECT_CREATED.to_string(),
            sequence: 1,
            event_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            digest: [0u8; 32],
            payload: json!({}),
        };
        let event = ReceivedEvent { envelope, payload: serde_json::json!({}) };
        let decision = decide_apply(&cursor, &event, None);
        assert!(matches!(decision, ApplyDecision::Accept));
    }

    #[test]
    fn test_decide_apply_sequence_gap() {
        let cursor = InboxCursor {
            consumer_id: CONSUMER_ID.to_string(),
            producer_id: PRODUCER_ID.to_string(),
            sequence: 5,
            updated_at: Utc::now(),
        };
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            producer_id: PRODUCER_ID.to_string(),
            event_type: EVENT_TYPE_DEFECT_CREATED.to_string(),
            sequence: 7,
            event_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            digest: [0u8; 32],
            payload: json!({}),
        };
        let event = ReceivedEvent { envelope, payload: serde_json::json!({}) };
        let decision = decide_apply(&cursor, &event, None);
        assert!(matches!(decision, ApplyDecision::SequenceGap { next_expected: 6 }));
    }
}
