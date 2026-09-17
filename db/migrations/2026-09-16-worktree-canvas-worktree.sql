-- 2026-09-16-worktree-canvas-worktree.sql (ULYS-57.1 T1, per WORKTREE-CANVAS-IMPL-PLAN-001.md §4.1)
-- v1.00 worktree 表 (per issue "5 表 DDL (worktree W)")
--
-- 作用: Worktree 业务表 (W = Write-optimized); graph-core WorktreeNode 关系投影
-- 设计: 1 row = 1 worktree; 阶段 1 占位 schema, P2 阶段 worker 子代理 fill 真实 column
-- 守门 #13 a 100% RLS + #13 b 物理删除禁止 + #13 d T 100% audit

BEGIN;

DROP TABLE IF EXISTS worktree_canvas_worktree CASCADE;

CREATE TABLE IF NOT EXISTS worktree_canvas_worktree (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- W row 主键 (UUID, 跟 graph_core WorktreeId 一致)
    repo_id UUID NOT NULL,
    -- W Repo 引用 (FK worktree_canvas_graph_node.id where node_kind='Repository')
    name VARCHAR(255) NOT NULL,
    -- W Worktree name (per DD §2.1)
    branch VARCHAR(255) NOT NULL,
    -- W Branch name (per DD §2.1)
    path TEXT NOT NULL,
    -- W Worktree 路径 (per DD §2.1)
    human_state VARCHAR(32) NOT NULL DEFAULT 'running',
    -- W 7 态 (per DD §2.1: Running/Waiting/Ready/Diverged/Conflict/Merged/Stale)
    machine_state VARCHAR(32) NOT NULL DEFAULT 'git_clean',
    -- W 12 态 (per DD §2.1: AgentExecuting/GitClean/...)
    ahead INT NOT NULL DEFAULT 0,
    behind INT NOT NULL DEFAULT 0,
    dirty BOOLEAN NOT NULL DEFAULT FALSE,
    health_score SMALLINT NOT NULL DEFAULT 100,
    -- W 0-100 health (per DD §2.1 + §12)
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    test_state VARCHAR(16) NOT NULL DEFAULT 'none',
    -- W 4 档 (per DD §2.1: passed/failed/running/none)
    risk_count INT NOT NULL DEFAULT 0,
    -- W 风险计数 (缓存, 阶段 2 来自 risk-engine)
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    agent_id UUID,
    task_id UUID,
    parent_id UUID,
    -- W 自引用 (per DD §2.1 Worktree.parent_id)
    merged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- M SCD Type 2 字段 (per 守门 #13 c)
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    pgpool_version INT NOT NULL DEFAULT 1,
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    workspace_id UUID NOT NULL,
    CONSTRAINT worktree_canvas_worktree_pk PRIMARY KEY (id),
    -- UNIQUE: 同一 repo 内 worktree name 不重复
    CONSTRAINT worktree_canvas_worktree_repo_name_unique UNIQUE (repo_id, name),
    -- UNIQUE: 同一 repo 内 worktree path 不重复
    CONSTRAINT worktree_canvas_worktree_repo_path_unique UNIQUE (repo_id, path)
);

-- 7 类 RLS policy
ALTER TABLE worktree_canvas_worktree ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree_canvas_worktree FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS wc_worktree_tenant_isolation ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_tenant_isolation ON worktree_canvas_worktree
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

DROP POLICY IF EXISTS wc_worktree_workspace_isolation ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_workspace_isolation ON worktree_canvas_worktree
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

DROP POLICY IF EXISTS wc_worktree_select_member ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_select_member ON worktree_canvas_worktree
    FOR SELECT USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_worktree_insert_member ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_insert_member ON worktree_canvas_worktree
    FOR INSERT WITH CHECK (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_worktree_update_member ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_update_member ON worktree_canvas_worktree
    FOR UPDATE USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_worktree_delete_platform_admin ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_delete_platform_admin ON worktree_canvas_worktree
    FOR DELETE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wc_worktree_superuser_bypass ON worktree_canvas_worktree;
CREATE POLICY wc_worktree_superuser_bypass ON worktree_canvas_worktree
    USING (current_setting('app.role', true) = 'superuser');

-- audit trigger
DROP TRIGGER IF EXISTS wc_worktree_audit ON worktree_canvas_worktree;
CREATE TRIGGER wc_worktree_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_canvas_worktree
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- Index
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_repo_idx
    ON worktree_canvas_worktree (repo_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_state_idx
    ON worktree_canvas_worktree (repo_id, human_state);
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_branch_idx
    ON worktree_canvas_worktree (repo_id, branch);
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_parent_idx
    ON worktree_canvas_worktree (parent_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_health_idx
    ON worktree_canvas_worktree (repo_id, health_score);
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_last_activity_idx
    ON worktree_canvas_worktree (repo_id, last_activity DESC);

COMMIT;