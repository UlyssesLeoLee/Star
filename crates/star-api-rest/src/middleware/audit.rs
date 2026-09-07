// SPDX-License-Identifier: MIT OR Apache-2.0
//! AuditLayer stub (per spec §1.4 + §5.1)
//!
//! MVP 阶段: no-op (per OPT-NEXT-06-code-stub §3.1 3 中间件 stub).
//! P2 阶段 worker 子代理实装:
//! - 每个 request 落 T 类 audit (per spec §5.1, 物理删除禁止 + append-only + RLS 13 类必携)
//! - ActorType: API Key 调 → `ActorType.Automation` (per spec §1.4)
//! - 审计表: `api_key_audit_log` (T 类, per spec §5.1 4 表)
//! - 字段: `api_key_id, request_id, method, path, status_code, actor_type, actor_id, tenant_id, ts`
//!
//! v0 phase 2 stub 标记 (per OPT-WORKER-01 模式): tracing 标注 `feature=AuditLayer`.

use axum::{extract::Request, middleware::Next, response::Response};

/// 审计中间件 stub — 当前 no-op
///
/// v0 phase 2 实装标记 (per OPT-NEXT-06-code-stub §3.1 3 中间件):
/// - 现状: pass-through, 不落 T 类 audit
/// - P2 阶段: 派 worker 子代理实装 api_key_audit_log append-only (per spec §5.1)
pub async fn audit_layer_stub(req: Request, next: Next) -> Response {
    tracing::debug!(
        feature = "AuditLayer",
        suggestion = "Phase M+ worker delegation per AGENTS.md §4 #20",
        method = %req.method(),
        uri = %req.uri(),
        "AuditLayer stub: pass-through (P2 phase real impl pending, T-class api_key_audit_log)"
    );
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// AuditLayer 中间件 pass-through 验证 (per OPT-NEXT-06-code-stub §3.1)
    #[tokio::test]
    async fn audit_layer_passes_through() {
        use axum::{routing::get, Router};
        async fn hello() -> &'static str {
            "ok"
        }
        let app = Router::new()
            .route("/test", get(hello))
            .layer(axum::middleware::from_fn(audit_layer_stub));
        let resp = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
        // AuditLayer pass-through → handler 返回 200
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
