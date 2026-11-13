//! canvas-engine 路由边界 + 枚举 serde 集成测试 (WBS 9.6)
//!
//! 在 tests/smoke.rs (5 用例基线) 之外补:
//! - /ready /api/v1/services / 三个未覆盖端点
//! - 404 / 405 路由边界
//! - Phase::Implemented serde + CanvasEngineError Display
//! - ServiceMetadata Copy 语义

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn router_ready_returns_json() {
    let app = canvas_engine::router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["service"], "canvas-engine");
    assert_eq!(v["ready"], true);
    assert_eq!(v["phase"], "skeleton");
}

#[tokio::test]
async fn router_services_status_returns_json() {
    let app = canvas_engine::router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/services")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["service"], "canvas-engine");
    assert_eq!(v["role"], "Star Canvas platform service");
    assert_eq!(v["phase"], "skeleton");
    let endpoints = v["endpoints"].as_array().expect("endpoints array");
    assert_eq!(
        endpoints,
        &[
            serde_json::json!("/healthz"),
            serde_json::json!("/ready"),
            serde_json::json!("/version"),
            serde_json::json!("/api/v1/services"),
        ]
    );
}

#[tokio::test]
async fn router_root_returns_html_with_title() {
    let app = canvas_engine::router();
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(content_type, "text/html; charset=utf-8");
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("<title>Star Canvas Engine</title>"), "{html}");
    assert!(html.contains("<h1>Star Canvas Engine</h1>"));
    // 导航链接齐全
    for link in ["/healthz", "/ready", "/version", "/api/v1/services"] {
        assert!(html.contains(link), "missing nav link {link}");
    }
}

#[tokio::test]
async fn router_unknown_path_returns_404() {
    let app = canvas_engine::router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn router_wrong_method_returns_405() {
    let app = canvas_engine::router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn router_version_json_matches_metadata() {
    let app = canvas_engine::router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let meta = canvas_engine::ServiceMetadata::CANVAS_ENGINE;
    assert_eq!(v["service"], meta.name);
    assert_eq!(v["version"], meta.version);
    // phase 序列化后是 variant 名 ("Skeleton")
    assert_eq!(
        v["phase"],
        serde_json::to_value(meta.phase).expect("serialize phase")
    );
}

#[test]
fn phase_implemented_serializes() {
    let json = serde_json::to_string(&canvas_engine::Phase::Implemented).unwrap();
    assert_eq!(json, "\"Implemented\"");
    let json = serde_json::to_string(&canvas_engine::Phase::Skeleton).unwrap();
    assert_eq!(json, "\"Skeleton\"");
    assert_ne!(
        canvas_engine::Phase::Skeleton,
        canvas_engine::Phase::Implemented
    );
}

#[test]
fn canvas_engine_error_display() {
    let e = canvas_engine::CanvasEngineError::NotImplemented;
    assert_eq!(
        e.to_string(),
        "canvas-engine service behavior is not implemented"
    );
    let _: &dyn std::error::Error = &e;
    let _: &dyn std::fmt::Debug = &e;
}

#[test]
fn service_metadata_is_copy_and_fields() {
    let m = canvas_engine::ServiceMetadata::CANVAS_ENGINE;
    let m2 = m; // Copy
    assert_eq!(m.name, m2.name);
    assert_eq!(m.phase, m2.phase);
    assert_eq!(m.http_port, m2.http_port);
    assert_eq!(m.version, m2.version);
    assert!(!m.version.is_empty());
}
