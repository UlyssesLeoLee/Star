//! Opt-in native receiver for Run-owner Infrastructure capacity results.
/*
@cypher schema=1 source_sha256=df3cbdeebff51fb139104179acf179feea7aecfa1910897aa819f42b00404f22
MERGE (self:File {path:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::module",kind:"module"})
MERGE (consumer:Type {id:"run_owner_runtime::NativeGrpcCapacityResultConsumer",kind:"struct"})
MERGE (consumer_error:Type {id:"run_owner_runtime::NativeCapacityResultConsumerError",kind:"enum"})
MERGE (persist_outcome:Type {id:"run_owner_runtime::CapacityResultPersistOutcome",kind:"enum"})
MERGE (fixture_delivery:Type {id:"run_owner_runtime::native_capacity_result_tests::FixtureDelivery",kind:"struct"})
MERGE (fixture_authorization:Type {id:"run_owner_runtime::native_capacity_result_tests::FixtureAuthorization",kind:"struct"})
MERGE (fixture_quarantine:Type {id:"run_owner_runtime::native_capacity_result_tests::FixtureQuarantine",kind:"struct"})
MERGE (connect:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::NativeGrpcCapacityResultConsumer.connect",kind:"method"})
MERGE (with_quarantine:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::NativeGrpcCapacityResultConsumer.with_quarantine",kind:"method"})
MERGE (consume:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::NativeGrpcCapacityResultConsumer.consume_one",kind:"method"})
MERGE (process:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::NativeGrpcCapacityResultConsumer.process_one",kind:"method"})
MERGE (quarantine_delivery:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::NativeGrpcCapacityResultConsumer.quarantine_delivery",kind:"method"})
MERGE (supports_scope:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::supports_capacity_result_scope",kind:"function"})
MERGE (valid_origin:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::validate_capacity_result_origin",kind:"function"})
MERGE (origin_matches:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::capacity_result_matches_origin",kind:"function"})
MERGE (classify_poison:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::capacity_result_poison_fault",kind:"function"})
MERGE (persist_or_quarantine:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::persist_or_quarantine_capacity_result",kind:"function"})
MERGE (commit_ack:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::persist_authorized_then_ack",kind:"function"})
MERGE (quarantine_ack:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::quarantine_then_ack",kind:"function"})
MERGE (test_scope_helper:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.scope",kind:"function"})
MERGE (test_delivery_helper:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.make_delivery",kind:"function"})
MERGE (test_wire_helper:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.valid_wire",kind:"function"})
MERGE (test_scope:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.scope_is_result_only_and_validated_before_connect",kind:"test"})
MERGE (test_origin:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.envelope_tenant_and_event_must_match_authenticated_origin",kind:"test"})
MERGE (test_bounds:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.provider_neutral_parser_enforces_wire_bounds",kind:"test"})
MERGE (test_commit:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.authorization_commit_reauthorization_and_ack_order_is_enforced",kind:"test"})
MERGE (test_quarantine:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.quarantine_failure_never_acknowledges",kind:"test"})
MERGE (test_conflict:Symbol {id:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs::tests.idempotency_conflict_is_quarantined_before_terminal_disposition",kind:"test"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(consumer)
MERGE (module)-[:DEFINES]->(consumer_error)
MERGE (module)-[:DEFINES]->(persist_outcome)
MERGE (module)-[:DEFINES]->(fixture_delivery)
MERGE (module)-[:DEFINES]->(fixture_authorization)
MERGE (module)-[:DEFINES]->(fixture_quarantine)
MERGE (consumer)-[:DEFINES]->(connect)
MERGE (consumer)-[:DEFINES]->(with_quarantine)
MERGE (consumer)-[:DEFINES]->(consume)
MERGE (consumer)-[:DEFINES]->(process)
MERGE (consumer)-[:DEFINES]->(quarantine_delivery)
MERGE (module)-[:DEFINES]->(supports_scope)
MERGE (module)-[:DEFINES]->(valid_origin)
MERGE (module)-[:DEFINES]->(origin_matches)
MERGE (module)-[:DEFINES]->(classify_poison)
MERGE (module)-[:DEFINES]->(persist_or_quarantine)
MERGE (module)-[:DEFINES]->(commit_ack)
MERGE (module)-[:DEFINES]->(quarantine_ack)
MERGE (module)-[:DEFINES]->(test_scope_helper)
MERGE (module)-[:DEFINES]->(test_delivery_helper)
MERGE (module)-[:DEFINES]->(test_wire_helper)
MERGE (module)-[:DEFINES]->(test_scope)
MERGE (module)-[:DEFINES]->(test_origin)
MERGE (module)-[:DEFINES]->(test_bounds)
MERGE (module)-[:DEFINES]->(test_commit)
MERGE (module)-[:DEFINES]->(test_quarantine)
MERGE (module)-[:DEFINES]->(test_conflict)
MERGE (consumer_error)-[:USES_TYPE]->(setup_error:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcSetupError",kind:"enum"})
MERGE (consumer_error)-[:USES_TYPE]->(receive_error:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveError",kind:"enum"})
MERGE (consumer_error)-[:USES_TYPE]->(capacity_error:Type {id:"run_owner_runtime::CapacityResultError",kind:"enum"})
MERGE (consumer)-[:USES_TYPE]->(transport:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAdmissionReceiveTransport",kind:"struct"})
MERGE (consumer)-[:USES_TYPE]->(authorization:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAdmissionReceiveAuthorization",kind:"struct"})
MERGE (consumer)-[:USES_TYPE]->(store:Type {id:"run_owner_runtime::PostgresRunOwnerCapacityStore",kind:"struct"})
MERGE (consumer)-[:USES_TYPE]->(quarantine:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveQuarantine",kind:"trait"})
MERGE (consumer)-[:USES_TYPE]->(outcome:Type {id:"run_owner_runtime::CapacityResultConsumeOutcome",kind:"enum"})
MERGE (connect)-[:CALLS]->(supports_scope)
MERGE (connect)-[:CALLS]->(transport_connect:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport.connect",kind:"method"})
MERGE (connect)-[:CALLS]->(authorization_factory:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport.authorization",kind:"method"})
MERGE (consume)-[:CALLS]->(process)
MERGE (process)-[:CALLS]->(pull_one:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport.pull_one",kind:"method"})
MERGE (process)-[:CALLS]->(valid_origin)
MERGE (process)-[:CALLS]->(parse:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::parse_capacity_result_envelope",kind:"function"})
MERGE (process)-[:CALLS]->(origin_matches)
MERGE (process)-[:CALLS]->(classify_poison)
MERGE (process)-[:CALLS]->(persist_or_quarantine)
MERGE (process)-[:USES_TYPE]->(persist_outcome)
MERGE (process)-[:CALLS]->(quarantine_delivery)
MERGE (quarantine_delivery)-[:CALLS]->(quarantine_ack)
MERGE (persist_or_quarantine)-[:CALLS]->(classify_poison)
MERGE (persist_or_quarantine)-[:CALLS]->(commit_ack)
MERGE (persist_or_quarantine)-[:CALLS]->(quarantine_ack)
MERGE (commit_ack)-[:CALLS]->(verify:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization.verify",kind:"method"})
MERGE (commit_ack)-[:CALLS]->(record:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::PostgresRunOwnerCapacityStore.record_capacity_result",kind:"method"})
MERGE (commit_ack)-[:CALLS]->(ack:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery.acknowledge",kind:"method"})
MERGE (quarantine_ack)-[:CALLS]->(verify_source:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization.verify_source",kind:"method"})
MERGE (quarantine_ack)-[:CALLS]->(record_quarantine:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveQuarantine.record",kind:"method"})
MERGE (quarantine_ack)-[:CALLS]->(terminate:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery.terminate_permanently",kind:"method"})
MERGE (module)-[:IMPORTS]->(capacity_file:File {path:"crates/run-owner-runtime/src/capacity_result.rs"})
MERGE (module)-[:IMPORTS]->(native_file:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
MERGE (module)-[:IMPORTS]->(receive_file:File {path:"crates/platform-messaging-runtime/src/admission_receive.rs"})
MERGE (test_scope)-[:TESTS]->(supports_scope)
MERGE (test_scope)-[:CALLS]->(test_scope_helper)
MERGE (test_origin)-[:TESTS]->(origin_matches)
MERGE (test_origin)-[:CALLS]->(test_scope_helper)
MERGE (test_origin)-[:CALLS]->(test_delivery_helper)
MERGE (test_origin)-[:CALLS]->(test_wire_helper)
MERGE (test_bounds)-[:CALLS]->(parse:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::parse_capacity_result_envelope",kind:"function"})
MERGE (test_bounds)-[:CALLS]->(test_wire_helper)
MERGE (test_commit)-[:CALLS]->(commit_ack)
MERGE (test_commit)-[:CALLS]->(test_delivery_helper)
MERGE (test_commit)-[:CALLS]->(test_wire_helper)
MERGE (test_quarantine)-[:CALLS]->(quarantine_ack)
MERGE (test_quarantine)-[:CALLS]->(test_delivery_helper)
MERGE (test_quarantine)-[:CALLS]->(test_wire_helper)
MERGE (test_conflict)-[:CALLS]->(persist_or_quarantine)
MERGE (test_conflict)-[:CALLS]->(test_delivery_helper)
MERGE (test_conflict)-[:CALLS]->(test_wire_helper)
@endcypher
*/

use std::{future::Future, sync::Arc, time::Duration};

use platform_messaging_runtime::{
    admission_receive::{
        AdmissionReceiveAuthorization, AdmissionReceiveError, AdmissionReceiveKind,
        AdmissionReceiveOrigin, AdmissionReceivePoisonFault, AdmissionReceiveQuarantine,
        AdmissionReceiveScope, AdmissionReceiveTransport, AdmissionReceivedDelivery,
    },
    native_grpc::{
        NativeGrpcAdmissionReceiveAuthorization, NativeGrpcAdmissionReceiveTransport,
        NativeGrpcReceiveClientConfig, NativeGrpcSetupError,
    },
};
use thiserror::Error;
use tokio::sync::watch;

use crate::{
    capacity_result::{
        parse_capacity_result_envelope, CapacityResultConsumeOutcome, CapacityResultError,
        ValidatedCapacityResult,
    },
    PostgresRunOwnerCapacityStore,
};

const NATIVE_CAPACITY_RESULT_DEADLINE: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CapacityResultPersistOutcome {
    Stored(bool),
    Quarantined,
}

/// Setup and receive failures for the opt-in native capacity result consumer.
#[derive(Debug, Error)]
pub enum NativeCapacityResultConsumerError {
    /// Native gRPC route establishment failed without exposing TLS material.
    #[error(transparent)]
    NativeSetup(#[from] NativeGrpcSetupError),
    /// Native source verification, transport, quarantine, or acknowledgement failed.
    #[error(transparent)]
    Receive(#[from] AdmissionReceiveError),
    /// Strict envelope validation or Run-owner persistence failed.
    #[error(transparent)]
    Capacity(#[from] CapacityResultError),
    /// This receiver accepts only a valid Infrastructure result route.
    #[error("native Run-owner receiver requires a valid Result route")]
    UnsupportedScope,
}

/// Explicitly driven one-delivery receiver for the Run-owner capacity-result Inbox.
///
/// No task is spawned and no Schedule/Run execution is started. The selected tenant, sender,
/// recipient, Result kind and immutable transport profile are fixed at construction. The native
/// transport authenticates each source log generation and cursor before the owner commit and
/// again before acknowledgement.
pub struct NativeGrpcCapacityResultConsumer {
    transport: NativeGrpcAdmissionReceiveTransport,
    authorization: NativeGrpcAdmissionReceiveAuthorization,
    store: PostgresRunOwnerCapacityStore,
    scope: AdmissionReceiveScope,
    quarantine: Option<Arc<dyn AdmissionReceiveQuarantine>>,
}

impl NativeGrpcCapacityResultConsumer {
    /// Connect one native result route; invalid kinds and identities fail before network I/O.
    pub async fn connect(
        config: NativeGrpcReceiveClientConfig<'_>,
        store: PostgresRunOwnerCapacityStore,
    ) -> Result<Self, NativeCapacityResultConsumerError> {
        let scope = config.scope;
        if !supports_capacity_result_scope(scope) {
            return Err(NativeCapacityResultConsumerError::UnsupportedScope);
        }
        let transport = NativeGrpcAdmissionReceiveTransport::connect(config).await?;
        let authorization = transport.authorization();
        Ok(Self {
            transport,
            authorization,
            store,
            scope,
            quarantine: None,
        })
    }

    /// Attach an explicitly configured durable poison receipt provider.
    ///
    /// Without it malformed or identity-mismatched messages remain unacknowledged.
    pub fn with_quarantine(mut self, quarantine: Arc<dyn AdmissionReceiveQuarantine>) -> Self {
        self.quarantine = Some(quarantine);
        self
    }

    /// Consume at most one delivery with a ten-second deadline and cancellation support.
    pub async fn consume_one(
        &mut self,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<CapacityResultConsumeOutcome, NativeCapacityResultConsumerError> {
        if *cancellation.borrow_and_update() || cancellation.has_changed().is_err() {
            return Ok(CapacityResultConsumeOutcome::Cancelled);
        }
        let cancellation_wait = async {
            loop {
                if cancellation.changed().await.is_err() || *cancellation.borrow_and_update() {
                    break;
                }
            }
        };
        tokio::select! {
            biased;
            _ = cancellation_wait => Ok(CapacityResultConsumeOutcome::Cancelled),
            result = tokio::time::timeout(NATIVE_CAPACITY_RESULT_DEADLINE, self.process_one()) => {
                result.map_err(|_| AdmissionReceiveError::Deadline)?
            }
        }
    }

    async fn process_one(
        &self,
    ) -> Result<CapacityResultConsumeOutcome, NativeCapacityResultConsumerError> {
        let Some(delivery) = self.transport.pull_one(self.scope).await? else {
            return Ok(CapacityResultConsumeOutcome::Idle);
        };
        validate_capacity_result_origin(self.scope, delivery.origin())?;

        let result = match parse_capacity_result_envelope(delivery.wire()) {
            Ok(result) => result,
            Err(error) => {
                let Some(fault) = capacity_result_poison_fault(&error) else {
                    return Err(error.into());
                };
                return self.quarantine_delivery(&delivery, fault).await.map(|()| {
                    CapacityResultConsumeOutcome::QuarantinedNative {
                        log_generation: delivery.origin().log_generation,
                        sequence: delivery.origin().sequence,
                    }
                });
            }
        };

        if !capacity_result_matches_origin(&result, self.scope, delivery.origin()) {
            return self
                .quarantine_delivery(&delivery, AdmissionReceivePoisonFault::IdentityMismatch)
                .await
                .map(|()| CapacityResultConsumeOutcome::QuarantinedNative {
                    log_generation: delivery.origin().log_generation,
                    sequence: delivery.origin().sequence,
                });
        }

        let event_id = result.event_id;
        let persist_outcome = persist_or_quarantine_capacity_result(
            &delivery,
            &self.authorization,
            self.quarantine.as_deref(),
            || self.store.record_capacity_result(result),
        )
        .await?;
        match persist_outcome {
            CapacityResultPersistOutcome::Stored(true) => {
                Ok(CapacityResultConsumeOutcome::Stored { event_id })
            }
            CapacityResultPersistOutcome::Stored(false) => {
                Ok(CapacityResultConsumeOutcome::Duplicate { event_id })
            }
            CapacityResultPersistOutcome::Quarantined => {
                Ok(CapacityResultConsumeOutcome::QuarantinedNative {
                    log_generation: delivery.origin().log_generation,
                    sequence: delivery.origin().sequence,
                })
            }
        }
    }

    async fn quarantine_delivery<D>(
        &self,
        delivery: &D,
        fault: AdmissionReceivePoisonFault,
    ) -> Result<(), NativeCapacityResultConsumerError>
    where
        D: AdmissionReceivedDelivery,
    {
        let Some(quarantine) = &self.quarantine else {
            return Err(AdmissionReceiveError::Contract.into());
        };
        quarantine_then_ack(delivery, &self.authorization, quarantine.as_ref(), fault).await?;
        Ok(())
    }
}

fn supports_capacity_result_scope(scope: AdmissionReceiveScope) -> bool {
    scope.kind == AdmissionReceiveKind::Result
        && !scope.tenant_id.is_nil()
        && !scope.sender_service_id.is_nil()
        && !scope.recipient_service_id.is_nil()
        && scope.sender_service_id != scope.recipient_service_id
}

fn validate_capacity_result_origin(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
) -> Result<(), AdmissionReceiveError> {
    if origin.scope != scope
        || origin.message_id.is_nil()
        || origin.provider_identity.is_empty()
        || origin.provider_identity.len() > 128
        || origin.log_id.is_empty()
        || origin.log_id.len() > 128
        || origin.log_generation.is_nil()
        || origin.sequence <= 0
    {
        return Err(AdmissionReceiveError::Contract);
    }
    Ok(())
}

fn capacity_result_matches_origin(
    result: &ValidatedCapacityResult,
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
) -> bool {
    result.tenant_id == scope.tenant_id && result.event_id == origin.message_id
}

fn capacity_result_poison_fault(
    error: &CapacityResultError,
) -> Option<AdmissionReceivePoisonFault> {
    match error {
        CapacityResultError::InvalidEvent | CapacityResultError::Json(_) => {
            Some(AdmissionReceivePoisonFault::MalformedWire)
        }
        // A payload conflict for an existing event ID is permanent and must be quarantined.
        CapacityResultError::IdempotencyConflict => {
            Some(AdmissionReceivePoisonFault::IdentityMismatch)
        }
        // Owner/database consistency errors remain unacknowledged for repair or redelivery.
        _ => None,
    }
}

async fn persist_or_quarantine_capacity_result<D, A, Q, F, Fut>(
    delivery: &D,
    authorization: &A,
    quarantine: Option<&Q>,
    persist: F,
) -> Result<CapacityResultPersistOutcome, NativeCapacityResultConsumerError>
where
    D: AdmissionReceivedDelivery,
    A: AdmissionReceiveAuthorization,
    Q: AdmissionReceiveQuarantine + ?Sized,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<bool, CapacityResultError>>,
{
    match persist_authorized_then_ack(delivery, authorization, persist).await {
        Ok(stored) => Ok(CapacityResultPersistOutcome::Stored(stored)),
        Err(NativeCapacityResultConsumerError::Capacity(
            error @ CapacityResultError::IdempotencyConflict,
        )) => {
            let Some(fault) = capacity_result_poison_fault(&error) else {
                return Err(NativeCapacityResultConsumerError::Capacity(error));
            };
            let Some(quarantine) = quarantine else {
                return Err(AdmissionReceiveError::Contract.into());
            };
            quarantine_then_ack(delivery, authorization, quarantine, fault).await?;
            Ok(CapacityResultPersistOutcome::Quarantined)
        }
        Err(error) => Err(error),
    }
}

async fn persist_authorized_then_ack<D, A, F, Fut>(
    delivery: &D,
    authorization: &A,
    persist: F,
) -> Result<bool, NativeCapacityResultConsumerError>
where
    D: AdmissionReceivedDelivery,
    A: AdmissionReceiveAuthorization,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<bool, CapacityResultError>>,
{
    authorization
        .verify(delivery.origin(), delivery.wire())
        .await?;
    let stored = persist().await?;
    authorization
        .verify(delivery.origin(), delivery.wire())
        .await?;
    delivery.acknowledge().await?;
    Ok(stored)
}

async fn quarantine_then_ack<D, A, Q>(
    delivery: &D,
    authorization: &A,
    quarantine: &Q,
    fault: AdmissionReceivePoisonFault,
) -> Result<(), AdmissionReceiveError>
where
    D: AdmissionReceivedDelivery,
    A: AdmissionReceiveAuthorization,
    Q: AdmissionReceiveQuarantine + ?Sized,
{
    authorization
        .verify_source(delivery.origin(), delivery.wire())
        .await?;
    quarantine
        .record(delivery.origin(), delivery.wire(), fault)
        .await?;
    authorization
        .verify_source(delivery.origin(), delivery.wire())
        .await?;
    delivery.terminate_permanently().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
    };

    use platform_messaging_runtime::admission_receive::{
        AdmissionReceiveAuthorization, AdmissionReceiveError, AdmissionReceiveKind,
        AdmissionReceiveOrigin, AdmissionReceivePoisonFault, AdmissionReceiveQuarantine,
        AdmissionReceiveScope, AdmissionReceivedDelivery,
    };
    use serde_json::json;
    use uuid::Uuid;

    use crate::capacity_result::{
        parse_capacity_result_envelope, CapacityResultError, MAX_ENVELOPE_BYTES,
    };

    use super::{
        capacity_result_matches_origin, capacity_result_poison_fault, persist_authorized_then_ack,
        persist_or_quarantine_capacity_result, quarantine_then_ack, supports_capacity_result_scope,
        CapacityResultPersistOutcome, NativeCapacityResultConsumerError,
    };

    #[derive(Clone)]
    struct FixtureDelivery {
        origin: AdmissionReceiveOrigin,
        wire: Vec<u8>,
        events: Arc<Mutex<Vec<&'static str>>>,
        ack_fails: bool,
    }

    impl AdmissionReceivedDelivery for FixtureDelivery {
        fn wire(&self) -> &[u8] {
            &self.wire
        }

        fn origin(&self) -> &AdmissionReceiveOrigin {
            &self.origin
        }

        async fn acknowledge(&self) -> Result<(), AdmissionReceiveError> {
            self.events.lock().unwrap().push("ack");
            if self.ack_fails {
                Err(AdmissionReceiveError::Transport)
            } else {
                Ok(())
            }
        }

        async fn terminate_permanently(&self) -> Result<(), AdmissionReceiveError> {
            self.events.lock().unwrap().push("terminate");
            Ok(())
        }
    }

    struct FixtureAuthorization {
        events: Arc<Mutex<Vec<&'static str>>>,
        fail_source: bool,
    }

    impl AdmissionReceiveAuthorization for FixtureAuthorization {
        async fn verify(
            &self,
            _: &AdmissionReceiveOrigin,
            _: &[u8],
        ) -> Result<(), AdmissionReceiveError> {
            self.events.lock().unwrap().push("verify");
            Ok(())
        }

        async fn verify_source(
            &self,
            _: &AdmissionReceiveOrigin,
            _: &[u8],
        ) -> Result<(), AdmissionReceiveError> {
            self.events.lock().unwrap().push("verify_source");
            if self.fail_source {
                Err(AdmissionReceiveError::Unauthorized)
            } else {
                Ok(())
            }
        }
    }

    struct FixtureQuarantine {
        events: Arc<Mutex<Vec<&'static str>>>,
        fail: bool,
    }

    impl AdmissionReceiveQuarantine for FixtureQuarantine {
        fn record<'a>(
            &'a self,
            _: &'a AdmissionReceiveOrigin,
            _: &'a [u8],
            _: AdmissionReceivePoisonFault,
        ) -> Pin<Box<dyn Future<Output = Result<(), AdmissionReceiveError>> + Send + 'a>> {
            Box::pin(async move {
                self.events.lock().unwrap().push("quarantine");
                if self.fail {
                    Err(AdmissionReceiveError::Quarantine)
                } else {
                    Ok(())
                }
            })
        }
    }

    fn scope(kind: AdmissionReceiveKind) -> AdmissionReceiveScope {
        AdmissionReceiveScope {
            tenant_id: Uuid::from_u128(1),
            sender_service_id: Uuid::from_u128(2),
            recipient_service_id: Uuid::from_u128(3),
            kind,
        }
    }

    fn make_delivery(wire: Vec<u8>, events: Arc<Mutex<Vec<&'static str>>>) -> FixtureDelivery {
        FixtureDelivery {
            origin: AdmissionReceiveOrigin {
                scope: scope(AdmissionReceiveKind::Result),
                message_id: Uuid::from_u128(2),
                provider_identity: "native-provider-v1".to_owned(),
                log_id: "run-owner-results".to_owned(),
                log_generation: Uuid::from_u128(4),
                sequence: 9,
            },
            wire,
            events,
            ack_fails: false,
        }
    }

    fn valid_wire() -> Vec<u8> {
        serde_json::to_vec(&json!({
            "envelope_version": 1,
            "tenant_id": Uuid::from_u128(1),
            "event_id": Uuid::from_u128(2),
            "event_type": "capacity_rejected",
            "schema_version": 2,
            "idempotency_key": format!("capacity_rejected:{}", Uuid::from_u128(5)),
            "correlation_id": Uuid::from_u128(6),
            "payload": {
                "schema_version": 2,
                "source_event_id": Uuid::from_u128(5),
                "occurrence_id": Uuid::from_u128(7),
                "task_execution_run_id": Uuid::from_u128(8),
                "binding_digest": "01".repeat(32),
                "reason_code": "host_capacity_insufficient"
            }
        }))
        .unwrap()
    }

    #[test]
    fn scope_is_result_only_and_validated_before_connect() {
        assert!(supports_capacity_result_scope(scope(
            AdmissionReceiveKind::Result
        )));
        assert!(!supports_capacity_result_scope(scope(
            AdmissionReceiveKind::Request
        )));
        assert!(!supports_capacity_result_scope(scope(
            AdmissionReceiveKind::Resolution
        )));
        let mut invalid = scope(AdmissionReceiveKind::Result);
        invalid.tenant_id = Uuid::nil();
        assert!(!supports_capacity_result_scope(invalid));
    }

    #[test]
    fn envelope_tenant_and_event_must_match_authenticated_origin() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let delivery = make_delivery(valid_wire(), events);
        let parsed = parse_capacity_result_envelope(delivery.wire()).unwrap();
        assert!(capacity_result_matches_origin(
            &parsed,
            scope(AdmissionReceiveKind::Result),
            delivery.origin()
        ));

        let mut wrong_scope = scope(AdmissionReceiveKind::Result);
        wrong_scope.tenant_id = Uuid::from_u128(99);
        assert!(!capacity_result_matches_origin(
            &parsed,
            wrong_scope,
            delivery.origin()
        ));
        let mut wrong_origin = delivery.origin().clone();
        wrong_origin.message_id = Uuid::from_u128(99);
        assert!(!capacity_result_matches_origin(
            &parsed,
            scope(AdmissionReceiveKind::Result),
            &wrong_origin
        ));
    }

    #[test]
    fn provider_neutral_parser_enforces_wire_bounds() {
        assert!(parse_capacity_result_envelope(&valid_wire()).is_ok());
        assert!(matches!(
            parse_capacity_result_envelope(&vec![b'x'; MAX_ENVELOPE_BYTES + 1]),
            Err(CapacityResultError::InvalidEvent)
        ));
    }

    #[tokio::test]
    async fn authorization_commit_reauthorization_and_ack_order_is_enforced() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let delivery = make_delivery(valid_wire(), Arc::clone(&events));
        let authorization = FixtureAuthorization {
            events: Arc::clone(&events),
            fail_source: false,
        };
        let commit_events = Arc::clone(&events);
        let stored = persist_authorized_then_ack(&delivery, &authorization, move || async move {
            commit_events.lock().unwrap().push("commit");
            Ok::<_, CapacityResultError>(true)
        })
        .await
        .unwrap();
        assert!(stored);
        assert_eq!(
            *events.lock().unwrap(),
            ["verify", "commit", "verify", "ack"]
        );

        let failed_events = Arc::new(Mutex::new(Vec::new()));
        let failed_delivery = make_delivery(valid_wire(), Arc::clone(&failed_events));
        let failed_authorization = FixtureAuthorization {
            events: Arc::clone(&failed_events),
            fail_source: false,
        };
        let result =
            persist_authorized_then_ack(&failed_delivery, &failed_authorization, || async {
                Err::<bool, _>(CapacityResultError::UnknownDemand)
            })
            .await;
        assert!(matches!(
            result,
            Err(NativeCapacityResultConsumerError::Capacity(
                CapacityResultError::UnknownDemand
            ))
        ));
        assert_eq!(*failed_events.lock().unwrap(), ["verify"]);
    }

    #[tokio::test]
    async fn idempotency_conflict_is_quarantined_before_terminal_disposition() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let delivery = make_delivery(valid_wire(), Arc::clone(&events));
        let authorization = FixtureAuthorization {
            events: Arc::clone(&events),
            fail_source: false,
        };
        let quarantine = FixtureQuarantine {
            events: Arc::clone(&events),
            fail: false,
        };
        let conflict_events = Arc::clone(&events);

        let outcome = persist_or_quarantine_capacity_result(
            &delivery,
            &authorization,
            Some(&quarantine),
            move || async move {
                conflict_events.lock().unwrap().push("idempotency_conflict");
                Err::<bool, _>(CapacityResultError::IdempotencyConflict)
            },
        )
        .await
        .unwrap();

        assert_eq!(outcome, CapacityResultPersistOutcome::Quarantined);
        assert_eq!(
            capacity_result_poison_fault(&CapacityResultError::IdempotencyConflict),
            Some(AdmissionReceivePoisonFault::IdentityMismatch)
        );
        assert_eq!(
            *events.lock().unwrap(),
            [
                "verify",
                "idempotency_conflict",
                "verify_source",
                "quarantine",
                "verify_source",
                "terminate"
            ]
        );

        let failed_events = Arc::new(Mutex::new(Vec::new()));
        let failed_delivery = make_delivery(valid_wire(), Arc::clone(&failed_events));
        let failed_authorization = FixtureAuthorization {
            events: Arc::clone(&failed_events),
            fail_source: false,
        };
        let failed_quarantine = FixtureQuarantine {
            events: Arc::clone(&failed_events),
            fail: true,
        };
        let conflict_events = Arc::clone(&failed_events);

        let result = persist_or_quarantine_capacity_result(
            &failed_delivery,
            &failed_authorization,
            Some(&failed_quarantine),
            move || async move {
                conflict_events.lock().unwrap().push("idempotency_conflict");
                Err::<bool, _>(CapacityResultError::IdempotencyConflict)
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(NativeCapacityResultConsumerError::Receive(
                AdmissionReceiveError::Quarantine
            ))
        ));
        assert_eq!(
            *failed_events.lock().unwrap(),
            [
                "verify",
                "idempotency_conflict",
                "verify_source",
                "quarantine"
            ]
        );
    }

    #[tokio::test]
    async fn quarantine_failure_never_acknowledges() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let delivery = make_delivery(valid_wire(), Arc::clone(&events));
        let authorization = FixtureAuthorization {
            events: Arc::clone(&events),
            fail_source: false,
        };
        let quarantine = FixtureQuarantine {
            events: Arc::clone(&events),
            fail: true,
        };
        let result = quarantine_then_ack(
            &delivery,
            &authorization,
            &quarantine,
            AdmissionReceivePoisonFault::MalformedWire,
        )
        .await;
        assert_eq!(result, Err(AdmissionReceiveError::Quarantine));
        assert_eq!(*events.lock().unwrap(), ["verify_source", "quarantine"]);

        let committed_events = Arc::new(Mutex::new(Vec::new()));
        let committed_delivery = make_delivery(valid_wire(), Arc::clone(&committed_events));
        let committed_authorization = FixtureAuthorization {
            events: Arc::clone(&committed_events),
            fail_source: false,
        };
        let committed_quarantine = FixtureQuarantine {
            events: Arc::clone(&committed_events),
            fail: false,
        };
        quarantine_then_ack(
            &committed_delivery,
            &committed_authorization,
            &committed_quarantine,
            AdmissionReceivePoisonFault::MalformedWire,
        )
        .await
        .unwrap();
        assert_eq!(
            *committed_events.lock().unwrap(),
            ["verify_source", "quarantine", "verify_source", "terminate"]
        );
    }
}
