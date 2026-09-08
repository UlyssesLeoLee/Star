# SRS-AGENT-RELATIONSHIP-001

> **Agent Relationship Graph (ARG) — 要件定義書 v0.1**
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联文档 (平行 view):
>   - `docs/architecture/2026-09-03-langgraph/` (LangGraph 統合アーキテクチャ, 9/3 落档 v0.2, 含 TMO 9 节点 / task 关系 DAG)
>   - `docs/architecture/2026-09-03-agent-runtime/` (STAR Agent Runtime, 9/3 落档, ADR-0045)
>   - `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (Agent Runtime SRS)
>   - `docs/requirements/SRS-AGENT-VIEW-001.md` v1.0 (Agent View, 9/5 落档, 画布节点 view)
>   - `docs/frontend-canvas-design.md` v0.1 (画布 + bezier connector)
> - 上位要件: `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 §4.4 (L0 ↔ L1 通信模型) + `SRS-AGENT-VIEW-001.md` §1.3 (节点渲染)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-08 JST
> - 受众: 详细設計工程师 / 架构审查者 / UI/UX 设计师 / SRE / 5 域 Lead 真人
> - 拍板来源: 2026-09-08 22:35 JST `ask_7d7ffcac2353adad7d3f6f69` (scope=新建 SRS-AGENT-RELATIONSHIP-001 / backend=Memgraph / 关系类型=4 核心 + 参考人类同事关系丰富化 / 成就系统=完整版 关系+协作行为+产出质量)

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-AGENT-RELATIONSHIP-001 |
| 文书名 | Agent Relationship Graph (ARG) 要件定義書 |
| 版本 | v0.1 |
| 作成日 | 2026-09-08 |
| 作成者 | Ulysses — Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) |
| 关联 commit | 待生成 (v0.1 落档后) |
| 关联文档 (parallel view) | `2026-09-03-langgraph/` + `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 + `SRS-AGENT-VIEW-001.md` v1.0 + `frontend-canvas-design.md` v0.1 |
| 上位文档 | `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 §4.4 |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档定义 STAR 平台 **Agent Relationship Graph (ARG)** 的需求规格说明书, 即**在 agent 节点之间引入关系层 (graph database edge)**, 让 agent 之间的关系:

1. **可定义** — 用户在 UI 上可视化建/编辑 agent 之间的关系
2. **可存储** — 关系持久化到图数据库 (Memgraph), 跨 session 持续
3. **可生效** — 关系真实影响 agent 之间的协作模式、token 分配、产出质量, 不止是"展示"标签
4. **可组合** — 通过不同关系配置的组合, 解锁丰富成就 (achievement), 鼓励用户探索

### 1.2 背景 (用户痛点)

STAR 平台已落地的 LangGraph 视图 (`2026-09-03-langgraph/01-requirements.md` v0.2) 已经定义**任务卡 (task card) 之间的 DAG 关系** (TMO M-N1..M-N7, 包含 merge / split / reorder / dep_set 等 9 节点), 用于"任务编排"。但用户在 `2026-09-08 22:35 JST` 反馈:

> "agent 界面内, 各个 agent 之间可以有图论数据库那种 edge, 可以设置 agent 之间的关系, 这种关系可以反映到它们之间的协作和工作内容中... 我希望 agent 之间的关系可以对它们的工作产生益处, 创造不同的 agents 团队, 通过不同团队配置的组合, 实现更加丰富的成就"

**核心问题**:
- 当前 LangGraph 视图只编排"任务", 不编排"agent 本身" — agent 是匿名的执行者, 之间的关系被压平
- Agent 之间没有 first-class 的关系概念, 只有"任务卡 DAG"间接体现调度
- 用户无法"组建团队" — 没有"Lead → Worker / Worker A ↔ Worker B 协作 / Worker 咨询 Reviewer" 等人类组织中常见的关系
- 没有激励用户探索"不同组合" 的机制 — 缺成就系统

**ARG 跟现有视图的关系**:
- **不取代** LangGraph 任务卡 DAG (TMO 9 节点), 那个是"任务编排"
- **不取代** Agent View 画布 (SRS-AGENT-VIEW-001), 那个是"个体 session 可视化"
- **新增第三层** — "agent 之间" 的关系层, 是 agent 维度的 social graph

### 1.3 包含范围 (In-Scope)

- ARG 节点类型: Agent 节点 (含 9 个 SA Archetype + 5 域 Lead 真人 + Custom Agent)
- ARG 关系类型: 4 核心 edge + 6 扩展 edge (参考人类同事关系, 共 10 类, 详见 §3.2)
- 图数据库: **Memgraph** (主存储, 跨 session 持久化)
- LangGraph 桥接: in-process subgraph 跟 Memgraph 双向同步 (per §4 同步协议)
- 关系→协作影响: 4 维度 (dispatch 路由 / 上下文共享 / 信任度 / 产出评估, 详见 §4.3)
- 团队模板: 5 个开箱即用模板 (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council)
- 成就系统: 3 维度 (拓扑 / 协作行为 / 产出质量, 共 ≥20 成就, MVP 10 + 扩展 10+)
- UI: Agent Relationship Editor (画布拖拽建关系) + Relationship View (图谱浏览) + Achievement Wall
- 跟 Agent View 集成: 已有 `agent-view` 画布扩展一个 tab 切到"关系层"
- API: REST + WebSocket (增删改关系 + 实时协作事件推送)
- 守门合规: 守门 #3 (5 域独立 Lead, 拒绝兼任) + 守门 #13 (W/T/M 三类横展开) + 守门 #14 v2 (5 域 Lead 决策 scope / RACI / 代签)

### 1.4 不包含范围 (Out-of-Scope)

- 真实 LLM 微调 (关系层不改模型权重, 只改 prompt 拼装 + dispatch 路由)
- 跨组织 agent 联邦 (本视图只覆盖 Star 仓内部 agent, 不跨仓跨平台)
- 关系的市场化交易 (no NFT / no token economy)
- Agent 自动学习关系 (本视图是"用户定义 + 协作行为反馈", 不做自动发现)
- 任务卡 DAG 改造 (那个归 TMO 9 节点, 本视图只读 task_relationships 字段, 不改)

### 1.5 用户故事

| 编号 | 角色 | 故事 | 优先级 |
|---|---|---|---|
| US-1 | 项目经理 (Ulysses) | 作为 PM, 我希望在 Agent View 里切换到"Relationship" 视角, 看到所有 agent 之间的图谱 (节点+边+标签), 一眼看出团队拓扑 | P0 |
| US-2 | PM | 作为 PM, 我希望拖拽两个 agent 创建一条 "delegates_to" 关系, 之后 Lead agent 自动把任务分给 Worker, 不需要再手动调 dispatch | P0 |
| US-3 | 5 域 Lead (真人) | 作为 Lead, 我希望定义"consults" 关系指向 Reviewer, 当我做关键决策时自动调 Reviewer 拿意见, 决策矩阵可追溯 | P0 |
| US-4 | Dev | 作为 Dev, 我希望给两个 Worker 设 "collaborates_with", 它们并行执行并合并产出, token OLU 节省 ≥20% | P0 |
| US-5 | Dev | 作为 Dev, 我希望用开箱即用的 "Hub-and-Spoke" 模板 1-click 创建 1 Lead + 4 Worker 团队, 不用挨个拖拽 | P1 |
| US-6 | PM | 作为 PM, 我希望"Reports To" 关系让 Worker 完成后自动汇总到 Lead 的 inbox, Lead 看到的是聚合报告 | P1 |
| US-7 | PM | 作为 PM, 我希望解锁成就 (如"跨 5 域全连接") 后在 Achievement Wall 看到徽章 + 描述, 激励探索 | P1 |
| US-8 | 5 域 Lead | 作为 Lead, 我希望关系定义被 audit log 记录, 改关系有版本历史, 真人到位后可追溯 | P1 |
| US-9 | SRE | 作为 SRE, 我希望关系影响 token OLU 计算, "collaborates_with" 团队的 token 消耗可观测 | P2 |
| US-10 | PM | 作为 PM, 我希望成就可分享 (导出 PNG + 描述), 用于团队周报 | P2 |

---

## §2 用语定义 (Ubiquitous Language)

| 用语 | 定义 | 出处 |
|---|---|---|
| **Agent Relationship Graph (ARG)** | Agent 节点 + 关系边的有向图, 存储在 Memgraph, 跨 session 持久 | 本 SRS 新增 |
| **Agent 节点 (Agent Node)** | 一个可执行实体, 类型 ∈ {SA-01..SA-09 9 个 Archetype, 5 域 Lead 真人, Custom Agent} | per [SRS-Runtime §2.3](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) |
| **关系边 (Relationship Edge)** | 两个 agent 之间的有向关系, 类型 ∈ 4 核心 + 6 扩展 = 10 类 (详见 §3.2) | 本 SRS 新增 |
| **4 核心关系** | delegates_to / consults / collaborates_with / reports_to (用户拍板 9/8 22:35) | per ask_7d7ffcac 选项 |
| **6 扩展关系** | mentors / peer_reviews / stand_in_for / shadows / challenges / trusts (参考人类同事关系丰富化) | 本 SRS 新增 (per ask_7d7ffcac other) |
| **Memgraph** | 开源图数据库, 用 Cypher 查询, 支持事务/索引/触发器, Docker 一键启动 | 本 SRS 选型 (per ask_7d7ffcac 推荐) |
| **In-process StateGraph** | LangGraph 进程内图状态, 跟 Memgraph 双向同步, 跟 [LangGraph 02 §2.1](../../architecture/2026-09-03-langgraph/02-basic-design.md) StateGraph 同形 | 本 SRS 新增桥接 |
| **同步桥 (Sync Bridge)** | Memgraph 写 → 触发 in-process StateGraph update, in-process 状态变 → 周期 flush 到 Memgraph | 本 SRS 新增 (per §4.4) |
| **关系→协作影响 (Edge Effect)** | 关系边不是标签, 会真实影响 dispatch 路由 / 上下文共享 / 信任度 / 产出评估 | 本 SRS 新增 (per §4.3, 用户原话"对它们的工作产生益处") |
| **团队模板 (Team Template)** | 预定义拓扑 + 关系配置, 1-click 部署, 5 个开箱即用 | 本 SRS 新增 (per §5.2) |
| **成就 (Achievement)** | 关系+协作+产出 3 维度触发条件解锁, 含徽章 + 描述 + 稀有度 | 本 SRS 新增 (per §6) |
| **关系 audit log** | 关系的增删改版本历史, append-only, per 守门 #13 Transaction 表 | 本 SRS 新增 (per §7.2) |
| **关系权重 (Edge Weight)** | 边的强度/置信度, 0.0-1.0, 影响调度优先级 (per §4.3.4) | 本 SRS 新增 |
| **信任度 (Trust Score)** | 累计协作成功率 × edge weight, 0.0-1.0, 越高的 agent 越被优先 dispatch | 本 SRS 新增 (per §4.3.3) |
| **Active Relationship** | 当前活跃的关系 (不是 archived), 只有 active 的关系影响协作 | 本 SRS 新增 |

---

## §3 业务背景 / 前提条件

### 3.1 业务背景

STAR 平台从 2026-08 起逐步落地 LangGraph (9/3 v0.2) + Agent Runtime (9/3) + Agent View (9/5) 三层架构, 但**所有视图都聚焦"个体 agent + 任务"**, 缺一个**"agent 之间的 social 层"**。

人类组织中常见的同事关系 (mentor-mentee / peer reviewer / stand-in / cross-domain consult / challenge-and-defend) 在 AI agent 团队中也有强烈需求, 但当前架构缺这一层抽象。

用户 9/8 22:35 JST 明确要求"参考人类社会中真实的同事关系, 设计丰富的关系" + "通过不同团队配置的组合, 实现更加丰富的成就", 这要求 ARG 不仅是"图数据库存储", 还要有**游戏化激励层**。

### 3.2 前提条件

| # | 前提 | 影响 |
|---|---|---|
| **PR-1** | LangGraph 任务卡 DAG 已经在 TMO 9 节点 (M-N1..M-N7) 里实现, 9/4 落档 | ARG 跟 TMO 是平行层, 不重复 |
| **PR-2** | Agent Runtime 已经有 9 个 SA Archetype (SA-01..SA-09), 9/3 落档 (per ADR-0045) | ARG 节点类型直接复用 SA Archetype |
| **PR-3** | 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B) | ARG 跟代签机制联动, 关系定义由 Mavis 默认落 |
| **PR-4** | Memgraph 用 Docker 启动, port 7687 (Bolt) + 7444 (HTTP), 数据卷持久化 | ARG 部署需要先起 Memgraph container |
| **PR-5** | 守门 #13 W/T/M 三类横展开已经落地 (per AGENTS.md §4 守门 #13) | ARG 的关系 audit log 必须是 Transaction 类, 节点 metadata 走 Master 类 |
| **PR-6** | gm-console frontend 已有无限画布 (per `frontend-canvas-design.md` v0.1) | ARG 编辑器复用画布组件 + 加边编辑能力 |
| **PR-7** | 守门 #9 实证子代理 RPC 不可靠 (per AGENTS.md §4 #9 主体) | ARG 同步桥用 in-process 推 + 周期 flush, 不用 RPC |

---

## §4 业务需求

### 4.1 关系类型设计 (4 核心 + 6 扩展)

#### 4.1.1 4 核心关系 (per 用户拍板 9/8 22:35)

| 关系类型 | 语义 | 协作影响 | 典型场景 |
|---|---|---|---|
| **delegates_to** (A → B) | A 把任务委派给 B, A 不参与执行 | B 收到任务, 完成后回报 A; A 的 token 节省 | Lead 把任务分给 Worker |
| **consults** (A → B) | A 关键决策时调 B 拿意见, B 给建议不执行 | A 的 prompt 注入"参考 B 视角", B 不消耗执行 token | 跨域 Lead 评审 |
| **collaborates_with** (A ↔ B, 无向) | A B 并行执行, 共同产出 | 双方并行跑, 结果 merge, 节省 wall-clock time | 双 agent 并行 review |
| **reports_to** (A → B) | A 的执行结果汇总到 B, B 决策/聚合 | A 完成后自动推送到 B 的 inbox, B 看到聚合报告 | Worker → Lead 汇报 |

#### 4.1.2 6 扩展关系 (per ask_7d7ffcac other "参考人类同事关系丰富化")

| 关系类型 | 语义 | 协作影响 | 典型场景 |
|---|---|---|---|
| **mentors** (A → B) | A 长期指导 B 成长, B 风格/能力受 A 影响 | B 启动时拉 A 的历史决策作为 context, 学习迁移 | Senior Lead 指导 Junior Worker |
| **peer_reviews** (A ↔ B, 无向) | A B 互相 review 对方的产出, 提升质量 | 完成后自动触发 review, 产出需 ≥ 阈值才通过 | Code review 配对 |
| **stand_in_for** (A → B) | A 故障时 B 接管, 备份关系 | A 状态 = failed/cancelled 自动 fallback 到 B | 关键 agent 备份 |
| **shadows** (A → B) | A 静默观察 B 执行, 不影响, 仅学习 | A 拉 B 的执行流作为 case study, 写入 A 的 knowledge base | 新人 onboarding 观察老员工 |
| **challenges** (A → B) | A 主动质疑/反驳 B 的决策, 强制双向论证 | B 必须先自证再让 A 接受, 提升决策严谨性 | Reviewer 主动挑战 |
| **trusts** (A → B) | A 信任 B 的产出, 跳过 review 直接采用 | B 的产出不走 verify 节点, 节省 token + 加速 | 高度信任的快捷通道 |

**总计 10 类关系** (4 核心 + 6 扩展), 足够覆盖人类组织中的常见关系 (领导/汇报/协作/师徒/评审/备份/挑战/信任)。

#### 4.1.3 关系属性

每条边有以下属性:

| 属性 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `id` | UUID | ✓ | 边唯一 ID |
| `from_agent` | UUID | ✓ | 源 agent |
| `to_agent` | UUID | ✓ | 目标 agent |
| `type` | enum (10) | ✓ | 关系类型 |
| `weight` | float 0.0-1.0 | ✓ | 关系强度, 默认 0.5 |
| `direction` | enum {directed, undirected} | ✓ | 4 核心里 collaborates_with + 6 扩展里 peer_reviews 是 undirected, 其余 8 个是 directed |
| `created_at` | timestamp | ✓ | 创建时间 |
| `created_by` | UUID | ✓ | 创建者 (Mavis / Ulysses / 5 域 Lead 真人) |
| `archived` | bool | ✓ | 是否归档, archived 关系不影响协作 |
| `version` | int | ✓ | 关系定义版本, 改一次 +1 (per 守门 #13 Transaction SCD Type 2) |
| `metadata` | JSON | ✗ | 扩展字段, 如"信任阈值 / review checklist / 协作 SLA" |

### 4.2 Memgraph 图模型

#### 4.2.1 节点 (Vertex)

```cypher
// Agent 节点
CREATE (:Agent {
  id: UUID,                    // 唯一 ID
  name: STRING,                // 展示名 (e.g. "Player-Lead-A")
  archetype: STRING,           // SA-01..SA-09 / LEAD-{DOMAIN} / CUSTOM
  domain: STRING,              // 5 域之一 (player/economy/match/social/admin) 或 null
  status: STRING,              // active / standby / archived
  trust_score: FLOAT,          // 0.0-1.0, 动态
  metadata: JSON,              // 扩展字段
  created_at: DATETIME,
  updated_at: DATETIME,
  version: INT                 // SCD Type 2
});

// 索引
CREATE INDEX ON :Agent(id);
CREATE INDEX ON :Agent(archetype);
CREATE INDEX ON :Agent(domain);
```

#### 4.2.2 边 (Edge) — 10 类关系全部有向

```cypher
// 4 核心关系 (示例: delegates_to)
CREATE (a:Agent {id: 'A'})-[r:DELEGATES_TO {
  id: UUID,
  weight: 0.7,
  archived: false,
  created_at: DATETIME,
  created_by: UUID,
  version: 1,
  metadata: JSON
}]->(b:Agent {id: 'B'});

// 6 扩展关系 (示例: mentors)
CREATE (a:Agent {id: 'A'})-[r:MENTORS {
  ... 同样的属性集
}]->(b:Agent {id: 'B'});

// 索引 (per 关系类型)
CREATE INDEX ON :DELEGATES_TO(from_agent);
CREATE INDEX ON :REPORTS_TO(to_agent);
```

#### 4.2.3 复合索引 (per 守门 #13 W/T/M)

- **Master** (`agents` 表) — SCD Type 2, 100% RLS 13 类
- **Transaction** (`agent_relationship_edges_audit` 表) — append-only, 100% audit, 改关系必+1 行
- **Work** (`team_template_instances` 表) — 短 TTL 30 天, 用户主动清理或转 Master

### 4.3 关系→协作影响机制 (4 维度) — 核心需求

> **用户原话**: "我希望 agent 之间的关系可以对它们的工作产生益处" — 关系**必须**影响协作, 不能是"展示"标签

| 维度 | 关系类型映射 | 实际效果 | 度量指标 |
|---|---|---|---|
| **4.3.1 Dispatch 路由** | delegates_to, reports_to, stand_in_for | Lead 收到任务后按"出度边"自动 dispatch 给下属; Worker 故障按 stand_in_for fallback | dispatch 路由命中率, fallback 触发率 |
| **4.3.2 上下文共享** | mentors, shadows | mentee 启动时拉 mentor 的历史决策; shadow agent 拉被执行 agent 的输出 | 上下文命中率, token 节省率 |
| **4.3.3 信任度加权** | trusts, peer_reviews | 信任度高的 agent 跳过 verify 节点, 加速 | verify 跳过率, 产出失败率 |
| **4.3.4 产出评估** | challenges, peer_reviews | challenges 强制双向论证; peer_reviews 双向 review 阈值 ≥0.8 | review 覆盖率, 一次通过率 |

#### 4.3.1 Dispatch 路由

- `delegates_to` 边让 L0 Top Agent 收到任务时, 按"出度边"自动 spawn L1 sub-agent
- `reports_to` 边让 L1 sub-agent 完成后, 状态推送目标 agent 的 inbox
- `stand_in_for` 边让 A failed 时, 自动 fallback 到 B, 任务不中断

**实现路径**: L0 dispatch_node 加载 ARG 后, 改写默认 dispatch 逻辑, 优先按关系边 spawn

#### 4.3.2 上下文共享

- `mentors` 关系: mentee agent 启动时, 拉 mentor agent 的最近 10 次 decision 入 context
- `shadows` 关系: shadow agent 静默订阅被观察 agent 的 event stream
- 节省 token: 共享 context 不重复加载, 减少输入 token 20-40%

#### 4.3.3 信任度加权

- `trusts` 关系: weight ≥ 0.8 的 trusts 边让 B 的产出跳过 verify 节点
- `peer_reviews` 关系: weight ≥ 0.7 + 协作 ≥ 5 次, 自动升级为 trusts
- 信任度动态: 每次成功协作 +0.01, 失败 -0.05, 范围 [0.0, 1.0]

#### 4.3.4 产出评估

- `challenges` 关系: B 必须先 self-justify 再让 A 接受, A 不接受则 escalate
- `peer_reviews` 关系: 双向 review, 产出评分 ≥ 0.8 才发布
- 评估标准可配置 (per `metadata.review_threshold` 字段)

### 4.4 同步桥 (Memgraph ↔ LangGraph StateGraph)

```
[UI: 关系编辑]  ──HTTP──>  [API: /api/arg/edges]  ──>  [Memgraph write]
                                                       │
                                                       ▼
                                            [EventBus: edge.changed] (in-process)
                                                       │
                                                       ▼
                                  [LangGraph StateGraph: top_state.arg_edges update]
                                                       │
                                                       ▼
                                            [Period flush (30s)] ──> [Memgraph read]
```

- **写路径**: UI → API → Memgraph → 触发 in-process EventBus → StateGraph 更新
- **读路径**: LangGraph 执行时读 in-process StateGraph 优先, fallback Memgraph
- **冲突解决**: 最后写胜出 (LWW), version 字段做 optimistic lock
- **离线降级**: Memgraph 不可达时, in-process 缓存继续工作, 重连后 flush

### 4.5 团队模板 (5 个开箱即用)

| 模板 | 拓扑 | 适用场景 | 节点数 |
|---|---|---|---|
| **Hub-and-Spoke** | 1 Lead ↔ 4 Worker (4 条 delegates_to) | 标准团队, 1 Lead 调度 4 Worker | 5 |
| **Mesh** | N 节点全连接 (N×(N-1)/2 条 collaborates_with) | 高密度协作, 跨域评审 | 5-10 |
| **Chain** | A → B → C → D (3 条 delegates_to) | 流水线型任务, 4 步连续 | 4 |
| **Hierarchical** | 1 Lead → 2 Sub-Lead → 6 Worker (树形) | 多层管理, 5 域 Lead 真人到位后 | 9 |
| **Review-Council** | 3 Reviewer ← 1 Lead (1 deleg + 3 consults) | 重大决策需 3 评审 | 4 |

模板支持"实例化": 用户点模板 → 选 agent → 1-click 部署, 自动创建节点+边

---

## §5 约束条件 (Constraints)

### 5.1 守门合规 (per AGENTS.md §4)

| 守门 | 约束 | ARG 落地 |
|---|---|---|
| #1 (R-05 不 push 已反转) | git push 守门 | ARG 文档同步 commit 必先跑守门 |
| #3 (5 域独立 Lead) | 5 域 Lead 拒绝兼任 | ARG 关系定义时, 跨域边强制 consults 而非 delegates_to |
| #5 (env 安全) | 不打印 env | Memgraph 连接字符串走 env, 不打印 |
| #6 (PowerShell only) | 守门 | ARG 部署脚本 PowerShell |
| #7 (0 unsafe) | 守门 | Memgraph 客户端 crate 0 unsafe |
| #9 (子代理 RPC) | 实证不可靠 | ARG 同步走 in-process 推 + 周期 flush (per §4.4), 不用 RPC |
| #10 (代签规则) | Mavis 默认代 Ulysses | ARG 关系修改 author = Ulysses per 9/8 15:19 第 6 次强化 |
| #13 (W/T/M 三类) | 横展开强制 | agents = Master, audit = Transaction, template instances = Work (per §4.2.3) |
| #14 v2 (5 域 Lead 拍板 D) | Mavis 临时代签, 真人到位后追溯 | ARG 5 域 Lead 关系定义由 Mavis 落, 真人到位后追溯签字 |

### 5.2 技术约束

| 约束 | 说明 |
|---|---|
| **Memgraph 版本** | ≥ 2.14, 用 Docker 启动, port 7687 Bolt + 7444 HTTP |
| **Cypher 兼容** | 100% Memgraph Cypher, 不依赖 Neo4j-only feature (per 选型决策) |
| **客户端** | Rust crate `memgraph-client` (待开发) 或 `r2d2-memgraph` (待调研) |
| **API 协议** | REST (CRUD) + WebSocket (实时事件) |
| **前端** | React + TypeScript + Zustand (per Agent View 既有栈) |
| **数据迁移** | 跨 Memgraph 升级, 用 mgconsole 导出 Cypher + 导入 |

### 5.3 业务约束

- **不取代** LangGraph 任务卡 DAG (TMO 9 节点), 两个图并存
- **不取代** Agent View 画布 (SRS-AGENT-VIEW-001), 是其 1 个 tab 视角
- **不引入** istio / nginx 边缘代理 (per 9/1 13:03 JST envoy 偏好, ARG 内部通信走 in-process 即可)
- **不引入** OpenAI / Anthropic 第三方 API (per 守门 #5 v2 + #23, 关系定义 LLM 用 mock, 不用外部)
- **5 域 Lead 真人到位前**所有关系由 Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2)

---

## §6 业务场景 (Use Cases)

### UC-01: 可视化建关系 (UI 拖拽)

- **Actor**: Primary User (Ulysses)
- **触发**: Agent View → 切换到 "Relationship" tab → 拖拽两个 agent
- **Flow**:
  1. UI 加载 ARG 视图, 显示现有节点+边
  2. 用户从 agent A 拖拽到 agent B, 弹出关系类型选择器
  3. 用户选 "delegates_to", 填 weight=0.7
  4. UI → POST /api/arg/edges → Memgraph write
  5. Memgraph 触发 EventBus → LangGraph StateGraph update
  6. UI 实时显示新边
- **Postcondition**: 关系持久化到 Memgraph, in-process 状态同步

### UC-02: 委派 (delegates_to) 自动 dispatch

- **Actor**: Top Agent + Lead Agent + Worker Agent
- **触发**: Lead 收到任务 (per `delegates_to` 边的 out-degree)
- **Flow**:
  1. Lead 收到用户 chat bar 输入
  2. Lead 的 LangGraph 加载 ARG, 发现 2 条 delegates_to 边 (指向 Worker A, B)
  3. Lead 改写 dispatch 逻辑, 自动 spawn Worker A + B
  4. Worker A, B 并行执行
  5. Worker A, B 完成后回报 Lead
  6. Lead 聚合, 反馈给用户
- **Postcondition**: 用户无感, 任务被自动分给 2 个 Worker, Lead 不参与执行

### UC-03: 咨询 (consults) 决策辅助

- **Actor**: Lead Agent + Reviewer Agent
- **触发**: Lead 遇到关键决策点 (per `consults` 边)
- **Flow**:
  1. Lead 走到 decision 节点
  2. Lead 的 LangGraph 加载 ARG, 发现 1 条 consults 边指向 Reviewer
  3. Lead 的 prompt 注入"参考 Reviewer 视角, Reviewer 历史: ..."
  4. Lead 调 Reviewer (sub-call, 不 spawn sub-agent)
  5. Reviewer 给建议, Lead 综合决策
- **Postcondition**: 决策综合 2 个视角, 决策矩阵可追溯

### UC-04: 协作 (collaborates_with) 并行+merge

- **Actor**: 2 个 Worker A, B
- **触发**: 用户输入 "用 2 个 agent 同时 review 这段代码"
- **Flow**:
  1. Top Agent 解析 → spawn Worker A, B (per `collaborates_with` 边)
  2. Worker A, B 并行执行 (不同 LLM model, 不同视角)
  3. 完成后结果 merge (per 同步桥, 自动)
  4. 产出是 A + B 共识 (intersection + union)
- **Postcondition**: token OLU 节省 ~20-30% (并行 wall-clock), 产出质量 +10-20% (双视角)

### UC-05: 模板实例化 (Hub-and-Spoke)

- **Actor**: Primary User
- **触发**: UI 模板库 → 选 "Hub-and-Spoke" → 选 5 个 agent
- **Flow**:
  1. UI 弹出 agent 选择器 (5 个 dropdown)
  2. 用户选 1 Lead + 4 Worker
  3. UI → POST /api/arg/templates/instantiate (template_id, agent_ids)
  4. 后端批量创建 4 条 delegates_to 边
  5. 1 事务提交到 Memgraph
  6. UI 显示新团队拓扑
- **Postcondition**: 1-click 创建 1 Lead + 4 Worker 团队, 立即可用

### UC-06: 成就解锁 (跨 5 域全连接)

- **Actor**: 系统 (Achievement Engine) + User
- **触发**: 用户配置了 5 域各 1 个 agent, 全部互相 connects (5×4/2 = 10 条 undirected 边)
- **Flow**:
  1. Memgraph 写入 10 条边
  2. EventBus 触发 achievement.evaluator
  3. evaluator 查 "跨 5 域全连接" 成就定义 (topology-based)
  4. 命中 → 解锁, 写入 Transaction 表
  5. UI: Achievement Wall 弹通知 + 徽章
- **Postcondition**: 成就记录到 audit log, 可分享

### UC-07: 关系权重动态调整

- **Actor**: 系统 (Trust Engine) + User
- **触发**: 协作成功 1 次, weight 自动 +0.01
- **Flow**:
  1. Worker A 完成 + Worker B 协作 (per `collaborates_with` 边, 当前 weight=0.5)
  2. 协作成功 event → Trust Engine 计算
  3. weight 0.5 → 0.51
  4. Memgraph update edge (version +1)
  5. UI: 边颜色从灰变绿
- **Postcondition**: 边强度反映协作历史

### UC-08: Stand-in fallback

- **Actor**: Worker A + Worker B (per `stand_in_for` 边)
- **触发**: Worker A 状态变 failed (LLM rate limit)
- **Flow**:
  1. Worker A 抛错
  2. 错误处理节点查 ARG, 发现 stand_in_for → Worker B
  3. 自动 spawn Worker B 接管任务 (复制 A 的 context)
  4. Worker B 完成, 任务不中断
  5. UI: 通知 A failed, B 接管
- **Postcondition**: 任务不中断, user 感知降级提示

---

## §7 功能性需求 (Functional Requirements)

### 7.1 数据需求

| 表 | 类型 (W/T/M) | 字段 | 索引 |
|---|---|---|---|
| `agents` | Master | id, name, archetype, domain, status, trust_score, metadata, created_at, updated_at, version | id, archetype, domain |
| `agent_relationship_edges` | Master | id, from_agent, to_agent, type, weight, direction, archived, metadata, version | id, from_agent, to_agent, type, composite (from+to+type+archived) |
| `agent_relationship_edges_audit` | Transaction | id, edge_id, action (create/update/archive), old_value, new_value, actor, timestamp | edge_id, timestamp |
| `team_template_instances` | Work (TTL 30 天) | id, template_id, instance_name, agent_ids, edges_json, created_at, expires_at | instance_name, expires_at |
| `achievements` | Master | id, code, name, description, category, rarity, icon_url | code, category |
| `achievement_unlocks` | Transaction | id, achievement_id, user_id, agent_ids, trigger_metadata, unlocked_at | user_id, unlocked_at |
| `relationship_events` | Transaction | id, edge_id, event_type, payload, timestamp | edge_id, timestamp |

### 7.2 API 需求

| Method | Path | 说明 | 守门 |
|---|---|---|---|
| `POST` | `/api/arg/agents` | 创建 agent | W/T/M 严格 |
| `GET` | `/api/arg/agents` | 列表 agent | 分页 + 过滤 |
| `PATCH` | `/api/arg/agents/{id}` | 更新 agent (SCD Type 2, version +1) | Master |
| `POST` | `/api/arg/edges` | 创建关系 | 必填 11 字段 |
| `GET` | `/api/arg/edges` | 列表关系 | 过滤 type/agent |
| `PATCH` | `/api/arg/edges/{id}` | 更新关系 (weight/metadata) | Master |
| `DELETE` | `/api/arg/edges/{id}` | 归档关系 (archived=true) | append-only audit |
| `GET` | `/api/arg/graph` | 整图查询 (Cypher) | 限频 |
| `POST` | `/api/arg/templates/instantiate` | 模板实例化 | 1 事务 |
| `GET` | `/api/arg/achievements` | 成就列表 | 分页 |
| `GET` | `/api/arg/achievements/me` | 我的解锁成就 | 用户维度 |
| `POST` | `/api/arg/achievements/evaluate` | 触发评估 (admin only) | 幂等 |
| `WS` | `/ws/arg/events` | 实时事件推送 (edge.changed / achievement.unlocked) | auth required |

### 7.3 性能需求

| 指标 | 目标 |
|---|---|
| 边创建 latency | P95 < 200ms (Memgraph Bolt) |
| 图查询 (Cypher) | P95 < 500ms (≤ 1000 节点) |
| 实时事件推送 | < 100ms (in-process EventBus) |
| 关系→协作 dispatch 路由命中 | ≥ 80% (per 关系自动派发占总 dispatch 比例) |
| 信任度跳过 verify 节省 token | ≥ 15% (per trusts 边启用) |
| 协作并行 wall-clock 节省 | ≥ 20% (per collaborates_with 启用) |
| 成就评估 latency | P95 < 1s (1 万边规模) |

---

## §8 非功能需求 (NFR)

### NFR-ARG-PERF-01: 性能

- 单图支持 ≥ 1000 节点 + 5000 边, 查询 P95 < 500ms
- 实时事件推送 < 100ms, 不阻塞主 dispatch 流程
- 周期 flush 30s, 不影响用户体验

### NFR-ARG-RELIABILITY-01: 可靠性

- Memgraph 不可达时, in-process 缓存继续工作 (per §4.4 离线降级)
- 重连后自动 flush, 不丢关系变更
- 边创建 100% 持久化 (Master + Transaction 双写)

### NFR-ARG-SECURITY-01: 安全

- RLS 13 类必携 (per 守门 #13), tenant_id 隔离
- 关系修改 audit log 必记 (per 守门 #13 Transaction)
- Memgraph 连接字符串走 env (per 守门 #5)

### NFR-ARG-USABILITY-01: 易用

- 1-click 模板实例化, ≤ 30s 部署
- 拖拽建边, ≤ 3 次点击完成
- 成就通知 ≤ 1s 弹出

### NFR-ARG-ACHIEVEMENT-01: 成就系统 (完整版, per 用户拍板 9/8 22:35)

成就 = 3 维度组合触发, 共 ≥ 20 个:

| 维度 | 类型 | 数量 | 例子 |
|---|---|---|---|
| **拓扑 (Topology)** | 静态图结构触发 | 8 | "跨 5 域全连接" / "Hub-and-Spoke 模板" / "Mesh 10 节点" / "Chain 5 步" / "Hierarchical 3 层" / "Review-Council 4 节点" / "自环检测" / "孤岛检测" |
| **协作行为 (Behavior)** | 运行时事件触发 | 7 | "首次 delegates_to dispatch" / "consults 决策辅助 100 次" / "collaborates_with 并行 50 次" / "stand_in_for 接管 5 次" / "peer_reviews 100% 一次通过" / "challenges 反驳 3 次" / "shadows 观察 24h" |
| **产出质量 (Output)** | 协作产出指标触发 | 5 | "trusts 跳过 verify 节省 100K token" / "协作并行节省 1h wall-clock" / "5 域 Lead 共识达成" / "0 失败 100 次协作" / "成就链 (5 个成就连环解锁)" |

**总计 20 成就, MVP 10 + 扩展 10+**, 每个含:
- `code` (唯一 ID, e.g. "TOP-001-MESH-5DOMAIN")
- `name` (中文 + 英文双语)
- `description` (解锁条件 + 益处说明)
- `category` (TOPOLOGY / BEHAVIOR / OUTPUT)
- `rarity` (COMMON / RARE / EPIC / LEGENDARY)
- `icon_url` (徽章图)
- `unlock_condition` (Cypher 查询 或 事件 pattern)

### NFR-ARG-OBSERVABILITY-01: 可观测

- 关系增删改 100% audit (Transaction 表)
- 关系→协作影响指标 (dispatch 命中率, token 节省率) Prometheus 导出
- 成就解锁事件 SSE 推送

---

## §9 验收标准 (Acceptance Criteria)

### AC-1 (P0): 4 核心关系可创建+可视化

- [ ] UI 拖拽 2 agent, 选 delegates_to, 创建边
- [ ] Memgraph 验证边存在
- [ ] LangGraph StateGraph 验证边已加载
- [ ] 边可视化显示 (颜色+箭头+标签)
- [ ] 跨 session 重启, 边持久化

### AC-2 (P0): 关系影响 dispatch (delegates_to)

- [ ] Lead agent 收到任务, 按 delegates_to 边自动 spawn Worker
- [ ] Worker 完成后自动回报 Lead
- [ ] Lead 不参与执行, token 节省 ≥ 30%

### AC-3 (P0): 关系影响决策 (consults)

- [ ] Lead 关键决策时按 consults 边调 Reviewer
- [ ] Reviewer 给建议, Lead 综合决策
- [ ] 决策矩阵可追溯 (含 Reviewer 视角)

### AC-4 (P0): 关系影响并行 (collaborates_with)

- [ ] 2 agent 协作, 并行执行
- [ ] 完成后结果自动 merge
- [ ] wall-clock 节省 ≥ 20% (vs 串行)

### AC-5 (1): 5 模板可实例化

- [ ] Hub-and-Spoke 模板 1-click 部署 1 Lead + 4 Worker
- [ ] Mesh 模板 N 节点全连接
- [ ] Chain 模板 A→B→C→D 流水线
- [ ] Hierarchical 模板 3 层管理
- [ ] Review-Council 模板 1 Lead + 3 Reviewer

### AC-6 (1): 6 扩展关系可创建+生效

- [ ] mentors 关系: mentee 拉 mentor 历史决策
- [ ] peer_reviews 关系: 双向 review, 阈值 ≥ 0.8
- [ ] stand_in_for 关系: agent 失败自动 fallback
- [ ] shadows 关系: 静默观察
- [ ] challenges 关系: 强制双向论证
- [ ] trusts 关系: 跳过 verify, 节省 token

### AC-7 (1): 成就系统 3 维度 ≥ 20 成就

- [ ] 8 拓扑成就可触发
- [ ] 7 协作行为成就可触发
- [ ] 5 产出质量成就可触发
- [ ] Achievement Wall UI 显示已解锁 + 稀有度
- [ ] 成就解锁事件 SSE 推送

### AC-8 (1): 关系 audit log 完整

- [ ] 关系增删改 100% 记录到 Transaction 表
- [ ] 改关系 version +1 (SCD Type 2)
- [ ] 改关系可回滚 (per audit log)

### AC-9 (2): 5 域 Lead 真人到位后追溯签字

- [ ] 真人 Lead 接管后, 关系定义可追溯到 Mavis 临时代签
- [ ] 真人签字覆盖修订历史
- [ ] 5 域 Lead RACI 关系表 (per 守门 #14 v2 CONTENT 4 维)

---

## §10 已知缺口 / 未来扩展 (Known Gaps)

| # | 缺口 | 影响 | 后续阶段 |
|---|---|---|---|
| **G-1** | **Memgraph 客户端 crate 缺** — `memgraph-client` / `r2d2-memgraph` 待调研/开发 | 实装阶段需先做客户端 | P3-C 实装阶段 |
| **G-2** | **Memgraph 部署** — Docker compose 模板未写, k3s 部署 yaml 未写 | 实装阶段需先部署 | P3-C 实装阶段 |
| **G-3** | **L0 ↔ L1 通信协议 + ARG 集成** — LangGraph StateGraph 怎么加载 ARG 边未设计 | 关系→协作影响无法落地 | P3-D 基本设计 |
| **G-4** | **trusts 跳过 verify 的安全审计** — 跳过 verify 后, 失败风险怎么控制待评估 | 节省 token 但增加风险 | DDD Review |
| **G-5** | **挑战关系 (challenges) 的双向论证 prompt 模板** 待写 | 实装阶段 prompt 模板 | P3-D 详细设计 |
| **G-6** | **成就评估的 Cypher 模板** 待写 8 个拓扑成就查询 | 实装阶段需先写 evaluator | P3-C 实装阶段 |
| **G-7** | **跨 session 持久化的 Memgraph 高可用** — 单点故障, 待加 replica set | 生产化待补 | 后续阶段 |
| **G-8** | **5 域 Lead 真人到位 timeline** 待 DDD Review 拍板 (per 9/3 19:43 守门 #14 v2) | 关系代签机制长期维持 | 真人到位时 |
| **G-9** | **跟 TMO 9 节点的边界** — 任务卡 DAG 跟 agent relationship graph 重叠场景待梳理 | 两个图都涉及"边"概念 | DDD Review |
| **G-10** | **成就可分享的导出格式** (PNG / 描述 JSON) 未设计 | US-10 实现 | P3-D 详细设计 |
| **G-11** | **ARG Schema V2 迁移路径** — V1 → V2 加新关系类型时怎么处理存量数据 | 未来扩展 | 后续阶段 |
| **G-12** | **ARG 跟 RGS 仓的独立边界** (per AGENTS.md §5 仓库拓扑) — ARG 是 Star 仓独立视图, 不引用 RGS 5 域镜像作为源头 | 命名解读合规 | 持续 |

---

## §11 签字栏 (5 角色 per AGENTS.md §3 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 |
| **SRE Lead** | ⏳ 待真人到位 (per 守门 #14 v2) | — |
| **平台** | ⏳ 待真人到位 (per 守门 #14 v2) | — |
| **评审主持** | ⏳ 待真人到位 (per 守门 #14 v2) | — |
| **PM** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 9/8 15:19 第 6 次强化) | 2026-09-08 |

> **派生规 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B)**: 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯签字覆盖修订历史

---

## §12 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版落档 — 4 核心 + 6 扩展 = 10 类关系 (per ask_7d7ffcac 选项 + other "参考人类同事关系丰富化"); Memgraph 后端选型 (per 拍板 backend_opt1); 4 维度关系→协作影响 (dispatch 路由 / 上下文共享 / 信任度 / 产出评估); 5 团队模板 (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council); 20 成就 3 维度 (8 拓扑 + 7 协作行为 + 5 产出质量, per 拍板 achievement_opt2 完整版); 守门 #1+#3+#5+#6+#7+#9+#10+#13+#14 v2 全过 (文档工作, 守门 #1 v25 cargo test --workspace -j 4 不需要跑); 12 已知缺口待 DDD Review + 后续阶段; 守门 #1 v15 docs 同步触达饱和确认: 本次有新事件触发 (用户发令 9/8 22:35), 不算饱和违规 | 2026-09-08 22:35 JST `ask_7d7ffcac2353adad7d3f6f69` 用户拍板 (scope=新建 SRS / backend=Memgraph / 关系=4 核心 + 人类同事丰富化 / 成就=完整版) + Ulysses 9/8 22:35 JST 用户发令"agent 之间图论数据库 edge, 关系影响协作, 创造团队, 丰富成就" |
