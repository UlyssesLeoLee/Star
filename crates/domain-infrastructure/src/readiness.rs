//! @cypher schema=1 source_sha256=c6f5fb76331536224893d6cc9108daa96b2b760b2db9c7e6184b923757b2882d
//! MERGE (self:File {path:"crates/domain-infrastructure/src/readiness.rs"})
//! MERGE (module:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::module",kind:"module"})
//! MERGE (state:Type {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureReadinessStateV1",kind:"enum"})
//! MERGE (acceptance:Type {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureCapacityReservationAcceptanceV2",kind:"struct"})
//! MERGE (evidence:Type {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureReadinessEvidenceV2",kind:"struct"})
//! MERGE (error:Type {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureReadinessError",kind:"enum"})
//! MERGE (schema_version:Variable {id:"crates/domain-infrastructure/src/readiness.rs::INFRASTRUCTURE_READINESS_SCHEMA_VERSION_V2",kind:"constant"})
//! MERGE (gate:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::validate_infrastructure_execution_readiness",kind:"function"})
//! MERGE (acceptance_matches:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureCapacityReservationAcceptanceV2.matches_demand",kind:"method"})
//! MERGE (evidence_matches:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureReadinessEvidenceV2.matches_demand",kind:"method"})
//! MERGE (provider_layers_match:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::InfrastructureReadinessEvidenceV2.provider_layers_match",kind:"method"})
//! MERGE (readiness_age:Variable {id:"crates/domain-infrastructure/src/readiness.rs::MAX_READINESS_AGE_SECONDS",kind:"constant"})
//! MERGE (binding:Type {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2",kind:"struct"})
//! MERGE (demand:Type {id:"crates/domain-infrastructure/src/lib.rs::HostInfrastructureDemandV2",kind:"struct"})
//! MERGE (binding_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2.validate",kind:"method"})
//! MERGE (binding_digest:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV2.digest",kind:"method"})
//! MERGE (demand_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::HostInfrastructureDemandV2.validate",kind:"method"})
//! MERGE (tests:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests",kind:"module"})
//! MERGE (fixture:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::ready_fixture",kind:"function"})
//! MERGE (id:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::id",kind:"function"})
//! MERGE (ready_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::matching_fresh_readiness_is_accepted",kind:"test"})
//! MERGE (binding_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::changed_current_binding_fails_closed",kind:"test"})
//! MERGE (reservation_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::expired_or_mismatched_host_reservation_fails_closed",kind:"test"})
//! MERGE (readiness_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::every_readiness_layer_must_be_ready",kind:"test"})
//! MERGE (stale_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::stale_or_future_readiness_evidence_fails_closed",kind:"test"})
//! MERGE (scope_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::readiness_evidence_must_match_host_and_cluster_generations_and_run",kind:"test"})
//! MERGE (dual_reservation_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::readiness_evidence_must_match_both_capacity_reservations",kind:"test"})
//! MERGE (remote_test:Symbol {id:"crates/domain-infrastructure/src/readiness.rs::tests::remote_provider_marks_multipass_layers_not_applicable",kind:"test"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(state)
//! MERGE (module)-[:DEFINES]->(acceptance)
//! MERGE (module)-[:DEFINES]->(evidence)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (module)-[:DEFINES]->(gate)
//! MERGE (module)-[:DEFINES]->(acceptance_matches)
//! MERGE (module)-[:DEFINES]->(evidence_matches)
//! MERGE (module)-[:DEFINES]->(provider_layers_match)
//! MERGE (module)-[:DEFINES]->(readiness_age)
//! MERGE (module)-[:DEFINES]->(tests)
//! MERGE (tests)-[:DEFINES]->(fixture)
//! MERGE (tests)-[:DEFINES]->(id)
//! MERGE (tests)-[:DEFINES]->(ready_test)
//! MERGE (tests)-[:DEFINES]->(binding_test)
//! MERGE (tests)-[:DEFINES]->(reservation_test)
//! MERGE (tests)-[:DEFINES]->(readiness_test)
//! MERGE (tests)-[:DEFINES]->(stale_test)
//! MERGE (tests)-[:DEFINES]->(scope_test)
//! MERGE (tests)-[:DEFINES]->(dual_reservation_test)
//! MERGE (tests)-[:DEFINES]->(remote_test)
//! MERGE (gate)-[:CALLS]->(acceptance_matches)
//! MERGE (gate)-[:CALLS]->(evidence_matches)
//! MERGE (gate)-[:CALLS]->(provider_layers_match)
//! MERGE (gate)-[:CALLS]->(binding_digest)
//! MERGE (gate)-[:CALLS]->(demand_validate)
//! MERGE (gate)-[:CALLS]->(binding_validate)
//! MERGE (gate)-[:USES]->(schema_version)
//! MERGE (gate)-[:USES]->(readiness_age)
//! MERGE (acceptance_matches)-[:USES_TYPE]->(demand)
//! MERGE (evidence_matches)-[:USES_TYPE]->(demand)
//! MERGE (provider_layers_match)-[:USES_TYPE]->(state)
//! MERGE (gate)-[:USES_TYPE]->(state)
//! MERGE (ready_test)-[:CALLS]->(fixture)
//! MERGE (ready_test)-[:CALLS]->(gate)
//! MERGE (binding_test)-[:CALLS]->(fixture)
//! MERGE (binding_test)-[:CALLS]->(gate)
//! MERGE (reservation_test)-[:CALLS]->(fixture)
//! MERGE (reservation_test)-[:CALLS]->(gate)
//! MERGE (readiness_test)-[:CALLS]->(fixture)
//! MERGE (readiness_test)-[:CALLS]->(gate)
//! MERGE (stale_test)-[:CALLS]->(fixture)
//! MERGE (stale_test)-[:CALLS]->(gate)
//! MERGE (scope_test)-[:CALLS]->(fixture)
//! MERGE (scope_test)-[:CALLS]->(gate)
//! MERGE (dual_reservation_test)-[:CALLS]->(fixture)
//! MERGE (dual_reservation_test)-[:CALLS]->(gate)
//! MERGE (remote_test)-[:CALLS]->(fixture)
//! MERGE (remote_test)-[:CALLS]->(gate)
//! MERGE (fixture)-[:CALLS]->(id)
//! MERGE (gate)-[:USES_TYPE]->(acceptance)
//! MERGE (gate)-[:USES_TYPE]->(evidence)
//! MERGE (gate)-[:USES_TYPE]->(error)
//! MERGE (acceptance)-[:USES_TYPE]->(demand)
//! MERGE (evidence)-[:USES_TYPE]->(demand)
//! MERGE (acceptance)-[:USES_TYPE]->(state)
//! MERGE (evidence)-[:USES_TYPE]->(state)
//! MERGE (module)-[:DEFINES]->(schema_version)
//! @endcypher

//! Versioned, fail-closed evidence checks between Host reservation and Run execution.
//!
//! This contract never authorizes an actor or starts a process. The caller must separately
//! recheck the immutable Schedule run-as principal, occurrence lease, quota, and current Run
//! binding in the owning transaction immediately before changing the TaskExecutionRun state.
//! Multipass remains behind the trusted Host Infrastructure Manager; these payloads contain no
//! host paths, credentials, shell commands, or VM instance names.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{HostInfrastructureDemandV2, ProviderKind, RunInfrastructureBindingSnapshotV2};

/// Wire schema version for Host reservation and readiness evidence.
const INFRASTRUCTURE_READINESS_SCHEMA_VERSION_V2: u16 = 2;
const MAX_READINESS_AGE_SECONDS: i64 = 30;

/// A provider observation; unknown and failed block execution, and not-applicable is provider-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfrastructureReadinessStateV1 {
    /// No trustworthy observation is available.
    Unknown,
    /// The named readiness check passed for the reported generation.
    Ready,
    /// The readiness check ran and failed.
    Failed,
    /// The bound provider has no such layer; accepted only for that provider profile.
    NotApplicable,
}

/// Infrastructure-owner receipt for a Host/environment capacity reservation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureCapacityReservationAcceptanceV2 {
    /// Explicit wire schema version.
    pub schema_version: u16,
    /// Demand Outbox event this receipt acknowledges.
    pub demand_event_id: Uuid,
    /// Infrastructure-owner acceptance event identity.
    pub acceptance_event_id: Uuid,
    /// Tenant and Run scope copied from the demand.
    pub tenant_id: Uuid,
    /// Project containing the admitted Run.
    pub project_id: Uuid,
    /// Cloud Branch that owns the Engineering Run.
    pub branch_id: Uuid,
    /// Canonical Engineering Run identity.
    pub engineering_run_id: Uuid,
    /// TaskExecutionRun waiting for Host capacity.
    pub task_execution_run_id: Uuid,
    /// Schedule Rule identity pinned by the occurrence.
    pub schedule_rule_id: Uuid,
    /// Schedule Rule revision pinned by the occurrence.
    pub schedule_rule_version: i64,
    /// Schedule occurrence that produced the demand.
    pub occurrence_id: Uuid,
    /// Current binding identity and revision accepted by Infrastructure.
    pub binding_id: Uuid,
    /// Monotonic binding revision.
    pub binding_revision: i64,
    /// HostEnvironment identity.
    pub host_environment_id: Uuid,
    /// Exact K3s cluster reserved for this Run.
    pub k3s_cluster_id: Uuid,
    /// Owned Host generation used for the reservation.
    pub host_generation: i64,
    /// Generation of the exact K3s cluster.
    pub k3s_cluster_generation: i64,
    /// Infrastructure-owner HostEnvironment capacity reservation identity.
    pub host_capacity_reservation_id: Uuid,
    /// Infrastructure-owner exact-cluster capacity reservation identity.
    pub cluster_capacity_reservation_id: Uuid,
    /// Binding digest acknowledged by Infrastructure.
    pub binding_digest: [u8; 32],
    /// Acceptance timestamp.
    pub accepted_at: DateTime<Utc>,
    /// Reservation lease expiry.
    pub expires_at: DateTime<Utc>,
}

impl InfrastructureCapacityReservationAcceptanceV2 {
    fn matches_demand(
        &self,
        demand: &HostInfrastructureDemandV2,
        expected_binding_digest: [u8; 32],
    ) -> bool {
        self.demand_event_id == demand.event_id
            && self.tenant_id == demand.tenant_id
            && self.project_id == demand.project_id
            && self.branch_id == demand.branch_id
            && self.engineering_run_id == demand.engineering_run_id
            && self.task_execution_run_id == demand.task_execution_run_id
            && self.schedule_rule_id == demand.schedule_rule_id
            && self.schedule_rule_version == demand.schedule_rule_version
            && self.occurrence_id == demand.occurrence_id
            && self.binding_id == demand.binding.binding_id
            && self.binding_revision == demand.binding.binding_revision
            && self.host_environment_id == demand.binding.host_environment_id
            && self.k3s_cluster_id == demand.binding.k3s_cluster_id
            && self.k3s_cluster_generation == demand.binding.k3s_cluster_generation
            && self.binding_digest == expected_binding_digest
    }
}

/// Fresh provider and runtime observations tied to one accepted Host generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureReadinessEvidenceV2 {
    /// Explicit wire schema version.
    pub schema_version: u16,
    /// Infrastructure-owner evidence event identity.
    pub event_id: Uuid,
    /// Scope copied from the accepted reservation.
    pub tenant_id: Uuid,
    /// Project containing the admitted Run.
    pub project_id: Uuid,
    /// Cloud Branch that owns the Engineering Run.
    pub branch_id: Uuid,
    /// Canonical Engineering Run identity.
    pub engineering_run_id: Uuid,
    /// TaskExecutionRun whose execution gate is being evaluated.
    pub task_execution_run_id: Uuid,
    /// Schedule Rule identity pinned by the occurrence.
    pub schedule_rule_id: Uuid,
    /// Schedule Rule revision pinned by the occurrence.
    pub schedule_rule_version: i64,
    /// Schedule occurrence that produced the demand.
    pub occurrence_id: Uuid,
    /// Accepted HostEnvironment reservation being observed.
    pub host_capacity_reservation_id: Uuid,
    /// Accepted exact-cluster reservation being observed.
    pub cluster_capacity_reservation_id: Uuid,
    /// Current binding identity and revision.
    pub binding_id: Uuid,
    /// Monotonic binding revision.
    pub binding_revision: i64,
    /// HostEnvironment identity.
    pub host_environment_id: Uuid,
    /// Exact K3s cluster whose readiness is observed.
    pub k3s_cluster_id: Uuid,
    /// Owned Host generation used for the reservation and readiness checks.
    pub host_generation: i64,
    /// Generation of the exact K3s cluster observed.
    pub k3s_cluster_generation: i64,
    /// Current Run binding digest observed by Infrastructure.
    pub binding_digest: [u8; 32],
    /// Trusted Manager operation receipt identity for this Host generation.
    pub manager_operation_receipt_id: Uuid,
    /// Manager operation completion; this is separate from daemon and guest readiness.
    pub manager_operation: InfrastructureReadinessStateV1,
    /// Multipass daemon readiness; must be NotApplicable for a remote K3s provider.
    pub multipass_daemon: InfrastructureReadinessStateV1,
    /// Aggregate readiness of all owned Linux guests backing this cluster; NotApplicable remotely.
    pub guest_vm: InfrastructureReadinessStateV1,
    /// Time at which the complete readiness snapshot was observed.
    pub observed_at: DateTime<Utc>,
    /// Kubernetes API readiness within the selected Linux guest/provider.
    pub k3s_api: InfrastructureReadinessStateV1,
    /// Number of Ready nodes observed in the bound Run environment.
    pub ready_node_count: u16,
    /// Minimum Ready node count required by the pinned provider profile.
    pub required_node_count: u16,
    /// Container runtime/CRI readiness on every required node.
    pub container_runtime: InfrastructureReadinessStateV1,
    /// Independently enforced Agent Runtime sandbox readiness.
    pub agent_runtime_sandbox: InfrastructureReadinessStateV1,
    /// Explicitly scoped Worktree workspace bridge readiness.
    pub workspace_bridge: InfrastructureReadinessStateV1,
}

impl InfrastructureReadinessEvidenceV2 {
    fn matches_demand(
        &self,
        demand: &HostInfrastructureDemandV2,
        host_capacity_reservation_id: Uuid,
        cluster_capacity_reservation_id: Uuid,
        expected_binding_digest: [u8; 32],
    ) -> bool {
        self.tenant_id == demand.tenant_id
            && self.project_id == demand.project_id
            && self.branch_id == demand.branch_id
            && self.engineering_run_id == demand.engineering_run_id
            && self.task_execution_run_id == demand.task_execution_run_id
            && self.schedule_rule_id == demand.schedule_rule_id
            && self.schedule_rule_version == demand.schedule_rule_version
            && self.occurrence_id == demand.occurrence_id
            && self.host_capacity_reservation_id == host_capacity_reservation_id
            && self.cluster_capacity_reservation_id == cluster_capacity_reservation_id
            && self.binding_id == demand.binding.binding_id
            && self.binding_revision == demand.binding.binding_revision
            && self.host_environment_id == demand.binding.host_environment_id
            && self.k3s_cluster_id == demand.binding.k3s_cluster_id
            && self.k3s_cluster_generation == demand.binding.k3s_cluster_generation
            && self.binding_digest == expected_binding_digest
    }

    fn provider_layers_match(&self, provider: ProviderKind) -> bool {
        match provider {
            ProviderKind::MultipassK3sCluster => {
                self.multipass_daemon == InfrastructureReadinessStateV1::Ready
                    && self.guest_vm == InfrastructureReadinessStateV1::Ready
            }
            ProviderKind::RemoteK3s => {
                self.multipass_daemon == InfrastructureReadinessStateV1::NotApplicable
                    && self.guest_vm == InfrastructureReadinessStateV1::NotApplicable
            }
        }
    }
}

/// Validate that current Host capacity and every independent readiness layer permit a later start.
///
/// The result is evidence validation only. A caller still needs the current authorization, lease,
/// Project/Run quota, and binding checks in the same Run-owner transaction that records start.
pub fn validate_infrastructure_execution_readiness(
    demand: &HostInfrastructureDemandV2,
    current_binding: &RunInfrastructureBindingSnapshotV2,
    reservation: &InfrastructureCapacityReservationAcceptanceV2,
    evidence: &InfrastructureReadinessEvidenceV2,
    now: DateTime<Utc>,
) -> Result<(), InfrastructureReadinessError> {
    demand
        .validate()
        .map_err(|_| InfrastructureReadinessError::InvalidDemand)?;
    current_binding
        .validate()
        .map_err(|_| InfrastructureReadinessError::InvalidBinding)?;
    let expected_binding_digest = demand
        .binding
        .digest()
        .map_err(|_| InfrastructureReadinessError::InvalidDemand)?;
    if now >= demand.deadline_at {
        return Err(InfrastructureReadinessError::AdmissionDeadlineExpired);
    }
    if current_binding
        .digest()
        .map_err(|_| InfrastructureReadinessError::InvalidBinding)?
        != expected_binding_digest
    {
        return Err(InfrastructureReadinessError::BindingChanged);
    }
    if reservation.schema_version != INFRASTRUCTURE_READINESS_SCHEMA_VERSION_V2
        || evidence.schema_version != INFRASTRUCTURE_READINESS_SCHEMA_VERSION_V2
    {
        return Err(InfrastructureReadinessError::UnsupportedSchemaVersion);
    }
    if [
        reservation.demand_event_id,
        reservation.acceptance_event_id,
        reservation.tenant_id,
        reservation.project_id,
        reservation.branch_id,
        reservation.engineering_run_id,
        reservation.task_execution_run_id,
        reservation.schedule_rule_id,
        reservation.occurrence_id,
        reservation.binding_id,
        reservation.host_environment_id,
        reservation.k3s_cluster_id,
        reservation.host_capacity_reservation_id,
        reservation.cluster_capacity_reservation_id,
    ]
    .iter()
    .any(Uuid::is_nil)
        || reservation.binding_revision <= 0
        || reservation.schedule_rule_version <= 0
        || reservation.host_generation <= 0
        || reservation.k3s_cluster_generation <= 0
        || reservation.accepted_at < demand.requested_at
        || reservation.accepted_at > now
        || reservation.expires_at <= reservation.accepted_at
    {
        return Err(InfrastructureReadinessError::InvalidHostReservation);
    }
    if reservation.expires_at <= now {
        return Err(InfrastructureReadinessError::HostReservationExpired);
    }
    if !reservation.matches_demand(demand, expected_binding_digest) {
        return Err(InfrastructureReadinessError::ScopeMismatch);
    }
    if [
        evidence.event_id,
        evidence.manager_operation_receipt_id,
        evidence.tenant_id,
        evidence.project_id,
        evidence.branch_id,
        evidence.engineering_run_id,
        evidence.task_execution_run_id,
        evidence.schedule_rule_id,
        evidence.occurrence_id,
        evidence.host_capacity_reservation_id,
        evidence.cluster_capacity_reservation_id,
        evidence.binding_id,
        evidence.host_environment_id,
        evidence.k3s_cluster_id,
    ]
    .iter()
    .any(Uuid::is_nil)
        || evidence.binding_revision <= 0
        || evidence.schedule_rule_version <= 0
        || evidence.host_generation <= 0
        || evidence.k3s_cluster_generation <= 0
        || evidence.required_node_count == 0
    {
        return Err(InfrastructureReadinessError::InvalidReadinessEvidence);
    }
    if !evidence.matches_demand(
        demand,
        reservation.host_capacity_reservation_id,
        reservation.cluster_capacity_reservation_id,
        expected_binding_digest,
    ) || evidence.host_generation != reservation.host_generation
        || evidence.k3s_cluster_generation != reservation.k3s_cluster_generation
    {
        return Err(InfrastructureReadinessError::ScopeMismatch);
    }
    if evidence.observed_at < reservation.accepted_at {
        return Err(InfrastructureReadinessError::ReadinessEvidenceStale);
    }
    let age = now.signed_duration_since(evidence.observed_at);
    if age < Duration::zero() || age > Duration::seconds(MAX_READINESS_AGE_SECONDS) {
        return Err(InfrastructureReadinessError::ReadinessEvidenceStale);
    }
    if evidence.manager_operation != InfrastructureReadinessStateV1::Ready
        || !evidence.provider_layers_match(current_binding.provider)
        || evidence.k3s_api != InfrastructureReadinessStateV1::Ready
        || evidence.ready_node_count < evidence.required_node_count
        || evidence.container_runtime != InfrastructureReadinessStateV1::Ready
        || evidence.agent_runtime_sandbox != InfrastructureReadinessStateV1::Ready
        || evidence.workspace_bridge != InfrastructureReadinessStateV1::Ready
    {
        return Err(InfrastructureReadinessError::ReadinessIncomplete);
    }
    Ok(())
}

/// Rejected infrastructure execution evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InfrastructureReadinessError {
    /// The demand or current binding does not satisfy its v1 contract.
    #[error("invalid infrastructure demand or binding")]
    InvalidDemand,
    /// The current binding no longer matches the demand's immutable snapshot.
    #[error("Run infrastructure binding changed")]
    BindingChanged,
    /// The wire schema version is not supported.
    #[error("unsupported infrastructure readiness schema version")]
    UnsupportedSchemaVersion,
    /// The accepted Host reservation is malformed.
    #[error("invalid Host capacity reservation")]
    InvalidHostReservation,
    /// The Host reservation lease expired.
    #[error("Host capacity reservation expired")]
    HostReservationExpired,
    /// A Host receipt or readiness fact belongs to another Run, occurrence, binding, or generation.
    #[error("infrastructure readiness scope mismatch")]
    ScopeMismatch,
    /// The Schedule admission deadline has elapsed.
    #[error("Schedule admission deadline expired")]
    AdmissionDeadlineExpired,
    /// The observation is too old or comes from the future.
    #[error("infrastructure readiness evidence is stale")]
    ReadinessEvidenceStale,
    /// The readiness evidence is malformed.
    #[error("invalid infrastructure readiness evidence")]
    InvalidReadinessEvidence,
    /// Manager/provider/guest, K3s, CRI, sandbox, or workspace bridge readiness is unknown or failed.
    #[error("infrastructure readiness is incomplete")]
    ReadinessIncomplete,
    /// Current Run binding is invalid or incomplete.
    #[error("invalid current Run infrastructure binding")]
    InvalidBinding,
}

#[cfg(test)]
mod tests {
    use super::{
        validate_infrastructure_execution_readiness, InfrastructureCapacityReservationAcceptanceV2,
        InfrastructureReadinessError, InfrastructureReadinessEvidenceV2,
        InfrastructureReadinessStateV1,
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

    fn ready_fixture() -> (
        HostInfrastructureDemandV2,
        RunInfrastructureBindingSnapshotV2,
        InfrastructureCapacityReservationAcceptanceV2,
        InfrastructureReadinessEvidenceV2,
    ) {
        let binding = RunInfrastructureBindingSnapshotV2 {
            schema_version: 2,
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            binding_id: id(5),
            binding_revision: 1,
            host_environment_id: id(6),
            k3s_cluster_id: id(18),
            k3s_cluster_generation: 3,
            cluster_placement_policy: ClusterPlacementPolicyV1::SharedPool,
            provider: ProviderKind::MultipassK3sCluster,
            provider_profile_id: id(7),
            provider_profile_revision: 3,
            provider_profile_digest: [0x11; 32],
            namespace: KubernetesDnsLabel::new("run-04").unwrap(),
            service_account: KubernetesDnsLabel::new("agent-runtime").unwrap(),
            runtime_profile_id: id(8),
            runtime_profile_revision: 2,
            runtime_profile_digest: [0x22; 32],
            resource_budget: ExecutionResourceBudgetV1 {
                max_rss_bytes: 512 * 1024 * 1024,
                max_cpu_ms: 60_000,
                max_runtime_ms: 600_000,
                max_child_processes: 8,
                max_parallel_tools: 4,
                max_provider_calls: 100,
                max_output_bytes: 1_048_576,
                max_event_buffer_bytes: 262_144,
            },
        };
        let requested_at = Utc.timestamp_opt(1_791_300_000, 0).unwrap();
        let deadline_at = requested_at + Duration::minutes(10);
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
            binding: binding.clone(),
            resource_budget: binding.resource_budget,
            requested_at,
            deadline_at,
        };
        let now = requested_at + Duration::seconds(100);
        let reservation = InfrastructureCapacityReservationAcceptanceV2 {
            schema_version: 2,
            demand_event_id: demand.event_id,
            acceptance_event_id: id(14),
            tenant_id: demand.tenant_id,
            project_id: demand.project_id,
            branch_id: demand.branch_id,
            engineering_run_id: demand.engineering_run_id,
            task_execution_run_id: demand.task_execution_run_id,
            schedule_rule_id: demand.schedule_rule_id,
            schedule_rule_version: demand.schedule_rule_version,
            occurrence_id: demand.occurrence_id,
            binding_id: binding.binding_id,
            binding_revision: binding.binding_revision,
            host_environment_id: binding.host_environment_id,
            k3s_cluster_id: binding.k3s_cluster_id,
            host_generation: 2,
            k3s_cluster_generation: binding.k3s_cluster_generation,
            host_capacity_reservation_id: id(15),
            cluster_capacity_reservation_id: id(19),
            binding_digest: binding.digest().unwrap(),
            accepted_at: now - Duration::seconds(10),
            expires_at: now + Duration::minutes(5),
        };
        let evidence = InfrastructureReadinessEvidenceV2 {
            schema_version: 2,
            event_id: id(16),
            tenant_id: demand.tenant_id,
            project_id: demand.project_id,
            branch_id: demand.branch_id,
            engineering_run_id: demand.engineering_run_id,
            task_execution_run_id: demand.task_execution_run_id,
            schedule_rule_id: demand.schedule_rule_id,
            schedule_rule_version: demand.schedule_rule_version,
            occurrence_id: demand.occurrence_id,
            host_capacity_reservation_id: reservation.host_capacity_reservation_id,
            cluster_capacity_reservation_id: reservation.cluster_capacity_reservation_id,
            binding_id: binding.binding_id,
            binding_revision: binding.binding_revision,
            host_environment_id: binding.host_environment_id,
            k3s_cluster_id: binding.k3s_cluster_id,
            host_generation: reservation.host_generation,
            k3s_cluster_generation: reservation.k3s_cluster_generation,
            binding_digest: binding.digest().unwrap(),
            manager_operation_receipt_id: id(17),
            manager_operation: InfrastructureReadinessStateV1::Ready,
            multipass_daemon: InfrastructureReadinessStateV1::Ready,
            guest_vm: InfrastructureReadinessStateV1::Ready,
            observed_at: now - Duration::seconds(2),
            k3s_api: InfrastructureReadinessStateV1::Ready,
            ready_node_count: 2,
            required_node_count: 1,
            container_runtime: InfrastructureReadinessStateV1::Ready,
            agent_runtime_sandbox: InfrastructureReadinessStateV1::Ready,
            workspace_bridge: InfrastructureReadinessStateV1::Ready,
        };
        (demand, binding, reservation, evidence)
    }

    #[test]
    fn matching_fresh_readiness_is_accepted() {
        let (demand, binding, reservation, evidence) = ready_fixture();
        let now = evidence.observed_at + Duration::seconds(2);
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Ok(())
        );
    }

    #[test]
    fn changed_current_binding_fails_closed() {
        let (demand, mut binding, reservation, evidence) = ready_fixture();
        binding.binding_revision += 1;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                evidence.observed_at + Duration::seconds(2),
            ),
            Err(InfrastructureReadinessError::BindingChanged)
        );
    }

    #[test]
    fn expired_or_mismatched_host_reservation_fails_closed() {
        let (demand, binding, mut reservation, evidence) = ready_fixture();
        let now = evidence.observed_at + Duration::seconds(2);
        reservation.expires_at = now;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::HostReservationExpired)
        );
        reservation.expires_at = now + Duration::minutes(1);
        reservation.demand_event_id = id(99);
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ScopeMismatch)
        );
    }

    #[test]
    fn every_readiness_layer_must_be_ready() {
        let (demand, binding, reservation, mut evidence) = ready_fixture();
        let now = evidence.observed_at + Duration::seconds(2);
        evidence.k3s_api = InfrastructureReadinessStateV1::Unknown;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
        evidence.k3s_api = InfrastructureReadinessStateV1::Ready;
        evidence.manager_operation = InfrastructureReadinessStateV1::Unknown;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
        evidence.manager_operation = InfrastructureReadinessStateV1::Ready;
        evidence.multipass_daemon = InfrastructureReadinessStateV1::Failed;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
        evidence.multipass_daemon = InfrastructureReadinessStateV1::NotApplicable;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
        evidence.multipass_daemon = InfrastructureReadinessStateV1::Ready;
        evidence.guest_vm = InfrastructureReadinessStateV1::Unknown;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
        evidence.guest_vm = InfrastructureReadinessStateV1::Ready;
        evidence.k3s_api = InfrastructureReadinessStateV1::Ready;
        evidence.ready_node_count = 0;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
        evidence.ready_node_count = 2;
        evidence.agent_runtime_sandbox = InfrastructureReadinessStateV1::Failed;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessIncomplete)
        );
    }

    #[test]
    fn remote_provider_marks_multipass_layers_not_applicable() {
        let (mut demand, _, mut reservation, mut evidence) = ready_fixture();
        demand.binding.provider = ProviderKind::RemoteK3s;
        let binding = demand.binding.clone();
        let digest = binding.digest().unwrap();
        reservation.binding_digest = digest;
        evidence.binding_digest = digest;
        evidence.multipass_daemon = InfrastructureReadinessStateV1::NotApplicable;
        evidence.guest_vm = InfrastructureReadinessStateV1::NotApplicable;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                evidence.observed_at + Duration::seconds(2),
            ),
            Ok(())
        );
    }

    #[test]
    fn stale_or_future_readiness_evidence_fails_closed() {
        let (demand, binding, reservation, mut evidence) = ready_fixture();
        let now = evidence.observed_at + Duration::seconds(31);
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessEvidenceStale)
        );
        evidence.observed_at = now + Duration::seconds(1);
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ReadinessEvidenceStale)
        );
    }

    #[test]
    fn readiness_evidence_must_match_host_and_cluster_generations_and_run() {
        let (demand, binding, reservation, mut evidence) = ready_fixture();
        let now = evidence.observed_at + Duration::seconds(2);
        evidence.host_generation += 1;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ScopeMismatch)
        );
        evidence.host_generation = reservation.host_generation;
        evidence.k3s_cluster_generation += 1;
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ScopeMismatch)
        );
    }

    #[test]
    fn readiness_evidence_must_match_both_capacity_reservations() {
        let (demand, binding, reservation, mut evidence) = ready_fixture();
        let now = evidence.observed_at + Duration::seconds(2);
        evidence.host_capacity_reservation_id = id(99);
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ScopeMismatch)
        );
        evidence.host_capacity_reservation_id = reservation.host_capacity_reservation_id;
        evidence.cluster_capacity_reservation_id = id(98);
        assert_eq!(
            validate_infrastructure_execution_readiness(
                &demand,
                &binding,
                &reservation,
                &evidence,
                now,
            ),
            Err(InfrastructureReadinessError::ScopeMismatch)
        );
    }
}
