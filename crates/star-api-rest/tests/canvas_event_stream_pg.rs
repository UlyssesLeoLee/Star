/*
@cypher schema=1 source_sha256=bece16b21115cace523d64ee5e2e0c6a736312bd9afa3bc1e2779ce62133ee01
MERGE (self:File {path:"crates/star-api-rest/tests/canvas_event_stream_pg.rs"})
MERGE (test:Symbol {id:"crates/star-api-rest/tests/canvas_event_stream_pg.rs::canvas_outbox_notification_is_transactional_and_worktree_scoped",kind:"test"})
MERGE (authTest:Symbol {id:"crates/star-api-rest/tests/canvas_event_stream_pg.rs::stream_denies_missing_canvas_scope_before_database_access",kind:"test"})
MERGE (missingHubTest:Symbol {id:"crates/star-api-rest/tests/canvas_event_stream_pg.rs::stream_fails_closed_when_process_listener_is_not_installed",kind:"test"})
MERGE (jwtConfig:Symbol {id:"crates/star-api-rest/tests/canvas_event_stream_pg.rs::route_test_jwt_config",kind:"function"})
MERGE (token:Symbol {id:"crates/star-api-rest/tests/canvas_event_stream_pg.rs::token_for_actor",kind:"function"})
MERGE (insert:Symbol {id:"crates/star-api-rest/tests/canvas_event_stream_pg.rs::insert_outbox_event",kind:"function"})
MERGE (migration:File {path:"db/migrations/2026-10-28-canvas-outbox-wakeup.sql"})
MERGE (database:Config {id:"STAR_CANVAS_WAKE_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (actor:Config {id:"STAR_CANVAS_WAKE_TEST_ACTOR_ID",kind:"environment_variable"})
MERGE (tenant:Config {id:"STAR_CANVAS_WAKE_TEST_TENANT_ID",kind:"environment_variable"})
MERGE (project:Config {id:"STAR_CANVAS_WAKE_TEST_PROJECT_ID",kind:"environment_variable"})
MERGE (worktree:Config {id:"STAR_CANVAS_WAKE_TEST_WORKTREE_ID",kind:"environment_variable"})
MERGE (listener:Type {id:"sqlx::postgres::PgListener",kind:"struct"})
MERGE (hub:Type {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub",kind:"struct"})
MERGE (endpoint:Endpoint {id:"GET /api/v1/worktrees/{worktree_id}/canvas-events/stream",kind:"http_route"})
MERGE (projectRole:Table {id:"permission.project_role_binding",kind:"table"})
MERGE (outbox:Table {id:"canvas.canvas_group_outbox",kind:"table"})
MERGE (channel:Topic {id:"postgresql.star_canvas_outbox_wake",kind:"topic"})
MERGE (test)-[:CALLS]->(insert)
MERGE (test)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub.start",kind:"method"})
MERGE (test)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::stream_worktree_canvas_events",kind:"function"})
MERGE (test)-[:CALLS]->(token)
MERGE (authTest)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::stream_worktree_canvas_events",kind:"function"})
MERGE (authTest)-[:CALLS]->(token)
MERGE (missingHubTest)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::stream_worktree_canvas_events",kind:"function"})
MERGE (missingHubTest)-[:CALLS]->(token)
MERGE (jwtConfig)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token",kind:"function"})
MERGE (token)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token",kind:"function"})
MERGE (test)-[:USES_TYPE]->(listener)
MERGE (test)-[:READS]->(database)
MERGE (test)-[:READS]->(actor)
MERGE (test)-[:READS]->(tenant)
MERGE (test)-[:READS]->(project)
MERGE (test)-[:READS]->(worktree)
MERGE (test)-[:TESTS]->(migration)
MERGE (test)-[:TESTS]->(endpoint)
MERGE (test)-[:WRITES]->(projectRole)
MERGE (insert)-[:WRITES]->(outbox)
MERGE (insert)-[:EMITS]->(channel)
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(authTest)
MERGE (self)-[:DEFINES]->(missingHubTest)
MERGE (self)-[:DEFINES]->(jwtConfig)
MERGE (self)-[:DEFINES]->(token)
MERGE (self)-[:DEFINES]->(insert)
@endcypher
*/
//! Isolated PostgreSQL acceptance for commit-only, Worktree-scoped Outbox wakeups.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures_util::StreamExt;
use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgListener, PgPoolOptions},
};
use star_api_rest::{
    auth::{JwtConfig, issue_token},
    group_api::{CanvasEventWakeHub, GroupApiState, build_group_router},
};
use tower::ServiceExt;
use uuid::Uuid;

const CHANNEL: &str = "star_canvas_outbox_wake";

fn route_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

#[tokio::test]
async fn stream_denies_missing_canvas_scope_before_database_access() {
    let jwt = route_test_jwt_config();
    let token = issue_token(
        &jwt,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        "worktree:read".to_owned(),
    )
    .expect("test-only RSA key should issue the bearer token")
    .0;
    let pool = PgPoolOptions::new()
        .connect_lazy_with(PgConnectOptions::new().database("canvas_wake_route_test"));
    let app = build_group_router(GroupApiState::new(jwt, pool));
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/worktrees/{}/canvas-events/stream",
                    Uuid::new_v4()
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("route should respond");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn stream_fails_closed_when_process_listener_is_not_installed() {
    let jwt = route_test_jwt_config();
    let token = issue_token(
        &jwt,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        "worktree:read canvas:read".to_owned(),
    )
    .expect("test-only RSA key should issue the bearer token")
    .0;
    let pool = PgPoolOptions::new()
        .connect_lazy_with(PgConnectOptions::new().database("canvas_wake_route_test"));
    let app = build_group_router(GroupApiState::new(jwt, pool));
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/worktrees/{}/canvas-events/stream",
                    Uuid::new_v4()
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request should be valid"),
        )
        .await
        .expect("route should respond");
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database and a fixture Worktree"]
async fn canvas_outbox_notification_is_transactional_and_worktree_scoped() {
    let database_url = std::env::var("STAR_CANVAS_WAKE_TEST_DATABASE_URL")
        .expect("acceptance runner must provide its disposable PostgreSQL URL");
    let tenant_id = parse_fixture_uuid("STAR_CANVAS_WAKE_TEST_TENANT_ID");
    let project_id = parse_fixture_uuid("STAR_CANVAS_WAKE_TEST_PROJECT_ID");
    let worktree_id = parse_fixture_uuid("STAR_CANVAS_WAKE_TEST_WORKTREE_ID");
    let actor_id = parse_fixture_uuid("STAR_CANVAS_WAKE_TEST_ACTOR_ID");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL database should accept the test connection");

    sqlx::raw_sql(include_str!(
        "../../../db/migrations/2026-10-28-canvas-outbox-wakeup.sql"
    ))
    .execute(&pool)
    .await
    .expect("wake trigger migration should apply to the isolated database");

    let mut listener = PgListener::connect_with(&pool)
        .await
        .expect("listener should connect to the same isolated database");
    listener
        .listen(CHANNEL)
        .await
        .expect("listener should subscribe to the wake-only channel");
    let hub = CanvasEventWakeHub::start(pool.clone())
        .await
        .expect("one process listener should start after the trigger migration");

    let mut rolled_back = pool
        .begin()
        .await
        .expect("rollback transaction should begin");
    insert_outbox_event(
        &mut rolled_back,
        tenant_id,
        project_id,
        worktree_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
    )
    .await;
    rolled_back
        .rollback()
        .await
        .expect("test event should roll back");
    let rolled_back_wake = tokio::time::timeout(Duration::from_millis(150), async {
        loop {
            let notification = listener.recv().await.expect("listener remains connected");
            if notification.channel() == CHANNEL
                && notification.payload() == worktree_id.to_string()
            {
                break;
            }
        }
    })
    .await;
    assert!(rolled_back_wake.is_err(), "rollback must not emit a wake");

    let canvas_id = Uuid::new_v4();
    assert_ne!(
        tenant_id, canvas_id,
        "fixture must distinguish tenant and Canvas ids"
    );
    assert_ne!(worktree_id, canvas_id);
    let mut committed = pool.begin().await.expect("commit transaction should begin");
    insert_outbox_event(
        &mut committed,
        tenant_id,
        project_id,
        worktree_id,
        canvas_id,
        Uuid::new_v4(),
    )
    .await;
    insert_outbox_event(
        &mut committed,
        tenant_id,
        project_id,
        worktree_id,
        canvas_id,
        Uuid::new_v4(),
    )
    .await;
    committed
        .commit()
        .await
        .expect("Outbox facts should commit");

    let notification = tokio::time::timeout(Duration::from_secs(2), listener.recv())
        .await
        .expect("commit should emit a wake")
        .expect("listener should remain connected");
    assert_eq!(notification.channel(), CHANNEL);
    assert_eq!(notification.payload(), worktree_id.to_string());
    assert_ne!(notification.payload(), tenant_id.to_string());
    assert_ne!(notification.payload(), canvas_id.to_string());

    let rows = sqlx::query(
        "SELECT count(*) AS event_count, count(DISTINCT stream_offset) AS cursor_count \
         FROM canvas.canvas_group_outbox \
         WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3",
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .fetch_one(&pool)
    .await
    .expect("durable Outbox events should remain queryable");
    assert_eq!(rows.get::<i64, _>("event_count"), 2);
    assert_eq!(rows.get::<i64, _>("cursor_count"), 2);

    let jwt = route_test_jwt_config();
    let app = build_group_router(
        GroupApiState::new(jwt.clone(), pool.clone()).with_canvas_event_wake_hub(hub),
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/worktrees/{worktree_id}/canvas-events/stream"
                ))
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        token_for_actor(&jwt, actor_id, tenant_id, "worktree:read canvas:read")
                    ),
                )
                .body(Body::empty())
                .expect("authenticated stream request should be valid"),
        )
        .await
        .expect("stream route should respond");
    assert_eq!(response.status(), StatusCode::OK);

    let revoked = sqlx::query(
        "UPDATE permission.project_role_binding \
         SET valid_to=clock_timestamp(), updated_at=clock_timestamp(), version=version+1 \
         WHERE tenant_id=$1 AND project_id=$2 AND user_id=$3 AND valid_to IS NULL",
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(actor_id)
    .execute(&pool)
    .await
    .expect("test fixture project membership should be revocable");
    assert_eq!(revoked.rows_affected(), 1);

    let mut after_revocation = pool
        .begin()
        .await
        .expect("post-revocation transaction should begin");
    insert_outbox_event(
        &mut after_revocation,
        tenant_id,
        project_id,
        worktree_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
    )
    .await;
    after_revocation
        .commit()
        .await
        .expect("post-revocation wake should commit");
    let mut body = response.into_body().into_data_stream();
    let next_frame = tokio::time::timeout(Duration::from_secs(2), body.next())
        .await
        .expect("authorization recheck should finish");
    assert!(
        next_frame.is_none(),
        "stream must close without sending a wake after membership revocation"
    );
}

fn parse_fixture_uuid(name: &str) -> Uuid {
    std::env::var(name)
        .expect("acceptance runner must provide fixture UUIDs")
        .parse()
        .expect("fixture value must be a UUID")
}

fn token_for_actor(jwt: &JwtConfig, actor_id: Uuid, tenant_id: Uuid, scope: &str) -> String {
    issue_token(
        jwt,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("test-only RSA key should issue the bearer token")
    .0
}

async fn insert_outbox_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    correlation_id: Uuid,
) {
    sqlx::query(
        "INSERT INTO canvas.canvas_group_outbox \
         (tenant_id, project_id, worktree_id, canvas_id, event_type, aggregate_version, actor_id, correlation_id) \
         VALUES ($1,$2,$3,$4,'canvas.test.wake',1,$5,$6)",
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(Uuid::new_v4())
    .bind(correlation_id)
    .execute(&mut **tx)
    .await
    .expect("test Outbox event should insert into the disposable database");
}
