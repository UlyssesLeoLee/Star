-- 2026-09-25-ulys-195-worktree-pg-backed.sql
--
-- ULYS-195 stage 2 PR #4 — PG-backed `import_external_worktrees` 实装.
-- Per issue ULYS-177.4 §4 软依赖顺序,本 PR 不依赖 ULYS-217 / ULYS-177.2,
-- 兼容性内建在 `human_state: Running` 默认映射.
--
-- ## 增量 schema
--
-- 1. `agent_id UUID NULL` — 已存在 (per 2026-09-16-worktree-canvas-worktree.sql),
--    现补注释 (per InMemoryWorktreeService Worktree.agent_type 字段语义).
-- 2. `locked_by UUID NULL` — 新列 (per `Worktree.locked_by: Option<UserId>` 字段).
-- 3. `import_source VARCHAR(32) NOT NULL DEFAULT 'internal'` — 新列
--    (per issue §6 PG schema 设计; source = 'internal' | 'external').
-- 4. `branch_alias VARCHAR(255) NULL` — 不引入, 复用 `branch` 列 + `import_source` 区分
--    (per 守门 #11 缺标比错标: 派生字段不让重复列).
--
-- ## 索引
--
-- - `worktree_canvas_worktree_repo_path_unique` (repo_id, path) UNIQUE
--   已在 2026-09-16-worktree-canvas-worktree.sql 实装, 本 PR 复用, 不重复索引.
-- - `worktree_canvas_worktree_import_source_idx` (repo_id, import_source)
--   用于跨 repo 查全表 import 来源统计.
-- - `worktree_canvas_worktree_branch_idx` (repo_id, branch)
--   已在 2026-09-16-worktree-canvas-worktree.sql 实装, 本 PR 复用 (force=true 冲突检测).
--
-- ## RLS / M 派生 / 守门 #13
--
-- 不动 RLS policy (沿用 wc_worktree_* 7 类). 本 PR 只新增列 + 索引.
-- M SCD Type 2 字段 (valid_from / valid_to / pgpool_version) 已在
-- 2026-09-16-worktree-canvas-worktree.sql 实装, 本 PR 沿用现有架构 (不引入新表).

BEGIN;

-- 新增 `locked_by` 列 (列已存在代理, 此次 ALTER ADD IF NOT EXISTS)
ALTER TABLE worktree_canvas_worktree
    ADD COLUMN IF NOT EXISTS locked_by UUID;

-- 新增 `import_source` 列
ALTER TABLE worktree_canvas_worktree
    ADD COLUMN IF NOT EXISTS import_source VARCHAR(32) NOT NULL DEFAULT 'internal';

-- 索引: 跨 repo 按 source 过滤 (per stage 2 PR #4 触发场景 4: cron 统计 + bulk 列表)
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_import_source_idx
    ON worktree_canvas_worktree (repo_id, import_source);

-- 注释 (per 守门 #DB-13 W 派生: schema 字段必须注释)
COMMENT ON COLUMN worktree_canvas_worktree.locked_by
    IS 'Worktree.locked_by: Optional<UserId>; set when locked=true (per Worktree struct)';
COMMENT ON COLUMN worktree_canvas_worktree.import_source
    IS 'worktree 来源: internal (WorktreeService::create) | external (git worktree add import)';

COMMIT;
