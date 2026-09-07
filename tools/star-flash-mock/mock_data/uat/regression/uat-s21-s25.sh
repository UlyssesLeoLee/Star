#!/usr/bin/env bash
# uat-s21-s25.sh - UAT 业务场景 21-25 (5 域 AC + 多租户 + RBAC + 审计 + 配额) regression test
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-07 14:30 JST user 发令

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scripts"
SCENARIOS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scenarios"

PYTHON_CMD="${PYTHON:-python3}"

echo "==== UAT 业务场景 21-25 (5 域 AC + 多租户 + RBAC + 审计 + 配额) regression test ===="

# 1. 跑 S21-S25 fixture generator
echo ""
echo "--- 跑 S21-S25 _generate_uat_s21_s25.py ---"
"$PYTHON_CMD" "$SCRIPTS_DIR/_generate_uat_s21_s25.py"

# 2. 验证 fixture 完整性
echo ""
echo "--- 验证 S21-S25 fixture 完整性 (5 场景 × 5 文件 = 25) ---"
for s in S21-5d-ac-acceptance S22-multitenant-isolation S23-rbac-scheme S24-audit-worm S25-quota-exceeded; do
    count=$(find "$SCENARIOS_DIR/$s" -type f 2>/dev/null | wc -l)
    if [ "$count" -eq 5 ]; then
        echo "  [OK] $s: 5 files"
    else
        echo "  [FAIL] $s: $count files (expected 5)"
        exit 1
    fi
done

# 3. 5 域 Lead CONTENT 4 维守门 (per 守门 #14 v2)
echo ""
echo "--- 5 域 Lead CONTENT 4 维守门 (per 守门 #14 v2) ---"
if grep -q 'raci_4_dim' "$SCENARIOS_DIR/S21-5d-ac-acceptance/expected_response.json"; then
    echo "  [OK] S21: 5 域 AC 跨引 + raci_4_dim"
else
    echo "  [FAIL] S21: expected raci_4_dim"
    exit 1
fi
# 4 维守门
for dim in decision_scope raci timeline mavis_sign_boundary; do
    if ! grep -q "\"$dim\"" "$SCENARIOS_DIR/S21-5d-ac-acceptance/expected_response.json"; then
        echo "  [FAIL] S21: raci_4_dim 缺 $dim"
        exit 1
    fi
done
echo "  [OK] S21: raci_4_dim 全 4 维 (decision_scope / raci / timeline / mavis_sign_boundary)"

# 4. 守门 #13 W/T/M 守门
echo ""
echo "--- 守门 #13 W/T/M 守门 ---"
# S21 = Transaction (audit 5 域)
# S22 = Transaction (tenant isolation)
# S23 = Master (RBAC SCD Type 2)
# S24 = Transaction (audit WORM)
# S25 = Work (quota short TTL)
declare -A WTM
WTM["S21-5d-ac-acceptance"]="Transaction"
WTM["S22-multitenant-isolation"]="Transaction"
WTM["S24-audit-worm"]="Transaction"
WTM["S23-rbac-scheme"]="Master"
WTM["S25-quota-exceeded"]="Work"

for s in "${!WTM[@]}"; do
    cls="${WTM[$s]}"
    if grep -q "\"class\": \"$cls\"" "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: $cls"
    else
        echo "  [FAIL] $s: expected $cls"
        exit 1
    fi
done

# 5. WORM 守门 (per ADR-0043 audit.onboarding.failed)
echo ""
echo "--- WORM 守门 (per ADR-0043) ---"
if grep -q 'worm_locked.*true' "$SCENARIOS_DIR/S24-audit-worm/expected_response.json"; then
    echo "  [OK] S24: WORM locked"
else
    echo "  [FAIL] S24: expected worm_locked=true"
    exit 1
fi

echo ""
echo "==== UAT 业务场景 21-25 regression test PASSED ===="
