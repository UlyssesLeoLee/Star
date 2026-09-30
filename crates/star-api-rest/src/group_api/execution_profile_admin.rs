//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"execution_profile_admin.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"execution_profile_admin",type:"module",language:"rust"}),
//!   (router:Function {name:"router",type:"function",language:"rust",visibility:"pub(super)"}),
//!   (project_handler:Function {name:"project_profile_lifecycle",type:"function",language:"rust",visibility:"private"}),
//!   (worktree_handler:Function {name:"worktree_profile_lifecycle",type:"function",language:"rust",visibility:"private"}),
//!   (mutate:Function {name:"mutate_profile",type:"function",language:"rust",visibility:"private"}),
//!   (authorize:Function {name:"authorize_profile_target",type:"function",language:"rust",visibility:"private"}),
//!   (lock:Function {name:"lock_profile_id",type:"function",language:"rust",visibility:"private"}),
//!   (load_current:Function {name:"load_current_profile",type:"function",language:"rust",visibility:"private"}),
//!   (load_historical:Function {name:"load_historical_profile",type:"function",language:"rust",visibility:"private"}),
//!   (verify_document:Function {name:"verify_profile_document",type:"function",language:"rust",visibility:"private"}),
//!   (verify_row:Function {name:"verify_profile_revision",type:"function",language:"rust",visibility:"private"}),
//!   (close_current:Function {name:"close_current_revision",type:"function",language:"rust",visibility:"private"}),
//!   (insert_revision:Function {name:"insert_profile_revision",type:"function",language:"rust",visibility:"private"}),
//!   (insert_audit:Function {name:"insert_profile_audit",type:"function",language:"rust",visibility:"private"}),
//!   (plan:Function {name:"plan_profile_transition",type:"function",language:"rust",visibility:"private"}),
//!   (parse_id:Function {name:"parse_id",type:"function",language:"rust",visibility:"private"}),
//!   (no_store:Function {name:"no_store",type:"function",language:"rust",visibility:"private"}),
//!   (no_store_response:Function {name:"no_store_response",type:"function",language:"rust",visibility:"private"}),
//!   (target_scope_kind:Function {name:"ProfileScopeTarget::scope_kind",type:"function",language:"rust",visibility:"private"}),
//!   (target:Class {name:"ProfileScopeTarget",type:"class",language:"rust"}),
//!   (resolved_target:Class {name:"ResolvedProfileScope",type:"class",language:"rust"}),
//!   (request:Class {name:"ProfileMutationBody",type:"class",language:"rust"}),
//!   (action:Enum {name:"ProfileMutationAction",type:"enum",language:"rust"}),
//!   (plan_type:Class {name:"ProfileTransitionPlan",type:"class",language:"rust"}),
//!   (revision:Class {name:"ProfileRevisionRow",type:"class",language:"rust"}),
//!   (tests:Module {name:"tests",type:"module",language:"rust"}),
//!   (publish_test:Function {name:"profile_publication_uses_expected_version_and_appends_successor",type:"function",language:"rust",visibility:"private"}),
//!   (state_test:Function {name:"profile_disable_and_reenable_require_explicit_state_transitions",type:"function",language:"rust",visibility:"private"}),
//!   (rollback_test:Function {name:"profile_rollback_requires_an_older_revision_and_creates_active_successor",type:"function",language:"rust",visibility:"private"}),
//!   (active_binding:Function {name:"group_api::active_binding",type:"function",language:"rust"}),
//!   (require_scope:Function {name:"group_api::require_scope",type:"function",language:"rust"}),
//!   (set_tenant:Function {name:"group_api::set_tenant",type:"function",language:"rust"}),
//!   (validate_actor:Function {name:"group_api::validate_actor",type:"function",language:"rust"}),
//!   (document_verify:Function {name:"AgentExecutionProfileDocument::verify",type:"function",language:"rust"}),
//!   (document_decode:Function {name:"AgentExecutionProfileDocument::decode_and_verify",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(router),(m)-[:CONTAINS]->(project_handler),
//!   (m)-[:CONTAINS]->(worktree_handler),(m)-[:CONTAINS]->(mutate),(m)-[:CONTAINS]->(authorize),
//!   (m)-[:CONTAINS]->(lock),(m)-[:CONTAINS]->(load_current),(m)-[:CONTAINS]->(load_historical),
//!   (m)-[:CONTAINS]->(verify_document),(m)-[:CONTAINS]->(verify_row),(m)-[:CONTAINS]->(close_current),
//!   (m)-[:CONTAINS]->(insert_revision),(m)-[:CONTAINS]->(insert_audit),(m)-[:CONTAINS]->(plan),
//!   (m)-[:CONTAINS]->(parse_id),(m)-[:CONTAINS]->(no_store),(m)-[:CONTAINS]->(no_store_response),
//!   (m)-[:CONTAINS]->(target_scope_kind),(m)-[:CONTAINS]->(target),(m)-[:CONTAINS]->(resolved_target),
//!   (m)-[:CONTAINS]->(request),(m)-[:CONTAINS]->(action),(m)-[:CONTAINS]->(plan_type),(m)-[:CONTAINS]->(revision),
//!   (m)-[:CONTAINS]->(tests),(tests)-[:CONTAINS]->(publish_test),(tests)-[:CONTAINS]->(state_test),(tests)-[:CONTAINS]->(rollback_test),
//!   (target)-[:HAS_METHOD]->(target_scope_kind),
//!   (router)-[:CALLS]->(project_handler),(router)-[:CALLS]->(worktree_handler),(router)-[:CALLS]->(no_store_response),
//!   (project_handler)-[:CALLS]->(mutate),(project_handler)-[:CALLS]->(parse_id),(worktree_handler)-[:CALLS]->(mutate),
//!   (worktree_handler)-[:CALLS]->(parse_id),(mutate)-[:CALLS]->(authorize),(mutate)-[:CALLS]->(lock),
//!   (mutate)-[:CALLS]->(load_current),(mutate)-[:CALLS]->(plan),(mutate)-[:CALLS]->(verify_document),
//!   (mutate)-[:CALLS]->(verify_row),(mutate)-[:CALLS]->(load_historical),(mutate)-[:CALLS]->(close_current),
//!   (mutate)-[:CALLS]->(insert_revision),(mutate)-[:CALLS]->(insert_audit),
//!   (authorize)-[:CALLS]->(active_binding),(authorize)-[:CALLS]->(require_scope),
//!   (authorize)-[:CALLS]->(set_tenant),(authorize)-[:CALLS]->(validate_actor),(authorize)-[:CALLS]->(target_scope_kind),
//!   (verify_row)-[:CALLS]->(document_decode),(mutate)-[:CALLS]->(document_verify),
//!   (no_store_response)-[:CALLS]->(no_store),(publish_test)-[:CALLS]->(plan),
//!   (state_test)-[:CALLS]->(plan),(rollback_test)-[:CALLS]->(plan),(plan)-[:USES]->(action);

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderValue, header},
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::post,
};
use domain_agent::execution_profile::{AgentExecutionProfileDocument, MAX_EXECUTION_PROFILE_BYTES};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

use super::{
    AuthenticatedUser, GroupApiError, GroupApiState, active_binding, require_scope, set_tenant,
    validate_actor,
};

const MAX_PROFILE_MUTATION_REQUEST_BYTES: usize = MAX_EXECUTION_PROFILE_BYTES + 2_048;

#[derive(Clone, Copy, Debug)]
enum ProfileScopeTarget {
    Project { project_id: Uuid },
    Worktree { worktree_id: Uuid },
}

impl ProfileScopeTarget {
    fn scope_kind(self) -> &'static str {
        match self {
            Self::Project { .. } => "project",
            Self::Worktree { .. } => "worktree",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ResolvedProfileScope {
    project_id: Uuid,
    scope_kind: &'static str,
    worktree_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum ProfileMutationBody {
    Publish {
        expected_current_version: i64,
        correlation_id: Uuid,
        profile_document: AgentExecutionProfileDocument,
    },
    Disable {
        expected_current_version: i64,
        correlation_id: Uuid,
    },
    Reenable {
        expected_current_version: i64,
        correlation_id: Uuid,
    },
    Rollback {
        expected_current_version: i64,
        target_profile_version: i64,
        correlation_id: Uuid,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProfileMutationAction {
    Publish,
    Disable,
    Reenable,
    Rollback,
}

#[derive(Clone, Copy, Debug)]
struct ProfileTransitionPlan {
    next_version: i64,
    lifecycle_state: &'static str,
    event_type: &'static str,
    source_profile_version: Option<i64>,
}

#[derive(Debug, FromRow)]
struct ProfileRevisionRow {
    project_id: Uuid,
    profile_id: Uuid,
    scope_kind: String,
    worktree_id: Option<Uuid>,
    profile_version: i64,
    schema_version: i32,
    lifecycle_state: String,
    content_digest: String,
    profile_document: Value,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/execution-profiles/{profile_id}/lifecycle",
            post(project_profile_lifecycle),
        )
        .route(
            "/api/v1/worktrees/{worktree_id}/execution-profiles/{profile_id}/lifecycle",
            post(worktree_profile_lifecycle),
        )
        .layer(DefaultBodyLimit::max(MAX_PROFILE_MUTATION_REQUEST_BYTES))
        .route_layer(map_response(no_store_response))
}

async fn project_profile_lifecycle(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((project_id, profile_id)): Path<(String, String)>,
    body: Bytes,
) -> Result<Response, GroupApiError> {
    let target = ProfileScopeTarget::Project {
        project_id: parse_id(&project_id)?,
    };
    mutate_profile(&state, &actor, target, parse_id(&profile_id)?, &body).await
}

async fn worktree_profile_lifecycle(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, profile_id)): Path<(String, String)>,
    body: Bytes,
) -> Result<Response, GroupApiError> {
    let target = ProfileScopeTarget::Worktree {
        worktree_id: parse_id(&worktree_id)?,
    };
    mutate_profile(&state, &actor, target, parse_id(&profile_id)?, &body).await
}

async fn mutate_profile(
    state: &GroupApiState,
    actor: &super::AuthUser,
    target: ProfileScopeTarget,
    profile_id: Uuid,
    body_bytes: &[u8],
) -> Result<Response, GroupApiError> {
    if profile_id.is_nil() || body_bytes.len() > MAX_PROFILE_MUTATION_REQUEST_BYTES {
        return Err(GroupApiError::bad_request());
    }
    let body: ProfileMutationBody = serde_json::from_slice(body_bytes)
        .map_err(|_| GroupApiError::invalid_request("invalid_execution_profile_mutation"))?;
    let (action, expected_current_version, target_version, correlation_id, candidate_document) =
        match body {
            ProfileMutationBody::Publish {
                expected_current_version,
                correlation_id,
                profile_document,
            } => {
                let verified = profile_document
                    .verify()
                    .map_err(|_| GroupApiError::invalid_request("invalid_execution_profile"))?;
                (
                    ProfileMutationAction::Publish,
                    expected_current_version,
                    None,
                    correlation_id,
                    Some(verified.document().clone()),
                )
            }
            ProfileMutationBody::Disable {
                expected_current_version,
                correlation_id,
            } => (
                ProfileMutationAction::Disable,
                expected_current_version,
                None,
                correlation_id,
                None,
            ),
            ProfileMutationBody::Reenable {
                expected_current_version,
                correlation_id,
            } => (
                ProfileMutationAction::Reenable,
                expected_current_version,
                None,
                correlation_id,
                None,
            ),
            ProfileMutationBody::Rollback {
                expected_current_version,
                target_profile_version,
                correlation_id,
            } => (
                ProfileMutationAction::Rollback,
                expected_current_version,
                Some(target_profile_version),
                correlation_id,
                None,
            ),
        };
    if expected_current_version < 0
        || correlation_id.is_nil()
        || (action != ProfileMutationAction::Publish && expected_current_version == 0)
        || target_version.is_some_and(|version| version <= 0 || version >= expected_current_version)
    {
        return Err(GroupApiError::invalid_request(
            "invalid_execution_profile_mutation",
        ));
    }

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    let resolved = authorize_profile_target(&mut tx, actor, target).await?;
    lock_profile_id(&mut tx, actor.tenant_id, profile_id).await?;
    let current = load_current_profile(&mut tx, actor.tenant_id, profile_id).await?;
    let current_document = current
        .as_ref()
        .map(|row| verify_profile_revision(row, actor.tenant_id, resolved))
        .transpose()?;
    let plan = plan_profile_transition(
        action,
        expected_current_version,
        current.as_ref().map(|row| row.profile_version),
        current.as_ref().map(|row| row.lifecycle_state.as_str()),
        target_version,
    )?;

    let document = match action {
        ProfileMutationAction::Publish => candidate_document
            .ok_or_else(|| GroupApiError::invalid_request("invalid_execution_profile"))?,
        ProfileMutationAction::Disable | ProfileMutationAction::Reenable => {
            current_document.ok_or_else(GroupApiError::not_found)?
        }
        ProfileMutationAction::Rollback => {
            let target_version = target_version.ok_or_else(GroupApiError::bad_request)?;
            let target_row =
                load_historical_profile(&mut tx, actor.tenant_id, profile_id, target_version)
                    .await?
                    .ok_or_else(GroupApiError::not_found)?;
            verify_profile_revision(&target_row, actor.tenant_id, resolved)?
        }
    };
    verify_profile_document(&document, actor.tenant_id, resolved)?;

    if let Some(row) = &current {
        close_current_revision(&mut tx, actor.tenant_id, profile_id, row.profile_version).await?;
    }
    let revision_id = insert_profile_revision(
        &mut tx,
        actor,
        profile_id,
        resolved,
        plan.next_version,
        plan.lifecycle_state,
        &document,
    )
    .await?;
    insert_profile_audit(
        &mut tx,
        actor,
        profile_id,
        resolved,
        plan,
        correlation_id,
        &document,
    )
    .await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    Ok(Json(json!({
        "profile_revision_id": revision_id,
        "profile_id": profile_id,
        "profile_version": plan.next_version,
        "lifecycle_state": plan.lifecycle_state,
        "content_digest": document.content_digest,
        "source_profile_version": plan.source_profile_version,
    }))
    .into_response())
}

async fn authorize_profile_target(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    target: ProfileScopeTarget,
) -> Result<ResolvedProfileScope, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, "execution-profile:publish")?;
    set_tenant(tx, actor.tenant_id).await?;
    let (project_id, worktree_id) = match target {
        ProfileScopeTarget::Project { project_id } => (project_id, None),
        ProfileScopeTarget::Worktree { worktree_id } => {
            let project_id = sqlx::query_scalar::<_, Uuid>(
                r#"SELECT p.project_id
                   FROM worktree_canvas_worktree w
                   JOIN multica.worktree_project_binding p
                     ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
                    AND p.project_id = w.project_id AND p.valid_to IS NULL
                   WHERE w.id = $1 AND w.tenant_id = $2
                   FOR SHARE OF w, p"#,
            )
            .bind(worktree_id)
            .bind(actor.tenant_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|_| GroupApiError::internal())?
            .ok_or_else(GroupApiError::not_found)?;
            (project_id, Some(worktree_id))
        }
    };
    let binding = active_binding(tx, actor, project_id).await?;
    if !matches!(binding.role.as_str(), "tenant_admin" | "project_admin") {
        return Err(GroupApiError::forbidden());
    }
    Ok(ResolvedProfileScope {
        project_id,
        scope_kind: target.scope_kind(),
        worktree_id,
    })
}

async fn lock_profile_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    profile_id: Uuid,
) -> Result<(), GroupApiError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || ':' || $2, 0))")
        .bind(tenant_id.to_string())
        .bind(profile_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn load_current_profile(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    profile_id: Uuid,
) -> Result<Option<ProfileRevisionRow>, GroupApiError> {
    sqlx::query_as::<_, ProfileRevisionRow>(
        r#"SELECT project_id, profile_id, scope_kind, worktree_id,
                  profile_version, schema_version, lifecycle_state,
                  rtrim(content_digest::text) AS content_digest, profile_document
           FROM multica.agent_execution_profile
           WHERE tenant_id = $1 AND profile_id = $2 AND valid_to IS NULL
           FOR UPDATE"#,
    )
    .bind(tenant_id)
    .bind(profile_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

async fn load_historical_profile(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    profile_id: Uuid,
    profile_version: i64,
) -> Result<Option<ProfileRevisionRow>, GroupApiError> {
    sqlx::query_as::<_, ProfileRevisionRow>(
        r#"SELECT project_id, profile_id, scope_kind, worktree_id,
                  profile_version, schema_version, lifecycle_state,
                  rtrim(content_digest::text) AS content_digest, profile_document
           FROM multica.agent_execution_profile
           WHERE tenant_id = $1 AND profile_id = $2 AND profile_version = $3
             AND valid_to IS NOT NULL
           FOR SHARE"#,
    )
    .bind(tenant_id)
    .bind(profile_id)
    .bind(profile_version)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

fn verify_profile_revision(
    row: &ProfileRevisionRow,
    tenant_id: Uuid,
    resolved: ResolvedProfileScope,
) -> Result<AgentExecutionProfileDocument, GroupApiError> {
    if row.profile_id.is_nil()
        || row.project_id != resolved.project_id
        || row.scope_kind != resolved.scope_kind
        || row.worktree_id != resolved.worktree_id
        || row.profile_version <= 0
        || row.schema_version <= 0
    {
        return Err(GroupApiError::not_found());
    }
    let encoded =
        serde_json::to_vec(&row.profile_document).map_err(|_| GroupApiError::internal())?;
    let verified = AgentExecutionProfileDocument::decode_and_verify(&encoded)
        .map_err(|_| GroupApiError::internal())?;
    let profile_scope = &verified.document().profile.scope;
    if profile_scope.tenant_id != tenant_id
        || profile_scope.project_id != resolved.project_id
        || profile_scope.worktree_id != resolved.worktree_id
        || verified.document().profile.schema_version != row.schema_version as u32
        || verified.document().content_digest != row.content_digest
    {
        return Err(GroupApiError::internal());
    }
    Ok(verified.document().clone())
}

fn verify_profile_document(
    document: &AgentExecutionProfileDocument,
    tenant_id: Uuid,
    resolved: ResolvedProfileScope,
) -> Result<(), GroupApiError> {
    if document.profile.scope.tenant_id != tenant_id
        || document.profile.scope.project_id != resolved.project_id
        || document.profile.scope.worktree_id != resolved.worktree_id
        || (resolved.scope_kind == "project" && resolved.worktree_id.is_some())
        || (resolved.scope_kind == "worktree" && resolved.worktree_id.is_none())
    {
        return Err(GroupApiError::invalid_request(
            "execution_profile_scope_mismatch",
        ));
    }
    Ok(())
}

async fn close_current_revision(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    profile_id: Uuid,
    expected_version: i64,
) -> Result<(), GroupApiError> {
    let result = sqlx::query(
        r#"UPDATE multica.agent_execution_profile
           SET valid_to = clock_timestamp()
           WHERE tenant_id = $1 AND profile_id = $2
             AND profile_version = $3 AND valid_to IS NULL"#,
    )
    .bind(tenant_id)
    .bind(profile_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    if result.rows_affected() != 1 {
        return Err(GroupApiError::conflict(
            "execution_profile_version_conflict",
        ));
    }
    Ok(())
}

async fn insert_profile_revision(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    profile_id: Uuid,
    resolved: ResolvedProfileScope,
    profile_version: i64,
    lifecycle_state: &str,
    document: &AgentExecutionProfileDocument,
) -> Result<Uuid, GroupApiError> {
    let profile_document = serde_json::to_value(document).map_err(|_| GroupApiError::internal())?;
    sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO multica.agent_execution_profile
           (tenant_id, project_id, profile_id, scope_kind, worktree_id,
            profile_version, schema_version, lifecycle_state, content_digest,
            profile_document, changed_by, valid_from)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, clock_timestamp())
           RETURNING profile_revision_id"#,
    )
    .bind(actor.tenant_id)
    .bind(resolved.project_id)
    .bind(profile_id)
    .bind(resolved.scope_kind)
    .bind(resolved.worktree_id)
    .bind(profile_version)
    .bind(i32::try_from(document.profile.schema_version).map_err(|_| GroupApiError::internal())?)
    .bind(lifecycle_state)
    .bind(&document.content_digest)
    .bind(profile_document)
    .bind(actor.user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())
}

async fn insert_profile_audit(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &super::AuthUser,
    profile_id: Uuid,
    resolved: ResolvedProfileScope,
    plan: ProfileTransitionPlan,
    correlation_id: Uuid,
    document: &AgentExecutionProfileDocument,
) -> Result<(), GroupApiError> {
    sqlx::query(
        r#"INSERT INTO multica.agent_execution_profile_audit_event
           (tenant_id, project_id, profile_id, profile_version, scope_kind,
            worktree_id, event_type, source_profile_version, actor_id, correlation_id, details)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
    )
    .bind(actor.tenant_id)
    .bind(resolved.project_id)
    .bind(profile_id)
    .bind(plan.next_version)
    .bind(resolved.scope_kind)
    .bind(resolved.worktree_id)
    .bind(plan.event_type)
    .bind(plan.source_profile_version)
    .bind(actor.user_id)
    .bind(correlation_id)
    .bind(json!({ "content_digest": document.content_digest }))
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

fn plan_profile_transition(
    action: ProfileMutationAction,
    expected_current_version: i64,
    current_version: Option<i64>,
    current_state: Option<&str>,
    target_profile_version: Option<i64>,
) -> Result<ProfileTransitionPlan, GroupApiError> {
    if expected_current_version < 0 || current_version.unwrap_or(0) != expected_current_version {
        return Err(GroupApiError::conflict(
            "execution_profile_version_conflict",
        ));
    }
    let next_version = expected_current_version
        .checked_add(1)
        .filter(|version| *version > 0)
        .ok_or_else(|| GroupApiError::conflict("execution_profile_version_conflict"))?;
    let (lifecycle_state, event_type, source_profile_version) = match action {
        ProfileMutationAction::Publish => {
            if current_state.is_some_and(|state| state != "active") {
                return Err(GroupApiError::conflict("execution_profile_state_conflict"));
            }
            ("active", "profile_published", None)
        }
        ProfileMutationAction::Disable => {
            if current_state != Some("active") {
                return Err(GroupApiError::conflict("execution_profile_state_conflict"));
            }
            ("disabled", "profile_disabled", None)
        }
        ProfileMutationAction::Reenable => {
            if current_state != Some("disabled") {
                return Err(GroupApiError::conflict("execution_profile_state_conflict"));
            }
            ("active", "profile_reenabled", None)
        }
        ProfileMutationAction::Rollback => {
            let target_version = target_profile_version.ok_or_else(GroupApiError::bad_request)?;
            if current_state.is_none() || target_version <= 0 || target_version >= next_version - 1
            {
                return Err(GroupApiError::invalid_request(
                    "invalid_execution_profile_rollback",
                ));
            }
            ("active", "profile_rolled_back", Some(target_version))
        }
    };
    Ok(ProfileTransitionPlan {
        next_version,
        lifecycle_state,
        event_type,
        source_profile_version,
    })
}

fn parse_id(value: &str) -> Result<Uuid, GroupApiError> {
    Uuid::parse_str(value).map_err(|_| GroupApiError::bad_request())
}

fn no_store<T: IntoResponse>(value: T) -> Response {
    let mut response = value.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn no_store_response(response: Response) -> Response {
    no_store(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_publication_uses_expected_version_and_appends_successor() {
        let first = plan_profile_transition(ProfileMutationAction::Publish, 0, None, None, None)
            .expect("initial profile can publish at version zero");
        assert_eq!(first.next_version, 1);
        assert_eq!(first.lifecycle_state, "active");
        assert_eq!(first.event_type, "profile_published");

        let successor = plan_profile_transition(
            ProfileMutationAction::Publish,
            4,
            Some(4),
            Some("active"),
            None,
        )
        .expect("current active profile can publish a successor");
        assert_eq!(successor.next_version, 5);
        assert!(
            plan_profile_transition(
                ProfileMutationAction::Publish,
                3,
                Some(4),
                Some("active"),
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn profile_disable_and_reenable_require_explicit_state_transitions() {
        let disabled = plan_profile_transition(
            ProfileMutationAction::Disable,
            2,
            Some(2),
            Some("active"),
            None,
        )
        .expect("active profile can be disabled");
        assert_eq!(disabled.next_version, 3);
        assert_eq!(disabled.lifecycle_state, "disabled");
        assert!(
            plan_profile_transition(
                ProfileMutationAction::Disable,
                3,
                Some(3),
                Some("disabled"),
                None,
            )
            .is_err()
        );

        let reenabled = plan_profile_transition(
            ProfileMutationAction::Reenable,
            3,
            Some(3),
            Some("disabled"),
            None,
        )
        .expect("disabled profile can be explicitly reenabled");
        assert_eq!(reenabled.next_version, 4);
        assert_eq!(reenabled.event_type, "profile_reenabled");
    }

    #[test]
    fn profile_rollback_requires_an_older_revision_and_creates_active_successor() {
        let rollback = plan_profile_transition(
            ProfileMutationAction::Rollback,
            7,
            Some(7),
            Some("disabled"),
            Some(3),
        )
        .expect("historical content can be restored as a new active revision");
        assert_eq!(rollback.next_version, 8);
        assert_eq!(rollback.lifecycle_state, "active");
        assert_eq!(rollback.event_type, "profile_rolled_back");
        assert_eq!(rollback.source_profile_version, Some(3));
        assert!(
            plan_profile_transition(
                ProfileMutationAction::Rollback,
                7,
                Some(7),
                Some("active"),
                Some(7),
            )
            .is_err()
        );
    }
}
