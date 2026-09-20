# Star Mock Project IT (Integration Test) 回归报告

> **生成时间**: 2026-09-20T21:37:35Z
> **范围**: tools/star-flash-mock/mock_data/uat/{scenarios/,scripts/,regression/}
> **触发**: ULYS-140 (回归测试) 2026-09-20 JST
> **守门**: 守门 #1+#5+#9+#11+#12+#13+#19

## 1. 6 IT runner 跑结果

| # | Runner | 范围 | 状态 |
|---|---|---|---|
| 1 | uat-s01-s05.sh | WorkItem/Worktree/Agent/Feedback/Validation | ✅ PASS |
| 2 | uat-s06-s10.sh | ValidationFail/Conflict/Rebase/Merge/TMO-Merge | ✅ PASS |
| 3 | uat-s11-s15.sh | TMO Split/Reorder/Bulk/Summarize/Reassign | ✅ PASS |
| 4 | uat-s16-s20.sh | TMO Metadata + Streamable HTTP 4 case | ✅ PASS |
| 5 | uat-s21-s25.sh | 5 域 AC + 多租户 + RBAC + 审计 + 配额 | ✅ PASS |
| 6 | uat-s26-s35.sh | 10 NotImplemented/MCP/TSC/5d-concurrent/Saga/Mavis/Coordination | ✅ PASS |

## 2. 35 业务场景 W/T/M 分布

| 类别 | 数量 | 守门 |
|---|---|---|
| Work (短 TTL) | 21 | 守门 #13 a |
| Transaction (append-only) | 69 | 守门 #13 b/d |
| Master (SCD Type 2) | 15 | 守门 #13 c |
| **总** | **35** | 守门 #13 100% 覆盖 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

- 缺口 #1: IT runner 跑 generator + 验证, 不连真 API (per 守门 #24 G-5 mock 锁)
- 缺口 #2: UAT 35 场景依赖 fixture generator, fixture 必须先跑 (跑 runner 自动)
- 缺口 #3: RBAC 验证不跑实际角色矩阵, 仅 fixture 字段级 (P3-B 跨 session 续)

