/*
@cypher schema=1 source_sha256=328ea5a01ddce8afc2b36a1fd815a4e0f5d7fdb89b7396228c32f8abb74e5464
MERGE (self:File {path:"crates/star-api-rest/tests/terminal_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/terminal_endpoints.rs::terminal endpoints suite",kind:"test_suite"})
MERGE (create:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::create_pane",kind:"function"})
MERGE (getpane:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::get_pane",kind:"function"})
MERGE (append:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::append_line",kind:"function"})
MERGE (scrollback:Symbol {id:"crates/star-api-rest/src/routes/terminal.rs::get_scrollback",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(create)
MERGE (suite)-[:TESTS]->(getpane)
MERGE (suite)-[:TESTS]->(append)
MERGE (suite)-[:TESTS]->(scrollback)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! terminal 端点集成测试 (per 2026-11-16 routes/terminal.rs)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! 注: routes/mod.rs 由 parent 统一注册 (per 任务约束 "不要碰 routes/mod.rs"),
//! 注册前本测试经 `#[path]` 直接引入 routes/terminal.rs 编译; parent 完成
//! `pub mod terminal;` + `.nest("/terminal", ...)` 后本测试依旧有效 (两处各自
//! 编译, 无符号冲突)。routes/terminal.rs 因此以 `star_api_rest::` 库名路径
//! 引用 error/response 模块, 双场景均解析成功。
//!
//! 覆盖:
//! - 创建 pane → 写行 → 拉全量滚动缓冲 → 读 pane 元数据 (case 1)
//! - ?since_seq 增量拉取 (case 2)
//! - 不存在 pane → 404 (3 个 pane 作用域端点) (case 3)
//! - 空行 / 非法 source → 400 (case 4)
//! - 非法 pane_id 路径 → 400 (case 5)

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

/// 隔离 app: /api/v1/terminal (fresh per-test state, 不走 OnceLock)
fn isolated_app() -> Router {
    Router::new().nest(
        "/api/v1/terminal",
        star_api_rest::routes::terminal::router_with_state(
            star_api_rest::routes::terminal::default_state(),
        ),
    )
}

/// oneshot 发送 + 解析 JSON body
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp: Response = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 256 * 1024)
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

/// 创建 pane 并返回 (status, body)
async fn create_pane(app: &Router, worktree_id: Uuid, title: &str) -> (StatusCode, Value) {
    send(
        app,
        post_json(
            "/api/v1/terminal/pane",
            json!({ "worktree_id": worktree_id, "title": title }),
        ),
    )
    .await
}

/// case 1: 创建 → 写 2 行 → 全量滚动缓冲 (seq 1..2) → pane 元数据 roundtrip
#[tokio::test]
async fn create_write_pull_scrollback_and_meta_roundtrip() {
    let app = isolated_app();
    let worktree = Uuid::new_v4();

    // 1. POST 创建 pane
    let (status, body) = create_pane(&app, worktree, "Task CLI").await;
    assert_eq!(status, StatusCode::OK, "POST /pane should 200");
    let pane_id = body["data"]["pane_id"]
        .as_str()
        .expect("pane_id")
        .to_string();
    assert_eq!(body["data"]["worktree_id"], worktree.to_string());
    assert_eq!(body["data"]["title"], "Task CLI");
    assert!(body["data"]["created_at_ms"].as_i64().unwrap() > 0);

    // 2. POST 写 2 行 (默认 stdout + 显式 stderr)
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/terminal/pane/{pane_id}/line"),
            json!({ "line": "hello" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["seq"].as_i64(), Some(1));

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/terminal/pane/{pane_id}/line"),
            json!({ "line": "world", "source": "stderr" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["seq"].as_i64(), Some(2));

    // 3. GET 全量滚动缓冲
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/terminal/pane/{pane_id}/scrollback")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["max_seq"].as_i64(), Some(2));
    let lines = body["data"]["lines"].as_array().expect("lines");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["seq"].as_i64(), Some(1));
    assert_eq!(lines[0]["text"], "hello");
    assert_eq!(lines[0]["source"], "stdout");
    assert_eq!(lines[1]["seq"].as_i64(), Some(2));
    assert_eq!(lines[1]["text"], "world");
    assert_eq!(lines[1]["source"], "stderr");

    // 4. GET pane 元数据
    let (status, body) = send(&app, get_uri(&format!("/api/v1/terminal/pane/{pane_id}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["pane_id"], pane_id.as_str());
    assert_eq!(body["data"]["worktree_id"], worktree.to_string());
    assert_eq!(body["data"]["title"], "Task CLI");
    assert_eq!(body["data"]["pane_count"].as_u64(), Some(1));
    assert_eq!(body["data"]["depth"].as_u64(), Some(0));
    assert_eq!(body["data"]["kind"], "single");
}

/// case 2: ?since_seq 增量拉取 (since=1 → seq 2..3; since=max → 空 lines)
#[tokio::test]
async fn scrollback_since_seq_incremental_replay() {
    let app = isolated_app();
    let (_, body) = create_pane(&app, Uuid::new_v4(), "inc").await;
    let pane_id = body["data"]["pane_id"].as_str().unwrap().to_string();

    for (i, text) in ["a", "b", "c"].iter().enumerate() {
        let (status, body) = send(
            &app,
            post_json(
                &format!("/api/v1/terminal/pane/{pane_id}/line"),
                json!({ "line": text }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["seq"].as_i64(), Some(i as i64 + 1));
    }

    // since_seq=1 → 只返回 seq 2, 3
    let (status, body) = send(
        &app,
        get_uri(&format!(
            "/api/v1/terminal/pane/{pane_id}/scrollback?since_seq=1"
        )),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["max_seq"].as_i64(), Some(3));
    let lines = body["data"]["lines"].as_array().expect("lines");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["seq"].as_i64(), Some(2));
    assert_eq!(lines[0]["text"], "b");
    assert_eq!(lines[1]["seq"].as_i64(), Some(3));
    assert_eq!(lines[1]["text"], "c");

    // since_seq=3 (已到 max) → 空 lines, max_seq 不变
    let (status, body) = send(
        &app,
        get_uri(&format!(
            "/api/v1/terminal/pane/{pane_id}/scrollback?since_seq=3"
        )),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["max_seq"].as_i64(), Some(3));
    assert_eq!(body["data"]["lines"].as_array().expect("lines").len(), 0);
}

/// case 3: 不存在 pane → 404 (GET pane / POST line / GET scrollback 三端点)
#[tokio::test]
async fn missing_pane_returns_404_for_all_scoped_endpoints() {
    let app = isolated_app();
    let missing = Uuid::new_v4();

    let (status, body) = send(&app, get_uri(&format!("/api/v1/terminal/pane/{missing}"))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/terminal/pane/{missing}/line"),
            json!({ "line": "x" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");

    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/terminal/pane/{missing}/scrollback")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");
}

/// case 4: 空行 / 非法 source → 400 VALIDATION_FAILED
#[tokio::test]
async fn append_line_rejects_empty_text_and_bad_source() {
    let app = isolated_app();
    let (_, body) = create_pane(&app, Uuid::new_v4(), "v").await;
    let pane_id = body["data"]["pane_id"].as_str().unwrap().to_string();

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/terminal/pane/{pane_id}/line"),
            json!({ "line": "" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/terminal/pane/{pane_id}/line"),
            json!({ "line": "x", "source": "bogus" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
}

/// case 5: 非法 pane_id 路径参数 → 400 (axum Path<Uuid> rejection)
#[tokio::test]
async fn invalid_pane_id_path_rejected_with_400() {
    let app = isolated_app();
    let resp: Response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/terminal/pane/not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
