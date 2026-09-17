-- 2026-09-16-worktree-canvas-idempotency.sql (ULYS-57.1 T1, per WORKTREE-CANVAS-IMPL-PLAN-001.md §4.1)
-- v1.00 idempotency 表 (per issue "5 表 DDL (idempotency W)")
--
-- 作用: 幂等键存储 (W = Write-optimized); 用于 action 重复检测 (FR-ACTION-006 idempotency_key)
-- 设计: 1 row = 1 idempotency key (UUID); 过期 24h 自动归档 (per 守门 #NFR-REL-001)
-- 守门 #13 a 100% RLS + #13 b 物理删除禁止 + #13 d T 100% audit

BEGIN;

DROP TABLE IF EXISTS worktree_canvas_idempotency CASCADE;

CREATE TABLE IF NOT EXISTS worktree_canvas_idempotency (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- W row 主键 (UUID)
    idempotency_key UUID NOT NULL,
    -- W 客户端提供的幂等键 (per DD §3.3 ActionRequest.idempotency_key)
    repo_id UUID NOT NULL,
    -- W Repo 引用 (per DD §3.3)
    action_type VARCHAR(64) NOT NULL,
    -- W Action 类型 (per DD §3.3 ActionRequest.action_type)
    action_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- W Action 参数 (per DD §3.3 ActionRequest.params)
    user_id UUID NOT NULL,
    -- W 触发 user (per DD §3.3 + audit)
    response_payload JSONB,
    -- W Action 结果 (per DD §3.3 ActionResult)
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    -- W 状态: pending / running / completed / failed / expired
    worktree_id UUID,
    -- W 影响 Worktree (per DD §3.3 ActionResult.worktree_id)
    duration_ms BIGINT,
    -- W 耗时毫秒 (per DD §3.3 ActionResult.duration_ms)
    warnings_count INT NOT NULL DEFAULT 0,
    errors_count INT NOT NULL DEFAULT 0,
    trace_id VARCHAR(64) NOT NULL,
    -- W 关联 trace_id (per 守门 #13 d audit context)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- W 创建时间
    started_at TIMESTAMPTZ,
    -- W 开始时间 (status=running 时设置)
    completed_at TIMESTAMPTZ,
    -- W 完成时间 (status=completed/failed 时设置)
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    -- W 过期时间 (per NFR-REL-001)
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    workspace_id UUID NOT NULL,
    CONSTRAINT worktree_canvas_idempotency_pk PRIMARY KEY (id),
    -- UNIQUE: 同一 idempotency_key 不重复 (per FR-ACTION-006)
    CONSTRAINT worktree_canvas_idempotency_key_unique UNIQUE (idempotency_key)
);

-- 7 类 RLS policy
ALTER TABLE worktree_canvas_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree_canvas_idempotency FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS wc_idempotency_tenant_isolation ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_tenant_isolation ON worktree_canvas_idempotency
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

DROP POLICY IF EXISTS wc_idempotency_workspace_isolation ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_workspace_isolation ON worktree_canvas_idempotency
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

DROP POLICY IF EXISTS wc_idempotency_select_member ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_select_member ON worktree_canvas_idempotency
    FOR SELECT USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_idempotency_insert_member ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_insert_member ON worktree_canvas_idempotency
    FOR INSERT WITH CHECK (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_idempotency_update_member ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_update_member ON worktree_canvas_idempotency
    FOR UPDATE USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_idempotency_delete_platform_admin ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_delete_platform_admin ON worktree_canvas_idempotency
    FOR DELETE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wc_idempotency_superuser_bypass ON worktree_canvas_idempotency;
CREATE POLICY wc_idempotency_superuser_bypass ON worktree_canvas_idempotency
    USING (current_setting('app.role', true) = 'superuser');

-- audit trigger
DROP TRIGGER IF EXISTS wc_idempotency_audit ON worktree_canvas_idempotency;
CREATE TRIGGER wc_idempotency_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_canvas_idempotency
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- Index
CREATE INDEX IF NOT EXISTS worktree_canvas_idempotency_repo_idx
    ON worktree_canvas_idempotency (repo_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_idempotency_status_idx
    ON worktree_canvas_idempotency (status);
CREATE INDEX IF NOT EXISTS worktree_canvas_idempotency_expires_idx
    ON worktree_canvas_idempotency (expires_at);
CREATE INDEX IF NOT EXISTS worktree_canvas_idempotency_user_idx
    ON worktree_canvas_idempotency (user_id);

COMMIT;