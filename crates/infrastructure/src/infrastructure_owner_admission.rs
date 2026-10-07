//! @cypher schema=1 source_sha256=6fb46e4b478fef28f96d7c194d939596f82454b1d6f32bb4c2d159b19ffe2fad
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_admission.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::module",kind:"module"})
//! MERGE (store:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::PostgresInfrastructureOwnerAdmission",kind:"struct"})
//! MERGE (outcome:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::InfrastructureOwnerAdmissionOutcome",kind:"enum"})
//! MERGE (error:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::InfrastructureOwnerAdmissionError",kind:"enum"})
//! MERGE (admit:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::PostgresInfrastructureOwnerAdmission.admit_inbox",kind:"method"})
//! MERGE (expire_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::PostgresInfrastructureOwnerAdmission.expire_due_reservations",kind:"method"})
//! MERGE (expire_one:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::PostgresInfrastructureOwnerAdmission.expire_candidate",kind:"method"})
//! MERGE (locks:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::lock_resource_scope",kind:"function"})
//! MERGE (lock_keys:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::unique_sorted_lock_keys",kind:"function"})
//! MERGE (aggregate:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::read_active_reservations",kind:"function"})
//! MERGE (accept:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::record_accepted",kind:"function"})
//! MERGE (reject:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::record_rejected",kind:"function"})
//! MERGE (read_profile:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::read_provider_profile_revision",kind:"function"})
//! MERGE (loaded_profile:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::LoadedProviderProfileRevision",kind:"struct"})
//! MERGE (expiry_candidate:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::ReservationExpiryCandidate",kind:"struct"})
//! MERGE (lifecycle_row:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::ReservationLifecycleRow",kind:"struct"})
//! MERGE (physical_row:Type {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::PhysicalReservationRow",kind:"struct"})
//! MERGE (same_facts:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::PhysicalReservationRow.matches",kind:"method"})
//! MERGE (expiry_outbox:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::write_expiration_outbox",kind:"function"})
//! MERGE (max_expiry_batch:Variable {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::MAX_EXPIRY_BATCH",kind:"constant"})
//! MERGE (expiry_bound_test:Symbol {id:"crates/infrastructure/src/infrastructure_owner_admission.rs::tests.expiry_scan_rejects_invalid_scope_and_batch_before_database_access",kind:"test"})
//! MERGE (set_tenant_scope:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::set_tenant_scope",kind:"function"})
//! MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
//! MERGE (bundle:Table {id:"infrastructure.capacity_reservation_bundle"})
//! MERGE (physical:Table {id:"infrastructure.capacity_physical_reservation",class:"Work"})
//! MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
//! MERGE (reservation_event:Table {id:"infrastructure.capacity_reservation_event"})
//! MERGE (physical_event:Table {id:"infrastructure.capacity_physical_reservation_event",class:"Transaction"})
//! MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox"})
//! MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery"})
//! MERGE (provider_profile:Table {id:"infrastructure.provider_profile_revision",class:"Master"})
//! MERGE (evaluator:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::evaluate_infrastructure_capacity_candidate",kind:"function"})
//! MERGE (derive_request:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureProviderProfileRevisionV1.capacity_request_for_binding",kind:"method"})
//! MERGE (pg_pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(store)
//! MERGE (module)-[:DEFINES]->(outcome)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (module)-[:DEFINES]->(locks)
//! MERGE (module)-[:DEFINES]->(lock_keys)
//! MERGE (module)-[:DEFINES]->(aggregate)
//! MERGE (module)-[:DEFINES]->(accept)
//! MERGE (module)-[:DEFINES]->(reject)
//! MERGE (module)-[:DEFINES]->(read_profile)
//! MERGE (module)-[:DEFINES]->(loaded_profile)
//! MERGE (module)-[:DEFINES]->(expiry_candidate)
//! MERGE (module)-[:DEFINES]->(lifecycle_row)
//! MERGE (module)-[:DEFINES]->(physical_row)
//! MERGE (module)-[:DEFINES]->(expiry_outbox)
//! MERGE (module)-[:DEFINES]->(max_expiry_batch)
//! MERGE (store)-[:USES_TYPE]->(pg_pool)
//! MERGE (admit)-[:CALLS]->(locks)
//! MERGE (admit)-[:CALLS]->(aggregate)
//! MERGE (admit)-[:CALLS]->(read_profile)
//! MERGE (admit)-[:CALLS]->(derive_request)
//! MERGE (admit)-[:CALLS]->(evaluator)
//! MERGE (admit)-[:CALLS]->(accept)
//! MERGE (admit)-[:CALLS]->(reject)
//! MERGE (admit)-[:READS]->(inbox)
//! MERGE (admit)-[:READS]->(bundle)
//! MERGE (admit)-[:READS]->(physical)
//! MERGE (admit)-[:WRITES]->(bundle)
//! MERGE (admit)-[:WRITES]->(physical)
//! MERGE (admit)-[:WRITES]->(physical_event)
//! MERGE (admit)-[:WRITES]->(receipt)
//! MERGE (admit)-[:WRITES]->(reservation_event)
//! MERGE (admit)-[:WRITES]->(outbox)
//! MERGE (admit)-[:WRITES]->(delivery)
//! MERGE (admit)-[:USES_TYPE]->(loaded_profile)
//! MERGE (expire_due)-[:CALLS]->(set_tenant_scope)
//! MERGE (expire_due)-[:CALLS]->(expire_one)
//! MERGE (expire_due)-[:READS]->(bundle)
//! MERGE (expire_due)-[:USES]->(max_expiry_batch)
//! MERGE (expire_one)-[:CALLS]->(locks)
//! MERGE (expire_one)-[:CALLS]->(set_tenant_scope)
//! MERGE (expire_one)-[:CALLS]->(same_facts)
//! MERGE (expire_one)-[:CALLS]->(expiry_outbox)
//! MERGE (expire_due)-[:USES_TYPE]->(expiry_candidate)
//! MERGE (expire_one)-[:USES_TYPE]->(expiry_candidate)
//! MERGE (expire_one)-[:USES_TYPE]->(lifecycle_row)
//! MERGE (expire_one)-[:USES_TYPE]->(physical_row)
//! MERGE (expire_one)-[:READS]->(bundle)
//! MERGE (expire_one)-[:READS]->(physical)
//! MERGE (expire_one)-[:READS]->(outbox)
//! MERGE (expire_one)-[:WRITES]->(bundle)
//! MERGE (expire_one)-[:WRITES]->(physical)
//! MERGE (expire_one)-[:WRITES]->(physical_event)
//! MERGE (expire_one)-[:WRITES]->(reservation_event)
//! MERGE (expire_one)-[:WRITES]->(outbox)
//! MERGE (expire_one)-[:WRITES]->(delivery)
//! MERGE (physical_row)-[:DEFINES]->(same_facts)
//! MERGE (same_facts)-[:USES_TYPE]->(lifecycle_row)
//! MERGE (expiry_outbox)-[:USES_TYPE]->(lifecycle_row)
//! MERGE (expiry_bound_test)-[:CALLS]->(expire_due)
//! MERGE (read_profile)-[:READS]->(provider_profile)
//! MERGE (read_profile)-[:USES_TYPE]->(loaded_profile)
//! @endcypher

//! Infrastructure-owner admission for one already received Run-owner demand.
//!
//! This transaction serializes competing reservations for the tenant and physical Host/exact
//! K3s cluster scopes, recomputes capacity from the global physical ledger, and writes both
//! ledgers plus receipt, audit facts, and owner Outbox atomically. It does not provision a VM,
//! start a K3s workload, or enable a TaskExecutionRun.

use chrono::{DateTime, Duration, Utc};
use domain_infrastructure::{
    evaluate_infrastructure_capacity_candidate, ClusterPlacementPolicyV1,
    InfrastructureCapacityEvaluationError, InfrastructureCapacityRequestV2,
    InfrastructureCapacitySnapshotV2, InfrastructureProviderProfileError,
    InfrastructureProviderProfileRevisionV1, InfrastructureResourceVectorV1, ProviderKind,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

use crate::infrastructure_owner_inbox::{
    payload_digest as demand_payload_digest, placement_policy_value, provider_kind_value,
    set_tenant_scope,
};

const OWNER_ROLE: &str = "SET LOCAL ROLE star_infrastructure_owner";
const ADMISSION_LEASE_SECONDS: i64 = 30;
const BASE_RETRY_SECONDS: i64 = 1;
const MAX_RETRY_SECONDS: i64 = 30;
const TERMINAL_RETENTION_HOURS: i64 = 24;
const MAX_EXPIRY_BATCH: usize = 64;

/// Idempotent result of processing one Infrastructure Inbox demand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfrastructureOwnerAdmissionOutcome {
    /// Both Host and exact-cluster reservations committed with their owner facts.
    Accepted {
        /// Paired reservation bundle identity.
        reservation_bundle_id: Uuid,
        /// Reservation identity consumed by Host-level capacity accounting.
        host_capacity_reservation_id: Uuid,
        /// Distinct reservation identity consumed by exact-cluster accounting.
        cluster_capacity_reservation_id: Uuid,
        /// True when the durable terminal receipt already existed.
        replayed: bool,
    },
    /// A permanent validation or deadline failure was recorded and published.
    Rejected {
        /// Stable machine-readable owner reason code.
        reason_code: String,
        /// True when the durable terminal receipt already existed.
        replayed: bool,
    },
    /// Temporary capacity or provider-observation issue; the Inbox remains retryable.
    RetryScheduled {
        /// Attempt number after this processing attempt.
        attempt_count: i32,
        /// Earliest time a consumer may retry this demand.
        next_attempt_at: DateTime<Utc>,
    },
    /// Another owner worker currently holds an unexpired Inbox lease.
    Busy {
        /// Current lease expiry.
        lease_expires_at: DateTime<Utc>,
    },
    /// A retryable demand has not reached its backoff time.
    NotDue {
        /// Scheduled retry time.
        next_attempt_at: DateTime<Utc>,
    },
}

/// PostgreSQL admission adapter for the Infrastructure owner's Inbox and capacity ledger.
#[derive(Debug, Clone)]
pub struct PostgresInfrastructureOwnerAdmission {
    pool: PgPool,
    actor_id: Uuid,
    worker_id: Uuid,
}

impl PostgresInfrastructureOwnerAdmission {
    /// Construct the owner adapter with stable service-actor and worker identities.
    pub fn new(
        pool: PgPool,
        actor_id: Uuid,
        worker_id: Uuid,
    ) -> Result<Self, InfrastructureOwnerAdmissionError> {
        if actor_id.is_nil() || worker_id.is_nil() {
            return Err(InfrastructureOwnerAdmissionError::InvalidOwnerIdentity);
        }
        Ok(Self {
            pool,
            actor_id,
            worker_id,
        })
    }

    /// Admit a persisted Inbox item against a trusted, fresh provider observation.
    ///
    /// The provider snapshot must come from trusted Infrastructure observation. The capacity
    /// request is derived from the tenant-scoped persisted profile revision inside this owner
    /// transaction; it is never caller supplied. Neither producer is wired to a public API or
    /// consumer, so Schedule execution remains fail-closed.
    pub async fn admit_inbox(
        &self,
        tenant_id: Uuid,
        inbox_id: Uuid,
        observed_snapshot: &InfrastructureCapacitySnapshotV2,
    ) -> Result<InfrastructureOwnerAdmissionOutcome, InfrastructureOwnerAdmissionError> {
        if tenant_id.is_nil() || inbox_id.is_nil() {
            return Err(InfrastructureOwnerAdmissionError::InvalidInboxIdentity);
        }

        let mut tx = self.pool.begin().await?;
        sqlx::query(OWNER_ROLE).execute(&mut *tx).await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        sqlx::query("SELECT set_config('app.actor_id', $1, true)")
            .bind(self.actor_id.to_string())
            .execute(&mut *tx)
            .await?;

        let Some((host_environment_id, k3s_cluster_id)) =
            read_inbox_lock_scope(&mut tx, tenant_id, inbox_id).await?
        else {
            return Err(InfrastructureOwnerAdmissionError::InboxNotFound);
        };
        lock_resource_scope(&mut tx, tenant_id, host_environment_id, k3s_cluster_id).await?;

        let row = read_inbox_for_update(&mut tx, tenant_id, inbox_id).await?;
        let demand = verify_inbox_demand(tenant_id, &row)?;
        if let Some(outcome) = read_terminal_outcome(&mut tx, tenant_id, &row, &demand).await? {
            tx.commit().await?;
            return Ok(outcome);
        }
        if matches!(row.processing_state.as_str(), "accepted" | "rejected") {
            return Err(InfrastructureOwnerAdmissionError::MissingTerminalReceipt);
        }

        let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *tx)
            .await?;
        if demand.deadline_at <= now {
            let reason_code = "admission_deadline_expired";
            record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now).await?;
            tx.commit().await?;
            return Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                reason_code: reason_code.to_owned(),
                replayed: false,
            });
        }

        match row.processing_state.as_str() {
            "retry" if row.next_attempt_at > now => {
                let next_attempt_at = row.next_attempt_at;
                tx.commit().await?;
                return Ok(InfrastructureOwnerAdmissionOutcome::NotDue { next_attempt_at });
            }
            "leased" if row.lease_expires_at.is_some_and(|expires| expires > now) => {
                let Some(lease_expires_at) = row.lease_expires_at else {
                    return Err(InfrastructureOwnerAdmissionError::InvalidInboxState);
                };
                tx.commit().await?;
                return Ok(InfrastructureOwnerAdmissionOutcome::Busy { lease_expires_at });
            }
            "received" | "retry" | "leased" => {}
            _ => return Err(InfrastructureOwnerAdmissionError::InvalidInboxState),
        }

        let Some(next_attempt_count) = row.attempt_count.checked_add(1) else {
            return Err(InfrastructureOwnerAdmissionError::AttemptCounterOverflow);
        };
        let lease_expires_at = std::cmp::min(
            now + Duration::seconds(ADMISSION_LEASE_SECONDS),
            demand.deadline_at,
        );
        sqlx::query(
            "UPDATE infrastructure.capacity_demand_inbox \
             SET processing_state = 'leased', attempt_count = $3, lease_owner_id = $4, \
                 lease_expires_at = $5, updated_at = $6 \
             WHERE tenant_id = $1 AND inbox_id = $2",
        )
        .bind(tenant_id)
        .bind(inbox_id)
        .bind(next_attempt_count)
        .bind(self.worker_id)
        .bind(lease_expires_at)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let Some(profile) = read_provider_profile_revision(
            &mut tx,
            tenant_id,
            demand.binding.provider_profile_id,
            demand.binding.provider_profile_revision,
        )
        .await?
        else {
            let reason_code = "provider_profile_not_found";
            record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now).await?;
            tx.commit().await?;
            return Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                reason_code: reason_code.to_owned(),
                replayed: false,
            });
        };
        if !profile.persisted_digest_matches {
            let reason_code = "provider_profile_digest_mismatch";
            record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now).await?;
            tx.commit().await?;
            return Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                reason_code: reason_code.to_owned(),
                replayed: false,
            });
        }
        let request = match profile
            .profile
            .capacity_request_for_binding(&demand.binding)
        {
            Ok(request) => request,
            Err(error) => {
                let reason_code = provider_profile_error_code(error);
                record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now).await?;
                tx.commit().await?;
                return Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                    reason_code: reason_code.to_owned(),
                    replayed: false,
                });
            }
        };

        if observed_snapshot.host_environment_id != row.host_environment_id
            || observed_snapshot.host_generation != row.host_generation
            || observed_snapshot.k3s_cluster_id != row.k3s_cluster_id
            || observed_snapshot.k3s_cluster_generation != row.k3s_cluster_generation
        {
            let reason_code = "provider_generation_or_scope_mismatch";
            record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now).await?;
            tx.commit().await?;
            return Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                reason_code: reason_code.to_owned(),
                replayed: false,
            });
        }

        let mut snapshot = observed_snapshot.clone();
        snapshot.active_reservations = Some(
            read_active_reservations(
                &mut tx,
                row.host_environment_id,
                row.host_generation,
                true,
                now,
            )
            .await?,
        );
        snapshot.cluster_active_reservations = Some(
            read_active_reservations(
                &mut tx,
                row.k3s_cluster_id,
                row.k3s_cluster_generation,
                false,
                now,
            )
            .await?,
        );

        match evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now) {
            Ok(candidate) => {
                let lease_expires_at =
                    match demand
                        .deadline_at
                        .checked_add_signed(Duration::milliseconds(
                            demand.resource_budget.max_runtime_ms,
                        )) {
                        Some(value) => value,
                        None => {
                            let reason_code = "reservation_lease_overflow";
                            record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now)
                                .await?;
                            tx.commit().await?;
                            return Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                                reason_code: reason_code.to_owned(),
                                replayed: false,
                            });
                        }
                    };
                let result = record_accepted(
                    &mut tx,
                    AcceptedAdmission {
                        tenant_id,
                        actor_id: self.actor_id,
                        row: &row,
                        demand: &demand,
                        snapshot: &snapshot,
                        request: &request,
                        candidate: &candidate,
                        lease_expires_at,
                        now,
                    },
                )
                .await?;
                tx.commit().await?;
                Ok(result)
            }
            Err(error) if is_retryable_capacity_error(error) => {
                let delay = retry_delay(next_attempt_count);
                let next_attempt_at = std::cmp::min(now + delay, demand.deadline_at);
                sqlx::query(
                    "UPDATE infrastructure.capacity_demand_inbox \
                     SET processing_state = 'retry', next_attempt_at = $3, lease_owner_id = NULL, \
                         lease_expires_at = NULL, updated_at = $4 \
                     WHERE tenant_id = $1 AND inbox_id = $2",
                )
                .bind(tenant_id)
                .bind(inbox_id)
                .bind(next_attempt_at)
                .bind(now)
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(InfrastructureOwnerAdmissionOutcome::RetryScheduled {
                    attempt_count: next_attempt_count,
                    next_attempt_at,
                })
            }
            Err(error) => {
                let reason_code = capacity_error_code(error);
                record_rejected(&mut tx, tenant_id, &row, &demand, reason_code, now).await?;
                tx.commit().await?;
                Ok(InfrastructureOwnerAdmissionOutcome::Rejected {
                    reason_code: reason_code.to_owned(),
                    replayed: false,
                })
            }
        }
    }

    /// Expire a bounded set of this tenant's elapsed admission holds.
    ///
    /// The caller must supply a tenant obtained from trusted owner scheduling. Each paired
    /// tenant/global reservation transition commits atomically and publishes an immutable
    /// Outbox fact. This primitive is not a Runtime lease-renewal protocol or a live worker.
    pub async fn expire_due_reservations(
        &self,
        tenant_id: Uuid,
        max_items: usize,
    ) -> Result<usize, InfrastructureOwnerAdmissionError> {
        if tenant_id.is_nil() {
            return Err(InfrastructureOwnerAdmissionError::InvalidTenantIdentity);
        }
        if !(1..=MAX_EXPIRY_BATCH).contains(&max_items) {
            return Err(InfrastructureOwnerAdmissionError::InvalidExpiryBatchSize);
        }

        let mut scan_tx = self.pool.begin().await?;
        sqlx::query(OWNER_ROLE).execute(&mut *scan_tx).await?;
        set_tenant_scope(&mut scan_tx, tenant_id).await?;
        let candidates: Vec<ReservationExpiryCandidate> = sqlx::query_as(
            "SELECT reservation_bundle_id, host_environment_id, k3s_cluster_id \
             FROM infrastructure.capacity_reservation_bundle \
             WHERE tenant_id = $1 AND reservation_state = 'held' \
               AND lease_expires_at <= clock_timestamp() \
             ORDER BY lease_expires_at, reservation_bundle_id LIMIT $2",
        )
        .bind(tenant_id)
        .bind(max_items as i64)
        .fetch_all(&mut *scan_tx)
        .await?;
        scan_tx.commit().await?;

        let mut expired_count = 0;
        for candidate in candidates {
            if self.expire_candidate(tenant_id, &candidate).await? {
                expired_count += 1;
            }
        }
        Ok(expired_count)
    }

    async fn expire_candidate(
        &self,
        tenant_id: Uuid,
        candidate: &ReservationExpiryCandidate,
    ) -> Result<bool, InfrastructureOwnerAdmissionError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(OWNER_ROLE).execute(&mut *tx).await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        sqlx::query("SELECT set_config('app.actor_id', $1, true)")
            .bind(self.actor_id.to_string())
            .execute(&mut *tx)
            .await?;
        lock_resource_scope(
            &mut tx,
            tenant_id,
            candidate.host_environment_id,
            candidate.k3s_cluster_id,
        )
        .await?;

        let row = sqlx::query_as::<_, ReservationLifecycleRow>(
            "SELECT reservation_bundle_id, source_event_id, occurrence_id, task_execution_run_id, \
                    binding_id, binding_revision, binding_digest, host_capacity_reservation_id, \
                    host_environment_id, host_generation, host_cpu_millicores, host_memory_bytes, \
                    host_ephemeral_storage_bytes, host_gpu_units, cluster_capacity_reservation_id, \
                    k3s_cluster_id, k3s_cluster_generation, cluster_cpu_millicores, \
                    cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units, \
                    capacity_snapshot_digest, capacity_observed_at, lease_expires_at, reservation_state \
             FROM infrastructure.capacity_reservation_bundle \
             WHERE tenant_id = $1 AND reservation_bundle_id = $2 FOR UPDATE",
        )
        .bind(tenant_id)
        .bind(candidate.reservation_bundle_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InfrastructureOwnerAdmissionError::ReservationNotFound)?;
        if row.host_environment_id != candidate.host_environment_id
            || row.k3s_cluster_id != candidate.k3s_cluster_id
        {
            return Err(InfrastructureOwnerAdmissionError::ReservationPairInconsistent);
        }

        let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *tx)
            .await?;
        if row.reservation_state != "held" || row.lease_expires_at > now {
            tx.commit().await?;
            return Ok(false);
        }

        let physical = sqlx::query_as::<_, PhysicalReservationRow>(
            "SELECT reservation_bundle_id, host_capacity_reservation_id, host_environment_id, \
                    host_generation, host_cpu_millicores, host_memory_bytes, \
                    host_ephemeral_storage_bytes, host_gpu_units, cluster_capacity_reservation_id, \
                    k3s_cluster_id, k3s_cluster_generation, cluster_cpu_millicores, \
                    cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units, \
                    capacity_snapshot_digest, capacity_observed_at, lease_expires_at, reservation_state \
             FROM infrastructure.capacity_physical_reservation \
             WHERE reservation_bundle_id = $1 FOR UPDATE",
        )
        .bind(row.reservation_bundle_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InfrastructureOwnerAdmissionError::ReservationPairInconsistent)?;
        if !physical.matches(&row) || physical.reservation_state != "held" {
            return Err(InfrastructureOwnerAdmissionError::ReservationPairInconsistent);
        }

        let correlation_id: Uuid = sqlx::query_scalar(
            "SELECT correlation_id FROM infrastructure.capacity_owner_outbox \
             WHERE tenant_id = $1 AND reservation_bundle_id = $2 \
               AND event_type = 'capacity_accepted'",
        )
        .bind(tenant_id)
        .bind(row.reservation_bundle_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InfrastructureOwnerAdmissionError::MissingAcceptedOutbox)?;
        let retention_expires_at = now + Duration::hours(TERMINAL_RETENTION_HOURS);

        let physical_update = sqlx::query(
            "UPDATE infrastructure.capacity_physical_reservation \
             SET reservation_state = 'expired', terminal_at = $2, \
                 retention_expires_at = $3, updated_at = $2 \
             WHERE reservation_bundle_id = $1 AND reservation_state = 'held' \
               AND lease_expires_at <= $2",
        )
        .bind(row.reservation_bundle_id)
        .bind(now)
        .bind(retention_expires_at)
        .execute(&mut *tx)
        .await?;
        if physical_update.rows_affected() != 1 {
            return Err(InfrastructureOwnerAdmissionError::ReservationPairInconsistent);
        }

        let tenant_update = sqlx::query(
            "UPDATE infrastructure.capacity_reservation_bundle \
             SET reservation_state = 'expired', terminal_at = $3, \
                 retention_expires_at = $4, updated_at = $3 \
             WHERE tenant_id = $1 AND reservation_bundle_id = $2 \
               AND reservation_state = 'held' AND lease_expires_at <= $3",
        )
        .bind(tenant_id)
        .bind(row.reservation_bundle_id)
        .bind(now)
        .bind(retention_expires_at)
        .execute(&mut *tx)
        .await?;
        if tenant_update.rows_affected() != 1 {
            return Err(InfrastructureOwnerAdmissionError::ReservationPairInconsistent);
        }

        sqlx::query(
            "INSERT INTO infrastructure.capacity_reservation_event \
             (tenant_id, reservation_bundle_id, event_type, host_capacity_reservation_id, \
              cluster_capacity_reservation_id, actor_id, correlation_id, details) \
             VALUES ($1,$2,'expired',$3,$4,$5,$6,$7)",
        )
        .bind(tenant_id)
        .bind(row.reservation_bundle_id)
        .bind(row.host_capacity_reservation_id)
        .bind(row.cluster_capacity_reservation_id)
        .bind(self.actor_id)
        .bind(correlation_id)
        .bind(json!({
            "source_event_id": row.source_event_id,
            "occurrence_id": row.occurrence_id,
            "lease_expires_at": row.lease_expires_at,
            "expired_at": now,
        }))
        .execute(&mut *tx)
        .await?;
        write_expiration_outbox(&mut tx, tenant_id, &row, correlation_id, now).await?;
        tx.commit().await?;
        Ok(true)
    }
}

#[derive(sqlx::FromRow)]
struct ReservationExpiryCandidate {
    reservation_bundle_id: Uuid,
    host_environment_id: Uuid,
    k3s_cluster_id: Uuid,
}

#[derive(sqlx::FromRow)]
struct ReservationLifecycleRow {
    reservation_bundle_id: Uuid,
    source_event_id: Uuid,
    occurrence_id: Uuid,
    task_execution_run_id: Uuid,
    binding_id: Uuid,
    binding_revision: i64,
    binding_digest: Vec<u8>,
    host_capacity_reservation_id: Uuid,
    host_environment_id: Uuid,
    host_generation: i64,
    host_cpu_millicores: i64,
    host_memory_bytes: i64,
    host_ephemeral_storage_bytes: i64,
    host_gpu_units: i32,
    cluster_capacity_reservation_id: Uuid,
    k3s_cluster_id: Uuid,
    k3s_cluster_generation: i64,
    cluster_cpu_millicores: i64,
    cluster_memory_bytes: i64,
    cluster_ephemeral_storage_bytes: i64,
    cluster_gpu_units: i32,
    capacity_snapshot_digest: Vec<u8>,
    capacity_observed_at: DateTime<Utc>,
    lease_expires_at: DateTime<Utc>,
    reservation_state: String,
}

#[derive(sqlx::FromRow)]
struct PhysicalReservationRow {
    reservation_bundle_id: Uuid,
    host_capacity_reservation_id: Uuid,
    host_environment_id: Uuid,
    host_generation: i64,
    host_cpu_millicores: i64,
    host_memory_bytes: i64,
    host_ephemeral_storage_bytes: i64,
    host_gpu_units: i32,
    cluster_capacity_reservation_id: Uuid,
    k3s_cluster_id: Uuid,
    k3s_cluster_generation: i64,
    cluster_cpu_millicores: i64,
    cluster_memory_bytes: i64,
    cluster_ephemeral_storage_bytes: i64,
    cluster_gpu_units: i32,
    capacity_snapshot_digest: Vec<u8>,
    capacity_observed_at: DateTime<Utc>,
    lease_expires_at: DateTime<Utc>,
    reservation_state: String,
}

impl PhysicalReservationRow {
    fn matches(&self, tenant: &ReservationLifecycleRow) -> bool {
        self.reservation_bundle_id == tenant.reservation_bundle_id
            && self.host_capacity_reservation_id == tenant.host_capacity_reservation_id
            && self.host_environment_id == tenant.host_environment_id
            && self.host_generation == tenant.host_generation
            && self.host_cpu_millicores == tenant.host_cpu_millicores
            && self.host_memory_bytes == tenant.host_memory_bytes
            && self.host_ephemeral_storage_bytes == tenant.host_ephemeral_storage_bytes
            && self.host_gpu_units == tenant.host_gpu_units
            && self.cluster_capacity_reservation_id == tenant.cluster_capacity_reservation_id
            && self.k3s_cluster_id == tenant.k3s_cluster_id
            && self.k3s_cluster_generation == tenant.k3s_cluster_generation
            && self.cluster_cpu_millicores == tenant.cluster_cpu_millicores
            && self.cluster_memory_bytes == tenant.cluster_memory_bytes
            && self.cluster_ephemeral_storage_bytes == tenant.cluster_ephemeral_storage_bytes
            && self.cluster_gpu_units == tenant.cluster_gpu_units
            && self.capacity_snapshot_digest == tenant.capacity_snapshot_digest
            && self.capacity_observed_at == tenant.capacity_observed_at
            && self.lease_expires_at == tenant.lease_expires_at
    }
}

async fn write_expiration_outbox(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    row: &ReservationLifecycleRow,
    correlation_id: Uuid,
    now: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let event_id = Uuid::new_v4();
    let idempotency_key = format!("capacity_expired:{}", row.source_event_id);
    let binding_digest = row
        .binding_digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let payload = json!({
        "schema_version": 2,
        "source_event_id": row.source_event_id,
        "occurrence_id": row.occurrence_id,
        "task_execution_run_id": row.task_execution_run_id,
        "binding_id": row.binding_id,
        "binding_revision": row.binding_revision,
        "binding_digest": binding_digest,
        "reservation_bundle_id": row.reservation_bundle_id,
        "host_capacity_reservation_id": row.host_capacity_reservation_id,
        "cluster_capacity_reservation_id": row.cluster_capacity_reservation_id,
        "host_environment_id": row.host_environment_id,
        "host_generation": row.host_generation,
        "k3s_cluster_id": row.k3s_cluster_id,
        "k3s_cluster_generation": row.k3s_cluster_generation,
        "lease_expires_at": row.lease_expires_at,
        "expired_at": now,
    });
    sqlx::query(
        "INSERT INTO infrastructure.capacity_owner_outbox \
         (event_id, tenant_id, source_event_id, occurrence_id, reservation_bundle_id, \
          event_type, schema_version, idempotency_key, correlation_id, payload) \
         VALUES ($1,$2,$3,$4,$5,'capacity_expired',2,$6,$7,$8)",
    )
    .bind(event_id)
    .bind(tenant_id)
    .bind(row.source_event_id)
    .bind(row.occurrence_id)
    .bind(row.reservation_bundle_id)
    .bind(idempotency_key)
    .bind(correlation_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO infrastructure.capacity_outbox_delivery (tenant_id, event_id) \
         VALUES ($1,$2)",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

struct DemandInboxRow {
    inbox_id: Uuid,
    source_event_id: Uuid,
    occurrence_id: Uuid,
    project_id: Uuid,
    branch_id: Uuid,
    engineering_run_id: Uuid,
    task_execution_run_id: Uuid,
    schedule_rule_id: Uuid,
    schedule_rule_revision: i64,
    binding_id: Uuid,
    binding_revision: i64,
    binding_digest: Vec<u8>,
    host_environment_id: Uuid,
    host_generation: i64,
    k3s_cluster_id: Uuid,
    k3s_cluster_generation: i64,
    provider_kind: String,
    placement_policy: String,
    payload: Value,
    payload_digest: Vec<u8>,
    deadline_at: DateTime<Utc>,
    processing_state: String,
    attempt_count: i32,
    next_attempt_at: DateTime<Utc>,
    lease_expires_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct DemandReceiptRow {
    source_event_id: Uuid,
    occurrence_id: Uuid,
    payload_digest: Vec<u8>,
    outcome: String,
    reservation_bundle_id: Option<Uuid>,
    reason_code: Option<String>,
}

async fn read_inbox_lock_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    inbox_id: Uuid,
) -> Result<Option<(Uuid, Uuid)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT host_environment_id, k3s_cluster_id \
         FROM infrastructure.capacity_demand_inbox \
         WHERE tenant_id = $1 AND inbox_id = $2",
    )
    .bind(tenant_id)
    .bind(inbox_id)
    .fetch_optional(&mut **tx)
    .await
}

async fn lock_resource_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    host_environment_id: Uuid,
    k3s_cluster_id: Uuid,
) -> Result<(), sqlx::Error> {
    let tenant_scope = format!("tenant:{tenant_id}");
    let keys = unique_sorted_lock_keys([
        advisory_lock_key(tx, &format!("{tenant_scope}:host"), host_environment_id).await?,
        advisory_lock_key(tx, &format!("{tenant_scope}:cluster"), k3s_cluster_id).await?,
        advisory_lock_key(tx, "physical:host", host_environment_id).await?,
        advisory_lock_key(tx, "physical:cluster", k3s_cluster_id).await?,
    ]);
    for key in keys {
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(key)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

fn unique_sorted_lock_keys(keys: [i64; 4]) -> Vec<i64> {
    let mut keys = keys.into_iter().collect::<Vec<_>>();
    keys.sort_unstable();
    keys.dedup();
    keys
}

async fn advisory_lock_key(
    tx: &mut Transaction<'_, Postgres>,
    scope: &str,
    resource_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT hashtextextended($1, 0)")
        .bind(format!(
            "star.infrastructure.capacity.{scope}:{resource_id}"
        ))
        .fetch_one(&mut **tx)
        .await
}

#[cfg(test)]
mod tests {
    use super::{
        unique_sorted_lock_keys, InfrastructureOwnerAdmissionError,
        PostgresInfrastructureOwnerAdmission,
    };
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[test]
    fn lock_key_collisions_keep_every_distinct_resource_lock() {
        assert_eq!(unique_sorted_lock_keys([9, 9, 3, 5]), vec![3, 5, 9]);
        assert_eq!(unique_sorted_lock_keys([4, 4, 4, 4]), vec![4]);
        assert_eq!(unique_sorted_lock_keys([8, 1, 7, 2]), vec![1, 2, 7, 8]);
    }

    #[tokio::test]
    async fn expiry_scan_rejects_invalid_scope_and_batch_before_database_access() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgresql://localhost/unused")
            .expect("valid lazy database URL");
        let owner = PostgresInfrastructureOwnerAdmission::new(pool, Uuid::new_v4(), Uuid::new_v4())
            .expect("non-nil owner identities");

        assert!(matches!(
            owner.expire_due_reservations(Uuid::nil(), 1).await,
            Err(InfrastructureOwnerAdmissionError::InvalidTenantIdentity)
        ));
        assert!(matches!(
            owner.expire_due_reservations(Uuid::new_v4(), 0).await,
            Err(InfrastructureOwnerAdmissionError::InvalidExpiryBatchSize)
        ));
        assert!(matches!(
            owner.expire_due_reservations(Uuid::new_v4(), 65).await,
            Err(InfrastructureOwnerAdmissionError::InvalidExpiryBatchSize)
        ));
    }
}

async fn read_inbox_for_update(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    inbox_id: Uuid,
) -> Result<DemandInboxRow, InfrastructureOwnerAdmissionError> {
    let row = sqlx::query(
        "SELECT inbox_id, source_event_id, occurrence_id, project_id, branch_id, \
                engineering_run_id, task_execution_run_id, schedule_rule_id, \
                schedule_rule_revision, binding_id, binding_revision, binding_digest, \
                host_environment_id, host_generation, k3s_cluster_id, \
                k3s_cluster_generation, provider_kind, placement_policy, payload, \
                payload_digest, deadline_at, processing_state, attempt_count, \
                next_attempt_at, lease_expires_at \
         FROM infrastructure.capacity_demand_inbox \
         WHERE tenant_id = $1 AND inbox_id = $2 FOR UPDATE",
    )
    .bind(tenant_id)
    .bind(inbox_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(InfrastructureOwnerAdmissionError::InboxNotFound)?;

    Ok(DemandInboxRow {
        inbox_id: row.try_get("inbox_id")?,
        source_event_id: row.try_get("source_event_id")?,
        occurrence_id: row.try_get("occurrence_id")?,
        project_id: row.try_get("project_id")?,
        branch_id: row.try_get("branch_id")?,
        engineering_run_id: row.try_get("engineering_run_id")?,
        task_execution_run_id: row.try_get("task_execution_run_id")?,
        schedule_rule_id: row.try_get("schedule_rule_id")?,
        schedule_rule_revision: row.try_get("schedule_rule_revision")?,
        binding_id: row.try_get("binding_id")?,
        binding_revision: row.try_get("binding_revision")?,
        binding_digest: row.try_get("binding_digest")?,
        host_environment_id: row.try_get("host_environment_id")?,
        host_generation: row.try_get("host_generation")?,
        k3s_cluster_id: row.try_get("k3s_cluster_id")?,
        k3s_cluster_generation: row.try_get("k3s_cluster_generation")?,
        provider_kind: row.try_get("provider_kind")?,
        placement_policy: row.try_get("placement_policy")?,
        payload: row.try_get("payload")?,
        payload_digest: row.try_get("payload_digest")?,
        deadline_at: row.try_get("deadline_at")?,
        processing_state: row.try_get("processing_state")?,
        attempt_count: row.try_get("attempt_count")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        lease_expires_at: row.try_get("lease_expires_at")?,
    })
}

fn verify_inbox_demand(
    tenant_id: Uuid,
    row: &DemandInboxRow,
) -> Result<domain_infrastructure::HostInfrastructureDemandV2, InfrastructureOwnerAdmissionError> {
    let demand: domain_infrastructure::HostInfrastructureDemandV2 =
        serde_json::from_value(row.payload.clone())
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidDemandPayload)?;
    demand
        .validate()
        .map_err(|_| InfrastructureOwnerAdmissionError::InvalidDemandPayload)?;
    let (canonical_payload, digest) = demand_payload_digest(&demand)
        .map_err(|_| InfrastructureOwnerAdmissionError::InvalidDemandPayload)?;
    let binding_digest = demand
        .binding
        .digest()
        .map_err(|_| InfrastructureOwnerAdmissionError::InvalidDemandPayload)?;
    if demand.tenant_id != tenant_id
        || demand.event_id != row.source_event_id
        || demand.occurrence_id != row.occurrence_id
        || demand.project_id != row.project_id
        || demand.branch_id != row.branch_id
        || demand.engineering_run_id != row.engineering_run_id
        || demand.task_execution_run_id != row.task_execution_run_id
        || demand.schedule_rule_id != row.schedule_rule_id
        || demand.schedule_rule_version != row.schedule_rule_revision
        || demand.binding.binding_id != row.binding_id
        || demand.binding.binding_revision != row.binding_revision
        || binding_digest.as_slice() != row.binding_digest.as_slice()
        || demand.binding.host_environment_id != row.host_environment_id
        || demand.binding.k3s_cluster_id != row.k3s_cluster_id
        || demand.binding.k3s_cluster_generation != row.k3s_cluster_generation
        || provider_kind_value(demand.binding.provider) != row.provider_kind
        || placement_policy_value(demand.binding.cluster_placement_policy) != row.placement_policy
        || demand.deadline_at.timestamp_micros() != row.deadline_at.timestamp_micros()
        || canonical_payload != row.payload
        || digest.as_slice() != row.payload_digest.as_slice()
        || row.host_generation <= 0
    {
        return Err(InfrastructureOwnerAdmissionError::InboxPayloadMismatch);
    }
    Ok(demand)
}

async fn read_provider_profile_revision(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    profile_id: Uuid,
    revision: i64,
) -> Result<Option<LoadedProviderProfileRevision>, InfrastructureOwnerAdmissionError> {
    let Some(row) = sqlx::query(
        "SELECT schema_version, tenant_id, profile_id, revision, provider_kind, placement_policy, \
                host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, host_gpu_units, \
                cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units, \
                profile_digest \
         FROM infrastructure.provider_profile_revision \
         WHERE tenant_id = $1 AND profile_id = $2 AND revision = $3 \
           AND valid_from <= transaction_timestamp()",
    )
    .bind(tenant_id)
    .bind(profile_id)
    .bind(revision)
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(None);
    };

    let provider: String = row.try_get("provider_kind")?;
    let provider = match provider.as_str() {
        "multipass_k3s_cluster" => ProviderKind::MultipassK3sCluster,
        "remote_k3s" => ProviderKind::RemoteK3s,
        _ => return Err(InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord),
    };
    let placement_policy: String = row.try_get("placement_policy")?;
    let placement_policy = match placement_policy.as_str() {
        "shared_pool" => ClusterPlacementPolicyV1::SharedPool,
        "run_dedicated" => ClusterPlacementPolicyV1::RunDedicated,
        "pinned_cluster" => ClusterPlacementPolicyV1::PinnedCluster,
        _ => return Err(InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord),
    };
    let resource_vector = |cpu: i64, memory: i64, storage: i64, gpu: i32| {
        Ok::<_, InfrastructureOwnerAdmissionError>(InfrastructureResourceVectorV1 {
            cpu_millicores: u64::try_from(cpu)
                .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?,
            memory_bytes: u64::try_from(memory)
                .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?,
            ephemeral_storage_bytes: u64::try_from(storage)
                .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?,
            gpu_units: u32::try_from(gpu)
                .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?,
        })
    };

    let profile = InfrastructureProviderProfileRevisionV1 {
        schema_version: u16::try_from(row.try_get::<i16, _>("schema_version")?)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?,
        tenant_id: row.try_get("tenant_id")?,
        profile_id: row.try_get("profile_id")?,
        revision: row.try_get("revision")?,
        provider,
        placement_policy,
        host_resources: resource_vector(
            row.try_get("host_cpu_millicores")?,
            row.try_get("host_memory_bytes")?,
            row.try_get("host_ephemeral_storage_bytes")?,
            row.try_get("host_gpu_units")?,
        )?,
        cluster_resources: resource_vector(
            row.try_get("cluster_cpu_millicores")?,
            row.try_get("cluster_memory_bytes")?,
            row.try_get("cluster_ephemeral_storage_bytes")?,
            row.try_get("cluster_gpu_units")?,
        )?,
    };
    let persisted_digest: Vec<u8> = row.try_get("profile_digest")?;
    let persisted_digest: [u8; 32] = persisted_digest
        .try_into()
        .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?;
    let persisted_digest_matches = profile
        .digest()
        .map_err(|_| InfrastructureOwnerAdmissionError::InvalidProviderProfileRecord)?
        == persisted_digest;

    Ok(Some(LoadedProviderProfileRevision {
        profile,
        persisted_digest_matches,
    }))
}

struct LoadedProviderProfileRevision {
    profile: InfrastructureProviderProfileRevisionV1,
    persisted_digest_matches: bool,
}

fn provider_profile_error_code(error: InfrastructureProviderProfileError) -> &'static str {
    match error {
        InfrastructureProviderProfileError::UnsupportedSchemaVersion => {
            "provider_profile_schema_unsupported"
        }
        InfrastructureProviderProfileError::InvalidIdentity
        | InfrastructureProviderProfileError::InvalidResources => "provider_profile_invalid",
        InfrastructureProviderProfileError::BindingMismatch => "provider_profile_binding_mismatch",
        InfrastructureProviderProfileError::DigestMismatch => "provider_profile_digest_mismatch",
    }
}

async fn read_active_reservations(
    tx: &mut Transaction<'_, Postgres>,
    resource_id: Uuid,
    generation: i64,
    host_scope: bool,
    now: DateTime<Utc>,
) -> Result<InfrastructureResourceVectorV1, InfrastructureOwnerAdmissionError> {
    let totals: (i64, i64, i64, i64) = if host_scope {
        sqlx::query_as(
            "SELECT COALESCE(SUM(host_cpu_millicores), 0)::BIGINT, \
                    COALESCE(SUM(host_memory_bytes), 0)::BIGINT, \
                    COALESCE(SUM(host_ephemeral_storage_bytes), 0)::BIGINT, \
                    COALESCE(SUM(host_gpu_units), 0)::BIGINT \
             FROM infrastructure.capacity_physical_reservation \
             WHERE host_environment_id = $1 AND host_generation = $2 \
               AND reservation_state = 'held' AND lease_expires_at > $3",
        )
        .bind(resource_id)
        .bind(generation)
        .bind(now)
        .fetch_one(&mut **tx)
        .await?
    } else {
        sqlx::query_as(
            "SELECT COALESCE(SUM(cluster_cpu_millicores), 0)::BIGINT, \
                    COALESCE(SUM(cluster_memory_bytes), 0)::BIGINT, \
                    COALESCE(SUM(cluster_ephemeral_storage_bytes), 0)::BIGINT, \
                    COALESCE(SUM(cluster_gpu_units), 0)::BIGINT \
             FROM infrastructure.capacity_physical_reservation \
             WHERE k3s_cluster_id = $1 \
               AND k3s_cluster_generation = $2 AND reservation_state = 'held' \
               AND lease_expires_at > $3",
        )
        .bind(resource_id)
        .bind(generation)
        .bind(now)
        .fetch_one(&mut **tx)
        .await?
    };
    let (cpu, memory, storage, gpu) = totals;
    Ok(InfrastructureResourceVectorV1 {
        cpu_millicores: u64::try_from(cpu)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidStoredCapacity)?,
        memory_bytes: u64::try_from(memory)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidStoredCapacity)?,
        ephemeral_storage_bytes: u64::try_from(storage)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidStoredCapacity)?,
        gpu_units: u32::try_from(gpu)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidStoredCapacity)?,
    })
}

async fn read_terminal_outcome(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    row: &DemandInboxRow,
    demand: &domain_infrastructure::HostInfrastructureDemandV2,
) -> Result<Option<InfrastructureOwnerAdmissionOutcome>, InfrastructureOwnerAdmissionError> {
    let receipts: Vec<DemandReceiptRow> = sqlx::query_as(
        "SELECT source_event_id, occurrence_id, payload_digest, outcome, \
                reservation_bundle_id, reason_code \
         FROM infrastructure.capacity_demand_receipt \
         WHERE tenant_id = $1 AND (source_event_id = $2 OR occurrence_id = $3) \
         ORDER BY receipt_id",
    )
    .bind(tenant_id)
    .bind(row.source_event_id)
    .bind(row.occurrence_id)
    .fetch_all(&mut **tx)
    .await?;
    if receipts.is_empty() {
        return Ok(None);
    }
    if receipts.len() != 1 {
        return Err(InfrastructureOwnerAdmissionError::IdempotencyConflict);
    }
    let receipt = &receipts[0];
    if receipt.source_event_id != row.source_event_id
        || receipt.occurrence_id != row.occurrence_id
        || receipt.payload_digest.as_slice() != row.payload_digest.as_slice()
        || row.source_event_id != demand.event_id
        || row.occurrence_id != demand.occurrence_id
    {
        return Err(InfrastructureOwnerAdmissionError::IdempotencyConflict);
    }
    match receipt.outcome.as_str() {
        "accepted" if row.processing_state == "accepted" => {
            let Some(reservation_bundle_id) = receipt.reservation_bundle_id else {
                return Err(InfrastructureOwnerAdmissionError::MissingReservationBundle);
            };
            let reservation: Option<(Uuid, Uuid)> = sqlx::query_as(
                "SELECT host_capacity_reservation_id, cluster_capacity_reservation_id \
                 FROM infrastructure.capacity_reservation_bundle \
                 WHERE tenant_id = $1 AND reservation_bundle_id = $2 \
                   AND source_event_id = $3 AND occurrence_id = $4",
            )
            .bind(tenant_id)
            .bind(reservation_bundle_id)
            .bind(row.source_event_id)
            .bind(row.occurrence_id)
            .fetch_optional(&mut **tx)
            .await?;
            let Some((host_capacity_reservation_id, cluster_capacity_reservation_id)) = reservation
            else {
                return Err(InfrastructureOwnerAdmissionError::MissingReservationBundle);
            };
            Ok(Some(InfrastructureOwnerAdmissionOutcome::Accepted {
                reservation_bundle_id,
                host_capacity_reservation_id,
                cluster_capacity_reservation_id,
                replayed: true,
            }))
        }
        "rejected" if row.processing_state == "rejected" => {
            let Some(reason_code) = &receipt.reason_code else {
                return Err(InfrastructureOwnerAdmissionError::InvalidTerminalReceipt);
            };
            Ok(Some(InfrastructureOwnerAdmissionOutcome::Rejected {
                reason_code: reason_code.clone(),
                replayed: true,
            }))
        }
        _ => Err(InfrastructureOwnerAdmissionError::InvalidTerminalReceipt),
    }
}

struct AcceptedAdmission<'a> {
    tenant_id: Uuid,
    actor_id: Uuid,
    row: &'a DemandInboxRow,
    demand: &'a domain_infrastructure::HostInfrastructureDemandV2,
    snapshot: &'a InfrastructureCapacitySnapshotV2,
    request: &'a InfrastructureCapacityRequestV2,
    candidate: &'a domain_infrastructure::InfrastructureCapacityCandidateV2,
    lease_expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
}

async fn record_accepted(
    tx: &mut Transaction<'_, Postgres>,
    admission: AcceptedAdmission<'_>,
) -> Result<InfrastructureOwnerAdmissionOutcome, InfrastructureOwnerAdmissionError> {
    let AcceptedAdmission {
        tenant_id,
        actor_id,
        row,
        demand,
        snapshot,
        request,
        candidate,
        lease_expires_at,
        now,
    } = admission;
    let reservation_bundle_id = Uuid::new_v4();
    let host_capacity_reservation_id = Uuid::new_v4();
    let mut cluster_capacity_reservation_id = Uuid::new_v4();
    while cluster_capacity_reservation_id == host_capacity_reservation_id {
        cluster_capacity_reservation_id = Uuid::new_v4();
    }
    let (snapshot_digest, host, cluster) = capacity_evidence(snapshot, request)?;

    sqlx::query(
        "INSERT INTO infrastructure.capacity_physical_reservation \
         (reservation_bundle_id, host_capacity_reservation_id, host_environment_id, host_generation, \
          host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, host_gpu_units, \
          cluster_capacity_reservation_id, k3s_cluster_id, k3s_cluster_generation, \
          cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units, \
          capacity_snapshot_digest, capacity_observed_at, lease_expires_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
    )
    .bind(reservation_bundle_id)
    .bind(host_capacity_reservation_id)
    .bind(candidate.host_environment_id)
    .bind(candidate.host_generation)
    .bind(host.cpu_millicores)
    .bind(host.memory_bytes)
    .bind(host.ephemeral_storage_bytes)
    .bind(host.gpu_units)
    .bind(cluster_capacity_reservation_id)
    .bind(candidate.k3s_cluster_id)
    .bind(candidate.k3s_cluster_generation)
    .bind(cluster.cpu_millicores)
    .bind(cluster.memory_bytes)
    .bind(cluster.ephemeral_storage_bytes)
    .bind(cluster.gpu_units)
    .bind(snapshot_digest.as_slice())
    .bind(candidate.observed_at)
    .bind(lease_expires_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO infrastructure.capacity_physical_reservation_event \
         (reservation_bundle_id, event_type, host_capacity_reservation_id, host_environment_id, \
          host_generation, host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, \
          host_gpu_units, cluster_capacity_reservation_id, k3s_cluster_id, k3s_cluster_generation, \
          cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units, \
          capacity_snapshot_digest, capacity_observed_at, lease_expires_at, actor_id) \
         VALUES ($1,'held',$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)",
    )
    .bind(reservation_bundle_id)
    .bind(host_capacity_reservation_id)
    .bind(candidate.host_environment_id)
    .bind(candidate.host_generation)
    .bind(host.cpu_millicores)
    .bind(host.memory_bytes)
    .bind(host.ephemeral_storage_bytes)
    .bind(host.gpu_units)
    .bind(cluster_capacity_reservation_id)
    .bind(candidate.k3s_cluster_id)
    .bind(candidate.k3s_cluster_generation)
    .bind(cluster.cpu_millicores)
    .bind(cluster.memory_bytes)
    .bind(cluster.ephemeral_storage_bytes)
    .bind(cluster.gpu_units)
    .bind(snapshot_digest.as_slice())
    .bind(candidate.observed_at)
    .bind(lease_expires_at)
    .bind(actor_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO infrastructure.capacity_reservation_bundle \
         (reservation_bundle_id, tenant_id, source_event_id, occurrence_id, task_execution_run_id, \
          binding_id, binding_revision, binding_digest, host_capacity_reservation_id, \
          host_environment_id, host_generation, host_cpu_millicores, host_memory_bytes, \
          host_ephemeral_storage_bytes, host_gpu_units, cluster_capacity_reservation_id, \
          k3s_cluster_id, k3s_cluster_generation, cluster_cpu_millicores, cluster_memory_bytes, \
          cluster_ephemeral_storage_bytes, cluster_gpu_units, capacity_snapshot_digest, \
          capacity_observed_at, lease_expires_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25)",
    )
    .bind(reservation_bundle_id)
    .bind(tenant_id)
    .bind(row.source_event_id)
    .bind(row.occurrence_id)
    .bind(demand.task_execution_run_id)
    .bind(demand.binding.binding_id)
    .bind(demand.binding.binding_revision)
    .bind(candidate.binding_digest.as_slice())
    .bind(host_capacity_reservation_id)
    .bind(candidate.host_environment_id)
    .bind(candidate.host_generation)
    .bind(host.cpu_millicores)
    .bind(host.memory_bytes)
    .bind(host.ephemeral_storage_bytes)
    .bind(host.gpu_units)
    .bind(cluster_capacity_reservation_id)
    .bind(candidate.k3s_cluster_id)
    .bind(candidate.k3s_cluster_generation)
    .bind(cluster.cpu_millicores)
    .bind(cluster.memory_bytes)
    .bind(cluster.ephemeral_storage_bytes)
    .bind(cluster.gpu_units)
    .bind(snapshot_digest.as_slice())
    .bind(candidate.observed_at)
    .bind(lease_expires_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO infrastructure.capacity_reservation_event \
         (tenant_id, reservation_bundle_id, event_type, host_capacity_reservation_id, \
          cluster_capacity_reservation_id, actor_id, correlation_id, details) \
         VALUES ($1,$2,'held',$3,$4,$5,$6,$7)",
    )
    .bind(tenant_id)
    .bind(reservation_bundle_id)
    .bind(host_capacity_reservation_id)
    .bind(cluster_capacity_reservation_id)
    .bind(actor_id)
    .bind(demand.correlation_id)
    .bind(json!({
        "source_event_id": row.source_event_id,
        "occurrence_id": row.occurrence_id,
        "binding_digest": hex_digest(&candidate.binding_digest),
        "capacity_snapshot_digest": hex_digest(&snapshot_digest),
        "capacity_observed_at": candidate.observed_at,
    }))
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO infrastructure.capacity_demand_receipt \
         (tenant_id, source_event_id, occurrence_id, payload_digest, outcome, \
          reservation_bundle_id, reason_code, correlation_id) \
         VALUES ($1,$2,$3,$4,'accepted',$5,NULL,$6)",
    )
    .bind(tenant_id)
    .bind(row.source_event_id)
    .bind(row.occurrence_id)
    .bind(row.payload_digest.as_slice())
    .bind(reservation_bundle_id)
    .bind(demand.correlation_id)
    .execute(&mut **tx)
    .await?;

    let terminal_at = now;
    let retention_expires_at = now + Duration::hours(TERMINAL_RETENTION_HOURS);
    sqlx::query(
        "UPDATE infrastructure.capacity_demand_inbox \
         SET processing_state = 'accepted', lease_owner_id = NULL, lease_expires_at = NULL, \
             terminal_at = $3, retention_expires_at = $4, updated_at = $3 \
         WHERE tenant_id = $1 AND inbox_id = $2",
    )
    .bind(tenant_id)
    .bind(row.inbox_id)
    .bind(terminal_at)
    .bind(retention_expires_at)
    .execute(&mut **tx)
    .await?;

    write_owner_outbox(
        tx,
        tenant_id,
        row,
        Some(reservation_bundle_id),
        "capacity_accepted",
        demand.correlation_id,
        json!({
            "schema_version": 2,
            "source_event_id": row.source_event_id,
            "occurrence_id": row.occurrence_id,
            "task_execution_run_id": demand.task_execution_run_id,
            "binding_digest": hex_digest(&candidate.binding_digest),
            "reservation_bundle_id": reservation_bundle_id,
            "host_capacity_reservation_id": host_capacity_reservation_id,
            "cluster_capacity_reservation_id": cluster_capacity_reservation_id,
            "host_environment_id": candidate.host_environment_id,
            "host_generation": candidate.host_generation,
            "k3s_cluster_id": candidate.k3s_cluster_id,
            "k3s_cluster_generation": candidate.k3s_cluster_generation,
            "lease_expires_at": lease_expires_at,
            "capacity_snapshot_digest": hex_digest(&snapshot_digest),
            "capacity_observed_at": candidate.observed_at,
        }),
    )
    .await?;

    Ok(InfrastructureOwnerAdmissionOutcome::Accepted {
        reservation_bundle_id,
        host_capacity_reservation_id,
        cluster_capacity_reservation_id,
        replayed: false,
    })
}

async fn record_rejected(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    row: &DemandInboxRow,
    demand: &domain_infrastructure::HostInfrastructureDemandV2,
    reason_code: &str,
    now: DateTime<Utc>,
) -> Result<(), InfrastructureOwnerAdmissionError> {
    let terminal_at = now;
    let retention_expires_at = now + Duration::hours(TERMINAL_RETENTION_HOURS);
    sqlx::query(
        "UPDATE infrastructure.capacity_demand_inbox \
         SET processing_state = 'rejected', lease_owner_id = NULL, lease_expires_at = NULL, \
             terminal_at = $3, retention_expires_at = $4, updated_at = $3 \
         WHERE tenant_id = $1 AND inbox_id = $2",
    )
    .bind(tenant_id)
    .bind(row.inbox_id)
    .bind(terminal_at)
    .bind(retention_expires_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO infrastructure.capacity_demand_receipt \
         (tenant_id, source_event_id, occurrence_id, payload_digest, outcome, \
          reservation_bundle_id, reason_code, correlation_id) \
         VALUES ($1,$2,$3,$4,'rejected',NULL,$5,$6)",
    )
    .bind(tenant_id)
    .bind(row.source_event_id)
    .bind(row.occurrence_id)
    .bind(row.payload_digest.as_slice())
    .bind(reason_code)
    .bind(demand.correlation_id)
    .execute(&mut **tx)
    .await?;
    write_owner_outbox(
        tx,
        tenant_id,
        row,
        None,
        "capacity_rejected",
        demand.correlation_id,
        json!({
            "schema_version": 2,
            "source_event_id": row.source_event_id,
            "occurrence_id": row.occurrence_id,
            "task_execution_run_id": demand.task_execution_run_id,
            "binding_digest": hex_digest(&demand.binding.digest().map_err(|_| InfrastructureOwnerAdmissionError::InvalidDemandPayload)?),
            "reason_code": reason_code,
        }),
    )
    .await?;
    Ok(())
}

async fn write_owner_outbox(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    row: &DemandInboxRow,
    reservation_bundle_id: Option<Uuid>,
    event_type: &str,
    correlation_id: Uuid,
    payload: Value,
) -> Result<(), sqlx::Error> {
    let event_id = Uuid::new_v4();
    let idempotency_key = format!("{}:{}", event_type, row.source_event_id);
    sqlx::query(
        "INSERT INTO infrastructure.capacity_owner_outbox \
         (event_id, tenant_id, source_event_id, occurrence_id, reservation_bundle_id, \
          event_type, schema_version, idempotency_key, correlation_id, payload) \
         VALUES ($1,$2,$3,$4,$5,$6,2,$7,$8,$9)",
    )
    .bind(event_id)
    .bind(tenant_id)
    .bind(row.source_event_id)
    .bind(row.occurrence_id)
    .bind(reservation_bundle_id)
    .bind(event_type)
    .bind(idempotency_key)
    .bind(correlation_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO infrastructure.capacity_outbox_delivery (tenant_id, event_id) \
         VALUES ($1,$2)",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

struct PersistedResourceVector {
    cpu_millicores: i64,
    memory_bytes: i64,
    ephemeral_storage_bytes: i64,
    gpu_units: i32,
}

fn capacity_evidence(
    snapshot: &InfrastructureCapacitySnapshotV2,
    request: &InfrastructureCapacityRequestV2,
) -> Result<
    ([u8; 32], PersistedResourceVector, PersistedResourceVector),
    InfrastructureOwnerAdmissionError,
> {
    let bytes = serde_json::to_vec(&(snapshot, request))?;
    let mut hasher = Sha256::new();
    hasher.update(b"star.infrastructure-capacity-observation.v2\0");
    hasher.update(bytes);
    let digest = hasher.finalize().into();
    Ok((
        digest,
        persist_resource_vector(request.host_resources)?,
        persist_resource_vector(request.cluster_resources)?,
    ))
}

fn persist_resource_vector(
    vector: InfrastructureResourceVectorV1,
) -> Result<PersistedResourceVector, InfrastructureOwnerAdmissionError> {
    Ok(PersistedResourceVector {
        cpu_millicores: i64::try_from(vector.cpu_millicores)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidCapacityRequest)?,
        memory_bytes: i64::try_from(vector.memory_bytes)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidCapacityRequest)?,
        ephemeral_storage_bytes: i64::try_from(vector.ephemeral_storage_bytes)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidCapacityRequest)?,
        gpu_units: i32::try_from(vector.gpu_units)
            .map_err(|_| InfrastructureOwnerAdmissionError::InvalidCapacityRequest)?,
    })
}

fn retry_delay(attempt_count: i32) -> Duration {
    let exponent = u32::try_from(attempt_count.saturating_sub(1))
        .unwrap_or(0)
        .min(5);
    Duration::seconds((BASE_RETRY_SECONDS * 2_i64.pow(exponent)).min(MAX_RETRY_SECONDS))
}

fn is_retryable_capacity_error(error: InfrastructureCapacityEvaluationError) -> bool {
    matches!(
        error,
        InfrastructureCapacityEvaluationError::CapacityUnknown
            | InfrastructureCapacityEvaluationError::SnapshotStale
            | InfrastructureCapacityEvaluationError::HostCapacityExhausted
            | InfrastructureCapacityEvaluationError::HostCapacityInsufficient
            | InfrastructureCapacityEvaluationError::ClusterCapacityExhausted
            | InfrastructureCapacityEvaluationError::ClusterCapacityInsufficient
    )
}

fn capacity_error_code(error: InfrastructureCapacityEvaluationError) -> &'static str {
    match error {
        InfrastructureCapacityEvaluationError::InvalidDemand => "invalid_demand",
        InfrastructureCapacityEvaluationError::UnsupportedSchemaVersion => {
            "unsupported_capacity_schema"
        }
        InfrastructureCapacityEvaluationError::InvalidSnapshot => "invalid_capacity_snapshot",
        InfrastructureCapacityEvaluationError::CapacityUnknown => "capacity_unknown",
        InfrastructureCapacityEvaluationError::ScopeMismatch => "capacity_scope_mismatch",
        InfrastructureCapacityEvaluationError::SnapshotStale => "capacity_snapshot_stale",
        InfrastructureCapacityEvaluationError::AdmissionDeadlineExpired => {
            "admission_deadline_expired"
        }
        InfrastructureCapacityEvaluationError::HostCapacityExhausted => "host_capacity_exhausted",
        InfrastructureCapacityEvaluationError::HostCapacityInsufficient => {
            "host_capacity_insufficient"
        }
        InfrastructureCapacityEvaluationError::ClusterCapacityExhausted => {
            "cluster_capacity_exhausted"
        }
        InfrastructureCapacityEvaluationError::ClusterCapacityInsufficient => {
            "cluster_capacity_insufficient"
        }
        InfrastructureCapacityEvaluationError::InvalidRequest => "invalid_capacity_request",
        InfrastructureCapacityEvaluationError::RequestBindingMismatch => {
            "capacity_request_binding_mismatch"
        }
        InfrastructureCapacityEvaluationError::RequestBelowRunMemoryBudget => {
            "capacity_below_run_memory_budget"
        }
    }
}

fn hex_digest(digest: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut value = String::with_capacity(64);
    for byte in digest {
        let _ = write!(value, "{byte:02x}");
    }
    value
}

/// Fail-closed errors from the PostgreSQL Infrastructure-owner admission transaction.
#[derive(Debug, Error)]
pub enum InfrastructureOwnerAdmissionError {
    /// Stable Infrastructure service actor or worker identity was missing.
    #[error("Infrastructure owner actor and worker identities must be non-nil")]
    InvalidOwnerIdentity,
    /// Tenant or Inbox identity was empty.
    #[error("Infrastructure Inbox identity is invalid")]
    InvalidInboxIdentity,
    /// A tenant-scoped reservation lifecycle request used an empty tenant identity.
    #[error("Infrastructure tenant identity is invalid")]
    InvalidTenantIdentity,
    /// A lifecycle scan requested no items or exceeded its hard memory/work bound.
    #[error("Infrastructure reservation expiry batch must be between 1 and 64")]
    InvalidExpiryBatchSize,
    /// The requested tenant-scoped Inbox row does not exist.
    #[error("Infrastructure demand Inbox row was not found")]
    InboxNotFound,
    /// A previously listed reservation disappeared before its tenant-scoped transaction.
    #[error("Infrastructure reservation bundle was not found")]
    ReservationNotFound,
    /// Tenant and global physical reservation facts do not describe the same held pair.
    #[error("Infrastructure tenant and physical reservation ledgers are inconsistent")]
    ReservationPairInconsistent,
    /// The accepted immutable owner event needed to preserve the original correlation is absent.
    #[error("accepted Infrastructure Outbox event is missing for the reservation")]
    MissingAcceptedOutbox,
    /// The persisted Inbox payload is not a valid typed V2 demand.
    #[error("Infrastructure Inbox demand payload is invalid")]
    InvalidDemandPayload,
    /// The persisted payload and its indexed columns or digest disagree.
    #[error("Infrastructure Inbox payload does not match its immutable columns")]
    InboxPayloadMismatch,
    /// The Inbox state is inconsistent with its lease fields or cannot be admitted.
    #[error("Infrastructure Inbox state is invalid")]
    InvalidInboxState,
    /// A terminal Inbox row does not have the immutable receipt required by its state.
    #[error("terminal Infrastructure Inbox row has no matching receipt")]
    MissingTerminalReceipt,
    /// A receipt cannot be replayed because the pair of reservation identities is incomplete.
    #[error("accepted Infrastructure receipt has no matching reservation bundle")]
    MissingReservationBundle,
    /// Receipt identities or state conflict with the Inbox item.
    #[error("Infrastructure terminal receipt is inconsistent")]
    InvalidTerminalReceipt,
    /// An event or occurrence idempotency identity has conflicting terminal data.
    #[error("Infrastructure admission idempotency identity conflicts")]
    IdempotencyConflict,
    /// The persisted resource total is outside the supported non-negative range.
    #[error("Infrastructure reservation ledger has an invalid capacity total")]
    InvalidStoredCapacity,
    /// A trusted profile request cannot be stored in PostgreSQL's bounded integer units.
    #[error("Infrastructure capacity request exceeds the supported persistence range")]
    InvalidCapacityRequest,
    /// A persisted provider profile contains a discriminator or resource value the owner cannot decode.
    #[error("Infrastructure provider-profile revision contains invalid persisted values")]
    InvalidProviderProfileRecord,
    /// Attempt counter can no longer be increased without overflow.
    #[error("Infrastructure admission attempt counter overflow")]
    AttemptCounterOverflow,
    /// Serialization failed while producing a bounded event payload or digest.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// SQLx/PostgreSQL rejected the owner transaction or a required role grant.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
