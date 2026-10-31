// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=5b7bb51ac4363db43429482515ca18cc698b7abefb13ef5e4e6329d4acb49823
MERGE (self:File {path:"crates/star-api-rest/tests/device_revocation_http_pg.rs"})
MERGE (authorized:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::authenticated_revoke_is_owner_scoped_and_audited",kind:"test"})
MERGE (unauthenticated:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::unauthenticated_revoke_is_rejected",kind:"test"})
MERGE (concurrent:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::concurrent_revoke_has_one_winner_and_one_audit_fact",kind:"test"})
MERGE (rollback:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::audit_failure_rolls_back_device_and_refresh_revocation",kind:"test"})
MERGE (context:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::test_context",kind:"function"})
MERGE (seedDevice:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::seed_device",kind:"function"})
MERGE (seedRefresh:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::seed_refresh",kind:"function"})
MERGE (revokeHttp:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::http_revoke",kind:"function"})
MERGE (role:Symbol {id:"crates/star-api-rest/tests/device_revocation_http_pg.rs::assert_app_role_is_constrained",kind:"function"})
MERGE (oauthRouter:Endpoint {id:"POST /oauth/device/{device_id}/revoke",kind:"http_endpoint"})
MERGE (binding:Table {id:"public.oauth_device_bindings",kind:"table"})
MERGE (refresh:Table {id:"public.oauth_refresh_tokens",kind:"table"})
MERGE (audit:Table {id:"public.oauth_device_binding_audit",kind:"table"})
MERGE (pool:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (adminPool:Config {id:"STAR_W16_DEVICE_AUTH_ADMIN_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (self)-[:DEFINES]->(authorized),(self)-[:DEFINES]->(unauthenticated),(self)-[:DEFINES]->(concurrent),(self)-[:DEFINES]->(rollback),(self)-[:DEFINES]->(context),(self)-[:DEFINES]->(seedDevice),(self)-[:DEFINES]->(seedRefresh),(self)-[:DEFINES]->(revokeHttp),(self)-[:DEFINES]->(role)
MERGE (authorized)-[:CALLS]->(context),(authorized)-[:CALLS]->(seedDevice),(authorized)-[:CALLS]->(seedRefresh),(authorized)-[:CALLS]->(revokeHttp)
MERGE (unauthenticated)-[:CALLS]->(context),(unauthenticated)-[:CALLS]->(revokeHttp)
MERGE (concurrent)-[:CALLS]->(context),(concurrent)-[:CALLS]->(seedDevice),(concurrent)-[:CALLS]->(revokeHttp)
MERGE (rollback)-[:CALLS]->(context),(rollback)-[:CALLS]->(seedDevice),(rollback)-[:CALLS]->(seedRefresh),(rollback)-[:CALLS]->(revokeHttp)
MERGE (context)-[:USES_TYPE]->(:Type {id:"star_api_rest::auth::oauth::OAuth2State",kind:"struct"})
MERGE (context)-[:USES_TYPE]->(:Type {id:"star_pg_adapter::repository::PgDeviceBindingRepository",kind:"struct"})
MERGE (context)-[:READS]->(pool),(context)-[:READS]->(adminPool)
MERGE (seedDevice)-[:WRITES]->(binding)
MERGE (seedRefresh)-[:WRITES]->(refresh)
MERGE (authorized)-[:TESTS]->(oauthRouter),(unauthenticated)-[:TESTS]->(oauthRouter),(concurrent)-[:TESTS]->(oauthRouter),(rollback)-[:TESTS]->(oauthRouter)
MERGE (authorized)-[:READS]->(binding),(authorized)-[:READS]->(refresh),(authorized)-[:READS]->(audit)
MERGE (concurrent)-[:READS]->(audit)
MERGE (rollback)-[:READS]->(binding),(rollback)-[:READS]->(refresh),(rollback)-[:READS]->(audit)
@endcypher
*/
//! PostgreSQL-backed HTTP acceptance tests for owner-scoped device revocation.
#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use star_api_rest::{
    auth::{issue_token_with_device_context, DeviceTokenContext},
    build_oauth_router,
};
use star_pg_adapter::repository::PgDeviceBindingRepository;
use tokio::sync::Barrier;
use tower::ServiceExt;
use uuid::Uuid;

mod common;

use common::MockOAuth2Bundles;

const DATABASE_NAME: &str = "w16a_device_revoke_http_acceptance";
const APP_ROLE: &str = "star_w16_device_revoke_app";

struct TestContext {
    admin_pool: PgPool,
    app_pool: PgPool,
    app: Router,
    jwt_config: Arc<star_api_rest::auth::JwtConfig>,
    tenant_id: Uuid,
    actor_id: Uuid,
    bearer: String,
}

async fn test_context() -> TestContext {
    let admin_url = std::env::var("STAR_W16_DEVICE_AUTH_ADMIN_TEST_DATABASE_URL")
        .expect("runner-owned isolated admin database URL is required");
    let app_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("runner-owned constrained application database URL is required");
    let admin_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&admin_url)
        .await
        .expect("runner-owned PostgreSQL admin endpoint");
    let app_pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&app_url)
        .await
        .expect("runner-owned PostgreSQL application endpoint");
    assert_local_database(&admin_pool, "postgres").await;
    assert_local_database(&app_pool, APP_ROLE).await;
    assert_app_role_is_constrained(&app_pool).await;

    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let session_device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO oauth_device_bindings          (tenant_id, actor_id, device_id, public_key_fingerprint, generation, status)          VALUES ($1,$2,$3,$4,1,'active')",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(session_device_id)
    .bind(fingerprint())
    .execute(&admin_pool)
    .await
    .expect("seed active device context for authenticated route requests");
    let bundles = MockOAuth2Bundles::new();
    let mut state = bundles.build_state().await;
    let jwt_config = Arc::clone(&state.jwt_config);
    state.device_binding_repo = Arc::new(PgDeviceBindingRepository::new(app_pool.clone()));
    let bearer = issue_token_with_device_context(
        &jwt_config,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        "device:revoke".to_owned(),
        Some(DeviceTokenContext {
            device_id: session_device_id,
            generation: 1,
        }),
    )
    .expect("issue signed test bearer")
    .0;
    let app = build_oauth_router().with_state(state);

    TestContext {
        admin_pool,
        app_pool,
        app,
        jwt_config,
        tenant_id,
        actor_id,
        bearer,
    }
}

async fn assert_local_database(pool: &PgPool, expected_user: &str) {
    let (database, address, user): (String, Option<String>, String) =
        sqlx::query_as("SELECT current_database(), host(inet_server_addr()), current_user::text")
            .fetch_one(pool)
            .await
            .expect("read isolated database identity");
    assert_eq!(database, DATABASE_NAME);
    assert_eq!(address.as_deref(), Some("127.0.0.1"));
    assert_eq!(user, expected_user);
}

async fn assert_app_role_is_constrained(pool: &PgPool) {
    let (superuser, bypass_rls, create_db, create_role, inherit, can_assume_owner): (
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = sqlx::query_as(
        "SELECT role.rolsuper, role.rolbypassrls, role.rolcreatedb, role.rolcreaterole,          role.rolinherit, pg_has_role(current_user, 'star_oauth_credential_owner', 'member')          FROM pg_roles AS role WHERE role.rolname=current_user",
    )
    .fetch_one(pool)
    .await
    .expect("inspect application login attributes");
    assert!(!superuser, "application login must not be a superuser");
    assert!(!bypass_rls, "application login must not bypass RLS");
    assert!(!create_db);
    assert!(!create_role);
    assert!(!inherit, "application login must require explicit SET ROLE");
    assert!(
        can_assume_owner,
        "application login may assume only the device owner contract"
    );

    let memberships: Vec<String> = sqlx::query_scalar(
        "SELECT COALESCE(array_agg(role.rolname ORDER BY role.rolname), ARRAY[]::text[])          FROM pg_auth_members AS membership          JOIN pg_roles AS role ON role.oid=membership.roleid          WHERE membership.member=(SELECT oid FROM pg_roles WHERE rolname=current_user)",
    )
    .fetch_one(pool)
    .await
    .expect("inspect direct application role memberships");
    assert_eq!(memberships, vec!["star_oauth_credential_owner".to_owned()]);

    let direct_read = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM oauth_device_bindings")
        .fetch_one(pool)
        .await;
    assert!(
        direct_read.is_err(),
        "login must not inherit direct device table privileges outside the repository transaction"
    );
}

async fn seed_device(
    context: &TestContext,
    tenant_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
    fingerprint: &str,
) {
    sqlx::query(
        "INSERT INTO oauth_device_bindings          (tenant_id, actor_id, device_id, public_key_fingerprint, generation, status)          VALUES ($1,$2,$3,$4,1,'active')",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .bind(fingerprint)
    .execute(&context.admin_pool)
    .await
    .expect("seed disposable active device binding");
}

async fn seed_refresh(
    context: &TestContext,
    tenant_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
) -> Uuid {
    let token_hash = format!("w16-device-revoke-test-{}", Uuid::new_v4().simple());
    sqlx::query_scalar(
        "INSERT INTO oauth_refresh_tokens          (tenant_id, token_hash, client_id, user_id, access_token_id, scope, expires_at, device_id, device_generation)          VALUES ($1,$2,'star-device',$3,$4,'device:revoke',transaction_timestamp()+interval '1 hour',$5,1)          RETURNING id",
    )
    .bind(tenant_id)
    .bind(token_hash)
    .bind(actor_id)
    .bind(Uuid::new_v4())
    .bind(device_id)
    .fetch_one(&context.admin_pool)
    .await
    .expect("seed disposable device-bound refresh token")
}

fn fingerprint() -> String {
    format!("{:064x}", Uuid::new_v4().as_u128())
}

async fn http_revoke(context: &TestContext, device_id: Uuid, bearer: Option<&str>) -> StatusCode {
    let mut builder = Request::builder()
        .method("POST")
        .uri(format!("/oauth/device/{device_id}/revoke"));
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    context
        .app
        .clone()
        .oneshot(builder.body(Body::empty()).expect("build revoke request"))
        .await
        .expect("device revoke route response")
        .status()
}

async fn row_state(
    pool: &PgPool,
    tenant_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
) -> (String, i64, Option<DateTime<Utc>>) {
    sqlx::query_as(
        "SELECT status, generation, revoked_at FROM oauth_device_bindings          WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .fetch_one(pool)
    .await
    .expect("read disposable device row")
}

async fn audit_rows(
    pool: &PgPool,
    tenant_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
) -> Vec<(String, i64, String)> {
    sqlx::query_as(
        "SELECT event_kind, generation, public_key_fingerprint          FROM oauth_device_binding_audit          WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3 ORDER BY generation",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .fetch_all(pool)
    .await
    .expect("read append-only device audit rows")
}

async fn count_refresh(
    pool: &PgPool,
    tenant_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
    revoked: bool,
) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM oauth_refresh_tokens          WHERE tenant_id=$1 AND user_id=$2 AND device_id=$3 AND          (($4 AND revoked_at IS NOT NULL) OR (NOT $4 AND revoked_at IS NULL))",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .bind(revoked)
    .fetch_one(pool)
    .await
    .expect("count disposable refresh rows")
}

#[tokio::test]
#[ignore = "requires runner-owned loopback PostgreSQL 18 and test-only role/audit prerequisites"]
async fn unauthenticated_revoke_is_rejected() {
    let context = test_context().await;
    let device_id = Uuid::new_v4();

    assert_eq!(
        http_revoke(&context, device_id, None).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        http_revoke(&context, device_id, Some("not-a-valid-jwt")).await,
        StatusCode::UNAUTHORIZED
    );
    let missing = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM oauth_device_binding_audit WHERE device_id=$1",
    )
    .bind(device_id)
    .fetch_one(&context.admin_pool)
    .await
    .expect("verify rejected requests did not write audit");
    assert_eq!(missing, 0);
    context.app_pool.close().await;
    context.admin_pool.close().await;
}

#[tokio::test]
#[ignore = "requires runner-owned loopback PostgreSQL 18 and test-only role/audit prerequisites"]
async fn authenticated_revoke_is_owner_scoped_and_audited() {
    let context = test_context().await;
    let device_id = Uuid::new_v4();
    let other_device_id = Uuid::new_v4();
    let other_actor = Uuid::new_v4();
    let public_key_fingerprint = fingerprint();
    seed_device(
        &context,
        context.tenant_id,
        context.actor_id,
        device_id,
        &public_key_fingerprint,
    )
    .await;
    seed_device(
        &context,
        context.tenant_id,
        context.actor_id,
        other_device_id,
        &fingerprint(),
    )
    .await;
    seed_refresh(&context, context.tenant_id, context.actor_id, device_id).await;
    seed_refresh(&context, context.tenant_id, context.actor_id, device_id).await;
    seed_refresh(
        &context,
        context.tenant_id,
        context.actor_id,
        other_device_id,
    )
    .await;

    let other_actor_device_id = Uuid::new_v4();
    seed_device(
        &context,
        context.tenant_id,
        other_actor,
        other_actor_device_id,
        &fingerprint(),
    )
    .await;
    let other_actor_bearer = issue_token_with_device_context(
        &context.jwt_config,
        other_actor,
        context.tenant_id,
        vec!["developer".to_owned()],
        "device:revoke".to_owned(),
        Some(DeviceTokenContext {
            device_id: other_actor_device_id,
            generation: 1,
        }),
    )
    .expect("issue another valid actor token")
    .0;
    assert_eq!(
        http_revoke(&context, device_id, Some(&other_actor_bearer)).await,
        StatusCode::BAD_REQUEST,
        "another actor cannot revoke an active owner-scoped device"
    );
    assert_eq!(
        row_state(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await,
        ("active".to_owned(), 1, None)
    );
    assert_eq!(
        audit_rows(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await
        .len(),
        0,
        "an unauthorized attempt must not append an audit fact"
    );
    assert_eq!(
        count_refresh(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
            false,
        )
        .await,
        2,
        "an unauthorized attempt must not revoke the owner's refresh tokens"
    );

    assert_eq!(
        http_revoke(&context, device_id, Some(&context.bearer)).await,
        StatusCode::NO_CONTENT
    );
    let (status, generation, revoked_at) = row_state(
        &context.admin_pool,
        context.tenant_id,
        context.actor_id,
        device_id,
    )
    .await;
    assert_eq!(status, "revoked");
    assert_eq!(generation, 2);
    assert!(revoked_at.is_some());
    assert_eq!(
        audit_rows(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await,
        vec![("revoked".to_owned(), 2, public_key_fingerprint)]
    );
    assert_eq!(
        count_refresh(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
            true,
        )
        .await,
        2
    );
    assert_eq!(
        count_refresh(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            other_device_id,
            false,
        )
        .await,
        1,
        "revoking one device must not revoke another device's refresh token"
    );

    assert_eq!(
        http_revoke(&context, device_id, Some(&context.bearer)).await,
        StatusCode::BAD_REQUEST,
        "the current endpoint rejects a repeated revocation"
    );
    assert_eq!(
        audit_rows(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await
        .len(),
        1,
        "duplicate and cross-actor attempts must not append audit facts"
    );
    context.app_pool.close().await;
    context.admin_pool.close().await;
}

#[tokio::test]
#[ignore = "requires runner-owned loopback PostgreSQL 18 and test-only role/audit prerequisites"]
async fn concurrent_revoke_has_one_winner_and_one_audit_fact() {
    let context = test_context().await;
    let device_id = Uuid::new_v4();
    seed_device(
        &context,
        context.tenant_id,
        context.actor_id,
        device_id,
        &fingerprint(),
    )
    .await;
    seed_refresh(&context, context.tenant_id, context.actor_id, device_id).await;

    let barrier = Arc::new(Barrier::new(2));
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..2 {
        let barrier = Arc::clone(&barrier);
        let app = context.app.clone();
        let token = context.bearer.clone();
        tasks.spawn(async move {
            barrier.wait().await;
            let request = Request::builder()
                .method("POST")
                .uri(format!("/oauth/device/{device_id}/revoke"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("build concurrent revoke request");
            app.oneshot(request)
                .await
                .expect("concurrent revoke route response")
                .status()
        });
    }
    let mut statuses = Vec::new();
    while let Some(result) = tasks.join_next().await {
        statuses.push(result.expect("concurrent request task"));
    }
    statuses.sort_by_key(StatusCode::as_u16);
    assert_eq!(
        statuses,
        vec![StatusCode::NO_CONTENT, StatusCode::BAD_REQUEST]
    );
    assert_eq!(
        audit_rows(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await
        .len(),
        1
    );
    assert_eq!(
        row_state(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await
        .0,
        "revoked"
    );
    context.app_pool.close().await;
    context.admin_pool.close().await;
}

#[tokio::test]
#[ignore = "requires runner-owned loopback PostgreSQL 18 and test-only role/audit prerequisites"]
async fn audit_failure_rolls_back_device_and_refresh_revocation() {
    let context = test_context().await;
    let device_id = Uuid::new_v4();
    let rollback_fingerprint = "f".repeat(64);
    seed_device(
        &context,
        context.tenant_id,
        context.actor_id,
        device_id,
        &rollback_fingerprint,
    )
    .await;
    seed_refresh(&context, context.tenant_id, context.actor_id, device_id).await;

    assert_eq!(
        http_revoke(&context, device_id, Some(&context.bearer)).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        "the test-only audit trigger must force the transaction to roll back"
    );
    assert_eq!(
        row_state(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
        )
        .await,
        ("active".to_owned(), 1, None)
    );
    assert_eq!(
        count_refresh(
            &context.admin_pool,
            context.tenant_id,
            context.actor_id,
            device_id,
            false,
        )
        .await,
        1
    );
    assert!(audit_rows(
        &context.admin_pool,
        context.tenant_id,
        context.actor_id,
        device_id,
    )
    .await
    .is_empty());

    context.app_pool.close().await;
    context.admin_pool.close().await;
}
