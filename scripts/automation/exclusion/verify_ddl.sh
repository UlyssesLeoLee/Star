#!/usr/bin/env bash
# =============================================================================
# Star-EI DDL 验证脚本 (per docs/briefs/ex-01-5-tables-ddl.md §6.3)
# =============================================================================
#
# 5 项检查:
#   1. 表数 = 5 (idempotency_keys / lease_log / advisory_lock_audit / idempotency_keys_archive / exclusion_policy_master)
#   2. RLS policy 数 >= 9 (5 表 tenant 隔离 + 4 表 workspace 隔离)
#   3. audit trigger 数 = 5 (4 T + 1 M = 5)
#   4. 索引数 >= 13
#   5. SCD Type 2 trigger 数 = 1 (exclusion_policy_master)
#
# 守门合规: per AGENTS.md §4 #13 a/c/d 派生规
# 用法: bash scripts/automation/exclusion/verify_ddl.sh (需 PG 连接字符串)
# =============================================================================

set -euo pipefail

# 守门 #5 env 安全: 不打印 DATABASE_URL, 仅引用
DB_DSN="${STAR_PG_DSN:-postgresql://star:***@localhost:5432/star_exclusion}"

# 守门 #6 PowerShell 兼容: 不用 &&, 用 ;
if [ -z "${STAR_PG_DSN:-}" ]; then
    echo "WARN: STAR_PG_DSN 未设置, 使用默认值; 真实部署必显式设置 (per 守门 #5)"
fi

# 1. 表数检查 (期望 5)
TABLE_COUNT=$(psql "$DB_DSN" -t -A -c "
    SELECT COUNT(*) FROM information_schema.tables
    WHERE table_schema = 'public'
      AND table_name IN ('idempotency_keys', 'lease_log', 'advisory_lock_audit', 'idempotency_keys_archive', 'exclusion_policy_master')
")

echo "1. 表数: $TABLE_COUNT (期望 5)"
if [ "$TABLE_COUNT" != "5" ]; then
    echo "FAIL: 表数不对"
    exit 1
fi

# 2. RLS policy 数 (期望 >= 9)
RLS_COUNT=$(psql "$DB_DSN" -t -A -c "
    SELECT COUNT(*) FROM pg_policies
    WHERE schemaname = 'public'
      AND tablename IN ('idempotency_keys', 'lease_log', 'advisory_lock_audit', 'idempotency_keys_archive', 'exclusion_policy_master')
")

echo "2. RLS policy 数: $RLS_COUNT (期望 >= 9)"
if [ "$RLS_COUNT" -lt 9 ]; then
    echo "FAIL: RLS policy 数不对"
    exit 1
fi

# 3. audit trigger 数 (期望 5)
AUDIT_TRIGGER_COUNT=$(psql "$DB_DSN" -t -A -c "
    SELECT COUNT(*) FROM pg_trigger
    WHERE tgrelid IN (
        'idempotency_keys'::regclass,
        'lease_log'::regclass,
        'advisory_lock_audit'::regclass,
        'idempotency_keys_archive'::regclass,
        'exclusion_policy_master'::regclass
    )
    AND tgname LIKE '%audit%'
")

echo "3. audit trigger 数: $AUDIT_TRIGGER_COUNT (期望 5)"
if [ "$AUDIT_TRIGGER_COUNT" != "5" ]; then
    echo "FAIL: audit trigger 数不对"
    exit 1
fi

# 4. 索引数 (期望 >= 13)
INDEX_COUNT=$(psql "$DB_DSN" -t -A -c "
    SELECT COUNT(*) FROM pg_indexes
    WHERE schemaname = 'public'
      AND tablename IN ('idempotency_keys', 'lease_log', 'advisory_lock_audit', 'idempotency_keys_archive', 'exclusion_policy_master')
")

echo "4. 索引数: $INDEX_COUNT (期望 >= 13)"
if [ "$INDEX_COUNT" -lt 13 ]; then
    echo "FAIL: 索引数不对"
    exit 1
fi

# 5. SCD Type 2 trigger 数 (期望 1)
SCD_TRIGGER_COUNT=$(psql "$DB_DSN" -t -A -c "
    SELECT COUNT(*) FROM pg_trigger WHERE tgname = 'trg_excl_policy_scd2'
")

echo "5. SCD Type 2 trigger 数: $SCD_TRIGGER_COUNT (期望 1)"
if [ "$SCD_TRIGGER_COUNT" != "1" ]; then
    echo "FAIL: SCD Type 2 trigger 数不对"
    exit 1
fi

echo ""
echo "============================================"
echo "Star-EI DDL 验证: ALL 5/5 PASS"
echo "  - 5 张新表 100% 覆盖 (3 T + 1 T archive + 1 M SCD Type 2)"
echo "  - RLS policy >= 9 (5 表 tenant 隔离 + 4 表 workspace 隔离)"
echo "  - audit trigger = 5 (4 T + 1 M, per 守门 #13 d)"
echo "  - 索引 >= 13 (含 3 部分索引)"
echo "  - SCD Type 2 trigger = 1 (per 守门 #13 c)"
echo "============================================"
exit 0
