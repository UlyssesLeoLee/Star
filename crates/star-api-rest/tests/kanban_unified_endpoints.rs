// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/kanban_unified_endpoints.rs"})
MERGE (read403:Symbol {id:"crates/star-api-rest/tests/kanban_unified_endpoints.rs::work_item_read_without_scope_is_403",kind:"test"})
MERGE (device401:Symbol {id:"crates/star-api-rest/tests/kanban_unified_endpoints.rs::shared_move_without_device_identity_is_401",kind:"test"})
MERGE (legacy501:Symbol {id:"crates/star-api-rest/tests/kanban_unified_endpoints.rs::legacy_work_item_writes_are_fail_closed_501",kind:"test"})
MERGE (moveMounted:Symbol {id:"crates/star-api-rest/tests/kanban_unified_endpoints.rs::move_route_is_mounted_and_post_only",kind:"test"})
MERGE (converge:Symbol {id:"crates/star-api-rest/tests/kanban_unified_endpoints.rs::kanban_and_sprint_entries_converge_on_one_truth",kind:"test"})
MERGE (self)-[:DEFINES]->(read403)
MERGE (self)-[:DEFINES]->(device401)
MERGE (self)-[:DEFINES]->(legacy501)
MERGE (self)-[:DEFINES]->(moveMounted)
MERGE (self)-[:DEFINES]->(converge)
MERGE (read403)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::list_work_items",kind:"function"})
MERGE (device401)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::move_work_item",kind:"function"})
MERGE (legacy501)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::create",kind:"function"})
MERGE (legacy501)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/routes/work_items.rs::update",kind:"function"})
MERGE (moveMounted)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::move_work_item",kind:"function"})
MERGE (converge)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::move_work_item",kind:"function"})
MERGE (converge)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::transition_work_item",kind:"function"})
MERGE (converge)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::transition_run_work_item",kind:"function"})
MERGE (converge)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::list_work_items",kind:"function"})
MERGE (converge)-[:TESTS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::list_run_work_items",kind:"function"})
MERGE (converge)-[:READS]->(:Config {id:"STAR_P1_KANBAN_TEST_DATABASE_URL",kind:"environment_variable"})
@endcypher
*/
//! P1 Kanban 统一端点集成测试 — Backlog/Sprint/Kanban 共享同一服务端真源。
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 `axum::Router::oneshot`。
//!
//! 分层:
//! - 无数据库 tests (本机可跑): 越权 403 / 设备身份 fail-closed 401 / 旧内存态写路由 501 / move 挂载;
//! - `#[ignore]` PostgreSQL tests (父代理在隔离库跑, `STAR_P1_KANBAN_TEST_DATABASE_URL`):
//!   同一卡从 kanban 语义 (列移动) 与 sprint 语义 (run 生命周期) 两入口写入 → 查询一致;
//!   并发 lock_version 冲突 → 409; 非法转换 → 409 明确错误; 越权 → 403; 创建挂 worktree(branch) → 按 worktree 查询命中。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use star_api_rest::{
    auth::{issue_token, issue_token_with_device_context, DeviceTokenContext, JwtConfig},
    build_supplemental_routers,
    group_api::{build_group_router, GroupApiState},
};
use tower::ServiceExt;
use uuid::Uuid;

fn route_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

/// Lazy pool with a dummy DSN — the non-DB tests never touch it
/// (鉴权/校验在数据库访问之前 fail-closed, per production_router.rs 同模式).
fn lazy_test_pool() -> PgPool {
    sqlx::PgPool::connect_lazy("postgresql://dummy:unused@localhost/kanban_unified_test")
        .expect("lazy pool")
}

fn plain_token(config: &JwtConfig, actor_id: Uuid, tenant_id: Uuid, scope: &str) -> String {
    issue_token(
        config,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("fixed test-only RSA key should issue a signed token")
    .0
}

fn device_token(
    config: &JwtConfig,
    actor_id: Uuid,
    tenant_id: Uuid,
    scope: &str,
    device_id: Uuid,
) -> String {
    issue_token_with_device_context(
        config,
        actor_id,
        tenant_id,
        vec!["developer".to_owned()],
        scope.to_owned(),
        Some(DeviceTokenContext {
            device_id,
            generation: 1,
        }),
    )
    .expect("fixed test-only RSA key should sign device-bound route test tokens")
    .0
}

fn get_request(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn post_request(path: &str, token: &str, idempotency_key: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("Idempotency-Key", idempotency_key)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

// =====================================================================
// 无数据库 tests (本机可跑, fail-closed 边界)
// =====================================================================

/// 越权读取被拒绝: 有会话但没有 work-item:read scope → 403。
#[tokio::test]
async fn work_item_read_without_scope_is_403() {
    let jwt = route_test_jwt_config();
    let app = build_group_router(GroupApiState::new(jwt.clone(), lazy_test_pool()));
    let token = plain_token(&jwt, Uuid::new_v4(), Uuid::new_v4(), "worktree:read");
    let response = app
        .oneshot(get_request(
            &format!("/api/v1/worktrees/{}/work-items", Uuid::new_v4()),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 共享写入 fail-closed: 无设备身份的 POST move → 401 (W16 设备代际守卫)。
#[tokio::test]
async fn shared_move_without_device_identity_is_401() {
    let jwt = route_test_jwt_config();
    let app = build_group_router(GroupApiState::new(jwt.clone(), lazy_test_pool()));
    let token = plain_token(
        &jwt,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "work-item:read work-item:write",
    );
    let response = app
        .oneshot(post_request(
            &format!(
                "/api/v1/worktrees/{}/work-items/{}/move",
                Uuid::new_v4(),
                Uuid::new_v4()
            ),
            &token,
            "k1",
            json!({ "target_column": "in_progress", "expected_version": 1 }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// 旧内存态写入路由 fail-closed: POST/PATCH /api/v1/work-items → 501,
/// 统一写入路径已收敛到 group_api (PostgreSQL + 鉴权)。
#[tokio::test]
async fn legacy_work_item_writes_are_fail_closed_501() {
    let jwt = route_test_jwt_config();
    let app = build_group_router(GroupApiState::new(jwt.clone(), lazy_test_pool()))
        .merge(build_supplemental_routers());
    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/work-items")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::NOT_IMPLEMENTED);
    let update = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(&format!("/api/v1/work-items/{}", Uuid::new_v4()))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::NOT_IMPLEMENTED);
}

/// move 路由已挂载且仅接受 POST (GET → 405 仍非 404)。
#[tokio::test]
async fn move_route_is_mounted_and_post_only() {
    let jwt = route_test_jwt_config();
    let app = build_group_router(GroupApiState::new(jwt.clone(), lazy_test_pool()));
    let token = plain_token(&jwt, Uuid::new_v4(), Uuid::new_v4(), "work-item:write");
    let response = app
        .oneshot(get_request(
            &format!(
                "/api/v1/worktrees/{}/work-items/{}/move",
                Uuid::new_v4(),
                Uuid::new_v4()
            ),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// =====================================================================
// PostgreSQL tests (#[ignore] — 父代理在隔离迁移库跑)
// =====================================================================

struct UnifiedKanbanFixture {
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

async fn seed_unified_kanban_fixture(pool: &PgPool) -> UnifiedKanbanFixture {
    let fixture = UnifiedKanbanFixture {
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
    let mut tx = pool.begin().await.expect("begin persisted unified fixture");
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
    .expect("set persisted fixture owner and Worktree scope");

    sqlx::query(
        "INSERT INTO permission.project_role_binding \
         (tenant_id,project_id,user_id,role,granted_by) VALUES ($1,$2,$3,'project_admin',$3)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Project management authority");
    sqlx::query(
        "INSERT INTO scm.cloud_branch \
         (branch_id,tenant_id,project_id,repository_id,remote_identity) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(fixture.branch_id)
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.repository_id)
    .bind(format!("kanban-unified-test:{}", fixture.branch_id))
    .execute(&mut *tx)
    .await
    .expect("seed canonical persisted Branch");
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
    .expect("seed current Branch revision");
    sqlx::query(
        "INSERT INTO permission.cloud_branch_role_binding \
         (tenant_id,branch_id,user_id,role,granted_by,version) \
         VALUES ($1,$2,$3,'developer',$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.branch_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Branch read authority");
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
    .expect("seed persisted Engineering Run");
    sqlx::query(
        "INSERT INTO multica.engineering_run_revision \
         (tenant_id,engineering_run_id,title,state,owner_user_id,changed_by,version) \
         VALUES ($1,$2,'Kanban Unified acceptance','active',$3,$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Run revision");
    sqlx::query(
        "INSERT INTO permission.engineering_run_role_binding \
         (tenant_id,engineering_run_id,user_id,role,granted_by,version) \
         VALUES ($1,$2,$3,'developer',$3,1)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.run_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed current Run read authority");
    sqlx::query(
        "INSERT INTO worktree_canvas_worktree \
         (id,tenant_id,workspace_id,repo_id,name,branch,path,project_id,owner_user_id) \
         VALUES ($1,$2,$3,$4,$5,'main',$6,$7,$8)",
    )
    .bind(fixture.worktree_id)
    .bind(fixture.tenant_id)
    .bind(fixture.workspace_id)
    .bind(fixture.repository_id)
    .bind(format!("kanban-unified-{}", fixture.worktree_id))
    .bind(format!("/tmp/star-kanban-unified-{}", fixture.worktree_id))
    .bind(fixture.project_id)
    .bind(fixture.actor_id)
    .execute(&mut *tx)
    .await
    .expect("seed persisted Worktree under tenant and workspace scope");
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
    .expect("seed persisted Project-to-Worktree binding");
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
    .expect("seed persisted Engineering Run-to-Worktree binding");
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
    .expect("seed current persisted active device binding");
    tx.commit()
        .await
        .expect("Run/Worktree/device fixture should commit");
    fixture
}

async fn cleanup_unified_kanban_fixture(pool: &PgPool, fixture: &UnifiedKanbanFixture) {
    let mut tx = pool.begin().await.expect("begin unified fixture cleanup");
    for statement in [
        "DELETE FROM multica.task_command_idempotency WHERE tenant_id=$1",
        "DELETE FROM multica.task_lifecycle_current WHERE tenant_id=$1",
        "DELETE FROM multica.task_metadata WHERE tenant_id=$1",
        "DELETE FROM multica.work_item_worktree WHERE tenant_id=$1",
        "DELETE FROM oauth_device_bindings WHERE tenant_id=$1 AND actor_id=$2",
        "DELETE FROM multica.engineering_run_worktree_binding WHERE tenant_id=$1",
        "DELETE FROM multica.worktree_project_binding WHERE tenant_id=$1",
        "DELETE FROM worktree_canvas_worktree WHERE tenant_id=$1 AND id=$3",
        "DELETE FROM permission.engineering_run_role_binding WHERE tenant_id=$1",
        "DELETE FROM multica.engineering_run_revision WHERE tenant_id=$1",
        "DELETE FROM multica.engineering_run WHERE tenant_id=$1",
        "DELETE FROM permission.cloud_branch_role_binding WHERE tenant_id=$1",
        "DELETE FROM scm.cloud_branch_revision WHERE tenant_id=$1",
        "DELETE FROM scm.cloud_branch WHERE tenant_id=$1",
        "DELETE FROM permission.project_role_binding WHERE tenant_id=$1",
    ] {
        sqlx::query(statement)
            .bind(fixture.tenant_id)
            .bind(fixture.actor_id)
            .bind(fixture.worktree_id)
            .execute(&mut *tx)
            .await
            .expect("cleanup unified fixture row");
    }
    tx.commit().await.expect("unified fixture cleanup commit");
}

async fn response_body(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("JSON response body")
}

/// 验收主流程: 同一卡从「kanban 语义」(列移动) 与「sprint 语义」(run 生命周期)
/// 两入口写入 → 任一入口查询一致; 并发 lock_version 冲突 → 409; 非法转换 → 409;
/// 越权 → 403; 创建挂 worktree (branch) → 按 worktree 查询命中。
#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_P1_KANBAN_TEST_DATABASE_URL"]
async fn kanban_and_sprint_entries_converge_on_one_truth() {
    let database_url = std::env::var("STAR_P1_KANBAN_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = PgPoolOptions::new()
        .max_connections(6)
        .connect(&database_url)
        .await
        .expect("connect to isolated unified test database");
    let fixture = seed_unified_kanban_fixture(&pool).await;
    let jwt = route_test_jwt_config();
    let app = build_group_router(GroupApiState::new(jwt.clone(), pool.clone()));
    let write_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "work-item:read work-item:write",
        fixture.device_id,
    );
    let read_token = device_token(
        &jwt,
        fixture.actor_id,
        fixture.tenant_id,
        "work-item:read",
        fixture.device_id,
    );
    let worktree_items_path = format!("/api/v1/worktrees/{}/work-items", fixture.worktree_id);
    let lifecycle_path = format!("/api/v1/worktrees/{}/work-items", fixture.worktree_id);

    // 1) 创建挂 branch/worktree (同一写入路径)
    let create = app
        .clone()
        .oneshot(post_request(
            &worktree_items_path,
            &write_token,
            "create-1",
            json!({
                "item_type": "task",
                "title": "P1 统一卡",
                "description": "kanban/sprint 双入口一致性验收",
                "priority": "medium",
                "labels": [],
                "correlation_id": Uuid::new_v4(),
            }),
        ))
        .await
        .unwrap();
    assert_eq!(
        create.status(),
        StatusCode::OK,
        "worktree-scoped create must succeed"
    );
    let created: Value = response_body(create).await;
    let work_item_id = created["work_item_id"]
        .as_str()
        .expect("created snapshot carries work_item_id")
        .to_string();
    assert_eq!(created["lifecycle"]["status"], "pending");
    assert_eq!(created["lifecycle"]["version"], 1);

    // 2) 创建挂 worktree → 按 worktree 查询命中 (Backlog 语义入口)
    let listed = app
        .clone()
        .oneshot(get_request(&worktree_items_path, &write_token))
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let listed_body: Value = response_body(listed).await;
    let items = listed_body["work_items"]
        .as_array()
        .expect("work_items array");
    assert!(
        items
            .iter()
            .any(|item| item["work_item_id"].as_str() == Some(work_item_id.as_str())),
        "created card must be visible from the worktree (branch) query"
    );

    // 3) kanban 语义: 同列移动 (todo → todo) 为视图重排, 不产生服务端写
    let noop_move = app
        .clone()
        .oneshot(post_request(
            &format!("{lifecycle_path}/{work_item_id}/move",),
            &write_token,
            "move-noop",
            json!({ "target_column": "todo", "expected_version": 1 }),
        ))
        .await
        .unwrap();
    assert_eq!(noop_move.status(), StatusCode::OK);
    let noop_body: Value = response_body(noop_move).await;
    assert_eq!(noop_body["moved"], false);
    assert_eq!(noop_body["task_card"]["lifecycle"]["status"], "pending");
    assert_eq!(noop_body["task_card"]["lifecycle"]["version"], 1);

    // 4) 状态规则统一: pending → in_progress 非法 → 409 明确错误码
    let illegal_move = app
        .clone()
        .oneshot(post_request(
            &format!("{lifecycle_path}/{work_item_id}/move",),
            &write_token,
            "move-illegal",
            json!({ "target_column": "in_progress", "expected_version": 1 }),
        ))
        .await
        .unwrap();
    assert_eq!(illegal_move.status(), StatusCode::CONFLICT);
    let illegal_body: Value = response_body(illegal_move).await;
    assert_eq!(illegal_body["error"]["code"], "invalid_transition");

    // 5) 领取 (合法转换 pending → claimed, v1 → v2)
    let claim = app
        .clone()
        .oneshot(post_request(
            &format!("{lifecycle_path}/{work_item_id}/lifecycle",),
            &write_token,
            "claim-1",
            json!({ "target_status": "claimed", "expected_version": 1, "correlation_id": Uuid::new_v4() }),
        ))
        .await
        .unwrap();
    assert_eq!(claim.status(), StatusCode::OK, "claim must succeed");

    // 6) 并发 lock_version 冲突: 两个 move 同 expected_version=2, 一个成功一个 409
    let first = app
        .clone()
        .oneshot(post_request(
            &format!("{lifecycle_path}/{work_item_id}/move",),
            &write_token,
            "move-race-a",
            json!({ "target_column": "in_progress", "expected_version": 2 }),
        ))
        .await
        .unwrap();
    let second = app
        .clone()
        .oneshot(post_request(
            &format!("{lifecycle_path}/{work_item_id}/move",),
            &write_token,
            "move-race-b",
            json!({ "target_column": "in_progress", "expected_version": 2 }),
        ))
        .await
        .unwrap();
    let statuses = [first.status(), second.status()];
    assert!(
        statuses.contains(&StatusCode::OK) && statuses.contains(&StatusCode::CONFLICT),
        "concurrent lock_version moves must yield exactly one success and one 409, got {:?}",
        statuses
    );
    let conflict_response = if first.status() == StatusCode::CONFLICT {
        first
    } else {
        second
    };
    let conflict_body: Value = response_body(conflict_response).await;
    assert_eq!(conflict_body["error"]["code"], "version_conflict");

    // 7) 越权写入被拒绝: 只有 work-item:read 的会话 → 403
    let forbidden = app
        .clone()
        .oneshot(post_request(
            &format!("{lifecycle_path}/{work_item_id}/lifecycle",),
            &read_token,
            "forbidden-1",
            json!({ "target_status": "cancelled", "expected_version": 3 }),
        ))
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    // 8) sprint 语义入口 (run-scoped lifecycle) 继续同一卡: in_progress → completed (v3 → v4)
    let run_transition = app
        .clone()
        .oneshot(post_request(
            &format!(
                "/api/v1/engineering-runs/{}/work-items/{}/lifecycle",
                fixture.run_id, work_item_id
            ),
            &write_token,
            "sprint-1",
            json!({
                "target_status": "completed",
                "expected_version": 3,
                "focus_worktree_id": fixture.worktree_id,
                "correlation_id": Uuid::new_v4(),
            }),
        ))
        .await
        .unwrap();
    assert_eq!(
        run_transition.status(),
        StatusCode::OK,
        "run-scoped transition must succeed"
    );
    let run_body: Value = response_body(run_transition).await;
    assert_eq!(run_body["task_card"]["lifecycle"]["status"], "completed");
    assert_eq!(run_body["task_card"]["lifecycle"]["version"], 4);

    // 9) 查询一致: kanban 入口 (worktree list) 与 sprint 入口 (run list) 都看到 completed v4
    let worktree_view = app
        .clone()
        .oneshot(get_request(&worktree_items_path, &write_token))
        .await
        .unwrap();
    let worktree_body: Value = response_body(worktree_view).await;
    let worktree_card = worktree_body["work_items"]
        .as_array()
        .expect("work_items array")
        .iter()
        .find(|item| item["work_item_id"].as_str() == Some(work_item_id.as_str()))
        .cloned()
        .expect("card must remain visible from the worktree query");
    assert_eq!(worktree_card["lifecycle"]["status"], "completed");
    assert_eq!(worktree_card["lifecycle"]["version"], 4);

    let run_view = app
        .clone()
        .oneshot(get_request(
            &format!("/api/v1/engineering-runs/{}/work-items", fixture.run_id),
            &write_token,
        ))
        .await
        .unwrap();
    assert_eq!(run_view.status(), StatusCode::OK);
    let run_body: Value = response_body(run_view).await;
    let run_card = run_body["work_items"]
        .as_array()
        .expect("work_items array")
        .iter()
        .find(|item| item["work_item_id"].as_str() == Some(work_item_id.as_str()))
        .cloned()
        .expect("card must remain visible from the run (sprint) query");
    assert_eq!(run_card["lifecycle"]["status"], "completed");
    assert_eq!(run_card["lifecycle"]["version"], 4);

    cleanup_unified_kanban_fixture(&pool, &fixture).await;
    pool.close().await;
}
