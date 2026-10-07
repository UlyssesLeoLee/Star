//! @cypher schema=1 source_sha256=796c88bd4b09c95a2dff20634f1c6adfb7f2cee1ff6a0aae55abc0198b90295b
//! MERGE (self:File {path:"crates/domain-infrastructure/src/capacity.rs"})
//! MERGE (module:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::module",kind:"module"})
//! MERGE (vector:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureResourceVectorV1",kind:"struct"})
//! MERGE (snapshot:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureCapacitySnapshotV2",kind:"struct"})
//! MERGE (request:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureCapacityRequestV2",kind:"struct"})
//! MERGE (candidate:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureCapacityCandidateV2",kind:"struct"})
//! MERGE (error:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureCapacityEvaluationError",kind:"enum"})
//! MERGE (capacity_add:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureResourceVectorV1.checked_add",kind:"method"})
//! MERGE (capacity_sub:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureResourceVectorV1.checked_sub",kind:"method"})
//! MERGE (evaluate:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::evaluate_infrastructure_capacity_candidate",kind:"function"})
//! MERGE (valid_request:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureCapacityRequestV2.validate",kind:"method"})
//! MERGE (binding_digest:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2.digest",kind:"method"})
//! MERGE (placement:Type {id:"crates/domain-infrastructure/src/lib.rs::ClusterPlacementPolicyV1",kind:"enum"})
//! MERGE (provider:Type {id:"crates/domain-infrastructure/src/lib.rs::ProviderKind",kind:"enum"})
//! MERGE (profile:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureProviderProfileRevisionV1",kind:"struct"})
//! MERGE (profile_error:Type {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureProviderProfileError",kind:"enum"})
//! MERGE (profile_validate:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureProviderProfileRevisionV1.validate",kind:"method"})
//! MERGE (profile_digest:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureProviderProfileRevisionV1.digest",kind:"method"})
//! MERGE (profile_capacity_request:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::InfrastructureProviderProfileRevisionV1.capacity_request_for_binding",kind:"method"})
//! MERGE (fixture:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::capacity_fixture",kind:"function"})
//! MERGE (matching_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::fresh_capacity_returns_remaining_headroom",kind:"test"})
//! MERGE (unknown_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::unknown_measurement_fails_closed",kind:"test"})
//! MERGE (stale_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::stale_or_future_snapshot_fails_closed",kind:"test"})
//! MERGE (scope_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::snapshot_must_match_bound_provider_host_and_cluster",kind:"test"})
//! MERGE (oversubscribed_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::oversubscribed_snapshot_fails_closed",kind:"test"})
//! MERGE (insufficient_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::each_resource_dimension_is_enforced",kind:"test"})
//! MERGE (malformed_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::malformed_or_unsupported_inputs_fail_closed",kind:"test"})
//! MERGE (deadline_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::expired_demand_fails_closed",kind:"test"})
//! MERGE (request_binding_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::request_must_match_binding_and_memory_budget",kind:"test"})
//! MERGE (arithmetic_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::unrepresentable_capacity_arithmetic_fails_closed",kind:"test"})
//! MERGE (profile_digest_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::provider_profile_digest_binds_resources_and_derives_the_request",kind:"test"})
//! MERGE (profile_binding_test:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::provider_profile_must_match_pinned_tenant_identity_and_placement",kind:"test"})
//! MERGE (test_id:Symbol {id:"crates/domain-infrastructure/src/capacity.rs::tests::id",kind:"function"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (self)-[:IMPORTS]->(placement)
//! MERGE (self)-[:IMPORTS]->(provider)
//! MERGE (module)-[:DEFINES]->(vector)
//! MERGE (module)-[:DEFINES]->(snapshot)
//! MERGE (module)-[:DEFINES]->(request)
//! MERGE (module)-[:DEFINES]->(candidate)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (module)-[:DEFINES]->(evaluate)
//! MERGE (module)-[:DEFINES]->(valid_request)
//! MERGE (module)-[:DEFINES]->(profile)
//! MERGE (module)-[:DEFINES]->(profile_error)
//! MERGE (profile)-[:DEFINES]->(profile_validate)
//! MERGE (profile)-[:DEFINES]->(profile_digest)
//! MERGE (profile)-[:DEFINES]->(profile_capacity_request)
//! MERGE (profile)-[:USES_TYPE]->(vector)
//! MERGE (profile)-[:USES_TYPE]->(placement)
//! MERGE (profile)-[:USES_TYPE]->(provider)
//! MERGE (profile_validate)-[:USES_TYPE]->(profile_error)
//! MERGE (profile_digest)-[:CALLS]->(profile_validate)
//! MERGE (profile_capacity_request)-[:CALLS]->(profile_validate)
//! MERGE (profile_capacity_request)-[:CALLS]->(profile_digest)
//! MERGE (profile_capacity_request)-[:CALLS]->(binding_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2.validate",kind:"method"})
//! MERGE (profile_capacity_request)-[:CALLS]->(binding_digest)
//! MERGE (profile_capacity_request)-[:USES_TYPE]->(request)
//! MERGE (profile_capacity_request)-[:USES_TYPE]->(profile_error)
//! MERGE (module)-[:DEFINES]->(fixture)
//! MERGE (module)-[:DEFINES]->(test_id)
//! MERGE (vector)-[:DEFINES]->(capacity_add)
//! MERGE (vector)-[:DEFINES]->(capacity_sub)
//! MERGE (evaluate)-[:CALLS]->(valid_request)
//! MERGE (evaluate)-[:CALLS]->(binding_digest)
//! MERGE (evaluate)-[:CALLS]->(capacity_add)
//! MERGE (evaluate)-[:CALLS]->(capacity_sub)
//! MERGE (evaluate)-[:CALLS]->(demand_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::HostInfrastructureDemandV2::validate",kind:"method"})
//! MERGE (evaluate)-[:USES_TYPE]->(snapshot)
//! MERGE (evaluate)-[:USES_TYPE]->(request)
//! MERGE (evaluate)-[:USES_TYPE]->(candidate)
//! MERGE (evaluate)-[:USES_TYPE]->(error)
//! MERGE (snapshot)-[:USES_TYPE]->(vector)
//! MERGE (request)-[:USES_TYPE]->(vector)
//! MERGE (candidate)-[:USES_TYPE]->(vector)
//! MERGE (matching_test)-[:CALLS]->(fixture)
//! MERGE (matching_test)-[:CALLS]->(evaluate)
//! MERGE (unknown_test)-[:CALLS]->(fixture)
//! MERGE (unknown_test)-[:CALLS]->(evaluate)
//! MERGE (stale_test)-[:CALLS]->(fixture)
//! MERGE (stale_test)-[:CALLS]->(evaluate)
//! MERGE (scope_test)-[:CALLS]->(fixture)
//! MERGE (scope_test)-[:CALLS]->(evaluate)
//! MERGE (oversubscribed_test)-[:CALLS]->(fixture)
//! MERGE (oversubscribed_test)-[:CALLS]->(evaluate)
//! MERGE (insufficient_test)-[:CALLS]->(fixture)
//! MERGE (insufficient_test)-[:CALLS]->(evaluate)
//! MERGE (malformed_test)-[:CALLS]->(fixture)
//! MERGE (malformed_test)-[:CALLS]->(evaluate)
//! MERGE (deadline_test)-[:CALLS]->(fixture)
//! MERGE (deadline_test)-[:CALLS]->(evaluate)
//! MERGE (request_binding_test)-[:CALLS]->(fixture)
//! MERGE (request_binding_test)-[:CALLS]->(evaluate)
//! MERGE (arithmetic_test)-[:CALLS]->(fixture)
//! MERGE (arithmetic_test)-[:CALLS]->(evaluate)
//! MERGE (profile_digest_test)-[:CALLS]->(fixture)
//! MERGE (profile_digest_test)-[:CALLS]->(profile_digest)
//! MERGE (profile_digest_test)-[:CALLS]->(profile_capacity_request)
//! MERGE (profile_binding_test)-[:CALLS]->(fixture)
//! MERGE (profile_binding_test)-[:CALLS]->(profile_capacity_request)
//! MERGE (fixture)-[:CALLS]->(test_id)
//! MERGE (evaluate)-[:CALLS]->(valid_request)
//! @endcypher

//! Pure, fail-closed assessment of physical Host and exact K3s cluster headroom.
//!
//! The candidate is not a reservation or authorization. The Infrastructure owner must call
//! this while serializing admission for the Host and selected cluster, persist the reservation
//! bundle and event atomically,
//! and separately validate current Project/Run authorization and provider readiness.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    ClusterPlacementPolicyV1, HostInfrastructureDemandV2, ProviderKind,
    RunInfrastructureBindingSnapshotV2,
};

const INFRASTRUCTURE_PROVIDER_PROFILE_SCHEMA_VERSION_V1: u16 = 1;
const INFRASTRUCTURE_CAPACITY_SNAPSHOT_SCHEMA_VERSION_V2: u16 = 2;
const INFRASTRUCTURE_CAPACITY_REQUEST_SCHEMA_VERSION_V2: u16 = 2;
const INFRASTRUCTURE_CAPACITY_CANDIDATE_SCHEMA_VERSION_V2: u16 = 2;
const MAX_INFRASTRUCTURE_CAPACITY_SNAPSHOT_AGE_SECONDS: i64 = 30;

/// Resource quantities attributed to a Host, expressed in stable wire units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureResourceVectorV1 {
    /// CPU capacity in millicores.
    pub cpu_millicores: u64,
    /// Resident memory capacity in bytes.
    pub memory_bytes: u64,
    /// Ephemeral storage capacity in bytes.
    pub ephemeral_storage_bytes: u64,
    /// GPU units exposed by the provider profile.
    pub gpu_units: u32,
}

impl InfrastructureResourceVectorV1 {
    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            cpu_millicores: self.cpu_millicores.checked_add(other.cpu_millicores)?,
            memory_bytes: self.memory_bytes.checked_add(other.memory_bytes)?,
            ephemeral_storage_bytes: self
                .ephemeral_storage_bytes
                .checked_add(other.ephemeral_storage_bytes)?,
            gpu_units: self.gpu_units.checked_add(other.gpu_units)?,
        })
    }

    fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self {
            cpu_millicores: self.cpu_millicores.checked_sub(other.cpu_millicores)?,
            memory_bytes: self.memory_bytes.checked_sub(other.memory_bytes)?,
            ephemeral_storage_bytes: self
                .ephemeral_storage_bytes
                .checked_sub(other.ephemeral_storage_bytes)?,
            gpu_units: self.gpu_units.checked_sub(other.gpu_units)?,
        })
    }
}

/// Immutable Infrastructure-owner profile revision that bounds a Run's capacity request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureProviderProfileRevisionV1 {
    /// Explicit profile schema version.
    pub schema_version: u16,
    /// Tenant that owns the profile revision.
    pub tenant_id: Uuid,
    /// Stable provider-profile identity.
    pub profile_id: Uuid,
    /// Immutable profile revision.
    pub revision: i64,
    /// Provider to which this profile applies.
    pub provider: ProviderKind,
    /// Placement policy allowed by this profile.
    pub placement_policy: ClusterPlacementPolicyV1,
    /// Maximum capacity charged against the physical Host for one admitted Run.
    pub host_resources: InfrastructureResourceVectorV1,
    /// Maximum capacity charged against the exact K3s cluster for one admitted Run.
    pub cluster_resources: InfrastructureResourceVectorV1,
}

impl InfrastructureProviderProfileRevisionV1 {
    /// Reject unsupported revisions, invalid identity, or capacity outside persisted ranges.
    pub fn validate(&self) -> Result<(), InfrastructureProviderProfileError> {
        if self.schema_version != INFRASTRUCTURE_PROVIDER_PROFILE_SCHEMA_VERSION_V1 {
            return Err(InfrastructureProviderProfileError::UnsupportedSchemaVersion);
        }
        if self.tenant_id.is_nil() || self.profile_id.is_nil() || self.revision <= 0 {
            return Err(InfrastructureProviderProfileError::InvalidIdentity);
        }
        for resources in [self.host_resources, self.cluster_resources] {
            if resources.cpu_millicores == 0
                || resources.memory_bytes == 0
                || resources.cpu_millicores > i64::MAX as u64
                || resources.memory_bytes > i64::MAX as u64
                || resources.ephemeral_storage_bytes > i64::MAX as u64
                || resources.gpu_units > i32::MAX as u32
            {
                return Err(InfrastructureProviderProfileError::InvalidResources);
            }
        }
        Ok(())
    }

    /// Compute a stable digest over every profile field that affects placement admission.
    pub fn digest(&self) -> Result<[u8; 32], InfrastructureProviderProfileError> {
        self.validate()?;
        let mut hasher = Sha256::new();
        hasher.update(b"star.infrastructure-provider-profile.v1\0");
        hasher.update(self.schema_version.to_be_bytes());
        hasher.update(self.tenant_id.as_bytes());
        hasher.update(self.profile_id.as_bytes());
        hasher.update(self.revision.to_be_bytes());
        hasher.update(self.provider.wire_value().as_bytes());
        hasher.update([0]);
        hasher.update(self.placement_policy.wire_value().as_bytes());
        for resources in [self.host_resources, self.cluster_resources] {
            hasher.update(resources.cpu_millicores.to_be_bytes());
            hasher.update(resources.memory_bytes.to_be_bytes());
            hasher.update(resources.ephemeral_storage_bytes.to_be_bytes());
            hasher.update(resources.gpu_units.to_be_bytes());
        }
        Ok(hasher.finalize().into())
    }

    /// Derive the only admissible capacity request for a binding pinned to this revision.
    pub fn capacity_request_for_binding(
        &self,
        binding: &RunInfrastructureBindingSnapshotV2,
    ) -> Result<InfrastructureCapacityRequestV2, InfrastructureProviderProfileError> {
        self.validate()?;
        binding
            .validate()
            .map_err(|_| InfrastructureProviderProfileError::BindingMismatch)?;
        if self.tenant_id != binding.tenant_id
            || self.profile_id != binding.provider_profile_id
            || self.revision != binding.provider_profile_revision
            || self.provider != binding.provider
            || self.placement_policy != binding.cluster_placement_policy
        {
            return Err(InfrastructureProviderProfileError::BindingMismatch);
        }
        if self.digest()? != binding.provider_profile_digest {
            return Err(InfrastructureProviderProfileError::DigestMismatch);
        }
        let binding_digest = binding
            .digest()
            .map_err(|_| InfrastructureProviderProfileError::BindingMismatch)?;
        Ok(InfrastructureCapacityRequestV2 {
            schema_version: INFRASTRUCTURE_CAPACITY_REQUEST_SCHEMA_VERSION_V2,
            binding_digest,
            host_resources: self.host_resources,
            cluster_resources: self.cluster_resources,
        })
    }
}

/// Fail-closed validation errors for a persisted provider-profile revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InfrastructureProviderProfileError {
    /// The row uses a schema version this binary does not understand.
    #[error("unsupported Infrastructure provider-profile schema version")]
    UnsupportedSchemaVersion,
    /// The tenant, profile identity, or revision is invalid.
    #[error("invalid Infrastructure provider-profile identity")]
    InvalidIdentity,
    /// A resource quantity is zero where required or cannot be persisted safely.
    #[error("invalid Infrastructure provider-profile resource vector")]
    InvalidResources,
    /// The row does not match the Run binding's pinned profile identity or policy.
    #[error("Infrastructure provider-profile does not match the Run binding")]
    BindingMismatch,
    /// The stored binding digest does not match the immutable profile contents.
    #[error("Infrastructure provider-profile digest does not match the Run binding")]
    DigestMismatch,
}

/// Fresh Host and exact-cluster observations used only to assess one placement candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureCapacitySnapshotV2 {
    /// Explicit snapshot contract version.
    pub schema_version: u16,
    /// HostEnvironment identity from the current Infrastructure binding.
    pub host_environment_id: Uuid,
    /// Monotonic provider-owned generation for this HostEnvironment.
    pub host_generation: i64,
    /// Exact K3s cluster identity selected by the binding.
    pub k3s_cluster_id: Uuid,
    /// Monotonic generation of the selected K3s cluster.
    pub k3s_cluster_generation: i64,
    /// Provider whose capacity is measured.
    pub provider: ProviderKind,
    /// Time at which the trusted provider measured the snapshot.
    pub observed_at: DateTime<Utc>,
    /// Total capacity exposed to this Infrastructure owner.
    pub total: Option<InfrastructureResourceVectorV1>,
    /// Capacity reserved for the base operating system and platform daemons.
    pub os_base_reserve: Option<InfrastructureResourceVectorV1>,
    /// Additional operator-configured safety reserve.
    pub host_safety_reserve: Option<InfrastructureResourceVectorV1>,
    /// Non-Star or otherwise unmanaged load measured on the provider.
    pub unmanaged_commitments: Option<InfrastructureResourceVectorV1>,
    /// Existing unexpired Star reservations from the Infrastructure owner ledger.
    pub active_reservations: Option<InfrastructureResourceVectorV1>,
    /// Aggregate allocatable resources of Ready nodes in the exact K3s cluster.
    pub cluster_allocatable: Option<InfrastructureResourceVectorV1>,
    /// Operator reserve held back from Kubernetes workload placement.
    pub cluster_safety_reserve: Option<InfrastructureResourceVectorV1>,
    /// Non-Star or otherwise unmanaged workloads in the cluster.
    pub cluster_unmanaged_commitments: Option<InfrastructureResourceVectorV1>,
    /// Existing unexpired Star reservations in the cluster ledger.
    pub cluster_active_reservations: Option<InfrastructureResourceVectorV1>,
}

/// Capacity requested by the currently validated provider/runtime profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureCapacityRequestV2 {
    /// Explicit request contract version.
    pub schema_version: u16,
    /// Digest of the exact Run binding whose provider/runtime profile produced this vector.
    pub binding_digest: [u8; 32],
    /// Physical Host quantities, including provider overhead, bounded by the profile.
    pub host_resources: InfrastructureResourceVectorV1,
    /// Kubernetes request quantities for the selected cluster, bounded by the profile.
    pub cluster_resources: InfrastructureResourceVectorV1,
}

impl InfrastructureCapacityRequestV2 {
    fn validate(&self) -> Result<(), InfrastructureCapacityEvaluationError> {
        if self.schema_version != INFRASTRUCTURE_CAPACITY_REQUEST_SCHEMA_VERSION_V2 {
            return Err(InfrastructureCapacityEvaluationError::UnsupportedSchemaVersion);
        }
        if self.binding_digest.iter().all(|byte| *byte == 0)
            || self.host_resources.cpu_millicores == 0
            || self.host_resources.memory_bytes == 0
            || self.cluster_resources.cpu_millicores == 0
            || self.cluster_resources.memory_bytes == 0
        {
            return Err(InfrastructureCapacityEvaluationError::InvalidRequest);
        }
        Ok(())
    }
}

/// Headroom calculation for a candidate; it does not reserve resources or authorize execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureCapacityCandidateV2 {
    /// Explicit candidate contract version.
    pub schema_version: u16,
    /// Run-owner demand and Schedule occurrence this candidate was evaluated for.
    pub demand_event_id: Uuid,
    /// Stable idempotency identity of the Schedule occurrence.
    pub occurrence_id: Uuid,
    /// Immutable Run binding identity and revision used by the capacity profile.
    pub binding_id: Uuid,
    /// Immutable Run binding revision used by the capacity profile.
    pub binding_revision: i64,
    /// Digest of the exact Run binding and profile inputs used for the request.
    pub binding_digest: [u8; 32],
    /// HostEnvironment and generation to which this result is bound.
    pub host_environment_id: Uuid,
    /// Host generation observed by the provider.
    pub host_generation: i64,
    /// Exact K3s cluster and generation to which this result is bound.
    pub k3s_cluster_id: Uuid,
    /// Generation of the exact K3s cluster.
    pub k3s_cluster_generation: i64,
    /// Snapshot timestamp used for this calculation.
    pub observed_at: DateTime<Utc>,
    /// Available physical Host capacity immediately before this candidate.
    pub host_available_before: InfrastructureResourceVectorV1,
    /// Remaining physical Host capacity if the candidate were reserved.
    pub host_available_after: InfrastructureResourceVectorV1,
    /// Available cluster allocatable capacity immediately before this candidate.
    pub cluster_available_before: InfrastructureResourceVectorV1,
    /// Remaining cluster capacity if the candidate were reserved.
    pub cluster_available_after: InfrastructureResourceVectorV1,
}

/// Assess whether both Host and exact-cluster headroom fit the bound Run.
///
/// Callers must serialize this calculation with reservation persistence per HostEnvironment;
/// two concurrent candidates cannot both rely on this pure result without an owner-side lock.
pub fn evaluate_infrastructure_capacity_candidate(
    demand: &HostInfrastructureDemandV2,
    snapshot: &InfrastructureCapacitySnapshotV2,
    request: &InfrastructureCapacityRequestV2,
    now: DateTime<Utc>,
) -> Result<InfrastructureCapacityCandidateV2, InfrastructureCapacityEvaluationError> {
    demand
        .validate()
        .map_err(|_| InfrastructureCapacityEvaluationError::InvalidDemand)?;
    request.validate()?;
    let binding_digest = demand
        .binding
        .digest()
        .map_err(|_| InfrastructureCapacityEvaluationError::InvalidDemand)?;
    if request.binding_digest != binding_digest {
        return Err(InfrastructureCapacityEvaluationError::RequestBindingMismatch);
    }
    let maximum_run_memory = u64::try_from(demand.resource_budget.max_rss_bytes)
        .map_err(|_| InfrastructureCapacityEvaluationError::InvalidDemand)?;
    if request.host_resources.memory_bytes < maximum_run_memory
        || request.cluster_resources.memory_bytes < maximum_run_memory
    {
        return Err(InfrastructureCapacityEvaluationError::RequestBelowRunMemoryBudget);
    }
    if now >= demand.deadline_at {
        return Err(InfrastructureCapacityEvaluationError::AdmissionDeadlineExpired);
    }
    if snapshot.schema_version != INFRASTRUCTURE_CAPACITY_SNAPSHOT_SCHEMA_VERSION_V2 {
        return Err(InfrastructureCapacityEvaluationError::UnsupportedSchemaVersion);
    }
    if snapshot.host_environment_id.is_nil()
        || snapshot.host_generation <= 0
        || snapshot.k3s_cluster_id.is_nil()
        || snapshot.k3s_cluster_generation <= 0
    {
        return Err(InfrastructureCapacityEvaluationError::InvalidSnapshot);
    }
    if snapshot.host_environment_id != demand.binding.host_environment_id
        || snapshot.k3s_cluster_id != demand.binding.k3s_cluster_id
        || snapshot.k3s_cluster_generation != demand.binding.k3s_cluster_generation
        || snapshot.provider != demand.binding.provider
    {
        return Err(InfrastructureCapacityEvaluationError::ScopeMismatch);
    }
    let age = now.signed_duration_since(snapshot.observed_at);
    if age < Duration::zero()
        || age > Duration::seconds(MAX_INFRASTRUCTURE_CAPACITY_SNAPSHOT_AGE_SECONDS)
    {
        return Err(InfrastructureCapacityEvaluationError::SnapshotStale);
    }

    let total = snapshot
        .total
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let os_base_reserve = snapshot
        .os_base_reserve
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let host_safety_reserve = snapshot
        .host_safety_reserve
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let unmanaged_commitments = snapshot
        .unmanaged_commitments
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let active_reservations = snapshot
        .active_reservations
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    if total.cpu_millicores == 0 || total.memory_bytes == 0 {
        return Err(InfrastructureCapacityEvaluationError::InvalidSnapshot);
    }

    let protected_capacity = os_base_reserve
        .checked_add(host_safety_reserve)
        .ok_or(InfrastructureCapacityEvaluationError::InvalidSnapshot)?;
    let capacity_after_reserves = total
        .checked_sub(protected_capacity)
        .ok_or(InfrastructureCapacityEvaluationError::InvalidSnapshot)?;
    let committed = unmanaged_commitments
        .checked_add(active_reservations)
        .ok_or(InfrastructureCapacityEvaluationError::InvalidSnapshot)?;
    let available_before = capacity_after_reserves
        .checked_sub(committed)
        .ok_or(InfrastructureCapacityEvaluationError::HostCapacityExhausted)?;
    let host_available_after = available_before
        .checked_sub(request.host_resources)
        .ok_or(InfrastructureCapacityEvaluationError::HostCapacityInsufficient)?;

    let cluster_allocatable = snapshot
        .cluster_allocatable
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let cluster_safety_reserve = snapshot
        .cluster_safety_reserve
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let cluster_unmanaged_commitments = snapshot
        .cluster_unmanaged_commitments
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    let cluster_active_reservations = snapshot
        .cluster_active_reservations
        .ok_or(InfrastructureCapacityEvaluationError::CapacityUnknown)?;
    if cluster_allocatable.cpu_millicores == 0 || cluster_allocatable.memory_bytes == 0 {
        return Err(InfrastructureCapacityEvaluationError::InvalidSnapshot);
    }
    let cluster_capacity_after_reserves =
        cluster_allocatable
            .checked_sub(cluster_safety_reserve)
            .ok_or(InfrastructureCapacityEvaluationError::InvalidSnapshot)?;
    let cluster_committed = cluster_unmanaged_commitments
        .checked_add(cluster_active_reservations)
        .ok_or(InfrastructureCapacityEvaluationError::InvalidSnapshot)?;
    let cluster_available_before = cluster_capacity_after_reserves
        .checked_sub(cluster_committed)
        .ok_or(InfrastructureCapacityEvaluationError::ClusterCapacityExhausted)?;
    let cluster_available_after = cluster_available_before
        .checked_sub(request.cluster_resources)
        .ok_or(InfrastructureCapacityEvaluationError::ClusterCapacityInsufficient)?;

    Ok(InfrastructureCapacityCandidateV2 {
        schema_version: INFRASTRUCTURE_CAPACITY_CANDIDATE_SCHEMA_VERSION_V2,
        demand_event_id: demand.event_id,
        occurrence_id: demand.occurrence_id,
        binding_id: demand.binding.binding_id,
        binding_revision: demand.binding.binding_revision,
        binding_digest,
        host_environment_id: snapshot.host_environment_id,
        host_generation: snapshot.host_generation,
        k3s_cluster_id: snapshot.k3s_cluster_id,
        k3s_cluster_generation: snapshot.k3s_cluster_generation,
        observed_at: snapshot.observed_at,
        host_available_before: available_before,
        host_available_after,
        cluster_available_before,
        cluster_available_after,
    })
}

/// Reject capacity inputs that cannot establish a safe candidate assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InfrastructureCapacityEvaluationError {
    /// The demand or current binding violates the shared typed contract.
    #[error("invalid Host infrastructure demand")]
    InvalidDemand,
    /// The capacity snapshot or request uses an unsupported version.
    #[error("unsupported Host capacity schema version")]
    UnsupportedSchemaVersion,
    /// The host measurement is malformed or arithmetic cannot be represented.
    #[error("invalid Host capacity snapshot")]
    InvalidSnapshot,
    /// One or more required capacity dimensions are unknown.
    #[error("Host capacity is unknown")]
    CapacityUnknown,
    /// Snapshot belongs to another HostEnvironment or provider.
    #[error("Host capacity scope does not match the binding")]
    ScopeMismatch,
    /// The capacity measurement is future-dated or older than 30 seconds.
    #[error("Host capacity snapshot is stale")]
    SnapshotStale,
    /// The demand's admission deadline has elapsed.
    #[error("Host capacity admission deadline expired")]
    AdmissionDeadlineExpired,
    /// Current commitments already exceed capacity after reserves.
    #[error("Host capacity is exhausted")]
    HostCapacityExhausted,
    /// The candidate request exceeds available capacity in at least one dimension.
    #[error("Host capacity is insufficient")]
    HostCapacityInsufficient,
    /// Existing cluster commitments exceed current Kubernetes allocatable resources.
    #[error("K3s cluster capacity is exhausted")]
    ClusterCapacityExhausted,
    /// The candidate exceeds an allocatable cluster dimension.
    #[error("K3s cluster capacity is insufficient")]
    ClusterCapacityInsufficient,
    /// A request with no CPU or memory budget cannot be admitted.
    #[error("invalid Host capacity request")]
    InvalidRequest,
    /// The request was generated from another or stale Run infrastructure binding.
    #[error("Host capacity request binding does not match the demand")]
    RequestBindingMismatch,
    /// The Host and selected-cluster requests must contain the pinned maximum Run RSS.
    #[error("Host or cluster request is below the pinned Run memory budget")]
    RequestBelowRunMemoryBudget,
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_infrastructure_capacity_candidate, InfrastructureCapacityEvaluationError,
        InfrastructureCapacityRequestV2, InfrastructureCapacitySnapshotV2,
        InfrastructureProviderProfileRevisionV1, InfrastructureResourceVectorV1,
    };
    use crate::{
        ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
        KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV2,
    };
    use chrono::{Duration, TimeZone, Utc};
    use uuid::Uuid;

    fn id(number: u128) -> Uuid {
        Uuid::from_u128(number)
    }

    fn capacity_fixture() -> (
        HostInfrastructureDemandV2,
        InfrastructureCapacitySnapshotV2,
        InfrastructureCapacityRequestV2,
        chrono::DateTime<Utc>,
    ) {
        let now = Utc.timestamp_opt(1_791_300_000, 0).unwrap();
        let budget = ExecutionResourceBudgetV1 {
            max_rss_bytes: 512 * 1024 * 1024,
            max_cpu_ms: 60_000,
            max_runtime_ms: 600_000,
            max_child_processes: 8,
            max_parallel_tools: 4,
            max_provider_calls: 100,
            max_output_bytes: 1_048_576,
            max_event_buffer_bytes: 262_144,
        };
        let vector = |cpu, memory, storage, gpu| InfrastructureResourceVectorV1 {
            cpu_millicores: cpu,
            memory_bytes: memory,
            ephemeral_storage_bytes: storage,
            gpu_units: gpu,
        };
        let profile = InfrastructureProviderProfileRevisionV1 {
            schema_version: 1,
            tenant_id: id(1),
            profile_id: id(7),
            revision: 3,
            provider: ProviderKind::MultipassK3sCluster,
            placement_policy: ClusterPlacementPolicyV1::SharedPool,
            host_resources: vector(1_000, 1024_u64.pow(3), 10 * 1024_u64.pow(3), 0),
            cluster_resources: vector(1_000, 1024_u64.pow(3), 10 * 1024_u64.pow(3), 0),
        };
        let binding = RunInfrastructureBindingSnapshotV2 {
            schema_version: 2,
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            binding_id: id(5),
            binding_revision: 1,
            host_environment_id: id(6),
            k3s_cluster_id: id(14),
            k3s_cluster_generation: 3,
            cluster_placement_policy: ClusterPlacementPolicyV1::SharedPool,
            provider: ProviderKind::MultipassK3sCluster,
            provider_profile_id: id(7),
            provider_profile_revision: 3,
            provider_profile_digest: profile.digest().unwrap(),
            namespace: KubernetesDnsLabel::new("run-04").unwrap(),
            service_account: KubernetesDnsLabel::new("agent-runtime").unwrap(),
            runtime_profile_id: id(8),
            runtime_profile_revision: 2,
            runtime_profile_digest: [0x22; 32],
            resource_budget: budget,
        };
        let demand = HostInfrastructureDemandV2 {
            schema_version: 2,
            event_id: id(9),
            correlation_id: id(10),
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            task_execution_run_id: id(11),
            schedule_rule_id: id(12),
            schedule_rule_version: 4,
            occurrence_id: id(13),
            binding,
            resource_budget: budget,
            requested_at: now - Duration::seconds(60),
            deadline_at: now + Duration::minutes(10),
        };
        let snapshot = InfrastructureCapacitySnapshotV2 {
            schema_version: 2,
            host_environment_id: id(6),
            host_generation: 2,
            k3s_cluster_id: id(14),
            k3s_cluster_generation: 3,
            provider: ProviderKind::MultipassK3sCluster,
            observed_at: now - Duration::seconds(2),
            total: Some(vector(
                8_000,
                16 * 1024_u64.pow(3),
                100 * 1024_u64.pow(3),
                2,
            )),
            os_base_reserve: Some(vector(500, 2 * 1024_u64.pow(3), 10 * 1024_u64.pow(3), 0)),
            host_safety_reserve: Some(vector(500, 2 * 1024_u64.pow(3), 10 * 1024_u64.pow(3), 0)),
            unmanaged_commitments: Some(vector(500, 1024_u64.pow(3), 2 * 1024_u64.pow(3), 0)),
            active_reservations: Some(vector(2_000, 4 * 1024_u64.pow(3), 20 * 1024_u64.pow(3), 1)),
            cluster_allocatable: Some(vector(6_000, 12 * 1024_u64.pow(3), 80 * 1024_u64.pow(3), 2)),
            cluster_safety_reserve: Some(vector(500, 2 * 1024_u64.pow(3), 4 * 1024_u64.pow(3), 0)),
            cluster_unmanaged_commitments: Some(vector(
                500,
                1024_u64.pow(3),
                4 * 1024_u64.pow(3),
                0,
            )),
            cluster_active_reservations: Some(vector(
                1_000,
                2 * 1024_u64.pow(3),
                10 * 1024_u64.pow(3),
                1,
            )),
        };
        let request = profile
            .capacity_request_for_binding(&demand.binding)
            .unwrap();
        (demand, snapshot, request, now)
    }

    #[test]
    fn fresh_capacity_returns_remaining_headroom() {
        let (demand, snapshot, request, now) = capacity_fixture();
        let candidate =
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now).unwrap();
        assert_eq!(
            candidate.host_environment_id,
            demand.binding.host_environment_id
        );
        assert_eq!(candidate.host_generation, 2);
        assert_eq!(candidate.k3s_cluster_id, demand.binding.k3s_cluster_id);
        assert_eq!(candidate.k3s_cluster_generation, 3);
        assert_eq!(candidate.demand_event_id, demand.event_id);
        assert_eq!(candidate.occurrence_id, demand.occurrence_id);
        assert_eq!(candidate.binding_id, demand.binding.binding_id);
        assert_eq!(candidate.binding_revision, demand.binding.binding_revision);
        assert_eq!(candidate.binding_digest, demand.binding.digest().unwrap());
        assert_eq!(candidate.host_available_before.cpu_millicores, 4_500);
        assert_eq!(candidate.host_available_after.cpu_millicores, 3_500);
        assert_eq!(
            candidate.host_available_after.memory_bytes,
            6 * 1024_u64.pow(3)
        );
        assert_eq!(
            candidate.host_available_after.ephemeral_storage_bytes,
            48 * 1024_u64.pow(3)
        );
        assert_eq!(candidate.host_available_after.gpu_units, 1);
        assert_eq!(candidate.cluster_available_before.cpu_millicores, 4_000);
        assert_eq!(candidate.cluster_available_after.cpu_millicores, 3_000);
        assert_eq!(
            candidate.cluster_available_after.memory_bytes,
            6 * 1024_u64.pow(3)
        );
        assert_eq!(candidate.cluster_available_after.gpu_units, 1);
    }

    #[test]
    fn unknown_measurement_fails_closed() {
        let (demand, mut snapshot, request, now) = capacity_fixture();
        snapshot.unmanaged_commitments = None;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::CapacityUnknown)
        );
        snapshot.unmanaged_commitments = Some(InfrastructureResourceVectorV1 {
            cpu_millicores: 500,
            memory_bytes: 1024_u64.pow(3),
            ephemeral_storage_bytes: 2 * 1024_u64.pow(3),
            gpu_units: 0,
        });
        snapshot.cluster_allocatable = None;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::CapacityUnknown)
        );
    }

    #[test]
    fn stale_or_future_snapshot_fails_closed() {
        let (demand, mut snapshot, request, now) = capacity_fixture();
        snapshot.observed_at = now - Duration::seconds(31);
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::SnapshotStale)
        );
        snapshot.observed_at = now + Duration::seconds(1);
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::SnapshotStale)
        );
    }

    #[test]
    fn snapshot_must_match_bound_provider_host_and_cluster() {
        let (demand, mut snapshot, request, now) = capacity_fixture();
        snapshot.host_environment_id = id(99);
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::ScopeMismatch)
        );
        snapshot.host_environment_id = demand.binding.host_environment_id;
        snapshot.k3s_cluster_id = id(99);
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::ScopeMismatch)
        );
        snapshot.k3s_cluster_id = demand.binding.k3s_cluster_id;
        snapshot.k3s_cluster_generation += 1;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::ScopeMismatch)
        );
        snapshot.k3s_cluster_generation = demand.binding.k3s_cluster_generation;
        snapshot.provider = ProviderKind::RemoteK3s;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::ScopeMismatch)
        );
    }

    #[test]
    fn oversubscribed_snapshot_fails_closed() {
        let (demand, mut snapshot, request, now) = capacity_fixture();
        snapshot.active_reservations = Some(InfrastructureResourceVectorV1 {
            cpu_millicores: 9_000,
            memory_bytes: 0,
            ephemeral_storage_bytes: 0,
            gpu_units: 0,
        });
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::HostCapacityExhausted)
        );
    }

    #[test]
    fn each_resource_dimension_is_enforced() {
        let (demand, snapshot, mut request, now) = capacity_fixture();
        for resources in [
            InfrastructureResourceVectorV1 {
                cpu_millicores: 4_501,
                memory_bytes: 512 * 1024 * 1024,
                ephemeral_storage_bytes: 0,
                gpu_units: 0,
            },
            InfrastructureResourceVectorV1 {
                cpu_millicores: 1,
                memory_bytes: 7 * 1024_u64.pow(3) + 1,
                ephemeral_storage_bytes: 0,
                gpu_units: 0,
            },
            InfrastructureResourceVectorV1 {
                cpu_millicores: 1,
                memory_bytes: 512 * 1024 * 1024,
                ephemeral_storage_bytes: 58 * 1024_u64.pow(3) + 1,
                gpu_units: 0,
            },
            InfrastructureResourceVectorV1 {
                cpu_millicores: 1,
                memory_bytes: 512 * 1024 * 1024,
                ephemeral_storage_bytes: 0,
                gpu_units: 2,
            },
        ] {
            request.host_resources = resources;
            assert_eq!(
                evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
                Err(InfrastructureCapacityEvaluationError::HostCapacityInsufficient)
            );
        }
        request.host_resources = InfrastructureResourceVectorV1 {
            cpu_millicores: 1_000,
            memory_bytes: 1024_u64.pow(3),
            ephemeral_storage_bytes: 10 * 1024_u64.pow(3),
            gpu_units: 0,
        };
        for resources in [
            InfrastructureResourceVectorV1 {
                cpu_millicores: 4_001,
                memory_bytes: 512 * 1024 * 1024,
                ephemeral_storage_bytes: 0,
                gpu_units: 0,
            },
            InfrastructureResourceVectorV1 {
                cpu_millicores: 1,
                memory_bytes: 7 * 1024_u64.pow(3) + 1,
                ephemeral_storage_bytes: 0,
                gpu_units: 0,
            },
            InfrastructureResourceVectorV1 {
                cpu_millicores: 1,
                memory_bytes: 512 * 1024 * 1024,
                ephemeral_storage_bytes: 62 * 1024_u64.pow(3) + 1,
                gpu_units: 0,
            },
            InfrastructureResourceVectorV1 {
                cpu_millicores: 1,
                memory_bytes: 512 * 1024 * 1024,
                ephemeral_storage_bytes: 0,
                gpu_units: 2,
            },
        ] {
            request.cluster_resources = resources;
            assert_eq!(
                evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
                Err(InfrastructureCapacityEvaluationError::ClusterCapacityInsufficient)
            );
        }
    }

    #[test]
    fn malformed_or_unsupported_inputs_fail_closed() {
        let (demand, mut snapshot, mut request, now) = capacity_fixture();
        snapshot.schema_version = 1;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::UnsupportedSchemaVersion)
        );
        snapshot.schema_version = 2;
        snapshot.host_generation = 0;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::InvalidSnapshot)
        );
        snapshot.host_generation = 2;
        request.schema_version = 3;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::UnsupportedSchemaVersion)
        );
        request.schema_version = 2;
        request.host_resources.cpu_millicores = 0;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::InvalidRequest)
        );
    }

    #[test]
    fn expired_demand_fails_closed() {
        let (mut demand, snapshot, request, now) = capacity_fixture();
        demand.deadline_at = now;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::AdmissionDeadlineExpired)
        );
    }

    #[test]
    fn request_must_match_binding_and_memory_budget() {
        let (demand, snapshot, mut request, now) = capacity_fixture();
        request.binding_digest = [0x99; 32];
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::RequestBindingMismatch)
        );
        request.binding_digest = demand.binding.digest().unwrap();
        request.host_resources.memory_bytes = 1;
        request.cluster_resources.memory_bytes = 1;
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::RequestBelowRunMemoryBudget)
        );
    }

    #[test]
    fn provider_profile_digest_binds_resources_and_derives_the_request() {
        let (demand, _, request, _) = capacity_fixture();
        let profile = InfrastructureProviderProfileRevisionV1 {
            schema_version: 1,
            tenant_id: demand.tenant_id,
            profile_id: demand.binding.provider_profile_id,
            revision: demand.binding.provider_profile_revision,
            provider: demand.binding.provider,
            placement_policy: demand.binding.cluster_placement_policy,
            host_resources: request.host_resources,
            cluster_resources: request.cluster_resources,
        };

        assert_eq!(
            profile.digest().unwrap(),
            demand.binding.provider_profile_digest
        );
        assert_eq!(
            profile
                .capacity_request_for_binding(&demand.binding)
                .unwrap(),
            request
        );

        let mut changed = profile;
        changed.host_resources.cpu_millicores += 1;
        assert_eq!(
            changed.capacity_request_for_binding(&demand.binding),
            Err(super::InfrastructureProviderProfileError::DigestMismatch)
        );
    }

    #[test]
    fn provider_profile_must_match_pinned_tenant_identity_and_placement() {
        let (demand, _, request, _) = capacity_fixture();
        let mut profile = InfrastructureProviderProfileRevisionV1 {
            schema_version: 1,
            tenant_id: demand.tenant_id,
            profile_id: demand.binding.provider_profile_id,
            revision: demand.binding.provider_profile_revision,
            provider: demand.binding.provider,
            placement_policy: demand.binding.cluster_placement_policy,
            host_resources: request.host_resources,
            cluster_resources: request.cluster_resources,
        };

        profile.tenant_id = id(99);
        assert_eq!(
            profile.capacity_request_for_binding(&demand.binding),
            Err(super::InfrastructureProviderProfileError::BindingMismatch)
        );
    }

    #[test]
    fn unrepresentable_capacity_arithmetic_fails_closed() {
        let (demand, mut snapshot, request, now) = capacity_fixture();
        snapshot.os_base_reserve = Some(InfrastructureResourceVectorV1 {
            cpu_millicores: 0,
            memory_bytes: u64::MAX,
            ephemeral_storage_bytes: 0,
            gpu_units: 0,
        });
        snapshot.host_safety_reserve = Some(InfrastructureResourceVectorV1 {
            cpu_millicores: 0,
            memory_bytes: 1,
            ephemeral_storage_bytes: 0,
            gpu_units: 0,
        });
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::InvalidSnapshot)
        );

        let (demand, mut snapshot, request, now) = capacity_fixture();
        snapshot.unmanaged_commitments = Some(InfrastructureResourceVectorV1 {
            cpu_millicores: u64::MAX,
            memory_bytes: 0,
            ephemeral_storage_bytes: 0,
            gpu_units: 0,
        });
        snapshot.active_reservations = Some(InfrastructureResourceVectorV1 {
            cpu_millicores: 1,
            memory_bytes: 0,
            ephemeral_storage_bytes: 0,
            gpu_units: 0,
        });
        assert_eq!(
            evaluate_infrastructure_capacity_candidate(&demand, &snapshot, &request, now),
            Err(InfrastructureCapacityEvaluationError::InvalidSnapshot)
        );
    }
}
