//! @cypher schema=1 source_sha256=93c6248496d8a5b6f7cc91ed148c6733b32247f33278fd16f355ec86024ca5e8
//! MERGE (self:File {path:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs"})
//! MERGE (test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_inbox_is_idempotent_and_honors_terminal_receipts",kind:"test"})
//! MERGE (binding:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::binding",kind:"function"})
//! MERGE (demand:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::demand",kind:"function"})
//! MERGE (payload_digest:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::payload_digest",kind:"function"})
//! MERGE (insert_receipt:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::insert_terminal_receipt",kind:"function"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerInbox",kind:"external_type"})
//! MERGE (pg_pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
//! MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
//! MERGE (self)-[:DEFINES]->(test)
//! MERGE (self)-[:DEFINES]->(binding)
//! MERGE (self)-[:DEFINES]->(demand)
//! MERGE (self)-[:DEFINES]->(payload_digest)
//! MERGE (self)-[:DEFINES]->(insert_receipt)
//! MERGE (test)-[:CALLS]->(binding)
//! MERGE (test)-[:CALLS]->(demand)
//! MERGE (test)-[:CALLS]->(store)
//! MERGE (test)-[:CALLS]->(insert_receipt)
//! MERGE (test)-[:READS]->(inbox)
//! MERGE (test)-[:READS]->(receipt)
//! MERGE (insert_receipt)-[:CALLS]->(payload_digest)
//! MERGE (insert_receipt)-[:WRITES]->(receipt)
//! MERGE (test)-[:USES_TYPE]->(pg_pool)
//! @endcypher

use chrono::{Duration, Utc};
use domain_infrastructure::{
    ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
    KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV2,
};
use infrastructure::infrastructure_owner_inbox::{
    CurrentInfrastructureBindingV2, InboxReceiveOutcome, PostgresInfrastructureOwnerInbox,
    TerminalDemandOutcome,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

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
        provider_profile_digest: [8; 32],
        namespace: KubernetesDnsLabel::new("run-test").expect("valid label"),
        service_account: KubernetesDnsLabel::new("agent").expect("valid label"),
        runtime_profile_id: Uuid::new_v4(),
        runtime_profile_revision: 1,
        runtime_profile_digest: [9; 32],
        resource_budget: budget(),
    }
}

fn demand(binding: RunInfrastructureBindingSnapshotV2) -> HostInfrastructureDemandV2 {
    let now = Utc::now();
    HostInfrastructureDemandV2 {
        schema_version: 2,
        event_id: Uuid::new_v4(),
        correlation_id: Uuid::new_v4(),
        tenant_id: binding.tenant_id,
        project_id: binding.project_id,
        branch_id: binding.branch_id,
        engineering_run_id: binding.engineering_run_id,
        task_execution_run_id: Uuid::new_v4(),
        schedule_rule_id: Uuid::new_v4(),
        schedule_rule_version: 1,
        occurrence_id: Uuid::new_v4(),
        resource_budget: binding.resource_budget,
        binding,
        requested_at: now,
        deadline_at: now + Duration::minutes(5),
    }
}

fn payload_digest(demand: &HostInfrastructureDemandV2) -> [u8; 32] {
    let value: Value = serde_json::to_value(demand).expect("typed demand serializes");
    let mut hasher = Sha256::new();
    hasher.update(b"star.host-infrastructure-demand.v2\0");
    hasher.update(serde_json::to_vec(&value).expect("JSON value serializes"));
    hasher.finalize().into()
}

async fn insert_terminal_receipt(pool: &PgPool, demand: &HostInfrastructureDemandV2) {
    let mut tx = pool.begin().await.expect("begin receipt transaction");
    sqlx::query("SET LOCAL ROLE star_infrastructure_owner")
        .execute(&mut *tx)
        .await
        .expect("restricted owner role is available");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(demand.tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set tenant RLS scope");
    sqlx::query(
        "INSERT INTO infrastructure.capacity_demand_receipt \
         (tenant_id, source_event_id, occurrence_id, payload_digest, outcome, reservation_bundle_id, correlation_id) \
         VALUES ($1, $2, $3, $4, 'accepted', $5, $6)",
    )
    .bind(demand.tenant_id)
    .bind(demand.event_id)
    .bind(demand.occurrence_id)
    .bind(payload_digest(demand).as_slice())
    .bind(Uuid::new_v4())
    .bind(demand.correlation_id)
    .execute(&mut *tx)
    .await
    .expect("append terminal Infrastructure receipt");
    tx.commit().await.expect("commit terminal receipt");
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL schema runner"]
async fn postgres_inbox_is_idempotent_and_honors_terminal_receipts() {
    let database_url = std::env::var("STAR_INFRA_OWNER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the owner test database URL");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    let store = PostgresInfrastructureOwnerInbox::new(pool.clone());
    let current_binding = |binding| CurrentInfrastructureBindingV2 {
        binding,
        host_generation: 1,
    };

    let first = demand(binding());
    let current = current_binding(first.binding.clone());
    let first_result = store
        .receive_demand(&first, &current)
        .await
        .expect("insert the first pending demand");
    let inbox_id = match first_result {
        InboxReceiveOutcome::Enqueued { inbox_id } => inbox_id,
        other => panic!("expected enqueue, received {other:?}"),
    };
    assert_eq!(
        store.receive_demand(&first, &current).await.unwrap(),
        InboxReceiveOutcome::DuplicatePending { inbox_id }
    );

    let mut changed_payload = first.clone();
    changed_payload.correlation_id = Uuid::new_v4();
    assert!(matches!(
        store.receive_demand(&changed_payload, &current).await,
        Err(infrastructure::infrastructure_owner_inbox::InfrastructureOwnerInboxError::IdempotencyConflict)
    ));

    let terminal = demand(binding());
    insert_terminal_receipt(&pool, &terminal).await;
    assert_eq!(
        store
            .receive_demand(&terminal, &current_binding(terminal.binding.clone()))
            .await
            .unwrap(),
        InboxReceiveOutcome::AlreadyTerminal {
            outcome: TerminalDemandOutcome::Accepted
        }
    );
    pool.close().await;
}
