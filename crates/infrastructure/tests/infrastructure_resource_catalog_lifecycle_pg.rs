//! @cypher schema=1 source_sha256=2bc11b40836795280ddc1012e205059722602ef8993f915db7c38d6e07fe2b0c
//! MERGE (self:File {path:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs"})
//! MERGE (test:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::postgres_writer_persists_signed_lifecycle_transitions_and_closes_retired_generations",kind:"test"})
//! MERGE (operation:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::registration_operation",kind:"function"})
//! MERGE (signed_receipt:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::signed_receipt",kind:"function"})
//! MERGE (lifecycle_operation:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::lifecycle_operation",kind:"function"})
//! MERGE (signed_lifecycle:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::signed_lifecycle",kind:"function"})
//! MERGE (assert_no_unclosed_retirement:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::assert_database_rejects_retirement_without_master_close",kind:"function"})
//! MERGE (count:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_lifecycle_pg.rs::count",kind:"function"})
//! MERGE (writer:Type {id:"infrastructure::PostgresInfrastructureResourceCatalogWriter",kind:"external_type"})
//! MERGE (keypair:Type {id:"ring::signature::Ed25519KeyPair",kind:"external_type"})
//! MERGE (pg_pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (receipt_table:Table {id:"infrastructure.resource_catalog_manager_receipt",class:"Transaction"})
//! MERGE (event_table:Table {id:"infrastructure.resource_catalog_event",class:"Transaction"})
//! MERGE (master_tables:Schema {id:"infrastructure.resource_catalog_master_tables"})
//! MERGE (test)-[:CALLS]->(operation)
//! MERGE (test)-[:CALLS]->(signed_receipt)
//! MERGE (test)-[:CALLS]->(lifecycle_operation)
//! MERGE (test)-[:CALLS]->(signed_lifecycle)
//! MERGE (test)-[:CALLS]->(assert_no_unclosed_retirement)
//! MERGE (test)-[:CALLS]->(writer)
//! MERGE (test)-[:CALLS]->(pg_pool)
//! MERGE (test)-[:CALLS]->(count)
//! MERGE (signed_receipt)-[:CALLS]->(keypair)
//! MERGE (signed_lifecycle)-[:CALLS]->(keypair)
//! MERGE (assert_no_unclosed_retirement)-[:WRITES]->(receipt_table)
//! MERGE (assert_no_unclosed_retirement)-[:WRITES]->(event_table)
//! MERGE (test)-[:READS]->(receipt_table)
//! MERGE (test)-[:READS]->(event_table)
//! MERGE (test)-[:READS]->(master_tables)
//! @endcypher

use chrono::Utc;
use infrastructure::infrastructure_resource_catalog_writer::{
    Ed25519InfrastructureReceiptVerifier, InfrastructureManagerResourceLifecycleOperationV1,
    InfrastructureManagerResourceOperationV1, InfrastructureResourceKindV1,
    InfrastructureResourceOwnerStateV1, K3sNodeRoleV1, ManagerResourceIdentityV1,
    PostgresInfrastructureResourceCatalogWriter, ResourceCatalogWriteError,
    SignedInfrastructureManagerLifecycleCommandV1, SignedInfrastructureManagerReceiptV1,
    TrustedInfrastructureManagerKey,
};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

const MANAGER_ID: Uuid = Uuid::from_u128(0x5300_0000_0000_0000_0000_0000_0000_0001);
const ACTOR_ID: Uuid = Uuid::from_u128(0x5600_0000_0000_0000_0000_0000_0000_0001);
const KEY_ID: &str = "resource-lifecycle-integration-v1";

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn registration_operation(
    sequence: u128,
    resource: ManagerResourceIdentityV1,
) -> InfrastructureManagerResourceOperationV1 {
    InfrastructureManagerResourceOperationV1 {
        schema_version: 1,
        operation_id: id(0x6200_0000_0000_0000_0000_0000_0000_0000 + sequence),
        receipt_id: id(0x6100_0000_0000_0000_0000_0000_0000_0000 + sequence),
        correlation_id: id(0x6500_0000_0000_0000_0000_0000_0000_0000 + sequence),
        actor_id: ACTOR_ID,
        manager_id: MANAGER_ID,
        observed_at: Utc::now(),
        resource,
    }
}

fn signed_receipt(
    operation: InfrastructureManagerResourceOperationV1,
    keypair: &Ed25519KeyPair,
) -> SignedInfrastructureManagerReceiptV1 {
    let mut receipt = SignedInfrastructureManagerReceiptV1 {
        operation,
        manager_key_id: KEY_ID.to_owned(),
        signature: Vec::new(),
    };
    receipt.signature = keypair
        .sign(
            &receipt
                .signing_message()
                .expect("valid registration to sign"),
        )
        .as_ref()
        .to_vec();
    receipt
}

fn lifecycle_operation(
    sequence: u128,
    resource_kind: InfrastructureResourceKindV1,
    resource_id: Uuid,
    resource_generation: i64,
    target_state: InfrastructureResourceOwnerStateV1,
) -> InfrastructureManagerResourceLifecycleOperationV1 {
    InfrastructureManagerResourceLifecycleOperationV1 {
        schema_version: 1,
        operation_id: id(0x7200_0000_0000_0000_0000_0000_0000_0000 + sequence),
        receipt_id: id(0x7100_0000_0000_0000_0000_0000_0000_0000 + sequence),
        correlation_id: id(0x7500_0000_0000_0000_0000_0000_0000_0000 + sequence),
        actor_id: ACTOR_ID,
        manager_id: MANAGER_ID,
        observed_at: Utc::now(),
        resource_kind,
        resource_id,
        resource_generation,
        target_state,
    }
}

fn signed_lifecycle(
    operation: InfrastructureManagerResourceLifecycleOperationV1,
    keypair: &Ed25519KeyPair,
) -> SignedInfrastructureManagerLifecycleCommandV1 {
    let mut command = SignedInfrastructureManagerLifecycleCommandV1 {
        operation,
        manager_key_id: KEY_ID.to_owned(),
        signature: Vec::new(),
    };
    command.signature = keypair
        .sign(
            &command
                .signing_message()
                .expect("valid lifecycle command to sign"),
        )
        .as_ref()
        .to_vec();
    command
}

async fn count(pool: &PgPool, query: &str, id: Uuid) -> i64 {
    sqlx::query(query)
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("query disposable PostgreSQL")
        .try_get("count")
        .expect("count result is an integer")
}

async fn assert_database_rejects_retirement_without_master_close(pool: &PgPool, resource_id: Uuid) {
    let receipt_id = id(0x7100_0000_0000_0000_0000_0000_0000_00f0);
    let operation_id = id(0x7200_0000_0000_0000_0000_0000_0000_00f0);
    let correlation_id = id(0x7500_0000_0000_0000_0000_0000_0000_00f0);
    let digest = [0xdd_u8; 32];
    let signature = [0xee_u8; 64];
    let mut tx = pool
        .begin()
        .await
        .expect("begin negative database transaction");
    sqlx::query("SET LOCAL ROLE star_infrastructure_owner")
        .execute(&mut *tx)
        .await
        .expect("assume Infrastructure owner role");
    sqlx::query(
        "INSERT INTO infrastructure.resource_catalog_manager_receipt (receipt_id, operation_id, schema_version, manager_id, manager_key_id, resource_kind, resource_id, resource_generation, event_type, owner_state, correlation_id, actor_id, payload_digest, signature, observed_at) VALUES ($1, $2, 1, $3, $4, 'kubernetes_node', $5, 1, 'retired', 'retired', $6, $7, $8, $9, clock_timestamp())",
    )
    .bind(receipt_id)
    .bind(operation_id)
    .bind(MANAGER_ID)
    .bind(KEY_ID)
    .bind(resource_id)
    .bind(correlation_id)
    .bind(ACTOR_ID)
    .bind(digest.as_slice())
    .bind(signature.as_slice())
    .execute(&mut *tx)
    .await
    .expect("insert structurally valid receipt for database-only invariant");
    sqlx::query(
        "INSERT INTO infrastructure.resource_catalog_event (resource_kind, resource_id, resource_generation, event_type, owner_state, source_command_id, correlation_id, actor_id, details_digest, manager_receipt_id) VALUES ('kubernetes_node', $1, 1, 'retired', 'retired', $2, $3, $4, $5, $6)",
    )
    .bind(resource_id)
    .bind(operation_id)
    .bind(correlation_id)
    .bind(ACTOR_ID)
    .bind(digest.as_slice())
    .bind(receipt_id)
    .execute(&mut *tx)
    .await
    .expect("insert matching retirement event before deferred close check");
    assert!(
        tx.commit().await.is_err(),
        "database must reject a retirement event without closing the exact Master generation",
    );
    assert_eq!(
        count(
            pool,
            "SELECT count(*) AS count FROM infrastructure.resource_catalog_manager_receipt WHERE operation_id = $1",
            operation_id,
        )
        .await,
        0,
        "failed deferred close check must roll the receipt back",
    );
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL 18.6 lifecycle runner"]
async fn postgres_writer_persists_signed_lifecycle_transitions_and_closes_retired_generations() {
    let database_url = std::env::var("STAR_INFRASTRUCTURE_RESOURCE_WRITER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the resource-writer test URL");
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    let keypair = Ed25519KeyPair::from_seed_unchecked(&[0x43; 32]).expect("valid test seed");
    let verifier = Ed25519InfrastructureReceiptVerifier::new([TrustedInfrastructureManagerKey {
        manager_id: MANAGER_ID,
        manager_key_id: KEY_ID.to_owned(),
        public_key: keypair
            .public_key()
            .as_ref()
            .try_into()
            .expect("32-byte Ed25519 key"),
    }])
    .expect("one explicitly trusted test Manager key");
    let writer = PostgresInfrastructureResourceCatalogWriter::new(pool.clone(), verifier);

    let host_id = id(0x6400_0000_0000_0000_0000_0000_0000_0001);
    let instance_id = id(0x6400_0000_0000_0000_0000_0000_0000_0002);
    let cluster_id = id(0x6400_0000_0000_0000_0000_0000_0000_0003);
    let membership_id = id(0x6400_0000_0000_0000_0000_0000_0000_0004);
    let node_id = id(0x6400_0000_0000_0000_0000_0000_0000_0005);
    let registrations = [
        registration_operation(
            1,
            ManagerResourceIdentityV1::HostEnvironment {
                host_environment_id: host_id,
                generation: 1,
                host_os: "windows".to_owned(),
                host_arch: "x86_64".to_owned(),
            },
        ),
        registration_operation(
            2,
            ManagerResourceIdentityV1::MultipassInstance {
                multipass_instance_id: instance_id,
                generation: 1,
                host_environment_id: host_id,
                host_generation: 1,
                provider_instance_name: "ci-life-01".to_owned(),
            },
        ),
        registration_operation(
            3,
            ManagerResourceIdentityV1::K3sCluster {
                k3s_cluster_id: cluster_id,
                generation: 1,
                host_environment_id: host_id,
                host_generation: 1,
                cluster_identity_digest: [0x31; 32],
                k3s_version: "v1.34.1+k3s1".to_owned(),
            },
        ),
        registration_operation(
            4,
            ManagerResourceIdentityV1::ClusterInstanceMembership {
                membership_id,
                generation: 1,
                k3s_cluster_id: cluster_id,
                k3s_cluster_generation: 1,
                multipass_instance_id: instance_id,
                multipass_instance_generation: 1,
                host_environment_id: host_id,
                host_generation: 1,
                node_role: K3sNodeRoleV1::Server,
            },
        ),
        registration_operation(
            5,
            ManagerResourceIdentityV1::KubernetesNode {
                kubernetes_node_id: node_id,
                generation: 1,
                membership_id,
                membership_generation: 1,
                k3s_cluster_id: cluster_id,
                k3s_cluster_generation: 1,
                multipass_instance_id: instance_id,
                multipass_instance_generation: 1,
                host_environment_id: host_id,
                host_generation: 1,
                kubernetes_node_uid: "node-life-0001".to_owned(),
                kubernetes_node_name: "ci-server-life-01".to_owned(),
                node_role: K3sNodeRoleV1::Server,
            },
        ),
    ];
    for operation in registrations {
        let signed = signed_receipt(operation, &keypair);
        writer
            .register(&signed)
            .await
            .expect("register resource first");
    }

    assert_database_rejects_retirement_without_master_close(&pool, node_id).await;

    let bad_signature = signed_lifecycle(
        lifecycle_operation(
            30,
            InfrastructureResourceKindV1::HostEnvironment,
            host_id,
            1,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    let mut bad_signature = bad_signature;
    bad_signature.signature[0] ^= 1;
    assert!(matches!(
        writer.transition(&bad_signature).await,
        Err(ResourceCatalogWriteError::InvalidSignature)
    ));
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) AS count FROM infrastructure.resource_catalog_manager_receipt WHERE operation_id = $1",
            bad_signature.operation.operation_id,
        )
        .await,
        0,
        "invalid signature must fail before persistence",
    );

    let stale_generation = signed_lifecycle(
        lifecycle_operation(
            31,
            InfrastructureResourceKindV1::HostEnvironment,
            host_id,
            99,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    assert!(matches!(
        writer.transition(&stale_generation).await,
        Err(ResourceCatalogWriteError::ResourceNotCurrent)
    ));

    let host_draining = signed_lifecycle(
        lifecycle_operation(
            11,
            InfrastructureResourceKindV1::HostEnvironment,
            host_id,
            1,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    let first_drain = writer
        .transition(&host_draining)
        .await
        .expect("move Host from active to draining");
    assert!(!first_drain.replayed);
    assert!(
        writer
            .transition(&host_draining)
            .await
            .expect("replay exact draining command")
            .replayed
    );
    let mut conflicting_drain = host_draining.clone();
    conflicting_drain.operation.actor_id = id(0x6600_0000_0000_0000_0000_0000_0000_0002);
    conflicting_drain.signature = keypair
        .sign(
            &conflicting_drain
                .signing_message()
                .expect("valid conflicting command"),
        )
        .as_ref()
        .to_vec();
    assert!(matches!(
        writer.transition(&conflicting_drain).await,
        Err(ResourceCatalogWriteError::IdempotencyConflict)
    ));

    let cluster_draining_a = signed_lifecycle(
        lifecycle_operation(
            13,
            InfrastructureResourceKindV1::K3sCluster,
            cluster_id,
            1,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    let cluster_draining_b = signed_lifecycle(
        lifecycle_operation(
            14,
            InfrastructureResourceKindV1::K3sCluster,
            cluster_id,
            1,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    let (cluster_a, cluster_b) = tokio::join!(
        writer.transition(&cluster_draining_a),
        writer.transition(&cluster_draining_b),
    );
    let race_results = [cluster_a, cluster_b];
    assert_eq!(
        race_results.iter().filter(|result| result.is_ok()).count(),
        1,
        "resource-generation advisory lock must admit only one same-state transition",
    );
    assert_eq!(
        race_results
            .iter()
            .filter(|result| matches!(
                result,
                Err(ResourceCatalogWriteError::InvalidStateTransition)
            ))
            .count(),
        1,
        "losing transition must observe the committed current state",
    );

    let node_draining = signed_lifecycle(
        lifecycle_operation(
            20,
            InfrastructureResourceKindV1::KubernetesNode,
            node_id,
            1,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    writer
        .transition(&node_draining)
        .await
        .expect("move node to draining");
    let node_active = signed_lifecycle(
        lifecycle_operation(
            21,
            InfrastructureResourceKindV1::KubernetesNode,
            node_id,
            1,
            InfrastructureResourceOwnerStateV1::Active,
        ),
        &keypair,
    );
    writer
        .transition(&node_active)
        .await
        .expect("reactivate a draining node");
    let duplicate_active = signed_lifecycle(
        lifecycle_operation(
            22,
            InfrastructureResourceKindV1::KubernetesNode,
            node_id,
            1,
            InfrastructureResourceOwnerStateV1::Active,
        ),
        &keypair,
    );
    assert!(matches!(
        writer.transition(&duplicate_active).await,
        Err(ResourceCatalogWriteError::InvalidStateTransition)
    ));
    let node_draining_again = signed_lifecycle(
        lifecycle_operation(
            23,
            InfrastructureResourceKindV1::KubernetesNode,
            node_id,
            1,
            InfrastructureResourceOwnerStateV1::Draining,
        ),
        &keypair,
    );
    writer
        .transition(&node_draining_again)
        .await
        .expect("drain the node before retirement");

    let host_retired = signed_lifecycle(
        lifecycle_operation(
            12,
            InfrastructureResourceKindV1::HostEnvironment,
            host_id,
            1,
            InfrastructureResourceOwnerStateV1::Retired,
        ),
        &keypair,
    );
    let instance_retired = signed_lifecycle(
        lifecycle_operation(
            15,
            InfrastructureResourceKindV1::MultipassInstance,
            instance_id,
            1,
            InfrastructureResourceOwnerStateV1::Retired,
        ),
        &keypair,
    );
    let cluster_retired = signed_lifecycle(
        lifecycle_operation(
            16,
            InfrastructureResourceKindV1::K3sCluster,
            cluster_id,
            1,
            InfrastructureResourceOwnerStateV1::Retired,
        ),
        &keypair,
    );
    let membership_retired = signed_lifecycle(
        lifecycle_operation(
            17,
            InfrastructureResourceKindV1::ClusterInstanceMembership,
            membership_id,
            1,
            InfrastructureResourceOwnerStateV1::Retired,
        ),
        &keypair,
    );
    let node_retired = signed_lifecycle(
        lifecycle_operation(
            24,
            InfrastructureResourceKindV1::KubernetesNode,
            node_id,
            1,
            InfrastructureResourceOwnerStateV1::Retired,
        ),
        &keypair,
    );
    for command in [
        &host_retired,
        &instance_retired,
        &cluster_retired,
        &membership_retired,
        &node_retired,
    ] {
        writer
            .transition(command)
            .await
            .expect("retire exact current resource generation");
    }
    assert!(
        writer
            .transition(&node_retired)
            .await
            .expect("replay exact terminal command")
            .replayed
    );
    let after_retirement = signed_lifecycle(
        lifecycle_operation(
            25,
            InfrastructureResourceKindV1::KubernetesNode,
            node_id,
            1,
            InfrastructureResourceOwnerStateV1::Active,
        ),
        &keypair,
    );
    assert!(matches!(
        writer.transition(&after_retirement).await,
        Err(ResourceCatalogWriteError::ResourceAlreadyRetired)
    ));

    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM infrastructure.resource_catalog_manager_receipt",
        )
        .fetch_one(&pool)
        .await
        .expect("count committed signed receipts"),
        15,
        "5 registrations plus 10 accepted lifecycle events; rejected transitions add no receipt",
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM infrastructure.resource_catalog_event")
            .fetch_one(&pool)
            .await
            .expect("count committed owner events"),
        15,
        "each accepted receipt has one matching immutable event",
    );
    for (table, column, resource_id) in [
        ("host_environment_catalog", "host_environment_id", host_id),
        (
            "multipass_instance_catalog",
            "multipass_instance_id",
            instance_id,
        ),
        ("k3s_cluster_catalog", "k3s_cluster_id", cluster_id),
        (
            "k3s_cluster_instance_membership",
            "membership_id",
            membership_id,
        ),
        ("kubernetes_node_catalog", "kubernetes_node_id", node_id),
    ] {
        let query = format!(
            "SELECT count(*) AS count FROM infrastructure.{table} WHERE {column} = $1 AND generation = 1 AND valid_to IS NOT NULL"
        );
        assert_eq!(
            count(&pool, &query, resource_id).await,
            1,
            "{table} closed once"
        );
    }
    pool.close().await;
}
