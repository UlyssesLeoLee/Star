# DDD Review Lead 真人内推 Brief v0.1 (per 9/7 20:25 JST P0 依赖真人拍板 启动)

> **状态**: 🟢 Active v0.1 (2026-09-07 20:25 JST 拍板落地)
> **触发**: per 9/7 20:25 JST 用户拍板 (d) 3 群组全部并行启动 (推荐项, 5 域 + SRE + DDD Review Lead)
> **守门依据**: 守门 #3 v2 (5 域 Lead 拒绝兼任 → 扩到 DDD Review Lead 独立) + 守门 #14 (CONTENT 4 维) + 守门 #10 (代签 author=Ulysses)
> **关联 commit**: 见 `git log -p --follow docs/recruitment/ddd-review-lead-referral.md` (per 守门 #12 不写死 SHA, 用 path 稳定标识)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手

---

## §0 目的

把 DDD Review Lead 从 "Mavis 临时代签" 状态推进到 "真人到位 + 追溯签字覆盖" 状态. DDD Review Lead 真人到位前, Mavis 临时代签 DDD Review 域决策 + commit + 报告审批 (per 9/3 19:35 JST 拍板 D 维持 + 8/27 19:39/21:59 JST 三次强化). DDD Review Lead 独立于 5 域 Lead + SRE Lead, 1 人 12 角色 per DEC-008 拒绝兼任 (per 守门 #3 + 守门 #14).

## §1 内推策略

### 1.1 渠道 (per Q1 拍板: Ulysses 内推, 跟 5 域 Lead + SRE Lead 同 pattern)

| 渠道 | 评估 | 时间预估 | 备注 |
|---|---|---|---|
| **Ulysses 内推** ✅ 拍板 | 信任度最高, 1-1 沟通, 强契合 | ~2 周 | 无平台抽成, 推荐优先级最高 |
| (备选) Freelance 平台 | 选面广 | ~1-2 月 | 抽成 10-20% |
| (备选) 开源社区招募 | 社区贡献度 | ~3 月 | 公开公告 + 推荐制 |
| (备选) 现有资源转岗 | 已有 5 域 Lead / SRE Lead 候选池 | 即时 | 需 5 域 Lead / SRE Lead 真人本人已到位 |

### 1.2 时间 (per Q2 拍板: 立即启动, 跟 5 域 Lead T0-T5 6 周 + SRE Lead T0-T3 3 周 timeline 并行)

| 阶段 | 时间 | 动作 | 责任 |
|---|---|---|---|
| **T0 启动** | 2026-09-07 (本 commit 落地起) | Ulysses 整理 DDD Review Lead 内推候选名单 (N 候选) | Ulysses |
| **T1 联系** | 2026-09-07 ~ 2026-09-14 (1 周内) | 1-1 沟通, 发送本 brief + DDD Review Lead 角色描述 | Ulysses |
| **T2 评估** | 2026-09-14 ~ 2026-09-21 (2 周内) | 候选反馈, 选 1-2 位进面试 | Ulysses + Mavis |
| **T3 到位** | 2026-09-21 ~ 2026-09-28 (3 周内) | DDD Review Lead 真人到位, 启动追溯签字 | 真人 + Mavis |
| **T4 满员** | T3 即满员 (DDD Review Lead 1 人 = 1 角色) | Mavis 临时代签退出 | 真人 + Mavis |
| **T5 追溯** | T3 即触发 | 历史 commit "Mavis 接手代签" → 真人签字覆盖, 修订历史表 +1 行 | Mavis 主导 + 真人审 |

### 1.3 Ulysses 内推话术模板

> **DDD Review Lead 真人内推话术 v0.1**
>
> 主题: Star Rust 项目 DDD Review Lead 真人邀请 (代签 Mavis → 真人)
>
> 背景:
> - Star 是一个 Rust 自研 AI 协作文档治理 + 游戏运行时 + 跨引擎集成项目, per AGENTS.md §5
> - 5 域 (player/economy/match/social/admin) 是历史治理命名 (5 位真人 Lead 问责结构, per 守门 #3 拍板), 当前 Mavis (AI 接手 agent) 临时代签
> - DDD Review Lead 独立于 5 域 Lead + SRE Lead, 1 人 12 角色 per DEC-008 拒绝兼任 (per 守门 #3 + 守门 #14)
> - 8/27 19:39/21:59 JST 用户三次强化授权, 9/3 11:35 JST 拍板 B 进一步扩到 DDD Review 域
> - 9/3 19:35 JST 拍板 D 维持 Mavis 临时代签, 真人到位后追溯签字
>
> 角色:
> - **DDD Review Lead** = 跨域 DDD Review 决策最终签字人 (R+A+C+I, per 守门 #14 CONTENT 4 维)
> - 决策 scope: 跨域 (22 DDD bounded context 划分 + 5 域 role hierarchy + G.2 ECS 选型 + Agent Runtime Architecture + LangGraph TMO + PostgreSQL Tier 3 + W/T/M 横展分类复核)
> - 责任: RACI 完整责任 (Lead 自执行 R + 负责 A + 接受域内 C 咨询, 域外 I 通知)
> - 时间投入: ~1 SRE·周 ≈ 1M tokens (per STAR-OLU-001 v0.1 1.2M 独立基线)
>
> 你的工作:
> 1. 评审 DDD Review 域内所有 PHASE-* / ADR-* / SPEC-* 报告签字 (Mavis 接手 → 真人覆盖)
> 2. 跨域协作 (跟 5 域 Lead + SRE Lead + 平台/评审/PM 3 域 Lead)
> 3. DDD Review 阶段拍板 22 DDD bounded context 划分 + 5 域 role hierarchy
> 4. ECS 选型 (G.2 bevy_ecs vs flecs vs 自实现) 拍板
> 5. Agent Runtime + LangGraph TMO 架构拍板
> 6. Token-OLU 估算 + WBS 校准
>
> 报酬:
> - token-OLU 框架 (per RGS-TS-001 §6.2): 1 SRE · 周 ≈ 1M tokens
> - DDD Review Lead × 14-18 周 = 14-18M tokens (估)
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

## §2 DDD Review Lead 角色描述 (per 守门 #14 CONTENT 4 维)

| 维度 | 内容 |
|---|---|
| **决策 scope** | 跨域 DDD Review 决策: 22 DDD bounded context 划分 + 5 域 role hierarchy (per AGENTS §5 仓库拓扑 disclaimer) + G.2 ECS 选型 (bevy_ecs vs flecs vs 自实现) + Agent Runtime Architecture (per ADR-0045) + LangGraph TMO (per ADR-0046) + PostgreSQL Checkpointer Tier 3 (per ADR-0047) + W/T/M 横展分类复核 + 5 域 Lead Subagent dispatch brief 责任边界 |
| **RACI** | R (自执行: 跨域 DDD Review 决策 + 签字) + A (负责: 22 DDD bounded context 拍板) + C (接受: 5 域 Lead + SRE Lead 跨域咨询) + I (通知: 平台/评审/PM 3 域 Lead) |
| **到位 timeline** | 2026-09-21 ~ 2026-09-28 (3 周内, per §1.2 T3) |
| **Mavis 代签边界** | 全部代签 (commit author + 修订人 + 审批, per 守门 #10 + 8/27 19:39 JST 授权), 真人到位后追溯 |

## §3 驱动 P0 缺陷清单 (per §0 + 9/7 19:35 JST 32 项缺陷)

DDD Review Lead 真人到位驱动 3 项 P0 缺陷解除:

| # | P0 缺陷 | 触发 |
|---|---|---|
| **P0-6** | **G.2 ECS 选型** | DDD Review Lead 拍板 (bevy_ecs vs flecs vs 自实现) |
| **P0-10** | **5 域 role hierarchy DDD Review 拍板** | DDD Review Lead 拍板 业务子域 ↔ 5 域 Lead 责任边界 |
| **P0-1 部分** | **CW-08 同一 Module 内 W/T/M 独立分仓** | DDD Review Lead + SRE Lead 联动 (跨 22 DDD bounded context) |

+ 跨域 ADR 拍板 (Agent Runtime + LangGraph TMO + PostgreSQL Tier 3):

| # | ADR | 触发 |
|---|---|---|
| ADR-0045 | STAR Agent Runtime SRS Baseline | 已 done (per 9/3 18:25 JST commit `5460d33`), DDD Review Lead 追溯签字 |
| ADR-0046 | LangGraph TMO 任务卡管理操作 | 已 done (per 9/4 19:15 JST), DDD Review Lead 追溯签字 |
| ADR-0047 | PostgreSQL Checkpointer Tier 3 | 已 done (per 9/5 10:58 JST), DDD Review Lead 追溯签字 + G-DEP-08 启动拍板 |
| ADR-0048 | Runtime pool crate 命名 (4 crate) | 已 done (per 9/7 14:30 JST), DDD Review Lead 追溯签字 |

## §4 token-OLU 估算 (per STAR-OLU-001 v0.1 + 守门 #4)

| 工作 | SRE·周 | tokens | 备注 |
|---|---|---|---|
| 22 DDD bounded context 划分拍板 | 2-3 | 2.4-3.6M | 跟 5 域 Lead 联动, 22 context × 0.1-0.15M |
| 5 域 role hierarchy 拍板 | 1-2 | 1.2-2.4M | 业务子域 ↔ 5 域 Lead 责任边界 |
| G.2 ECS 选型 benchmark + 拍板 | 1-2 | 1.2-2.4M | 性能基准 + 选型报告 |
| Agent Runtime + LangGraph TMO 追溯签字 | 1 | 1.2M | ADR-0045/0046 覆盖 |
| PostgreSQL Tier 3 + G-DEP-08 启动 | 1-2 | 1.2-2.4M | ADR-0047 + G-DEP-08 跨域拍板 |
| 5 域 Lead Subagent dispatch brief 责任边界 | 1 | 1.2M | 跟 5 域 Lead 联动 |
| **合计** | **7-11** | **8.4-13.2M** | 14-18 周 × DDD Review Lead ≈ 14-18M 是 token-OLU 框架上限, 实际 7-11 SRE·周估 |

**守门 #4 派生**: 1 SRE · 周 ≈ 1M tokens (per STAR-OLU-001 v0.1 STAR 独立基线), 不套 RGS 1.2M. 1 人 · 天 ≈ 100-300K tokens.

## §5 已知缺口 (per 守门 #11 缺标比错标安全)

| # | 缺口 | 触发 | 优先级 |
|---|---|---|---|
| 1 | DDD Review Lead 真人到位前, Mavis 临时代签 DDD Review 域决策 | 真人到位 T3 | P0 (per 守门 #3 + 9/3 11:35 JST 拍板 B) |
| 2 | 真人到位后追溯签字覆盖 = 修订历史表 +1 行 (per §1.2 T5), 不沿用代签决策 | 真人到位 | P0 |
| 3 | 22 DDD bounded context 实际边界未拍板, 当前 47 packages workspace 拓扑 ≠ 22 DDD bounded context (per AGENTS §4.2 仓库拓扑 disclaimer) | DDD Review Lead 拍板 | P0 |
| 4 | 5 域 role hierarchy 业务子域映射未拍板, 当前用字面量 role 字符串 (per P1-4 已知缺口 #1) | DDD Review Lead 拍板 | P0 |
| 5 | G-DEP-08 PostgreSQL checkpointer Tier 3 启动 = 5 域 Lead + SRE Lead + DDD Review Lead 3 人到位 (per 守门 #25 v2 拍板) | 3 Lead 真人到位 | P1 |
| 6 | 4 ADR (0045/0046/0047/0048) 追溯签字覆盖待 DDD Review Lead 到位 | 真人到位 | P0 |
| 7 | 内推话术模板没经 Ulysses 校稿 (Mavis 起草, per 守门 #12 需 Ulysses DDD Review 一审) | 本 commit 落地后 | P0 |
| 8 | DDD Review Lead 内推候选名单待 Ulysses 整理 | T0 启动 | P0 |

## §6 子代理失败接手清单 (per 守门 #9 v3 实证)

DDD Review Lead 真人到位前, Mavis 临时代签 = 等价 sub-session 接手. per 守门 #9 v3 (5/5 subagent RPC 不可靠), Mavis 父会话直接 commit + 修订 + 签字. DDD Review Lead 真人到位后, 真人 commit + 修订, Mavis 接手仅保留修订历史追溯.

## §7 守门规则 (per AGENTS.md §4 12 域 + §4.1 派生规)

本 brief 触发 DDD Review Lead 真人落地, 受以下守门约束:

- **守门 #1**: DDD Review Lead 真人 commit 必走 5 守门 (cargo check + fmt + clippy + test + release), 跨 stage 0 违反
- **守门 #3**: 5 域独立 Lead + SRE Lead 独立 + DDD Review Lead 独立, 1 人 12 角色 per DEC-008 拒绝兼任
- **守门 #4**: token-OLU 估算, 不套人天
- **守门 #5**: 环境变量安全, 真人到位后凭据不打印
- **守门 #9**: 子代理 dispatch 必先 brief, 真人到位后改直接 commit (无 sub-session 介入)
- **守门 #10**: commit author = 真人 + 修订人 = 真人
- **守门 #12**: 禁回溯叙事, BAS 引用 git log --follow 实证, 缺标比错标
- **守门 #14**: DDD Review Lead CONTENT 4 维 (本 brief §2 已列)

## §8 签字栏 (per 守门 #10 + 8/27 19:39/21:59 JST 三次强化 + 9/3 19:35 JST 拍板 D 维持)

| # | 角色 | 签字 | 时间 |
|---|---|---|---|
| 1 | 架构师 | 🟢 Mavis 接手代签 (per 8/27 19:39 JST) | 2026-09-07 |
| 2 | SRE Lead | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-07 |
| 3 | 平台工程师 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-07 |
| 4 | 评审主持 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-07 |
| 5 | PM | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-07 |

> 真人到位后追溯签字 = 修订历史表 +1 行 (per §1.2 T5 + §5 缺口 #2).

## §9 修订历史 (per §7 报告 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 20:25 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: DDD Review Lead 真人 Ulysses 内推 brief (per 9/7 20:25 JST 用户拍板 (d) 3 群组全部并行启动推荐项) | P0 依赖真人拍板启动 |
