/*
@cypher schema=1 source_sha256=89d0c59188a9bd6bb43dcdd2b79902675737c923f3303eabed271ff2b3010f22
MERGE (self:File {path:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs"})
MERGE (test:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::run_worktree_binding_mutations_require_authentication",kind:"test"})
MERGE (scopeTest:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::run_worktree_binding_mutations_require_both_scopes_before_database_access",kind:"test"})
MERGE (testJwt:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::route_test_jwt_config",kind:"function"})
MERGE (testToken:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::route_test_token",kind:"function"})
MERGE (issueToken:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token",kind:"function"})
MERGE (putEndpoint:Endpoint {id:"PUT /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (deleteEndpoint:Endpoint {id:"DELETE /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(scopeTest)
MERGE (self)-[:DEFINES]->(testJwt)
MERGE (self)-[:DEFINES]->(testToken)
MERGE (test)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (scopeTest)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (scopeTest)-[:CALLS]->(testJwt)
MERGE (scopeTest)-[:CALLS]->(testToken)
MERGE (testToken)-[:CALLS]->(issueToken)
MERGE (test)-[:TESTS]->(putEndpoint)
MERGE (test)-[:TESTS]->(deleteEndpoint)
MERGE (scopeTest)-[:TESTS]->(putEndpoint)
MERGE (scopeTest)-[:TESTS]->(deleteEndpoint)
@endcypher
*/

//! Route registration and authorization boundary checks for Run-to-Worktree writes.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use sqlx::postgres::PgPoolOptions;
use star_api_rest::{
    auth::{issue_token, JwtConfig},
    group_api::{build_group_router, GroupApiState},
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

fn route_test_token(config: &JwtConfig, scope: &str) -> String {
    issue_token(
        config,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("fixed test-only RSA key should sign route test tokens")
    .0
}

#[tokio::test]
async fn run_worktree_binding_mutations_require_authentication() {
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://localhost/unavailable-for-binding-route-test")
        .expect("lazy test pool configuration should parse");
    let app = build_group_router(GroupApiState::new(jwt, pool));
    let path = "/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/worktrees/00000000-0000-0000-0000-000000000003";

    for method in [Method::PUT, Method::DELETE] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .expect("route test request should build"),
            )
            .await
            .expect("router should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "private, no-store"
        );
        assert_eq!(
            response.headers().get(header::VARY).unwrap(),
            "Authorization"
        );
    }
}

#[tokio::test]
async fn run_worktree_binding_mutations_require_both_scopes_before_database_access() {
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://localhost/unavailable-for-binding-route-test")
        .expect("lazy test pool configuration should parse");
    let app = build_group_router(GroupApiState::new(jwt.clone(), pool));
    let path = "/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/worktrees/00000000-0000-0000-0000-000000000003";

    for scope in ["worktree:manage", "project:write"] {
        let token = route_test_token(&jwt, scope);
        for method in [Method::PUT, Method::DELETE] {
            let body = format!(
                r#"{{"expected_binding":{{"state":"unassigned"}},"expected_project_binding_version":1,"correlation_id":"{}"}}"#,
                Uuid::new_v4()
            );
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("Idempotency-Key", Uuid::new_v4().to_string())
                        .body(Body::from(body))
                        .expect("scope test request should build"),
                )
                .await
                .expect("router should return a response");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL).unwrap(),
                "private, no-store"
            );
            assert_eq!(
                response.headers().get(header::VARY).unwrap(),
                "Authorization"
            );
        }
    }
}
