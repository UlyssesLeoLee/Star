// SPDX-License-Identifier: MIT OR Apache-2.0
//! AuthLayer stub (per spec §1.3)
//!
//! MVP 阶段: no-op (per OPT-NEXT-06-code-stub §3.1 3 中间件 stub).
//! P2 阶段 worker 子代理实装:
//! - 抽 `Authorization: Bearer <api_key>` header
//! - 查 `api_keys` 表 (M 类, SCD Type 2, per spec §5.1)
//! - 验 key 状态 (`active`) + 验 bcrypt/Argon2id hash (per spec §6 G-09)
//! - 检查 scope (`developer.read` / `developer.write` / `webhook.manage` 等)
//! - IP 白名单 (per spec §6 G-03, P2 阶段)
//! - 通过 → 注入 `ActorContext` 到 request extension (per P0-1 联动审计, AGENTS.md §4.1 v16)
//!
//! v0 phase 2 stub 标记 (per OPT-WORKER-01 模式): tracing 标注 `feature=AuthLayer`,
//! suggestion 引用 `Phase M+ worker delegation per AGENTS.md §4 #20`.

use axum::{extract::Request, middleware::Next, response::Response};

/// 鉴权中间件 stub — 当前 no-op
///
/// v0 phase 2 实装标记 (per OPT-NEXT-06-code-stub §3.1 3 中间件):
/// - 现状: pass-through, 不注入 ActorContext
/// - P2 阶段: 派 worker 子代理实装 (派前必先 `automation/dispatcher.py brief(...)`)
pub async fn auth_layer_stub(req: Request, next: Next) -> Response {
    tracing::debug!(
        feature = "AuthLayer",
        suggestion = "Phase M+ worker delegation per AGENTS.md §4 #20",
        method = %req.method(),
        uri = %req.uri(),
        "AuthLayer stub: pass-through (P2 phase real impl pending)"
    );
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// AuthLayer 中间件 pass-through 验证 (per OPT-NEXT-06-code-stub §3.1)
    #[tokio::test]
    async fn auth_layer_passes_through() {
        // 构造一个简单 axum app 走 AuthLayer, 验证不阻断
        use axum::{routing::get, Router};
        async fn hello() -> &'static str {
            "ok"
        }
        let app = Router::new()
            .route("/test", get(hello))
            .layer(axum::middleware::from_fn(auth_layer_stub));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("authorization", "Bearer stub_key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // AuthLayer pass-through → handler 返回 200
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
