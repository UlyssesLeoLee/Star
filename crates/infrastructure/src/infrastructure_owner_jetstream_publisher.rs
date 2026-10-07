//! Bounded JetStream publication for tenant-scoped Infrastructure Outbox claims.
//!
//! @cypher schema=1 source_sha256=f30d2040868a73d4e7959c569cfabfa8c0762855f2c942cfdd0ed79aa1cf675b
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::infrastructure_owner_jetstream_publisher",kind:"module"})
//! MERGE (publisher:Type {id:"infrastructure::InfrastructureOwnerJetStreamPublisher",kind:"struct"})
//! MERGE (outcome:Type {id:"infrastructure::JetStreamPublishBatchOutcome",kind:"struct"})
//! MERGE (error:Type {id:"infrastructure::JetStreamPublisherError",kind:"enum"})
//! MERGE (publish_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.publish_due_for_tenant",kind:"method"})
//! MERGE (build_envelope:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::build_envelope",kind:"function"})
//! MERGE (delivery_store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"external_type"})
//! MERGE (claim:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryClaim",kind:"external_type"})
//! MERGE (jetstream:Type {id:"async_nats::jetstream::Context",kind:"external_type"})
//! MERGE (publish_message:Type {id:"async_nats::jetstream::message::PublishMessage",kind:"external_type"})
//! MERGE (module)-[:DEFINES]->(publisher)
//! MERGE (module)-[:DEFINES]->(outcome)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (publisher)-[:DEFINES]->(publish_due)
//! MERGE (publish_due)-[:CALLS]->(build_envelope)
//! MERGE (publish_due)-[:USES_TYPE]->(delivery_store)
//! MERGE (publish_due)-[:USES_TYPE]->(claim)
//! MERGE (publish_due)-[:USES_TYPE]->(jetstream)
//! MERGE (publish_due)-[:USES_TYPE]->(publish_message)
//! @endcypher

use std::time::Duration;

use async_nats::jetstream::{message::PublishMessage, Context};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::infrastructure_owner_outbox_delivery::{
    InfrastructureOwnerOutboxDeliveryClaim, InfrastructureOwnerOutboxDeliveryError,
    OutboxDeliveryFailureKind, PostgresInfrastructureOwnerOutboxDelivery,
};

const MAX_PUBLISH_BATCH_SIZE: i64 = 8;
const MIN_PUBLISH_LEASE_SECONDS: i64 = 120;
const MAX_PUBLISH_LEASE_SECONDS: i64 = 300;
const MAX_DELIVERY_ATTEMPTS: i32 = 12;
const MAX_PAYLOAD_BYTES: usize = 16 * 1024;
const MAX_ENVELOPE_BYTES: usize = 128 * 1024;
const ACK_TIMEOUT: Duration = Duration::from_secs(10);
const SUBJECT_PREFIX: &str = "star.infrastructure.v2";

/// Tenant-scoped result counts for one bounded publisher pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JetStreamPublishBatchOutcome {
    /// Number of Outbox claims leased from PostgreSQL.
    pub claimed: usize,
    /// Messages acknowledged by JetStream and fenced as delivered in PostgreSQL.
    pub delivered: usize,
    /// Transient failures durably scheduled for retry.
    pub retry_scheduled: usize,
    /// Invalid event contracts and exhausted deliveries moved to terminal dead state.
    pub dead_lettered: usize,
}

/// Errors from parameter validation or the Infrastructure Outbox owner store.
#[derive(Debug, Error)]
pub enum JetStreamPublisherError {
    /// A batch or lease value exceeds the publisher's bounded processing window.
    #[error("JetStream publisher parameters are invalid")]
    InvalidParameters,
    /// The Infrastructure owner database rejected a claim, fence, or state update.
    #[error(transparent)]
    DeliveryStore(#[from] InfrastructureOwnerOutboxDeliveryError),
}

/// Publishes durable Infrastructure Outbox facts to JetStream one at a time.
///
/// The caller supplies the already-authenticated JetStream context and an owner-scoped
/// delivery store. This adapter never discovers tenants, starts a worker, or configures a
/// broker. A later consumer must still deduplicate the immutable `event_id` in its own Inbox.
#[derive(Debug, Clone)]
pub struct InfrastructureOwnerJetStreamPublisher {
    delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
    jetstream: Context,
}

impl InfrastructureOwnerJetStreamPublisher {
    /// Construct the publisher with an established client context and Infrastructure store.
    pub fn new(
        delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
        jetstream: Context,
    ) -> Self {
        Self {
            delivery_store,
            jetstream,
        }
    }

    /// Lease and publish one tenant's due events with bounded memory and serial PubAck waits.
    ///
    /// PostgreSQL marks an event delivered only after JetStream returns PubAck. A timeout is
    /// ambiguous (the server may already have stored the event), so it is recorded as transient
    /// and retried with the same message ID. The durable Run-owner Inbox remains responsible for
    /// deduplication beyond JetStream's configured duplicate window.
    pub async fn publish_due_for_tenant(
        &self,
        tenant_id: Uuid,
        lease_owner_id: Uuid,
        max_items: i64,
        lease_seconds: i64,
    ) -> Result<JetStreamPublishBatchOutcome, JetStreamPublisherError> {
        if tenant_id.is_nil()
            || lease_owner_id.is_nil()
            || !(1..=MAX_PUBLISH_BATCH_SIZE).contains(&max_items)
            || !(MIN_PUBLISH_LEASE_SECONDS..=MAX_PUBLISH_LEASE_SECONDS).contains(&lease_seconds)
        {
            return Err(JetStreamPublisherError::InvalidParameters);
        }

        let claims = self
            .delivery_store
            .claim_due(tenant_id, lease_owner_id, max_items, lease_seconds)
            .await?;
        let mut outcome = JetStreamPublishBatchOutcome {
            claimed: claims.len(),
            ..JetStreamPublishBatchOutcome::default()
        };

        for claim in claims {
            let (subject, message) = match build_publish_message(&claim) {
                Ok(message) => message,
                Err(()) => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Permanent)
                        .await?;
                    outcome.dead_lettered += 1;
                    continue;
                }
            };

            let publish_result = tokio::time::timeout(ACK_TIMEOUT, async {
                let ack = self
                    .jetstream
                    .send_publish(subject, message)
                    .await
                    .map_err(|_| ())?;
                ack.await.map(|_| ()).map_err(|_| ())
            })
            .await
            .map_err(|_| ())
            .and_then(|result| result);

            match publish_result {
                Ok(()) => {
                    self.delivery_store.mark_delivered(&claim).await?;
                    outcome.delivered += 1;
                }
                Err(()) => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Transient)
                        .await?;
                    if claim.attempt_count >= MAX_DELIVERY_ATTEMPTS {
                        outcome.dead_lettered += 1;
                    } else {
                        outcome.retry_scheduled += 1;
                    }
                }
            }
        }

        Ok(outcome)
    }
}

#[derive(Serialize)]
struct InfrastructureEventEnvelope {
    envelope_version: u8,
    tenant_id: Uuid,
    event_id: Uuid,
    event_type: String,
    schema_version: i16,
    idempotency_key: String,
    correlation_id: Uuid,
    payload: Value,
}

fn build_envelope(
    claim: &InfrastructureOwnerOutboxDeliveryClaim,
) -> Result<(String, Vec<u8>, String), ()> {
    let subject_event = match claim.event_type.as_str() {
        "capacity_accepted" => "capacity_accepted",
        "capacity_rejected" => "capacity_rejected",
        "capacity_released" => "capacity_released",
        "capacity_expired" => "capacity_expired",
        _ => return Err(()),
    };
    if claim.tenant_id.is_nil()
        || claim.event_id.is_nil()
        || claim.lease_owner_id.is_nil()
        || claim.correlation_id.is_nil()
        || claim.attempt_count <= 0
        || claim.schema_version != 2
        || claim.idempotency_key.is_empty()
        || claim.idempotency_key.len() > 200
        || claim.payload_json.len() > MAX_PAYLOAD_BYTES
    {
        return Err(());
    }

    let payload: Value = serde_json::from_str(&claim.payload_json).map_err(|_| ())?;
    if !payload.is_object() {
        return Err(());
    }
    let envelope = InfrastructureEventEnvelope {
        envelope_version: 1,
        tenant_id: claim.tenant_id,
        event_id: claim.event_id,
        event_type: claim.event_type.clone(),
        schema_version: claim.schema_version,
        idempotency_key: claim.idempotency_key.clone(),
        correlation_id: claim.correlation_id,
        payload,
    };
    let payload = serde_json::to_vec(&envelope).map_err(|_| ())?;
    if payload.len() > MAX_ENVELOPE_BYTES {
        return Err(());
    }

    let subject = format!("{SUBJECT_PREFIX}.{subject_event}");
    let message_id = claim.event_id.to_string();
    Ok((subject, payload, message_id))
}

fn build_publish_message(
    claim: &InfrastructureOwnerOutboxDeliveryClaim,
) -> Result<(String, PublishMessage), ()> {
    let (subject, payload, message_id) = build_envelope(claim)?;
    let message = PublishMessage::build()
        .payload(payload.into())
        .message_id(message_id);
    Ok((subject, message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn claim(event_type: &str, payload_json: &str) -> InfrastructureOwnerOutboxDeliveryClaim {
        InfrastructureOwnerOutboxDeliveryClaim {
            tenant_id: Uuid::from_u128(1),
            event_id: Uuid::from_u128(2),
            lease_owner_id: Uuid::from_u128(3),
            attempt_count: 1,
            event_type: event_type.to_owned(),
            schema_version: 2,
            idempotency_key: "capacity-demand:run-1:occurrence-1".to_owned(),
            correlation_id: Uuid::from_u128(4),
            payload_json: payload_json.to_owned(),
            lease_expires_at: Utc::now(),
        }
    }

    #[test]
    fn jetstream_subject_is_allowlisted_and_payload_is_bounded() {
        let (subject, payload, message_id) = build_envelope(&claim(
            "capacity_accepted",
            r#"{"host_reservation_id":"host-1","cluster_reservation_id":"cluster-1"}"#,
        ))
        .expect("valid schema v2 event should produce a JetStream message");
        assert_eq!(subject, "star.infrastructure.v2.capacity_accepted");
        assert_eq!(message_id, Uuid::from_u128(2).to_string());
        let envelope: Value = serde_json::from_slice(&payload).expect("valid JSON envelope");
        assert_eq!(envelope["envelope_version"], 1);
        assert_eq!(envelope["event_id"], Uuid::from_u128(2).to_string());
        assert_eq!(envelope["payload"]["host_reservation_id"], "host-1");
    }

    #[test]
    fn unknown_event_types_and_schema_versions_fail_closed() {
        assert!(build_envelope(&claim("arbitrary.subject", "{}")).is_err());

        let mut unsupported = claim("capacity_rejected", "{}");
        unsupported.schema_version = 3;
        assert!(build_envelope(&unsupported).is_err());
    }

    #[test]
    fn malformed_oversized_and_non_object_payloads_fail_closed() {
        assert!(build_envelope(&claim("capacity_released", "not-json")).is_err());
        assert!(build_envelope(&claim("capacity_released", "[]")).is_err());
        let oversized = format!("{{\"data\":\"{}\"}}", "x".repeat(MAX_PAYLOAD_BYTES));
        assert!(build_envelope(&claim("capacity_released", &oversized)).is_err());
    }

    #[test]
    fn idempotency_identity_and_required_scope_must_be_present() {
        let mut missing_correlation = claim("capacity_expired", "{}");
        missing_correlation.correlation_id = Uuid::nil();
        assert!(build_envelope(&missing_correlation).is_err());

        let mut missing_key = claim("capacity_expired", "{}");
        missing_key.idempotency_key.clear();
        assert!(build_envelope(&missing_key).is_err());
    }
}
