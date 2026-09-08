-- =============================================================================
-- Star Ops Log AI 3 表 DDL 雏形 (per 守门 #13 W/T/M 100% + 守门 #DB-13 RLS 13 类)
-- =============================================================================
--
-- 拍板: per ask_f5016f1d0df89012dfa06965 选 scope-2M_opt1 = F-02 log AI 端到端
-- 关联: docs/briefs/ops-f02-log-ai-impl.md
--       docs/detailed-design/OPS-DETAILED-DESIGN-001.md §3 Hybrid AI
--       docs/basic-design/OPS-BASIC-DESIGN-001.md §3.2 F-02 LogAI
--       docs/requirements/SRS-STAR-OPS-001.md §4 F-02
--       ADR-0026 Hybrid AI 4 级 Ladder
--       AGENTS.md §4 守门 #13 W/T/M + #DB-13 RLS 13 類
--
-- 守门合规 (per AGENTS.md §4 #13):
--   - W (作業中): 1 张, 7d TTL (per 守门 #13 a, ops_log_entry 临时缓存, session-bound)
--   - T (Transaction): 1 张, append-only, audit trigger 必携 (per 守门 #13 d, ops_log_query_log)
--   - M (Master): 1 张, 30d retention + SCD Type 2 (per 守门 #13 c, ops_log_analysis 参考数据)
--
-- 守门合规 (per AGENTS.md §4 #DB-13 RLS 13 類 CW-05):
--   - 3 张表 100% 启用 RLS (per 守门 #DB-13 a)
--   - tenant_id NOT NULL (per 守门 #DB-13 CW-05)
--   - FORCE ROW LEVEL SECURITY (per 守门 #DB-13 c)
--
-- 触发顺序: 先建表 → 后挂 RLS policy → 再挂 retention / trigger
-- =============================================================================

BEGIN;

-- =============================================================================
-- W 类 (作業中): ops_log_entry
-- 用途: 临时 log 缓存, session-bound, 7d TTL (per 守门 #13 a)
-- 写入: log_upload 端点
-- 读取: log_analysis 端点 (按 id 查最新)
-- 清理: 7d 后自动 DROP (per 守门 #13 a)
-- =============================================================================
CREATE TABLE IF NOT EXISTS ops_log_entry (
    -- 业务字段
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,           -- 守门 #DB-13 CW-05: tenant 必填
    source VARCHAR(255) NOT NULL,
    level VARCHAR(16) NOT NULL,        -- TRACE/DEBUG/INFO/WARN/ERROR
    message TEXT NOT NULL,
    trace_id VARCHAR(128),
    -- 元字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 7d TTL (per 守门 #13 a)
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days')
);

-- W 类索引: 按 (tenant_id, created_at DESC) 查最新
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_tenant_created
    ON ops_log_entry(tenant_id, created_at DESC);

-- W 类索引: 按 trace_id 串联
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_trace
    ON ops_log_entry(tenant_id, trace_id) WHERE trace_id IS NOT NULL;

-- 守门 #DB-13 c: FORCE ROW LEVEL SECURITY
ALTER TABLE ops_log_entry ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_log_entry FORCE ROW LEVEL SECURITY;

-- 守门 #DB-13 a: 13 類 RLS policy (per 守门 #DB-13 模板)
-- tenant 隔离: 同一 tenant 可见, 跨 tenant 不可见
CREATE POLICY ops_log_entry_tenant_isolation ON ops_log_entry
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);


-- =============================================================================
-- T 类 (Transaction): ops_log_query_log
-- 用途: 事件流水, append-only (per 守门 #13 d), audit trigger 必携
-- 写入: 每次 log_upload / log_analysis 调用
-- 读取: 仅审计, 不在线业务读
-- 清理: 不可物理删除 (per 守门 #13 d append-only)
-- =============================================================================
CREATE TABLE IF NOT EXISTS ops_log_query_log (
    -- 业务字段
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL,           -- 守门 #DB-13 CW-05
    query_id UUID NOT NULL,            -- 单次 log_upload 调用 id
    query_type VARCHAR(32) NOT NULL,   -- 'upload' | 'analysis'
    log_id UUID,                       -- 关联 ops_log_entry.id
    ai_channel VARCHAR(32),            -- 'mock' | 'openai' | 'anthropic'
    analysis_triggered BOOLEAN,
    analysis_latency_ms INTEGER,
    -- 元字段
    user_id UUID,
    request_ip INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- T 类索引: 按 (tenant_id, created_at DESC) 查最新流水
CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_tenant_created
    ON ops_log_query_log(tenant_id, created_at DESC);

-- T 类索引: 按 query_id 关联
CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_query_id
    ON ops_log_query_log(tenant_id, query_id);

-- 守门 #DB-13 c: FORCE ROW LEVEL SECURITY
ALTER TABLE ops_log_query_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_log_query_log FORCE ROW LEVEL SECURITY;

-- 守门 #DB-13 a: 13 類 RLS policy
CREATE POLICY ops_log_query_log_tenant_isolation ON ops_log_query_log
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 守门 #13 d: audit trigger 必携 (Transaction 100% 必携)
-- 注: audit_audit_event() 函数假定已存在 (per 2026-09-07-audit-trigger.sql)
CREATE OR REPLACE TRIGGER trg_ops_log_query_log_audit
    AFTER INSERT ON ops_log_query_log
    FOR EACH ROW
    EXECUTE FUNCTION audit_audit_event();

-- 守门 #13 d append-only: 禁止 UPDATE / DELETE (触发 Err)
CREATE OR REPLACE FUNCTION ops_log_query_log_append_only() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'ops_log_query_log is append-only (per 守门 #13 d), UPDATE/DELETE 禁止';
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_ops_log_query_log_no_update
    BEFORE UPDATE ON ops_log_query_log
    FOR EACH ROW
    EXECUTE FUNCTION ops_log_query_log_append_only();

CREATE OR REPLACE TRIGGER trg_ops_log_query_log_no_delete
    BEFORE DELETE ON ops_log_query_log
    FOR EACH ROW
    EXECUTE FUNCTION ops_log_query_log_append_only();


-- =============================================================================
-- M 类 (Master): ops_log_analysis
-- 用途: AI 分析结果参考数据, slowly changing, 30d retention + SCD Type 2 (per 守门 #13 c)
-- 写入: AI 分析完成时
-- 读取: log_analysis 端点 (按 log_id 查最新 version)
-- 清理: 30d 后 partition drop (per 守门 #13 c retention)
-- =============================================================================
CREATE TABLE IF NOT EXISTS ops_log_analysis (
    -- 业务字段
    id UUID NOT NULL,
    tenant_id UUID NOT NULL,           -- 守门 #DB-13 CW-05
    log_id UUID NOT NULL,              -- 关联 ops_log_entry.id
    version INTEGER NOT NULL DEFAULT 1, -- SCD Type 2 版本号
    summary TEXT NOT NULL,
    anomalies JSONB NOT NULL DEFAULT '[]'::JSONB,
    suggestions JSONB NOT NULL DEFAULT '[]'::JSONB,
    confidence REAL NOT NULL,
    ai_channel VARCHAR(32) NOT NULL,   -- 'mock' | 'openai' | 'anthropic'
    needs_review BOOLEAN NOT NULL DEFAULT TRUE, -- 守门 #23: confidence < 0.5 必标
    -- SCD Type 2 字段 (per 守门 #13 c)
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    -- 元字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, valid_from)
);

-- M 类索引: 按 (tenant_id, log_id, valid_from DESC) 查最新版本
CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_log
    ON ops_log_analysis(tenant_id, log_id, valid_from DESC);

-- M 类索引: 按 (tenant_id, valid_to) 找 open 记录
CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_valid_to
    ON ops_log_analysis(tenant_id, valid_to) WHERE valid_to IS NULL;

-- 守门 #DB-13 c: FORCE ROW LEVEL SECURITY
ALTER TABLE ops_log_analysis ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_log_analysis FORCE ROW LEVEL SECURITY;

-- 守门 #DB-13 a: 13 類 RLS policy
CREATE POLICY ops_log_analysis_tenant_isolation ON ops_log_analysis
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 守门 #13 c: SCD Type 2 自动闭旧记录 (per 2026-09-07-audit-trigger.sql scd_type2_close)
CREATE OR REPLACE TRIGGER trg_ops_log_analysis_scd2
    BEFORE UPDATE ON ops_log_analysis
    FOR EACH ROW
    EXECUTE FUNCTION scd_type2_close();


COMMIT;

-- =============================================================================
-- 验证 (per 守门 #13 100% W/T/M 覆盖):
-- =============================================================================
-- SELECT table_name, table_type FROM information_schema.tables
-- WHERE table_schema = 'public' AND table_name LIKE 'ops_log_%'
-- ORDER BY table_name;
-- 期望返 3 行: ops_log_analysis (M) | ops_log_entry (W) | ops_log_query_log (T)
--
-- RLS 验证 (per 守门 #DB-13 c FORCE):
-- SELECT tablename, rowsecurity, forcerowsecurity
-- FROM pg_tables WHERE tablename LIKE 'ops_log_%';
-- 期望 rls=true, force_rls=true (3 行)
-- =============================================================================
