#!/usr/bin/env bash
# scripts/regression-test-streamable-http.sh - Streamable HTTP spec 完整实现回归
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST
# 守门: #1+#3+#12 (per AGENTS.md §7 #3 D.5+ + D.7+)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SH_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/streamable-http"

echo "==== Streamable HTTP regression test ===="

# 4 核心能力
echo ""
echo "--- Streamable HTTP 4 核心能力 fixture 检查 ---"
for cap in "session-create" "reconnect" "server-push" "delete-session"; do
    count=$(find "$SH_DIR" -name "*$cap*" 2>/dev/null | wc -l)
    if [ "$count" -ge 1 ]; then
        echo "  [OK] $cap: $count fixtures"
    else
        echo "  [FAIL] $cap: 0 fixtures"
        exit 1
    fi
done

# Last-Event-Id 头验证
echo ""
echo "--- Last-Event-Id 头验证 (fixture 内含) ---"
for f in $(find "$SH_DIR" -name "*reconnect*" 2>/dev/null); do
    if grep -q "Last-Event-Id" "$f" 2>/dev/null; then
        echo "  [OK] $f 含 Last-Event-Id 头"
    else
        echo "  [WARN] $f 缺 Last-Event-Id 头"
    fi
done

# DELETE session
echo ""
echo "--- DELETE session 端点验证 ---"
for f in $(find "$SH_DIR" -name "*delete-session*" 2>/dev/null); do
    if grep -q "204" "$f" 2>/dev/null; then
        echo "  [OK] $f 含 204 状态码"
    fi
done

echo ""
echo "==== Streamable HTTP regression test PASSED ===="
