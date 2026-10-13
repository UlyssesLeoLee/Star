// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0c5031fdd20789d1ba2cb02075fb3d0d59a8d3c2bcc18f8b3579d8a443099423
MERGE (self:File {path:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs"})
MERGE (test:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::native_origin_verification_sqlx_persists_and_revalidates_exact_origin",kind:"test"})
MERGE (fixture:Type {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::TestFixture",kind:"struct"})
MERGE (delivery:Type {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::Delivery",kind:"struct"})
MERGE (proof:Type {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::OriginProof",kind:"struct"})
MERGE (accepted_identity:Type {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::AcceptedIdentity",kind:"struct"})
MERGE (prepare:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::prepare_fixture",kind:"function"})
MERGE (database_precondition:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::require_d20_database",kind:"function"})
MERGE (provision:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::provision_origin_fixture",kind:"function"})
MERGE (insert_certificate:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::insert_certificate",kind:"function"})
MERGE (owner_scope:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::set_owner_scope",kind:"function"})
MERGE (append:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::append_authenticated",kind:"function"})
MERGE (pull:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::pull_authenticated",kind:"function"})
MERGE (verify:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::verify_origin",kind:"function"})
MERGE (ack:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::acknowledge_authenticated",kind:"function"})
MERGE (revoke:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::revoke_certificate",kind:"function"})
MERGE (source_assert:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::assert_persisted_source",kind:"function"})
MERGE (receipt_assert:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::assert_ack_receipt_and_outbox",kind:"function"})
MERGE (sqlstate_assert:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::assert_sqlstate",kind:"function"})
MERGE (receipt_retry:Symbol {id:"crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs::receipt_failure_rolls_back_and_retry_succeeds",kind:"function"})
MERGE (profile_master:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster",kind:"struct"})
MERGE (profile_publish:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster.publish_revision",kind:"method"})
MERGE (profile_revision:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision",kind:"struct"})
MERGE (append_db:Function {id:"platform_transport.append_authenticated",kind:"database_function"})
MERGE (pull_db:Function {id:"platform_transport.pull_authenticated",kind:"database_function"})
MERGE (verify_db:Function {id:"platform_transport.verify_origin_authenticated",kind:"database_function"})
MERGE (ack_db:Function {id:"platform_transport.acknowledge_authenticated",kind:"database_function"})
MERGE (message_work:Table {id:"platform_transport.message_work",class:"Work"})
MERGE (receipt:Table {id:"platform_transport.delivery_receipt",class:"Transaction"})
MERGE (outbox:Table {id:"platform_transport.receipt_outbox",class:"Transaction"})
MERGE (certificate:Table {id:"platform_transport.service_certificate_identity",class:"Master"})
MERGE (log_master:Table {id:"platform_transport.log_master",class:"Master"})
MERGE (profile_binding:Table {id:"platform_transport.profile_log_binding",class:"Master"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(proof)
MERGE (self)-[:DEFINES]->(accepted_identity)
MERGE (self)-[:DEFINES]->(prepare)
MERGE (self)-[:DEFINES]->(database_precondition)
MERGE (self)-[:DEFINES]->(provision)
MERGE (self)-[:DEFINES]->(insert_certificate)
MERGE (self)-[:DEFINES]->(owner_scope)
MERGE (self)-[:DEFINES]->(append)
MERGE (self)-[:DEFINES]->(pull)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(ack)
MERGE (self)-[:DEFINES]->(revoke)
MERGE (self)-[:DEFINES]->(source_assert)
MERGE (self)-[:DEFINES]->(receipt_assert)
MERGE (self)-[:DEFINES]->(sqlstate_assert)
MERGE (self)-[:DEFINES]->(receipt_retry)
MERGE (self)-[:IMPORTS]->(profile_master)
MERGE (prepare)-[:USES_TYPE]->(profile_revision)
MERGE (test)-[:CALLS]->(prepare)
MERGE (test)-[:CALLS]->(database_precondition)
MERGE (prepare)-[:CALLS]->(profile_publish)
MERGE (prepare)-[:CALLS]->(provision)
MERGE (provision)-[:CALLS]->(owner_scope)
MERGE (provision)-[:CALLS]->(insert_certificate)
MERGE (provision)-[:WRITES]->(certificate)
MERGE (provision)-[:WRITES]->(log_master)
MERGE (provision)-[:WRITES]->(profile_binding)
MERGE (test)-[:CALLS]->(append)
MERGE (test)-[:CALLS]->(pull)
MERGE (test)-[:CALLS]->(verify)
MERGE (test)-[:CALLS]->(ack)
MERGE (test)-[:CALLS]->(revoke)
MERGE (test)-[:CALLS]->(source_assert)
MERGE (test)-[:CALLS]->(receipt_assert)
MERGE (test)-[:CALLS]->(sqlstate_assert)
MERGE (test)-[:CALLS]->(receipt_retry)
MERGE (append)-[:CALLS]->(append_db)
MERGE (pull)-[:CALLS]->(pull_db)
MERGE (verify)-[:CALLS]->(verify_db)
MERGE (ack)-[:CALLS]->(ack_db)
MERGE (ack_db)-[:CALLS]->(verify_db)
MERGE (receipt_retry)-[:CALLS]->(ack_db)
MERGE (revoke)-[:WRITES]->(certificate)
MERGE (source_assert)-[:READS]->(message_work)
MERGE (receipt_assert)-[:READS]->(receipt)
MERGE (receipt_assert)-[:READS]->(outbox)
MERGE (receipt_retry)-[:WRITES]->(receipt)
MERGE (receipt_retry)-[:READS]->(receipt)
MERGE (receipt)-[:EMITS]->(outbox)
MERGE (append_db)-[:WRITES]->(message_work)
MERGE (append_db)-[:WRITES]->(receipt)
MERGE (pull_db)-[:WRITES]->(message_work)
MERGE (verify_db)-[:READS]->(message_work)
MERGE (verify_db)-[:READS]->(certificate)
MERGE (ack_db)-[:WRITES]->(message_work)
MERGE (ack_db)-[:WRITES]->(receipt)
MERGE (test)-[:TESTS]->(append_db)
MERGE (test)-[:TESTS]->(pull_db)
MERGE (test)-[:TESTS]->(verify_db)
MERGE (test)-[:TESTS]->(ack_db)
MERGE (test)-[:TESTS]->(message_work)
MERGE (test)-[:TESTS]->(receipt)
MERGE (test)-[:TESTS]->(outbox)
MERGE (test)-[:TESTS]->(certificate)
MERGE (test)-[:TESTS]->(log_master)
MERGE (test)-[:TESTS]->(profile_binding)
@endcypher
*/

//! SQLx integration coverage for the native origin verification transaction boundary.

use std::env;

use platform_messaging_runtime::{
    transport_profile::{
        OpaqueSecretReference, TransportMessageKind, TransportProfileDefinition,
        TransportProfileLifecycle,
    },
    transport_profile_pg::{PgTransportProfileMaster, ProfileChangeRequest},
};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_PLATFORM_PROFILE_TEST_DATABASE_URL";
const PROVIDER_ID: &str = "star-rust-log";
const PROVIDER_VERSION: &str = "1.0.0";
const ENDPOINT_IDENTITY: &str = "native-origin-sqlx";
const MESSAGE_KIND: &str = "request";

struct TestFixture {
    deployment_id: Uuid,
    tenant_id: Uuid,
    actor_service_id: Uuid,
    sender_service_id: Uuid,
    recipient_service_id: Uuid,
    profile_id: Uuid,
    profile_digest: Vec<u8>,
    log_id: String,
    generation: Uuid,
    sender_certificate: [u8; 32],
    receiver_certificate: [u8; 32],
    unknown_receiver_service_id: Uuid,
    unknown_route_certificate: [u8; 32],
    revoked_sender_certificate: [u8; 32],
}

struct Delivery {
    proof: OriginProof,
    lease_id: Uuid,
    wire: Vec<u8>,
}

#[derive(Clone)]
struct OriginProof {
    log_id: String,
    generation: Uuid,
    sequence: i64,
    message_id: Uuid,
    payload_sha256: Vec<u8>,
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL profile database with D20 migrations and roles applied"]
async fn native_origin_verification_sqlx_persists_and_revalidates_exact_origin() {
    let database_url = env::var(DATABASE_URL_ENV).expect(
        "D20 SQLx proof requires STAR_PLATFORM_PROFILE_TEST_DATABASE_URL pointing to a disposable PostgreSQL database",
    );
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to the disposable PostgreSQL integration database");

    require_d20_database(&pool).await;
    let fixture = prepare_fixture(&pool).await;

    let first_message_id = Uuid::new_v4();
    let first_wire = b"native-origin exact raw wire\x00with binary".to_vec();
    let accepted = append_authenticated(
        &pool,
        &fixture,
        &fixture.sender_certificate,
        first_message_id,
        &first_wire,
    )
    .await
    .expect("append exact raw wire through the authenticated SQL route");
    assert!(!accepted.exact_replay);
    assert_eq!(accepted.log_id, fixture.log_id);
    assert_eq!(accepted.sha256, Sha256::digest(&first_wire).as_slice());

    let exact_replay = append_authenticated(
        &pool,
        &fixture,
        &fixture.sender_certificate,
        first_message_id,
        &first_wire,
    )
    .await
    .expect("exact append replay must return its original durable identity");
    assert!(exact_replay.exact_replay);
    assert_eq!(exact_replay.sequence, accepted.sequence);
    assert_eq!(exact_replay.generation, fixture.generation);

    let changed_wire = b"native-origin changed bytes".to_vec();
    let replay_conflict = append_authenticated(
        &pool,
        &fixture,
        &fixture.sender_certificate,
        first_message_id,
        &changed_wire,
    )
    .await
    .expect_err("same message identity with different wire bytes must fail closed");
    assert_sqlstate(&replay_conflict, "22000");

    let first_delivery = pull_authenticated(&pool, &fixture)
        .await
        .expect("pull the accepted message as its authorized receiver")
        .expect("the accepted message should be leased");
    assert_eq!(first_delivery.proof.message_id, first_message_id);
    assert_eq!(first_delivery.proof.log_id, fixture.log_id);
    assert_eq!(first_delivery.proof.generation, fixture.generation);
    assert_eq!(first_delivery.proof.sequence, accepted.sequence);
    assert_eq!(first_delivery.wire, first_wire);
    assert_eq!(
        first_delivery.proof.payload_sha256,
        Sha256::digest(&first_delivery.wire).as_slice()
    );
    assert_persisted_source(&pool, &fixture, &first_delivery).await;
    assert!(verify_origin(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &first_delivery.proof,
    )
    .await
    .expect("verify current exact origin"));

    let mut invalid = first_delivery.proof.clone();
    invalid.log_id.push_str("-other");
    assert!(
        !verify_origin(&pool, &fixture, &fixture.receiver_certificate, &invalid)
            .await
            .expect("different log route should be denied")
    );
    let mut invalid = first_delivery.proof.clone();
    invalid.generation = Uuid::new_v4();
    assert!(
        !verify_origin(&pool, &fixture, &fixture.receiver_certificate, &invalid)
            .await
            .expect("different generation should be denied")
    );
    let mut invalid = first_delivery.proof.clone();
    invalid.sequence += 100;
    assert!(
        !verify_origin(&pool, &fixture, &fixture.receiver_certificate, &invalid)
            .await
            .expect("different sequence should be denied")
    );
    let mut invalid = first_delivery.proof.clone();
    invalid.message_id = Uuid::new_v4();
    assert!(
        !verify_origin(&pool, &fixture, &fixture.receiver_certificate, &invalid)
            .await
            .expect("different message identity should be denied")
    );
    let mut invalid = first_delivery.proof.clone();
    invalid.payload_sha256[0] ^= 0xff;
    assert!(
        !verify_origin(&pool, &fixture, &fixture.receiver_certificate, &invalid)
            .await
            .expect("different exact-wire digest should be denied")
    );

    let unknown_certificate = [0x7d; 32];
    let unknown_error = verify_origin(&pool, &fixture, &unknown_certificate, &first_delivery.proof)
        .await
        .expect_err("unknown certificate must not verify an origin");
    assert_sqlstate(&unknown_error, "42501");
    let mismatched_certificate = verify_origin(
        &pool,
        &fixture,
        &fixture.unknown_route_certificate,
        &first_delivery.proof,
    )
    .await
    .expect_err("known certificate outside the profile route must not verify an origin");
    assert_sqlstate(&mismatched_certificate, "42501");
    revoke_certificate(&pool, &fixture, &fixture.revoked_sender_certificate)
        .await
        .expect("revoke an isolated sender credential before its attempted append");
    let revoked_before_append = append_authenticated(
        &pool,
        &fixture,
        &fixture.revoked_sender_certificate,
        Uuid::new_v4(),
        b"revoked source must not append",
    )
    .await
    .expect_err("revoked source certificate must not append a new origin");
    assert_sqlstate(&revoked_before_append, "42501");

    let mut invalid_ack_proof = first_delivery.proof.clone();
    invalid_ack_proof.payload_sha256[0] ^= 0xff;
    let invalid_ack = acknowledge_authenticated(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &invalid_ack_proof,
        first_delivery.lease_id,
    )
    .await
    .expect_err("acknowledgement must re-verify the exact source origin");
    assert_sqlstate(&invalid_ack, "42501");

    let mut mismatched_generation = first_delivery.proof.clone();
    mismatched_generation.generation = Uuid::new_v4();
    let invalid_ack = acknowledge_authenticated(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &mismatched_generation,
        first_delivery.lease_id,
    )
    .await
    .expect_err("acknowledgement must reject a different generation");
    assert_sqlstate(&invalid_ack, "42501");

    let mut mismatched_sequence = first_delivery.proof.clone();
    mismatched_sequence.sequence += 1;
    let invalid_ack = acknowledge_authenticated(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &mismatched_sequence,
        first_delivery.lease_id,
    )
    .await
    .expect_err("acknowledgement must reject a different sequence");
    assert_sqlstate(&invalid_ack, "42501");

    let mut mismatched_message = first_delivery.proof.clone();
    mismatched_message.message_id = Uuid::new_v4();
    let invalid_ack = acknowledge_authenticated(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &mismatched_message,
        first_delivery.lease_id,
    )
    .await
    .expect_err("acknowledgement must reject a different message identity");
    assert_sqlstate(&invalid_ack, "42501");

    assert!(verify_origin(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &first_delivery.proof,
    )
    .await
    .expect("failed verification must leave the leased source proof retryable"));

    let stale_lease = acknowledge_authenticated(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &first_delivery.proof,
        Uuid::new_v4(),
    )
    .await
    .expect_err("acknowledgement with a stale lease identity must fail");
    assert_sqlstate(&stale_lease, "40001");
    assert!(verify_origin(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &first_delivery.proof,
    )
    .await
    .expect("stale lease failure must leave the current lease unchanged"));

    receipt_failure_rolls_back_and_retry_succeeds(&pool, &fixture, &first_delivery)
        .await
        .expect("receipt persistence conflict must roll back the ACK and allow retry");
    assert!(!verify_origin(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &first_delivery.proof,
    )
    .await
    .expect("acked messages must no longer be live origin proofs"));
    assert_ack_receipt_and_outbox(&pool, &fixture, &first_delivery).await;

    let second_message_id = Uuid::new_v4();
    let second_wire = b"source certificate historical proof".to_vec();
    append_authenticated(
        &pool,
        &fixture,
        &fixture.sender_certificate,
        second_message_id,
        &second_wire,
    )
    .await
    .expect("append a second message before source certificate revocation");
    let second_delivery = pull_authenticated(&pool, &fixture)
        .await
        .expect("pull the second exact message")
        .expect("the second message should be leased");
    assert_eq!(second_delivery.proof.message_id, second_message_id);

    sqlx::query("SELECT pg_sleep(0.01)")
        .execute(&pool)
        .await
        .expect("separate acceptance and revocation transaction timestamps");
    revoke_certificate(&pool, &fixture, &fixture.sender_certificate)
        .await
        .expect("record source certificate revocation after acceptance");
    assert!(verify_origin(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &second_delivery.proof,
    )
    .await
    .expect("post-acceptance source revocation preserves historical origin evidence"));

    revoke_certificate(&pool, &fixture, &fixture.receiver_certificate)
        .await
        .expect("record receiver certificate revocation");
    let revoked_receiver = verify_origin(
        &pool,
        &fixture,
        &fixture.receiver_certificate,
        &second_delivery.proof,
    )
    .await
    .expect_err("revoked receiver certificate must not verify a live origin");
    assert_sqlstate(&revoked_receiver, "42501");

    pool.close().await;
}

#[derive(Debug)]
struct AcceptedIdentity {
    log_id: String,
    generation: Uuid,
    sequence: i64,
    sha256: Vec<u8>,
    exact_replay: bool,
}

async fn require_d20_database(pool: &PgPool) {
    let ready: bool = sqlx::query_scalar(
        r#"
        SELECT CASE
            WHEN NOT EXISTS (
                SELECT 1 FROM pg_roles
                 WHERE rolname IN ('star_platform_transport_owner', 'star_platform_transport_runtime')
                 HAVING count(*) = 2
            ) THEN FALSE
            WHEN to_regclass('platform_messaging.transport_profile_revision') IS NULL
              OR to_regclass('platform_transport.message_work') IS NULL
              OR to_regclass('platform_transport.service_certificate_identity') IS NULL
              OR to_regclass('platform_transport.delivery_receipt') IS NULL
              OR to_regclass('platform_transport.receipt_outbox') IS NULL
              OR to_regprocedure('platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)') IS NULL
              OR to_regprocedure('platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)') IS NULL
              OR to_regprocedure('platform_transport.verify_origin_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,bytea)') IS NULL
              OR to_regprocedure('platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)') IS NULL
            THEN FALSE
            ELSE
                pg_has_role(current_user, 'star_platform_transport_owner', 'MEMBER')
                AND pg_has_role(current_user, 'star_platform_transport_runtime', 'MEMBER')
                AND has_function_privilege(
                    'star_platform_transport_runtime',
                    'platform_transport.verify_origin_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,bytea)',
                    'EXECUTE'
                )
                AND has_function_privilege(
                    'star_platform_transport_runtime',
                    'platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)',
                    'EXECUTE'
                )
                AND has_function_privilege(
                    'star_platform_transport_runtime',
                    'platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)',
                    'EXECUTE'
                )
                AND has_function_privilege(
                    'star_platform_transport_runtime',
                    'platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)',
                    'EXECUTE'
                )
                AND EXISTS (
                    SELECT 1 FROM pg_attribute
                     WHERE attrelid = to_regclass('platform_transport.message_work')
                       AND attname = 'source_certificate_sha256'
                       AND NOT attisdropped
                )
                AND (
                    SELECT c.relrowsecurity AND c.relforcerowsecurity
                      FROM pg_class c
                     WHERE c.oid = to_regclass('platform_transport.message_work')
                )
                AND NOT EXISTS (
                    SELECT 1
                      FROM (VALUES
                          (to_regclass('platform_transport.service_certificate_identity')),
                          (to_regclass('platform_transport.log_master')),
                          (to_regclass('platform_transport.profile_log_binding')),
                          (to_regclass('platform_transport.delivery_receipt')),
                          (to_regclass('platform_transport.receipt_outbox'))
                      ) AS expected_table(oid)
                      JOIN pg_class c ON c.oid = expected_table.oid
                     WHERE NOT c.relrowsecurity OR NOT c.relforcerowsecurity
                )
        END
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("inspect required D20 PostgreSQL schema, roles, grants, and RLS");
    assert!(
        ready,
        "D20 SQLx integration prerequisites missing: apply the platform transport profile, native server-auth, native log, and native-origin migrations, then configure the disposable database roles and grants"
    );
}

async fn prepare_fixture(pool: &PgPool) -> TestFixture {
    let fixture = TestFixture {
        deployment_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        actor_service_id: Uuid::new_v4(),
        sender_service_id: Uuid::new_v4(),
        recipient_service_id: Uuid::new_v4(),
        profile_id: Uuid::new_v4(),
        profile_digest: Vec::new(),
        log_id: format!("native-origin-{}", Uuid::new_v4().simple()),
        generation: Uuid::new_v4(),
        sender_certificate: Sha256::digest(Uuid::new_v4().as_bytes()).into(),
        receiver_certificate: Sha256::digest(Uuid::new_v4().as_bytes()).into(),
        unknown_receiver_service_id: Uuid::new_v4(),
        unknown_route_certificate: Sha256::digest(Uuid::new_v4().as_bytes()).into(),
        revoked_sender_certificate: Sha256::digest(Uuid::new_v4().as_bytes()).into(),
    };

    let definition = TransportProfileDefinition::new(
        PROVIDER_ID.to_owned(),
        PROVIDER_VERSION.to_owned(),
        ENDPOINT_IDENTITY.to_owned(),
        "native-origin-sqlx-v1".to_owned(),
        vec![fixture.sender_service_id],
        vec![fixture.recipient_service_id],
        vec![TransportMessageKind::Request],
        Some(
            OpaqueSecretReference::new("secretref:test/native-origin-sqlx".to_owned())
                .expect("valid opaque fixture reference"),
        ),
    )
    .expect("valid bounded native transport profile");
    let command = ProfileChangeRequest::new(
        fixture.profile_id,
        None,
        TransportProfileLifecycle::Active,
        definition,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        format!("native-origin-create-{}", Uuid::new_v4().simple()),
    )
    .expect("valid profile publication request");
    let profile_master = PgTransportProfileMaster::new(
        pool.clone(),
        fixture.deployment_id,
        fixture.actor_service_id,
    )
    .expect("valid profile owner scope");
    let revision = profile_master
        .publish_revision(fixture.tenant_id, &command)
        .await
        .expect("publish isolated test transport profile");
    let mut fixture = fixture;
    fixture.profile_digest = revision.digest().to_vec();
    provision_origin_fixture(pool, &fixture, revision.revision()).await;
    fixture
}

async fn provision_origin_fixture(pool: &PgPool, fixture: &TestFixture, revision: i64) {
    let mut tx = pool.begin().await.expect("begin owner fixture transaction");
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await
        .expect("assume the restricted transport owner for fixture provisioning");

    insert_certificate(
        &mut tx,
        fixture,
        &fixture.sender_certificate,
        fixture.sender_service_id,
    )
    .await;
    insert_certificate(
        &mut tx,
        fixture,
        &fixture.receiver_certificate,
        fixture.recipient_service_id,
    )
    .await;
    insert_certificate(
        &mut tx,
        fixture,
        &fixture.unknown_route_certificate,
        fixture.unknown_receiver_service_id,
    )
    .await;
    insert_certificate(
        &mut tx,
        fixture,
        &fixture.revoked_sender_certificate,
        fixture.sender_service_id,
    )
    .await;

    sqlx::query(
        r#"
        INSERT INTO platform_transport.log_master (
            deployment_id, tenant_id, log_id, generation, sender_service_id,
            recipient_service_id, message_kind, payload_budget_bytes,
            terminal_retention_hours, max_delivery_attempts, lifecycle
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 65536, 24, 6, 'active')
        "#,
    )
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(&fixture.log_id)
    .bind(fixture.generation)
    .bind(fixture.sender_service_id)
    .bind(fixture.recipient_service_id)
    .bind(MESSAGE_KIND)
    .execute(&mut *tx)
    .await
    .expect("insert the exact active log generation");

    sqlx::query(
        r#"
        INSERT INTO platform_transport.profile_log_binding (
            deployment_id, tenant_id, profile_id, revision, profile_digest,
            log_generation_policy, sender_service_id, recipient_service_id,
            message_kind, log_id, generation, provisioned_by_service_id, request_id
        ) VALUES (
            $1, $2, $3, $4, $5, 'native-origin-sqlx-v1', $6, $7, $8, $9, $10, $11, $12
        )
        "#,
    )
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(fixture.profile_id)
    .bind(revision)
    .bind(&fixture.profile_digest)
    .bind(fixture.sender_service_id)
    .bind(fixture.recipient_service_id)
    .bind(MESSAGE_KIND)
    .bind(&fixture.log_id)
    .bind(fixture.generation)
    .bind(fixture.actor_service_id)
    .bind(Uuid::new_v4())
    .execute(&mut *tx)
    .await
    .expect("bind the profile to the exact active log generation");

    tx.commit().await.expect("commit owner fixture facts");
}

async fn insert_certificate(
    tx: &mut Transaction<'_, Postgres>,
    fixture: &TestFixture,
    certificate_sha256: &[u8; 32],
    service_id: Uuid,
) {
    sqlx::query(
        r#"
        INSERT INTO platform_transport.service_certificate_identity (
            certificate_sha256, deployment_id, tenant_id, service_id,
            provider_id, provider_version, endpoint_identity, lifecycle_state,
            registered_by_service_id, registration_request_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, $9)
        "#,
    )
    .bind(certificate_sha256.as_slice())
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(service_id)
    .bind(PROVIDER_ID)
    .bind(PROVIDER_VERSION)
    .bind(ENDPOINT_IDENTITY)
    .bind(fixture.actor_service_id)
    .bind(Uuid::new_v4())
    .execute(&mut **tx)
    .await
    .expect("register an isolated test certificate identity");
}

async fn set_owner_scope(tx: &mut Transaction<'_, Postgres>, fixture: &TestFixture) {
    sqlx::query("SELECT set_config('app.platform_deployment_id', $1, true)")
        .bind(fixture.deployment_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set owner deployment scope");
    sqlx::query("SELECT set_config('app.platform_tenant_id', $1, true)")
        .bind(fixture.tenant_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set owner tenant scope");
    sqlx::query("SELECT set_config('app.platform_profile_tenant_id', $1, true)")
        .bind(fixture.tenant_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set owner profile tenant scope");
    sqlx::query("SELECT set_config('app.platform_actor_service_id', $1, true)")
        .bind(fixture.actor_service_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set owner actor scope");
}

async fn append_authenticated(
    pool: &PgPool,
    fixture: &TestFixture,
    certificate: &[u8; 32],
    message_id: Uuid,
    wire: &[u8],
) -> Result<AcceptedIdentity, sqlx::Error> {
    let digest = Sha256::digest(wire);
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL ROLE star_platform_transport_runtime")
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query(
        r#"
        SELECT accepted_log_id, accepted_generation, accepted_sequence, accepted_sha256, exact_replay
          FROM platform_transport.append_authenticated($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(certificate.as_slice())
    .bind(&fixture.profile_digest)
    .bind(fixture.recipient_service_id)
    .bind(MESSAGE_KIND)
    .bind(message_id)
    .bind(wire)
    .bind(digest.as_slice())
    .fetch_one(&mut *tx)
    .await;
    match result {
        Ok(row) => {
            let accepted = AcceptedIdentity {
                log_id: row.try_get("accepted_log_id")?,
                generation: row.try_get("accepted_generation")?,
                sequence: row.try_get("accepted_sequence")?,
                sha256: row.try_get("accepted_sha256")?,
                exact_replay: row.try_get("exact_replay")?,
            };
            tx.commit().await?;
            Ok(accepted)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn pull_authenticated(
    pool: &PgPool,
    fixture: &TestFixture,
) -> Result<Option<Delivery>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL ROLE star_platform_transport_runtime")
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query(
        r#"
        SELECT delivery_log_id, delivery_generation, delivery_sequence, message_id,
               wire, payload_sha256, lease_id
          FROM platform_transport.pull_authenticated($1, $2, $3, $4, 60)
        "#,
    )
    .bind(fixture.receiver_certificate.as_slice())
    .bind(&fixture.profile_digest)
    .bind(fixture.sender_service_id)
    .bind(MESSAGE_KIND)
    .fetch_optional(&mut *tx)
    .await;
    match result {
        Ok(row) => {
            let delivery = row
                .map(|row| {
                    let proof = OriginProof {
                        log_id: row.try_get("delivery_log_id")?,
                        generation: row.try_get("delivery_generation")?,
                        sequence: row.try_get("delivery_sequence")?,
                        message_id: row.try_get("message_id")?,
                        payload_sha256: row.try_get("payload_sha256")?,
                    };
                    Ok::<_, sqlx::Error>(Delivery {
                        proof,
                        lease_id: row.try_get("lease_id")?,
                        wire: row.try_get("wire")?,
                    })
                })
                .transpose()?;
            tx.commit().await?;
            Ok(delivery)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn verify_origin(
    pool: &PgPool,
    fixture: &TestFixture,
    certificate: &[u8; 32],
    proof: &OriginProof,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL ROLE star_platform_transport_runtime")
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query_scalar(
        r#"
        SELECT platform_transport.verify_origin_authenticated(
            $1, $2, $3, $4, $5, $6, $7, $8, $9
        )
        "#,
    )
    .bind(certificate.as_slice())
    .bind(&fixture.profile_digest)
    .bind(fixture.sender_service_id)
    .bind(MESSAGE_KIND)
    .bind(&proof.log_id)
    .bind(proof.generation)
    .bind(proof.sequence)
    .bind(proof.message_id)
    .bind(&proof.payload_sha256)
    .fetch_one(&mut *tx)
    .await;
    match result {
        Ok(authorized) => {
            tx.commit().await?;
            Ok(authorized)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn acknowledge_authenticated(
    pool: &PgPool,
    fixture: &TestFixture,
    certificate: &[u8; 32],
    proof: &OriginProof,
    lease_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL ROLE star_platform_transport_runtime")
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query_scalar(
        r#"
        SELECT platform_transport.acknowledge_authenticated(
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
        )
        "#,
    )
    .bind(certificate.as_slice())
    .bind(&fixture.profile_digest)
    .bind(fixture.sender_service_id)
    .bind(MESSAGE_KIND)
    .bind(&proof.log_id)
    .bind(proof.generation)
    .bind(proof.sequence)
    .bind(lease_id)
    .bind(proof.message_id)
    .bind(&proof.payload_sha256)
    .fetch_one(&mut *tx)
    .await;
    match result {
        Ok(acknowledged) => {
            tx.commit().await?;
            Ok(acknowledged)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn revoke_certificate(
    pool: &PgPool,
    fixture: &TestFixture,
    certificate: &[u8; 32],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query(
        r#"
        UPDATE platform_transport.service_certificate_identity
           SET lifecycle_state = 'revoked',
               revoked_by_service_id = $1,
               revocation_request_id = $2,
               revoked_at = transaction_timestamp()
         WHERE certificate_sha256 = $3
        "#,
    )
    .bind(fixture.actor_service_id)
    .bind(Uuid::new_v4())
    .bind(certificate.as_slice())
    .execute(&mut *tx)
    .await;
    match result {
        Ok(result) if result.rows_affected() == 1 => tx.commit().await,
        Ok(_) => {
            tx.rollback().await?;
            Err(sqlx::Error::RowNotFound)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn assert_persisted_source(pool: &PgPool, fixture: &TestFixture, delivery: &Delivery) {
    let mut tx = pool.begin().await.expect("begin scoped source inspection");
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await
        .expect("assume owner role to inspect the isolated source fact");
    let row = sqlx::query(
        r#"
        SELECT wire, payload_sha256, source_certificate_sha256
          FROM platform_transport.message_work
         WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3
           AND generation = $4 AND sequence = $5
        "#,
    )
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(&delivery.proof.log_id)
    .bind(delivery.proof.generation)
    .bind(delivery.proof.sequence)
    .fetch_one(&mut *tx)
    .await
    .expect("read the exact persisted delivery source fact");
    let wire: Vec<u8> = row.try_get("wire").expect("stored wire");
    let digest: Vec<u8> = row.try_get("payload_sha256").expect("stored wire digest");
    let source_certificate: Vec<u8> = row
        .try_get("source_certificate_sha256")
        .expect("captured source certificate");
    assert_eq!(wire, delivery.wire);
    assert_eq!(digest, Sha256::digest(&delivery.wire).as_slice());
    assert_eq!(source_certificate, fixture.sender_certificate);
    tx.commit().await.expect("finish scoped source inspection");
}

async fn receipt_failure_rolls_back_and_retry_succeeds(
    pool: &PgPool,
    fixture: &TestFixture,
    delivery: &Delivery,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SAVEPOINT receipt_persistence_failure")
        .execute(&mut *tx)
        .await?;

    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"
        INSERT INTO platform_transport.delivery_receipt (
            receipt_id, deployment_id, tenant_id, log_id, generation, sequence,
            sender_service_id, recipient_service_id, message_kind, message_id,
            action, payload_sha256, lease_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'acked', $11, $12)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(&delivery.proof.log_id)
    .bind(delivery.proof.generation)
    .bind(delivery.proof.sequence)
    .bind(fixture.sender_service_id)
    .bind(fixture.recipient_service_id)
    .bind(MESSAGE_KIND)
    .bind(Uuid::new_v4())
    .bind(&delivery.proof.payload_sha256)
    .bind(delivery.lease_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("RESET ROLE").execute(&mut *tx).await?;
    sqlx::query("SET LOCAL ROLE star_platform_transport_runtime")
        .execute(&mut *tx)
        .await?;
    let failed_ack = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT platform_transport.acknowledge_authenticated(
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
        )
        "#,
    )
    .bind(fixture.receiver_certificate.as_slice())
    .bind(&fixture.profile_digest)
    .bind(fixture.sender_service_id)
    .bind(MESSAGE_KIND)
    .bind(&delivery.proof.log_id)
    .bind(delivery.proof.generation)
    .bind(delivery.proof.sequence)
    .bind(delivery.lease_id)
    .bind(delivery.proof.message_id)
    .bind(&delivery.proof.payload_sha256)
    .fetch_one(&mut *tx)
    .await
    .expect_err("duplicate receipt sequence must abort the ACK receipt transaction");
    assert_sqlstate(&failed_ack, "23505");

    sqlx::query("ROLLBACK TO SAVEPOINT receipt_persistence_failure")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL ROLE star_platform_transport_runtime")
        .execute(&mut *tx)
        .await?;
    let retried_ack: bool = sqlx::query_scalar(
        r#"
        SELECT platform_transport.acknowledge_authenticated(
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
        )
        "#,
    )
    .bind(fixture.receiver_certificate.as_slice())
    .bind(&fixture.profile_digest)
    .bind(fixture.sender_service_id)
    .bind(MESSAGE_KIND)
    .bind(&delivery.proof.log_id)
    .bind(delivery.proof.generation)
    .bind(delivery.proof.sequence)
    .bind(delivery.lease_id)
    .bind(delivery.proof.message_id)
    .bind(&delivery.proof.payload_sha256)
    .fetch_one(&mut *tx)
    .await?;
    assert!(
        retried_ack,
        "the same live lease should succeed after rollback"
    );
    tx.commit().await
}

async fn assert_ack_receipt_and_outbox(pool: &PgPool, fixture: &TestFixture, delivery: &Delivery) {
    let mut tx = pool.begin().await.expect("begin receipt inspection");
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await
        .expect("assume owner role to inspect committed receipt facts");
    let counts: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            (SELECT count(*) FROM platform_transport.delivery_receipt
              WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3
                AND generation = $4 AND sequence = $5 AND message_id = $6 AND action = 'acked'),
            (SELECT count(*) FROM platform_transport.receipt_outbox
              WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3
                AND generation = $4 AND sequence = $5 AND message_id = $6
                AND event_kind = 'message.acked')
        "#,
    )
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(&delivery.proof.log_id)
    .bind(delivery.proof.generation)
    .bind(delivery.proof.sequence)
    .bind(delivery.proof.message_id)
    .fetch_one(&mut *tx)
    .await
    .expect("read committed acknowledgement receipt and outbox event");
    assert_eq!(counts, (1, 1));
    tx.commit().await.expect("finish receipt inspection");
}

fn assert_sqlstate(error: &sqlx::Error, expected: &str) {
    let actual = match error {
        sqlx::Error::Database(database) => database.code(),
        other => panic!("expected PostgreSQL SQLSTATE {expected}, got {other}"),
    };
    assert_eq!(
        actual.as_deref(),
        Some(expected),
        "unexpected PostgreSQL error code or transaction outcome"
    );
}
