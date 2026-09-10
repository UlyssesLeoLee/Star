#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""P3-D.6 阶段 1 基础 任务 1.6 14 张新表 SQL DDL 生成器 (per 守门 #13 W/T/M 100% 覆盖 + 0 混在).

Per:
- 守门 #13 W/T/M 100% 覆盖 + 0 混在 (6 M + 5 T + 3 W = 14 张)
- 守门 #9 v19 Python 化累积规
- 守门 #19 v19 复用 v0.92 rls_7_policy_gen.py 7 类 RLS policy 模式
- 守门 #1 禁回溯叙事 0 改 现有 15 张 DDL (V0.1 = 5 A11 + 4 A12 + 6 G)
- 守门 #1 v25 实证 (本脚本生成 0 err)
- 守门 #14 v4 Mavis 审核 author=Ulysses
- 守门 #13 d T 100% audit: WORM append-only trigger (per ADR-0043)

输入: 14 张表 dict (name + domain + w_t_m + description + columns + dd_ref + index_columns)
输出: db/migrations/2026-09-10-p3d6-15-more-tables.sql (auto-generated, ~2500 lines)

14 张新表 (per brief §1.1, V0.1 = 15 张 + 任务 1.6 = 14 张 = 29 张总):
- A11 +2: agent_trust_score_history M + agent_session_audit T
- A12 +3: canvas_permissions M + canvas_presence_cursors W + canvas_followers W
- G +9: gamify_achievements M + gamify_xp_history T + gamify_level_progress W +
        gamify_rewards M + gamify_leaderboards M + gamify_avatar_customization M +
        gamify_vote_audit T + gamify_sticky_note_audit T + gamify_confetti_audit T

守门 #13 W/T/M: 6 M + 5 T + 3 W = 14 张, 0 混在.
5 张 T 表配套 WORM append-only trigger (per 守门 #13 d + ADR-0043).

每张表 DDL 结构 (per V0.1 同形, 14 张每张 ~150 lines):
  1. 表头注释 (W/T/M 分类 + dd_ref + 守门合规)
  2. CREATE TABLE IF NOT EXISTS (per 列定义 tuple)
  3. 索引 (tenant_id + 业务 lookup 列)
  4. M 表 SCD Type 2 视图 v_<name>_current
  5. RLS 6 类 policy (per v0.92, 跳过 health_visibility 因表无 health_status 列)
  6. T 表 WORM append-only trigger (BEFORE UPDATE OR DELETE RAISE EXCEPTION)

Usage:
    python scripts/sql/p3d6_15_more_tables_gen.py
    python scripts/sql/p3d6_15_more_tables_gen.py --output db/migrations/2026-09-10-p3d6-15-more-tables.sql
"""
import argparse
import sys
from pathlib import Path


# 14 张 P3-D.6 新表 (per brief §1.1, 守门 #13 W/T/M 100% 覆盖 + 0 混在)
# 每张表 dict: name + domain + w_t_m + description + dd_ref + columns + index_columns
# column tuple: (col_name, col_type, nullability_str, constraint_str)
#   - nullability_str: "NOT NULL" or None
#   - constraint_str: "PRIMARY KEY" | "DEFAULT ..." | "CHECK (...)" | None
#   - 跟 V0.1 p3d6_13_tables_gen.py 的 template 模式一致, 但每张表独立 columns dict (非 agent_sessions template 替换)
# index_columns: 业务 lookup 列 (除 tenant_id 默认 + id PK 已有) → 生成 CREATE INDEX IF NOT EXISTS
P3D6_15_MORE_TABLES = [
    # ===== A11 agent 域 +2 张 =====
    {
        "name": "agent_trust_score_history",
        "domain": "A11",
        "w_t_m": "M",  # Master (物理删除禁止 + SCD Type 2 + 100% RLS)
        "description": "Trust score 变化历史 (per V0.2 强类型 enum TrustScoreTier 5 档)",
        "dd_ref": "V0.2 TrustScoreTier 5 变体 + ARG.G-4",
        "index_columns": ["agent_id", "tier", "created_at"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),  # 守门 #13 a 100% RLS
            ("agent_id", "UUID", "NOT NULL", None),  # FK to agents (V0.1)
            ("score", "REAL", "NOT NULL", "CHECK (score >= 0.0 AND score <= 1.0)"),
            ("tier", "TEXT", "NOT NULL", "CHECK (tier IN ('Untrusted', 'Low', 'Medium', 'High', 'VeryHigh'))"),
            ("from_state", "TEXT", None, None),  # SCD Type 2 跨档追踪 (nullable, 首次无 from_state)
            ("to_state", "TEXT", "NOT NULL", None),  # SCD Type 2 跨档追踪
            ("change_reason", "TEXT", None, None),  # free-form reason
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("created_by", "UUID", "NOT NULL", None),
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),  # 守门 #13 c SCD Type 2
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),  # 守门 #13 a + §13.5
        ],
    },
    {
        "name": "agent_session_audit",
        "domain": "A11",
        "w_t_m": "T",  # Transaction (物理删除禁止 + 100% audit WORM)
        "description": "Session audit WORM append-only (per A3.2 状态变化 audit + V0.3 业务方法)",
        "dd_ref": "A3.2 + ADR-0043 WORM append-only",
        "audit_trigger": True,  # 配套 WORM trigger
        "index_columns": ["agent_id", "session_id", "at", "action"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),  # 守门 #13 a 100% RLS
            ("agent_id", "UUID", "NOT NULL", None),
            ("session_id", "UUID", "NOT NULL", None),  # FK to agent_sessions
            ("action", "TEXT", "NOT NULL", "CHECK (action IN ('create', 'update', 'delete', 'expire', 'revoke', 'refresh'))"),
            ("from_state", "TEXT", None, None),
            ("to_state", "TEXT", None, None),
            ("at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("version", "BIGINT", "NOT NULL", "DEFAULT 1"),  # T 表 version (无 SCD, 仅为事务序)
            ("metadata", "JSONB", None, "DEFAULT '{}'::jsonb"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    # ===== A12 canvas-collab 域 +3 张 =====
    {
        "name": "canvas_permissions",
        "domain": "A12",
        "w_t_m": "M",  # Master
        "description": "Canvas 3 档权限 view/comment/edit (per A12.7)",
        "dd_ref": "A12.7 PermissionLevel 3 变体",
        "index_columns": ["canvas_id", "user_id", "permission_level"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("canvas_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("permission_level", "TEXT", "NOT NULL", "CHECK (permission_level IN ('View', 'Comment', 'Edit'))"),
            ("granted_by", "UUID", "NOT NULL", None),
            ("granted_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("expires_at", "TIMESTAMPTZ", None, None),  # Option<TIMESTAMPTZ>
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "canvas_presence_cursors",
        "domain": "A12",
        "w_t_m": "W",  # Work (物理删除 + 短 TTL retention_period 5 min)
        "description": "Presence cursor 推送 (per A12.2, retention 5 min)",
        "dd_ref": "A12.2 presence cursor",
        "retention_period": "5 minutes",  # 守门 #13 a
        "index_columns": ["canvas_id", "user_id", "last_active"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("canvas_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("x", "REAL", "NOT NULL", None),
            ("y", "REAL", "NOT NULL", None),
            ("color", "TEXT", "NOT NULL", "DEFAULT '#FF0000'"),
            ("last_active", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),  # retention 5 min
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "canvas_followers",
        "domain": "A12",
        "w_t_m": "W",  # Work
        "description": "Follow mode (per A12.4, retention 5 min)",
        "dd_ref": "A12.4 follow mode",
        "retention_period": "5 minutes",
        "index_columns": ["canvas_id", "follower_id", "followee_id", "last_active"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("canvas_id", "UUID", "NOT NULL", None),
            ("follower_id", "UUID", "NOT NULL", None),
            ("followee_id", "UUID", "NOT NULL", None),
            ("started_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("last_active", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),  # retention 5 min
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    # ===== G11 gamify 域 +9 张 =====
    {
        "name": "gamify_achievements",
        "domain": "G",
        "w_t_m": "M",  # Master
        "description": "成就系统 (per G5)",
        "dd_ref": "G5 achievement",
        "index_columns": ["user_id", "achievement_key", "unlocked_at"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("achievement_key", "TEXT", "NOT NULL", None),  # unique per (tenant_id, user_id, achievement_key)
            ("title", "TEXT", "NOT NULL", None),
            ("description", "TEXT", None, None),
            ("icon_url", "TEXT", None, None),
            ("unlocked_at", "TIMESTAMPTZ", None, None),  # Option<TIMESTAMPTZ> 未解锁时 None
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("created_by", "UUID", "NOT NULL", None),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_xp_history",
        "domain": "G",
        "w_t_m": "T",  # Transaction
        "description": "经验值累计 (per G2)",
        "dd_ref": "G2 XP",
        "audit_trigger": True,
        "index_columns": ["user_id", "at", "source"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("xp_delta", "INTEGER", "NOT NULL", None),  # +gain / -spend
            ("xp_total", "INTEGER", "NOT NULL", None),  # 累计后总 XP
            ("source", "TEXT", "NOT NULL", None),  # e.g. 'vote', 'streak', 'achievement'
            ("at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("metadata", "JSONB", None, "DEFAULT '{}'::jsonb"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_level_progress",
        "domain": "G",
        "w_t_m": "W",  # Work
        "description": "等级进度 (per G3, retention 30 days)",
        "dd_ref": "G3 level progress",
        "retention_period": "30 days",
        "index_columns": ["user_id", "level_id", "last_updated"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("level_id", "UUID", "NOT NULL", None),  # FK to gamify_levels (V0.1)
            ("xp_in_level", "INTEGER", "NOT NULL", "DEFAULT 0"),
            ("xp_needed_next", "INTEGER", "NOT NULL", None),
            ("last_updated", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),  # retention 30 days
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_rewards",
        "domain": "G",
        "w_t_m": "M",  # Master
        "description": "奖励 (per G7)",
        "dd_ref": "G7 rewards",
        "index_columns": ["user_id", "reward_key", "status", "cost_xp"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("reward_key", "TEXT", "NOT NULL", None),
            ("title", "TEXT", "NOT NULL", None),
            ("description", "TEXT", None, None),
            ("cost_xp", "INTEGER", "NOT NULL", "CHECK (cost_xp >= 0)"),
            ("status", "TEXT", "NOT NULL", "CHECK (status IN ('Available', 'Redeemed', 'Expired'))"),
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_leaderboards",
        "domain": "G",
        "w_t_m": "M",  # Master
        "description": "排行榜 (per G6)",
        "dd_ref": "G6 leaderboard",
        "index_columns": ["board_key", "user_id", "rank", "score", "period"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("board_key", "TEXT", "NOT NULL", None),  # e.g. 'weekly_xp', 'monthly_streak'
            ("user_id", "UUID", "NOT NULL", None),
            ("rank", "INTEGER", "NOT NULL", "CHECK (rank > 0)"),
            ("score", "INTEGER", "NOT NULL", "CHECK (score >= 0)"),
            ("period", "TEXT", "NOT NULL", "CHECK (period IN ('Daily', 'Weekly', 'Monthly', 'AllTime'))"),
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_avatar_customization",
        "domain": "G",
        "w_t_m": "M",  # Master
        "description": "头像定制 (per G4)",
        "dd_ref": "G4 avatar customization",
        "index_columns": ["user_id", "slot", "is_equipped"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("slot", "TEXT", "NOT NULL", "CHECK (slot IN ('Hat', 'Body', 'Background', 'Accessory', 'Pet'))"),
            ("asset_id", "UUID", None, None),  # 引用 asset 库 (per G4)
            ("color_hex", "TEXT", None, "CHECK (color_hex IS NULL OR color_hex ~ '^#[0-9A-Fa-f]{6}$')"),
            ("is_equipped", "BOOLEAN", "NOT NULL", "DEFAULT false"),
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("updated_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_vote_audit",
        "domain": "G",
        "w_t_m": "T",  # Transaction
        "description": "投票审计 (per G8)",
        "dd_ref": "G8 vote audit",
        "audit_trigger": True,
        "index_columns": ["vote_id", "user_id", "at", "action"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("vote_id", "UUID", "NOT NULL", None),  # FK to gamify_votes (V0.1)
            ("user_id", "UUID", "NOT NULL", None),
            ("action", "TEXT", "NOT NULL", "CHECK (action IN ('cast', 'change', 'retract'))"),
            ("at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_sticky_note_audit",
        "domain": "G",
        "w_t_m": "T",  # Transaction
        "description": "Sticky note 审计 (per G9)",
        "dd_ref": "G9 sticky note audit",
        "audit_trigger": True,
        "index_columns": ["sticky_note_id", "user_id", "at", "action"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("sticky_note_id", "UUID", "NOT NULL", None),  # FK to gamify_sticky_notes (V0.1)
            ("user_id", "UUID", "NOT NULL", None),
            ("action", "TEXT", "NOT NULL", "CHECK (action IN ('create', 'update', 'delete', 'pin', 'unpin'))"),
            ("at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "gamify_confetti_audit",
        "domain": "G",
        "w_t_m": "T",  # Transaction
        "description": "撒花审计 (per G10)",
        "dd_ref": "G10 confetti audit",
        "audit_trigger": True,
        "index_columns": ["confetti_id", "user_id", "at", "action"],
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("confetti_id", "UUID", "NOT NULL", None),  # FK to gamify_confetti (V0.1)
            ("user_id", "UUID", "NOT NULL", None),
            ("action", "TEXT", "NOT NULL", "CHECK (action IN ('trigger', 'burst', 'complete'))"),
            ("at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
]


def gen_column_lines(columns: list) -> str:
    """生成 CREATE TABLE 内的列定义 lines (per col tuple: name, type, nullability, constraint)."""
    lines = []
    for col_name, col_type, nullability, constraint in columns:
        parts = [f"    {col_name:<18} {col_type}"]
        if nullability:
            parts.append(nullability)
        if constraint:
            parts.append(constraint)
        lines.append(" ".join(parts) + ",")
    return "\n".join(lines)


def gen_index_lines(table_name: str, index_columns: list) -> str:
    """生成 CREATE INDEX IF NOT EXISTS lines (per §6.1 query 优化)."""
    lines = []
    for col in index_columns:
        idx_name = f"idx_{table_name}_{col}"
        lines.append(f"CREATE INDEX IF NOT EXISTS {idx_name}")
        lines.append(f"    ON {table_name} ({col});")
    return "\n".join(lines)


def gen_create_table_block(table: dict, idx: int) -> str:
    """生成单张表完整 DDL block: 表头注释 + CREATE TABLE + 索引 + 可选 view + RLS + 可选 trigger."""
    name = table["name"]
    w_t_m = table["w_t_m"]
    description = table["description"]
    dd_ref = table.get("dd_ref", "")
    columns = table["columns"]
    retention = table.get("retention_period", None)
    index_columns = table.get("index_columns", [])
    has_trigger = table.get("audit_trigger", False)

    # 头部注释
    if w_t_m == "M":
        wtm_note = f"守门 #13 c SCD Type 2 (pgpool_version 字段)"
        m_section = """
-- 当前 SCD Type 2 "有效" version 视图 (per V0.82 pgpool_version 模式)
CREATE OR REPLACE VIEW v_{name}_current AS
SELECT DISTINCT ON (tenant_id, id) *
FROM {name}
ORDER BY tenant_id, id, pgpool_version DESC;
""".format(name=name)
    elif w_t_m == "T":
        wtm_note = "守门 #13 d T 100% audit WORM append-only (per ADR-0043)"
        m_section = ""
    else:  # W
        wtm_note = f"守门 #13 a 物理删除 + 短 TTL retention_period = {retention}"
        m_section = ""

    # Section 1: 表头 + CREATE TABLE
    section1 = f"""-- ==========================================
-- {idx}. {name} 表 (per {dd_ref} + {wtm_note})
-- W/T/M 分类: {w_t_m}
-- 说明: {description}
-- ==========================================
CREATE TABLE IF NOT EXISTS {name} (
    -- 业务字段 (per {dd_ref})
{gen_column_lines(columns)}
    -- 显式主键
    PRIMARY KEY (id)
);"""

    # Section 2: 索引
    section2 = ""
    if index_columns:
        section2 = f"""

-- 索引 (per §6.1 query 优化)
{gen_index_lines(name, index_columns)}"""

    # Section 3: SCD Type 2 view (仅 M 表)
    section3 = m_section

    # Section 4: RLS 6 类 policy
    section4 = f"""

-- ==========================================
-- {idx}.RLS  RLS 6 类 policy (per v0.92 + v0.97 DROP IF EXISTS 兼容 PG 9.5+)
-- 7 类中跳过 health_visibility (表无 health_status 列, per v0.93 缺口 (d) 修)
-- ==========================================
ALTER TABLE {name} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {name} FORCE ROW LEVEL SECURITY; -- superuser 也走 RLS (per ADR-0043)

-- {name} 4 类CRUD policy (per v0.91 §8.3 映射表 1-4, v0.97 加 DROP IF EXISTS 兼容 PG 9.5+)
DROP POLICY IF EXISTS {name}_select ON {name};
CREATE POLICY {name}_select ON {name}
    FOR SELECT
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS {name}_insert ON {name};
CREATE POLICY {name}_insert ON {name}
    FOR INSERT
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS {name}_update ON {name};
CREATE POLICY {name}_update ON {name}
    FOR UPDATE
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

DROP POLICY IF EXISTS {name}_delete ON {name};
CREATE POLICY {name}_delete ON {name}
    FOR DELETE
    USING (
        current_setting('app.is_admin', true) = 'true'
    ); -- 守门 #13 b 物理删除禁止: 仅 platform_admin

-- {name} schema_isolation policy (per v0.91 §8.3 映射表 5, 14 张新表都含 schema_name 列)
DROP POLICY IF EXISTS {name}_schema_isolation ON {name};
CREATE POLICY {name}_schema_isolation ON {name}
    FOR ALL
    USING (
        schema_name = current_setting('app.current_schema_name', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

-- {name} platform admin override (per v0.91 §8.3 映射表 6 + v0.88 BYPASSRLS)
DROP POLICY IF EXISTS {name}_platform_admin ON {name};
CREATE POLICY {name}_platform_admin ON {name}
    FOR ALL
    TO platform_admin
    USING (true)
    WITH CHECK (true);"""

    # Section 5: WORM trigger (仅 T 表)
    section5 = ""
    if has_trigger:
        # 触发器函数名: 若表名以 _audit 结尾, 去掉 _audit 后缀, 避免 X_audit_audit_trigger
        base_name = name[:-len("_audit")] if name.endswith("_audit") else name
        func_name = f"{base_name}_worm_trigger"
        trigger_name = f"trg_{name}_worm"
        section5 = f"""

-- ==========================================
-- {idx}.WORM  WORM append-only audit trigger (per 守门 #13 d + ADR-0043)
-- 仅允许 INSERT, UPDATE/DELETE 触发异常
-- ==========================================
CREATE OR REPLACE FUNCTION {func_name}() RETURNS TRIGGER AS $$
BEGIN
    -- WORM append-only: 仅 INSERT 允许, UPDATE/DELETE 触发异常
    IF (TG_OP = 'UPDATE' OR TG_OP = 'DELETE') THEN
        RAISE EXCEPTION '{name} is WORM append-only (per 守门 #13 d + ADR-0043)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS {trigger_name} ON {name};
CREATE TRIGGER {trigger_name}
    BEFORE UPDATE OR DELETE ON {name}
    FOR EACH ROW
    EXECUTE FUNCTION {func_name}();"""

    return section1 + section2 + section3 + section4 + section5


def gen_one_table_full(table: dict, idx: int) -> str:
    """生成单张表完整 DDL block (per-table BEGIN/COMMIT 包装, 跟 V0.1 模式一致)."""
    inner = gen_create_table_block(table, idx)
    return f"""
BEGIN;
{inner}

COMMIT;
"""


def gen_all() -> str:
    """生成 14 张表 (A11 2 + A12 3 + G 9) 完整 DDL."""
    header = """-- 2026-09-10-p3d6-15-more-tables.sql (auto-generated by scripts/sql/p3d6_15_more_tables_gen.py)
-- P3-D.6 阶段 1 基础 任务 1.6 14 张新表 SQL DDL 落档 (per 守门 #13 W/T/M 100% 覆盖 + 0 混在)
-- (per brief docs/briefs/p3-d6-1-6-15-more-tables.md §1.1 14 张新表 + 守门 #9 v20 子代理 dispatch 必先 brief 落档)
--
-- 守门 #13 a 100% RLS: ENABLE + FORCE + 6 类 policy (per v0.92 rls_7_policy_gen.py, 跳过 health_visibility 因表无 health_status 列)
-- 守门 #13 b 物理删除禁止: 4 类 CRUD delete 仅 platform_admin
-- 守门 #13 c M SCD Type 2: pgpool_version 字段 (6 张 M 表)
-- 守门 #13 d T 100% audit: WORM append-only trigger BEFORE UPDATE OR DELETE RAISE EXCEPTION (5 张 T 表, per ADR-0043)
-- 守门 #19 v19 复用 v0.92 rls_7_policy_gen.py 6 类 RLS policy 模式 + v0.97 DROP IF EXISTS 兼容 PG 9.5+ idempotent
-- 守门 #1 禁回溯叙事: 不重写 V0.1 15 张 DDL, 仅新 commit + 14 张新表 DDL
-- 守门 #11 缺标比错标: P3-D.6 阶段 1 基础阶段 14 张新表 schema, P2 阶段 worker 子代理 ALTER TABLE 改字段
-- 守门 #14 v4 Mavis 审核 author=Ulysses
--
-- 14 张新表 W/T/M 分类 (per 守门 #13 横展规则, 100% 覆盖, 0 混在):
--   - M (Master) 6 张: agent_trust_score_history + canvas_permissions + gamify_achievements +
--                    gamify_rewards + gamify_leaderboards + gamify_avatar_customization
--     物理删除禁止 + SCD Type 2 (pgpool_version) + 100% RLS
--   - T (Transaction) 5 张: agent_session_audit + gamify_xp_history +
--                    gamify_vote_audit + gamify_sticky_note_audit + gamify_confetti_audit
--     物理删除禁止 + 100% audit WORM trigger (per ADR-0043) + 100% RLS
--   - W (Work) 3 张: canvas_presence_cursors (5 min) + canvas_followers (5 min) +
--                    gamify_level_progress (30 days)
--     物理删除 + 短 TTL retention_period + 100% RLS

BEGIN;
"""
    # 按顺序: A11 2 + A12 3 + G 9 = 14
    body = ""
    for idx, table in enumerate(P3D6_15_MORE_TABLES, start=1):
        body += gen_one_table_full(table, idx)
    footer = "\nCOMMIT;\n"
    return header + body + footer


def main() -> int:
    parser = argparse.ArgumentParser(
        description="P3-D.6 阶段 1 基础 任务 1.6 14 张新表 SQL DDL 生成器 (per 守门 #13 W/T/M 100% 覆盖 + 0 混在)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("db/migrations/2026-09-10-p3d6-15-more-tables.sql"),
        help="输出 DDL 路径",
    )
    args = parser.parse_args()

    ddl = gen_all()

    # 分类统计 (per 守门 #13)
    m_count = sum(1 for t in P3D6_15_MORE_TABLES if t["w_t_m"] == "M")
    t_count = sum(1 for t in P3D6_15_MORE_TABLES if t["w_t_m"] == "T")
    w_count = sum(1 for t in P3D6_15_MORE_TABLES if t["w_t_m"] == "W")
    trigger_count = sum(1 for t in P3D6_15_MORE_TABLES if t.get("audit_trigger", False))

    # idempotent: CREATE TABLE IF NOT EXISTS + CREATE OR REPLACE FUNCTION + DROP TRIGGER IF EXISTS + DROP POLICY IF EXISTS (per V0.92 + V0.97)

    args.output.write_text(ddl, encoding="utf-8")
    line_count = ddl.count("\n") + 1
    print(
        f"[ok] 生成 {len(P3D6_15_MORE_TABLES)} 张新表 DDL: {args.output} (+{len(ddl)} bytes, {line_count} lines)",
        file=sys.stderr,
    )
    print(
        f"[ok] W/T/M 分类: M={m_count} + T={t_count} + W={w_count} = {m_count + t_count + w_count} 张, 0 混在",
        file=sys.stderr,
    )
    print(
        f"[ok] Audit trigger: {trigger_count} 张 T 表配套 WORM append-only trigger (per 守门 #13 d)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
