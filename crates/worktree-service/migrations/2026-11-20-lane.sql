-- P0-lane: Lane 正式数据 (2026-11-20)
-- 验收: 没有远端仓库/分支关联时也能创建和管理 Lane; 撤权后 list/get 立即不可见。
--
-- 本文件由 P0-lane agent 生成, 位于 crates/worktree-service/migrations/ 独立存放。
-- 注册进 db/migrations/ 迁移链 + 实际 apply 由父代理在合并时完成 (见 P0-lane 报告待办)。
--
-- 表设计:
--   lane (public schema, 主表)             — 无任何仓库/分支列, 远端关联是未来可选元数据
--   multica.worktree_lane_binding (双时态) — Worktree↔Lane 归属, 照 multica.worktree_project_binding 模式
--
-- 守门 #13: a) 100% RLS + b) 物理删除禁止 (lane 走逻辑删除 deleted_at? P0 先做硬删除,
--   迁移链注册时父代理可补 soft-delete trigger; 见待办)

BEGIN;

CREATE TABLE IF NOT EXISTS lane (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'),
    project_id UUID NOT NULL CHECK (project_id <> '00000000-0000-0000-0000-000000000000'),
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 128),
    description TEXT,
    created_by UUID NOT NULL,
    lock_version INTEGER NOT NULL DEFAULT 1 CHECK (lock_version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_lane_tenant_project_name
    ON lane (tenant_id, project_id, name);
CREATE INDEX IF NOT EXISTS idx_lane_project_directory
    ON lane (tenant_id, project_id, created_at DESC, id DESC);

ALTER TABLE lane ENABLE ROW LEVEL SECURITY;
ALTER TABLE lane FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS lane_tenant_select ON lane;
CREATE POLICY lane_tenant_select ON lane
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DROP POLICY IF EXISTS lane_tenant_insert ON lane;
CREATE POLICY lane_tenant_insert ON lane
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DROP POLICY IF EXISTS lane_tenant_update ON lane;
CREATE POLICY lane_tenant_update ON lane
    FOR UPDATE USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DROP POLICY IF EXISTS lane_tenant_delete ON lane;
CREATE POLICY lane_tenant_delete ON lane
    FOR DELETE USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

-- ---------------------------------------------------------------------
-- Worktree ↔ Lane 双时态绑定 (照 multica.worktree_project_binding 模式)
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS multica.worktree_lane_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    lane_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    assigned_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, lane_id)
        REFERENCES lane (tenant_id, id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_worktree_lane_binding_active
    ON multica.worktree_lane_binding (tenant_id, worktree_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_worktree_lane_binding_lane
    ON multica.worktree_lane_binding (tenant_id, lane_id, worktree_id) WHERE valid_to IS NULL;

ALTER TABLE multica.worktree_lane_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.worktree_lane_binding FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS worktree_lane_binding_tenant_select ON multica.worktree_lane_binding;
CREATE POLICY worktree_lane_binding_tenant_select ON multica.worktree_lane_binding
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DROP POLICY IF EXISTS worktree_lane_binding_tenant_insert ON multica.worktree_lane_binding;
CREATE POLICY worktree_lane_binding_tenant_insert ON multica.worktree_lane_binding
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DROP POLICY IF EXISTS worktree_lane_binding_tenant_update ON multica.worktree_lane_binding;
CREATE POLICY worktree_lane_binding_tenant_update ON multica.worktree_lane_binding
    FOR UPDATE USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DROP POLICY IF EXISTS worktree_lane_binding_tenant_delete ON multica.worktree_lane_binding;
CREATE POLICY worktree_lane_binding_tenant_delete ON multica.worktree_lane_binding
    FOR DELETE USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMIT;
