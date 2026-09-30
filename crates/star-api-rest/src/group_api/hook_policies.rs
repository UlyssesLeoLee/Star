//! CYPHER STRUCTURE MANIFEST
//! CREATE
//!   (f:File {name:"hook_policies.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"hook_policies",type:"module",language:"rust"}),
//!   (pr:Class {name:"PolicyRow",type:"class",language:"rust"}),
//!   (dr:Class {name:"DraftRow",type:"class",language:"rust"}),
//!   (ar:Class {name:"AuditRow",type:"class",language:"rust"}),
//!   (db:Class {name:"DraftBody",type:"class",language:"rust"}),
//!   (pb:Class {name:"PublishBody",type:"class",language:"rust"}),
//!   (rb:Class {name:"RollbackBody",type:"class",language:"rust"}),
//!   (sc:Class {name:"PolicyScope",type:"class",language:"rust"}),
//!   (ac:Class {name:"Access",type:"class",language:"rust"}),
//!   (stage:Class {name:"RebaseStageRow",type:"class",language:"rust"}),
//!   (limit:Variable {name:"PROJECT_REBASE_BATCH_SIZE",type:"variable",language:"rust"}),
//!   (request_limit:Variable {name:"MAX_POLICY_REQUEST_BYTES",type:"variable",language:"rust"}),
//!   (rt:Function {name:"router",type:"function",language:"rust"}),
//!   (gp:Function {name:"get_project_policy",type:"function",language:"rust"}),
//!   (gw:Function {name:"get_worktree_effective_policy",type:"function",language:"rust"}),
//!   (sd:Function {name:"save_project_draft",type:"function",language:"rust"}),
//!   (sw:Function {name:"save_worktree_draft",type:"function",language:"rust"}),
//!   (pd:Function {name:"publish_project_draft",type:"function",language:"rust"}),
//!   (pw:Function {name:"publish_worktree_draft",type:"function",language:"rust"}),
//!   (rp:Function {name:"rollback_project_policy",type:"function",language:"rust"}),
//!   (rw:Function {name:"rollback_worktree_policy",type:"function",language:"rust"}),
//!   (as_str:Function {name:"PolicyScope::as_str",type:"function",language:"rust"}),
//!   (read:Function {name:"read_policy_scope",type:"function",language:"rust"}),
//!   (read_tx:Function {name:"read_policy_in_tx",type:"function",language:"rust"}),
//!   (save:Function {name:"save_draft",type:"function",language:"rust"}),
//!   (publish:Function {name:"publish_draft",type:"function",language:"rust"}),
//!   (publish_project:Function {name:"publish_project_document",type:"function",language:"rust"}),
//!   (publish_worktree:Function {name:"publish_worktree_document",type:"function",language:"rust"}),
//!   (rollback:Function {name:"rollback_policy",type:"function",language:"rust"}),
//!   (rebase:Function {name:"stage_project_overlay_rebases",type:"function",language:"rust"}),
//!   (replay:Function {name:"publish_staged_overlays",type:"function",language:"rust"}),
//!   (auth:Function {name:"authorize_scope",type:"function",language:"rust"}),
//!   (load:Function {name:"load_current_policy",type:"function",language:"rust"}),
//!   (draft:Function {name:"load_current_draft",type:"function",language:"rust"}),
//!   (audit:Function {name:"load_audit",type:"function",language:"rust"}),
//!   (decode:Function {name:"decode_document",type:"function",language:"rust"}),
//!   (parse_body:Function {name:"parse_draft_body",type:"function",language:"rust"}),
//!   (parse_id:Function {name:"parse_id",type:"function",language:"rust"}),
//!   (validate_draft:Function {name:"validate_draft_document",type:"function",language:"rust"}),
//!   (validate_scope:Function {name:"validate_document_scope",type:"function",language:"rust"}),
//!   (verify:Function {name:"verify_stored_policy",type:"function",language:"rust"}),
//!   (normalize_project:Function {name:"normalize_project_document",type:"function",language:"rust"}),
//!   (normalize_worktree:Function {name:"normalize_worktree_document",type:"function",language:"rust"}),
//!   (insert_policy:Function {name:"insert_policy",type:"function",language:"rust"}),
//!   (insert_audit:Function {name:"insert_audit",type:"function",language:"rust"}),
//!   (tests:Module {name:"tests",type:"module",language:"rust"}),
//!   (test_doc:Function {name:"policy_document",type:"function",language:"rust"}),
//!   (test_project:Function {name:"project_normalization_recomputes_digest_for_next_revision",type:"function",language:"rust"}),
//!   (test_worktree:Function {name:"worktree_normalization_pins_the_current_project_revision",type:"function",language:"rust"}),
//!   (test_scope:Function {name:"scope_validation_rejects_a_document_from_another_worktree",type:"function",language:"rust"}),
//!   (test_size:Function {name:"decode_rejects_documents_over_the_policy_bound",type:"function",language:"rust"}),
//!   (test_request_size:Function {name:"draft_request_parser_rejects_oversized_envelopes",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(pr),(m)-[:CONTAINS]->(dr),(m)-[:CONTAINS]->(ar),
//!   (m)-[:CONTAINS]->(db),(m)-[:CONTAINS]->(pb),(m)-[:CONTAINS]->(rb),(m)-[:CONTAINS]->(sc),
//!   (m)-[:CONTAINS]->(ac),(m)-[:CONTAINS]->(stage),(m)-[:CONTAINS]->(limit),(m)-[:CONTAINS]->(request_limit),(m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(gp),
//!   (m)-[:CONTAINS]->(gw),(m)-[:CONTAINS]->(sd),(m)-[:CONTAINS]->(sw),(m)-[:CONTAINS]->(pd),
//!   (m)-[:CONTAINS]->(pw),(m)-[:CONTAINS]->(rp),(m)-[:CONTAINS]->(rw),(m)-[:CONTAINS]->(read),
//!   (m)-[:CONTAINS]->(as_str),(m)-[:CONTAINS]->(read_tx),(m)-[:CONTAINS]->(validate_draft),(m)-[:CONTAINS]->(validate_scope),(m)-[:CONTAINS]->(parse_body),(m)-[:CONTAINS]->(parse_id),
//!   (m)-[:CONTAINS]->(save),(m)-[:CONTAINS]->(publish),(m)-[:CONTAINS]->(publish_project),
//!   (m)-[:CONTAINS]->(publish_worktree),(m)-[:CONTAINS]->(rollback),(m)-[:CONTAINS]->(rebase),
//!   (m)-[:CONTAINS]->(auth),(m)-[:CONTAINS]->(load),(m)-[:CONTAINS]->(draft),(m)-[:CONTAINS]->(audit),
//!   (m)-[:CONTAINS]->(decode),(m)-[:CONTAINS]->(verify),(m)-[:CONTAINS]->(normalize_project),
//!   (m)-[:CONTAINS]->(normalize_worktree),(m)-[:CONTAINS]->(insert_policy),(m)-[:CONTAINS]->(insert_audit),
//!   (m)-[:CONTAINS]->(tests),(tests)-[:CONTAINS]->(test_doc),(tests)-[:CONTAINS]->(test_project),(tests)-[:CONTAINS]->(test_worktree),(tests)-[:CONTAINS]->(test_scope),(tests)-[:CONTAINS]->(test_size),(tests)-[:CONTAINS]->(test_request_size),
//!   (rt)-[:CALLS]->(gp),(rt)-[:CALLS]->(gw),(rt)-[:CALLS]->(sd),(rt)-[:CALLS]->(sw),
//!   (rt)-[:CALLS]->(pd),(rt)-[:CALLS]->(pw),(rt)-[:CALLS]->(rp),(rt)-[:CALLS]->(rw),
//!   (sc)-[:HAS_METHOD]->(as_str),(gp)-[:CALLS]->(parse_id),(gp)-[:CALLS]->(read),(gw)-[:CALLS]->(parse_id),(gw)-[:CALLS]->(auth),(gw)-[:CALLS]->(read_tx),
//!   (sd)-[:CALLS]->(parse_id),(sd)-[:CALLS]->(parse_body),(sd)-[:CALLS]->(save),(sw)-[:CALLS]->(parse_id),(sw)-[:CALLS]->(parse_body),(sw)-[:CALLS]->(save),
//!   (pd)-[:CALLS]->(publish),(pw)-[:CALLS]->(publish),(rp)-[:CALLS]->(rollback),(rw)-[:CALLS]->(rollback),
//!   (read)-[:CALLS]->(auth),(read)-[:CALLS]->(read_tx),(read_tx)-[:CALLS]->(load),(read_tx)-[:CALLS]->(draft),(read_tx)-[:CALLS]->(audit),(read_tx)-[:CALLS]->(verify),(read_tx)-[:CALLS]->(decode),(read_tx)-[:CALLS]->(validate_scope),
//!   (save)-[:CALLS]->(auth),(save)-[:CALLS]->(load),(save)-[:CALLS]->(draft),(save)-[:CALLS]->(validate_draft),
//!   (validate_draft)-[:CALLS]->(validate_scope),(validate_draft)-[:CALLS]->(verify),
//!   (save)-[:CALLS]->(insert_audit),(publish)-[:CALLS]->(auth),(publish)-[:CALLS]->(load),
//!   (publish)-[:CALLS]->(draft),(publish)-[:CALLS]->(decode),(publish)-[:CALLS]->(publish_project),
//!   (publish)-[:CALLS]->(publish_worktree),(rollback)-[:CALLS]->(decode),
//!   (publish_project)-[:CALLS]->(rebase),(publish_project)-[:CALLS]->(replay),(publish_project)-[:CALLS]->(normalize_project),
//!   (publish_project)-[:CALLS]->(insert_policy),(publish_project)-[:CALLS]->(insert_audit),
//!   (publish_worktree)-[:CALLS]->(normalize_worktree),(publish_worktree)-[:CALLS]->(insert_policy),
//!   (publish_worktree)-[:CALLS]->(insert_audit),(rollback)-[:CALLS]->(auth),(rollback)-[:CALLS]->(load),(rollback)-[:CALLS]->(draft),
//!   (rollback)-[:CALLS]->(verify),(rollback)-[:CALLS]->(publish_project),(rollback)-[:CALLS]->(publish_worktree),
//!   (rebase)-[:CALLS]->(verify),(replay)-[:CALLS]->(decode),(replay)-[:CALLS]->(insert_policy),(replay)-[:CALLS]->(insert_audit),(verify)-[:CALLS]->(decode),
//!   (parse_body)-[:USES]->(request_limit),
//!   (test_project)-[:CALLS]->(test_doc),(test_project)-[:CALLS]->(normalize_project),
//!   (test_worktree)-[:CALLS]->(test_doc),(test_worktree)-[:CALLS]->(normalize_worktree),
//!   (test_scope)-[:CALLS]->(test_doc),(test_scope)-[:CALLS]->(validate_scope),
//!   (test_size)-[:CALLS]->(decode),(test_request_size)-[:CALLS]->(parse_body),(test_request_size)-[:USES]->(request_limit),
//!   (publish_project)-[:USES]->(limit),(rebase)-[:USES]->(limit),(rt)-[:USES]->(request_limit);

//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"hook_policies",type:"module"}),(load:Function {name:"load_current_policy",type:"function"}),(verify:Function {name:"verify_stored_policy",type:"function"});
//! CREATE (effective:Function {name:"load_verified_effective_snapshot",type:"function",language:"rust",visibility:"pub(super)",complexity:"moderate"});
//! CREATE (m)-[:CONTAINS]->(effective),(effective)-[:CALLS]->(load),(effective)-[:CALLS]->(verify);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"hook_policies",type:"module"}),(rt:Function {name:"router",type:"function"});
//! CREATE (eventsQuery:Class {name:"HookEventsQuery",type:"class",language:"rust"}),(eventCursor:Class {name:"HookEventCursor",type:"class",language:"rust"}),(eventProjection:Class {name:"HookEventProjection",type:"class",language:"rust"}),(projectKey:Variable {name:"project_id",type:"variable",language:"rust"}),(occurredKey:Variable {name:"occurred_at",type:"variable",language:"rust"}),(eventKey:Variable {name:"event_id",type:"variable",language:"rust"}),(listEvents:Function {name:"list_hook_events",type:"function",language:"rust",visibility:"private",complexity:"moderate"}),(decodeEventCursor:Function {name:"decode_hook_event_cursor",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(encodeEventCursor:Function {name:"encode_hook_event_cursor",type:"function",language:"rust",visibility:"private",complexity:"simple"});
//! CREATE (m)-[:CONTAINS]->(eventsQuery),(m)-[:CONTAINS]->(eventCursor),(m)-[:CONTAINS]->(eventProjection),(m)-[:CONTAINS]->(listEvents),(m)-[:CONTAINS]->(decodeEventCursor),(m)-[:CONTAINS]->(encodeEventCursor),(rt)-[:CALLS]->(listEvents),(listEvents)-[:CALLS]->(decodeEventCursor),(listEvents)-[:CALLS]->(encodeEventCursor),(listEvents)-[:USES]->(eventProjection),(eventCursor)-[:USES]->(projectKey),(eventCursor)-[:USES]->(occurredKey),(eventCursor)-[:USES]->(eventKey);
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::header::{CACHE_CONTROL, VARY},
    routing::{get, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use domain_hook::{HookPolicyDocument, MAX_POLICY_DOCUMENT_BYTES};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, work_items::authorize_worktree,
    AuthenticatedUser, GroupApiError, GroupApiState,
};

// Each batch holds at most 16 verified 64 KiB policy documents in Rust memory.
const PROJECT_REBASE_BATCH_SIZE: usize = 16;
const MAX_POLICY_REQUEST_BYTES: usize = MAX_POLICY_DOCUMENT_BYTES + 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyScope {
    Project,
    Worktree,
}

impl PolicyScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Worktree => "worktree",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
struct PolicyRow {
    policy_set_id: Uuid,
    project_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    policy_version: i64,
    schema_version: i16,
    evaluator_api_version: i16,
    digest: Vec<u8>,
    policy_document: Value,
    inherited_project_policy_set_id: Option<Uuid>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
struct DraftRow {
    draft_id: Uuid,
    base_policy_set_id: Option<Uuid>,
    inherited_project_policy_set_id: Option<Uuid>,
    draft_version: i64,
    policy_document: Value,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
struct AuditRow {
    event_id: Uuid,
    event_type: String,
    policy_set_id: Option<Uuid>,
    draft_id: Option<Uuid>,
    policy_version: Option<i64>,
    correlation_id: Uuid,
    details: Value,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftBody {
    expected_draft_version: Option<i64>,
    expected_current_policy_set_id: Option<Uuid>,
    correlation_id: Uuid,
    policy_document: HookPolicyDocument,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishBody {
    expected_draft_version: i64,
    correlation_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RollbackBody {
    target_policy_set_id: Uuid,
    expected_current_policy_set_id: Uuid,
    correlation_id: Uuid,
}

#[derive(Debug, FromRow)]
struct RebaseStageRow {
    worktree_id: Uuid,
    previous_policy_set_id: Uuid,
    policy_document: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HookEventsQuery {
    limit: Option<i64>,
    cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookEventCursor {
    version: u8,
    project_id: Uuid,
    occurred_at: DateTime<Utc>,
    event_id: Uuid,
}

#[derive(Debug, FromRow, Serialize)]
struct HookEventProjection {
    event_id: Uuid,
    worktree_id: Option<Uuid>,
    work_item_id: Option<Uuid>,
    run_id: Option<Uuid>,
    actor_id: Uuid,
    correlation_id: Uuid,
    source_kind: String,
    hook_phase: String,
    hook_decision: String,
    hook_reason_code: String,
    matched_rule_id: Option<Uuid>,
    project_policy_version: Option<i64>,
    worktree_policy_version: Option<i64>,
    evaluator_api_version: i16,
    policy_digest: Option<String>,
    evaluated_condition_count: i32,
    duration_ms: i64,
    timed_out: bool,
    occurred_at: DateTime<Utc>,
}

const MAX_HOOK_EVENT_CURSOR_BYTES: usize = 512;

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/hook-policy",
            get(get_project_policy),
        )
        .route(
            "/api/v1/projects/{project_id}/hook-events",
            get(list_hook_events),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/hook-policy/effective",
            get(get_worktree_effective_policy),
        )
        .route(
            "/api/v1/projects/{project_id}/hook-policy/draft",
            put(save_project_draft),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/hook-policy/draft",
            put(save_worktree_draft),
        )
        .route(
            "/api/v1/projects/{project_id}/hook-policy/publish",
            post(publish_project_draft),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/hook-policy/publish",
            post(publish_worktree_draft),
        )
        .route(
            "/api/v1/projects/{project_id}/hook-policy/rollback",
            post(rollback_project_policy),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/hook-policy/rollback",
            post(rollback_worktree_policy),
        )
        .layer(DefaultBodyLimit::max(MAX_POLICY_REQUEST_BYTES))
}

async fn list_hook_events(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    Query(query): Query<HookEventsQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    let project_id = parse_id(&project_id)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100) as usize;
    let cursor = query
        .cursor
        .as_deref()
        .map(decode_hook_event_cursor)
        .transpose()?;
    if cursor
        .as_ref()
        .is_some_and(|cursor| cursor.project_id != project_id)
    {
        return Err(GroupApiError::invalid_request("cursor_scope_mismatch"));
    }

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    authorize_scope(
        &mut tx,
        &actor,
        PolicyScope::Project,
        Some(project_id),
        None,
        Access::Read,
    )
    .await?;
    let mut events = sqlx::query_as::<_, HookEventProjection>(
        r#"SELECT event_id, worktree_id, work_item_id, run_id, actor_id,
                  correlation_id, source_kind, hook_phase, hook_decision,
                  hook_reason_code, matched_rule_id, project_policy_version,
                  worktree_policy_version, evaluator_api_version, policy_digest,
                  evaluated_condition_count, duration_ms, timed_out, occurred_at
           FROM multica.hook_execution_event
           WHERE tenant_id = $1 AND project_id = $2
             AND ($3::TIMESTAMPTZ IS NULL OR (occurred_at, event_id) < ($3, $4))
           ORDER BY occurred_at DESC, event_id DESC
           LIMIT $5"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(cursor.as_ref().map(|value| value.occurred_at))
    .bind(cursor.as_ref().map(|value| value.event_id))
    .bind((limit + 1) as i64)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let has_more = events.len() > limit;
    events.truncate(limit);
    let next_cursor = if has_more {
        events
            .last()
            .map(|event| encode_hook_event_cursor(project_id, event))
            .transpose()?
    } else {
        None
    };
    let body = json!({
        "events": events,
        "next_cursor": next_cursor,
        "coverage": {
            "scope": "hook_execution_event_ledger",
            "status": "partial",
            "reported_percentage": Value::Null,
            "instrumented_phases": ["worktree_archive"],
            "not_yet_instrumented_phases": ["run_admission", "tool", "validation", "review", "worktree_cleanup", "after_commit"],
            "note": "Uninstrumented phases are unknown, not zero-risk or zero-volume."
        }
    });
    Ok((
        [(CACHE_CONTROL, "no-store"), (VARY, "Authorization")],
        Json(body),
    ))
}

fn decode_hook_event_cursor(value: &str) -> Result<HookEventCursor, GroupApiError> {
    if value.is_empty() || value.len() > MAX_HOOK_EVENT_CURSOR_BYTES {
        return Err(GroupApiError::invalid_request("invalid_hook_event_cursor"));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_event_cursor"))?;
    let cursor: HookEventCursor = serde_json::from_slice(&bytes)
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_event_cursor"))?;
    if cursor.version != 1 {
        return Err(GroupApiError::invalid_request(
            "unsupported_hook_event_cursor",
        ));
    }
    Ok(cursor)
}

fn encode_hook_event_cursor(
    project_id: Uuid,
    event: &HookEventProjection,
) -> Result<String, GroupApiError> {
    let cursor = HookEventCursor {
        version: 1,
        project_id,
        occurred_at: event.occurred_at,
        event_id: event.event_id,
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| GroupApiError::internal())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

async fn get_project_policy(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, GroupApiError> {
    let project_id = parse_id(&project_id)?;
    read_policy_scope(&state, &actor, PolicyScope::Project, project_id, None).await
}

async fn get_worktree_effective_policy(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = parse_id(&worktree_id)?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let project_id = authorize_scope(
        &mut tx,
        &actor,
        PolicyScope::Worktree,
        None,
        Some(worktree_id),
        Access::Read,
    )
    .await?;
    let result = read_policy_in_tx(
        &mut tx,
        &actor,
        PolicyScope::Worktree,
        project_id,
        Some(worktree_id),
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(result))
}

async fn save_project_draft(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    bytes: Bytes,
) -> Result<Json<Value>, GroupApiError> {
    let project_id = parse_id(&project_id)?;
    let body = parse_draft_body(&bytes)?;
    save_draft(&state, &actor, PolicyScope::Project, project_id, None, body).await
}

async fn save_worktree_draft(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    bytes: Bytes,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = parse_id(&worktree_id)?;
    let body = parse_draft_body(&bytes)?;
    save_draft(
        &state,
        &actor,
        PolicyScope::Worktree,
        Uuid::nil(),
        Some(worktree_id),
        body,
    )
    .await
}

async fn publish_project_draft(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    Json(body): Json<PublishBody>,
) -> Result<Json<Value>, GroupApiError> {
    let project_id = parse_id(&project_id)?;
    publish_draft(&state, &actor, PolicyScope::Project, project_id, None, body).await
}

async fn publish_worktree_draft(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    Json(body): Json<PublishBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = parse_id(&worktree_id)?;
    publish_draft(
        &state,
        &actor,
        PolicyScope::Worktree,
        Uuid::nil(),
        Some(worktree_id),
        body,
    )
    .await
}

async fn rollback_project_policy(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    Json(body): Json<RollbackBody>,
) -> Result<Json<Value>, GroupApiError> {
    let project_id = parse_id(&project_id)?;
    rollback_policy(&state, &actor, PolicyScope::Project, project_id, None, body).await
}

async fn rollback_worktree_policy(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    Json(body): Json<RollbackBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = parse_id(&worktree_id)?;
    rollback_policy(
        &state,
        &actor,
        PolicyScope::Worktree,
        Uuid::nil(),
        Some(worktree_id),
        body,
    )
    .await
}

#[derive(Debug, Clone, Copy)]
enum Access {
    Read,
    Write,
    Publish,
}

async fn authorize_scope(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    scope: PolicyScope,
    project_id: Option<Uuid>,
    worktree_id: Option<Uuid>,
    access: Access,
) -> Result<Uuid, GroupApiError> {
    validate_actor(actor)?;
    let required = match access {
        Access::Read => "hook:read",
        Access::Write => "hook:write",
        Access::Publish => "hook:publish",
    };
    require_scope(actor, required)?;
    set_tenant(tx, actor.tenant_id).await?;
    let project_id = match (scope, project_id, worktree_id) {
        (PolicyScope::Project, Some(project_id), None) => project_id,
        (PolicyScope::Worktree, None, Some(worktree_id)) => {
            authorize_worktree(tx, actor, worktree_id).await?.project_id
        }
        _ => return Err(GroupApiError::bad_request()),
    };
    let binding = active_binding(tx, actor, project_id).await?;
    if matches!(access, Access::Publish)
        && !matches!(binding.role.as_str(), "tenant_admin" | "project_admin")
    {
        return Err(GroupApiError::forbidden());
    }
    Ok(project_id)
}

async fn read_policy_scope(
    state: &GroupApiState,
    actor: &super::AuthUser,
    scope: PolicyScope,
    project_id: Uuid,
    worktree_id: Option<Uuid>,
) -> Result<Json<Value>, GroupApiError> {
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    authorize_scope(
        &mut tx,
        actor,
        scope,
        Some(project_id),
        worktree_id,
        Access::Read,
    )
    .await?;
    let result = read_policy_in_tx(&mut tx, actor, scope, project_id, worktree_id).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(result))
}

async fn read_policy_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    scope: PolicyScope,
    project_id: Uuid,
    worktree_id: Option<Uuid>,
) -> Result<Value, GroupApiError> {
    let project = load_current_policy(
        tx,
        actor.tenant_id,
        project_id,
        PolicyScope::Project,
        None,
        false,
    )
    .await?;
    let project_document = project
        .as_ref()
        .map(|row| verify_stored_policy(row, actor.tenant_id, project_id, None))
        .transpose()?;
    let (current, effective) = if scope == PolicyScope::Project {
        (project.clone(), project_document.clone())
    } else {
        let worktree_id = worktree_id.ok_or_else(GroupApiError::bad_request)?;
        let overlay = load_current_policy(
            tx,
            actor.tenant_id,
            project_id,
            PolicyScope::Worktree,
            Some(worktree_id),
            false,
        )
        .await?;
        let overlay_document = overlay
            .as_ref()
            .map(|row| verify_stored_policy(row, actor.tenant_id, project_id, Some(worktree_id)))
            .transpose()?;
        if let (Some(project_row), Some(project_doc), Some(overlay_row), Some(overlay_doc)) = (
            project.as_ref(),
            project_document.as_ref(),
            overlay.as_ref(),
            overlay_document.as_ref(),
        ) {
            if overlay_row.inherited_project_policy_set_id != Some(project_row.policy_set_id)
                || overlay_doc.project_version != project_doc.project_version
                || overlay_doc.project_rules != project_doc.project_rules
            {
                return Err(GroupApiError::internal());
            }
        } else if overlay.is_some() {
            return Err(GroupApiError::internal());
        }
        (
            overlay.clone(),
            overlay_document.or(project_document.clone()),
        )
    };
    let draft = load_current_draft(tx, actor.tenant_id, project_id, scope, worktree_id, false)
        .await?
        .filter(|row| row.expires_at > Utc::now());
    if let Some(draft) = draft.as_ref() {
        let draft_document =
            decode_document(&draft.policy_document).map_err(|_| GroupApiError::internal())?;
        validate_document_scope(&draft_document, actor.tenant_id, project_id, worktree_id)
            .map_err(|_| GroupApiError::internal())?;
        draft_document
            .verify()
            .map_err(|_| GroupApiError::internal())?;
    }
    let draft_is_stale = draft.as_ref().is_some_and(|row| {
        row.base_policy_set_id != current.as_ref().map(|policy| policy.policy_set_id)
            || (scope == PolicyScope::Worktree
                && row.inherited_project_policy_set_id
                    != project.as_ref().map(|policy| policy.policy_set_id))
    });
    let audit = load_audit(tx, actor.tenant_id, project_id, scope, worktree_id).await?;
    Ok(json!({
        "scope": { "kind": scope.as_str(), "project_id": project_id, "worktree_id": worktree_id },
        "policy_set_id": current.as_ref().map(|row| row.policy_set_id),
        "policy_document": current.as_ref().and(effective.as_ref()),
        "effective_policy_document": effective,
        "inherited_project_policy_set_id": current.as_ref().and_then(|row| row.inherited_project_policy_set_id),
        "draft": draft.map(|row| json!({
            "draft_id": row.draft_id,
            "draft_version": row.draft_version,
            "base_policy_set_id": row.base_policy_set_id,
            "inherited_project_policy_set_id": row.inherited_project_policy_set_id,
            "expires_at": row.expires_at,
            "is_stale": draft_is_stale,
            "policy_document": row.policy_document,
        })),
        "audit": audit,
    }))
}

async fn save_draft(
    state: &GroupApiState,
    actor: &super::AuthUser,
    scope: PolicyScope,
    requested_project_id: Uuid,
    worktree_id: Option<Uuid>,
    body: DraftBody,
) -> Result<Json<Value>, GroupApiError> {
    if body.correlation_id.is_nil()
        || body
            .expected_draft_version
            .is_some_and(|version| version < 0)
    {
        return Err(GroupApiError::invalid_request("invalid_hook_draft"));
    }
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let project_id = authorize_scope(
        &mut tx,
        actor,
        scope,
        (scope == PolicyScope::Project).then_some(requested_project_id),
        worktree_id,
        Access::Write,
    )
    .await?;
    let project = load_current_policy(
        &mut tx,
        actor.tenant_id,
        project_id,
        PolicyScope::Project,
        None,
        true,
    )
    .await?;
    let current = if scope == PolicyScope::Project {
        project.clone()
    } else {
        load_current_policy(
            &mut tx,
            actor.tenant_id,
            project_id,
            PolicyScope::Worktree,
            worktree_id,
            true,
        )
        .await?
    };
    if let Some(current) = current.as_ref() {
        verify_stored_policy(
            current,
            actor.tenant_id,
            project_id,
            if scope == PolicyScope::Worktree {
                worktree_id
            } else {
                None
            },
        )?;
    }
    if body.expected_current_policy_set_id != current.as_ref().map(|row| row.policy_set_id) {
        return Err(GroupApiError::conflict("hook_policy_version_conflict"));
    }
    validate_draft_document(
        &body.policy_document,
        actor.tenant_id,
        project_id,
        scope,
        worktree_id,
        project.as_ref(),
        current.as_ref(),
    )?;
    sqlx::query(
        "DELETE FROM multica.hook_policy_draft WHERE tenant_id = $1 AND project_id = $2 AND scope_kind = $3 AND worktree_id IS NOT DISTINCT FROM $4 AND expires_at <= now()",
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(scope.as_str())
    .bind(worktree_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let existing = load_current_draft(
        &mut tx,
        actor.tenant_id,
        project_id,
        scope,
        worktree_id,
        true,
    )
    .await?;
    let (draft_id, draft_version) = match existing {
        Some(row) => {
            if body.expected_draft_version != Some(row.draft_version) {
                return Err(GroupApiError::conflict("hook_draft_version_conflict"));
            }
            let next = row
                .draft_version
                .checked_add(1)
                .ok_or_else(GroupApiError::internal)?;
            sqlx::query(
                r#"UPDATE multica.hook_policy_draft
                   SET base_policy_set_id = $1, inherited_project_policy_set_id = $2,
                       policy_document = $3, draft_version = $4, updated_by = $5,
                       updated_at = now(), expires_at = LEAST(now() + retention_period, created_at + retention_period)
                   WHERE tenant_id = $6 AND draft_id = $7"#,
            )
            .bind(current.as_ref().map(|row| row.policy_set_id))
            .bind(if scope == PolicyScope::Worktree { project.as_ref().map(|row| row.policy_set_id) } else { None })
            .bind(serde_json::to_value(&body.policy_document).map_err(|_| GroupApiError::internal())?)
            .bind(next)
            .bind(actor.user_id)
            .bind(actor.tenant_id)
            .bind(row.draft_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
            (row.draft_id, next)
        }
        None => {
            if body.expected_draft_version.unwrap_or(0) != 0 {
                return Err(GroupApiError::conflict("hook_draft_version_conflict"));
            }
            let row = sqlx::query_as::<_, (Uuid, i64)>(
                r#"INSERT INTO multica.hook_policy_draft
                   (tenant_id, project_id, scope_kind, worktree_id, base_policy_set_id,
                    inherited_project_policy_set_id, policy_document, created_by, updated_by)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
                   RETURNING draft_id, draft_version"#,
            )
            .bind(actor.tenant_id)
            .bind(project_id)
            .bind(scope.as_str())
            .bind(worktree_id)
            .bind(current.as_ref().map(|row| row.policy_set_id))
            .bind(if scope == PolicyScope::Worktree {
                project.as_ref().map(|row| row.policy_set_id)
            } else {
                None
            })
            .bind(
                serde_json::to_value(&body.policy_document)
                    .map_err(|_| GroupApiError::internal())?,
            )
            .bind(actor.user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
            row
        }
    };
    insert_audit(
        &mut tx,
        actor,
        project_id,
        scope,
        worktree_id,
        None,
        Some(draft_id),
        None,
        "draft_saved",
        body.correlation_id,
        json!({ "draft_version": draft_version }),
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(
        json!({ "draft_id": draft_id, "draft_version": draft_version }),
    ))
}

async fn publish_draft(
    state: &GroupApiState,
    actor: &super::AuthUser,
    scope: PolicyScope,
    requested_project_id: Uuid,
    worktree_id: Option<Uuid>,
    body: PublishBody,
) -> Result<Json<Value>, GroupApiError> {
    if body.expected_draft_version <= 0 || body.correlation_id.is_nil() {
        return Err(GroupApiError::invalid_request("invalid_hook_publish"));
    }
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let project_id = authorize_scope(
        &mut tx,
        actor,
        scope,
        (scope == PolicyScope::Project).then_some(requested_project_id),
        worktree_id,
        Access::Publish,
    )
    .await?;
    let current_project = load_current_policy(
        &mut tx,
        actor.tenant_id,
        project_id,
        PolicyScope::Project,
        None,
        true,
    )
    .await?;
    let current_scope = if scope == PolicyScope::Project {
        current_project.clone()
    } else {
        load_current_policy(
            &mut tx,
            actor.tenant_id,
            project_id,
            PolicyScope::Worktree,
            worktree_id,
            true,
        )
        .await?
    };
    let draft = load_current_draft(
        &mut tx,
        actor.tenant_id,
        project_id,
        scope,
        worktree_id,
        true,
    )
    .await?
    .ok_or_else(GroupApiError::not_found)?;
    if draft.expires_at <= Utc::now() {
        return Err(GroupApiError::conflict("hook_draft_expired"));
    }
    if draft.draft_version != body.expected_draft_version
        || draft.base_policy_set_id != current_scope.as_ref().map(|row| row.policy_set_id)
        || (scope == PolicyScope::Worktree
            && draft.inherited_project_policy_set_id
                != current_project.as_ref().map(|row| row.policy_set_id))
    {
        return Err(GroupApiError::conflict("hook_policy_version_conflict"));
    }
    let document =
        decode_document(&draft.policy_document).map_err(|_| GroupApiError::internal())?;
    let new_policy_set_id = if scope == PolicyScope::Project {
        publish_project_document(
            &mut tx,
            actor,
            project_id,
            current_project,
            document,
            Some(draft.draft_id),
            None,
            body.correlation_id,
            "policy_published",
        )
        .await?
    } else {
        publish_worktree_document(
            &mut tx,
            actor,
            project_id,
            worktree_id.ok_or_else(GroupApiError::bad_request)?,
            current_project,
            current_scope,
            document,
            Some(draft.draft_id),
            None,
            body.correlation_id,
            "policy_published",
        )
        .await?
    };
    sqlx::query("DELETE FROM multica.hook_policy_draft WHERE tenant_id = $1 AND draft_id = $2")
        .bind(actor.tenant_id)
        .bind(draft.draft_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(json!({ "policy_set_id": new_policy_set_id })))
}

async fn rollback_policy(
    state: &GroupApiState,
    actor: &super::AuthUser,
    scope: PolicyScope,
    requested_project_id: Uuid,
    worktree_id: Option<Uuid>,
    body: RollbackBody,
) -> Result<Json<Value>, GroupApiError> {
    if body.correlation_id.is_nil()
        || body.target_policy_set_id == body.expected_current_policy_set_id
    {
        return Err(GroupApiError::invalid_request("invalid_hook_rollback"));
    }
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let project_id = authorize_scope(
        &mut tx,
        actor,
        scope,
        (scope == PolicyScope::Project).then_some(requested_project_id),
        worktree_id,
        Access::Publish,
    )
    .await?;
    let current_project = load_current_policy(
        &mut tx,
        actor.tenant_id,
        project_id,
        PolicyScope::Project,
        None,
        true,
    )
    .await?;
    let current_scope = if scope == PolicyScope::Project {
        current_project.clone()
    } else {
        load_current_policy(
            &mut tx,
            actor.tenant_id,
            project_id,
            PolicyScope::Worktree,
            worktree_id,
            true,
        )
        .await?
    };
    if current_scope.as_ref().map(|row| row.policy_set_id)
        != Some(body.expected_current_policy_set_id)
    {
        return Err(GroupApiError::conflict("hook_policy_version_conflict"));
    }
    if load_current_draft(
        &mut tx,
        actor.tenant_id,
        project_id,
        scope,
        worktree_id,
        true,
    )
    .await?
    .is_some()
    {
        return Err(GroupApiError::conflict("hook_draft_exists"));
    }
    let target = sqlx::query_as::<_, PolicyRow>(
        r#"SELECT policy_set_id, project_id, scope_kind, worktree_id, policy_version,
                  schema_version, evaluator_api_version, digest, policy_document,
                  inherited_project_policy_set_id, valid_to
           FROM multica.hook_policy_set
           WHERE tenant_id = $1 AND project_id = $2 AND policy_set_id = $3
             AND scope_kind = $4 AND worktree_id IS NOT DISTINCT FROM $5
           FOR SHARE"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(body.target_policy_set_id)
    .bind(scope.as_str())
    .bind(worktree_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if target.valid_to.is_none() {
        return Err(GroupApiError::invalid_request(
            "rollback_target_not_historical",
        ));
    }
    let mut document = verify_stored_policy(&target, actor.tenant_id, project_id, worktree_id)?;
    let new_policy_set_id = if scope == PolicyScope::Project {
        publish_project_document(
            &mut tx,
            actor,
            project_id,
            current_project,
            document,
            None,
            Some(target.policy_set_id),
            body.correlation_id,
            "policy_rolled_back",
        )
        .await?
    } else {
        let project_row = current_project
            .as_ref()
            .ok_or_else(|| GroupApiError::conflict("project_policy_missing"))?;
        let project_doc = verify_stored_policy(project_row, actor.tenant_id, project_id, None)?;
        document.project_rules = project_doc.project_rules;
        document.project_version = project_doc.project_version;
        document.worktree_id = Some(
            worktree_id
                .ok_or_else(GroupApiError::bad_request)?
                .into_bytes(),
        );
        document.worktree_version = current_scope
            .as_ref()
            .and_then(|row| u64::try_from(row.policy_version).ok())
            .and_then(|v| v.checked_add(1))
            .or(Some(1));
        document.digest = document
            .computed_digest()
            .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
        publish_worktree_document(
            &mut tx,
            actor,
            project_id,
            worktree_id.ok_or_else(GroupApiError::bad_request)?,
            current_project,
            current_scope,
            document,
            None,
            Some(target.policy_set_id),
            body.correlation_id,
            "policy_rolled_back",
        )
        .await?
    };
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(
        json!({ "policy_set_id": new_policy_set_id, "rolled_back_from": target.policy_set_id }),
    ))
}

#[allow(clippy::too_many_arguments)]
async fn publish_project_document(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    current: Option<PolicyRow>,
    document: HookPolicyDocument,
    draft_id: Option<Uuid>,
    rollback_source: Option<Uuid>,
    correlation_id: Uuid,
    event_type: &'static str,
) -> Result<Uuid, GroupApiError> {
    let current_project_doc = current
        .as_ref()
        .map(|row| verify_stored_policy(row, actor.tenant_id, project_id, None))
        .transpose()?;
    let next_version = current
        .as_ref()
        .map(|row| row.policy_version)
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(GroupApiError::internal)?;
    let project_document =
        normalize_project_document(document, actor.tenant_id, project_id, next_version)?;
    let rebased_worktree_count = if let Some(current) = current.as_ref() {
        stage_project_overlay_rebases(
            tx,
            actor,
            project_id,
            current.policy_set_id,
            current_project_doc
                .as_ref()
                .ok_or_else(GroupApiError::internal)?,
            &project_document,
        )
        .await?
    } else {
        0
    };
    if let Some(current) = current.as_ref() {
        sqlx::query("UPDATE multica.hook_policy_set SET valid_to = now() WHERE tenant_id = $1 AND policy_set_id = $2 AND valid_to IS NULL")
            .bind(actor.tenant_id)
            .bind(current.policy_set_id)
            .execute(&mut **tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
    }
    let project_policy_id = insert_policy(
        tx,
        actor,
        project_id,
        PolicyScope::Project,
        None,
        next_version,
        None,
        &project_document,
    )
    .await?;
    if rebased_worktree_count > 0 {
        publish_staged_overlays(tx, actor, project_id, project_policy_id, correlation_id).await?;
    }
    insert_audit(
        tx, actor, project_id, PolicyScope::Project, None,
        Some(project_policy_id), draft_id, Some(next_version), event_type, correlation_id,
        json!({ "rollback_source_policy_set_id": rollback_source, "rebased_worktree_count": rebased_worktree_count }),
    )
    .await?;
    Ok(project_policy_id)
}

#[allow(clippy::too_many_arguments)]
async fn publish_worktree_document(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    worktree_id: Uuid,
    current_project: Option<PolicyRow>,
    current_worktree: Option<PolicyRow>,
    document: HookPolicyDocument,
    draft_id: Option<Uuid>,
    rollback_source: Option<Uuid>,
    correlation_id: Uuid,
    event_type: &'static str,
) -> Result<Uuid, GroupApiError> {
    let project =
        current_project.ok_or_else(|| GroupApiError::conflict("project_policy_missing"))?;
    let project_document = verify_stored_policy(&project, actor.tenant_id, project_id, None)?;
    let current_version = current_worktree
        .as_ref()
        .map(|row| row.policy_version)
        .unwrap_or(0);
    let next_version = current_version
        .checked_add(1)
        .ok_or_else(GroupApiError::internal)?;
    let document = normalize_worktree_document(
        document,
        actor.tenant_id,
        project_id,
        worktree_id,
        &project_document,
        next_version,
    )?;
    if let Some(current) = current_worktree.as_ref() {
        if current.inherited_project_policy_set_id != Some(project.policy_set_id) {
            return Err(GroupApiError::conflict("worktree_policy_rebase_required"));
        }
        let current_document =
            verify_stored_policy(current, actor.tenant_id, project_id, Some(worktree_id))?;
        if current_document.project_version != project_document.project_version
            || current_document.project_rules != project_document.project_rules
        {
            return Err(GroupApiError::internal());
        }
        sqlx::query("UPDATE multica.hook_policy_set SET valid_to = now() WHERE tenant_id = $1 AND policy_set_id = $2 AND valid_to IS NULL")
            .bind(actor.tenant_id)
            .bind(current.policy_set_id)
            .execute(&mut **tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
    }
    let policy_id = insert_policy(
        tx,
        actor,
        project_id,
        PolicyScope::Worktree,
        Some(worktree_id),
        next_version,
        Some(project.policy_set_id),
        &document,
    )
    .await?;
    insert_audit(
        tx,
        actor,
        project_id,
        PolicyScope::Worktree,
        Some(worktree_id),
        Some(policy_id),
        draft_id,
        Some(next_version),
        event_type,
        correlation_id,
        json!({ "rollback_source_policy_set_id": rollback_source }),
    )
    .await?;
    Ok(policy_id)
}

async fn stage_project_overlay_rebases(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    previous_project_policy_id: Uuid,
    previous_project_document: &HookPolicyDocument,
    next_project_document: &HookPolicyDocument,
) -> Result<usize, GroupApiError> {
    sqlx::query(
        r#"CREATE TEMP TABLE hook_policy_overlay_rebase_stage (
               worktree_id UUID PRIMARY KEY,
               previous_policy_set_id UUID NOT NULL,
               policy_document JSONB NOT NULL
           ) ON COMMIT DROP"#,
    )
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    let mut after_worktree_id = None;
    let mut total = 0usize;
    loop {
        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"SELECT policy_set_id, project_id, scope_kind, worktree_id, policy_version,
                      schema_version, evaluator_api_version, digest, policy_document,
                      inherited_project_policy_set_id, valid_to
               FROM multica.hook_policy_set
               WHERE tenant_id = $1 AND project_id = $2 AND scope_kind = 'worktree'
                 AND inherited_project_policy_set_id = $3 AND valid_to IS NULL
                 AND ($4::UUID IS NULL OR worktree_id > $4)
               ORDER BY worktree_id LIMIT $5 FOR UPDATE"#,
        )
        .bind(actor.tenant_id)
        .bind(project_id)
        .bind(previous_project_policy_id)
        .bind(after_worktree_id)
        .bind(PROJECT_REBASE_BATCH_SIZE as i64)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
        if rows.is_empty() {
            break;
        }

        let mut staged = Vec::with_capacity(rows.len());
        for row in rows {
            let worktree_id = row.worktree_id.ok_or_else(GroupApiError::internal)?;
            if row.inherited_project_policy_set_id != Some(previous_project_policy_id) {
                return Err(GroupApiError::internal());
            }
            let mut document =
                verify_stored_policy(&row, actor.tenant_id, project_id, Some(worktree_id))?;
            if document.project_version != previous_project_document.project_version
                || document.project_rules != previous_project_document.project_rules
            {
                return Err(GroupApiError::internal());
            }
            let next_worktree_version = row
                .policy_version
                .checked_add(1)
                .ok_or_else(GroupApiError::internal)?;
            document
                .project_rules
                .clone_from(&next_project_document.project_rules);
            document.project_version = next_project_document.project_version;
            document.worktree_version =
                Some(u64::try_from(next_worktree_version).map_err(|_| GroupApiError::internal())?);
            document.digest = document.computed_digest().map_err(|_| {
                GroupApiError::conflict("worktree_overlay_incompatible_with_project_policy")
            })?;
            document.clone().verify().map_err(|_| {
                GroupApiError::conflict("worktree_overlay_incompatible_with_project_policy")
            })?;
            staged.push(RebaseStageRow {
                worktree_id,
                previous_policy_set_id: row.policy_set_id,
                policy_document: serde_json::to_value(document)
                    .map_err(|_| GroupApiError::internal())?,
            });
        }

        for row in &staged {
            sqlx::query(
                "INSERT INTO hook_policy_overlay_rebase_stage (worktree_id, previous_policy_set_id, policy_document) VALUES ($1, $2, $3)",
            )
            .bind(row.worktree_id)
            .bind(row.previous_policy_set_id)
            .bind(&row.policy_document)
            .execute(&mut **tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
            sqlx::query(
                "UPDATE multica.hook_policy_set SET valid_to = now() WHERE tenant_id = $1 AND policy_set_id = $2 AND valid_to IS NULL",
            )
            .bind(actor.tenant_id)
            .bind(row.previous_policy_set_id)
            .execute(&mut **tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
        }
        after_worktree_id = staged.last().map(|row| row.worktree_id);
        total = total
            .checked_add(staged.len())
            .ok_or_else(GroupApiError::internal)?;
    }
    Ok(total)
}

async fn publish_staged_overlays(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    project_policy_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), GroupApiError> {
    let mut after_worktree_id = None;
    loop {
        let rows = sqlx::query_as::<_, RebaseStageRow>(
            r#"SELECT worktree_id, previous_policy_set_id, policy_document
               FROM hook_policy_overlay_rebase_stage
               WHERE ($1::UUID IS NULL OR worktree_id > $1)
               ORDER BY worktree_id LIMIT $2 FOR UPDATE"#,
        )
        .bind(after_worktree_id)
        .bind(PROJECT_REBASE_BATCH_SIZE as i64)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
        if rows.is_empty() {
            break;
        }
        for row in &rows {
            let document =
                decode_document(&row.policy_document).map_err(|_| GroupApiError::internal())?;
            let worktree_id =
                Uuid::from_bytes(document.worktree_id.ok_or_else(GroupApiError::internal)?);
            if worktree_id != row.worktree_id {
                return Err(GroupApiError::internal());
            }
            let version = document
                .worktree_version
                .ok_or_else(GroupApiError::internal)?;
            let policy_id = insert_policy(
                tx,
                actor,
                project_id,
                PolicyScope::Worktree,
                Some(worktree_id),
                i64::try_from(version).map_err(|_| GroupApiError::internal())?,
                Some(project_policy_id),
                &document,
            )
            .await?;
            insert_audit(
                tx,
                actor,
                project_id,
                PolicyScope::Worktree,
                Some(worktree_id),
                Some(policy_id),
                None,
                Some(i64::try_from(version).map_err(|_| GroupApiError::internal())?),
                "policy_published",
                correlation_id,
                json!({
                    "reason": "project_baseline_rebase",
                    "previous_policy_set_id": row.previous_policy_set_id
                }),
            )
            .await?;
        }
        after_worktree_id = rows.last().map(|row| row.worktree_id);
        for row in &rows {
            sqlx::query("DELETE FROM hook_policy_overlay_rebase_stage WHERE worktree_id = $1")
                .bind(row.worktree_id)
                .execute(&mut **tx)
                .await
                .map_err(|_| GroupApiError::internal())?;
        }
    }
    Ok(())
}

fn validate_draft_document(
    document: &HookPolicyDocument,
    tenant_id: Uuid,
    project_id: Uuid,
    scope: PolicyScope,
    worktree_id: Option<Uuid>,
    project: Option<&PolicyRow>,
    current: Option<&PolicyRow>,
) -> Result<(), GroupApiError> {
    validate_document_scope(document, tenant_id, project_id, worktree_id)?;
    if scope == PolicyScope::Project {
        let next = current
            .map(|row| row.policy_version)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(GroupApiError::internal)?;
        if document.project_version != u64::try_from(next).map_err(|_| GroupApiError::internal())?
            || document.worktree_id.is_some()
            || document.worktree_version.is_some()
            || !document.worktree_rules.is_empty()
        {
            return Err(GroupApiError::invalid_request(
                "invalid_project_hook_policy",
            ));
        }
    } else {
        let project = project.ok_or_else(|| GroupApiError::conflict("project_policy_missing"))?;
        let project_document = verify_stored_policy(project, tenant_id, project_id, None)?;
        if let Some(current) = current {
            let current_document =
                verify_stored_policy(current, tenant_id, project_id, worktree_id)?;
            if current.inherited_project_policy_set_id != Some(project.policy_set_id)
                || current_document.project_version != project_document.project_version
                || current_document.project_rules != project_document.project_rules
            {
                return Err(GroupApiError::internal());
            }
        }
        let next = current
            .map(|row| row.policy_version)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(GroupApiError::internal)?;
        if document.project_version != project_document.project_version
            || document.project_rules != project_document.project_rules
            || document.worktree_version
                != Some(u64::try_from(next).map_err(|_| GroupApiError::internal())?)
        {
            return Err(GroupApiError::conflict("hook_policy_version_conflict"));
        }
    }
    document
        .clone()
        .verify()
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
    Ok(())
}

fn validate_document_scope(
    document: &HookPolicyDocument,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Option<Uuid>,
) -> Result<(), GroupApiError> {
    if Uuid::from_bytes(document.tenant_id) != tenant_id
        || Uuid::from_bytes(document.project_id) != project_id
        || document.worktree_id.map(Uuid::from_bytes) != worktree_id
    {
        return Err(GroupApiError::invalid_request("hook_policy_scope_mismatch"));
    }
    let encoded = serde_json::to_vec(document)
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
    if encoded.len() > MAX_POLICY_DOCUMENT_BYTES {
        return Err(GroupApiError::invalid_request("hook_policy_too_large"));
    }
    Ok(())
}

fn normalize_project_document(
    mut document: HookPolicyDocument,
    tenant_id: Uuid,
    project_id: Uuid,
    version: i64,
) -> Result<HookPolicyDocument, GroupApiError> {
    document.tenant_id = tenant_id.into_bytes();
    document.project_id = project_id.into_bytes();
    document.worktree_id = None;
    document.project_version = u64::try_from(version).map_err(|_| GroupApiError::internal())?;
    document.worktree_version = None;
    document.worktree_rules.clear();
    document.digest = document
        .computed_digest()
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
    document
        .clone()
        .verify()
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
    Ok(document)
}

fn normalize_worktree_document(
    mut document: HookPolicyDocument,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    project_document: &HookPolicyDocument,
    version: i64,
) -> Result<HookPolicyDocument, GroupApiError> {
    document.tenant_id = tenant_id.into_bytes();
    document.project_id = project_id.into_bytes();
    document.worktree_id = Some(worktree_id.into_bytes());
    document.project_version = project_document.project_version;
    document.worktree_version =
        Some(u64::try_from(version).map_err(|_| GroupApiError::internal())?);
    document
        .project_rules
        .clone_from(&project_document.project_rules);
    document.digest = document
        .computed_digest()
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
    document
        .clone()
        .verify()
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_policy"))?;
    Ok(document)
}

async fn load_current_policy(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    scope: PolicyScope,
    worktree_id: Option<Uuid>,
    for_update: bool,
) -> Result<Option<PolicyRow>, GroupApiError> {
    let lock = if for_update {
        "FOR UPDATE"
    } else {
        "FOR SHARE"
    };
    let query = format!(
        r#"SELECT policy_set_id, project_id, scope_kind, worktree_id, policy_version,
                  schema_version, evaluator_api_version, digest, policy_document,
                  inherited_project_policy_set_id, valid_to
           FROM multica.hook_policy_set
           WHERE tenant_id = $1 AND project_id = $2 AND scope_kind = $3
             AND worktree_id IS NOT DISTINCT FROM $4 AND valid_to IS NULL {lock}"#
    );
    sqlx::query_as::<_, PolicyRow>(&query)
        .bind(tenant_id)
        .bind(project_id)
        .bind(scope.as_str())
        .bind(worktree_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())
}

/// Load one immutable effective policy in the caller's authorized transaction. Missing Project
/// baseline is represented as `None`; the Rust evaluator turns that condition into a deny.
pub(super) async fn load_verified_effective_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
) -> Result<Option<domain_hook::VerifiedHookPolicySnapshot>, GroupApiError> {
    let Some(project_row) =
        load_current_policy(tx, tenant_id, project_id, PolicyScope::Project, None, false).await?
    else {
        return Ok(None);
    };
    let project_document = verify_stored_policy(&project_row, tenant_id, project_id, None)?;
    let effective_document = if let Some(worktree_row) = load_current_policy(
        tx,
        tenant_id,
        project_id,
        PolicyScope::Worktree,
        Some(worktree_id),
        false,
    )
    .await?
    {
        if worktree_row.inherited_project_policy_set_id != Some(project_row.policy_set_id) {
            return Err(GroupApiError::conflict("worktree_policy_rebase_required"));
        }
        let worktree_document =
            verify_stored_policy(&worktree_row, tenant_id, project_id, Some(worktree_id))?;
        if worktree_document.project_version != project_document.project_version
            || worktree_document.project_rules != project_document.project_rules
        {
            return Err(GroupApiError::internal());
        }
        worktree_document
    } else {
        project_document
    };
    effective_document
        .verify()
        .map(Some)
        .map_err(|_| GroupApiError::internal())
}

async fn load_current_draft(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    scope: PolicyScope,
    worktree_id: Option<Uuid>,
    for_update: bool,
) -> Result<Option<DraftRow>, GroupApiError> {
    let lock = if for_update {
        "FOR UPDATE"
    } else {
        "FOR SHARE"
    };
    let query = format!(
        r#"SELECT draft_id, base_policy_set_id, inherited_project_policy_set_id,
                  draft_version, policy_document, expires_at
           FROM multica.hook_policy_draft
           WHERE tenant_id = $1 AND project_id = $2 AND scope_kind = $3
             AND worktree_id IS NOT DISTINCT FROM $4 AND draft_state IN ('editing','pending_approval')
           {lock}"#
    );
    sqlx::query_as::<_, DraftRow>(&query)
        .bind(tenant_id)
        .bind(project_id)
        .bind(scope.as_str())
        .bind(worktree_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())
}

async fn load_audit(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    scope: PolicyScope,
    worktree_id: Option<Uuid>,
) -> Result<Vec<AuditRow>, GroupApiError> {
    sqlx::query_as::<_, AuditRow>(
        r#"SELECT event_id, event_type, policy_set_id, draft_id, policy_version,
                  correlation_id, details, occurred_at
           FROM multica.hook_policy_audit_event
           WHERE tenant_id = $1 AND project_id = $2 AND scope_kind = $3
             AND worktree_id IS NOT DISTINCT FROM $4
           ORDER BY occurred_at DESC, event_id DESC LIMIT 100"#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(scope.as_str())
    .bind(worktree_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

fn verify_stored_policy(
    row: &PolicyRow,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Option<Uuid>,
) -> Result<HookPolicyDocument, GroupApiError> {
    let document = decode_document(&row.policy_document).map_err(|_| GroupApiError::internal())?;
    validate_document_scope(&document, tenant_id, project_id, worktree_id)
        .map_err(|_| GroupApiError::internal())?;
    let expected_version = if worktree_id.is_some() {
        document.worktree_version
    } else {
        Some(document.project_version)
    };
    if row.project_id != project_id
        || row.worktree_id != worktree_id
        || row.scope_kind
            != if worktree_id.is_some() {
                "worktree"
            } else {
                "project"
            }
        || expected_version != u64::try_from(row.policy_version).ok()
        || row.schema_version != i16::try_from(document.schema_version).unwrap_or_default()
        || row.evaluator_api_version
            != i16::try_from(document.evaluator_api_version).unwrap_or_default()
        || row.digest.as_slice() != document.digest
    {
        return Err(GroupApiError::internal());
    }
    document
        .clone()
        .verify()
        .map_err(|_| GroupApiError::internal())?;
    Ok(document)
}

fn decode_document(value: &Value) -> Result<HookPolicyDocument, ()> {
    let bytes = serde_json::to_vec(value).map_err(|_| ())?;
    if bytes.len() > MAX_POLICY_DOCUMENT_BYTES {
        return Err(());
    }
    serde_json::from_slice(&bytes).map_err(|_| ())
}

fn parse_draft_body(bytes: &[u8]) -> Result<DraftBody, GroupApiError> {
    if bytes.len() > MAX_POLICY_REQUEST_BYTES {
        return Err(GroupApiError::invalid_request("hook_policy_too_large"));
    }
    let body: DraftBody = serde_json::from_slice(bytes)
        .map_err(|_| GroupApiError::invalid_request("invalid_hook_draft"))?;
    if body.correlation_id.is_nil() {
        return Err(GroupApiError::invalid_request("invalid_hook_draft"));
    }
    Ok(body)
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value).map_err(|_| GroupApiError::bad_request())
}

#[allow(clippy::too_many_arguments)]
async fn insert_policy(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    scope: PolicyScope,
    worktree_id: Option<Uuid>,
    version: i64,
    inherited_project_policy_set_id: Option<Uuid>,
    document: &HookPolicyDocument,
) -> Result<Uuid, GroupApiError> {
    let policy_document = serde_json::to_value(document).map_err(|_| GroupApiError::internal())?;
    sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO multica.hook_policy_set
           (tenant_id, project_id, scope_kind, worktree_id, policy_version,
            schema_version, evaluator_api_version, digest, policy_document,
            inherited_project_policy_set_id, published_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING policy_set_id"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(scope.as_str())
    .bind(worktree_id)
    .bind(version)
    .bind(i16::try_from(document.schema_version).map_err(|_| GroupApiError::internal())?)
    .bind(i16::try_from(document.evaluator_api_version).map_err(|_| GroupApiError::internal())?)
    .bind(document.digest.to_vec())
    .bind(policy_document)
    .bind(inherited_project_policy_set_id)
    .bind(actor.user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

#[allow(clippy::too_many_arguments)]
async fn insert_audit(
    tx: &mut Transaction<'_, Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    scope: PolicyScope,
    worktree_id: Option<Uuid>,
    policy_set_id: Option<Uuid>,
    draft_id: Option<Uuid>,
    policy_version: Option<i64>,
    event_type: &str,
    correlation_id: Uuid,
    details: Value,
) -> Result<(), GroupApiError> {
    sqlx::query(
        r#"INSERT INTO multica.hook_policy_audit_event
           (tenant_id, project_id, scope_kind, worktree_id, policy_set_id, draft_id,
            actor_id, event_type, policy_version, correlation_id, details)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(scope.as_str())
    .bind(worktree_id)
    .bind(policy_set_id)
    .bind(draft_id)
    .bind(actor.user_id)
    .bind(event_type)
    .bind(policy_version)
    .bind(correlation_id)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_event_cursor_is_project_scoped_and_versioned() {
        let project_id = Uuid::new_v4();
        let event = HookEventProjection {
            event_id: Uuid::new_v4(),
            worktree_id: Some(Uuid::new_v4()),
            work_item_id: None,
            run_id: None,
            actor_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            source_kind: "worktree_lifecycle".to_owned(),
            hook_phase: "worktree_archive".to_owned(),
            hook_decision: "allow".to_owned(),
            hook_reason_code: "allowed_by_builtin_baseline".to_owned(),
            matched_rule_id: None,
            project_policy_version: Some(1),
            worktree_policy_version: None,
            evaluator_api_version: 1,
            policy_digest: None,
            evaluated_condition_count: 0,
            duration_ms: 0,
            timed_out: false,
            occurred_at: Utc::now(),
        };

        let encoded = encode_hook_event_cursor(project_id, &event).expect("encoded cursor");
        let decoded = decode_hook_event_cursor(&encoded).expect("decoded cursor");

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.project_id, project_id);
        assert_eq!(decoded.event_id, event.event_id);
        assert_eq!(decoded.occurred_at, event.occurred_at);
        assert!(decode_hook_event_cursor(&"x".repeat(MAX_HOOK_EVENT_CURSOR_BYTES + 1)).is_err());
    }

    #[test]
    fn hook_event_query_rejects_unknown_fields() {
        assert!(serde_json::from_value::<HookEventsQuery>(json!({
            "limit": 25,
            "offset": 100
        }))
        .is_err());
    }

    fn policy_document(
        tenant_id: Uuid,
        project_id: Uuid,
        worktree_id: Option<Uuid>,
        project_version: u64,
        worktree_version: Option<u64>,
    ) -> HookPolicyDocument {
        let mut document = HookPolicyDocument {
            schema_version: 1,
            evaluator_api_version: 1,
            tenant_id: tenant_id.into_bytes(),
            project_id: project_id.into_bytes(),
            worktree_id: worktree_id.map(Uuid::into_bytes),
            project_version,
            worktree_version,
            digest: [0; 32],
            project_rules: Vec::new(),
            worktree_rules: Vec::new(),
        };
        document.digest = document.computed_digest().expect("empty policy digest");
        document
    }

    #[test]
    fn project_normalization_recomputes_digest_for_next_revision() {
        let tenant_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let document = policy_document(tenant_id, project_id, None, 1, None);

        let normalized =
            normalize_project_document(document, tenant_id, project_id, 2).expect("valid baseline");

        assert_eq!(normalized.project_version, 2);
        assert_eq!(normalized.worktree_id, None);
        assert_eq!(normalized.worktree_version, None);
        assert!(normalized.clone().verify().is_ok());
    }

    #[test]
    fn worktree_normalization_pins_the_current_project_revision() {
        let tenant_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let worktree_id = Uuid::new_v4();
        let baseline = policy_document(tenant_id, project_id, None, 4, None);
        let overlay = policy_document(tenant_id, project_id, Some(worktree_id), 3, Some(1));

        let normalized =
            normalize_worktree_document(overlay, tenant_id, project_id, worktree_id, &baseline, 2)
                .expect("valid overlay");

        assert_eq!(normalized.project_version, 4);
        assert_eq!(normalized.worktree_version, Some(2));
        assert_eq!(normalized.worktree_id, Some(worktree_id.into_bytes()));
        assert!(normalized.clone().verify().is_ok());
    }

    #[test]
    fn scope_validation_rejects_a_document_from_another_worktree() {
        let tenant_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let document = policy_document(tenant_id, project_id, Some(Uuid::new_v4()), 1, Some(1));

        let result =
            validate_document_scope(&document, tenant_id, project_id, Some(Uuid::new_v4()));

        assert!(result.is_err());
    }

    #[test]
    fn decode_rejects_documents_over_the_policy_bound() {
        let oversized = Value::String("x".repeat(MAX_POLICY_DOCUMENT_BYTES + 1));

        assert!(decode_document(&oversized).is_err());
    }

    #[test]
    fn draft_request_parser_rejects_oversized_envelopes() {
        let bytes = vec![b' '; MAX_POLICY_REQUEST_BYTES + 1];

        assert!(parse_draft_body(&bytes).is_err());
    }
}
