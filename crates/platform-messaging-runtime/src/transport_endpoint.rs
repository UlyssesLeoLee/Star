//! Resolves immutable transport profile identities through deployment-trusted endpoint data.
/*
@cypher schema=1 source_sha256=30da8d44605c6a7480b381c053200163aaf399d40b246f7f78f2efad5cb685b2
MERGE (self:File {path:"crates/platform-messaging-runtime/src/transport_endpoint.rs"})
MERGE (module:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::transport_endpoint",kind:"module"})
MERGE (config:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointConfig",kind:"struct"})
MERGE (resolver:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver",kind:"struct"})
MERGE (resolved:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint",kind:"struct"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::EndpointResolutionError",kind:"enum"})
MERGE (config_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointConfig.new",kind:"method"})
MERGE (config_debug:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointConfig.fmt",kind:"method"})
MERGE (resolver_new:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.new",kind:"method"})
MERGE (resolve:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.resolve",kind:"method"})
MERGE (resolved_debug:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.fmt",kind:"method"})
MERGE (valid_identity:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::valid_endpoint_identity",kind:"function"})
MERGE (validate_tls:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::validate_tls_server_name",kind:"function"})
MERGE (validate_uri:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::validate_https_uri",kind:"function"})
MERGE (validate_path:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::valid_uri_path",kind:"function"})
MERGE (canonical_key:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::canonical_destination_key",kind:"function"})
MERGE (valid_dns:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::valid_dns_name",kind:"function"})
MERGE (profile:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision",kind:"struct"})
MERGE (profile_lifecycle:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileLifecycle",kind:"enum"})
MERGE (profile_definition:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileDefinition",kind:"struct"})
MERGE (profile_secret:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::OpaqueSecretReference",kind:"struct"})
MERGE (collections:Type {id:"std::collections::{BTreeMap,BTreeSet}",kind:"external_type"})
MERGE (uuid:Type {id:"uuid::Uuid",kind:"external_type"})
MERGE (thiserror:Type {id:"thiserror::Error",kind:"external_type"})
MERGE (test_valid:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.active_profile_resolves_to_its_trusted_endpoint",kind:"test"})
MERGE (test_unknown:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.unknown_endpoint_identity_is_rejected",kind:"test"})
MERGE (test_duplicate:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.duplicate_identities_and_destinations_are_rejected",kind:"test"})
MERGE (test_lifecycle:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.disabled_and_revoked_profiles_are_rejected",kind:"test"})
MERGE (test_mismatch:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.deployment_mismatch_is_rejected",kind:"test"})
MERGE (test_malformed:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.malformed_uris_and_tls_names_are_rejected",kind:"test"})
MERGE (test_bound:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.endpoint_registry_is_bounded",kind:"test"})
MERGE (test_debug:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::tests.debug_output_redacts_endpoint_uri_and_secret_reference",kind:"test"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(config)
MERGE (module)-[:DEFINES]->(resolver)
MERGE (module)-[:DEFINES]->(resolved)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(config_new)
MERGE (module)-[:DEFINES]->(config_debug)
MERGE (module)-[:DEFINES]->(resolver_new)
MERGE (module)-[:DEFINES]->(resolve)
MERGE (module)-[:DEFINES]->(resolved_debug)
MERGE (module)-[:DEFINES]->(valid_identity)
MERGE (module)-[:DEFINES]->(validate_tls)
MERGE (module)-[:DEFINES]->(validate_uri)
MERGE (module)-[:DEFINES]->(validate_path)
MERGE (module)-[:DEFINES]->(canonical_key)
MERGE (module)-[:DEFINES]->(valid_dns)
MERGE (self)-[:IMPORTS]->(profile)
MERGE (self)-[:IMPORTS]->(collections)
MERGE (self)-[:IMPORTS]->(uuid)
MERGE (self)-[:IMPORTS]->(thiserror)
MERGE (config_new)-[:CALLS]->(valid_identity)
MERGE (config_new)-[:CALLS]->(validate_tls)
MERGE (config_new)-[:CALLS]->(validate_uri)
MERGE (resolver_new)-[:CALLS]->(canonical_key)
MERGE (resolve)-[:USES_TYPE]->(profile)
MERGE (resolve)-[:USES_TYPE]->(profile_lifecycle)
MERGE (resolve)-[:USES_TYPE]->(profile_definition)
MERGE (resolve)-[:USES_TYPE]->(profile_secret)
MERGE (resolve)-[:READS]->(profile)
MERGE (validate_uri)-[:CALLS]->(validate_path)
MERGE (validate_uri)-[:CALLS]->(valid_dns)
MERGE (validate_tls)-[:CALLS]->(valid_dns)
MERGE (canonical_key)-[:CALLS]->(validate_uri)
MERGE (module)-[:DEFINES]->(test_valid)
MERGE (module)-[:DEFINES]->(test_unknown)
MERGE (module)-[:DEFINES]->(test_duplicate)
MERGE (module)-[:DEFINES]->(test_lifecycle)
MERGE (module)-[:DEFINES]->(test_mismatch)
MERGE (module)-[:DEFINES]->(test_malformed)
MERGE (module)-[:DEFINES]->(test_bound)
MERGE (test_valid)-[:TESTS]->(resolve)
MERGE (test_unknown)-[:TESTS]->(resolve)
MERGE (test_duplicate)-[:TESTS]->(resolver_new)
MERGE (test_lifecycle)-[:TESTS]->(resolve)
MERGE (test_mismatch)-[:TESTS]->(resolve)
MERGE (test_malformed)-[:TESTS]->(config_new)
MERGE (test_bound)-[:TESTS]->(resolver_new)
MERGE (test_debug)-[:TESTS]->(config_debug)
MERGE (test_debug)-[:TESTS]->(resolved_debug)
@endcypher
*/

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use uuid::Uuid;

use crate::transport_profile::{
    OpaqueSecretReference, TransportProfileLifecycle, TransportProfileRevision,
};

/// Maximum number of deployment-trusted transport endpoints held by one resolver.
pub const MAX_TRUSTED_ENDPOINTS: usize = 64;
/// Maximum serialized HTTPS endpoint URI size.
pub const MAX_HTTPS_URI_BYTES: usize = 2_048;
/// Maximum DNS TLS server-name size.
pub const MAX_TLS_SERVER_NAME_BYTES: usize = 253;

/// One endpoint entry supplied by deployment-owned configuration, never by a request.
#[derive(Clone, PartialEq, Eq)]
pub struct TrustedEndpointConfig {
    endpoint_identity: String,
    https_uri: String,
    tls_server_name: String,
}

impl std::fmt::Debug for TrustedEndpointConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrustedEndpointConfig")
            .field("endpoint_identity", &self.endpoint_identity)
            .field("host", &self.tls_server_name)
            .finish_non_exhaustive()
    }
}

impl TrustedEndpointConfig {
    /// Validate one fixed HTTPS destination and its TLS certificate name.
    pub fn new(
        endpoint_identity: String,
        https_uri: String,
        tls_server_name: String,
    ) -> Result<Self, EndpointResolutionError> {
        if !valid_endpoint_identity(&endpoint_identity) {
            return Err(EndpointResolutionError::InvalidEndpointIdentity);
        }
        let tls_server_name = validate_tls_server_name(&tls_server_name)?;
        validate_https_uri(&https_uri, &tls_server_name)?;
        Ok(Self {
            endpoint_identity,
            https_uri,
            tls_server_name,
        })
    }
}

/// Deployment-scoped allow-list from immutable endpoint identities to fixed HTTPS targets.
#[derive(Clone, Debug)]
pub struct TrustedEndpointResolver {
    deployment_id: Uuid,
    entries: BTreeMap<String, TrustedEndpointConfig>,
}

impl TrustedEndpointResolver {
    /// Create a bounded resolver from deployment-owned endpoint configuration.
    pub fn new(
        deployment_id: Uuid,
        entries: Vec<TrustedEndpointConfig>,
    ) -> Result<Self, EndpointResolutionError> {
        if deployment_id.is_nil() {
            return Err(EndpointResolutionError::InvalidDeploymentIdentity);
        }
        if entries.is_empty() || entries.len() > MAX_TRUSTED_ENDPOINTS {
            return Err(EndpointResolutionError::InvalidRegistrySize);
        }

        let mut by_identity = BTreeMap::new();
        let mut destinations = BTreeSet::new();
        for entry in entries {
            let destination = canonical_destination_key(&entry.https_uri, &entry.tls_server_name)?;
            if !destinations.insert(destination) {
                return Err(EndpointResolutionError::DuplicateEndpointDestination);
            }
            if by_identity
                .insert(entry.endpoint_identity.clone(), entry)
                .is_some()
            {
                return Err(EndpointResolutionError::DuplicateEndpointIdentity);
            }
        }

        Ok(Self {
            deployment_id,
            entries: by_identity,
        })
    }

    /// Resolve a profile pin to its deployment-trusted URI and TLS server name.
    ///
    /// The resolver does not perform network requests. Its output contains no redirect
    /// destination or redirect override; transport adapters must keep redirects disabled.
    /// The caller must provide a current revision from the trusted profile resolver because
    /// this immutable profile snapshot cannot observe a later disable or revocation.
    pub fn resolve(
        &self,
        profile: &TransportProfileRevision,
    ) -> Result<ResolvedTransportEndpoint, EndpointResolutionError> {
        if profile.deployment_id() != self.deployment_id {
            return Err(EndpointResolutionError::DeploymentMismatch);
        }
        if profile.lifecycle() != TransportProfileLifecycle::Active {
            return Err(EndpointResolutionError::InactiveProfile);
        }
        if profile.revision() <= 0 || profile.profile_id().is_nil() || profile.tenant_id().is_nil()
        {
            return Err(EndpointResolutionError::InvalidProfilePin);
        }

        let endpoint_identity = profile.definition().endpoint_identity();
        let endpoint = self
            .entries
            .get(endpoint_identity)
            .ok_or(EndpointResolutionError::UnknownEndpointIdentity)?;

        Ok(ResolvedTransportEndpoint {
            deployment_id: profile.deployment_id(),
            tenant_id: profile.tenant_id(),
            profile_id: profile.profile_id(),
            profile_revision: profile.revision(),
            profile_digest: profile.digest(),
            endpoint_identity: endpoint.endpoint_identity.clone(),
            https_uri: endpoint.https_uri.clone(),
            tls_server_name: endpoint.tls_server_name.clone(),
            secret_reference: profile.definition().secret_reference().cloned(),
        })
    }
}

/// A profile-bound, validated destination. Construction is restricted to the trusted resolver.
#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedTransportEndpoint {
    deployment_id: Uuid,
    tenant_id: Uuid,
    profile_id: Uuid,
    profile_revision: i64,
    profile_digest: [u8; 32],
    endpoint_identity: String,
    https_uri: String,
    tls_server_name: String,
    secret_reference: Option<OpaqueSecretReference>,
}

impl std::fmt::Debug for ResolvedTransportEndpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ResolvedTransportEndpoint")
            .field("deployment_id", &self.deployment_id)
            .field("tenant_id", &self.tenant_id)
            .field("profile_id", &self.profile_id)
            .field("profile_revision", &self.profile_revision)
            .field("profile_digest", &self.profile_digest)
            .field("endpoint_identity", &self.endpoint_identity)
            .field("host", &self.tls_server_name)
            .finish_non_exhaustive()
    }
}

impl ResolvedTransportEndpoint {
    /// Deployment identity pinned by the profile.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    /// Tenant identity pinned by the profile.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Stable profile identity pinned by this resolution.
    pub fn profile_id(&self) -> Uuid {
        self.profile_id
    }

    /// Immutable profile revision pinned by this resolution.
    pub fn profile_revision(&self) -> i64 {
        self.profile_revision
    }

    /// Canonical profile digest pinned by this resolution.
    pub fn profile_digest(&self) -> [u8; 32] {
        self.profile_digest
    }

    /// Deployment-trusted endpoint identity selected by the profile.
    pub fn endpoint_identity(&self) -> &str {
        &self.endpoint_identity
    }

    /// Validated HTTPS URI supplied by deployment-owned configuration.
    pub fn https_uri(&self) -> &str {
        &self.https_uri
    }

    /// DNS name used to validate the server certificate.
    pub fn tls_server_name(&self) -> &str {
        &self.tls_server_name
    }

    /// Opaque secret-manager locator; this resolver never loads credential material.
    pub fn secret_reference(&self) -> Option<&OpaqueSecretReference> {
        self.secret_reference.as_ref()
    }
}

/// Fail-closed endpoint configuration or profile resolution error.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EndpointResolutionError {
    /// Deployment identity is nil.
    #[error("invalid deployment identity")]
    InvalidDeploymentIdentity,
    /// The deployment endpoint registry is empty or exceeds its entry bound.
    #[error("invalid trusted endpoint registry size")]
    InvalidRegistrySize,
    /// An endpoint identity is malformed.
    #[error("invalid endpoint identity")]
    InvalidEndpointIdentity,
    /// Two deployment entries use one endpoint identity.
    #[error("duplicate trusted endpoint identity")]
    DuplicateEndpointIdentity,
    /// Two identities resolve to the same canonical destination.
    #[error("duplicate trusted endpoint destination")]
    DuplicateEndpointDestination,
    /// The configured TLS server name is not a valid DNS name.
    #[error("invalid TLS server name")]
    InvalidTlsServerName,
    /// The configured endpoint is not a bounded, valid HTTPS URI.
    #[error("invalid HTTPS endpoint URI")]
    InvalidHttpsUri,
    /// The URI host does not exactly identify the TLS server name.
    #[error("HTTPS host and TLS server name do not match")]
    UriServerNameMismatch,
    /// The profile belongs to a different deployment than this resolver.
    #[error("transport profile deployment mismatch")]
    DeploymentMismatch,
    /// The profile revision is disabled or revoked.
    #[error("transport profile is inactive")]
    InactiveProfile,
    /// The profile pin is malformed.
    #[error("invalid transport profile pin")]
    InvalidProfilePin,
    /// The profile names no endpoint in the trusted deployment registry.
    #[error("unknown trusted endpoint identity")]
    UnknownEndpointIdentity,
}

fn valid_endpoint_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn validate_tls_server_name(value: &str) -> Result<String, EndpointResolutionError> {
    if !valid_dns_name(value) || value.parse::<std::net::IpAddr>().is_ok() {
        return Err(EndpointResolutionError::InvalidTlsServerName);
    }
    Ok(value.to_ascii_lowercase())
}

fn valid_dns_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TLS_SERVER_NAME_BYTES
        && value.is_ascii()
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn validate_https_uri(uri: &str, tls_server_name: &str) -> Result<u16, EndpointResolutionError> {
    if uri.is_empty()
        || uri.len() > MAX_HTTPS_URI_BYTES
        || !uri.is_ascii()
        || !uri.starts_with("https://")
        || uri.bytes().any(|byte| {
            byte.is_ascii_control()
                || byte.is_ascii_whitespace()
                || matches!(byte, b'?' | b'#' | b'\\')
        })
    {
        return Err(EndpointResolutionError::InvalidHttpsUri);
    }

    let remainder = &uri["https://".len()..];
    let (authority, path) = match remainder.split_once('/') {
        Some((authority, path)) => (authority, Some(path)),
        None => (remainder, None),
    };
    if authority.is_empty() || authority.contains('@') || authority.contains('%') {
        return Err(EndpointResolutionError::InvalidHttpsUri);
    }

    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => {
            let parsed_port = port
                .parse::<u16>()
                .ok()
                .filter(|port| *port > 0)
                .ok_or(EndpointResolutionError::InvalidHttpsUri)?;
            if host.contains(':')
                || port.is_empty()
                || !port.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(EndpointResolutionError::InvalidHttpsUri);
            }
            (host, Some(parsed_port))
        }
        None => (authority, None),
    };

    if !valid_dns_name(host) {
        return Err(EndpointResolutionError::InvalidHttpsUri);
    }
    if !host.eq_ignore_ascii_case(tls_server_name) {
        return Err(EndpointResolutionError::UriServerNameMismatch);
    }
    if path.is_some_and(|path| !valid_uri_path(path)) {
        return Err(EndpointResolutionError::InvalidHttpsUri);
    }

    Ok(port.unwrap_or(443))
}

fn valid_uri_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return false;
            }
            index += 3;
            continue;
        }
        if !(byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'.'
                    | b'_'
                    | b'~'
                    | b'!'
                    | b'$'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b';'
                    | b'='
                    | b':'
                    | b'@'
                    | b'/'
            ))
        {
            return false;
        }
        index += 1;
    }
    true
}

fn canonical_destination_key(
    uri: &str,
    tls_server_name: &str,
) -> Result<String, EndpointResolutionError> {
    let port = validate_https_uri(uri, tls_server_name)?;
    let remainder = &uri["https://".len()..];
    let path = match remainder.split_once('/') {
        Some((_, path)) => format!("/{path}"),
        None => "/".to_owned(),
    };
    Ok(format!("{tls_server_name}:{port}{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport_profile::{TransportMessageKind, TransportProfileDefinition};

    const DEPLOYMENT_ID: Uuid = Uuid::from_u128(10);

    fn endpoint(identity: &str, uri: &str, server_name: &str) -> TrustedEndpointConfig {
        TrustedEndpointConfig::new(identity.to_owned(), uri.to_owned(), server_name.to_owned())
            .unwrap()
    }

    fn profile(
        endpoint_identity: &str,
        lifecycle: TransportProfileLifecycle,
        deployment_id: Uuid,
    ) -> TransportProfileRevision {
        let definition = TransportProfileDefinition::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            endpoint_identity.to_owned(),
            "provider-log-generation-v1".to_owned(),
            vec![Uuid::from_u128(20)],
            vec![Uuid::from_u128(21)],
            vec![TransportMessageKind::Request],
            Some(
                OpaqueSecretReference::new("secretref:platform/admission-client".to_owned())
                    .unwrap(),
            ),
        )
        .unwrap();
        TransportProfileRevision::new(
            deployment_id,
            Uuid::from_u128(11),
            Uuid::from_u128(12),
            1,
            lifecycle,
            definition,
        )
        .unwrap()
    }

    fn resolver() -> TrustedEndpointResolver {
        TrustedEndpointResolver::new(
            DEPLOYMENT_ID,
            vec![endpoint(
                "run-admission",
                "https://transport.internal.example:7443/grpc/v1",
                "transport.internal.example",
            )],
        )
        .unwrap()
    }

    #[test]
    fn active_profile_resolves_to_its_trusted_endpoint() {
        let profile = profile(
            "run-admission",
            TransportProfileLifecycle::Active,
            DEPLOYMENT_ID,
        );
        let resolved = resolver().resolve(&profile).unwrap();

        assert_eq!(resolved.deployment_id(), DEPLOYMENT_ID);
        assert_eq!(resolved.tenant_id(), Uuid::from_u128(11));
        assert_eq!(resolved.profile_id(), Uuid::from_u128(12));
        assert_eq!(resolved.profile_revision(), 1);
        assert_eq!(resolved.profile_digest(), profile.digest());
        assert_eq!(resolved.endpoint_identity(), "run-admission");
        assert_eq!(
            resolved.https_uri(),
            "https://transport.internal.example:7443/grpc/v1"
        );
        assert_eq!(resolved.tls_server_name(), "transport.internal.example");
        assert_eq!(
            resolved
                .secret_reference()
                .map(OpaqueSecretReference::as_str),
            Some("secretref:platform/admission-client")
        );
        assert_eq!(
            format!("{resolved:?}").contains("admission-client"),
            false,
            "debug output must not reveal the opaque credential locator"
        );
    }

    #[test]
    fn debug_output_redacts_endpoint_uri_and_secret_reference() {
        let profile = profile(
            "run-admission",
            TransportProfileLifecycle::Active,
            DEPLOYMENT_ID,
        );
        let resolver = resolver();
        let resolved = resolver.resolve(&profile).unwrap();
        for debug in [
            format!("{resolved:?}"),
            format!("{resolver:?}"),
            format!(
                "{:?}",
                endpoint(
                    "run-admission",
                    "https://transport.internal.example:7443/grpc/v1",
                    "transport.internal.example",
                )
            ),
        ] {
            assert!(!debug.contains("/grpc/v1"));
            assert!(!debug.contains("admission-client"));
        }
        assert!(format!("{resolved:?}").contains("transport.internal.example"));
    }

    #[test]
    fn unknown_endpoint_identity_is_rejected() {
        assert_eq!(
            resolver().resolve(&profile(
                "untrusted-endpoint",
                TransportProfileLifecycle::Active,
                DEPLOYMENT_ID,
            )),
            Err(EndpointResolutionError::UnknownEndpointIdentity)
        );
    }

    #[test]
    fn duplicate_identities_and_destinations_are_rejected() {
        assert_eq!(
            TrustedEndpointResolver::new(
                DEPLOYMENT_ID,
                vec![
                    endpoint(
                        "run-admission",
                        "https://a.internal.example",
                        "a.internal.example"
                    ),
                    endpoint(
                        "run-admission",
                        "https://b.internal.example",
                        "b.internal.example"
                    ),
                ],
            )
            .err(),
            Some(EndpointResolutionError::DuplicateEndpointIdentity)
        );
        assert_eq!(
            TrustedEndpointResolver::new(
                DEPLOYMENT_ID,
                vec![
                    endpoint(
                        "endpoint-a",
                        "https://a.internal.example",
                        "a.internal.example"
                    ),
                    endpoint(
                        "endpoint-b",
                        "https://a.internal.example:443/",
                        "a.internal.example"
                    ),
                ],
            )
            .err(),
            Some(EndpointResolutionError::DuplicateEndpointDestination)
        );
    }

    #[test]
    fn disabled_and_revoked_profiles_are_rejected() {
        for lifecycle in [
            TransportProfileLifecycle::Disabled,
            TransportProfileLifecycle::Revoked,
        ] {
            assert_eq!(
                resolver().resolve(&profile("run-admission", lifecycle, DEPLOYMENT_ID)),
                Err(EndpointResolutionError::InactiveProfile)
            );
        }
    }

    #[test]
    fn deployment_mismatch_is_rejected() {
        assert_eq!(
            resolver().resolve(&profile(
                "run-admission",
                TransportProfileLifecycle::Active,
                Uuid::from_u128(99),
            )),
            Err(EndpointResolutionError::DeploymentMismatch)
        );
    }

    #[test]
    fn malformed_uris_and_tls_names_are_rejected() {
        for (uri, server_name) in [
            (
                "http://transport.internal.example",
                "transport.internal.example",
            ),
            (
                "https://user@transport.internal.example",
                "transport.internal.example",
            ),
            (
                "https://transport.internal.example?token=x",
                "transport.internal.example",
            ),
            (
                "https://transport.internal.example#fragment",
                "transport.internal.example",
            ),
            (
                "https://transport.internal.example:0",
                "transport.internal.example",
            ),
            (
                "https://transport.internal.example:65536",
                "transport.internal.example",
            ),
            (
                "https://transport.internal.example/%GG",
                "transport.internal.example",
            ),
            ("https://transport.internal.example", "*.internal.example"),
            (
                "https://transport.internal.example",
                "transport..internal.example",
            ),
            (
                "https://other.internal.example",
                "transport.internal.example",
            ),
            ("https://127.0.0.1", "127.0.0.1"),
        ] {
            assert!(
                TrustedEndpointConfig::new(
                    "run-admission".to_owned(),
                    uri.to_owned(),
                    server_name.to_owned(),
                )
                .is_err(),
                "unexpectedly accepted URI {uri:?} with TLS name {server_name:?}"
            );
        }

        let too_long_uri = format!("https://transport.internal.example/{}", "a".repeat(2_048));
        assert_eq!(
            TrustedEndpointConfig::new(
                "run-admission".to_owned(),
                too_long_uri,
                "transport.internal.example".to_owned(),
            )
            .err(),
            Some(EndpointResolutionError::InvalidHttpsUri)
        );
    }

    #[test]
    fn endpoint_registry_is_bounded() {
        let entries = (0..=MAX_TRUSTED_ENDPOINTS)
            .map(|index| {
                let name = format!("node-{index}.internal.example");
                endpoint(
                    &format!("endpoint-{index}"),
                    &format!("https://{name}"),
                    &name,
                )
            })
            .collect();
        assert_eq!(
            TrustedEndpointResolver::new(DEPLOYMENT_ID, entries).err(),
            Some(EndpointResolutionError::InvalidRegistrySize)
        );
    }
}
