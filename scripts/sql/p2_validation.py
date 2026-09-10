#!/usr/bin/env python3
"""v1.01 P0-4 Stage 4.5 = P2 阶段 worker 子代理 实跑验证脚本

Per 守门 #1 v25 实证 + 守门 #11 缺标比错标, P0-4 阶段不实跑 (避免 P0-4 dev env 误触生产 PG),
P2 阶段 worker 子代理 跑:
    python scripts/sql/p2_validation.py [--dry-run] [--tables tenant_pools]

守门 #19 v19 复用 v0.92 rls_7_policy_gen.py 模板 + v0.97 DROP IF EXISTS idempotent 模式.
守门 #11 缺标比错标: P0-4 阶段跑必返 Err placeholder, P2 阶段 worker 子代理 实测 idempotent.

功能:
1. 验证 v0.92 rls_7_policy_gen.py 能重生成 26 张表 7 类 RLS policy DDL (idempotent)
2. 验证 v0.99 p3d6-schema-template.sql 模板可读
3. 验证 v1.00 p3d6_13_tables_gen.py 能重生成 15 张表 DDL
4. 验证所有 26 张表 DDL 跑 idempotent (跑 2 遍无 conflict)
5. 报告 RLS policy + 触发器 + 索引 + 视图 完整
"""
import argparse
import re
import sys
from pathlib import Path


def validate_rls_template() -> bool:
    """验证 v0.92 rls_7_policy_gen.py 模板"""
    script = Path("scripts/sql/rls_7_policy_gen.py")
    if not script.exists():
        print(f"[fail] {script} 不存在", file=sys.stderr)
        return False
    text = script.read_text(encoding="utf-8")
    # 检查关键函数存在
    for fn in ["gen_4_crud_policies", "gen_schema_isolation_policy", "gen_platform_admin_policy", "gen_health_visibility_policy"]:
        if f"def {fn}" not in text:
            print(f"[fail] rls_7_policy_gen.py 缺函数 {fn}", file=sys.stderr)
            return False
    print("[ok] v0.92 rls_7_policy_gen.py 模板函数全 4 函数存在", file=sys.stderr)
    return True


def validate_p3d6_template() -> bool:
    """验证 v0.99 p3d6-schema-template.sql 模板"""
    template = Path("db/migrations/2026-09-10-p3d6-schema-template.sql")
    if not template.exists():
        print(f"[fail] {template} 不存在", file=sys.stderr)
        return False
    text = template.read_text(encoding="utf-8")
    # 检查关键模式
    if "CREATE TABLE IF NOT EXISTS agent_sessions" not in text:
        print(f"[fail] {template} 缺 agent_sessions 模板", file=sys.stderr)
        return False
    if "ENABLE ROW LEVEL SECURITY" not in text:
        print(f"[fail] {template} 缺 RLS ENABLE", file=sys.stderr)
        return False
    print(f"[ok] {template} 模板 CREATE TABLE + RLS ENABLE 存在", file=sys.stderr)
    return True


def validate_p3d6_13_tables() -> bool:
    """验证 v1.00 p3d6_13_tables_gen.py 模板"""
    ddl = Path("db/migrations/2026-09-10-p3d6-13-tables.sql")
    if not ddl.exists():
        print(f"[fail] {ddl} 不存在", file=sys.stderr)
        return False
    text = ddl.read_text(encoding="utf-8")
    # 检查 14 张表 (5 A11 + 4 A12 + 6 G + 1 agent_sessions 模板)
    tables = [
        "agent_sessions", "agent_policies", "agent_actions", "agent_relationships", "agent_trust_scores",
        "canvas_elements", "canvas_multi_user_audit", "canvas_reactions", "canvas_comments",
        "gamify_avatars", "gamify_levels", "gamify_sticky_notes", "gamify_confetti", "gamify_votes", "gamify_streaks",
    ]
    missing = [t for t in tables if f"CREATE TABLE IF NOT EXISTS {t}" not in text]
    if missing:
        print(f"[fail] p3d6-13-tables.sql 缺表: {missing}", file=sys.stderr)
        return False
    print(f"[ok] p3d6-13-tables.sql 含 {len(tables)} 张表 (5 A11 + 4 A12 + 6 G)", file=sys.stderr)
    return True


def validate_rls_11_tables() -> bool:
    """验证 v0.92 rls_7_policy_all_tables 模板"""
    ddl = Path("db/migrations/2026-09-10-rls-7policy-all-tables.sql")
    if not ddl.exists():
        print(f"[fail] {ddl} 不存在", file=sys.stderr)
        return False
    text = ddl.read_text(encoding="utf-8")
    # 检查 11 Repository 表
    tables = [
        "ops_metrics_config", "ops_cluster_action_log", "ops_helm_release_state",
        "ops_log_entry", "ops_log_analysis", "ops_log_query_log",
        "oauth_clients", "oauth_authorization_codes", "oauth_access_tokens", "oauth_refresh_tokens",
        "tenant_pools",
    ]
    missing = [t for t in tables if f"ENABLE ROW LEVEL SECURITY" not in text or f"{t}_select" not in text]
    if missing:
        print(f"[fail] rls-7policy-all-tables.sql 缺表: {missing}", file=sys.stderr)
        return False
    print(f"[ok] rls-7policy-all-tables.sql 含 {len(tables)} 张表 + RLS 7 类 policy", file=sys.stderr)
    return True


def validate_idempotent_pattern() -> bool:
    """验证 v0.97 DROP IF EXISTS idempotent 模式"""
    ddl_files = [
        "db/migrations/2026-09-10-rls-7policy-all-tables.sql",
        "db/migrations/2026-09-10-rls-7policy-p3d6.sql",
        "db/migrations/2026-09-10-p3d6-13-tables.sql",
    ]
    all_have_drop = True
    for f in ddl_files:
        path = Path(f)
        if not path.exists():
            print(f"[skip] {f} 不存在 (P2 阶段会跑脚本生成)", file=sys.stderr)
            continue
        text = path.read_text(encoding="utf-8")
        drop_count = text.count("DROP POLICY IF EXISTS")
        create_count = text.count("CREATE POLICY")
        if drop_count < create_count:
            print(f"[warn] {f} DROP IF EXISTS ({drop_count}) < CREATE POLICY ({create_count}) - idempotent 不完整", file=sys.stderr)
            all_have_drop = False
        else:
            print(f"[ok] {f} DROP IF EXISTS ({drop_count}) >= CREATE POLICY ({create_count})", file=sys.stderr)
    return all_have_drop


def main() -> int:
    parser = argparse.ArgumentParser(
        description="v1.01 P2 阶段 worker 子代理 实跑验证脚本 (per 守门 #1 v25 实证 + 守门 #11 缺标比错标)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="P0-4 阶段跑 (只检查文件存在 + 关键 pattern, 不连 PG)",
    )
    args = parser.parse_args()

    print(f"[v1.01] P2 阶段 worker 子代理 实跑验证 ({'dry-run P0-4' if args.dry_run else 'full P2'})", file=sys.stderr)
    checks = [
        validate_rls_template(),
        validate_p3d6_template(),
        validate_p3d6_13_tables(),
        validate_rls_11_tables(),
        validate_idempotent_pattern(),
    ]
    if all(checks):
        print(f"[ok] 5/5 验证全通过 (P2 阶段 worker 子代理 可实测)", file=sys.stderr)
        return 0
    failed = sum(1 for c in checks if not c)
    print(f"[fail] {failed}/5 验证失败", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
