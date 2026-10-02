/*
@cypher schema=1 source_sha256=7f281dc13a35ff72f970c3909fb30162e00acc9b0dde5c2ee2f74ed86fb6a405
MERGE (self:File {path:"crates/star-pg-adapter/src/repository/automation_schedule.rs"})
MERGE (repo:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository"})
MERGE (error:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::ScheduleRepositoryError"})
MERGE (result:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::ScheduleDispatchResult"})
MERGE (terminal:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::OccurrenceTerminalOutcome"})
MERGE (persist:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::persist_materialization_batch",kind:"method"})
MERGE (load:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::load_current_rule",kind:"method"})
MERGE (claim:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::claim_due_occurrences",kind:"method"})
MERGE (heartbeat:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::heartbeat",kind:"method"})
MERGE (failure:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::record_failure",kind:"method"})
MERGE (terminalize:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::finish",kind:"method"})
MERGE (purge:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::purge_expired_dispatch_rows",kind:"method"})
MERGE (tenant:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::set_tenant_scope",kind:"function"})
MERGE (decode:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::decode_rule",kind:"function"})
MERGE (rule_match:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::persisted_rule_matches",kind:"function"})
MERGE (pool:ExternalService {id:"sqlx.PgPool",kind:"type"})
MERGE (transaction:ExternalService {id:"sqlx.Transaction",kind:"type"})
MERGE (schedule_rule:Table {id:"automation.schedule_rule_revision"})
MERGE (occurrence:Table {id:"automation.occurrence"})
MERGE (dispatch:Table {id:"automation.occurrence_dispatch"})
MERGE (event:Table {id:"automation.occurrence_event"})
MERGE (self)-[:DEFINES]->(repo)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(result)
MERGE (self)-[:DEFINES]->(terminal)
MERGE (self)-[:DEFINES]->(persist)
MERGE (self)-[:DEFINES]->(load)
MERGE (self)-[:DEFINES]->(claim)
MERGE (self)-[:DEFINES]->(heartbeat)
MERGE (self)-[:DEFINES]->(failure)
MERGE (self)-[:DEFINES]->(terminalize)
MERGE (self)-[:DEFINES]->(purge)
MERGE (self)-[:DEFINES]->(tenant)
MERGE (self)-[:DEFINES]->(decode)
MERGE (self)-[:DEFINES]->(rule_match)
MERGE (repo)-[:USES_TYPE]->(pool)
MERGE (persist)-[:CALLS]->(set_tenant_scope)
MERGE (persist)-[:WRITES]->(occurrence)
MERGE (persist)-[:WRITES]->(dispatch)
MERGE (persist)-[:WRITES]->(event)
MERGE (load)-[:CALLS]->(set_tenant_scope)
MERGE (load)-[:CALLS]->(decode)
MERGE (load)-[:READS]->(schedule_rule)
MERGE (claim)-[:CALLS]->(set_tenant_scope)
MERGE (claim)-[:READS]->(schedule_rule)
MERGE (claim)-[:WRITES]->(dispatch)
MERGE (claim)-[:WRITES]->(event)
MERGE (heartbeat)-[:CALLS]->(set_tenant_scope)
MERGE (heartbeat)-[:WRITES]->(dispatch)
MERGE (failure)-[:CALLS]->(set_tenant_scope)
MERGE (failure)-[:READS]->(schedule_rule)
MERGE (failure)-[:WRITES]->(dispatch)
MERGE (failure)-[:WRITES]->(event)
MERGE (terminalize)-[:CALLS]->(set_tenant_scope)
MERGE (terminalize)-[:WRITES]->(dispatch)
MERGE (terminalize)-[:WRITES]->(event)
MERGE (purge)-[:CALLS]->(set_tenant_scope)
MERGE (purge)-[:WRITES]->(dispatch)
MERGE (rule_match)-[:READS]->(schedule_rule)
@endcypher
*/

//! PostgreSQL persistence and fenced dispatch operations for Schedule occurrences.

use chrono::{DateTime, Duration, Utc};
use domain_automation::RuleId;
use domain_automation::schedule::{
    AutomationOccurrenceLeaseFenceV1, AutomationScheduleRuleRevisionV1,
    MaterializedScheduleOccurrenceV1, ScheduleDstFoldPolicyV1, ScheduleDstGapPolicyV1,
    ScheduleMaterializationBatchV1, ScheduleMisfirePolicyV1, ScheduleOccurrenceDispositionV1,
    ScheduleOverlapPolicyV1, SchedulePausePolicyV1, ScheduleRetryPolicyV1, ScheduleTargetV1,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

const MAX_CLAIM_PAGE: i64 = 100;
const MAX_LEASE_SECONDS: i64 = 300;

/// Safe outcomes written after the owning Run has reached a terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccurrenceTerminalOutcome {
    /// The admitted Run completed successfully.
    Succeeded,
    /// The admitted Run or occurrence was explicitly cancelled.
    Cancelled,
}

/// Result of a fenced failure transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleDispatchResult {
    /// A retry has been durably scheduled.
    RetryScheduled {
        /// Database-authoritative time when the retry becomes claimable.
        next_attempt_at: DateTime<Utc>,
    },
    /// Retry budget or occurrence deadline is exhausted.
    Failed,
}

/// Errors that do not expose SQL parameters, tenant data, or connection details.
#[derive(Debug, Error)]
pub enum ScheduleRepositoryError {
    /// A required tenant, worker, lease duration, batch, or failure code is invalid.
    #[error("invalid schedule repository input")]
    InvalidInput,
    /// A materialized occurrence does not match its persisted immutable Rule revision.
    #[error("schedule Rule revision does not match the persisted snapshot")]
    RuleRevisionMismatch,
    /// The supplied occurrence snapshot is incomplete or malformed.
    #[error("invalid Schedule occurrence snapshot")]
    InvalidOccurrence,
    /// No row matched the current tenant, owner, fencing generation, and lease.
    #[error("Schedule occurrence lease is stale, expired, or not owned by this worker")]
    StaleFence,
    /// A persisted schedule policy contains a value unknown to this adapter.
    #[error("persisted Schedule Rule contains an unsupported policy value")]
    CorruptRule,
    /// A PostgreSQL operation failed; the driver message is retained for diagnostics.
    #[error("Schedule PostgreSQL operation failed")]
    Database(#[source] sqlx::Error),
}

impl From<sqlx::Error> for ScheduleRepositoryError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

/// Tenant-scoped repository for materialized Schedule occurrences and worker leases.
#[derive(Clone)]
pub struct PgAutomationScheduleRepository {
    pool: PgPool,
}

impl PgAutomationScheduleRepository {
    /// Construct an adapter over the application's shared bounded PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load the enabled current Rule revision under tenant RLS for materialization.
    pub async fn load_current_rule(
        &self,
        tenant_id: Uuid,
        rule_id: RuleId,
    ) -> Result<AutomationScheduleRuleRevisionV1, ScheduleRepositoryError> {
        if tenant_id.is_nil() || rule_id.as_uuid().is_nil() {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        let row = sqlx::query(
            "SELECT * FROM automation.schedule_rule_revision \
             WHERE tenant_id = $1 AND rule_id = $2 AND enabled AND valid_to IS NULL",
        )
        .bind(tenant_id)
        .bind(rule_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ScheduleRepositoryError::RuleRevisionMismatch)?;
        let rule = decode_rule(&row)?;
        tx.commit().await?;
        Ok(rule)
    }

    /// Persist one bounded materialization page atomically and idempotently.
    pub async fn persist_materialization_batch(
        &self,
        tenant_id: Uuid,
        batch: &ScheduleMaterializationBatchV1,
    ) -> Result<Vec<Uuid>, ScheduleRepositoryError> {
        if tenant_id.is_nil() || batch.occurrences.len() > 256 {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        let mut inserted = Vec::with_capacity(batch.occurrences.len());
        for occurrence in &batch.occurrences {
            validate_materialized_occurrence(tenant_id, occurrence)?;
            let rule = &occurrence.snapshot.rule_snapshot;
            if !persisted_rule_matches(&mut tx, rule).await? {
                return Err(ScheduleRepositoryError::RuleRevisionMismatch);
            }
            let target_snapshot = serde_json::to_value(&rule.target)
                .map_err(|_| ScheduleRepositoryError::InvalidOccurrence)?;
            let target_bytes = serde_json::to_vec(&target_snapshot)
                .map_err(|_| ScheduleRepositoryError::InvalidOccurrence)?;
            let target_digest = hex::encode(Sha256::digest(target_bytes));
            let key = &occurrence.snapshot.key;
            let inserted_id = sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO automation.occurrence \
                    (occurrence_id, tenant_id, project_id, rule_id, rule_version, \
                     scheduled_for_utc, scheduled_local_label, utc_offset_seconds, \
                     parser_version, tzdb_version, target_snapshot, target_snapshot_digest, materialized_at) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) \
                 ON CONFLICT (tenant_id, rule_id, rule_version, scheduled_for_utc) DO NOTHING \
                 RETURNING occurrence_id",
            )
            .bind(occurrence.snapshot.occurrence_id)
            .bind(tenant_id)
            .bind(rule.project_id)
            .bind(key.rule_id.as_uuid())
            .bind(i64::try_from(key.rule_version).map_err(|_| ScheduleRepositoryError::InvalidOccurrence)?)
            .bind(key.scheduled_for_utc)
            .bind(&occurrence.snapshot.scheduled_local_label)
            .bind(occurrence.snapshot.utc_offset_seconds)
            .bind(&rule.parser_version)
            .bind(&rule.tzdb_version)
            .bind(target_snapshot)
            .bind(target_digest)
            .bind(occurrence.snapshot.materialized_at)
            .fetch_optional(&mut *tx)
            .await?;

            let Some(occurrence_id) = inserted_id else {
                continue;
            };
            let is_skipped = occurrence.disposition == ScheduleOccurrenceDispositionV1::Skipped;
            sqlx::query(
                "WITH stamp AS (SELECT clock_timestamp() AS now) \
                 INSERT INTO automation.occurrence_dispatch \
                    (tenant_id, occurrence_id, dispatch_state, next_attempt_at, occurrence_deadline_at, terminal_at, expires_at) \
                 SELECT $1,$2,$3,$4, GREATEST($5, stamp.now) + ($6 * INTERVAL '1 second'), \
                    CASE WHEN $7 THEN stamp.now END, \
                    CASE WHEN $7 THEN stamp.now + INTERVAL '30 days' END \
                 FROM stamp",
            )
            .bind(tenant_id)
            .bind(occurrence_id)
            .bind(if is_skipped { "skipped" } else { "pending" })
            .bind(key.scheduled_for_utc)
            .bind(occurrence.snapshot.materialized_at)
            .bind(i64::from(rule.deadline_seconds))
            .bind(is_skipped)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO automation.occurrence_event \
                    (tenant_id, project_id, occurrence_id, event_type, correlation_id, details) \
                 VALUES ($1,$2,$3,$4,gen_random_uuid(),$5)",
            )
            .bind(tenant_id)
            .bind(rule.project_id)
            .bind(occurrence_id)
            .bind(if is_skipped {
                "skipped"
            } else {
                "materialized"
            })
            .bind(json!({
                "scheduled_local_label": occurrence.snapshot.scheduled_local_label,
                "utc_offset_seconds": occurrence.snapshot.utc_offset_seconds,
                "initial_disposition": if is_skipped { "skipped" } else { "pending" }
            }))
            .execute(&mut *tx)
            .await?;
            inserted.push(occurrence_id);
        }
        tx.commit().await?;
        Ok(inserted)
    }

    /// Claim due occurrences with tenant-local RLS, `SKIP LOCKED`, and a new fencing generation.
    pub async fn claim_due_occurrences(
        &self,
        tenant_id: Uuid,
        worker_id: Uuid,
        lease_seconds: i64,
        limit: i64,
    ) -> Result<Vec<AutomationOccurrenceLeaseFenceV1>, ScheduleRepositoryError> {
        if tenant_id.is_nil()
            || worker_id.is_nil()
            || !(1..=MAX_LEASE_SECONDS).contains(&lease_seconds)
            || !(1..=MAX_CLAIM_PAGE).contains(&limit)
        {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        finalize_expired_occurrences(&mut tx, tenant_id, limit).await?;
        let rows = sqlx::query(
            "WITH due AS MATERIALIZED ( \
                SELECT d.tenant_id, d.occurrence_id, d.dispatch_state AS previous_state \
                FROM automation.occurrence_dispatch d \
                JOIN automation.occurrence o USING (tenant_id, occurrence_id) \
                JOIN automation.schedule_rule_revision r \
                  ON r.tenant_id = o.tenant_id AND r.project_id = o.project_id \
                 AND r.rule_id = o.rule_id AND r.rule_version = o.rule_version \
                WHERE d.tenant_id = $1 AND d.occurrence_deadline_at > clock_timestamp() \
                  AND d.attempt_count < r.retry_max_attempts \
                  AND ((d.dispatch_state IN ('pending','retry_wait') AND d.next_attempt_at <= clock_timestamp()) \
                    OR (d.dispatch_state = 'leased' AND d.lease_expires_at <= clock_timestamp())) \
                ORDER BY COALESCE(d.lease_expires_at,d.next_attempt_at), d.occurrence_id \
                FOR UPDATE OF d SKIP LOCKED LIMIT $4 \
             ), claimed AS ( \
                UPDATE automation.occurrence_dispatch d \
                SET dispatch_state = 'leased', attempt_count = d.attempt_count + 1, \
                    fencing_generation = d.fencing_generation + 1, lease_owner_id = $2, \
                    lease_expires_at = LEAST(clock_timestamp() + ($3 * INTERVAL '1 second'), d.occurrence_deadline_at), \
                    last_failure_code = NULL, updated_at = clock_timestamp() \
                FROM due \
                WHERE d.tenant_id = due.tenant_id AND d.occurrence_id = due.occurrence_id \
                RETURNING d.tenant_id, d.occurrence_id, d.attempt_count, d.fencing_generation, \
                          d.lease_owner_id, d.lease_expires_at, d.occurrence_deadline_at \
             ), expired_events AS ( \
                INSERT INTO automation.occurrence_event \
                    (tenant_id, project_id, occurrence_id, event_type, attempt_no, fencing_generation, correlation_id, details) \
                SELECT c.tenant_id, o.project_id, c.occurrence_id, 'lease_expired', c.attempt_count - 1, \
                       c.fencing_generation - 1, gen_random_uuid(), jsonb_build_object('reclaimed_by', $2) \
                FROM claimed c JOIN due ON due.tenant_id = c.tenant_id AND due.occurrence_id = c.occurrence_id \
                JOIN automation.occurrence o \
                  ON o.tenant_id = c.tenant_id AND o.occurrence_id = c.occurrence_id \
                WHERE due.previous_state = 'leased' RETURNING event_id \
             ), claimed_events AS ( \
                INSERT INTO automation.occurrence_event \
                    (tenant_id, project_id, occurrence_id, event_type, attempt_no, fencing_generation, correlation_id, details) \
                SELECT c.tenant_id, o.project_id, c.occurrence_id, 'lease_claimed', c.attempt_count, \
                       c.fencing_generation, gen_random_uuid(), jsonb_build_object('worker_id', $2) \
                FROM claimed c JOIN automation.occurrence o USING (tenant_id, occurrence_id) \
                RETURNING event_id \
             ) \
             SELECT tenant_id, occurrence_id, fencing_generation, lease_owner_id, lease_expires_at, occurrence_deadline_at \
             FROM claimed ORDER BY lease_expires_at, occurrence_id",
        )
        .bind(tenant_id)
        .bind(worker_id)
        .bind(lease_seconds)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        let fences = rows
            .iter()
            .map(|row| {
                Ok(AutomationOccurrenceLeaseFenceV1 {
                    tenant_id: row.try_get("tenant_id")?,
                    occurrence_id: row.try_get("occurrence_id")?,
                    fencing_generation: row.try_get::<i64, _>("fencing_generation")? as u64,
                    lease_owner_id: row.try_get("lease_owner_id")?,
                    lease_expires_at: row.try_get("lease_expires_at")?,
                    deadline_at: row.try_get("occurrence_deadline_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        tx.commit().await?;
        Ok(fences)
    }

    /// Extend a live lease without changing its owner or fencing generation.
    pub async fn heartbeat(
        &self,
        tenant_id: Uuid,
        fence: &AutomationOccurrenceLeaseFenceV1,
        lease_seconds: i64,
    ) -> Result<AutomationOccurrenceLeaseFenceV1, ScheduleRepositoryError> {
        if tenant_id.is_nil()
            || tenant_id != fence.tenant_id
            || fence.occurrence_id.is_nil()
            || fence.lease_owner_id.is_nil()
            || fence.fencing_generation == 0
            || !(1..=MAX_LEASE_SECONDS).contains(&lease_seconds)
        {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        let row = sqlx::query(
            "UPDATE automation.occurrence_dispatch \
             SET lease_expires_at = GREATEST(lease_expires_at, \
                 LEAST(clock_timestamp() + ($5 * INTERVAL '1 second'), occurrence_deadline_at)), \
                 updated_at = clock_timestamp() \
             WHERE tenant_id = $1 AND occurrence_id = $2 AND dispatch_state = 'leased' \
               AND lease_owner_id = $3 AND fencing_generation = $4 \
               AND lease_expires_at > clock_timestamp() AND occurrence_deadline_at > clock_timestamp() \
             RETURNING fencing_generation, lease_owner_id, lease_expires_at, occurrence_deadline_at",
        )
        .bind(tenant_id)
        .bind(fence.occurrence_id)
        .bind(fence.lease_owner_id)
        .bind(i64::try_from(fence.fencing_generation).map_err(|_| ScheduleRepositoryError::InvalidInput)?)
        .bind(lease_seconds)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ScheduleRepositoryError::StaleFence)?;
        let renewed = AutomationOccurrenceLeaseFenceV1 {
            tenant_id,
            occurrence_id: fence.occurrence_id,
            fencing_generation: row.try_get::<i64, _>("fencing_generation")? as u64,
            lease_owner_id: row.try_get("lease_owner_id")?,
            lease_expires_at: row.try_get("lease_expires_at")?,
            deadline_at: row.try_get("occurrence_deadline_at")?,
        };
        tx.commit().await?;
        Ok(renewed)
    }

    /// Record a bounded failure and either schedule exponential backoff or terminate the occurrence.
    pub async fn record_failure(
        &self,
        tenant_id: Uuid,
        fence: &AutomationOccurrenceLeaseFenceV1,
        failure_code: &str,
    ) -> Result<ScheduleDispatchResult, ScheduleRepositoryError> {
        if tenant_id.is_nil()
            || tenant_id != fence.tenant_id
            || fence.occurrence_id.is_nil()
            || fence.lease_owner_id.is_nil()
            || fence.fencing_generation == 0
            || failure_code.is_empty()
            || failure_code.len() > 64
            || !failure_code.is_ascii()
            || failure_code.chars().any(char::is_control)
        {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        let row = sqlx::query(
            "SELECT d.attempt_count, d.occurrence_deadline_at, clock_timestamp() AS db_now, \
                    r.retry_max_attempts, r.retry_initial_backoff_seconds, r.retry_max_backoff_seconds \
             FROM automation.occurrence_dispatch d \
             JOIN automation.occurrence o USING (tenant_id, occurrence_id) \
             JOIN automation.schedule_rule_revision r \
               ON r.tenant_id = o.tenant_id AND r.project_id = o.project_id \
              AND r.rule_id = o.rule_id AND r.rule_version = o.rule_version \
             WHERE d.tenant_id = $1 AND d.occurrence_id = $2 AND d.dispatch_state = 'leased' \
               AND d.lease_owner_id = $3 AND d.fencing_generation = $4 \
               AND d.lease_expires_at > clock_timestamp() AND d.occurrence_deadline_at > clock_timestamp() \
             FOR UPDATE OF d",
        )
        .bind(tenant_id)
        .bind(fence.occurrence_id)
        .bind(fence.lease_owner_id)
        .bind(i64::try_from(fence.fencing_generation).map_err(|_| ScheduleRepositoryError::InvalidInput)?)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ScheduleRepositoryError::StaleFence)?;

        let attempt_count: i16 = row.try_get("attempt_count")?;
        let max_attempts: i16 = row.try_get("retry_max_attempts")?;
        let initial_backoff: i32 = row.try_get("retry_initial_backoff_seconds")?;
        let max_backoff: i32 = row.try_get("retry_max_backoff_seconds")?;
        let db_now: DateTime<Utc> = row.try_get("db_now")?;
        let deadline: DateTime<Utc> = row.try_get("occurrence_deadline_at")?;
        let exponent = u32::try_from(attempt_count.saturating_sub(1))
            .map_err(|_| ScheduleRepositoryError::CorruptRule)?
            .min(30);
        let multiplier = 1_i64.checked_shl(exponent).unwrap_or(i64::MAX);
        let backoff_seconds =
            (i64::from(initial_backoff).saturating_mul(multiplier)).min(i64::from(max_backoff));
        let next_attempt_at = db_now + Duration::seconds(backoff_seconds);
        let retry_allowed = attempt_count < max_attempts && next_attempt_at < deadline;
        let generation = i64::try_from(fence.fencing_generation)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?;

        if retry_allowed {
            sqlx::query(
                "UPDATE automation.occurrence_dispatch \
                 SET dispatch_state = 'retry_wait', next_attempt_at = $5, lease_owner_id = NULL, \
                     lease_expires_at = NULL, last_failure_code = $6, updated_at = $7 \
                 WHERE tenant_id = $1 AND occurrence_id = $2 AND lease_owner_id = $3 \
                   AND fencing_generation = $4 AND dispatch_state = 'leased'",
            )
            .bind(tenant_id)
            .bind(fence.occurrence_id)
            .bind(fence.lease_owner_id)
            .bind(generation)
            .bind(next_attempt_at)
            .bind(failure_code)
            .bind(db_now)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO automation.occurrence_event \
                    (tenant_id, project_id, occurrence_id, event_type, attempt_no, fencing_generation, correlation_id, details) \
                 SELECT $1,o.project_id,$2,'retry_scheduled',$3,$4,gen_random_uuid(), \
                        jsonb_build_object('failure_code',$5,'next_attempt_at',$6) \
                 FROM automation.occurrence o WHERE o.tenant_id = $1 AND o.occurrence_id = $2",
            )
            .bind(tenant_id)
            .bind(fence.occurrence_id)
            .bind(attempt_count)
            .bind(generation)
            .bind(failure_code)
            .bind(next_attempt_at)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(ScheduleDispatchResult::RetryScheduled { next_attempt_at })
        } else {
            sqlx::query(
                "UPDATE automation.occurrence_dispatch \
                 SET dispatch_state = 'failed', lease_owner_id = NULL, lease_expires_at = NULL, \
                     last_failure_code = $5, terminal_at = $6, expires_at = $6 + retention_period, updated_at = $6 \
                 WHERE tenant_id = $1 AND occurrence_id = $2 AND lease_owner_id = $3 \
                   AND fencing_generation = $4 AND dispatch_state = 'leased'",
            )
            .bind(tenant_id)
            .bind(fence.occurrence_id)
            .bind(fence.lease_owner_id)
            .bind(generation)
            .bind(failure_code)
            .bind(db_now)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO automation.occurrence_event \
                    (tenant_id, project_id, occurrence_id, event_type, attempt_no, fencing_generation, correlation_id, details) \
                 SELECT $1,o.project_id,$2,'dispatch_failed',$3,$4,gen_random_uuid(), \
                        jsonb_build_object('failure_code',$5) \
                 FROM automation.occurrence o WHERE o.tenant_id = $1 AND o.occurrence_id = $2",
            )
            .bind(tenant_id)
            .bind(fence.occurrence_id)
            .bind(attempt_count)
            .bind(generation)
            .bind(failure_code)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(ScheduleDispatchResult::Failed)
        }
    }

    /// Finish an admitted occurrence under its current, unexpired fencing token.
    pub async fn finish(
        &self,
        tenant_id: Uuid,
        fence: &AutomationOccurrenceLeaseFenceV1,
        outcome: OccurrenceTerminalOutcome,
    ) -> Result<(), ScheduleRepositoryError> {
        if tenant_id.is_nil()
            || tenant_id != fence.tenant_id
            || fence.occurrence_id.is_nil()
            || fence.lease_owner_id.is_nil()
            || fence.fencing_generation == 0
        {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let (state, event) = match outcome {
            OccurrenceTerminalOutcome::Succeeded => ("succeeded", "run_succeeded"),
            OccurrenceTerminalOutcome::Cancelled => ("cancelled", "cancelled"),
        };
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        let generation = i64::try_from(fence.fencing_generation)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?;
        let row = sqlx::query(
            "WITH stamp AS (SELECT clock_timestamp() AS now) \
             UPDATE automation.occurrence_dispatch d \
             SET dispatch_state = $5, lease_owner_id = NULL, lease_expires_at = NULL, \
                 terminal_at = stamp.now, expires_at = stamp.now + d.retention_period, updated_at = stamp.now \
             FROM stamp \
             WHERE d.tenant_id = $1 AND d.occurrence_id = $2 AND d.dispatch_state = 'leased' \
               AND d.lease_owner_id = $3 AND d.fencing_generation = $4 \
               AND d.lease_expires_at > stamp.now AND d.occurrence_deadline_at > stamp.now \
             RETURNING d.tenant_id, d.occurrence_id, d.attempt_count, d.fencing_generation",
        )
        .bind(tenant_id)
        .bind(fence.occurrence_id)
        .bind(fence.lease_owner_id)
        .bind(generation)
        .bind(state)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ScheduleRepositoryError::StaleFence)?;
        let attempt: i16 = row.try_get("attempt_count")?;
        sqlx::query(
            "INSERT INTO automation.occurrence_event \
                (tenant_id, project_id, occurrence_id, event_type, attempt_no, fencing_generation, correlation_id) \
             SELECT $1,o.project_id,$2,$3,$4,$5,gen_random_uuid() \
             FROM automation.occurrence o WHERE o.tenant_id = $1 AND o.occurrence_id = $2",
        )
        .bind(tenant_id)
        .bind(fence.occurrence_id)
        .bind(event)
        .bind(attempt)
        .bind(generation)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Remove only expired terminal dispatch Work rows; immutable occurrence facts remain.
    pub async fn purge_expired_dispatch_rows(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<u64, ScheduleRepositoryError> {
        if tenant_id.is_nil() || !(1..=MAX_CLAIM_PAGE).contains(&limit) {
            return Err(ScheduleRepositoryError::InvalidInput);
        }
        let mut tx = self.pool.begin().await?;
        set_tenant_scope(&mut tx, tenant_id).await?;
        let result = sqlx::query(
            "WITH expired AS ( \
                SELECT tenant_id, occurrence_id FROM automation.occurrence_dispatch \
                WHERE tenant_id = $1 AND dispatch_state IN ('succeeded','failed','skipped','cancelled') \
                  AND terminal_at IS NOT NULL AND expires_at <= clock_timestamp() \
                ORDER BY expires_at, occurrence_id FOR UPDATE SKIP LOCKED LIMIT $2 \
             ) \
             DELETE FROM automation.occurrence_dispatch d USING expired e \
             WHERE d.tenant_id = e.tenant_id AND d.occurrence_id = e.occurrence_id",
        )
        .bind(tenant_id)
        .bind(limit)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected())
    }
}

async fn set_tenant_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), ScheduleRepositoryError> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn finalize_expired_occurrences(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    limit: i64,
) -> Result<(), ScheduleRepositoryError> {
    sqlx::query(
        "WITH expired AS MATERIALIZED ( \
            SELECT d.tenant_id, d.occurrence_id, d.attempt_count, d.fencing_generation, \
                   d.occurrence_deadline_at, o.project_id \
            FROM automation.occurrence_dispatch d \
            JOIN automation.occurrence o USING (tenant_id, occurrence_id) \
            JOIN automation.schedule_rule_revision r \
              ON r.tenant_id = o.tenant_id AND r.project_id = o.project_id \
             AND r.rule_id = o.rule_id AND r.rule_version = o.rule_version \
            WHERE d.tenant_id = $1 AND d.dispatch_state IN ('pending','retry_wait','leased') \
              AND (d.occurrence_deadline_at <= clock_timestamp() \
                OR (d.dispatch_state = 'leased' AND d.lease_expires_at <= clock_timestamp() \
                    AND d.attempt_count >= r.retry_max_attempts)) \
            ORDER BY d.occurrence_deadline_at, d.occurrence_id \
            FOR UPDATE OF d SKIP LOCKED LIMIT $2 \
         ), failed AS ( \
            UPDATE automation.occurrence_dispatch d \
            SET dispatch_state = 'failed', lease_owner_id = NULL, lease_expires_at = NULL, \
                last_failure_code = CASE WHEN d.occurrence_deadline_at <= stamp.now \
                    THEN 'deadline_expired' ELSE 'attempts_exhausted' END, \
                terminal_at = stamp.now, expires_at = stamp.now + d.retention_period, updated_at = stamp.now \
            FROM expired CROSS JOIN (SELECT clock_timestamp() AS now) stamp \
            WHERE d.tenant_id = expired.tenant_id AND d.occurrence_id = expired.occurrence_id \
            RETURNING d.tenant_id, d.occurrence_id, d.attempt_count, d.fencing_generation, \
                      d.last_failure_code, expired.project_id \
         ) \
         INSERT INTO automation.occurrence_event \
            (tenant_id, project_id, occurrence_id, event_type, attempt_no, fencing_generation, correlation_id, details) \
         SELECT tenant_id, project_id, occurrence_id, \
                CASE WHEN last_failure_code = 'deadline_expired' THEN 'deadline_expired' ELSE 'dispatch_failed' END, \
                attempt_count, NULLIF(fencing_generation, 0), gen_random_uuid(), \
                jsonb_build_object('failure_code', last_failure_code) \
         FROM failed",
    )
    .bind(tenant_id)
    .bind(limit)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn validate_materialized_occurrence(
    tenant_id: Uuid,
    occurrence: &MaterializedScheduleOccurrenceV1,
) -> Result<(), ScheduleRepositoryError> {
    let snapshot = &occurrence.snapshot;
    if snapshot.validate().is_err()
        || !snapshot.rule_snapshot.enabled
        || snapshot.key.tenant_id != tenant_id
        || snapshot.key.rule_id != snapshot.rule_snapshot.rule_id
        || snapshot.key.rule_version != snapshot.rule_snapshot.rule_version
        || snapshot.rule_snapshot.tenant_id != tenant_id
        || snapshot.rule_snapshot.project_id != snapshot.rule_snapshot.target.project_id
    {
        return Err(ScheduleRepositoryError::InvalidOccurrence);
    }
    Ok(())
}

async fn persisted_rule_matches(
    tx: &mut Transaction<'_, Postgres>,
    rule: &AutomationScheduleRuleRevisionV1,
) -> Result<bool, ScheduleRepositoryError> {
    if !rule.enabled {
        return Ok(false);
    }
    let (overlap_policy, overlap_max_active) = match rule.overlap_policy {
        ScheduleOverlapPolicyV1::Skip => ("skip", 1),
        ScheduleOverlapPolicyV1::QueueOne => ("queue_one", 1),
        ScheduleOverlapPolicyV1::AllowBounded { max_active } => ("allow_bounded", max_active),
    };
    let (misfire_policy, misfire_max_occurrences) = match rule.misfire_policy {
        ScheduleMisfirePolicyV1::Skip => ("skip", 1),
        ScheduleMisfirePolicyV1::CoalesceLatest => ("coalesce_latest", 1),
        ScheduleMisfirePolicyV1::CatchUp { max_occurrences } => ("catch_up", max_occurrences),
    };
    let dst_gap = match rule.dst_gap_policy {
        ScheduleDstGapPolicyV1::Skip => "skip",
        ScheduleDstGapPolicyV1::ShiftForward => "shift_forward",
    };
    let dst_fold = match rule.dst_fold_policy {
        ScheduleDstFoldPolicyV1::EarlierInstant => "earlier_instant",
        ScheduleDstFoldPolicyV1::LaterInstant => "later_instant",
    };
    let pause = match rule.pause_policy {
        SchedulePausePolicyV1::SkipElapsed => "skip_elapsed",
        SchedulePausePolicyV1::CoalesceLatest => "coalesce_latest",
    };
    let matched = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM automation.schedule_rule_revision \
         WHERE tenant_id = $1 AND project_id = $2 AND rule_id = $3 AND rule_version = $4 \
           AND valid_to IS NULL AND enabled IS TRUE AND enabled = $5 AND cron_expression = $6 AND time_zone = $7 \
           AND parser_version = $8 AND tzdb_version = $9 AND dst_gap_policy = $10 \
           AND dst_fold_policy = $11 AND overlap_policy = $12 AND overlap_max_active = $13 \
           AND misfire_policy = $14 AND misfire_max_occurrences = $15 AND pause_policy = $16 \
           AND retry_max_attempts = $17 AND retry_initial_backoff_seconds = $18 \
           AND retry_max_backoff_seconds = $19 AND deadline_seconds = $20 \
           AND branch_id = $21 AND engineering_run_id = $22 AND repository_id = $23 \
           AND worktree_id = $24 AND work_item_id = $25 AND execution_profile_id = $26 \
           AND execution_profile_version = $27 AND execution_profile_digest = $28 \
           AND hook_set_version = $29 AND hook_set_digest = $30)",
    )
    .bind(rule.tenant_id)
    .bind(rule.project_id)
    .bind(rule.rule_id.as_uuid())
    .bind(i64::try_from(rule.rule_version).map_err(|_| ScheduleRepositoryError::InvalidInput)?)
    .bind(rule.enabled)
    .bind(&rule.cron_expression)
    .bind(&rule.time_zone)
    .bind(&rule.parser_version)
    .bind(&rule.tzdb_version)
    .bind(dst_gap)
    .bind(dst_fold)
    .bind(overlap_policy)
    .bind(i16::try_from(overlap_max_active).map_err(|_| ScheduleRepositoryError::InvalidInput)?)
    .bind(misfire_policy)
    .bind(
        i16::try_from(misfire_max_occurrences)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?,
    )
    .bind(pause)
    .bind(
        i16::try_from(rule.retry_policy.max_attempts)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?,
    )
    .bind(
        i32::try_from(rule.retry_policy.initial_backoff_seconds)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?,
    )
    .bind(
        i32::try_from(rule.retry_policy.max_backoff_seconds)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?,
    )
    .bind(i32::try_from(rule.deadline_seconds).map_err(|_| ScheduleRepositoryError::InvalidInput)?)
    .bind(rule.target.branch_id)
    .bind(rule.target.engineering_run_id)
    .bind(rule.target.repository_id)
    .bind(rule.target.worktree_id)
    .bind(rule.target.work_item_id)
    .bind(rule.target.execution_profile_id)
    .bind(
        i64::try_from(rule.target.execution_profile_version)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?,
    )
    .bind(&rule.target.execution_profile_digest)
    .bind(
        i64::try_from(rule.target.hook_set_version)
            .map_err(|_| ScheduleRepositoryError::InvalidInput)?,
    )
    .bind(&rule.target.hook_set_digest)
    .fetch_one(&mut **tx)
    .await?;
    Ok(matched)
}

fn decode_rule(row: &PgRow) -> Result<AutomationScheduleRuleRevisionV1, ScheduleRepositoryError> {
    let overlap_policy: String = row.try_get("overlap_policy")?;
    let overlap_max_active: i16 = row.try_get("overlap_max_active")?;
    let overlap_policy = match overlap_policy.as_str() {
        "skip" => ScheduleOverlapPolicyV1::Skip,
        "queue_one" => ScheduleOverlapPolicyV1::QueueOne,
        "allow_bounded" if overlap_max_active > 0 => ScheduleOverlapPolicyV1::AllowBounded {
            max_active: overlap_max_active as u16,
        },
        _ => return Err(ScheduleRepositoryError::CorruptRule),
    };
    let misfire_policy: String = row.try_get("misfire_policy")?;
    let misfire_max_occurrences: i16 = row.try_get("misfire_max_occurrences")?;
    let misfire_policy = match misfire_policy.as_str() {
        "skip" => ScheduleMisfirePolicyV1::Skip,
        "coalesce_latest" => ScheduleMisfirePolicyV1::CoalesceLatest,
        "catch_up" if misfire_max_occurrences > 0 => ScheduleMisfirePolicyV1::CatchUp {
            max_occurrences: misfire_max_occurrences as u16,
        },
        _ => return Err(ScheduleRepositoryError::CorruptRule),
    };
    let dst_gap_policy = match row.try_get::<String, _>("dst_gap_policy")?.as_str() {
        "skip" => ScheduleDstGapPolicyV1::Skip,
        "shift_forward" => ScheduleDstGapPolicyV1::ShiftForward,
        _ => return Err(ScheduleRepositoryError::CorruptRule),
    };
    let dst_fold_policy = match row.try_get::<String, _>("dst_fold_policy")?.as_str() {
        "earlier_instant" => ScheduleDstFoldPolicyV1::EarlierInstant,
        "later_instant" => ScheduleDstFoldPolicyV1::LaterInstant,
        _ => return Err(ScheduleRepositoryError::CorruptRule),
    };
    let pause_policy = match row.try_get::<String, _>("pause_policy")?.as_str() {
        "skip_elapsed" => SchedulePausePolicyV1::SkipElapsed,
        "coalesce_latest" => SchedulePausePolicyV1::CoalesceLatest,
        _ => return Err(ScheduleRepositoryError::CorruptRule),
    };

    let target = ScheduleTargetV1 {
        project_id: row.try_get("project_id")?,
        branch_id: row.try_get("branch_id")?,
        engineering_run_id: row.try_get("engineering_run_id")?,
        repository_id: row.try_get("repository_id")?,
        worktree_id: row.try_get("worktree_id")?,
        work_item_id: row.try_get("work_item_id")?,
        execution_profile_id: row.try_get("execution_profile_id")?,
        execution_profile_version: u64::try_from(
            row.try_get::<i64, _>("execution_profile_version")?,
        )
        .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
        execution_profile_digest: row.try_get("execution_profile_digest")?,
        hook_set_version: u64::try_from(row.try_get::<i64, _>("hook_set_version")?)
            .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
        hook_set_digest: row.try_get("hook_set_digest")?,
    };
    let rule = AutomationScheduleRuleRevisionV1 {
        tenant_id: row.try_get("tenant_id")?,
        rule_id: RuleId::from_uuid(row.try_get("rule_id")?),
        rule_version: u64::try_from(row.try_get::<i64, _>("rule_version")?)
            .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
        enabled: row.try_get("enabled")?,
        project_id: row.try_get("project_id")?,
        cron_expression: row.try_get("cron_expression")?,
        time_zone: row.try_get("time_zone")?,
        parser_version: row.try_get("parser_version")?,
        tzdb_version: row.try_get("tzdb_version")?,
        dst_gap_policy,
        dst_fold_policy,
        overlap_policy,
        misfire_policy,
        pause_policy,
        retry_policy: ScheduleRetryPolicyV1 {
            max_attempts: u16::try_from(row.try_get::<i16, _>("retry_max_attempts")?)
                .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
            initial_backoff_seconds: u32::try_from(
                row.try_get::<i32, _>("retry_initial_backoff_seconds")?,
            )
            .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
            max_backoff_seconds: u32::try_from(row.try_get::<i32, _>("retry_max_backoff_seconds")?)
                .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
        },
        deadline_seconds: u32::try_from(row.try_get::<i32, _>("deadline_seconds")?)
            .map_err(|_| ScheduleRepositoryError::CorruptRule)?,
        target,
    };
    rule.validate()
        .map_err(|_| ScheduleRepositoryError::CorruptRule)?;
    Ok(rule)
}
