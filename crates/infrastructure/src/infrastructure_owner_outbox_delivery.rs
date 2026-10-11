//! PostgreSQL delivery state machine for immutable Infrastructure owner Outbox events.
//!
//! @cypher schema=1 source_sha256=9c6090b2029c702abc9c3e45d34079c3d5e0514e988c52854f1d94605ab99c3f
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::infrastructure_owner_outbox_delivery",kind:"module"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"struct"})
//! MERGE (claim:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryClaim",kind:"struct"})
//! MERGE (failure:Type {id:"infrastructure::OutboxDeliveryFailureKind",kind:"enum"})
//! MERGE (error:Type {id:"infrastructure::InfrastructureOwnerOutboxDeliveryError",kind:"enum"})
//! MERGE (claim_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_due",kind:"method"})
//! MERGE (mark_delivered:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.mark_delivered",kind:"method"})
//! MERGE (record_failure:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.record_failure",kind:"method"})
//! MERGE (publish_due:Symbol {id:"crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs::InfrastructureOwnerJetStreamPublisher.publish_due_for_tenant",kind:"method"})
//! MERGE (set_scope:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::set_owner_scope",kind:"function"})
//! MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery",class:"Work"})
//! MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox",class:"Transaction"})
//! MERGE (pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(store)
//! MERGE (module)-[:DEFINES]->(claim)
//! MERGE (module)-[:DEFINES]->(failure)
//! MERGE (module)-[:DEFINES]->(error)
//! MERGE (store)-[:DEFINES]->(claim_due)
//! MERGE (store)-[:DEFINES]->(mark_delivered)
//! MERGE (store)-[:DEFINES]->(record_failure)
//! MERGE (store)-[:USES_TYPE]->(pool)
//! MERGE (claim_due)-[:CALLS]->(set_scope)
//! MERGE (claim_due)-[:READS]->(delivery)
//! MERGE (claim_due)-[:WRITES]->(delivery)
//! MERGE (claim_due)-[:READS]->(outbox)
//! MERGE (mark_delivered)-[:CALLS]->(set_scope)
//! MERGE (mark_delivered)-[:WRITES]->(delivery)
//! MERGE (record_failure)-[:CALLS]->(set_scope)
//! MERGE (record_failure)-[:WRITES]->(delivery)
//! MERGE (publish_due)-[:CALLS]->(claim_due)
//! MERGE (publish_due)-[:CALLS]->(mark_delivered)
//! MERGE (publish_due)-[:CALLS]->(record_failure)
//! @endcypher

use sqlx::{postgres::PgRow, PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

const OWNER_ROLE: &str = "SET LOCAL ROLE star_infrastructure_owner";
const MAX_BATCH_SIZE: i64 = 64;
const MAX_LEASE_SECONDS: i64 = 300;
const MAX_DELIVERY_ATTEMPTS: i32 = 12;
const MAX_BACKOFF_SECONDS: i64 = 3_600;

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
               AND lease_expires_at > clock_timestamp()",
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
