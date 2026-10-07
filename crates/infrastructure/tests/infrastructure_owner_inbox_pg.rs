//! @cypher schema=1 source_sha256=31fbd324585d82564969a38836ed757ae420f8f7eff908186a2e4d5bfcf1c3a1
//! MERGE (self:File {path:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs"})
//! MERGE (test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_inbox_is_idempotent_and_honors_terminal_receipts",kind:"test"})
//! MERGE (admission_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_serializes_host_and_cluster_reservations",kind:"test"})
//! MERGE (rejection_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_rejects_provider_generation_mismatch",kind:"test"})
//! MERGE (expiry_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_ignores_expired_capacity_holds",kind:"test"})
//! MERGE (binding:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::binding",kind:"function"})
//! MERGE (demand:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::demand",kind:"function"})
//! MERGE (payload_digest:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::payload_digest",kind:"function"})
//! MERGE (insert_receipt:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::insert_terminal_receipt",kind:"function"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerInbox",kind:"external_type"})
//! MERGE (pg_pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
//! MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
//! MERGE (reservation:Table {id:"infrastructure.capacity_reservation_bundle"})
//! MERGE (reservation_event:Table {id:"infrastructure.capacity_reservation_event"})
//! MERGE (owner_outbox:Table {id:"infrastructure.capacity_owner_outbox"})
//! MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery"})
//! MERGE (admission:Type {id:"infrastructure::PostgresInfrastructureOwnerAdmission",kind:"external_type"})
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
//! MERGE (admission_test)-[:CALLS]->(binding)
//! MERGE (admission_test)-[:CALLS]->(demand)
//! MERGE (admission_test)-[:CALLS]->(store)
//! MERGE (admission_test)-[:CALLS]->(admission)
//! MERGE (admission_test)-[:READS]->(reservation)
//! MERGE (admission_test)-[:READS]->(receipt)
//! MERGE (admission_test)-[:READS]->(reservation_event)
//! MERGE (admission_test)-[:READS]->(owner_outbox)
//! MERGE (admission_test)-[:READS]->(delivery)
//! MERGE (rejection_test)-[:CALLS]->(binding)
//! MERGE (rejection_test)-[:CALLS]->(demand)
//! MERGE (rejection_test)-[:CALLS]->(store)
//! MERGE (rejection_test)-[:CALLS]->(admission)
//! MERGE (rejection_test)-[:READS]->(reservation)
//! MERGE (rejection_test)-[:READS]->(receipt)
//! MERGE (expiry_test)-[:CALLS]->(binding)
//! MERGE (expiry_test)-[:CALLS]->(demand)
//! MERGE (expiry_test)-[:CALLS]->(store)
//! MERGE (expiry_test)-[:CALLS]->(admission)
//! MERGE (expiry_test)-[:READS]->(reservation)
//! MERGE (expiry_test)-[:WRITES]->(reservation)
//! @endcypher

use chrono::{Duration, Utc};
use domain_infrastructure::{
    ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
    InfrastructureCapacityRequestV2, InfrastructureCapacitySnapshotV2,
    InfrastructureResourceVectorV1, KubernetesDnsLabel, ProviderKind,
    RunInfrastructureBindingSnapshotV2,
};
use infrastructure::infrastructure_owner_admission::{
    InfrastructureOwnerAdmissionOutcome, PostgresInfrastructureOwnerAdmission,
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

fn resource(cpu_millicores: u64, memory_bytes: u64) -> InfrastructureResourceVectorV1 {
    InfrastructureResourceVectorV1 {
        cpu_millicores,
        memory_bytes,
        ephemeral_storage_bytes: 0,
        gpu_units: 0,
    }
}

fn capacity_snapshot(demand: &HostInfrastructureDemandV2) -> InfrastructureCapacitySnapshotV2 {
    InfrastructureCapacitySnapshotV2 {
        schema_version: 2,
        host_environment_id: demand.binding.host_environment_id,
        host_generation: 1,
        k3s_cluster_id: demand.binding.k3s_cluster_id,
        k3s_cluster_generation: demand.binding.k3s_cluster_generation,
        provider: demand.binding.provider,
        observed_at: Utc::now(),
        total: Some(resource(1_200, 4 * 1024 * 1024 * 1024)),
        os_base_reserve: Some(resource(100, 256 * 1024 * 1024)),
        host_safety_reserve: Some(resource(100, 256 * 1024 * 1024)),
        unmanaged_commitments: Some(resource(0, 0)),
        active_reservations: Some(resource(0, 0)),
        cluster_allocatable: Some(resource(1_000, 4 * 1024 * 1024 * 1024)),
        cluster_safety_reserve: Some(resource(0, 0)),
        cluster_unmanaged_commitments: Some(resource(0, 0)),
        cluster_active_reservations: Some(resource(0, 0)),
    }
}

fn capacity_request(demand: &HostInfrastructureDemandV2) -> InfrastructureCapacityRequestV2 {
    InfrastructureCapacityRequestV2 {
        schema_version: 2,
        binding_digest: demand.binding.digest().expect("valid binding has a digest"),
        host_resources: resource(600, 512 * 1024 * 1024),
        cluster_resources: resource(600, 512 * 1024 * 1024),
    }
}

async fn enqueue_demand(
    store: &PostgresInfrastructureOwnerInbox,
    item: &HostInfrastructureDemandV2,
) -> Uuid {
    let current = CurrentInfrastructureBindingV2 {
        binding: item.binding.clone(),
        host_generation: 1,
    };
    match store
        .receive_demand(item, &current)
        .await
        .expect("receive capacity demand")
    {
        InboxReceiveOutcome::Enqueued { inbox_id } => inbox_id,
        other => panic!("expected demand to enqueue, received {other:?}"),
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

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL schema runner"]
async fn postgres_admission_serializes_host_and_cluster_reservations() {
    let database_url = std::env::var("STAR_INFRA_OWNER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the owner test database URL");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    let inbox_store = PostgresInfrastructureOwnerInbox::new(pool.clone());
    let admission_store =
        PostgresInfrastructureOwnerAdmission::new(pool.clone(), Uuid::new_v4(), Uuid::new_v4())
            .expect("owner identities are present");

    let first = demand(binding());
    let second = demand(first.binding.clone());
    let first_id = enqueue_demand(&inbox_store, &first).await;
    let second_id = enqueue_demand(&inbox_store, &second).await;
    let snapshot = capacity_snapshot(&first);
    let request = capacity_request(&first);

    let (first_result, second_result) = tokio::join!(
        admission_store.admit_inbox(first.tenant_id, first_id, &snapshot, &request),
        admission_store.admit_inbox(second.tenant_id, second_id, &snapshot, &request),
    );
    let first_result = first_result.expect("first admission transaction");
    let second_result = second_result.expect("second admission transaction");
    let (accepted_inbox_id, accepted) = match (first_result, second_result) {
        (
            accepted @ InfrastructureOwnerAdmissionOutcome::Accepted { .. },
            InfrastructureOwnerAdmissionOutcome::RetryScheduled { .. },
        ) => (first_id, accepted),
        (
            InfrastructureOwnerAdmissionOutcome::RetryScheduled { .. },
            accepted @ InfrastructureOwnerAdmissionOutcome::Accepted { .. },
        ) => (second_id, accepted),
        pair => panic!("expected one accepted pair and one retry, received {pair:?}"),
    };

    let InfrastructureOwnerAdmissionOutcome::Accepted {
        reservation_bundle_id,
        host_capacity_reservation_id,
        cluster_capacity_reservation_id,
        replayed: false,
    } = accepted
    else {
        panic!("first accepted result must be new");
    };
    assert_ne!(
        host_capacity_reservation_id,
        cluster_capacity_reservation_id
    );
    assert!(matches!(
        admission_store
            .admit_inbox(first.tenant_id, accepted_inbox_id, &snapshot, &request)
            .await
            .expect("replay accepted admission"),
        InfrastructureOwnerAdmissionOutcome::Accepted {
            reservation_bundle_id: replayed_bundle_id,
            replayed: true,
            ..
        } if replayed_bundle_id == reservation_bundle_id
    ));

    let (bundles, receipts, events, outbox, deliveries): (i64, i64, i64, i64, i64) =
        sqlx::query_as(
            "SELECT \
                (SELECT count(*) FROM infrastructure.capacity_reservation_bundle WHERE tenant_id = $1), \
                (SELECT count(*) FROM infrastructure.capacity_demand_receipt WHERE tenant_id = $1), \
                (SELECT count(*) FROM infrastructure.capacity_reservation_event WHERE tenant_id = $1), \
                (SELECT count(*) FROM infrastructure.capacity_owner_outbox WHERE tenant_id = $1), \
                (SELECT count(*) FROM infrastructure.capacity_outbox_delivery WHERE tenant_id = $1)",
        )
        .bind(first.tenant_id)
        .fetch_one(&pool)
        .await
        .expect("read atomic owner admission facts");
    assert_eq!(
        (bundles, receipts, events, outbox, deliveries),
        (1, 1, 1, 1, 1)
    );

    let saved_ids: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT host_capacity_reservation_id, cluster_capacity_reservation_id \
         FROM infrastructure.capacity_reservation_bundle \
         WHERE tenant_id = $1 AND reservation_bundle_id = $2",
    )
    .bind(first.tenant_id)
    .bind(reservation_bundle_id)
    .fetch_optional(&pool)
    .await
    .expect("read saved paired reservation");
    assert_eq!(
        saved_ids,
        Some((
            host_capacity_reservation_id,
            cluster_capacity_reservation_id
        ))
    );
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL schema runner"]
async fn postgres_admission_rejects_provider_generation_mismatch() {
    let database_url = std::env::var("STAR_INFRA_OWNER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the owner test database URL");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    let inbox_store = PostgresInfrastructureOwnerInbox::new(pool.clone());
    let admission_store =
        PostgresInfrastructureOwnerAdmission::new(pool.clone(), Uuid::new_v4(), Uuid::new_v4())
            .expect("owner identities are present");
    let item = demand(binding());
    let current = CurrentInfrastructureBindingV2 {
        binding: item.binding.clone(),
        host_generation: 1,
    };
    let inbox_id = match inbox_store
        .receive_demand(&item, &current)
        .await
        .expect("receive capacity demand")
    {
        InboxReceiveOutcome::Enqueued { inbox_id } => inbox_id,
        other => panic!("expected demand to enqueue, received {other:?}"),
    };
    let mut snapshot = capacity_snapshot(&item);
    snapshot.host_generation += 1;
    let request = capacity_request(&item);
    assert_eq!(
        admission_store
            .admit_inbox(item.tenant_id, inbox_id, &snapshot, &request)
            .await
            .expect("record generation mismatch rejection"),
        InfrastructureOwnerAdmissionOutcome::Rejected {
            reason_code: "provider_generation_or_scope_mismatch".to_owned(),
            replayed: false,
        }
    );
    assert_eq!(
        admission_store
            .admit_inbox(
                item.tenant_id,
                inbox_id,
                &capacity_snapshot(&item),
                &request,
            )
            .await
            .expect("replay terminal rejection"),
        InfrastructureOwnerAdmissionOutcome::Rejected {
            reason_code: "provider_generation_or_scope_mismatch".to_owned(),
            replayed: true,
        }
    );
    let (bundles, receipts, outbox): (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            (SELECT count(*) FROM infrastructure.capacity_reservation_bundle WHERE tenant_id = $1), \
            (SELECT count(*) FROM infrastructure.capacity_demand_receipt WHERE tenant_id = $1), \
            (SELECT count(*) FROM infrastructure.capacity_owner_outbox WHERE tenant_id = $1)",
    )
    .bind(item.tenant_id)
    .fetch_one(&pool)
    .await
    .expect("read rejected owner facts");
    assert_eq!((bundles, receipts, outbox), (0, 1, 1));
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL schema runner"]
async fn postgres_admission_ignores_expired_capacity_holds() {
    let database_url = std::env::var("STAR_INFRA_OWNER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the owner test database URL");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    let inbox_store = PostgresInfrastructureOwnerInbox::new(pool.clone());
    let admission_store =
        PostgresInfrastructureOwnerAdmission::new(pool.clone(), Uuid::new_v4(), Uuid::new_v4())
            .expect("owner identities are present");
    let first = demand(binding());
    let second = demand(first.binding.clone());
    let first_id = enqueue_demand(&inbox_store, &first).await;
    let second_id = enqueue_demand(&inbox_store, &second).await;
    let snapshot = capacity_snapshot(&first);
    let request = capacity_request(&first);
    let accepted = admission_store
        .admit_inbox(first.tenant_id, first_id, &snapshot, &request)
        .await
        .expect("reserve initial Host and cluster capacity");
    let InfrastructureOwnerAdmissionOutcome::Accepted {
        reservation_bundle_id,
        replayed: false,
        ..
    } = accepted
    else {
        panic!("initial demand must be accepted");
    };

    sqlx::query(
        "UPDATE infrastructure.capacity_reservation_bundle \
         SET lease_expires_at = clock_timestamp() + interval '2 seconds' \
         WHERE tenant_id = $1 AND reservation_bundle_id = $2",
    )
    .bind(first.tenant_id)
    .bind(reservation_bundle_id)
    .execute(&pool)
    .await
    .expect("shorten the test lease while retaining the reservation");
    tokio::time::sleep(std::time::Duration::from_millis(2_100)).await;

    assert!(matches!(
        admission_store
            .admit_inbox(
                second.tenant_id,
                second_id,
                &capacity_snapshot(&second),
                &capacity_request(&second),
            )
            .await
            .expect("expired hold no longer consumes capacity"),
        InfrastructureOwnerAdmissionOutcome::Accepted {
            replayed: false,
            ..
        }
    ));
    pool.close().await;
}
