#!/usr/bin/env bash
# scripts/regression-test-five-domain-v2.sh - 5 域 Lead 30 fixture 走查 (per 守门 #3 + #14)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-B 5 域 Lead 30 fixture
# 守门: #3 5 域独立 Lead (不映射 DDD) + #14 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
FIVE_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/five-domain"
AUDIT_DOC="$REPO_ROOT/docs/qa/STAR-FIVE-DOMAIN-LEAD-AUDIT-001.md"

echo "==== 5 域 Lead 30 fixture 走查 (per 守门 #3 + #14) ===="

# ===== 1. 5 域 fixture 完整性 =====
echo ""
echo "--- 1. 5 域 fixture 完整性 (5 域 × 6 case = 30) ---"
domains=(player economy match social admin)
total=0
for d in "${domains[@]}"; do
    count=$(find "$FIVE_DIR/$d" -name "*.json" 2>/dev/null | wc -l)
    echo "  $d: $count 份 (target: 6)"
    if [ "$count" -ne 6 ]; then
        echo "  [FAIL] $d 域 fixture 不达 6 份"
        exit 1
    fi
    total=$((total + count))
done
echo "  total: $total 份 (target: 30)"

if [ "$total" -ne 30 ]; then
    echo "  [FAIL] 5 域总 fixture 不达 30 份"
    exit 1
fi
echo "  [OK] 30 份 fixture 全部落地"

# ===== 2. RACI 4 维验证 (per 守门 #14) =====
echo ""
echo "--- 2. 守门 #14 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) ---"
raci_invalid=0
for f in $(find "$FIVE_DIR" -name "*.json" 2>/dev/null); do
    for dim in "决策_scope" "RACI" "到位_timeline" "Mavis_代签边界"; do
        if ! grep -q "$dim" "$f" 2>/dev/null; then
            echo "  [FAIL] $f 缺 RACI 4 维: $dim"
            raci_invalid=$((raci_invalid + 1))
        fi
    done
done
if [ "$raci_invalid" -eq 0 ]; then
    echo "  [OK] 30 份 fixture 全部含 RACI 4 维"
else
    echo "  [FAIL] $raci_invalid fixture 缺 RACI 4 维"
    exit 1
fi

# ===== 3. 5 域 Lead 真人到位 audit (per 守门 #3) =====
echo ""
echo "--- 3. 5 域 Lead 真人到位 audit (per 守门 #3) ---"
audit_invalid=0
for f in $(find "$FIVE_DIR" -name "*.json" 2>/dev/null); do
    if ! grep -q "Mavis 临时代签" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 Mavis 临时代签 audit"
        audit_invalid=$((audit_invalid + 1))
    fi
    if ! grep -q "lead_status" "$f" 2>/dev/null; then
        echo "  [FAIL] $f 缺 lead_status"
        audit_invalid=$((audit_invalid + 1))
    fi
done
if [ "$audit_invalid" -eq 0 ]; then
    echo "  [OK] 30 份 fixture 全部含 Mavis 临时代签 audit + lead_status"
else
    echo "  [FAIL] $audit_invalid fixture 缺 audit 字段"
    exit 1
fi

# ===== 4. 6 case 完整 (list / create / update / soft_delete / raci_check / mavis_sign) =====
echo ""
echo "--- 4. 6 case 完整 (5 域 × 6) ---"
cases=(01-list 02-create 03-update 04-soft-delete 05-raci 06-mavis-sign)
case_total=0
for d in "${domains[@]}"; do
    for c in "${cases[@]}"; do
        count=$(find "$FIVE_DIR/$d" -name "*$c*.json" 2>/dev/null | wc -l)
        if [ "$count" -lt 1 ]; then
            echo "  [FAIL] $d 域缺 case $c"
            exit 1
        fi
        case_total=$((case_total + count))
    done
done
echo "  6 case × 5 域 = 30 份全部含"

# ===== 5. 跨文档引用 (per 守门 #12) =====
echo ""
echo "--- 5. 跨文档引用 (per docs/test-design.md v0.7 §25 + docs/qa/STAR-FIVE-DOMAIN-LEAD-AUDIT-001.md) ---"
if [ -f "$AUDIT_DOC" ]; then
    echo "  [OK] audit doc 存在"
else
    echo "  [WARN] audit doc 缺失, 缺标"
fi

# ===== 6. JSON 格式校验 =====
echo ""
echo "--- 6. fixture JSON 格式校验 ---"
invalid_count=0
for f in $(find "$FIVE_DIR" -name "*.json" 2>/dev/null); do
    if ! python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
        echo "  [FAIL] invalid JSON: $f"
        invalid_count=$((invalid_count + 1))
    fi
done
if [ "$invalid_count" -eq 0 ]; then
    echo "  [OK] 30 份 fixture 全部有效 JSON"
else
    exit 1
fi

# ===== 7. 守门 #5 无 secret 泄露 =====
echo ""
echo "--- 7. 守门 #5 无 secret 泄露 ---"
forbidden_patterns=("password=" "api_key=" "secret=" "BEGIN PRIVATE KEY" "GHCR_PAT")
leak_count=0
for pattern in "${forbidden_patterns[@]}"; do
    matches=$(grep -r -l -i --include="*.json" "$pattern" "$FIVE_DIR" 2>/dev/null || true)
    if [ -n "$matches" ]; then
        echo "  [FAIL] forbidden pattern '$pattern' found in: $matches"
        leak_count=$((leak_count + 1))
    fi
done
if [ "$leak_count" -eq 0 ]; then
    echo "  [OK] no secret leak in 30 fixture"
else
    exit 1
fi

echo ""
echo "==== 5 域 Lead 30 fixture regression test PASSED ===="
echo "  [OK] 30 份 fixture (5 域 × 6 case) 全部落地"
echo "  [OK] 守门 #3 5 域独立 Lead 验证 (Mavis 临时代签)"
echo "  [OK] 守门 #14 RACI 4 维 全部含"
echo "  [OK] 6 case × 5 域 完整"
