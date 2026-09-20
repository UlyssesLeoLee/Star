# Star Mock Project UT + IT + ST 综合回归报告 (v1.1, per ULYS-140)

> **生成时间**: 2026-09-20T21:38:06Z
> **范围**: tools/star-flash-mock/{scripts/,mock_data/,docs/,k3s/}
> **触发**: ULYS-140 (回归测试) 2026-09-20 JST
> **守门**: 守门 #1+#9+#11+#12+#13+#19+#24

## 1. 三层总结

| 层 | 范围 | PASS | FAIL | 总 |
|---|---|---|---|---|
| §UT (Unit Test) | tools/star-flash-mock/scripts/ 11 scripts | 11 | 0 | 11 |
| §IT (Integration Test) | mock_data/uat/regression/ 1 runner (35 场景) | 1 | 0 | 1 |
| §ST (System Test) | scripts/regression-test-system.sh 7 段 | 1 | 0 | 1 |
| **总计** | — | **13** | **0** | **13** |

## 2. §UT (Unit Test) 11 scripts 详细

| # | Script | 状态 |
|---|---|---|
| 1 | `[PASS] smoke-test.sh` | - |
| 2 | `[PASS] regression-test-langgraph.sh` | - |
| 3 | `[PASS] regression-test-agent-runtime.sh` | - |
| 4 | `[PASS] regression-test-agent-runtime-v2.sh` | - |
| 5 | `[PASS] regression-test-mcp.sh` | - |
| 6 | `[PASS] regression-test-streamable-http.sh` | - |
| 7 | `[PASS] regression-test-db-wtm.sh` | - |
| 8 | `[PASS] regression-test-db-wtm-100.sh` | - |
| 9 | `[PASS] regression-test-five-domain.sh` | - |
| 10 | `[PASS] regression-test-five-domain-v2.sh` | - |
| 11 | `[PASS] regression-test-openclaw.sh` | - |

## 3. §IT (Integration Test) 1 runner 详细

- `[PASS] run-it.sh (S01-S35)`

## 4. §ST (System Test) 1 runner 详细

- `[PASS] regression-test-system.sh`

## 5. mock_data fixture 统计

- `openclaw/`: 20 份 fixture
- `langgraph/tmo/`: 21 份 fixture
- `langgraph/sa-10/`: 6 份 fixture
- `langgraph/sa-01..09/`: 7 份 fixture
- `agent-runtime/l0-dispatcher/`: 3 份 fixture
- `agent-runtime/l1-ecs/`: 23 份 fixture
- `agent-runtime/l2-pools/`: 8 份 fixture
- `mcp/`: 16 份 fixture
- `streamable-http/`: 6 份 fixture
- `db-wtm/work/`: 16 份 fixture
- `db-wtm/transaction/`: 49 份 fixture
- `db-wtm/master/`: 45 份 fixture
- `five-domain/`: 30 份 fixture
- `uat/scenarios/`: 140 份 fixture

## 6. 已知缺口 (per 守门 #11 缺标比错标)

- 缺口 #1: ~~16 MCP tool 仅覆盖 6, 缺 10~~ → **已闭合** (v1.2 commit ) mock_data/mcp/ 实测 16 份 fixture 全 16 tool, smoke 验证 PASS
- 缺口 #2: ~~Agent Runtime L1 ECS 9 Archetype 仅 2 fixture~~ → **已闭合** (v1.2 同 commit) mock_data/agent-runtime/l1-ecs/ 实测 9 份 archetype (sa-01..09) + 14 份 system-*
- 缺口 #3: Streamable HTTP 5xx 错误 + retry 完整 case 缺失 (per docs/test-design/TEST-DESIGN-STREAMABLE-HTTP-001.md) — **部分闭合** v1.2 加 5xx-internal-error + retry-with-backoff 2 份 fixture
- 缺口 #4: 5 域 fixture 复用 frontend/src/mocks/data/five-domain.ts (per README v0.2 §1.2) — 跨项目, 跨 session 续
- 缺口 #5: k3s/ 仅 2 yaml, 缺 star-mock ConfigMap + Secret → **已闭合** (v1.2 同 commit) 加 k3s/star-mock-configmap.yaml + k3s/star-mock-secret.yaml (Secret 仅 K8s secretName 引用 + 占位符, 0 真实 secret)
- 缺口 #6: ST 层静态验证, 不连真 k3s (per 守门 #24 v2 G-5 mock 锁) — by design, 不闭合
- 缺口 #7: docs/ 回归报告 缺自动化 PR/CI 触发 → **已闭合** (v1.2 同 commit) 加 .github/workflows/star-flash-mock-regression.yml, PR + push 影响 tools/star-flash-mock/** 即跑 run-all.sh + upload report

## 7. 后续改进 (per ULYS-140 改进规)

1. v1.1: 增加 §ST 层 (regression-test-system.sh), 覆盖 k3s yaml + envoy + 3000 端口契约
2. v1.1: 增加 IT aggregator (run-it.sh), 6 个 IT runner 一键跑
3. v1.1: §UT 增加 3 个 v2 scripts (agent-runtime-v2, db-wtm-100, five-domain-v2)
4. **v1.2 (本轮, commit):** 闭合 #1+#2+#3+#5+#7 (CI workflow + streamable 5xx/retry fixture + star-mock ConfigMap/Secret yaml + 报告 §6 错标修正)
5. v1.3 (跨 session 续): 接入 P5 DB W/T/M 100% 覆盖率强制校验
6. v1.3 (跨 session 续): 接入 frontend handlers-5d.test.ts (per §UT 5 域 fixture 检查)

