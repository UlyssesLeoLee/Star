//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"worktrees.rs",type:"file",language:"rust"}),(m:Module {name:"worktrees",type:"module",language:"rust"}),
//!   (q:Class {name:"IndexQuery",type:"class"}),(c:Class {name:"IndexCursor",type:"class"}),(p:Class {name:"ManagementPlanBody",type:"class"}),(w:Class {name:"ManageWorktreeRow",type:"class"}),(i:Class {name:"PlanIdempotency",type:"class"}),(mp:Class {name:"ManagementPlanRow",type:"class"}),
//!   (rt:Function {name:"router",type:"function"}),(li:Function {name:"list_project_worktrees",type:"function"}),(lm:Function {name:"list_project_members",type:"function"}),(pl:Function {name:"create_management_plan",type:"function"}),(co:Function {name:"confirm_management_plan",type:"function"}),(au:Function {name:"authorize_worktree",type:"function"}),(cu:Function {name:"decode_cursor",type:"function"}),(ec:Function {name:"encode_cursor",type:"function"}),(rh:Function {name:"request_hash",type:"function"}),(val:Function {name:"validate_plan",type:"function"}),(rm:Function {name:"require_manager_role",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(q),(m)-[:CONTAINS]->(c),(m)-[:CONTAINS]->(p),(m)-[:CONTAINS]->(w),(m)-[:CONTAINS]->(i),(m)-[:CONTAINS]->(mp),(m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(li),(m)-[:CONTAINS]->(lm),(m)-[:CONTAINS]->(pl),(m)-[:CONTAINS]->(co),(m)-[:CONTAINS]->(au),(m)-[:CONTAINS]->(cu),(m)-[:CONTAINS]->(ec),(m)-[:CONTAINS]->(rh),(m)-[:CONTAINS]->(val),(m)-[:CONTAINS]->(rm),
//!   (rt)-[:CALLS]->(li),(rt)-[:CALLS]->(lm),(rt)-[:CALLS]->(pl),(rt)-[:CALLS]->(co),(li)-[:CALLS]->(au),(li)-[:CALLS]->(cu),(li)-[:CALLS]->(ec),(lm)-[:CALLS]->(au),(pl)-[:CALLS]->(au),(pl)-[:CALLS]->(val),(pl)-[:CALLS]->(rh),(pl)-[:CALLS]->(rm),(co)-[:CALLS]->(au),(co)-[:CALLS]->(rm);
//! MATCH (lm:Function {name:"list_project_members"})
//! CREATE (va:Function {name:"validate_actor",type:"function"}),(rs:Function {name:"require_scope",type:"function"}),(st:Function {name:"set_tenant",type:"function"}),(ab:Function {name:"active_binding",type:"function"}),
//!        (lm)-[:CALLS]->(va),(lm)-[:CALLS]->(rs),(lm)-[:CALLS]->(st),(lm)-[:CALLS]->(ab);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktrees",type:"module"}),(rt:Function {name:"router",type:"function"});
//! CREATE (projects:Function {name:"list_authorized_projects",type:"function",language:"rust",visibility:"private",complexity:"moderate"}),(projectsQuery:Class {name:"AuthorizedProjectsQuery",type:"class",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(projects),(m)-[:CONTAINS]->(projectsQuery),(rt)-[:CALLS]->(projects),(projects)-[:CALLS]->(validate_actor),(projects)-[:CALLS]->(require_scope),(projects)-[:CALLS]->(set_tenant);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktrees",type:"module"}),(li:Function {name:"list_project_worktrees",type:"function"}),(wp:Function {name:"worktree_projection",type:"function"}),(observerTrait:Interface {name:"WorktreeGitLockObserver",type:"interface"});
//! CREATE (observations:Function {name:"observe_git_locks",type:"function",language:"rust",visibility:"private",complexity:"moderate"}),(observe:Function {name:"observe_git_lock",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(bounded:Function {name:"observe_git_lock_with_timeout",type:"function",language:"rust",visibility:"private",complexity:"moderate"}),(testModule:Module {name:"git_lock_observation_tests",type:"module",language:"rust"}),(test:Class {name:"TestGitLockObserver",type:"class",language:"rust"}),(testObserve:Function {name:"TestGitLockObserver::observe",type:"function",language:"rust"}),(query:Function {name:"query",type:"function",language:"rust",visibility:"private"}),(missing:Function {name:"missing_provider_or_runtime_returns_unknown",type:"function",language:"rust",visibility:"private"}),(unavailable:Function {name:"unavailable_provider_returns_unknown",type:"function",language:"rust",visibility:"private"}),(stale:Function {name:"stale_or_unstamped_provider_result_is_unknown",type:"function",language:"rust",visibility:"private"}),(fresh:Function {name:"fresh_provider_result_preserves_git_lock_state",type:"function",language:"rust",visibility:"private"}),(timeoutTest:Function {name:"slow_provider_times_out_to_unknown",type:"function",language:"rust",visibility:"private"}),(batchTest:Function {name:"batch_observations_preserve_order_and_unknown_missing_runtime",type:"function",language:"rust",visibility:"private"});
//! CREATE (m)-[:CONTAINS]->(observations),(m)-[:CONTAINS]->(observe),(m)-[:CONTAINS]->(bounded),(m)-[:CONTAINS]->(testModule),(testModule)-[:CONTAINS]->(test),(testModule)-[:CONTAINS]->(query),(testModule)-[:CONTAINS]->(missing),(testModule)-[:CONTAINS]->(unavailable),(testModule)-[:CONTAINS]->(stale),(testModule)-[:CONTAINS]->(fresh),(testModule)-[:CONTAINS]->(timeoutTest),(testModule)-[:CONTAINS]->(batchTest),(test)-[:HAS_METHOD]->(testObserve),(test)-[:IMPLEMENTS]->(observerTrait),(li)-[:CALLS]->(observations),(observations)-[:CALLS]->(observe),(observe)-[:CALLS]->(bounded),(bounded)-[:CALLS]->(observerTrait),(observations)-[:CALLS]->(wp),(missing)-[:CALLS]->(observe),(missing)-[:CALLS]->(observations),(unavailable)-[:CALLS]->(observe),(stale)-[:CALLS]->(observe),(fresh)-[:CALLS]->(observe),(timeoutTest)-[:CALLS]->(bounded),(batchTest)-[:CALLS]->(observations);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktrees",type:"module"});
//! CREATE (projectQueryTests:Module {name:"authorized_projects_query_tests",type:"module",language:"rust"}),(rejectsUnknown:Function {name:"rejects_unknown_project_query_fields",type:"function",language:"rust",visibility:"private",complexity:"simple"});
//! CREATE (m)-[:CONTAINS]->(projectQueryTests),(projectQueryTests)-[:CONTAINS]->(rejectsUnknown);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktrees",type:"module"}),(co:Function {name:"confirm_management_plan",type:"function"});
//! CREATE (archiveOutcome:Class {name:"ArchiveHookOutcome",type:"class",language:"rust"}),(archiveFacts:Class {name:"ArchiveGateFacts",type:"class",language:"rust"}),(confirmContext:Class {name:"ManagementConfirmationContext",type:"class",language:"rust"}),(confirmState:Enum {name:"ManagementConfirmationState",type:"enum",language:"rust"}),(lockContext:Function {name:"lock_management_confirmation_context",type:"function",language:"rust",visibility:"private",complexity:"complex"}),(prepare:Function {name:"prepare_archive_gate_facts",type:"function",language:"rust",visibility:"private",complexity:"complex"}),(archiveGate:Function {name:"evaluate_archive_gate",type:"function",language:"rust",visibility:"private",complexity:"moderate"}),(lockState:Function {name:"hook_retention_lock_state",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(fresh:Function {name:"archive_readiness_is_fresh",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(factsFresh:Function {name:"archive_gate_facts_are_fresh",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(projection:Function {name:"archive_hook_projection",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(archiveTests:Module {name:"archive_gate_tests",type:"module",language:"rust"}),(load:Function {name:"load_verified_effective_snapshot",type:"function",language:"rust",visibility:"pub(super)"}),(archiveObserver:Interface {name:"WorktreeArchiveReadinessObserver",type:"interface",language:"rust"}),(gitLockTimeout:Function {name:"observe_git_lock_with_timeout",type:"function",language:"rust",visibility:"private"});
//! CREATE (m)-[:CONTAINS]->(archiveOutcome),(m)-[:CONTAINS]->(archiveFacts),(m)-[:CONTAINS]->(confirmContext),(m)-[:CONTAINS]->(confirmState),(m)-[:CONTAINS]->(lockContext),(m)-[:CONTAINS]->(prepare),(m)-[:CONTAINS]->(archiveGate),(m)-[:CONTAINS]->(lockState),(m)-[:CONTAINS]->(fresh),(m)-[:CONTAINS]->(factsFresh),(m)-[:CONTAINS]->(projection),(m)-[:CONTAINS]->(archiveTests),(co)-[:CALLS]->(lockContext),(co)-[:CALLS]->(prepare),(co)-[:CALLS]->(archiveGate),(co)-[:CALLS]->(projection),(co)-[:CALLS]->(factsFresh),(prepare)-[:CALLS]->(archiveObserver),(prepare)-[:CALLS]->(gitLockTimeout),(archiveGate)-[:CALLS]->(load),(archiveOutcome)-[:USES]->(projection),(archiveFacts)-[:USES]->(lockState),(factsFresh)-[:CALLS]->(fresh);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktrees",type:"module"}),(co:Function {name:"confirm_management_plan",type:"function"}),(archiveTests:Module {name:"archive_gate_tests",type:"module"});
//! CREATE (appendEvent:Function {name:"append_archive_hook_event",type:"function",language:"rust",visibility:"private",complexity:"moderate"}),(decisionName:Function {name:"hook_decision_name",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(reasonName:Function {name:"hook_reason_name",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(eventTest:Function {name:"archive_hook_event_names_are_stable",type:"function",language:"rust",visibility:"private",complexity:"simple"});
//! CREATE (m)-[:CONTAINS]->(appendEvent),(m)-[:CONTAINS]->(decisionName),(m)-[:CONTAINS]->(reasonName),(archiveTests)-[:CONTAINS]->(eventTest),(co)-[:CALLS]->(appendEvent),(appendEvent)-[:CALLS]->(decisionName),(appendEvent)-[:CALLS]->(reasonName),(eventTest)-[:CALLS]->(decisionName),(eventTest)-[:CALLS]->(reasonName);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (archiveTests:Module {name:"archive_gate_tests",type:"module"}),(lockState:Function {name:"hook_retention_lock_state",type:"function"}),(fresh:Function {name:"archive_readiness_is_fresh",type:"function"});
//! CREATE (lockTest:Function {name:"lock_facts_fail_closed",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(freshTest:Function {name:"readiness_rejects_stale_and_future_observations",type:"function",language:"rust",visibility:"private",complexity:"simple"}),(gateFreshTest:Function {name:"archive_gate_facts_require_fresh_observations",type:"function",language:"rust",visibility:"private",complexity:"simple"});
//! CREATE (archiveTests)-[:CONTAINS]->(lockTest),(archiveTests)-[:CONTAINS]->(freshTest),(archiveTests)-[:CONTAINS]->(gateFreshTest),(lockTest)-[:CALLS]->(lockState),(freshTest)-[:CALLS]->(fresh),(gateFreshTest)-[:CALLS]->(factsFresh);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktrees",type:"module"}),(factsFresh:Function {name:"archive_gate_facts_are_fresh"}),(archiveTests:Module {name:"archive_gate_tests"});
//! CREATE (fenceMargin:Function {name:"archive_fence_has_commit_margin",type:"function",language:"rust",visibility:"private",complexity:"simple"});
//! CREATE (fenceTest:Function {name:"admission_fence_requires_commit_margin",type:"function",language:"rust",visibility:"private",complexity:"simple"});
//! CREATE (m)-[:CONTAINS]->(fenceMargin),(factsFresh)-[:CALLS]->(fenceMargin),(archiveTests)-[:CONTAINS]->(fenceTest),(fenceTest)-[:CALLS]->(fenceMargin);

use axum::{
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, VARY},
        HeaderMap,
    },
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use domain_hook::{
    evaluate as evaluate_hook, HookDecision, HookEventEnvelope, HookPhase, HookScope,
    RetentionLockState, EVENT_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Transaction};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, worktree_projection,
    AuthenticatedUser, GroupApiError, GroupApiState, WorktreeArchiveReadiness,
    WorktreeArchiveReadinessQuery, WorktreeGitLockObservation, WorktreeGitLockObserver,
    WorktreeGitLockObserverError, WorktreeGitLockQuery, WorktreeIndexRow,
};

#[derive(Debug, Deserialize)]
struct IndexQuery {
    limit: Option<i64>,
    cursor: Option<String>,
    owner_user_id: Option<String>,
    human_state: Option<String>,
    include_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizedProjectsQuery {
    limit: Option<i64>,
    cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexCursor {
    project_id: Uuid,
    updated_at: DateTime<Utc>,
    worktree_id: Uuid,
    owner_user_id: Option<Uuid>,
    human_state: Option<String>,
    include_archived: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ManagementPlanBody {
    operation: String,
    owner_user_id: Option<Uuid>,
    archived: Option<bool>,
    expected_version: i32,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, FromRow)]
struct ManageWorktreeRow {
    project_id: Uuid,
    repository_id: Uuid,
    owner_user_id: Option<Uuid>,
    runtime_id: Option<Uuid>,
    archived: bool,
    version: i32,
}

struct ArchiveHookOutcome {
    event_id: Uuid,
    evaluation: domain_hook::HookEvaluation,
    duration_ms: i64,
    readiness_observed_at: DateTime<Utc>,
    admission_fence_expires_at: DateTime<Utc>,
    git_lock_observed_at: Option<DateTime<Utc>>,
}

struct ArchiveGateFacts {
    event_id: Uuid,
    event: HookEventEnvelope,
    readiness_observed_at: DateTime<Utc>,
    admission_fence_expires_at: DateTime<Utc>,
    git_lock_observed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
struct PlanIdempotency {
    request_hash: Vec<u8>,
    plan_response: Value,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route("/api/v1/projects", get(list_authorized_projects))
        .route(
            "/api/v1/projects/{project_id}/worktrees",
            get(list_project_worktrees),
        )
        .route(
            "/api/v1/projects/{project_id}/members",
            get(list_project_members),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/management-plans",
            post(create_management_plan),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/management-plans/{plan_id}/confirm",
            post(confirm_management_plan),
        )
}

async fn list_authorized_projects(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Query(query): Query<AuthorizedProjectsQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "project:read")?;
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let cursor = query
        .cursor
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| GroupApiError::invalid_request("invalid_project_cursor"))?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let mut projects = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        SELECT project_id, role
        FROM permission.project_role_binding
        WHERE tenant_id = $1 AND user_id = $2 AND valid_from <= now() AND valid_to IS NULL
          AND ($3::UUID IS NULL OR project_id > $3)
        ORDER BY project_id
        LIMIT $4
        "#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(cursor)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let has_more = projects.len() > limit as usize;
    if has_more {
        projects.pop();
    }
    let next_cursor = if has_more {
        projects
            .last()
            .map(|(project_id, _)| project_id.to_string())
    } else {
        None
    };
    Ok((
        [(CACHE_CONTROL, "no-store"), (VARY, "Authorization")],
        Json(json!({
            "projects": projects.into_iter().map(|(project_id, role)| json!({
                "project_id": project_id,
                "role": role,
            })).collect::<Vec<_>>(),
            "limit": limit,
            "next_cursor": next_cursor,
        })),
    ))
}

async fn list_project_worktrees(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    Query(query): Query<IndexQuery>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "project:read")?;
    let project_id = Uuid::parse_str(&project_id).map_err(|_| GroupApiError::bad_request())?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let cursor = query.cursor.as_deref().map(decode_cursor).transpose()?;
    let owner_user_id = query
        .owner_user_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| GroupApiError::invalid_request("invalid_owner_filter"))?;
    let include_archived = query.include_archived.unwrap_or(false);
    if cursor.as_ref().is_some_and(|value| {
        value.project_id != project_id
            || value.owner_user_id != owner_user_id
            || value.human_state.as_deref() != query.human_state.as_deref()
            || value.include_archived != include_archived
    }) {
        return Err(GroupApiError::invalid_request("cursor_filter_mismatch"));
    }

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let binding = active_binding(&mut tx, &actor, project_id).await?;
    let mut rows = sqlx::query_as::<_, WorktreeIndexRow>(
        r#"
        SELECT w.id, w.name, w.repo_id, w.workspace_id, p.project_id, w.path, w.parent_id,
               w.owner_user_id, w.work_item_id, w.agent_id, w.agent_session_id, w.runtime_id,
               w.branch, w.human_state, w.machine_state, w.ahead, w.behind, w.dirty, w.health_score,
               w.last_activity, w.test_state, w.risk_count, w.locked, w.archived, w.version,
               w.pull_request_url, w.created_at, w.updated_at
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.tenant_id = $1 AND p.project_id = $2
          AND ($3::TIMESTAMPTZ IS NULL OR (w.updated_at, w.id) < ($3, $4))
          AND ($5::UUID IS NULL OR w.owner_user_id = $5)
          AND ($6::TEXT IS NULL OR w.human_state = $6)
          AND ($7::BOOLEAN OR w.archived = FALSE)
        ORDER BY w.updated_at DESC, w.id DESC
        LIMIT $8
        "#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(cursor.as_ref().map(|value| value.updated_at))
    .bind(cursor.as_ref().map(|value| value.worktree_id))
    .bind(owner_user_id)
    .bind(query.human_state.as_deref())
    .bind(include_archived)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let next_cursor = if has_more {
        let row = rows.last().ok_or_else(GroupApiError::internal)?;
        Some(encode_cursor(&IndexCursor {
            project_id,
            updated_at: row.updated_at,
            worktree_id: row.id,
            owner_user_id,
            human_state: query.human_state.clone(),
            include_archived,
        })?)
    } else {
        None
    };
    let queries = rows
        .iter()
        .map(|row| {
            row.runtime_id.map(|runtime_id| WorktreeGitLockQuery {
                tenant_id: actor.tenant_id,
                project_id: row.project_id,
                repository_id: row.repo_id,
                worktree_id: row.id,
                runtime_id,
            })
        })
        .collect();
    let observations = observe_git_locks(state.worktree_git_lock_observer.clone(), queries).await;
    let worktrees = rows
        .into_iter()
        .zip(observations)
        .map(|(row, observation)| worktree_projection(row, observation))
        .collect::<Vec<_>>();
    Ok(Json(json!({
        "project_id": project_id,
        "role": binding.role,
        "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
        "limit": limit,
        "next_cursor": next_cursor,
        "worktrees": worktrees,
    })))
}

async fn list_project_members(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "project:read")?;
    let project_id = Uuid::parse_str(&project_id).map_err(|_| GroupApiError::bad_request())?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let binding = active_binding(&mut tx, &actor, project_id).await?;
    let members = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        SELECT user_id, role
        FROM permission.project_role_binding
        WHERE tenant_id = $1 AND project_id = $2 AND valid_to IS NULL
        ORDER BY user_id
        "#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    Ok(Json(json!({
        "project_id": project_id,
        "role": binding.role,
        "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
        "members": members.into_iter().map(|(user_id, role)| json!({
            "user_id": user_id,
            "role": role,
        })).collect::<Vec<_>>(),
    })))
}

async fn create_management_plan(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<ManagementPlanBody>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "worktree:manage")?;
    validate_plan(&body)?;
    let worktree_id = Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .ok_or_else(|| GroupApiError::invalid_request("idempotency_key_required"))?
        .to_owned();
    let hash = request_hash(worktree_id, &body)?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    require_manager_role(&binding.role)?;
    if let Some(existing) = sqlx::query_as::<_, PlanIdempotency>(
        r#"
        SELECT request_hash, plan_response
        FROM multica.worktree_management_plan
        WHERE tenant_id = $1 AND requester_id = $2 AND idempotency_key = $3
        "#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(&key)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    {
        if existing.request_hash.as_slice() != hash.as_slice() {
            return Err(GroupApiError::conflict("idempotency_key_reused"));
        }
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(existing.plan_response));
    }

    let current = sqlx::query_as::<_, ManageWorktreeRow>(
        r#"
        SELECT w.project_id, w.repo_id AS repository_id, w.owner_user_id, w.runtime_id, w.archived, w.version
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2 AND p.project_id = $3
        FOR UPDATE OF w
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if current.version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    if let Some(target_owner) = body.owner_user_id {
        let has_membership = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT id FROM permission.project_role_binding
            WHERE tenant_id = $1 AND project_id = $2 AND user_id = $3 AND valid_to IS NULL
            FOR SHARE
            "#,
        )
        .bind(actor.tenant_id)
        .bind(worktree.project_id)
        .bind(target_owner)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?
        .is_some();
        if !has_membership {
            return Err(GroupApiError::invalid_request(
                "owner_must_be_project_member",
            ));
        }
    }

    let confirm_by = sqlx::query_scalar::<_, DateTime<Utc>>("SELECT now() + INTERVAL '5 minutes'")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    let plan_id = Uuid::new_v4();
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    let plan_response = json!({
        "plan_id": plan_id,
        "worktree_id": worktree_id,
        "operation": body.operation,
        "expected_version": body.expected_version,
        "expires_at": confirm_by,
        "status": "pending",
        "correlation_id": correlation_id,
    });
    sqlx::query(
        r#"
        INSERT INTO multica.worktree_management_plan
            (plan_id, tenant_id, project_id, worktree_id, requester_id, operation,
             target_owner_user_id, target_archived, expected_version, idempotency_key,
             request_hash, correlation_id, plan_response, confirm_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
    )
    .bind(plan_id)
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(actor.user_id)
    .bind(&body.operation)
    .bind(body.owner_user_id)
    .bind(body.archived)
    .bind(body.expected_version)
    .bind(&key)
    .bind(&hash)
    .bind(correlation_id)
    .bind(&plan_response)
    .bind(confirm_by)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::conflict("idempotency_race"))?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(plan_response))
}

async fn confirm_management_plan(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, plan_id)): Path<(String, String)>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "worktree:manage")?;
    let worktree_id = Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let plan_id = Uuid::parse_str(&plan_id).map_err(|_| GroupApiError::bad_request())?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let mut context =
        match lock_management_confirmation_context(&mut tx, &actor, worktree_id, plan_id).await? {
            ManagementConfirmationState::Confirmed(result) => {
                tx.commit().await.map_err(|_| GroupApiError::internal())?;
                return Ok(Json(result));
            }
            ManagementConfirmationState::Expired => {
                tx.commit().await.map_err(|_| GroupApiError::internal())?;
                return Err(GroupApiError::conflict("management_plan_expired"));
            }
            ManagementConfirmationState::Pending(context) => *context,
        };

    // Host Runtime calls can take up to two seconds. Release the Worktree and plan row locks
    // before asking the host to drain, then reacquire and reauthorize before applying anything.
    let archive_facts = if context.plan.target_archived == Some(true) {
        let expected_project_id = context.current.project_id;
        let expected_repository_id = context.current.repository_id;
        let expected_runtime_id = context.current.runtime_id;
        let expected_version = context.plan.expected_version;
        let correlation_id = context.plan.correlation_id;
        tx.rollback().await.map_err(|_| GroupApiError::internal())?;
        let facts = prepare_archive_gate_facts(
            &state,
            &actor,
            worktree_id,
            plan_id,
            &context.current,
            expected_version,
            correlation_id,
        )
        .await?;

        tx = state
            .resolver
            .pool
            .begin()
            .await
            .map_err(|_| GroupApiError::internal())?;
        context = match lock_management_confirmation_context(&mut tx, &actor, worktree_id, plan_id)
            .await?
        {
            ManagementConfirmationState::Confirmed(result) => {
                tx.commit().await.map_err(|_| GroupApiError::internal())?;
                return Ok(Json(result));
            }
            ManagementConfirmationState::Expired => {
                tx.commit().await.map_err(|_| GroupApiError::internal())?;
                return Err(GroupApiError::conflict("management_plan_expired"));
            }
            ManagementConfirmationState::Pending(context) => *context,
        };
        if context.plan.target_archived != Some(true)
            || context.plan.correlation_id != correlation_id
            || context.plan.expected_version != expected_version
            || context.current.project_id != expected_project_id
            || context.current.repository_id != expected_repository_id
            || context.current.runtime_id != expected_runtime_id
            || context.current.version != expected_version
        {
            return Err(GroupApiError::conflict("version_conflict"));
        }
        Some(facts)
    } else {
        None
    };

    let worktree = context.worktree;
    let current = context.current;
    let plan = context.plan;
    let archive_hook = if let Some(facts) = archive_facts.as_ref() {
        let outcome = evaluate_archive_gate(&actor, &mut tx, worktree_id, &current, facts).await?;
        if outcome.evaluation.decision != HookDecision::Allow {
            append_archive_hook_event(
                &mut tx,
                &actor,
                worktree.project_id,
                worktree_id,
                plan.correlation_id,
                &outcome,
            )
            .await?;
            let before = json!({
                "owner_user_id": current.owner_user_id,
                "archived": current.archived,
                "version": current.version,
            });
            let after = json!({
                "owner_user_id": current.owner_user_id,
                "archived": current.archived,
                "version": current.version,
                "execution_applied": false,
                "hook_evaluation": archive_hook_projection(&outcome),
            });
            sqlx::query(
                r#"INSERT INTO multica.worktree_management_audit
                   (tenant_id, project_id, worktree_id, plan_id, actor_id, idempotency_key,
                    correlation_id, operation, before_state, after_state)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            )
            .bind(actor.tenant_id)
            .bind(worktree.project_id)
            .bind(worktree_id)
            .bind(plan_id)
            .bind(actor.user_id)
            .bind(&plan.idempotency_key)
            .bind(plan.correlation_id)
            .bind(&plan.operation)
            .bind(before)
            .bind(after)
            .execute(&mut *tx)
            .await
            .map_err(|_| GroupApiError::internal())?;
            if outcome.evaluation.decision == HookDecision::Deny {
                sqlx::query(
                    "UPDATE multica.worktree_management_plan SET status = 'cancelled' WHERE plan_id = $1 AND tenant_id = $2",
                )
                .bind(plan_id)
                .bind(actor.tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(|_| GroupApiError::internal())?;
            }
            tx.commit().await.map_err(|_| GroupApiError::internal())?;
            let code = match outcome.evaluation.decision {
                HookDecision::Deny => "worktree_archive_hook_denied",
                HookDecision::RequireHuman => "worktree_archive_requires_human_approval",
                HookDecision::Defer => "worktree_archive_gate_deferred",
                HookDecision::Allow => unreachable!("allowed evaluation was checked above"),
            };
            return Err(GroupApiError::conflict(code));
        }
        Some(outcome)
    } else {
        None
    };
    if let Some(target_owner) = plan.target_owner_user_id {
        let has_membership = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT id FROM permission.project_role_binding
            WHERE tenant_id = $1 AND project_id = $2 AND user_id = $3 AND valid_to IS NULL
            FOR SHARE
            "#,
        )
        .bind(actor.tenant_id)
        .bind(worktree.project_id)
        .bind(target_owner)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?
        .is_some();
        if !has_membership {
            return Err(GroupApiError::invalid_request("owner_membership_revoked"));
        }
        let previous_version = sqlx::query_scalar::<_, i32>(
            r#"
            UPDATE multica.worktree_owner_assignment
            SET valid_to = now()
            WHERE tenant_id = $1 AND worktree_id = $2 AND valid_to IS NULL
            RETURNING version
            "#,
        )
        .bind(actor.tenant_id)
        .bind(worktree_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?
        .unwrap_or(0);
        sqlx::query(
            r#"
            INSERT INTO multica.worktree_owner_assignment
                (tenant_id, project_id, worktree_id, owner_user_id, assigned_by, version)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(actor.tenant_id)
        .bind(worktree.project_id)
        .bind(worktree_id)
        .bind(target_owner)
        .bind(actor.user_id)
        .bind(previous_version + 1)
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    }

    if let Some(outcome) = archive_hook.as_ref() {
        let rechecked = authorize_worktree(&mut tx, &actor, worktree_id).await?;
        let rechecked_binding = active_binding(&mut tx, &actor, rechecked.project_id).await?;
        require_manager_role(&rechecked_binding.role)?;
        if rechecked.project_id != worktree.project_id
            || rechecked.version != plan.expected_version
            || rechecked.version != current.version
            || rechecked.archived != current.archived
        {
            return Err(GroupApiError::conflict("version_conflict"));
        }
        if !archive_gate_facts_are_fresh(outcome) {
            return Err(GroupApiError::conflict(
                "worktree_archive_observation_stale",
            ));
        }
        append_archive_hook_event(
            &mut tx,
            &actor,
            worktree.project_id,
            worktree_id,
            plan.correlation_id,
            outcome,
        )
        .await?;
    }

    let before = json!({
        "owner_user_id": current.owner_user_id,
        "archived": current.archived,
        "version": current.version,
    });
    sqlx::query(
        r#"
        UPDATE worktree_canvas_worktree
        SET owner_user_id = COALESCE($4, owner_user_id),
            archived = COALESCE($5, archived),
            expires_at = CASE WHEN $5 IS TRUE THEN now() + retention_period
                              WHEN $5 IS FALSE THEN NULL
                              ELSE expires_at END,
            version = version + 1,
            updated_at = now()
        WHERE id = $1 AND tenant_id = $2 AND project_id = $3 AND version = $6
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(plan.target_owner_user_id)
    .bind(plan.target_archived)
    .bind(current.version)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let updated = sqlx::query_as::<_, ManageWorktreeRow>(
        r#"
        SELECT w.project_id, w.repo_id AS repository_id, w.owner_user_id, w.runtime_id, w.archived, w.version
        FROM worktree_canvas_worktree w WHERE w.id = $1 AND w.tenant_id = $2
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let after = json!({
        "owner_user_id": updated.owner_user_id,
        "archived": updated.archived,
        "version": updated.version,
        "hook_evaluation": archive_hook.as_ref().map(archive_hook_projection),
    });
    let result = json!({
        "worktree_id": worktree_id,
        "plan_id": plan_id,
        "status": "confirmed",
        "before": before,
        "after": after,
        "correlation_id": plan.correlation_id,
    });
    sqlx::query(
        r#"
        INSERT INTO multica.worktree_management_audit
            (tenant_id, project_id, worktree_id, plan_id, actor_id, idempotency_key,
             correlation_id, operation, before_state, after_state)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(plan_id)
    .bind(actor.user_id)
    .bind(&plan.idempotency_key)
    .bind(plan.correlation_id)
    .bind(&plan.operation)
    .bind(before)
    .bind(after)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"
        UPDATE multica.worktree_management_plan
        SET status = 'confirmed', result_body = $2, confirmed_at = now()
        WHERE plan_id = $1 AND tenant_id = $3
        "#,
    )
    .bind(plan_id)
    .bind(&result)
    .bind(actor.tenant_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(result))
}

#[derive(Debug, FromRow)]
struct ManagementPlanRow {
    requester_id: Uuid,
    operation: String,
    target_owner_user_id: Option<Uuid>,
    target_archived: Option<bool>,
    expected_version: i32,
    status: String,
    expired: bool,
    idempotency_key: String,
    correlation_id: Uuid,
    result_body: Option<Value>,
}

struct ManagementConfirmationContext {
    worktree: ManageWorktreeRow,
    current: ManageWorktreeRow,
    plan: ManagementPlanRow,
}

enum ManagementConfirmationState {
    Confirmed(Value),
    Expired,
    Pending(Box<ManagementConfirmationContext>),
}

async fn lock_management_confirmation_context(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
    plan_id: Uuid,
) -> Result<ManagementConfirmationState, GroupApiError> {
    let worktree = authorize_worktree(tx, actor, worktree_id).await?;
    let binding = active_binding(tx, actor, worktree.project_id).await?;
    require_manager_role(&binding.role)?;
    let current = sqlx::query_as::<_, ManageWorktreeRow>(
        r#"
        SELECT w.project_id, w.repo_id AS repository_id, w.owner_user_id, w.runtime_id, w.archived, w.version
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2 AND p.project_id = $3
        FOR UPDATE OF w
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;

    let plan = sqlx::query_as::<_, ManagementPlanRow>(
        r#"
        SELECT requester_id, operation, target_owner_user_id, target_archived,
               expected_version, status, confirm_by <= now() AS expired,
               idempotency_key, correlation_id, result_body
        FROM multica.worktree_management_plan
        WHERE plan_id = $1 AND tenant_id = $2 AND project_id = $3 AND worktree_id = $4
        FOR UPDATE
        "#,
    )
    .bind(plan_id)
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if plan.requester_id != actor.user_id {
        return Err(GroupApiError::not_found());
    }
    if plan.status == "confirmed" {
        return Ok(ManagementConfirmationState::Confirmed(
            plan.result_body.ok_or_else(GroupApiError::internal)?,
        ));
    }
    if plan.status != "pending" {
        return Err(GroupApiError::conflict("management_plan_expired"));
    }
    if plan.expired {
        sqlx::query(
            "UPDATE multica.worktree_management_plan SET status = 'expired' WHERE plan_id = $1 AND tenant_id = $2",
        )
        .bind(plan_id)
        .bind(actor.tenant_id)
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
        return Ok(ManagementConfirmationState::Expired);
    }
    if plan.expected_version != current.version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    Ok(ManagementConfirmationState::Pending(Box::new(
        ManagementConfirmationContext {
            worktree,
            current,
            plan,
        },
    )))
}

async fn prepare_archive_gate_facts(
    state: &GroupApiState,
    actor: &super::AuthUser,
    worktree_id: Uuid,
    operation_id: Uuid,
    worktree: &ManageWorktreeRow,
    expected_version: i32,
    correlation_id: Uuid,
) -> Result<ArchiveGateFacts, GroupApiError> {
    let runtime_id = worktree.runtime_id.ok_or_else(|| {
        GroupApiError::feature_unavailable("worktree_archive_runtime_unavailable")
    })?;
    let readiness_observer = state
        .worktree_archive_readiness_observer
        .as_ref()
        .ok_or_else(|| {
            GroupApiError::feature_unavailable("worktree_archive_readiness_unavailable")
        })?;
    let readiness_query = WorktreeArchiveReadinessQuery {
        operation_id,
        tenant_id: actor.tenant_id,
        project_id: worktree.project_id,
        repository_id: worktree.repository_id,
        worktree_id,
        runtime_id,
        actor_id: actor.user_id,
        expected_lifecycle_version: expected_version,
        correlation_id,
        max_drain_wait: Duration::from_secs(2),
    };
    let lock_query = WorktreeGitLockQuery {
        tenant_id: actor.tenant_id,
        project_id: worktree.project_id,
        repository_id: worktree.repository_id,
        worktree_id,
        runtime_id,
    };
    let git_lock = observe_git_lock_with_timeout(
        state.worktree_git_lock_observer.as_deref(),
        Some(lock_query),
        Duration::from_secs(2),
    )
    .await;
    match git_lock.state {
        super::WorktreeGitLockState::Unlocked => {}
        super::WorktreeGitLockState::Locked => {
            return Err(GroupApiError::conflict(
                "worktree_archive_git_lock_conflict",
            ));
        }
        super::WorktreeGitLockState::Unknown => {
            return Err(GroupApiError::feature_unavailable(
                "worktree_archive_git_lock_unavailable",
            ));
        }
    }

    let readiness = tokio::time::timeout(
        Duration::from_secs(2),
        readiness_observer.prepare_and_observe(readiness_query),
    )
    .await;
    let readiness: WorktreeArchiveReadiness = match readiness {
        Ok(Ok(snapshot)) => snapshot,
        Ok(Err(_)) | Err(_) => {
            return Err(GroupApiError::feature_unavailable(
                "worktree_archive_readiness_unavailable",
            ));
        }
    };
    if !readiness.drain_completed {
        return Err(GroupApiError::conflict("worktree_archive_drain_incomplete"));
    }
    if !archive_readiness_is_fresh(readiness.observed_at) {
        return Err(GroupApiError::feature_unavailable(
            "worktree_archive_readiness_stale",
        ));
    }
    if !archive_fence_has_commit_margin(readiness.admission_fence_expires_at) {
        return Err(GroupApiError::feature_unavailable(
            "worktree_archive_admission_fence_too_short",
        ));
    }

    let event_id = Uuid::new_v4();
    let event = HookEventEnvelope {
        event_id: event_id.into_bytes(),
        event_schema_version: EVENT_SCHEMA_VERSION,
        phase: HookPhase::BeforeWorktreeArchiveCleanup,
        scope: HookScope {
            tenant_id: actor.tenant_id.into_bytes(),
            project_id: worktree.project_id.into_bytes(),
            worktree_id: worktree_id.into_bytes(),
            actor_id: actor.user_id.into_bytes(),
            correlation_id: correlation_id.into_bytes(),
        },
        actor_authorized: true,
        lifecycle_version_matches: worktree.version == expected_version,
        runtime_healthy: Some(readiness.runtime_healthy),
        retention_lock: hook_retention_lock_state(&git_lock),
        active_run_count: readiness.active_run_count,
        active_agent_lease_count: readiness.active_agent_lease_count,
        file_claim_count: readiness.file_claim_count,
        owned_process_count: readiness.owned_process_count,
    };
    Ok(ArchiveGateFacts {
        event_id,
        event,
        readiness_observed_at: readiness.observed_at,
        admission_fence_expires_at: readiness.admission_fence_expires_at,
        git_lock_observed_at: git_lock.observed_at,
    })
}

async fn evaluate_archive_gate(
    actor: &super::AuthUser,
    tx: &mut Transaction<'_, sqlx::Postgres>,
    worktree_id: Uuid,
    worktree: &ManageWorktreeRow,
    facts: &ArchiveGateFacts,
) -> Result<ArchiveHookOutcome, GroupApiError> {
    let snapshot = super::hook_policies::load_verified_effective_snapshot(
        tx,
        actor.tenant_id,
        worktree.project_id,
        worktree_id,
    )
    .await?;
    let evaluation_started = Instant::now();
    let evaluation = evaluate_hook(&facts.event, snapshot.as_ref());
    let duration_ms = i64::try_from(evaluation_started.elapsed().as_millis()).unwrap_or(i64::MAX);
    Ok(ArchiveHookOutcome {
        event_id: facts.event_id,
        evaluation,
        duration_ms,
        readiness_observed_at: facts.readiness_observed_at,
        admission_fence_expires_at: facts.admission_fence_expires_at,
        git_lock_observed_at: facts.git_lock_observed_at,
    })
}

async fn append_archive_hook_event(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    project_id: Uuid,
    worktree_id: Uuid,
    correlation_id: Uuid,
    outcome: &ArchiveHookOutcome,
) -> Result<(), GroupApiError> {
    let evaluation = &outcome.evaluation;
    let evaluator_api_version =
        i16::try_from(evaluation.evaluator_api_version).map_err(|_| GroupApiError::internal())?;
    let project_policy_version = evaluation
        .project_version
        .map(i64::try_from)
        .transpose()
        .map_err(|_| GroupApiError::internal())?;
    let worktree_policy_version = evaluation
        .worktree_version
        .map(i64::try_from)
        .transpose()
        .map_err(|_| GroupApiError::internal())?;
    let details = json!({
        "readiness_observed_at": outcome.readiness_observed_at,
        "admission_fence_expires_at": outcome.admission_fence_expires_at,
        "git_lock_observed_at": outcome.git_lock_observed_at,
    });

    sqlx::query(
        r#"INSERT INTO multica.hook_execution_event
           (event_id, tenant_id, project_id, worktree_id, work_item_id, run_id,
            actor_id, correlation_id, source_kind, hook_phase, hook_decision,
            hook_reason_code, matched_rule_id, project_policy_version,
            worktree_policy_version, evaluator_api_version, policy_digest,
            evaluated_condition_count, duration_ms, timed_out, details)
           VALUES ($1, $2, $3, $4, NULL, NULL, $5, $6, 'worktree_lifecycle',
                   'worktree_archive', $7, $8, $9, $10, $11, $12, $13, $14, $15, false, $16)"#,
    )
    .bind(outcome.event_id)
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(worktree_id)
    .bind(actor.user_id)
    .bind(correlation_id)
    .bind(hook_decision_name(evaluation.decision))
    .bind(hook_reason_name(evaluation.reason_code))
    .bind(evaluation.matched_rule_id.map(Uuid::from_bytes))
    .bind(project_policy_version)
    .bind(worktree_policy_version)
    .bind(evaluator_api_version)
    .bind(evaluation.policy_digest.map(hex::encode))
    .bind(i32::from(evaluation.evaluated_condition_count))
    .bind(outcome.duration_ms)
    .bind(details)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

fn hook_decision_name(decision: HookDecision) -> &'static str {
    match decision {
        HookDecision::Allow => "allow",
        HookDecision::Deny => "deny",
        HookDecision::RequireHuman => "require_human",
        HookDecision::Defer => "defer",
    }
}

fn hook_reason_name(reason: domain_hook::HookReasonCode) -> &'static str {
    use domain_hook::HookReasonCode;

    match reason {
        HookReasonCode::AllowedByBuiltinBaseline => "allowed_by_builtin_baseline",
        HookReasonCode::IncompleteScope => "incomplete_scope",
        HookReasonCode::EventSchemaUnsupported => "event_schema_unsupported",
        HookReasonCode::ActorNotAuthorized => "actor_not_authorized",
        HookReasonCode::LifecycleVersionStale => "lifecycle_version_stale",
        HookReasonCode::RuntimeUnhealthyOrUnknown => "runtime_unhealthy_or_unknown",
        HookReasonCode::RetentionLockUnusable => "retention_lock_unusable",
        HookReasonCode::ExecutionNotDrained => "execution_not_drained",
        HookReasonCode::PolicyUnavailable => "policy_unavailable",
        HookReasonCode::PolicyInvalid => "policy_invalid",
        HookReasonCode::RuleDenied => "rule_denied",
        HookReasonCode::HumanApprovalRequired => "human_approval_required",
        HookReasonCode::ExternalConditionPending => "external_condition_pending",
    }
}

fn archive_readiness_is_fresh(observed_at: DateTime<Utc>) -> bool {
    let now = Utc::now();
    observed_at <= now && now.signed_duration_since(observed_at) <= chrono::Duration::seconds(5)
}

fn archive_fence_has_commit_margin(expires_at: DateTime<Utc>) -> bool {
    expires_at > Utc::now() + chrono::Duration::seconds(5)
}

fn archive_gate_facts_are_fresh(outcome: &ArchiveHookOutcome) -> bool {
    let now = Utc::now();
    let git_lock_is_fresh = outcome.git_lock_observed_at.is_some_and(|observed_at| {
        observed_at <= now
            && now.signed_duration_since(observed_at) <= chrono::Duration::seconds(30)
    });
    archive_readiness_is_fresh(outcome.readiness_observed_at)
        && archive_fence_has_commit_margin(outcome.admission_fence_expires_at)
        && git_lock_is_fresh
}

fn hook_retention_lock_state(observation: &WorktreeGitLockObservation) -> RetentionLockState {
    match observation.state {
        super::WorktreeGitLockState::Unlocked => RetentionLockState::Fresh,
        super::WorktreeGitLockState::Locked => RetentionLockState::Conflict,
        super::WorktreeGitLockState::Unknown => RetentionLockState::Unknown,
    }
}

fn archive_hook_projection(outcome: &ArchiveHookOutcome) -> Value {
    json!({
        "event_id": outcome.event_id,
        "decision": outcome.evaluation.decision,
        "reason_code": outcome.evaluation.reason_code,
        "matched_rule_id": outcome.evaluation.matched_rule_id.map(hex::encode),
        "evaluator_api_version": outcome.evaluation.evaluator_api_version,
        "project_policy_version": outcome.evaluation.project_version,
        "worktree_policy_version": outcome.evaluation.worktree_version,
        "policy_digest": outcome.evaluation.policy_digest.map(hex::encode),
        "duration_ms": outcome.duration_ms,
        "timed_out": false,
        "readiness_observed_at": outcome.readiness_observed_at,
        "admission_fence_expires_at": outcome.admission_fence_expires_at,
        "git_lock_observed_at": outcome.git_lock_observed_at,
    })
}

async fn authorize_worktree(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
) -> Result<ManageWorktreeRow, GroupApiError> {
    set_tenant(tx, actor.tenant_id).await?;
    let worktree = sqlx::query_as::<_, ManageWorktreeRow>(
        r#"
        SELECT w.project_id, w.repo_id AS repository_id, w.owner_user_id, w.runtime_id, w.archived, w.version
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    Ok(worktree)
}

fn decode_cursor(value: &str) -> Result<IndexCursor, GroupApiError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| GroupApiError::invalid_request("invalid_cursor"))?;
    serde_json::from_slice(&bytes).map_err(|_| GroupApiError::invalid_request("invalid_cursor"))
}

fn encode_cursor(value: &IndexCursor) -> Result<String, GroupApiError> {
    let bytes = serde_json::to_vec(value).map_err(|_| GroupApiError::internal())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn request_hash(worktree_id: Uuid, body: &ManagementPlanBody) -> Result<Vec<u8>, GroupApiError> {
    let bytes = serde_json::to_vec(&(worktree_id, body)).map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(bytes).to_vec())
}

fn validate_plan(body: &ManagementPlanBody) -> Result<(), GroupApiError> {
    let valid = match body.operation.as_str() {
        "assign_owner" => body.owner_user_id.is_some() && body.archived.is_none(),
        "set_archived" => body.archived.is_some() && body.owner_user_id.is_none(),
        _ => false,
    };
    if !valid || body.expected_version < 1 {
        return Err(GroupApiError::invalid_request("invalid_management_plan"));
    }
    Ok(())
}

fn require_manager_role(role: &str) -> Result<(), GroupApiError> {
    match role {
        "tenant_admin" | "project_admin" => Ok(()),
        _ => Err(GroupApiError::forbidden()),
    }
}

async fn observe_git_lock(
    observer: Option<&dyn WorktreeGitLockObserver>,
    query: Option<WorktreeGitLockQuery>,
) -> WorktreeGitLockObservation {
    observe_git_lock_with_timeout(observer, query, Duration::from_secs(2)).await
}

async fn observe_git_lock_with_timeout(
    observer: Option<&dyn WorktreeGitLockObserver>,
    query: Option<WorktreeGitLockQuery>,
    timeout: Duration,
) -> WorktreeGitLockObservation {
    let (Some(observer), Some(query)) = (observer, query) else {
        return WorktreeGitLockObservation::unknown();
    };
    match tokio::time::timeout(timeout, observer.observe(query)).await {
        Ok(Ok(observation)) => observation.normalize(Utc::now()),
        Ok(Err(WorktreeGitLockObserverError::Unavailable)) | Err(_) => {
            WorktreeGitLockObservation::unknown()
        }
    }
}

async fn observe_git_locks(
    observer: Option<Arc<dyn WorktreeGitLockObserver>>,
    queries: Vec<Option<WorktreeGitLockQuery>>,
) -> Vec<WorktreeGitLockObservation> {
    let Some(observer) = observer else {
        return vec![WorktreeGitLockObservation::unknown(); queries.len()];
    };

    let results = Arc::new(tokio::sync::Mutex::new(vec![
        WorktreeGitLockObservation::unknown();
        queries.len()
    ]));
    let concurrency = Arc::new(tokio::sync::Semaphore::new(8));
    let mut tasks = tokio::task::JoinSet::new();

    for (index, query) in queries.into_iter().enumerate() {
        let Some(query) = query else {
            continue;
        };
        let observer = observer.clone();
        let results = results.clone();
        let concurrency = concurrency.clone();
        tasks.spawn(async move {
            let Ok(_permit) = concurrency.acquire_owned().await else {
                return;
            };
            let observation = observe_git_lock(Some(observer.as_ref()), Some(query)).await;
            results.lock().await[index] = observation;
        });
    }

    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while !tasks.is_empty() {
        match tokio::time::timeout_at(deadline, tasks.join_next()).await {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => {
                tasks.abort_all();
                break;
            }
        }
    }
    while tasks.join_next().await.is_some() {}

    let observations = results.lock().await.clone();
    observations
}

#[cfg(test)]
mod git_lock_observation_tests {
    use super::super::WorktreeGitLockState;
    use super::*;

    struct TestGitLockObserver {
        observation: WorktreeGitLockObservation,
        unavailable: bool,
        delay: Duration,
    }

    #[async_trait::async_trait]
    impl WorktreeGitLockObserver for TestGitLockObserver {
        async fn observe(
            &self,
            _query: WorktreeGitLockQuery,
        ) -> Result<WorktreeGitLockObservation, WorktreeGitLockObserverError> {
            tokio::time::sleep(self.delay).await;
            if self.unavailable {
                Err(WorktreeGitLockObserverError::Unavailable)
            } else {
                Ok(self.observation.clone())
            }
        }
    }

    fn query() -> WorktreeGitLockQuery {
        WorktreeGitLockQuery {
            tenant_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            runtime_id: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn missing_provider_or_runtime_returns_unknown() {
        let missing_provider = observe_git_lock(None, Some(query())).await;
        let missing_runtime = observe_git_lock(None, None).await;
        assert_eq!(missing_provider.state, WorktreeGitLockState::Unknown);
        assert_eq!(missing_runtime.state, WorktreeGitLockState::Unknown);
    }

    #[tokio::test]
    async fn unavailable_provider_returns_unknown() {
        let observer = TestGitLockObserver {
            observation: WorktreeGitLockObservation {
                state: WorktreeGitLockState::Locked,
                observed_at: Some(Utc::now()),
            },
            unavailable: true,
            delay: Duration::ZERO,
        };
        let result = observe_git_lock(Some(&observer), Some(query())).await;
        assert_eq!(result.state, WorktreeGitLockState::Unknown);
        assert!(result.observed_at.is_none());
    }

    #[tokio::test]
    async fn stale_or_unstamped_provider_result_is_unknown() {
        for observed_at in [
            None,
            Some(Utc::now() - chrono::Duration::seconds(31)),
            Some(Utc::now() + chrono::Duration::seconds(1)),
        ] {
            let observer = TestGitLockObserver {
                observation: WorktreeGitLockObservation {
                    state: WorktreeGitLockState::Unlocked,
                    observed_at,
                },
                unavailable: false,
                delay: Duration::ZERO,
            };
            let result = observe_git_lock(Some(&observer), Some(query())).await;
            assert_eq!(result.state, WorktreeGitLockState::Unknown);
        }
    }

    #[tokio::test]
    async fn fresh_provider_result_preserves_git_lock_state() {
        let observer = TestGitLockObserver {
            observation: WorktreeGitLockObservation {
                state: WorktreeGitLockState::Locked,
                observed_at: Some(Utc::now()),
            },
            unavailable: false,
            delay: Duration::ZERO,
        };
        let result = observe_git_lock(Some(&observer), Some(query())).await;
        assert_eq!(result.state, WorktreeGitLockState::Locked);
        assert!(result.observed_at.is_some());
    }

    #[tokio::test]
    async fn slow_provider_times_out_to_unknown() {
        let observer = TestGitLockObserver {
            observation: WorktreeGitLockObservation {
                state: WorktreeGitLockState::Unlocked,
                observed_at: Some(Utc::now()),
            },
            unavailable: false,
            delay: Duration::from_millis(50),
        };
        let result =
            observe_git_lock_with_timeout(Some(&observer), Some(query()), Duration::from_millis(5))
                .await;
        assert_eq!(result.state, WorktreeGitLockState::Unknown);
        assert!(result.observed_at.is_none());
    }

    #[tokio::test]
    async fn batch_observations_preserve_order_and_unknown_missing_runtime() {
        let observer = Arc::new(TestGitLockObserver {
            observation: WorktreeGitLockObservation {
                state: WorktreeGitLockState::Unlocked,
                observed_at: Some(Utc::now()),
            },
            unavailable: false,
            delay: Duration::ZERO,
        });
        let results =
            observe_git_locks(Some(observer), vec![Some(query()), None, Some(query())]).await;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].state, WorktreeGitLockState::Unlocked);
        assert_eq!(results[1].state, WorktreeGitLockState::Unknown);
        assert_eq!(results[2].state, WorktreeGitLockState::Unlocked);
    }
}

#[cfg(test)]
mod authorized_projects_query_tests {
    use super::*;

    #[test]
    fn rejects_unknown_project_query_fields() {
        let valid: AuthorizedProjectsQuery =
            serde_json::from_value(json!({ "limit": 50, "cursor": Uuid::new_v4() })).unwrap();
        assert_eq!(valid.limit, Some(50));
        assert!(valid.cursor.is_some());

        let invalid = serde_json::from_value::<AuthorizedProjectsQuery>(json!({
            "limit": 50,
            "local_seed": true
        }));
        assert!(invalid.is_err());
    }
}

#[cfg(test)]
mod archive_gate_tests {
    use super::*;

    #[test]
    fn lock_facts_fail_closed() {
        assert_eq!(
            hook_retention_lock_state(&WorktreeGitLockObservation {
                state: super::super::WorktreeGitLockState::Unlocked,
                observed_at: Some(Utc::now()),
            }),
            RetentionLockState::Fresh
        );
        assert_eq!(
            hook_retention_lock_state(&WorktreeGitLockObservation {
                state: super::super::WorktreeGitLockState::Locked,
                observed_at: Some(Utc::now()),
            }),
            RetentionLockState::Conflict
        );
        assert_eq!(
            hook_retention_lock_state(&WorktreeGitLockObservation::unknown()),
            RetentionLockState::Unknown
        );
    }

    #[test]
    fn readiness_rejects_stale_and_future_observations() {
        assert!(archive_readiness_is_fresh(Utc::now()));
        assert!(!archive_readiness_is_fresh(
            Utc::now() - chrono::Duration::seconds(6)
        ));
        assert!(!archive_readiness_is_fresh(
            Utc::now() + chrono::Duration::seconds(1)
        ));
    }

    #[test]
    fn archive_gate_facts_require_fresh_observations() {
        let now = Utc::now();
        let evaluation = domain_hook::HookEvaluation {
            phase: HookPhase::BeforeWorktreeArchiveCleanup,
            decision: HookDecision::Allow,
            reason_code: domain_hook::HookReasonCode::AllowedByBuiltinBaseline,
            matched_rule_id: None,
            evaluator_api_version: domain_hook::EVALUATOR_API_VERSION,
            project_version: Some(1),
            worktree_version: None,
            policy_digest: Some([1; 32]),
            evaluated_condition_count: 0,
        };
        let fresh = ArchiveHookOutcome {
            event_id: Uuid::new_v4(),
            evaluation,
            duration_ms: 1,
            readiness_observed_at: now,
            admission_fence_expires_at: now + chrono::Duration::seconds(10),
            git_lock_observed_at: Some(now),
        };
        assert!(archive_gate_facts_are_fresh(&fresh));

        let stale_lock = ArchiveHookOutcome {
            git_lock_observed_at: Some(now - chrono::Duration::seconds(31)),
            ..fresh
        };
        assert!(!archive_gate_facts_are_fresh(&stale_lock));
    }

    #[test]
    fn admission_fence_requires_commit_margin() {
        let now = Utc::now();
        assert!(archive_fence_has_commit_margin(
            now + chrono::Duration::seconds(10)
        ));
        assert!(!archive_fence_has_commit_margin(
            now + chrono::Duration::seconds(4)
        ));
        assert!(!archive_fence_has_commit_margin(now));
    }

    #[test]
    fn archive_hook_event_names_are_stable() {
        assert_eq!(
            hook_decision_name(HookDecision::RequireHuman),
            "require_human"
        );
        assert_eq!(
            hook_reason_name(domain_hook::HookReasonCode::ExternalConditionPending),
            "external_condition_pending"
        );
    }
}
