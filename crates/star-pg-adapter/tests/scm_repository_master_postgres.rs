/*
@cypher schema=1 source_sha256=b702add8b1c960b4e846d9b861f23d7a2eb7c4b288f1c410819fa0da6030b18f
MERGE (self:File {path:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs"})
MERGE (connect:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::connect_pools",kind:"function"})
MERGE (identity:Type {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::TargetIdentity",kind:"struct"})
MERGE (confirmation:Type {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::DisposableTargetConfirmation",kind:"struct"})
MERGE (confirm:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::disposable_target_confirmation",kind:"function"})
MERGE (guard:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::validate_target_pair",kind:"function"})
MERGE (migrationGuard:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::validate_migration_identity",kind:"function"})
MERGE (verify:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::target_identity",kind:"function"})
MERGE (tryApply:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::try_apply_repository_master_migration",kind:"function"})
MERGE (targetGuard:Type {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::DisposableTargetGuard",kind:"struct"})
MERGE (guardTest:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::disposable_target_guard_rejects_mismatched_real_identity_facts",kind:"test"})
MERGE (apply:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::apply_repository_master_migration",kind:"function"})
MERGE (scoped:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::scoped_repository_count",kind:"function"})
MERGE (metadata:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::metadata",kind:"function"})
MERGE (test:Symbol {id:"crates/star-pg-adapter/tests/scm_repository_master_postgres.rs::scm_repository_master_is_tenant_scoped_scd2_and_append_only",kind:"test"})
MERGE (migration:Config {id:"db/migrations/2026-10-23-scm-repository-master.sql",kind:"sql_migration"})
MERGE (create:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::create",kind:"method"})
MERGE (find:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::find_current",kind:"method"})
MERGE (revise:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::revise",kind:"method"})
MERGE (repository:Type {id:"crates/domain-scm/src/lib.rs::RepositoryMasterMetadata",kind:"struct"})
MERGE (pool:ExternalService {id:"sqlx.PgPool",kind:"type"})
MERGE (poolConnection:ExternalService {id:"sqlx.PoolConnection<Postgres>",kind:"type"})
MERGE (transaction:ExternalService {id:"sqlx.Transaction<Postgres>",kind:"type"})
MERGE (connection:ExternalService {id:"sqlx.PgConnection",kind:"type"})
MERGE (master:Table {id:"scm.repository_master",kind:"M_SCD2"})
MERGE (event:Table {id:"scm.repository_master_event",kind:"T_append_only"})
MERGE (fixture:Table {id:"star_test.disposable_database_target",kind:"test_fixture"})
MERGE (public_probe:Table {id:"pg_namespace.wld1_scm_public_*",kind:"temporary_test_fixture"})
MERGE (drift_probe:Table {id:"pg_namespace.wld1_scm_drift_*",kind:"temporary_test_fixture"})
MERGE (owner_role:Type {id:"pg_authid.star_scm_repository_owner",kind:"database_role"})
MERGE (self)-[:DEFINES]->(connect)
MERGE (self)-[:DEFINES]->(identity)
MERGE (self)-[:DEFINES]->(confirmation)
MERGE (self)-[:DEFINES]->(confirm)
MERGE (self)-[:DEFINES]->(guard)
MERGE (self)-[:DEFINES]->(migrationGuard)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(tryApply)
MERGE (self)-[:DEFINES]->(targetGuard)
MERGE (self)-[:DEFINES]->(guardTest)
MERGE (self)-[:DEFINES]->(apply)
MERGE (self)-[:DEFINES]->(scoped)
MERGE (self)-[:DEFINES]->(metadata)
MERGE (self)-[:DEFINES]->(test)
MERGE (connect)-[:USES_TYPE]->(pool)
MERGE (connect)-[:USES_TYPE]->(connection)
MERGE (connect)-[:USES_TYPE]->(targetGuard)
MERGE (connect)-[:CALLS]->(confirm)
MERGE (connect)-[:CALLS]->(verify)
MERGE (connect)-[:CALLS]->(guard)
MERGE (verify)-[:USES_TYPE]->(identity)
MERGE (verify)-[:USES_TYPE]->(connection)
MERGE (verify)-[:READS]->(fixture)
MERGE (guard)-[:USES_TYPE]->(identity)
MERGE (guard)-[:USES_TYPE]->(confirmation)
MERGE (migrationGuard)-[:CALLS]->(guard)
MERGE (migrationGuard)-[:USES_TYPE]->(targetGuard)
MERGE (tryApply)-[:READS]->(migration)
MERGE (tryApply)-[:CALLS]->(verify)
MERGE (tryApply)-[:CALLS]->(migrationGuard)
MERGE (tryApply)-[:USES_TYPE]->(pool)
MERGE (tryApply)-[:USES_TYPE]->(poolConnection)
MERGE (tryApply)-[:USES_TYPE]->(transaction)
MERGE (tryApply)-[:USES_TYPE]->(connection)
MERGE (tryApply)-[:USES_TYPE]->(targetGuard)
MERGE (guardTest)-[:CALLS]->(guard)
MERGE (apply)-[:READS]->(migration)
MERGE (apply)-[:CALLS]->(tryApply)
MERGE (scoped)-[:READS]->(master)
MERGE (scoped)-[:USES_TYPE]->(pool)
MERGE (metadata)-[:USES_TYPE]->(repository)
MERGE (test)-[:CALLS]->(connect)
MERGE (test)-[:CALLS]->(apply)
MERGE (test)-[:CALLS]->(tryApply)
MERGE (test)-[:CALLS]->(verify)
MERGE (test)-[:CALLS]->(guard)
MERGE (test)-[:CALLS]->(scoped)
MERGE (test)-[:CALLS]->(metadata)
MERGE (test)-[:CALLS]->(create)
MERGE (test)-[:CALLS]->(find)
MERGE (test)-[:CALLS]->(revise)
MERGE (test)-[:CALLS]->(:ExternalService {id:"tokio::spawn",kind:"function"})
MERGE (test)-[:WRITES]->(public_probe)
MERGE (test)-[:WRITES]->(drift_probe)
MERGE (test)-[:WRITES]->(owner_role)
MERGE (test)-[:READS]->(master)
MERGE (test)-[:READS]->(event)
MERGE (test)-[:WRITES]->(event)
MERGE (test)-[:TESTS]->(master)
MERGE (test)-[:TESTS]->(migration)
MERGE (test)-[:TESTS]->(guard)
MERGE (test)-[:TESTS]->(event);
@endcypher
*/

//! Disposable PostgreSQL acceptance tests for the SCM Repository master owner adapter.

use domain_scm::{RepositoryId, RepositoryMasterMetadata, TenantId, UserId};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgConnection, PgPool};
use star_pg_adapter::repository::{
    PgScmRepositoryMasterRepository, ScmRepositoryMasterError, ScmRepositoryMasterRepository,
};
use std::net::IpAddr;
use uuid::Uuid;

const REPOSITORY_MASTER_MIGRATION: &str =
    include_str!("../../../db/migrations/2026-10-23-scm-repository-master.sql");

/// The fixture must create a fresh local cluster and a unique database named
/// star_scm_test_<suffix>, then provision one readable marker row at
/// star_test.disposable_database_target(singleton BOOLEAN PRIMARY KEY, target_token UUID).
/// This gate proves the selected database and loopback endpoint match the operator-provisioned
/// fixture marker; it cannot independently prove that the containing VM/cluster is disposable.
#[derive(Debug, Clone)]
struct DisposableTargetConfirmation {
    database_name: String,
    target_token: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetIdentity {
    session_user: String,
    current_user: String,
    database_name: String,
    server_address: IpAddr,
    server_port: i32,
    target_token: Uuid,
    is_superuser: bool,
    can_create_roles: bool,
}

fn disposable_target_confirmation() -> DisposableTargetConfirmation {
    assert_eq!(
        std::env::var("STAR_SCM_REPOSITORY_DISPOSABLE_DATABASE").unwrap_or_default(),
        "1",
        "set the first explicit opt-in only for a disposable local fixture"
    );
    assert_eq!(
        std::env::var("STAR_SCM_REPOSITORY_DISPOSABLE_CONFIRM").unwrap_or_default(),
        "I_CONFIRM_THIS_LOCAL_DATABASE_AND_CLUSTER_ARE_DISPOSABLE",
        "the second explicit opt-in must confirm this test database and cluster are disposable"
    );
    let database_name = std::env::var("STAR_SCM_REPOSITORY_CONFIRM_DATABASE")
        .expect("confirm the exact disposable test database name");
    assert!(
        database_name.starts_with("star_scm_test_"),
        "the confirmed target must use the dedicated star_scm_test_ database prefix"
    );
    let target_token = Uuid::parse_str(
        &std::env::var("STAR_SCM_REPOSITORY_DISPOSABLE_TARGET_TOKEN")
            .expect("the isolated fixture supplies its disposable target token"),
    )
    .expect("the disposable target token must be a UUID");
    assert!(
        !target_token.is_nil(),
        "the disposable target token must be nonzero"
    );
    DisposableTargetConfirmation {
        database_name,
        target_token,
    }
}

async fn target_identity(connection: &mut PgConnection) -> TargetIdentity {
    let (
        session_user,
        current_user,
        database_name,
        server_address,
        server_port,
        is_superuser,
        can_create_roles,
    ): (String, String, String, Option<String>, Option<i32>, bool, bool) = sqlx::query_as(
        "SELECT session_user::text, current_user::text, current_database(), inet_server_addr()::text, inet_server_port(), login_role.rolsuper, (login_role.rolsuper OR login_role.rolcreaterole) FROM pg_roles AS login_role WHERE login_role.rolname = session_user",
    )
    .fetch_one(&mut *connection)
    .await
    .expect("read actual PostgreSQL login, database and accepted server endpoint");
    let target_token = sqlx::query_scalar::<_, Uuid>(
        "SELECT target_token FROM star_test.disposable_database_target WHERE singleton IS TRUE",
    )
    .fetch_one(&mut *connection)
    .await
    .expect("read the pre-provisioned disposable-cluster fixture marker before migration");
    TargetIdentity {
        session_user,
        current_user,
        database_name,
        server_address: server_address
            .expect("require a TCP endpoint; Unix sockets do not prove the fixture host")
            .parse()
            .expect("PostgreSQL must report its actual server IP address"),
        server_port: server_port.expect("PostgreSQL must report its accepted TCP port"),
        target_token,
        is_superuser,
        can_create_roles,
    }
}

fn validate_target_pair(
    admin: &TargetIdentity,
    runtime: &TargetIdentity,
    expected: &DisposableTargetConfirmation,
) -> Result<(), &'static str> {
    if admin.session_user == runtime.session_user {
        return Err("admin and runtime must use different actual login roles");
    }
    if admin.current_user != admin.session_user || runtime.current_user != runtime.session_user {
        return Err("pre-migration current_user must equal each actual session_user");
    }
    if !admin.can_create_roles {
        return Err("admin login cannot create the migration's dedicated role");
    }
    if !admin.is_superuser {
        return Err(
            "admin login must be a superuser for isolated migration and cross-tenant assertions",
        );
    }
    if runtime.is_superuser {
        return Err("runtime login must not be a superuser");
    }
    if admin.database_name != runtime.database_name
        || admin.database_name != expected.database_name
        || !admin.database_name.starts_with("star_scm_test_")
    {
        return Err("both logins must reach the explicitly confirmed disposable database");
    }
    if !admin.server_address.is_loopback()
        || !runtime.server_address.is_loopback()
        || admin.server_address != runtime.server_address
        || admin.server_port != runtime.server_port
    {
        return Err("both logins must reach the same loopback PostgreSQL endpoint");
    }
    if admin.target_token != expected.target_token || runtime.target_token != expected.target_token
    {
        return Err("both logins must read the explicitly confirmed fixture token");
    }
    Ok(())
}

fn validate_migration_identity(
    actual_admin: &TargetIdentity,
    target_guard: &DisposableTargetGuard,
) -> Result<(), &'static str> {
    validate_target_pair(
        actual_admin,
        &target_guard.runtime_preflight,
        &target_guard.confirmation,
    )?;
    if actual_admin != &target_guard.admin_preflight {
        return Err("admin migration session differs from the preflight identity");
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct DisposableTargetGuard {
    confirmation: DisposableTargetConfirmation,
    admin_preflight: TargetIdentity,
    runtime_preflight: TargetIdentity,
}

async fn connect_pools() -> (PgPool, PgPool, DisposableTargetGuard) {
    // All opt-ins and live target checks run before the caller can execute migration SQL.
    let confirmation = disposable_target_confirmation();
    let admin_url = std::env::var("STAR_SCM_REPOSITORY_ADMIN_DATABASE_URL")
        .expect("the disposable PostgreSQL fixture supplies an admin URL");
    let runtime_url = std::env::var("STAR_SCM_REPOSITORY_TEST_DATABASE_URL")
        .expect("the disposable PostgreSQL fixture supplies a non-superuser owner-member URL");

    let admin = PgPoolOptions::new()
        // Bound fixture resource use; migration safety comes from rechecking on its own transaction.
        .max_connections(1)
        .connect(&admin_url)
        .await
        .expect("the confirmed disposable admin pool connects");
    let runtime = PgPoolOptions::new()
        .max_connections(4)
        .connect(&runtime_url)
        .await
        .expect("the confirmed disposable runtime pool connects");

    let mut admin_connection = admin
        .acquire()
        .await
        .expect("the confirmed disposable admin connection acquires");
    let mut runtime_connection = runtime
        .acquire()
        .await
        .expect("the confirmed disposable runtime connection acquires");
    let (admin_identity, runtime_identity) = tokio::join!(
        target_identity(&mut *admin_connection),
        target_identity(&mut *runtime_connection)
    );
    validate_target_pair(&admin_identity, &runtime_identity, &confirmation)
        .expect("reject an unverified target before applying any migration");
    (
        admin,
        runtime,
        DisposableTargetGuard {
            confirmation,
            admin_preflight: admin_identity,
            runtime_preflight: runtime_identity,
        },
    )
}

#[test]
fn disposable_target_guard_rejects_mismatched_real_identity_facts() {
    let token = Uuid::new_v4();
    let confirmation = DisposableTargetConfirmation {
        database_name: "star_scm_test_fixture_a1b2".to_string(),
        target_token: token,
    };
    let admin = TargetIdentity {
        session_user: "fixture_admin".to_string(),
        current_user: "fixture_admin".to_string(),
        database_name: confirmation.database_name.clone(),
        server_address: "127.0.0.1".parse().unwrap(),
        server_port: 5432,
        target_token: token,
        is_superuser: true,
        can_create_roles: true,
    };
    let mut runtime = TargetIdentity {
        session_user: "fixture_runtime".to_string(),
        current_user: "fixture_runtime".to_string(),
        database_name: confirmation.database_name.clone(),
        server_address: "127.0.0.1".parse().unwrap(),
        server_port: 5432,
        target_token: token,
        is_superuser: false,
        can_create_roles: false,
    };

    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_ok());
    runtime.database_name = "production".to_string();
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.database_name = confirmation.database_name.clone();
    runtime.server_address = "192.0.2.10".parse().unwrap();
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.server_address = "127.0.0.1".parse().unwrap();
    runtime.current_user = "switched_role".to_string();
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.current_user = runtime.session_user.clone();
    runtime.server_port = 5433;
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.server_port = 5432;
    runtime.session_user = admin.session_user.clone();
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.session_user = "fixture_runtime".to_string();
    runtime.target_token = Uuid::new_v4();
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.target_token = token;
    runtime.is_superuser = true;
    assert!(validate_target_pair(&admin, &runtime, &confirmation).is_err());
    runtime.is_superuser = false;

    let target_guard = DisposableTargetGuard {
        confirmation: confirmation.clone(),
        admin_preflight: admin.clone(),
        runtime_preflight: runtime.clone(),
    };
    assert!(validate_migration_identity(&admin, &target_guard).is_ok());
    let mut replacement_endpoint = admin.clone();
    replacement_endpoint.server_address = "127.0.0.2".parse().unwrap();
    assert!(
        validate_migration_identity(&replacement_endpoint, &target_guard).is_err(),
        "migration must reject a replaced PostgreSQL server endpoint"
    );
    replacement_endpoint.server_address = admin.server_address;
    replacement_endpoint.server_port = 5433;
    assert!(
        validate_migration_identity(&replacement_endpoint, &target_guard).is_err(),
        "migration must reject a replaced PostgreSQL server port"
    );

    let mut underprivileged_admin = admin.clone();
    underprivileged_admin.is_superuser = false;
    assert!(validate_target_pair(&underprivileged_admin, &runtime, &confirmation).is_err());
    underprivileged_admin.is_superuser = true;
    underprivileged_admin.can_create_roles = false;
    assert!(validate_target_pair(&underprivileged_admin, &runtime, &confirmation).is_err());
}

async fn apply_repository_master_migration(admin: &PgPool, target_guard: &DisposableTargetGuard) {
    try_apply_repository_master_migration(admin, target_guard)
        .await
        .expect("apply every physical migration statement in one isolated transaction");
}

async fn try_apply_repository_master_migration(
    admin: &PgPool,
    target_guard: &DisposableTargetGuard,
) -> Result<(), String> {
    // Hold one explicit physical connection for the identity check and every DDL statement.
    // max_connections(1) only bounds resources; it does not provide connection affinity.
    let mut migration_connection = admin.acquire().await.map_err(|error| error.to_string())?;
    let mut tx = (&mut *migration_connection)
        .begin()
        .await
        .map_err(|error| error.to_string())?;
    let migration_identity = target_identity(&mut *tx).await;
    let target_check = validate_migration_identity(&migration_identity, target_guard);
    if let Err(reason) = target_check {
        let reason = reason.to_owned();
        return match tx.rollback().await {
            Ok(()) => Err(reason),
            Err(rollback_error) => Err(format!(
                "{reason}; target-check rollback failed: {rollback_error}"
            )),
        };
    }
    for statement in REPOSITORY_MASTER_MIGRATION.lines() {
        let statement = statement.trim();
        if statement.is_empty() || statement.starts_with("--") {
            continue;
        }
        if let Err(error) = sqlx::query(statement).execute(&mut *tx).await {
            return match tx.rollback().await {
                Ok(()) => Err(error.to_string()),
                Err(rollback_error) => Err(format!(
                    "migration failed: {error}; rollback failed: {rollback_error}"
                )),
            };
        }
    }
    tx.commit().await.map_err(|error| error.to_string())
}

async fn scoped_repository_count(
    runtime: &PgPool,
    tenant_id: TenantId,
    repository_id: RepositoryId,
) -> (i64, bool) {
    let mut tx = runtime
        .begin()
        .await
        .expect("begin owner-scoped read transaction");
    sqlx::query("SET LOCAL ROLE star_scm_repository_owner")
        .execute(&mut *tx)
        .await
        .expect("enter the SCM owner role");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set transaction tenant scope");
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM scm.repository_master WHERE repository_id = $1",
    )
    .bind(repository_id.as_uuid())
    .fetch_one(&mut *tx)
    .await
    .expect("read repositories through tenant RLS without a tenant predicate");
    let rls_active = sqlx::query_scalar::<_, bool>(
        "SELECT row_security_active('scm.repository_master'::regclass)",
    )
    .fetch_one(&mut *tx)
    .await
    .expect("inspect effective row security under the owner role");
    tx.commit()
        .await
        .expect("commit owner-scoped read transaction");
    (count, rls_active)
}

fn metadata(name: &str) -> RepositoryMasterMetadata {
    RepositoryMasterMetadata {
        name: name.to_string(),
        provider_key: Some("forge".to_string()),
        remote_identity: Some("org/repository".to_string()),
        remote_url: Some("HTTPS://example.test/org/repository.git".to_string()),
        default_branch: Some("main".to_string()),
    }
}

#[tokio::test]
#[ignore = "requires a disposable PostgreSQL fixture with a non-superuser member of star_scm_repository_owner"]
async fn scm_repository_master_is_tenant_scoped_scd2_and_append_only() {
    let (admin, runtime, target_guard) = connect_pools().await;
    let public_probe = format!("wld1_scm_public_{}", Uuid::new_v4().simple());
    let admin_public_probe = admin.clone();
    let target_guard_for_bootstrap = target_guard.clone();
    let public_probe_for_task = public_probe.clone();
    let bootstrap_result = tokio::spawn(async move {
        let bootstrap_result: Result<(), String> = async {
            sqlx::query(&format!("CREATE SCHEMA {public_probe_for_task}"))
                .execute(&admin_public_probe)
                .await
                .map_err(|error| error.to_string())?;
            sqlx::query(&format!(
                "GRANT USAGE ON SCHEMA {public_probe_for_task} TO PUBLIC"
            ))
            .execute(&admin_public_probe)
            .await
            .map_err(|error| error.to_string())?;
            apply_repository_master_migration(&admin_public_probe, &target_guard_for_bootstrap)
                .await;
            apply_repository_master_migration(&admin_public_probe, &target_guard_for_bootstrap)
                .await;
            Ok(())
        }
        .await;
        bootstrap_result
    })
    .await;
    let drop_public_sql = format!("DROP SCHEMA IF EXISTS {public_probe}");
    let public_probe_cleanup = sqlx::query(&drop_public_sql).execute(&admin).await;
    let mut bootstrap_errors = Vec::new();
    match bootstrap_result {
        Err(error) => bootstrap_errors.push(format!("PUBLIC-grant bootstrap task failed: {error}")),
        Ok(Err(error)) => {
            bootstrap_errors.push(format!("PUBLIC-grant bootstrap operations failed: {error}"))
        }
        Ok(Ok(())) => {}
    }
    if let Err(error) = public_probe_cleanup {
        bootstrap_errors.push(format!(
            "remove the disposable PUBLIC-only grant probe: {error}"
        ));
    }
    assert!(
        bootstrap_errors.is_empty(),
        "PUBLIC-grant bootstrap and cleanup failures: {bootstrap_errors:?}"
    );

    let runtime_is_superuser =
        sqlx::query_scalar::<_, bool>("SELECT rolsuper FROM pg_roles WHERE rolname = current_user")
            .fetch_one(&runtime)
            .await
            .expect("inspect runtime role attributes");
    assert!(
        !runtime_is_superuser,
        "runtime test role must not bypass RLS"
    );
    let can_set_owner_role = sqlx::query_scalar::<_, bool>(
        "SELECT pg_has_role(current_user, 'star_scm_repository_owner', 'MEMBER')",
    )
    .fetch_one(&runtime)
    .await
    .expect("inspect owner-role membership");
    assert!(
        can_set_owner_role,
        "runtime test role must be provisioned as an owner-role member"
    );
    let owner_role_attributes: (bool, bool, bool, bool, bool, bool, bool) = sqlx::query_as(
        "SELECT rolcanlogin, rolsuper, rolcreatedb, rolcreaterole, rolreplication, rolbypassrls, rolinherit FROM pg_roles WHERE rolname = 'star_scm_repository_owner'",
    )
    .fetch_one(&admin)
    .await
    .expect("inspect the dedicated owner-role boundary");
    assert_eq!(
        owner_role_attributes,
        (false, false, false, false, false, false, false),
        "the owner role must remain NOLOGIN, unprivileged, and non-inheriting"
    );
    let owner_has_parent_roles: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_auth_members WHERE member = 'star_scm_repository_owner'::regrole)",
    )
    .fetch_one(&admin)
    .await
    .expect("inspect owner-role memberships");
    assert!(
        !owner_has_parent_roles,
        "the owner role must not be able to assume broader roles"
    );

    let tenant_id = TenantId::new();
    let other_tenant_id = TenantId::new();
    let repository_id = RepositoryId::new();
    let actor_id = UserId::new();
    let repository = PgScmRepositoryMasterRepository::new(runtime.clone());

    let (left, right) = tokio::join!(
        repository.create(
            tenant_id,
            repository_id,
            metadata("canonical-repository"),
            actor_id,
            Uuid::new_v4(),
        ),
        repository.create(
            tenant_id,
            repository_id,
            metadata("canonical-repository"),
            actor_id,
            Uuid::new_v4(),
        ),
    );
    let created = match (left, right) {
        (Ok(revision), Err(ScmRepositoryMasterError::AlreadyExists))
        | (Err(ScmRepositoryMasterError::AlreadyExists), Ok(revision)) => revision,
        (left, right) => panic!("one serialized create must win: {left:?}; {right:?}"),
    };
    assert_eq!(created.version, 1);
    assert_eq!(created.created_by, actor_id);

    let mut orphaned_close = runtime
        .begin()
        .await
        .expect("begin a direct owner-role revision close");
    sqlx::query("SET LOCAL ROLE star_scm_repository_owner")
        .execute(&mut *orphaned_close)
        .await
        .expect("enter the SCM owner role");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *orphaned_close)
        .await
        .expect("set transaction tenant scope");
    sqlx::query("SELECT set_config('app.actor_id', $1, true)")
        .bind(actor_id.to_string())
        .execute(&mut *orphaned_close)
        .await
        .expect("set transaction actor scope");
    let closed = sqlx::query(
        "UPDATE scm.repository_master SET valid_to = transaction_timestamp() WHERE tenant_id = $1 AND repository_id = $2 AND version = 1 AND valid_to IS NULL",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .execute(&mut *orphaned_close)
    .await
    .expect("owner role may attempt the narrow current-revision close");
    assert_eq!(closed.rows_affected(), 1);
    assert!(
        orphaned_close.commit().await.is_err(),
        "the deferred invariant must reject closing the only current revision at commit"
    );
    let current_after_rejected_close = repository
        .find_current(tenant_id, repository_id)
        .await
        .unwrap()
        .expect("failed commit must roll back the orphaned close");
    assert_eq!(current_after_rejected_close.version, 1);
    let audit_after_rejected_close: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM scm.repository_master_event WHERE tenant_id = $1 AND repository_id = $2",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .fetch_one(&admin)
    .await
    .expect("inspect audit after rejected close");
    assert_eq!(
        audit_after_rejected_close, 1,
        "rejected close must leave no partial audit or history change"
    );

    let mut denied_update = runtime
        .begin()
        .await
        .expect("begin restricted owner-role write");
    sqlx::query("SET LOCAL ROLE star_scm_repository_owner")
        .execute(&mut *denied_update)
        .await
        .expect("enter the SCM owner role");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *denied_update)
        .await
        .expect("set transaction tenant scope");
    let direct_metadata_update = sqlx::query(
        "UPDATE scm.repository_master SET name = 'bypass-adapter' WHERE tenant_id = $1 AND repository_id = $2 AND valid_to IS NULL",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .execute(&mut *denied_update)
    .await;
    assert!(
        direct_metadata_update.is_err(),
        "the owner role may close a revision but must not mutate metadata columns"
    );
    denied_update
        .rollback()
        .await
        .expect("rollback the rejected direct metadata mutation");

    assert_eq!(
        repository
            .find_current(other_tenant_id, repository_id)
            .await
            .unwrap(),
        None,
        "tenant-scoped repository reads must not reveal another tenant's identity"
    );
    assert_eq!(
        repository
            .revise(
                tenant_id,
                repository_id,
                8,
                metadata("stale-write"),
                actor_id,
                Uuid::new_v4(),
            )
            .await
            .unwrap_err(),
        ScmRepositoryMasterError::VersionConflict
    );
    assert_eq!(
        repository
            .revise(
                other_tenant_id,
                repository_id,
                1,
                metadata("cross-tenant-write"),
                UserId::new(),
                Uuid::new_v4(),
            )
            .await
            .unwrap_err(),
        ScmRepositoryMasterError::NotFound
    );

    let revision_actor_id = UserId::new();
    let revised = repository
        .revise(
            tenant_id,
            repository_id,
            1,
            metadata("canonical-repository-renamed"),
            revision_actor_id,
            Uuid::new_v4(),
        )
        .await
        .unwrap();
    assert_eq!(revised.version, 2);
    assert_eq!(revised.created_by, created.created_by);
    assert_eq!(revised.changed_by, revision_actor_id);
    assert_eq!(
        repository
            .revise(
                tenant_id,
                repository_id,
                1,
                metadata("stale-write"),
                actor_id,
                Uuid::new_v4(),
            )
            .await
            .unwrap_err(),
        ScmRepositoryMasterError::VersionConflict
    );

    let (own_rows, own_rls) = scoped_repository_count(&runtime, tenant_id, repository_id).await;
    let (foreign_rows, foreign_rls) =
        scoped_repository_count(&runtime, other_tenant_id, repository_id).await;
    assert!(
        own_rls && foreign_rls,
        "FORCE RLS must remain active under the owner role"
    );
    assert_eq!(own_rows, 1);
    assert_eq!(
        foreign_rows, 0,
        "tenant RLS must hide rows without a tenant SQL predicate"
    );

    let (closed_revisions, current_revisions): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE valid_to IS NOT NULL), count(*) FILTER (WHERE valid_to IS NULL) FROM scm.repository_master WHERE tenant_id = $1 AND repository_id = $2",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .fetch_one(&admin)
    .await
    .expect("inspect SCD2 revision history");
    assert_eq!((closed_revisions, current_revisions), (1, 1));

    let audit_rows: Vec<(i32, String, Uuid)> = sqlx::query_as(
        "SELECT repository_version, action, actor_id FROM scm.repository_master_event WHERE tenant_id = $1 AND repository_id = $2 ORDER BY repository_version",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .fetch_all(&admin)
    .await
    .expect("inspect append-only audit events");
    assert_eq!(audit_rows.len(), 2);
    assert_eq!(
        audit_rows[0],
        (1, "created".to_string(), actor_id.as_uuid())
    );
    assert_eq!(
        audit_rows[1],
        (2, "revised".to_string(), revision_actor_id.as_uuid())
    );

    let mutation = sqlx::query(
        "UPDATE scm.repository_master_event SET action = 'revised' WHERE tenant_id = $1 AND repository_id = $2 AND repository_version = 1",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .execute(&admin)
    .await;
    assert!(
        mutation.is_err(),
        "database trigger must reject audit mutation"
    );

    let drift_probe = format!("wld1_scm_drift_{}", Uuid::new_v4().simple());
    let admin_probe = admin.clone();
    let target_guard_for_drift_probe = target_guard.clone();
    let drift_probe_task = drift_probe.clone();
    let probe_result = tokio::spawn(async move {
        let probe_result: Result<(String, bool, bool, (i64, i64), (i64, i64)), String> = async {
            sqlx::query(&format!("CREATE SCHEMA {drift_probe_task}"))
                .execute(&admin_probe)
                .await
                .map_err(|error| error.to_string())?;
            sqlx::query("ALTER ROLE star_scm_repository_owner LOGIN")
                .execute(&admin_probe)
                .await
                .map_err(|error| error.to_string())?;
            sqlx::query(&format!(
                "GRANT USAGE ON SCHEMA {drift_probe_task} TO star_scm_repository_owner"
            ))
            .execute(&admin_probe)
            .await
            .map_err(|error| error.to_string())?;
            let counts_before: (i64, i64) = sqlx::query_as(
                "SELECT (SELECT count(*) FROM scm.repository_master), (SELECT count(*) FROM scm.repository_master_event)",
            )
            .fetch_one(&admin_probe)
            .await
            .map_err(|error| error.to_string())?;
            let migration_error = match try_apply_repository_master_migration(
                &admin_probe,
                &target_guard_for_drift_probe,
            )
            .await
            {
                Ok(()) => return Err("migration unexpectedly accepted the external grant".to_owned()),
                Err(error) => error.to_string(),
            };
            let owner_still_login: bool = sqlx::query_scalar(
                "SELECT rolcanlogin FROM pg_roles WHERE rolname = 'star_scm_repository_owner'",
            )
            .fetch_one(&admin_probe)
            .await
            .map_err(|error| error.to_string())?;
            let drift_grant_still_present: bool = sqlx::query_scalar(
                "SELECT has_schema_privilege('star_scm_repository_owner', $1, 'USAGE')",
            )
            .bind(&drift_probe_task)
            .fetch_one(&admin_probe)
            .await
            .map_err(|error| error.to_string())?;
            let counts_after: (i64, i64) = sqlx::query_as(
                "SELECT (SELECT count(*) FROM scm.repository_master), (SELECT count(*) FROM scm.repository_master_event)",
            )
            .fetch_one(&admin_probe)
            .await
            .map_err(|error| error.to_string())?;
            Ok((
                migration_error,
                owner_still_login,
                drift_grant_still_present,
                counts_before,
                counts_after,
            ))
        }
        .await;
        probe_result
    })
    .await;

    // The probe runs in a child task so even an assertion/panic in its body is
    // observed here, after this outer task has attempted fixture cleanup.
    let mut failures = Vec::new();
    if let Err(error) = sqlx::query("ALTER ROLE star_scm_repository_owner NOLOGIN")
        .execute(&admin)
        .await
    {
        failures.push(format!("restore owner role login boundary: {error}"));
    }
    let revoke_drift_sql =
        format!("REVOKE ALL PRIVILEGES ON SCHEMA {drift_probe} FROM star_scm_repository_owner");
    if let Err(error) = sqlx::query(&revoke_drift_sql).execute(&admin).await {
        failures.push(format!("remove injected owner grant: {error}"));
    }
    let drop_drift_sql = format!("DROP SCHEMA IF EXISTS {drift_probe}");
    if let Err(error) = sqlx::query(&drop_drift_sql).execute(&admin).await {
        failures.push(format!("drop privilege-drift probe: {error}"));
    }
    match &probe_result {
        Err(error) => failures.push(format!("privilege-drift probe task failed: {error}")),
        Ok(Err(error)) => {
            failures.push(format!("privilege-drift probe operations failed: {error}"))
        }
        Ok(Ok(_)) => {}
    }
    assert!(
        failures.is_empty(),
        "privilege-drift probe and cleanup failures: {failures:?}"
    );
    let (
        drift_error,
        owner_still_login,
        drift_grant_still_present,
        counts_before_drift_replay,
        counts_after_drift_replay,
    ) = probe_result
        .expect("privilege-drift probe task must not panic")
        .expect("privilege-drift probe operations must succeed");
    assert!(
        drift_error.contains("direct privileges outside the SCM schema"),
        "expected the direct-grant preflight rejection, got: {drift_error}"
    );
    assert!(
        owner_still_login,
        "failed preflight must leave role attributes unchanged"
    );
    assert!(
        drift_grant_still_present,
        "failed migration must not revoke external grants"
    );
    assert_eq!(
        counts_after_drift_replay, counts_before_drift_replay,
        "rejected migration must leave no master or audit rows behind"
    );
}
