//! Validated, serializable identity for a resolved KMS provider profile.
//!
//! The profile digest is provided by a trusted resolver. This module does not
//! claim to canonicalize or hash profile data.

/*
@cypher schema=1 source_sha256=97ad34af6cc277f80f57c8a0d0d8479515c1fe54949082248054fbbf46495622
MERGE (self:File {path:"crates/domain-kms/src/profile.rs"})
MERGE (descriptor:Symbol {id:"crates/domain-kms/src/profile.rs::KmsProviderDescriptor",kind:"type"})
MERGE (identity:Symbol {id:"crates/domain-kms/src/profile.rs::KmsProfileIdentity",kind:"type"})
MERGE (digest:Symbol {id:"crates/domain-kms/src/profile.rs::ProfileDigest",kind:"type"})
MERGE (error:Symbol {id:"crates/domain-kms/src/profile.rs::KmsProfileError",kind:"enum"})
MERGE (provider_id_check:Symbol {id:"crates/domain-kms/src/profile.rs::valid_provider_id",kind:"function"})
MERGE (version_check:Symbol {id:"crates/domain-kms/src/profile.rs::valid_version",kind:"function"})
MERGE (reference_check:Symbol {id:"crates/domain-kms/src/profile.rs::valid_reference",kind:"function"})
MERGE (capabilities:Symbol {id:"crates/domain-kms/src/profile.rs::KmsCapabilities",kind:"type"})
MERGE (self)-[:DEFINES]->(descriptor)
MERGE (self)-[:DEFINES]->(identity)
MERGE (self)-[:DEFINES]->(digest)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(provider_id_check)
MERGE (self)-[:DEFINES]->(version_check)
MERGE (self)-[:DEFINES]->(reference_check)
MERGE (self)-[:DEFINES]->(capabilities)
MERGE (descriptor)-[:CALLS]->(provider_id_check)
MERGE (descriptor)-[:CALLS]->(version_check)
MERGE (identity)-[:CALLS]->(reference_check)
MERGE (identity)-[:USES_TYPE]->(descriptor)
MERGE (identity)-[:USES_TYPE]->(capabilities)
MERGE (identity)-[:USES_TYPE]->(digest)
@endcypher
*/

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// A stable provider identifier and implementation version.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "KmsProviderDescriptorWire")]
pub struct KmsProviderDescriptor {
    provider_id: String,
    provider_version: String,
}

impl KmsProviderDescriptor {
    /// Creates a descriptor after validating its non-secret identifiers.
    pub fn new(
        provider_id: impl Into<String>,
        provider_version: impl Into<String>,
    ) -> Result<Self, KmsProfileError> {
        let provider_id = provider_id.into();
        let provider_version = provider_version.into();
        if !valid_provider_id(&provider_id) || !valid_version(&provider_version) {
            return Err(KmsProfileError::InvalidProviderDescriptor);
        }
        Ok(Self {
            provider_id,
            provider_version,
        })
    }

    /// Returns the stable provider identifier.
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Returns the provider implementation version.
    pub fn provider_version(&self) -> &str {
        &self.provider_version
    }
}

impl std::fmt::Debug for KmsProviderDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KmsProviderDescriptor")
            .field("provider_id", &self.provider_id)
            .field("provider_version", &self.provider_version)
            .finish()
    }
}

#[derive(Deserialize)]
struct KmsProviderDescriptorWire {
    provider_id: String,
    provider_version: String,
}

impl TryFrom<KmsProviderDescriptorWire> for KmsProviderDescriptor {
    type Error = KmsProfileError;

    fn try_from(wire: KmsProviderDescriptorWire) -> Result<Self, Self::Error> {
        Self::new(wire.provider_id, wire.provider_version)
    }
}

/// KMS operations the resolved profile explicitly permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KmsCapability {
    /// Generate a plaintext data key and its provider-wrapped form.
    GenerateDataKey,
    /// Wrap a caller-supplied data key.
    WrapDataKey,
    /// Unwrap a previously wrapped data key.
    UnwrapDataKey,
    /// Rewrap a data key under another key reference within the same provider profile lineage.
    RewrapDataKey,
}

/// Deterministically ordered set of KMS operations allowed by a profile.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KmsCapabilities(BTreeSet<KmsCapability>);

impl KmsCapabilities {
    /// Creates a capability set.
    pub fn new(capabilities: impl IntoIterator<Item = KmsCapability>) -> Self {
        Self(capabilities.into_iter().collect())
    }

    /// Returns whether the set contains a capability.
    pub fn contains(&self, capability: KmsCapability) -> bool {
        self.0.contains(&capability)
    }

    /// Returns the number of capabilities in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the capability set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the capabilities in stable sorted order.
    pub fn iter(&self) -> impl Iterator<Item = KmsCapability> + '_ {
        self.0.iter().copied()
    }
}

/// A fixed-size digest supplied by a trusted profile-resolution boundary.
///
/// This crate deliberately does not compute the digest: canonical serialization
/// and digest generation remain an open integration requirement.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ProfileDigest([u8; 32]);

impl ProfileDigest {
    /// Accepts a digest produced by the trusted resolver.
    ///
    /// An all-zero value is rejected as an unresolved/sentinel digest.
    pub fn from_trusted_resolver(bytes: [u8; 32]) -> Result<Self, KmsProfileError> {
        if bytes.iter().all(|byte| *byte == 0) {
            return Err(KmsProfileError::UnresolvedDigest);
        }
        Ok(Self(bytes))
    }

    /// Returns the fixed-size digest bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for ProfileDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProfileDigest([redacted])")
    }
}

impl<'de> Deserialize<'de> for ProfileDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <[u8; 32]>::deserialize(deserializer)?;
        Self::from_trusted_resolver(bytes).map_err(D::Error::custom)
    }
}

/// Immutable provider and key-reference identity pinned to an envelope.
///
/// Endpoint and key references use a restricted opaque-reference grammar. Do not
/// place URLs with credentials, bearer tokens, or arbitrary secret material here.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "KmsProfileIdentityWire")]
pub struct KmsProfileIdentity {
    provider: KmsProviderDescriptor,
    revision: u64,
    endpoint_ref: String,
    key_ref: String,
    capabilities: KmsCapabilities,
    digest: ProfileDigest,
}

impl KmsProfileIdentity {
    /// Constructs a pinned profile identity from trusted resolution results.
    pub fn new(
        provider: KmsProviderDescriptor,
        revision: u64,
        endpoint_ref: impl Into<String>,
        key_ref: impl Into<String>,
        capabilities: KmsCapabilities,
        digest: ProfileDigest,
    ) -> Result<Self, KmsProfileError> {
        let endpoint_ref = endpoint_ref.into();
        let key_ref = key_ref.into();
        if revision == 0 {
            return Err(KmsProfileError::InvalidRevision);
        }
        if !valid_reference(&endpoint_ref) || !valid_reference(&key_ref) {
            return Err(KmsProfileError::InvalidReference);
        }
        if capabilities.is_empty() {
            return Err(KmsProfileError::EmptyCapabilities);
        }
        Ok(Self {
            provider,
            revision,
            endpoint_ref,
            key_ref,
            capabilities,
            digest,
        })
    }

    /// Returns the provider identity.
    pub fn provider(&self) -> &KmsProviderDescriptor {
        &self.provider
    }

    /// Returns the monotonically assigned resolver revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the opaque, non-secret endpoint reference.
    pub fn endpoint_ref(&self) -> &str {
        &self.endpoint_ref
    }

    /// Returns the opaque key reference.
    pub fn key_ref(&self) -> &str {
        &self.key_ref
    }

    /// Returns the profile capability snapshot.
    pub fn capabilities(&self) -> &KmsCapabilities {
        &self.capabilities
    }

    /// Returns the digest supplied by the trusted resolver.
    pub fn digest(&self) -> ProfileDigest {
        self.digest
    }

    /// Returns whether this profile permits the requested operation.
    pub fn supports(&self, capability: KmsCapability) -> bool {
        self.capabilities.contains(capability)
    }
}

impl std::fmt::Debug for KmsProfileIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KmsProfileIdentity")
            .field("provider", &self.provider)
            .field("revision", &self.revision)
            .field("endpoint_ref", &"[redacted]")
            .field("key_ref", &"[redacted]")
            .field("capabilities", &self.capabilities)
            .field("digest", &self.digest)
            .finish()
    }
}

#[derive(Deserialize)]
struct KmsProfileIdentityWire {
    provider: KmsProviderDescriptor,
    revision: u64,
    endpoint_ref: String,
    key_ref: String,
    capabilities: KmsCapabilities,
    digest: ProfileDigest,
}

impl TryFrom<KmsProfileIdentityWire> for KmsProfileIdentity {
    type Error = KmsProfileError;

    fn try_from(wire: KmsProfileIdentityWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.provider,
            wire.revision,
            wire.endpoint_ref,
            wire.key_ref,
            wire.capabilities,
            wire.digest,
        )
    }
}

/// Validation errors for provider and profile identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum KmsProfileError {
    /// Provider id or version did not match the restricted identifier grammar.
    #[error("invalid provider descriptor")]
    InvalidProviderDescriptor,
    /// Profile revision must be nonzero.
    #[error("invalid profile revision")]
    InvalidRevision,
    /// Endpoint or key reference did not match the opaque-reference grammar.
    #[error("invalid profile reference")]
    InvalidReference,
    /// An unresolved profile must not claim an all-zero digest.
    #[error("profile digest is unresolved")]
    UnresolvedDigest,
    /// A profile must pin at least one allowed capability.
    #[error("profile has no capabilities")]
    EmptyCapabilities,
}

fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
}

fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

fn valid_reference(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains("//")
        && !value
            .split('/')
            .any(|segment| segment == "." || segment == "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> ProfileDigest {
        ProfileDigest::from_trusted_resolver([7; 32]).unwrap()
    }

    fn profile() -> KmsProfileIdentity {
        KmsProfileIdentity::new(
            KmsProviderDescriptor::new("provider-a", "1.2.3").unwrap(),
            4,
            "kms-prod",
            "team-a/key-a",
            KmsCapabilities::new([KmsCapability::WrapDataKey, KmsCapability::UnwrapDataKey]),
            digest(),
        )
        .unwrap()
    }

    #[test]
    fn rejects_zero_digest_and_unsafe_references() {
        assert_eq!(
            ProfileDigest::from_trusted_resolver([0; 32]),
            Err(KmsProfileError::UnresolvedDigest)
        );
        assert_eq!(
            KmsProfileIdentity::new(
                KmsProviderDescriptor::new("provider-a", "1.2.3").unwrap(),
                1,
                "kms-prod?token=secret",
                "key-a",
                KmsCapabilities::new([KmsCapability::WrapDataKey]),
                digest(),
            ),
            Err(KmsProfileError::InvalidReference)
        );
    }

    #[test]
    fn profile_round_trip_preserves_identity_and_debug_redacts_references() {
        let profile = profile();
        let encoded = serde_json::to_string(&profile).unwrap();
        let decoded: KmsProfileIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, profile);
        assert!(format!("{profile:?}").contains("[redacted]"));
        assert!(!format!("{profile:?}").contains("kms-prod"));
        assert!(profile.supports(KmsCapability::WrapDataKey));
        assert!(!profile.supports(KmsCapability::GenerateDataKey));
    }
}
