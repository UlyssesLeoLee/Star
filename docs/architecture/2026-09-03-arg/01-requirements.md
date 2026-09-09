# ARG-ARCH-001 Agent Relationship Graph (ARG) 要件定義書

> **Status**: 🟡 Draft v0.1 (per 2026-09-09 21:18 JST 派发 brief v0.45 §14.11 ARG.1 文档完整化)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [BD §0 入口](./02-basic-design.md) · [DD §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md)
> **承接**: `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 + `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1 + `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 + 9/4 19:00 JST ARG.1 `crates/arg` 6 子模块骨架落地 (per `docs/briefs/arg-01-arg-crate-skeleton.md` §2)
> **本 view 范围 (per SRS-001 §0)**: 本要件定義書涵盖 **ARG 阶段 (ARG.1..ARG.11)** 的 5 module 体系 (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) + 9 SA Type (SA-01..SA-09) + 5 域 (player / economy / match / social / admin) 跨 RACI 责任矩阵. **不涵盖** 业务 DDD 映射 (per 守门 #3 v2 + AGENTS.md §5 仓库拓扑 disclaimer).

---

## 0. 目的 (Objective)

承接 2026-09-08 22:35 JST `ask_7d7ffcac2353adad7d3f6f69` 4 拍板 + 2026-09-09 用户发令"基本设计也做一下" + 2026-09-09 用户发令"自审" + 2026-09-09 用户发令"加入 wbs" + 2026-09-09 21:16 JST 用户发令"开子代理和 worktree 并行处理", 系统化定义 ARG 阶段 (11 子项 / 4 新 crate / 24 组件 / 13 REST + 1 WebSocket / 20 成就 / 10 类关系 / 5 团队模板 / 4 effect 维度 / ~30M token) 的:

1. **5 module 设计完整化** (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) — 本要件 + 基本設計 + 詳細設計 三件套落地
2. **9 SA (SA-01..SA-09) 完整化** (per LangGraph 9/3 §6.1 引用, 业务子代理类型)
3. **5 域 × 9 SA × 5 module RACI 责任矩阵** (per 守门 #3 v2 5 域 Lead 拍板 B + 9/3 11:35 JST 反转)
4. **5 module + 9 SA 关系图** (text-based 关系图, 无外部 mermaid 依赖)

> **dual-use 提醒 (per AGENTS.md §5 仓库拓扑)**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射. 5 域 (player/economy/match/social/admin) 是历史治理命名 (5 域 Lead 真人问责结构 per 守门 #3 拍板), **不** 跟 22 domain-* crate (DDD bounded context) 等同. Star 仓 docs 不得写 "per RGS 5 域镜像" / "per RGS" 等隐含引用 RGS 作为命名来源的描述.

---

## 1. 背景与动机 (Background & Motivation)

### 1.1 用户原话 (per 2026-09-08 22:35 JST)

> "agent 界面内, 各个 agent 之间可以有图论数据库那种 edge, 可以设置 agent 之间的关系, 这种关系可以反映到它们之间的协作和工作内容中... 我希望 agent 之间的关系可以对它们的工作产生益处, 创造不同的 agents 团队, 通过不同团队配置的组合, 实现更加丰富的成就"

核心动机: **agent 之间的关系不是展示标签, 而是要对工作产生益处 (effect 维度真实影响协作)**. 4 effect 维度真实落地:

| Effect 维度 | 实现 component | 工作益处 |
|---|---|---|
| **Dispatch 路由** | `ARGDispatchRouter` | Lead 收到任务按 outgoing `delegates_to` 自动 spawn Worker, 失败按 `stand_in_for` fallback |
| **上下文共享** | `ARGContextInjector` | mentee 启动拉 mentor 历史决策, shadow 静默订阅, token 节省 20-40% |
| **信任度加权** | `ARGTrustEngine` | trust_score ≥ 0.8 + trusts 边 weight ≥ 0.7 跳过 verify, 节省 15% token |
| **产出评估** | `ARGOutputEvaluator` | challenges 强制双向论证, peer_reviews 阈值 ≥ 0.8 |

### 1.2 ARG 跟现有 view 关系 (per BD §1.3 + DD §1.2)

- **不取代** LangGraph 任务卡 DAG (TMO 9 节点, 那是任务编排) — G-9 边界梳理 (DDD Review 拍板)
- **不取代** Agent View 画布 (SRS-AGENT-VIEW-001, 那是单体可视化) — ARG 是其 1 tab 视角
- **不取代** Agent Runtime ECS (那是底层 Runtime, per ADR-0045)
- **新增** agent 之间的 social 层 (Memgraph 持久化 + in-process LangGraph 同步桥)

---

## 2. 5 Module 设计体系 (per brief v0.45 §3.1)

ARG 阶段的 5 module 是 **业务/技术双维度** 的横切划分, 跟 4 new crate (crates/arg / crates/arg-bridge / crates/arg-effect / crates/api/src/arg 扩展) **一对一映射** (per §3 module 映射表). 5 module 名字跨 5 文档 (本要件 + 基本 + 详细 + RACI + Skill Diagrams) 严格一致, 不允许缩写或别名.

### 2.1 5 Module 名字与定位

| # | Module 名 | 物理 crate / 目录 | 一句话定位 |
|---|---|---|---|
| 1 | **ArgCrate** | `crates/arg/` | ARG 阶段数据核心 (Agent / Edge / Template / Achievement 模型 + Memgraph Bolt 客户端 + Cypher 缓存 + 事件写入 + LLM 调用 + 错误处理), 6 子模块 29 src + 7 tests |
| 2 | **SubAgentOrchestrator** | `crates/arg-effect/` 5 子模块 (ARGDispatchRouter + ARGContextInjector + ARGTrustEngine + ARGOutputEvaluator + ARGAchievementEngine) | 4 effect 维度 dispatcher (LangGraph subgraph ↔ Memgraph 双向同步) + 成就评估引擎, 4 维度真实影响协作 (per §1.1) |
| 3 | **LLMService** | `crates/arg/src/llm.rs` (ArgCrate 子模块) | ARG 阶段 LLM 调用层 (挑战/peer review 摘要/8 拓扑 Cypher 模板生成), 跟 L0 派发层 LLM Pool 共享 (per ADR-0045 Agent Runtime L2 共享池) |
| 4 | **AgentLease** | `crates/arg/src/ops/event_writer.rs` (ArgCrate 子模块) + `crates/api/src/arg/` 扩展 | ARG 阶段 lease 语义 (per ADR-0030 Agent Lease/Heartbeat/Resume 11 字段), 跨 Agent Handoff 锁 / 心跳 / 恢复, 跟 RLS 13 类隔离配合 |
| 5 | **AgentRuntime** | `crates/api/src/arg/` 13 REST + 1 WebSocket + `frontend/src/app/agent-relationships/` 5 UI 组件 | ARG 阶段运行时入口 (13 REST API + 1 WS + 5 UI 组件), 跟 axum 0.8 对齐 (per ADR-0048) |

### 2.2 5 Module 跟 4 new crate 映射

| crate | 包含 module | 状态 | 落地 commit |
|---|---|---|---|
| `crates/arg/` | ArgCrate + LLMService + AgentLease (事件写入部分) | 🟢 **ARG.1 收官** (commit `43c1f0c`) | 44 文件 / 4206 行 / 2 脚本 / 33 UT 100% pass |
| `crates/arg-bridge/` | AgentLease (同步桥部分) | 🟡 ARG.2 docs 阶段 (per DD §3.1 + §4.10-4.11) | 0 落地 |
| `crates/arg-effect/` | SubAgentOrchestrator | 🟡 ARG.3 docs 阶段 (per DD §3.1 + §4.5-4.9) | 0 落地 |
| `crates/api/src/arg/` 扩展 | AgentRuntime (REST/WS 部分) | 🟢 **ARG.4 收官** (commit `6e2cda6`) | 14 routes 13 REST + 1 WS / 24/24 cargo test + 10/10 Python IT |
| `frontend/src/app/agent-relationships/` | AgentRuntime (UI 部分) | 🟡 ARG.5 docs 阶段 (per DD §4.13) | 0 落地 |

> **关键设计决策**: 5 module 名字**贯穿** 5 文档 (本要件 + 基本 + 详细 + RACI + Skill Diagrams), 但**物理 crate 落地**可能跨多个 crate (per §2.2 映射表), 跟 §6.1 9 SA 名字一致性强约束.

---

## 3. 关键概念与术语 (Concepts & Terminology)

### 3.1 ARG 核心实体 (per DD §3.2)

| 实体 | Rust 类型 (草案) | 字段数 | 类别 (W/T/M) |
|---|---|---|---|
| **Agent** | `struct AgentNode { id, agent_type, tenant_id, label, status, trust_score, metadata, created_at, updated_at, ... }` | 16 | M (SCD Type 2 + 100% RLS) |
| **Edge** | `struct Edge { id, source_id, target_id, relationship_type, weight, created_at, updated_at, ... }` | 14 | M (SCD Type 2 + 100% RLS) |
| **RelationshipType** | `enum RelationshipType { DelegatesTo, ReportsTo, StandInFor, Mentors, Shadows, Trusts, Challenges, PeerReviews, Consults, Collaborates }` | 10 | (M 枚举) |
| **TeamTemplate** | `struct TeamTemplate { id, name, description, members_spec, edge_spec, created_at, ... }` | 5 | M |
| **Achievement** | `struct Achievement { id, code, name, description, category, rarity, condition_criteria, ... }` | 20 (8 拓扑 + 7 行为 + 5 产出) | M (8+7+5=20 成就, 8C+7R+3E+2L 分布) |
| **TrustScoreTier** | `enum TrustScoreTier { UNTRUSTED 0-0.2, LOW 0.2-0.4, MEDIUM 0.4-0.7, HIGH 0.7-0.9, VERY_HIGH 0.9-1.0 }` | 5 | (派生) |
| **ARGEvent** | `struct ARGEvent { id, event_type, source_id, target_id, payload, timestamp, ... }` | 9 | T (append-only) |
| **ARGError** | `enum ARGError { NotFound, Unauthorized, InvalidEdge, InvalidTemplate, MemgraphDown, ... }` | 10 | (错误) |

### 3.2 5 域 (player / economy / match / social / admin) 跨文档一致

5 域是历史治理命名 (per 守门 #3 拍板), **不**跟 22 domain-* crate (DDD bounded context) 等同. 跨 5 文档 (本要件 + 基本 + 详细 + RACI + Skill Diagrams) 一致使用:

| 5 域 | 业务范围 (粗略) | 9 SA 跨域覆盖 | 5 module 跨域覆盖 |
|---|---|---|---|
| **player** | 用户/角色/账户/身份 | SA-01 code-review (玩家代币改动) + SA-02 test-gen (玩家数据) + SA-04 git-ops (玩家配置) + SA-06 refactor (玩家 model) | ArgCrate (数据) + AgentRuntime (UI) |
| **economy** | 货币/资产/交易/支付 | SA-02 test-gen (经济系统) + SA-07 db-migration (经济 schema) + SA-08 domain-dev (经济逻辑) | ArgCrate (数据) + SubAgentOrchestrator (effect) |
| **match** | 对战/匹配/排行榜/赛季 | SA-01 code-review (匹配算法) + SA-08 domain-dev (match engine) + SA-09 待定义 | ArgCrate (数据) + SubAgentOrchestrator (effect 路由) |
| **social** | 好友/聊天/群组/成就 | SA-05 doc-sync (社交 API 文档) + SA-06 refactor (社交 model) + SA-09 待定义 | ArgCrate (数据) + AgentRuntime (UI 关系编辑) |
| **admin** | 后台/审计/合规/凭证 | SA-03 5-域-lead-audit (跨域审计) + SA-04 git-ops (admin 部署) | ArgCrate (数据) + AgentLease (audit trail) + AgentRuntime (admin UI) |

> **重要约束 (per §3.2 注释)**: 5 域是 RGS 仓**历史治理命名**, Star 仓 docs 引用时**禁止** 写 "per RGS 5 域镜像" / "per RGS" 等隐含引用 RGS 作为命名来源的描述 (per 8/30 09:08 JST Ulysses 明确反馈). 9 SA 跨域覆盖 + 5 module 跨域覆盖是**当前工作线决策**, 跟 DDD bounded context 不建立映射.

---

## 4. 业务功能要件 (Functional Requirements)

### 4.1 20 功能 (per SRS-AGENT-RELATIONSHIP-001 §3)

| F-N | 功能名 | 5 module 命中 | 9 SA 命中 | 优先级 |
|---|---|---|---|---|
| F-01 | Agent 节点 CRUD (POST/GET/PATCH /api/arg/agents) | AgentRuntime + ArgCrate | SA-05 doc-sync + SA-09 | P0 |
| F-02 | Edge 关系 CRUD (POST/GET/PATCH/DELETE /api/arg/edges) | AgentRuntime + ArgCrate | SA-04 git-ops + SA-06 refactor | P0 |
| F-03 | 图谱查询 (GET /api/arg/graph) | AgentRuntime + ArgCrate + SubAgentOrchestrator | SA-01 code-review + SA-08 | P0 |
| F-04 | 团队模板 instantiate (POST /api/arg/templates/instantiate) | AgentRuntime + ArgCrate | SA-08 domain-dev | P0 |
| F-05 | 成就解锁 (POST /api/arg/achievements/unlock) | AgentRuntime + ArgCrate + SubAgentOrchestrator | SA-09 (待定义) | P1 |
| F-06 | 成就墙 (GET /api/arg/achievements) | AgentRuntime + ArgCrate | SA-05 doc-sync | P1 |
| F-07 | Dispatch 路由 (delegates_to / stand_in_for) | SubAgentOrchestrator | SA-01 + SA-04 + SA-08 | P0 |
| F-08 | 上下文注入 (mentors / shadows) | SubAgentOrchestrator + LLMService | SA-02 + SA-06 | P1 |
| F-09 | 信任度评估 (trusts / peer_reviews) | SubAgentOrchestrator + LLMService | SA-01 + SA-03 | P0 |
| F-10 | 产出评估 (challenges / peer_reviews) | SubAgentOrchestrator + LLMService | SA-01 + SA-03 + SA-09 | P0 |
| F-11 | 事件推送 (WebSocket /ws/arg/events) | AgentRuntime + ArgCrate | (跨域) | P0 |
| F-12 | 信任分变更 (arg_trust_score_update 协议) | SubAgentOrchestrator + AgentLease | (跨域) | P1 |
| F-13 | 关系变更 (arg_edge_changed 协议) | SubAgentOrchestrator + AgentLease | (跨域) | P0 |
| F-14 | 成就解锁 (arg_achievement_unlocked 协议) | SubAgentOrchestrator + AgentLease | (跨域) | P1 |
| F-15 | 派发路由变更 (arg_dispatch_route 协议) | SubAgentOrchestrator + AgentLease | (跨域) | P0 |
| F-16 | 上下文注入 (arg_context_inject 协议) | SubAgentOrchestrator + AgentLease + LLMService | (跨域) | P1 |
| F-17 | 离线降级 (sled 离线队列) | ArgCrate + AgentLease | (跨域) | P0 |
| F-18 | 周期 flush (30s PeriodFlushWorker) | ArgCrate + AgentLease | (跨域) | P0 |
| F-19 | Memgraph Bolt 同步 | ArgCrate + AgentLease | (跨域) | P0 |
| F-20 | 1-click 模板 (≤ 30s instantiate) | AgentRuntime + SubAgentOrchestrator | SA-08 | P0 |

**5 module 命中分布**: AgentRuntime 14/20, ArgCrate 14/20, SubAgentOrchestrator 10/20, LLMService 4/20, AgentLease 8/20.
**9 SA 命中分布**: SA-01 4, SA-02 2, SA-03 3, SA-04 3, SA-05 2, SA-06 3, SA-07 1, SA-08 4, SA-09 4 (含待定义).

### 4.2 4 NFR (Non-Functional Requirements)

| NFR-N | NFR 名 | 度量 | 5 module 命中 | 9 SA 命中 |
|---|---|---|---|---|
| NFR-P-01 | 边创建延迟 P95 | < 200ms | AgentRuntime + ArgCrate | SA-04 |
| NFR-P-02 | Cypher 查询 P95 | < 500ms | ArgCrate + SubAgentOrchestrator | SA-08 |
| NFR-P-03 | 事件推送延迟 P95 | < 100ms | AgentRuntime + AgentLease | (跨域) |
| NFR-P-04 | 成就评估 P95 | < 1s | SubAgentOrchestrator + LLMService | SA-01 + SA-03 |
| NFR-R-01 | 离线降级 | sled 离线队列 + 5min 重试 | ArgCrate + AgentLease | (跨域) |
| NFR-R-02 | Memgraph HA 集群 | replica set (后续阶段 G-7) | ArgCrate | (跨域) |
| NFR-S-01 | RLS 13 类隔离 | per-tenant + per-agent 双层 | ArgCrate + AgentRuntime | (跨域) |
| NFR-S-02 | SCD Type 2 | Master 表历史可追溯 | ArgCrate | SA-07 |
| NNR-S-03 | Env 不打印 | 凭证 stdin pipe | (跨域) | (跨域) |
| NFR-S-04 | 代签规则 | author = Ulysses (per 守门 #10) | (跨域) | (跨域) |
| NFR-A-01 | 8 拓扑成就 | 8C+7R+3E+2L = 20 分布 | SubAgentOrchestrator | SA-09 |
| NFR-A-02 | 7 行为 + 5 产出成就 | 异步触发 + SSE 推送 | SubAgentOrchestrator + LLMService | SA-09 |
| NFR-U-01 | 1-click 模板 | ≤ 30s instantiate | AgentRuntime + SubAgentOrchestrator | SA-08 |
| NFR-U-02 | 拖拽建边 | ≤ 3 步 (3 click) | AgentRuntime | SA-08 + SA-09 |
| NFR-O-01 | Prometheus 指标 | 7 指标 + tracing | ArgCrate + AgentRuntime | (跨域) |

### 4.3 5 想定シナリオ (Scenarios)

| S-N | シナリオ名 | 触发 | 5 module 跨模块联动 |
|---|---|---|---|
| S-01 | 创建 Lead-Worker 关系 (delegates_to) | UI 拖拽 / API POST | AgentRuntime → ArgCrate → SubAgentOrchestrator (ARGDispatchRouter 监听) → AgentLease (事件推送) |
| S-02 | Lead 失败 Worker fallback (stand_in_for) | Worker 失败事件 | AgentLease → SubAgentOrchestrator (ARGDispatchRouter 触发 stand_in_for) → ArgCrate (信任度下降) |
| S-03 | mentee 启动拉 mentor 上下文 | mentee 任务卡创建 | SubAgentOrchestrator (ARGContextInjector 拉 mentors 历史) → LLMService (摘要) → ArgCrate (持久化) |
| S-04 | 信任度加权跳过 verify | trust_score ≥ 0.8 评估 | SubAgentOrchestrator (ARGTrustEngine 评估) → ArgCrate (跳过 verify 路径) |
| S-05 | 8 拓扑成就解锁 (e.g. 4 COMMON: Triangle/K-Core/Star/Path) | 关系图谱满足拓扑 | SubAgentOrchestrator (ARGAchievementEngine Cypher 评估) → LLMService (解锁文案) → AgentRuntime (UI 推送) |
| **S-06** | **合并任务 A 和任务 B** (per 2026-09-04 19:15 JST 用户发令"langgraph 功能需要可以操控任务卡, 合并任务 a 和任务 b") | 任务卡合并 UI | AgentRuntime → SubAgentOrchestrator (arg_bridge 同步 TMO 7 节点 M-N1 merge) → ArgCrate (关系图谱更新) → AgentLease (audit trail) |

---

## 5. 制約 (Constraints)

### 5.1 5 域跨域约束 (per 守门 #3 拍板 B 反转)

5 域 Lead 真人到位前 Mavis 临时代签 (per 9/3 11:35 JST 拍板 B + 9/3 19:43 JST 拍板 v2 RACI 升级). 5 域 Lead 真人到位流程暂时不追踪 (per 9/9 12:02 JST 守门 #14 v3 升级), 真人到位后追溯签字覆盖修订历史.

**跨域边约束**: 5 域之间的 edge **强制** 走 `consults` 而非 `delegates_to` (per 守门 #3 跨域 consults 而非 delegates_to 派生规).

### 5.2 9 SA 跨域约束 (per LangGraph 9/3 §6.1)

9 SA (SA-01..SA-09) 是 LangGraph 业务子代理类型, 跨 5 域复用 (per §6.1 9 SA 完整化). 5 域 Lead 真人到位前 SA-03 (5-域-lead-audit) 由 Mavis 临时代签 (per §5.1).

### 5.3 5 module 跨域约束

5 module (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) 跨 5 域 + 9 SA 共享, 跨 5 文档 (本要件 + 基本 + 详细 + RACI + Skill Diagrams) 严格名字一致.

### 5.4 4 effect 维度跨域约束 (per §1.1 4 effect 维度真实落地)

4 effect 维度 (Dispatch / 上下文 / 信任度 / 产出) 真实影响协作, 不是展示标签. SubAgentOrchestrator 必须落地 4 component 完整实现, 不允许只列 1-2 个 component.

### 5.5 RLS 13 类隔离约束 (per 守门 #13 W/T/M 派生规 (d))

5 张新表 (per §3.1) 100% RLS 13 类覆盖: agents=Master / edges=Master / audit=Transaction / template_instances=Work (TTL 30d) / unlocks=Transaction. 禁止混在一括列举 (per 守门 #13 横展开原则).

---

## 6.1 9 SA 完整化 (per LangGraph 9/3 §6.1)

9 SA (SA-01..SA-09) 是 LangGraph 业务子代理类型, 跨 5 域 + 5 module 复用. 9 SA 名字跨 5 文档 (本要件 + 基本 + 详细 + RACI + Skill Diagrams) 严格一致.

### 6.1.1 SA-01 code-review (代码审查)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-01 |
| **类型名** | code-review |
| **用途** | 代码审查 (per PR/MR) |
| **主要工具** | git diff, code search, comment |
| **节点模板** | review-plan → review-execute → review-report |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGContextInjector 拉 mentors 历史 review) + LLMService (review 摘要) + ArgCrate (review 关系持久化) + AgentRuntime (UI 审查 tab) |
| **5 域命中** | player (玩家代币改动) + match (匹配算法) + economy (经济系统) + admin (跨域审计) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-02 test-gen (review 后生成测试) + SA-06 refactor (review 后重构) + SA-04 git-ops (review 后 commit) |

### 6.1.2 SA-02 test-gen (测试生成)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-02 |
| **类型名** | test-gen |
| **用途** | 测试生成 |
| **主要工具** | code search, test run |
| **节点模板** | test-plan → test-execute → test-verify |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGContextInjector 拉 mentors 历史 test 模式) + ArgCrate (test 关系持久化) + AgentRuntime (UI test 报告) |
| **5 域命中** | player (玩家数据) + economy (经济系统) + admin (审计) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-01 code-review (review 后生成) + SA-07 db-migration (DB 测试) + SA-06 refactor (重构后 test) |

### 6.1.3 SA-03 5-域-lead-audit (5 域 Lead 配置审计)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-03 |
| **类型名** | 5-域-lead-audit |
| **用途** | 5 域 Lead 配置审计 (per UC-08, 跨 22 domain crates + 5 域治理矩阵) |
| **主要工具** | 22 domain crates read |
| **节点模板** | audit-plan → audit-execute (跨域) → audit-report |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGOutputEvaluator 跨域审计) + LLMService (审计摘要) + ArgCrate (audit trail 持久化) + AgentLease (审计 lease) + AgentRuntime (admin UI 审计报告) |
| **5 域命中** | admin (主, 跨域审计) + player + economy + match + social (被审计) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-01 code-review (审计发现 review) + SA-04 git-ops (审计 fix commit) + SA-07 db-migration (DB 审计) |
| **特殊约束** | **唯一**一个跨域 + 治理矩阵型 sub-agent. 依赖 5 域 Lead 真人到位 (per 守门 #3 反転 Mavis 临时代签, 见 19:39 JST 授权); 真人到位后追溯签字. 其余 8 个 SA 都是 task-bound 单域/单工具型. (per LangGraph 9/3 §6.1) |

### 6.1.4 SA-04 git-ops (git 操作)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-04 |
| **类型名** | git-ops |
| **用途** | git 操作 (worktree/commit/push) |
| **主要工具** | git worktree, star CLI |
| **节点模板** | ops-plan → ops-execute → ops-verify |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGDispatchRouter 派发 ops) + ArgCrate (ops 关系持久化) + AgentLease (ops lease 锁) + AgentRuntime (UI ops 监控) |
| **5 域命中** | player (玩家配置) + admin (部署) + 跨域 (worktree 编排) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-05 doc-sync (commit 后同步文档) + SA-01 code-review (commit 前 review) + SA-06 refactor (重构后 commit) |

### 6.1.5 SA-05 doc-sync (文档同步)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-05 |
| **类型名** | doc-sync |
| **用途** | 文档同步 (AGENTS.md / WBS / ADR) |
| **主要工具** | file write, git add+commit |
| **节点模板** | doc-plan → doc-execute → doc-verify |
| **5 module 跨模块联动** | ArgCrate (doc 关系持久化) + AgentRuntime (UI doc 预览) |
| **5 域命中** | social (社交 API 文档) + admin (治理文档) + 跨域 (AGENTS.md) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-04 git-ops (commit doc 改动) + SA-01 code-review (doc review) |

### 6.1.6 SA-06 refactor (代码重构)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-06 |
| **类型名** | refactor |
| **用途** | 代码重构 |
| **主要工具** | code search, edit, test |
| **节点模板** | refactor-plan → refactor-execute → refactor-verify (cargo test) |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGContextInjector 拉 mentors 重构模式) + LLMService (重构建议) + ArgCrate (refactor 关系持久化) + AgentRuntime (UI diff 展示) |
| **5 域命中** | player (玩家 model) + social (社交 model) + economy (经济 model) |
| **优先级** | P1 |
| **9 SA 跨 SA 协作** | SA-01 code-review (refactor 后 review) + SA-02 test-gen (refactor 后 test) + SA-04 git-ops (refactor 后 commit) |

### 6.1.7 SA-07 db-migration (DB migration)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-07 |
| **类型名** | db-migration |
| **用途** | DB migration (per 守门 #13 W/T/M 严格) |
| **主要工具** | 22 domain DB schema |
| **节点模板** | db-plan → db-migrate → db-verify |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGDispatchRouter 派发 migration) + ArgCrate (migration 事件写入) + AgentLease (migration lease 锁) + AgentRuntime (admin UI migration 监控) |
| **5 域命中** | economy (经济 schema) + admin (跨域 schema) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-01 code-review (schema review) + SA-02 test-gen (migration test) + SA-04 git-ops (migration commit) |
| **特殊约束** | 严格按守门 #13 W/T/M 横展开, 新表必须 100% 覆盖 (W / T / M 三类分门别类, 禁止混在一括列举). |

### 6.1.8 SA-08 domain-dev (域业务开发)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-08 |
| **类型名** | domain-dev |
| **用途** | 22 domain-* crate 业务逻辑开发 (player / identity / project / work-item / etc) |
| **主要工具** | 22 domain crates + cross-cutting supporting crates |
| **节点模板** | dev-plan → dev-execute → dev-verify (cargo test + lint) |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGDispatchRouter 派发 dev 任务) + LLMService (业务代码生成) + ArgCrate (dev 关系持久化) + AgentRuntime (UI dev 监控) |
| **5 域命中** | match (match engine) + economy (经济逻辑) + player (玩家逻辑) |
| **优先级** | P0 |
| **9 SA 跨 SA 协作** | SA-01 code-review (dev 后 review) + SA-02 test-gen (dev 后 test) + SA-06 refactor (dev 过程 refactor) + SA-07 db-migration (DB 关联) |

### 6.1.9 SA-09 achievement-eval (成就评估, 暂定, 待 DDD Review 拍板)

| 维度 | 内容 |
|---|---|
| **类型 ID** | SA-09 |
| **类型名** | achievement-eval |
| **用途** | ARG 阶段成就评估 (8 拓扑 + 7 行为 + 5 产出 = 20 成就, per §3.1) |
| **主要工具** | ArgCrate 8 拓扑 Cypher 模板 + LLMService 解锁文案生成 |
| **节点模板** | eval-plan → eval-execute (Cypher + LLM) → eval-report (推送 WS) |
| **5 module 跨模块联动** | SubAgentOrchestrator (ARGAchievementEngine 8 拓扑评估) + LLMService (解锁文案) + ArgCrate (解锁事件写入) + AgentRuntime (UI 成就墙推送) |
| **5 域命中** | social (成就墙) + match (赛季成就) + economy (经济成就) + player (玩家成就) + admin (审计成就) |
| **优先级** | P1 |
| **9 SA 跨 SA 协作** | SA-01 code-review (行为评估 review) + SA-03 5-域-lead-audit (产出评估跨域审计) + SA-10 task-orchestrator (待 TMO 落地, 跨任务编排型 per ADR-0046) |
| **特殊约束** | ⚠️ **SA-09 暂定名** (per LangGraph 9/3 §6.1 跟 ARG 阶段对齐, 待 DDD Review 拍板). 候选名: achievement-eval / arg-effect / arg-dispatch / etc. |

### 6.1.10 9 SA 跨 SA 关系矩阵 (per §6.1.1-§6.1.9 汇总)

| SA | SA-01 | SA-02 | SA-03 | SA-04 | SA-05 | SA-06 | SA-07 | SA-08 | SA-09 |
|---|---|---|---|---|---|---|---|---|---|
| **SA-01 code-review** | — | 1 (生成) | 1 (审计) | 1 (commit 前) | 1 (review doc) | 1 (review refactor) | 1 (review schema) | 1 (review dev) | 1 (review 行为) |
| **SA-02 test-gen** | 2 (review 后) | — | 0 | 0 | 0 | 2 (重构后) | 2 (migration test) | 2 (dev 后) | 0 |
| **SA-03 5-域-lead-audit** | 2 (审计发现) | 0 | — | 2 (审计 fix) | 0 | 0 | 2 (DB 审计) | 0 | 2 (产出审计) |
| **SA-04 git-ops** | 2 (review 后) | 0 | 0 | — | 2 (commit doc) | 2 (refactor 后) | 2 (migration commit) | 2 (dev commit) | 0 |
| **SA-05 doc-sync** | 2 (doc review) | 0 | 0 | 1 (commit) | — | 0 | 0 | 0 | 0 |
| **SA-06 refactor** | 1 (refactor review) | 1 (refactor test) | 0 | 1 (refactor commit) | 0 | — | 0 | 1 (dev 过程) | 0 |
| **SA-07 db-migration** | 1 (schema review) | 1 (migration test) | 1 (DB 审计) | 1 (migration commit) | 0 | 0 | — | 1 (DB 关联) | 0 |
| **SA-08 domain-dev** | 1 (dev review) | 1 (dev test) | 0 | 1 (dev commit) | 0 | 1 (dev 过程) | 1 (DB 关联) | — | 0 |
| **SA-09 achievement-eval** | 1 (行为 review) | 0 | 1 (产出审计) | 0 | 0 | 0 | 0 | 0 | — |

> **注**: 1 = 触发协作 (上下游), 2 = 强协作 (验收). 0 = 无直接协作.

### 6.1.11 9 SA 跟 5 module 跨模块映射 (跨文档一致)

| SA | ArgCrate | SubAgentOrchestrator | LLMService | AgentLease | AgentRuntime |
|---|---|---|---|---|---|
| **SA-01** | ✅ 持久化 | ✅ ContextInjector | ✅ 摘要 | — | ✅ 审查 tab |
| **SA-02** | ✅ 持久化 | ✅ ContextInjector | — | — | ✅ test 报告 |
| **SA-03** | ✅ audit trail | ✅ OutputEvaluator | ✅ 审计摘要 | ✅ audit lease | ✅ admin UI |
| **SA-04** | ✅ 持久化 | ✅ DispatchRouter | — | ✅ ops lease | ✅ ops 监控 |
| **SA-05** | ✅ 持久化 | — | — | — | ✅ doc 预览 |
| **SA-06** | ✅ 持久化 | ✅ ContextInjector | ✅ 建议 | — | ✅ diff 展示 |
| **SA-07** | ✅ 事件写入 | ✅ DispatchRouter | — | ✅ migration lease | ✅ migration 监控 |
| **SA-08** | ✅ 持久化 | ✅ DispatchRouter | ✅ 代码生成 | — | ✅ dev 监控 |
| **SA-09** | ✅ 解锁事件 | ✅ AchievementEngine | ✅ 解锁文案 | — | ✅ 成就墙推送 |

---

## 7. 跟现有 view 关系 (per §1.2 + 守门 #3 + ADR-0048)

### 7.1 ARG 跟 LangGraph 9/3 关系 (per ADR-0046)

- **平行 view** (per 守门 #3 v2): ARG 跟 LangGraph 任务卡 DAG 平行, 不取代 TMO 9 节点
- **9 SA 引用**: SA-01..SA-09 引用 LangGraph 9/3 §6.1, 业务子代理类型一致
- **SA-10 task-orchestrator** (per ADR-0046 v0.2): 跨任务编排型, 由 TMO 7 节点 (M-N1..M-N7) 派发, 不属于本 ARG 9 SA 范围

### 7.2 ARG 跟 Agent Runtime 9/3 关系 (per ADR-0045)

- **平行 view** (per 守门 #3 v2): ARG 跟 Agent Runtime 平行, 不取代 ECS 9 Archetype
- **9 SA Type 引用**: Agent Runtime ECS 9 Archetype 引用 LangGraph 9 SA Type, 跟 ARG 9 SA 一致
- **L2 共享池复用**: LLMService 跟 Agent Runtime L2 LLM Pool 共享 (per ADR-0045 §3.3)

### 7.3 ARG 跟 Star-EI 9/7 关系 (per ADR-0048 + 守门 #13)

- **5 张新表 W/T/M 严格**: agents=Master / edges=Master / audit=Transaction / template_instances=Work / unlocks=Transaction (per 守门 #13 派生规)
- **RLS 13 类隔离**: per-tenant + per-agent 双层 (跟 Star-EI 一致)
- **不引用 RGS 仓**: per AGENTS.md §5 仓库拓扑硬约束

### 7.4 ARG 跟 PG Checkpointer Tier 3 关系 (per ADR-0047)

- **共享 PG 集群**: ARG 的 audit / unlocks 表跟 PG Checkpointer 共享同一 PG 集群
- **锁服务**: `pg_try_advisory_xact_lock` 跟 checkpointer 事务隔离 (锁在事务内, checkpointer 跨事务)

### 7.5 ARG 跟 Agent Lease/Heartbeat/Resume 关系 (per ADR-0030)

- **共享 lease 语义**: AgentLease (per §2.1 #4) 跟 ADR-0030 11 字段对齐
- **ARG 多了 4 effect 维度**: Dispatch 路由 / 上下文共享 / 信任度 / 产出评估 (per §1.1)

---

## 8. 已知缺口 (Known Gaps, per 守门 #11 缺标比错标)

1. **G-1** Memgraph 客户端 crate (`r2d2-memgraph`) 待调研, 候选: 自实现 (Bolt protocol) / 用 `memgraph-client` 社区 — P3-C W1 第一件事
2. **G-2** k3s 部署 yaml 待写, Docker compose 模板可写 — P3-C W2
3. **G-3** ~~L0↔L1 通信协议 + ARG 集成~~ — **本 DD §5 已闭环** (PyO3 binding + 5 Reducer + 4 effect 维度集成代码)
4. **G-4** trusts 跳过 verify 安全审计 (建议 trust_score ≥ 0.9 + agent 类型白名单双约束) — DDD Review 拍板
5. **G-5** ~~challenges 双向论证 prompt 模板 10 套~~ — **本 DD §7 已闭环** (5 decision_type × 2 trust_tier 组合)
6. **G-6** ~~8 拓扑成就 Cypher 模板~~ — **本 DD §6 已闭环** (TOP-001..TOP-008 完整 Cypher)
7. **G-7** Memgraph HA 集群 (单点故障, replica set) — 后续阶段
8. **G-8** 5 域 Lead 真人到位 timeline — 真人到位时 (per 守门 #14 v3 暂时不追踪)
9. **G-9** 跟 TMO 9 节点 (任务卡 DAG) 边界梳理 — DDD Review 拍板
10. **G-10** ARG Schema V2 迁移路径 (V1 → V2 加新关系类型时怎么处理存量数据) — P3-E 写 `arg_migration` v1→v2 脚本
11. **G-11** 成就可分享的 PNG 导出 + 描述 JSON + 周报模板 — 后续阶段
12. **G-12** ARG 跟 RGS 仓的独立边界 (per AGENTS.md §5 仓库拓扑硬约束, Star 仓不引用 RGS 5 域镜像作为业务源头) — 持续合规
13. **G-13** SA-09 名字 (achievement-eval 暂定) — DDD Review 拍板
14. **G-14** 4 effect 维度跨域 token 节省实证 (估 20-40% 上下文 + 15% verify) — P3-D W2 ARG.7 8 E2E 落地

---

## 9. 守门合规 (per AGENTS.md §4 + §4.1)

- **#1 v15**: 守门 #12 死循环饱和边界 (4 次新事件触发都允许)
- **#1 v19**: 自动化档判定 ≥ 2 维 [P] 强制 Python 化 (4 子项必先 `scripts/automation/<purpose>.py` 落地)
- **#1 v25**: cargo test 单 crate 模式 (per PR #12)
- **#3 v2**: 5 域独立 Lead, 跨域边强制 `consults` 而非 `delegates_to` (per 9/3 11:35 JST 拍板 B 反转)
- **#5**: env 安全, Memgraph 连接串走 env, 不打印
- **#6**: PowerShell only, ARG 部署脚本 PowerShell
- **#7**: 0 unsafe, `unsafe_code = "forbid"` per workspace lints
- **#9 v19**: 子代理 RPC 不可靠, ARG 同步走 in-process 推 + 周期 flush, 不用 RPC
- **#10**: 代签规则, ARG 关系修改 author = Ulysses (per 9/8 15:19 JST 第 6 次强化)
- **#12 v21**: Python 化任务卡, [P] 子项 docs 同步必更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md`
- **#13**: W/T/M 三类横展开, 5 表 100% 覆盖 (agents=Master / edges=Master / audit=Transaction / template_instances=Work TTL 30d / unlocks=Transaction)
- **#14 v3**: 5 域 Lead 拍板 D, Mavis 永久代签 (per 9/9 12:02 JST 升级, 真人到位流程暂时不追踪)
- **#19 v19**: 守门 #12 死循环饱和边界 (本轮 1 次新事件触发, 允许)

---

## 10. 签字栏 (per 守门 #14 v3 Mavis 永久代签)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| SRE Lead (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 平台 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| 评审主持 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |
| PM (Mavis 接手 agent per DEC-008) | 🟢 Mavis 永久代签 | 2026-09-09 |

---

## 11. 修订历史 (per 守门 #3 禁回溯叙事)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 21:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 module 设计完整化 (ArgCrate / SubAgentOrchestrator / LLMService / AgentLease / AgentRuntime) + 9 SA 完整化 (SA-01..SA-09) + 5 域 (player/economy/match/social/admin) 跨文档一致 + 20 业务功能 + 15 NFR + 5 想定シナリオ (含 S-06 任务卡合并) + 8 制約 (5 域跨域 + 9 SA 跨域 + 5 module 跨域 + 4 effect + RLS 13 类) + 9 SA 跨 SA 关系矩阵 + 9 SA × 5 module 映射表 | 2026-09-09 21:16 JST 用户发令"开子代理和 worktree 并行处理" + brief v0.45 §14.11 ARG.1 文档完整化 |
