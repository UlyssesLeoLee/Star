//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"group_api.rs",type:"file",language:"rust"}),(m:Module {name:"group_api",type:"module",language:"rust"}),
//!   (ga:Module {name:"group_apps",type:"module",language:"rust"}),(gp:Interface {name:"GroupAppRegistryProvider",type:"interface",language:"rust"}),
//!   (s:Class {name:"GroupApiState",type:"class"}),(r:Class {name:"GroupContextResolver",type:"class"}),(e:Class {name:"GroupApiError",type:"class"}),(cp:Module {name:"cli_sessions",type:"module",language:"rust"}),(tp:Interface {name:"TaskCliSessionProvisioner",type:"interface"}),(pe:Enum {name:"TaskCliSessionProvisionError",type:"enum"}),(cm:Class {name:"TaskCliSessionStartCommand",type:"class"}),(receipt:Class {name:"TaskCliSessionReceipt",type:"class"}),
//!   (b:Class {name:"ProjectBinding",type:"class"}),(w:Class {name:"WorktreeIndexRow",type:"class"}),
//!   (mi:Module {name:"work_items",type:"module",language:"rust"}),(mt:Module {name:"worktrees",type:"module",language:"rust"}),(ca:Module {name:"canvas",type:"module",language:"rust"}),
//!   (sn:Function {name:"GroupApiState::new",type:"function"}),(sp:Function {name:"GroupApiState::with_task_cli_session_provisioner",type:"function"}),(sf:Function {name:"GroupApiState::from_ref",type:"function"}),(rn:Function {name:"GroupContextResolver::new",type:"function"}),(rc:Function {name:"GroupContextResolver::resolve_worktree_context",type:"function"}),(fr:Function {name:"FromRef::from_ref",type:"function"}),(csr:Function {name:"cli_sessions::router",type:"function"}),
//!   (eb:Function {name:"GroupApiError::bad_request",type:"function"}),(eu:Function {name:"GroupApiError::unauthorized",type:"function"}),(ef:Function {name:"GroupApiError::forbidden",type:"function"}),(en:Function {name:"GroupApiError::not_found",type:"function"}),(ei:Function {name:"GroupApiError::internal",type:"function"}),(ec:Function {name:"GroupApiError::conflict",type:"function"}),(iv:Function {name:"GroupApiError::invalid_request",type:"function"}),(us:Function {name:"GroupApiError::service_unavailable",type:"function"}),(er:Function {name:"GroupApiError::into_response",type:"function"}),
//!   (st:Function {name:"set_tenant",type:"function"}),(ab:Function {name:"active_binding",type:"function"}),(va:Function {name:"validate_actor",type:"function"}),(rs:Function {name:"require_scope",type:"function"}),(re:Function {name:"resolve_worktree_context",type:"function"}),(bu:Function {name:"build_group_router",type:"function"}),(wp:Function {name:"worktree_projection",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(s),(m)-[:CONTAINS]->(r),(m)-[:CONTAINS]->(e),(m)-[:CONTAINS]->(b),(m)-[:CONTAINS]->(w),(m)-[:CONTAINS]->(mi),(m)-[:CONTAINS]->(mt),(m)-[:CONTAINS]->(ca),(m)-[:CONTAINS]->(cp),(cp)-[:CONTAINS]->(tp),(cp)-[:CONTAINS]->(pe),(cp)-[:CONTAINS]->(cm),(cp)-[:CONTAINS]->(receipt),(cp)-[:CONTAINS]->(csr),
//!   (s)-[:HAS_METHOD]->(sn),(s)-[:HAS_METHOD]->(sp),(s)-[:HAS_METHOD]->(sf),(r)-[:HAS_METHOD]->(rn),(r)-[:HAS_METHOD]->(rc),(e)-[:HAS_METHOD]->(eb),(e)-[:HAS_METHOD]->(eu),(e)-[:HAS_METHOD]->(ef),(e)-[:HAS_METHOD]->(en),(e)-[:HAS_METHOD]->(ei),(e)-[:HAS_METHOD]->(ec),(e)-[:HAS_METHOD]->(iv),(e)-[:HAS_METHOD]->(us),(e)-[:HAS_METHOD]->(er),(m)-[:CONTAINS]->(fr),
//!   (m)-[:CONTAINS]->(st),(m)-[:CONTAINS]->(ab),(m)-[:CONTAINS]->(va),(m)-[:CONTAINS]->(rs),(m)-[:CONTAINS]->(re),(m)-[:CONTAINS]->(bu),(m)-[:CONTAINS]->(wp),
//!   (st)-[:CALLS]->(ei),(ab)-[:CALLS]->(ei),(ab)-[:CALLS]->(en),(va)-[:CALLS]->(eu),(rs)-[:CALLS]->(ef),(rc)-[:CALLS]->(st),(rc)-[:CALLS]->(ab),(rc)-[:CALLS]->(ei),(rc)-[:CALLS]->(en),(re)-[:CALLS]->(va),(re)-[:CALLS]->(rs),(re)-[:CALLS]->(eb),(re)-[:CALLS]->(rc),(rc)-[:CALLS]->(wp),(sn)-[:CALLS]->(rn),(bu)-[:CALLS]->(re),(bu)-[:CALLS]->(csr);

//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"group_api",type:"module"}),(s:Class {name:"GroupApiState",type:"class"}),(e:Class {name:"GroupApiError",type:"class"}),(b:Function {name:"build_group_router",type:"function"}),(su:Function {name:"GroupApiError::service_unavailable",type:"function"});
//! CREATE (sc:Module {name:"scoped_chat",type:"module",language:"rust"}),(wf:Interface {name:"ScopedChatWorkflow",type:"interface",language:"rust"}),(sw:Function {name:"GroupApiState::with_scoped_chat_workflow",type:"function",language:"rust"}),(fu:Function {name:"GroupApiError::feature_unavailable",type:"function",language:"rust"}),(sr:Function {name:"scoped_chat::router",type:"function",language:"rust"});
//! MATCH (b:Function {name:"build_group_router",type:"function"}),(sw:Function {name:"GroupApiState::with_scoped_chat_workflow",type:"function"}),(s:Class {name:"GroupApiState",type:"class"}),(e:Class {name:"GroupApiError",type:"class"}),(fu:Function {name:"GroupApiError::feature_unavailable",type:"function"}),(sr:Function {name:"scoped_chat::router",type:"function"}),(sc:Module {name:"scoped_chat",type:"module"}),(wf:Interface {name:"ScopedChatWorkflow",type:"interface"});
//! CREATE (m)-[:CONTAINS]->(sc),(s)-[:USES]->(wf),(sw)-[:USES]->(wf),(e)-[:HAS_METHOD]->(fu),(b)-[:CALLS]->(sr),(sc)-[:CONTAINS]->(wf),(sr)-[:CALLS]->(sw),(fu)-[:CALLS]->(su);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"group_api",type:"module"}),(s:Class {name:"GroupApiState",type:"class"}),(w:Interface {name:"ScopedChatWorkflow",type:"interface"});
//! CREATE (ss:Module {name:"scoped_chat_store",type:"module",language:"rust"}),(pg:Class {name:"PgScopedChatWorkflow",type:"class",language:"rust"}),(pgnew:Function {name:"PgScopedChatWorkflow::new",type:"function",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(ss),(s)-[:USES]->(pg),(ss)-[:CONTAINS]->(pg),(pg)-[:USES]->(w),(pg)-[:HAS_METHOD]->(pgnew);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(s:Class {name:"GroupApiState",type:"class"});
//! CREATE (access:Class {name:"TaskCliSessionAccessCommand",type:"class",language:"rust"}),(status:Class {name:"TaskCliSessionStatus",type:"class",language:"rust"}),(state:Enum {name:"TaskCliSessionState",type:"enum",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(access),(m)-[:CONTAINS]->(status),(m)-[:CONTAINS]->(state),(s)-[:USES]->(access);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"});
//! CREATE (list:Class {name:"TaskCliSessionListCommand",type:"class",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(list);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (ss:Module {name:"scoped_chat_store",type:"module"});
//! CREATE (protector:Interface {name:"TranscriptBodyProtector",type:"interface",language:"rust"}),(context:Class {name:"TranscriptProtectionContext",type:"class",language:"rust"}),(body:Class {name:"ProtectedTranscriptBody",type:"class",language:"rust"});
//! CREATE (ss)-[:CONTAINS]->(protector),(ss)-[:CONTAINS]->(context),(ss)-[:CONTAINS]->(body);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"group_api",type:"module"}),(ga:Module {name:"group_apps",type:"module"}),(s:Class {name:"GroupApiState",type:"class"}),(gp:Interface {name:"GroupAppRegistryProvider",type:"interface"});
//! CREATE (install:Function {name:"GroupApiState::with_group_app_registry",type:"function",language:"rust"}),(registryRouter:Function {name:"group_apps::router",type:"function",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(ga),(ga)-[:CONTAINS]->(gp),(s)-[:HAS_METHOD]->(install),(install)-[:USES]->(gp),(registryRouter)-[:CALLS]->(ga);

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::auth::{AuthUser, JwtConfig, oauth::AuthenticatedUser};

mod canvas;
mod cli_sessions;
mod group_apps;
mod scoped_chat;
mod scoped_chat_store;
mod work_items;
mod worktrees;

pub use cli_sessions::{
    TaskCliSessionAccessCommand, TaskCliSessionListCommand, TaskCliSessionProvisionError,
    TaskCliSessionProvisioner, TaskCliSessionReceipt, TaskCliSessionStartCommand,
    TaskCliSessionState, TaskCliSessionStatus,
};
pub use group_apps::{
    GroupAppNavigationEntry, GroupAppRegistryError, GroupAppRegistryProvider,
    GroupAppRegistryQuery, GroupAppRegistrySnapshot, PgGroupAppRegistryProvider,
};
pub use scoped_chat::{
    AuthorizedChatTarget, ScopedChatCommand, ScopedChatEntityRef, ScopedChatReceipt,
    ScopedChatScope, ScopedChatWorkflow, ScopedChatWorkflowError,
};
pub use scoped_chat_store::{
    PgScopedChatWorkflow, ProtectedTranscriptBody, TranscriptBodyProtector,
    TranscriptProtectionContext,
};

#[derive(Clone)]
pub struct GroupApiState {
    jwt: Arc<JwtConfig>,
    resolver: GroupContextResolver,
    task_cli_session_provisioner: Option<Arc<dyn TaskCliSessionProvisioner>>,
    scoped_chat_workflow: Option<Arc<dyn ScopedChatWorkflow>>,
    group_app_registry: Option<Arc<dyn GroupAppRegistryProvider>>,
}

impl GroupApiState {
    pub fn new(jwt: Arc<JwtConfig>, pool: PgPool) -> Self {
        Self {
            jwt,
            resolver: GroupContextResolver::new(pool),
            task_cli_session_provisioner: None,
            scoped_chat_workflow: None,
            group_app_registry: None,
        }
    }

    /// Install the trusted Local Runtime session provisioner for Task Card CLI routes.
    pub fn with_task_cli_session_provisioner(
        mut self,
        provisioner: Arc<dyn TaskCliSessionProvisioner>,
    ) -> Self {
        self.task_cli_session_provisioner = Some(provisioner);
        self
    }

    /// Install the trusted scope-aware chat/L0 workflow adapter.
    pub fn with_scoped_chat_workflow(mut self, workflow: Arc<dyn ScopedChatWorkflow>) -> Self {
        self.scoped_chat_workflow = Some(workflow);
        self
    }

    /// Install a trusted provider for the current, authorized Group App navigation projection.
    pub fn with_group_app_registry(mut self, registry: Arc<dyn GroupAppRegistryProvider>) -> Self {
        self.group_app_registry = Some(registry);
        self
    }
}

impl FromRef<GroupApiState> for Arc<JwtConfig> {
    fn from_ref(state: &GroupApiState) -> Self {
        state.jwt.clone()
    }
}

#[derive(Clone)]
pub struct GroupContextResolver {
    pool: PgPool,
}

impl GroupContextResolver {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn resolve_worktree_context(
        &self,
        actor: &AuthUser,
        worktree_id: Uuid,
    ) -> Result<Value, GroupApiError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| GroupApiError::internal())?;
        set_tenant(&mut tx, actor.tenant_id).await?;
        let worktree = sqlx::query_as::<_, WorktreeIndexRow>(
            r#"
            SELECT w.id, w.name, w.repo_id, w.workspace_id, p.project_id, w.path, w.parent_id,
                   w.owner_user_id, w.work_item_id, w.agent_id, w.agent_session_id, w.runtime_id,
                   w.branch, w.human_state, w.machine_state,
                   w.ahead, w.behind, w.dirty, w.health_score, w.last_activity, w.test_state,
                   w.risk_count, w.locked, w.archived, w.version, w.pull_request_url,
                   w.created_at, w.updated_at
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
        tx.commit().await.map_err(|_| GroupApiError::internal())?;

        let granted_scopes: Vec<String> = actor
            .scope
            .split_ascii_whitespace()
            .map(str::to_owned)
            .collect();
        Ok(json!({
            "group_context": {
                "tenant_id": actor.tenant_id,
                "workspace_id": worktree.workspace_id,
                "project_id": worktree.project_id,
                "repository_id": worktree.repo_id,
                "worktree_id": worktree.id,
                "actor_id": actor.user_id,
                "actor_kind": "user",
                "actor_roles": [binding.role],
                "granted_scopes": granted_scopes,
                "context_version": binding.version,
                "resolved_at": chrono::Utc::now(),
                "permission_snapshot_ref": format!("{}:v{}", binding.id, binding.version),
                "correlation_id": Uuid::new_v4(),
            },
            "worktree": worktree_projection(worktree),
        }))
    }
}

#[derive(Debug)]
struct GroupApiError {
    status: StatusCode,
    code: &'static str,
}

impl GroupApiError {
    fn bad_request() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_id",
        }
    }
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "invalid_actor",
        }
    }
    fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "insufficient_scope",
        }
    }
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "resource_not_found",
        }
    }
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
        }
    }

    fn service_unavailable() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "task_cli_runtime_unavailable",
        }
    }

    fn feature_unavailable(code: &'static str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code,
        }
    }

    fn conflict(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }

    fn invalid_request(code: &'static str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code,
        }
    }
}

impl IntoResponse for GroupApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": { "code": self.code } }))).into_response()
    }
}

#[derive(Debug, FromRow)]
struct ProjectBinding {
    id: Uuid,
    role: String,
    version: i32,
}

#[derive(Debug, FromRow)]
struct WorktreeIndexRow {
    id: Uuid,
    name: String,
    repo_id: Uuid,
    workspace_id: Uuid,
    project_id: Uuid,
    path: String,
    parent_id: Option<Uuid>,
    owner_user_id: Option<Uuid>,
    work_item_id: Option<Uuid>,
    agent_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    runtime_id: Option<Uuid>,
    branch: String,
    human_state: String,
    machine_state: String,
    ahead: i32,
    behind: i32,
    dirty: bool,
    health_score: i16,
    last_activity: chrono::DateTime<chrono::Utc>,
    test_state: String,
    risk_count: i32,
    locked: bool,
    archived: bool,
    version: i32,
    pull_request_url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

fn worktree_projection(row: WorktreeIndexRow) -> Value {
    json!({
        "id": row.id,
        "name": row.name,
        "repository_id": row.repo_id,
        "workspace_id": row.workspace_id,
        "project_id": row.project_id,
        "path": row.path,
        "parent_id": row.parent_id,
        "owner_user_id": row.owner_user_id,
        "work_item_id": row.work_item_id,
        "agent_id": row.agent_id,
        "agent_session_id": row.agent_session_id,
        "runtime_id": row.runtime_id,
        "branch": row.branch,
        "human_state": row.human_state,
        "machine_state": row.machine_state,
        "ahead": row.ahead,
        "behind": row.behind,
        "dirty": row.dirty,
        "health_score": row.health_score,
        "last_activity": row.last_activity,
        "test_state": row.test_state,
        "risk_count": row.risk_count,
        "locked": row.locked,
        "archived": row.archived,
        "version": row.version,
        "pull_request_url": row.pull_request_url,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}

async fn set_tenant(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), GroupApiError> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn active_binding(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
) -> Result<ProjectBinding, GroupApiError> {
    sqlx::query_as::<_, ProjectBinding>(
        r#"
        SELECT id, role, version
        FROM permission.project_role_binding
        WHERE tenant_id = $1
          AND project_id = $2
          AND user_id = $3
          AND valid_to IS NULL
        FOR SHARE
        "#,
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(actor.user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)
}

fn validate_actor(actor: &AuthUser) -> Result<(), GroupApiError> {
    if actor.user_id.is_nil() || actor.tenant_id.is_nil() {
        return Err(GroupApiError::unauthorized());
    }
    Ok(())
}

fn require_scope(actor: &AuthUser, required: &str) -> Result<(), GroupApiError> {
    if actor
        .scope
        .split_ascii_whitespace()
        .any(|scope| scope == required)
    {
        Ok(())
    } else {
        Err(GroupApiError::forbidden())
    }
}

async fn resolve_worktree_context(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
) -> Result<Json<Value>, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "worktree:read")?;
    let worktree_id = Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let result = state
        .resolver
        .resolve_worktree_context(&actor, worktree_id)
        .await?;
    Ok(Json(result))
}

pub fn build_group_router(state: GroupApiState) -> Router {
    Router::new()
        .route(
            "/api/v1/worktrees/{worktree_id}/group-context",
            get(resolve_worktree_context),
        )
        .merge(worktrees::router())
        .merge(work_items::router())
        .merge(cli_sessions::router())
        .merge(scoped_chat::router())
        .merge(canvas::router())
        .merge(group_apps::router())
        .with_state(state)
}
