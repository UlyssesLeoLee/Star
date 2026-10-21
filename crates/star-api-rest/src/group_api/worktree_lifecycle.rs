/*
@cypher schema=1 source_sha256=a705920a895bf5d80a26729424d271c3bdcd0874ae2c26f991b21ad81cdf4119
MERGE (self:File {path:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs"})
MERGE (provider:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleProvider",kind:"trait"})
MERGE (providerList:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleProvider.list_project_repositories",kind:"method"})
MERGE (providerCandidates:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleProvider.list_import_candidates",kind:"method"})
MERGE (providerCreate:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleProvider.create",kind:"method"})
MERGE (providerImport:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleProvider.import",kind:"method"})
MERGE (repository:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::ProjectWorktreeRepository",kind:"struct"})
MERGE (candidate:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::WorktreeImportCandidate",kind:"struct"})
MERGE (repositoryQuery:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeRepositoryQuery",kind:"struct"})
MERGE (candidateQuery:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeImportCandidatesQuery",kind:"struct"})
MERGE (createCommand:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeCreateCommand",kind:"struct"})
MERGE (importCommand:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeImportCommand",kind:"struct"})
MERGE (receipt:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleReceipt",kind:"struct"})
MERGE (state:Type {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::RunWorktreeLifecycleState",kind:"enum"})
MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::router",kind:"function"})
MERGE (legacy:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::reject_legacy_project_scope",kind:"function"})
MERGE (listRepositories:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::list_run_worktree_repositories",kind:"function"})
MERGE (validateRepositories:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_run_repositories",kind:"function"})
MERGE (displayName:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_display_name",kind:"function"})
MERGE (authorizeScope:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::authorize_run_worktree_scope",kind:"function"})
MERGE (repositoryBinding:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::require_run_repository_binding",kind:"function"})
MERGE (requireProvider:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::require_lifecycle_provider",kind:"function"})
MERGE (listCandidates:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::list_import_candidates",kind:"function"})
MERGE (createWorktree:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::create_run_worktree",kind:"function"})
MERGE (importWorktree:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::import_run_worktree",kind:"function"})
MERGE (readKey:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::read_idempotency_key",kind:"function"})
MERGE (fingerprint:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::request_fingerprint",kind:"function"})
MERGE (correlation:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::resolve_correlation_id",kind:"function"})
MERGE (validateCreate:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_create_body",kind:"function"})
MERGE (validateRef:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_git_ref",kind:"function"})
MERGE (validateCandidate:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_candidate_id",kind:"function"})
MERGE (validateHead:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_head_commit",kind:"function"})
MERGE (validateReceipt:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::validate_receipt",kind:"function"})
MERGE (mapError:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::map_provider_error",kind:"function"})
MERGE (tests:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests",kind:"module"})
MERGE (projectionTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::run_repository_projection_rejects_invalid_duplicate_and_unbound_run_rows",kind:"function"})
MERGE (hostlessTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::hostless_run_can_bind_local_repository_without_remote_branch",kind:"function"})
MERGE (bindingTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::worktree_mutation_fails_closed_without_run_repository_binding",kind:"function"})
MERGE (providerTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::lifecycle_provider_absence_is_service_unavailable",kind:"function"})
MERGE (correlationTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::idempotency_correlation_id_is_stable_and_operation_scoped",kind:"function"})
MERGE (nilCorrelationTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::explicit_nil_correlation_id_is_rejected",kind:"function"})
MERGE (receiptTest:Symbol {id:"crates/star-api-rest/src/group_api/worktree_lifecycle.rs::tests::receipt_must_match_run_project_repository_and_binding_state",kind:"function"})
MERGE (runAuthorization:Symbol {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::authorize_run_owned_task_scope",kind:"function"})
MERGE (runScope:Type {id:"crates/star-api-rest/src/group_api/engineering_runs.rs::RunOwnedTaskScope",kind:"struct"})
MERGE (apiState:Type {id:"crates/star-api-rest/src/group_api.rs::GroupApiState",kind:"struct"})
MERGE (self)-[:DEFINES]->(provider)
MERGE (self)-[:DEFINES]->(providerList)
MERGE (self)-[:DEFINES]->(providerCandidates)
MERGE (self)-[:DEFINES]->(providerCreate)
MERGE (self)-[:DEFINES]->(providerImport)
MERGE (self)-[:DEFINES]->(repository)
MERGE (self)-[:DEFINES]->(candidate)
MERGE (self)-[:DEFINES]->(repositoryQuery)
MERGE (self)-[:DEFINES]->(candidateQuery)
MERGE (self)-[:DEFINES]->(createCommand)
MERGE (self)-[:DEFINES]->(importCommand)
MERGE (self)-[:DEFINES]->(receipt)
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(legacy)
MERGE (self)-[:DEFINES]->(listRepositories)
MERGE (self)-[:DEFINES]->(validateRepositories)
MERGE (self)-[:DEFINES]->(displayName)
MERGE (self)-[:DEFINES]->(authorizeScope)
MERGE (self)-[:DEFINES]->(repositoryBinding)
MERGE (self)-[:DEFINES]->(requireProvider)
MERGE (self)-[:DEFINES]->(listCandidates)
MERGE (self)-[:DEFINES]->(createWorktree)
MERGE (self)-[:DEFINES]->(importWorktree)
MERGE (self)-[:DEFINES]->(readKey)
MERGE (self)-[:DEFINES]->(fingerprint)
MERGE (self)-[:DEFINES]->(correlation)
MERGE (self)-[:DEFINES]->(validateCreate)
MERGE (self)-[:DEFINES]->(validateRef)
MERGE (self)-[:DEFINES]->(validateCandidate)
MERGE (self)-[:DEFINES]->(validateHead)
MERGE (self)-[:DEFINES]->(validateReceipt)
MERGE (self)-[:DEFINES]->(mapError)
MERGE (self)-[:DEFINES]->(tests)
MERGE (self)-[:DEFINES]->(projectionTest)
MERGE (self)-[:DEFINES]->(hostlessTest)
MERGE (self)-[:DEFINES]->(bindingTest)
MERGE (self)-[:DEFINES]->(providerTest)
MERGE (self)-[:DEFINES]->(correlationTest)
MERGE (self)-[:DEFINES]->(nilCorrelationTest)
MERGE (self)-[:DEFINES]->(receiptTest)
MERGE (provider)-[:DEFINES]->(providerList)
MERGE (provider)-[:DEFINES]->(providerCandidates)
MERGE (provider)-[:DEFINES]->(providerCreate)
MERGE (provider)-[:DEFINES]->(providerImport)
MERGE (providerList)-[:USES_TYPE]->(repositoryQuery)
MERGE (providerList)-[:USES_TYPE]->(repository)
MERGE (providerCandidates)-[:USES_TYPE]->(candidateQuery)
MERGE (providerCandidates)-[:USES_TYPE]->(candidate)
MERGE (providerCreate)-[:USES_TYPE]->(createCommand)
MERGE (providerCreate)-[:USES_TYPE]->(receipt)
MERGE (providerImport)-[:USES_TYPE]->(importCommand)
MERGE (providerImport)-[:USES_TYPE]->(receipt)
MERGE (router)-[:EXPOSES]->(legacy)
MERGE (router)-[:EXPOSES]->(listRepositories)
MERGE (router)-[:EXPOSES]->(listCandidates)
MERGE (router)-[:EXPOSES]->(createWorktree)
MERGE (router)-[:EXPOSES]->(importWorktree)
MERGE (listRepositories)-[:USES_TYPE]->(apiState)
MERGE (listRepositories)-[:CALLS]->(authorizeScope)
MERGE (listRepositories)-[:CALLS]->(requireProvider)
MERGE (listRepositories)-[:CALLS]->(validateRepositories)
MERGE (listRepositories)-[:CALLS]->(providerList)
MERGE (authorizeScope)-[:CALLS]->(runAuthorization)
MERGE (authorizeScope)-[:USES_TYPE]->(runScope)
MERGE (authorizeScope)-[:USES_TYPE]->(apiState)
MERGE (listCandidates)-[:CALLS]->(authorizeScope)
MERGE (listCandidates)-[:CALLS]->(repositoryBinding)
MERGE (listCandidates)-[:CALLS]->(requireProvider)
MERGE (listCandidates)-[:CALLS]->(providerCandidates)
MERGE (listCandidates)-[:CALLS]->(validateCandidate)
MERGE (listCandidates)-[:CALLS]->(validateRef)
MERGE (listCandidates)-[:CALLS]->(displayName)
MERGE (listCandidates)-[:CALLS]->(validateHead)
MERGE (listCandidates)-[:USES_TYPE]->(apiState)
MERGE (createWorktree)-[:CALLS]->(validateCreate)
MERGE (createWorktree)-[:CALLS]->(readKey)
MERGE (createWorktree)-[:CALLS]->(authorizeScope)
MERGE (createWorktree)-[:CALLS]->(repositoryBinding)
MERGE (createWorktree)-[:CALLS]->(correlation)
MERGE (createWorktree)-[:CALLS]->(fingerprint)
MERGE (createWorktree)-[:CALLS]->(requireProvider)
MERGE (createWorktree)-[:CALLS]->(providerCreate)
MERGE (createWorktree)-[:CALLS]->(validateReceipt)
MERGE (createWorktree)-[:USES_TYPE]->(apiState)
MERGE (createWorktree)-[:USES_TYPE]->(createCommand)
MERGE (importWorktree)-[:CALLS]->(validateCandidate)
MERGE (importWorktree)-[:CALLS]->(readKey)
MERGE (importWorktree)-[:CALLS]->(authorizeScope)
MERGE (importWorktree)-[:CALLS]->(repositoryBinding)
MERGE (importWorktree)-[:CALLS]->(correlation)
MERGE (importWorktree)-[:CALLS]->(fingerprint)
MERGE (importWorktree)-[:CALLS]->(requireProvider)
MERGE (importWorktree)-[:CALLS]->(providerImport)
MERGE (importWorktree)-[:CALLS]->(validateReceipt)
MERGE (importWorktree)-[:USES_TYPE]->(apiState)
MERGE (importWorktree)-[:USES_TYPE]->(importCommand)
MERGE (validateRepositories)-[:CALLS]->(validateRef)
MERGE (validateRepositories)-[:CALLS]->(displayName)
MERGE (validateCreate)-[:CALLS]->(validateRef)
MERGE (validateReceipt)-[:CALLS]->(validateRef)
MERGE (validateReceipt)-[:USES_TYPE]->(receipt)
MERGE (validateReceipt)-[:USES_TYPE]->(state)
MERGE (requireProvider)-[:USES_TYPE]->(apiState)
MERGE (requireProvider)-[:USES_TYPE]->(provider)
MERGE (projectionTest)-[:CALLS]->(validateRepositories)
MERGE (hostlessTest)-[:CALLS]->(repositoryBinding)
MERGE (bindingTest)-[:CALLS]->(repositoryBinding)
MERGE (providerTest)-[:CALLS]->(requireProvider)
MERGE (correlationTest)-[:CALLS]->(correlation)
MERGE (nilCorrelationTest)-[:CALLS]->(correlation)
MERGE (receiptTest)-[:CALLS]->(validateReceipt)
MERGE (tests)-[:TESTS]->(router)
@endcypher
*/
//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"worktree_lifecycle.rs",type:"file",language:"rust"}),(m:Module {name:"worktree_lifecycle",type:"module",language:"rust"}),
//!   (provider:Interface {name:"RunWorktreeLifecycleProvider",type:"interface",language:"rust",visibility:"pub"}),(provider_list:Function {name:"RunWorktreeLifecycleProvider::list_project_repositories",type:"function",language:"rust",visibility:"pub"}),(provider_candidates:Function {name:"RunWorktreeLifecycleProvider::list_import_candidates",type:"function",language:"rust",visibility:"pub"}),(provider_create:Function {name:"RunWorktreeLifecycleProvider::create",type:"function",language:"rust",visibility:"pub"}),(provider_import:Function {name:"RunWorktreeLifecycleProvider::import",type:"function",language:"rust",visibility:"pub"}),
//!   (provider_error:Enum {name:"WorktreeLifecycleProviderError",type:"enum",language:"rust",visibility:"pub"}),(candidate:Class {name:"WorktreeImportCandidate",type:"class",language:"rust",visibility:"pub"}),(candidate_query:Class {name:"RunWorktreeImportCandidatesQuery",type:"class",language:"rust",visibility:"pub"}),(repository:Class {name:"ProjectWorktreeRepository",type:"class",language:"rust",visibility:"pub"}),(repository_query:Class {name:"RunWorktreeRepositoryQuery",type:"class",language:"rust",visibility:"pub"}),(create_command:Class {name:"RunWorktreeCreateCommand",type:"class",language:"rust",visibility:"pub"}),(import_command:Class {name:"RunWorktreeImportCommand",type:"class",language:"rust",visibility:"pub"}),(receipt:Class {name:"RunWorktreeLifecycleReceipt",type:"class",language:"rust",visibility:"pub"}),(state:Enum {name:"RunWorktreeLifecycleState",type:"enum",language:"rust",visibility:"pub"}),
//!   (router:Function {name:"router",type:"function",language:"rust",visibility:"pub"}),(legacy:Function {name:"reject_legacy_project_scope",type:"function",language:"rust",visibility:"private"}),(list_repositories:Function {name:"list_run_worktree_repositories",type:"function",language:"rust",visibility:"private"}),(validate_repositories:Function {name:"validate_run_repositories",type:"function",language:"rust",visibility:"private"}),(display_name:Function {name:"validate_display_name",type:"function",language:"rust",visibility:"private"}),(authorize_scope:Function {name:"authorize_run_worktree_scope",type:"function",language:"rust",visibility:"private"}),(repository_binding:Function {name:"require_run_repository_binding",type:"function",language:"rust",visibility:"private"}),(require_provider:Function {name:"require_lifecycle_provider",type:"function",language:"rust",visibility:"private"}),
//!   (list_candidates:Function {name:"list_import_candidates",type:"function",language:"rust",visibility:"private"}),(create_worktree:Function {name:"create_run_worktree",type:"function",language:"rust",visibility:"private"}),(import_worktree:Function {name:"import_run_worktree",type:"function",language:"rust",visibility:"private"}),(writer_role:Function {name:"require_project_creator_role",type:"function",language:"rust",visibility:"private"}),(idempotency:Function {name:"read_idempotency_key",type:"function",language:"rust",visibility:"private"}),(fingerprint:Function {name:"request_fingerprint",type:"function",language:"rust",visibility:"private"}),(correlation:Function {name:"resolve_correlation_id",type:"function",language:"rust",visibility:"private"}),(validate_create:Function {name:"validate_create_body",type:"function",language:"rust",visibility:"private"}),(validate_ref:Function {name:"validate_git_ref",type:"function",language:"rust",visibility:"private"}),(validate_candidate:Function {name:"validate_candidate_id",type:"function",language:"rust",visibility:"private"}),(validate_head:Function {name:"validate_head_commit",type:"function",language:"rust",visibility:"private"}),(validate_receipt:Function {name:"validate_receipt",type:"function",language:"rust",visibility:"private"}),(map_error:Function {name:"map_provider_error",type:"function",language:"rust",visibility:"private"}),
//!   (tests:Module {name:"worktree_lifecycle_tests",type:"module",language:"rust"}),(ref_test:Function {name:"git_refs_reject_path_and_option_injection",type:"function",language:"rust"}),(candidate_test:Function {name:"candidate_id_is_opaque_and_path_free",type:"function",language:"rust"}),(role_test:Function {name:"creator_role_is_project_member_writer",type:"function",language:"rust"}),(head_test:Function {name:"head_commit_is_full_hex_object_id",type:"function",language:"rust"}),(projection_test:Function {name:"run_repository_projection_rejects_invalid_duplicate_and_unbound_run_rows",type:"function",language:"rust"}),(hostless_test:Function {name:"hostless_run_can_bind_local_repository_without_remote_branch",type:"function",language:"rust"}),(binding_test:Function {name:"worktree_mutation_fails_closed_without_run_repository_binding",type:"function",language:"rust"}),(provider_test:Function {name:"lifecycle_provider_absence_is_service_unavailable",type:"function",language:"rust"}),(correlation_test:Function {name:"idempotency_correlation_id_is_stable_and_operation_scoped",type:"function",language:"rust"}),(nil_correlation_test:Function {name:"explicit_nil_correlation_id_is_rejected",type:"function",language:"rust"}),(route_test:Function {name:"lifecycle_routes_merge_with_existing_index_read_route",type:"function",language:"rust"}),(dto_test:Function {name:"request_dtos_reject_paths_and_urls",type:"function",language:"rust"}),(receipt_test:Function {name:"receipt_must_match_run_project_repository_and_binding_state",type:"function",language:"rust"});
//! MATCH (run_authorize:Function {name:"authorize_run_owned_task_scope",type:"function"}),(run_scope:Class {name:"RunOwnedTaskScope",type:"class"});
//! CREATE (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(provider),(provider)-[:HAS_METHOD]->(provider_list),(provider)-[:HAS_METHOD]->(provider_candidates),(provider)-[:HAS_METHOD]->(provider_create),(provider)-[:HAS_METHOD]->(provider_import),(m)-[:CONTAINS]->(provider_error),(m)-[:CONTAINS]->(candidate),(m)-[:CONTAINS]->(candidate_query),(m)-[:CONTAINS]->(repository),(m)-[:CONTAINS]->(repository_query),(m)-[:CONTAINS]->(create_command),(m)-[:CONTAINS]->(import_command),(m)-[:CONTAINS]->(receipt),(m)-[:CONTAINS]->(state),(m)-[:CONTAINS]->(router),
//!        (m)-[:CONTAINS]->(legacy),(m)-[:CONTAINS]->(list_repositories),(m)-[:CONTAINS]->(validate_repositories),(m)-[:CONTAINS]->(display_name),(m)-[:CONTAINS]->(authorize_scope),(m)-[:CONTAINS]->(repository_binding),(m)-[:CONTAINS]->(require_provider),(m)-[:CONTAINS]->(list_candidates),(m)-[:CONTAINS]->(create_worktree),(m)-[:CONTAINS]->(import_worktree),(m)-[:CONTAINS]->(writer_role),(m)-[:CONTAINS]->(idempotency),(m)-[:CONTAINS]->(fingerprint),(m)-[:CONTAINS]->(correlation),(m)-[:CONTAINS]->(validate_create),(m)-[:CONTAINS]->(validate_ref),(m)-[:CONTAINS]->(validate_candidate),(m)-[:CONTAINS]->(validate_head),(m)-[:CONTAINS]->(validate_receipt),(m)-[:CONTAINS]->(map_error),
//!        (router)-[:EXPOSES]->(legacy),(router)-[:EXPOSES]->(list_repositories),(router)-[:EXPOSES]->(list_candidates),(router)-[:EXPOSES]->(create_worktree),(router)-[:EXPOSES]->(import_worktree),
//!        (list_repositories)-[:CALLS]->(authorize_scope),(list_repositories)-[:CALLS]->(require_provider),(list_repositories)-[:CALLS]->(validate_repositories),(list_repositories)-[:CALLS]->(provider_list),(list_candidates)-[:CALLS]->(authorize_scope),(list_candidates)-[:CALLS]->(repository_binding),(list_candidates)-[:CALLS]->(require_provider),(list_candidates)-[:CALLS]->(provider_candidates),(list_candidates)-[:CALLS]->(validate_candidate),(list_candidates)-[:CALLS]->(validate_ref),(list_candidates)-[:CALLS]->(display_name),(list_candidates)-[:CALLS]->(validate_head),
//!        (create_worktree)-[:CALLS]->(validate_create),(create_worktree)-[:CALLS]->(idempotency),(create_worktree)-[:CALLS]->(authorize_scope),(create_worktree)-[:CALLS]->(repository_binding),(create_worktree)-[:CALLS]->(correlation),(create_worktree)-[:CALLS]->(fingerprint),(create_worktree)-[:CALLS]->(require_provider),(create_worktree)-[:CALLS]->(provider_create),(create_worktree)-[:CALLS]->(validate_receipt),(create_worktree)-[:USES_TYPE]->(create_command),(import_worktree)-[:CALLS]->(validate_candidate),(import_worktree)-[:CALLS]->(idempotency),(import_worktree)-[:CALLS]->(authorize_scope),(import_worktree)-[:CALLS]->(repository_binding),(import_worktree)-[:CALLS]->(correlation),(import_worktree)-[:CALLS]->(fingerprint),(import_worktree)-[:CALLS]->(require_provider),(import_worktree)-[:CALLS]->(provider_import),(import_worktree)-[:CALLS]->(validate_receipt),(import_worktree)-[:USES_TYPE]->(import_command),
//!        (authorize_scope)-[:CALLS]->(run_authorize),(authorize_scope)-[:USES_TYPE]->(run_scope),(authorize_scope)-[:CALLS]->(writer_role),(validate_repositories)-[:CALLS]->(display_name),(validate_repositories)-[:CALLS]->(validate_ref),(validate_create)-[:CALLS]->(validate_ref),(validate_receipt)-[:CALLS]->(validate_ref),(map_error)-[:USES_TYPE]->(provider_error),
//!        (m)-[:CONTAINS]->(tests),(tests)-[:CONTAINS]->(ref_test),(tests)-[:CONTAINS]->(candidate_test),(tests)-[:CONTAINS]->(role_test),(tests)-[:CONTAINS]->(head_test),(tests)-[:CONTAINS]->(projection_test),(tests)-[:CONTAINS]->(hostless_test),(tests)-[:CONTAINS]->(binding_test),(tests)-[:CONTAINS]->(provider_test),(tests)-[:CONTAINS]->(correlation_test),(tests)-[:CONTAINS]->(nil_correlation_test),(tests)-[:CONTAINS]->(route_test),(tests)-[:CONTAINS]->(dto_test),(tests)-[:CONTAINS]->(receipt_test),
//!        (ref_test)-[:CALLS]->(validate_ref),(candidate_test)-[:CALLS]->(validate_candidate),(role_test)-[:CALLS]->(writer_role),(head_test)-[:CALLS]->(validate_head),(projection_test)-[:CALLS]->(validate_repositories),(hostless_test)-[:CALLS]->(repository_binding),(binding_test)-[:CALLS]->(repository_binding),(provider_test)-[:CALLS]->(require_provider),(correlation_test)-[:CALLS]->(correlation),(nil_correlation_test)-[:CALLS]->(correlation),(route_test)-[:CALLS]->(router),(dto_test)-[:CALLS]->(validate_create),(receipt_test)-[:CALLS]->(validate_receipt);
//! CYPHER STRUCTURAL MANIFEST END


use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, VARY},
        HeaderMap, StatusCode,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::AuthUser;

use super::{
    require_scope, set_tenant, validate_actor, AuthenticatedUser, GroupApiError, GroupApiState,
};
use super::engineering_runs::{authorize_run_owned_task_scope, RunOwnedTaskScope};

/// An opaque Git Worktree candidate returned for a Run-bound Repository by the trusted host registry.
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
pub struct RunWorktreeImportCandidatesQuery {
    pub tenant_id: Uuid,
    pub engineering_run_id: Uuid,
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
pub struct RunWorktreeRepositoryQuery {
    pub tenant_id: Uuid,
    pub engineering_run_id: Uuid,
    pub project_id: Uuid,
    pub actor_id: Uuid,
    pub permission_snapshot_ref: String,
    pub repository_id: Option<Uuid>,
}

/// A Run-authorized Worktree creation request. Physical paths and remote URLs are
/// resolved exclusively by the trusted lifecycle provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunWorktreeCreateCommand {
    pub tenant_id: Uuid,
    pub engineering_run_id: Uuid,
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
pub struct RunWorktreeImportCommand {
    pub tenant_id: Uuid,
    pub engineering_run_id: Uuid,
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
pub struct RunWorktreeLifecycleReceipt {
    pub operation_id: Uuid,
    pub worktree_id: Uuid,
    pub engineering_run_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub branch: String,
    pub state: RunWorktreeLifecycleState,
    /// Present only after the Run↔Worktree owner confirms the active binding.
    pub run_binding_id: Option<Uuid>,
    /// Positive revision paired with `run_binding_id`; absent while binding is pending.
    pub run_binding_version: Option<i32>,
    pub accepted_at: DateTime<Utc>,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunWorktreeLifecycleState {
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

/// Trusted host adapter for Run-scoped repository discovery and Git Worktree lifecycle.
/// Discovery must be filtered to the current Project and Run scope. Implementations must
/// recheck current Project and Run grants plus the Run's repository binding immediately
/// before a Git side effect; re-resolve opaque import candidates against that binding;
/// derive local paths only from trusted server-side repository
/// configuration; and persist an idempotent owner-local operation, audit, and outbox fact
/// before returning a receipt. Run↔Worktree binding remains owned by the existing binding
/// owner; this provider must not write that owner's tables directly. `Ready` is valid only
/// after the binding owner confirms the active binding. Never trust the permission
/// snapshot as a current authorization grant.
#[async_trait]
pub trait RunWorktreeLifecycleProvider: Send + Sync {
    async fn list_project_repositories(
        &self,
        query: RunWorktreeRepositoryQuery,
    ) -> Result<Vec<ProjectWorktreeRepository>, WorktreeLifecycleProviderError>;

    async fn list_import_candidates(
        &self,
        query: RunWorktreeImportCandidatesQuery,
    ) -> Result<Vec<WorktreeImportCandidate>, WorktreeLifecycleProviderError>;

    async fn create(
        &self,
        command: RunWorktreeCreateCommand,
    ) -> Result<RunWorktreeLifecycleReceipt, WorktreeLifecycleProviderError>;

    async fn import(
        &self,
        command: RunWorktreeImportCommand,
    ) -> Result<RunWorktreeLifecycleReceipt, WorktreeLifecycleProviderError>;
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
        // Preserve the legacy Project-scoped surface while clients migrate to Run scope.
        // It remains unavailable because a Project alone cannot establish a Worktree owner.
        .route(
            "/api/v1/projects/{project_id}/worktree-repositories",
            get(reject_legacy_project_scope),
        )
        .route(
            "/api/v1/projects/{project_id}/worktree-import-candidates",
            get(reject_legacy_project_scope),
        )
        .route(
            "/api/v1/projects/{project_id}/worktrees",
            post(reject_legacy_project_scope),
        )
        .route(
            "/api/v1/projects/{project_id}/worktrees/import",
            post(reject_legacy_project_scope),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/worktree-repositories",
            get(list_run_worktree_repositories),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/worktree-import-candidates",
            get(list_import_candidates),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/worktrees",
            post(create_run_worktree),
        )
        .route(
            "/api/v1/engineering-runs/{engineering_run_id}/worktrees/import",
            post(import_run_worktree),
        )
}

async fn reject_legacy_project_scope(
    AuthenticatedUser(_actor): AuthenticatedUser,
) -> Result<StatusCode, GroupApiError> {
    Err(GroupApiError::feature_unavailable(
        "worktree_lifecycle_unavailable",
    ))
}

async fn list_run_worktree_repositories(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, GroupApiError> {
    let run_id = Uuid::parse_str(&run_id).map_err(|_| GroupApiError::bad_request())?;
    let scope = authorize_run_worktree_scope(&state, &actor, run_id, "worktree:read", false).await?;
    let provider = require_lifecycle_provider(state.run_worktree_lifecycle_provider.clone())?;
    let repositories = provider
        .list_project_repositories(RunWorktreeRepositoryQuery {
            tenant_id: actor.tenant_id,
            engineering_run_id: run_id,
            project_id: scope.project_id,
            actor_id: actor.user_id,
            permission_snapshot_ref: scope.permission_snapshot_ref.clone(),
            repository_id: scope.repository_id,
        })
        .await
        .map_err(map_provider_error)?;
    validate_run_repositories(&repositories, scope.repository_id)?;
    Ok((
        [(CACHE_CONTROL, "no-store"), (VARY, "Authorization")],
        Json(json!({ "project_id": scope.project_id, "engineering_run_id": run_id, "repositories": repositories })),
    ))
}

fn validate_run_repositories(
    repositories: &[ProjectWorktreeRepository],
    run_repository_id: Option<Uuid>,
) -> Result<(), GroupApiError> {
    let mut seen = HashSet::new();
    if repositories.len() > 100
        || repositories.iter().any(|repository| {
            repository.repository_id.is_nil()
                || run_repository_id.is_some_and(|bound| bound != repository.repository_id)
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

async fn authorize_run_worktree_scope(
    state: &GroupApiState,
    actor: &AuthUser,
    run_id: Uuid,
    required_scope: &'static str,
    write: bool,
) -> Result<RunOwnedTaskScope, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, required_scope)?;
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let scope = authorize_run_owned_task_scope(&mut tx, actor, run_id).await?;
    if write {
        require_project_creator_role(&scope.role)?;
    }
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(scope)
}

fn require_run_repository_binding(
    scope: &RunOwnedTaskScope,
    requested_repository_id: Uuid,
) -> Result<(), GroupApiError> {
    match scope.repository_id {
        Some(bound_repository_id) if bound_repository_id == requested_repository_id => Ok(()),
        Some(_) => Err(GroupApiError::conflict("worktree_repository_mismatch")),
        None => Err(GroupApiError::conflict("run_repository_binding_required")),
    }
}

fn require_lifecycle_provider(
    provider: Option<Arc<dyn RunWorktreeLifecycleProvider>>,
) -> Result<Arc<dyn RunWorktreeLifecycleProvider>, GroupApiError> {
    provider.ok_or_else(|| GroupApiError::feature_unavailable("worktree_lifecycle_unavailable"))
}

async fn list_import_candidates(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<ImportCandidatesRequest>,
) -> Result<impl IntoResponse, GroupApiError> {
    let run_id = Uuid::parse_str(&run_id).map_err(|_| GroupApiError::bad_request())?;
    let repository_id =
        Uuid::parse_str(&query.repository_id).map_err(|_| GroupApiError::bad_request())?;
    let limit = query.limit.unwrap_or(25).clamp(1, 50);
    let scope = authorize_run_worktree_scope(&state, &actor, run_id, "worktree:create", true).await?;
    require_run_repository_binding(&scope, repository_id)?;
    let provider = require_lifecycle_provider(state.run_worktree_lifecycle_provider.clone())?;
    let provider_query = RunWorktreeImportCandidatesQuery {
        tenant_id: actor.tenant_id,
        engineering_run_id: run_id,
        project_id: scope.project_id,
        actor_id: actor.user_id,
        permission_snapshot_ref: scope.permission_snapshot_ref.clone(),
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
            "project_id": scope.project_id,
            "engineering_run_id": run_id,
            "repository_id": repository_id,
            "candidates": candidates,
        })),
    ))
}

async fn create_run_worktree(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<CreateWorktreeBody>,
) -> Result<impl IntoResponse, GroupApiError> {
    let run_id = Uuid::parse_str(&run_id).map_err(|_| GroupApiError::bad_request())?;
    validate_create_body(&body)?;
    let key = read_idempotency_key(&headers)?;
    let scope = authorize_run_worktree_scope(&state, &actor, run_id, "worktree:create", true).await?;
    require_run_repository_binding(&scope, body.repository_id)?;
    let correlation_id = resolve_correlation_id(
        body.correlation_id,
        actor.tenant_id,
        run_id,
        actor.user_id,
        "create",
        &key,
    )?;
    let command = RunWorktreeCreateCommand {
        tenant_id: actor.tenant_id,
        engineering_run_id: run_id,
        project_id: scope.project_id,
        actor_id: actor.user_id,
        permission_snapshot_ref: scope.permission_snapshot_ref.clone(),
        repository_id: body.repository_id,
        branch: body.branch.clone(),
        base_ref: body.base_ref.clone(),
        idempotency_key: key,
        request_fingerprint: request_fingerprint(
            actor.tenant_id,
            run_id,
            actor.user_id,
            "create",
            &body,
        )?,
        correlation_id,
    };
    let provider = require_lifecycle_provider(state.run_worktree_lifecycle_provider.clone())?;
    let receipt = provider.create(command).await.map_err(map_provider_error)?;
    validate_receipt(&receipt, run_id, scope.project_id, body.repository_id, correlation_id)?;
    if receipt.branch != body.branch {
        return Err(GroupApiError::internal());
    }
    Ok((
        StatusCode::ACCEPTED,
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::to_value(receipt).map_err(|_| GroupApiError::internal())?),
    ))
}

async fn import_run_worktree(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(run_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<ImportWorktreeBody>,
) -> Result<impl IntoResponse, GroupApiError> {
    let run_id = Uuid::parse_str(&run_id).map_err(|_| GroupApiError::bad_request())?;
    if !validate_candidate_id(&body.candidate_id) {
        return Err(GroupApiError::invalid_request("invalid_import_candidate"));
    }
    let key = read_idempotency_key(&headers)?;
    let scope = authorize_run_worktree_scope(&state, &actor, run_id, "worktree:create", true).await?;
    require_run_repository_binding(&scope, body.repository_id)?;
    let correlation_id = resolve_correlation_id(
        body.correlation_id,
        actor.tenant_id,
        run_id,
        actor.user_id,
        "import",
        &key,
    )?;
    let command = RunWorktreeImportCommand {
        tenant_id: actor.tenant_id,
        engineering_run_id: run_id,
        project_id: scope.project_id,
        actor_id: actor.user_id,
        permission_snapshot_ref: scope.permission_snapshot_ref.clone(),
        repository_id: body.repository_id,
        candidate_id: body.candidate_id.clone(),
        idempotency_key: key,
        request_fingerprint: request_fingerprint(
            actor.tenant_id,
            run_id,
            actor.user_id,
            "import",
            &body,
        )?,
        correlation_id,
    };
    let provider = require_lifecycle_provider(state.run_worktree_lifecycle_provider.clone())?;
    let receipt = provider.import(command).await.map_err(map_provider_error)?;
    validate_receipt(&receipt, run_id, scope.project_id, body.repository_id, correlation_id)?;
    Ok((
        StatusCode::ACCEPTED,
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::to_value(receipt).map_err(|_| GroupApiError::internal())?),
    ))
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

fn resolve_correlation_id(
    explicit: Option<Uuid>,
    tenant_id: Uuid,
    run_id: Uuid,
    actor_id: Uuid,
    operation: &str,
    idempotency_key: &str,
) -> Result<Uuid, GroupApiError> {
    if let Some(correlation_id) = explicit {
        return if correlation_id.is_nil() {
            Err(GroupApiError::invalid_request("invalid_correlation_id"))
        } else {
            Ok(correlation_id)
        };
    }

    let material = serde_json::to_vec(&(
        tenant_id,
        run_id,
        actor_id,
        operation,
        idempotency_key,
    ))
    .map_err(|_| GroupApiError::internal())?;
    let digest = Sha256::digest(material);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(Uuid::from_bytes(bytes))
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
    receipt: &RunWorktreeLifecycleReceipt,
    run_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), GroupApiError> {
    if receipt.operation_id.is_nil()
        || receipt.worktree_id.is_nil()
        || receipt.engineering_run_id != run_id
        || receipt.project_id != project_id
        || receipt.repository_id != repository_id
        || receipt.correlation_id != correlation_id
        || !validate_git_ref(&receipt.branch)
        || match receipt.state {
            RunWorktreeLifecycleState::Provisioning => {
                receipt.run_binding_id.is_some() || receipt.run_binding_version.is_some()
            }
            RunWorktreeLifecycleState::Ready => {
                receipt.run_binding_id.is_none()
                    || receipt.run_binding_id.is_some_and(Uuid::is_nil)
                    || receipt.run_binding_version.is_none_or(|version| version <= 0)
            }
        }
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
    fn run_repository_projection_rejects_invalid_duplicate_and_unbound_run_rows() {
        let repository_id = Uuid::new_v4();
        let rows = [ProjectWorktreeRepository {
            repository_id,
            name: "star-core".to_owned(),
            default_branch: "main".to_owned(),
        }];
        assert!(validate_run_repositories(&rows, None).is_ok());
        assert!(validate_run_repositories(&rows, Some(repository_id)).is_ok());
        assert!(validate_run_repositories(&rows, Some(Uuid::new_v4())).is_err());
        let duplicate = [rows[0].clone(), rows[0].clone()];
        assert!(validate_run_repositories(&duplicate, None).is_err());
        let unsafe_branch = [ProjectWorktreeRepository {
            repository_id: Uuid::new_v4(),
            name: "star-core".to_owned(),
            default_branch: "../outside".to_owned(),
        }];
        assert!(validate_run_repositories(&unsafe_branch, None).is_err());
        let host_path_name = [ProjectWorktreeRepository {
            repository_id: Uuid::new_v4(),
            name: "C:\\private\\repository".to_owned(),
            default_branch: "main".to_owned(),
        }];
        assert!(validate_run_repositories(&host_path_name, None).is_err());
    }

    #[test]
    fn hostless_run_can_bind_local_repository_without_remote_branch() {
        let repository_id = Uuid::new_v4();
        let scope = RunOwnedTaskScope {
            engineering_run_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: Some(repository_id),
            branch_id: None,
            role: "developer".to_owned(),
            permission_snapshot_ref: "run-grant:v1".to_owned(),
        };

        assert!(require_run_repository_binding(&scope, repository_id).is_ok());
    }

    #[test]
    fn worktree_mutation_fails_closed_without_run_repository_binding() {
        let scope = RunOwnedTaskScope {
            engineering_run_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: None,
            branch_id: None,
            role: "developer".to_owned(),
            permission_snapshot_ref: "run-grant:v1".to_owned(),
        };

        let error = require_run_repository_binding(&scope, Uuid::new_v4())
            .err()
            .expect("unbound Run must fail closed");
        assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    }

    #[test]
    fn lifecycle_provider_absence_is_service_unavailable() {
        let error = require_lifecycle_provider(None)
            .err()
            .expect("provider absence must fail closed");
        assert_eq!(error.into_response().status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn idempotency_correlation_id_is_stable_and_operation_scoped() {
        let tenant_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();

        let first = resolve_correlation_id(
            None,
            tenant_id,
            run_id,
            actor_id,
            "create",
            "worktree-create-01",
        )
        .unwrap();
        let replay = resolve_correlation_id(
            None,
            tenant_id,
            run_id,
            actor_id,
            "create",
            "worktree-create-01",
        )
        .unwrap();
        let import = resolve_correlation_id(
            None,
            tenant_id,
            run_id,
            actor_id,
            "import",
            "worktree-create-01",
        )
        .unwrap();

        assert_eq!(first, replay);
        assert_ne!(first, import);
    }

    #[test]
    fn explicit_nil_correlation_id_is_rejected() {
        assert!(resolve_correlation_id(
            Some(Uuid::nil()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "create",
            "worktree-create-01",
        )
        .is_err());
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
    fn receipt_must_match_run_project_repository_and_binding_state() {
        let project_id = Uuid::new_v4();
        let repository_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        let mut receipt = RunWorktreeLifecycleReceipt {
            operation_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            engineering_run_id: run_id,
            project_id,
            repository_id,
            branch: "feature/a".to_owned(),
            state: RunWorktreeLifecycleState::Provisioning,
            run_binding_id: None,
            run_binding_version: None,
            accepted_at: Utc::now(),
            correlation_id,
        };
        assert!(validate_receipt(&receipt, run_id, project_id, repository_id, correlation_id).is_ok());
        receipt.engineering_run_id = Uuid::new_v4();
        assert!(validate_receipt(&receipt, run_id, project_id, repository_id, correlation_id).is_err());
        receipt.engineering_run_id = run_id;
        receipt.state = RunWorktreeLifecycleState::Ready;
        assert!(validate_receipt(&receipt, run_id, project_id, repository_id, correlation_id).is_err());
        receipt.run_binding_id = Some(Uuid::new_v4());
        receipt.run_binding_version = Some(1);
        assert!(validate_receipt(&receipt, run_id, project_id, repository_id, correlation_id).is_ok());
        receipt.repository_id = Uuid::new_v4();
        assert!(validate_receipt(&receipt, run_id, project_id, repository_id, correlation_id).is_err());
        receipt.repository_id = repository_id;
        receipt.correlation_id = Uuid::new_v4();
        assert!(validate_receipt(&receipt, run_id, project_id, repository_id, correlation_id).is_err());
    }
}
