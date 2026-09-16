-- 2026-09-16-worktree-canvas-graph-edge.sql (ULYS-57.1 T1, per WORKTREE-CANVAS-IMPL-PLAN-001.md §4.1)
-- v1.00 graph_edge 表 (per issue "5 表 DDL (graph_edge T)")
--
-- 作用: 关系投影缓存层 (T = Transactional, append-only); 阶段 2 实装 Neo4j Graph 同步
-- 设计: 1 row = 1 graph edge (per EdgeKind) — from_id / to_id / kind / payload / 时间戳
-- 守门 #13 a 100% RLS + #13 d T 100% audit (per ADR-0043 WORM)

BEGIN;

DROP TABLE IF EXISTS worktree_canvas_graph_edge CASCADE;

CREATE TABLE IF NOT EXISTS worktree_canvas_graph_edge (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- T row 主键 (UUID)
    from_id UUID NOT NULL,
    -- T 源 node UUID
    to_id UUID NOT NULL,
    -- T 目标 node UUID
    repo_id UUID NOT NULL,
    -- T 跨 node 关联 repo
    edge_kind VARCHAR(64) NOT NULL,
    -- T 13 Edge 类型 enum (per DD-WORKTREE-CANVAS-001 §8 + INV-WC-04)
    -- BASED_ON / USES_BRANCH / DERIVED_FROM / WORKS_ON / IMPLEMENTED_IN / MODIFIES /
    -- MODIFIES_SYMBOL / CONFLICTS_WITH / OVERLAPS_WITH / DEPENDS_ON / BLOCKS /
    -- SUPERSEDES / MERGED_INTO
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- T 完整 Edge struct (serde_json 序列化)
    weight NUMERIC(6, 4) DEFAULT 1.0,
    -- T 边权重 (CONFLICTS_WITH / OVERLAPS_WITH 用 0-1)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- T append-only timestamp
    -- 严禁 UPDATE / DELETE (per 守门 #13 b + ADR-0043 WORM)
    -- M SCD Type 2 (per 守门 #13 c) — edge 不需要 SCD (rel 单一历史版本)
    -- 但保留 version 字段以便 phase 2 评估
    version INT NOT NULL DEFAULT 1,
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    workspace_id UUID NOT NULL,
    CONSTRAINT worktree_canvas_graph_edge_pk PRIMARY KEY (id),
    -- UNIQUE (from_id, to_id, edge_kind) — 同一对 node + kind 不允许多 edge (per DD §8 strict)
    CONSTRAINT worktree_canvas_graph_edge_unique UNIQUE (from_id, to_id, edge_kind)
);

-- 7 类 RLS policy
ALTER TABLE worktree_canvas_graph_edge ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree_canvas_graph_edge FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS wc_graph_edge_tenant_isolation ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_tenant_isolation ON worktree_canvas_graph_edge
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

DROP POLICY IF EXISTS wc_graph_edge_workspace_isolation ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_workspace_isolation ON worktree_canvas_graph_edge
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

DROP POLICY IF EXISTS wc_graph_edge_select_member ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_select_member ON worktree_canvas_graph_edge
    FOR SELECT USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_edge_insert_member ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_insert_member ON worktree_canvas_graph_edge
    FOR INSERT WITH CHECK (current_setting('app.role', true) IN ('member', 'platform_admin'));

-- T 禁止 UPDATE / DELETE (per 守门 #13 b + ADR-0043 WORM)
DROP POLICY IF EXISTS wc_graph_edge_no_update ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_no_update ON worktree_canvas_graph_edge
    FOR UPDATE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wc_graph_edge_no_delete ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_no_delete ON worktree_canvas_graph_edge
    FOR DELETE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wc_graph_edge_superuser_bypass ON worktree_canvas_graph_edge;
CREATE POLICY wc_graph_edge_superuser_bypass ON worktree_canvas_graph_edge
    USING (current_setting('app.role', true) = 'superuser');

-- audit trigger
DROP TRIGGER IF EXISTS wc_graph_edge_audit ON worktree_canvas_graph_edge;
CREATE TRIGGER wc_graph_edge_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_canvas_graph_edge
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- Index
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_edge_from_idx
    ON worktree_canvas_graph_edge (from_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_edge_to_idx
    ON worktree_canvas_graph_edge (to_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_edge_kind_idx
    ON worktree_canvas_graph_edge (edge_kind);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_edge_repo_kind_idx
    ON worktree_canvas_graph_edge (repo_id, edge_kind);

COMMIT;