// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=d72603d272515c110d085fd8a007cb8e102356ac36faae73c5e98332ca412f57
MERGE (self:File {path:"crates/star-api-rest/tests/canvas_device_auth_pg.rs"})
MERGE (forged:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::forged_device_headers_do_not_replace_verified_claims",kind:"test"})
MERGE (legacy:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::legacy_token_and_forged_identity_cannot_write_canvas",kind:"test"})
MERGE (stale:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::stale_generation_rejects_shared_canvas_write",kind:"test"})
MERGE (revoke:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::active_canvas_sse_closes_after_device_binding_revocation",kind:"test"})
MERGE (fixture:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::seed_canvas_sse_fixture",kind:"function"})
MERGE (wake:Symbol {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::insert_canvas_outbox_wake",kind:"function"})
MERGE (context:Type {id:"crates/star-api-rest/src/auth/oauth/middleware.rs::AuthenticatedDeviceContext",kind:"struct"})
MERGE (canvas:Endpoint {id:"POST /api/v1/worktrees/{worktree_id}/canvases",kind:"http_route"})
MERGE (stream:Endpoint {id:"GET /api/v1/worktrees/{worktree_id}/canvas-events/stream",kind:"http_route"})
MERGE (db:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (outboxMigration:File {path:"db/migrations/2026-10-28-canvas-outbox-wakeup.sql"})
MERGE (deviceMigration:File {path:"db/migrations/2026-10-28-w16-device-auth.sql"})
MERGE (hub:Type {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub",kind:"struct"})
MERGE (fixtureType:Type {id:"crates/star-api-rest/tests/canvas_device_auth_pg.rs::CanvasSseFixture",kind:"struct"})
MERGE (outbox:Table {name:"canvas.canvas_group_outbox",type:"table"})
MERGE (binding:Table {name:"oauth_device_bindings",type:"table"})
MERGE (forged)-[:TESTS]->(context)
MERGE (legacy)-[:TESTS]->(canvas)
MERGE (stale)-[:TESTS]->(canvas)
MERGE (stale)-[:READS]->(db)
MERGE (stale)-[:TESTS]->(:Table {name:"oauth_device_bindings",type:"table"})
MERGE (revoke)-[:CALLS]->(fixture)
MERGE (revoke)-[:CALLS]->(wake)
MERGE (revoke)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub.start",kind:"method"})
MERGE (revoke)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::stream_worktree_canvas_events",kind:"function"})
MERGE (revoke)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token_with_device_context",kind:"function"})
MERGE (revoke)-[:READS]->(db)
MERGE (revoke)-[:WRITES]->(binding)
MERGE (revoke)-[:TESTS]->(stream)
MERGE (revoke)-[:TESTS]->(outboxMigration)
MERGE (revoke)-[:TESTS]->(deviceMigration)
MERGE (fixture)-[:WRITES]->(:Table {id:"permission.project_role_binding",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"scm.cloud_branch",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"scm.cloud_branch_revision",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"permission.cloud_branch_role_binding",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"multica.engineering_run",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"multica.engineering_run_revision",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"permission.engineering_run_role_binding",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"public.worktree_canvas_worktree",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"multica.worktree_project_binding",kind:"table"})
MERGE (fixture)-[:WRITES]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
MERGE (fixture)-[:WRITES]->(binding)
MERGE (wake)-[:WRITES]->(outbox)
MERGE (fixture)-[:USES_TYPE]->(fixtureType)
MERGE (wake)-[:USES_TYPE]->(fixtureType)
MERGE (revoke)-[:USES_TYPE]->(hub)
MERGE (self)-[:DEFINES]->(forged),(self)-[:DEFINES]->(legacy),(self)-[:DEFINES]->(stale),
    (self)-[:DEFINES]->(revoke),(self)-[:DEFINES]->(fixture),(self)-[:DEFINES]->(wake),
    (self)-[:DEFINES]->(fixtureType)
@endcypher
*/
//! Device-claim boundary checks for shared Canvas routes.

use std::{sync::Arc, time::Duration};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use star_api_rest::{
    auth::oauth::AuthenticatedDeviceContext,
    auth::{issue_token, issue_token_with_device_context, DeviceTokenContext, JwtConfig},
    group_api::{build_group_router, CanvasEventWakeHub, GroupApiState},
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

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn active_canvas_sse_closes_after_device_binding_revocation() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(6)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let local_server: bool = sqlx::query_scalar(
        "SELECT COALESCE(inet_server_addr() <<= '127.0.0.0/8'::inet \
             OR inet_server_addr() = '::1'::inet, TRUE)",
    )
    .fetch_one(&pool)
    .await
    .expect("inspect PostgreSQL server locality without exposing connection details");
    assert!(
        local_server,
        "Canvas SSE acceptance only permits a local PostgreSQL server or local socket"
    );
    sqlx::raw_sql(include_str!(
        "../../../db/migrations/2026-10-28-canvas-outbox-wakeup.sql"
    ))
    .execute(&pool)
    .await
    .expect("Canvas Outbox wake trigger migration should be installed");

    let fixture = seed_canvas_sse_fixture(&pool).await;
    let hub = CanvasEventWakeHub::start(pool.clone())
        .await
        .expect("both Canvas and device-binding PostgreSQL wake triggers must be installed");
    let app = build_group_router(
        GroupApiState::new(jwt.clone(), pool.clone()).with_canvas_event_wake_hub(hub),
    );
    let token = issue_token_with_device_context(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        vec!["developer".to_owned()],
        "worktree:read canvas:read".to_owned(),
        Some(DeviceTokenContext {
            device_id: fixture.device_id,
            generation: 1,
        }),
    )
    .expect("test-only RSA key should issue a device-bound token")
    .0;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/worktrees/{}/canvas-events/stream",
                    fixture.worktree_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("stream request should be valid"),
        )
        .await
        .expect("persisted Run/Worktree/device authorization should admit the stream");
    assert_eq!(response.status(), StatusCode::OK);

    let mut body = response.into_body().into_data_stream();
    insert_canvas_outbox_wake(&pool, &fixture).await;
    let first_frame = tokio::time::timeout(Duration::from_secs(2), body.next())
        .await
        .expect("committed Canvas Outbox notification should reach the active stream")
        .expect("stream should remain open after an authorized wake")
        .expect("SSE frame should be encoded successfully");
    assert!(
        String::from_utf8_lossy(&first_frame).contains("event: canvas-wake"),
        "the live route should deliver the real Worktree-scoped PostgreSQL wake"
    );

    let mut tx = pool.begin().await.expect("device revocation transaction");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(fixture.tenant_id.to_string())
        .bind(fixture.actor_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set device owner scope");
    let revoked = sqlx::query(
        "UPDATE oauth_device_bindings \
         SET status='revoked', revoked_at=clock_timestamp() \
         WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3 AND generation=1 AND status='active'",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.actor_id)
    .bind(fixture.device_id)
    .execute(&mut *tx)
    .await
    .expect("revoke persisted device binding and invoke the W16 trigger");
    assert_eq!(revoked.rows_affected(), 1);
    tx.commit()
        .await
        .expect("device revocation should commit before PostgreSQL emits NOTIFY");

    let closed = tokio::time::timeout(Duration::from_secs(5), body.next())
        .await
        .expect("active SSE stream must close within the five-second test bound");
    assert!(
        closed.is_none(),
        "revoked device authorization must terminate the stream without another SSE frame"
    );

    let trigger_name: Option<String> = sqlx::query_scalar(
        "SELECT tgname::text FROM pg_trigger \
         WHERE tgrelid=to_regclass('public.oauth_device_bindings') \
           AND tgname='w16_oauth_device_binding_generation_wake' \
           AND NOT tgisinternal AND tgenabled IN ('O','A')",
    )
    .fetch_optional(&pool)
    .await
    .expect("inspect installed W16 wake trigger");
    assert_eq!(
        trigger_name.as_deref(),
        Some("w16_oauth_device_binding_generation_wake")
    );
}

struct CanvasSseFixture {
    tenant_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    branch_id: Uuid,
    run_id: Uuid,
    worktree_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
    canvas_id: Uuid,
}

async fn seed_canvas_sse_fixture(pool: &PgPool) -> CanvasSseFixture {
    let fixture = CanvasSseFixture {
        tenant_id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        repository_id: Uuid::new_v4(),
        branch_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        worktree_id: Uuid::new_v4(),
        workspace_id: Uuid::new_v4(),
        actor_id: Uuid::new_v4(),
        device_id: Uuid::new_v4(),
        canvas_id: Uuid::new_v4(),
    };
    let mut tx = pool
        .begin()
        .await
        .expect("begin persisted authorization fixture");
    sqlx::query(
        "SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true), \
         set_config('app.correlation_id',$3,true), set_config('app.workspace_id',$4,true), \
         set_config('app.role','member',true)",
    )
    .bind(fixture.tenant_id.to_string())
    .bind(fixture.actor_id.to_string())
    .bind(Uuid::new_v4().to_string())
    .bind(fixture.workspace_id.to_string())
    .execute(&mut *tx)
    .await
    .expect("set persisted fixture owner and Worktree scope");

    sqlx::query(
        "INSERT INTO permission.project_role_binding \
         (tenant_id,project_id,user_id,role,granted_by) VALUES ($1,$2,$3,'project_admin',$3)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Project management authority");
    sqlx::query(
        "INSERT INTO scm.cloud_branch \
         (branch_id,tenant_id,project_id,repository_id,remote_identity) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(fixture.branch_id)
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(format!("canvas-sse-test:{}", fixture.branch_id))
    .execute(&mut *tx)
    .await
    .expect("seed canonical persisted Branch");
    sqlx::query(
        "INSERT INTO scm.cloud_branch_revision \
         (tenant_id,branch_id,name,full_ref,state,changed_by,version) \
         VALUES ($1,$2,'main','refs/heads/main','active',$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.branch_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Branch revision");
    sqlx::query(
        "INSERT INTO permission.cloud_branch_role_binding \
         (tenant_id,branch_id,user_id,role,granted_by,version) \
         VALUES ($1,$2,$3,'developer',$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.branch_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Branch read authority");
    sqlx::query(
        "INSERT INTO multica.engineering_run \
         (engineering_run_id,tenant_id,project_id,repository_id,branch_id) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(fixture.run_id)
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(fixture.branch_id)
    .execute(&mut *tx)
    .await
    .expect("seed persisted Engineering Run");
    sqlx::query(
        "INSERT INTO multica.engineering_run_revision \
         (tenant_id,engineering_run_id,title,state,owner_user_id,changed_by,version) \
         VALUES ($1,$2,'Canvas SSE acceptance','active',$3,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Run revision");
    sqlx::query(
        "INSERT INTO permission.engineering_run_role_binding \
         (tenant_id,engineering_run_id,user_id,role,granted_by,version) \
         VALUES ($1,$2,$3,'developer',$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Run read authority");
    sqlx::query(
        "INSERT INTO worktree_canvas_worktree \
         (id,tenant_id,workspace_id,repo_id,name,branch,path,project_id,owner_user_id) \
         VALUES ($1,$2,$3,$4,$5,'main',$6,$7,$8)",
    )
    .bind(fixture.worktree_id)
    .bind(fixture.tenant_id)
    .bind(fixture.workspace_id)
    .bind(fixture.repository_id)
    .bind(format!("canvas-sse-{}", fixture.worktree_id))
    .bind(format!("/tmp/star-canvas-sse-{}", fixture.worktree_id))
    .bind(fixture.project_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed persisted Worktree under tenant and workspace scope");
    let project_binding_id: Uuid = sqlx::query_scalar(
        "INSERT INTO multica.worktree_project_binding \
         (tenant_id,project_id,worktree_id,assigned_by) VALUES ($1,$2,$3,$4) RETURNING binding_id",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.worktree_id)
    .bind(fixture.actor_id)
    .fetch_one(&mut *tx)
    .await
    .expect("seed persisted Project-to-Worktree binding");
    sqlx::query(
        "INSERT INTO multica.engineering_run_worktree_binding \
         (tenant_id,project_id,repository_id,branch_id,engineering_run_id,worktree_id,project_binding_id,assigned_by,version) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(fixture.branch_id)
    .bind(fixture.run_id)
    .bind(fixture.worktree_id)
    .bind(project_binding_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed persisted Engineering Run-to-Worktree binding");
    sqlx::query(
        "INSERT INTO oauth_device_bindings \
         (tenant_id,actor_id,device_id,public_key_fingerprint,generation,status) \
         VALUES ($1,$2,$3,$4,1,'active')",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.actor_id)
    .bind(fixture.device_id)
    .bind(format!("{:064x}", fixture.device_id.as_u128()))
    .execute(&mut *tx)
    .await
    .expect("seed current persisted active device binding");
    tx.commit()
        .await
        .expect("Run/Worktree/device fixture should commit");
    fixture
}

async fn insert_canvas_outbox_wake(pool: &PgPool, fixture: &CanvasSseFixture) {
    let mut tx = pool.begin().await.expect("begin Canvas wake transaction");
    sqlx::query(
        "INSERT INTO canvas.canvas_group_outbox \
         (tenant_id,project_id,worktree_id,canvas_id,event_type,aggregate_version,actor_id,correlation_id) \
         VALUES ($1,$2,$3,$4,'canvas.test.wake',1,$5,$6)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.worktree_id)
    .bind(fixture.canvas_id)
    .bind(fixture.actor_id)
    .bind(Uuid::new_v4())
    .execute(&mut *tx)
    .await
    .expect("insert persisted Worktree-scoped Canvas Outbox fact");
    tx.commit()
        .await
        .expect("Canvas Outbox wake should be emitted only after commit");
}
