//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"scoped_chat.rs",type:"file",language:"rust"}),(m:Module {name:"scoped_chat",type:"module",language:"rust"}),
//!   (scope:Class {name:"ScopedChatScope",type:"class",language:"rust"}),(entity:Class {name:"ScopedChatEntityRef",type:"class",language:"rust"}),(target:Class {name:"AuthorizedChatTarget",type:"class",language:"rust"}),(command:Class {name:"ScopedChatCommand",type:"class",language:"rust"}),(receipt:Class {name:"ScopedChatReceipt",type:"class",language:"rust"}),(error:Enum {name:"ScopedChatWorkflowError",type:"enum",language:"rust"}),(workflow:Interface {name:"ScopedChatWorkflow",type:"interface",language:"rust"}),(body:Class {name:"ScopedChatMessageBody",type:"class",language:"rust"}),(envelope:Class {name:"GroupContextEnvelope",type:"class",language:"rust"}),(projection:Class {name:"GroupContextProjection",type:"class",language:"rust"}),
//!   (maxTargets:Variable {name:"MAX_GLOBAL_TARGETS",type:"variable",language:"rust"}),(maxRefs:Variable {name:"MAX_ENTITY_REFS",type:"variable",language:"rust"}),(maxBytes:Variable {name:"MAX_MESSAGE_BYTES",type:"variable",language:"rust"}),
//!   (router:Function {name:"router",type:"function",language:"rust"}),(submit:Function {name:"submit_scoped_chat_message",type:"function",language:"rust"}),(validate:Function {name:"validate_body",type:"function",language:"rust"}),(fingerprint:Function {name:"request_fingerprint",type:"function",language:"rust"}),(map_error:Function {name:"map_workflow_error",type:"function",language:"rust"}),(target_context:Function {name:"resolve_authorized_target",type:"function",language:"rust"}),(idempotency:Function {name:"read_idempotency_key",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(scope),(m)-[:CONTAINS]->(entity),(m)-[:CONTAINS]->(target),(m)-[:CONTAINS]->(command),(m)-[:CONTAINS]->(receipt),(m)-[:CONTAINS]->(error),(m)-[:CONTAINS]->(workflow),(m)-[:CONTAINS]->(body),(m)-[:CONTAINS]->(envelope),(m)-[:CONTAINS]->(projection),(m)-[:CONTAINS]->(maxTargets),(m)-[:CONTAINS]->(maxRefs),(m)-[:CONTAINS]->(maxBytes),(m)-[:CONTAINS]->(router),(m)-[:CONTAINS]->(submit),(m)-[:CONTAINS]->(validate),(m)-[:CONTAINS]->(fingerprint),(m)-[:CONTAINS]->(map_error),(m)-[:CONTAINS]->(target_context),(m)-[:CONTAINS]->(idempotency),
//!   (workflow)-[:HAS_METHOD]->(wf_submit:Function {name:"ScopedChatWorkflow::submit",type:"function",language:"rust"}),(m)-[:CONTAINS]->(wf_submit),
//!   (router)-[:CALLS]->(submit),(submit)-[:CALLS]->(validate),(submit)-[:CALLS]->(fingerprint),(submit)-[:CALLS]->(map_error),(submit)-[:CALLS]->(target_context),(submit)-[:CALLS]->(idempotency),(submit)-[:CALLS]->(wf_submit),(submit)-[:USES]->(maxTargets),(validate)-[:USES]->(maxTargets),(validate)-[:USES]->(maxRefs),(validate)-[:USES]->(maxBytes),(target_context)-[:USES]->(envelope),(envelope)-[:USES]->(projection);
//! MATCH (submit:Function {name:"submit_scoped_chat_message"}),(validate:Function {name:"validate_body"}),(fingerprint:Function {name:"request_fingerprint"}),(map:Function {name:"map_workflow_error"}),(target:Function {name:"resolve_authorized_target"}),(key:Function {name:"read_idempotency_key"}),(actor:Function {name:"validate_actor"}),(scope:Function {name:"require_scope"}),(bad:Function {name:"GroupApiError::bad_request"}),(invalid:Function {name:"GroupApiError::invalid_request"}),(internal:Function {name:"GroupApiError::internal"}),(notFound:Function {name:"GroupApiError::not_found"}),(conflict:Function {name:"GroupApiError::conflict"}),(unavailable:Function {name:"GroupApiError::feature_unavailable"}),(resolver:Function {name:"GroupContextResolver::resolve_worktree_context"});
//! CREATE (submit)-[:CALLS]->(actor),(submit)-[:CALLS]->(scope),(submit)-[:CALLS]->(bad),(validate)-[:CALLS]->(invalid),(target)-[:CALLS]->(resolver),(target)-[:CALLS]->(internal),(fingerprint)-[:CALLS]->(internal),(key)-[:CALLS]->(invalid),(map)-[:CALLS]->(conflict),(map)-[:CALLS]->(unavailable),(map)-[:CALLS]->(internal);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"scoped_chat"}),(router:Function {name:"router"}),(submit:Function {name:"submit_scoped_chat_message"}),(validateActor:Function {name:"validate_actor"}),(requireScope:Function {name:"require_scope"}),(setTenant:Function {name:"set_tenant"}),(internal:Function {name:"GroupApiError::internal"}),(badRequest:Function {name:"GroupApiError::bad_request"}),(notFound:Function {name:"GroupApiError::not_found"}),(resolveTarget:Function {name:"resolve_authorized_target"}),(activeBinding:Function {name:"active_binding"});
//! CREATE (query:Class {name:"ChatTargetQuery",type:"class",language:"rust"}),(target:Class {name:"ScopedChatTarget",type:"class",language:"rust"}),(targetPage:Class {name:"ScopedChatTargetPage",type:"class",language:"rust"}),(targetRow:Class {name:"ChatTargetRow",type:"class",language:"rust"}),(listTargets:Function {name:"list_global_chat_targets",type:"function",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(query),(m)-[:CONTAINS]->(target),(m)-[:CONTAINS]->(targetPage),(m)-[:CONTAINS]->(targetRow),(m)-[:CONTAINS]->(listTargets),(router)-[:CALLS]->(listTargets),(listTargets)-[:CALLS]->(validateActor),(listTargets)-[:CALLS]->(requireScope),(listTargets)-[:CALLS]->(activeBinding),(listTargets)-[:CALLS]->(setTenant),(listTargets)-[:CALLS]->(internal),(listTargets)-[:CALLS]->(badRequest),(listTargets)-[:CALLS]->(notFound),(listTargets)-[:USES]->(query),(listTargets)-[:USES]->(targetPage),(listTargets)-[:USES]->(targetRow);

use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, AuthenticatedUser, GroupApiError,
    GroupApiState,
};

const MAX_GLOBAL_TARGETS: usize = 20;
const MAX_TARGET_PAGE_SIZE: i64 = 100;
const MAX_ENTITY_REFS: usize = 100;
const MAX_MESSAGE_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScopedChatScope {
    Worktree,
    Global,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedChatEntityRef {
    pub ref_type: String,
    pub ref_id: Uuid,
    pub worktree_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct AuthorizedChatTarget {
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub context_version: i64,
    pub permission_snapshot_ref: String,
}

#[derive(Clone, Debug)]
pub struct ScopedChatCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub current_worktree_id: Uuid,
    pub scope: ScopedChatScope,
    pub target_worktree_ids: Vec<Uuid>,
    pub authorized_targets: Vec<AuthorizedChatTarget>,
    pub session_id: Option<Uuid>,
    pub message: String,
    pub entity_refs: Vec<ScopedChatEntityRef>,
    pub correlation_id: Uuid,
    pub idempotency_key: Uuid,
    pub request_fingerprint: [u8; 32],
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScopedChatReceipt {
    pub session_id: Uuid,
    pub run_id: Uuid,
    pub scope: ScopedChatScope,
    pub target_worktree_ids: Vec<Uuid>,
    pub correlation_id: Uuid,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
struct ScopedChatTarget {
    worktree_id: Uuid,
    project_id: Uuid,
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct ScopedChatTargetPage {
    targets: Vec<ScopedChatTarget>,
    limit: i64,
    next_cursor: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum ScopedChatWorkflowError {
    Conflict,
    Unavailable,
    Internal,
}

/// Production implementations persist transcript/run intent before dispatch and reauthorize
/// each checkpoint resume and capability call. This adapter must not treat the supplied
/// `authorized_targets` as durable grants.
#[async_trait]
pub trait ScopedChatWorkflow: Send + Sync {
    async fn submit(
        &self,
        command: ScopedChatCommand,
    ) -> Result<ScopedChatReceipt, ScopedChatWorkflowError>;
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopedChatMessageBody {
    scope: ScopedChatScope,
    #[serde(default)]
    target_worktree_ids: Vec<Uuid>,
    #[serde(default)]
    session_id: Option<Uuid>,
    message: String,
    #[serde(default)]
    entity_refs: Vec<ScopedChatEntityRef>,
    #[serde(default)]
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatTargetQuery {
    limit: Option<i64>,
    cursor: Option<String>,
}

#[derive(Debug, FromRow)]
struct ChatTargetRow {
    worktree_id: Uuid,
    project_id: Uuid,
    name: String,
}

#[derive(Deserialize)]
struct GroupContextEnvelope {
    group_context: GroupContextProjection,
}

#[derive(Deserialize)]
struct GroupContextProjection {
    project_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    context_version: i64,
    permission_snapshot_ref: String,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/worktrees/{worktree_id}/chat/targets",
            get(list_global_chat_targets),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/chat/messages",
            post(submit_scoped_chat_message),
        )
}

async fn list_global_chat_targets(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    Query(query): Query<ChatTargetQuery>,
) -> Result<Json<ScopedChatTargetPage>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "chat:submit")?;
    let current_worktree_id =
        Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let limit = query.limit.unwrap_or(50).clamp(1, MAX_TARGET_PAGE_SIZE);
    let cursor = query
        .cursor
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| GroupApiError::invalid_request("invalid_chat_target_cursor"))?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let current_project_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT p.project_id
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        FOR SHARE OF w, p
        "#,
    )
    .bind(current_worktree_id)
    .bind(actor.tenant_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    active_binding(&mut tx, &actor, current_project_id).await?;

    let mut rows = sqlx::query_as::<_, ChatTargetRow>(
        r#"
        SELECT w.id AS worktree_id, p.project_id, w.name
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.tenant_id = $1
          AND w.archived = FALSE
          AND ($2::UUID IS NULL OR w.id > $2)
          AND EXISTS (
              SELECT 1
              FROM permission.project_role_binding membership
              WHERE membership.tenant_id = w.tenant_id
                AND membership.project_id = p.project_id
                AND membership.user_id = $3
                AND membership.valid_to IS NULL
          )
        ORDER BY w.id ASC
        LIMIT $4
        "#,
    )
    .bind(actor.tenant_id)
    .bind(cursor)
    .bind(actor.user_id)
    .bind(limit + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let next_cursor = has_more
        .then(|| rows.last().map(|row| row.worktree_id.to_string()))
        .flatten();
    let targets = rows
        .into_iter()
        .map(|row| ScopedChatTarget {
            worktree_id: row.worktree_id,
            project_id: row.project_id,
            name: row.name,
        })
        .collect();
    Ok(Json(ScopedChatTargetPage {
        targets,
        limit,
        next_cursor,
    }))
}

async fn submit_scoped_chat_message(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<ScopedChatMessageBody>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "chat:submit")?;
    validate_body(&body)?;

    let current_worktree_id =
        Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let idempotency_key = read_idempotency_key(&headers)?;
    let mut target_worktree_ids = match body.scope {
        ScopedChatScope::Worktree => {
            if !body.target_worktree_ids.is_empty()
                && body.target_worktree_ids != [current_worktree_id]
            {
                return Err(GroupApiError::invalid_request("chat_scope_target_mismatch"));
            }
            vec![current_worktree_id]
        }
        ScopedChatScope::Global => body.target_worktree_ids.clone(),
    };
    target_worktree_ids.sort_unstable();

    let current_target = resolve_authorized_target(&state, &actor, current_worktree_id).await?;
    let mut authorized_targets = Vec::with_capacity(target_worktree_ids.len());
    for target_worktree_id in &target_worktree_ids {
        if *target_worktree_id == current_worktree_id {
            authorized_targets.push(current_target.clone());
        } else {
            authorized_targets
                .push(resolve_authorized_target(&state, &actor, *target_worktree_id).await?);
        }
    }
    if body
        .entity_refs
        .iter()
        .any(|entity_ref| !target_worktree_ids.contains(&entity_ref.worktree_id))
    {
        return Err(GroupApiError::invalid_request("chat_entity_scope_mismatch"));
    }

    // Keep retries stable when clients omit correlation_id: the idempotency key is
    // already a unique request identity and remains the correlation fallback.
    let correlation_id = body.correlation_id.unwrap_or(idempotency_key);
    let fingerprint = request_fingerprint(
        actor.tenant_id,
        actor.user_id,
        current_worktree_id,
        body.scope,
        &target_worktree_ids,
        body.session_id,
        &body.message,
        &body.entity_refs,
        correlation_id,
    )?;
    let Some(workflow) = state.scoped_chat_workflow.clone() else {
        return Err(GroupApiError::feature_unavailable(
            "scoped_chat_workflow_unavailable",
        ));
    };

    let receipt = workflow
        .submit(ScopedChatCommand {
            tenant_id: actor.tenant_id,
            actor_id: actor.user_id,
            current_worktree_id,
            scope: body.scope,
            target_worktree_ids,
            authorized_targets,
            session_id: body.session_id,
            message: body.message,
            entity_refs: body.entity_refs,
            correlation_id,
            idempotency_key,
            request_fingerprint: fingerprint,
        })
        .await
        .map_err(map_workflow_error)?;

    Ok((StatusCode::ACCEPTED, Json(receipt)).into_response())
}

fn validate_body(body: &ScopedChatMessageBody) -> Result<(), GroupApiError> {
    let target_count = body.target_worktree_ids.len();
    if body.message.trim().is_empty() || body.message.len() > MAX_MESSAGE_BYTES {
        return Err(GroupApiError::invalid_request("invalid_chat_message"));
    }
    if body.entity_refs.len() > MAX_ENTITY_REFS {
        return Err(GroupApiError::invalid_request("too_many_chat_entity_refs"));
    }
    if body.scope == ScopedChatScope::Global
        && (target_count == 0 || target_count > MAX_GLOBAL_TARGETS)
    {
        return Err(GroupApiError::invalid_request(
            "invalid_global_chat_targets",
        ));
    }
    let mut unique_targets = body.target_worktree_ids.clone();
    unique_targets.sort_unstable();
    unique_targets.dedup();
    if unique_targets.len() != target_count {
        return Err(GroupApiError::invalid_request("duplicate_chat_target"));
    }
    let mut unique_refs = body
        .entity_refs
        .iter()
        .map(|entity_ref| {
            (
                entity_ref.ref_type.as_str(),
                entity_ref.ref_id,
                entity_ref.worktree_id,
            )
        })
        .collect::<Vec<_>>();
    unique_refs.sort_unstable();
    unique_refs.dedup();
    if unique_refs.len() != body.entity_refs.len()
        || body.entity_refs.iter().any(|entity_ref| {
            entity_ref.ref_type.trim().is_empty() || entity_ref.ref_type.len() > 100
        })
    {
        return Err(GroupApiError::invalid_request("invalid_chat_entity_refs"));
    }
    Ok(())
}

async fn resolve_authorized_target(
    state: &GroupApiState,
    actor: &crate::auth::AuthUser,
    worktree_id: Uuid,
) -> Result<AuthorizedChatTarget, GroupApiError> {
    let context = state
        .resolver
        .resolve_worktree_context(actor, worktree_id)
        .await?;
    let envelope: GroupContextEnvelope =
        serde_json::from_value(context).map_err(|_| GroupApiError::internal())?;
    let group_context = envelope.group_context;
    Ok(AuthorizedChatTarget {
        project_id: group_context.project_id,
        repository_id: group_context.repository_id,
        worktree_id: group_context.worktree_id,
        context_version: group_context.context_version,
        permission_snapshot_ref: group_context.permission_snapshot_ref,
    })
}

fn request_fingerprint(
    tenant_id: Uuid,
    actor_id: Uuid,
    current_worktree_id: Uuid,
    scope: ScopedChatScope,
    target_worktree_ids: &[Uuid],
    session_id: Option<Uuid>,
    message: &str,
    entity_refs: &[ScopedChatEntityRef],
    correlation_id: Uuid,
) -> Result<[u8; 32], GroupApiError> {
    let canonical = serde_json::to_vec(&(
        tenant_id,
        actor_id,
        current_worktree_id,
        scope,
        target_worktree_ids,
        session_id,
        message,
        entity_refs,
        correlation_id,
    ))
    .map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(canonical).into())
}

fn read_idempotency_key(headers: &HeaderMap) -> Result<Uuid, GroupApiError> {
    headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| GroupApiError::invalid_request("idempotency_key_required"))
}

fn map_workflow_error(error: ScopedChatWorkflowError) -> GroupApiError {
    match error {
        ScopedChatWorkflowError::Conflict => GroupApiError::conflict("chat_request_conflict"),
        ScopedChatWorkflowError::Unavailable => {
            GroupApiError::feature_unavailable("scoped_chat_workflow_unavailable")
        }
        ScopedChatWorkflowError::Internal => GroupApiError::internal(),
    }
}
