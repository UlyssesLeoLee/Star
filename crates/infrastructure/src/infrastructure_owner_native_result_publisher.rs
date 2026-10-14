// @cypher schema=1 source_sha256=7333460776de9496cd167d62ddca7dd20ddbd5ae3bfc703b0422bafbbe35a27f
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
// MERGE (endpoint:Type {id:"platform_messaging_runtime::transport_endpoint::ResolvedTransportEndpoint",kind:"external_type"})
// MERGE (route_tenant:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.tenant_id",kind:"method"})
// MERGE (route_deployment:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.deployment_id",kind:"method"})
// MERGE (route_profile:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.profile",kind:"method"})
// MERGE (route_recipient:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.recipient_service_id",kind:"method"})
// MERGE (route_test_profile_binding:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.from_resolved_profile_for_test",kind:"method"})
// MERGE (route_new:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.from_resolved_endpoint",kind:"method"})
// MERGE (native_append_file:File {path:"crates/platform-messaging-runtime/src/native_append.rs"})
// MERGE (native_grpc_file:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
// MERGE (admission_delivery_file:File {path:"crates/platform-messaging-runtime/src/admission_delivery.rs"})
// MERGE (transport_endpoint_file:File {path:"crates/platform-messaging-runtime/src/transport_endpoint.rs"})
// MERGE (transport_profile_file:File {path:"crates/platform-messaging-runtime/src/transport_profile.rs"})
// MERGE (transport:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAppendTransport",kind:"external_type"})
// MERGE (append_result:Symbol {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport.append_result",kind:"method"})
// MERGE (transport_profile:Symbol {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport.profile",kind:"method"})
// MERGE (endpoint_resolve:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.resolve",kind:"method"})
// MERGE (endpoint_resolver_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.new",kind:"method"})
// MERGE (endpoint_config_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointConfig.new",kind:"method"})
// MERGE (profile_revision_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision.new",kind:"method"})
// MERGE (profile_definition_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileDefinition.new",kind:"method"})
// MERGE (admission_profile_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile.new",kind:"method"})
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
// MERGE (test_tenant_route:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::tests::route_rejects_transport_profile_from_another_tenant_before_claim_or_append",kind:"test"})
// MERGE (test_endpoint:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::tests::endpoint",kind:"function"})
// MERGE (test_profile_for_endpoint:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::tests::profile_for_endpoint",kind:"function"})
// MERGE (test_route:Symbol {id:"crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::tests::route",kind:"function"})
// MERGE (self)-[:DEFINES]->(publisher)
// MERGE (self)-[:DEFINES]->(outcome)
// MERGE (self)-[:DEFINES]->(error)
// MERGE (self)-[:DEFINES]->(count_retry)
// MERGE (self)-[:DEFINES]->(cancel_requested)
// MERGE (self)-[:DEFINES]->(valid_receipt)
// MERGE (self)-[:DEFINES]->(wait_cancel)
// MERGE (self)-[:DEFINES]->(test_profile)
// MERGE (self)-[:DEFINES]->(test_log)
// MERGE (self)-[:DEFINES]->(test_tenant_route)
// MERGE (self)-[:DEFINES]->(test_endpoint)
// MERGE (self)-[:DEFINES]->(test_profile_for_endpoint)
// MERGE (self)-[:DEFINES]->(test_route)
// MERGE (publisher)-[:DEFINES]->(new)
// MERGE (publisher)-[:DEFINES]->(publish)
// MERGE (publisher)-[:USES_TYPE]->(store)
// MERGE (publisher)-[:USES_TYPE]->(route)
// MERGE (publisher)-[:USES_TYPE]->(transport)
// MERGE (new)-[:USES_TYPE]->(route)
// MERGE (new)-[:USES_TYPE]->(transport)
// MERGE (new)-[:USES_TYPE]->(endpoint)
// MERGE (new)-[:CALLS]->(route_new)
// MERGE (self)-[:DEPENDS_ON]->(native_append_file)
// MERGE (self)-[:DEPENDS_ON]->(native_grpc_file)
// MERGE (self)-[:IMPORTS]->(admission_delivery_file)
// MERGE (self)-[:IMPORTS]->(transport_endpoint_file)
// MERGE (self)-[:IMPORTS]->(transport_profile_file)
// MERGE (native_append_file)-[:DEFINES]->(append_result)
// MERGE (native_append_file)-[:DEFINES]->(transport_profile)
// MERGE (publish)-[:CALLS]->(claim_native)
// MERGE (publish)-[:CALLS]->(route_recipient)
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
// MERGE (test_tenant_route)-[:TESTS]->(route_tenant)
// MERGE (test_tenant_route)-[:TESTS]->(route_deployment)
// MERGE (test_tenant_route)-[:TESTS]->(route_profile)
// MERGE (test_tenant_route)-[:CALLS]->(test_endpoint)
// MERGE (test_tenant_route)-[:CALLS]->(test_profile_for_endpoint)
// MERGE (test_tenant_route)-[:CALLS]->(route_test_profile_binding)
// MERGE (test_route)-[:CALLS]->(test_endpoint)
// MERGE (test_route)-[:CALLS]->(test_profile_for_endpoint)
// MERGE (test_route)-[:CALLS]->(route_test_profile_binding)
// MERGE (test_profile)-[:CALLS]->(test_route)
// MERGE (test_log)-[:CALLS]->(test_route)
// MERGE (test_endpoint)-[:CALLS]->(profile_definition_new)
// MERGE (test_endpoint)-[:CALLS]->(profile_revision_new)
// MERGE (test_endpoint)-[:CALLS]->(endpoint_config_new)
// MERGE (test_endpoint)-[:CALLS]->(endpoint_resolver_new)
// MERGE (test_endpoint)-[:CALLS]->(endpoint_resolve)
// MERGE (test_profile_for_endpoint)-[:CALLS]->(admission_profile_new)
// MERGE (valid_receipt)-[:CALLS]->(route_profile)
// @endcypher
//! Explicitly driven, bounded Infrastructure publisher for exact native capacity-result appends.

use std::time::Duration;

use platform_messaging_runtime::{
    native_append::{NativeAppendError, NativeAppendReceipt, NativeAppendTransport},
    native_grpc::NativeGrpcAppendTransport,
    transport_endpoint::ResolvedTransportEndpoint,
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
    /// The Infrastructure owner store rejected a lease, route pin, receipt, or transaction.
    #[error(transparent)]
    DeliveryStore(#[from] InfrastructureOwnerOutboxDeliveryError),
}

/// Owner-local native publisher with no background task, tenant discovery, or NATS fallback.
pub struct InfrastructureOwnerNativeResultPublisher {
    delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
    transport: NativeGrpcAppendTransport,
    route: NativeInfrastructureDeliveryRoute,
}

impl InfrastructureOwnerNativeResultPublisher {
    /// Construct from an already authenticated client and its trusted, immutable route snapshot.
    pub fn new(
        delivery_store: PostgresInfrastructureOwnerOutboxDelivery,
        transport: NativeGrpcAppendTransport,
        endpoint: &ResolvedTransportEndpoint,
        sender_service_id: Uuid,
        recipient_service_id: Uuid,
    ) -> Result<Self, NativeResultPublisherError> {
        let route = NativeInfrastructureDeliveryRoute::from_resolved_endpoint(
            endpoint,
            &transport,
            sender_service_id,
            recipient_service_id,
        )?;
        Ok(Self {
            delivery_store,
            transport,
            route,
        })
    }

    /// Claim, pin, append, and persist receipts for one bounded route-bound tenant batch.
    ///
    /// The exact route and bytes are persisted before any external append. A timeout or
    /// cancellation is ambiguous and therefore retries only the same event ID and bytes on the
    /// same profile. The database tenant scope is derived from the route's resolved endpoint;
    /// callers cannot supply a separate tenant. NATS is never selected as a fallback after a
    /// native route is pinned.
    pub async fn publish_due_for_tenant(
        &self,
        lease_owner_id: Uuid,
        max_items: i64,
        lease_seconds: i64,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<NativeResultPublishBatchOutcome, NativeResultPublisherError> {
        if lease_owner_id.is_nil()
            || !(1..=MAX_NATIVE_PUBLISH_BATCH_SIZE).contains(&max_items)
            || !(MIN_NATIVE_PUBLISH_LEASE_SECONDS..=MAX_NATIVE_PUBLISH_LEASE_SECONDS)
                .contains(&lease_seconds)
        {
            return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters.into());
        }
        let claims = self
            .delivery_store
            .claim_native_due(lease_owner_id, max_items, lease_seconds, &self.route)
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
                        self.route.recipient_service_id(),
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
    receipt.profile == *route.profile()
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
    use platform_messaging_runtime::{
        admission_delivery::AdmissionTransportProfile,
        transport_endpoint::{TrustedEndpointConfig, TrustedEndpointResolver},
        transport_profile::{
            TransportMessageKind, TransportProfileDefinition, TransportProfileLifecycle,
            TransportProfileRevision,
        },
    };

    fn endpoint(
        tenant_id: Uuid,
        profile_id: Uuid,
    ) -> platform_messaging_runtime::transport_endpoint::ResolvedTransportEndpoint {
        let deployment_id = Uuid::from_u128(100);
        let endpoint_identity = "native-endpoint";
        let definition = TransportProfileDefinition::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            endpoint_identity.to_owned(),
            "provider-log-generation-v1".to_owned(),
            vec![Uuid::from_u128(2)],
            vec![Uuid::from_u128(3)],
            vec![TransportMessageKind::Result],
            None,
        )
        .expect("valid profile definition");
        let revision = TransportProfileRevision::new(
            deployment_id,
            tenant_id,
            profile_id,
            3,
            TransportProfileLifecycle::Active,
            definition,
        )
        .expect("valid immutable profile revision");
        let endpoint_config = TrustedEndpointConfig::new(
            endpoint_identity.to_owned(),
            "https://transport.internal.example:7443/grpc/v1".to_owned(),
            "transport.internal.example".to_owned(),
        )
        .expect("valid trusted endpoint config");
        TrustedEndpointResolver::new(deployment_id, vec![endpoint_config])
            .expect("valid trusted endpoint resolver")
            .resolve(&revision)
            .expect("profile resolves to trusted endpoint")
    }

    fn profile_for_endpoint(
        endpoint: &platform_messaging_runtime::transport_endpoint::ResolvedTransportEndpoint,
    ) -> AdmissionTransportProfile {
        AdmissionTransportProfile::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            endpoint.profile_digest(),
            endpoint.endpoint_identity().to_owned(),
        )
        .expect("valid transport profile")
    }

    fn route() -> NativeInfrastructureDeliveryRoute {
        let endpoint = endpoint(Uuid::from_u128(41), Uuid::from_u128(1));
        NativeInfrastructureDeliveryRoute::from_resolved_profile_for_test(
            &endpoint,
            profile_for_endpoint(&endpoint),
            Uuid::from_u128(2),
            Uuid::from_u128(3),
        )
        .expect("route binds the trusted endpoint tenant and profile")
    }

    #[test]
    fn route_rejects_transport_profile_from_another_tenant_before_claim_or_append() {
        let tenant_a = Uuid::from_u128(41);
        let tenant_b = Uuid::from_u128(42);
        let endpoint_a = endpoint(tenant_a, Uuid::from_u128(1));
        let endpoint_b = endpoint(tenant_b, Uuid::from_u128(1));
        let profile_b = profile_for_endpoint(&endpoint_b);

        let route_b = NativeInfrastructureDeliveryRoute::from_resolved_profile_for_test(
            &endpoint_b,
            profile_b.clone(),
            Uuid::from_u128(2),
            Uuid::from_u128(3),
        )
        .expect("tenant B route is bound to the resolver tenant");
        assert_eq!(route_b.tenant_id(), tenant_b);
        assert_eq!(route_b.deployment_id(), endpoint_b.deployment_id());

        assert!(matches!(
            NativeInfrastructureDeliveryRoute::from_resolved_profile_for_test(
                &endpoint_a,
                profile_b,
                Uuid::from_u128(2),
                Uuid::from_u128(3),
            ),
            Err(InfrastructureOwnerOutboxDeliveryError::RouteConflict)
        ));
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
            &receipt(route.profile().clone(), digest),
            &route,
            &digest,
        ));

        let mut mismatched = receipt(route.profile().clone(), digest);
        mismatched.wire_sha256 = [8; 32];
        assert!(!valid_native_receipt(&mismatched, &route, &digest));

        let mut malformed = receipt(route.profile().clone(), digest);
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

        let mut malformed = receipt(route.profile().clone(), digest);
        malformed.log_id = "../foreign-log".to_owned();
        assert!(!valid_native_receipt(&malformed, &route, &digest));
    }
}
