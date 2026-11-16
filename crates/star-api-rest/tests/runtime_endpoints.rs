/*
@cypher schema=1 source_sha256=7800b425931b5e31bfd4e0837d265ea70f0fe6912d66715a4a652409b58646f0
MERGE (self:File {path:"crates/star-api-rest/tests/runtime_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/runtime_endpoints.rs::runtime endpoints suite",kind:"test_suite"})
MERGE (runtimeRouter:Symbol {id:"crates/star-api-rest/src/routes/runtime.rs::router_with_state",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:TESTS]->(runtimeRouter)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! runtime 端点集成测试 (per 2026-11-16 routes/runtime.rs, domain-local-runtime 接线)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot,
//! 用 caller-supplied state (`router_with_state` + `default_state`) 隔离。
//!
//! 覆盖:
//! - 全链路: 注册 → 心跳 → 列表 → 挂载 → 心跳列表 (case 1)
//! - 列表按 user_id 过滤 (case 2)
//! - 未知 runtime_id 心跳/挂载 → 404 RUNTIME_NOT_FOUND (case 3/4)
//! - nil tenant 注册 → 400 VALIDATION_FAILED (case 5)
//! - 同 (tenant,user,device) 重复注册 → 409 RUNTIME_CONFLICT (case 6)

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use star_api_rest::routes::runtime::{default_state, router_with_state};
use tower::ServiceExt;
use uuid::Uuid;

/// 隔离 app: /api/v1/runtime (fresh state per test)
fn isolated_app() -> Router {
    Router::new().nest("/api/v1/runtime", router_with_state(default_state()))
}

/// oneshot 发送 + 解析 JSON body
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp: Response = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).expect("json");
    (status, body)
}

/// POST JSON helper
fn post_json(path: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap()
}

/// GET helper
fn get_uri(path: &str) -> Request<Body> {
    Request::builder().uri(path).body(Body::empty()).unwrap()
}

/// register 请求体 (字段与 RegisterRuntimeCommand 一一对应)
fn register_body(tenant_id: Uuid, user_id: Uuid, device_id: Uuid, hostname: &str) -> Value {
    json!({
        "tenant_id": tenant_id,
        "user_id": user_id,
        "device_id": device_id,
        "version": "1.0.0",
        "capabilities": ["git", "rust"],
        "metadata": { "host_name": hostname }
    })
}

/// case 1: 全链路 注册 → 心跳 → 列表 → 挂载 → 心跳列表
#[tokio::test]
async fn register_heartbeat_list_mount_heartbeats_roundtrip() {
    let app = isolated_app();
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    // 1. POST /register → 200 + runtime_id
    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/runtime/register",
            json!({
                "tenant_id": tenant_id,
                "user_id": user_id,
                "device_id": Uuid::new_v4(),
                "version": "1.0.0",
                "capabilities": ["git", "rust"],
                "metadata": { "host_name": "dev-box-1" }
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "register should 200: {body}");
    let runtime_id = body["data"]["runtime_id"]
        .as_str()
        .expect("runtime_id")
        .to_string();
    assert_eq!(body["data"]["status"], "ONLINE");
    assert_eq!(body["data"]["runtime"]["user_id"], user_id.to_string());
    // 实体 JSON 中 status 为枚举派生序列化 ("Online"); 顶层 status 走 as_str() ("ONLINE")
    assert_eq!(body["data"]["runtime"]["status"], "Online");
    assert_eq!(
        body["data"]["runtime"]["metadata"]["host_name"],
        "dev-box-1"
    );

    // 2. POST /{runtime_id}/heartbeat → 200 accepted
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/runtime/{runtime_id}/heartbeat"),
            json!({
                "tenant_id": tenant_id,
                "load_average": 0.42,
                "memory_used_bytes": 512 * 1024 * 1024_u64
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "heartbeat should 200: {body}");
    assert_eq!(body["data"]["accepted"], true);
    assert_eq!(body["data"]["runtime_id"], runtime_id.as_str());
    assert_eq!(body["data"]["status"], "ONLINE");

    // 3. GET /api/v1/runtime → 1 条, metadata.host_name 一致
    let (status, body) = send(&app, get_uri("/api/v1/runtime")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(1));
    let runtimes = body["data"]["runtimes"].as_array().expect("runtimes");
    assert_eq!(runtimes[0]["id"], runtime_id.as_str());
    assert_eq!(runtimes[0]["status"], "Online");

    // 4. POST /{runtime_id}/mount → 200 mounted
    let worktree_id = Uuid::new_v4();
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/runtime/{runtime_id}/mount"),
            json!({
                "tenant_id": tenant_id,
                "worktree_id": worktree_id,
                "local_path": "D:\\worktrees\\wt-1"
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "mount should 200: {body}");
    assert_eq!(body["data"]["mounted"], true);
    assert_eq!(body["data"]["worktree_id"], worktree_id.to_string());
    // 实体 JSON 中 MountStatus 为枚举派生序列化 ("Active"); as_str() 形式才是 "ACTIVE"
    assert_eq!(body["data"]["mount"]["status"], "Active");

    // 5. GET /{runtime_id}/heartbeats → 1 条, 载荷回显
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/runtime/{runtime_id}/heartbeats")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(1));
    let heartbeats = body["data"]["heartbeats"].as_array().expect("heartbeats");
    // f32 经 JSON 往返有精度误差, 容差比较
    let load_average = heartbeats[0]["load_average"]
        .as_f64()
        .expect("load_average");
    assert!(
        (load_average - 0.42).abs() < 1e-6,
        "load_average round-trip: {load_average}"
    );
    assert_eq!(heartbeats[0]["memory_used_bytes"], 512 * 1024 * 1024_u64);
}

/// case 2: GET /api/v1/runtime?user_id= 过滤
#[tokio::test]
async fn list_runtimes_filters_by_user_id() {
    let app = isolated_app();
    let tenant_id = Uuid::new_v4();
    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();

    for user in [user_a, user_b] {
        let (status, _) = send(
            &app,
            post_json(
                "/api/v1/runtime/register",
                register_body(tenant_id, user, Uuid::new_v4(), "box"),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // 全量: 2 条
    let (status, body) = send(&app, get_uri("/api/v1/runtime")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(2));

    // 过滤 user_a: 1 条
    let (status, body) = send(&app, get_uri(&format!("/api/v1/runtime?user_id={user_a}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(1));
    assert_eq!(body["data"]["runtimes"][0]["user_id"], user_a.to_string());

    // 未知 user: 0 条
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/runtime?user_id={}", Uuid::new_v4())),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(0));
}

/// case 3: 未知 runtime_id 心跳 → 404 RUNTIME_NOT_FOUND
#[tokio::test]
async fn heartbeat_unknown_runtime_returns_404() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/runtime/{}/heartbeat", Uuid::new_v4()),
            json!({ "tenant_id": Uuid::new_v4() }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(body["error"]["code"], "RUNTIME_NOT_FOUND");
}

/// case 4: 未知 runtime_id 挂载 → 404 RUNTIME_NOT_FOUND
#[tokio::test]
async fn mount_unknown_runtime_returns_404() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/runtime/{}/mount", Uuid::new_v4()),
            json!({
                "tenant_id": Uuid::new_v4(),
                "worktree_id": Uuid::new_v4(),
                "local_path": "/tmp/wt"
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(body["error"]["code"], "RUNTIME_NOT_FOUND");
}

/// case 5: nil tenant 注册 → 400 VALIDATION_FAILED (INV-RT-01, 防 ActorContext::new panic)
#[tokio::test]
async fn register_nil_tenant_returns_400() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/runtime/register",
            json!({
                "tenant_id": Uuid::nil(),
                "user_id": Uuid::new_v4(),
                "device_id": Uuid::new_v4(),
                "version": "1.0.0"
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
}

/// case 6: 同 (tenant, user, device) 重复注册 → 409 RUNTIME_CONFLICT
#[tokio::test]
async fn duplicate_device_register_returns_409() {
    let app = isolated_app();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let first = send(
        &app,
        post_json(
            "/api/v1/runtime/register",
            register_body(tenant_id, user_id, device_id, "box"),
        ),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);

    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/runtime/register",
            register_body(tenant_id, user_id, device_id, "box"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");
    assert_eq!(body["error"]["code"], "RUNTIME_CONFLICT");
}
