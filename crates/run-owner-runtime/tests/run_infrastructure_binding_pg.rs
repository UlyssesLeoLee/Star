//! Real SQLx binding revisions against the disposable PostgreSQL runner.
/*
@cypher schema=1 source_sha256=d6570faa2de4aeae40f1b1938d7f030ce6efd2f3cd6af48ff1bda6f137e429f9
MERGE (self:File {path:"crates/run-owner-runtime/tests/run_infrastructure_binding_pg.rs"})
MERGE (test:Symbol {id:"crates/run-owner-runtime/tests/run_infrastructure_binding_pg.rs::binding_roundtrip_cas_and_rollback",kind:"test"})
MERGE (fixture:Symbol {id:"crates/run-owner-runtime/tests/run_infrastructure_binding_pg.rs::binding_fixture",kind:"function"})
MERGE (save:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::PostgresRunInfrastructureBindingStore.save_revision",kind:"method"})
MERGE (resolve:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::PostgresRunInfrastructureBindingStore.resolve_current",kind:"method"})
MERGE (master:Table {id:"multica.run_infrastructure_binding_revision"})
MERGE (events:Table {id:"multica.run_infrastructure_binding_event"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (test)-[:CALLS]->(fixture)
MERGE (test)-[:CALLS]->(save)
MERGE (test)-[:CALLS]->(resolve)
MERGE (test)-[:TESTS]->(master)
MERGE (test)-[:TESTS]->(events)
@endcypher
*/

use std::time::Duration;

use domain_infrastructure::{
    ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, KubernetesDnsLabel, ProviderKind,
    RunInfrastructureBindingSnapshotV2,
};
use run_owner_runtime::{
    PostgresRunInfrastructureBindingStore, RunInfrastructureBindingScope,
    RunInfrastructureBindingStoreError,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires phase9f4d_run_owner_consumer.py --schema-only --binding-tests"]
async fn binding_roundtrip_cas_and_rollback() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("STAR_RUN_OWNER_TEST_DATABASE_URL")?;
    let admin = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await?;
    // This login has no superuser/BYPASSRLS rights. Only the disposable admin seeds
    // canonical parents and installs a scoped fault; the adapter uses this login.
    sqlx::raw_sql(
        "CREATE ROLE star_binding_test_login LOGIN NOINHERIT NOSUPERUSER NOBYPASSRLS;
         GRANT star_multica_run_owner TO star_binding_test_login;
         ALTER ROLE star_binding_test_login SET statement_timeout = '10s';",
    )
    .execute(&admin)
    .await?;
    let options: PgConnectOptions = url.parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options.username("star_binding_test_login"))
        .await?;
    let store = PostgresRunInfrastructureBindingStore::new(pool.clone());
    let snapshot = binding_fixture();
    let actor = Uuid::new_v4();
    let scope = RunInfrastructureBindingScope {
        tenant_id: snapshot.tenant_id,
        project_id: snapshot.project_id,
        branch_id: snapshot.branch_id,
        engineering_run_id: snapshot.engineering_run_id,
    };
    let repository_id = Uuid::new_v4();
    let mut seed = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role = replica")
        .execute(&mut *seed)
        .await?;
    sqlx::query(
        "INSERT INTO scm.cloud_branch (branch_id, tenant_id, project_id, repository_id, remote_identity)
         VALUES ($1,$2,$3,$4,'test://binding-sqlx')",
    )
    .bind(scope.branch_id).bind(scope.tenant_id).bind(scope.project_id).bind(repository_id)
    .execute(&mut *seed).await?;
    sqlx::query(
        "INSERT INTO multica.engineering_run (engineering_run_id, tenant_id, project_id, repository_id, branch_id)
         VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(scope.engineering_run_id).bind(scope.tenant_id).bind(scope.project_id)
    .bind(repository_id).bind(scope.branch_id).execute(&mut *seed).await?;
    seed.commit().await?;

    assert!(matches!(
        store.resolve_current(scope).await,
        Err(RunInfrastructureBindingStoreError::NotFound)
    ));
    store.save_revision(actor, None, &snapshot).await?;
    let resolved = store.resolve_current(scope).await?;
    assert_eq!(resolved.snapshot, snapshot);
    assert_eq!(resolved.digest, snapshot.digest()?);
    assert!(matches!(
        store
            .resolve_current(RunInfrastructureBindingScope {
                tenant_id: Uuid::new_v4(),
                ..scope
            })
            .await,
        Err(RunInfrastructureBindingStoreError::NotFound)
    ));

    let mut successor = snapshot.clone();
    successor.binding_revision = 2;
    successor.k3s_cluster_generation = 2;
    assert!(matches!(
        store.save_revision(actor, Some(0), &successor).await,
        Err(RunInfrastructureBindingStoreError::RevisionConflict)
    ));
    // Both writers read the same expected revision; exactly one may commit.
    let (first, second) = tokio::join!(
        store.save_revision(actor, Some(1), &successor),
        store.save_revision(actor, Some(1), &successor)
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    let loser = if first.is_err() { first } else { second };
    assert!(matches!(
        loser,
        Err(RunInfrastructureBindingStoreError::RevisionConflict)
    ));
    assert_eq!(store.resolve_current(scope).await?.snapshot, successor);

    // Fail the INSERT after the old current revision was closed. The entire
    // transaction, including its superseded audit fact, must roll back.
    sqlx::raw_sql(&format!(
        "CREATE FUNCTION multica.binding_test_insert_fault() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN IF NEW.binding_id = '{}'::uuid AND NEW.binding_revision = 3 THEN
           RAISE EXCEPTION 'disposable binding insert fault'; END IF; RETURN NEW; END $$;
         CREATE TRIGGER binding_test_insert_fault BEFORE INSERT ON multica.run_infrastructure_binding_revision
         FOR EACH ROW EXECUTE FUNCTION multica.binding_test_insert_fault();", snapshot.binding_id
    )).execute(&admin).await?;
    let mut third = successor.clone();
    third.binding_revision = 3;
    assert!(matches!(
        store.save_revision(actor, Some(2), &third).await,
        Err(RunInfrastructureBindingStoreError::Database(_))
    ));
    assert_eq!(store.resolve_current(scope).await?.snapshot, successor);
    let revisions: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM multica.run_infrastructure_binding_revision WHERE binding_id = $1",
    )
    .bind(snapshot.binding_id)
    .fetch_one(&admin)
    .await?;
    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM multica.run_infrastructure_binding_event WHERE binding_id = $1",
    )
    .bind(snapshot.binding_id)
    .fetch_one(&admin)
    .await?;
    assert_eq!(revisions, 2);
    assert_eq!(event_count, 3);
    sqlx::raw_sql(
        "DROP TRIGGER binding_test_insert_fault ON multica.run_infrastructure_binding_revision;
         DROP FUNCTION multica.binding_test_insert_fault();",
    )
    .execute(&admin)
    .await?;
    store.save_revision(actor, Some(2), &third).await?;
    assert_eq!(store.resolve_current(scope).await?.snapshot, third);
    // An owner-writable row with a forged digest passes the SQL envelope checks
    // but must never be returned as a trusted binding by the Rust resolver.
    let mut forged = third.clone();
    forged.binding_revision = 4;
    let mut transaction = pool.begin().await?;
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut *transaction)
        .await?;
    sqlx::query(
        "SELECT set_config('app.tenant_id', $1, true), set_config('app.actor_id', $2, true)",
    )
    .bind(scope.tenant_id.to_string())
    .bind(actor.to_string())
    .execute(&mut *transaction)
    .await?;
    sqlx::query("UPDATE multica.run_infrastructure_binding_revision SET valid_to = transaction_timestamp() WHERE binding_id = $1 AND valid_to IS NULL")
        .bind(snapshot.binding_id).execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO multica.run_infrastructure_binding_revision
        (tenant_id,project_id,branch_id,engineering_run_id,binding_id,binding_revision,schema_version,binding_snapshot,binding_digest,changed_by)
        VALUES ($1,$2,$3,$4,$5,4,2,$6,$7,$8)")
        .bind(scope.tenant_id).bind(scope.project_id).bind(scope.branch_id)
        .bind(scope.engineering_run_id).bind(snapshot.binding_id)
        .bind(sqlx::types::Json(serde_json::to_value(&forged)?))
        .bind(vec![0_u8;32]).bind(actor).execute(&mut *transaction).await?;
    transaction.commit().await?;
    assert!(matches!(
        store.resolve_current(scope).await,
        Err(RunInfrastructureBindingStoreError::InvalidStoredBinding)
    ));
    pool.close().await;
    admin.close().await;
    Ok(())
}

fn binding_fixture() -> RunInfrastructureBindingSnapshotV2 {
    RunInfrastructureBindingSnapshotV2 {
        schema_version: 2,
        tenant_id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        branch_id: Uuid::new_v4(),
        engineering_run_id: Uuid::new_v4(),
        binding_id: Uuid::new_v4(),
        binding_revision: 1,
        host_environment_id: Uuid::new_v4(),
        k3s_cluster_id: Uuid::new_v4(),
        k3s_cluster_generation: 1,
        cluster_placement_policy: ClusterPlacementPolicyV1::PinnedCluster,
        provider: ProviderKind::MultipassK3sCluster,
        provider_profile_id: Uuid::new_v4(),
        provider_profile_revision: 1,
        provider_profile_digest: [9; 32],
        namespace: KubernetesDnsLabel::new("run-sqlx").unwrap(),
        service_account: KubernetesDnsLabel::new("agent-sqlx").unwrap(),
        runtime_profile_id: Uuid::new_v4(),
        runtime_profile_revision: 1,
        runtime_profile_digest: [11; 32],
        resource_budget: ExecutionResourceBudgetV1 {
            max_rss_bytes: 64 * 1024 * 1024,
            max_cpu_ms: 1000,
            max_runtime_ms: 30000,
            max_child_processes: 1,
            max_parallel_tools: 1,
            max_provider_calls: 1,
            max_output_bytes: 4096,
            max_event_buffer_bytes: 4096,
        },
    }
}
