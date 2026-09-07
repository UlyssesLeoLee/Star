#!/usr/bin/env bash
# uat-s06-s10.sh - UAT 业务场景 6-10 (S06-S10) regression test (per 守门 #13 W/T/M + #19)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-07 14:30 JST user 发令

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scripts"
SCENARIOS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scenarios"

PYTHON_CMD="${PYTHON:-python3}"

echo "==== UAT 业务场景 6-10 (S06-S10) regression test ===="

# 1. 跑 S06-S10 fixture generator
echo ""
echo "--- 跑 S06-S10 _generate_uat_s06_s10.py ---"
"$PYTHON_CMD" "$SCRIPTS_DIR/_generate_uat_s06_s10.py"

# 2. 验证 fixture 完整性
echo ""
echo "--- 验证 S06-S10 fixture 完整性 (5 场景 × 5 文件 = 25) ---"
for s in S06-validation-fail S07-conflict-detect S08-rebase-merge S09-merge-request S10-tmo-merge; do
    count=$(find "$SCENARIOS_DIR/$s" -type f 2>/dev/null | wc -l)
    if [ "$count" -eq 5 ]; then
        echo "  [OK] $s: 5 files"
    else
        echo "  [FAIL] $s: $count files (expected 5)"
        exit 1
    fi
done

# 3. W/T/M + TMO 守门
echo ""
echo "--- W/T/M + TMO 守门 (per 守门 #13 + LangGraph TMO 02 §2.6) ---"
# S06 = Work (validation fail short TTL)
# S07 = Work (conflict detect 1h)
# S08 = Transaction (rebase + merge audit)
# S09 = Transaction (merge request append-only)
# S10 = Transaction (TMO M-N1 stash_append_only)
for s in S06-validation-fail S07-conflict-detect; do
    if grep -q '"class": "Work"' "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: Work (短 TTL)"
    else
        echo "  [FAIL] $s: expected Work"
        exit 1
    fi
done
for s in S08-rebase-merge S09-merge-request S10-tmo-merge; do
    if grep -q '"class": "Transaction"' "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: Transaction (append-only)"
    else
        echo "  [FAIL] $s: expected Transaction"
        exit 1
    fi
done
# TMO M-N1 stash_append_only 守门 (per 守门 #13 d)
if grep -q 'stash_append_only.*true' "$SCENARIOS_DIR/S10-tmo-merge/expected_response.json"; then
    echo "  [OK] S10: TMO M-N1 stash_append_only=true"
else
    echo "  [FAIL] S10: TMO M-N1 stash_append_only missing"
    exit 1
fi

echo ""
echo "==== UAT 业务场景 6-10 regression test PASSED ===="
