#!/usr/bin/env bash
# scripts/regression-test-math.sh — Star Mock Project 数学不变量回归 (per 2026-11-21 用户发令"加入数学测试的内容")
# 数学族: vec3 / mat3 / quat / aabb / transform / coords (mock_data/math/ 30 份 COMPUTE fixture)
# oracle 设计: _generate_math_fixtures.py 算 expected, _lib_validate.py validate-math 独立重算不变量;
#   任何一侧公式漂移都会被捕获 (互为 oracle, per 守门 #11 缺标比错标)。
# 守门 (per AGENTS.md §4):
#   - 守门 #1: 失败即停 (exit 1)
#   - 守门 #11: 缺标显式列 (fixture 缺失/数量不足不掩盖)
#   - 守门 #19: Python 化累积规 (复用 _lib_validate.py, 不新造验证逻辑)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
MOCK_ROOT="$REPO_ROOT/tools/star-flash-mock"
MATH_DATA="$MOCK_ROOT/mock_data/math"
# Windows native python3 需 Windows 路径 (MSYS path conversion off, per smoke-test.sh 同款)
LIB_WIN="$(cygpath -w "$MOCK_ROOT/scripts/_lib_validate.py" 2>/dev/null || echo "$MOCK_ROOT/scripts/_lib_validate.py")"
MATH_DATA_WIN="$(cygpath -w "$MATH_DATA" 2>/dev/null || echo "$MATH_DATA")"
PYTHON_CMD="${PYTHON:-python3}"

TMPDIR_LOCAL="${TMPDIR:-${TMP:-${TEMP:-.}}}"
TMPDIR_LOCAL="$(mkdir -p "$TMPDIR_LOCAL" 2>/dev/null && cd "$TMPDIR_LOCAL" && pwd || echo .)"
MATH_OUT="$TMPDIR_LOCAL/math_validate.out"
MATH_ERR="$TMPDIR_LOCAL/math_validate.err"

echo "==== Star Mock Project 数学不变量回归 (math) ===="
echo "MATH_DATA: $MATH_DATA"

# ===== 1. 数学 fixture 目录与数量检查 (守门 #11 缺标显式列) =====
if [ ! -d "$MATH_DATA" ]; then
    echo "  [FAIL] mock_data/math/ 缺失"
    exit 1
fi
math_count=$(find "$MATH_DATA" -name "*.json" 2>/dev/null | wc -l)
echo "  math fixtures: $math_count"
if [ "$math_count" -lt 30 ]; then
    echo "  [FAIL] math fixture 数量 < 30 (6 族 × 5 case), 缺标"
    exit 1
fi
for family in vec3 mat3 quat aabb transform coords; do
    fc=$(find "$MATH_DATA/$family" -name "*.json" 2>/dev/null | wc -l)
    if [ "$fc" -ge 5 ]; then
        echo "  [OK] math/$family: $fc fixtures (>= 5)"
    else
        echo "  [FAIL] math/$family: $fc fixtures (< 5, 缺标)"
        exit 1
    fi
done

# ===== 2. 数学不变量独立重算验证 (validate-math, 互为 oracle) =====
echo ""
echo "--- 数学不变量重算验证 (validate-math) ---"
if "$PYTHON_CMD" "$LIB_WIN" validate-math "$MATH_DATA_WIN" >"$MATH_OUT" 2>"$MATH_ERR"; then
    grep -E "^TOTAL=|^FAILURES=" "$MATH_OUT"
    echo "  [OK] 全部数学不变量在容差内一致"
else
    grep -E "^TOTAL=|^FAILURES=" "$MATH_OUT" || true
    cat "$MATH_ERR"
    echo "  [FAIL] 数学不变量验证失败 (公式漂移或 fixture 损坏)"
    exit 1
fi

echo ""
echo "==== regression-test-math PASSED ===="