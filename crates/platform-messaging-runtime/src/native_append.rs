// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=65686b8a68c06e1693816a761726c1b221ed491a9f01b47bc2089b63962a36a9
MERGE (self:File {path:"crates/platform-messaging-runtime/src/native_append.rs"})
MERGE (append_profile:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile",kind:"struct"})
MERGE (receipt:Type {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendReceipt",kind:"struct"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendError",kind:"enum"})
MERGE (transport:Type {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport",kind:"trait"})
MERGE (append_result:Symbol {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport.append_result",kind:"method"})
MERGE (wire_limit:Config {id:"crates/platform-messaging-runtime/src/native_append.rs::MAX_NATIVE_APPEND_WIRE_BYTES"})
MERGE (uuid:ExternalService {id:"uuid::Uuid",kind:"type"})
MERGE (self)-[:IMPORTS]->(admission_delivery:File {path:"crates/platform-messaging-runtime/src/admission_delivery.rs"})
MERGE (self)-[:IMPORTS]->(uuid)
MERGE (self)-[:DEFINES]->(receipt)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(wire_limit)
MERGE (transport)-[:DEFINES]->(append_result)
MERGE (receipt)-[:USES_TYPE]->(append_profile)
MERGE (append_result)-[:USES_TYPE]->(receipt)
MERGE (append_result)-[:USES_TYPE]->(error)
MERGE (append_result)-[:USES_TYPE]->(uuid)
@endcypher
*/
//! Low-level, owner-neutral interface for one exact native log append.

use std::future::Future;

use thiserror::Error;
use uuid::Uuid;

use crate::admission_delivery::AdmissionTransportProfile;

/// Maximum immutable wire size accepted by the low-level native append API.
pub const MAX_NATIVE_APPEND_WIRE_BYTES: usize = 16 * 1024;

/// Exact-append receipt returned by the native server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAppendReceipt {
    /// Non-secret profile summary, including the pinned endpoint identity and digest.
    pub profile: AdmissionTransportProfile,
    /// Stable native log identity.
    pub log_id: String,
    /// Generation changes when a provider recreates a log with the same name.
    pub log_generation: Uuid,
    /// Positive sequence assigned by the native append server.
    pub sequence: i64,
    /// Whether this append returned the existing receipt for an exact replay.
    pub exact_replay: bool,
    /// SHA-256 digest of the original immutable wire bytes.
    pub wire_sha256: [u8; 32],
}

/// Bounded low-level append failures. Variants never contain payload or credential data.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NativeAppendError {
    /// The recipient, message identity, or exact wire is outside the accepted bounds.
    #[error("invalid native append input")]
    InvalidInput,
    /// All bounded append permits are in use.
    #[error("native append capacity is busy")]
    Busy,
    /// The native append RPC did not complete successfully.
    #[error("native append failed")]
    Transport,
    /// The server response did not prove acceptance of the exact requested wire.
    #[error("native append receipt is invalid")]
    InvalidReceipt,
}

/// Owner-neutral port for appending exact `result` messages to a pinned native log.
pub trait NativeAppendTransport: Send + Sync {
    /// Non-secret profile snapshot pinned when the native client was connected.
    fn profile(&self) -> &AdmissionTransportProfile;

    /// Append the caller's exact immutable wire bytes with a stable idempotency identity.
    fn append_result<'a>(
        &'a self,
        recipient_service_id: Uuid,
        message_id: Uuid,
        wire: &'a [u8],
    ) -> impl Future<Output = Result<NativeAppendReceipt, NativeAppendError>> + Send + 'a;
}
