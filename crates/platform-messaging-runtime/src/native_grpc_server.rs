// @cypher schema=1 source_sha256=a8046029df90a2af872912b126193113f410cb7151d5fdbd52d6da6653e1c952
// MERGE (self:File {path:"crates/platform-messaging-runtime/src/native_grpc_server.rs"})
// MERGE (server:Type {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer",kind:"struct"})
// MERGE (server_error:Type {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcServerError",kind:"enum"})
// MERGE (new:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer.new",kind:"method"})
// MERGE (into_service:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer.into_service",kind:"method"})
// MERGE (try_admit:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer.try_admit",kind:"method"})
// MERGE (append:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer.append",kind:"method"})
// MERGE (pull:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer.pull_one",kind:"method"})
// MERGE (ack:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer.acknowledge_one",kind:"method"})
// MERGE (serve:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::serve_native_grpc",kind:"function"})
// MERGE (authenticated_peer:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::authenticated_peer",kind:"function"})
// MERGE (profile_digest:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::profile_digest",kind:"function"})
// MERGE (uuid_field:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::uuid_field",kind:"function"})
// MERGE (validate_kind:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::validate_message_kind",kind:"function"})
// MERGE (valid_log_id:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::valid_log_id",kind:"function"})
// MERGE (decode_delivery:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::decode_delivery",kind:"function"})
// MERGE (map_database_error:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::map_database_error",kind:"function"})
// MERGE (fingerprint:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::peer_certificate_fingerprint",kind:"function"})
// MERGE (server_tls:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::server_mtls_config",kind:"function"})
// MERGE (proto:File {path:"crates/platform-messaging-proto/proto/native_transport.proto"})
// MERGE (auth:Function {id:"platform_transport.authorize_profile_route"})
// MERGE (append_fn:Function {id:"platform_transport.append_authenticated"})
// MERGE (pull_fn:Function {id:"platform_transport.pull_authenticated"})
// MERGE (ack_fn:Function {id:"platform_transport.acknowledge_authenticated"})
// MERGE (bounded:Config {id:"native_grpc_server::MAX_SERVER_IN_FLIGHT"})
// MERGE (request_deadline:Config {id:"native_grpc_server::SERVER_REQUEST_DEADLINE"})
// MERGE (wire_limit:Config {id:"native_grpc_server::MAX_WIRE_BYTES"})
// MERGE (impl_trait:Type {id:"platform_messaging_proto::wire::durable_transport_server::DurableTransport",kind:"trait"})
// MERGE (profile_digest_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::tests::profile_digest_must_be_exact_sha256_width",kind:"test"})
// MERGE (wire_identity_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::tests::wire_identity_fields_are_bounded_and_typed",kind:"test"})
// MERGE (self)-[:DEFINES]->(server)
// MERGE (self)-[:DEFINES]->(server_error)
// MERGE (self)-[:DEFINES]->(new)
// MERGE (self)-[:DEFINES]->(into_service)
// MERGE (self)-[:DEFINES]->(try_admit)
// MERGE (self)-[:DEFINES]->(append)
// MERGE (self)-[:DEFINES]->(pull)
// MERGE (self)-[:DEFINES]->(ack)
// MERGE (self)-[:DEFINES]->(serve)
// MERGE (self)-[:DEFINES]->(authenticated_peer)
// MERGE (self)-[:DEFINES]->(profile_digest)
// MERGE (self)-[:DEFINES]->(uuid_field)
// MERGE (self)-[:DEFINES]->(validate_kind)
// MERGE (self)-[:DEFINES]->(valid_log_id)
// MERGE (self)-[:DEFINES]->(decode_delivery)
// MERGE (self)-[:DEFINES]->(map_database_error)
// MERGE (self)-[:IMPORTS]->(proto)
// MERGE (self)-[:USES_TYPE]->(impl_trait)
// MERGE (self)-[:USES_TYPE]->(bounded)
// MERGE (self)-[:USES_TYPE]->(request_deadline)
// MERGE (self)-[:USES_TYPE]->(wire_limit)
// MERGE (server)-[:IMPLEMENTS]->(impl_trait)
// MERGE (new)-[:CALLS]->(:ExternalService {id:"tokio::sync::Semaphore::new",kind:"method"})
// MERGE (into_service)-[:CALLS]->(:ExternalService {id:"DurableTransportServer::new",kind:"method"})
// MERGE (try_admit)-[:CALLS]->(:ExternalService {id:"tokio::sync::Semaphore::try_acquire",kind:"method"})
// MERGE (append)-[:CALLS]->(try_admit)
// MERGE (append)-[:CALLS]->(authenticated_peer)
// MERGE (append)-[:CALLS]->(profile_digest)
// MERGE (append)-[:CALLS]->(uuid_field)
// MERGE (append)-[:CALLS]->(validate_kind)
// MERGE (append)-[:CALLS]->(valid_log_id)
// MERGE (append)-[:CALLS]->(append_fn)
// MERGE (pull)-[:CALLS]->(try_admit)
// MERGE (pull)-[:CALLS]->(authenticated_peer)
// MERGE (pull)-[:CALLS]->(profile_digest)
// MERGE (pull)-[:CALLS]->(uuid_field)
// MERGE (pull)-[:CALLS]->(validate_kind)
// MERGE (pull)-[:CALLS]->(decode_delivery)
// MERGE (pull)-[:CALLS]->(pull_fn)
// MERGE (ack)-[:CALLS]->(try_admit)
// MERGE (ack)-[:CALLS]->(authenticated_peer)
// MERGE (ack)-[:CALLS]->(profile_digest)
// MERGE (ack)-[:CALLS]->(uuid_field)
// MERGE (ack)-[:CALLS]->(validate_kind)
// MERGE (ack)-[:CALLS]->(valid_log_id)
// MERGE (ack)-[:CALLS]->(ack_fn)
// MERGE (serve)-[:CALLS]->(server_tls)
// MERGE (serve)-[:CALLS]->(new)
// MERGE (serve)-[:CALLS]->(into_service)
// MERGE (authenticated_peer)-[:CALLS]->(fingerprint)
// MERGE (decode_delivery)-[:CALLS]->(valid_log_id)
// MERGE (decode_delivery)-[:CALLS]->(validate_kind)
// MERGE (decode_delivery)-[:CALLS]->(map_database_error)
// MERGE (self)-[:DEFINES]->(profile_digest_test)
// MERGE (self)-[:DEFINES]->(wire_identity_test)
// MERGE (profile_digest_test)-[:TESTS]->(profile_digest)
// MERGE (wire_identity_test)-[:TESTS]->(uuid_field)
// MERGE (wire_identity_test)-[:TESTS]->(valid_log_id)
// MERGE (wire_identity_test)-[:TESTS]->(validate_kind)
// MERGE (append_fn)-[:CALLS]->(auth)
// MERGE (pull_fn)-[:CALLS]->(auth)
// MERGE (ack_fn)-[:CALLS]->(auth)
// @endcypher
//! Native durable transport gRPC server adapter.

use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

use platform_messaging_proto::wire;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use thiserror::Error;
use tokio::sync::{Semaphore, SemaphorePermit};
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::native_grpc::{
    MAX_NATIVE_GRPC_MESSAGE_BYTES, NativeGrpcSetupError, peer_certificate_fingerprint,
    server_mtls_config,
};

const MAX_SERVER_IN_FLIGHT: usize = 8;
const SERVER_REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const MAX_WIRE_BYTES: usize = 16 * 1024;

/// Native gRPC serving or TLS configuration failure.
#[derive(Debug, Error)]
pub enum NativeGrpcServerError {
    /// TLS key, certificate or client CA material was invalid.
    #[error(transparent)]
    Setup(#[from] NativeGrpcSetupError),
    /// The listener could not be configured or started.
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

/// PostgreSQL-backed server for Star's native, bounded durable transport protocol.
#[derive(Clone)]
pub struct NativeGrpcTransportServer {
    pool: PgPool,
    in_flight: Arc<Semaphore>,
}

impl NativeGrpcTransportServer {
    /// Construct one bounded RPC service over the deployment-managed database pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            in_flight: Arc::new(Semaphore::new(MAX_SERVER_IN_FLIGHT)),
        }
    }

    /// Convert to the generated Tonic service and bound decoded/encoded protobuf frames.
    pub fn into_service(self) -> wire::durable_transport_server::DurableTransportServer<Self> {
        wire::durable_transport_server::DurableTransportServer::new(self)
            .max_decoding_message_size(MAX_NATIVE_GRPC_MESSAGE_BYTES)
            .max_encoding_message_size(MAX_NATIVE_GRPC_MESSAGE_BYTES)
    }

    fn try_admit(&self) -> Result<SemaphorePermit<'_>, Status> {
        self.in_flight.try_acquire().map_err(|_| {
            Status::resource_exhausted("native transport server is at its bounded in-flight limit")
        })
    }
}

#[tonic::async_trait]
impl wire::durable_transport_server::DurableTransport for NativeGrpcTransportServer {
    async fn append(
        &self,
        request: Request<wire::AppendRequest>,
    ) -> Result<Response<wire::AppendResponse>, Status> {
        let _permit = self.try_admit()?;
        let certificate_sha256 = authenticated_peer(&request)?;
        let request = request.into_inner();
        let profile_digest = profile_digest(request.profile)?;
        let recipient_service_id = uuid_field(&request.recipient_service_id, "recipient service")?;
        let message_id = uuid_field(&request.message_id, "message")?;
        validate_message_kind(&request.message_kind)?;
        if request.wire.is_empty() || request.wire.len() > MAX_WIRE_BYTES {
            return Err(Status::invalid_argument(
                "transport wire exceeds its bounded payload size",
            ));
        }
        let payload_sha256: [u8; 32] = Sha256::digest(&request.wire).into();
        if request.wire_sha256.as_slice() != payload_sha256.as_slice() {
            return Err(Status::invalid_argument(
                "transport wire digest does not match the exact payload",
            ));
        }

        let result = tokio::time::timeout(
            SERVER_REQUEST_DEADLINE,
            sqlx::query(
                "SELECT * FROM platform_transport.append_authenticated(\
                 $1,$2,$3,$4,$5,$6,$7)",
            )
            .bind(certificate_sha256.to_vec())
            .bind(profile_digest.to_vec())
            .bind(recipient_service_id)
            .bind(request.message_kind)
            .bind(message_id)
            .bind(request.wire)
            .bind(payload_sha256.to_vec())
            .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("native transport append deadline elapsed"))?
        .map_err(map_database_error)?;
        let log_id: String = result
            .try_get("accepted_log_id")
            .map_err(map_database_error)?;
        if !valid_log_id(&log_id) {
            return Err(Status::data_loss(
                "native transport returned an invalid log receipt",
            ));
        }
        let generation: Uuid = result
            .try_get("accepted_generation")
            .map_err(map_database_error)?;
        let sequence: i64 = result
            .try_get("accepted_sequence")
            .map_err(map_database_error)?;
        let accepted_sha256: Vec<u8> = result
            .try_get("accepted_sha256")
            .map_err(map_database_error)?;
        let exact_replay: bool = result.try_get("exact_replay").map_err(map_database_error)?;
        if sequence <= 0 || accepted_sha256.as_slice() != payload_sha256.as_slice() {
            return Err(Status::data_loss(
                "native transport returned a mismatched durable receipt",
            ));
        }
        Ok(Response::new(wire::AppendResponse {
            log_id,
            log_generation: generation.as_bytes().to_vec(),
            sequence,
            wire_sha256: payload_sha256.to_vec(),
            exact_replay,
        }))
    }

    async fn pull_one(
        &self,
        request: Request<wire::PullOneRequest>,
    ) -> Result<Response<wire::PullOneResponse>, Status> {
        let _permit = self.try_admit()?;
        let certificate_sha256 = authenticated_peer(&request)?;
        let request = request.into_inner();
        let profile_digest = profile_digest(request.profile)?;
        let sender_service_id = uuid_field(&request.sender_service_id, "sender service")?;
        validate_message_kind(&request.message_kind)?;
        if !(30..=120).contains(&request.lease_seconds) {
            return Err(Status::invalid_argument(
                "transport lease must be between 30 and 120 seconds",
            ));
        }
        let result = tokio::time::timeout(
            SERVER_REQUEST_DEADLINE,
            sqlx::query(
                "SELECT p.delivery_log_id,p.delivery_generation,p.delivery_sequence,\
                 p.sender_service_id,p.message_kind,p.message_id,p.wire,p.payload_sha256,\
                 p.lease_id,(extract(epoch FROM p.lease_expires_at)*1000)::BIGINT AS lease_expires_unix_ms,\
                 p.attempt_count FROM platform_transport.pull_authenticated($1,$2,$3,$4,$5) p",
            )
            .bind(certificate_sha256.to_vec())
            .bind(profile_digest.to_vec())
            .bind(sender_service_id)
            .bind(request.message_kind)
            .bind(request.lease_seconds)
            .fetch_optional(&self.pool),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("native transport pull deadline elapsed"))?
        .map_err(map_database_error)?;
        let delivery = result.map(decode_delivery).transpose()?;
        Ok(Response::new(wire::PullOneResponse { delivery }))
    }

    async fn acknowledge_one(
        &self,
        request: Request<wire::AcknowledgeOneRequest>,
    ) -> Result<Response<wire::AcknowledgeOneResponse>, Status> {
        let _permit = self.try_admit()?;
        let certificate_sha256 = authenticated_peer(&request)?;
        let request = request.into_inner();
        let profile_digest = profile_digest(request.profile)?;
        let sender_service_id = uuid_field(&request.sender_service_id, "sender service")?;
        let generation = uuid_field(&request.log_generation, "log generation")?;
        let lease_id = uuid_field(&request.lease_id, "lease")?;
        let message_id = uuid_field(&request.message_id, "message")?;
        validate_message_kind(&request.message_kind)?;
        if !valid_log_id(&request.log_id)
            || request.sequence <= 0
            || request.wire_sha256.len() != 32
        {
            return Err(Status::invalid_argument(
                "invalid native transport acknowledgement boundary",
            ));
        }
        let result = tokio::time::timeout(
            SERVER_REQUEST_DEADLINE,
            sqlx::query(
                "SELECT platform_transport.acknowledge_authenticated(\
                 $1,$2,$3,$4,$5,$6,$7,$8,$9,$10) AS acknowledged",
            )
            .bind(certificate_sha256.to_vec())
            .bind(profile_digest.to_vec())
            .bind(sender_service_id)
            .bind(request.message_kind)
            .bind(request.log_id)
            .bind(generation)
            .bind(request.sequence)
            .bind(lease_id)
            .bind(message_id)
            .bind(request.wire_sha256)
            .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| {
            Status::deadline_exceeded("native transport acknowledgement deadline elapsed")
        })?
        .map_err(map_database_error)?;
        let acknowledged: bool = result.try_get("acknowledged").map_err(map_database_error)?;
        Ok(Response::new(wire::AcknowledgeOneResponse { acknowledged }))
    }
}

/// Start the gRPC endpoint with mandatory client-certificate verification.
pub async fn serve_native_grpc(
    address: SocketAddr,
    pool: PgPool,
    server_certificate: &Path,
    server_private_key: &Path,
    trusted_client_ca: &Path,
) -> Result<(), NativeGrpcServerError> {
    let tls = server_mtls_config(server_certificate, server_private_key, trusted_client_ca)?;
    let service = NativeGrpcTransportServer::new(pool).into_service();
    let mut server = Server::builder()
        .tls_config(tls)?
        .concurrency_limit_per_connection(MAX_SERVER_IN_FLIGHT);
    server.add_service(service).serve(address).await?;
    Ok(())
}

fn authenticated_peer<T>(request: &Request<T>) -> Result<[u8; 32], Status> {
    peer_certificate_fingerprint(request)
        .map_err(|_| Status::unauthenticated("mutual TLS peer certificate is required"))
}

fn profile_digest(profile: Option<wire::ProfilePin>) -> Result<[u8; 32], Status> {
    let digest = profile
        .ok_or_else(|| Status::invalid_argument("transport profile pin is required"))?
        .profile_digest;
    digest
        .try_into()
        .map_err(|_| Status::invalid_argument("transport profile digest must be SHA-256"))
}

fn uuid_field(bytes: &[u8], name: &'static str) -> Result<Uuid, Status> {
    let value = Uuid::from_slice(bytes)
        .map_err(|_| Status::invalid_argument(format!("invalid {name} identity")))?;
    if value.is_nil() {
        return Err(Status::invalid_argument(format!("invalid {name} identity")));
    }
    Ok(value)
}

fn validate_message_kind(value: &str) -> Result<(), Status> {
    match value {
        "request" | "result" | "resolution" => Ok(()),
        _ => Err(Status::invalid_argument(
            "unsupported native transport message kind",
        )),
    }
}

fn valid_log_id(value: &str) -> bool {
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

fn decode_delivery(row: sqlx::postgres::PgRow) -> Result<wire::Delivery, Status> {
    let log_id: String = row.try_get("delivery_log_id").map_err(map_database_error)?;
    let log_generation: Uuid = row
        .try_get("delivery_generation")
        .map_err(map_database_error)?;
    let sequence: i64 = row
        .try_get("delivery_sequence")
        .map_err(map_database_error)?;
    let sender_service_id: Uuid = row
        .try_get("sender_service_id")
        .map_err(map_database_error)?;
    let message_kind: String = row.try_get("message_kind").map_err(map_database_error)?;
    let message_id: Uuid = row.try_get("message_id").map_err(map_database_error)?;
    let wire_bytes: Vec<u8> = row.try_get("wire").map_err(map_database_error)?;
    let payload_sha256: Vec<u8> = row.try_get("payload_sha256").map_err(map_database_error)?;
    let lease_id: Uuid = row.try_get("lease_id").map_err(map_database_error)?;
    let lease_expires_unix_ms: i64 = row
        .try_get("lease_expires_unix_ms")
        .map_err(map_database_error)?;
    let attempt_count: i16 = row.try_get("attempt_count").map_err(map_database_error)?;
    let actual_sha256: [u8; 32] = Sha256::digest(&wire_bytes).into();
    if !valid_log_id(&log_id)
        || sequence <= 0
        || sender_service_id.is_nil()
        || message_id.is_nil()
        || lease_id.is_nil()
        || !(1..=MAX_WIRE_BYTES).contains(&wire_bytes.len())
        || payload_sha256.as_slice() != actual_sha256.as_slice()
        || attempt_count <= 0
    {
        return Err(Status::data_loss(
            "native transport returned an invalid durable delivery",
        ));
    }
    validate_message_kind(&message_kind)
        .map_err(|_| Status::data_loss("native transport returned an invalid message kind"))?;
    Ok(wire::Delivery {
        log_id,
        log_generation: log_generation.as_bytes().to_vec(),
        sequence,
        sender_service_id: sender_service_id.as_bytes().to_vec(),
        message_kind,
        message_id: message_id.as_bytes().to_vec(),
        wire: wire_bytes,
        wire_sha256: actual_sha256.to_vec(),
        lease_id: lease_id.as_bytes().to_vec(),
        lease_expires_unix_ms,
        attempt_count: i32::from(attempt_count),
    })
}

fn map_database_error(error: sqlx::Error) -> Status {
    let code = match &error {
        sqlx::Error::Database(database) => database.code(),
        _ => None,
    };
    match code.as_deref() {
        Some("22023") => Status::invalid_argument("invalid native transport request"),
        Some("42501") => Status::permission_denied("native transport route is not authorized"),
        Some("40001") => Status::aborted("native transport lease or idempotency conflict"),
        Some("54000") => Status::resource_exhausted("native transport route budget is exhausted"),
        Some("57014") => Status::deadline_exceeded("native transport database deadline elapsed"),
        _ => Status::unavailable("native transport durable store is unavailable"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_digest_must_be_exact_sha256_width() {
        assert!(
            profile_digest(Some(wire::ProfilePin {
                profile_digest: vec![1; 32],
            }))
            .is_ok()
        );
        assert!(
            profile_digest(Some(wire::ProfilePin {
                profile_digest: vec![1; 31],
            }))
            .is_err()
        );
        assert!(profile_digest(None).is_err());
    }

    #[test]
    fn wire_identity_fields_are_bounded_and_typed() {
        assert!(uuid_field(Uuid::new_v4().as_bytes(), "message").is_ok());
        assert!(uuid_field(Uuid::nil().as_bytes(), "message").is_err());
        assert!(valid_log_id("run-admission:request"));
        assert!(!valid_log_id("../untrusted"));
        assert!(validate_message_kind("request").is_ok());
        assert!(validate_message_kind("custom").is_err());
    }
}
