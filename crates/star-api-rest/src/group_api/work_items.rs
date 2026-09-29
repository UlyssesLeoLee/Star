//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"work_items.rs",type:"file",language:"rust"}),(m:Module {name:"work_items",type:"module",language:"rust"}),
//!   (c:Class {name:"CreateWorkItemBody",type:"class"}),(t:Class {name:"LifecycleTransitionBody",type:"class"}),(a:Class {name:"AiTaskData",type:"class"}),(x:Class {name:"WorkItemListQuery",type:"class"}),(s:Class {name:"WorktreeScope",type:"class"}),(l:Class {name:"CurrentLifecycle",type:"class"}),(i:Class {name:"IdempotencyRow",type:"class"}),(p:Class {name:"WorkItemProjection",type:"class"}),
//!   (rt:Function {name:"router",type:"function"}),(cr:Function {name:"create_work_item",type:"function"}),(li:Function {name:"list_work_items",type:"function"}),(ge:Function {name:"get_work_item",type:"function"}),(tr:Function {name:"transition_work_item",type:"function"}),(au:Function {name:"authorize_worktree",type:"function"}),(ld:Function {name:"load_work_item",type:"function"}),(lk:Function {name:"lookup_idempotency",type:"function"}),(sv:Function {name:"save_idempotency",type:"function"}),(hk:Function {name:"idempotency_key",type:"function"}),(rh:Function {name:"request_hash",type:"function"}),(pi:Function {name:"parse_id",type:"function"}),(vc:Function {name:"validate_create_body",type:"function"}),(ais:Function {name:"validate_ai_task_scope",type:"function"}),(vtb:Function {name:"validate_transition_body",type:"function"}),(vt:Function {name:"valid_transition",type:"function"}),(pr:Function {name:"project_work_item",type:"function"}),(dp:Function {name:"default_priority",type:"function"}),(rw:Function {name:"require_task_writer",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(c),(m)-[:CONTAINS]->(t),(m)-[:CONTAINS]->(a),(m)-[:CONTAINS]->(x),(m)-[:CONTAINS]->(s),(m)-[:CONTAINS]->(l),(m)-[:CONTAINS]->(i),(m)-[:CONTAINS]->(p),
//!   (m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(cr),(m)-[:CONTAINS]->(li),(m)-[:CONTAINS]->(ge),(m)-[:CONTAINS]->(tr),(m)-[:CONTAINS]->(au),(m)-[:CONTAINS]->(ld),(m)-[:CONTAINS]->(lk),(m)-[:CONTAINS]->(sv),(m)-[:CONTAINS]->(hk),(m)-[:CONTAINS]->(rh),(m)-[:CONTAINS]->(pi),(m)-[:CONTAINS]->(vc),(m)-[:CONTAINS]->(ais),(m)-[:CONTAINS]->(vtb),(m)-[:CONTAINS]->(vt),(m)-[:CONTAINS]->(pr),(m)-[:CONTAINS]->(dp),(m)-[:CONTAINS]->(rw),
//!   (rt)-[:CALLS]->(cr),(rt)-[:CALLS]->(li),(rt)-[:CALLS]->(ge),(rt)-[:CALLS]->(tr),(cr)-[:CALLS]->(vc),(cr)-[:CALLS]->(au),(cr)-[:CALLS]->(rw),(cr)-[:CALLS]->(ais),(cr)-[:CALLS]->(pi),(cr)-[:CALLS]->(hk),(cr)-[:CALLS]->(rh),(cr)-[:CALLS]->(lk),(cr)-[:CALLS]->(sv),(cr)-[:CALLS]->(ld),(li)-[:CALLS]->(au),(li)-[:CALLS]->(pr),(ge)-[:CALLS]->(au),(ge)-[:CALLS]->(ld),(tr)-[:CALLS]->(vtb),(tr)-[:CALLS]->(au),(tr)-[:CALLS]->(rw),(tr)-[:CALLS]->(pi),(tr)-[:CALLS]->(hk),(tr)-[:CALLS]->(rh),(tr)-[:CALLS]->(lk),(tr)-[:CALLS]->(vt),(tr)-[:CALLS]->(sv),(tr)-[:CALLS]->(ld),(ld)-[:CALLS]->(pr),(vc)-[:CALLS]->(dp);
//! CREATE (rb:Class {name:"ReviewCommandBody",type:"class",language:"rust"}),(rv:Function {name:"review_work_item",type:"function",language:"rust"}),(vrb:Function {name:"validate_review_body",type:"function",language:"rust"}),(rr:Function {name:"require_task_reviewer",type:"function",language:"rust"});
//! MATCH (m:Module {name:"work_items",type:"module",language:"rust"}),(rt:Function {name:"router",type:"function"}),(rv:Function {name:"review_work_item",type:"function",language:"rust"}),(rb:Class {name:"ReviewCommandBody",type:"class",language:"rust"}),(vrb:Function {name:"validate_review_body",type:"function",language:"rust"}),(rr:Function {name:"require_task_reviewer",type:"function",language:"rust"}),(hk:Function {name:"request_hash",type:"function"}),(lk:Function {name:"lookup_idempotency",type:"function"}),(sv:Function {name:"save_idempotency",type:"function"}),(ld:Function {name:"load_work_item",type:"function"});
//! CREATE (m)-[:CONTAINS]->(rb),(m)-[:CONTAINS]->(rv),(m)-[:CONTAINS]->(vrb),(m)-[:CONTAINS]->(rr),(rt)-[:CALLS]->(rv),(rv)-[:USES]->(rb),(rv)-[:CALLS]->(vrb),(rv)-[:CALLS]->(rr),(rv)-[:CALLS]->(hk),(rv)-[:CALLS]->(lk),(rv)-[:CALLS]->(sv),(rv)-[:CALLS]->(ld);

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Transaction};
use uuid::Uuid;

use super::{
    AuthenticatedUser, GroupApiError, GroupApiState, active_binding, require_scope, set_tenant,
    validate_actor,
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
}

#[derive(Debug, FromRow)]
pub(super) struct WorktreeScope {
    pub(super) workspace_id: Uuid,
    pub(super) project_id: Uuid,
    pub(super) repo_id: Uuid,
}

#[derive(Debug, FromRow)]
struct CurrentLifecycle {
    status: String,
    review_state: String,
    active_worktree_id: Option<Uuid>,
    claimed_by: Option<Uuid>,
    claim_expired: bool,
    version: i32,
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
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    require_task_writer(&binding.role)?;
    validate_ai_task_scope(&body, worktree.repo_id)?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }

    let work_item_id = Uuid::new_v4();
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    sqlx::query(
        r#"
        INSERT INTO multica.task_metadata
            (work_item_id, tenant_id, workspace_id, project_id, item_type, title,
             description, priority, labels, ai_task_data, reporter_user_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(work_item_id)
    .bind(actor.tenant_id)
    .bind(worktree.workspace_id)
    .bind(worktree.project_id)
    .bind(&body.item_type)
    .bind(body.title.trim())
    .bind(&body.description)
    .bind(&body.priority)
    .bind(&body.labels)
    .bind(body.ai_task_data.as_ref().map(|value| json!(value)))
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    sqlx::query(
        r#"
        INSERT INTO multica.task_lifecycle_current (work_item_id, tenant_id, status, version)
        VALUES ($1, $2, 'pending', 1)
        "#,
    )
    .bind(work_item_id)
    .bind(actor.tenant_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    sqlx::query(
        r#"
        INSERT INTO multica.work_item_worktree (tenant_id, project_id, work_item_id, worktree_id)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(work_item_id)
    .bind(worktree_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    sqlx::query(
        r#"
        INSERT INTO multica.task_lifecycle_audit
            (tenant_id, project_id, worktree_id, work_item_id, task_card_id,
             to_status, event_type, actor_id, idempotency_key, correlation_id, details)
        VALUES ($1, $2, $3, $4, $4, 'pending', 'created', $5, $6, $7, $8)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .bind(actor.user_id)
    .bind(&key)
    .bind(correlation_id)
    .bind(json!({ "source": "task_card" }))
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    let response = load_work_item(
        &mut tx,
        actor.tenant_id,
        worktree.project_id,
        worktree_id,
        work_item_id,
    )
    .await?;
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
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
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3 AND l.valid_to IS NULL
        ORDER BY c.updated_at DESC, m.work_item_id DESC
        LIMIT $4
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let items = rows.into_iter().map(project_work_item).collect::<Vec<_>>();
    Ok(Json(json!({
        "worktree_id": worktree_id,
        "project_id": worktree.project_id,
        "role": binding.role,
        "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
        "limit": limit,
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
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    require_task_writer(&binding.role)?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }

    let current = sqlx::query_as::<_, CurrentLifecycle>(
        r#"
        SELECT c.status, c.review_state, c.active_worktree_id, c.claimed_by,
               COALESCE(c.claim_expires_at <= now(), FALSE) AS claim_expired, c.version
        FROM multica.task_lifecycle_current c
        JOIN multica.task_metadata m
          ON m.tenant_id = c.tenant_id AND m.work_item_id = c.work_item_id AND m.valid_to IS NULL
        JOIN multica.work_item_worktree l
          ON l.tenant_id = c.tenant_id AND l.work_item_id = c.work_item_id
         AND l.project_id = m.project_id AND l.worktree_id = $3 AND l.valid_to IS NULL
        WHERE c.tenant_id = $1 AND c.work_item_id = $2 AND m.project_id = $4
        FOR UPDATE OF c
        "#,
    )
    .bind(actor.tenant_id)
    .bind(work_item_id)
    .bind(worktree_id)
    .bind(worktree.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;

    // A concurrent retry can wait for the lifecycle row lock while the first request commits.
    // Re-read idempotency after acquiring that lock before evaluating the now-stale version.
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    if current.version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    if matches!(current.status.as_str(), "claimed" | "in_progress")
        && current.active_worktree_id != Some(worktree_id)
    {
        return Err(GroupApiError::conflict("active_worktree_conflict"));
    }
    if current.review_state == "pending_review" && body.target_status == "completed" {
        return Err(GroupApiError::conflict("review_required"));
    }
    if current.status == "claimed"
        && body.target_status == "in_progress"
        && (current.claimed_by != Some(actor.user_id) || current.claim_expired)
    {
        return Err(GroupApiError::conflict(
            "claim_expired_or_owned_by_other_actor",
        ));
    }
    let expired_claim_requeue =
        current.status == "claimed" && body.target_status == "pending" && current.claim_expired;
    if !valid_transition(&current.status, &body.target_status) && !expired_claim_requeue {
        return Err(GroupApiError::conflict("invalid_transition"));
    }

    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    sqlx::query(
        r#"
        UPDATE multica.task_lifecycle_current
        SET status = $4,
            active_worktree_id = CASE WHEN $4 IN ('claimed','in_progress') THEN $3 ELSE NULL END,
            claimed_by = CASE WHEN $4 = 'claimed' THEN $5
                              WHEN $4 = 'in_progress' THEN claimed_by
                              ELSE NULL END,
            claim_expires_at = CASE WHEN $4 = 'claimed' THEN now() + INTERVAL '30 seconds'
                                    ELSE NULL END,
            version = version + 1,
            expires_at = CASE WHEN $4 IN ('completed','failed','cancelled') THEN now() + retention_period ELSE NULL END,
            updated_at = now()
        WHERE tenant_id = $1 AND work_item_id = $2 AND version = $6
        "#,
    )
    .bind(actor.tenant_id)
    .bind(work_item_id)
    .bind(worktree_id)
    .bind(&body.target_status)
    .bind(actor.user_id)
    .bind(body.expected_version)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    sqlx::query(
        r#"
        INSERT INTO multica.task_lifecycle_audit
            (tenant_id, project_id, worktree_id, work_item_id, task_card_id,
             from_status, to_status, event_type, actor_id, idempotency_key, correlation_id, details)
        VALUES ($1, $2, $3, $4, $4, $5, $6, 'status_transition', $7, $8, $9, $10)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .bind(&current.status)
    .bind(&body.target_status)
    .bind(actor.user_id)
    .bind(&key)
    .bind(correlation_id)
    .bind(json!({ "reason": body.reason }))
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    let response = load_work_item(
        &mut tx,
        actor.tenant_id,
        worktree.project_id,
        worktree_id,
        work_item_id,
    )
    .await?;
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
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
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let worktree = authorize_worktree(&mut tx, &actor, worktree_id).await?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    if body.action == "submit" {
        require_task_writer(&binding.role)?;
    } else {
        require_task_reviewer(&binding.role)?;
    }
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }

    let current = sqlx::query_as::<_, CurrentLifecycle>(
        r#"
        SELECT c.status, c.review_state, c.active_worktree_id, c.claimed_by,
               COALESCE(c.claim_expires_at <= now(), FALSE) AS claim_expired, c.version
        FROM multica.task_lifecycle_current c
        JOIN multica.task_metadata m
          ON m.tenant_id = c.tenant_id AND m.work_item_id = c.work_item_id AND m.valid_to IS NULL
        JOIN multica.work_item_worktree l
          ON l.tenant_id = c.tenant_id AND l.work_item_id = c.work_item_id
         AND l.project_id = m.project_id AND l.worktree_id = $3 AND l.valid_to IS NULL
        WHERE c.tenant_id = $1 AND c.work_item_id = $2 AND m.project_id = $4
        FOR UPDATE OF c
        "#,
    )
    .bind(actor.tenant_id)
    .bind(work_item_id)
    .bind(worktree_id)
    .bind(worktree.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;

    // A concurrent retry can wait for the lifecycle row lock while the first request commits.
    // Re-read idempotency before evaluating the now-stale version.
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    if current.version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    if current.status != "in_progress" || current.active_worktree_id != Some(worktree_id) {
        return Err(GroupApiError::conflict("review_worktree_conflict"));
    }

    let (next_status, next_review_state, event_type) = match body.action.as_str() {
        "submit"
            if current.claimed_by == Some(actor.user_id)
                && !current.claim_expired
                && matches!(current.review_state.as_str(), "none" | "rejected") =>
        {
            ("in_progress", "pending_review", "review_requested")
        }
        "submit" => return Err(GroupApiError::forbidden()),
        "accept"
            if current.review_state == "pending_review"
                && current.claimed_by != Some(actor.user_id) =>
        {
            ("completed", "accepted", "review_accepted")
        }
        "reject"
            if current.review_state == "pending_review"
                && current.claimed_by != Some(actor.user_id) =>
        {
            ("failed", "rejected", "review_rejected")
        }
        "accept" | "reject" if current.claimed_by == Some(actor.user_id) => {
            return Err(GroupApiError::forbidden());
        }
        _ => return Err(GroupApiError::conflict("review_not_pending")),
    };

    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    sqlx::query(
        r#"
        UPDATE multica.task_lifecycle_current
        SET status = $4,
            review_state = $5,
            active_worktree_id = CASE WHEN $6 THEN NULL ELSE active_worktree_id END,
            claimed_by = CASE WHEN $6 THEN NULL ELSE claimed_by END,
            claim_expires_at = CASE WHEN $6 THEN NULL ELSE claim_expires_at END,
            version = version + 1,
            expires_at = CASE WHEN $6 THEN now() + retention_period ELSE NULL END,
            updated_at = now()
        WHERE tenant_id = $1 AND work_item_id = $2 AND version = $3
        "#,
    )
    .bind(actor.tenant_id)
    .bind(work_item_id)
    .bind(body.expected_version)
    .bind(next_status)
    .bind(next_review_state)
    .bind(body.action != "submit")
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    sqlx::query(
        r#"
        INSERT INTO multica.task_lifecycle_audit
            (tenant_id, project_id, worktree_id, work_item_id, task_card_id,
             from_status, to_status, event_type, actor_id, idempotency_key, correlation_id, details)
        VALUES ($1, $2, $3, $4, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .bind(&current.status)
    .bind(next_status)
    .bind(event_type)
    .bind(actor.user_id)
    .bind(&key)
    .bind(correlation_id)
    .bind(json!({ "review_state": next_review_state, "reason": body.reason }))
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    let response = load_work_item(
        &mut tx,
        actor.tenant_id,
        worktree.project_id,
        worktree_id,
        work_item_id,
    )
    .await?;
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
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

fn valid_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("pending", "claimed")
            | ("pending", "cancelled")
            | ("claimed", "in_progress")
            | ("claimed", "cancelled")
            | ("in_progress", "completed")
            | ("in_progress", "failed")
            | ("in_progress", "cancelled")
            | ("failed", "pending")
    )
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

fn default_priority() -> String {
    "medium".to_owned()
}

pub(super) fn require_task_writer(role: &str) -> Result<(), GroupApiError> {
    match role {
        "tenant_admin" | "project_admin" | "developer" | "agent" => Ok(()),
        _ => Err(GroupApiError::forbidden()),
    }
}

fn require_task_reviewer(role: &str) -> Result<(), GroupApiError> {
    match role {
        "tenant_admin" | "project_admin" | "developer" => Ok(()),
        _ => Err(GroupApiError::forbidden()),
    }
}
