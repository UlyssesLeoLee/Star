//! PostgreSQL delivery state machine for immutable Infrastructure owner Outbox events.
//!
//! @cypher schema=1 source_sha256=cb3fbded017b117e9337640109953108a6dca36c26a7091e39d207ecdf8d0de2
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::infrastructure_owner_outbox_delivery",kind:"module"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"struct"})
//! MERGE (claim:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryClaim",kind:"struct"})
//! MERGE (native_route:Type {id:"infrastructure::NativeInfrastructureDeliveryRoute",kind:"struct"})
//! MERGE (native_wire:Type {id:"infrastructure::NativeInfrastructureDeliveryWire",kind:"struct"})
//! MERGE (failure:Type {id:"infrastructure::OutboxDeliveryFailureKind",kind:"enum"})
//! MERGE (error:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryError",kind:"enum"})
//! MERGE (claim_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_due",kind:"method"})
//! MERGE (claim_native_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_native_due",kind:"method"})
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
//! MERGE (pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(store)
//! MERGE (module)-[:DEFINES]->(claim)
//! MERGE (module)-[:DEFINES]->(native_route)
//! MERGE (module)-[:DEFINES]->(native_wire)
//! MERGE (module)-[:DEFINES]->(failure)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (store)-[:DEFINES]->(claim_due)
//! MERGE (store)-[:DEFINES]->(claim_native_due)
//! MERGE (store)-[:DEFINES]->(get_or_pin_wire)
//! MERGE (store)-[:DEFINES]->(mark_delivered)
//! MERGE (store)-[:DEFINES]->(mark_native_delivered)
//! MERGE (store)-[:DEFINES]->(record_failure)
//! MERGE (store)-[:USES_TYPE]->(pool)
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
//! MERGE (claim_native_due)-[:READS]->(delivery)
//! MERGE (claim_native_due)-[:WRITES]->(delivery)
//! MERGE (claim_native_due)-[:READS]->(outbox)
//! MERGE (claim_native_due)-[:READS]->(route_pin)
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
    admission_delivery::AdmissionTransportProfile, native_append::NativeAppendReceipt,
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
    /// Immutable profile identity resolved by trusted deployment configuration.
    pub profile_id: Uuid,
    /// Immutable profile revision resolved by trusted deployment configuration.
    pub profile_revision: i64,
    /// Non-secret provider/profile snapshot used by the native client.
    pub profile: AdmissionTransportProfile,
    /// Authenticated Infrastructure sender identity.
    pub sender_service_id: Uuid,
    /// Authorized Run-owner result recipient.
    pub recipient_service_id: Uuid,
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

    /// Lease result events that are unpinned or already pinned to this exact native route.
    ///
    /// Route bytes are persisted separately by `get_or_pin_native_wire` under this lease fence
    /// before any append RPC starts. NATS claims exclude every native-pinned event.
    #[cfg(feature = "native-grpc-adapter")]
    pub async fn claim_native_due(
        &self,
        tenant_id: Uuid,
        lease_owner_id: Uuid,
        max_items: i64,
        lease_seconds: i64,
        route: &NativeInfrastructureDeliveryRoute,
    ) -> Result<Vec<InfrastructureOwnerOutboxDeliveryClaim>, InfrastructureOwnerOutboxDeliveryError>
    {
        validate_native_route(route)?;
        if tenant_id.is_nil()
            || lease_owner_id.is_nil()
            || !(1..=MAX_NATIVE_BATCH_SIZE).contains(&max_items)
            || !(MIN_NATIVE_LEASE_SECONDS..=MAX_NATIVE_LEASE_SECONDS).contains(&lease_seconds)
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
    if route.profile_id.is_nil()
        || route.profile_revision <= 0
        || !profile_valid
        || route.sender_service_id.is_nil()
        || route.recipient_service_id.is_nil()
        || route.sender_service_id == route.recipient_service_id
    {
        return Err(InfrastructureOwnerOutboxDeliveryError::InvalidParameters);
    }
    Ok(())
}
