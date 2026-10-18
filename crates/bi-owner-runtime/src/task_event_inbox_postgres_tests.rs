// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=7e43d238bd794a70e80e913c5b37e8d6e055aacfc73c30ee8831847ed2294238
MERGE (self:File {path:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs"})
MERGE (parent:File {path:"crates/bi-owner-runtime/src/task_event_inbox.rs"})
MERGE (module:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::postgres_tests",kind:"module"})
MERGE (test:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::pg_runtime_commit_is_durable_idempotent_and_conflict_safe",kind:"test"})
MERGE (database_identity:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::require_database_identity",kind:"function"})
MERGE (runtime_contract:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::require_runtime_contract",kind:"function"})
MERGE (wire_fixture:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::fresh_lifecycle_wire",kind:"function"})
MERGE (route_fixture:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::route_pin",kind:"function"})
MERGE (database_url_env:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::DATABASE_URL_ENV",kind:"constant"})
MERGE (database_name:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::DATABASE_NAME",kind:"constant"})
MERGE (commit_signature:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs::COMMIT_FUNCTION",kind:"constant"})
MERGE (owner_role:Config {id:"star_bi_owner",kind:"postgres_role"})
MERGE (store:Type {id:"bi_owner_runtime::TaskEventInboxStore",kind:"struct"})
MERGE (principal:Type {id:"bi_owner_runtime::AuthenticatedTaskEventPrincipal",kind:"struct"})
MERGE (route_pin_type:Type {id:"bi_owner_runtime::TaskEventRoutePin",kind:"struct"})
MERGE (receipt_outcome:Type {id:"bi_owner_runtime::ReceiptOutcome",kind:"enum"})
MERGE (inbox_error:Type {id:"bi_owner_runtime::TaskEventInboxError",kind:"enum"})
MERGE (commit:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventInboxStore.commit_exact_wire",kind:"method"})
MERGE (lifecycle_fixture:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::tests.valid_lifecycle_wire",kind:"function"})
MERGE (commit_fn:Database {id:"bi.commit_task_event_v2",kind:"stored_function"})
MERGE (inbox:Table {id:"bi.task_event_inbox",class:"Transaction"})
MERGE (fact:Table {id:"bi.task_event_fact",class:"Transaction"})
MERGE (incident:Table {id:"bi.task_event_integrity_incident",class:"Transaction"})
MERGE (rust_std:ExternalService {id:"rust::std",kind:"standard_library"})
MERGE (serde_json:ExternalService {id:"serde_json",kind:"crate"})
MERGE (sqlx:ExternalService {id:"sqlx",kind:"crate"})
MERGE (uuid:ExternalService {id:"uuid",kind:"crate"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(database_identity)
MERGE (self)-[:DEFINES]->(runtime_contract)
MERGE (self)-[:DEFINES]->(wire_fixture)
MERGE (self)-[:DEFINES]->(route_fixture)
MERGE (self)-[:IMPORTS]->(rust_std)
MERGE (self)-[:IMPORTS]->(serde_json)
MERGE (self)-[:IMPORTS]->(sqlx)
MERGE (self)-[:IMPORTS]->(uuid)
MERGE (module)-[:CALLS]->(test)
MERGE (test)-[:CALLS]->(database_identity)
MERGE (test)-[:CALLS]->(runtime_contract)
MERGE (runtime_contract)-[:READS]->(owner_role)
MERGE (test)-[:CALLS]->(wire_fixture)
MERGE (test)-[:CALLS]->(route_fixture)
MERGE (test)-[:CALLS]->(commit)
MERGE (test)-[:CALLS]->(lifecycle_fixture)
MERGE (test)-[:READS]->(database_url_env)
MERGE (test)-[:READS]->(database_name)
MERGE (test)-[:READS]->(commit_signature)
MERGE (test)-[:USES_TYPE]->(store)
MERGE (test)-[:USES_TYPE]->(principal)
MERGE (test)-[:USES_TYPE]->(route_pin_type)
MERGE (test)-[:USES_TYPE]->(receipt_outcome)
MERGE (test)-[:USES_TYPE]->(inbox_error)
MERGE (test)-[:TESTS]->(commit_fn)
MERGE (test)-[:TESTS]->(inbox)
MERGE (test)-[:TESTS]->(fact)
MERGE (test)-[:TESTS]->(incident)
MERGE (commit)-[:CALLS]->(commit_fn)
MERGE (parent)-[:IMPORTS]->(module)
@endcypher
*/
//! Disposable PostgreSQL coverage for the BI owner Inbox persistence boundary.

use std::{env, net::IpAddr};

use super::tests::valid_lifecycle_wire;
use super::{
    AuthenticatedTaskEventPrincipal, ReceiptOutcome, TaskEventInboxError, TaskEventInboxStore,
    TaskEventRoutePin,
};
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Row,
};
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_BI_TASK_EVENT_TEST_DATABASE_URL";
const DATABASE_NAME: &str = "star_bi_task_event_test";
const COMMIT_FUNCTION: &str = "bi.commit_task_event_v2(uuid,uuid,uuid,uuid,uuid,varchar,integer,varchar,integer,uuid,uuid,timestamptz,jsonb,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,bytea,bytea)";

#[tokio::test]
#[ignore = "requires the dedicated star_bi_task_event_test PostgreSQL database with BI migrations and runtime grants applied"]
async fn pg_runtime_commit_is_durable_idempotent_and_conflict_safe() {
    let database_url = env::var(DATABASE_URL_ENV).expect(
        "BI persistence proof requires STAR_BI_TASK_EVENT_TEST_DATABASE_URL for a disposable PostgreSQL database",
    );
    let options = database_url
        .parse::<PgConnectOptions>()
        .expect("parse the disposable PostgreSQL connection options");
    let host = options.get_host();
    let loopback = host == "localhost"
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    assert!(
        loopback && options.get_port() != 5432,
        "the opt-in BI persistence test requires loopback PostgreSQL on a non-default port"
    );

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options.clone())
        .await
        .expect("connect to the disposable BI persistence database");
    require_database_identity(&pool).await;
    require_runtime_contract(&pool).await;

    let (tenant_id, event_id, wire) = fresh_lifecycle_wire();
    let route = route_pin();
    let principal = AuthenticatedTaskEventPrincipal {
        tenant_id,
        sender_service_id: route.sender_service_id,
        receiver_service_id: route.receiver_service_id,
    };
    let store = TaskEventInboxStore::new(pool.clone(), route.receiver_service_id);

    let committed = store
        .commit_exact_wire(&principal, &route, &wire)
        .await
        .expect("the BI owner function must commit the Inbox receipt and fact");
    assert_eq!(committed.event_id, event_id);
    assert_eq!(committed.outcome, ReceiptOutcome::Committed);

    let replay = store
        .commit_exact_wire(&principal, &route, &wire)
        .await
        .expect("an exact redelivery must return the existing durable receipt");
    assert_eq!(replay.outcome, ReceiptOutcome::ExactReplay);
    assert_eq!(replay.receipt_id, committed.receipt_id);
    assert_eq!(replay.committed_at, committed.committed_at);
    assert_eq!(replay.wire_sha256, committed.wire_sha256);
    assert_eq!(replay.route_pin_sha256, committed.route_pin_sha256);

    let mut changed: Value = serde_json::from_slice(&wire).expect("the exact fixture parses");
    changed["payload"]["lifecycle"]["status"] = json!("completed");
    changed["payload"]["change"]["to_status"] = json!("completed");
    let conflicting_wire =
        serde_json::to_vec(&changed).expect("the conflicting fixture serializes");
    assert!(matches!(
        store
            .commit_exact_wire(&principal, &route, &conflicting_wire)
            .await,
        Err(TaskEventInboxError::IdentityConflict)
    ));

    let replay_after_conflict = store
        .commit_exact_wire(&principal, &route, &wire)
        .await
        .expect("a conflicting identity must not alter the original receipt or fact");
    assert_eq!(replay_after_conflict.outcome, ReceiptOutcome::ExactReplay);
    assert_eq!(replay_after_conflict.receipt_id, committed.receipt_id);
    assert_eq!(replay_after_conflict.wire_sha256, committed.wire_sha256);

    pool.close().await;

    // A fresh pool forces a new PostgreSQL connection, so the replay comes from committed DB state.
    let reopened_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("reconnect to the same disposable BI persistence database");
    require_database_identity(&reopened_pool).await;
    require_runtime_contract(&reopened_pool).await;
    let reopened_store = TaskEventInboxStore::new(reopened_pool.clone(), route.receiver_service_id);
    let durable_replay = reopened_store
        .commit_exact_wire(&principal, &route, &wire)
        .await
        .expect("a committed receipt must survive closing its original pool");
    assert_eq!(durable_replay.outcome, ReceiptOutcome::ExactReplay);
    assert_eq!(durable_replay.receipt_id, committed.receipt_id);
    assert_eq!(durable_replay.committed_at, committed.committed_at);
    assert_eq!(durable_replay.wire_sha256, committed.wire_sha256);
    assert_eq!(durable_replay.route_pin_sha256, committed.route_pin_sha256);
    reopened_pool.close().await;
}

async fn require_database_identity(pool: &PgPool) {
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect("read the connected database identity before writing fixtures");
    assert_eq!(
        database, DATABASE_NAME,
        "this test may write only to its dedicated disposable BI database"
    );
}

async fn require_runtime_contract(pool: &PgPool) {
    let row = sqlx::query(
        r#"
        SELECT
            pg_has_role(current_user, 'star_bi_runtime', 'MEMBER') AS runtime_member,
            NOT pg_has_role(session_user, 'star_bi_owner', 'SET') AS cannot_set_owner,
            has_function_privilege(current_user, $1, 'EXECUTE') AS can_call_commit,
            NOT rolsuper AND NOT rolbypassrls AS current_role_is_restricted,
            EXISTS (
                SELECT 1 FROM pg_roles
                 WHERE rolname = 'star_bi_runtime'
                   AND NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb
                   AND NOT rolcreaterole AND NOT rolbypassrls AND NOT rolinherit
            ) AS runtime_role_is_locked,
            EXISTS (
                SELECT 1 FROM pg_roles
                 WHERE rolname = 'star_bi_owner'
                   AND NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb
                   AND NOT rolcreaterole AND NOT rolreplication
                   AND NOT rolbypassrls AND NOT rolinherit
            ) AS owner_role_is_locked,
            to_regclass('bi.task_event_inbox') IS NOT NULL
              AND (SELECT relrowsecurity AND relforcerowsecurity FROM pg_class
                    WHERE oid = to_regclass('bi.task_event_inbox')) AS inbox_force_rls,
            to_regclass('bi.task_event_fact') IS NOT NULL
              AND (SELECT relrowsecurity AND relforcerowsecurity FROM pg_class
                    WHERE oid = to_regclass('bi.task_event_fact')) AS fact_force_rls,
            to_regclass('bi.task_event_integrity_incident') IS NOT NULL
              AND (SELECT relrowsecurity AND relforcerowsecurity FROM pg_class
                    WHERE oid = to_regclass('bi.task_event_integrity_incident')) AS incident_force_rls,
            NOT has_table_privilege(current_user, 'bi.task_event_inbox', 'INSERT')
              AND NOT has_table_privilege(current_user, 'bi.task_event_inbox', 'UPDATE')
              AND NOT has_table_privilege(current_user, 'bi.task_event_inbox', 'DELETE')
              AND NOT has_table_privilege(current_user, 'bi.task_event_fact', 'INSERT')
              AND NOT has_table_privilege(current_user, 'bi.task_event_fact', 'UPDATE')
              AND NOT has_table_privilege(current_user, 'bi.task_event_fact', 'DELETE')
              AND NOT has_table_privilege(current_user, 'bi.task_event_integrity_incident', 'INSERT')
              AND NOT has_table_privilege(current_user, 'bi.task_event_integrity_incident', 'UPDATE')
              AND NOT has_table_privilege(current_user, 'bi.task_event_integrity_incident', 'DELETE') AS no_direct_dml
        FROM pg_roles
        WHERE rolname = current_user
        "#,
    )
    .bind(COMMIT_FUNCTION)
    .fetch_one(pool)
    .await
    .expect("inspect BI runtime role, function grant, table privileges, and RLS");

    assert!(row
        .try_get::<bool, _>("runtime_member")
        .expect("runtime membership"));
    assert!(row
        .try_get::<bool, _>("cannot_set_owner")
        .expect("authenticated login cannot SET ROLE to the BI owner"));
    assert!(row
        .try_get::<bool, _>("can_call_commit")
        .expect("commit function grant"));
    assert!(row
        .try_get::<bool, _>("current_role_is_restricted")
        .expect("restricted login role"));
    assert!(row
        .try_get::<bool, _>("runtime_role_is_locked")
        .expect("locked runtime role"));
    assert!(row
        .try_get::<bool, _>("owner_role_is_locked")
        .expect("locked BI owner role"));
    assert!(row
        .try_get::<bool, _>("inbox_force_rls")
        .expect("Inbox FORCE RLS"));
    assert!(row
        .try_get::<bool, _>("fact_force_rls")
        .expect("fact FORCE RLS"));
    assert!(row
        .try_get::<bool, _>("incident_force_rls")
        .expect("incident FORCE RLS"));
    assert!(row
        .try_get::<bool, _>("no_direct_dml")
        .expect("runtime direct DML denial"));
}

fn fresh_lifecycle_wire() -> (Uuid, Uuid, Vec<u8>) {
    let (_, fixture) = valid_lifecycle_wire();
    let mut value: Value =
        serde_json::from_slice(&fixture).expect("valid lifecycle fixture parses");
    let tenant_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let work_item_id = Uuid::new_v4();
    value["tenant_id"] = json!(tenant_id);
    value["project_id"] = json!(project_id);
    value["engineering_run_id"] = json!(run_id);
    value["event_id"] = json!(event_id);
    value["work_item_id"] = json!(work_item_id);
    value["actor_id"] = json!(Uuid::new_v4());
    value["correlation_id"] = json!(Uuid::new_v4());
    value["payload"]["work_item_id"] = json!(work_item_id);
    value["payload"]["owner"]["tenant_id"] = json!(tenant_id);
    value["payload"]["owner"]["project_id"] = json!(project_id);
    value["payload"]["owner"]["engineering_run_id"] = json!(run_id);
    (
        tenant_id,
        event_id,
        serde_json::to_vec(&value).expect("fresh lifecycle fixture serializes"),
    )
}

fn route_pin() -> TaskEventRoutePin {
    TaskEventRoutePin {
        sender_service_id: Uuid::new_v4(),
        receiver_service_id: Uuid::new_v4(),
        transport_provider_id: "star-rust-log".to_owned(),
        transport_provider_version: "1.0.0".to_owned(),
        transport_profile_id: Uuid::new_v4(),
        transport_profile_revision: 1,
        transport_profile_digest: [0x42; 32],
        endpoint_identity: "bi-event-pg-test".to_owned(),
        destination_id: "bi-task-events".to_owned(),
        destination_generation: Uuid::new_v4(),
    }
}
