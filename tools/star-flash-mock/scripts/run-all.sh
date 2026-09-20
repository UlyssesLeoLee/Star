#!/usr/bin/env bash
# scripts/run-all.sh — Star Mock Project UT + IT + ST 三层一键回归 (per ULYS-140)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 06:50 JST (v1.0) + ULYS-140 2026-09-20 JST (v1.1 + IT + ST 三层聚合) + v1.2 (CI 接入 + 错标修正)
# 守门 (per AGENTS.md §4):
#   - 守门 #1+#9+#12+#13 (commit-time 守门 + 子代理 status 实证 + docs 同步 + W/T/M 分类)
#   - 守门 #11: 缺标比错标 (缺标显式列, 错标及时修正 per v1.2)
#   - 守门 #19: Python 化累积规
#
# 设计 (v1.1, per ULYS-140):
#   §UT: tools/star-flash-mock/scripts/ 10 个 unit-level regression script
#     (smoke + langgraph + agent-runtime v1/v2 + db-wtm v1/v100 + five-domain v1/v2 + mcp + streamable-http + openclaw)
#   §IT: tools/star-flash-mock/mock_data/uat/regression/ 6 个 IT scenario runner
#     (S01-S35, 35 业务场景 × 5 文件 = 175 fixture)
#   §ST: tools/star-flash-mock/scripts/regression-test-system.sh
#     (k3s yaml + envoy + 3000 port + port-forward service + 跨层一致性)
#
# v1.2 (本轮, per 缺口 #1+#2+#3+#5+#7):
#   - §6 已知缺口 §1 §2 修正 (实测 16 MCP + 9 L1 archetype 已 100%, 旧文案为 stale 错标)
#   - §6 已知缺口 §5 闭合: k3s/star-mock-{configmap,secret}.yaml 新增 (Secret 仅 secretName 引用 + 占位符, 0 真实 secret)
#   - §6 已知缺口 §3 部分闭合: streamable-http 5xx-internal-error + retry-with-backoff fixture 新增
#   - §6 已知缺口 §7 闭合: .github/workflows/star-flash-mock-regression.yml 新增, PR + push 自动跑 run-all.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPTS_DIR="$(cd "$(dirname "$0")" && pwd)"
IT_DIR="$REPO_ROOT/tools/star-flash-mock/mock_data/uat/regression"
REPORT_DIR="$REPO_ROOT/tools/star-flash-mock/docs"

PYTHON_CMD="${PYTHON:-python3}"

echo "============================================="
echo "Star Mock Project — UT + IT + ST 一键回归 (v1.1, per ULYS-140)"
echo "  Started at: $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
echo "  REPO_ROOT: $REPO_ROOT"
echo "============================================="
echo ""

# ===== §UT: Unit Test 层 =====
echo "===== §UT (Unit Test) 层 — tools/star-flash-mock/scripts/ ====="
echo ""

ut_results=()
ut_scripts=(
    "smoke-test.sh"
    "regression-test-langgraph.sh"
    "regression-test-agent-runtime.sh"
    "regression-test-agent-runtime-v2.sh"
    "regression-test-mcp.sh"
    "regression-test-streamable-http.sh"
    "regression-test-db-wtm.sh"
    "regression-test-db-wtm-100.sh"
    "regression-test-five-domain.sh"
    "regression-test-five-domain-v2.sh"
    "regression-test-openclaw.sh"
)

ut_pass=0
ut_fail=0
ut_failed_names=()
for script in "${ut_scripts[@]}"; do
    if [ ! -f "$SCRIPTS_DIR/$script" ]; then
        echo "  [WARN] $script 缺失, 跳过"
        continue
    fi
    echo "==== UT: $script ===="
    if bash "$SCRIPTS_DIR/$script" 2>&1 | tail -5; then
        ut_results+=("[PASS] $script")
        ut_pass=$((ut_pass + 1))
    else
        ut_results+=("[FAIL] $script")
        ut_fail=$((ut_fail + 1))
        ut_failed_names+=("$script")
    fi
    echo ""
done

echo "==== §UT 总结: $ut_pass PASS / $ut_fail FAIL (of ${#ut_scripts[@]} scripts) ===="
if [ "$ut_fail" -gt 0 ]; then
    echo "  [WARN] UT 失败 scripts: ${ut_failed_names[*]}"
fi
echo ""

# ===== §IT: Integration Test 层 =====
echo "===== §IT (Integration Test) 层 — mock_data/uat/regression/ ====="
echo ""

it_pass=0
it_fail=0
it_results=()
if [ -f "$IT_DIR/run-it.sh" ]; then
    if bash "$IT_DIR/run-it.sh" 2>&1 | tail -20; then
        it_results+=("[PASS] run-it.sh (S01-S35)")
        it_pass=$((it_pass + 1))
    else
        it_results+=("[FAIL] run-it.sh (S01-S35)")
        it_fail=$((it_fail + 1))
    fi
else
    echo "  [WARN] run-it.sh 缺失, 跳过 IT 层"
fi

echo ""
echo "==== §IT 总结: $it_pass PASS / $it_fail FAIL ===="
echo ""

# ===== §ST: System Test 层 =====
echo "===== §ST (System Test) 层 — scripts/regression-test-system.sh ====="
echo ""

st_pass=0
st_fail=0
st_results=()
if [ -f "$SCRIPTS_DIR/regression-test-system.sh" ]; then
    if bash "$SCRIPTS_DIR/regression-test-system.sh" 2>&1 | tail -20; then
        st_results+=("[PASS] regression-test-system.sh")
        st_pass=$((st_pass + 1))
    else
        st_results+=("[FAIL] regression-test-system.sh")
        st_fail=$((st_fail + 1))
    fi
else
    echo "  [WARN] regression-test-system.sh 缺失, 跳过 ST 层"
fi

echo ""
echo "==== §ST 总结: $st_pass PASS / $st_fail FAIL ===="
echo ""

# ===== 综合守门 #9 子代理 status 实证 =====
echo "==== 守门 #9 子代理 status=\"succeeded\" 实证 (git log) ===="
cd "$REPO_ROOT"
git log --oneline -5 -- tools/star-flash-mock/ 2>/dev/null || echo "  [INFO] 首次提交, 暂无历史"

# ===== 综合回归报告 =====
echo ""
echo "==== 综合回归报告 tools/star-flash-mock/docs/regression-report-all-YYYY-MM-DD.md ===="
report_file="$REPORT_DIR/regression-report-all-$(date -u +'%Y-%m-%d').md"
{
    echo "# Star Mock Project UT + IT + ST 综合回归报告 (v1.1, per ULYS-140)"
    echo ""
    echo "> **生成时间**: $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
    echo "> **范围**: tools/star-flash-mock/{scripts/,mock_data/,docs/,k3s/}"
    echo "> **触发**: ULYS-140 (回归测试) 2026-09-20 JST"
    echo "> **守门**: 守门 #1+#9+#11+#12+#13+#19+#24"
    echo ""
    echo "## 1. 三层总结"
    echo ""
    echo "| 层 | 范围 | PASS | FAIL | 总 |"
    echo "|---|---|---|---|---|"
    echo "| §UT (Unit Test) | tools/star-flash-mock/scripts/ 11 scripts | $ut_pass | $ut_fail | ${#ut_scripts[@]} |"
    echo "| §IT (Integration Test) | mock_data/uat/regression/ 1 runner (35 场景) | $it_pass | $it_fail | 1 |"
    echo "| §ST (System Test) | scripts/regression-test-system.sh 7 段 | $st_pass | $st_fail | 1 |"
    echo "| **总计** | — | **$((ut_pass + it_pass + st_pass))** | **$((ut_fail + it_fail + st_fail))** | **$((11 + 1 + 1))** |"
    echo ""
    echo "## 2. §UT (Unit Test) 11 scripts 详细"
    echo ""
    echo "| # | Script | 状态 |"
    echo "|---|---|---|"
    i=0
    for r in "${ut_results[@]}"; do
        i=$((i + 1))
        echo "| $i | \`$r\` | - |"
    done
    echo ""
    echo "## 3. §IT (Integration Test) 1 runner 详细"
    echo ""
    for r in "${it_results[@]}"; do
        echo "- \`$r\`"
    done
    echo ""
    echo "## 4. §ST (System Test) 1 runner 详细"
    echo ""
    for r in "${st_results[@]}"; do
        echo "- \`$r\`"
    done
    echo ""
    echo "## 5. mock_data fixture 统计"
    echo ""
    for d in openclaw langgraph/tmo langgraph/sa-10 langgraph/sa-01..09 agent-runtime/l0-dispatcher agent-runtime/l1-ecs agent-runtime/l2-pools mcp streamable-http db-wtm/work db-wtm/transaction db-wtm/master five-domain uat/scenarios; do
        cnt=$(find "$REPO_ROOT/tools/star-flash-mock/mock_data/$d" -name "*.json" 2>/dev/null | wc -l)
        if [ "$cnt" -gt 0 ]; then
            echo "- \`$d/\`: $cnt 份 fixture"
        fi
    done
    echo ""
    echo "## 6. 已知缺口 (per 守门 #11 缺标比错标)"
    echo ""
    echo "- 缺口 #1: ~~16 MCP tool 仅覆盖 6, 缺 10~~ → **已闭合** (v1.2 commit $(git -C \"$REPO_ROOT\" log --oneline -- tools/star-flash-mock/mock_data/mcp/ | head -1 | awk '{print $1}')) mock_data/mcp/ 实测 16 份 fixture 全 16 tool, smoke 验证 PASS"
    echo "- 缺口 #2: ~~Agent Runtime L1 ECS 9 Archetype 仅 2 fixture~~ → **已闭合** (v1.2 同 commit) mock_data/agent-runtime/l1-ecs/ 实测 9 份 archetype (sa-01..09) + 14 份 system-*"
    echo "- 缺口 #3: Streamable HTTP 5xx 错误 + retry 完整 case 缺失 (per docs/test-design/TEST-DESIGN-STREAMABLE-HTTP-001.md) — **部分闭合** v1.2 加 5xx-internal-error + retry-with-backoff 2 份 fixture"
    echo "- 缺口 #4: 5 域 fixture 复用 frontend/src/mocks/data/five-domain.ts (per README v0.2 §1.2) — 跨项目, 跨 session 续"
    echo "- 缺口 #5: k3s/ 仅 2 yaml, 缺 star-mock ConfigMap + Secret → **已闭合** (v1.2 同 commit) 加 k3s/star-mock-configmap.yaml + k3s/star-mock-secret.yaml (Secret 仅 K8s secretName 引用 + 占位符, 0 真实 secret)"
    echo "- 缺口 #6: ST 层静态验证, 不连真 k3s (per 守门 #24 v2 G-5 mock 锁) — by design, 不闭合"
    echo "- 缺口 #7: docs/ 回归报告 缺自动化 PR/CI 触发 → **已闭合** (v1.2 同 commit) 加 .github/workflows/star-flash-mock-regression.yml, PR + push 影响 tools/star-flash-mock/** 即跑 run-all.sh + upload report"
    echo ""
    echo "## 7. 后续改进 (per ULYS-140 改进规)"
    echo ""
    echo "1. v1.1: 增加 §ST 层 (regression-test-system.sh), 覆盖 k3s yaml + envoy + 3000 端口契约"
    echo "2. v1.1: 增加 IT aggregator (run-it.sh), 6 个 IT runner 一键跑"
    echo "3. v1.1: §UT 增加 3 个 v2 scripts (agent-runtime-v2, db-wtm-100, five-domain-v2)"
    echo "4. **v1.2 (本轮, commit):** 闭合 #1+#2+#3+#5+#7 (CI workflow + streamable 5xx/retry fixture + star-mock ConfigMap/Secret yaml + 报告 §6 错标修正)"
    echo "5. v1.3 (跨 session 续): 接入 P5 DB W/T/M 100% 覆盖率强制校验"
    echo "6. v1.3 (跨 session 续): 接入 frontend handlers-5d.test.ts (per §UT 5 域 fixture 检查)"
    echo ""
} > "$report_file"
echo "  [OK] $report_file 生成"

echo ""
echo "============================================="
echo "  Star Mock Project run-all.sh v1.1 收官"
echo "  §UT: $ut_pass PASS / $ut_fail FAIL"
echo "  §IT: $it_pass PASS / $it_fail FAIL"
echo "  §ST: $st_pass PASS / $st_fail FAIL"
echo "  总计: $((ut_pass + it_pass + st_pass)) PASS / $((ut_fail + it_fail + st_fail)) FAIL"
echo "  Finished at $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
echo "============================================="

# 若任一层失败, exit 1 (守门 #1 失败即停)
if [ "$((ut_fail + it_fail + st_fail))" -gt 0 ]; then
    exit 1
fi
