//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"cli_sessions.rs",type:"file",language:"rust"}),(m:Module {name:"cli_sessions",type:"module",language:"rust"}),
//!   (p:Interface {name:"TaskCliSessionProvisioner",type:"interface",language:"rust"}),(e:Enum {name:"TaskCliSessionProvisionError",type:"enum",language:"rust"}),(c:Class {name:"TaskCliSessionStartCommand",type:"class",language:"rust"}),(a:Class {name:"TaskCliSessionAccessCommand",type:"class",language:"rust"}),(r:Class {name:"TaskCliSessionReceipt",type:"class",language:"rust"}),(ss:Class {name:"TaskCliSessionStatus",type:"class",language:"rust"}),(state:Enum {name:"TaskCliSessionState",type:"enum",language:"rust"}),(b:Class {name:"StartTaskCliSessionBody",type:"class",language:"rust"}),(w:Class {name:"WorktreeTaskCliScope",type:"class",language:"rust"}),(l:Class {name:"TaskCliLifecycle",type:"class",language:"rust"}),(req:Variable {name:"request_fingerprint",type:"variable",language:"rust"}),
//!   (rt:Function {name:"router",type:"function",language:"rust"}),(st:Function {name:"start_task_cli_session",type:"function",language:"rust"}),(list:Function {name:"list_task_cli_sessions",type:"function",language:"rust"}),(get:Function {name:"get_task_cli_session_status",type:"function",language:"rust"}),(cancel:Function {name:"cancel_task_cli_session",type:"function",language:"rust"}),(attach:Function {name:"reattach_task_cli_session",type:"function",language:"rust"}),(authorize:Function {name:"authorize_task_cli_session_access",type:"function",language:"rust"}),(authorize_parent:Function {name:"authorize_task_cli_session_parent",type:"function",language:"rust"}),(valid:Function {name:"validate_receipt",type:"function",language:"rust"}),(no_store:Function {name:"no_store",type:"function",language:"rust"}),(val:Function {name:"validate_start_body",type:"function",language:"rust"}),(limit:Function {name:"validate_session_list_limit",type:"function",language:"rust"}),(valid_list:Function {name:"session_statuses_are_valid",type:"function",language:"rust"}),(fp:Function {name:"request_fingerprint",type:"function",language:"rust"}),(map:Function {name:"map_provision_error",type:"function",language:"rust"}),(sp:Function {name:"TaskCliSessionProvisioner::start_task_cli_session",type:"function",language:"rust"}),(list_method:Function {name:"TaskCliSessionProvisioner::list_task_cli_sessions",type:"function",language:"rust"}),(status:Function {name:"TaskCliSessionProvisioner::task_cli_session_status",type:"function",language:"rust"}),(stop:Function {name:"TaskCliSessionProvisioner::cancel_task_cli_session",type:"function",language:"rust"}),(ticket:Function {name:"TaskCliSessionProvisioner::reattach_task_cli_session",type:"function",language:"rust"}),(va:Function {name:"validate_actor",type:"function",language:"rust"}),(rs:Function {name:"require_scope",type:"function",language:"rust"}),(ik:Function {name:"idempotency_key",type:"function",language:"rust"}),(tenant:Function {name:"set_tenant",type:"function",language:"rust"}),(ab:Function {name:"active_binding",type:"function",language:"rust"}),(writer:Function {name:"require_task_writer",type:"function",language:"rust"}),(bad:Function {name:"GroupApiError::bad_request",type:"function",language:"rust"}),(internal:Function {name:"GroupApiError::internal",type:"function",language:"rust"}),(not_found:Function {name:"GroupApiError::not_found",type:"function",language:"rust"}),(conflict:Function {name:"GroupApiError::conflict",type:"function",language:"rust"}),(unavailable:Function {name:"GroupApiError::service_unavailable",type:"function",language:"rust"}),(invalid:Function {name:"GroupApiError::invalid_request",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(p),(m)-[:CONTAINS]->(e),(m)-[:CONTAINS]->(c),(m)-[:CONTAINS]->(a),(m)-[:CONTAINS]->(r),(m)-[:CONTAINS]->(ss),(m)-[:CONTAINS]->(state),(m)-[:CONTAINS]->(b),(m)-[:CONTAINS]->(w),(m)-[:CONTAINS]->(l),(m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(st),(m)-[:CONTAINS]->(get),(m)-[:CONTAINS]->(cancel),(m)-[:CONTAINS]->(attach),(m)-[:CONTAINS]->(authorize),(m)-[:CONTAINS]->(valid),(m)-[:CONTAINS]->(no_store),(m)-[:CONTAINS]->(val),(m)-[:CONTAINS]->(fp),(m)-[:CONTAINS]->(map),(p)-[:HAS_METHOD]->(sp),(p)-[:HAS_METHOD]->(status),(p)-[:HAS_METHOD]->(stop),(p)-[:HAS_METHOD]->(ticket),
//!   (m)-[:CONTAINS]->(list),(m)-[:CONTAINS]->(limit),(m)-[:CONTAINS]->(valid_list),(m)-[:CONTAINS]->(authorize_parent),(p)-[:HAS_METHOD]->(list_method),(list)-[:CALLS]->(authorize_parent),(list)-[:CALLS]->(list_method),(list)-[:CALLS]->(limit),(list)-[:CALLS]->(valid_list),(list)-[:CALLS]->(no_store),
//!   (rt)-[:CALLS]->(st),(rt)-[:CALLS]->(list),(rt)-[:CALLS]->(get),(rt)-[:CALLS]->(cancel),(rt)-[:CALLS]->(attach),(st)-[:CALLS]->(val),(st)-[:CALLS]->(fp),(st)-[:CALLS]->(map),(st)-[:CALLS]->(va),(st)-[:CALLS]->(rs),(st)-[:CALLS]->(ik),(st)-[:CALLS]->(tenant),(st)-[:CALLS]->(ab),(st)-[:CALLS]->(writer),(st)-[:CALLS]->(sp),(st)-[:CALLS]->(bad),(st)-[:CALLS]->(internal),(st)-[:CALLS]->(not_found),(st)-[:CALLS]->(conflict),(st)-[:CALLS]->(unavailable),(st)-[:CALLS]->(invalid),(st)-[:USES]->(req),(list)-[:CALLS]->(authorize_parent),(list)-[:CALLS]->(list_method),(list)-[:CALLS]->(no_store),(get)-[:CALLS]->(authorize),(get)-[:CALLS]->(status),(cancel)-[:CALLS]->(authorize),(cancel)-[:CALLS]->(stop),(attach)-[:CALLS]->(authorize),(attach)-[:CALLS]->(ticket),(attach)-[:CALLS]->(valid),(attach)-[:CALLS]->(no_store),(get)-[:CALLS]->(no_store),(cancel)-[:CALLS]->(no_store),(authorize)-[:CALLS]->(authorize_parent),(val)-[:CALLS]->(invalid),(fp)-[:CALLS]->(internal),(map)-[:CALLS]->(unavailable),(map)-[:CALLS]->(conflict),(map)-[:CALLS]->(not_found),(map)-[:CALLS]->(internal);
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(s:Function {name:"GroupApiState::with_task_cli_session_provisioner",type:"function"}),(p:Interface {name:"TaskCliSessionProvisioner",type:"interface"}),(g:Class {name:"GroupApiState",type:"class"});
//! CREATE (m)-[:USES]->(g),(st)-[:USES]->(p),(g)-[:USES]->(p);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(list:Function {name:"list_task_cli_sessions",type:"function"}),(limit:Function {name:"validate_session_list_limit",type:"function"}),(valid:Function {name:"session_statuses_are_valid",type:"function"});
//! CREATE (list_command:Class {name:"TaskCliSessionListCommand",type:"class",language:"rust"}),(list_query:Class {name:"TaskCliSessionListQuery",type:"class",language:"rust"}),(parent:Class {name:"TaskCliSessionParent",type:"class",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(list_command),(m)-[:CONTAINS]->(list_query),(m)-[:CONTAINS]->(parent),(list)-[:CALLS]->(limit),(list)-[:CALLS]->(valid);

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

use super::{
    AuthUser, AuthenticatedUser, GroupApiError, GroupApiState, active_binding, require_scope,
    set_tenant, validate_actor,
};

/// A REST-authorized request for provisioning a Task Card CLI session in Local Runtime.
/// The provisioner must resolve the profile from its trusted catalog and recheck live ACL,
/// Worktree/runtime health and policy immediately before grant issuance and spawn; persist
/// TaskRun intent/result audit against the supplied correlation ID. This command is not itself
/// an execution grant.
#[derive(Clone)]
pub struct TaskCliSessionStartCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub expected_lifecycle_version: i32,
    pub approved_launch_profile_id: Uuid,
    pub correlation_id: Uuid,
    pub idempotency_key: String,
    pub request_fingerprint: [u8; 32],
}

/// Current Group authorization context supplied to Local Runtime for a session operation.
#[derive(Clone, Debug)]
pub struct TaskCliSessionAccessCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub session_id: Uuid,
    pub correlation_id: Uuid,
}

/// Authorized, bounded request for recent sessions attached to one canonical Task/Worktree.
#[derive(Clone, Debug)]
pub struct TaskCliSessionListCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub correlation_id: Uuid,
    pub limit: usize,
}

/// Successful response data. The attachment ticket is short-lived and returned only once over
/// authenticated HTTPS with `Cache-Control: no-store`; it must never be logged or persisted by UI.
#[derive(Serialize)]
pub struct TaskCliSessionReceipt {
    pub session_id: Uuid,
    pub attachment_ticket: String,
    pub attachment_ticket_expires_at: DateTime<Utc>,
}

/// Public lifecycle states for the Task Card session list; terminal output is never included.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCliSessionState {
    Starting,
    Running,
    Disconnected,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
}

/// Redacted session projection returned to authorized Task Card clients.
#[derive(Clone, Debug, Serialize)]
pub struct TaskCliSessionStatus {
    pub session_id: Uuid,
    pub state: TaskCliSessionState,
    pub exit_code: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

/// Provisioning failures intentionally expose only stable public error categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskCliSessionProvisionError {
    RuntimeUnavailable,
    SessionConflict,
    SessionNotFound,
    Internal,
}

/// Trusted bridge from the REST authorization boundary to Local Runtime.
/// Implementations must be idempotent by `(tenant, actor, idempotency_key)`, compare the request
/// fingerprint on retries, and return only after a sandboxed session is actually running and a
/// fresh single-use WebSocket attachment ticket has been issued. Every lifecycle operation must
/// compare the complete session binding and recheck current ACL and Runtime health. Lifecycle
/// operations are correlated for TaskRun Audit, cancel is idempotent, and reattach must issue a
/// new ticket rather than return a ticket previously issued to the browser.
#[async_trait]
pub trait TaskCliSessionProvisioner: Send + Sync {
    async fn start_task_cli_session(
        &self,
        command: TaskCliSessionStartCommand,
    ) -> Result<TaskCliSessionReceipt, TaskCliSessionProvisionError>;

    /// Return recent redacted session metadata for this canonical Task binding, without tickets
    /// or terminal output. The implementation must recheck the complete current binding.
    async fn list_task_cli_sessions(
        &self,
        command: TaskCliSessionListCommand,
    ) -> Result<Vec<TaskCliSessionStatus>, TaskCliSessionProvisionError>;

    async fn task_cli_session_status(
        &self,
        access: TaskCliSessionAccessCommand,
    ) -> Result<TaskCliSessionStatus, TaskCliSessionProvisionError>;

    async fn cancel_task_cli_session(
        &self,
        access: TaskCliSessionAccessCommand,
    ) -> Result<TaskCliSessionStatus, TaskCliSessionProvisionError>;

    async fn reattach_task_cli_session(
        &self,
        access: TaskCliSessionAccessCommand,
    ) -> Result<TaskCliSessionReceipt, TaskCliSessionProvisionError>;
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StartTaskCliSessionBody {
    expected_lifecycle_version: i32,
    approved_launch_profile_id: Uuid,
    correlation_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskCliSessionListQuery {
    limit: Option<usize>,
}

#[derive(Clone, Debug)]
struct TaskCliSessionParent {
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    runtime_id: Uuid,
    correlation_id: Uuid,
}

#[derive(Debug, FromRow)]
struct WorktreeTaskCliScope {
    project_id: Uuid,
    repository_id: Uuid,
    runtime_id: Option<Uuid>,
    archived: bool,
}

#[derive(Debug, FromRow)]
struct TaskCliLifecycle {
    status: String,
    review_state: String,
    active_worktree_id: Option<Uuid>,
    claimed_by: Option<Uuid>,
    version: i32,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new().route(
        "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions",
        get(list_task_cli_sessions).post(start_task_cli_session),
    ).route(
        "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}",
        get(get_task_cli_session_status).delete(cancel_task_cli_session),
    ).route(
        "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}/attachment-tickets",
        post(reattach_task_cli_session),
    )
}

async fn list_task_cli_sessions(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    Query(query): Query<TaskCliSessionListQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let limit = validate_session_list_limit(query.limit)?;
    let parent = authorize_task_cli_session_parent(
        &state,
        &actor,
        "agent_session:read",
        &worktree_id,
        &work_item_id,
        correlation_id,
    )
    .await?;
    let command = TaskCliSessionListCommand {
        tenant_id: parent.tenant_id,
        actor_id: parent.actor_id,
        project_id: parent.project_id,
        repository_id: parent.repository_id,
        worktree_id: parent.worktree_id,
        work_item_id: parent.work_item_id,
        runtime_id: parent.runtime_id,
        correlation_id: parent.correlation_id,
        limit,
    };
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let sessions = provisioner
        .list_task_cli_sessions(command)
        .await
        .map_err(map_provision_error)?;
    if sessions.len() > limit || !session_statuses_are_valid(&sessions) {
        return Err(GroupApiError::internal());
    }
    Ok(no_store(Json(json!({ "sessions": sessions }))))
}

async fn start_task_cli_session(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<StartTaskCliSessionBody>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "agent_session:start")?;
    validate_start_body(&body)?;

    let worktree_id = Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let work_item_id = Uuid::parse_str(&work_item_id).map_err(|_| GroupApiError::bad_request())?;
    let idempotency_key = super::work_items::idempotency_key(&headers)?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let worktree = sqlx::query_as::<_, WorktreeTaskCliScope>(
        r#"
        SELECT p.project_id, w.repo_id AS repository_id, w.runtime_id, w.archived
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
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let binding = active_binding(&mut tx, &actor, worktree.project_id).await?;
    super::work_items::require_task_writer(&binding.role)?;
    if worktree.archived {
        return Err(GroupApiError::conflict("worktree_archived"));
    }
    let runtime_id = worktree
        .runtime_id
        .ok_or_else(|| GroupApiError::conflict("runtime_not_assigned"))?;
    let request_fingerprint = request_fingerprint(
        actor.tenant_id,
        actor.user_id,
        worktree.project_id,
        worktree.repository_id,
        worktree_id,
        work_item_id,
        runtime_id,
        &body,
    )?;

    let lifecycle = sqlx::query_as::<_, TaskCliLifecycle>(
        r#"
        SELECT c.status, c.review_state, c.active_worktree_id, c.claimed_by, c.version
        FROM multica.work_item_worktree l
        JOIN multica.task_metadata m
          ON m.tenant_id = l.tenant_id AND m.work_item_id = l.work_item_id
         AND m.project_id = l.project_id AND m.valid_to IS NULL
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3
          AND l.work_item_id = $4 AND l.valid_to IS NULL
        FOR SHARE OF l, m, c
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;

    if lifecycle.version != body.expected_lifecycle_version {
        return Err(GroupApiError::conflict("version_conflict"));
    }
    if lifecycle.status != "in_progress"
        || lifecycle.review_state == "pending_review"
        || lifecycle.active_worktree_id != Some(worktree_id)
        || lifecycle.claimed_by != Some(actor.user_id)
    {
        return Err(GroupApiError::conflict("task_not_executable_in_worktree"));
    }

    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let receipt = provisioner
        .start_task_cli_session(TaskCliSessionStartCommand {
            tenant_id: actor.tenant_id,
            actor_id: actor.user_id,
            project_id: worktree.project_id,
            repository_id: worktree.repository_id,
            worktree_id,
            work_item_id,
            runtime_id,
            expected_lifecycle_version: body.expected_lifecycle_version,
            approved_launch_profile_id: body.approved_launch_profile_id,
            correlation_id: body.correlation_id,
            idempotency_key,
            request_fingerprint,
        })
        .await
        .map_err(map_provision_error)?;
    validate_receipt(&receipt, receipt.session_id)?;

    let mut response = Json(json!({
        "session_id": receipt.session_id,
        "worktree_id": worktree_id,
        "work_item_id": work_item_id,
        "runtime_id": runtime_id,
        "status": "running",
        "attachment_ticket": receipt.attachment_ticket,
        "attachment_ticket_expires_at": receipt.attachment_ticket_expires_at,
        "correlation_id": body.correlation_id,
    }))
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::PRAGMA,
        axum::http::HeaderValue::from_static("no-cache"),
    );
    Ok(response)
}

async fn get_task_cli_session_status(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, session_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let access = authorize_task_cli_session_access(
        &state,
        &actor,
        "agent_session:read",
        &worktree_id,
        &work_item_id,
        &session_id,
        correlation_id,
    )
    .await?;
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let status = provisioner
        .task_cli_session_status(access.clone())
        .await
        .map_err(map_provision_error)?;
    if status.session_id != access.session_id {
        return Err(GroupApiError::internal());
    }
    Ok(no_store(Json(status)))
}

async fn cancel_task_cli_session(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, session_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let access = authorize_task_cli_session_access(
        &state,
        &actor,
        "agent_session:cancel",
        &worktree_id,
        &work_item_id,
        &session_id,
        correlation_id,
    )
    .await?;
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let status = provisioner
        .cancel_task_cli_session(access.clone())
        .await
        .map_err(map_provision_error)?;
    if status.session_id != access.session_id {
        return Err(GroupApiError::internal());
    }
    Ok(no_store(Json(status)))
}

async fn reattach_task_cli_session(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, session_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let access = authorize_task_cli_session_access(
        &state,
        &actor,
        "agent_session:attach",
        &worktree_id,
        &work_item_id,
        &session_id,
        correlation_id,
    )
    .await?;
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let receipt = provisioner
        .reattach_task_cli_session(access.clone())
        .await
        .map_err(map_provision_error)?;
    validate_receipt(&receipt, access.session_id)?;
    let mut response = Json(json!({
        "session_id": receipt.session_id,
        "worktree_id": access.worktree_id,
        "work_item_id": access.work_item_id,
        "runtime_id": access.runtime_id,
        "status": "attachment_ticket_issued",
        "attachment_ticket": receipt.attachment_ticket,
        "attachment_ticket_expires_at": receipt.attachment_ticket_expires_at,
        "correlation_id": access.correlation_id,
    }))
    .into_response();
    set_no_store_headers(&mut response);
    Ok(response)
}

/// Re-resolve the canonical Task/Worktree relationship and current Project writer membership on
/// every session operation. The Runtime provisioner must repeat live policy and session-binding
/// checks after this transaction commits to close the authorization-to-action race.
async fn authorize_task_cli_session_parent(
    state: &GroupApiState,
    actor: &AuthUser,
    scope: &str,
    worktree_id: &str,
    work_item_id: &str,
    correlation_id: Uuid,
) -> Result<TaskCliSessionParent, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, scope)?;
    let worktree_id = Uuid::parse_str(worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let work_item_id = Uuid::parse_str(work_item_id).map_err(|_| GroupApiError::bad_request())?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let worktree = sqlx::query_as::<_, WorktreeTaskCliScope>(
        r#"
        SELECT p.project_id, w.repo_id AS repository_id, w.runtime_id, w.archived
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
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let binding = active_binding(&mut tx, actor, worktree.project_id).await?;
    super::work_items::require_task_writer(&binding.role)?;
    if worktree.archived && scope == "agent_session:attach" {
        return Err(GroupApiError::conflict("worktree_archived"));
    }

    let _task_exists = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM multica.work_item_worktree l
        JOIN multica.task_metadata m
          ON m.tenant_id = l.tenant_id AND m.work_item_id = l.work_item_id
         AND m.project_id = l.project_id AND m.valid_to IS NULL
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3
          AND l.work_item_id = $4 AND l.valid_to IS NULL
        FOR SHARE OF l, m, c
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let runtime_id = worktree
        .runtime_id
        .ok_or_else(|| GroupApiError::conflict("runtime_not_assigned"))?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    Ok(TaskCliSessionParent {
        tenant_id: actor.tenant_id,
        actor_id: actor.user_id,
        project_id: worktree.project_id,
        repository_id: worktree.repository_id,
        worktree_id,
        work_item_id,
        runtime_id,
        correlation_id,
    })
}

async fn authorize_task_cli_session_access(
    state: &GroupApiState,
    actor: &AuthUser,
    scope: &str,
    worktree_id: &str,
    work_item_id: &str,
    session_id: &str,
    correlation_id: Uuid,
) -> Result<TaskCliSessionAccessCommand, GroupApiError> {
    let session_id = Uuid::parse_str(session_id).map_err(|_| GroupApiError::bad_request())?;
    let parent = authorize_task_cli_session_parent(
        state,
        actor,
        scope,
        worktree_id,
        work_item_id,
        correlation_id,
    )
    .await?;
    Ok(TaskCliSessionAccessCommand {
        tenant_id: parent.tenant_id,
        actor_id: parent.actor_id,
        project_id: parent.project_id,
        repository_id: parent.repository_id,
        worktree_id: parent.worktree_id,
        work_item_id: parent.work_item_id,
        runtime_id: parent.runtime_id,
        session_id,
        correlation_id: parent.correlation_id,
    })
}

fn validate_session_list_limit(limit: Option<usize>) -> Result<usize, GroupApiError> {
    let limit = limit.unwrap_or(20);
    if !(1..=50).contains(&limit) {
        return Err(GroupApiError::bad_request());
    }
    Ok(limit)
}

fn session_statuses_are_valid(sessions: &[TaskCliSessionStatus]) -> bool {
    let mut ids = std::collections::HashSet::with_capacity(sessions.len());
    sessions
        .iter()
        .all(|session| !session.session_id.is_nil() && ids.insert(session.session_id))
}

fn required_correlation_id(headers: &axum::http::HeaderMap) -> Result<Uuid, GroupApiError> {
    let value = headers
        .get("x-correlation-id")
        .and_then(|header| header.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .filter(|value| !value.is_nil())
        .ok_or_else(|| GroupApiError::invalid_request("correlation_id_required"))?;
    Ok(value)
}

fn validate_receipt(
    receipt: &TaskCliSessionReceipt,
    expected_session_id: Uuid,
) -> Result<(), GroupApiError> {
    let now = Utc::now();
    if receipt.session_id != expected_session_id
        || receipt.session_id.is_nil()
        || receipt.attachment_ticket.is_empty()
        || receipt.attachment_ticket.len() > 512
        || receipt.attachment_ticket_expires_at <= now
        || receipt.attachment_ticket_expires_at > now + Duration::seconds(60)
    {
        return Err(GroupApiError::internal());
    }
    Ok(())
}

fn no_store(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    set_no_store_headers(&mut response);
    response
}

fn set_no_store_headers(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::PRAGMA,
        axum::http::HeaderValue::from_static("no-cache"),
    );
}

fn validate_start_body(body: &StartTaskCliSessionBody) -> Result<(), GroupApiError> {
    if body.expected_lifecycle_version < 1
        || body.approved_launch_profile_id.is_nil()
        || body.correlation_id.is_nil()
    {
        return Err(GroupApiError::invalid_request(
            "invalid_task_cli_session_start",
        ));
    }
    Ok(())
}

fn request_fingerprint(
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    runtime_id: Uuid,
    body: &StartTaskCliSessionBody,
) -> Result<[u8; 32], GroupApiError> {
    let encoded = serde_json::to_vec(&(
        tenant_id,
        actor_id,
        project_id,
        repository_id,
        worktree_id,
        work_item_id,
        runtime_id,
        body,
    ))
    .map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(encoded).into())
}

fn map_provision_error(error: TaskCliSessionProvisionError) -> GroupApiError {
    match error {
        TaskCliSessionProvisionError::RuntimeUnavailable => GroupApiError::service_unavailable(),
        TaskCliSessionProvisionError::SessionConflict => {
            GroupApiError::conflict("task_cli_session_conflict")
        }
        TaskCliSessionProvisionError::SessionNotFound => GroupApiError::not_found(),
        TaskCliSessionProvisionError::Internal => GroupApiError::internal(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_list_limit_is_bounded() {
        assert_eq!(validate_session_list_limit(None).unwrap(), 20);
        assert_eq!(validate_session_list_limit(Some(1)).unwrap(), 1);
        assert_eq!(validate_session_list_limit(Some(50)).unwrap(), 50);
        assert!(validate_session_list_limit(Some(0)).is_err());
        assert!(validate_session_list_limit(Some(51)).is_err());
    }

    #[test]
    fn session_list_rejects_nil_and_duplicate_ids() {
        let session_id = Uuid::new_v4();
        let status = |session_id| TaskCliSessionStatus {
            session_id,
            state: TaskCliSessionState::Running,
            exit_code: None,
            updated_at: Utc::now(),
        };

        assert!(session_statuses_are_valid(&[status(session_id)]));
        assert!(!session_statuses_are_valid(&[
            status(session_id),
            status(session_id)
        ]));
        assert!(!session_statuses_are_valid(&[status(Uuid::nil())]));
    }
}
