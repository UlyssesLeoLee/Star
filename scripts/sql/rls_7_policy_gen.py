#!/usr/bin/env python3
"""v0.92 P0-4 Stage 3.8 = 7 类 RLS policy 模板生成器 (per v0.91 命名)

跨 session 续做 v0.91 §3 已知缺口 (a) 'P3-D.6 14+15 张表落地时, 7 类 policy 模板需走 v0.91 命名'.

Usage (P2 阶段 worker 子代理 + k3s-deployable):
    python scripts/sql/rls_7_policy_gen.py \\
        --tables tenant_pools,ops_metrics_config,oauth_clients \\
        --output db/migrations/2026-09-10-rls-7policy-all-tables.sql

P0-4 阶段: 脚本声明 + 11 star-pg-adapter Repository 表名 hardcode 默认值.
P2 阶段: 实际生成 14+15 张表 (P3-D.6) DDL 跑 + 应用 + 测.

守门 #19 v19 批量改: 1 脚本 + N 输出, 走幂等 (per v0.74/v0.75 模式).
守门 #1 禁回溯叙事: 不重写 v0.87 已有 tenant_pools DDL, 仅生成新表 DDL.
守门 #11 缺标比错标: P0-4 阶段只生成模板, P2 阶段 worker 子代理实测 testcontainers.

7 类 policy 命名 (per v0.91 §8 命名修正):
1. {table}_select (跨 tenant_id 隔离 + admin bypass)
2. {table}_insert (必填当前 tenant_id)
3. {table}_update (跨 tenant_id 隔离 + admin bypass)
4. {table}_delete (仅 platform_admin, 守门 #13 b 物理删除禁止)
5. {table}_schema_isolation (跨 schema_name 隔离, per §13.5)
6. {table}_platform_admin (TO platform_admin 跨所有约束, per v0.88 BYPASSRLS)
7. {table}_health_visibility (5 health_status OR 合并, 仅适用含 health_status 列的表)
"""
import argparse
import sys
from pathlib import Path


# 11 star-pg-adapter Repository 表 (per v0.67 + v0.43 + v0.82 收官)
# 跟 db/migrations/2026-09-08-ops-*.sql + 2026-09-09-oauth2-server.sql + 2026-09-10-tenant-pools.sql 1:1 对应
DEFAULT_TABLES = [
    # 6 ops (per v0.67 6/6 ops Repository 收官)
    "ops_metrics_config",
    "ops_cluster_action_log",
    "ops_helm_release_state",
    "ops_log_entry",
    "ops_log_analysis",
    "ops_log_query_log",
    # 4 oauth (per v0.43 OAuth2 server + v0.47 5 endpoints 收官)
    "oauth_clients",
    "oauth_authorization_codes",
    "oauth_access_tokens",
    "oauth_refresh_tokens",
    # 1 tenant_pool (per v0.82 P0-4 Stage 3.1)
    "tenant_pools",
]

# P3-D.6 15 张占位表名 (per CANVAS-IMPL-PLAN-001 §1.6 '14+15 张表 SQL DDL 落地', 实际 15 张 G 域)
# v0.94 P0-4 Stage 4.0 跨 session 续做 v0.92 §3 已知缺口 (b) '11 Repository 表 跟 P3-D.6 14+15 张表有重叠':
# P0-4 阶段占位表名 (P2 阶段 worker 子代理 + P3-D.6 阶段 2 任务 2.x 实跑时 fill 真实表名).
# 走守门 #19 v19 复用 v0.92 rls_7_policy_gen.py 模式.
P3D6_PLACEHOLDER_TABLES = [
    # A11 域 (agent 域 5 表, per DD-CANVAS-AGENT-001 §3.1)
    "agent_sessions",        # Agent 会话 (per DD-CANVAS-AGENT-001 §4.14.1)
    "agent_policies",        # Agent policies (per DD §3.2)
    "agent_actions",         # Agent actions audit (per DD §3.3)
    "agent_relationships",   # Agent 关系 (per ARG.11 5 域 Lead)
    "agent_trust_scores",    # Agent trust score (per ARG.G-4)
    # A12 域 (canvas-collab 域 4 表, per DD §3.1, 协调性检查 v0.63 命名修正)
    "canvas_elements",       # Canvas 元素 (per C-25 CanvasElementsBackend)
    "canvas_multi_user_audit",  # Canvas 多用户 audit (per C-26, 守门 #13 d T 100% audit)
    "canvas_reactions",      # Canvas reactions (per G7 reaction)
    "canvas_comments",       # Canvas comments (per A12.4)
    # G 域 (gamify 6 表, per SRS-CANVAS-GAMIFY-001)
    "gamify_avatars",         # G1 avatar
    "gamify_levels",          # G2 level
    "gamify_sticky_notes",    # G3 sticky
    "gamify_confetti",        # G4 confetti
    "gamify_votes",           # G5 vote
    "gamify_streaks",         # G6 streak
]

# 哪些表含 health_status 列 (走 7 类 policy #7 health_visibility)
# 其他表跳过 #7 (避免 USING 引用不存在列)
TABLES_WITH_HEALTH_STATUS = {"tenant_pools"}

# 哪些表含 schema_name 列 (走 7 类 policy #5 schema_isolation)
# v0.93 P0-4 Stage 3.9 跨 session 续做 v0.92 §3 已知缺口 (d):
# 11 star-pg-adapter Repository 表 (除 tenant_pools) 默认 'public' 单 schema, 无 schema_name 列.
# P0-4 阶段跳过 #5 schema_isolation policy (避免 USING 引用不存在列, per 守门 #11 缺标比错标).
# P2 阶段扩展多 schema (per §13.5) 时, ALTER TABLE ADD COLUMN schema_name, 加进白名单.
TABLES_WITH_SCHEMA_NAME = {"tenant_pools"}


def gen_enable_rls(table: str) -> str:
    """生成 ENABLE + FORCE ROW LEVEL SECURITY (守门 #13 a 100% RLS)"""
    return f"""-- {table} RLS (per v0.91 §8 命名, 7 类 policy 模板)
ALTER TABLE {table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {table} FORCE ROW LEVEL SECURITY; -- superuser 也走 RLS (per ADR-0043)
"""


def gen_4_crud_policies(table: str) -> str:
    """生成 4 类 CRUD policy (跨 tenant_id 隔离 + admin bypass)"""
    return f"""-- {table} 4 类 CRUD policy (per v0.91 §8.3 映射表 1-4)
CREATE POLICY {table}_select ON {table}
    FOR SELECT
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

CREATE POLICY {table}_insert ON {table}
    FOR INSERT
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

CREATE POLICY {table}_update ON {table}
    FOR UPDATE
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

CREATE POLICY {table}_delete ON {table}
    FOR DELETE
    USING (
        current_setting('app.is_admin', true) = 'true'
    );
"""


def gen_schema_isolation_policy(table: str) -> str:
    """生成 schema isolation policy (per §13.5 多 schema 隔离)"""
    return f"""-- {table} schema isolation (per v0.91 §8.3 映射表 5)
CREATE POLICY {table}_schema_isolation ON {table}
    FOR ALL
    USING (
        schema_name = current_setting('app.current_schema_name', true)
        OR current_setting('app.is_admin', true) = 'true'
    );
"""


def gen_platform_admin_policy(table: str) -> str:
    """生成 platform admin override policy (per v0.88 BYPASSRLS)"""
    return f"""-- {table} platform admin override (per v0.91 §8.3 映射表 6 + v0.88 BYPASSRLS)
CREATE POLICY {table}_platform_admin ON {table}
    FOR ALL
    TO platform_admin
    USING (true)
    WITH CHECK (true);
"""


def gen_health_visibility_policy(table: str) -> str:
    """生成 5 health_status OR 合并 visibility policy (per v0.91 §8.3 映射表 7)"""
    return f"""-- {table} health_status 派生 visibility (per v0.91 §8.3 映射表 7, 5 status OR 合并)
CREATE POLICY {table}_health_visibility ON {table}
    FOR SELECT
    USING (
        (health_status = 'Healthy' AND current_setting('app.can_see_healthy', true) = 'true')
        OR (health_status = 'Degraded' AND current_setting('app.can_see_degraded', true) = 'true')
        OR (health_status = 'Down' AND current_setting('app.can_see_down', true) = 'true')
        OR (health_status = 'Deleted' AND current_setting('app.can_see_deleted', true) = 'true')
        OR (health_status = 'NotChecked' AND current_setting('app.can_see_notchecked', true) = 'true')
        OR current_setting('app.is_admin', true) = 'true'
    );
"""


def gen_one_table(table: str) -> str:
    """生成 1 表的 7 类 RLS policy 完整 DDL"""
    has_health = table in TABLES_WITH_HEALTH_STATUS
    has_schema = table in TABLES_WITH_SCHEMA_NAME
    parts = [
        f"\n-- ==========================================",
        f"-- {table} 7 类 RLS policy (per v0.91 命名)",
        f"-- ==========================================\n",
        gen_enable_rls(table),
        gen_4_crud_policies(table),
    ]
    if has_schema:
        parts.append(gen_schema_isolation_policy(table))
    else:
        # v0.93 修 v0.92 §3 已知缺口 (d): 跳过 #5 schema_isolation policy (表无 schema_name 列)
        parts.append(
            f"-- {table} schema_isolation 跳过 (表无 schema_name 列, per v0.93 缺口 (d) 修)\n"
            f"-- P2 阶段扩展多 schema (per §13.5) 时 ALTER TABLE ADD COLUMN schema_name + 加进 TABLES_WITH_SCHEMA_NAME 白名单\n"
        )
    parts.append(gen_platform_admin_policy(table))
    if has_health:
        parts.append(gen_health_visibility_policy(table))
    return "\n".join(parts) + "\n"


def gen_all(tables: list[str]) -> str:
    """生成所有表的 7 类 RLS policy 完整 DDL"""
    header = """-- 2026-09-10-rls-7policy-all-tables.sql (auto-generated by scripts/sql/rls_7_policy_gen.py)
-- v0.92 P0-4 Stage 3.8 = 7 类 RLS policy 模板生成器 (per v0.91 命名)
-- (per v0.91 §3 已知缺口 (a) 'P3-D.6 14+15 张表落地时, 7 类 policy 模板需走 v0.91 命名' 跨 session 续做)
--
-- 守门 #13 a 100% RLS: ENABLE + FORCE
-- 守门 #13 b 物理删除禁止: 4 类 CRUD delete 仅 platform_admin
-- 守门 #13 c M SCD Type 2: 跟 v0.82 pgpool_version 字段协同
-- 守门 #13 d T 100% audit: audit trigger 跨 RLS 写 (per ADR-0043 WORM)
-- 守门 #19 v19 批量改: 1 脚本 + N 输出, 走幂等
-- 守门 #1 禁回溯叙事: 不重写 v0.87 已有 tenant_pools DDL, 仅生成新表 DDL
-- 守门 #11 缺标比错标: P0-4 阶段只生成模板, P2 阶段 worker 子代理 + testcontainers 实测

BEGIN;

"""
    body = "".join(gen_one_table(t) for t in tables)
    footer = "\nCOMMIT;\n"
    return header + body + footer


def main() -> int:
    parser = argparse.ArgumentParser(
        description="v0.92 7 类 RLS policy 模板生成器 (per v0.91 命名, 11 Repository 表 + 15 P3-D.6 占位表默认)",
    )
    parser.add_argument(
        "--tables",
        default=",".join(DEFAULT_TABLES),
        help=f"逗号分隔表名 (默认 11 Repository 表: {','.join(DEFAULT_TABLES)})",
    )
    parser.add_argument(
        "--p3d6",
        action="store_true",
        help=f"扩展 P3-D.6 15 张占位表 (per v0.94 跨 session 续做 v0.92 §3 已知缺口 (b))",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("db/migrations/2026-09-10-rls-7policy-all-tables.sql"),
        help="输出 DDL 路径 (默认 db/migrations/2026-09-10-rls-7policy-all-tables.sql)",
    )
    args = parser.parse_args()

    tables = [t.strip() for t in args.tables.split(",") if t.strip()]
    if args.p3d6:
        tables = tables + P3D6_PLACEHOLDER_TABLES
        print(
            f"[v0.94] 扩展 P3-D.6 {len(P3D6_PLACEHOLDER_TABLES)} 张占位表, 总 {len(tables)} 张",
            file=sys.stderr,
        )
    ddl = gen_all(tables)
    args.output.write_text(ddl, encoding="utf-8")
    print(f"[ok] 生成 {len(tables)} 表 7 类 RLS policy DDL: {args.output} (+{len(ddl)} bytes)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
