//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"canvas.rs",type:"file",language:"rust"}),(m:Module {name:"canvas",type:"module",language:"rust"}),
//!   (cb:Class {name:"CreateCanvasBody",type:"class"}),(eb:Class {name:"CreateElementBody",type:"class"}),(ub:Class {name:"UpdateElementBody",type:"class"}),(er:Class {name:"EntityRefBody",type:"class"}),(sb:Class {name:"SetEntityRefBody",type:"class"}),(xb:Class {name:"ClearEntityRefBody",type:"class"}),(db:Class {name:"DeleteElementBody",type:"class"}),(lr:Class {name:"CanvasListRow",type:"class"}),(cv:Class {name:"CurrentCanvasVersion",type:"class"}),(cew:Class {name:"CurrentElement",type:"class"}),(cer:Class {name:"CurrentEntityRef",type:"class"}),(row:Class {name:"CanvasElementRow",type:"class"}),(cbw:Class {name:"CreateCanvasWorkItemBody",type:"class"}),(eq:Class {name:"CanvasEventsQuery",type:"class"}),(orow:Class {name:"CanvasOutboxRow",type:"class"}),(viewport:Class {name:"CanvasViewport",type:"class"}),(frame:Class {name:"CanvasFrameBody",type:"class"}),(connector:Class {name:"CanvasConnectorBody",type:"class"}),(documentBody:Class {name:"UpdateCanvasDocumentBody",type:"class"}),(documentRow:Class {name:"CurrentCanvasDocument",type:"class"}),
//!   (rt:Function {name:"router",type:"function"}),(lc:Function {name:"list_canvases",type:"function"}),(cc:Function {name:"create_canvas",type:"function"}),(ev:Function {name:"list_canvas_events",type:"function"}),(cw:Function {name:"create_canvas_work_item",type:"function"}),(cwr:Function {name:"create_work_item_records",type:"function"}),(le:Function {name:"list_elements",type:"function"}),(ce:Function {name:"create_element",type:"function"}),(ue:Function {name:"update_element",type:"function"}),(de:Function {name:"delete_element",type:"function"}),(sr:Function {name:"set_entity_ref",type:"function"}),(cr:Function {name:"clear_entity_ref",type:"function"}),(authz:Function {name:"authorize_group_scope",type:"function"}),(rw:Function {name:"request_worktree",type:"function"}),(rc:Function {name:"require_canvas",type:"function"}),(ve:Function {name:"validate_entity_ref",type:"function"}),(ir:Function {name:"insert_entity_ref",type:"function"}),(ae:Function {name:"append_event",type:"function"}),(lock:Function {name:"lock_element",type:"function"}),(close:Function {name:"close_element_version",type:"function"}),(bump:Function {name:"bump_element_version",type:"function"}),(ld:Function {name:"load_element",type:"function"}),(qe:Function {name:"query_elements",type:"function"}),(ep:Function {name:"element_projection",type:"function"}),(hs:Function {name:"has_scope",type:"function"}),(veb:Function {name:"validate_element_body",type:"function"}),(vr:Function {name:"validate_required_ref",type:"function"}),(ii:Function {name:"identity_key_present",type:"function"}),(eo:Function {name:"empty_object",type:"function"}),(ud:Function {name:"update_canvas_document",type:"function"}),(vdoc:Function {name:"validate_canvas_document",type:"function"}),(vrefs:Function {name:"validate_canvas_document_references",type:"function"}),(color:Function {name:"is_safe_hex_color",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(cb),(m)-[:CONTAINS]->(eb),(m)-[:CONTAINS]->(ub),(m)-[:CONTAINS]->(er),(m)-[:CONTAINS]->(sb),(m)-[:CONTAINS]->(xb),(m)-[:CONTAINS]->(db),(m)-[:CONTAINS]->(lr),(m)-[:CONTAINS]->(cv),(m)-[:CONTAINS]->(cew),(m)-[:CONTAINS]->(cer),(m)-[:CONTAINS]->(row),(m)-[:CONTAINS]->(cbw),(m)-[:CONTAINS]->(eq),(m)-[:CONTAINS]->(orow),(m)-[:CONTAINS]->(viewport),(m)-[:CONTAINS]->(frame),(m)-[:CONTAINS]->(connector),(m)-[:CONTAINS]->(documentBody),(m)-[:CONTAINS]->(documentRow),
//!   (m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(lc),(m)-[:CONTAINS]->(cc),(m)-[:CONTAINS]->(ev),(m)-[:CONTAINS]->(cw),(m)-[:CONTAINS]->(le),(m)-[:CONTAINS]->(ce),(m)-[:CONTAINS]->(ue),(m)-[:CONTAINS]->(de),(m)-[:CONTAINS]->(sr),(m)-[:CONTAINS]->(cr),(m)-[:CONTAINS]->(authz),(m)-[:CONTAINS]->(rw),(m)-[:CONTAINS]->(rc),(m)-[:CONTAINS]->(ve),(m)-[:CONTAINS]->(ir),(m)-[:CONTAINS]->(ae),(m)-[:CONTAINS]->(lock),(m)-[:CONTAINS]->(close),(m)-[:CONTAINS]->(bump),(m)-[:CONTAINS]->(ld),(m)-[:CONTAINS]->(qe),(m)-[:CONTAINS]->(ep),(m)-[:CONTAINS]->(hs),(m)-[:CONTAINS]->(veb),(m)-[:CONTAINS]->(vr),(m)-[:CONTAINS]->(ii),(m)-[:CONTAINS]->(eo),(m)-[:CONTAINS]->(ud),(m)-[:CONTAINS]->(vdoc),(m)-[:CONTAINS]->(vrefs),(m)-[:CONTAINS]->(color),
//!   (rt)-[:CALLS]->(lc),(rt)-[:CALLS]->(cc),(rt)-[:CALLS]->(ev),(rt)-[:CALLS]->(cw),(rt)-[:CALLS]->(le),(rt)-[:CALLS]->(ce),(rt)-[:CALLS]->(ue),(rt)-[:CALLS]->(de),(rt)-[:CALLS]->(sr),(rt)-[:CALLS]->(cr),(rt)-[:CALLS]->(ud),(lc)-[:CALLS]->(rw),(lc)-[:CALLS]->(authz),(lc)-[:CALLS]->(rc),(cc)-[:CALLS]->(rw),(cc)-[:CALLS]->(authz),(cc)-[:CALLS]->(ae),(ev)-[:CALLS]->(rw),(ev)-[:CALLS]->(authz),(cw)-[:CALLS]->(rw),(cw)-[:CALLS]->(authz),(cw)-[:CALLS]->(rc),(cw)-[:CALLS]->(ve),(cw)-[:CALLS]->(ir),(cw)-[:CALLS]->(ae),(cw)-[:CALLS]->(cwr),(ce)-[:CALLS]->(authz),(ce)-[:CALLS]->(rc),(ce)-[:CALLS]->(veb),(ce)-[:CALLS]->(ve),(ce)-[:CALLS]->(ir),(ce)-[:CALLS]->(ae),(ue)-[:CALLS]->(authz),(ue)-[:CALLS]->(rc),(ue)-[:CALLS]->(lock),(ue)-[:CALLS]->(close),(ue)-[:CALLS]->(ae),(de)-[:CALLS]->(authz),(de)-[:CALLS]->(lock),(de)-[:CALLS]->(close),(de)-[:CALLS]->(ae),(sr)-[:CALLS]->(authz),(sr)-[:CALLS]->(lock),(sr)-[:CALLS]->(ve),(sr)-[:CALLS]->(ir),(sr)-[:CALLS]->(bump),(sr)-[:CALLS]->(ae),(cr)-[:CALLS]->(authz),(cr)-[:CALLS]->(lock),(cr)-[:CALLS]->(vr),(cr)-[:CALLS]->(bump),(cr)-[:CALLS]->(ae),(le)-[:CALLS]->(authz),(le)-[:CALLS]->(rc),(le)-[:CALLS]->(qe),(qe)-[:CALLS]->(hs),(ld)-[:CALLS]->(qe),(bump)-[:CALLS]->(close),(authz)-[:CALLS]->(ve),(ud)-[:CALLS]->(rw),(ud)-[:CALLS]->(authz),(ud)-[:CALLS]->(rc),(ud)-[:CALLS]->(vdoc),(ud)-[:CALLS]->(vrefs),(ud)-[:CALLS]->(ae),(ud)-[:CALLS]->(color),(vdoc)-[:CALLS]->(color);

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

use super::work_items::{
    authorize_worktree, create_work_item_records, idempotency_key, lookup_idempotency, parse_id,
    request_hash, require_task_writer, save_idempotency, CreateWorkItemBody, WorktreeScope,
};
use super::{
    active_binding, require_scope, validate_actor, AuthUser, AuthenticatedUser, GroupApiError,
    GroupApiState,
};

#[derive(Debug, Deserialize, Serialize)]
struct CreateCanvasBody {
    title: String,
    correlation_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanvasViewport {
    x: f64,
    y: f64,
    zoom: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanvasFrameBody {
    id: Uuid,
    title: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    #[serde(default)]
    element_ids: Vec<Uuid>,
    #[serde(default)]
    is_slide: bool,
    #[serde(default)]
    order: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanvasConnectorBody {
    id: Uuid,
    kind: String,
    from_element_id: Uuid,
    to_element_id: Uuid,
    routing: String,
    arrow_start: bool,
    arrow_end: bool,
    color: String,
    width: f64,
    label: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateCanvasDocumentBody {
    expected_version: i64,
    viewport: CanvasViewport,
    frames: Vec<CanvasFrameBody>,
    connectors: Vec<CanvasConnectorBody>,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateCanvasWorkItemBody {
    work_item: CreateWorkItemBody,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    #[serde(default)]
    rotation: f64,
    #[serde(default)]
    z_index: i32,
    #[serde(default = "empty_object")]
    content: Value,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    hidden: bool,
}

#[derive(Debug, Deserialize)]
struct CanvasEventsQuery {
    after_offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EntityRefBody {
    ref_type: String,
    ref_id: Uuid,
    worktree_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateElementBody {
    element_id: Option<Uuid>,
    kind: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    #[serde(default)]
    rotation: f64,
    #[serde(default)]
    z_index: i32,
    #[serde(default = "empty_object")]
    content: Value,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    hidden: bool,
    entity_ref: Option<EntityRefBody>,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateElementBody {
    expected_version: i64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    z_index: i32,
    content: Value,
    locked: bool,
    hidden: bool,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SetEntityRefBody {
    ref_type: String,
    ref_id: Uuid,
    worktree_id: Uuid,
    expected_version: i64,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClearEntityRefBody {
    expected_version: i64,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeleteElementBody {
    expected_version: i64,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, FromRow, Serialize)]
struct CanvasListRow {
    canvas_id: Uuid,
    title: String,
    version: i64,
    created_by: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    element_count: i64,
    document_version: i64,
    viewport: Value,
    frames: Value,
    connectors: Value,
}

#[derive(Debug, FromRow)]
struct CanvasOutboxRow {
    stream_offset: i64,
    event_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Option<Uuid>,
    event_type: String,
    schema_version: i32,
    aggregate_version: i64,
    actor_id: Uuid,
    correlation_id: Uuid,
    payload: Value,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
struct CurrentCanvasVersion {
    version: i64,
}

#[derive(Debug, FromRow)]
struct CurrentCanvasDocument {
    document_version: i64,
}

#[derive(Debug, FromRow)]
struct CurrentElement {
    kind: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    z_index: i32,
    content: Value,
    locked: bool,
    hidden: bool,
    version: i64,
}

#[derive(Debug, FromRow)]
struct CurrentEntityRef {
    ref_type: String,
    ref_id: Uuid,
    version: i64,
}

#[derive(Debug, FromRow)]
struct CanvasElementRow {
    worktree_id: Uuid,
    element_id: Uuid,
    kind: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    z_index: i32,
    content: Value,
    locked: bool,
    hidden: bool,
    version: i64,
    created_by: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    entity_ref_type: Option<String>,
    entity_ref_id: Option<Uuid>,
    entity_ref_version: Option<i64>,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/worktrees/{worktree_id}/canvases",
            get(list_canvases).post(create_canvas),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/document",
            put(update_canvas_document),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/canvas-events",
            get(list_canvas_events),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/work-items",
            post(create_canvas_work_item),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements",
            get(list_elements).post(create_element),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements/{element_id}",
            put(update_element).delete(delete_element),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements/{element_id}/entity-ref",
            put(set_entity_ref).delete(clear_entity_ref),
        )
}

async fn list_canvas_events(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    Query(query): Query<CanvasEventsQuery>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:read")?;
    let after_offset = query.after_offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    if after_offset < 0 || !(1..=100).contains(&limit) {
        return Err(GroupApiError::invalid_request(
            "invalid_canvas_event_cursor",
        ));
    }
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, false).await?;
    let mut rows = sqlx::query_as::<_, CanvasOutboxRow>(
        r#"SELECT stream_offset,event_id,project_id,worktree_id,canvas_id,element_id,event_type,
                  schema_version,aggregate_version,actor_id,correlation_id,payload,occurred_at
           FROM canvas.canvas_group_outbox
           WHERE tenant_id=$1 AND worktree_id=$2 AND stream_offset>$3
           ORDER BY stream_offset
           LIMIT $4"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(after_offset)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let has_more = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    let next_offset = rows
        .last()
        .map(|row| row.stream_offset)
        .unwrap_or(after_offset);
    let events = rows
        .into_iter()
        .map(|row| {
            json!({
                "stream_offset": row.stream_offset.to_string(),
                "event_id": row.event_id,
                "project_id": row.project_id,
                "worktree_id": row.worktree_id,
                "canvas_id": row.canvas_id,
                "element_id": row.element_id,
                "event_type": row.event_type,
                "schema_version": row.schema_version,
                "aggregate_version": row.aggregate_version,
                "actor_id": row.actor_id,
                "correlation_id": row.correlation_id,
                "payload": row.payload,
                "occurred_at": row.occurred_at,
            })
        })
        .collect::<Vec<_>>();
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(json!({
        "worktree_id": worktree_id,
        "project_id": scope.project_id,
        "events": events,
        "has_more": has_more,
        "next_offset": next_offset.to_string(),
    })))
}

async fn list_canvases(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:read")?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, false).await?;
    let rows = sqlx::query_as::<_, CanvasListRow>(
        r#"
        SELECT c.canvas_id, c.title, c.version, c.created_by, c.created_at,
               d.document_version, d.viewport, d.frames, d.connectors,
               (SELECT count(*) FROM canvas.canvas_elements_backend e
                WHERE e.tenant_id = c.tenant_id AND e.worktree_id = c.worktree_id
                  AND e.canvas_id = c.canvas_id AND e.is_current) AS element_count
        FROM canvas.group_canvas_registry c
        JOIN canvas.canvas_document_state d
          ON d.tenant_id=c.tenant_id AND d.worktree_id=c.worktree_id
         AND d.canvas_id=c.canvas_id AND d.is_current AND d.valid_to IS NULL
        WHERE c.tenant_id = $1 AND c.worktree_id = $2 AND c.is_current AND c.valid_to IS NULL
        ORDER BY c.created_at DESC, c.canvas_id
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let canvases = rows
        .into_iter()
        .map(|row| {
            json!({
                "canvas_id": row.canvas_id,
                "title": row.title,
                "version": row.version,
                "created_by": row.created_by,
                "created_at": row.created_at,
                "element_count": row.element_count,
                "document": {
                    "version": row.document_version,
                    "viewport": row.viewport,
                    "frames": row.frames,
                    "connectors": row.connectors,
                },
            })
        })
        .collect::<Vec<_>>();
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(
        json!({"worktree_id": worktree_id, "project_id": scope.project_id, "canvases": canvases}),
    ))
}

async fn create_canvas(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<CreateCanvasBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    if body.title.trim().is_empty() || body.title.trim().len() > 200 {
        return Err(GroupApiError::invalid_request("invalid_canvas_title"));
    }
    let key = idempotency_key(&headers)?;
    let hash = request_hash(&format!("canvas:create:{worktree_id}"), &body)?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }

    let canvas_id = Uuid::new_v4();
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    sqlx::query(
        r#"
        INSERT INTO canvas.group_canvas_registry
            (tenant_id, worktree_id, canvas_id, version, title, created_by)
        VALUES ($1, $2, $3, 1, $4, $5)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(body.title.trim())
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"INSERT INTO canvas.canvas_document_state
           (tenant_id,worktree_id,canvas_id,document_version,viewport,frames,connectors,updated_by)
           VALUES ($1,$2,$3,1,'{"x":0,"y":0,"zoom":1}'::JSONB,'[]'::JSONB,'[]'::JSONB,$4)"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let response = json!({
        "canvas": {"canvas_id": canvas_id, "worktree_id": worktree_id, "project_id": scope.project_id,
                   "title": body.title.trim(), "version": 1, "created_by": actor.user_id}
    });
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        None,
        "canvas.created",
        correlation_id,
        1,
        json!({"title": body.title.trim()}),
    )
    .await?;
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn update_canvas_document(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<UpdateCanvasDocumentBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    let canvas_id = parse_id(&canvas_id)?;
    validate_canvas_document(&body)?;
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:document:update:{worktree_id}:{canvas_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }

    let current = sqlx::query_as::<_, CurrentCanvasDocument>(
        r#"SELECT document_version FROM canvas.canvas_document_state
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3
             AND is_current AND valid_to IS NULL FOR UPDATE"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if current.document_version != body.expected_version {
        return Err(GroupApiError::conflict("canvas_document_version_conflict"));
    }
    validate_canvas_document_references(&mut tx, actor.tenant_id, worktree_id, canvas_id, &body)
        .await?;

    let next_version = current.document_version + 1;
    let viewport = serde_json::to_value(&body.viewport).map_err(|_| GroupApiError::internal())?;
    let frames = serde_json::to_value(&body.frames).map_err(|_| GroupApiError::internal())?;
    let connectors =
        serde_json::to_value(&body.connectors).map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"UPDATE canvas.canvas_document_state
           SET valid_to=GREATEST(clock_timestamp(), valid_from + interval '1 microsecond'), is_current=FALSE
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND is_current AND valid_to IS NULL"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"INSERT INTO canvas.canvas_document_state
           (tenant_id,worktree_id,canvas_id,document_version,viewport,frames,connectors,updated_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(next_version)
    .bind(&viewport)
    .bind(&frames)
    .bind(&connectors)
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        None,
        "canvas.document.updated",
        correlation_id,
        next_version,
        json!({"document_version": next_version, "frame_count": body.frames.len(),
               "connector_count": body.connectors.len()}),
    )
    .await?;
    let response = json!({
        "canvas_id": canvas_id,
        "worktree_id": worktree_id,
        "document": {
            "version": next_version,
            "viewport": viewport,
            "frames": frames,
            "connectors": connectors,
        },
        "correlation_id": correlation_id,
    });
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn create_canvas_work_item(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<CreateCanvasWorkItemBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    require_scope(&actor, "work-item:write")?;
    require_scope(&actor, "work-item:read")?;
    let canvas_id = parse_id(&canvas_id)?;
    validate_element_body(
        "work_item_card",
        body.x,
        body.y,
        body.width,
        body.height,
        body.rotation,
        &body.content,
    )?;
    validate_required_ref("work_item_card", Some("work_item"))?;
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:work-item:create:{worktree_id}:{canvas_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }

    let correlation_id = body.work_item.correlation_id.unwrap_or_else(Uuid::new_v4);
    let (work_item_id, work_item) = create_work_item_records(
        &mut tx,
        &actor,
        worktree_id,
        &scope,
        &body.work_item,
        &key,
        correlation_id,
        "canvas",
    )
    .await?;
    let element_id = Uuid::new_v4();
    let entity_ref = EntityRefBody {
        ref_type: "work_item".to_owned(),
        ref_id: work_item_id,
        worktree_id,
    };
    validate_entity_ref(
        &mut tx,
        &actor,
        actor.tenant_id,
        &scope,
        worktree_id,
        &entity_ref,
    )
    .await?;
    sqlx::query(
        r#"INSERT INTO canvas.canvas_elements_backend
           (tenant_id,worktree_id,canvas_id,element_id,kind,x,y,width,height,rotation,z_index,
            content,locked,hidden,version,created_by)
           VALUES ($1,$2,$3,$4,'work_item_card',$5,$6,$7,$8,$9,$10,$11,$12,$13,1,$14)"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .bind(body.x)
    .bind(body.y)
    .bind(body.width)
    .bind(body.height)
    .bind(body.rotation)
    .bind(body.z_index)
    .bind(&body.content)
    .bind(body.locked)
    .bind(body.hidden)
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    insert_entity_ref(
        &mut tx,
        actor.tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        &entity_ref,
        1,
        actor.user_id,
    )
    .await?;
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        "canvas.work_item.created",
        correlation_id,
        1,
        json!({"work_item_id": work_item_id, "kind": "work_item_card"}),
    )
    .await?;
    let element = load_element(
        &mut tx,
        actor.tenant_id,
        &scope,
        worktree_id,
        canvas_id,
        element_id,
        true,
    )
    .await?;
    let response = json!({
        "work_item": work_item,
        "element": element,
        "correlation_id": correlation_id,
    });
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn list_elements(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id)): Path<(String, String)>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:read")?;
    let canvas_id = parse_id(&canvas_id)?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, false).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    let elements = query_elements(
        &mut tx,
        actor.tenant_id,
        &scope,
        worktree_id,
        canvas_id,
        None,
        has_scope(&actor, "work-item:read"),
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(
        json!({"canvas_id": canvas_id, "worktree_id": worktree_id,
                   "elements": elements.into_iter().map(element_projection).collect::<Vec<_>>()}),
    ))
}

async fn create_element(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<CreateElementBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    let canvas_id = parse_id(&canvas_id)?;
    validate_element_body(
        &body.kind,
        body.x,
        body.y,
        body.width,
        body.height,
        body.rotation,
        &body.content,
    )?;
    validate_required_ref(
        &body.kind,
        body.entity_ref.as_ref().map(|r| r.ref_type.as_str()),
    )?;
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:element:create:{worktree_id}:{canvas_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    if let Some(entity_ref) = &body.entity_ref {
        validate_entity_ref(
            &mut tx,
            &actor,
            actor.tenant_id,
            &scope,
            worktree_id,
            entity_ref,
        )
        .await?;
    }

    let element_id = body.element_id.unwrap_or_else(Uuid::new_v4);
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    sqlx::query(
        r#"
        INSERT INTO canvas.canvas_elements_backend
            (tenant_id, worktree_id, canvas_id, element_id, kind, x, y, width, height,
             rotation, z_index, content, locked, hidden, version, created_by)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,1,$15)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .bind(&body.kind)
    .bind(body.x)
    .bind(body.y)
    .bind(body.width)
    .bind(body.height)
    .bind(body.rotation)
    .bind(body.z_index)
    .bind(&body.content)
    .bind(body.locked)
    .bind(body.hidden)
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    if let Some(entity_ref) = &body.entity_ref {
        insert_entity_ref(
            &mut tx,
            actor.tenant_id,
            worktree_id,
            canvas_id,
            element_id,
            entity_ref,
            1,
            actor.user_id,
        )
        .await?;
    }
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        "canvas.element.created",
        correlation_id,
        1,
        json!({"kind": body.kind, "entity_ref": body.entity_ref}),
    )
    .await?;
    let response = json!({"element": load_element(&mut tx, actor.tenant_id, &scope, worktree_id, canvas_id, element_id, has_scope(&actor, "work-item:read")).await?});
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn update_element(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id, element_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<UpdateElementBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    let canvas_id = parse_id(&canvas_id)?;
    let element_id = parse_id(&element_id)?;
    validate_element_body(
        "shape",
        body.x,
        body.y,
        body.width,
        body.height,
        body.rotation,
        &body.content,
    )?;
    if body.expected_version < 1 {
        return Err(GroupApiError::invalid_request("invalid_expected_version"));
    }
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:element:update:{worktree_id}:{canvas_id}:{element_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    let current =
        lock_element(&mut tx, actor.tenant_id, worktree_id, canvas_id, element_id).await?;
    if current.version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    close_element_version(
        &mut tx,
        actor.tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        current.version,
    )
    .await?;
    let next_version = current.version + 1;
    sqlx::query(
        r#"
        INSERT INTO canvas.canvas_elements_backend
            (tenant_id,worktree_id,canvas_id,element_id,kind,x,y,width,height,rotation,z_index,
             content,locked,hidden,version,created_by)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .bind(&current.kind)
    .bind(body.x)
    .bind(body.y)
    .bind(body.width)
    .bind(body.height)
    .bind(body.rotation)
    .bind(body.z_index)
    .bind(&body.content)
    .bind(body.locked)
    .bind(body.hidden)
    .bind(next_version)
    .bind(actor.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        "canvas.element.updated",
        correlation_id,
        next_version,
        json!({"version": next_version}),
    )
    .await?;
    let response = json!({"element": load_element(&mut tx, actor.tenant_id, &scope, worktree_id, canvas_id, element_id, has_scope(&actor, "work-item:read")).await?});
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn set_entity_ref(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id, element_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<SetEntityRefBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    let canvas_id = parse_id(&canvas_id)?;
    let element_id = parse_id(&element_id)?;
    if body.expected_version < 0 || !matches!(body.ref_type.as_str(), "work_item" | "worktree") {
        return Err(GroupApiError::invalid_request("invalid_entity_ref"));
    }
    let entity_ref = EntityRefBody {
        ref_type: body.ref_type.clone(),
        ref_id: body.ref_id,
        worktree_id: body.worktree_id,
    };
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:entity-ref:set:{worktree_id}:{canvas_id}:{element_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    let element =
        lock_element(&mut tx, actor.tenant_id, worktree_id, canvas_id, element_id).await?;
    validate_required_ref(&element.kind, Some(entity_ref.ref_type.as_str()))?;
    validate_entity_ref(
        &mut tx,
        &actor,
        actor.tenant_id,
        &scope,
        worktree_id,
        &entity_ref,
    )
    .await?;
    let current = sqlx::query_as::<_, CurrentEntityRef>(
        r#"SELECT ref_type, ref_id, version FROM canvas.canvas_entity_ref
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4
             AND is_current AND valid_to IS NULL FOR UPDATE"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let previous_version = match &current {
        Some(row) => row.version,
        None => sqlx::query_scalar::<_, i64>(
            r#"SELECT COALESCE(MAX(version),0) FROM canvas.canvas_entity_ref
               WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4"#,
        )
        .bind(actor.tenant_id)
        .bind(worktree_id)
        .bind(canvas_id)
        .bind(element_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?,
    };
    if previous_version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    if current
        .as_ref()
        .is_some_and(|row| row.ref_type == body.ref_type && row.ref_id == body.ref_id)
    {
        let response = json!({"element": load_element(&mut tx, actor.tenant_id, &scope, worktree_id, canvas_id, element_id, has_scope(&actor, "work-item:read")).await?});
        save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    if current.is_some() {
        sqlx::query(
            r#"UPDATE canvas.canvas_entity_ref SET valid_to=clock_timestamp(), is_current=FALSE
               WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4 AND is_current"#,
        )
        .bind(actor.tenant_id).bind(worktree_id).bind(canvas_id).bind(element_id)
        .execute(&mut *tx).await.map_err(|_| GroupApiError::internal())?;
    }
    let next_ref_version = previous_version
        .checked_add(1)
        .ok_or_else(|| GroupApiError::conflict("version_exhausted"))?;
    let next_element_version = bump_element_version(
        &mut tx,
        actor.tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        &element,
        actor.user_id,
    )
    .await?;
    insert_entity_ref(
        &mut tx,
        actor.tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        &entity_ref,
        next_ref_version,
        actor.user_id,
    )
    .await?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        "canvas.entity_ref.set",
        correlation_id,
        next_element_version,
        json!({"ref_type": body.ref_type, "ref_id": body.ref_id,
               "entity_ref_version": next_ref_version}),
    )
    .await?;
    let response = json!({"element": load_element(&mut tx, actor.tenant_id, &scope, worktree_id, canvas_id, element_id, has_scope(&actor, "work-item:read")).await?});
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn clear_entity_ref(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id, element_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<ClearEntityRefBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    let canvas_id = parse_id(&canvas_id)?;
    let element_id = parse_id(&element_id)?;
    if body.expected_version < 1 {
        return Err(GroupApiError::invalid_request("invalid_expected_version"));
    }
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:entity-ref:clear:{worktree_id}:{canvas_id}:{element_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    let element =
        lock_element(&mut tx, actor.tenant_id, worktree_id, canvas_id, element_id).await?;
    validate_required_ref(&element.kind, None)?;
    let current = sqlx::query_as::<_, CurrentEntityRef>(
        r#"SELECT ref_type, ref_id, version FROM canvas.canvas_entity_ref
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4
             AND is_current AND valid_to IS NULL FOR UPDATE"#,
    )
    .bind(actor.tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if current.version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    let next_element_version = bump_element_version(
        &mut tx,
        actor.tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        &element,
        actor.user_id,
    )
    .await?;
    sqlx::query(
        r#"UPDATE canvas.canvas_entity_ref SET valid_to=clock_timestamp(), is_current=FALSE
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4 AND is_current"#,
    )
    .bind(actor.tenant_id).bind(worktree_id).bind(canvas_id).bind(element_id)
    .execute(&mut *tx).await.map_err(|_| GroupApiError::internal())?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        "canvas.entity_ref.cleared",
        correlation_id,
        next_element_version,
        json!({"previous_ref_type": current.ref_type, "previous_ref_id": current.ref_id,
               "entity_ref_version": current.version}),
    )
    .await?;
    let response = json!({"element": load_element(&mut tx, actor.tenant_id, &scope, worktree_id, canvas_id, element_id, has_scope(&actor, "work-item:read")).await?,
                           "entity_ref_version": current.version});
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

async fn delete_element(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, canvas_id, element_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<DeleteElementBody>,
) -> Result<Json<Value>, GroupApiError> {
    let worktree_id = request_worktree(&actor, &worktree_id, "canvas:write")?;
    let canvas_id = parse_id(&canvas_id)?;
    let element_id = parse_id(&element_id)?;
    if body.expected_version < 1 {
        return Err(GroupApiError::invalid_request("invalid_expected_version"));
    }
    let key = idempotency_key(&headers)?;
    let hash = request_hash(
        &format!("canvas:element:delete:{worktree_id}:{canvas_id}:{element_id}"),
        &body,
    )?;
    let (mut tx, scope) =
        authorize_group_scope(&state.resolver.pool, &actor, worktree_id, true).await?;
    require_canvas(&mut tx, actor.tenant_id, worktree_id, canvas_id).await?;
    if let Some(response) = lookup_idempotency(&mut tx, &actor, &key, &hash).await? {
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(Json(response));
    }
    let current =
        lock_element(&mut tx, actor.tenant_id, worktree_id, canvas_id, element_id).await?;
    if current.version != body.expected_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    sqlx::query(
        r#"UPDATE canvas.canvas_entity_ref SET valid_to=clock_timestamp(), is_current=FALSE
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4 AND is_current"#,
    )
    .bind(actor.tenant_id).bind(worktree_id).bind(canvas_id).bind(element_id)
    .execute(&mut *tx).await.map_err(|_| GroupApiError::internal())?;
    close_element_version(
        &mut tx,
        actor.tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        current.version,
    )
    .await?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    append_event(
        &mut tx,
        &actor,
        &scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        "canvas.element.removed",
        correlation_id,
        current.version + 1,
        json!({"previous_version": current.version}),
    )
    .await?;
    let response =
        json!({"element_id": element_id, "deleted": true, "version": current.version + 1});
    save_idempotency(&mut tx, &actor, &key, &hash, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(Json(response))
}

fn request_worktree(
    actor: &AuthUser,
    value: &str,
    canvas_scope: &str,
) -> Result<Uuid, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, "worktree:read")?;
    require_scope(actor, canvas_scope)?;
    parse_id(value)
}

async fn authorize_group_scope<'a>(
    pool: &'a PgPool,
    actor: &AuthUser,
    worktree_id: Uuid,
    write: bool,
) -> Result<(Transaction<'a, Postgres>, WorktreeScope), GroupApiError> {
    let mut tx = pool.begin().await.map_err(|_| GroupApiError::internal())?;
    let scope = authorize_worktree(&mut tx, actor, worktree_id).await?;
    let binding = active_binding(&mut tx, actor, scope.project_id).await?;
    if write {
        require_task_writer(&binding.role)?;
    }
    sqlx::query("SELECT set_config('app.worktree_id', $1, true)")
        .bind(worktree_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok((tx, scope))
}

async fn require_canvas(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
) -> Result<i64, GroupApiError> {
    sqlx::query_as::<_, CurrentCanvasVersion>(
        r#"SELECT version FROM canvas.group_canvas_registry
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND is_current AND valid_to IS NULL
           FOR SHARE"#,
    )
    .bind(tenant_id).bind(worktree_id).bind(canvas_id)
    .fetch_optional(&mut **tx).await.map_err(|_| GroupApiError::internal())?
    .map(|row| row.version).ok_or_else(GroupApiError::not_found)
}

async fn validate_entity_ref(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    tenant_id: Uuid,
    scope: &WorktreeScope,
    worktree_id: Uuid,
    entity_ref: &EntityRefBody,
) -> Result<(), GroupApiError> {
    if entity_ref.worktree_id != worktree_id {
        return Err(GroupApiError::invalid_request(
            "entity_ref_worktree_mismatch",
        ));
    }
    match entity_ref.ref_type.as_str() {
        "worktree" if entity_ref.ref_id == worktree_id => Ok(()),
        "worktree" => Err(GroupApiError::not_found()),
        "work_item" => {
            require_scope(actor, "work-item:read")?;
            let current = sqlx::query_scalar::<_, Uuid>(
                r#"SELECT m.work_item_id
                   FROM multica.task_metadata m
                   JOIN multica.work_item_worktree l
                     ON l.tenant_id=m.tenant_id AND l.project_id=m.project_id
                    AND l.work_item_id=m.work_item_id AND l.worktree_id=$3 AND l.valid_to IS NULL
                   JOIN multica.task_lifecycle_current c
                     ON c.tenant_id=m.tenant_id AND c.work_item_id=m.work_item_id
                   WHERE m.tenant_id=$1 AND m.project_id=$2 AND m.work_item_id=$4 AND m.valid_to IS NULL
                   FOR SHARE OF m, l"#,
            )
            .bind(tenant_id).bind(scope.project_id).bind(worktree_id).bind(entity_ref.ref_id)
            .fetch_optional(&mut **tx).await.map_err(|_| GroupApiError::internal())?;
            if current.is_some() {
                Ok(())
            } else {
                Err(GroupApiError::not_found())
            }
        }
        _ => Err(GroupApiError::invalid_request("invalid_entity_ref_type")),
    }
}

async fn insert_entity_ref(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Uuid,
    entity_ref: &EntityRefBody,
    version: i64,
    actor_id: Uuid,
) -> Result<(), GroupApiError> {
    sqlx::query(
        r#"INSERT INTO canvas.canvas_entity_ref
           (tenant_id,worktree_id,canvas_id,element_id,ref_type,ref_id,version,created_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .bind(&entity_ref.ref_type)
    .bind(entity_ref.ref_id)
    .bind(version)
    .bind(actor_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn append_event(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    scope: &WorktreeScope,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Option<Uuid>,
    event_type: &str,
    correlation_id: Uuid,
    aggregate_version: i64,
    details: Value,
) -> Result<(), GroupApiError> {
    let stream_lock_key = format!("{}:{worktree_id}", actor.tenant_id);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(stream_lock_key)
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"INSERT INTO canvas.canvas_group_audit
           (tenant_id,project_id,worktree_id,canvas_id,element_id,event_type,actor_id,correlation_id,details)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
    )
    .bind(actor.tenant_id).bind(scope.project_id).bind(worktree_id).bind(canvas_id).bind(element_id)
    .bind(event_type).bind(actor.user_id).bind(correlation_id).bind(&details)
    .execute(&mut **tx).await.map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"INSERT INTO canvas.canvas_group_outbox
           (tenant_id,project_id,worktree_id,canvas_id,element_id,event_type,aggregate_version,actor_id,correlation_id,payload)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"#,
    )
    .bind(actor.tenant_id).bind(scope.project_id).bind(worktree_id).bind(canvas_id).bind(element_id)
    .bind(event_type).bind(aggregate_version).bind(actor.user_id).bind(correlation_id)
    .bind(json!({"details": details, "correlation_id": correlation_id}))
    .execute(&mut **tx).await.map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn lock_element(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Uuid,
) -> Result<CurrentElement, GroupApiError> {
    sqlx::query_as::<_, CurrentElement>(
        r#"SELECT kind,version
           FROM canvas.canvas_elements_backend
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4
             AND is_current AND valid_to IS NULL FOR UPDATE"#,
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)
}

async fn close_element_version(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Uuid,
    version: i64,
) -> Result<(), GroupApiError> {
    let result = sqlx::query(
        r#"UPDATE canvas.canvas_elements_backend SET valid_to=clock_timestamp(),is_current=FALSE
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=$4
             AND version=$5 AND is_current AND valid_to IS NULL"#,
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    if result.rows_affected() != 1 {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    Ok(())
}

async fn bump_element_version(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Uuid,
    current: &CurrentElement,
    actor_id: Uuid,
) -> Result<i64, GroupApiError> {
    let next_version = current
        .version
        .checked_add(1)
        .ok_or_else(|| GroupApiError::conflict("version_exhausted"))?;
    close_element_version(
        tx,
        tenant_id,
        worktree_id,
        canvas_id,
        element_id,
        current.version,
    )
    .await?;
    sqlx::query(
        r#"INSERT INTO canvas.canvas_elements_backend
           (tenant_id,worktree_id,canvas_id,element_id,kind,x,y,width,height,rotation,z_index,
            content,locked,hidden,version,created_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)"#,
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(element_id)
    .bind(&current.kind)
    .bind(current.x)
    .bind(current.y)
    .bind(current.width)
    .bind(current.height)
    .bind(current.rotation)
    .bind(current.z_index)
    .bind(&current.content)
    .bind(current.locked)
    .bind(current.hidden)
    .bind(next_version)
    .bind(actor_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(next_version)
}

async fn query_elements(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    scope: &WorktreeScope,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Option<Uuid>,
    include_work_item_refs: bool,
) -> Result<Vec<CanvasElementRow>, GroupApiError> {
    let rows = sqlx::query_as::<_, CanvasElementRow>(ELEMENT_SELECT_SQL)
        .bind(tenant_id)
        .bind(worktree_id)
        .bind(scope.project_id)
        .bind(canvas_id)
        .bind(element_id)
        .bind(include_work_item_refs)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(rows)
}

async fn load_element(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    scope: &WorktreeScope,
    worktree_id: Uuid,
    canvas_id: Uuid,
    element_id: Uuid,
    include_work_item_refs: bool,
) -> Result<Value, GroupApiError> {
    query_elements(
        tx,
        tenant_id,
        scope,
        worktree_id,
        canvas_id,
        Some(element_id),
        include_work_item_refs,
    )
    .await?
    .into_iter()
    .next()
    .map(element_projection)
    .ok_or_else(GroupApiError::not_found)
}

const ELEMENT_SELECT_SQL: &str = r#"
    SELECT e.worktree_id,e.element_id,e.kind,e.x,e.y,e.width,e.height,e.rotation,e.z_index,e.content,
           e.locked,e.hidden,e.version,e.created_by,e.created_at,e.valid_from AS updated_at,
           r.ref_type AS entity_ref_type,r.ref_id AS entity_ref_id,
           r.version AS entity_ref_version
    FROM canvas.canvas_elements_backend e
    LEFT JOIN canvas.canvas_entity_ref r
      ON r.tenant_id=e.tenant_id AND r.worktree_id=e.worktree_id AND r.canvas_id=e.canvas_id
     AND r.element_id=e.element_id AND r.is_current AND r.valid_to IS NULL
     AND ($6::BOOLEAN OR r.ref_type <> 'work_item')
     AND ((r.ref_type='worktree' AND r.ref_id=$2) OR
          (r.ref_type='work_item' AND EXISTS (
             SELECT 1 FROM multica.task_metadata em
             JOIN multica.work_item_worktree el ON el.tenant_id=em.tenant_id
                AND el.project_id=em.project_id AND el.work_item_id=em.work_item_id
                AND el.worktree_id=$2 AND el.valid_to IS NULL
             JOIN multica.task_lifecycle_current ec ON ec.tenant_id=em.tenant_id
                AND ec.work_item_id=em.work_item_id
             WHERE em.tenant_id=e.tenant_id AND em.project_id=$3
               AND em.work_item_id=r.ref_id AND em.valid_to IS NULL)))
    WHERE e.tenant_id=$1 AND e.worktree_id=$2 AND e.canvas_id=$4
      AND e.is_current AND e.valid_to IS NULL
      AND ($5::UUID IS NULL OR e.element_id=$5)
    ORDER BY e.z_index,e.element_id
"#;

fn element_projection(row: CanvasElementRow) -> Value {
    let entity_ref =
        row.entity_ref_type
            .as_ref()
            .zip(row.entity_ref_id)
            .map(|(ref_type, ref_id)| {
                json!({"ref_type": ref_type, "ref_id": ref_id, "worktree_id": row.worktree_id,
               "version": row.entity_ref_version})
            });
    json!({"element_id":row.element_id,"kind":row.kind,"x":row.x,"y":row.y,
           "width":row.width,"height":row.height,"rotation":row.rotation,"z_index":row.z_index,
           "content":row.content,"locked":row.locked,"hidden":row.hidden,"version":row.version,
           "created_by":row.created_by,"created_at":row.created_at,"updated_at":row.updated_at,
           "entity_ref":entity_ref})
}

fn has_scope(actor: &AuthUser, scope: &str) -> bool {
    actor
        .scope
        .split_ascii_whitespace()
        .any(|granted| granted == scope)
}

fn validate_element_body(
    kind: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    content: &Value,
) -> Result<(), GroupApiError> {
    const KINDS: &[&str] = &[
        "sticky_note",
        "text",
        "shape",
        "image",
        "embed",
        "work_item_card",
        "worktree_node",
        "agent_cursor",
        "automation_node",
        "comment_pin",
    ];
    if !KINDS.contains(&kind)
        || !x.is_finite()
        || !y.is_finite()
        || x.abs() > 1_000_000.0
        || y.abs() > 1_000_000.0
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || width > 10_000.0
        || height <= 0.0
        || height > 10_000.0
        || !rotation.is_finite()
        || rotation.abs() > 36_000.0
        || !content.is_object()
        || serde_json::to_vec(content)
            .map(|bytes| bytes.len() > 65_536)
            .unwrap_or(true)
        || identity_key_present(content)
    {
        return Err(GroupApiError::invalid_request("invalid_canvas_element"));
    }
    Ok(())
}

fn validate_required_ref(kind: &str, ref_type: Option<&str>) -> Result<(), GroupApiError> {
    let required_type = match kind {
        "work_item_card" => Some("work_item"),
        "worktree_node" => Some("worktree"),
        _ => None,
    };
    if required_type.is_some_and(|required| ref_type != Some(required)) {
        return Err(GroupApiError::invalid_request(
            "canvas_element_entity_ref_required",
        ));
    }
    if ref_type.is_some_and(|value| !matches!(value, "work_item" | "worktree")) {
        return Err(GroupApiError::invalid_request("invalid_entity_ref_type"));
    }
    Ok(())
}

fn validate_canvas_document(body: &UpdateCanvasDocumentBody) -> Result<(), GroupApiError> {
    if body.expected_version <= 0
        || !body.viewport.x.is_finite()
        || !body.viewport.y.is_finite()
        || body.viewport.x.abs() > 1_000_000.0
        || body.viewport.y.abs() > 1_000_000.0
        || !body.viewport.zoom.is_finite()
        || !(0.1..=4.0).contains(&body.viewport.zoom)
        || body.frames.len() > 500
        || body.connectors.len() > 2_000
        || serde_json::to_vec(body)
            .map(|bytes| bytes.len() > 1_000_000)
            .unwrap_or(true)
    {
        return Err(GroupApiError::invalid_request("invalid_canvas_document"));
    }

    let mut frame_ids = HashSet::new();
    for frame in &body.frames {
        let element_ids = frame.element_ids.iter().collect::<HashSet<_>>();
        if !frame_ids.insert(frame.id)
            || frame.title.trim().is_empty()
            || frame.title.trim().len() > 200
            || !frame.x.is_finite()
            || !frame.y.is_finite()
            || frame.x.abs() > 1_000_000.0
            || frame.y.abs() > 1_000_000.0
            || !frame.width.is_finite()
            || !frame.height.is_finite()
            || frame.width <= 0.0
            || frame.height <= 0.0
            || frame.width > 100_000.0
            || frame.height > 100_000.0
            || frame.element_ids.len() > 500
            || element_ids.len() != frame.element_ids.len()
        {
            return Err(GroupApiError::invalid_request("invalid_canvas_frame"));
        }
    }

    let mut connector_ids = HashSet::new();
    for connector in &body.connectors {
        if !connector_ids.insert(connector.id)
            || connector.from_element_id == connector.to_element_id
            || !matches!(
                connector.kind.as_str(),
                "work_item_relation" | "agent_handoff" | "free" | "dependency"
            )
            || !matches!(
                connector.routing.as_str(),
                "straight" | "curved" | "orthogonal"
            )
            || !is_safe_hex_color(&connector.color)
            || !connector.width.is_finite()
            || !(0.5..=16.0).contains(&connector.width)
            || connector
                .label
                .as_ref()
                .is_some_and(|label| label.len() > 200)
        {
            return Err(GroupApiError::invalid_request("invalid_canvas_connector"));
        }
    }
    Ok(())
}

async fn validate_canvas_document_references(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    canvas_id: Uuid,
    body: &UpdateCanvasDocumentBody,
) -> Result<(), GroupApiError> {
    let mut element_ids = HashSet::new();
    for frame in &body.frames {
        element_ids.extend(frame.element_ids.iter().copied());
    }
    for connector in &body.connectors {
        element_ids.insert(connector.from_element_id);
        element_ids.insert(connector.to_element_id);
    }
    if element_ids.is_empty() {
        return Ok(());
    }

    let expected_count = element_ids.len();
    let element_ids = element_ids.into_iter().collect::<Vec<_>>();
    let found_ids = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT element_id FROM canvas.canvas_elements_backend
           WHERE tenant_id=$1 AND worktree_id=$2 AND canvas_id=$3 AND element_id=ANY($4)
             AND is_current AND valid_to IS NULL FOR SHARE"#,
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(canvas_id)
    .bind(&element_ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    if found_ids.len() != expected_count {
        return Err(GroupApiError::invalid_request(
            "canvas_document_element_reference_invalid",
        ));
    }
    Ok(())
}

fn is_safe_hex_color(color: &str) -> bool {
    let Some(hex) = color.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 6) && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn identity_key_present(value: &Value) -> bool {
    match value {
        Value::Object(object) => object.iter().any(|(key, child)| {
            matches!(
                key.as_str(),
                "work_item_id"
                    | "task_card_id"
                    | "worktree_id"
                    | "entity_ref"
                    | "entity_refs"
                    | "ref_id"
                    | "ref_type"
            ) || identity_key_present(child)
        }),
        Value::Array(items) => items.iter().any(identity_key_present),
        _ => false,
    }
}

fn empty_object() -> Value {
    json!({})
}
