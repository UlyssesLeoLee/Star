// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/bi_analytics_endpoints.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::bi_analytics_endpoints_tests",kind:"module"})
MERGE (authTest:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::bi_task_event_routes_require_authentication",kind:"test"})
MERGE (scopeTest:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::bi_task_event_routes_require_project_read_scope_before_database_access",kind:"test"})
MERGE (parseTest:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::bi_task_event_routes_reject_malformed_run_id",kind:"test"})
MERGE (acceptanceTest:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::real_postgres_bi_summary_and_events_reconcile",kind:"test"})
MERGE (jwt:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::route_test_jwt_config",kind:"function"})
MERGE (token:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::route_test_token",kind:"function"})
MERGE (sendGet:Symbol {id:"crates/star-api-rest/tests/bi_analytics_endpoints.rs::send_authorized_get",kind:"function"})
MERGE (sumEndpoint:Endpoint {id:"GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-summary",kind:"http_route"})
MERGE (eventsEndpoint:Endpoint {id:"GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-events",kind:"http_route"})
MERGE (buildRouter:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (issueToken:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token",kind:"function"})
MERGE (jwtPrivate:File {path:"crates/star-api-rest/tests/fixtures/schedule-route-test-private.pem"})
MERGE (jwtPublic:File {path:"crates/star-api-rest/tests/fixtures/schedule-route-test-public.pem"})
MERGE (databaseUrl:Config {id:"STAR_BI_ANALYTICS_HTTP_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (phase:Config {id:"STAR_BI_ANALYTICS_HTTP_TEST_PHASE",kind:"environment_variable"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEFINES]->(authTest)
MERGE (self)-[:DEFINES]->(scopeTest)
MERGE (self)-[:DEFINES]->(parseTest)
MERGE (self)-[:DEFINES]->(acceptanceTest)
MERGE (self)-[:DEFINES]->(jwt)
MERGE (self)-[:DEFINES]->(token)
MERGE (self)-[:DEFINES]->(sendGet)
MERGE (authTest)-[:TESTS]->(sumEndpoint)
MERGE (authTest)-[:TESTS]->(eventsEndpoint)
MERGE (scopeTest)-[:TESTS]->(sumEndpoint)
MERGE (scopeTest)-[:TESTS]->(eventsEndpoint)
MERGE (parseTest)-[:TESTS]->(sumEndpoint)
MERGE (parseTest)-[:TESTS]->(eventsEndpoint)
MERGE (acceptanceTest)-[:TESTS]->(sumEndpoint)
MERGE (acceptanceTest)-[:TESTS]->(eventsEndpoint)
MERGE (authTest)-[:CALLS]->(buildRouter)
MERGE (scopeTest)-[:CALLS]->(buildRouter)
MERGE (parseTest)-[:CALLS]->(buildRouter)
MERGE (acceptanceTest)-[:CALLS]->(buildRouter)
MERGE (scopeTest)-[:CALLS]->(jwt)
MERGE (scopeTest)-[:CALLS]->(token)
MERGE (parseTest)-[:CALLS]->(jwt)
MERGE (parseTest)-[:CALLS]->(token)
MERGE (acceptanceTest)-[:CALLS]->(jwt)
MERGE (acceptanceTest)-[:CALLS]->(token)
MERGE (jwt)-[:READS]->(jwtPrivate)
MERGE (jwt)-[:READS]->(jwtPublic)
MERGE (acceptanceTest)-[:READS]->(databaseUrl)
MERGE (acceptanceTest)-[:READS]->(phase)
@endcypher
*/
//! Route registration and authorization boundary checks for the Run-scoped BI analytics
//! endpoints (`task-events` and its derived `task-summary`).
//!
//! The 401/403/400 checks run against a lazy, never-connecting pool — the handlers must
//! reject those requests before any database access. The end-to-end reconciliation check
//! is an isolated PostgreSQL acceptance test (`#[ignore]`) that walks the full router with
//! the production owner query store, asserting the summary aggregate counts every event the
//! `task-events` endpoint returns for the same scope.

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
    Router,
};
use bi_owner_runtime::PgTaskEventQueryStore;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use star_api_rest::{
    auth::{issue_token, JwtConfig},
    group_api::{build_group_router, GroupApiState},
};
use tower::ServiceExt;
use uuid::Uuid;

const ACCEPTANCE_TENANT: &str = "10000000-0000-0000-0000-000000000001";
const ACCEPTANCE_OTHER_TENANT: &str = "10000000-0000-0000-0000-000000000002";
const ACCEPTANCE_PROJECT: &str = "20000000-0000-0000-0000-000000000001";
const ACCEPTANCE_ACTOR: &str = "30000000-0000-0000-0000-000000000001";
const ACCEPTANCE_RUN: &str = "60000000-0000-0000-0000-000000000001";
const BI_READ_SCOPE: &str = "project:read";

fn route_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

fn route_test_token(config: &JwtConfig, scope: &str) -> String {
    issue_token(
        config,
        Uuid::parse_str(ACCEPTANCE_ACTOR).expect("acceptance actor UUID is valid"),
        Uuid::parse_str(ACCEPTANCE_TENANT).expect("acceptance tenant UUID is valid"),
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("fixed test-only RSA key should sign BI route test tokens")
    .0
}

fn route_test_token_for_actor(
    config: &JwtConfig,
    actor_id: &str,
    tenant_id: &str,
    scope: &str,
) -> String {
    issue_token(
        config,
        Uuid::parse_str(actor_id).expect("acceptance actor UUID is valid"),
        Uuid::parse_str(tenant_id).expect("acceptance tenant UUID is valid"),
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("fixed test-only RSA key should sign acceptance tokens")
    .0
}

struct GetReply {
    status: StatusCode,
    body: Value,
}

async fn send_authorized_get(app: Router, token: Option<&str>, uri: &str) -> GetReply {
    let mut request = Request::builder().method(Method::GET).uri(uri);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .oneshot(
            request
                .body(Body::empty())
                .expect("BI route request should build"),
        )
        .await
        .expect("router should return an HTTP response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 256 * 1024)
        .await
        .expect("bounded BI response body should be readable");
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&bytes) }));
    GetReply { status, body }
}

fn lazy_unavailable_app() -> Router {
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://localhost/unavailable-for-bi-route-test")
        .expect("lazy test pool configuration should parse");
    build_group_router(GroupApiState::new(jwt, pool))
}

fn bi_paths() -> [String; 2] {
    [
        format!("/api/v1/engineering-runs/{ACCEPTANCE_RUN}/bi/task-events"),
        format!("/api/v1/engineering-runs/{ACCEPTANCE_RUN}/bi/task-summary"),
    ]
}

#[tokio::test]
async fn bi_task_event_routes_require_authentication() {
    let app = lazy_unavailable_app();
    for path in bi_paths() {
        let reply = send_authorized_get(app.clone(), None, &path).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "path {path}");
    }
}

#[tokio::test]
async fn bi_task_event_routes_require_project_read_scope_before_database_access() {
    let jwt = route_test_jwt_config();
    let app = lazy_unavailable_app();
    let token = route_test_token(&jwt, "worktree:read");
    for path in bi_paths() {
        let reply = send_authorized_get(app.clone(), Some(&token), &path).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "path {path}");
        assert_eq!(reply.body["error"]["code"], "insufficient_scope");
    }
}

#[tokio::test]
async fn bi_task_event_routes_reject_malformed_run_id_before_database_access() {
    let jwt = route_test_jwt_config();
    let app = lazy_unavailable_app();
    let token = route_test_token(&jwt, BI_READ_SCOPE);
    for path in [
        "/api/v1/engineering-runs/not-a-uuid/bi/task-events",
        "/api/v1/engineering-runs/not-a-uuid/bi/task-summary",
    ] {
        let reply = send_authorized_get(app.clone(), Some(&token), path).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "path {path}");
        assert_eq!(reply.body["error"]["code"], "invalid_id");
    }
}

/// Runs only under the isolated PostgreSQL BI acceptance runner. The application pool
/// connects as a non-superuser runtime login whose role topology grants both the
/// engineering-run authorization reads and the `bi.list_task_event_page_v1` owner query.
/// The summary aggregate must count exactly the events the paginated `task-events`
/// endpoint returns for the same scope, for whatever dataset is currently seeded.
#[tokio::test]
#[ignore = "requires the isolated bi-analytics authenticated HTTP acceptance runner"]
async fn real_postgres_bi_summary_and_events_reconcile() {
    let database_url = std::env::var("STAR_BI_ANALYTICS_HTTP_TEST_DATABASE_URL")
        .expect("runner must provide the disposable runtime-role database URL");
    let phase = std::env::var("STAR_BI_ANALYTICS_HTTP_TEST_PHASE")
        .expect("runner must select a bounded acceptance phase");
    assert_eq!(
        phase, "reconcile",
        "runner phase must match this acceptance test"
    );
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("runtime role should connect to the disposable PostgreSQL cluster");

    let jwt = route_test_jwt_config();
    let state = GroupApiState::new(jwt.clone(), pool.clone())
        .with_bi_task_event_query_port(Arc::new(PgTaskEventQueryStore::new(pool)));
    let app = build_group_router(state);
    let token = route_test_token(&jwt, BI_READ_SCOPE);

    let summary_path = format!("/api/v1/engineering-runs/{ACCEPTANCE_RUN}/bi/task-summary");
    let summary_reply = send_authorized_get(app.clone(), Some(&token), &summary_path).await;
    assert_eq!(
        summary_reply.status,
        StatusCode::OK,
        "{:?}",
        summary_reply.body
    );
    let summary = &summary_reply.body;
    assert_eq!(summary["project_id"], ACCEPTANCE_PROJECT);
    assert_eq!(summary["engineering_run_id"], ACCEPTANCE_RUN);
    let total_count = summary["total_count"]
        .as_u64()
        .expect("summary total_count must be an integer");
    let work_item_count = summary["work_item_count"]
        .as_u64()
        .expect("summary work_item_count must be an integer");
    let distribution_total: u64 = summary["status_distribution"]
        .as_array()
        .expect("status_distribution must be an array")
        .iter()
        .map(|bucket| {
            bucket["count"]
                .as_u64()
                .expect("bucket count must be an integer")
        })
        .sum();
    assert_eq!(
        distribution_total, work_item_count,
        "status distribution must sum to the distinct work item count"
    );
    assert_eq!(
        summary["source_event_range"]["event_count"].as_u64(),
        Some(total_count),
        "source_event_range.event_count must equal total_count"
    );

    // Walk the full task-events keyset stream and require the summary total to equal it.
    let mut seen = 0u64;
    let mut cursor: Option<(String, String)> = None;
    for _ in 0..2_000 {
        let mut query = String::from("?limit=50");
        if let Some((occurred_at, event_id)) = &cursor {
            query.push_str(&format!(
                "&before_occurred_at={}&before_event_id={}",
                occurred_at, event_id
            ));
        }
        let events_path =
            format!("/api/v1/engineering-runs/{ACCEPTANCE_RUN}/bi/task-events{query}");
        let reply = send_authorized_get(app.clone(), Some(&token), &events_path).await;
        assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
        let items = reply.body["items"]
            .as_array()
            .expect("task-events items must be an array");
        seen += items.len() as u64;
        if reply.body["has_more"] == json!(true) {
            let next = reply.body["next_cursor"]
                .as_object()
                .expect("has_more pages must carry a cursor");
            cursor = Some((
                next["before_occurred_at"]
                    .as_str()
                    .expect("cursor timestamp")
                    .to_string(),
                next["before_event_id"]
                    .as_str()
                    .expect("cursor event id")
                    .to_string(),
            ));
        } else {
            break;
        }
    }
    assert_eq!(
        seen, total_count,
        "summary total_count must equal the full task-events stream count"
    );

    // Cross-tenant access must fail closed (tenant-scoped run lookup: no data leak).
    let foreign_token = route_test_token_for_actor(
        &jwt,
        "30000000-0000-0000-0000-000000000003",
        ACCEPTANCE_OTHER_TENANT,
        BI_READ_SCOPE,
    );
    let cross_tenant = send_authorized_get(app.clone(), Some(&foreign_token), &summary_path).await;
    assert_eq!(
        cross_tenant.status,
        StatusCode::NOT_FOUND,
        "cross-tenant run lookup is tenant-scoped and must not leak the run"
    );
    let cross_events_path = format!("/api/v1/engineering-runs/{ACCEPTANCE_RUN}/bi/task-events");
    let cross_tenant_events =
        send_authorized_get(app.clone(), Some(&foreign_token), &cross_events_path).await;
    assert_eq!(cross_tenant_events.status, StatusCode::NOT_FOUND);
}
