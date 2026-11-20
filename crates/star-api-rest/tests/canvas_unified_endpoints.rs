// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/canvas_unified_endpoints.rs"})
MERGE (readOnlyScope:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::read_only_scope_is_rejected_on_all_write_verbs",kind:"test"})
MERGE (missingWorktreeScope:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::canvas_write_scope_without_worktree_read_is_rejected",kind:"test"})
MERGE (legacyToken:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::legacy_token_without_device_context_cannot_read_canvas",kind:"test"})
MERGE (malformedId:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::malformed_worktree_id_is_rejected_before_authorization",kind:"test"})
MERGE (twoEntries:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::two_entries_share_same_canvas_snapshot",kind:"test"})
MERGE (readOnlyMember:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::read_only_member_write_verbs_forbidden",kind:"test"})
MERGE (idempotent:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::idempotent_create_element_does_not_duplicate",kind:"test"})
MERGE (conflict:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::stale_version_conflict_returns_409_with_both_sides",kind:"test"})
MERGE (documentConflict:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::document_version_conflict_returns_409_with_both_sides",kind:"test"})
MERGE (cursor:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::cursor_catch_up_loses_nothing_duplicates_nothing",kind:"test"})
MERGE (seed:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::seed_canvas_unified_fixture",kind:"function"})
MERGE (send:Symbol {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::send",kind:"function"})
MERGE (resolveEndpoint:Endpoint {id:"GET /api/v1/worktrees/{worktree_id}/canvas",kind:"http_route"})
MERGE (eventsEndpoint:Endpoint {id:"GET /api/v1/worktrees/{worktree_id}/canvas-events",kind:"http_route"})
MERGE (db:Config {id:"STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (fixture:Type {id:"crates/star-api-rest/tests/canvas_unified_endpoints.rs::CanvasUnifiedFixture",kind:"struct"})
MERGE (self)-[:DEFINES]->(readOnlyScope)
MERGE (self)-[:DEFINES]->(missingWorktreeScope)
MERGE (self)-[:DEFINES]->(legacyToken)
MERGE (self)-[:DEFINES]->(malformedId)
MERGE (self)-[:DEFINES]->(twoEntries)
MERGE (self)-[:DEFINES]->(readOnlyMember)
MERGE (self)-[:DEFINES]->(idempotent)
MERGE (self)-[:DEFINES]->(conflict)
MERGE (self)-[:DEFINES]->(documentConflict)
MERGE (self)-[:DEFINES]->(cursor)
MERGE (self)-[:DEFINES]->(seed)
MERGE (self)-[:DEFINES]->(send)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (twoEntries)-[:TESTS]->(resolveEndpoint)
MERGE (cursor)-[:TESTS]->(eventsEndpoint)
MERGE (twoEntries)-[:READS]->(db)
MERGE (readOnlyMember)-[:READS]->(db)
MERGE (idempotent)-[:READS]->(db)
MERGE (conflict)-[:READS]->(db)
MERGE (documentConflict)-[:READS]->(db)
MERGE (cursor)-[:READS]->(db)
MERGE (seed)-[:USES_TYPE]->(fixture)
@endcypher
*/
//! P1 Canvas 数据源合并集成测试 (per TASK-p1-canvas-data):
//! 独立 /canvas/[id] 入口与 Group Canvas 共用同一授权真源。
//!
//! 覆盖 (验收三条):
//! - 同一 Worktree 从两个入口 (两设备会话) GET 同一画布 → 快照一致
//! - 只读成员 (role=member) POST/PUT/DELETE → 403, GET /canvas 的 can_write=false
//! - 幂等键重复写 → 不产生重复 entry
//! - 版本冲突 → 409 + current/submitted 双方内容 (草稿不静默覆盖)
//! - 断线恢复: 按 after_offset cursor catch-up → 事件不丢不重
//!
//! 前 4 条 fail-closed 用例不依赖数据库 (在触碰 PgPool 前拒绝);
//! 其余 6 条走 axum::Router::oneshot + 隔离 PostgreSQL
//! (STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL, 沿用 canvas_device_auth_pg 约定, #[ignore])。

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use star_api_rest::{
    auth::{issue_token, issue_token_with_device_context, DeviceTokenContext, JwtConfig},
    group_api::{build_group_router, GroupApiState},
};
use tower::ServiceExt;
use uuid::Uuid;

const PG_TEST_DATABASE_URL: &str = "STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL";

fn route_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

/// 隔离 app: 生产形态 build_group_router + GroupApiState (测试专属 JWT/Pool)。
fn unified_app(jwt: Arc<JwtConfig>, pool: PgPool) -> Router {
    build_group_router(GroupApiState::new(jwt, pool))
}

/// 惰性连接 pool: 任何真正触库的用例都会连接失败 —
/// 保证「fail-closed 拒绝先于数据库访问」的断言有效。
fn lazy_pool(database: &str) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(PgConnectOptions::new().database(database))
}

/// oneshot 发送 + 解析 JSON body (空 body 返回 json!({}))
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.expect("oneshot");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("bounded body");
    let body = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).expect("json body")
    };
    (status, body)
}

fn device_token(
    jwt: &JwtConfig,
    actor_id: Uuid,
    tenant_id: Uuid,
    scope: &str,
    device_id: Uuid,
    generation: i64,
) -> String {
    issue_token_with_device_context(
        jwt,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        scope.to_owned(),
        Some(DeviceTokenContext {
            device_id,
            generation,
        }),
    )
    .expect("test-only RSA key should issue a device-bound token")
    .0
}

fn post_json(path: &str, token: &str, key: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("idempotency-key", key)
        .body(Body::from(payload.to_string()))
        .expect("request")
}

fn put_json(path: &str, token: &str, key: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("idempotency-key", key)
        .body(Body::from(payload.to_string()))
        .expect("request")
}

fn delete_json(path: &str, token: &str, key: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("idempotency-key", key)
        .body(Body::from(payload.to_string()))
        .expect("request")
}

fn get_uri(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request")
}

// =====================================================================
// 不依赖数据库的 fail-closed oneshot (真跑, 验证授权先于任何 DB 访问)
// =====================================================================

#[tokio::test]
async fn read_only_scope_is_rejected_on_all_write_verbs() {
    let jwt = route_test_jwt_config();
    let actor_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let token = device_token(
        &jwt,
        actor_id,
        tenant_id,
        "worktree:read canvas:read",
        device_id,
        1,
    );
    let app = unified_app(jwt, lazy_pool("canvas_unified_readonly_test"));
    let worktree = Uuid::new_v4();
    let canvas = Uuid::new_v4();
    let element = Uuid::new_v4();

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/worktrees/{worktree}/canvases"),
            &token,
            &Uuid::new_v4().to_string(),
            json!({"title": "readonly member must not create"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "insufficient_scope");

    let (status, _) = send(
        &app,
        put_json(
            &format!("/api/v1/worktrees/{worktree}/canvases/{canvas}/elements/{element}"),
            &token,
            &Uuid::new_v4().to_string(),
            json!({"expected_version": 1, "update_mode": "position", "x": 10.0, "y": 10.0}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send(
        &app,
        delete_json(
            &format!("/api/v1/worktrees/{worktree}/canvases/{canvas}/elements/{element}"),
            &token,
            &Uuid::new_v4().to_string(),
            json!({"expected_version": 1}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn canvas_write_scope_without_worktree_read_is_rejected() {
    let jwt = route_test_jwt_config();
    let actor_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let token = device_token(&jwt, actor_id, tenant_id, "canvas:write", device_id, 1);
    let app = unified_app(jwt, lazy_pool("canvas_unified_scope_test"));

    let (status, body) = send(
        &app,
        post_json(
            &format!("/api/v1/worktrees/{}/canvases", Uuid::new_v4()),
            &token,
            &Uuid::new_v4().to_string(),
            json!({"title": "missing worktree:read"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "insufficient_scope");
}

#[tokio::test]
async fn legacy_token_without_device_context_cannot_read_canvas() {
    let jwt = route_test_jwt_config();
    let legacy = issue_token(
        &jwt,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        "worktree:read canvas:read".to_owned(),
    )
    .expect("legacy token should issue")
    .0;
    let app = unified_app(jwt, lazy_pool("canvas_unified_legacy_test"));

    let (status, _) = send(
        &app,
        get_uri(
            &format!("/api/v1/worktrees/{}/canvas", Uuid::new_v4()),
            &legacy,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn malformed_worktree_id_is_rejected_before_authorization() {
    let jwt = route_test_jwt_config();
    let actor_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let token = device_token(
        &jwt,
        actor_id,
        tenant_id,
        "worktree:read canvas:read",
        device_id,
        1,
    );
    let app = unified_app(jwt, lazy_pool("canvas_unified_malformed_test"));

    let (status, _) = send(&app, get_uri("/api/v1/worktrees/not-a-uuid/canvas", &token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// =====================================================================
// PostgreSQL 隔离库用例 (父代理合并时以 STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL 运行)
// =====================================================================

struct CanvasUnifiedFixture {
    tenant_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    branch_id: Uuid,
    run_id: Uuid,
    worktree_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
}

/// 复制 canvas_device_auth_pg::seed_canvas_sse_fixture 的授权行种子模式:
/// 写入一条 tenant/project/branch/run/worktree/device 授权链, 角色可配。
async fn seed_canvas_unified_fixture(
    pool: &PgPool,
    project_role: &str,
    branch_role: &str,
    run_role: &str,
) -> CanvasUnifiedFixture {
    let fixture = CanvasUnifiedFixture {
        tenant_id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        repository_id: Uuid::new_v4(),
        branch_id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        worktree_id: Uuid::new_v4(),
        workspace_id: Uuid::new_v4(),
        actor_id: Uuid::new_v4(),
        device_id: Uuid::new_v4(),
    };
    let mut tx = pool.begin().await.expect("begin fixture");
    sqlx::query(
        "SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true), \
         set_config('app.correlation_id',$3,true), set_config('app.workspace_id',$4,true), \
         set_config('app.role','member',true)",
    )
    .bind(fixture.tenant_id.to_string())
    .bind(fixture.actor_id.to_string())
    .bind(Uuid::new_v4().to_string())
    .bind(fixture.workspace_id.to_string())
    .execute(&mut *tx)
    .await
    .expect("set fixture scope");
    sqlx::query(
        "INSERT INTO permission.project_role_binding \
         (tenant_id,project_id,user_id,role,granted_by) VALUES ($1,$2,$3,$4,$3)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.actor_id)
    .bind(project_role)
    .execute(&mut *tx)
    .await
    .expect("seed project role");
    sqlx::query(
        "INSERT INTO scm.cloud_branch \
         (branch_id,tenant_id,project_id,repository_id,remote_identity) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(fixture.branch_id)
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(format!("canvas-unified:{}", fixture.branch_id))
    .execute(&mut *tx)
    .await
    .expect("seed branch");
    sqlx::query(
        "INSERT INTO scm.cloud_branch_revision \
         (tenant_id,branch_id,name,full_ref,state,changed_by,version) \
         VALUES ($1,$2,'main','refs/heads/main','active',$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.branch_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed branch revision");
    sqlx::query(
        "INSERT INTO permission.cloud_branch_role_binding \
         (tenant_id,branch_id,user_id,role,granted_by,version) \
         VALUES ($1,$2,$3,$4,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.branch_id)
    .bind(fixture.actor_id)
    .bind(branch_role)
    .execute(&mut *tx)
    .await
    .expect("seed branch role");
    sqlx::query(
        "INSERT INTO multica.engineering_run \
         (engineering_run_id,tenant_id,project_id,repository_id,branch_id) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(fixture.run_id)
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(fixture.branch_id)
    .execute(&mut *tx)
    .await
    .expect("seed run");
    sqlx::query(
        "INSERT INTO multica.engineering_run_revision \
         (tenant_id,engineering_run_id,title,state,owner_user_id,changed_by,version) \
         VALUES ($1,$2,'Canvas unified acceptance','active',$3,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed run revision");
    sqlx::query(
        "INSERT INTO permission.engineering_run_role_binding \
         (tenant_id,engineering_run_id,user_id,role,granted_by,version) \
         VALUES ($1,$2,$3,$4,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(fixture.actor_id)
    .bind(run_role)
    .execute(&mut *tx)
    .await
    .expect("seed run role");
    sqlx::query(
        "INSERT INTO worktree_canvas_worktree \
         (id,tenant_id,workspace_id,repo_id,name,branch,path,project_id,owner_user_id) \
         VALUES ($1,$2,$3,$4,$5,'main',$6,$7,$8)",
    )
    .bind(fixture.worktree_id)
    .bind(fixture.tenant_id)
    .bind(fixture.workspace_id)
    .bind(fixture.repository_id)
    .bind(format!("canvas-unified-{}", fixture.worktree_id))
    .bind(format!("/tmp/star-canvas-unified-{}", fixture.worktree_id))
    .bind(fixture.project_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed worktree");
    let project_binding_id: Uuid = sqlx::query_scalar(
        "INSERT INTO multica.worktree_project_binding \
         (tenant_id,project_id,worktree_id,assigned_by) VALUES ($1,$2,$3,$4) RETURNING binding_id",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.worktree_id)
    .bind(fixture.actor_id)
    .fetch_one(&mut *tx)
    .await
    .expect("seed worktree project binding");
    sqlx::query(
        "INSERT INTO multica.engineering_run_worktree_binding \
         (tenant_id,project_id,repository_id,branch_id,engineering_run_id,worktree_id,project_binding_id,assigned_by,version) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(fixture.branch_id)
    .bind(fixture.run_id)
    .bind(fixture.worktree_id)
    .bind(project_binding_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed run worktree binding");
    sqlx::query(
        "INSERT INTO oauth_device_bindings \
         (tenant_id,actor_id,device_id,public_key_fingerprint,generation,status) \
         VALUES ($1,$2,$3,$4,1,'active')",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.actor_id)
    .bind(fixture.device_id)
    .bind(format!("{:064x}", fixture.device_id.as_u128()))
    .execute(&mut *tx)
    .await
    .expect("seed device binding");
    tx.commit().await.expect("commit fixture");
    fixture
}

async fn seed_second_device(pool: &PgPool, fixture: &CanvasUnifiedFixture) -> Uuid {
    let device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO oauth_device_bindings \
         (tenant_id,actor_id,device_id,public_key_fingerprint,generation,status) \
         VALUES ($1,$2,$3,$4,1,'active')",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.actor_id)
    .bind(device_id)
    .bind(format!("{:064x}", device_id.as_u128()))
    .execute(pool)
    .await
    .expect("seed second device binding");
    device_id
}

async fn seed_second_actor_device(
    pool: &PgPool,
    fixture: &CanvasUnifiedFixture,
    project_role: &str,
    branch_role: &str,
    run_role: &str,
) -> (Uuid, Uuid) {
    let actor_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let mut tx = pool.begin().await.expect("begin member fixture");
    sqlx::query(
        "INSERT INTO permission.project_role_binding \
         (tenant_id,project_id,user_id,role,granted_by) VALUES ($1,$2,$3,$4,$3)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(actor_id)
    .bind(project_role)
    .execute(&mut *tx)
    .await
    .expect("seed member project role");
    sqlx::query(
        "INSERT INTO permission.cloud_branch_role_binding \
         (tenant_id,branch_id,user_id,role,granted_by,version) VALUES ($1,$2,$3,$4,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.branch_id)
    .bind(actor_id)
    .bind(branch_role)
    .execute(&mut *tx)
    .await
    .expect("seed member branch role");
    sqlx::query(
        "INSERT INTO permission.engineering_run_role_binding \
         (tenant_id,engineering_run_id,user_id,role,granted_by,version) VALUES ($1,$2,$3,$4,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(actor_id)
    .bind(run_role)
    .execute(&mut *tx)
    .await
    .expect("seed member run role");
    sqlx::query(
        "INSERT INTO oauth_device_bindings \
         (tenant_id,actor_id,device_id,public_key_fingerprint,generation,status) \
         VALUES ($1,$2,$3,$4,1,'active')",
    )
    .bind(fixture.tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .bind(format!("{:064x}", device_id.as_u128()))
    .execute(&mut *tx)
    .await
    .expect("seed member device binding");
    tx.commit().await.expect("commit member fixture");
    (actor_id, device_id)
}

async fn create_canvas_through_route(
    app: &Router,
    fixture: &CanvasUnifiedFixture,
    token: &str,
    title: &str,
) -> Uuid {
    let (status, body) = send(
        app,
        post_json(
            &format!("/api/v1/worktrees/{}/canvases", fixture.worktree_id),
            token,
            &Uuid::new_v4().to_string(),
            json!({"title": title}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create canvas: {body}");
    let canvas_id: Uuid = serde_json::from_value(body["canvas"]["canvas_id"].clone())
        .expect("canvas_id should be a uuid");
    canvas_id
}

async fn create_element_through_route(
    app: &Router,
    fixture: &CanvasUnifiedFixture,
    canvas_id: Uuid,
    token: &str,
    key: &str,
    x: f64,
) -> Uuid {
    let (status, body) = send(
        app,
        post_json(
            &format!(
                "/api/v1/worktrees/{}/canvases/{canvas_id}/elements",
                fixture.worktree_id
            ),
            token,
            key,
            json!({
                "kind": "sticky_note",
                "x": x,
                "y": 20.0,
                "width": 180.0,
                "height": 100.0,
                "rotation": 0.0,
                "z_index": 1,
                "content": {"text": "unified"},
                "locked": false,
                "hidden": false,
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create element: {body}");
    let element_id: Uuid =
        serde_json::from_value(body["element"]["element_id"].clone()).expect("element_id uuid");
    element_id
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn two_entries_share_same_canvas_snapshot() {
    let database_url = std::env::var(PG_TEST_DATABASE_URL)
        .expect("ignored PostgreSQL test requires its database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let fixture =
        seed_canvas_unified_fixture(&pool, "project_admin", "developer", "developer").await;
    let app = unified_app(jwt.clone(), pool.clone());
    let writer_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        fixture.device_id,
        1,
    );
    let canvas_id =
        create_canvas_through_route(&app, &fixture, &writer_token, "shared canvas").await;
    let element_id = create_element_through_route(
        &app,
        &fixture,
        canvas_id,
        &writer_token,
        &Uuid::new_v4().to_string(),
        40.0,
    )
    .await;

    // 入口 1: 独立 /canvas/[id] 入口的解析路由 GET /canvas
    let (status, entry_one) = send(
        &app,
        get_uri(
            &format!("/api/v1/worktrees/{}/canvas", fixture.worktree_id),
            &writer_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_value::<Uuid>(entry_one["canvas"]["canvas_id"].clone()).expect("uuid"),
        canvas_id
    );
    assert_eq!(entry_one["canvas"]["version"], 1);
    assert_eq!(
        entry_one["elements"][0]["element_id"],
        element_id.to_string()
    );
    assert_eq!(entry_one["permission"]["can_write"], true);

    // 入口 2: 同一 worktree 的 Group Canvas 列表入口
    let (status, entry_two) = send(
        &app,
        get_uri(
            &format!("/api/v1/worktrees/{}/canvases", fixture.worktree_id),
            &writer_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let canvases = entry_two["canvases"].as_array().expect("canvases array");
    assert_eq!(canvases.len(), 1);
    assert_eq!(
        serde_json::from_value::<Uuid>(canvases[0]["canvas_id"].clone()).expect("uuid"),
        canvas_id
    );
    assert_eq!(
        canvases[0]["document"]["version"],
        entry_one["canvas"]["version"]
    );

    // 第二设备 (另一入口会话) 读同一画布 → 元素一致
    let second_device = seed_second_device(&pool, &fixture).await;
    let second_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read canvas:read",
        second_device,
        1,
    );
    let (status, second_entry) = send(
        &app,
        get_uri(
            &format!("/api/v1/worktrees/{}/canvas", fixture.worktree_id),
            &second_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        second_entry["canvas"]["canvas_id"],
        entry_one["canvas"]["canvas_id"]
    );
    assert_eq!(second_entry["elements"], entry_one["elements"]);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn read_only_member_write_verbs_forbidden() {
    let database_url = std::env::var(PG_TEST_DATABASE_URL)
        .expect("ignored PostgreSQL test requires its database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let fixture =
        seed_canvas_unified_fixture(&pool, "project_admin", "developer", "developer").await;
    let app = unified_app(jwt.clone(), pool.clone());
    let writer_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        fixture.device_id,
        1,
    );
    let canvas_id =
        create_canvas_through_route(&app, &fixture, &writer_token, "guarded canvas").await;
    let element_id = create_element_through_route(
        &app,
        &fixture,
        canvas_id,
        &writer_token,
        &Uuid::new_v4().to_string(),
        10.0,
    )
    .await;

    let (member_id, member_device) =
        seed_second_actor_device(&pool, &fixture, "member", "member", "member").await;
    let member_token = device_token(
        &jwt,
        member_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        member_device,
        1,
    );

    // 读允许, 但权限快照声明不可写
    let (status, snapshot) = send(
        &app,
        get_uri(
            &format!("/api/v1/worktrees/{}/canvas", fixture.worktree_id),
            &member_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snapshot["permission"]["can_write"], false);

    // POST / PUT / DELETE 全部 403 (授权真源按角色拒绝, 不裸奔)
    let (status, _) = send(
        &app,
        post_json(
            &format!("/api/v1/worktrees/{}/canvases", fixture.worktree_id),
            &member_token,
            &Uuid::new_v4().to_string(),
            json!({"title": "member cannot create"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send(
        &app,
        put_json(
            &format!(
                "/api/v1/worktrees/{}/canvases/{canvas_id}/elements/{element_id}",
                fixture.worktree_id
            ),
            &member_token,
            &Uuid::new_v4().to_string(),
            json!({"expected_version": 1, "update_mode": "position", "x": 300.0, "y": 300.0}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send(
        &app,
        delete_json(
            &format!(
                "/api/v1/worktrees/{}/canvases/{canvas_id}/elements/{element_id}",
                fixture.worktree_id
            ),
            &member_token,
            &Uuid::new_v4().to_string(),
            json!({"expected_version": 1}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn idempotent_create_element_does_not_duplicate() {
    let database_url = std::env::var(PG_TEST_DATABASE_URL)
        .expect("ignored PostgreSQL test requires its database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let fixture =
        seed_canvas_unified_fixture(&pool, "project_admin", "developer", "developer").await;
    let app = unified_app(jwt.clone(), pool.clone());
    let writer_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        fixture.device_id,
        1,
    );
    let canvas_id =
        create_canvas_through_route(&app, &fixture, &writer_token, "idempotent canvas").await;
    let shared_key = Uuid::new_v4().to_string();
    let first =
        create_element_through_route(&app, &fixture, canvas_id, &writer_token, &shared_key, 50.0)
            .await;
    let second =
        create_element_through_route(&app, &fixture, canvas_id, &writer_token, &shared_key, 50.0)
            .await;
    assert_eq!(
        first, second,
        "idempotent replay returns the existing element"
    );

    let (status, snapshot) = send(
        &app,
        get_uri(
            &format!("/api/v1/worktrees/{}/canvas", fixture.worktree_id),
            &writer_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let elements = snapshot["elements"].as_array().expect("elements array");
    assert_eq!(
        elements.len(),
        1,
        "duplicate write must not create a second entry"
    );
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn stale_version_conflict_returns_409_with_both_sides() {
    let database_url = std::env::var(PG_TEST_DATABASE_URL)
        .expect("ignored PostgreSQL test requires its database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let fixture =
        seed_canvas_unified_fixture(&pool, "project_admin", "developer", "developer").await;
    let app = unified_app(jwt.clone(), pool.clone());
    let writer_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        fixture.device_id,
        1,
    );
    let canvas_id =
        create_canvas_through_route(&app, &fixture, &writer_token, "conflict canvas").await;
    let element_id = create_element_through_route(
        &app,
        &fixture,
        canvas_id,
        &writer_token,
        &Uuid::new_v4().to_string(),
        60.0,
    )
    .await;

    // 第一次移动成功 (v1 → v2)
    let (status, _) = send(
        &app,
        put_json(
            &format!(
                "/api/v1/worktrees/{}/canvases/{canvas_id}/elements/{element_id}",
                fixture.worktree_id
            ),
            &writer_token,
            &Uuid::new_v4().to_string(),
            json!({"expected_version": 1, "update_mode": "position", "x": 200.0, "y": 200.0}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 第二次用过期 expected_version=1 → 409 + 双方内容
    let (status, conflict) = send(
        &app,
        put_json(
            &format!(
                "/api/v1/worktrees/{}/canvases/{canvas_id}/elements/{element_id}",
                fixture.worktree_id
            ),
            &writer_token,
            &Uuid::new_v4().to_string(),
            json!({"expected_version": 1, "update_mode": "position", "x": 400.0, "y": 400.0}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(conflict["error"]["code"], "version_conflict");
    // 服务端最新侧: 元素当前版本与坐标
    assert_eq!(conflict["current"]["element_id"], element_id.to_string());
    assert_eq!(conflict["current"]["version"], 2);
    assert_eq!(conflict["current"]["x"], 200.0);
    // 客户端提交侧: 草稿内容保留
    assert_eq!(conflict["submitted"]["expected_version"], 1);
    assert_eq!(conflict["submitted"]["x"], 400.0);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn document_version_conflict_returns_409_with_both_sides() {
    let database_url = std::env::var(PG_TEST_DATABASE_URL)
        .expect("ignored PostgreSQL test requires its database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let fixture =
        seed_canvas_unified_fixture(&pool, "project_admin", "developer", "developer").await;
    let app = unified_app(jwt.clone(), pool.clone());
    let writer_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        fixture.device_id,
        1,
    );
    let canvas_id =
        create_canvas_through_route(&app, &fixture, &writer_token, "doc conflict canvas").await;

    // 文档已 v1 → 提交 expected_version=2 (伪造超前版本) → 409 + 双方
    let (status, conflict) = send(
        &app,
        put_json(
            &format!(
                "/api/v1/worktrees/{}/canvases/{canvas_id}/document",
                fixture.worktree_id
            ),
            &writer_token,
            &Uuid::new_v4().to_string(),
            json!({
                "expected_version": 2,
                "viewport": {"x": 10.0, "y": 10.0, "zoom": 1.0},
                "frames": [],
                "connectors": [],
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        conflict["error"]["code"],
        "canvas_document_version_conflict"
    );
    assert_eq!(conflict["current"]["version"], 1);
    assert_eq!(conflict["submitted"]["expected_version"], 2);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated, fully migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn cursor_catch_up_loses_nothing_duplicates_nothing() {
    let database_url = std::env::var(PG_TEST_DATABASE_URL)
        .expect("ignored PostgreSQL test requires its database URL");
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("isolated PostgreSQL endpoint");
    let fixture =
        seed_canvas_unified_fixture(&pool, "project_admin", "developer", "developer").await;
    let app = unified_app(jwt.clone(), pool.clone());
    let writer_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "worktree:read worktree:write canvas:read canvas:write",
        fixture.device_id,
        1,
    );
    let canvas_id =
        create_canvas_through_route(&app, &fixture, &writer_token, "catch-up canvas").await;
    create_element_through_route(
        &app,
        &fixture,
        canvas_id,
        &writer_token,
        &Uuid::new_v4().to_string(),
        70.0,
    )
    .await;
    create_element_through_route(
        &app,
        &fixture,
        canvas_id,
        &writer_token,
        &Uuid::new_v4().to_string(),
        90.0,
    )
    .await;

    let events_path = format!("/api/v1/worktrees/{}/canvas-events", fixture.worktree_id);
    // 断线前的持久化 cursor 从 0 重放
    let (status, first_page) = send(
        &app,
        get_uri(
            &format!("{events_path}?after_offset=0&limit=100"),
            &writer_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let first_events = first_page["events"].as_array().expect("events array");
    assert!(
        first_events.len() >= 2,
        "canvas.created + 2 element events: {first_page}"
    );
    assert_eq!(first_page["has_more"], false);
    let next_offset = first_page["next_offset"]
        .as_str()
        .expect("next_offset")
        .to_owned();

    // 重放同一 cursor → 相同事件序列 (不丢不重)
    let (_, replay) = send(
        &app,
        get_uri(
            &format!("{events_path}?after_offset=0&limit=100"),
            &writer_token,
        ),
    )
    .await;
    assert_eq!(replay["events"], first_page["events"]);

    // 从持久化 cursor 续传 → 空页, cursor 不变
    let (status, tail) = send(
        &app,
        get_uri(
            &format!("{events_path}?after_offset={next_offset}&limit=100"),
            &writer_token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tail["events"].as_array().expect("events array").len(), 0);
    assert_eq!(tail["has_more"], false);
    assert_eq!(tail["next_offset"], next_offset);

    // 事件 offset 严格递增
    let offsets: Vec<i64> = first_events
        .iter()
        .map(|event| {
            event["stream_offset"]
                .as_str()
                .expect("offset")
                .parse()
                .expect("i64")
        })
        .collect();
    assert!(
        offsets.windows(2).all(|pair| pair[0] < pair[1]),
        "offsets strictly increasing"
    );
    pool.close().await;
}
