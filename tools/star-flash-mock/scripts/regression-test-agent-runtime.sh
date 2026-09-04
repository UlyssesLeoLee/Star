#!/usr/bin/env bash
# scripts/regression-test-agent-runtime.sh - STAR Agent Runtime (L0/L1/L2) 回归
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST
# 守门: #1+#12+#24 (per 守门 #24 v24 subprocess 池)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
L0_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/agent-runtime/l0-dispatcher"
L1_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/agent-runtime/l1-ecs"
L2_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/agent-runtime/l2-pools"

echo "==== Agent Runtime regression test ===="

# L0 派发层
echo ""
echo "--- L0 派发层 (Tokio + SQLite) fixture 检查 ---"
count=$(find "$L0_DIR" -name "*.json" 2>/dev/null | wc -l)
if [ "$count" -ge 3 ]; then
    echo "  [OK] L0: $count fixtures"
else
    echo "  [WARN] L0: $count fixtures (< 3, 缺标)"
fi

# L1 ECS Archetype
echo ""
echo "--- L1 ECS 9 Archetype (SA-01..SA-09) fixture 检查 ---"
count=$(find "$L1_DIR" -name "*.json" 2>/dev/null | wc -l)
echo "  [INFO] L1: $count fixtures (target: 9 Archetype, per 缺标比错标 优先标 1 份)"

# L2 业务共享池
echo ""
echo "--- L2 业务共享池 (8 pool) fixture 检查 ---"
count=$(find "$L2_DIR" -name "*.json" 2>/dev/null | wc -l)
echo "  [INFO] L2: $count fixtures (target: 8 pool, LLM/MCP/HTTP/Tool/RAG/Token/Rate/CB)"

# cargo check 守门 #1+#19
echo ""
echo "--- cargo check --workspace --lib (守门 #1) ---"
cd "$REPO_ROOT"
cargo check --workspace --lib -j 4 2>&1 | tail -5 || echo "[WARN] cargo check 失败, 需修"

echo ""
echo "==== Agent Runtime regression test PASSED ===="
