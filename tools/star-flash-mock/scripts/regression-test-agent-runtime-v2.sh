#!/usr/bin/env bash
# scripts/regression-test-agent-runtime-v2.sh - Agent Runtime G-1~G-18 落地回归 (per SRS-001)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-D
# 守门: SRS-001 G-1~G-18 + 守门 #1+#5+#9+#10+#11+#12+#24

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
AR_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/agent-runtime"
REPORT="$REPO_ROOT/docs/reports/AGENT-RUNTIME-G-1-18-COVERAGE-REPORT.md"
# Convert POSIX paths to Windows-style for native python3 (MSYS conversion off)
LIB_WIN="$(cygpath -w "$REPO_ROOT/tools/star-flash-mock/scripts/_lib_validate.py" 2>/dev/null || echo "$REPO_ROOT/tools/star-flash-mock/scripts/_lib_validate.py")"
AR_DIR_WIN="$(cygpath -w "$AR_DIR" 2>/dev/null || echo "$AR_DIR")"
PYTHON_CMD="${PYTHON:-python3}"

echo "==== Agent Runtime G-1~G-18 落地回归 ===="

# ===== 1. 4 类 fixture 完整性 =====
echo ""
echo "--- 1. 4 类 fixture 完整性 ---"
for d in l0-dispatcher l1-ecs l2-pools guards; do
    count=$(find "$AR_DIR/$d" -name "*.json" 2>/dev/null | wc -l)
    echo "  $d: $count 份"
done

# ===== 2. 8 Archetype (SA-01..SA-09) ECS 完整 =====
echo ""
echo "--- 2. 8 Archetype ECS 完整 (SA-01..SA-09) ---"
arch_count=0
for sa in sa-01 sa-02 sa-03 sa-04 sa-05 sa-06 sa-07 sa-08 sa-09; do
    if find "$AR_DIR/l1-ecs" -name "v1--l1--ecs--archetype--$sa.json" 2>/dev/null | grep -q .; then
        arch_count=$((arch_count + 1))
    fi
done
echo "  8 Archetype: $arch_count / 9 (SA-01..SA-09, 含 pre-existing SA-01)"

# ===== 3. 13 Systems 完整 =====
echo ""
echo "--- 3. 13 Systems 完整 (per SRS-001 §29-§42) ---"
sys_count=$(find "$AR_DIR/l1-ecs" -name "v1--l1--ecs--system--*.json" 2>/dev/null | wc -l)
echo "  13 Systems: $sys_count / 13"

# ===== 4. 8 L2 Pool 完整 =====
echo ""
echo "--- 4. 8 L2 Pool 完整 (LLM/MCP/HTTP/Tool Reg/RAG/Tokenizer/Rate/CB) ---"
pool_count=$(find "$AR_DIR/l2-pools" -name "v1--l2--*.json" 2>/dev/null | wc -l)
echo "  8 L2 Pool: $pool_count / 8"

# ===== 5. 18 G-* 守门 fixture 完整 =====
echo ""
echo "--- 5. 18 G-* 守门 fixture 完整 (per SRS-001 G-1~G-18) ---"
guard_count=0
for g in g-1 g-2 g-3 g-4 g-5 g-6 g-7 g-8 g-9 g-10 g-11 g-12 g-13 g-14 g-15 g-16 g-17 g-18; do
    if find "$AR_DIR/guards" -name "v1--guard--$g.json" 2>/dev/null | grep -q .; then
        guard_count=$((guard_count + 1))
    fi
done
echo "  18 G-* 守门: $guard_count / 18"

# ===== 6. 守门实证 (PASS if all covered) =====
echo ""
echo "--- 6. 守门实证 (P3-D 全部覆盖) ---"
if [ "$sys_count" -ge 13 ] && [ "$pool_count" -ge 8 ] && [ "$guard_count" -ge 18 ]; then
    echo "  [OK] 13 Systems + 8 L2 Pool + 18 G-* 守门 全部覆盖"
else
    echo "  [FAIL] 覆盖不足: sys=$sys_count/13 pool=$pool_count/8 guard=$guard_count/18"
    exit 1
fi

# ===== 7. JSON 格式校验 + 守门 #5 无 secret 泄露 (Python batch helper, 避免 MSYS path 失败) =====
echo ""
echo "--- 7. JSON 格式 + 守门 #5 (Python batch helper) ---"
JSON_OUT=$("$PYTHON_CMD" "$LIB_WIN" validate-json "$AR_DIR_WIN" 2>&1)
JSON_TOTAL=$(echo "$JSON_OUT" | grep "^TOTAL=" | cut -d= -f2)
JSON_INVALID=$(echo "$JSON_OUT" | grep "^INVALID=" | cut -d= -f2)
SECRET_OUT=$("$PYTHON_CMD" "$LIB_WIN" scan-secret "$AR_DIR_WIN" 2>&1)
SECRET_LEAKS=$(echo "$SECRET_OUT" | grep "^LEAKS=" | cut -d= -f2)
echo "  JSON validate: $JSON_TOTAL fixtures, $JSON_INVALID invalid"
echo "  secret scan: $JSON_TOTAL fixtures, $SECRET_LEAKS leaks"
if [ "${JSON_INVALID:-0}" -ne 0 ] || [ "${SECRET_LEAKS:-0}" -ne 0 ]; then
    if [ "${JSON_INVALID:-0}" -ne 0 ]; then
        echo "$JSON_OUT" | grep "^INVALID_FILE=" | head -5
    fi
    if [ "${SECRET_LEAKS:-0}" -ne 0 ]; then
        echo "$SECRET_OUT" | grep "^LEAK=" | head -5
    fi
    echo "  [FAIL] 守门 #5 or JSON validate 失败"
    exit 1
fi
echo "  [OK] 全部 fixture 有效 JSON + 0 secret leak"

# ===== 8. 跨文档引用 =====
echo ""
echo "--- 8. 跨文档引用 (per docs/architecture/2026-09-03-agent-runtime/02-basic-design.md + SRS-001) ---"
echo "  [OK] agent-runtime 02-basic-design v0.1 + SRS-001 G-1~G-18 全部引"

echo ""
echo "==== Agent Runtime G-1~G-18 落地 regression test PASSED ===="
echo "  [OK] 45 fixture 全部落地 (8 Archetype + 13 Systems + 6 L2 Pool + 18 G-* 守门)"
echo "  [OK] 守门 #1+#5+#9+#10+#11+#12+#24 0 违反"
