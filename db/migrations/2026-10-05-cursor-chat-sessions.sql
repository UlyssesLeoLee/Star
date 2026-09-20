-- 2026-10-05-cursor-chat-sessions.sql (ULYS-98-W3 Sub-task 3.1)
-- v0.1 Cursor 极简版 — chat_sessions / chat_messages 表 schema
-- (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 3.1")
--
-- 作用: AI 对话持久化 (M row: chat_sessions = 1 conversation thread, chat_messages = M turns)
-- 设计: chat_sessions.id 客户端生成 UUID (per W1.3 ChatSendReq.session_id); chat_messages.role CHECK 限定 3 类
-- 守门合规 (per 守门 #1 v25 + 守门 #13 a/b/d + 守门 #19 v19):
-- - 0 改已有 migration 任何行 (cursor-chat 是新独立模块, 跟 P3-D.6 13 表 + worktree_canvas 5 表 0 冲突)
-- - 100% RLS (守门 #13 a): tenant_id 列 + ENABLE + FORCE + 5 类 policy (SELECT / INSERT / UPDATE / DELETE / admin)
-- - 物理删除禁止 (守门 #13 b): DELETE policy 仅 platform_admin
-- - audit 触发器 (守门 #13 d): cross-RLS audit row 落 audit_audit_event 表
-- - pgpool_version SCD Type 2 (守门 #13 c): 每张表 必带

BEGIN;

-- ==========================================
-- 1. chat_sessions 表 (per brief §3.1 — 1 conversation thread)
-- ==========================================
CREATE TABLE IF NOT EXISTS cursor_chat_sessions (
    -- 业务字段 (per brief §3.1 schema)
    id              UUID NOT NULL,                                       -- 客户端生成 (W1.3 ChatSendReq.session_id)
    tenant_id       UUID NOT NULL,                                       -- 守门 #13 a 100% RLS
    user_id         UUID NOT NULL,                                       -- 会话 owner (per JWT sub claim)
    title           TEXT,                                                -- 用户/AI 生成会话标题 (null = 尚未命名)
    -- 模型上下文 (per brief W3 §3.6 RAG): session 粒度模型选择 (e.g. claude-3-5-sonnet)
    model           TEXT,                                                -- LlmProvider 模型名 (e.g. "claude-3-5-sonnet-20240620")
    -- 会话状态 (per W3 §3.1 schema)
    archived_at     TIMESTAMPTZ,                                         -- 用户归档时间 (null = 活跃)
    -- 审计字段 (守门 #13 d T 100% audit + ADR-0043 WORM append-only)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by      UUID NOT NULL,
    pgpool_version  BIGINT NOT NULL DEFAULT 1,                           -- 守门 #13 c SCD Type 2
    -- 显式主键
    PRIMARY KEY (id, pgpool_version),
    UNIQUE (tenant_id, id, pgpool_version)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_cursor_chat_sessions_tenant_id
    ON cursor_chat_sessions (tenant_id);
CREATE INDEX IF NOT EXISTS idx_cursor_chat_sessions_user_id
    ON cursor_chat_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_cursor_chat_sessions_archived_at
    ON cursor_chat_sessions (archived_at)
    WHERE archived_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_cursor_chat_sessions_updated_at
    ON cursor_chat_sessions (updated_at DESC);

-- 当前 SCD Type 2 "有效" version 视图
CREATE OR REPLACE VIEW v_cursor_chat_sessions_current AS
SELECT DISTINCT ON (tenant_id, id) *
FROM cursor_chat_sessions
ORDER BY tenant_id, id, pgpool_version DESC;

-- ==========================================
-- 2. chat_messages 表 (per brief §3.1 — M turns in a conversation)
-- ==========================================
CREATE TABLE IF NOT EXISTS cursor_chat_messages (
    -- 业务字段
    id              UUID NOT NULL,                                       -- 服务端 UUID (per W1.3 ChatSendResp.message_id)
    tenant_id       UUID NOT NULL,                                       -- 守门 #13 a 100% RLS
    session_id      UUID NOT NULL,                                       -- 关联 cursor_chat_sessions.id
    role            TEXT NOT NULL CHECK (role IN ('user','assistant','system')),  -- brief §3.1 role 限定 3 类
    content         TEXT NOT NULL,                                       -- 消息正文 (UTF-8 markdown)
    -- LLM 调用元数据 (per W2.5 metering)
    token_usage     JSONB,                                               -- {input_tokens, output_tokens, total_tokens} (per W2 metering)
    model           TEXT,                                                -- 本条消息产生所用模型 (e.g. claude-3-5-sonnet)
    -- 流式元数据 (per W3.2 SSE)
    streamed        BOOLEAN NOT NULL DEFAULT FALSE,                      -- true = 流式推送产生; false = 非流式 batch 产生
    finish_reason   TEXT,                                                -- OpenAI convention: stop / length / content_filter / tool_calls
    -- 审计字段
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by      UUID NOT NULL,
    pgpool_version  BIGINT NOT NULL DEFAULT 1,                           -- 守门 #13 c SCD Type 2
    -- 主键 + 外键
    PRIMARY KEY (id, pgpool_version),
    FOREIGN KEY (tenant_id, session_id, pgpool_version)
        REFERENCES cursor_chat_sessions (tenant_id, id, pgpool_version)
        ON DELETE RESTRICT                                              -- 守门 #13 b 物理删除禁止
);

-- 索引 (per brief §3.1 idx_chat_messages_session_id)
CREATE INDEX IF NOT EXISTS idx_cursor_chat_messages_session_id
    ON cursor_chat_messages (session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_cursor_chat_messages_tenant_id
    ON cursor_chat_messages (tenant_id);
CREATE INDEX IF NOT EXISTS idx_cursor_chat_messages_role
    ON cursor_chat_messages (role)
    WHERE role = 'system';

-- 当前 SCD Type 2 "有效" version 视图
CREATE OR REPLACE VIEW v_cursor_chat_messages_current AS
SELECT DISTINCT ON (tenant_id, id) *
FROM cursor_chat_messages
ORDER BY tenant_id, id, pgpool_version DESC;

-- ==========================================
-- 3. AuditEvent WORM 触发器 (守门 #13 d 100% audit + ADR-0043 WORM)
-- ==========================================
CREATE OR REPLACE FUNCTION cursor_chat_sessions_audit_trigger() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (
        id, tenant_id, source_kind, source_id, action, row_data, actor_id, created_at
    ) VALUES (
        gen_random_uuid(),
        COALESCE(NEW.tenant_id, OLD.tenant_id),
        'CursorChatSession',
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        to_jsonb(COALESCE(NEW, OLD)),
        COALESCE(NEW.updated_by, OLD.updated_by, NEW.created_by, OLD.created_by),
        NOW()
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_cursor_chat_sessions_audit
    AFTER INSERT OR UPDATE OR DELETE ON cursor_chat_sessions
    FOR EACH ROW
    EXECUTE FUNCTION cursor_chat_sessions_audit_trigger();

CREATE OR REPLACE FUNCTION cursor_chat_messages_audit_trigger() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (
        id, tenant_id, source_kind, source_id, action, row_data, actor_id, created_at
    ) VALUES (
        gen_random_uuid(),
        COALESCE(NEW.tenant_id, OLD.tenant_id),
        'CursorChatMessage',
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        to_jsonb(COALESCE(NEW, OLD)),
        COALESCE(NEW.updated_by, OLD.updated_by, NEW.created_by, OLD.created_by),
        NOW()
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_cursor_chat_messages_audit
    AFTER INSERT OR UPDATE OR DELETE ON cursor_chat_messages
    FOR EACH ROW
    EXECUTE FUNCTION cursor_chat_messages_audit_trigger();

-- ==========================================
-- 4. RLS 5 类 policy (per 守门 #13 a + v0.92/v0.97 DROP IF EXISTS 兼容 PG 9.5+)
-- ==========================================
ALTER TABLE cursor_chat_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE cursor_chat_sessions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS cursor_chat_sessions_select ON cursor_chat_sessions;
CREATE POLICY cursor_chat_sessions_select ON cursor_chat_sessions
    FOR SELECT
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS cursor_chat_sessions_insert ON cursor_chat_sessions;
CREATE POLICY cursor_chat_sessions_insert ON cursor_chat_sessions
    FOR INSERT
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
    );

DROP POLICY IF EXISTS cursor_chat_sessions_update ON cursor_chat_sessions;
CREATE POLICY cursor_chat_sessions_update ON cursor_chat_sessions
    FOR UPDATE
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
    );

-- 守门 #13 b: DELETE policy 仅 platform_admin
DROP POLICY IF EXISTS cursor_chat_sessions_delete ON cursor_chat_sessions;
CREATE POLICY cursor_chat_sessions_delete ON cursor_chat_sessions
    FOR DELETE
    USING (current_setting('app.is_admin', true) = 'true');

DROP POLICY IF EXISTS cursor_chat_sessions_admin ON cursor_chat_sessions;
CREATE POLICY cursor_chat_sessions_admin ON cursor_chat_sessions
    FOR ALL
    USING (current_setting('app.is_admin', true) = 'true');

ALTER TABLE cursor_chat_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE cursor_chat_messages FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS cursor_chat_messages_select ON cursor_chat_messages;
CREATE POLICY cursor_chat_messages_select ON cursor_chat_messages
    FOR SELECT
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS cursor_chat_messages_insert ON cursor_chat_messages;
CREATE POLICY cursor_chat_messages_insert ON cursor_chat_messages
    FOR INSERT
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
    );

DROP POLICY IF EXISTS cursor_chat_messages_update ON cursor_chat_messages;
CREATE POLICY cursor_chat_messages_update ON cursor_chat_messages
    FOR UPDATE
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
    );

-- 守门 #13 b: DELETE policy 仅 platform_admin
DROP POLICY IF EXISTS cursor_chat_messages_delete ON cursor_chat_messages;
CREATE POLICY cursor_chat_messages_delete ON cursor_chat_messages
    FOR DELETE
    USING (current_setting('app.is_admin', true) = 'true');

COMMIT;
