// SPDX-License-Identifier: Apache-2.0
// Adapted from D:/RustGameServer/crates/shared-platform/src/tls.rs at
// 89458d887e8bfddf6637f9d80ea65f2091e8b6f2 (Apache-2.0).
// TLS loading and fail-closed behavior are retained; application identity stays Star-owned.
/*
@cypher schema=1 source_sha256=b0cfae0370c4cebb1dfd0bcf3dc229900882006353d1e449c53073e19e1ee1f4
MERGE (self:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
MERGE (tls_client:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::client_tls_config",kind:"function"})
MERGE (tls_server:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::server_mtls_config",kind:"function"})
MERGE (peer_fingerprint:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::peer_certificate_fingerprint",kind:"function"})
MERGE (read_pem:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::read_pem",kind:"function"})
MERGE (valid_domain:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::valid_server_domain",kind:"function"})
MERGE (valid_https_endpoint:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::valid_https_endpoint",kind:"function"})
MERGE (setup_error:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcSetupError",kind:"enum"})
MERGE (identity_error:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcIdentityError",kind:"struct"})
MERGE (transport:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionTransport",kind:"struct"})
MERGE (client_config:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcClientConfig",kind:"struct"})
MERGE (connect:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionTransport.connect",kind:"method"})
MERGE (publish:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionTransport.publish",kind:"method"})
MERGE (proto:File {path:"crates/platform-messaging-proto/src/lib.rs"})
MERGE (admission:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransport",kind:"trait"})
MERGE (receipt:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportReceipt",kind:"struct"})
MERGE (tonic:ExternalService {id:"tonic::transport",kind:"library"})
MERGE (tonic_request:ExternalService {id:"tonic::Request::peer_certs",kind:"method"})
MERGE (tonic_append:ExternalService {id:"platform_messaging_proto::wire::DurableTransportClient::append",kind:"method"})
MERGE (tonic_uri_host:ExternalService {id:"tonic::transport::Uri::host",kind:"method"})
MERGE (tonic_uri_scheme:ExternalService {id:"tonic::transport::Uri::scheme_str",kind:"method"})
MERGE (sha256:ExternalService {id:"sha2::Sha256::digest",kind:"method"})
MERGE (wire_limit:Config {id:"crates/platform-messaging-runtime/src/native_grpc.rs::MAX_NATIVE_GRPC_MESSAGE_BYTES"})
MERGE (pem_limit:Config {id:"crates/platform-messaging-runtime/src/native_grpc.rs::MAX_PEM_FILE_BYTES"})
MERGE (in_flight_limit:Config {id:"crates/platform-messaging-runtime/src/native_grpc.rs::MAX_NATIVE_GRPC_IN_FLIGHT"})
MERGE (provider_id:Config {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NATIVE_PROVIDER_ID"})
MERGE (setup_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::missing_server_mtls_material_fails_closed",kind:"test"})
MERGE (domain_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::client_rejects_empty_or_invalid_domain_before_opening_files",kind:"test"})
MERGE (dns_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::server_domain_rejects_invalid_dns_labels",kind:"test"})
MERGE (identity_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::missing_peer_certificate_is_never_anonymous",kind:"test"})
MERGE (pem_limit_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::oversized_pem_file_is_rejected_after_bounded_read",kind:"test"})
MERGE (endpoint_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::endpoint_host_must_match_tls_server_domain",kind:"test"})
MERGE (semaphore:ExternalService {id:"tokio::sync::Semaphore::acquire_owned",kind:"method"})
MERGE (bounded_read:ExternalService {id:"std::io::Read::read_to_end",kind:"method"})
MERGE (self)-[:DEFINES]->(tls_client)
MERGE (self)-[:DEFINES]->(tls_server)
MERGE (self)-[:DEFINES]->(peer_fingerprint)
MERGE (self)-[:DEFINES]->(read_pem)
MERGE (self)-[:DEFINES]->(valid_domain)
MERGE (self)-[:DEFINES]->(valid_https_endpoint)
MERGE (self)-[:DEFINES]->(setup_error)
MERGE (self)-[:DEFINES]->(identity_error)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(client_config)
MERGE (self)-[:DEFINES]->(connect)
MERGE (self)-[:DEFINES]->(publish)
MERGE (self)-[:IMPORTS]->(proto)
MERGE (self)-[:DEPENDS_ON]->(native_proto_package:Config {id:"cargo.package.platform-messaging-proto"})
MERGE (self)-[:IMPORTS]->(admission)
MERGE (self)-[:IMPORTS]->(receipt)
MERGE (self)-[:IMPORTS]->(tonic)
MERGE (self)-[:IMPORTS]->(sha256)
MERGE (self)-[:DEFINES]->(wire_limit)
MERGE (self)-[:DEFINES]->(pem_limit)
MERGE (self)-[:DEFINES]->(in_flight_limit)
MERGE (self)-[:DEFINES]->(provider_id)
MERGE (self)-[:DEFINES]->(setup_test)
MERGE (self)-[:DEFINES]->(domain_test)
MERGE (self)-[:DEFINES]->(dns_test)
MERGE (self)-[:DEFINES]->(identity_test)
MERGE (self)-[:DEFINES]->(pem_limit_test)
MERGE (self)-[:DEFINES]->(endpoint_test)
MERGE (transport)-[:IMPLEMENTS]->(admission)
MERGE (connect)-[:USES_TYPE]->(client_config)
MERGE (connect)-[:CALLS]->(tls_client)
MERGE (connect)-[:CALLS]->(valid_https_endpoint)
MERGE (valid_https_endpoint)-[:CALLS]->(valid_domain)
MERGE (valid_https_endpoint)-[:CALLS]->(tonic_uri_host)
MERGE (valid_https_endpoint)-[:CALLS]->(tonic_uri_scheme)
MERGE (connect)-[:CALLS]->(tonic)
MERGE (connect)-[:USES_TYPE]->(provider_id)
MERGE (publish)-[:CALLS]->(tonic_append)
MERGE (publish)-[:CALLS]->(receipt)
MERGE (publish)-[:CALLS]->(sha256)
MERGE (publish)-[:USES_TYPE]->(wire_limit)
MERGE (publish)-[:CALLS]->(semaphore)
MERGE (peer_fingerprint)-[:CALLS]->(tonic_request)
MERGE (peer_fingerprint)-[:CALLS]->(sha256)
MERGE (tls_client)-[:CALLS]->(read_pem)
MERGE (tls_client)-[:CALLS]->(valid_domain)
MERGE (tls_server)-[:CALLS]->(read_pem)
MERGE (read_pem)-[:USES_TYPE]->(pem_limit)
MERGE (read_pem)-[:CALLS]->(bounded_read)
MERGE (setup_test)-[:TESTS]->(tls_server)
MERGE (domain_test)-[:TESTS]->(tls_client)
MERGE (dns_test)-[:TESTS]->(valid_domain)
MERGE (identity_test)-[:TESTS]->(peer_fingerprint)
MERGE (pem_limit_test)-[:TESTS]->(read_pem)
MERGE (endpoint_test)-[:TESTS]->(valid_https_endpoint)
@endcypher
*/
//! Native gRPC mTLS client adapter for Star's durable transport protocol.

use std::{io::Read, path::Path, sync::Arc, time::Duration};

use platform_messaging_proto::wire;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::sync::Semaphore;
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity, ServerTlsConfig, Uri},
    Request,
};
use uuid::Uuid;

use crate::admission_delivery::{
    AdmissionDeliveryClaim, AdmissionDeliveryOwner, AdmissionTransport, AdmissionTransportError,
    AdmissionTransportProfile, AdmissionTransportReceipt,
};

/// The transport wire limit plus protobuf envelope headroom.
pub const MAX_NATIVE_GRPC_MESSAGE_BYTES: usize = 32 * 1024;
const MAX_PEM_FILE_BYTES: u64 = 1024 * 1024;
const MAX_NATIVE_GRPC_IN_FLIGHT: usize = 8;
const NATIVE_PROVIDER_ID: &str = "star-rust-log";

/// Bounded setup errors that never include PEM contents or private-key material.
#[derive(Debug, Error)]
pub enum NativeGrpcSetupError {
    /// A certificate/key file is missing, unreadable, empty or too large.
    #[error("invalid TLS material file: {0}")]
    TlsFile(String),
    /// The configured HTTPS endpoint or timeout is invalid.
    #[error("invalid native transport endpoint configuration")]
    Endpoint,
    /// The HTTP/2 channel could not be established.
    #[error("native transport connection failed")]
    Connect,
    /// The immutable transport profile does not identify this adapter endpoint.
    #[error("native transport profile mismatch")]
    Profile,
}

/// Missing certificate evidence is an authentication failure, never an anonymous fallback.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("mutual TLS peer certificate is required")]
pub struct NativeGrpcIdentityError;

/// Hash the leaf certificate presented by the mutually authenticated peer.
///
/// The returned digest is only a lookup key. The trusted server must resolve it against its
/// active certificate registry and then authorize the resulting service identity and route.
pub fn peer_certificate_fingerprint<T>(
    request: &Request<T>,
) -> Result<[u8; 32], NativeGrpcIdentityError> {
    let certificates = request.peer_certs().ok_or(NativeGrpcIdentityError)?;
    let leaf = certificates.first().ok_or(NativeGrpcIdentityError)?;
    Ok(Sha256::digest(leaf.as_ref()).into())
}

/// Load a PEM file with a strict upper bound; called during trusted service startup only.
fn read_pem(path: &Path) -> Result<Vec<u8>, NativeGrpcSetupError> {
    let file = std::fs::File::open(path)
        .map_err(|_| NativeGrpcSetupError::TlsFile(path.display().to_string()))?;
    let mut pem = Vec::with_capacity(MAX_PEM_FILE_BYTES as usize + 1);
    file.take(MAX_PEM_FILE_BYTES + 1)
        .read_to_end(&mut pem)
        .map_err(|_| NativeGrpcSetupError::TlsFile(path.display().to_string()))?;
    if pem.is_empty() || pem.len() as u64 > MAX_PEM_FILE_BYTES {
        return Err(NativeGrpcSetupError::TlsFile(path.display().to_string()));
    }
    Ok(pem)
}

fn valid_server_domain(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.is_ascii()
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric())
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric())
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn valid_https_endpoint(uri: &str, server_domain: &str) -> bool {
    if !valid_server_domain(server_domain) {
        return false;
    }
    let Ok(uri) = uri.parse::<Uri>() else {
        return false;
    };
    uri.scheme_str() == Some("https")
        && uri
            .host()
            .is_some_and(|host| host.eq_ignore_ascii_case(server_domain))
}

/// Build a mandatory client-certificate TLS identity and pin server-name validation.
///
/// The returned configuration has no plaintext mode. The caller must also use an `https` URI.
pub fn client_tls_config(
    server_domain: &str,
    server_ca_path: &Path,
    client_cert_path: &Path,
    client_key_path: &Path,
) -> Result<ClientTlsConfig, NativeGrpcSetupError> {
    if !valid_server_domain(server_domain) {
        return Err(NativeGrpcSetupError::Endpoint);
    }
    let ca = Certificate::from_pem(read_pem(server_ca_path)?);
    let identity = Identity::from_pem(read_pem(client_cert_path)?, read_pem(client_key_path)?);
    Ok(ClientTlsConfig::new()
        .domain_name(server_domain.to_owned())
        .ca_certificate(ca)
        .identity(identity))
}

/// Build a server TLS config that requires a client certificate chaining to the supplied CA.
pub fn server_mtls_config(
    server_cert_path: &Path,
    server_key_path: &Path,
    client_ca_path: &Path,
) -> Result<ServerTlsConfig, NativeGrpcSetupError> {
    let identity = Identity::from_pem(read_pem(server_cert_path)?, read_pem(server_key_path)?);
    let client_ca = Certificate::from_pem(read_pem(client_ca_path)?);
    Ok(ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca)
        .client_auth_optional(false))
}

/// Star's native gRPC implementation of the provider-neutral admission transport port.
#[derive(Clone)]
pub struct NativeGrpcAdmissionTransport {
    client: wire::durable_transport_client::DurableTransportClient<Channel>,
    profile: AdmissionTransportProfile,
    owner: AdmissionDeliveryOwner,
    in_flight: Arc<Semaphore>,
}

/// Immutable construction inputs for one pinned native transport client.
pub struct NativeGrpcClientConfig<'a> {
    /// Star delivery owner that will use this client.
    pub owner: AdmissionDeliveryOwner,
    /// HTTPS endpoint URI for the native transport server.
    pub uri: &'a str,
    /// DNS name pinned in the server certificate.
    pub server_domain: &'a str,
    /// Immutable endpoint identity pinned by the transport profile.
    pub endpoint_identity: &'a str,
    /// Trusted CA bundle used to validate the server certificate.
    pub server_ca_path: &'a Path,
    /// Client certificate presented for mutual TLS authentication.
    pub client_cert_path: &'a Path,
    /// Private key corresponding to `client_cert_path`.
    pub client_key_path: &'a Path,
    /// Connection establishment timeout, capped at 30 seconds.
    pub connect_timeout: Duration,
    /// Per-request timeout, capped at 10 seconds.
    pub request_timeout: Duration,
    /// Profile identity that pins provider and endpoint configuration.
    pub profile: AdmissionTransportProfile,
}

impl NativeGrpcAdmissionTransport {
    /// Connect one multiplexed, mutually authenticated HTTP/2 channel to a pinned endpoint.
    pub async fn connect(config: NativeGrpcClientConfig<'_>) -> Result<Self, NativeGrpcSetupError> {
        let NativeGrpcClientConfig {
            owner,
            uri,
            server_domain,
            endpoint_identity,
            server_ca_path,
            client_cert_path,
            client_key_path,
            connect_timeout,
            request_timeout,
            profile,
        } = config;
        if profile.provider_id != NATIVE_PROVIDER_ID
            || profile.endpoint_identity != endpoint_identity
        {
            return Err(NativeGrpcSetupError::Profile);
        }
        if connect_timeout.is_zero()
            || connect_timeout > Duration::from_secs(30)
            || request_timeout.is_zero()
            || request_timeout > Duration::from_secs(10)
            || !valid_https_endpoint(uri, server_domain)
        {
            return Err(NativeGrpcSetupError::Endpoint);
        }
        let endpoint = Endpoint::from_shared(uri.to_owned())
            .map_err(|_| NativeGrpcSetupError::Endpoint)?
            .connect_timeout(connect_timeout)
            .timeout(request_timeout)
            .tls_config(client_tls_config(
                server_domain,
                server_ca_path,
                client_cert_path,
                client_key_path,
            )?)
            .map_err(|_| NativeGrpcSetupError::Endpoint)?;
        let channel = endpoint
            .connect()
            .await
            .map_err(|_| NativeGrpcSetupError::Connect)?;
        let client = wire::durable_transport_client::DurableTransportClient::new(channel)
            .max_encoding_message_size(MAX_NATIVE_GRPC_MESSAGE_BYTES)
            .max_decoding_message_size(MAX_NATIVE_GRPC_MESSAGE_BYTES);
        Ok(Self {
            client,
            profile,
            owner,
            in_flight: Arc::new(Semaphore::new(MAX_NATIVE_GRPC_IN_FLIGHT)),
        })
    }
}

impl AdmissionTransport for NativeGrpcAdmissionTransport {
    fn profile(&self) -> &AdmissionTransportProfile {
        &self.profile
    }

    async fn publish(
        &self,
        owner: AdmissionDeliveryOwner,
        claim: &AdmissionDeliveryClaim,
    ) -> Result<AdmissionTransportReceipt, AdmissionTransportError> {
        if owner != self.owner
            || claim.transport_profile != self.profile
            || claim.wire.is_empty()
            || claim.wire.len() > 16 * 1024
        {
            return Err(AdmissionTransportError);
        }
        let _permit = self
            .in_flight
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AdmissionTransportError)?;
        let payload_sha256: [u8; 32] = Sha256::digest(&claim.wire).into();
        let request = wire::AppendRequest {
            profile: Some(wire::ProfilePin {
                profile_digest: claim.transport_profile.profile_digest.to_vec(),
            }),
            recipient_service_id: claim.target_service_id.as_bytes().to_vec(),
            message_kind: claim.message_kind.clone(),
            message_id: claim.message_id.as_bytes().to_vec(),
            wire: claim.wire.clone(),
            wire_sha256: payload_sha256.to_vec(),
        };
        // Clone the cheap multiplexed Channel handle per call, so independent deliveries
        // can run concurrently without a process-wide async mutex.
        let mut client = self.client.clone();
        let response = client
            .append(Request::new(request))
            .await
            .map_err(|_| AdmissionTransportError)?
            .into_inner();
        if response.wire_sha256.as_slice() != payload_sha256.as_slice() || response.sequence <= 0 {
            return Err(AdmissionTransportError);
        }
        let generation =
            Uuid::from_slice(&response.log_generation).map_err(|_| AdmissionTransportError)?;
        AdmissionTransportReceipt::new(
            self.profile.clone(),
            response.log_id,
            generation,
            response.sequence.to_be_bytes().to_vec(),
            &claim.wire,
        )
        .map_err(|_| AdmissionTransportError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_server_mtls_material_fails_closed() {
        let result = server_mtls_config(
            Path::new("missing-server-cert.pem"),
            Path::new("missing-server-key.pem"),
            Path::new("missing-client-ca.pem"),
        );
        assert!(matches!(result, Err(NativeGrpcSetupError::TlsFile(_))));
    }

    #[test]
    fn oversized_pem_file_is_rejected_after_bounded_read() {
        let path = std::env::temp_dir().join(format!(
            "star-native-grpc-pem-{}-{}.pem",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::write(&path, vec![b'x'; MAX_PEM_FILE_BYTES as usize + 1]).unwrap();
        let result = read_pem(&path);
        let cleanup = std::fs::remove_file(path);
        assert!(matches!(result, Err(NativeGrpcSetupError::TlsFile(_))));
        assert!(cleanup.is_ok());
    }

    #[test]
    fn client_rejects_empty_or_invalid_domain_before_opening_files() {
        let result = client_tls_config(
            "bad domain",
            Path::new("missing-ca.pem"),
            Path::new("missing-client.pem"),
            Path::new("missing-key.pem"),
        );
        assert!(matches!(result, Err(NativeGrpcSetupError::Endpoint)));
    }

    #[test]
    fn server_domain_rejects_invalid_dns_labels() {
        assert!(!valid_server_domain("example..invalid"));
        assert!(!valid_server_domain("-example.invalid"));
        assert!(valid_server_domain("transport.internal.example"));
    }

    #[test]
    fn endpoint_host_must_match_tls_server_domain() {
        assert!(valid_https_endpoint(
            "https://transport.internal.example:7443",
            "transport.internal.example",
        ));
        assert!(!valid_https_endpoint(
            "https://other.internal.example:7443",
            "transport.internal.example",
        ));
        assert!(!valid_https_endpoint(
            "http://transport.internal.example:7443",
            "transport.internal.example",
        ));
    }

    #[test]
    fn missing_peer_certificate_is_never_anonymous() {
        assert_eq!(
            peer_certificate_fingerprint(&Request::new(())),
            Err(NativeGrpcIdentityError),
        );
    }
}
