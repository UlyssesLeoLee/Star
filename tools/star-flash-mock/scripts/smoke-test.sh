#!/usr/bin/env bash
# scripts/smoke-test.sh — Star Mock Project smoke test (per 守门 #12 commit-time + 守门 #1 集成)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 06:50 JST user 拍板 (全栈覆盖 v0.7) + ULYS-140 2026-09-20 JST (Python 化加速)
# 守门 (per AGENTS.md §4):
#   - 守门 #1: 集成就绪 (mock_data 完整 + scripts 可执行)
#   - 守门 #5: 无 secret 泄露 (fixture 不含 env 凭据)
#   - 守门 #11: 缺标比错标 (smoke 失败不掩盖, 显式列缺)
#   - 守门 #12: commit-time docs 同步 (本脚本落档 + regression-report)
#   - 守门 #19: Python 化累积规 (ULYS-140 v1.1 — 用 _lib_validate.py 替代慢 grep loop)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
MOCK_ROOT="$REPO_ROOT/tools/star-flash-mock"
MOCK_DATA="$MOCK_ROOT/mock_data"
# Use Windows-style path for python3 (Windows native, MSYS path conversion off)
LIB_WIN="$(cygpath -w "$MOCK_ROOT/scripts/_lib_validate.py" 2>/dev/null || echo "$MOCK_ROOT/scripts/_lib_validate.py")"
MOCK_DATA_WIN="$(cygpath -w "$MOCK_DATA" 2>/dev/null || echo "$MOCK_DATA")"

PYTHON_CMD="${PYTHON:-python3}"

echo "==== Star Mock Project smoke test ===="
echo "REPO_ROOT: $REPO_ROOT"
echo "MOCK_ROOT: $MOCK_ROOT"
echo ""

# ===== 1. 目录结构检查 =====
echo "--- 1. 目录结构检查 ---"
for dir in scripts mock_data docs k3s mock_data/openclaw mock_data/langgraph mock_data/agent-runtime mock_data/mcp mock_data/streamable-http mock_data/db-wtm mock_data/uat; do
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

# Use a writable temp dir; /tmp may not be mounted in MSYS shells under Multica Hermes
TMPDIR_LOCAL="${TMPDIR:-${TMP:-${TEMP:-.}}}"
TMPDIR_LOCAL="$(mkdir -p "$TMPDIR_LOCAL" 2>/dev/null && cd "$TMPDIR_LOCAL" && pwd || echo .)"
SMOKE_SECRET_OUT="$TMPDIR_LOCAL/smoke_secret.out"
SMOKE_SECRET_ERR="$TMPDIR_LOCAL/smoke_secret.err"
SMOKE_JSON_OUT="$TMPDIR_LOCAL/smoke_json.out"
SMOKE_JSON_ERR="$TMPDIR_LOCAL/smoke_json.err"

# ===== 3. 守门 #5 无 secret 泄露 (Python batch — per ULYS-140 v1.1 加速) =====
echo ""
echo "--- 3. 守门 #5 无 secret 泄露 (Python batch) ---"
if "$PYTHON_CMD" "$LIB_WIN" scan-secret "$MOCK_DATA_WIN" >"$SMOKE_SECRET_OUT" 2>"$SMOKE_SECRET_ERR"; then
    grep -E "^TOTAL=|^LEAKS=" "$SMOKE_SECRET_OUT"
    echo "  [OK] no secret leak in fixtures"
else
    cat "$SMOKE_SECRET_ERR"
    echo "  [FAIL] secret leak detected"
    exit 1
fi

# ===== 4. fixture JSON 格式校验 (Python batch — per ULYS-140 v1.1 加速) =====
echo ""
echo "--- 4. fixture JSON 格式校验 (Python batch) ---"
if "$PYTHON_CMD" "$LIB_WIN" validate-json "$MOCK_DATA_WIN" >"$SMOKE_JSON_OUT" 2>"$SMOKE_JSON_ERR"; then
    grep -E "^TOTAL=|^INVALID=" "$SMOKE_JSON_OUT"
    echo "  [OK] all fixtures valid JSON"
else
    cat "$SMOKE_JSON_ERR"
    echo "  [FAIL] invalid JSON detected"
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
