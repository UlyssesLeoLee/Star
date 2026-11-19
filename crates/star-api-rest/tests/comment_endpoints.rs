/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/comment_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::p0 comment suite",kind:"test_suite"})
MERGE (auth401:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::unauthenticated_comment_routes_401",kind:"test"})
MERGE (tenant403:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::comment_tenant_mismatch_body_403",kind:"test"})
MERGE (del403:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::delete_others_comment_403",kind:"test"})
MERGE (scope:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::comment_list_is_tenant_scoped",kind:"test"})
MERGE (fanout:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::comment_event_fanout_apply_is_idempotent",kind:"test"})
MERGE (poison:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::unknown_outbox_event_type_dead_letters",kind:"test"})
MERGE (restart:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::restart_preserves_comments",kind:"test"})
MERGE (pg:Symbol {id:"crates/star-api-rest/tests/comment_endpoints.rs::pg_duplicate_comment_delivery_creates_single_comment",kind:"test"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
//! P0 评论生产化验收测试 (2026-11-20, per TASK-p0-notify.md §4)
//!
//! 守门 #25 v25: 全部走 axum::Router::oneshot; verified actor 经 request
//! extension 注入 `AuthUser` (等价生产 middleware 验签后注入)。
//!
//! 验收映射:
//! - 未登录 → 401; 跨租户 body → 403; 删他人评论 → 403
//! - 评论列表租户隔离; 创建 → 重建 state → 评论仍在
//! - comment.created 事件幂等 apply + notify_recipients 通知扇出不重复
//! - 未知事件类型 → 死信 (fail-closed, 不静默跳过)

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use serde_json::{json, Value};
use star_api_rest::auth::AuthUser;
use star_api_rest::routes::notifications::{
    self, InMemoryNotifyStore, NotificationKind, NotificationsApiState, NotifyRelay, NotifyStore,
    OutboxEvent,
};
use tower::ServiceExt;
use uuid::Uuid;

/// 固定 user id
fn uid(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

/// 构造已验证 actor
fn actor(user: u128, tenant: u128) -> AuthUser {
    AuthUser {
        user_id: uid(user),
        tenant_id: uid(tenant),
        roles: vec!["user".to_string()],
        scope: "notifications:read notifications:write".to_string(),
        email: None,
        projects: Vec::new(),
    }
}

/// 带已验证 actor extension 的请求
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

/// verified 评论 app (store 注入, 重启测试共享 store)
fn comments_app(store: Arc<dyn NotifyStore>) -> Router {
    let state = Arc::new(NotificationsApiState::from_store(store));
    Router::new().nest(
        "/api/v1/comments",
        notifications::comments_router_with_verified_state(state),
    )
}

/// 创建 verified 评论 → (status, body)
async fn create_comment(app: &Router, actor: &AuthUser, payload: Value) -> (StatusCode, Value) {
    send(
        app,
        request_with_actor(
            "POST",
            "/api/v1/comments",
            Some(actor.clone()),
            Some(payload),
        ),
    )
    .await
}

/// 验收 1: 评论路由未登录 → 401 (GET/POST/DELETE 三端点)
#[tokio::test]
async fn unauthenticated_comment_routes_401() {
    let app = comments_app(Arc::new(InMemoryNotifyStore::new()));

    let (status, _) = send(
        &app,
        request_with_actor("GET", "/api/v1/comments", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = send(
        &app,
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

    let (status, _) = send(
        &app,
        request_with_actor(
            "DELETE",
            &format!("/api/v1/comments/{}", Uuid::new_v4()),
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// 验收 2: 跨租户评论创建 → 403 POLICY_DENIED
#[tokio::test]
async fn comment_tenant_mismatch_body_403() {
    let app = comments_app(Arc::new(InMemoryNotifyStore::new()));
    let alice = actor(0x1001, 0x2001);

    let (status, body) = create_comment(
        &app,
        &alice,
        json!({
            "tenant_id": uid(0x2002).to_string(),
            "target_type": "work_item",
            "target_id": "wi-1",
            "body": "越权评论",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");
}

/// 验收 3: 删他人评论 → 403; 作者可删 (真实身份判定)
#[tokio::test]
async fn delete_others_comment_403() {
    let app = comments_app(Arc::new(InMemoryNotifyStore::new()));
    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2001);
    let tenant = uid(0x2001).to_string();

    let (status, body) = create_comment(
        &app,
        &alice,
        json!({
            "tenant_id": tenant,
            "target_type": "merge_request",
            "target_id": "mr-1",
            "body": "alice 的评论",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let comment = &body["data"]["comment"];
    assert_eq!(
        comment["author_id"],
        uid(0x1001).to_string(),
        "author is the verified actor"
    );
    let id = comment["id"].as_str().unwrap().to_string();

    // bob 删 alice 的评论 → 403
    let (status, body) = send(
        &app,
        request_with_actor(
            "DELETE",
            &format!("/api/v1/comments/{id}"),
            Some(bob.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    // 评论仍在
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/comments", Some(alice.clone()), None),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);

    // alice 自删 → 200
    let (status, _) = send(
        &app,
        request_with_actor(
            "DELETE",
            &format!("/api/v1/comments/{id}"),
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// 验收 4: 评论列表租户隔离 (跨租户不可见)
#[tokio::test]
async fn comment_list_is_tenant_scoped() {
    let app = comments_app(Arc::new(InMemoryNotifyStore::new()));
    let alice = actor(0x1001, 0x2001);
    let bob = actor(0x1002, 0x2002); // 另一租户
    let t1 = uid(0x2001).to_string();
    let t2 = uid(0x2002).to_string();

    let (status, _) = create_comment(
        &app,
        &alice,
        json!({ "tenant_id": t1, "target_type": "work_item", "target_id": "wi-1", "body": "t1 评论" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // alice (t1) 可见
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/comments", Some(alice.clone()), None),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);

    // bob (t2) 不可见 — 且 bob 无法创建 t1 评论
    let (status, _) = create_comment(
        &app,
        &bob,
        json!({ "tenant_id": t1, "target_type": "work_item", "target_id": "wi-1", "body": "偷渡" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (_, body) = send(
        &app,
        request_with_actor("GET", "/api/v1/comments", Some(bob.clone()), None),
    )
    .await;
    assert_eq!(
        body["data"]["total"], 0,
        "cross-tenant comments must not leak"
    );
    // t2 参数只用于显式声明 (避免未使用告警)
    assert_eq!(t2, uid(0x2002).to_string());
}

/// 验收 5: comment.created 事件幂等 apply + 通知扇出 (重复投递不重复)
#[tokio::test]
async fn comment_event_fanout_apply_is_idempotent() {
    let store: Arc<dyn NotifyStore> = Arc::new(InMemoryNotifyStore::new());
    let relay = NotifyRelay::new();
    let tenant = uid(0x2001).to_string();

    let comment = notifications::Comment {
        id: Uuid::new_v4().to_string(),
        tenant_id: tenant.clone(),
        target_type: "work_item".to_string(),
        target_id: "wi-1".to_string(),
        author_id: uid(0x1001).to_string(),
        author_name: "alice".to_string(),
        body: "LGTM".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    // 联动: 评论触发 2 个收件人的 Comment 通知
    let mut payload = serde_json::to_value(&comment).unwrap();
    payload["notify_recipients"] = json!([uid(0x2005).to_string(), uid(0x2006).to_string()]);
    let event = OutboxEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "comment.created".to_string(),
        tenant_id: tenant.clone(),
        payload,
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: 0,
        delivered: false,
        dead: false,
    };

    // 同一事件入队两次 (重复投递模拟) → outbox 只留一行
    store.enqueue_outbox(event.clone()).await.unwrap();
    store.enqueue_outbox(event.clone()).await.unwrap();

    // 投递: 评论 1 条 + 每个收件人 1 条通知
    let outcomes = relay.drain(&store, i64::MAX).await.unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0],
        notifications::RelayOutcome::Applied { .. }
    ));
    assert_eq!(
        store
            .list_comments(Some(&tenant), None, None)
            .await
            .unwrap()
            .len(),
        1
    );
    for recipient in [uid(0x2005), uid(0x2006)] {
        let list = store
            .list_notifications(Some(&tenant), Some(&recipient.to_string()), None, false)
            .await
            .unwrap();
        assert_eq!(
            list.len(),
            1,
            "each recipient gets exactly one comment notification"
        );
        assert_eq!(list[0].kind, NotificationKind::Comment);
    }

    // 直接重放 apply → 评论与通知都不增
    relay.apply_event(&store, &event).await.unwrap();
    assert_eq!(
        store
            .list_comments(Some(&tenant), None, None)
            .await
            .unwrap()
            .len(),
        1
    );
    for recipient in [uid(0x2005), uid(0x2006)] {
        assert_eq!(
            store
                .list_notifications(Some(&tenant), Some(&recipient.to_string()), None, false)
                .await
                .unwrap()
                .len(),
            1,
            "fan-out replay must not duplicate notifications"
        );
    }
}

/// 验收 6: 未知 outbox 事件类型 → 死信 (fail-closed, 不静默跳过)
#[tokio::test]
async fn unknown_outbox_event_type_dead_letters() {
    let store: Arc<dyn NotifyStore> = Arc::new(InMemoryNotifyStore::new());
    let relay = NotifyRelay::new();
    let event = OutboxEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "comment.exploded".to_string(),
        tenant_id: uid(0x2001).to_string(),
        payload: json!({}),
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: 0,
        delivered: false,
        dead: false,
    };
    store.enqueue_outbox(event.clone()).await.unwrap();
    let outcomes = relay.drain(&store, i64::MAX).await.unwrap();
    assert!(matches!(
        outcomes[0],
        notifications::RelayOutcome::DeadLettered { .. }
    ));
    // 死信后不再重试
    assert!(store
        .list_due_outbox(i64::MAX, 10)
        .await
        .unwrap()
        .is_empty());
}

/// 验收 7: 创建 → 重建 state/仓储 → 评论仍在
#[tokio::test]
async fn restart_preserves_comments() {
    let store: Arc<dyn NotifyStore> = Arc::new(InMemoryNotifyStore::new());
    let alice = actor(0x1001, 0x2001);
    let tenant = uid(0x2001).to_string();

    let app1 = comments_app(store.clone());
    let (status, body) = create_comment(
        &app1,
        &alice,
        json!({ "tenant_id": tenant, "target_type": "work_item", "target_id": "wi-1", "body": "重启前评论" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = body["data"]["comment"]["id"].as_str().unwrap().to_string();
    drop(app1);

    // 重建 state/router (同一 store = 持久层)
    let app2 = comments_app(store.clone());
    let (_, body) = send(
        &app2,
        request_with_actor(
            "GET",
            "/api/v1/comments?target_type=work_item&target_id=wi-1",
            Some(alice.clone()),
            None,
        ),
    )
    .await;
    assert_eq!(body["data"]["total"], 1);
    assert_eq!(body["data"]["comments"][0]["id"], id.as_str());
    assert_eq!(body["data"]["comments"][0]["body"], "重启前评论");
}

/// PG 验收: 数据库层幂等唯一约束 — 同 idempotency_key 重复投递评论 → 一条。
///
/// 跑法: STAR_P0_NOTIFY_TEST_DATABASE_URL=<隔离 PG URL> cargo test -p star-api-rest
/// --test comment_endpoints pg_ -- --ignored
#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_P0_NOTIFY_TEST_DATABASE_URL"]
async fn pg_duplicate_comment_delivery_creates_single_comment() {
    let database_url = std::env::var("STAR_P0_NOTIFY_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("connect");
    for line in notifications::NOTIFY_PG_DDL.lines() {
        let statement = line.trim();
        if !statement.is_empty() && !statement.starts_with("--") {
            sqlx::query(statement).execute(&pool).await.expect("DDL");
        }
    }
    for table in ["p0_notifications", "p0_comments", "p0_notify_outbox"] {
        sqlx::query(&format!("TRUNCATE TABLE {table}"))
            .execute(&pool)
            .await
            .expect("truncate");
    }

    let store = notifications::PgNotifyStore::new(pool.clone());
    let relay = NotifyRelay::new();
    let tenant = uid(0x2001).to_string();

    let comment = notifications::Comment {
        id: Uuid::new_v4().to_string(),
        tenant_id: tenant.clone(),
        target_type: "work_item".to_string(),
        target_id: "wi-1".to_string(),
        author_id: uid(0x1001).to_string(),
        author_name: "alice".to_string(),
        body: "PG 幂等评论".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut payload = serde_json::to_value(&comment).unwrap();
    payload["notify_recipients"] = json!([uid(0x2005).to_string()]);
    let event = OutboxEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "comment.created".to_string(),
        tenant_id: tenant.clone(),
        payload,
        created_at: chrono::Utc::now().to_rfc3339(),
        attempts: 0,
        next_attempt_at_ms: 0,
        delivered: false,
        dead: false,
    };

    // 双份入队 + 双轮 drain → 评论与通知仍各一条 (DB 唯一约束生效)
    store.enqueue_outbox(event.clone()).await.unwrap();
    store.enqueue_outbox(event.clone()).await.unwrap();
    relay
        .drain(&(Arc::new(store) as Arc<dyn NotifyStore>), i64::MAX)
        .await
        .unwrap();
    let store = notifications::PgNotifyStore::new(pool.clone());
    relay
        .drain(&(Arc::new(store) as Arc<dyn NotifyStore>), i64::MAX)
        .await
        .unwrap();
    let store = notifications::PgNotifyStore::new(pool.clone());
    assert_eq!(
        store
            .list_comments(Some(&tenant), None, None)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .list_notifications(Some(&tenant), Some(&uid(0x2005).to_string()), None, false)
            .await
            .unwrap()
            .len(),
        1
    );
}
