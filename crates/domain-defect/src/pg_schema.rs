//! 缺陷管理 PostgreSQL 表 DDL（Phase A.1.2，参考 `crates/infrastructure/src/run_task_card_pg.rs` 表命名惯例）
//!
//! 表命名空间：`defect.*`（与 `multica.*` 并行，避免冲突）
//! 所有表均带 `tenant_id` 做跨租户隔离；业务事实禁止物理删除（`deleted_at` 软删除）。
//!
//! 这些 DDL 字符串供迁移工具或测试桩执行；生产环境通过 Neon 迁移脚本逐条应用
//! （物理迁移：`db/migrations/2026-11-25-defect-domain-phase-a.sql`，含 RLS 与审计不可变触发器）。

/*
@cypher schema=1 source_sha256=f33005db91b354c4cd273b33d33272a6a7d370fdfe8141f7658bf7c8f02a3b86
MERGE (self:File {path:"crates/domain-defect/src/pg_schema.rs"})
MERGE (allTables:Symbol {id:"crates/domain-defect/src/pg_schema.rs::CREATE_ALL_TABLES",kind:"const"})
MERGE (workItems:Table {name:"defect.work_items",type:"table",class:"T_scd2"})
MERGE (verifications:Table {name:"defect.verifications",type:"table",class:"T"})
MERGE (suggestions:Table {name:"defect.suggestions",type:"table",class:"T"})
MERGE (commandIdempotency:Table {name:"defect.command_idempotency",type:"table",class:"T"})
MERGE (auditEvents:Table {name:"defect.audit_events",type:"table",class:"T_append_only"})
MERGE (outbox:Table {name:"defect.outbox",type:"table",class:"T"})
MERGE (fieldValues:Table {name:"defect.field_values",type:"table",class:"T"})
MERGE (relations:Table {name:"defect.relations",type:"table",class:"T"})
MERGE (comments:Table {name:"defect.comments",type:"table",class:"T"})
MERGE (drafts:Table {name:"defect.drafts",type:"table",class:"W_ttl"})
MERGE (statusEvents:Table {name:"defect.status_events",type:"table",class:"T_append_only"})
MERGE (self)-[:DEFINES]->(allTables)
MERGE (allTables)-[:DEFINES]->(workItems),(allTables)-[:DEFINES]->(verifications),(allTables)-[:DEFINES]->(suggestions),(allTables)-[:DEFINES]->(commandIdempotency),(allTables)-[:DEFINES]->(auditEvents),(allTables)-[:DEFINES]->(outbox),(allTables)-[:DEFINES]->(fieldValues),(allTables)-[:DEFINES]->(relations),(allTables)-[:DEFINES]->(comments),(allTables)-[:DEFINES]->(drafts),(allTables)-[:DEFINES]->(statusEvents)
@endcypher
*/

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
    fixer_id           UUID,
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
///
/// per DD §4.2 / G-12：envelope 列（stream_id/source_sequence/event_id/correlation_id/
/// event_digest/schema_version/occurred_at）；`(stream_id, source_sequence)` 唯一保证
/// owner 单调重放顺序，`event_id` UNIQUE 钉住稳定事件身份；投递语义 at-least-once。
pub const CREATE_TABLE_DEFECT_OUTBOX: &str = r#"
CREATE TABLE IF NOT EXISTS defect.outbox (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id        UUID NOT NULL,
    source_sequence  BIGINT NOT NULL,
    event_id         UUID NOT NULL UNIQUE,
    event_type       TEXT NOT NULL,
    schema_version   INTEGER NOT NULL DEFAULT 1,
    producer_id      TEXT NOT NULL,
    consumer_id      TEXT NOT NULL,
    tenant_id        UUID NOT NULL,
    actor_id         UUID,
    correlation_id   UUID NOT NULL,
    event_digest     BYTEA NOT NULL CHECK (octet_length(event_digest) = 32),
    payload          JSONB NOT NULL,
    occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    status           TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','sent','failed')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at          TIMESTAMPTZ,
    UNIQUE (stream_id, source_sequence)
);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created ON defect.outbox(created_at);
"#;

/// 自定义字段值表（缺陷域自有，per DD §0 `defect_field_value`；active/superseded；G-21）
pub const CREATE_TABLE_DEFECT_FIELD_VALUES: &str = r#"
CREATE TABLE IF NOT EXISTS defect.field_values (
    id             UUID PRIMARY KEY,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    field_id       TEXT NOT NULL,
    value          JSONB NOT NULL,
    value_revision INTEGER NOT NULL DEFAULT 1,
    status         TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','superseded')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (defect_id, field_id, value_revision)
);
CREATE INDEX IF NOT EXISTS idx_defect_field_values_defect_field ON defect.field_values(defect_id, field_id);
"#;

/// 缺陷关系表（缺陷域自有，per DD §0 `defect_relation`；禁自引用 per FR-DI-011；G-21）
pub const CREATE_TABLE_DEFECT_RELATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.relations (
    id                UUID PRIMARY KEY,
    source_defect_id  UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    target_defect_id  UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    relation_type     TEXT NOT NULL CHECK (relation_type IN ('parent','child','related','duplicate','blocks','blocked_by')),
    relation_revision INTEGER NOT NULL DEFAULT 1,
    status            TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','revoked')),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (source_defect_id <> target_defect_id),
    UNIQUE (source_defect_id, target_defect_id, relation_type)
);
CREATE INDEX IF NOT EXISTS idx_defect_relations_source ON defect.relations(source_defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_relations_target ON defect.relations(target_defect_id);
"#;

/// 评论表（缺陷域自有，per DD §0 `defect_comment`；逻辑删除 active/deleted；G-21）
pub const CREATE_TABLE_DEFECT_COMMENTS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.comments (
    id               UUID PRIMARY KEY,
    defect_id        UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    author_id        UUID NOT NULL,
    content          TEXT NOT NULL,
    comment_revision INTEGER NOT NULL DEFAULT 1,
    status           TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','deleted')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_comments_defect_time ON defect.comments(defect_id, created_at DESC);
"#;

/// 草稿表（缺陷域自有，W 类；会话 owner；tenant 直挂；TTL/retention per G-24：有界大小 + expires_at；G-21）
pub const CREATE_TABLE_DEFECT_DRAFTS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.drafts (
    id             UUID PRIMARY KEY,
    tenant_id      UUID NOT NULL,
    project_id     UUID NOT NULL,
    lane_id        UUID,
    worktree_id    UUID,
    defect_id      UUID REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    draft_revision INTEGER NOT NULL DEFAULT 1,
    status         TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','submitted','expired')),
    payload        JSONB NOT NULL DEFAULT '{}',
    expires_at     TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '7 days'),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (octet_length(payload::text) <= 1048576)
);
CREATE INDEX IF NOT EXISTS idx_defect_drafts_tenant_status ON defect.drafts(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_defect_drafts_expires ON defect.drafts(expires_at) WHERE status = 'open';
"#;

/// 状态事件表（缺陷域自有，per DD §0 `defect_status_event` + D-DI-05 推荐独立建表；Append-only；G-22）
pub const CREATE_TABLE_DEFECT_STATUS_EVENTS: &str = r#"
CREATE TABLE IF NOT EXISTS defect.status_events (
    event_id       UUID PRIMARY KEY,
    event_revision INTEGER NOT NULL DEFAULT 1,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    event_type     TEXT NOT NULL CHECK (event_type IN ('status_changed','field_changed','assignee_changed','relation_changed','verification_changed')),
    actor_user_id  UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    project_id     UUID NOT NULL,
    from_status    TEXT,
    to_status      TEXT,
    reason         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (defect_id, event_revision)
);
CREATE INDEX IF NOT EXISTS idx_defect_status_events_defect_time ON defect.status_events(defect_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_defect_status_events_type ON defect.status_events(event_type);
"#;


/// 通知 Inbox 幂等行（per G-8 消费者回路）
pub const CREATE_TABLE_DEFECT_NOTIFICATION_INBOX: &str = r#"
CREATE TABLE IF NOT EXISTS defect.notification_inbox (
    sequence       BIGINT PRIMARY KEY,
    event_type     TEXT NOT NULL,
    event_id       UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32),
    payload        JSONB NOT NULL,
    received_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    applied_at     TIMESTAMPTZ,
    receipt_id     UUID
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_defect_notification_inbox_event ON defect.notification_inbox(event_id);
"#;

/// Inbox 持久游标（per G-8；可挂起，fail-closed 不丢位点）
pub const CREATE_TABLE_DEFECT_NOTIFICATION_INBOX_CURSOR: &str = r#"
CREATE TABLE IF NOT EXISTS defect.notification_inbox_cursor (
    consumer_id     TEXT NOT NULL,
    producer_id     TEXT NOT NULL,
    sequence        BIGINT NOT NULL DEFAULT 0,
    suspended_reason TEXT CHECK (suspended_reason IN ('sequence_gap','retention_gap','digest_conflict')),
    suspended_at    TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (consumer_id, producer_id)
);
"#;

/// 通知投影（per G-8 消费结果）
pub const CREATE_TABLE_DEFECT_NOTIFICATION_PROJECTION: &str = r#"
CREATE TABLE IF NOT EXISTS defect.notification_projection (
    event_id       UUID PRIMARY KEY,
    event_type     TEXT NOT NULL,
    correlation_id UUID NOT NULL,
    payload        JSONB NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
"#;

/// 隔离事故（per G-8 摘要冲突 / 非法信封）
pub const CREATE_TABLE_DEFECT_NOTIFICATION_INBOX_INCIDENT: &str = r#"
CREATE TABLE IF NOT EXISTS defect.notification_inbox_incident (
    event_id       UUID NOT NULL,
    reason         TEXT NOT NULL,
    event_type     TEXT NOT NULL,
    correlation_id UUID NOT NULL,
    payload        JSONB NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (event_id, reason)
);
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
    fixer_id           UUID,
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
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id        UUID NOT NULL,
    source_sequence  BIGINT NOT NULL,
    event_id         UUID NOT NULL UNIQUE,
    event_type       TEXT NOT NULL,
    schema_version   INTEGER NOT NULL DEFAULT 1,
    producer_id      TEXT NOT NULL,
    consumer_id      TEXT NOT NULL,
    tenant_id        UUID NOT NULL,
    actor_id         UUID,
    correlation_id   UUID NOT NULL,
    event_digest     BYTEA NOT NULL CHECK (octet_length(event_digest) = 32),
    payload          JSONB NOT NULL,
    occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    status           TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','sent','failed')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at          TIMESTAMPTZ,
    UNIQUE (stream_id, source_sequence)
);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created ON defect.outbox(created_at);

CREATE TABLE IF NOT EXISTS defect.field_values (
    id             UUID PRIMARY KEY,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    field_id       TEXT NOT NULL,
    value          JSONB NOT NULL,
    value_revision INTEGER NOT NULL DEFAULT 1,
    status         TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','superseded')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (defect_id, field_id, value_revision)
);
CREATE INDEX IF NOT EXISTS idx_defect_field_values_defect_field ON defect.field_values(defect_id, field_id);

CREATE TABLE IF NOT EXISTS defect.relations (
    id                UUID PRIMARY KEY,
    source_defect_id  UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    target_defect_id  UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    relation_type     TEXT NOT NULL CHECK (relation_type IN ('parent','child','related','duplicate','blocks','blocked_by')),
    relation_revision INTEGER NOT NULL DEFAULT 1,
    status            TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','revoked')),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (source_defect_id <> target_defect_id),
    UNIQUE (source_defect_id, target_defect_id, relation_type)
);
CREATE INDEX IF NOT EXISTS idx_defect_relations_source ON defect.relations(source_defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_relations_target ON defect.relations(target_defect_id);

CREATE TABLE IF NOT EXISTS defect.comments (
    id               UUID PRIMARY KEY,
    defect_id        UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    author_id        UUID NOT NULL,
    content          TEXT NOT NULL,
    comment_revision INTEGER NOT NULL DEFAULT 1,
    status           TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','deleted')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_comments_defect_time ON defect.comments(defect_id, created_at DESC);

CREATE TABLE IF NOT EXISTS defect.drafts (
    id             UUID PRIMARY KEY,
    tenant_id      UUID NOT NULL,
    project_id     UUID NOT NULL,
    lane_id        UUID,
    worktree_id    UUID,
    defect_id      UUID REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    draft_revision INTEGER NOT NULL DEFAULT 1,
    status         TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','submitted','expired')),
    payload        JSONB NOT NULL DEFAULT '{}',
    expires_at     TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '7 days'),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (octet_length(payload::text) <= 1048576)
);
CREATE INDEX IF NOT EXISTS idx_defect_drafts_tenant_status ON defect.drafts(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_defect_drafts_expires ON defect.drafts(expires_at) WHERE status = 'open';

CREATE TABLE IF NOT EXISTS defect.status_events (
    event_id       UUID PRIMARY KEY,
    event_revision INTEGER NOT NULL DEFAULT 1,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    event_type     TEXT NOT NULL CHECK (event_type IN ('status_changed','field_changed','assignee_changed','relation_changed','verification_changed')),
    actor_user_id  UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    project_id     UUID NOT NULL,
    from_status    TEXT,
    to_status      TEXT,
    reason         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (defect_id, event_revision)
);
CREATE INDEX IF NOT EXISTS idx_defect_status_events_defect_time ON defect.status_events(defect_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_defect_status_events_type ON defect.status_events(event_type);
CREATE TABLE IF NOT EXISTS defect.notification_inbox (
    sequence       BIGINT PRIMARY KEY,
    event_type     TEXT NOT NULL,
    event_id       UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32),
    payload        JSONB NOT NULL,
    received_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    applied_at     TIMESTAMPTZ,
    receipt_id     UUID
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_defect_notification_inbox_event ON defect.notification_inbox(event_id);
CREATE TABLE IF NOT EXISTS defect.notification_inbox_cursor (
    consumer_id     TEXT NOT NULL,
    producer_id     TEXT NOT NULL,
    sequence        BIGINT NOT NULL DEFAULT 0,
    suspended_reason TEXT CHECK (suspended_reason IN ('sequence_gap','retention_gap','digest_conflict')),
    suspended_at    TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (consumer_id, producer_id)
);
CREATE TABLE IF NOT EXISTS defect.notification_projection (
    event_id       UUID PRIMARY KEY,
    event_type     TEXT NOT NULL,
    correlation_id UUID NOT NULL,
    payload        JSONB NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS defect.notification_inbox_incident (
    event_id       UUID NOT NULL,
    reason         TEXT NOT NULL,
    event_type     TEXT NOT NULL,
    correlation_id UUID NOT NULL,
    payload        JSONB NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (event_id, reason)
);

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
            CREATE_TABLE_DEFECT_FIELD_VALUES,
            CREATE_TABLE_DEFECT_RELATIONS,
            CREATE_TABLE_DEFECT_COMMENTS,
            CREATE_TABLE_DEFECT_DRAFTS,
            CREATE_TABLE_DEFECT_STATUS_EVENTS,
            CREATE_ALL_TABLES,
        ] {
            assert!(!ddl.trim().is_empty());
        }
    }

    #[test]
    fn create_all_tables_covers_all_defect_tables() {
        for table in [
            "defect.work_items",
            "defect.verifications",
            "defect.suggestions",
            "defect.command_idempotency",
            "defect.audit_events",
            "defect.outbox",
            "defect.field_values",
            "defect.relations",
            "defect.comments",
            "defect.drafts",
            "defect.status_events",
        ] {
            assert!(
                CREATE_ALL_TABLES.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
                "CREATE_ALL_TABLES 缺表: {table}"
            );
        }
    }

    #[test]
    fn outbox_ddl_carries_envelope_columns() {
        for ddl in [CREATE_TABLE_DEFECT_OUTBOX, CREATE_ALL_TABLES] {
            let compact: String = ddl.split_whitespace().collect::<Vec<_>>().join(" ");
            for fragment in [
                "stream_id UUID NOT NULL",
                "source_sequence BIGINT NOT NULL",
                "event_id UUID NOT NULL UNIQUE",
                "schema_version INTEGER NOT NULL DEFAULT 1",
                "correlation_id UUID NOT NULL",
                "event_digest BYTEA NOT NULL CHECK (octet_length(event_digest) = 32)",
                "occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()",
                "UNIQUE (stream_id, source_sequence)",
            ] {
                assert!(
                    compact.contains(fragment),
                    "outbox DDL 缺 envelope 列: {fragment}"
                );
            }
        }
    }
}
