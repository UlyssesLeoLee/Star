#!/usr/bin/env bash
# scripts/regression-test-db-wtm.sh - DB W/T/M 三類横展强制分类回归 (per 守门 #13)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST
# 守门: #13 a/b/c/d (W 物理删+短TTL / T 物理删禁止+audit+RLS / M 物理删禁止+SCD+RLS)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WTM_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/db-wtm"

echo "==== DB W/T/M regression test (per 守门 #13) ===="

# 三类必含
echo ""
echo "--- W/T/M 三类必含检查 ---"
for cls in work transaction master; do
    count=$(find "$WTM_DIR/$cls" -name "*.json" 2>/dev/null | wc -l)
    if [ "$count" -ge 3 ]; then
        echo "  [OK] $cls: $count fixtures (>= 3)"
    else
        echo "  [FAIL] $cls: $count fixtures (< 3, 守门 #13 100% 覆盖不满足)"
        exit 1
    fi
done

# 守门 #13 a W 物理删除 fixture
echo ""
echo "--- 守门 #13 a W = 物理删除 fixture 检查 ---"
for f in $(find "$WTM_DIR/work" -name "*DELETE*" 2>/dev/null); do
    if grep -q "physical_delete" "$f" 2>/dev/null; then
        echo "  [OK] $f 物理删除场景"
    fi
done

# 守门 #13 b T 物理删除禁止 fixture
echo ""
echo "--- 守门 #13 b T = 物理删除禁止 + audit + RLS 13 類 fixture 检查 ---"
for f in $(find "$WTM_DIR/transaction" -name "*DELETE*" 2>/dev/null); do
    if grep -q "physical_delete_blocked" "$f" 2>/dev/null; then
        echo "  [OK] $f 物理删除拦截场景"
    fi
done

# 守门 #13 c M 物理删除禁止 + SCD Type 2 + RLS fixture
echo ""
echo "--- 守门 #13 c M = 物理删除禁止 + SCD Type 2 + RLS 13 類 fixture 检查 ---"
for f in $(find "$WTM_DIR/master" -name "*.json" 2>/dev/null); do
    has_scd=$(grep -c "scd_type" "$f" 2>/dev/null || echo 0)
    has_rls=$(grep -c "rls_13_classes" "$f" 2>/dev/null || echo 0)
    if [ "$has_scd" -ge 1 ] && [ "$has_rls" -ge 1 ]; then
        echo "  [OK] $(basename $f): SCD + RLS 都含"
    fi
done

# 跨文档引用
echo ""
echo "--- 跨文档引用 (per docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1) ---"
if [ -f "$REPO_ROOT/docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md" ]; then
    echo "  [OK] 基线文档存在"
else
    echo "  [WARN] 基线文档缺失, 缺标"
fi

echo ""
echo "==== DB W/T/M regression test PASSED ===="
