//! Provider-neutral capability profile pinning for agent extension points.
//!
//! W12 第一阶段 (WBS 7.13): Memory/Skill/Context/Validation/Loop 五类 provider
//! contract 的 `profile / version / digest` 三元组固定与兼容性校验。digest 不匹配
//! 一律 **fail closed** —— 不降级解析、不静默替换、不暴露未固定的能力。
//!
//! 模式参照 `crates/domain-kms/src/profile.rs` 与 `provider_contract.rs`:
//! - digest 由可信 resolver 提供, 本模块**不计算**摘要 (与 domain-kms 一致);
//! - provider-neutral 校验 facade 在边界上强制 pin, 任何声称的 profile 与 pin
//!   不一致即拒绝;
//! - Run 级 pin set 固定每个 capability kind 恰好一个 profile。
//!
//! 超出本阶段范围 (写入"未完成"): capability broker、隔离 worker、provider 真实
//! 实现与 canonical serialization/digest 生成。

/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/domain-agent/src/capability_profile.rs"})
MERGE (kind:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityKind",kind:"enum"})
MERGE (digest:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfileDigest",kind:"type"})
MERGE (profile:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfile",kind:"type"})
MERGE (binding:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfileBinding",kind:"type"})
MERGE (pin_set:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfilePinSet",kind:"type"})
MERGE (descriptor:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProviderDescriptor",kind:"type"})
MERGE (provider:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProvider",kind:"trait"})
MERGE (client:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProviderClient",kind:"type"})
MERGE (error:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfileError",kind:"enum"})
MERGE (verify:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfile.verify_against",kind:"method"})
MERGE (verify_set:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProfilePinSet.verify_presented",kind:"method"})
MERGE (check:Symbol {id:"crates/domain-agent/src/capability_profile.rs::CapabilityProviderClient.check",kind:"method"})
MERGE (self)-[:DEFINES]->(kind)
MERGE (self)-[:DEFINES]->(digest)
MERGE (self)-[:DEFINES]->(profile)
MERGE (self)-[:DEFINES]->(binding)
MERGE (self)-[:DEFINES]->(pin_set)
MERGE (self)-[:DEFINES]->(descriptor)
MERGE (self)-[:DEFINES]->(provider)
MERGE (self)-[:DEFINES]->(client)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(verify_set)
MERGE (self)-[:DEFINES]->(check)
MERGE (profile)-[:USES_TYPE]->(digest)
MERGE (binding)-[:USES_TYPE]->(kind)
MERGE (binding)-[:USES_TYPE]->(profile)
MERGE (pin_set)-[:USES_TYPE]->(binding)
MERGE (client)-[:USES_TYPE]->(profile)
MERGE (client)-[:USES_TYPE]->(provider)
MERGE (check)-[:CALLS]->(verify)
MERGE (verify_set)-[:CALLS]->(verify)
MERGE (tests:Symbol {id:"crates/domain-agent/src/capability_profile.rs::tests",kind:"module"})
MERGE (self)-[:DEFINES]->(tests)
MERGE (tests)-[:TESTS]->(profile)
MERGE (tests)-[:TESTS]->(pin_set)
MERGE (tests)-[:TESTS]->(client)
@endcypher
*/

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::fmt;
use thiserror::Error;

/// Capability extension-point contracts a Run pins (W12: Memory/Skill/Context/Validation/Loop).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKind {
    /// Memory read/write provider contract.
    Memory,
    /// Skill selection provider contract.
    Skill,
    /// Context assembly provider contract.
    Context,
    /// Independent validation provider contract.
    Validation,
    /// Engineering Loop policy provider contract.
    Loop,
}

impl CapabilityKind {
    /// Returns the stable wire identifier for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Skill => "skill",
            Self::Context => "context",
            Self::Validation => "validation",
            Self::Loop => "loop",
        }
    }

    /// All capability kinds in deterministic order.
    pub const ALL: [CapabilityKind; 5] = [
        CapabilityKind::Memory,
        CapabilityKind::Skill,
        CapabilityKind::Context,
        CapabilityKind::Validation,
        CapabilityKind::Loop,
    ];
}

impl fmt::Display for CapabilityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

const MAX_NAME_BYTES: usize = 64;
const MAX_PROVIDER_REF_BYTES: usize = 256;
const MAX_CONTRACT_VERSION_BYTES: usize = 64;
const SHA256_HEX_LENGTH: usize = 64;

/// A fixed-size 32-byte profile digest supplied by a trusted resolver.
///
/// This module deliberately does not compute the digest; canonical serialization
/// and digest generation remain an open integration requirement (same posture as
/// domain-kms `ProfileDigest`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct CapabilityProfileDigest([u8; 32]);

impl CapabilityProfileDigest {
    /// Accepts a digest produced by the trusted resolver.
    ///
    /// An all-zero value is rejected as an unresolved/sentinel digest.
    pub fn from_trusted_resolver(bytes: [u8; 32]) -> Result<Self, CapabilityProfileError> {
        if bytes.iter().all(|byte| *byte == 0) {
            return Err(CapabilityProfileError::UnresolvedDigest);
        }
        Ok(Self(bytes))
    }

    /// Parses a 64-character hexadecimal digest (case-insensitive).
    ///
    /// An all-zero hex value is rejected as an unresolved digest.
    pub fn from_hex(hex: &str) -> Result<Self, CapabilityProfileError> {
        if hex.len() != SHA256_HEX_LENGTH || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(CapabilityProfileError::InvalidDigestHex);
        }
        let mut bytes = [0u8; 32];
        for (index, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
            let pair =
                std::str::from_utf8(chunk).map_err(|_| CapabilityProfileError::InvalidDigestHex)?;
            bytes[index] = u8::from_str_radix(pair, 16)
                .map_err(|_| CapabilityProfileError::InvalidDigestHex)?;
        }
        Self::from_trusted_resolver(bytes)
    }

    /// Returns the fixed-size digest bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the lowercase hexadecimal representation.
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl fmt::Debug for CapabilityProfileDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CapabilityProfileDigest([redacted])")
    }
}

impl<'de> Deserialize<'de> for CapabilityProfileDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <[u8; 32]>::deserialize(deserializer)?;
        Self::from_trusted_resolver(bytes).map_err(D::Error::custom)
    }
}

/// Pinned capability profile: the `name / version / digest` triple plus the
/// provider reference it resolves to.
///
/// A profile is immutable once constructed; validation fails closed on any
/// malformed field.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CapabilityProfileWire")]
pub struct CapabilityProfile {
    name: String,
    version: u32,
    digest: CapabilityProfileDigest,
    provider_ref: String,
}

impl CapabilityProfile {
    /// Constructs a pinned profile from trusted resolution results.
    pub fn new(
        name: impl Into<String>,
        version: u32,
        digest: CapabilityProfileDigest,
        provider_ref: impl Into<String>,
    ) -> Result<Self, CapabilityProfileError> {
        let name = name.into();
        let provider_ref = provider_ref.into();
        if !valid_name(&name) {
            return Err(CapabilityProfileError::InvalidName);
        }
        if version == 0 {
            return Err(CapabilityProfileError::InvalidVersion);
        }
        if !valid_provider_ref(&provider_ref) {
            return Err(CapabilityProfileError::InvalidProviderRef);
        }
        Ok(Self {
            name,
            version,
            digest,
            provider_ref,
        })
    }

    /// Returns the stable profile name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the monotonic profile version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the digest supplied by the trusted resolver.
    pub fn digest(&self) -> CapabilityProfileDigest {
        self.digest
    }

    /// Returns the opaque, non-secret provider reference.
    pub fn provider_ref(&self) -> &str {
        &self.provider_ref
    }

    /// Compatibility check against a claimed profile; any mismatch fails closed.
    ///
    /// Name, version, digest, and provider reference must all match exactly.
    /// There is no partial acceptance and no fallback resolution.
    pub fn verify_against(&self, claimed: &Self) -> Result<(), CapabilityProfileError> {
        if self.name != claimed.name {
            return Err(CapabilityProfileError::NameMismatch);
        }
        if self.version != claimed.version {
            return Err(CapabilityProfileError::VersionMismatch);
        }
        if self.digest != claimed.digest {
            return Err(CapabilityProfileError::DigestMismatch);
        }
        if self.provider_ref != claimed.provider_ref {
            return Err(CapabilityProfileError::ProviderRefMismatch);
        }
        Ok(())
    }
}

impl fmt::Debug for CapabilityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapabilityProfile")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("digest", &self.digest)
            .field("provider_ref", &self.provider_ref)
            .finish()
    }
}

#[derive(Deserialize)]
struct CapabilityProfileWire {
    name: String,
    version: u32,
    digest: CapabilityProfileDigest,
    provider_ref: String,
}

impl TryFrom<CapabilityProfileWire> for CapabilityProfile {
    type Error = CapabilityProfileError;

    fn try_from(wire: CapabilityProfileWire) -> Result<Self, Self::Error> {
        Self::new(wire.name, wire.version, wire.digest, wire.provider_ref)
    }
}

/// One `kind -> profile` binding inside a pinned set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityProfileBinding {
    /// Extension-point contract kind.
    pub kind: CapabilityKind,
    /// Exactly pinned profile for that kind.
    pub profile: CapabilityProfile,
}

/// Immutable Run-level pin set: exactly one pinned [`CapabilityProfile`] per
/// capability kind, verified against presented profiles before any use.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CapabilityProfilePinSetWire")]
pub struct CapabilityProfilePinSet {
    bindings: Vec<CapabilityProfileBinding>,
}

impl CapabilityProfilePinSet {
    /// Builds a pin set from `(kind, profile)` pairs.
    ///
    /// Rejects an empty set and duplicate kinds; bindings are kept in
    /// deterministic kind order.
    pub fn new(
        bindings: impl IntoIterator<Item = (CapabilityKind, CapabilityProfile)>,
    ) -> Result<Self, CapabilityProfileError> {
        let mut collected: Vec<CapabilityProfileBinding> = bindings
            .into_iter()
            .map(|(kind, profile)| CapabilityProfileBinding { kind, profile })
            .collect();
        for (index, binding) in collected.iter().enumerate() {
            if collected[..index]
                .iter()
                .any(|other| other.kind == binding.kind)
            {
                return Err(CapabilityProfileError::DuplicateKind(binding.kind));
            }
        }
        if collected.is_empty() {
            return Err(CapabilityProfileError::EmptyPinSet);
        }
        collected.sort_by_key(|binding| binding.kind);
        Ok(Self {
            bindings: collected,
        })
    }

    /// Returns the profile pinned for a kind, if any.
    pub fn pinned(&self, kind: CapabilityKind) -> Option<&CapabilityProfile> {
        self.bindings
            .iter()
            .find(|binding| binding.kind == kind)
            .map(|binding| &binding.profile)
    }

    /// Returns the number of pinned kinds.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns whether the pin set is empty (never true for a constructed set).
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Iterates pinned kinds in deterministic order.
    pub fn kinds(&self) -> impl Iterator<Item = CapabilityKind> + '_ {
        self.bindings.iter().map(|binding| binding.kind)
    }

    /// Verifies a presented pin set against this pinned set; fail closed.
    ///
    /// Every pinned kind must be presented with an exactly matching profile,
    /// and no kind may be presented that is not pinned. Any mismatch (name,
    /// version, digest, or provider reference) is an error.
    pub fn verify_presented(&self, presented: &Self) -> Result<(), CapabilityProfileError> {
        for binding in &self.bindings {
            match presented.pinned(binding.kind) {
                None => return Err(CapabilityProfileError::MissingPinnedProfile(binding.kind)),
                Some(claimed) => binding.profile.verify_against(claimed)?,
            }
        }
        for kind in presented.kinds() {
            if self.pinned(kind).is_none() {
                return Err(CapabilityProfileError::UnpinnedProfilePresented(kind));
            }
        }
        Ok(())
    }
}

impl fmt::Debug for CapabilityProfilePinSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(
                self.bindings
                    .iter()
                    .map(|binding| (binding.kind, binding.profile.name())),
            )
            .finish()
    }
}

#[derive(Deserialize)]
struct CapabilityProfilePinSetWire {
    bindings: Vec<CapabilityProfileBinding>,
}

impl TryFrom<CapabilityProfilePinSetWire> for CapabilityProfilePinSet {
    type Error = CapabilityProfileError;

    fn try_from(wire: CapabilityProfilePinSetWire) -> Result<Self, Self::Error> {
        Self::new(wire.bindings.into_iter().map(|b| (b.kind, b.profile)))
    }
}

/// Stable provider-neutral adapter descriptor: provider reference and contract version.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "CapabilityProviderDescriptorWire")]
pub struct CapabilityProviderDescriptor {
    provider_ref: String,
    contract_version: String,
}

impl CapabilityProviderDescriptor {
    /// Creates a descriptor after validating its non-secret identifiers.
    pub fn new(
        provider_ref: impl Into<String>,
        contract_version: impl Into<String>,
    ) -> Result<Self, CapabilityProfileError> {
        let provider_ref = provider_ref.into();
        let contract_version = contract_version.into();
        if !valid_provider_ref(&provider_ref) {
            return Err(CapabilityProfileError::InvalidProviderRef);
        }
        if !valid_contract_version(&contract_version) {
            return Err(CapabilityProfileError::InvalidContractVersion);
        }
        Ok(Self {
            provider_ref,
            contract_version,
        })
    }

    /// Returns the opaque provider reference.
    pub fn provider_ref(&self) -> &str {
        &self.provider_ref
    }

    /// Returns the provider contract version.
    pub fn contract_version(&self) -> &str {
        &self.contract_version
    }
}

impl fmt::Debug for CapabilityProviderDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapabilityProviderDescriptor")
            .field("provider_ref", &self.provider_ref)
            .field("contract_version", &self.contract_version)
            .finish()
    }
}

#[derive(Deserialize)]
struct CapabilityProviderDescriptorWire {
    provider_ref: String,
    contract_version: String,
}

impl TryFrom<CapabilityProviderDescriptorWire> for CapabilityProviderDescriptor {
    type Error = CapabilityProfileError;

    fn try_from(wire: CapabilityProviderDescriptorWire) -> Result<Self, Self::Error> {
        Self::new(wire.provider_ref, wire.contract_version)
    }
}

/// Provider-neutral capability adapter contract.
///
/// Adapters expose only metadata (descriptor + claimed profile); no capability
/// implementation code crosses this boundary. Business callers must use
/// [`CapabilityProviderClient`] so pinning is enforced before any use.
pub trait CapabilityProvider: Send + Sync {
    /// Returns the stable implementation descriptor advertised by this adapter.
    fn descriptor(&self) -> &CapabilityProviderDescriptor;

    /// Returns the exact profile this adapter claims to implement.
    fn claimed_profile(&self) -> &CapabilityProfile;
}

/// Validating facade that enforces profile pinning around a [`CapabilityProvider`].
///
/// The facade performs no capability work itself; it is the boundary check that
/// must pass (fail closed) before any pinned capability is invoked.
pub struct CapabilityProviderClient<P> {
    pin: CapabilityProfile,
    provider: P,
}

impl<P: CapabilityProvider> CapabilityProviderClient<P> {
    /// Creates a facade pinning `pin`; the provider is not validated here.
    pub fn new(pin: CapabilityProfile, provider: P) -> Self {
        Self { pin, provider }
    }

    /// Enforces the pin at the boundary: provider reference and the claimed
    /// `name / version / digest` triple must match exactly, else fail closed.
    pub fn check(&self) -> Result<(), CapabilityProfileError> {
        if self.provider.descriptor().provider_ref() != self.pin.provider_ref() {
            return Err(CapabilityProfileError::ProviderRefMismatch);
        }
        self.pin.verify_against(self.provider.claimed_profile())
    }

    /// Returns the immutable pinned profile.
    pub fn pin(&self) -> &CapabilityProfile {
        &self.pin
    }

    /// Borrows the wrapped provider adapter.
    pub fn provider(&self) -> &P {
        &self.provider
    }
}

/// Validation errors for capability profiles and pin sets.
///
/// All mismatch variants are fail-closed: callers must not substitute,
/// downgrade, or ignore them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CapabilityProfileError {
    /// Profile name did not match the restricted identifier grammar.
    #[error("invalid capability profile name")]
    InvalidName,
    /// Profile version must be nonzero.
    #[error("invalid capability profile version")]
    InvalidVersion,
    /// Provider reference did not match the opaque-reference grammar.
    #[error("invalid provider reference")]
    InvalidProviderRef,
    /// Provider contract version did not match the restricted grammar.
    #[error("invalid provider contract version")]
    InvalidContractVersion,
    /// An unresolved profile must not claim an all-zero digest.
    #[error("capability profile digest is unresolved")]
    UnresolvedDigest,
    /// Digest was not a 64-character hexadecimal string.
    #[error("invalid capability profile digest hex")]
    InvalidDigestHex,
    /// Claimed profile name differs from the pin.
    #[error("capability profile name mismatch")]
    NameMismatch,
    /// Claimed profile version differs from the pin.
    #[error("capability profile version mismatch")]
    VersionMismatch,
    /// Claimed profile digest differs from the pin.
    #[error("capability profile digest mismatch")]
    DigestMismatch,
    /// Claimed provider reference differs from the pin.
    #[error("capability provider reference mismatch")]
    ProviderRefMismatch,
    /// A pinned kind was not presented at all.
    #[error("missing pinned capability profile: {0}")]
    MissingPinnedProfile(CapabilityKind),
    /// A kind was presented that the Run does not pin.
    #[error("unpinned capability profile presented: {0}")]
    UnpinnedProfilePresented(CapabilityKind),
    /// The same kind was pinned more than once.
    #[error("duplicate capability profile kind: {0}")]
    DuplicateKind(CapabilityKind),
    /// A pin set must contain at least one binding.
    #[error("capability profile pin set is empty")]
    EmptyPinSet,
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_NAME_BYTES
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
}

fn valid_contract_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CONTRACT_VERSION_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

fn valid_provider_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROVIDER_REF_BYTES
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

    fn digest(byte: u8) -> CapabilityProfileDigest {
        CapabilityProfileDigest::from_trusted_resolver([byte; 32]).unwrap()
    }

    fn profile(name: &str, version: u32, byte: u8) -> CapabilityProfile {
        CapabilityProfile::new(name, version, digest(byte), format!("providers/{name}")).unwrap()
    }

    fn binding(kind: CapabilityKind, byte: u8) -> (CapabilityKind, CapabilityProfile) {
        (
            kind,
            CapabilityProfile::new(
                kind.as_str(),
                1,
                digest(byte),
                format!("providers/{}", kind.as_str()),
            )
            .unwrap(),
        )
    }

    fn full_set() -> CapabilityProfilePinSet {
        CapabilityProfilePinSet::new(CapabilityKind::ALL.iter().map(|kind| binding(*kind, 7)))
            .unwrap()
    }

    #[test]
    fn digest_rejects_unresolved_and_invalid_hex() {
        assert_eq!(
            CapabilityProfileDigest::from_trusted_resolver([0; 32]),
            Err(CapabilityProfileError::UnresolvedDigest)
        );
        assert_eq!(
            CapabilityProfileDigest::from_hex("zz"),
            Err(CapabilityProfileError::InvalidDigestHex)
        );
        assert_eq!(
            CapabilityProfileDigest::from_hex("ab12"),
            Err(CapabilityProfileError::InvalidDigestHex)
        );
        let hex = "ab".repeat(32);
        assert_eq!(
            CapabilityProfileDigest::from_hex(&hex).unwrap().to_hex(),
            hex
        );
        assert_eq!(
            CapabilityProfileDigest::from_hex(&"0".repeat(64)),
            Err(CapabilityProfileError::UnresolvedDigest)
        );
    }

    #[test]
    fn profile_validates_name_version_and_provider_ref() {
        assert_eq!(
            CapabilityProfile::new("-bad", 1, digest(1), "providers/x"),
            Err(CapabilityProfileError::InvalidName)
        );
        assert_eq!(
            CapabilityProfile::new("memory", 0, digest(1), "providers/x"),
            Err(CapabilityProfileError::InvalidVersion)
        );
        assert_eq!(
            CapabilityProfile::new("memory", 1, digest(1), "providers/x?token=secret"),
            Err(CapabilityProfileError::InvalidProviderRef)
        );
        assert_eq!(
            CapabilityProfile::new("memory", 1, digest(1), "providers/../x"),
            Err(CapabilityProfileError::InvalidProviderRef)
        );
    }

    #[test]
    fn verify_against_fails_closed_on_every_field() {
        let pinned = profile("memory", 3, 1);
        assert!(pinned.verify_against(&profile("memory", 3, 1)).is_ok());

        assert_eq!(
            pinned.verify_against(&profile("skill", 3, 1)),
            Err(CapabilityProfileError::NameMismatch)
        );
        assert_eq!(
            pinned.verify_against(&profile("memory", 4, 1)),
            Err(CapabilityProfileError::VersionMismatch)
        );
        assert_eq!(
            pinned.verify_against(&profile("memory", 3, 2)),
            Err(CapabilityProfileError::DigestMismatch)
        );
        let other_ref = CapabilityProfile::new("memory", 3, digest(1), "providers/other").unwrap();
        assert_eq!(
            pinned.verify_against(&other_ref),
            Err(CapabilityProfileError::ProviderRefMismatch)
        );
    }

    #[test]
    fn pin_set_rejects_empty_and_duplicate_kinds() {
        assert_eq!(
            CapabilityProfilePinSet::new(std::iter::empty()),
            Err(CapabilityProfileError::EmptyPinSet)
        );
        let memory = binding(CapabilityKind::Memory, 1);
        let duplicate = CapabilityProfilePinSet::new([memory.clone(), memory]);
        assert_eq!(
            duplicate,
            Err(CapabilityProfileError::DuplicateKind(
                CapabilityKind::Memory
            ))
        );
    }

    #[test]
    fn pin_set_verifies_presented_set_fail_closed() {
        let pinned = full_set();
        let mut presented = full_set();
        assert!(pinned.verify_presented(&presented).is_ok());

        // 缺一个 pin 的 kind → 拒绝。
        presented
            .bindings
            .retain(|binding| binding.kind != CapabilityKind::Loop);
        assert_eq!(
            pinned.verify_presented(&presented),
            Err(CapabilityProfileError::MissingPinnedProfile(
                CapabilityKind::Loop
            ))
        );

        // pinned 中某 kind 的 digest 与 presented 不一致 → 拒绝 (fail closed)。
        let mut digest_mismatch = full_set();
        digest_mismatch
            .bindings
            .retain(|binding| binding.kind != CapabilityKind::Context);
        digest_mismatch.bindings.push(CapabilityProfileBinding {
            kind: CapabilityKind::Context,
            profile: CapabilityProfile::new("context", 1, digest(9), "providers/context").unwrap(),
        });
        assert_eq!(
            pinned.verify_presented(&digest_mismatch),
            Err(CapabilityProfileError::DigestMismatch)
        );

        // presented 多出未 pin 的 kind → 拒绝。
        let subset = CapabilityProfilePinSet::new([
            binding(CapabilityKind::Memory, 7),
            binding(CapabilityKind::Skill, 7),
        ])
        .unwrap();
        assert_eq!(
            subset.verify_presented(&full_set()),
            Err(CapabilityProfileError::UnpinnedProfilePresented(
                CapabilityKind::Context
            ))
        );
    }

    #[test]
    fn pin_set_round_trips_and_debug_redacts_digest() {
        let pinned = full_set();
        let encoded = serde_json::to_string(&pinned).unwrap();
        let decoded: CapabilityProfilePinSet = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, pinned);
        assert_eq!(pinned.len(), CapabilityKind::ALL.len());
        assert!(pinned.pinned(CapabilityKind::Memory).is_some());
        let debug = format!("{pinned:?}");
        assert!(debug.contains("memory"));
        assert!(!debug.contains("redacted"));
    }

    #[test]
    fn deserializing_zero_digest_profile_fails_closed() {
        let zero = r#"{"name":"memory","version":1,"digest":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"provider_ref":"providers/memory"}"#;
        assert!(serde_json::from_str::<CapabilityProfile>(zero).is_err());
    }

    struct TestCapabilityProvider {
        descriptor: CapabilityProviderDescriptor,
        claimed: CapabilityProfile,
    }

    impl CapabilityProvider for TestCapabilityProvider {
        fn descriptor(&self) -> &CapabilityProviderDescriptor {
            &self.descriptor
        }

        fn claimed_profile(&self) -> &CapabilityProfile {
            &self.claimed
        }
    }

    fn client_for(
        claimed: CapabilityProfile,
        provider_ref: &str,
    ) -> CapabilityProviderClient<TestCapabilityProvider> {
        CapabilityProviderClient::new(
            profile("memory", 5, 11),
            TestCapabilityProvider {
                descriptor: CapabilityProviderDescriptor::new(provider_ref, "1.0").unwrap(),
                claimed,
            },
        )
    }

    #[test]
    fn facade_accepts_exact_pin_and_fails_closed_on_mismatch() {
        let exact = client_for(profile("memory", 5, 11), "providers/memory");
        assert!(exact.check().is_ok());

        let digest_mismatch = client_for(profile("memory", 5, 12), "providers/memory");
        assert_eq!(
            digest_mismatch.check(),
            Err(CapabilityProfileError::DigestMismatch)
        );

        let ref_mismatch = client_for(profile("memory", 5, 11), "providers/other");
        assert_eq!(
            ref_mismatch.check(),
            Err(CapabilityProfileError::ProviderRefMismatch)
        );

        let name_mismatch_claimed =
            CapabilityProfile::new("skill", 5, digest(11), "providers/memory").unwrap();
        let name_mismatch = client_for(name_mismatch_claimed, "providers/memory");
        assert_eq!(
            name_mismatch.check(),
            Err(CapabilityProfileError::NameMismatch)
        );
    }
}
