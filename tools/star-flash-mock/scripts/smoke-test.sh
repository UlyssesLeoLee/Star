#!/usr/bin/env bash
# scripts/smoke-test.sh - Star Mock Project smoke test (per 守门 #12 commit-time + 守门 #1 集成)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 06:50 JST user 拍板 (全栈覆盖 v0.7)
# 守门 (per AGENTS.md §4):
#   - 守门 #1: 集成就绪 (mock_data 完整 + scripts 可执行)
#   - 守门 #5: 无 secret 泄露 (fixture 不含 env 凭据)
#   - 守门 #11: 缺标比错标 (smoke 失败不掩盖, 显式列缺)
#   - 守门 #12: commit-time docs 同步 (本脚本落档 + regression-report)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
MOCK_ROOT="$REPO_ROOT/tools/star-flash-mock"
MOCK_DATA="$MOCK_ROOT/mock_data"

echo "==== Star Mock Project smoke test ===="
echo "REPO_ROOT: $REPO_ROOT"
echo "MOCK_ROOT: $MOCK_ROOT"
echo ""

# ===== 1. 目录结构检查 =====
echo "--- 1. 目录结构检查 ---"
for dir in scripts mock_data docs k3s mock_data/openclaw mock_data/langgraph mock_data/agent-runtime mock_data/mcp mock_data/streamable-http mock_data/db-wtm; do
    if [ -d "$MOCK_ROOT/$dir" ]; then
        echo "  [OK] $dir exists"
    else
        echo "  [FAIL] $dir missing"
        exit 1
    fi
done

# ===== 2. mock_data fixture 数量统计 =====
echo ""
echo "--- 2. mock_data fixture 统计 ---"
total=0
for f in $(find "$MOCK_DATA" -name "*.json" 2>/dev/null); do
    total=$((total + 1))
done
echo "  total fixtures: $total"
if [ "$total" -lt 50 ]; then
    echo "  [WARN] fixture count < 50, 缺标"
fi

# ===== 3. 守门 #5 无 secret 泄露 =====
echo ""
echo "--- 3. 守门 #5 无 secret 泄露 ---"
forbidden_patterns=("password=" "api_key=" "secret=" "token=" "credentials" "BEGIN PRIVATE KEY" "GHCR_PAT")
leak_count=0
for pattern in "${forbidden_patterns[@]}"; do
    matches=$(grep -r -l -i --include="*.json" "$pattern" "$MOCK_DATA" 2>/dev/null || true)
    if [ -n "$matches" ]; then
        echo "  [FAIL] forbidden pattern '$pattern' found in: $matches"
        leak_count=$((leak_count + 1))
    fi
done
if [ "$leak_count" -eq 0 ]; then
    echo "  [OK] no secret leak in fixtures"
else
    echo "  [FAIL] $leak_count secret leaks"
    exit 1
fi

# ===== 4. fixture JSON 格式校验 =====
echo ""
echo "--- 4. fixture JSON 格式校验 ---"
invalid_count=0
for f in $(find "$MOCK_DATA" -name "*.json" 2>/dev/null); do
    if ! python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
        echo "  [FAIL] invalid JSON: $f"
        invalid_count=$((invalid_count + 1))
    fi
done
if [ "$invalid_count" -eq 0 ]; then
    echo "  [OK] all fixtures valid JSON"
else
    echo "  [FAIL] $invalid_count invalid JSON fixtures"
    exit 1
fi

# ===== 5. 守门 #13 DB W/T/M 三类覆盖 =====
echo ""
echo "--- 5. 守门 #13 DB W/T/M 三类覆盖 ---"
for cls in work transaction master; do
    count=$(find "$MOCK_DATA/db-wtm/$cls" -name "*.json" 2>/dev/null | wc -l)
    if [ "$count" -ge 3 ]; then
        echo "  [OK] db-wtm/$cls: $count fixtures (>= 3)"
    else
        echo "  [WARN] db-wtm/$cls: $count fixtures (< 3, 缺标)"
    fi
done

echo ""
echo "==== smoke-test PASSED ===="
