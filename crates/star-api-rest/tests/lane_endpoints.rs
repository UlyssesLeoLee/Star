/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/tests/lane_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_endpoints",kind:"test_suite"})
MERGE (unauth:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_read_requires_authentication_401",kind:"test"})
MERGE (garbage:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_read_rejects_garbage_token_401",kind:"test"})
MERGE (scope:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_list_requires_project_scope_403",kind:"test"})
MERGE (badProject:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_list_invalid_project_uuid_400",kind:"test"})
MERGE (badLane:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_get_invalid_lane_uuid_400",kind:"test"})
MERGE (legacy:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_create_legacy_session_without_device_context_401",kind:"test"})
MERGE (mounted:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::lane_routes_mounted_in_production_router",kind:"test"})
MERGE (pg:Symbol {id:"crates/star-api-rest/tests/lane_endpoints.rs::pg_lane_endpoints",kind:"test_suite"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(unauth)
MERGE (suite)-[:DEFINES]->(garbage)
MERGE (suite)-[:DEFINES]->(scope)
MERGE (suite)-[:DEFINES]->(badProject)
MERGE (suite)-[:DEFINES]->(badLane)
MERGE (suite)-[:DEFINES]->(legacy)
MERGE (suite)-[:DEFINES]->(mounted)
MERGE (self)-[:DEFINES]->(pg)
@endcypher
*/
//! Lane 端点集成测试 (P0-lane 2026-11-20)
//!
//! 守门 #25 v25: 集成测试不发真 HTTP — 全部走 axum::Router::oneshot。
//!
//! 两层:
//! 1. 非 #[ignore] 测试 — 无 PG 也能跑 (401/403/400 都在触达 DB 前判定, fail-closed)。
//!    验证: 未登录 401、无效 token 401、缺 project scope 403、非法 UUID 400、
//!    无 device context 的旧会话无法写 Lane (401)、lane 路由已挂载 (非 404)。
//! 2. #[ignore] PG 测试 — 走真实 PostgreSQL, 需要:
//!    - 环境变量 `STAR_PG_LANE_TEST_DATABASE_URL`
//!    - 该库已应用 db/migrations 全链 + `crates/worktree-service/migrations/2026-11-20-lane.sql`
//!    运行: cargo test -p star-api-rest --test lane_endpoints -- --ignored --test-threads=1
//!
//! PG 测试验证: 无远端仓库/分支也能创建 Lane (验收第一条)、跨租户/非成员 403、
//! 创建→重建 router→仍在 (真 PG 持久化)、撤权后 list 403 不可见、
//! worktree 挂 lane 后按 lane 过滤正确、乐观锁 409、删除。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::json;
use star_api_rest::auth::{issue_token_with_device_context, DeviceTokenContext, JwtConfig};
use star_api_rest::group_api::{build_group_router, GroupApiState};
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

fn route_test_token(
    config: &JwtConfig,
    user_id: Uuid,
    tenant_id: Uuid,
    scope: &str,
    device: Option<DeviceTokenContext>,
) -> String {
    issue_token_with_device_context(
        config,
        user_id,
        tenant_id,
        vec!["developer".to_owned()],
        scope.to_owned(),
        device,
    )
    .expect("fixed test-only RSA key should sign device-bound route test tokens")
    .0
}

fn get_uri(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn get_with_body(path: &str, token: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp: axum::response::Response = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (status, body)
}

fn lazy_app(jwt: Arc<JwtConfig>) -> Router {
    let pool = sqlx::PgPool::connect_lazy("postgresql://dummy:***@localhost/lane_endpoints_test")
        .expect("lazy pool");
    let state = GroupApiState::new(jwt, pool);
    build_group_router(state)
}

const PROJECT: Uuid = Uuid::from_u128(0x10000000_0000_4000_8000_000000000001);
const LANE: Uuid = Uuid::from_u128(0x10000000_0000_4000_8000_000000000002);

// =====================================================================
// 非 ignore: fail-closed 前置判定 (不触达 DB)
// =====================================================================

#[tokio::test]
async fn lane_read_requires_authentication_401() {
    let app = lazy_app(route_test_jwt_config());
    let req = Request::builder()
        .uri(format!("/api/v1/projects/{PROJECT}/lanes"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn lane_read_rejects_garbage_token_401() {
    let app = lazy_app(route_test_jwt_config());
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/projects/{PROJECT}/lanes"), "not-a-jwt"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("BEARER_INVALID_FORMAT"));
}

#[tokio::test]
async fn lane_list_requires_project_scope_403() {
    let app = lazy_app(route_test_jwt_config());
    let token = route_test_token(
        &route_test_jwt_config(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        "worktree:read",
        None,
    );
    let (status, body) = send(
        &app,
        get_uri(&format!("/api/v1/projects/{PROJECT}/lanes"), &token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], json!("insufficient_scope"));
}

#[tokio::test]
async fn lane_list_invalid_project_uuid_400() {
    let app = lazy_app(route_test_jwt_config());
    let token = route_test_token(
        &route_test_jwt_config(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        "project:read",
        None,
    );
    let (status, _) = send(&app, get_uri("/api/v1/projects/not-a-uuid/lanes", &token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lane_get_invalid_lane_uuid_400() {
    let app = lazy_app(route_test_jwt_config());
    let token = route_test_token(
        &route_test_jwt_config(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        "project:read",
        None,
    );
    let (status, _) = send(
        &app,
        get_uri(
            &format!("/api/v1/projects/{PROJECT}/lanes/not-a-uuid"),
            &token,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lane_create_legacy_session_without_device_context_401() {
    // 旧会话 (无 device context) 不能写 Lane — extractor fail-closed 401, 不触达 DB。
    let app = lazy_app(route_test_jwt_config());
    let token = route_test_token(
        &route_test_jwt_config(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        "project:write",
        None,
    );
    let (status, _) = send(
        &app,
        get_with_body(
            &format!("/api/v1/projects/{PROJECT}/lanes"),
            &token,
            json!({ "name": "core-runtime" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn lane_routes_mounted_in_production_router() {
    let jwt = route_test_jwt_config();
    let token = route_test_token(&jwt, Uuid::new_v4(), Uuid::new_v4(), "project:read", None);
    let pool = sqlx::PgPool::connect_lazy("postgresql://dummy:***@localhost/lane_mount_test")
        .expect("lazy pool");
    let state = GroupApiState::new(jwt, pool);
    let app = build_group_router(state);
    for path in [
        format!("/api/v1/projects/{PROJECT}/lanes"),
        format!("/api/v1/projects/{PROJECT}/lanes/{LANE}"),
        format!("/api/v1/projects/{PROJECT}/lanes/{LANE}/worktrees"),
    ] {
        // lazy pool 无连接 → 500 (或更早的 400/401); 404 = 未挂载
        let (status, _) = send(&app, get_uri(&path, &token)).await;
        assert_ne!(
            status,
            StatusCode::NOT_FOUND,
            "lane endpoint {path} should be mounted (got 404)"
        );
    }
}

// =====================================================================
// #[ignore] PG 测试 (真实 PostgreSQL; 见文件头前置条件)
// =====================================================================

mod pg {
    use super::*;

    fn pg_url() -> String {
        std::env::var("STAR_PG_LANE_TEST_DATABASE_URL").expect(
            "set STAR_PG_LANE_TEST_DATABASE_URL (full db/migrations chain + lane migration)",
        )
    }

    async fn pg_pool() -> sqlx::PgPool {
        sqlx::PgPool::connect(&pg_url()).await.expect("connect pg")
    }

    async fn pg_app() -> (Arc<JwtConfig>, Router, sqlx::PgPool) {
        let jwt = route_test_jwt_config();
        let pool = pg_pool().await;
        let state = GroupApiState::new(jwt.clone(), pool.clone());
        (jwt, build_group_router(state), pool)
    }

    /// 测试租户 fixture — 每测试独立 (tenant/project/user/device 全新 UUID)
    struct TenantFixture {
        tenant_id: Uuid,
        project_id: Uuid,
        admin_user: Uuid,
        member_user: Uuid,
        device_id: Uuid,
        device_generation: i64,
    }

    impl TenantFixture {
        fn new() -> Self {
            Self {
                tenant_id: Uuid::new_v4(),
                project_id: Uuid::new_v4(),
                admin_user: Uuid::new_v4(),
                member_user: Uuid::new_v4(),
                device_id: Uuid::new_v4(),
                device_generation: 1,
            }
        }
    }

    async fn seed_role_binding(
        pool: &sqlx::PgPool,
        tenant: Uuid,
        project: Uuid,
        user: Uuid,
        role: &str,
    ) {
        let mut conn = pool.acquire().await.expect("acquire");
        let _ = sqlx::query("SELECT set_config('app.tenant_id', $1, false)")
            .bind(tenant.to_string())
            .execute(&mut *conn)
            .await
            .expect("set tenant");
        let _ = sqlx::query("SELECT set_config('app.role', $1, false)")
            .bind("superuser")
            .execute(&mut *conn)
            .await
            .expect("set role");
        sqlx::query(
            r#"
            INSERT INTO permission.project_role_binding
                (id, tenant_id, project_id, user_id, role, granted_by, version)
            VALUES ($1, $2, $3, $4, $5, $6, 1)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(tenant)
        .bind(project)
        .bind(user)
        .bind(role)
        .bind(user)
        .execute(&mut *conn)
        .await
        .expect("insert role binding");
    }

    async fn seed_device_binding(pool: &sqlx::PgPool, f: &TenantFixture) {
        sqlx::query(
            r#"
            INSERT INTO oauth_device_bindings
                (tenant_id, actor_id, device_id, public_key_fingerprint, generation, status)
            VALUES ($1, $2, $3, $4, $5, 'active')
            "#,
        )
        .bind(f.tenant_id)
        .bind(f.admin_user)
        .bind(f.device_id)
        .bind("a".repeat(64))
        .bind(f.device_generation)
        .execute(pool)
        .await
        .expect("insert device binding");
    }

    async fn seed_worktree(
        pool: &sqlx::PgPool,
        tenant: Uuid,
        project: Uuid,
        worktree_id: Uuid,
        name: &str,
    ) {
        let mut conn = pool.acquire().await.expect("acquire");
        let _ = sqlx::query("SELECT set_config('app.tenant_id', $1, false)")
            .bind(tenant.to_string())
            .execute(&mut *conn)
            .await
            .expect("set tenant");
        let _ = sqlx::query("SELECT set_config('app.role', $1, false)")
            .bind("superuser")
            .execute(&mut *conn)
            .await
            .expect("set role");
        sqlx::query(
            r#"
            INSERT INTO worktree_canvas_worktree
                (id, tenant_id, repo_id, name, branch, path, workspace_id, project_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(worktree_id)
        .bind(tenant)
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(format!("feat/{name}"))
        .bind(format!("/tmp/{name}"))
        .bind(Uuid::new_v4())
        .bind(project)
        .execute(&mut *conn)
        .await
        .expect("insert worktree");
        sqlx::query(
            r#"
            INSERT INTO multica.worktree_project_binding
                (binding_id, tenant_id, project_id, worktree_id, assigned_by, version)
            VALUES ($1, $2, $3, $4, $5, 1)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(tenant)
        .bind(project)
        .bind(worktree_id)
        .bind(tenant)
        .execute(&mut *conn)
        .await
        .expect("insert worktree project binding");
    }

    fn write_token(config: &JwtConfig, f: &TenantFixture, scope: &str) -> String {
        route_test_token(
            config,
            f.admin_user,
            f.tenant_id,
            scope,
            Some(DeviceTokenContext {
                device_id: f.device_id,
                generation: f.device_generation,
            }),
        )
    }

    fn read_token(config: &JwtConfig, tenant: Uuid, user: Uuid, scope: &str) -> String {
        route_test_token(config, user, tenant, scope, None)
    }

    async fn create_lane_via_api(
        app: &Router,
        token: &str,
        project: Uuid,
        name: &str,
    ) -> (StatusCode, serde_json::Value) {
        send(
            app,
            get_with_body(
                &format!("/api/v1/projects/{project}/lanes"),
                token,
                json!({ "name": name }),
            ),
        )
        .await
    }

    #[tokio::test]
    #[ignore = "requires STAR_PG_LANE_TEST_DATABASE_URL (full migration chain + lane migration)"]
    async fn create_lane_without_remote_and_persists_across_router_rebuild() {
        let (jwt, app, pool) = pg_app().await;
        let f = TenantFixture::new();
        seed_role_binding(
            &pool,
            f.tenant_id,
            f.project_id,
            f.admin_user,
            "project_admin",
        )
        .await;
        seed_device_binding(&pool, &f).await;
        let token = write_token(&jwt, &f, "project:write");

        // 验收第一条: body 只有 name — 无任何仓库/分支字段, 创建成功。
        let (status, body) = create_lane_via_api(&app, &token, f.project_id, "core-runtime").await;
        assert_eq!(status, StatusCode::OK, "create lane body: {body}");
        let lane_id: Uuid = body["lane"]["lane_id"]
            .as_str()
            .expect("lane_id")
            .parse()
            .expect("uuid");
        assert_eq!(body["lane"]["name"], json!("core-runtime"));
        assert_eq!(body["lane"]["repository_binding"], json!(null));
        assert_eq!(body["lane"]["branch_binding"], json!(null));
        assert_eq!(body["lane"]["lock_version"], json!(1));

        // 重建 router (同一 pool) → 数据仍在 (真 PG 持久化, 不是 handler 内存态)
        let rebuilt_state = GroupApiState::new(jwt.clone(), pool.clone());
        let rebuilt = build_group_router(rebuilt_state);
        let read = read_token(&jwt, f.tenant_id, f.admin_user, "project:read");
        let (status, list) = send(
            &rebuilt,
            get_uri(&format!("/api/v1/projects/{}/lanes", f.project_id), &read),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "list lanes: {list}");
        let lanes = list["lanes"].as_array().expect("lanes array");
        assert_eq!(lanes.len(), 1);
        assert_eq!(lanes[0]["lane_id"], json!(lane_id));
        assert_eq!(lanes[0]["name"], json!("core-runtime"));
    }

    #[tokio::test]
    #[ignore = "requires STAR_PG_LANE_TEST_DATABASE_URL (full migration chain + lane migration)"]
    async fn non_member_and_cross_tenant_get_403() {
        let (jwt, app, pool) = pg_app().await;
        let f = TenantFixture::new();
        seed_role_binding(
            &pool,
            f.tenant_id,
            f.project_id,
            f.admin_user,
            "project_admin",
        )
        .await;
        seed_device_binding(&pool, &f).await;
        let token = write_token(&jwt, &f, "project:write");
        let (status, body) = create_lane_via_api(&app, &token, f.project_id, "only-members").await;
        assert_eq!(status, StatusCode::OK, "create lane: {body}");
        let lane_id = body["lane"]["lane_id"]
            .as_str()
            .expect("lane_id")
            .to_owned();

        // 非成员 (同租户, 无 binding) → 403
        let outsider = Uuid::new_v4();
        let outsider_token = read_token(&jwt, f.tenant_id, outsider, "project:read");
        let (status, _) = send(
            &app,
            get_uri(
                &format!("/api/v1/projects/{}/lanes", f.project_id),
                &outsider_token,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // 跨租户 (不同 tenant, 无 binding) → 403
        let other_tenant = Uuid::new_v4();
        let cross_token = read_token(&jwt, other_tenant, f.admin_user, "project:read");
        let (status, _) = send(
            &app,
            get_uri(
                &format!("/api/v1/projects/{}/lanes/{}", f.project_id, lane_id),
                &cross_token,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[ignore = "requires STAR_PG_LANE_TEST_DATABASE_URL (full migration chain + lane migration)"]
    async fn revoking_binding_hides_existing_lanes() {
        let (jwt, app, pool) = pg_app().await;
        let f = TenantFixture::new();
        seed_role_binding(
            &pool,
            f.tenant_id,
            f.project_id,
            f.admin_user,
            "project_admin",
        )
        .await;
        seed_device_binding(&pool, &f).await;
        let token = write_token(&jwt, &f, "project:write");
        let (status, body) = create_lane_via_api(&app, &token, f.project_id, "visible-lane").await;
        assert_eq!(status, StatusCode::OK, "create lane: {body}");

        // 撤权: 关闭 actor 的 project_role_binding → list/get 立即 fail-closed 403 (旧 Lane 不可见)
        let mut conn = pool.acquire().await.expect("acquire");
        sqlx::query(
            r#"
            UPDATE permission.project_role_binding
            SET valid_to = now()
            WHERE tenant_id = $1 AND project_id = $2 AND user_id = $3 AND valid_to IS NULL
            "#,
        )
        .bind(f.tenant_id)
        .bind(f.project_id)
        .bind(f.admin_user)
        .execute(&mut *conn)
        .await
        .expect("revoke binding");

        let read = read_token(&jwt, f.tenant_id, f.admin_user, "project:read");
        let (status, _) = send(
            &app,
            get_uri(&format!("/api/v1/projects/{}/lanes", f.project_id), &read),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[ignore = "requires STAR_PG_LANE_TEST_DATABASE_URL (full migration chain + lane migration)"]
    async fn attach_worktree_and_filter_by_lane() {
        let (jwt, app, pool) = pg_app().await;
        let f = TenantFixture::new();
        seed_role_binding(
            &pool,
            f.tenant_id,
            f.project_id,
            f.admin_user,
            "project_admin",
        )
        .await;
        seed_device_binding(&pool, &f).await;
        let token = write_token(&jwt, &f, "project:write");

        let (_, lane_a) = create_lane_via_api(&app, &token, f.project_id, "lane-a").await;
        let (_, lane_b) = create_lane_via_api(&app, &token, f.project_id, "lane-b").await;
        let lane_a_id = lane_a["lane"]["lane_id"]
            .as_str()
            .expect("lane-a")
            .to_owned();
        let lane_b_id = lane_b["lane"]["lane_id"]
            .as_str()
            .expect("lane-b")
            .to_owned();

        let worktree_id = Uuid::new_v4();
        seed_worktree(
            &pool,
            f.tenant_id,
            f.project_id,
            worktree_id,
            "wt-lane-test",
        )
        .await;

        // 挂 lane-a
        let (status, attach) = send(
            &app,
            get_with_body(
                &format!(
                    "/api/v1/projects/{}/lanes/{}/worktrees",
                    f.project_id, lane_a_id
                ),
                &token,
                json!({ "worktree_id": worktree_id }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "attach: {attach}");
        assert_eq!(attach["binding_id"].is_null(), false);

        // lane-a 过滤正确: 含该 worktree, engineering_run_id 诚实 NULL
        let read = read_token(&jwt, f.tenant_id, f.admin_user, "project:read");
        let (status, list_a) = send(
            &app,
            get_uri(
                &format!(
                    "/api/v1/projects/{}/lanes/{}/worktrees",
                    f.project_id, lane_a_id
                ),
                &read,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "list lane-a worktrees: {list_a}");
        let wts = list_a["worktrees"].as_array().expect("worktrees");
        assert_eq!(wts.len(), 1);
        assert_eq!(wts[0]["worktree_id"], json!(worktree_id));
        assert_eq!(
            wts[0]["lane_id"],
            json!(Uuid::parse_str(&lane_a_id).unwrap())
        );
        assert_eq!(wts[0]["engineering_run_id"], json!(null));

        // lane-b 为空 (挂 lane 后按 lane 过滤正确)
        let (status, list_b) = send(
            &app,
            get_uri(
                &format!(
                    "/api/v1/projects/{}/lanes/{}/worktrees",
                    f.project_id, lane_b_id
                ),
                &read,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "list lane-b worktrees: {list_b}");
        assert_eq!(list_b["worktrees"].as_array().expect("array").len(), 0);
    }

    #[tokio::test]
    #[ignore = "requires STAR_PG_LANE_TEST_DATABASE_URL (full migration chain + lane migration)"]
    async fn update_lane_optimistic_lock_and_delete() {
        let (jwt, app, pool) = pg_app().await;
        let f = TenantFixture::new();
        seed_role_binding(
            &pool,
            f.tenant_id,
            f.project_id,
            f.admin_user,
            "project_admin",
        )
        .await;
        seed_device_binding(&pool, &f).await;
        let token = write_token(&jwt, &f, "project:write");
        let (status, body) = create_lane_via_api(&app, &token, f.project_id, "rename-me").await;
        assert_eq!(status, StatusCode::OK, "create: {body}");
        let lane_id = body["lane"]["lane_id"]
            .as_str()
            .expect("lane_id")
            .to_owned();

        let patch = |expected: i32, name: &str| {
            Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/projects/{}/lanes/{}",
                    f.project_id, lane_id
                ))
                .header("authorization", format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "name": name, "expected_version": expected }).to_string(),
                ))
                .unwrap()
        };

        // 版本冲突 → 409
        let (status, _) = send(&app, patch(99, "wrong-version")).await;
        assert_eq!(status, StatusCode::CONFLICT);

        // 正确版本 → 200, lock_version +1
        let (status, updated) = send(&app, patch(1, "renamed")).await;
        assert_eq!(status, StatusCode::OK, "patch: {updated}");
        assert_eq!(updated["lane"]["name"], json!("renamed"));
        assert_eq!(updated["lane"]["lock_version"], json!(2));

        // 删除 → 200; 再 GET → 404
        let del = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/projects/{}/lanes/{}",
                f.project_id, lane_id
            ))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let (status, deleted) = send(&app, del).await;
        assert_eq!(status, StatusCode::OK, "delete: {deleted}");
        assert_eq!(deleted["deleted"], json!(true));

        let read = read_token(&jwt, f.tenant_id, f.admin_user, "project:read");
        let (status, _) = send(
            &app,
            get_uri(
                &format!("/api/v1/projects/{}/lanes/{}", f.project_id, lane_id),
                &read,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore = "requires STAR_PG_LANE_TEST_DATABASE_URL (full migration chain + lane migration)"]
    async fn developer_role_cannot_create_lane() {
        let (jwt, app, pool) = pg_app().await;
        let f = TenantFixture::new();
        // 成员但非管理员
        seed_role_binding(&pool, f.tenant_id, f.project_id, f.admin_user, "developer").await;
        seed_device_binding(&pool, &f).await;
        let token = write_token(&jwt, &f, "project:write");
        let (status, body) = create_lane_via_api(&app, &token, f.project_id, "denied").await;
        assert_eq!(status, StatusCode::FORBIDDEN, "create by developer: {body}");

        // 成员可读 (列表为空, 不 403)
        let read = read_token(&jwt, f.tenant_id, f.admin_user, "project:read");
        let (status, list) = send(
            &app,
            get_uri(&format!("/api/v1/projects/{}/lanes", f.project_id), &read),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "list: {list}");
        assert_eq!(list["lanes"].as_array().expect("array").len(), 0);
    }
}
