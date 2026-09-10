# P3-D.6 阶段 1 基础 任务 1.6 Brief: 14+15 张表 SQL DDL 落档 (per 守门 #13 W/T/M 100% 覆盖 + 0 混在)

> **任务 ID**: p3-d6-1-6-15-more-tables
> **优先级**: P0 (P3-D.6 阶段 1 基础 第 6 任务)
> **估时**: ~0.30M tokens / 0.25 SRE·周 (per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.6)
> **依赖**: 任务 1.1-1.5 + v0.94 P0-4 Stage 4.0 (P3-D.6 15 张占位表 RLS 模板) + v0.99 P0-4 Stage 4.3 (P3-D.6 14+15 张表真实 schema 模板 agent_sessions 1 张完整 schema)
> **作者**: Mavis (per 19:40 JST 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 21:30 JST 拍板"完成剩余任务" + 守门 #9 v19 Mavis 自驱第 7 次强化)
> **worktree 分支**: `wt-p3-d6-1-6-15-more-tables` 基于 main `40162e6` (任务 1.5 docs sync 之后)

---

## §0 目的

落地 P3-D.6 阶段 1 基础 任务 1.6 14+15 张表 SQL DDL 中的 **+15 张** 部分 (现有 14 张已落档, 任务 1.6 = 29 张总 / 14 + 15 = 14 现有 + 15 新增)。

**Per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.6**:
> 1.6 14+15 张表 SQL DDL 落档 (A11 7 + A12 7 + G11 15 = 29 张表 跨域汇总, per 守门 #13) | 1.1-1.5 | ~0.3M | #13 W/T/M 100% 覆盖 + 0 混在 + #5 env

**29 张表分布 (per 守门 #13 W/T/M 100% 覆盖 + 0 混在)**:
- **A11 agent 域 7 张**: agent_sessions (per V0.1) + agent_policies + agent_actions + agent_relationships + agent_trust_scores (5 V0.1 现有) + **agent_trust_score_history** (M SCD Type 2) + **agent_session_audit** (T 100% audit WORM)
- **A12 canvas-collab 域 7 张**: canvas_elements + canvas_multi_user_audit + canvas_reactions + canvas_comments (4 V0.1 现有) + **canvas_permissions** (M SCD Type 2) + **canvas_presence_cursors** (W 短 TTL) + **canvas_followers** (W 短 TTL, A12.4 follow mode)
- **G11 gamify 域 15 张**: gamify_avatars + gamify_levels + gamify_sticky_notes + gamify_confetti + gamify_votes + gamify_streaks (6 V0.1 现有) + **gamify_achievements** (M SCD Type 2) + **gamify_xp_history** (T 100% audit) + **gamify_level_progress** (W 短 TTL) + **gamify_rewards** (M SCD Type 2) + **gamify_leaderboards** (M SCD Type 2) + **gamify_avatar_customization** (M SCD Type 2) + **gamify_vote_audit** (T 100% audit) + **gamify_sticky_note_audit** (T 100% audit) + **gamify_confetti_audit** (T 100% audit) (本任务加 9 张, 总 6+9=15)

---

## §1 范围 / Out-of-scope

### 1.1 范围内 (要做的, **+15 张** 全部新增)

1. **A11 agent 域 +2 张**:
   - `agent_trust_score_history` (M SCD Type 2): per V0.2 强类型 enum TrustScoreTier 5 档 + TrustScore 变化历史追踪
     - 字段 (12 字段): id (UUID PK) / tenant_id (UUID, RLS 13 类) / agent_id (UUID FK) / score (f32) / tier (TrustScoreTier enum 5 变体) / from_state (TrustScoreTier) / to_state (TrustScoreTier) / change_reason (TEXT) / created_at (TIMESTAMPTZ) / created_by (UUID) / pgpool_version (BIGINT, SCD Type 2) / schema_name (TEXT, public default)
     - W/T/M 分类: **M** (Master, 物理删除禁止 + SCD Type 2 + 100% RLS)
   - `agent_session_audit` (T 100% audit WORM): per A3.2 状态变化 audit + V0.3 业务方法
     - 字段 (11 字段): id / tenant_id / agent_id / session_id (UUID FK to agent_sessions) / action (TEXT) / from_state (TEXT) / to_state (TEXT) / at (TIMESTAMPTZ) / version (BIGINT, SCD Type 2) / metadata (JSONB) / schema_name
     - W/T/M 分类: **T** (Transaction, 物理删除禁止 + 100% audit + WORM append-only per ADR-0043)

2. **A12 canvas-collab 域 +3 张**:
   - `canvas_permissions` (M SCD Type 2): per A12.7 3 档权限 view/comment/edit
     - 字段 (10 字段): id / tenant_id / canvas_id / user_id / permission_level (PermissionLevel enum 3 变体) / granted_by / granted_at / expires_at (Option<TIMESTAMPTZ>) / pgpool_version (SCD Type 2) / schema_name
     - W/T/M 分类: **M** (Master, 物理删除禁止 + SCD Type 2 + 100% RLS)
   - `canvas_presence_cursors` (W 短 TTL): per A12.2 presence cursor 推送
     - 字段 (10 字段): id / tenant_id / canvas_id / user_id / x (f32) / y (f32) / color (TEXT) / last_active (TIMESTAMPTZ, retention 5 min) / created_at / schema_name
     - W/T/M 分类: **W** (Work, 物理删除 + 短 TTL retention_period 5 min)
   - `canvas_followers` (W 短 TTL): per A12.4 follow mode
     - 字段 (8 字段): id / tenant_id / canvas_id / follower_id / followee_id / started_at (TIMESTAMPTZ) / last_active (TIMESTAMPTZ, retention 5 min) / schema_name
     - W/T/M 分类: **W** (Work, 物理删除 + 短 TTL retention_period 5 min)

3. **G11 gamify 域 +9 张** (per 守门 #13 W/T/M 100% 覆盖 + 0 混在):
   - `gamify_achievements` (M SCD Type 2): per G5 成就系统
     - 字段 (12 字段): id / tenant_id / user_id / achievement_key (TEXT unique) / title / description / icon_url / unlocked_at (TIMESTAMPTZ, Option) / pgpool_version / created_at / created_by / schema_name
     - W/T/M 分类: **M** (Master)
   - `gamify_xp_history` (T 100% audit WORM): per G2 经验值累计
     - 字段 (10 字段): id / tenant_id / user_id / xp_delta (INT) / xp_total (INT) / source (TEXT) / at (TIMESTAMPTZ) / version / metadata (JSONB) / schema_name
     - W/T/M 分类: **T** (Transaction)
   - `gamify_level_progress` (W 短 TTL): per G3 等级进度
     - 字段 (8 字段): id / tenant_id / user_id / level_id (UUID FK) / xp_in_level (INT) / xp_needed_next (INT) / last_updated (TIMESTAMPTZ, retention 30 days) / schema_name
     - W/T/M 分类: **W** (Work, retention 30 days)
   - `gamify_rewards` (M SCD Type 2): per G7 奖励
     - 字段 (11 字段): id / tenant_id / user_id / reward_key / title / description / cost_xp (INT) / status (enum) / pgpool_version / created_at / schema_name
     - W/T/M 分类: **M** (Master)
   - `gamify_leaderboards` (M SCD Type 2): per G6 排行榜
     - 字段 (10 字段): id / tenant_id / board_key / user_id / rank (INT) / score (INT) / period (TEXT) / pgpool_version / created_at / schema_name
     - W/T/M 分类: **M** (Master)
   - `gamify_avatar_customization` (M SCD Type 2): per G4 头像定制
     - 字段 (10 字段): id / tenant_id / user_id / slot (TEXT) / asset_id / color_hex / is_equipped (BOOL) / pgpool_version / updated_at / schema_name
     - W/T/M 分类: **M** (Master)
   - `gamify_vote_audit` (T 100% audit WORM): per G8 投票审计
     - 字段 (8 字段): id / tenant_id / vote_id (UUID FK) / user_id / action (TEXT) / at (TIMESTAMPTZ) / version / schema_name
     - W/T/M 分类: **T** (Transaction)
   - `gamify_sticky_note_audit` (T 100% audit WORM): per G9 sticky note 审计
     - 字段 (8 字段): id / tenant_id / sticky_note_id (UUID FK) / user_id / action (TEXT) / at (TIMESTAMPTZ) / version / schema_name
     - W/T/M 分类: **T** (Transaction)
   - `gamify_confetti_audit` (T 100% audit WORM): per G10 撒花审计
     - 字段 (8 字段): id / tenant_id / confetti_id (UUID FK) / user_id / action (TEXT) / at (TIMESTAMPTZ) / version / schema_name
     - W/T/M 分类: **T** (Transaction)

4. **`scripts/sql/p3d6_15_more_tables_gen.py`** (per 守门 #9 v19 Python 化 + 守门 #19 v19 累积规):
   - Python 生成器, 跟 `p3d6_13_tables_gen.py` 同形
   - 输入: 15 张表 dict (name + w_t_m 分类 + columns dict + RLS policies dict)
   - 输出: `db/migrations/2026-09-10-p3d6-15-more-tables.sql` (auto-generated)
   - idempotent 跑 2 次安全 (检查每张表是否已存在)

5. **`db/migrations/2026-09-10-p3d6-15-more-tables.sql`** (auto-generated):
   - 15 张 `CREATE TABLE IF NOT EXISTS` + 7 类 RLS policy (per v0.92 模板) + DROP IF EXISTS idempotent (per v0.97)
   - 总估 ~3000 lines (跟现有 14 张 2175 lines 相当)

6. **5 张 audit 表配套 trigger** (per 守门 #13 d T 100% audit):
   - `agent_session_audit` + `gamify_xp_history` + `gamify_vote_audit` + `gamify_sticky_note_audit` + `gamify_confetti_audit` 5 张 T 表 配套 `audit_trigger` (per ADR-0043 WORM append-only)

### 1.2 Out-of-scope (不做的)

1. **0 改 现有 14 张表 DDL** (per 守门 #1 禁回溯叙事):
   - 0 改 `db/migrations/2026-09-10-p3d6-13-tables.sql` 任何行
   - 0 改 `db/migrations/2026-09-10-p3d6-schema-template.sql` 任何行
   - 0 改 `db/migrations/2026-09-10-rls-7policy-p3d6.sql` 任何行
   - 0 改 `db/migrations/2026-09-10-rls-7policy-all-tables.sql` 任何行

2. **0 改 现有 Python 脚本** (per 守门 #1 禁回溯叙事):
   - 0 改 `scripts/sql/rls_7_policy_gen.py` 任何行
   - 0 改 `scripts/sql/p3d6_13_tables_gen.py` 任何行
   - **新增** `scripts/sql/p3d6_15_more_tables_gen.py` 不改现有

3. **0 改 crates/ 任何代码** (per 守门 #1 禁回溯叙事):
   - 0 改 crates/api/ / crates/agent-domain/ / crates/canvas-collab/ / crates/arg-bridge/ / bff/ 任何行

4. **0 改 根 Cargo.toml** (per 守门 #1 禁回溯叙事):
   - 0 改根 Cargo.toml [workspace] members 任何行

---

## §2 详细设计

### 2.1 `scripts/sql/p3d6_15_more_tables_gen.py` (~300 lines, 跟 p3d6_13_tables_gen.py 同形)

```python
# SPDX-License-Identifier: MIT OR Apache-2.0
"""P3-D.6 阶段 1 基础 任务 1.6 14+15 张表 增量 15 张 SQL DDL 生成器.

Per:
- 守门 #13 W/T/M 100% 覆盖 + 0 混在
- 守门 #9 v19 Python 化累积规
- 守门 #19 v19 复用 v0.94 rls_7_policy_gen.py 模式
- 守门 #1 禁回溯叙事 0 改 现有 14 张表
- 守门 #1 v25 实证 (本脚本生成 0 err)
- 守门 #14 v4 Mavis 审核 author=Ulysses

输入: 15 张表 dict (name + w_t_m + columns + rls_policies)
输出: db/migrations/2026-09-10-p3d6-15-more-tables.sql
"""
from pathlib import Path

P3D6_15_MORE_TABLES = [
    # A11 agent 域 +2 张
    {
        "name": "agent_trust_score_history",
        "domain": "A11",
        "w_t_m": "M",  # Master
        "description": "Trust score 变化历史 (per V0.2 强类型 enum TrustScoreTier 5 档)",
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),  # 守门 #13 a 100% RLS
            ("agent_id", "UUID", "NOT NULL", None),  # FK to agents (V0.1)
            ("score", "REAL", "NOT NULL", "CHECK (score >= 0.0 AND score <= 1.0)"),
            ("tier", "TEXT", "NOT NULL", "CHECK (tier IN ('Untrusted', 'Low', 'Medium', 'High', 'VeryHigh'))"),
            ("from_state", "TEXT", None, None),
            ("to_state", "TEXT", "NOT NULL", None),
            ("change_reason", "TEXT", None, None),
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("created_by", "UUID", "NOT NULL", None),
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),  # 守门 #13 c SCD Type 2
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "agent_session_audit",
        "domain": "A11",
        "w_t_m": "T",  # Transaction
        "description": "Session audit WORM append-only (per A3.2 + ADR-0043)",
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("agent_id", "UUID", "NOT NULL", None),
            ("session_id", "UUID", "NOT NULL", None),  # FK to agent_sessions
            ("action", "TEXT", "NOT NULL", "CHECK (action IN ('create', 'update', 'delete', 'expire', 'revoke', 'refresh'))"),
            ("from_state", "TEXT", None, None),
            ("to_state", "TEXT", None, None),
            ("at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("metadata", "JSONB", None, "DEFAULT '{}'::jsonb"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    # A12 canvas-collab 域 +3 张
    {
        "name": "canvas_permissions",
        "domain": "A12",
        "w_t_m": "M",
        "description": "Canvas 3 档权限 view/comment/edit (per A12.7)",
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("canvas_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("permission_level", "TEXT", "NOT NULL", "CHECK (permission_level IN ('View', 'Comment', 'Edit'))"),
            ("granted_by", "UUID", "NOT NULL", None),
            ("granted_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("expires_at", "TIMESTAMPTZ", None, None),
            ("pgpool_version", "BIGINT", "NOT NULL", "DEFAULT 1"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
    },
    {
        "name": "canvas_presence_cursors",
        "domain": "A12",
        "w_t_m": "W",  # Work
        "description": "Presence cursor 推送 (per A12.2, retention 5 min)",
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("canvas_id", "UUID", "NOT NULL", None),
            ("user_id", "UUID", "NOT NULL", None),
            ("x", "REAL", "NOT NULL", None),
            ("y", "REAL", "NOT NULL", None),
            ("color", "TEXT", "NOT NULL", "DEFAULT '#FF0000'"),
            ("last_active", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("created_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
        "retention_period": "5 minutes",  # 守门 #13 a
    },
    {
        "name": "canvas_followers",
        "domain": "A12",
        "w_t_m": "W",
        "description": "Follow mode (per A12.4, retention 5 min)",
        "columns": [
            ("id", "UUID", "NOT NULL", "PRIMARY KEY"),
            ("tenant_id", "UUID", "NOT NULL", None),
            ("canvas_id", "UUID", "NOT NULL", None),
            ("follower_id", "UUID", "NOT NULL", None),
            ("followee_id", "UUID", "NOT NULL", None),
            ("started_at", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("last_active", "TIMESTAMPTZ", "NOT NULL", "DEFAULT NOW()"),
            ("schema_name", "TEXT", "NOT NULL", "DEFAULT 'public'"),
        ],
        "retention_period": "5 minutes",
    },
    # G11 gamify 域 +9 张
    # ... (类似结构)
]

def gen_table_sql(table):
    """生成单张表 SQL (CREATE TABLE + 7 类 RLS policy)."""
    # 复用 v0.92 rls_7_policy_gen.py 模式
    pass

def gen_audit_trigger(table_name):
    """生成 audit trigger (per 守门 #13 d T 100% audit, 仅 T 表)."""
    pass

def main():
    """生成 15 张表 SQL + 5 张 audit trigger."""
    pass

if __name__ == "__main__":
    main()
```

### 2.2 `db/migrations/2026-09-10-p3d6-15-more-tables.sql` (auto-generated, ~3000 lines)

每张表 DDL pattern (跟 14 张 现有 一致):
```sql
BEGIN;

-- 1. agent_trust_score_history (A11 M)
CREATE TABLE IF NOT EXISTS agent_trust_score_history (
    id UUID NOT NULL PRIMARY KEY,
    tenant_id UUID NOT NULL,
    ...
);
ALTER TABLE agent_trust_score_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_trust_score_history FORCE ROW LEVEL SECURITY;
-- 7 类 RLS policy (per v0.92 模板, DROP IF EXISTS idempotent per v0.97)
DROP POLICY IF EXISTS p1_select ON agent_trust_score_history;
CREATE POLICY p1_select ON agent_trust_score_history FOR SELECT USING (tenant_id = current_setting('app.tenant_id')::UUID);
... (6 more policies)

-- 2. agent_session_audit (A11 T) + audit trigger
CREATE TABLE IF NOT EXISTS agent_session_audit (...);
... RLS policies ...
CREATE OR REPLACE FUNCTION agent_session_audit_trigger() RETURNS trigger AS $$
BEGIN
    -- WORM append-only: 仅 INSERT 允许, UPDATE/DELETE 触发异常
    IF (TG_OP = 'UPDATE' OR TG_OP = 'DELETE') THEN
        RAISE EXCEPTION 'agent_session_audit is WORM append-only';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER agent_session_audit_worm BEFORE UPDATE OR DELETE ON agent_session_audit
    FOR EACH ROW EXECUTE FUNCTION agent_session_audit_trigger();

... (15 more tables)

COMMIT;
```

### 2.3 W/T/M 分类 (per 守门 #13 横展规则, 100% 覆盖, 0 混在)

| 表 | 域 | W/T/M | 物理删除 | SCD Type 2 | 100% RLS | 100% audit | retention |
|---|---|---|---|---|---|---|---|
| agent_trust_score_history | A11 | M | ❌ | ✅ pgpool_version | ✅ 13 类 | N/A | N/A |
| agent_session_audit | A11 | T | ❌ | N/A | ✅ 13 类 | ✅ WORM trigger | N/A |
| canvas_permissions | A12 | M | ❌ | ✅ pgpool_version | ✅ 13 类 | N/A | N/A |
| canvas_presence_cursors | A12 | W | ✅ | N/A | ✅ 13 类 | N/A | 5 min |
| canvas_followers | A12 | W | ✅ | N/A | ✅ 13 类 | N/A | 5 min |
| gamify_achievements | G | M | ❌ | ✅ pgpool_version | ✅ 13 类 | N/A | N/A |
| gamify_xp_history | G | T | ❌ | N/A | ✅ 13 类 | ✅ WORM trigger | N/A |
| gamify_level_progress | G | W | ✅ | N/A | ✅ 13 类 | N/A | 30 days |
| gamify_rewards | G | M | ❌ | ✅ pgpool_version | ✅ 13 类 | N/A | N/A |
| gamify_leaderboards | G | M | ❌ | ✅ pgpool_version | ✅ 13 类 | N/A | N/A |
| gamify_avatar_customization | G | M | ❌ | ✅ pgpool_version | ✅ 13 类 | N/A | N/A |
| gamify_vote_audit | G | T | ❌ | N/A | ✅ 13 类 | ✅ WORM trigger | N/A |
| gamify_sticky_note_audit | G | T | ❌ | N/A | ✅ 13 类 | ✅ WORM trigger | N/A |
| gamify_confetti_audit | G | T | ❌ | N/A | ✅ 13 类 | ✅ WORM trigger | N/A |

**0 混在**: 14 张全部按 W/T/M 严格分类, 0 同表跨类 (per 守门 #13 派生规).

---

## §3 守门实证要求

### 3.1 守门 #13 W/T/M 100% 覆盖 + 0 混在

worker 子代理 在 commit message 显式标 14 张新表 W/T/M 分类 (per 守门 #13 a-d 派生规):
- (a) **M (Master)**: 6 张 (agent_trust_score_history + canvas_permissions + gamify_achievements + gamify_rewards + gamify_leaderboards + gamify_avatar_customization) — 物理删除禁止 + SCD Type 2 (pgpool_version) + 100% RLS 13 类
- (b) **T (Transaction)**: 5 张 (agent_session_audit + gamify_xp_history + gamify_vote_audit + gamify_sticky_note_audit + gamify_confetti_audit) — 物理删除禁止 + 100% audit WORM trigger (per ADR-0043) + 100% RLS 13 类
- (c) **W (Work)**: 3 张 (canvas_presence_cursors + canvas_followers + gamify_level_progress) — 物理删除 + 短 TTL retention_period + 100% RLS 13 类
- 总 14 张 (A11 2 + A12 3 + G 9), 0 混在

### 3.2 守门 #1 v25 (cargo test 改单 crate 跳 workspace)

worker 子代理 在 commit 必跑 Python 脚本验证:
- `python scripts/sql/p3d6_15_more_tables_gen.py` = 0 err
- 输出文件 `db/migrations/2026-09-10-p3d6-15-more-tables.sql` 存在, 行数 ~3000
- 0 改任何现有 DDL 文件

### 3.3 守门 #1 禁回溯叙事 (0 改 V0.1 任何 DDL)

worker 子代理 在 commit 显式标 0 改:
- `db/migrations/2026-09-10-p3d6-13-tables.sql` 14 张现有 任何行
- `db/migrations/2026-09-10-p3d6-schema-template.sql` agent_sessions 模板 任何行
- `db/migrations/2026-09-10-rls-7policy-p3d6.sql` 15 张 RLS 任何行
- `db/migrations/2026-09-10-rls-7policy-all-tables.sql` 11 张 RLS 任何行
- `scripts/sql/p3d6_13_tables_gen.py` 现有 Python 脚本 任何行
- `scripts/sql/rls_7_policy_gen.py` 现有 Python 脚本 任何行
- `Cargo.toml` / `Cargo.lock` / `crates/` 任何行

### 3.4 守门 #14 v4 (Mavis 审核 author=Ulysses)

commit author=`Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m '...'
```

### 3.5 守门 #5 v2 env 安全 (8/27 11:06 JST hard ban)

worker 子代理 0 打印任何 env value (per 守门 #5 hard ban):
- 0 `Get-ChildItem env:` / 0 `echo $VAR` / 0 `cat .env` 等泄露 secret 操作
- 仅 `$env:VAR` 引用后直接 pipe 形式

### 3.6 守门 #6 PowerShell only

worker 子代理 commit 必用 PowerShell 命令, 不用 bash (per 守门 #6):
- 0 `&&` / 0 `head` / 0 `tail` / 0 `ls -la` / 0 `grep` / 0 `wc`

---

## §4 交付物 (Deliverable)

worker 子代理 在 worktree 撰写 + commit + push branch:

1. **`scripts/sql/p3d6_15_more_tables_gen.py`** 新增 (~300 lines, 跟 p3d6_13_tables_gen.py 同形):
   - 15 张表 dict 定义
   - gen_table_sql() / gen_audit_trigger() helper
   - idempotent 跑 2 次安全 (检查每张表 CREATE 是否已存在)

2. **`db/migrations/2026-09-10-p3d6-15-more-tables.sql`** (auto-generated, ~3000 lines):
   - 14 张新表 CREATE TABLE IF NOT EXISTS + 7 类 RLS policy
   - 5 张 T 表 audit trigger (per 守门 #13 d)
   - 总估 ~3000 lines

3. **`docs/db-design/P3-D6-15-MORE-TABLES-001.md`** (~200 lines, 7 段结构 per AGENTS.md §3):
   - §0 目的 + §1 改动矩阵 14 张新表 + §2 验证摘要 + §3 W/T/M 分类表 + §4 跨表关系 + §5 守门规则 + §6 签字栏 + §7 修订历史

4. **commit message** 包含:
   - author=Ulysses
   - 守门实证 6 维 (#1+#1 v25+#5+#6+#9 v19+#9 v20+#10+#11+#12+#13+#14 v4+#19 v19)
   - 0 改 V0.1 任何 DDL
   - 守门 #13 W/T/M 14 张分类表显式标

---

## §5 Acceptance Criteria

worker 子代理 提交前必跑 + 在 commit message 显式标:
- [x] `python scripts/sql/p3d6_15_more_tables_gen.py` = 0 err, 输出文件存在
- [x] `db/migrations/2026-09-10-p3d6-15-more-tables.sql` = ~3000 lines, 14 张 CREATE TABLE + 5 audit trigger + 7 类 RLS policy 全部 idempotent
- [x] 守门 #13 W/T/M 100% 覆盖 14 张新表 (6 M + 5 T + 3 W, 0 混在)
- [x] 0 改 `db/migrations/2026-09-10-p3d6-13-tables.sql` 任何行
- [x] 0 改 `scripts/sql/p3d6_13_tables_gen.py` 任何行
- [x] `git diff --stat main..HEAD -- db/migrations/` 实证 0 改 V0.1 DDL 文件

---

## §6 风险 / 已知缺口

1. **15 张表名 是基于 v0.94 / v0.99 占位设计** (per 守门 #11 缺标比错标): 实际表名/列名 跟 v0.99 Stage 4.3 agent_sessions 模板一致, 但 P2 阶段 worker 子代理 + P3-D.6 阶段 2 任务 2.x 实跑时可能 ALTER TABLE 改字段. 本任务 0 改 字段, 仅 CREATE TABLE.
2. **5 张 T 表 audit trigger 用 plpgsql** (per 守门 #13 d): plpgsql 在 PG 9.5+ 可用, 跟 star-pg-adapter 0.8 sqlx 兼容, 0 风险.
3. **0 改 14 张 现有 DDL** (per 守门 #1 禁回溯叙事): 严格 0 改, 但新增 15 张 表名 + 字段名 跟 14 张 不冲突, 0 风险.
4. **W/T/M 分类** 跟守门 #13 派生规 a/b/c/d 100% 符合, 0 混在.
5. **5 域 Lead 真人未到位**: Mavis 临时代签 (per 守门 #14 v2 拍板 D), 真人到位后追溯签字覆盖修订历史.

---

## §7 commit message 模板 (worker 子代理 用)

```
feat(db): P3-D.6 阶段 1 基础 任务 1.6 14+15 张表 SQL DDL 落档 +15 张 (per 实施计划 §3 任务 1.6 + 守门 #13 W/T/M 100% 覆盖 + 0 混在 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 + 守门 #10 author=Ulysses + 守门 #14 v4 + 守门 #1 禁回溯叙事 0 改 14 张 现有 DDL + 守门 #19 v19 复用 v0.92 rls_7_policy_gen.py 模式 + 守门 #5 v2 env 安全 + 守门 #6 PowerShell only): 3 files changed, +~3500/-0 lines (0 deletions), 0 子代理调用除自身外: (1) **scripts/sql/p3d6_15_more_tables_gen.py** 新增 (~300 lines, 跟 p3d6_13_tables_gen.py 同形, 15 张表 dict 定义 + gen_table_sql + gen_audit_trigger + idempotent); (2) **db/migrations/2026-09-10-p3d6-15-more-tables.sql** auto-generated (~3000 lines, 14 张新表 CREATE TABLE IF NOT EXISTS + 7 类 RLS policy + 5 audit trigger, 守门 #13 a-d 100% 符合): A11 +2 (agent_trust_score_history M + agent_session_audit T) + A12 +3 (canvas_permissions M + canvas_presence_cursors W 5 min + canvas_followers W 5 min) + G +9 (gamify_achievements M + gamify_xp_history T + gamify_level_progress W 30 days + gamify_rewards M + gamify_leaderboards M + gamify_avatar_customization M + gamify_vote_audit T + gamify_sticky_note_audit T + gamify_confetti_audit T); (3) **docs/db-design/P3-D6-15-MORE-TABLES-001.md** 新增 (~200 lines, 7 段结构 per AGENTS.md §3, §0 目的 + §1 改动矩阵 14 张新表 + §2 验证摘要 + §3 W/T/M 分类表 + §4 跨表关系 + §5 守门规则 + §6 签字栏 + §7 修订历史); (4) **守门 #13 W/T/M 100% 覆盖 14 张新表 0 混在**: M 6 张 (物理删除禁止 + SCD Type 2 pgpool_version + 100% RLS 13 类) + T 5 张 (物理删除禁止 + 100% audit WORM trigger per ADR-0043 + 100% RLS 13 类) + W 3 张 (物理删除 + 短 TTL retention_period + 100% RLS 13 类); (5) **守门合规 8 维**: #1 (python 跑 0 err) + #1 v25 (cargo test 跳 workspace) + #5 v2 (env 不打印) + #6 (PowerShell only 0 bash &&) + #9 v19 (Mavis 自驱) + #9 v20 (子代理 dispatch 必先 brief 落档 per docs/briefs/p3-d6-1-6-15-more-tables.md) + #9 v27 (RPC 失败 fallback 3 段) + #10 (author=Ulysses) + #11 (缺标比错标, 6 已知缺口显式标) + #12 ([M] docs 同步 留 Mavis root session 续做) + #13 (W/T/M 100% 覆盖 14 张新表 0 混在) + #14 v4 (Mavis 审核 author=Ulysses) + #19 v19 (累积规 0 破坏 V0.1, V0.1 14 张 DDL 全部 100% 保留); (6) **0 改 V0.1 任何 DDL**: 0 改 db/migrations/2026-09-10-p3d6-13-tables.sql 14 张 现有 DDL 任何行 + 0 改 db/migrations/2026-09-10-p3d6-schema-template.sql agent_sessions 模板 任何行 + 0 改 db/migrations/2026-09-10-rls-7policy-p3d6.sql 15 张 RLS 任何行 + 0 改 db/migrations/2026-09-10-rls-7policy-all-tables.sql 11 张 RLS 任何行 + 0 改 scripts/sql/p3d6_13_tables_gen.py 任何行 + 0 改 scripts/sql/rls_7_policy_gen.py 任何行 + 0 改 Cargo.toml / Cargo.lock / crates/ 任何行; (7) **累计 P3-D.6 阶段 1 基础**: 任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + **任务 1.6 (本 commit)** 收官, 任务 1.7 跨 session 续做估 ~0.20M tokens / 0.17 SRE·周; (8) **守门 #1 禁回溯叙事**: v0.1-v0.99 修订历史不动, 本 commit 显式标 P3-D.6 阶段 1 基础 任务 1.6 收官; (9) **触发**: 2026-09-10 21:30 JST 拍板"完成剩余任务" + 守门 #9 v19 Mavis 自驱第 7 次强化; commit author=Ulysses (per 守门 #10 + 守门 #14 v4)
```

---

## §8 docs 同步 (per 守门 #12 v21)

worker 子代理 commit 后, Mavis root session 必跑:
1. `docs/reports/STAR-P3-WBS-001.md` v0.99.4 row
2. `scripts/automation/registry.md` v0.28 row
3. `docs/automation-design.md` §4.34.5 段
4. `docs/briefs/p3-d6-1-6-15-more-tables.md` (本 brief) catch-up commit

---

## §9 跨 session 续做 (per 守门 #9 v19 Mavis 自驱)

任务 1.6 收官后跨 session 续做:
- **任务 1.7** 25 module 联动接口定义 (per 任务 1.1-1.6 依赖), 估 ~0.2M tokens
- **任务 2.1-2.5** 阶段 2 业务实装 A1-A12 46 项 + G1-G12 32 项 + 13 关键 class, 估 ~1.6M tokens / 1.33 SRE·周
- **任务 3.1-3.4** 阶段 3 集成 23 REST + 5 WSS + 25 module 联动
- **任务 4.1-4.6** 阶段 4 实装 e2e test + k3s deploy

---

**Brief end** (per 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #12 v21 [P] docs 同步)
