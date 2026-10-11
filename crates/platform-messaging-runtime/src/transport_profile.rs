// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=365f852049cd79f70987e89d4053b3539970e0530a4e39cfe5ec061681526574
MERGE (self:File {path:"crates/platform-messaging-runtime/src/transport_profile.rs"})
MERGE (kind:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportMessageKind",kind:"enum"})
MERGE (lifecycle:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileLifecycle",kind:"enum"})
MERGE (direction:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportDirection",kind:"enum"})
MERGE (secret:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::OpaqueSecretReference",kind:"struct"})
MERGE (definition:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileDefinition",kind:"struct"})
MERGE (revision:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision",kind:"struct"})
MERGE (session:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::AuthenticatedTransportSession",kind:"trait"})
MERGE (context:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::AuthenticatedTransportContext",kind:"struct"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::ProfileDefinitionError",kind:"enum"})
MERGE (secret_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::OpaqueSecretReference.new",kind:"method"})
MERGE (definition_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileDefinition.new",kind:"method"})
MERGE (digest:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileDefinition.digest",kind:"method"})
MERGE (revision_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision.new",kind:"method"})
MERGE (context_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::AuthenticatedTransportContext.from_session",kind:"method"})
MERGE (authorize:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision.authorize",kind:"method"})
MERGE (valid_label:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::valid_label",kind:"function"})
MERGE (put_bytes:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::put_bytes",kind:"function"})
MERGE (put_text:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::put_text",kind:"function"})
MERGE (put_ids:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::put_ids",kind:"function"})
MERGE (test_digest:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::tests.profile_digest_is_canonical_and_revision_bound",kind:"test"})
MERGE (test_secret:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::tests.secret_reference_is_opaque_and_debug_redacted",kind:"test"})
MERGE (test_scope:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::tests.profile_authorization_checks_provider_endpoint_and_route",kind:"test"})
MERGE (self)-[:DEFINES]->(kind)
MERGE (self)-[:DEFINES]->(lifecycle)
MERGE (self)-[:DEFINES]->(direction)
MERGE (self)-[:DEFINES]->(secret)
MERGE (self)-[:DEFINES]->(definition)
MERGE (self)-[:DEFINES]->(revision)
MERGE (self)-[:DEFINES]->(session)
MERGE (self)-[:DEFINES]->(context)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(secret_new)
MERGE (self)-[:DEFINES]->(definition_new)
MERGE (self)-[:DEFINES]->(digest)
MERGE (self)-[:DEFINES]->(revision_new)
MERGE (self)-[:DEFINES]->(context_new)
MERGE (self)-[:DEFINES]->(authorize)
MERGE (self)-[:DEFINES]->(valid_label)
MERGE (self)-[:DEFINES]->(put_bytes)
MERGE (self)-[:DEFINES]->(put_text)
MERGE (self)-[:DEFINES]->(put_ids)
MERGE (self)-[:DEFINES]->(test_digest)
MERGE (self)-[:DEFINES]->(test_secret)
MERGE (self)-[:DEFINES]->(test_scope)
MERGE (secret_new)-[:CALLS]->(valid_label)
MERGE (definition_new)-[:CALLS]->(valid_label)
MERGE (digest)-[:CALLS]->(put_bytes)
MERGE (digest)-[:CALLS]->(put_text)
MERGE (digest)-[:CALLS]->(put_ids)
MERGE (revision_new)-[:CALLS]->(digest)
MERGE (context_new)-[:CALLS]->(valid_label)
MERGE (authorize)-[:USES_TYPE]->(context)
MERGE (authorize)-[:USES_TYPE]->(kind)
MERGE (test_digest)-[:TESTS]->(digest)
MERGE (test_secret)-[:TESTS]->(secret_new)
MERGE (test_scope)-[:TESTS]->(authorize)
@endcypher
*/
//! Bounded, versioned transport profile facts and authenticated route binding.

use std::fmt;

use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

/// Maximum sender or recipient identities held in one profile.
pub const MAX_PROFILE_SERVICE_IDENTITIES: usize = 32;
/// Maximum characters in a durable log generation policy key.
pub const MAX_LOG_GENERATION_POLICY_BYTES: usize = 64;

/// Admission message kind permitted by a transport profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransportMessageKind {
    /// Schedule-to-Run preparation request.
    Request,
    /// Run-to-Schedule preparation result.
    Result,
    /// Schedule-to-Run confirmation or compensation.
    Resolution,
}

impl TransportMessageKind {
    /// Stable database and Outbox representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Result => "result",
            Self::Resolution => "resolution",
        }
    }

    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "request" => Some(Self::Request),
            "result" => Some(Self::Result),
            "resolution" => Some(Self::Resolution),
            _ => None,
        }
    }
}

/// Lifecycle of the current immutable profile revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportProfileLifecycle {
    /// New deliveries may resolve this revision.
    Active,
    /// The revision remains visible for audit but cannot receive new claims.
    Disabled,
    /// The profile has been revoked and cannot receive new claims.
    Revoked,
}

impl TransportProfileLifecycle {
    /// Stable database representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }

    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }
}

/// Direction of a transport operation relative to the local authenticated service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportDirection {
    /// The local service is the message sender.
    Publish,
    /// The local service is the message recipient.
    Receive,
}

/// Secret-manager reference; this type can never contain credential bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct OpaqueSecretReference(String);

impl OpaqueSecretReference {
    /// Validate and store an opaque secret-manager reference.
    pub fn new(value: String) -> Result<Self, ProfileDefinitionError> {
        let Some(locator) = value.strip_prefix("secretref:") else {
            return Err(ProfileDefinitionError::InvalidSecretReference);
        };
        if !valid_label(locator, 192, b"._/-")
            || !locator
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(ProfileDefinitionError::InvalidSecretReference);
        }
        Ok(Self(value))
    }

    /// Return the reference for a trusted secret-manager lookup.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for OpaqueSecretReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OpaqueSecretReference([redacted])")
    }
}

/// Immutable provider configuration covered by the profile digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportProfileDefinition {
    provider_id: String,
    provider_version: String,
    endpoint_identity: String,
    log_generation_policy: String,
    allowed_sender_service_ids: Vec<Uuid>,
    allowed_recipient_service_ids: Vec<Uuid>,
    allowed_message_kinds: Vec<TransportMessageKind>,
    secret_reference: Option<OpaqueSecretReference>,
}

impl TransportProfileDefinition {
    /// Build a bounded profile definition and normalize set-like scope fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: String,
        provider_version: String,
        endpoint_identity: String,
        log_generation_policy: String,
        mut allowed_sender_service_ids: Vec<Uuid>,
        mut allowed_recipient_service_ids: Vec<Uuid>,
        mut allowed_message_kinds: Vec<TransportMessageKind>,
        secret_reference: Option<OpaqueSecretReference>,
    ) -> Result<Self, ProfileDefinitionError> {
        if !valid_label(&provider_id, 64, b"-")
            || !provider_id
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_lowercase())
            || provider_id != provider_id.to_ascii_lowercase()
        {
            return Err(ProfileDefinitionError::InvalidProvider);
        }
        if !valid_label(&provider_version, 64, b"._+-")
            || !provider_version
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(ProfileDefinitionError::InvalidProviderVersion);
        }
        if !valid_label(&endpoint_identity, 128, b"._:-")
            || !endpoint_identity
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(ProfileDefinitionError::InvalidEndpointIdentity);
        }
        if !valid_label(
            &log_generation_policy,
            MAX_LOG_GENERATION_POLICY_BYTES,
            b"._-",
        ) || !log_generation_policy
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase())
            || log_generation_policy != log_generation_policy.to_ascii_lowercase()
        {
            return Err(ProfileDefinitionError::InvalidGenerationPolicy);
        }
        normalize_identities(&mut allowed_sender_service_ids)?;
        normalize_identities(&mut allowed_recipient_service_ids)?;
        if allowed_message_kinds.is_empty() || allowed_message_kinds.len() > 3 {
            return Err(ProfileDefinitionError::InvalidMessageKinds);
        }
        allowed_message_kinds.sort_unstable();
        allowed_message_kinds.dedup();
        Ok(Self {
            provider_id,
            provider_version,
            endpoint_identity,
            log_generation_policy,
            allowed_sender_service_ids,
            allowed_recipient_service_ids,
            allowed_message_kinds,
            secret_reference,
        })
    }

    /// Stable provider key.
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Exact provider protocol or build contract version.
    pub fn provider_version(&self) -> &str {
        &self.provider_version
    }

    /// Non-secret, stable endpoint label.
    pub fn endpoint_identity(&self) -> &str {
        &self.endpoint_identity
    }

    /// Versioned policy for identifying a durable provider log generation.
    pub fn log_generation_policy(&self) -> &str {
        &self.log_generation_policy
    }

    /// Allow-listed authenticated sender identities.
    pub fn allowed_sender_service_ids(&self) -> &[Uuid] {
        &self.allowed_sender_service_ids
    }

    /// Allow-listed authenticated recipient identities.
    pub fn allowed_recipient_service_ids(&self) -> &[Uuid] {
        &self.allowed_recipient_service_ids
    }

    /// Allow-listed message kinds.
    pub fn allowed_message_kinds(&self) -> &[TransportMessageKind] {
        &self.allowed_message_kinds
    }

    /// Optional opaque reference resolved by the trusted secret manager.
    pub fn secret_reference(&self) -> Option<&OpaqueSecretReference> {
        self.secret_reference.as_ref()
    }

    /// Hash the exact deployment/profile/revision tuple and bounded typed configuration.
    pub fn digest(
        &self,
        deployment_id: Uuid,
        tenant_id: Uuid,
        profile_id: Uuid,
        revision: i64,
        lifecycle: TransportProfileLifecycle,
    ) -> Result<[u8; 32], ProfileDefinitionError> {
        if deployment_id.is_nil() || tenant_id.is_nil() || profile_id.is_nil() || revision <= 0 {
            return Err(ProfileDefinitionError::InvalidIdentity);
        }

        let mut canonical = Vec::with_capacity(1024);
        canonical.extend_from_slice(b"STAR-TRANSPORT-PROFILE-V1\0");
        put_bytes(&mut canonical, deployment_id.as_bytes());
        put_bytes(&mut canonical, tenant_id.as_bytes());
        put_bytes(&mut canonical, profile_id.as_bytes());
        put_bytes(&mut canonical, &revision.to_be_bytes());
        canonical.push(match lifecycle {
            TransportProfileLifecycle::Active => 1,
            TransportProfileLifecycle::Disabled => 2,
            TransportProfileLifecycle::Revoked => 3,
        });
        canonical.extend_from_slice(&1_i16.to_be_bytes());
        put_text(&mut canonical, &self.provider_id);
        put_text(&mut canonical, &self.provider_version);
        put_text(&mut canonical, &self.endpoint_identity);
        put_text(&mut canonical, &self.log_generation_policy);
        put_ids(&mut canonical, &self.allowed_sender_service_ids);
        put_ids(&mut canonical, &self.allowed_recipient_service_ids);
        canonical.push(self.allowed_message_kinds.len() as u8);
        for kind in &self.allowed_message_kinds {
            canonical.push(match kind {
                TransportMessageKind::Request => 1,
                TransportMessageKind::Result => 2,
                TransportMessageKind::Resolution => 3,
            });
        }
        match &self.secret_reference {
            Some(reference) => {
                canonical.push(1);
                put_text(&mut canonical, reference.as_str());
            }
            None => canonical.push(0),
        }
        Ok(Sha256::digest(canonical).into())
    }
}

/// One immutable, digest-checked revision returned by a trusted resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportProfileRevision {
    deployment_id: Uuid,
    tenant_id: Uuid,
    profile_id: Uuid,
    revision: i64,
    lifecycle: TransportProfileLifecycle,
    definition: TransportProfileDefinition,
    digest: [u8; 32],
}

impl TransportProfileRevision {
    /// Construct a revision and calculate its canonical digest.
    pub fn new(
        deployment_id: Uuid,
        tenant_id: Uuid,
        profile_id: Uuid,
        revision: i64,
        lifecycle: TransportProfileLifecycle,
        definition: TransportProfileDefinition,
    ) -> Result<Self, ProfileDefinitionError> {
        let digest =
            definition.digest(deployment_id, tenant_id, profile_id, revision, lifecycle)?;
        Ok(Self {
            deployment_id,
            tenant_id,
            profile_id,
            revision,
            lifecycle,
            definition,
            digest,
        })
    }

    /// Profile owner deployment.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    /// Profile tenant.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Stable profile identity.
    pub fn profile_id(&self) -> Uuid {
        self.profile_id
    }

    /// Monotonic immutable revision number.
    pub fn revision(&self) -> i64 {
        self.revision
    }

    /// Current or historical lifecycle state.
    pub fn lifecycle(&self) -> TransportProfileLifecycle {
        self.lifecycle
    }

    /// Immutable non-secret configuration.
    pub fn definition(&self) -> &TransportProfileDefinition {
        &self.definition
    }

    /// SHA-256 over the canonical revision bytes.
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Verify the session provider and the configured sender/recipient/message-kind tuple.
    pub fn authorize(
        &self,
        context: &AuthenticatedTransportContext,
        kind: TransportMessageKind,
    ) -> Result<(), ProfileAuthorizationError> {
        if self.lifecycle != TransportProfileLifecycle::Active {
            return Err(ProfileAuthorizationError::Inactive);
        }
        if self.deployment_id != context.deployment_id
            || self.tenant_id != context.tenant_id
            || self.definition.provider_id != context.provider_id
            || self.definition.provider_version != context.provider_version
            || self.definition.endpoint_identity != context.endpoint_identity
        {
            return Err(ProfileAuthorizationError::IdentityMismatch);
        }
        let (sender_service_id, recipient_service_id) = context.sender_recipient();
        if !self
            .definition
            .allowed_sender_service_ids
            .contains(&sender_service_id)
            || !self
                .definition
                .allowed_recipient_service_ids
                .contains(&recipient_service_id)
            || !self.definition.allowed_message_kinds.contains(&kind)
        {
            return Err(ProfileAuthorizationError::ScopeDenied);
        }
        Ok(())
    }
}

/// Contract implemented only by a transport session after provider/service authentication.
///
/// Implementations must derive deployment, endpoint and service identities from trusted
/// connection configuration and authenticated peer evidence. They must never copy those
/// identities from an admission message body or user-controlled routing field.
pub trait AuthenticatedTransportSession {
    /// Trusted deployment identity.
    fn deployment_id(&self) -> Uuid;
    /// Trusted tenant assigned to this channel.
    fn tenant_id(&self) -> Uuid;
    /// Authenticated provider key.
    fn provider_id(&self) -> &str;
    /// Authenticated provider protocol version.
    fn provider_version(&self) -> &str;
    /// Trusted non-secret endpoint label.
    fn endpoint_identity(&self) -> &str;
    /// Local service identity from the trusted deployment binding.
    fn local_service_id(&self) -> Uuid;
    /// Remote service identity verified by the authenticated transport.
    fn peer_service_id(&self) -> Uuid;
    /// Direction of the operation relative to the local service.
    fn direction(&self) -> TransportDirection;
}

/// Validated identity and route metadata extracted from an authenticated transport session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedTransportContext {
    deployment_id: Uuid,
    tenant_id: Uuid,
    provider_id: String,
    provider_version: String,
    endpoint_identity: String,
    local_service_id: Uuid,
    peer_service_id: Uuid,
    direction: TransportDirection,
}

impl AuthenticatedTransportContext {
    /// Validate the trusted transport session before any profile lookup.
    pub fn from_session<S: AuthenticatedTransportSession>(
        session: &S,
    ) -> Result<Self, ProfileDefinitionError> {
        if session.deployment_id().is_nil()
            || session.tenant_id().is_nil()
            || session.local_service_id().is_nil()
            || session.peer_service_id().is_nil()
            || session.local_service_id() == session.peer_service_id()
            || !valid_label(session.provider_id(), 64, b"-")
            || !session
                .provider_id()
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_lowercase())
            || session.provider_id() != session.provider_id().to_ascii_lowercase()
            || !valid_label(session.provider_version(), 64, b"._+-")
            || !session
                .provider_version()
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
            || !valid_label(session.endpoint_identity(), 128, b"._:-")
            || !session
                .endpoint_identity()
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(ProfileDefinitionError::InvalidIdentity);
        }
        Ok(Self {
            deployment_id: session.deployment_id(),
            tenant_id: session.tenant_id(),
            provider_id: session.provider_id().to_owned(),
            provider_version: session.provider_version().to_owned(),
            endpoint_identity: session.endpoint_identity().to_owned(),
            local_service_id: session.local_service_id(),
            peer_service_id: session.peer_service_id(),
            direction: session.direction(),
        })
    }

    /// Deployment used as the first database scope.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    /// Tenant used as the second database scope.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Stable provider key supplied by the authenticated session.
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Provider version supplied by the authenticated session.
    pub fn provider_version(&self) -> &str {
        &self.provider_version
    }

    /// Non-secret endpoint label supplied by trusted session configuration.
    pub fn endpoint_identity(&self) -> &str {
        &self.endpoint_identity
    }

    fn sender_recipient(&self) -> (Uuid, Uuid) {
        match self.direction {
            TransportDirection::Publish => (self.local_service_id, self.peer_service_id),
            TransportDirection::Receive => (self.peer_service_id, self.local_service_id),
        }
    }
}

/// Invalid immutable profile or trusted session identity.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProfileDefinitionError {
    /// An identity or revision was empty or outside the permitted range.
    #[error("invalid transport profile identity")]
    InvalidIdentity,
    /// The provider key is invalid.
    #[error("invalid transport provider key")]
    InvalidProvider,
    /// The provider version is invalid.
    #[error("invalid transport provider version")]
    InvalidProviderVersion,
    /// The endpoint identity is invalid or resembles an endpoint URI.
    #[error("invalid transport endpoint identity")]
    InvalidEndpointIdentity,
    /// The log-generation policy key is invalid.
    #[error("invalid log-generation policy")]
    InvalidGenerationPolicy,
    /// The sender/recipient identity scope is invalid.
    #[error("invalid transport service identity scope")]
    InvalidServiceScope,
    /// The message-kind scope is invalid.
    #[error("invalid transport message-kind scope")]
    InvalidMessageKinds,
    /// The value is not an opaque secret-manager reference.
    #[error("invalid opaque secret reference")]
    InvalidSecretReference,
}

/// A trusted provider or service identity did not match the profile scope.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProfileAuthorizationError {
    /// A disabled or revoked profile cannot receive new deliveries.
    #[error("transport profile is inactive")]
    Inactive,
    /// Deployment, tenant, provider, version or endpoint identity differs.
    #[error("authenticated transport identity does not match the profile")]
    IdentityMismatch,
    /// The authenticated sender, recipient or message kind is not allowed.
    #[error("authenticated route is outside the profile scope")]
    ScopeDenied,
}

fn normalize_identities(values: &mut Vec<Uuid>) -> Result<(), ProfileDefinitionError> {
    if values.is_empty()
        || values.len() > MAX_PROFILE_SERVICE_IDENTITIES
        || values.iter().any(Uuid::is_nil)
    {
        return Err(ProfileDefinitionError::InvalidServiceScope);
    }
    values.sort_unstable();
    values.dedup();
    if values.is_empty() {
        return Err(ProfileDefinitionError::InvalidServiceScope);
    }
    Ok(())
}

fn valid_label(value: &str, max_bytes: usize, extra: &[u8]) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || extra.contains(&byte))
}

fn put_bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u32).to_be_bytes());
    output.extend_from_slice(value);
}

fn put_text(output: &mut Vec<u8>, value: &str) {
    put_bytes(output, value.as_bytes());
}

fn put_ids(output: &mut Vec<u8>, values: &[Uuid]) {
    output.extend_from_slice(&(values.len() as u16).to_be_bytes());
    for value in values {
        output.extend_from_slice(value.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> TransportProfileDefinition {
        TransportProfileDefinition::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            "run-admission".to_owned(),
            "provider-log-generation-v1".to_owned(),
            vec![Uuid::from_u128(2), Uuid::from_u128(1)],
            vec![Uuid::from_u128(4), Uuid::from_u128(3)],
            vec![TransportMessageKind::Result, TransportMessageKind::Request],
            Some(
                OpaqueSecretReference::new("secretref:platform/admission-client".to_owned())
                    .unwrap(),
            ),
        )
        .unwrap()
    }

    struct TestSession {
        direction: TransportDirection,
        provider_id: &'static str,
        provider_version: &'static str,
        endpoint_identity: &'static str,
    }

    impl AuthenticatedTransportSession for TestSession {
        fn deployment_id(&self) -> Uuid {
            Uuid::from_u128(10)
        }

        fn tenant_id(&self) -> Uuid {
            Uuid::from_u128(11)
        }

        fn provider_id(&self) -> &str {
            self.provider_id
        }

        fn provider_version(&self) -> &str {
            self.provider_version
        }

        fn endpoint_identity(&self) -> &str {
            self.endpoint_identity
        }

        fn local_service_id(&self) -> Uuid {
            Uuid::from_u128(1)
        }

        fn peer_service_id(&self) -> Uuid {
            Uuid::from_u128(3)
        }

        fn direction(&self) -> TransportDirection {
            self.direction
        }
    }

    #[test]
    fn profile_digest_is_canonical_and_revision_bound() {
        let profile = definition();
        let reordered = TransportProfileDefinition::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            "run-admission".to_owned(),
            "provider-log-generation-v1".to_owned(),
            vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            vec![Uuid::from_u128(3), Uuid::from_u128(4)],
            vec![TransportMessageKind::Request, TransportMessageKind::Result],
            Some(
                OpaqueSecretReference::new("secretref:platform/admission-client".to_owned())
                    .unwrap(),
            ),
        )
        .unwrap();
        let identity = (
            Uuid::from_u128(10),
            Uuid::from_u128(11),
            Uuid::from_u128(12),
        );
        let digest = profile
            .digest(
                identity.0,
                identity.1,
                identity.2,
                1,
                TransportProfileLifecycle::Active,
            )
            .unwrap();
        assert_eq!(
            digest,
            reordered
                .digest(
                    identity.0,
                    identity.1,
                    identity.2,
                    1,
                    TransportProfileLifecycle::Active
                )
                .unwrap()
        );
        assert_ne!(
            digest,
            profile
                .digest(
                    identity.0,
                    identity.1,
                    identity.2,
                    2,
                    TransportProfileLifecycle::Active
                )
                .unwrap()
        );
        let definition_with_fields =
            |provider_id: &str, provider_version: &str, endpoint: &str, policy: &str| {
                TransportProfileDefinition::new(
                    provider_id.to_owned(),
                    provider_version.to_owned(),
                    endpoint.to_owned(),
                    policy.to_owned(),
                    vec![Uuid::from_u128(1)],
                    vec![Uuid::from_u128(2)],
                    vec![TransportMessageKind::Request],
                    None,
                )
            };
        assert!(matches!(
            definition_with_fields("1provider", "1.0.0", "run-admission", "provider-log-v1"),
            Err(ProfileDefinitionError::InvalidProvider)
        ));
        assert!(matches!(
            definition_with_fields("star-provider", ".1.0", "run-admission", "provider-log-v1"),
            Err(ProfileDefinitionError::InvalidProviderVersion)
        ));
        assert!(matches!(
            definition_with_fields("star-provider", "1.0", ":run-admission", "provider-log-v1"),
            Err(ProfileDefinitionError::InvalidEndpointIdentity)
        ));
        assert!(matches!(
            definition_with_fields("star-provider", "1.0", "run-admission", "1provider-log-v1"),
            Err(ProfileDefinitionError::InvalidGenerationPolicy)
        ));
        assert_ne!(
            digest,
            profile
                .digest(
                    identity.0,
                    identity.1,
                    identity.2,
                    1,
                    TransportProfileLifecycle::Revoked
                )
                .unwrap()
        );
    }

    #[test]
    fn secret_reference_is_opaque_and_debug_redacted() {
        let reference =
            OpaqueSecretReference::new("secretref:platform/admission-client".to_owned()).unwrap();
        assert_eq!(
            format!("{reference:?}"),
            "OpaqueSecretReference([redacted])"
        );
        assert!(OpaqueSecretReference::new("secretref:https://host/token".to_owned()).is_err());
        assert!(OpaqueSecretReference::new("secretref:_platform/token".to_owned()).is_err());
        assert!(OpaqueSecretReference::new("password=very-secret".to_owned()).is_err());
    }

    #[test]
    fn profile_authorization_checks_provider_endpoint_and_route() {
        let context = AuthenticatedTransportContext::from_session(&TestSession {
            direction: TransportDirection::Publish,
            provider_id: "star-rust-log",
            provider_version: "1.0.0",
            endpoint_identity: "run-admission",
        })
        .unwrap();
        let profile = TransportProfileRevision::new(
            Uuid::from_u128(10),
            Uuid::from_u128(11),
            Uuid::from_u128(12),
            1,
            TransportProfileLifecycle::Active,
            definition(),
        )
        .unwrap();
        assert_eq!(
            profile.authorize(&context, TransportMessageKind::Request),
            Ok(())
        );
        assert_eq!(
            profile.authorize(&context, TransportMessageKind::Resolution),
            Err(ProfileAuthorizationError::ScopeDenied)
        );
        let receive_context = AuthenticatedTransportContext::from_session(&TestSession {
            direction: TransportDirection::Receive,
            provider_id: "star-rust-log",
            provider_version: "1.0.0",
            endpoint_identity: "run-admission",
        })
        .unwrap();
        assert_eq!(
            profile.authorize(&receive_context, TransportMessageKind::Request),
            Err(ProfileAuthorizationError::ScopeDenied)
        );
        for invalid_session in [
            TestSession {
                direction: TransportDirection::Publish,
                provider_id: "1star-rust-log",
                provider_version: "1.0.0",
                endpoint_identity: "run-admission",
            },
            TestSession {
                direction: TransportDirection::Publish,
                provider_id: "star-rust-log",
                provider_version: ".1.0",
                endpoint_identity: "run-admission",
            },
            TestSession {
                direction: TransportDirection::Publish,
                provider_id: "star-rust-log",
                provider_version: "1.0.0",
                endpoint_identity: ":run-admission",
            },
        ] {
            assert_eq!(
                AuthenticatedTransportContext::from_session(&invalid_session),
                Err(ProfileDefinitionError::InvalidIdentity)
            );
        }
        let revoked = TransportProfileRevision::new(
            Uuid::from_u128(10),
            Uuid::from_u128(11),
            Uuid::from_u128(12),
            2,
            TransportProfileLifecycle::Revoked,
            definition(),
        )
        .unwrap();
        assert_eq!(
            revoked.authorize(&context, TransportMessageKind::Request),
            Err(ProfileAuthorizationError::Inactive)
        );
    }
}
