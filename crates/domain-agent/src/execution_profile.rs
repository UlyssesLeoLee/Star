//! CYPHER STRUCTURE MANIFEST
//! CREATE
//!   (f:File {name:"execution_profile.rs",type:"file",language:"rust"}),
//!   (draft:Class {name:"AgentExecutionProfileDraft",type:"class",language:"rust"}),
//!   (document:Class {name:"AgentExecutionProfileDocument",type:"class",language:"rust"}),
//!   (verified:Class {name:"VerifiedAgentExecutionProfile",type:"class",language:"rust"}),
//!   (provider_entry:Class {name:"ExecutionProviderCatalogEntry",type:"class",language:"rust"}),
//!   (skill_entry:Class {name:"ExecutionSkillCatalogEntry",type:"class",language:"rust"}),
//!   (worktree_state:Class {name:"WorktreeRunState",type:"class",language:"rust"}),
//!   (admission_facts:Class {name:"ExecutionProfileAdmissionFacts",type:"class",language:"rust"}),
//!   (resolved:Class {name:"ResolvedAgentExecutionProfile",type:"class",language:"rust"}),
//!   (resolver:Class {name:"ExecutionProfileResolver",type:"class",language:"rust"}),
//!   (scope:Class {name:"ExecutionProfileScope",type:"class",language:"rust"}),
//!   (provider:Class {name:"ProviderReference",type:"class",language:"rust"}),
//!   (memory:Class {name:"MemoryPolicySnapshot",type:"class",language:"rust"}),
//!   (memory_scope:Class {name:"MemoryReadScope",type:"class",language:"rust"}),
//!   (skill:Class {name:"SkillBindingSnapshot",type:"class",language:"rust"}),
//!   (context:Class {name:"ContextPolicySnapshot",type:"class",language:"rust"}),
//!   (validation:Class {name:"ValidationPolicySnapshot",type:"class",language:"rust"}),
//!   (loop:Class {name:"LoopBudgetSnapshot",type:"class",language:"rust"}),
//!   (resources:Class {name:"ResourceBudgetSnapshot",type:"class",language:"rust"}),
//!   (hookset:Class {name:"HookSetSnapshot",type:"class",language:"rust"}),
//!   (grants:Class {name:"GrantSnapshot",type:"class",language:"rust"}),
//!   (manifest:Class {name:"ProjectEngineeringManifestRef",type:"class",language:"rust"}),
//!   (error:Class {name:"ExecutionProfileError",type:"class",language:"rust"}),
//!   (seal:Function {name:"AgentExecutionProfileDraft::seal",type:"function",language:"rust"}),
//!   (verify:Function {name:"AgentExecutionProfileDocument::verify",type:"function",language:"rust"}),
//!   (decode:Function {name:"AgentExecutionProfileDocument::decode_and_verify",type:"function",language:"rust"}),
//!   (document_ref:Function {name:"VerifiedAgentExecutionProfile::document",type:"function",language:"rust"}),
//!   (scope_check:Function {name:"VerifiedAgentExecutionProfile::validate_for_scope",type:"function",language:"rust"}),
//!   (resolve:Function {name:"ExecutionProfileResolver::resolve",type:"function",language:"rust"}),
//!   (resolved_document:Function {name:"ResolvedAgentExecutionProfile::document",type:"function",language:"rust"}),
//!   (resolved_digest:Function {name:"ResolvedAgentExecutionProfile::content_digest",type:"function",language:"rust"}),
//!   (provider_catalog_check:Function {name:"validate_provider_catalog",type:"function",language:"rust"}),
//!   (skill_catalog_check:Function {name:"validate_skill_catalog",type:"function",language:"rust"}),
//!   (resolve_provider:Function {name:"resolve_provider",type:"function",language:"rust"}),
//!   (resolve_skill:Function {name:"resolve_skill",type:"function",language:"rust"}),
//!   (validate:Function {name:"validate_profile",type:"function",language:"rust"}),
//!   (provider_check:Function {name:"validate_provider",type:"function",language:"rust"}),
//!   (budget_check:Function {name:"validate_budgets",type:"function",language:"rust"}),
//!   (grant_check:Function {name:"check_grants",type:"function",language:"rust"}),
//!   (labels_check:Function {name:"validate_labels",type:"function",language:"rust"}),
//!   (identifier_check:Function {name:"validate_identifier",type:"function",language:"rust"}),
//!   (digest_check:Function {name:"validate_digest",type:"function",language:"rust"}),
//!   (digest:Function {name:"compute_digest",type:"function",language:"rust"}),
//!   (fixture_digest:Function {name:"tests::digest",type:"function",language:"rust"}),
//!   (fixture_provider:Function {name:"tests::provider",type:"function",language:"rust"}),
//!   (fixture:Function {name:"tests::valid_draft",type:"function",language:"rust"}),
//!   (test_roundtrip:Function {name:"tests::verified_profile_round_trips_and_binds_scope",type:"function",language:"rust"}),
//!   (test_tamper:Function {name:"tests::tampering_and_cross_scope_execution_are_rejected",type:"function",language:"rust"}),
//!   (test_limits:Function {name:"tests::missing_memory_grants_and_unbounded_limits_fail_closed",type:"function",language:"rust"}),
//!   (test_memory_scope:Function {name:"tests::scoped_memory_and_context_requirements_are_enforced",type:"function",language:"rust"}),
//!   (test_canonical:Function {name:"tests::noncanonical_lists_and_oversized_documents_are_rejected",type:"function",language:"rust"}),
//!   (test_resolver_success:Function {name:"tests::resolver_admits_only_exact_current_profile_dependencies",type:"function",language:"rust"}),
//!   (test_provider_drift:Function {name:"tests::resolver_rejects_provider_drift_and_invalid_catalog_order",type:"function",language:"rust"}),
//!   (test_grant_lifecycle:Function {name:"tests::resolver_rejects_expired_or_changed_grants_and_draining_worktrees",type:"function",language:"rust"}),
//!   (test_skill_hook:Function {name:"tests::resolver_rejects_revoked_skills_and_changed_hooksets",type:"function",language:"rust"}),
//!   (test_catalog_bounds:Function {name:"tests::execution_catalogs_enforce_entry_bounds",type:"function",language:"rust"}),
//!   (catalogs_fixture:Function {name:"tests::catalogs_from",type:"function",language:"rust"}),
//!   (admission_fixture:Function {name:"tests::admission_facts",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(draft),
//!   (f)-[:CONTAINS]->(document),
//!   (f)-[:CONTAINS]->(verified),
//!   (f)-[:CONTAINS]->(provider_entry),
//!   (f)-[:CONTAINS]->(skill_entry),
//!   (f)-[:CONTAINS]->(worktree_state),
//!   (f)-[:CONTAINS]->(admission_facts),
//!   (f)-[:CONTAINS]->(resolved),
//!   (f)-[:CONTAINS]->(resolver),
//!   (f)-[:CONTAINS]->(scope),
//!   (f)-[:CONTAINS]->(provider),
//!   (f)-[:CONTAINS]->(memory),
//!   (f)-[:CONTAINS]->(memory_scope),
//!   (f)-[:CONTAINS]->(skill),
//!   (f)-[:CONTAINS]->(context),
//!   (f)-[:CONTAINS]->(validation),
//!   (f)-[:CONTAINS]->(loop),
//!   (f)-[:CONTAINS]->(resources),
//!   (f)-[:CONTAINS]->(hookset),
//!   (f)-[:CONTAINS]->(grants),
//!   (f)-[:CONTAINS]->(manifest),
//!   (f)-[:CONTAINS]->(error),
//!   (f)-[:CONTAINS]->(validate),
//!   (f)-[:CONTAINS]->(provider_check),
//!   (f)-[:CONTAINS]->(budget_check),
//!   (f)-[:CONTAINS]->(grant_check),
//!   (f)-[:CONTAINS]->(labels_check),
//!   (f)-[:CONTAINS]->(identifier_check),
//!   (f)-[:CONTAINS]->(digest_check),
//!   (f)-[:CONTAINS]->(digest),
//!   (f)-[:CONTAINS]->(fixture_digest),
//!   (f)-[:CONTAINS]->(fixture_provider),
//!   (f)-[:CONTAINS]->(fixture),
//!   (f)-[:CONTAINS]->(test_roundtrip),
//!   (f)-[:CONTAINS]->(test_tamper),
//!   (f)-[:CONTAINS]->(test_limits),
//!   (f)-[:CONTAINS]->(test_memory_scope),
//!   (f)-[:CONTAINS]->(test_canonical),
//!   (f)-[:CONTAINS]->(resolve),
//!   (f)-[:CONTAINS]->(resolved_document),
//!   (f)-[:CONTAINS]->(resolved_digest),
//!   (f)-[:CONTAINS]->(provider_catalog_check),
//!   (f)-[:CONTAINS]->(skill_catalog_check),
//!   (f)-[:CONTAINS]->(resolve_provider),
//!   (f)-[:CONTAINS]->(resolve_skill),
//!   (f)-[:CONTAINS]->(test_resolver_success),
//!   (f)-[:CONTAINS]->(test_provider_drift),
//!   (f)-[:CONTAINS]->(test_grant_lifecycle),
//!   (f)-[:CONTAINS]->(test_skill_hook),
//!   (f)-[:CONTAINS]->(test_catalog_bounds),
//!   (f)-[:CONTAINS]->(catalogs_fixture),
//!   (f)-[:CONTAINS]->(admission_fixture),
//!   (draft)-[:HAS_METHOD]->(seal),
//!   (document)-[:HAS_METHOD]->(verify),
//!   (document)-[:HAS_METHOD]->(decode),
//!   (verified)-[:HAS_METHOD]->(document_ref),
//!   (verified)-[:HAS_METHOD]->(scope_check),
//!   (resolver)-[:HAS_METHOD]->(resolve),
//!   (resolved)-[:HAS_METHOD]->(resolved_document),
//!   (resolved)-[:HAS_METHOD]->(resolved_digest),
//!   (seal)-[:CALLS]->(validate),
//!   (seal)-[:CALLS]->(digest),
//!   (verify)-[:CALLS]->(validate),
//!   (verify)-[:CALLS]->(digest),
//!   (decode)-[:CALLS]->(verify),
//!   (resolve)-[:CALLS]->(scope_check),
//!   (resolve)-[:CALLS]->(provider_catalog_check),
//!   (resolve)-[:CALLS]->(skill_catalog_check),
//!   (resolve)-[:CALLS]->(resolve_provider),
//!   (resolve)-[:CALLS]->(resolve_skill),
//!   (provider_catalog_check)-[:CALLS]->(provider_check),
//!   (skill_catalog_check)-[:CALLS]->(labels_check),
//!   (skill_catalog_check)-[:CALLS]->(identifier_check),
//!   (skill_catalog_check)-[:CALLS]->(digest_check),
//!   (resolve_provider)-[:CALLS]->(grant_check),
//!   (resolve_skill)-[:CALLS]->(grant_check),
//!   (validate)-[:CALLS]->(provider_check),
//!   (validate)-[:CALLS]->(budget_check),
//!   (validate)-[:CALLS]->(grant_check),
//!   (validate)-[:CALLS]->(labels_check),
//!   (validate)-[:CALLS]->(digest_check),
//!   (provider_check)-[:CALLS]->(labels_check),
//!   (provider_check)-[:CALLS]->(digest_check),
//!   (provider_check)-[:CALLS]->(identifier_check),
//!   (labels_check)-[:CALLS]->(identifier_check),
//!   (fixture_provider)-[:CALLS]->(fixture_digest),
//!   (fixture)-[:CALLS]->(fixture_digest),
//!   (fixture)-[:CALLS]->(fixture_provider),
//!   (test_roundtrip)-[:CALLS]->(fixture),
//!   (test_tamper)-[:CALLS]->(fixture),
//!   (test_limits)-[:CALLS]->(fixture),
//!   (test_limits)-[:CALLS]->(fixture_provider),
//!   (test_roundtrip)-[:CALLS]->(seal),
//!   (test_roundtrip)-[:CALLS]->(decode),
//!   (test_roundtrip)-[:CALLS]->(scope_check),
//!   (test_tamper)-[:CALLS]->(seal),
//!   (test_tamper)-[:CALLS]->(verify),
//!   (test_tamper)-[:CALLS]->(scope_check),
//!   (test_limits)-[:CALLS]->(seal),
//!   (test_memory_scope)-[:CALLS]->(fixture),
//!   (test_memory_scope)-[:CALLS]->(fixture_provider),
//!   (test_memory_scope)-[:CALLS]->(seal),
//!   (test_canonical)-[:CALLS]->(fixture),
//!   (test_canonical)-[:CALLS]->(seal),
//!   (test_canonical)-[:CALLS]->(decode),
//!   (test_resolver_success)-[:CALLS]->(fixture),
//!   (test_resolver_success)-[:CALLS]->(seal),
//!   (test_resolver_success)-[:CALLS]->(catalogs_fixture),
//!   (test_resolver_success)-[:CALLS]->(admission_fixture),
//!   (test_resolver_success)-[:CALLS]->(resolve),
//!   (test_provider_drift)-[:CALLS]->(fixture),
//!   (test_provider_drift)-[:CALLS]->(seal),
//!   (test_provider_drift)-[:CALLS]->(catalogs_fixture),
//!   (test_provider_drift)-[:CALLS]->(admission_fixture),
//!   (test_provider_drift)-[:CALLS]->(resolve),
//!   (test_grant_lifecycle)-[:CALLS]->(fixture),
//!   (test_grant_lifecycle)-[:CALLS]->(seal),
//!   (test_grant_lifecycle)-[:CALLS]->(catalogs_fixture),
//!   (test_grant_lifecycle)-[:CALLS]->(admission_fixture),
//!   (test_grant_lifecycle)-[:CALLS]->(resolve),
//!   (test_skill_hook)-[:CALLS]->(fixture),
//!   (test_skill_hook)-[:CALLS]->(seal),
//!   (test_skill_hook)-[:CALLS]->(catalogs_fixture),
//!   (test_skill_hook)-[:CALLS]->(admission_fixture),
//!   (test_skill_hook)-[:CALLS]->(resolve),
//!   (test_catalog_bounds)-[:CALLS]->(fixture_digest),
//!   (test_catalog_bounds)-[:CALLS]->(fixture_provider),
//!   (test_catalog_bounds)-[:CALLS]->(provider_catalog_check),
//!   (test_catalog_bounds)-[:CALLS]->(skill_catalog_check);
//! Immutable execution profile contract and bounded dependency resolver for Run admission.
//! Registry persistence and Run writer integration are not implied by this domain slice.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

/// Profile schema version supported by this verifier.
pub const EXECUTION_PROFILE_SCHEMA_VERSION: u32 = 1;
/// Maximum serialized profile size.
pub const MAX_EXECUTION_PROFILE_BYTES: usize = 65_536;
const SHA256_HEX_LENGTH: usize = 64;
const MAX_SKILLS: usize = 128;
const MAX_CAPABILITIES: usize = 64;
const MAX_ACCEPTANCE_CRITERIA: usize = 256;
const MAX_CONTEXT_BYTES: u32 = 67_108_864;
const MAX_CONTEXT_TOKENS: u32 = 16_777_216;
const MAX_CONTEXT_SOURCES: u16 = 4_096;
const MAX_MEMORY_BYTES: u32 = 16_777_216;
const MAX_MEMORY_TOKENS: u32 = 4_194_304;
const MAX_MEMORY_ITEMS: u16 = 4_096;
const MAX_MEMORY_AGE_SECONDS: u64 = 315_360_000;
const MAX_RSS_BYTES: u64 = 8_589_934_592;
const MAX_CPU_MS: u64 = 86_400_000;
const MAX_RUNTIME_MS: u64 = 86_400_000;
const MAX_CHILD_PROCESSES: u16 = 256;
const MAX_PARALLEL_TOOLS: u16 = 256;
const MAX_PROVIDER_CALLS: u32 = 100_000;
const MAX_LOOP_ITERATIONS: u32 = 100_000;
const MAX_OUTPUT_BYTES: u64 = 134_217_728;
const MAX_EVENT_BUFFER_BYTES: u32 = 16_777_216;
const MAX_LABEL_BYTES: usize = 128;
const MAX_PROVIDER_CATALOG_ENTRIES: usize = 256;
const MAX_SKILL_CATALOG_ENTRIES: usize = 4_096;

/// Tenant, Project, and optional Worktree profile scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionProfileScope {
    /// Tenant identity.
    pub tenant_id: Uuid,
    /// Project identity.
    pub project_id: Uuid,
    /// Optional profile-pinned Worktree.
    pub worktree_id: Option<Uuid>,
}

/// Versioned provider reference; digests identify implementation and safe configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderReference {
    /// Stable provider identifier.
    pub provider_id: String,
    /// Provider contract version.
    pub version: u32,
    /// Implementation SHA-256 digest.
    pub implementation_digest: String,
    /// Non-secret configuration SHA-256 digest.
    pub configuration_digest: String,
    /// Sorted capability labels required or supported by the provider.
    pub capabilities: Vec<String>,
}

/// Memory policy is explicit; unavailable required memory is never silently substituted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum MemoryPolicySnapshot {
    /// Memory intentionally disabled.
    Disabled,
    /// Bounded memory provider and retrieval scope.
    Enabled {
        /// Provider identity and capability declaration.
        provider: ProviderReference,
        /// Retrieval scope.
        read_scope: MemoryReadScope,
        /// Maximum number of selected records.
        max_items: u16,
        /// Maximum selected memory bytes.
        max_bytes: u32,
        /// Maximum selected memory tokens.
        max_tokens: u32,
        /// Maximum age of selected source data.
        max_source_age_seconds: u64,
        /// Every selected item must carry provenance.
        provenance_required: bool,
    },
    /// Required capability was unavailable; this state fails profile verification.
    Unavailable {
        /// Stable non-secret reason identifier.
        reason: String,
    },
}

/// Scope ceiling for Memory retrieval.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryReadScope {
    /// Restrict retrieval to a Worktree.
    Worktree,
    /// Permit retrieval across the owning Project.
    Project,
}

/// Versioned and content-addressed Skill selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkillBindingSnapshot {
    /// Stable Skill identifier.
    pub skill_id: String,
    /// Selected Skill version.
    pub version: u32,
    /// Skill definition SHA-256 digest.
    pub content_digest: String,
    /// Sorted required capability labels.
    pub capabilities: Vec<String>,
}

/// Context assembly bounds and information-preservation requirements.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextPolicySnapshot {
    /// Versioned ContextAssembler implementation.
    pub assembler: ProviderReference,
    /// Maximum context bytes.
    pub max_input_bytes: u32,
    /// Maximum context tokens.
    pub max_input_tokens: u32,
    /// Maximum number of source records.
    pub max_sources: u16,
    /// Compaction policy SHA-256 digest.
    pub compaction_policy_digest: String,
    /// Preserve the accepted Task Contract.
    pub preserve_task_contract: bool,
    /// Preserve acceptance criteria.
    pub preserve_acceptance_criteria: bool,
    /// Preserve authorization scope.
    pub preserve_authorization_scope: bool,
}

/// Independent validation provider, suite, toolchain, and acceptance criteria.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationPolicySnapshot {
    /// Validation provider.
    pub provider: ProviderReference,
    /// Validation suite SHA-256 digest.
    pub suite_digest: String,
    /// Toolchain/environment SHA-256 digest.
    pub toolchain_digest: String,
    /// Sorted acceptance criterion identifiers.
    pub acceptance_criteria: Vec<String>,
}

/// Engineering Loop policy and termination limits. Schedule occurrence policy stays with Automation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoopBudgetSnapshot {
    /// Versioned Engineering Loop policy implementation.
    pub policy: ProviderReference,
    /// Maximum iteration count.
    pub max_iterations: u32,
    /// Stop after this many iterations without progress.
    pub max_no_progress_iterations: u32,
    /// Maximum wall-clock milliseconds.
    pub max_wall_clock_ms: u64,
    /// Maximum provider calls.
    pub max_provider_calls: u32,
}

/// Run-level hard ceilings for memory, CPU, subprocesses, output, and concurrency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBudgetSnapshot {
    /// Maximum resident memory bytes.
    pub max_rss_bytes: u64,
    /// Maximum CPU milliseconds.
    pub max_cpu_ms: u64,
    /// Maximum Run wall-clock milliseconds.
    pub max_runtime_ms: u64,
    /// Maximum child processes.
    pub max_child_processes: u16,
    /// Maximum concurrent tools.
    pub max_parallel_tools: u16,
    /// Maximum provider calls.
    pub max_provider_calls: u32,
    /// Maximum captured output bytes.
    pub max_output_bytes: u64,
    /// Maximum event-buffer bytes before backpressure.
    pub max_event_buffer_bytes: u32,
}

/// Effective Rust-native HookSet pinned for Run admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HookSetSnapshot {
    /// HookSet identity.
    pub hook_set_id: Uuid,
    /// Published HookSet version.
    pub version: u64,
    /// Effective inherited HookSet SHA-256 digest.
    pub effective_digest: String,
}

/// Admission-time capability grant snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrantSnapshot {
    /// Grant-set identity.
    pub grant_set_id: Uuid,
    /// Grant-set version.
    pub version: u64,
    /// Sorted granted capability labels.
    pub capabilities: Vec<String>,
    /// Grant expiry timestamp in Unix epoch milliseconds.
    pub expires_at_epoch_ms: u64,
}

/// Optional engineering manifest pinned to repository commit and content digest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectEngineeringManifestRef {
    /// Repository identity.
    pub repository_id: Uuid,
    /// Full commit object identifier.
    pub commit_oid: String,
    /// Manifest SHA-256 digest.
    pub manifest_digest: String,
}

/// Mutable profile source; seal it before admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentExecutionProfileDraft {
    /// Schema version.
    pub schema_version: u32,
    /// Tenant/Project/Worktree scope.
    pub scope: ExecutionProfileScope,
    /// Selected Agent provider.
    pub agent_provider: ProviderReference,
    /// Memory behavior and limits.
    pub memory: MemoryPolicySnapshot,
    /// Frozen Skill bindings.
    pub skills: Vec<SkillBindingSnapshot>,
    /// Context construction policy.
    pub context: ContextPolicySnapshot,
    /// Independent validation policy.
    pub validation: ValidationPolicySnapshot,
    /// Engineering Loop policy and budget; Schedule Loop remains an Automation concern.
    pub loop_budget: LoopBudgetSnapshot,
    /// Hard Run resource budget.
    pub resource_budget: ResourceBudgetSnapshot,
    /// Effective HookSet snapshot.
    pub hook_set: HookSetSnapshot,
    /// Admission-time grants.
    pub grants: GrantSnapshot,
    /// Optional Project engineering manifest.
    pub engineering_manifest: Option<ProjectEngineeringManifestRef>,
}

/// Serialized profile plus digest over its profile payload.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentExecutionProfileDocument {
    /// Profile payload.
    pub profile: AgentExecutionProfileDraft,
    /// Lowercase SHA-256 digest of the serialized payload.
    pub content_digest: String,
}

/// Verified immutable profile accepted by later admission steps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedAgentExecutionProfile {
    document: AgentExecutionProfileDocument,
}

/// One bounded provider catalog entry visible to Run admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionProviderCatalogEntry {
    /// Exact provider implementation and capability snapshot.
    pub provider: ProviderReference,
    /// Whether the provider is currently available for new work.
    pub available: bool,
}

/// One bounded Skill registry entry visible to Run admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionSkillCatalogEntry {
    /// Exact Skill manifest binding.
    pub skill: SkillBindingSnapshot,
    /// Whether the Skill is currently enabled and not revoked.
    pub available: bool,
}

/// Current Worktree lifecycle state used by Run admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorktreeRunState {
    /// Worktree accepts new Runs.
    Active,
    /// Worktree is draining and rejects new Runs.
    Draining,
    /// Worktree is archived and rejects new Runs.
    Archived,
}

/// Current bounded facts required to resolve a verified Profile for a Run.
/// The caller must perform actor and GroupContext authorization before constructing these facts.
pub struct ExecutionProfileAdmissionFacts<'a> {
    /// Exact target tenant/project/worktree scope.
    pub requested_scope: &'a ExecutionProfileScope,
    /// Current time in Unix epoch milliseconds.
    pub now_epoch_ms: u64,
    /// Current grant snapshot read after authorization.
    pub current_grants: &'a GrantSnapshot,
    /// Sorted provider catalog visible to this target.
    pub providers: &'a [ExecutionProviderCatalogEntry],
    /// Sorted Skill catalog visible to this target.
    pub skills: &'a [ExecutionSkillCatalogEntry],
    /// Current effective Worktree HookSet.
    pub effective_hook_set: &'a HookSetSnapshot,
    /// Current Worktree lifecycle state.
    pub worktree_state: WorktreeRunState,
}

/// Resolved Profile that passed current provider, Skill, grant, HookSet, and lifecycle checks.
#[derive(Debug)]
pub struct ResolvedAgentExecutionProfile<'a> {
    profile: &'a VerifiedAgentExecutionProfile,
}

/// Resolves immutable Profile references against bounded current registries.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExecutionProfileResolver;

/// Bounded parse, validation, digest, and scope failures.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExecutionProfileError {
    /// Encoded profile exceeds the hard limit.
    #[error("profile exceeds the 64 KiB limit")]
    TooLarge,
    /// JSON is malformed or contains unknown fields.
    #[error("profile JSON is invalid")]
    InvalidJson,
    /// Schema version is unsupported.
    #[error("unsupported profile schema version")]
    UnsupportedSchema,
    /// A required field or bound is invalid.
    #[error("invalid profile field: {0}")]
    InvalidField(&'static str),
    /// A list is unsorted, duplicated, or too large.
    #[error("invalid profile list: {0}")]
    InvalidList(&'static str),
    /// Required provider, memory, or capability grant is unavailable.
    #[error("required capability is unavailable")]
    CapabilityUnavailable,
    /// Payload digest does not match.
    #[error("profile digest mismatch")]
    DigestMismatch,
    /// Profile cannot run in the requested scope.
    #[error("profile scope mismatch")]
    ScopeMismatch,
    /// A required provider or Skill is missing, unavailable, revoked, or changed.
    #[error("required provider or Skill is unavailable")]
    RegistryUnavailable,
    /// Current grant differs from the immutable Profile grant snapshot.
    #[error("current grant differs from the Profile snapshot")]
    GrantChanged,
    /// Current grant snapshot is expired.
    #[error("current grant is expired")]
    GrantExpired,
    /// Current effective HookSet differs from the Profile snapshot.
    #[error("effective HookSet changed")]
    HookSetChanged,
    /// Worktree is not accepting new Runs.
    #[error("Worktree is not accepting new Runs")]
    WorktreeUnavailable,
    /// Provider or Skill catalog is oversized, unsorted, or duplicated.
    #[error("execution catalog is invalid")]
    InvalidCatalog,
}

impl AgentExecutionProfileDraft {
    /// Validate and freeze this draft with a deterministic SHA-256 digest.
    pub fn seal(self) -> Result<VerifiedAgentExecutionProfile, ExecutionProfileError> {
        validate_profile(&self)?;
        let document = AgentExecutionProfileDocument {
            content_digest: compute_digest(&self)?,
            profile: self,
        };
        document.verify()
    }
}

impl AgentExecutionProfileDocument {
    /// Decode a bounded JSON document and verify its contents.
    pub fn decode_and_verify(
        bytes: &[u8],
    ) -> Result<VerifiedAgentExecutionProfile, ExecutionProfileError> {
        if bytes.len() > MAX_EXECUTION_PROFILE_BYTES {
            return Err(ExecutionProfileError::TooLarge);
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| ExecutionProfileError::InvalidJson)?;
        value.verify()
    }

    /// Validate fields and verify the immutable profile digest.
    pub fn verify(self) -> Result<VerifiedAgentExecutionProfile, ExecutionProfileError> {
        validate_profile(&self.profile)?;
        validate_digest(&self.content_digest, "content_digest")?;
        let encoded = serde_json::to_vec(&self)
            .map_err(|_| ExecutionProfileError::InvalidField("profile"))?;
        if encoded.len() > MAX_EXECUTION_PROFILE_BYTES {
            return Err(ExecutionProfileError::TooLarge);
        }
        if self.content_digest != compute_digest(&self.profile)? {
            return Err(ExecutionProfileError::DigestMismatch);
        }
        Ok(VerifiedAgentExecutionProfile { document: self })
    }
}

impl VerifiedAgentExecutionProfile {
    /// Borrow the immutable verified document.
    #[must_use]
    pub fn document(&self) -> &AgentExecutionProfileDocument {
        &self.document
    }

    /// Confirm tenant, Project, and Worktree scope for a Run.
    pub fn validate_for_scope(
        &self,
        requested: &ExecutionProfileScope,
    ) -> Result<(), ExecutionProfileError> {
        let stored = &self.document.profile.scope;
        if stored.tenant_id != requested.tenant_id
            || stored.project_id != requested.project_id
            || stored
                .worktree_id
                .is_some_and(|id| Some(id) != requested.worktree_id)
            || requested.worktree_id.is_none()
        {
            return Err(ExecutionProfileError::ScopeMismatch);
        }
        Ok(())
    }
}

impl ExecutionProfileResolver {
    /// Resolve this Profile against current, already-authorized Worktree facts.
    pub fn resolve<'a>(
        &self,
        profile: &'a VerifiedAgentExecutionProfile,
        facts: &ExecutionProfileAdmissionFacts<'_>,
    ) -> Result<ResolvedAgentExecutionProfile<'a>, ExecutionProfileError> {
        profile.validate_for_scope(facts.requested_scope)?;
        if facts.worktree_state != WorktreeRunState::Active {
            return Err(ExecutionProfileError::WorktreeUnavailable);
        }
        if facts.now_epoch_ms == 0 {
            return Err(ExecutionProfileError::InvalidField("now_epoch_ms"));
        }

        let selected = &profile.document().profile.grants;
        if facts.current_grants != selected {
            return Err(ExecutionProfileError::GrantChanged);
        }
        if facts.now_epoch_ms >= selected.expires_at_epoch_ms {
            return Err(ExecutionProfileError::GrantExpired);
        }

        validate_provider_catalog(facts.providers)?;
        validate_skill_catalog(facts.skills)?;

        let draft = &profile.document().profile;
        resolve_provider(&draft.agent_provider, facts.providers)?;
        if let MemoryPolicySnapshot::Enabled { provider, .. } = &draft.memory {
            resolve_provider(provider, facts.providers)?;
        }
        resolve_provider(&draft.context.assembler, facts.providers)?;
        resolve_provider(&draft.validation.provider, facts.providers)?;
        resolve_provider(&draft.loop_budget.policy, facts.providers)?;
        for skill in &draft.skills {
            resolve_skill(skill, facts.skills)?;
        }
        if facts.effective_hook_set != &draft.hook_set {
            return Err(ExecutionProfileError::HookSetChanged);
        }

        Ok(ResolvedAgentExecutionProfile { profile })
    }
}

impl ResolvedAgentExecutionProfile<'_> {
    /// Borrow the immutable profile document that passed current admission checks.
    #[must_use]
    pub fn document(&self) -> &AgentExecutionProfileDocument {
        self.profile.document()
    }

    /// Return the verified Profile content digest.
    #[must_use]
    pub fn content_digest(&self) -> &str {
        &self.profile.document().content_digest
    }
}

fn validate_provider_catalog(
    entries: &[ExecutionProviderCatalogEntry],
) -> Result<(), ExecutionProfileError> {
    if entries.len() > MAX_PROVIDER_CATALOG_ENTRIES {
        return Err(ExecutionProfileError::InvalidCatalog);
    }
    let mut previous: Option<(&str, u32)> = None;
    for entry in entries {
        validate_provider(&entry.provider).map_err(|_| ExecutionProfileError::InvalidCatalog)?;
        let key = (entry.provider.provider_id.as_str(), entry.provider.version);
        if previous.is_some_and(|value| value >= key) {
            return Err(ExecutionProfileError::InvalidCatalog);
        }
        previous = Some(key);
    }
    Ok(())
}

fn validate_skill_catalog(
    entries: &[ExecutionSkillCatalogEntry],
) -> Result<(), ExecutionProfileError> {
    if entries.len() > MAX_SKILL_CATALOG_ENTRIES {
        return Err(ExecutionProfileError::InvalidCatalog);
    }
    let mut previous: Option<(&str, u32)> = None;
    for entry in entries {
        validate_identifier(&entry.skill.skill_id, "catalog.skill_id")
            .map_err(|_| ExecutionProfileError::InvalidCatalog)?;
        validate_digest(&entry.skill.content_digest, "catalog.skill_digest")
            .map_err(|_| ExecutionProfileError::InvalidCatalog)?;
        validate_labels(
            &entry.skill.capabilities,
            MAX_CAPABILITIES,
            "catalog.skill_capabilities",
        )
        .map_err(|_| ExecutionProfileError::InvalidCatalog)?;
        if entry.skill.version == 0 {
            return Err(ExecutionProfileError::InvalidCatalog);
        }
        let key = (entry.skill.skill_id.as_str(), entry.skill.version);
        if previous.is_some_and(|value| value >= key) {
            return Err(ExecutionProfileError::InvalidCatalog);
        }
        previous = Some(key);
    }
    Ok(())
}

fn resolve_provider(
    required: &ProviderReference,
    entries: &[ExecutionProviderCatalogEntry],
) -> Result<(), ExecutionProfileError> {
    let index = entries
        .binary_search_by(|entry| {
            (entry.provider.provider_id.as_str(), entry.provider.version)
                .cmp(&(required.provider_id.as_str(), required.version))
        })
        .map_err(|_| ExecutionProfileError::RegistryUnavailable)?;
    let entry = &entries[index];
    if !entry.available
        || entry.provider.implementation_digest != required.implementation_digest
        || entry.provider.configuration_digest != required.configuration_digest
    {
        return Err(ExecutionProfileError::RegistryUnavailable);
    }
    check_grants(&required.capabilities, &entry.provider.capabilities)
}

fn resolve_skill(
    required: &SkillBindingSnapshot,
    entries: &[ExecutionSkillCatalogEntry],
) -> Result<(), ExecutionProfileError> {
    let index = entries
        .binary_search_by(|entry| {
            (entry.skill.skill_id.as_str(), entry.skill.version)
                .cmp(&(required.skill_id.as_str(), required.version))
        })
        .map_err(|_| ExecutionProfileError::RegistryUnavailable)?;
    let entry = &entries[index];
    if !entry.available || entry.skill.content_digest != required.content_digest {
        return Err(ExecutionProfileError::RegistryUnavailable);
    }
    check_grants(&required.capabilities, &entry.skill.capabilities)
}

fn validate_profile(profile: &AgentExecutionProfileDraft) -> Result<(), ExecutionProfileError> {
    if profile.schema_version != EXECUTION_PROFILE_SCHEMA_VERSION {
        return Err(ExecutionProfileError::UnsupportedSchema);
    }
    let scope = &profile.scope;
    if scope.tenant_id.is_nil()
        || scope.project_id.is_nil()
        || scope.worktree_id.is_some_and(|id| id.is_nil())
    {
        return Err(ExecutionProfileError::InvalidField("scope"));
    }
    let grants = &profile.grants;
    validate_labels(
        &grants.capabilities,
        MAX_CAPABILITIES,
        "grants.capabilities",
    )?;
    if grants.grant_set_id.is_nil() || grants.version == 0 || grants.expires_at_epoch_ms == 0 {
        return Err(ExecutionProfileError::InvalidField("grants"));
    }
    validate_provider(&profile.agent_provider)?;
    check_grants(&profile.agent_provider.capabilities, &grants.capabilities)?;
    if profile.skills.len() > MAX_SKILLS {
        return Err(ExecutionProfileError::InvalidList("skills"));
    }
    let mut prior_skill: Option<&str> = None;
    for skill in &profile.skills {
        validate_identifier(&skill.skill_id, "skills.skill_id")?;
        validate_digest(&skill.content_digest, "skills.content_digest")?;
        validate_labels(&skill.capabilities, MAX_CAPABILITIES, "skills.capabilities")?;
        if skill.version == 0 || prior_skill.is_some_and(|prior| prior >= skill.skill_id.as_str()) {
            return Err(ExecutionProfileError::InvalidList("skills"));
        }
        prior_skill = Some(&skill.skill_id);
        check_grants(&skill.capabilities, &grants.capabilities)?;
    }
    match &profile.memory {
        MemoryPolicySnapshot::Disabled => {}
        MemoryPolicySnapshot::Unavailable { .. } => {
            return Err(ExecutionProfileError::CapabilityUnavailable);
        }
        MemoryPolicySnapshot::Enabled {
            provider,
            read_scope,
            max_items,
            max_bytes,
            max_tokens,
            max_source_age_seconds,
            provenance_required,
        } => {
            validate_provider(provider)?;
            check_grants(&provider.capabilities, &grants.capabilities)?;
            if *max_items == 0
                || *max_items > MAX_MEMORY_ITEMS
                || *max_bytes == 0
                || *max_bytes > MAX_MEMORY_BYTES
                || *max_tokens == 0
                || *max_tokens > MAX_MEMORY_TOKENS
                || *max_source_age_seconds == 0
                || *max_source_age_seconds > MAX_MEMORY_AGE_SECONDS
                || !provenance_required
            {
                return Err(ExecutionProfileError::InvalidField("memory"));
            }
            if scope.worktree_id.is_some() && *read_scope != MemoryReadScope::Worktree {
                return Err(ExecutionProfileError::ScopeMismatch);
            }
        }
    }
    let context = &profile.context;
    validate_provider(&context.assembler)?;
    check_grants(
        &context.assembler.capabilities,
        &profile.grants.capabilities,
    )?;
    validate_digest(
        &context.compaction_policy_digest,
        "context.compaction_policy_digest",
    )?;
    if context.max_input_bytes == 0
        || context.max_input_bytes > MAX_CONTEXT_BYTES
        || context.max_input_tokens == 0
        || context.max_input_tokens > MAX_CONTEXT_TOKENS
        || context.max_sources == 0
        || context.max_sources > MAX_CONTEXT_SOURCES
        || !context.preserve_task_contract
        || !context.preserve_acceptance_criteria
        || !context.preserve_authorization_scope
    {
        return Err(ExecutionProfileError::InvalidField("context"));
    }
    let validation = &profile.validation;
    validate_provider(&validation.provider)?;
    check_grants(
        &validation.provider.capabilities,
        &profile.grants.capabilities,
    )?;
    validate_digest(&validation.suite_digest, "validation.suite_digest")?;
    validate_digest(&validation.toolchain_digest, "validation.toolchain_digest")?;
    if validation.provider.provider_id == profile.agent_provider.provider_id
        || validation.acceptance_criteria.is_empty()
        || validation.acceptance_criteria.len() > MAX_ACCEPTANCE_CRITERIA
    {
        return Err(ExecutionProfileError::InvalidField("validation"));
    }
    validate_labels(
        &validation.acceptance_criteria,
        MAX_ACCEPTANCE_CRITERIA,
        "validation.acceptance_criteria",
    )?;
    validate_provider(&profile.loop_budget.policy)?;
    check_grants(
        &profile.loop_budget.policy.capabilities,
        &profile.grants.capabilities,
    )?;
    validate_budgets(&profile.loop_budget, &profile.resource_budget)?;
    if profile.hook_set.hook_set_id.is_nil() || profile.hook_set.version == 0 {
        return Err(ExecutionProfileError::InvalidField("hook_set"));
    }
    validate_digest(
        &profile.hook_set.effective_digest,
        "hook_set.effective_digest",
    )?;
    if let Some(manifest) = &profile.engineering_manifest {
        if manifest.repository_id.is_nil()
            || !matches!(manifest.commit_oid.len(), 40 | 64)
            || !manifest
                .commit_oid
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ExecutionProfileError::InvalidField(
                "engineering_manifest.commit_oid",
            ));
        }
        validate_digest(
            &manifest.manifest_digest,
            "engineering_manifest.manifest_digest",
        )?;
    }
    Ok(())
}

fn validate_provider(provider: &ProviderReference) -> Result<(), ExecutionProfileError> {
    validate_identifier(&provider.provider_id, "provider.provider_id")?;
    validate_digest(
        &provider.implementation_digest,
        "provider.implementation_digest",
    )?;
    validate_digest(
        &provider.configuration_digest,
        "provider.configuration_digest",
    )?;
    validate_labels(
        &provider.capabilities,
        MAX_CAPABILITIES,
        "provider.capabilities",
    )?;
    if provider.version == 0 {
        return Err(ExecutionProfileError::InvalidField("provider.version"));
    }
    Ok(())
}

fn validate_budgets(
    loop_budget: &LoopBudgetSnapshot,
    resources: &ResourceBudgetSnapshot,
) -> Result<(), ExecutionProfileError> {
    if loop_budget.max_iterations == 0
        || loop_budget.max_iterations > MAX_LOOP_ITERATIONS
        || loop_budget.max_no_progress_iterations == 0
        || loop_budget.max_no_progress_iterations > loop_budget.max_iterations
        || loop_budget.max_wall_clock_ms == 0
        || loop_budget.max_provider_calls == 0
        || loop_budget.max_provider_calls > MAX_PROVIDER_CALLS
        || resources.max_rss_bytes == 0
        || resources.max_rss_bytes > MAX_RSS_BYTES
        || resources.max_cpu_ms == 0
        || resources.max_cpu_ms > MAX_CPU_MS
        || resources.max_runtime_ms == 0
        || resources.max_runtime_ms > MAX_RUNTIME_MS
        || resources.max_child_processes == 0
        || resources.max_child_processes > MAX_CHILD_PROCESSES
        || resources.max_parallel_tools == 0
        || resources.max_parallel_tools > MAX_PARALLEL_TOOLS
        || resources.max_provider_calls == 0
        || resources.max_provider_calls > MAX_PROVIDER_CALLS
        || resources.max_output_bytes == 0
        || resources.max_output_bytes > MAX_OUTPUT_BYTES
        || resources.max_event_buffer_bytes == 0
        || resources.max_event_buffer_bytes > MAX_EVENT_BUFFER_BYTES
        || loop_budget.max_wall_clock_ms > resources.max_runtime_ms
        || loop_budget.max_provider_calls > resources.max_provider_calls
    {
        return Err(ExecutionProfileError::InvalidField("budgets"));
    }
    Ok(())
}

fn check_grants(requested: &[String], granted: &[String]) -> Result<(), ExecutionProfileError> {
    if requested
        .iter()
        .any(|capability| granted.binary_search(capability).is_err())
    {
        return Err(ExecutionProfileError::CapabilityUnavailable);
    }
    Ok(())
}

fn validate_labels(
    labels: &[String],
    maximum: usize,
    field: &'static str,
) -> Result<(), ExecutionProfileError> {
    if labels.len() > maximum {
        return Err(ExecutionProfileError::InvalidList(field));
    }
    let mut previous: Option<&str> = None;
    for label in labels {
        validate_identifier(label, field)?;
        if previous.is_some_and(|value| value >= label.as_str()) {
            return Err(ExecutionProfileError::InvalidList(field));
        }
        previous = Some(label);
    }
    Ok(())
}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), ExecutionProfileError> {
    if value.is_empty()
        || value.len() > MAX_LABEL_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_.:/".contains(&byte)
        })
    {
        return Err(ExecutionProfileError::InvalidField(field));
    }
    Ok(())
}

fn validate_digest(value: &str, field: &'static str) -> Result<(), ExecutionProfileError> {
    if value.len() != SHA256_HEX_LENGTH
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ExecutionProfileError::InvalidField(field));
    }
    Ok(())
}

fn compute_digest(profile: &AgentExecutionProfileDraft) -> Result<String, ExecutionProfileError> {
    let bytes =
        serde_json::to_vec(profile).map_err(|_| ExecutionProfileError::InvalidField("profile"))?;
    if bytes.len() > MAX_EXECUTION_PROFILE_BYTES {
        return Err(ExecutionProfileError::TooLarge);
    }
    let hash = Sha256::digest(bytes);
    let mut result = String::with_capacity(SHA256_HEX_LENGTH);
    for byte in hash {
        use std::fmt::Write as _;
        write!(&mut result, "{byte:02x}")
            .map_err(|_| ExecutionProfileError::InvalidField("content_digest"))?;
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> String {
        std::iter::repeat(character)
            .take(SHA256_HEX_LENGTH)
            .collect()
    }

    fn provider(id: &str, capabilities: &[&str]) -> ProviderReference {
        ProviderReference {
            provider_id: id.to_owned(),
            version: 1,
            implementation_digest: digest('a'),
            configuration_digest: digest('b'),
            capabilities: capabilities.iter().map(|item| (*item).to_owned()).collect(),
        }
    }

    fn valid_draft() -> AgentExecutionProfileDraft {
        AgentExecutionProfileDraft {
            schema_version: EXECUTION_PROFILE_SCHEMA_VERSION,
            scope: ExecutionProfileScope {
                tenant_id: Uuid::from_u128(1),
                project_id: Uuid::from_u128(2),
                worktree_id: None,
            },
            agent_provider: provider("agent.local", &["tool.read"]),
            memory: MemoryPolicySnapshot::Disabled,
            skills: vec![SkillBindingSnapshot {
                skill_id: "review.rust".to_owned(),
                version: 1,
                content_digest: digest('c'),
                capabilities: vec!["tool.read".to_owned()],
            }],
            context: ContextPolicySnapshot {
                assembler: provider("context.assembler", &[]),
                max_input_bytes: 32_768,
                max_input_tokens: 8_192,
                max_sources: 32,
                compaction_policy_digest: digest('d'),
                preserve_task_contract: true,
                preserve_acceptance_criteria: true,
                preserve_authorization_scope: true,
            },
            validation: ValidationPolicySnapshot {
                provider: provider("validator.rust", &[]),
                suite_digest: digest('e'),
                toolchain_digest: digest('f'),
                acceptance_criteria: vec!["ac-compile".to_owned(), "ac-tests".to_owned()],
            },
            loop_budget: LoopBudgetSnapshot {
                policy: provider("loop.engineering", &[]),
                max_iterations: 8,
                max_no_progress_iterations: 2,
                max_wall_clock_ms: 120_000,
                max_provider_calls: 24,
            },
            resource_budget: ResourceBudgetSnapshot {
                max_rss_bytes: 1_073_741_824,
                max_cpu_ms: 300_000,
                max_runtime_ms: 180_000,
                max_child_processes: 8,
                max_parallel_tools: 4,
                max_provider_calls: 32,
                max_output_bytes: 2_097_152,
                max_event_buffer_bytes: 262_144,
            },
            hook_set: HookSetSnapshot {
                hook_set_id: Uuid::from_u128(3),
                version: 4,
                effective_digest: digest('1'),
            },
            grants: GrantSnapshot {
                grant_set_id: Uuid::from_u128(5),
                version: 2,
                capabilities: vec!["tool.read".to_owned()],
                expires_at_epoch_ms: 1_900_000_000_000,
            },
            engineering_manifest: Some(ProjectEngineeringManifestRef {
                repository_id: Uuid::from_u128(6),
                commit_oid: "1234567890123456789012345678901234567890".to_owned(),
                manifest_digest: digest('2'),
            }),
        }
    }

    fn catalogs_from(
        draft: &AgentExecutionProfileDraft,
    ) -> (
        Vec<ExecutionProviderCatalogEntry>,
        Vec<ExecutionSkillCatalogEntry>,
    ) {
        let mut providers = vec![
            draft.agent_provider.clone(),
            draft.context.assembler.clone(),
            draft.validation.provider.clone(),
            draft.loop_budget.policy.clone(),
        ];
        if let MemoryPolicySnapshot::Enabled { provider, .. } = &draft.memory {
            providers.push(provider.clone());
        }
        providers.sort_by(|left, right| {
            (left.provider_id.as_str(), left.version)
                .cmp(&(right.provider_id.as_str(), right.version))
        });
        providers.dedup_by(|left, right| {
            left.provider_id == right.provider_id && left.version == right.version && left == right
        });
        let providers = providers
            .into_iter()
            .map(|provider| ExecutionProviderCatalogEntry {
                provider,
                available: true,
            })
            .collect();
        let skills = draft
            .skills
            .iter()
            .cloned()
            .map(|skill| ExecutionSkillCatalogEntry {
                skill,
                available: true,
            })
            .collect();
        (providers, skills)
    }

    fn admission_facts<'a>(
        requested_scope: &'a ExecutionProfileScope,
        current_grants: &'a GrantSnapshot,
        providers: &'a [ExecutionProviderCatalogEntry],
        skills: &'a [ExecutionSkillCatalogEntry],
        effective_hook_set: &'a HookSetSnapshot,
        now_epoch_ms: u64,
        worktree_state: WorktreeRunState,
    ) -> ExecutionProfileAdmissionFacts<'a> {
        ExecutionProfileAdmissionFacts {
            requested_scope,
            now_epoch_ms,
            current_grants,
            providers,
            skills,
            effective_hook_set,
            worktree_state,
        }
    }

    #[test]
    fn verified_profile_round_trips_and_binds_scope() {
        let verified = valid_draft().seal().expect("valid profile");
        let bytes = serde_json::to_vec(verified.document()).expect("serialize");
        let decoded = AgentExecutionProfileDocument::decode_and_verify(&bytes).expect("verify");
        decoded
            .validate_for_scope(&ExecutionProfileScope {
                tenant_id: Uuid::from_u128(1),
                project_id: Uuid::from_u128(2),
                worktree_id: Some(Uuid::from_u128(8)),
            })
            .expect("Project profile can be used by its Worktree");
        assert_eq!(
            decoded.document().content_digest,
            verified.document().content_digest
        );
    }

    #[test]
    fn tampering_and_cross_scope_execution_are_rejected() {
        let verified = valid_draft().seal().expect("valid profile");
        let mut changed = verified.document().clone();
        changed.profile.loop_budget.max_iterations += 1;
        assert_eq!(
            changed.verify().unwrap_err(),
            ExecutionProfileError::DigestMismatch
        );
        assert_eq!(
            verified.validate_for_scope(&ExecutionProfileScope {
                tenant_id: Uuid::from_u128(1),
                project_id: Uuid::from_u128(9),
                worktree_id: Some(Uuid::from_u128(8)),
            }),
            Err(ExecutionProfileError::ScopeMismatch)
        );
    }

    #[test]
    fn missing_memory_grants_and_unbounded_limits_fail_closed() {
        let missing_memory = AgentExecutionProfileDraft {
            memory: MemoryPolicySnapshot::Unavailable {
                reason: "provider_missing".to_owned(),
            },
            ..valid_draft()
        };
        assert_eq!(
            missing_memory.seal().unwrap_err(),
            ExecutionProfileError::CapabilityUnavailable
        );
        let ungranted = AgentExecutionProfileDraft {
            agent_provider: provider("agent.local", &["tool.write"]),
            ..valid_draft()
        };
        assert_eq!(
            ungranted.seal().unwrap_err(),
            ExecutionProfileError::CapabilityUnavailable
        );
        let ungranted_context = AgentExecutionProfileDraft {
            context: ContextPolicySnapshot {
                assembler: provider("context.assembler", &["context.remote"]),
                ..valid_draft().context
            },
            ..valid_draft()
        };
        assert_eq!(
            ungranted_context.seal().unwrap_err(),
            ExecutionProfileError::CapabilityUnavailable
        );
        let ungranted_loop_policy = AgentExecutionProfileDraft {
            loop_budget: LoopBudgetSnapshot {
                policy: provider("loop.engineering", &["loop.remote"]),
                ..valid_draft().loop_budget
            },
            ..valid_draft()
        };
        assert_eq!(
            ungranted_loop_policy.seal().unwrap_err(),
            ExecutionProfileError::CapabilityUnavailable
        );
        let over_budget = valid_draft();
        let invalid_loop = AgentExecutionProfileDraft {
            loop_budget: LoopBudgetSnapshot {
                max_wall_clock_ms: 200_000,
                ..over_budget.loop_budget
            },
            ..over_budget
        };
        assert_eq!(
            invalid_loop.seal().unwrap_err(),
            ExecutionProfileError::InvalidField("budgets")
        );
        let oversized = valid_draft();
        let unbounded_rss = AgentExecutionProfileDraft {
            resource_budget: ResourceBudgetSnapshot {
                max_rss_bytes: u64::MAX,
                ..oversized.resource_budget
            },
            ..oversized
        };
        assert_eq!(
            unbounded_rss.seal().unwrap_err(),
            ExecutionProfileError::InvalidField("budgets")
        );
    }
    #[test]
    fn scoped_memory_and_context_requirements_are_enforced() {
        let enabled = AgentExecutionProfileDraft {
            memory: MemoryPolicySnapshot::Enabled {
                provider: provider("memory.local", &["tool.read"]),
                read_scope: MemoryReadScope::Project,
                max_items: 32,
                max_bytes: 16_384,
                max_tokens: 4_096,
                max_source_age_seconds: 86_400,
                provenance_required: true,
            },
            ..valid_draft()
        };
        assert!(enabled.clone().seal().is_ok());
        let worktree_scoped = AgentExecutionProfileDraft {
            scope: ExecutionProfileScope {
                tenant_id: Uuid::from_u128(1),
                project_id: Uuid::from_u128(2),
                worktree_id: Some(Uuid::from_u128(8)),
            },
            ..enabled
        };
        assert_eq!(
            worktree_scoped.seal().unwrap_err(),
            ExecutionProfileError::ScopeMismatch
        );
        let lossy = valid_draft();
        let invalid_context = AgentExecutionProfileDraft {
            context: ContextPolicySnapshot {
                preserve_authorization_scope: false,
                ..lossy.context
            },
            ..lossy
        };
        assert_eq!(
            invalid_context.seal().unwrap_err(),
            ExecutionProfileError::InvalidField("context")
        );
    }

    #[test]
    fn resolver_admits_only_exact_current_profile_dependencies() {
        let draft = valid_draft();
        let verified = draft.clone().seal().expect("valid profile");
        let (providers, skills) = catalogs_from(&draft);
        let scope = ExecutionProfileScope {
            tenant_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            worktree_id: Some(Uuid::from_u128(8)),
        };
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        let resolved = ExecutionProfileResolver
            .resolve(&verified, &facts)
            .expect("current dependencies resolve");
        assert_eq!(
            resolved.content_digest(),
            verified.document().content_digest.as_str()
        );
    }

    #[test]
    fn resolver_rejects_provider_drift_and_invalid_catalog_order() {
        let draft = valid_draft();
        let verified = draft.clone().seal().expect("valid profile");
        let (mut providers, skills) = catalogs_from(&draft);
        let scope = ExecutionProfileScope {
            tenant_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            worktree_id: Some(Uuid::from_u128(8)),
        };
        let agent = providers
            .iter_mut()
            .find(|entry| entry.provider.provider_id == "agent.local")
            .expect("agent catalog entry");
        agent.available = false;
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::RegistryUnavailable
        );

        let (mut providers, skills) = catalogs_from(&draft);
        let agent = providers
            .iter_mut()
            .find(|entry| entry.provider.provider_id == "agent.local")
            .expect("agent catalog entry");
        agent.provider.implementation_digest = digest('9');
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::RegistryUnavailable
        );

        let (mut providers, skills) = catalogs_from(&draft);
        let agent = providers
            .iter_mut()
            .find(|entry| entry.provider.provider_id == "agent.local")
            .expect("agent catalog entry");
        agent.provider.capabilities.clear();
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::CapabilityUnavailable
        );

        let (mut providers, skills) = catalogs_from(&draft);
        providers.reverse();
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::InvalidCatalog
        );
    }

    #[test]
    fn resolver_rejects_expired_or_changed_grants_and_draining_worktrees() {
        let draft = valid_draft();
        let verified = draft.clone().seal().expect("valid profile");
        let (providers, skills) = catalogs_from(&draft);
        let scope = ExecutionProfileScope {
            tenant_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            worktree_id: Some(Uuid::from_u128(8)),
        };
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            draft.grants.expires_at_epoch_ms,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::GrantExpired
        );

        let mut changed_grant = draft.grants.clone();
        changed_grant.version += 1;
        let facts = admission_facts(
            &scope,
            &changed_grant,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::GrantChanged
        );

        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Draining,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::WorktreeUnavailable
        );
    }

    #[test]
    fn resolver_rejects_revoked_skills_and_changed_hooksets() {
        let draft = valid_draft();
        let verified = draft.clone().seal().expect("valid profile");
        let (providers, mut skills) = catalogs_from(&draft);
        let scope = ExecutionProfileScope {
            tenant_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            worktree_id: Some(Uuid::from_u128(8)),
        };
        skills[0].available = false;
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &draft.hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::RegistryUnavailable
        );

        skills[0].available = true;
        let mut changed_hook_set = draft.hook_set.clone();
        changed_hook_set.version += 1;
        let facts = admission_facts(
            &scope,
            &draft.grants,
            &providers,
            &skills,
            &changed_hook_set,
            1_800_000_000_000,
            WorktreeRunState::Active,
        );
        assert_eq!(
            ExecutionProfileResolver
                .resolve(&verified, &facts)
                .unwrap_err(),
            ExecutionProfileError::HookSetChanged
        );
    }

    #[test]
    fn execution_catalogs_enforce_entry_bounds() {
        let provider_entry = ExecutionProviderCatalogEntry {
            provider: provider("catalog.local", &[]),
            available: true,
        };
        assert_eq!(
            validate_provider_catalog(&[provider_entry.clone(), provider_entry.clone()]),
            Err(ExecutionProfileError::InvalidCatalog)
        );
        let oversized_providers = vec![provider_entry; MAX_PROVIDER_CATALOG_ENTRIES + 1];
        assert_eq!(
            validate_provider_catalog(&oversized_providers),
            Err(ExecutionProfileError::InvalidCatalog)
        );

        let skill_entry = ExecutionSkillCatalogEntry {
            skill: SkillBindingSnapshot {
                skill_id: "catalog.skill".to_owned(),
                version: 1,
                content_digest: digest('a'),
                capabilities: Vec::new(),
            },
            available: true,
        };
        assert_eq!(
            validate_skill_catalog(&[skill_entry.clone(), skill_entry.clone()]),
            Err(ExecutionProfileError::InvalidCatalog)
        );
        let oversized_skills = vec![skill_entry; MAX_SKILL_CATALOG_ENTRIES + 1];
        assert_eq!(
            validate_skill_catalog(&oversized_skills),
            Err(ExecutionProfileError::InvalidCatalog)
        );
    }

    #[test]
    fn noncanonical_lists_and_oversized_documents_are_rejected() {
        let noncanonical = AgentExecutionProfileDraft {
            grants: GrantSnapshot {
                capabilities: vec!["tool.write".to_owned(), "tool.read".to_owned()],
                ..valid_draft().grants
            },
            ..valid_draft()
        };
        assert_eq!(
            noncanonical.seal().unwrap_err(),
            ExecutionProfileError::InvalidList("grants.capabilities")
        );
        let oversized = vec![0; MAX_EXECUTION_PROFILE_BYTES + 1];
        assert_eq!(
            AgentExecutionProfileDocument::decode_and_verify(&oversized).unwrap_err(),
            ExecutionProfileError::TooLarge
        );
    }
}
