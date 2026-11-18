/*
@cypher schema=1 source_sha256=f0b65245019f29ad0620079fbb512b4b4337e6aabe26af7365e3a02d0ab7bc3d
MERGE (self:File {path:"crates/star-api-rest/tests/production_router.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/production_router.rs::production_router_supplemental_modules",kind:"test_suite"})
MERGE (test:Symbol {id:"crates/star-api-rest/tests/production_router.rs::supplemental_modules_mounted_in_production_router",kind:"test"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(test)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! 生产路由挂载验证 (F15 G-01 2026-11-17)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! 验证 `build_group_router(state).merge(build_supplemental_routers())` 组合中，
//! 8 个补充模块路由 (`/memory` `/agents` `/runtime` `/terminal` `/ide` `/im`
//! `/notifications` `/comments`) 在 `/api/v1/` 前缀下可达 (非 404)。
//!
//! 仅做「路由可达」探测，不做业务断言（业务逻辑由各模块 oneshot 测试覆盖）。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use star_api_rest::{
    auth::{issue_token_with_device_context, DeviceTokenContext, JwtConfig},
    build_supplemental_routers,
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
    issue_token_with_device_context(
        config,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        scope.to_owned(),
        Some(DeviceTokenContext {
            device_id: Uuid::new_v4(),
            generation: 1,
        }),
    )
    .expect("fixed test-only RSA key should sign device-bound route test tokens")
    .0
}

fn get_uri(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

/// 生产组合路由：group + supplemental，仅做 GET 探测
#[tokio::test]
async fn supplemental_modules_mounted_in_production_router() {
    let jwt = route_test_jwt_config();
    let token = route_test_token(&jwt, "worktree:read");
    // lazy pool with dummy DSN (tests don't hit DB; GroupApiState only needs pool for resolver)
    let pool =
        sqlx::PgPool::connect_lazy("postgresql://dummy:dummy@localhost/production_router_test")
            .expect("lazy pool");
    let state = GroupApiState::new(jwt, pool);
    let app = build_group_router(state).merge(build_supplemental_routers());

    // 每个端点至少能返回非 404 (401/403/405/422/500 均视为「已挂载」)
    let endpoints = [
        "/api/v1/memory",
        "/api/v1/agents",
        "/api/v1/runtime",
        "/api/v1/terminal/pane", // POST only, GET → 405 仍非 404
        "/api/v1/ide/version",
        "/api/v1/im/conversations",
        "/api/v1/notifications",
        "/api/v1/comments",
    ];

    for path in endpoints {
        let (status, _) = send(&app, get_uri(path, &token)).await;
        assert_ne!(
            status,
            StatusCode::NOT_FOUND,
            "endpoint {path} should be mounted in production router (got 404)"
        );
    }
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp: axum::response::Response = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (status, body)
}

use std::sync::Arc;
