//! canvas-realtime lib tests
//!
//! 守门 #1 v25 cargo test 单 crate 实证 + 守门 #11 缺标比错标.

#[test]
fn service_metadata_is_skeleton() {
    let m = canvas_realtime::ServiceMetadata::CANVAS_REALTIME;
    assert_eq!(m.name, "canvas-realtime");
    assert_eq!(m.http_port, 8082);
    assert_eq!(m.ws_port, 8083);
    assert_eq!(m.phase, canvas_realtime::Phase::Skeleton);
}

#[test]
fn service_metadata_version_matches_pkg() {
    let m = canvas_realtime::ServiceMetadata::CANVAS_REALTIME;
    assert_eq!(m.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn phase_serializes() {
    let p = canvas_realtime::Phase::Skeleton;
    let s = serde_json::to_string(&p).expect("serialize");
    assert_eq!(s, "\"Skeleton\"");
}

#[tokio::test]
async fn router_healthz_returns_ok() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    let app = canvas_realtime::router();
    let response = app
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(&body_bytes[..], b"ok");
}

#[tokio::test]
async fn router_ready_reports_skeleton_phase() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    let app = canvas_realtime::router();
    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body_bytes).expect("json");
    assert_eq!(v["ready"], true);
    assert_eq!(v["phase"], "skeleton");
}