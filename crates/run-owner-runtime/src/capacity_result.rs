//! Bounded JetStream consumer for Infrastructure-owned capacity result facts.
/*
@cypher schema=1 source_sha256=a614bf6ab0d341f04950020674b465c8d1929454592ce7a328dd73f7b2eef0e2
MERGE (self:File {path:"crates/run-owner-runtime/src/capacity_result.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::capacity_result",kind:"module"})
MERGE (consumer:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer",kind:"struct"})
MERGE (outcome:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::CapacityResultConsumeOutcome",kind:"enum"})
MERGE (error:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::CapacityResultError",kind:"enum"})
MERGE (envelope:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::InfrastructureResultEnvelope",kind:"struct"})
MERGE (accepted:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::AcceptedPayload",kind:"struct"})
MERGE (rejected:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::RejectedPayload",kind:"struct"})
MERGE (expired:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::ExpiredPayload",kind:"struct"})
MERGE (validated:Type {id:"crates/run-owner-runtime/src/capacity_result.rs::ValidatedCapacityResult",kind:"struct"})
MERGE (new_consumer:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.new",kind:"method"})
MERGE (consume:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.consume_one",kind:"method"})
MERGE (persist_ack_method:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.persist_then_ack",kind:"method"})
MERGE (record:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::PostgresRunOwnerCapacityStore.record_capacity_result",kind:"method"})
MERGE (set_scope:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::set_run_owner_scope",kind:"function"})
MERGE (persist_ack:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::persist_then_ack",kind:"function"})
MERGE (load:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::load_and_match_demand",kind:"function"})
MERGE (bind_scope:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::bind_result_scope_from_demand",kind:"function"})
MERGE (verify_duplicate:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::verify_duplicate_result",kind:"function"})
MERGE (parse:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::parse_capacity_result_message",kind:"function"})
MERGE (parse_envelope:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::parse_capacity_result_envelope",kind:"function"})
MERGE (common:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::common_payload",kind:"function"})
MERGE (validate_payload:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::validate_result_payload",kind:"function"})
MERGE (digest:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::result_digest",kind:"function"})
MERGE (decode_digest:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::decode_digest",kind:"function"})
MERGE (decode_hex:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::decode_hex",kind:"function"})
MERGE (reason:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::is_reason_code",kind:"function"})
MERGE (match_demand:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::validate_payload_matches_demand",kind:"function"})
MERGE (test_id:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.id",kind:"function"})
MERGE (test_demand:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.demand_fixture",kind:"function"})
MERGE (test_digest:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.binding_digest",kind:"function"})
MERGE (test_wire:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.result_wire",kind:"function"})
MERGE (test_rejected:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.rejected_payload",kind:"function"})
MERGE (test_accepted_payload:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.accepted_payload",kind:"function"})
MERGE (test_expired_payload:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.expired_payload",kind:"function"})
MERGE (test_envelope:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.valid_envelope",kind:"function"})
MERGE (test_payload:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.valid_payload",kind:"function"})
MERGE (test_headers:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.headers",kind:"function"})
MERGE (subject_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.envelope_subject_and_jetstream_message_id_are_strictly_bound",kind:"test"})
MERGE (unknown_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.unknown_fields_versions_and_unproduced_release_contract_fail_closed",kind:"test"})
MERGE (accepted_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.accepted_payload_is_parsed_and_bound_to_the_stored_run_snapshot",kind:"test"})
MERGE (expired_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.expired_payload_is_parsed_and_binding_revision_is_enforced",kind:"test"})
MERGE (scope_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.result_scope_is_hydrated_from_the_stored_demand_not_the_wire_envelope",kind:"test"})
MERGE (bounds_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.payload_and_envelope_byte_bounds_are_enforced",kind:"test"})
MERGE (ack_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.acknowledgement_is_after_commit_and_never_runs_after_store_failure",kind:"test"})
MERGE (replay_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.ack_failure_leaves_committed_event_safe_for_duplicate_redelivery",kind:"test"})
MERGE (quarantine:Type {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine",kind:"struct"})
MERGE (fault:Type {id:"crates/platform-messaging-runtime/src/lib.rs::PermanentPoisonFault",kind:"enum"})
MERGE (quarantine_method:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.with_quarantine",kind:"method"})
MERGE (quarantine_error:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.quarantine_or_return",kind:"method"})
MERGE (classify_fault:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::permanent_poison_fault",kind:"function"})
MERGE (quarantine_test:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::tests.run_owner_data_errors_never_enter_poison_quarantine",kind:"test"})
MERGE (platform_file:File {path:"crates/platform-messaging-runtime/src/lib.rs"})
MERGE (demand:Table {id:"multica.infrastructure_capacity_demand_outbox"})
MERGE (inbox:Table {id:"multica.infrastructure_capacity_result_inbox"})
MERGE (topic:Topic {id:"star.infrastructure.v2.capacity_*"})
MERGE (message:Type {id:"external::async_nats::jetstream::Message",kind:"external_type"})
MERGE (jetstream:Type {id:"external::async_nats::jetstream::consumer::PullConsumer",kind:"external_type"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(consumer)
MERGE (module)-[:DEFINES]->(outcome)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(envelope)
MERGE (module)-[:DEFINES]->(accepted)
MERGE (module)-[:DEFINES]->(rejected)
MERGE (module)-[:DEFINES]->(expired)
MERGE (module)-[:DEFINES]->(validated)
MERGE (module)-[:DEFINES]->(record)
MERGE (module)-[:DEFINES]->(set_scope)
MERGE (module)-[:DEFINES]->(persist_ack)
MERGE (module)-[:DEFINES]->(load)
MERGE (module)-[:DEFINES]->(bind_scope)
MERGE (module)-[:DEFINES]->(verify_duplicate)
MERGE (module)-[:DEFINES]->(parse)
MERGE (module)-[:DEFINES]->(parse_envelope)
MERGE (module)-[:DEFINES]->(common)
MERGE (module)-[:DEFINES]->(validate_payload)
MERGE (module)-[:DEFINES]->(digest)
MERGE (module)-[:DEFINES]->(decode_digest)
MERGE (module)-[:DEFINES]->(decode_hex)
MERGE (module)-[:DEFINES]->(reason)
MERGE (module)-[:DEFINES]->(match_demand)
MERGE (module)-[:DEFINES]->(test_id)
MERGE (module)-[:DEFINES]->(test_demand)
MERGE (module)-[:DEFINES]->(test_digest)
MERGE (module)-[:DEFINES]->(test_wire)
MERGE (module)-[:DEFINES]->(test_rejected)
MERGE (module)-[:DEFINES]->(test_accepted_payload)
MERGE (module)-[:DEFINES]->(test_expired_payload)
MERGE (module)-[:DEFINES]->(test_envelope)
MERGE (module)-[:DEFINES]->(test_payload)
MERGE (module)-[:DEFINES]->(test_headers)
MERGE (module)-[:DEFINES]->(subject_test)
MERGE (module)-[:DEFINES]->(unknown_test)
MERGE (module)-[:DEFINES]->(accepted_test)
MERGE (module)-[:DEFINES]->(expired_test)
MERGE (module)-[:DEFINES]->(scope_test)
MERGE (module)-[:DEFINES]->(bounds_test)
MERGE (module)-[:DEFINES]->(ack_test)
MERGE (module)-[:DEFINES]->(replay_test)
MERGE (module)-[:DEFINES]->(quarantine_method)
MERGE (module)-[:DEFINES]->(quarantine_error)
MERGE (module)-[:DEFINES]->(classify_fault)
MERGE (module)-[:DEFINES]->(quarantine_test)
MERGE (module)-[:IMPORTS]->(platform_file)
MERGE (consumer)-[:DEFINES]->(new_consumer)
MERGE (consumer)-[:DEFINES]->(consume)
MERGE (consumer)-[:DEFINES]->(persist_ack_method)
MERGE (consume)-[:CALLS]->(persist_ack_method)
MERGE (consume)-[:USES_TYPE]->(jetstream)
MERGE (persist_ack_method)-[:USES_TYPE]->(message)
MERGE (persist_ack_method)-[:CALLS]->(parse)
MERGE (persist_ack_method)-[:CALLS]->(record)
MERGE (persist_ack_method)-[:CALLS]->(quarantine_error)
MERGE (quarantine_error)-[:USES_TYPE]->(message)
MERGE (quarantine_method)-[:USES_TYPE]->(quarantine)
MERGE (quarantine_error)-[:CALLS]->(classify_fault)
MERGE (classify_fault)-[:USES_TYPE]->(fault)
MERGE (quarantine_error)-[:CALLS]->(record_poison:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.record_message",kind:"method"})
MERGE (quarantine_test)-[:TESTS]->(classify_fault)
MERGE (parse)-[:HANDLES]->(topic)
MERGE (record)-[:CALLS]->(load)
MERGE (record)-[:CALLS]->(verify_duplicate)
MERGE (record)-[:CALLS]->(bind_scope)
MERGE (record)-[:CALLS]->(set_scope)
MERGE (load)-[:CALLS]->(bind_scope)
MERGE (bind_scope)-[:CALLS]->(match_demand)
MERGE (parse)-[:CALLS]->(parse_envelope)
MERGE (parse_envelope)-[:CALLS]->(validate_payload)
MERGE (parse_envelope)-[:CALLS]->(common)
MERGE (parse_envelope)-[:CALLS]->(digest)
MERGE (validate_payload)-[:CALLS]->(common)
MERGE (validate_payload)-[:CALLS]->(decode_digest)
MERGE (validate_payload)-[:CALLS]->(reason)
MERGE (decode_digest)-[:CALLS]->(decode_hex)
MERGE (record)-[:READS]->(demand)
MERGE (record)-[:WRITES]->(inbox)
MERGE (subject_test)-[:CALLS]->(parse)
MERGE (unknown_test)-[:CALLS]->(parse)
MERGE (accepted_test)-[:CALLS]->(parse)
MERGE (accepted_test)-[:CALLS]->(match_demand)
MERGE (expired_test)-[:CALLS]->(parse)
MERGE (expired_test)-[:CALLS]->(match_demand)
MERGE (scope_test)-[:CALLS]->(parse)
MERGE (scope_test)-[:CALLS]->(bind_scope)
MERGE (bounds_test)-[:CALLS]->(parse)
MERGE (ack_test)-[:CALLS]->(persist_ack)
MERGE (replay_test)-[:CALLS]->(persist_ack)
@endcypher
*/

#[cfg(test)]
use std::future::Future;
use std::time::Duration;

use async_nats::jetstream::{
    consumer::{AckPolicy, PullConsumer},
    Message,
};
use chrono::{DateTime, Utc};
use domain_infrastructure::HostInfrastructureDemandV2;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;
use tokio::sync::watch;
use uuid::Uuid;

use crate::outbox::{
    set_run_owner_scope, PostgresRunOwnerCapacityStore, RunOwnerCapacityStoreError,
};

const SUBJECT_PREFIX: &str = "star.infrastructure.v2";
const MAX_PAYLOAD_BYTES: usize = 16 * 1024;
pub(super) const MAX_ENVELOPE_BYTES: usize = 128 * 1024;
// async-nats `max_bytes` limits client buffering of messages already received; it is not an
// ingress or wire-size limit. Production must cap NATS and the JetStream stream at the envelope
// limit and measure peak consumer RSS before enabling this consumer.
const MAX_FETCH_BYTES: usize = MAX_ENVELOPE_BYTES + 4 * 1024;
const MAX_FETCH_MESSAGES: usize = 1;
const MAX_FETCH_EXPIRY: Duration = Duration::from_secs(2);
const DEMAND_SCHEMA_VERSION: i16 = 2;
const CAPACITY_RESULT_PARSER_VERSION: i16 = 1;

/// Result of one bounded JetStream pull attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityResultConsumeOutcome {
    /// Cancellation was observed before or during the bounded pull request.
    Cancelled,
    /// The pull request expired without a message.
    Idle,
    /// A new immutable result fact committed and was acknowledged.
    Stored {
        /// Infrastructure-owner event identity.
        event_id: Uuid,
    },
    /// The same immutable result fact was already committed and was acknowledged.
    Duplicate {
        /// Infrastructure-owner event identity.
        event_id: Uuid,
    },
    /// A permanent malformed event receipt committed and the source delivery was acknowledged.
    Quarantined {
        /// Server-assigned source stream sequence.
        stream_sequence: i64,
        /// True when the same source event had already been quarantined.
        duplicate: bool,
    },
    #[cfg(feature = "native-grpc-adapter")]
    /// A poison receipt committed and a native durable-log delivery was terminally acknowledged.
    QuarantinedNative {
        /// Authenticated source log generation.
        log_generation: Uuid,
        /// Authenticated durable-log cursor.
        sequence: i64,
    },
}

/// Errors while consuming or validating Infrastructure capacity result events.
#[derive(Debug, Error)]
pub enum CapacityResultError {
    /// The supplied durable PullConsumer does not use explicit acknowledgements.
    #[error("capacity consumer must be durable and use explicit acknowledgement")]
    InvalidConsumer,
    /// An envelope, subject, header, payload, or scope violates its versioned contract.
    #[error("invalid Infrastructure capacity result event")]
    InvalidEvent,
    /// A result references no Run-owner demand Outbox row.
    #[error("capacity result references an unknown Run-owner demand")]
    UnknownDemand,
    /// A result event ID was reused with different immutable scope or contents.
    #[error("Infrastructure result event ID was reused with different content")]
    IdempotencyConflict,
    /// A demand Outbox row is malformed or inconsistent with its typed JSON payload.
    #[error("stored Run-owner capacity demand is malformed")]
    InvalidStoredDemand,
    /// The current Infrastructure publisher has no release event producer/schema.
    #[error("capacity_released has no producer contract in this phase")]
    ReleasedContractUnavailable,
    /// An owner store operation failed. The JetStream message remains unacknowledged.
    #[error(transparent)]
    Store(#[from] RunOwnerCapacityStoreError),
    /// A PostgreSQL operation failed. The JetStream message remains unacknowledged.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    /// JSON encoding or decoding failed. The JetStream message remains unacknowledged.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// JetStream pull or durable acknowledgement failed.
    #[error("JetStream capacity result operation failed")]
    JetStream(#[source] async_nats::Error),
    /// The bounded JetStream pull stream ended with an error.
    #[error("JetStream capacity result fetch failed")]
    Fetch(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// The platform quarantine owner could not commit or acknowledge a permanent poison event.
    #[error(transparent)]
    Quarantine(#[from] platform_messaging_runtime::QuarantineError),
}

/// A single-message, bounded consumer supplied with an existing durable PullConsumer.
///
/// It does not create streams/consumers, configure credentials, change Run execution state, or
/// drain a shared NATS client. It records owner-scoped result facts and only double-acks after
/// the PostgreSQL transaction has committed.
pub struct RunOwnerCapacityResultConsumer {
    store: PostgresRunOwnerCapacityStore,
    consumer: PullConsumer,
    quarantine: Option<platform_messaging_runtime::PlatformPoisonQuarantine>,
}

impl RunOwnerCapacityResultConsumer {
    /// Wrap an existing durable PullConsumer after checking its acknowledgement contract.
    pub fn new(
        store: PostgresRunOwnerCapacityStore,
        consumer: PullConsumer,
    ) -> Result<Self, CapacityResultError> {
        let config = &consumer.cached_info().config;
        if config.durable_name.as_deref().is_none_or(str::is_empty)
            || config.ack_policy != AckPolicy::Explicit
            || config.deliver_subject.is_some()
        {
            return Err(CapacityResultError::InvalidConsumer);
        }
        Ok(Self {
            store,
            consumer,
            quarantine: None,
        })
    }

    /// Attach an explicitly configured Platform Messaging quarantine owner.
    ///
    /// Without this owner, permanent malformed messages remain unacknowledged. This method does
    /// not select a body-retention policy; that policy was required when constructing the service.
    pub fn with_quarantine(
        mut self,
        quarantine: platform_messaging_runtime::PlatformPoisonQuarantine,
    ) -> Self {
        self.quarantine = Some(quarantine);
        self
    }

    /// Pull and persist at most one result message, stopping on cancellation without acking it.
    pub async fn consume_one(
        &mut self,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<CapacityResultConsumeOutcome, CapacityResultError> {
        loop {
            if *cancellation.borrow_and_update() {
                return Ok(CapacityResultConsumeOutcome::Cancelled);
            }

            let mut messages = tokio::select! {
                biased;
                changed = cancellation.changed() => {
                    if changed.is_err() || *cancellation.borrow_and_update() {
                        return Ok(CapacityResultConsumeOutcome::Cancelled);
                    }
                    continue;
                }
                result = self.consumer
                    .fetch()
                    .max_messages(MAX_FETCH_MESSAGES)
                    .max_bytes(MAX_FETCH_BYTES)
                    .expires(MAX_FETCH_EXPIRY)
                    .messages() => result.map_err(|error| {
                        CapacityResultError::Fetch(Box::new(error))
                    })?,
            };

            let message = tokio::select! {
                biased;
                changed = cancellation.changed() => {
                    if changed.is_err() || *cancellation.borrow_and_update() {
                        return Ok(CapacityResultConsumeOutcome::Cancelled);
                    }
                    continue;
                }
                item = messages.next() => item,
            };

            let Some(message) = message else {
                return Ok(CapacityResultConsumeOutcome::Idle);
            };
            let message = message.map_err(CapacityResultError::Fetch)?;
            return self.persist_then_ack(message).await;
        }
    }

    async fn persist_then_ack(
        &self,
        message: Message,
    ) -> Result<CapacityResultConsumeOutcome, CapacityResultError> {
        let result = match parse_capacity_result_message(
            message.subject.as_str(),
            message.headers.as_ref(),
            &message.payload,
        ) {
            Ok(result) => result,
            Err(error) => return self.quarantine_or_return(message, error).await,
        };
        let event_id = result.event_id;
        let stored = match self.store.record_capacity_result(result).await {
            Ok(stored) => stored,
            Err(error) => return self.quarantine_or_return(message, error).await,
        };
        message
            .double_ack()
            .await
            .map_err(CapacityResultError::JetStream)?;
        Ok(if stored {
            CapacityResultConsumeOutcome::Stored { event_id }
        } else {
            CapacityResultConsumeOutcome::Duplicate { event_id }
        })
    }

    async fn quarantine_or_return(
        &self,
        message: Message,
        error: CapacityResultError,
    ) -> Result<CapacityResultConsumeOutcome, CapacityResultError> {
        let Some(fault) = permanent_poison_fault(&error) else {
            return Err(error);
        };
        let Some(quarantine) = &self.quarantine else {
            return Err(error);
        };
        let outcome = quarantine
            .record_message(&message, fault, CAPACITY_RESULT_PARSER_VERSION)
            .await?;
        message
            .double_ack_with(async_nats::jetstream::AckKind::Term)
            .await
            .map_err(CapacityResultError::JetStream)?;
        Ok(match outcome {
            platform_messaging_runtime::QuarantineRecordOutcome::Stored {
                stream_sequence, ..
            } => CapacityResultConsumeOutcome::Quarantined {
                stream_sequence,
                duplicate: false,
            },
            platform_messaging_runtime::QuarantineRecordOutcome::Duplicate { stream_sequence } => {
                CapacityResultConsumeOutcome::Quarantined {
                    stream_sequence,
                    duplicate: true,
                }
            }
        })
    }
}

fn permanent_poison_fault(
    error: &CapacityResultError,
) -> Option<platform_messaging_runtime::PermanentPoisonFault> {
    use platform_messaging_runtime::PermanentPoisonFault;
    match error {
        CapacityResultError::InvalidEvent => Some(PermanentPoisonFault::InvalidCapacityResultEvent),
        CapacityResultError::Json(_) => Some(PermanentPoisonFault::MalformedCapacityResultJson),
        CapacityResultError::IdempotencyConflict => {
            Some(PermanentPoisonFault::CapacityResultEventIdConflict)
        }
        // InvalidStoredDemand is a Run-owner fact-integrity failure, not a message poison code.
        // Unknown demand may be delivery-order/consistency related; transient failures likewise
        // remain unacknowledged for owner repair or redelivery.
        _ => None,
    }
}

// This helper remains for unit coverage of strict commit-before-ACK ordering. The live consumer
// uses the same ordering directly and routes only permanent wire faults to the platform owner.
#[cfg(test)]
async fn persist_then_ack<T, P, A>(persist: P, ack: A) -> Result<T, CapacityResultError>
where
    P: Future<Output = Result<T, CapacityResultError>>,
    A: Future<Output = Result<(), CapacityResultError>>,
{
    let committed = persist.await?;
    ack.await?;
    Ok(committed)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InfrastructureResultEnvelope {
    envelope_version: u8,
    tenant_id: Uuid,
    event_id: Uuid,
    event_type: String,
    schema_version: i16,
    idempotency_key: String,
    correlation_id: Uuid,
    payload: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptedPayload {
    schema_version: u16,
    source_event_id: Uuid,
    occurrence_id: Uuid,
    task_execution_run_id: Uuid,
    binding_digest: String,
    reservation_bundle_id: Uuid,
    host_capacity_reservation_id: Uuid,
    cluster_capacity_reservation_id: Uuid,
    host_environment_id: Uuid,
    host_generation: i64,
    k3s_cluster_id: Uuid,
    k3s_cluster_generation: i64,
    lease_expires_at: DateTime<Utc>,
    capacity_snapshot_digest: String,
    capacity_observed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RejectedPayload {
    schema_version: u16,
    source_event_id: Uuid,
    occurrence_id: Uuid,
    task_execution_run_id: Uuid,
    binding_digest: String,
    reason_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpiredPayload {
    schema_version: u16,
    source_event_id: Uuid,
    occurrence_id: Uuid,
    task_execution_run_id: Uuid,
    binding_id: Uuid,
    binding_revision: i64,
    binding_digest: String,
    reservation_bundle_id: Uuid,
    host_capacity_reservation_id: Uuid,
    cluster_capacity_reservation_id: Uuid,
    host_environment_id: Uuid,
    host_generation: i64,
    k3s_cluster_id: Uuid,
    k3s_cluster_generation: i64,
    lease_expires_at: DateTime<Utc>,
    expired_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(super) struct ValidatedCapacityResult {
    pub(super) tenant_id: Uuid,
    pub(super) event_id: Uuid,
    source_event_id: Uuid,
    project_id: Uuid,
    branch_id: Uuid,
    engineering_run_id: Uuid,
    work_item_id: Uuid,
    pub(super) task_execution_run_id: Uuid,
    pub(super) occurrence_id: Uuid,
    event_type: String,
    schema_version: i16,
    idempotency_key: String,
    correlation_id: Uuid,
    payload_digest: [u8; 32],
    payload: Value,
}

impl PostgresRunOwnerCapacityStore {
    pub(super) async fn record_capacity_result(
        &self,
        mut result: ValidatedCapacityResult,
    ) -> Result<bool, CapacityResultError> {
        let mut tx = self.pool().begin().await?;
        set_run_owner_scope(&mut tx, result.tenant_id).await?;
        load_and_match_demand(&mut tx, &mut result).await?;

        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"INSERT INTO multica.infrastructure_capacity_result_inbox
                    (event_id, tenant_id, source_event_id, project_id, branch_id,
                     engineering_run_id, work_item_id, task_execution_run_id, occurrence_id,
                     event_type, schema_version, idempotency_key, correlation_id,
                     payload_digest, payload, received_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,now())
               ON CONFLICT (tenant_id, event_id) DO NOTHING
               RETURNING event_id"#,
        )
        .bind(result.event_id)
        .bind(result.tenant_id)
        .bind(result.source_event_id)
        .bind(result.project_id)
        .bind(result.branch_id)
        .bind(result.engineering_run_id)
        .bind(result.work_item_id)
        .bind(result.task_execution_run_id)
        .bind(result.occurrence_id)
        .bind(&result.event_type)
        .bind(result.schema_version)
        .bind(&result.idempotency_key)
        .bind(result.correlation_id)
        .bind(result.payload_digest.as_slice())
        .bind(&result.payload)
        .fetch_optional(&mut *tx)
        .await?;

        let inserted = if inserted.is_some() {
            true
        } else {
            // The unique key is the concurrency arbiter. In the normal READ COMMITTED owner
            // transaction, PostgreSQL waits for a competing insert to commit before this next
            // statement reads it. At stronger isolation, an invisible winner fails closed below.
            verify_duplicate_result(&mut tx, &result).await?;
            false
        };
        tx.commit().await?;
        Ok(inserted)
    }
}

async fn load_and_match_demand(
    tx: &mut Transaction<'_, Postgres>,
    result: &mut ValidatedCapacityResult,
) -> Result<(), CapacityResultError> {
    // Demand Outbox rows are WORM Transaction facts. Read them without a row lock because this
    // owner role intentionally has SELECT/INSERT only; no mutating projection is performed here.
    let row = sqlx::query(
        r#"SELECT project_id, branch_id, engineering_run_id, work_item_id,
                  task_execution_run_id, occurrence_id, schema_version,
                  idempotency_key, correlation_id, payload
           FROM multica.infrastructure_capacity_demand_outbox
           WHERE tenant_id = $1 AND event_id = $2
           "#,
    )
    .bind(result.tenant_id)
    .bind(result.source_event_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(CapacityResultError::UnknownDemand)?;

    let payload: Value = row
        .try_get("payload")
        .map_err(|_| CapacityResultError::InvalidStoredDemand)?;
    let demand: HostInfrastructureDemandV2 =
        serde_json::from_value(payload).map_err(|_| CapacityResultError::InvalidStoredDemand)?;
    let work_item_id: Uuid = row.try_get("work_item_id")?;
    if demand.validate().is_err()
        || row.try_get::<Uuid, _>("project_id")? != demand.project_id
        || row.try_get::<Uuid, _>("branch_id")? != demand.branch_id
        || row.try_get::<Uuid, _>("engineering_run_id")? != demand.engineering_run_id
        || work_item_id.is_nil()
        || row.try_get::<Uuid, _>("task_execution_run_id")? != demand.task_execution_run_id
        || row.try_get::<Uuid, _>("occurrence_id")? != demand.occurrence_id
        || row.try_get::<i16, _>("schema_version")? != DEMAND_SCHEMA_VERSION
        || row.try_get::<Uuid, _>("correlation_id")? != demand.correlation_id
        || demand.tenant_id != result.tenant_id
        || demand.event_id != result.source_event_id
        || demand.task_execution_run_id != result.task_execution_run_id
        || demand.occurrence_id != result.occurrence_id
        || demand.correlation_id != result.correlation_id
    {
        return Err(CapacityResultError::InvalidStoredDemand);
    }
    let expected_key = format!("capacity-demand:v2:{}", demand.occurrence_id);
    if row.try_get::<String, _>("idempotency_key")? != expected_key {
        return Err(CapacityResultError::InvalidStoredDemand);
    }
    bind_result_scope_from_demand(result, &demand, work_item_id)
}

fn bind_result_scope_from_demand(
    result: &mut ValidatedCapacityResult,
    demand: &HostInfrastructureDemandV2,
    work_item_id: Uuid,
) -> Result<(), CapacityResultError> {
    if work_item_id.is_nil()
        || demand.tenant_id != result.tenant_id
        || demand.event_id != result.source_event_id
        || demand.task_execution_run_id != result.task_execution_run_id
        || demand.occurrence_id != result.occurrence_id
        || demand.correlation_id != result.correlation_id
    {
        return Err(CapacityResultError::InvalidStoredDemand);
    }
    validate_payload_matches_demand(result, demand)?;

    // Project, Branch, and Engineering Run are trusted from the typed Outbox row; they are not
    // part of the Infrastructure result envelope and therefore start unset during parsing.
    result.project_id = demand.project_id;
    result.branch_id = demand.branch_id;
    result.engineering_run_id = demand.engineering_run_id;
    result.work_item_id = work_item_id;
    Ok(())
}

async fn verify_duplicate_result(
    tx: &mut Transaction<'_, Postgres>,
    result: &ValidatedCapacityResult,
) -> Result<(), CapacityResultError> {
    // Result Inbox rows are also WORM. Do not take FOR UPDATE: an exact replay is acknowledged
    // only after this read confirms the already-committed immutable payload and scope.
    let row = sqlx::query(
        r#"SELECT source_event_id, project_id, branch_id, engineering_run_id,
                  work_item_id, task_execution_run_id, occurrence_id, event_type,
                  schema_version, idempotency_key, correlation_id, payload_digest, payload
           FROM multica.infrastructure_capacity_result_inbox
           WHERE tenant_id = $1 AND event_id = $2
           "#,
    )
    .bind(result.tenant_id)
    .bind(result.event_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(CapacityResultError::IdempotencyConflict)?;
    let digest: Vec<u8> = row.try_get("payload_digest")?;
    let payload: Value = row.try_get("payload")?;
    let same = row.try_get::<Uuid, _>("source_event_id")? == result.source_event_id
        && row.try_get::<Uuid, _>("project_id")? == result.project_id
        && row.try_get::<Uuid, _>("branch_id")? == result.branch_id
        && row.try_get::<Uuid, _>("engineering_run_id")? == result.engineering_run_id
        && row.try_get::<Uuid, _>("work_item_id")? == result.work_item_id
        && row.try_get::<Uuid, _>("task_execution_run_id")? == result.task_execution_run_id
        && row.try_get::<Uuid, _>("occurrence_id")? == result.occurrence_id
        && row.try_get::<String, _>("event_type")? == result.event_type
        && row.try_get::<i16, _>("schema_version")? == result.schema_version
        && row.try_get::<String, _>("idempotency_key")? == result.idempotency_key
        && row.try_get::<Uuid, _>("correlation_id")? == result.correlation_id
        && digest.as_slice() == result.payload_digest.as_slice()
        && payload == result.payload;
    if same {
        Ok(())
    } else {
        Err(CapacityResultError::IdempotencyConflict)
    }
}

fn parse_capacity_result_message(
    subject: &str,
    headers: Option<&async_nats::HeaderMap>,
    wire: &[u8],
) -> Result<ValidatedCapacityResult, CapacityResultError> {
    // `wire` is already materialized in `Message::payload` here; this parser guard does not bound
    // the allocation made while receiving the message. Broker and stream limits must do that.
    let headers = headers.ok_or(CapacityResultError::InvalidEvent)?;
    let mut message_ids = headers.get_all("Nats-Msg-Id");
    let message_id = message_ids
        .next()
        .ok_or(CapacityResultError::InvalidEvent)?
        .as_str()
        .to_owned();
    if message_ids.next().is_some() {
        return Err(CapacityResultError::InvalidEvent);
    }

    let result = parse_capacity_result_envelope(wire)?;
    if message_id != result.event_id.to_string() {
        return Err(CapacityResultError::InvalidEvent);
    }
    let expected_subject = format!("{SUBJECT_PREFIX}.{}", result.event_type);
    if subject != expected_subject {
        return Err(CapacityResultError::InvalidEvent);
    }
    Ok(result)
}

/// Strict provider-neutral envelope and payload parser shared by every receive adapter.
pub(super) fn parse_capacity_result_envelope(
    wire: &[u8],
) -> Result<ValidatedCapacityResult, CapacityResultError> {
    if wire.is_empty() || wire.len() > MAX_ENVELOPE_BYTES {
        return Err(CapacityResultError::InvalidEvent);
    }
    let envelope: InfrastructureResultEnvelope = serde_json::from_slice(wire)?;
    if envelope.envelope_version != 1
        || envelope.schema_version != DEMAND_SCHEMA_VERSION
        || envelope.tenant_id.is_nil()
        || envelope.event_id.is_nil()
        || envelope.correlation_id.is_nil()
        || envelope.idempotency_key.is_empty()
        || envelope.idempotency_key.len() > 200
        || !envelope.payload.is_object()
        || serde_json::to_vec(&envelope.payload)?.len() > MAX_PAYLOAD_BYTES
    {
        return Err(CapacityResultError::InvalidEvent);
    }

    let source_event_id = validate_result_payload(&envelope.event_type, &envelope.payload)?;
    let expected_idempotency_key = format!("{}:{source_event_id}", envelope.event_type);
    if envelope.idempotency_key != expected_idempotency_key {
        return Err(CapacityResultError::InvalidEvent);
    }
    let payload_digest = result_digest(&envelope.payload)?;
    let common = common_payload(&envelope.event_type, &envelope.payload)?;
    Ok(ValidatedCapacityResult {
        tenant_id: envelope.tenant_id,
        event_id: envelope.event_id,
        source_event_id,
        project_id: Uuid::nil(),
        branch_id: Uuid::nil(),
        engineering_run_id: Uuid::nil(),
        work_item_id: Uuid::nil(),
        task_execution_run_id: common.task_execution_run_id,
        occurrence_id: common.occurrence_id,
        event_type: envelope.event_type,
        schema_version: envelope.schema_version,
        idempotency_key: envelope.idempotency_key,
        correlation_id: envelope.correlation_id,
        payload_digest,
        payload: envelope.payload,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommonPayload {
    schema_version: u16,
    source_event_id: Uuid,
    occurrence_id: Uuid,
    task_execution_run_id: Uuid,
    binding_digest: String,
}

fn common_payload(event_type: &str, payload: &Value) -> Result<CommonPayload, CapacityResultError> {
    let common = match event_type {
        "capacity_accepted" => {
            let accepted: AcceptedPayload = serde_json::from_value(payload.clone())?;
            CommonPayload {
                schema_version: accepted.schema_version,
                source_event_id: accepted.source_event_id,
                occurrence_id: accepted.occurrence_id,
                task_execution_run_id: accepted.task_execution_run_id,
                binding_digest: accepted.binding_digest,
            }
        }
        "capacity_rejected" => {
            let rejected: RejectedPayload = serde_json::from_value(payload.clone())?;
            CommonPayload {
                schema_version: rejected.schema_version,
                source_event_id: rejected.source_event_id,
                occurrence_id: rejected.occurrence_id,
                task_execution_run_id: rejected.task_execution_run_id,
                binding_digest: rejected.binding_digest,
            }
        }
        "capacity_expired" => {
            let expired: ExpiredPayload = serde_json::from_value(payload.clone())?;
            CommonPayload {
                schema_version: expired.schema_version,
                source_event_id: expired.source_event_id,
                occurrence_id: expired.occurrence_id,
                task_execution_run_id: expired.task_execution_run_id,
                binding_digest: expired.binding_digest,
            }
        }
        "capacity_released" => return Err(CapacityResultError::ReleasedContractUnavailable),
        _ => return Err(CapacityResultError::InvalidEvent),
    };
    if common.schema_version != DEMAND_SCHEMA_VERSION as u16
        || common.source_event_id.is_nil()
        || common.occurrence_id.is_nil()
        || common.task_execution_run_id.is_nil()
        || decode_digest(&common.binding_digest).is_none()
    {
        return Err(CapacityResultError::InvalidEvent);
    }
    Ok(common)
}

fn validate_result_payload(event_type: &str, payload: &Value) -> Result<Uuid, CapacityResultError> {
    let common = common_payload(event_type, payload)?;
    match event_type {
        "capacity_accepted" => {
            let accepted: AcceptedPayload = serde_json::from_value(payload.clone())?;
            let _timestamps = (accepted.lease_expires_at, accepted.capacity_observed_at);
            if accepted.reservation_bundle_id.is_nil()
                || accepted.host_capacity_reservation_id.is_nil()
                || accepted.cluster_capacity_reservation_id.is_nil()
                || accepted.host_environment_id.is_nil()
                || accepted.host_generation <= 0
                || accepted.k3s_cluster_id.is_nil()
                || accepted.k3s_cluster_generation <= 0
                || decode_digest(&accepted.capacity_snapshot_digest).is_none()
            {
                return Err(CapacityResultError::InvalidEvent);
            }
        }
        "capacity_rejected" => {
            let rejected: RejectedPayload = serde_json::from_value(payload.clone())?;
            if !is_reason_code(&rejected.reason_code) {
                return Err(CapacityResultError::InvalidEvent);
            }
        }
        "capacity_expired" => {
            let expired: ExpiredPayload = serde_json::from_value(payload.clone())?;
            let _timestamps = (expired.lease_expires_at, expired.expired_at);
            if expired.binding_id.is_nil()
                || expired.binding_revision <= 0
                || expired.reservation_bundle_id.is_nil()
                || expired.host_capacity_reservation_id.is_nil()
                || expired.cluster_capacity_reservation_id.is_nil()
                || expired.host_environment_id.is_nil()
                || expired.host_generation <= 0
                || expired.k3s_cluster_id.is_nil()
                || expired.k3s_cluster_generation <= 0
            {
                return Err(CapacityResultError::InvalidEvent);
            }
        }
        "capacity_released" => return Err(CapacityResultError::ReleasedContractUnavailable),
        _ => return Err(CapacityResultError::InvalidEvent),
    }
    Ok(common.source_event_id)
}

fn result_digest(payload: &Value) -> Result<[u8; 32], serde_json::Error> {
    let bytes = serde_json::to_vec(payload)?;
    let mut hasher = Sha256::new();
    hasher.update(b"star.infrastructure-capacity-result-payload.v2\0");
    hasher.update(bytes);
    Ok(hasher.finalize().into())
}

fn decode_digest(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut digest = [0u8; 32];
    for (index, chunk) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let high = decode_hex(chunk[0])?;
        let low = decode_hex(chunk[1])?;
        digest[index] = (high << 4) | low;
    }
    Some(digest)
}

fn decode_hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn is_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_payload_matches_demand(
    result: &ValidatedCapacityResult,
    demand: &HostInfrastructureDemandV2,
) -> Result<(), CapacityResultError> {
    let common = common_payload(&result.event_type, &result.payload)?;
    let expected_binding_digest = demand
        .binding
        .digest()
        .map_err(|_| CapacityResultError::InvalidStoredDemand)?;
    if common.source_event_id != demand.event_id
        || common.occurrence_id != demand.occurrence_id
        || common.task_execution_run_id != demand.task_execution_run_id
        || decode_digest(&common.binding_digest) != Some(expected_binding_digest)
    {
        return Err(CapacityResultError::InvalidEvent);
    }
    match result.event_type.as_str() {
        "capacity_accepted" => {
            let accepted: AcceptedPayload = serde_json::from_value(result.payload.clone())?;
            if accepted.host_environment_id != demand.binding.host_environment_id
                || accepted.k3s_cluster_id != demand.binding.k3s_cluster_id
                || accepted.k3s_cluster_generation != demand.binding.k3s_cluster_generation
            {
                return Err(CapacityResultError::InvalidEvent);
            }
        }
        "capacity_expired" => {
            let expired: ExpiredPayload = serde_json::from_value(result.payload.clone())?;
            if expired.binding_id != demand.binding.binding_id
                || expired.binding_revision != demand.binding.binding_revision
                || expired.host_environment_id != demand.binding.host_environment_id
                || expired.k3s_cluster_id != demand.binding.k3s_cluster_id
                || expired.k3s_cluster_generation != demand.binding.k3s_cluster_generation
            {
                return Err(CapacityResultError::InvalidEvent);
            }
        }
        "capacity_rejected" => {}
        "capacity_released" => return Err(CapacityResultError::ReleasedContractUnavailable),
        _ => return Err(CapacityResultError::InvalidEvent),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        bind_result_scope_from_demand, parse_capacity_result_message, permanent_poison_fault,
        persist_then_ack, validate_payload_matches_demand, CapacityResultError, MAX_ENVELOPE_BYTES,
        MAX_FETCH_BYTES, MAX_FETCH_EXPIRY, MAX_FETCH_MESSAGES, MAX_PAYLOAD_BYTES,
    };
    use async_nats::HeaderMap;
    use chrono::{TimeZone, Utc};
    use domain_infrastructure::{
        ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
        KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV2,
    };
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    fn valid_envelope(payload: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "envelope_version": 1,
            "tenant_id": Uuid::from_u128(1),
            "event_id": Uuid::from_u128(2),
            "event_type": "capacity_rejected",
            "schema_version": 2,
            "idempotency_key": format!("capacity_rejected:{}", Uuid::from_u128(3)),
            "correlation_id": Uuid::from_u128(4),
            "payload": payload,
        }))
        .unwrap()
    }

    fn valid_payload() -> serde_json::Value {
        json!({
            "schema_version": 2,
            "source_event_id": Uuid::from_u128(3),
            "occurrence_id": Uuid::from_u128(5),
            "task_execution_run_id": Uuid::from_u128(6),
            "binding_digest": "01".repeat(32),
            "reason_code": "host_capacity_insufficient",
        })
    }

    fn headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Nats-Msg-Id", Uuid::from_u128(2).to_string());
        headers
    }

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn demand_fixture() -> HostInfrastructureDemandV2 {
        let budget = ExecutionResourceBudgetV1 {
            max_rss_bytes: 64 * 1024 * 1024,
            max_cpu_ms: 1_000,
            max_runtime_ms: 30_000,
            max_child_processes: 1,
            max_parallel_tools: 1,
            max_provider_calls: 1,
            max_output_bytes: 4_096,
            max_event_buffer_bytes: 4_096,
        };
        let binding = RunInfrastructureBindingSnapshotV2 {
            schema_version: 2,
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            binding_id: id(5),
            binding_revision: 1,
            host_environment_id: id(6),
            k3s_cluster_id: id(7),
            k3s_cluster_generation: 1,
            cluster_placement_policy: ClusterPlacementPolicyV1::PinnedCluster,
            provider: ProviderKind::MultipassK3sCluster,
            provider_profile_id: id(8),
            provider_profile_revision: 1,
            provider_profile_digest: [9; 32],
            namespace: KubernetesDnsLabel::new("run-test").unwrap(),
            service_account: KubernetesDnsLabel::new("agent-test").unwrap(),
            runtime_profile_id: id(10),
            runtime_profile_revision: 1,
            runtime_profile_digest: [11; 32],
            resource_budget: budget,
        };
        let requested_at = Utc.with_ymd_and_hms(2026, 10, 7, 0, 0, 0).unwrap();
        HostInfrastructureDemandV2 {
            schema_version: 2,
            event_id: id(12),
            correlation_id: id(13),
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            task_execution_run_id: id(14),
            schedule_rule_id: id(15),
            schedule_rule_version: 1,
            occurrence_id: id(16),
            binding,
            resource_budget: budget,
            requested_at,
            deadline_at: requested_at + chrono::Duration::minutes(5),
        }
    }

    fn binding_digest(demand: &HostInfrastructureDemandV2) -> String {
        demand
            .binding
            .digest()
            .unwrap()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn result_wire(event_type: &str, payload: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "envelope_version": 1,
            "tenant_id": id(1),
            "event_id": id(2),
            "event_type": event_type,
            "schema_version": 2,
            "idempotency_key": format!("{event_type}:{}", payload["source_event_id"].as_str().unwrap()),
            "correlation_id": id(13),
            "payload": payload,
        }))
        .unwrap()
    }

    fn accepted_payload(demand: &HostInfrastructureDemandV2) -> serde_json::Value {
        json!({
            "schema_version": 2,
            "source_event_id": demand.event_id,
            "occurrence_id": demand.occurrence_id,
            "task_execution_run_id": demand.task_execution_run_id,
            "binding_digest": binding_digest(demand),
            "reservation_bundle_id": id(20),
            "host_capacity_reservation_id": id(21),
            "cluster_capacity_reservation_id": id(22),
            "host_environment_id": demand.binding.host_environment_id,
            "host_generation": 1,
            "k3s_cluster_id": demand.binding.k3s_cluster_id,
            "k3s_cluster_generation": demand.binding.k3s_cluster_generation,
            "lease_expires_at": "2026-10-07T00:05:00Z",
            "capacity_snapshot_digest": "ab".repeat(32),
            "capacity_observed_at": "2026-10-07T00:00:00Z",
        })
    }

    fn rejected_payload(demand: &HostInfrastructureDemandV2) -> serde_json::Value {
        json!({
            "schema_version": 2,
            "source_event_id": demand.event_id,
            "occurrence_id": demand.occurrence_id,
            "task_execution_run_id": demand.task_execution_run_id,
            "binding_digest": binding_digest(demand),
            "reason_code": "host_capacity_insufficient",
        })
    }

    fn expired_payload(demand: &HostInfrastructureDemandV2) -> serde_json::Value {
        json!({
            "schema_version": 2,
            "source_event_id": demand.event_id,
            "occurrence_id": demand.occurrence_id,
            "task_execution_run_id": demand.task_execution_run_id,
            "binding_id": demand.binding.binding_id,
            "binding_revision": demand.binding.binding_revision,
            "binding_digest": binding_digest(demand),
            "reservation_bundle_id": id(20),
            "host_capacity_reservation_id": id(21),
            "cluster_capacity_reservation_id": id(22),
            "host_environment_id": demand.binding.host_environment_id,
            "host_generation": 1,
            "k3s_cluster_id": demand.binding.k3s_cluster_id,
            "k3s_cluster_generation": demand.binding.k3s_cluster_generation,
            "lease_expires_at": "2026-10-07T00:05:00Z",
            "expired_at": "2026-10-07T00:06:00Z",
        })
    }

    #[test]
    fn envelope_subject_and_jetstream_message_id_are_strictly_bound() {
        let wire = valid_envelope(valid_payload());
        let parsed = parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&headers()),
            &wire,
        );
        assert!(parsed.is_ok());

        assert!(parse_capacity_result_message(
            "star.infrastructure.v2.capacity_accepted",
            Some(&headers()),
            &wire,
        )
        .is_err());

        let mut mismatch = headers();
        mismatch.insert("Nats-Msg-Id", Uuid::from_u128(99).to_string());
        assert!(parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&mismatch),
            &wire,
        )
        .is_err());
    }

    #[test]
    fn unknown_fields_versions_and_unproduced_release_contract_fail_closed() {
        let mut payload = valid_payload();
        payload["extra"] = json!(true);
        assert!(parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&headers()),
            &valid_envelope(payload),
        )
        .is_err());

        let mut unsupported =
            serde_json::from_slice::<serde_json::Value>(&valid_envelope(valid_payload())).unwrap();
        unsupported["schema_version"] = json!(3);
        assert!(parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&headers()),
            &serde_json::to_vec(&unsupported).unwrap(),
        )
        .is_err());

        let release = json!({
            "schema_version": 2,
            "source_event_id": Uuid::from_u128(3),
            "occurrence_id": Uuid::from_u128(5),
            "task_execution_run_id": Uuid::from_u128(6),
            "binding_digest": "01".repeat(32),
        });
        let release_envelope = serde_json::to_vec(&json!({
            "envelope_version": 1,
            "tenant_id": Uuid::from_u128(1),
            "event_id": Uuid::from_u128(2),
            "event_type": "capacity_released",
            "schema_version": 2,
            "idempotency_key": format!("capacity_released:{}", Uuid::from_u128(3)),
            "correlation_id": Uuid::from_u128(4),
            "payload": release,
        }))
        .unwrap();
        assert!(matches!(
            parse_capacity_result_message(
                "star.infrastructure.v2.capacity_released",
                Some(&headers()),
                &release_envelope,
            ),
            Err(CapacityResultError::ReleasedContractUnavailable)
        ));
    }

    #[test]
    fn accepted_payload_is_parsed_and_bound_to_the_stored_run_snapshot() {
        let demand = demand_fixture();
        let payload = accepted_payload(&demand);
        let wire = result_wire("capacity_accepted", payload.clone());
        let parsed = parse_capacity_result_message(
            "star.infrastructure.v2.capacity_accepted",
            Some(&headers()),
            &wire,
        )
        .unwrap();
        assert!(validate_payload_matches_demand(&parsed, &demand).is_ok());

        let mut mismatched_binding = payload;
        mismatched_binding["host_environment_id"] = json!(id(99));
        let mismatched = parse_capacity_result_message(
            "star.infrastructure.v2.capacity_accepted",
            Some(&headers()),
            &result_wire("capacity_accepted", mismatched_binding),
        )
        .unwrap();
        assert!(matches!(
            validate_payload_matches_demand(&mismatched, &demand),
            Err(CapacityResultError::InvalidEvent)
        ));
    }

    #[test]
    fn expired_payload_is_parsed_and_binding_revision_is_enforced() {
        let demand = demand_fixture();
        let payload = expired_payload(&demand);
        let parsed = parse_capacity_result_message(
            "star.infrastructure.v2.capacity_expired",
            Some(&headers()),
            &result_wire("capacity_expired", payload.clone()),
        )
        .unwrap();
        assert!(validate_payload_matches_demand(&parsed, &demand).is_ok());

        let mut mismatched_binding = payload;
        mismatched_binding["binding_revision"] = json!(2);
        let mismatched = parse_capacity_result_message(
            "star.infrastructure.v2.capacity_expired",
            Some(&headers()),
            &result_wire("capacity_expired", mismatched_binding),
        )
        .unwrap();
        assert!(matches!(
            validate_payload_matches_demand(&mismatched, &demand),
            Err(CapacityResultError::InvalidEvent)
        ));
    }

    #[test]
    fn result_scope_is_hydrated_from_the_stored_demand_not_the_wire_envelope() {
        let demand = demand_fixture();
        let parsed = parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&headers()),
            &result_wire("capacity_rejected", rejected_payload(&demand)),
        )
        .unwrap();
        assert_eq!(parsed.project_id, Uuid::nil());
        assert_eq!(parsed.branch_id, Uuid::nil());
        assert_eq!(parsed.engineering_run_id, Uuid::nil());

        let work_item_id = id(17);
        let mut bound = parsed;
        bind_result_scope_from_demand(&mut bound, &demand, work_item_id).unwrap();
        assert_eq!(bound.project_id, demand.project_id);
        assert_eq!(bound.branch_id, demand.branch_id);
        assert_eq!(bound.engineering_run_id, demand.engineering_run_id);
        assert_eq!(bound.work_item_id, work_item_id);
    }

    #[test]
    fn payload_and_envelope_byte_bounds_are_enforced() {
        const {
            assert!(MAX_FETCH_MESSAGES == 1);
            assert!(MAX_FETCH_BYTES > MAX_ENVELOPE_BYTES);
        }
        assert_eq!(MAX_FETCH_EXPIRY.as_secs(), 2);

        let oversized = vec![b'x'; MAX_ENVELOPE_BYTES + 1];
        assert!(parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&headers()),
            &oversized,
        )
        .is_err());

        let payload = json!({
            "schema_version": 2,
            "source_event_id": Uuid::from_u128(3),
            "occurrence_id": Uuid::from_u128(5),
            "task_execution_run_id": Uuid::from_u128(6),
            "binding_digest": "01".repeat(32),
            "reason_code": "x".repeat(MAX_PAYLOAD_BYTES),
        });
        assert!(parse_capacity_result_message(
            "star.infrastructure.v2.capacity_rejected",
            Some(&headers()),
            &valid_envelope(payload),
        )
        .is_err());
    }

    #[tokio::test]
    async fn acknowledgement_is_after_commit_and_never_runs_after_store_failure() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let persist_order = Arc::clone(&order);
        let ack_order = Arc::clone(&order);
        let result = persist_then_ack(
            async move {
                persist_order.lock().unwrap().push("commit");
                Ok::<_, CapacityResultError>(17)
            },
            async move {
                ack_order.lock().unwrap().push("ack");
                Ok::<_, CapacityResultError>(())
            },
        )
        .await
        .unwrap();
        assert_eq!(result, 17);
        assert_eq!(*order.lock().unwrap(), ["commit", "ack"]);

        let ack_called = Arc::new(Mutex::new(false));
        let ack_called_in_future = Arc::clone(&ack_called);
        let failed = persist_then_ack(
            async { Err::<(), _>(CapacityResultError::UnknownDemand) },
            async move {
                *ack_called_in_future.lock().unwrap() = true;
                Ok(())
            },
        )
        .await;
        assert!(matches!(failed, Err(CapacityResultError::UnknownDemand)));
        assert!(!*ack_called.lock().unwrap());
    }

    #[tokio::test]
    async fn ack_failure_leaves_committed_event_safe_for_duplicate_redelivery() {
        let persisted_count = Arc::new(Mutex::new(0usize));
        let persist_first = Arc::clone(&persisted_count);
        let first = persist_then_ack(
            async move {
                *persist_first.lock().unwrap() += 1;
                Ok::<_, CapacityResultError>(true)
            },
            async { Err(CapacityResultError::InvalidEvent) },
        )
        .await;
        assert!(matches!(first, Err(CapacityResultError::InvalidEvent)));
        assert_eq!(*persisted_count.lock().unwrap(), 1);

        let persist_replay = Arc::clone(&persisted_count);
        let replay = persist_then_ack(
            async move {
                // The database Inbox already contains the event; its idempotent read returns duplicate.
                let persisted = *persist_replay.lock().unwrap();
                Ok::<_, CapacityResultError>(persisted == 0)
            },
            async { Ok::<_, CapacityResultError>(()) },
        )
        .await
        .unwrap();
        assert!(!replay);
        assert_eq!(*persisted_count.lock().unwrap(), 1);
    }

    #[test]
    fn run_owner_data_errors_never_enter_poison_quarantine() {
        use platform_messaging_runtime::PermanentPoisonFault;

        assert_eq!(
            permanent_poison_fault(&CapacityResultError::InvalidEvent),
            Some(PermanentPoisonFault::InvalidCapacityResultEvent)
        );
        assert_eq!(
            permanent_poison_fault(&CapacityResultError::Json(
                serde_json::from_str::<serde_json::Value>("{").unwrap_err()
            )),
            Some(PermanentPoisonFault::MalformedCapacityResultJson)
        );
        assert_eq!(
            permanent_poison_fault(&CapacityResultError::IdempotencyConflict),
            Some(PermanentPoisonFault::CapacityResultEventIdConflict)
        );
        assert_eq!(
            permanent_poison_fault(&CapacityResultError::UnknownDemand),
            None
        );
        assert_eq!(
            permanent_poison_fault(&CapacityResultError::InvalidStoredDemand),
            None
        );
        assert_eq!(
            permanent_poison_fault(&CapacityResultError::ReleasedContractUnavailable),
            None
        );
    }
}
