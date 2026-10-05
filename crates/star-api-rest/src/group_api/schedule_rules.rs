//! @cypher schema=1 source_sha256=46871b38410fcca82c658125f9e219bcfbaca3cbe7c36a6d7ee22d09d8fc136c
//! MERGE (self:File {path:"crates/star-api-rest/src/group_api/schedule_rules.rs"})
//! MERGE (module:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::module",kind:"module"})
//! MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::router",kind:"function"})
//! MERGE (list:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::list_rules",kind:"function"})
//! MERGE (get:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::get_rule",kind:"function"})
//! MERGE (create:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::create_rule",kind:"function"})
//! MERGE (revise:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::revise_rule",kind:"function"})
//! MERGE (begin:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::begin_schedule_tx",kind:"function"})
//! MERGE (authorize:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::authorize_run",kind:"function"})
//! MERGE (writer:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::require_schedule_writer",kind:"function"})
//! MERGE (prepare:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::prepare_rule",kind:"function"})
//! MERGE (validate_recurrence:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::validate_recurrence",kind:"function"})
//! MERGE (persist:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::persist_revision",kind:"function"})
//! MERGE (record_event:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::record_rule_event",kind:"function"})
//! MERGE (load:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::load_current_rule",kind:"function"})
//! MERGE (lock:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::lock_idempotency_key",kind:"function"})
//! MERGE (lookup:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::lookup_idempotency",kind:"function"})
//! MERGE (save:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::save_idempotency",kind:"function"})
//! MERGE (key_hash:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::idempotency_key_hash",kind:"function"})
//! MERGE (fingerprint:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::request_fingerprint",kind:"function"})
//! MERGE (response:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::private_json",kind:"function"})
//! MERGE (private_response:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::private_response",kind:"function"})
//! MERGE (response_middleware:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::private_response_middleware",kind:"function"})
//! MERGE (parse_id:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::parse_id",kind:"function"})
//! MERGE (validate_body:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::validate_write_body",kind:"function"})
//! MERGE (overlap:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::overlap_columns",kind:"function"})
//! MERGE (misfire:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::misfire_columns",kind:"function"})
//! MERGE (gap:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::gap_name",kind:"function"})
//! MERGE (fold:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::fold_name",kind:"function"})
//! MERGE (pause:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::pause_name",kind:"function"})
//! MERGE (page_limit:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::MAX_RULE_PAGE",kind:"constant"})
//! MERGE (replay_ttl:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::IDEMPOTENCY_RETENTION_HOURS",kind:"constant"})
//! MERGE (query:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::RuleListQuery",kind:"struct"})
//! MERGE (target_input:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::RuleTargetInput",kind:"struct"})
//! MERGE (write_body:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::RuleWriteBody",kind:"struct"})
//! MERGE (run_as_actor:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::run_as_actor_id",kind:"field"})
//! MERGE (json_row:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::RuleJsonRow",kind:"struct"})
//! MERGE (target_row:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::AuthorizedTargetRow",kind:"struct"})
//! MERGE (replay:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::IdempotencyReplay",kind:"struct"})
//! MERGE (idempotency_row:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::IdempotencyRow",kind:"struct"})
//! MERGE (tests:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests",kind:"module"})
//! MERGE (policy_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::overlap_and_misfire_policies_map_to_database_columns",kind:"test"})
//! MERGE (role_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::task_execution_rules_do_not_grant_agent_role_schedule_authority",kind:"test"})
//! MERGE (response_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_rule_responses_are_private",kind:"test"})
//! MERGE (route_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_rule_routes_are_mounted_and_require_authentication",kind:"test"})
//! MERGE (scope_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_rule_routes_reject_missing_scopes_before_database_access",kind:"test"})
//! MERGE (body_limit_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_rule_write_body_limit_is_enforced",kind:"test"})
//! MERGE (run_as_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_rule_request_cannot_select_run_as_actor",kind:"test"})
//! MERGE (acl_database_test:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::schedule_run_as_authorization_rechecks_canonical_directory_grants",kind:"test"})
//! MERGE (jwt_fixture:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::route_test_jwt_config",kind:"function"})
//! MERGE (token_fixture:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::route_test_token",kind:"function"})
//! MERGE (body_fixture:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::tests::route_test_body",kind:"function"})
//! MERGE (build_router:Symbol {id:"crates/star-api-rest/src/group_api.rs::build_group_router",kind:"function"})
//! MERGE (issue_token:Symbol {id:"crates/star-api-rest/src/auth/mod.rs::issue_token",kind:"function"})
//! MERGE (require_scope:Symbol {id:"crates/star-api-rest/src/group_api.rs::require_scope",kind:"function"})
//! MERGE (set_tenant:Symbol {id:"crates/star-api-rest/src/group_api.rs::set_tenant",kind:"function"})
//! MERGE (validate_actor:Symbol {id:"crates/star-api-rest/src/group_api.rs::validate_actor",kind:"function"})
//! MERGE (run_scope:Type {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::RunTaskScope"})
//! MERGE (admission_snapshot:Symbol {id:"crates/star-api-rest/src/group_api/execution_catalogs.rs::load_current_execution_admission_snapshot",kind:"function"})
//! MERGE (materializer:Symbol {id:"crates/domain-automation/src/schedule.rs::materialize_schedule_window",kind:"function"})
//! MERGE (authorize_principal:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::authorize_run_principal",kind:"function"})
//! MERGE (authorize_run_as:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::authorize_schedule_run_as",kind:"function"})
//! MERGE (authorization_row:Symbol {id:"crates/star-api-rest/src/group_api/schedule_rules.rs::ScheduleRunAuthorizationRow",kind:"struct"})
//! MERGE (rules:Table {id:"automation.schedule_rule_revision"})
//! MERGE (run_as_column:Column {id:"automation.schedule_rule_revision.run_as_actor_id"})
//! MERGE (project_grants:Table {id:"permission.project_role_binding"})
//! MERGE (branch_grants:Table {id:"permission.cloud_branch_role_binding"})
//! MERGE (run_grants:Table {id:"permission.engineering_run_role_binding"})
//! MERGE (run_directory:Table {id:"multica.engineering_run"})
//! MERGE (run_revisions:Table {id:"multica.engineering_run_revision"})
//! MERGE (branch_directory:Table {id:"scm.cloud_branch"})
//! MERGE (branch_revisions:Table {id:"scm.cloud_branch_revision"})
//! MERGE (audit:Table {id:"automation.schedule_rule_audit"})
//! MERGE (outbox:Table {id:"automation.schedule_rule_outbox"})
//! MERGE (idempotency:Table {id:"automation.schedule_rule_command_idempotency"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(router)
//! MERGE (module)-[:DEFINES]->(list)
//! MERGE (module)-[:DEFINES]->(get)
//! MERGE (module)-[:DEFINES]->(create)
//! MERGE (module)-[:DEFINES]->(revise)
//! MERGE (module)-[:DEFINES]->(begin)
//! MERGE (module)-[:DEFINES]->(authorize)
//! MERGE (module)-[:DEFINES]->(authorize_principal)
//! MERGE (module)-[:DEFINES]->(authorize_run_as)
//! MERGE (module)-[:DEFINES]->(authorization_row)
//! MERGE (module)-[:DEFINES]->(writer)
//! MERGE (module)-[:DEFINES]->(prepare)
//! MERGE (module)-[:DEFINES]->(validate_recurrence)
//! MERGE (module)-[:DEFINES]->(persist)
//! MERGE (module)-[:DEFINES]->(record_event)
//! MERGE (module)-[:DEFINES]->(load)
//! MERGE (module)-[:DEFINES]->(lock)
//! MERGE (module)-[:DEFINES]->(lookup)
//! MERGE (module)-[:DEFINES]->(save)
//! MERGE (module)-[:DEFINES]->(key_hash)
//! MERGE (module)-[:DEFINES]->(fingerprint)
//! MERGE (module)-[:DEFINES]->(response)
//! MERGE (module)-[:DEFINES]->(private_response)
//! MERGE (module)-[:DEFINES]->(response_middleware)
//! MERGE (module)-[:DEFINES]->(parse_id)
//! MERGE (module)-[:DEFINES]->(validate_body)
//! MERGE (module)-[:DEFINES]->(overlap)
//! MERGE (module)-[:DEFINES]->(misfire)
//! MERGE (module)-[:DEFINES]->(gap)
//! MERGE (module)-[:DEFINES]->(fold)
//! MERGE (module)-[:DEFINES]->(pause)
//! MERGE (module)-[:DEFINES]->(page_limit)
//! MERGE (module)-[:DEFINES]->(replay_ttl)
//! MERGE (module)-[:DEFINES]->(query)
//! MERGE (module)-[:DEFINES]->(target_input)
//! MERGE (module)-[:DEFINES]->(write_body)
//! MERGE (module)-[:DEFINES]->(json_row)
//! MERGE (module)-[:DEFINES]->(target_row)
//! MERGE (module)-[:DEFINES]->(replay)
//! MERGE (module)-[:DEFINES]->(idempotency_row)
//! MERGE (module)-[:DEFINES]->(tests)
//! MERGE (tests)-[:DEFINES]->(policy_test)
//! MERGE (tests)-[:DEFINES]->(role_test)
//! MERGE (tests)-[:DEFINES]->(response_test)
//! MERGE (tests)-[:DEFINES]->(route_test)
//! MERGE (tests)-[:DEFINES]->(scope_test)
//! MERGE (tests)-[:DEFINES]->(body_limit_test)
//! MERGE (tests)-[:DEFINES]->(jwt_fixture)
//! MERGE (tests)-[:DEFINES]->(token_fixture)
//! MERGE (tests)-[:DEFINES]->(body_fixture)
//! MERGE (router)-[:USES]->(response_middleware)
//! MERGE (router)-[:CALLS]->(list)
//! MERGE (router)-[:CALLS]->(get)
//! MERGE (router)-[:CALLS]->(create)
//! MERGE (router)-[:CALLS]->(revise)
//! MERGE (list)-[:CALLS]->(parse_id)
//! MERGE (list)-[:CALLS]->(begin)
//! MERGE (list)-[:CALLS]->(authorize)
//! MERGE (list)-[:CALLS]->(response)
//! MERGE (get)-[:CALLS]->(parse_id)
//! MERGE (get)-[:CALLS]->(begin)
//! MERGE (get)-[:CALLS]->(authorize)
//! MERGE (get)-[:CALLS]->(load)
//! MERGE (get)-[:CALLS]->(response)
//! MERGE (create)-[:CALLS]->(validate_body)
//! MERGE (create)-[:CALLS]->(parse_id)
//! MERGE (create)-[:CALLS]->(key_hash)
//! MERGE (create)-[:CALLS]->(fingerprint)
//! MERGE (create)-[:CALLS]->(begin)
//! MERGE (create)-[:CALLS]->(authorize)
//! MERGE (create)-[:CALLS]->(lock)
//! MERGE (create)-[:CALLS]->(lookup)
//! MERGE (create)-[:CALLS]->(prepare)
//! MERGE (create)-[:CALLS]->(validate_recurrence)
//! MERGE (create)-[:CALLS]->(persist)
//! MERGE (create)-[:CALLS]->(load)
//! MERGE (create)-[:CALLS]->(record_event)
//! MERGE (create)-[:CALLS]->(save)
//! MERGE (create)-[:CALLS]->(response)
//! MERGE (revise)-[:CALLS]->(validate_body)
//! MERGE (revise)-[:CALLS]->(parse_id)
//! MERGE (revise)-[:CALLS]->(key_hash)
//! MERGE (revise)-[:CALLS]->(fingerprint)
//! MERGE (revise)-[:CALLS]->(begin)
//! MERGE (revise)-[:CALLS]->(authorize)
//! MERGE (revise)-[:CALLS]->(lock)
//! MERGE (revise)-[:CALLS]->(lookup)
//! MERGE (revise)-[:CALLS]->(prepare)
//! MERGE (revise)-[:CALLS]->(validate_recurrence)
//! MERGE (revise)-[:CALLS]->(persist)
//! MERGE (revise)-[:CALLS]->(load)
//! MERGE (revise)-[:CALLS]->(record_event)
//! MERGE (revise)-[:CALLS]->(save)
//! MERGE (revise)-[:CALLS]->(response)
//! MERGE (create)-[:CALLS]->(authorize_run_as)
//! MERGE (revise)-[:CALLS]->(authorize_run_as)
//! MERGE (begin)-[:CALLS]->(validate_actor)
//! MERGE (begin)-[:CALLS]->(require_scope)
//! MERGE (begin)-[:CALLS]->(set_tenant)
//! MERGE (authorize)-[:CALLS]->(authorize_principal)
//! MERGE (authorize_run_as)-[:CALLS]->(authorize_principal)
//! MERGE (authorize_principal)-[:CALLS]->(writer)
//! MERGE (authorize_principal)-[:CALLS]->(set_tenant)
//! MERGE (authorize_principal)-[:READS]->(project_grants)
//! MERGE (authorize_principal)-[:READS]->(branch_grants)
//! MERGE (authorize_principal)-[:READS]->(run_grants)
//! MERGE (authorize_principal)-[:READS]->(run_directory)
//! MERGE (authorize_principal)-[:READS]->(run_revisions)
//! MERGE (authorize_principal)-[:READS]->(branch_directory)
//! MERGE (authorize_principal)-[:READS]->(branch_revisions)
//! MERGE (authorize_principal)-[:USES_TYPE]->(authorization_row)
//! MERGE (authorize_principal)-[:USES_TYPE]->(run_scope)
//! MERGE (authorization_row)-[:USES_TYPE]->(project_grants)
//! MERGE (authorization_row)-[:USES_TYPE]->(branch_grants)
//! MERGE (authorization_row)-[:USES_TYPE]->(run_grants)
//! MERGE (prepare)-[:CALLS]->(admission_snapshot)
//! MERGE (validate_recurrence)-[:CALLS]->(materializer)
//! MERGE (list)-[:USES]->(page_limit)
//! MERGE (save)-[:USES]->(replay_ttl)
//! MERGE (response)-[:CALLS]->(private_response)
//! MERGE (response_middleware)-[:CALLS]->(private_response)
//! MERGE (policy_test)-[:CALLS]->(overlap)
//! MERGE (policy_test)-[:CALLS]->(misfire)
//! MERGE (role_test)-[:CALLS]->(writer)
//! MERGE (response_test)-[:CALLS]->(response)
//! MERGE (response_test)-[:CALLS]->(response_middleware)
//! MERGE (route_test)-[:CALLS]->(build_router)
//! MERGE (scope_test)-[:CALLS]->(build_router)
//! MERGE (scope_test)-[:CALLS]->(token_fixture)
//! MERGE (scope_test)-[:CALLS]->(body_fixture)
//! MERGE (body_limit_test)-[:CALLS]->(build_router)
//! MERGE (body_limit_test)-[:CALLS]->(token_fixture)
//! MERGE (scope_test)-[:CALLS]->(jwt_fixture)
//! MERGE (body_limit_test)-[:CALLS]->(jwt_fixture)
//! MERGE (tests)-[:DEFINES]->(run_as_test)
//! MERGE (run_as_test)-[:TESTS]->(write_body)
//! MERGE (tests)-[:DEFINES]->(acl_database_test)
//! MERGE (acl_database_test)-[:TESTS]->(authorize_run_as)
//! MERGE (token_fixture)-[:CALLS]->(issue_token)
//! MERGE (persist)-[:WRITES]->(rules)
//! MERGE (persist)-[:WRITES]->(run_as_column)
//! MERGE (revise)-[:WRITES]->(rules)
//! MERGE (revise)-[:READS]->(run_as_column)
//! MERGE (prepare)-[:CONFIGURES]->(run_as_column)
//! MERGE (record_event)-[:WRITES]->(audit)
//! MERGE (record_event)-[:WRITES]->(outbox)
//! MERGE (list)-[:READS]->(rules)
//! MERGE (load)-[:READS]->(rules)
//! MERGE (lookup)-[:READS]->(idempotency)
//! MERGE (lookup)-[:WRITES]->(idempotency)
//! MERGE (save)-[:WRITES]->(idempotency)
//! @endcypher

//! Authenticated Run-scoped REST lifecycle for immutable Schedule rule revisions.

use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use domain_automation::schedule::{
    materialize_schedule_window, AutomationScheduleRuleRevisionV1, ScheduleDstFoldPolicyV1,
    ScheduleDstGapPolicyV1, ScheduleMisfirePolicyV1, ScheduleOverlapPolicyV1,
    SchedulePausePolicyV1, ScheduleRetryPolicyV1, ScheduleTargetV1, SCHEDULE_PARSER_VERSION,
    SCHEDULE_TZDB_VERSION,
};
use domain_automation::RuleId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{
    require_scope, set_tenant, validate_actor, AuthUser, AuthenticatedUser, GroupApiError,
    GroupApiState,
};

const MAX_RULE_PAGE: i64 = 100;
const IDEMPOTENCY_RETENTION_HOURS: i64 = 24;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleListQuery {
    limit: Option<i64>,
    after_rule_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RuleTargetInput {
    worktree_id: Uuid,
    work_item_id: Uuid,
    execution_profile_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RuleWriteBody {
    #[serde(default)]
    expected_current_version: Option<i64>,
    display_name: String,
    enabled: bool,
    cron_expression: String,
    time_zone: String,
    dst_gap_policy: ScheduleDstGapPolicyV1,
    dst_fold_policy: ScheduleDstFoldPolicyV1,
    overlap_policy: ScheduleOverlapPolicyV1,
    misfire_policy: ScheduleMisfirePolicyV1,
    pause_policy: SchedulePausePolicyV1,
    retry_policy: ScheduleRetryPolicyV1,
    deadline_seconds: u32,
    target: RuleTargetInput,
}

#[derive(FromRow)]
struct RuleJsonRow {
    rule: Value,
}

#[derive(FromRow)]
struct AuthorizedTargetRow {
    repository_id: Uuid,
    branch_id: Uuid,
    engineering_run_id: Uuid,
    worktree_archived: bool,
}

#[derive(FromRow)]
struct ScheduleRunAuthorizationRow {
    engineering_run_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    branch_id: Uuid,
    run_state: String,
    project_role: String,
    run_role: String,
    run_grant_binding_id: Uuid,
    run_grant_version: i32,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/engineering-runs/{run_id}/automation/schedule-rules",
            get(list_rules).post(create_rule),
        )
        .route(
            "/api/v1/projects/{project_id}/engineering-runs/{run_id}/automation/schedule-rules/{rule_id}",
            get(get_rule).put(revise_rule),
        )
        .layer(DefaultBodyLimit::max(16 * 1024))
        .route_layer(map_response(private_response_middleware))
}

async fn list_rules(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project, run)): Path<(String, String)>,
    Query(query): Query<RuleListQuery>,
) -> Result<Response, GroupApiError> {
    let project_id = parse_id(&project)?;
    let run_id = parse_id(&run)?;
    let limit = query.limit.unwrap_or(50);
    if !(1..=MAX_RULE_PAGE).contains(&limit) || query.after_rule_id.is_some_and(|id| id.is_nil()) {
        return Err(GroupApiError::invalid_request("invalid_schedule_rule_page"));
    }
    let mut tx = begin_schedule_tx(&state, &actor, false).await?;
    authorize_run(&mut tx, &actor, project_id, run_id, false).await?;
    let mut rows = sqlx::query_as::<_, RuleJsonRow>(
        r#"SELECT to_jsonb(r) AS rule
           FROM automation.schedule_rule_revision r
            WHERE r.tenant_id=$1 AND r.project_id=$2 AND r.engineering_run_id=$3
              AND r.valid_from<=now() AND r.valid_to IS NULL AND ($4::uuid IS NULL OR r.rule_id>$4)
           ORDER BY r.rule_id LIMIT $5"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(query.after_rule_id)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let has_more = rows.len() > limit as usize;
    if has_more {
        rows.pop();
    }
    let next_after_rule_id = if has_more {
        rows.last()
            .and_then(|row| row.rule["rule_id"].as_str())
            .map(str::to_owned)
    } else {
        None
    };
    Ok(private_json(
        StatusCode::OK,
        json!({
            "project_id": project_id,
            "engineering_run_id": run_id,
            "rules": rows.into_iter().map(|row| row.rule).collect::<Vec<_>>(),
            "limit": limit,
            "next_after_rule_id": next_after_rule_id
        }),
    ))
}

async fn get_rule(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project, run, rule)): Path<(String, String, String)>,
) -> Result<Response, GroupApiError> {
    let project_id = parse_id(&project)?;
    let run_id = parse_id(&run)?;
    let rule_id = parse_id(&rule)?;
    let mut tx = begin_schedule_tx(&state, &actor, false).await?;
    authorize_run(&mut tx, &actor, project_id, run_id, false).await?;
    let row = load_current_rule(&mut tx, actor.tenant_id, project_id, run_id, rule_id).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(private_json(StatusCode::OK, json!({"rule": row})))
}

async fn create_rule(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project, run)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<RuleWriteBody>,
) -> Result<Response, GroupApiError> {
    validate_write_body(&body, true)?;
    let project_id = parse_id(&project)?;
    let run_id = parse_id(&run)?;
    let key_hash = idempotency_key_hash(&headers)?;
    let request_hash = request_fingerprint(&actor, project_id, run_id, "create", &body)?;
    let mut tx = begin_schedule_tx(&state, &actor, true).await?;
    let run_scope = authorize_run(&mut tx, &actor, project_id, run_id, true).await?;
    lock_idempotency_key(&mut tx, &actor, project_id, "create", &key_hash).await?;
    if let Some(replay) = lookup_idempotency(
        &mut tx,
        &actor,
        project_id,
        "create",
        &key_hash,
        &request_hash,
    )
    .await?
    {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(private_json(replay.status, replay.body));
    }

    if body.enabled {
        authorize_schedule_run_as(&mut tx, actor.tenant_id, actor.user_id, project_id, run_id)
            .await?;
    }

    let (rule, target) = prepare_rule(
        &mut tx,
        &actor,
        project_id,
        run_id,
        &run_scope,
        &body,
        actor.user_id,
        Uuid::new_v4(),
        1,
    )
    .await?;
    validate_recurrence(&rule)?;
    persist_revision(&mut tx, &actor, &body.display_name, &rule, &target).await?;
    let response = load_current_rule(
        &mut tx,
        actor.tenant_id,
        project_id,
        run_id,
        rule.rule_id.as_uuid(),
    )
    .await?;
    let status = StatusCode::CREATED;
    record_rule_event(&mut tx, &actor, &rule, "created", None).await?;
    let response_body = json!({"rule": response});
    save_idempotency(
        &mut tx,
        &actor,
        project_id,
        "create",
        &key_hash,
        &request_hash,
        status,
        &response_body,
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(private_json(status, response_body))
}

async fn revise_rule(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project, run, rule)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<RuleWriteBody>,
) -> Result<Response, GroupApiError> {
    validate_write_body(&body, false)?;
    let project_id = parse_id(&project)?;
    let run_id = parse_id(&run)?;
    let rule_id = parse_id(&rule)?;
    let key_hash = idempotency_key_hash(&headers)?;
    let request_hash = request_fingerprint(&actor, project_id, run_id, "revise", &body)?;
    let mut tx = begin_schedule_tx(&state, &actor, true).await?;
    let run_scope = authorize_run(&mut tx, &actor, project_id, run_id, true).await?;
    lock_idempotency_key(&mut tx, &actor, project_id, "revise", &key_hash).await?;
    if let Some(replay) = lookup_idempotency(
        &mut tx,
        &actor,
        project_id,
        "revise",
        &key_hash,
        &request_hash,
    )
    .await?
    {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(private_json(replay.status, replay.body));
    }
    let (current_version, run_as_actor_id) = sqlx::query_as::<_, (i64, Uuid)>(
        r#"SELECT rule_version, run_as_actor_id FROM automation.schedule_rule_revision
           WHERE tenant_id=$1 AND project_id=$2 AND engineering_run_id=$3
              AND rule_id=$4 AND valid_from<=now() AND valid_to IS NULL FOR UPDATE"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(rule_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if Some(current_version) != body.expected_current_version {
        return Err(GroupApiError::conflict("schedule_rule_version_conflict"));
    }
    if body.enabled {
        authorize_schedule_run_as(
            &mut tx,
            actor.tenant_id,
            run_as_actor_id,
            project_id,
            run_id,
        )
        .await?;
    }
    let next_version = current_version
        .checked_add(1)
        .ok_or_else(|| GroupApiError::conflict("schedule_rule_version_exhausted"))?;
    let (revision, target) = prepare_rule(
        &mut tx,
        &actor,
        project_id,
        run_id,
        &run_scope,
        &body,
        run_as_actor_id,
        rule_id,
        next_version as u64,
    )
    .await?;
    validate_recurrence(&revision)?;
    sqlx::query(
        r#"UPDATE automation.schedule_rule_revision SET valid_to=now()
           WHERE tenant_id=$1 AND project_id=$2 AND rule_id=$3
             AND rule_version=$4 AND valid_to IS NULL"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(rule_id)
    .bind(current_version)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::conflict("schedule_rule_version_conflict"))?;
    persist_revision(&mut tx, &actor, &body.display_name, &revision, &target).await?;
    let response = load_current_rule(&mut tx, actor.tenant_id, project_id, run_id, rule_id).await?;
    let status = StatusCode::OK;
    record_rule_event(&mut tx, &actor, &revision, "revised", Some(current_version)).await?;
    let response_body = json!({"rule": response});
    save_idempotency(
        &mut tx,
        &actor,
        project_id,
        "revise",
        &key_hash,
        &request_hash,
        status,
        &response_body,
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(private_json(status, response_body))
}

async fn begin_schedule_tx(
    state: &GroupApiState,
    actor: &AuthUser,
    write: bool,
) -> Result<Transaction<'static, Postgres>, GroupApiError> {
    validate_actor(actor)?;
    require_scope(
        actor,
        if write {
            "work-item:write"
        } else {
            "work-item:read"
        },
    )?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    sqlx::query("SELECT set_config('app.actor_id',$1,true), set_config('statement_timeout','5000',true), set_config('lock_timeout','1000',true)")
        .bind(actor.user_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(tx)
}

async fn authorize_run(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    run_id: Uuid,
    write: bool,
) -> Result<super::engineering_runs::RunTaskScope, GroupApiError> {
    authorize_run_principal(
        tx,
        actor.tenant_id,
        actor.user_id,
        project_id,
        run_id,
        write,
        false,
    )
    .await
}

/// Recheck the scheduled execution principal's current Project, Branch and Engineering Run grants.
/// The caller must run this inside the same transaction that enables a rule or admits a scheduled
/// Run; an absent row or database error is a fail-closed result. This read does not serialize a
/// concurrent grant revocation; that race remains a separate admission/release gate. This checks
/// directory authorization only: current target/Profile/HookSet/quota and Runtime fence checks
/// remain separate gates.
pub(super) async fn authorize_schedule_run_as(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    run_as_actor_id: Uuid,
    project_id: Uuid,
    run_id: Uuid,
) -> Result<super::engineering_runs::RunTaskScope, GroupApiError> {
    authorize_run_principal(
        tx,
        tenant_id,
        run_as_actor_id,
        project_id,
        run_id,
        true,
        true,
    )
    .await
}

async fn authorize_run_principal(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    run_id: Uuid,
    require_writer: bool,
    require_active_run: bool,
) -> Result<super::engineering_runs::RunTaskScope, GroupApiError> {
    if tenant_id.is_nil() || actor_id.is_nil() || project_id.is_nil() || run_id.is_nil() {
        return Err(GroupApiError::not_found());
    }
    set_tenant(tx, tenant_id).await?;
    let authorization_sql = schedule_run_authorization_sql();
    let authority = sqlx::query_as::<_, ScheduleRunAuthorizationRow>(&authorization_sql)
        .bind(tenant_id)
        .bind(actor_id)
        .bind(project_id)
        .bind(run_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?
        .ok_or_else(GroupApiError::not_found)?;

    if require_active_run && authority.run_state != "active" {
        return Err(GroupApiError::conflict("engineering_run_not_active"));
    }
    if require_writer {
        require_schedule_writer(&authority.project_role)?;
        require_schedule_writer(&authority.run_role)?;
    }
    Ok(super::engineering_runs::RunTaskScope {
        engineering_run_id: authority.engineering_run_id,
        project_id: authority.project_id,
        repository_id: authority.repository_id,
        branch_id: authority.branch_id,
        role: authority.run_role,
        permission_snapshot_ref: format!(
            "{}:v{}",
            authority.run_grant_binding_id, authority.run_grant_version
        ),
    })
}

fn schedule_run_authorization_sql() -> &'static str {
    r#"SELECT r.engineering_run_id,r.project_id,r.repository_id,r.branch_id,
                  rv.state AS run_state,p.role AS project_role,
                  rg.role AS run_role,
                  rg.binding_id AS run_grant_binding_id,rg.version AS run_grant_version
           FROM multica.engineering_run r
           JOIN multica.engineering_run_revision rv
             ON rv.tenant_id=r.tenant_id AND rv.engineering_run_id=r.engineering_run_id
            AND rv.valid_from<=now() AND rv.valid_to IS NULL AND rv.state<>'archived'
           JOIN scm.cloud_branch b
             ON b.tenant_id=r.tenant_id AND b.branch_id=r.branch_id
            AND b.project_id=r.project_id AND b.repository_id=r.repository_id
           JOIN scm.cloud_branch_revision bv
             ON bv.tenant_id=b.tenant_id AND bv.branch_id=b.branch_id
            AND bv.valid_from<=now() AND bv.valid_to IS NULL AND bv.state='active'
           JOIN permission.project_role_binding p
             ON p.tenant_id=r.tenant_id AND p.project_id=r.project_id AND p.user_id=$2
            AND p.valid_from<=now() AND p.valid_to IS NULL
           JOIN permission.cloud_branch_role_binding bg
             ON bg.tenant_id=b.tenant_id AND bg.branch_id=b.branch_id AND bg.user_id=$2
            AND bg.valid_from<=now() AND bg.valid_to IS NULL
           JOIN permission.engineering_run_role_binding rg
             ON rg.tenant_id=r.tenant_id AND rg.engineering_run_id=r.engineering_run_id
            AND rg.user_id=$2 AND rg.valid_from<=now() AND rg.valid_to IS NULL
           WHERE r.tenant_id=$1 AND r.project_id=$3 AND r.engineering_run_id=$4"#
}

fn require_schedule_writer(role: &str) -> Result<(), GroupApiError> {
    match role {
        "tenant_admin" | "project_admin" | "developer" => Ok(()),
        _ => Err(GroupApiError::forbidden()),
    }
}

async fn prepare_rule(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    run_id: Uuid,
    run_scope: &super::engineering_runs::RunTaskScope,
    body: &RuleWriteBody,
    run_as_actor_id: Uuid,
    rule_id: Uuid,
    rule_version: u64,
) -> Result<(AutomationScheduleRuleRevisionV1, ScheduleTargetV1), GroupApiError> {
    let target_row = sqlx::query_as::<_, AuthorizedTargetRow>(
        r#"SELECT w.repo_id AS repository_id, b.branch_id, b.engineering_run_id,
                  w.archived AS worktree_archived
           FROM worktree_canvas_worktree w
           JOIN multica.worktree_project_binding p
             ON p.tenant_id=w.tenant_id AND p.worktree_id=w.id
            AND p.project_id=w.project_id AND p.valid_from<=now() AND p.valid_to IS NULL
           JOIN multica.engineering_run_worktree_binding b
             ON b.tenant_id=w.tenant_id AND b.project_id=p.project_id
            AND b.repository_id=w.repo_id AND b.worktree_id=w.id
            AND b.project_binding_id=p.binding_id AND b.valid_from<=now() AND b.valid_to IS NULL
           JOIN multica.task_metadata m
             ON m.tenant_id=b.tenant_id AND m.project_id=b.project_id
            AND m.repository_id=b.repository_id AND m.branch_id=b.branch_id
            AND m.engineering_run_id=b.engineering_run_id AND m.work_item_id=$4
            AND m.valid_from<=now() AND m.valid_to IS NULL
           JOIN multica.work_item_worktree l
             ON l.tenant_id=m.tenant_id AND l.project_id=m.project_id
            AND l.work_item_id=m.work_item_id AND l.worktree_id=w.id
            AND l.valid_from<=now() AND l.valid_to IS NULL
           WHERE w.tenant_id=$1 AND w.project_id=$2 AND b.engineering_run_id=$3
             AND w.id=$5 AND w.archived=FALSE
           FOR SHARE OF w,p,b,m,l"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(body.target.work_item_id)
    .bind(body.target.worktree_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if target_row.worktree_archived
        || target_row.engineering_run_id != run_id
        || target_row.repository_id != run_scope.repository_id
        || target_row.branch_id != run_scope.branch_id
    {
        return Err(GroupApiError::not_found());
    }
    let snapshot = super::execution_catalogs::load_current_execution_admission_snapshot(
        tx,
        actor.tenant_id,
        project_id,
        body.target.worktree_id,
        body.target.execution_profile_id,
    )
    .await?;
    let target = ScheduleTargetV1 {
        project_id,
        branch_id: run_scope.branch_id,
        engineering_run_id: run_scope.engineering_run_id,
        repository_id: run_scope.repository_id,
        worktree_id: body.target.worktree_id,
        work_item_id: body.target.work_item_id,
        execution_profile_id: snapshot.profile_identity.profile_id,
        execution_profile_version: snapshot.profile_identity.version,
        execution_profile_digest: snapshot.profile_identity.content_digest.clone(),
        hook_set_version: snapshot.hook_set.version,
        hook_set_digest: snapshot.hook_set.effective_digest.clone(),
    };
    let rule = AutomationScheduleRuleRevisionV1 {
        tenant_id: actor.tenant_id,
        rule_id: RuleId::from_uuid(rule_id),
        rule_version,
        run_as_actor_id,
        enabled: body.enabled,
        project_id,
        cron_expression: body.cron_expression.trim().to_owned(),
        time_zone: body.time_zone.trim().to_owned(),
        parser_version: SCHEDULE_PARSER_VERSION.to_owned(),
        tzdb_version: SCHEDULE_TZDB_VERSION.to_owned(),
        dst_gap_policy: body.dst_gap_policy,
        dst_fold_policy: body.dst_fold_policy,
        overlap_policy: body.overlap_policy,
        misfire_policy: body.misfire_policy,
        pause_policy: body.pause_policy,
        retry_policy: body.retry_policy,
        deadline_seconds: body.deadline_seconds,
        target: target.clone(),
    };
    rule.validate()
        .map_err(|_| GroupApiError::invalid_request("invalid_schedule_rule"))?;
    Ok((rule, target))
}

fn validate_recurrence(rule: &AutomationScheduleRuleRevisionV1) -> Result<(), GroupApiError> {
    let now = Utc::now();
    let mut candidate = rule.clone();
    candidate.enabled = true;
    materialize_schedule_window(
        &candidate,
        now,
        now + ChronoDuration::seconds(120),
        now,
        now,
        1,
    )
    .map(|_| ())
    .map_err(|_| GroupApiError::invalid_request("invalid_schedule_recurrence"))
}

async fn persist_revision(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    display_name: &str,
    rule: &AutomationScheduleRuleRevisionV1,
    target: &ScheduleTargetV1,
) -> Result<(), GroupApiError> {
    let (overlap, overlap_max) = overlap_columns(rule.overlap_policy);
    let (misfire, misfire_max) = misfire_columns(rule.misfire_policy);
    sqlx::query(
        r#"INSERT INTO automation.schedule_rule_revision
           (tenant_id,project_id,rule_id,rule_version,display_name,enabled,cron_expression,time_zone,
            parser_version,tzdb_version,dst_gap_policy,dst_fold_policy,overlap_policy,overlap_max_active,
            misfire_policy,misfire_max_occurrences,pause_policy,retry_max_attempts,
            retry_initial_backoff_seconds,retry_max_backoff_seconds,deadline_seconds,branch_id,
            engineering_run_id,repository_id,worktree_id,work_item_id,execution_profile_id,
            execution_profile_version,execution_profile_digest,hook_set_version,hook_set_digest,
            run_as_actor_id,changed_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,
                   $22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33)"#,
    )
    .bind(rule.tenant_id).bind(rule.project_id).bind(rule.rule_id.as_uuid())
    .bind(i64::try_from(rule.rule_version).map_err(|_| GroupApiError::internal())?)
    .bind(display_name.trim()).bind(rule.enabled).bind(&rule.cron_expression).bind(&rule.time_zone)
    .bind(&rule.parser_version).bind(&rule.tzdb_version)
    .bind(gap_name(rule.dst_gap_policy)).bind(fold_name(rule.dst_fold_policy))
    .bind(overlap).bind(overlap_max).bind(misfire).bind(misfire_max)
    .bind(pause_name(rule.pause_policy))
    .bind(i16::try_from(rule.retry_policy.max_attempts).map_err(|_| GroupApiError::internal())?)
    .bind(i32::try_from(rule.retry_policy.initial_backoff_seconds).map_err(|_| GroupApiError::internal())?)
    .bind(i32::try_from(rule.retry_policy.max_backoff_seconds).map_err(|_| GroupApiError::internal())?)
    .bind(i32::try_from(rule.deadline_seconds).map_err(|_| GroupApiError::internal())?)
    .bind(target.branch_id).bind(target.engineering_run_id).bind(target.repository_id)
    .bind(target.worktree_id).bind(target.work_item_id).bind(target.execution_profile_id)
    .bind(i64::try_from(target.execution_profile_version).map_err(|_| GroupApiError::internal())?)
    .bind(&target.execution_profile_digest)
    .bind(i64::try_from(target.hook_set_version).map_err(|_| GroupApiError::internal())?)
    .bind(&target.hook_set_digest).bind(rule.run_as_actor_id).bind(actor.user_id)
    .execute(&mut **tx).await.map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn record_rule_event(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    rule: &AutomationScheduleRuleRevisionV1,
    event: &'static str,
    superseded_version: Option<i64>,
) -> Result<(), GroupApiError> {
    let correlation_id = Uuid::new_v4();
    let rule_id = rule.rule_id.as_uuid();
    let version = i64::try_from(rule.rule_version).map_err(|_| GroupApiError::internal())?;
    if let Some(old_version) = superseded_version {
        sqlx::query(
            r#"INSERT INTO automation.schedule_rule_audit
               (tenant_id,project_id,rule_id,rule_version,actor_id,action,correlation_id,details)
               VALUES ($1,$2,$3,$4,$5,'superseded',$6,$7)"#,
        )
        .bind(actor.tenant_id)
        .bind(rule.project_id)
        .bind(rule_id)
        .bind(old_version)
        .bind(actor.user_id)
        .bind(correlation_id)
        .bind(json!({"successor_version": version}))
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    }
    sqlx::query(
        r#"INSERT INTO automation.schedule_rule_audit
           (tenant_id,project_id,rule_id,rule_version,actor_id,action,correlation_id,details)
           VALUES ($1,$2,$3,$4,$5,'created',$6,$7)"#,
    )
    .bind(actor.tenant_id)
    .bind(rule.project_id)
    .bind(rule_id)
    .bind(version)
    .bind(actor.user_id)
    .bind(correlation_id)
    .bind(
        json!({"enabled": rule.enabled, "engineering_run_id": rule.target.engineering_run_id,
                 "run_as_actor_id": rule.run_as_actor_id}),
    )
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let payload = json!({
        "schema_version": 1,
        "tenant_id": actor.tenant_id,
        "project_id": rule.project_id,
        "engineering_run_id": rule.target.engineering_run_id,
        "rule_id": rule_id,
        "rule_version": version,
        "changed_by": actor.user_id,
        "run_as_actor_id": rule.run_as_actor_id,
    });
    sqlx::query(
        r#"INSERT INTO automation.schedule_rule_outbox
           (tenant_id,project_id,engineering_run_id,rule_id,rule_version,event_type,correlation_id,payload)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
    )
    .bind(actor.tenant_id).bind(rule.project_id).bind(rule.target.engineering_run_id)
    .bind(rule_id).bind(version).bind(format!("schedule_rule.{event}"))
    .bind(correlation_id).bind(payload)
    .execute(&mut **tx).await.map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn load_current_rule(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    run_id: Uuid,
    rule_id: Uuid,
) -> Result<Value, GroupApiError> {
    sqlx::query_scalar(
        r#"SELECT to_jsonb(r) FROM automation.schedule_rule_revision r
           WHERE r.tenant_id=$1 AND r.project_id=$2 AND r.engineering_run_id=$3
              AND r.rule_id=$4 AND r.valid_from<=now() AND r.valid_to IS NULL"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(rule_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)
}

struct IdempotencyReplay {
    status: StatusCode,
    body: Value,
}

async fn lock_idempotency_key(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    operation: &str,
    key_hash: &[u8; 32],
) -> Result<(), GroupApiError> {
    let lock_key = format!(
        "schedule:{}/{}/{}/{}/{}",
        actor.tenant_id,
        project_id,
        actor.user_id,
        operation,
        hex::encode(key_hash)
    );
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn lookup_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    operation: &str,
    key_hash: &[u8; 32],
    request_hash: &[u8; 32],
) -> Result<Option<IdempotencyReplay>, GroupApiError> {
    sqlx::query(
        r#"DELETE FROM automation.schedule_rule_command_idempotency
           WHERE tenant_id=$1 AND project_id=$2 AND actor_id=$3 AND operation=$4
             AND idempotency_key_hash=$5 AND expires_at<=now()"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(actor.user_id)
    .bind(operation)
    .bind(key_hash.as_slice())
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"WITH expired AS (
             SELECT ctid FROM automation.schedule_rule_command_idempotency
             WHERE tenant_id=$1 AND project_id=$2 AND actor_id=$3 AND expires_at<=now()
             ORDER BY expires_at LIMIT 64 FOR UPDATE SKIP LOCKED
           ) DELETE FROM automation.schedule_rule_command_idempotency i
             USING expired e WHERE i.ctid=e.ctid"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(actor.user_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let row = sqlx::query_as::<_, IdempotencyRow>(
        r#"SELECT request_hash,response_status,response_body
           FROM automation.schedule_rule_command_idempotency
           WHERE tenant_id=$1 AND project_id=$2 AND actor_id=$3 AND operation=$4
             AND idempotency_key_hash=$5 AND expires_at>now() FOR UPDATE"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(actor.user_id)
    .bind(operation)
    .bind(key_hash.as_slice())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let Some(row) = row else {
        return Ok(None);
    };
    if row.request_hash.as_slice() != request_hash {
        return Err(GroupApiError::conflict("idempotency_key_reused"));
    }
    let status =
        StatusCode::from_u16(row.response_status as u16).map_err(|_| GroupApiError::internal())?;
    Ok(Some(IdempotencyReplay {
        status,
        body: row.response_body,
    }))
}

#[derive(FromRow)]
struct IdempotencyRow {
    request_hash: Vec<u8>,
    response_status: i16,
    response_body: Value,
}

async fn save_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    operation: &str,
    key_hash: &[u8; 32],
    request_hash: &[u8; 32],
    status: StatusCode,
    response: &Value,
) -> Result<(), GroupApiError> {
    sqlx::query(
        r#"INSERT INTO automation.schedule_rule_command_idempotency
           (tenant_id,project_id,actor_id,operation,idempotency_key_hash,request_hash,response_status,response_body,
            retention_period,expires_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,make_interval(hours=>$9::int),now()+make_interval(hours=>$9::int))"#,
    )
    .bind(actor.tenant_id).bind(project_id).bind(actor.user_id).bind(operation)
    .bind(key_hash.as_slice()).bind(request_hash.as_slice()).bind(status.as_u16() as i16)
    .bind(response).bind(IDEMPOTENCY_RETENTION_HOURS as i32)
    .execute(&mut **tx).await.map_err(|_| GroupApiError::conflict("idempotency_race"))?;
    Ok(())
}

fn idempotency_key_hash(headers: &HeaderMap) -> Result<[u8; 32], GroupApiError> {
    let key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|key| {
            !key.is_empty()
                && key.len() <= 128
                && key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
        })
        .ok_or_else(|| GroupApiError::invalid_request("idempotency_key_required"))?;
    Ok(Sha256::digest(key.as_bytes()).into())
}

fn request_fingerprint<T: Serialize>(
    actor: &AuthUser,
    project_id: Uuid,
    run_id: Uuid,
    operation: &str,
    body: &T,
) -> Result<[u8; 32], GroupApiError> {
    let encoded = serde_json::to_vec(&(
        actor.tenant_id,
        actor.user_id,
        project_id,
        run_id,
        operation,
        body,
    ))
    .map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(encoded).into())
}

fn private_json(status: StatusCode, body: Value) -> Response {
    private_response((status, Json(body)).into_response())
}

fn private_response(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Authorization"));
    response
}

async fn private_response_middleware(response: Response) -> Response {
    private_response(response)
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value)
        .map_err(|_| GroupApiError::bad_request())
        .and_then(|id| {
            if id.is_nil() {
                Err(GroupApiError::bad_request())
            } else {
                Ok(id)
            }
        })
}

fn validate_write_body(body: &RuleWriteBody, creating: bool) -> Result<(), GroupApiError> {
    let name = body.display_name.trim();
    if name.is_empty()
        || name.len() > 120
        || name.chars().any(char::is_control)
        || body.cron_expression.trim().is_empty()
        || body.cron_expression.len() > 120
        || body.time_zone.trim().is_empty()
        || body.time_zone.len() > 128
        || body.target.worktree_id.is_nil()
        || body.target.work_item_id.is_nil()
        || body.target.execution_profile_id.is_nil()
        || (creating && body.expected_current_version.is_some())
        || (!creating
            && body
                .expected_current_version
                .map_or(true, |version| version < 1))
    {
        return Err(GroupApiError::invalid_request("invalid_schedule_rule"));
    }
    Ok(())
}

fn overlap_columns(policy: ScheduleOverlapPolicyV1) -> (&'static str, i16) {
    match policy {
        ScheduleOverlapPolicyV1::Skip => ("skip", 1),
        ScheduleOverlapPolicyV1::QueueOne => ("queue_one", 1),
        ScheduleOverlapPolicyV1::AllowBounded { max_active } => {
            ("allow_bounded", max_active as i16)
        }
    }
}

fn misfire_columns(policy: ScheduleMisfirePolicyV1) -> (&'static str, i16) {
    match policy {
        ScheduleMisfirePolicyV1::Skip => ("skip", 1),
        ScheduleMisfirePolicyV1::CoalesceLatest => ("coalesce_latest", 1),
        ScheduleMisfirePolicyV1::CatchUp { max_occurrences } => {
            ("catch_up", max_occurrences as i16)
        }
    }
}

fn gap_name(policy: ScheduleDstGapPolicyV1) -> &'static str {
    match policy {
        ScheduleDstGapPolicyV1::Skip => "skip",
        ScheduleDstGapPolicyV1::ShiftForward => "shift_forward",
    }
}

fn fold_name(policy: ScheduleDstFoldPolicyV1) -> &'static str {
    match policy {
        ScheduleDstFoldPolicyV1::EarlierInstant => "earlier_instant",
        ScheduleDstFoldPolicyV1::LaterInstant => "later_instant",
    }
}

fn pause_name(policy: SchedulePausePolicyV1) -> &'static str {
    match policy {
        SchedulePausePolicyV1::SkipElapsed => "skip_elapsed",
        SchedulePausePolicyV1::CoalesceLatest => "coalesce_latest",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn route_test_jwt_config() -> Arc<crate::auth::JwtConfig> {
        Arc::new(crate::auth::JwtConfig {
            private_key_pem: include_str!("../../tests/fixtures/schedule-route-test-private.pem")
                .to_owned(),
            public_key_pem: include_str!("../../tests/fixtures/schedule-route-test-public.pem")
                .to_owned(),
            issuer: "https://api.star.local".to_owned(),
            audience: "star-api-rest".to_owned(),
            ttl_seconds: 3600,
        })
    }

    fn route_test_token(config: &crate::auth::JwtConfig, scope: &str) -> String {
        crate::auth::issue_token(
            config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            vec!["developer".to_owned()],
            scope.to_owned(),
        )
        .expect("fixed test-only RSA key should sign route test tokens")
        .0
    }

    fn route_test_body(expected_current_version: Option<i64>) -> String {
        let body = RuleWriteBody {
            expected_current_version,
            display_name: "route-test".to_owned(),
            enabled: true,
            cron_expression: "0 * * * *".to_owned(),
            time_zone: "UTC".to_owned(),
            dst_gap_policy: ScheduleDstGapPolicyV1::Skip,
            dst_fold_policy: ScheduleDstFoldPolicyV1::EarlierInstant,
            overlap_policy: ScheduleOverlapPolicyV1::Skip,
            misfire_policy: ScheduleMisfirePolicyV1::Skip,
            pause_policy: SchedulePausePolicyV1::SkipElapsed,
            retry_policy: ScheduleRetryPolicyV1 {
                max_attempts: 1,
                initial_backoff_seconds: 1,
                max_backoff_seconds: 1,
            },
            deadline_seconds: 60,
            target: RuleTargetInput {
                worktree_id: Uuid::new_v4(),
                work_item_id: Uuid::new_v4(),
                execution_profile_id: Uuid::new_v4(),
            },
        };
        serde_json::to_string(&body).expect("route test request body should serialize")
    }

    #[test]
    fn schedule_rule_request_cannot_select_run_as_actor() {
        let mut body: Value = serde_json::from_str(&route_test_body(None))
            .expect("valid schedule write body should parse");
        body["run_as_actor_id"] = json!(Uuid::new_v4());
        assert!(serde_json::from_value::<RuleWriteBody>(body).is_err());
    }

    #[test]
    fn overlap_and_misfire_policies_map_to_database_columns() {
        assert_eq!(
            overlap_columns(ScheduleOverlapPolicyV1::AllowBounded { max_active: 8 }),
            ("allow_bounded", 8)
        );
        assert_eq!(
            misfire_columns(ScheduleMisfirePolicyV1::CatchUp {
                max_occurrences: 16
            }),
            ("catch_up", 16)
        );
    }

    #[test]
    fn task_execution_rules_do_not_grant_agent_role_schedule_authority() {
        assert!(require_schedule_writer("tenant_admin").is_ok());
        assert!(require_schedule_writer("developer").is_ok());
        assert!(require_schedule_writer("project_admin").is_ok());
        assert!(require_schedule_writer("viewer").is_err());
        assert!(require_schedule_writer("agent").is_err());
        assert!(require_schedule_writer("").is_err());
    }

    #[tokio::test]
    #[ignore = "requires the disposable canonical directory ACL fixture from phase9f3_schedule.py"]
    async fn schedule_run_as_authorization_rechecks_canonical_directory_grants() {
        use sqlx::postgres::PgPoolOptions;

        let database_url = std::env::var("STAR_SCHEDULE_ACL_DATABASE_URL")
            .expect("the Schedule runner must provide its disposable ACL database URL");
        assert!(
            database_url.starts_with("postgresql://schedule_runtime@127.0.0.1:")
                && database_url.ends_with("/postgres"),
            "the ACL test only permits the runner's loopback schedule_runtime database"
        );
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("the disposable canonical ACL database must be reachable");
        let has_directory_update_privileges = sqlx::query_scalar::<_, bool>(
                r#"SELECT has_any_column_privilege(current_user, 'permission.project_role_binding', 'UPDATE')
                       OR has_any_column_privilege(current_user, 'permission.cloud_branch_role_binding', 'UPDATE')
                       OR has_any_column_privilege(current_user, 'permission.engineering_run_role_binding', 'UPDATE')
                       OR has_any_column_privilege(current_user, 'scm.cloud_branch', 'UPDATE')
                       OR has_any_column_privilege(current_user, 'scm.cloud_branch_revision', 'UPDATE')
                       OR has_any_column_privilege(current_user, 'multica.engineering_run', 'UPDATE')
                       OR has_any_column_privilege(current_user, 'multica.engineering_run_revision', 'UPDATE')"#,
            )
            .fetch_one(&pool)
            .await
            .expect("runtime role column privileges should be queryable");
        assert!(
            !has_directory_update_privileges,
            "runtime role must be read-only on canonical directory authorization tables"
        );
        let tenant_id = Uuid::parse_str("21000000-0000-4000-8000-000000000001").unwrap();
        let actor_id = Uuid::parse_str("22000000-0000-4000-8000-000000000001").unwrap();
        let cases = [
            (
                "active grants",
                "23000000-0000-4000-8000-000000000001",
                "25000000-0000-4000-8000-000000000001",
                true,
            ),
            (
                "revoked Project grant",
                "23000000-0000-4000-8000-000000000002",
                "25000000-0000-4000-8000-000000000002",
                false,
            ),
            (
                "revoked Branch grant",
                "23000000-0000-4000-8000-000000000003",
                "25000000-0000-4000-8000-000000000003",
                false,
            ),
            (
                "revoked Run grant",
                "23000000-0000-4000-8000-000000000004",
                "25000000-0000-4000-8000-000000000004",
                false,
            ),
            (
                "paused Engineering Run",
                "23000000-0000-4000-8000-000000000005",
                "25000000-0000-4000-8000-000000000005",
                false,
            ),
        ];
        let mut tx = pool.begin().await.expect("begin ACL test transaction");
        set_tenant(&mut tx, tenant_id)
            .await
            .expect("set tenant for fixture visibility assertions");
        sqlx::query("SELECT set_config('app.actor_id',$1,true)")
            .bind(actor_id.to_string())
            .execute(&mut *tx)
            .await
            .expect("set actor for fixture visibility assertions");
        let visible_authority_parts = sqlx::query_scalar::<_, Vec<bool>>(
            r#"SELECT ARRAY[
                   EXISTS (SELECT 1 FROM multica.engineering_run r
                           WHERE r.tenant_id=$1 AND r.project_id=$3 AND r.engineering_run_id=$4),
                   EXISTS (SELECT 1 FROM multica.engineering_run r
                           JOIN multica.engineering_run_revision rv
                             ON rv.tenant_id=r.tenant_id AND rv.engineering_run_id=r.engineering_run_id
                            AND rv.valid_from<=now() AND rv.valid_to IS NULL AND rv.state<>'archived'
                           WHERE r.tenant_id=$1 AND r.project_id=$3 AND r.engineering_run_id=$4),
                   EXISTS (SELECT 1 FROM multica.engineering_run r
                           JOIN scm.cloud_branch b
                             ON b.tenant_id=r.tenant_id AND b.branch_id=r.branch_id
                            AND b.project_id=r.project_id AND b.repository_id=r.repository_id
                           WHERE r.tenant_id=$1 AND r.project_id=$3 AND r.engineering_run_id=$4),
                   EXISTS (SELECT 1 FROM multica.engineering_run r
                           JOIN scm.cloud_branch b
                             ON b.tenant_id=r.tenant_id AND b.branch_id=r.branch_id
                            AND b.project_id=r.project_id AND b.repository_id=r.repository_id
                           JOIN scm.cloud_branch_revision bv
                             ON bv.tenant_id=b.tenant_id AND bv.branch_id=b.branch_id
                            AND bv.valid_from<=now() AND bv.valid_to IS NULL AND bv.state='active'
                           WHERE r.tenant_id=$1 AND r.project_id=$3 AND r.engineering_run_id=$4),
                   EXISTS (SELECT 1 FROM permission.project_role_binding p
                           WHERE p.tenant_id=$1 AND p.project_id=$3 AND p.user_id=$2
                             AND p.valid_from<=now() AND p.valid_to IS NULL),
                   EXISTS (SELECT 1 FROM multica.engineering_run r
                           JOIN scm.cloud_branch b
                             ON b.tenant_id=r.tenant_id AND b.branch_id=r.branch_id
                            AND b.project_id=r.project_id AND b.repository_id=r.repository_id
                           JOIN permission.cloud_branch_role_binding bg
                             ON bg.tenant_id=b.tenant_id AND bg.branch_id=b.branch_id
                            AND bg.user_id=$2 AND bg.valid_from<=now() AND bg.valid_to IS NULL
                           WHERE r.tenant_id=$1 AND r.project_id=$3 AND r.engineering_run_id=$4),
                   EXISTS (SELECT 1 FROM permission.engineering_run_role_binding rg
                           WHERE rg.tenant_id=$1 AND rg.engineering_run_id=$4 AND rg.user_id=$2
                             AND rg.valid_from<=now() AND rg.valid_to IS NULL)
               ]"#,
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(Uuid::parse_str(cases[0].1).unwrap())
        .bind(Uuid::parse_str(cases[0].2).unwrap())
        .fetch_one(&mut *tx)
        .await
        .expect("canonical authorization fixture rows should be queryable");
        assert_eq!(
            visible_authority_parts,
            vec![true; 7],
            "active ACL fixture components should all be visible to the runtime role"
        );
        tx.rollback()
            .await
            .expect("release canonical query diagnostic locks");
        for (scenario, project_id, run_id, should_authorize) in cases {
            let project_id = Uuid::parse_str(project_id).unwrap();
            let run_id = Uuid::parse_str(run_id).unwrap();
            let mut tx = pool.begin().await.expect("begin isolated ACL scenario");
            set_tenant(&mut tx, tenant_id)
                .await
                .expect("set tenant for isolated ACL scenario");
            sqlx::query("SELECT set_config('app.actor_id',$1,true)")
                .bind(actor_id.to_string())
                .execute(&mut *tx)
                .await
                .expect("set actor for isolated ACL scenario");
            let result =
                authorize_schedule_run_as(&mut tx, tenant_id, actor_id, project_id, run_id).await;
            assert_eq!(
                result.is_ok(),
                should_authorize,
                "canonical run-as authorization mismatch for {scenario}: {:?}",
                result.as_ref().err()
            );
            if should_authorize {
                let scope = result.expect("active current grants should authorize run-as");
                assert_eq!(scope.project_id, project_id);
                assert_eq!(scope.engineering_run_id, run_id);
            }
            tx.rollback()
                .await
                .expect("release isolated ACL scenario locks");
        }
        pool.close().await;
    }

    #[tokio::test]
    async fn schedule_rule_responses_are_private() {
        let success = private_json(StatusCode::OK, json!({"rule": {}}));
        assert_eq!(
            success.headers().get(header::CACHE_CONTROL).unwrap(),
            &HeaderValue::from_static("private, no-store")
        );
        assert_eq!(
            success.headers().get(header::VARY).unwrap(),
            &HeaderValue::from_static("Authorization")
        );

        let error = private_response_middleware(GroupApiError::not_found().into_response()).await;
        assert_eq!(
            error.headers().get(header::CACHE_CONTROL).unwrap(),
            &HeaderValue::from_static("private, no-store")
        );
        assert_eq!(
            error.headers().get(header::VARY).unwrap(),
            &HeaderValue::from_static("Authorization")
        );
    }

    #[tokio::test]
    async fn schedule_rule_routes_are_mounted_and_require_authentication() {
        use axum::{
            body::Body,
            http::{header, Method, Request},
        };
        use sqlx::postgres::PgPoolOptions;
        use tower::ServiceExt;

        let jwt = Arc::new(crate::auth::JwtConfig {
            private_key_pem: String::new(),
            public_key_pem: String::new(),
            issuer: "https://api.star.local".to_owned(),
            audience: "star-api-rest".to_owned(),
            ttl_seconds: 3600,
        });
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://localhost/unavailable-for-schedule-route-test")
            .expect("lazy test pool configuration should parse");
        let app = super::super::build_group_router(super::super::GroupApiState::new(jwt, pool));
        let collection = "/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/automation/schedule-rules";
        let item = format!("{collection}/00000000-0000-0000-0000-000000000003");
        let cases = [
            (Method::GET, collection.to_owned(), false),
            (Method::GET, item.clone(), false),
            (Method::POST, collection.to_owned(), true),
            (Method::PUT, item, true),
        ];

        for (method, path, has_json_body) in cases {
            let mut request = Request::builder().method(method).uri(path);
            let body = if has_json_body {
                request = request.header(header::CONTENT_TYPE, "application/json");
                Body::from("{}")
            } else {
                Body::empty()
            };
            let response = app
                .clone()
                .oneshot(request.body(body).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL).unwrap(),
                &HeaderValue::from_static("private, no-store")
            );
            assert_eq!(
                response.headers().get(header::VARY).unwrap(),
                &HeaderValue::from_static("Authorization")
            );
        }
    }

    #[tokio::test]
    async fn schedule_rule_routes_reject_missing_scopes_before_database_access() {
        use axum::{
            body::Body,
            http::{header, Method, Request},
        };
        use sqlx::postgres::PgPoolOptions;
        use tower::ServiceExt;

        let jwt = route_test_jwt_config();
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://localhost/unavailable-for-schedule-route-test")
            .expect("lazy test pool configuration should parse");
        let app =
            super::super::build_group_router(super::super::GroupApiState::new(jwt.clone(), pool));
        let collection = "/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/automation/schedule-rules";
        let item = format!("{collection}/00000000-0000-0000-0000-000000000003");
        let cases = [
            (Method::GET, collection.to_owned(), false, None),
            (Method::GET, item.clone(), false, None),
            (Method::POST, collection.to_owned(), true, None),
            (Method::PUT, item, true, Some(1)),
        ];

        for (method, path, has_json_body, expected_version) in cases {
            let token_scope = if has_json_body {
                "work-item:read"
            } else {
                "work-item:write"
            };
            let token = route_test_token(&jwt, token_scope);
            let mut request = Request::builder()
                .method(method)
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {token}"));
            let body = if has_json_body {
                request = request
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "route-test-scope-denied");
                Body::from(route_test_body(expected_version))
            } else {
                Body::empty()
            };
            let response = app
                .clone()
                .oneshot(request.body(body).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL).unwrap(),
                &HeaderValue::from_static("private, no-store")
            );
            assert_eq!(
                response.headers().get(header::VARY).unwrap(),
                &HeaderValue::from_static("Authorization")
            );
        }
    }

    #[tokio::test]
    async fn schedule_rule_write_body_limit_is_enforced() {
        use axum::{
            body::Body,
            http::{header, Request},
        };
        use sqlx::postgres::PgPoolOptions;
        use tower::ServiceExt;

        let jwt = route_test_jwt_config();
        let token = route_test_token(&jwt, "work-item:write");
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://localhost/unavailable-for-schedule-route-test")
            .expect("lazy test pool configuration should parse");
        let app = super::super::build_group_router(super::super::GroupApiState::new(jwt, pool));
        let body = vec![b'x'; 16 * 1024 + 1];
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/projects/00000000-0000-0000-0000-000000000001/engineering-runs/00000000-0000-0000-0000-000000000002/automation/schedule-rules")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            &HeaderValue::from_static("private, no-store")
        );
        assert_eq!(
            response.headers().get(header::VARY).unwrap(),
            &HeaderValue::from_static("Authorization")
        );
    }
}
