# 5 域 Lead 真人 Ulysses 内推 Brief v0.1 (per 9/5 10:43 JST G-DEP-03 拍板)

> **状态**: 🟢 Active v0.1 (2026-09-05 10:43 JST 拍板落地)
> **触发**: per 9/5 10:43 JST `ask_409cbd32edc309d71a083e2a` 用户拍板 (Q1=内推, Q2=立即启动, 推荐项)
> **守门依据**: 守门 #3 (5 域独立 Lead, 不接受兼任) + 守门 #14 (5 域 Lead CONTENT 4 维) + 守门 #10 (代签 author=Ulysses)
> **关联 commit**: 见 `git log -p --follow docs/recruitment/5-business-domain-lead-referral.md` (per 守门 #12 不写死 SHA, 用 path 稳定标识)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手

---

## §0 目的

把 5 域 (player / economy / match / social / admin) Lead 从 "Mavis 临时代签" 状态推进到 "真人到位 + 追溯签字覆盖" 状态. 5 域 Lead 真人到位前, Mavis 临时代签所有 5 域决策 + commit + 报告审批 (per 9/3 19:35 JST 拍板 D 维持 + 8/27 19:39/21:59 JST 三次强化).

## §1 内推策略

### 1.1 渠道 (per Q1 拍板: Ulysses 内推)

| 渠道 | 评估 | 时间预估 | 备注 |
|---|---|---|---|
| **Ulysses 内推** ✅ 拍板 | 信任度最高, 1-1 沟通, 强契合 | ~2 周 | 无平台抽成, 推荐优先级最高 |
| (备选) Freelance 平台 | 选面广 | ~1-2 月 | 抽成 10-20% |
| (备选) 开源社区招募 | 社区贡献度 | ~3 月 | 公开公告 + 推荐制 |
| 维持 Mavis 临时代签 | 0 启动 | 无限 | per 9/3 19:35 JST 拍板 D 维持 |

### 1.2 时间 (per Q2 拍板: 立即启动)

| 阶段 | 时间 | 动作 | 责任 |
|---|---|---|---|
| **T0 启动** | 2026-09-05 (本 commit 落地起) | Ulysses 整理 5 域内推候选名单 (5 域 × N 候选) | Ulysses |
| **T1 联系** | 2026-09-05 ~ 2026-09-12 (1 周内) | 1-1 沟通, 发送本 brief + 域 Lead 角色描述 | Ulysses |
| **T2 评估** | 2026-09-12 ~ 2026-09-19 (2 周内) | 候选反馈, 选 1-2 位进面试 | Ulysses + Mavis |
| **T3 到位** | 2026-09-19 ~ 2026-09-26 (3 周内) | 至少 1 域 Lead 真人到位, 启动追溯签字 | 真人 + Mavis |
| **T4 满员** | 2026-09-26 ~ 2026-10-17 (6 周内) | 5 域 Lead 全部到位, Mavis 临时代签退出 | 真人 + Mavis |
| **T5 追溯** | T3 / T4 各到位 1 人即触发 | 历史 commit "Mavis 接手代签" → 真人签字覆盖, 修订历史表 +1 行 | Mavis 主导 + 真人审 |

### 1.3 Ulysses 内推话术模板 (per 5 域各 1 份)

> **[域] Lead 真人内推话术 v0.1**
>
> 主题: Star Rust 项目 [域] Lead 真人邀请 (代签 Mavis → 真人)
>
> 背景:
> - Star 是一个 Rust 自研 [跨域] 项目 (AI 协作文档治理 + 游戏运行时 + 跨引擎集成), per AGENTS.md §5
> - 5 域 (player/economy/match/social/admin) 是历史治理命名 (5 位真人 Lead 问责结构, per 守门 #3 拍板), 当前 Mavis (AI 接手 agent) 临时代签
> - 8/27 19:39/21:59 JST 用户三次强化授权, 9/3 11:35 JST 拍板 B 进一步扩到跨域编排 + DDD Review
> - 9/3 19:35 JST 拍板 D 维持 Mavis 临时代签, 真人到位后追溯签字
>
> 角色:
> - **[域] Lead** = 该业务子域决策最终签字人 (R+A+C+I, per 守门 #14 5 域 Lead CONTENT 4 维)
> - 决策 scope: 域内 + 跨域 (Both, per 守门 #14)
> - 责任: RACI 完整责任 (Lead 自执行 R + 负责 A + 接受域内 C 咨询, 域外 I 通知)
> - 时间投入: ~1 SRE·周 ≈ 1M tokens (per STAR-OLU-001 v0.1 1.2M 独立基线)
>
> 你的工作:
> 1. 评审 [域] 域内所有 PHASE-* / ADR-* / SPEC-* 报告签字 (Mavis 接手 → 真人覆盖)
> 2. 跨域协作 (跟其他 4 域 Lead + SRE/平台/评审/PM 4 域 Lead, 1 人 12 角色 per DEC-008 拒绝兼任)
> 3. DDD Review 阶段拍板该域 DDD bounded context 划分
> 4. Token-OLU 估算 + WBS 校准
>
> 报酬:
> - token-OLU 框架 (per RGS-TS-001 §6.2): 1 SRE · 周 ≈ 1M tokens
> - 5 域 Lead × 14-18 周 = 80-120M tokens
> - 实际可按"决策次数"或"签字覆盖 commit 数"计费
>
> 启动:
> - 内推通过后, 我 (Ulysses) 直接拉你进 Star Lead 群 + 加 git 协作者
> - 第 1 周以"看历史 commit + 提问"为主, 不要求立即产出
> - 追溯签字覆盖 = 你的第 1 个具体动作, 不接受"先签 1 份试试"敷衍
>
> 期待回复: 1 周内 yes/no + 排期 30 分钟 1-1 沟通
>
> — Ulysses

## §2 5 域 Lead 角色描述 (per 守门 #14 5 域 Lead CONTENT 4 维)

### 2.1 player 域 Lead (玩家子域)

| 维度 | 内容 |
|---|---|
| **决策 scope** | 玩家账号/角色/存档/登录状态/租赁 (per spec/services/01-03 + spec/agents/01 §2 Lease) |
| **RACI** | R (自执行: 域内决策 + 签字) + A (负责: 域内 DDD Review 拍板) + C (接受: 域内咨询) + I (通知: 跨域 I) |
| **到位 timeline** | 2026-09-19 ~ 2026-09-26 (3 周内, per §1.2 T3) |
| **Mavis 代签边界** | 全部代签 (commit author + 修订人 + 审批, per 守门 #10 + 8/27 19:39 JST 授权), 真人到位后追溯 |

### 2.2 economy 域 Lead (经济子域)

| 维度 | 内容 |
|---|---|
| **决策 scope** | 经济/库存/商店/订单/跨域 Saga 协调 (per Q-003 跨域核心问题) |
| **RACI** | 同 §2.1 |
| **到位 timeline** | 同 §1.2 T3 |
| **Mavis 代签边界** | 同 §2.1 |

### 2.3 match 域 Lead (匹配/对战子域)

| 维度 | 内容 |
|---|---|
| **决策 scope** | 匹配/对战/战斗逻辑/战术/战报同步 |
| **RACI** | 同 §2.1 |
| **到位 timeline** | 同 §1.2 T3 |
| **Mavis 代签边界** | 同 §2.1 |

### 2.4 social 域 Lead (社交子域)

| 维度 | 内容 |
|---|---|
| **决策 scope** | 聊天/好友/公会/排行榜/通知 |
| **RACI** | 同 §2.1 |
| **到位 timeline** | 同 §1.2 T3 |
| **Mavis 代签边界** | 同 §2.1 |

### 2.5 admin 域 Lead (管理子域)

| 维度 | 内容 |
|---|---|
| **决策 scope** | COC 控制面/审计/合规/客服/RBAC (per spec/services/07-audit-model) |
| **RACI** | 同 §2.1 |
| **到位 timeline** | 同 §1.2 T3 |
| **Mavis 代签边界** | 同 §2.1 |

## §3 token-OLU 估算 (per STAR-OLU-001 v0.1 + 守门 #4)

| 域 | 工作量 (SRE·周) | tokens | 备注 |
|---|---|---|---|
| player | 2-3 | 2.4-3.6M | 域内决策 + 追溯签字覆盖 |
| economy | 3-4 | 3.6-4.8M | Q-003 Saga 跨域核心问题 1-2 周额外 |
| match | 2-3 | 2.4-3.6M | 域内决策 + 战报同步 |
| social | 2 | 2.4M | 域内决策 |
| admin | 2-3 | 2.4-3.6M | COC + 审计 + RBAC |
| **合计** | **11-15** | **13.2-18M** | 14-18 周 × 5 域 ≈ 80-120M 是 token-OLU 框架上限, 实际 11-15 SRE·周估 |

**守门 #4 派生**: 1 SRE · 周 ≈ 1M tokens (per STAR-OLU-001 v0.1 STAR 独立基线), 不套 RGS 1.2M. 1 人 · 天 ≈ 100-300K tokens.

## §4 已知缺口 (per 守门 #11 缺标比错标安全)

| # | 缺口 | 触发 | 优先级 |
|---|---|---|---|
| 1 | 5 域 Lead 真人到位前, Mavis 临时代签所有 5 域决策 | 真人到位 T3/T4 | P0 (per 守门 #3 + 9/3 11:35 JST 拍板 B) |
| 2 | 真人到位后追溯签字覆盖 = 修订历史表 +1 行 (per §1.2 T5), 不沿用代签决策 | 真人到位 | P0 |
| 3 | 5 域 Lead Subagent dispatch 模板 (`docs/briefs/5-leads/{domain}.md`) 跟真人 Lead 责任边界不清, 待 DDD Review 拍板 | 真人到位 + DDD Review | P1 |
| 4 | G-DEP-09 PostgreSQL checkpointer Tier 3 启动时间 = 真人到位后 (T3 至少 1 人到位) | 真人到位 | P1 |
| 5 | 5 域 Lead 真人退出机制 (per "拒绝兼任"硬约束) 待 DDD Review 拍板 | 长期 | P2 |
| 6 | 内推话术模板没经 Ulysses 校稿 (Mavis 起草, per 守门 #12 需 Ulysses DDD Review 一审) | 本 commit 落地后 | P0 |

## §5 子代理失败接手清单 (per 守门 #9 v3 实证)

5 域 Lead 真人到位前, Mavis 临时代签 = 等价 sub-session 接手. per 守门 #9 v3 (5/5 subagent RPC 不可靠), Mavis 父会话直接 commit + 修订 + 签字. 5 域 Lead 真人到位后, 真人 commit + 修订, Mavis 接手仅保留修订历史追溯.

## §6 守门规则 (per AGENTS.md §4 12 域 + §4.1 派生规)

本 brief 触发 5 域 Lead 真人落地, 受以下守门约束:

- **守门 #1**: 5 域 Lead 真人 commit 必走 5 守门 (cargo check + fmt + clippy + test + release), 跨 stage 0 违反
- **守门 #3**: 5 域独立 Lead, 不接受兼任 (per 8/21 JST 拍板)
- **守门 #4**: token-OLU 估算, 不套人天
- **守门 #5**: 环境变量安全, 真人到位后凭据不打印
- **守门 #9**: 子代理 dispatch 必先 brief, 真人到位后改直接 commit (无 sub-session 介入)
- **守门 #10**: commit author = 真人 + 修订人 = 真人
- **守门 #12**: 禁回溯叙事, BAS 引用 git log --follow 实证, 缺标比错标
- **守门 #14**: 5 域 Lead CONTENT 4 维 (本 brief §2 已列)

## §7 签字栏 (per 守门 #10 + 8/27 19:39/21:59 JST 三次强化 + 9/3 19:35 JST 拍板 D 维持)

| # | 角色 | 5 域签字 | 时间 |
|---|---|---|---|
| 1 | 架构师 | 🟢 Mavis 接手代签 (per 8/27 19:39 JST) | 2026-09-05 |
| 2 | SRE Lead | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-05 |
| 3 | 平台工程师 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-05 |
| 4 | 评审主持 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-05 |
| 5 | PM | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-05 |

> 真人到位后追溯签字 = 修订历史表 +1 行 (per §1.2 T5 + §4 缺口 #2).

## §8 修订历史 (per §7 报告 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 域 Lead 真人 Ulysses 内推 brief (per 9/5 10:43 JST `ask_409cbd32edc309d71a083e2a` 用户拍板 Q1=内推+Q2=立即启动) | G-DEP-08 跨 session 续落地 |
