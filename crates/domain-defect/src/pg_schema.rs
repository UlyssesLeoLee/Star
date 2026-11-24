//! 缺陷管理 PostgreSQL 表 DDL（Phase A.1.2，参考 `crates/infrastructure/src/run_task_card_pg.rs` 表命名惯例）
//!
//! 表命名空间：`defect.*`（与 `multica.*` 并行，避免冲突）
//! 所有表均带 `tenant_id` 做跨租户隔离；业务事实禁止物理删除（`deleted_at` 软删除）。
//!
//! 这些 DDL 字符串供迁移工具或测试桩执行；生产环境通过 Neon 迁移脚本逐条应用。

/// 缺陷主表（复用 WorkItemId 作为 defect_id，与 Bug WorkItem 稳定身份同一空间）
pub const CREATE_TABLE_DEFECTS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.work_items (
    defect_id          UUID PRIMARY KEY,
    tenant_id          UUID NOT NULL,
    project_id         UUID NOT NULL,
    lane_id            UUID,
    worktree_id        UUID,
    branch_id          UUID,
    branch_name        TEXT,
    title              TEXT NOT NULL,
    phenomenon         TEXT NOT NULL,
    expected_result    TEXT,
    actual_result      TEXT,
    reproduction_steps TEXT,
    impact_scope       TEXT,
    discovery_stage    TEXT,
    severity           TEXT NOT NULL DEFAULT 'unknown',
    priority           TEXT NOT NULL DEFAULT 'unscheduled',
    status             TEXT NOT NULL DEFAULT 'triage',
    reporter_id        UUID NOT NULL,
    assignee_id        UUID,
    environment        TEXT,
    labels             TEXT[] NOT NULL DEFAULT '{}',
    discovery_version  TEXT,
    fix_version        TEXT,
    lifecycle_version  INTEGER NOT NULL DEFAULT 1,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at         TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_tenant_project ON defect.work_items(tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_lane ON defect.work_items(lane_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_status ON defect.work_items(status);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_fix_version ON defect.work_items(fix_version);
"#;

/// 验证记录表（保存验证主体、修复版本、方法、结果、证据）
pub const CREATE_TABLE_DEFECT_VERIFICATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.verifications (
    id             UUID PRIMARY KEY,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    verifier_id    UUID NOT NULL,
    fix_version    TEXT NOT NULL,
    method         TEXT,
    result         TEXT NOT NULL CHECK (result IN ('passed','failed')),
    evidence_id    UUID,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at     TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_defect ON defect.verifications(defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_verifier ON defect.verifications(verifier_id);
"#;

/// 智能建议草稿表
pub const CREATE_TABLE_DEFECT_SUGGESTIONS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.suggestions (
    id                     UUID PRIMARY KEY,
    defect_id              UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    suggested_severity     TEXT,
    suggested_priority     TEXT,
    suggested_discovery_stage TEXT,
    reason                 TEXT,
    status                 TEXT NOT NULL DEFAULT 'draft',
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_suggestions_defect ON defect.suggestions(defect_id);
"#;

/// 幂等表（tenant_id + actor_id + idempotency_key 唯一，防重复提交）
pub const CREATE_TABLE_DEFECT_COMMAND_IDEMPOTENCY: &str = r#"
CREATE TABLE IF NOT EXISTS defect.command_idempotency (
    tenant_id      UUID NOT NULL,
    actor_id       UUID NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash   BYTEA NOT NULL,
    response_body  JSONB NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, actor_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_defect_idempotency_expires ON defect.command_idempotency(expires_at);
"#;

/// 审计事件表（owner 事务内写入，不可篡改）
pub const CREATE_TABLE_DEFECT_AUDIT_EVENTS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.audit_events (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type     TEXT NOT NULL,
    actor_user_id  UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    project_id     UUID,
    target_id      TEXT,
    payload        JSONB NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_tenant ON defect.audit_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_project ON defect.audit_events(project_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_created ON defect.audit_events(created_at);
"#;

/// Outbox 表（事件投递，fail-closed，投递前保持未发送状态）
pub const CREATE_TABLE_DEFECT_OUTBOX: &str = r#"
CREATE TABLE IF NOT EXISTS defect.outbox (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT NOT NULL,
    aggregate_id   TEXT NOT NULL,
    event_type     TEXT NOT NULL,
    payload        JSONB NOT NULL,
    status         TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','sent','failed')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at        TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created ON defect.outbox(created_at);
"#;

/// 一次性执行全部建表（供测试桩或启动时确保 schema）
pub const CREATE_ALL_TABLES: &str = "CREATE SCHEMA IF NOT EXISTS defect;
CREATE TABLE IF NOT EXISTS defect.work_items (
    defect_id          UUID PRIMARY KEY,
    tenant_id          UUID NOT NULL,
    project_id         UUID NOT NULL,
    lane_id            UUID,
    worktree_id        UUID,
    branch_id          UUID,
    branch_name        TEXT,
    title              TEXT NOT NULL,
    phenomenon         TEXT NOT NULL,
    expected_result    TEXT,
    actual_result      TEXT,
    reproduction_steps TEXT,
    impact_scope       TEXT,
    discovery_stage    TEXT,
    severity           TEXT NOT NULL DEFAULT 'unknown',
    priority           TEXT NOT NULL DEFAULT 'unscheduled',
    status             TEXT NOT NULL DEFAULT 'triage',
    reporter_id        UUID NOT NULL,
    assignee_id        UUID,
    environment        TEXT,
    labels             TEXT[] NOT NULL DEFAULT '{}',
    discovery_version  TEXT,
    fix_version        TEXT,
    lifecycle_version  INTEGER NOT NULL DEFAULT 1,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at         TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_tenant_project ON defect.work_items(tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_lane ON defect.work_items(lane_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_status ON defect.work_items(status);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_fix_version ON defect.work_items(fix_version);

CREATE TABLE IF NOT EXISTS defect.verifications (
    id             UUID PRIMARY KEY,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    verifier_id    UUID NOT NULL,
    fix_version    TEXT NOT NULL,
    method         TEXT,
    result         TEXT NOT NULL CHECK (result IN ('passed','failed')),
    evidence_id    UUID,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at     TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_defect ON defect.verifications(defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_verifier ON defect.verifications(verifier_id);

CREATE TABLE IF NOT EXISTS defect.suggestions (
    id                     UUID PRIMARY KEY,
    defect_id              UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    suggested_severity     TEXT,
    suggested_priority     TEXT,
    suggested_discovery_stage TEXT,
    reason                 TEXT,
    status                 TEXT NOT NULL DEFAULT 'draft',
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_suggestions_defect ON defect.suggestions(defect_id);

CREATE TABLE IF NOT EXISTS defect.command_idempotency (
    tenant_id      UUID NOT NULL,
    actor_id       UUID NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash   BYTEA NOT NULL,
    response_body  JSONB NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, actor_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_defect_idempotency_expires ON defect.command_idempotency(expires_at);

CREATE TABLE IF NOT EXISTS defect.audit_events (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type     TEXT NOT NULL,
    actor_user_id  UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    project_id     UUID,
    target_id      TEXT,
    payload        JSONB NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_tenant ON defect.audit_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_project ON defect.audit_events(project_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_created ON defect.audit_events(created_at);

CREATE TABLE IF NOT EXISTS defect.outbox (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT NOT NULL,
    aggregate_id   TEXT NOT NULL,
    event_type     TEXT NOT NULL,
    payload        JSONB NOT NULL,
    status         TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','sent','failed')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at        TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created ON defect.outbox(created_at);
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddl_strings_non_empty() {
        for ddl in [
            CREATE_TABLE_DEFECTS,
            CREATE_TABLE_DEFECT_VERIFICATIONS,
            CREATE_TABLE_DEFECT_SUGGESTIONS,
            CREATE_TABLE_DEFECT_COMMAND_IDEMPOTENCY,
            CREATE_TABLE_DEFECT_AUDIT_EVENTS,
            CREATE_TABLE_DEFECT_OUTBOX,
            CREATE_ALL_TABLES,
        ] {
            assert!(!ddl.trim().is_empty());
        }
    }
}
