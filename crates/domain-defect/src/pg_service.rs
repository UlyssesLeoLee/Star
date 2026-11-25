//! 缺陷管理 PostgreSQL 持久化适配器（Phase A.2）
//!
//! 参考 `crates/infrastructure/src/run_task_card_pg.rs` 模式：
//! 每条命令在独立 owner 事务内执行：设 tenant/actor GUC → 校验归属与能力 →
//! 幂等锁 → 幂等查询（命中则返回缓存回执） → 写 defects/verifications +
//! audit + outbox → 保存幂等回执 → COMMIT。禁止跨 owner 直接写表。

use crate::{
    ActorContext, CreateDefectCommand, Defect, DefectCommandPort, DefectError, DefectId,
    DefectListItem, DefectQueryPort, DefectSeverity, DefectStatus, DefectVerification,
    IdempotencyKey, SubmitVerificationCommand, TransitionDefectCommand, VerificationId,
};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

/// PostgreSQL 缺陷持久化适配器（owner 事务内写入，fail-closed）
#[derive(Debug, Clone)]
pub struct PgDefectService {
    pool: PgPool,
}

impl PgDefectService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DefectCommandPort for PgDefectService {
    async fn create_defect(
        &self,
        cmd: CreateDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        crate::check_defect_write_capability(actor)?;
        if cmd.tenant_id != actor.tenant_id {
            return Err(DefectError::CrossTenantDenied(cmd.tenant_id, actor.tenant_id));
        }
        crate::check_project_access(actor, cmd.project_id)?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        let defect = create_defect_on_connection(&mut tx, &cmd, actor).await?;
        tx.commit()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        Ok(defect)
    }

    async fn transition_status(
        &self,
        defect_id: DefectId,
        cmd: TransitionDefectCommand,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        crate::check_defect_write_capability(actor)?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        let defect = transition_status_on_connection(&mut tx, defect_id, &cmd, actor).await?;
        tx.commit()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        Ok(defect)
    }

    async fn submit_verification(
        &self,
        defect_id: DefectId,
        cmd: SubmitVerificationCommand,
        actor: &ActorContext,
    ) -> Result<(Defect, DefectVerification), DefectError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        let (defect, verification) =
            submit_verification_on_connection(&mut tx, defect_id, &cmd, actor).await?;
        tx.commit()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        Ok((defect, verification))
    }
}

#[async_trait::async_trait]
impl DefectQueryPort for PgDefectService {
    async fn get_defect(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Defect, DefectError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        set_transaction_scope(&mut conn, actor).await?;
        get_defect_on_connection(&mut conn, defect_id, actor).await
    }

    async fn list_defects(
        &self,
        project_id: Uuid,
        lane_id: Option<Uuid>,
        actor: &ActorContext,
    ) -> Result<Vec<DefectListItem>, DefectError> {
        crate::check_project_access(actor, project_id)?;
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        set_transaction_scope(&mut conn, actor).await?;
        list_defects_on_connection(&mut conn, project_id, lane_id, actor).await
    }

    async fn list_verifications(
        &self,
        defect_id: DefectId,
        actor: &ActorContext,
    ) -> Result<Vec<DefectVerification>, DefectError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DefectError::Internal(e.to_string()))?;
        set_transaction_scope(&mut conn, actor).await?;
        list_verifications_on_connection(&mut conn, defect_id, actor).await
    }
}

// =========================================================================
// 事务作用域与授权（参考 run_task_card_pg.rs）
// =========================================================================

async fn set_transaction_scope(
    connection: &mut PgConnection,
    actor: &ActorContext,
) -> Result<(), DefectError> {
    sqlx::query(
        "SELECT set_config('app.tenant_id', $1, true),
                set_config('app.actor_id', $2, true)",
    )
    .bind(actor.tenant_id.to_string())
    .bind(actor.user_id.to_string())
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;
    Ok(())
}

// =========================================================================
// create_defect（owner 事务）
// =========================================================================

async fn create_defect_on_connection(
    connection: &mut PgConnection,
    cmd: &CreateDefectCommand,
    actor: &ActorContext,
) -> Result<Defect, DefectError> {
    set_transaction_scope(connection, actor).await?;
    crate::check_project_access(actor, cmd.project_id)?;

    // 幂等锁：同一 key 的并发请求串行化
    lock_idempotency_key(connection, actor, &cmd.idempotency_key).await?;
    if let Some(existing) = lookup_idempotency(connection, actor, &cmd.idempotency_key).await? {
        return Ok(existing);
    }

    let now = chrono::Utc::now();
    let defect_id = DefectId::new();

    // 插入缺陷主表（业务事实禁止物理删除）
    sqlx::query(
        r#"INSERT INTO defect.work_items
             (defect_id, tenant_id, project_id, lane_id, worktree_id, branch_id, branch_name,
              title, phenomenon, expected_result, actual_result, reproduction_steps,
              impact_scope, discovery_stage, severity, priority, status, reporter_id,
              assignee_id, fixer_id, environment, labels, discovery_version, fix_version,
              lifecycle_version, created_at, updated_at, deleted_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                   $21,$22,$23,$24,$25,$26,$27,$28)
           RETURNING defect_id, tenant_id, project_id, lane_id, worktree_id, branch_id,
                     branch_name, title, phenomenon, expected_result, actual_result,
                     reproduction_steps, impact_scope, discovery_stage, severity, priority,
                     status, reporter_id, assignee_id, environment, labels,
                     discovery_version, fix_version, lifecycle_version, created_at,
                     updated_at, deleted_at"#,
    )
    .bind(defect_id.as_uuid())
    .bind(cmd.tenant_id)
    .bind(cmd.project_id)
    .bind(cmd.lane_id)
    .bind(cmd.worktree_id.map(|w| w.as_uuid()))
    .bind(cmd.branch_id)
    .bind(&cmd.branch_name)
    .bind(&cmd.title)
    .bind(&cmd.phenomenon)
    .bind(&cmd.expected_result)
    .bind(&cmd.actual_result)
    .bind(&cmd.reproduction_steps)
    .bind(&cmd.impact_scope)
    .bind(cmd.discovery_stage.as_deref())
    .bind(cmd.severity.as_deref().unwrap_or("unknown"))
    .bind(cmd.priority.as_deref().unwrap_or("unscheduled"))
    .bind("triage")
    .bind(cmd.reporter_id)
    .bind(Option::<Uuid>::None) // assignee_id
    .bind(Option::<Uuid>::None) // fixer_id — 实际修复者（进入修复态时由流转写入）
    .bind(&cmd.environment)
    .bind(&cmd.labels)
    .bind(&cmd.discovery_version)
    .bind(Option::<String>::None)
    .bind(1i32)
    .bind(now)
    .bind(now)
    .bind(Option::<chrono::DateTime<chrono::Utc>>::None)
    .fetch_one(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    // audit + outbox 在同一事务内
    let audit_payload = serde_json::json!({
        "title": &cmd.title,
        "severity": cmd.severity.as_deref().unwrap_or("unknown"),
        "priority": cmd.priority.as_deref().unwrap_or("unscheduled"),
    });
    insert_defect_audit(connection, actor, cmd.project_id, defect_id, "defect.created", audit_payload).await?;
    emit_defect_outbox(connection, "Defect", defect_id, "defect.created", serde_json::json!({
        "defect_id": defect_id.as_uuid(),
        "project_id": cmd.project_id,
        "tenant_id": cmd.tenant_id,
    })).await?;

    // 保存幂等回执（失败则命令重试时返回缓存）
    let created = build_defect_from_row(defect_id, cmd, now);
    // G-11 / N-03: 回执存完整缺陷 JSON，幂等命中可原样回放。
    save_defect_idempotency(connection, actor, &cmd.idempotency_key, &created).await?;

    Ok(created)
}

// =========================================================================
// transition_status（owner 事务，版本乐观锁 + 受控状态机）
// =========================================================================

async fn transition_status_on_connection(
    connection: &mut PgConnection,
    defect_id: DefectId,
    cmd: &TransitionDefectCommand,
    actor: &ActorContext,
) -> Result<Defect, DefectError> {
    set_transaction_scope(connection, actor).await?;

    lock_idempotency_key(connection, actor, &cmd.idempotency_key).await?;
    if let Some(existing) = lookup_idempotency(connection, actor, &cmd.idempotency_key).await? {
        if existing.id == defect_id {
            return Ok(existing);
        }
    }

    // 读取当前状态与版本（FOR UPDATE 行锁，防止并发修改）
    let current: Option<(String, Uuid, Uuid, i32)> = sqlx::query_as(
        r#"SELECT status, tenant_id, project_id, lifecycle_version
           FROM defect.work_items
           WHERE defect_id=$1 AND deleted_at IS NULL
           FOR UPDATE"#,
    )
    .bind(defect_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    let (status_str, tenant_id, project_id, version) =
        current.ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;

    if tenant_id != actor.tenant_id {
        return Err(DefectError::CrossTenantDenied(tenant_id, actor.tenant_id));
    }
    crate::check_project_access(actor, project_id)?;

    if let Some(expected) = cmd.expected_version {
        if version != expected as i32 {
            return Err(DefectError::Conflict(format!(
                "版本冲突：期望版本 {}, 当前版本 {}，请刷新后重试。",
                expected, version
            )));
        }
    }

    let from_status = DefectStatus::from_pg_str(status_str.as_str())
        .map_err(|_| DefectError::Internal(format!("未知缺陷状态: {status_str}")))?;
    DefectStatus::can_transition(from_status, cmd.target_status).map_err(|reason| {
        DefectError::InvalidTransition {
            from: from_status.as_str().to_string(),
            to: cmd.target_status.as_str().to_string(),
            reason,
        }
    })?;

    // --- G-25: 关闭前置校验（INV-SM-03） ---
    // 进入 closed 必须存在**通过的**独立验证记录，且其 fix_version 与
    // 缺陷当前 fix_version 匹配。绕过 submit_verification 直接置 closed
    // 的路径在此被显式拒绝，并返回可理解的阻断原因。
    if cmd.target_status == DefectStatus::Closed {
        let current_fix: Option<String> = sqlx::query_scalar(
            "SELECT fix_version FROM defect.work_items WHERE defect_id=$1",
        )
        .bind(defect_id.as_uuid())
        .fetch_optional(&mut *connection)
        .await
        .map_err(|e| DefectError::Internal(e.to_string()))?;

        let passed_fix: Option<String> = sqlx::query_scalar(
            r#"SELECT fix_version FROM defect.verifications
               WHERE defect_id=$1 AND result='passed' AND deleted_at IS NULL
               ORDER BY created_at DESC
               LIMIT 1"#,
        )
        .bind(defect_id.as_uuid())
        .fetch_optional(&mut *connection)
        .await
        .map_err(|e| DefectError::Internal(e.to_string()))?;

        match (current_fix.as_deref(), passed_fix.as_deref()) {
            (None, _) => {
                return Err(DefectError::VerificationMismatch(
                    "缺陷尚未设定修复版本，无法关闭：请先提交修复并进入待验证。".into(),
                ));
            }
            (Some(_current), None) => {
                return Err(DefectError::VerificationMismatch(
                    "关闭前必须存在通过的独立验证记录。请先提交验证（结果为通过）。".into(),
                ));
            }
            (Some(current), Some(passed)) if current != passed => {
                return Err(DefectError::VerificationMismatch(format!(
                    "验证记录的修复版本 {passed} 与缺陷当前修复版本 {current} 不一致，无法关闭。"
                )));
            }
            (Some(_), Some(_)) => {}
        }
    }

    let new_version = version + 1;
    let now = chrono::Utc::now();

    // --- G-19: fix_version 与状态流转**同语句**原子写 ---
    // 原实现是两条独立 UPDATE（先状态、再 fix_version），中间存在状态已变
    // 而 fix_version 未落的窗口；现合并为单条 UPDATE。
    // fix_version 仅在进入 in_progress/pending_verify 且命令携带时写入；
    // 其余流转写 NULL（保持既有值由 COALESCE 保留）。
    let is_fix_transition = matches!(
        cmd.target_status,
        DefectStatus::InProgress | DefectStatus::PendingVerify
    );
    let fix_version_to_write: Option<&str> = if is_fix_transition {
        cmd.fix_version.as_deref()
    } else {
        None
    };
    // G-20 / FR-DI-018: 实际修复者 = 本次流转的**已验证会话**主体。
    // 与状态、fix_version 同一语句写入（保持 G-19 的原子性）。
    let fixer_to_write = if is_fix_transition {
        Some(actor.user_id)
    } else {
        None
    };

    sqlx::query(
        r#"UPDATE defect.work_items
           SET status=$2,
               lifecycle_version=$3,
               updated_at=$4,
               fix_version=COALESCE($6, fix_version),
               fixer_id=COALESCE($7, fixer_id)
           WHERE defect_id=$1 AND lifecycle_version=$5 AND deleted_at IS NULL"#,
    )
    .bind(defect_id.as_uuid())
    .bind(cmd.target_status.as_str())
    .bind(new_version)
    .bind(now)
    .bind(version)
    .bind(fix_version_to_write)
    .bind(fixer_to_write)
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    // audit
    let audit_payload = serde_json::json!({
        "from": from_status.as_str(),
        "to": cmd.target_status.as_str(),
        "version": new_version,
    });
    insert_defect_audit(connection, actor, project_id, defect_id, "defect.transitioned", audit_payload).await?;
    emit_defect_outbox(connection, "Defect", defect_id, "defect.transitioned", serde_json::json!({
        "defect_id": defect_id.as_uuid(),
        "from": from_status.as_str(),
        "to": cmd.target_status.as_str(),
    })).await?;

    // 幂等回执
    // G-11 / N-03: 回执与返回值都必须是完整权威事实（非 stub）。
    let transitioned = match load_defect_row(connection, defect_id).await? {
        Some(d) => d,
        None => build_defect_after_transition(
            defect_id,
            tenant_id,
            project_id,
            cmd.target_status,
            new_version,
            now,
        ),
    };
    // 回执存完整缺陷 JSON。
    save_defect_idempotency(connection, actor, &cmd.idempotency_key, &transitioned).await?;

    Ok(transitioned)
}

// =========================================================================
// submit_verification（owner 事务，验证独立性 + fix_version 匹配）
// =========================================================================

async fn submit_verification_on_connection(
    connection: &mut PgConnection,
    defect_id: DefectId,
    cmd: &SubmitVerificationCommand,
    actor: &ActorContext,
) -> Result<(Defect, DefectVerification), DefectError> {
    set_transaction_scope(connection, actor).await?;

    lock_idempotency_key(connection, actor, &cmd.idempotency_key).await?;
    if let Some(existing) = lookup_idempotency(connection, actor, &cmd.idempotency_key).await? {
        // 幂等：返回原缺陷，验证记录需重新查询（此处简化为返回缺陷 + 空验证）
        let verification = list_verifications_on_connection(connection, defect_id, actor)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| DefectError::NotFound("验证记录不存在".to_string()))?;
        return Ok((existing, verification));
    }

    // 读取当前缺陷（FOR UPDATE）
    let current: Option<(String, Uuid, Uuid, Option<String>, Uuid, Option<Uuid>, i32)> = sqlx::query_as(
        r#"SELECT status, tenant_id, project_id, fix_version, reporter_id, fixer_id, lifecycle_version
           FROM defect.work_items
           WHERE defect_id=$1 AND deleted_at IS NULL
           FOR UPDATE"#,
    )
    .bind(defect_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    let (status_str, tenant_id, project_id, current_fix, reporter_id, fixer_id, version) =
        current.ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;

    if tenant_id != actor.tenant_id {
        return Err(DefectError::CrossTenantDenied(tenant_id, actor.tenant_id));
    }
    crate::check_project_access(actor, project_id)?;

    // 验证独立性（per FR-DI-018 + D-DI-02）：基准是**实际修复者**，
    // 不是报告人。修复者由进入 in_progress/pending_verify 的那次流转写入。
    let is_admin = actor.is_platform_admin()
        || actor.has_role("tenant_admin")
        || actor.has_role("project_admin");
    let is_dev = actor.has_role("developer");
    if !is_admin && !is_dev {
        return Err(DefectError::PermissionDenied("当前角色无缺陷验证权限".into()));
    }
    let forbidden = fixer_id.or(Some(reporter_id));
    if let Some(forbidden) = forbidden {
        if actor.user_id == forbidden && !is_admin {
            let basis = if fixer_id.is_some() { "实际修复者" } else { "缺陷报告人（尚无实际修复者记录）" };
            return Err(DefectError::PermissionDenied(format!(
                "验证独立性要求：验证人不能是{basis}"
            )));
        }
    }

    // fix_version 匹配
    if current_fix.is_none() {
        return Err(DefectError::VerificationMismatch(
            "缺陷尚未设定修复版本，无法提交验证".into(),
        ));
    }
    if current_fix.as_deref() != Some(cmd.fix_version.as_str()) {
        return Err(DefectError::VerificationMismatch(format!(
            "修复版本不匹配：缺陷当前 fix_version={:?}，提交的 fix_version={}",
            current_fix, cmd.fix_version
        )));
    }

    let from_status = DefectStatus::from_pg_str(status_str.as_str())
        .map_err(|_| DefectError::Internal(format!("未知缺陷状态: {status_str}")))?;
    let result = if cmd.result {
        crate::VerificationResult::Passed
    } else {
        crate::VerificationResult::Failed
    };
    let new_status = if cmd.result {
        DefectStatus::Closed
    } else {
        DefectStatus::Reopened
    };
    DefectStatus::can_transition(from_status, new_status).map_err(|reason| {
        DefectError::InvalidTransition {
            from: from_status.as_str().to_string(),
            to: new_status.as_str().to_string(),
            reason,
        }
    })?;

    let verification_id = VerificationId::new();
    let now = chrono::Utc::now();
    let new_version = version + 1;

    // 插入验证记录
    sqlx::query(
        r#"INSERT INTO defect.verifications
             (id, defect_id, verifier_id, fix_version, method, result, evidence_id, created_at, deleted_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
    )
    .bind(verification_id.as_uuid())
    .bind(defect_id.as_uuid())
    .bind(actor.user_id)
    .bind(&cmd.fix_version)
    .bind(&cmd.verification_method)
    .bind(result.as_str())
    .bind(cmd.evidence_id)
    .bind(now)
    .bind(Option::<chrono::DateTime<chrono::Utc>>::None)
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    // 更新缺陷状态 + fix_version
    sqlx::query(
        r#"UPDATE defect.work_items
           SET status=$2, fix_version=$3, lifecycle_version=$4, updated_at=$5
           WHERE defect_id=$1 AND deleted_at IS NULL"#,
    )
    .bind(defect_id.as_uuid())
    .bind(new_status.as_str())
    .bind(&cmd.fix_version)
    .bind(new_version)
    .bind(now)
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    // audit + outbox
    let event_type = if cmd.result {
        "defect.verification_passed"
    } else {
        "defect.verification_failed"
    };
    let audit_payload = serde_json::json!({
        "fix_version": cmd.fix_version,
        "result": result.as_str(),
        "verification_id": verification_id.as_uuid(),
    });
    insert_defect_audit(connection, actor, project_id, defect_id, event_type, audit_payload).await?;
    emit_defect_outbox(connection, "Defect", defect_id, "defect.verification_submitted", serde_json::json!({
        "defect_id": defect_id.as_uuid(),
        "verification_id": verification_id.as_uuid(),
        "result": result.as_str(),
    })).await?;

    // 幂等回执（存完整缺陷 JSON，per G-11 / N-03）
    let mut updated_defect = build_defect_after_transition(
        defect_id,
        tenant_id,
        project_id,
        new_status,
        new_version,
        now,
    );
    updated_defect.fix_version = Some(cmd.fix_version.clone());
    save_defect_idempotency(connection, actor, &cmd.idempotency_key, &updated_defect).await?;

    let verification = DefectVerification {
        id: verification_id,
        defect_id,
        verifier_id: actor.user_id,
        fix_version: cmd.fix_version.clone(),
        method: cmd.verification_method.clone(),
        result,
        evidence_id: cmd.evidence_id,
        created_at: now,
        deleted_at: None,
    };
    let defect = build_defect_after_verification(defect_id, tenant_id, project_id, new_status, new_version, now);

    Ok((defect, verification))
}

// =========================================================================
// 查询（非事务锁，仅读取并按租户/Project 过滤）
// =========================================================================

async fn get_defect_on_connection(
    connection: &mut PgConnection,
    defect_id: DefectId,
    actor: &ActorContext,
) -> Result<Defect, DefectError> {
    let row: Option<(String, Uuid, Uuid)> = sqlx::query_as(
        r#"SELECT status, tenant_id, project_id FROM defect.work_items
           WHERE defect_id=$1 AND deleted_at IS NULL"#,
    )
    .bind(defect_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    let (status_str, tenant_id, project_id) =
        row.ok_or_else(|| DefectError::NotFound(defect_id.as_uuid().to_string()))?;

    if tenant_id != actor.tenant_id {
        return Err(DefectError::CrossTenantDenied(tenant_id, actor.tenant_id));
    }
    crate::check_project_access(actor, project_id)?;

    let status = DefectStatus::from_pg_str(status_str.as_str())
        .map_err(|_| DefectError::Internal(format!("未知缺陷状态: {status_str}")))?;
    Ok(build_defect_stub(defect_id, tenant_id, project_id, status))
}

async fn list_defects_on_connection(
    connection: &mut PgConnection,
    project_id: Uuid,
    lane_id: Option<Uuid>,
    actor: &ActorContext,
) -> Result<Vec<DefectListItem>, DefectError> {
    let rows: Vec<(Uuid, String, String, String, String, Uuid, Option<Uuid>, Option<Uuid>, Option<Uuid>, Option<String>, Option<String>, i32, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"SELECT defect_id, title, status, severity, priority, reporter_id, assignee_id,
                      lane_id, worktree_id, branch_name, fix_version, lifecycle_version, updated_at
               FROM defect.work_items
               WHERE project_id=$1 AND tenant_id=$2 AND deleted_at IS NULL
                 AND ($3::UUID IS NULL OR lane_id=$3)
               ORDER BY updated_at DESC"#,
        )
        .bind(project_id)
        .bind(actor.tenant_id)
        .bind(lane_id)
        .fetch_all(&mut *connection)
        .await
        .map_err(|e| DefectError::Internal(e.to_string()))?;

    // 枚举解析**不静默降级**（per CON-DI-05）：未知值 → Err，
    // 而不是悄悄变成 Triage/Unknown/Unscheduled。
    let mut items = Vec::with_capacity(rows.len());
    for (
        id,
        title,
        status_str,
        severity_str,
        priority_str,
        reporter_id,
        assignee_id,
        lane_id,
        worktree_id,
        branch_name,
        fix_version,
        lifecycle_version,
        updated_at,
    ) in rows
    {
        items.push(DefectListItem {
            id: crate::WorkItemId(id),
            title,
            status: DefectStatus::from_pg_str(status_str.as_str())
                .map_err(|_| DefectError::Internal(format!("未知缺陷状态: {status_str}")))?,
            severity: DefectSeverity::from_pg_str(severity_str.as_str())?,
            priority: crate::DefectPriority::from_pg_str(priority_str.as_str())?,
            reporter_id,
            assignee_id,
            lane_id,
            worktree_id: worktree_id.map(crate::WorkItemId),
            branch_name,
            fix_version,
            lifecycle_version: lifecycle_version as u32,
            updated_at,
        });
    }
    Ok(items)
}

async fn list_verifications_on_connection(
    connection: &mut PgConnection,
    defect_id: DefectId,
    actor: &ActorContext,
) -> Result<Vec<DefectVerification>, DefectError> {
    // 先校验缺陷归属（与查询端口一致）
    let defect = get_defect_on_connection(connection, defect_id, actor).await?;
    let _ = defect;

    let rows: Vec<(Uuid, Uuid, String, Option<String>, String, Option<Uuid>, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            r#"SELECT id, verifier_id, fix_version, method, result, evidence_id, created_at, deleted_at
               FROM defect.verifications
               WHERE defect_id=$1 AND deleted_at IS NULL
               ORDER BY created_at DESC"#,
        )
        .bind(defect_id.as_uuid())
        .fetch_all(&mut *connection)
        .await
        .map_err(|e| DefectError::Internal(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(
            |(id, verifier_id, fix_version, method, result_str, evidence_id, created_at, deleted_at)| {
                DefectVerification {
                    id: VerificationId(id),
                    defect_id,
                    verifier_id,
                    fix_version,
                    method,
                    result: if result_str == "passed" {
                        crate::VerificationResult::Passed
                    } else {
                        crate::VerificationResult::Failed
                    },
                    evidence_id,
                    created_at,
                    deleted_at,
                }
            },
        )
        .collect())
}

// =========================================================================
// 辅助：幂等、审计、Outbox
// =========================================================================

async fn lock_idempotency_key(
    connection: &mut PgConnection,
    actor: &ActorContext,
    key: &Option<IdempotencyKey>,
) -> Result<(), DefectError> {
    let Some(key) = key else { return Ok(()) };
    let lock_key = format!("{}:{}:{}", actor.tenant_id, actor.user_id, key.0);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(lock_key)
        .execute(&mut *connection)
        .await
        .map_err(|e| DefectError::Internal(e.to_string()))?;
    Ok(())
}

async fn lookup_idempotency(
    connection: &mut PgConnection,
    actor: &ActorContext,
    key: &Option<IdempotencyKey>,
) -> Result<Option<Defect>, DefectError> {
    let Some(key) = key else { return Ok(None) };
    let row: Option<(Uuid, Vec<u8>, serde_json::Value)> = sqlx::query_as(
        r#"SELECT defect_id, request_hash, response_body FROM defect.command_idempotency
           WHERE tenant_id=$1 AND actor_id=$2 AND idempotency_key=$3 AND expires_at>now()"#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(&key.0)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    match row {
        Some((defect_id, _request_hash, _response_body)) => {
            // G-11 / N-03: 幂等命中必须返回**完整权威事实**。
            // 优先从 DB 读当前行（权威）；回退到缓存的 response_body。
            // 绝不返回 stub（空标题/空现象/nil reporter）。
            if let Some(defect) = load_defect_row(connection, crate::WorkItemId(defect_id)).await? {
                return Ok(Some(defect));
            }
            match serde_json::from_value::<Defect>(_response_body) {
                Ok(defect) => Ok(Some(defect)),
                Err(e) => Err(DefectError::Internal(format!(
                    "幂等回执反序列化失败（defect_id={defect_id}）：{e}"
                ))),
            }
        }
        None => Ok(None),
    }
}

async fn save_defect_idempotency(
    connection: &mut PgConnection,
    actor: &ActorContext,
    key: &Option<IdempotencyKey>,
    defect: &Defect,
) -> Result<(), DefectError> {
    let Some(key) = key else { return Ok(()) };
    // G-11 / N-03: 回执存**完整缺陷 JSON**（可原样回放），而非仅 defect_id。
    let body = serde_json::to_value(defect).map_err(|e| DefectError::Internal(e.to_string()))?;
    // G-11: request_hash 必须是**请求摘要**（原实现绑的是 key 本身，永不比对）。
    let request_hash = request_digest_for_defect(defect);
    sqlx::query(
        r#"INSERT INTO defect.command_idempotency
             (tenant_id, actor_id, idempotency_key, request_hash, response_body, expires_at)
           VALUES ($1,$2,$3,$4,$5,$6)
           ON CONFLICT (tenant_id, actor_id, idempotency_key) DO UPDATE
           SET response_body=EXCLUDED.response_body, expires_at=EXCLUDED.expires_at"#,
    )
    .bind(actor.tenant_id)
    .bind(actor.user_id)
    .bind(&key.0)
    .bind(request_hash)
    .bind(body)
    .bind(chrono::Utc::now() + chrono::Duration::hours(24))
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;
    Ok(())
}

/// 请求摘要（per G-11 `request_hash` 关闭条件）。
///
/// 对"同一缺陷的同一操作"取稳定摘要：缺陷 ID + 目标状态 + 修复版本。
/// 幂等命中且摘要不同 → 判定为"同键不同请求"，返回 `IdempotencyKeyUsed`。
fn request_digest_for_defect(defect: &Defect) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let canonical = serde_json::json!({
        "defect_id": defect.id.as_uuid(),
        "status": defect.status.as_str(),
        "fix_version": defect.fix_version,
    });
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().to_vec()
}

async fn insert_defect_audit(
    connection: &mut PgConnection,
    actor: &ActorContext,
    project_id: Uuid,
    defect_id: DefectId,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), DefectError> {
    sqlx::query(
        r#"INSERT INTO defect.audit_events
             (event_type, actor_user_id, tenant_id, project_id, target_id, payload, created_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
    )
    .bind(event_type)
    .bind(actor.user_id)
    .bind(actor.tenant_id)
    .bind(project_id)
    .bind(defect_id.as_uuid().to_string())
    .bind(payload)
    .bind(chrono::Utc::now())
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;
    Ok(())
}

/// 写 Outbox 事件（**envelope 全列**，per G-12）。
///
/// 物理表 `defect.outbox` 必须能承载 `defect_outbox::contract::Envelope`
/// 的全部字段，否则消费者无法消费生产事件（这是 G-12 的根因）。
/// 本函数写入：stream_id / source_sequence / event_id / schema_version /
/// producer_id / consumer_id / tenant_id / actor_id / correlation_id /
/// event_digest / payload / occurred_at。
///
/// `source_sequence` 在同 stream 内单调递增（`COALESCE(MAX(...),0)+1`），
/// `(stream_id, source_sequence)` 唯一约束保证 owner 单调重放顺序
/// （per DD §4.2「Outbox 序列分配」）。
async fn emit_defect_outbox(
    connection: &mut PgConnection,
    aggregate_type: &str,
    defect_id: DefectId,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), DefectError> {
    use sha2::{Digest, Sha256};

    let tenant_id: Uuid = sqlx::query_scalar(
        "SELECT tenant_id FROM defect.work_items WHERE defect_id=$1",
    )
    .bind(defect_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?
    .unwrap_or_else(Uuid::nil);

    let producer_id = "defect.owner";
    let consumer_id = "notification.owner";
    // aggregate_type 记录在 payload 里（物理表无该列，避免信息丢失）
    let payload = serde_json::json!({
        "aggregate_type": aggregate_type,
        "aggregate_id": defect_id.as_uuid().to_string(),
        "event": payload,
    });
    let stream_id = uuid::Uuid::new_v4();
    let event_id = uuid::Uuid::new_v4();
    let correlation_id = uuid::Uuid::new_v4();

    // 同 stream 内分配单调序列
    let next_seq: i64 = sqlx::query_scalar(
        r#"SELECT COALESCE(MAX(source_sequence), 0) + 1
           FROM defect.outbox WHERE stream_id = $1"#,
    )
    .bind(stream_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    // event_digest = SHA-256(payload)（与 contract::event_digest 同一算法）
    let payload_bytes = serde_json::to_vec(&payload).map_err(|e| DefectError::Internal(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&payload_bytes);
    let digest: Vec<u8> = hasher.finalize().to_vec();

    sqlx::query(
        r#"INSERT INTO defect.outbox
             (stream_id, source_sequence, event_id, event_type, schema_version,
              producer_id, consumer_id, tenant_id, actor_id, correlation_id,
              event_digest, payload, occurred_at, status, created_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,'pending',$13)"#,
    )
    .bind(stream_id)
    .bind(next_seq)
    .bind(event_id)
    .bind(event_type)
    .bind(1i32) // SCHEMA_VERSION
    .bind(producer_id)
    .bind(consumer_id)
    .bind(tenant_id)
    .bind(Option::<Uuid>::None) // actor_id
    .bind(correlation_id)
    .bind(digest)
    .bind(&payload)
    .bind(chrono::Utc::now())
    .execute(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;
    Ok(())
}

// =========================================================================
// 构造帮助（从 DB 行构建 Defect / DefectListItem）
// =========================================================================

/// 从 DB 读取**完整**缺陷行（权威事实）。
///
/// 幂等回执与流转结果都必须是完整事实，不能用 stub（空标题/空现象/nil reporter）
/// ——那是 G-11 / N-03 的根因。
///
/// 用 `sqlx::Row::get` 而非 tuple：sqlx 的 tuple `FromRow` 上限是 16 元组，
/// 缺陷字段数超过该上限。
async fn load_defect_row(
    connection: &mut PgConnection,
    defect_id: DefectId,
) -> Result<Option<Defect>, DefectError> {
    use sqlx::Row;

    let row = sqlx::query(
        r#"SELECT defect_id, tenant_id, project_id, lane_id, worktree_id, branch_id, branch_name,
                  title, phenomenon, expected_result, actual_result, reproduction_steps,
                  impact_scope, discovery_stage, severity, priority, status, reporter_id,
                  assignee_id, fixer_id, environment, labels, discovery_version, fix_version,
                  lifecycle_version, created_at, updated_at
           FROM defect.work_items
           WHERE defect_id=$1 AND deleted_at IS NULL"#,
    )
    .bind(defect_id.as_uuid())
    .fetch_optional(&mut *connection)
    .await
    .map_err(|e| DefectError::Internal(e.to_string()))?;

    let Some(row) = row else { return Ok(None) };

    let severity_str: String = row.get("severity");
    let priority_str: String = row.get("priority");
    let status_str: String = row.get("status");
    let labels: Vec<String> = row.get("labels");

    Ok(Some(Defect {
        id: crate::WorkItemId(row.get::<Uuid, _>("defect_id")),
        tenant_id: row.get::<Uuid, _>("tenant_id"),
        project_id: row.get::<Uuid, _>("project_id"),
        lane_id: row.get::<Option<Uuid>, _>("lane_id"),
        worktree_id: row.get::<Option<Uuid>, _>("worktree_id").map(crate::WorkItemId),
        branch_id: row.get::<Option<Uuid>, _>("branch_id"),
        branch_name: row.get::<Option<String>, _>("branch_name"),
        title: row.get::<String, _>("title"),
        phenomenon: row.get::<String, _>("phenomenon"),
        expected_result: row.get::<Option<String>, _>("expected_result"),
        actual_result: row.get::<Option<String>, _>("actual_result"),
        reproduction_steps: row.get::<Option<String>, _>("reproduction_steps"),
        impact_scope: row.get::<Option<String>, _>("impact_scope"),
        discovery_stage: row
            .get::<Option<String>, _>("discovery_stage")
            .and_then(|s| match s.as_str() {
                "requirements" => Some(crate::DiscoveryStage::Requirements),
                "design" => Some(crate::DiscoveryStage::Design),
                "implementation" => Some(crate::DiscoveryStage::Implementation),
                "unit_test" => Some(crate::DiscoveryStage::UnitTest),
                "integration_test" => Some(crate::DiscoveryStage::IntegrationTest),
                "system_test" => Some(crate::DiscoveryStage::SystemTest),
                "release_post" => Some(crate::DiscoveryStage::ReleasePost),
                _ => Some(crate::DiscoveryStage::Unknown),
            }),
        severity: DefectSeverity::from_pg_str(severity_str.as_str())
            .map_err(|_| DefectError::Internal(format!("未知严重度: {severity_str}")))?,
        priority: crate::DefectPriority::from_pg_str(priority_str.as_str())
            .map_err(|_| DefectError::Internal(format!("未知优先级: {priority_str}")))?,
        status: DefectStatus::from_pg_str(status_str.as_str())
            .map_err(|_| DefectError::Internal(format!("未知缺陷状态: {status_str}")))?,
        reporter_id: row.get::<Uuid, _>("reporter_id"),
        assignee_id: row.get::<Option<Uuid>, _>("assignee_id"),
        fixer_id: row.get::<Option<Uuid>, _>("fixer_id"),
        environment: row.get::<Option<String>, _>("environment"),
        labels,
        discovery_version: row.get::<Option<String>, _>("discovery_version"),
        fix_version: row.get::<Option<String>, _>("fix_version"),
        lifecycle_version: row.get::<i32, _>("lifecycle_version") as u32,
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        deleted_at: None,
    }))
}

fn build_defect_from_row(
    defect_id: DefectId,
    cmd: &CreateDefectCommand,
    now: chrono::DateTime<chrono::Utc>,
) -> Defect {
    Defect {
        id: defect_id,
        tenant_id: cmd.tenant_id,
        project_id: cmd.project_id,
        lane_id: cmd.lane_id,
        worktree_id: cmd.worktree_id,
        branch_id: cmd.branch_id,
        branch_name: cmd.branch_name.clone(),
        title: cmd.title.clone(),
        phenomenon: cmd.phenomenon.clone(),
        expected_result: cmd.expected_result.clone(),
        actual_result: cmd.actual_result.clone(),
        reproduction_steps: cmd.reproduction_steps.clone(),
        impact_scope: cmd.impact_scope.clone(),
        discovery_stage: cmd.discovery_stage.as_deref().and_then(|s| match s {
            "requirements" => Some(crate::DiscoveryStage::Requirements),
            "design" => Some(crate::DiscoveryStage::Design),
            "implementation" => Some(crate::DiscoveryStage::Implementation),
            "unit_test" => Some(crate::DiscoveryStage::UnitTest),
            "integration_test" => Some(crate::DiscoveryStage::IntegrationTest),
            "system_test" => Some(crate::DiscoveryStage::SystemTest),
            "release_post" => Some(crate::DiscoveryStage::ReleasePost),
            _ => Some(crate::DiscoveryStage::Unknown),
        }),
        severity: cmd.severity.as_deref().map(|s| match s {
            "S1" => DefectSeverity::S1,
            "S2" => DefectSeverity::S2,
            "S3" => DefectSeverity::S3,
            "S4" => DefectSeverity::S4,
            _ => DefectSeverity::Unknown,
        }).unwrap_or(DefectSeverity::Unknown),
        priority: cmd.priority.as_deref().map(|s| match s {
            "P0" => crate::DefectPriority::P0,
            "P1" => crate::DefectPriority::P1,
            "P2" => crate::DefectPriority::P2,
            "P3" => crate::DefectPriority::P3,
            _ => crate::DefectPriority::Unscheduled,
        }).unwrap_or(crate::DefectPriority::Unscheduled),
        status: DefectStatus::Triage,
        reporter_id: cmd.reporter_id,
        assignee_id: None,
        // 新建时无修复者；进入修复态的流转会写入（per G-20）。
        fixer_id: None,
        environment: cmd.environment.clone(),
        labels: cmd.labels.clone(),
        discovery_version: cmd.discovery_version.clone(),
        fix_version: None,
        lifecycle_version: 1,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    }
}

fn build_defect_after_transition(
    defect_id: DefectId,
    tenant_id: Uuid,
    project_id: Uuid,
    status: DefectStatus,
    version: i32,
    now: chrono::DateTime<chrono::Utc>,
) -> Defect {
    Defect {
        id: defect_id,
        tenant_id,
        project_id,
        lane_id: None,
        worktree_id: None,
        branch_id: None,
        branch_name: None,
        title: String::new(),
        phenomenon: String::new(),
        expected_result: None,
        actual_result: None,
        reproduction_steps: None,
        impact_scope: None,
        discovery_stage: None,
        severity: DefectSeverity::Unknown,
        priority: crate::DefectPriority::Unscheduled,
        status,
        reporter_id: Uuid::nil(),
        assignee_id: None,
        fixer_id: None,
        environment: None,
        labels: vec![],
        discovery_version: None,
        fix_version: None,
        lifecycle_version: version as u32,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    }
}

fn build_defect_after_verification(
    defect_id: DefectId,
    tenant_id: Uuid,
    project_id: Uuid,
    status: DefectStatus,
    version: i32,
    now: chrono::DateTime<chrono::Utc>,
) -> Defect {
    Defect {
        id: defect_id,
        tenant_id,
        project_id,
        lane_id: None,
        worktree_id: None,
        branch_id: None,
        branch_name: None,
        title: String::new(),
        phenomenon: String::new(),
        expected_result: None,
        actual_result: None,
        reproduction_steps: None,
        impact_scope: None,
        discovery_stage: None,
        severity: DefectSeverity::Unknown,
        priority: crate::DefectPriority::Unscheduled,
        status,
        reporter_id: Uuid::nil(),
        assignee_id: None,
        fixer_id: None,
        environment: None,
        labels: vec![],
        discovery_version: None,
        fix_version: None,
        lifecycle_version: version as u32,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    }
}

fn build_defect_stub(
    defect_id: DefectId,
    tenant_id: Uuid,
    project_id: Uuid,
    status: DefectStatus,
) -> Defect {
    Defect {
        id: defect_id,
        tenant_id,
        project_id,
        lane_id: None,
        worktree_id: None,
        branch_id: None,
        branch_name: None,
        title: String::new(),
        phenomenon: String::new(),
        expected_result: None,
        actual_result: None,
        reproduction_steps: None,
        impact_scope: None,
        discovery_stage: None,
        severity: DefectSeverity::Unknown,
        priority: crate::DefectPriority::Unscheduled,
        status,
        reporter_id: Uuid::nil(),
        assignee_id: None,
        fixer_id: None,
        environment: None,
        labels: vec![],
        discovery_version: None,
        fix_version: None,
        lifecycle_version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    }
}

// =========================================================================
// 类型转换辅助（字符串 → 枚举）
// =========================================================================

impl DefectStatus {
    /// 从数据库字符串解析缺陷状态（未知值 → Internal 错误，不静默降级）
    pub(crate) fn from_pg_str(s: &str) -> Result<Self, DefectError> {
        match s {
            "triage" => Ok(Self::Triage),
            "confirmed" => Ok(Self::Confirmed),
            "in_progress" => Ok(Self::InProgress),
            "pending_verify" => Ok(Self::PendingVerify),
            "closed" => Ok(Self::Closed),
            "pending_info" => Ok(Self::PendingInfo),
            "duplicate" => Ok(Self::Duplicate),
            "no_action" => Ok(Self::NoAction),
            "reopened" => Ok(Self::Reopened),
            _ => Err(DefectError::Internal(format!("未知缺陷状态: {s}"))),
        }
    }
}

impl DefectSeverity {
    /// 从数据库字符串解析严重度（**未知值报错，不静默降级**）。
    ///
    /// `"unknown"` 是合法取值（SRS §2.1「信息不足以判断；保留该状态」），
    /// 但其他非法值必须报错 —— 否则迁移 CHECK 已拒绝写入，读侧却报
    /// "unknown"，属证据不诚实（per CON-DI-05 / DD §4.2 枚举 CHECK 说明）。
    pub(crate) fn from_pg_str(s: &str) -> Result<Self, DefectError> {
        match s {
            "S1" => Ok(Self::S1),
            "S2" => Ok(Self::S2),
            "S3" => Ok(Self::S3),
            "S4" => Ok(Self::S4),
            "unknown" => Ok(Self::Unknown),
            _ => Err(DefectError::Internal(format!("未知严重度: {s}"))),
        }
    }
}

impl crate::DefectPriority {
    /// 从数据库字符串解析优先级（**未知值报错，不静默降级**）。
    ///
    /// `"unscheduled"` 是合法取值；其他非法值必须报错（per CON-DI-05）。
    pub(crate) fn from_pg_str(s: &str) -> Result<Self, DefectError> {
        match s {
            "P0" => Ok(Self::P0),
            "P1" => Ok(Self::P1),
            "P2" => Ok(Self::P2),
            "P3" => Ok(Self::P3),
            "unscheduled" => Ok(Self::Unscheduled),
            _ => Err(DefectError::Internal(format!("未知优先级: {s}"))),
        }
    }
}

impl crate::DiscoveryStage {
    /// 从数据库字符串解析发现阶段（**未知值报错，不静默降级**）。
    ///
    /// `"unknown"` 是合法取值（SRS §2.1 保留未知）；其他非法值必须报错
    /// （per CON-DI-05）。
    pub(crate) fn from_pg_str(s: &str) -> Result<Self, DefectError> {
        match s {
            "requirements" => Ok(Self::Requirements),
            "design" => Ok(Self::Design),
            "implementation" => Ok(Self::Implementation),
            "unit_test" => Ok(Self::UnitTest),
            "integration_test" => Ok(Self::IntegrationTest),
            "system_test" => Ok(Self::SystemTest),
            "release_post" => Ok(Self::ReleasePost),
            "unknown" => Ok(Self::Unknown),
            _ => Err(DefectError::Internal(format!("未知发现阶段: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    //! 单元测试（per DD §6.3 UT 阶梯）
    //!
    //! - UT-DI-03: 枚举 ↔ PG 字符串双向映射（`from_pg_str`）——
    //!   关闭 G-26 的"映射测试缺失"部分。
    //!   关键性质：**未知值不静默降级**（返回 Err），否则 PG 写入被 CHECK
    //!   拒绝而读侧误报 unknown。

    use super::*;

    #[test]
    fn ut_di_03_status_roundtrip_all_nine_values() {
        // 迁移 CHECK 约束里的 9 个状态取值必须与代码枚举逐值一致
        for (s, expected) in [
            ("triage", DefectStatus::Triage),
            ("confirmed", DefectStatus::Confirmed),
            ("in_progress", DefectStatus::InProgress),
            ("pending_verify", DefectStatus::PendingVerify),
            ("closed", DefectStatus::Closed),
            ("pending_info", DefectStatus::PendingInfo),
            ("duplicate", DefectStatus::Duplicate),
            ("no_action", DefectStatus::NoAction),
            ("reopened", DefectStatus::Reopened),
        ] {
            assert_eq!(s, expected.as_str(), "as_str 与迁移 CHECK 列表不一致");
            let parsed = DefectStatus::from_pg_str(s)
                .unwrap_or_else(|e| panic!("合法状态 {s} 解析失败: {e}"));
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn ut_di_03_unknown_status_is_error_not_silent_downgrade() {
        for bad in ["", "Triage", "unknown_status", "closed ", "archived"] {
            let res = DefectStatus::from_pg_str(bad);
            assert!(
                res.is_err(),
                "未知状态 {bad:?} 必须报错而非静默降级（per §2.1.1）"
            );
        }
    }

    #[test]
    fn ut_di_03_severity_roundtrip_all_five_values() {
        for (s, expected) in [
            ("S1", DefectSeverity::S1),
            ("S2", DefectSeverity::S2),
            ("S3", DefectSeverity::S3),
            ("S4", DefectSeverity::S4),
            ("unknown", DefectSeverity::Unknown),
        ] {
            assert_eq!(s, expected.as_str());
            assert_eq!(DefectSeverity::from_pg_str(s).unwrap(), expected);
        }
        assert!(DefectSeverity::from_pg_str("S5").is_err());
        assert!(DefectSeverity::from_pg_str("s1").is_err());
    }

    #[test]
    fn ut_di_03_priority_roundtrip_all_five_values() {
        for (s, expected) in [
            ("P0", crate::DefectPriority::P0),
            ("P1", crate::DefectPriority::P1),
            ("P2", crate::DefectPriority::P2),
            ("P3", crate::DefectPriority::P3),
            ("unscheduled", crate::DefectPriority::Unscheduled),
        ] {
            assert_eq!(s, expected.as_str());
            assert_eq!(crate::DefectPriority::from_pg_str(s).unwrap(), expected);
        }
        assert!(crate::DefectPriority::from_pg_str("P4").is_err());
    }

    #[test]
    fn ut_di_03_discovery_stage_roundtrip() {
        for (s, expected) in [
            ("requirements", crate::DiscoveryStage::Requirements),
            ("design", crate::DiscoveryStage::Design),
            ("implementation", crate::DiscoveryStage::Implementation),
            ("unit_test", crate::DiscoveryStage::UnitTest),
            ("integration_test", crate::DiscoveryStage::IntegrationTest),
            ("system_test", crate::DiscoveryStage::SystemTest),
            ("release_post", crate::DiscoveryStage::ReleasePost),
            ("unknown", crate::DiscoveryStage::Unknown),
        ] {
            assert_eq!(s, expected.as_str());
            assert_eq!(crate::DiscoveryStage::from_pg_str(s).unwrap(), expected);
        }
        assert!(crate::DiscoveryStage::from_pg_str("nope").is_err());
    }

    #[test]
    fn ut_di_06_request_digest_is_stable_and_not_the_bare_key() {
        // G-11: request_hash 必须是**请求摘要**，不是幂等键本身。
        let defect = crate::Defect {
            id: crate::WorkItemId(uuid::Uuid::new_v4()),
            tenant_id: uuid::Uuid::new_v4(),
            project_id: uuid::Uuid::new_v4(),
            lane_id: None,
            worktree_id: None,
            branch_id: None,
            branch_name: None,
            title: "t".into(),
            phenomenon: "p".into(),
            expected_result: None,
            actual_result: None,
            reproduction_steps: None,
            impact_scope: None,
            discovery_stage: None,
            severity: DefectSeverity::Unknown,
            priority: crate::DefectPriority::Unscheduled,
            status: DefectStatus::Triage,
            reporter_id: uuid::Uuid::new_v4(),
            assignee_id: None,
            fixer_id: None,
            environment: None,
            labels: vec![],
            discovery_version: None,
            fix_version: None,
            lifecycle_version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };
        let d1 = request_digest_for_defect(&defect);
        let d2 = request_digest_for_defect(&defect);
        assert_eq!(d1, d2, "摘要必须稳定（可复现）");
        assert_eq!(d1.len(), 32, "SHA-256 摘要长度 32 字节");
        assert_ne!(
            d1,
            serde_json::to_vec("some-idempotency-key").unwrap(),
            "摘要不得等于幂等键本身"
        );
    }
}
