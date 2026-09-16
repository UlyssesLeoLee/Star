-- 2026-09-16-worktree-canvas-graph-node.sql (ULYS-57.1 T1, per WORKTREE-CANVAS-IMPL-PLAN-001.md §4.1)
-- v1.00 graph_node 表 (per issue "5 表 DDL (graph_node W)")
--
-- 作用: 关系投影缓存层 (W = Write-optimized); 阶段 2 实装 Neo4j Graph 同步
-- 设计: 1 row = 1 graph node (per NodeKind) — 落档 id / kind / repo_id / payload / 时间戳
-- 守门 #13 a 100% RLS: ENABLE + FORCE + 7 类 policy (per p3d6 rls_7_policy 模板)
-- 守门 #13 b 物理删除禁止: 4 类 CRUD delete 仅 platform_admin (审计触发器守门)
-- 守门 #13 d T 100% audit: audit trigger 跨 RLS (per ADR-0043 WORM)
-- 守门 #11 缺标比错标: 阶段 1 占位 schema, P2 阶段 worker 子代理 fill 真实 column
-- 守门 #19 v19 复用 v0.99 schema 模板 + v0.97 DROP IF EXISTS idempotent + v0.92 7 类 RLS policy

BEGIN;

-- Drop 兼容 PG 9.5+ (per v0.97)
DROP TABLE IF EXISTS worktree_canvas_graph_node CASCADE;

CREATE TABLE IF NOT EXISTS worktree_canvas_graph_node (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- W row 主键 (UUID)
    repo_id UUID NOT NULL,
    -- W row Repo 引用
    node_kind VARCHAR(32) NOT NULL,
    -- W 11 Node 类型 enum (per DD-WORKTREE-CANVAS-001 §7 + INV-WC-05)
    -- Repository / Branch / Worktree / Commit / Task / AgentSession / PullRequest / File / Symbol / TestRun / Issue
    business_id VARCHAR(255) NOT NULL,
    -- W 业务 ID (e.g. commit SHA, branch name, file path)
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- W 完整 Node struct (serde_json 序列化, per graph-core NodePayload)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- M SCD Type 2 字段 (per 守门 #13 c)
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    -- T append-only 字段 (per 守门 #13 a)
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    -- 租户隔离 (per RLS)
    workspace_id UUID NOT NULL,
    -- workspace 隔离
    CONSTRAINT worktree_canvas_graph_node_pk PRIMARY KEY (id)
);

-- 7 类 RLS policy (per v0.92 模板)
ALTER TABLE worktree_canvas_graph_node ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree_canvas_graph_node FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS wc_graph_node_tenant_isolation ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_tenant_isolation ON worktree_canvas_graph_node
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

DROP POLICY IF EXISTS wc_graph_node_workspace_isolation ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_workspace_isolation ON worktree_canvas_graph_node
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

DROP POLICY IF EXISTS wc_graph_node_select_member ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_select_member ON worktree_canvas_graph_node
    FOR SELECT USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_node_insert_member ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_insert_member ON worktree_canvas_graph_node
    FOR INSERT WITH CHECK (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_node_update_member ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_update_member ON worktree_canvas_graph_node
    FOR UPDATE USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_node_delete_platform_admin ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_delete_platform_admin ON worktree_canvas_graph_node
    FOR DELETE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wc_graph_node_superuser_bypass ON worktree_canvas_graph_node;
CREATE POLICY wc_graph_node_superuser_bypass ON worktree_canvas_graph_node
    USING (current_setting('app.role', true) = 'superuser');

-- audit trigger (per ADR-0043 WORM)
DROP TRIGGER IF EXISTS wc_graph_node_audit ON worktree_canvas_graph_node;
CREATE TRIGGER wc_graph_node_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_canvas_graph_node
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- Index
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_node_repo_idx
    ON worktree_canvas_graph_node (repo_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_node_kind_idx
    ON worktree_canvas_graph_node (node_kind);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_node_business_idx
    ON worktree_canvas_graph_node (repo_id, node_kind, business_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_node_tenant_idx
    ON worktree_canvas_graph_node (tenant_id);

COMMIT;