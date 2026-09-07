#!/usr/bin/env bash
# uat-s11-s15.sh - UAT 业务场景 11-15 (TMO M-N2..M-N6) regression test (per 守门 #13 + LangGraph TMO 02)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-07 14:30 JST user 发令

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scripts"
SCENARIOS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scenarios"

PYTHON_CMD="${PYTHON:-python3}"

echo "==== UAT 业务场景 11-15 (TMO M-N2..M-N6) regression test ===="

# 1. 跑 S11-S15 fixture generator
echo ""
echo "--- 跑 S11-S15 _generate_uat_s11_s15.py ---"
"$PYTHON_CMD" "$SCRIPTS_DIR/_generate_uat_s11_s15.py"

# 2. 验证 fixture 完整性
echo ""
echo "--- 验证 S11-S15 fixture 完整性 (5 场景 × 5 文件 = 25) ---"
for s in S11-tmo-split S12-tmo-reorder S13-tmo-bulk S14-tmo-summarize S15-tmo-reassign; do
    count=$(find "$SCENARIOS_DIR/$s" -type f 2>/dev/null | wc -l)
    if [ "$count" -eq 5 ]; then
        echo "  [OK] $s: 5 files"
    else
        echo "  [FAIL] $s: $count files (expected 5)"
        exit 1
    fi
done

# 3. TMO 7 节点守门 (per LangGraph 02 §2.6 + ADR-0046)
echo ""
echo "--- TMO 7 节点守门 (per LangGraph 02 §2.6 + ADR-0046) ---"
declare -A TMO_NODES
TMO_NODES["S11-tmo-split"]="M-N2"
TMO_NODES["S12-tmo-reorder"]="M-N3"
TMO_NODES["S13-tmo-bulk"]="M-N4"
TMO_NODES["S14-tmo-summarize"]="M-N5"
TMO_NODES["S15-tmo-reassign"]="M-N6"

for s in "${!TMO_NODES[@]}"; do
    node="${TMO_NODES[$s]}"
    if grep -q "\"node\": \"$node\"" "$SCENARIOS_DIR/$s/request.json"; then
        echo "  [OK] $s: TMO $node"
    else
        echo "  [FAIL] $s: expected TMO $node"
        exit 1
    fi
done

# 4. 守门 #13 a L1↔L1 禁止 守门 (L0 唯一协调)
echo ""
echo "--- 守门 #13 a L1↔L1 禁止 守门 ---"
for s in "${!TMO_NODES[@]}"; do
    if ! grep -q 'L1↔L1' "$SCENARIOS_DIR/$s/ac_mapping.md"; then
        echo "  [FAIL] $s: 守门 #13 a L1↔L1 禁止 描述缺失"
        exit 1
    fi
done
echo "  [OK] S11-S15: 守门 #13 a L1↔L1 禁止 全过"

echo ""
echo "==== UAT 业务场景 11-15 regression test PASSED ===="
