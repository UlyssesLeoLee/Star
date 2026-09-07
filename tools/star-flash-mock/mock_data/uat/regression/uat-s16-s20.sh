#!/usr/bin/env bash
# uat-s16-s20.sh - UAT 业务场景 16-20 (TMO M-N7 + Streamable HTTP 4) regression test
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-07 14:30 JST user 发令

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scripts"
SCENARIOS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scenarios"

PYTHON_CMD="${PYTHON:-python3}"

echo "==== UAT 业务场景 16-20 (TMO M-N7 + Streamable HTTP) regression test ===="

# 1. 跑 S16-S20 fixture generator
echo ""
echo "--- 跑 S16-S20 _generate_uat_s16_s20.py ---"
"$PYTHON_CMD" "$SCRIPTS_DIR/_generate_uat_s16_s20.py"

# 2. 验证 fixture 完整性
echo ""
echo "--- 验证 S16-S20 fixture 完整性 (5 场景 × 5 文件 = 25) ---"
for s in S16-tmo-metadata S17-streamable-connect S18-streamable-push S19-streamable-reconnect S20-streamable-delete; do
    count=$(find "$SCENARIOS_DIR/$s" -type f 2>/dev/null | wc -l)
    if [ "$count" -eq 5 ]; then
        echo "  [OK] $s: 5 files"
    else
        echo "  [FAIL] $s: $count files (expected 5)"
        exit 1
    fi
done

# 3. TMO M-N7 + Streamable HTTP 4 守门
echo ""
echo "--- TMO M-N7 + Streamable HTTP 4 守门 ---"
# S16 = TMO M-N7 metadata (Master SCD Type 2)
if grep -q '"node": "M-N7"' "$SCENARIOS_DIR/S16-tmo-metadata/request.json"; then
    echo "  [OK] S16: TMO M-N7 metadata"
else
    echo "  [FAIL] S16: expected TMO M-N7"
    exit 1
fi
# S17-S20 = Streamable HTTP 4 场景
for s in S17-streamable-connect S18-streamable-push S19-streamable-reconnect S20-streamable-delete; do
    if grep -q '/api/mcp/streamable' "$SCENARIOS_DIR/$s/request.json"; then
        echo "  [OK] $s: Streamable HTTP endpoint"
    else
        echo "  [FAIL] $s: expected Streamable HTTP endpoint"
        exit 1
    fi
done

# 4. 守门 #13 c Master SCD 守门
echo ""
echo "--- 守门 #13 c Master SCD 守门 ---"
if grep -q '"class": "Master"' "$SCENARIOS_DIR/S16-tmo-metadata/expected_response.json"; then
    echo "  [OK] S16: Master SCD Type 2"
else
    echo "  [FAIL] S16: expected Master class"
    exit 1
fi

# 5. SSE format 守门 (S18 server-push)
echo ""
echo "--- SSE format 守门 (S18 server-push) ---"
if grep -q 'text/event-stream' "$SCENARIOS_DIR/S18-streamable-push/expected_response.json"; then
    echo "  [OK] S18: text/event-stream content type"
else
    echo "  [FAIL] S18: expected text/event-stream"
    exit 1
fi

echo ""
echo "==== UAT 业务场景 16-20 regression test PASSED ===="
