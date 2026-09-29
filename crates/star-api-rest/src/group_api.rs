//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"group_api.rs",type:"file",language:"rust"}),(m:Module {name:"group_api",type:"module",language:"rust"}),
//!   (s:Class {name:"GroupApiState",type:"class"}),(r:Class {name:"GroupContextResolver",type:"class"}),(e:Class {name:"GroupApiError",type:"class"}),
//!   (b:Class {name:"ProjectBinding",type:"class"}),(w:Class {name:"WorktreeIndexRow",type:"class"}),
//!   (mi:Module {name:"work_items",type:"module",language:"rust"}),(mt:Module {name:"worktrees",type:"module",language:"rust"}),
//!   (sn:Function {name:"GroupApiState::new",type:"function"}),(sf:Function {name:"GroupApiState::from_ref",type:"function"}),(rn:Function {name:"GroupContextResolver::new",type:"function"}),(rc:Function {name:"GroupContextResolver::resolve_worktree_context",type:"function"}),(fr:Function {name:"FromRef::from_ref",type:"function"}),
//!   (eb:Function {name:"GroupApiError::bad_request",type:"function"}),(eu:Function {name:"GroupApiError::unauthorized",type:"function"}),(ef:Function {name:"GroupApiError::forbidden",type:"function"}),(en:Function {name:"GroupApiError::not_found",type:"function"}),(ei:Function {name:"GroupApiError::internal",type:"function"}),(ec:Function {name:"GroupApiError::conflict",type:"function"}),(iv:Function {name:"GroupApiError::invalid_request",type:"function"}),(er:Function {name:"GroupApiError::into_response",type:"function"}),
//!   (st:Function {name:"set_tenant",type:"function"}),(ab:Function {name:"active_binding",type:"function"}),(va:Function {name:"validate_actor",type:"function"}),(rs:Function {name:"require_scope",type:"function"}),(re:Function {name:"resolve_worktree_context",type:"function"}),(bu:Function {name:"build_group_router",type:"function"}),(wp:Function {name:"worktree_projection",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(s),(m)-[:CONTAINS]->(r),(m)-[:CONTAINS]->(e),(m)-[:CONTAINS]->(b),(m)-[:CONTAINS]->(w),(m)-[:CONTAINS]->(mi),(m)-[:CONTAINS]->(mt),
//!   (s)-[:HAS_METHOD]->(sn),(s)-[:HAS_METHOD]->(sf),(r)-[:HAS_METHOD]->(rn),(r)-[:HAS_METHOD]->(rc),(e)-[:HAS_METHOD]->(eb),(e)-[:HAS_METHOD]->(eu),(e)-[:HAS_METHOD]->(ef),(e)-[:HAS_METHOD]->(en),(e)-[:HAS_METHOD]->(ei),(e)-[:HAS_METHOD]->(ec),(e)-[:HAS_METHOD]->(iv),(e)-[:HAS_METHOD]->(er),(m)-[:CONTAINS]->(fr),
//!   (m)-[:CONTAINS]->(st),(m)-[:CONTAINS]->(ab),(m)-[:CONTAINS]->(va),(m)-[:CONTAINS]->(rs),(m)-[:CONTAINS]->(re),(m)-[:CONTAINS]->(bu),(m)-[:CONTAINS]->(wp),
//!   (st)-[:CALLS]->(ei),(ab)-[:CALLS]->(ei),(ab)-[:CALLS]->(en),(va)-[:CALLS]->(eu),(rs)-[:CALLS]->(ef),(rc)-[:CALLS]->(st),(rc)-[:CALLS]->(ab),(rc)-[:CALLS]->(ei),(rc)-[:CALLS]->(en),(re)-[:CALLS]->(va),(re)-[:CALLS]->(rs),(re)-[:CALLS]->(eb),(re)-[:CALLS]->(rc),(rc)-[:CALLS]->(wp),(sn)-[:CALLS]->(rn),(bu)-[:CALLS]->(re);

use std::sync::Arc;

use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::auth::{oauth::AuthenticatedUser, AuthUser, JwtConfig};

mod work_items;
mod worktrees;

#[derive(Clone)]
pub struct GroupApiState {
    jwt: Arc<JwtConfig>,
    resolver: GroupContextResolver,
}

impl GroupApiState {
    pub fn new(jwt: Arc<JwtConfig>, pool: PgPool) -> Self {
        Self {
            jwt,
            resolver: GroupContextResolver::new(pool),
        }
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
        .with_state(state)
}
