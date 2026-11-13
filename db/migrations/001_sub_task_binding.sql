-- 001_sub_task_binding.sql (WBS 6.3 Sub-task Binding, Transaction append-only)
--
-- sub_task_binding 表: 子任务绑定关系 (per docs/architecture/2026-09-09-subtask-binding/
-- 03-detailed-design.md §5 Table 1 + 01-requirements.md §4 W/T/M 横展開)
--
-- 分类: Transaction (业务事实, append-only)
--   - 物理删除禁止 (per 守门 #13 b)
--   - 审计必携 (binding 事件全部走 audit, 见 scripts/automation/task_ops/spawn_subtask/)
--   - RLS 13 类必携 (本表: tenant_id + workspace_id + parent_task_id 3 维必携, per 01 §4)
-- 血缘: 1:1 exclusive (child_task_id UNIQUE), edge_type 语义 = spawn_exclusive (per UC-14 step 7.7)
--
-- 已知缺口 (per 守门 #12 缺标比错标, 落档不掩盖):
--   G-DDL-01: 运行期 child_task_id 为 `st-` 前缀 hex 串 (per 03-detailed §3.3.1.1 step 4,
--   e.g. st-d4a6407cf3b146f7), 本表 child_task_id UUID 类型对齐 DD §5 原文;
--   Tier 3 PostgreSQL 接入时需统一 (DDD Review 拍板: TEXT 或 UUID v7 内嵌).

BEGIN;

CREATE TABLE IF NOT EXISTS sub_task_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_task_id UUID NOT NULL,
    child_task_id UUID NOT NULL UNIQUE,  -- 1:1 exclusive, UNIQUE (per F-28)
    sa_type TEXT NOT NULL CHECK (sa_type IN (
        'SA-01','SA-02','SA-03','SA-04','SA-05',
        'SA-06','SA-07','SA-08','SA-09','SA-10'
    )),
    bound_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    bound_by TEXT NOT NULL,
    bound_via TEXT NOT NULL CHECK (bound_via IN ('ui_rightclick', 'l0_chat', 'api')),
    -- 生命周期状态 (Transaction 事实, append-only 演进)
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled', 'failed', 'completed')),
    -- 派生血缘 (保留 per ADR-0046 §2.4)
    merged_from JSONB,  -- M-N1 血缘
    split_into JSONB,   -- M-N2 血缘
    superseded_by UUID,
    -- W/T/M: Transaction, 物理删除禁止, append-only
    -- RLS 13 类必携 (per 守门 #13): tenant_id + workspace_id + parent_task_id
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    -- 审计
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT rls_tenant_isolation CHECK (tenant_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_sub_task_binding_parent
    ON sub_task_binding(parent_task_id) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_sub_task_binding_tenant
    ON sub_task_binding(tenant_id);

-- RLS (per 守门 #13 RLS 13 類, per DD §5)
ALTER TABLE sub_task_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE sub_task_binding FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS sub_task_binding_tenant_isolation ON sub_task_binding;
CREATE POLICY sub_task_binding_tenant_isolation ON sub_task_binding
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

COMMIT;
