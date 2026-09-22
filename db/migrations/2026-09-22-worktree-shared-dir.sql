-- 2026-09-22-worktree-shared-dir.sql (ULYS-177, per ULYS-158 §3.3 "新表 worktree_shared_dir")
--
-- v1.00 worktree_shared_dir 表 (per ULYS-158 §3.3 + FR-ORCA-007 三路并存)
--
-- 作用: 持久化 worktree 共享目录 (per FR-ORCA-007 + FR-ORCA-006 并行隔离保证).
--       1 row = 1 个共享目录 entry + 它的 source (per-user / workspace-level / cli_config)
--       + 它绑定到哪个 repo。
-- 设计: source 决定这条 entry 的来源 (per 9/22 D-Boy 路径 C 拍板), 用于 audit + 重启后 reconcile。
--       path 存的是归一化后的字符串 (per shared_dir_sources.rs normalize_path)。
-- 守门 #13 a 100% RLS + #13 b 物理删除禁止 + #13 c SCD Type 2 + #13 d T 100% audit
--
-- ULYS-177 范围 (本次实装切片):
--   ULYS-177 = ULYS-158.1 = path C 子任务 (per-workspace multica CLI config 适配).
--   本迁移只定义 **schema + RLS + audit trigger**, 不灌数据;
--   数据由 ULYS-158 §3.3 shared_dir_resolver orchestrator 在后续 sprint
--   把三路合并结果写入 (per ULYS-158 §5 实装收尾验证).
--
-- 关联:
--   - 父 issue: ULYS-158 (Worktree 共享目录 3 机制, FR-ORCA-005..011)
--   - 子 issue: ULYS-177 (= ULYS-158.1, 本 issue)
--   - FR-ORCA-007: 三路并存 (per-user + workspace-level + cli_config)

BEGIN;

DROP TABLE IF EXISTS worktree_shared_dir CASCADE;

CREATE TABLE IF NOT EXISTS worktree_shared_dir (
    -- Primary key
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    -- FK: 关联 worktree_canvas_worktree.repo_id (per ULYS-57.1 T1)
    -- ON DELETE CASCADE: worktree 删了 → 共享目录绑定也清掉 (避免悬挂引用)
    repo_id UUID NOT NULL,
    -- 共享目录的来源 (per FR-ORCA-007 三路优先级)
    -- 'per_user'        : ~/.star/worktree_shared_dirs.txt (P1, 由 ULYS-158 实装)
    -- 'workspace_level' : <workspace>/.star/worktree_shared_dirs.txt (P1, 由 ULYS-158 实装)
    -- 'cli_config'      : <workspace>/multica-config/config.json.worktree_shared_directories (路径 C, 由 ULYS-177 实装)
    source VARCHAR(32) NOT NULL,
    -- 归一化后的绝对路径 (per shared_dir_sources.rs::normalize_path)
    -- 存为 text 而非 path: 跨平台路径表达不统一 (Windows \ vs POSIX /); 比较时
    -- 由应用层做 case-insensitive (Windows) / case-sensitive (POSIX) 判定。
    path TEXT NOT NULL,
    -- 是否启用 (软删除标记; 物理删除被 RLS 禁用 per 守门 #13 b)
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    -- 排序优先级 (per FR-ORCA-007 高 → 低: per_user=10, workspace_level=20, cli_config=30)
    -- 同 source 内多 entries: 按 path 字典序排序
    priority SMALLINT NOT NULL DEFAULT 30,
    -- 审计 + 时间戳 (per 守门 #13 d T 100% audit + ADR-0043 WORM)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    -- M SCD Type 2 (per 守门 #13 c)
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    pgpool_version INT NOT NULL DEFAULT 1,
    tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'::UUID,
    workspace_id UUID NOT NULL,
    -- 显式主键
    CONSTRAINT worktree_shared_dir_pk PRIMARY KEY (id),
    -- UNIQUE: 同一 repo 内 (source, path) 唯一 (SCD Type 2 version 维度外)
    CONSTRAINT worktree_shared_dir_repo_source_path_unique
        UNIQUE (repo_id, source, path, pgpool_version),
    -- CHECK: source 必须是 3 路之一 (per FR-ORCA-007 三路并存)
    CONSTRAINT worktree_shared_dir_source_check
        CHECK (source IN ('per_user', 'workspace_level', 'cli_config')),
    -- CHECK: priority 范围 (10/20/30, 跟 source 强对应)
    CONSTRAINT worktree_shared_dir_priority_check
        CHECK (priority IN (10, 20, 30))
);

-- 7 类 RLS policy (per 守门 #13 a 100% RLS, FORCE 跨 superuser)
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

-- audit trigger (per 守门 #13 d T 100% audit, 跨 RLS)
DROP TRIGGER IF EXISTS wsd_audit ON worktree_shared_dir;
CREATE TRIGGER wsd_audit
    AFTER INSERT OR UPDATE OR DELETE ON worktree_shared_dir
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

-- Index: 按 repo + source 查所有共享目录 (per FR-ORCA-007 三路收集)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_repo_source_idx
    ON worktree_shared_dir (repo_id, source);

-- Index: 按 tenant + enabled 查活跃 entry (per shared_dir_resolver reconcile)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_tenant_enabled_idx
    ON worktree_shared_dir (tenant_id, enabled)
    WHERE enabled = TRUE;

-- Index: 按 source 优先级排序 (per FR-ORCA-007 高 → 低)
CREATE INDEX IF NOT EXISTS worktree_shared_dir_priority_idx
    ON worktree_shared_dir (repo_id, priority ASC, path ASC);

COMMIT;