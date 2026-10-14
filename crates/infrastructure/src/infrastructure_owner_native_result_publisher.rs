// @cypher schema=1 source_sha256=8f19856368209c161af5db177be341a3eda05c14880ec087a793723f1050aeff
// MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs"})
// MERGE (publisher:Type {id:"infrastructure::InfrastructureOwnerNativeResultPublisher",kind:"struct"})
// MERGE (outcome:Type {id:"infrastructure::NativeResultPublishBatchOutcome",kind:"struct"})
// MERGE (error:Type {id:"infrastructure::NativeResultPublisherError",kind:"enum"})
// MERGE (new:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::InfrastructureOwnerNativeResultPublisher.new",kind:"method"})
// MERGE (publish:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::InfrastructureOwnerNativeResultPublisher.publish_due_for_tenant",kind:"method"})
// MERGE (valid_receipt:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::valid_native_receipt",kind:"function"})
// MERGE (wait_cancel:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::wait_for_cancellation",kind:"function"})
// MERGE (cancel_requested:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::cancellation_requested",kind:"function"})
// MERGE (count_retry:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::count_retry_or_dead",kind:"function"})
// MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"external_type"})
// MERGE (route:Type {id:"infrastructure::NativeInfrastructureDeliveryRoute",kind:"external_type"})
// MERGE (native_append_file:File {path:"crates/platform-messaging-runtime/src/native_append.rs"})
// MERGE (transport:Type {id:"platform_messaging_runtime::native_append::NativeAppendTransport",kind:"external_type"})
// MERGE (append_result:Symbol {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport.append_result",kind:"method"})
// MERGE (transport_profile:Symbol {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport.profile",kind:"method"})
// MERGE (receipt:Type {id:"platform_messaging_runtime::native_append::NativeAppendReceipt",kind:"external_type"})
// MERGE (claim_native:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_native_due",kind:"method"})
// MERGE (pin_wire:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.get_or_pin_native_wire",kind:"method"})
// MERGE (record_failure:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.record_failure",kind:"method"})
// MERGE (mark_delivered:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.mark_native_delivered",kind:"method"})
// MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery",class:"Work"})
// MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox",class:"Transaction"})
// MERGE (route_pin:Table {id:"infrastructure.capacity_outbox_transport_pin",class:"Transaction"})
// MERGE (append_receipt:Table {id:"infrastructure.capacity_native_append_receipt",class:"Transaction"})
// MERGE (test_profile:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::tests::receipt_must_match_exact_profile_wire_and_log_identity",kind:"test"})
// MERGE (test_log:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::tests::receipt_rejects_profile_and_log_id_mismatch",kind:"test"})
// MERGE (self)-[:DEFINES]->(publisher)
// MERGE (self)-[:DEFINES]->(outcome)
// MERGE (self)-[:DEFINES]->(error)
// MERGE (self)-[:DEFINES]->(count_retry)
// MERGE (self)-[:DEFINES]->(cancel_requested)
// MERGE (self)-[:DEFINES]->(valid_receipt)
// MERGE (self)-[:DEFINES]->(wait_cancel)
// MERGE (self)-[:DEFINES]->(test_profile)
// MERGE (self)-[:DEFINES]->(test_log)
// MERGE (publisher)-[:DEFINES]->(new)
// MERGE (publisher)-[:DEFINES]->(publish)
// MERGE (publisher)-[:USES_TYPE]->(store)
// MERGE (publisher)-[:USES_TYPE]->(route)
// MERGE (publisher)-[:USES_TYPE]->(transport)
// MERGE (new)-[:USES_TYPE]->(route)
// MERGE (new)-[:USES_TYPE]->(transport)
// MERGE (self)-[:DEPENDS_ON]->(native_append_file)
// MERGE (native_append_file)-[:DEFINES]->(append_result)
// MERGE (native_append_file)-[:DEFINES]->(transport_profile)
// MERGE (new)-[:CALLS]->(transport_profile)
// MERGE (publish)-[:CALLS]->(claim_native)
// MERGE (publish)-[:CALLS]->(pin_wire)
// MERGE (publish)-[:CALLS]->(append_result)
// MERGE (publish)-[:CALLS]->(record_failure)
// MERGE (publish)-[:CALLS]->(mark_delivered)
// MERGE (publish)-[:CALLS]->(count_retry)
// MERGE (publish)-[:CALLS]->(valid_receipt)
// MERGE (publish)-[:CALLS]->(wait_cancel)
// MERGE (publish)-[:CALLS]->(cancel_requested)
// MERGE (claim_native)-[:READS]->(delivery)
// MERGE (claim_native)-[:READS]->(outbox)
// MERGE (claim_native)-[:READS]->(route_pin)
// MERGE (claim_native)-[:WRITES]->(delivery)
// MERGE (pin_wire)-[:READS]->(delivery)
// MERGE (pin_wire)-[:READS]->(route_pin)
// MERGE (pin_wire)-[:WRITES]->(route_pin)
// MERGE (record_failure)-[:WRITES]->(delivery)
// MERGE (mark_delivered)-[:READS]->(route_pin)
// MERGE (mark_delivered)-[:WRITES]->(delivery)
// MERGE (mark_delivered)-[:WRITES]->(append_receipt)
// MERGE (test_profile)-[:TESTS]->(valid_receipt)
// MERGE (test_log)-[:TESTS]->(valid_receipt)
// MERGE (publish)-[:EMITS]->(native_result:Topic {id:"platform.native.result.append",kind:"topic"})
// MERGE (valid_receipt)-[:USES_TYPE]->(receipt)
// @endcypher
//! Explicitly driven, bounded Infrastructure publisher for exact native capacity-result appends.

use std::time::Duration;

use platform_messaging_runtime::native_append::{
    NativeAppendError, NativeAppendReceipt, NativeAppendTransport,
};
use thiserror::Error;
use tokio::sync::watch;
use uuid::Uuid;

use crate::infrastructure_owner_outbox_delivery::{
    InfrastructureOwnerOutboxDeliveryError, NativeInfrastructureDeliveryRoute,
    OutboxDeliveryFailureKind, PostgresInfrastructureOwnerOutboxDelivery,
};

const MAX_NATIVE_PUBLISH_BATCH_SIZE: i64 = 8;
const MIN_NATIVE_PUBLISH_LEASE_SECONDS: i64 = 120;
const MAX_NATIVE_PUBLISH_LEASE_SECONDS: i64 = 300;
const MAX_DELIVERY_ATTEMPTS: i32 = 12;
const NATIVE_APPEND_DEADLINE: Duration = Duration::from_secs(10);

/// Result counts for one bounded, explicitly invoked publisher pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeResultPublishBatchOutcome {
    /// Number of current-fence Outbox events claimed.
    pub claimed: usize,
    /// Number of exact native append receipts persisted with delivery completion.
    pub appended: usize,
    /// Number of appends that reused the native server's exact prior receipt.
    pub exact_replays: usize,
    /// Number of transient failures scheduled for a bounded retry.
    pub retry_scheduled: usize,
    /// Number of invalid or exhausted events moved to the terminal dead state.
    pub dead_lettered: usize,
    /// Whether cancellation stopped this pass before another event started.
    pub cancelled: bool,
}

/// Errors from native route setup or the Infrastructure owner delivery store.
#[derive(Debug, Error)]
pub enum NativeResultPublisherError {
    /// The selected native client and trusted route do not describe the same profile.
    #[error("native result transport profile does not match the trusted route")]
    ProfileMismatch,
    /// The Infrastructure owner store rejected a lease, route pin, receipt, or transaction.
    #[error(transparent)]
    DeliveryStore(#[from] InfrastructureOwnerOutboxDeliveryError),
}

/// Owner-local native publisher with no background task, tenant discovery, or NATS fallback.
pub struct InfrastructureOwnerNativeResultPublisher<T> {
    delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
    transport: T,
    route: NativeInfrastructureDeliveryRoute,
}

impl<T: NativeAppendTransport> InfrastructureOwnerNativeResultPublisher<T> {
    /// Construct from an already authenticated client and a trusted, immutable route snapshot.
    pub fn new(
        delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
        transport: T,
        route: NativeInfrastructureDeliveryRoute,
    ) -> Result<Self, NativeResultPublisherError> {
        if transport.profile() != &route.profile {
            return Err(NativeResultPublisherError::ProfileMismatch);
        }
        Ok(Self {
            delivery_store,
            transport,
            route,
        })
    }

    /// Claim, pin, append, and persist receipts for one bounded tenant batch.
    ///
    /// The exact route and bytes are persisted before any external append. A timeout or
    /// cancellation is ambiguous and therefore retries only the same event ID and bytes on the
    /// same profile. NATS is never selected as a fallback after a native route is pinned.
    pub async fn publish_due_for_tenant(
        &self,
        tenant_id: Uuid,
        lease_owner_id: Uuid,
        max_items: i64,
        lease_seconds: i64,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<NativeResultPublishBatchOutcome, NativeResultPublisherError> {
        if tenant_id.is_nil()
            || lease_owner_id.is_nil()
            || !(1..=MAX_NATIVE_PUBLISH_BATCH_SIZE).contains(&max_items)
            || !(MIN_NATIVE_PUBLISH_LEASE_SECONDS..=MAX_NATIVE_PUBLISH_LEASE_SECONDS)
                .contains(&lease_seconds)
        {
            return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters.into());
        }
        let claims = self
            .delivery_store
            .claim_native_due(
                tenant_id,
                lease_owner_id,
                max_items,
                lease_seconds,
                &self.route,
            )
            .await?;
        let mut outcome = NativeResultPublishBatchOutcome {
            claimed: claims.len(),
            ..NativeResultPublishBatchOutcome::default()
        };

        for claim in claims {
            if cancellation_requested(cancellation) {
                self.delivery_store
                    .record_failure(&claim, OutboxDeliveryFailureKind::Transient)
                    .await?;
                outcome.cancelled = true;
                count_retry_or_dead(&mut outcome, claim.attempt_count);
                break;
            }

            let pinned_wire = match self
                .delivery_store
                .get_or_pin_native_wire(&claim, &self.route)
                .await
            {
                Ok(wire) => wire,
                Err(InfrastructureOwnerOutboxDeliveryError::NativeWireInvalid) => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Permanent)
                        .await?;
                    outcome.dead_lettered += 1;
                    continue;
                }
                Err(error) => return Err(error.into()),
            };

            let appended = tokio::select! {
                biased;
                _ = wait_for_cancellation(cancellation) => None,
                response = tokio::time::timeout(
                    NATIVE_APPEND_DEADLINE,
                    self.transport.append_result(
                        self.route.recipient_service_id,
                        claim.event_id,
                        &pinned_wire.payload,
                    ),
                ) => Some(response),
            };

            let receipt = match appended {
                None => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Transient)
                        .await?;
                    outcome.cancelled = true;
                    count_retry_or_dead(&mut outcome, claim.attempt_count);
                    break;
                }
                Some(Err(_))
                | Some(Ok(Err(NativeAppendError::Busy | NativeAppendError::Transport))) => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Transient)
                        .await?;
                    count_retry_or_dead(&mut outcome, claim.attempt_count);
                    continue;
                }
                Some(Ok(Err(NativeAppendError::InvalidInput))) => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Permanent)
                        .await?;
                    outcome.dead_lettered += 1;
                    continue;
                }
                Some(Ok(Err(NativeAppendError::InvalidReceipt))) => {
                    self.delivery_store
                        .record_failure(&claim, OutboxDeliveryFailureKind::Transient)
                        .await?;
                    count_retry_or_dead(&mut outcome, claim.attempt_count);
                    continue;
                }
                Some(Ok(Ok(receipt))) => receipt,
            };

            if !valid_native_receipt(&receipt, &self.route, &pinned_wire.sha256) {
                self.delivery_store
                    .record_failure(&claim, OutboxDeliveryFailureKind::Transient)
                    .await?;
                count_retry_or_dead(&mut outcome, claim.attempt_count);
                continue;
            }
            self.delivery_store
                .mark_native_delivered(&claim, &self.route, &receipt)
                .await?;
            outcome.appended += 1;
            if receipt.exact_replay {
                outcome.exact_replays += 1;
            }
        }

        Ok(outcome)
    }
}

fn count_retry_or_dead(outcome: &mut NativeResultPublishBatchOutcome, attempt_count: i32) {
    if attempt_count >= MAX_DELIVERY_ATTEMPTS {
        outcome.dead_lettered += 1;
    } else {
        outcome.retry_scheduled += 1;
    }
}

fn valid_native_receipt(
    receipt: &NativeAppendReceipt,
    route: &NativeInfrastructureDeliveryRoute,
    expected_sha256: &[u8; 32],
) -> bool {
    receipt.profile == route.profile
        && receipt.wire_sha256 == *expected_sha256
        && !receipt.log_id.is_empty()
        && receipt.log_id.len() <= 128
        && receipt.log_id.as_bytes()[0].is_ascii_alphanumeric()
        && receipt
            .log_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
        && !receipt.log_generation.is_nil()
        && receipt.sequence > 0
}

fn cancellation_requested(cancellation: &mut watch::Receiver<bool>) -> bool {
    *cancellation.borrow_and_update() || cancellation.has_changed().is_err()
}

async fn wait_for_cancellation(cancellation: &mut watch::Receiver<bool>) {
    loop {
        if cancellation.changed().await.is_err() || *cancellation.borrow_and_update() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use platform_messaging_runtime::admission_delivery::AdmissionTransportProfile;

    fn route() -> NativeInfrastructureDeliveryRoute {
        NativeInfrastructureDeliveryRoute {
            profile_id: Uuid::from_u128(1),
            profile_revision: 3,
            profile: AdmissionTransportProfile::new(
                "star-rust-log".to_owned(),
                "1.0.0".to_owned(),
                [7; 32],
                "local-star-rust-log".to_owned(),
            )
            .expect("valid trusted profile"),
            sender_service_id: Uuid::from_u128(2),
            recipient_service_id: Uuid::from_u128(3),
        }
    }

    fn receipt(profile: AdmissionTransportProfile, digest: [u8; 32]) -> NativeAppendReceipt {
        NativeAppendReceipt {
            profile,
            log_id: "run-results-v1".to_owned(),
            log_generation: Uuid::from_u128(4),
            sequence: 1,
            exact_replay: false,
            wire_sha256: digest,
        }
    }

    #[test]
    fn receipt_must_match_exact_profile_wire_and_log_identity() {
        let route = route();
        let digest = [9; 32];
        assert!(valid_native_receipt(
            &receipt(route.profile.clone(), digest),
            &route,
            &digest,
        ));

        let mut mismatched = receipt(route.profile.clone(), digest);
        mismatched.wire_sha256 = [8; 32];
        assert!(!valid_native_receipt(&mismatched, &route, &digest));

        let mut malformed = receipt(route.profile.clone(), digest);
        malformed.sequence = 0;
        assert!(!valid_native_receipt(&malformed, &route, &digest));
    }

    #[test]
    fn receipt_rejects_profile_and_log_id_mismatch() {
        let route = route();
        let digest = [5; 32];
        let other_profile = AdmissionTransportProfile::new(
            "star-rust-log".to_owned(),
            "1.0.1".to_owned(),
            [6; 32],
            "local-star-rust-log".to_owned(),
        )
        .expect("valid trusted profile");
        assert!(!valid_native_receipt(
            &receipt(other_profile, digest),
            &route,
            &digest,
        ));

        let mut malformed = receipt(route.profile.clone(), digest);
        malformed.log_id = "../foreign-log".to_owned();
        assert!(!valid_native_receipt(&malformed, &route, &digest));
    }
}
