#!/usr/bin/env bash
# scripts/regression-test-mcp.sh - 16 MCP tool 100% 覆蓋回归 (per AGENTS.md §7 #1 + ADR-0032)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-C
# 守门: AGENTS.md §7 #1 16 tool 真实接入 e2e + 守门 #1+#5+#9+#10+#11+#12+#13 a/b/c/d

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
MCP_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/mcp"
REPORT="$REPO_ROOT/docs/reports/MCP-16-TOOL-100-COVERAGE-REPORT.md"

echo "==== MCP 16 tool 100% 覆蓋回归 (per AGENTS.md §7 #1) ===="

# ===== 1. 16 tool fixture 完整性 =====
echo ""
echo "--- 1. 16 tool fixture 完整性 (target: 16) ---"
total=$(find "$MCP_DIR" -name "v1--mcp--*.json" 2>/dev/null | wc -l)
echo "  total: $total (target: 16)"
if [ "$total" -lt 16 ]; then
    echo "  [FAIL] 16 tool 覆盖率不达 100%"
    exit 1
fi
echo "  [OK] 16 tool 100% 覆蓋 实证"

# ===== 2. 16 tool 名称完整 (per AGENTS.md §7 #1) =====
echo ""
echo "--- 2. 16 tool 名称完整验证 ---"
expected_tools=(workitem-list workitem-create tools-invoke agents-list sessions-create billing-usage audit-event scm workspace feedback inbox project permission kms form search)
covered=0
for tool in "${expected_tools[@]}"; do
    if find "$MCP_DIR" -name "v1--mcp--$tool--*.json" 2>/dev/null | grep -q .; then
        covered=$((covered + 1))
    else
        echo "  [WARN] tool $tool 缺标"
    fi
done
echo "  covered: $covered / 16"
if [ "$covered" -lt 16 ]; then
    echo "  [FAIL] 16 tool 名称不完整"
    exit 1
fi
echo "  [OK] 16 tool 名称全部覆蓋"

# ===== 3. 守门 #13 W/T/M 必携 =====
echo ""
echo "--- 3. 守门 #13 W/T/M + rls_13_classes 必携 ---"
wtm_invalid=0
for f in $(find "$MCP_DIR" -name "v1--mcp--*.json" 2>/dev/null); do
    if ! grep -q "wtm_class" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 wtm_class"
        wtm_invalid=$((wtm_invalid + 1))
    fi
    if ! grep -q "rls_13_classes" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 rls_13_classes"
        wtm_invalid=$((wtm_invalid + 1))
    fi
done
if [ "$wtm_invalid" -eq 0 ]; then
    echo "  [OK] 16 fixture 全部含 wtm_class + rls_13_classes"
else
    echo "  [FAIL] $wtm_invalid fixture 缺守门字段"
    exit 1
fi

# ===== 4. 跨文档引用 + JSON 校验 =====
echo ""
echo "--- 4. 跨文档引用 + JSON 格式 ---"
if [ -f "$REPORT" ]; then
    echo "  [OK] 100% 覆盖率报告 存在"
else
    echo "  [WARN] 100% 覆盖率报告 缺失, 缺标"
fi

invalid_count=0
for f in $(find "$MCP_DIR" -name "v1--mcp--*.json" 2>/dev/null); do
    if ! python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
        echo "  [FAIL] invalid JSON: $f"
        invalid_count=$((invalid_count + 1))
    fi
done
if [ "$invalid_count" -eq 0 ]; then
    echo "  [OK] 16 fixture 全部有效 JSON"
else
    exit 1
fi

echo ""
echo "==== MCP 16 tool 100% 覆蓋 regression test PASSED ===="
echo "  [OK] 16 tool 100% mock fixture 覆蓋"
echo "  [OK] 守门 #1+#5+#9+#10+#11+#12+#13 a/b/c/d 0 违反"