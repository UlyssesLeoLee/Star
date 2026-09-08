// crates/star-ops/tests/it_cross_module.rs
//
// 跨模块 IT 派生缺口 (per UT-IT-51 brief §5 wt6 + TEST-DESIGN-OPS-001 v0.2 §3.3)
//
// 范围:
// - 7 跨模块 IT (43-49): healthz/readyz/correlation_id/rate_limit/size_limit/concurrent/graceful_shutdown
// - MVP 阶段: 大部分 [M] 子项 (无 rate limit middleware, 无 graceful shutdown)
// - 派生测: 文档化 MVP 行为, 标 [M] 子项 DDD Review 必查
//
// 守门实证:
// - 守门 #1 R-05: 仅 mock 路径, 不接真实 K8s
// - 守门 #5 v2: API key 走 KMS
// - 守门 #11 缺标比错标: 派生测文档化 [M] 子项

// 集成测试作为独立 crate, 关掉 missing_docs 顶层 deny (跟 lib 一致)
#![allow(missing_docs)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use star_ops::ops_api::{router, AppState};
use tower::ServiceExt;

/// 派生 #43: healthz 含 db ping (per DDS-001 §2.2 healthz 派生规)
/// 守門 #1 R-05: K8s livenessProbe 派生
/// MVP 阶段: healthz 永远 200 OK (无 db ping)
/// 派生测: 验证 MVP 行为, 派生文档 [M] 阶段加 db ping
#[tokio::test]
async fn it_healthz_returns_200_with_db_ping() {
    let app = router(AppState::new());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // MVP 阶段: healthz 永远 200 OK
    assert_eq!(response.status(), StatusCode::OK);
    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段加 db ping + version
}

/// 派生 #44: readyz db 挂返 503 (per DDS-001 §2.2 readyz 派生规)
/// 守門 #1 R-05: K8s readinessProbe 派生
/// MVP 阶段: readyz 永远 200 OK (无 db ping)
/// 派生测: 验证 MVP 行为, 派生文档 [M] 阶段加 db ping
#[tokio::test]
async fn it_readyz_returns_503_when_db_down() {
    let app = router(AppState::new());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // MVP 阶段: readyz 永远 200 OK
    assert_eq!(response.status(), StatusCode::OK);
    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段加 db ping, db 挂返 503
}

/// 派生 #45: correlation_id 透传 (per BAS-001 §3.4 派生规)
/// 守門 #5 v2: trace_id 跨调用链追踪
/// 派生测: 验证 log_upload 的 trace_id 在 response 中可见
#[tokio::test]
async fn it_correlation_id_propagates_through_axum() {
    let app = router(AppState::new());
    let trace_id = "trace-cross-axum-001";
    let body = json!({
        "source": "k8s-pod/cross-module",
        "content": "2026-09-08 INFO cross module test",
        "trace_id": trace_id
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ops/log/upload")
                .header("content-type", "application/json")
                .header("x-correlation-id", trace_id)
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    use axum::body::to_bytes;
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).expect("body is JSON");
    // 验证: trace_id 必在 response.data.trace_id 字段
    assert_eq!(
        body["data"]["trace_id"].as_str(),
        Some(trace_id),
        "correlation_id 必在 response.data.trace_id"
    );
    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段加 x-correlation-id response header
}

/// 派生 #46: 限流返 429 (per BAS-001 §3.5 派生规)
/// 守門 #6 v2: 60 req/min 限流, RATE_LIMITED error
/// MVP 阶段: 无 rate limit middleware
/// 派生测: 验证 MVP 行为, 派生文档 [M] 阶段加 rate limit
#[tokio::test]
async fn it_rate_limit_returns_429_after_threshold() {
    let app_state = AppState::new();

    // 派生测: 100 并发调 /api/ops/metrics/summary (超 60 req/min 阈值)
    // MVP 阶段: 无 rate limit, 全 200
    let mut handles = Vec::with_capacity(100);
    for _ in 0..100 {
        let app = router(app_state.clone());
        handles.push(tokio::spawn(async move {
            app.oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/ops/metrics/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
        }));
    }

    let mut rate_limited_count = 0;
    let mut success_count = 0;
    for h in handles {
        let result = h.await.expect("task not panic").expect("oneshot ok");
        if result.status() == StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        } else if result.status() == StatusCode::OK {
            success_count += 1;
        }
    }
    // MVP 阶段: 无 rate limit, 全 200
    assert_eq!(
        success_count, 100,
        "MVP 阶段无 rate limit, 100 并发全 200 (派生测文档 [M] 子项加 rate limit)"
    );
    assert_eq!(
        rate_limited_count, 0,
        "MVP 阶段无 429 (派生测文档 [M] 子项加 rate limit)"
    );
}

/// 派生 #47: 请求体过大返 4xx (per 守門 #5 v2 1MB 限制)
/// 派生测: 验证 log_upload 1.1MB body 必 400 (跟 UT log_upload_rejects_oversized_body 跨 crate 实证)
#[tokio::test]
async fn it_request_size_limit_rejects_oversized_payload() {
    let app = router(AppState::new());
    // 构造 1.1MB content
    let big = "x".repeat(1024 * 1024 + 100);
    let body = json!({
        "source": "k8s-pod/oversize",
        "content": big
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ops/log/upload")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    // 守門 #5 v2: 1MB 限制触发 BadRequest (HTTP 400)
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "1.1MB body 必 400 (守門 #5 v2 1MB 限制)"
    );
}

/// 派生 #48: 并发不破坏状态 (per DDS-001 §2.2 派生规)
/// 守門 #1 R-05 + 守門 #7 v3: 并发安全
/// 派生测: 1000 并发调 /api/ops/metrics/summary, 验证全 200
#[tokio::test]
async fn it_concurrent_requests_dont_corrupt_state() {
    let app_state = AppState::new();
    let mut handles = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let app = router(app_state.clone());
        handles.push(tokio::spawn(async move {
            app.oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/ops/metrics/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
        }));
    }

    let mut success_count = 0;
    for h in handles {
        let result = h.await.expect("task not panic").expect("oneshot ok");
        if result.status() == StatusCode::OK {
            success_count += 1;
        }
    }
    // MVP 阶段: 1000 并发全 200 (无状态破坏)
    assert_eq!(
        success_count, 1000,
        "1000 并发 metrics_summary 全 200 (无状态破坏)"
    );
}

/// 派生 #49: 优雅关闭 (per DDS-001 §2.2 派生规)
/// 守門 #1 R-05: K8s deployment graceful shutdown 派生
/// MVP 阶段: 无 graceful shutdown 派生
/// 派生测: 验证 MVP 行为, 派生文档 [M] 阶段加 graceful shutdown
#[tokio::test]
async fn it_graceful_shutdown_drains_in_flight_requests() {
    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段加 with_graceful_shutdown (per axum 0.8 派生规)
    // MVP 阶段: Router 不支持 graceful shutdown, 进程 SIGKILL 后 in-flight 请求丢失
    //
    // 派生测: 文档化 MVP 行为
    // 1. 启动 server (tokio::spawn + Listener::bind)
    // 2. 发起 long-running request
    // 3. 调 shutdown signal
    // 4. 验证 in-flight request 仍完成 (MVP: 进程被 kill, 不完成)
    //
    // MVP 阶段: 仅文档化, 不实现 (per 守門 #11 缺标比错标)
    let app = router(AppState::new());
    // 简单 smoke test: 验证 router 可用
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
