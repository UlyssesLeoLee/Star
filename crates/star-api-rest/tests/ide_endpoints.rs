/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/ide_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/ide_endpoints.rs::ide endpoints suite",kind:"test_suite"})
MERGE (open:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::open_doc",kind:"function"})
MERGE (insert:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::insert",kind:"function"})
MERGE (sync:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::sync_text",kind:"function"})
MERGE (outline:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::outline",kind:"function"})
MERGE (scmlog:Symbol {id:"crates/star-api-rest/src/routes/ide.rs::scm_log",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(open)
MERGE (suite)-[:TESTS]->(insert)
MERGE (suite)-[:TESTS]->(sync)
MERGE (suite)-[:TESTS]->(outline)
MERGE (suite)-[:TESTS]->(scmlog)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! ide 端点集成测试 (per 2026-11-17 routes/ide.rs)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! 注: routes/mod.rs 由 parent 统一注册 (per 任务约束 "不要碰 routes/mod.rs"),
//! 注册前本测试经 `#[path]` 直接引入 routes/ide.rs 编译; parent 完成
//! `pub mod ide;` + `.nest("/ide", ...)` 后本测试依旧有效 (两处各自编译,
//! 无符号冲突)。routes/ide.rs 因此以 `star_api_rest::` 库名路径引用
//! error/response 模块, 双场景均解析成功。

// `#[path]` 引入的 ide 模块 pub 项在本测试 crate 内不可达, 会触发 workspace
// unreachable_pub=deny; 注册进 lib 后 (star_api_rest::routes::ide) 可达即无此问题,
// 故仅在测试 crate 根豁免 (per terminal_endpoints 注册前后的同款演进)。
#![allow(unreachable_pub)]
//!
//! 覆盖:
//! - version 探测 / open → insert → undo → redo 编辑语义 (case 1)
//! - 无活动文档时编辑原语 404 (case 2)
//! - 多标签 activate/close (case 3)
//! - 全文本 sync + export + mark_saved (case 4)
//! - outline: 带 text 返回真实语义节点; 无 text 且无 IDE_PROJECT_ROOT 诚实降级 (case 5)
//! - scm/log 在非 git 目录返回 error 字段 (200, 不报 500) (case 6)

#[path = "../src/routes/ide.rs"]
mod ide;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

/// 隔离 app: /api/v1/ide (fresh per-test state, 不走 OnceLock)
fn isolated_app() -> Router {
    Router::new().nest(
        "/api/v1/ide",
        ide::router_with_state(ide::default_state()),
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

fn post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("build request")
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("build request")
}

/// data.view 取 text 的快捷方式。
fn view_text(body: &Value) -> String {
    body["data"]["view"]["text"].as_str().unwrap_or_default().to_string()
}

#[tokio::test]
async fn version_probes_kernel() {
    let app = isolated_app();
    let (status, body) = send(&app, get("/api/v1/ide/version")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["service"], "ide-kernel");
    assert!(!body["data"]["kernel_version"].as_str().unwrap_or_default().is_empty());
}

#[tokio::test]
async fn open_insert_undo_redo_roundtrip() {
    let app = isolated_app();
    // open
    let (status, body) = send(
        &app,
        post(
            "/api/v1/ide/doc/open",
            json!({ "path": "a.txt", "text": "ac" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "ac");
    assert_eq!(body["data"]["view"]["dirty"], false, "刚打开不应 dirty");

    // caret + insert "b" → "abc"
    let (status, _body) = send(&app, post("/api/v1/ide/doc/caret", json!({ "row": 0, "col": 1 }))).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = send(
        &app,
        post("/api/v1/ide/doc/insert", json!({ "row": 0, "col": 1, "text": "b" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "abc");
    assert_eq!(body["data"]["view"]["dirty"], true);

    // undo → "ac"; redo → "abc"
    let (status, body) = send(&app, post("/api/v1/ide/doc/undo", json!({}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "ac");
    let (status, body) = send(&app, post("/api/v1/ide/doc/redo", json!({}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "abc");
}

#[tokio::test]
async fn edit_ops_without_active_doc_return_404() {
    let app = isolated_app();
    let (status, body) = send(&app, post("/api/v1/ide/doc/insert", json!({ "row": 0, "col": 0, "text": "x" }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");
    let (status, _) = send(&app, post("/api/v1/ide/doc/undo", json!({}))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn tabs_activate_and_close() {
    let app = isolated_app();
    send(&app, post("/api/v1/ide/doc/open", json!({ "path": "a.txt", "text": "AAA" }))).await;
    send(&app, post("/api/v1/ide/doc/open", json!({ "path": "b.txt", "text": "BBB" }))).await;

    let (status, body) = send(&app, get("/api/v1/ide/doc/tabs")).await;
    assert_eq!(status, StatusCode::OK);
    let tabs = body["data"]["tabs"].as_array().expect("tabs array");
    assert_eq!(tabs.len(), 2);
    // 第二个是活动文档
    let (status, body) = send(&app, get("/api/v1/ide/doc/active")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "BBB");

    // activate 第一个
    let first_id = tabs[0]["id"].as_u64().expect("id");
    let (status, body) = send(&app, post("/api/v1/ide/doc/activate", json!({ "id": first_id }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "AAA");

    // close 第一个 → 活动回到 b
    let (status, body) = send(&app, post("/api/v1/ide/doc/close", json!({ "id": first_id }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "BBB");
}

#[tokio::test]
async fn sync_export_mark_saved() {
    let app = isolated_app();
    send(&app, post("/api/v1/ide/doc/open", json!({ "path": "crlf.txt", "text": "a\r\nb" }))).await;

    // sync 整段替换 (undo 栈重置)
    let (status, body) = send(
        &app,
        post("/api/v1/ide/doc/sync", json!({ "text": "x\ny\nz" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view_text(&body), "x\ny\nz");

    // export 按声明行尾符 (sync 的 LF 文本 → 检测为 LF → LF 导出;
    // CRLF 往返由 ide-kernel 自身测试 export_preserves_crlf 覆盖)
    let (status, body) = send(&app, get("/api/v1/ide/doc/export")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["path"], "crlf.txt");
    assert_eq!(body["data"]["text"], "x\ny\nz");

    // dirty → mark_saved → clean
    let (status, body) = send(&app, post("/api/v1/ide/doc/mark_saved", json!({}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["view"]["dirty"], false);
}

#[tokio::test]
async fn outline_with_text_returns_semantic_nodes() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post(
            "/api/v1/ide/outline",
            json!({ "path": "test.rs", "text": "fn main() {}\nstruct Foo { x: i32 }\nimpl Foo { fn bar(&self) {} }" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["language"], "Rust");
    let entries = body["data"]["entries"].as_array().expect("entries");
    assert!(!entries.is_empty(), "Rust outline 不应为空");
    let names: Vec<&str> = entries
        .iter()
        .map(|e| e["name"].as_str().unwrap_or_default())
        .collect();
    assert!(names.contains(&"main"), "names = {names:?}");
    assert!(names.contains(&"Foo"));

    // 无 text 且无 IDE_PROJECT_ROOT → 诚实降级 (200 + error 字段)
    let (status, body) = send(&app, post("/api/v1/ide/outline", json!({ "path": "test.rs" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["error"].as_str().is_some(), "应带 error 说明");
    assert_eq!(body["data"]["entries"].as_array().expect("empty entries").len(), 0);
}

#[tokio::test]
async fn scm_log_non_git_root_degrades_honestly() {
    let app = isolated_app();
    // 指向一个非 git 临时目录 → 200 + error 字段 (不 500)
    let temp = std::env::temp_dir().join(format!("ide-kernel-scm-{}", std::process::id()));
    std::fs::create_dir_all(&temp).expect("temp dir");
    let (status, body) = send(
        &app,
        get(&format!("/api/v1/ide/scm/log?n=5&root={}", temp.display())),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["error"].as_str().is_some(), "非 git 目录应带 error");
    assert_eq!(body["data"]["commits"].as_array().expect("empty commits").len(), 0);
    let _ = std::fs::remove_dir_all(&temp);
}

#[tokio::test]
async fn find_and_replace_all() {
    let app = isolated_app();
    send(&app, post("/api/v1/ide/doc/open", json!({ "path": "f.txt", "text": "cat cat cat" }))).await;

    let (status, body) = send(
        &app,
        post("/api/v1/ide/doc/find", json!({ "pattern": "cat", "case_sensitive": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["hit"]["start_col"], 0);

    let (status, body) = send(
        &app,
        post(
            "/api/v1/ide/doc/replace_all",
            json!({ "pattern": "cat", "replacement": "dog", "case_sensitive": true, "limit": 100 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["replaced"], 3);
    let (_, body) = send(&app, get("/api/v1/ide/doc/active")).await;
    assert_eq!(view_text(&body), "dog dog dog");
}

#[tokio::test]
async fn path_safety_rejects_escape() {
    let root = std::path::Path::new("C:/tmp/root");
    assert!(ide::safe_join(root, "../etc/passwd").is_none());
    assert!(ide::safe_join(root, "C:/absolute").is_none());
    assert!(ide::safe_join(root, "a/b.txt").is_some());
}
