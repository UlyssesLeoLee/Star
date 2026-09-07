# Brief: OPT-NEXT-02 — Phase E P3-C/E/F 编排 (per OPT-WBS-15..18, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟡 等待 5 域 Lead + SRE/平台/评审/PM 4 角色真人到位
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

完成 Phase E 5 子项, 等真人到位 (per `STAR-P4-UNIMPL-WBS-001.md` §6):

- **E.1**: E.6 5 域 Saga 实装 (跨域补偿/失败回滚 per Q-003) — 估 4.5M
- **E.2**: E.7 5 域 DDD 边界验证 (per `scripts/automation/ddd_review.py` 实证)
- **E.3**: F.1 DDD Review 阶段 5 角色真人到位 (架构+SRE+平台+评审+PM) — 估 4M
- **E.4**: CONTENT-REVIEW-PACK 21 份 docs 评审 (13 docs + 6 P3 报告 + 2 INC-SESSION)
- **E.5**: REGISTRY 5 行追溯签字 (覆盖 Mavis 临时代签)

**估 token**: 4.5M (E.1 主导)

## 2. 依赖 (per 守门 #3 + #14 v2 5 域 Lead CONTENT 4 维)

| # | 依赖 | 状态 |
|---|---|---|
| 1 | 5 域 Lead 真人到位 (T4 满员 ~ 10/17 JST) | 🔴 阻塞 |
| 2 | SRE Lead 真人到位 | 🔴 阻塞 |
| 3 | 平台 Lead 真人到位 | 🔴 阻塞 |
| 4 | 评审主持真人到位 | 🔴 阻塞 |
| 5 | PM 真人到位 | 🔴 阻塞 |
| 6 | DDD Review 阶段 5 角色全部到位 (E.3 触发) | 🔴 阻塞 |

## 3. 实施路径

### E.1 — 5 域 Saga 实装

- match 域 Lead 拍板 Saga 跨域补偿规则 (per Q-003 实证)
- 5 域各域 Lead 拍板本域事务边界
- 跨域失败回滚: 补偿事务 (compensation transaction) + retry policy
- E2E 测试: 跨 5 域 happy path + 5 类失败 (per BDD style)

### E.2 — DDD 边界验证

- `scripts/automation/ddd_review.py` 跑全 22 domain
- 验证: bounded context 不跨域 / 强类型 ID 唯一 / 事件不重复
- 报告: `docs/reports/PHASE-P4-E-DDD-REVIEW.md`

### E.3 — 5 角色真人到位

- 触发 = 真人到位 (T3 ≥1 域, T4 满员, T5 全角色)
- 拍板: per 守门 #14 v2 5 域 Lead CONTENT 4 维

### E.4 — CONTENT-REVIEW-PACK 21 docs

- 13 docs (per 守门 #3 报告族)
- 6 P3 报告 (per `STAR-P3-WBS-001.md` v0.6)
- 2 INC-SESSION (per H1 + H2 拍板)
- 真人签字覆盖 Mavis 临时代签

### E.5 — REGISTRY 5 行追溯

- 5 域 Lead 真人签字 (修订历史表 +1 行 per 域)
- 不沿用代签决策 (per守门 #1 禁回溯叙事)

## 4. Worktree

```bash
git worktree add -b feat/opt-phase-e D:/Star/.worktrees/wt-opt-phase-e main
cd D:/Star/.worktrees/wt-opt-phase-e
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err
- 守门 #3: 5 域 Lead + 4 角色真人到位签字
- 守门 #10: author=Ulysses
- 守门 #14 v2: CONTENT 4 维 (scope/RACI/timeline/代签边界)
- 守门 #12: 禁回溯叙事, BAS git 实证

## 6. 提交

5 commit (per 5 子项) 或 1 合并 commit (建议分, 便于追溯)

## 7. 失败处理

真人未到位 → 报告阻塞, 不 commit (per P4-UNIMPL §2 A.3 拍板)

## 8. 状态

🟡 **推下 session** (T4-T5 触发, 估 ~10/17 JST 之后)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 #2
- 基线: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §6
- 5 域 Lead: `docs/recruitment/5-business-domain-lead-referral.md` v0.1
