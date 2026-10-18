/*
@cypher schema=1 source_sha256=89c5cf65f74e83d61402e4dd8ca2cb2ed7da445ca763e09c0398f563688e440a
MERGE (self:File {path:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::module",kind:"module"})
MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::router",kind:"function"})
MERGE (bind:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::bind_worktree",kind:"function"})
MERGE (detach:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::detach_worktree",kind:"function"})
MERGE (mutate:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::mutate_binding",kind:"function"})
MERGE (loadWorktree:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::load_project_worktree",kind:"function"})
MERGE (loadCurrent:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::load_current_run_binding",kind:"function"})
MERGE (nextVersion:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::next_binding_version",kind:"function"})
MERGE (closeBinding:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::close_current_binding",kind:"function"})
MERGE (insertBinding:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::insert_current_binding",kind:"function"})
MERGE (authorize:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::authorize_run_manager",kind:"function"})
MERGE (begin:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::begin_write",kind:"function"})
MERGE (transition:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::validate_transition",kind:"function"})
MERGE (validateProjectVersion:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::validate_project_binding_version",kind:"function"})
MERGE (validateTarget:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::validate_target_repository",kind:"function"})
MERGE (requireProjectManager:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::require_project_manager",kind:"function"})
MERGE (parseId:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::parse_id",kind:"function"})
MERGE (validateBody:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::validate_body",kind:"function"})
MERGE (lockIdempotency:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::lock_idempotency_key",kind:"function"})
MERGE (idempotencyLockKey:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::idempotency_lock_key",kind:"function"})
MERGE (replay:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::load_idempotency_replay",kind:"function"})
MERGE (pruneReplay:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::prune_expired_idempotency",kind:"function"})
MERGE (saveReplay:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::save_idempotency_replay",kind:"function"})
MERGE (worktreeLock:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::lock_worktree_binding",kind:"function"})
MERGE (worktreeLockKey:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::worktree_binding_lock_key",kind:"function"})
MERGE (writerContext:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::set_writer_context",kind:"function"})
MERGE (emit:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::append_outbox_event",kind:"function"})
MERGE (responseBody:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::binding_response",kind:"function"})
MERGE (reply:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::private_json",kind:"function"})
MERGE (responseMiddleware:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::private_response_middleware",kind:"function"})
MERGE (errorMap:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::map_binding_write_error",kind:"function"})
MERGE (lockErrorMap:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::map_advisory_lock_error",kind:"function"})
MERGE (lockSqlState:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::map_advisory_lock_sqlstate",kind:"function"})
MERGE (putEndpoint:Endpoint {id:"PUT /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (deleteEndpoint:Endpoint {id:"DELETE /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",kind:"http_route"})
MERGE (request:Type {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::BindingMutationBody",kind:"struct"})
MERGE (expected:Type {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::ExpectedBinding",kind:"enum"})
MERGE (current:Type {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::ActiveRunWorktreeBinding",kind:"struct"})
MERGE (action:Type {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::MutationAction",kind:"enum"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(router)
MERGE (module)-[:DEFINES]->(bind)
MERGE (module)-[:DEFINES]->(detach)
MERGE (module)-[:DEFINES]->(mutate)
MERGE (module)-[:DEFINES]->(authorize)
MERGE (module)-[:DEFINES]->(begin)
MERGE (module)-[:DEFINES]->(transition)
MERGE (module)-[:DEFINES]->(validateBody)
MERGE (module)-[:DEFINES]->(replay)
MERGE (module)-[:DEFINES]->(saveReplay)
MERGE (module)-[:DEFINES]->(emit)
MERGE (module)-[:DEFINES]->(reply)
MERGE (module)-[:DEFINES]->(responseMiddleware)
MERGE (module)-[:DEFINES]->(errorMap)
MERGE (module)-[:DEFINES]->(lockErrorMap)
MERGE (module)-[:DEFINES]->(lockSqlState)
MERGE (module)-[:DEFINES]->(request)
MERGE (module)-[:DEFINES]->(expected)
MERGE (module)-[:DEFINES]->(current)
MERGE (module)-[:DEFINES]->(action)
MERGE (router)-[:CALLS]->(bind)
MERGE (router)-[:CALLS]->(detach)
MERGE (router)-[:CALLS]->(responseMiddleware)
MERGE (bind)-[:CALLS]->(mutate)
MERGE (detach)-[:CALLS]->(mutate)
MERGE (mutate)-[:CALLS]->(begin)
MERGE (mutate)-[:CALLS]->(authorize)
MERGE (mutate)-[:CALLS]->(requireProjectManager)
MERGE (mutate)-[:CALLS]->(parseId)
MERGE (mutate)-[:CALLS]->(validateProjectVersion)
MERGE (mutate)-[:CALLS]->(validateTarget)
MERGE (mutate)-[:CALLS]->(transition)
MERGE (mutate)-[:CALLS]->(validateBody)
MERGE (mutate)-[:CALLS]->(writerContext)
MERGE (mutate)-[:CALLS]->(lockIdempotency)
MERGE (mutate)-[:CALLS]->(replay)
MERGE (mutate)-[:CALLS]->(pruneReplay)
MERGE (mutate)-[:CALLS]->(worktreeLock)
MERGE (mutate)-[:CALLS]->(loadWorktree)
MERGE (mutate)-[:CALLS]->(loadCurrent)
MERGE (mutate)-[:CALLS]->(nextVersion)
MERGE (mutate)-[:CALLS]->(closeBinding)
MERGE (mutate)-[:CALLS]->(insertBinding)
MERGE (mutate)-[:CALLS]->(responseBody)
MERGE (mutate)-[:CALLS]->(saveReplay)
MERGE (mutate)-[:CALLS]->(emit)
MERGE (mutate)-[:CALLS]->(reply)
MERGE (mutate)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::validate_actor",kind:"function"})
MERGE (mutate)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::require_scope",kind:"function"})
MERGE (router)-[:EXPOSES]->(putEndpoint)
MERGE (router)-[:EXPOSES]->(deleteEndpoint)
MERGE (lockIdempotency)-[:CALLS]->(idempotencyLockKey)
MERGE (lockIdempotency)-[:CALLS]->(lockErrorMap)
MERGE (worktreeLock)-[:CALLS]->(worktreeLockKey)
MERGE (worktreeLock)-[:CALLS]->(lockErrorMap)
MERGE (lockErrorMap)-[:CALLS]->(lockSqlState)
MERGE (closeBinding)-[:CALLS]->(errorMap)
MERGE (insertBinding)-[:CALLS]->(errorMap)
MERGE (responseMiddleware)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/run_worktree_bindings.rs::private_response",kind:"function"})
MERGE (mutate)-[:USES_TYPE]->(request)
MERGE (request)-[:USES_TYPE]->(expected)
MERGE (transition)-[:USES_TYPE]->(expected)
MERGE (transition)-[:USES_TYPE]->(current)
MERGE (transition)-[:USES_TYPE]->(action)
MERGE (mutate)-[:USES_TYPE]->(current)
MERGE (mutate)-[:USES_TYPE]->(action)
MERGE (mutate)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::authorize_run_owned_task_scope",kind:"function"})
MERGE (mutate)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::idempotency_key",kind:"function"})
MERGE (mutate)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/work_items.rs::request_hash",kind:"function"})
MERGE (mutate)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::active_binding",kind:"function"})
MERGE (begin)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api.rs::set_tenant",kind:"function"})
MERGE (authorize)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::authorize_run_owned_task_scope",kind:"function"})
MERGE (loadWorktree)-[:READS]->(:Table {id:"multica.worktree_project_binding",kind:"table"})
MERGE (loadWorktree)-[:READS]->(:Table {id:"worktree_canvas_worktree",kind:"table"})
MERGE (loadCurrent)-[:READS]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
MERGE (nextVersion)-[:READS]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
MERGE (closeBinding)-[:WRITES]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
MERGE (insertBinding)-[:WRITES]->(:Table {id:"multica.engineering_run_worktree_binding",kind:"table"})
MERGE (replay)-[:READS]->(:Table {id:"multica.engineering_run_command_idempotency",kind:"table"})
MERGE (pruneReplay)-[:WRITES]->(:Table {id:"multica.engineering_run_command_idempotency",kind:"table"})
MERGE (saveReplay)-[:WRITES]->(:Table {id:"multica.engineering_run_command_idempotency",kind:"table"})
MERGE (emit)-[:WRITES]->(:Table {id:"multica.engineering_run_outbox",kind:"table"})
@endcypher
*/

//! Authenticated, Project-scoped SCD2 mutations for Run-to-Worktree ownership.
//! Remote repository and Branch values are optional Run metadata; the local
//! Worktree checkout remains the managed object and execution admission stays off.

use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::put,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{
    active_binding,
    engineering_runs::{authorize_run_owned_task_scope, RunOwnedTaskScope},
    require_scope, set_tenant, validate_actor,
    work_items::{idempotency_key, request_hash},
    AuthUser, AuthenticatedUser, GroupApiError, GroupApiState, ProjectBinding,
};

const MAX_WRITE_BODY_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
enum ExpectedBinding {
    Unassigned,
    Bound { binding_id: Uuid, version: i32 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BindingMutationBody {
    expected_binding: ExpectedBinding,
    expected_project_binding_version: i32,
    correlation_id: Uuid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MutationKind {
    Put,
    Detach,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MutationAction {
    Bind,
    Reassign,
    Detach,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
struct BindingSnapshot {
    binding_id: Uuid,
    engineering_run_id: Uuid,
    version: i32,
}

#[derive(Clone, Debug, FromRow)]
struct WorktreeProjectState {
    project_binding_id: Uuid,
    project_binding_version: i32,
    repository_id: Uuid,
    local_branch: String,
    archived: bool,
}

#[derive(Clone, Debug, FromRow)]
struct ActiveRunWorktreeBinding {
    binding_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    engineering_run_id: Uuid,
    version: i32,
}

impl ActiveRunWorktreeBinding {
    fn snapshot(&self) -> BindingSnapshot {
        BindingSnapshot {
            binding_id: self.binding_id,
            engineering_run_id: self.engineering_run_id,
            version: self.version,
        }
    }
}

#[derive(FromRow)]
struct RunCommandReplay {
    request_hash: Vec<u8>,
    response_body: Value,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}",
            put(bind_worktree).delete(detach_worktree),
        )
        .layer(DefaultBodyLimit::max(MAX_WRITE_BODY_BYTES))
        .route_layer(map_response(private_response_middleware))
}

async fn bind_worktree(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project_id, run_id, worktree_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<BindingMutationBody>,
) -> Result<Response, GroupApiError> {
    mutate_binding(
        state,
        actor,
        project_id,
        run_id,
        worktree_id,
        headers,
        body,
        MutationKind::Put,
    )
    .await
}

async fn detach_worktree(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project_id, run_id, worktree_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(body): Json<BindingMutationBody>,
) -> Result<Response, GroupApiError> {
    mutate_binding(
        state,
        actor,
        project_id,
        run_id,
        worktree_id,
        headers,
        body,
        MutationKind::Detach,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn mutate_binding(
    state: GroupApiState,
    actor: AuthUser,
    project_id: String,
    run_id: String,
    worktree_id: String,
    headers: HeaderMap,
    body: BindingMutationBody,
    kind: MutationKind,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "project:write")?;
    require_scope(&actor, "worktree:manage")?;
    let project_id = parse_id(&project_id)?;
    let run_id = parse_id(&run_id)?;
    let worktree_id = parse_id(&worktree_id)?;
    validate_body(&body, kind)?;

    let key = idempotency_key(&headers)?;
    let operation = match kind {
        MutationKind::Put => "put-run-worktree-binding",
        MutationKind::Detach => "detach-run-worktree-binding",
    };
    let request_operation = format!("{operation}:{project_id}:{run_id}:{worktree_id}");
    let request_digest = request_hash(&request_operation, &body)?;

    let mut tx = begin_write(&state, &actor).await?;
    let project = active_binding(&mut tx, &actor, project_id).await?;
    require_project_manager(&project)?;
    set_writer_context(&mut tx, actor.user_id, body.correlation_id).await?;
    lock_idempotency_key(&mut tx, &actor, &key).await?;

    if let Some(replay) = load_idempotency_replay(&mut tx, &actor, &key).await? {
        if replay.request_hash != request_digest {
            return Err(GroupApiError::conflict("idempotency_key_reused"));
        }
        authorize_run_manager(&mut tx, &actor, project_id, run_id).await?;
        tx.commit().await.map_err(|_| GroupApiError::internal())?;
        return Ok(private_json(StatusCode::OK, replay.response_body));
    }
    prune_expired_idempotency(&mut tx, &actor, &key).await?;

    lock_worktree_binding(&mut tx, actor.tenant_id, worktree_id).await?;
    let worktree = load_project_worktree(&mut tx, actor.tenant_id, project_id, worktree_id)
        .await?
        .ok_or_else(GroupApiError::not_found)?;
    validate_project_binding_version(
        body.expected_project_binding_version,
        worktree.project_binding_version,
    )?;
    if worktree.archived {
        return Err(GroupApiError::conflict("worktree_archived"));
    }

    let current = load_current_run_binding(&mut tx, actor.tenant_id, worktree_id).await?;
    if current.as_ref().is_some_and(|binding| {
        binding.project_id != project_id || binding.repository_id != worktree.repository_id
    }) {
        return Err(GroupApiError::conflict("current_binding_context_mismatch"));
    }
    let current_snapshot = current.as_ref().map(ActiveRunWorktreeBinding::snapshot);
    let action = validate_transition(
        kind,
        run_id,
        &body.expected_binding,
        current_snapshot.as_ref(),
    )?;

    let mut run_ids = vec![run_id];
    if let Some(binding) = current.as_ref() {
        if !run_ids.contains(&binding.engineering_run_id) {
            run_ids.push(binding.engineering_run_id);
        }
    }
    run_ids.sort_unstable();
    let mut target_scope: Option<RunOwnedTaskScope> = None;
    for authorized_run_id in run_ids {
        let scope = authorize_run_manager(&mut tx, &actor, project_id, authorized_run_id).await?;
        if authorized_run_id == run_id {
            target_scope = Some(scope);
        }
    }
    let target_scope = target_scope.ok_or_else(GroupApiError::not_found)?;
    validate_target_repository(&target_scope, worktree.repository_id)?;

    let next_version = match action {
        MutationAction::Bind | MutationAction::Reassign => {
            Some(next_binding_version(&mut tx, actor.tenant_id, worktree_id).await?)
        }
        MutationAction::Detach => None,
    };

    if let Some(previous) = current.as_ref() {
        close_current_binding(&mut tx, actor.tenant_id, worktree_id, previous).await?;
    }

    let new_binding = match action {
        MutationAction::Bind | MutationAction::Reassign => Some(
            insert_current_binding(
                &mut tx,
                &actor,
                project_id,
                run_id,
                worktree_id,
                &worktree,
                target_scope.branch_id,
                next_version.expect("binding transitions have a successor version"),
            )
            .await?,
        ),
        MutationAction::Detach => None,
    };

    let outbox_run_id = match action {
        MutationAction::Detach => current
            .as_ref()
            .map(|binding| binding.engineering_run_id)
            .ok_or_else(|| GroupApiError::conflict("stale_worktree_run_binding"))?,
        MutationAction::Bind | MutationAction::Reassign => run_id,
    };
    let response = binding_response(
        action,
        project_id,
        run_id,
        worktree_id,
        &worktree,
        current.as_ref(),
        new_binding.as_ref(),
    );
    append_outbox_event(
        &mut tx,
        &actor,
        project_id,
        outbox_run_id,
        body.correlation_id,
        action,
        worktree_id,
        current.as_ref(),
        new_binding.as_ref(),
        worktree.project_binding_version,
    )
    .await?;
    save_idempotency_replay(&mut tx, &actor, &key, &request_digest, &response).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(private_json(StatusCode::OK, response))
}

async fn begin_write(
    state: &GroupApiState,
    actor: &AuthUser,
) -> Result<Transaction<'static, Postgres>, GroupApiError> {
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    sqlx::query(
        "SELECT set_config('statement_timeout','5000',true), set_config('lock_timeout','1000',true)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(tx)
}

fn validate_body(body: &BindingMutationBody, kind: MutationKind) -> Result<(), GroupApiError> {
    if body.correlation_id.is_nil() || body.expected_project_binding_version <= 0 {
        return Err(GroupApiError::invalid_request(
            "invalid_run_worktree_binding",
        ));
    }
    match (&body.expected_binding, kind) {
        (ExpectedBinding::Unassigned, MutationKind::Put) => Ok(()),
        (
            ExpectedBinding::Bound {
                binding_id,
                version,
            },
            _,
        ) if !binding_id.is_nil() && *version > 0 => Ok(()),
        (ExpectedBinding::Unassigned, MutationKind::Detach) => Err(GroupApiError::invalid_request(
            "detach_requires_bound_state",
        )),
        _ => Err(GroupApiError::invalid_request("invalid_expected_binding")),
    }
}

fn validate_project_binding_version(expected: i32, actual: i32) -> Result<(), GroupApiError> {
    if expected == actual {
        Ok(())
    } else {
        Err(GroupApiError::conflict("stale_project_binding"))
    }
}

fn validate_transition(
    kind: MutationKind,
    route_run_id: Uuid,
    expected: &ExpectedBinding,
    current: Option<&BindingSnapshot>,
) -> Result<MutationAction, GroupApiError> {
    match (kind, expected, current) {
        (MutationKind::Put, ExpectedBinding::Unassigned, None) => Ok(MutationAction::Bind),
        (MutationKind::Put, ExpectedBinding::Unassigned, Some(_)) => {
            Err(GroupApiError::conflict("stale_worktree_run_binding"))
        }
        (
            MutationKind::Put,
            ExpectedBinding::Bound {
                binding_id,
                version,
            },
            Some(current),
        ) if current.binding_id == *binding_id && current.version == *version => {
            if current.engineering_run_id == route_run_id {
                Err(GroupApiError::conflict("worktree_already_bound_to_run"))
            } else {
                Ok(MutationAction::Reassign)
            }
        }
        (MutationKind::Put, ExpectedBinding::Bound { .. }, _) => {
            Err(GroupApiError::conflict("stale_worktree_run_binding"))
        }
        (MutationKind::Detach, ExpectedBinding::Unassigned, _) => Err(
            GroupApiError::invalid_request("detach_requires_bound_state"),
        ),
        (MutationKind::Detach, ExpectedBinding::Bound { .. }, None) => {
            Err(GroupApiError::conflict("stale_worktree_run_binding"))
        }
        (
            MutationKind::Detach,
            ExpectedBinding::Bound {
                binding_id,
                version,
            },
            Some(current),
        ) if current.binding_id == *binding_id
            && current.version == *version
            && current.engineering_run_id == route_run_id =>
        {
            Ok(MutationAction::Detach)
        }
        (
            MutationKind::Detach,
            ExpectedBinding::Bound {
                binding_id,
                version,
            },
            Some(current),
        ) if current.binding_id == *binding_id
            && current.version == *version
            && current.engineering_run_id != route_run_id =>
        {
            Err(GroupApiError::not_found())
        }
        (MutationKind::Detach, ExpectedBinding::Bound { .. }, Some(_)) => {
            Err(GroupApiError::conflict("stale_worktree_run_binding"))
        }
    }
}

fn validate_target_repository(
    target: &RunOwnedTaskScope,
    worktree_repository_id: Uuid,
) -> Result<(), GroupApiError> {
    if target
        .repository_id
        .is_some_and(|repository_id| repository_id != worktree_repository_id)
        || (target.branch_id.is_some() && target.repository_id.is_none())
    {
        return Err(GroupApiError::conflict("worktree_repository_mismatch"));
    }
    Ok(())
}

fn require_project_manager(project: &ProjectBinding) -> Result<(), GroupApiError> {
    if matches!(project.role.as_str(), "tenant_admin" | "project_admin") {
        Ok(())
    } else {
        Err(GroupApiError::not_found())
    }
}

async fn authorize_run_manager(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    run_id: Uuid,
) -> Result<RunOwnedTaskScope, GroupApiError> {
    let scope = authorize_run_owned_task_scope(tx, actor, run_id).await?;
    if scope.project_id != project_id
        || !matches!(scope.role.as_str(), "tenant_admin" | "project_admin")
    {
        return Err(GroupApiError::not_found());
    }
    Ok(scope)
}

async fn set_writer_context(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), GroupApiError> {
    sqlx::query(
        "SELECT set_config('app.actor_id',$1,true), set_config('app.correlation_id',$2,true)",
    )
    .bind(actor_id.to_string())
    .bind(correlation_id.to_string())
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn lock_idempotency_key(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    key: &str,
) -> Result<(), GroupApiError> {
    let lock_key = idempotency_lock_key(actor.tenant_id, actor.user_id, key);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await
        .map_err(|error| map_advisory_lock_error(error, "idempotency_lock_busy"))?;
    Ok(())
}

fn idempotency_lock_key(tenant_id: Uuid, actor_id: Uuid, key: &str) -> String {
    format!("{tenant_id}:{actor_id}:{key}")
}

async fn load_idempotency_replay(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    key: &str,
) -> Result<Option<RunCommandReplay>, GroupApiError> {
    sqlx::query_as::<_, RunCommandReplay>(
        "SELECT request_hash,response_body FROM multica.engineering_run_command_idempotency
         WHERE tenant_id=$1 AND actor_id=$2 AND idempotency_key=$3 AND expires_at>now() FOR UPDATE",
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

async fn prune_expired_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    key: &str,
) -> Result<(), GroupApiError> {
    sqlx::query(
        "DELETE FROM multica.engineering_run_command_idempotency
         WHERE tenant_id=$1 AND actor_id=$2 AND idempotency_key=$3 AND expires_at<=now()",
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(key)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        "WITH expired AS (
            SELECT tenant_id,actor_id,idempotency_key
            FROM multica.engineering_run_command_idempotency
            WHERE tenant_id=$1 AND actor_id=$2 AND expires_at<=now()
            ORDER BY expires_at,idempotency_key
            LIMIT 32 FOR UPDATE SKIP LOCKED
         )
         DELETE FROM multica.engineering_run_command_idempotency target
         USING expired
         WHERE target.tenant_id=expired.tenant_id AND target.actor_id=expired.actor_id
           AND target.idempotency_key=expired.idempotency_key",
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn save_idempotency_replay(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    key: &str,
    request_digest: &[u8],
    response: &Value,
) -> Result<(), GroupApiError> {
    sqlx::query(
        "INSERT INTO multica.engineering_run_command_idempotency
         (tenant_id,actor_id,idempotency_key,request_hash,response_body,expires_at)
         VALUES ($1,$2,$3,$4,$5,now()+INTERVAL '24 hours')",
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(key)
    .bind(request_digest)
    .bind(response)
    .execute(&mut **tx)
    .await
    .map_err(|error| match &error {
        sqlx::Error::Database(database_error)
            if database_error.code().as_deref() == Some("23505") =>
        {
            GroupApiError::conflict("idempotency_race")
        }
        _ => GroupApiError::internal(),
    })?;
    Ok(())
}

async fn lock_worktree_binding(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
) -> Result<(), GroupApiError> {
    let lock_key = worktree_binding_lock_key(tenant_id, worktree_id);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await
        .map_err(|error| map_advisory_lock_error(error, "worktree_binding_busy"))?;
    Ok(())
}

fn map_advisory_lock_error(error: sqlx::Error, conflict_code: &'static str) -> GroupApiError {
    match error {
        sqlx::Error::Database(database_error) => {
            map_advisory_lock_sqlstate(database_error.code().as_deref(), conflict_code)
        }
        _ => GroupApiError::internal(),
    }
}

fn map_advisory_lock_sqlstate(
    sqlstate: Option<&str>,
    conflict_code: &'static str,
) -> GroupApiError {
    if matches!(sqlstate, Some("55P03" | "57014")) {
        GroupApiError::conflict(conflict_code)
    } else {
        GroupApiError::internal()
    }
}

fn worktree_binding_lock_key(tenant_id: Uuid, worktree_id: Uuid) -> String {
    format!("engineering-run-worktree:{tenant_id}:{worktree_id}")
}

async fn load_project_worktree(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
) -> Result<Option<WorktreeProjectState>, GroupApiError> {
    sqlx::query_as::<_, WorktreeProjectState>(
        "SELECT p.binding_id AS project_binding_id,p.version AS project_binding_version,
                w.repo_id AS repository_id,w.branch AS local_branch,w.archived
         FROM multica.worktree_project_binding p
         JOIN worktree_canvas_worktree w ON w.tenant_id=p.tenant_id AND w.id=p.worktree_id
         WHERE p.tenant_id=$1 AND p.project_id=$2 AND p.worktree_id=$3
           AND p.valid_from<=now() AND p.valid_to IS NULL
           AND w.project_id=p.project_id
         FOR SHARE OF p,w",
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(worktree_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

async fn load_current_run_binding(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
) -> Result<Option<ActiveRunWorktreeBinding>, GroupApiError> {
    sqlx::query_as::<_, ActiveRunWorktreeBinding>(
        "SELECT binding_id,project_id,repository_id,engineering_run_id,version
         FROM multica.engineering_run_worktree_binding
         WHERE tenant_id=$1 AND worktree_id=$2 AND valid_from<=now() AND valid_to IS NULL
         FOR UPDATE",
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

async fn next_binding_version(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
) -> Result<i32, GroupApiError> {
    let current = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(version) FROM multica.engineering_run_worktree_binding
         WHERE tenant_id=$1 AND worktree_id=$2",
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .unwrap_or(0);
    current
        .checked_add(1)
        .filter(|version| *version > 0)
        .ok_or_else(|| GroupApiError::conflict("binding_version_exhausted"))
}

async fn close_current_binding(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    worktree_id: Uuid,
    current: &ActiveRunWorktreeBinding,
) -> Result<(), GroupApiError> {
    let result = sqlx::query(
        "UPDATE multica.engineering_run_worktree_binding
         SET valid_to=now()
         WHERE tenant_id=$1 AND worktree_id=$2 AND binding_id=$3
           AND version=$4 AND valid_to IS NULL",
    )
    .bind(tenant_id)
    .bind(worktree_id)
    .bind(current.binding_id)
    .bind(current.version)
    .execute(&mut **tx)
    .await
    .map_err(map_binding_write_error)?;
    if result.rows_affected() != 1 {
        return Err(GroupApiError::conflict("stale_worktree_run_binding"));
    }
    Ok(())
}

async fn insert_current_binding(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    run_id: Uuid,
    worktree_id: Uuid,
    worktree: &WorktreeProjectState,
    run_branch_id: Option<Uuid>,
    version: i32,
) -> Result<BindingSnapshot, GroupApiError> {
    let row = sqlx::query_as::<_, BindingSnapshot>(
        "INSERT INTO multica.engineering_run_worktree_binding
         (tenant_id,project_id,repository_id,branch_id,engineering_run_id,worktree_id,
          project_binding_id,assigned_by,version)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         RETURNING binding_id,engineering_run_id,version",
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(worktree.repository_id)
    .bind(run_branch_id)
    .bind(run_id)
    .bind(worktree_id)
    .bind(worktree.project_binding_id)
    .bind(actor.user_id)
    .bind(version)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_binding_write_error)?;
    Ok(row)
}

async fn append_outbox_event(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    project_id: Uuid,
    aggregate_run_id: Uuid,
    correlation_id: Uuid,
    action: MutationAction,
    worktree_id: Uuid,
    previous: Option<&ActiveRunWorktreeBinding>,
    current: Option<&BindingSnapshot>,
    project_binding_version: i32,
) -> Result<(), GroupApiError> {
    let (event_type, operation) = match action {
        MutationAction::Bind => ("engineering_run.worktree_bound", "bound"),
        MutationAction::Reassign => ("engineering_run.worktree_reassigned", "reassigned"),
        MutationAction::Detach => ("engineering_run.worktree_detached", "detached"),
    };
    let payload = json!({
        "schema_version": 1,
        "operation": operation,
        "project_id": project_id,
        "engineering_run_id": aggregate_run_id,
        "worktree_id": worktree_id,
        "actor_id": actor.user_id,
        "project_binding_version": project_binding_version,
        "previous_binding": previous.map(|binding| json!({
            "binding_id": binding.binding_id,
            "engineering_run_id": binding.engineering_run_id,
            "version": binding.version
        })),
        "current_binding": current.map(|binding| json!({
            "binding_id": binding.binding_id,
            "engineering_run_id": binding.engineering_run_id,
            "version": binding.version
        })),
        "execution_admission_available": false
    });
    sqlx::query(
        "INSERT INTO multica.engineering_run_outbox
         (tenant_id,project_id,engineering_run_id,event_type,schema_version,correlation_id,payload)
         VALUES ($1,$2,$3,$4,1,$5,$6)",
    )
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(aggregate_run_id)
    .bind(event_type)
    .bind(correlation_id)
    .bind(payload)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

fn binding_response(
    action: MutationAction,
    project_id: Uuid,
    route_run_id: Uuid,
    worktree_id: Uuid,
    worktree: &WorktreeProjectState,
    previous: Option<&ActiveRunWorktreeBinding>,
    current: Option<&BindingSnapshot>,
) -> Value {
    let state = match action {
        MutationAction::Bind => "bound",
        MutationAction::Reassign => "reassigned",
        MutationAction::Detach => "unassigned",
    };
    json!({
        "project_id": project_id,
        "engineering_run_id": route_run_id,
        "worktree": {
            "worktree_id": worktree_id,
            "local_branch": worktree.local_branch,
            "project_binding_id": worktree.project_binding_id,
            "project_binding_version": worktree.project_binding_version,
            "binding_state": state,
            "previous_binding_id": previous.map(|binding| binding.binding_id),
            "binding_id": current.map(|binding| binding.binding_id),
            "binding_version": current.map(|binding| binding.version)
        },
        "execution_admission_available": false
    })
}

fn map_binding_write_error(error: sqlx::Error) -> GroupApiError {
    if let sqlx::Error::Database(database_error) = &error {
        if matches!(
            database_error.code().as_deref(),
            Some("23505" | "23503" | "23514")
        ) {
            return GroupApiError::conflict("run_worktree_binding_rejected");
        }
        if matches!(database_error.code().as_deref(), Some("55P03" | "57014")) {
            return GroupApiError::conflict("worktree_binding_busy");
        }
    }
    GroupApiError::internal()
}

fn private_json(status: StatusCode, value: Value) -> Response {
    (status, Json(value)).into_response()
}

fn private_response(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Authorization"));
    response
}

async fn private_response_middleware(response: Response) -> Response {
    private_response(response)
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value)
        .map_err(|_| GroupApiError::bad_request())
        .and_then(|id| {
            if id.is_nil() {
                Err(GroupApiError::bad_request())
            } else {
                Ok(id)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(run_id: Uuid, version: i32) -> BindingSnapshot {
        BindingSnapshot {
            binding_id: Uuid::from_u128(0x1000 + version as u128),
            engineering_run_id: run_id,
            version,
        }
    }

    fn body(expected_binding: ExpectedBinding, project_version: i32) -> BindingMutationBody {
        BindingMutationBody {
            expected_binding,
            expected_project_binding_version: project_version,
            correlation_id: Uuid::from_u128(0x9000),
        }
    }

    #[test]
    fn advisory_lock_timeouts_are_conflicts_but_other_errors_stay_internal() {
        for sqlstate in [Some("55P03"), Some("57014")] {
            let mapped = map_advisory_lock_sqlstate(sqlstate, "worktree_binding_busy");
            assert_eq!(mapped.status, StatusCode::CONFLICT);
            assert_eq!(mapped.code, "worktree_binding_busy");

            let idempotency_mapped = map_advisory_lock_sqlstate(sqlstate, "idempotency_lock_busy");
            assert_eq!(idempotency_mapped.status, StatusCode::CONFLICT);
            assert_eq!(idempotency_mapped.code, "idempotency_lock_busy");
        }

        for sqlstate in [None, Some("23505")] {
            let mapped = map_advisory_lock_sqlstate(sqlstate, "worktree_binding_busy");
            assert_eq!(mapped.status, StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(mapped.code, "internal_error");
        }
    }

    #[test]
    fn first_bind_requires_an_unassigned_current_state() {
        let target = Uuid::from_u128(1);
        assert_eq!(
            validate_transition(
                MutationKind::Put,
                target,
                &ExpectedBinding::Unassigned,
                None,
            )
            .unwrap(),
            MutationAction::Bind
        );
        assert!(validate_transition(
            MutationKind::Put,
            target,
            &ExpectedBinding::Unassigned,
            Some(&snapshot(Uuid::from_u128(2), 1)),
        )
        .is_err());
    }

    #[test]
    fn rebind_requires_exact_binding_identity_and_a_different_run() {
        let source_run = Uuid::from_u128(2);
        let target_run = Uuid::from_u128(3);
        let current = snapshot(source_run, 4);
        let expected = ExpectedBinding::Bound {
            binding_id: current.binding_id,
            version: current.version,
        };
        assert_eq!(
            validate_transition(MutationKind::Put, target_run, &expected, Some(&current),).unwrap(),
            MutationAction::Reassign
        );
        assert!(
            validate_transition(MutationKind::Put, source_run, &expected, Some(&current),).is_err()
        );
        assert!(validate_transition(
            MutationKind::Put,
            target_run,
            &ExpectedBinding::Bound {
                binding_id: current.binding_id,
                version: current.version + 1,
            },
            Some(&current),
        )
        .is_err());
    }

    #[test]
    fn detach_requires_the_current_run_and_exact_scd2_version() {
        let run_id = Uuid::from_u128(4);
        let current = snapshot(run_id, 2);
        let expected = ExpectedBinding::Bound {
            binding_id: current.binding_id,
            version: current.version,
        };
        assert_eq!(
            validate_transition(MutationKind::Detach, run_id, &expected, Some(&current),).unwrap(),
            MutationAction::Detach
        );
        assert!(validate_transition(
            MutationKind::Detach,
            Uuid::from_u128(5),
            &expected,
            Some(&current),
        )
        .is_err());
        assert!(validate_transition(
            MutationKind::Detach,
            run_id,
            &ExpectedBinding::Unassigned,
            None,
        )
        .is_err());
    }

    #[test]
    fn project_binding_version_and_hostless_run_rules_are_explicit() {
        assert!(validate_project_binding_version(3, 3).is_ok());
        assert!(validate_project_binding_version(2, 3).is_err());
        let hostless = RunOwnedTaskScope {
            engineering_run_id: Uuid::from_u128(6),
            project_id: Uuid::from_u128(7),
            repository_id: None,
            branch_id: None,
            role: "project_admin".to_owned(),
            permission_snapshot_ref: "grant:v1".to_owned(),
        };
        assert!(validate_target_repository(&hostless, Uuid::from_u128(8)).is_ok());
        let mismatched = RunOwnedTaskScope {
            repository_id: Some(Uuid::from_u128(9)),
            ..hostless
        };
        assert!(validate_target_repository(&mismatched, Uuid::from_u128(8)).is_err());
    }

    #[test]
    fn idempotency_digest_and_worktree_lock_are_scoped() {
        let request_a = body(ExpectedBinding::Unassigned, 1);
        let request_b = body(
            ExpectedBinding::Bound {
                binding_id: Uuid::from_u128(10),
                version: 1,
            },
            1,
        );
        let digest_a = request_hash("bind:project:run:worktree", &request_a).unwrap();
        let digest_b = request_hash("bind:project:run:worktree", &request_b).unwrap();
        assert_ne!(digest_a, digest_b);

        let tenant = Uuid::from_u128(11);
        let worktree = Uuid::from_u128(12);
        assert_eq!(
            worktree_binding_lock_key(tenant, worktree),
            worktree_binding_lock_key(tenant, worktree)
        );
        assert_ne!(
            worktree_binding_lock_key(tenant, worktree),
            worktree_binding_lock_key(tenant, Uuid::from_u128(13))
        );
        assert_ne!(
            worktree_binding_lock_key(tenant, worktree),
            worktree_binding_lock_key(Uuid::from_u128(14), worktree)
        );
        assert_eq!(
            idempotency_lock_key(tenant, Uuid::from_u128(15), "same-command"),
            idempotency_lock_key(tenant, Uuid::from_u128(15), "same-command")
        );
        assert_ne!(
            idempotency_lock_key(tenant, Uuid::from_u128(15), "same-command"),
            idempotency_lock_key(tenant, Uuid::from_u128(16), "same-command")
        );
        assert_ne!(
            idempotency_lock_key(tenant, Uuid::from_u128(15), "same-command"),
            idempotency_lock_key(tenant, Uuid::from_u128(15), "other-command")
        );
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL connection for advisory-lock concurrency"]
    async fn postgres_advisory_lock_serializes_commands_for_one_worktree() {
        use sqlx::postgres::PgPoolOptions;

        let database_url = std::env::var("STAR_WLD0_BINDING_TEST_DATABASE_URL")
            .expect("set a disposable PostgreSQL URL to run this ignored boundary test");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("connect to the explicitly supplied disposable PostgreSQL database");
        let tenant_id = Uuid::from_u128(0x101);
        let worktree_id = Uuid::from_u128(0x202);
        let other_worktree_id = Uuid::from_u128(0x303);
        let mut first = pool.begin().await.expect("start first lock transaction");
        lock_worktree_binding(&mut first, tenant_id, worktree_id)
            .await
            .expect("acquire first Worktree lock");

        let mut second = pool.begin().await.expect("start second lock transaction");
        let same_worktree_acquired = sqlx::query_scalar::<_, bool>(
            "SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))",
        )
        .bind(worktree_binding_lock_key(tenant_id, worktree_id))
        .fetch_one(&mut *second)
        .await
        .expect("probe the lock held by the first transaction");
        assert!(!same_worktree_acquired);

        let other_worktree_acquired = sqlx::query_scalar::<_, bool>(
            "SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))",
        )
        .bind(worktree_binding_lock_key(tenant_id, other_worktree_id))
        .fetch_one(&mut *second)
        .await
        .expect("probe an independent Worktree lock");
        assert!(other_worktree_acquired);

        first
            .commit()
            .await
            .expect("release the first Worktree lock");
        let retry_acquired = sqlx::query_scalar::<_, bool>(
            "SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))",
        )
        .bind(worktree_binding_lock_key(tenant_id, worktree_id))
        .fetch_one(&mut *second)
        .await
        .expect("retry after the first transaction commits");
        assert!(retry_acquired);
        second
            .commit()
            .await
            .expect("release the second lock transaction");
        pool.close().await;
    }
}
