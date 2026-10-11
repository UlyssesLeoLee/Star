//! Cypher structural manifest.
//! CREATE (f:File {name:"engineering_runs.rs",type:"file",language:"rust"}),
//! (query:Class {name:"DirectoryQuery",type:"class"}),(cursor:Class {name:"DirectoryCursor",type:"class"}),
//! (page:Class {name:"DirectoryPage",type:"class"}),(branch:Class {name:"CloudBranchRow",type:"class"}),
//! (run:Class {name:"EngineeringRunRow",type:"class"}),(grant:Class {name:"DirectoryGrant",type:"class"}),
//! (auth:Class {name:"RunAuthority",type:"class"}),(focus:Class {name:"RunWorktreeRow",type:"class"}),
//! (contextQuery:Class {name:"ContextQuery",type:"class"}),
//! (router:Function {name:"router",type:"function"}),(parse:Function {name:"parse_id",type:"function"}),
//! (read:Function {name:"begin_read",type:"function"}),(br:Function {name:"branch_authority",type:"function"}),
//! (ra:Function {name:"run_authority",type:"function"}),(branches:Function {name:"list_branches",type:"function"}),
//! (runs:Function {name:"list_runs",type:"function"}),(worktrees:Function {name:"list_run_worktrees",type:"function"}),
//! (context:Function {name:"resolve_run_context",type:"function"}),(legacy:Function {name:"resolve_worktree_run_context",type:"function"}),
//! (load:Function {name:"load_focus",type:"function"}),(envelope:Function {name:"context_envelope",type:"function"}),
//! (reply:Function {name:"reply",type:"function"}),(decode:Function {name:"DirectoryQuery::page",type:"function"}),
//! (next:Function {name:"DirectoryPage::next_cursor",type:"function"}),(permission:Function {name:"DirectoryGrant::snapshot",type:"function"}),
//! (sql:Variable {name:"WORKTREE_SELECT",type:"variable"}),
//! (f)-[:CONTAINS]->(query),(f)-[:CONTAINS]->(cursor),(f)-[:CONTAINS]->(page),(f)-[:CONTAINS]->(branch),(f)-[:CONTAINS]->(run),(f)-[:CONTAINS]->(grant),(f)-[:CONTAINS]->(auth),(f)-[:CONTAINS]->(focus),(f)-[:CONTAINS]->(contextQuery),
//! (f)-[:CONTAINS]->(router),(f)-[:CONTAINS]->(parse),(f)-[:CONTAINS]->(read),(f)-[:CONTAINS]->(br),(f)-[:CONTAINS]->(ra),(f)-[:CONTAINS]->(branches),(f)-[:CONTAINS]->(runs),(f)-[:CONTAINS]->(worktrees),(f)-[:CONTAINS]->(context),(f)-[:CONTAINS]->(legacy),(f)-[:CONTAINS]->(load),(f)-[:CONTAINS]->(envelope),(f)-[:CONTAINS]->(reply),(f)-[:CONTAINS]->(sql),
//! (query)-[:HAS_METHOD]->(decode),(page)-[:HAS_METHOD]->(next),(grant)-[:HAS_METHOD]->(permission),
//! (router)-[:CALLS]->(branches),(router)-[:CALLS]->(runs),(router)-[:CALLS]->(worktrees),(router)-[:CALLS]->(context),(router)-[:CALLS]->(legacy),
//! (branches)-[:CALLS]->(decode),(runs)-[:CALLS]->(decode),(worktrees)-[:CALLS]->(decode),(branches)-[:CALLS]->(read),(runs)-[:CALLS]->(read),(worktrees)-[:CALLS]->(read),(context)-[:CALLS]->(read),(legacy)-[:CALLS]->(read),
//! (runs)-[:CALLS]->(br),(ra)-[:CALLS]->(br),(worktrees)-[:CALLS]->(ra),(context)-[:CALLS]->(ra),(legacy)-[:CALLS]->(ra),(context)-[:CALLS]->(load),(legacy)-[:CALLS]->(load),(context)-[:CALLS]->(envelope),(legacy)-[:CALLS]->(envelope),(envelope)-[:CALLS]->(permission),(load)-[:USES]->(sql),(worktrees)-[:USES]->(sql);

//! CYPHER STRUCTURE MANIFEST: external call relationships.
//! MATCH (f:File {name:"engineering_runs.rs",type:"file"})
//! CREATE (sqlQuery:Function {name:"sqlx::query_as",type:"function"}),(sqlScalar:Function {name:"sqlx::query_scalar",type:"function"}),(sqlPlain:Function {name:"sqlx::query",type:"function"}),(execute:Function {name:"Query::execute",type:"function"}),
//! (validate:Function {name:"validate_actor",type:"function"}),(scope:Function {name:"require_scope",type:"function"}),(tenant:Function {name:"set_tenant",type:"function"}),(active:Function {name:"active_binding",type:"function"}),
//! (begin:Function {name:"PgPool::begin",type:"function"}),(commit:Function {name:"Transaction::commit",type:"function"}),
//! (bind:Function {name:"Query::bind",type:"function"}),(fetch:Function {name:"Query::fetch_optional",type:"function"}),(fetchAll:Function {name:"Query::fetch_all",type:"function"}),
//! (bad:Function {name:"GroupApiError::bad_request",type:"function"}),(internal:Function {name:"GroupApiError::internal",type:"function"}),(missing:Function {name:"GroupApiError::not_found",type:"function"}),(invalid:Function {name:"GroupApiError::invalid_request",type:"function"}),
//! (decode64:Function {name:"URL_SAFE_NO_PAD::decode",type:"function"}),(encode64:Function {name:"URL_SAFE_NO_PAD::encode",type:"function"}),(fromJson:Function {name:"serde_json::from_slice",type:"function"}),(toJson:Function {name:"serde_json::to_vec",type:"function"}),
//! (parseUuid:Function {name:"Uuid::parse_str",type:"function"}),(nil:Function {name:"Uuid::is_nil",type:"function"}),(uuid:Function {name:"Uuid::new_v4",type:"function"}),(clock:Function {name:"Utc::now",type:"function"}),
//! (json:Function {name:"json!",type:"function"}),(format:Function {name:"format!",type:"function"}),(get:Function {name:"axum::routing::get",type:"function"}),(new:Function {name:"Router::new",type:"function"}),(route:Function {name:"Router::route",type:"function"}),(wrap:Function {name:"Json",type:"function"})
//! WITH f, sqlQuery, sqlScalar, sqlPlain, execute, validate, scope, tenant, active, begin, commit, bind, fetch, fetchAll, bad, internal, missing, invalid, decode64, encode64, fromJson, toJson, parseUuid, nil, uuid, clock, json, format, get, new, route, wrap
//! MATCH (decode:Function {name:"DirectoryQuery::page",type:"function"}),(next:Function {name:"DirectoryPage::next_cursor",type:"function"}),(parse:Function {name:"parse_id",type:"function"}),
//! (read:Function {name:"begin_read",type:"function"}),(br:Function {name:"branch_authority",type:"function"}),(ra:Function {name:"run_authority",type:"function"}),
//! (branches:Function {name:"list_branches",type:"function"}),(runs:Function {name:"list_runs",type:"function"}),(worktrees:Function {name:"list_run_worktrees",type:"function"}),
//! (load:Function {name:"load_focus",type:"function"}),(context:Function {name:"resolve_run_context",type:"function"}),(legacy:Function {name:"resolve_worktree_run_context",type:"function"}),
//! (envelope:Function {name:"context_envelope",type:"function"}),(permission:Function {name:"DirectoryGrant::snapshot",type:"function"}),(router:Function {name:"router",type:"function"}),(reply:Function {name:"reply",type:"function"})
//! CREATE (decode)-[:CALLS]->(decode64),(decode)-[:CALLS]->(fromJson),(decode)-[:CALLS]->(bad),(decode)-[:CALLS]->(invalid),(decode)-[:CALLS]->(nil),(next)-[:CALLS]->(encode64),(next)-[:CALLS]->(toJson),(next)-[:CALLS]->(internal),(parse)-[:CALLS]->(parseUuid),(parse)-[:CALLS]->(nil),(parse)-[:CALLS]->(bad),
//! (read)-[:CALLS]->(validate),(read)-[:CALLS]->(scope),(read)-[:CALLS]->(tenant),(read)-[:CALLS]->(begin),(read)-[:CALLS]->(sqlPlain),(read)-[:CALLS]->(execute),(read)-[:CALLS]->(internal),
//! (br)-[:CALLS]->(sqlQuery),(br)-[:CALLS]->(bind),(br)-[:CALLS]->(fetch),(br)-[:CALLS]->(active),(br)-[:CALLS]->(internal),(br)-[:CALLS]->(missing),
//! (ra)-[:CALLS]->(sqlQuery),(ra)-[:CALLS]->(bind),(ra)-[:CALLS]->(fetch),(ra)-[:CALLS]->(internal),(ra)-[:CALLS]->(missing),
//! (branches)-[:CALLS]->(parse),(branches)-[:CALLS]->(active),(branches)-[:CALLS]->(sqlQuery),(branches)-[:CALLS]->(bind),(branches)-[:CALLS]->(fetchAll),(branches)-[:CALLS]->(commit),(branches)-[:CALLS]->(next),(branches)-[:CALLS]->(reply),(branches)-[:CALLS]->(json),(branches)-[:CALLS]->(internal),
//! (runs)-[:CALLS]->(parse),(runs)-[:CALLS]->(sqlQuery),(runs)-[:CALLS]->(bind),(runs)-[:CALLS]->(fetchAll),(runs)-[:CALLS]->(commit),(runs)-[:CALLS]->(next),(runs)-[:CALLS]->(reply),(runs)-[:CALLS]->(json),(runs)-[:CALLS]->(internal),
//! (worktrees)-[:CALLS]->(parse),(worktrees)-[:CALLS]->(scope),(worktrees)-[:CALLS]->(sqlQuery),(worktrees)-[:CALLS]->(bind),(worktrees)-[:CALLS]->(fetchAll),(worktrees)-[:CALLS]->(format),(worktrees)-[:CALLS]->(commit),(worktrees)-[:CALLS]->(next),(worktrees)-[:CALLS]->(reply),(worktrees)-[:CALLS]->(json),(worktrees)-[:CALLS]->(internal),
//! (load)-[:CALLS]->(sqlQuery),(load)-[:CALLS]->(bind),(load)-[:CALLS]->(fetch),(load)-[:CALLS]->(format),(load)-[:CALLS]->(scope),(load)-[:CALLS]->(internal),(load)-[:CALLS]->(missing),
//! (context)-[:CALLS]->(parse),(context)-[:CALLS]->(commit),(context)-[:CALLS]->(reply),(context)-[:CALLS]->(internal),(legacy)-[:CALLS]->(parse),(legacy)-[:CALLS]->(sqlScalar),(legacy)-[:CALLS]->(bind),(legacy)-[:CALLS]->(fetch),(legacy)-[:CALLS]->(scope),(legacy)-[:CALLS]->(commit),(legacy)-[:CALLS]->(reply),(legacy)-[:CALLS]->(internal),(legacy)-[:CALLS]->(missing),
//! (envelope)-[:CALLS]->(format),(envelope)-[:CALLS]->(json),(envelope)-[:CALLS]->(clock),(envelope)-[:CALLS]->(uuid),(permission)-[:CALLS]->(json),(router)-[:CALLS]->(new),(router)-[:CALLS]->(route),(router)-[:CALLS]->(get),(reply)-[:CALLS]->(wrap);
use axum::{
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, VARY},
        HeaderName,
    },
    routing::get,
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, AuthUser, AuthenticatedUser,
    GroupApiError, GroupApiState, ProjectBinding,
};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryQuery {
    limit: Option<i64>,
    cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryCursor {
    kind: String,
    parent_id: Uuid,
    after_id: Uuid,
}

struct DirectoryPage {
    limit: i64,
    after_id: Option<Uuid>,
    kind: &'static str,
    parent_id: Uuid,
}

impl DirectoryQuery {
    fn page(&self, kind: &'static str, parent_id: Uuid) -> Result<DirectoryPage, GroupApiError> {
        let limit = self.limit.unwrap_or(50);
        if !(1..=100).contains(&limit) {
            return Err(GroupApiError::bad_request());
        }
        let after_id = if let Some(encoded) = &self.cursor {
            if encoded.len() > 512 {
                return Err(GroupApiError::bad_request());
            }
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .map_err(|_| GroupApiError::bad_request())?;
            let cursor: DirectoryCursor =
                serde_json::from_slice(&bytes).map_err(|_| GroupApiError::bad_request())?;
            if cursor.kind != kind || cursor.parent_id != parent_id || cursor.after_id.is_nil() {
                return Err(GroupApiError::invalid_request(
                    "directory_cursor_scope_mismatch",
                ));
            }
            Some(cursor.after_id)
        } else {
            None
        };
        Ok(DirectoryPage {
            limit,
            after_id,
            kind,
            parent_id,
        })
    }
}

impl DirectoryPage {
    fn next_cursor(&self, after_id: Option<Uuid>) -> Result<Option<String>, GroupApiError> {
        after_id
            .map(|after_id| {
                let bytes = serde_json::to_vec(&DirectoryCursor {
                    kind: self.kind.to_owned(),
                    parent_id: self.parent_id,
                    after_id,
                })
                .map_err(|_| GroupApiError::internal())?;
                Ok(URL_SAFE_NO_PAD.encode(bytes))
            })
            .transpose()
    }
}

#[derive(FromRow, Serialize)]
struct CloudBranchRow {
    branch_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    name: String,
    full_ref: String,
    head_commit_id: Option<String>,
    state: String,
    version: i32,
}

#[derive(FromRow, Serialize)]
struct EngineeringRunRow {
    engineering_run_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    branch_id: Uuid,
    title: String,
    state: String,
    owner_user_id: Option<Uuid>,
    version: i32,
}

#[derive(FromRow)]
struct DirectoryGrant {
    binding_id: Uuid,
    role: String,
    version: i32,
}

impl DirectoryGrant {
    fn snapshot(&self) -> Value {
        json!({"binding_id": self.binding_id, "role": self.role, "version": self.version})
    }
}

struct RunAuthority {
    branch: CloudBranchRow,
    run: EngineeringRunRow,
    project: ProjectBinding,
    branch_grant: DirectoryGrant,
    run_grant: DirectoryGrant,
}

pub(super) struct RunTaskScope {
    pub(super) engineering_run_id: Uuid,
    pub(super) project_id: Uuid,
    pub(super) repository_id: Uuid,
    pub(super) branch_id: Uuid,
    pub(super) role: String,
    pub(super) permission_snapshot_ref: String,
}

#[derive(FromRow, Serialize)]
struct RunWorktreeRow {
    worktree_id: Uuid,
    engineering_run_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    workspace_id: Uuid,
    name: String,
    branch: String,
    human_state: String,
    machine_state: String,
    agent_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    runtime_id: Option<Uuid>,
    archived: bool,
    version: i32,
    binding_id: Uuid,
    binding_version: i32,
    project_binding_id: Uuid,
    project_binding_version: i32,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextQuery {
    focus_worktree_id: Option<String>,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route("/api/v1/projects/{project_id}/branches", get(list_branches))
        .route(
            "/api/v1/branches/{branch_id}/engineering-runs",
            get(list_runs),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/worktrees",
            get(list_run_worktrees),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/context",
            get(resolve_run_context),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/run-context",
            get(resolve_worktree_run_context),
        )
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    let id = Uuid::parse_str(value).map_err(|_| GroupApiError::bad_request())?;
    if id.is_nil() {
        return Err(GroupApiError::bad_request());
    }
    Ok(id)
}

async fn begin_read(
    state: &GroupApiState,
    actor: &AuthUser,
) -> Result<Transaction<'static, Postgres>, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, "project:read")?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    sqlx::query("SELECT set_config('statement_timeout', '3000', true), set_config('lock_timeout', '1000', true)")
        .execute(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(tx)
}

async fn branch_authority(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    branch_id: Uuid,
) -> Result<(CloudBranchRow, ProjectBinding, DirectoryGrant), GroupApiError> {
    let branch = sqlx::query_as::<_, CloudBranchRow>(
        "SELECT b.branch_id,b.project_id,b.repository_id,v.name,v.full_ref,v.head_commit_id,v.state,v.version
         FROM scm.cloud_branch b JOIN scm.cloud_branch_revision v ON v.tenant_id=b.tenant_id AND v.branch_id=b.branch_id
         WHERE b.tenant_id=$1 AND b.branch_id=$2 AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state='active'
         FOR SHARE OF b,v")
        .bind(actor.tenant_id).bind(branch_id).fetch_optional(&mut **tx).await
        .map_err(|_| GroupApiError::internal())?.ok_or_else(GroupApiError::not_found)?;
    let project = active_binding(tx, actor, branch.project_id).await?;
    let grant = sqlx::query_as::<_, DirectoryGrant>(
        "SELECT binding_id,role,version FROM permission.cloud_branch_role_binding
         WHERE tenant_id=$1 AND branch_id=$2 AND user_id=$3 AND valid_from<=now() AND valid_to IS NULL FOR SHARE")
        .bind(actor.tenant_id).bind(branch_id).bind(actor.user_id).fetch_optional(&mut **tx).await
        .map_err(|_| GroupApiError::internal())?.ok_or_else(GroupApiError::not_found)?;
    Ok((branch, project, grant))
}

async fn run_authority(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    run_id: Uuid,
) -> Result<RunAuthority, GroupApiError> {
    let run = sqlx::query_as::<_, EngineeringRunRow>(
        "SELECT r.engineering_run_id,r.project_id,r.repository_id,r.branch_id,v.title,v.state,v.owner_user_id,v.version
         FROM multica.engineering_run r JOIN multica.engineering_run_revision v
         ON v.tenant_id=r.tenant_id AND v.engineering_run_id=r.engineering_run_id
         WHERE r.tenant_id=$1 AND r.engineering_run_id=$2 AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state<>'archived'
         FOR SHARE OF r,v")
        .bind(actor.tenant_id).bind(run_id).fetch_optional(&mut **tx).await
        .map_err(|_| GroupApiError::internal())?.ok_or_else(GroupApiError::not_found)?;
    let (branch, project, branch_grant) = branch_authority(tx, actor, run.branch_id).await?;
    if branch.project_id != run.project_id || branch.repository_id != run.repository_id {
        return Err(GroupApiError::not_found());
    }
    let run_grant = sqlx::query_as::<_, DirectoryGrant>(
        "SELECT binding_id,role,version FROM permission.engineering_run_role_binding
         WHERE tenant_id=$1 AND engineering_run_id=$2 AND user_id=$3 AND valid_from<=now() AND valid_to IS NULL FOR SHARE")
        .bind(actor.tenant_id).bind(run_id).bind(actor.user_id).fetch_optional(&mut **tx).await
        .map_err(|_| GroupApiError::internal())?.ok_or_else(GroupApiError::not_found)?;
    Ok(RunAuthority {
        branch,
        run,
        project,
        branch_grant,
        run_grant,
    })
}

pub(super) async fn authorize_run_task_scope(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    run_id: Uuid,
) -> Result<RunTaskScope, GroupApiError> {
    let authority = run_authority(tx, actor, run_id).await?;
    Ok(RunTaskScope {
        engineering_run_id: authority.run.engineering_run_id,
        project_id: authority.run.project_id,
        repository_id: authority.run.repository_id,
        branch_id: authority.run.branch_id,
        role: authority.run_grant.role,
        permission_snapshot_ref: format!(
            "{}:v{}",
            authority.run_grant.binding_id, authority.run_grant.version
        ),
    })
}

async fn list_branches(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    Query(query): Query<DirectoryQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    let project_id = parse_id(&project_id)?;
    let page = query.page("branches", project_id)?;
    let mut tx = begin_read(&state, &actor).await?;
    active_binding(&mut tx, &actor, project_id).await?;
    let mut rows = sqlx::query_as::<_, CloudBranchRow>(
        "SELECT b.branch_id,b.project_id,b.repository_id,v.name,v.full_ref,v.head_commit_id,v.state,v.version
         FROM scm.cloud_branch b JOIN scm.cloud_branch_revision v ON v.tenant_id=b.tenant_id AND v.branch_id=b.branch_id
         JOIN permission.cloud_branch_role_binding g ON g.tenant_id=b.tenant_id AND g.branch_id=b.branch_id
         WHERE b.tenant_id=$1 AND b.project_id=$2 AND g.user_id=$3
         AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state='active'
         AND g.valid_from<=now() AND g.valid_to IS NULL AND ($4::uuid IS NULL OR b.branch_id>$4)
         ORDER BY b.branch_id LIMIT $5")
        .bind(actor.tenant_id).bind(project_id).bind(actor.user_id).bind(page.after_id).bind(page.limit+1)
        .fetch_all(&mut *tx).await.map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let more = rows.len() > page.limit as usize;
    if more {
        rows.pop();
    }
    let next = page.next_cursor(if more {
        rows.last().map(|r| r.branch_id)
    } else {
        None
    })?;
    Ok(reply(
        json!({"project_id":project_id,"branches":rows,"limit":page.limit,"next_cursor":next}),
    ))
}

async fn list_runs(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(branch_id): Path<String>,
    Query(query): Query<DirectoryQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    let branch_id = parse_id(&branch_id)?;
    let page = query.page("runs", branch_id)?;
    let mut tx = begin_read(&state, &actor).await?;
    let (branch, _, _) = branch_authority(&mut tx, &actor, branch_id).await?;
    let mut rows = sqlx::query_as::<_, EngineeringRunRow>(
        "SELECT r.engineering_run_id,r.project_id,r.repository_id,r.branch_id,v.title,v.state,v.owner_user_id,v.version
         FROM multica.engineering_run r JOIN multica.engineering_run_revision v ON v.tenant_id=r.tenant_id AND v.engineering_run_id=r.engineering_run_id
         JOIN permission.engineering_run_role_binding g ON g.tenant_id=r.tenant_id AND g.engineering_run_id=r.engineering_run_id
         WHERE r.tenant_id=$1 AND r.branch_id=$2 AND g.user_id=$3
         AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state<>'archived'
         AND g.valid_from<=now() AND g.valid_to IS NULL AND ($4::uuid IS NULL OR r.engineering_run_id>$4)
         ORDER BY r.engineering_run_id LIMIT $5")
        .bind(actor.tenant_id).bind(branch_id).bind(actor.user_id).bind(page.after_id).bind(page.limit+1)
        .fetch_all(&mut *tx).await.map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let more = rows.len() > page.limit as usize;
    if more {
        rows.pop();
    }
    let next = page.next_cursor(if more {
        rows.last().map(|r| r.engineering_run_id)
    } else {
        None
    })?;
    Ok(reply(
        json!({"project_id":branch.project_id,"branch_id":branch_id,"engineering_runs":rows,"limit":page.limit,"next_cursor":next}),
    ))
}

// No filesystem path is projected. The exact historical Project binding FK is revalidated as current.
const WORKTREE_SELECT: &str =
    "SELECT w.id AS worktree_id,b.engineering_run_id,b.project_id,b.repository_id,w.workspace_id,w.name,w.branch,
     w.human_state,w.machine_state,w.agent_id,w.agent_session_id,w.runtime_id,w.archived,w.version,
     b.binding_id,b.version AS binding_version,p.binding_id AS project_binding_id,p.version AS project_binding_version
     FROM multica.engineering_run_worktree_binding b
     JOIN multica.worktree_project_binding p ON p.tenant_id=b.tenant_id AND p.binding_id=b.project_binding_id
     AND p.project_id=b.project_id AND p.worktree_id=b.worktree_id
     JOIN worktree_canvas_worktree w ON w.tenant_id=b.tenant_id AND w.id=b.worktree_id
     AND w.project_id=b.project_id AND w.repo_id=b.repository_id
     WHERE b.tenant_id=$1 AND b.engineering_run_id=$2 AND b.valid_from<=now() AND b.valid_to IS NULL
     AND p.valid_from<=now() AND p.valid_to IS NULL";

async fn list_run_worktrees(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<DirectoryQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    require_scope(&actor, "worktree:read")?;
    let run_id = parse_id(&run_id)?;
    let page = query.page("worktrees", run_id)?;
    let mut tx = begin_read(&state, &actor).await?;
    let authority = run_authority(&mut tx, &actor, run_id).await?;
    let sql = format!("{WORKTREE_SELECT} AND ($3::uuid IS NULL OR w.id>$3) ORDER BY w.id LIMIT $4");
    let mut rows = sqlx::query_as::<_, RunWorktreeRow>(&sql)
        .bind(actor.tenant_id)
        .bind(run_id)
        .bind(page.after_id)
        .bind(page.limit + 1)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let more = rows.len() > page.limit as usize;
    if more {
        rows.pop();
    }
    let next = page.next_cursor(if more {
        rows.last().map(|r| r.worktree_id)
    } else {
        None
    })?;
    Ok(reply(
        json!({"project_id":authority.run.project_id,"branch_id":authority.run.branch_id,
        "engineering_run_id":run_id,"worktrees":rows,"limit":page.limit,"next_cursor":next}),
    ))
}

async fn load_focus(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    run_id: Uuid,
    worktree_id: Uuid,
) -> Result<RunWorktreeRow, GroupApiError> {
    require_scope(actor, "worktree:read")?;
    let sql = format!("{WORKTREE_SELECT} AND w.id=$3 FOR SHARE OF b,p,w");
    sqlx::query_as::<_, RunWorktreeRow>(&sql)
        .bind(actor.tenant_id)
        .bind(run_id)
        .bind(worktree_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?
        .ok_or_else(GroupApiError::not_found)
}

fn context_envelope(
    actor: &AuthUser,
    authority: RunAuthority,
    focus: Option<RunWorktreeRow>,
) -> Value {
    let binding_snapshot = focus.as_ref().map(|row| {
        (
            row.binding_id,
            row.binding_version,
            row.worktree_id,
            row.version,
        )
    });
    let project_binding_snapshot = focus
        .as_ref()
        .map(|row| (row.project_binding_id, row.project_binding_version));
    let context_version = format!(
        "p{}:{}:b{}:{}:r{}:{}:bv{}:rv{}",
        authority.project.id,
        authority.project.version,
        authority.branch_grant.binding_id,
        authority.branch_grant.version,
        authority.run_grant.binding_id,
        authority.run_grant.version,
        authority.branch.version,
        authority.run.version
    );
    let focus_version = focus
        .as_ref()
        .map(|_| format!("w{:?}:wp{:?}", binding_snapshot, project_binding_snapshot));
    json!({
        "run_context": {
            "tenant_id":actor.tenant_id,"project_id":authority.run.project_id,
            "repository_id":authority.run.repository_id,"branch_id":authority.run.branch_id,
            "engineering_run_id":authority.run.engineering_run_id,
            "actor_id":actor.user_id,"actor_kind":"user",
            "authorization": {
                "project":{"binding_id":authority.project.id,"role":authority.project.role,"version":authority.project.version},
                "branch":authority.branch_grant.snapshot(),"engineering_run":authority.run_grant.snapshot(),
            },
            "granted_scopes":actor.scope.split_ascii_whitespace().collect::<Vec<_>>(),
            "context_version":context_version,"resolved_at":Utc::now(),"correlation_id":Uuid::new_v4(),
        },
        "branch":authority.branch,"engineering_run":authority.run,"focus":focus,"focus_version":focus_version,
        "capabilities":{"run_owned_apps_available":false,"execution_admission_available":false},
    })
}

async fn resolve_run_context(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<ContextQuery>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    let run_id = parse_id(&run_id)?;
    let focus_id = query
        .focus_worktree_id
        .as_deref()
        .map(parse_id)
        .transpose()?;
    let mut tx = begin_read(&state, &actor).await?;
    let authority = run_authority(&mut tx, &actor, run_id).await?;
    let focus = if let Some(worktree_id) = focus_id {
        Some(load_focus(&mut tx, &actor, run_id, worktree_id).await?)
    } else {
        None
    };
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(reply(context_envelope(&actor, authority, focus)))
}

/// Compatibility discovery only; it does not redirect or grant old Worktree-owned Apps Run ownership.
async fn resolve_worktree_run_context(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, GroupApiError> {
    require_scope(&actor, "worktree:read")?;
    let worktree_id = parse_id(&worktree_id)?;
    let mut tx = begin_read(&state, &actor).await?;
    let run_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT engineering_run_id FROM multica.engineering_run_worktree_binding
         WHERE tenant_id=$1 AND worktree_id=$2 AND valid_from<=now() AND valid_to IS NULL FOR SHARE")
        .bind(actor.tenant_id).bind(worktree_id).fetch_optional(&mut *tx).await.map_err(|_| GroupApiError::internal())?;
    let run_id = run_id.ok_or_else(GroupApiError::not_found)?;
    let authority = run_authority(&mut tx, &actor, run_id).await?;
    let focus = load_focus(&mut tx, &actor, run_id, worktree_id).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(reply(context_envelope(&actor, authority, Some(focus))))
}

fn reply(value: Value) -> ([(HeaderName, &'static str); 2], Json<Value>) {
    (
        [(CACHE_CONTROL, "no-store"), (VARY, "Authorization")],
        Json(value),
    )
}
