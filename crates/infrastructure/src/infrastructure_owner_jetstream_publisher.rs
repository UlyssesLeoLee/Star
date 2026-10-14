//! Bounded JetStream publication for tenant-scoped Infrastructure Outbox claims.
//!
//! @cypher schema=1 source_sha256=c351c7aaa5f216461616498d9104ea318f7f8424b81d3af5d082b049862b90d7
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::infrastructure_owner_jetstream_publisher",kind:"module"})
//! MERGE (publisher:Type {id:"infrastructure::InfrastructureOwnerJetStreamPublisher",kind:"struct"})
//! MERGE (outcome:Type {id:"infrastructure::JetStreamPublishBatchOutcome",kind:"struct"})
//! MERGE (route_diagnostics:Type {id:"infrastructure::JetStreamNativeRoutePinDiagnosticState",kind:"enum"})
//! MERGE (error:Type {id:"infrastructure::JetStreamPublisherError",kind:"enum"})
//! MERGE (publish_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.publish_due_for_tenant",kind:"method"})
//! MERGE (new_publisher:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.new",kind:"method"})
//! MERGE (from_callback:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.from_publish_callback",kind:"method"})
//! MERGE (with_native_route:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.with_native_route_snapshot",kind:"method"})
//! MERGE (with_test_hook:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.with_test_publish_hook",kind:"method"})
//! MERGE (build_envelope:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::build_envelope",kind:"function"})
//! MERGE (build_publish_message:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::build_publish_message",kind:"function"})
//! MERGE (publish_callback:Type {id:"infrastructure::JetStreamPublishCallback",kind:"type"})
//! MERGE (publish_future:Type {id:"infrastructure::PublishFuture",kind:"type"})
//! MERGE (diagnose_native_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.diagnose_native_due",kind:"method"})
//! MERGE (claim_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_due",kind:"method"})
//! MERGE (mark_delivered:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.mark_delivered",kind:"method"})
//! MERGE (record_failure:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.record_failure",kind:"method"})
//! MERGE (send_publish:ExternalService {id:"async_nats::jetstream::Context::send_publish",kind:"method"})
//! MERGE (delivery_store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"external_type"})
//! MERGE (claim:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryClaim",kind:"external_type"})
//! MERGE (event_wire:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs::build_capacity_result_wire",kind:"function"})
//! MERGE (jetstream:Type {id:"async_nats::jetstream::Context",kind:"external_type"})
//! MERGE (publish_message:Type {id:"async_nats::jetstream::message::PublishMessage",kind:"external_type"})
//! MERGE (module)-[:DEFINES]->(publisher)
//! MERGE (module)-[:DEFINES]->(outcome)
//! MERGE (module)-[:DEFINES]->(route_diagnostics)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (module)-[:DEFINES]->(build_envelope)
//! MERGE (module)-[:DEFINES]->(build_publish_message)
//! MERGE (module)-[:DEFINES]->(publish_callback)
//! MERGE (module)-[:DEFINES]->(publish_future)
//! MERGE (publisher)-[:DEFINES]->(publish_due)
//! MERGE (publisher)-[:DEFINES]->(new_publisher)
//! MERGE (publisher)-[:DEFINES]->(from_callback)
//! MERGE (publisher)-[:DEFINES]->(with_native_route)
//! MERGE (publisher)-[:DEFINES]->(with_test_hook)
//! MERGE (new_publisher)-[:CALLS]->(from_callback)
//! MERGE (from_callback)-[:USES_TYPE]->(publish_callback)
//! MERGE (from_callback)-[:USES_TYPE]->(delivery_store)
//! MERGE (with_test_hook)-[:CALLS]->(from_callback)
//! MERGE (with_test_hook)-[:USES_TYPE]->(publish_callback)
//! MERGE (new_publisher)-[:USES_TYPE]->(jetstream)
//! MERGE (new_publisher)-[:USES_TYPE]->(publish_message)
//! MERGE (new_publisher)-[:CALLS]->(send_publish)
//! MERGE (publish_due)-[:CALLS]->(build_publish_message)
//! MERGE (publish_due)-[:CALLS]->(claim_due)
//! MERGE (publish_due)-[:CALLS]->(mark_delivered)
//! MERGE (publish_due)-[:CALLS]->(record_failure)
//! MERGE (build_envelope)-[:CALLS]->(event_wire)
//! MERGE (build_publish_message)-[:CALLS]->(build_envelope)
//! MERGE (publish_due)-[:USES_TYPE]->(delivery_store)
//! MERGE (publish_due)-[:USES_TYPE]->(claim)
//! MERGE (publish_due)-[:USES_TYPE]->(publish_message)
//! MERGE (publish_due)-[:CALLS]->(diagnose_native_due)
//! MERGE (publish_due)-[:USES_TYPE]->(publish_callback)
//! MERGE (publish_due)-[:RETURNS]->(outcome)
//! MERGE (native_route:Type {id:"infrastructure::NativeInfrastructureDeliveryRoute",kind:"external_type"})
//! MERGE (quarantine_page:Type {id:"infrastructure::NativeRoutePinQuarantinePage",kind:"external_type"})
//! MERGE (publisher)-[:USES_TYPE]->(native_route)
//! MERGE (publisher)-[:USES_TYPE]->(publish_callback)
//! MERGE (publish_callback)-[:USES_TYPE]->(publish_future)
//! MERGE (route_diagnostics)-[:USES_TYPE]->(quarantine_page)
//! MERGE (outcome)-[:USES_TYPE]->(route_diagnostics)
//! MERGE (publish_due)-[:USES_TYPE]->(route_diagnostics)
//! MERGE (publish_due)-[:USES_TYPE]->(publish_future)
//! MERGE (with_native_route)-[:USES_TYPE]->(native_route)
//! @endcypher

use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use async_nats::jetstream::{message::PublishMessage, Context};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "native-grpc-adapter")]
use crate::infrastructure_owner_outbox_delivery::NativeInfrastructureDeliveryRoute;
use crate::infrastructure_owner_outbox_delivery::{
    InfrastructureOwnerOutboxDeliveryClaim, InfrastructureOwnerOutboxDeliveryError,
    NativeRoutePinQuarantinePage, OutboxDeliveryFailureKind,
    PostgresInfrastructureOwnerOutboxDelivery,
};
use crate::infrastructure_owner_outbox_wire::build_capacity_result_wire;

const MAX_PUBLISH_BATCH_SIZE: i64 = 8;
const MIN_PUBLISH_LEASE_SECONDS: i64 = 120;
const MAX_PUBLISH_LEASE_SECONDS: i64 = 300;
const MAX_DELIVERY_ATTEMPTS: i32 = 12;
const MAX_ENVELOPE_BYTES: usize = 128 * 1024;
#[cfg(test)]
const MAX_PAYLOAD_BYTES: usize = 16 * 1024;
const ACK_TIMEOUT: Duration = Duration::from_secs(10);
const SUBJECT_PREFIX: &str = "star.infrastructure.v2";
type PublishFuture = Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'static>>;
type JetStreamPublishCallback = dyn Fn(String, PublishMessage) -> PublishFuture + Send + Sync;

/// Whether the JetStream pass could compare native route pins to a trusted resolved snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum JetStreamNativeRoutePinDiagnosticState {
    /// No resolver-produced snapshot was bound to this publisher, so no blocked count is claimed.
    #[default]
    Unavailable,
    /// Bounded tenant diagnostics from the same Infrastructure owner store used by claims.
    Available {
        /// Total due native-pinned rows observed, capped at the configured sample size.
        due_native_pinned: usize,
        /// True when the bounded due-row sample has more rows beyond this count.
        due_native_pinned_truncated: bool,
        /// Durable, bounded quarantine state and any newly quarantined rows.
        route_pin_quarantine: NativeRoutePinQuarantinePage,
    },
}

/// Tenant-scoped result counts for one bounded publisher pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JetStreamPublishBatchOutcome {
    /// Number of Outbox claims leased from PostgreSQL.
    pub claimed: usize,
    /// Messages acknowledged by JetStream and fenced as delivered in PostgreSQL.
    pub delivered: usize,
    /// Transient failures durably scheduled for retry.
    pub retry_scheduled: usize,
    /// Invalid event contracts and exhausted deliveries moved to terminal dead state.
    pub dead_lettered: usize,
    /// Explicit native route-pin visibility; unavailable is never represented as zero.
    pub native_route_pin_diagnostics: JetStreamNativeRoutePinDiagnosticState,
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
#[derive(Clone)]
pub struct InfrastructureOwnerJetStreamPublisher {
    delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
    publish: Arc<JetStreamPublishCallback>,
    #[cfg(feature = "native-grpc-adapter")]
    native_route_snapshot: Option<NativeInfrastructureDeliveryRoute>,
}

impl InfrastructureOwnerJetStreamPublisher {
    /// Construct the publisher with an established client context and Infrastructure store.
    pub fn new(
        delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
        jetstream: Context,
    ) -> Self {
        let publish = move |subject, message| {
            let jetstream = jetstream.clone();
            Box::pin(async move {
                let ack = jetstream
                    .send_publish(subject, message)
                    .await
                    .map_err(|_| ())?;
                ack.await.map(|_| ()).map_err(|_| ())
            }) as PublishFuture
        };
        Self::from_publish_callback(delivery_store, publish)
    }

    fn from_publish_callback<F>(
        delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
        publish: F,
    ) -> Self
    where
        F: Fn(String, PublishMessage) -> PublishFuture + Send + Sync + 'static,
    {
        Self {
            delivery_store,
            publish: Arc::new(publish),
            #[cfg(feature = "native-grpc-adapter")]
            native_route_snapshot: None,
        }
    }

    /// Bind a resolver-produced route snapshot for tenant-scoped native-pin diagnostics.
    #[cfg(feature = "native-grpc-adapter")]
    pub fn with_native_route_snapshot(mut self, route: NativeInfrastructureDeliveryRoute) -> Self {
        self.native_route_snapshot = Some(route);
        self
    }

    /// Construct a publisher with a bounded fake callback for disposable store integration tests.
    #[cfg(feature = "native-grpc-test-support")]
    pub fn with_test_publish_hook<F, Fut>(
        delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
        publish_hook: F,
    ) -> Self
    where
        F: Fn(String, PublishMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), ()>> + Send + 'static,
    {
        Self::from_publish_callback(delivery_store, move |subject, message| {
            Box::pin(publish_hook(subject, message))
        })
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

        #[cfg(feature = "native-grpc-adapter")]
        let native_route_pin_diagnostics = match self.native_route_snapshot.as_ref() {
            Some(route) if route.tenant_id() == tenant_id => {
                let page = self
                    .delivery_store
                    .diagnose_native_due(lease_owner_id, route)
                    .await?;
                JetStreamNativeRoutePinDiagnosticState::Available {
                    due_native_pinned: page.due_native_pinned,
                    due_native_pinned_truncated: page.due_native_pinned_truncated,
                    route_pin_quarantine: page,
                }
            }
            _ => JetStreamNativeRoutePinDiagnosticState::Unavailable,
        };
        #[cfg(not(feature = "native-grpc-adapter"))]
        let native_route_pin_diagnostics = JetStreamNativeRoutePinDiagnosticState::Unavailable;

        let claims = self
            .delivery_store
            .claim_due(tenant_id, lease_owner_id, max_items, lease_seconds)
            .await?;
        let mut outcome = JetStreamPublishBatchOutcome {
            claimed: claims.len(),
            native_route_pin_diagnostics,
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

            let publish_result =
                tokio::time::timeout(ACK_TIMEOUT, (self.publish)(subject, message))
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

fn build_envelope(
    claim: &InfrastructureOwnerOutboxDeliveryClaim,
) -> Result<(String, Vec<u8>, String), ()> {
    let subject_event = match claim.event_type.as_str() {
        "capacity_accepted" => "capacity_accepted",
        "capacity_rejected" => "capacity_rejected",
        "capacity_expired" => "capacity_expired",
        _ => return Err(()),
    };
    let payload = build_capacity_result_wire(claim).map_err(|_| ())?;
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
    use serde_json::Value;

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
        assert!(build_envelope(&claim("capacity_released", "{}")).is_err());

        let mut unsupported = claim("capacity_rejected", "{}");
        unsupported.schema_version = 3;
        assert!(build_envelope(&unsupported).is_err());
    }

    #[test]
    fn malformed_oversized_and_non_object_payloads_fail_closed() {
        assert!(build_envelope(&claim("capacity_accepted", "not-json")).is_err());
        assert!(build_envelope(&claim("capacity_accepted", "[]")).is_err());
        let oversized = format!("{{\"data\":\"{}\"}}", "x".repeat(MAX_PAYLOAD_BYTES));
        assert!(build_envelope(&claim("capacity_accepted", &oversized)).is_err());
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
