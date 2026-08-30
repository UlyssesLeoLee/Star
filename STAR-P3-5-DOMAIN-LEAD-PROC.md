# STAR-P3-5-DOMAIN-LEAD-PROC 5 域 Lead 真人到位 流程 (per 8/21 JST 拒绝兼任硬约束)

> **Status**: 🟡 Draft (P3 全 5 阶段 60/65 拍板完成, 5 域 Lead 真人到位是跨阶段硬约束)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008)
> **承接**: STAR-P3-WBS-001 §7 阻塞 #7 (5 域 Lead 真人到位) + §6 E.5 / F.1 / C.9 / DDD Review 阶段

---

## §0 背景

P3 全 5 阶段 60/65 拍板完成 (per P3-C-D-SELECTION-RESULT.md + P3-E-F-SELECTION-RESULT.md), 11 wt 并行实装需跨 session 续做. **5 域 Lead 真人到位** 是跨阶段硬约束, 阻塞:
- C.9 5 域 Lead 真人到位 (P3-C 拍板 9/9 子项, 跨 session 续)
- E.5 5 域 Lead 真人到位 (P3-E 拍板 7/7 子项, 跨 session 续)
- F.1 5 域 Lead 真人到位 (P3-F 拍板 5/6 子项, 跨 session 续)
- DDD Review 阶段: 5 域 Lead 真人签字 (守门 #3 / #5 角色)

**硬约束 (per 8/21 JST Ulysses 拍板)**: 不接受兼任 — 架构师不能兼任 player Lead, SRE 不能兼任 admin Lead. 5 个真人 = 5 个独立个体.

---

## §1 5 域 Lead 角色定义 (per RGS 5 域镜像)

| # | 域 | 角色职责 | 守门 | 不接受兼任 |
|---|---|---|---|---|
| 1 | **player** (玩家域) | 用户/identity/workspace 业务边界, 跟 RGS 5 域 player 镜像 | 守门 #1+#9+#12 | 不能是架构师 (避免架构师"自己审自己") |
| 2 | **economy** (经济域) | billing / pricing / cost-trace 业务边界 (per RGS) | 守门 #1+#9+#12 | 不能是 SRE (避免 SRE 既写又审) |
| 3 | **match** (匹配域) | workflow / 状态机 / saga 编排 业务边界 (per RGS) | 守门 #1+#9+#12 | 不能是 reviewer (避免 reviewer 既写又审) |
| 4 | **social** (社交域) | collaboration / 通知 / 评论 业务边界 (per RGS) | 守门 #1+#9+#12 | 不能是 PM (避免 PM 既写又审) |
| 5 | **admin** (管理域) | RBAC / permission / tenant 业务边界 (per RGS) | 守门 #1+#9+#12 | 不能是 SRE (避免 SRE 既写又审, 重复 #2) |

---

## §2 真人到位流程 (5 步)

### 步骤 1: Ulysses 找 5 个真人 (跨 session)

- **方法 A (推荐)**: Ulysses 个人网络 / 公司内部 5 个工程师, 每人认领 1 域, 签署 DDD Review 协议
- **方法 B (备选)**: 通过 freelance 平台 (e.g. Toptal / Upwork) 找 5 个 Rust 工程师, 每域 1 个 Lead
- **方法 C (备选)**: 开源社区招募, 5 域 Lead 公开认领

### 步骤 2: 5 域 Lead 注册 (1 commit 落档)

文件: `STAR-P3-5-DOMAIN-LEAD-REGISTRY.md` 7 段结构

```markdown
| 域 | Lead 姓名 | 邮箱 | 角色 | 域边界 docs | 状态 |
|---|---|---|---|---|---|
| player | <name> | <email> | Player Lead | docs/architecture/player.md | 🟢 到岗 |
| economy | <name> | <email> | Economy Lead | docs/architecture/economy.md | 🟢 到岗 |
| match | <name> | <email> | Match Lead | docs/architecture/match.md | 🟢 到岗 |
| social | <name> | <email> | Social Lead | docs/architecture/social.md | 🟢 到岗 |
| admin | <name> | <email> | Admin Lead | docs/architecture/admin.md | 🟢 到岗 |
```

### 步骤 3: 5 域 Lead 域边界 docs 落地 (5 commit 落档)

每域 1 份 域边界 docs (per ADR-0031 Context Graph MVP 4 节点 / Phase 2 12+10 节点/关系):

**P3-E.7 5 域 docs 阶段 已实装** (per `e67bc8c` commit, 2026-08-30 08:55 JST):
- `docs/ddd/01-player-bc.md` (Player 域 BoundedContext + 3 Aggregate: User / Workspace / Device + 7 pub + 3 sub 跨域事件)
- `docs/ddd/02-economy-bc.md` (Economy 域 BoundedContext + 4 Aggregate: BillingAccount / Subscription / Invoice / ApiKey + 7 pub + 4 sub)
- `docs/ddd/03-match-bc.md` (Match 域 BoundedContext + 3 Aggregate: Workflow / WorkflowInstance / SagaInstance + 7 pub + 5 sub)
- `docs/ddd/04-social-bc.md` (Social 域 BoundedContext + 3 Aggregate: Notification / Comment / Mention + 7 pub + 12 sub)
- `docs/ddd/05-admin-bc.md` (Admin 域 BoundedContext + 4 Aggregate: Tenant / Permission / Role / KmsKey + 8 pub + 8 sub)

> **真人到位前过渡**: 5 份 docs 已由架构师代签 (per `ec6dee0` 选项 4 应急). 真人到位后, 签字栏 #1 追溯签字覆盖应急代签.
>
> **5 域 Lead review 模板**: 真人到位后, 真人先 review 5 份 docs (§1-§5 章节), 然后:
> - **§1 BoundedContext 定义**: 确认业务子域 + Aggregate Root 划分合理
> - **§2 Aggregate 字段 + 命令 + 事件**: 字段类型 / 索引 / 约束 (per docs/ddd/0X-*.md §5 已知缺口)
> - **§3 跨域事件**: pub / sub 事件 schema + at-least-once / exactly-once 投递
> - **§4 Cargo crate 引用**: 跨域 crate 散落 (per §4 注解), 是否需独立 crate phase 2
> - **§6 签字栏 #1**: 真人签字覆盖架构师代签
> - **§7 修订历史**: 真人 + 修订日期 + 修订内容
>
> **审阅时间预算**: 每域真人 review 30-60 分钟 (BoundedContext + Aggregate + 跨域事件 3 章节), 5 域 = 2.5-5 小时 (per STAR-OLU-001 §6 质量门 5 维).

### 步骤 4: 5 域 Lead DDD Review 签字 (守门 #3+#5)

每子项 5 域 Lead 签字, 守门 #3 (评审) + #5 (PM) + #1 (架构负责人) 5 角色:

```markdown
| 子项 | 架构 | SRE | 平台 | 评审 | PM | DDD 5 域 Lead |
|---|---|---|---|---|---|---|
| B.1 OpenClaw HTTP | ✅ | ✅ | ✅ | ✅ | ✅ | player / economy / match / social / admin |
```

### 步骤 5: 跨 session 续做 + 真人到位验收

5 域 Lead 到岗后, 11 wt 并行实装可以接入:
- 5 域业务子域 (P3-C.1-C.5) 推进时, 真人签字 docs
- 跨域 Saga (P3-C.6) 真人主持跨域 review
- 跨域 E2E (P3-F.2) 真人跑测试验收

**真人到位验收 checklist** (per STAR-OLU-001 §6 质量门 5 维):
1. [ ] 5 域 Lead 真人到位, 每人 1 域 (per §1 拒绝兼任硬约束)
2. [ ] 5 域 Lead review 5 份 DDD 边界 docs (per §3 已实装), 签字栏 #1 追溯签字
3. [ ] 5 域 Lead review 6 份 P3 阶段收官报告 (PHASE-P3-C1-IMPL-REPORT.md + PHASE-P3-C2-C5-IMPL-REPORT.md + PHASE-P3-C6-C8-IMPL-REPORT.md + PHASE-P3-D1-D7-IMPL-REPORT.md + PHASE-P3-E1-E4-IMPL-REPORT.md + PHASE-P3-F1-F5-IMPL-REPORT.md), 签字栏 #1 追溯签字覆盖架构师代签
4. [ ] 5 域 Lead review PHASE-P3-CROSS-STAGE-INC-SESSION-004.md, 签字栏 5 角色全部追溯签字
5. [ ] E.6 Saga 跨域编排 (per §3 match 域 Lead 真人补详细补偿机制) phase 2 启动
6. [ ] DDD Review 阶段 (per F.5 P3-quality-gate-5d.md §6 DDD Review) 5 域 Lead 真人 + SRE Lead + 平台 + 评审 + PM 5 角色真人到位 (per AGENTS §3 模板)
7. [ ] 质量门 5/5 (per STAR-OLU-001 §6) 实证: 功能完整 / 测试覆盖 / 守门 0 违反 / 文档同步 / git 证据

**真人到位后 5 域边界 docs 收尾**:
- 5 域 Lead 真人签字栏 #1 追溯签字 (覆盖架构师代签)
- 5 域 Lead 补 §5 已知缺口 (字段类型 / 索引 / 约束 / ABAC conditions / 跨域事件总线架构等)
- 5 域 Lead 拍板 §4 Cargo crate 引用 (是否需独立 domain-X crate phase 2 拆分)

---

## §3 拍板选项 (Ulysses 一键决定)

### 选项 1: 推荐 5 步流程 (个人网络 / 公司内部, 跨 session 续)

- 推荐方法 A: Ulysses 找 5 个工程师, 每人认领 1 域
- 真人到位跨 session 续, 不阻塞 P3-C/D/E/F 11 wt 并行实装 (5 域子域可以先以架构师代签, 真人到位后追溯签字)

### 选项 2: 备选 freelance 平台 (Toptal / Upwork)

- 找 5 个 Rust 工程师, 每域 1 个 Lead
- 风险: freelance 质量不可控, DDD Review 可能不深入

### 选项 3: 备选 开源社区招募

- 5 域 Lead 公开认领
- 风险: 招募周期长, 真人到位慢

### 选项 4: 暂时跳过 5 域 Lead 真人, 用架构师代签 (per 8/27 19:39 JST 用户授权)

- 架构师代签所有 5 域 Lead 签字 (per Mavis 接手代签流程)
- 风险: 违反 8/21 JST 拒绝兼任硬约束, DDD Review 质量降级
- 选项 4 仅作为应急, 不推荐

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- 5 域 Lead 真人到位 = Ulysses 主动找真人, agent 不能代找 (per 1 人公司 + AI 协作 token-OLU 模式)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 5 域 Lead 真人到位是 P3 阶段跨阶段硬约束, 阻塞 P3-C/E/F 跨 session 续 | 8/21 JST 拒绝兼任硬约束 |
| 2 | 真人到位 = Ulysses 主动找人, agent 不能代找 | 1 人公司 + AI 协作模式 |
| 3 | DDD Review 阶段 5 域真人签字不可省 (per AGENTS §3 模板) | AGENTS §3 |
| 4 | 暂跳过 5 域 Lead 真人 = 应急选项 4, 不推荐 (违反硬约束) | 选项 4 仅应急 |
| 5 | 守门 #12 commit-time 同步 (本文件 commit 即触发, 后续 docs 同步接 v15 派生饱和) | AGENTS §4.1 v15 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft; 5 域 Lead 真人到位 5 步流程 + 4 拍板选项 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 域 Lead 真人到位 5 步流程 + 4 拍板选项 + 5 域角色定义 (player / economy / match / social / admin) | 2026-08-30 P3 全 5 阶段 60/65 拍板完成, 5 域 Lead 真人到位是跨阶段硬约束 |
| v0.2 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | §3 步骤 3 增强: 5 域 DDD 边界 docs 落地 (per `e67bc8c` commit) + 真人到位前过渡 + review 模板 + 审阅时间预算; §4 步骤 5 增强: 真人到位验收 checklist (7 项 per STAR-OLU-001 §6 质量门 5 维) + 5 域边界 docs 收尾 3 项 | 2026-08-30 09:01 JST P3-E.7 5 域 DDD 边界 docs 落地 (commit `e67bc8c`) 是新事件, 触发流程草案与 docs 同步, 守门 #12 commit-time 同步 |
