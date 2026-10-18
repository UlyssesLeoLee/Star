/*
@cypher schema=1 source_sha256=976ab891712a5d621c4309d0db2ca7ad0fcfc7134f8a1cf5c6074dc7ce49b99a
MERGE (self:File {path:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs"})
MERGE (test:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::run_worktree_binding_mutations_require_authentication",kind:"test"})
MERGE (putEndpoint:Endpoint {id:"PUT /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (deleteEndpoint:Endpoint {id:"DELETE /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (self)-[:DEFINES]->(test)
MERGE (test)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (test)-[:TESTS]->(putEndpoint)
MERGE (test)-[:TESTS]->(deleteEndpoint)
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
    auth::JwtConfig,
    group_api::{build_group_router, GroupApiState},
};
use tower::ServiceExt;

#[tokio::test]
async fn run_worktree_binding_mutations_require_authentication() {
    let jwt = Arc::new(JwtConfig {
        private_key_pem: String::new(),
        public_key_pem: String::new(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    });
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
