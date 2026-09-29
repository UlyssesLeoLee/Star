//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"worktree_lifecycle.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"worktree_lifecycle",type:"module",language:"rust"}),
//!   (p:Interface {name:"ProjectWorktreeLifecycleProvider",type:"interface",language:"rust",visibility:"pub"}),
//!   (repo_list:Function {name:"ProjectWorktreeLifecycleProvider::list_project_repositories",type:"function",language:"rust",visibility:"pub"}),
//!   (pe:Enum {name:"WorktreeLifecycleProviderError",type:"enum",language:"rust",visibility:"pub"}),
//!   (cq:Class {name:"WorktreeImportCandidatesQuery",type:"class",language:"rust",visibility:"pub"}),
//!   (prq:Class {name:"ProjectWorktreeRepositoryQuery",type:"class",language:"rust",visibility:"pub"}),(pr:Class {name:"ProjectWorktreeRepository",type:"class",language:"rust",visibility:"pub"}),
//!   (repo_test:Function {name:"project_repository_projection_rejects_invalid_or_duplicate_rows",type:"function",language:"rust"}),
//!   (ic:Class {name:"WorktreeImportCandidate",type:"class",language:"rust",visibility:"pub"}),
//!   (cc:Class {name:"ProjectWorktreeCreateCommand",type:"class",language:"rust",visibility:"pub"}),
//!   (im:Class {name:"ProjectWorktreeImportCommand",type:"class",language:"rust",visibility:"pub"}),
//!   (rc:Class {name:"WorktreeLifecycleReceipt",type:"class",language:"rust",visibility:"pub"}),
//!   (rt:Function {name:"router",type:"function",language:"rust",visibility:"pub"}),
//!   (ac:Function {name:"authorize_project_creator",type:"function",language:"rust",visibility:"private"}),(lr:Function {name:"list_project_worktree_repositories",type:"function",language:"rust",visibility:"private"}),(validate_project_repositories:Function {name:"validate_project_repositories",type:"function",language:"rust",visibility:"private"}),(displayName:Function {name:"validate_display_name",type:"function",language:"rust",visibility:"private"}),
//!   (li:Function {name:"list_import_candidates",type:"function",language:"rust",visibility:"private"}),
//!   (cr:Function {name:"create_project_worktree",type:"function",language:"rust",visibility:"private"}),
//!   (imf:Function {name:"import_project_worktree",type:"function",language:"rust",visibility:"private"}),
//!   (ik:Function {name:"read_idempotency_key",type:"function",language:"rust",visibility:"private"}),
//!   (fp:Function {name:"request_fingerprint",type:"function",language:"rust",visibility:"private"}),
//!   (vr:Function {name:"validate_git_ref",type:"function",language:"rust",visibility:"private"}),
//!   (vc:Function {name:"validate_candidate_id",type:"function",language:"rust",visibility:"private"}),
//!   (rr:Function {name:"validate_receipt",type:"function",language:"rust",visibility:"private"}),(hc:Function {name:"validate_head_commit",type:"function",language:"rust",visibility:"private"}),(wtr:Function {name:"worktrees::router",type:"function",language:"rust"}),
//!   (em:Function {name:"map_provider_error",type:"function",language:"rust",visibility:"private"}),
//!   (tests:Module {name:"worktree_lifecycle_tests",type:"module",language:"rust"}),
//!   (ref_test:Function {name:"git_refs_reject_path_and_option_injection",type:"function",language:"rust"}),
//!   (candidate_test:Function {name:"candidate_id_is_opaque_and_path_free",type:"function",language:"rust"}),
//!   (role_test:Function {name:"creator_role_is_project_member_writer",type:"function",language:"rust"}),
//!   (receipt_test:Function {name:"receipt_must_match_project_and_repository",type:"function",language:"rust"}),(head_commit_test:Function {name:"head_commit_is_full_hex_object_id",type:"function",language:"rust"}),(route_merge_test:Function {name:"lifecycle_routes_merge_with_existing_index_read_route",type:"function",language:"rust"}),(unknown_fields_test:Function {name:"request_dtos_reject_paths_and_urls",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(p),(p)-[:HAS_METHOD]->(repo_list),(m)-[:CONTAINS]->(pe),(m)-[:CONTAINS]->(cq),(m)-[:CONTAINS]->(ic),(m)-[:CONTAINS]->(cc),(m)-[:CONTAINS]->(im),(m)-[:CONTAINS]->(rc),(m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(ac),(m)-[:CONTAINS]->(li),(m)-[:CONTAINS]->(cr),(m)-[:CONTAINS]->(imf),(m)-[:CONTAINS]->(ik),(m)-[:CONTAINS]->(fp),(m)-[:CONTAINS]->(vr),(m)-[:CONTAINS]->(vc),(m)-[:CONTAINS]->(rr),(m)-[:CONTAINS]->(em),(m)-[:CONTAINS]->(tests),(m)-[:CONTAINS]->(hc),(m)-[:CONTAINS]->(prq),(m)-[:CONTAINS]->(pr),(m)-[:CONTAINS]->(validate_project_repositories),(m)-[:CONTAINS]->(displayName),(tests)-[:CONTAINS]->(ref_test),(tests)-[:CONTAINS]->(candidate_test),(tests)-[:CONTAINS]->(role_test),(tests)-[:CONTAINS]->(receipt_test),(tests)-[:CONTAINS]->(head_commit_test),(tests)-[:CONTAINS]->(route_merge_test),(tests)-[:CONTAINS]->(unknown_fields_test),(tests)-[:CONTAINS]->(repo_test),
//!   (rt)-[:CALLS]->(lr),(rt)-[:CALLS]->(li),(rt)-[:CALLS]->(cr),(rt)-[:CALLS]->(imf),(lr)-[:CALLS]->(ac),(lr)-[:CALLS]->(validate_project_repositories),(lr)-[:USES]->(prq),(li)-[:CALLS]->(ac),(li)-[:CALLS]->(vc),(li)-[:CALLS]->(displayName),(validate_project_repositories)-[:CALLS]->(displayName),(cr)-[:CALLS]->(ac),(cr)-[:CALLS]->(ik),(cr)-[:CALLS]->(vr),(cr)-[:CALLS]->(fp),(cr)-[:CALLS]->(rr),(cr)-[:CALLS]->(em),(imf)-[:CALLS]->(ac),(imf)-[:CALLS]->(ik),(imf)-[:CALLS]->(vc),(imf)-[:CALLS]->(fp),(imf)-[:CALLS]->(rr),(imf)-[:CALLS]->(em),(fp)-[:USES]->(cq),(cr)-[:USES]->(cc),(imf)-[:USES]->(im),(vr)-[:USES]->(cc),(rr)-[:USES]->(rc),(ref_test)-[:CALLS]->(vr),(candidate_test)-[:CALLS]->(vc),(candidate_test)-[:CALLS]->(displayName),(role_test)-[:CALLS]->(ac),(receipt_test)-[:CALLS]->(rr),(head_commit_test)-[:CALLS]->(hc),(route_merge_test)-[:CALLS]->(rt),(route_merge_test)-[:CALLS]->(wtr),
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"worktree_lifecycle",type:"module"}),(p:Interface {name:"ProjectWorktreeLifecycleProvider",type:"interface"}),(hc:Function {name:"validate_head_commit",type:"function"}),(wtr:Function {name:"worktrees::router",type:"function"}),(li:Function {name:"list_import_candidates",type:"function"}),(cr:Function {name:"create_project_worktree",type:"function"}),(imf:Function {name:"import_project_worktree",type:"function"}),(ac:Function {name:"authorize_project_creator",type:"function"}),(fp:Function {name:"request_fingerprint",type:"function"}),(rr:Function {name:"validate_receipt",type:"function"}),(vr:Function {name:"validate_git_ref",type:"function"}),(vc:Function {name:"validate_candidate_id",type:"function"}),(em:Function {name:"map_provider_error",type:"function"});
//! CREATE (createMethod:Function {name:"ProjectWorktreeLifecycleProvider::create",type:"function",language:"rust"}),(importMethod:Function {name:"ProjectWorktreeLifecycleProvider::import",type:"function",language:"rust"}),(listMethod:Function {name:"ProjectWorktreeLifecycleProvider::list_import_candidates",type:"function",language:"rust"}),(state:Enum {name:"WorktreeLifecycleState",type:"enum",language:"rust"}),(queryBody:Class {name:"ImportCandidatesRequest",type:"class",language:"rust"}),(createBody:Class {name:"CreateWorktreeBody",type:"class",language:"rust"}),(importBody:Class {name:"ImportWorktreeBody",type:"class",language:"rust"}),(role:Function {name:"require_project_creator_role",type:"function",language:"rust",visibility:"private"}),(validateCreate:Function {name:"validate_create_body",type:"function",language:"rust",visibility:"private"}),(snapshot:Function {name:"permission_snapshot_ref",type:"function",language:"rust",visibility:"private"});
//! CREATE (m)-[:CONTAINS]->(state),(m)-[:CONTAINS]->(queryBody),(m)-[:CONTAINS]->(createBody),(m)-[:CONTAINS]->(importBody),(m)-[:CONTAINS]->(role),(m)-[:CONTAINS]->(validateCreate),(m)-[:CONTAINS]->(snapshot),(li)-[:CALLS]->(ac),(li)-[:CALLS]->(vc),(li)-[:CALLS]->(vr),(li)-[:CALLS]->(em),(li)-[:CALLS]->(snapshot),(cr)-[:CALLS]->(ac),(cr)-[:CALLS]->(ik),(cr)-[:CALLS]->(validateCreate),(cr)-[:CALLS]->(fp),(cr)-[:CALLS]->(rr),(cr)-[:CALLS]->(em),(cr)-[:CALLS]->(snapshot),(imf)-[:CALLS]->(ac),(imf)-[:CALLS]->(ik),(imf)-[:CALLS]->(vc),(imf)-[:CALLS]->(fp),(imf)-[:CALLS]->(rr),(imf)-[:CALLS]->(em),(imf)-[:CALLS]->(snapshot),(ac)-[:CALLS]->(role),(rr)-[:CALLS]->(vr),(validateCreate)-[:CALLS]->(vr),(fp)-[:USES]->(createBody),(fp)-[:USES]->(importBody),(cr)-[:USES]->(createBody),(imf)-[:USES]->(importBody);
//! MATCH (m:Module {name:"worktree_lifecycle",type:"module"}),(ik:Function {name:"read_idempotency_key",type:"function"}),(em:Function {name:"map_provider_error",type:"function"}),(tests:Module {name:"worktree_lifecycle_tests",type:"module"}),(ref_test:Function {name:"git_refs_reject_path_and_option_injection",type:"function"}),(candidate_test:Function {name:"candidate_id_is_opaque_and_path_free",type:"function"}),(role_test:Function {name:"creator_role_is_project_member_writer",type:"function"}),(receipt_test:Function {name:"receipt_must_match_project_and_repository",type:"function"}),(head_commit_test:Function {name:"head_commit_is_full_hex_object_id",type:"function"}),(route_merge_test:Function {name:"lifecycle_routes_merge_with_existing_index_read_route",type:"function"}),(hc:Function {name:"validate_head_commit",type:"function"}),(wtr:Function {name:"worktrees::router",type:"function"}),(vr:Function {name:"validate_git_ref",type:"function"}),(vc:Function {name:"validate_candidate_id",type:"function"}),(ac:Function {name:"authorize_project_creator",type:"function"}),(rr:Function {name:"validate_receipt",type:"function"}),(role:Function {name:"require_project_creator_role",type:"function"}),(validateCreate:Function {name:"validate_create_body",type:"function"}),(queryBody:Class {name:"ImportCandidatesRequest",type:"class"}),(importBody:Class {name:"ImportWorktreeBody",type:"class"}),(unknown_fields_test:Function {name:"request_dtos_reject_paths_and_urls",type:"function"});
//! CREATE (m)-[:CONTAINS]->(ik),(ref_test)-[:CALLS]->(vr),(candidate_test)-[:CALLS]->(vc),(role_test)-[:CALLS]->(role),(receipt_test)-[:CALLS]->(rr),(head_commit_test)-[:CALLS]->(hc),(route_merge_test)-[:CALLS]->(rt),(route_merge_test)-[:CALLS]->(wtr),(unknown_fields_test)-[:CALLS]->(validateCreate),(unknown_fields_test)-[:CALLS]->(queryBody),(unknown_fields_test)-[:CALLS]->(importBody);
//! CYPHER STRUCTURAL MANIFEST END

use std::collections::HashSet;

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{
        HeaderMap, StatusCode,
        header::{CACHE_CONTROL, VARY},
    },
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::AuthUser;

use super::{
    AuthenticatedUser, GroupApiError, GroupApiState, ProjectBinding, active_binding, require_scope,
    set_tenant, validate_actor,
};

/// A Project-scoped opaque Git Worktree candidate returned by the trusted host registry.
/// This DTO deliberately contains no checkout path, repository URL, or executable command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeImportCandidate {
    pub candidate_id: String,
    pub repository_id: Uuid,
    pub name: String,
    pub branch: String,
    pub head_commit: String,
    pub dirty: bool,
    pub observed_at: DateTime<Utc>,
}

/// Request context for bounded discovery of externally-created Worktrees.
#[derive(Debug, Clone)]
pub struct WorktreeImportCandidatesQuery {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub actor_id: Uuid,
    pub permission_snapshot_ref: String,
    pub repository_id: Uuid,
    pub limit: usize,
}

/// A safe Project repository option for Worktree lifecycle UI. It intentionally excludes
/// remote URLs and host checkout paths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectWorktreeRepository {
    pub repository_id: Uuid,
    pub name: String,
    pub default_branch: String,
}

#[derive(Debug, Clone)]
pub struct ProjectWorktreeRepositoryQuery {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub actor_id: Uuid,
    pub permission_snapshot_ref: String,
}

/// A project-authorized Worktree creation request. Physical paths and remote URLs are
/// resolved exclusively by the trusted lifecycle provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectWorktreeCreateCommand {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub actor_id: Uuid,
    pub permission_snapshot_ref: String,
    pub repository_id: Uuid,
    pub branch: String,
    pub base_ref: String,
    pub idempotency_key: String,
    pub request_fingerprint: [u8; 32],
    pub correlation_id: Uuid,
}

/// Import is bound to an opaque candidate from the trusted host registry; the caller
/// never supplies a local path, Git URL, or filesystem selector.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectWorktreeImportCommand {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub actor_id: Uuid,
    pub permission_snapshot_ref: String,
    pub repository_id: Uuid,
    pub candidate_id: String,
    pub idempotency_key: String,
    pub request_fingerprint: [u8; 32],
    pub correlation_id: Uuid,
}

/// Durable acceptance returned only after the provider has recorded the operation and
/// Worktree projection/outbox state. It intentionally omits local filesystem paths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeLifecycleReceipt {
    pub operation_id: Uuid,
    pub worktree_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub branch: String,
    pub state: WorktreeLifecycleState,
    pub accepted_at: DateTime<Utc>,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeLifecycleState {
    Provisioning,
    Ready,
}

/// Errors are intentionally coarse to avoid exposing host paths or repository details.
#[derive(Debug, thiserror::Error)]
pub enum WorktreeLifecycleProviderError {
    #[error("trusted Worktree lifecycle provider is unavailable")]
    Unavailable,
    #[error("resource is not available in the authorized Project scope")]
    NotFound,
    #[error("the Worktree operation conflicts with current state")]
    Conflict,
    #[error("the Worktree request is invalid")]
    InvalidSpecification,
    #[error("the Worktree lifecycle operation failed")]
    Internal,
}

/// Trusted host adapter for Project repository resolution and Git Worktree lifecycle.
/// Implementations must recheck current Project membership and Project↔Repository
/// binding immediately before a Git side effect; re-resolve opaque import candidates
/// against that binding; derive all local paths from server-side configuration; persist operation + Worktree/Project binding + Audit + Outbox
/// idempotently before returning a receipt; and never trust the request's permission
/// snapshot as a current authorization grant.
#[async_trait]
pub trait ProjectWorktreeLifecycleProvider: Send + Sync {
    async fn list_project_repositories(
        &self,
        query: ProjectWorktreeRepositoryQuery,
    ) -> Result<Vec<ProjectWorktreeRepository>, WorktreeLifecycleProviderError>;

    async fn list_import_candidates(
        &self,
        query: WorktreeImportCandidatesQuery,
    ) -> Result<Vec<WorktreeImportCandidate>, WorktreeLifecycleProviderError>;

    async fn create(
        &self,
        command: ProjectWorktreeCreateCommand,
    ) -> Result<WorktreeLifecycleReceipt, WorktreeLifecycleProviderError>;

    async fn import(
        &self,
        command: ProjectWorktreeImportCommand,
    ) -> Result<WorktreeLifecycleReceipt, WorktreeLifecycleProviderError>;
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportCandidatesRequest {
    repository_id: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreateWorktreeBody {
    repository_id: Uuid,
    branch: String,
    base_ref: String,
    correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ImportWorktreeBody {
    repository_id: Uuid,
    candidate_id: String,
    correlation_id: Option<Uuid>,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/worktree-repositories",
            get(list_project_worktree_repositories),
        )
        .route(
            "/api/v1/projects/{project_id}/worktree-import-candidates",
            get(list_import_candidates),
        )
        .route(
            "/api/v1/projects/{project_id}/worktrees",
            post(create_project_worktree),
        )
        .route(
            "/api/v1/projects/{project_id}/worktrees/import",
            post(import_project_worktree),
        )
}

async fn list_project_worktree_repositories(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, GroupApiError> {
    let project_id = Uuid::parse_str(&project_id).map_err(|_| GroupApiError::bad_request())?;
    validate_actor(&actor)?;
    require_scope(&actor, "project:read")?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let binding = active_binding(&mut tx, &actor, project_id).await?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    let provider = state
        .worktree_lifecycle_provider
        .ok_or_else(|| GroupApiError::feature_unavailable("worktree_lifecycle_unavailable"))?;
    let repositories = provider
        .list_project_repositories(ProjectWorktreeRepositoryQuery {
            tenant_id: actor.tenant_id,
            project_id,
            actor_id: actor.user_id,
            permission_snapshot_ref: permission_snapshot_ref(&binding),
        })
        .await
        .map_err(map_provider_error)?;
    validate_project_repositories(&repositories)?;
    Ok((
        [(CACHE_CONTROL, "no-store"), (VARY, "Authorization")],
        Json(json!({ "project_id": project_id, "repositories": repositories })),
    ))
}

fn validate_project_repositories(
    repositories: &[ProjectWorktreeRepository],
) -> Result<(), GroupApiError> {
    let mut seen = HashSet::new();
    if repositories.len() > 100
        || repositories.iter().any(|repository| {
            repository.repository_id.is_nil()
                || !validate_display_name(&repository.name)
                || !validate_git_ref(&repository.default_branch)
                || !seen.insert(repository.repository_id)
        })
    {
        return Err(GroupApiError::internal());
    }
    Ok(())
}

fn validate_display_name(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 128
        && !value.chars().any(char::is_control)
        && !value.contains("://")
        && !value.contains("..")
        && !value.starts_with('/')
        && !value.starts_with('\\')
        && !value.as_bytes().get(1).is_some_and(|byte| {
            *byte == b':'
                && value
                    .as_bytes()
                    .get(2)
                    .is_some_and(|slash| matches!(*slash, b'/' | b'\\'))
        })
}

async fn authorize_project_creator(
    state: &GroupApiState,
    actor: &AuthUser,
    project_id: Uuid,
) -> Result<ProjectBinding, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, "worktree:create")?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let binding = active_binding(&mut tx, actor, project_id).await?;
    require_project_creator_role(&binding.role)?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(binding)
}

async fn list_import_candidates(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    Query(query): Query<ImportCandidatesRequest>,
) -> Result<impl IntoResponse, GroupApiError> {
    let project_id = Uuid::parse_str(&project_id).map_err(|_| GroupApiError::bad_request())?;
    let repository_id =
        Uuid::parse_str(&query.repository_id).map_err(|_| GroupApiError::bad_request())?;
    let limit = query.limit.unwrap_or(25).clamp(1, 50);
    let binding = authorize_project_creator(&state, &actor, project_id).await?;
    let provider = state
        .worktree_lifecycle_provider
        .ok_or_else(|| GroupApiError::feature_unavailable("worktree_lifecycle_unavailable"))?;
    let provider_query = WorktreeImportCandidatesQuery {
        tenant_id: actor.tenant_id,
        project_id,
        actor_id: actor.user_id,
        permission_snapshot_ref: permission_snapshot_ref(&binding),
        repository_id,
        limit,
    };
    let candidates = provider
        .list_import_candidates(provider_query)
        .await
        .map_err(map_provider_error)?;
    let mut seen = HashSet::new();
    if candidates.len() > limit
        || candidates.iter().any(|candidate| {
            candidate.repository_id != repository_id
                || !validate_candidate_id(&candidate.candidate_id)
                || !validate_git_ref(&candidate.branch)
                || !validate_display_name(&candidate.name)
                || !validate_head_commit(&candidate.head_commit)
                || !seen.insert(candidate.candidate_id.clone())
        })
    {
        return Err(GroupApiError::internal());
    }
    Ok((
        [(CACHE_CONTROL, "no-store"), (VARY, "Authorization")],
        Json(json!({
            "project_id": project_id,
            "repository_id": repository_id,
            "candidates": candidates,
        })),
    ))
}

async fn create_project_worktree(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<CreateWorktreeBody>,
) -> Result<impl IntoResponse, GroupApiError> {
    let project_id = Uuid::parse_str(&project_id).map_err(|_| GroupApiError::bad_request())?;
    validate_create_body(&body)?;
    let key = read_idempotency_key(&headers)?;
    let binding = authorize_project_creator(&state, &actor, project_id).await?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    let command = ProjectWorktreeCreateCommand {
        tenant_id: actor.tenant_id,
        project_id,
        actor_id: actor.user_id,
        permission_snapshot_ref: permission_snapshot_ref(&binding),
        repository_id: body.repository_id,
        branch: body.branch.clone(),
        base_ref: body.base_ref.clone(),
        idempotency_key: key,
        request_fingerprint: request_fingerprint(
            actor.tenant_id,
            project_id,
            actor.user_id,
            "create",
            &body,
        )?,
        correlation_id,
    };
    let provider = state
        .worktree_lifecycle_provider
        .ok_or_else(|| GroupApiError::feature_unavailable("worktree_lifecycle_unavailable"))?;
    let receipt = provider.create(command).await.map_err(map_provider_error)?;
    validate_receipt(&receipt, project_id, body.repository_id, correlation_id)?;
    if receipt.branch != body.branch {
        return Err(GroupApiError::internal());
    }
    Ok((
        StatusCode::ACCEPTED,
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::to_value(receipt).map_err(|_| GroupApiError::internal())?),
    ))
}

async fn import_project_worktree(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(project_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<ImportWorktreeBody>,
) -> Result<impl IntoResponse, GroupApiError> {
    let project_id = Uuid::parse_str(&project_id).map_err(|_| GroupApiError::bad_request())?;
    if !validate_candidate_id(&body.candidate_id) {
        return Err(GroupApiError::invalid_request("invalid_import_candidate"));
    }
    let key = read_idempotency_key(&headers)?;
    let binding = authorize_project_creator(&state, &actor, project_id).await?;
    let correlation_id = body.correlation_id.unwrap_or_else(Uuid::new_v4);
    let command = ProjectWorktreeImportCommand {
        tenant_id: actor.tenant_id,
        project_id,
        actor_id: actor.user_id,
        permission_snapshot_ref: permission_snapshot_ref(&binding),
        repository_id: body.repository_id,
        candidate_id: body.candidate_id.clone(),
        idempotency_key: key,
        request_fingerprint: request_fingerprint(
            actor.tenant_id,
            project_id,
            actor.user_id,
            "import",
            &body,
        )?,
        correlation_id,
    };
    let provider = state
        .worktree_lifecycle_provider
        .ok_or_else(|| GroupApiError::feature_unavailable("worktree_lifecycle_unavailable"))?;
    let receipt = provider.import(command).await.map_err(map_provider_error)?;
    validate_receipt(&receipt, project_id, body.repository_id, correlation_id)?;
    Ok((
        StatusCode::ACCEPTED,
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::to_value(receipt).map_err(|_| GroupApiError::internal())?),
    ))
}

fn permission_snapshot_ref(binding: &ProjectBinding) -> String {
    format!("{}:v{}", binding.id, binding.version)
}

fn require_project_creator_role(role: &str) -> Result<(), GroupApiError> {
    match role {
        "tenant_admin" | "project_admin" | "developer" => Ok(()),
        _ => Err(GroupApiError::forbidden()),
    }
}

fn read_idempotency_key(headers: &HeaderMap) -> Result<String, GroupApiError> {
    let key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
        })
        .ok_or_else(|| GroupApiError::invalid_request("idempotency_key_required"))?;
    Ok(key.to_owned())
}

fn request_fingerprint<T: Serialize>(
    tenant_id: Uuid,
    project_id: Uuid,
    actor_id: Uuid,
    operation: &str,
    payload: &T,
) -> Result<[u8; 32], GroupApiError> {
    let bytes = serde_json::to_vec(&(tenant_id, project_id, actor_id, operation, payload))
        .map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(bytes).into())
}

fn validate_create_body(body: &CreateWorktreeBody) -> Result<(), GroupApiError> {
    if !validate_git_ref(&body.branch) || !validate_git_ref(&body.base_ref) {
        return Err(GroupApiError::invalid_request("invalid_worktree_ref"));
    }
    Ok(())
}

fn validate_git_ref(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 255
        || (value.starts_with('-') || value.starts_with('.') || value.starts_with('/'))
        || (value.ends_with('.') || value.ends_with('/'))
        || value.contains("..")
        || value.contains("//")
        || value.contains("@{")
        || value.bytes().any(|byte| {
            byte.is_ascii_control()
                || byte.is_ascii_whitespace()
                || b"\\:?*[]^~".contains(&byte)
                || !byte.is_ascii()
        })
    {
        return false;
    }
    value.split('/').all(|component| {
        !component.is_empty()
            && !component.starts_with('.')
            && !component.ends_with('.')
            && !component.ends_with(".lock")
    })
}

fn validate_candidate_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
}

fn validate_head_commit(value: &str) -> bool {
    (value.len() == 40 || value.len() == 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn validate_receipt(
    receipt: &WorktreeLifecycleReceipt,
    project_id: Uuid,
    repository_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), GroupApiError> {
    if receipt.operation_id.is_nil()
        || receipt.worktree_id.is_nil()
        || receipt.project_id != project_id
        || receipt.repository_id != repository_id
        || receipt.correlation_id != correlation_id
        || !validate_git_ref(&receipt.branch)
    {
        return Err(GroupApiError::internal());
    }
    Ok(())
}

fn map_provider_error(error: WorktreeLifecycleProviderError) -> GroupApiError {
    match error {
        WorktreeLifecycleProviderError::Unavailable => {
            GroupApiError::feature_unavailable("worktree_lifecycle_unavailable")
        }
        WorktreeLifecycleProviderError::NotFound => GroupApiError::not_found(),
        WorktreeLifecycleProviderError::Conflict => {
            GroupApiError::conflict("worktree_lifecycle_conflict")
        }
        WorktreeLifecycleProviderError::InvalidSpecification => {
            GroupApiError::invalid_request("invalid_worktree_specification")
        }
        WorktreeLifecycleProviderError::Internal => GroupApiError::internal(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_refs_reject_path_and_option_injection() {
        assert!(validate_git_ref("feature/worktree-cleanup"));
        assert!(validate_git_ref("origin/main"));
        assert!(!validate_git_ref("../outside"));
        assert!(!validate_git_ref("feature//nested"));
        assert!(!validate_git_ref("-c"));
        assert!(!validate_git_ref("feature/branch.lock"));
        assert!(!validate_git_ref("refs/heads/feature name"));
        assert!(!validate_git_ref("feature\\branch"));
    }

    #[test]
    fn candidate_id_is_opaque_and_path_free() {
        assert!(validate_candidate_id("candidate_01-Abc"));
        assert!(!validate_candidate_id("../repo"));
        assert!(!validate_candidate_id("C:\\repo"));
        assert!(!validate_candidate_id(""));
    }

    #[test]
    fn creator_role_is_project_member_writer() {
        assert!(require_project_creator_role("developer").is_ok());
        assert!(require_project_creator_role("project_admin").is_ok());
        assert!(require_project_creator_role("tenant_admin").is_ok());
        assert!(require_project_creator_role("viewer").is_err());
        assert!(require_project_creator_role("agent").is_err());
    }

    #[test]
    fn head_commit_is_full_hex_object_id() {
        assert!(validate_head_commit(&"a".repeat(40)));
        assert!(validate_head_commit(&"b".repeat(64)));
        assert!(!validate_head_commit("abc"));
        assert!(!validate_head_commit(&"g".repeat(40)));
    }

    #[test]
    fn project_repository_projection_rejects_invalid_or_duplicate_rows() {
        let repository_id = Uuid::new_v4();
        let rows = [ProjectWorktreeRepository {
            repository_id,
            name: "star-core".to_owned(),
            default_branch: "main".to_owned(),
        }];
        assert!(validate_project_repositories(&rows).is_ok());
        let duplicate = [rows[0].clone(), rows[0].clone()];
        assert!(validate_project_repositories(&duplicate).is_err());
        let unsafe_branch = [ProjectWorktreeRepository {
            repository_id: Uuid::new_v4(),
            name: "star-core".to_owned(),
            default_branch: "../outside".to_owned(),
        }];
        assert!(validate_project_repositories(&unsafe_branch).is_err());
        let host_path_name = [ProjectWorktreeRepository {
            repository_id: Uuid::new_v4(),
            name: "C:\\private\\repository".to_owned(),
            default_branch: "main".to_owned(),
        }];
        assert!(validate_project_repositories(&host_path_name).is_err());
    }

    #[test]
    fn lifecycle_routes_merge_with_existing_index_read_route() {
        let _combined = super::router().merge(super::super::worktrees::router());
    }
    #[test]
    fn request_dtos_reject_paths_and_urls() {
        let repository_id = Uuid::new_v4().to_string();
        assert!(serde_json::from_value::<ImportCandidatesRequest>(json!({"repository_id": repository_id, "path": "C:\\repo", "repo_url": "https://example.invalid/repo.git"})).is_err());
        assert!(serde_json::from_value::<CreateWorktreeBody>(json!({"repository_id": Uuid::new_v4(), "branch": "feature/a", "base_ref": "main", "path": "C:\\repo"})).is_err());
        assert!(serde_json::from_value::<ImportWorktreeBody>(json!({"repository_id": Uuid::new_v4(), "candidate_id": "candidate_01", "repo_url": "https://example.invalid/repo.git"})).is_err());
    }

    #[test]
    fn receipt_must_match_project_and_repository() {
        let project_id = Uuid::new_v4();
        let repository_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let mut receipt = WorktreeLifecycleReceipt {
            operation_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            project_id,
            repository_id,
            branch: "feature/a".to_owned(),
            state: WorktreeLifecycleState::Provisioning,
            accepted_at: Utc::now(),
            correlation_id,
        };
        assert!(validate_receipt(&receipt, project_id, repository_id, correlation_id).is_ok());
        receipt.repository_id = Uuid::new_v4();
        assert!(validate_receipt(&receipt, project_id, repository_id, correlation_id).is_err());
        receipt.repository_id = repository_id;
        receipt.correlation_id = Uuid::new_v4();
        assert!(validate_receipt(&receipt, project_id, repository_id, correlation_id).is_err());
    }
}
