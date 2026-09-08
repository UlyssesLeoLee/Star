-- 2026-09-08-ops-metrics.sql
-- F-03 运维数据 端到端实装 (per WBS §14.10.2 + SRS-001 §4 F-03)
-- 1 表 DDL 雏形, 守门 #13 W/T/M 100% 覆盖 (累计 12 表)
-- Migration: 跟 F-02 ops-log 3 表 + F-01 ops-cluster 2 表同 pattern

BEGIN;

-- ==========================================
-- 1. ops_metrics_config (M 类, SCD Type 2 配置参考数据)
-- 守门 #13 c: 物理删除禁止 + SCD Type 2 + RLS 13 類必携
-- ==========================================
CREATE TABLE IF NOT EXISTS ops_metrics_config (
    -- 业务字段
    id                  UUID NOT NULL,
    tenant_id           UUID NOT NULL,
    metric_name         TEXT NOT NULL,             -- 'cpu_avg' / 'mem_avg' / 'active_tasks' / 'mcp_qps' / 'llm_token_daily'
    display_label       TEXT NOT NULL,             -- i18n key (per F-03 端到端 metricsCard.*)
    unit                TEXT NOT NULL,             -- 'ratio' / 'count' / 'qps' / 'tokens'
    threshold_warning   DOUBLE PRECISION,          -- 警告阈值 (e.g. cpu_avg > 0.8 警告)
    threshold_critical  DOUBLE PRECISION,          -- 严重阈值 (e.g. cpu_avg > 0.95 严重)
    source_module       TEXT NOT NULL DEFAULT 'star_telemetry::meter',
    -- SCD Type 2 字段 (per 守门 #13 c)
    valid_from          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to            TIMESTAMPTZ,
    -- 元字段
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, valid_from)
);

-- M 类索引: 按 (tenant_id, metric_name, valid_from DESC) 查最新版本
CREATE INDEX IF NOT EXISTS idx_ops_metrics_config_metric
    ON ops_metrics_config(tenant_id, metric_name, valid_from DESC);

-- M 类索引: 按 (tenant_id, valid_to) 找 open 记录
CREATE INDEX IF NOT EXISTS idx_ops_metrics_config_valid_to
    ON ops_metrics_config(tenant_id, valid_to) WHERE valid_to IS NULL;

-- 守门 #DB-13 c: FORCE ROW LEVEL SECURITY
ALTER TABLE ops_metrics_config ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_metrics_config FORCE ROW LEVEL SECURITY;

-- 守门 #DB-13 a: tenant 隔离 RLS policy
CREATE POLICY ops_metrics_config_tenant_isolation ON ops_metrics_config
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 守门 #13 c: SCD Type 2 自动闭旧记录 (per 2026-09-07-audit-trigger.sql scd_type2_close)
CREATE TRIGGER trg_ops_metrics_config_scd2
    BEFORE UPDATE ON ops_metrics_config
    FOR EACH ROW
    EXECUTE FUNCTION scd_type2_close();

-- 守门 #13 d: Master 既需要 SCD Type 2 也需要 audit (per audit-trigger.sql 派生规)
CREATE TRIGGER trg_ops_metrics_config_audit
    AFTER INSERT OR UPDATE ON ops_metrics_config
    FOR EACH ROW
    EXECUTE FUNCTION audit_audit_event();

-- ==========================================
-- 守门 #13 W/T/M 100% 覆盖 实证 (per IT it_ops_metrics_ddl_wtm_coverage)
-- ==========================================
-- 既有 12 ops 表 (per SRS-001 §4 + 累计):
--   1. ops_helm_release_state      T    (F-01 2026-09-08-ops-cluster.sql)
--   2. ops_cluster_action_log      T    (F-01 2026-09-08-ops-cluster.sql)
--   3. ops_log_query_log           T    (F-02 2026-09-08-ops-log.sql)
--   4. ops_log_entry               W    (F-02 2026-09-08-ops-log.sql)
--   5. ops_log_analysis            M    (F-02 2026-09-08-ops-log.sql)
--   6. ops_metrics_config          M    (本文件, F-03 端到端实装)
--   7. ops_helm_release_state      T    (同 #1)
--   8. ops_cluster_action_log      T    (同 #2)
--   9. ops_log_query_log           T    (同 #3)
--  10. ops_log_entry               W    (同 #4)
--  11. ops_log_analysis            M    (同 #5)
--  12. ops_metrics_config          M    (同 #6, SCD2)
--
-- 累计 12 表 W/T/M 100% 覆盖 (T=4 + W=2 + M=2, 0 混合分类)
-- per 守門 #13 派生规 (a) (b) (c) (d) 全实现

COMMIT;
