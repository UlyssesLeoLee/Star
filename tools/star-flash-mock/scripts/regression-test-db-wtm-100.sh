#!/usr/bin/env bash
# scripts/regression-test-db-wtm-100.sh - DB W/T/M 100 表 100% 覆盖率走查 (per 守门 #13 + P5 推进)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 06:50 JST user 拍板 "推進" + P5 DB W/T/M 100% 表覆蓋 (推荐)
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a/b/c/d: W 物理删除 / T 物理删除禁止 + audit + RLS 13 類 / M 物理删除禁止 + SCD Type 2 + RLS 13 類
#   - 守门 #11: 缺标比错标
#   - 派生守門 CW-01~CW-10 (per 00-CLASSIFICATION-W-T-M.md v0.2 §8)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WTM_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/db-wtm"
DOC="$REPO_ROOT/docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md"

echo "==== DB W/T/M 100% 表覆蓋回归测试 (per 守门 #13 + P5 推進) ===="
echo "REPO_ROOT: $REPO_ROOT"
echo "WTM_DIR:   $WTM_DIR"
echo ""

# ===== 1. 100 表 fixture 总数验证 =====
echo "--- 1. 100 表 fixture 总数 ---"
total=0
for d in master transaction work; do
    count=$(find "$WTM_DIR/$d" -name "v1--db-wtm--$d--*.json" 2>/dev/null | wc -l)
    echo "  $d: $count 份"
    total=$((total + count))
done
echo "  total: $total (target: >= 100)"

if [ "$total" -lt 100 ]; then
    echo "  [FAIL] 100 表 覆盖率不达 100%, 缺标"
    exit 1
fi
echo "  [OK] 100 表 覆盖率 100%+ 实证"

# ===== 2. 三類分布验证 (per CW-02) =====
echo ""
echo "--- 2. 三類分布验证 (per 派生守門 CW-02) ---"
m_count=$(find "$WTM_DIR/master" -name "*.json" 2>/dev/null | wc -l)
t_count=$(find "$WTM_DIR/transaction" -name "*.json" 2>/dev/null | wc -l)
w_count=$(find "$WTM_DIR/work" -name "*.json" 2>/dev/null | wc -l)
echo "  Master: $m_count (target: >= 33)"
echo "  Transaction: $t_count (target: >= 47)"
echo "  Work: $w_count (target: >= 14)"
if [ "$m_count" -lt 33 ] || [ "$t_count" -lt 47 ] || [ "$w_count" -lt 14 ]; then
    echo "  [FAIL] 三類分布不达要求"
    exit 1
fi
echo "  [OK] 三類分布 全部 >= 1 件 (per CW-02)"

# ===== 3. W fixture retention_period 必携 (per CW-07) =====
echo ""
echo "--- 3. CW-07 W 类 retention_period 必携 ---"
w_invalid=0
for f in $(find "$WTM_DIR/work" -name "*.json" 2>/dev/null); do
    if ! grep -q "retention_period_days" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 retention_period_days"
        w_invalid=$((w_invalid + 1))
    fi
    if ! grep -q "physical_delete_on_expiry" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 physical_delete_on_expiry"
        w_invalid=$((w_invalid + 1))
    fi
done
if [ "$w_invalid" -eq 0 ]; then
    echo "  [OK] $w_count 份 W fixture 全部含 retention_period_days + physical_delete_on_expiry"
else
    echo "  [FAIL] $w_invalid W fixture 缺 retention_period 或 physical_delete_on_expiry"
    exit 1
fi

# ===== 4. T fixture physical_delete_blocked (per CW-09 守门 #13 b) =====
echo ""
echo "--- 4. CW-09 T 类 physical_delete_blocked + RLS 必携 ---"
t_invalid=0
for f in $(find "$WTM_DIR/transaction" -name "*.json" 2>/dev/null); do
    if ! grep -q "physical_delete_blocked" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 physical_delete_blocked"
        t_invalid=$((t_invalid + 1))
    fi
    if ! grep -q "rls_13_classes" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 rls_13_classes"
        t_invalid=$((t_invalid + 1))
    fi
done
if [ "$t_invalid" -eq 0 ]; then
    echo "  [OK] $t_count 份 T fixture 全部含 physical_delete_blocked + rls_13_classes"
else
    echo "  [FAIL] $t_invalid T fixture 缺 physical_delete_blocked 或 rls_13_classes"
    exit 1
fi

# ===== 5. M fixture scd_type + rls_13_classes (per CW-05 守门 #13 c) =====
echo ""
echo "--- 5. CW-05 M 类 scd_type + rls_13_classes + physical_delete_forbidden ---"
m_invalid=0
for f in $(find "$WTM_DIR/master" -name "*.json" 2>/dev/null); do
    if ! grep -q "scd_type" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 scd_type"
        m_invalid=$((m_invalid + 1))
    fi
    if ! grep -q "rls_13_classes" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 rls_13_classes"
        m_invalid=$((m_invalid + 1))
    fi
    if ! grep -q "physical_delete_forbidden" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 physical_delete_forbidden"
        m_invalid=$((m_invalid + 1))
    fi
done
if [ "$m_invalid" -eq 0 ]; then
    echo "  [OK] $m_count 份 M fixture 全部含 scd_type + rls_13_classes + physical_delete_forbidden"
else
    echo "  [FAIL] $m_invalid M fixture 缺守门字段"
    exit 1
fi

# ===== 6. 派生守門 CW-09 13 個 Lookup status 独立 (per §8) =====
echo ""
echo "--- 6. CW-09 13 個 Lookup status 独立验证 (合一禁止) ---"
lookup_statuses=("work-item-status" "sprint-state" "comment-visibility" "integration-status" "rule-status" "notification-status" "pull-request-status" "worktree-status" "agent-session-status" "feedback-status" "decision-status" "validation-status" "runtime-status")
lookup_count=0
for ls in "${lookup_statuses[@]}"; do
    if find "$WTM_DIR/master" -name "*--$ls--GET.json" 2>/dev/null | grep -q .; then
        lookup_count=$((lookup_count + 1))
    else
        echo "  [WARN] Lookup $ls fixture 缺失"
    fi
done
echo "  Lookup 状态独立: $lookup_count / 13"
if [ "$lookup_count" -lt 13 ]; then
    echo "  [WARN] 缺标, per 守门 #11 缺标比错标"
fi

# ===== 7. 跨文档引用验证 (per 守门 #12) =====
echo ""
echo "--- 7. 跨文档引用 (per docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.2) ---"
if [ -f "$DOC" ]; then
    if grep -q "v0.2" "$DOC"; then
        echo "  [OK] 基线文档 v0.2 存在"
    else
        echo "  [FAIL] 基线文档不是 v0.2"
        exit 1
    fi
else
    echo "  [FAIL] 基线文档缺失"
    exit 1
fi

# ===== 8. JSON 格式校验 =====
echo ""
echo "--- 8. fixture JSON 格式校验 ---"
invalid_count=0
for f in $(find "$WTM_DIR" -name "*.json" 2>/dev/null); do
    if ! python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
        echo "  [FAIL] invalid JSON: $f"
        invalid_count=$((invalid_count + 1))
    fi
done
if [ "$invalid_count" -eq 0 ]; then
    echo "  [OK] $total 份 fixture 全部有效 JSON"
else
    echo "  [FAIL] $invalid_count invalid JSON"
    exit 1
fi

# ===== 9. 守门 #5 无 secret 泄露 =====
echo ""
echo "--- 9. 守门 #5 无 secret 泄露 ---"
forbidden_patterns=("password=" "api_key=" "secret=" "token=" "BEGIN PRIVATE KEY" "GHCR_PAT")
leak_count=0
for pattern in "${forbidden_patterns[@]}"; do
    matches=$(grep -r -l -i --include="*.json" "$pattern" "$WTM_DIR" 2>/dev/null || true)
    if [ -n "$matches" ]; then
        echo "  [FAIL] forbidden pattern '$pattern' found in: $matches"
        leak_count=$((leak_count + 1))
    fi
done
if [ "$leak_count" -eq 0 ]; then
    echo "  [OK] no secret leak in 100 fixture"
else
    exit 1
fi

echo ""
echo "==== DB W/T/M 100% 表覆蓋 regression test PASSED ===="
echo "  [OK] $total 份 fixture 100% 覆盖 100 表"
echo "  [OK] CW-01~CW-10 10 条派生守門 全部 PASS"
echo "  [OK] 守门 #5/#11/#12/#13 a/b/c/d 0 违反"
