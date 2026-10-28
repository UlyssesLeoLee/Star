//! Bounded, provider-neutral authenticated encryption for coalesced transcript checkpoints.

/*
@cypher schema=1 source_sha256=7f2320acf6cf8d8a6003e8b4963fcda37094fe7047d1bb9dab4692c113ea2479
MERGE (self:File {path:"crates/domain-kms/src/transcript_envelope.rs"})
MERGE (profile:File {path:"crates/domain-kms/src/profile.rs"})
MERGE (provider_contract:File {path:"crates/domain-kms/src/provider_contract.rs"})
MERGE (aad:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptAad",kind:"type"})
MERGE (envelope:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelope",kind:"type"})
MERGE (protector:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelopeProtector",kind:"type"})
MERGE (error:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelopeError",kind:"type"})
MERGE (aad_new:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptAad.new",kind:"method"})
MERGE (aad_encode:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptAad.canonical_bytes",kind:"method"})
MERGE (envelope_encode:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelope.to_json",kind:"method"})
MERGE (envelope_decode:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelope.from_json",kind:"method"})
MERGE (envelope_validate:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelope.validate",kind:"method"})
MERGE (protector_new:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelopeProtector.new",kind:"method"})
MERGE (protect:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelopeProtector.protect",kind:"method"})
MERGE (decrypt:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::TranscriptEnvelopeProtector.decrypt",kind:"method"})
MERGE (tests:Symbol {id:"crates/domain-kms/src/transcript_envelope.rs::tests",kind:"module"})
MERGE (self)-[:IMPORTS]->(profile)
MERGE (self)-[:IMPORTS]->(provider_contract)
MERGE (self)-[:DEFINES]->(aad)
MERGE (self)-[:DEFINES]->(envelope)
MERGE (self)-[:DEFINES]->(protector)
MERGE (self)-[:DEFINES]->(error)
MERGE (aad)-[:HAS_METHOD]->(aad_new)
MERGE (aad)-[:HAS_METHOD]->(aad_encode)
MERGE (envelope)-[:HAS_METHOD]->(envelope_encode)
MERGE (envelope)-[:HAS_METHOD]->(envelope_decode)
MERGE (envelope)-[:HAS_METHOD]->(envelope_validate)
MERGE (protector)-[:HAS_METHOD]->(protector_new)
MERGE (protector)-[:HAS_METHOD]->(protect)
MERGE (protector)-[:HAS_METHOD]->(decrypt)
MERGE (protect)-[:CALLS]->(aad_encode)
MERGE (protect)-[:CALLS]->(provider_generate:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.generate_data_key",kind:"method"})
MERGE (decrypt)-[:CALLS]->(aad_encode)
MERGE (decrypt)-[:CALLS]->(provider_unwrap:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient.unwrap_data_key",kind:"method"})
MERGE (protector)-[:USES_TYPE]->(profile_identity:Symbol {id:"crates/domain-kms/src/profile.rs::KmsProfileIdentity",kind:"type"})
MERGE (protector)-[:USES_TYPE]->(provider_client:Symbol {id:"crates/domain-kms/src/provider_contract.rs::KmsProviderClient",kind:"type"})
MERGE (self)-[:DEFINES]->(tests)
MERGE (tests)-[:TESTS]->(aad)
MERGE (tests)-[:TESTS]->(envelope)
MERGE (tests)-[:TESTS]->(protector)
@endcypher
*/

use crate::profile::{KmsCapability, KmsProfileIdentity};
use crate::provider_contract::{
    KmsCallContext, KmsProvider, KmsProviderClient, KmsProviderError, KmsProviderErrorKind,
    SecretDataKey, WrappedDataKey,
};
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Current transcript envelope format version.
pub const TRANSCRIPT_ENVELOPE_FORMAT_VERSION: u32 = 1;
/// Current canonical associated-data schema version.
pub const TRANSCRIPT_AAD_SCHEMA_VERSION: u16 = 1;
/// Maximum plaintext bytes accepted for one coalesced checkpoint.
pub const MAX_TRANSCRIPT_BODY_BYTES: usize = 256 * 1024;
/// Maximum wrapped data-key bytes accepted for one envelope.
pub const MAX_WRAPPED_DATA_KEY_BYTES: usize = 64 * 1024;
/// Maximum serialized JSON bytes accepted by the bounded envelope decoder.
pub const MAX_TRANSCRIPT_ENVELOPE_JSON_BYTES: usize = 1024 * 1024;

const AEAD_SUITE: &str = "AES-256-GCM";
const AEAD_NONCE_BYTES: usize = 12;
const AEAD_TAG_BYTES: usize = 16;
const MAX_CIPHERTEXT_BYTES: usize = MAX_TRANSCRIPT_BODY_BYTES + AEAD_TAG_BYTES;
const MAX_CIPHERTEXT_BASE64_BYTES: usize = MAX_CIPHERTEXT_BYTES.div_ceil(3) * 4;
const NONCE_BASE64_BYTES: usize = AEAD_NONCE_BYTES.div_ceil(3) * 4;
const AAD_DOMAIN_SEPARATOR: &[u8; 8] = b"STAR-W16";
const MAX_RESOURCE_KIND_BYTES: usize = 64;
// 8 domain + 2 schema + 16 tenant + 16 project + 16 run + 1 worktree tag
// + 16 optional worktree + 2 resource-kind length + 64 kind + 16 resource
// + 16 actor + 8 base revision + 16 payload + 8 checkpoint + 8 retention
// + 4 envelope format = 217 bytes with a present worktree.
const MAX_AAD_BYTES: usize = 217;

/// Safe, static envelope failures that never carry plaintext or provider response text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TranscriptEnvelopeError {
    /// A context identifier or canonical field failed validation.
    #[error("transcript envelope context is invalid")]
    InvalidContext,
    /// An envelope is malformed or uses an unsupported format or algorithm.
    #[error("transcript envelope format is invalid or unsupported")]
    InvalidEnvelope,
    /// An input exceeded a fixed memory or encoding limit.
    #[error("transcript envelope resource limit exceeded")]
    ResourceLimit,
    /// The envelope does not belong to the protector's pinned profile.
    #[error("transcript envelope profile does not match")]
    ProfileMismatch,
    /// AEAD authentication failed.
    #[error("transcript envelope authentication failed")]
    IntegrityFailure,
    /// The provider operation failed with a safe provider-neutral classification.
    #[error("KMS operation failed ({0:?})")]
    Provider(KmsProviderErrorKind),
}

impl From<KmsProviderError> for TranscriptEnvelopeError {
    fn from(error: KmsProviderError) -> Self {
        Self::Provider(error.kind())
    }
}

/// Owner-validated identifiers and revisions encoded into canonical binary AAD.
///
/// IDs are raw 16-byte UUID representations from trusted owner context. This value
/// is not serialized and must not be built from an untrusted client payload.
pub struct TranscriptAad {
    tenant_id: [u8; 16],
    project_id: [u8; 16],
    worktree_id: Option<[u8; 16]>,
    run_id: [u8; 16],
    resource_kind: String,
    resource_id: [u8; 16],
    actor_id: [u8; 16],
    base_revision: u64,
    payload_id: [u8; 16],
    checkpoint_revision: u64,
    retention_policy_revision: u64,
}

impl TranscriptAad {
    /// Creates AAD after validating identifiers, resource kind, and revision presence.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: [u8; 16],
        project_id: [u8; 16],
        worktree_id: Option<[u8; 16]>,
        run_id: [u8; 16],
        resource_kind: impl Into<String>,
        resource_id: [u8; 16],
        actor_id: [u8; 16],
        base_revision: u64,
        payload_id: [u8; 16],
        checkpoint_revision: u64,
        retention_policy_revision: u64,
    ) -> Result<Self, TranscriptEnvelopeError> {
        let aad = Self {
            tenant_id,
            project_id,
            worktree_id,
            run_id,
            resource_kind: resource_kind.into(),
            resource_id,
            actor_id,
            base_revision,
            payload_id,
            checkpoint_revision,
            retention_policy_revision,
        };
        aad.validate()?;
        Ok(aad)
    }

    fn validate(&self) -> Result<(), TranscriptEnvelopeError> {
        if is_nil_id(&self.tenant_id)
            || is_nil_id(&self.project_id)
            || self.worktree_id.is_some_and(|id| is_nil_id(&id))
            || is_nil_id(&self.run_id)
            || is_nil_id(&self.resource_id)
            || is_nil_id(&self.actor_id)
            || is_nil_id(&self.payload_id)
            || self.checkpoint_revision == 0
            || self.retention_policy_revision == 0
            || !is_valid_resource_kind(&self.resource_kind)
        {
            return Err(TranscriptEnvelopeError::InvalidContext);
        }
        Ok(())
    }

    fn canonical_bytes(
        &self,
        envelope_format_version: u32,
    ) -> Result<Vec<u8>, TranscriptEnvelopeError> {
        self.validate()?;
        if envelope_format_version != TRANSCRIPT_ENVELOPE_FORMAT_VERSION {
            return Err(TranscriptEnvelopeError::InvalidEnvelope);
        }

        let mut aad = Vec::with_capacity(MAX_AAD_BYTES);
        aad.extend_from_slice(AAD_DOMAIN_SEPARATOR);
        aad.extend_from_slice(&TRANSCRIPT_AAD_SCHEMA_VERSION.to_be_bytes());
        aad.extend_from_slice(&self.tenant_id);
        aad.extend_from_slice(&self.project_id);
        aad.extend_from_slice(&self.run_id);
        match self.worktree_id {
            None => aad.push(0),
            Some(worktree_id) => {
                aad.push(1);
                aad.extend_from_slice(&worktree_id);
            }
        }
        let resource_kind_len = u16::try_from(self.resource_kind.len())
            .map_err(|_| TranscriptEnvelopeError::InvalidContext)?;
        aad.extend_from_slice(&resource_kind_len.to_be_bytes());
        aad.extend_from_slice(self.resource_kind.as_bytes());
        aad.extend_from_slice(&self.resource_id);
        aad.extend_from_slice(&self.actor_id);
        aad.extend_from_slice(&self.base_revision.to_be_bytes());
        aad.extend_from_slice(&self.payload_id);
        aad.extend_from_slice(&self.checkpoint_revision.to_be_bytes());
        aad.extend_from_slice(&self.retention_policy_revision.to_be_bytes());
        aad.extend_from_slice(&envelope_format_version.to_be_bytes());
        if aad.len() > MAX_AAD_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        Ok(aad)
    }
}

/// Versioned authenticated ciphertext and the provider-wrapped, opaque data key.
///
/// The profile identity is carried by wrapped_dek. The envelope intentionally omits
/// a plaintext digest and does not claim durable storage or key recoverability.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptEnvelope {
    format_version: u32,
    aad_schema_version: u16,
    aead_suite: String,
    wrapped_dek: WrappedDataKey,
    nonce: String,
    ciphertext_and_tag: String,
}

impl std::fmt::Debug for TranscriptEnvelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranscriptEnvelope")
            .field("format_version", &self.format_version)
            .field("aad_schema_version", &self.aad_schema_version)
            .field("aead_suite", &self.aead_suite)
            .field("wrapped_dek", &self.wrapped_dek)
            .field("nonce", &"[redacted]")
            .field("ciphertext_and_tag", &"[redacted]")
            .finish()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TranscriptEnvelopeWire {
    format_version: u32,
    aad_schema_version: u16,
    aead_suite: String,
    wrapped_dek: WrappedDataKey,
    nonce: String,
    ciphertext_and_tag: String,
}

impl TranscriptEnvelope {
    /// Returns the envelope format version.
    pub fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the AAD schema version.
    pub fn aad_schema_version(&self) -> u16 {
        self.aad_schema_version
    }

    /// Returns the named AEAD suite.
    pub fn aead_suite(&self) -> &str {
        &self.aead_suite
    }

    /// Returns provider-wrapped key metadata pinned to its original profile.
    pub fn wrapped_dek(&self) -> &WrappedDataKey {
        &self.wrapped_dek
    }

    /// Encodes the envelope as bounded JSON for persistence by an authorized owner.
    pub fn to_json(&self) -> Result<Vec<u8>, TranscriptEnvelopeError> {
        self.validate()?;
        let encoded =
            serde_json::to_vec(self).map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        if encoded.len() > MAX_TRANSCRIPT_ENVELOPE_JSON_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        Ok(encoded)
    }

    /// Decodes JSON only after applying the total byte limit, then validates every field.
    pub fn from_json(encoded: &[u8]) -> Result<Self, TranscriptEnvelopeError> {
        if encoded.len() > MAX_TRANSCRIPT_ENVELOPE_JSON_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        let wire: TranscriptEnvelopeWire = serde_json::from_slice(encoded)
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        Self::try_from(wire)
    }

    fn validate(&self) -> Result<(), TranscriptEnvelopeError> {
        if self.format_version != TRANSCRIPT_ENVELOPE_FORMAT_VERSION
            || self.aad_schema_version != TRANSCRIPT_AAD_SCHEMA_VERSION
            || self.aead_suite != AEAD_SUITE
        {
            return Err(TranscriptEnvelopeError::InvalidEnvelope);
        }
        if self.wrapped_dek.wrapped_bytes().len() > MAX_WRAPPED_DATA_KEY_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        self.decode_parts().map(|_| ())
    }

    fn decode_parts(&self) -> Result<([u8; AEAD_NONCE_BYTES], Vec<u8>), TranscriptEnvelopeError> {
        if self.nonce.len() != NONCE_BASE64_BYTES {
            return Err(TranscriptEnvelopeError::InvalidEnvelope);
        }
        if self.ciphertext_and_tag.len() > MAX_CIPHERTEXT_BASE64_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        let nonce = BASE64
            .decode(self.nonce.as_bytes())
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        let nonce: [u8; AEAD_NONCE_BYTES] = nonce
            .try_into()
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        let ciphertext = BASE64
            .decode(self.ciphertext_and_tag.as_bytes())
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        if ciphertext.len() < AEAD_TAG_BYTES || ciphertext.len() > MAX_CIPHERTEXT_BYTES {
            return Err(TranscriptEnvelopeError::InvalidEnvelope);
        }
        Ok((nonce, ciphertext))
    }
}

impl TryFrom<TranscriptEnvelopeWire> for TranscriptEnvelope {
    type Error = TranscriptEnvelopeError;

    fn try_from(wire: TranscriptEnvelopeWire) -> Result<Self, Self::Error> {
        let envelope = Self {
            format_version: wire.format_version,
            aad_schema_version: wire.aad_schema_version,
            aead_suite: wire.aead_suite,
            wrapped_dek: wire.wrapped_dek,
            nonce: wire.nonce,
            ciphertext_and_tag: wire.ciphertext_and_tag,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

/// Encrypts and decrypts bounded transcript payloads through one pinned KMS profile.
pub struct TranscriptEnvelopeProtector<P: KmsProvider> {
    kms: KmsProviderClient<P>,
    profile: KmsProfileIdentity,
}

impl<P: KmsProvider> TranscriptEnvelopeProtector<P> {
    /// Creates a protector from a trusted profile snapshot; no implicit discovery occurs.
    pub fn new(
        kms: KmsProviderClient<P>,
        profile: KmsProfileIdentity,
    ) -> Result<Self, TranscriptEnvelopeError> {
        if !profile.supports(KmsCapability::GenerateDataKey)
            || !profile.supports(KmsCapability::UnwrapDataKey)
        {
            return Err(TranscriptEnvelopeError::Provider(
                KmsProviderErrorKind::CapabilityDenied,
            ));
        }
        Ok(Self { kms, profile })
    }

    /// Generates one data key, encrypts one bounded checkpoint, and returns opaque metadata.
    pub async fn protect(
        &self,
        context: &KmsCallContext,
        aad: &TranscriptAad,
        plaintext: &[u8],
    ) -> Result<TranscriptEnvelope, TranscriptEnvelopeError> {
        if plaintext.len() > MAX_TRANSCRIPT_BODY_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        let aad_bytes = aad.canonical_bytes(TRANSCRIPT_ENVELOPE_FORMAT_VERSION)?;
        let bounded_context = bounded_context(context)?;
        let generated = self
            .kms
            .generate_data_key(&self.profile, &bounded_context)
            .await
            .map_err(TranscriptEnvelopeError::from)?;
        if generated.wrapped.profile() != &self.profile {
            return Err(TranscriptEnvelopeError::ProfileMismatch);
        }
        if generated.wrapped.wrapped_bytes().len() > MAX_WRAPPED_DATA_KEY_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }

        let cipher = Aes256Gcm::new_from_slice(generated.plaintext.expose())
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        let mut nonce_bytes = [0u8; AEAD_NONCE_BYTES];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::try_from(nonce_bytes.as_slice())
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: &aad_bytes,
                },
            )
            .map_err(|_| TranscriptEnvelopeError::IntegrityFailure)?;

        let envelope = TranscriptEnvelope {
            format_version: TRANSCRIPT_ENVELOPE_FORMAT_VERSION,
            aad_schema_version: TRANSCRIPT_AAD_SCHEMA_VERSION,
            aead_suite: AEAD_SUITE.to_owned(),
            wrapped_dek: generated.wrapped,
            nonce: BASE64.encode(nonce_bytes),
            ciphertext_and_tag: BASE64.encode(ciphertext),
        };
        envelope.validate()?;
        Ok(envelope)
    }

    /// Validates identity and versions, unwraps the pinned key, then authenticates before return.
    pub async fn decrypt(
        &self,
        context: &KmsCallContext,
        aad: &TranscriptAad,
        envelope: &TranscriptEnvelope,
    ) -> Result<Vec<u8>, TranscriptEnvelopeError> {
        envelope.validate()?;
        if envelope.wrapped_dek.profile() != &self.profile {
            return Err(TranscriptEnvelopeError::ProfileMismatch);
        }
        let aad_bytes = aad.canonical_bytes(envelope.format_version)?;
        let (nonce_bytes, ciphertext) = envelope.decode_parts()?;
        let bounded_context = bounded_context(context)?;
        let data_key = self
            .kms
            .unwrap_data_key(&self.profile, &bounded_context, &envelope.wrapped_dek)
            .await
            .map_err(TranscriptEnvelopeError::from)?;
        let cipher = Aes256Gcm::new_from_slice(data_key.expose())
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        let nonce = Nonce::try_from(nonce_bytes.as_slice())
            .map_err(|_| TranscriptEnvelopeError::InvalidEnvelope)?;
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &ciphertext,
                    aad: &aad_bytes,
                },
            )
            .map_err(|_| TranscriptEnvelopeError::IntegrityFailure)?;
        if plaintext.len() > MAX_TRANSCRIPT_BODY_BYTES {
            return Err(TranscriptEnvelopeError::ResourceLimit);
        }
        Ok(plaintext)
    }
}

fn bounded_context(context: &KmsCallContext) -> Result<KmsCallContext, TranscriptEnvelopeError> {
    let max_response_bytes = context
        .max_response_bytes()
        .min(MAX_WRAPPED_DATA_KEY_BYTES as u32);
    KmsCallContext::new(
        context.operation_id(),
        context.timeout_ms(),
        max_response_bytes,
    )
    .map_err(|_| TranscriptEnvelopeError::InvalidContext)
}

fn is_nil_id(id: &[u8; 16]) -> bool {
    id.iter().all(|byte| *byte == 0)
}

fn is_valid_resource_kind(value: &str) -> bool {
    let bytes = value.as_bytes();
    let Some(first) = bytes.first().copied() else {
        return false;
    };
    let Some(last) = bytes.last().copied() else {
        return false;
    };
    !bytes.is_empty()
        && bytes.len() <= MAX_RESOURCE_KIND_BYTES
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && (last.is_ascii_lowercase() || last.is_ascii_digit())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{KmsCapabilities, KmsProviderDescriptor, ProfileDigest};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    const TEST_KEY: [u8; 32] = [0x5a; 32];

    struct TestProvider {
        descriptor: KmsProviderDescriptor,
        generate_calls: Arc<AtomicUsize>,
        unwrap_calls: Arc<AtomicUsize>,
        discovery_calls: Arc<AtomicUsize>,
        unwrap_failure: Option<KmsProviderErrorKind>,
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
            self.discovery_calls.fetch_add(1, Ordering::SeqCst);
            Ok(KmsCapabilities::new([
                KmsCapability::GenerateDataKey,
                KmsCapability::UnwrapDataKey,
            ]))
        }

        async fn generate_data_key(
            &self,
            profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
        ) -> Result<crate::provider_contract::GeneratedDataKey, KmsProviderError> {
            self.generate_calls.fetch_add(1, Ordering::SeqCst);
            Ok(crate::provider_contract::GeneratedDataKey {
                plaintext: SecretDataKey::from_bytes(TEST_KEY),
                wrapped: WrappedDataKey::new(profile.clone(), "key-v1", vec![0xa5; 32])?,
            })
        }

        async fn wrap_data_key(
            &self,
            profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
            _plaintext: &SecretDataKey,
        ) -> Result<WrappedDataKey, KmsProviderError> {
            WrappedDataKey::new(profile.clone(), "key-v1", vec![0xa5; 32])
        }

        async fn unwrap_data_key(
            &self,
            _profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
            _wrapped: &WrappedDataKey,
        ) -> Result<SecretDataKey, KmsProviderError> {
            self.unwrap_calls.fetch_add(1, Ordering::SeqCst);
            if let Some(kind) = self.unwrap_failure {
                return Err(KmsProviderError::new(kind));
            }
            Ok(SecretDataKey::from_bytes(TEST_KEY))
        }

        async fn rewrap_data_key(
            &self,
            _source_profile: &KmsProfileIdentity,
            target_profile: &KmsProfileIdentity,
            _context: &KmsCallContext,
            _wrapped: &WrappedDataKey,
        ) -> Result<WrappedDataKey, KmsProviderError> {
            WrappedDataKey::new(target_profile.clone(), "key-v2", vec![0xa6; 32])
        }
    }

    struct Fixture {
        protector: TranscriptEnvelopeProtector<TestProvider>,
        aad: TranscriptAad,
        context: KmsCallContext,
        generate_calls: Arc<AtomicUsize>,
        unwrap_calls: Arc<AtomicUsize>,
        discovery_calls: Arc<AtomicUsize>,
        profile: KmsProfileIdentity,
    }

    fn fixture(unwrap_failure: Option<KmsProviderErrorKind>) -> Fixture {
        let profile = KmsProfileIdentity::new(
            KmsProviderDescriptor::new("test-provider", "1.0").unwrap(),
            1,
            "test-endpoint",
            "test-key",
            KmsCapabilities::new([KmsCapability::GenerateDataKey, KmsCapability::UnwrapDataKey]),
            ProfileDigest::from_trusted_resolver([0x33; 32]).unwrap(),
        )
        .unwrap();
        let generate_calls = Arc::new(AtomicUsize::new(0));
        let unwrap_calls = Arc::new(AtomicUsize::new(0));
        let discovery_calls = Arc::new(AtomicUsize::new(0));
        let provider = TestProvider {
            descriptor: KmsProviderDescriptor::new("test-provider", "1.0").unwrap(),
            generate_calls: Arc::clone(&generate_calls),
            unwrap_calls: Arc::clone(&unwrap_calls),
            discovery_calls: Arc::clone(&discovery_calls),
            unwrap_failure,
        };
        let protector =
            TranscriptEnvelopeProtector::new(KmsProviderClient::new(provider), profile.clone())
                .unwrap();
        let aad = TranscriptAad::new(
            [0x11; 16],
            [0x22; 16],
            Some([0x44; 16]),
            [0x33; 16],
            "canvas-draft",
            [0x55; 16],
            [0x66; 16],
            0x0102_0304_0506_0708,
            [0x77; 16],
            0x1112_1314_1516_1718,
            0x2122_2324_2526_2728,
        )
        .unwrap();
        Fixture {
            protector,
            aad,
            context: KmsCallContext::new("test-op", 5_000, 1024 * 1024).unwrap(),
            generate_calls,
            unwrap_calls,
            discovery_calls,
            profile,
        }
    }

    fn aad_with_worktree(worktree_id: Option<[u8; 16]>) -> TranscriptAad {
        TranscriptAad::new(
            [0x11; 16],
            [0x22; 16],
            worktree_id,
            [0x33; 16],
            "canvas-draft",
            [0x55; 16],
            [0x66; 16],
            0x0102_0304_0506_0708,
            [0x77; 16],
            0x1112_1314_1516_1718,
            0x2122_2324_2526_2728,
        )
        .unwrap()
    }

    #[test]
    fn canonical_aad_matches_versioned_field_order_vector() {
        // Vector order: domain, schema, tenant, project, run, worktree tag+UUID,
        // kind length+bytes, resource, actor, base revision, payload,
        // checkpoint revision, retention revision, envelope format.
        let aad = fixture(None).aad;
        let actual = aad
            .canonical_bytes(TRANSCRIPT_ENVELOPE_FORMAT_VERSION)
            .unwrap();
        let mut expected = b"STAR-W16".to_vec();
        expected.extend_from_slice(&1u16.to_be_bytes());
        expected.extend_from_slice(&[0x11; 16]);
        expected.extend_from_slice(&[0x22; 16]);
        expected.extend_from_slice(&[0x33; 16]);
        expected.push(1);
        expected.extend_from_slice(&[0x44; 16]);
        expected.extend_from_slice(&12u16.to_be_bytes());
        expected.extend_from_slice(b"canvas-draft");
        expected.extend_from_slice(&[0x55; 16]);
        expected.extend_from_slice(&[0x66; 16]);
        expected.extend_from_slice(&0x0102_0304_0506_0708u64.to_be_bytes());
        expected.extend_from_slice(&[0x77; 16]);
        expected.extend_from_slice(&0x1112_1314_1516_1718u64.to_be_bytes());
        expected.extend_from_slice(&0x2122_2324_2526_2728u64.to_be_bytes());
        expected.extend_from_slice(&1u32.to_be_bytes());
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 165);
    }

    #[test]
    fn optional_worktree_tag_and_aad_maximum_are_canonical() {
        let absent = aad_with_worktree(None)
            .canonical_bytes(TRANSCRIPT_ENVELOPE_FORMAT_VERSION)
            .unwrap();
        assert_eq!(absent.len(), 149);
        assert_eq!(absent[58], 0);

        let present = aad_with_worktree(Some([0x44; 16]))
            .canonical_bytes(TRANSCRIPT_ENVELOPE_FORMAT_VERSION)
            .unwrap();
        assert_eq!(present.len(), 165);
        assert_eq!(present[58], 1);
        assert_eq!(&present[59..75], &[0x44; 16]);

        let max_kind = TranscriptAad::new(
            [1; 16],
            [2; 16],
            Some([3; 16]),
            [4; 16],
            "a".repeat(MAX_RESOURCE_KIND_BYTES),
            [5; 16],
            [6; 16],
            0,
            [7; 16],
            1,
            1,
        )
        .unwrap()
        .canonical_bytes(TRANSCRIPT_ENVELOPE_FORMAT_VERSION)
        .unwrap();
        assert_eq!(max_kind.len(), MAX_AAD_BYTES);
    }

    #[tokio::test]
    async fn encrypts_and_decrypts_one_bounded_checkpoint_without_discovery() {
        let fixture = fixture(None);
        let plaintext = b"checkpoint body";
        let envelope = fixture
            .protector
            .protect(&fixture.context, &fixture.aad, plaintext)
            .await
            .unwrap();
        let encoded = envelope.to_json().unwrap();
        let decoded = TranscriptEnvelope::from_json(&encoded).unwrap();
        let restored = fixture
            .protector
            .decrypt(&fixture.context, &fixture.aad, &decoded)
            .await
            .unwrap();

        assert_eq!(restored, plaintext);
        assert_eq!(fixture.generate_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.unwrap_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.discovery_calls.load(Ordering::SeqCst), 0);
        assert_eq!(envelope.wrapped_dek().profile(), &fixture.profile);
    }

    #[tokio::test]
    async fn changed_aad_and_ciphertext_tag_fail_authentication() {
        let fixture = fixture(None);
        let envelope = fixture
            .protector
            .protect(&fixture.context, &fixture.aad, b"private body")
            .await
            .unwrap();
        let moved_worktree = aad_with_worktree(Some([0x45; 16]));
        assert_eq!(
            fixture
                .protector
                .decrypt(&fixture.context, &moved_worktree, &envelope)
                .await
                .unwrap_err(),
            TranscriptEnvelopeError::IntegrityFailure
        );

        let mut tampered = envelope.clone();
        let mut ciphertext = BASE64.decode(&tampered.ciphertext_and_tag).unwrap();
        *ciphertext.last_mut().unwrap() ^= 1;
        tampered.ciphertext_and_tag = BASE64.encode(ciphertext);
        assert_eq!(
            fixture
                .protector
                .decrypt(&fixture.context, &fixture.aad, &tampered)
                .await
                .unwrap_err(),
            TranscriptEnvelopeError::IntegrityFailure
        );
    }

    #[tokio::test]
    async fn malformed_versions_and_oversized_inputs_fail_before_provider_calls() {
        let fixture = fixture(None);
        let too_large = vec![b'x'; MAX_TRANSCRIPT_BODY_BYTES + 1];
        assert_eq!(
            fixture
                .protector
                .protect(&fixture.context, &fixture.aad, &too_large)
                .await
                .unwrap_err(),
            TranscriptEnvelopeError::ResourceLimit
        );
        assert_eq!(fixture.generate_calls.load(Ordering::SeqCst), 0);

        assert_eq!(
            TranscriptEnvelope::from_json(&vec![b' '; MAX_TRANSCRIPT_ENVELOPE_JSON_BYTES + 1])
                .unwrap_err(),
            TranscriptEnvelopeError::ResourceLimit
        );
        assert_eq!(
            fixture
                .aad
                .canonical_bytes(TRANSCRIPT_ENVELOPE_FORMAT_VERSION + 1),
            Err(TranscriptEnvelopeError::InvalidEnvelope)
        );

        let envelope = fixture
            .protector
            .protect(&fixture.context, &fixture.aad, b"bounded")
            .await
            .unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&envelope.to_json().unwrap()).unwrap();
        value["format_version"] = serde_json::json!(2);
        let encoded = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            TranscriptEnvelope::from_json(&encoded).unwrap_err(),
            TranscriptEnvelopeError::InvalidEnvelope
        );
    }

    #[tokio::test]
    async fn maximum_plaintext_and_wrapped_key_limits_are_inclusive() {
        let fixture = fixture(None);
        let plaintext = vec![b'x'; MAX_TRANSCRIPT_BODY_BYTES];
        let mut envelope = fixture
            .protector
            .protect(&fixture.context, &fixture.aad, &plaintext)
            .await
            .unwrap();
        assert_eq!(
            BASE64.decode(&envelope.ciphertext_and_tag).unwrap().len(),
            MAX_CIPHERTEXT_BYTES
        );

        envelope.wrapped_dek = WrappedDataKey::new(
            fixture.profile.clone(),
            "key-v1",
            vec![0xa5; MAX_WRAPPED_DATA_KEY_BYTES],
        )
        .unwrap();
        assert!(envelope.to_json().is_ok());

        envelope.wrapped_dek = WrappedDataKey::new(
            fixture.profile,
            "key-v1",
            vec![0xa5; MAX_WRAPPED_DATA_KEY_BYTES + 1],
        )
        .unwrap();
        assert_eq!(
            envelope.to_json().unwrap_err(),
            TranscriptEnvelopeError::ResourceLimit
        );
    }

    #[tokio::test]
    async fn provider_failure_is_safe_and_profile_mismatch_fails_closed() {
        let fixture = fixture(Some(KmsProviderErrorKind::TemporarilyUnavailable));
        let envelope = fixture
            .protector
            .protect(&fixture.context, &fixture.aad, b"private")
            .await
            .unwrap();
        let failure = fixture
            .protector
            .decrypt(&fixture.context, &fixture.aad, &envelope)
            .await
            .unwrap_err();
        assert_eq!(
            failure,
            TranscriptEnvelopeError::Provider(KmsProviderErrorKind::TemporarilyUnavailable)
        );
        assert!(!failure.to_string().contains("private"));

        let mut wrong_profile = envelope;
        wrong_profile.wrapped_dek = WrappedDataKey::new(
            KmsProfileIdentity::new(
                KmsProviderDescriptor::new("other-provider", "1.0").unwrap(),
                1,
                "other-endpoint",
                "other-key",
                KmsCapabilities::new([
                    KmsCapability::GenerateDataKey,
                    KmsCapability::UnwrapDataKey,
                ]),
                ProfileDigest::from_trusted_resolver([0x34; 32]).unwrap(),
            )
            .unwrap(),
            "key-v1",
            vec![0xa5; 32],
        )
        .unwrap();
        assert_eq!(
            fixture
                .protector
                .decrypt(&fixture.context, &fixture.aad, &wrong_profile)
                .await
                .unwrap_err(),
            TranscriptEnvelopeError::ProfileMismatch
        );
    }

    #[tokio::test]
    async fn serialized_envelope_and_errors_do_not_contain_plaintext_or_data_key() {
        let fixture = fixture(None);
        let plaintext = b"plaintext-sentinel-never-serialized";
        let envelope = fixture
            .protector
            .protect(&fixture.context, &fixture.aad, plaintext)
            .await
            .unwrap();
        let encoded = envelope.to_json().unwrap();
        let encoded_text = String::from_utf8(encoded).unwrap();
        assert!(!encoded_text.contains("plaintext-sentinel-never-serialized"));
        let decimal_key = format!(
            "[{}]",
            vec![90; 32]
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(!encoded_text.contains(&decimal_key));
        assert!(!format!("{envelope:?}").contains("plaintext-sentinel"));
        assert_eq!(
            format!("{:?}", SecretDataKey::from_bytes(TEST_KEY)),
            "SecretDataKey([redacted])"
        );
    }
}
