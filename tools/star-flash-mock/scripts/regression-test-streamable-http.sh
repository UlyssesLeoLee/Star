#!/usr/bin/env bash
# scripts/regression-test-streamable-http.sh - Streamable HTTP spec 完整实现回归
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST (v1.0) + ULYS-140 2026-09-20 JST (v1.2 加 5xx + retry fixture)
# 守门: #1+#3+#12 (per AGENTS.md §7 #3 D.5+ + D.7+)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SH_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/streamable-http"
PYTHON_CMD="${PYTHON:-python3}"

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

# v1.2 (per ULYS-140 缺口 #3): 5xx 错误 + retry-with-backoff fixture 验证
echo ""
echo "--- v1.2 5xx 错误 + retry-with-backoff fixture (per 缺口 #3) ---"

# 5xx 内部错误 fixture
if [ -f "$SH_DIR/v1--streamable--server-push-sse--500-internal-error.json" ]; then
    echo "  [OK] 5xx-internal-error fixture 存在"
    if grep -q '"status": 500' "$SH_DIR/v1--streamable--server-push-sse--500-internal-error.json"; then
        echo "  [OK] 含 500 状态码"
    else
        echo "  [FAIL] 缺 500 状态码"
        exit 1
    fi
    if grep -q 'Retry-After' "$SH_DIR/v1--streamable--server-push-sse--500-internal-error.json"; then
        echo "  [OK] 含 Retry-After 头 (契约必携 per 守门 #1)"
    else
        echo "  [FAIL] 缺 Retry-After 头"
        exit 1
    fi
    if grep -q 'retry_strategy' "$SH_DIR/v1--streamable--server-push-sse--500-internal-error.json"; then
        echo "  [OK] 含 retry_strategy (exponential backoff)"
    else
        echo "  [FAIL] 缺 retry_strategy"
        exit 1
    fi
else
    echo "  [FAIL] 5xx-internal-error fixture 缺失"
    exit 1
fi

# retry-with-backoff-after-503 fixture
if [ -f "$SH_DIR/v1--streamable--retry-with-backoff-after-503.json" ]; then
    echo "  [OK] retry-with-backoff-after-503 fixture 存在"
    # 验证 request_sequence 3 步
    seq_count=$(grep -c '"step":' "$SH_DIR/v1--streamable--retry-with-backoff-after-503.json" || true)
    if [ "$seq_count" -eq 3 ]; then
        echo "  [OK] request_sequence 3 步 (503 → 503 → 200)"
    else
        echo "  [FAIL] request_sequence 步数异常 (期望 3, 实测 $seq_count)"
        exit 1
    fi
    if grep -q 'X-Retry-Attempt' "$SH_DIR/v1--streamable--retry-with-backoff-after-503.json"; then
        echo "  [OK] 含 X-Retry-Attempt header (retry 计数器契约)"
    else
        echo "  [FAIL] 缺 X-Retry-Attempt header"
        exit 1
    fi
    if grep -q 'idempotency' "$SH_DIR/v1--streamable--retry-with-backoff-after-503.json"; then
        echo "  [OK] 含 idempotency 段 (Last-Event-Id 去重契约)"
    else
        echo "  [FAIL] 缺 idempotency 段"
        exit 1
    fi
else
    echo "  [FAIL] retry-with-backoff-after-503 fixture 缺失"
    exit 1
fi

# 总 fixture 统计 (≥ 6, 旧 4 + 新 2)
total=$(find "$SH_DIR" -name "*.json" 2>/dev/null | wc -l)
echo ""
echo "--- Streamable HTTP 总 fixture 统计 ---"
echo "  total: $total fixtures (v1.2 ≥ 6)"
if [ "$total" -lt 6 ]; then
    echo "  [FAIL] 总 fixture < 6"
    exit 1
fi

echo ""
echo "==== Streamable HTTP regression test PASSED ===="