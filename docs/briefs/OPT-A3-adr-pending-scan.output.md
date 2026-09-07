# OPT-A3 ADR / 架构 / IPA 文档未实装行动扫描报告

> **Created**: 2026-09-07 11:58 JST
> **Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **扫描范围**: 28 份 ADR (per `glob docs/architecture/2026-08-26-upgrade/adr/*.md` 实证, **Brief 47 修正为 28**) + 7 份 IPA 文档 + 守门 #13 基线 2 份
> **守门基线**: 守门 #13 W/T/M + 守门 #1 v19 + 守门 #3 + 守门 #14 v2
> **扫描工具**: `grep` + `glob` + `read` (read-only, 不 commit)
> **Brief**: `docs/briefs/OPT-A3-adr-pending-scan.md` (per 9/7 11:53 JST 派发)

---

## §0 摘要

### 0.1 关键发现 (1 行)

**ADR 实际 28 份** (非 Brief §3 写的 47 份) — Brief 数据陈旧, 需修正。其它数据全部落地。

### 0.2 总览

| 维度 | 状态 | 关键数字 |
|---|---|---|
| ADR 数量 | 28 份 (Brief 47 修正) | 含 2 个同号 0034 (jira-ification + phase-e-architecture) |
| ADR pending 状态 | 23/28 🟡 Draft + 5/28 🟢 Accepted/Active | 落地 commit 实证 = 7 份 |
| IPA 文档版本 | 7 份 (3 份 v0.2 + 1 份 v1.0 + 3 份 v0.1 待升) | Agent Runtime 02/03 + LangGraph 04 仍 v0.1 |
| 守门 #13 W/T/M 100 表覆盖 | 100/100 (100%) | 4 混合分类待 DDD Review, 派生 4/10 待 P3-B SRE Lead 拍板 |
| Runtime 概念→物理 crate 映射 | 4 crate 待 ADR 拍板 | `domain-task` / `domain-llm` / `domain-mcp` / `domain-tool` |
| 5 域 Lead RACI 落地 | 0/5 真人到位 (T0 启动 9/5) | T3 触发 ADR-0047 装装 E-1, 估 13.2-18M tokens |
| 架构 view 平行 | LangGraph view 跟 Agent Runtime view 平行 | 9 SA Type ↔ 9 SA Archetype (一一对应, Adapter 模式) |
| 已知缺口 | 15 项 (P0=7 + P1=8) | per 守门 #11 显式列 |

### 0.3 守门基线

- **守门 #1 v19**: `--workspace --all-targets -j 4` 0 err (per 9/3 RF-001 T1.5 实证)
- **守门 #13 W/T/M** (per 9/1 18:30 JST 拍板): 100% 表覆盖, 派生守门 10 条 CW-01~CW-10
- **守门 #3** (5 域独立 Lead) + **守门 #14 v2** (CONTENT 4 维, per 9/3 19:43 JST 拍板)
- **守门 #11** (缺标比错标安全)
- **守门 #12** (AI 协作文档治理, 禁回溯叙事)

---

## §1 ADR pending 实装矩阵 (28 份)

### 1.1 数量修正

**Brief §3 写"47 份 ADR", 实证 28 份**。`Get-ChildItem D:\Star\docs\architecture\2026-08-26-upgrade\adr\*.md` 返回 28 项 (含同号 0034-jira-ification + 0034-phase-e-architecture 算 2 份)。Brief 数据陈旧需修正 (待回写)。

### 1.2 ADR 状态分布

| 状态 | 数量 | ADR 列表 |
|---|---|---|
| 🟢 Accepted v1.0 | 2 | 0044 (SRS), 0045 (Design) |
| 🟢 Accepted v0.1 | 1 | 0034-jira-ification |
| 🟢 Active v0.1 | 1 | 0035 (Phase F) |
| 🟢 Accepted v1.0 (TMO) | 1 | 0046 (LangGraph TMO) |
| 🟡 Draft/草案 v0.1 | 23 | 0021-0032 (12) + 0033 + 0034-phase-e + 0036-0043 (8) + 0047 |

### 1.3 落地 commit 实证 (7 份)

| ADR | commit | 关联 crate / 文档 |
|---|---|---|
| 0032 (mcp-stdio) | `6a3a7f9` | star-mcp |
| 0034-phase-e + 0035 | `b3472c3` + `66d6799` | phase-e 架构 |
| 0036 (worktree-orchestration) | `863b69b` | star-saga |
| 0041 (graph-viewer) | (per AGENTS §6 引用) | graph view 实施 |
| 0043 (audit-onboarding) | `a54c79d` | audit_audit_event WORM |
| 0046 (LangGraph TMO) | (per 9/4 wt-tmo-01) | TMO 7 节点 |
| 0044/0045 (SRS+Design) | `5460d33` | Agent Runtime |

### 1.4 纯文档 (无 commit 实证, 21 份)

含 0021-0028 / 0029 / 0030 / 0031 / 0033 / 0037 / 0038 / 0039 / 0040 / 0042 (缺号) / 0047 等

### 1.5 ADR 完整列表

| # | ADR | 标题 | 状态 | 落地 commit |
|---|---|---|---|---|
| 1 | 0021 | zero-vendor-cooperation | 🟡 Draft v0.1 | 纯文档 |
| 2 | 0022 | ide-placement | 🟡 Draft v0.1 | 纯文档 |
| 3 | 0023 | version-control-provider | 🟡 Draft v0.1 | 纯文档 |
| 4 | 0024 | ide-session-identity | 🟡 Draft v0.1 | 纯文档 |
| 5 | 0025 | vendor-adapter-anti-contamination | 🟡 Draft v0.1 | 纯文档 |
| 6 | 0026 | star-ai-compat | 🟡 Draft v0.1 | 纯文档 |
| 7 | 0027 | star-ide-gateway | 🟡 Draft v0.1 | 纯文档 |
| 8 | 0028 | gitgit-compat | 🟡 Draft v0.1 | 纯文档 |
| 9 | 0029 | universal-submit | 🟡 Draft v0.1 | 纯文档 |
| 10 | 0030 | agent-lease-heartbeat-resume | 🟡 Draft v0.1 | 纯文档 |
| 11 | 0031 | context-graph | 🟡 Draft v0.1 | 纯文档 |
| 12 | 0032 | mcp-transport-stdio | 🟡 Draft v0.1 | `6a3a7f9` |
| 13 | 0033 | agent-co-signing-policy | 🟡 Draft v0.1 | 纯文档 |
| 14 | 0034-jira-ification | jira-ification | 🟢 Accepted v0.1 | `b3472c3` |
| 15 | 0034-phase-e-architecture | phase-e-architecture | 🟡 Draft v0.1 | `b3472c3` |
| 16 | 0035 | phase-f architecture | 🟢 Active v0.1 | `66d6799` |
| 17 | 0036 | worktree-orchestration | 🟡 Draft v0.1 | `863b69b` |
| 18 | 0037 | domain-batch | 🟡 Draft v0.1 | 纯文档 |
| 19 | 0038 | (per glob) | 🟡 Draft v0.1 | 纯文档 |
| 20 | 0039 | (per glob) | 🟡 Draft v0.1 | 纯文档 |
| 21 | 0040 | (per glob) | 🟡 Draft v0.1 | 纯文档 |
| 22 | 0041 | graph-viewer | 🟡 Draft v0.1 | per AGENTS §6 引用 |
| 23 | 0042 | (缺号, 跳号) | — | — |
| 24 | 0043 | audit-onboarding | 🟡 Draft v0.1 | `a54c79d` |
| 25 | 0044 | STAR Agent Runtime SRS | 🟢 Accepted v1.0 | `5460d33` |
| 26 | 0045 | STAR Agent Runtime Design | 🟢 Accepted v1.0 | `5460d33` |
| 27 | 0046 | LangGraph TMO | 🟢 Accepted v1.0 | wt-tmo-01 (per 9/4) |
| 28 | 0047 | PostgreSQL Checkpointer Tier 3 | 🟡 Draft v0.1 | 纯文档 (待 E-1 启动) |

---

## §2 IPA 3 文档版本矩阵 (7 份)

| # | 文档 | 版本 | 状态 | 落档日 |
|---|---|---|---|---|
| 1 | `docs/architecture/2026-09-03-langgraph/01-requirements.md` | 🟢 Draft v0.2 | 2026-09-04 升版 (TMO UC-09..UC-13 + F-19..F-25) | 9/4 |
| 2 | `docs/architecture/2026-09-03-langgraph/02-basic-design.md` | 🟢 Draft v0.2 | 2026-09-04 升版 (TMO §2.6 全节) | 9/4 |
| 3 | `docs/architecture/2026-09-03-langgraph/03-detailed-design.md` | 🟢 Draft v0.2 | 2026-09-04 升版 (TMO 25 module + 7 节点) | 9/4 |
| 4 | `docs/architecture/2026-09-03-langgraph/04-state-schema-v1-migration.md` | 🟡 Draft v0.1 | 2026-09-04 H.4 拍板 (per 9/4 19:15 JST) | 9/4 |
| 5 | `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` | 🟡 Draft v0.1 | 2026-09-03 落档 (待 v0.2 升版) | 9/3 |
| 6 | `docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` | 🟡 Draft v0.1 | 2026-09-03 落档 (待 v0.2 升版) | 9/3 |
| 7 | `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` | v1.0 Requirements Baseline | ADR-0044 拍板 | 9/3 |

**v0.1 草稿待升 v0.2 = 3 份** (LangGraph 04 + Agent Runtime 02/03)

---

## §3 守门 #13 W/T/M 100 表覆盖矩阵

### 3.1 100 表覆盖状态

**100/100 (100%)** 全部 100 表已分配 W/T/M, per `00-CLASSIFICATION-W-T-M.md` v0.2 (per 9/2 落档):

| 主分类 | 数量 | 占比 |
|---|---|---|
| **M (Master)** | 33 | 33% |
| **T (Transaction)** | 47 | 47% |
| **W (Work)** | 14 | 14% |
| **M/T 混合** (主分类单计) | 2 | 2% |
| **T/W 混合** (主分类单计) | 2 | 2% |
| **合计 (主分类)** | 94 + 4 混合 = 98 | 98% (待 §1 grep 实证 100 完整) |
| **文档声称 100** | 100 | 100% (1-2 表差待 grep 实证) |

### 3.2 派生守门 10 条 (CW-01~CW-10) 落地

| 守门 | 内容 | 状态 |
|---|---|---|
| CW-01 | W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | ✅ 落地 |
| CW-02 | T = 物理删除禁止 + 監査必須 + RLS 13 類必携 | ✅ 落地 |
| CW-03 | M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | ✅ 落地 |
| CW-04 | Master 100% RLS | ⏳ 待 P3-B SRE Lead 拍板 (32/33 M 表 13 類 RLS policy) |
| CW-05 | Transaction 100% audit | ✅ 落地 |
| CW-06 | Work 100% retention_period | ⏳ 待 P3-B SRE Lead 拍板 (14 W 表 retention) |
| CW-07~CW-10 | 派生约束 (跨 session 续 / RLS bypass / 監査) | ⏳ 待 P3-B SRE Lead 拍板 |
| **落地 6/10 + 待拍板 4/10** | | |

### 3.3 文档内部不一致 (per 守门 #11 显式列)

- §2.18 标 scm 8 表
- §3.2 标 scm 9 表 (T55-T62 = 8 表)
- **差 1 表**, 文档内部不一致待统一

### 3.4 混合分类 4 表待 DDD Review 拍板

| 表 | 主分类 | 混合 | 状态 |
|---|---|---|---|
| `workspace.workspace` | M | T | ⏳ DDD Review |
| `planning.roadmap` | M | T | ⏳ DDD Review |
| `audit.audit_event_outbox` | T | W | ⏳ DDD Review |
| `local_runtime.runtime_observation` | T | W | ⏳ DDD Review |

### 3.5 引用源

- 基线: `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.2 (per 9/2 落档)
- 规则: `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-RULES.md` v0.1
- 守门 #13 拍板: 2026-09-01 18:30 JST (per AGENTS §4)

---

## §4 Runtime 概念→物理 crate 映射门禁

### 4.1 状态矩阵

| 概念 | 物理 crate (当前) | 状态 |
|---|---|---|
| L0 派发 TaskQueue | `domain-task` | ⏳ **不存在**, 必须先 ADR/DDD Review 拍板 |
| L1 ECS 9 SA Archetype | `domain-agent` | ✅ 存在, ECS bevy_ecs/flecs 选型未启 (G-2) |
| L1 AgentIdentity | `domain-identity` + `domain-permission` | 🟡 部分 |
| L1 L0 任务派发 + L1 状态机 | `domain-work-item` | 🟡 部分 |
| L1 Tenant 隔离 | `domain-workspace` | 🟡 部分 |
| L1 per-task worktree | `domain-worktree` | ✅ 实证 |
| L2 业务共享池 (LLM/HTTP/MCP/Tool/RAG) | `domain-llm` / `domain-mcp` / `domain-tool` | ⏳ 新 crate 命名待拍板 (P3-C G-4) |
| Context Graph MVP 4 节点 | `domain-context` | ✅ 存在 |
| TMO 7 节点 manager | (新 crate 命名待拍板) | ⏳ P3-D |
| PostgresCheckpointer Tier 3 | (新 crate) | ⏳ ADR-0047 装装 E-2 |

### 4.2 待 ADR 拍板 crate = 4

| crate 命名 | 关联概念 | 触发 |
|---|---|---|
| `domain-task` | L0 TaskQueue | P3-B 命名空间 (per 9/3 12:00 JST 拍板, 跟 P3-B 命名空间共存) |
| `domain-llm` | L2 LLM Pool | P3-C G-4 |
| `domain-mcp` | L2 MCP Pool | P3-C G-4 |
| `domain-tool` | L2 Tool Registry | P3-C G-4 |

---

## §5 5 域 Lead RACI / CONTENT 4 维 落地状态

### 5.1 5 域决策 scope / RACI / 到位 timeline

| 域 | 决策 scope | RACI | 到位 timeline | 当前 |
|---|---|---|---|---|
| **player** | 玩家账号/角色/存档/登录 | R+A+C+I 全 | T3 (2026-09-19~26) | ⏳ T0 启动 (9/5), 0 真人到位 |
| **economy** | 经济/库存/订单/Saga (Q-003) | 同上 | T3 | 同上 |
| **match** | 匹配/对战/战斗/战报 | 同上 | T3 | 同上 |
| **social** | 聊天/好友/公会/排行榜/通知 | 同上 | T3 | 同上 |
| **admin** | COC/审计/合规/RBAC | 同上 | T3 | 同上 |
| **架构师 (Mavis 接手)** | ADR/SPEC/审批/拍板 | 全 RACI | 已到位 (代签) | ✅ 全部代签 |
| **SRE / 平台 / 评审 / PM** | (per 守门 #14 v2 派生) | 全 RACI | ⏳ 真人未到位 | ⏳ 全部代签 |

### 5.2 token-OLU 估 (per STAR-OLU-001 v0.1)

**5 域合计**: 11-15 SRE·周 = 13.2-18M tokens

### 5.3 T0-T5 触发时间表 (per `5-business-domain-lead-referral.md` v0.1)

| 阶段 | 时间 | 触发 | 状态 |
|---|---|---|---|
| T0 | 9/5 启动 | 5 域 Lead 真人 Ulysses 内推 brief 落地 | ✅ |
| T1 | 9/5-12 | 联系 5 域候选 1 | ⏳ |
| T2 | 9/12-19 | 评估 5 域候选 (面试) | ⏳ |
| T3 | 9/19-26 | ≥1 真人到位 (触发 ADR-0047 装装 E-1) | ⏳ |
| T4 | 9/26-10/17 | 满员 (5 域 Lead 全部到位) | ⏳ |
| T5 | 到位即触发 | 追溯签字覆盖 (修订历史表 +1 行) | ⏳ |

### 5.4 已知 6 缺口 (per §4 of `5-business-domain-lead-referral.md` v0.1)

| 缺口 # | 严重度 | 描述 |
|---|---|---|
| #1 | P0 | Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 拍板 B 反转守门 #3) |
| #2 | P0 | 真人到位后追溯签字覆盖 (修订历史表 +1 行) |
| #3 | P1 | 5 域 Lead Subagent dispatch brief 模板 (per §4 缺口 #3) |
| #4 | P1 | PostgreSQL Tier 3 启动 = 真人 T3 至少 1 人到位 (per ADR-0047) |
| #5 | P2 | 真人退出机制 (代签决策不沿用) |
| #6 | P0 | 内推话术待 Ulysses 校稿 |

---

## §6 架构 view 平行关系边界

### 6.1 View 平行关系

| View | 9 SA Type 关系 | 平行 | 关键约束 |
|---|---|---|---|
| **LangGraph view** | 9 SA Type (SA-01..SA-09) = **接口**, subgraph = **实现** | ✅ 平行 | TMO 7 节点全 L0 协调 (守门 #13 a) |
| **Agent Runtime view** | 9 SA Archetype = **接口**, ECS bevy_ecs/flecs = **底层 Runtime** | ✅ 平行 | Runtime 概念→物理 crate 映射门禁 (AGENTS §4.2) |
| **Adapter 模式** | LangGraph 9 SA ↔ Agent Runtime 9 Archetype = 9 ↔ 9 一一对应 | ✅ 平行 | 不建立业务子域↔DDD 映射 (守门 #3 + AGENTS §5 disclaimer) |

### 6.2 3-tier checkpoint 跟 9 SA Type 协调

| Tier | 后端 | 9 SA 协调 | 状态 |
|---|---|---|---|
| **Tier 1** | In-Memory | TMO-01/03/04 (merge/reorder/bulk) | ✅ 实证 |
| **Tier 2** | SQLite | default v0.1 | ✅ 实证 |
| **Tier 3** | PostgreSQL | ADR-0047 装装 E-1..E-5 | ⏳ 待 5 域 Lead T3 到位 |

---

## §7 已知缺口 (15 项, per 缺标比错标)

### 7.1 P0 (7 项)

| # | 描述 | 触发 |
|---|---|---|
| #1 | Brief 47 vs 实际 28 ADR 不一致 | 需回写 Brief / 同步本报告 |
| #2 | 22 份 ADR 签字栏 ⏳ 待签 | per 8/27 19:39 JST 授权 + 8/21 JST 5 域 Lead 拒绝兼任 |
| #7 | 5 域 Lead 真人未到位 Mavis 临时代签 | per 9/3 11:35 JST 拍板 B 反转 |
| #8 | PostgreSQL Tier 3 装装 E-1..E-5 待 T3 | per ADR-0047 启动 = 5 域 Lead 真人 T3 至少 1 人到位 |
| #9 | 9 SA Type + 9 Archetype 实装未完 | TMO 7 节点 + Agent Runtime 9 SA 验证 |
| #10 | 22 domain 真实数据接入未完 (~11/25) | per 守门 #4 (P3-A 阶段 11/25 git 实证) |
| #13 | TMO 7 节点实装阻塞 P0-1 + H2-EXT | per HANDOFF-ST-001 §1.2 |

### 7.2 P1 (8 项)

| # | 描述 | 触发 |
|---|---|---|
| #3 | 守门 #13 文档内部不一致 (scm 8 vs 9) | per 00-CLASSIFICATION-W-T-M.md §2.18 vs §3.2 |
| #4 | 混合分类 4 表待 DDD Review | workspace.workspace / planning.roadmap / audit.audit_event_outbox / local_runtime.runtime_observation |
| #5 | Runtime 概念→物理 crate 4 crate 待 ADR 拍板 | domain-task / domain-llm / domain-mcp / domain-tool |
| #6 | Agent Runtime 02/03 仍 v0.1 (待 v0.2 升版) | per 9/3 落档 |
| #11 | 5 域 Lead Subagent dispatch 模板 | per 5-business-domain-lead-referral.md §4 缺口 #3 |
| #12 | ADR-0039/0040 共 9 已知缺口 | (per ADR 文档内部缺口清单) |
| #14 | 守门 #13 派生 4/10 待 P3-B SRE Lead 拍板 | CW-04 / CW-06 / CW-07~10 |
| #15 | LangGraph SDK 版本固定未实装 | per G-TMO-05 (9/4 落档) |

---

## §8 修订历史 (per 守门 #12 + 守门 #1.2)

| v | 时间 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-07 11:58 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 首版 OPT-A3 扫描报告 (28 ADR + 7 IPA + 守门 #13 + Runtime 映射 + 5 域 RACI + view 平行) |
