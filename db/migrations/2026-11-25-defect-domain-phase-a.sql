-- @cypher schema=1 source_sha256=8e8a9d7de5be547f8f1b38e8cbdf4852944029b3b3883d57e5e4f65eaf3fec32
-- MERGE (self:File {path:"db/migrations/2026-11-25-defect-domain-phase-a.sql"})
-- MERGE (workItems:Table {name:"defect.work_items",type:"table",class:"T_scd2"})
-- MERGE (verifications:Table {name:"defect.verifications",type:"table",class:"T"})
-- MERGE (suggestions:Table {name:"defect.suggestions",type:"table",class:"T"})
-- MERGE (commandIdempotency:Table {name:"defect.command_idempotency",type:"table",class:"T"})
-- MERGE (auditEvents:Table {name:"defect.audit_events",type:"table",class:"T_append_only"})
-- MERGE (outbox:Table {name:"defect.outbox",type:"table",class:"T"})
-- MERGE (fieldValues:Table {name:"defect.field_values",type:"table",class:"T"})
-- MERGE (relations:Table {name:"defect.relations",type:"table",class:"T"})
-- MERGE (comments:Table {name:"defect.comments",type:"table",class:"T"})
-- MERGE (drafts:Table {name:"defect.drafts",type:"table",class:"W_ttl"})
-- MERGE (statusEvents:Table {name:"defect.status_events",type:"table",class:"T_append_only"})
-- MERGE (workItemsPolicy:Policy {id:"defect.work_items::defect_work_items_tenant_scope",kind:"row_level_security"})
-- MERGE (verificationsPolicy:Policy {id:"defect.verifications::defect_verifications_tenant_scope",kind:"row_level_security"})
-- MERGE (suggestionsPolicy:Policy {id:"defect.suggestions::defect_suggestions_tenant_scope",kind:"row_level_security"})
-- MERGE (idempotencyPolicy:Policy {id:"defect.command_idempotency::defect_idempotency_tenant_scope",kind:"row_level_security"})
-- MERGE (auditPolicy:Policy {id:"defect.audit_events::defect_audit_events_tenant_scope",kind:"row_level_security"})
-- MERGE (outboxPolicy:Policy {id:"defect.outbox::defect_outbox_tenant_scope",kind:"row_level_security"})
-- MERGE (fieldValuesPolicy:Policy {id:"defect.field_values::defect_field_values_tenant_scope",kind:"row_level_security"})
-- MERGE (relationsPolicy:Policy {id:"defect.relations::defect_relations_tenant_scope",kind:"row_level_security"})
-- MERGE (commentsPolicy:Policy {id:"defect.comments::defect_comments_tenant_scope",kind:"row_level_security"})
-- MERGE (draftsPolicy:Policy {id:"defect.drafts::defect_drafts_tenant_scope",kind:"row_level_security"})
-- MERGE (statusEventsPolicy:Policy {id:"defect.status_events::defect_status_events_tenant_scope",kind:"row_level_security"})
-- MERGE (rejectMutation:Database {id:"defect.reject_defect_audit_mutation",kind:"trigger_function"})
-- MERGE (self)-[:CREATES]->(workItems),(self)-[:CREATES]->(verifications),(self)-[:CREATES]->(suggestions),(self)-[:CREATES]->(commandIdempotency),(self)-[:CREATES]->(auditEvents),(self)-[:CREATES]->(outbox),(self)-[:CREATES]->(fieldValues),(self)-[:CREATES]->(relations),(self)-[:CREATES]->(comments),(self)-[:CREATES]->(drafts),(self)-[:CREATES]->(statusEvents)
-- MERGE (self)-[:CONFIGURES]->(workItemsPolicy),(self)-[:CONFIGURES]->(verificationsPolicy),(self)-[:CONFIGURES]->(suggestionsPolicy),(self)-[:CONFIGURES]->(idempotencyPolicy),(self)-[:CONFIGURES]->(auditPolicy),(self)-[:CONFIGURES]->(outboxPolicy),(self)-[:CONFIGURES]->(fieldValuesPolicy),(self)-[:CONFIGURES]->(relationsPolicy),(self)-[:CONFIGURES]->(commentsPolicy),(self)-[:CONFIGURES]->(draftsPolicy),(self)-[:CONFIGURES]->(statusEventsPolicy)
-- MERGE (workItemsPolicy)-[:PROTECTS]->(workItems)
-- MERGE (verificationsPolicy)-[:PROTECTS]->(verifications)
-- MERGE (suggestionsPolicy)-[:PROTECTS]->(suggestions)
-- MERGE (idempotencyPolicy)-[:PROTECTS]->(commandIdempotency)
-- MERGE (auditPolicy)-[:PROTECTS]->(auditEvents)
-- MERGE (outboxPolicy)-[:PROTECTS]->(outbox)
-- MERGE (fieldValuesPolicy)-[:PROTECTS]->(fieldValues)
-- MERGE (relationsPolicy)-[:PROTECTS]->(relations)
-- MERGE (commentsPolicy)-[:PROTECTS]->(comments)
-- MERGE (draftsPolicy)-[:PROTECTS]->(drafts)
-- MERGE (statusEventsPolicy)-[:PROTECTS]->(statusEvents)
-- MERGE (self)-[:DEFINES]->(rejectMutation)
-- MERGE (rejectMutation)-[:GUARDS]->(auditEvents),(rejectMutation)-[:GUARDS]->(statusEvents)
-- @endcypher
-- 缺陷域 Phase A：defect.* 10 表 + 索引 + RLS + 审计不可变触发器。
-- 6 表基础 DDL 逐列取自 DD §4.2 目标 DDL（outbox 含 envelope 列，G-12）；field_values/relations/comments/drafts 为缺陷域自有 4 表（G-21，drafts 为 W 类带 expires_at TTL per G-24）；status_events 按 D-DI-05 推荐独立建表（G-22）；audit_events 挂不可变触发器（G-11）。
-- 迁移 runner（crates/star-pg-adapter/src/lib.rs apply_migration）按物理行执行：每行一条完整语句，跳过空行与 `--` 注释行；runner 自带事务，禁止 BEGIN/COMMIT。
CREATE SCHEMA IF NOT EXISTS defect;
-- 缺陷核心事实 (T, SCD2；业务事实禁止物理删除)
CREATE TABLE IF NOT EXISTS defect.work_items (defect_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, project_id UUID NOT NULL, lane_id UUID, worktree_id UUID, branch_id UUID, branch_name TEXT, title TEXT NOT NULL, phenomenon TEXT NOT NULL, expected_result TEXT, actual_result TEXT, reproduction_steps TEXT, impact_scope TEXT, discovery_stage TEXT, severity TEXT NOT NULL DEFAULT 'unknown' CHECK (severity IN ('S1','S2','S3','S4','unknown')), priority TEXT NOT NULL DEFAULT 'unscheduled' CHECK (priority IN ('P0','P1','P2','P3','unscheduled')), status TEXT NOT NULL DEFAULT 'triage' CHECK (status IN ('triage','confirmed','in_progress','pending_verify','closed','pending_info','duplicate','no_action','reopened')), reporter_id UUID NOT NULL, assignee_id UUID, fixer_id UUID, environment TEXT, labels TEXT[] NOT NULL DEFAULT '{}', discovery_version TEXT, fix_version TEXT, lifecycle_version INTEGER NOT NULL DEFAULT 1, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), deleted_at TIMESTAMPTZ);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_tenant_project ON defect.work_items(tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_lane ON defect.work_items(lane_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_status ON defect.work_items(status);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_fix_version ON defect.work_items(fix_version);
-- 验证记录 (T)
CREATE TABLE IF NOT EXISTS defect.verifications (id UUID PRIMARY KEY, defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, verifier_id UUID NOT NULL, fix_version TEXT NOT NULL, method TEXT, result TEXT NOT NULL CHECK (result IN ('passed','failed')), evidence_id UUID, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), deleted_at TIMESTAMPTZ);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_defect ON defect.verifications(defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_verifier ON defect.verifications(verifier_id);
-- 智能建议草稿 (T)
CREATE TABLE IF NOT EXISTS defect.suggestions (id UUID PRIMARY KEY, defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, suggested_severity TEXT, suggested_priority TEXT, suggested_discovery_stage TEXT, reason TEXT, status TEXT NOT NULL DEFAULT 'draft', created_at TIMESTAMPTZ NOT NULL DEFAULT now());
CREATE INDEX IF NOT EXISTS idx_defect_suggestions_defect ON defect.suggestions(defect_id);
-- 幂等回执 (T；同键不同请求按 request_hash 判定)
CREATE TABLE IF NOT EXISTS defect.command_idempotency (tenant_id UUID NOT NULL, actor_id UUID NOT NULL, idempotency_key TEXT NOT NULL, request_hash BYTEA NOT NULL, response_body JSONB NOT NULL, expires_at TIMESTAMPTZ NOT NULL, PRIMARY KEY (tenant_id, actor_id, idempotency_key));
CREATE INDEX IF NOT EXISTS idx_defect_idempotency_expires ON defect.command_idempotency(expires_at);
-- 审计事件 (T；Append-only，禁 UPDATE/DELETE)
CREATE TABLE IF NOT EXISTS defect.audit_events (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), event_type TEXT NOT NULL, actor_user_id UUID NOT NULL, tenant_id UUID NOT NULL, project_id UUID, target_id TEXT, payload JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now());
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_tenant ON defect.audit_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_project ON defect.audit_events(project_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_created ON defect.audit_events(created_at);
-- Outbox 事件 (T；增补 envelope 列后方可被 defect_outbox 消费者消费，per G-12)
CREATE TABLE IF NOT EXISTS defect.outbox (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), stream_id UUID NOT NULL, source_sequence BIGINT NOT NULL, event_id UUID NOT NULL UNIQUE, event_type TEXT NOT NULL, schema_version INTEGER NOT NULL DEFAULT 1, producer_id TEXT NOT NULL, consumer_id TEXT NOT NULL, tenant_id UUID NOT NULL, actor_id UUID, correlation_id UUID NOT NULL, event_digest BYTEA NOT NULL CHECK (octet_length(event_digest) = 32), payload JSONB NOT NULL, occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(), status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','sent','failed')), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), sent_at TIMESTAMPTZ, UNIQUE (stream_id, source_sequence));
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created ON defect.outbox(created_at);
-- 自定义字段值 (T；缺陷域自有，per DD §0 defect_field_value，G-21)
CREATE TABLE IF NOT EXISTS defect.field_values (id UUID PRIMARY KEY, defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, field_id TEXT NOT NULL, value JSONB NOT NULL, value_revision INTEGER NOT NULL DEFAULT 1, status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','superseded')), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE (defect_id, field_id, value_revision));
CREATE INDEX IF NOT EXISTS idx_defect_field_values_defect_field ON defect.field_values(defect_id, field_id);
-- 缺陷关系 (T；缺陷域自有，per DD §0 defect_relation；禁自引用 per FR-DI-011，G-21)
CREATE TABLE IF NOT EXISTS defect.relations (id UUID PRIMARY KEY, source_defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, target_defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, relation_type TEXT NOT NULL CHECK (relation_type IN ('parent','child','related','duplicate','blocks','blocked_by')), relation_revision INTEGER NOT NULL DEFAULT 1, status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','revoked')), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), CHECK (source_defect_id <> target_defect_id), UNIQUE (source_defect_id, target_defect_id, relation_type));
CREATE INDEX IF NOT EXISTS idx_defect_relations_source ON defect.relations(source_defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_relations_target ON defect.relations(target_defect_id);
-- 评论 (T；缺陷域自有，per DD §0 defect_comment；逻辑删除，G-21)
CREATE TABLE IF NOT EXISTS defect.comments (id UUID PRIMARY KEY, defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, author_id UUID NOT NULL, content TEXT NOT NULL, comment_revision INTEGER NOT NULL DEFAULT 1, status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','deleted')), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now());
CREATE INDEX IF NOT EXISTS idx_defect_comments_defect_time ON defect.comments(defect_id, created_at DESC);
-- 缺陷草稿 (W；会话 owner；有界大小 + expires_at TTL，per SRS §5.1 / G-24；tenant 直挂便于会话级 RLS，G-21)
CREATE TABLE IF NOT EXISTS defect.drafts (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, project_id UUID NOT NULL, lane_id UUID, worktree_id UUID, defect_id UUID REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, draft_revision INTEGER NOT NULL DEFAULT 1, status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','submitted','expired')), payload JSONB NOT NULL DEFAULT '{}', expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '7 days'), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), CHECK (octet_length(payload::text) <= 1048576));
CREATE INDEX IF NOT EXISTS idx_defect_drafts_tenant_status ON defect.drafts(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_defect_drafts_expires ON defect.drafts(expires_at) WHERE status = 'open';
-- 状态事件 (T；Append-only；per D-DI-05 推荐独立建表，G-22)
CREATE TABLE IF NOT EXISTS defect.status_events (event_id UUID PRIMARY KEY, event_revision INTEGER NOT NULL DEFAULT 1, defect_id UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE, event_type TEXT NOT NULL CHECK (event_type IN ('status_changed','field_changed','assignee_changed','relation_changed','verification_changed')), actor_user_id UUID NOT NULL, tenant_id UUID NOT NULL, project_id UUID NOT NULL, from_status TEXT, to_status TEXT, reason TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE (defect_id, event_revision));
CREATE INDEX IF NOT EXISTS idx_defect_status_events_defect_time ON defect.status_events(defect_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_defect_status_events_type ON defect.status_events(event_type);
-- 审计/状态事件不可变触发器 (G-11；参照 multica.reject_task_audit_mutation，2026-09-29-worktree-group-phase-2c.sql:128-137)
CREATE OR REPLACE FUNCTION defect.reject_defect_audit_mutation() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'defect event history is append-only'; END; $$;
DROP TRIGGER IF EXISTS defect_audit_events_no_mutation ON defect.audit_events;
CREATE TRIGGER defect_audit_events_no_mutation BEFORE UPDATE OR DELETE ON defect.audit_events FOR EACH ROW EXECUTE FUNCTION defect.reject_defect_audit_mutation();
DROP TRIGGER IF EXISTS defect_status_events_no_mutation ON defect.status_events;
CREATE TRIGGER defect_status_events_no_mutation BEFORE UPDATE OR DELETE ON defect.status_events FOR EACH ROW EXECUTE FUNCTION defect.reject_defect_audit_mutation();
-- RLS：10 表逐表 ENABLE + FORCE + tenant 策略 (DD §4.2 既有 6 表策略 + 新 4 表)
ALTER TABLE defect.work_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.work_items FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_work_items_tenant_scope ON defect.work_items USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.verifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.verifications FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_verifications_tenant_scope ON defect.verifications USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.verifications.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.verifications.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.suggestions ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.suggestions FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_suggestions_tenant_scope ON defect.suggestions USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.suggestions.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.suggestions.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.command_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.command_idempotency FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_idempotency_tenant_scope ON defect.command_idempotency USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.audit_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.audit_events FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_audit_events_tenant_scope ON defect.audit_events USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_outbox_tenant_scope ON defect.outbox USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.field_values ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.field_values FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_field_values_tenant_scope ON defect.field_values USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.field_values.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.field_values.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.relations ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.relations FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_relations_tenant_scope ON defect.relations USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.relations.source_defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.relations.source_defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.comments ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.comments FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_comments_tenant_scope ON defect.comments USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.comments.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.comments.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.drafts ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.drafts FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_drafts_tenant_scope ON defect.drafts USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.status_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.status_events FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_status_events_tenant_scope ON defect.status_events USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
-- 通知 Inbox 幂等行 (T；per G-8 消费者回路)：同序列只存一份，digest 不符即隔离
CREATE TABLE IF NOT EXISTS defect.notification_inbox (sequence BIGINT PRIMARY KEY CHECK (sequence > 0), event_type TEXT NOT NULL, event_id UUID NOT NULL, correlation_id UUID NOT NULL, payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32), payload JSONB NOT NULL, received_at TIMESTAMPTZ NOT NULL DEFAULT now(), applied_at TIMESTAMPTZ, receipt_id UUID);
CREATE UNIQUE INDEX IF NOT EXISTS idx_defect_notification_inbox_event ON defect.notification_inbox(event_id);
-- Inbox 持久游标 (T；per G-8)：可挂起，fail-closed 不丢位点
CREATE TABLE IF NOT EXISTS defect.notification_inbox_cursor (consumer_id TEXT NOT NULL, producer_id TEXT NOT NULL, sequence BIGINT NOT NULL DEFAULT 0 CHECK (sequence >= 0), suspended_reason TEXT CHECK (suspended_reason IN ('sequence_gap','retention_gap','digest_conflict')), suspended_at TIMESTAMPTZ, updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY (consumer_id, producer_id));
-- 通知投影 (T；per G-8 消费结果)
CREATE TABLE IF NOT EXISTS defect.notification_projection (event_id UUID PRIMARY KEY, event_type TEXT NOT NULL, correlation_id UUID NOT NULL, payload JSONB NOT NULL, recorded_at TIMESTAMPTZ NOT NULL DEFAULT now());
-- 隔离事故 (T；摘要冲突 / 非法信封，不覆盖既有事实)
CREATE TABLE IF NOT EXISTS defect.notification_inbox_incident (event_id UUID NOT NULL, reason TEXT NOT NULL, event_type TEXT NOT NULL, correlation_id UUID NOT NULL, payload JSONB NOT NULL, recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY (event_id, reason));
ALTER TABLE defect.notification_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.notification_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_notification_inbox_tenant_scope ON defect.notification_inbox USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.notification_inbox_cursor ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.notification_inbox_cursor FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_notification_inbox_cursor_tenant_scope ON defect.notification_inbox_cursor USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.notification_projection ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.notification_projection FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_notification_projection_tenant_scope ON defect.notification_projection USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.notification_inbox_incident ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.notification_inbox_incident FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_notification_inbox_incident_tenant_scope ON defect.notification_inbox_incident USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
COMMENT ON TABLE defect.notification_inbox IS 'T: 通知 Inbox 幂等行 (per G-8 消费者回路)';
COMMENT ON TABLE defect.notification_inbox_cursor IS 'T: Inbox 持久游标，可挂起 fail-closed';
COMMENT ON TABLE defect.notification_projection IS 'T: 通知投影 (消费结果)';
COMMENT ON TABLE defect.notification_inbox_incident IS 'T: 隔离事故 (摘要冲突 / 非法信封)';
-- W/T/M 表分类注释 (per 守门 #13)
COMMENT ON TABLE defect.work_items IS 'T: 缺陷核心事实 (SCD2)，业务事实禁止物理删除';
COMMENT ON TABLE defect.verifications IS 'T: 验证记录 (pending/passed/failed 事实)';
COMMENT ON TABLE defect.suggestions IS 'T: 智能建议草稿';
COMMENT ON TABLE defect.command_idempotency IS 'T: 命令幂等回执，24h TTL (expires_at)';
COMMENT ON TABLE defect.audit_events IS 'T: 审计事件，Append-only，禁 UPDATE/DELETE';
COMMENT ON TABLE defect.outbox IS 'T: Outbox 事件 (envelope)，at-least-once 投递语义';
COMMENT ON TABLE defect.field_values IS 'T: 版本化自定义字段值 (active/superseded)';
COMMENT ON TABLE defect.relations IS 'T: 缺陷关系 (parent/child/related/duplicate/blocks/blocked_by)，禁自引用';
COMMENT ON TABLE defect.comments IS 'T: 缺陷评论，逻辑删除 (active/deleted)';
COMMENT ON TABLE defect.drafts IS 'W: 会话草稿，有界大小 (1MiB) + expires_at TTL (7d 默认)';
COMMENT ON TABLE defect.status_events IS 'T: 状态/字段/负责人/关系/验证变化事件，Append-only';
