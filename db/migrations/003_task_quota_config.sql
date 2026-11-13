-- 003_task_quota_config.sql (WBS 6.3 Sub-task Binding, Master SCD Type 2)
--
-- task_quota_config 表: 父任务 sub-task 配额配置 (per docs/architecture/2026-09-09-subtask-binding/
-- 03-detailed-design.md §5 Table 3 + 01-requirements.md §2 F-31 + §4 W/T/M 横展開)
--
-- 分类: Master (参考, 慢变, SCD Type 2)
--   - 物理删除禁止 (per 守门 #13 c)
--   - 审计必携 + RLS 13 类必携 (本表: tenant_id + parent_task_id 2 维必携, per 01 §4)
--   - SCD Type 2: close_old_quota_config 触发器 (BEFORE INSERT OR UPDATE 自动 close 旧行)
-- 默认值 (per F-31 + NFR-SB-04): max_subtasks=2, per_subtask_token_budget=100000
-- 语义对齐: scripts/automation/task_ops/compat.py QuotaConfigStore (内存版, 同 SCD 语义;
-- PostgreSQL Tier 3 待 5 域 Lead 真人到位, per 03-detailed §7 新增风险).

BEGIN;

CREATE TABLE IF NOT EXISTS task_quota_config (
    config_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_task_id UUID NOT NULL,
    max_subtasks INT NOT NULL DEFAULT 2,
    current_subtasks INT NOT NULL DEFAULT 0,
    per_subtask_token_budget BIGINT NOT NULL DEFAULT 100000,
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ,  -- SCD Type 2 end (NULL = current)
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by TEXT NOT NULL,
    -- W/T/M: Master, 物理删除禁止, SCD Type 2, RLS 13 類必携 (per 守门 #13 c/d)
    tenant_id UUID NOT NULL,
    CONSTRAINT chk_scd_type2 CHECK (
        (is_current = TRUE AND effective_to IS NULL) OR
        (is_current = FALSE AND effective_to IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_task_quota_config_current
    ON task_quota_config(parent_task_id) WHERE is_current = TRUE;

-- 触发器: SCD Type 2 自动 close 旧 row (per DD §5)
CREATE OR REPLACE FUNCTION close_old_quota_config()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_current = TRUE THEN
        UPDATE task_quota_config
        SET effective_to = NOW(), is_current = FALSE
        WHERE parent_task_id = NEW.parent_task_id
          AND is_current = TRUE
          AND config_id != NEW.config_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_close_old_quota_config ON task_quota_config;
CREATE TRIGGER trg_close_old_quota_config
BEFORE INSERT OR UPDATE ON task_quota_config
FOR EACH ROW EXECUTE FUNCTION close_old_quota_config();

-- RLS (per 守门 #13 RLS 13 類, per DD §5)
ALTER TABLE task_quota_config ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_quota_config FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS task_quota_config_tenant_isolation ON task_quota_config;
CREATE POLICY task_quota_config_tenant_isolation ON task_quota_config
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

COMMIT;
