//! @cypher schema=1 source_sha256=f456a1a8020cc77695579022bbc99da60573cb095d2cafede106d247b9760604
//! MERGE (self:File {path:"crates/infrastructure/tests/infrastructure_resource_catalog_writer_pg.rs"})
//! MERGE (test:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_writer_pg.rs::postgres_writer_registers_atomic_idempotent_signed_resource_facts",kind:"test"})
//! MERGE (operation:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_writer_pg.rs::operation",kind:"function"})
//! MERGE (signed_receipt:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_writer_pg.rs::signed_receipt",kind:"function"})
//! MERGE (count:Symbol {id:"crates/infrastructure/tests/infrastructure_resource_catalog_writer_pg.rs::count",kind:"function"})
//! MERGE (writer:Type {id:"infrastructure::PostgresInfrastructureResourceCatalogWriter",kind:"external_type"})
//! MERGE (keypair:Type {id:"ring::signature::Ed25519KeyPair",kind:"external_type"})
//! MERGE (pg_pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (receipt_table:Table {id:"infrastructure.resource_catalog_manager_receipt",class:"Transaction"})
//! MERGE (host_table:Table {id:"infrastructure.host_environment_catalog",class:"Master"})
//! MERGE (instance_table:Table {id:"infrastructure.multipass_instance_catalog",class:"Master"})
//! MERGE (cluster_table:Table {id:"infrastructure.k3s_cluster_catalog",class:"Master"})
//! MERGE (membership_table:Table {id:"infrastructure.k3s_cluster_instance_membership",class:"Master"})
//! MERGE (node_table:Table {id:"infrastructure.kubernetes_node_catalog",class:"Master"})
//! MERGE (event_table:Table {id:"infrastructure.resource_catalog_event",class:"Transaction"})
//! MERGE (self)-[:DEFINES]->(test)
//! MERGE (self)-[:DEFINES]->(operation)
//! MERGE (self)-[:DEFINES]->(signed_receipt)
//! MERGE (self)-[:DEFINES]->(count)
//! MERGE (test)-[:CALLS]->(operation)
//! MERGE (test)-[:CALLS]->(signed_receipt)
//! MERGE (test)-[:CALLS]->(writer)
//! MERGE (test)-[:CALLS]->(pg_pool)
//! MERGE (test)-[:CALLS]->(count)
//! MERGE (signed_receipt)-[:CALLS]->(keypair)
//! MERGE (test)-[:READS]->(receipt_table)
//! MERGE (test)-[:READS]->(event_table)
//! MERGE (test)-[:READS]->(host_table)
//! MERGE (test)-[:READS]->(instance_table)
//! MERGE (test)-[:READS]->(cluster_table)
//! MERGE (test)-[:READS]->(membership_table)
//! MERGE (test)-[:READS]->(node_table)
//! @endcypher

use chrono::Utc;
use infrastructure::infrastructure_resource_catalog_writer::{
    Ed25519InfrastructureReceiptVerifier, InfrastructureManagerResourceOperationV1, K3sNodeRoleV1,
    ManagerResourceIdentityV1, PostgresInfrastructureResourceCatalogWriter,
    ResourceCatalogWriteError, SignedInfrastructureManagerReceiptV1,
    TrustedInfrastructureManagerKey,
};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

const MANAGER_ID: Uuid = Uuid::from_u128(0x5300_0000_0000_0000_0000_0000_0000_0001);
const ACTOR_ID: Uuid = Uuid::from_u128(0x5600_0000_0000_0000_0000_0000_0000_0001);
const KEY_ID: &str = "resource-writer-integration-v1";

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn operation(
    sequence: u128,
    resource: ManagerResourceIdentityV1,
) -> InfrastructureManagerResourceOperationV1 {
    InfrastructureManagerResourceOperationV1 {
        schema_version: 1,
        operation_id: id(0x5200_0000_0000_0000_0000_0000_0000_0000 + sequence),
        receipt_id: id(0x5100_0000_0000_0000_0000_0000_0000_0000 + sequence),
        correlation_id: id(0x5500_0000_0000_0000_0000_0000_0000_0000 + sequence),
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
        .sign(&receipt.signing_message().expect("valid operation to sign"))
        .as_ref()
        .to_vec();
    receipt
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

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL 18.6 resource-writer runner"]
async fn postgres_writer_registers_atomic_idempotent_signed_resource_facts() {
    let database_url = std::env::var("STAR_INFRASTRUCTURE_RESOURCE_WRITER_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL runner must set the resource-writer test URL");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");

    let keypair = Ed25519KeyPair::from_seed_unchecked(&[0x42; 32]).expect("valid test seed");
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

    let host_id = id(0x5400_0000_0000_0000_0000_0000_0000_0001);
    let instance_id = id(0x5400_0000_0000_0000_0000_0000_0000_0002);
    let cluster_id = id(0x5400_0000_0000_0000_0000_0000_0000_0003);
    let membership_id = id(0x5400_0000_0000_0000_0000_0000_0000_0004);
    let node_id = id(0x5400_0000_0000_0000_0000_0000_0000_0005);
    let host = signed_receipt(
        operation(
            1,
            ManagerResourceIdentityV1::HostEnvironment {
                host_environment_id: host_id,
                generation: 1,
                host_os: "windows".to_owned(),
                host_arch: "x86_64".to_owned(),
            },
        ),
        &keypair,
    );
    assert!(
        !writer
            .register(&host)
            .await
            .expect("register Host")
            .replayed
    );

    let instance = signed_receipt(
        operation(
            2,
            ManagerResourceIdentityV1::MultipassInstance {
                multipass_instance_id: instance_id,
                generation: 1,
                host_environment_id: host_id,
                host_generation: 1,
                provider_instance_name: "ci-test-01".to_owned(),
            },
        ),
        &keypair,
    );
    writer
        .register(&instance)
        .await
        .expect("register Multipass instance");

    let cluster = signed_receipt(
        operation(
            3,
            ManagerResourceIdentityV1::K3sCluster {
                k3s_cluster_id: cluster_id,
                generation: 1,
                host_environment_id: host_id,
                host_generation: 1,
                cluster_identity_digest: [0x11; 32],
                k3s_version: "v1.34.1+k3s1".to_owned(),
            },
        ),
        &keypair,
    );
    writer
        .register(&cluster)
        .await
        .expect("register K3s cluster");

    let membership = signed_receipt(
        operation(
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
        &keypair,
    );
    writer
        .register(&membership)
        .await
        .expect("register cluster membership");

    let node = signed_receipt(
        operation(
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
                kubernetes_node_uid: "node-uid-0001".to_owned(),
                kubernetes_node_name: "ci-server-01".to_owned(),
                node_role: K3sNodeRoleV1::Server,
            },
        ),
        &keypair,
    );
    writer
        .register(&node)
        .await
        .expect("register Kubernetes node");

    assert!(
        writer
            .register(&host)
            .await
            .expect("replay exact Host operation")
            .replayed
    );
    let mut conflicting = host.operation.clone();
    conflicting.actor_id = id(0x5600_0000_0000_0000_0000_0000_0000_0002);
    let conflicting = signed_receipt(conflicting, &keypair);
    assert!(matches!(
        writer.register(&conflicting).await,
        Err(ResourceCatalogWriteError::IdempotencyConflict)
    ));

    let invalid_operation = operation(
        6,
        ManagerResourceIdentityV1::HostEnvironment {
            host_environment_id: id(0x5400_0000_0000_0000_0000_0000_0000_0006),
            generation: 1,
            host_os: "windows".to_owned(),
            host_arch: "x86_64".to_owned(),
        },
    );
    let mut invalid_signature = signed_receipt(invalid_operation.clone(), &keypair);
    invalid_signature.signature[0] ^= 1;
    assert!(matches!(
        writer.register(&invalid_signature).await,
        Err(ResourceCatalogWriteError::InvalidSignature)
    ));
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) AS count FROM infrastructure.resource_catalog_manager_receipt WHERE operation_id = $1",
            invalid_operation.operation_id,
        )
        .await,
        0
    );

    let missing_parent_operation = operation(
        7,
        ManagerResourceIdentityV1::MultipassInstance {
            multipass_instance_id: id(0x5400_0000_0000_0000_0000_0000_0000_0007),
            generation: 1,
            host_environment_id: id(0x5400_0000_0000_0000_0000_0000_0000_0099),
            host_generation: 1,
            provider_instance_name: "orphan-test-01".to_owned(),
        },
    );
    let missing_parent = signed_receipt(missing_parent_operation.clone(), &keypair);
    assert!(matches!(
        writer.register(&missing_parent).await,
        Err(ResourceCatalogWriteError::Database(_))
    ));
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) AS count FROM infrastructure.resource_catalog_manager_receipt WHERE operation_id = $1",
            missing_parent_operation.operation_id,
        )
        .await,
        0,
        "failed parent validation must roll back the inserted receipt",
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) AS count FROM infrastructure.resource_catalog_event WHERE source_command_id = $1",
            missing_parent_operation.operation_id,
        )
        .await,
        0,
        "failed parent validation must not emit an event",
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
        let query =
            format!("SELECT count(*) AS count FROM infrastructure.{table} WHERE {column} = $1");
        assert_eq!(
            count(&pool, &query, resource_id).await,
            1,
            "{table} fact committed once"
        );
    }
    assert_eq!(
        sqlx::query(
            "SELECT count(*) AS count FROM infrastructure.resource_catalog_manager_receipt"
        )
        .fetch_one(&pool)
        .await
        .expect("count receipts")
        .try_get::<i64, _>("count")
        .expect("receipt count is an integer"),
        5,
        "one signed receipt per resource identity",
    );
    assert_eq!(
        sqlx::query("SELECT count(*) AS count FROM infrastructure.resource_catalog_event")
            .fetch_one(&pool)
            .await
            .expect("count events")
            .try_get::<i64, _>("count")
            .expect("event count is an integer"),
        5,
        "one registration event per receipt",
    );
    pool.close().await;
}
