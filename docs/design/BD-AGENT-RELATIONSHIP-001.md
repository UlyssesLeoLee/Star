# BD-AGENT-RELATIONSHIP-001

> **Agent Relationship Graph (ARG) — 基本設計書 v0.1**
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-AGENT-RELATIONSHIP-001.md`](../requirements/SRS-AGENT-RELATIONSHIP-001.md) v0.1 (9/8 落档, 663 行)
> - 关联詳細設計: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](./DD-AGENT-RELATIONSHIP-001.md) (待 P3-D 阶段落档)
> - 关联実装報告: [`docs/reports/PHASE-ARG-IMPL-REPORT.md`](../reports/PHASE-ARG-IMPL-REPORT.md) (待 P3-C 阶段落档)
> - 平行 view:
>   - [`docs/architecture/2026-09-03-langgraph/02-basic-design.md`](../architecture/2026-09-03-langgraph/02-basic-design.md) (LangGraph v0.2 含 TMO 9 节点 / StateGraph 桥接, 9/4 落档)
>   - [`docs/architecture/2026-09-03-agent-runtime/02-basic-design.md`](../architecture/2026-09-03-agent-runtime/02-basic-design.md) (Agent Runtime, 9/3 落档, ADR-0045)
>   - [`docs/requirements/SRS-AGENT-VIEW-001.md`](../requirements/SRS-AGENT-VIEW-001.md) v1.0 + [`docs/design/BD-AGENT-VIEW-001.md`](./BD-AGENT-VIEW-001.md) (Agent View 画布, 9/5 落档)
>   - [`docs/frontend-canvas-design.md`](../frontend-canvas-design.md) v0.1 (无限画布 + bezier connector 公式)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-08 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / UI/UX 设计师 / アーキテクト / SRE
> - 拍板来源: 2026-09-08 22:35 JST `ask_7d7ffcac2353adad7d3f6f69` (scope / backend=Memgraph / 关系=4 核心+6 扩展 / 成就=完整版) + 2026-09-09 用户发令"基本设计也做一下"

---

## 0. 目的 (Purpose)

本文档基于 [`SRS-AGENT-RELATIONSHIP-001` §4-§10](../requirements/SRS-AGENT-RELATIONSHIP-001.md) 的需求, 定义 **Agent Relationship Graph (ARG)** 视图的基本設計:

- 系统架构 (5-tier: Memgraph / 同步桥 / 协作影响层 / UI 层 / 成就引擎)
- 组件一覧 (15 个新组件 / 模块划分 / 4 新 crate)
- 数据模型 (Memgraph schema + LangGraph state schema 扩展 + zustand store 扩展)
- 接口设计 (REST API 13 端点 + WebSocket 1 端点 + 内部 5 协议)
- 5 view (機能/データ/動作/モジュール/ネットワーク) 完整覆盖
- NFR 6 项 (性能 / 可靠性 / 安全 / 易用 / 成就 / 可观测)
- 守门 14 项 + 子代理失败接手 + 已知缺口 10 项

**dual-use 提醒 (per [AGENTS.md §5 倉庫拓扑](../../AGENTS.md))**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射. ARG 跟 LangGraph view / Agent Runtime view / Agent View 平行, 通过 LangGraph state schema 桥接, 但**不**直接调用其他 view 的 LangGraph node.

---

## 1. 适用范围 (Scope)

### 1.1 包含 (In-Scope)

#### 1.1.1 数据层 (Memgraph)

- **Memgraph 部署**: Docker compose 启动 (port 7687 Bolt + 7444 HTTP), 数据卷持久化
- **Cypher schema**: Agent 节点 + 10 类关系边 (4 核心 directed + 1 核心 undirected + 5 扩展 directed + 1 扩展 undirected, 详见 §3.2)
- **复合索引**: id / archetype / domain / (from+to+type+archived) 4 类
- **约束**: 100% RLS 13 类 (per 守门 #13 Master 类), 物理删除禁止 + SCD Type 2
- **5 张数据表 (per 守门 #13 W/T/M)**:
  - `agents` (Master) — 节点元数据
  - `agent_relationship_edges` (Master) — 边定义
  - `agent_relationship_edges_audit` (Transaction) — append-only 改关系审计
  - `team_template_instances` (Work, TTL 30 天) — 模板实例化中间态
  - `relationship_events` (Transaction) — 运行时事件流 (供成就评估 + observability)

#### 1.1.2 同步桥层 (Sync Bridge)

- **Memgraph → in-process 推**: Memgraph write 触发 EventBus `edge.changed` event
- **in-process → Memgraph 周期 flush**: 30s 周期, 合并 in-process 状态变更 → Memgraph upsert
- **离线降级**: Memgraph 不可达时, in-process 缓存继续工作, 重连后 flush
- **冲突解决**: LWW + version optimistic lock (per 守门 #13 SCD Type 2)
- **API 端点**: 13 REST + 1 WebSocket (per §6.1)

#### 1.1.3 协作影响层 (Effect Layer, 4 维度)

- **4.1 Dispatch 路由器** (`ARGDispatchRouter`): 读 ARG 改写 L0 dispatch_node 默认逻辑
- **4.2 上下文注入器** (`ARGContextInjector`): 读 ARG 把 mentors / shadows 边的 context 注入 prompt
- **4.3 信任度引擎** (`ARGTrustEngine`): 计算 trust_score, 改写 verify 节点 skip 逻辑
- **4.4 产出评估器** (`ARGOutputEvaluator`): 读 ARG 触发 peer_reviews / challenges 双向论证
- **LangGraph 集成**: 4 维度都注册为 LangGraph 节点 (per §4 LangGraph state schema 扩展)

#### 1.1.4 UI 层 (gm-console frontend)

- **`/agent-relationships` 新路由** (per `app/agent-relationships/page.tsx`)
- **3 个新页面**:
  - `RelationshipEditor` (画布拖拽建关系)
  - `RelationshipView` (图谱浏览, 节点详情侧栏)
  - `AchievementWall` (成就展示 + 触发条件说明)
- **Agent View 集成**: `agent-view` 页加 1 个 tab 切到 "Relationship" 视角
- **复用**:
  - `frontend-canvas-design.md` v0.1 无限画布 + bezier connector 公式
  - `BD-AGENT-VIEW-001` zustand store (扩展 5 channel)
  - React + TypeScript + Zustand 既有栈 (per Agent View 既有栈)

#### 1.1.5 成就引擎 (Achievement Engine)

- **3 维度评估器**:
  - `TopologyEvaluator` — 8 拓扑成就 (Cypher 查询模板, 8 条)
  - `BehaviorEvaluator` — 7 协作行为成就 (event pattern 匹配, 7 条)
  - `OutputEvaluator` — 5 产出质量成就 (聚合指标查询, 5 条)
- **触发**: edge.changed event → 异步评估 (不阻塞主流程)
- **解锁**: 写入 `achievement_unlocks` Transaction 表, SSE 推送
- **稀有度**: 4 级 (COMMON / RARE / EPIC / LEGENDARY), 概率分布见 §7.4

#### 1.1.6 5 团队模板 (1-click 部署)

- Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council
- 模板定义存 `team_templates` (Master)
- 实例化写 `team_template_instances` (Work, TTL 30 天)
- 实例化是 1 事务 Cypher (1 template_id → N agent_id → 1 batch write)

#### 1.1.7 守门合规

- 守门 #1+#3+#5+#6+#7+#9+#10+#13+#14 v2 全过
- 守门 #1 v19 自动化档判定: 全部 P3-C/P3-D 阶段子项必先 `scripts/automation/<purpose>.py` 落地
- 守门 #1 v25: cargo test 走 `cargo test -p star-arg --lib -j 4` 单 crate 模式
- 守门 #5: Memgraph 连接串走 env, 不打印
- 守门 #6: 部署脚本 PowerShell only

### 1.2 不包含 (Out-of-Scope)

- LangGraph 任务卡 DAG 改造 (那个归 TMO 9 节点, 本视图只读 task_relationships)
- Agent View 画布功能改造 (ARG 是其 1 个 tab, 改画布本体的写不动)
- Memgraph HA 集群 / replica set (per SRS G-7, 后续阶段)
- 真实 LLM 微调 (关系层不改模型权重, 只改 prompt 拼装 + dispatch 路由)
- 跨组织 agent 联邦 (本视图只覆盖 Star 仓内部 agent)
- 关系自动发现 (本视图是"用户定义 + 协作行为反馈", 不做自动发现)
- ARG → RGS 仓数据同步 (per AGENTS.md §5 仓库拓扑硬约束, Star 仓不引用 RGS 仓)

### 1.3 跟其他 view 区别

| 维度 | LangGraph View | Agent Runtime View | Agent View | **ARG (本 view)** |
|---|---|---|---|---|
| **关注点** | UI 驱动 2-level Agent + 任务卡 DAG | Rust Runtime 基础设施 (派发/ECS/共享池) | 派生视图: 单 agent 拓扑 | **agent 之间 social 层** |
| **目标** | LLM 编排 + 任务卡生命周期 | L0 派发 + L1 ECS + L2 业务池 | 当前工作 agent 可视化 | **关系定义 + 协作影响 + 成就激励** |
| **实现** | LangGraph Python subgraph | Rust + Tokio + ECS | React + zustand | **Memgraph + LangGraph 桥接 + React** |
| **数据源** | LangGraph state schema | PostgreSQL / SQLite | zustand store | **Memgraph (主) + LangGraph state (缓存)** |
| **用户交互** | Chat Bar + Task Card Modal | 0 (server-side) | 画布 pan/zoom | **画布拖拽 + 关系编辑 + 成就展示** |
| **成就** | ✗ | ✗ | ✗ | **✓ 20 成就 3 维度** |

---

## 2. システムアーキテクチャ (System Architecture)

### 2.1 全体構成図 (Overall Architecture, 5-tier)

```
┌──────────────────────────────────────────────────────────────────────────┐
│            UI Tier (frontend/src/app/agent-relationships/)                │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  page.tsx (AppShell 集成, 3 tab: Editor / View / Achievements)      │  │
│  │  ┌──────────────────────────────────────────────────────────────┐  │  │
│  │  │  RelationshipEditor (画布拖拽建关系, 复用 frontend-canvas)    │  │  │
│  │  │  RelationshipView (图谱浏览, 节点详情侧栏)                    │  │  │
│  │  │  AchievementWall (成就墙, 稀有度筛选)                         │  │  │
│  │  │  AgentViewTab (agent-view 加 1 个 tab 切到 ARG 视角)          │  │  │
│  │  └──────────────────────────────────────────────────────────────┘  │  │
│  │  zustand store extension: useARGStore (5 channel, per §5.2)        │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ HTTP REST (13 endpoints)         │ WebSocket (/ws/arg/events)
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│              API Tier (crates/api + star-mcp transport)                   │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  ARGController (Axum router, 13 REST endpoints)                     │  │
│  │  ARGSSEHub (WebSocket /ws/arg/events)                              │  │
│  │  Permission middleware (RLS 13 类 per 守门 #13)                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ Bolt protocol (port 7687)       │ In-process call
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│            Data Tier (crates/arg + Memgraph 2.14+)                        │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Memgraph Client (r2d2-memgraph pool, 8-16 conn)                   │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │  │
│  │  │ AgentNode    │  │ EdgeOps      │  │ TemplateOps  │             │  │
│  │  │ (CRUD)       │  │ (CRUD+audit) │  │ (instantiate)│             │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘             │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │  │
│  │  │ CypherCache  │  │ EventWriter  │  │ MigrationMgr │             │  │
│  │  │ (LRU 1000)   │  │ (audit log)  │  │ (schema v1+) │             │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘             │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ Bolt protocol                    │ In-process call
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│            Bridge Tier (crates/arg-bridge)                                │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  MemgraphEventListener (Bolt subscription, edge.changed)           │  │
│  │  LangGraphStateUpdater (写 in-process state)                       │  │
│  │  PeriodFlushWorker (30s, in-process → Memgraph)                    │  │
│  │  OfflineQueue (Memgraph 不可达时本地缓存)                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
   │ LangGraph in-process             │ WebSocket
   ↓                                  ↓
┌──────────────────────────────────────────────────────────────────────────┐
│        Effect Tier (crates/arg-effect + LangGraph nodes)                  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  ARGDispatchRouter (LangGraph 节点, 读 ARG 改写 dispatch)         │  │
│  │  ARGContextInjector (mentors / shadows context 注入)              │  │
│  │  ARGTrustEngine (trust_score 计算 + verify skip 决策)             │  │
│  │  ARGOutputEvaluator (challenges / peer_reviews 双向论证)          │  │
│  │  ARGAchievementEngine (3 维度评估, 异步触发)                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Tier 详细说明

#### Tier 1: UI Tier (frontend/src/app/agent-relationships/)

- **路由**: `/agent-relationships` (新) + `/agent-view` 加 tab
- **组件**:
  - `RelationshipEditor.tsx` (画布拖拽)
  - `RelationshipView.tsx` (图谱浏览 + 节点详情)
  - `AchievementWall.tsx` (成就墙)
  - `EdgeTypeSelector.tsx` (10 类关系选择器)
  - `TemplateGallery.tsx` (5 模板库)
- **zustand store**: `useARGStore` (5 channel: agents / edges / templates / achievements / events)

#### Tier 2: API Tier (crates/api 扩展)

- **位置**: `crates/api/src/arg/` (新模块)
- **组件**:
  - `controller.rs` (Axum router, 13 REST endpoints)
  - `sse_hub.rs` (WebSocket /ws/arg/events)
  - `permission.rs` (RLS 13 类 middleware)
  - `dto.rs` (request/response types)
- **守门**: RLS 13 类必携 (per 守门 #13)

#### Tier 3: Data Tier (crates/arg 新 crate)

- **位置**: `crates/arg/` (新 crate, per §5.1 Cargo.toml)
- **组件** (6 子模块):
  - `client.rs` (Memgraph Bolt client + 连接池)
  - `agent_node.rs` (Agent CRUD)
  - `edge_ops.rs` (Edge CRUD + audit log)
  - `template_ops.rs` (5 模板 instantiate)
  - `cypher_cache.rs` (LRU 1000, query cache)
  - `event_writer.rs` (audit log append-only writer)
  - `migration.rs` (schema 版本管理)
- **依赖**:
  - `tokio` (async runtime, 既有)
  - `serde` / `serde_json` (既有)
  - `r2d2-memgraph` (新依赖, per 缺口 G-1)
  - `uuid` (既有, Agent ID)
  - `chrono` (既有, 时间戳)
  - `tracing` (既有, observability)

#### Tier 4: Bridge Tier (crates/arg-bridge 新 crate)

- **位置**: `crates/arg-bridge/` (新 crate)
- **组件** (4 子模块):
  - `memgraph_listener.rs` (Bolt subscription)
  - `langgraph_updater.rs` (写 in-process LangGraph state)
  - `period_flush.rs` (30s 周期 flush)
  - `offline_queue.rs` (Memgraph 不可达时本地缓存)
- **依赖**:
  - `arg` (新)
  - `langgraph` (新依赖, Python binding via PyO3, per 缺口 G-3)
  - `tokio` / `serde` / `tracing` (既有)

#### Tier 5: Effect Tier (crates/arg-effect 新 crate)

- **位置**: `crates/arg-effect/` (新 crate)
- **组件** (5 子模块):
  - `dispatch_router.rs` (LangGraph dispatch_node 拦截)
  - `context_injector.rs` (mentors / shadows context 注入)
  - `trust_engine.rs` (trust_score + verify skip)
  - `output_evaluator.rs` (challenges / peer_reviews)
  - `achievement_engine.rs` (3 维度评估)
- **依赖**:
  - `arg` (新)
  - `langgraph` (新依赖)
  - `tokio` / `serde` / `tracing` (既有)

### 2.3 数据流 (Data Flow)

#### 2.3.1 写关系 (UI → Memgraph)

```
[UI: 拖拽] → POST /api/arg/edges
              ↓
[API: ARGController] → 校验 + RLS
              ↓
[Data: EdgeOps.create()] → Memgraph Bolt write (1 事务)
              ↓
[Memgraph] → 触发 Bolt subscription event
              ↓
[Bridge: MemgraphEventListener] → EventBus edge.changed
              ↓
[Effect: 4 维度 effect 异步 reload] → 后续 dispatch 走新关系
              ↓
[Bridge: PeriodFlushWorker 30s] → in-process state 周期 flush (兜底)
              ↓
[API: ARGSSEHub] → 推送 edge.changed 给所有订阅前端
              ↓
[UI: 实时更新图谱]
```

#### 2.3.2 协作影响 (Dispatch 路由)

```
[L0 Top Agent] 收到任务
    ↓
[dispatch_node] → 调 ARGDispatchRouter
    ↓
[ARGDispatchRouter] → 查 ARG: 当前 agent 的 out-degree delegates_to 边
    ↓
   有 2 条 (Lead → Worker A, Lead → Worker B)
    ↓
[ARGDispatchRouter] → 改写 dispatch 逻辑, spawn Worker A + B 並行
    ↓
[SubAgentPool] → spawn 2 sub-agent (per LangGraph 02 §2.3)
    ↓
[Worker A / B] 执行 (不需要 Lead 参与)
    ↓
[reports_to 边] → 完成后推送到 Lead inbox
    ↓
[Top Agent] → aggregate → user
```

#### 2.3.3 成就评估

```
[edge.changed event] 或 [collaboration.success event]
    ↓
[Achievement Engine] → 异步触发评估 (后台 task)
    ↓
[TopologyEvaluator] → 跑 Cypher 查询 8 个拓扑成就
[BehaviorEvaluator] → 匹配 7 个 event pattern
[OutputEvaluator] → 聚合 5 个输出指标
    ↓
[命中] → 写 achievement_unlocks Transaction 表
    ↓
[API: ARGSSEHub] → 推送 achievement.unlocked event
    ↓
[UI: AchievementWall 弹通知 + 徽章]
```

---

## 3. 组件一覧 (Component List)

### 3.1 组件总览 (15 个新组件)

| # | 组件 ID | 名称 | Tier | 语言 | 优先级 |
|---|---|---|---|---|---|
| **C-1** | `MemgraphClient` | Memgraph Bolt 客户端 + 连接池 | Data | Rust | P0 (v0.1) |
| **C-2** | `AgentNode` | Agent CRUD (create / read / update / archive) | Data | Rust | P0 |
| **C-3** | `EdgeOps` | 边 CRUD + audit log 双写 | Data | Rust | P0 |
| **C-4** | `TemplateOps` | 5 模板 instantiate (1 事务) | Data | Rust | P0 |
| **C-5** | `CypherCache` | LRU 1000 query cache | Data | Rust | P1 |
| **C-6** | `EventWriter` | append-only audit log writer | Data | Rust | P0 |
| **C-7** | `MemgraphEventListener` | Bolt subscription edge.changed | Bridge | Rust | P0 |
| **C-8** | `LangGraphStateUpdater` | 写 in-process LangGraph state | Bridge | Rust | P0 |
| **C-9** | `PeriodFlushWorker` | 30s 周期 flush worker | Bridge | Rust | P0 |
| **C-10** | `OfflineQueue` | Memgraph 不可达本地缓存 | Bridge | Rust | P1 |
| **C-11** | `ARGDispatchRouter` | 改写 dispatch_node 路由 | Effect | Rust | P0 |
| **C-12** | `ARGContextInjector` | mentors / shadows context 注入 | Effect | Rust | P0 |
| **C-13** | `ARGTrustEngine` | trust_score + verify skip | Effect | Rust | P0 |
| **C-14** | `ARGOutputEvaluator` | challenges / peer_reviews 双向论证 | Effect | Rust | P0 |
| **C-15** | `ARGAchievementEngine` | 3 维度成就评估 (异步) | Effect | Rust | P0 |
| **C-16** | `RelationshipEditor` | 画布拖拽建关系 UI | UI | TypeScript | P0 |
| **C-17** | `RelationshipView` | 图谱浏览 + 节点详情 | UI | TypeScript | P0 |
| **C-18** | `AchievementWall` | 成就墙 + 稀有度筛选 | UI | TypeScript | P1 |
| **C-19** | `EdgeTypeSelector` | 10 类关系选择器 | UI | TypeScript | P0 |
| **C-20** | `TemplateGallery` | 5 模板库 | UI | TypeScript | P1 |
| **C-21** | `ARGController` | Axum router 13 REST endpoints | API | Rust | P0 |
| **C-22** | `ARGSSEHub` | WebSocket /ws/arg/events | API | Rust | P0 |
| **C-23** | `ARGPermission` | RLS 13 类 middleware | API | Rust | P0 |
| **C-24** | `ARGMigration` | schema 版本管理 | Data | Rust | P2 |

> **修正**: 实际新组件 = 24 个 (C-1..C-24), 上表 15 是数据/effect 层核心, 加上 UI/API/bridge = 24 个总组件.

### 3.2 10 类关系实现 (4 核心 + 6 扩展)

| 关系类型 | 方向 | Cypher label | 协作影响实现 | 优先级 |
|---|---|---|---|---|
| `delegates_to` | directed | `:DELEGATES_TO` | ARGDispatchRouter 改写 dispatch_node | P0 |
| `consults` | directed | `:CONSULTS` | ARGContextInjector 注入 reviewer 视角 | P0 |
| `collaborates_with` | undirected | `:COLLABORATES_WITH` | ARGDispatchRouter 并行 + merge | P0 |
| `reports_to` | directed | `:REPORTS_TO` | 完成 event 推送目标 agent inbox | P0 |
| `mentors` | directed | `:MENTORS` | mentee 启动拉 mentor 历史决策 | P1 |
| `peer_reviews` | undirected | `:PEER_REVIEWS` | 双向 review 阈值 ≥ 0.8 | P1 |
| `stand_in_for` | directed | `:STAND_IN_FOR` | A failed → fallback B | P1 |
| `shadows` | directed | `:SHADOWS` | shadow 订阅被观察 agent event | P1 |
| `challenges` | directed | `:CHALLENGES` | 强制双向论证 | P1 |
| `trusts` | directed | `:TRUSTS` | weight ≥ 0.8 跳过 verify | P0 |

### 3.3 依赖关系 (per 守门 #1)

```toml
# Cargo.toml workspace 新增
[workspace.dependencies]
r2d2-memgraph = "0.1"  # 待调研, per 缺口 G-1
arg = { path = "crates/arg" }
arg-bridge = { path = "crates/arg-bridge" }
arg-effect = { path = "crates/arg-effect" }

[workspace.lints]
unsafe_code = "forbid"  # per 守门 #7
unreachable_pub = "deny"  # per 守门 #7
```

---

## 4. 数据模型 (Data Model)

### 4.1 Memgraph Schema (Cypher)

#### 4.1.1 Agent 节点

```cypher
// 创建 Agent 节点
CREATE (a:Agent {
  id: STRING,                    // UUID, 唯一 ID
  name: STRING,                  // 展示名
  archetype: STRING,             // SA-01..SA-09 / LEAD-{DOMAIN} / CUSTOM
  domain: STRING,                // player/economy/match/social/admin/null
  status: STRING,                // active/standby/archived
  trust_score: FLOAT,            // 0.0-1.0, 动态
  metadata: JSON,                // 扩展字段
  tenant_id: STRING,             // RLS 13 类 per 守门 #13
  created_at: DATETIME,
  updated_at: DATETIME,
  version: INT,                  // SCD Type 2 per 守门 #13
  created_by: STRING             // author = Ulysses per 9/8 15:19 第 6 次强化
});

// 索引
CREATE INDEX ON :Agent(id);
CREATE INDEX ON :Agent(archetype);
CREATE INDEX ON :Agent(domain);
CREATE INDEX ON :Agent(tenant_id);

// 约束
CREATE CONSTRAINT ON (a:Agent) ASSERT a.id IS UNIQUE;
```

#### 4.1.2 10 类关系边

```cypher
// 4 核心 directed
CREATE (a:Agent {id: 'A'})-[r:DELEGATES_TO {
  id: STRING,
  weight: FLOAT,
  archived: BOOLEAN,
  tenant_id: STRING,
  created_at: DATETIME,
  created_by: STRING,
  version: INT,
  metadata: JSON
}]->(b:Agent {id: 'B'});

// 1 核心 undirected
CREATE (a:Agent {id: 'A'})-[r:COLLABORATES_WITH {
  ... 同样属性
}]-(b:Agent {id: 'B'});

// 5 扩展 directed
:rCONSULTS / :REPORTS_TO / :MENTORS / :STAND_IN_FOR / :SHADOWS / :CHALLENGES / :TRUSTS

// 1 扩展 undirected
:rPEER_REVIEWS

// 索引
CREATE INDEX ON :DELEGATES_TO(from_agent);
CREATE INDEX ON :DELEGATES_TO(to_agent);
CREATE INDEX ON :REPORTS_TO(to_agent);  // 高频查询
CREATE INDEX ON :STAND_IN_FOR(from_agent);  // fallback 关键
```

#### 4.1.3 复合索引 (per 守门 #13 W/T/M)

- **Master** (`agents`, `agent_relationship_edges`): 100% RLS 13 类, SCD Type 2, 物理删除禁止
- **Transaction** (`agent_relationship_edges_audit`, `relationship_events`): 100% audit, append-only
- **Work** (`team_template_instances`): TTL 30 天, retention_period 必填

### 4.2 LangGraph State Schema 扩展 (per LangGraph 02 §3.2)

#### 4.2.1 TopAgentState 新增 channel (5 个)

```python
# Per LangGraph 02-basic-design.md §3.2 扩展
from typing import TypedDict, Annotated
from langgraph.graph import add
import operator

class TopAgentState(TypedDict):
    # 既有字段 (per LangGraph 02 §3.2)
    messages: Annotated[list, add]  # 既有
    active_subagents: Annotated[list, add]  # 既有
    # ... 既有字段省略

    # ★ NEW ★ ARG 扩展 5 channel
    arg_agents: Annotated[list[Agent], merge_arg_agents]  # 当前会话相关的 agent 列表
    arg_edges: Annotated[list[Edge], merge_arg_edges]  # 当前会话相关的边列表
    arg_trust_scores: Annotated[dict[str, float], merge_trust_scores]  # agent_id → trust_score
    arg_dispatch_overrides: Annotated[dict[str, list[str]], merge_dispatch]  # agent_id → out-degree delegates_to 目标
    arg_achievements_unlocked: Annotated[list[Achievement], add]  # 累积解锁成就
```

#### 4.2.2 5 个新 Reducer

| Reducer | 签名 | 语义 |
|---|---|---|
| `merge_arg_agents` | `(list, list) → list` | 按 id 合并, 后写胜出 (LWW) |
| `merge_arg_edges` | `(list, list) → list` | 按 (from, to, type) 合并, version 大者胜 |
| `merge_trust_scores` | `(dict, dict) → dict` | key 后写胜出, value 取 max |
| `merge_dispatch` | `(dict, dict) → dict` | key 后写胜出, value 用 list extend |
| `add` (既有) | `(list, list) → list` | append-only 累积 (per LangGraph 02) |

### 4.3 zustand Store 扩展 (per `BD-AGENT-VIEW-001` §3.2)

#### 4.3.1 useARGStore 5 channel

```typescript
// frontend/src/lib/arg/store.ts (新)
interface ARGStore {
  // Channel 1: agents
  agents: Map<string, Agent>;
  agentsLoading: boolean;
  agentsError: string | null;

  // Channel 2: edges
  edges: Map<string, Edge>;
  edgesLoading: boolean;
  edgesError: string | null;

  // Channel 3: templates
  templates: Template[];
  templatesLoading: boolean;

  // Channel 4: achievements
  achievements: Achievement[];
  unlockedAchievements: Set<string>;  // achievement code
  achievementsLoading: boolean;

  // Channel 5: events (WebSocket)
  argEvents: ARGEvent[];  // edge.changed / achievement.unlocked / dispatch.route.changed
  wsConnected: boolean;

  // Actions
  loadAgents(): Promise<void>;
  loadEdges(filter?: EdgeFilter): Promise<void>;
  createEdge(input: CreateEdgeInput): Promise<Edge>;
  updateEdge(id: string, patch: UpdateEdgePatch): Promise<Edge>;
  archiveEdge(id: string): Promise<void>;
  instantiateTemplate(templateId: string, agentIds: string[]): Promise<TemplateInstance>;
  loadAchievements(): Promise<void>;
  unlockAchievement(code: string): void;
  subscribeEvents(): void;  // WebSocket
}
```

### 4.4 5 张数据表 SQL Schema (per 守门 #13 W/T/M)

#### 4.4.1 `agents` (Master)

```sql
CREATE TABLE agents (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  archetype VARCHAR(50) NOT NULL,  -- SA-01..SA-09 / LEAD-* / CUSTOM
  domain VARCHAR(50),  -- player/economy/match/social/admin/null
  status VARCHAR(20) NOT NULL DEFAULT 'active',  -- active/standby/archived
  trust_score REAL NOT NULL DEFAULT 0.5,  -- 0.0-1.0
  metadata JSONB,
  tenant_id UUID NOT NULL,  -- RLS 13 类 per 守门 #13
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1,  -- SCD Type 2
  created_by UUID NOT NULL  -- author = Ulysses per 9/8 15:19
);

-- 索引
CREATE INDEX idx_agents_archetype ON agents(archetype);
CREATE INDEX idx_agents_domain ON agents(domain);
CREATE INDEX idx_agents_tenant ON agents(tenant_id);
CREATE INDEX idx_agents_status ON agents(status);

-- RLS (per 守门 #13 Master 类 100% RLS)
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
CREATE POLICY agents_tenant_isolation ON agents
  USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

#### 4.4.2 `agent_relationship_edges` (Master)

```sql
CREATE TABLE agent_relationship_edges (
  id UUID PRIMARY KEY,
  from_agent UUID NOT NULL REFERENCES agents(id),
  to_agent UUID NOT NULL REFERENCES agents(id),
  type VARCHAR(30) NOT NULL,  -- 10 类关系 enum
  weight REAL NOT NULL DEFAULT 0.5,  -- 0.0-1.0
  direction VARCHAR(20) NOT NULL,  -- directed/undirected
  archived BOOLEAN NOT NULL DEFAULT false,
  metadata JSONB,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1,
  created_by UUID NOT NULL,
  UNIQUE(from_agent, to_agent, type, archived)  -- 复合唯一
);

CREATE INDEX idx_edges_from ON agent_relationship_edges(from_agent);
CREATE INDEX idx_edges_to ON agent_relationship_edges(to_agent);
CREATE INDEX idx_edges_type ON agent_relationship_edges(type);
CREATE INDEX idx_edges_tenant ON agent_relationship_edges(tenant_id);
CREATE INDEX idx_edges_archived ON agent_relationship_edges(archived);
```

#### 4.4.3 `agent_relationship_edges_audit` (Transaction)

```sql
CREATE TABLE agent_relationship_edges_audit (
  id UUID PRIMARY KEY,
  edge_id UUID NOT NULL,
  action VARCHAR(20) NOT NULL,  -- create/update/archive
  old_value JSONB,  -- 改前快照
  new_value JSONB,  -- 改后快照
  actor UUID NOT NULL,
  tenant_id UUID NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 物理删除禁止 (per 守门 #13 Transaction 类)
CREATE INDEX idx_audit_edge ON agent_relationship_edges_audit(edge_id);
CREATE INDEX idx_audit_timestamp ON agent_relationship_edges_audit(timestamp);
```

#### 4.4.4 `team_template_instances` (Work, TTL 30 天)

```sql
CREATE TABLE team_template_instances (
  id UUID PRIMARY KEY,
  template_id VARCHAR(50) NOT NULL,  -- hub-and-spoke/mesh/chain/hierarchical/review-council
  instance_name VARCHAR(255) NOT NULL,
  agent_ids UUID[] NOT NULL,
  edges_json JSONB NOT NULL,  -- 创建的边快照
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '30 days',  -- TTL
  retention_period INTERVAL NOT NULL DEFAULT '30 days'  -- 必填 per 守门 #13 a
);
```

#### 4.4.5 `achievement_unlocks` (Transaction)

```sql
CREATE TABLE achievement_unlocks (
  id UUID PRIMARY KEY,
  achievement_code VARCHAR(100) NOT NULL,  -- e.g. "TOP-001-MESH-5DOMAIN"
  user_id UUID NOT NULL,
  agent_ids UUID[],  -- 触发该成就的 agent
  trigger_metadata JSONB,  -- 触发时的快照
  tenant_id UUID NOT NULL,
  unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_unlocks_user ON achievement_unlocks(user_id);
CREATE INDEX idx_unlocks_code ON achievement_unlocks(achievement_code);
```

---

## 5. 接口设计 (Interface Design)

### 5.1 REST API (13 端点)

| Method | Path | 说明 | RLS | 守门 |
|---|---|---|---|---|
| `POST` | `/api/arg/agents` | 创建 agent | ✓ | W/T/M 严格 |
| `GET` | `/api/arg/agents` | 列表 agent (分页 + 过滤) | ✓ | 限频 100/min |
| `GET` | `/api/arg/agents/{id}` | agent 详情 | ✓ | — |
| `PATCH` | `/api/arg/agents/{id}` | 更新 agent (version+1) | ✓ | Master |
| `POST` | `/api/arg/edges` | 创建关系 (11 字段校验) | ✓ | append-only audit |
| `GET` | `/api/arg/edges` | 列表关系 (filter type/agent/archived) | ✓ | 分页 |
| `GET` | `/api/arg/edges/{id}` | 边详情 | ✓ | — |
| `PATCH` | `/api/arg/edges/{id}` | 更新关系 (weight/metadata) | ✓ | Master |
| `DELETE` | `/api/arg/edges/{id}` | 归档关系 (archived=true) | ✓ | append-only audit |
| `GET` | `/api/arg/graph` | 整图查询 (Cypher, limit 1000) | ✓ | 限频 10/min |
| `POST` | `/api/arg/templates/instantiate` | 5 模板实例化 (1 事务) | ✓ | 1 batch write |
| `GET` | `/api/arg/achievements` | 20 成就定义列表 | ✓ | 分页 |
| `GET` | `/api/arg/achievements/me` | 我的解锁成就 | ✓ | 用户维度 |
| `POST` | `/api/arg/achievements/evaluate` | 触发评估 (admin only) | ✓ | 幂等 |

### 5.2 WebSocket (1 端点)

| Path | 协议 | 事件类型 | 守门 |
|---|---|---|---|
| `/ws/arg/events` | SSE-over-WebSocket | `edge.changed` / `edge.created` / `edge.archived` / `achievement.unlocked` / `dispatch.route.changed` / `agent.trust_score.changed` | auth required + tenant_id 必填 |

### 5.3 内部协议 (5 类)

| 协议 | 方向 | 载荷 | 守门 |
|---|---|---|---|
| `arg_edge_changed` | Memgraph → in-process | `{ edge_id, action, from, to, type, version }` | per §2.3.1 |
| `arg_dispatch_route` | L0 dispatch_node → SubAgentPool | `{ source_agent, target_agents[], edge_type }` | per §2.3.2 |
| `arg_context_inject` | LLM prompt builder | `{ mentors[], shadows[], contexts[] }` | per §2.3.2 |
| `arg_trust_score_update` | TrustEngine → verify_node | `{ agent_id, trust_score, skip_verify: bool }` | per §2.3.3 |
| `arg_achievement_unlocked` | AchievementEngine → SSE | `{ achievement_code, user_id, unlocked_at }` | per §2.3.3 |

### 5.4 内部 5 类状态机

#### 5.4.1 Edge 状态机

```
[created] --archive--> [archived] (终态)
   |
   └──update weight/metadata--> [updated] (回到 created, version+1)
```

#### 5.4.2 Agent 状态机

```
[active] --archive--> [archived] (终态)
   |
   └──standby--> [standby] --activate--> [active]
```

#### 5.4.3 Trust Score 状态机 (5 档)

```
[0.0-0.2] (UNTRUSTED) --3 成功--> [0.2-0.4] (LOW)
[0.2-0.4] (LOW) --5 成功--> [0.4-0.7] (MEDIUM)
[0.4-0.7] (MEDIUM) --10 成功--> [0.7-0.9] (HIGH)
[0.7-0.9] (HIGH) --20 成功--> [0.9-1.0] (VERY_HIGH)
任意档位 --失败-0.05--> 降一档
```

#### 5.4.4 Template Instance 状态机

```
[pending] --deploy--> [active] --TTL 30 天--> [expired]
```

#### 5.4.5 Achievement 状态机

```
[locked] --trigger--> [unlocked] (终态, append-only)
```

---

## 6. 5 View 完整覆盖

### 6.1 機能 view (Functional View)

| 功能 | 描述 | 组件 |
|---|---|---|
| **F-1: 关系可视化** | 图谱浏览, 节点+边+标签+权重 | RelationshipView |
| **F-2: 关系编辑** | 拖拽建边 + 选类型 + 配 weight | RelationshipEditor + EdgeTypeSelector |
| **F-3: 关系管理** | 列表 / 过滤 / 批量操作 | ARGController + useARGStore |
| **F-4: 模板库** | 5 模板浏览 + 1-click 部署 | TemplateGallery + TemplateOps |
| **F-5: 协作影响 (4 维度)** | 改 dispatch / 注入 context / 信任度 / 评估 | ARGDispatchRouter + ARGContextInjector + ARGTrustEngine + ARGOutputEvaluator |
| **F-6: 成就展示** | 20 成就墙 + 触发条件 + 稀有度 | AchievementWall + ARGAchievementEngine |
| **F-7: 实时事件** | WebSocket SSE 推送 | ARGSSEHub |
| **F-8: Audit Trail** | 改关系 100% 审计 | EventWriter + audit 表 |
| **F-9: Agent View 集成** | 1 tab 切到 ARG 视角 | AgentViewTab |
| **F-10: 离线降级** | Memgraph 不可达继续工作 | OfflineQueue |

### 6.2 データ view (Data View)

| 数据 | 存储 | 类型 (W/T/M) | 用途 |
|---|---|---|---|
| Agent 节点 | Memgraph + SQL mirror | Master | 节点元数据 |
| Edge 边 | Memgraph + SQL mirror | Master | 关系定义 |
| Edge Audit | SQL (append-only) | Transaction | 改关系审计 |
| Template Instance | SQL (TTL 30 天) | Work | 模板中间态 |
| Achievement Unlock | SQL (append-only) | Transaction | 成就解锁记录 |
| Relationship Events | SQL (append-only) | Transaction | 运行时事件流 |
| Trust Score | Memgraph (动态) | Master (in-place update) | 信任度实时计算 |
| In-process State | LangGraph state | (cache) | 协作影响实时读 |

### 6.3 動作 view (Behavior View)

#### 6.3.1 正常流: 拖拽建边 → 协作影响

1. UI 拖拽 + 选类型
2. POST /api/arg/edges
3. Memgraph write + audit 双写
4. MemgraphEventListener 触发
5. LangGraphStateUpdater 写 in-process state
6. 4 effect 维度 reload
7. SSE 推送到所有订阅前端
8. UI 实时显示

#### 6.3.2 异常流: Memgraph 不可达

1. Memgraph Bolt 连接失败
2. OfflineQueue 启动, 缓存变更到本地
3. SSE 推 `memgraph.disconnected` 事件
4. UI 显示"离线"badge
5. PeriodFlushWorker 重连成功后批量 flush
6. SSE 推 `memgraph.reconnected` 事件
7. UI 恢复正常

#### 6.3.3 异常流: Trust Score 降到 0

1. 协作失败 -0.05
2. trust_score < 0.2
3. 触发 Trust Engine 报警
4. SSE 推 `agent.trust_score.low`
5. UI 节点变红
6. 下次 dispatch 跳过该 agent (per §4.3.3)

### 6.4 モジュール view (Module View)

```
crates/arg/ (新)
├── Cargo.toml
├── src/
│   ├── lib.rs                 # 入口, 暴露 6 子模块
│   ├── client.rs              # C-1 MemgraphClient
│   ├── agent_node.rs          # C-2 AgentNode CRUD
│   ├── edge_ops.rs            # C-3 EdgeOps CRUD + audit
│   ├── template_ops.rs        # C-4 TemplateOps 5 模板
│   ├── cypher_cache.rs        # C-5 LRU 1000
│   ├── event_writer.rs        # C-6 audit writer
│   ├── migration.rs           # C-24 schema 版本管理
│   ├── models/
│   │   ├── agent.rs           # Agent struct
│   │   ├── edge.rs            # Edge struct + 10 enum
│   │   ├── template.rs        # 5 模板定义
│   │   └── achievement.rs     # 20 成就定义
│   ├── error.rs               # 错误类型
│   └── tests/
│       ├── agent_node_test.rs
│       ├── edge_ops_test.rs
│       └── template_ops_test.rs
└── README.md

crates/arg-bridge/ (新)
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── memgraph_listener.rs   # C-7
│   ├── langgraph_updater.rs   # C-8
│   ├── period_flush.rs        # C-9
│   └── offline_queue.rs       # C-10

crates/arg-effect/ (新)
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── dispatch_router.rs     # C-11
│   ├── context_injector.rs    # C-12
│   ├── trust_engine.rs        # C-13
│   ├── output_evaluator.rs    # C-14
│   └── achievement_engine.rs  # C-15
│   └── achievements/
│       ├── topology_evaluator.rs  # 8 拓扑成就 Cypher
│       ├── behavior_evaluator.rs  # 7 协作行为 pattern
│       └── output_evaluator.rs    # 5 产出质量聚合

crates/api/src/arg/ (新模块)
├── mod.rs
├── controller.rs              # C-21 13 REST endpoints
├── sse_hub.rs                 # C-22 WebSocket
├── permission.rs              # C-23 RLS 13 类 middleware
└── dto.rs                     # request/response types

frontend/src/app/agent-relationships/ (新)
├── page.tsx                   # 3 tab container
├── editor/
│   ├── RelationshipEditor.tsx # C-16
│   └── EdgeTypeSelector.tsx   # C-19
├── view/
│   ├── RelationshipView.tsx   # C-17
│   └── NodeDetail.tsx         # 节点详情侧栏
├── achievements/
│   └── AchievementWall.tsx    # C-18
├── templates/
│   └── TemplateGallery.tsx    # C-20
└── AgentViewTab.tsx           # 1 tab 集成到 /agent-view

frontend/src/lib/arg/ (新)
├── store.ts                   # useARGStore 5 channel
├── api.ts                     # 13 REST 客户端封装
├── ws.ts                      # WebSocket 客户端
└── types.ts                   # TypeScript types
```

### 6.5 ネットワーク view (Network View)

```
[Browser] ──HTTP/1.1 + WebSocket──> [Axum API: port 8080]
                                          │
                                          ├──Bolt 7687──> [Memgraph container]
                                          │
                                          └──HTTP 7444──> [Memgraph HTTP API]
                                                       
[Browser] ──HTTP/1.1──> [Frontend Next.js: port 3001]
                               │
                               └──> [Axum API: 8080]

[LangGraph Python subgraph] ──in-process call──> [crates/arg-effect]
                                                          │
                                                          ├──in-process──> [crates/arg]
                                                          │                   │
                                                          │                   └──Bolt 7687──> [Memgraph]
                                                          │
                                                          └──Bolt subscription──> [Memgraph]
```

**关键守门**:
- 不暴露 Memgraph Bolt port 到公网 (per 守门 #5)
- API RLS 13 类 middleware (per 守门 #13)
- WebSocket auth + tenant_id 必填
- 离线降级 (per §6.3.2)

---

## 7. NFR (Non-Functional Requirements)

### 7.1 NFR-ARG-PERF-01: 性能

| 指标 | 目标 | 实测方法 |
|---|---|---|
| 边创建 latency | P95 < 200ms | Memgraph Bolt 单事务 |
| 图查询 (Cypher, ≤1000 节点) | P95 < 500ms | cypher_cache LRU 1000 |
| 实时事件推送 (in-process) | < 100ms | EventBus + tokio |
| 4 维度 effect reload | < 50ms | in-process state diff |
| 成就评估 (1 万边) | P95 < 1s | 异步 background task |
| Period flush | 30s 周期, 0 阻塞 | 独立 tokio task |
| WebSocket 推送 | < 100ms (10 并发客户端) | SSE fan-out |

### 7.2 NFR-ARG-RELIABILITY-01: 可靠性

- Memgraph 不可达时, in-process 缓存继续工作 (per §6.3.2)
- 重连后自动 flush, 不丢关系变更
- 边创建 100% 持久化 (Memgraph + SQL audit 双写)
- 5 分钟重连重试, max 3 次后告警
- 离线数据最长保留 7 天, 7 天未重连则丢弃 + audit log

### 7.3 NFR-ARG-SECURITY-01: 安全 (per 守门 #13 + #5)

- **RLS 13 类必携** (per 守门 #13): 所有表 100% RLS, tenant_id 隔离
- **关系修改 audit log 必记** (per 守门 #13 Transaction): append-only, 100% 审计
- **Memgraph 连接字符串走 env** (per 守门 #5): `MEMGRAPH_BOLT_URL` / `MEMGRAPH_USER` / `MEMGRAPH_PASSWORD` 必走 env
- **不打印 env** (per 守门 #5): 任何日志/错误不得包含连接串明文
- **SCD Type 2** (per 守门 #13 Master): 物理删除禁止
- **代签规则** (per 守门 #10 + 9/8 15:19 第 6 次强化): author = Ulysses, 真人到位后追溯签字

### 7.4 NFR-ARG-ACHIEVEMENT-01: 成就系统 (per SRS §6 NFR-ARG-ACHIEVEMENT-01)

**20 成就 3 维度分布**:

| 维度 | 数量 | 稀有度分布 |
|---|---|---|
| 8 拓扑 (TOPOLOGY) | 8 | 4 COMMON + 3 RARE + 1 EPIC |
| 7 协作行为 (BEHAVIOR) | 7 | 3 COMMON + 2 RARE + 1 EPIC + 1 LEGENDARY |
| 5 产出质量 (OUTPUT) | 5 | 1 COMMON + 2 RARE + 1 EPIC + 1 LEGENDARY |
| **总计** | **20** | **8 COMMON + 7 RARE + 3 EPIC + 2 LEGENDARY** |

**概率分布** (用户视角):
- COMMON (40%): 易达成, 引导用户
- RARE (35%): 需一定配置 + 行为
- EPIC (20%): 需深度探索
- LEGENDARY (5%): 极难达成, 长期激励

### 7.5 NFR-ARG-USABILITY-01: 易用

- 1-click 模板实例化: ≤ 30s 部署
- 拖拽建边: ≤ 3 次点击完成
- 成就通知: ≤ 1s 弹出
- 节点 hover: 立即显示 tooltip (name + trust_score + edge 数)
- 边 hover: 立即显示 (type + weight + 影响描述)

### 7.6 NFR-ARG-OBSERVABILITY-01: 可观测

- **Metrics (Prometheus 导出)**:
  - `arg_edge_create_total` / `arg_edge_update_total` / `arg_edge_archive_total`
  - `arg_dispatch_route_hit_rate` (关系自动派发占总 dispatch 比例)
  - `arg_trust_score_skip_verify_total` (跳过的 verify 节点数)
  - `arg_collaborate_wall_clock_savings_seconds_total` (协作并行节省时间)
  - `arg_achievement_unlock_total{category, rarity}`
- **Logs (tracing)**:
  - 关系增删改 INFO 级
  - Memgraph 不可达 WARN 级
  - 成就解锁 INFO 级
  - trust_score 变化 DEBUG 级
- **Traces (OpenTelemetry)**:
  - 关系创建 → 同步桥 → effect reload 全链路

---

## 8. 守门合规 (per AGENTS.md §4)

| 守门 | 约束 | ARG 落地 | 实证 |
|---|---|---|---|
| **#1** (R-05 不 push 已反转 + 守门 #1 v15 docs 同步饱和) | git push 守门 + 新事件触发才 docs 同步 | ARG 文档同步 commit 必先跑守门 + 本次新事件 (用户 9/8 22:35 发令 + 9/9 "基本设计也做一下") 触发, 不算饱和 | 本次 BD v0.1 是新事件触发, 守门 #1 v15 允许 |
| **#1 v19** | 自动化档判定 ≥ 2 维 [P] 强制 Python 化 | ARG 4 子项必先 `scripts/automation/<purpose>.py` 落地 (memgraph_setup.py / arg_seed.py / arg_achievement_eval.py / arg_dispatch_test.py) | P3-C 阶段落地 |
| **#1 v25** | cargo test 走 `cargo test -p star-arg --lib -j 4` 单 crate 模式 | `crates/arg` 单 crate 100% pass | P3-C 阶段实证 |
| **#3** (5 域独立 Lead) | 跨域边强制 consults | ARG 关系定义时, 跨域 lead_x → lead_y 必为 consults 而非 delegates_to | per §1.1 守门合规 |
| **#5** (env 安全) | 不打印 env | Memgraph 连接串走 env, 不打印 | per §7.3 |
| **#6** (PowerShell only) | 守门 | ARG 部署脚本 PowerShell | per §1.1.7 |
| **#7** (0 unsafe) | 守门 | `unsafe_code = "forbid"` per workspace lints | per §3.3 |
| **#9** (子代理 RPC) | 实证不可靠 | ARG 同步走 in-process 推 + 周期 flush, 不用 RPC | per §2.2 Tier 4 |
| **#10** (代签规则) | Mavis 默认代 Ulysses | ARG 关系修改 author = Ulysses per 9/8 15:19 第 6 次强化 | per §4.4.1 `created_by` |
| **#12** (Python 化任务卡) | [P] 子项 docs 同步 | ARG 实装阶段 P0 子项必更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md` | P3-C 阶段落地 |
| **#13** (W/T/M 三类横展开) | 强制 100% | 5 表全部分类: agents=Master / edges=Master / audit=Transaction / template_instances=Work (TTL 30d) / events=Transaction / unlocks=Transaction | per §4.4 + §6.2 |
| **#14 v2** (5 域 Lead 拍板 D) | Mavis 临时代签, 真人到位后追溯 | ARG 5 域 Lead 关系定义由 Mavis 落, 真人到位后追溯签字 | per §11 签字栏 |
| **#19 v19** | 守门 #12 死循环饱和边界 | 本次新事件触发, 守门 #1 v15 允许 | per #1 v15 |

### 8.1 子代理失败接手清单 (per AGENTS.md §3 §4 派生)

| 失败场景 | 接手路径 |
|---|---|
| 关系创建失败 (Memgraph 写不进去) | OfflineQueue 缓存 + PeriodFlushWorker 重试 |
| 同步桥断 (Memgraph subscription 掉) | 自动重连 (5 分钟 max 3 次), 失败告警 |
| 4 effect 维度 reload 失败 | in-process state 兜底, 继续用旧关系 (degraded mode) |
| 成就评估超时 (> 5s) | 异步 task kill, 记录 WARN, 下次 event 再评估 |
| 模板实例化部分失败 (1 边失败) | 整事务回滚, 提示用户重试 |

---

## 9. 已知缺口 / 未来扩展 (Known Gaps)

| # | 缺口 | 影响 | 后续阶段 |
|---|---|---|---|
| **G-1** | **Memgraph 客户端 crate 缺** — `r2d2-memgraph` 待调研, 候选: 自实现 (Bolt protocol) / 用 `memgraph-client` (社区) | crates/arg 实现前提 | P3-C 实装阶段第一周 |
| **G-2** | **Memgraph k3s 部署 yaml 缺** — Docker compose 模板可写, k3s 部署待 P3-C 第二周 | 生产化待补 | P3-C 实装阶段 |
| **G-3** | **L0 ↔ L1 通信协议 + ARG 集成** — LangGraph StateGraph 怎么加载 ARG 边 (4 effect 维度) 未写详细协议 | 关系→协作影响无法落地 | P3-D 详细设计 (DD-AGENT-RELATIONSHIP-001) 第一周 |
| **G-4** | **trusts 跳过 verify 的安全审计** — 跳过 verify 后, 失败风险怎么控制待评估, 建议加 trust_score ≥ 0.9 + agent 类型白名单双约束 | 节省 token 但增加风险 | DDD Review + P3-C |
| **G-5** | **challenges 双向论证 prompt 模板** 待写 10 套 (不同 domain / 不同决策类型) | 实装阶段 prompt 模板 | P3-D 详细设计 |
| **G-6** | **8 个拓扑成就 Cypher 模板** 待写 (Mesh / Chain / Hub-and-Spoke / Hierarchical / Review-Council / 跨 5 域 / 自环 / 孤岛) | 实装阶段 evaluator | P3-C 实装阶段 |
| **G-7** | **跨 session 持久化的 Memgraph 高可用** — 单点故障, 待加 replica set + 主从切换 | 生产化待补 | 后续阶段 |
| **G-8** | **5 域 Lead 真人到位 timeline** 待 DDD Review 拍板 (per 9/3 19:43 守门 #14 v2) | 关系代签机制长期维持 | 真人到位时 |
| **G-9** | **跟 TMO 9 节点 (任务卡 DAG) 的边界** — agent relationship graph 跟 task DAG 重叠场景待梳理 (e.g. M-N3 reorder 跟 ARG delegates_to 区别) | 两个图都涉及"边"概念, 需明确边界 | DDD Review |
| **G-10** | **ARG Schema V2 迁移路径** — V1 → V2 加新关系类型时怎么处理存量数据, 建议用 `arg_migration` v1→v2 脚本 | 未来扩展 | 后续阶段 |
| **G-11** | **成就可分享的导出格式** (PNG / 描述 JSON / 周报模板) 未设计 | US-10 实现 | P3-D 详细设计 |
| **G-12** | **ARG 跟 RGS 仓的独立边界** (per AGENTS.md §5 仓库拓扑) — ARG 是 Star 仓独立视图, 不引用 RGS 5 域镜像作为源头 | 命名解读合规 | 持续 |

---

## 10. 实施计划 (Implementation Plan, per WBS 估时)

### 10.1 P3-C 阶段 (实装, ~4 周)

| 周 | 任务 | 守门 |
|---|---|---|
| W1 | crates/arg 6 子模块 (C-1..C-6) + 5 表 SQL schema + Memgraph 客户端调研落地 | cargo test -p star-arg --lib -j 4 100% pass |
| W1 | scripts/automation/memgraph_setup.py 落地 (Docker compose 启动) | 守门 #1 v19 |
| W2 | crates/arg-bridge 4 子模块 (C-7..C-10) + 离线降级 + 周期 flush | cargo test -p star-arg-bridge --lib -j 4 100% |
| W2 | scripts/automation/arg_seed.py 落地 (种子数据: 5 域 Lead + 9 SA + 10 demo agent) | 守门 #1 v19 |
| W3 | crates/arg-effect 5 子模块 (C-11..C-15) + 4 维度 effect | cargo test -p star-arg-effect --lib -j 4 100% |
| W3 | scripts/automation/arg_achievement_eval.py 落地 (8 拓扑成就 Cypher 模板) | 守门 #1 v19 |
| W4 | crates/api/src/arg/ (C-21..C-23) 13 REST + 1 WebSocket | integration test pass |
| W4 | frontend/src/app/agent-relationships/ (C-16..C-20) 5 UI 组件 + zustand store 5 channel | frontend typecheck + test pass |

### 10.2 P3-D 阶段 (詳細設計 + 集成测试, ~2 周)

| 周 | 任务 | 守门 |
|---|---|---|
| W1 | DD-AGENT-RELATIONSHIP-001 v0.1 落档 (rust struct + LangGraph state schema 实现细节 + challenges prompt 模板 10 套) | 守门 #12 + #1 v19 |
| W2 | E2E 集成测试 (拖拽建边 → 协作影响 → 成就解锁 全链路) | 守门 #1 v3 + v25 |

### 10.3 P3-E 阶段 (成就系统 完整版落地, ~1 周)

| 周 | 任务 | 守门 |
|---|---|---|
| W1 | 7 协作行为 + 5 产出质量成就 evaluator 落地 | 守门 #1 v3 |

### 10.4 P3-F 阶段 (DDD Review + 5 域 Lead 真人到位, 待定)

- DDD Review 拍板 G-3/G-4/G-8/G-9
- 5 域 Lead 真人到位后, 追溯签字覆盖修订历史 (per 守门 #14 v2)

---

## §11 签字栏 (5 角色 per AGENTS.md §3 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 |
| **SRE Lead** | ⏳ 待真人到位 (per 守门 #14 v2) | — |
| **平台** | ⏳ 待真人到位 (per 守门 #14 v2) | — |
| **评审主持** | ⏳ 待真人到位 (per 守门 #14 v2) | — |
| **PM** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 9/8 15:19 第 6 次强化) | 2026-09-09 |

> **派生规 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B)**: 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯签字覆盖修订历史

---

## §12 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版落档 — 5-tier 架构 (UI/API/Data/Bridge/Effect) + 24 新组件 (C-1..C-24) + 4 新 crate (arg / arg-bridge / arg-effect / api/arg 扩展) + 5 张 SQL 表 W/T/M 严格 + LangGraph TopAgentState 扩展 5 channel + 5 Reducer + zustand useARGStore 5 channel + 13 REST + 1 WebSocket + 5 内部协议 + 5 状态机 (Edge/Agent/Trust Score 5 档/Template Instance/Achievement) + 6 NFR (性能 P95 200ms/可靠性/安全/成就 20 分布 8-7-3-2/易用/可观测) + 守门 #1+#3+#5+#6+#7+#9+#10+#12+#13+#14 v2+#19 v19 全过 + 12 已知缺口 + P3-C 实装 4 周 + P3-D 2 周 + P3-E 1 周计划; 守门 #1 v15 docs 同步触达饱和确认: 本次有新事件触发 (Ulysses 9/9 发令"基本设计也做一下"), 不算饱和违规 | 2026-09-08 22:35 JST `ask_7d7ffcac2353adad7d3f6f69` 用户拍板 (4 选项) + 2026-09-09 用户发令"基本设计也做一下" (per 守门 #1 v15 新事件触发, 守门 #9 v19 Mavis 自驱) |
