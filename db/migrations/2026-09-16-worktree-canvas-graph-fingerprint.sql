-- 2026-09-16-worktree-canvas-graph-fingerprint.sql (ULYS-57.1 T1, per WORKTREE-CANVAS-IMPL-PLAN-001.md §4.1)
-- v1.00 graph_fingerprint 表 (per issue "5 表 DDL (graph_fingerprint M)")
--
-- 作用: Graph 状态快照 (M = Materialized view derived from W/T)
-- 设计: 每 (repo_id, content_hash) 一条 fingerprint, 用于 cache hit 短路 + invariant violation 检测
-- 守门 #13 c M SCD Type 2: valid_from / valid_to + pgpool_version + 视图
-- 守门 #13 d 100% audit: audit trigger 跨 RLS
-- 阶段 1 占位 schema, P2 阶段 worker 子代理 fill 真实 column

BEGIN;

DROP TABLE IF EXISTS worktree_canvas_graph_fingerprint CASCADE;

CREATE TABLE IF NOT EXISTS worktree_canvas_graph_fingerprint (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- M row 主键 (UUID)
    repo_id UUID NOT NULL,
    -- M Repo 引用
    content_hash CHAR(64) NOT NULL,
    -- M SHA-256 of canonicalized graph state (per content of all nodes + edges)
    fingerprint JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- M 快照: 节点数 / 边数 / 风险数 / 健康分等汇总
    node_count INT NOT NULL DEFAULT 0,
    edge_count INT NOT NULL DEFAULT 0,
    worktree_count INT NOT NULL DEFAULT 0,
    risk_count INT NOT NULL DEFAULT 0,
    health_avg NUMERIC(5, 2) DEFAULT 100.0,
    -- M SCD Type 2 字段 (per 守门 #13 c)
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    pgpool_version INT NOT NULL DEFAULT 1,
    -- M pgpool 副本版本号
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- M 计算时间 (用于 staleness 判断)
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    workspace_id UUID NOT NULL,
    CONSTRAINT worktree_canvas_graph_fingerprint_pk PRIMARY KEY (id),
    -- UNIQUE: 同一 (repo_id, content_hash, valid_from) 不重复
    CONSTRAINT worktree_canvas_graph_fingerprint_unique UNIQUE (repo_id, content_hash, valid_from)
);

-- 7 类 RLS policy
ALTER TABLE worktree_canvas_graph_fingerprint ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree_canvas_graph_fingerprint FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS wc_graph_fp_tenant_isolation ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_tenant_isolation ON worktree_canvas_graph_fingerprint
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

DROP POLICY IF EXISTS wc_graph_fp_workspace_isolation ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_workspace_isolation ON worktree_canvas_graph_fingerprint
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

DROP POLICY IF EXISTS wc_graph_fp_select_member ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_select_member ON worktree_canvas_graph_fingerprint
    FOR SELECT USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_fp_insert_member ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_insert_member ON worktree_canvas_graph_fingerprint
    FOR INSERT WITH CHECK (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_fp_update_member ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_update_member ON worktree_canvas_graph_fingerprint
    FOR UPDATE USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wc_graph_fp_delete_platform_admin ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_delete_platform_admin ON worktree_canvas_graph_fingerprint
    FOR DELETE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wc_graph_fp_superuser_bypass ON worktree_canvas_graph_fingerprint;
CREATE POLICY wc_graph_fp_superuser_bypass ON worktree_canvas_graph_fingerprint
    USING (current_setting('app.role', true) = 'superuser');

-- audit trigger
DROP TRIGGER IF EXISTS wc_graph_fp_audit ON worktree_canvas_graph_fingerprint;
CREATE TRIGGER wc_graph_fp_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_canvas_graph_fingerprint
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- M SCD Type 2 view: 当前 valid fingerprint (per 守门 #13 c)
DROP VIEW IF EXISTS wc_graph_fingerprint_current CASCADE;
CREATE VIEW wc_graph_fingerprint_current AS
SELECT *
FROM worktree_canvas_graph_fingerprint
WHERE valid_to IS NULL
ORDER BY repo_id, computed_at DESC;

-- Index
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_fingerprint_repo_idx
    ON worktree_canvas_graph_fingerprint (repo_id);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_fingerprint_content_hash_idx
    ON worktree_canvas_graph_fingerprint (content_hash);
CREATE INDEX IF NOT EXISTS worktree_canvas_graph_fingerprint_valid_idx
    ON worktree_canvas_graph_fingerprint (repo_id, valid_to);

COMMIT;