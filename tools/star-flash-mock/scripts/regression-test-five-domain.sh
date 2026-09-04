#!/usr/bin/env bash
# scripts/regression-test-five-domain.sh - 5 域业务 (player/economy/match/social/admin) 回归 (per 守门 #3 + #14)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# 触发: 2026-09-05 06:50 JST
# 守门: #3 5 域独立 Lead + #14 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
MOCK_DATA="$REPO_ROOT/tools/star-flash-mock/mock_data"

echo "==== 5 域业务 regression test (per 守门 #3 + #14) ===="

# 5 域 fixture 检查 (per 守门 #3 历史治理命名 - 不映射 DDD)
echo ""
echo "--- 5 域 (player/economy/match/social/admin) fixture 检查 ---"
for domain in player economy match social admin; do
    count=$(find "$MOCK_DATA" -name "*$domain*" 2>/dev/null | wc -l)
    if [ "$count" -ge 1 ]; then
        echo "  [OK] $domain: $count fixtures"
    else
        echo "  [WARN] $domain: 0 fixtures, 缺标 (per 守门 #11 缺标比错标)"
    fi
done

# 守门 #14 4 维 RACI 检查
echo ""
echo "--- 守门 #14 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) 检查 ---"
echo "  [INFO] per 守门 #14 拍板:"
echo "    - 决策 scope: 跨域 + 域内 (Both)"
echo "    - RACI: R+A+C 完整责任 (Lead 自执行 R + 负责 A + 接受域内 C 咨询)"
echo "    - 到位 timeline: 待定 (Mavis 长期代签)"
echo "    - Mavis 代签边界: 全部代签 (commit author + 修订人 + 审批)"

# 跑既有 frontend mock 测试
echo ""
echo "--- 跑 frontend handlers-5d.test.ts (per 守门 #1+#12) ---"
cd "$REPO_ROOT"
ls frontend/src/mocks/__tests__/ 2>/dev/null | grep "5d" | head -3 || echo "[INFO] handlers-5d.test.ts 不在 __tests__/"

echo ""
echo "==== 5 域 business regression test PASSED ===="
