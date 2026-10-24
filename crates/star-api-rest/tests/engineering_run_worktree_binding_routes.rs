/*
@cypher schema=1 source_sha256=cb62de59a6a930dd641ce5604a6449ebdc89534a05aa67f99d2c863727859bae
MERGE (self:File {path:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs"})
MERGE (test:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::run_worktree_binding_mutations_require_authentication",kind:"test"})
MERGE (scopeTest:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::run_worktree_binding_mutations_require_both_scopes_before_database_access",kind:"test"})
MERGE (httpAcceptance:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::real_postgres_authenticated_binding_http_acceptance",kind:"test"})
MERGE (testJwt:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::route_test_jwt_config",kind:"function"})
MERGE (testToken:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::route_test_token",kind:"function"})
MERGE (tokenForActor:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::route_test_token_for_actor",kind:"function"})
MERGE (sendMutation:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::send_binding_mutation",kind:"function"})
MERGE (assertPrivate:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::assert_private_response",kind:"function"})
MERGE (mutationBody:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::mutation_body",kind:"function"})
MERGE (currentBinding:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::current_binding",kind:"function"})
MERGE (ownerScopeProbe:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::owner_scope_probe",kind:"function"})
MERGE (issueToken:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token",kind:"function"})
MERGE (databaseUrl:Config {id:"STAR_BINDING_HTTP_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (phase:Config {id:"STAR_BINDING_HTTP_TEST_PHASE",kind:"environment_variable"})
MERGE (jwtPrivate:File {path:"crates/star-api-rest/tests/fixtures/schedule-route-test-private.pem"})
MERGE (jwtPublic:File {path:"crates/star-api-rest/tests/fixtures/schedule-route-test-public.pem"})
MERGE (putEndpoint:Endpoint {id:"PUT /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (deleteEndpoint:Endpoint {id:"DELETE /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (runCreateEndpoint:Endpoint {id:"POST /api/v1/projects/{project_id}/engineering-runs",kind:"http_route"})
MERGE (runListEndpoint:Endpoint {id:"GET /api/v1/projects/{project_id}/engineering-runs",kind:"http_route"})
MERGE (runContextEndpoint:Endpoint {id:"GET /api/v1/engineering-runs/{engineering_run_id}/context",kind:"http_route"})
MERGE (runDirectoryRequest:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::send_project_run_request",kind:"function"})
MERGE (assertDirectory:Symbol {id:"crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs::assert_directory_response",kind:"function"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(scopeTest)
MERGE (self)-[:DEFINES]->(httpAcceptance)
MERGE (self)-[:DEFINES]->(testJwt)
MERGE (self)-[:DEFINES]->(testToken)
MERGE (self)-[:DEFINES]->(tokenForActor)
MERGE (self)-[:DEFINES]->(sendMutation)
MERGE (self)-[:DEFINES]->(assertPrivate)
MERGE (self)-[:DEFINES]->(mutationBody)
MERGE (self)-[:DEFINES]->(currentBinding)
MERGE (self)-[:DEFINES]->(ownerScopeProbe)
MERGE (self)-[:DEFINES]->(runDirectoryRequest)
MERGE (self)-[:DEFINES]->(assertDirectory)
MERGE (test)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (scopeTest)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (scopeTest)-[:CALLS]->(testJwt)
MERGE (scopeTest)-[:CALLS]->(testToken)
MERGE (httpAcceptance)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
MERGE (httpAcceptance)-[:CALLS]->(testJwt)
MERGE (httpAcceptance)-[:CALLS]->(tokenForActor)
MERGE (httpAcceptance)-[:CALLS]->(sendMutation)
MERGE (httpAcceptance)-[:CALLS]->(assertPrivate)
MERGE (httpAcceptance)-[:CALLS]->(mutationBody)
MERGE (httpAcceptance)-[:CALLS]->(currentBinding)
MERGE (httpAcceptance)-[:CALLS]->(ownerScopeProbe)
MERGE (httpAcceptance)-[:CALLS]->(runDirectoryRequest)
MERGE (httpAcceptance)-[:CALLS]->(assertDirectory)
MERGE (runDirectoryRequest)-[:CALLS]->(tokenForActor)
MERGE (sendMutation)-[:CALLS]->(tokenForActor)
MERGE (testToken)-[:CALLS]->(issueToken)
MERGE (tokenForActor)-[:CALLS]->(issueToken)
MERGE (testJwt)-[:READS]->(jwtPrivate)
MERGE (testJwt)-[:READS]->(jwtPublic)
MERGE (httpAcceptance)-[:READS]->(databaseUrl)
MERGE (httpAcceptance)-[:READS]->(phase)
MERGE (currentBinding)-[:READS]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
MERGE (ownerScopeProbe)-[:READS]->(:Table {id:"permission.project_role_binding",kind:"table"})
MERGE (ownerScopeProbe)-[:READS]->(:Table {id:"multica.engineering_run",kind:"table"})
MERGE (ownerScopeProbe)-[:READS]->(:Table {id:"multica.engineering_run_revision",kind:"table"})
MERGE (ownerScopeProbe)-[:READS]->(:Table {id:"permission.engineering_run_role_binding",kind:"table"})
MERGE (ownerScopeProbe)-[:READS]->(:Table {id:"public.worktree_canvas_worktree",kind:"table"})
MERGE (ownerScopeProbe)-[:READS]->(:Table {id:"multica.worktree_project_binding",kind:"table"})
MERGE (test)-[:TESTS]->(putEndpoint)
MERGE (test)-[:TESTS]->(deleteEndpoint)
MERGE (scopeTest)-[:TESTS]->(putEndpoint)
MERGE (scopeTest)-[:TESTS]->(deleteEndpoint)
MERGE (httpAcceptance)-[:TESTS]->(putEndpoint)
MERGE (httpAcceptance)-[:TESTS]->(deleteEndpoint)
MERGE (httpAcceptance)-[:TESTS]->(runCreateEndpoint)
MERGE (httpAcceptance)-[:TESTS]->(runListEndpoint)
MERGE (httpAcceptance)-[:TESTS]->(runContextEndpoint)
@endcypher
*/

//! Route registration and authorization boundary checks for Run-to-Worktree writes.

use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use star_api_rest::{
    auth::{JwtConfig, issue_token},
    group_api::{GroupApiState, build_group_router},
};
use tower::ServiceExt;
use uuid::Uuid;

const ACCEPTANCE_TENANT: &str = "10000000-0000-0000-0000-000000000001";
const ACCEPTANCE_OTHER_TENANT: &str = "10000000-0000-0000-0000-000000000002";
const ACCEPTANCE_PROJECT: &str = "20000000-0000-0000-0000-000000000001";
const ACCEPTANCE_ACTOR: &str = "30000000-0000-0000-0000-000000000001";
const ACCEPTANCE_DEVELOPER: &str = "30000000-0000-0000-0000-000000000002";
const ACCEPTANCE_OTHER_ACTOR: &str = "30000000-0000-0000-0000-000000000003";
const ACCEPTANCE_RUN: &str = "60000000-0000-0000-0000-000000000001";
const ACCEPTANCE_WORKTREE: &str = "90000000-0000-0000-0000-000000000001";
const ACCEPTANCE_BINDING_SCOPE: &str = "project:write worktree:manage";

fn route_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        private_key_pem: include_str!("fixtures/schedule-route-test-private.pem").to_owned(),
        public_key_pem: include_str!("fixtures/schedule-route-test-public.pem").to_owned(),
        issuer: "https://api.star.local".to_owned(),
        audience: "star-api-rest".to_owned(),
        ttl_seconds: 3600,
    })
}

fn route_test_token(config: &JwtConfig, scope: &str) -> String {
    issue_token(
        config,
        Uuid::new_v4(),
        Uuid::new_v4(),
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("fixed test-only RSA key should sign route test tokens")
    .0
}

fn route_test_token_for_actor(
    config: &JwtConfig,
    actor_id: &str,
    tenant_id: &str,
    scope: &str,
) -> String {
    issue_token(
        config,
        Uuid::parse_str(actor_id).expect("acceptance actor UUID is valid"),
        Uuid::parse_str(tenant_id).expect("acceptance tenant UUID is valid"),
        vec!["developer".to_owned()],
        scope.to_owned(),
    )
    .expect("fixed test-only RSA key should sign acceptance tokens")
    .0
}

struct MutationReply {
    status: StatusCode,
    cache_control: String,
    vary: String,
    body: Value,
}

fn mutation_body(expected_binding: Value, project_version: i32, correlation_id: Uuid) -> Value {
    json!({
        "expected_binding": expected_binding,
        "expected_project_binding_version": project_version,
        "correlation_id": correlation_id
    })
}

async fn send_binding_mutation(
    app: Router,
    config: &JwtConfig,
    actor_id: &str,
    tenant_id: &str,
    method: Method,
    idempotency_key: &str,
    body: &Value,
) -> MutationReply {
    let token = route_test_token_for_actor(config, actor_id, tenant_id, ACCEPTANCE_BINDING_SCOPE);
    let path = format!(
        "/api/v1/projects/{ACCEPTANCE_PROJECT}/engineering-runs/{ACCEPTANCE_RUN}/worktrees/{ACCEPTANCE_WORKTREE}"
    );
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("Idempotency-Key", idempotency_key)
                .body(Body::from(body.to_string()))
                .expect("acceptance request should build"),
        )
        .await
        .expect("router should return an HTTP response");
    let status = response.status();
    let cache_control = response
        .headers()
        .get(header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let vary = response
        .headers()
        .get(header::VARY)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("bounded HTTP response body should be readable");
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes)}));
    MutationReply {
        status,
        cache_control,
        vary,
        body,
    }
}

async fn send_project_run_request(
    app: Router,
    config: &JwtConfig,
    actor_id: &str,
    tenant_id: &str,
    scope: &str,
    method: Method,
    uri: &str,
    idempotency_key: Option<&str>,
    body: Option<&Value>,
) -> MutationReply {
    let token = route_test_token_for_actor(config, actor_id, tenant_id, scope);
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(key) = idempotency_key {
        request = request.header("Idempotency-Key", key);
    }
    let body = body.map(Value::to_string).unwrap_or_default();
    let response = app
        .oneshot(
            request
                .body(Body::from(body))
                .expect("Run directory request should build"),
        )
        .await
        .expect("router should return a Run directory response");
    let status = response.status();
    let cache_control = response
        .headers()
        .get(header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let vary = response
        .headers()
        .get(header::VARY)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("bounded Run directory response body should be readable");
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes)}));
    MutationReply {
        status,
        cache_control,
        vary,
        body,
    }
}

fn assert_directory_response(reply: &MutationReply) {
    assert_eq!(reply.cache_control, "no-store");
    assert_eq!(reply.vary, "Authorization");
}

fn assert_private_response(reply: &MutationReply) {
    assert_eq!(reply.cache_control, "private, no-store");
    assert_eq!(reply.vary, "Authorization");
}

async fn current_binding(pool: &sqlx::PgPool) -> (Uuid, i32) {
    let mut tx = pool
        .begin()
        .await
        .expect("runtime database transaction should start");
    sqlx::query("SET LOCAL ROLE star_engineering_directory_owner")
        .execute(&mut *tx)
        .await
        .expect("runtime may select the narrow owner role");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true),set_config('app.actor_id',$2,true)")
        .bind(ACCEPTANCE_TENANT)
        .bind(ACCEPTANCE_ACTOR)
        .execute(&mut *tx)
        .await
        .expect("RLS identity should be transaction local");
    let row = sqlx::query_as::<_, (Uuid, i32)>(
        "SELECT binding_id,version FROM multica.engineering_run_worktree_binding \
         WHERE tenant_id=$1 AND worktree_id=$2 AND valid_to IS NULL",
    )
    .bind(Uuid::parse_str(ACCEPTANCE_TENANT).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_WORKTREE).unwrap())
    .fetch_one(&mut *tx)
    .await
    .expect("HTTP bind should create one current binding");
    tx.commit()
        .await
        .expect("binding read transaction should commit");
    row
}

async fn owner_scope_probe(pool: &sqlx::PgPool) -> Value {
    let mut tx = pool
        .begin()
        .await
        .expect("runtime database diagnostic transaction should start");
    sqlx::query("SET LOCAL ROLE star_engineering_directory_owner")
        .execute(&mut *tx)
        .await
        .expect("runtime may select the narrow owner role for diagnostics");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(ACCEPTANCE_TENANT)
        .execute(&mut *tx)
        .await
        .expect("diagnostic tenant should be transaction local");
    let project_grant_for_share: Option<String> = sqlx::query_scalar(
        "SELECT role FROM permission.project_role_binding
         WHERE tenant_id=$1 AND project_id=$2 AND user_id=$3
           AND valid_from<=now() AND valid_to IS NULL FOR SHARE",
    )
    .bind(Uuid::parse_str(ACCEPTANCE_TENANT).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_PROJECT).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_ACTOR).unwrap())
    .fetch_optional(&mut *tx)
    .await
    .expect("Project grant FOR SHARE probe should match active_binding");
    sqlx::query("SELECT set_config('app.actor_id',$1,true)")
        .bind(ACCEPTANCE_ACTOR)
        .execute(&mut *tx)
        .await
        .expect("diagnostic actor should be transaction local");
    let row: String = sqlx::query_scalar(
        "SELECT json_build_object(\
           'project_grants',(SELECT json_agg(json_build_object('role',role,'project_id',project_id))\
             FROM permission.project_role_binding WHERE tenant_id=$1 AND project_id=$2 AND user_id=$3 AND valid_to IS NULL),\
           'runs',(SELECT json_agg(json_build_object('project_id',project_id,'repository_id',repository_id,'branch_id',branch_id))\
             FROM multica.engineering_run WHERE tenant_id=$1 AND engineering_run_id=$4),\
           'run_revisions',(SELECT json_agg(json_build_object('state',state,'valid_to',valid_to))\
             FROM multica.engineering_run_revision WHERE tenant_id=$1 AND engineering_run_id=$4),\
           'run_grants',(SELECT json_agg(json_build_object('role',role))\
             FROM permission.engineering_run_role_binding WHERE tenant_id=$1 AND engineering_run_id=$4 AND user_id=$3 AND valid_to IS NULL),\
           'worktrees',(SELECT json_agg(json_build_object('id',id,'project_id',project_id,'repo_id',repo_id))\
             FROM worktree_canvas_worktree WHERE tenant_id=$1 AND id=$5),\
           'project_worktree_bindings',(SELECT json_agg(json_build_object('project_id',project_id,'version',version,'valid_to',valid_to))\
             FROM multica.worktree_project_binding WHERE tenant_id=$1 AND worktree_id=$5 AND valid_to IS NULL)\
         )::text",
    )
    .bind(Uuid::parse_str(ACCEPTANCE_TENANT).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_PROJECT).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_ACTOR).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_RUN).unwrap())
    .bind(Uuid::parse_str(ACCEPTANCE_WORKTREE).unwrap())
    .fetch_one(&mut *tx)
    .await
    .expect("owner RLS facts should be readable for a bounded test diagnostic");
    tx.commit()
        .await
        .expect("diagnostic read transaction should commit");
    let mut diagnostic: Value =
        serde_json::from_str(&row).expect("scope diagnostic should be JSON");
    diagnostic["project_grant_for_share"] = json!(project_grant_for_share);
    diagnostic
}

#[tokio::test]
async fn run_worktree_binding_mutations_require_authentication() {
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://localhost/unavailable-for-binding-route-test")
        .expect("lazy test pool configuration should parse");
    let app = build_group_router(GroupApiState::new(jwt, pool));
    let path = "/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/worktrees/00000000-0000-0000-0000-000000000003";

    for method in [Method::PUT, Method::DELETE] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .expect("route test request should build"),
            )
            .await
            .expect("router should return a response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "private, no-store"
        );
        assert_eq!(
            response.headers().get(header::VARY).unwrap(),
            "Authorization"
        );
    }
}

#[tokio::test]
async fn run_worktree_binding_mutations_require_both_scopes_before_database_access() {
    let jwt = route_test_jwt_config();
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://localhost/unavailable-for-binding-route-test")
        .expect("lazy test pool configuration should parse");
    let app = build_group_router(GroupApiState::new(jwt.clone(), pool));
    let path = "/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/worktrees/00000000-0000-0000-0000-000000000003";

    for scope in ["worktree:manage", "project:write"] {
        let token = route_test_token(&jwt, scope);
        for method in [Method::PUT, Method::DELETE] {
            let body = format!(
                r#"{{"expected_binding":{{"state":"unassigned"}},"expected_project_binding_version":1,"correlation_id":"{}"}}"#,
                Uuid::new_v4()
            );
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header(header::AUTHORIZATION, format!("Bearer {token}"))
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("Idempotency-Key", Uuid::new_v4().to_string())
                        .body(Body::from(body))
                        .expect("scope test request should build"),
                )
                .await
                .expect("router should return a response");

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL).unwrap(),
                "private, no-store"
            );
            assert_eq!(
                response.headers().get(header::VARY).unwrap(),
                "Authorization"
            );
        }
    }
}

/// Runs only under the isolated PostgreSQL acceptance runner. The application
/// pool connects as a non-superuser runtime login and the production handler
/// selects the owner role inside each mutation transaction.
#[tokio::test]
#[ignore = "requires the isolated phase9f2 authenticated HTTP acceptance runner"]
async fn real_postgres_authenticated_binding_http_acceptance() {
    let database_url = std::env::var("STAR_BINDING_HTTP_TEST_DATABASE_URL")
        .expect("runner must provide the disposable runtime-role database URL");
    let phase = std::env::var("STAR_BINDING_HTTP_TEST_PHASE")
        .expect("runner must select a bounded acceptance phase");
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("runtime role should connect to the disposable PostgreSQL cluster");
    let (session_user, current_user): (String, String) =
        sqlx::query_as("SELECT session_user::text,current_user::text")
            .fetch_one(&pool)
            .await
            .expect("runtime connection identity should be readable");
    assert_eq!(session_user, "erun_directory_api_test");
    assert_eq!(current_user, "erun_directory_api_test");
    let scope_probe = owner_scope_probe(&pool).await;

    let jwt = route_test_jwt_config();
    let app = build_group_router(GroupApiState::new(jwt.clone(), pool.clone()));
    match phase.as_str() {
        "hostless-run" => {
            let collection_path = format!("/api/v1/projects/{ACCEPTANCE_PROJECT}/engineering-runs");
            let create_body = json!({
                "title": "Hostless Run HTTP acceptance",
                "correlation_id": Uuid::new_v4(),
            });
            let create_key = Uuid::new_v4().to_string();
            let created = send_project_run_request(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                "project:write",
                Method::POST,
                &collection_path,
                Some(&create_key),
                Some(&create_body),
            )
            .await;
            assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
            assert_directory_response(&created);
            let created_run = &created.body["engineering_run"];
            assert_eq!(created_run["project_id"], ACCEPTANCE_PROJECT);
            assert_eq!(created_run["repository_id"], Value::Null);
            assert_eq!(created_run["branch_id"], Value::Null);
            let run_id = created_run["engineering_run_id"]
                .as_str()
                .expect("created Run id should be a UUID string")
                .to_owned();

            let replay = send_project_run_request(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                "project:write",
                Method::POST,
                &collection_path,
                Some(&create_key),
                Some(&create_body),
            )
            .await;
            assert_eq!(replay.status, StatusCode::OK, "{:?}", replay.body);
            assert_directory_response(&replay);
            assert_eq!(
                replay.body, created.body,
                "exact create replay returns its receipt"
            );

            let listed = send_project_run_request(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                "project:read",
                Method::GET,
                &collection_path,
                None,
                None,
            )
            .await;
            assert_eq!(listed.status, StatusCode::OK, "{:?}", listed.body);
            assert_directory_response(&listed);
            let rows = listed.body["engineering_runs"]
                .as_array()
                .expect("Run directory should return an array");
            let listed_run = rows
                .iter()
                .find(|row| row["engineering_run_id"] == run_id)
                .expect("new hostless Run should appear in its Project directory");
            assert_eq!(listed_run["repository_id"], Value::Null);
            assert_eq!(listed_run["branch_id"], Value::Null);

            let context_path = format!("/api/v1/engineering-runs/{run_id}/context");
            let opened = send_project_run_request(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                "project:read",
                Method::GET,
                &context_path,
                None,
                None,
            )
            .await;
            assert_eq!(opened.status, StatusCode::OK, "{:?}", opened.body);
            assert_directory_response(&opened);
            assert_eq!(opened.body["run_context"]["project_id"], ACCEPTANCE_PROJECT);
            assert_eq!(opened.body["run_context"]["engineering_run_id"], run_id);
            assert_eq!(opened.body["run_context"]["repository_id"], Value::Null);
            assert_eq!(opened.body["run_context"]["branch_id"], Value::Null);
            assert_eq!(
                opened.body["run_context"]["authorization"]["branch"],
                Value::Null
            );
            assert_eq!(opened.body["branch"], Value::Null);
            assert_eq!(opened.body["focus"], Value::Null);
            assert_eq!(opened.body["focus_version"], Value::Null);
            assert_eq!(
                opened.body["capabilities"]["execution_admission_available"],
                false
            );

            let member_list = send_project_run_request(
                app.clone(),
                &jwt,
                ACCEPTANCE_DEVELOPER,
                ACCEPTANCE_TENANT,
                "project:read",
                Method::GET,
                &collection_path,
                None,
                None,
            )
            .await;
            assert_eq!(member_list.status, StatusCode::OK, "{:?}", member_list.body);
            assert_directory_response(&member_list);
            assert!(
                member_list.body["engineering_runs"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );

            let member_open = send_project_run_request(
                app,
                &jwt,
                ACCEPTANCE_DEVELOPER,
                ACCEPTANCE_TENANT,
                "project:read",
                Method::GET,
                &context_path,
                None,
                None,
            )
            .await;
            assert_eq!(member_open.status, StatusCode::NOT_FOUND);
        }
        "bind-race" => {
            let unassigned = json!({"state":"unassigned"});
            let first_body = mutation_body(unassigned.clone(), 1, Uuid::new_v4());
            let second_body = mutation_body(unassigned.clone(), 1, Uuid::new_v4());
            let first_key = Uuid::new_v4().to_string();
            let second_key = Uuid::new_v4().to_string();
            let (first, second) = tokio::join!(
                send_binding_mutation(
                    app.clone(),
                    &jwt,
                    ACCEPTANCE_ACTOR,
                    ACCEPTANCE_TENANT,
                    Method::PUT,
                    &first_key,
                    &first_body,
                ),
                send_binding_mutation(
                    app.clone(),
                    &jwt,
                    ACCEPTANCE_ACTOR,
                    ACCEPTANCE_TENANT,
                    Method::PUT,
                    &second_key,
                    &second_body,
                ),
            );
            assert_private_response(&first);
            assert_private_response(&second);
            assert_eq!(
                [first.status.as_u16(), second.status.as_u16()]
                    .into_iter()
                    .filter(|status| *status == StatusCode::OK.as_u16())
                    .count(),
                1,
                "one concurrent compare-and-swap writer must win; first={} {:?}; second={} {:?}; authority={:?}",
                first.status,
                first.body,
                second.status,
                second.body,
                scope_probe,
            );
            assert_eq!(
                [first.status.as_u16(), second.status.as_u16()]
                    .into_iter()
                    .filter(|status| *status == StatusCode::CONFLICT.as_u16())
                    .count(),
                1,
                "the stale concurrent writer must conflict"
            );
            let (winner_body, winner_key, winner_reply) = if first.status == StatusCode::OK {
                (&first_body, &first_key, &first)
            } else {
                (&second_body, &second_key, &second)
            };
            let replay = send_binding_mutation(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::PUT,
                winner_key,
                winner_body,
            )
            .await;
            assert_private_response(&replay);
            assert_eq!(replay.status, StatusCode::OK);
            assert_eq!(
                replay.body, winner_reply.body,
                "exact replay returns the saved receipt"
            );

            let changed_body = mutation_body(unassigned.clone(), 1, Uuid::new_v4());
            let reused_key = send_binding_mutation(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::PUT,
                winner_key,
                &changed_body,
            )
            .await;
            assert_private_response(&reused_key);
            assert_eq!(reused_key.status, StatusCode::CONFLICT);

            let (binding_id, version) = current_binding(&pool).await;
            let stale = mutation_body(
                json!({"state":"bound","binding_id":binding_id,"version":version + 1}),
                1,
                Uuid::new_v4(),
            );
            let stale_reply = send_binding_mutation(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::PUT,
                &Uuid::new_v4().to_string(),
                &stale,
            )
            .await;
            assert_private_response(&stale_reply);
            assert_eq!(stale_reply.status, StatusCode::CONFLICT);

            let unauthorized = send_binding_mutation(
                app.clone(),
                &jwt,
                ACCEPTANCE_DEVELOPER,
                ACCEPTANCE_TENANT,
                Method::PUT,
                &Uuid::new_v4().to_string(),
                &mutation_body(unassigned.clone(), 1, Uuid::new_v4()),
            )
            .await;
            assert_private_response(&unauthorized);
            assert_eq!(unauthorized.status, StatusCode::NOT_FOUND);

            let cross_tenant = send_binding_mutation(
                app,
                &jwt,
                ACCEPTANCE_OTHER_ACTOR,
                ACCEPTANCE_OTHER_TENANT,
                Method::PUT,
                &Uuid::new_v4().to_string(),
                &mutation_body(unassigned, 1, Uuid::new_v4()),
            )
            .await;
            assert_private_response(&cross_tenant);
            assert_eq!(cross_tenant.status, StatusCode::NOT_FOUND);

            let mut tx = pool
                .begin()
                .await
                .expect("RLS probe transaction should start");
            sqlx::query("SET LOCAL ROLE star_engineering_directory_owner")
                .execute(&mut *tx)
                .await
                .expect("runtime login may select the non-login owner role");
            sqlx::query(
                "SELECT set_config('app.tenant_id',$1,true),set_config('app.actor_id',$2,true)",
            )
            .bind(ACCEPTANCE_OTHER_TENANT)
            .bind(ACCEPTANCE_OTHER_ACTOR)
            .execute(&mut *tx)
            .await
            .expect("RLS probe identity should be transaction local");
            let visible: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM multica.engineering_run_worktree_binding WHERE worktree_id=$1",
            )
            .bind(Uuid::parse_str(ACCEPTANCE_WORKTREE).unwrap())
            .fetch_one(&mut *tx)
            .await
            .expect("runtime owner role should retain scoped select");
            assert_eq!(visible, 0, "tenant RLS must hide the accepted binding");
            tx.rollback()
                .await
                .expect("RLS probe must not persist state");
        }
        "rollback-detach" => {
            let (binding_id, version) = current_binding(&pool).await;
            let body = mutation_body(
                json!({"state":"bound","binding_id":binding_id,"version":version}),
                1,
                Uuid::new_v4(),
            );
            let reply = send_binding_mutation(
                app,
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::DELETE,
                &Uuid::new_v4().to_string(),
                &body,
            )
            .await;
            assert_private_response(&reply);
            assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
        }
        "detach" => {
            let (binding_id, version) = current_binding(&pool).await;
            let body = mutation_body(
                json!({"state":"bound","binding_id":binding_id,"version":version}),
                1,
                Uuid::new_v4(),
            );
            let key = Uuid::new_v4().to_string();
            let reply = send_binding_mutation(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::DELETE,
                &key,
                &body,
            )
            .await;
            assert_private_response(&reply);
            assert_eq!(reply.status, StatusCode::OK);
            assert_eq!(reply.body["worktree"]["binding_state"], "unassigned");

            let replay = send_binding_mutation(
                app.clone(),
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::DELETE,
                &key,
                &body,
            )
            .await;
            assert_private_response(&replay);
            assert_eq!(replay.status, StatusCode::OK);
            assert_eq!(
                replay.body, reply.body,
                "detach replay returns the durable receipt"
            );

            let changed = mutation_body(
                json!({"state":"bound","binding_id":binding_id,"version":version}),
                1,
                Uuid::new_v4(),
            );
            let reused_key = send_binding_mutation(
                app,
                &jwt,
                ACCEPTANCE_ACTOR,
                ACCEPTANCE_TENANT,
                Method::DELETE,
                &key,
                &changed,
            )
            .await;
            assert_private_response(&reused_key);
            assert_eq!(reused_key.status, StatusCode::CONFLICT);
        }
        other => panic!("unknown runner phase: {other}"),
    }
    pool.close().await;
}
