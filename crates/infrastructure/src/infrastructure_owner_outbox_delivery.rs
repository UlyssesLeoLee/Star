//! PostgreSQL delivery state machine for immutable Infrastructure owner Outbox events.
//!
//! @cypher schema=1 source_sha256=468f523c1f558042c191bf295ba5dd11347c9a8043ee5548c48d900c6f0ec29e
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::infrastructure_owner_outbox_delivery",kind:"module"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"struct"})
//! MERGE (claim:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryClaim",kind:"struct"})
//! MERGE (native_route:Type {id:"infrastructure::NativeInfrastructureDeliveryRoute",kind:"struct"})
//! MERGE (native_wire:Type {id:"infrastructure::NativeInfrastructureDeliveryWire",kind:"struct"})
//! MERGE (quarantine_diagnostic:Type {id:"infrastructure::NativeRoutePinQuarantineDiagnostic",kind:"struct"})
//! MERGE (quarantine_cursor:Type {id:"infrastructure::NativeRoutePinQuarantineCursor",kind:"struct"})
//! MERGE (quarantine_page:Type {id:"infrastructure::NativeRoutePinQuarantinePage",kind:"struct"})
//! MERGE (claim_batch:Type {id:"infrastructure::NativeInfrastructureDeliveryClaimBatch",kind:"struct"})
//! MERGE (route_new:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.from_resolved_endpoint",kind:"method"})
//! MERGE (route_profile_binding:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.from_resolved_profile",kind:"method"})
//! MERGE (route_test_profile_binding:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.from_resolved_profile_for_test",kind:"method"})
//! MERGE (route_tenant:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.tenant_id",kind:"method"})
//! MERGE (route_deployment:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.deployment_id",kind:"method"})
//! MERGE (route_profile:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.profile",kind:"method"})
//! MERGE (route_recipient:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::NativeInfrastructureDeliveryRoute.recipient_service_id",kind:"method"})
//! MERGE (failure:Type {id:"infrastructure::OutboxDeliveryFailureKind",kind:"enum"})
//! MERGE (error:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryError",kind:"enum"})
//! MERGE (claim_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_due",kind:"method"})
//! MERGE (claim_native_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_native_due",kind:"method"})
//! MERGE (diagnose_native_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.diagnose_native_due",kind:"method"})
//! MERGE (diagnose_route_pins:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::diagnose_native_route_pins_in_transaction",kind:"function"})
//! MERGE (quarantine_page_read:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.native_route_pin_quarantine_page",kind:"method"})
//! MERGE (load_quarantine_page:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::load_native_route_pin_quarantine_page",kind:"function"})
//! MERGE (get_or_pin_wire:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.get_or_pin_native_wire",kind:"method"})
//! MERGE (mark_delivered:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.mark_delivered",kind:"method"})
//! MERGE (mark_native_delivered:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.mark_native_delivered",kind:"method"})
//! MERGE (record_failure:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.record_failure",kind:"method"})
//! MERGE (publish_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.publish_due_for_tenant",kind:"method"})
//! MERGE (set_scope:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::set_owner_scope",kind:"function"})
//! MERGE (validate_route:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::validate_native_route",kind:"function"})
//! MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery",class:"Work"})
//! MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox",class:"Transaction"})
//! MERGE (route_pin:Table {id:"infrastructure.capacity_outbox_transport_pin",class:"Transaction"})
//! MERGE (append_receipt:Table {id:"infrastructure.capacity_native_append_receipt",class:"Transaction"})
//! MERGE (quarantine:Table {id:"infrastructure.capacity_native_route_pin_quarantine",class:"Transaction"})
//! MERGE (pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (endpoint:Type {id:"platform_messaging_runtime::transport_endpoint::ResolvedTransportEndpoint",kind:"external_type"})
//! MERGE (append_transport:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAppendTransport",kind:"external_type"})
//! MERGE (native_append_file:File {path:"crates/platform-messaging-runtime/src/native_append.rs"})
//! MERGE (native_grpc_file:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
//! MERGE (transport_profile:Symbol {id:"crates/platform-messaging-runtime/src/native_append.rs::NativeAppendTransport.profile",kind:"method"})
//! MERGE (endpoint_deployment:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.deployment_id",kind:"method"})
//! MERGE (endpoint_tenant:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.tenant_id",kind:"method"})
//! MERGE (endpoint_profile_id:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.profile_id",kind:"method"})
//! MERGE (endpoint_profile_revision:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.profile_revision",kind:"method"})
//! MERGE (endpoint_profile_digest:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.profile_digest",kind:"method"})
//! MERGE (endpoint_identity:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint.endpoint_identity",kind:"method"})
//! MERGE (profile_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile.new",kind:"method"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(store)
//! MERGE (module)-[:DEFINES]->(claim)
//! MERGE (module)-[:DEFINES]->(diagnose_route_pins)
//! MERGE (module)-[:DEFINES]->(load_quarantine_page)
//! MERGE (module)-[:DEFINES]->(native_route)
//! MERGE (module)-[:DEFINES]->(native_wire)
//! MERGE (module)-[:DEFINES]->(quarantine_diagnostic)
//! MERGE (module)-[:DEFINES]->(quarantine_cursor)
//! MERGE (module)-[:DEFINES]->(quarantine_page)
//! MERGE (module)-[:DEFINES]->(claim_batch)
//! MERGE (native_route)-[:DEFINES]->(route_new)
//! MERGE (native_route)-[:DEFINES]->(route_tenant)
//! MERGE (native_route)-[:DEFINES]->(route_deployment)
//! MERGE (native_route)-[:DEFINES]->(route_profile)
//! MERGE (native_route)-[:DEFINES]->(route_recipient)
//! MERGE (module)-[:DEFINES]->(failure)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (store)-[:DEFINES]->(claim_due)
//! MERGE (store)-[:DEFINES]->(claim_native_due)
//! MERGE (store)-[:DEFINES]->(diagnose_native_due)
//! MERGE (store)-[:DEFINES]->(quarantine_page_read)
//! MERGE (store)-[:DEFINES]->(get_or_pin_wire)
//! MERGE (store)-[:DEFINES]->(mark_delivered)
//! MERGE (store)-[:DEFINES]->(mark_native_delivered)
//! MERGE (store)-[:DEFINES]->(record_failure)
//! MERGE (store)-[:USES_TYPE]->(pool)
//! MERGE (route_new)-[:USES_TYPE]->(endpoint)
//! MERGE (route_new)-[:USES_TYPE]->(append_transport)
//! MERGE (native_route)-[:DEFINES]->(route_profile_binding)
//! MERGE (native_route)-[:DEFINES]->(route_test_profile_binding)
//! MERGE (module)-[:IMPORTS]->(native_append_file)
//! MERGE (module)-[:IMPORTS]->(native_grpc_file)
//! MERGE (route_new)-[:CALLS]->(transport_profile)
//! MERGE (route_new)-[:CALLS]->(route_profile_binding)
//! MERGE (route_test_profile_binding)-[:CALLS]->(route_profile_binding)
//! MERGE (route_profile_binding)-[:USES_TYPE]->(endpoint)
//! MERGE (route_profile_binding)-[:CALLS]->(endpoint_deployment)
//! MERGE (route_profile_binding)-[:CALLS]->(endpoint_tenant)
//! MERGE (route_profile_binding)-[:CALLS]->(endpoint_profile_id)
//! MERGE (route_profile_binding)-[:CALLS]->(endpoint_profile_revision)
//! MERGE (route_profile_binding)-[:CALLS]->(endpoint_profile_digest)
//! MERGE (route_profile_binding)-[:CALLS]->(endpoint_identity)
//! MERGE (route_profile_binding)-[:CALLS]->(validate_route)
//! MERGE (validate_route)-[:CALLS]->(profile_new)
//! MERGE (claim_due)-[:CALLS]->(set_scope)
//! MERGE (claim_due)-[:READS]->(route_pin)
//! MERGE (claim_due)-[:WRITES]->(route_pin)
//! MERGE (claim_due)-[:READS]->(delivery)
//! MERGE (claim_due)-[:WRITES]->(delivery)
//! MERGE (claim_due)-[:READS]->(outbox)
//! MERGE (mark_delivered)-[:CALLS]->(set_scope)
//! MERGE (mark_delivered)-[:WRITES]->(delivery)
//! MERGE (claim_native_due)-[:CALLS]->(set_scope)
//! MERGE (claim_native_due)-[:CALLS]->(validate_route)
//! MERGE (claim_native_due)-[:CALLS]->(route_tenant)
//! MERGE (claim_native_due)-[:CALLS]->(diagnose_route_pins)
//! MERGE (claim_native_due)-[:READS]->(delivery)
//! MERGE (claim_native_due)-[:WRITES]->(delivery)
//! MERGE (claim_native_due)-[:READS]->(outbox)
//! MERGE (claim_native_due)-[:READS]->(route_pin)
//! MERGE (claim_native_due)-[:RETURNS]->(claim_batch)
//! MERGE (diagnose_native_due)-[:CALLS]->(set_scope)
//! MERGE (diagnose_native_due)-[:CALLS]->(validate_route)
//! MERGE (diagnose_native_due)-[:CALLS]->(route_tenant)
//! MERGE (diagnose_native_due)-[:CALLS]->(diagnose_route_pins)
//! MERGE (diagnose_native_due)-[:USES_TYPE]->(native_route)
//! MERGE (diagnose_native_due)-[:USES_TYPE]->(pool)
//! MERGE (diagnose_native_due)-[:RETURNS]->(quarantine_page)
//! MERGE (diagnose_route_pins)-[:USES_TYPE]->(native_route)
//! MERGE (diagnose_route_pins)-[:USES_TYPE]->(quarantine_page)
//! MERGE (diagnose_route_pins)-[:CALLS]->(load_quarantine_page)
//! MERGE (diagnose_route_pins)-[:READS]->(delivery)
//! MERGE (diagnose_route_pins)-[:READS]->(route_pin)
//! MERGE (diagnose_route_pins)-[:READS]->(append_receipt)
//! MERGE (diagnose_route_pins)-[:READS]->(quarantine)
//! MERGE (diagnose_route_pins)-[:WRITES]->(delivery)
//! MERGE (diagnose_route_pins)-[:WRITES]->(quarantine)
//! MERGE (diagnose_route_pins)-[:RETURNS]->(quarantine_page)
//! MERGE (quarantine_page_read)-[:CALLS]->(set_scope)
//! MERGE (quarantine_page_read)-[:CALLS]->(load_quarantine_page)
//! MERGE (quarantine_page_read)-[:RETURNS]->(quarantine_page)
//! MERGE (load_quarantine_page)-[:READS]->(quarantine)
//! MERGE (load_quarantine_page)-[:RETURNS]->(quarantine_page)
//! MERGE (get_or_pin_wire)-[:CALLS]->(set_scope)
//! MERGE (get_or_pin_wire)-[:CALLS]->(validate_route)
//! MERGE (get_or_pin_wire)-[:READS]->(delivery)
//! MERGE (get_or_pin_wire)-[:READS]->(route_pin)
//! MERGE (get_or_pin_wire)-[:WRITES]->(route_pin)
//! MERGE (mark_native_delivered)-[:CALLS]->(set_scope)
//! MERGE (mark_native_delivered)-[:READS]->(route_pin)
//! MERGE (mark_native_delivered)-[:WRITES]->(delivery)
//! MERGE (mark_native_delivered)-[:WRITES]->(append_receipt)
//! MERGE (record_failure)-[:CALLS]->(set_scope)
//! MERGE (record_failure)-[:WRITES]->(delivery)
//! MERGE (publish_due)-[:CALLS]->(claim_due)
//! MERGE (publish_due)-[:CALLS]->(mark_delivered)
//! MERGE (publish_due)-[:CALLS]->(record_failure)
//! @endcypher

use sqlx::{postgres::PgRow, PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "native-grpc-adapter")]
use platform_messaging_runtime::{
    admission_delivery::AdmissionTransportProfile,
    native_append::{NativeAppendReceipt, NativeAppendTransport},
    native_grpc::NativeGrpcAppendTransport,
    transport_endpoint::ResolvedTransportEndpoint,
};
#[cfg(feature = "native-grpc-adapter")]
use sha2::{Digest, Sha256};

#[cfg(feature = "native-grpc-adapter")]
use crate::infrastructure_owner_outbox_wire::build_capacity_result_wire;

const OWNER_ROLE: &str = "SET LOCAL ROLE star_infrastructure_owner";
const MAX_BATCH_SIZE: i64 = 64;
const MAX_LEASE_SECONDS: i64 = 300;
const MAX_DELIVERY_ATTEMPTS: i32 = 12;
const MAX_BACKOFF_SECONDS: i64 = 3_600;
#[cfg(feature = "native-grpc-adapter")]
const MAX_NATIVE_BATCH_SIZE: i64 = 8;
#[cfg(feature = "native-grpc-adapter")]
const MAX_NATIVE_ROUTE_PIN_QUARANTINE_BATCH: i64 = 64;
#[cfg(feature = "native-grpc-adapter")]
const MAX_NATIVE_ROUTE_PIN_DIAGNOSTIC_PAGE_SIZE: i64 = 32;
#[cfg(feature = "native-grpc-adapter")]
const MAX_NATIVE_PINNED_DUE_SAMPLE: i64 = 64;
#[cfg(feature = "native-grpc-adapter")]
const MIN_NATIVE_LEASE_SECONDS: i64 = 120;
#[cfg(feature = "native-grpc-adapter")]
const MAX_NATIVE_LEASE_SECONDS: i64 = 300;

/// PostgreSQL owner-side delivery store for immutable Outbox facts.
#[derive(Debug, Clone)]
pub struct PostgresInfrastructureOwnerOutboxDelivery {
    pool: PgPool,
}

/// Immutable event envelope plus the lease fence acquired for this delivery attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct InfrastructureOwnerOutboxDeliveryClaim {
    /// Tenant partition owning the event.
    pub tenant_id: Uuid,
    /// Immutable owner Outbox event identity.
    pub event_id: Uuid,
    /// Worker identity used to acquire this lease.
    pub lease_owner_id: Uuid,
    /// Monotonic attempt fence; stale attempts cannot acknowledge a reclaimed lease.
    pub attempt_count: i32,
    /// Typed event name from the immutable Outbox row.
    pub event_type: String,
    /// Version of the event payload contract.
    pub schema_version: i16,
    /// Stable idempotency key for downstream consumers.
    pub idempotency_key: String,
    /// Cross-owner tracing identity.
    pub correlation_id: Uuid,
    /// Bounded serialized JSON body; claim does not expand it into a heap object graph.
    pub payload_json: String,
    /// Database-clock lease deadline.
    pub lease_expires_at: chrono::DateTime<chrono::Utc>,
}

/// Trusted route identity fixed for the lifetime of one native Infrastructure delivery.
#[cfg(feature = "native-grpc-adapter")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeInfrastructureDeliveryRoute {
    /// Deployment identity from the trusted profile endpoint resolver.
    deployment_id: Uuid,
    /// Tenant identity from the trusted profile endpoint resolver.
    tenant_id: Uuid,
    /// Immutable profile identity resolved by trusted deployment configuration.
    profile_id: Uuid,
    /// Immutable profile revision resolved by trusted deployment configuration.
    profile_revision: i64,
    /// Non-secret provider/profile snapshot used by the native client.
    profile: AdmissionTransportProfile,
    /// Authenticated Infrastructure sender identity.
    sender_service_id: Uuid,
    /// Authorized Run-owner result recipient.
    recipient_service_id: Uuid,
}

#[cfg(feature = "native-grpc-adapter")]
impl NativeInfrastructureDeliveryRoute {
    /// Bind tenant and profile identity to the trusted endpoint used by the native client.
    ///
    /// The endpoint must come from `TrustedEndpointResolver::resolve`, and `transport` must be
    /// the already connected native gRPC client used for appends. Its profile digest and endpoint
    /// identity must match the resolver snapshot before the route can be constructed. This stays
    /// crate-private until production startup has a trusted endpoint and credential resolver.
    pub(crate) fn from_resolved_endpoint(
        endpoint: &ResolvedTransportEndpoint,
        transport: &NativeGrpcAppendTransport,
        sender_service_id: Uuid,
        recipient_service_id: Uuid,
    ) -> Result<Self, InfrastructureOwnerOutboxDeliveryError> {
        Self::from_resolved_profile(
            endpoint,
            transport.profile().clone(),
            sender_service_id,
            recipient_service_id,
        )
    }

    fn from_resolved_profile(
        endpoint: &ResolvedTransportEndpoint,
        profile: AdmissionTransportProfile,
        sender_service_id: Uuid,
        recipient_service_id: Uuid,
    ) -> Result<Self, InfrastructureOwnerOutboxDeliveryError> {
        if endpoint.deployment_id().is_nil()
            || endpoint.tenant_id().is_nil()
            || endpoint.profile_id().is_nil()
            || endpoint.profile_revision() <= 0
            || profile.profile_digest != endpoint.profile_digest()
            || profile.endpoint_identity.as_str() != endpoint.endpoint_identity()
        {
            return Err(InfrastructureOwnerOutboxDeliveryError::RouteConflict);
        }

        let route = Self {
            deployment_id: endpoint.deployment_id(),
            tenant_id: endpoint.tenant_id(),
            profile_id: endpoint.profile_id(),
            profile_revision: endpoint.profile_revision(),
            profile,
            sender_service_id,
            recipient_service_id,
        };
        validate_native_route(&route)?;
        Ok(route)
    }

    #[cfg(any(test, feature = "native-grpc-test-support"))]
    /// Build a route snapshot for store-level tests from a resolver-produced endpoint.
    ///
    /// This test-only feature does not construct a connected transport or prove a durable append.
    pub fn from_resolved_profile_for_test(
        endpoint: &ResolvedTransportEndpoint,
        profile: AdmissionTransportProfile,
        sender_service_id: Uuid,
        recipient_service_id: Uuid,
    ) -> Result<Self, InfrastructureOwnerOutboxDeliveryError> {
        Self::from_resolved_profile(endpoint, profile, sender_service_id, recipient_service_id)
    }

    /// Deployment identity bound to the resolved native transport profile.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    /// Tenant identity bound to the resolved native transport profile.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Immutable profile identity bound to the resolved native transport profile.
    pub fn profile_id(&self) -> Uuid {
        self.profile_id
    }

    /// Immutable profile revision bound to the resolved native transport profile.
    pub fn profile_revision(&self) -> i64 {
        self.profile_revision
    }

    /// Non-secret transport profile snapshot bound by the endpoint resolver.
    pub fn profile(&self) -> &AdmissionTransportProfile {
        &self.profile
    }

    /// Authorized Run-owner result recipient.
    pub fn recipient_service_id(&self) -> Uuid {
        self.recipient_service_id
    }

    /// Authenticated Infrastructure sender identity.
    pub fn sender_service_id(&self) -> Uuid {
        self.sender_service_id
    }
}

/// Exact route-pinned bytes used for this and every later append retry.
#[cfg(feature = "native-grpc-adapter")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeInfrastructureDeliveryWire {
    /// Immutable serialized owner event bytes.
    pub payload: Vec<u8>,
    /// SHA-256 of `payload` persisted with the immutable route pin.
    pub sha256: [u8; 32],
}

/// A bounded, non-secret description of one immutable native route-pin mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRoutePinQuarantineDiagnostic {
    /// Immutable Infrastructure Outbox event identity.
    pub event_id: Uuid,
    /// Allow-listed route identity fields that differed when the row was quarantined.
    pub mismatch_fields: Vec<String>,
    /// Resolved profile identity that was in use when the mismatch was observed.
    pub observed_profile_id: Uuid,
    /// Resolved profile revision that was in use when the mismatch was observed.
    pub observed_profile_revision: i64,
    /// Database-clock time when the immutable diagnostic fact was appended.
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Stable cursor for bounded operator diagnostics; ordered by time then immutable event ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeRoutePinQuarantineCursor {
    /// Timestamp from the last returned diagnostic.
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// Event ID from the last returned diagnostic.
    pub event_id: Uuid,
}

/// One tenant-scoped page of route-pin quarantine state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeRoutePinQuarantinePage {
    /// Total quarantined native route pins for the resolved tenant.
    pub total_blocked: i64,
    /// Bounded diagnostics, never including payload bytes or transport error text.
    pub entries: Vec<NativeRoutePinQuarantineDiagnostic>,
    /// Cursor for the next page when more diagnostics remain.
    pub next_cursor: Option<NativeRoutePinQuarantineCursor>,
    /// Number of new quarantine facts appended by the current diagnostic pass.
    pub newly_quarantined: usize,
    /// Number of due native-pinned rows observed, capped to a bounded sample.
    pub due_native_pinned: usize,
    /// Whether more due native-pinned rows exist beyond the bounded sample.
    pub due_native_pinned_truncated: bool,
}

/// Native delivery claims together with route-pin operational state from the same owner transaction.
#[cfg(feature = "native-grpc-adapter")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeInfrastructureDeliveryClaimBatch {
    /// Due claims on the exact immutable native route.
    pub claims: Vec<InfrastructureOwnerOutboxDeliveryClaim>,
    /// Bounded blocked-state evidence for this resolved tenant.
    pub route_pin_quarantine: NativeRoutePinQuarantinePage,
}

/// Classification used to choose retry or terminal-dead behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxDeliveryFailureKind {
    /// A transport or dependency failure may succeed on a later attempt.
    Transient,
    /// The event cannot be delivered and should become terminal immediately.
    Permanent,
}

/// Errors returned by the Infrastructure Outbox delivery state machine.
#[derive(Debug, Error)]
pub enum InfrastructureOwnerOutboxDeliveryError {
    /// A tenant, worker, batch, lease, or claim fence is invalid.
    #[error("Infrastructure Outbox delivery parameters are invalid")]
    InvalidParameters,
    /// The lease expired, was reclaimed, or belongs to another worker/attempt.
    #[error("Infrastructure Outbox delivery lease was lost or expired")]
    LeaseLost,
    /// The Outbox event has an immutable route that differs from the selected provider/profile.
    #[error("Infrastructure Outbox event is pinned to a different transport route")]
    RouteConflict,
    /// A native result event cannot satisfy its versioned wire contract or size limit.
    #[error("Infrastructure capacity-result event is invalid for native delivery")]
    NativeWireInvalid,
    /// The native server's receipt does not prove the exact pinned route and bytes.
    #[error("Infrastructure native append receipt does not match the pinned delivery")]
    NativeReceiptInvalid,
    /// PostgreSQL rejected the owner transaction.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug)]
struct ClaimedOutboxRow {
    tenant_id: Uuid,
    event_id: Uuid,
    attempt_count: i32,
    event_type: String,
    schema_version: i16,
    idempotency_key: String,
    correlation_id: Uuid,
    payload_json: String,
    lease_expires_at: chrono::DateTime<chrono::Utc>,
}

impl<'r> sqlx::FromRow<'r, PgRow> for ClaimedOutboxRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            tenant_id: row.try_get("tenant_id")?,
            event_id: row.try_get("event_id")?,
            attempt_count: row.try_get("attempt_count")?,
            event_type: row.try_get("event_type")?,
            schema_version: row.try_get("schema_version")?,
            idempotency_key: row.try_get("idempotency_key")?,
            correlation_id: row.try_get("correlation_id")?,
            payload_json: row.try_get("payload_json")?,
            lease_expires_at: row.try_get("lease_expires_at")?,
        })
    }
}

impl PostgresInfrastructureOwnerOutboxDelivery {
    /// Construct the store over the application's PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Atomically lease due events using database time and `SKIP LOCKED`.
    ///
    /// Expired leases can be reclaimed. Each reclaim increments `attempt_count`, which fences
    /// acknowledgements from the previous owner. Batch size and lease duration are bounded to
    /// keep owner workers' memory and database occupancy predictable.
    pub async fn claim_due(
        &self,
        tenant_id: Uuid,
        lease_owner_id: Uuid,
        max_items: i64,
        lease_seconds: i64,
    ) -> Result<Vec<InfrastructureOwnerOutboxDeliveryClaim>, InfrastructureOwnerOutboxDeliveryError>
    {
        if tenant_id.is_nil()
            || lease_owner_id.is_nil()
            || !(1..=MAX_BATCH_SIZE).contains(&max_items)
            || !(1..=MAX_LEASE_SECONDS).contains(&lease_seconds)
        {
            return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
        }

        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, tenant_id).await?;
        sqlx::query(
            r#"WITH exhausted AS (
                   SELECT d.tenant_id, d.event_id
                     FROM infrastructure.capacity_outbox_delivery AS d
                    WHERE d.tenant_id = $1
                      AND d.delivery_state = 'leased'
                      AND d.lease_expires_at <= clock_timestamp()
                      AND d.attempt_count >= $3
                     AND NOT EXISTS (
                           SELECT 1
                             FROM infrastructure.capacity_outbox_transport_pin AS route
                            WHERE route.tenant_id = d.tenant_id
                              AND route.event_id = d.event_id
                              AND route.transport_provider = 'native-grpc'
                     )
                    ORDER BY d.lease_expires_at, d.event_id
                    LIMIT $2
                    FOR UPDATE OF d SKIP LOCKED
               )
               UPDATE infrastructure.capacity_outbox_delivery AS d
                  SET delivery_state = 'dead',
                      lease_owner_id = NULL,
                      lease_expires_at = NULL,
                      terminal_at = clock_timestamp(),
                      retention_expires_at = clock_timestamp() + interval '24 hours',
                      updated_at = clock_timestamp()
                 FROM exhausted
                WHERE d.tenant_id = exhausted.tenant_id
                  AND d.event_id = exhausted.event_id"#,
        )
        .bind(tenant_id)
        .bind(max_items)
        .bind(MAX_DELIVERY_ATTEMPTS)
        .execute(&mut *tx)
        .await?;
        let rows: Vec<ClaimedOutboxRow> = sqlx::query_as(
            r#"WITH due AS (
                   SELECT d.tenant_id, d.event_id
                     FROM infrastructure.capacity_outbox_delivery AS d
                     WHERE d.tenant_id = $1
                       AND NOT EXISTS (
                           SELECT 1
                             FROM infrastructure.capacity_outbox_transport_pin AS route
                            WHERE route.tenant_id = d.tenant_id
                              AND route.event_id = d.event_id
                              AND route.transport_provider = 'native-grpc'
                     )
                      AND ((d.delivery_state IN ('pending','retry')
                            AND d.next_attempt_at <= clock_timestamp())
                        OR (d.delivery_state = 'leased'
                            AND d.lease_expires_at <= clock_timestamp()
                            AND d.attempt_count < $5))
                    ORDER BY CASE WHEN d.delivery_state = 'leased'
                                  THEN d.lease_expires_at ELSE d.next_attempt_at END,
                             d.event_id
                    LIMIT $3
                    FOR UPDATE OF d SKIP LOCKED
               ), claimed AS (
                   UPDATE infrastructure.capacity_outbox_delivery AS d
                      SET delivery_state = 'leased',
                          attempt_count = d.attempt_count + 1,
                          lease_owner_id = $2,
                          lease_expires_at = clock_timestamp()
                              + make_interval(secs => $4::double precision),
                          updated_at = clock_timestamp()
                     FROM due
                    WHERE d.tenant_id = due.tenant_id
                      AND d.event_id = due.event_id
                 RETURNING d.tenant_id, d.event_id, d.attempt_count, d.lease_expires_at
               )
               SELECT c.tenant_id, c.event_id, c.attempt_count, c.lease_expires_at,
                      o.event_type, o.schema_version, o.idempotency_key,
                      o.correlation_id, o.payload::text AS payload_json
                 FROM claimed AS c
                 JOIN infrastructure.capacity_owner_outbox AS o
                   ON o.tenant_id = c.tenant_id AND o.event_id = c.event_id
                ORDER BY c.event_id"#,
        )
        .bind(tenant_id)
        .bind(lease_owner_id)
        .bind(max_items)
        .bind(lease_seconds)
        .bind(MAX_DELIVERY_ATTEMPTS)
        .fetch_all(&mut *tx)
        .await?;
        for row in &rows {
            sqlx::query(
                "INSERT INTO infrastructure.capacity_outbox_transport_pin \
                 (tenant_id, event_id, transport_provider) \
                 VALUES ($1, $2, 'nats-jetstream') ON CONFLICT (tenant_id, event_id) DO NOTHING",
            )
            .bind(row.tenant_id)
            .bind(row.event_id)
            .execute(&mut *tx)
            .await?;
            let provider: String = sqlx::query_scalar(
                "SELECT transport_provider FROM infrastructure.capacity_outbox_transport_pin \
                 WHERE tenant_id = $1 AND event_id = $2",
            )
            .bind(row.tenant_id)
            .bind(row.event_id)
            .fetch_one(&mut *tx)
            .await?;
            if provider != "nats-jetstream" {
                return Err(InfrastructureOwnerOutboxDeliveryError::RouteConflict);
            }
        }
        tx.commit().await?;

        Ok(rows
            .into_iter()
            .map(|row| InfrastructureOwnerOutboxDeliveryClaim {
                tenant_id: row.tenant_id,
                event_id: row.event_id,
                lease_owner_id,
                attempt_count: row.attempt_count,
                event_type: row.event_type,
                schema_version: row.schema_version,
                idempotency_key: row.idempotency_key,
                correlation_id: row.correlation_id,
                payload_json: row.payload_json,
                lease_expires_at: row.lease_expires_at,
            })
            .collect())
    }

    /// Inspect due native-pinned work and durably quarantine stale route pins.
    ///
    /// This path does not lease or publish any event. The quarantine fact is append-only and
    /// tenant scoped; it never rewrites the route pin or replaces an active lease or receipt.
    #[cfg(feature = "native-grpc-adapter")]
    pub async fn diagnose_native_due(
        &self,
        lease_owner_id: Uuid,
        route: &NativeInfrastructureDeliveryRoute,
    ) -> Result<NativeRoutePinQuarantinePage, InfrastructureOwnerOutboxDeliveryError> {
        validate_native_route(route)?;
        if lease_owner_id.is_nil() {
            return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
        }

        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, route.tenant_id()).await?;
        let diagnostics =
            diagnose_native_route_pins_in_transaction(&mut tx, route, lease_owner_id).await?;
        tx.commit().await?;
        Ok(diagnostics)
    }

    /// Quarantine stale native route pins, lease due events, and return bounded diagnostics.
    ///
    /// The quarantine fact is append-only and tenant scoped. It never rewrites the route pin, never
    /// replaces an active lease or receipt, and does not expose an administrative recovery path.
    /// Returning this batch from the primary claim path distinguishes no due events from work
    /// quarantined because its immutable route pin no longer matches.
    #[cfg(feature = "native-grpc-adapter")]
    pub async fn claim_native_due(
        &self,
        lease_owner_id: Uuid,
        max_items: i64,
        lease_seconds: i64,
        route: &NativeInfrastructureDeliveryRoute,
    ) -> Result<NativeInfrastructureDeliveryClaimBatch, InfrastructureOwnerOutboxDeliveryError>
    {
        validate_native_route(route)?;
        let tenant_id = route.tenant_id();
        if lease_owner_id.is_nil()
            || !(1..=MAX_NATIVE_BATCH_SIZE).contains(&max_items)
            || !(MIN_NATIVE_LEASE_SECONDS..=MAX_NATIVE_LEASE_SECONDS).contains(&lease_seconds)
        {
            return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
        }

        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, tenant_id).await?;
        let route_pin_quarantine =
            diagnose_native_route_pins_in_transaction(&mut tx, route, lease_owner_id).await?;

        sqlx::query(
            r#"WITH exhausted AS (
                   SELECT d.tenant_id, d.event_id
                     FROM infrastructure.capacity_outbox_delivery AS d
                    WHERE d.tenant_id = $1
                      AND d.delivery_state = 'leased'
                      AND d.lease_expires_at <= clock_timestamp()
                      AND d.attempt_count >= $3
                      AND NOT EXISTS (
                           SELECT 1
                             FROM infrastructure.capacity_outbox_transport_pin AS route
                            WHERE route.tenant_id = d.tenant_id
                              AND route.event_id = d.event_id
                              AND (route.transport_provider <> 'native-grpc'
                                    OR route.profile_id IS DISTINCT FROM $4
                                    OR route.profile_revision IS DISTINCT FROM $5
                                    OR route.provider_id IS DISTINCT FROM $6
                                    OR route.provider_version IS DISTINCT FROM $7
                                    OR route.profile_digest IS DISTINCT FROM $8
                                    OR route.endpoint_identity IS DISTINCT FROM $9
                                    OR route.sender_service_id IS DISTINCT FROM $10
                                    OR route.recipient_service_id IS DISTINCT FROM $11
                                   OR route.message_kind IS DISTINCT FROM 'result')
                      )
                    ORDER BY d.lease_expires_at, d.event_id
                    LIMIT $2
                    FOR UPDATE OF d SKIP LOCKED
               )
               UPDATE infrastructure.capacity_outbox_delivery AS d
                  SET delivery_state = 'dead',
                      lease_owner_id = NULL,
                      lease_expires_at = NULL,
                      terminal_at = clock_timestamp(),
                      retention_expires_at = clock_timestamp() + interval '24 hours',
                      updated_at = clock_timestamp()
                 FROM exhausted
                WHERE d.tenant_id = exhausted.tenant_id
                  AND d.event_id = exhausted.event_id"#,
        )
        .bind(tenant_id)
        .bind(max_items)
        .bind(MAX_DELIVERY_ATTEMPTS)
        .bind(route.profile_id)
        .bind(route.profile_revision)
        .bind(&route.profile.provider_id)
        .bind(&route.profile.provider_version)
        .bind(route.profile.profile_digest.to_vec())
        .bind(&route.profile.endpoint_identity)
        .bind(route.sender_service_id)
        .bind(route.recipient_service_id)
        .execute(&mut *tx)
        .await?;

        let rows: Vec<ClaimedOutboxRow> = sqlx::query_as(
            r#"WITH due AS (
                   SELECT d.tenant_id, d.event_id
                     FROM infrastructure.capacity_outbox_delivery AS d
                    WHERE d.tenant_id = $1
                      AND NOT EXISTS (
                           SELECT 1
                             FROM infrastructure.capacity_outbox_transport_pin AS route
                            WHERE route.tenant_id = d.tenant_id
                              AND route.event_id = d.event_id
                              AND (route.transport_provider <> 'native-grpc'
                                   OR route.profile_id IS DISTINCT FROM $6
                                   OR route.profile_revision IS DISTINCT FROM $7
                                   OR route.provider_id IS DISTINCT FROM $8
                                   OR route.provider_version IS DISTINCT FROM $9
                                   OR route.profile_digest IS DISTINCT FROM $10
                                   OR route.endpoint_identity IS DISTINCT FROM $11
                                   OR route.sender_service_id IS DISTINCT FROM $12
                                   OR route.recipient_service_id IS DISTINCT FROM $13
                                   OR route.message_kind IS DISTINCT FROM 'result')
                      )
                       AND NOT EXISTS (
                            SELECT 1
                              FROM infrastructure.capacity_native_route_pin_quarantine AS q
                             WHERE q.tenant_id = d.tenant_id AND q.event_id = d.event_id)
                      AND ((d.delivery_state IN ('pending','retry')
                            AND d.next_attempt_at <= clock_timestamp())
                        OR (d.delivery_state = 'leased'
                            AND d.lease_expires_at <= clock_timestamp()
                            AND d.attempt_count < $5))
                    ORDER BY CASE WHEN d.delivery_state = 'leased'
                                  THEN d.lease_expires_at ELSE d.next_attempt_at END,
                             d.event_id
                    LIMIT $3
                    FOR UPDATE OF d SKIP LOCKED
               ), claimed AS (
                   UPDATE infrastructure.capacity_outbox_delivery AS d
                      SET delivery_state = 'leased',
                          attempt_count = d.attempt_count + 1,
                          lease_owner_id = $2,
                          lease_expires_at = clock_timestamp()
                              + make_interval(secs => $4::double precision),
                          updated_at = clock_timestamp()
                     FROM due
                    WHERE d.tenant_id = due.tenant_id
                      AND d.event_id = due.event_id
                 RETURNING d.tenant_id, d.event_id, d.attempt_count, d.lease_expires_at
               )
               SELECT c.tenant_id, c.event_id, c.attempt_count, c.lease_expires_at,
                      o.event_type, o.schema_version, o.idempotency_key,
                      o.correlation_id, o.payload::text AS payload_json
                 FROM claimed AS c
                 JOIN infrastructure.capacity_owner_outbox AS o
                   ON o.tenant_id = c.tenant_id AND o.event_id = c.event_id
                ORDER BY c.event_id"#,
        )
        .bind(tenant_id)
        .bind(lease_owner_id)
        .bind(max_items)
        .bind(lease_seconds)
        .bind(MAX_DELIVERY_ATTEMPTS)
        .bind(route.profile_id)
        .bind(route.profile_revision)
        .bind(&route.profile.provider_id)
        .bind(&route.profile.provider_version)
        .bind(route.profile.profile_digest.to_vec())
        .bind(&route.profile.endpoint_identity)
        .bind(route.sender_service_id)
        .bind(route.recipient_service_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(NativeInfrastructureDeliveryClaimBatch {
            claims: rows
                .into_iter()
                .map(|row| InfrastructureOwnerOutboxDeliveryClaim {
                    tenant_id: row.tenant_id,
                    event_id: row.event_id,
                    lease_owner_id,
                    attempt_count: row.attempt_count,
                    event_type: row.event_type,
                    schema_version: row.schema_version,
                    idempotency_key: row.idempotency_key,
                    correlation_id: row.correlation_id,
                    payload_json: row.payload_json,
                    lease_expires_at: row.lease_expires_at,
                })
                .collect(),
            route_pin_quarantine,
        })
    }

    /// Read one bounded page of immutable route-pin quarantine diagnostics for the route tenant.
    #[cfg(feature = "native-grpc-adapter")]
    pub async fn native_route_pin_quarantine_page(
        &self,
        route: &NativeInfrastructureDeliveryRoute,
        cursor: Option<NativeRoutePinQuarantineCursor>,
        limit: i64,
    ) -> Result<NativeRoutePinQuarantinePage, InfrastructureOwnerOutboxDeliveryError> {
        validate_native_route(route)?;
        if !(1..=MAX_NATIVE_ROUTE_PIN_DIAGNOSTIC_PAGE_SIZE).contains(&limit) {
            return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
        }
        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, route.tenant_id()).await?;
        let page = load_native_route_pin_quarantine_page(&mut tx, route.tenant_id(), cursor, limit)
            .await?;
        tx.commit().await?;
        Ok(page)
    }

    /// Load exact previously pinned bytes, or persist the first native route and wire under fence.
    #[cfg(feature = "native-grpc-adapter")]
    pub async fn get_or_pin_native_wire(
        &self,
        claim: &InfrastructureOwnerOutboxDeliveryClaim,
        route: &NativeInfrastructureDeliveryRoute,
    ) -> Result<NativeInfrastructureDeliveryWire, InfrastructureOwnerOutboxDeliveryError> {
        validate_claim(claim)?;
        validate_native_route(route)?;
        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, claim.tenant_id).await?;
        let current_lease = sqlx::query(
            "SELECT 1 FROM infrastructure.capacity_outbox_delivery \
             WHERE tenant_id = $1 AND event_id = $2 AND delivery_state = 'leased' \
               AND lease_owner_id = $3 AND attempt_count = $4 \
               AND lease_expires_at > clock_timestamp() FOR UPDATE",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .bind(claim.lease_owner_id)
        .bind(claim.attempt_count)
        .fetch_optional(&mut *tx)
        .await?;
        if current_lease.is_none() {
            return Err(InfrastructureOwnerOutboxDeliveryError::LeaseLost);
        }

        let existing = sqlx::query(
            "SELECT transport_provider, profile_id, profile_revision, provider_id, \
                    provider_version, profile_digest, endpoint_identity, sender_service_id, \
                    recipient_service_id, message_kind, wire_payload, wire_sha256 \
             FROM infrastructure.capacity_outbox_transport_pin \
             WHERE tenant_id = $1 AND event_id = $2",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .fetch_optional(&mut *tx)
        .await?;

        let wire = if let Some(row) = existing {
            let matches_route = row.try_get::<String, _>("transport_provider")? == "native-grpc"
                && row.try_get::<Option<Uuid>, _>("profile_id")? == Some(route.profile_id)
                && row.try_get::<Option<i64>, _>("profile_revision")?
                    == Some(route.profile_revision)
                && row.try_get::<Option<String>, _>("provider_id")?
                    == Some(route.profile.provider_id.clone())
                && row.try_get::<Option<String>, _>("provider_version")?
                    == Some(route.profile.provider_version.clone())
                && row.try_get::<Option<Vec<u8>>, _>("profile_digest")?
                    == Some(route.profile.profile_digest.to_vec())
                && row.try_get::<Option<String>, _>("endpoint_identity")?
                    == Some(route.profile.endpoint_identity.clone())
                && row.try_get::<Option<Uuid>, _>("sender_service_id")?
                    == Some(route.sender_service_id)
                && row.try_get::<Option<Uuid>, _>("recipient_service_id")?
                    == Some(route.recipient_service_id)
                && row.try_get::<Option<String>, _>("message_kind")? == Some("result".to_owned());
            if !matches_route {
                return Err(InfrastructureOwnerOutboxDeliveryError::RouteConflict);
            }
            let payload: Vec<u8> = row.try_get("wire_payload")?;
            let digest: Vec<u8> = row.try_get("wire_sha256")?;
            let digest: [u8; 32] = digest
                .as_slice()
                .try_into()
                .map_err(|_| InfrastructureOwnerOutboxDeliveryError::NativeReceiptInvalid)?;
            let actual: [u8; 32] = Sha256::digest(&payload).into();
            if payload.is_empty() || payload.len() > 16 * 1024 || digest != actual {
                return Err(InfrastructureOwnerOutboxDeliveryError::NativeReceiptInvalid);
            }
            NativeInfrastructureDeliveryWire {
                payload,
                sha256: digest,
            }
        } else {
            let payload = build_capacity_result_wire(claim)
                .map_err(|_| InfrastructureOwnerOutboxDeliveryError::NativeWireInvalid)?;
            if payload.is_empty() || payload.len() > 16 * 1024 {
                return Err(InfrastructureOwnerOutboxDeliveryError::NativeWireInvalid);
            }
            let sha256: [u8; 32] = Sha256::digest(&payload).into();
            sqlx::query(
                "INSERT INTO infrastructure.capacity_outbox_transport_pin \
                 (tenant_id, event_id, transport_provider, profile_id, profile_revision, \
                  provider_id, provider_version, profile_digest, endpoint_identity, \
                  sender_service_id, recipient_service_id, message_kind, wire_payload, wire_sha256) \
                 VALUES ($1, $2, 'native-grpc', $3, $4, $5, $6, $7, $8, $9, $10, \
                         'result', $11, $12)",
            )
            .bind(claim.tenant_id)
            .bind(claim.event_id)
            .bind(route.profile_id)
            .bind(route.profile_revision)
            .bind(&route.profile.provider_id)
            .bind(&route.profile.provider_version)
            .bind(route.profile.profile_digest.to_vec())
            .bind(&route.profile.endpoint_identity)
            .bind(route.sender_service_id)
            .bind(route.recipient_service_id)
            .bind(&payload)
            .bind(sha256.to_vec())
            .execute(&mut *tx)
            .await?;
            NativeInfrastructureDeliveryWire { payload, sha256 }
        };
        tx.commit().await?;
        Ok(wire)
    }

    /// Atomically persist the exact native receipt and complete the matching delivery fence.
    #[cfg(feature = "native-grpc-adapter")]
    pub async fn mark_native_delivered(
        &self,
        claim: &InfrastructureOwnerOutboxDeliveryClaim,
        route: &NativeInfrastructureDeliveryRoute,
        receipt: &NativeAppendReceipt,
    ) -> Result<(), InfrastructureOwnerOutboxDeliveryError> {
        validate_claim(claim)?;
        validate_native_route(route)?;
        if receipt.profile != route.profile
            || receipt.log_id.is_empty()
            || receipt.log_id.len() > 128
            || !receipt.log_id.as_bytes()[0].is_ascii_alphanumeric()
            || !receipt.log_id.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
            || receipt.log_generation.is_nil()
            || receipt.sequence <= 0
        {
            return Err(InfrastructureOwnerOutboxDeliveryError::NativeReceiptInvalid);
        }

        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, claim.tenant_id).await?;
        let pinned_digest: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT wire_sha256 FROM infrastructure.capacity_outbox_transport_pin \
             WHERE tenant_id = $1 AND event_id = $2 AND transport_provider = 'native-grpc' \
               AND profile_id = $3 AND profile_revision = $4 AND provider_id = $5 \
               AND provider_version = $6 AND profile_digest = $7 AND endpoint_identity = $8 \
               AND sender_service_id = $9 AND recipient_service_id = $10 \
               AND message_kind = 'result'",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .bind(route.profile_id)
        .bind(route.profile_revision)
        .bind(&route.profile.provider_id)
        .bind(&route.profile.provider_version)
        .bind(route.profile.profile_digest.to_vec())
        .bind(&route.profile.endpoint_identity)
        .bind(route.sender_service_id)
        .bind(route.recipient_service_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(pinned_digest) = pinned_digest else {
            return Err(InfrastructureOwnerOutboxDeliveryError::RouteConflict);
        };
        let pinned_digest: [u8; 32] = pinned_digest
            .as_slice()
            .try_into()
            .map_err(|_| InfrastructureOwnerOutboxDeliveryError::NativeReceiptInvalid)?;
        if receipt.wire_sha256 != pinned_digest {
            return Err(InfrastructureOwnerOutboxDeliveryError::NativeReceiptInvalid);
        }

        sqlx::query(
            "INSERT INTO infrastructure.capacity_native_append_receipt \
             (tenant_id, event_id, profile_id, profile_revision, provider_id, provider_version, \
              profile_digest, endpoint_identity, sender_service_id, recipient_service_id, \
              message_kind, message_id, log_id, log_generation, sequence, exact_replay, \
              wire_sha256, lease_owner_id, attempt_count) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'result', $2, $11, $12, \
                     $13, $14, $15, $16, $17)",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .bind(route.profile_id)
        .bind(route.profile_revision)
        .bind(&route.profile.provider_id)
        .bind(&route.profile.provider_version)
        .bind(route.profile.profile_digest.to_vec())
        .bind(&route.profile.endpoint_identity)
        .bind(route.sender_service_id)
        .bind(route.recipient_service_id)
        .bind(&receipt.log_id)
        .bind(receipt.log_generation)
        .bind(receipt.sequence)
        .bind(receipt.exact_replay)
        .bind(receipt.wire_sha256.to_vec())
        .bind(claim.lease_owner_id)
        .bind(claim.attempt_count)
        .execute(&mut *tx)
        .await?;

        let delivered = sqlx::query(
            "UPDATE infrastructure.capacity_outbox_delivery \
             SET delivery_state = 'delivered', lease_owner_id = NULL, lease_expires_at = NULL, \
                 terminal_at = clock_timestamp(), \
                 retention_expires_at = clock_timestamp() + interval '24 hours', \
                 updated_at = clock_timestamp() \
             WHERE tenant_id = $1 AND event_id = $2 AND delivery_state = 'leased' \
               AND lease_owner_id = $3 AND attempt_count = $4 \
               AND lease_expires_at > clock_timestamp() \
               AND EXISTS (SELECT 1 FROM infrastructure.capacity_outbox_transport_pin AS route \
                            WHERE route.tenant_id = $1 AND route.event_id = $2 \
                              AND route.transport_provider = 'native-grpc' \
                              AND route.wire_sha256 = $5)",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .bind(claim.lease_owner_id)
        .bind(claim.attempt_count)
        .bind(pinned_digest.to_vec())
        .execute(&mut *tx)
        .await?;
        if delivered.rows_affected() != 1 {
            return Err(InfrastructureOwnerOutboxDeliveryError::LeaseLost);
        }
        tx.commit().await?;
        Ok(())
    }

    /// Mark one claimed event delivered if the current unexpired lease fence still matches.
    pub async fn mark_delivered(
        &self,
        claim: &InfrastructureOwnerOutboxDeliveryClaim,
    ) -> Result<(), InfrastructureOwnerOutboxDeliveryError> {
        validate_claim(claim)?;
        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, claim.tenant_id).await?;
        let result = sqlx::query(
            "UPDATE infrastructure.capacity_outbox_delivery \
             SET delivery_state = 'delivered', lease_owner_id = NULL, lease_expires_at = NULL, \
                 terminal_at = clock_timestamp(), \
                 retention_expires_at = clock_timestamp() + interval '24 hours', \
                 updated_at = clock_timestamp() \
             WHERE tenant_id = $1 AND event_id = $2 AND delivery_state = 'leased' \
               AND lease_owner_id = $3 AND attempt_count = $4 \
               AND lease_expires_at > clock_timestamp() \
               AND EXISTS (SELECT 1 FROM infrastructure.capacity_outbox_transport_pin AS route \
                            WHERE route.tenant_id = $1 AND route.event_id = $2 \
                              AND route.transport_provider = 'nats-jetstream')",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .bind(claim.lease_owner_id)
        .bind(claim.attempt_count)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() != 1 {
            return Err(InfrastructureOwnerOutboxDeliveryError::LeaseLost);
        }
        tx.commit().await?;
        Ok(())
    }

    /// Schedule a bounded retry or mark a permanently failed/exhausted event dead.
    ///
    /// Raw transport errors are deliberately not persisted here. A caller can log a bounded,
    /// redacted diagnostic separately; delivery facts remain small and contain no secret text.
    pub async fn record_failure(
        &self,
        claim: &InfrastructureOwnerOutboxDeliveryClaim,
        failure: OutboxDeliveryFailureKind,
    ) -> Result<(), InfrastructureOwnerOutboxDeliveryError> {
        validate_claim(claim)?;
        let terminal = failure == OutboxDeliveryFailureKind::Permanent
            || claim.attempt_count >= MAX_DELIVERY_ATTEMPTS;
        let backoff_seconds = retry_backoff_seconds(claim.attempt_count);
        let mut tx = self.pool.begin().await?;
        set_owner_scope(&mut tx, claim.tenant_id).await?;
        let result = sqlx::query(
            "UPDATE infrastructure.capacity_outbox_delivery \
             SET delivery_state = CASE WHEN $5 THEN 'dead' ELSE 'retry' END, \
                 next_attempt_at = CASE WHEN $5 THEN next_attempt_at \
                     ELSE clock_timestamp() + make_interval(secs => $6::double precision) END, \
                 lease_owner_id = NULL, lease_expires_at = NULL, \
                 terminal_at = CASE WHEN $5 THEN clock_timestamp() ELSE NULL END, \
                 retention_expires_at = CASE WHEN $5 \
                     THEN clock_timestamp() + interval '24 hours' ELSE NULL END, \
                 updated_at = clock_timestamp() \
             WHERE tenant_id = $1 AND event_id = $2 AND delivery_state = 'leased' \
               AND lease_owner_id = $3 AND attempt_count = $4 \
               AND lease_expires_at > clock_timestamp()",
        )
        .bind(claim.tenant_id)
        .bind(claim.event_id)
        .bind(claim.lease_owner_id)
        .bind(claim.attempt_count)
        .bind(terminal)
        .bind(backoff_seconds)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() != 1 {
            return Err(InfrastructureOwnerOutboxDeliveryError::LeaseLost);
        }
        tx.commit().await?;
        Ok(())
    }
}

fn retry_backoff_seconds(attempt_count: i32) -> i64 {
    let exponent = attempt_count.saturating_sub(1).clamp(0, 12) as u32;
    1_i64
        .checked_shl(exponent)
        .unwrap_or(MAX_BACKOFF_SECONDS)
        .min(MAX_BACKOFF_SECONDS)
}

fn validate_claim(
    claim: &InfrastructureOwnerOutboxDeliveryClaim,
) -> Result<(), InfrastructureOwnerOutboxDeliveryError> {
    if claim.tenant_id.is_nil()
        || claim.event_id.is_nil()
        || claim.lease_owner_id.is_nil()
        || claim.attempt_count <= 0
    {
        return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
    }
    Ok(())
}

async fn set_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(OWNER_ROLE).execute(&mut **tx).await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(feature = "native-grpc-adapter")]
async fn diagnose_native_route_pins_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
    route: &NativeInfrastructureDeliveryRoute,
    lease_owner_id: Uuid,
) -> Result<NativeRoutePinQuarantinePage, InfrastructureOwnerOutboxDeliveryError> {
    let tenant_id = route.tenant_id();
    let observed_due_native_pins = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT d.event_id
             FROM infrastructure.capacity_outbox_delivery AS d
             JOIN infrastructure.capacity_outbox_transport_pin AS pinned
               ON pinned.tenant_id = d.tenant_id AND pinned.event_id = d.event_id
            WHERE d.tenant_id = $1
              AND pinned.transport_provider = 'native-grpc'
              AND ((d.delivery_state IN ('pending','retry')
                    AND d.next_attempt_at <= clock_timestamp())
                OR (d.delivery_state = 'leased'
                    AND d.lease_expires_at <= clock_timestamp()))
            ORDER BY CASE WHEN d.delivery_state = 'leased'
                          THEN d.lease_expires_at ELSE d.next_attempt_at END,
                     d.event_id
            LIMIT $2"#,
    )
    .bind(tenant_id)
    .bind(MAX_NATIVE_PINNED_DUE_SAMPLE + 1)
    .fetch_all(&mut **tx)
    .await?;

    let quarantined_event_ids = sqlx::query_scalar::<_, Uuid>(
        r#"WITH stale AS (
               SELECT d.tenant_id, d.event_id
                 FROM infrastructure.capacity_outbox_delivery AS d
                 JOIN infrastructure.capacity_outbox_transport_pin AS pinned
                   ON pinned.tenant_id = d.tenant_id AND pinned.event_id = d.event_id
                WHERE d.tenant_id = $1
                  AND pinned.transport_provider = 'native-grpc'
                  AND (pinned.profile_id IS DISTINCT FROM $3
                       OR pinned.profile_revision IS DISTINCT FROM $4
                       OR pinned.provider_id IS DISTINCT FROM $5
                       OR pinned.provider_version IS DISTINCT FROM $6
                       OR pinned.profile_digest IS DISTINCT FROM $7
                       OR pinned.endpoint_identity IS DISTINCT FROM $8
                       OR pinned.sender_service_id IS DISTINCT FROM $9
                       OR pinned.recipient_service_id IS DISTINCT FROM $10)
                  AND (d.delivery_state IN ('pending','retry','dead')
                       OR (d.delivery_state = 'leased'
                           AND d.lease_expires_at <= clock_timestamp()))
                  AND NOT EXISTS (
                       SELECT 1
                         FROM infrastructure.capacity_native_route_pin_quarantine AS q
                        WHERE q.tenant_id = d.tenant_id AND q.event_id = d.event_id)
                  AND NOT EXISTS (
                       SELECT 1
                         FROM infrastructure.capacity_native_append_receipt AS receipt
                        WHERE receipt.tenant_id = d.tenant_id AND receipt.event_id = d.event_id)
                ORDER BY d.updated_at, d.event_id
                LIMIT $2
                FOR UPDATE OF d SKIP LOCKED
           )
           INSERT INTO infrastructure.capacity_native_route_pin_quarantine (
               tenant_id, event_id, observed_profile_id, observed_profile_revision,
               observed_provider_id, observed_provider_version, observed_profile_digest,
               observed_endpoint_identity, observed_sender_service_id,
               observed_recipient_service_id, mismatch_fields)
           SELECT pinned.tenant_id, pinned.event_id, $3, $4, $5, $6, $7, $8, $9, $10,
                  ARRAY_REMOVE(ARRAY[
                      CASE WHEN pinned.profile_id IS DISTINCT FROM $3 THEN 'profile_id' END,
                      CASE WHEN pinned.profile_revision IS DISTINCT FROM $4 THEN 'profile_revision' END,
                      CASE WHEN pinned.provider_id IS DISTINCT FROM $5 THEN 'provider_id' END,
                      CASE WHEN pinned.provider_version IS DISTINCT FROM $6 THEN 'provider_version' END,
                      CASE WHEN pinned.profile_digest IS DISTINCT FROM $7 THEN 'profile_digest' END,
                      CASE WHEN pinned.endpoint_identity IS DISTINCT FROM $8 THEN 'endpoint_identity' END,
                      CASE WHEN pinned.sender_service_id IS DISTINCT FROM $9 THEN 'sender_service_id' END,
                      CASE WHEN pinned.recipient_service_id IS DISTINCT FROM $10 THEN 'recipient_service_id' END
                  ]::TEXT[], NULL::TEXT)
             FROM stale
             JOIN infrastructure.capacity_outbox_transport_pin AS pinned
               ON pinned.tenant_id = stale.tenant_id AND pinned.event_id = stale.event_id
           ON CONFLICT (tenant_id, event_id) DO NOTHING
           RETURNING event_id"#,
    )
    .bind(tenant_id)
    .bind(MAX_NATIVE_ROUTE_PIN_QUARANTINE_BATCH)
    .bind(route.profile_id)
    .bind(route.profile_revision)
    .bind(&route.profile.provider_id)
    .bind(&route.profile.provider_version)
    .bind(route.profile.profile_digest.to_vec())
    .bind(&route.profile.endpoint_identity)
    .bind(route.sender_service_id)
    .bind(route.recipient_service_id)
    .fetch_all(&mut **tx)
    .await?;

    if !quarantined_event_ids.is_empty() {
        sqlx::query(
            "UPDATE infrastructure.capacity_outbox_delivery \
             SET delivery_state = 'leased', lease_owner_id = $3, \
                 lease_expires_at = clock_timestamp() + interval '5 seconds', \
                 updated_at = clock_timestamp() \
             WHERE tenant_id = $1 AND event_id = ANY($2) \
               AND delivery_state IN ('pending','retry')",
        )
        .bind(tenant_id)
        .bind(&quarantined_event_ids)
        .bind(lease_owner_id)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "UPDATE infrastructure.capacity_outbox_delivery \
             SET delivery_state = 'dead', lease_owner_id = NULL, lease_expires_at = NULL, \
                 terminal_at = clock_timestamp(), \
                 retention_expires_at = clock_timestamp() + interval '24 hours', \
                 updated_at = clock_timestamp() \
             WHERE tenant_id = $1 AND event_id = ANY($2) \
               AND delivery_state = 'leased' \
               AND (lease_owner_id = $3 OR lease_expires_at <= clock_timestamp())",
        )
        .bind(tenant_id)
        .bind(&quarantined_event_ids)
        .bind(lease_owner_id)
        .execute(&mut **tx)
        .await?;
    }

    let mut page = load_native_route_pin_quarantine_page(
        tx,
        tenant_id,
        None,
        MAX_NATIVE_ROUTE_PIN_DIAGNOSTIC_PAGE_SIZE,
    )
    .await?;
    page.newly_quarantined = quarantined_event_ids.len();
    page.due_native_pinned = observed_due_native_pins
        .len()
        .min(MAX_NATIVE_PINNED_DUE_SAMPLE as usize);
    page.due_native_pinned_truncated =
        observed_due_native_pins.len() > MAX_NATIVE_PINNED_DUE_SAMPLE as usize;
    Ok(page)
}

#[cfg(feature = "native-grpc-adapter")]
async fn load_native_route_pin_quarantine_page(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    cursor: Option<NativeRoutePinQuarantineCursor>,
    limit: i64,
) -> Result<NativeRoutePinQuarantinePage, InfrastructureOwnerOutboxDeliveryError> {
    let total_blocked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_route_pin_quarantine \
         WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&mut **tx)
    .await?;

    let rows: Vec<(Uuid, Vec<String>, Uuid, i64, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT event_id, mismatch_fields, observed_profile_id, \
                    observed_profile_revision, detected_at \
             FROM infrastructure.capacity_native_route_pin_quarantine \
             WHERE tenant_id = $1 \
               AND ($2::timestamptz IS NULL \
                    OR (detected_at, event_id) > ($2::timestamptz, $3::uuid)) \
             ORDER BY detected_at, event_id LIMIT $4",
    )
    .bind(tenant_id)
    .bind(cursor.map(|value| value.detected_at))
    .bind(cursor.map(|value| value.event_id))
    .bind(limit + 1)
    .fetch_all(&mut **tx)
    .await?;

    let has_more = rows.len() as i64 > limit;
    let mut entries: Vec<NativeRoutePinQuarantineDiagnostic> = rows
        .into_iter()
        .take(limit as usize)
        .map(
            |(
                event_id,
                mismatch_fields,
                observed_profile_id,
                observed_profile_revision,
                detected_at,
            )| {
                NativeRoutePinQuarantineDiagnostic {
                    event_id,
                    mismatch_fields,
                    observed_profile_id,
                    observed_profile_revision,
                    detected_at,
                }
            },
        )
        .collect();
    let next_cursor = if has_more {
        entries.last().map(|last| NativeRoutePinQuarantineCursor {
            detected_at: last.detected_at,
            event_id: last.event_id,
        })
    } else {
        None
    };

    Ok(NativeRoutePinQuarantinePage {
        total_blocked,
        entries: std::mem::take(&mut entries),
        next_cursor,
        newly_quarantined: 0,
        due_native_pinned: 0,
        due_native_pinned_truncated: false,
    })
}

#[cfg(feature = "native-grpc-adapter")]
fn validate_native_route(
    route: &NativeInfrastructureDeliveryRoute,
) -> Result<(), InfrastructureOwnerOutboxDeliveryError> {
    let profile_valid = AdmissionTransportProfile::new(
        route.profile.provider_id.clone(),
        route.profile.provider_version.clone(),
        route.profile.profile_digest,
        route.profile.endpoint_identity.clone(),
    )
    .is_ok();
    if route.deployment_id.is_nil()
        || route.profile_id.is_nil()
        || route.profile_revision <= 0
        || route.tenant_id.is_nil()
        || !profile_valid
        || route.sender_service_id.is_nil()
        || route.recipient_service_id.is_nil()
        || route.sender_service_id == route.recipient_service_id
    {
        return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
    }
    Ok(())
}
