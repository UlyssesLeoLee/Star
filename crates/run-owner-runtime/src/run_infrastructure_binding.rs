//! Run-owner persistence for immutable, versioned Run infrastructure bindings.
/*
@cypher schema=1 source_sha256=9418fa4f3018d1951ea51c5c3b68b7c880fa723ff39662dfce291ba7d1956063
MERGE (self:File {path:"crates/run-owner-runtime/src/run_infrastructure_binding.rs"})
MERGE (scope:Type {id:"run_owner_runtime::RunInfrastructureBindingScope",kind:"struct"})
MERGE (resolved:Type {id:"run_owner_runtime::ResolvedRunInfrastructureBinding",kind:"struct"})
MERGE (store:Type {id:"run_owner_runtime::PostgresRunInfrastructureBindingStore",kind:"struct"})
MERGE (store_error:Type {id:"run_owner_runtime::RunInfrastructureBindingStoreError",kind:"enum"})
MERGE (scope_validate:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::RunInfrastructureBindingScope.validate",kind:"method"})
MERGE (store_new:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::PostgresRunInfrastructureBindingStore.new",kind:"method"})
MERGE (save_revision:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::PostgresRunInfrastructureBindingStore.save_revision",kind:"method"})
MERGE (resolve_current:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::PostgresRunInfrastructureBindingStore.resolve_current",kind:"method"})
MERGE (set_actor_scope:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::set_run_binding_actor_scope",kind:"function"})
MERGE (validate_record:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::validate_persisted_binding",kind:"function"})
MERGE (unit_tests:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::tests",kind:"module"})
MERGE (fixture:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::tests::valid_binding",kind:"function"})
MERGE (valid_record_test:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::tests::persisted_record_requires_matching_scope_revision_and_digest",kind:"test"})
MERGE (invalid_record_test:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::tests::persisted_record_rejects_scope_and_digest_mismatch",kind:"test"})
MERGE (binding_type:Type {id:"domain_infrastructure::RunInfrastructureBindingSnapshotV2",kind:"external_type"})
MERGE (pool_type:Type {id:"sqlx::PgPool",kind:"external_type"})
MERGE (transaction_type:Type {id:"sqlx::Transaction<Postgres>",kind:"external_type"})
MERGE (binding_master:Table {id:"multica.run_infrastructure_binding_revision",class:"Master"})
MERGE (binding_event:Table {id:"multica.run_infrastructure_binding_event",class:"Transaction"})
MERGE (owner_role:DatabaseRole {id:"star_multica_run_owner"})
MERGE (self)-[:DEFINES]->(scope)
MERGE (self)-[:DEFINES]->(resolved)
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(store_error)
MERGE (self)-[:DEFINES]->(scope_validate)
MERGE (self)-[:DEFINES]->(store_new)
MERGE (self)-[:DEFINES]->(save_revision)
MERGE (self)-[:DEFINES]->(resolve_current)
MERGE (self)-[:DEFINES]->(set_actor_scope)
MERGE (self)-[:DEFINES]->(validate_record)
MERGE (self)-[:DEFINES]->(unit_tests)
MERGE (unit_tests)-[:DEFINES]->(fixture)
MERGE (unit_tests)-[:DEFINES]->(valid_record_test)
MERGE (unit_tests)-[:DEFINES]->(invalid_record_test)
MERGE (store)-[:USES_TYPE]->(pool_type)
MERGE (store)-[:USES_TYPE]->(binding_type)
MERGE (scope)-[:USES_TYPE]->(binding_type)
MERGE (resolved)-[:USES_TYPE]->(binding_type)
MERGE (save_revision)-[:CALLS]->(scope_validate)
MERGE (save_revision)-[:CALLS]->(set_actor_scope)
MERGE (save_revision)-[:CALLS]->(binding_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2.validate",kind:"method"})
MERGE (save_revision)-[:CALLS]->(binding_digest:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2.digest",kind:"method"})
MERGE (save_revision)-[:WRITES]->(binding_master)
MERGE (save_revision)-[:WRITES]->(binding_event)
MERGE (save_revision)-[:USES_TYPE]->(transaction_type)
MERGE (resolve_current)-[:CALLS]->(scope_validate)
MERGE (resolve_current)-[:CALLS]->(set_actor_scope)
MERGE (resolve_current)-[:CALLS]->(validate_record)
MERGE (resolve_current)-[:READS]->(binding_master)
MERGE (resolve_current)-[:USES_TYPE]->(transaction_type)
MERGE (validate_record)-[:CALLS]->(binding_validate)
MERGE (validate_record)-[:CALLS]->(binding_digest)
MERGE (valid_record_test)-[:CALLS]->(fixture)
MERGE (valid_record_test)-[:CALLS]->(binding_digest)
MERGE (valid_record_test)-[:CALLS]->(validate_record)
MERGE (invalid_record_test)-[:CALLS]->(fixture)
MERGE (invalid_record_test)-[:CALLS]->(binding_digest)
MERGE (invalid_record_test)-[:CALLS]->(validate_record)
MERGE (store)-[:OPERATES_AS]->(owner_role)
@endcypher
*/

use domain_infrastructure::{InfrastructureContractError, RunInfrastructureBindingSnapshotV2};
use sqlx::{types::Json, PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

/// The canonical tenant/Project/Branch/Engineering Run identity for a binding lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunInfrastructureBindingScope {
    /// Tenant that owns the Project and Engineering Run.
    pub tenant_id: Uuid,
    /// Project containing the selected Branch and Run.
    pub project_id: Uuid,
    /// Cloud Branch bound to the Engineering Run.
    pub branch_id: Uuid,
    /// Canonical Engineering Run identity.
    pub engineering_run_id: Uuid,
}

impl RunInfrastructureBindingScope {
    fn validate(self) -> Result<(), RunInfrastructureBindingStoreError> {
        if [
            self.tenant_id,
            self.project_id,
            self.branch_id,
            self.engineering_run_id,
        ]
        .iter()
        .any(Uuid::is_nil)
        {
            return Err(RunInfrastructureBindingStoreError::InvalidScope);
        }
        Ok(())
    }
}

/// A current persisted binding and the recomputed canonical digest sent to other owners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRunInfrastructureBinding {
    /// Validated binding snapshot returned by the Run owner.
    pub snapshot: RunInfrastructureBindingSnapshotV2,
    /// Recomputed canonical digest for downstream owner commands.
    pub digest: [u8; 32],
}

/// Least-privilege PostgreSQL adapter for the Run owner's binding Master.
#[derive(Clone)]
pub struct PostgresRunInfrastructureBindingStore {
    pool: PgPool,
}

#[derive(Debug, Error)]
/// Errors returned while validating or persisting a Run-owned binding revision.
pub enum RunInfrastructureBindingStoreError {
    /// PostgreSQL operation failed.
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    /// Snapshot violates the shared Infrastructure binding contract.
    #[error("binding contract is invalid: {0}")]
    Contract(#[from] InfrastructureContractError),
    /// Snapshot could not be serialized for persistence.
    #[error("binding snapshot serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    /// One or more canonical scope identifiers are nil.
    #[error("binding scope contains a nil identity")]
    InvalidScope,
    /// The actor identity is nil for a revision write.
    #[error("binding actor identity is nil")]
    InvalidActor,
    /// The snapshot does not belong to the requested Run scope.
    #[error("binding snapshot scope does not match the requested scope")]
    ScopeMismatch,
    /// The expected current revision does not match the stored revision.
    #[error("binding revision is stale or conflicts with the current revision")]
    RevisionConflict,
    /// No current binding revision exists for the requested Run.
    #[error("no current infrastructure binding exists for this Run")]
    NotFound,
    /// Stored snapshot, revision, or digest failed integrity validation.
    #[error("stored binding snapshot, revision, or digest is inconsistent")]
    InvalidStoredBinding,
}

impl PostgresRunInfrastructureBindingStore {
    /// Create a store over the application pool; all operations activate the restricted Run role.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Atomically close the expected current revision and append its successor.
    ///
    /// `expected_current_revision = None` is valid only for the first revision (1). Later
    /// revisions keep the stable binding ID and increment the revision by exactly one.
    pub async fn save_revision(
        &self,
        actor_id: Uuid,
        expected_current_revision: Option<i64>,
        snapshot: &RunInfrastructureBindingSnapshotV2,
    ) -> Result<(), RunInfrastructureBindingStoreError> {
        if actor_id.is_nil() {
            return Err(RunInfrastructureBindingStoreError::InvalidActor);
        }
        snapshot.validate()?;
        let scope = RunInfrastructureBindingScope {
            tenant_id: snapshot.tenant_id,
            project_id: snapshot.project_id,
            branch_id: snapshot.branch_id,
            engineering_run_id: snapshot.engineering_run_id,
        };
        scope.validate()?;
        let binding_digest = snapshot.digest()?;
        let schema_version = i16::try_from(snapshot.schema_version)
            .map_err(|_| RunInfrastructureBindingStoreError::InvalidStoredBinding)?;
        let serialized = serde_json::to_value(snapshot)?;

        let mut transaction = self.pool.begin().await?;
        set_run_binding_actor_scope(&mut transaction, scope.tenant_id, actor_id).await?;
        let current = sqlx::query(
            "SELECT binding_id, binding_revision \
             FROM multica.run_infrastructure_binding_revision \
             WHERE tenant_id = $1 AND project_id = $2 AND branch_id = $3 \
               AND engineering_run_id = $4 AND valid_to IS NULL FOR UPDATE",
        )
        .bind(scope.tenant_id)
        .bind(scope.project_id)
        .bind(scope.branch_id)
        .bind(scope.engineering_run_id)
        .fetch_optional(&mut *transaction)
        .await?;

        match (current, expected_current_revision) {
            (None, None) if snapshot.binding_revision == 1 => {}
            (Some(row), Some(expected_revision)) => {
                let current_id: Uuid = row.try_get("binding_id")?;
                let current_revision: i64 = row.try_get("binding_revision")?;
                if current_revision != expected_revision
                    || current_id != snapshot.binding_id
                    || current_revision.checked_add(1) != Some(snapshot.binding_revision)
                {
                    return Err(RunInfrastructureBindingStoreError::RevisionConflict);
                }
                let closed = sqlx::query(
                    "UPDATE multica.run_infrastructure_binding_revision \
                     SET valid_to = transaction_timestamp() \
                     WHERE tenant_id = $1 AND binding_id = $2 AND binding_revision = $3 \
                       AND valid_to IS NULL",
                )
                .bind(scope.tenant_id)
                .bind(current_id)
                .bind(current_revision)
                .execute(&mut *transaction)
                .await?;
                if closed.rows_affected() != 1 {
                    return Err(RunInfrastructureBindingStoreError::RevisionConflict);
                }
            }
            _ => return Err(RunInfrastructureBindingStoreError::RevisionConflict),
        }

        sqlx::query(
            "INSERT INTO multica.run_infrastructure_binding_revision (\
                 tenant_id, project_id, branch_id, engineering_run_id, binding_id, \
                 binding_revision, schema_version, binding_snapshot, binding_digest, changed_by\
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(scope.tenant_id)
        .bind(scope.project_id)
        .bind(scope.branch_id)
        .bind(scope.engineering_run_id)
        .bind(snapshot.binding_id)
        .bind(snapshot.binding_revision)
        .bind(schema_version)
        .bind(Json(serialized))
        .bind(binding_digest.to_vec())
        .bind(actor_id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    /// Resolve the current binding for a canonical Run scope and verify its stored digest.
    pub async fn resolve_current(
        &self,
        scope: RunInfrastructureBindingScope,
    ) -> Result<ResolvedRunInfrastructureBinding, RunInfrastructureBindingStoreError> {
        scope.validate()?;
        let mut transaction = self.pool.begin().await?;
        set_run_binding_actor_scope(&mut transaction, scope.tenant_id, Uuid::nil()).await?;
        let row = sqlx::query(
            "SELECT binding_id, binding_revision, binding_snapshot, binding_digest \
             FROM multica.run_infrastructure_binding_revision \
             WHERE tenant_id = $1 AND project_id = $2 AND branch_id = $3 \
               AND engineering_run_id = $4 AND valid_to IS NULL",
        )
        .bind(scope.tenant_id)
        .bind(scope.project_id)
        .bind(scope.branch_id)
        .bind(scope.engineering_run_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(RunInfrastructureBindingStoreError::NotFound)?;

        let binding_id: Uuid = row.try_get("binding_id")?;
        let binding_revision: i64 = row.try_get("binding_revision")?;
        let snapshot: Json<RunInfrastructureBindingSnapshotV2> = row.try_get("binding_snapshot")?;
        let persisted_digest: Vec<u8> = row.try_get("binding_digest")?;
        let digest = validate_persisted_binding(
            scope,
            binding_id,
            binding_revision,
            &persisted_digest,
            &snapshot.0,
        )?;
        transaction.commit().await?;
        Ok(ResolvedRunInfrastructureBinding {
            snapshot: snapshot.0,
            digest,
        })
    }
}

async fn set_run_binding_actor_scope(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut **transaction)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **transaction)
        .await?;
    if !actor_id.is_nil() {
        sqlx::query("SELECT set_config('app.actor_id', $1, true)")
            .bind(actor_id.to_string())
            .execute(&mut **transaction)
            .await?;
    }
    Ok(())
}

fn validate_persisted_binding(
    scope: RunInfrastructureBindingScope,
    binding_id: Uuid,
    binding_revision: i64,
    persisted_digest: &[u8],
    snapshot: &RunInfrastructureBindingSnapshotV2,
) -> Result<[u8; 32], RunInfrastructureBindingStoreError> {
    if persisted_digest.len() != 32
        || snapshot.tenant_id != scope.tenant_id
        || snapshot.project_id != scope.project_id
        || snapshot.branch_id != scope.branch_id
        || snapshot.engineering_run_id != scope.engineering_run_id
        || snapshot.binding_id != binding_id
        || snapshot.binding_revision != binding_revision
    {
        return Err(RunInfrastructureBindingStoreError::InvalidStoredBinding);
    }
    snapshot.validate()?;
    let digest = snapshot.digest()?;
    if persisted_digest != digest {
        return Err(RunInfrastructureBindingStoreError::InvalidStoredBinding);
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::{
        validate_persisted_binding, RunInfrastructureBindingScope,
        RunInfrastructureBindingStoreError,
    };
    use domain_infrastructure::{
        ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, KubernetesDnsLabel, ProviderKind,
        RunInfrastructureBindingSnapshotV2,
    };
    use uuid::Uuid;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn valid_binding() -> RunInfrastructureBindingSnapshotV2 {
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
            provider_profile_digest: [9; 32],
            namespace: KubernetesDnsLabel::new("run-1").unwrap(),
            service_account: KubernetesDnsLabel::new("agent-1").unwrap(),
            runtime_profile_id: id(10),
            runtime_profile_revision: 1,
            runtime_profile_digest: [11; 32],
            resource_budget: ExecutionResourceBudgetV1 {
                max_rss_bytes: 64 * 1024 * 1024,
                max_cpu_ms: 1_000,
                max_runtime_ms: 30_000,
                max_child_processes: 1,
                max_parallel_tools: 1,
                max_provider_calls: 1,
                max_output_bytes: 4_096,
                max_event_buffer_bytes: 4_096,
            },
        }
    }

    #[test]
    fn persisted_record_requires_matching_scope_revision_and_digest() {
        let snapshot = valid_binding();
        let scope = RunInfrastructureBindingScope {
            tenant_id: snapshot.tenant_id,
            project_id: snapshot.project_id,
            branch_id: snapshot.branch_id,
            engineering_run_id: snapshot.engineering_run_id,
        };
        let digest = snapshot.digest().unwrap();
        assert_eq!(
            validate_persisted_binding(
                scope,
                snapshot.binding_id,
                snapshot.binding_revision,
                &digest,
                &snapshot,
            )
            .unwrap(),
            digest
        );
    }

    #[test]
    fn persisted_record_rejects_scope_and_digest_mismatch() {
        let snapshot = valid_binding();
        let scope = RunInfrastructureBindingScope {
            tenant_id: id(20),
            project_id: snapshot.project_id,
            branch_id: snapshot.branch_id,
            engineering_run_id: snapshot.engineering_run_id,
        };
        let digest = snapshot.digest().unwrap();
        assert!(matches!(
            validate_persisted_binding(
                scope,
                snapshot.binding_id,
                snapshot.binding_revision,
                &digest,
                &snapshot,
            ),
            Err(RunInfrastructureBindingStoreError::InvalidStoredBinding)
        ));
        let scope = RunInfrastructureBindingScope {
            tenant_id: snapshot.tenant_id,
            project_id: snapshot.project_id,
            branch_id: snapshot.branch_id,
            engineering_run_id: snapshot.engineering_run_id,
        };
        let wrong_digest = [0; 32];
        assert!(matches!(
            validate_persisted_binding(
                scope,
                snapshot.binding_id,
                snapshot.binding_revision,
                &wrong_digest,
                &snapshot,
            ),
            Err(RunInfrastructureBindingStoreError::InvalidStoredBinding)
        ));

        let digest = snapshot.digest().unwrap();
        assert!(matches!(
            validate_persisted_binding(
                scope,
                id(21),
                snapshot.binding_revision,
                &digest,
                &snapshot,
            ),
            Err(RunInfrastructureBindingStoreError::InvalidStoredBinding)
        ));
        assert!(matches!(
            validate_persisted_binding(
                scope,
                snapshot.binding_id,
                snapshot.binding_revision + 1,
                &digest,
                &snapshot,
            ),
            Err(RunInfrastructureBindingStoreError::InvalidStoredBinding)
        ));

        let mut malformed = serde_json::to_value(&snapshot).unwrap();
        malformed["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<RunInfrastructureBindingSnapshotV2>(malformed).is_err());
    }
}
