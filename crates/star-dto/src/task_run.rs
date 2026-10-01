//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"task_run.rs",type:"file",language:"rust"}),(m:Module {name:"task_run",type:"module",language:"rust"}),
//!   (profile:Class {name:"TaskRunProfileRevisionIdentity",type:"class",language:"rust"}),(catalog:Class {name:"TaskRunCatalogRevisionIdentity",type:"class",language:"rust"}),(hook:Class {name:"TaskRunHookSetIdentity",type:"class",language:"rust"}),(budget:Class {name:"TaskRunResourceBudget",type:"class",language:"rust"}),(binding:Class {name:"TaskRunSpawnFenceBindingV1",type:"class",language:"rust"}),(fence:Class {name:"TaskRunSpawnFence",type:"class",language:"rust"}),
//!   (digest:Function {name:"TaskRunSpawnFenceBindingV1::binding_digest",type:"function",language:"rust"}),(binding_valid:Function {name:"TaskRunSpawnFenceBindingV1::is_valid",type:"function",language:"rust"}),(well_formed:Function {name:"TaskRunSpawnFence::is_well_formed",type:"function",language:"rust"}),(valid:Function {name:"TaskRunSpawnFence::is_valid_at",type:"function",language:"rust"}),(digest_valid:Function {name:"is_lower_hex_sha256_digest",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(profile),(m)-[:CONTAINS]->(catalog),(m)-[:CONTAINS]->(hook),(m)-[:CONTAINS]->(budget),(m)-[:CONTAINS]->(binding),(m)-[:CONTAINS]->(fence),(binding)-[:HAS_METHOD]->(digest),(binding)-[:HAS_METHOD]->(binding_valid),(fence)-[:HAS_METHOD]->(well_formed),(fence)-[:HAS_METHOD]->(valid),(well_formed)-[:CALLS]->(binding_valid),(well_formed)-[:CALLS]->(digest),(valid)-[:CALLS]->(well_formed),(well_formed)-[:CALLS]->(digest_valid);

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Immutable revision identity resolved by the authority responsible for a Task Run profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRunProfileRevisionIdentity {
    /// Stable profile ID.
    pub profile_id: Uuid,
    /// Immutable profile revision.
    pub version: u64,
    /// Lowercase SHA-256 digest of the canonical profile document.
    pub content_digest: String,
}

/// Compact catalog revision tuple; catalog entries remain in their bounded shared snapshot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRunCatalogRevisionIdentity {
    /// Current provider catalog revision.
    pub provider_catalog_revision: u64,
    /// Current skill registry revision.
    pub skill_catalog_revision: u64,
    /// GrantSet identity.
    pub grant_set_id: Uuid,
    /// Immutable GrantSet revision.
    pub grant_set_version: u64,
}

/// Effective HookSet revision applied to this admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRunHookSetIdentity {
    /// HookSet identity.
    pub hook_set_id: Uuid,
    /// Published HookSet revision.
    pub version: u64,
    /// Lowercase SHA-256 digest of the effective inherited HookSet.
    pub effective_digest: String,
}

/// Fixed-size Run resource ceilings carried across the Runtime boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRunResourceBudget {
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

/// Bounded, versioned cross-crate representation of all facts that authorize one Runtime spawn.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRunSpawnFenceBindingV1 {
    /// Tenant scope.
    pub tenant_id: Uuid,
    /// Authenticated actor scope.
    pub actor_id: Uuid,
    /// Project scope.
    pub project_id: Uuid,
    /// Repository scope.
    pub repository_id: Uuid,
    /// Worktree scope.
    pub worktree_id: Uuid,
    /// Task Card scope.
    pub work_item_id: Uuid,
    /// Target Runtime scope.
    pub runtime_id: Uuid,
    /// Lifecycle revision expected at admission.
    pub expected_lifecycle_version: i32,
    /// Approved OS command profile identity.
    pub approved_launch_profile: TaskRunProfileRevisionIdentity,
    /// AgentExecutionProfile identity.
    pub execution_profile: TaskRunProfileRevisionIdentity,
    /// Current catalog revision tuple.
    pub catalog_revisions: TaskRunCatalogRevisionIdentity,
    /// Verified effective HookSet identity.
    pub hook_set: TaskRunHookSetIdentity,
    /// Fixed resource ceilings for this Run.
    pub resource_budget: TaskRunResourceBudget,
    /// Original user request fingerprint used for idempotency.
    pub request_fingerprint: [u8; 32],
}

impl TaskRunSpawnFenceBindingV1 {
    /// Hash this exact typed binding with the C4 domain separator used by REST and Runtime.
    pub fn binding_digest(&self) -> Result<[u8; 32], serde_json::Error> {
        let encoded = serde_json::to_vec(self)?;
        let mut hasher = Sha256::new();
        hasher.update(b"star.task_run_spawn_fence.v1\0");
        hasher.update(encoded);
        Ok(hasher.finalize().into())
    }

    /// Reject incomplete identities, noncanonical digests, and empty resource ceilings.
    pub fn is_valid(&self) -> bool {
        ![
            self.tenant_id,
            self.actor_id,
            self.project_id,
            self.repository_id,
            self.worktree_id,
            self.work_item_id,
            self.runtime_id,
            self.approved_launch_profile.profile_id,
            self.execution_profile.profile_id,
            self.catalog_revisions.grant_set_id,
            self.hook_set.hook_set_id,
        ]
        .contains(&Uuid::nil())
            && self.expected_lifecycle_version > 0
            && self.approved_launch_profile.version > 0
            && self.execution_profile.version > 0
            && self.catalog_revisions.grant_set_version > 0
            && self.hook_set.version > 0
            && is_lower_hex_sha256_digest(&self.approved_launch_profile.content_digest)
            && is_lower_hex_sha256_digest(&self.execution_profile.content_digest)
            && is_lower_hex_sha256_digest(&self.hook_set.effective_digest)
            && self.resource_budget.max_rss_bytes > 0
            && self.resource_budget.max_cpu_ms > 0
            && self.resource_budget.max_runtime_ms > 0
            && self.resource_budget.max_child_processes > 0
            && self.resource_budget.max_parallel_tools > 0
            && self.resource_budget.max_provider_calls > 0
            && self.resource_budget.max_output_bytes > 0
            && self.resource_budget.max_event_buffer_bytes > 0
    }
}

/// Opaque Runtime-resident single-use handle bound to the complete typed admission snapshot.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRunSpawnFence {
    /// Opaque Runtime-local identity; never persist this in the Run record or BI projection.
    pub fence_id: Uuid,
    /// Time at which this Runtime fence was issued.
    pub issued_at: DateTime<Utc>,
    /// Fence expiration time.
    pub expires_at: DateTime<Utc>,
    /// Exact bounded decision binding.
    pub binding: TaskRunSpawnFenceBindingV1,
    /// Digest of `binding` under the versioned C4 domain separator.
    pub binding_digest: [u8; 32],
}

impl TaskRunSpawnFence {
    /// Validate all fixed-shape fields and the digest before serialization or hashing.
    /// Digest strings are checked first so untrusted oversized strings never enter a hash buffer.
    pub fn is_well_formed(&self) -> bool {
        !self.fence_id.is_nil()
            && self.expires_at > self.issued_at
            && self.expires_at.signed_duration_since(self.issued_at) <= Duration::seconds(30)
            && self.binding.is_valid()
            && self
                .binding
                .binding_digest()
                .is_ok_and(|digest| digest == self.binding_digest)
    }

    /// Check the bounded validity window and digest before this fence can reach atomic consume.
    pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
        self.is_well_formed()
            && self.issued_at <= now
            && self.expires_at > now
            && now.signed_duration_since(self.issued_at) <= Duration::seconds(30)
    }
}

fn is_lower_hex_sha256_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
