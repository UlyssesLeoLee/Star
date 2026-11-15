/*
@cypher schema=1 source_sha256=573c7af274d537adf1a1c40cc1f1e6e0325ad6dd989dc61f8b39ac10ec4c4ccb
MERGE (self:File {path:"crates/star-api-rest/tests/memory_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/memory_endpoints.rs::memory endpoints suite",kind:"test_suite"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! memory + agents 端点集成测试 (per 2026-11-15 routes/memory.rs + routes/agents.rs)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! 覆盖:
//! - POST → GET 列 → GET 按 agent → DELETE 全链路 (build_router 真实接线)
//! - 按 agent 隔离 + 双层 (short_term/long_term) 列出
//! - 非法 layer → 400
//! - long_term 物理删除禁止 → 403
//! - 不存在 record → 404
//! - GET /api/v1/agents 内存样例 2-3 个

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use star_api_rest::build_router;
use tower::ServiceExt;
use uuid::Uuid;

/// 隔离 app: /api/v1/memory (fresh state) + /api/v1/agents (无状态)
fn isolated_app() -> Router {
    Router::new()
        .nest(
            "/api/v1/memory",
            star_api_rest::routes::memory::router_with_state(
                star_api_rest::routes::memory::default_state(),
            ),
        )
        .nest("/api/v1/agents", star_api_rest::routes::agents::router())
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

fn post_memory(agent_id: Uuid, layer: &str, content: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/memory")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "agent_id": agent_id,
                "layer": layer,
                "content": content,
            }))
            .unwrap(),
        ))
        .unwrap()
}

fn get_uri(path: &str) -> Request<Body> {
    Request::builder().uri(path).body(Body::empty()).unwrap()
}

fn delete_uri(path: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(path)
        .body(Body::empty())
        .unwrap()
}

/// case 1: build_router 真实接线全链路 POST → GET 列 → GET 按 agent → DELETE
#[tokio::test]
async fn memory_full_crud_roundtrip_via_build_router() {
    let app = build_router();
    let agent = Uuid::new_v4();

    // 1. POST 写入 short_term
    let (status, body) = send(
        &app,
        post_memory(agent, "short_term", json!({"note": "hello"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "POST /api/v1/memory should 200");
    let record_id = body["data"]["record_id"]
        .as_str()
        .expect("record_id")
        .to_string();
    assert_eq!(body["data"]["layer"], "short_term");
    assert!(!record_id.is_empty());

    // 2. GET /api/v1/memory 列表包含该记录
    let (status, body) = send(&app, get_uri("/api/v1/memory")).await;
    assert_eq!(status, StatusCode::OK);
    let records = body["data"]["records"].as_array().expect("records");
    assert!(body["data"]["total"].as_u64().unwrap() >= 1);
    assert!(records.iter().any(|r| r["record_id"] == record_id.as_str()));

    // 3. GET /api/v1/memory/agent/{id} 返回该 agent 全部记录
    let (status, body) = send(&app, get_uri(&format!("/api/v1/memory/agent/{agent}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(1));
    let records = body["data"]["records"].as_array().expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["record_id"], record_id.as_str());
    assert_eq!(records[0]["content"]["note"], "hello");

    // 4. DELETE /api/v1/memory/record/{id}
    let (status, body) = send(
        &app,
        delete_uri(&format!("/api/v1/memory/record/{record_id}")),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "DELETE short_term record should 200"
    );
    assert_eq!(body["data"]["deleted"], record_id.as_str());

    // 5. 删除后 agent 列表为空
    let (status, body) = send(&app, get_uri(&format!("/api/v1/memory/agent/{agent}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(0));
}

/// case 2: 按 agent 隔离 + 双层列出 (fresh state, 不走 OnceLock)
#[tokio::test]
async fn memory_list_scoped_by_agent_and_layer() {
    let app = isolated_app();
    let a1 = Uuid::new_v4();
    let a2 = Uuid::new_v4();

    let (status, _) = send(
        &app,
        post_memory(a1, "short_term", json!({"k": "a1-short"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(&app, post_memory(a1, "long_term", json!({"k": "a1-long"}))).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(&app, post_memory(a2, "long_term", json!({"k": "a2-long"}))).await;
    assert_eq!(status, StatusCode::OK);

    // 全局列表: 3 条, 双层标注齐全
    let (status, body) = send(&app, get_uri("/api/v1/memory")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(3));
    let records = body["data"]["records"].as_array().unwrap();
    let layers: Vec<&str> = records
        .iter()
        .map(|r| r["layer"].as_str().unwrap())
        .collect();
    assert_eq!(layers.iter().filter(|l| **l == "short_term").count(), 1);
    assert_eq!(layers.iter().filter(|l| **l == "long_term").count(), 2);

    // a1 只看到自己的 2 条
    let (status, body) = send(&app, get_uri(&format!("/api/v1/memory/agent/{a1}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(2));

    // a2 只看到自己的 1 条
    let (status, body) = send(&app, get_uri(&format!("/api/v1/memory/agent/{a2}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(1));

    // 未知 agent → 200 空列表
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/memory/agent/{}", Uuid::new_v4())),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"].as_u64(), Some(0));
}

/// case 3: 非法 layer → 400 VALIDATION_FAILED
#[tokio::test]
async fn memory_post_invalid_layer_returns_400() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post_memory(Uuid::new_v4(), "invalid_layer", json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
}

/// case 4: long_term 物理删除禁止 (T 派生, per 守门 #13 b) → 403
#[tokio::test]
async fn memory_delete_long_term_returns_403() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post_memory(Uuid::new_v4(), "long_term", json!({"k": "v"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let record_id = body["data"]["record_id"].as_str().unwrap().to_string();
    let (status, body) = send(
        &app,
        delete_uri(&format!("/api/v1/memory/record/{record_id}")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");
}

/// case 5: 删除不存在 record → 404 RESOURCE_NOT_FOUND
#[tokio::test]
async fn memory_delete_missing_record_returns_404() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        delete_uri(&format!("/api/v1/memory/record/{}", Uuid::new_v4())),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");
}

/// case 6: GET /api/v1/agents 返回 2-3 个内存样例 (agent-domain Agent)
#[tokio::test]
async fn agents_list_returns_memory_samples() {
    let app = build_router();
    let (status, body) = send(&app, get_uri("/api/v1/agents")).await;
    assert_eq!(status, StatusCode::OK);
    let agents = body["data"]["agents"].as_array().expect("agents array");
    assert!(
        (2..=3).contains(&agents.len()),
        "expect 2-3 sample agents, got {}",
        agents.len()
    );
    assert_eq!(body["data"]["total"].as_u64().unwrap(), agents.len() as u64);
    for a in agents {
        assert!(a["id"].is_string());
        assert!(a["name"].is_string());
        assert!(a["role"].is_string());
        assert!(a["kind"].is_string());
        assert!(a["status"].is_string());
    }
}
