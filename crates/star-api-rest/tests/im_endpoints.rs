/*
@cypher schema=1 source_sha256=f1c19eff217261f761c268a0ac06404121a40d66208536f1e1a510631f61c535
MERGE (self:File {path:"crates/star-api-rest/tests/im_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/im_endpoints.rs::im endpoints suite",kind:"test_suite"})
MERGE (createConv:Symbol {id:"crates/star-api-rest/src/routes/im.rs::create_conversation",kind:"function"})
MERGE (listConvs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_conversations",kind:"function"})
MERGE (sendMsg:Symbol {id:"crates/star-api-rest/src/routes/im.rs::send_message",kind:"function"})
MERGE (listMsgs:Symbol {id:"crates/star-api-rest/src/routes/im.rs::list_messages",kind:"function"})
MERGE (markRead:Symbol {id:"crates/star-api-rest/src/routes/im.rs::mark_read",kind:"function"})
MERGE (presence:Symbol {id:"crates/star-api-rest/src/routes/im.rs::presence",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(createConv)
MERGE (suite)-[:TESTS]->(listConvs)
MERGE (suite)-[:TESTS]->(sendMsg)
MERGE (suite)-[:TESTS]->(listMsgs)
MERGE (suite)-[:TESTS]->(markRead)
MERGE (suite)-[:TESTS]->(presence)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! im 端点集成测试 (per 2026-11-16 routes/im.rs, IM1.0 语义对齐独立实现)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! 注: routes/mod.rs 由 parent 统一注册 (per 任务约束 "不要碰 routes/mod.rs 与
//! lib.rs"), 注册前本测试经 `#[path]` 直接引入 routes/im.rs 编译; parent 完成
//! `pub mod im;` + `.nest("/im", ...)` 后本测试依旧有效 (两处各自编译, 无符号
//! 冲突, per tests/terminal_endpoints.rs 同款模式)。routes/im.rs 因此以
//! `star_api_rest::` 库名路径引用 error/response 模块, 双场景均解析成功。
//!
//! 覆盖:
//! - 创建 DM → 发消息 → 按 sequence 拉取 → 标记已读 → 未读数归零 (case 1)
//! - DM 幂等 (同对用户返回既有会话) (case 2)
//! - 群聊 title + creator=owner 角色 (case 3)
//! - after_sequence 分页 + has_more + limit (case 4)
//! - 不存在会话 → 404 (发消息/拉消息/标已读三端点) (case 5)
//! - 非成员发消息 → 403 (case 6)
//! - mark_read 上界夹紧 + 单调守卫 (case 7)
//! - reply_to 回显 + 悬空 reply_to → 400 (case 8)
//! - presence → connected:false + 空成员 (case 9)
//! - 缺 X-User-Id 头 → 400 (case 10)
//! - memberIds (camelCase) serde 别名 (case 11)

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

/// 隔离 app: /api/v1/im (fresh per-test state, 不走 OnceLock)
fn isolated_app() -> Router {
    Router::new().nest(
        "/api/v1/im",
        star_api_rest::routes::im::router_with_state(star_api_rest::routes::im::default_state()),
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

/// GET helper (带 X-User-Id 当前用户头)
fn get_uri(path: &str, user_id: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("x-user-id", user_id)
        .body(Body::empty())
        .unwrap()
}

/// 创建会话 (user 为当前用户, member_ids 为其会话成员数组) → (status, body)
async fn create_conversation(
    app: &Router,
    user_id: &str,
    member_ids: &[&str],
    title: Option<&str>,
) -> (StatusCode, Value) {
    let mut payload = json!({ "member_ids": member_ids });
    if let Some(t) = title {
        payload["title"] = Value::String(t.to_string());
    }
    send(app, post_json("/api/v1/im/conversations", user_id, payload)).await
}

/// 发消息 → (status, body)
async fn send_message(
    app: &Router,
    conv_id: &str,
    user_id: &str,
    text: &str,
) -> (StatusCode, Value) {
    send(
        app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            user_id,
            json!({ "kind": "text", "content": { "text": text } }),
        ),
    )
    .await
}

/// case 1: 创建 DM → 发消息 → 按 sequence 拉取 → 标记已读 → 未读数正确
#[tokio::test]
async fn create_dm_send_list_read_unread_roundtrip() {
    let app = isolated_app();
    let (u_a, u_b) = ("user-a", "user-b");

    // 1. u_a 创建与 u_b 的 DM
    let (status, body) = create_conversation(&app, u_a, &[u_a, u_b], None).await;
    assert_eq!(status, StatusCode::OK, "POST /conversations should 200");
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(body["data"]["kind"], "dm");
    assert_eq!(body["data"]["members"].as_array().unwrap().len(), 2);

    // 2. u_a 发 2 条
    let (status, body) = send_message(&app, &conv_id, u_a, "hi").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["sequence"], 1);
    assert_eq!(body["data"]["state"], "sent");
    let (_, body) = send_message(&app, &conv_id, u_a, "there").await;
    assert_eq!(body["data"]["sequence"], 2);

    // 3. u_b 按 sequence 拉取 (after_sequence=0 → 全量, 升序)
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/im/conversations/{conv_id}/messages"), u_b),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let messages = body["data"]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["sequence"], 1);
    assert_eq!(messages[1]["sequence"], 2);
    assert_eq!(messages[1]["content"]["text"], "there");
    assert_eq!(messages[0]["state"], "sent");

    // 4. u_b 未读 = 2 (u_a 发的 2 条, 自己发的不算)
    let (status, body) = send(&app, get_uri("/api/v1/im/conversations", u_b)).await;
    assert_eq!(status, StatusCode::OK);
    let convs = body["data"]["conversations"].as_array().unwrap();
    assert_eq!(convs.len(), 1);
    assert_eq!(convs[0]["conversation_id"], conv_id.as_str());
    assert_eq!(convs[0]["unread"], 2);
    assert_eq!(convs[0]["last_sequence"], 2);

    // 5. u_b 标记已读 sequence=2 → unread 0, 服务端回显实际值
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/read"),
            u_b,
            json!({ "sequence": 2 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["last_read_sequence"], 2);
    assert_eq!(body["data"]["advanced"], true);
    assert_eq!(body["data"]["unread"], 0);

    // 6. 未读数在列表里归零
    let (_, body) = send(&app, get_uri("/api/v1/im/conversations", u_b)).await;
    let convs = body["data"]["conversations"].as_array().unwrap();
    assert_eq!(convs[0]["unread"], 0);
}

/// case 2: DM 幂等 — 同对用户重复创建返回既有会话
#[tokio::test]
async fn dm_creation_is_idempotent_for_same_pair() {
    let app = isolated_app();
    let (u_a, u_b) = ("alice", "bob");

    let (_, body1) = create_conversation(&app, u_a, &[u_a, u_b], None).await;
    let id1 = body1["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(body1["data"]["kind"], "dm");

    // 反向发起 (u_b 创建与 u_a 的 DM) → 同一会话
    let (status, body2) = create_conversation(&app, u_b, &[u_b, u_a], None).await;
    assert_eq!(status, StatusCode::OK);
    let id2 = body2["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(id1, id2, "same pair must reuse the existing DM");

    // 会话总数仍为 1
    let (_, list) = send(&app, get_uri("/api/v1/im/conversations", u_a)).await;
    assert_eq!(list["data"]["conversations"].as_array().unwrap().len(), 1);
}

/// case 3: 群聊 — 3 人 → kind=group, title 回显, creator 为 owner
#[tokio::test]
async fn group_creation_with_title_and_owner_role() {
    let app = isolated_app();
    let (u_a, u_b, u_c) = ("owner-a", "member-b", "member-c");

    let (status, body) =
        create_conversation(&app, u_a, &[u_a, u_b, u_c], Some("Backend 小组")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["kind"], "group");
    assert_eq!(body["data"]["title"], "Backend 小组");
    let members = body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);
    let owner = members
        .iter()
        .find(|m| m["user_id"] == u_a)
        .expect("creator present");
    assert_eq!(owner["role"], "owner");
    assert!(members.iter().any(|m| m["role"] == "member"));
}

/// case 4: after_sequence 分页 + has_more + limit 截断
#[tokio::test]
async fn message_pagination_by_after_sequence() {
    let app = isolated_app();
    let (u_a, u_b) = ("pager-a", "pager-b");
    let (_, body) = create_conversation(&app, u_a, &[u_a, u_b], None).await;
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    for i in 1..=3 {
        let (status, _) = send_message(&app, &conv_id, u_a, &format!("m{i}")).await;
        assert_eq!(status, StatusCode::OK);
    }

    // after_sequence=1 → 只回 seq 2..3, next_after_sequence=3, has_more=false
    let (status, body) = send(
        &app,
        get_uri(
            &format!("/api/v1/im/conversations/{conv_id}/messages?after_sequence=1"),
            u_b,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let messages = body["data"]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["sequence"], 2);
    assert_eq!(body["data"]["next_after_sequence"], 3);
    assert_eq!(body["data"]["has_more"], false);

    // limit=1 → 1 条 + has_more=true
    let (status, body) = send(
        &app,
        get_uri(
            &format!("/api/v1/im/conversations/{conv_id}/messages?limit=1"),
            u_b,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["messages"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"]["has_more"], true);
}

/// case 5: 不存在会话 → 404 (发消息 / 拉消息 / 标已读三端点)
#[tokio::test]
async fn missing_conversation_returns_404_on_all_scoped_endpoints() {
    let app = isolated_app();
    let missing = uuid::Uuid::new_v4().to_string();
    let user = "ghost-user";

    let (status, body) = send_message(&app, &missing, user, "hello?").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "RESOURCE_NOT_FOUND");

    let (status, _) = send(
        &app,
        get_uri(
            &format!("/api/v1/im/conversations/{missing}/messages"),
            user,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{missing}/read"),
            user,
            json!({ "sequence": 1 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// case 6: 非成员发消息 / 标已读 → 403 POLICY_DENIED
#[tokio::test]
async fn non_member_cannot_send_or_mark_read() {
    let app = isolated_app();
    let (u_a, u_b, u_c) = ("member-a", "member-b", "intruder-c");
    let (_, body) = create_conversation(&app, u_a, &[u_a, u_b], None).await;
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, body) = send_message(&app, &conv_id, u_c, "intrusion").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    let (status, _) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/read"),
            u_c,
            json!({ "sequence": 1 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

/// case 7: mark_read 上界夹紧 + 单调守卫
#[tokio::test]
async fn mark_read_clamps_to_max_sequence_and_is_monotonic() {
    let app = isolated_app();
    let (u_a, u_b) = ("clamp-a", "clamp-b");
    let (_, body) = create_conversation(&app, u_a, &[u_a, u_b], None).await;
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    send_message(&app, &conv_id, u_a, "one").await;
    send_message(&app, &conv_id, u_a, "two").await;

    // 上报 999 → 夹紧到已分配最大 sequence 2 (不是 999)
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/read"),
            u_b,
            json!({ "sequence": 999 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["last_read_sequence"], 2);

    // 再上报 1 → 不倒退, advanced=false
    let (_, body) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/read"),
            u_b,
            json!({ "sequence": 1 }),
        ),
    )
    .await;
    assert_eq!(body["data"]["last_read_sequence"], 2);
    assert_eq!(body["data"]["advanced"], false);

    // 负值 → 400
    let (status, _) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/read"),
            u_b,
            json!({ "sequence": -1 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// case 8: reply_to 回显 + 悬空 reply_to → 400
#[tokio::test]
async fn reply_to_echoes_and_dangling_reply_is_rejected() {
    let app = isolated_app();
    let (u_a, u_b) = ("reply-a", "reply-b");
    let (_, body) = create_conversation(&app, u_a, &[u_a, u_b], None).await;
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (_, first) = send_message(&app, &conv_id, u_a, "original").await;
    let first_id = first["data"]["message_id"].as_str().unwrap().to_string();

    // 回复既有消息 → reply_to 回显, sequence 单调 +1
    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            u_b,
            json!({ "kind": "text", "content": { "text": "reply" }, "reply_to": first_id }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["reply_to"], first_id.as_str());
    assert_eq!(body["data"]["sequence"], 2);

    // 悬空 reply_to → 400
    let (status, _) = send(
        &app,
        post_json(
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            u_b,
            json!({ "kind": "text", "content": { "text": "x" }, "reply_to": uuid::Uuid::new_v4().to_string() }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// case 9: presence — 无真实连接 → connected:false + 空成员列表
#[tokio::test]
async fn presence_reports_disconnected_with_empty_members() {
    let app = isolated_app();
    let (status, body) = send(&app, get_uri("/api/v1/im/presence", "any-user")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["connected"], false);
    let members = body["data"]["members"].as_array().unwrap();
    assert!(members.is_empty());
}

/// case 10: 缺 X-User-Id 头 → 400 VALIDATION_FAILED
#[tokio::test]
async fn missing_user_header_returns_400() {
    let app = isolated_app();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/im/conversations")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({ "member_ids": ["a", "b"] })).unwrap(),
        ))
        .unwrap();
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");
}

/// case 11: 任务原文拼写 `memberIds` (camelCase) 作为 snake_case `member_ids` 的
/// serde 别名被接受
#[tokio::test]
async fn member_ids_camel_case_alias_is_accepted() {
    let app = isolated_app();
    let (u_a, u_b) = ("camel-a", "camel-b");
    let (status, body) = send(
        &app,
        post_json(
            "/api/v1/im/conversations",
            u_a,
            json!({ "memberIds": [u_a, u_b] }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["kind"], "dm");
    assert_eq!(body["data"]["members"].as_array().unwrap().len(), 2);
}
