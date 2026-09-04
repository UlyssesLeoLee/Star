#!/usr/bin/env bash
# scripts/run-all.sh - 一键跑全部 star-flash-mock 回归脚本
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST
# 守门: #1+#9+#12+#13 (commit-time 守门 + 子代理 status 实证 + docs 同步)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
REPORT_DIR="$REPO_ROOT/tools/star-flash-mock/docs"

echo "=========================================="
echo "Star Mock Project - run-all.sh"
echo "  Started at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=========================================="
echo ""

# 跑全部脚本
results=()
scripts=("smoke-test.sh" "regression-test-langgraph.sh" "regression-test-agent-runtime.sh" "regression-test-mcp.sh" "regression-test-streamable-http.sh" "regression-test-db-wtm.sh" "regression-test-five-domain.sh" "regression-test-openclaw.sh")

for script in "${scripts[@]}"; do
    echo ""
    echo "==== 跑 $script ===="
    if bash "$SCRIPTS_DIR/$script" 2>&1; then
        results+=("[PASS] $script")
    else
        results+=("[FAIL] $script")
    fi
done

# 收尾 - 守门 #9 子代理 status="succeeded" 实证 (git log 实证)
echo ""
echo "==== 守门 #9 实证 (git log --follow mock project) ===="
cd "$REPO_ROOT"
git log --oneline -5 -- tools/star-flash-mock/ 2>/dev/null || echo "[INFO] 首次提交, 暂无历史"

# 收尾 - 写回归报告
echo ""
echo "==== 写 docs/regression-report-YYYY-MM-DD.md ===="
report_file="$REPORT_DIR/regression-report-$(date -u +%Y-%m-%d).md"
{
    echo "# Star Mock Project 回归测试报告"
    echo ""
    echo "> **生成时间**: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "> **范围**: tools/star-flash-mock/{scripts/,mock_data/,docs/,k3s/}"
    echo "> **触发**: 2026-09-05 06:50 JST user 拍板 (全栈覆盖 v0.7)"
    echo "> **守门**: 守门 #1+#9+#12+#13"
    echo ""
    echo "## 1. 跑脚本结果"
    echo ""
    echo "| # | 脚本 | 状态 |"
    echo "|---|---|---|"
    for r in "${results[@]}"; do
        echo "| - | \`$r\` | - |"
    done
    echo ""
    echo "## 2. mock_data fixture 统计"
    echo ""
    for d in openclaw langgraph/tmo langgraph/sa-10 langgraph/sa-01..09 agent-runtime/l0-dispatcher agent-runtime/l1-ecs agent-runtime/l2-pools mcp streamable-http db-wtm/work db-wtm/transaction db-wtm/master; do
        cnt=$(find "$REPO_ROOT/tools/star-flash-mock/mock_data/$d" -name "*.json" 2>/dev/null | wc -l)
        echo "- \`$d/\`: $cnt 份 fixture"
    done
    echo ""
    echo "## 3. 已知缺口 (per 守门 #11 缺标比错标)"
    echo ""
    echo "- 缺口 #1: 16 MCP tool 仅覆盖 6, 缺 10 (audit_event / scm / workspace / feedback / inbox / project / permission / kms / form / search)"
    echo "- 缺口 #2: Agent Runtime L1 ECS 9 Archetype 仅 2 fixture (SA-01 + lifecycle)"
    echo "- 缺口 #3: Streamable HTTP 5xx 错误 + retry 完整 case 缺失"
    echo "- 缺口 #4: 5 域 fixture 仅 0-1 份 (per README #4)"
    echo "- 缺口 #5: DB W/T/M 仅 12 fixture (4 W + 4 T + 4 M), 缺 RLS 13 類必携完整对账"
    echo "- 缺口 #6: k3s/ 仅 2 yaml, 缺 star-mock ConfigMap + Secret"
    echo "- 缺口 #7: docs/ 回归报告 (本文件) 仅初次生成"
    echo ""
} > "$report_file"
echo "  [OK] $report_file"

echo ""
echo "=========================================="
echo "  run-all.sh completed at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "=========================================="
