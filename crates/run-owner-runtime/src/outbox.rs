//! Transactional Run-owner capacity demand persistence.
/*
@cypher schema=1 source_sha256=e0e04b5cbc6b534ec11b109f278f92f4469a05730106f38f7a038bbc33c80a62
MERGE (self:File {path:"crates/run-owner-runtime/src/outbox.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::outbox",kind:"module"})
MERGE (store:Type {id:"run_owner_runtime::PostgresRunOwnerCapacityStore",kind:"struct"})
MERGE (outcome:Type {id:"run_owner_runtime::CapacityDemandAppendOutcome",kind:"enum"})
MERGE (error:Type {id:"run_owner_runtime::RunOwnerCapacityStoreError",kind:"enum"})
MERGE (new_store:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::PostgresRunOwnerCapacityStore.new",kind:"method"})
MERGE (append:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::PostgresRunOwnerCapacityStore.append_capacity_demand",kind:"method"})
MERGE (scope:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::set_run_owner_scope",kind:"function"})
MERGE (validate:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::validate_demand_scope",kind:"function"})
MERGE (unit_id:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::tests.id",kind:"function"})
MERGE (unit_demand:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::tests.demand",kind:"function"})
MERGE (unit_test:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::tests.demand_scope_requires_valid_contract_and_work_item_identity",kind:"test"})
MERGE (validate_run:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::validate_task_execution_run_scope",kind:"function"})
MERGE (demand:Type {id:"domain_infrastructure::HostInfrastructureDemandV2",kind:"external_type"})
MERGE (transaction:Type {id:"sqlx::Transaction<Postgres>",kind:"external_type"})
MERGE (outbox:Table {id:"multica.infrastructure_capacity_demand_outbox"})
MERGE (task_run:Table {id:"multica.task_execution_run"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(store)
MERGE (module)-[:DEFINES]->(outcome)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(scope)
MERGE (module)-[:DEFINES]->(validate)
MERGE (module)-[:DEFINES]->(unit_id)
MERGE (module)-[:DEFINES]->(unit_demand)
MERGE (module)-[:DEFINES]->(unit_test)
MERGE (module)-[:DEFINES]->(validate_run)
MERGE (store)-[:DEFINES]->(append)
MERGE (store)-[:DEFINES]->(new_store)
MERGE (append)-[:USES_TYPE]->(demand)
MERGE (append)-[:USES_TYPE]->(transaction)
MERGE (append)-[:CALLS]->(scope)
MERGE (append)-[:CALLS]->(validate)
MERGE (append)-[:CALLS]->(validate_run)
MERGE (append)-[:WRITES]->(outbox)
MERGE (validate_run)-[:READS]->(task_run)
MERGE (validate)-[:USES_TYPE]->(demand)
MERGE (unit_demand)-[:CALLS]->(unit_id)
MERGE (unit_test)-[:CALLS]->(validate)
MERGE (unit_test)-[:CALLS]->(unit_demand)
MERGE (unit_test)-[:TESTS]->(append)
@endcypher
*/

use domain_infrastructure::HostInfrastructureDemandV2;
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

const DEMAND_SCHEMA_VERSION: i16 = 2;
const DEMAND_KEY_PREFIX: &str = "capacity-demand:v2:";

/// Whether the caller-owned Run transaction inserted or replayed a demand event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityDemandAppendOutcome {
    /// A new immutable demand event was inserted into the caller's transaction.
    Inserted {
        /// Immutable event identity.
        event_id: Uuid,
    },
    /// An identical event with the same idempotency key already exists.
    Duplicate {
        /// Existing immutable event identity.
        event_id: Uuid,
    },
}

/// Run-owner database errors. No cross-owner Infrastructure or Schedule rows are written here.
#[derive(Debug, Error)]
pub enum RunOwnerCapacityStoreError {
    /// A Run demand does not satisfy its typed scope or idempotency contract.
    #[error("invalid Run-owner infrastructure capacity demand")]
    InvalidDemand,
    /// An immutable event ID or idempotency key was reused with different content or scope.
    #[error("Run-owner capacity demand idempotency conflict")]
    IdempotencyConflict,
    /// The Run-owner database operation failed.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    /// The versioned demand could not be serialized to its immutable Outbox value.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

/// PostgreSQL adapter for Run-owned demand Outbox and capacity-result Inbox facts.
#[derive(Debug, Clone)]
pub struct PostgresRunOwnerCapacityStore {
    pool: PgPool,
}

impl PostgresRunOwnerCapacityStore {
    /// Construct a Run-owner store over the application's PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Append a typed capacity demand using the caller's existing Run-owner transaction.
    ///
    /// The method does not begin, commit, or roll back the transaction. The caller can therefore
    /// atomically commit Run-owner facts and this Run-owner Outbox row; it cannot commit Schedule-
    /// owned occurrence/link/Outbox state. For Runs with preparation lineage, the database also
    /// requires a confirmation event, a live pending reservation and no compensation receipt.
    /// This primitive does not authenticate or consume Schedule confirmation and is not production
    /// admission authority. A scheduled caller must validate confirmation in its owner transaction.
    /// The stable key is tenant-scoped and occurrence-scoped; replay with changed content is a
    /// hard conflict.
    pub async fn append_capacity_demand(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        work_item_id: Uuid,
        demand: &HostInfrastructureDemandV2,
    ) -> Result<CapacityDemandAppendOutcome, RunOwnerCapacityStoreError> {
        validate_demand_scope(work_item_id, demand)?;
        let payload = serde_json::to_value(demand)?;
        let idempotency_key = format!("{DEMAND_KEY_PREFIX}{}", demand.occurrence_id);

        set_run_owner_scope(tx, demand.tenant_id).await?;
        validate_task_execution_run_scope(tx, work_item_id, demand).await?;
        let inserted: Option<(Uuid,)> = sqlx::query_as(
            r#"INSERT INTO multica.infrastructure_capacity_demand_outbox
                    (event_id, tenant_id, project_id, branch_id, engineering_run_id,
                     work_item_id, task_execution_run_id, occurrence_id, schema_version,
                     idempotency_key, correlation_id, payload, occurred_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
               ON CONFLICT DO NOTHING
               RETURNING event_id"#,
        )
        .bind(demand.event_id)
        .bind(demand.tenant_id)
        .bind(demand.project_id)
        .bind(demand.branch_id)
        .bind(demand.engineering_run_id)
        .bind(work_item_id)
        .bind(demand.task_execution_run_id)
        .bind(demand.occurrence_id)
        .bind(DEMAND_SCHEMA_VERSION)
        .bind(&idempotency_key)
        .bind(demand.correlation_id)
        .bind(&payload)
        .bind(demand.requested_at)
        .fetch_optional(&mut **tx)
        .await?;

        if inserted.is_some() {
            return Ok(CapacityDemandAppendOutcome::Inserted {
                event_id: demand.event_id,
            });
        }

        // This Outbox is append-only, and the owner role has no UPDATE privilege. A unique
        // constraint arbitrates concurrent appends; a subsequent SELECT compares the immutable
        // row (or fails closed if a stronger transaction snapshot cannot see it).
        let rows = sqlx::query(
            r#"SELECT event_id, project_id, branch_id, engineering_run_id, work_item_id,
                      task_execution_run_id, occurrence_id, schema_version, idempotency_key,
                      correlation_id, payload
               FROM multica.infrastructure_capacity_demand_outbox
               WHERE tenant_id = $1 AND (event_id = $2 OR idempotency_key = $3)
               ORDER BY event_id"#,
        )
        .bind(demand.tenant_id)
        .bind(demand.event_id)
        .bind(&idempotency_key)
        .fetch_all(&mut **tx)
        .await?;
        if rows.len() != 1 {
            return Err(RunOwnerCapacityStoreError::IdempotencyConflict);
        }
        let row = &rows[0];
        let existing_payload: Value = row.try_get("payload")?;
        let same = row.try_get::<Uuid, _>("event_id")? == demand.event_id
            && row.try_get::<Uuid, _>("project_id")? == demand.project_id
            && row.try_get::<Uuid, _>("branch_id")? == demand.branch_id
            && row.try_get::<Uuid, _>("engineering_run_id")? == demand.engineering_run_id
            && row.try_get::<Uuid, _>("work_item_id")? == work_item_id
            && row.try_get::<Uuid, _>("task_execution_run_id")? == demand.task_execution_run_id
            && row.try_get::<Uuid, _>("occurrence_id")? == demand.occurrence_id
            && row.try_get::<i16, _>("schema_version")? == DEMAND_SCHEMA_VERSION
            && row.try_get::<String, _>("idempotency_key")? == idempotency_key
            && row.try_get::<Uuid, _>("correlation_id")? == demand.correlation_id
            && existing_payload == payload;
        if !same {
            return Err(RunOwnerCapacityStoreError::IdempotencyConflict);
        }

        Ok(CapacityDemandAppendOutcome::Duplicate {
            event_id: demand.event_id,
        })
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }
}

fn validate_demand_scope(
    work_item_id: Uuid,
    demand: &HostInfrastructureDemandV2,
) -> Result<(), RunOwnerCapacityStoreError> {
    demand
        .validate()
        .map_err(|_| RunOwnerCapacityStoreError::InvalidDemand)?;
    if work_item_id.is_nil() {
        return Err(RunOwnerCapacityStoreError::InvalidDemand);
    }
    Ok(())
}

async fn validate_task_execution_run_scope(
    tx: &mut Transaction<'_, Postgres>,
    work_item_id: Uuid,
    demand: &HostInfrastructureDemandV2,
) -> Result<(), RunOwnerCapacityStoreError> {
    let row = sqlx::query(
        r#"SELECT run_origin, automation_rule_id, automation_rule_version,
                  automation_occurrence_id, correlation_id
           FROM multica.task_execution_run
           WHERE tenant_id = $1 AND project_id = $2 AND work_item_id = $3 AND run_id = $4"#,
    )
    .bind(demand.tenant_id)
    .bind(demand.project_id)
    .bind(work_item_id)
    .bind(demand.task_execution_run_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(RunOwnerCapacityStoreError::InvalidDemand)?;

    let matches = row.try_get::<String, _>("run_origin")? == "schedule"
        && row.try_get::<Option<Uuid>, _>("automation_rule_id")? == Some(demand.schedule_rule_id)
        && row.try_get::<Option<i64>, _>("automation_rule_version")?
            == Some(demand.schedule_rule_version)
        && row.try_get::<Option<Uuid>, _>("automation_occurrence_id")?
            == Some(demand.occurrence_id)
        && row.try_get::<Uuid, _>("correlation_id")? == demand.correlation_id;
    if !matches {
        return Err(RunOwnerCapacityStoreError::InvalidDemand);
    }
    Ok(())
}

pub(crate) async fn set_run_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_demand_scope, RunOwnerCapacityStoreError};
    use chrono::{TimeZone, Utc};
    use domain_infrastructure::{
        ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
        KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV2,
    };
    use uuid::Uuid;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn demand() -> HostInfrastructureDemandV2 {
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
            namespace: KubernetesDnsLabel::new("run-1").unwrap(),
            service_account: KubernetesDnsLabel::new("agent-1").unwrap(),
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

    #[test]
    fn demand_scope_requires_valid_contract_and_work_item_identity() {
        assert!(validate_demand_scope(id(17), &demand()).is_ok());
        assert!(matches!(
            validate_demand_scope(Uuid::nil(), &demand()),
            Err(RunOwnerCapacityStoreError::InvalidDemand)
        ));

        let mut mismatched = demand();
        mismatched.task_execution_run_id = Uuid::nil();
        assert!(matches!(
            validate_demand_scope(id(17), &mismatched),
            Err(RunOwnerCapacityStoreError::InvalidDemand)
        ));
    }
}
