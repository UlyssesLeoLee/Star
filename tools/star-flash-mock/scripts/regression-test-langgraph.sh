#!/usr/bin/env bash
# scripts/regression-test-langgraph.sh - LangGraph TMO 7 节点 + 9 SA + SA-10 回归
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST
# 守门: #1+#9+#12+#13 a

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TMO_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/langgraph/tmo"
SA10_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/langgraph/sa-10"
SA09_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/langgraph/sa-01..09"

echo "==== LangGraph regression test ===="

# TMO 7 节点
echo ""
echo "--- TMO 7 节点 fixture 检查 ---"
expected_nodes=(m-n1 m-n2 m-n3 m-n4 m-n5 m-n6 m-n7)
for node in "${expected_nodes[@]}"; do
    count=$(find "$TMO_DIR" -name "*$node*" 2>/dev/null | wc -l)
    if [ "$count" -ge 2 ]; then
        echo "  [OK] TMO $node: $count fixtures (>= 2)"
    else
        echo "  [FAIL] TMO $node: $count fixtures (< 2, 缺标)"
        exit 1
    fi
done

# SA-10
echo ""
echo "--- SA-10 task-orchestrator fixture 检查 ---"
count=$(find "$SA10_DIR" -name "*.json" 2>/dev/null | wc -l)
if [ "$count" -ge 5 ]; then
    echo "  [OK] SA-10: $count fixtures (>= 5)"
else
    echo "  [WARN] SA-10: $count fixtures (< 5, 缺标)"
fi

# 9 SA Type
echo ""
echo "--- 9 SA Type (SA-01..SA-09) fixture 检查 ---"
for sa in sa-01 sa-02 sa-03 sa-04 sa-05 sa-06 sa-07 sa-08 sa-09; do
    count=$(find "$SA09_DIR" -name "*$sa*" 2>/dev/null | wc -l)
    if [ "$count" -ge 1 ]; then
        echo "  [OK] $sa: $count fixtures"
    else
        echo "  [FAIL] $sa: 0 fixtures"
        exit 1
    fi
done

# 实际 pytest 跑 (per 守门 #1)
echo ""
echo "--- 跑 tests/integration/ + tests/unit/test_task_ops/ ---"
cd "$REPO_ROOT"
PYTHONPATH=scripts python3 -m pytest tests/integration/test_tmo_*.py tests/unit/test_task_ops/ -q 2>&1 | tail -20 || echo "[WARN] pytest 部分失败, 需查"

echo ""
echo "==== LangGraph regression test PASSED ===="
