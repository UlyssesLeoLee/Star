// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=8b592d2ad0f8f10007fcee417113255f880e932f0968cca14fe86c33ed87a73e
MERGE (self:File {path:"crates/platform-messaging-runtime/tests/transport_profile_pg.rs"})
MERGE (test:Symbol {id:"crates/platform-messaging-runtime/tests/transport_profile_pg.rs::profile_master_cas_outbox_rls_and_resolver",kind:"test"})
MERGE (owner:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster",kind:"struct"})
MERGE (resolver:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileResolver",kind:"struct"})
MERGE (change:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::ProfileChangeRequest",kind:"struct"})
MERGE (profile:Table {id:"platform_messaging.transport_profile_revision",class:"Master"})
MERGE (audit:Table {id:"platform_messaging.transport_profile_revision_audit",class:"Transaction"})
MERGE (outbox:Table {id:"platform_messaging.transport_profile_revision_outbox",class:"Transaction"})
MERGE (self)-[:DEFINES]->(test)
MERGE (test)-[:USES_TYPE]->(owner)
MERGE (test)-[:USES_TYPE]->(resolver)
MERGE (test)-[:USES_TYPE]->(change)
MERGE (test)-[:TESTS]->(profile)
MERGE (test)-[:TESTS]->(audit)
MERGE (test)-[:TESTS]->(outbox)
@endcypher
*/
//! Isolated PostgreSQL acceptance for D17 transport profile persistence.

use std::env;

use platform_messaging_runtime::{
    transport_profile::{
        AuthenticatedTransportSession, TransportDirection, TransportMessageKind,
        TransportProfileDefinition, TransportProfileLifecycle,
    },
    transport_profile_pg::{
        PgTransportProfileMaster, PgTransportProfileResolver, ProfileChangeRequest,
        TransportProfileStoreError,
    },
};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_PLATFORM_PROFILE_TEST_DATABASE_URL";

struct TestSession {
    deployment_id: Uuid,
    tenant_id: Uuid,
    local_service_id: Uuid,
    peer_service_id: Uuid,
    endpoint_identity: String,
}

impl AuthenticatedTransportSession for TestSession {
    fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    fn provider_id(&self) -> &str {
        "star-rust-log"
    }

    fn provider_version(&self) -> &str {
        "1.0.0"
    }

    fn endpoint_identity(&self) -> &str {
        &self.endpoint_identity
    }

    fn local_service_id(&self) -> Uuid {
        self.local_service_id
    }

    fn peer_service_id(&self) -> Uuid {
        self.peer_service_id
    }

    fn direction(&self) -> TransportDirection {
        TransportDirection::Publish
    }
}

#[tokio::test]
#[ignore = "requires the disposable D17 PostgreSQL runner"]
async fn profile_master_cas_outbox_rls_and_resolver() {
    let database_url = env::var(DATABASE_URL_ENV).expect("disposable profile database URL");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable profile database");

    let deployment_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let actor_service_id = Uuid::new_v4();
    let profile_id = Uuid::new_v4();
    let sender_service_id = Uuid::new_v4();
    let recipient_service_id = Uuid::new_v4();
    let master = PgTransportProfileMaster::new(pool.clone(), deployment_id, actor_service_id)
        .expect("valid profile owner");

    let first_definition =
        profile_definition(sender_service_id, recipient_service_id, "run-admission");
    let first = profile_change(
        profile_id,
        None,
        first_definition.clone(),
        "profile-create-1",
    );
    let first_revision = master
        .publish_revision(tenant_id, &first)
        .await
        .expect("publish first profile revision");
    assert_eq!(first_revision.revision(), 1);
    assert_eq!(
        master
            .publish_revision(tenant_id, &first)
            .await
            .expect("exact replay returns the committed revision"),
        first_revision
    );

    let duplicate_first = profile_change(
        profile_id,
        None,
        profile_definition(sender_service_id, recipient_service_id, "other-endpoint"),
        "profile-create-conflict",
    );
    assert!(matches!(
        master.publish_revision(tenant_id, &duplicate_first).await,
        Err(TransportProfileStoreError::CompareAndSwapConflict)
    ));

    let second_definition =
        profile_definition(sender_service_id, recipient_service_id, "run-admission-v2");
    let second = profile_change(
        profile_id,
        Some(1),
        second_definition.clone(),
        "profile-revision-2",
    );
    let second_revision = master
        .publish_revision(tenant_id, &second)
        .await
        .expect("publish successor revision");
    assert_eq!(second_revision.revision(), 2);

    let resolver = PgTransportProfileResolver::new(pool.clone());
    let session = TestSession {
        deployment_id,
        tenant_id,
        local_service_id: sender_service_id,
        peer_service_id: recipient_service_id,
        endpoint_identity: "run-admission-v2".to_owned(),
    };
    assert_eq!(
        resolver
            .resolve(&session, profile_id, TransportMessageKind::Request)
            .await
            .expect("resolve authorized current profile"),
        second_revision
    );
    assert!(matches!(
        resolver
            .resolve(&session, profile_id, TransportMessageKind::Resolution)
            .await,
        Err(TransportProfileStoreError::Unauthorized(_))
    ));
    let wrong_endpoint = TestSession {
        endpoint_identity: "other-endpoint".to_owned(),
        ..session
    };
    assert!(matches!(
        resolver
            .resolve(&wrong_endpoint, profile_id, TransportMessageKind::Request)
            .await,
        Err(TransportProfileStoreError::Unauthorized(_))
    ));

    let competing_a = profile_change(
        profile_id,
        Some(2),
        second_definition.clone(),
        "profile-race-a",
    );
    let competing_b = profile_change(
        profile_id,
        Some(2),
        second_definition.clone(),
        "profile-race-b",
    );
    let (result_a, result_b) = tokio::join!(
        master.publish_revision(tenant_id, &competing_a),
        master.publish_revision(tenant_id, &competing_b)
    );
    assert_eq!(
        (if result_a.is_ok() { 1 } else { 0 }) + (if result_b.is_ok() { 1 } else { 0 }),
        1,
        "only one CAS successor may commit"
    );
    for result in [result_a, result_b] {
        if let Err(error) = result {
            assert!(matches!(
                error,
                TransportProfileStoreError::CompareAndSwapConflict
            ));
        }
    }

    let mut revoke = profile_change(
        profile_id,
        Some(3),
        second_definition.clone(),
        "profile-revoke-4",
    );
    revoke.lifecycle = TransportProfileLifecycle::Revoked;
    let revoked = master
        .publish_revision(tenant_id, &revoke)
        .await
        .expect("publish explicit revocation");
    assert_eq!(revoked.revision(), 4);
    assert!(matches!(
        resolver
            .resolve(&wrong_endpoint, profile_id, TransportMessageKind::Request)
            .await,
        Err(TransportProfileStoreError::Inactive)
    ));

    let exact_historical_replay = master
        .publish_revision(tenant_id, &first)
        .await
        .expect("historical exact replay returns the original revision");
    assert_eq!(exact_historical_replay, first_revision);

    assert_profile_rows(&pool, deployment_id, tenant_id, profile_id).await;
    assert_resolver_rls(&pool, deployment_id, tenant_id).await;
    pool.close().await;
}

fn profile_definition(
    sender_service_id: Uuid,
    recipient_service_id: Uuid,
    endpoint_identity: &str,
) -> TransportProfileDefinition {
    TransportProfileDefinition::new(
        "star-rust-log".to_owned(),
        "1.0.0".to_owned(),
        endpoint_identity.to_owned(),
        "provider-log-generation-v1".to_owned(),
        vec![sender_service_id],
        vec![recipient_service_id],
        vec![TransportMessageKind::Request],
        Some(
            platform_messaging_runtime::transport_profile::OpaqueSecretReference::new(
                "secretref:platform/admission-client".to_owned(),
            )
            .expect("valid opaque secret reference"),
        ),
    )
    .expect("valid bounded transport profile")
}

fn profile_change(
    profile_id: Uuid,
    expected_revision: Option<i64>,
    definition: TransportProfileDefinition,
    idempotency_key: &str,
) -> ProfileChangeRequest {
    ProfileChangeRequest::new(
        profile_id,
        expected_revision,
        TransportProfileLifecycle::Active,
        definition,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        idempotency_key.to_owned(),
    )
    .expect("valid profile change")
}

async fn assert_profile_rows(
    pool: &PgPool,
    deployment_id: Uuid,
    tenant_id: Uuid,
    profile_id: Uuid,
) {
    let rows = sqlx::query(
        "SELECT revision, valid_to, lifecycle_state \
         FROM platform_messaging.transport_profile_revision \
         WHERE deployment_id=$1 AND tenant_id=$2 AND profile_id=$3 ORDER BY revision",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(profile_id)
    .fetch_all(pool)
    .await
    .expect("read profile revisions");
    assert_eq!(rows.len(), 4);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            row.try_get::<i64, _>("revision").unwrap(),
            (index + 1) as i64
        );
        assert_eq!(
            row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("valid_to")
                .unwrap()
                .is_none(),
            index == 3
        );
    }
    assert_eq!(
        rows[3].try_get::<String, _>("lifecycle_state").unwrap(),
        "revoked"
    );

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM platform_messaging.transport_profile_revision_audit \
         WHERE deployment_id=$1 AND tenant_id=$2 AND profile_id=$3",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(profile_id)
    .fetch_one(pool)
    .await
    .expect("count profile audit events");
    let outbox_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM platform_messaging.transport_profile_revision_outbox \
         WHERE deployment_id=$1 AND tenant_id=$2 AND profile_id=$3",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(profile_id)
    .fetch_one(pool)
    .await
    .expect("count profile Outbox events");
    assert_eq!(audit_count, 4);
    assert_eq!(outbox_count, 4);
}

async fn assert_resolver_rls(pool: &PgPool, deployment_id: Uuid, tenant_id: Uuid) {
    let mut tx = pool.begin().await.expect("begin resolver RLS probe");
    sqlx::query("SET LOCAL ROLE star_platform_profile_resolver")
        .execute(&mut *tx)
        .await
        .expect("assume resolver role");
    sqlx::query("SELECT set_config('app.platform_deployment_id',$1,true)")
        .bind(deployment_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set deployment scope");
    sqlx::query("SELECT set_config('app.platform_profile_tenant_id',$1,true)")
        .bind(Uuid::new_v4().to_string())
        .execute(&mut *tx)
        .await
        .expect("set mismatched tenant scope");
    let visible: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM platform_messaging.transport_profile_revision \
         WHERE deployment_id=$1 AND tenant_id=$2",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await
    .expect("RLS filtered profile lookup");
    assert_eq!(visible, 0);
    tx.commit().await.expect("finish resolver RLS probe");
}
