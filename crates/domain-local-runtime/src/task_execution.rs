//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"task_execution.rs",type:"file",language:"rust"}),(m:Module {name:"task_execution",type:"module",language:"rust"}),
//!   (ctx:Class {name:"TaskExecutionContext",type:"class"}),(binding:Class {name:"RuntimeWorktreeBinding",type:"class"}),(profile:Class {name:"ApprovedTaskLaunchProfile",type:"class"}),(prepared:Class {name:"PreparedTaskCliExecution",type:"class"}),(err:Class {name:"TaskExecutionError",type:"class"}),(grant:Class {name:"SignedTaskExecutionGrant",type:"class"}),(signer:Class {name:"Ed25519TaskExecutionGrantSigner",type:"class"}),(verifier_impl:Class {name:"Ed25519TaskExecutionGrantVerifier",type:"class"}),(verify_error:Enum {name:"TaskExecutionGrantVerificationError",type:"enum"}),(verifier:Interface {name:"TaskExecutionGrantVerifier",type:"trait"}),
//!   (prep:Function {name:"prepare_task_cli_execution",type:"function"}),(prepare_verified:Function {name:"prepare_and_consume_verified_task_cli_execution",type:"function"}),(sign:Function {name:"Ed25519TaskExecutionGrantSigner::sign",type:"function"}),(verify:Function {name:"Ed25519TaskExecutionGrantVerifier::verify",type:"function"}),(signing_bytes:Function {name:"grant_signing_bytes",type:"function"}),(validate_profile:Function {name:"validate_profile",type:"function"}),(validate_context:Function {name:"validate_context",type:"function"}),(canonical_dir:Function {name:"canonical_directory",type:"function"}),(env_key:Function {name:"is_allowed_profile_env_key",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(ctx),(m)-[:CONTAINS]->(binding),(m)-[:CONTAINS]->(profile),(m)-[:CONTAINS]->(prepared),(m)-[:CONTAINS]->(err),(m)-[:CONTAINS]->(grant),(m)-[:CONTAINS]->(signer),(m)-[:CONTAINS]->(verifier_impl),(m)-[:CONTAINS]->(verify_error),(m)-[:CONTAINS]->(verifier),(m)-[:CONTAINS]->(prep),(m)-[:CONTAINS]->(prepare_verified),(m)-[:CONTAINS]->(sign),(m)-[:CONTAINS]->(verify),(m)-[:CONTAINS]->(signing_bytes),(m)-[:CONTAINS]->(validate_profile),(m)-[:CONTAINS]->(validate_context),(m)-[:CONTAINS]->(canonical_dir),(m)-[:CONTAINS]->(env_key),
//!   (prep)-[:CALLS]->(validate_context),(prep)-[:CALLS]->(validate_profile),(prep)-[:CALLS]->(canonical_dir),(validate_profile)-[:CALLS]->(env_key);
//! CREATE (nonce_store:Interface {name:"TaskExecutionNonceStore",type:"trait"}),(nonce_error:Enum {name:"TaskExecutionNonceStoreError",type:"enum"}),(consume:Function {name:"prepare_and_consume_task_cli_execution",type:"function"});
//! MATCH (m:Module {name:"task_execution",type:"module"}),(nonce_store:Interface {name:"TaskExecutionNonceStore",type:"trait"}),(nonce_error:Enum {name:"TaskExecutionNonceStoreError",type:"enum"}),(consume:Function {name:"prepare_and_consume_task_cli_execution",type:"function"}),(prepare_verified:Function {name:"prepare_and_consume_verified_task_cli_execution",type:"function"}),(sign:Function {name:"Ed25519TaskExecutionGrantSigner::sign",type:"function"}),(verify:Function {name:"Ed25519TaskExecutionGrantVerifier::verify",type:"function"}),(signing_bytes:Function {name:"grant_signing_bytes",type:"function"}),(signer:Class {name:"Ed25519TaskExecutionGrantSigner",type:"class"}),(verifier_impl:Class {name:"Ed25519TaskExecutionGrantVerifier",type:"class"}),(verifier:Interface {name:"TaskExecutionGrantVerifier",type:"trait"}),(prep:Function {name:"prepare_task_cli_execution",type:"function"});
//! CREATE (m)-[:CONTAINS]->(nonce_store),(m)-[:CONTAINS]->(nonce_error),(m)-[:CONTAINS]->(consume),(m)-[:CONTAINS]->(verifier),(consume)-[:CALLS]->(prep),(consume)-[:USES]->(nonce_store),(prepare_verified)-[:CALLS]->(verify),(prepare_verified)-[:CALLS]->(consume),(signer)-[:HAS_METHOD]->(sign),(verifier_impl)-[:HAS_METHOD]->(verify),(verifier_impl)-[:IMPLEMENTS]->(verifier),(sign)-[:CALLS]->(signing_bytes),(verify)-[:CALLS]->(signing_bytes);
//! CREATE (pty:Module {name:"pty",type:"module",language:"rust"});
//! MATCH (m:Module {name:"task_execution",type:"module",language:"rust"}),(pty:Module {name:"pty",type:"module",language:"rust"}) CREATE (m)-[:CONTAINS]->(pty);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"task_execution",type:"module",language:"rust"}),(ctx:Class {name:"TaskExecutionContext",type:"class"}),(profile:Class {name:"ApprovedTaskLaunchProfile",type:"class"}),(prepared:Class {name:"PreparedTaskCliExecution",type:"class"}),(nonce_store:Interface {name:"TaskExecutionNonceStore",type:"trait"}),(verify:Function {name:"Ed25519TaskExecutionGrantVerifier::verify",type:"function"}),(prepare:Function {name:"prepare_task_cli_execution",type:"function"}),(registry:Class {name:"CliSessionRegistry",type:"class"});
//! CREATE (fence:Class {name:"TaskRunSpawnFence",type:"class",language:"rust"}),(fence_binding:Class {name:"TaskRunSpawnFenceBindingV2",type:"class",language:"rust"}),(prepare_fenced:Function {name:"prepare_and_consume_verified_task_run_spawn_fence",type:"function",language:"rust"}),(validate_fence:Function {name:"validate_profile_bound_fence",type:"function",language:"rust"}),(consume_profile:Function {name:"TaskExecutionNonceStore::consume_profile_bound",type:"function",language:"rust"}),(fence_id:Variable {name:"spawn_fence_id",type:"variable",language:"rust"}),(fence_digest:Variable {name:"spawn_fence_binding_digest",type:"variable",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(prepare_fenced),(m)-[:CONTAINS]->(validate_fence),(ctx)-[:USES]->(fence),(profile)-[:USES]->(fence_binding),(prepared)-[:USES]->(fence_id),(prepared)-[:USES]->(fence_digest),(prepare_fenced)-[:CALLS]->(verify),(prepare_fenced)-[:CALLS]->(validate_fence),(prepare_fenced)-[:CALLS]->(prepare),(prepare_fenced)-[:CALLS]->(consume_profile),(validate_fence)-[:CALLS]->(fence),(validate_fence)-[:USES]->(fence_binding),(consume_profile)-[:USES]->(fence_id),(consume_profile)-[:USES]->(fence_digest),(registry)-[:IMPLEMENTS]->(nonce_store);
//! CREATE (fence_test:Function {name:"profile_bound_fence_rejects_binding_drift_and_stale_issue_time",type:"function",language:"rust"}),(signature_test:Function {name:"profile_bound_grant_signature_covers_fence",type:"function",language:"rust"}),(legacy_test:Function {name:"legacy_grant_signing_bytes_remain_v1_when_fence_absent",type:"function",language:"rust"}),(legacy_consumer_test:Function {name:"legacy_consumer_rejects_profile_bound_grant",type:"function",language:"rust"}),(bounded_payload_test:Function {name:"signed_fence_payload_rejects_unbounded_digest_before_serialization",type:"function",language:"rust"}),(prepare_fence_mode:Function {name:"prepare_task_cli_execution_with_fence_mode",type:"function",language:"rust"}),(fence_well_formed:Function {name:"TaskRunSpawnFence::is_well_formed",type:"function",language:"rust"}),(digest_valid:Function {name:"is_lower_hex_sha256_digest",type:"function",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(fence_test),(m)-[:CONTAINS]->(signature_test),(m)-[:CONTAINS]->(legacy_test),(m)-[:CONTAINS]->(legacy_consumer_test),(m)-[:CONTAINS]->(bounded_payload_test),(m)-[:CONTAINS]->(prepare_fence_mode),(m)-[:CONTAINS]->(fence_well_formed),(m)-[:CONTAINS]->(digest_valid),(fence_test)-[:CALLS]->(validate_fence),(signature_test)-[:CALLS]->(sign),(signature_test)-[:CALLS]->(verify),(legacy_test)-[:CALLS]->(signing_bytes),(legacy_consumer_test)-[:CALLS]->(consume),(bounded_payload_test)-[:CALLS]->(signing_bytes),(signing_bytes)-[:CALLS]->(fence_well_formed),(prepare)-[:CALLS]->(prepare_fence_mode),(prepare_fenced)-[:CALLS]->(prepare_fence_mode),(prepare_fence_mode)-[:CALLS]->(validate_context),(validate_context)-[:USES]->(err),(validate_fence)-[:CALLS]->(digest_valid);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (m:Module {name:"task_execution",type:"module"}),(prepared:Class {name:"PreparedTaskCliExecution",type:"class"}),(consume:Function {name:"prepare_and_consume_verified_task_run_spawn_fence",type:"function"});
//! CREATE (directory:Class {name:"TaskRunEngineeringRunIdentityV1",type:"class",language:"rust"}),(fixture:Function {name:"profile_bound_fence_tests::binding",type:"function",language:"rust"}),(m)-[:CONTAINS]->(fixture),(prepared)-[:USES]->(directory),(consume)-[:USES]->(directory),(fixture)-[:USES]->(directory);
//! MATCH (signatureTest:Function {name:"profile_bound_grant_signature_covers_fence",type:"function"}),(consume:Function {name:"prepare_and_consume_verified_task_run_spawn_fence",type:"function"}),(registry:Class {name:"CliSessionRegistry",type:"class"});
//! CREATE (openRegistry:Function {name:"CliSessionRegistry::open",type:"function",language:"rust"}),(signatureTest)-[:CALLS]->(consume),(signatureTest)-[:CALLS]->(openRegistry),(openRegistry)-[:USES]->(registry);

use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use ring::signature::{self, Ed25519KeyPair, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use star_dto::task_run::{
    TaskRunEngineeringRunIdentityV1, TaskRunSpawnFence, TaskRunSpawnFenceBindingV2,
};
use thiserror::Error;
use uuid::Uuid;

use crate::cli_session_registry::{CliSessionRegistry, CliSessionRegistryError};

#[path = "task_pty.rs"]
#[allow(dead_code)]
pub(crate) mod pty;

/// Authorization facts resolved by the server for one Task Card CLI request.
/// The caller must consume `nonce` atomically before spawning a process.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskExecutionContext {
    /// Tenant authorized to run the task.
    pub tenant_id: Uuid,
    /// Project containing the selected Worktree.
    pub project_id: Uuid,
    /// Repository mounted by the selected Worktree.
    pub repository_id: Uuid,
    /// Authenticated human or agent actor.
    pub actor_id: Uuid,
    /// Explicit checkout scope for this execution.
    pub worktree_id: Uuid,
    /// Canonical task / Task Card identity.
    pub work_item_id: Uuid,
    /// Local Runtime selected for this execution.
    pub runtime_id: Uuid,
    /// Server-approved command profile selected for this execution.
    pub approved_launch_profile_id: Uuid,
    /// Immutable policy revision evaluated when the grant was issued.
    pub policy_version: i64,
    /// Time the short-lived grant was issued.
    pub issued_at: DateTime<Utc>,
    /// Expiration time; grants are limited to five minutes.
    pub expires_at: DateTime<Utc>,
    /// Unique grant nonce that the caller must consume exactly once.
    pub nonce: Uuid,
    /// Optional V2 directory/dual-Profile fence; absent only for legacy non-Run CLI grants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_fence: Option<TaskRunSpawnFence>,
}

/// Server-issued Task CLI grant signed over the complete immutable execution context.
/// The signature is an integrity token, not an encrypted payload or a browser session token.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedTaskExecutionGrant {
    /// Key identifier used to select an issuer public key during rotation.
    pub key_id: String,
    /// Trusted server-resolved scope and launch authorization facts.
    pub context: TaskExecutionContext,
    /// Lowercase hex Ed25519 signature over the versioned canonical payload; do not log it.
    pub signature: String,
}

/// Grant verification failures; callers must fail closed for every variant.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum TaskExecutionGrantVerificationError {
    /// Issuer key material or key identifier is invalid.
    #[error("task execution grant key is invalid")]
    InvalidKey,
    /// No configured public key matches the signed grant's key id.
    #[error("task execution grant issuer is unknown")]
    UnknownKey,
    /// The signature is malformed or does not authenticate the supplied context.
    #[error("task execution grant signature is invalid")]
    InvalidSignature,
    /// The signed context could not be encoded for verification.
    #[error("task execution grant payload is invalid")]
    InvalidPayload,
}

/// Issuer-side signer. The PKCS#8 private key belongs only in the trusted API signer process.
pub struct Ed25519TaskExecutionGrantSigner {
    key_id: String,
    private_key_pkcs8: Arc<[u8]>,
}

impl Ed25519TaskExecutionGrantSigner {
    /// Load an issuer key and validate it immediately. The caller must obtain PKCS#8 from a
    /// secret manager; never load it in a Local Runtime verifier process.
    pub fn new(
        key_id: impl Into<String>,
        private_key_pkcs8: impl AsRef<[u8]>,
    ) -> Result<Self, TaskExecutionGrantVerificationError> {
        let key_id = key_id.into();
        let private_key_pkcs8 = private_key_pkcs8.as_ref();
        if !valid_grant_key_id(&key_id) || Ed25519KeyPair::from_pkcs8(private_key_pkcs8).is_err() {
            return Err(TaskExecutionGrantVerificationError::InvalidKey);
        }
        Ok(Self {
            key_id,
            private_key_pkcs8: Arc::from(private_key_pkcs8),
        })
    }

    /// Sign the complete context at a trusted API boundary after current ACL checks.
    pub fn sign(
        &self,
        context: TaskExecutionContext,
    ) -> Result<SignedTaskExecutionGrant, TaskExecutionGrantVerificationError> {
        let payload = grant_signing_bytes(&self.key_id, &context)?;
        let pair = Ed25519KeyPair::from_pkcs8(&self.private_key_pkcs8)
            .map_err(|_| TaskExecutionGrantVerificationError::InvalidKey)?;
        let signature = pair.sign(&payload);
        Ok(SignedTaskExecutionGrant {
            key_id: self.key_id.clone(),
            context,
            signature: hex::encode(signature.as_ref()),
        })
    }
}

/// Runtime-side public-key verifier. It has no signing capability or private key material.
pub struct Ed25519TaskExecutionGrantVerifier {
    public_keys: HashMap<String, Arc<[u8]>>,
}

impl Ed25519TaskExecutionGrantVerifier {
    /// Configure trusted issuer keys by rotation-safe key id; each Ed25519 public key is 32 bytes.
    /// Keep retired keys configured until all grants signed by them have expired (maximum five
    /// minutes) so key rotation does not invalidate still-live grants.
    pub fn new(
        public_keys: HashMap<String, Vec<u8>>,
    ) -> Result<Self, TaskExecutionGrantVerificationError> {
        if public_keys.is_empty()
            || public_keys
                .iter()
                .any(|(key_id, public_key)| !valid_grant_key_id(key_id) || public_key.len() != 32)
        {
            return Err(TaskExecutionGrantVerificationError::InvalidKey);
        }
        Ok(Self {
            public_keys: public_keys
                .into_iter()
                .map(|(key_id, key)| (key_id, Arc::from(key)))
                .collect(),
        })
    }

    /// Verify an untrusted grant received by the Local Runtime.
    pub fn verify(
        &self,
        grant: &SignedTaskExecutionGrant,
    ) -> Result<TaskExecutionContext, TaskExecutionGrantVerificationError> {
        if !valid_grant_key_id(&grant.key_id) || grant.signature.len() != 128 {
            return Err(TaskExecutionGrantVerificationError::InvalidSignature);
        }
        let public_key = self
            .public_keys
            .get(&grant.key_id)
            .ok_or(TaskExecutionGrantVerificationError::UnknownKey)?;
        let signature = hex::decode(&grant.signature)
            .map_err(|_| TaskExecutionGrantVerificationError::InvalidSignature)?;
        let payload = grant_signing_bytes(&grant.key_id, &grant.context)?;
        UnparsedPublicKey::new(&signature::ED25519, public_key.as_ref())
            .verify(&payload, &signature)
            .map_err(|_| TaskExecutionGrantVerificationError::InvalidSignature)?;
        Ok(grant.context.clone())
    }
}

/// Verifier abstraction permits a host-backed asymmetric/KMS verifier without changing the
/// Local Runtime execution sequence.
pub trait TaskExecutionGrantVerifier: Send + Sync {
    /// Authenticate the issuer and integrity of a grant before trusting any context field.
    fn verify(
        &self,
        grant: &SignedTaskExecutionGrant,
    ) -> Result<TaskExecutionContext, TaskExecutionGrantVerificationError>;
}

impl TaskExecutionGrantVerifier for Ed25519TaskExecutionGrantVerifier {
    fn verify(
        &self,
        grant: &SignedTaskExecutionGrant,
    ) -> Result<TaskExecutionContext, TaskExecutionGrantVerificationError> {
        Ed25519TaskExecutionGrantVerifier::verify(self, grant)
    }
}

/// Stable domain-separated payload used by both issuer and Runtime verifier.
pub fn grant_signing_bytes(
    key_id: &str,
    context: &TaskExecutionContext,
) -> Result<Vec<u8>, TaskExecutionGrantVerificationError> {
    if context
        .spawn_fence
        .as_ref()
        .is_some_and(|fence| !fence.is_well_formed())
    {
        return Err(TaskExecutionGrantVerificationError::InvalidPayload);
    }
    let domain: &[u8] = if context.spawn_fence.is_some() {
        b"star.task-cli.execution-grant.v3\0"
    } else {
        b"star.task-cli.execution-grant.v1\0"
    };
    if !valid_grant_key_id(key_id) {
        return Err(TaskExecutionGrantVerificationError::InvalidPayload);
    }
    let serialized = serde_json::to_vec(context)
        .map_err(|_| TaskExecutionGrantVerificationError::InvalidPayload)?;
    let mut payload = Vec::with_capacity(domain.len() + key_id.len() + 1 + serialized.len());
    payload.extend_from_slice(domain);
    payload.extend_from_slice(key_id.as_bytes());
    payload.push(0);
    payload.extend_from_slice(&serialized);
    Ok(payload)
}

fn valid_grant_key_id(key_id: &str) -> bool {
    !key_id.is_empty()
        && key_id.len() <= 128
        && key_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// Runtime mount identity paired with the canonical checkout resolved by Worktree/Git services.
/// `mounted_path` is runtime-reported and is never trusted on its own.
#[derive(Clone, Debug)]
pub struct RuntimeWorktreeBinding {
    /// Tenant owning the mounted checkout.
    pub tenant_id: Uuid,
    /// Project owning the mounted checkout.
    pub project_id: Uuid,
    /// Repository owning the mounted checkout.
    pub repository_id: Uuid,
    /// Worktree mounted in this Runtime.
    pub worktree_id: Uuid,
    /// Local Runtime that reported the mount.
    pub runtime_id: Uuid,
    /// Runtime-reported path, which must be verified against the canonical path.
    pub mounted_path: PathBuf,
    /// Path resolved by the trusted Worktree/Git service for this Worktree.
    pub canonical_checkout_path: PathBuf,
}

/// Server-owned launch configuration. User input can select the profile ID but cannot replace
/// its executable, arguments, static environment additions, or timeout. Secret-bearing values
/// must come from a separate, audited secret capability and must never be stored in this profile.
#[derive(Clone, Debug)]
pub struct ApprovedTaskLaunchProfile {
    /// Stable profile identity.
    pub id: Uuid,
    /// Immutable profile revision.
    pub version: i64,
    /// Lowercase SHA-256 digest of the canonical approved profile document.
    pub content_digest: String,
    /// Absolute executable path approved by an administrator.
    pub executable: PathBuf,
    /// Fixed arguments approved by the profile; no user-supplied command text.
    pub args: Vec<String>,
    /// Explicit non-secret environment additions approved for this profile.
    pub static_environment: HashMap<String, String>,
    /// Process time limit in seconds.
    pub max_runtime_seconds: u32,
}

/// Fully resolved CLI spawn request; `worktree_dir` comes only from the canonical binding.
#[derive(Clone, Debug)]
pub struct PreparedTaskCliExecution {
    /// Verified canonical directory identity for a Task Run; absent for legacy non-Run grants.
    pub engineering_run: Option<TaskRunEngineeringRunIdentityV1>,
    /// Authorized tenant.
    pub tenant_id: Uuid,
    /// Project containing the selected Worktree.
    pub project_id: Uuid,
    /// Repository containing the canonical checkout.
    pub repository_id: Uuid,
    /// Authorized actor.
    pub actor_id: Uuid,
    /// Explicit Worktree scope.
    pub worktree_id: Uuid,
    /// Canonical task identity.
    pub work_item_id: Uuid,
    /// Selected Local Runtime.
    pub runtime_id: Uuid,
    /// Approved profile identity.
    pub profile_id: Uuid,
    /// Immutable approved profile revision.
    pub profile_version: i64,
    /// Policy revision used to authorize execution.
    pub policy_version: i64,
    /// Single-use grant nonce.
    pub nonce: Uuid,
    /// In-memory Runtime fence correlation only; never persist this opaque ID in Run/BI records.
    pub spawn_fence_id: Option<Uuid>,
    /// Digest of the complete dual-Profile fence binding for an outcome receipt.
    pub spawn_fence_binding_digest: Option<[u8; 32]>,
    /// Grant issue time.
    pub issued_at: DateTime<Utc>,
    /// Grant expiration time.
    pub expires_at: DateTime<Utc>,
    /// Resolved approved executable path.
    pub command: PathBuf,
    /// Fixed approved executable arguments.
    pub args: Vec<String>,
    /// Approved non-secret static environment additions only.
    pub static_environment: HashMap<String, String>,
    /// Canonical Worktree checkout selected from the trusted binding.
    pub worktree_dir: PathBuf,
    /// Maximum execution duration.
    pub max_runtime_seconds: u32,
}

/// Errors that prevent preparation of a Task Card CLI spawn request.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TaskExecutionError {
    #[error("task execution scope does not match the runtime Worktree binding")]
    /// Grant identifiers do not match the selected Project, Worktree, Runtime, or profile.
    ScopeMismatch,
    #[error("task execution grant is missing, expired, or outside the allowed lifetime")]
    /// Grant is invalid or no longer live.
    InvalidGrant,
    #[error("approved launch profile is invalid")]
    /// Profile contains an invalid executable, argument, environment, or time limit.
    InvalidProfile,
    /// Profile-bound grants must pass through the dual-fence consumer.
    #[error("profile-bound task grant requires the spawn-fence consumer")]
    ProfileBoundGrantRequiresFenceConsumer,
    #[error("runtime Worktree checkout could not be verified")]
    /// Runtime mount is unavailable or differs from the canonical Worktree checkout.
    InvalidCheckout,
    #[error("task execution grant nonce was already consumed")]
    /// The one-time grant nonce was already consumed by this Runtime.
    GrantReplay,
    /// The one-time Profile-bound fence was already consumed by this Runtime.
    #[error("task Run spawn fence was already consumed")]
    SpawnFenceReplay,
    /// A dual-Profile fence is required for Profile-bound Run admission.
    #[error("task Run spawn fence is required")]
    SpawnFenceRequired,
    /// The fence does not match the current Runtime scope or authority state.
    #[error("task Run spawn fence binding is invalid")]
    SpawnFenceMismatch,
    /// The short-lived fence is expired or outside its bounded issue window.
    #[error("task Run spawn fence is expired")]
    SpawnFenceExpired,
    #[error("task execution grant nonce store is unavailable")]
    /// Durable grant nonce storage could not complete the consume operation.
    NonceStoreUnavailable,
}

/// Failure returned by the durable one-time grant nonce store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskExecutionNonceStoreError {
    /// The grant has expired or its tenant/nonce/expiry is invalid.
    InvalidOrExpired,
    /// A previous request already consumed this nonce.
    Replay,
    /// A previous request already consumed this Runtime fence.
    FenceReplay,
    /// The durable store failed and execution must not proceed.
    Unavailable,
}

/// Durable one-time grant nonce consumption used immediately before process start.
pub trait TaskExecutionNonceStore {
    /// Consume a grant nonce once for a tenant and expiry window.
    fn consume(
        &self,
        tenant_id: Uuid,
        nonce: Uuid,
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), TaskExecutionNonceStoreError>;

    /// Atomically consume a Profile-bound grant nonce and Runtime fence in one durable transaction.
    /// Implementations without that boundary fail closed by default.
    fn consume_profile_bound(
        &self,
        _tenant_id: Uuid,
        _nonce: Uuid,
        _fence_id: Uuid,
        _binding_digest: [u8; 32],
        _expires_at: DateTime<Utc>,
        _now: DateTime<Utc>,
    ) -> Result<(), TaskExecutionNonceStoreError> {
        Err(TaskExecutionNonceStoreError::Unavailable)
    }
}

impl TaskExecutionNonceStore for CliSessionRegistry {
    fn consume(
        &self,
        tenant_id: Uuid,
        nonce: Uuid,
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), TaskExecutionNonceStoreError> {
        self.consume_task_grant_nonce(tenant_id, nonce, expires_at, now)
            .map_err(|error| match error {
                CliSessionRegistryError::GrantNonceReplay => TaskExecutionNonceStoreError::Replay,
                CliSessionRegistryError::InvalidGrantNonce => {
                    TaskExecutionNonceStoreError::InvalidOrExpired
                }
                _ => TaskExecutionNonceStoreError::Unavailable,
            })
    }

    fn consume_profile_bound(
        &self,
        tenant_id: Uuid,
        nonce: Uuid,
        fence_id: Uuid,
        binding_digest: [u8; 32],
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), TaskExecutionNonceStoreError> {
        self.consume_task_run_spawn_fence(
            tenant_id,
            nonce,
            fence_id,
            binding_digest,
            expires_at,
            now,
        )
        .map_err(|error| match error {
            CliSessionRegistryError::GrantNonceReplay => TaskExecutionNonceStoreError::Replay,
            CliSessionRegistryError::SpawnFenceReplay => TaskExecutionNonceStoreError::FenceReplay,
            CliSessionRegistryError::InvalidGrantNonce => {
                TaskExecutionNonceStoreError::InvalidOrExpired
            }
            _ => TaskExecutionNonceStoreError::Unavailable,
        })
    }
}

const MAX_GRANT_LIFETIME: Duration = Duration::minutes(5);
const MAX_ARGUMENTS: usize = 128;
const MAX_ENVIRONMENT_ENTRIES: usize = 64;
const MAX_TEXT_BYTES: usize = 8_192;

/// Resolve a Task Card execution to a profile and exact canonical checkout.
/// This helper validates the consistency and lifetime of a grant; it does not authenticate its
/// issuer, verify a signature, or prove that its fields came from the authorization service. Only
/// call it with a grant already authenticated by the trusted server boundary. Callers still need
/// current ACL revalidation, durable single-use nonce consumption, process supervision, audit
/// writes, and output redaction before spawn.
pub fn prepare_task_cli_execution(
    context: &TaskExecutionContext,
    binding: &RuntimeWorktreeBinding,
    profile: &ApprovedTaskLaunchProfile,
    now: DateTime<Utc>,
) -> Result<PreparedTaskCliExecution, TaskExecutionError> {
    prepare_task_cli_execution_with_fence_mode(context, binding, profile, now, false)
}

fn prepare_task_cli_execution_with_fence_mode(
    context: &TaskExecutionContext,
    binding: &RuntimeWorktreeBinding,
    profile: &ApprovedTaskLaunchProfile,
    now: DateTime<Utc>,
    allow_profile_bound_fence: bool,
) -> Result<PreparedTaskCliExecution, TaskExecutionError> {
    validate_context(context, binding, profile, now, allow_profile_bound_fence)?;
    validate_profile(profile)?;

    let mounted_path = canonical_directory(&binding.mounted_path)?;
    let checkout_path = canonical_directory(&binding.canonical_checkout_path)?;
    if mounted_path != checkout_path {
        return Err(TaskExecutionError::InvalidCheckout);
    }
    let command =
        fs::canonicalize(&profile.executable).map_err(|_| TaskExecutionError::InvalidProfile)?;
    if !command.is_file() {
        return Err(TaskExecutionError::InvalidProfile);
    }

    Ok(PreparedTaskCliExecution {
        engineering_run: None,
        tenant_id: context.tenant_id,
        project_id: context.project_id,
        repository_id: context.repository_id,
        actor_id: context.actor_id,
        worktree_id: context.worktree_id,
        work_item_id: context.work_item_id,
        runtime_id: context.runtime_id,
        profile_id: profile.id,
        profile_version: profile.version,
        policy_version: context.policy_version,
        nonce: context.nonce,
        spawn_fence_id: None,
        spawn_fence_binding_digest: None,
        issued_at: context.issued_at,
        expires_at: context.expires_at,
        command,
        args: profile.args.clone(),
        static_environment: profile.static_environment.clone(),
        worktree_dir: checkout_path,
        max_runtime_seconds: profile.max_runtime_seconds,
    })
}

/// Validate and durably consume a short-lived Task Card CLI grant before spawn.
///
/// The caller must authenticate the grant issuer/signature and recheck current ACL and Runtime
/// health before calling. The nonce is retained as a short-lived replay receipt; any store
/// failure is fail-closed. This function still does not start or supervise a process.
pub fn prepare_and_consume_task_cli_execution(
    context: &TaskExecutionContext,
    binding: &RuntimeWorktreeBinding,
    profile: &ApprovedTaskLaunchProfile,
    now: DateTime<Utc>,
    nonce_store: &impl TaskExecutionNonceStore,
) -> Result<PreparedTaskCliExecution, TaskExecutionError> {
    let prepared = prepare_task_cli_execution(context, binding, profile, now.clone())?;
    nonce_store
        .consume(context.tenant_id, context.nonce, context.expires_at, now)
        .map_err(|error| match error {
            TaskExecutionNonceStoreError::Replay => TaskExecutionError::GrantReplay,
            TaskExecutionNonceStoreError::FenceReplay => TaskExecutionError::SpawnFenceReplay,
            TaskExecutionNonceStoreError::InvalidOrExpired => TaskExecutionError::InvalidGrant,
            TaskExecutionNonceStoreError::Unavailable => TaskExecutionError::NonceStoreUnavailable,
        })?;
    Ok(prepared)
}

/// Authenticate the grant signature, then validate scope/profile/checkout and durably consume
/// its nonce. The caller must still recheck current ACL and Runtime health immediately before
/// process spawn and write the required TaskRun audit record before creating side effects.
pub fn prepare_and_consume_verified_task_cli_execution(
    grant: &SignedTaskExecutionGrant,
    verifier: &impl TaskExecutionGrantVerifier,
    binding: &RuntimeWorktreeBinding,
    profile: &ApprovedTaskLaunchProfile,
    now: DateTime<Utc>,
    nonce_store: &impl TaskExecutionNonceStore,
) -> Result<PreparedTaskCliExecution, TaskExecutionError> {
    let context = verifier
        .verify(grant)
        .map_err(|_| TaskExecutionError::InvalidGrant)?;
    prepare_and_consume_task_cli_execution(&context, binding, profile, now, nonce_store)
}

/// Authenticate and atomically consume the directory/dual-Profile fence before a future
/// production spawn adapter. `current_binding` must be freshly rebuilt from authoritative ACL,
/// active Project/Branch/Run grants and revisions, verified current Git checkout, Task/lifecycle,
/// Profile/catalog/HookSet sources and committed ResourceBudget; this function does not itself
/// provide those sources or create a process.
pub fn prepare_and_consume_verified_task_run_spawn_fence(
    grant: &SignedTaskExecutionGrant,
    verifier: &impl TaskExecutionGrantVerifier,
    worktree_binding: &RuntimeWorktreeBinding,
    profile: &ApprovedTaskLaunchProfile,
    current_binding: &TaskRunSpawnFenceBindingV2,
    now: DateTime<Utc>,
    nonce_store: &impl TaskExecutionNonceStore,
) -> Result<PreparedTaskCliExecution, TaskExecutionError> {
    let context = verifier
        .verify(grant)
        .map_err(|_| TaskExecutionError::InvalidGrant)?;
    let fence = context
        .spawn_fence
        .as_ref()
        .ok_or(TaskExecutionError::SpawnFenceRequired)?;
    validate_profile_bound_fence(&context, profile, fence, current_binding, now.clone())?;
    let mut prepared = prepare_task_cli_execution_with_fence_mode(
        &context,
        worktree_binding,
        profile,
        now.clone(),
        true,
    )?;
    let binding_digest = fence
        .binding
        .binding_digest()
        .map_err(|_| TaskExecutionError::SpawnFenceMismatch)?;
    nonce_store
        .consume_profile_bound(
            context.tenant_id,
            context.nonce,
            fence.fence_id,
            binding_digest,
            fence.expires_at,
            now,
        )
        .map_err(|error| match error {
            TaskExecutionNonceStoreError::Replay => TaskExecutionError::GrantReplay,
            TaskExecutionNonceStoreError::FenceReplay => TaskExecutionError::SpawnFenceReplay,
            TaskExecutionNonceStoreError::InvalidOrExpired => TaskExecutionError::SpawnFenceExpired,
            TaskExecutionNonceStoreError::Unavailable => TaskExecutionError::NonceStoreUnavailable,
        })?;
    prepared.spawn_fence_id = Some(fence.fence_id);
    prepared.spawn_fence_binding_digest = Some(binding_digest);
    prepared.engineering_run = Some(fence.binding.engineering_run.clone());
    Ok(prepared)
}

fn validate_profile_bound_fence(
    context: &TaskExecutionContext,
    profile: &ApprovedTaskLaunchProfile,
    fence: &TaskRunSpawnFence,
    current_binding: &TaskRunSpawnFenceBindingV2,
    now: DateTime<Utc>,
) -> Result<(), TaskExecutionError> {
    if fence.expires_at <= now {
        return Err(TaskExecutionError::SpawnFenceExpired);
    }
    if !fence.is_valid_at(now.clone()) {
        return Err(TaskExecutionError::SpawnFenceMismatch);
    }
    if &fence.binding != current_binding
        || !current_binding.is_valid()
        || fence.binding.tenant_id != context.tenant_id
        || fence.binding.actor_id != context.actor_id
        || fence.binding.project_id != context.project_id
        || fence.binding.repository_id != context.repository_id
        || fence.binding.worktree_id != context.worktree_id
        || fence.binding.work_item_id != context.work_item_id
        || fence.binding.runtime_id != context.runtime_id
        || fence.binding.approved_launch_profile.profile_id != context.approved_launch_profile_id
        || fence.binding.approved_launch_profile.profile_id != profile.id
        || u64::try_from(profile.version).ok()
            != Some(fence.binding.approved_launch_profile.version)
        || profile.content_digest != fence.binding.approved_launch_profile.content_digest
        || context.issued_at > fence.issued_at
        || fence.expires_at > context.expires_at
    {
        return Err(TaskExecutionError::SpawnFenceMismatch);
    }
    Ok(())
}

fn validate_context(
    context: &TaskExecutionContext,
    binding: &RuntimeWorktreeBinding,
    profile: &ApprovedTaskLaunchProfile,
    now: DateTime<Utc>,
    allow_profile_bound_fence: bool,
) -> Result<(), TaskExecutionError> {
    if context.spawn_fence.is_some() && !allow_profile_bound_fence {
        return Err(TaskExecutionError::ProfileBoundGrantRequiresFenceConsumer);
    }
    let ids = [
        context.tenant_id,
        context.project_id,
        context.repository_id,
        context.actor_id,
        context.worktree_id,
        context.work_item_id,
        context.runtime_id,
        context.approved_launch_profile_id,
        context.nonce,
    ];
    if ids.contains(&Uuid::nil())
        || context.policy_version <= 0
        || context.issued_at > now
        || context.expires_at <= now
        || context.expires_at - context.issued_at > MAX_GRANT_LIFETIME
    {
        return Err(TaskExecutionError::InvalidGrant);
    }
    if context.tenant_id != binding.tenant_id
        || context.project_id != binding.project_id
        || context.repository_id != binding.repository_id
        || context.worktree_id != binding.worktree_id
        || context.runtime_id != binding.runtime_id
        || context.approved_launch_profile_id != profile.id
    {
        return Err(TaskExecutionError::ScopeMismatch);
    }
    Ok(())
}

fn validate_profile(profile: &ApprovedTaskLaunchProfile) -> Result<(), TaskExecutionError> {
    if profile.id.is_nil()
        || profile.version <= 0
        || !profile.executable.is_absolute()
        || profile.args.len() > MAX_ARGUMENTS
        || profile.max_runtime_seconds == 0
        || profile.max_runtime_seconds > 4 * 60 * 60
        || !is_lower_hex_sha256_digest(&profile.content_digest)
        || profile
            .args
            .iter()
            .any(|arg| arg.len() > MAX_TEXT_BYTES || arg.as_bytes().contains(&0))
        || profile.static_environment.len() > MAX_ENVIRONMENT_ENTRIES
        || profile.static_environment.iter().any(|(key, value)| {
            !is_allowed_profile_env_key(key)
                || value.len() > MAX_TEXT_BYTES
                || value.as_bytes().contains(&0)
        })
    {
        return Err(TaskExecutionError::InvalidProfile);
    }
    Ok(())
}

fn is_lower_hex_sha256_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_directory(path: &std::path::Path) -> Result<PathBuf, TaskExecutionError> {
    let canonical = fs::canonicalize(path).map_err(|_| TaskExecutionError::InvalidCheckout)?;
    if !canonical.is_dir() {
        return Err(TaskExecutionError::InvalidCheckout);
    }
    Ok(canonical)
}

fn is_allowed_profile_env_key(key: &str) -> bool {
    if key.is_empty()
        || key.len() > 128
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return false;
    }
    let parts = key.split('_').collect::<Vec<_>>();
    let blocked_secret_parts = [
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "CREDENTIAL",
        "CREDENTIALS",
        "PRIVATE",
        "COOKIE",
        "SESSION",
        "CERT",
        "CERTIFICATE",
    ];
    !matches!(
        key,
        "PATH"
            | "HOME"
            | "USERPROFILE"
            | "SYSTEMROOT"
            | "COMSPEC"
            | "LD_PRELOAD"
            | "DYLD_INSERT_LIBRARIES"
            | "BASH_ENV"
            | "ENV"
    ) && !parts.iter().any(|part| blocked_secret_parts.contains(part))
        && !(parts.windows(2).any(|pair| pair == ["API", "KEY"]))
        && !(parts.windows(2).any(|pair| pair == ["ACCESS", "KEY"]))
}

#[cfg(test)]
mod profile_bound_fence_tests {
    use super::*;
    use chrono::TimeZone;
    use ring::{rand::SystemRandom, signature::KeyPair};
    use star_dto::task_run::{
        TaskRunCatalogRevisionIdentity, TaskRunHookSetIdentity, TaskRunProfileRevisionIdentity,
        TaskRunResourceBudget,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct NonceConsumeCounter(AtomicUsize);

    impl TaskExecutionNonceStore for NonceConsumeCounter {
        fn consume(
            &self,
            _tenant_id: Uuid,
            _nonce: Uuid,
            _expires_at: DateTime<Utc>,
            _now: DateTime<Utc>,
        ) -> Result<(), TaskExecutionNonceStoreError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn binding() -> TaskRunSpawnFenceBindingV2 {
        let engineering_run = TaskRunEngineeringRunIdentityV1 {
            tenant_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            branch_id: Uuid::new_v4(),
            branch_full_ref: "refs/heads/dev".to_owned(),
            engineering_run_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            work_item_id: Uuid::new_v4(),
            worktree_project_binding_id: Uuid::new_v4(),
            worktree_project_binding_version: 1,
            project_role_binding_id: Uuid::new_v4(),
            project_role_binding_version: 1,
            project_role: "developer".to_owned(),
            branch_revision_id: Uuid::new_v4(),
            branch_revision_version: 1,
            branch_role_binding_id: Uuid::new_v4(),
            branch_role_binding_version: 1,
            branch_role: "developer".to_owned(),
            engineering_run_revision_id: Uuid::new_v4(),
            engineering_run_revision_version: 1,
            engineering_run_role_binding_id: Uuid::new_v4(),
            engineering_run_role_binding_version: 1,
            engineering_run_role: "agent".to_owned(),
            engineering_run_worktree_binding_id: Uuid::new_v4(),
            engineering_run_worktree_binding_version: 1,
        };
        TaskRunSpawnFenceBindingV2 {
            tenant_id: engineering_run.tenant_id,
            actor_id: Uuid::new_v4(),
            project_id: engineering_run.project_id,
            repository_id: engineering_run.repository_id,
            worktree_id: engineering_run.worktree_id,
            work_item_id: engineering_run.work_item_id,
            runtime_id: Uuid::new_v4(),
            expected_lifecycle_version: 1,
            approved_launch_profile: TaskRunProfileRevisionIdentity {
                profile_id: Uuid::new_v4(),
                version: 3,
                content_digest: "a".repeat(64),
            },
            execution_profile: TaskRunProfileRevisionIdentity {
                profile_id: Uuid::new_v4(),
                version: 7,
                content_digest: "b".repeat(64),
            },
            catalog_revisions: TaskRunCatalogRevisionIdentity {
                provider_catalog_revision: 11,
                skill_catalog_revision: 12,
                grant_set_id: Uuid::new_v4(),
                grant_set_version: 13,
            },
            hook_set: TaskRunHookSetIdentity {
                hook_set_id: Uuid::new_v4(),
                version: 2,
                effective_digest: "c".repeat(64),
            },
            engineering_run,
            resource_budget: TaskRunResourceBudget {
                max_rss_bytes: 1024,
                max_cpu_ms: 1000,
                max_runtime_ms: 20_000,
                max_child_processes: 1,
                max_parallel_tools: 2,
                max_provider_calls: 3,
                max_output_bytes: 4096,
                max_event_buffer_bytes: 2048,
            },
            request_fingerprint: [9; 32],
        }
    }

    #[test]
    fn profile_bound_fence_rejects_binding_drift_and_stale_issue_time() {
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let binding = binding();
        let fence = TaskRunSpawnFence {
            fence_id: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(20),
            binding: binding.clone(),
            binding_digest: binding.binding_digest().unwrap(),
        };
        let context = TaskExecutionContext {
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            repository_id: binding.repository_id,
            actor_id: binding.actor_id,
            worktree_id: binding.worktree_id,
            work_item_id: binding.work_item_id,
            runtime_id: binding.runtime_id,
            approved_launch_profile_id: binding.approved_launch_profile.profile_id,
            policy_version: 1,
            issued_at: now - Duration::seconds(1),
            expires_at: now + Duration::seconds(60),
            nonce: Uuid::new_v4(),
            spawn_fence: Some(fence.clone()),
        };
        let profile = ApprovedTaskLaunchProfile {
            id: binding.approved_launch_profile.profile_id,
            version: 3,
            content_digest: binding.approved_launch_profile.content_digest.clone(),
            executable: PathBuf::from("C:\\runtime\\approved.exe"),
            args: Vec::new(),
            static_environment: HashMap::new(),
            max_runtime_seconds: 20,
        };

        assert!(validate_profile_bound_fence(&context, &profile, &fence, &binding, now).is_ok());

        let mut drifted = binding.clone();
        drifted.request_fingerprint[0] ^= 1;
        assert_eq!(
            validate_profile_bound_fence(&context, &profile, &fence, &drifted, now),
            Err(TaskExecutionError::SpawnFenceMismatch)
        );
        for field in 0..10 {
            let mut current = binding.clone();
            match field {
                0 => current.engineering_run.engineering_run_id = Uuid::new_v4(),
                1 => current.engineering_run.branch_id = Uuid::new_v4(),
                2 => {
                    current
                        .engineering_run
                        .engineering_run_worktree_binding_version += 1
                }
                3 => current.engineering_run.branch_revision_version += 1,
                4 => current.engineering_run.engineering_run_revision_version += 1,
                5 => current.engineering_run.project_role_binding_version += 1,
                6 => current.engineering_run.branch_role_binding_version += 1,
                7 => current.engineering_run.engineering_run_role_binding_version += 1,
                8 => current.engineering_run.engineering_run_role = "viewer".to_owned(),
                _ => current.engineering_run.branch_full_ref = "refs/heads/other".to_owned(),
            }
            assert_eq!(
                validate_profile_bound_fence(&context, &profile, &fence, &current, now),
                Err(TaskExecutionError::SpawnFenceMismatch),
                "directory/grant change {field} must fail closed"
            );
        }
        assert_eq!(
            validate_profile_bound_fence(
                &context,
                &profile,
                &fence,
                &binding,
                now + Duration::seconds(21)
            ),
            Err(TaskExecutionError::SpawnFenceExpired)
        );
    }

    #[test]
    fn legacy_grant_signing_bytes_remain_v1_when_fence_absent() {
        let binding = binding();
        let context = TaskExecutionContext {
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            repository_id: binding.repository_id,
            actor_id: binding.actor_id,
            worktree_id: binding.worktree_id,
            work_item_id: binding.work_item_id,
            runtime_id: binding.runtime_id,
            approved_launch_profile_id: binding.approved_launch_profile.profile_id,
            policy_version: 1,
            issued_at: Utc.timestamp_opt(1_700_000_000, 0).single().unwrap(),
            expires_at: Utc.timestamp_opt(1_700_000_060, 0).single().unwrap(),
            nonce: Uuid::new_v4(),
            spawn_fence: None,
        };
        let payload = grant_signing_bytes("issuer-1", &context).unwrap();
        assert!(payload.starts_with(b"star.task-cli.execution-grant.v1\0issuer-1\0"));
        let encoded = serde_json::to_value(&context).unwrap();
        assert!(encoded.get("spawn_fence").is_none());
    }

    #[test]
    fn profile_bound_grant_signature_covers_fence() {
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let binding = binding();
        let fence = TaskRunSpawnFence {
            fence_id: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(20),
            binding_digest: binding.binding_digest().unwrap(),
            binding: binding.clone(),
        };
        let context = TaskExecutionContext {
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            repository_id: binding.repository_id,
            actor_id: binding.actor_id,
            worktree_id: binding.worktree_id,
            work_item_id: binding.work_item_id,
            runtime_id: binding.runtime_id,
            approved_launch_profile_id: binding.approved_launch_profile.profile_id,
            policy_version: 1,
            issued_at: now,
            expires_at: now + Duration::minutes(1),
            nonce: Uuid::new_v4(),
            spawn_fence: Some(fence),
        };
        let key_pair_pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
        let signer =
            Ed25519TaskExecutionGrantSigner::new("issuer-1", key_pair_pkcs8.as_ref()).unwrap();
        let grant = signer.sign(context).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(key_pair_pkcs8.as_ref()).unwrap();
        let verifier = Ed25519TaskExecutionGrantVerifier::new(HashMap::from([(
            "issuer-1".to_owned(),
            key_pair.public_key().as_ref().to_vec(),
        )]))
        .unwrap();

        assert!(verifier.verify(&grant).is_ok());
        assert!(grant_signing_bytes("issuer-1", &grant.context)
            .unwrap()
            .starts_with(b"star.task-cli.execution-grant.v3\0"));
        let checkout = tempfile::tempdir().unwrap();
        let registry =
            CliSessionRegistry::open(checkout.path().join("grant-receipts.sqlite")).unwrap();
        let runtime_binding = RuntimeWorktreeBinding {
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            repository_id: binding.repository_id,
            worktree_id: binding.worktree_id,
            runtime_id: binding.runtime_id,
            mounted_path: checkout.path().to_path_buf(),
            canonical_checkout_path: checkout.path().to_path_buf(),
        };
        let profile = ApprovedTaskLaunchProfile {
            id: binding.approved_launch_profile.profile_id,
            version: 3,
            content_digest: binding.approved_launch_profile.content_digest.clone(),
            executable: std::env::current_exe().unwrap(),
            args: Vec::new(),
            static_environment: HashMap::new(),
            max_runtime_seconds: 20,
        };
        let mut moved = binding.clone();
        moved
            .engineering_run
            .engineering_run_worktree_binding_version += 1;
        assert!(matches!(
            prepare_and_consume_verified_task_run_spawn_fence(
                &grant,
                &verifier,
                &runtime_binding,
                &profile,
                &moved,
                now,
                &registry,
            ),
            Err(TaskExecutionError::SpawnFenceMismatch)
        ));
        let prepared = prepare_and_consume_verified_task_run_spawn_fence(
            &grant,
            &verifier,
            &runtime_binding,
            &profile,
            &binding,
            now,
            &registry,
        )
        .unwrap();
        assert_eq!(
            prepared.engineering_run.as_ref(),
            Some(&binding.engineering_run)
        );
        assert!(matches!(
            prepare_and_consume_verified_task_run_spawn_fence(
                &grant,
                &verifier,
                &runtime_binding,
                &profile,
                &binding,
                now,
                &registry,
            ),
            Err(TaskExecutionError::GrantReplay)
        ));

        let mut tampered = grant.clone();
        tampered.context.spawn_fence.as_mut().unwrap().fence_id = Uuid::new_v4();
        assert!(matches!(
            verifier.verify(&tampered),
            Err(TaskExecutionGrantVerificationError::InvalidSignature)
        ));
        let mut tampered = grant;
        let fence = tampered.context.spawn_fence.as_mut().unwrap();
        fence.binding.engineering_run.engineering_run_id = Uuid::new_v4();
        fence.binding_digest = fence.binding.binding_digest().unwrap();
        assert!(matches!(
            verifier.verify(&tampered),
            Err(TaskExecutionGrantVerificationError::InvalidSignature)
        ));
    }

    #[test]
    fn legacy_consumer_rejects_profile_bound_grant() {
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let binding = binding();
        let fence = TaskRunSpawnFence {
            fence_id: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(20),
            binding: binding.clone(),
            binding_digest: binding.binding_digest().unwrap(),
        };
        let context = TaskExecutionContext {
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            repository_id: binding.repository_id,
            actor_id: binding.actor_id,
            worktree_id: binding.worktree_id,
            work_item_id: binding.work_item_id,
            runtime_id: binding.runtime_id,
            approved_launch_profile_id: binding.approved_launch_profile.profile_id,
            policy_version: 1,
            issued_at: now,
            expires_at: now + Duration::minutes(1),
            nonce: Uuid::new_v4(),
            spawn_fence: Some(fence),
        };
        let runtime_binding = RuntimeWorktreeBinding {
            tenant_id: context.tenant_id,
            project_id: context.project_id,
            repository_id: context.repository_id,
            worktree_id: context.worktree_id,
            runtime_id: context.runtime_id,
            mounted_path: PathBuf::from("C:\\runtime\\missing-checkout"),
            canonical_checkout_path: PathBuf::from("C:\\runtime\\missing-checkout"),
        };
        let profile = ApprovedTaskLaunchProfile {
            id: context.approved_launch_profile_id,
            version: 3,
            content_digest: "a".repeat(64),
            executable: PathBuf::from("C:\\runtime\\approved.exe"),
            args: Vec::new(),
            static_environment: HashMap::new(),
            max_runtime_seconds: 20,
        };

        let nonce_store = NonceConsumeCounter(AtomicUsize::new(0));
        assert!(matches!(
            prepare_and_consume_task_cli_execution(
                &context,
                &runtime_binding,
                &profile,
                now,
                &nonce_store,
            ),
            Err(TaskExecutionError::ProfileBoundGrantRequiresFenceConsumer)
        ));
        assert_eq!(nonce_store.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn signed_fence_payload_rejects_unbounded_digest_before_serialization() {
        let now = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let mut binding = binding();
        binding.approved_launch_profile.content_digest = "a".repeat(1_000_000);
        let fence = TaskRunSpawnFence {
            fence_id: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(20),
            binding,
            binding_digest: [0; 32],
        };
        let context = TaskExecutionContext {
            tenant_id: fence.binding.tenant_id,
            project_id: fence.binding.project_id,
            repository_id: fence.binding.repository_id,
            actor_id: fence.binding.actor_id,
            worktree_id: fence.binding.worktree_id,
            work_item_id: fence.binding.work_item_id,
            runtime_id: fence.binding.runtime_id,
            approved_launch_profile_id: fence.binding.approved_launch_profile.profile_id,
            policy_version: 1,
            issued_at: now,
            expires_at: now + Duration::minutes(1),
            nonce: Uuid::new_v4(),
            spawn_fence: Some(fence),
        };

        assert_eq!(
            grant_signing_bytes("issuer-1", &context),
            Err(TaskExecutionGrantVerificationError::InvalidPayload)
        );
    }
}
