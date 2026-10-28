//! Provider-neutral, bounded KMS operations and their identity-pinning facade.

/*
@cypher schema=1 source_sha256=a49b189787ba8c9fdd5892d34ecb1ec98e32a711c16a4b76969e50d3542fe7c7
MERGE (self:File {path:"crates/domain-kms/src/provider_contract.rs"})
MERGE (profiles:File {path:"crates/domain-kms/src/profile.rs"})
MERGE (self)-[:IMPORTS]->(profiles)
MERGE (profile_identity:Symbol {id:"crates/domain-kms/src/profile.rs::KmsProfileIdentity",kind:"type"})
MERGE (provider:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider",kind:"trait"})
MERGE (client:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient",kind:"type"})
MERGE (wrapped:Symbol {id:"crates/domain-kms/src/provider_contract.rs::WrappedDataKey",kind:"type"})
MERGE (context:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsCallContext",kind:"type"})
MERGE (error:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderError",kind:"type"})
MERGE (check:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.check_operation",kind:"method"})
MERGE (provider_check:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.check_provider",kind:"method"})
MERGE (discover:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.discover_capabilities",kind:"method"})
MERGE (adapter_descriptor:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider.descriptor",kind:"method"})
MERGE (generate:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.generate_data_key",kind:"method"})
MERGE (wrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.wrap_data_key",kind:"method"})
MERGE (unwrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.unwrap_data_key",kind:"method"})
MERGE (rewrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.rewrap_data_key",kind:"method"})
MERGE (adapter_discover:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider.discover_capabilities",kind:"method"})
MERGE (adapter_generate:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider.generate_data_key",kind:"method"})
MERGE (adapter_wrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider.wrap_data_key",kind:"method"})
MERGE (adapter_unwrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider.unwrap_data_key",kind:"method"})
MERGE (adapter_rewrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProvider.rewrap_data_key",kind:"method"})
MERGE (identity_check:Symbol {id:"crates/domain-kms/src/provider_contract.rs::validate_wrapped_identity",kind:"function"})
MERGE (generated_identity_check:Symbol {id:"crates/domain-kms/src/provider_contract.rs::validate_generated_identity",kind:"function"})
MERGE (limit_check:Symbol {id:"crates/domain-kms/src/provider_contract.rs::ensure_response_bound",kind:"function"})
MERGE (self)-[:DEFINES]->(provider)
MERGE (self)-[:DEFINES]->(client)
MERGE (self)-[:DEFINES]->(wrapped)
MERGE (self)-[:DEFINES]->(context)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(check)
MERGE (self)-[:DEFINES]->(provider_check)
MERGE (self)-[:DEFINES]->(discover)
MERGE (self)-[:DEFINES]->(generate)
MERGE (self)-[:DEFINES]->(wrap)
MERGE (self)-[:DEFINES]->(unwrap)
MERGE (self)-[:DEFINES]->(rewrap)
MERGE (provider)-[:DEFINES]->(adapter_discover)
MERGE (provider)-[:DEFINES]->(adapter_descriptor)
MERGE (provider)-[:DEFINES]->(adapter_generate)
MERGE (provider)-[:DEFINES]->(adapter_wrap)
MERGE (provider)-[:DEFINES]->(adapter_unwrap)
MERGE (provider)-[:DEFINES]->(adapter_rewrap)
MERGE (self)-[:DEFINES]->(identity_check)
MERGE (self)-[:DEFINES]->(generated_identity_check)
MERGE (self)-[:DEFINES]->(limit_check)
MERGE (discover)-[:CALLS]->(provider_check)
MERGE (discover)-[:CALLS]->(adapter_discover)
MERGE (provider_check)-[:CALLS]->(adapter_descriptor)
MERGE (generate)-[:CALLS]->(adapter_generate)
MERGE (wrap)-[:CALLS]->(adapter_wrap)
MERGE (unwrap)-[:CALLS]->(adapter_unwrap)
MERGE (rewrap)-[:CALLS]->(adapter_rewrap)
MERGE (check)-[:CALLS]->(discover)
MERGE (generate)-[:CALLS]->(check)
MERGE (wrap)-[:CALLS]->(check)
MERGE (unwrap)-[:CALLS]->(check)
MERGE (rewrap)-[:CALLS]->(check)
MERGE (generate)-[:CALLS]->(generated_identity_check)
MERGE (wrap)-[:CALLS]->(identity_check)
MERGE (unwrap)-[:CALLS]->(limit_check)
MERGE (rewrap)-[:CALLS]->(identity_check)
MERGE (rewrap)-[:CALLS]->(limit_check)
MERGE (identity_check)-[:CALLS]->(limit_check)
MERGE (generated_identity_check)-[:CALLS]->(identity_check)
MERGE (tests:Symbol {id:"crates/domain-kms/src/provider_contract.rs::tests",kind:"module"})
MERGE (self)-[:DEFINES]->(tests)
MERGE (tests)-[:TESTS]->(client)
MERGE (tests)-[:TESTS]->(wrapped)
MERGE (tests)-[:TESTS]->(context)
MERGE (tests)-[:TESTS]->(generate)
MERGE (tests)-[:TESTS]->(wrap)
MERGE (tests)-[:TESTS]->(unwrap)
MERGE (tests)-[:TESTS]->(rewrap)
MERGE (provider)-[:USES_TYPE]->(context)
MERGE (provider)-[:USES_TYPE]->(profile_identity)
MERGE (client)-[:USES_TYPE]->(wrapped)
MERGE (client)-[:USES_TYPE]->(context)
MERGE (client)-[:USES_TYPE]->(profile_identity)
@endcypher
*/

use crate::profile::{KmsCapabilities, KmsCapability, KmsProfileIdentity, KmsProviderDescriptor};
use async_trait::async_trait;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::fmt;
use thiserror::Error;

/// Maximum response payload accepted by the provider-neutral contract (1 MiB).
pub const MAX_KMS_RESPONSE_BYTES: u32 = 1_048_576;
/// Maximum per-operation timeout accepted by the provider-neutral contract (2 minutes).
pub const MAX_KMS_TIMEOUT_MS: u32 = 120_000;
const MAX_OPERATION_ID_BYTES: usize = 128;
const MAX_KEY_VERSION_BYTES: usize = 128;

/// Bounded metadata required to execute one KMS operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsCallContext {
    operation_id: String,
    timeout_ms: u32,
    max_response_bytes: u32,
}

impl KmsCallContext {
    /// Creates a call context with a safe opaque operation ID and finite resource bounds.
    pub fn new(
        operation_id: impl Into<String>,
        timeout_ms: u32,
        max_response_bytes: u32,
    ) -> Result<Self, KmsProviderError> {
        let operation_id = operation_id.into();
        if operation_id.is_empty()
            || operation_id.len() > MAX_OPERATION_ID_BYTES
            || !operation_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        {
            return Err(KmsProviderError::new(KmsProviderErrorKind::InvalidRequest));
        }
        if timeout_ms == 0 || timeout_ms > MAX_KMS_TIMEOUT_MS {
            return Err(KmsProviderError::new(KmsProviderErrorKind::InvalidRequest));
        }
        if max_response_bytes == 0 || max_response_bytes > MAX_KMS_RESPONSE_BYTES {
            return Err(KmsProviderError::new(KmsProviderErrorKind::InvalidRequest));
        }
        Ok(Self {
            operation_id,
            timeout_ms,
            max_response_bytes,
        })
    }

    /// Returns the opaque operation correlation identifier.
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Returns the operation timeout in milliseconds.
    pub fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    /// Returns the maximum response bytes accepted for this operation.
    pub fn max_response_bytes(&self) -> u32 {
        self.max_response_bytes
    }
}

/// Provider-neutral failure classes; raw provider messages and payloads are excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KmsProviderErrorKind {
    /// Request metadata or operation parameters failed validation.
    InvalidRequest,
    /// Provider identity, profile identity, or wrapped-key identity did not match.
    IdentityMismatch,
    /// The resolved profile or provider does not permit the operation.
    CapabilityDenied,
    /// The referenced key or version does not exist.
    KeyNotFound,
    /// Provider authorization denied the operation.
    PermissionDenied,
    /// The provider rejected the operation as unsupported.
    Unsupported,
    /// The provider asked the caller to slow down.
    RateLimited,
    /// The provider is temporarily unavailable.
    TemporarilyUnavailable,
    /// The operation exceeded its deadline.
    Timeout,
    /// Wrapped data failed integrity or authenticity validation.
    IntegrityFailure,
    /// Provider output exceeded the operation's response bound.
    ResourceLimit,
    /// A provider failure without a more specific safe classification occurred.
    Internal,
}

/// Safe provider-neutral KMS failure without provider response text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("KMS provider operation failed: {kind:?}")]
pub struct KmsProviderError {
    kind: KmsProviderErrorKind,
}

impl KmsProviderError {
    /// Creates an error from a provider-neutral failure class.
    pub fn new(kind: KmsProviderErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the provider-neutral failure class.
    pub fn kind(&self) -> KmsProviderErrorKind {
        self.kind
    }

    /// Returns whether retrying the same pinned operation may be appropriate.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            KmsProviderErrorKind::RateLimited
                | KmsProviderErrorKind::TemporarilyUnavailable
                | KmsProviderErrorKind::Timeout
        )
    }
}

/// A 256-bit plaintext data key that cannot be cloned or serialized.
pub struct SecretDataKey([u8; 32]);

impl SecretDataKey {
    /// Wraps a freshly generated 256-bit key in a non-serializable secret type.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrows the key bytes for the duration of an operation.
    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for SecretDataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretDataKey([redacted])")
    }
}

impl Drop for SecretDataKey {
    fn drop(&mut self) {
        // Best-effort clearing without claiming the guarantees of an audited zeroize crate.
        self.0.fill(0);
        std::hint::black_box(&self.0);
    }
}

/// Opaque provider-wrapped key metadata safe for persistence.
///
/// It contains no plaintext data key, credentials, bearer token, or application payload.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(try_from = "WrappedDataKeyWire")]
pub struct WrappedDataKey {
    profile: KmsProfileIdentity,
    key_version: String,
    wrapped_bytes: Vec<u8>,
}

impl WrappedDataKey {
    /// Constructs wrapped metadata pinned to a profile identity.
    pub fn new(
        profile: KmsProfileIdentity,
        key_version: impl Into<String>,
        wrapped_bytes: Vec<u8>,
    ) -> Result<Self, KmsProviderError> {
        let key_version = key_version.into();
        if !valid_key_version(&key_version) || wrapped_bytes.is_empty() {
            return Err(KmsProviderError::new(KmsProviderErrorKind::InvalidRequest));
        }
        if wrapped_bytes.len() > MAX_KMS_RESPONSE_BYTES as usize {
            return Err(KmsProviderError::new(KmsProviderErrorKind::ResourceLimit));
        }
        Ok(Self {
            profile,
            key_version,
            wrapped_bytes,
        })
    }

    /// Returns the exact profile identity that was active when the key was wrapped.
    pub fn profile(&self) -> &KmsProfileIdentity {
        &self.profile
    }

    /// Returns the provider key version reference.
    pub fn key_version(&self) -> &str {
        &self.key_version
    }

    /// Borrows the opaque provider-wrapped bytes.
    pub fn wrapped_bytes(&self) -> &[u8] {
        &self.wrapped_bytes
    }
}

impl fmt::Debug for WrappedDataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WrappedDataKey")
            .field("profile", &self.profile)
            .field("key_version", &self.key_version)
            .field("wrapped_bytes", &"[redacted]")
            .finish()
    }
}

#[derive(Deserialize)]
struct WrappedDataKeyWire {
    profile: KmsProfileIdentity,
    key_version: String,
    wrapped_bytes: Vec<u8>,
}

impl TryFrom<WrappedDataKeyWire> for WrappedDataKey {
    type Error = KmsProviderError;

    fn try_from(wire: WrappedDataKeyWire) -> Result<Self, Self::Error> {
        Self::new(wire.profile, wire.key_version, wire.wrapped_bytes)
    }
}

impl<'de> Deserialize<'de> for WrappedDataKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WrappedDataKeyWire::deserialize(deserializer)?;
        Self::try_from(wire).map_err(D::Error::custom)
    }
}

/// Plaintext and wrapped form returned together by data-key generation.
pub struct GeneratedDataKey {
    /// Short-lived plaintext data key; never serialize or log it.
    pub plaintext: SecretDataKey,
    /// Persistable wrapped form pinned to the resolved profile.
    pub wrapped: WrappedDataKey,
}

impl fmt::Debug for GeneratedDataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedDataKey")
            .field("plaintext", &"[redacted]")
            .field("wrapped", &self.wrapped)
            .finish()
    }
}

/// Low-level adapter contract with provider-neutral inputs, results, and failures.
///
/// Business callers should use [`KmsProviderClient`] so identity, capabilities,
/// wrapped-key provenance, and response limits are checked at the boundary. Adapters
/// must treat [`KmsCallContext::timeout_ms`] as a hard operation deadline and cancel
/// provider work when it expires.
#[async_trait]
pub trait KmsProvider: Send + Sync {
    /// Returns the stable implementation descriptor advertised by this adapter.
    fn descriptor(&self) -> &KmsProviderDescriptor;

    /// Discovers operations supported by the resolved endpoint/key profile.
    async fn discover_capabilities(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
    ) -> Result<KmsCapabilities, KmsProviderError>;

    /// Generates a plaintext data key and provider-wrapped representation.
    async fn generate_data_key(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
    ) -> Result<GeneratedDataKey, KmsProviderError>;

    /// Wraps a caller-supplied plaintext data key.
    async fn wrap_data_key(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        plaintext: &SecretDataKey,
    ) -> Result<WrappedDataKey, KmsProviderError>;

    /// Unwraps a key only under the exact identity pinned in its metadata.
    async fn unwrap_data_key(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        wrapped: &WrappedDataKey,
    ) -> Result<SecretDataKey, KmsProviderError>;

    /// Rewraps within one provider implementation and endpoint lineage.
    async fn rewrap_data_key(
        &self,
        source_profile: &KmsProfileIdentity,
        target_profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        wrapped: &WrappedDataKey,
    ) -> Result<WrappedDataKey, KmsProviderError>;
}

/// Validating facade that enforces profile pinning around a [`KmsProvider`].
pub struct KmsProviderClient<P> {
    provider: P,
}

impl<P: KmsProvider> KmsProviderClient<P> {
    /// Creates a facade for an adapter; no provider is selected or enabled here.
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Discovers capabilities and verifies that the profile grants only supported operations.
    pub async fn discover_capabilities(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
    ) -> Result<KmsCapabilities, KmsProviderError> {
        self.check_provider(profile)?;
        let actual = self
            .provider
            .discover_capabilities(profile, context)
            .await?;
        if profile
            .capabilities()
            .iter()
            .any(|capability| !actual.contains(capability))
        {
            return Err(KmsProviderError::new(
                KmsProviderErrorKind::CapabilityDenied,
            ));
        }
        // The profile is the authorization snapshot. Never expose provider capabilities
        // that the trusted resolver did not pin into this profile identity.
        Ok(profile.capabilities().clone())
    }

    /// Generates a data key after provider and profile capability checks.
    pub async fn generate_data_key(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
    ) -> Result<GeneratedDataKey, KmsProviderError> {
        self.check_operation(profile, context, KmsCapability::GenerateDataKey)
            .await?;
        let generated = self.provider.generate_data_key(profile, context).await?;
        validate_generated_identity(profile, &generated, context)?;
        Ok(generated)
    }

    /// Wraps a data key after provider and profile capability checks.
    pub async fn wrap_data_key(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        plaintext: &SecretDataKey,
    ) -> Result<WrappedDataKey, KmsProviderError> {
        self.check_operation(profile, context, KmsCapability::WrapDataKey)
            .await?;
        let wrapped = self
            .provider
            .wrap_data_key(profile, context, plaintext)
            .await?;
        validate_wrapped_identity(profile, &wrapped, context)?;
        Ok(wrapped)
    }

    /// Unwraps only metadata pinned to the exact requested profile identity.
    pub async fn unwrap_data_key(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        wrapped: &WrappedDataKey,
    ) -> Result<SecretDataKey, KmsProviderError> {
        if wrapped.profile() != profile {
            return Err(KmsProviderError::new(
                KmsProviderErrorKind::IdentityMismatch,
            ));
        }
        ensure_response_bound(wrapped, context)?;
        self.check_operation(profile, context, KmsCapability::UnwrapDataKey)
            .await?;
        self.provider
            .unwrap_data_key(profile, context, wrapped)
            .await
    }

    /// Rewraps within the same provider id/version and endpoint; cross-provider migration fails closed.
    pub async fn rewrap_data_key(
        &self,
        source_profile: &KmsProfileIdentity,
        target_profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        wrapped: &WrappedDataKey,
    ) -> Result<WrappedDataKey, KmsProviderError> {
        if wrapped.profile() != source_profile
            || source_profile.provider() != target_profile.provider()
            || source_profile.endpoint_ref() != target_profile.endpoint_ref()
        {
            return Err(KmsProviderError::new(
                KmsProviderErrorKind::IdentityMismatch,
            ));
        }
        ensure_response_bound(wrapped, context)?;
        self.check_operation(source_profile, context, KmsCapability::RewrapDataKey)
            .await?;
        self.check_operation(target_profile, context, KmsCapability::RewrapDataKey)
            .await?;
        let result = self
            .provider
            .rewrap_data_key(source_profile, target_profile, context, wrapped)
            .await?;
        validate_wrapped_identity(target_profile, &result, context)?;
        Ok(result)
    }

    async fn check_operation(
        &self,
        profile: &KmsProfileIdentity,
        context: &KmsCallContext,
        required: KmsCapability,
    ) -> Result<(), KmsProviderError> {
        if !profile.supports(required) {
            return Err(KmsProviderError::new(
                KmsProviderErrorKind::CapabilityDenied,
            ));
        }
        let actual = self.discover_capabilities(profile, context).await?;
        if !actual.contains(required) {
            return Err(KmsProviderError::new(
                KmsProviderErrorKind::CapabilityDenied,
            ));
        }
        Ok(())
    }

    fn check_provider(&self, profile: &KmsProfileIdentity) -> Result<(), KmsProviderError> {
        if self.provider.descriptor() != profile.provider() {
            return Err(KmsProviderError::new(
                KmsProviderErrorKind::IdentityMismatch,
            ));
        }
        Ok(())
    }
}

fn validate_wrapped_identity(
    expected: &KmsProfileIdentity,
    wrapped: &WrappedDataKey,
    context: &KmsCallContext,
) -> Result<(), KmsProviderError> {
    if wrapped.profile() != expected {
        return Err(KmsProviderError::new(
            KmsProviderErrorKind::IdentityMismatch,
        ));
    }
    ensure_response_bound(wrapped, context)
}

fn validate_generated_identity(
    expected: &KmsProfileIdentity,
    generated: &GeneratedDataKey,
    context: &KmsCallContext,
) -> Result<(), KmsProviderError> {
    validate_wrapped_identity(expected, &generated.wrapped, context)?;
    let combined_bytes = generated
        .wrapped
        .wrapped_bytes()
        .len()
        .saturating_add(generated.plaintext.expose().len());
    if combined_bytes > context.max_response_bytes() as usize {
        return Err(KmsProviderError::new(KmsProviderErrorKind::ResourceLimit));
    }
    Ok(())
}

fn ensure_response_bound(
    wrapped: &WrappedDataKey,
    context: &KmsCallContext,
) -> Result<(), KmsProviderError> {
    if wrapped.wrapped_bytes().len() > context.max_response_bytes() as usize {
        return Err(KmsProviderError::new(KmsProviderErrorKind::ResourceLimit));
    }
    Ok(())
}

fn valid_key_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_KEY_VERSION_BYTES
        && !value.contains("//")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{KmsProfileError, ProfileDigest};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestProvider {
        descriptor: KmsProviderDescriptor,
        capabilities: KmsCapabilities,
        calls: Arc<AtomicUsize>,
        output_profile_override: Option<KmsProfileIdentity>,
    }

    #[async_trait]
    impl KmsProvider for TestProvider {
        fn descriptor(&self) -> &KmsProviderDescriptor {
            &self.descriptor
        }

        async fn discover_capabilities(
            &self,
            _profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
        ) -> Result<KmsCapabilities, KmsProviderError> {
            Ok(self.capabilities.clone())
        }

        async fn generate_data_key(
            &self,
            profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
        ) -> Result<GeneratedDataKey, KmsProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let wrapped_profile = self
                .output_profile_override
                .as_ref()
                .unwrap_or(profile)
                .clone();
            Ok(GeneratedDataKey {
                plaintext: SecretDataKey::from_bytes([5; 32]),
                wrapped: WrappedDataKey::new(wrapped_profile, "v1", vec![8; 32])?,
            })
        }

        async fn wrap_data_key(
            &self,
            profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
            _plaintext: &SecretDataKey,
        ) -> Result<WrappedDataKey, KmsProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let wrapped_profile = self
                .output_profile_override
                .as_ref()
                .unwrap_or(profile)
                .clone();
            WrappedDataKey::new(wrapped_profile, "v1", vec![8; 32])
        }

        async fn unwrap_data_key(
            &self,
            _profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
            _wrapped: &WrappedDataKey,
        ) -> Result<SecretDataKey, KmsProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(SecretDataKey::from_bytes([5; 32]))
        }

        async fn rewrap_data_key(
            &self,
            _source_profile: &KmsProfileIdentity,
            target_profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
            _wrapped: &WrappedDataKey,
        ) -> Result<WrappedDataKey, KmsProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let wrapped_profile = self
                .output_profile_override
                .as_ref()
                .unwrap_or(target_profile)
                .clone();
            WrappedDataKey::new(wrapped_profile, "v2", vec![9; 32])
        }
    }

    fn profile(provider_id: &str, digest_byte: u8) -> KmsProfileIdentity {
        KmsProfileIdentity::new(
            KmsProviderDescriptor::new(provider_id, "1.0").unwrap(),
            1,
            "kms-prod",
            "tenant/key",
            KmsCapabilities::new([
                KmsCapability::GenerateDataKey,
                KmsCapability::WrapDataKey,
                KmsCapability::UnwrapDataKey,
                KmsCapability::RewrapDataKey,
            ]),
            ProfileDigest::from_trusted_resolver([digest_byte; 32]).unwrap(),
        )
        .unwrap()
    }

    fn context(max_response_bytes: u32) -> KmsCallContext {
        KmsCallContext::new("op-1", 5_000, max_response_bytes).unwrap()
    }

    fn client(provider_id: &str) -> (KmsProviderClient<TestProvider>, Arc<AtomicUsize>) {
        client_with_output_profile(provider_id, None)
    }

    fn client_with_output_profile(
        provider_id: &str,
        output_profile_override: Option<KmsProfileIdentity>,
    ) -> (KmsProviderClient<TestProvider>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let client = KmsProviderClient::new(TestProvider {
            descriptor: KmsProviderDescriptor::new(provider_id, "1.0").unwrap(),
            capabilities: KmsCapabilities::new([
                KmsCapability::GenerateDataKey,
                KmsCapability::WrapDataKey,
                KmsCapability::UnwrapDataKey,
                KmsCapability::RewrapDataKey,
            ]),
            calls: Arc::clone(&calls),
            output_profile_override,
        });
        (client, calls)
    }

    #[tokio::test]
    async fn operation_pins_profile_and_rejects_provider_mismatch_before_call() {
        let (client, calls) = client("provider-a");
        let wrong_profile = profile("provider-b", 1);
        let error = client
            .generate_data_key(&wrong_profile, &context(1024))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::IdentityMismatch);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn rejects_provider_output_bound_to_a_different_profile() {
        let expected = profile("provider-a", 11);
        let other = profile("provider-a", 12);
        let (client, calls) = client_with_output_profile("provider-a", Some(other));
        let error = client
            .generate_data_key(&expected, &context(1024))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::IdentityMismatch);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn rejects_capability_not_pinned_by_profile() {
        let (client, calls) = client("provider-a");
        let resolved = profile("provider-a", 2);
        let profile = KmsProfileIdentity::new(
            resolved.provider().clone(),
            resolved.revision(),
            resolved.endpoint_ref(),
            resolved.key_ref(),
            KmsCapabilities::new([KmsCapability::WrapDataKey]),
            ProfileDigest::from_trusted_resolver([3; 32]).unwrap(),
        )
        .unwrap();
        let error = client
            .generate_data_key(&profile, &context(1024))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::CapabilityDenied);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn capability_discovery_returns_only_profile_pinned_capabilities() {
        let (client, _) = client("provider-a");
        let resolved = profile("provider-a", 9);
        let profile = KmsProfileIdentity::new(
            resolved.provider().clone(),
            resolved.revision(),
            resolved.endpoint_ref(),
            resolved.key_ref(),
            KmsCapabilities::new([KmsCapability::WrapDataKey]),
            ProfileDigest::from_trusted_resolver([10; 32]).unwrap(),
        )
        .unwrap();
        let discovered = client
            .discover_capabilities(&profile, &context(64))
            .await
            .unwrap();
        assert_eq!(discovered, profile.capabilities().clone());
        assert_eq!(discovered.len(), 1);
    }

    #[tokio::test]
    async fn wrapped_metadata_is_bound_to_profile_and_response_limit() {
        let (client, _) = client("provider-a");
        let profile = profile("provider-a", 4);
        let wrapped = client
            .generate_data_key(&profile, &context(64))
            .await
            .unwrap()
            .wrapped;
        assert_eq!(wrapped.profile(), &profile);
        assert_eq!(wrapped.wrapped_bytes(), &[8; 32]);
        let encoded = serde_json::to_string(&wrapped).unwrap();
        let decoded: WrappedDataKey = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, wrapped);

        let error = client
            .unwrap_data_key(&profile, &context(16), &wrapped)
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::ResourceLimit);

        let error = client
            .generate_data_key(&profile, &context(63))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::ResourceLimit);
    }

    #[tokio::test]
    async fn rejects_cross_provider_rewrap_and_mismatched_wrapped_identity() {
        let (client, calls) = client("provider-a");
        let source = profile("provider-a", 5);
        let target = profile("provider-b", 6);
        let wrapped = WrappedDataKey::new(source.clone(), "v1", vec![1, 2, 3]).unwrap();
        let error = client
            .rewrap_data_key(&source, &target, &context(64), &wrapped)
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::IdentityMismatch);
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let other_identity = profile("provider-a", 7);
        let error = client
            .unwrap_data_key(&other_identity, &context(64), &wrapped)
            .await
            .unwrap_err();
        assert_eq!(error.kind(), KmsProviderErrorKind::IdentityMismatch);
    }

    #[test]
    fn operation_bounds_and_safe_error_mapping_are_enforced() {
        assert_eq!(
            KmsCallContext::new("unsafe?secret", 10, 10)
                .unwrap_err()
                .kind(),
            KmsProviderErrorKind::InvalidRequest
        );
        assert_eq!(
            KmsCallContext::new("op", MAX_KMS_TIMEOUT_MS + 1, 10)
                .unwrap_err()
                .kind(),
            KmsProviderErrorKind::InvalidRequest
        );
        assert!(KmsProviderError::new(KmsProviderErrorKind::Timeout).is_retryable());
        assert!(!KmsProviderError::new(KmsProviderErrorKind::PermissionDenied).is_retryable());
        assert_eq!(
            KmsProfileIdentity::new(
                KmsProviderDescriptor::new("provider-a", "1.0").unwrap(),
                0,
                "endpoint",
                "key",
                KmsCapabilities::new([KmsCapability::WrapDataKey]),
                ProfileDigest::from_trusted_resolver([8; 32]).unwrap(),
            )
            .unwrap_err(),
            KmsProfileError::InvalidRevision
        );
    }
}
