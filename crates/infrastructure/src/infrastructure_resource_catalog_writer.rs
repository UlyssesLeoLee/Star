//! @cypher schema=1 source_sha256=a792d08e19eefc7a5af0e23ab06341e5881529e15cfd51ae255504665cf98c14
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs"})
//! MERGE (operation:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureManagerResourceOperationV1",kind:"struct"})
//! MERGE (identity:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::ManagerResourceIdentityV1",kind:"enum"})
//! MERGE (node_role:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::K3sNodeRoleV1",kind:"enum"})
//! MERGE (resource_kind:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureResourceKindV1",kind:"enum"})
//! MERGE (owner_state:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureResourceOwnerStateV1",kind:"enum"})
//! MERGE (lifecycle_operation:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureManagerResourceLifecycleOperationV1",kind:"struct"})
//! MERGE (lifecycle_command:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::SignedInfrastructureManagerLifecycleCommandV1",kind:"struct"})
//! MERGE (receipt:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::SignedInfrastructureManagerReceiptV1",kind:"struct"})
//! MERGE (verifier_trait:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureManagerReceiptVerifier",kind:"trait"})
//! MERGE (lifecycle_verifier_trait:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureManagerLifecycleVerifier",kind:"trait"})
//! MERGE (keyring:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::Ed25519InfrastructureReceiptVerifier",kind:"struct"})
//! MERGE (writer:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::PostgresInfrastructureResourceCatalogWriter",kind:"struct"})
//! MERGE (outcome:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::ResourceCatalogRegistrationOutcome",kind:"struct"})
//! MERGE (lifecycle_outcome:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::ResourceCatalogLifecycleOutcome",kind:"struct"})
//! MERGE (error:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::ResourceCatalogWriteError",kind:"enum"})
//! MERGE (trusted_key:Type {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::TrustedInfrastructureManagerKey",kind:"struct"})
//! MERGE (new_verifier:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::Ed25519InfrastructureReceiptVerifier::new",kind:"method"})
//! MERGE (verify:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::Ed25519InfrastructureReceiptVerifier::verify_and_digest",kind:"method"})
//! MERGE (verify_message:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::Ed25519InfrastructureReceiptVerifier::verify_message",kind:"method"})
//! MERGE (message:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::SignedInfrastructureManagerReceiptV1::signing_message",kind:"method"})
//! MERGE (lifecycle_message:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::SignedInfrastructureManagerLifecycleCommandV1::signing_message",kind:"method"})
//! MERGE (verify_lifecycle:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::Ed25519InfrastructureReceiptVerifier::verify_lifecycle_and_digest",kind:"method"})
//! MERGE (register:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::PostgresInfrastructureResourceCatalogWriter::register",kind:"method"})
//! MERGE (transition:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::PostgresInfrastructureResourceCatalogWriter::transition",kind:"method"})
//! MERGE (insert_master:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::insert_master",kind:"function"})
//! MERGE (record_event:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::record_registered_event",kind:"function"})
//! MERGE (lock_operation:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::lock_resource_catalog_operation",kind:"function"})
//! MERGE (lock_generation:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::lock_resource_generation",kind:"function"})
//! MERGE (current_state:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::current_resource_state",kind:"function"})
//! MERGE (parse_state:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::parse_owner_state",kind:"function"})
//! MERGE (validate_transition:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::validate_state_transition",kind:"function"})
//! MERGE (insert_lifecycle_receipt:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::insert_lifecycle_receipt",kind:"function"})
//! MERGE (record_lifecycle_event:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::record_lifecycle_event",kind:"function"})
//! MERGE (close_master:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::close_resource_master",kind:"function"})
//! MERGE (identity_method:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::ManagerResourceIdentityV1.identity",kind:"method"})
//! MERGE (kind_as_str:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureResourceKindV1.as_str",kind:"method"})
//! MERGE (kind_master_table:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureResourceKindV1.master_table_and_id_column",kind:"method"})
//! MERGE (state_as_str:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureResourceOwnerStateV1.as_str",kind:"method"})
//! MERGE (state_event_type:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::InfrastructureResourceOwnerStateV1.event_type",kind:"method"})
//! MERGE (role_as_str:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::K3sNodeRoleV1.as_str",kind:"method"})
//! MERGE (validate_key:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::validate_key_id",kind:"function"})
//! MERGE (validate_operation:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::validate_operation",kind:"function"})
//! MERGE (validate_lifecycle_operation:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::validate_lifecycle_operation",kind:"function"})
//! MERGE (valid_dns:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::valid_dns_name",kind:"function"})
//! MERGE (test_operation:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.operation",kind:"test"})
//! MERGE (test_signed_receipt:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.signed_receipt",kind:"test"})
//! MERGE (test_signature:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.verifies_signature_and_returns_stable_signed_payload_digest",kind:"test"})
//! MERGE (test_tamper:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.rejects_tampered_facts_and_untrusted_manager_keys",kind:"test"})
//! MERGE (test_bounds:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.rejects_bad_signature_length_and_invalid_resource_identity",kind:"test"})
//! MERGE (test_keyring_bound:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.limits_trusted_keyring_size",kind:"test"})
//! MERGE (test_signed_lifecycle:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.signed_lifecycle",kind:"test"})
//! MERGE (test_lifecycle_signature:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.verifies_lifecycle_signature_with_a_distinct_domain_and_rejects_tampering",kind:"test"})
//! MERGE (test_lifecycle_state_machine:Symbol {id:"crates/infrastructure/src/infrastructure_resource_catalog_writer.rs::tests.lifecycle_state_machine_allows_only_drain_reactivation_and_retirement",kind:"test"})
//! MERGE (receipt_table:Table {id:"infrastructure.resource_catalog_manager_receipt",class:"Transaction"})
//! MERGE (master_tables:Schema {id:"infrastructure.resource_catalog_master_tables"})
//! MERGE (event_table:Table {id:"infrastructure.resource_catalog_event",class:"Transaction"})
//! MERGE (postgres:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (ed25519:ExternalService {id:"ring::signature::ED25519",kind:"crypto"})
//! MERGE (sha256:ExternalService {id:"sha2::Sha256",kind:"hash"})
//! MERGE (self)-[:DEFINES]->(operation)
//! MERGE (self)-[:DEFINES]->(identity)
//! MERGE (self)-[:DEFINES]->(node_role)
//! MERGE (self)-[:DEFINES]->(resource_kind)
//! MERGE (self)-[:DEFINES]->(owner_state)
//! MERGE (self)-[:DEFINES]->(lifecycle_operation)
//! MERGE (self)-[:DEFINES]->(lifecycle_command)
//! MERGE (self)-[:DEFINES]->(receipt)
//! MERGE (self)-[:DEFINES]->(verifier_trait)
//! MERGE (self)-[:DEFINES]->(lifecycle_verifier_trait)
//! MERGE (self)-[:DEFINES]->(keyring)
//! MERGE (self)-[:DEFINES]->(writer)
//! MERGE (self)-[:DEFINES]->(outcome)
//! MERGE (self)-[:DEFINES]->(lifecycle_outcome)
//! MERGE (self)-[:DEFINES]->(error)
//! MERGE (self)-[:DEFINES]->(trusted_key)
//! MERGE (self)-[:DEFINES]->(insert_master)
//! MERGE (self)-[:DEFINES]->(record_event)
//! MERGE (self)-[:DEFINES]->(lock_operation)
//! MERGE (self)-[:DEFINES]->(lock_generation)
//! MERGE (self)-[:DEFINES]->(current_state)
//! MERGE (self)-[:DEFINES]->(parse_state)
//! MERGE (self)-[:DEFINES]->(validate_transition)
//! MERGE (self)-[:DEFINES]->(insert_lifecycle_receipt)
//! MERGE (self)-[:DEFINES]->(record_lifecycle_event)
//! MERGE (self)-[:DEFINES]->(close_master)
//! MERGE (self)-[:DEFINES]->(identity_method)
//! MERGE (self)-[:DEFINES]->(kind_as_str)
//! MERGE (self)-[:DEFINES]->(kind_master_table)
//! MERGE (self)-[:DEFINES]->(state_as_str)
//! MERGE (self)-[:DEFINES]->(state_event_type)
//! MERGE (self)-[:DEFINES]->(role_as_str)
//! MERGE (self)-[:DEFINES]->(validate_key)
//! MERGE (self)-[:DEFINES]->(validate_operation)
//! MERGE (self)-[:DEFINES]->(validate_lifecycle_operation)
//! MERGE (self)-[:DEFINES]->(valid_dns)
//! MERGE (self)-[:DEFINES]->(test_operation)
//! MERGE (self)-[:DEFINES]->(test_signed_receipt)
//! MERGE (self)-[:DEFINES]->(test_signature)
//! MERGE (self)-[:DEFINES]->(test_tamper)
//! MERGE (self)-[:DEFINES]->(test_bounds)
//! MERGE (self)-[:DEFINES]->(test_keyring_bound)
//! MERGE (self)-[:DEFINES]->(test_signed_lifecycle)
//! MERGE (self)-[:DEFINES]->(test_lifecycle_signature)
//! MERGE (self)-[:DEFINES]->(test_lifecycle_state_machine)
//! MERGE (keyring)-[:IMPLEMENTS]->(verifier_trait)
//! MERGE (keyring)-[:IMPLEMENTS]->(lifecycle_verifier_trait)
//! MERGE (new_verifier)-[:USES_TYPE]->(trusted_key)
//! MERGE (new_verifier)-[:CALLS]->(validate_key)
//! MERGE (verify)-[:CALLS]->(message)
//! MERGE (verify)-[:CALLS]->(verify_message)
//! MERGE (verify_lifecycle)-[:CALLS]->(lifecycle_message)
//! MERGE (verify_lifecycle)-[:CALLS]->(verify_message)
//! MERGE (verify_message)-[:DEPENDS_ON]->(ed25519)
//! MERGE (verify_message)-[:DEPENDS_ON]->(sha256)
//! MERGE (writer)-[:USES_TYPE]->(postgres)
//! MERGE (register)-[:CALLS]->(verify)
//! MERGE (register)-[:CALLS]->(insert_master)
//! MERGE (register)-[:CALLS]->(record_event)
//! MERGE (register)-[:CALLS]->(identity_method)
//! MERGE (transition)-[:CALLS]->(verify_lifecycle)
//! MERGE (transition)-[:CALLS]->(lock_operation)
//! MERGE (transition)-[:CALLS]->(lock_generation)
//! MERGE (transition)-[:CALLS]->(current_state)
//! MERGE (transition)-[:CALLS]->(kind_as_str)
//! MERGE (transition)-[:CALLS]->(state_as_str)
//! MERGE (transition)-[:CALLS]->(state_event_type)
//! MERGE (transition)-[:CALLS]->(validate_transition)
//! MERGE (transition)-[:CALLS]->(insert_lifecycle_receipt)
//! MERGE (transition)-[:CALLS]->(record_lifecycle_event)
//! MERGE (transition)-[:CALLS]->(close_master)
//! MERGE (transition)-[:WRITES]->(receipt_table)
//! MERGE (transition)-[:WRITES]->(master_tables)
//! MERGE (transition)-[:WRITES]->(event_table)
//! MERGE (message)-[:CALLS]->(validate_key)
//! MERGE (message)-[:CALLS]->(validate_operation)
//! MERGE (lifecycle_message)-[:CALLS]->(validate_key)
//! MERGE (lifecycle_message)-[:CALLS]->(validate_lifecycle_operation)
//! MERGE (validate_operation)-[:CALLS]->(identity_method)
//! MERGE (validate_operation)-[:CALLS]->(valid_dns)
//! MERGE (current_state)-[:CALLS]->(parse_state)
//! MERGE (current_state)-[:CALLS]->(kind_as_str)
//! MERGE (current_state)-[:CALLS]->(kind_master_table)
//! MERGE (insert_lifecycle_receipt)-[:CALLS]->(kind_as_str)
//! MERGE (record_lifecycle_event)-[:CALLS]->(kind_as_str)
//! MERGE (record_lifecycle_event)-[:CALLS]->(state_as_str)
//! MERGE (record_lifecycle_event)-[:CALLS]->(state_event_type)
//! MERGE (close_master)-[:CALLS]->(kind_master_table)
//! MERGE (insert_master)-[:CALLS]->(role_as_str)
//! MERGE (record_event)-[:CALLS]->(identity_method)
//! MERGE (test_signature)-[:CALLS]->(test_signed_receipt)
//! MERGE (test_signature)-[:CALLS]->(verify)
//! MERGE (test_tamper)-[:CALLS]->(test_signed_receipt)
//! MERGE (test_tamper)-[:CALLS]->(verify)
//! MERGE (test_bounds)-[:CALLS]->(test_signed_receipt)
//! MERGE (test_bounds)-[:CALLS]->(verify)
//! MERGE (test_bounds)-[:CALLS]->(message)
//! MERGE (test_keyring_bound)-[:CALLS]->(new_verifier)
//! MERGE (test_lifecycle_signature)-[:CALLS]->(test_signed_lifecycle)
//! MERGE (test_lifecycle_signature)-[:CALLS]->(verify_lifecycle)
//! MERGE (test_lifecycle_state_machine)-[:CALLS]->(validate_transition)
//! MERGE (test_signed_lifecycle)-[:CALLS]->(lifecycle_message)
//! MERGE (test_signed_receipt)-[:CALLS]->(test_operation)
//! MERGE (test_signed_receipt)-[:CALLS]->(message)
//! MERGE (register)-[:WRITES]->(receipt_table)
//! MERGE (register)-[:WRITES]->(master_tables)
//! MERGE (register)-[:WRITES]->(event_table)
//! MERGE (test_lifecycle_signature)-[:TESTS]->(verify_lifecycle)
//! MERGE (test_lifecycle_state_machine)-[:TESTS]->(validate_transition)
//! @endcypher

//! Persists one signed, trusted Infrastructure Manager registration as an atomic owner fact.
//!
//! Provider inventory is deliberately not an input to this writer. A caller must supply a
//! versioned Manager operation signed by a key bound to the Manager identity. Signature
//! verification happens before database access; the owner transaction stores the immutable
//! receipt, one Master generation, and its matching registration event together.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use ring::signature::{self, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

const OWNER_ROLE: &str = "SET LOCAL ROLE star_infrastructure_owner";
const RECEIPT_DOMAIN: &[u8] = b"star.infrastructure.resource-catalog.manager-receipt.v1\0";
const LIFECYCLE_DOMAIN: &[u8] = b"star.infrastructure.resource-catalog.manager-lifecycle.v1\0";
const MAX_KEY_ID_BYTES: usize = 128;
const MAX_TRUSTED_MANAGER_KEYS: usize = 64;

/// Immutable facts a trusted Rust Infrastructure Manager observed for one resource registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureManagerResourceOperationV1 {
    /// Wire contract version.
    pub schema_version: u16,
    /// Idempotency identity for this single resource registration.
    pub operation_id: Uuid,
    /// Immutable receipt identity assigned by the Manager.
    pub receipt_id: Uuid,
    /// Correlation identity shared with the initiating owner command.
    pub correlation_id: Uuid,
    /// Trusted actor identity recorded in the resource Master row.
    pub actor_id: Uuid,
    /// Infrastructure Manager identity associated with the signing key.
    pub manager_id: Uuid,
    /// Time the Manager observed the resource facts.
    pub observed_at: DateTime<Utc>,
    /// Exact resource identity and bounded provider facts to register.
    pub resource: ManagerResourceIdentityV1,
}

/// Typed identity snapshot for one Host, Multipass instance, K3s cluster, membership, or node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resource_kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ManagerResourceIdentityV1 {
    /// Multipass-backed host execution environment.
    HostEnvironment {
        /// Stable HostEnvironment identity.
        host_environment_id: Uuid,
        /// SCD2 generation.
        generation: i64,
        /// Rust host operating system token.
        host_os: String,
        /// Rust host architecture token.
        host_arch: String,
    },
    /// Exact Multipass guest identity observed by the trusted Manager.
    MultipassInstance {
        /// Stable Multipass instance identity.
        multipass_instance_id: Uuid,
        /// SCD2 generation.
        generation: i64,
        /// Exact HostEnvironment identity and generation.
        host_environment_id: Uuid,
        /// Exact HostEnvironment generation.
        host_generation: i64,
        /// Bounded provider-side instance name.
        provider_instance_name: String,
    },
    /// Exact K3s control-plane identity observed by the trusted Manager.
    K3sCluster {
        /// Stable K3s cluster identity.
        k3s_cluster_id: Uuid,
        /// SCD2 generation.
        generation: i64,
        /// Exact HostEnvironment identity and generation.
        host_environment_id: Uuid,
        /// Exact HostEnvironment generation.
        host_generation: i64,
        /// Non-secret digest of the cluster identity material.
        cluster_identity_digest: [u8; 32],
        /// Observed K3s version token.
        k3s_version: String,
    },
    /// Exact one-to-one guest membership of a K3s cluster.
    ClusterInstanceMembership {
        /// Stable membership identity.
        membership_id: Uuid,
        /// SCD2 generation.
        generation: i64,
        /// Exact K3s cluster identity and generation.
        k3s_cluster_id: Uuid,
        /// Exact K3s cluster generation.
        k3s_cluster_generation: i64,
        /// Exact Multipass instance identity and generation.
        multipass_instance_id: Uuid,
        /// Exact Multipass instance generation.
        multipass_instance_generation: i64,
        /// Exact HostEnvironment identity and generation.
        host_environment_id: Uuid,
        /// Exact HostEnvironment generation.
        host_generation: i64,
        /// K3s node role for this guest.
        node_role: K3sNodeRoleV1,
    },
    /// Kubernetes node identity returned by the pinned K3s API.
    KubernetesNode {
        /// Stable Kubernetes node identity.
        kubernetes_node_id: Uuid,
        /// SCD2 generation.
        generation: i64,
        /// Exact membership, cluster, instance, and host generations.
        membership_id: Uuid,
        /// Exact membership generation.
        membership_generation: i64,
        /// Exact K3s cluster identity and generation.
        k3s_cluster_id: Uuid,
        /// Exact K3s cluster generation.
        k3s_cluster_generation: i64,
        /// Exact Multipass instance identity and generation.
        multipass_instance_id: Uuid,
        /// Exact Multipass instance generation.
        multipass_instance_generation: i64,
        /// Exact HostEnvironment identity and generation.
        host_environment_id: Uuid,
        /// Exact HostEnvironment generation.
        host_generation: i64,
        /// Kubernetes object UID, never a credential.
        kubernetes_node_uid: String,
        /// Kubernetes node name.
        kubernetes_node_name: String,
        /// K3s node role.
        node_role: K3sNodeRoleV1,
    },
}

/// K3s node role used by both guest membership and the Kubernetes node catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum K3sNodeRoleV1 {
    /// K3s server/control-plane node.
    Server,
    /// K3s worker node.
    Agent,
}

/// Canonical Infrastructure resource kinds accepted by signed lifecycle commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum InfrastructureResourceKindV1 {
    /// HostEnvironment identity.
    HostEnvironment,
    /// Multipass guest identity.
    MultipassInstance,
    /// K3s cluster identity.
    K3sCluster,
    /// Exact guest membership in a K3s cluster.
    ClusterInstanceMembership,
    /// Kubernetes node identity.
    KubernetesNode,
}

impl InfrastructureResourceKindV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::HostEnvironment => "host_environment",
            Self::MultipassInstance => "multipass_instance",
            Self::K3sCluster => "k3s_cluster",
            Self::ClusterInstanceMembership => "cluster_instance_membership",
            Self::KubernetesNode => "kubernetes_node",
        }
    }

    fn master_table_and_id_column(self) -> (&'static str, &'static str) {
        match self {
            Self::HostEnvironment => ("host_environment_catalog", "host_environment_id"),
            Self::MultipassInstance => ("multipass_instance_catalog", "multipass_instance_id"),
            Self::K3sCluster => ("k3s_cluster_catalog", "k3s_cluster_id"),
            Self::ClusterInstanceMembership => ("k3s_cluster_instance_membership", "membership_id"),
            Self::KubernetesNode => ("kubernetes_node_catalog", "kubernetes_node_id"),
        }
    }
}

/// Owner state requested by a signed Manager lifecycle command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum InfrastructureResourceOwnerStateV1 {
    /// Resource may be selected for a new eligible binding.
    Active,
    /// Resource remains known but must drain before retirement.
    Draining,
    /// Resource generation is terminal and its Master interval must close.
    Retired,
}

impl InfrastructureResourceOwnerStateV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Draining => "draining",
            Self::Retired => "retired",
        }
    }

    fn event_type(self) -> &'static str {
        if self == Self::Retired {
            "retired"
        } else {
            "state_changed"
        }
    }
}

/// Immutable state transition observed by the trusted Infrastructure Manager.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureManagerResourceLifecycleOperationV1 {
    /// Wire contract version.
    pub schema_version: u16,
    /// Idempotency identity for this single lifecycle transition.
    pub operation_id: Uuid,
    /// Immutable receipt identity assigned by the Manager.
    pub receipt_id: Uuid,
    /// Correlation identity shared with the initiating owner command.
    pub correlation_id: Uuid,
    /// Trusted actor identity recorded in the event.
    pub actor_id: Uuid,
    /// Infrastructure Manager identity associated with the signing key.
    pub manager_id: Uuid,
    /// Time the Manager observed the requested state.
    pub observed_at: DateTime<Utc>,
    /// Exact resource kind, identity and SCD2 generation being changed.
    pub resource_kind: InfrastructureResourceKindV1,
    /// Stable resource identity.
    pub resource_id: Uuid,
    /// Exact SCD2 generation.
    pub resource_generation: i64,
    /// Requested state; retired is terminal.
    pub target_state: InfrastructureResourceOwnerStateV1,
}

/// A signed, domain-separated Manager lifecycle command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedInfrastructureManagerLifecycleCommandV1 {
    /// Signed lifecycle operation body.
    pub operation: InfrastructureManagerResourceLifecycleOperationV1,
    /// Stable deployment-managed signing key identifier.
    pub manager_key_id: String,
    /// Ed25519 signature over `signing_message()`.
    pub signature: Vec<u8>,
}

impl SignedInfrastructureManagerLifecycleCommandV1 {
    /// Produce the versioned, domain-separated bytes a Manager must sign.
    pub fn signing_message(&self) -> Result<Vec<u8>, ResourceCatalogWriteError> {
        validate_lifecycle_operation(&self.operation)?;
        validate_key_id(&self.manager_key_id)?;
        let body = serde_json::to_vec(&self.operation)?;
        let mut message =
            Vec::with_capacity(LIFECYCLE_DOMAIN.len() + 2 + self.manager_key_id.len() + body.len());
        message.extend_from_slice(LIFECYCLE_DOMAIN);
        message.extend_from_slice(&(self.manager_key_id.len() as u16).to_be_bytes());
        message.extend_from_slice(self.manager_key_id.as_bytes());
        message.extend_from_slice(&body);
        Ok(message)
    }
}

/// A Manager operation plus its detached Ed25519 signature and key identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedInfrastructureManagerReceiptV1 {
    /// Signed operation body.
    pub operation: InfrastructureManagerResourceOperationV1,
    /// Stable deployment-managed signing key identifier.
    pub manager_key_id: String,
    /// Ed25519 signature over `signing_message()`.
    pub signature: Vec<u8>,
}

impl SignedInfrastructureManagerReceiptV1 {
    /// Produce the versioned, domain-separated bytes a Manager must sign.
    pub fn signing_message(&self) -> Result<Vec<u8>, ResourceCatalogWriteError> {
        validate_operation(&self.operation)?;
        validate_key_id(&self.manager_key_id)?;
        let body = serde_json::to_vec(&self.operation)?;
        let mut message =
            Vec::with_capacity(RECEIPT_DOMAIN.len() + 2 + self.manager_key_id.len() + body.len());
        message.extend_from_slice(RECEIPT_DOMAIN);
        message.extend_from_slice(&(self.manager_key_id.len() as u16).to_be_bytes());
        message.extend_from_slice(self.manager_key_id.as_bytes());
        message.extend_from_slice(&body);
        Ok(message)
    }
}

/// A configured Manager key that may authorize catalog registrations.
#[derive(Debug, Clone)]
pub struct TrustedInfrastructureManagerKey {
    /// Infrastructure Manager identity bound to this key.
    pub manager_id: Uuid,
    /// Deployment-managed key identifier.
    pub manager_key_id: String,
    /// Raw 32-byte Ed25519 public key.
    pub public_key: [u8; 32],
}

/// Verifies Manager signatures and derives the digest persisted with the receipt.
pub trait InfrastructureManagerReceiptVerifier: Send + Sync {
    /// Verify the exact operation/key tuple and return its signed-message SHA-256 digest.
    fn verify_and_digest(
        &self,
        receipt: &SignedInfrastructureManagerReceiptV1,
    ) -> Result<[u8; 32], ResourceCatalogWriteError>;
}

/// Verifies signed Manager lifecycle commands.
pub trait InfrastructureManagerLifecycleVerifier: Send + Sync {
    /// Verify the exact command/key tuple and return its signed-message SHA-256 digest.
    fn verify_lifecycle_and_digest(
        &self,
        command: &SignedInfrastructureManagerLifecycleCommandV1,
    ) -> Result<[u8; 32], ResourceCatalogWriteError>;
}

/// Ed25519 verifier over an explicit manager/key allow-list supplied by trusted configuration.
#[derive(Debug, Clone)]
pub struct Ed25519InfrastructureReceiptVerifier {
    keys: HashMap<(Uuid, String), [u8; 32]>,
}

impl Ed25519InfrastructureReceiptVerifier {
    /// Build a verifier from the deployment's trusted Manager key set.
    pub fn new(
        trusted_keys: impl IntoIterator<Item = TrustedInfrastructureManagerKey>,
    ) -> Result<Self, ResourceCatalogWriteError> {
        let mut keys = HashMap::with_capacity(MAX_TRUSTED_MANAGER_KEYS);
        for key in trusted_keys {
            if keys.len() == MAX_TRUSTED_MANAGER_KEYS {
                return Err(ResourceCatalogWriteError::TooManyTrustedKeys);
            }
            if key.manager_id.is_nil() {
                return Err(ResourceCatalogWriteError::InvalidReceipt);
            }
            validate_key_id(&key.manager_key_id)?;
            if keys
                .insert((key.manager_id, key.manager_key_id), key.public_key)
                .is_some()
            {
                return Err(ResourceCatalogWriteError::DuplicateTrustedKey);
            }
        }
        if keys.is_empty() {
            return Err(ResourceCatalogWriteError::EmptyTrustedKeyring);
        }
        Ok(Self { keys })
    }
}

impl InfrastructureManagerReceiptVerifier for Ed25519InfrastructureReceiptVerifier {
    fn verify_and_digest(
        &self,
        receipt: &SignedInfrastructureManagerReceiptV1,
    ) -> Result<[u8; 32], ResourceCatalogWriteError> {
        let message = receipt.signing_message()?;
        self.verify_message(
            receipt.operation.manager_id,
            &receipt.manager_key_id,
            &receipt.signature,
            &message,
        )
    }
}

impl InfrastructureManagerLifecycleVerifier for Ed25519InfrastructureReceiptVerifier {
    fn verify_lifecycle_and_digest(
        &self,
        command: &SignedInfrastructureManagerLifecycleCommandV1,
    ) -> Result<[u8; 32], ResourceCatalogWriteError> {
        let message = command.signing_message()?;
        self.verify_message(
            command.operation.manager_id,
            &command.manager_key_id,
            &command.signature,
            &message,
        )
    }
}

impl Ed25519InfrastructureReceiptVerifier {
    fn verify_message(
        &self,
        manager_id: Uuid,
        manager_key_id: &str,
        signature: &[u8],
        message: &[u8],
    ) -> Result<[u8; 32], ResourceCatalogWriteError> {
        if signature.len() != 64 {
            return Err(ResourceCatalogWriteError::InvalidSignature);
        }
        let public_key = self
            .keys
            .get(&(manager_id, manager_key_id.to_owned()))
            .ok_or(ResourceCatalogWriteError::UntrustedManagerKey)?;
        UnparsedPublicKey::new(&signature::ED25519, public_key)
            .verify(message, signature)
            .map_err(|_| ResourceCatalogWriteError::InvalidSignature)?;
        Ok(Sha256::digest(message).into())
    }
}

/// Stable outcome of one idempotent catalog registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCatalogRegistrationOutcome {
    /// Immutable receipt identity.
    pub receipt_id: Uuid,
    /// Idempotency identity shared with the source Manager operation.
    pub operation_id: Uuid,
    /// True when the same verified operation had already committed.
    pub replayed: bool,
}

/// Stable outcome of one idempotent signed resource lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCatalogLifecycleOutcome {
    /// Immutable receipt identity.
    pub receipt_id: Uuid,
    /// Idempotency identity shared with the source Manager operation.
    pub operation_id: Uuid,
    /// Exact resource identity whose state changed.
    pub resource_id: Uuid,
    /// Exact SCD2 generation whose state changed.
    pub resource_generation: i64,
    /// Persisted terminal/current owner state.
    pub owner_state: InfrastructureResourceOwnerStateV1,
    /// True when the same verified operation had already committed.
    pub replayed: bool,
}

/// PostgreSQL owner writer for signed Manager resource registrations.
pub struct PostgresInfrastructureResourceCatalogWriter<V> {
    pool: PgPool,
    verifier: V,
}

impl<V> PostgresInfrastructureResourceCatalogWriter<V>
where
    V: InfrastructureManagerReceiptVerifier,
{
    /// Create a writer using a pool whose application identity may assume the NOLOGIN owner role.
    pub fn new(pool: PgPool, verifier: V) -> Self {
        Self { pool, verifier }
    }

    /// Verify and atomically persist the signed receipt, one Master generation, and its event.
    pub async fn register(
        &self,
        receipt: &SignedInfrastructureManagerReceiptV1,
    ) -> Result<ResourceCatalogRegistrationOutcome, ResourceCatalogWriteError> {
        let digest = self.verifier.verify_and_digest(receipt)?;
        let (resource_kind, resource_id, generation) = receipt.operation.resource.identity();
        let mut tx = self.pool.begin().await?;
        sqlx::query(OWNER_ROLE).execute(&mut *tx).await?;
        let lock_key = format!(
            "star.infrastructure.resource-catalog:{}",
            receipt.operation.operation_id
        );
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(lock_key)
            .execute(&mut *tx)
            .await?;

        // The advisory lock is the idempotency fence; FOR UPDATE would require UPDATE on this WORM table.
        if let Some(row) = sqlx::query(
            "SELECT receipt_id, manager_id, manager_key_id, resource_kind, resource_id, resource_generation, correlation_id, actor_id, payload_digest, signature FROM infrastructure.resource_catalog_manager_receipt WHERE operation_id = $1",
        )
        .bind(receipt.operation.operation_id)
        .fetch_optional(&mut *tx)
        .await?
        {
            let matches = row.try_get::<Uuid, _>("receipt_id")? == receipt.operation.receipt_id
                && row.try_get::<Uuid, _>("manager_id")? == receipt.operation.manager_id
                && row.try_get::<String, _>("manager_key_id")? == receipt.manager_key_id
                && row.try_get::<String, _>("resource_kind")? == resource_kind
                && row.try_get::<Uuid, _>("resource_id")? == resource_id
                && row.try_get::<i64, _>("resource_generation")? == generation
                && row.try_get::<Uuid, _>("correlation_id")? == receipt.operation.correlation_id
                && row.try_get::<Uuid, _>("actor_id")? == receipt.operation.actor_id
                && row.try_get::<Vec<u8>, _>("payload_digest")? == digest
                && row.try_get::<Vec<u8>, _>("signature")? == receipt.signature;
            if !matches {
                return Err(ResourceCatalogWriteError::IdempotencyConflict);
            }
            tx.commit().await?;
            return Ok(ResourceCatalogRegistrationOutcome {
                receipt_id: receipt.operation.receipt_id,
                operation_id: receipt.operation.operation_id,
                replayed: true,
            });
        }

        sqlx::query(
            "INSERT INTO infrastructure.resource_catalog_manager_receipt (receipt_id, operation_id, schema_version, manager_id, manager_key_id, resource_kind, resource_id, resource_generation, event_type, owner_state, correlation_id, actor_id, payload_digest, signature, observed_at) VALUES ($1, $2, 1, $3, $4, $5, $6, $7, 'registered', 'active', $8, $9, $10, $11, $12)",
        )
        .bind(receipt.operation.receipt_id)
        .bind(receipt.operation.operation_id)
        .bind(receipt.operation.manager_id)
        .bind(&receipt.manager_key_id)
        .bind(resource_kind)
        .bind(resource_id)
        .bind(generation)
        .bind(receipt.operation.correlation_id)
        .bind(receipt.operation.actor_id)
        .bind(digest.as_slice())
        .bind(&receipt.signature)
        .bind(receipt.operation.observed_at)
        .execute(&mut *tx)
        .await?;

        insert_master(&mut tx, &receipt.operation).await?;
        record_registered_event(&mut tx, &receipt.operation, digest).await?;
        tx.commit().await?;
        Ok(ResourceCatalogRegistrationOutcome {
            receipt_id: receipt.operation.receipt_id,
            operation_id: receipt.operation.operation_id,
            replayed: false,
        })
    }
}

impl<V> PostgresInfrastructureResourceCatalogWriter<V>
where
    V: InfrastructureManagerLifecycleVerifier,
{
    /// Verify and atomically persist one signed state event and optional SCD2 retirement close.
    pub async fn transition(
        &self,
        command: &SignedInfrastructureManagerLifecycleCommandV1,
    ) -> Result<ResourceCatalogLifecycleOutcome, ResourceCatalogWriteError> {
        let digest = self.verifier.verify_lifecycle_and_digest(command)?;
        let operation = &command.operation;
        let resource_kind = operation.resource_kind.as_str();
        let event_type = operation.target_state.event_type();
        let owner_state = operation.target_state.as_str();
        let mut tx = self.pool.begin().await?;
        sqlx::query(OWNER_ROLE).execute(&mut *tx).await?;
        lock_resource_catalog_operation(&mut tx, operation.operation_id).await?;

        // Idempotency lookup precedes current-state validation so exact replays remain successful
        // after a resource has drained or been retired.
        if let Some(row) = sqlx::query(
            "SELECT receipt_id, manager_id, manager_key_id, resource_kind, resource_id, resource_generation, event_type, owner_state, correlation_id, actor_id, payload_digest, signature FROM infrastructure.resource_catalog_manager_receipt WHERE operation_id = $1",
        )
        .bind(operation.operation_id)
        .fetch_optional(&mut *tx)
        .await?
        {
            let matches = row.try_get::<Uuid, _>("receipt_id")? == operation.receipt_id
                && row.try_get::<Uuid, _>("manager_id")? == operation.manager_id
                && row.try_get::<String, _>("manager_key_id")? == command.manager_key_id
                && row.try_get::<String, _>("resource_kind")? == resource_kind
                && row.try_get::<Uuid, _>("resource_id")? == operation.resource_id
                && row.try_get::<i64, _>("resource_generation")? == operation.resource_generation
                && row.try_get::<String, _>("event_type")? == event_type
                && row.try_get::<String, _>("owner_state")? == owner_state
                && row.try_get::<Uuid, _>("correlation_id")? == operation.correlation_id
                && row.try_get::<Uuid, _>("actor_id")? == operation.actor_id
                && row.try_get::<Vec<u8>, _>("payload_digest")? == digest
                && row.try_get::<Vec<u8>, _>("signature")? == command.signature;
            if !matches {
                return Err(ResourceCatalogWriteError::IdempotencyConflict);
            }
            tx.commit().await?;
            return Ok(ResourceCatalogLifecycleOutcome {
                receipt_id: operation.receipt_id,
                operation_id: operation.operation_id,
                resource_id: operation.resource_id,
                resource_generation: operation.resource_generation,
                owner_state: operation.target_state,
                replayed: true,
            });
        }

        lock_resource_generation(
            &mut tx,
            resource_kind,
            operation.resource_id,
            operation.resource_generation,
        )
        .await?;
        let previous_state = current_resource_state(&mut tx, operation).await?;
        validate_state_transition(previous_state, operation.target_state)?;

        insert_lifecycle_receipt(&mut tx, command, digest, event_type, owner_state).await?;
        record_lifecycle_event(&mut tx, command, digest, event_type, owner_state).await?;
        if operation.target_state == InfrastructureResourceOwnerStateV1::Retired {
            close_resource_master(&mut tx, operation).await?;
        }
        tx.commit().await?;
        Ok(ResourceCatalogLifecycleOutcome {
            receipt_id: operation.receipt_id,
            operation_id: operation.operation_id,
            resource_id: operation.resource_id,
            resource_generation: operation.resource_generation,
            owner_state: operation.target_state,
            replayed: false,
        })
    }
}

async fn lock_resource_catalog_operation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation_id: Uuid,
) -> Result<(), ResourceCatalogWriteError> {
    let lock_key = format!("star.infrastructure.resource-catalog:{operation_id}");
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn lock_resource_generation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    resource_kind: &str,
    resource_id: Uuid,
    generation: i64,
) -> Result<(), ResourceCatalogWriteError> {
    let lock_key =
        format!("infrastructure.resource_catalog_event:{resource_kind}:{resource_id}:{generation}");
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn current_resource_state(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation: &InfrastructureManagerResourceLifecycleOperationV1,
) -> Result<InfrastructureResourceOwnerStateV1, ResourceCatalogWriteError> {
    let kind = operation.resource_kind.as_str();
    let (table, id_column) = operation.resource_kind.master_table_and_id_column();
    let current_query = format!(
        "SELECT event.owner_state FROM infrastructure.resource_catalog_event event JOIN infrastructure.{table} master ON master.{id_column} = event.resource_id AND master.generation = event.resource_generation WHERE event.resource_kind = $1 AND event.resource_id = $2 AND event.resource_generation = $3 AND master.valid_to IS NULL ORDER BY event.recorded_at DESC, event.event_id DESC LIMIT 1"
    );
    if let Some(state) = sqlx::query_scalar::<_, String>(&current_query)
        .bind(kind)
        .bind(operation.resource_id)
        .bind(operation.resource_generation)
        .fetch_optional(&mut **tx)
        .await?
    {
        return parse_owner_state(&state);
    }

    let latest_query = "SELECT owner_state FROM infrastructure.resource_catalog_event WHERE resource_kind = $1 AND resource_id = $2 AND resource_generation = $3 ORDER BY recorded_at DESC, event_id DESC LIMIT 1";
    let latest = sqlx::query_scalar::<_, String>(latest_query)
        .bind(kind)
        .bind(operation.resource_id)
        .bind(operation.resource_generation)
        .fetch_optional(&mut **tx)
        .await?;
    if latest.as_deref() == Some("retired") {
        Err(ResourceCatalogWriteError::ResourceAlreadyRetired)
    } else {
        Err(ResourceCatalogWriteError::ResourceNotCurrent)
    }
}

fn parse_owner_state(
    state: &str,
) -> Result<InfrastructureResourceOwnerStateV1, ResourceCatalogWriteError> {
    match state {
        "active" => Ok(InfrastructureResourceOwnerStateV1::Active),
        "draining" => Ok(InfrastructureResourceOwnerStateV1::Draining),
        "retired" => Ok(InfrastructureResourceOwnerStateV1::Retired),
        _ => Err(ResourceCatalogWriteError::InvalidCatalogState),
    }
}

fn validate_state_transition(
    previous: InfrastructureResourceOwnerStateV1,
    target: InfrastructureResourceOwnerStateV1,
) -> Result<(), ResourceCatalogWriteError> {
    let valid = matches!(
        (previous, target),
        (
            InfrastructureResourceOwnerStateV1::Active,
            InfrastructureResourceOwnerStateV1::Draining
        ) | (
            InfrastructureResourceOwnerStateV1::Draining,
            InfrastructureResourceOwnerStateV1::Active
        ) | (
            InfrastructureResourceOwnerStateV1::Active,
            InfrastructureResourceOwnerStateV1::Retired
        ) | (
            InfrastructureResourceOwnerStateV1::Draining,
            InfrastructureResourceOwnerStateV1::Retired
        )
    );
    if valid {
        Ok(())
    } else if previous == InfrastructureResourceOwnerStateV1::Retired {
        Err(ResourceCatalogWriteError::ResourceAlreadyRetired)
    } else {
        Err(ResourceCatalogWriteError::InvalidStateTransition)
    }
}

async fn insert_lifecycle_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &SignedInfrastructureManagerLifecycleCommandV1,
    digest: [u8; 32],
    event_type: &str,
    owner_state: &str,
) -> Result<(), ResourceCatalogWriteError> {
    let operation = &command.operation;
    sqlx::query(
        "INSERT INTO infrastructure.resource_catalog_manager_receipt (receipt_id, operation_id, schema_version, manager_id, manager_key_id, resource_kind, resource_id, resource_generation, event_type, owner_state, correlation_id, actor_id, payload_digest, signature, observed_at) VALUES ($1, $2, 1, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(operation.receipt_id)
    .bind(operation.operation_id)
    .bind(operation.manager_id)
    .bind(&command.manager_key_id)
    .bind(operation.resource_kind.as_str())
    .bind(operation.resource_id)
    .bind(operation.resource_generation)
    .bind(event_type)
    .bind(owner_state)
    .bind(operation.correlation_id)
    .bind(operation.actor_id)
    .bind(digest.as_slice())
    .bind(&command.signature)
    .bind(operation.observed_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn record_lifecycle_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &SignedInfrastructureManagerLifecycleCommandV1,
    digest: [u8; 32],
    event_type: &str,
    owner_state: &str,
) -> Result<(), ResourceCatalogWriteError> {
    let operation = &command.operation;
    sqlx::query(
        "INSERT INTO infrastructure.resource_catalog_event (resource_kind, resource_id, resource_generation, event_type, owner_state, source_command_id, correlation_id, actor_id, details_digest, manager_receipt_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(operation.resource_kind.as_str())
    .bind(operation.resource_id)
    .bind(operation.resource_generation)
    .bind(event_type)
    .bind(owner_state)
    .bind(operation.operation_id)
    .bind(operation.correlation_id)
    .bind(operation.actor_id)
    .bind(digest.as_slice())
    .bind(operation.receipt_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn close_resource_master(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation: &InfrastructureManagerResourceLifecycleOperationV1,
) -> Result<(), ResourceCatalogWriteError> {
    let (table, id_column) = operation.resource_kind.master_table_and_id_column();
    let query = format!(
        "UPDATE infrastructure.{table} SET valid_to = clock_timestamp() WHERE {id_column} = $1 AND generation = $2 AND valid_to IS NULL"
    );
    let result = sqlx::query(&query)
        .bind(operation.resource_id)
        .bind(operation.resource_generation)
        .execute(&mut **tx)
        .await?;
    if result.rows_affected() != 1 {
        return Err(ResourceCatalogWriteError::ResourceNotCurrent);
    }
    Ok(())
}

async fn insert_master(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation: &InfrastructureManagerResourceOperationV1,
) -> Result<(), ResourceCatalogWriteError> {
    let query = match &operation.resource {
        ManagerResourceIdentityV1::HostEnvironment {
            host_environment_id,
            generation,
            host_os,
            host_arch,
        } => sqlx::query(
            "INSERT INTO infrastructure.host_environment_catalog (host_environment_id, generation, provider_kind, host_os, host_arch, actor_id) VALUES ($1, $2, 'multipass_local', $3, $4, $5)",
        )
        .bind(host_environment_id)
        .bind(generation)
        .bind(host_os)
        .bind(host_arch)
        .bind(operation.actor_id),
        ManagerResourceIdentityV1::MultipassInstance {
            multipass_instance_id,
            generation,
            host_environment_id,
            host_generation,
            provider_instance_name,
        } => sqlx::query(
            "INSERT INTO infrastructure.multipass_instance_catalog (multipass_instance_id, generation, host_environment_id, host_generation, provider_instance_name, actor_id) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(multipass_instance_id)
        .bind(generation)
        .bind(host_environment_id)
        .bind(host_generation)
        .bind(provider_instance_name)
        .bind(operation.actor_id),
        ManagerResourceIdentityV1::K3sCluster {
            k3s_cluster_id,
            generation,
            host_environment_id,
            host_generation,
            cluster_identity_digest,
            k3s_version,
        } => sqlx::query(
            "INSERT INTO infrastructure.k3s_cluster_catalog (k3s_cluster_id, generation, host_environment_id, host_generation, provider_kind, cluster_identity_digest, k3s_version, actor_id) VALUES ($1, $2, $3, $4, 'multipass_local', $5, $6, $7)",
        )
        .bind(k3s_cluster_id)
        .bind(generation)
        .bind(host_environment_id)
        .bind(host_generation)
        .bind(cluster_identity_digest.as_slice())
        .bind(k3s_version)
        .bind(operation.actor_id),
        ManagerResourceIdentityV1::ClusterInstanceMembership {
            membership_id,
            generation,
            k3s_cluster_id,
            k3s_cluster_generation,
            multipass_instance_id,
            multipass_instance_generation,
            host_environment_id,
            host_generation,
            node_role,
        } => sqlx::query(
            "INSERT INTO infrastructure.k3s_cluster_instance_membership (membership_id, generation, k3s_cluster_id, k3s_cluster_generation, multipass_instance_id, multipass_instance_generation, host_environment_id, host_generation, node_role, actor_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(membership_id)
        .bind(generation)
        .bind(k3s_cluster_id)
        .bind(k3s_cluster_generation)
        .bind(multipass_instance_id)
        .bind(multipass_instance_generation)
        .bind(host_environment_id)
        .bind(host_generation)
        .bind(node_role.as_str())
        .bind(operation.actor_id),
        ManagerResourceIdentityV1::KubernetesNode {
            kubernetes_node_id,
            generation,
            membership_id,
            membership_generation,
            k3s_cluster_id,
            k3s_cluster_generation,
            multipass_instance_id,
            multipass_instance_generation,
            host_environment_id,
            host_generation,
            kubernetes_node_uid,
            kubernetes_node_name,
            node_role,
        } => sqlx::query(
            "INSERT INTO infrastructure.kubernetes_node_catalog (kubernetes_node_id, generation, membership_id, membership_generation, k3s_cluster_id, k3s_cluster_generation, multipass_instance_id, multipass_instance_generation, host_environment_id, host_generation, kubernetes_node_uid, kubernetes_node_name, node_role, actor_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(kubernetes_node_id)
        .bind(generation)
        .bind(membership_id)
        .bind(membership_generation)
        .bind(k3s_cluster_id)
        .bind(k3s_cluster_generation)
        .bind(multipass_instance_id)
        .bind(multipass_instance_generation)
        .bind(host_environment_id)
        .bind(host_generation)
        .bind(kubernetes_node_uid)
        .bind(kubernetes_node_name)
        .bind(node_role.as_str())
        .bind(operation.actor_id),
    };
    query.execute(&mut **tx).await?;
    Ok(())
}

async fn record_registered_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    operation: &InfrastructureManagerResourceOperationV1,
    digest: [u8; 32],
) -> Result<(), ResourceCatalogWriteError> {
    let (resource_kind, resource_id, generation) = operation.resource.identity();
    sqlx::query(
        "INSERT INTO infrastructure.resource_catalog_event (resource_kind, resource_id, resource_generation, event_type, owner_state, source_command_id, correlation_id, actor_id, details_digest, manager_receipt_id) VALUES ($1, $2, $3, 'registered', 'active', $4, $5, $6, $7, $8)",
    )
    .bind(resource_kind)
    .bind(resource_id)
    .bind(generation)
    .bind(operation.operation_id)
    .bind(operation.correlation_id)
    .bind(operation.actor_id)
    .bind(digest.as_slice())
    .bind(operation.receipt_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

impl ManagerResourceIdentityV1 {
    fn identity(&self) -> (&'static str, Uuid, i64) {
        match self {
            Self::HostEnvironment {
                host_environment_id,
                generation,
                ..
            } => ("host_environment", *host_environment_id, *generation),
            Self::MultipassInstance {
                multipass_instance_id,
                generation,
                ..
            } => ("multipass_instance", *multipass_instance_id, *generation),
            Self::K3sCluster {
                k3s_cluster_id,
                generation,
                ..
            } => ("k3s_cluster", *k3s_cluster_id, *generation),
            Self::ClusterInstanceMembership {
                membership_id,
                generation,
                ..
            } => ("cluster_instance_membership", *membership_id, *generation),
            Self::KubernetesNode {
                kubernetes_node_id,
                generation,
                ..
            } => ("kubernetes_node", *kubernetes_node_id, *generation),
        }
    }
}

impl K3sNodeRoleV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Agent => "agent",
        }
    }
}

fn validate_key_id(key_id: &str) -> Result<(), ResourceCatalogWriteError> {
    if key_id.is_empty()
        || key_id.len() > MAX_KEY_ID_BYTES
        || !key_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(ResourceCatalogWriteError::InvalidReceipt);
    }
    Ok(())
}

fn validate_operation(
    operation: &InfrastructureManagerResourceOperationV1,
) -> Result<(), ResourceCatalogWriteError> {
    if operation.schema_version != 1
        || operation.operation_id.is_nil()
        || operation.receipt_id.is_nil()
        || operation.correlation_id.is_nil()
        || operation.actor_id.is_nil()
        || operation.manager_id.is_nil()
    {
        return Err(ResourceCatalogWriteError::InvalidReceipt);
    }
    let (_, resource_id, generation) = operation.resource.identity();
    if resource_id.is_nil() || generation <= 0 {
        return Err(ResourceCatalogWriteError::InvalidReceipt);
    }
    let valid = match &operation.resource {
        ManagerResourceIdentityV1::HostEnvironment {
            host_os, host_arch, ..
        } => {
            matches!(host_os.as_str(), "windows" | "linux" | "macos")
                && !host_arch.is_empty()
                && host_arch.len() <= 32
                && host_arch
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        }
        ManagerResourceIdentityV1::MultipassInstance {
            host_environment_id,
            host_generation,
            provider_instance_name,
            ..
        } => {
            !host_environment_id.is_nil()
                && *host_generation > 0
                && !provider_instance_name.is_empty()
                && provider_instance_name.len() <= 63
                && (provider_instance_name.as_bytes()[0].is_ascii_lowercase()
                    || provider_instance_name.as_bytes()[0].is_ascii_digit())
                && provider_instance_name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !provider_instance_name.ends_with('-')
        }
        ManagerResourceIdentityV1::K3sCluster {
            host_environment_id,
            host_generation,
            k3s_version,
            ..
        } => {
            !host_environment_id.is_nil()
                && *host_generation > 0
                && !k3s_version.is_empty()
                && k3s_version.len() <= 64
                && k3s_version
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".+_-".contains(&b))
        }
        ManagerResourceIdentityV1::ClusterInstanceMembership {
            k3s_cluster_id,
            k3s_cluster_generation,
            multipass_instance_id,
            multipass_instance_generation,
            host_environment_id,
            host_generation,
            ..
        } => {
            !k3s_cluster_id.is_nil()
                && *k3s_cluster_generation > 0
                && !multipass_instance_id.is_nil()
                && *multipass_instance_generation > 0
                && !host_environment_id.is_nil()
                && *host_generation > 0
        }
        ManagerResourceIdentityV1::KubernetesNode {
            membership_id,
            membership_generation,
            k3s_cluster_id,
            k3s_cluster_generation,
            multipass_instance_id,
            multipass_instance_generation,
            host_environment_id,
            host_generation,
            kubernetes_node_uid,
            kubernetes_node_name,
            ..
        } => {
            !membership_id.is_nil()
                && *membership_generation > 0
                && !k3s_cluster_id.is_nil()
                && *k3s_cluster_generation > 0
                && !multipass_instance_id.is_nil()
                && *multipass_instance_generation > 0
                && !host_environment_id.is_nil()
                && *host_generation > 0
                && !kubernetes_node_uid.is_empty()
                && kubernetes_node_uid.len() <= 128
                && kubernetes_node_uid
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
                && valid_dns_name(kubernetes_node_name)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(ResourceCatalogWriteError::InvalidReceipt)
    }
}

fn validate_lifecycle_operation(
    operation: &InfrastructureManagerResourceLifecycleOperationV1,
) -> Result<(), ResourceCatalogWriteError> {
    if operation.schema_version != 1
        || operation.operation_id.is_nil()
        || operation.receipt_id.is_nil()
        || operation.correlation_id.is_nil()
        || operation.actor_id.is_nil()
        || operation.manager_id.is_nil()
        || operation.resource_id.is_nil()
        || operation.resource_generation <= 0
    {
        return Err(ResourceCatalogWriteError::InvalidReceipt);
    }
    Ok(())
}

fn valid_dns_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && (label.as_bytes()[0].is_ascii_lowercase()
                    || label.as_bytes()[0].is_ascii_digit())
                && (label.as_bytes()[label.len() - 1].is_ascii_lowercase()
                    || label.as_bytes()[label.len() - 1].is_ascii_digit())
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// Fail-closed errors from Manager receipt verification and catalog persistence.
#[derive(Debug, Error)]
pub enum ResourceCatalogWriteError {
    /// A signed operation is outside the bounded versioned contract.
    #[error("Infrastructure Manager resource receipt is invalid")]
    InvalidReceipt,
    /// The configured Manager key set contains no trusted entries.
    #[error("Infrastructure Manager receipt keyring is empty")]
    EmptyTrustedKeyring,
    /// The configured Manager identity/key-id tuple is duplicated.
    #[error("Infrastructure Manager receipt key is duplicated")]
    DuplicateTrustedKey,
    /// The deployment keyring exceeds the fixed in-memory lookup bound.
    #[error("Infrastructure Manager receipt keyring exceeds 64 entries")]
    TooManyTrustedKeys,
    /// No trusted public key matches the Manager and key identifier in the receipt.
    #[error("Infrastructure Manager signing key is not trusted")]
    UntrustedManagerKey,
    /// The detached Ed25519 signature is malformed or does not verify.
    #[error("Infrastructure Manager receipt signature is invalid")]
    InvalidSignature,
    /// A prior operation ID exists with different immutable receipt facts.
    #[error("Infrastructure Manager operation idempotency key conflicts")]
    IdempotencyConflict,
    /// The requested exact resource generation is not current in the owner catalog.
    #[error("Infrastructure resource generation is not current")]
    ResourceNotCurrent,
    /// The requested generation has already reached its terminal retired state.
    #[error("Infrastructure resource generation is already retired")]
    ResourceAlreadyRetired,
    /// The requested active/draining transition is not allowed from the current state.
    #[error("Infrastructure resource state transition is invalid")]
    InvalidStateTransition,
    /// The persisted catalog contains an unsupported owner-state value.
    #[error("Infrastructure resource catalog state is invalid")]
    InvalidCatalogState,
    /// Serialization failed while building the signed message.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// SQLx/PostgreSQL rejected the owner transaction or an exact-generation reference.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::{
        validate_state_transition, Ed25519InfrastructureReceiptVerifier,
        InfrastructureManagerLifecycleVerifier, InfrastructureManagerReceiptVerifier,
        InfrastructureManagerResourceLifecycleOperationV1,
        InfrastructureManagerResourceOperationV1, InfrastructureResourceKindV1,
        InfrastructureResourceOwnerStateV1, ManagerResourceIdentityV1,
        SignedInfrastructureManagerLifecycleCommandV1, SignedInfrastructureManagerReceiptV1,
        TrustedInfrastructureManagerKey,
    };
    use chrono::{TimeZone, Utc};
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use uuid::Uuid;

    fn operation() -> InfrastructureManagerResourceOperationV1 {
        InfrastructureManagerResourceOperationV1 {
            schema_version: 1,
            operation_id: Uuid::from_u128(1),
            receipt_id: Uuid::from_u128(2),
            correlation_id: Uuid::from_u128(3),
            actor_id: Uuid::from_u128(4),
            manager_id: Uuid::from_u128(5),
            observed_at: Utc.timestamp_opt(1_791_300_000, 0).unwrap(),
            resource: ManagerResourceIdentityV1::HostEnvironment {
                host_environment_id: Uuid::from_u128(6),
                generation: 1,
                host_os: "windows".to_owned(),
                host_arch: "x86_64".to_owned(),
            },
        }
    }

    fn signed_receipt(seed: &[u8; 32]) -> (SignedInfrastructureManagerReceiptV1, Ed25519KeyPair) {
        let keypair = Ed25519KeyPair::from_seed_unchecked(seed).unwrap();
        let mut receipt = SignedInfrastructureManagerReceiptV1 {
            operation: operation(),
            manager_key_id: "test-key-1".to_owned(),
            signature: Vec::new(),
        };
        receipt.signature = keypair
            .sign(&receipt.signing_message().unwrap())
            .as_ref()
            .to_vec();
        (receipt, keypair)
    }

    fn signed_lifecycle(
        seed: &[u8; 32],
    ) -> (
        SignedInfrastructureManagerLifecycleCommandV1,
        Ed25519KeyPair,
    ) {
        let keypair = Ed25519KeyPair::from_seed_unchecked(seed).unwrap();
        let operation = InfrastructureManagerResourceLifecycleOperationV1 {
            schema_version: 1,
            operation_id: Uuid::from_u128(101),
            receipt_id: Uuid::from_u128(102),
            correlation_id: Uuid::from_u128(103),
            actor_id: Uuid::from_u128(104),
            manager_id: Uuid::from_u128(105),
            observed_at: Utc.timestamp_opt(1_791_300_000, 0).unwrap(),
            resource_kind: InfrastructureResourceKindV1::HostEnvironment,
            resource_id: Uuid::from_u128(106),
            resource_generation: 1,
            target_state: InfrastructureResourceOwnerStateV1::Draining,
        };
        let mut command = SignedInfrastructureManagerLifecycleCommandV1 {
            operation,
            manager_key_id: "test-key-1".to_owned(),
            signature: Vec::new(),
        };
        command.signature = keypair
            .sign(&command.signing_message().unwrap())
            .as_ref()
            .to_vec();
        (command, keypair)
    }

    #[test]
    fn verifies_signature_and_returns_stable_signed_payload_digest() {
        let seed = [7_u8; 32];
        let (receipt, keypair) = signed_receipt(&seed);
        let verifier =
            Ed25519InfrastructureReceiptVerifier::new([TrustedInfrastructureManagerKey {
                manager_id: receipt.operation.manager_id,
                manager_key_id: receipt.manager_key_id.clone(),
                public_key: keypair.public_key().as_ref().try_into().unwrap(),
            }])
            .unwrap();
        let first = verifier.verify_and_digest(&receipt).unwrap();
        let second = verifier.verify_and_digest(&receipt).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn rejects_tampered_facts_and_untrusted_manager_keys() {
        let seed = [9_u8; 32];
        let (receipt, keypair) = signed_receipt(&seed);
        let verifier =
            Ed25519InfrastructureReceiptVerifier::new([TrustedInfrastructureManagerKey {
                manager_id: receipt.operation.manager_id,
                manager_key_id: receipt.manager_key_id.clone(),
                public_key: keypair.public_key().as_ref().try_into().unwrap(),
            }])
            .unwrap();
        let mut tampered = receipt.clone();
        tampered.operation.actor_id = Uuid::from_u128(42);
        assert!(verifier.verify_and_digest(&tampered).is_err());

        let other_manager =
            Ed25519InfrastructureReceiptVerifier::new([TrustedInfrastructureManagerKey {
                manager_id: Uuid::from_u128(99),
                manager_key_id: receipt.manager_key_id.clone(),
                public_key: keypair.public_key().as_ref().try_into().unwrap(),
            }])
            .unwrap();
        assert!(other_manager.verify_and_digest(&receipt).is_err());
    }

    #[test]
    fn rejects_bad_signature_length_and_invalid_resource_identity() {
        let seed = [11_u8; 32];
        let (mut receipt, keypair) = signed_receipt(&seed);
        let verifier =
            Ed25519InfrastructureReceiptVerifier::new([TrustedInfrastructureManagerKey {
                manager_id: receipt.operation.manager_id,
                manager_key_id: receipt.manager_key_id.clone(),
                public_key: keypair.public_key().as_ref().try_into().unwrap(),
            }])
            .unwrap();
        receipt.signature.pop();
        assert!(verifier.verify_and_digest(&receipt).is_err());
        receipt.signature = keypair
            .sign(&receipt.signing_message().unwrap())
            .as_ref()
            .to_vec();
        receipt.operation.resource = ManagerResourceIdentityV1::MultipassInstance {
            multipass_instance_id: Uuid::from_u128(7),
            generation: 1,
            host_environment_id: Uuid::nil(),
            host_generation: 1,
            provider_instance_name: "star-test-01".to_owned(),
        };
        assert!(verifier.verify_and_digest(&receipt).is_err());
    }

    #[test]
    fn limits_trusted_keyring_size() {
        let keys = (0..=64).map(|index| TrustedInfrastructureManagerKey {
            manager_id: Uuid::from_u128(index + 1),
            manager_key_id: format!("key-{index}"),
            public_key: [index as u8; 32],
        });
        assert!(matches!(
            Ed25519InfrastructureReceiptVerifier::new(keys),
            Err(super::ResourceCatalogWriteError::TooManyTrustedKeys)
        ));
    }

    #[test]
    fn verifies_lifecycle_signature_with_a_distinct_domain_and_rejects_tampering() {
        let seed = [13_u8; 32];
        let (command, keypair) = signed_lifecycle(&seed);
        let verifier =
            Ed25519InfrastructureReceiptVerifier::new([TrustedInfrastructureManagerKey {
                manager_id: command.operation.manager_id,
                manager_key_id: command.manager_key_id.clone(),
                public_key: keypair.public_key().as_ref().try_into().unwrap(),
            }])
            .unwrap();
        let first = verifier.verify_lifecycle_and_digest(&command).unwrap();
        assert_eq!(
            first,
            verifier.verify_lifecycle_and_digest(&command).unwrap()
        );

        let mut tampered = command;
        tampered.operation.target_state = InfrastructureResourceOwnerStateV1::Retired;
        assert!(verifier.verify_lifecycle_and_digest(&tampered).is_err());
    }

    #[test]
    fn lifecycle_state_machine_allows_only_drain_reactivation_and_retirement() {
        use InfrastructureResourceOwnerStateV1::{Active, Draining, Retired};

        for (previous, target) in [
            (Active, Draining),
            (Draining, Active),
            (Active, Retired),
            (Draining, Retired),
        ] {
            assert!(validate_state_transition(previous, target).is_ok());
        }
        for (previous, target) in [(Active, Active), (Draining, Draining)] {
            assert!(matches!(
                validate_state_transition(previous, target),
                Err(super::ResourceCatalogWriteError::InvalidStateTransition)
            ));
        }
        assert!(matches!(
            validate_state_transition(Retired, Active),
            Err(super::ResourceCatalogWriteError::ResourceAlreadyRetired)
        ));
    }
}
