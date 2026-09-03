# DDD Review 阶段 Lead 真实身份采集流程 v0.1

> **状态**: 🟡 草案 v0.1
> **日期**: 2026-08-29
> **基点 commit**: `789913e`
> **触发**: 8/21 JST 5 域独立 Lead 拒绝兼任硬约束 + 8/27 21:59 JST AGENTS.md §9 5 域真实身份 DDD Review 阶段补 + F1-LeadRoster (33c38c1) 14 个 [DDD Review 阶段补] 空位
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
> **签批**: 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化)

---

## 0. 目的

补 F1-LeadRoster 14 个 [DDD Review 阶段补] 空位的具体流程 — 谁、何时、如何采集, 签字流程, 归档位置.

## 1. 5 域 Lead 真实身份采集 (per 8/21 JST 拒绝兼任)

### 1.1 5 域分配

| 域 | 范围 | 实际 1 人 12 角色 (Ulysses) | Lead 真实身份 |
|---|---|---|---|
| 架构 | 全栈架构决策 | ✅ (Ulysses 本人, per DEC-008) | Ulysses (已 8/27 签字) |
| SRE | 生产环境/监控/告警 | ❌ (待 DDD Review) | **[DDD Review 阶段补]** |
| 平台 | 平台工程/DevOps/CI/CD | ❌ (待 DDD Review) | **[DDD Review 阶段补]** |
| 评审 | 设计评审/PR review/守门 | ❌ (待 DDD Review) | **[DDD Review 阶段补]** |
| PM | 项目管理/优先级/资源 | ❌ (待 DDD Review) | **[DDD Review 阶段补]** |

### 1.2 采集流程 (4 步)

**Step 1: 候选人提名 (T+0)**
- Ulysses 通过邮件/IM/会议提名 4 域 (SRE/平台/评审/PM) 候选人
- 候选条件: 5+ 年相关域经验 + 真实身份 + 可签字承诺
- 拒绝兼任 (per 8/21 JST 硬约束): 1 候选人只可负责 1 域

**Step 2: 候选人确认 (T+1 周)**
- 候选人确认接受, 提供: 真实姓名 + 邮箱 + 经验 (年) + 签字承诺
- Ulysses 维护 `RGS-LEAD-ROSTER.md` + `STAR-LEAD-ROSTER.md`, 替换 [DDD Review 阶段补] 占位

**Step 3: 签字 (T+2 周, DDD Review 会议)**
- 4 域 Lead 各自在 RGS-LEAD-ROSTER.md + STAR-LEAD-ROSTER.md 签字 (per 守门 12 项 #10 代签规则应用)
- Ulysses 主持会议, 4 Lead 5 角色 (架构+SRE+平台+评审+PM) 全员签字

**Step 4: 归档 (T+2 周)**
- 4 域 Lead 真实身份 commit 入 git
- 邮箱按 8/27 11:06 JST hard ban 仍用 redacted (实际邮箱不入 git, 内部通讯录)
- commit author = Ulysses (per 8/27 19:39/21:59 JST 三次强化)

## 2. 12 域 Lead 真实身份采集 (per DEC-008)

类似 5 域流程, 但 12 域 5 域独立真实身份 + 6 域 Ulysses 兼任 (架构/治理/DEC-008/Ulysses 本人). 范围 2x5 + 6 = 16 域, 实际只有 5 域需 DDD Review 补真实身份.

## 3. 拒绝兼任硬约束 (per 8/21 JST)

1 候选人只可负责 1 域. Q-003 Saga 跨域核心问题需要 5 域 Lead 独立决策权, 兼任会让责任矩阵和 RACI 模糊化.

## 4. 守门 (per AGENTS.md §4 12 项)

- ✅ **R-05 不 push** (8/27 11:09 JST): 真实身份 commit 后, Mavis 接手 push (per 8/29 03:30 JST 推 29 commit 经验)
- ✅ **5 域独立 Lead 不兼任** (8/21 JST): 1 候选人 1 域
- ✅ **代签规则应用** (8/27 19:39/21:59 JST): 4 域 Lead 签字 author = Ulysses
- ✅ **缺标比错标安全** (8/26 JST): 14 个空位显式, 实际填写时不编造
- ✅ **AI 协作文档治理** (8/26 JST): 无回溯叙事
- ✅ **环境变量安全** (8/27 11:06 JST hard ban): 邮箱 redacted, 不入 git

## 5. 时间表

| 阶段 | T+ | 责任 | 交付 |
|---|---|---|---|
| Step 1 候选人提名 | 0 | Ulysses | 4 域候选人列表 |
| Step 2 候选人确认 | +1 周 | 4 候选 | 真实姓名/邮箱/经验/签字 |
| Step 3 签字会议 | +2 周 | Ulysses + 4 Lead | RGS-LEAD-ROSTER.md + STAR-LEAD-ROSTER.md 替换 14 个空位 |
| Step 4 归档 commit | +2 周 | Ulysses | 1 commit author=Ulysses |

## 6. 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构 | Ulysses (一人公司 12 角色 per DEC-008) | 2026-08-29 | 🟢 Active; DDD Review 流程模板 v0.1 |
| 2 | SRE Lead | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 (per 8/21 JST 拒绝兼任) |
| 3 | 平台 | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 |
| 4 | 评审 | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 |
| 5 | PM | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses — Mavis 接手 | 初版: DDD Review 阶段 Lead 真实身份采集流程 (4 步 + 1 时间表 + 5 角色签字) | F1-LeadRoster (33c38c1) 14 个 [DDD Review 阶段补] 空位 + Phase F.2+ 选项'全部' |
