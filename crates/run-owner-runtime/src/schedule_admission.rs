//! Owner-local, bounded admission receipt/result transactions. No executor is enabled.
/*
@cypher schema=1 source_sha256=af14b4e1409e16514235303777865356bfb7d3196362b93f43c720e400c2ee7c
MERGE (self:File {path:"crates/run-owner-runtime/src/schedule_admission.rs"})
MERGE (begin:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore.begin",kind:"method"})
MERGE (commit:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.commit_result",kind:"method"})
MERGE (inbox:Table {id:"multica.schedule_admission_inbox"})
MERGE (outbox:Table {id:"multica.schedule_admission_result_outbox"})
MERGE (scope:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::set_run_owner_scope",kind:"function"})
MERGE (store:Type {id:"run_owner_runtime::PostgresScheduleAdmissionStore",kind:"struct"})
MERGE (session:Type {id:"run_owner_runtime::ScheduleAdmissionTransaction",kind:"struct"})
MERGE (decode:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionCommandV1::decode_json",kind:"method"})
MERGE (match:Symbol {id:"crates/domain-automation/src/schedule.rs::ScheduleRunAdmissionResultV1::validate_for_command",kind:"method"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(session)
MERGE (self)-[:DEFINES]->(begin)
MERGE (self)-[:DEFINES]->(commit)
MERGE (revalidate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_directory_and_binding",kind:"method"})
MERGE (self)-[:DEFINES]->(revalidate)
MERGE (revalidate)-[:CALLS]->(verify_directory:Symbol {id:"db/migrations/2026-10-10-run-owner-admission-currentness.sql::verify_schedule_run_as_current",kind:"function"})
MERGE (revalidate)-[:CALLS]->(resolve_locked:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::resolve_current_binding_locked",kind:"function"})
MERGE (revalidate)-[:CALLS]->(scope)
MERGE (task:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_task_binding",kind:"method"})
MERGE (self)-[:DEFINES]->(task)
MERGE (policy:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_profile_and_hooks",kind:"method"})
MERGE (self)-[:DEFINES]->(policy)
MERGE (catalog:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_execution_catalogs",kind:"method"})
MERGE (self)-[:DEFINES]->(catalog)
MERGE (catalog)-[:CALLS]->(policy)
MERGE (catalog)-[:CALLS]->(catalog_load:Symbol {id:"crates/run-owner-runtime/src/schedule_catalog.rs::load_current",kind:"function"})
MERGE (policy)-[:CALLS]->(task)
MERGE (policy)-[:CALLS]->(load_policy:Symbol {id:"crates/run-owner-runtime/src/schedule_policy.rs::load_current",kind:"function"})
MERGE (task)-[:CALLS]->(revalidate)
MERGE (task)-[:CALLS]->(verify_task:Symbol {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql::verify_schedule_task_current",kind:"function"})
MERGE (begin)-[:CALLS]->(scope)
MERGE (begin)-[:CALLS]->(decode)
MERGE (begin)-[:CALLS]->(match)
MERGE (commit)-[:CALLS]->(match)
MERGE (begin)-[:READS]->(inbox)
MERGE (begin)-[:READS]->(outbox)
MERGE (commit)-[:WRITES]->(inbox)
MERGE (commit)-[:WRITES]->(outbox)
MERGE (capacity:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_project_capacity",kind:"method"})
MERGE (self)-[:DEFINES]->(capacity)
MERGE (capacity)-[:CALLS]->(catalog)
MERGE (capacity)-[:CALLS]->(capacity_load:Symbol {id:"crates/run-owner-runtime/src/schedule_capacity.rs::load_current",kind:"function"})
MERGE (prepare:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.prepare_and_commit",kind:"method"})
MERGE (self)-[:DEFINES]->(prepare)
MERGE (prepare)-[:CALLS]->(capacity)
MERGE (prepare)-[:CALLS]->(commit)
MERGE (result:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.preparation_result",kind:"method"})
MERGE (time:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.validate_preparation_time",kind:"method"})
MERGE (self)-[:DEFINES]->(result)
MERGE (self)-[:DEFINES]->(time)
MERGE (prepare)-[:CALLS]->(result)
MERGE (prepare)-[:CALLS]->(time)
MERGE (prepare)-[:CALLS]->(native_prepare:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-preparation.sql::prepare_schedule_execution_run",kind:"function"})
@endcypher
*/

use domain_automation::schedule::{
    ScheduleAdmissionContractError, ScheduleAdmissionRejectionCodeV1,
    ScheduleRunAdmissionCommandV1, ScheduleRunAdmissionOutcomeV1, ScheduleRunAdmissionResultV1,
};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

/// Failures in a Run-owned admission fact transaction.
#[derive(Debug, Error)]
pub enum ScheduleAdmissionStoreError {
    /// Invalid, oversized or mismatched wire contract.
    #[error(transparent)]
    Contract(#[from] ScheduleAdmissionContractError),
    /// The independently authenticated tenant does not match the body claim.
    #[error("admission tenant does not match trusted transport scope")]
    UntrustedScope,
    /// A request identity was reused with different exact bytes or corrupt persisted facts.
    #[error("admission request replay conflicts with persisted facts")]
    ReplayConflict,
    /// Current canonical directory or binding does not authorize this fresh attempt.
    #[error("current admission prerequisite rejected: {0:?}")]
    PrerequisiteRejected(ScheduleAdmissionRejectionCodeV1),
    /// Run-owned persisted binding integrity or storage failed.
    #[error(transparent)]
    Binding(#[from] crate::RunInfrastructureBindingStoreError),
    /// Persistence failed; the owner transaction must not be acknowledged.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// The receiver must compose real service authentication before invoking this store.
/// A tenant argument is an injected scope, never proof of broker/service authentication.
#[derive(Clone)]
pub struct PostgresScheduleAdmissionStore {
    pool: PgPool,
}

/// Scope injected by a trusted receiver after independent Schedule service authentication.
/// This type is not a wire DTO and its construction is not authentication evidence.
#[derive(Debug, Clone, Copy)]
pub struct ScheduleAdmissionSource {
    /// Tenant permitted by the receiving deployment's service authorization.
    pub tenant_id: Uuid,
    /// Stable authenticated Schedule service principal, distinct from the rule's run-as actor.
    pub schedule_service_id: Uuid,
}

/// Begin either recovers the exact committed result or owns one locked new transaction.
pub enum ScheduleAdmissionStart {
    /// Stable result for exact transport replay; no new Run/reservation may be created.
    Replay(ScheduleRunAdmissionResultV1),
    /// Caller must reauthorize, resolve current policies/binding and reserve quota here.
    New(Box<ScheduleAdmissionTransaction>),
}

/// A locked Run-owner transaction; dropping it rolls back all uncommitted owner facts.
pub struct ScheduleAdmissionTransaction {
    transaction: Transaction<'static, Postgres>,
    command: ScheduleRunAdmissionCommandV1,
    wire: Vec<u8>,
    wire_sha256: [u8; 32],
    source_schedule_service_id: Uuid,
}

impl PostgresScheduleAdmissionStore {
    /// Construct a persistence store over the application's role-capable pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Serialize concurrent delivery for one tenant/request before any Run writes.
    ///
    /// The lock lives only until commit/rollback. Exact committed replay returns the
    /// persisted result, even if current policies later changed: replay is recovery
    /// of a historical admission fact, not permission to start/resume execution.
    /// A fresh logical attempt has a new request identity and requires fresh checks.
    pub async fn begin(
        &self,
        source: ScheduleAdmissionSource,
        command_wire: &[u8],
    ) -> Result<ScheduleAdmissionStart, ScheduleAdmissionStoreError> {
        let command = ScheduleRunAdmissionCommandV1::decode_json(command_wire)?;
        let trusted_tenant_id = source.tenant_id;
        if trusted_tenant_id.is_nil()
            || source.schedule_service_id.is_nil()
            || trusted_tenant_id != command.tenant_id
        {
            return Err(ScheduleAdmissionStoreError::UntrustedScope);
        }
        let wire_sha256: [u8; 32] = Sha256::digest(command_wire).into();
        let mut transaction = self.pool.begin().await?;
        crate::outbox::set_run_owner_scope(&mut transaction, trusted_tenant_id).await?;
        sqlx::query("SET LOCAL lock_timeout = '5s'")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("SET LOCAL statement_timeout = '10s'")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!(
                "schedule-admission:{trusted_tenant_id}:{}",
                command.request_id
            ))
            .execute(&mut *transaction)
            .await?;
        let row = sqlx::query(
            "SELECT i.command_wire, i.command_sha256, i.source_schedule_service_id, o.result_wire
             FROM multica.schedule_admission_inbox i
             JOIN multica.schedule_admission_result_outbox o
               ON o.tenant_id = i.tenant_id AND o.request_id = i.request_id AND o.result_id = i.result_id
             WHERE i.tenant_id = $1 AND i.request_id = $2",
        ).bind(trusted_tenant_id).bind(command.request_id)
            .fetch_optional(&mut *transaction).await?;
        if let Some(row) = row {
            let persisted_wire: Vec<u8> = row.try_get("command_wire")?;
            let persisted_digest: Vec<u8> = row.try_get("command_sha256")?;
            if persisted_wire != command_wire
                || persisted_digest != wire_sha256
                || row.try_get::<Uuid, _>("source_schedule_service_id")?
                    != source.schedule_service_id
            {
                return Err(ScheduleAdmissionStoreError::ReplayConflict);
            }
            let result_wire: Vec<u8> = row.try_get("result_wire")?;
            let result = ScheduleRunAdmissionResultV1::decode_json(&result_wire)?;
            result.validate_for_command(&command)?;
            transaction.commit().await?;
            return Ok(ScheduleAdmissionStart::Replay(result));
        }
        Ok(ScheduleAdmissionStart::New(Box::new(
            ScheduleAdmissionTransaction {
                transaction,
                command,
                wire: command_wire.to_vec(),
                wire_sha256,
                source_schedule_service_id: source.schedule_service_id,
            },
        )))
    }
}

impl ScheduleAdmissionTransaction {
    /// Atomically create a non-executing Run, Project reservation/events and admission result.
    /// Infrastructure demand must await a separate authenticated Schedule confirmation.
    /// No Runtime, Task lifecycle claim, Hook execution or external-owner write occurs here.
    pub async fn prepare_and_commit(
        mut self,
    ) -> Result<ScheduleRunAdmissionResultV1, ScheduleAdmissionStoreError> {
        if let Err(ScheduleAdmissionStoreError::PrerequisiteRejected(code)) =
            self.validate_preparation_time()
        {
            let result = self.preparation_result(None, Some(code))?;
            self.commit_result(&result).await?;
            return Ok(result);
        }
        let capacity = match self.revalidate_project_capacity().await {
            Ok(value) => value,
            Err(ScheduleAdmissionStoreError::PrerequisiteRejected(code)) => {
                let result = self.preparation_result(None, Some(code))?;
                self.commit_result(&result).await?;
                return Ok(result);
            }
            Err(error) => return Err(error),
        };
        capacity.validate_lifetime()?;
        let run_id = Uuid::new_v4();
        let profile = serde_json::to_string(capacity.execution().policies().profile().document())
            .map_err(|_| ScheduleAdmissionStoreError::ReplayConflict)?;
        let outcome: String =
            sqlx::query_scalar("SELECT multica.prepare_schedule_execution_run($1,$2,$3,$4,$5)")
                .bind(&self.wire)
                .bind(profile)
                .bind(run_id)
                .bind(Uuid::new_v4())
                .bind(Uuid::new_v4())
                .fetch_one(&mut *self.transaction)
                .await?;
        let code = match outcome.as_str() {
            "prepared" => None,
            "target_unavailable" => Some(ScheduleAdmissionRejectionCodeV1::TargetUnavailable),
            "binding_unavailable" => Some(ScheduleAdmissionRejectionCodeV1::BindingUnavailable),
            "quota_exceeded" => Some(ScheduleAdmissionRejectionCodeV1::QuotaExceeded),
            "occurrence_expired" => Some(ScheduleAdmissionRejectionCodeV1::OccurrenceExpired),
            _ => return Err(ScheduleAdmissionStoreError::ReplayConflict),
        };
        // A late fence failure drops the whole transaction, including any staged Run/reservation.
        // It must not commit a rejected result with newly staged accepted preparation facts.
        capacity.validate_lifetime()?;
        self.validate_preparation_time()?;
        let result = self.preparation_result(code.is_none().then_some(run_id), code)?;
        self.commit_result(&result).await?;
        Ok(result)
    }
    fn validate_preparation_time(&self) -> Result<(), ScheduleAdmissionStoreError> {
        let occurrence = &self.command.occurrence;
        let now = chrono::Utc::now();
        let deadline =
            occurrence
                .key
                .scheduled_for_utc
                .checked_add_signed(chrono::Duration::seconds(i64::from(
                    occurrence.rule_snapshot.deadline_seconds,
                )));
        let code = if deadline.is_none_or(|deadline| deadline <= now) {
            Some(ScheduleAdmissionRejectionCodeV1::OccurrenceExpired)
        } else if occurrence.key.scheduled_for_utc > now {
            Some(ScheduleAdmissionRejectionCodeV1::TargetUnavailable)
        } else {
            None
        };
        match code {
            Some(code) => Err(ScheduleAdmissionStoreError::PrerequisiteRejected(code)),
            None => Ok(()),
        }
    }
    fn preparation_result(
        &self,
        run_id: Option<Uuid>,
        code: Option<ScheduleAdmissionRejectionCodeV1>,
    ) -> Result<ScheduleRunAdmissionResultV1, ScheduleAdmissionStoreError> {
        let command = &self.command;
        let result = ScheduleRunAdmissionResultV1 {
            schema_version: 1,
            tenant_id: command.tenant_id,
            result_id: Uuid::new_v4(),
            request_id: command.request_id,
            admission_attempt: command.admission_attempt,
            correlation_id: command.correlation_id,
            occurrence_id: command.occurrence.occurrence_id,
            outcome: if run_id.is_some() {
                ScheduleRunAdmissionOutcomeV1::Accepted
            } else {
                ScheduleRunAdmissionOutcomeV1::Rejected
            },
            task_execution_run_id: run_id,
            rejection_code: code,
            binding_digest: command.binding_digest()?,
        };
        result.validate_for_command(command)?;
        Ok(result)
    }

    /// Serialize Project capacity against current Run ceilings in the owner transaction.
    /// This comparison creates neither a reservation nor a Run and cannot start execution.
    pub async fn revalidate_project_capacity(
        &mut self,
    ) -> Result<crate::ScheduleProjectCapacitySnapshot, ScheduleAdmissionStoreError> {
        let execution = self.revalidate_execution_catalogs().await?;
        crate::schedule_capacity::load_current(&mut self.transaction, &self.command, execution)
            .await
    }

    /// Verify current referenced Provider/Skill/Grant catalogs under the owner transaction.
    /// The short-lived result is not a Runtime, Hook evaluation or resource reservation permit.
    pub async fn revalidate_execution_catalogs(
        &mut self,
    ) -> Result<crate::ScheduleExecutionSnapshot, ScheduleAdmissionStoreError> {
        let policies = self.revalidate_profile_and_hooks().await?;
        crate::schedule_catalog::load_current(&mut self.transaction, &self.command, policies).await
    }
    /// Verify current pinned Profile/effective HookSet and their binding resource budget.
    /// Current catalogs, native Hook evaluation, quotas and execution remain separate gates.
    pub async fn revalidate_profile_and_hooks(
        &mut self,
    ) -> Result<crate::SchedulePolicySnapshot, ScheduleAdmissionStoreError> {
        self.revalidate_task_binding().await?;
        crate::schedule_policy::load_current(&mut self.transaction, &self.command).await
    }
    /// Lock the current authorized canonical task/checkout scope in this transaction.
    /// Profile, HookSet, quota and execution readiness remain separate prerequisites.
    pub async fn revalidate_task_binding(&mut self) -> Result<(), ScheduleAdmissionStoreError> {
        self.revalidate_directory_and_binding().await?;
        let command = &self.command;
        let target = &command.occurrence.rule_snapshot.target;
        let permitted: bool = sqlx::query_scalar(
            "SELECT multica.verify_schedule_task_current($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(command.tenant_id)
        .bind(command.occurrence.run_as_actor_id)
        .bind(target.project_id)
        .bind(target.branch_id)
        .bind(target.engineering_run_id)
        .bind(target.repository_id)
        .bind(target.worktree_id)
        .bind(target.work_item_id)
        .fetch_one(&mut *self.transaction)
        .await?;
        if !permitted {
            return Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
                ScheduleAdmissionRejectionCodeV1::TargetUnavailable,
            ));
        }
        Ok(())
    }
    /// Reauthorize the fixed creator and lock current directory/binding facts until commit.
    ///
    /// This is one prerequisite only. It checks current Project/Branch/Run membership,
    /// Project/Run writer roles and exact active directory scope, then the complete
    /// pinned infrastructure snapshot/digest. Task/Profile/Hook/quota/reservation,
    /// transport authentication and Runtime readiness are still separate gates.
    pub async fn revalidate_directory_and_binding(
        &mut self,
    ) -> Result<(), ScheduleAdmissionStoreError> {
        let command = &self.command;
        let target = &command.occurrence.rule_snapshot.target;
        crate::outbox::set_run_owner_scope(&mut self.transaction, command.tenant_id).await?;
        sqlx::query("SELECT set_config('app.actor_id', $1, true)")
            .bind(command.occurrence.run_as_actor_id.to_string())
            .execute(&mut *self.transaction)
            .await?;
        let permitted: bool =
            sqlx::query_scalar("SELECT multica.verify_schedule_run_as_current($1,$2,$3,$4,$5,$6)")
                .bind(command.tenant_id)
                .bind(command.occurrence.run_as_actor_id)
                .bind(target.project_id)
                .bind(target.branch_id)
                .bind(target.engineering_run_id)
                .bind(target.repository_id)
                .fetch_one(&mut *self.transaction)
                .await?;
        if !permitted {
            return Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
                ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized,
            ));
        }
        let scope = crate::RunInfrastructureBindingScope {
            tenant_id: command.tenant_id,
            project_id: target.project_id,
            branch_id: target.branch_id,
            engineering_run_id: target.engineering_run_id,
        };
        let current = match crate::run_infrastructure_binding::resolve_current_binding_locked(
            &mut self.transaction,
            scope,
        )
        .await
        {
            Ok(current) => current,
            Err(crate::RunInfrastructureBindingStoreError::NotFound) => {
                return Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
                    ScheduleAdmissionRejectionCodeV1::BindingUnavailable,
                ));
            }
            Err(error) => return Err(error.into()),
        };
        if current.snapshot != command.infrastructure_binding
            || current.digest != command.binding_digest()?
        {
            return Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
                ScheduleAdmissionRejectionCodeV1::BindingUnavailable,
            ));
        }
        Ok(())
    }
    /// The strict immutable command whose request is locked by this transaction.
    pub fn command(&self) -> &ScheduleRunAdmissionCommandV1 {
        &self.command
    }

    /// Run-owned authorization projections, quota, Run and events must use this transaction.
    /// No Schedule/Infrastructure-owned rows may be written through it.
    pub fn transaction(&mut self) -> &mut Transaction<'static, Postgres> {
        &mut self.transaction
    }

    /// Commit a receipt/result pair atomically with the caller's other Run-owned facts.
    ///
    /// This persistence primitive does not perform authorization or quota admission.
    /// SQL requires an accepted result to reference a matching non-executing Run;
    /// production consumers must add all policy/reservation checks before calling it.
    pub async fn commit_result(
        mut self,
        result: &ScheduleRunAdmissionResultV1,
    ) -> Result<(), ScheduleAdmissionStoreError> {
        result.validate_for_command(&self.command)?;
        let result_wire = result.encode_json()?;
        let command = &self.command;
        let rule = &command.occurrence.rule_snapshot;
        let target = &rule.target;
        let rule_version = i64::try_from(rule.rule_version)
            .map_err(|_| ScheduleAdmissionStoreError::ReplayConflict)?;
        sqlx::query(
            "INSERT INTO multica.schedule_admission_inbox
            (tenant_id,request_id,result_id,project_id,branch_id,engineering_run_id,
             work_item_id,worktree_id,occurrence_id,admission_attempt,correlation_id,
             run_as_actor_id,rule_id,rule_version,command_wire,command_sha256,source_schedule_service_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)",
        )
        .bind(command.tenant_id)
        .bind(command.request_id)
        .bind(result.result_id)
        .bind(target.project_id)
        .bind(target.branch_id)
        .bind(target.engineering_run_id)
        .bind(target.work_item_id)
        .bind(target.worktree_id)
        .bind(command.occurrence.occurrence_id)
        .bind(i64::from(command.admission_attempt))
        .bind(command.correlation_id)
        .bind(command.occurrence.run_as_actor_id)
        .bind(rule.rule_id.as_uuid())
        .bind(rule_version)
        .bind(&self.wire)
        .bind(self.wire_sha256.as_slice())
        .bind(self.source_schedule_service_id)
        .execute(&mut *self.transaction)
        .await?;
        let outcome = match result.outcome {
            ScheduleRunAdmissionOutcomeV1::Accepted => "accepted",
            ScheduleRunAdmissionOutcomeV1::Rejected => "rejected",
        };
        sqlx::query(
            "INSERT INTO multica.schedule_admission_result_outbox
             (tenant_id,request_id,result_id,outcome,task_execution_run_id,result_wire)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(command.tenant_id)
        .bind(command.request_id)
        .bind(result.result_id)
        .bind(outcome)
        .bind(result.task_execution_run_id)
        .bind(result_wire)
        .execute(&mut *self.transaction)
        .await?;
        self.transaction.commit().await?;
        Ok(())
    }
}
