//! Locked Project capacity comparison against current verified Run resource ceilings.
/*
@cypher schema=1 source_sha256=b199ec1f37f79dd0a1fbf9e67677a33e5562c1f45b0529daaf9a5e0d68fcaac1
MERGE (self:File {path:"crates/run-owner-runtime/src/schedule_capacity.rs"})
MERGE (load:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::load_current",kind:"function"})
MERGE (self)-[:DEFINES]->(load)
MERGE (self)-[:DEFINES]->(snapshot:Type {id:"run_owner_runtime::ScheduleProjectCapacitySnapshot",kind:"struct"})
MERGE (load)-[:CALLS]->(read:Symbol {id:"db/migrations/2026-10-10-run-owner-project-capacity.sql::read_schedule_project_capacity_current",kind:"function"})
MERGE (load)-[:CALLS]->(fits:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::CapacityEnvelope.fits",kind:"method"})
MERGE (load)-[:CALLS]->(lifetime:Symbol {id:"crates/run-owner-runtime/src/schedule_catalog.rs::ScheduleExecutionSnapshot.validate_lifetime",kind:"method"})
MERGE (self)-[:IMPORTS]->(profile:File {path:"crates/domain-agent/src/execution_profile.rs"})
MERGE (self)-[:DEFINES]->(fits)
MERGE (self)-[:DEFINES]->(validate:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::ScheduleProjectCapacitySnapshot.validate_lifetime",kind:"method"})
MERGE (validate)-[:CALLS]->(lifetime)
MERGE (exact_test:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::tests.exact_fit_and_every_ceiling",kind:"test"})
MERGE (occupied_test:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::tests.occupied_capacity_and_overflow_fail_closed",kind:"test"})
MERGE (identity_test:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::tests.missing_identity_and_disabled_capacity_reject",kind:"test"})
MERGE (self)-[:DEFINES]->(exact_test)
MERGE (self)-[:DEFINES]->(occupied_test)
MERGE (self)-[:DEFINES]->(identity_test)
MERGE (exact_test)-[:TESTS]->(fits)
MERGE (occupied_test)-[:TESTS]->(fits)
MERGE (identity_test)-[:TESTS]->(fits)
@endcypher
*/
use crate::{ScheduleAdmissionStoreError, ScheduleExecutionSnapshot};
use domain_agent::execution_profile::ResourceBudgetSnapshot;
use domain_automation::schedule::{
    ScheduleAdmissionRejectionCodeV1, ScheduleRunAdmissionCommandV1,
};
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// A capacity comparison tied to the originating owner transaction, never a reservation.
pub struct ScheduleProjectCapacitySnapshot {
    execution: ScheduleExecutionSnapshot,
    quota_revision_id: Uuid,
    quota_version: u64,
}
impl ScheduleProjectCapacitySnapshot {
    /// Verified policy and referenced execution dependencies.
    pub fn execution(&self) -> &ScheduleExecutionSnapshot {
        &self.execution
    }
    /// Current locked Project quota revision.
    pub fn quota_revision_id(&self) -> Uuid {
        self.quota_revision_id
    }
    /// Current locked Project quota version.
    pub fn quota_version(&self) -> u64 {
        self.quota_version
    }
    /// Recheck the execution fence before writes in the same live transaction.
    pub fn validate_lifetime(&self) -> Result<(), ScheduleAdmissionStoreError> {
        self.execution.validate_lifetime()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Quota {
    quota_revision_id: Uuid,
    quota_version: u64,
    max_active_runs: u64,
    max_rss_bytes: u64,
    max_cpu_ms_per_run: u64,
    max_runtime_ms_per_run: u64,
    max_child_processes: u64,
    max_parallel_tools: u64,
    max_provider_calls: u64,
    max_output_bytes: u64,
    max_event_buffer_bytes: u64,
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Usage {
    active_run_count: u64,
    reserved_rss_bytes: u64,
    reserved_child_processes: u64,
    reserved_parallel_tools: u64,
    reserved_provider_calls: u64,
    reserved_output_bytes: u64,
    reserved_event_buffer_bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapacityEnvelope {
    quota: Quota,
    usage: Usage,
}
fn add_fits(used: u64, requested: u64, limit: u64) -> bool {
    used.checked_add(requested)
        .is_some_and(|total| total <= limit)
}
impl CapacityEnvelope {
    fn fits(&self, b: &ResourceBudgetSnapshot) -> bool {
        let q = &self.quota;
        let u = &self.usage;
        !q.quota_revision_id.is_nil()
            && q.quota_version > 0
            && [
                q.max_active_runs,
                q.max_rss_bytes,
                q.max_cpu_ms_per_run,
                q.max_runtime_ms_per_run,
                q.max_child_processes,
                q.max_parallel_tools,
                q.max_provider_calls,
                q.max_output_bytes,
                q.max_event_buffer_bytes,
            ]
            .iter()
            .all(|n| *n > 0)
            && [
                b.max_rss_bytes,
                b.max_cpu_ms,
                b.max_runtime_ms,
                b.max_output_bytes,
            ]
            .iter()
            .all(|n| *n <= i64::MAX as u64)
            && add_fits(u.active_run_count, 1, q.max_active_runs)
            && b.max_cpu_ms <= q.max_cpu_ms_per_run
            && b.max_runtime_ms <= q.max_runtime_ms_per_run
            && add_fits(u.reserved_rss_bytes, b.max_rss_bytes, q.max_rss_bytes)
            && add_fits(
                u.reserved_child_processes,
                u64::from(b.max_child_processes),
                q.max_child_processes,
            )
            && add_fits(
                u.reserved_parallel_tools,
                u64::from(b.max_parallel_tools),
                q.max_parallel_tools,
            )
            && add_fits(
                u.reserved_provider_calls,
                u64::from(b.max_provider_calls),
                q.max_provider_calls,
            )
            && add_fits(
                u.reserved_output_bytes,
                b.max_output_bytes,
                q.max_output_bytes,
            )
            && add_fits(
                u.reserved_event_buffer_bytes,
                u64::from(b.max_event_buffer_bytes),
                q.max_event_buffer_bytes,
            )
    }
}
fn rejected() -> ScheduleAdmissionStoreError {
    ScheduleAdmissionStoreError::PrerequisiteRejected(
        ScheduleAdmissionRejectionCodeV1::QuotaExceeded,
    )
}
pub(crate) async fn load_current(
    tx: &mut Transaction<'_, Postgres>,
    command: &ScheduleRunAdmissionCommandV1,
    execution: ScheduleExecutionSnapshot,
) -> Result<ScheduleProjectCapacitySnapshot, ScheduleAdmissionStoreError> {
    let t = &command.occurrence.rule_snapshot.target;
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT multica.read_schedule_project_capacity_current($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(command.tenant_id)
    .bind(command.occurrence.run_as_actor_id)
    .bind(t.project_id)
    .bind(t.branch_id)
    .bind(t.engineering_run_id)
    .bind(t.repository_id)
    .bind(t.worktree_id)
    .bind(t.work_item_id)
    .fetch_one(&mut **tx)
    .await?;
    let raw = raw.ok_or_else(rejected)?;
    if raw.len() > 4096 {
        return Err(rejected());
    }
    let value: CapacityEnvelope = serde_json::from_str(&raw).map_err(|_| rejected())?;
    if !value.fits(
        &execution
            .policies()
            .profile()
            .document()
            .profile
            .resource_budget,
    ) {
        return Err(rejected());
    }
    execution.validate_lifetime()?;
    Ok(ScheduleProjectCapacitySnapshot {
        execution,
        quota_revision_id: value.quota.quota_revision_id,
        quota_version: value.quota.quota_version,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn budget() -> ResourceBudgetSnapshot {
        ResourceBudgetSnapshot {
            max_rss_bytes: 10,
            max_cpu_ms: 20,
            max_runtime_ms: 30,
            max_child_processes: 1,
            max_parallel_tools: 1,
            max_provider_calls: 2,
            max_output_bytes: 40,
            max_event_buffer_bytes: 50,
        }
    }
    fn capacity() -> CapacityEnvelope {
        CapacityEnvelope {
            quota: Quota {
                quota_revision_id: Uuid::new_v4(),
                quota_version: 1,
                max_active_runs: 1,
                max_rss_bytes: 10,
                max_cpu_ms_per_run: 20,
                max_runtime_ms_per_run: 30,
                max_child_processes: 1,
                max_parallel_tools: 1,
                max_provider_calls: 2,
                max_output_bytes: 40,
                max_event_buffer_bytes: 50,
            },
            usage: Usage::default(),
        }
    }
    #[test]
    fn exact_fit_and_every_ceiling() {
        assert!(capacity().fits(&budget()));
        for i in 0..8 {
            let mut b = budget();
            match i {
                0 => b.max_rss_bytes += 1,
                1 => b.max_cpu_ms += 1,
                2 => b.max_runtime_ms += 1,
                3 => b.max_child_processes += 1,
                4 => b.max_parallel_tools += 1,
                5 => b.max_provider_calls += 1,
                6 => b.max_output_bytes += 1,
                _ => b.max_event_buffer_bytes += 1,
            }
            assert!(!capacity().fits(&b), "dimension {i}");
        }
    }
    #[test]
    fn occupied_capacity_and_overflow_fail_closed() {
        for i in 0..7 {
            let mut c = capacity();
            match i {
                0 => c.usage.active_run_count = 1,
                1 => c.usage.reserved_rss_bytes = 1,
                2 => c.usage.reserved_child_processes = 1,
                3 => c.usage.reserved_parallel_tools = 1,
                4 => c.usage.reserved_provider_calls = 1,
                5 => c.usage.reserved_output_bytes = 1,
                _ => c.usage.reserved_event_buffer_bytes = 1,
            }
            assert!(!c.fits(&budget()), "usage dimension {i}");
        }
        assert!(!add_fits(u64::MAX, 1, u64::MAX));
        let mut b = budget();
        b.max_cpu_ms = u64::MAX;
        assert!(!capacity().fits(&b));
    }
    #[test]
    fn missing_identity_and_disabled_capacity_reject() {
        let mut c = capacity();
        c.quota.quota_revision_id = Uuid::nil();
        assert!(!c.fits(&budget()));
        let mut c = capacity();
        c.quota.quota_version = 0;
        assert!(!c.fits(&budget()));
        let mut c = capacity();
        c.quota.max_active_runs = 0;
        assert!(!c.fits(&budget()));
    }
}
