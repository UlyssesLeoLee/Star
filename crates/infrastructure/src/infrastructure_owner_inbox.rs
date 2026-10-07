//! @cypher schema=1 source_sha256=b5fb2f8e1e9a8fc11a0527696b12b8cf6aca3c1a3099236ca9725ffe49674292
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_inbox.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::module",kind:"module"})
//! MERGE (binding:Type {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::CurrentInfrastructureBindingV2",kind:"struct"})
//! MERGE (outcome:Type {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::InboxReceiveOutcome",kind:"enum"})
//! MERGE (terminal:Type {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::TerminalDemandOutcome",kind:"enum"})
//! MERGE (store:Type {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::PostgresInfrastructureOwnerInbox",kind:"struct"})
//! MERGE (error:Type {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::InfrastructureOwnerInboxError",kind:"enum"})
//! MERGE (receive:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::PostgresInfrastructureOwnerInbox.receive_demand",kind:"method"})
//! MERGE (verify:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::verify_current_binding",kind:"function"})
//! MERGE (payload:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::payload_digest",kind:"function"})
//! MERGE (scope:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::set_tenant_scope",kind:"function"})
//! MERGE (provider:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::provider_kind_value",kind:"function"})
//! MERGE (placement:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::placement_policy_value",kind:"function"})
//! MERGE (tests:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::tests",kind:"module"})
//! MERGE (match_test:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::tests::current_binding_must_match_and_host_generation_must_be_positive",kind:"test"})
//! MERGE (digest_test:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::tests::payload_digest_is_stable",kind:"test"})
//! MERGE (pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (demand:Type {id:"domain_infrastructure::HostInfrastructureDemandV2",kind:"external_type"})
//! MERGE (binding_snapshot:Type {id:"domain_infrastructure::RunInfrastructureBindingSnapshotV2",kind:"external_type"})
//! MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
//! MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(binding)
//! MERGE (module)-[:DEFINES]->(outcome)
//! MERGE (module)-[:DEFINES]->(terminal)
//! MERGE (module)-[:DEFINES]->(store)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (module)-[:DEFINES]->(verify)
//! MERGE (module)-[:DEFINES]->(payload)
//! MERGE (module)-[:DEFINES]->(scope)
//! MERGE (module)-[:DEFINES]->(provider)
//! MERGE (module)-[:DEFINES]->(placement)
//! MERGE (module)-[:DEFINES]->(tests)
//! MERGE (tests)-[:DEFINES]->(match_test)
//! MERGE (tests)-[:DEFINES]->(digest_test)
//! MERGE (store)-[:USES_TYPE]->(pool)
//! MERGE (store)-[:USES_TYPE]->(demand)
//! MERGE (store)-[:USES_TYPE]->(binding_snapshot)
//! MERGE (receive)-[:CALLS]->(verify)
//! MERGE (receive)-[:CALLS]->(payload)
//! MERGE (receive)-[:CALLS]->(scope)
//! MERGE (receive)-[:CALLS]->(provider)
//! MERGE (receive)-[:CALLS]->(placement)
//! MERGE (receive)-[:WRITES]->(inbox)
//! MERGE (receive)-[:READS]->(receipt)
//! MERGE (match_test)-[:CALLS]->(verify)
//! MERGE (digest_test)-[:CALLS]->(payload)
//! @endcypher

//! Tenant-scoped, idempotent receipt of Run-owner capacity demands.
//!
//! This adapter only persists a validated demand in the Infrastructure Inbox. It does not
//! acknowledge capacity, create reservations, call Multipass/K3s, or enable task execution.

use chrono::Utc;
use domain_infrastructure::{
    ClusterPlacementPolicyV1, HostInfrastructureDemandV2, ProviderKind,
    RunInfrastructureBindingSnapshotV2,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use uuid::Uuid;

/// Current owner-resolved binding identity and Host inventory generation.
///
/// The caller must obtain these values from trusted Infrastructure-owned state, never from the
/// incoming event. The exact binding is compared before the Inbox transaction begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentInfrastructureBindingV2 {
    /// Binding read from the current Infrastructure owner source of truth.
    pub binding: RunInfrastructureBindingSnapshotV2,
    /// Current provider-owned generation of the referenced HostEnvironment.
    pub host_generation: i64,
}

/// Result of receiving a demand; none of these outcomes imply that capacity was admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxReceiveOutcome {
    /// A new demand was committed to the nonterminal Inbox.
    Enqueued {
        /// Persisted Inbox row identity.
        inbox_id: Uuid,
    },
    /// The same event and occurrence were already present with identical content.
    DuplicatePending {
        /// Existing Inbox row identity.
        inbox_id: Uuid,
    },
    /// An immutable terminal receipt already exists for this exact demand.
    AlreadyTerminal {
        /// Immutable receipt outcome.
        outcome: TerminalDemandOutcome,
    },
}

/// Immutable terminal outcome already recorded by the Infrastructure owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalDemandOutcome {
    /// The owner previously accepted and reserved the demand.
    Accepted,
    /// The owner previously rejected the demand.
    Rejected,
}

/// PostgreSQL store for the Infrastructure-owner demand Inbox.
#[derive(Debug, Clone)]
pub struct PostgresInfrastructureOwnerInbox {
    pool: PgPool,
}

impl PostgresInfrastructureOwnerInbox {
    /// Construct the store over the application's PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Validate current binding identity and persist a demand idempotently.
    ///
    /// The call uses the restricted NOLOGIN Infrastructure owner role and tenant RLS inside one
    /// transaction. The application login must be explicitly granted membership by deployment.
    pub async fn receive_demand(
        &self,
        demand: &HostInfrastructureDemandV2,
        current: &CurrentInfrastructureBindingV2,
    ) -> Result<InboxReceiveOutcome, InfrastructureOwnerInboxError> {
        let binding_digest = verify_current_binding(demand, current)?;
        if demand.deadline_at <= Utc::now() {
            return Err(InfrastructureOwnerInboxError::DeadlineExpired);
        }
        let (payload, payload_digest) = payload_digest(demand)?;

        let mut tx = self.pool.begin().await?;
        sqlx::query("SET LOCAL ROLE star_infrastructure_owner")
            .execute(&mut *tx)
            .await?;
        set_tenant_scope(&mut tx, demand.tenant_id).await?;

        if let Some(outcome) = read_terminal_receipt(&mut tx, demand, &payload_digest).await? {
            tx.commit().await?;
            return Ok(InboxReceiveOutcome::AlreadyTerminal { outcome });
        }

        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"INSERT INTO infrastructure.capacity_demand_inbox
                (tenant_id, source_event_id, occurrence_id, project_id, branch_id,
                 engineering_run_id, task_execution_run_id, schedule_rule_id,
                 schedule_rule_revision, binding_id, binding_revision, binding_digest,
                 host_environment_id, host_generation, k3s_cluster_id,
                 k3s_cluster_generation, provider_kind, placement_policy, payload,
                 payload_digest, deadline_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
               ON CONFLICT DO NOTHING
               RETURNING inbox_id"#,
        )
        .bind(demand.tenant_id)
        .bind(demand.event_id)
        .bind(demand.occurrence_id)
        .bind(demand.project_id)
        .bind(demand.branch_id)
        .bind(demand.engineering_run_id)
        .bind(demand.task_execution_run_id)
        .bind(demand.schedule_rule_id)
        .bind(demand.schedule_rule_version)
        .bind(demand.binding.binding_id)
        .bind(demand.binding.binding_revision)
        .bind(binding_digest.as_slice())
        .bind(demand.binding.host_environment_id)
        .bind(current.host_generation)
        .bind(demand.binding.k3s_cluster_id)
        .bind(demand.binding.k3s_cluster_generation)
        .bind(provider_kind_value(demand.binding.provider))
        .bind(placement_policy_value(
            demand.binding.cluster_placement_policy,
        ))
        .bind(&payload)
        .bind(payload_digest.as_slice())
        .bind(demand.deadline_at)
        .fetch_optional(&mut *tx)
        .await?;

        let outcome = if let Some((inbox_id,)) = inserted {
            InboxReceiveOutcome::Enqueued { inbox_id }
        } else if let Some(outcome) =
            read_terminal_receipt(&mut tx, demand, &payload_digest).await?
        {
            InboxReceiveOutcome::AlreadyTerminal { outcome }
        } else {
            read_pending_duplicate(&mut tx, demand, &payload, &payload_digest).await?
        };
        tx.commit().await?;
        Ok(outcome)
    }
}

/// Verify that the current Infrastructure owner resolver returned the exact embedded binding.
fn verify_current_binding(
    demand: &HostInfrastructureDemandV2,
    current: &CurrentInfrastructureBindingV2,
) -> Result<[u8; 32], InfrastructureOwnerInboxError> {
    demand
        .validate()
        .map_err(|_| InfrastructureOwnerInboxError::InvalidDemand)?;
    current
        .binding
        .validate()
        .map_err(|_| InfrastructureOwnerInboxError::InvalidDemand)?;
    if current.host_generation <= 0 {
        return Err(InfrastructureOwnerInboxError::InvalidHostGeneration);
    }
    let event_digest = demand
        .binding
        .digest()
        .map_err(|_| InfrastructureOwnerInboxError::InvalidDemand)?;
    let current_digest = current
        .binding
        .digest()
        .map_err(|_| InfrastructureOwnerInboxError::InvalidDemand)?;
    if event_digest != current_digest {
        return Err(InfrastructureOwnerInboxError::BindingChanged);
    }
    Ok(current_digest)
}

/// Serialize the typed event as canonical JSON and bind it to its wire-contract version.
fn payload_digest(
    demand: &HostInfrastructureDemandV2,
) -> Result<(Value, [u8; 32]), InfrastructureOwnerInboxError> {
    let payload = serde_json::to_value(demand)?;
    let bytes = serde_json::to_vec(&payload)?;
    let mut hasher = Sha256::new();
    hasher.update(b"star.host-infrastructure-demand.v2\0");
    hasher.update(bytes);
    Ok((payload, hasher.finalize().into()))
}

async fn read_terminal_receipt(
    tx: &mut Transaction<'_, Postgres>,
    demand: &HostInfrastructureDemandV2,
    payload_digest: &[u8; 32],
) -> Result<Option<TerminalDemandOutcome>, InfrastructureOwnerInboxError> {
    let rows: Vec<(Uuid, Uuid, Vec<u8>, String)> = sqlx::query_as(
        "SELECT source_event_id, occurrence_id, payload_digest, outcome \
         FROM infrastructure.capacity_demand_receipt \
         WHERE tenant_id = $1 AND (source_event_id = $2 OR occurrence_id = $3) \
         ORDER BY receipt_id",
    )
    .bind(demand.tenant_id)
    .bind(demand.event_id)
    .bind(demand.occurrence_id)
    .fetch_all(&mut **tx)
    .await?;
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != 1 {
        return Err(InfrastructureOwnerInboxError::IdempotencyConflict);
    }
    let (source_event_id, occurrence_id, existing_digest, outcome) = &rows[0];
    if *source_event_id != demand.event_id
        || *occurrence_id != demand.occurrence_id
        || existing_digest.as_slice() != payload_digest
    {
        return Err(InfrastructureOwnerInboxError::IdempotencyConflict);
    }
    let outcome = match outcome.as_str() {
        "accepted" => TerminalDemandOutcome::Accepted,
        "rejected" => TerminalDemandOutcome::Rejected,
        _ => return Err(InfrastructureOwnerInboxError::IdempotencyConflict),
    };
    Ok(Some(outcome))
}

async fn read_pending_duplicate(
    tx: &mut Transaction<'_, Postgres>,
    demand: &HostInfrastructureDemandV2,
    payload: &Value,
    payload_digest: &[u8; 32],
) -> Result<InboxReceiveOutcome, InfrastructureOwnerInboxError> {
    let rows: Vec<(Uuid, Uuid, Uuid, Vec<u8>, Value, String)> = sqlx::query_as(
        "SELECT inbox_id, source_event_id, occurrence_id, payload_digest, payload, processing_state \
         FROM infrastructure.capacity_demand_inbox \
         WHERE tenant_id = $1 AND (source_event_id = $2 OR occurrence_id = $3) \
         ORDER BY inbox_id FOR UPDATE",
    )
    .bind(demand.tenant_id)
    .bind(demand.event_id)
    .bind(demand.occurrence_id)
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() != 1 {
        return Err(InfrastructureOwnerInboxError::IdempotencyConflict);
    }
    let (inbox_id, source_event_id, occurrence_id, existing_digest, existing_payload, state) =
        &rows[0];
    if *source_event_id != demand.event_id
        || *occurrence_id != demand.occurrence_id
        || existing_digest.as_slice() != payload_digest
        || existing_payload != payload
    {
        return Err(InfrastructureOwnerInboxError::IdempotencyConflict);
    }
    match state.as_str() {
        "received" | "retry" | "leased" => Ok(InboxReceiveOutcome::DuplicatePending {
            inbox_id: *inbox_id,
        }),
        "accepted" | "rejected" => Err(InfrastructureOwnerInboxError::MissingTerminalReceipt),
        _ => Err(InfrastructureOwnerInboxError::IdempotencyConflict),
    }
}

async fn set_tenant_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn provider_kind_value(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::MultipassK3sCluster => "multipass_k3s_cluster",
        ProviderKind::RemoteK3s => "remote_k3s",
    }
}

fn placement_policy_value(policy: ClusterPlacementPolicyV1) -> &'static str {
    match policy {
        ClusterPlacementPolicyV1::SharedPool => "shared_pool",
        ClusterPlacementPolicyV1::RunDedicated => "run_dedicated",
        ClusterPlacementPolicyV1::PinnedCluster => "pinned_cluster",
    }
}

/// Errors from the Infrastructure Inbox receive operation.
#[derive(Debug, Error)]
pub enum InfrastructureOwnerInboxError {
    /// The demand does not satisfy the versioned domain contract.
    #[error("invalid Host infrastructure demand")]
    InvalidDemand,
    /// The binding embedded in the event is no longer the current binding.
    #[error("current Run infrastructure binding changed before Inbox receipt")]
    BindingChanged,
    /// The trusted Host inventory has no positive generation.
    #[error("Host inventory generation is unknown or invalid")]
    InvalidHostGeneration,
    /// The demand deadline elapsed before the Inbox transaction began.
    #[error("capacity demand deadline expired before Inbox receipt")]
    DeadlineExpired,
    /// An event or occurrence idempotency key was reused with different content.
    #[error("Inbox idempotency key was reused with different event content")]
    IdempotencyConflict,
    /// A terminal inbox row exists without its required immutable receipt.
    #[error("terminal Inbox state has no immutable receipt")]
    MissingTerminalReceipt,
    /// The typed demand could not be serialized as JSON.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// PostgreSQL rejected the transaction or a required runtime role was unavailable.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::{payload_digest, verify_current_binding, CurrentInfrastructureBindingV2};
    use chrono::{Duration, Utc};
    use domain_infrastructure::{
        ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
        KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV2,
    };
    use uuid::Uuid;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn budget() -> ExecutionResourceBudgetV1 {
        ExecutionResourceBudgetV1 {
            max_rss_bytes: 512 * 1024 * 1024,
            max_cpu_ms: 60_000,
            max_runtime_ms: 600_000,
            max_child_processes: 8,
            max_parallel_tools: 4,
            max_provider_calls: 100,
            max_output_bytes: 64 * 1024,
            max_event_buffer_bytes: 16 * 1024,
        }
    }

    fn binding() -> RunInfrastructureBindingSnapshotV2 {
        RunInfrastructureBindingSnapshotV2 {
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
            provider_profile_digest: [8; 32],
            namespace: KubernetesDnsLabel::new("run-1").expect("valid label"),
            service_account: KubernetesDnsLabel::new("agent").expect("valid label"),
            runtime_profile_id: id(9),
            runtime_profile_revision: 1,
            runtime_profile_digest: [9; 32],
            resource_budget: budget(),
        }
    }

    fn demand(binding: RunInfrastructureBindingSnapshotV2) -> HostInfrastructureDemandV2 {
        let now = Utc::now();
        HostInfrastructureDemandV2 {
            schema_version: 2,
            event_id: id(10),
            correlation_id: id(11),
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            branch_id: binding.branch_id,
            engineering_run_id: binding.engineering_run_id,
            task_execution_run_id: id(12),
            schedule_rule_id: id(13),
            schedule_rule_version: 1,
            occurrence_id: id(14),
            resource_budget: binding.resource_budget,
            binding,
            requested_at: now,
            deadline_at: now + Duration::minutes(5),
        }
    }

    #[test]
    fn current_binding_must_match_and_host_generation_must_be_positive() {
        let demand = demand(binding());
        let current = CurrentInfrastructureBindingV2 {
            binding: demand.binding.clone(),
            host_generation: 1,
        };
        assert_eq!(
            verify_current_binding(&demand, &current).expect("current binding matches"),
            demand.binding.digest().expect("valid binding has a digest")
        );

        let mut changed = current.clone();
        changed.binding.provider_profile_revision += 1;
        assert!(verify_current_binding(&demand, &changed).is_err());

        let mut unknown_generation = current;
        unknown_generation.host_generation = 0;
        assert!(verify_current_binding(&demand, &unknown_generation).is_err());
    }

    #[test]
    fn payload_digest_is_stable_for_the_same_typed_event() {
        let demand = demand(binding());
        assert_eq!(
            payload_digest(&demand).unwrap(),
            payload_digest(&demand).unwrap()
        );
    }
}
