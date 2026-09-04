#!/usr/bin/env bash
# scripts/regression-test-openclaw.sh - OpenClaw v1 既有 fixture 回归
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST (迁移自 docs/reports/wiremock-openclaw/ → tools/star-flash-mock/mock_data/openclaw/)
# 守门: #12 commit-time docs 同步

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OC_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/openclaw"

echo "==== OpenClaw v1 regression test ===="

# 20 份 fixture 检查 (5 资源 × 4 method)
echo ""
echo "--- OpenClaw v1 20 份 fixture 检查 ---"
count=$(find "$OC_DIR" -name "*.json" 2>/dev/null | wc -l)
echo "  total: $count"
if [ "$count" -lt 20 ]; then
    echo "  [FAIL] OpenClaw v1 缺标 (< 20 份)"
    exit 1
fi

# 5 资源覆盖
echo ""
echo "--- 5 资源 (agents/cost/messages/sessions/tools-invoke) 覆盖检查 ---"
for resource in agents cost messages sessions tools-invoke; do
    count=$(find "$OC_DIR" -name "*$resource*" 2>/dev/null | wc -l)
    if [ "$count" -ge 4 ]; then
        echo "  [OK] $resource: $count fixtures (4 method × 1 resource)"
    else
        echo "  [WARN] $resource: $count fixtures"
    fi
done

echo ""
echo "==== OpenClaw v1 regression test PASSED ===="
