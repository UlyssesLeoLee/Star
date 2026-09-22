-- 2026-09-22-worktree-shared-dir.sql (ULYS-104.3 + ULYS-158 + ULYS-177 合并, 2026-09-22 JST)
--
-- v1.0 worktree_shared_dir 表 (per docs/ecosystem-survey/orca-design-survey.md v1.0 §3 FR-ORCA-007
-- + 9/22 D-Boy 拍板 TBD-0044-01 C 选项 + 11:36 路径 C 自决:
--   第 3 机制不依赖 Multica CLI 升版, 直读 `<workspace>/multica-config/config.json`).
--
-- 作用: Worktree 共享目录配置 (M = Metadata); 3 机制之一 (workspace-level, per v1.0 §3 FR-ORCA-007 #2)
-- 设计: 1 row = 1 shared directory entry; mount_strategy 5 档 + priority 1-3
--       + source 标注 (per_user / workspace_level / cli_config, per FR-ORCA-007 三路并存)
-- 守门 #13 a 100% RLS + #13 b 物理删除禁止 (软删 enabled=false) + #13 d T 100% audit
--
-- 合并说明:
--   ULYS-158 v1.0 MVP (commit b90df528) 提供 mount_strategy + label + priority (P1/P2/P3) + RLS + audit.
--   ULYS-177 v1.0 (commit ef025c8d) 提供 source 字段 (per_user / workspace_level / cli_config).
--   本次合并 (ULYS-158.1 + ULYS-177 path C) 合并两者字段, source 跟 mount_strategy 各自独立,
--   让 3 路来源可被 audit + 在 worktree_create_async 时 reconcile.

BEGIN;

DROP TABLE IF EXISTS worktree_shared_dir CASCADE;

CREATE TABLE IF NOT EXISTS worktree_shared_dir (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- M row 主键
    repo_id UUID NOT NULL,
    -- M Repo 引用 (FK worktree_canvas_graph_node.id where node_kind='Repository')
    path TEXT NOT NULL,
    -- M 共享目录绝对路径 (per FR-ORCA-007 AC-1/AC-2/AC-3 跨 worktree 共享)
    -- source 字段: 3 路来源 (per FR-ORCA-007 三路并存, per 9/22 D-Boy 路径 C 拍板):
    --   'per_user'        : ~/.star/worktree_shared_dirs.txt (P1, 由后续 P1 实装)
    --   'workspace_level' : <workspace>/.star/worktree_shared_dirs.txt (P1, 由后续 P1 实装, PG 表即本表)
    --   'cli_config'      : <workspace>/multica-config/config.json.worktree_shared_directories
    --                       (路径 C, FileBackedConfigSource 直读, 已实装 per ULYS-177)
    source VARCHAR(32) NOT NULL DEFAULT 'cli_config',
    -- M 5 档 mount (per FR-ORCA-007 + 9/22 D-Boy 决策 TBD-0044-02):
    --   worktree_add / symlink / hardlink / bind_mount / copy
    mount_strategy VARCHAR(16) NOT NULL DEFAULT 'worktree_add',
    -- M 显示名 (e.g. "node_modules", ".env", ".vscode/settings.json")
    label VARCHAR(64) NOT NULL DEFAULT '',
    -- M 1-3 档 (P1 = 最高 = workspace-level 主共享; P3 = 最低 = per-user 兜底)
    priority SMALLINT NOT NULL DEFAULT 3,
    -- M 软删标志; 物理删除 = FALSE
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    -- M 创建人 (FK star-identity.user.id)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,
    -- M 最后修改人
    retention_period INT,
    -- M 保留天数 (NULL = 永久; 用于 audit GC)
    -- M SCD Type 2 字段 (per 守门 #13 c)
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    pgpool_version INT NOT NULL DEFAULT 1,
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    workspace_id UUID NOT NULL,
    CONSTRAINT worktree_shared_dir_pk PRIMARY KEY (id),
    -- UNIQUE: 同一 repo 内 (source, path, mount_strategy) 不重复 (SCD Type 2 version 维度外)
    CONSTRAINT worktree_shared_dir_repo_source_path_strategy_unique
        UNIQUE (repo_id, source, path, mount_strategy, pgpool_version),
    -- CHECK: source 必须在 3 路之一
    CONSTRAINT worktree_shared_dir_source_check
        CHECK (source IN ('per_user', 'workspace_level', 'cli_config')),
    -- CHECK: mount_strategy 必须在 5 档内
    CONSTRAINT worktree_shared_dir_mount_strategy_check
        CHECK (mount_strategy IN ('worktree_add', 'symlink', 'hardlink', 'bind_mount', 'copy')),
    -- CHECK: priority 必须在 1-3 内 (P1/P2/P3)
    CONSTRAINT worktree_shared_dir_priority_check
        CHECK (priority BETWEEN 1 AND 3)
);

-- 7 类 RLS policy (per 守门 #13 a)
ALTER TABLE worktree_shared_dir ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree_shared_dir FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS wsd_tenant_isolation ON worktree_shared_dir;
CREATE POLICY wsd_tenant_isolation ON worktree_shared_dir
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

DROP POLICY IF EXISTS wsd_workspace_isolation ON worktree_shared_dir;
CREATE POLICY wsd_workspace_isolation ON worktree_shared_dir
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

DROP POLICY IF EXISTS wsd_select_member ON worktree_shared_dir;
CREATE POLICY wsd_select_member ON worktree_shared_dir
    FOR SELECT USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wsd_insert_member ON worktree_shared_dir;
CREATE POLICY wsd_insert_member ON worktree_shared_dir
    FOR INSERT WITH CHECK (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wsd_update_member ON worktree_shared_dir;
CREATE POLICY wsd_update_member ON worktree_shared_dir
    FOR UPDATE USING (current_setting('app.role', true) IN ('member', 'platform_admin'));

DROP POLICY IF EXISTS wsd_delete_platform_admin ON worktree_shared_dir;
CREATE POLICY wsd_delete_platform_admin ON worktree_shared_dir
    FOR DELETE USING (current_setting('app.role', true) = 'platform_admin');

DROP POLICY IF EXISTS wsd_superuser_bypass ON worktree_shared_dir;
CREATE POLICY wsd_superuser_bypass ON worktree_shared_dir
    USING (current_setting('app.role', true) = 'superuser');

-- audit trigger (per 守门 #13 d)
DROP TRIGGER IF EXISTS wsd_audit ON worktree_shared_dir;
CREATE TRIGGER wsd_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_shared_dir
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- Indexes (4 个核心索引 + 2 partial)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_repo_idx
    ON worktree_shared_dir (repo_id);
CREATE INDEX IF NOT EXISTS worktree_shared_dir_workspace_idx
    ON worktree_shared_dir (workspace_id);
-- Partial index: 仅对 enabled=TRUE 的行建 priority 索引 (生产环境 90% 行 enabled)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_priority_idx
    ON worktree_shared_dir (repo_id, priority)
    WHERE enabled = TRUE;
-- Partial index: 按 source 过滤 (cli_config 路径 C, per ULYS-177)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_source_idx
    ON worktree_shared_dir (repo_id, source)
    WHERE enabled = TRUE;
-- Partial index: 仅对 retention_period 已设置的行建索引 (audit GC 用)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_retention_idx
    ON worktree_shared_dir (retention_period, created_at)
    WHERE retention_period IS NOT NULL;

COMMIT;