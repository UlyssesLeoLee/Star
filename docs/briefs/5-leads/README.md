# 5 域 Lead Subagent Brief 索引 (per 守门 #14 CONTENT 4 维 + 守门 #3 反转 + 守门 #14 v2/v3)

> **Status**: 🟡 索引说明 (per docs/briefs/wt-5lead-outreach.md §3.3 落档)
> **Created**: 2026-09-09
> **依据**: commit `965a3b7 docs(v2-6): 5 域 Lead 全部子代理兼任 v0.1 (守门 #3 反转 + 守门 #14 修订, 5 子代理 brief + dispatch 协议)`
> **责任边界**: 真人 Lead 到位前 Mavis 临时代签 (per 守门 #14 v3); 真人到位后追溯签字覆盖 (per 守门 #14 v2 缺口 #2 + 守门 #1 禁回溯叙事)

---

## 0. 索引目的

5 域 × 1 Subagent Brief 文件索引, **跟 [docs/recruitment/5-business-domain-lead-referral.md v0.1](../../../recruitment/5-business-domain-lead-referral.md) 配套**:
- 内推 brief 范式 = 5 域 Lead 真人寻访 (per docs/recruitment/)
- Subagent Brief = 5 域 Lead 决策落地 (per docs/briefs/5-leads/)

## 1. 5 域 × 1 Subagent Brief 落档

| 域 | Brief 路径 | 状态 | 覆盖 |
|---|---|---|---|
| player | [./player.md](./player.md) | ✅ v0.1 已落 (per commit `965a3b7`) | 玩家生命周期 / 账户 / 角色 / 存档 / 在线状态 |
| economy | [./economy.md](./economy.md) | ✅ v0.1 已落 | 经济域 (token 计量 + 守门 #4 token-OLU) |
| match | [./match.md](./match.md) | ✅ v0.1 已落 | 匹配域 (saga orchestrator + ECS) |
| social | [./social.md](./social.md) | ✅ v0.1 已落 | 社交域 (event bus + EventBus) |
| admin | [./admin.md](./admin.md) | ✅ v0.1 已落 | 管理域 (COC 控制台 / 审计 / 合规 / 监控 / RBAC) |

## 2. 责任边界 (per 守门 #14 v2/v3 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

### 2.1 决策 scope (per 守门 #14 CONTENT 4 维 + 9/3 19:43 JST 拍板)

- 5 域 Lead 全 RACI 覆盖 (R + A + C)
- 域内: Lead 自执行 R + 负责 A + 接受域内 C 咨询, 域外 I 通知
- 跨域: 5 域 Lead 全 RACI 协调 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B 衍生)

### 2.2 真人到位前 (当前)

- 5 域 Lead 真人未到位, **Mavis 临时代签 5 域 Lead 决策** (per 守门 #14 v3 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- SubAgent Dispatch Brief 由 Mavis 维护, 真人到位后追溯签字覆盖 (per 守门 #14 v2 缺口 #2)

### 2.3 真人到位后 (期望 T0-T5)

- 5 域 Lead 真人到位后, **修订历史表 +1 行追溯签字覆盖** (per 守门 #14 v2 缺口 #2 实证)
- SubAgent Dispatch Brief 由真人 Lead 维护, Mavis 仅代理执行
- 跨域编排决策 (5 域 Lead 协调 + DDD Review + Saga orchestrator) 由真人 Lead 拍板

## 3. 引用

- [docs/recruitment/5-business-domain-lead-referral.md v0.1](../../../recruitment/5-business-domain-lead-referral.md) — 内推 brief 范式
- [docs/recruitment/status/{player,economy,match,social,admin}.md](../../../recruitment/status/) — 5 域寻访状态
- [docs/briefs/wt-5lead-outreach.md](../wt-5lead-outreach.md) — 5 域 Lead 寻访 brief
- [AGENTS.md §4 #14 5 域 Lead 独立真实身份 + 真人到位追溯](../../../../AGENTS.md)
- [AGENTS.md §4.1 v25 守门 #14 v25 5 域 Lead 内推 + timeline 拍板](../../../../AGENTS.md)
- [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.3.1 §3.3 line 174 G-DEP-03 ✅ 已拍板](../../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md)
- commit `965a3b7 docs(v2-6): 5 域 Lead 全部子代理兼任 v0.1` — 原 v0.1 落档实证

## 4. 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.1 | 2026-09-09 | 初版索引说明 (per docs/briefs/wt-5lead-outreach.md §3.3 落档); 跟原 v0.1 (commit `965a3b7`) 5 域 Lead Subagent Brief 索引对齐, 不重写原 v0.x |

---

**per 守门 #14 v3 Mavis 永久代签**: 真人到位后修订历史 +1 行追溯签字覆盖.