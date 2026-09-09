# ARG-ARCH-001 Agent Relationship Graph (ARG) RACI 责任矩阵

> **Status**: 🟡 Draft v0.1 (per 2026-09-09 21:18 JST 派发 brief v0.45 §14.11 ARG.1 文档完整化)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md)
> **承接**: 守门 #14 v2 拍板 D (per 2026-09-03 19:43 JST) + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 (Mavis 永久代签)

---

## 0. 目的 (Objective)

定义 ARG 阶段 **5 域 (player / economy / match / social / admin) × 9 SA (SA-01..SA-09) × 5 module (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime)** 的 RACI 责任矩阵, 跨 5 文档严格一致.

**RACI 定义** (per 守门 #14 v2 拍板 v3 RACI 责任):
- **R** (Responsible) = 负责执行 (Mavis 接手 / 真人 Lead 自执行)
- **A** (Accountable) = 负责问责 (Mavis 接手 / 真人 Lead 问责)
- **C** (Consulted) = 接受咨询 (域内)
- **I** (Informed) = 通知 (域外)

**5 域 Lead 真人到位前 Mavis 临时代签** (per 9/3 11:35 JST 拍板 B + 9/9 12:02 JST 守门 #14 v3 升级), 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).

---

## 1. 5 域 RACI 责任 (横向: 5 域 Lead × 5 module)

### 1.1 5 域 Lead × 5 module RACI 总表

| 5 域 Lead | ArgCrate | SubAgentOrchestrator | LLMService | AgentLease | AgentRuntime |
|---|---|---|---|---|---|
| **player Lead** (per 守门 #3 v2) | R+A | C (player 业务) | C (player prompt) | I | R+A |
| **economy Lead** (per 守门 #3 v2) | R+A | C (economy 业务) | C (economy prompt) | I | R+A |
| **match Lead** (per 守门 #3 v2) | R+A | R+A (Dispatch 路由主战场) | C (match prompt) | I | R+A |
| **social Lead** (per 守门 #3 v2) | R+A | R+A (Context 注入主战场) | C (social prompt) | I | R+A |
| **admin Lead** (per 守门 #3 v2) | R+A | R+A (Trust + Output 评估主战场) | R+A (审计摘要) | R+A (audit trail) | R+A (admin UI) |
| **跨域协调 (Mavis 接手)** | I | I (5 域 Lead 决策) | I | I | I |

> **关键派生**:
> - **player + economy Lead**: ArgCrate + AgentRuntime 主 R+A (业务 CRUD 主战场), SubAgentOrchestrator + LLMService 是 C (业务咨询)
> - **match Lead**: Dispatch 路由是 match 主战场 (per §3.1 4 effect 维度 #1), 所以 R+A
> - **social Lead**: Context 注入是 social 主战场 (per §3.2 4 effect 维度 #2), 所以 R+A
> - **admin Lead**: Trust + Output 评估 + 审计是 admin 主战场 (per §3.3-§3.4 4 effect 维度 #3-#4), 所以全部 R+A
> - **跨域协调**: Mavis 接手 (per 守门 #3 v2), 真人到位后追溯签字

### 1.2 5 域 Lead 跨域 RACI 约束 (per 守门 #3 v2 跨域 consults 而非 delegates_to)

| 5 域 Lead → 5 域 Lead | 跨域关系类型 | RACI | 跨域约束 |
|---|---|---|---|
| player → economy | consults (e.g. 玩家代币改动) | C + C | 跨域咨询, 不允许 delegates_to |
| player → match | consults (e.g. 玩家匹配) | C + C | 跨域咨询 |
| player → social | consults (e.g. 玩家好友) | C + C | 跨域咨询 |
| player → admin | consults (e.g. 玩家审计) | C + I | admin 是 I (审计通知) |
| economy → match | consults (e.g. 比赛奖励) | C + C | 跨域咨询 |
| economy → social | consults (e.g. 社交交易) | C + C | 跨域咨询 |
| economy → admin | consults (e.g. 经济审计) | C + I | admin 是 I |
| match → social | consults (e.g. 赛季好友) | C + C | 跨域咨询 |
| match → admin | consults (e.g. 比赛审计) | C + I | admin 是 I |
| social → admin | consults (e.g. 社交合规) | C + I | admin 是 I |

> **派生约束 (per 守门 #3 v2 跨域 consults 派生规)**: 5 域之间**禁止** `delegates_to` / `reports_to` 边, **强制** `consults` / `collaborates` 边. ArgCrate.create_edge() 会检查此约束 (per 詳細 §1.2.2 关键算法).

---

## 2. 9 SA × 5 module RACI (纵向: 9 SA Type × 5 module)

### 2.1 9 SA × 5 module RACI 总表

| 9 SA | ArgCrate | SubAgentOrchestrator | LLMService | AgentLease | AgentRuntime |
|---|---|---|---|---|---|
| **SA-01 code-review** | R+A (持久化) | R (ARGContextInjector 拉 mentors review 模式) | R+A (review 摘要) | I (event: review.completed) | R+A (UI 审查 tab) |
| **SA-02 test-gen** | R+A (持久化) | R (ARGContextInjector 拉 mentors test 模式) | I (无 LLM 摘要) | I (event: test.completed) | R+A (UI test 报告) |
| **SA-03 5-域-lead-audit** | R+A (audit trail) | R+A (ARGOutputEvaluator 跨域审计) | R+A (审计摘要) | R+A (audit lease) | R+A (admin UI 审计) |
| **SA-04 git-ops** | R+A (持久化) | R+A (ARGDispatchRouter 派发 ops) | I (无 LLM) | R+A (ops lease) | R+A (UI ops 监控) |
| **SA-05 doc-sync** | R+A (持久化) | I (无 effect 维度) | I (无 LLM) | I (event: doc.synced) | R+A (UI doc 预览) |
| **SA-06 refactor** | R+A (持久化) | R (ARGContextInjector 拉 mentors refactor 模式) | R+A (重构建议) | I (event: refactor.completed) | R+A (UI diff 展示) |
| **SA-07 db-migration** | R+A (事件写入) | R (ARGDispatchRouter 派发 migration) | I (无 LLM) | R+A (migration lease) | R+A (admin UI migration 监控) |
| **SA-08 domain-dev** | R+A (持久化) | R+A (ARGDispatchRouter 派发 dev 任务) | R+A (业务代码生成) | I (event: dev.completed) | R+A (UI dev 监控) |
| **SA-09 achievement-eval** | R+A (解锁事件) | R+A (ARGAchievementEngine 8 拓扑评估) | R+A (解锁文案) | I (event: achievement.unlocked) | R+A (UI 成就墙推送) |

### 2.2 9 SA RACI 关键派生

- **SA-03 5-域-lead-audit** 是**唯一**一个全 module R+A 的 SA (per 要件 §6.1.3 特殊约束), 因为跨 5 域治理 + 审计是 admin 主战场
- **SA-09 achievement-eval** 是次全 module R+A (4/5 module, AgentLease 是 I), 因为成就是跨域共享 + 推送密集
- **SA-05 doc-sync** 是最简单的 SA (4/5 module I, 仅 ArgCrate + AgentRuntime R+A), 因为文档同步无 effect 维度
- **LLMService R+A 范围**: 仅 4 个 SA (SA-01 review / SA-03 审计 / SA-06 refactor / SA-08 domain-dev / SA-09 解锁), 其他 4 个 SA 是 I
- **AgentLease R+A 范围**: 仅 3 个 SA (SA-03 audit / SA-04 ops / SA-07 migration), 其他 6 个 SA 是 I (event 触发但 lease 不强管控)

---

## 3. 5 module × 9 SA × 5 域 完整 RACI 矩阵 (3 维)

### 3.1 ArgCrate × 9 SA × 5 域 (per 詳細 §1.1-§1.2)

| ArgCrate | player | economy | match | social | admin |
|---|---|---|---|---|---|
| **SA-01 code-review** | R+A | R+A | R+A | R+A | R+A |
| **SA-02 test-gen** | R+A | R+A | R+A | I | R+A |
| **SA-03 5-域-lead-audit** | I (被审计) | I (被审计) | I (被审计) | I (被审计) | R+A (审计) |
| **SA-04 git-ops** | R+A | I | R+A | I | R+A |
| **SA-05 doc-sync** | I | I | I | R+A (社交 API 文档) | R+A (治理文档) |
| **SA-06 refactor** | R+A (玩家 model) | R+A (经济 model) | I | R+A (社交 model) | I |
| **SA-07 db-migration** | I | R+A (经济 schema) | I | I | R+A (跨域 schema) |
| **SA-08 domain-dev** | R+A | R+A | R+A | I | I |
| **SA-09 achievement-eval** | R+A (玩家成就) | R+A (经济成就) | R+A (赛季成就) | R+A (成就墙) | I |

### 3.2 SubAgentOrchestrator × 9 SA × 5 域

| SubAgentOrchestrator | player | economy | match | social | admin |
|---|---|---|---|---|---|
| **SA-01 code-review** | I | I | I | I | I (ARGContextInjector 拉 mentors review) |
| **SA-02 test-gen** | I | I | I | I | I (ARGContextInjector 拉 mentors test) |
| **SA-03 5-域-lead-audit** | I (被审计) | I | I | I | R+A (ARGOutputEvaluator 跨域) |
| **SA-04 git-ops** | I | I | R+A (ARGDispatchRouter 派发 ops) | I | I |
| **SA-05 doc-sync** | I (无 effect) | I | I | I | I |
| **SA-06 refactor** | I | I | I | I | I (ARGContextInjector 拉 mentors refactor) |
| **SA-07 db-migration** | I | I | I | I | R+A (ARGDispatchRouter 派发 migration) |
| **SA-08 domain-dev** | I | I | R+A (ARGDispatchRouter 派发 dev) | I | I |
| **SA-09 achievement-eval** | I | I | I | R+A (ARGAchievementEngine) | R+A |

### 3.3 LLMService × 9 SA × 5 域

| LLMService | player | economy | match | social | admin |
|---|---|---|---|---|---|
| **SA-01 code-review** | C (player prompt) | C | C | C | C |
| **SA-02 test-gen** | I (无 LLM 摘要) | I | I | I | I |
| **SA-03 5-域-lead-audit** | C | C | C | C | R+A (审计摘要) |
| **SA-04 git-ops** | I (无 LLM) | I | I | I | I |
| **SA-05 doc-sync** | I (无 LLM) | I | I | I | I |
| **SA-06 refactor** | C (player prompt) | C | I | C | I |
| **SA-07 db-migration** | I (无 LLM) | I | I | I | I |
| **SA-08 domain-dev** | C (player 业务) | C | C | I | I |
| **SA-09 achievement-eval** | R+A (玩家成就文案) | R+A (经济成就文案) | R+A (赛季成就文案) | R+A (社交成就文案) | C |

### 3.4 AgentLease × 9 SA × 5 域

| AgentLease | player | economy | match | social | admin |
|---|---|---|---|---|---|
| **SA-01 code-review** | I (event: review.completed) | I | I | I | I |
| **SA-02 test-gen** | I (event: test.completed) | I | I | I | I |
| **SA-03 5-域-lead-audit** | I (被审计) | I | I | I | R+A (audit lease) |
| **SA-04 git-ops** | I (event: ops.completed) | I | I (ops lease) | I | I |
| **SA-05 doc-sync** | I (event: doc.synced) | I | I | I | I |
| **SA-06 refactor** | I (event: refactor.completed) | I | I | I | I |
| **SA-07 db-migration** | I (event: migration.completed) | I (migration lease) | I | I | R+A (migration lease) |
| **SA-08 domain-dev** | I (event: dev.completed) | I | I | I | I |
| **SA-09 achievement-eval** | I (event: achievement.unlocked) | I | I | I | I |

### 3.5 AgentRuntime × 9 SA × 5 域

| AgentRuntime | player | economy | match | social | admin |
|---|---|---|---|---|---|
| **SA-01 code-review** | C (UI 审查 tab) | C | C | C | C |
| **SA-02 test-gen** | C (UI test 报告) | C | C | I | C |
| **SA-03 5-域-lead-audit** | I (被审计) | I | I | I | R+A (admin UI 审计) |
| **SA-04 git-ops** | C (UI ops 监控) | I | C | I | C |
| **SA-05 doc-sync** | C (UI doc 预览) | C | C | R+A | C |
| **SA-06 refactor** | C (UI diff 展示) | C | I | C | I |
| **SA-07 db-migration** | I (admin UI migration 监控) | C | I | I | R+A |
| **SA-08 domain-dev** | C (UI dev 监控) | C | C | I | C |
| **SA-09 achievement-eval** | C (UI 成就墙推送) | C | C | R+A (主) | I |

---

## 4. 跨域 RACI 派生约束 (per 守门 #3 v2 拍板 D 维持)

### 4.1 跨域 R+A 决策 5 域 Lead 真人到位后追溯签字

| 跨域决策 | 当前 RACI (Mavis 临时代签) | 真人到位后 RACI |
|---|---|---|
| 跨域边 (consults / collaborates) 类型选择 | 跨域协调 (Mavis 接手) R+A | 5 域 Lead Lead 自执行 R+A, 跨域协调 C |
| 跨域 trust_score 调整 (≥ 0.1 delta) | 跨域协调 (Mavis 接手) R+A | admin Lead R+A + 其他 4 域 Lead C |
| 跨域成就解锁 (8 拓扑等) | 跨域协调 (Mavis 接手) R+A | 5 域 Lead 共同 R (协作) + 跨域协调 A |
| 跨域事件推送 (5 协议) | 跨域协调 (Mavis 接手) R+A | 5 域 Lead 各 R (域内) + 跨域协调 A |
| 5 域 Lead 真人寻访 | 跨域协调 (Mavis 接手) R+A | (待定, per 守门 #14 v3 暂时不追踪) |

### 4.2 5 域 Lead 真人到位流程 (per 守门 #14 v3 暂时不追踪)

- 真人到位流程 = 已作废 (per 9/8 05:27 JST 用户发令), 跨 session 不再追踪
- Mavis 永久代签维持, 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事)
- 5 域 Lead 寻访流程 = 不在 WBS 任何阻塞表 / 累计统计 / 修订历史中出现

---

## 5. 4 Effect 维度 RACI (per 要件 §1.1 + 基本 §3.2)

| 4 Effect 维度 | 5 module 跨模块 RACI | 5 域 RACI | 9 SA RACI |
|---|---|---|---|
| **#1 Dispatch 路由** (delegates_to / stand_in_for) | ARGDispatchRouter (SubAgentOrchestrator) R+A + ArgCrate R + AgentLease I + AgentRuntime I | match R+A (主) + admin C + 其他 3 域 I | SA-04 R+A + SA-08 R + SA-01 I |
| **#2 上下文共享** (mentors / shadows) | ARGContextInjector (SubAgentOrchestrator) R+A + LLMService R (摘要) + ArgCrate R + AgentLease I | social R+A (主) + admin C + 其他 3 域 I | SA-01 R + SA-02 R + SA-06 R + 其他 I |
| **#3 信任度加权** (trusts / peer_reviews) | ARGTrustEngine (SubAgentOrchestrator) R+A + LLMService C + ArgCrate R + AgentLease I | admin R+A (主) + 其他 4 域 C | SA-01 R + SA-03 R+A + SA-09 R + 其他 I |
| **#4 产出评估** (challenges / peer_reviews) | ARGOutputEvaluator (SubAgentOrchestrator) R+A + LLMService R (摘要) + ArgCrate R (audit) + AgentLease I | admin R+A (主) + 其他 4 域 C | SA-01 R + SA-03 R+A + SA-09 R+A + 其他 I |

---

## 6. 5 协议 RACI (per 要件 §4.1 F-12..F-16 + 詳細 §2.1)

| 5 协议 | ArgCrate | SubAgentOrchestrator | LLMService | AgentLease | AgentRuntime |
|---|---|---|---|---|---|
| **arg_edge_changed** (F-13) | R (持久化) | R+A (监听 + 处理) | I | R (推送) | I (UI 通知) |
| **arg_dispatch_route** (F-15) | I (read agent) | R+A (DispatchRouter 派发) | I | R (推送) | I (UI 通知) |
| **arg_context_inject** (F-16) | R (read history) | R+A (ContextInjector 注入) | R (摘要) | R (推送) | I (UI 通知) |
| **arg_trust_score_update** (F-12) | R+A (持久化) | R+A (TrustEngine 评估) | I | R (推送) | I (UI 通知) |
| **arg_achievement_unlocked** (F-14) | R+A (持久化) | R+A (AchievementEngine 解锁) | R (文案) | R (推送) | R+A (UI 成就墙) |

---

## 7. 守门合规 (per AGENTS.md §4 + §4.1)

- **#1 v15**: 守门 #12 死循环饱和边界
- **#3 v2**: 5 域独立 Lead, 跨域边强制 `consults` 而非 `delegates_to`
- **#5**: env 安全
- **#6**: PowerShell only
- **#9 v19**: 子代理 RPC 不可靠
- **#10**: 代签规则, RACI 表格 author = Ulysses
- **#14 v2 v3**: 5 域 Lead 拍板 D, Mavis 永久代签
- **#19 v19**: 守门 #12 死循环饱和边界

---

## 8. 签字栏 (per 守门 #14 v3 Mavis 永久代签)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| SRE Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 平台 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 评审主持 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| PM (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |

**5 域 Lead** (per 守门 #3 v2 拍板 B + 9/9 12:02 JST 守门 #14 v3 升级):

| 角色 | 签字 | 日期 |
|---|---|---|
| player Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 (真人到位后追溯) | 2026-09-09 |
| economy Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 (真人到位后追溯) | 2026-09-09 |
| match Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 (真人到位后追溯) | 2026-09-09 |
| social Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 (真人到位后追溯) | 2026-09-09 |
| admin Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 (真人到位后追溯) | 2026-09-09 |

---

## 9. 修订历史 (per 守门 #3 禁回溯叙事)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 21:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 域 × 9 SA × 5 module RACI 完整化 (5 域 RACI 总表 + 9 SA × 5 module 总表 + 3 维完整矩阵 + 4 effect 维度 RACI + 5 协议 RACI + 跨域派生约束 + 5 域 Lead 真人到位流程作废声明) | 2026-09-09 21:16 JST 用户发令"开子代理和 worktree 并行处理" + brief v0.45 §14.11 ARG.1 文档完整化 |
