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

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{
        HeaderMap,
        header::{CACHE_CONTROL, VARY},
    },
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Transaction};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

use super::{
    AuthenticatedUser, GroupApiError, GroupApiState, WorktreeGitLockObservation,
    WorktreeGitLockObserver, WorktreeGitLockObserverError, WorktreeGitLockQuery, WorktreeIndexRow,
    active_binding, require_scope, set_tenant, validate_actor, worktree_projection,
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
    owner_user_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    runtime_id: Option<Uuid>,
    archived: bool,
    version: i32,
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
        SELECT w.project_id, w.owner_user_id, w.agent_session_id, w.runtime_id, w.archived, w.version
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
    if body.archived == Some(true)
        && (current.agent_session_id.is_some() || current.runtime_id.is_some())
    {
        return Err(GroupApiError::conflict("active_execution_requires_stop"));
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
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    require_manager_role(&binding.role)?;
    let current = sqlx::query_as::<_, ManageWorktreeRow>(
        r#"
        SELECT w.project_id, w.owner_user_id, w.agent_session_id, w.runtime_id, w.archived, w.version
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
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if plan.requester_id != actor.user_id {
        return Err(GroupApiError::not_found());
    }
    if plan.status == "confirmed" {
        let result = plan.result_body.ok_or_else(GroupApiError::internal)?;
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(result));
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
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Err(GroupApiError::conflict("management_plan_expired"));
    }
    if plan.expected_version != current.version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    if plan.target_archived == Some(true)
        && (current.agent_session_id.is_some() || current.runtime_id.is_some())
    {
        return Err(GroupApiError::conflict("active_execution_requires_stop"));
    }
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
        SELECT w.project_id, w.owner_user_id, w.agent_session_id, w.runtime_id, w.archived, w.version
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

async fn authorize_worktree(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
) -> Result<ManageWorktreeRow, GroupApiError> {
    set_tenant(tx, actor.tenant_id).await?;
    let worktree = sqlx::query_as::<_, ManageWorktreeRow>(
        r#"
        SELECT w.project_id, w.owner_user_id, w.agent_session_id, w.runtime_id, w.archived, w.version
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
