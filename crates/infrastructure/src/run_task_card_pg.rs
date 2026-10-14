//! PostgreSQL owner adapter for the Run-scoped six-state Task Card contract.
//!
//! The adapter repeats authorization and binding checks inside each write transaction. It does
//! not publish lifecycle Outbox events: the current migration only emits metadata events.
//!
//! @cypher schema=1 source_sha256=44e1386e05a86aa79d9beca78acf7710c1802a8d86743709562de7110d82e78e
//! MERGE (self:File {path:"crates/infrastructure/src/run_task_card_pg.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::module",kind:"module"})
//! MERGE (adapter:Type {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter",kind:"struct"})
//! MERGE (scope:Type {id:"crates/infrastructure/src/run_task_card_pg.rs::RunTaskCardRunScope",kind:"struct"})
//! MERGE (current:Type {id:"crates/infrastructure/src/run_task_card_pg.rs::CurrentLifecycle",kind:"struct"})
//! MERGE (create:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter.create",kind:"method"})
//! MERGE (transition:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter.transition",kind:"method"})
//! MERGE (review:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter.review",kind:"method"})
//! MERGE (createComposed:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter.create_composed_in_transaction",kind:"method"})
//! MERGE (createComposedAudit:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter.create_composed_with_audit_in_transaction",kind:"method"})
//! MERGE (lockComposedIdempotency:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::PgRunTaskCardAdapter.lock_composed_idempotency_in_transaction",kind:"method"})
//! MERGE (createOnConnection:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::create_on_connection",kind:"function"})
//! MERGE (transitionOnConnection:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::transition_on_connection",kind:"function"})
//! MERGE (reviewOnConnection:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::review_on_connection",kind:"function"})
//! MERGE (authorize:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::authorize_run_task_scope",kind:"function"})
//! MERGE (scopeGuc:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::set_transaction_scope",kind:"function"})
//! MERGE (lockCurrent:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::lock_current_lifecycle",kind:"function"})
//! MERGE (lockIdempotency:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::lock_idempotency_key",kind:"function"})
//! MERGE (lookupIdempotency:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::lookup_idempotency",kind:"function"})
//! MERGE (saveIdempotency:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::save_idempotency",kind:"function"})
//! MERGE (insertAudit:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::insert_lifecycle_audit",kind:"function"})
//! MERGE (loadSnapshot:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::load_snapshot",kind:"function"})
//! MERGE (validateAudit:Symbol {id:"crates/infrastructure/src/run_task_card_pg.rs::validate_audit_details",kind:"function"})
//! MERGE (pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (connection:Type {id:"sqlx::PgConnection",kind:"external_type"})
//! MERGE (statementTimeout:Config {id:"postgresql.statement_timeout"})
//! MERGE (lockTimeout:Config {id:"postgresql.lock_timeout"})
//! MERGE (metadata:Table {id:"multica.task_metadata"})
//! MERGE (lifecycle:Table {id:"multica.task_lifecycle_current"})
//! MERGE (audit:Table {id:"multica.task_lifecycle_audit"})
//! MERGE (idempotency:Table {id:"multica.task_command_idempotency"})
//! MERGE (relation:Table {id:"multica.work_item_worktree"})
//! MERGE (outbox:Table {id:"multica.task_run_outbox"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(adapter)
//! MERGE (module)-[:DEFINES]->(scope)
//! MERGE (module)-[:DEFINES]->(current)
//! MERGE (module)-[:DEFINES]->(create)
//! MERGE (module)-[:DEFINES]->(transition)
//! MERGE (module)-[:DEFINES]->(review)
//! MERGE (module)-[:DEFINES]->(createComposed)
//! MERGE (module)-[:DEFINES]->(createComposedAudit)
//! MERGE (module)-[:DEFINES]->(lockComposedIdempotency)
//! MERGE (module)-[:DEFINES]->(createOnConnection)
//! MERGE (module)-[:DEFINES]->(transitionOnConnection)
//! MERGE (module)-[:DEFINES]->(reviewOnConnection)
//! MERGE (module)-[:DEFINES]->(authorize)
//! MERGE (module)-[:DEFINES]->(scopeGuc)
//! MERGE (module)-[:DEFINES]->(lockCurrent)
//! MERGE (module)-[:DEFINES]->(lockIdempotency)
//! MERGE (module)-[:DEFINES]->(lookupIdempotency)
//! MERGE (module)-[:DEFINES]->(saveIdempotency)
//! MERGE (module)-[:DEFINES]->(insertAudit)
//! MERGE (module)-[:DEFINES]->(loadSnapshot)
//! MERGE (module)-[:DEFINES]->(validateAudit)
//! MERGE (adapter)-[:USES_TYPE]->(pool)
//! MERGE (create)-[:CALLS]->(createOnConnection)
//! MERGE (transition)-[:CALLS]->(transitionOnConnection)
//! MERGE (review)-[:CALLS]->(reviewOnConnection)
//! MERGE (createComposed)-[:CALLS]->(createComposedAudit)
//! MERGE (createComposedAudit)-[:CALLS]->(validateAudit)
//! MERGE (createComposedAudit)-[:CALLS]->(createOnConnection)
//! MERGE (lockComposedIdempotency)-[:CALLS]->(lockIdempotency)
//! MERGE (createOnConnection)-[:CALLS]->(scopeGuc)
//! MERGE (createOnConnection)-[:CALLS]->(authorize)
//! MERGE (createOnConnection)-[:CALLS]->(lockIdempotency)
//! MERGE (createOnConnection)-[:CALLS]->(lookupIdempotency)
//! MERGE (createOnConnection)-[:CALLS]->(insertAudit)
//! MERGE (createOnConnection)-[:CALLS]->(loadSnapshot)
//! MERGE (createOnConnection)-[:CALLS]->(saveIdempotency)
//! MERGE (transitionOnConnection)-[:CALLS]->(scopeGuc)
//! MERGE (transitionOnConnection)-[:CALLS]->(authorize)
//! MERGE (transitionOnConnection)-[:CALLS]->(lockIdempotency)
//! MERGE (transitionOnConnection)-[:CALLS]->(lookupIdempotency)
//! MERGE (transitionOnConnection)-[:CALLS]->(lockCurrent)
//! MERGE (transitionOnConnection)-[:CALLS]->(insertAudit)
//! MERGE (transitionOnConnection)-[:CALLS]->(loadSnapshot)
//! MERGE (transitionOnConnection)-[:CALLS]->(saveIdempotency)
//! MERGE (reviewOnConnection)-[:CALLS]->(scopeGuc)
//! MERGE (reviewOnConnection)-[:CALLS]->(authorize)
//! MERGE (reviewOnConnection)-[:CALLS]->(lockIdempotency)
//! MERGE (reviewOnConnection)-[:CALLS]->(lookupIdempotency)
//! MERGE (reviewOnConnection)-[:CALLS]->(lockCurrent)
//! MERGE (reviewOnConnection)-[:CALLS]->(insertAudit)
//! MERGE (reviewOnConnection)-[:CALLS]->(loadSnapshot)
//! MERGE (reviewOnConnection)-[:CALLS]->(saveIdempotency)
//! MERGE (createOnConnection)-[:WRITES]->(metadata)
//! MERGE (createOnConnection)-[:WRITES]->(lifecycle)
//! MERGE (createOnConnection)-[:WRITES]->(relation)
//! MERGE (createOnConnection)-[:TRIGGERS]->(outbox)
//! MERGE (transitionOnConnection)-[:WRITES]->(lifecycle)
//! MERGE (reviewOnConnection)-[:WRITES]->(lifecycle)
//! MERGE (insertAudit)-[:WRITES]->(audit)
//! MERGE (saveIdempotency)-[:WRITES]->(idempotency)
//! MERGE (commandPort:Symbol {id:"crates/domain-work-item/src/run_task_card.rs::RunTaskCardCommandPort",kind:"trait"})
//! MERGE (adapter)-[:IMPLEMENTS]->(commandPort)
//! MERGE (scopeGuc)-[:CONFIGURES]->(statementTimeout)
//! MERGE (scopeGuc)-[:CONFIGURES]->(lockTimeout)
//! @endcypher

use chrono::{DateTime, Utc};
use domain_work_item::run_task_card::{
    CreateRunTaskCardCommand, ReviewRunTaskCardCommand, RunTaskCardAiData,
    RunTaskCardCommandContext, RunTaskCardCommandPort, RunTaskCardError, RunTaskCardItemType,
    RunTaskCardLifecycleSnapshot, RunTaskCardPriority, RunTaskCardReviewAction,
    RunTaskCardReviewState, RunTaskCardSnapshot, RunTaskCardStatus, RunTaskCardWriteReceipt,
    TransitionRunTaskCardCommand,
};
use domain_work_item::{RepositoryId, WorkItemId, WorktreeId};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

/// PostgreSQL adapter for canonical Run Task Card create, lifecycle, and review commands.
#[derive(Debug, Clone)]
pub struct PgRunTaskCardAdapter {
    pool: PgPool,
}

#[derive(Debug, Clone)]
struct RunTaskCardRunScope {
    workspace_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    branch_id: Uuid,
    engineering_run_id: Uuid,
}

#[derive(Debug, Clone)]
struct CurrentLifecycle {
    status: RunTaskCardStatus,
    review_state: RunTaskCardReviewState,
    active_worktree_id: Option<Uuid>,
    claimed_by: Option<Uuid>,
    claim_expired: bool,
    version: i32,
}

#[derive(Debug, Clone, Copy)]
enum RequiredTaskRole {
    Writer,
    Reviewer,
}

impl PgRunTaskCardAdapter {
    /// Construct the Run Task Card store over the application's PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a Task Card, audit facts, and durable idempotency receipt in one owner transaction.
    pub async fn create(
        &self,
        context: RunTaskCardCommandContext,
        command: CreateRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        context.validate()?;
        command.validate()?;
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        let receipt = create_on_connection(
            &mut *tx,
            &context,
            &command,
            true,
            json!({"source":"task_card"}),
        )
        .await?;
        tx.commit().await.map_err(database_error)?;
        Ok(receipt)
    }

    /// Create Task Card rows inside a caller-owned transaction for atomic Canvas composition.
    ///
    /// This participant revalidates and locks current Project/Branch/Run/Worktree bindings and
    /// roles, sets tenant/actor/correlation GUCs, and writes Task Card lifecycle audit facts. It
    /// deliberately does not read or write `task_command_idempotency`: the caller must check its
    /// composite Canvas idempotency receipt before calling and persist the final composite receipt
    /// in this same transaction. On any error the caller must roll back the whole transaction.
    pub async fn create_composed_in_transaction(
        &self,
        connection: &mut PgConnection,
        context: &RunTaskCardCommandContext,
        command: &CreateRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        self.create_composed_with_audit_in_transaction(
            connection,
            context,
            command,
            json!({"source":"task_card"}),
        )
        .await
    }

    /// Canvas/application composition variant that adds bounded owner-local audit details.
    ///
    /// `audit_details` must be a JSON object no larger than 4 KiB and must not contain secrets.
    /// The caller may include fields such as `source`, `canvas_id`, and `element_id`; no Canvas
    /// type or schema is imported into the Work Item domain.
    pub async fn create_composed_with_audit_in_transaction(
        &self,
        connection: &mut PgConnection,
        context: &RunTaskCardCommandContext,
        command: &CreateRunTaskCardCommand,
        audit_details: Value,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        context.validate()?;
        command.validate()?;
        validate_audit_details(&audit_details)?;
        create_on_connection(connection, context, command, false, audit_details).await
    }

    /// Acquire the same transaction-scoped idempotency lock used by Task Card commands.
    ///
    /// Composite owners such as Canvas call this before looking up their complete response receipt,
    /// then persist that response in the same transaction as the composed write.
    pub async fn lock_composed_idempotency_in_transaction(
        &self,
        connection: &mut PgConnection,
        context: &RunTaskCardCommandContext,
    ) -> Result<(), RunTaskCardError> {
        context.validate()?;
        lock_idempotency_key(connection, context).await
    }

    /// Apply a six-state lifecycle transition and persist its replay receipt atomically.
    pub async fn transition(
        &self,
        context: RunTaskCardCommandContext,
        command: TransitionRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        context.validate()?;
        command.validate()?;
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        let receipt = transition_on_connection(&mut *tx, &context, &command).await?;
        tx.commit().await.map_err(database_error)?;
        Ok(receipt)
    }

    /// Apply a review command and persist its replay receipt atomically.
    pub async fn review(
        &self,
        context: RunTaskCardCommandContext,
        command: ReviewRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        context.validate()?;
        command.validate()?;
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        let receipt = review_on_connection(&mut *tx, &context, &command).await?;
        tx.commit().await.map_err(database_error)?;
        Ok(receipt)
    }

    /// Caller-owned transaction participant for a lifecycle command.
    pub async fn transition_in_transaction(
        &self,
        connection: &mut PgConnection,
        context: &RunTaskCardCommandContext,
        command: &TransitionRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        context.validate()?;
        command.validate()?;
        transition_on_connection(connection, context, command).await
    }

    /// Caller-owned transaction participant for a review command.
    pub async fn review_in_transaction(
        &self,
        connection: &mut PgConnection,
        context: &RunTaskCardCommandContext,
        command: &ReviewRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        context.validate()?;
        command.validate()?;
        review_on_connection(connection, context, command).await
    }
}

#[async_trait::async_trait]
impl RunTaskCardCommandPort for PgRunTaskCardAdapter {
    async fn create(
        &self,
        context: RunTaskCardCommandContext,
        command: CreateRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        PgRunTaskCardAdapter::create(self, context, command).await
    }

    async fn transition(
        &self,
        context: RunTaskCardCommandContext,
        command: TransitionRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        PgRunTaskCardAdapter::transition(self, context, command).await
    }

    async fn review(
        &self,
        context: RunTaskCardCommandContext,
        command: ReviewRunTaskCardCommand,
    ) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
        PgRunTaskCardAdapter::review(self, context, command).await
    }
}

async fn create_on_connection(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    command: &CreateRunTaskCardCommand,
    task_idempotency: bool,
    audit_details: Value,
) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
    set_transaction_scope(connection, context).await?;
    let scope = authorize_run_task_scope(connection, context, RequiredTaskRole::Writer).await?;
    if let Some(ai_data) = &command.ai_task_data {
        if ai_data.repository_scope.len() != 1
            || ai_data.repository_scope.first() != Some(&RepositoryId(scope.repository_id))
        {
            return Err(RunTaskCardError::InvalidCommand(
                "ai_task_repository_out_of_scope",
            ));
        }
    }
    if task_idempotency {
        lock_idempotency_key(connection, context).await?;
        if let Some(receipt) = lookup_idempotency(connection, context).await? {
            return Ok(receipt);
        }
    }

    sqlx::query("SELECT set_config('app.correlation_id', $1, true)")
        .bind(context.correlation_id.to_string())
        .execute(&mut *connection)
        .await
        .map_err(database_error)?;

    let work_item_id = WorkItemId::new();
    sqlx::query(
        r#"INSERT INTO multica.task_metadata
             (work_item_id, tenant_id, workspace_id, project_id, repository_id, branch_id,
              engineering_run_id, item_type, title, description, priority, labels, ai_task_data,
              reporter_user_id)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)"#,
    )
    .bind(work_item_id.as_uuid())
    .bind(context.tenant_id.as_uuid())
    .bind(scope.workspace_id)
    .bind(scope.project_id)
    .bind(scope.repository_id)
    .bind(scope.branch_id)
    .bind(scope.engineering_run_id)
    .bind(command.item_type.as_str())
    .bind(command.title.trim())
    .bind(&command.description)
    .bind(command.priority.as_str())
    .bind(&command.labels)
    .bind(
        command
            .ai_task_data
            .as_ref()
            .map(|value| serde_json::to_value(value))
            .transpose()
            .map_err(|_| RunTaskCardError::Unavailable)?,
    )
    .bind(context.actor_id.as_uuid())
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;

    sqlx::query(
        "INSERT INTO multica.task_lifecycle_current (work_item_id, tenant_id, status, version) VALUES ($1,$2,'pending',1)",
    )
    .bind(work_item_id.as_uuid())
    .bind(context.tenant_id.as_uuid())
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;

    sqlx::query(
        "INSERT INTO multica.work_item_worktree (tenant_id,project_id,work_item_id,worktree_id) VALUES ($1,$2,$3,$4)",
    )
    .bind(context.tenant_id.as_uuid())
    .bind(scope.project_id)
    .bind(work_item_id.as_uuid())
    .bind(context.worktree_id.as_uuid())
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;

    sqlx::query(
        r#"INSERT INTO multica.task_lifecycle_audit
             (tenant_id,project_id,worktree_id,work_item_id,task_card_id,to_status,event_type,
              actor_id,idempotency_key,correlation_id,details)
           VALUES ($1,$2,$3,$4,$4,'pending','created',$5,$6,$7,$8)"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(scope.project_id)
    .bind(context.worktree_id.as_uuid())
    .bind(work_item_id.as_uuid())
    .bind(context.actor_id.as_uuid())
    .bind(&context.idempotency_key)
    .bind(context.correlation_id)
    .bind(audit_details)
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;

    let snapshot = load_snapshot(connection, context, scope.project_id, work_item_id).await?;
    if task_idempotency {
        save_idempotency(connection, context, &snapshot).await?;
    }
    Ok(RunTaskCardWriteReceipt {
        snapshot,
        replayed: false,
    })
}

fn validate_audit_details(details: &Value) -> Result<(), RunTaskCardError> {
    if !details.is_object()
        || serde_json::to_vec(details)
            .map_err(|_| RunTaskCardError::InvalidCommand("invalid_audit_details"))?
            .len()
            > 4_096
    {
        return Err(RunTaskCardError::InvalidCommand("invalid_audit_details"));
    }
    Ok(())
}

async fn transition_on_connection(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    command: &TransitionRunTaskCardCommand,
) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
    set_transaction_scope(connection, context).await?;
    let scope = authorize_run_task_scope(connection, context, RequiredTaskRole::Writer).await?;
    lock_idempotency_key(connection, context).await?;
    if let Some(receipt) = lookup_idempotency(connection, context).await? {
        return Ok(receipt);
    }
    let current = lock_current_lifecycle(connection, context, &scope, command.work_item_id).await?;
    // A retry may have waited for the lifecycle lock while its first request committed.
    if let Some(receipt) = lookup_idempotency(connection, context).await? {
        return Ok(receipt);
    }
    if current.version != command.expected_version {
        return Err(RunTaskCardError::Conflict("version_conflict"));
    }
    if matches!(
        current.status,
        RunTaskCardStatus::Claimed | RunTaskCardStatus::InProgress
    ) && current.active_worktree_id != Some(context.worktree_id.as_uuid())
    {
        return Err(RunTaskCardError::Conflict("active_worktree_conflict"));
    }
    if current.review_state == RunTaskCardReviewState::PendingReview
        && command.target_status == RunTaskCardStatus::Completed
    {
        return Err(RunTaskCardError::Conflict("review_required"));
    }
    if current.status == RunTaskCardStatus::Claimed
        && command.target_status == RunTaskCardStatus::InProgress
        && (current.claimed_by != Some(context.actor_id.as_uuid()) || current.claim_expired)
    {
        return Err(RunTaskCardError::Conflict(
            "claim_expired_or_owned_by_other_actor",
        ));
    }
    let expired_claim_requeue = current.status == RunTaskCardStatus::Claimed
        && command.target_status == RunTaskCardStatus::Pending
        && current.claim_expired;
    if !current.status.allows(command.target_status) && !expired_claim_requeue {
        return Err(RunTaskCardError::Conflict("invalid_transition"));
    }

    let update = sqlx::query(
        r#"UPDATE multica.task_lifecycle_current
           SET status=$4,
               active_worktree_id=CASE WHEN $4 IN ('claimed','in_progress') THEN $3 ELSE NULL END,
               claimed_by=CASE WHEN $4='claimed' THEN $5 WHEN $4='in_progress' THEN claimed_by ELSE NULL END,
               claim_expires_at=CASE WHEN $4='claimed' THEN now()+INTERVAL '30 seconds' ELSE NULL END,
               version=version+1,
               expires_at=CASE WHEN $4 IN ('completed','failed','cancelled') THEN now()+retention_period ELSE NULL END,
               updated_at=now()
           WHERE tenant_id=$1 AND work_item_id=$2 AND version=$6"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(command.work_item_id.as_uuid())
    .bind(context.worktree_id.as_uuid())
    .bind(command.target_status.as_str())
    .bind(context.actor_id.as_uuid())
    .bind(command.expected_version)
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;
    if update.rows_affected() != 1 {
        return Err(RunTaskCardError::Conflict("version_conflict"));
    }
    insert_lifecycle_audit(
        connection,
        context,
        scope.project_id,
        command.work_item_id,
        Some(current.status.as_str()),
        Some(command.target_status.as_str()),
        "status_transition",
        json!({"reason":command.reason}),
    )
    .await?;
    let snapshot =
        load_snapshot(connection, context, scope.project_id, command.work_item_id).await?;
    save_idempotency(connection, context, &snapshot).await?;
    Ok(RunTaskCardWriteReceipt {
        snapshot,
        replayed: false,
    })
}

async fn review_on_connection(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    command: &ReviewRunTaskCardCommand,
) -> Result<RunTaskCardWriteReceipt, RunTaskCardError> {
    set_transaction_scope(connection, context).await?;
    let required_role = if command.action == RunTaskCardReviewAction::Submit {
        RequiredTaskRole::Writer
    } else {
        RequiredTaskRole::Reviewer
    };
    let scope = authorize_run_task_scope(connection, context, required_role).await?;
    lock_idempotency_key(connection, context).await?;
    if let Some(receipt) = lookup_idempotency(connection, context).await? {
        return Ok(receipt);
    }
    let current = lock_current_lifecycle(connection, context, &scope, command.work_item_id).await?;
    if let Some(receipt) = lookup_idempotency(connection, context).await? {
        return Ok(receipt);
    }
    if current.version != command.expected_version {
        return Err(RunTaskCardError::Conflict("version_conflict"));
    }
    if current.status != RunTaskCardStatus::InProgress
        || current.active_worktree_id != Some(context.worktree_id.as_uuid())
    {
        return Err(RunTaskCardError::Conflict("review_worktree_conflict"));
    }
    let (next_status, next_review, event_type, clear_claim) = match command.action {
        RunTaskCardReviewAction::Submit
            if current.claimed_by == Some(context.actor_id.as_uuid())
                && !current.claim_expired
                && matches!(
                    current.review_state,
                    RunTaskCardReviewState::None | RunTaskCardReviewState::Rejected
                ) =>
        {
            ("in_progress", "pending_review", "review_requested", false)
        }
        RunTaskCardReviewAction::Submit => return Err(RunTaskCardError::Forbidden),
        RunTaskCardReviewAction::Accept
            if current.review_state == RunTaskCardReviewState::PendingReview
                && current.claimed_by != Some(context.actor_id.as_uuid()) =>
        {
            ("completed", "accepted", "review_accepted", true)
        }
        RunTaskCardReviewAction::Reject
            if current.review_state == RunTaskCardReviewState::PendingReview
                && current.claimed_by != Some(context.actor_id.as_uuid()) =>
        {
            ("failed", "rejected", "review_rejected", true)
        }
        RunTaskCardReviewAction::Accept | RunTaskCardReviewAction::Reject
            if current.claimed_by == Some(context.actor_id.as_uuid()) =>
        {
            return Err(RunTaskCardError::Forbidden)
        }
        _ => return Err(RunTaskCardError::Conflict("review_not_pending")),
    };
    let update = sqlx::query(
        r#"UPDATE multica.task_lifecycle_current
           SET status=$4, review_state=$5,
               active_worktree_id=CASE WHEN $6 THEN NULL ELSE active_worktree_id END,
               claimed_by=CASE WHEN $6 THEN NULL ELSE claimed_by END,
               claim_expires_at=CASE WHEN $6 THEN NULL ELSE claim_expires_at END,
               version=version+1,
               expires_at=CASE WHEN $6 THEN now()+retention_period ELSE NULL END,
               updated_at=now()
           WHERE tenant_id=$1 AND work_item_id=$2 AND version=$3"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(command.work_item_id.as_uuid())
    .bind(command.expected_version)
    .bind(next_status)
    .bind(next_review)
    .bind(clear_claim)
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;
    if update.rows_affected() != 1 {
        return Err(RunTaskCardError::Conflict("version_conflict"));
    }
    insert_lifecycle_audit(
        connection,
        context,
        scope.project_id,
        command.work_item_id,
        Some(current.status.as_str()),
        Some(next_status),
        event_type,
        json!({"review_state":next_review,"reason":command.reason}),
    )
    .await?;
    let snapshot =
        load_snapshot(connection, context, scope.project_id, command.work_item_id).await?;
    save_idempotency(connection, context, &snapshot).await?;
    Ok(RunTaskCardWriteReceipt {
        snapshot,
        replayed: false,
    })
}

async fn set_transaction_scope(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
) -> Result<(), RunTaskCardError> {
    sqlx::query(
        "SELECT set_config('app.tenant_id',$1,true),set_config('app.actor_id',$2,true),set_config('app.correlation_id',$3,true),set_config('statement_timeout','5000',true),set_config('lock_timeout','1000',true)",
    )
    .bind(context.tenant_id.to_string())
    .bind(context.actor_id.to_string())
    .bind(context.correlation_id.to_string())
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn authorize_run_task_scope(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    required_role: RequiredTaskRole,
) -> Result<RunTaskCardRunScope, RunTaskCardError> {
    let (workspace_id, project_id, worktree_repository_id): (Uuid, Uuid, Uuid) = sqlx::query_as(
        r#"SELECT w.workspace_id,p.project_id,w.repo_id
           FROM worktree_canvas_worktree w
           JOIN multica.worktree_project_binding p ON p.tenant_id=w.tenant_id AND p.worktree_id=w.id
             AND p.project_id=w.project_id AND p.valid_to IS NULL
           WHERE w.tenant_id=$1 AND w.id=$2
           FOR SHARE OF w,p"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(context.worktree_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?
    .ok_or(RunTaskCardError::NotFound)?;

    let (engineering_run_id, repository_id, branch_id): (Uuid, Uuid, Uuid) = sqlx::query_as(
        r#"SELECT engineering_run_id,repository_id,branch_id
           FROM multica.engineering_run_worktree_binding
           WHERE tenant_id=$1 AND project_id=$2 AND worktree_id=$3 AND valid_to IS NULL
           FOR SHARE"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(project_id)
    .bind(context.worktree_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?
    .ok_or(RunTaskCardError::NotFound)?;

    let (run_project_id, run_repository_id, run_branch_id): (Uuid, Uuid, Uuid) = sqlx::query_as(
        r#"SELECT r.project_id,r.repository_id,r.branch_id
           FROM multica.engineering_run r
           JOIN multica.engineering_run_revision v ON v.tenant_id=r.tenant_id AND v.engineering_run_id=r.engineering_run_id
           WHERE r.tenant_id=$1 AND r.engineering_run_id=$2
             AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state<>'archived'
           FOR SHARE OF r,v"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(engineering_run_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?
    .ok_or(RunTaskCardError::NotFound)?;
    let (branch_project_id, branch_repository_id): (Uuid, Uuid) = sqlx::query_as(
        r#"SELECT b.project_id,b.repository_id
           FROM scm.cloud_branch b
           JOIN scm.cloud_branch_revision v ON v.tenant_id=b.tenant_id AND v.branch_id=b.branch_id
           WHERE b.tenant_id=$1 AND b.branch_id=$2
             AND v.valid_from<=now() AND v.valid_to IS NULL AND v.state='active'
           FOR SHARE OF b,v"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(branch_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?
    .ok_or(RunTaskCardError::NotFound)?;
    if project_id != run_project_id
        || project_id != branch_project_id
        || repository_id != run_repository_id
        || repository_id != branch_repository_id
        || branch_id != run_branch_id
        || repository_id != worktree_repository_id
    {
        return Err(RunTaskCardError::NotFound);
    }

    let project_role: Option<String> = sqlx::query_scalar(
        r#"SELECT role FROM permission.project_role_binding
           WHERE tenant_id=$1 AND project_id=$2 AND user_id=$3 AND valid_from<=now() AND valid_to IS NULL
           FOR SHARE"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(project_id)
    .bind(context.actor_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?;
    let project_role = project_role.ok_or(RunTaskCardError::NotFound)?;
    let branch_grant: Option<Uuid> = sqlx::query_scalar(
        r#"SELECT binding_id FROM permission.cloud_branch_role_binding
           WHERE tenant_id=$1 AND branch_id=$2 AND user_id=$3 AND valid_from<=now() AND valid_to IS NULL
           FOR SHARE"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(branch_id)
    .bind(context.actor_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?;
    if branch_grant.is_none() {
        return Err(RunTaskCardError::NotFound);
    }
    let run_role: Option<String> = sqlx::query_scalar(
        r#"SELECT role FROM permission.engineering_run_role_binding
           WHERE tenant_id=$1 AND engineering_run_id=$2 AND user_id=$3
             AND valid_from<=now() AND valid_to IS NULL
           FOR SHARE"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(engineering_run_id)
    .bind(context.actor_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?;
    let run_role = run_role.ok_or(RunTaskCardError::NotFound)?;
    let allowed = match required_role {
        RequiredTaskRole::Writer => is_writer(&project_role) && is_writer(&run_role),
        RequiredTaskRole::Reviewer => is_reviewer(&project_role) && is_reviewer(&run_role),
    };
    if !allowed {
        return Err(RunTaskCardError::Forbidden);
    }
    Ok(RunTaskCardRunScope {
        workspace_id,
        project_id,
        repository_id,
        branch_id,
        engineering_run_id,
    })
}

fn is_writer(role: &str) -> bool {
    matches!(
        role,
        "tenant_admin" | "project_admin" | "developer" | "agent"
    )
}

fn is_reviewer(role: &str) -> bool {
    matches!(role, "tenant_admin" | "project_admin" | "developer")
}

async fn lock_idempotency_key(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
) -> Result<(), RunTaskCardError> {
    let lock_key = format!(
        "{}:{}:{}",
        context.tenant_id, context.actor_id, context.idempotency_key
    );
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(lock_key)
        .execute(&mut *connection)
        .await
        .map_err(database_error)?;
    Ok(())
}

async fn lookup_idempotency(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
) -> Result<Option<RunTaskCardWriteReceipt>, RunTaskCardError> {
    let previous: Option<(Vec<u8>, Value)> = sqlx::query_as(
        r#"SELECT request_hash,response_body FROM multica.task_command_idempotency
           WHERE tenant_id=$1 AND actor_id=$2 AND idempotency_key=$3 AND expires_at>now()"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(context.actor_id.as_uuid())
    .bind(&context.idempotency_key)
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?;
    match previous {
        Some((hash, body)) if hash.as_slice() == context.request_fingerprint.as_slice() => {
            let snapshot =
                serde_json::from_value(body).map_err(|_| RunTaskCardError::Unavailable)?;
            Ok(Some(RunTaskCardWriteReceipt {
                snapshot,
                replayed: true,
            }))
        }
        Some(_) => Err(RunTaskCardError::Conflict("idempotency_key_reused")),
        None => Ok(None),
    }
}

async fn save_idempotency(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    snapshot: &RunTaskCardSnapshot,
) -> Result<(), RunTaskCardError> {
    sqlx::query(
        r#"DELETE FROM multica.task_command_idempotency
           WHERE tenant_id=$1 AND actor_id=$2 AND idempotency_key=$3 AND expires_at<=now()"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(context.actor_id.as_uuid())
    .bind(&context.idempotency_key)
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;
    let body = serde_json::to_value(snapshot).map_err(|_| RunTaskCardError::Unavailable)?;
    sqlx::query(
        r#"INSERT INTO multica.task_command_idempotency
           (tenant_id,actor_id,idempotency_key,request_hash,response_body)
           VALUES ($1,$2,$3,$4,$5)"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(context.actor_id.as_uuid())
    .bind(&context.idempotency_key)
    .bind(context.request_fingerprint.as_slice())
    .bind(body)
    .execute(&mut *connection)
    .await
    .map_err(|error| match &error {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
            RunTaskCardError::Conflict("idempotency_race")
        }
        _ => database_error(error),
    })?;
    Ok(())
}

async fn lock_current_lifecycle(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    scope: &RunTaskCardRunScope,
    work_item_id: WorkItemId,
) -> Result<CurrentLifecycle, RunTaskCardError> {
    let row: Option<(String, String, Option<Uuid>, Option<Uuid>, bool, i32)> = sqlx::query_as::<
        _,
        (String, String, Option<Uuid>, Option<Uuid>, bool, i32),
    >(
        r#"SELECT c.status,c.review_state,c.active_worktree_id,c.claimed_by,
                  COALESCE(c.claim_expires_at<=now(),FALSE),c.version
           FROM multica.task_lifecycle_current c
           JOIN multica.task_metadata m ON m.tenant_id=c.tenant_id AND m.work_item_id=c.work_item_id AND m.valid_to IS NULL
           JOIN multica.work_item_worktree l ON l.tenant_id=c.tenant_id AND l.work_item_id=c.work_item_id
             AND l.project_id=m.project_id AND l.worktree_id=$3 AND l.valid_to IS NULL
           WHERE c.tenant_id=$1 AND c.work_item_id=$2 AND m.project_id=$4
             AND m.repository_id=$5 AND m.branch_id=$6 AND m.engineering_run_id=$7
           FOR UPDATE OF c"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(work_item_id.as_uuid())
    .bind(context.worktree_id.as_uuid())
    .bind(scope.project_id)
    .bind(scope.repository_id)
    .bind(scope.branch_id)
    .bind(scope.engineering_run_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?;
    let Some((status, review_state, active_worktree_id, claimed_by, claim_expired, version)) = row
    else {
        return Err(RunTaskCardError::NotFound);
    };
    Ok(CurrentLifecycle {
        status: RunTaskCardStatus::try_from(status.as_str())
            .map_err(|_| RunTaskCardError::Unavailable)?,
        review_state: parse_review_state(&review_state).ok_or(RunTaskCardError::Unavailable)?,
        active_worktree_id,
        claimed_by,
        claim_expired,
        version,
    })
}

fn parse_review_state(value: &str) -> Option<RunTaskCardReviewState> {
    match value {
        "none" => Some(RunTaskCardReviewState::None),
        "pending_review" => Some(RunTaskCardReviewState::PendingReview),
        "accepted" => Some(RunTaskCardReviewState::Accepted),
        "rejected" => Some(RunTaskCardReviewState::Rejected),
        _ => None,
    }
}

async fn insert_lifecycle_audit(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    project_id: Uuid,
    work_item_id: WorkItemId,
    from_status: Option<&str>,
    to_status: Option<&str>,
    event_type: &str,
    details: Value,
) -> Result<(), RunTaskCardError> {
    sqlx::query(
        r#"INSERT INTO multica.task_lifecycle_audit
             (tenant_id,project_id,worktree_id,work_item_id,task_card_id,from_status,to_status,
              event_type,actor_id,idempotency_key,correlation_id,details)
           VALUES ($1,$2,$3,$4,$4,$5,$6,$7,$8,$9,$10,$11)"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(project_id)
    .bind(context.worktree_id.as_uuid())
    .bind(work_item_id.as_uuid())
    .bind(from_status)
    .bind(to_status)
    .bind(event_type)
    .bind(context.actor_id.as_uuid())
    .bind(&context.idempotency_key)
    .bind(context.correlation_id)
    .bind(details)
    .execute(&mut *connection)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn load_snapshot(
    connection: &mut PgConnection,
    context: &RunTaskCardCommandContext,
    project_id: Uuid,
    work_item_id: WorkItemId,
) -> Result<RunTaskCardSnapshot, RunTaskCardError> {
    let row: Option<(
        Uuid,
        String,
        String,
        String,
        String,
        Vec<String>,
        Option<Value>,
        Uuid,
        String,
        String,
        Option<Uuid>,
        i32,
        DateTime<Utc>,
        DateTime<Utc>,
    )> = sqlx::query_as(
        r#"SELECT m.work_item_id,m.item_type,m.title,m.description,m.priority,m.labels,m.ai_task_data,
                  m.reporter_user_id,c.status,c.review_state,c.active_worktree_id,c.version,
                  c.updated_at,m.updated_at
           FROM multica.work_item_worktree l
           JOIN multica.task_metadata m ON m.tenant_id=l.tenant_id AND m.work_item_id=l.work_item_id
             AND m.project_id=l.project_id AND m.valid_to IS NULL
           JOIN multica.engineering_run_worktree_binding b ON b.tenant_id=l.tenant_id
             AND b.project_id=l.project_id AND b.worktree_id=l.worktree_id AND b.valid_to IS NULL
             AND b.repository_id=m.repository_id AND b.branch_id=m.branch_id
             AND b.engineering_run_id=m.engineering_run_id
           JOIN multica.task_lifecycle_current c ON c.tenant_id=m.tenant_id AND c.work_item_id=m.work_item_id
           WHERE l.tenant_id=$1 AND l.project_id=$2 AND l.worktree_id=$3
             AND l.work_item_id=$4 AND l.valid_to IS NULL"#,
    )
    .bind(context.tenant_id.as_uuid())
    .bind(project_id)
    .bind(context.worktree_id.as_uuid())
    .bind(work_item_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(database_error)?;
    let Some((
        id,
        item_type,
        title,
        description,
        priority,
        labels,
        ai_task_data,
        reporter,
        status,
        review_state,
        active_worktree,
        version,
        lifecycle_updated,
        metadata_updated,
    )) = row
    else {
        return Err(RunTaskCardError::NotFound);
    };
    let ai_task_data = ai_task_data
        .map(serde_json::from_value::<RunTaskCardAiData>)
        .transpose()
        .map_err(|_| RunTaskCardError::Unavailable)?;
    Ok(RunTaskCardSnapshot {
        work_item_id: WorkItemId(id),
        task_card_id: WorkItemId(id),
        item_type: RunTaskCardItemType::try_from(item_type.as_str())
            .map_err(|_| RunTaskCardError::Unavailable)?,
        title,
        description,
        priority: RunTaskCardPriority::try_from(priority.as_str())
            .map_err(|_| RunTaskCardError::Unavailable)?,
        labels,
        ai_task_data,
        reporter_user_id: domain_work_item::UserId(reporter),
        lifecycle: RunTaskCardLifecycleSnapshot {
            status: RunTaskCardStatus::try_from(status.as_str())
                .map_err(|_| RunTaskCardError::Unavailable)?,
            review_state: parse_review_state(&review_state).ok_or(RunTaskCardError::Unavailable)?,
            active_worktree_id: active_worktree.map(WorktreeId),
            version,
            updated_at: lifecycle_updated,
        },
        metadata_updated_at: metadata_updated,
    })
}

fn database_error(error: sqlx::Error) -> RunTaskCardError {
    tracing::error!(error = %error, "Run Task Card PostgreSQL operation failed");
    RunTaskCardError::Unavailable
}
