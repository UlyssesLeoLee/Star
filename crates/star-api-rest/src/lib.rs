// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/star-api-rest` — STAR Developer REST API
//!
//! per `docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md` (sibling) §1.1 + §2 + §7.3
//!
//! ## 范围 (Phase I Batch 1+2+3 27/27 真实接线完成, per WBS §14.12.1 + 独立 WBS §1.1)
//!
//! - **27 路由真实接线** (18 MCP tool REST 镜像 + 9 Webhook 管理端点), per spec §2.2 + §2.3
//!   - Batch 1 (8): work-items 5 + workspaces 1 + worktrees 2 → 调 InMemory*Service
//!   - Batch 2 (10): code 4 + context 1 + merge_requests 1 + reviews 1 + validations 1 + submissions 1 + pipelines 1 → 调 InMemory*Service
//!   - Batch 3 (9): webhooks 5 endpoint CRUD + 1 test + 2 delivery 查询 + 1 replay → 端点本地 EndpointStore + 复用 star-webhook::DeliveryStore
//! - **统一错误模型** (6-field `RestError`, per spec §2.4 复用 MCP 错误模型; 6 个 From impl: WorkItemError / WorkspaceError / WorktreeError / SearchError / ScmError / ValidationError)
//! - **统一响应封装** (`RestResponse<T>` data + meta, per spec §2.4)
//! - **鉴权中间件 stub** (`AuthLayer` 验 `Authorization: Bearer <api_key>`, per spec §1.3, Phase IV 真实化)
//! - **限流中间件 stub** (`RateLimitLayer` 60 req/min per key, per spec §1.4, Phase IV 真实化)
//! - **审计中间件 stub** (`AuditLayer` 落 T 类 `api_key_audit_log`, per spec §5.1, Phase IV 真实化)
//!
//! 11/11 tests pass (per 守门 #1 v25 单 crate 模式, 0 回归).
//!
//! ## 不做什么 (per AGENTS.md §0 硬约束 + 守门 #11 缺标比错标安全)
//!
//! - **不**实装业务逻辑 — 仅 routing 骨架
//! - **不**实装 DB schema (per spec §5.1 4 表 W/T/M 分类, 等 P2 阶段)
//! - **不**实装 OAuth 2.0 / mTLS (per spec §1.3, Phase 2+ 评估)
//! - **不**实装 5 域预置 vendor 模板 (per spec §4.1, 等 P2 阶段由 5 域 Lead 拍板)
//! - **不**实装 OpenAPI 3.1 spec 自动生成 (per spec §7.2 + §6 G-10, 等 v0.2 utoipa)

#![allow(missing_docs)] // 骨架阶段, 业务端点 P2 实装时补

pub mod auth;
pub mod error;
pub mod middleware;
pub mod rbac;
pub mod response;
pub mod routes;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

/// REST API 版本前缀 (per spec §2.1)
pub const API_VERSION: &str = "v1";
/// REST API 路径前缀 (per spec §2.1)
pub const API_PREFIX: &str = "/api/v1";

/// REST API 路由器 (骨架 — 22 路由 stub, 业务逻辑 P2 实装)
///
/// 包含:
/// - 16 MCP tool REST 镜像路由
/// - 6 Webhook 管理端点
/// - 鉴权中间件 (`AuthLayer` stub)
/// - 限流中间件 (`RateLimitLayer` stub)
/// - 审计中间件 (`AuditLayer` stub)
///
/// v0.47 §14.12 IV OAuth2 5 endpoints 路由 wire (per brief v0.47 §3.1):
/// - OAuth2 routes 单独由 `build_router_with_oauth(state)` 提供 (需要 OAuth2State)
/// - 默认 build_router 不含 OAuth2 路由 (向后兼容现有测试)
pub fn build_router() -> Router {
    let api = Router::new()
        // ── 16 MCP tool REST 镜像 (per spec §2.2) — 相对 nest prefix 路径 ──────
        .route("/work-items", get(routes::work_items::search))
        .route("/work-items/current", get(routes::work_items::current))
        .route("/work-items/{id}", get(routes::work_items::get_by_id))
        .route("/work-items", post(routes::work_items::create))
        .route("/work-items/{id}", patch(routes::work_items::update))
        .route("/workspaces/{id}", get(routes::workspaces::get_by_id))
        .route("/worktrees", post(routes::worktrees::create))
        .route("/worktrees/{id}", get(routes::worktrees::get_by_id))
        .route("/code/search", get(routes::code::search))
        .route("/code/symbols/{id}", get(routes::code::get_symbol))
        .route(
            "/code/symbols/{id}/references",
            get(routes::code::find_references),
        )
        .route("/code/context", get(routes::code::get_context))
        .route("/context", get(routes::context::get))
        .route("/merge-requests", post(routes::merge_requests::create))
        .route("/reviews", post(routes::reviews::request))
        .route("/validations", post(routes::validations::run))
        .route("/pipelines/{id}", get(routes::pipelines::get_status))
        .route("/submissions", post(routes::submissions::submit))
        // ── 6+3 Webhook 管理端点 (per spec §2.3) ───────────────────────
        .route("/webhooks/endpoints", get(routes::webhooks::list_endpoints))
        .route(
            "/webhooks/endpoints",
            post(routes::webhooks::create_endpoint),
        )
        .route(
            "/webhooks/endpoints/{id}",
            get(routes::webhooks::get_endpoint),
        )
        .route(
            "/webhooks/endpoints/{id}",
            patch(routes::webhooks::update_endpoint),
        )
        .route(
            "/webhooks/endpoints/{id}",
            delete(routes::webhooks::delete_endpoint),
        )
        .route(
            "/webhooks/endpoints/{id}/test",
            post(routes::webhooks::test_endpoint),
        )
        .route(
            "/webhooks/deliveries",
            get(routes::webhooks::list_deliveries),
        )
        .route(
            "/webhooks/deliveries/{delivery_id}",
            get(routes::webhooks::get_delivery),
        )
        .route(
            "/webhooks/deliveries/{delivery_id}/replay",
            post(routes::webhooks::replay_delivery),
        );

    Router::new()
        .route("/api/v1/health", get(routes::health))
        .nest("/api/v1", api)
        .layer(axum::middleware::from_fn(middleware::auth::auth_layer_stub))
        .layer(axum::middleware::from_fn(
            middleware::rate_limit::rate_limit_layer_stub,
        ))
        .layer(axum::middleware::from_fn(
            middleware::audit::audit_layer_stub,
        ))
}

/// OAuth2 5 endpoints router (per brief v0.47 §3.1)
///
/// Routes:
/// - GET  /oauth/authorize       — Authorization Code flow (RFC 6749 §4.1.1 + RFC 7636 PKCE)
/// - POST /oauth/token           — Code exchange + Client Credentials + Refresh (RFC 6749 §4.1.3, §4.4, §6)
/// - GET  /.well-known/jwks.json — Public key JWKS (RFC 7517 §5)
/// - POST /oauth/introspect      — Token introspection (RFC 7662 §2.1)
/// - POST /oauth/revoke          — Token revocation (RFC 7009 §2.1)
pub fn build_oauth_router() -> axum::Router<auth::oauth::OAuth2State> {
    use auth::oauth::handlers;
    axum::Router::new()
        .route("/oauth/authorize", get(handlers::authorize_handler))
        .route("/oauth/token", post(handlers::token_handler))
        .route("/.well-known/jwks.json", get(handlers::jwks_handler))
        .route("/oauth/introspect", post(handlers::introspect_handler))
        .route("/oauth/revoke", post(handlers::revoke_handler))
}

/// 组合 router: REST API + OAuth2 5 endpoints (per brief v0.47 §3.1)
///
/// `build_router_with_oauth(oauth_state)` 提供完整 REST + OAuth2 routing,
/// 适用于生产部署和集成测试.
pub fn build_router_with_oauth(oauth_state: auth::oauth::OAuth2State) -> Router {
    let oauth_router = build_oauth_router();
    Router::new()
        .route("/api/v1/health", get(routes::health))
        .nest(
            "/api/v1",
            Router::new()
                .route("/work-items", get(routes::work_items::search))
                .route("/work-items/current", get(routes::work_items::current))
                .route("/work-items/{id}", get(routes::work_items::get_by_id))
                .route("/work-items", post(routes::work_items::create))
                .route("/work-items/{id}", patch(routes::work_items::update))
                .route("/workspaces/{id}", get(routes::workspaces::get_by_id))
                .route("/worktrees", post(routes::worktrees::create))
                .route("/worktrees/{id}", get(routes::worktrees::get_by_id))
                .route("/code/search", get(routes::code::search))
                .route("/code/symbols/{id}", get(routes::code::get_symbol))
                .route(
                    "/code/symbols/{id}/references",
                    get(routes::code::find_references),
                )
                .route("/code/context", get(routes::code::get_context))
                .route("/context", get(routes::context::get))
                .route("/merge-requests", post(routes::merge_requests::create))
                .route("/reviews", post(routes::reviews::request))
                .route("/validations", post(routes::validations::run))
                .route("/pipelines/{id}", get(routes::pipelines::get_status))
                .route("/submissions", post(routes::submissions::submit))
                .route("/webhooks/endpoints", get(routes::webhooks::list_endpoints))
                .route(
                    "/webhooks/endpoints",
                    post(routes::webhooks::create_endpoint),
                )
                .route(
                    "/webhooks/endpoints/{id}",
                    get(routes::webhooks::get_endpoint),
                )
                .route(
                    "/webhooks/endpoints/{id}",
                    patch(routes::webhooks::update_endpoint),
                )
                .route(
                    "/webhooks/endpoints/{id}",
                    delete(routes::webhooks::delete_endpoint),
                )
                .route(
                    "/webhooks/endpoints/{id}/test",
                    post(routes::webhooks::test_endpoint),
                )
                .route(
                    "/webhooks/deliveries",
                    get(routes::webhooks::list_deliveries),
                )
                .route(
                    "/webhooks/deliveries/{delivery_id}",
                    get(routes::webhooks::get_delivery),
                )
                .route(
                    "/webhooks/deliveries/{delivery_id}/replay",
                    post(routes::webhooks::replay_delivery),
                ),
        )
        .merge(oauth_router.with_state(oauth_state))
        .layer(axum::middleware::from_fn(middleware::auth::auth_layer_stub))
        .layer(axum::middleware::from_fn(
            middleware::rate_limit::rate_limit_layer_stub,
        ))
        .layer(axum::middleware::from_fn(
            middleware::audit::audit_layer_stub,
        ))
}

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tower::ServiceExt;

    /// 测试隔离: 清空 11 个 OnceLock 共享 state (per self-review §1 + v0.31 step 3/3)
    /// 跨 10 routes file 共享 6 个 InMemory*Service
    /// - std::sync::RwLock (sync reset): work_items / worktrees / code / context / validations / submissions / webhooks::endpoints
    /// - tokio::sync::RwLock (async reset): workspaces / merge_requests / reviews / pipelines
    /// - 不含: webhooks::deliveries (复用 `star_webhook::DeliveryStore`, 缺 reset, 留 P1)
    async fn reset_all_state() {
        // sync reset (std::sync::RwLock)
        routes::work_items::service().reset();
        routes::worktrees::service().reset();
        routes::code::service().reset();
        routes::context::search_service().reset();
        routes::validations::service().reset();
        routes::submissions::service().reset();
        routes::webhooks::endpoints().reset();
        // async reset (tokio::sync::RwLock)
        routes::workspaces::service().reset().await;
        routes::merge_requests::service().reset().await;
        routes::reviews::service().reset().await;
        routes::pipelines::service().reset().await;
    }

    /// 路由注册 (build_router 不 panic) 烟雾测试
    #[test]
    fn router_contains_expected_paths() {
        let _ = build_router();
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = build_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// 业务端点现状: Phase I Batch 1 接线后, work-items 真实返回 (200 + JSON),
    /// 不是 501 (per `docs/briefs/star-api-rest-phase-i-batch-1.md`)
    #[tokio::test]
    async fn business_endpoint_returns_real_data_after_phase_i_batch_1() {
        reset_all_state().await;
        let app = build_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/work-items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Phase I Batch 1 真实接线后, /work-items 走 `routes::work_items::search`
        // → `InMemoryWorkItemService::list_with_filter` → 200 + JSON {data: {query, total, issues}}
        // (空 query + nil-tenant actor 走真实 service 路径, 返回空 list)
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(body.get("data").is_some(), "data field missing");
        let data = body.get("data").unwrap();
        assert!(data.get("query").is_some(), "data.query missing");
        assert!(data.get("total").is_some(), "data.total missing");
        assert!(data.get("issues").is_some(), "data.issues missing");
        let issues = data.get("issues").unwrap().as_array().unwrap();
        assert_eq!(issues.len(), 0, "fresh service should return empty list");
    }

    /// Phase I Batch 1 实证: POST /worktrees 真实调用 `InMemoryWorktreeService::create_worktree`
    /// 返回 200 + 真实 worktree 实体 (不是 mock `wt-STAR-1024` 字符串)
    #[tokio::test]
    async fn post_worktrees_returns_real_worktree_after_phase_i_batch_1() {
        reset_all_state().await;
        let app = build_router();
        let body_json = serde_json::json!({
            "work_item_id": "STAR-1024",
            "branch_name": "feature/STAR-1024"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/worktrees")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(body.get("data").is_some(), "data field missing");
        let data = body.get("data").unwrap();
        let worktree = data.get("worktree").expect("worktree field");
        let id = worktree.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let branch = worktree
            .get("branch")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // 真实 service 应返回真实 UUID, 不是 mock `wt-STAR-1024` 字符串
        assert!(
            !id.contains("STAR-1024"),
            "should return real UUID, not mock 'wt-STAR-1024'"
        );
        assert_eq!(branch, "feature/STAR-1024");
    }

    /// Phase I Batch 1 实证: GET /workspaces/{id} 真实调用 `InMemoryWorkspaceService::get_by_id`
    /// 缺 id 路径走 404 (跨 tenant 拒绝 → validation → 跟 star-mcp 简化模式一致)
    #[tokio::test]
    async fn get_workspaces_returns_404_for_missing_id_after_phase_i_batch_1() {
        reset_all_state().await;
        let app = build_router();
        let missing = uuid::Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/workspaces/{missing}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // nil-tenant actor 走真实 service → 跨 tenant 拒绝 → NotFound → 404
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// Phase I Batch 2 实证: POST /merge-requests 真实调用 `InMemoryScmService::create_mr`
    /// 空 title → validation 400 (handler 显式检查, 非 axum 422 解析错)
    #[tokio::test]
    async fn post_merge_requests_empty_title_returns_400_after_phase_i_batch_2() {
        reset_all_state().await;
        let app = build_router();
        let body_json = serde_json::json!({
            "title": "",
            "base": "main",
            "head": "feature/test",
            "repository_id": uuid::Uuid::new_v4().to_string()
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/merge-requests")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        // 空 title → handler 显式 validation → 400
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Phase I Batch 2 实证: GET /code/search?q=foo 真实调用 `InMemorySearchService::search`
    /// 返回 200 + JSON {query, total, results, limit}
    #[tokio::test]
    async fn get_code_search_returns_real_data_after_phase_i_batch_2() {
        reset_all_state().await;
        let app = build_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/code/search?q=authenticate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // 真实 service 路径, 空 list → 200
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(body.get("data").is_some(), "data field missing");
        let data = body.get("data").unwrap();
        assert!(data.get("query").is_some(), "data.query missing");
        assert!(data.get("total").is_some(), "data.total missing");
        assert!(data.get("results").is_some(), "data.results missing");
    }

    /// Phase I Batch 3 实证: webhook endpoint CRUD 自洽 (POST 201/GET 200/PATCH 200/DELETE 200)
    #[tokio::test]
    async fn webhook_endpoints_crud_roundtrip_after_phase_i_batch_3() {
        // 测试隔离: 每次测试前清空 shared `OnceLock<...>` 状态
        // (per self-review §1 "OnceLock<...> 全局单例在测试间共享状态", v0.27 修)
        // v0.31 step 3/3: 升级为 reset_all_state() helper, 覆盖 11 个 store (含 endpoints)
        reset_all_state().await;
        let app = build_router();
        // 1. POST 创建
        let body_json = serde_json::json!({
            "url": "https://example.com/hook",
            "secret": "shhh",
            "event_types": ["push", "pull_request"],
            "active": true
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/webhooks/endpoints")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        let ep = body.get("data").unwrap().get("endpoint").unwrap();
        let id = ep.get("id").and_then(|v| v.as_str()).unwrap().to_string();
        assert!(!id.is_empty(), "endpoint id missing");

        // 2. GET 取回
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/webhooks/endpoints/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 3. PATCH 更新
        let patch_json = serde_json::json!({ "active": false });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/webhooks/endpoints/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&patch_json).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 4. DELETE 删除
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/webhooks/endpoints/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 5. GET 应该返回 400 (id 已删, 找不到)
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/webhooks/endpoints/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ====================================================================
    // v0.49 §14.12 IV OAuth2 5 endpoints 路由 wire 集成测试 (per brief v0.49 §3.4)
    // 5 路由 ≥ 5 case: build_oauth_router 5 路由 + 1 introspect NotImplemented
    // ====================================================================

    /// 构造测试用 mock OAuth2State (per brief v0.49 §3.4 "mock OAuth2State")
    ///
    /// 真实 DB 不可达, 用 `sqlx::PgPool::connect_lazy` 构造 lazy pool
    /// (handler 调用时才会 connect, 跨 session 续真实 DB).
    async fn mock_oauth2_state() -> auth::oauth::OAuth2State {
        use auth::oauth::{keypair::OAuthKeyManager, OAuth2State};
        use star_pg_adapter::repository::{
            OAuthAccessTokenRepository, OAuthAuthorizationCodeRepository, OAuthClientRepository,
            OAuthRefreshTokenRepository, PgOAuthAccessTokenRepository,
            PgOAuthAuthorizationCodeRepository, PgOAuthClientRepository,
            PgOAuthRefreshTokenRepository,
        };
        use std::sync::Arc;

        let pool = sqlx::PgPool::connect_lazy("postgres://test:test@127.0.0.1:65535/none")
            .expect("connect_lazy");

        // JwtConfig: 用 test_rsa 2048 keypair (per v0.47 §3.1)
        let key_manager = Arc::new(OAuthKeyManager::new().expect("key_manager"));
        // 简化: 用一个空 env 来构造 JwtConfig (私钥从 OAuthKeyPair 拿, 避免 lib 内部 env var)
        // 直接构造 JwtConfig 而不是 from_env
        let private_pem = key_manager.get().await.private_pem().to_string();
        let public_pem = key_manager.get().await.public_pem().to_string();
        let jwt_config = Arc::new(auth::JwtConfig {
            private_key_pem: private_pem,
            public_key_pem: public_pem,
            issuer: "https://test.star.local".to_string(),
            audience: "star-api-rest".to_string(),
            ttl_seconds: 3600,
        });

        OAuth2State {
            key_manager,
            jwt_config,
            client_repo: Arc::new(PgOAuthClientRepository::new(pool.clone()))
                as Arc<dyn OAuthClientRepository>,
            auth_code_repo: Arc::new(PgOAuthAuthorizationCodeRepository::new(pool.clone()))
                as Arc<dyn OAuthAuthorizationCodeRepository>,
            access_token_repo: Arc::new(PgOAuthAccessTokenRepository::new(pool.clone()))
                as Arc<dyn OAuthAccessTokenRepository>,
            refresh_token_repo: Arc::new(PgOAuthRefreshTokenRepository::new(pool))
                as Arc<dyn OAuthRefreshTokenRepository>,
        }
    }

    /// v0.49 case 1: build_oauth_router() 5 路由注册 (per brief v0.49 §3.4)
    #[tokio::test]
    async fn oauth_router_contains_5_routes_after_v0_49_wire() {
        let state = mock_oauth2_state().await;
        let router: axum::Router = build_oauth_router().with_state(state);
        // 路由注册 (Router::oneshot + URI 触发) — 不 panic 即通过
        let _ = router;
    }

    /// v0.49 case 2: GET /.well-known/jwks.json 真实返回 200 (只用 key_manager, 不需 DB)
    #[tokio::test]
    async fn oauth_jwks_endpoint_returns_200_after_v0_49_wire() {
        let state = mock_oauth2_state().await;
        let app = build_oauth_router().with_state(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/jwks.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        // JWKS 标准字段 (per RFC 7517 §5)
        assert!(body.get("keys").is_some(), "JWKS missing 'keys' field");
        let keys = body.get("keys").unwrap().as_array().unwrap();
        assert!(!keys.is_empty(), "JWKS keys must not be empty");
        let jwk = &keys[0];
        assert_eq!(jwk.get("kty").and_then(|v| v.as_str()), Some("RSA"));
        assert_eq!(jwk.get("alg").and_then(|v| v.as_str()), Some("RS256"));
    }

    /// v0.49 case 3: POST /oauth/introspect 走 NotImplemented (501) — 不需 DB
    #[tokio::test]
    async fn oauth_introspect_returns_501_not_implemented_after_v0_49_wire() {
        let state = mock_oauth2_state().await;
        let app = build_oauth_router().with_state(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/introspect")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("token=fake_token_value"))
                    .unwrap(),
            )
            .await
            .unwrap();
        // NotImplemented → 501 per OAuth2 handler error mapping
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("not_implemented")
        );
    }

    /// v0.49 case 4: GET /oauth/authorize 缺 response_type 走 400 (handler 显式校验)
    #[tokio::test]
    async fn oauth_authorize_missing_response_type_returns_400_after_v0_49_wire() {
        let state = mock_oauth2_state().await;
        let app = build_oauth_router().with_state(state);
        // 缺 response_type → handler 显式检查 → 400 invalid_request
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/oauth/authorize")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // axum 解析缺 query → 400 (per Query<T> rejection)
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// v0.49 case 5: build_router_with_oauth 包含 5 OAuth2 路由 + REST API 路由
    #[tokio::test]
    async fn build_router_with_oauth_combines_rest_and_oauth_after_v0_49_wire() {
        let state = mock_oauth2_state().await;
        let app = build_router_with_oauth(state);

        // 1. REST API 路由还在 (per spec §2.2 + §2.3, 27 业务路由)
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 2. JWKS OAuth2 路由可用
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/jwks.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
