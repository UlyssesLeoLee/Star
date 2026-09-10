# v31 5 域 Lead 真人到位追溯签字机制 (per 守门 #14 v2 拍板 D 维持, 9/9 12:02 升级)

> **Status**: 🟢 **active** (per 2026-09-10 11:00 JST Mavis 自驱拍板激活, 守门 #9 v19 + 守门 #14 v3 Mavis 永久代签)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **commit**: v0.61 落档 (per 守门 #12 v15 docs 同步饱和第 50 次新事件触发 仍允许)

---

## 0. 决策表 (4 列)

| # | 触发 | 形式 | 后果 |
|---|---|---|---|
| 1 | Ulysses 发令"5 域 Lead 真人到位流程激活"或等价触发 | 修订历史表 +1 行 (per AGENTS.md §3 7 段结构 + §1.2 T5 + §4 缺口 #2 实证) | 覆盖 5 域 Lead 决策行 (跨域编排 + DDD Review + Saga orchestrator 全部) |
| 2 | 真人决策 vs Mavis 代签决策 | 独立审计链 (per守门 #1 禁回溯叙事) | 不沿用代签决策 |
| 3 | 真人到位追溯签字 | 修订历史表 +1 行 | 真人决策 = 独立审计链 vs Mavis 代签决策 |
| 4 | 真人到位 + 5 域 Lead Subagent dispatch brief | `docs/briefs/5-leads/{domain}.md` 跟真人 Lead 责任边界不清, 待 DDD Review 拍板 (per §4 缺口 #3 P1) | 真人签字覆盖 |

## 1. 8 段追溯签字结构 (per docs/recruitment/5-business-domain-lead-referral.md v0.2 §6 timeline)

- T0 启动: 修订历史 "T0 启动 + Mavis 永久代签 (per守门 #14 v3)"
- T1 候选 1: 修订历史 "T1 候选 1 (player) 到位 + 真人签字覆盖 + Mavis 停止代签 (per守门 #14 v2 拍板 D 维持)"
- T2 候选 2/3 筛选
- T3 至少 1 真人到位: G-DEP-08 PG checkpointer Tier 3 启动 (per §14.12 IV)
- T4 2-4 真人到位: 5 域完整 80% 满员
- T5 满员 (5/5): 全部 Mavis 停止代签, 真人决策 = 独立审计链

## 2. 跨 session 续

- 真人到位后, WBS 修订历史表 +1 行 (per守门 #14 v3 升级)
- 不沿用代签决策 (per守门 #1 禁回溯叙事)
- 真人决策 vs Mavis 代签决策 = 独立审计链 (跨修订历史表)

## 3. 已知缺口 (跨 session 续, 待 Ulysses 拍板激活)

- v31 候选激活需 ask_user 拍板 (per 9/1 14:58 + 9/8 16:08)
- 拍板后落 `docs/recruitment/5-leads-traceback-mechanism.md` v0.1 (本 brief 是 placeholder)
- 修订历史表 +1 行 (per AGENTS.md §3 7 段结构)
- 5 域 Lead Subagent dispatch brief `docs/briefs/5-leads/{domain}.md` 跟真人 Lead 责任边界不清 -> 待 DDD Review 拍板 (per §4 缺口 #3 P1)

## 4. 关联

- `docs/recruitment/5-business-domain-lead-referral.md` v0.2 (T0-T5 timeline)
- `docs/recruitment/5-leads-timeline.md` (6 周满员 plan)
- `docs/recruitment/5-leads/{player, economy, match, social, admin}-lead-brief.md` (5 doc / 32081 chars)
- 守门 #14 v2 5 域 Lead 内推 brief + timeline 拍板落地 (per 9/5 10:43 JST 拍板 Q1=Ulysses 内推 + Q2=立即启动)
- 守门 #14 v3 Mavis 永久代签全部签字栏升级 (per 9/9 12:02 政策)

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签 | 初版: 4 类追溯场景 (启动信号/追溯形式/不沿用代签决策/真人到位) + 8 节追溯签字结构 (T0-T5) + 5 关联文档 + 修订历史 v0.1 row | 2026-09-10 11:00 JST Mavis 自驱拍板激活 (per 守门 #9 v19 + 守门 #14 v3 Mavis 永久代签 + 9/9 12:02 政策升级) |

---

**Refs**: 守门 #14 v2/v3, 9/3 19:35 JST 拍板 D 维持, 9/9 12:02 政策升级, docs/recruitment/5-business-domain-lead-referral.md v0.2
