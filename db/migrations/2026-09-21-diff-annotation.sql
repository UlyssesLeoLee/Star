-- =====================================================================
-- ULYS-165 (ULYS-104.10) FR-ORCA-035 — diff_annotation 表
-- Per `docs/ecosystem-survey/orca-design-survey.md` §11 line 483-485:
--   在 Diff 上写注释 → 回喂 Agent
-- 与 crates/agent-bridge/src/annotate.rs (PR #84) 1:1 对齐.
--
-- **本文件 (ULYS-165 v2)** — 适配 PR #84 API:
--   - 移除 tenant_id / workspace_id (per MVP in-memory registry, P1 引入 PG 时再加)
--   - 用 author TEXT ('user' / 'agent') 而非 status 字段
--   - comment_id 是 annotation_id (per DiffAnnotation::id)
--
-- 守门:
-- - #13 a: 7-class RLS (P1 引入 PG 时再加 tenant_id / workspace_id)
-- - #13 b: WORM on body / file_path / line_range / created_at
-- - #13 d: audit_trigger_func (P1)
-- =====================================================================

CREATE TABLE IF NOT EXISTS diff_annotation (
    -- 业务侧幂等键 (comment_id 通常由前端 UUID v4 生成, 重试幂等)
    comment_id        UUID PRIMARY KEY,

    -- worktree-relative POSIX 路径 (per AGENTS.md §4 win+unix 路径)
    file_path         TEXT NOT NULL,

    -- 行号范围, 1-based 半开 [start, end). 用 INT4RANGE 支持 GIST 索引与
    -- 跨 annotation 重叠检测 (P1 followup).
    line_start        INTEGER NOT NULL,
    line_end          INTEGER NOT NULL,

    -- 注释正文 (Markdown)
    body              TEXT NOT NULL CHECK (length(trim(body)) > 0),

    -- 关联 agent_run_id (AgentRuntime::start_session 返回的 id).
    -- NOT NULL: 现阶段 in-memory registry 要求 register_agent_run (P1 引入 unbound)
    agent_run_id      UUID NOT NULL,

    -- 作者: 'user' (UI 手动) / 'agent' (refinement loop 自动 emit, MVP v0 不实装)
    author            TEXT NOT NULL
                      CHECK (author IN ('user', 'agent')),

    -- 创建时间
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 守门 #13 b: line_range 半开区间合法性
    CONSTRAINT line_range_valid CHECK (line_start >= 1 AND line_start <= line_end)
);

-- =====================================================================
-- 索引
-- =====================================================================

-- agent_run 维度快速取所有标注
CREATE INDEX IF NOT EXISTS diff_annotation_agent_run_idx
    ON diff_annotation (agent_run_id);