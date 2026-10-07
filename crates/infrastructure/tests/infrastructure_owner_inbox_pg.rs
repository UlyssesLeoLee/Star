//! @cypher schema=1 source_sha256=b40fc1df5e0a38682bb7aa4a150f0a3c3cd16b660504cec37490185b29fc3f2a
//! MERGE (self:File {path:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs"})
//! MERGE (test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_inbox_is_idempotent_and_honors_terminal_receipts",kind:"test"})
//! MERGE (admission_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_serializes_host_and_cluster_reservations",kind:"test"})
//! MERGE (global_admission_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_serializes_physical_capacity_across_tenants",kind:"test"})
//! MERGE (rejection_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_rejects_provider_generation_mismatch",kind:"test"})
//! MERGE (profile_digest_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_rejects_a_profile_digest_mismatch",kind:"test"})
//! MERGE (expiry_test:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::postgres_admission_ignores_expired_capacity_holds",kind:"test"})
//! MERGE (binding:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::binding",kind:"function"})
//! MERGE (demand:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::demand",kind:"function"})
//! MERGE (payload_digest:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::payload_digest",kind:"function"})
//! MERGE (insert_receipt:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::insert_terminal_receipt",kind:"function"})
//! MERGE (insert_profile:Symbol {id:"crates/infrastructure/tests/infrastructure_owner_inbox_pg.rs::insert_provider_profile",kind:"function"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerInbox",kind:"external_type"})
//! MERGE (pg_pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
//! MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
//! MERGE (reservation:Table {id:"infrastructure.capacity_reservation_bundle"})
//! MERGE (physical_reservation:Table {id:"infrastructure.capacity_physical_reservation",class:"Work"})
//! MERGE (physical_event:Table {id:"infrastructure.capacity_physical_reservation_event",class:"Transaction"})
//! MERGE (reservation_event:Table {id:"infrastructure.capacity_reservation_event"})
//! MERGE (owner_outbox:Table {id:"infrastructure.capacity_owner_outbox"})
//! MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery"})
//! MERGE (provider_profile:Table {id:"infrastructure.provider_profile_revision",class:"Master"})
//! MERGE (admission:Type {id:"infrastructure::PostgresInfrastructureOwnerAdmission",kind:"external_type"})
//! MERGE (provider_profile_type:Type {id:"domain_infrastructure::InfrastructureProviderProfileRevisionV1",kind:"external_type"})
//! MERGE (self)-[:DEFINES]->(test)
//! MERGE (self)-[:DEFINES]->(binding)
//! MERGE (self)-[:DEFINES]->(demand)
//! MERGE (self)-[:DEFINES]->(payload_digest)
//! MERGE (self)-[:DEFINES]->(insert_receipt)
//! MERGE (self)-[:DEFINES]->(insert_profile)
//! MERGE (test)-[:CALLS]->(binding)
//! MERGE (test)-[:CALLS]->(demand)
//! MERGE (test)-[:CALLS]->(store)
//! MERGE (test)-[:CALLS]->(insert_receipt)
//! MERGE (test)-[:READS]->(inbox)
//! MERGE (test)-[:READS]->(receipt)
//! MERGE (insert_receipt)-[:CALLS]->(payload_digest)
//! MERGE (insert_receipt)-[:WRITES]->(receipt)
//! MERGE (insert_profile)-[:WRITES]->(provider_profile)
//! MERGE (insert_profile)-[:USES_TYPE]->(provider_profile_type)
//! MERGE (test)-[:USES_TYPE]->(pg_pool)
//! MERGE (admission_test)-[:CALLS]->(binding)
//! MERGE (admission_test)-[:CALLS]->(demand)
//! MERGE (admission_test)-[:CALLS]->(store)
//! MERGE (admission_test)-[:CALLS]->(admission)
//! MERGE (admission_test)-[:READS]->(reservation)
//! MERGE (global_admission_test)-[:CALLS]->(binding)
//! MERGE (global_admission_test)-[:CALLS]->(demand)
//! MERGE (global_admission_test)-[:CALLS]->(store)
//! MERGE (global_admission_test)-[:CALLS]->(admission)
//! MERGE (global_admission_test)-[:READS]->(physical_reservation)
//! MERGE (global_admission_test)-[:READS]->(physical_event)
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
//! MERGE (profile_digest_test)-[:CALLS]->(binding)
//! MERGE (profile_digest_test)-[:CALLS]->(demand)
//! MERGE (profile_digest_test)-[:CALLS]->(store)
//! MERGE (profile_digest_test)-[:CALLS]->(admission)
//! MERGE (profile_digest_test)-[:CALLS]->(insert_profile)
//! MERGE (profile_digest_test)-[:READS]->(reservation)
//! MERGE (profile_digest_test)-[:READS]->(receipt)
//! MERGE (profile_digest_test)-[:READS]->(owner_outbox)
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
    InfrastructureCapacitySnapshotV2, InfrastructureProviderProfileRevisionV1,
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
use sqlx::{PgPool, postgres::PgPoolOptions};
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
    let tenant_id = Uuid::new_v4();
    let provider_profile_id = Uuid::new_v4();
    let profile = InfrastructureProviderProfileRevisionV1 {
        schema_version: 1,
        tenant_id,
        profile_id: provider_profile_id,
        revision: 1,
        provider: ProviderKind::MultipassK3sCluster,
        placement_policy: ClusterPlacementPolicyV1::PinnedCluster,
        host_resources: resource(600, 512 * 1024 * 1024),
        cluster_resources: resource(600, 512 * 1024 * 1024),
    };
    RunInfrastructureBindingSnapshotV2 {
        schema_version: 2,
        tenant_id,
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
        provider_profile_id,
        provider_profile_revision: 1,
        provider_profile_digest: profile.digest().expect("valid provider profile has digest"),
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

fn provider_profile_for_binding(
    binding: &RunInfrastructureBindingSnapshotV2,
) -> InfrastructureProviderProfileRevisionV1 {
    InfrastructureProviderProfileRevisionV1 {
        schema_version: 1,
        tenant_id: binding.tenant_id,
        profile_id: binding.provider_profile_id,
        revision: binding.provider_profile_revision,
        provider: binding.provider,
        placement_policy: binding.cluster_placement_policy,
        host_resources: resource(600, 512 * 1024 * 1024),
        cluster_resources: resource(600, 512 * 1024 * 1024),
    }
}

async fn insert_provider_profile(
    pool: &PgPool,
    binding: &RunInfrastructureBindingSnapshotV2,
    digest_override: Option<[u8; 32]>,
) {
    let profile = provider_profile_for_binding(binding);
    let digest = digest_override.unwrap_or_else(|| profile.digest().expect("profile digest"));
    let actor_id = Uuid::new_v4();
    let provider_kind = match profile.provider {
        ProviderKind::MultipassK3sCluster => "multipass_k3s_cluster",
        ProviderKind::RemoteK3s => "remote_k3s",
    };
    let placement_policy = match profile.placement_policy {
        ClusterPlacementPolicyV1::SharedPool => "shared_pool",
        ClusterPlacementPolicyV1::RunDedicated => "run_dedicated",
        ClusterPlacementPolicyV1::PinnedCluster => "pinned_cluster",
    };
    let mut tx = pool
        .begin()
        .await
        .expect("begin profile fixture transaction");
    sqlx::query(
        "SELECT set_config('app.tenant_id', $1, true), set_config('app.actor_id', $2, true)",
    )
    .bind(binding.tenant_id.to_string())
    .bind(actor_id.to_string())
    .execute(&mut *tx)
    .await
    .expect("set profile fixture context");
    sqlx::query(
        "INSERT INTO infrastructure.provider_profile_revision (\
            tenant_id, profile_id, revision, schema_version, provider_kind, placement_policy, \
            host_cpu_millicores, host_memory_bytes, host_ephemeral_storage_bytes, host_gpu_units, \
            cluster_cpu_millicores, cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units, \
            profile_digest, changed_by) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) \
         ON CONFLICT (tenant_id, profile_id, revision) DO NOTHING",
    )
    .bind(profile.tenant_id)
    .bind(profile.profile_id)
    .bind(profile.revision)
    .bind(profile.schema_version as i16)
    .bind(provider_kind)
    .bind(placement_policy)
    .bind(profile.host_resources.cpu_millicores as i64)
    .bind(profile.host_resources.memory_bytes as i64)
    .bind(profile.host_resources.ephemeral_storage_bytes as i64)
    .bind(profile.host_resources.gpu_units as i32)
    .bind(profile.cluster_resources.cpu_millicores as i64)
    .bind(profile.cluster_resources.memory_bytes as i64)
    .bind(profile.cluster_resources.ephemeral_storage_bytes as i64)
    .bind(profile.cluster_resources.gpu_units as i32)
    .bind(digest.as_slice())
    .bind(actor_id)
    .execute(&mut *tx)
    .await
    .expect("insert immutable provider profile fixture");
    tx.commit().await.expect("commit provider profile fixture");
}

async fn enqueue_demand(
    pool: &PgPool,
    store: &PostgresInfrastructureOwnerInbox,
    item: &HostInfrastructureDemandV2,
) -> Uuid {
    insert_provider_profile(pool, &item.binding, None).await;
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
    let first_id = enqueue_demand(&pool, &inbox_store, &first).await;
    let second_id = enqueue_demand(&pool, &inbox_store, &second).await;
    let snapshot = capacity_snapshot(&first);

    let (first_result, second_result) = tokio::join!(
        admission_store.admit_inbox(first.tenant_id, first_id, &snapshot),
        admission_store.admit_inbox(second.tenant_id, second_id, &snapshot),
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
            .admit_inbox(first.tenant_id, accepted_inbox_id, &snapshot)
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
    let reserved_profile_vector: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT host_cpu_millicores, host_memory_bytes, cluster_cpu_millicores, cluster_memory_bytes \
         FROM infrastructure.capacity_reservation_bundle \
         WHERE tenant_id = $1 AND reservation_bundle_id = $2",
    )
    .bind(first.tenant_id)
    .bind(reservation_bundle_id)
    .fetch_one(&pool)
    .await
    .expect("read profile-derived paired reservation");
    assert_eq!(
        reserved_profile_vector,
        (600, 512 * 1024 * 1024, 600, 512 * 1024 * 1024)
    );
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL schema runner"]
async fn postgres_admission_serializes_physical_capacity_across_tenants() {
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

    for shared_host in [true, false] {
        let first = demand(binding());
        let mut second_binding = binding();
        if shared_host {
            second_binding.host_environment_id = first.binding.host_environment_id;
        } else {
            second_binding.k3s_cluster_id = first.binding.k3s_cluster_id;
        }
        let second = demand(second_binding);
        assert_ne!(first.tenant_id, second.tenant_id);
        if shared_host {
            assert_ne!(first.binding.k3s_cluster_id, second.binding.k3s_cluster_id);
        } else {
            assert_ne!(
                first.binding.host_environment_id,
                second.binding.host_environment_id
            );
        }

        let first_id = enqueue_demand(&pool, &inbox_store, &first).await;
        let second_id = enqueue_demand(&pool, &inbox_store, &second).await;
        let first_snapshot = capacity_snapshot(&first);
        let second_snapshot = capacity_snapshot(&second);
        let (first_result, second_result) = tokio::join!(
            admission_store.admit_inbox(first.tenant_id, first_id, &first_snapshot),
            admission_store.admit_inbox(second.tenant_id, second_id, &second_snapshot),
        );
        let first_result = first_result.expect("first cross-tenant admission transaction");
        let second_result = second_result.expect("second cross-tenant admission transaction");
        let accepted_tenant_id = match (first_result, second_result) {
            (
                InfrastructureOwnerAdmissionOutcome::Accepted { .. },
                InfrastructureOwnerAdmissionOutcome::RetryScheduled { .. },
            ) => first.tenant_id,
            (
                InfrastructureOwnerAdmissionOutcome::RetryScheduled { .. },
                InfrastructureOwnerAdmissionOutcome::Accepted { .. },
            ) => second.tenant_id,
            pair => panic!("expected one global acceptance and one retry, received {pair:?}"),
        };

        let (physical_reservations, physical_events): (i64, i64) = if shared_host {
            sqlx::query_as(
                "SELECT \
                    (SELECT count(*) FROM infrastructure.capacity_physical_reservation \
                     WHERE host_environment_id = $1), \
                    (SELECT count(*) FROM infrastructure.capacity_physical_reservation_event \
                     WHERE host_environment_id = $1)",
            )
            .bind(first.binding.host_environment_id)
            .fetch_one(&pool)
            .await
            .expect("read globally shared Host capacity facts")
        } else {
            sqlx::query_as(
                "SELECT \
                    (SELECT count(*) FROM infrastructure.capacity_physical_reservation \
                     WHERE k3s_cluster_id = $1), \
                    (SELECT count(*) FROM infrastructure.capacity_physical_reservation_event \
                     WHERE k3s_cluster_id = $1)",
            )
            .bind(first.binding.k3s_cluster_id)
            .fetch_one(&pool)
            .await
            .expect("read globally shared K3s cluster capacity facts")
        };
        assert_eq!((physical_reservations, physical_events), (1, 1));

        let tenant_bundle_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM infrastructure.capacity_reservation_bundle WHERE tenant_id = $1",
        )
        .bind(accepted_tenant_id)
        .fetch_one(&pool)
        .await
        .expect("read the accepted tenant's paired reservation");
        assert_eq!(tenant_bundle_count, 1);
    }
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
    insert_provider_profile(&pool, &item.binding, None).await;
    let mut snapshot = capacity_snapshot(&item);
    snapshot.host_generation += 1;
    assert_eq!(
        admission_store
            .admit_inbox(item.tenant_id, inbox_id, &snapshot)
            .await
            .expect("record generation mismatch rejection"),
        InfrastructureOwnerAdmissionOutcome::Rejected {
            reason_code: "provider_generation_or_scope_mismatch".to_owned(),
            replayed: false,
        }
    );
    assert_eq!(
        admission_store
            .admit_inbox(item.tenant_id, inbox_id, &capacity_snapshot(&item),)
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
async fn postgres_admission_rejects_a_profile_digest_mismatch() {
    let database_url = std::env::var("STAR_INFRA_OWNER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the owner test database URL");
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    let inbox_store = PostgresInfrastructureOwnerInbox::new(pool.clone());
    let admission_store =
        PostgresInfrastructureOwnerAdmission::new(pool.clone(), Uuid::new_v4(), Uuid::new_v4())
            .expect("owner identities are present");
    let item = demand(binding());
    insert_provider_profile(&pool, &item.binding, Some([0x77; 32])).await;
    let inbox_id = enqueue_demand(&pool, &inbox_store, &item).await;

    assert_eq!(
        admission_store
            .admit_inbox(item.tenant_id, inbox_id, &capacity_snapshot(&item))
            .await
            .expect("record profile digest mismatch rejection"),
        InfrastructureOwnerAdmissionOutcome::Rejected {
            reason_code: "provider_profile_digest_mismatch".to_owned(),
            replayed: false,
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
    .expect("read rejected profile facts");
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
    let first_id = enqueue_demand(&pool, &inbox_store, &first).await;
    let second_id = enqueue_demand(&pool, &inbox_store, &second).await;
    let snapshot = capacity_snapshot(&first);
    let accepted = admission_store
        .admit_inbox(first.tenant_id, first_id, &snapshot)
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
    sqlx::query(
        "UPDATE infrastructure.capacity_physical_reservation \
         SET lease_expires_at = clock_timestamp() + interval '2 seconds' \
         WHERE reservation_bundle_id = $1",
    )
    .bind(reservation_bundle_id)
    .execute(&pool)
    .await
    .expect("shorten the global physical lease for the expiry test");
    tokio::time::sleep(std::time::Duration::from_millis(2_100)).await;

    assert!(matches!(
        admission_store
            .admit_inbox(second.tenant_id, second_id, &capacity_snapshot(&second),)
            .await
            .expect("expired hold no longer consumes capacity"),
        InfrastructureOwnerAdmissionOutcome::Accepted {
            replayed: false,
            ..
        }
    ));
    pool.close().await;
}
