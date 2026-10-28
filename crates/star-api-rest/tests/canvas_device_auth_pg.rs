// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=07b12149a1ed9c42b1e7cdc0d7ac1fbf471d5b087c0a3dbcbde1f94df05d0ac2
MERGE (self:File {path:"crates/star-api-rest/tests/canvas_device_auth_pg.rs"})
MERGE (forged:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::forged_device_headers_do_not_replace_verified_claims",kind:"test"})
MERGE (legacy:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::legacy_token_and_forged_identity_cannot_write_canvas",kind:"test"})
MERGE (stale:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::stale_generation_rejects_shared_canvas_write",kind:"test"})
MERGE (context:Type {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedDeviceContext",kind:"struct"})
MERGE (canvas:Endpoint {id:"POST /api/v1/worktrees/{worktree_id}/canvases",kind:"http_route"})
MERGE (db:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (forged)-[:TESTS]->(context)
MERGE (legacy)-[:TESTS]->(canvas)
MERGE (stale)-[:TESTS]->(canvas)
MERGE (stale)-[:READS]->(db)
MERGE (stale)-[:TESTS]->(:Table {name:"oauth_device_bindings",type:"table"})
MERGE (self)-[:DEFINES]->(forged),(self)-[:DEFINES]->(legacy),(self)-[:DEFINES]->(stale)
@endcypher
*/
//! Device-claim boundary checks for shared Canvas routes.

use std::sync::Arc;

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
};
use serde_json::{Value, json};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use star_api_rest::{
    auth::oauth::AuthenticatedDeviceContext,
    auth::{DeviceTokenContext, JwtConfig, issue_token, issue_token_with_device_context},
    group_api::{GroupApiState, build_group_router},
};
use tower::ServiceExt;
use uuid::Uuid;

fn route_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

async fn echo_device(context: AuthenticatedDeviceContext) -> impl IntoResponse {
    Json(json!({
        "actor_id": context.actor.user_id,
        "tenant_id": context.actor.tenant_id,
        "device_id": context.device.device_id,
        "generation": context.device.generation,
    }))
}

#[tokio::test]
async fn forged_device_headers_do_not_replace_verified_claims() {
    let jwt = route_test_jwt_config();
    let actor_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let verified_device_id = Uuid::new_v4();
    let forged_device_id = Uuid::new_v4();
    let token = issue_token_with_device_context(
        &jwt,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        "canvas:read".to_owned(),
        Some(DeviceTokenContext {
            device_id: verified_device_id,
            generation: 7,
        }),
    )
    .expect("test-only RSA key should issue a signed token")
    .0;

    let app = Router::new()
        .route("/device-context", get(echo_device))
        .with_state(jwt.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/device-context")
                .header("authorization", format!("Bearer {token}"))
                .header("x-device-id", forged_device_id.to_string())
                .header("x-actor-id", Uuid::new_v4().to_string())
                .header("x-tenant-id", Uuid::new_v4().to_string())
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("extractor route should respond");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("bounded response body");
    let body: Value = serde_json::from_slice(&body).expect("JSON response");
    assert_eq!(body["actor_id"], actor_id.to_string());
    assert_eq!(body["tenant_id"], tenant_id.to_string());
    assert_eq!(body["device_id"], verified_device_id.to_string());
    assert_eq!(body["generation"], 7);
    assert_ne!(verified_device_id, forged_device_id);
}

#[tokio::test]
async fn legacy_token_and_forged_identity_cannot_write_canvas() {
    let jwt = route_test_jwt_config();
    let token = issue_token(
        &jwt,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        "worktree:write canvas:write".to_owned(),
    )
    .expect("test-only RSA key should issue a legacy token")
    .0;
    let pool = PgPoolOptions::new()
        .connect_lazy_with(PgConnectOptions::new().database("canvas_device_auth_route_test"));
    let app = build_group_router(GroupApiState::new(jwt, pool));
    let forged_device_id = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/worktrees/{}/canvases", Uuid::new_v4()))
                .header("authorization", format!("Bearer {token}"))
                .header("x-device-id", forged_device_id.to_string())
                .header("content-type", "application/json")
                .header("idempotency-key", Uuid::new_v4().to_string())
                .body(Body::from(
                    json!({
                        "title": "must not become shared",
                        "actor_id": Uuid::new_v4(),
                        "tenant_id": Uuid::new_v4(),
                        "device_id": forged_device_id,
                        "device_generation": 7,
                    })
                    .to_string(),
                ))
                .expect("request should be valid"),
        )
        .await
        .expect("route should reject before database access");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn stale_generation_rejects_shared_canvas_write() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let jwt = route_test_jwt_config();
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");

    let mut tx = pool.begin().await.expect("begin device fixture");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(actor_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set device owner scope");
    sqlx::query(
        "INSERT INTO oauth_device_bindings \
         (tenant_id,actor_id,device_id,public_key_fingerprint,generation,status) \
         VALUES ($1,$2,$3,$4,2,'active')",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .bind("a".repeat(64))
    .execute(&mut *tx)
    .await
    .expect("insert current device generation");
    tx.commit().await.expect("commit device fixture");

    let token = issue_token_with_device_context(
        &jwt,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        "worktree:write canvas:write".to_owned(),
        Some(DeviceTokenContext {
            device_id,
            generation: 1,
        }),
    )
    .expect("test-only RSA key should issue a stale device token")
    .0;
    let app = build_group_router(GroupApiState::new(jwt, pool.clone()));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/worktrees/{}/canvases", Uuid::new_v4()))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("idempotency-key", Uuid::new_v4().to_string())
                .body(Body::from(json!({"title":"stale device"}).to_string()))
                .expect("request should be valid"),
        )
        .await
        .expect("stale device must fail closed before worktree lookup");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let mut tx = pool.begin().await.expect("begin fixture cleanup");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(actor_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set cleanup scope");
    sqlx::query(
        "DELETE FROM oauth_device_bindings \
         WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .execute(&mut *tx)
    .await
    .expect("delete device fixture");
    tx.commit().await.expect("commit fixture cleanup");
    pool.close().await;
}
