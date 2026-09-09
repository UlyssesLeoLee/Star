# Admin 域 Lead 真人 Ulysses 内推 Brief v0.1 (per 9/10 07:24 JST 拍板 v0.54)

> **状态**: 🟡 Active v0.1 (2026-09-10 07:24 JST brief v0.54 派发)
> **触发**: per docs/briefs/v0.54-5lead-outreach.md §3.1 + docs/recruitment/5-business-domain-lead-referral.md v0.1 §2.5
> **守门依据**: 守门 #3 (5 域独立 Lead, 不接受兼任) + 守门 #14 v2 (5 域 Lead CONTENT 4 维) + 守门 #10 (代签 author=Ulysses)
> **关联**: 父文档 docs/recruitment/5-business-domain-lead-referral.md v0.1 §2.5 + spec/services/07-audit-model + ADR-0030 Lease + Heartbeat
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)

---

## §0 目的

为 admin 域 DDD bounded context 招募真人 Lead, 替换 Mavis 临时代签. 真人到位后追溯签字覆盖, 修订历史表 +1 行 (per 守门 #14 v2 §4 缺口 #2 + 9/5 10:43 JST 拍板 D).

## §1 admin 域核心职责 (per docs/recruitment/5-business-domain-lead-referral.md v0.1 §2.5 + spec/services/07-audit-model)

| 决策 scope | 内容 |
|---|---|
| COC 控制面 | 跨域协调控制台 / 紧急停服 / 全服公告 / 跨域事件总线 / Saga 监控 (per spec/services/07) |
| 审计 | 5 域操作审计 / 经济流水审计 / 战斗日志审计 / 登录审计 / 合规审计 (per spec/services/07-audit-model) |
| 合规 | GDPR / CCPA / 网络安全法 / 未成年人保护 / 数据本地化 / 实名认证 (per spec/services/07) |
| 客服 | 工单系统 / 自动回复 / 升级路径 / SLA / 客诉处理 (per spec/services/07) |
| RBAC | 角色定义 / 权限矩阵 / 5 域 RBAC / 跨域权限 / 权限继承 / 临时授权 (per spec/services/07 + v0.49 §14.12 IV 5 域 RBAC) |

**RACI**: R (自执行: 域内决策 + RBAC 跨域协调) + A (负责: 域内 DDD Review 拍板 + 审计/合规拍板) + C (接受: 域内咨询) + I (通知: 跨域 I, 4 域均需)

**Mavis 代签边界**: 全部代签 (commit author + 修订人 + 审批, per 守门 #10 + 8/27 19:39 JST 授权), 真人到位后追溯签字覆盖

## §2 Ulysses 内推话术模板 (admin 域特化)

> **主题**: Star Rust 项目 Admin 域 Lead 真人邀请 (代签 Mavis → 真人)
>
> **背景**:
> - Star 是一个 Rust 自研 AI 协作 + 游戏运行时 + 跨引擎集成项目 (per AGENTS.md §5 + docs/architecture/domain-local-runtime.md)
> - Admin 域是 5 域 DDD bounded context 第 5 域, 决定 COC 控制面/审计/合规/客服/RBAC 5 类业务实体的最终签字权
> - Admin 域是 5 域中**跨域覆盖最广**的域: RBAC 跨 5 域 + 审计跨 5 域 + 紧急停服跨 5 域
> - 当前 Mavis (AI 接手 agent per DEC-008) 临时代签, 9/3 11:35 JST 拍板 B 进一步扩到跨域编排 + DDD Review, 9/3 19:35 JST 拍板 D 维持 Mavis 临时代签
>
> **角色**: Admin Lead = 该业务子域决策最终签字人 (R+A+C+I, per 守门 #14 5 域 Lead CONTENT 4 维)
> **决策 scope**: 域内 + 跨域 (Both, per 守门 #14); 跨域协调**全部 4 域**: player + economy + match + social (RBAC/审计/合规 跨域必覆盖)
>
> **你的工作**:
> 1. 评审 admin 域内所有 PHASE-* / ADR-* / SPEC-* 报告签字 (Mavis 接手 → 真人覆盖)
> 2. 跨域协作 (跟其他 4 域 Lead + SRE/平台/评审/PM 4 域 Lead, 1 人 12 角色 per DEC-008 拒绝兼任)
> 3. DDD Review 阶段拍板 admin 域 DDD bounded context 划分 + 5 域 RBAC 终版 + 审计/合规拍板
> 4. Token-OLU 估算 + WBS 校准 (admin 域估 2-3 SRE·周 ≈ 2.4-3.6M tokens, per STAR-OLU-001 v0.1)
>
> **跟 ADR-0030 Lease + Heartbeat 关系** (per docs/architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md):
> - Admin 域是 Lease/Heartbeat 的**审计 + RBAC 监督方**: 所有 Agent 操作的 Lease 持有 + 心跳 都需 admin 域审计记录
> - 紧急停服走 Agent Lost 后 5 步 (per ADR-0030 §2.2): 保存 Workspace → Worktree → Context Snapshot → 释放 Task Lease → 允许其他 Agent Resume (admin 域强制触发)
> - Resume payload 11 字段 (per ADR-0030 §2.3) 中 `failed_attempts` + `open_diagnostics` 字段是 admin 域审计核心 (Agent 失败原因)
> - Admin Lead 拍板: 5 域 RBAC 终版矩阵 (per v0.49 §14.12 IV 实证) + 审计日志保留期 (per 守门 #13 W/T/M 横展, T 类 永久 / 5 年) + 合规审计触发条件
>
> **报酬**: token-OLU 框架 (per RGS-TS-001 §6.2): 1 SRE · 周 ≈ 1M tokens; admin 域 × 14-18 周 = ~2.4-3.6M tokens
>
> **启动**: 内推通过后, Ulysses 直接拉你进 Star Lead 群 + 加 git 协作者; 第 1 周以"看历史 commit + 提问"为主, 不要求立即产出
>
> **期待回复**: 1 周内 yes/no + 排期 30 分钟 1-1 沟通
>
> — Ulysses

## §3 3 候选名单 (per docs/recruitment/5-business-domain-lead-referral.md §1 候选 1/2/3)

| 候选 | 渠道 | 评估 | 时间预估 | 信任度 | 备注 |
|---|---|---|---|---|---|
| 候选 1 | Ulysses 内推 (per §1.1 拍板) | 信任度最高, 1-1 沟通, 强契合 | ~2 周 | 🟢 高 | 无平台抽成, 推荐优先级最高; 候选画像: 有后台管理系统 5+ 年经验, 熟悉 RBAC + 审计 + 合规 (GDPR/CCPA) |
| 候选 2 | Freelance 平台 (Toptal / Upwork) | 选面广, 可看历史评价 | ~1-2 月 | 🟡 中 | 抽成 10-20%; 兜底渠道, 候选 1 不可达时启动 |
| 候选 3 | 开源社区 (GitHub / Rust 社区) | 社区贡献度可见 | ~3 月 | 🟡 中 | 公开公告 + 推荐制; 兜底渠道, 候选 1+2 不可达时启动 |

## §4 5 域职责 (per 守门 #14 5 域 Lead CONTENT 4 维 + 守门 #3 5 域独立 Lead)

| 维度 | admin 域特化内容 |
|---|---|
| **决策 scope** | COC 控制面/审计/合规/客服/RBAC 5 类业务实体的最终签字 (per §1) |
| **RACI** | R + A (负责: 域内 DDD Review + 5 域 RBAC 终版 + 审计/合规拍板) + C + I (通知 4 域, 全覆盖) |
| **到位 timeline** | T3 (2026-09-19 ~ 2026-09-26, 3 周内, per docs/recruitment/5-business-domain-lead-referral.md v0.1 §1.2 T3) |
| **Mavis 代签边界** | 全部代签, 真人到位后追溯 (per 守门 #10 + 8/27 19:39/21:59 JST 授权) |

## §5 已知缺口 (per 守门 #11 缺标比错标安全)

| # | 缺口 | 触发 | 优先级 |
|---|---|---|---|
| 1 | admin Lead 真人到位前, Mavis 临时代签所有 admin 域决策 + 5 域 RBAC 跨域 | 真人到位 T3 | P0 |
| 2 | 真人到位后追溯签字覆盖 = 修订历史表 +1 行, 不沿用代签决策 | 真人到位 | P0 |
| 3 | admin Lead Subagent dispatch 模板 (`docs/briefs/5-leads/admin.md`) 跟真人 Lead 责任边界不清, 待 DDD Review 拍板 | 真人到位 + DDD Review | P1 |
| 4 | 5 域 RBAC 终版矩阵 (per v0.49 §14.12 IV 实证基础) 待 admin Lead 真人到位后决策 | 真人到位 + DDD Review | P0 |
| 5 | admin 域 ADR-0030 审计日志保留期 + 紧急停服触发条件 + 5 域 RBAC 监督策略 待 DDD Review 拍板 | DDD Review | P1 |
| 6 | 内推话术模板没经 Ulysses 校稿 (Mavis 起草, per 守门 #12 需 Ulysses DDD Review 一审) | 本 commit 落地后 | P0 |
| 7 | 候选 1 联系方式 Ulysses 私有持, 暂不入档 (per 守门 #5 env 安全) | 持续 | P0 |

## §6 守门规则 (per AGENTS.md §4 12 域 + §4.1 派生规)

- **守门 #1**: admin Lead 真人 commit 必走 5 守门 (cargo check + fmt + clippy + test + release)
- **守门 #3**: admin 域独立 Lead, 不接受兼任 (per 8/21 JST 拍板 + 9/3 11:35 JST 拍板 B 反转)
- **守门 #4**: token-OLU 估算, 不套人天 (2-3 SRE·周 ≈ 2.4-3.6M tokens, per STAR-OLU-001 v0.1)
- **守门 #5**: 环境变量安全, 真人到位后凭据不打印
- **守门 #9**: 子代理 dispatch 必先 brief, 真人到位后改直接 commit (无 sub-session 介入)
- **守门 #10**: commit author = 真人 + 修订人 = 真人
- **守门 #12**: 禁回溯叙事, BAS 引用 git log --follow 实证, 缺标比错标
- **守门 #13**: W/T/M 横展 (T 类审计永久保留, M 类 RBAC 慢变 SCD, per docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md v0.1)
- **守门 #14**: 5 域 Lead CONTENT 4 维 (本 brief §4 已列)
- **守门 #14 v3 永久代签**: Mavis 临时代签 admin 域决策维持, 真人到位后追溯

## §7 签字栏 (per 守门 #10 + 8/27 19:39/21:59 JST 三次强化 + 9/3 19:35 JST 拍板 D 维持)

| # | 角色 | admin 域签字 | 时间 |
|---|---|---|---|
| 1 | 架构师 | 🟢 Mavis 接手代签 (per 8/27 19:39 JST) | 2026-09-10 |
| 2 | SRE Lead | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-10 |
| 3 | 平台工程师 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-10 |
| 4 | 评审主持 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-10 |
| 5 | PM | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生) | 2026-09-10 |

> 真人到位后追溯签字 = 修订历史表 +1 行 (per docs/recruitment/5-business-domain-lead-referral.md v0.1 §1.2 T5 + §4 缺口 #2).

## §8 修订历史 (per §7 报告 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: admin 域 Lead 真人 Ulysses 内推 brief (per docs/briefs/v0.54-5lead-outreach.md §3.1 + docs/recruitment/5-business-domain-lead-referral.md v0.1 §2.5 + spec/services/07-audit-model + v0.49 §14.12 IV 5 域 RBAC + ADR-0030 Lease + Heartbeat 关系) | G-DEP-08 跨 session 续落地 + 9/10 07:24 JST 用户发令"继续" |

---

**per 守门 #14 v3 Mavis 永久代签**: 修订人 = Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手, 审批 = 架构师 (Mavis 接手 agent per DEC-008), 真人到位后修订历史 +1 行追溯签字覆盖.
