// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=739e94f92dc4c023a1d33b3991ceaa005063dd979ca0b9c84f03215eb689deb5
MERGE (self:File {path:"crates/run-owner-runtime/src/task_outbox_delivery.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::task_outbox_delivery",kind:"module"})
MERGE (store:Type {id:"run_owner_runtime::TaskRunOutboxDeliveryStore",kind:"struct"})
MERGE (claim:Type {id:"run_owner_runtime::TaskRunOutboxDeliveryClaim",kind:"struct"})
MERGE (route:Type {id:"run_owner_runtime::TaskRunOutboxDeliveryRoute",kind:"struct"})
MERGE (prepared:Type {id:"run_owner_runtime::PreparedTaskRunOutboxDelivery",kind:"struct"})
MERGE (error:Type {id:"run_owner_runtime::TaskRunOutboxDeliveryError",kind:"enum"})
MERGE (invalid_proof:Type {id:"run_owner_runtime::TaskRunOutboxInvalidEnvelopeProof",kind:"struct"})
MERGE (claim_due:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryStore.claim_due",kind:"method"})
MERGE (pin:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryStore.pin_route_for_first_send",kind:"method"})
MERGE (retry:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryStore.settle_retry",kind:"method"})
MERGE (dead:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryStore.settle_dead",kind:"method"})
MERGE (build_wire:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::build_exact_wire",kind:"function"})
MERGE (route_factory:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryRoute.from_resolved_endpoint",kind:"method"})
MERGE (validate_claim:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::validate_claim",kind:"function"})
MERGE (validate_event:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::validate_event",kind:"function"})
MERGE (invalid_proof_fn:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryClaim.invalid_envelope_proof",kind:"method"})
MERGE (claim_debug:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::TaskRunOutboxDeliveryClaim.fmt",kind:"method"})
MERGE (prepared_debug:Symbol {id:"crates/run-owner-runtime/src/task_outbox_delivery.rs::PreparedTaskRunOutboxDelivery.fmt",kind:"method"})
MERGE (delivery:Table {id:"multica.task_run_outbox_delivery"})
MERGE (route_pin:Table {id:"multica.task_run_outbox_delivery_route_pin"})
MERGE (source:Table {id:"multica.task_run_outbox"})
MERGE (claim_fn:Symbol {id:"db/migrations/2026-10-16-task-run-outbox-delivery.sql::multica.claim_task_run_outbox_delivery",kind:"database_function"})
MERGE (pin_fn:Symbol {id:"db/migrations/2026-10-16-task-run-outbox-route-pin.sql::multica.get_or_pin_task_run_outbox_delivery_route",kind:"database_function"})
MERGE (settle_fn:Symbol {id:"db/migrations/2026-10-16-task-run-outbox-delivery.sql::multica.settle_task_run_outbox_delivery",kind:"database_function"})
MERGE (resolved_endpoint:Type {id:"platform_messaging_runtime::ResolvedTransportEndpoint",kind:"external_type"})
MERGE (transport_profile:Type {id:"platform_messaging_runtime::TransportProfileRevision",kind:"external_type"})
MERGE (transport_context:Type {id:"platform_messaging_runtime::AuthenticatedTransportContext",kind:"external_type"})
MERGE (transport_kind:Type {id:"platform_messaging_runtime::TransportMessageKind",kind:"external_type"})
MERGE (transaction:Type {id:"sqlx::Transaction<Postgres>",kind:"external_type"})
MERGE (debug_trait:Type {id:"std::fmt::Debug",kind:"external_trait"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(store)
MERGE (module)-[:DEFINES]->(claim)
MERGE (module)-[:DEFINES]->(route)
MERGE (module)-[:DEFINES]->(prepared)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(invalid_proof)
MERGE (store)-[:DEFINES]->(claim_due)
MERGE (store)-[:DEFINES]->(pin)
MERGE (store)-[:DEFINES]->(retry)
MERGE (store)-[:DEFINES]->(dead)
MERGE (route)-[:DEFINES]->(route_factory)
MERGE (claim_due)-[:CALLS]->(claim_fn)
MERGE (claim_due)-[:READS]->(delivery)
MERGE (claim_due)-[:READS]->(source)
MERGE (claim_due)-[:USES_TYPE]->(transaction)
MERGE (pin)-[:CALLS]->(pin_fn)
MERGE (pin)-[:CALLS]->(build_wire)
MERGE (pin)-[:WRITES]->(route_pin)
MERGE (pin)-[:USES_TYPE]->(transaction)
MERGE (retry)-[:CALLS]->(settle_fn)
MERGE (dead)-[:CALLS]->(settle_fn)
MERGE (route_factory)-[:USES_TYPE]->(resolved_endpoint)
MERGE (route_factory)-[:USES_TYPE]->(transport_profile)
MERGE (route_factory)-[:USES_TYPE]->(transport_context)
MERGE (route_factory)-[:USES_TYPE]->(transport_kind)
MERGE (build_wire)-[:CALLS]->(validate_event)
MERGE (pin)-[:CALLS]->(validate_claim)
MERGE (claim)-[:DEFINES]->(invalid_proof_fn)
MERGE (invalid_proof_fn)-[:CALLS]->(validate_claim)
MERGE (claim)-[:IMPLEMENTS]->(debug_trait)
MERGE (prepared)-[:IMPLEMENTS]->(debug_trait)
MERGE (claim)-[:DEFINES]->(claim_debug)
MERGE (prepared)-[:DEFINES]->(prepared_debug)
@endcypher
*/
//! Bounded PostgreSQL access for the Task Run Outbox delivery substrate.
//!
//! This module prepares an exact, route-pinned wire but does not send it or accept a receiver
//! receipt. A transport append is not a Task-owner Inbox commit, and accepted settlement remains
//! unavailable until the authenticated receiver and durable receipt path exists.

use std::fmt;

use chrono::{DateTime, Utc};
use platform_messaging_runtime::{
    transport_endpoint::ResolvedTransportEndpoint,
    transport_profile::{
        AuthenticatedTransportContext, TransportDirection, TransportMessageKind,
        TransportProfileRevision,
    },
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

const MAX_BATCH_SIZE: i32 = 8;
const MAX_LEASE_SECONDS: i32 = 300;
const MAX_DELIVERY_ATTEMPTS: i32 = 12;
const MAX_TASK_PAYLOAD_BYTES: usize = 8 * 1024;
const MAX_TASK_WIRE_BYTES: usize = 16 * 1024;

/// Bounded immutable Task source event plus its active delivery lease fence.
#[derive(Clone, PartialEq, Eq)]
pub struct TaskRunOutboxDeliveryClaim {
    /// Tenant owning this delivery.
    tenant_id: Uuid,
    /// Project scope in the immutable source event.
    project_id: Uuid,
    /// Run scope; Tasks may be created without a remote host Branch.
    engineering_run_id: Uuid,
    /// Stable immutable Task event identity.
    event_id: Uuid,
    /// Task Card aggregate identity.
    work_item_id: Uuid,
    /// Independent Task metadata or lifecycle version stream.
    aggregate_type: String,
    /// Version within the aggregate stream.
    aggregate_version: i32,
    /// Allowlisted source event type.
    event_type: String,
    /// Versioned Task event envelope schema.
    schema_version: i32,
    /// Source actor identity.
    actor_id: Uuid,
    /// Stable correlation identity.
    correlation_id: Uuid,
    /// Bounded source JSON body.
    payload: Value,
    /// Source event time.
    occurred_at: DateTime<Utc>,
    /// Worker currently holding this claim.
    worker_id: Uuid,
    /// Monotonic delivery attempt fence.
    attempt_count: i32,
    /// Database-clock lease expiry.
    lease_expires_at: DateTime<Utc>,
}

impl fmt::Debug for TaskRunOutboxDeliveryClaim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TaskRunOutboxDeliveryClaim")
            .field("tenant_id", &self.tenant_id)
            .field("project_id", &self.project_id)
            .field("engineering_run_id", &self.engineering_run_id)
            .field("event_id", &self.event_id)
            .field("work_item_id", &self.work_item_id)
            .field("aggregate_type", &self.aggregate_type)
            .field("aggregate_version", &self.aggregate_version)
            .field("event_type", &self.event_type)
            .field("schema_version", &self.schema_version)
            .field("actor_id", &self.actor_id)
            .field("correlation_id", &self.correlation_id)
            .field("payload", &"[redacted]")
            .field("occurred_at", &self.occurred_at)
            .field("worker_id", &self.worker_id)
            .field("attempt_count", &self.attempt_count)
            .field("lease_expires_at", &self.lease_expires_at)
            .finish()
    }
}

impl TaskRunOutboxDeliveryClaim {
    /// Return the tenant identity bound to this database claim.
    pub fn tenant_id(&self) -> Uuid { self.tenant_id }
    /// Return the Project identity bound to the source event.
    pub fn project_id(&self) -> Uuid { self.project_id }
    /// Return the Engineering Run identity bound to the source event.
    pub fn engineering_run_id(&self) -> Uuid { self.engineering_run_id }
    /// Return the stable immutable source event identity.
    pub fn event_id(&self) -> Uuid { self.event_id }
    /// Return the Task Card aggregate identity.
    pub fn work_item_id(&self) -> Uuid { self.work_item_id }
    /// Return the aggregate stream name.
    pub fn aggregate_type(&self) -> &str { &self.aggregate_type }
    /// Return the version within this aggregate stream.
    pub fn aggregate_version(&self) -> i32 { self.aggregate_version }
    /// Return the typed source event name.
    pub fn event_type(&self) -> &str { &self.event_type }
    /// Return the source event schema version.
    pub fn schema_version(&self) -> i32 { self.schema_version }
    /// Return the source actor identity.
    pub fn actor_id(&self) -> Uuid { self.actor_id }
    /// Return the stable correlation identity.
    pub fn correlation_id(&self) -> Uuid { self.correlation_id }
    /// Return a borrowed view of the bounded immutable source payload.
    pub fn payload(&self) -> &Value { &self.payload }
    /// Return the source event time.
    pub fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    /// Return the current lease worker identity.
    pub fn worker_id(&self) -> Uuid { self.worker_id }
    /// Return the current monotonic lease attempt.
    pub fn attempt_count(&self) -> i32 { self.attempt_count }
    /// Return the database-clock lease expiry.
    pub fn lease_expires_at(&self) -> DateTime<Utc> { self.lease_expires_at }

    /// Create a fenced dead-letter proof only when this immutable source event fails validation.
    pub fn invalid_envelope_proof(
        &self,
    ) -> Result<TaskRunOutboxInvalidEnvelopeProof, TaskRunOutboxDeliveryError> {
        validate_claim_fence(self)?;
        match validate_claim(self) {
            Err(TaskRunOutboxDeliveryError::InvalidContract) => {
                Ok(TaskRunOutboxInvalidEnvelopeProof {
                    tenant_id: self.tenant_id,
                    event_id: self.event_id,
                    worker_id: self.worker_id,
                    attempt_count: self.attempt_count,
                })
            }
            Ok(()) => Err(TaskRunOutboxDeliveryError::InvalidContract),
            Err(error) => Err(error),
        }
    }
}

/// A current transport route resolved from a trusted endpoint and matching profile snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunOutboxDeliveryRoute {
    tenant_id: Uuid,
    sender_service_id: Uuid,
    receiver_service_id: Uuid,
    provider_id: String,
    provider_version: String,
    profile_id: Uuid,
    profile_revision: i64,
    profile_digest: [u8; 32],
    endpoint_identity: String,
    destination_id: String,
    destination_generation: Uuid,
}

impl TaskRunOutboxDeliveryRoute {
    /// Bind a non-secret transport profile to an endpoint produced by the trusted resolver.
    ///
    /// Destination identity and generation come from deployment-owned routing configuration;
    /// they are never accepted from a Task event or caller request.
    pub fn from_resolved_endpoint(
        endpoint: &ResolvedTransportEndpoint,
        profile: &TransportProfileRevision,
        context: &AuthenticatedTransportContext,
        destination_id: String,
        destination_generation: Uuid,
    ) -> Result<Self, TaskRunOutboxDeliveryError> {
        profile
            .authorize(context, TransportMessageKind::Event)
            .map_err(|_| TaskRunOutboxDeliveryError::InvalidContract)?;
        let sender_service_id = context.sender_service_id();
        let receiver_service_id = context.recipient_service_id();
        if context.direction() != TransportDirection::Publish
            || endpoint.deployment_id().is_nil()
            || endpoint.tenant_id().is_nil()
            || endpoint.profile_id().is_nil()
            || endpoint.profile_revision() <= 0
            || endpoint.deployment_id() != profile.deployment_id()
            || endpoint.deployment_id() != context.deployment_id()
            || endpoint.tenant_id() != profile.tenant_id()
            || endpoint.tenant_id() != context.tenant_id()
            || endpoint.profile_id() != profile.profile_id()
            || endpoint.profile_revision() != profile.revision()
            || endpoint.profile_digest() != profile.digest()
            || endpoint.endpoint_identity() != profile.definition().endpoint_identity()
            || endpoint.endpoint_identity() != context.endpoint_identity()
            || sender_service_id.is_nil()
            || receiver_service_id.is_nil()
            || sender_service_id == receiver_service_id
            || destination_generation.is_nil()
            || !valid_provider_id(profile.definition().provider_id())
            || !valid_transport_version(profile.definition().provider_version())
            || !valid_route_label(&destination_id)
        {
            return Err(TaskRunOutboxDeliveryError::InvalidContract);
        }
        Ok(Self {
            tenant_id: endpoint.tenant_id(),
            sender_service_id,
            receiver_service_id,
            provider_id: profile.definition().provider_id().to_owned(),
            provider_version: profile.definition().provider_version().to_owned(),
            profile_id: endpoint.profile_id(),
            profile_revision: endpoint.profile_revision(),
            profile_digest: endpoint.profile_digest(),
            endpoint_identity: endpoint.endpoint_identity().to_owned(),
            destination_id,
            destination_generation,
        })
    }

    /// Return the authenticated sender service identity.
    pub fn sender_service_id(&self) -> Uuid { self.sender_service_id }
    /// Return the pinned receiver service identity.
    pub fn receiver_service_id(&self) -> Uuid { self.receiver_service_id }
    /// Return the pinned provider identifier.
    pub fn provider_id(&self) -> &str { &self.provider_id }
    /// Return the pinned provider protocol version.
    pub fn provider_version(&self) -> &str { &self.provider_version }
    /// Return the pinned Transport Profile identity.
    pub fn profile_id(&self) -> Uuid { self.profile_id }
    /// Return the pinned Transport Profile revision.
    pub fn profile_revision(&self) -> i64 { self.profile_revision }
    /// Return the pinned Transport Profile content digest.
    pub fn profile_digest(&self) -> [u8; 32] { self.profile_digest }
    /// Return the deployment-trusted endpoint label.
    pub fn endpoint_identity(&self) -> &str { &self.endpoint_identity }
    /// Return the configured opaque destination label.
    pub fn destination_id(&self) -> &str { &self.destination_id }
    /// Return the generation of the configured destination.
    pub fn destination_generation(&self) -> Uuid { self.destination_generation }
}

/// A claim whose exact bytes and immutable first-send route pin were verified by PostgreSQL.
#[derive(Clone, PartialEq, Eq)]
pub struct PreparedTaskRunOutboxDelivery {
    claim: TaskRunOutboxDeliveryClaim,
    route: TaskRunOutboxDeliveryRoute,
    wire: Vec<u8>,
    wire_sha256: [u8; 32],
}

impl fmt::Debug for PreparedTaskRunOutboxDelivery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedTaskRunOutboxDelivery")
            .field("claim", &self.claim)
            .field("route", &self.route)
            .field("wire_len", &self.wire.len())
            .field("wire_sha256", &self.wire_sha256)
            .finish()
    }
}

impl PreparedTaskRunOutboxDelivery {
    /// Current event and lease fence used to prepare these bytes.
    pub fn claim(&self) -> &TaskRunOutboxDeliveryClaim {
        &self.claim
    }

    /// Current route, already checked against the immutable first-send pin.
    pub fn route(&self) -> &TaskRunOutboxDeliveryRoute {
        &self.route
    }

    /// Exact bounded wire bytes to use on every retry.
    pub fn wire(&self) -> &[u8] {
        &self.wire
    }

    /// SHA-256 digest pinned in the delivery owner's database.
    pub fn wire_sha256(&self) -> [u8; 32] {
        self.wire_sha256
    }
}

/// Fenced proof that an immutable Task source event violates its versioned contract.
///
/// Fields are private, so callers can obtain a proof only through claim validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunOutboxInvalidEnvelopeProof {
    tenant_id: Uuid,
    event_id: Uuid,
    worker_id: Uuid,
    attempt_count: i32,
}

/// Invalid source event, stale route, or PostgreSQL failure; messages never contain payloads.
#[derive(Debug, Error)]
pub enum TaskRunOutboxDeliveryError {
    /// Caller or stored event is outside the versioned, bounded contract.
    #[error("invalid Task Run Outbox delivery contract")]
    InvalidContract,
    /// An immutable first-send route or exact wire differs from its stored pin.
    #[error("Task Run delivery route conflicts with its immutable pin")]
    RouteConflict,
    /// The fenced SQL function rejected a stale or expired lease.
    #[error("Task Run delivery lease fence rejected")]
    LeaseFence,
    /// The owner database operation failed.
    #[error("Task Run delivery database operation failed")]
    Database(#[from] sqlx::Error),
    /// The typed immutable envelope could not be serialized.
    #[error("Task Run delivery serialization failed")]
    Serialization(#[from] serde_json::Error),
}

/// Run-owner database port for bounded Task Outbox claims and immutable route pins.
#[derive(Debug, Clone)]
pub struct TaskRunOutboxDeliveryStore {
    pool: PgPool,
}

impl TaskRunOutboxDeliveryStore {
    /// Bind this store to the deployment-managed Run-owner pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Claim at most eight source events for one tenant under the active attempt fence.
    ///
    /// The SQL claim is committed before envelope validation so a permanently invalid source
    /// event can be recorded as dead under this same returned lease instead of being reclaimed
    /// forever by a rolled-back claim.
    pub async fn claim_due(
        &self,
        tenant_id: Uuid,
        worker_id: Uuid,
        limit: i32,
        lease_seconds: i32,
    ) -> Result<Vec<TaskRunOutboxDeliveryClaim>, TaskRunOutboxDeliveryError> {
        if tenant_id.is_nil()
            || worker_id.is_nil()
            || !(1..=MAX_BATCH_SIZE).contains(&limit)
            || !(30..=MAX_LEASE_SECONDS).contains(&lease_seconds)
        {
            return Err(TaskRunOutboxDeliveryError::InvalidContract);
        }

        let mut tx = self.begin_scoped(tenant_id).await?;
        let rows = sqlx::query("SELECT * FROM multica.claim_task_run_outbox_delivery($1,$2,$3)")
            .bind(worker_id)
            .bind(limit)
            .bind(lease_seconds)
            .fetch_all(&mut *tx)
            .await?;

        let mut claims = Vec::with_capacity(rows.len());
        for row in &rows {
            let claim = TaskRunOutboxDeliveryClaim {
                tenant_id: row.try_get("tenant_id")?,
                project_id: row.try_get("project_id")?,
                engineering_run_id: row.try_get("engineering_run_id")?,
                event_id: row.try_get("event_id")?,
                work_item_id: row.try_get("work_item_id")?,
                aggregate_type: row.try_get("aggregate_type")?,
                aggregate_version: row.try_get("aggregate_version")?,
                event_type: row.try_get("event_type")?,
                schema_version: row.try_get("schema_version")?,
                actor_id: row.try_get("actor_id")?,
                correlation_id: row.try_get("correlation_id")?,
                payload: row.try_get("payload")?,
                occurred_at: row.try_get("occurred_at")?,
                worker_id,
                attempt_count: row.try_get("attempt_count")?,
                lease_expires_at: row.try_get("lease_expires_at")?,
            };
            validate_claim_fence(&claim)?;
            if claim.tenant_id != tenant_id || claim.worker_id != worker_id {
                return Err(TaskRunOutboxDeliveryError::InvalidContract);
            }
            claims.push(claim);
        }
        tx.commit().await?;
        Ok(claims)
    }

    /// Persist or verify the immutable route and exact-wire digest before any send.
    ///
    /// The route-pin SQL function rechecks the current database lease and source schema. This
    /// method does not send bytes and does not produce a business acceptance receipt.
    pub async fn pin_route_for_first_send(
        &self,
        claim: &TaskRunOutboxDeliveryClaim,
        route: &TaskRunOutboxDeliveryRoute,
    ) -> Result<PreparedTaskRunOutboxDelivery, TaskRunOutboxDeliveryError> {
        validate_claim(claim)?;
        if claim.tenant_id != route.tenant_id {
            return Err(TaskRunOutboxDeliveryError::InvalidContract);
        }
        let wire = build_exact_wire(claim)?;
        let wire_sha256: [u8; 32] = Sha256::digest(&wire).into();

        let mut tx = self.begin_scoped(claim.tenant_id).await?;
        let row = sqlx::query(
            r#"SELECT * FROM multica.get_or_pin_task_run_outbox_delivery_route(
                   $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)"#,
        )
        .bind(claim.event_id)
        .bind(claim.worker_id)
        .bind(claim.attempt_count)
        .bind(route.sender_service_id)
        .bind(route.receiver_service_id)
        .bind(&route.provider_id)
        .bind(&route.provider_version)
        .bind(route.profile_id)
        .bind(route.profile_revision)
        .bind(route.profile_digest.as_slice())
        .bind(&route.endpoint_identity)
        .bind(&route.destination_id)
        .bind(route.destination_generation)
        .bind(claim.schema_version)
        .bind(wire_sha256.as_slice())
        .fetch_one(&mut *tx)
        .await
        .map_err(map_lease_error)?;

        let persisted_profile_digest: Vec<u8> = row.try_get("transport_profile_digest")?;
        let persisted_wire_digest: Vec<u8> = row.try_get("wire_sha256")?;
        let persisted_worker: Uuid = row.try_get("pinned_by_worker_id")?;
        let persisted_attempt: i32 = row.try_get("pinned_attempt_count")?;
        let route_matches = row.try_get::<Uuid, _>("tenant_id")? == claim.tenant_id
            && row.try_get::<Uuid, _>("engineering_run_id")? == claim.engineering_run_id
            && row.try_get::<Uuid, _>("event_id")? == claim.event_id
            && row.try_get::<Uuid, _>("sender_service_id")? == route.sender_service_id
            && row.try_get::<Uuid, _>("receiver_service_id")? == route.receiver_service_id
            && row.try_get::<String, _>("transport_provider_id")? == route.provider_id
            && row.try_get::<String, _>("transport_provider_version")? == route.provider_version
            && row.try_get::<Uuid, _>("transport_profile_id")? == route.profile_id
            && row.try_get::<i64, _>("transport_profile_revision")? == route.profile_revision
            && persisted_profile_digest.as_slice() == route.profile_digest.as_slice()
            && row.try_get::<String, _>("endpoint_identity")? == route.endpoint_identity
            && row.try_get::<String, _>("destination_id")? == route.destination_id
            && row.try_get::<Uuid, _>("destination_generation")? == route.destination_generation
            && row.try_get::<i32, _>("schema_version")? == claim.schema_version
            && persisted_wire_digest.as_slice() == wire_sha256.as_slice()
            && !persisted_worker.is_nil()
            && (1..=MAX_DELIVERY_ATTEMPTS).contains(&persisted_attempt);
        if !route_matches {
            return Err(TaskRunOutboxDeliveryError::RouteConflict);
        }

        tx.commit().await?;
        Ok(PreparedTaskRunOutboxDelivery {
            claim: claim.clone(),
            route: route.clone(),
            wire,
            wire_sha256,
        })
    }

    /// Schedule an exact retry under the current worker and attempt fence.
    pub async fn settle_retry(
        &self,
        claim: &TaskRunOutboxDeliveryClaim,
    ) -> Result<(), TaskRunOutboxDeliveryError> {
        validate_claim(claim)?;
        let mut tx = self.begin_scoped(claim.tenant_id).await?;
        sqlx::query(
            r#"SELECT multica.settle_task_run_outbox_delivery(
                   $1,$2,$3,'retry',NULL,NULL,NULL)"#,
        )
        .bind(claim.event_id)
        .bind(claim.worker_id)
        .bind(claim.attempt_count)
        .execute(&mut *tx)
        .await
        .map_err(map_lease_error)?;
        tx.commit().await?;
        Ok(())
    }

    /// Permanently dead-letter only a current claim proven to violate the Task event contract.
    pub async fn settle_dead(
        &self,
        proof: &TaskRunOutboxInvalidEnvelopeProof,
    ) -> Result<(), TaskRunOutboxDeliveryError> {
        let mut tx = self.begin_scoped(proof.tenant_id).await?;
        sqlx::query(
            r#"SELECT multica.settle_task_run_outbox_delivery(
                   $1,$2,$3,'dead',NULL,NULL,'invalid_envelope')"#,
        )
        .bind(proof.event_id)
        .bind(proof.worker_id)
        .bind(proof.attempt_count)
        .execute(&mut *tx)
        .await
        .map_err(map_lease_error)?;
        tx.commit().await?;
        Ok(())
    }

    async fn begin_scoped(
        &self,
        tenant_id: Uuid,
    ) -> Result<Transaction<'static, Postgres>, TaskRunOutboxDeliveryError> {
        let mut tx = self.pool.begin().await?;
        crate::outbox::set_run_owner_scope(&mut tx, tenant_id).await?;
        sqlx::query("SET LOCAL lock_timeout = '5s'")
            .execute(&mut *tx)
            .await?;
        sqlx::query("SET LOCAL statement_timeout = '10s'")
            .execute(&mut *tx)
            .await?;
        Ok(tx)
    }
}

#[derive(Serialize)]
struct TaskRunOutboxWireEnvelope<'a> {
    envelope_version: u8,
    tenant_id: Uuid,
    project_id: Uuid,
    engineering_run_id: Uuid,
    event_id: Uuid,
    work_item_id: Uuid,
    aggregate_type: &'a str,
    aggregate_version: i32,
    event_type: &'a str,
    schema_version: i32,
    actor_id: Uuid,
    correlation_id: Uuid,
    occurred_at: DateTime<Utc>,
    payload: &'a Value,
}

fn build_exact_wire(
    claim: &TaskRunOutboxDeliveryClaim,
) -> Result<Vec<u8>, TaskRunOutboxDeliveryError> {
    validate_claim(claim)?;
    let envelope = TaskRunOutboxWireEnvelope {
        envelope_version: 1,
        tenant_id: claim.tenant_id,
        project_id: claim.project_id,
        engineering_run_id: claim.engineering_run_id,
        event_id: claim.event_id,
        work_item_id: claim.work_item_id,
        aggregate_type: &claim.aggregate_type,
        aggregate_version: claim.aggregate_version,
        event_type: &claim.event_type,
        schema_version: claim.schema_version,
        actor_id: claim.actor_id,
        correlation_id: claim.correlation_id,
        occurred_at: claim.occurred_at,
        payload: &claim.payload,
    };
    let wire = serde_json::to_vec(&envelope)?;
    if wire.len() > MAX_TASK_WIRE_BYTES {
        return Err(TaskRunOutboxDeliveryError::InvalidContract);
    }
    Ok(wire)
}

fn validate_claim(claim: &TaskRunOutboxDeliveryClaim) -> Result<(), TaskRunOutboxDeliveryError> {
    validate_claim_fence(claim)?;
    if claim.project_id.is_nil()
        || claim.engineering_run_id.is_nil()
        || claim.work_item_id.is_nil()
        || claim.actor_id.is_nil()
        || claim.correlation_id.is_nil()
        || claim.aggregate_version <= 0
    {
        return Err(TaskRunOutboxDeliveryError::InvalidContract);
    }
    validate_event(
        &claim.aggregate_type,
        &claim.event_type,
        claim.schema_version,
        &claim.payload,
        claim.tenant_id,
        claim.project_id,
        claim.engineering_run_id,
        claim.work_item_id,
        claim.aggregate_version,
    )
}

fn validate_claim_fence(
    claim: &TaskRunOutboxDeliveryClaim,
) -> Result<(), TaskRunOutboxDeliveryError> {
    if claim.tenant_id.is_nil()
        || claim.event_id.is_nil()
        || claim.worker_id.is_nil()
        || !(1..=MAX_DELIVERY_ATTEMPTS).contains(&claim.attempt_count)
    {
        return Err(TaskRunOutboxDeliveryError::InvalidContract);
    }
    Ok(())
}

fn validate_event(
    aggregate_type: &str,
    event_type: &str,
    schema_version: i32,
    payload: &Value,
    tenant_id: Uuid,
    project_id: Uuid,
    engineering_run_id: Uuid,
    work_item_id: Uuid,
    aggregate_version: i32,
) -> Result<(), TaskRunOutboxDeliveryError> {
    let compatible = match (schema_version, aggregate_type, event_type) {
        (1 | 2, "metadata", "task.created" | "task.updated" | "task.owner_changed") => true,
        (3, "lifecycle", "task.lifecycle.changed") => true,
        _ => false,
    };
    if !compatible || !payload.is_object() {
        return Err(TaskRunOutboxDeliveryError::InvalidContract);
    }
    let task_id = Value::String(work_item_id.to_string());
    let project = Value::String(project_id.to_string());
    let run = Value::String(engineering_run_id.to_string());
    let tenant = Value::String(tenant_id.to_string());
    let version = Value::from(aggregate_version);
    let payload_scope_matches = match schema_version {
        1 => {
            payload["work_item_id"] == task_id
                && payload["project_id"] == project
                && payload["engineering_run_id"] == run
                && payload["metadata_version"] == version
        }
        2 => {
            payload["schema_version"] == Value::from(2)
                && payload["work_item_id"] == task_id
                && payload["owner"]["tenant_id"] == tenant
                && payload["owner"]["project_id"] == project
                && payload["owner"]["engineering_run_id"] == run
                && payload["metadata_version"] == version
        }
        3 => {
            payload["schema_version"] == Value::from(3)
                && payload["aggregate"]["type"] == aggregate_type
                && payload["aggregate"]["version"] == version
                && payload["work_item_id"] == task_id
                && payload["owner"]["tenant_id"] == tenant
                && payload["owner"]["project_id"] == project
                && payload["owner"]["engineering_run_id"] == run
        }
        _ => false,
    };
    if !payload_scope_matches {
        return Err(TaskRunOutboxDeliveryError::InvalidContract);
    }
    if serde_json::to_vec(payload)?.len() > MAX_TASK_PAYLOAD_BYTES {
        return Err(TaskRunOutboxDeliveryError::InvalidContract);
    }
    Ok(())
}

fn valid_provider_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_transport_version(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-')
        })
}

fn valid_route_label(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

fn map_lease_error(error: sqlx::Error) -> TaskRunOutboxDeliveryError {
    match &error {
        sqlx::Error::Database(database)
            if matches!(database.code().as_deref(), Some("40001" | "55000")) =>
        {
            if database.code().as_deref() == Some("40001") {
                TaskRunOutboxDeliveryError::LeaseFence
            } else {
                TaskRunOutboxDeliveryError::RouteConflict
            }
        }
        _ => TaskRunOutboxDeliveryError::Database(error),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn claim() -> TaskRunOutboxDeliveryClaim {
        TaskRunOutboxDeliveryClaim {
            tenant_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            engineering_run_id: Uuid::from_u128(3),
            event_id: Uuid::from_u128(4),
            work_item_id: Uuid::from_u128(5),
            aggregate_type: "lifecycle".to_owned(),
            aggregate_version: 7,
            event_type: "task.lifecycle.changed".to_owned(),
            schema_version: 3,
            actor_id: Uuid::from_u128(6),
            correlation_id: Uuid::from_u128(7),
            payload: serde_json::json!({
                "schema_version": 3,
                "aggregate": {"type": "lifecycle", "version": 7},
                "work_item_id": Uuid::from_u128(5),
                "owner": {
                    "tenant_id": Uuid::from_u128(1),
                    "project_id": Uuid::from_u128(2),
                    "engineering_run_id": Uuid::from_u128(3)
                },
                "lifecycle": {"status": "in_progress"}
            }),
            occurred_at: Utc.with_ymd_and_hms(2026, 10, 16, 12, 30, 0).unwrap(),
            worker_id: Uuid::from_u128(8),
            attempt_count: 1,
            lease_expires_at: Utc.with_ymd_and_hms(2026, 10, 16, 12, 31, 0).unwrap(),
        }
    }

    #[test]
    fn exact_wire_is_stable_and_preserves_run_aggregate_identity() {
        let source = claim();
        let first = build_exact_wire(&source).unwrap();
        let replay = build_exact_wire(&source).unwrap();
        let parsed: Value = serde_json::from_slice(&first).unwrap();

        assert_eq!(first, replay);
        assert!(first.len() <= MAX_TASK_WIRE_BYTES);
        assert_eq!(parsed["engineering_run_id"], source.engineering_run_id.to_string());
        assert_eq!(parsed["event_id"], source.event_id.to_string());
        assert_eq!(parsed["aggregate_type"], "lifecycle");
        assert_eq!(parsed["aggregate_version"], 7);
        assert_eq!(parsed["schema_version"], 3);
    }

    #[test]
    fn unknown_event_schema_and_oversized_payload_fail_closed() {
        let mut source = claim();
        source.event_type = "task.lifecycle.unknown".to_owned();
        assert!(matches!(
            build_exact_wire(&source),
            Err(TaskRunOutboxDeliveryError::InvalidContract)
        ));

        let mut source = claim();
        source.payload["owner"]["engineering_run_id"] = Uuid::from_u128(99).to_string().into();
        assert!(matches!(
            build_exact_wire(&source),
            Err(TaskRunOutboxDeliveryError::InvalidContract)
        ));

        let mut source = claim();
        source.payload = serde_json::json!({"large": "x".repeat(MAX_TASK_PAYLOAD_BYTES)});
        assert!(matches!(
            build_exact_wire(&source),
            Err(TaskRunOutboxDeliveryError::InvalidContract)
        ));
    }

    #[test]
    fn route_labels_reject_endpoint_and_secret_shaped_values() {
        assert!(valid_route_label("task-run-events-v1"));
        assert!(!valid_route_label("https://receiver.example/secret"));
        assert!(!valid_provider_id("NATS"));
        assert!(!valid_transport_version("0.50.0\n"));
    }
}
