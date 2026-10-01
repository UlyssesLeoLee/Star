//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"execution_resources.rs",type:"file",language:"rust"}),
//!   (m:Module {name:"execution_resources",type:"module",language:"rust"}),
//!   (budget:Class {name:"ResourceBudgetReservation",type:"class",language:"rust"}),
//!   (quota:Class {name:"ProjectExecutionResourceQuotaRow",type:"class",language:"rust"}),
//!   (lock:Class {name:"ProjectExecutionResourceAdmissionLock",type:"class",language:"rust"}),
//!   (usage:Class {name:"ProjectExecutionResourceReservationUsage",type:"class",language:"rust"}),
//!   (reserve:Function {name:"reserve_project_execution_resources",type:"function",language:"rust"}),
//!   (acquire_lock:Function {name:"acquire_project_execution_resource_admission_lock",type:"function",language:"rust"}),
//!   (load_quota:Function {name:"load_project_execution_resource_quota",type:"function",language:"rust"}),
//!   (load_usage:Function {name:"load_project_execution_resource_reservation_usage",type:"function",language:"rust"}),
//!   (fits:Function {name:"ResourceBudgetReservation::fits",type:"function",language:"rust"}),
//!   (from_document:Function {name:"ResourceBudgetReservation::from_document",type:"function",language:"rust"}),
//!   (checked_i64:Function {name:"checked_i64",type:"function",language:"rust"}),
//!   (map_db_error:Function {name:"map_admission_database_error",type:"function",language:"rust"}),
//!   (tests:Module {name:"execution_resources_tests",type:"module",language:"rust"}),
//!   (test_exact:Function {name:"reservation_accepts_exact_project_capacity",type:"function",language:"rust"}),
//!   (test_per_run:Function {name:"reservation_rejects_per_run_limit_excess",type:"function",language:"rust"}),
//!   (test_overflow:Function {name:"reservation_rejects_aggregate_overflow",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(budget),(m)-[:CONTAINS]->(quota),(m)-[:CONTAINS]->(lock),
//!   (m)-[:CONTAINS]->(usage),(m)-[:CONTAINS]->(reserve),(m)-[:CONTAINS]->(acquire_lock),(m)-[:CONTAINS]->(load_quota),
//!   (m)-[:CONTAINS]->(load_usage),(m)-[:CONTAINS]->(fits),(m)-[:CONTAINS]->(from_document),
//!   (m)-[:CONTAINS]->(checked_i64),(m)-[:CONTAINS]->(map_db_error),(m)-[:CONTAINS]->(tests),(tests)-[:CONTAINS]->(test_exact),
//!   (tests)-[:CONTAINS]->(test_per_run),(tests)-[:CONTAINS]->(test_overflow),
//!   (reserve)-[:CALLS]->(acquire_lock),(reserve)-[:CALLS]->(load_quota),(reserve)-[:CALLS]->(load_usage),
//!   (acquire_lock)-[:CALLS]->(map_db_error),(load_quota)-[:CALLS]->(map_db_error),
//!   (load_usage)-[:CALLS]->(map_db_error),(reserve)-[:CALLS]->(map_db_error),
//!   (reserve)-[:CALLS]->(from_document),(reserve)-[:CALLS]->(fits),
//!   (from_document)-[:CALLS]->(checked_i64),(fits)-[:USES]->(quota),(fits)-[:USES]->(usage),
//!   (test_exact)-[:CALLS]->(fits),(test_per_run)-[:CALLS]->(fits),(test_overflow)-[:CALLS]->(fits);
//! CYPHER STRUCTURAL MANIFEST END

use chrono::{DateTime, Utc};
use domain_agent::execution_profile::AgentExecutionProfileDocument;
use serde::Serialize;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::{CurrentExecutionAdmissionSnapshot, GroupApiError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResourceBudgetReservation {
    max_rss_bytes: i64,
    max_cpu_ms: i64,
    max_runtime_ms: i64,
    max_child_processes: i64,
    max_parallel_tools: i64,
    max_provider_calls: i64,
    max_output_bytes: i64,
    max_event_buffer_bytes: i64,
}

impl ResourceBudgetReservation {
    fn from_document(document: &AgentExecutionProfileDocument) -> Result<Self, GroupApiError> {
        let budget = &document.profile.resource_budget;
        Ok(Self {
            max_rss_bytes: checked_i64(budget.max_rss_bytes)?,
            max_cpu_ms: checked_i64(budget.max_cpu_ms)?,
            max_runtime_ms: checked_i64(budget.max_runtime_ms)?,
            max_child_processes: i64::from(budget.max_child_processes),
            max_parallel_tools: i64::from(budget.max_parallel_tools),
            max_provider_calls: i64::from(budget.max_provider_calls),
            max_output_bytes: checked_i64(budget.max_output_bytes)?,
            max_event_buffer_bytes: i64::from(budget.max_event_buffer_bytes),
        })
    }

    fn fits(
        self,
        quota: &ProjectExecutionResourceQuotaRow,
        usage: &ProjectExecutionResourceReservationUsage,
    ) -> bool {
        let Some(active_run_count) = usage.active_run_count.checked_add(1) else {
            return false;
        };
        active_run_count <= quota.max_active_runs
            && self.max_rss_bytes <= quota.max_rss_bytes
            && self.max_cpu_ms <= quota.max_cpu_ms_per_run
            && self.max_runtime_ms <= quota.max_runtime_ms_per_run
            && add_fits(
                usage.reserved_rss_bytes,
                self.max_rss_bytes,
                quota.max_rss_bytes,
            )
            && add_fits(
                usage.reserved_child_processes,
                self.max_child_processes,
                quota.max_child_processes,
            )
            && add_fits(
                usage.reserved_parallel_tools,
                self.max_parallel_tools,
                quota.max_parallel_tools,
            )
            && add_fits(
                usage.reserved_provider_calls,
                self.max_provider_calls,
                quota.max_provider_calls,
            )
            && add_fits(
                usage.reserved_output_bytes,
                self.max_output_bytes,
                quota.max_output_bytes,
            )
            && add_fits(
                usage.reserved_event_buffer_bytes,
                self.max_event_buffer_bytes,
                quota.max_event_buffer_bytes,
            )
    }
}

#[derive(Debug, FromRow)]
struct ProjectExecutionResourceQuotaRow {
    quota_revision_id: Uuid,
    quota_version: i64,
    lifecycle_state: String,
    max_active_runs: i64,
    max_rss_bytes: i64,
    max_cpu_ms_per_run: i64,
    max_runtime_ms_per_run: i64,
    max_child_processes: i64,
    max_parallel_tools: i64,
    max_provider_calls: i64,
    max_output_bytes: i64,
    max_event_buffer_bytes: i64,
}

#[derive(Debug, FromRow)]
struct ProjectExecutionResourceReservationUsage {
    active_run_count: i64,
    reserved_rss_bytes: i64,
    reserved_child_processes: i64,
    reserved_parallel_tools: i64,
    reserved_provider_calls: i64,
    reserved_output_bytes: i64,
    reserved_event_buffer_bytes: i64,
}

#[derive(Serialize)]
struct ReservedBudgetDetails {
    max_rss_bytes: i64,
    max_cpu_ms: i64,
    max_runtime_ms: i64,
    max_child_processes: i64,
    max_parallel_tools: i64,
    max_provider_calls: i64,
    max_output_bytes: i64,
    max_event_buffer_bytes: i64,
}

/// Reserve Project capacity across all Worktrees in the same Run admission transaction.
/// The Run insert is rolled back if the Project has no current quota or lacks capacity.
#[allow(clippy::too_many_arguments)]
pub(super) async fn reserve_project_execution_resources(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    run_id: Uuid,
    actor_id: Uuid,
    correlation_id: Uuid,
    admission_fence_id: Uuid,
    admission_fence_expires_at: DateTime<Utc>,
    snapshot: &CurrentExecutionAdmissionSnapshot,
) -> Result<(), GroupApiError> {
    if admission_fence_id.is_nil() || admission_fence_expires_at <= Utc::now() {
        return Err(GroupApiError::feature_unavailable(
            "run_admission_fence_expired",
        ));
    }

    let profile_document = snapshot.verified_profile.document();
    let budget = ResourceBudgetReservation::from_document(profile_document)?;
    acquire_project_execution_resource_admission_lock(tx, tenant_id, project_id).await?;
    let quota = load_project_execution_resource_quota(tx, tenant_id, project_id).await?;
    if quota.lifecycle_state != "active" {
        return Err(GroupApiError::feature_unavailable(
            "project_execution_resource_quota_unavailable",
        ));
    }
    let usage =
        load_project_execution_resource_reservation_usage(tx, tenant_id, project_id).await?;
    if !budget.fits(&quota, &usage) {
        return Err(GroupApiError::conflict("execution_resource_quota_exceeded"));
    }

    let reservation_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let budget_details = ReservedBudgetDetails {
        max_rss_bytes: budget.max_rss_bytes,
        max_cpu_ms: budget.max_cpu_ms,
        max_runtime_ms: budget.max_runtime_ms,
        max_child_processes: budget.max_child_processes,
        max_parallel_tools: budget.max_parallel_tools,
        max_provider_calls: budget.max_provider_calls,
        max_output_bytes: budget.max_output_bytes,
        max_event_buffer_bytes: budget.max_event_buffer_bytes,
    };
    let details = serde_json::json!({
        "reservation_id": reservation_id,
        "quota_revision_id": quota.quota_revision_id,
        "quota_version": quota.quota_version,
        "admission_fence_id": admission_fence_id,
        "reserved_maxima": budget_details,
        "resource_measurements": "not_observed_at_admission"
    });

    sqlx::query(
        r#"
        INSERT INTO multica.project_execution_resource_reservation (
            reservation_id, tenant_id, project_id, work_item_id, worktree_id, run_id,
            quota_revision_id, quota_version, admission_fence_id, reserved_by,
            correlation_id, reservation_state, lease_expires_at,
            max_rss_bytes, max_cpu_ms, max_runtime_ms, max_child_processes,
            max_parallel_tools, max_provider_calls, max_output_bytes, max_event_buffer_bytes
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending', $12,
            $13, $14, $15, $16, $17, $18, $19, $20
        )
        "#,
    )
    .bind(reservation_id)
    .bind(tenant_id)
    .bind(project_id)
    .bind(work_item_id)
    .bind(worktree_id)
    .bind(run_id)
    .bind(quota.quota_revision_id)
    .bind(quota.quota_version)
    .bind(admission_fence_id)
    .bind(actor_id)
    .bind(correlation_id)
    .bind(admission_fence_expires_at)
    .bind(budget.max_rss_bytes)
    .bind(budget.max_cpu_ms)
    .bind(budget.max_runtime_ms)
    .bind(budget.max_child_processes)
    .bind(budget.max_parallel_tools)
    .bind(budget.max_provider_calls)
    .bind(budget.max_output_bytes)
    .bind(budget.max_event_buffer_bytes)
    .execute(&mut **tx)
    .await
    .map_err(map_admission_database_error)?;

    sqlx::query(
        r#"
        INSERT INTO multica.project_execution_resource_reservation_event (
            event_id, tenant_id, project_id, reservation_id, run_id, event_type,
            actor_id, correlation_id, details
        ) VALUES ($1, $2, $3, $4, $5, 'reserved', $6, $7, $8)
        "#,
    )
    .bind(event_id)
    .bind(tenant_id)
    .bind(project_id)
    .bind(reservation_id)
    .bind(run_id)
    .bind(actor_id)
    .bind(correlation_id)
    .bind(&details)
    .execute(&mut **tx)
    .await
    .map_err(map_admission_database_error)?;

    // This event records admission-time maxima only. Runtime measurements stay NULL until a
    // trusted monitor reports observed RSS/CPU/output values.
    sqlx::query(
        r#"
        INSERT INTO multica.task_execution_run_event (
            event_id, tenant_id, project_id, run_id, work_item_id, event_type,
            actor_id, correlation_id, details
        ) VALUES ($1, $2, $3, $4, $5, 'resource_budget_reserved', $6, $7, $8)
        "#,
    )
    .bind(event_id)
    .bind(tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(work_item_id)
    .bind(actor_id)
    .bind(correlation_id)
    .bind(&details)
    .execute(&mut **tx)
    .await
    .map_err(map_admission_database_error)?;

    Ok(())
}

/// Write a short-lived Project allocation epoch before reading reservations. A row lock alone
/// does not refresh a REPEATABLE READ transaction's snapshot, so concurrent upserts intentionally
/// serialize or produce a retryable serialization conflict rather than overcommit capacity.
async fn acquire_project_execution_resource_admission_lock(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
) -> Result<(), GroupApiError> {
    sqlx::query(
        r#"
        INSERT INTO multica.project_execution_resource_admission_lock (
            tenant_id, project_id, allocation_epoch, updated_at, expires_at
        ) VALUES ($1, $2, 1, clock_timestamp(), clock_timestamp() + interval '30 days')
        ON CONFLICT (tenant_id, project_id) DO UPDATE
           SET allocation_epoch = multica.project_execution_resource_admission_lock.allocation_epoch + 1,
               updated_at = clock_timestamp(),
               expires_at = clock_timestamp() + interval '30 days'
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .execute(&mut **tx)
    .await
    .map_err(map_admission_database_error)?;
    Ok(())
}

async fn load_project_execution_resource_quota(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
) -> Result<ProjectExecutionResourceQuotaRow, GroupApiError> {
    sqlx::query_as::<_, ProjectExecutionResourceQuotaRow>(
        r#"
        SELECT quota_revision_id, quota_version, lifecycle_state, max_active_runs,
               max_rss_bytes, max_cpu_ms_per_run, max_runtime_ms_per_run,
               max_child_processes, max_parallel_tools, max_provider_calls,
               max_output_bytes, max_event_buffer_bytes
          FROM multica.project_execution_resource_quota
         WHERE tenant_id = $1 AND project_id = $2 AND valid_to IS NULL
         FOR UPDATE
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_admission_database_error)?
    .ok_or_else(|| {
        GroupApiError::feature_unavailable("project_execution_resource_quota_unavailable")
    })
}

async fn load_project_execution_resource_reservation_usage(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    project_id: Uuid,
) -> Result<ProjectExecutionResourceReservationUsage, GroupApiError> {
    sqlx::query_as::<_, ProjectExecutionResourceReservationUsage>(
        r#"
        SELECT COUNT(*)::BIGINT AS active_run_count,
               COALESCE(SUM(max_rss_bytes), 0)::BIGINT AS reserved_rss_bytes,
               COALESCE(SUM(max_child_processes), 0)::BIGINT AS reserved_child_processes,
               COALESCE(SUM(max_parallel_tools), 0)::BIGINT AS reserved_parallel_tools,
               COALESCE(SUM(max_provider_calls), 0)::BIGINT AS reserved_provider_calls,
               COALESCE(SUM(max_output_bytes), 0)::BIGINT AS reserved_output_bytes,
               COALESCE(SUM(max_event_buffer_bytes), 0)::BIGINT AS reserved_event_buffer_bytes
          FROM multica.project_execution_resource_reservation
         WHERE tenant_id = $1 AND project_id = $2
           AND (reservation_state = 'active'
                OR (reservation_state = 'pending' AND lease_expires_at > clock_timestamp()))
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_admission_database_error)
}

fn checked_i64(value: u64) -> Result<i64, GroupApiError> {
    i64::try_from(value).map_err(|_| GroupApiError::conflict("execution_resource_budget_invalid"))
}

fn map_admission_database_error(error: sqlx::Error) -> GroupApiError {
    if error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("40001")
    {
        GroupApiError::conflict("execution_resource_admission_race")
    } else {
        GroupApiError::internal()
    }
}

fn add_fits(used: i64, requested: i64, limit: i64) -> bool {
    used.checked_add(requested)
        .is_some_and(|total| total <= limit)
}

#[cfg(test)]
mod tests {
    use super::{
        add_fits, ProjectExecutionResourceQuotaRow, ProjectExecutionResourceReservationUsage,
        ResourceBudgetReservation,
    };
    use uuid::Uuid;

    fn quota() -> ProjectExecutionResourceQuotaRow {
        ProjectExecutionResourceQuotaRow {
            quota_revision_id: Uuid::new_v4(),
            quota_version: 1,
            lifecycle_state: "active".to_owned(),
            max_active_runs: 4,
            max_rss_bytes: 100,
            max_cpu_ms_per_run: 1000,
            max_runtime_ms_per_run: 1000,
            max_child_processes: 8,
            max_parallel_tools: 8,
            max_provider_calls: 64,
            max_output_bytes: 1000,
            max_event_buffer_bytes: 256,
        }
    }

    fn usage() -> ProjectExecutionResourceReservationUsage {
        ProjectExecutionResourceReservationUsage {
            active_run_count: 1,
            reserved_rss_bytes: 40,
            reserved_child_processes: 2,
            reserved_parallel_tools: 3,
            reserved_provider_calls: 10,
            reserved_output_bytes: 500,
            reserved_event_buffer_bytes: 128,
        }
    }

    fn budget() -> ResourceBudgetReservation {
        ResourceBudgetReservation {
            max_rss_bytes: 60,
            max_cpu_ms: 1000,
            max_runtime_ms: 1000,
            max_child_processes: 6,
            max_parallel_tools: 5,
            max_provider_calls: 54,
            max_output_bytes: 500,
            max_event_buffer_bytes: 128,
        }
    }

    #[test]
    fn reservation_accepts_exact_project_capacity() {
        assert!(budget().fits(&quota(), &usage()));
    }

    #[test]
    fn reservation_rejects_per_run_limit_excess() {
        let mut requested = budget();
        requested.max_cpu_ms = 1001;
        assert!(!requested.fits(&quota(), &usage()));
    }

    #[test]
    fn reservation_rejects_aggregate_overflow() {
        assert!(!add_fits(i64::MAX, 1, i64::MAX));
        let mut current = usage();
        current.reserved_rss_bytes = i64::MAX;
        assert!(!budget().fits(&quota(), &current));
    }
}
