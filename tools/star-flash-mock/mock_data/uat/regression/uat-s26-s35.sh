#!/usr/bin/env bash
# uat-s26-s35.sh - UAT 业务场景 26-35 (3 incidents 404 + 4 mcp tools + 2 tsc err + 5 域异步 + 5 域 Saga + Mavis 追溯 + TMO 7 + L1↔L1) regression test
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-07 16:15 JST user 发令 "测试结果中是否存在404或者交互不符合预期，协作不符合预期，这些都要100%覆盖"

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scripts"
SCENARIOS_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/scenarios"

# Use py launcher (Windows-compatible, falls back to python3)
PYTHON_CMD="${PYTHON:-python3}"

echo "==== UAT 业务场景 26-35 (100% 覆盖 3 类失败/异常) regression test ===="

# 1. 跑 S26-S35 fixture generator
echo ""
echo "--- 跑 S26-S35 _generate_uat_s26_s35.py ---"
"$PYTHON_CMD" "$SCRIPTS_DIR/_generate_uat_s26_s35.py"

# 2. 验证 fixture 完整性
echo ""
echo "--- 验证 S26-S35 fixture 完整性 (10 场景 × 5 文件 = 50) ---"
for s in S26-notimplemented-incident-probe S27-notimplemented-incident-alert S28-notimplemented-incident-rollback S29-mcp-tool-failed-empty-result S30-tsc-err-render-fallback S31-async-timeout-5d-concurrent S32-5d-cross-domain-saga-coordination S33-mavis-proxy-real-person-signoff S34-tmo-7node-orchestration-coordination S35-l1-l1-prohibition-l0-coordination; do
    count=$(find "$SCENARIOS_DIR/$s" -type f 2>/dev/null | wc -l)
    if [ "$count" -eq 5 ]; then
        echo "  [OK] $s: 5 files"
    else
        echo "  [FAIL] $s: $count files (expected 5)"
        exit 1
    fi
done

# 3. 守门 #13 W/T/M 守门 (per 9/1 18:30 JST 拍板)
# 3 Master / 5 Transaction / 2 Work = 10 (per brief §1.2)
echo ""
echo "--- 守门 #13 W/T/M 守门 (per 9/1 18:30 JST 拍板) ---"
# S26-S28 = Work (404 negative result cache, 1d TTL)
# S29 = Master (mcp tool state SCD Type 2)
# S30-S32 + S34-S35 = Transaction (event log)
# S33 = Master (mavis signoff history SCD Type 2)
declare -A WTM
WTM["S26-notimplemented-incident-probe"]="Work"
WTM["S27-notimplemented-incident-alert"]="Work"
WTM["S28-notimplemented-incident-rollback"]="Work"
WTM["S29-mcp-tool-failed-empty-result"]="Master"
WTM["S30-tsc-err-render-fallback"]="Transaction"
WTM["S31-async-timeout-5d-concurrent"]="Transaction"
WTM["S32-5d-cross-domain-saga-coordination"]="Transaction"
WTM["S33-mavis-proxy-real-person-signoff"]="Master"
WTM["S34-tmo-7node-orchestration-coordination"]="Transaction"
WTM["S35-l1-l1-prohibition-l0-coordination"]="Transaction"

master_count=0
transaction_count=0
work_count=0
for s in "${!WTM[@]}"; do
    cls="${WTM[$s]}"
    if grep -q "\"class\": \"$cls\"" "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: $cls"
        case "$cls" in
            Master) master_count=$((master_count + 1)) ;;
            Transaction) transaction_count=$((transaction_count + 1)) ;;
            Work) work_count=$((work_count + 1)) ;;
        esac
    else
        echo "  [FAIL] $s: expected $cls"
        exit 1
    fi
done

# 守门: 2 Master / 5 Transaction / 3 Work (per brief §1.2 场景分类列表 S26-S28=3 Work / S29+33=2 Master / S30+31+32+34+35=5 Transaction)
echo ""
echo "--- W/T/M 守门 统计 (per brief §1.2 场景列表: 2 Master / 5 Transaction / 3 Work) ---"
echo "  Master: $master_count (expected 2)"
echo "  Transaction: $transaction_count (expected 5)"
echo "  Work: $work_count (expected 3)"
if [ "$master_count" -ne 2 ] || [ "$transaction_count" -ne 5 ] || [ "$work_count" -ne 3 ]; then
    echo "  [FAIL] W/T/M 分类不符合 brief §1.2 2/5/3"
    exit 1
fi
echo "  [OK] W/T/M 分类 2/5/3 守门通过"

# 4. 404 文案守门 (per REQ-OPS-003 §30.6 boundary)
echo ""
echo "--- 404 文案守门 (per REQ-OPS-003 §30.6) ---"
for s in S26-notimplemented-incident-probe S27-notimplemented-incident-alert S28-notimplemented-incident-rollback; do
    if grep -q "REQ-OPS-003" "$SCENARIOS_DIR/$s/expected_response.json"; then
        echo "  [OK] $s: REQ-OPS-003 boundary 文案"
    else
        echo "  [FAIL] $s: 缺 REQ-OPS-003 boundary 文案"
        exit 1
    fi
done

# 5. 4 mcp tools empty result 守门 (per 9/5 报告 §3.7 pre-existing)
echo ""
echo "--- 4 mcp tools empty result 守门 (per 9/5 报告 §3.7) ---"
for tool in find_references get_code_context get_symbol search_code; do
    if grep -q "\"$tool\"" "$SCENARIOS_DIR/S29-mcp-tool-failed-empty-result/expected_response.json"; then
        echo "  [OK] S29: $tool empty result 守门"
    else
        echo "  [FAIL] S29: 缺 $tool empty result"
        exit 1
    fi
done

# 6. 2 tsc err locations 守门 (per Worker 12 实证)
echo ""
echo "--- 2 tsc err locations 守门 (per Worker 12 实证) ---"
for loc in "src/app/agent-view/page.tsx:398" "src/lib/store.ts:562"; do
    if grep -q "$loc" "$SCENARIOS_DIR/S30-tsc-err-render-fallback/request.json"; then
        echo "  [OK] S30: $loc 守门"
    else
        echo "  [FAIL] S30: 缺 $loc"
        exit 1
    fi
done

# 7. 5 域 Lead RACI 4 维守门 (per 守门 #14 v2)
echo ""
echo "--- 5 域 Lead RACI 4 维守门 (per 守门 #14 v2) ---"
for dim in decision_scope raci timeline mavis_sign_boundary; do
    if ! grep -q "\"$dim\"" "$SCENARIOS_DIR/S32-5d-cross-domain-saga-coordination/expected_response.json"; then
        echo "  [FAIL] S32: raci_4_dim 缺 $dim"
        exit 1
    fi
done
echo "  [OK] S32: raci_4_dim 全 4 维 (decision_scope / raci / timeline / mavis_sign_boundary)"

# 8. Mavis 临时代签 → 真人到位追溯签字 守门 (per 守门 #14 v2 + #1 禁回溯叙事)
echo ""
echo "--- Mavis 临时代签追溯签字 守门 (per 守门 #14 v2 + #1 禁回溯叙事) ---"
if grep -q "Mavis 临时代签" "$SCENARIOS_DIR/S33-mavis-proxy-real-person-signoff/expected_response.json"; then
    echo "  [OK] S33: Mavis 临时代签 守门"
else
    echo "  [FAIL] S33: 缺 Mavis 临时代签 字段"
    exit 1
fi
if grep -q "Ulysses <ulysses@mavis.local>" "$SCENARIOS_DIR/S33-mavis-proxy-real-person-signoff/expected_response.json"; then
    echo "  [OK] S33: commit author = Ulysses 守门"
else
    echo "  [FAIL] S33: 缺 commit author 字段"
    exit 1
fi
if grep -q "架构师 (Mavis 接手 agent per DEC-008)" "$SCENARIOS_DIR/S33-mavis-proxy-real-person-signoff/expected_response.json"; then
    echo "  [OK] S33: 审批 = 架构师 (Mavis 接手) 守门"
else
    echo "  [FAIL] S33: 缺审批 字段"
    exit 1
fi

# 9. TMO 7 节点守门 (per LangGraph 02 §2.6 + ADR-0046)
echo ""
echo "--- TMO 7 节点守门 (per LangGraph 02 §2.6 + ADR-0046) ---"
for node in M-N1 M-N2 M-N3 M-N4 M-N5 M-N6 M-N7; do
    if ! grep -q "\"$node\"" "$SCENARIOS_DIR/S34-tmo-7node-orchestration-coordination/request.json"; then
        echo "  [FAIL] S34: 缺 TMO 节点 $node"
        exit 1
    fi
done
echo "  [OK] S34: TMO 7 节点 (M-N1..M-N7) 全守门"

# 10. L1↔L1 禁止 L0 协调 守门 (per 守门 #13 a)
echo ""
echo "--- L1↔L1 禁止 L0 协调 守门 (per 守门 #13 a) ---"
if grep -q "l1_to_l1_direct_prohibited.*true\|l1_to_l1_prohibited.*true" "$SCENARIOS_DIR/S35-l1-l1-prohibition-l0-coordination/expected_response.json"; then
    echo "  [OK] S35: L1↔L1 禁止 守门"
else
    echo "  [FAIL] S35: 缺 L1↔L1 禁止字段"
    exit 1
fi
if grep -q "l0_coordination_enforced.*true\|l0_coordinated.*true" "$SCENARIOS_DIR/S35-l1-l1-prohibition-l0-coordination/expected_response.json"; then
    echo "  [OK] S35: L0 协调强制 守门"
else
    echo "  [FAIL] S35: 缺 L0 协调强制字段"
    exit 1
fi

echo ""
echo "==== UAT 业务场景 26-35 regression test PASSED (5/5 categories) ===="
