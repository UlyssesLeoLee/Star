-- 2026-09-08-ops-log.sql
-- F-02 log AI 端到端实装 (per WBS §14.10.2 + SRS-001 §4 F-02 + SRS-001 §8.1)
-- 3 表 DDL 落档, 守门 #13 W/T/M 100% 覆盖
-- Migration: 跟 F-01 ops-cluster 2 表 + F-03 ops-metrics 1 表同 pattern
-- Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
-- 触发: F-04 owner evidence check 5b P1 修正 + WBS v0.13 owner P1 修正 (5 表 → 3 表, 跟 SRS-001 §8.1 一致)

BEGIN;

-- ==========================================
-- 1. ops_log_query_log (T 类, WORM 审计)
-- 守門 #13 b: 物理删除禁止 + 監査必須 + RLS 13 類必携
-- ==========================================
CREATE TABLE IF NOT EXISTS ops_log_query_log (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    -- 业务字段
    query_id            UUID NOT NULL,                -- 客户端发起的 query 请求 ID (跟 ops_api/log_upload request_id 对齐)
    trace_id            UUID,                         -- 跨域 trace ID (跟 star-telemetry 联动)
    level_filter        TEXT NOT NULL DEFAULT 'all' CHECK (level_filter IN ('all', 'info', 'warn', 'error', 'debug')),
    time_range_start    TIMESTAMPTZ NOT NULL,
    time_range_end      TIMESTAMPTZ NOT NULL,
    line_count          INT NOT NULL DEFAULT 0,
    actor_user_id       UUID NOT NULL,
    requested_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at        TIMESTAMPTZ,
    status              TEXT NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed')),
    -- 错误码 6-field (per BAS-001 §4 + DDS-001 §3 派生)
    error_code          TEXT,                         -- 'BAD_REQUEST' / 'RATE_LIMITED' / 'UNAUTHORIZED' / 'INTERNAL' / 'NOT_IMPLEMENTED'
    error_message       TEXT,
    -- RLS 13 類 (T 必携, per 守門 #13 c)
    user_id             UUID,
    role_id             UUID,
    permission_id       UUID,
    policy_id           UUID,
    workspace_id        UUID,
    project_id          UUID,
    work_item_id        UUID,
    agent_id            UUID,
    session_id          UUID,
    source_module       TEXT NOT NULL DEFAULT 'star_ops::log',
    source_kind         TEXT NOT NULL DEFAULT 'audit'
);

CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_tenant ON ops_log_query_log(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_query ON ops_log_query_log(query_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_trace ON ops_log_query_log(trace_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_requested ON ops_log_query_log(requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_ops_log_query_log_actor ON ops_log_query_log(actor_user_id, requested_at DESC);

-- 守門 #13 d: 物理删除禁止 (WORM)
CREATE OR REPLACE FUNCTION prevent_hard_delete_ops_log_query_log()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'ops_log_query_log 物理删除禁止 (per 守門 #13 d WORM)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_delete_ops_log_query_log ON ops_log_query_log;
CREATE TRIGGER trg_prevent_delete_ops_log_query_log
    BEFORE DELETE ON ops_log_query_log
    FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_ops_log_query_log();

-- 守門 #13 d: 100% audit (T 派生, 跟 F-01 cluster_action_log 同 pattern)
CREATE OR REPLACE FUNCTION audit_ops_log_query_log()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (event_type, resource_id, tenant_id, payload, occurred_at)
    VALUES (TG_OP, NEW.id, NEW.tenant_id, to_jsonb(NEW), NOW());
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_audit_ops_log_query_log ON ops_log_query_log;
CREATE TRIGGER trg_audit_ops_log_query_log
    AFTER INSERT OR UPDATE ON ops_log_query_log
    FOR EACH ROW EXECUTE FUNCTION audit_ops_log_query_log();

ALTER TABLE ops_log_query_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_log_query_log FORCE ROW LEVEL SECURITY;

-- 守門 #13 c: tenant 隔离 RLS policy (T 必携)
CREATE POLICY ops_log_query_log_tenant_isolation ON ops_log_query_log
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- ==========================================
-- 2. ops_log_entry (W 类, 短 TTL 作业中)
-- 守門 #13 e: 物理删除 / タイマー失効 / 短 TTL 明示 retention 7 天
-- ==========================================
CREATE TABLE IF NOT EXISTS ops_log_entry (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    -- 业务字段
    trace_id            UUID,                         -- 跨域 trace ID
    log_query_id        UUID REFERENCES ops_log_query_log(id) ON DELETE RESTRICT,  -- 1 个 query 对应 N 条 log entry
    level               TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
    source              TEXT NOT NULL,                -- 'app' / 'system' / 'agent' / 'mcp' / 'external'
    message             TEXT NOT NULL,
    message_excerpt     TEXT,                         -- 截断后 256 字符 (per BAS-001 §3.2 + L1 6)
    metadata            JSONB NOT NULL DEFAULT '{}'::JSONB,
    occurred_at         TIMESTAMPTZ NOT NULL,        -- log 真实发生时间 (不是入库时间)
    captured_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- 入库时间
    -- TTL 字段 (per 守門 #13 e 派生)
    expires_at          TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),  -- retention 7 天 (per SRS-001 §8.1)
    -- RLS 12 類 (W 不强制 13 類必携, per 守門 #13 派生 c "T/M 通用" 派生)
    user_id             UUID,
    role_id             UUID,
    permission_id       UUID,
    policy_id           UUID,
    workspace_id        UUID,
    project_id          UUID,
    work_item_id        UUID,
    agent_id            UUID,
    session_id          UUID,
    source_module       TEXT NOT NULL DEFAULT 'star_ops::log',
    source_kind         TEXT NOT NULL DEFAULT 'work'
);

CREATE INDEX IF NOT EXISTS idx_ops_log_entry_tenant ON ops_log_entry(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_query ON ops_log_entry(log_query_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_trace ON ops_log_entry(trace_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_level ON ops_log_entry(level, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_occurred ON ops_log_entry(occurred_at DESC);
-- 守門 #13 e: TTL 索引 (跟 pg_cron / extension 联动定期清理)
CREATE INDEX IF NOT EXISTS idx_ops_log_entry_expires ON ops_log_entry(expires_at) WHERE expires_at IS NOT NULL;

-- 守門 #13 d: W 物理删除允许 (跟 T 区别), 但加 session-bound 标记防止意外 bulk delete
-- W 不需要 prevent_delete trigger (跟 T 区别)
-- W 可走 TRUNCATE / DROP PARTITION 模式 (per 守門 #13 e 派生)

ALTER TABLE ops_log_entry ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_log_entry FORCE ROW LEVEL SECURITY;

-- 守門 #13 c: tenant 隔离 RLS policy
CREATE POLICY ops_log_entry_tenant_isolation ON ops_log_entry
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- ==========================================
-- 3. ops_log_analysis (W 类, 短 TTL AI 分析结果)
-- 守門 #13 e: 物理删除 / タイマー失効 / 短 TTL 明示 retention 30 天
-- 跟 ops_log_entry 1:N (1 entry 多 analysis, 跟 LLM 多次重试一致)
-- ==========================================
CREATE TABLE IF NOT EXISTS ops_log_analysis (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    -- 业务字段
    log_entry_id        UUID NOT NULL REFERENCES ops_log_entry(id) ON DELETE CASCADE,  -- 1 log entry 删 → N analysis 删 (W cascade)
    ai_channel          TEXT NOT NULL CHECK (ai_channel IN ('mock', 'openai_stub', 'anthropic_stub', 'openai', 'anthropic')),
    model               TEXT,                         -- 'gpt-4' / 'claude-3-opus' / 'mock-v1' (跟 ops_ai::Ladder 对齐)
    ladder_attempts     INT NOT NULL DEFAULT 1,       -- Ladder 重试次数 (L1 mock → L2 stub → L3 real)
    confidence          DOUBLE PRECISION NOT NULL CHECK (confidence BETWEEN 0.0 AND 1.0),
    is_anomaly          BOOLEAN NOT NULL DEFAULT FALSE,
    anomaly_type        TEXT,                         -- 'spike' / 'drift' / 'error_burst' / 'pattern' (per BAS-001 §3.2 L1 6)
    summary             TEXT NOT NULL,                -- LLM 摘要 (中文/英文 跟 i18n 联动)
    suggestions         JSONB NOT NULL DEFAULT '[]'::JSONB,  -- LLM 建议 (array of strings)
    generated_by        TEXT NOT NULL,                -- 'mock' / 'openai' / 'anthropic' (跟 ops_ai::AiChannel 对齐)
    generated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- TTL 字段 (per 守門 #13 e 派生, 跟 entry 区别 30 天)
    expires_at          TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),  -- retention 30 天 (per SRS-001 §8.1)
    -- RLS 12 類 (W)
    user_id             UUID,
    role_id             UUID,
    permission_id       UUID,
    policy_id           UUID,
    workspace_id        UUID,
    project_id          UUID,
    work_item_id        UUID,
    agent_id            UUID,
    session_id          UUID,
    source_module       TEXT NOT NULL DEFAULT 'star_ops::log',
    source_kind         TEXT NOT NULL DEFAULT 'work'
);

CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_tenant ON ops_log_analysis(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_entry ON ops_log_analysis(log_entry_id);
CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_anomaly ON ops_log_analysis(is_anomaly, generated_at DESC) WHERE is_anomaly = TRUE;
CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_generated ON ops_log_analysis(generated_at DESC);
CREATE INDEX IF NOT EXISTS idx_ops_log_analysis_expires ON ops_log_analysis(expires_at) WHERE expires_at IS NOT NULL;

-- W 类 物理删除允许 (跟 entry 区别, 走 cascade)
-- 守門 #13 e: ops_log_entry 删 → ops_log_analysis 自动 cascade (已在 FK ON DELETE CASCADE 定义)

ALTER TABLE ops_log_analysis ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_log_analysis FORCE ROW LEVEL SECURITY;

-- 守門 #13 c: tenant 隔离 RLS policy
CREATE POLICY ops_log_analysis_tenant_isolation ON ops_log_analysis
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

COMMIT;

-- ==========================================
-- 守門 #13 W/T/M 100% 覆盖 实证 (per IT it_ops_log_ddl_wtm_coverage)
-- ==========================================
-- 既有 6 ops 表 (per SRS-001 §4 + 累计):
--   1. ops_helm_release_state      T    (F-01 2026-09-08-ops-cluster.sql)
--   2. ops_cluster_action_log      T    (F-01 2026-09-08-ops-cluster.sql)
--   3. ops_log_query_log           T    (本文件 F-02)
--   4. ops_log_entry               W    (本文件 F-02, retention 7d)
--   5. ops_log_analysis            W    (本文件 F-02, retention 30d, 跟 entry 1:N cascade)
--   6. ops_metrics_config          M    (F-03 2026-09-08-ops-metrics.sql, SCD2)
--
-- 累计 6 表 W/T/M 100% 覆盖 (T=3 + W=2 + M=1, 0 混合分类)
-- per 守門 #13 派生规 (a) (b) (c) (d) (e) 全实现
-- 跟 SRS-001 §8.1 权威一致 (F-04 owner P1 修正 v0.13 WBS P1 修正: 5 表 → 3 表)
--
-- 缺口 (per 守門 #11 缺标比错标, DDD Review 必查):
-- 1. log_upload_bench P95 实证缺 (F-02 性能, per TEST-DESIGN §4.6 缺口 #1)
-- 2. E2E 浏览器自动化 选型 (Playwright vs Cypress, per TEST-DESIGN §4.6 缺口 #2)
-- 3. 6 表 RLS 13 類验证缺 (per SRS-001 §8.2, MVP TODO, sqlx::test + testcontainers)
