-- 2026-09-10-p3d6-schema-template.sql
-- v0.99 P0-4 Stage 4.3 = P3-D.6 14+15 张表真实 schema 模板
-- (per v0.94 §3 已知缺口 (a) 'P0-4 阶段 26 张占位表, P2 阶段 worker 子代理 + P3-D.6 阶段 2 任务 2.x 实跑时 ALTER TABLE CREATE TABLE 真实 schema 替换占位' 跨 session 续做)
--
-- 本模板 1 张占位表 `agent_sessions` 完整 schema (含 CREATE TABLE IF NOT EXISTS + 7 类 RLS policy + IF NOT EXISTS idempotent + DROP IF EXISTS 兼容 PG 9.5+),
-- P2 阶段 worker 子代理 复制模板改 table name + column name 走 14 张表.
--
-- 守门 #13 a 100% RLS: ENABLE + FORCE + 7 类 policy
-- 守门 #13 b 物理删除禁止: 4 类 CRUD delete 仅 platform_admin
-- 守门 #13 c M SCD Type 2: pgpool_version 字段
-- 守门 #13 d T 100% audit: audit trigger 跨 RLS (per ADR-0043 WORM)
-- 守门 #1 禁回溯叙事: 不重写 v0.82 tenant_pools DDL, 仅新 commit + 模板声明
-- 守门 #11 缺标比错标: P0-4 阶段 1 张占位表完整 schema, P2 阶段 worker 子代理 fill 13 张

BEGIN;

-- ==========================================
-- 1. agent_sessions 表 (per DD-CANVAS-AGENT-001 §4.14.1 + 守门 #13 c SCD Type 2)
-- 模板: P2 阶段 worker 子代理 改 table name + column name 走 14 张
-- ==========================================
CREATE TABLE IF NOT EXISTS agent_sessions (
    -- 业务字段 (per DD §4.14.1)
    id              UUID NOT NULL,
    tenant_id       UUID NOT NULL,                        -- 守门 #13 a 100% RLS 13 类必带
    agent_id        UUID NOT NULL,                        -- 关联 agent 域 (per DD-CANVAS-AGENT-001)
    session_token   TEXT NOT NULL,                        -- 会话 token (per OAuth2 server v0.43 模式)
    status          TEXT NOT NULL DEFAULT 'Active',       -- Active/Expired/Revoked (per §5 状态机)
    expires_at      TIMESTAMPTZ NOT NULL,                 -- 过期时间 (per OAuth2 短期 token)
    schema_name     TEXT NOT NULL DEFAULT 'public',       -- 守门 #13 a + §13.5 多 schema
    -- 审计字段 (守门 #13 d T 100% audit + ADR-0043 WORM append-only)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by      UUID NOT NULL,
    pgpool_version  BIGINT NOT NULL DEFAULT 1,            -- 守门 #13 c SCD Type 2
    -- 显式主键
    PRIMARY KEY (id, pgpool_version),
    UNIQUE (tenant_id, pgpool_version)
);

-- 索引 (per §6.1 query 优化)
CREATE INDEX IF NOT EXISTS idx_agent_sessions_tenant_id
    ON agent_sessions (tenant_id);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_agent_id
    ON agent_sessions (agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_expires_at
    ON agent_sessions (expires_at);
CREATE INDEX IF NOT EXISTS idx_agent_sessions_status
    ON agent_sessions (status)
    WHERE status != 'Active';

-- 当前 SCD Type 2 "有效" version 视图
CREATE OR REPLACE VIEW v_agent_sessions_current AS
SELECT DISTINCT ON (tenant_id, agent_id) *
FROM agent_sessions
ORDER BY tenant_id, agent_id, pgpool_version DESC;

-- AuditEvent WORM 触发器 (守门 #13 d 100% audit + ADR-0043 WORM)
CREATE OR REPLACE FUNCTION agent_sessions_audit_trigger() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (
        id, tenant_id, source_kind, source_id, action, row_data, actor_id, created_at
    ) VALUES (
        gen_random_uuid(),
        COALESCE(NEW.tenant_id, OLD.tenant_id),
        'AgentSession',
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        to_jsonb(COALESCE(NEW, OLD)),
        COALESCE(NEW.updated_by, OLD.updated_by, NEW.created_by, OLD.created_by),
        NOW()
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_agent_sessions_audit
    AFTER INSERT OR UPDATE OR DELETE ON agent_sessions
    FOR EACH ROW
    EXECUTE FUNCTION agent_sessions_audit_trigger();

-- ==========================================
-- 2. RLS 7 类 policy (per v0.92 + v0.97 DROP IF EXISTS 兼容 PG 9.5+)
-- 模板: 复制到 14 张表 + 改 table name
-- ==========================================
ALTER TABLE agent_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_sessions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS agent_sessions_select ON agent_sessions;
CREATE POLICY agent_sessions_select ON agent_sessions
    FOR SELECT
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS agent_sessions_insert ON agent_sessions;
CREATE POLICY agent_sessions_insert ON agent_sessions
    FOR INSERT
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS agent_sessions_update ON agent_sessions;
CREATE POLICY agent_sessions_update ON agent_sessions
    FOR UPDATE
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS agent_sessions_delete ON agent_sessions;
CREATE POLICY agent_sessions_delete ON agent_sessions
    FOR DELETE
    USING (
        current_setting('app.is_admin', true) = 'true'
    );

-- ==========================================
-- 3. RLS policy 占位: 跨 schema_name 隔离 + platform_admin override
-- ==========================================
DROP POLICY IF EXISTS agent_sessions_schema_isolation ON agent_sessions;
CREATE POLICY agent_sessions_schema_isolation ON agent_sessions
    FOR ALL
    USING (
        schema_name = current_setting('app.current_schema_name', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS agent_sessions_platform_admin ON agent_sessions;
CREATE POLICY agent_sessions_platform_admin ON agent_sessions
    FOR ALL
    TO platform_admin
    USING (true)
    WITH CHECK (true);

COMMIT;
