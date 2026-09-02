// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/star-api-rest` — STAR Developer REST API (骨架阶段 v0.1)
//!
//! per `docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md` (sibling) §1.1 + §2 + §7.3
//!
//! ## 范围 (MVP 骨架 — 仅 routing 形状, 不接业务逻辑)
//!
//! - **22 路由 stub** (16 MCP tool REST 镜像 + 6 Webhook 管理端点), per spec §2.2 + §2.3
//! - **统一错误模型** (6-field `RestError`, per spec §2.4 复用 MCP 错误模型)
//! - **统一响应封装** (`RestResponse<T>` data + meta, per spec §2.4)
//! - **鉴权中间件 stub** (`AuthLayer` 验 `Authorization: Bearer <api_key>`, per spec §1.3)
//! - **限流中间件 stub** (`RateLimitLayer` 60 req/min per key, per spec §1.4)
//! - **审计中间件 stub** (`AuditLayer` 落 T 类 `api_key_audit_log`, per spec §5.1)
//!
//! 所有业务端点当前返 `501 Not Implemented` + `error.code = "NOT_IMPLEMENTED"`,
//! 等待 P2 阶段 (Phase M+) worker 子代理实装业务逻辑.
//! 派前必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md` (per AGENTS.md §4 #20 守门派生).
//!
//! ## 不做什么 (per AGENTS.md §0 硬约束 + 守门 #11 缺标比错标安全)
//!
//! - **不**实装业务逻辑 — 仅 routing 骨架
//! - **不**实装 DB schema (per spec §5.1 4 表 W/T/M 分类, 等 P2 阶段)
//! - **不**实装 OAuth 2.0 / mTLS (per spec §1.3, Phase 2+ 评估)
//! - **不**实装 5 域预置 vendor 模板 (per spec §4.1, 等 P2 阶段由 5 域 Lead 拍板)
//! - **不**实装 OpenAPI 3.1 spec 自动生成 (per spec §7.2 + §6 G-10, 等 v0.2 utoipa)

#![allow(missing_docs)] // 骨架阶段, 业务端点 P2 实装时补

pub mod error;
pub mod middleware;
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

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

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

    /// 业务端点当前返 501 Not Implemented (P2 阶段 worker 子代理实装时改断言)
    #[tokio::test]
    async fn business_endpoint_returns_501_not_implemented() {
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
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
