//! @cypher schema=1 source_sha256=5346750da31e4cdf4f8a4f9956a7cb1642071dcef59b9e070dd0944fe90c2381
//! MERGE (self:File {path:"crates/domain-infrastructure/src/lib.rs"})
//! MERGE (provider:Type {id:"crates/domain-infrastructure/src/lib.rs::ProviderKind",kind:"enum"})
//! MERGE (label:Type {id:"crates/domain-infrastructure/src/lib.rs::KubernetesDnsLabel",kind:"struct"})
//! MERGE (budget:Type {id:"crates/domain-infrastructure/src/lib.rs::ExecutionResourceBudgetV1",kind:"struct"})
//! MERGE (binding:Type {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV1",kind:"struct"})
//! MERGE (demand:Type {id:"crates/domain-infrastructure/src/lib.rs::HostInfrastructureDemandV1",kind:"struct"})
//! MERGE (error:Type {id:"crates/domain-infrastructure/src/lib.rs::InfrastructureContractError",kind:"enum"})
//! MERGE (binding_schema_version:Variable {id:"crates/domain-infrastructure/src/lib.rs::RUN_INFRASTRUCTURE_BINDING_SCHEMA_VERSION_V1",kind:"constant"})
//! MERGE (demand_schema_version:Variable {id:"crates/domain-infrastructure/src/lib.rs::HOST_INFRASTRUCTURE_DEMAND_SCHEMA_VERSION_V1",kind:"constant"})
//! MERGE (label_limit:Variable {id:"crates/domain-infrastructure/src/lib.rs::MAX_KUBERNETES_DNS_LABEL_BYTES",kind:"constant"})
//! MERGE (hex_alphabet:Variable {id:"crates/domain-infrastructure/src/lib.rs::hex_bytes::HEX",kind:"constant"})
//! MERGE (provider_wire:Symbol {id:"crates/domain-infrastructure/src/lib.rs::ProviderKind.wire_value",kind:"method"})
//! MERGE (label_new:Symbol {id:"crates/domain-infrastructure/src/lib.rs::KubernetesDnsLabel.new",kind:"method"})
//! MERGE (label_as_str:Symbol {id:"crates/domain-infrastructure/src/lib.rs::KubernetesDnsLabel.as_str",kind:"method"})
//! MERGE (label_try_from:Symbol {id:"crates/domain-infrastructure/src/lib.rs::KubernetesDnsLabel.try_from",kind:"method"})
//! MERGE (label_into_string:Symbol {id:"crates/domain-infrastructure/src/lib.rs::String.from",kind:"method"})
//! MERGE (budget_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::ExecutionResourceBudgetV1.validate",kind:"method"})
//! MERGE (binding_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV1.validate",kind:"method"})
//! MERGE (binding_digest:Symbol {id:"crates/domain-infrastructure/src/lib.rs::RunInfrastructureBindingSnapshotV1.digest",kind:"method"})
//! MERGE (demand_validate:Symbol {id:"crates/domain-infrastructure/src/lib.rs::HostInfrastructureDemandV1.validate",kind:"method"})
//! MERGE (dedupe_key:Symbol {id:"crates/domain-infrastructure/src/lib.rs::HostInfrastructureDemandV1.deduplication_key",kind:"method"})
//! MERGE (hex_bytes:Symbol {id:"crates/domain-infrastructure/src/lib.rs::hex_bytes",kind:"function"})
//! MERGE (test_id:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::id",kind:"function"})
//! MERGE (valid_budget:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::valid_budget",kind:"function"})
//! MERGE (valid_binding:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::valid_binding",kind:"function"})
//! MERGE (valid_demand:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::valid_demand",kind:"function"})
//! MERGE (namespace_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::kubernetes_namespace_is_a_single_bounded_dns_label",kind:"test"})
//! MERGE (budget_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::resource_budget_rejects_negative_or_zero_runtime",kind:"test"})
//! MERGE (binding_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::binding_requires_complete_non_nil_scope_and_revision",kind:"test"})
//! MERGE (digest_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::binding_digest_changes_when_any_pinned_revision_changes",kind:"test"})
//! MERGE (digest_stability_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::binding_digest_is_stable_and_covers_profile_digest_namespace_and_budget",kind:"test"})
//! MERGE (demand_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::demand_scope_and_budget_must_match_the_binding",kind:"test"})
//! MERGE (idempotency_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::schedule_occurrence_is_the_idempotency_scope",kind:"test"})
//! MERGE (schema_version_test:Symbol {id:"crates/domain-infrastructure/src/lib.rs::tests::contract_versions_are_explicit_and_reject_unknown_values",kind:"test"})
//! MERGE (self)-[:DEFINES]->(provider)
//! MERGE (self)-[:DEFINES]->(label)
//! MERGE (self)-[:DEFINES]->(budget)
//! MERGE (self)-[:DEFINES]->(binding)
//! MERGE (self)-[:DEFINES]->(demand)
//! MERGE (self)-[:DEFINES]->(error)
//! MERGE (self)-[:DEFINES]->(binding_schema_version)
//! MERGE (self)-[:DEFINES]->(demand_schema_version)
//! MERGE (self)-[:DEFINES]->(label_limit)
//! MERGE (self)-[:DEFINES]->(hex_alphabet)
//! MERGE (self)-[:DEFINES]->(provider_wire)
//! MERGE (self)-[:DEFINES]->(label_new)
//! MERGE (self)-[:DEFINES]->(label_as_str)
//! MERGE (self)-[:DEFINES]->(label_try_from)
//! MERGE (self)-[:DEFINES]->(label_into_string)
//! MERGE (self)-[:DEFINES]->(budget_validate)
//! MERGE (self)-[:DEFINES]->(binding_validate)
//! MERGE (self)-[:DEFINES]->(binding_digest)
//! MERGE (self)-[:DEFINES]->(demand_validate)
//! MERGE (self)-[:DEFINES]->(dedupe_key)
//! MERGE (self)-[:DEFINES]->(hex_bytes)
//! MERGE (self)-[:DEFINES]->(test_id)
//! MERGE (self)-[:DEFINES]->(valid_budget)
//! MERGE (self)-[:DEFINES]->(valid_binding)
//! MERGE (self)-[:DEFINES]->(valid_demand)
//! MERGE (self)-[:DEFINES]->(namespace_test)
//! MERGE (self)-[:DEFINES]->(budget_test)
//! MERGE (self)-[:DEFINES]->(binding_test)
//! MERGE (self)-[:DEFINES]->(digest_test)
//! MERGE (self)-[:DEFINES]->(digest_stability_test)
//! MERGE (self)-[:DEFINES]->(demand_test)
//! MERGE (self)-[:DEFINES]->(idempotency_test)
//! MERGE (self)-[:DEFINES]->(schema_version_test)
//! MERGE (binding_validate)-[:CALLS]->(budget_validate)
//! MERGE (binding_digest)-[:CALLS]->(binding_validate)
//! MERGE (binding_digest)-[:CALLS]->(hex_bytes)
//! MERGE (binding_digest)-[:CALLS]->(label_as_str)
//! MERGE (binding_digest)-[:CALLS]->(provider_wire)
//! MERGE (demand_validate)-[:CALLS]->(binding_validate)
//! MERGE (demand_validate)-[:CALLS]->(budget_validate)
//! MERGE (binding_validate)-[:USES]->(binding_schema_version)
//! MERGE (demand_validate)-[:USES]->(demand_schema_version)
//! MERGE (valid_binding)-[:CALLS]->(label_new)
//! MERGE (valid_binding)-[:CALLS]->(test_id)
//! MERGE (valid_binding)-[:CALLS]->(valid_budget)
//! MERGE (valid_demand)-[:CALLS]->(valid_binding)
//! MERGE (valid_demand)-[:CALLS]->(test_id)
//! MERGE (valid_demand)-[:CALLS]->(valid_budget)
//! MERGE (valid_demand)-[:CALLS]->(demand_validate)
//! MERGE (namespace_test)-[:CALLS]->(label_new)
//! MERGE (budget_test)-[:CALLS]->(budget_validate)
//! MERGE (binding_test)-[:CALLS]->(binding_validate)
//! MERGE (binding_test)-[:CALLS]->(valid_binding)
//! MERGE (digest_test)-[:CALLS]->(binding_digest)
//! MERGE (digest_test)-[:CALLS]->(valid_binding)
//! MERGE (digest_stability_test)-[:CALLS]->(binding_digest)
//! MERGE (digest_stability_test)-[:CALLS]->(valid_binding)
//! MERGE (demand_test)-[:CALLS]->(valid_demand)
//! MERGE (demand_test)-[:CALLS]->(demand_validate)
//! MERGE (idempotency_test)-[:CALLS]->(valid_demand)
//! MERGE (idempotency_test)-[:CALLS]->(dedupe_key)
//! MERGE (schema_version_test)-[:CALLS]->(binding_validate)
//! MERGE (schema_version_test)-[:CALLS]->(demand_validate)
//! MERGE (test_id)-[:USES_TYPE]->(provider)
//! MERGE (valid_budget)-[:USES_TYPE]->(budget)
//! MERGE (label_try_from)-[:CALLS]->(label_new)
//! MERGE (label_new)-[:USES_TYPE]->(error)
//! MERGE (label_new)-[:USES]->(label_limit)
//! MERGE (hex_bytes)-[:USES]->(hex_alphabet)
//! MERGE (provider_wire)-[:USES_TYPE]->(provider)
//! MERGE (budget_validate)-[:USES_TYPE]->(error)
//! MERGE (binding_validate)-[:USES_TYPE]->(error)
//! MERGE (demand_validate)-[:USES_TYPE]->(error)
//! MERGE (binding)-[:USES_TYPE]->(provider)
//! MERGE (binding)-[:USES_TYPE]->(label)
//! MERGE (binding)-[:USES_TYPE]->(budget)
//! MERGE (demand)-[:USES_TYPE]->(binding)
//! MERGE (demand)-[:USES_TYPE]->(budget)
//! @endcypher

//! Versioned domain contracts for shared Host/K3s infrastructure admission.
//!
//! These types contain only authorization-scoped identifiers, pinned revisions,
//! bounded resource values, and readiness-independent demand facts. They do not
//! expose Multipass commands, host paths, credentials, or guest execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

const MAX_KUBERNETES_DNS_LABEL_BYTES: usize = 63;

/// Wire schema version for the Run infrastructure binding snapshot.
pub const RUN_INFRASTRUCTURE_BINDING_SCHEMA_VERSION_V1: u16 = 1;
/// Wire schema version for Host infrastructure demand events.
pub const HOST_INFRASTRUCTURE_DEMAND_SCHEMA_VERSION_V1: u16 = 1;

/// A provider profile supported by the versioned infrastructure contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// A shared Linux guest managed by Multipass with K3s running inside it.
    MultipassK3sGuest,
    /// A separately configured remote Linux K3s provider.
    RemoteK3s,
}

impl ProviderKind {
    fn wire_value(self) -> &'static str {
        match self {
            Self::MultipassK3sGuest => "multipass_k3s_guest",
            Self::RemoteK3s => "remote_k3s",
        }
    }
}

/// A Kubernetes namespace or service-account name constrained to one DNS label.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KubernetesDnsLabel(String);

impl KubernetesDnsLabel {
    /// Validate and construct a lowercase RFC 1123 DNS label of at most 63 bytes.
    pub fn new(value: impl Into<String>) -> Result<Self, InfrastructureContractError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= MAX_KUBERNETES_DNS_LABEL_BYTES
            && value
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && value
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if !valid {
            return Err(InfrastructureContractError::InvalidKubernetesDnsLabel);
        }
        Ok(Self(value))
    }

    /// Return the validated label.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for KubernetesDnsLabel {
    type Error = InfrastructureContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<KubernetesDnsLabel> for String {
    fn from(value: KubernetesDnsLabel) -> Self {
        value.0
    }
}

/// Resource limits pinned to a Run and used for Host-capacity admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionResourceBudgetV1 {
    /// Maximum resident memory in bytes.
    pub max_rss_bytes: i64,
    /// Maximum CPU time in milliseconds.
    pub max_cpu_ms: i64,
    /// Maximum wall-clock runtime in milliseconds.
    pub max_runtime_ms: i64,
    /// Maximum child processes.
    pub max_child_processes: i32,
    /// Maximum parallel tool calls.
    pub max_parallel_tools: i32,
    /// Maximum provider calls.
    pub max_provider_calls: i64,
    /// Maximum emitted output in bytes.
    pub max_output_bytes: i64,
    /// Maximum buffered event payload in bytes.
    pub max_event_buffer_bytes: i64,
}

impl ExecutionResourceBudgetV1 {
    /// Reject negative values and a missing runtime deadline.
    pub fn validate(&self) -> Result<(), InfrastructureContractError> {
        if self.max_rss_bytes < 0
            || self.max_cpu_ms < 0
            || self.max_runtime_ms <= 0
            || self.max_child_processes < 0
            || self.max_parallel_tools < 0
            || self.max_provider_calls < 0
            || self.max_output_bytes < 0
            || self.max_event_buffer_bytes < 0
        {
            return Err(InfrastructureContractError::InvalidResourceBudget);
        }
        Ok(())
    }
}

/// Immutable Run-scoped infrastructure identity captured during admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunInfrastructureBindingSnapshotV1 {
    /// Explicit wire schema version; readers reject values they do not support.
    pub schema_version: u16,
    /// Tenant that owns this binding.
    pub tenant_id: Uuid,
    /// Project containing the canonical Run.
    pub project_id: Uuid,
    /// Cloud Branch that owns the Engineering Run.
    pub branch_id: Uuid,
    /// Canonical collaboration Engineering Run, not TaskExecutionRun.
    pub engineering_run_id: Uuid,
    /// Immutable binding identity.
    pub binding_id: Uuid,
    /// Monotonic binding revision.
    pub binding_revision: i64,
    /// Shared HostEnvironment identity; this is not a Multipass instance name.
    pub host_environment_id: Uuid,
    /// Provider selected by the current binding.
    pub provider: ProviderKind,
    /// Immutable provider-profile identity and revision.
    pub provider_profile_id: Uuid,
    /// Provider-profile revision.
    pub provider_profile_revision: i64,
    /// SHA-256 of the provider-profile document.
    pub provider_profile_digest: [u8; 32],
    /// Kubernetes namespace allocated to this Run.
    pub namespace: KubernetesDnsLabel,
    /// Kubernetes identity used by workloads in this Run.
    pub service_account: KubernetesDnsLabel,
    /// Immutable Agent Runtime profile identity and revision.
    pub runtime_profile_id: Uuid,
    /// Agent Runtime profile revision.
    pub runtime_profile_revision: i64,
    /// SHA-256 of the Agent Runtime profile document.
    pub runtime_profile_digest: [u8; 32],
    /// Exact resource limits pinned for the Run.
    pub resource_budget: ExecutionResourceBudgetV1,
}

impl RunInfrastructureBindingSnapshotV1 {
    /// Validate the complete tenant/Project/Branch/Run and provider snapshot.
    pub fn validate(&self) -> Result<(), InfrastructureContractError> {
        if self.schema_version != RUN_INFRASTRUCTURE_BINDING_SCHEMA_VERSION_V1 {
            return Err(InfrastructureContractError::UnsupportedSchemaVersion);
        }
        if [
            self.tenant_id,
            self.project_id,
            self.branch_id,
            self.engineering_run_id,
            self.binding_id,
            self.host_environment_id,
            self.provider_profile_id,
            self.runtime_profile_id,
        ]
        .iter()
        .any(Uuid::is_nil)
            || self.binding_revision <= 0
            || self.provider_profile_revision <= 0
            || self.runtime_profile_revision <= 0
        {
            return Err(InfrastructureContractError::InvalidBindingIdentity);
        }
        self.resource_budget.validate()
    }

    /// Return a deterministic SHA-256 digest over every pinned binding field.
    pub fn digest(&self) -> Result<[u8; 32], InfrastructureContractError> {
        self.validate()?;
        let fields = [
            self.schema_version.to_string(),
            self.tenant_id.to_string(),
            self.project_id.to_string(),
            self.branch_id.to_string(),
            self.engineering_run_id.to_string(),
            self.binding_id.to_string(),
            self.binding_revision.to_string(),
            self.host_environment_id.to_string(),
            self.provider.wire_value().to_owned(),
            self.provider_profile_id.to_string(),
            self.provider_profile_revision.to_string(),
            hex_bytes(&self.provider_profile_digest),
            self.namespace.as_str().to_owned(),
            self.service_account.as_str().to_owned(),
            self.runtime_profile_id.to_string(),
            self.runtime_profile_revision.to_string(),
            hex_bytes(&self.runtime_profile_digest),
            self.resource_budget.max_rss_bytes.to_string(),
            self.resource_budget.max_cpu_ms.to_string(),
            self.resource_budget.max_runtime_ms.to_string(),
            self.resource_budget.max_child_processes.to_string(),
            self.resource_budget.max_parallel_tools.to_string(),
            self.resource_budget.max_provider_calls.to_string(),
            self.resource_budget.max_output_bytes.to_string(),
            self.resource_budget.max_event_buffer_bytes.to_string(),
        ];
        let mut hasher = Sha256::new();
        hasher.update(b"star.run-infrastructure-binding.v1");
        hasher.update([0]);
        for field in fields {
            hasher.update((field.len() as u64).to_be_bytes());
            hasher.update(field.as_bytes());
        }
        Ok(hasher.finalize().into())
    }
}

/// Versioned Run-owner event asking Infrastructure to reserve shared Host capacity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostInfrastructureDemandV1 {
    /// Explicit wire schema version; consumers reject values they do not support.
    pub schema_version: u16,
    /// Unique Outbox event identity.
    pub event_id: Uuid,
    /// Event correlation shared with the admitted TaskExecutionRun.
    pub correlation_id: Uuid,
    /// Tenant that scopes idempotent consumption.
    pub tenant_id: Uuid,
    /// Project containing the admitted Run.
    pub project_id: Uuid,
    /// Cloud Branch that owns the Engineering Run.
    pub branch_id: Uuid,
    /// Canonical Engineering Run identity.
    pub engineering_run_id: Uuid,
    /// TaskExecutionRun identity created by the Run owner.
    pub task_execution_run_id: Uuid,
    /// Schedule Rule identity pinned by the occurrence.
    pub schedule_rule_id: Uuid,
    /// Schedule Rule revision pinned by the occurrence.
    pub schedule_rule_version: i64,
    /// Occurrence identity; (tenant_id, occurrence_id) is the consumer idempotency scope.
    pub occurrence_id: Uuid,
    /// Immutable binding selected for this trigger/retry/resume.
    pub binding: RunInfrastructureBindingSnapshotV1,
    /// Requested and reserved Run-scoped capacity.
    pub resource_budget: ExecutionResourceBudgetV1,
    /// Time this owner committed the demand.
    pub requested_at: DateTime<Utc>,
    /// Latest time this request may be admitted.
    pub deadline_at: DateTime<Utc>,
}

impl HostInfrastructureDemandV1 {
    /// Validate scope, idempotency identity, immutable binding, and deadline.
    pub fn validate(&self) -> Result<(), InfrastructureContractError> {
        if self.schema_version != HOST_INFRASTRUCTURE_DEMAND_SCHEMA_VERSION_V1 {
            return Err(InfrastructureContractError::UnsupportedSchemaVersion);
        }
        if [
            self.event_id,
            self.correlation_id,
            self.tenant_id,
            self.project_id,
            self.branch_id,
            self.engineering_run_id,
            self.task_execution_run_id,
            self.schedule_rule_id,
            self.occurrence_id,
        ]
        .iter()
        .any(Uuid::is_nil)
            || self.schedule_rule_version <= 0
            || self.deadline_at <= self.requested_at
        {
            return Err(InfrastructureContractError::InvalidDemandIdentity);
        }
        self.binding.validate()?;
        self.resource_budget.validate()?;
        if self.tenant_id != self.binding.tenant_id
            || self.project_id != self.binding.project_id
            || self.branch_id != self.binding.branch_id
            || self.engineering_run_id != self.binding.engineering_run_id
            || self.resource_budget != self.binding.resource_budget
        {
            return Err(InfrastructureContractError::DemandBindingMismatch);
        }
        Ok(())
    }

    /// Return the idempotency key that the Infrastructure Inbox must enforce.
    pub fn deduplication_key(&self) -> (Uuid, Uuid) {
        (self.tenant_id, self.occurrence_id)
    }
}

/// Errors returned when a binding or demand violates its versioned contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InfrastructureContractError {
    /// Name is not a single lowercase RFC 1123 Kubernetes DNS label.
    #[error("invalid Kubernetes DNS label")]
    InvalidKubernetesDnsLabel,
    /// A resource budget is negative or has no bounded runtime deadline.
    #[error("invalid execution resource budget")]
    InvalidResourceBudget,
    /// Binding identity or revision is missing or invalid.
    #[error("invalid Run infrastructure binding identity")]
    InvalidBindingIdentity,
    /// Demand identity, version, or deadline is invalid.
    #[error("invalid Host infrastructure demand identity")]
    InvalidDemandIdentity,
    /// Demand scope or resource budget differs from its pinned binding.
    #[error("Host infrastructure demand does not match its binding")]
    DemandBindingMismatch,
    /// A serialized contract declares a schema version this implementation does not support.
    #[error("unsupported infrastructure contract schema version")]
    UnsupportedSchemaVersion,
}

fn hex_bytes(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(value.len() * 2);
    for byte in value {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionResourceBudgetV1, HostInfrastructureDemandV1, InfrastructureContractError,
        KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV1,
    };
    use chrono::{Duration, TimeZone, Utc};
    use uuid::Uuid;

    fn id(number: u128) -> Uuid {
        Uuid::from_u128(number)
    }

    fn valid_budget() -> ExecutionResourceBudgetV1 {
        ExecutionResourceBudgetV1 {
            max_rss_bytes: 512 * 1024 * 1024,
            max_cpu_ms: 60_000,
            max_runtime_ms: 600_000,
            max_child_processes: 8,
            max_parallel_tools: 4,
            max_provider_calls: 100,
            max_output_bytes: 1_048_576,
            max_event_buffer_bytes: 262_144,
        }
    }

    fn valid_binding() -> RunInfrastructureBindingSnapshotV1 {
        RunInfrastructureBindingSnapshotV1 {
            schema_version: 1,
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            binding_id: id(5),
            binding_revision: 1,
            host_environment_id: id(6),
            provider: ProviderKind::MultipassK3sGuest,
            provider_profile_id: id(7),
            provider_profile_revision: 3,
            provider_profile_digest: [0x11; 32],
            namespace: KubernetesDnsLabel::new("run-04").expect("valid namespace"),
            service_account: KubernetesDnsLabel::new("agent-runtime").expect("valid account"),
            runtime_profile_id: id(8),
            runtime_profile_revision: 2,
            runtime_profile_digest: [0x22; 32],
            resource_budget: valid_budget(),
        }
    }

    fn valid_demand() -> HostInfrastructureDemandV1 {
        HostInfrastructureDemandV1 {
            schema_version: 1,
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
            binding: valid_binding(),
            resource_budget: valid_budget(),
            requested_at: Utc.timestamp_opt(1_791_300_000, 0).unwrap(),
            deadline_at: Utc.timestamp_opt(1_791_300_600, 0).unwrap(),
        }
    }

    #[test]
    fn kubernetes_namespace_is_a_single_bounded_dns_label() {
        assert_eq!(
            KubernetesDnsLabel::new("run-04").unwrap().as_str(),
            "run-04"
        );
        for invalid in [
            "",
            "Upper",
            "-bad",
            "bad-",
            "two.labels",
            "contains_underscore",
        ] {
            assert_eq!(
                KubernetesDnsLabel::new(invalid),
                Err(InfrastructureContractError::InvalidKubernetesDnsLabel)
            );
        }
        assert_eq!(
            KubernetesDnsLabel::new("a".repeat(64)),
            Err(InfrastructureContractError::InvalidKubernetesDnsLabel)
        );
    }

    #[test]
    fn resource_budget_rejects_negative_or_zero_runtime() {
        let mut budget = valid_budget();
        assert!(budget.validate().is_ok());
        budget.max_runtime_ms = 0;
        assert_eq!(
            budget.validate(),
            Err(InfrastructureContractError::InvalidResourceBudget)
        );
        budget = valid_budget();
        budget.max_output_bytes = -1;
        assert_eq!(
            budget.validate(),
            Err(InfrastructureContractError::InvalidResourceBudget)
        );
    }

    #[test]
    fn binding_requires_complete_non_nil_scope_and_revision() {
        let mut binding = valid_binding();
        assert!(binding.validate().is_ok());
        binding.engineering_run_id = Uuid::nil();
        assert_eq!(
            binding.validate(),
            Err(InfrastructureContractError::InvalidBindingIdentity)
        );
        binding = valid_binding();
        binding.binding_revision = 0;
        assert_eq!(
            binding.validate(),
            Err(InfrastructureContractError::InvalidBindingIdentity)
        );
    }

    #[test]
    fn binding_digest_changes_when_any_pinned_revision_changes() {
        let binding = valid_binding();
        let original = binding.digest().unwrap();
        let mut changed = binding;
        changed.provider_profile_revision += 1;
        assert_ne!(original, changed.digest().unwrap());
    }

    #[test]
    fn binding_digest_is_stable_and_covers_profile_digest_namespace_and_budget() {
        let binding = valid_binding();
        let original = binding.digest().unwrap();
        assert_eq!(original, binding.clone().digest().unwrap());

        let mut changed = binding.clone();
        changed.provider_profile_digest[0] ^= 0xff;
        assert_ne!(original, changed.digest().unwrap());

        let mut changed = binding.clone();
        changed.namespace = KubernetesDnsLabel::new("run-05").unwrap();
        assert_ne!(original, changed.digest().unwrap());

        let mut changed = binding;
        changed.resource_budget.max_output_bytes += 1;
        assert_ne!(original, changed.digest().unwrap());
    }

    #[test]
    fn demand_scope_and_budget_must_match_the_binding() {
        let mut demand = valid_demand();
        assert!(demand.validate().is_ok());
        demand.branch_id = id(99);
        assert_eq!(
            demand.validate(),
            Err(InfrastructureContractError::DemandBindingMismatch)
        );
        demand = valid_demand();
        demand.resource_budget.max_rss_bytes += 1;
        assert_eq!(
            demand.validate(),
            Err(InfrastructureContractError::DemandBindingMismatch)
        );
        demand = valid_demand();
        demand.deadline_at = demand.requested_at - Duration::seconds(1);
        assert_eq!(
            demand.validate(),
            Err(InfrastructureContractError::InvalidDemandIdentity)
        );
    }

    #[test]
    fn schedule_occurrence_is_the_idempotency_scope() {
        let demand = valid_demand();
        assert_eq!(demand.deduplication_key(), (id(1), id(13)));
    }

    #[test]
    fn contract_versions_are_explicit_and_reject_unknown_values() {
        let mut binding = valid_binding();
        assert_eq!(binding.schema_version, 1);
        assert!(binding.validate().is_ok());
        binding.schema_version = 2;
        assert_eq!(
            binding.validate(),
            Err(InfrastructureContractError::UnsupportedSchemaVersion)
        );

        let mut demand = valid_demand();
        assert_eq!(demand.schema_version, 1);
        assert!(demand.validate().is_ok());
        demand.schema_version = 2;
        assert_eq!(
            demand.validate(),
            Err(InfrastructureContractError::UnsupportedSchemaVersion)
        );
    }
}
