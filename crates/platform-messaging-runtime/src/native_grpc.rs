// SPDX-License-Identifier: Apache-2.0
// Adapted from D:/RustGameServer/crates/shared-platform/src/tls.rs at
// 89458d887e8bfddf6637f9d80ea65f2091e8b6f2 (Apache-2.0).
// TLS loading and fail-closed behavior are retained; application identity stays Star-owned.
/*
@cypher schema=1 source_sha256=540c6acda8696c0a906764300e127c5527ceb99d8d2c1d842065a7a31275bf66
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
MERGE (receive_transport:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport",kind:"struct"})
MERGE (receive_config:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcReceiveClientConfig",kind:"struct"})
MERGE (receive_delivery:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionDelivery",kind:"struct"})
MERGE (connect:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionTransport.connect",kind:"method"})
MERGE (publish:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionTransport.publish",kind:"method"})
MERGE (connect_receive_client:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::connect_native_client",kind:"function"})
MERGE (receive_connect:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport.connect",kind:"method"})
MERGE (pull_one:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport.pull_one",kind:"method"})
MERGE (acknowledge:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionDelivery.acknowledge",kind:"method"})
MERGE (terminate:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionDelivery.terminate_permanently",kind:"method"})
MERGE (validate_delivery:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::validate_native_delivery",kind:"function"})
MERGE (validate_scope:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::valid_receive_scope",kind:"function"})
MERGE (kind_mapping:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::native_message_kind",kind:"function"})
MERGE (provider_identity:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::native_provider_identity",kind:"function"})
MERGE (proto:File {path:"crates/platform-messaging-proto/src/lib.rs"})
MERGE (admission:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransport",kind:"trait"})
MERGE (receipt:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportReceipt",kind:"struct"})
MERGE (receive_port:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport",kind:"trait"})
MERGE (delivery_port:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery",kind:"trait"})
MERGE (tonic:ExternalService {id:"tonic::transport",kind:"library"})
MERGE (tonic_request:ExternalService {id:"tonic::Request::peer_certs",kind:"method"})
MERGE (tonic_append:ExternalService {id:"platform_messaging_proto::wire::DurableTransportClient::append",kind:"method"})
MERGE (tonic_pull:ExternalService {id:"platform_messaging_proto::wire::DurableTransportClient::pull_one",kind:"method"})
MERGE (tonic_ack:ExternalService {id:"platform_messaging_proto::wire::DurableTransportClient::acknowledge_one",kind:"method"})
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
MERGE (native_receive_valid_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::native_receive_maps_exact_server_identity_and_wire_proof",kind:"test"})
MERGE (native_receive_route_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::native_receive_rejects_wrong_route_sender_kind_and_digest",kind:"test"})
MERGE (native_receive_bounds_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::native_receive_rejects_malformed_or_oversized_delivery_metadata",kind:"test"})
MERGE (native_provider_identity_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::tests::native_provider_identity_pins_provider_version_profile_and_endpoint",kind:"test"})
MERGE (semaphore:ExternalService {id:"tokio::sync::Semaphore::acquire_owned",kind:"method"})
MERGE (permit:ExternalService {id:"tokio::sync::OwnedSemaphorePermit",kind:"type"})
MERGE (bounded_read:ExternalService {id:"std::io::Read::read_to_end",kind:"method"})
MERGE (uuid_parse:ExternalService {id:"uuid::Uuid::from_slice",kind:"method"})
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
MERGE (self)-[:DEFINES]->(receive_transport)
MERGE (self)-[:DEFINES]->(receive_config)
MERGE (self)-[:DEFINES]->(receive_delivery)
MERGE (self)-[:DEFINES]->(connect)
MERGE (self)-[:DEFINES]->(publish)
MERGE (self)-[:DEFINES]->(connect_receive_client)
MERGE (self)-[:DEFINES]->(receive_connect)
MERGE (self)-[:DEFINES]->(pull_one)
MERGE (self)-[:DEFINES]->(acknowledge)
MERGE (self)-[:DEFINES]->(terminate)
MERGE (self)-[:DEFINES]->(validate_delivery)
MERGE (self)-[:DEFINES]->(validate_scope)
MERGE (self)-[:DEFINES]->(kind_mapping)
MERGE (self)-[:DEFINES]->(provider_identity)
MERGE (self)-[:IMPORTS]->(proto)
MERGE (self)-[:DEPENDS_ON]->(native_proto_package:Config {id:"cargo.package.platform-messaging-proto"})
MERGE (self)-[:IMPORTS]->(admission)
MERGE (self)-[:IMPORTS]->(receipt)
MERGE (self)-[:IMPORTS]->(receive_port)
MERGE (self)-[:IMPORTS]->(delivery_port)
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
MERGE (self)-[:DEFINES]->(native_receive_valid_test)
MERGE (self)-[:DEFINES]->(native_receive_route_test)
MERGE (self)-[:DEFINES]->(native_receive_bounds_test)
MERGE (self)-[:DEFINES]->(native_provider_identity_test)
MERGE (transport)-[:IMPLEMENTS]->(admission)
MERGE (connect)-[:USES_TYPE]->(client_config)
MERGE (connect)-[:CALLS]->(connect_receive_client)
MERGE (receive_connect)-[:USES_TYPE]->(receive_config)
MERGE (receive_connect)-[:CALLS]->(connect_receive_client)
MERGE (receive_connect)-[:CALLS]->(validate_scope)
MERGE (receive_connect)-[:CALLS]->(semaphore)
MERGE (receive_transport)-[:IMPLEMENTS]->(receive_port)
MERGE (receive_delivery)-[:IMPLEMENTS]->(delivery_port)
MERGE (connect_receive_client)-[:CALLS]->(tls_client)
MERGE (connect_receive_client)-[:CALLS]->(valid_https_endpoint)
MERGE (connect_receive_client)-[:CALLS]->(tonic)
MERGE (valid_https_endpoint)-[:CALLS]->(valid_domain)
MERGE (valid_https_endpoint)-[:CALLS]->(tonic_uri_host)
MERGE (valid_https_endpoint)-[:CALLS]->(tonic_uri_scheme)
MERGE (connect_receive_client)-[:USES_TYPE]->(provider_id)
MERGE (publish)-[:CALLS]->(tonic_append)
MERGE (publish)-[:CALLS]->(receipt)
MERGE (publish)-[:CALLS]->(sha256)
MERGE (publish)-[:USES_TYPE]->(wire_limit)
MERGE (publish)-[:CALLS]->(semaphore)
MERGE (pull_one)-[:CALLS]->(tonic_pull)
MERGE (pull_one)-[:CALLS]->(validate_delivery)
MERGE (pull_one)-[:CALLS]->(kind_mapping)
MERGE (pull_one)-[:CALLS]->(semaphore)
MERGE (pull_one)-[:CALLS]->(provider_identity)
MERGE (pull_one)-[:USES_TYPE]->(permit)
MERGE (acknowledge)-[:CALLS]->(tonic_ack)
MERGE (acknowledge)-[:CALLS]->(kind_mapping)
MERGE (acknowledge)-[:CALLS]->(semaphore)
MERGE (terminate)-[:CALLS]->(acknowledge)
MERGE (validate_delivery)-[:CALLS]->(sha256)
MERGE (validate_delivery)-[:CALLS]->(uuid_parse)
MERGE (validate_delivery)-[:CALLS]->(validate_scope)
MERGE (validate_delivery)-[:CALLS]->(kind_mapping)
MERGE (provider_identity)-[:CALLS]->(sha256)
MERGE (native_receive_valid_test)-[:TESTS]->(validate_delivery)
MERGE (native_receive_route_test)-[:TESTS]->(validate_delivery)
MERGE (native_receive_bounds_test)-[:TESTS]->(validate_delivery)
MERGE (native_provider_identity_test)-[:TESTS]->(provider_identity)
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
    Request,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity, ServerTlsConfig, Uri},
};
use uuid::Uuid;

use crate::admission_delivery::{
    AdmissionDeliveryClaim, AdmissionDeliveryOwner, AdmissionTransport, AdmissionTransportError,
    AdmissionTransportProfile, AdmissionTransportReceipt,
};
use crate::admission_receive::{
    AdmissionReceiveError, AdmissionReceiveKind, AdmissionReceiveOrigin, AdmissionReceiveScope,
    AdmissionReceiveTransport, AdmissionReceivedDelivery,
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
    /// The configured receive scope or durable delivery lease is invalid.
    #[error("invalid native transport receive route or lease")]
    ReceiveRoute,
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

/// Immutable construction inputs for one native receiver bound to one owner route.
pub struct NativeGrpcReceiveClientConfig<'a> {
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
    /// Pinned route and owner identities resolved by trusted deployment configuration.
    pub scope: AdmissionReceiveScope,
    /// Transport lease in seconds. Native delivery leases are bounded to 30–120 seconds.
    pub lease_seconds: u32,
    /// Profile identity that pins provider and endpoint configuration.
    pub profile: AdmissionTransportProfile,
}

/// Star's native gRPC implementation of the provider-neutral admission receive port.
pub struct NativeGrpcAdmissionReceiveTransport {
    client: wire::durable_transport_client::DurableTransportClient<Channel>,
    profile: AdmissionTransportProfile,
    provider_identity: String,
    scope: AdmissionReceiveScope,
    lease_seconds: i32,
    in_flight: Arc<Semaphore>,
    outstanding: Arc<Semaphore>,
}

/// One bounded native log delivery and its exact lease-fenced receipt identity.
pub struct NativeGrpcAdmissionDelivery {
    client: wire::durable_transport_client::DurableTransportClient<Channel>,
    profile: AdmissionTransportProfile,
    scope: AdmissionReceiveScope,
    origin: AdmissionReceiveOrigin,
    wire: Vec<u8>,
    lease_id: Uuid,
    wire_sha256: [u8; 32],
    in_flight: Arc<Semaphore>,
    _outstanding_permit: tokio::sync::OwnedSemaphorePermit,
}

struct NativeGrpcEndpointConfig<'a> {
    uri: &'a str,
    server_domain: &'a str,
    endpoint_identity: &'a str,
    server_ca_path: &'a Path,
    client_cert_path: &'a Path,
    client_key_path: &'a Path,
    connect_timeout: Duration,
    request_timeout: Duration,
    profile: &'a AdmissionTransportProfile,
}

async fn connect_native_client(
    config: NativeGrpcEndpointConfig<'_>,
) -> Result<wire::durable_transport_client::DurableTransportClient<Channel>, NativeGrpcSetupError> {
    if config.profile.provider_id != NATIVE_PROVIDER_ID
        || config.profile.endpoint_identity != config.endpoint_identity
    {
        return Err(NativeGrpcSetupError::Profile);
    }
    if config.connect_timeout.is_zero()
        || config.connect_timeout > Duration::from_secs(30)
        || config.request_timeout.is_zero()
        || config.request_timeout > Duration::from_secs(10)
        || !valid_https_endpoint(config.uri, config.server_domain)
    {
        return Err(NativeGrpcSetupError::Endpoint);
    }
    let endpoint = Endpoint::from_shared(config.uri.to_owned())
        .map_err(|_| NativeGrpcSetupError::Endpoint)?
        .connect_timeout(config.connect_timeout)
        .timeout(config.request_timeout)
        .tls_config(client_tls_config(
            config.server_domain,
            config.server_ca_path,
            config.client_cert_path,
            config.client_key_path,
        )?)
        .map_err(|_| NativeGrpcSetupError::Endpoint)?;
    let channel = endpoint
        .connect()
        .await
        .map_err(|_| NativeGrpcSetupError::Connect)?;
    Ok(
        wire::durable_transport_client::DurableTransportClient::new(channel)
            .max_encoding_message_size(MAX_NATIVE_GRPC_MESSAGE_BYTES)
            .max_decoding_message_size(MAX_NATIVE_GRPC_MESSAGE_BYTES),
    )
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
        let client = connect_native_client(NativeGrpcEndpointConfig {
            uri,
            server_domain,
            endpoint_identity,
            server_ca_path,
            client_cert_path,
            client_key_path,
            connect_timeout,
            request_timeout,
            profile: &profile,
        })
        .await?;
        Ok(Self {
            client,
            profile,
            owner,
            in_flight: Arc::new(Semaphore::new(MAX_NATIVE_GRPC_IN_FLIGHT)),
        })
    }
}

impl NativeGrpcAdmissionReceiveTransport {
    /// Connect one bounded, mutually authenticated receive route to the pinned profile endpoint.
    pub async fn connect(
        config: NativeGrpcReceiveClientConfig<'_>,
    ) -> Result<Self, NativeGrpcSetupError> {
        let NativeGrpcReceiveClientConfig {
            uri,
            server_domain,
            endpoint_identity,
            server_ca_path,
            client_cert_path,
            client_key_path,
            connect_timeout,
            request_timeout,
            scope,
            lease_seconds,
            profile,
        } = config;
        if !valid_receive_scope(scope) || !(30..=120).contains(&lease_seconds) {
            return Err(NativeGrpcSetupError::ReceiveRoute);
        }
        let client = connect_native_client(NativeGrpcEndpointConfig {
            uri,
            server_domain,
            endpoint_identity,
            server_ca_path,
            client_cert_path,
            client_key_path,
            connect_timeout,
            request_timeout,
            profile: &profile,
        })
        .await?;
        let provider_identity = native_provider_identity(&profile);
        Ok(Self {
            client,
            profile,
            provider_identity,
            scope,
            lease_seconds: lease_seconds as i32,
            in_flight: Arc::new(Semaphore::new(MAX_NATIVE_GRPC_IN_FLIGHT)),
            outstanding: Arc::new(Semaphore::new(MAX_NATIVE_GRPC_IN_FLIGHT)),
        })
    }
}

impl AdmissionReceiveTransport for NativeGrpcAdmissionReceiveTransport {
    type Delivery = NativeGrpcAdmissionDelivery;

    async fn pull_one(
        &self,
        scope: AdmissionReceiveScope,
    ) -> Result<Option<Self::Delivery>, AdmissionReceiveError> {
        if scope != self.scope {
            return Err(AdmissionReceiveError::Contract);
        }
        // Bound leased-but-unacknowledged deliveries as well as concurrent RPCs. The permit
        // moves into the delivery handle and is released only when that handle is dropped.
        let outstanding_permit = self
            .outstanding
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?;
        let _permit = self
            .in_flight
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?;
        let request = wire::PullOneRequest {
            profile: Some(wire::ProfilePin {
                profile_digest: self.profile.profile_digest.to_vec(),
            }),
            sender_service_id: scope.sender_service_id.as_bytes().to_vec(),
            message_kind: native_message_kind(scope.kind).to_owned(),
            lease_seconds: self.lease_seconds,
        };
        let mut client = self.client.clone();
        let response = client
            .pull_one(Request::new(request))
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?
            .into_inner();
        let Some(delivery) = response.delivery else {
            return Ok(None);
        };
        let validated = validate_native_delivery(scope, &self.provider_identity, delivery)?;
        Ok(Some(NativeGrpcAdmissionDelivery {
            client: self.client.clone(),
            profile: self.profile.clone(),
            scope,
            origin: validated.origin,
            wire: validated.wire,
            lease_id: validated.lease_id,
            wire_sha256: validated.wire_sha256,
            in_flight: self.in_flight.clone(),
            _outstanding_permit: outstanding_permit,
        }))
    }
}

impl AdmissionReceivedDelivery for NativeGrpcAdmissionDelivery {
    fn wire(&self) -> &[u8] {
        &self.wire
    }

    fn origin(&self) -> &AdmissionReceiveOrigin {
        &self.origin
    }

    async fn acknowledge(&self) -> Result<(), AdmissionReceiveError> {
        let _permit = self
            .in_flight
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?;
        let request = wire::AcknowledgeOneRequest {
            profile: Some(wire::ProfilePin {
                profile_digest: self.profile.profile_digest.to_vec(),
            }),
            sender_service_id: self.scope.sender_service_id.as_bytes().to_vec(),
            message_kind: native_message_kind(self.scope.kind).to_owned(),
            log_id: self.origin.log_id.clone(),
            log_generation: self.origin.log_generation.as_bytes().to_vec(),
            sequence: self.origin.sequence,
            lease_id: self.lease_id.as_bytes().to_vec(),
            message_id: self.origin.message_id.as_bytes().to_vec(),
            wire_sha256: self.wire_sha256.to_vec(),
        };
        let mut client = self.client.clone();
        let response = client
            .acknowledge_one(Request::new(request))
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?
            .into_inner();
        if !response.acknowledged {
            return Err(AdmissionReceiveError::Transport);
        }
        Ok(())
    }

    async fn terminate_permanently(&self) -> Result<(), AdmissionReceiveError> {
        // The owner-neutral receiver persists the permanent poison receipt before this call.
        self.acknowledge().await
    }
}

/// Derive the opaque source key that receiver authorization and quarantine must share.
///
/// This value is deterministic profile metadata, not peer authentication evidence or a secret.
pub fn native_provider_identity(profile: &AdmissionTransportProfile) -> String {
    let mut digest = Sha256::new();
    digest.update(b"star-native-grpc-provider-identity-v1\0");
    for value in [
        profile.provider_id.as_bytes(),
        profile.provider_version.as_bytes(),
        profile.endpoint_identity.as_bytes(),
    ] {
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value);
    }
    digest.update(profile.profile_digest);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut identity = String::with_capacity(64);
    for byte in digest.finalize() {
        identity.push(HEX[(byte >> 4) as usize] as char);
        identity.push(HEX[(byte & 0x0f) as usize] as char);
    }
    identity
}

struct ValidatedNativeDelivery {
    origin: AdmissionReceiveOrigin,
    wire: Vec<u8>,
    lease_id: Uuid,
    wire_sha256: [u8; 32],
}

fn validate_native_delivery(
    scope: AdmissionReceiveScope,
    provider_identity: &str,
    delivery: wire::Delivery,
) -> Result<ValidatedNativeDelivery, AdmissionReceiveError> {
    if !valid_receive_scope(scope)
        || provider_identity.is_empty()
        || provider_identity.len() > 128
        || delivery.log_id.is_empty()
        || delivery.log_id.len() > 128
        || !delivery
            .log_id
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !delivery
            .log_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
        || delivery.sequence <= 0
        || delivery.message_kind != native_message_kind(scope.kind)
        || delivery.wire.is_empty()
        || delivery.wire.len() > 16 * 1024
        || delivery.attempt_count <= 0
        || delivery.lease_expires_unix_ms <= 0
    {
        return Err(AdmissionReceiveError::Contract);
    }
    let sender_service_id = Uuid::from_slice(&delivery.sender_service_id)
        .map_err(|_| AdmissionReceiveError::Contract)?;
    let message_id =
        Uuid::from_slice(&delivery.message_id).map_err(|_| AdmissionReceiveError::Contract)?;
    let log_generation =
        Uuid::from_slice(&delivery.log_generation).map_err(|_| AdmissionReceiveError::Contract)?;
    let lease_id =
        Uuid::from_slice(&delivery.lease_id).map_err(|_| AdmissionReceiveError::Contract)?;
    if sender_service_id != scope.sender_service_id
        || message_id.is_nil()
        || log_generation.is_nil()
        || lease_id.is_nil()
    {
        return Err(AdmissionReceiveError::Contract);
    }
    let wire_sha256: [u8; 32] = delivery
        .wire_sha256
        .as_slice()
        .try_into()
        .map_err(|_| AdmissionReceiveError::Contract)?;
    let actual_sha256: [u8; 32] = Sha256::digest(&delivery.wire).into();
    if wire_sha256 != actual_sha256 {
        return Err(AdmissionReceiveError::Contract);
    }
    Ok(ValidatedNativeDelivery {
        origin: AdmissionReceiveOrigin {
            scope,
            message_id,
            provider_identity: provider_identity.to_owned(),
            log_id: delivery.log_id,
            log_generation,
            sequence: delivery.sequence,
        },
        wire: delivery.wire,
        lease_id,
        wire_sha256,
    })
}

fn valid_receive_scope(scope: AdmissionReceiveScope) -> bool {
    !scope.tenant_id.is_nil()
        && !scope.sender_service_id.is_nil()
        && !scope.recipient_service_id.is_nil()
        && scope.sender_service_id != scope.recipient_service_id
}

fn native_message_kind(kind: AdmissionReceiveKind) -> &'static str {
    match kind {
        AdmissionReceiveKind::Request => "request",
        AdmissionReceiveKind::Result => "result",
        AdmissionReceiveKind::Resolution => "resolution",
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

    fn test_profile() -> AdmissionTransportProfile {
        AdmissionTransportProfile::new(
            NATIVE_PROVIDER_ID.to_owned(),
            "1".to_owned(),
            [7; 32],
            "native-endpoint-a".to_owned(),
        )
        .unwrap()
    }

    fn receive_scope(kind: AdmissionReceiveKind) -> AdmissionReceiveScope {
        AdmissionReceiveScope {
            tenant_id: Uuid::new_v4(),
            sender_service_id: Uuid::new_v4(),
            recipient_service_id: Uuid::new_v4(),
            kind,
        }
    }

    fn native_delivery(scope: AdmissionReceiveScope, payload: Vec<u8>) -> wire::Delivery {
        let wire_sha256: [u8; 32] = Sha256::digest(&payload).into();
        wire::Delivery {
            log_id: "native-log-1".to_owned(),
            log_generation: Uuid::new_v4().as_bytes().to_vec(),
            sequence: 1,
            sender_service_id: scope.sender_service_id.as_bytes().to_vec(),
            message_kind: native_message_kind(scope.kind).to_owned(),
            message_id: Uuid::new_v4().as_bytes().to_vec(),
            wire: payload,
            wire_sha256: wire_sha256.to_vec(),
            lease_id: Uuid::new_v4().as_bytes().to_vec(),
            lease_expires_unix_ms: 1_000_000,
            attempt_count: 1,
        }
    }

    #[test]
    fn native_receive_maps_exact_server_identity_and_wire_proof() {
        let scope = receive_scope(AdmissionReceiveKind::Request);
        let payload = b"bounded exact wire".to_vec();
        let expected_digest: [u8; 32] = Sha256::digest(&payload).into();
        let delivery = native_delivery(scope, payload.clone());
        let provider_identity = native_provider_identity(&test_profile());

        let validated = validate_native_delivery(scope, &provider_identity, delivery).unwrap();

        assert_eq!(validated.origin.scope, scope);
        assert_eq!(validated.origin.provider_identity, provider_identity);
        assert_eq!(validated.origin.sequence, 1);
        assert_eq!(validated.wire, payload);
        assert_eq!(validated.wire_sha256, expected_digest);
        assert!(!validated.lease_id.is_nil());
    }

    #[test]
    fn native_receive_rejects_wrong_route_sender_kind_and_digest() {
        let scope = receive_scope(AdmissionReceiveKind::Result);
        let provider_identity = native_provider_identity(&test_profile());

        let mut wrong_sender = native_delivery(scope, b"wire".to_vec());
        wrong_sender.sender_service_id = Uuid::new_v4().as_bytes().to_vec();
        assert!(matches!(
            validate_native_delivery(scope, &provider_identity, wrong_sender),
            Err(AdmissionReceiveError::Contract)
        ));

        let mut wrong_kind = native_delivery(scope, b"wire".to_vec());
        wrong_kind.message_kind = "request".to_owned();
        assert!(matches!(
            validate_native_delivery(scope, &provider_identity, wrong_kind),
            Err(AdmissionReceiveError::Contract)
        ));

        let mut wrong_digest = native_delivery(scope, b"wire".to_vec());
        wrong_digest.wire_sha256[0] ^= 0xff;
        assert!(matches!(
            validate_native_delivery(scope, &provider_identity, wrong_digest),
            Err(AdmissionReceiveError::Contract)
        ));
    }

    #[test]
    fn native_receive_rejects_malformed_or_oversized_delivery_metadata() {
        let scope = receive_scope(AdmissionReceiveKind::Resolution);
        let provider_identity = native_provider_identity(&test_profile());

        let mut invalid_log = native_delivery(scope, b"wire".to_vec());
        invalid_log.log_id = "../native-log".to_owned();
        assert!(matches!(
            validate_native_delivery(scope, &provider_identity, invalid_log),
            Err(AdmissionReceiveError::Contract)
        ));

        let oversized = native_delivery(scope, vec![b'x'; 16 * 1024 + 1]);
        assert!(matches!(
            validate_native_delivery(scope, &provider_identity, oversized),
            Err(AdmissionReceiveError::Contract)
        ));

        let mut invalid_lease = native_delivery(scope, b"wire".to_vec());
        invalid_lease.lease_id = Uuid::nil().as_bytes().to_vec();
        assert!(matches!(
            validate_native_delivery(scope, &provider_identity, invalid_lease),
            Err(AdmissionReceiveError::Contract)
        ));
    }

    #[test]
    fn native_provider_identity_pins_provider_version_profile_and_endpoint() {
        let profile = test_profile();
        let identity = native_provider_identity(&profile);
        assert_eq!(identity.len(), 64);
        assert!(identity.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(native_provider_identity(&profile), identity);

        let mut different_endpoint = profile.clone();
        different_endpoint.endpoint_identity.push_str("-other");
        assert_ne!(native_provider_identity(&different_endpoint), identity);

        let mut different_version = profile.clone();
        different_version.provider_version.push_str(".next");
        assert_ne!(native_provider_identity(&different_version), identity);

        let mut different_provider = profile.clone();
        different_provider.provider_id = "other-provider".to_owned();
        assert_ne!(native_provider_identity(&different_provider), identity);

        let mut different_digest = profile;
        different_digest.profile_digest[0] ^= 0xff;
        assert_ne!(native_provider_identity(&different_digest), identity);
    }

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
