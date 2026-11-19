/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/notify_im_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::p0 notify+im suite",kind:"test_suite"})
MERGE (auth401:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::unauthenticated_notifications_and_im_reject_401",kind:"test"})
MERGE (crossTenant:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::cross_tenant_query_and_body_are_403",kind:"test"})
MERGE (recipient:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::notification_recipient_isolation",kind:"test"})
MERGE (nonMember:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::non_member_and_cross_tenant_conversation_read_403",kind:"test"})
MERGE (restart:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::restart_rebuild_state_preserves_unread_and_messages",kind:"test"})
MERGE (idem:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::duplicate_event_delivery_creates_single_notification",kind:"test"})
MERGE (unreadConsistent:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::unread_count_matches_list_and_mark_read_idempotent",kind:"test"})
MERGE (prodRouter:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::production_router_requires_valid_bearer_token",kind:"test"})
MERGE (gateway:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::im_gateway_reports_not_connected",kind:"test"})
MERGE (dmIdem:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::verified_dm_creation_is_idempotent_per_tenant",kind:"test"})
MERGE (relay:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::outbox_retry_backoff_caps_and_dead_letters",kind:"test"})
MERGE (pg:Symbol {id:"crates/star-api-rest/tests/notify_im_endpoints.rs::pg_restart_preserves_notifications_and_im",kind:"test"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! P0 通知 + IM 生产化验收测试 (2026-11-20, per TASK-p0-notify.md §4)
//!
//! 守门 #25 v25: 全部走 axum::Router::oneshot, 不发真 HTTP。
//! verified 语义: actor 经 request extension 注入 `AuthUser` (等价生产
//! middleware 验签后注入的已验证会话); 不注入 → 401 (fail-closed)。
//!
//! 验收映射:
//! - 未登录 (无已验证会话) → 401
//! - 跨租户 query/body → 403; 收件人隔离 (只回自己的 + 租户广播)
//! - 非成员/跨租户读会话 → 403
//! - 创建 → 重建 state/仓储 → 未读与消息仍在 (PG 版见 #[ignore] 验收测试)
//! - 重复事件投递 → 通知数不增 (Outbox 幂等键唯一约束)
//! - unread-count 与列表一致; mark-read 幂等

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use star_api_rest::auth::AuthUser;
use star_api_rest::routes::im::{
    self, ImApiState, ImGateway, ImGatewayError, InMemoryImStore, Message, UnconnectedImGateway,
};
use star_api_rest::routes::notifications::{
    self, InMemoryNotifyStore, Notification, NotificationKind, NotificationsApiState, NotifyRelay,
    NotifyStore, NotifyStoreError, OutboxEvent,
};
use tower::ServiceExt;
use uuid::Uuid;

/// 固定 user id (确定性, 便于断言)
fn uid(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

/// 构造已验证 actor (等价生产 middleware 注入的 AuthUser)
fn actor(user: u128, tenant: u128) -> AuthUser {
    AuthUser {
        user_id: uid(user),
        tenant_id: uid(tenant),
        roles: vec!["user".to_string()],
        scope: "notifications:read notifications:write im:read im:write".to_string(),
        email: None,
        projects: Vec::new(),
    }
}

/// 带已验证 actor extension 的请求 (body 可选)
fn request_with_actor(
    method: &str,
    uri: &str,
    actor: Option<AuthUser>,
    body: Option<Value>,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(a) = actor {
        builder = builder.extension(a);
    }
    match body {
        Some(payload) => {
            builder = builder.header("content-type", "application/json");
            builder
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    }
}

/// oneshot 发送 + 解析 JSON
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp: Response = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    (status, body)
}

/// verified 通知 app (notifications + comments 共享一个 store)
fn verified_notify_app(store: Arc<dyn NotifyStore>) -> (Router, Arc<NotificationsApiState>) {
    let state = Arc::new(NotificationsApiState::from_store(store));
    let app = Router::new()
        .nest(
            "/api/v1/notifications",
            notifications::router_with_verified_state(state.clone()),
        )
        .nest(
            "/api/v1/comments",
            notifications::comments_router_with_verified_state(state.clone()),
        );
    (app, state)
}

/// verified IM app
fn verified_im_app(store: Arc<dyn im::ImStore>) -> Router {
    let state = Arc::new(ImApiState::from_parts(
        store,
        Arc::new(UnconnectedImGateway),
    ));
    Router::new().nest("/api/v1/im", im::router_with_verified_state(state))
}

/// 创建 verified 通知 → (status, body)
async fn create_notification(
    app: &Router,
    actor: &AuthUser,
    payload: Value,
) -> (StatusCode, Value) {
    send(
        app,
        request_with_actor(
            "POST",
            "/api/v1/notifications",
            Some(actor.clone()),
            Some(payload),
        ),
    )
    .await
}

/// 创建 verified 会话 → (status, body)
async fn create_conversation(
    app: &Router,
    actor: &AuthUser,
    member_ids: &[&str],
) -> (StatusCode, Value) {
    send(
        app,
        request_with_actor(
            "POST",
            "/api/v1/im/conversations",
            Some(actor.clone()),
            Some(json!({ "member_ids": member_ids })),
        ),
    )
    .await
}

/// 发 verified 消息 → (status, body)
async fn send_message(
    app: &Router,
    actor: &AuthUser,
    conv_id: &str,
    text: &str,
) -> (StatusCode, Value) {
    send(
        app,
        request_with_actor(
            "POST",
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            Some(actor.clone()),
            Some(json!({ "kind": "text", "content": { "text": text } })),
        ),
    )
    .await
}

/// 验收 1: 未登录 (无已验证会话) → 401; X-User-Id 头不被信任
#[tokio::test]
async fn unauthenticated_notifications_and_im_reject_401() {
    let (notify_app, _) = verified_notify_app(Arc::new(InMemoryNotifyStore::new()));
    let im_app = verified_im_app(Arc::new(InMemoryImStore::new()));

    // 通知: 无 actor → 401
    let (status, body) = send(
        &notify_app,
        request_with_actor("GET", "/api/v1/notifications", None, None),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "notifications GET without session must 401"
    );
    assert_eq!(body["error"], "AUTH_REQUIRED");

    // 未读计数: 无 actor → 401
    let (status, _) = send(
        &notify_app,
        request_with_actor("GET", "/api/v1/notifications/unread-count", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 评论: 无 actor → 401
    let (status, _) = send(
        &notify_app,
        request_with_actor(
            "POST",
            "/api/v1/comments",
            None,
            Some(json!({
                "tenant_id": "t", "target_type": "work_item", "target_id": "w", "body": "b"
            })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // IM: 无 actor → 401
    let (status, _) = send(
        &im_app,
        request_with_actor("GET", "/api/v1/im/conversations", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 伪造 X-User-Id 头 (无 extension) 仍 401 — 不许 header 裸奔
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/notifications")
        .header("x-user-id", "evil-user")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(&notify_app, req).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "X-User-Id header must not bypass verified auth"
    );
}

/// 验收 2: 跨租户 query / body → 403 POLICY_DENIED
#[tokio::test]
async fn cross_tenant_query_and_body_are_403() {
    let (app, _) = verified_notify_app(Arc::new(InMemoryNotifyStore::new()));
    let alice = actor(0x1001, 0x2001); // tenant 0x2001
    let other_tenant = uid(0x2002).to_string();

    // query tenant 不符 → 403
    let (status, body) = send(
        &app,
        request_with_actor(
            "GET",
            &format!("/api/v1/notifications?tenant_id={other_tenant}"),
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "cross-tenant query must 403");
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    // unread-count query 不符 → 403
    let (status, _) = send(
        &app,
        request_with_actor(
            "GET",
            &format!("/api/v1/notifications/unread-count?tenant_id={other_tenant}"),
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // body tenant 不符 → 403
    let (status, _) = create_notification(
        &app,
        &alice,
        json!({ "tenant_id": other_tenant, "kind": "system", "subject": "s", "body": "b" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "cross-tenant create must 403"
    );

    // 评论 body tenant 不符 → 403
    let (status, _) = send(
        &app,
        request_with_actor(
            "POST",
            "/api/v1/comments",
            Some(alice.clone()),
            Some(json!({ "tenant_id": other_tenant, "target_type": "work_item", "target_id": "w", "body": "b" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

/// 验收 3: 收件人隔离 — 个人通知只回 actor 自己; 广播租户内可见;
/// 他人个人通知标记已读 → 403
#[tokio::test]
async fn notification_recipient_isolation() {
    let store: Arc<dyn NotifyStore> = Arc::new(InMemoryNotifyStore::new());
    let (app, state) = verified_notify_app(store.clone());
    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2001);
    let tenant = uid(0x2001).to_string();
    let alice_id = uid(0x1001).to_string();

    // alice 给自己创建个人通知 (recipient = alice)
    let (status, body) = create_notification(
        &app,
        &alice,
        json!({
            "tenant_id": tenant,
            "kind": "mention",
            "subject": "私信",
            "body": "b",
            "recipient_user_id": alice_id,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let notif_id = body["data"]["notification"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // bob 看不到 alice 的个人通知
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/notifications", Some(bob.clone()), None),
    )
    .await;
    assert_eq!(
        body["data"]["total"], 0,
        "bob must not see alice's personal notification"
    );

    // bob 未读计数 0, alice 未读计数 1
    let (_, body) = send(
        &app,
        request_with_actor(
            "GET",
            "/api/v1/notifications/unread-count",
            Some(bob.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 0);
    let (_, body) = send(
        &app,
        request_with_actor(
            "GET",
            "/api/v1/notifications/unread-count",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 1);

    // bob 标记 alice 的个人通知 → 403 (收件人不符)
    let (status, body) = send(
        &app,
        request_with_actor(
            "POST",
            &format!("/api/v1/notifications/{notif_id}/read"),
            Some(bob.clone()),
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    // 租户广播 (recipient NULL, 系统级生产者写入) 对双方可见
    let broadcast = Notification {
        id: Uuid::new_v4().to_string(),
        tenant_id: tenant,
        kind: NotificationKind::System,
        subject: "广播".to_string(),
        body: "全体注意".to_string(),
        subject_url: None,
        actor_id: None,
        actor_name: None,
        target_type: None,
        target_id: None,
        recipient_user_id: None,
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    store.insert_notification(broadcast, None).await.unwrap();
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/notifications", Some(bob.clone()), None),
    )
    .await;
    assert_eq!(
        body["data"]["total"], 1,
        "tenant broadcast visible to members"
    );
    assert_eq!(
        body["data"]["notifications"][0]["recipient_user_id"],
        Value::Null
    );

    // 状态对象可访问 (sanity)
    let _ = state.store.get_notification(&notif_id).await.unwrap();
}

/// 验收 4: 非成员读会话 → 403; 跨租户读会话 → 403; 列表无泄漏
#[tokio::test]
async fn non_member_and_cross_tenant_conversation_read_403() {
    let app = verified_im_app(Arc::new(InMemoryImStore::new()));
    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2001); // 同租户非成员
    let carol = actor(0x1003, 0x2002); // 跨租户
    let alice_str = uid(0x1001).to_string();
    let bob_str = uid(0x1002).to_string();

    let (status, body) = create_conversation(&app, &alice, &[&alice_str, &bob_str]).await;
    assert_eq!(status, StatusCode::OK);
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    // 同租户非成员读消息 → 403
    let (status, body) = send(
        &app,
        request_with_actor(
            "GET",
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            Some(actor(0x1009, 0x2001)),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    // 跨租户读消息 → 403 (即使伪造成员名)
    let (status, _) = send(
        &app,
        request_with_actor(
            "GET",
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            Some(carol.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 跨租户用户列表无泄漏
    let (status, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/im/conversations", Some(carol.clone()), None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["conversations"].as_array().unwrap().len(), 0);

    // bob (成员) 可读
    let (status, _) = send(
        &app,
        request_with_actor(
            "GET",
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            Some(bob.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// 验收 5: 创建 → 重建 state/仓储 → 未读与消息仍在 (持久化边界在 store,
/// 不在 handler; 真进程重启由 PG 版 #[ignore] 验收测试覆盖)
#[tokio::test]
async fn restart_rebuild_state_preserves_unread_and_messages() {
    let notify_store: Arc<dyn NotifyStore> = Arc::new(InMemoryNotifyStore::new());
    let im_store: Arc<dyn im::ImStore> = Arc::new(InMemoryImStore::new());
    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2001);
    let alice_str = uid(0x1001).to_string();
    let bob_str = uid(0x1002).to_string();

    // ── 第一次"进程" ──
    let (app1, _) = verified_notify_app(notify_store.clone());
    let im_app1 = verified_im_app(im_store.clone());

    let (_, body) = create_conversation(&im_app1, &alice, &[&alice_str, &bob_str]).await;
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    send_message(&im_app1, &alice, &conv_id, "你好").await;
    send_message(&im_app1, &alice, &conv_id, "在吗").await;

    let (_, body) = create_notification(
        &app1,
        &alice,
        json!({
            "tenant_id": uid(0x2001).to_string(),
            "kind": "system",
            "subject": "重启前通知",
            "body": "b",
            "recipient_user_id": alice_str,
        }),
    )
    .await;
    assert_eq!(body["data"]["notification"]["subject"], "重启前通知");

    // bob 未读 = 2 (alice 发的 2 条)
    let (_, body) = send(
        &im_app1,
        request_with_actor("GET", "/api/v1/im/conversations", Some(bob.clone()), None),
    )
    .await;
    assert_eq!(body["data"]["conversations"][0]["unread"], 2);
    drop(app1);
    drop(im_app1);

    // ── "重启": 重建 state/router (同一 store 即为持久层; PG 版换新 pool) ──
    let (app2, _) = verified_notify_app(notify_store.clone());
    let im_app2 = verified_im_app(im_store.clone());

    // 消息仍在 (sequence 1..2)
    let (status, body) = send(
        &im_app2,
        request_with_actor(
            "GET",
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            Some(bob.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let messages = body["data"]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2, "messages survive state rebuild");
    assert_eq!(messages[0]["content"]["text"], "你好");

    // 未读仍在
    let (_, body) = send(
        &im_app2,
        request_with_actor("GET", "/api/v1/im/conversations", Some(bob.clone()), None),
    )
    .await;
    assert_eq!(body["data"]["conversations"][0]["unread"], 2);

    // 通知未读仍在
    let (_, body) = send(
        &app2,
        request_with_actor(
            "GET",
            "/api/v1/notifications/unread-count",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(
        body["data"]["count"], 1,
        "unread count survives state rebuild"
    );
}

/// 验收 6: 重复投递不产生重复通知 (幂等键唯一约束, 三道防线)
#[tokio::test]
async fn duplicate_event_delivery_creates_single_notification() {
    let store: Arc<dyn NotifyStore> = Arc::new(InMemoryNotifyStore::new());
    let relay = NotifyRelay::new();
    let tenant = uid(0x2001).to_string();
    let recipient = uid(0x1001).to_string();

    let notification = Notification {
        id: Uuid::new_v4().to_string(),
        tenant_id: tenant.clone(),
        kind: NotificationKind::ReviewRequest,
        subject: "评审".to_string(),
        body: "请评审".to_string(),
        subject_url: None,
        actor_id: None,
        actor_name: None,
        target_type: None,
        target_id: None,
        recipient_user_id: Some(recipient.clone()),
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let event = OutboxEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "notification.created".to_string(),
        tenant_id: tenant.clone(),
        payload: serde_json::to_value(&notification).unwrap(),
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: 0,
        delivered: false,
        dead: false,
    };

    // 防线 1: 同一事件入队两次 → 只保留一条 outbox 行
    store.enqueue_outbox(event.clone()).await.unwrap();
    store.enqueue_outbox(event.clone()).await.unwrap();

    // 防线 2: relay 投递 → 通知落地一条
    let outcomes = relay.drain(&store, i64::MAX).await.unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0],
        notifications::RelayOutcome::Applied { .. }
    ));
    assert_eq!(
        store
            .list_notifications(Some(&tenant), Some(&recipient), None, false)
            .await
            .unwrap()
            .len(),
        1,
        "exactly one notification after delivery"
    );

    // 防线 3: apply 直接重放同一事件 → Duplicate, 不产生第二份
    let outcome = relay.apply_event(&store, &event).await.unwrap();
    assert_eq!(outcome, notifications::InsertOutcome::Duplicate);
    assert_eq!(
        store
            .list_notifications(Some(&tenant), Some(&recipient), None, false)
            .await
            .unwrap()
            .len(),
        1,
        "replay must not duplicate the notification"
    );

    // 已投递事件不再出现在 due 列表 (再次 drain 无事发生)
    let again = relay.drain(&store, i64::MAX).await.unwrap();
    assert!(again.is_empty());
}

/// 验收 7: unread-count 与列表一致; mark-read 幂等
#[tokio::test]
async fn unread_count_matches_list_and_mark_read_idempotent() {
    let (app, _) = verified_notify_app(Arc::new(InMemoryNotifyStore::new()));
    let alice = actor(0x1001, 0x2001);
    let tenant = uid(0x2001).to_string();
    let alice_id = uid(0x1001).to_string();

    for i in 0..2 {
        let (status, _) = create_notification(
            &app,
            &alice,
            json!({
                "tenant_id": tenant,
                "kind": "comment",
                "subject": format!("n{i}"),
                "body": "b",
                "recipient_user_id": alice_id,
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // 服务端单一真源: unread-count == unread_only 列表 total
    let (_, body) = send(
        &app,
        request_with_actor(
            "GET",
            "/api/v1/notifications/unread-count",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 2);
    let (_, body) = send(
        &app,
        request_with_actor(
            "GET",
            "/api/v1/notifications?unread_only=true",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(
        body["data"]["total"], 2,
        "list unread total matches unread-count"
    );

    // 取第一条标记已读 (幂等: 两次都 200)
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/notifications", Some(alice.clone()), None),
    )
    .await;
    let first_id = body["data"]["notifications"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    for _ in 0..2 {
        let (status, body) = send(
            &app,
            request_with_actor(
                "POST",
                &format!("/api/v1/notifications/{first_id}/read"),
                Some(alice.clone()),
                Some(json!({})),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "mark-read is idempotent");
        assert_eq!(body["data"]["notification"]["read"], true);
    }

    // 计数与列表同步减一
    let (_, body) = send(
        &app,
        request_with_actor(
            "GET",
            "/api/v1/notifications/unread-count",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 1);
    let (_, body) = send(
        &app,
        request_with_actor(
            "GET",
            "/api/v1/notifications?unread_only=true",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);
}

/// 验收 8: 生产 router() 拒绝未登录 (无 token → 401; 无效 token → 401)
#[tokio::test]
async fn production_router_requires_valid_bearer_token() {
    // 生产 router(): OnceLock 共享 state + JWT 验签 middleware (fail-closed)
    let prod = Router::new()
        .nest("/api/v1/notifications", notifications::router())
        .nest("/api/v1/im", im::router());

    // 无 Authorization 头 → 401
    let (status, _) = send(
        &prod,
        request_with_actor("GET", "/api/v1/notifications", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 无效 Bearer token → 401 (测试环境无 JWT 公钥 → deny-all, 全部拒绝)
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/im/conversations")
        .header("authorization", "Bearer definitely-not-a-valid-token")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(&prod, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 注入 extension 也不行 (生产 middleware 会先验签; 无 token 即拒绝)
    let (status, _) = send(
        &prod,
        request_with_actor(
            "GET",
            "/api/v1/notifications",
            Some(actor(0x1001, 0x2001)),
            None,
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "production router requires Bearer token"
    );
}

/// 验收 9: IM 网关诚实标注未接入 — push 恒 NotConnected, 不假发送
#[tokio::test]
async fn im_gateway_reports_not_connected() {
    let gateway = UnconnectedImGateway;
    let msg = Message {
        id: Uuid::new_v4().to_string(),
        conversation_id: "c".to_string(),
        sequence: 1,
        sender_id: "u".to_string(),
        kind: "text".to_string(),
        content: json!({ "text": "hi" }),
        reply_to: None,
        state: im::MessageState::Sent,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let result = gateway.push_message(&msg).await;
    assert!(matches!(result, Err(ImGatewayError::NotConnected(_))));
}

/// 验收 10: verified DM 幂等 (同租户同对用户 → 既有会话; 跨租户不碰撞)
#[tokio::test]
async fn verified_dm_creation_is_idempotent_per_tenant() {
    let app = verified_im_app(Arc::new(InMemoryImStore::new()));
    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2001);
    let alice_str = uid(0x1001).to_string();
    let bob_str = uid(0x1002).to_string();

    let (_, body1) = create_conversation(&app, &alice, &[&alice_str, &bob_str]).await;
    let id1 = body1["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(body1["data"]["tenant_id"], uid(0x2001).to_string());

    // 同对反向发起 → 同一会话
    let (_, body2) = create_conversation(&app, &bob, &[&bob_str, &alice_str]).await;
    assert_eq!(
        body2["data"]["conversation_id"].as_str().unwrap(),
        id1.as_str(),
        "same pair in same tenant reuses the DM"
    );

    // 会话总数仍为 1
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/im/conversations", Some(alice.clone()), None),
    )
    .await;
    assert_eq!(body["data"]["conversations"].as_array().unwrap().len(), 1);
}

/// 投递失败注入 store: insert_notification 恒失败 (其余委托内存实现)
struct FailingNotifyStore {
    inner: InMemoryNotifyStore,
}

#[async_trait]
impl NotifyStore for FailingNotifyStore {
    async fn insert_notification(
        &self,
        _n: Notification,
        _idempotency_key: Option<&str>,
    ) -> Result<notifications::InsertOutcome, NotifyStoreError> {
        Err(NotifyStoreError::Unavailable(
            "injected failure".to_string(),
        ))
    }
    async fn get_notification(&self, id: &str) -> Result<Option<Notification>, NotifyStoreError> {
        self.inner.get_notification(id).await
    }
    async fn set_notification_read(
        &self,
        id: &str,
        read: bool,
    ) -> Result<Option<Notification>, NotifyStoreError> {
        self.inner.set_notification_read(id, read).await
    }
    async fn list_notifications(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
        kind: Option<NotificationKind>,
        unread_only: bool,
    ) -> Result<Vec<Notification>, NotifyStoreError> {
        self.inner
            .list_notifications(tenant_id, recipient_user_id, kind, unread_only)
            .await
    }
    async fn unread_count(
        &self,
        tenant_id: Option<&str>,
        recipient_user_id: Option<&str>,
    ) -> Result<i64, NotifyStoreError> {
        self.inner.unread_count(tenant_id, recipient_user_id).await
    }
    async fn insert_comment(
        &self,
        c: notifications::Comment,
        idempotency_key: Option<&str>,
    ) -> Result<notifications::InsertOutcome, NotifyStoreError> {
        self.inner.insert_comment(c, idempotency_key).await
    }
    async fn get_comment(
        &self,
        id: &str,
    ) -> Result<Option<notifications::Comment>, NotifyStoreError> {
        self.inner.get_comment(id).await
    }
    async fn remove_comment(&self, id: &str) -> Result<bool, NotifyStoreError> {
        self.inner.remove_comment(id).await
    }
    async fn list_comments(
        &self,
        tenant_id: Option<&str>,
        target_type: Option<&str>,
        target_id: Option<&str>,
    ) -> Result<Vec<notifications::Comment>, NotifyStoreError> {
        self.inner
            .list_comments(tenant_id, target_type, target_id)
            .await
    }
    async fn enqueue_outbox(&self, ev: OutboxEvent) -> Result<(), NotifyStoreError> {
        self.inner.enqueue_outbox(ev).await
    }
    async fn list_due_outbox(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> Result<Vec<OutboxEvent>, NotifyStoreError> {
        self.inner.list_due_outbox(now_ms, limit).await
    }
    async fn mark_outbox_delivered(&self, event_id: &str) -> Result<(), NotifyStoreError> {
        self.inner.mark_outbox_delivered(event_id).await
    }
    async fn mark_outbox_attempt(
        &self,
        event_id: &str,
        attempts: u32,
        next_attempt_at_ms: i64,
        dead: bool,
    ) -> Result<(), NotifyStoreError> {
        self.inner
            .mark_outbox_attempt(event_id, attempts, next_attempt_at_ms, dead)
            .await
    }
}

/// 验收 11: 重试有上限 (OUTBOX_MAX_RETRY=5) + 有界退避 (RelayBackoffPolicy 复用)
#[tokio::test]
async fn outbox_retry_backoff_caps_and_dead_letters() {
    let store: Arc<dyn NotifyStore> = Arc::new(FailingNotifyStore {
        inner: InMemoryNotifyStore::new(),
    });
    let relay = NotifyRelay::new();

    let notification = Notification {
        id: Uuid::new_v4().to_string(),
        tenant_id: uid(0x2001).to_string(),
        kind: NotificationKind::System,
        subject: "s".to_string(),
        body: "b".to_string(),
        subject_url: None,
        actor_id: None,
        actor_name: None,
        target_type: None,
        target_id: None,
        recipient_user_id: None,
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let event = OutboxEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "notification.created".to_string(),
        tenant_id: notification.tenant_id.clone(),
        payload: serde_json::to_value(&notification).unwrap(),
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: 0,
        delivered: false,
        dead: false,
    };
    store.enqueue_outbox(event.clone()).await.unwrap();

    // 第 1 次 drain: 瞬时失败 → Retried(1), 下次可投递时间被退避推进
    let outcomes = relay.drain(&store, 0).await.unwrap();
    assert!(
        matches!(&outcomes[0], notifications::RelayOutcome::Retried { attempts, .. } if *attempts == 1)
    );
    let due_at_0 = store.list_due_outbox(0, 10).await.unwrap();
    assert!(due_at_0.is_empty(), "backoff defers the next attempt");

    // 退避到期后持续失败 → 到第 5 次超限 → DeadLettered
    let mut deadline: i64;
    let mut last: Option<notifications::RelayOutcome> = None;
    for i in 2..=10 {
        deadline = 10_000_000 * i64::from(i); // 远超 max backoff 30s
        let outcomes = relay.drain(&store, deadline).await.unwrap();
        if !outcomes.is_empty() {
            last = Some(outcomes[0].clone());
        }
        if matches!(last, Some(notifications::RelayOutcome::DeadLettered { .. })) {
            break;
        }
    }
    assert!(
        matches!(last, Some(notifications::RelayOutcome::DeadLettered { .. })),
        "exhausted retries must dead-letter, got {last:?}"
    );
    // 死信后不再重试 (due 列表为空, 即使时间无限推进)
    let due = store.list_due_outbox(i64::MAX, 10).await.unwrap();
    assert!(due.is_empty(), "dead-lettered events must not be retried");
}

// =====================================================================
// PG 验收 (真重启/幂等约束; #[ignore] 需要真实 PostgreSQL)
// =====================================================================

/// 在隔离 PG 库建 schema (每行一条语句, 与仓库 migration runner 同纪律)
async fn pg_init_schema(pool: &sqlx::PgPool) {
    for ddl in [notifications::NOTIFY_PG_DDL, im::IM_PG_DDL] {
        for line in ddl.lines() {
            let statement = line.trim();
            if !statement.is_empty() && !statement.starts_with("--") {
                sqlx::query(statement)
                    .execute(pool)
                    .await
                    .expect("p0 schema DDL must apply");
            }
        }
    }
}

/// 清空 p0 表 (隔离测试重复运行)
async fn pg_truncate(pool: &sqlx::PgPool) {
    for table in [
        "p0_notifications",
        "p0_comments",
        "p0_notify_outbox",
        "p0_im_messages",
        "p0_im_members",
        "p0_im_dm_pairs",
        "p0_im_sequences",
        "p0_im_conversations",
    ] {
        sqlx::query(&format!("TRUNCATE TABLE {table}"))
            .execute(pool)
            .await
            .expect("truncate");
    }
}

/// PG 验收: 真进程重启语义 — 重建 pool/store 后通知未读与 IM 消息仍在;
/// 幂等唯一约束在数据库层生效。
///
/// 跑法: STAR_P0_NOTIFY_TEST_DATABASE_URL=<隔离 PG URL> cargo test -p star-api-rest
/// --test notify_im_endpoints pg_ -- --ignored
#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_P0_NOTIFY_TEST_DATABASE_URL"]
async fn pg_restart_preserves_notifications_and_im() {
    let database_url = std::env::var("STAR_P0_NOTIFY_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("connect");
    pg_init_schema(&pool).await;
    pg_truncate(&pool).await;

    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2001);
    let alice_str = uid(0x1001).to_string();
    let bob_str = uid(0x1002).to_string();

    // ── 进程 1: PG store ──
    let (app1, _) = verified_notify_app(Arc::new(notifications::PgNotifyStore::new(pool.clone())));
    let im_app1 = verified_im_app(Arc::new(im::PgImStore::new(pool.clone())));

    let (_, body) = create_conversation(&im_app1, &alice, &[&alice_str, &bob_str]).await;
    let conv_id = body["data"]["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    send_message(&im_app1, &alice, &conv_id, "持久化消息").await;

    let (_, body) = create_notification(
        &app1,
        &alice,
        json!({
            "tenant_id": uid(0x2001).to_string(),
            "kind": "system",
            "subject": "PG 重启前",
            "body": "b",
            "recipient_user_id": alice_str,
        }),
    )
    .await;
    assert_eq!(body["data"]["notification"]["subject"], "PG 重启前");
    drop(app1);
    drop(im_app1);
    drop(pool);

    // ── "重启": 全新 pool + 全新 store ──
    let pool2 = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("reconnect");
    let (app2, _) = verified_notify_app(Arc::new(notifications::PgNotifyStore::new(pool2.clone())));
    let im_app2 = verified_im_app(Arc::new(im::PgImStore::new(pool2.clone())));

    // 消息跨进程恢复
    let (status, body) = send(
        &im_app2,
        request_with_actor(
            "GET",
            &format!("/api/v1/im/conversations/{conv_id}/messages"),
            Some(bob.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["messages"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"]["messages"][0]["content"]["text"], "持久化消息");

    // 未读跨进程恢复
    let (_, body) = send(
        &im_app2,
        request_with_actor("GET", "/api/v1/im/conversations", Some(bob.clone()), None),
    )
    .await;
    assert_eq!(body["data"]["conversations"][0]["unread"], 1);

    // 通知未读跨进程恢复
    let (_, body) = send(
        &app2,
        request_with_actor(
            "GET",
            "/api/v1/notifications/unread-count",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["count"], 1);

    // 幂等唯一约束在 DB 层: 同 idempotency_key 重复插入 → Duplicate
    let store2 = notifications::PgNotifyStore::new(pool2.clone());
    let n = Notification {
        id: Uuid::new_v4().to_string(),
        tenant_id: uid(0x2001).to_string(),
        kind: NotificationKind::Mention,
        subject: "idem".to_string(),
        body: "b".to_string(),
        subject_url: None,
        actor_id: None,
        actor_name: None,
        target_type: None,
        target_id: None,
        recipient_user_id: Some(alice_str.clone()),
        read: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let key = "pg-test-idem-key-1";
    assert_eq!(
        store2
            .insert_notification(n.clone(), Some(key))
            .await
            .unwrap(),
        notifications::InsertOutcome::Inserted
    );
    assert_eq!(
        store2.insert_notification(n, Some(key)).await.unwrap(),
        notifications::InsertOutcome::Duplicate,
        "PG unique idempotency constraint must dedupe"
    );
    assert_eq!(
        store2
            .list_notifications(
                Some(&uid(0x2001).to_string()),
                Some(&alice_str),
                None,
                false
            )
            .await
            .unwrap()
            .iter()
            .filter(|x| x.subject == "idem")
            .count(),
        1
    );
}
