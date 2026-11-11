// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=5b2b6846b5eec60f2ff7e95c4315c11620420d218657f9678f847b71a59ddef0
MERGE (self:File {path:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs"})
MERGE (module:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::task_event_inbox_runtime_grants_pg",kind:"module"})
MERGE (privilege_test:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::pg_runtime_privilege_boundary_is_closed",kind:"test"})
MERGE (set_role_test:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::pg_runtime_owner_set_role_is_denied",kind:"test"})
MERGE (cross_domain_test:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::pg_runtime_cross_domain_privileges_are_denied",kind:"test"})
MERGE (tenant_test:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::pg_runtime_tenant_mismatch_fails_closed",kind:"test"})
MERGE (rollback_test:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::pg_runtime_commit_rollback_is_atomic",kind:"test"})
MERGE (database_url_constant:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::DATABASE_URL_ENV",kind:"constant"})
MERGE (database_name_constant:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::DATABASE_NAME",kind:"constant"})
MERGE (commit_function_constant:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::COMMIT_FUNCTION",kind:"constant"})
MERGE (query_function_constant:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::QUERY_FUNCTION",kind:"constant"})
MERGE (quarantine_function_constant:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::QUARANTINE_FUNCTION",kind:"constant"})
MERGE (commit_sql_constant:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::COMMIT_SQL",kind:"constant"})
MERGE (connect:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::disposable_pool",kind:"function"})
MERGE (identity:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_database_identity",kind:"function"})
MERGE (runtime_principal:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_runtime_principal",kind:"function"})
MERGE (contract:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_runtime_contract",kind:"function"})
MERGE (cross_domain_boundary:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_cross_domain_privilege_boundary",kind:"function"})
MERGE (set_role:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_owner_set_role_denied",kind:"function"})
MERGE (tenant_mismatch:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_tenant_mismatch_denied",kind:"function"})
MERGE (rollback:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::require_rollback_discards_commit",kind:"function"})
MERGE (fixture:Type {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::CommitFixture",kind:"struct"})
MERGE (fixture_new:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::CommitFixture::new",kind:"method"})
MERGE (call_commit:Symbol {id:"crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs::call_commit",kind:"function"})
MERGE (database_url_env:Config {id:"STAR_BI_TASK_EVENT_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (database_name:Database {id:"star_bi_task_event_test",kind:"disposable_test_database"})
MERGE (commit_fn:Database {id:"bi.commit_task_event_v3",kind:"stored_function"})
MERGE (query_fn:Database {id:"bi.list_task_event_page_v1",kind:"stored_function"})
MERGE (quarantine_fn:Database {id:"bi.record_task_event_quarantine_v2",kind:"stored_function"})
MERGE (append_only_guard:Database {id:"bi.reject_task_event_mutation()",kind:"trigger_function"})
MERGE (quarantine_guard:Database {id:"bi.reject_task_event_quarantine_mutation()",kind:"trigger_function"})
MERGE (runtime_role:Config {id:"star_bi_runtime",kind:"postgres_role"})
MERGE (owner_role:Config {id:"star_bi_owner",kind:"postgres_role"})
MERGE (transport_runtime_role:Config {id:"star_platform_transport_runtime",kind:"postgres_role"})
MERGE (append_authenticated:Database {id:"platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)",kind:"stored_function"})
MERGE (pull_authenticated:Database {id:"platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)",kind:"stored_function"})
MERGE (ack_authenticated:Database {id:"platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)",kind:"stored_function"})
MERGE (append_message:Database {id:"platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)",kind:"stored_function"})
MERGE (pull_one:Database {id:"platform_transport.pull_one(text,uuid,integer)",kind:"stored_function"})
MERGE (ack_one:Database {id:"platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)",kind:"stored_function"})
MERGE (purge_terminal:Database {id:"platform_transport.purge_terminal(text,uuid,integer)",kind:"stored_function"})
MERGE (inbox:Table {id:"bi.task_event_inbox",class:"Transaction"})
MERGE (fact:Table {id:"bi.task_event_fact",class:"Transaction"})
MERGE (incident:Table {id:"bi.task_event_integrity_incident",class:"Transaction"})
MERGE (quarantine_table:Table {id:"bi.task_event_quarantine",class:"Transaction"})
MERGE (rust_std:ExternalService {id:"rust::std",kind:"standard_library"})
MERGE (chrono:ExternalService {id:"chrono",kind:"crate"})
MERGE (serde_json:ExternalService {id:"serde_json",kind:"crate"})
MERGE (sqlx:ExternalService {id:"sqlx",kind:"crate"})
MERGE (uuid:ExternalService {id:"uuid",kind:"crate"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEFINES]->(privilege_test)
MERGE (self)-[:DEFINES]->(set_role_test)
MERGE (self)-[:DEFINES]->(cross_domain_test)
MERGE (self)-[:DEFINES]->(tenant_test)
MERGE (self)-[:DEFINES]->(rollback_test)
MERGE (self)-[:DEFINES]->(database_url_constant)
MERGE (self)-[:DEFINES]->(database_name_constant)
MERGE (self)-[:DEFINES]->(commit_function_constant)
MERGE (self)-[:DEFINES]->(query_function_constant)
MERGE (self)-[:DEFINES]->(quarantine_function_constant)
MERGE (self)-[:DEFINES]->(commit_sql_constant)
MERGE (self)-[:DEFINES]->(connect)
MERGE (self)-[:DEFINES]->(identity)
MERGE (self)-[:DEFINES]->(runtime_principal)
MERGE (self)-[:DEFINES]->(contract)
MERGE (self)-[:DEFINES]->(cross_domain_boundary)
MERGE (self)-[:DEFINES]->(set_role)
MERGE (self)-[:DEFINES]->(tenant_mismatch)
MERGE (self)-[:DEFINES]->(rollback)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (self)-[:DEFINES]->(call_commit)
MERGE (fixture)-[:DEFINES]->(fixture_new)
MERGE (self)-[:IMPORTS]->(rust_std)
MERGE (self)-[:IMPORTS]->(chrono)
MERGE (self)-[:IMPORTS]->(serde_json)
MERGE (self)-[:IMPORTS]->(sqlx)
MERGE (self)-[:IMPORTS]->(uuid)
MERGE (connect)-[:READS]->(database_url_env)
MERGE (connect)-[:READS]->(database_url_constant)
MERGE (connect)-[:CALLS]->(identity)
MERGE (identity)-[:READS]->(database_name_constant)
MERGE (identity)-[:TESTS]->(database_name)
MERGE (contract)-[:READS]->(commit_function_constant)
MERGE (contract)-[:READS]->(query_function_constant)
MERGE (contract)-[:READS]->(quarantine_function_constant)
MERGE (runtime_principal)-[:TESTS]->(commit_fn)
MERGE (runtime_principal)-[:TESTS]->(runtime_role)
MERGE (runtime_principal)-[:TESTS]->(owner_role)
MERGE (call_commit)-[:READS]->(commit_sql_constant)
MERGE (privilege_test)-[:CALLS]->(connect)
MERGE (privilege_test)-[:CALLS]->(contract)
MERGE (set_role_test)-[:CALLS]->(connect)
MERGE (set_role_test)-[:CALLS]->(runtime_principal)
MERGE (set_role_test)-[:CALLS]->(set_role)
MERGE (cross_domain_test)-[:CALLS]->(connect)
MERGE (cross_domain_test)-[:CALLS]->(runtime_principal)
MERGE (cross_domain_test)-[:CALLS]->(cross_domain_boundary)
MERGE (tenant_test)-[:CALLS]->(connect)
MERGE (tenant_test)-[:CALLS]->(runtime_principal)
MERGE (tenant_test)-[:CALLS]->(tenant_mismatch)
MERGE (rollback_test)-[:CALLS]->(connect)
MERGE (rollback_test)-[:CALLS]->(runtime_principal)
MERGE (rollback_test)-[:CALLS]->(rollback)
MERGE (contract)-[:TESTS]->(commit_fn)
MERGE (contract)-[:TESTS]->(query_fn)
MERGE (contract)-[:TESTS]->(quarantine_fn)
MERGE (contract)-[:TESTS]->(append_only_guard)
MERGE (contract)-[:TESTS]->(quarantine_guard)
MERGE (contract)-[:TESTS]->(runtime_role)
MERGE (contract)-[:TESTS]->(owner_role)
MERGE (cross_domain_boundary)-[:TESTS]->(runtime_role)
MERGE (cross_domain_boundary)-[:TESTS]->(transport_runtime_role)
MERGE (cross_domain_boundary)-[:TESTS]->(append_authenticated)
MERGE (cross_domain_boundary)-[:TESTS]->(pull_authenticated)
MERGE (cross_domain_boundary)-[:TESTS]->(ack_authenticated)
MERGE (cross_domain_boundary)-[:TESTS]->(append_message)
MERGE (cross_domain_boundary)-[:TESTS]->(pull_one)
MERGE (cross_domain_boundary)-[:TESTS]->(ack_one)
MERGE (cross_domain_boundary)-[:TESTS]->(purge_terminal)
MERGE (contract)-[:TESTS]->(inbox)
MERGE (contract)-[:TESTS]->(fact)
MERGE (contract)-[:TESTS]->(incident)
MERGE (contract)-[:TESTS]->(quarantine_table)
MERGE (tenant_mismatch)-[:CALLS]->(fixture_new)
MERGE (tenant_mismatch)-[:CALLS]->(call_commit)
MERGE (rollback)-[:CALLS]->(fixture_new)
MERGE (rollback)-[:CALLS]->(call_commit)
MERGE (call_commit)-[:CALLS]->(commit_fn)
MERGE (call_commit)-[:USES_TYPE]->(fixture)
@endcypher
*/
//! Opt-in grants and fail-closed checks for the BI Task Event runtime boundary.

use std::{env, net::IpAddr};

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    PgPool, Postgres, Row, Transaction,
};
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_BI_TASK_EVENT_TEST_DATABASE_URL";
const DATABASE_NAME: &str = "star_bi_task_event_test";
const COMMIT_FUNCTION: &str = "bi.commit_task_event_v3(uuid,uuid,uuid,uuid,uuid,varchar,integer,varchar,integer,uuid,uuid,timestamptz,jsonb,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,bytea,bytea,varchar,varchar,uuid,bigint,bytea,uuid)";
const QUERY_FUNCTION: &str = "bi.list_task_event_page_v1(uuid,uuid,uuid,timestamptz,uuid,integer)";
const QUARANTINE_FUNCTION: &str = "bi.record_task_event_quarantine_v2(uuid,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,varchar,varchar,uuid,bigint,bytea,uuid,bytea,integer,varchar)";
const COMMIT_SQL: &str = r#"
    SELECT outcome, receipt_id, committed_at
      FROM bi.commit_task_event_v3(
          $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,
          $17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31
      )
"#;

#[tokio::test]
#[ignore = "requires the dedicated disposable star_bi_task_event_test PostgreSQL database with BI migrations and runtime grants applied"]
async fn pg_runtime_privilege_boundary_is_closed() {
    let pool = disposable_pool().await;
    let failures = require_runtime_contract(&pool).await;
    pool.close().await;
    assert!(
        failures.is_empty(),
        "BI runtime contract failures: {}",
        failures.join("; ")
    );
}

#[tokio::test]
#[ignore = "requires the dedicated disposable star_bi_task_event_test PostgreSQL database with BI and platform transport migrations applied"]
async fn pg_runtime_cross_domain_privileges_are_denied() {
    let pool = disposable_pool().await;
    require_runtime_principal(&pool).await;
    let failures = require_cross_domain_privilege_boundary(&pool).await;
    pool.close().await;
    assert!(
        failures.is_empty(),
        "BI runtime cross-domain privilege failures: {}",
        failures.join("; ")
    );
}

#[tokio::test]
#[ignore = "requires the dedicated disposable star_bi_task_event_test PostgreSQL database with BI migrations and runtime grants applied"]
async fn pg_runtime_owner_set_role_is_denied() {
    let pool = disposable_pool().await;
    require_runtime_principal(&pool).await;
    require_owner_set_role_denied(&pool).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires the dedicated disposable star_bi_task_event_test PostgreSQL database with BI migrations and runtime grants applied"]
async fn pg_runtime_tenant_mismatch_fails_closed() {
    let pool = disposable_pool().await;
    require_runtime_principal(&pool).await;
    require_tenant_mismatch_denied(&pool).await;
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires the dedicated disposable star_bi_task_event_test PostgreSQL database with BI migrations and runtime grants applied"]
async fn pg_runtime_commit_rollback_is_atomic() {
    let pool = disposable_pool().await;
    require_runtime_principal(&pool).await;
    require_rollback_discards_commit(&pool).await;
    pool.close().await;
}

async fn disposable_pool() -> PgPool {
    let database_url = env::var(DATABASE_URL_ENV).expect(
        "BI grants proof requires STAR_BI_TASK_EVENT_TEST_DATABASE_URL for a disposable PostgreSQL database",
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
        "the opt-in BI grants test requires loopback PostgreSQL on a non-default port"
    );

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("connect to the disposable BI Task Event database");
    require_database_identity(&pool).await;
    pool
}

async fn require_database_identity(pool: &PgPool) {
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect(
            "read the connected database identity before catalog checks or transactional probes",
        );
    assert_eq!(
        database, DATABASE_NAME,
        "this test may use only the dedicated disposable BI Task Event database"
    );
}

async fn require_runtime_principal(pool: &PgPool) {
    let row = sqlx::query(
        r#"
        SELECT EXISTS (
                   SELECT 1
                     FROM pg_roles
                    WHERE rolname = current_user
                      AND NOT rolsuper
                      AND NOT rolcreatedb
                      AND NOT rolcreaterole
                      AND NOT rolreplication
                      AND NOT rolbypassrls
               ) AS current_role_is_restricted,
               current_user = session_user
                   AND current_user <> 'star_bi_runtime'
                   AS current_role_is_external_login,
               EXISTS (
                   SELECT 1
                     FROM pg_roles
                    WHERE rolname = session_user
                      AND rolname <> 'star_bi_runtime'
                      AND rolcanlogin
                      AND NOT rolsuper
                      AND NOT rolcreatedb
                      AND NOT rolcreaterole
                      AND NOT rolreplication
                      AND NOT rolbypassrls
               ) AS session_role_is_restricted_login,
               EXISTS (
                   SELECT 1
                     FROM pg_roles
                    WHERE rolname = 'star_bi_runtime'
                      AND NOT rolcanlogin
                      AND NOT rolsuper
                      AND NOT rolcreatedb
                      AND NOT rolcreaterole
                      AND NOT rolreplication
                      AND NOT rolbypassrls
               ) AS bi_runtime_group_is_restricted_nologin,
               NOT COALESCE(
                   pg_has_role(session_user, 'star_bi_owner', 'SET'),
                   false
               ) AS session_cannot_set_owner,
               COALESCE(
                   pg_has_role(current_user, 'star_bi_runtime', 'MEMBER'),
                   false
               ) AS current_role_is_runtime_member,
               COALESCE(
                   has_function_privilege(current_user, to_regprocedure($1)::oid, 'EXECUTE'),
                   false
               ) AS runtime_can_execute_commit
        "#,
    )
    .bind(COMMIT_FUNCTION)
    .fetch_one(pool)
    .await
    .expect("inspect the authenticated BI runtime principal");
    for (column, expectation) in [
        (
            "current_role_is_restricted",
            "current_user must have restricted role attributes",
        ),
        (
            "current_role_is_external_login",
            "current_user must remain the distinct session login and not star_bi_runtime",
        ),
        (
            "session_role_is_restricted_login",
            "session_user must be a restricted login role distinct from star_bi_runtime",
        ),
        (
            "bi_runtime_group_is_restricted_nologin",
            "star_bi_runtime must exist as a restricted NOLOGIN group role",
        ),
        (
            "session_cannot_set_owner",
            "session_user must not be able to SET ROLE to star_bi_owner",
        ),
        (
            "current_role_is_runtime_member",
            "current_user must belong to star_bi_runtime",
        ),
        (
            "runtime_can_execute_commit",
            "current_user must have the BI commit grant",
        ),
    ] {
        let satisfies: bool = row
            .try_get(column)
            .unwrap_or_else(|error| panic!("read {column}: {error}"));
        assert!(satisfies, "{expectation}");
    }
}

async fn require_runtime_contract(pool: &PgPool) -> Vec<String> {
    let row = sqlx::query(
        r#"
        WITH commit_target AS (
            SELECT to_regprocedure($1)::oid AS function_oid
        ),
        allowlist AS (
            SELECT function_oid FROM commit_target
            UNION ALL
            SELECT to_regprocedure($2)::oid
            UNION ALL
            SELECT to_regprocedure($3)::oid
        ),
        trigger_only_functions AS (
            SELECT to_regprocedure('bi.reject_task_event_mutation()')::oid AS function_oid
            UNION ALL
            SELECT to_regprocedure('bi.reject_task_event_quarantine_mutation()')::oid
        ),
        bi_functions AS (
            SELECT p.oid, p.proacl, p.proowner, p.prosecdef, p.proconfig
              FROM pg_proc AS p
              JOIN pg_namespace AS n ON n.oid = p.pronamespace
             WHERE n.nspname = 'bi'
               AND p.prokind = 'f'
        ),
        principals AS (
            SELECT current_user AS role_name
            UNION
            SELECT session_user AS role_name
        )
        SELECT
            (
                SELECT count(*) = 3 AND COALESCE(bool_and(function_oid IS NOT NULL), false)
                  FROM allowlist
            ) AS allowlist_functions_exist,
            NOT EXISTS (
                SELECT 1
                  FROM allowlist AS allowed
                 WHERE allowed.function_oid IS NULL
                    OR NOT COALESCE(
                        has_function_privilege(current_user, allowed.function_oid, 'EXECUTE'),
                        false
                    )
            ) AS runtime_can_execute_allowlist,
            NOT EXISTS (
                SELECT 1
                  FROM allowlist AS allowed
                  JOIN pg_proc AS p ON p.oid = allowed.function_oid
                  CROSS JOIN LATERAL aclexplode(
                      COALESCE(p.proacl, acldefault('f', p.proowner))
                  ) AS acl
                 WHERE acl.grantee = 0::oid
                   AND acl.privilege_type = 'EXECUTE'
            ) AS public_cannot_execute_allowlist,
            (
                SELECT count(*) = 2 AND COALESCE(bool_and(function_oid IS NOT NULL), false)
                  FROM trigger_only_functions
            ) AS trigger_functions_exist,
            COALESCE((
                SELECT string_agg(
                    DISTINCT p.oid::regprocedure::text,
                    ', ' ORDER BY p.oid::regprocedure::text
                )
                  FROM trigger_only_functions AS trigger_function
                  JOIN pg_proc AS p ON p.oid = trigger_function.function_oid
                  CROSS JOIN LATERAL aclexplode(
                      COALESCE(p.proacl, acldefault('f', p.proowner))
                  ) AS acl
                 WHERE acl.grantee = 0::oid
                   AND acl.privilege_type = 'EXECUTE'
            ), '') AS public_trigger_function_execute,
            COALESCE((
                SELECT string_agg(
                    DISTINCT f.oid::regprocedure::text,
                    ', ' ORDER BY f.oid::regprocedure::text
                )
                  FROM bi_functions AS f
                  CROSS JOIN principals
                 WHERE COALESCE(
                           has_function_privilege(principals.role_name, f.oid, 'EXECUTE'),
                           false
                       )
                   AND NOT EXISTS (
                       SELECT 1 FROM allowlist AS allowed
                        WHERE allowed.function_oid = f.oid
                   )
            ), '') AS runtime_unexpected_bi_functions,
            EXISTS (
                SELECT 1
                  FROM pg_roles
                 WHERE rolname = current_user
                   AND NOT rolsuper
                   AND NOT rolcreatedb
                   AND NOT rolcreaterole
                   AND NOT rolreplication
                   AND NOT rolbypassrls
            ) AS current_role_is_restricted,
            EXISTS (
                SELECT 1
                  FROM pg_roles
                 WHERE rolname = session_user
                   AND rolcanlogin
                   AND NOT rolsuper
                   AND NOT rolcreatedb
                   AND NOT rolcreaterole
                   AND NOT rolreplication
                   AND NOT rolbypassrls
            ) AS session_role_is_restricted,
            pg_has_role(current_user, 'star_bi_runtime', 'MEMBER')
                AS current_role_is_runtime_member,
            NOT pg_has_role(session_user, 'star_bi_owner', 'SET')
                AS session_cannot_set_owner,
            EXISTS (
                SELECT 1
                  FROM pg_roles
                 WHERE rolname = 'star_bi_runtime'
                   AND NOT rolcanlogin
                   AND NOT rolsuper
                   AND NOT rolcreatedb
                   AND NOT rolcreaterole
                   AND NOT rolreplication
                   AND NOT rolbypassrls
                   AND NOT rolinherit
            ) AS runtime_role_is_locked,
            EXISTS (
                SELECT 1
                  FROM pg_roles
                 WHERE rolname = 'star_bi_owner'
                   AND NOT rolcanlogin
                   AND NOT rolsuper
                   AND NOT rolcreatedb
                   AND NOT rolcreaterole
                   AND NOT rolreplication
                   AND NOT rolbypassrls
                   AND NOT rolinherit
            ) AS owner_role_is_locked,
            (
                SELECT count(*) = 4
                   AND COALESCE(bool_and(relrowsecurity AND relforcerowsecurity), false)
                  FROM pg_class
                 WHERE oid IN (
                     to_regclass('bi.task_event_inbox'),
                     to_regclass('bi.task_event_fact'),
                     to_regclass('bi.task_event_integrity_incident'),
                     to_regclass('bi.task_event_quarantine')
                 )
            ) AS all_tables_force_rls,
            NOT EXISTS (
                SELECT 1
                  FROM principals
                  CROSS JOIN (VALUES
                      ('bi.task_event_inbox'),
                      ('bi.task_event_fact'),
                      ('bi.task_event_integrity_incident'),
                      ('bi.task_event_quarantine')
                  ) AS protected_tables(table_name)
                  CROSS JOIN (VALUES ('INSERT'), ('UPDATE'), ('DELETE'), ('TRUNCATE'))
                      AS mutations(privilege_name)
                 WHERE has_table_privilege(
                           principals.role_name,
                           protected_tables.table_name,
                           mutations.privilege_name
                       )
            ) AS no_direct_table_mutation,
            COALESCE((
                SELECT p.prosecdef
                   AND pg_get_userbyid(p.proowner) = 'star_bi_owner'
                   AND COALESCE(p.proconfig, ARRAY[]::text[])
                       @> ARRAY['search_path=pg_catalog', 'row_security=on']::text[]
                   AND (
                       SELECT count(*) = 1
                         FROM unnest(COALESCE(p.proconfig, ARRAY[]::text[])) AS settings(setting)
                        WHERE settings.setting LIKE 'search_path=%'
                          AND settings.setting = 'search_path=pg_catalog'
                   )
                  FROM commit_target AS t
                  JOIN pg_proc AS p ON p.oid = t.function_oid
            ), false) AS commit_function_has_fixed_security_settings
        "#,
    )
    .bind(COMMIT_FUNCTION)
    .bind(QUERY_FUNCTION)
    .bind(QUARANTINE_FUNCTION)
    .fetch_one(pool)
    .await
    .expect(
        "inspect BI function allowlist, principals, table privileges, RLS, and function settings",
    );

    let mut failures = Vec::new();
    for (column, expectation) in [
        (
            "allowlist_functions_exist",
            "all three allowlisted BI functions exist with their fixed signatures",
        ),
        (
            "runtime_can_execute_allowlist",
            "the runtime can execute all three allowlisted BI functions",
        ),
        (
            "public_cannot_execute_allowlist",
            "PUBLIC cannot execute any allowlisted BI function",
        ),
        (
            "trigger_functions_exist",
            "both BI Task Event trigger-only functions exist with their fixed signatures",
        ),
        (
            "current_role_is_restricted",
            "the authenticated runtime role is restricted",
        ),
        ("session_role_is_restricted", "the login role is restricted"),
        (
            "current_role_is_runtime_member",
            "the authenticated role belongs to BI runtime",
        ),
        (
            "session_cannot_set_owner",
            "the login cannot SET ROLE to the BI owner",
        ),
        (
            "runtime_role_is_locked",
            "the BI runtime role has locked attributes",
        ),
        (
            "owner_role_is_locked",
            "the BI owner role has locked attributes",
        ),
        (
            "all_tables_force_rls",
            "all four BI Task Event tables FORCE ROW LEVEL SECURITY",
        ),
        (
            "no_direct_table_mutation",
            "the runtime and login roles have no direct protected-table mutation privileges",
        ),
        (
            "commit_function_has_fixed_security_settings",
            "the owner function is SECURITY DEFINER with fixed search_path and row_security",
        ),
    ] {
        match row.try_get::<bool, _>(column) {
            Ok(true) => {}
            Ok(false) => failures.push(expectation.to_owned()),
            Err(error) => failures.push(format!("{expectation}: {error}")),
        }
    }
    let unexpected_functions: String = row
        .try_get("runtime_unexpected_bi_functions")
        .expect("read unexpected BI function identities");
    if !unexpected_functions.is_empty() {
        failures.push(format!(
            "runtime principals have EXECUTE on non-allowlisted BI functions: {unexpected_functions}"
        ));
    }
    let public_trigger_functions: String = row
        .try_get("public_trigger_function_execute")
        .expect("read PUBLIC EXECUTE grants for BI Task Event trigger functions");
    if !public_trigger_functions.is_empty() {
        failures.push(format!(
            "PUBLIC can execute BI Task Event trigger-only functions: {public_trigger_functions}"
        ));
    }
    failures
}

async fn require_cross_domain_privilege_boundary(pool: &PgPool) -> Vec<String> {
    let row = sqlx::query(
        r#"
        WITH principals AS (
            SELECT session_user AS principal_name
            UNION
            SELECT current_user AS principal_name
        ),
        transport_function_signatures(signature) AS (
            VALUES
                ('platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)'::text),
                ('platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)'::text),
                ('platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)'::text),
                ('platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)'::text),
                ('platform_transport.pull_one(text,uuid,integer)'::text),
                ('platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)'::text),
                ('platform_transport.purge_terminal(text,uuid,integer)'::text)
        ),
        resolved_transport_functions AS (
            -- Resolve through pg_proc rather than to_regprocedure(): a correctly restricted
            -- BI runtime holds no USAGE on platform_transport, and to_regprocedure() raises
            -- 42501 for a principal without schema USAGE. The catalog join reads the same
            -- identities without granting the BI runtime any cross-domain schema access.
            SELECT signatures.signature, functions.function_oid
              FROM transport_function_signatures AS signatures
              LEFT JOIN LATERAL (
                  SELECT p.oid AS function_oid
                    FROM pg_proc AS p
                    JOIN pg_namespace AS n ON n.oid = p.pronamespace
                   WHERE n.nspname = 'platform_transport'
                     AND p.proname = split_part(
                             split_part(signatures.signature, '.', 2), '(', 1)
                     AND (
                           SELECT string_agg(format_type(u.argument_type, NULL), ',' ORDER BY u.ordinal)
                             FROM unnest(p.proargtypes) WITH ORDINALITY AS u(argument_type, ordinal)
                         ) = split_part(split_part(signatures.signature, '(', 2), ')', 1)
               ) AS functions ON true
        ),
        unexpected_star_memberships AS (
            SELECT principals.principal_name, roles.rolname
              FROM principals
              JOIN pg_roles AS roles
                ON roles.rolname ~ '^star_'
               AND roles.rolname <> 'star_bi_runtime'
               AND roles.rolname <> principals.principal_name
             WHERE COALESCE(
                       pg_has_role(principals.principal_name, roles.rolname, 'MEMBER'),
                       false
                   )
                OR COALESCE(
                       pg_has_role(principals.principal_name, roles.rolname, 'SET'),
                       false
                   )
        ),
        unexpected_transport_execute AS (
            SELECT principals.principal_name, functions.signature
              FROM principals
              CROSS JOIN resolved_transport_functions AS functions
             WHERE functions.function_oid IS NOT NULL
               AND COALESCE(
                       has_function_privilege(
                           principals.principal_name,
                           functions.function_oid,
                           'EXECUTE'
                       ),
                       false
                   )
        )
        SELECT EXISTS (
                   SELECT 1 FROM pg_roles
                    WHERE rolname = 'star_platform_transport_runtime'
               ) AS transport_runtime_role_exists,
               (
                   SELECT count(*) = 7
                      AND COALESCE(bool_and(function_oid IS NOT NULL), false)
                     FROM resolved_transport_functions
               ) AS all_transport_functions_exist,
               NOT EXISTS (
                   SELECT 1
                     FROM principals
                    WHERE NOT COALESCE(
                        pg_has_role(principal_name, 'star_bi_runtime', 'MEMBER'),
                        false
                    )
               ) AS all_principals_are_bi_runtime_members,
               COALESCE((
                   SELECT string_agg(
                       DISTINCT format('%s via principal %s', rolname, principal_name),
                       ', ' ORDER BY format('%s via principal %s', rolname, principal_name)
                   )
                     FROM unexpected_star_memberships
               ), '') AS unexpected_star_role_memberships,
               COALESCE((
                   SELECT string_agg(
                       DISTINCT format('%s via principal %s', signature, principal_name),
                       ', ' ORDER BY format('%s via principal %s', signature, principal_name)
                   )
                     FROM unexpected_transport_execute
               ), '') AS unexpected_transport_function_execute,
               COALESCE((
                   SELECT string_agg(signature, ', ' ORDER BY signature)
                     FROM resolved_transport_functions
                    WHERE function_oid IS NULL
               ), '') AS missing_transport_function_signatures
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("inspect the BI runtime's cross-domain role and function privileges");

    let mut failures = Vec::new();
    for (column, expectation) in [
        (
            "transport_runtime_role_exists",
            "the platform transport runtime role exists in the migrated database",
        ),
        (
            "all_transport_functions_exist",
            "all seven platform transport functions exist with the reviewed signatures",
        ),
        (
            "all_principals_are_bi_runtime_members",
            "both session_user and current_user belong to the BI runtime role",
        ),
    ] {
        match row.try_get::<bool, _>(column) {
            Ok(true) => {}
            Ok(false) => failures.push(expectation.to_owned()),
            Err(error) => failures.push(format!("{expectation}: {error}")),
        }
    }

    let unexpected_roles: String = row
        .try_get("unexpected_star_role_memberships")
        .expect("read unexpected Star role memberships");
    if !unexpected_roles.is_empty() {
        failures.push(format!(
            "BI principals have non-allowlisted Star role membership or SET paths: {unexpected_roles}"
        ));
    }

    let unexpected_functions: String = row
        .try_get("unexpected_transport_function_execute")
        .expect("read unexpected platform transport EXECUTE privileges");
    if !unexpected_functions.is_empty() {
        failures.push(format!(
            "BI principals have cross-domain platform transport EXECUTE: {unexpected_functions}"
        ));
    }

    let missing_functions: String = row
        .try_get("missing_transport_function_signatures")
        .expect("read missing platform transport function signatures");
    if !missing_functions.is_empty() {
        failures.push(format!(
            "platform transport migrations are incomplete; missing functions: {missing_functions}"
        ));
    }
    failures
}

async fn require_owner_set_role_denied(pool: &PgPool) {
    let mut tx = pool
        .begin()
        .await
        .expect("start the SET ROLE probe transaction");
    let attempt = sqlx::query("SET ROLE star_bi_owner")
        .execute(&mut *tx)
        .await;
    tx.rollback()
        .await
        .expect("roll back the SET ROLE probe transaction");
    let error = attempt.expect_err("the authenticated runtime session must not SET ROLE to owner");
    let code = error
        .as_database_error()
        .and_then(|database| database.code());
    assert_eq!(
        code.as_deref(),
        Some("42501"),
        "SET ROLE to star_bi_owner must fail with insufficient_privilege"
    );
}

async fn require_tenant_mismatch_denied(pool: &PgPool) {
    let fixture = CommitFixture::new();
    let active_tenant_id = Uuid::new_v4();
    assert_ne!(fixture.tenant_id, active_tenant_id);

    let mut tx = pool
        .begin()
        .await
        .expect("start the tenant-mismatch probe transaction");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(active_tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("scope the tenant-mismatch transaction");
    let attempt = call_commit(&mut tx, &fixture).await;
    tx.rollback()
        .await
        .expect("roll back the tenant-mismatch probe before asserting its result");

    let error = attempt.expect_err("a tenant mismatch must be rejected before commit");
    let code = error
        .as_database_error()
        .and_then(|database| database.code());
    assert_eq!(
        code.as_deref(),
        Some("42501"),
        "tenant mismatch must fail with insufficient_privilege"
    );
}

async fn require_rollback_discards_commit(pool: &PgPool) {
    let fixture = CommitFixture::new();
    let mut tx = pool
        .begin()
        .await
        .expect("start the commit rollback probe transaction");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(fixture.tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("scope the commit rollback transaction");
    sqlx::query("SAVEPOINT before_bi_commit_probe")
        .execute(&mut *tx)
        .await
        .expect("create a savepoint before the BI owner call");

    let first_outcome = call_commit(&mut tx, &fixture)
        .await
        .and_then(|row| row.try_get::<String, _>("outcome"));
    let second_outcome = if matches!(first_outcome.as_deref(), Ok("committed")) {
        let rolled_back_to_savepoint = sqlx::query("ROLLBACK TO SAVEPOINT before_bi_commit_probe")
            .execute(&mut *tx)
            .await
            .is_ok();
        if rolled_back_to_savepoint {
            Some(
                call_commit(&mut tx, &fixture)
                    .await
                    .and_then(|row| row.try_get::<String, _>("outcome")),
            )
        } else {
            None
        }
    } else {
        None
    };
    tx.rollback()
        .await
        .expect("roll back every commit rollback probe write");

    assert_eq!(
        first_outcome.expect("the first owner call should return an outcome"),
        "committed",
        "the fixture should be accepted inside the transaction"
    );
    assert_eq!(
        second_outcome
            .expect("the savepoint rollback should allow a second owner call")
            .expect("the second owner call should return an outcome"),
        "committed",
        "a savepoint rollback must discard the first uncommitted receipt and fact"
    );
}

struct CommitFixture {
    tenant_id: Uuid,
    project_id: Uuid,
    run_id: Uuid,
    event_id: Uuid,
    work_item_id: Uuid,
    actor_id: Uuid,
    correlation_id: Uuid,
    sender_service_id: Uuid,
    receiver_service_id: Uuid,
    transport_profile_id: Uuid,
    destination_generation: Uuid,
    occurred_at: DateTime<Utc>,
    payload: Value,
}

impl CommitFixture {
    fn new() -> Self {
        let tenant_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let work_item_id = Uuid::new_v4();
        let payload = json!({
            "schema_version": 3,
            "aggregate": {"type": "lifecycle", "version": 7},
            "work_item_id": work_item_id.to_string(),
            "owner": {
                "tenant_id": tenant_id.to_string(),
                "project_id": project_id.to_string(),
                "engineering_run_id": run_id.to_string()
            },
            "lifecycle": {"status": "in_progress", "review_state": "none"},
            "change": {
                "event_type": "task.lifecycle.changed",
                "from_status": "claimed",
                "to_status": "in_progress",
                "private_note": "must not be retained by BI"
            },
            "host_context": {"branch_name": "grants-probe-only"}
        });

        Self {
            tenant_id,
            project_id,
            run_id,
            event_id,
            work_item_id,
            actor_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            sender_service_id: Uuid::new_v4(),
            receiver_service_id: Uuid::new_v4(),
            transport_profile_id: Uuid::new_v4(),
            destination_generation: Uuid::new_v4(),
            occurred_at: Utc::now(),
            payload,
        }
    }
}

async fn call_commit(
    tx: &mut Transaction<'_, Postgres>,
    fixture: &CommitFixture,
) -> Result<PgRow, sqlx::Error> {
    sqlx::query(COMMIT_SQL)
        .bind(fixture.tenant_id)
        .bind(fixture.project_id)
        .bind(fixture.run_id)
        .bind(fixture.event_id)
        .bind(fixture.work_item_id)
        .bind("lifecycle")
        .bind(7_i32)
        .bind("task.lifecycle.changed")
        .bind(3_i32)
        .bind(fixture.actor_id)
        .bind(fixture.correlation_id)
        .bind(fixture.occurred_at.clone())
        .bind(fixture.payload.clone())
        .bind(fixture.sender_service_id)
        .bind(fixture.receiver_service_id)
        .bind("star-rust-log")
        .bind("1.0.0")
        .bind(fixture.transport_profile_id)
        .bind(1_i64)
        .bind(vec![0x42_u8; 32])
        .bind("bi-event-grants-pg-test")
        .bind("bi-task-events")
        .bind(fixture.destination_generation)
        .bind(vec![0x24_u8; 32])
        .bind(vec![0x53_u8; 32])
        .bind("star-rust-log")
        .bind("bi-task-events")
        .bind(fixture.destination_generation)
        .bind(17_i64)
        .bind(17_i64.to_be_bytes().to_vec())
        .bind(fixture.event_id)
        .fetch_one(&mut **tx)
        .await
}
