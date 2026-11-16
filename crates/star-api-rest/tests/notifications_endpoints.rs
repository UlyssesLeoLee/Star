/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/notifications_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/notifications_endpoints.rs::notifications endpoints suite",kind:"test_suite"})
MERGE (listNotifs:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_notifications",kind:"function"})
MERGE (unread:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::unread_count",kind:"function"})
MERGE (createNotif:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_notification",kind:"function"})
MERGE (patchRead:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::patch_notification_read",kind:"function"})
MERGE (markRead:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::mark_notification_read",kind:"function"})
MERGE (listComments:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::list_comments",kind:"function"})
MERGE (createComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::create_comment",kind:"function"})
MERGE (deleteComment:Symbol {id:"crates/star-api-rest/src/routes/notifications.rs::delete_comment",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(listNotifs)
MERGE (suite)-[:TESTS]->(unread)
MERGE (suite)-[:TESTS]->(createNotif)
MERGE (suite)-[:TESTS]->(patchRead)
MERGE (suite)-[:TESTS]->(markRead)
MERGE (suite)-[:TESTS]->(listComments)
MERGE (suite)-[:TESTS]->(createComment)
MERGE (suite)-[:TESTS]->(deleteComment)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! notifications + comments 端点集成测试 (per 2026-11-17 routes/notifications.rs)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! routes/mod.rs 已注册 `pub mod notifications;`, lib.rs 两处 build_router 已
//! `.nest("/notifications", routes::notifications::router())` +
//! `.nest("/comments", routes::notifications::comments_router())`; 本测试经
//! lib crate 引用 `star_api_rest::routes::notifications`, 用
//! `default_state()` + `router_with_state` / `comments_router_with_state`
//! 构造 per-test 隔离 router (不走 OnceLock 生产共享 state, per 守门 #25 v25)。
//!
//! 覆盖:
//! - 创建通知 → 列表回显 (tenant 过滤 + total) (case 1)
//! - unread-count 随 PATCH read 变化 + unread_only 过滤 (case 2)
//! - kind 过滤 + limit/offset 分页 + 非法 kind → 400 (case 3)
//! - POST /{id}/read 标记已读 + 幂等 (case 4)
//! - 不存在通知 → 404 (PATCH / POST read 双端点) (case 5)
//! - 创建校验: 缺 tenant_id/kind/subject/body + 非法 kind/target_type +
//!   target 成对守卫 + 可选字段回显 (case 6)
//! - 评论 CRUD 往返 (作者=X-User-Id, 过滤, 删除, 已删 404) (case 7)
//! - 评论删除权限: 非作者 → 403 POLICY_DENIED (case 8)
//! - 评论端点缺 X-User-Id 头 → 400 (case 9)
//! - 通知列表新在前排序 (case 10)
//! - unread-count 按 tenant 隔离 (case 11)
//! - 评论创建校验: 缺必填字段 → 400 (case 12)

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use star_api_rest::routes::notifications;
use tower::ServiceExt;

/// 隔离 app: /api/v1/notifications + /api/v1/comments 共享一个 fresh state
/// (镜像 parent 注册后的生产形态: 同一 state_arc 供两个 nest 使用)
fn isolated_app() -> Router {
    let state = notifications::default_state();
    Router::new()
        .nest(
            "/api/v1/notifications",
            notifications::router_with_state(state.clone()),
        )
        .nest(
            "/api/v1/comments",
            notifications::comments_router_with_state(state),
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

/// POST JSON helper (带 X-User-Id 当前用户头)
fn post_json(path: &str, user_id: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .header("x-user-id", user_id)
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap()
}

/// PATCH JSON helper (带 X-User-Id 当前用户头)
fn patch_json(path: &str, user_id: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("PATCH")
        .uri(path)
        .header("content-type", "application/json")
        .header("x-user-id", user_id)
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap()
}

/// GET helper (带 X-User-Id 当前用户头)
fn get_uri(path: &str, user_id: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("x-user-id", user_id)
        .body(Body::empty())
        .unwrap()
}

/// DELETE helper (带 X-User-Id 当前用户头)
fn delete_uri(path: &str, user_id: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(path)
        .header("x-user-id", user_id)
        .body(Body::empty())
        .unwrap()
}

/// 创建通知 → (status, body)
async fn create_notification(
    app: &Router,
    tenant_id: &str,
    kind: &str,
    subject: &str,
) -> (StatusCode, Value) {
    send(
        app,
        post_json(
            "/api/v1/notifications",
            "system-actor",
            json!({
                "tenant_id": tenant_id,
                "kind": kind,
                "subject": subject,
                "body": format!("body of {subject}"),
            }),
        ),
    )
    .await
}

/// case 1: 创建通知 → 列表回显 (tenant 过滤 + total)
#[tokio::test]
async fn notification_create_then_list_roundtrip() {
    let app = isolated_app();
    let (status, body) = create_notification(&app, "tenant-1", "review_request", "评审请求").await;
    assert_eq!(status, StatusCode::OK, "POST /notifications should 200");
    let notif = &body["data"]["notification"];
    let id = notif["id"].as_str().unwrap().to_string();
    assert_eq!(notif["kind"], "review_request");
    assert_eq!(notif["read"], false);
    assert_eq!(notif["tenant_id"], "tenant-1");
    assert_eq!(notif["subject"], "评审请求");
    assert!(notif["created_at"].as_str().unwrap().contains('T'));

    let (status, body) = send(
        &app,
        get_uri("/api/v1/notifications?tenant_id=tenant-1", "any"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"], 1);
    let list = body["data"]["notifications"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], id.as_str());

    // 别的 tenant 看不到
    let (_, body) = send(
        &app,
        get_uri("/api/v1/notifications?tenant_id=tenant-2", "any"),
    )
    .await;
    assert_eq!(body["data"]["total"], 0);
}

/// case 2: unread-count 随 PATCH read 变化 + unread_only 过滤
#[tokio::test]
async fn unread_count_tracks_read_state() {
    let app = isolated_app();
    create_notification(&app, "tenant-1", "comment", "n1").await;
    let (_, body) = create_notification(&app, "tenant-1", "mention", "n2").await;
    let id2 = body["data"]["notification"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications/unread-count?tenant_id=tenant-1",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 2);

    // PATCH read=true → 未读减一
    let (status, body) = send(
        &app,
        patch_json(
            &format!("/api/v1/notifications/{id2}"),
            "any",
            json!({ "read": true }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["notification"]["read"], true);

    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications/unread-count?tenant_id=tenant-1",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 1);

    // unread_only=true → 只回未读
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications?tenant_id=tenant-1&unread_only=true",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);
    assert_eq!(body["data"]["notifications"].as_array().unwrap().len(), 1);

    // unread_only=false → 全量
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications?tenant_id=tenant-1&unread_only=false",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 2);
}

/// case 3: kind 过滤 + limit/offset 分页 + 非法 kind → 400
#[tokio::test]
async fn list_kind_filter_and_pagination() {
    let app = isolated_app();
    create_notification(&app, "tenant-1", "comment", "c1").await;
    create_notification(&app, "tenant-1", "comment", "c2").await;
    create_notification(&app, "tenant-1", "mention", "m1").await;

    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications?tenant_id=tenant-1&kind=comment",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 2);

    // limit=1 → 1 条, total 不变
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications?tenant_id=tenant-1&kind=comment&limit=1",
            "any",
        ),
    )
    .await;
    let page = body["data"]["notifications"].as_array().unwrap();
    assert_eq!(page.len(), 1);
    assert_eq!(body["data"]["total"], 2);
    let first_id = page[0]["id"].as_str().unwrap().to_string();

    // offset=1 → 下一条 (非第一条)
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications?tenant_id=tenant-1&kind=comment&limit=1&offset=1",
            "any",
        ),
    )
    .await;
    let page = body["data"]["notifications"].as_array().unwrap();
    assert_eq!(page.len(), 1);
    assert_ne!(page[0]["id"].as_str().unwrap(), first_id.as_str());
    assert_eq!(body["data"]["offset"], 1);

    // 非法 kind → 400 VALIDATION_FAILED
    let (status, body) = send(
        &app,
        get_uri("/api/v1/notifications?tenant_id=tenant-1&kind=spam", "any"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
}

/// case 4: POST /{id}/read 标记已读 + 幂等
#[tokio::test]
async fn mark_read_endpoint_sets_read_and_is_idempotent() {
    let app = isolated_app();
    let (_, body) = create_notification(&app, "tenant-1", "system", "sys").await;
    let id = body["data"]["notification"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/notifications/{id}/read"),
            "any",
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["notification"]["read"], true);

    // 幂等: 再次标记 → 仍 200 read:true
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/notifications/{id}/read"),
            "any",
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["notification"]["read"], true);

    // 未读归零
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications/unread-count?tenant_id=tenant-1",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 0);
}

/// case 5: 不存在通知 → 404 (PATCH / POST read 双端点)
#[tokio::test]
async fn missing_notification_returns_404() {
    let app = isolated_app();
    let missing = uuid::Uuid::new_v4().to_string();

    let (status, body) = send(
        &app,
        patch_json(
            &format!("/api/v1/notifications/{missing}"),
            "any",
            json!({ "read": true }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");

    let (status, _) = send(
        &app,
        post_json(
            &format!("/api/v1/notifications/{missing}/read"),
            "any",
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// case 6: 创建校验 — 缺必填字段 / 非法枚举 / target 成对守卫 / 可选字段回显
#[tokio::test]
async fn create_notification_validates_required_fields() {
    let app = isolated_app();

    // 缺 tenant_id
    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({ "kind": "comment", "subject": "s", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");

    // 缺 kind
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({ "tenant_id": "t1", "subject": "s", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 缺 subject
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({ "tenant_id": "t1", "kind": "comment", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 空 body
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({ "tenant_id": "t1", "kind": "comment", "subject": "s", "body": "  " }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 非法 kind
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({ "tenant_id": "t1", "kind": "spam", "subject": "s", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 非法 target_type
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({
                "tenant_id": "t1", "kind": "system", "subject": "s", "body": "b",
                "target_type": "nope", "target_id": "x"
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // target_id 单独出现 (target 成对守卫)
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({ "tenant_id": "t1", "kind": "system", "subject": "s", "body": "b", "target_id": "x" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 合法 + 可选字段全量 → 200 回显
    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/notifications",
            "any",
            json!({
                "tenant_id": "t1", "kind": "mention", "subject": "s", "body": "b",
                "subject_url": "/mr/1", "actor_id": "u-1", "actor_name": "张三",
                "target_type": "merge_request", "target_id": "mr-1"
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let notif = &body["data"]["notification"];
    assert_eq!(notif["kind"], "mention");
    assert_eq!(notif["subject_url"], "/mr/1");
    assert_eq!(notif["actor_id"], "u-1");
    assert_eq!(notif["actor_name"], "张三");
    assert_eq!(notif["target_type"], "merge_request");
    assert_eq!(notif["target_id"], "mr-1");
    assert_eq!(notif["read"], false);
}

/// case 7: 评论 CRUD 往返 (作者=X-User-Id, 过滤, 删除, 已删 404)
#[tokio::test]
async fn comments_crud_roundtrip() {
    let app = isolated_app();
    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "alice",
            json!({
                "tenant_id": "tenant-1",
                "target_type": "work_item",
                "target_id": "wi-1",
                "body": "LGTM",
                "author_name": "Alice",
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "POST /comments should 200");
    let comment = &body["data"]["comment"];
    let id = comment["id"].as_str().unwrap().to_string();
    assert_eq!(comment["author_id"], "alice");
    assert_eq!(comment["author_name"], "Alice");
    assert_eq!(comment["target_type"], "work_item");
    assert_eq!(comment["body"], "LGTM");
    assert!(comment["created_at"].as_str().unwrap().contains('T'));

    // 过滤列表
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/comments?target_type=work_item&target_id=wi-1",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);
    let list = body["data"]["comments"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], id.as_str());

    // 不匹配的 target_id → 0
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/comments?target_type=work_item&target_id=other",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 0);

    // 无过滤全量
    let (_, body) = send(&app, get_uri("/api/v1/comments", "any")).await;
    assert_eq!(body["data"]["total"], 1);

    // author_name 缺省回落为 X-User-Id
    let (_, body) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "bob",
            json!({ "tenant_id": "tenant-1", "target_type": "work_item", "target_id": "wi-1", "body": "+1" }),
        ),
    )
    .await;
    assert_eq!(body["data"]["comment"]["author_name"], "bob");

    // 删除自己的 → 200
    let (status, body) = send(&app, delete_uri(&format!("/api/v1/comments/{id}"), "alice")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["deleted"], true);
    assert_eq!(body["data"]["comment_id"], id.as_str());

    // 已删 → 列表归零, 再删 404
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/comments?target_type=work_item&target_id=wi-1",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 1, "bob 的评论仍在");
    let (status, _) = send(&app, delete_uri(&format!("/api/v1/comments/{id}"), "alice")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// case 8: 评论删除权限 — 非作者 → 403 POLICY_DENIED
#[tokio::test]
async fn comment_delete_requires_ownership() {
    let app = isolated_app();
    let (_, body) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "alice",
            json!({
                "tenant_id": "tenant-1", "target_type": "merge_request", "target_id": "mr-1", "body": "hi"
            }),
        ),
    )
    .await;
    let id = body["data"]["comment"]["id"].as_str().unwrap().to_string();

    let (status, body) = send(&app, delete_uri(&format!("/api/v1/comments/{id}"), "bob")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    // 评论仍在
    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/comments?target_type=merge_request&target_id=mr-1",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);

    // 作者可删
    let (status, _) = send(&app, delete_uri(&format!("/api/v1/comments/{id}"), "alice")).await;
    assert_eq!(status, StatusCode::OK);
}

/// case 9: 评论端点缺 X-User-Id 头 → 400 VALIDATION_FAILED
#[tokio::test]
async fn comment_endpoints_require_x_user_id_header() {
    let app = isolated_app();

    // POST 创建无头
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/comments")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "tenant_id": "t1", "target_type": "work_item", "target_id": "w", "body": "b"
            }))
            .unwrap(),
        ))
        .unwrap();
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");

    // DELETE 无头 (任意 id, 校验先于 404)
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/comments/{}", uuid::Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// case 10: 通知列表新在前 (created_at DESC)
#[tokio::test]
async fn notification_list_orders_newest_first() {
    let app = isolated_app();
    let (_, b1) = create_notification(&app, "tenant-1", "system", "first").await;
    let (_, b2) = create_notification(&app, "tenant-1", "system", "second").await;
    let id_first = b1["data"]["notification"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let id_second = b2["data"]["notification"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (_, body) = send(
        &app,
        get_uri("/api/v1/notifications?tenant_id=tenant-1", "any"),
    )
    .await;
    let list = body["data"]["notifications"].as_array().unwrap();
    assert_eq!(list[0]["id"], id_second.as_str());
    assert_eq!(list[1]["id"], id_first.as_str());
}

/// case 11: unread-count 按 tenant 隔离
#[tokio::test]
async fn unread_count_scoped_by_tenant() {
    let app = isolated_app();
    create_notification(&app, "tenant-a", "system", "a1").await;
    create_notification(&app, "tenant-b", "system", "b1").await;

    let (_, body) = send(
        &app,
        get_uri(
            "/api/v1/notifications/unread-count?tenant_id=tenant-a",
            "any",
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 1);

    // 无 tenant 过滤 → 全量
    let (_, body) = send(&app, get_uri("/api/v1/notifications/unread-count", "any")).await;
    assert_eq!(body["data"]["count"], 2);
}

/// case 12: 评论创建校验 — 缺必填字段 → 400
#[tokio::test]
async fn create_comment_validates_required_fields() {
    let app = isolated_app();

    // 缺 body
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "alice",
            json!({ "tenant_id": "t1", "target_type": "work_item", "target_id": "w" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 缺 target_type
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "alice",
            json!({ "tenant_id": "t1", "target_id": "w", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 缺 target_id
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "alice",
            json!({ "tenant_id": "t1", "target_type": "work_item", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 缺 tenant_id
    let (status, _) = send(
        &app,
        post_json(
            "/api/v1/comments",
            "alice",
            json!({ "target_type": "work_item", "target_id": "w", "body": "b" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
