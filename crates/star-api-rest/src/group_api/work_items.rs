//! @cypher schema=1 source_sha256=fa611b8814d39d79cf36b34e954270929a79e659ba11c01a0b16475878128c62
//! MERGE (self:File {path:"crates/star-api-rest/src/group_api/work_items.rs"})
//! MERGE (module:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::module",kind:"module"})
//! MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::router",kind:"function"})
//! MERGE (create:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::create_work_item",kind:"function"})
//! MERGE (list:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::list_work_items",kind:"function"})
//! MERGE (listRun:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::list_run_work_items",kind:"function"})
//! MERGE (get:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::get_work_item",kind:"function"})
//! MERGE (transition:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::transition_work_item",kind:"function"})
//! MERGE (review:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::review_work_item",kind:"function"})
//! MERGE (authorizeWorktree:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::authorize_worktree",kind:"function"})
//! MERGE (authorizeRun:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::authorize_task_run_for_worktree",kind:"function"})
//! MERGE (load:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::load_work_item",kind:"function"})
//! MERGE (lookupIdempotency:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::lookup_idempotency",kind:"function"})
//! MERGE (saveIdempotency:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::save_idempotency",kind:"function"})
//! MERGE (context:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::run_task_card_context",kind:"function"})
//! MERGE (mapError:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::map_run_task_card_error",kind:"function"})
//! MERGE (project:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::project_work_item",kind:"function"})
//! MERGE (port:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort",kind:"trait"})
//! MERGE (writerPort:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort.create",kind:"method"})
//! MERGE (transitionPort:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort.transition",kind:"method"})
//! MERGE (reviewPort:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort.review",kind:"method"})
//! MERGE (authorizeRunScope:Symbol {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::authorize_run_task_scope",kind:"function"})
//! MERGE (taskMetadata:Table {id:"multica.task_metadata"})
//! MERGE (taskLifecycle:Table {id:"multica.task_lifecycle_current"})
//! MERGE (worktreeTask:Table {id:"multica.work_item_worktree"})
//! MERGE (runBinding:Table {id:"multica.engineering_run_worktree_binding"})
//! MERGE (idempotency:Table {id:"multica.task_command_idempotency"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(router)
//! MERGE (module)-[:DEFINES]->(create)
//! MERGE (module)-[:DEFINES]->(list)
//! MERGE (module)-[:DEFINES]->(listRun)
//! MERGE (module)-[:DEFINES]->(get)
//! MERGE (module)-[:DEFINES]->(transition)
//! MERGE (module)-[:DEFINES]->(review)
//! MERGE (module)-[:DEFINES]->(authorizeWorktree)
//! MERGE (module)-[:DEFINES]->(authorizeRun)
//! MERGE (module)-[:DEFINES]->(load)
//! MERGE (module)-[:DEFINES]->(lookupIdempotency)
//! MERGE (module)-[:DEFINES]->(saveIdempotency)
//! MERGE (module)-[:DEFINES]->(context)
//! MERGE (module)-[:DEFINES]->(mapError)
//! MERGE (module)-[:DEFINES]->(project)
//! MERGE (router)-[:CALLS]->(create)
//! MERGE (router)-[:CALLS]->(list)
//! MERGE (router)-[:CALLS]->(listRun)
//! MERGE (router)-[:CALLS]->(get)
//! MERGE (router)-[:CALLS]->(transition)
//! MERGE (router)-[:CALLS]->(review)
//! MERGE (create)-[:CALLS]->(context)
//! MERGE (create)-[:CALLS]->(writerPort)
//! MERGE (create)-[:CALLS]->(mapError)
//! MERGE (transition)-[:CALLS]->(context)
//! MERGE (transition)-[:CALLS]->(transitionPort)
//! MERGE (transition)-[:CALLS]->(mapError)
//! MERGE (review)-[:CALLS]->(context)
//! MERGE (review)-[:CALLS]->(reviewPort)
//! MERGE (review)-[:CALLS]->(mapError)
//! MERGE (list)-[:CALLS]->(authorizeWorktree)
//! MERGE (list)-[:CALLS]->(authorizeRun)
//! MERGE (list)-[:CALLS]->(project)
//! MERGE (listRun)-[:CALLS]->(authorizeRunScope)
//! MERGE (listRun)-[:CALLS]->(project)
//! MERGE (get)-[:CALLS]->(authorizeWorktree)
//! MERGE (get)-[:CALLS]->(authorizeRun)
//! MERGE (get)-[:CALLS]->(load)
//! MERGE (authorizeRun)-[:CALLS]->(authorizeRunScope)
//! MERGE (list)-[:READS]->(taskMetadata)
//! MERGE (list)-[:READS]->(taskLifecycle)
//! MERGE (list)-[:READS]->(worktreeTask)
//! MERGE (list)-[:READS]->(runBinding)
//! MERGE (listRun)-[:READS]->(taskMetadata)
//! MERGE (listRun)-[:READS]->(taskLifecycle)
//! MERGE (authorizeRun)-[:READS]->(runBinding)
//! MERGE (load)-[:READS]->(taskMetadata)
//! MERGE (load)-[:READS]->(taskLifecycle)
//! MERGE (load)-[:READS]->(worktreeTask)
//! MERGE (lookupIdempotency)-[:READS]->(idempotency)
//! MERGE (saveIdempotency)-[:WRITES]->(idempotency)
//! @endcypher
//!
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use domain_work_item::run_task_card::{
    CreateRunTaskCardCommand, ReviewRunTaskCardCommand, RunTaskCardAiData,
    RunTaskCardCommandContext, RunTaskCardError, RunTaskCardItemType, RunTaskCardPriority,
    RunTaskCardReviewAction, RunTaskCardStatus, TransitionRunTaskCardCommand,
};
use domain_work_item::{TenantId, UserId, WorktreeId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Transaction};
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, AuthenticatedUser, GroupApiError,
    GroupApiState,
};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct CreateWorkItemBody {
    pub(super) item_type: String,
    pub(super) title: String,
    #[serde(default)]
    pub(super) description: String,
    #[serde(default = "default_priority")]
    pub(super) priority: String,
    #[serde(default)]
    pub(super) labels: Vec<String>,
    pub(super) ai_task_data: Option<AiTaskData>,
    pub(super) correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LifecycleTransitionBody {
    target_status: String,
    expected_version: i32,
    correlation_id: Option<Uuid>,
    reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReviewCommandBody {
    action: String,
    expected_version: i32,
    correlation_id: Option<Uuid>,
    reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct AiTaskData {
    pub(super) objective: String,
    pub(super) repository_scope: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
struct WorkItemListQuery {
    limit: Option<i64>,
    after: Option<Uuid>,
}

#[derive(Debug, FromRow)]
pub(super) struct WorktreeScope {
    pub(super) workspace_id: Uuid,
    pub(super) project_id: Uuid,
    pub(super) repo_id: Uuid,
}

#[derive(Debug, FromRow)]
struct IdempotencyRow {
    request_hash: Vec<u8>,
    response_body: Value,
}

#[derive(Debug, FromRow)]
struct WorkItemProjection {
    work_item_id: Uuid,
    item_type: String,
    title: String,
    description: String,
    priority: String,
    labels: Vec<String>,
    ai_task_data: Option<Value>,
    reporter_user_id: Uuid,
    status: String,
    review_state: String,
    active_worktree_id: Option<Uuid>,
    version: i32,
    lifecycle_updated_at: chrono::DateTime<chrono::Utc>,
    metadata_updated_at: chrono::DateTime<chrono::Utc>,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/worktrees/{worktree_id}/work-items",
            get(list_work_items).post(create_work_item),
        )
        .route(
            "/api/v1/engineering-runs/{run_id}/work-items",
            get(list_run_work_items),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}",
            get(get_work_item),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/lifecycle",
            post(transition_work_item),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/review",
            post(review_work_item),
        )
}

async fn create_work_item(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<CreateWorkItemBody>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:write")?;
    validate_create_body(&body)?;
    let worktree_id = parse_id(&worktree_id)?;
    let key = idempotency_key(&headers)?;
    let hash = request_hash(&format!("create:{worktree_id}"), &body)?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    let context = run_task_card_context(
        actor.tenant_id,
        actor.user_id,
        worktree_id,
        key,
        hash,
        correlation_id,
    )?;
    let command = CreateRunTaskCardCommand {
        item_type: RunTaskCardItemType::try_from(body.item_type.as_str())
            .map_err(map_run_task_card_error)?,
        title: body.title,
        description: body.description,
        priority: RunTaskCardPriority::try_from(body.priority.as_str())
            .map_err(map_run_task_card_error)?,
        labels: body.labels,
        ai_task_data: body.ai_task_data.map(|data| RunTaskCardAiData {
            objective: data.objective,
            repository_scope: data.repository_scope.into_iter().map(Into::into).collect(),
        }),
    };
    let receipt = state
        .run_task_card_command_port
        .create(context, command)
        .await
        .map_err(map_run_task_card_error)?;
    Ok(Json(
        serde_json::to_value(receipt.snapshot).map_err(|_| GroupApiError::internal())?,
    ))
}

async fn list_work_items(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    Query(query): Query<WorkItemListQuery>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let run_scope =
        authorize_task_run_for_worktree(&mut tx, &actor, worktree_id, &worktree).await?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    let rows = sqlx::query_as::<_, WorkItemProjection>(
        r#"
        SELECT m.work_item_id, m.item_type, m.title, m.description, m.priority, m.labels, m.ai_task_data,
               m.reporter_user_id, c.status, c.review_state, c.active_worktree_id,
               c.version, c.updated_at AS lifecycle_updated_at, m.updated_at AS metadata_updated_at
        FROM multica.work_item_worktree l
        JOIN multica.task_metadata m
          ON m.tenant_id = l.tenant_id AND m.work_item_id = l.work_item_id
         AND m.project_id = l.project_id AND m.valid_to IS NULL
        JOIN multica.engineering_run_worktree_binding run_binding
          ON run_binding.tenant_id = l.tenant_id AND run_binding.project_id = l.project_id
         AND run_binding.worktree_id = l.worktree_id AND run_binding.valid_to IS NULL
         AND run_binding.repository_id = m.repository_id AND run_binding.branch_id = m.branch_id
         AND run_binding.engineering_run_id = m.engineering_run_id
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3 AND l.valid_to IS NULL
          AND m.repository_id = $4 AND m.branch_id = $5 AND m.engineering_run_id = $6
          AND ($7::UUID IS NULL OR m.work_item_id < $7)
        ORDER BY m.work_item_id DESC
        LIMIT $8
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(run_scope.repository_id)
    .bind(run_scope.branch_id)
    .bind(run_scope.engineering_run_id)
    .bind(query.after)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let mut rows = rows;
    let has_more = rows.len() > limit as usize;
    if has_more {
        rows.pop();
    }
    let next_after = has_more
        .then(|| rows.last().map(|row| row.work_item_id))
        .flatten();
    let items = rows.into_iter().map(project_work_item).collect::<Vec<_>>();
    Ok(Json(json!({
        "worktree_id": worktree_id,
        "engineering_run_id": run_scope.engineering_run_id,
        "project_id": worktree.project_id,
        "role": binding.role,
        "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
        "run_role": run_scope.role,
        "run_permission_snapshot_ref": run_scope.permission_snapshot_ref,
        "limit": limit,
        "next_after": next_after,
        "work_items": items,
    })))
}

async fn list_run_work_items(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<WorkItemListQuery>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:read")?;
    let run_id = parse_id(&run_id)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    super::set_tenant(&mut tx, actor.tenant_id).await?;
    let run_scope =
        super::engineering_runs::authorize_run_task_scope(&mut tx, &actor, run_id).await?;
    let rows = sqlx::query_as::<_, WorkItemProjection>(
        r#"
        SELECT m.work_item_id, m.item_type, m.title, m.description, m.priority, m.labels, m.ai_task_data,
               m.reporter_user_id, c.status, c.review_state, c.active_worktree_id,
               c.version, c.updated_at AS lifecycle_updated_at, m.updated_at AS metadata_updated_at
        FROM multica.task_metadata m
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE m.tenant_id = $1 AND m.project_id = $2 AND m.repository_id = $3
          AND m.branch_id = $4 AND m.engineering_run_id = $5
          AND m.valid_to IS NULL AND ($6::UUID IS NULL OR m.work_item_id < $6)
        ORDER BY m.work_item_id DESC
        LIMIT $7
        "#,
    )
    .bind(actor.tenant_id)
    .bind(run_scope.project_id)
    .bind(run_scope.repository_id)
    .bind(run_scope.branch_id)
    .bind(run_scope.engineering_run_id)
    .bind(query.after)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let mut rows = rows;
    let has_more = rows.len() > limit as usize;
    if has_more {
        rows.pop();
    }
    let next_after = has_more
        .then(|| rows.last().map(|row| row.work_item_id))
        .flatten();
    let items = rows.into_iter().map(project_work_item).collect::<Vec<_>>();
    Ok(Json(json!({
        "engineering_run_id": run_scope.engineering_run_id,
        "project_id": run_scope.project_id,
        "repository_id": run_scope.repository_id,
        "branch_id": run_scope.branch_id,
        "role": run_scope.role,
        "permission_snapshot_ref": run_scope.permission_snapshot_ref,
        "limit": limit,
        "next_after": next_after,
        "work_items": items,
    })))
}

async fn get_work_item(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let work_item_id = parse_id(&work_item_id)?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let _run_scope =
        authorize_task_run_for_worktree(&mut tx, &actor, worktree_id, &worktree).await?;
    let _binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    let result = load_work_item(
        &mut tx,
        actor.tenant_id,
        worktree.project_id,
        worktree_id,
        work_item_id,
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(result))
}

async fn transition_work_item(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<LifecycleTransitionBody>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:write")?;
    validate_transition_body(&body)?;
    let worktree_id = parse_id(&worktree_id)?;
    let work_item_id = parse_id(&work_item_id)?;
    let key = idempotency_key(&headers)?;
    let hash = request_hash(&format!("transition:{worktree_id}:{work_item_id}"), &body)?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    let context = run_task_card_context(
        actor.tenant_id,
        actor.user_id,
        worktree_id,
        key,
        hash,
        correlation_id,
    )?;
    let command = TransitionRunTaskCardCommand {
        work_item_id: work_item_id.into(),
        target_status: RunTaskCardStatus::try_from(body.target_status.as_str())
            .map_err(map_run_task_card_error)?,
        expected_version: body.expected_version,
        reason: body.reason,
    };
    let receipt = state
        .run_task_card_command_port
        .transition(context, command)
        .await
        .map_err(map_run_task_card_error)?;
    Ok(Json(
        serde_json::to_value(receipt.snapshot).map_err(|_| GroupApiError::internal())?,
    ))
}
async fn review_work_item(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<ReviewCommandBody>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "work-item:write")?;
    validate_review_body(&body)?;
    let worktree_id = parse_id(&worktree_id)?;
    let work_item_id = parse_id(&work_item_id)?;
    let key = idempotency_key(&headers)?;
    let hash = request_hash(&format!("review:{worktree_id}:{work_item_id}"), &body)?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    let context = run_task_card_context(
        actor.tenant_id,
        actor.user_id,
        worktree_id,
        key,
        hash,
        correlation_id,
    )?;
    let command = ReviewRunTaskCardCommand {
        work_item_id: work_item_id.into(),
        action: RunTaskCardReviewAction::try_from(body.action.as_str())
            .map_err(map_run_task_card_error)?,
        expected_version: body.expected_version,
        reason: body.reason,
    };
    let receipt = state
        .run_task_card_command_port
        .review(context, command)
        .await
        .map_err(map_run_task_card_error)?;
    Ok(Json(
        serde_json::to_value(receipt.snapshot).map_err(|_| GroupApiError::internal())?,
    ))
}
pub(super) async fn authorize_task_run_for_worktree(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
    worktree: &WorktreeScope,
) -> Result<super::engineering_runs::RunTaskScope, GroupApiError> {
    let (run_id, repository_id, branch_id) = sqlx::query_as::<_, (Uuid, Uuid, Uuid)>(
        r#"
        SELECT engineering_run_id, repository_id, branch_id
        FROM multica.engineering_run_worktree_binding
        WHERE tenant_id = $1 AND project_id = $2 AND worktree_id = $3 AND valid_to IS NULL
        FOR SHARE
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;

    let scope = super::engineering_runs::authorize_run_task_scope(tx, actor, run_id).await?;
    if scope.repository_id != worktree.repo_id
        || scope.repository_id != repository_id
        || scope.branch_id != branch_id
        || scope.project_id != worktree.project_id
    {
        return Err(GroupApiError::not_found());
    }
    Ok(scope)
}

pub(super) async fn authorize_worktree(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    worktree_id: Uuid,
) -> Result<WorktreeScope, GroupApiError> {
    set_tenant(tx, actor.tenant_id).await?;
    sqlx::query_as::<_, WorktreeScope>(
        r#"
        SELECT w.workspace_id, p.project_id, w.repo_id
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        FOR SHARE OF w, p
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)
}

pub(super) async fn load_work_item(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
) -> Result<Value, GroupApiError> {
    let row = sqlx::query_as::<_, WorkItemProjection>(
        r#"
        SELECT m.work_item_id, m.item_type, m.title, m.description, m.priority, m.labels, m.ai_task_data,
               m.reporter_user_id, c.status, c.review_state, c.active_worktree_id,
               c.version, c.updated_at AS lifecycle_updated_at, m.updated_at AS metadata_updated_at
        FROM multica.work_item_worktree l
        JOIN multica.task_metadata m
          ON m.tenant_id = l.tenant_id AND m.work_item_id = l.work_item_id
         AND m.project_id = l.project_id AND m.valid_to IS NULL
        JOIN multica.engineering_run_worktree_binding run_binding
          ON run_binding.tenant_id = l.tenant_id AND run_binding.project_id = l.project_id
         AND run_binding.worktree_id = l.worktree_id AND run_binding.valid_to IS NULL
         AND run_binding.repository_id = m.repository_id AND run_binding.branch_id = m.branch_id
         AND run_binding.engineering_run_id = m.engineering_run_id
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3
          AND l.work_item_id = $4 AND l.valid_to IS NULL
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    Ok(project_work_item(row))
}

pub(super) async fn lookup_idempotency(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    key: &str,
    request_hash: &[u8],
) -> Result<Option<Value>, GroupApiError> {
    let previous = sqlx::query_as::<_, IdempotencyRow>(
        r#"
        SELECT request_hash, response_body
        FROM multica.task_command_idempotency
        WHERE tenant_id = $1 AND actor_id = $2 AND idempotency_key = $3 AND expires_at > now()
        "#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    match previous {
        Some(previous) if previous.request_hash.as_slice() == request_hash => {
            Ok(Some(previous.response_body))
        }
        Some(_) => Err(GroupApiError::conflict("idempotency_key_reused")),
        None => Ok(None),
    }
}

pub(super) async fn save_idempotency(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    key: &str,
    request_hash: &[u8],
    response: &Value,
) -> Result<(), GroupApiError> {
    sqlx::query(
        r#"
        DELETE FROM multica.task_command_idempotency
        WHERE tenant_id = $1 AND actor_id = $2 AND idempotency_key = $3 AND expires_at <= now()
        "#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(key)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"
        INSERT INTO multica.task_command_idempotency
            (tenant_id, actor_id, idempotency_key, request_hash, response_body)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(key)
    .bind(request_hash)
    .bind(response)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::conflict("idempotency_race"))?;
    Ok(())
}

pub(super) fn idempotency_key(headers: &HeaderMap) -> Result<String, GroupApiError> {
    let key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .ok_or_else(|| GroupApiError::invalid_request("idempotency_key_required"))?;
    Ok(key.to_owned())
}

pub(super) fn request_hash<T: Serialize>(
    operation: &str,
    body: &T,
) -> Result<Vec<u8>, GroupApiError> {
    let bytes = serde_json::to_vec(&(operation, body)).map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(bytes).to_vec())
}

pub(super) fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value).map_err(|_| GroupApiError::bad_request())
}

pub(super) fn validate_create_body(body: &CreateWorkItemBody) -> Result<(), GroupApiError> {
    const TYPES: &[&str] = &["epic", "story", "task", "bug", "subtask", "ai_task"];
    const PRIORITIES: &[&str] = &["low", "medium", "high", "urgent"];
    if !TYPES.contains(&body.item_type.as_str())
        || !PRIORITIES.contains(&body.priority.as_str())
        || body.title.trim().is_empty()
        || body.title.len() > 500
        || body.description.len() > 100_000
        || body.labels.len() > 100
        || body.labels.iter().any(|label| label.len() > 128)
        || (body.item_type == "ai_task") != body.ai_task_data.is_some()
        || body.ai_task_data.as_ref().is_some_and(|data| {
            data.objective.trim().is_empty()
                || data.objective.len() > 10_000
                || data.repository_scope.is_empty()
                || data.repository_scope.len() > 100
        })
    {
        return Err(GroupApiError::invalid_request("invalid_work_item"));
    }
    Ok(())
}

pub(super) fn validate_ai_task_scope(
    body: &CreateWorkItemBody,
    repository_id: Uuid,
) -> Result<(), GroupApiError> {
    if body.ai_task_data.as_ref().is_some_and(|data| {
        data.repository_scope.len() != 1 || data.repository_scope[0] != repository_id
    }) {
        return Err(GroupApiError::invalid_request(
            "ai_task_repository_out_of_scope",
        ));
    }
    Ok(())
}

fn validate_transition_body(body: &LifecycleTransitionBody) -> Result<(), GroupApiError> {
    const STATUSES: &[&str] = &[
        "pending",
        "claimed",
        "in_progress",
        "completed",
        "failed",
        "cancelled",
    ];
    if !STATUSES.contains(&body.target_status.as_str())
        || body.expected_version < 1
        || body
            .reason
            .as_ref()
            .is_some_and(|reason| reason.len() > 4_000)
    {
        return Err(GroupApiError::invalid_request("invalid_lifecycle_command"));
    }
    Ok(())
}

fn validate_review_body(body: &ReviewCommandBody) -> Result<(), GroupApiError> {
    if !["submit", "accept", "reject"].contains(&body.action.as_str())
        || body.expected_version < 1
        || body
            .reason
            .as_ref()
            .is_some_and(|reason| reason.len() > 4_000)
        || (body.action == "reject"
            && body
                .reason
                .as_ref()
                .is_none_or(|reason| reason.trim().is_empty()))
    {
        return Err(GroupApiError::invalid_request("invalid_review_command"));
    }
    Ok(())
}

fn project_work_item(row: WorkItemProjection) -> Value {
    json!({
        "work_item_id": row.work_item_id,
        "task_card_id": row.work_item_id,
        "item_type": row.item_type,
        "title": row.title,
        "description": row.description,
        "priority": row.priority,
        "labels": row.labels,
        "ai_task_data": row.ai_task_data,
        "reporter_user_id": row.reporter_user_id,
        "lifecycle": {
            "status": row.status,
            "review_state": row.review_state,
            "active_worktree_id": row.active_worktree_id,
            "version": row.version,
            "updated_at": row.lifecycle_updated_at,
        },
        "metadata_updated_at": row.metadata_updated_at,
    })
}

pub(super) fn run_task_card_context(
    tenant_id: Uuid,
    actor_id: Uuid,
    worktree_id: Uuid,
    idempotency_key: String,
    request_fingerprint: Vec<u8>,
    correlation_id: Uuid,
) -> Result<RunTaskCardCommandContext, GroupApiError> {
    let request_fingerprint: [u8; 32] = request_fingerprint
        .as_slice()
        .try_into()
        .map_err(|_| GroupApiError::internal())?;
    Ok(RunTaskCardCommandContext {
        tenant_id: TenantId(tenant_id),
        actor_id: UserId(actor_id),
        worktree_id: WorktreeId(worktree_id),
        idempotency_key,
        request_fingerprint,
        correlation_id,
    })
}

pub(super) fn map_run_task_card_error(error: RunTaskCardError) -> GroupApiError {
    GroupApiError::from_run_task_card(error)
}

fn default_priority() -> String {
    "medium".to_owned()
}

pub(super) fn require_task_writer(role: &str) -> Result<(), GroupApiError> {
    match role {
        "tenant_admin" | "project_admin" | "developer" | "agent" => Ok(()),
        _ => Err(GroupApiError::forbidden()),
    }
}
