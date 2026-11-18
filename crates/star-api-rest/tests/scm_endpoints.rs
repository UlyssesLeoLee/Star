// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=8734b8676691909226445f1f76cdaa9a8514beb73fe8c5f3c1548fdf817b1001
MERGE (self:File {path:"crates/star-api-rest/tests/scm_endpoints.rs"})
MERGE (harness:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::harness_with_active_device",kind:"function"})
MERGE (t1:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_routes_require_bearer_token",kind:"function"})
MERGE (t2:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_post_without_device_context_is_401",kind:"function"})
MERGE (t3:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_post_with_revoked_device_generation_is_401",kind:"function"})
MERGE (t4:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_cross_handler_chain_shares_one_service",kind:"function"})
MERGE (t5:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_list_visibility_is_project_scoped",kind:"function"})
MERGE (t6:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_decision_only_reviewer_self_and_no_repeat",kind:"function"})
MERGE (t7:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_pipeline_not_found_and_cross_tenant",kind:"function"})
MERGE (t8:Symbol {id:"crates/star-api-rest/tests/scm_endpoints.rs::scm_create_mr_validation_errors",kind:"function"})
MERGE (scmRouter:Symbol {id:"crates/star-api-rest/src/lib.rs::build_scm_router",kind:"function"})
MERGE (scmState:Type {id:"crates/star-api-rest/src/routes/scm_state.rs::ScmApiState",kind:"struct"})
MERGE (scmStateNew:Symbol {id:"crates/star-api-rest/src/routes/scm_state.rs::ScmApiState.new",kind:"method"})
MERGE (issueProjects:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token_with_project_memberships",kind:"function"})
MERGE (self)-[:DEFINES]->(harness)
MERGE (self)-[:DEFINES]->(t1)
MERGE (self)-[:DEFINES]->(t2)
MERGE (self)-[:DEFINES]->(t3)
MERGE (self)-[:DEFINES]->(t4)
MERGE (self)-[:DEFINES]->(t5)
MERGE (self)-[:DEFINES]->(t6)
MERGE (self)-[:DEFINES]->(t7)
MERGE (self)-[:DEFINES]->(t8)
MERGE (harness)-[:CALLS]->(scmRouter)
MERGE (harness)-[:CALLS]->(scmStateNew)
MERGE (harness)-[:USES_TYPE]->(scmState)
MERGE (t1)-[:TESTS]->(scmRouter)
MERGE (t2)-[:TESTS]->(scmRouter)
MERGE (t3)-[:TESTS]->(scmRouter)
MERGE (t4)-[:TESTS]->(scmRouter)
MERGE (t5)-[:TESTS]->(scmRouter)
MERGE (t6)-[:TESTS]->(scmRouter)
MERGE (t7)-[:TESTS]->(scmRouter)
MERGE (t8)-[:TESTS]->(scmRouter)
MERGE (t1)-[:CALLS]->(issueProjects)
MERGE (t4)-[:CALLS]->(issueProjects)
MERGE (t5)-[:CALLS]->(issueProjects)
MERGE (t6)-[:CALLS]->(issueProjects)
@endcypher
*/
//! SCM REST 生产路由集成测试 (2026-11-19 批次 B "有名无实"治理)。
//!
//! 守门 #25 v25: 不发真 HTTP — 全部走 axum::Router::oneshot; 不发真 PG —
//! 设备绑定走本文件内 mock (is_active 按注入的 active 集合裁决)。
//!
//! 覆盖 (per F06 §6 + INV-SCM-04 / INV-SCM-07):
//! - 无 Authorization → 401; 垃圾 token → 401; POST 无设备上下文 → 401
//! - 跨 handler 可见性: create_mr → request_review → list → decide →
//!   transition → register_pipeline → get_status 全链路同一共享 service
//! - INV-SCM-04: 无 project 成员资格的 developer 列表为空 / 带 repo 过滤 422;
//!   tenant_admin 可见整租户
//! - 裁决仅 reviewer 本人 (他人 403); 重复裁决 400; 非法裁决状态 400
//! - INV-SCM-07: Draft→Merged 非法 400; Draft→Open 合法; 未知状态 400
//! - 跨 tenant pipeline 查询 403; 不存在 pipeline 422

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::{DateTime, Utc};
use domain_scm::{
    ActorContext, ConflictStrategy, InMemoryScmService, Pipeline, PipelineId, PipelineStatus,
    ProjectId, PullRequestId, RegisterRepositoryCommand, RepositoryOwnership, ScmCommandPort,
    ScmProvider, TenantId,
};
use serde_json::{json, Value};
use star_api_rest::auth::{DeviceTokenContext, JwtConfig};
use star_api_rest::build_scm_router;
use star_api_rest::routes::scm_state::ScmApiState;
use star_pg_adapter::repository::{
    DeviceBinding, DeviceBindingRepository, DeviceChallengeCreateResult, DeviceEnrollmentChallenge,
};
use star_pg_adapter::PgAdapterError;
use tower::ServiceExt;
use uuid::Uuid;

// ---------------------------------------------------------------------
// 设备绑定 mock: is_active 由测试注入的 (device_id → generation) 决定
// ---------------------------------------------------------------------

struct ScmTestDeviceRepo {
    active: Vec<(Uuid, i64)>,
}

#[async_trait]
impl DeviceBindingRepository for ScmTestDeviceRepo {
    async fn create_challenge(
        &self,
        _challenge: &DeviceEnrollmentChallenge,
    ) -> Result<DeviceChallengeCreateResult, PgAdapterError> {
        unimplemented!("not exercised by SCM route tests")
    }

    async fn find_challenge(
        &self,
        _tenant_id: Uuid,
        _actor_id: Uuid,
        _challenge_id: Uuid,
    ) -> Result<Option<DeviceEnrollmentChallenge>, PgAdapterError> {
        unimplemented!("not exercised by SCM route tests")
    }

    async fn complete_challenge(
        &self,
        _tenant_id: Uuid,
        _actor_id: Uuid,
        _challenge_id: Uuid,
        _challenge_hash: &[u8],
        _device_id: Uuid,
        _now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError> {
        unimplemented!("not exercised by SCM route tests")
    }

    async fn is_active(
        &self,
        _tenant_id: Uuid,
        _actor_id: Uuid,
        device_id: Uuid,
        generation: i64,
    ) -> Result<bool, PgAdapterError> {
        Ok(self
            .active
            .iter()
            .any(|(id, gen)| *id == device_id && *gen == generation))
    }

    async fn revoke(
        &self,
        _tenant_id: Uuid,
        _actor_id: Uuid,
        _device_id: Uuid,
        _now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError> {
        unimplemented!("not exercised by SCM route tests")
    }
}

fn jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

struct Harness {
    app: Router,
    service: Arc<InMemoryScmService>,
    jwt: Arc<JwtConfig>,
    tenant: Uuid,
    project: Uuid,
}

impl Harness {
    fn new() -> Self {
        let service = InMemoryScmService::new_for_test();
        let jwt = jwt_config();
        let tenant = Uuid::new_v4();
        let project = Uuid::new_v4();
        let state = ScmApiState::new(
            Arc::clone(&service),
            Arc::clone(&jwt),
            Arc::new(ScmTestDeviceRepo { active: Vec::new() }),
        );
        Self {
            app: build_scm_router(state),
            service,
            jwt,
            tenant,
            project,
        }
    }

    fn token_with_device(
        &self,
        user: Uuid,
        roles: Vec<&str>,
        projects: Vec<Uuid>,
        device: Option<(Uuid, i64)>,
    ) -> String {
        star_api_rest::auth::issue_token_with_project_memberships(
            &self.jwt,
            user,
            self.tenant,
            roles.into_iter().map(str::to_owned).collect(),
            "read write".to_owned(),
            device.map(|(device_id, generation)| DeviceTokenContext {
                device_id,
                generation,
            }),
            projects,
        )
        .expect("fixed test-only RSA key should sign SCM route test tokens")
        .0
    }

    /// 预注册仓库 (project_admin 身份, 直连 service, 不经 HTTP)
    async fn seed_repo(&self, project: Uuid) -> Uuid {
        let admin = ActorContext::new(Uuid::new_v4(), self.tenant)
            .with_role("project_admin")
            .with_project(project);
        let cmd = RegisterRepositoryCommand {
            tenant_id: TenantId::from(self.tenant),
            project_id: ProjectId::from_uuid(project),
            provider: ScmProvider::Github,
            external_id: format!("acme/repo-{}", Uuid::new_v4()),
            url: "https://github.com/acme/repo".to_owned(),
            default_branch: "main".to_owned(),
            ownership: RepositoryOwnership::Connected,
            conflict_strategy: ConflictStrategy::LatestWins,
            credential_id: Some(Uuid::new_v4()),
        };
        self.service
            .register_repository(cmd, admin)
            .await
            .expect("seed repository")
            .id
            .as_uuid()
    }
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 256 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).expect("json body");
    (status, body)
}

fn post_json(path: &str, token: Option<&str>, payload: Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap()
}

fn get_req(path: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri(path);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder.body(Body::empty()).unwrap()
}

/// 构造带活跃设备的 harness + token (设备代际经 mock repo 核实)
fn harness_with_active_device() -> (Harness, Uuid, String) {
    let mut h = Harness::new();
    let user = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let state = ScmApiState::new(
        Arc::clone(&h.service),
        Arc::clone(&h.jwt),
        Arc::new(ScmTestDeviceRepo {
            active: vec![(device_id, 1)],
        }),
    );
    h.app = build_scm_router(state);
    let token = h.token_with_device(
        user,
        vec!["developer"],
        vec![h.project],
        Some((device_id, 1)),
    );
    (h, user, token)
}

// ---------------------------------------------------------------------
// 鉴权门禁
// ---------------------------------------------------------------------

#[tokio::test]
async fn scm_routes_require_bearer_token() {
    let h = Harness::new();
    let (status, _) = send(&h.app, get_req("/api/v1/merge-requests", None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            None,
            json!({"title": "t", "head": "f", "base": "main", "repository_id": Uuid::new_v4().to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = send(
        &h.app,
        get_req("/api/v1/merge-requests", Some("garbage-token")),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn scm_post_without_device_context_is_401() {
    let h = Harness::new();
    let repo = h.seed_repo(h.project).await;
    // GET 不需要设备上下文; POST 共享变更必须 (per requires_active_device_context)
    let token = h.token_with_device(Uuid::new_v4(), vec!["developer"], vec![h.project], None);
    let (status, _) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&token),
            json!({"title": "t", "head": "f", "base": "main", "repository_id": repo.to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn scm_post_with_revoked_device_generation_is_401() {
    let h = Harness::new();
    let repo = h.seed_repo(h.project).await;
    // token 声称 generation=2, 但 mock repo 无此活跃绑定 → 401
    let token = h.token_with_device(
        Uuid::new_v4(),
        vec!["developer"],
        vec![h.project],
        Some((Uuid::new_v4(), 2)),
    );
    let (status, _) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&token),
            json!({"title": "t", "head": "f", "base": "main", "repository_id": repo.to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------
// 跨 handler 全链路: 单一共享 service (根因修复的核心证据)
// ---------------------------------------------------------------------

#[tokio::test]
async fn scm_cross_handler_chain_shares_one_service() {
    let (h, author, author_token) = harness_with_active_device();
    let repo = h.seed_repo(h.project).await;
    let reviewer = Uuid::new_v4();

    // 1. author 创建 MR
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&author_token),
            json!({
                "title": "feat: real scm",
                "head": "feature/x",
                "base": "main",
                "repository_id": repo.to_string(),
                "description": "batch B"
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create mr: {body}");
    let mr_id = body["data"]["mr"]["id"].as_str().unwrap().to_owned();
    assert_eq!(body["data"]["mr"]["state"], "DRAFT");
    assert_eq!(body["data"]["mr"]["author_user_id"], author.to_string());

    // 2. author 请求评审 (reviewer 是另一个真实 user) — 旧实现此处必 NotFound
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/reviews",
            Some(&author_token),
            json!({"mr_id": mr_id, "reviewers": [reviewer.to_string()]}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "request review: {body}");
    let review_id = body["data"]["review"]["reviews"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(body["data"]["review"]["reviews"][0]["state"], "PENDING");

    // 3. reviewer 用自己的 token 列该 PR 的 review (GET 不需设备上下文)
    let reviewer_token = h.token_with_device(reviewer, vec!["developer"], vec![h.project], None);
    let (status, body) = send(
        &h.app,
        get_req(
            &format!("/api/v1/reviews?mr_id={mr_id}"),
            Some(&reviewer_token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list reviews: {body}");
    assert_eq!(body["data"]["reviews"].as_array().unwrap().len(), 1);

    // 4. reviewer 本人裁决 Approved (POST 需活跃设备)
    let reviewer_device = Uuid::new_v4();
    let state = ScmApiState::new(
        Arc::clone(&h.service),
        Arc::clone(&h.jwt),
        Arc::new(ScmTestDeviceRepo {
            active: vec![(reviewer_device, 1)],
        }),
    );
    let reviewer_app = build_scm_router(state);
    let reviewer_post_token = h.token_with_device(
        reviewer,
        vec!["developer"],
        vec![h.project],
        Some((reviewer_device, 1)),
    );
    let (status, body) = send(
        &reviewer_app,
        post_json(
            &format!("/api/v1/reviews/{review_id}/decision"),
            Some(&reviewer_post_token),
            json!({"state": "Approved", "body": "LGTM"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "decide: {body}");
    assert_eq!(body["data"]["review"]["state"], "APPROVED");
    assert_eq!(body["data"]["review"]["body"], "LGTM");

    // 5. author 列表可见自己的 MR (INV-SCM-04 project 成员资格)
    let (status, body) = send(
        &h.app,
        get_req("/api/v1/merge-requests", Some(&author_token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list mrs: {body}");
    let items = body["data"]["merge_requests"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], mr_id.as_str());

    // 6. 状态机: Draft → Open 合法; Draft → Merged 非法 (INV-SCM-07)
    let (status, body) = send(
        &h.app,
        post_json(
            &format!("/api/v1/merge-requests/{mr_id}/transition"),
            Some(&author_token),
            json!({"to_state": "MERGED"}),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "illegal transition: {body}"
    );
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");

    let (status, body) = send(
        &h.app,
        post_json(
            &format!("/api/v1/merge-requests/{mr_id}/transition"),
            Some(&author_token),
            json!({"to_state": "OPEN"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "legal transition: {body}");
    assert_eq!(body["data"]["mr"]["state"], "OPEN");

    // 7. pipeline: 直连 service 注册 (webhook 侧职责), REST 按 external_id 可查
    let pipeline = Pipeline {
        id: PipelineId::new(),
        tenant_id: TenantId::from(h.tenant),
        pull_request_id: PullRequestId::from_uuid(Uuid::parse_str(&mr_id).unwrap()),
        external_id: "PIPE-TEST-1".to_owned(),
        status: PipelineStatus::Running,
        head_sha: "abc123".to_owned(),
        url: Some("https://ci.example/1".to_owned()),
        started_at: Some(Utc::now()),
        finished_at: None,
        created_at: Utc::now(),
        lock_version: 1,
    };
    let seed_actor = ActorContext::new(author, h.tenant).with_role("developer");
    h.service
        .register_pipeline(pipeline, seed_actor)
        .await
        .expect("seed pipeline");
    let (status, body) = send(
        &h.app,
        get_req("/api/v1/pipelines/PIPE-TEST-1", Some(&author_token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "pipeline status: {body}");
    assert_eq!(body["data"]["pipeline"]["status"], "RUNNING");
    assert_eq!(body["data"]["pipeline"]["head_sha"], "abc123");
}

// ---------------------------------------------------------------------
// INV-SCM-04 可见性 + 裁决权限 + 错误映射
// ---------------------------------------------------------------------

#[tokio::test]
async fn scm_list_visibility_is_project_scoped() {
    let (h, _author, author_token) = harness_with_active_device();
    let repo = h.seed_repo(h.project).await;
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&author_token),
            json!({"title": "t", "head": "f", "base": "main", "repository_id": repo.to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // 无 project 成员资格的 developer: 列表为空, 带 repo 过滤 → 422 SCM_NOT_FOUND
    let outsider = h.token_with_device(Uuid::new_v4(), vec!["developer"], vec![], None);
    let (status, body) = send(&h.app, get_req("/api/v1/merge-requests", Some(&outsider))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["merge_requests"].as_array().unwrap().len(), 0);

    let (status, body) = send(
        &h.app,
        get_req(
            &format!("/api/v1/merge-requests?repository_id={repo}"),
            Some(&outsider),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "SCM_NOT_FOUND");

    // tenant_admin 可见整租户 (无需 project 成员资格)
    let admin = h.token_with_device(Uuid::new_v4(), vec!["tenant_admin"], vec![], None);
    let (status, body) = send(&h.app, get_req("/api/v1/merge-requests", Some(&admin))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["merge_requests"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn scm_decision_only_reviewer_self_and_no_repeat() {
    let (h, author, author_token) = harness_with_active_device();
    let repo = h.seed_repo(h.project).await;
    let reviewer = Uuid::new_v4();
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&author_token),
            json!({"title": "t", "head": "f", "base": "main", "repository_id": repo.to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let mr_id = body["data"]["mr"]["id"].as_str().unwrap().to_owned();
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/reviews",
            Some(&author_token),
            json!({"mr_id": mr_id, "reviewers": [reviewer.to_string()]}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let review_id = body["data"]["review"]["reviews"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    // 非 reviewer (author 本人) 裁决 → 403 POLICY_DENIED
    let (status, body) = send(
        &h.app,
        post_json(
            &format!("/api/v1/reviews/{review_id}/decision"),
            Some(&author_token),
            json!({"state": "Approved"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "POLICY_DENIED");

    // reviewer 本人裁决成功
    let reviewer_device = Uuid::new_v4();
    let state = ScmApiState::new(
        Arc::clone(&h.service),
        Arc::clone(&h.jwt),
        Arc::new(ScmTestDeviceRepo {
            active: vec![(reviewer_device, 1)],
        }),
    );
    let reviewer_app = build_scm_router(state);
    let reviewer_token = h.token_with_device(
        reviewer,
        vec!["developer"],
        vec![h.project],
        Some((reviewer_device, 1)),
    );
    let (status, body) = send(
        &reviewer_app,
        post_json(
            &format!("/api/v1/reviews/{review_id}/decision"),
            Some(&reviewer_token),
            json!({"state": "ChangesRequested", "body": "fix tests"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["review"]["state"], "CHANGES_REQUESTED");

    // 重复裁决 → 400 VALIDATION_FAILED (domain InvalidState)
    let (status, body) = send(
        &reviewer_app,
        post_json(
            &format!("/api/v1/reviews/{review_id}/decision"),
            Some(&reviewer_token),
            json!({"state": "Approved"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");

    // 非法裁决状态 → 400 (REST 层白名单)
    let (status, _) = send(
        &reviewer_app,
        post_json(
            &format!("/api/v1/reviews/{review_id}/decision"),
            Some(&reviewer_token),
            json!({"state": "Merged"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let _ = author;
}

#[tokio::test]
async fn scm_pipeline_not_found_and_cross_tenant() {
    let (h, _author, author_token) = harness_with_active_device();
    // 不存在 → 422 SCM_NOT_FOUND
    let (status, body) = send(
        &h.app,
        get_req("/api/v1/pipelines/PIPE-MISSING", Some(&author_token)),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "SCM_NOT_FOUND");

    // 跨 tenant 注册 → 403 POLICY_DENIED
    let other_tenant = Uuid::new_v4();
    let pipeline = Pipeline {
        id: PipelineId::new(),
        tenant_id: TenantId::from(other_tenant),
        pull_request_id: PullRequestId::new(),
        external_id: "PIPE-OTHER".to_owned(),
        status: PipelineStatus::Success,
        head_sha: "def456".to_owned(),
        url: None,
        started_at: None,
        finished_at: None,
        created_at: Utc::now(),
        lock_version: 1,
    };
    let seed_actor = ActorContext::new(Uuid::new_v4(), other_tenant).with_role("developer");
    h.service
        .register_pipeline(pipeline, seed_actor)
        .await
        .expect("seed cross-tenant pipeline");
    let (status, body) = send(
        &h.app,
        get_req("/api/v1/pipelines/PIPE-OTHER", Some(&author_token)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error"]["code"], "POLICY_DENIED");
}

#[tokio::test]
async fn scm_create_mr_validation_errors() {
    let (h, _author, author_token) = harness_with_active_device();
    // 缺 title → 400
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&author_token),
            json!({"title": "", "head": "f", "base": "main", "repository_id": Uuid::new_v4().to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_FAILED");

    // 非法 UUID → 400
    let (status, _) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&author_token),
            json!({"title": "t", "head": "f", "base": "main", "repository_id": "not-a-uuid"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 不存在的仓库 → 422 SCM_NOT_FOUND
    let (status, body) = send(
        &h.app,
        post_json(
            "/api/v1/merge-requests",
            Some(&author_token),
            json!({"title": "t", "head": "f", "base": "main", "repository_id": Uuid::new_v4().to_string()}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "SCM_NOT_FOUND");
}
