-- 002_sub_task_runtime.sql (WBS 6.3 Sub-task Binding, Work 短 TTL)
--
-- sub_task_runtime 表: 子任务运行时状态 (per docs/architecture/2026-09-09-subtask-binding/
-- 03-detailed-design.md §5 Table 2 + 01-requirements.md §4 W/T/M 横展開)
--
-- 分类: Work (短 TTL, 完成后清理)
--   - 物理删除允许 (TTL 失効, retention_period = 24h, per 守门 #13 a 派生规)
--   - 不需 audit, 不需 RLS (per 01 §4: 简化 tenant_id + device_id 2 维)
-- 依赖: 001_sub_task_binding.sql (FK child_task_id → sub_task_binding.child_task_id)

BEGIN;

CREATE TABLE IF NOT EXISTS sub_task_runtime (
    runtime_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    child_task_id UUID NOT NULL UNIQUE REFERENCES sub_task_binding(child_task_id),
    current_step TEXT,
    tokens_used BIGINT NOT NULL DEFAULT 0,
    last_llm_call_at TIMESTAMPTZ,
    last_mcp_call_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',  -- Work 短 TTL (per 守门 #13 a)
    -- W/T/M: Work, 物理删除允许 (TTL 失効), 不需 audit, 不需 RLS
    -- 简化: 仅 tenant_id + device_id 2 维 (per 01 §4)
    tenant_id UUID NOT NULL,
    device_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sub_task_runtime_expires
    ON sub_task_runtime(expires_at);

COMMIT;
