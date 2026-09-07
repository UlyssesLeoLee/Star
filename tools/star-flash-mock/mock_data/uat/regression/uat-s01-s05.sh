#!/usr/bin/env bash
# uat-s01-s05.sh - UAT 业务场景 1-5 regression test (per 守门 #13 W/T/M + #19 Python 化)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-07 14:30 JST user 发令

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scripts"
SCENARIOS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scenarios"

# Use py launcher (Windows-compatible, falls back to python3)
PYTHON_CMD="${PYTHON:-python3}"

echo "==== UAT 业务场景 1-5 (S01-S05) regression test ===="

# 1. 跑 S01-S05 fixture generator
echo ""
echo "--- 跑 S01-S05 _generate_uat_s01_s05.py ---"
"$PYTHON_CMD" "$SCRIPTS_DIR/_generate_uat_s01_s05.py"

# 2. 验证 fixture 完整性
echo ""
echo "--- 验证 S01-S05 fixture 完整性 (5 场景 × 5 文件 = 25) ---"
for s in S01-workitem-create S02-worktree-create S03-agent-running S04-feedback-loop S05-validation-pass; do
    count=$(find "$SCENARIOS_DIR/$s" -type f 2>/dev/null | wc -l)
    if [ "$count" -eq 5 ]; then
        echo "  [OK] $s: 5 files"
    else
        echo "  [FAIL] $s: $count files (expected 5)"
        exit 1
    fi
done

# 3. W/T/M 分类守门 (per 守门 #13)
echo ""
echo "--- W/T/M 分类守门 (per 守门 #13) ---"
# S01 = Transaction (workitem created)
# S02 = Master (worktree SCD Type 2)
# S03 = Transaction (agent session)
# S04 = Transaction (feedback cross-domain)
# S05 = Work (validation short TTL)
for s in S01-workitem-create S03-agent-running S04-feedback-loop; do
    if grep -q '"class": "Transaction"' "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: Transaction (append-only)"
    else
        echo "  [FAIL] $s: expected Transaction"
        exit 1
    fi
done
for s in S02-worktree-create; do
    if grep -q '"class": "Master"' "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: Master (SCD Type 2)"
    else
        echo "  [FAIL] $s: expected Master"
        exit 1
    fi
done
for s in S05-validation-pass; do
    if grep -q '"class": "Work"' "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: Work (短 TTL)"
    else
        echo "  [FAIL] $s: expected Work"
        exit 1
    fi
done

echo ""
echo "==== UAT 业务场景 1-5 regression test PASSED ===="
