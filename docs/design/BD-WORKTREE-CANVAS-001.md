# BD-WORKTREE-CANVAS-001

> **AI Worktree Graph Canvas — 基本設計書 v1.0** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟢 Draft v1.0 (2026-09-15, 需求已落档, 基本設計派生)
> - 上游: [`docs/requirements/SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.0 (115 项需求: FR-WT 38 + FR-UI 14 + FR-GRAPH 12 + FR-RISK 11 + FR-AGENT 8 + FR-EXPLAIN 4 + FR-SEARCH 6 + FR-ACTION 10 + NFR 23)
> - 下游: 詳細設計 [`docs/design/DD-WORKTREE-CANVAS-001.md`](DD-WORKTREE-CANVAS-001.md) v1.0 (本 commit 同期落档) + 实装代码
> - 关联追踪矩阵: [`docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md`](TRACEABILITY-WORKTREE-CANVAS-001.md) v1.0 (本 commit 同期落档)
> - 守门基线: 守门 #1+#3+#5+#6+#9+#10+#11+#13+#14 v3+#14 v4+#22+#28+#29 共 14 项必过
> - 平行参考: `docs/requirements/SRS-CANVAS-001.md` v1.1 (无限画布总册) + `docs/frontend-canvas-design.md` v0.1 (V0.1 实装基线) + `SRS-AGENT-VIEW-001.md` v1.0 (个体视图) + `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (ARG)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3 Mavis 永久代签)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)
> - 日期: 2026-09-15 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

---

## §0 文档目的 + 决策点 + 继承清单

### 0.1 文档目的

本文档按 日本 IPA SEC 標準 (情報システム等の整備に係る標準的指針 / 基本設計書 テンプレート) 制定 **AI Worktree Graph Canvas** 的基本設計書, 涵蓋:

(a) 系统整体架构 + 模块边界 (14 模块, per §1)
(b) 组件关系 (per §1.2)
(c) 9 大子模块架构 (Git Observer / Graph Core / Risk Engine / Health Engine / Agent Bridge / Canvas Renderer / Layout Engine / Query Engine / Action Engine, per §2-§11)
(d) 6 个支撑模块 (Event Bus / Persistence / Cache / State Management, per §12-§15)
(e) 数据模型 + Node / Edge 字段 (per §17-§18, MVP 重点)
(f) Worktree 状态机 (7 Human State, per §20)
(g) Health / Risk 计算模型 (per §21-§22)
(h) Semantic Zoom + Focus Mode + View Mode 设计 (per §23-§25)
(i) Inspector + Search DSL (per §26)
(j) API 概览 + Event 定义 + 数据流 + 时序图 (per §27-§30)
(k) 异常处理 + 并发控制 + 性能策略 (per §31-§33)
(l) 安全设计 + 可观测性 + 测试策略 (per §34-§36)
(m) Requirements Traceability 115 项 (per §37)

**MVP 范围** (per SRS §28 派生):
- 5 Node (Repository / Mainline / Worktree / Task / AgentSession)
- 7 Human State (RUNNING/WAITING/READY/DIVERGED/CONFLICT/MERGED/STALE)
- 13 Edge (全类型支持, MVP UI 默认显示 4 类: DERIVED_FROM / DEPENDS_ON / CONFLICTS_WITH / MERGED_INTO)
- 8 Action (Create/Open/Compare/Sync/Merge/Lock/Delete/Cleanup)
- 5 View Mode (TREE/DEPENDENCY/RISK/AGENT/HISTORY)
- 6 Semantic Zoom (L0-L5)

### 0.2 决策点 (Decision Points, BD 拍板)

| D-ID | 决策 | 推荐方案 | 备选 | 触发 | 拍板状态 |
|---|---|---|---|---|---|
| D-GRAPH-001 | Graph DB 选型 | **Neo4j 5.x** (成熟 + Cypher + Causal Cluster) | Memgraph / Embedded (Sled/Roapi) / Custom | FR-GRAPH-001, BR-17 | BD 拍板 |
| D-CANVAS-001 | Canvas Renderer 选型 | **React-Flow 11.x** (MVP) → 1000+ WT 切换 **PixiJS** | 自研 WebGL/Canvas | FR-UI-001, BR-18, NFR-PERF-002 | BD 拍板 |
| D-GIT-001 | Git Adapter 选型 | **libgit2** (Rust 原生 + 跨平台) | git CLI (subprocess) / Gitoxide (Rust) | FR-WT-029/033/035, BR-19, NFR-REL-001 | BD 拍板 |
| D-AGENT-001 | Agent Runtime 选型 | **Multica 内置** (MVP) + 适配 Claude Code / Codex / OpenCode | 仅 Multica | FR-AGENT-001, BR-19 | BD 拍板 |
| D-LLM-001 | LLM 选型 (Explanation) | **Claude Sonnet** (质量 + 速度平衡) | GPT-4o / Llama-3-70B (本地) | FR-EXPLAIN-001, NFR-SEC-001 | BD 拍板 |
| D-CRDT-001 | CRDT 协同算法 | **Yjs** (成熟 + React 集成) | Automerge / LWW | per `SRS-CANVAS-AGENT-001` A12.6 | BD 拍板 |
| D-INDEX-001 | Graph DB Index 选型 | **Composite Index** (Worktree.id + human_state) + **Range Index** (Health.value) | 单字段 Index | NFR-PERF-003 | BD 拍板 |
| D-CACHE-001 | Cache 层选型 | **L1 moka (Rust in-mem LRU) + L2 Redis 7.x + L3 Graph DB 自身** | 仅 L1 / 仅 L2 | NFR-CACHE-001, NFR-PERF-005 | BD 拍板 |
| D-AUDIT-001 | Audit Log 存储 | **PostgreSQL 分区表 (按月) + S3 冷归档 (1 年后)** | 仅 PG / PG + ClickHouse | NFR-AUDIT-001 | BD 拍板 |
| D-EVENT-001 | Event Bus 选型 | **Redis Streams** (持久 + 消费者组) + **WebSocket/SSE** (UI 推送) | Kafka / NATS / PostgreSQL LISTEN/NOTIFY | §25 + §28 + NFR-REL-002 | BD 拍板 |
| D-LAYOUT-001 | Layout 算法选型 | **dagre.js** (TREE 派生树) + **d3-force** (Graph Edge 平衡) + **ELK.js** (复杂图) | 自研 Sugiyama | FR-UI-001, NFR-PERF-002 | BD 拍板 |
| D-SEARCH-001 | Search DSL Parser | **自研 PEG.js 风格** + **chevrotain** (TS) | PEG.js / 近邻表达式 | FR-SEARCH-001, FR-SEARCH-003 | BD 拍板 |
| D-RISK-001 | Risk Score 阈值 | **0.7 = 高风险, 0.4-0.7 = 中风险, <0.4 = 低风险** | 0.5/0.8 单阈值 | FR-RISK-006 | BD 拍板 |
| D-HEALTH-001 | Health Score 算法 | **加权扣分制 (13 因素权重表 §21.3)** + **下限 0** | 神经网络 / 决策树 | FR-WT-006 | BD 拍板 |
| D-COLOR-001 | 颜色编码 | **D3 SchemeCategory10 + StatusPill 60+ 色码复用** + 色盲友好调色板 | 自定义调色板 | FR-WT-007, §34 UI 原则 | BD 拍板 |

**拍板原则** (per §三十五): "可以指出备选方案, 但必须选定推荐方案"; MVP 优先推荐方案, P2 阶段允许切换备选。

### 0.3 Requirements 继承清单 (per §三十一 Requirements Traceability)

本 BD 继承 SRS v1.0 全部 115 项需求 ID, 详细追踪矩阵见 §37 + 独立 `TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0。

继承大类:

| 需求大类 | 数量 | BD 章节映射 |
|---|---|---|
| FR-WT (Worktree 核心) | 38 | §17-§22 + §24-§26 |
| FR-UI (无限画布 UI) | 14 | §23-§26 |
| FR-GRAPH (Graph 数据模型) | 12 | §5 + §17-§18 |
| FR-RISK (Risk Engine) | 11 | §6 |
| FR-AGENT (Agent 集成) | 8 | §8 |
| FR-EXPLAIN (AI Explanation) | 4 | §16 |
| FR-SEARCH (搜索 + 过滤) | 6 | §26 |
| FR-ACTION (操作 + 权限) | 10 | §22 |
| NFR (非功能需求) | 23 | §31-§36 |
| **总计** | **126** | (per 自审校核, 含 11 NFR-NFR 边界) |

---

## §1 系统整体架构

### 1.1 系统拓扑

```
┌─────────────────────────────────────────────────────────────────────┐
│  Browser (Next.js 14.2.5 SPA)                                       │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  /worktree-canvas 页面                                        │ │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐  │ │
│  │  │ Canvas Toolbar  │ │ Canvas View     │ │ Inspector Panel │  │ │
│  │  │ (View/Zoom/...) │ │ (React-Flow)    │ │ (11 tabs)       │  │ │
│  │  └─────────────────┘ └─────────────────┘ └─────────────────┘  │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │ zustand Store (worktrees/graph/events/viewport)         │ │ │
│  │  └─────────────────────────────────────────────────────────┘ │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                          │ HTTP/SSE/WebSocket                     │
└──────────────────────────┼──────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  BFF (Node.js + tRPC, port 3000)                                    │
│  - REST API (OpenAPI 3.0, 11 endpoints, per SRS §8.1)              │
│  - SSE Event Stream (15 events, per SRS §8.2)                       │
│  - RBAC 权限检查 (5 角色, per NFR-PERM-001)                          │
│  - Action 验证 + Idempotency (per FR-ACTION-003/006)               │
│  - Audit Log 写入 (per FR-ACTION-005)                                │
└──────────────────────────┬──────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Core Services (Rust 1.80+, port 50051-50065, 14 modules)          │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │
│  │Graph    │ │Git      │ │Risk     │ │Health   │ │Worktree │      │
│  │Core     │ │Observer │ │Engine   │ │Engine   │ │Service  │      │
│  │(Neo4j)  │ │(libgit2)│ │(11 risk)│ │(13 fac) │ │(CRUD)   │      │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │
│  │Agent    │ │Relation │ │Canvas   │ │Layout   │ │Query    │      │
│  │Bridge   │ │Engine   │ │Renderer │ │Engine   │ │Engine   │      │
│  │(Trait)  │ │(Edge)   │ │(React-  │ │(dagre)  │ │(Cypher) │      │
│  │         │ │         │ │ Flow)   │ │         │ │         │      │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                   │
│  │Action   │ │Event    │ │Persist- │ │Cache    │                   │
│  │Engine   │ │Bus      │ │ence     │ │(moka+   │                   │
│  │(18 act) │ │(Redis   │ │(PG)     │ │ Redis)  │                   │
│  │         │ │ Stream) │ │         │ │         │                   │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘                   │
└──────────────────────────┬──────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Storage Layer                                                      │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────┐ │
│  │ Neo4j 5.x    │ │ PostgreSQL  │ │ Redis 7.x   │ │ S3 Cold    │ │
│  │ (Graph DB)   │ │ 15+ (Audit/ │ │ (Cache +    │ │ (Archive   │ │
│  │ 11 Node +    │ │ Risk/Health │ │ Event Bus + │ │ Audit 1y+) │ │
│  │ 13 Edge +    │ │ history)    │ │ SSE pubsub) │ │            │ │
│  │ Index)       │ │             │ │             │ │            │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  External (Git Provider + Agent Runtime + LLM)                      │
│  - libgit2 (git CLI fallback)                                       │
│  - Multica Agent Runtime + Claude Code / Codex / OpenCode          │
│  - Claude Sonnet (Explanation LLM)                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 组件关系 (Mermaid)

```mermaid
graph TB
    subgraph Browser["Browser SPA"]
        UI[Worktree Canvas Page]
        Store[zustand Store]
        UI --> Store
    end

    subgraph BFF["BFF (Node.js)"]
        API[REST API]
        SSE[SSE Event Stream]
        RBAC[RBAC Permission]
        ACTION_VAL[Action Validator]
    end

    subgraph Core["Core Services (Rust)"]
        GC[Graph Core]
        GO[Git Observer]
        RE[Risk Engine]
        HE[Health Engine]
        WS[Worktree Service]
        AB[Agent Bridge]
        RE_ENGINE[Relationship Engine]
        CR[Canvas Renderer State]
        LE[Layout Engine]
        QE[Query Engine]
        AE[Action Engine]
        EB[Event Bus]
        PS[Persistence]
        CH[Cache]
    end

    subgraph Storage["Storage"]
        NEO[Neo4j]
        PG[PostgreSQL]
        REDIS[Redis]
        S3[S3 Cold]
    end

    subgraph External["External"]
        GIT[libgit2]
        AGENT[Agent Runtime]
        LLM[Claude Sonnet]
    end

    UI -->|HTTP| API
    UI -->|SSE| SSE
    API --> RBAC
    API --> ACTION_VAL
    API --> AE
    SSE --> EB
    RBAC --> PG
    ACTION_VAL --> AE
    AE --> WS
    AE --> RE
    AE --> HE
    AE --> AB
    AE --> PS
    AE --> CH
    WS --> GC
    WS --> GO
    WS --> GIT
    GO --> GIT
    GO --> EB
    RE --> GC
    RE --> GO
    RE --> EB
    HE --> GC
    HE --> RE
    HE --> EB
    GC --> NEO
    GC --> CH
    QE --> GC
    QE --> CH
    LE --> GC
    LE --> CH
    RE_ENGINE --> GC
    AB --> AGENT
    AB --> EB
    EB --> REDIS
    EB --> SSE
    PS --> PG
    PS --> S3
    CH --> REDIS
    CR --> LLM
```

### 1.3 数据流 (4 类)

| 数据流 | 路径 | 频率 | 优化 |
|---|---|---|---|
| **Git → Graph** | libgit2 → GitObserver → EventBus → GraphCore → Neo4j | 高频 (每次 commit) | 增量 + 防抖 100ms |
| **Risk → Health** | RiskEngine → GraphCore → HealthEngine → Neo4j + Cache | 中频 (10s 一次) | L1 Cache 命中跳过 |
| **Graph → UI** | Neo4j → QueryEngine → BFF API → zustand Store → Canvas | 低频 (用户操作时) | L2 Cache + Incremental Query |
| **Event → UI** | EventBus (Redis Streams) → BFF SSE → zustand → Canvas | 实时 (15 类事件) | SSE keep-alive + Reconnect |

### 1.4 关键设计原则 (per SRS §二 R1-R10)

| 原则 | BD 实现 |
|---|---|
| R1 Human-first | 7 Human State 压缩 + R7 异常优先 + AI Explanation |
| R2 Worktree-first | Worktree 卡片 12 字段 (MVP) + 默认突出 |
| R3 Graph-native | 13 Edge 类型 + Tree Backbone + Semantic Graph Edge |
| R4 AI-native | AgentSession 一等实体 + AGENT VIEW + LLM Explanation |
| R5 Progressive Disclosure | Inspector 11 tab 渐进 + Search DSL |
| R6 Semantic Zoom | 6 级 LOD (L0-L5) |
| R7 Exception-first | RISK VIEW + 风险节点发光 + 正常弱化 |
| R8 Explainable State | Health 13 因素可解释 + Risk reason + LLM Explanation |
| R9 Non-destructive Visualization | 画布只读 + 18 Action 走 ActionEngine |
| R10 Graph as Context | Neo4j Graph 同时服务 UI + AI Agent |

---

## §2 模块边界 (14 模块, per SRS §二十四)

### 2.1 模块清单 + 职责

| # | 模块 | 职责 | 对应 SRS | 拍板 |
|---|---|---|---|---|
| M-01 | **Graph Core** | Graph Repository Trait + 11 Node + 13 Edge + Cypher Query | FR-GRAPH-001..012 | D-GRAPH-001 |
| M-02 | **Git Observer** | libgit2 监听 + 事件合并 + 防抖 + 增量推送 | FR-WT-009/010/011, NFR-REL-002 | D-GIT-001 |
| M-03 | **Git Adapter** | libgit2 封装 + git CLI fallback + 跨平台 | FR-WT-033/035, NFR-SEC-002 | D-GIT-001 |
| M-04 | **Worktree Service** | Worktree CRUD + 18 Action 执行 + Idempotency | FR-WT-001..038 | — |
| M-05 | **Relationship Engine** | 13 Edge 创建/查询 + Conflict Prediction | FR-RISK-005, FR-WT-031/032 | — |
| M-06 | **Risk Engine** | 11 Risk 检测 V1/V2/V3 + Score 计算 + 解决跟踪 | FR-RISK-001..011 | D-RISK-001 |
| M-07 | **Health Engine** | Health Score 13 因素加权 + 扣分明细 + 趋势 | FR-WT-006/027/028 | D-HEALTH-001 |
| M-08 | **Agent Bridge** | AgentRuntime Trait + 4 域适配 (Multica/Claude Code/Codex/OpenCode) | FR-AGENT-001..008 | D-AGENT-001 |
| M-09 | **Canvas Renderer** | Canvas Renderer Trait + React-Flow / PixiJS 适配 | FR-UI-001..014 | D-CANVAS-001 |
| M-10 | **Layout Engine** | dagre + d3-force + ELK.js + Incremental Layout | FR-UI-001/010, NFR-PERF-002 | D-LAYOUT-001 |
| M-11 | **Query Engine** | Cypher 翻译 + Search DSL Parser + NL Query 翻译 | FR-SEARCH-001/002/003 | D-SEARCH-001 |
| M-12 | **Action Engine** | 18 Action 抽象 + 3 分类 + Idempotency + Audit | FR-ACTION-001..010 | — |
| M-13 | **Event Bus** | 15 事件 + Redis Streams + 4 消费者 (Graph/UI/Risk/Notification) | §25 + §28 | D-EVENT-001 |
| M-14 | **Persistence Layer** | PostgreSQL (Audit/Risk/Health) + S3 Cold Archive | NFR-AUDIT-001, FR-ACTION-005 | D-AUDIT-001 |

### 2.2 模块解耦规则

- **零直接依赖**: 14 模块之间不允许直接调用, 全部通过 Event Bus + Trait Interface 通信
- **Trait 优先**: 4 大 Trait (`GraphRepository` / `CanvasRenderer` / `GitProvider` / `AgentRuntime`) 是模块边界
- **API 优先**: 模块间 API 调用走 BFF REST API (OpenAPI 3.0), 不允许 in-process 共享状态
- **Event 异步**: 跨模块通知走 Event Bus (Redis Streams), 同步调用仅限单次 RPC

### 2.3 物理布局 (Rust workspace + TS frontend)

```
crates/
├── graph-core/                    # M-01
├── git-observer/                  # M-02
├── git-adapter/                   # M-03
├── worktree-service/              # M-04
├── relationship-engine/           # M-05
├── risk-engine/                   # M-06
├── health-engine/                 # M-07
├── agent-bridge/                  # M-08
├── canvas-renderer-core/          # M-09 (状态层, 渲染层在 frontend)
├── layout-engine/                 # M-10
├── query-engine/                  # M-11
├── action-engine/                 # M-12
├── event-bus/                     # M-13
└── persistence/                   # M-14

frontend/src/
├── app/(worktree-canvas)/page.tsx
├── components/worktree-canvas/
│   ├── CanvasView.tsx
│   ├── WorktreeCard.tsx
│   ├── Inspector.tsx
│   ├── Toolbar.tsx
│   ├── SearchBox.tsx
│   ├── Minimap.tsx
│   └── ViewModeSelector.tsx
├── lib/worktree-canvas/
│   ├── layout.ts                  # dagre + d3-force
│   ├── search-dsl.ts              # chevrotain parser
│   ├── color-encoding.ts          # StatusPill 60+ 色码
│   └── explanation.ts             # LLM client
└── store/
    └── worktreeCanvasStore.ts     # zustand
```

---

## §3 组件关系 (Mermaid Component Diagram)

### 3.1 Worktree Canvas 完整组件图

```mermaid
graph TB
    subgraph UI["UI Components"]
        Page[WorktreeCanvasPage]
        Toolbar[Toolbar]
        CanvasView[CanvasView]
        WorktreeCard[WorktreeCard]
        EdgeView[EdgeView]
        Inspector[Inspector]
        Minimap[Minimap]
        SearchBox[SearchBox]
        ViewSelector[ViewModeSelector]
    end

    subgraph Store["zustand Store"]
        S_Worktrees[worktreesAtom]
        S_Graph[graphAtom]
        S_Events[eventsAtom]
        S_Viewport[viewportAtom]
        S_Filter[filterAtom]
        S_Mode[viewModeAtom]
    end

    subgraph Hooks["React Hooks"]
        H_UseWorktrees[useWorktrees]
        H_UseGraph[useGraph]
        H_UseEvents[useEvents SSE]
        H_UseLayout[useLayout]
        H_UseSearch[useSearch]
        H_UseExplain[useExplain]
    end

    subgraph Backend["BFF REST API"]
        API_WT[GET /worktrees]
        API_Graph[GET /worktrees/:id/graph]
        API_Health[GET /worktrees/:id/health]
        API_Risks[GET /worktrees/:id/risks]
        API_Action[POST /worktrees/:id/actions/:action]
        API_Search[POST /search]
        API_Explain[GET /explain/:worktree_id]
        API_Audit[GET /audit]
    end

    subgraph SSE["SSE Event Stream"]
        SSE_Events[/api/v1/events/stream]
    end

    Page --> Toolbar
    Page --> CanvasView
    Page --> Inspector
    Page --> Minimap
    Page --> SearchBox
    Page --> ViewSelector

    Toolbar --> S_Viewport
    Toolbar --> S_Mode
    CanvasView --> S_Worktrees
    CanvasView --> S_Graph
    CanvasView --> S_Viewport
    CanvasView --> WorktreeCard
    CanvasView --> EdgeView
    Inspector --> S_Worktrees
    SearchBox --> S_Filter
    Minimap --> S_Viewport
    ViewSelector --> S_Mode

    H_UseWorktrees --> API_WT
    H_UseGraph --> API_Graph
    H_UseEvents --> SSE_Events
    H_UseLayout --> S_Worktrees
    H_UseSearch --> API_Search
    H_UseExplain --> API_Explain

    S_Worktrees --> H_UseWorktrees
    S_Graph --> H_UseGraph
    S_Events --> H_UseEvents
```

---

## §4 Git Observer 架构

### 4.1 职责

- 监听 Git Repository 变化 (HEAD / Refs / Working Tree / Index / Worktree List)
- 监听 Agent Runtime 变化 (Agent Started / Stopped / Tool Call)
- 监听 Test State 变化 (TestRun 写入)
- 事件合并 + 防抖 (per NFR-REL-002)
- 增量推送 Event Bus

### 4.2 监听源 (7 类)

| 源 | 频率 | 事件 |
|---|---|---|
| HEAD | 中 (commit) | WorktreeChanged, BranchUpdated |
| Refs | 中 (push) | BranchUpdated, MainUpdated |
| Working Tree | 高 (每次 save) | Dirty 状态变化 |
| Index | 中 (git add) | WorktreeChanged |
| Worktree List | 低 (git worktree add/remove) | WorktreeCreated, WorktreeDeleted |
| Agent Runtime | 高 (每秒) | AgentStarted, AgentStopped, TaskChanged |
| Test State | 中 (test 完成) | TestCompleted |

### 4.3 防抖策略

```
┌─────────────────────────────────────────────────────────────┐
│  Event Watcher (per source)                                  │
│  - libgit2 notify (inotify/FSEvents)                        │
│  - fsnotify / tokio::fs::watch                              │
└────────────────────────┬────────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Debounce Aggregator                                         │
│  - 100ms window (per source)                                 │
│  - merge same event (e.g. 连续 5 次 Save 合并 1 次)          │
│  - max 1000 events in queue                                  │
└────────────────────────┬────────────────────────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Event Bus Producer (Redis Streams XADD)                     │
└─────────────────────────────────────────────────────────────┘
```

---

## §5 Graph Core 架构

### 5.1 GraphRepository Trait

```rust
#[async_trait]
pub trait GraphRepository: Send + Sync {
    /// Insert/Update a Node
    async fn upsert_node(&self, node: GraphNode) -> Result<NodeId, GraphError>;

    /// Insert/Update an Edge
    async fn upsert_edge(&self, edge: GraphEdge) -> Result<EdgeId, GraphError>;

    /// Delete a Node (and all its edges)
    async fn delete_node(&self, id: NodeId) -> Result<(), GraphError>;

    /// Delete an Edge
    async fn delete_edge(&self, id: EdgeId) -> Result<(), GraphError>;

    /// Execute a Cypher query
    async fn query(&self, cypher: &str, params: HashMap<String, Value>) -> Result<Vec<Row>, GraphError>;

    /// N-hop neighborhood traversal
    async fn traverse(
        &self,
        start: NodeId,
        edge_types: &[EdgeType],
        max_hop: u8,  // 1-3
        direction: TraversalDirection,  // Out / In / Both
    ) -> Result<Vec<GraphNode>, GraphError>;
}
```

### 5.2 Neo4j 实现 (D-GRAPH-001 推荐)

```rust
pub struct Neo4jRepository {
    driver: neo4j::Driver,
    database: String,
}

impl Neo4jRepository {
    pub async fn new(uri: &str, user: &str, pass: &str) -> Result<Self> {
        let driver = neo4j::Driver::new(uri, user, pass).await?;
        Ok(Self { driver, database: "neo4j".to_string() })
    }
}

#[async_trait]
impl GraphRepository for Neo4jRepository {
    async fn upsert_node(&self, node: GraphNode) -> Result<NodeId, GraphError> {
        let session = self.driver.session(&self.database).await?;
        let cypher = match node {
            GraphNode::Worktree(wt) => format!(
                "MERGE (n:Worktree {{id: $id}}) SET n = $props RETURN n.id",
                wt
            ),
            // ... 其他 10 种 Node
        };
        let result = session.run(cypher, node.to_params()).await?;
        Ok(result.into())
    }

    async fn traverse(
        &self,
        start: NodeId,
        edge_types: &[EdgeType],
        max_hop: u8,
        direction: TraversalDirection,
    ) -> Result<Vec<GraphNode>, GraphError> {
        let cypher = format!(
            "MATCH (n {{id: $start}}) \
             MATCH (n)-[r:{}*1..{}]-{}(m) \
             RETURN DISTINCT m",
            edge_types_to_cypher(edge_types), max_hop, direction_to_cypher(direction),
        );
        // ... 执行 + 反序列化
    }
}
```

### 5.3 Graph Schema (Cypher DDL)

```cypher
// ============ 11 Node 类型 ============
CREATE CONSTRAINT worktree_id IF NOT EXISTS ON (n:Worktree) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT repo_id IF NOT EXISTS ON (n:Repository) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT branch_id IF NOT EXISTS ON (n:Branch) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT mainline_id IF NOT EXISTS ON (n:Mainline) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT commit_id IF NOT EXISTS ON (n:Commit) ASSERT n.sha IS UNIQUE;
CREATE CONSTRAINT task_id IF NOT EXISTS ON (n:Task) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT agent_id IF NOT EXISTS ON (n:AgentSession) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT pr_id IF NOT EXISTS ON (n:PullRequest) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT file_id IF NOT EXISTS ON (n:File) ASSERT (n.repo_id, n.path) IS UNIQUE;
CREATE CONSTRAINT symbol_id IF NOT EXISTS ON (n:Symbol) ASSERT (n.file_id, n.qualified_name) IS UNIQUE;
CREATE CONSTRAINT test_id IF NOT EXISTS ON (n:TestRun) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT issue_id IF NOT EXISTS ON (n:Issue) ASSERT n.id IS UNIQUE;

// ============ Index ============
CREATE INDEX worktree_branch IF NOT EXISTS ON (n:Worktree) (n.repo_id, n.branch);
CREATE INDEX worktree_human_state IF NOT EXISTS ON (n:Worktree) (n.human_state);
CREATE INDEX worktree_health IF NOT EXISTS ON (n:Worktree) (n.health_score);
CREATE INDEX worktree_last_activity IF NOT EXISTS ON (n:Worktree) (n.last_activity);
CREATE INDEX agent_status IF NOT EXISTS ON (n:AgentSession) (n.status);
CREATE INDEX task_status IF NOT EXISTS ON (n:Task) (n.status);

// ============ 13 Edge 类型 ============
// (Edge 类型由 Cypher 查询指定, 不需要预先 CREATE, MERGE 时自动创建)
```

### 5.4 Graph Index 策略 (D-INDEX-001)

| 索引类型 | 字段 | 加速查询 |
|---|---|---|
| Unique Constraint | Worktree.id | 按 ID 查询 |
| Composite Index | (repo_id, branch) | 按 Repository + Branch 过滤 |
| Composite Index | (repo_id, human_state) | 按 Repository + 状态过滤 |
| Range Index | health_score | Health 排序 / <50 过滤 |
| Range Index | last_activity | Stale 过滤 (>7d) |
| Index | AgentSession.status | AGENT VIEW 过滤 |

---

## §6 Risk Engine 架构

### 6.1 11 Risk 类型定义

| Risk ID | 类型 | 检测源 | V 阶段 | Score 公式 |
|---|---|---|---|---|
| RISK-001 | MergeConflict | git merge-base | V1 | 1.0 (hard) |
| RISK-002 | FileOverlap | diff file list intersect | V1 | min(1, shared_files / 10) |
| RISK-003 | SymbolOverlap | tree-sitter symbol overlap | V3 | min(1, shared_symbols / 5) |
| RISK-004 | Stale | last_activity > 7d | V1 | 1.0 if > 30d, 0.7 if > 7d |
| RISK-005 | Divergence | ahead > 5 AND behind > 5 | V1 | min(1, (ahead + behind) / 50) |
| RISK-006 | Dependency | DEPENDS_ON Edge unresolved | V1 | 1.0 if blocked |
| RISK-007 | TestFailure | TestRun.state = failed | V1 | 1.0 |
| RISK-008 | BuildFailure | CI status = failed | V1 | 1.0 |
| RISK-009 | AgentIncomplete | AgentSession.status not in {completed, failed} | V1 | 0.7 |
| RISK-010 | ReviewMissing | PR created but no review | V1 | 0.5 |
| RISK-011 | Superseded | SUPERSEDES Edge exists | V1 | 1.0 (do not merge) |

### 6.2 V1 → V2 → V3 演进路径

| V 阶段 | 检测能力 | 依赖 | 实装期 |
|---|---|---|---|
| **V1** | Git Status + Ahead/Behind + File Overlap (path intersect) | libgit2 + Git Status | MVP (P0) |
| **V2** | Diff Overlap + Line Overlap (unified diff 三方合并预演) | libgit2 + diff3 算法 | V1.1 (P1) |
| **V3** | Symbol Overlap + AST-level Overlap (tree-sitter 解析) | tree-sitter (Rust/TS/Python/Go/Java) | V1.2 (P2) |

### 6.3 Conflict Prediction 算法 (FR-RISK-005)

```rust
async fn predict_conflict(wt_a: WorktreeId, wt_b: WorktreeId) -> Result<ConflictPrediction> {
    // 1. 获取 2 Worktree 的修改文件列表
    let files_a = git_diff_files(wt_a, merge_base(wt_a, wt_b))?;
    let files_b = git_diff_files(wt_b, merge_base(wt_a, wt_b))?;

    // 2. 计算 shared files
    let shared_files: Vec<_> = files_a.intersection(&files_b).collect();

    // 3. 对每个 shared file, 计算 Line Overlap (V2)
    let mut shared_symbols = vec![];
    for file in &shared_files {
        let diff_a = parse_diff(wt_a, file)?;
        let diff_b = parse_diff(wt_b, file)?;
        // 三方合并预演: 检测 line-level conflict
        let conflicts = three_way_merge_preview(diff_a, diff_b)?;
        shared_symbols.extend(conflicts);
    }

    // 4. 计算 Risk Score
    let risk_score = if shared_files.is_empty() {
        0.0
    } else {
        min(1.0, (shared_files.len() as f32 / 10.0) + (shared_symbols.len() as f32 / 5.0) * 0.5)
    };

    // 5. 生成 CONFLICTS_WITH Edge
    let reason = format!(
        "WT-{} and WT-{} modified {} shared files ({} shared symbols)",
        wt_a, wt_b, shared_files.len(), shared_symbols.len()
    );

    Ok(ConflictPrediction {
        risk_score,
        shared_files,
        shared_symbols,
        reason,
        confidence: 0.95,
    })
}
```

---

## §7 Health Engine 架构

### 7.1 Health Score 13 因素 (FR-WT-006)

| # | 因素 | 权重 | 检测 | 扣分公式 |
|---|---|---|---|---|
| 1 | MergeConflict | -25 | CONFLICTS_WITH Edge exists | -25 if exists |
| 2 | BehindMain | -10 | behind > 20 | min(-10, -(behind / 10)) |
| 3 | Ahead | -5 | ahead > 30 | min(-5, -(ahead / 30)) |
| 4 | Dirty | -5 | git status dirty | -5 if dirty |
| 5 | TestFailure | -20 | TestRun.state = failed | -20 if failed |
| 6 | BuildFailure | -15 | CI status = failed | -15 if failed |
| 7 | CodeReviewMissing | -10 | PR no review | -10 if PR open > 3d no review |
| 8 | AgentIncomplete | -10 | Agent.status not terminal | -10 if incomplete |
| 9 | Inactivity | -10 | last_activity > 7d | -10 if > 7d, -15 if > 30d |
| 10 | FileOverlap | -15 | shared_files > 5 with other WT | min(-15, -(shared / 5)) |
| 11 | SymbolOverlap | -10 | shared_symbols > 3 | min(-10, -(symbols / 3)) |
| 12 | Dependency | -10 | DEPENDS_ON Edge unresolved | -10 if blocked |
| 13 | Blocked | -10 | BLOCKS Edge unresolved | -10 if blocked |

### 7.2 Health Score 计算公式

```rust
fn compute_health_score(worktree: &Worktree, context: &HealthContext) -> HealthScore {
    let mut score = 100_u8;
    let mut deductions = vec![];

    // 1. Conflict (最严重)
    if context.has_conflict {
        score = score.saturating_sub(25);
        deductions.push(Deduction { factor: "MergeConflict".into(), points: -25, reason: "WT has CONFLICTS_WITH edges".into() });
    }

    // 2. TestFailure
    if context.test_state == TestState::Failed {
        score = score.saturating_sub(20);
        deductions.push(Deduction { factor: "TestFailure".into(), points: -20, reason: "Test run failed".into() });
    }

    // ... 其他 11 因素

    // 3. Behind
    if worktree.behind > 20 {
        let pts = (worktree.behind / 10).min(10) as u8;
        score = score.saturating_sub(pts);
        deductions.push(Deduction { factor: "BehindMain".into(), points: -(pts as i8), reason: format!("Behind main by {} commits", worktree.behind) });
    }

    // ... 其他扣分

    HealthScore {
        value: score,
        deductions,
        computed_at: Utc::now(),
    }
}
```

### 7.3 Health 阈值 (D-HEALTH-001)

| Health 范围 | 状态 | UI 表现 |
|---|---|---|
| 80-100 | HEALTHY | 绿色环 |
| 50-79 | WARNING | 黄色环 |
| 0-49 | CRITICAL | 红色环 + ⚠ 图标 |

---

## §8 Agent Bridge 架构

### 8.1 AgentRuntime Trait

```rust
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    /// Get single session
    async fn get_session(&self, id: AgentId) -> Result<AgentSession, AgentError>;

    /// List sessions (filter by worktree, task, status)
    async fn list_sessions(&self, filter: AgentFilter) -> Result<Vec<AgentSession>, AgentError>;

    /// Subscribe to events (Started, Stopped, ToolCall, etc.)
    async fn subscribe_events(&self) -> Result<Box<dyn Stream<Item = AgentEvent> + Send>, AgentError>;

    /// Get metrics (token_usage, tool_calls, last_activity)
    async fn get_session_metrics(&self, id: AgentId) -> Result<AgentMetrics, AgentError>;
}
```

### 8.2 4 大 Agent 适配器 (D-AGENT-001 推荐 Multica 内置 + 3 适配)

| 适配器 | 协议 | MVP | 拍板 |
|---|---|---|---|
| **MulticaAdapter** | Multica 内部 API | ✅ | D-AGENT-001 推荐 |
| **ClaudeCodeAdapter** | Claude Code JSON-RPC | P1 | D-AGENT-001 备选 |
| **CodexAdapter** | Codex CLI stream-json | P1 | D-AGENT-001 备选 |
| **OpenCodeAdapter** | OpenCode 协议 | P2 | D-AGENT-001 备选 |

---

## §9 Canvas 架构

### 9.1 CanvasRenderer Trait (M-09 状态层)

```rust
#[async_trait]
pub trait CanvasRenderer: Send + Sync {
    /// Initialize canvas
    async fn init(&self, viewport: Viewport) -> Result<(), CanvasError>;

    /// Render nodes (LOD-aware)
    async fn render_nodes(&self, nodes: Vec<RenderNode>, lod: LodLevel) -> Result<(), CanvasError>;

    /// Render edges
    async fn render_edges(&self, edges: Vec<RenderEdge>) -> Result<(), CanvasError>;

    /// Update viewport (pan/zoom)
    async fn update_viewport(&self, viewport: Viewport) -> Result<(), CanvasError>;

    /// Highlight focus node + N-hop neighbors
    async fn focus(&self, node_id: NodeId, hop: u8) -> Result<(), CanvasError>;

    /// Filter (show only matching nodes/edges)
    async fn filter(&self, filter: Filter) -> Result<(), CanvasError>;
}
```

### 9.2 React-Flow 11.x 适配 (MVP, D-CANVAS-001)

| 优势 | 限制 | 缓解 |
|---|---|---|
| 成熟 + React 集成好 | 500+ 节点卡顿 | LOD + Clustering (FR-UI-009/010) |
| 内置 pan/zoom/minimap | 大规模节点需手动优化 | React.memo + 虚拟化 |
| 自定义 Node / Edge 简单 | 不支持 WebGL | 1000+ 切换 PixiJS (D-CANVAS-001 备选) |

### 9.3 PixiJS 备选 (1000+ WT)

- WebGL 加速, 支持 5000+ 节点流畅
- 集成复杂度高 (需自研 React-Pixi 桥接)
- MVP 不启用, P1.5 性能不达预期切换

---

## §10 Layout Engine 架构

### 10.1 Layout 算法分层 (D-LAYOUT-001)

| 算法 | 用途 | 库 |
|---|---|---|
| **dagre.js** | TREE 派生树布局 (Main → Worktree) | `@dagrejs/dagre` |
| **d3-force** | Graph Edge 平衡 (CONFLICTS/DEPENDS 边) | `d3-force` |
| **ELK.js** | 复杂图布局 (>200 节点含多层 Edge) | `elkjs` |

### 10.2 Incremental Layout 策略 (NFR-PERF-002)

- 节点拖动时, 不全量重排, 只更新受影响区域
- 新增节点时, 在 Parent 附近插入, 不触发全局 layout
- 删除节点时, 只 collapse 子树, 不重排

### 10.3 Layout 输出

```typescript
type LayoutOutput = {
    nodes: Array<{
        id: NodeId;
        x: number;
        y: number;
        width: number;
        height: number;
    }>;
    edges: Array<{
        from: NodeId;
        to: NodeId;
        path: string;  // SVG path "M x1 y1 C cx1 cy1 cx2 cy2 x2 y2"
    }>;
    bbox: { x: number; y: number; width: number; height: number };
};
```

---

## §11 Query Engine 架构

### 11.1 Query Engine 流程

```
DSL Query ("show:conflict behind:>20 agent:codex")
  ↓ Search DSL Parser (chevrotain)
Filter { states: [CONFLICT], behind_min: 20, agent: codex }
  ↓ Query Builder
Cypher: "MATCH (w:Worktree)-[:WORKS_ON]->(a:AgentSession)
         WHERE w.human_state = 'CONFLICT' AND w.behind > 20 AND a.agent_type = 'codex'
         RETURN w"
  ↓ GraphRepository.query()
Result Set
  ↓ Frontend Filter
Highlight nodes + dim others
```

### 11.2 Cypher 查询示例 (FR-GRAPH-007)

```cypher
// N-hop Neighborhood (FR-GRAPH-012)
MATCH (w:Worktree {id: $id})-[r:CONFLICTS_WITH|DEPENDS_ON|BLOCKS*1..3]-(m:Worktree)
RETURN DISTINCT m, r, w

// Search DSL: show:unmerged behind:>10 agent:codex
MATCH (w:Worktree)-[:WORKS_ON]->(a:AgentSession)
WHERE w.human_state <> 'MERGED'
  AND w.behind > 10
  AND a.agent_type = 'codex'
RETURN w

// TREE VIEW: Main → Worktree 派生
MATCH (main:Mainline {repo_id: $repo_id})<-[:DERIVED_FROM*]-(w:Worktree)
RETURN main, w
```

---

## §12 Action Engine 架构

### 12.1 18 Action + 3 分类 (D 决策)

#### 12.1.1 Safe (5 项, 无副作用)

| Action | Description | Risk Class |
|---|---|---|
| `Open` | Open Worktree in file browser | Safe |
| `OpenInIDE` | Open in VSCode / Cursor | Safe |
| `Compare` | Compare 2 Worktrees (read-only diff) | Safe |
| `Focus` | Enter Focus Mode | Safe |
| `ExplainRisk` | AI Explanation (read-only) | Safe |

#### 12.1.2 Warning (8 项, 有副作用但可撤销)

| Action | Description | Risk Class |
|---|---|---|
| `SyncMain` | git fetch + rebase onto main | Warning |
| `Rebase` | git rebase onto target branch | Warning |
| `Merge` | git merge Worktree to main | Warning |
| `CreatePR` | Open PR via GitHub/GitLab API | Warning |
| `Lock` / `Unlock` | Lock / Unlock Worktree | Warning |
| `Archive` | Archive (not delete) | Warning |
| `MarkSuperseded` | Mark Worktree as Superseded | Warning |
| `SetDependency` / `RemoveDependency` | Manage DEPENDS_ON Edge | Warning |

#### 12.1.3 Destructive (5 项, 高风险不可撤销)

| Action | Description | Risk Class |
|---|---|---|
| `Delete` | git worktree remove + branch delete | Destructive |
| `Cleanup` | Batch delete STALE Worktrees | Destructive |
| `ForceMerge` | Merge ignoring conflicts | Destructive |
| `ForceRebase` | Rebase dropping commits | Destructive |
| `ForceDelete` | Delete without confirmation | Destructive |

### 12.2 Action Engine Trait

```rust
#[async_trait]
pub trait ActionEngine: Send + Sync {
    /// Execute action (with Idempotency Key)
    async fn execute(
        &self,
        action: ActionRequest,
        idempotency_key: Uuid,
    ) -> Result<ActionResult, ActionError>;

    /// Validate action (RBAC + Pre-condition check)
    async fn validate(&self, action: &ActionRequest, user: &User) -> Result<(), ActionError>;

    /// Get action metadata (classification, description)
    fn metadata(&self, action_type: ActionType) -> ActionMetadata;

    /// List available actions for user + worktree
    async fn list_available(
        &self,
        user: &User,
        worktree: &Worktree,
    ) -> Result<Vec<ActionType>, ActionError>;
}
```

### 12.3 二次确认机制 (FR-ACTION-003)

```typescript
type ConfirmDialog = {
    title: string;
    description: string;
    impact: {
        affected_count: number;
        affected_paths: string[];
        reversible: boolean;
    };
    confirm_text: string;        // e.g. "Delete 3 Worktrees"
    cancel_text: string;         // "Cancel"
    requires_typed_confirm?: boolean;  // Destructive: require typing "DELETE"
};

// Destructive Action 示例
const DeleteConfirm: ConfirmDialog = {
    title: "Delete Worktree?",
    description: "This will permanently delete the worktree and its branch.",
    impact: {
        affected_count: 1,
        affected_paths: [".git/worktrees/wt-xxx", "refs/heads/wt-xxx"],
        reversible: false,
    },
    confirm_text: "Delete",
    requires_typed_confirm: true,
};
```

---

## §13 Event Bus 架构

### 13.1 15 事件定义 (D-EVENT-001: Redis Streams + SSE)

| Event Type | Payload | 消费者 | 触发频率 |
|---|---|---|---|
| `WorktreeCreated` | WorktreeNode | Graph, UI, Risk | 低 |
| `WorktreeChanged` | WorktreeNode (diff) | Graph, UI, Risk | 高 |
| `WorktreeDeleted` | `{ id }` | Graph, UI | 低 |
| `WorktreeMerged` | `{ id, merged_at }` | Graph, UI, RecentlyMerged | 低 |
| `BranchUpdated` | `{ repo_id, branch, head_sha }` | Graph, UI | 中 |
| `MainUpdated` | `{ repo_id, new_head }` | Graph, Risk (Conflict), UI | 中 |
| `AgentStarted` | AgentSessionNode | Graph, UI (AGENT VIEW) | 中 |
| `AgentStopped` | `{ id, result }` | Graph, UI | 中 |
| `TaskChanged` | TaskNode | Graph, UI | 中 |
| `TestCompleted` | `{ worktree_id, test_state }` | Graph, Risk, Health | 中 |
| `RiskDetected` | RiskEvent | Graph, UI, Notification | 中 |
| `RiskResolved` | `{ risk_id }` | Graph, UI | 低 |
| `HealthChanged` | `{ worktree_id, health }` | Graph, UI | 中 |
| `RelationCreated` | Edge | Graph, UI | 中 |
| `RelationRemoved` | `{ edge_id }` | Graph, UI | 低 |

### 13.2 Event Bus 流程 (Redis Streams)

```
Event Producer (GitObserver / ActionEngine / etc.)
  ↓ XADD stream:canvas-events * type=WorktreeChanged payload={...}
Redis Stream (stream:canvas-events)
  ↓ XREADGROUP $group consumer-1 BLOCK 100
Event Consumer (GraphCore / RiskEngine / BFF SSE)
  ↓ Process Event
  ↓ XACK stream:canvas-events $group event-id
```

### 13.3 Event 防抖 (NFR-REL-002)

| Event | 防抖窗口 | 合并策略 |
|---|---|---|
| `WorktreeChanged` (Working Tree) | 100ms | Last-write-wins (同 Worktree 合并) |
| `HealthChanged` | 5s | 30s 内只推送最终值 |
| `RiskDetected` | 1s | 同 (RiskType, Worktree) 合并 |
| `TestCompleted` | 0 (即时) | 不防抖 (重要事件) |

---

## §14 Persistence Layer 架构

### 14.1 PostgreSQL Schema (per D-AUDIT-001)

```sql
-- ============ Audit Log (T 表, append-only, SCD Type 2) ============
CREATE TABLE canvas_audit (
    id BIGSERIAL PRIMARY KEY,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,                          -- NULL = current
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    actor_id UUID NOT NULL,
    actor_type VARCHAR(20) NOT NULL,               -- user / agent / system
    action_type VARCHAR(50) NOT NULL,              -- 18 Action types
    worktree_id UUID,
    repo_id UUID,
    payload JSONB NOT NULL,                        -- action params
    result VARCHAR(20) NOT NULL,                   -- success / failed / cancelled
    error_message TEXT,
    idempotency_key UUID NOT NULL UNIQUE,         -- FR-ACTION-006
    client_ip INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- 分区 (按月)
CREATE TABLE canvas_audit_2026_09 PARTITION OF canvas_audit
    FOR VALUES FROM ('2026-09-01') TO ('2026-10-01');

-- Index
CREATE INDEX idx_audit_worktree ON canvas_audit (worktree_id, created_at DESC);
CREATE INDEX idx_audit_actor ON canvas_audit (actor_id, created_at DESC);
CREATE INDEX idx_audit_action ON canvas_audit (action_type, created_at DESC);

-- ============ Risk Event Log (T 表) ============
CREATE TABLE canvas_risk_event (
    id BIGSERIAL PRIMARY KEY,
    risk_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    risk_type VARCHAR(50) NOT NULL,                -- 11 types
    risk_score REAL NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    reason TEXT,
    payload JSONB
) PARTITION BY RANGE (detected_at);

CREATE INDEX idx_risk_worktree ON canvas_risk_event (worktree_id, detected_at DESC);
CREATE INDEX idx_risk_unresolved ON canvas_risk_event (worktree_id) WHERE resolved_at IS NULL;

-- ============ Health History (T 表, time-series) ============
CREATE TABLE canvas_health_history (
    worktree_id UUID NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL,
    health_score SMALLINT NOT NULL,
    deductions JSONB NOT NULL,
    PRIMARY KEY (worktree_id, computed_at)
) PARTITION BY RANGE (computed_at);

-- ============ Search History (M 表) ============
CREATE TABLE canvas_search_history (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    query_type VARCHAR(20) NOT NULL,               -- dsl / nl
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_user ON canvas_search_history (user_id, created_at DESC);
```

### 14.2 S3 Cold Archive (1 年后)

- 1 年前的 Audit Log 归档到 S3 (Parquet 格式, 按月压缩)
- 保留 7 年 (合规)
- 元数据 (查询索引) 留在 PG

---

## §15 Cache 架构 (D-CACHE-001)

### 15.1 三级缓存

| 层级 | 存储 | 内容 | TTL | 命中率目标 |
|---|---|---|---|---|
| **L1** | moka (Rust in-mem LRU) | HealthScore / RiskScore / LayoutOutput | 5min | 50% |
| **L2** | Redis 7.x | Worktree + Graph Query Result + Explanation | 30min | 80% |
| **L3** | Neo4j 自身 | 全量数据 | 永久 | (DB cache) |

### 15.2 Cache Key 设计

```
L1: health:{worktree_id}:v1 → HealthScore
L1: risk:{worktree_id}:v1 → Risk[]
L1: layout:{repo_id}:view:{view_mode}:v1 → LayoutOutput

L2: worktree:{worktree_id} → WorktreeNode (JSON)
L2: graph:neighborhood:{worktree_id}:hop:{n} → GraphResult
L2: explanation:{worktree_id}:risk_hash → ExplanationText
L2: search:{filter_hash} → WorktreeId[]
```

### 15.3 Cache Invalidation 策略

| 事件 | 影响 Cache |
|---|---|
| `WorktreeChanged` | 失效: worktree:{id}, health:{id}, risk:{id} |
| `RiskDetected` | 失效: risk:{id}, health:{id}, explanation:{id} |
| `HealthChanged` | 失效: health:{id} |
| `RelationCreated/Removed` | 失效: graph:neighborhood:{*}, layout:{*:view:*} |
| `BranchUpdated / MainUpdated` | 失效: graph:neighborhood:{*} |

---

## §16 AI Explanation Layer 架构

### 16.1 Explanation API 流程 (FR-EXPLAIN-001)

```
Worktree ID + Context Type (state/risk/health/recommend)
  ↓ BFF API: GET /api/v1/explain/{worktree_id}?type=risk
  ↓ RBAC check (view:explanation)
  ↓ Aggregate Structured Context (per FR-EXPLAIN-002):
     {
       worktree: {...},
       risks: [...],
       health: { value, deductions },
       git_status: {...},
       conflicting_worktrees: [...],
       dependencies: [...]
     }
  ↓ Cache Check (Redis, TTL 5min)
  ↓ If miss: LLM Call (Claude Sonnet, per D-LLM-001)
  ↓ Prompt: "你只能基于以下 JSON 数据解释, 不允许添加 JSON 之外的推断...
            Output: 1-3 句话, Markdown"
  ↓ Response + Cache Write
  ↓ Return to UI (Tooltip / Full Text)
```

### 16.2 Prompt Engineering (FR-EXPLAIN-002)

```markdown
你是一个 Worktree 风险解释助手。**你只能基于以下 JSON 数据生成解释, 不允许添加任何 JSON 之外的推断**。

## 结构化数据
```json
{{ context_json }}
```

## 输出要求
- 长度: 1-3 句话
- 格式: Markdown
- 内容: 自然语言解释状态/风险/健康/推荐
- **禁止**: 凭空生成 Git 命令、推荐外部工具、添加主观判断
```

### 16.3 LLM 成本控制 (per §十三 #6)

- Cache TTL 5min (per FR-EXPLAIN-004)
- 批量 Explanation (一次 LLM 调用解释 5-10 个 Worktree)
- 降级: 模板生成 (无 LLM 时用 Mustache 模板拼接)

---

## §17 Node Model 详细

### 17.1 Worktree Node (P0, MVP 重点)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeNode {
    pub id: WorktreeId,
    pub repo_id: RepoId,
    pub name: String,
    pub branch: String,
    pub human_state: HumanState,
    pub machine_state: MachineState,
    pub ahead: u32,
    pub behind: u32,
    pub dirty: bool,
    pub health_score: HealthScore,
    pub last_activity: DateTime<Utc>,
    pub agent_id: Option<AgentId>,
    pub task_id: Option<TaskId>,
    pub risk_count: u32,
    pub test_state: TestState,
    pub locked: bool,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub parent_id: Option<WorktreeId>,
}
```

(其他 10 种 Node: Repository / Branch / Mainline / Task / AgentSession / Commit / PullRequest / File / Symbol / TestRun / Issue, 字段定义在 DD §7)

---

## §18 Edge Model 详细

### 18.1 13 Edge 类型

| Edge | From → To | 字段 |
|---|---|---|
| **BASED_ON** | Worktree → Commit | created_at |
| **USES_BRANCH** | Worktree → Branch | created_at |
| **DERIVED_FROM** | Worktree → Worktree | created_at |
| **WORKS_ON** | AgentSession → Worktree | started_at |
| **IMPLEMENTED_IN** | Task → Worktree | created_at |
| **MODIFIES** | Worktree → File | lines_added, lines_removed |
| **MODIFIES_SYMBOL** | Worktree → Symbol | lines_added, lines_removed |
| **CONFLICTS_WITH** | Worktree → Worktree | risk_score, shared_files, shared_symbols, detected_at, reason, confidence |
| **OVERLAPS_WITH** | Worktree → Worktree | overlap_score, shared_files |
| **DEPENDS_ON** | Worktree → Worktree | required_state, created_at |
| **BLOCKS** | Worktree → Worktree | created_at |
| **SUPERSEDES** | Worktree → Worktree | created_at, reason |
| **MERGED_INTO** | Worktree → Branch | merged_at, commit_sha |

(各 Edge 详细字段在 DD §8)

---

## §19 Graph Schema (Cypher DDL 示例)

```cypher
// ============ Constraints (11 Node 类型) ============
CREATE CONSTRAINT worktree_id IF NOT EXISTS ON (n:Worktree) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT repo_id IF NOT EXISTS ON (n:Repository) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT branch_id IF NOT EXISTS ON (n:Branch) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT mainline_id IF NOT EXISTS ON (n:Mainline) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT commit_sha IF NOT EXISTS ON (n:Commit) ASSERT n.sha IS UNIQUE;
CREATE CONSTRAINT task_id IF NOT EXISTS ON (n:Task) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT agent_id IF NOT EXISTS ON (n:AgentSession) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT pr_id IF NOT EXISTS ON (n:PullRequest) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT file_id IF NOT EXISTS ON (n:File) ASSERT (n.repo_id, n.path) IS UNIQUE;
CREATE CONSTRAINT symbol_id IF NOT EXISTS ON (n:Symbol) ASSERT (n.file_id, n.qualified_name) IS UNIQUE;
CREATE CONSTRAINT test_id IF NOT EXISTS ON (n:TestRun) ASSERT n.id IS UNIQUE;
CREATE CONSTRAINT issue_id IF NOT EXISTS ON (n:Issue) ASSERT n.id IS UNIQUE;

// ============ Index (13 Index) ============
CREATE INDEX worktree_branch IF NOT EXISTS ON (n:Worktree) (n.repo_id, n.branch);
CREATE INDEX worktree_human_state IF NOT EXISTS ON (n:Worktree) (n.human_state);
CREATE INDEX worktree_health IF NOT EXISTS ON (n:Worktree) (n.health_score);
CREATE INDEX worktree_last_activity IF NOT EXISTS ON (n:Worktree) (n.last_activity);
CREATE INDEX worktree_repo_state IF NOT EXISTS ON (n:Worktree) (n.repo_id, n.human_state);
CREATE INDEX worktree_locked IF NOT EXISTS ON (n:Worktree) (n.locked) WHERE n.locked = TRUE;
CREATE INDEX worktree_archived IF NOT EXISTS ON (n:Worktree) (n.archived) WHERE n.archived = TRUE;
CREATE INDEX agent_status IF NOT EXISTS ON (n:AgentSession) (n.status);
CREATE INDEX agent_type_status IF NOT EXISTS ON (n:AgentSession) (n.agent_type, n.status);
CREATE INDEX task_status IF NOT EXISTS ON (n:Task) (n.status);
CREATE INDEX file_repo IF NOT EXISTS ON (n:File) (n.repo_id);
CREATE INDEX symbol_file IF NOT EXISTS ON (n:Symbol) (n.file_id);
```

---

## §20 Worktree State Machine (7 Human State)

### 20.1 状态机定义

```mermaid
stateDiagram-v2
    [*] --> Running: WorktreeCreated
    Running --> Waiting: Agent paused
    Waiting --> Running: Agent resumed
    Running --> Ready: Health >= 80 + no risk
    Running --> Diverged: Ahead > 5 AND Behind > 5
    Running --> Conflict: CONFLICTS_WITH detected
    Ready --> Diverged: ahead/behind changed
    Ready --> Conflict: CONFLICTS_WITH detected
    Diverged --> Ready: Sync Main
    Diverged --> Conflict: CONFLICTS_WITH detected
    Conflict --> Ready: Conflict resolved
    Conflict --> Diverged: Conflict resolved but diverged
    Running --> Stale: last_activity > 7d
    Waiting --> Stale: last_activity > 7d
    Diverged --> Stale: last_activity > 7d
    Conflict --> Stale: last_activity > 7d
    Ready --> Stale: last_activity > 7d
    Stale --> Running: Activity resumed
    Running --> Merged: git merge success
    Ready --> Merged: git merge success
    Diverged --> Merged: git merge success
    Conflict --> Merged: Force merge (destructive)
    Merged --> [*]
```

### 20.2 状态进入/离开条件 (per §9.1 SRS)

(详见 SRS §9.1, BD 不重复)

---

## §21 Health Calculation Model

(详见 §7, BD 不重复)

---

## §22 Risk Calculation Model

(详见 §6, BD 不重复)

---

## §23 Semantic Zoom 设计 (6 级 L0-L5)

### 23.1 LOD 阈值

| LOD | 缩放范围 | 显示 Node | 显示 Edge |
|---|---|---|---|
| **L0 Project** | zoom < 0.2 | 仅 Repository (聚合统计) | Repository → Main |
| **L1 Repo+Worktree** | 0.2 <= zoom < 0.5 | Repository + Mainline + Worktree (聚类) | DERIVED_FROM (骨架) |
| **L2 Task+Agent** | 0.5 <= zoom < 0.8 | + Task + AgentSession | + WORKS_ON, IMPLEMENTED_IN |
| **L3 Commit** | 0.8 <= zoom < 1.2 | + Commit (折叠) | + BASED_ON |
| **L4 File** | 1.2 <= zoom < 2.0 | + File | + MODIFIES |
| **L5 Symbol** | zoom >= 2.0 | + Symbol | + MODIFIES_SYMBOL |

### 23.2 LOD 切换算法

```typescript
function getLodLevel(zoom: number): LodLevel {
    if (zoom < 0.2) return LodLevel.L0;
    if (zoom < 0.5) return LodLevel.L1;
    if (zoom < 0.8) return LodLevel.L2;
    if (zoom < 1.2) return LodLevel.L3;
    if (zoom < 2.0) return LodLevel.L4;
    return LodLevel.L5;
}

function shouldRenderNode(node: GraphNode, lod: LodLevel, viewport: Viewport): boolean {
    // LOD 过滤: 不在当前 LOD 的节点不渲染
    if (!isNodeTypeInLod(node.type, lod)) return false;

    // Viewport Virtualization: 不在视口内的节点不渲染
    if (!isInViewport(node.position, viewport, lod)) return false;

    return true;
}
```

### 23.3 距离非线性映射 (FR-WT-005)

```typescript
function visualDistance(commitDistance: number): number {
    // visualDistance = log(commitDistance + 1)
    // 0 commit → 0px, 1 commit → ~21px, 10 commit → ~70px, 100 commit → ~138px
    return Math.log(commitDistance + 1) * 30;  // 30px base
}

// 示例
visualDistance(0) === 0      // 0 commit
visualDistance(1) === ~20.79 // 1 commit
visualDistance(10) === ~69.08 // 10 commit
visualDistance(100) === 138.63 // 100 commit
visualDistance(1000) === 207.94 // 1000 commit
```

---

## §24 Focus Mode 设计 (FR-WT-038)

### 24.1 Focus 算法

```typescript
async function focus(worktreeId: WorktreeId, hop: 1 | 2 | 3): Promise<FocusResult> {
    // 1. N-hop Cypher Query
    const cypher = `
        MATCH (w:Worktree {id: $id})
        MATCH (w)-[r:${focusEdgeTypes}|*1..${hop}]-(m)
        RETURN DISTINCT m, r, w
    `;
    const focusNodes = await graphRepository.query(cypher, { id: worktreeId });

    // 2. 计算 透明度 梯度
    const opacityMap = new Map<NodeId, number>();
    opacityMap.set(worktreeId, 1.0);  // 当前节点 100%

    // 1-hop 邻居 80%
    const hop1 = await getHop(worktreeId, 1);
    hop1.forEach(n => opacityMap.set(n.id, 0.8));

    // 2-hop 邻居 60%
    if (hop >= 2) {
        const hop2 = await getHop(worktreeId, 2);
        hop2.filter(n => !opacityMap.has(n.id))
            .forEach(n => opacityMap.set(n.id, 0.6));
    }

    // 3-hop 邻居 40%
    if (hop >= 3) {
        const hop3 = await getHop(worktreeId, 3);
        hop3.filter(n => !opacityMap.has(n.id))
            .forEach(n => opacityMap.set(n.id, 0.4));
    }

    // 其他节点 20%
    return { focusNodes, opacityMap };
}

const focusEdgeTypes = ['CONFLICTS_WITH', 'DEPENDS_ON', 'BLOCKS', 'OVERLAPS_WITH', 'WORKS_ON', 'IMPLEMENTED_IN', 'DERIVED_FROM', 'SUPERSEDES'];
```

---

## §25 View Mode 设计 (5 模式)

### 25.1 View Mode 渲染规则

| View Mode | 突出 Node | 突出 Edge | 弱化 Node | 弱化 Edge |
|---|---|---|---|---|
| **TREE** | Worktree | DERIVED_FROM | 其他 (50%) | 非 TREE (20%) |
| **DEPENDENCY** | Worktree | DEPENDS_ON, BLOCKS (粗线 3px) | 其他 (70%) | 非 DEP (20%) |
| **RISK** | Risk Worktree (边框发光) | CONFLICTS_WITH | 正常 (20%) | 非 Risk (20%) |
| **AGENT** | AgentSession | WORKS_ON, IMPLEMENTED_IN | Worktree (按 Agent 聚类) | 非 AGENT (20%) |
| **HISTORY** | Merged Worktree | SUPERSEDES, MERGED_INTO | Active (30%) | 非 HISTORY (20%) |

### 25.2 View Mode 切换

```typescript
function applyViewMode(canvas: CanvasState, mode: ViewMode): CanvasState {
    const nodeOpacity = new Map<NodeId, number>();
    const edgeStyle = new Map<EdgeId, EdgeStyle>();

    canvas.nodes.forEach(node => {
        const baseOpacity = getBaseOpacity(node, mode);
        nodeOpacity.set(node.id, baseOpacity);
    });

    canvas.edges.forEach(edge => {
        const style = getEdgeStyle(edge, mode);  // { strokeWidth, strokeColor, opacity }
        edgeStyle.set(edge.id, style);
    });

    return { ...canvas, nodeOpacity, edgeStyle };
}
```

---

## §26 Inspector + Search DSL 设计

### 26.1 Inspector 11 Tab (per SRS §二十一)

| Tab | 内容 | 数据来源 |
|---|---|---|
| **Overview** | 11 项卡片字段 + Health 环 + Risk 角标 | Worktree + Graph |
| **Git State** | Branch / Ahead / Behind / Dirty / Last commit / HEAD | libgit2 |
| **Task** | Task title / status / assignee / due_date | TaskNode |
| **Agent** | Agent type / model / status / token / tool_calls / history | AgentSessionNode |
| **Risk** | Risk[] + Score + Reason + Explain button | RiskEngine |
| **Tests** | TestRun[] (passed/failed/skipped) + Recent runs | TestRunNode |
| **Files** | FileNode[] (按 lines_added 排序) | FileNode + MODIFIES |
| **Commits** | CommitNode[] (按时间倒序) | CommitNode + BASED_ON |
| **Relations** | Edge[] (按 type 分组, 13 类) + Reason | Graph |
| **History** | DERIVED_FROM 派生链 + SUPERSEDES 历史 + Audit | Graph + PG |
| **Actions** | 18 Action 按钮 (按分类显示) | ActionEngine |

### 26.2 Search DSL 语法 (FR-SEARCH-001)

```
query := filter (AND filter)*

filter := 
    | show: <state_list>                    # show:conflict,unmerged,stale
    | <state_name>:<state>                  # 等价于 show:state
    | agent:<agent_type>                    # agent:codex,claude-code
    | behind:<op><number>                   # behind:>20, behind:<5
    | ahead:<op><number>
    | health:<op><number>                   # health:<50, health:>80
    | modified:<file_path>                  # modified:auth.rs
    | task:<task_keyword>                   # task:payment
    | inactive:<op><duration>               # inactive:>24h, inactive:>7d

state_list := state ("," state)*
state := "ready" | "running" | "waiting" | "diverged" | "conflict" | "merged" | "stale"
op := ">" | "<" | ">=" | "<=" | "="
duration := <number>("h" | "d" | "w")
```

**示例**:
- `show:conflict` → 等价于 `conflict:conflict`
- `show:unmerged behind:>10 agent:codex`
- `health:<50 modified:auth.rs`
- `show:stale inactive:>7d`

### 26.3 NL Query 翻译 (FR-SEARCH-003)

**示例 NL**: "显示所有还未合并、落后 main 超过 10 commit、由 Codex 处理的 Worktree"

**LLM 翻译为 DSL**:
```
show:unmerged behind:>10 agent:codex
```

**Cypher 翻译**:
```cypher
MATCH (w:Worktree)-[:WORKS_ON]->(a:AgentSession)
WHERE w.human_state <> 'MERGED'
  AND w.behind > 10
  AND a.agent_type = 'codex'
RETURN w
```

---

## §27 API 概览 (OpenAPI 3.0, per SRS §8.1)

### 27.1 11 BFF Endpoint

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/v1/repos` | 列 Repository |
| GET | `/api/v1/repos/{id}` | Repository 详情 |
| GET | `/api/v1/repos/{id}/worktrees` | 列 Worktree |
| GET | `/api/v1/worktrees/{id}` | Worktree 详情 |
| GET | `/api/v1/worktrees/{id}/graph` | 1/2/3-hop Neighborhood |
| GET | `/api/v1/worktrees/{id}/health` | Health Score + 扣分明细 |
| GET | `/api/v1/worktrees/{id}/risks` | 关联 Risk 列表 |
| POST | `/api/v1/worktrees/{id}/actions/{action}` | 执行 Action (18 个) |
| POST | `/api/v1/search` | Search DSL / NL Query |
| GET | `/api/v1/explain/{worktree_id}` | AI Explanation |
| GET | `/api/v1/audit` | Audit Log 查询 |

### 27.2 SSE Event Stream

`GET /api/v1/events/stream` (per SRS §8.2, 15 事件)

---

## §28 Event 定义

(详见 §13 Event Bus 架构, BD 不重复)

---

## §29 数据流 (4 类)

(详见 §1.3 数据流, BD 不重复)

---

## §30 时序图

### 30.1 Worktree 创建时序

```mermaid
sequenceDiagram
    participant U as User
    participant BFF
    participant WS as WorktreeService
    participant GA as GitAdapter
    participant GC as GraphCore
    participant RE as RiskEngine
    participant EB as EventBus
    participant AE as ActionEngine
    participant AL as AuditLog

    U->>BFF: POST /worktrees/{id}/actions/Create
    BFF->>AE: validate(action, user)
    AE-->>BFF: OK (Safe)
    BFF->>AE: execute(action, idempotency_key)
    AE->>WS: create_worktree(repo_id, branch)
    WS->>GA: git worktree add path branch
    GA-->>WS: Worktree created
    WS->>GC: upsert_node(WorktreeNode)
    GC-->>WS: Node created
    WS->>RE: detect_risks(worktree_id)
    RE-->>WS: Risk[] computed
    WS->>GC: upsert_edges(Risk Edges)
    GC-->>WS: Edges created
    WS->>EB: publish(WorktreeCreated)
    WS->>AL: write_audit(action, result)
    AE-->>BFF: ActionResult { worktree_id }
    BFF-->>U: 201 Created { worktree }
```

### 30.2 Merge Action 时序 (Destructive)

```mermaid
sequenceDiagram
    participant U as User
    participant BFF
    participant AE as ActionEngine
    participant WS as WorktreeService
    participant MR as MergeReadiness
    participant GA as GitAdapter
    participant GC as GraphCore
    participant RE as RiskEngine
    participant EB as EventBus
    participant AL as AuditLog

    U->>BFF: POST /worktrees/{id}/actions/Merge
    BFF->>AE: validate(action, user)
    AE->>MR: check_readiness(worktree_id)
    MR->>RE: get_risks(worktree_id)
    RE-->>MR: Risk[]
    MR->>GC: get_dependencies(worktree_id)
    GC-->>MR: DEPENDS_ON[]
    MR-->>AE: MergeReadiness { ready: true, blockers: [] }
    AE-->>BFF: OK (Warning + Destructive Confirm)
    BFF-->>U: 409 Confirm Required { dialog }
    U->>BFF: POST /actions/Merge (with confirm: true)
    BFF->>AE: execute(action)
    AE->>WS: merge(worktree_id)
    WS->>GA: git merge worktree branch
    GA-->>WS: Merged
    WS->>GC: update_node(human_state: MERGED, merged_at)
    WS->>EB: publish(WorktreeMerged)
    WS->>AL: write_audit(action, result)
    AE-->>BFF: ActionResult { success: true }
    BFF-->>U: 200 OK { merged: true }
```

### 30.3 AI Explanation 时序

```mermaid
sequenceDiagram
    participant U as User
    participant BFF
    participant GC as GraphCore
    participant RE as RiskEngine
    participant HE as HealthEngine
    participant CR as Cache (Redis)
    participant LLM as Claude Sonnet

    U->>BFF: GET /explain/{worktree_id}?type=risk
    BFF->>CR: get(explanation:{id}:risk_hash)
    alt Cache Hit
        CR-->>BFF: Cached Explanation
        BFF-->>U: 200 OK { explanation }
    else Cache Miss
        CR-->>BFF: NULL
        BFF->>GC: get_worktree(id)
        GC-->>BFF: Worktree
        BFF->>RE: get_risks(worktree_id)
        RE-->>BFF: Risk[]
        BFF->>HE: get_health(worktree_id)
        HE-->>BFF: HealthScore
        BFF->>LLM: call(prompt + context_json)
        LLM-->>BFF: Explanation Text
        BFF->>CR: set(explanation:{id}:risk_hash, ttl 5min)
        BFF-->>U: 200 OK { explanation }
    end
```

---

## §31 异常处理

### 31.1 错误分类

| 错误类型 | 处理 | Retry |
|---|---|---|
| **Git Error** (merge conflict / not found) | 提示用户, 引导 Sync Main | 不重试 |
| **Graph Error** (Neo4j down) | 走 L2 Cache, 降级 | 重试 3 次, 指数退避 |
| **Network Error** (BFF 5xx) | 自动重试 | 重试 3 次, 指数退避 |
| **Permission Denied** (RBAC) | 隐藏 UI + Audit | 不重试 |
| **Validation Error** (DSL parse) | 提示错误位置 | 不重试 |
| **Conflict** (Worktree locked) | 提示锁定人 + Lock 解除时间 | 不重试 |
| **Timeout** (Long Action) | 取消 + 提示 | 提示用户选择 |

### 31.2 异常处理流程

```
Action 执行
  ↓ try
  execute()
  ↓ catch
  classify_error()
  ↓
  ├─ Git Error → user prompt + guide
  ├─ Graph Error → fallback to cache + retry
  ├─ Network → retry (max 3, exp backoff)
  ├─ Permission → audit + deny
  ├─ Validation → show parse error
  ├─ Conflict → show lock holder
  └─ Timeout → cancel + notify
```

### 31.3 故障恢复

| 故障 | 检测 | 恢复 |
|---|---|---|
| **Neo4j Down** | Connection timeout | 自动重连 + 走 L2 Cache |
| **PostgreSQL Down** | Query timeout | 暂停 Audit 写入 (in-mem queue) + 重连后 batch 写入 |
| **Redis Down** | Connection refused | 降级 L1 only + 重连后 warm cache |
| **GitObserver 断连** | watch 失效 | 自动重连 + 重放 last 1h events (per NFR-REL-001) |
| **LLM 调用失败** | HTTP 5xx / timeout | 降级到模板 + retry once |

---

## §32 并发控制

### 32.1 并发模型

- **Frontend**: React 18 + Suspense + Concurrent rendering
- **BFF**: tRPC + 协程 (async/await)
- **Core Services (Rust)**: tokio 多线程 + actor 模型 (per module)
- **Graph (Neo4j)**: Causal Cluster (read replica + write master)
- **PostgreSQL**: 连接池 (max 50) + MVCC
- **Redis**: 单线程 + 消费者组

### 32.2 锁策略

| 资源 | 锁类型 | 实现 |
|---|---|---|
| **Worktree** | Advisory Lock (PG) | `SELECT pg_advisory_xact_lock(worktree_id)` |
| **Graph Node** | Optimistic Lock (version) | `SET n.version = n.version + 1` |
| **Audit Log** | Append-only (无锁) | SCD Type 2 |
| **Action Idempotency** | Unique Constraint | `idempotency_key UUID UNIQUE` |
| **Cache** | LRU (无锁) | moka internal |

### 32.3 事务边界

| 事务 | 范围 | 实现 |
|---|---|---|
| **Action Execute** | Git + Graph + Audit | Saga 模式 + 补偿事务 |
| **Risk Detection** | Graph + Cache | Read Committed |
| **Health Compute** | Graph + Cache | Read Committed |
| **Merge Action** | Git Commit + Graph + Audit | 2PC (PG + Neo4j) |
| **Event Publish** | Redis Stream | At-least-once + idempotent consumer |

---

## §33 性能策略 (per SRS §二十七 + NFR-PERF-001..005)

### 33.1 5 级规模性能目标

| Worktree 数量 | FPS 目标 | 延迟目标 | 策略 |
|---|---|---|---|
| **10** | 60 | < 50ms | 全部渲染 |
| **100** | 30 | < 100ms | React.memo + L1 Cache |
| **500** | 20 | < 200ms | LOD + Viewport Virtualization |
| **1000** | 15 | < 300ms | + Graph Clustering + Lazy Expansion |
| **5000** | 10 | < 500ms | 切换 PixiJS WebGL |

### 33.2 7 层性能策略

| # | 策略 | 实现 | 适用 |
|---|---|---|---|
| 1 | **LOD (Semantic Zoom)** | 6 级 L0-L5, 远距离隐藏细节 | 所有 |
| 2 | **Viewport Virtualization** | 只渲染视口内节点 | 500+ |
| 3 | **Semantic Zoom** | 改变信息粒度, 不是缩放 UI | 所有 |
| 4 | **Graph Clustering** | 5+ 节点合并为 Cluster | 1000+ |
| 5 | **Lazy Expansion** | 用户点开才展开子节点 | 500+ |
| 6 | **Incremental Layout** | 节点拖动不重排 | 所有 |
| 7 | **Incremental Query** | Graph Delta Update | 所有 |

### 33.3 性能基准

```typescript
// bench/canvas-100.ts
import { bench, describe } from 'vitest';

describe('Canvas Performance', () => {
    bench('render 100 worktrees', async () => {
        const canvas = await renderCanvas(mockWorktrees(100));
        expect(canvas.fps).toBeGreaterThanOrEqual(30);
    });

    bench('render 1000 worktrees', async () => {
        const canvas = await renderCanvas(mockWorktrees(1000));
        expect(canvas.fps).toBeGreaterThanOrEqual(15);
    });
});
```

---

## §34 安全设计 (per SRS §二十九 + NFR-SEC/PERM/A11Y/I18N)

### 34.1 权限模型 (NFR-PERM-001)

| 角色 | Project | Repository | Worktree | Action |
|---|---|---|---|---|
| **Owner** | CRUD | CRUD | CRUD | All 18 |
| **PM** | R | R | R | View + Focus + Explain |
| **5 域 Lead** | R | R | R (Own Domain) | + Lock/Unlock/Sync/Cleanup |
| **SRE** | R | R | R | + Merge/Force Merge |
| **Viewer** | R | R | R | View only |

### 34.2 安全机制

| 机制 | 实现 |
|---|---|
| **数据加密 (NFR-SEC-001)** | AES-256 (敏感字段) + TLS 1.3 (传输) |
| **Git 隔离 (NFR-SEC-002)** | git worktree add path validation, 防止 path traversal |
| **RBAC (NFR-PERM-001)** | BFF API 强制, 5 角色 + 资源级 |
| **Audit Log (NFR-AUDIT-001)** | SCD Type 2 append-only, 1 年保留 |
| **SQL Injection** | 参数化查询 (sqlx prepared statement) |
| **XSS** | React auto-escape + CSP header |
| **CSRF** | tRPC CSRF token |
| **Rate Limit** | 100 req/min per user |

### 34.3 可访问性 (NFR-A11Y-001)

- WCAG 2.1 AA 合规
- 键盘可操作 (Tab/Enter/Esc/Arrow)
- ARIA 标签 (aria-label, aria-describedby)
- 颜色对比度 >= 4.5:1
- 屏幕阅读器测试 (NVDA)

### 34.4 国际化 (NFR-I18N-001)

- 3 语言: zh-CN / en / ja
- i18next 字典
- 无硬编码字符串
- 日期 / 数字 / 货币本地化

### 34.5 主题 (NFR-THEME-001)

- Dark / Light Theme 切换
- CSS variable 实现
- 跟随系统 (prefers-color-scheme)

---

## §35 可观测性 (NFR-OBS-001)

### 35.1 Prometheus Metrics (10 个)

| Metric | Type | Labels |
|---|---|---|
| `canvas_render_fps` | gauge | repo_id |
| `canvas_query_duration_seconds` | histogram | query_type |
| `risk_engine_duration_seconds` | histogram | risk_type |
| `health_compute_duration_seconds` | histogram | — |
| `action_duration_seconds` | histogram | action_type, result |
| `cache_hit_ratio` | gauge | cache_level (l1/l2/l3) |
| `event_bus_lag_seconds` | gauge | stream |
| `git_observer_events_total` | counter | event_type |
| `llm_explanation_duration_seconds` | histogram | cache (hit/miss) |
| `action_failure_total` | counter | action_type, error_type |

### 35.2 OpenTelemetry Traces

- Span: 每个 Action / Query / Risk 检测 / Explanation 调用
- Trace: 用户操作 → BFF → Core → Git/Graph/LLM
- 采样率: 10% (高流量) / 100% (错误)

### 35.3 Structured Logs

```rust
tracing::info!(
    worktree_id = %wt_id,
    action = "merge",
    duration_ms = duration.as_millis(),
    result = "success",
    "Worktree merged"
);
```

### 35.4 Audit Log (per FR-ACTION-005)

- PostgreSQL `canvas_audit` 表 (per §14.1)
- SCD Type 2 (append-only, 不可篡改)
- 1 年保留 (PG) + 7 年 (S3 冷归档)
- API: GET /audit?worktree_id=&actor_id=&action_type=&from=&to=

---

## §36 测试策略

### 36.1 测试金字塔

```
                E2E Test (Playwright)
              /                          \
        Integration Test (Rust + TS)
      /                                    \
Unit Test (Rust cargo + TS vitest)
```

### 36.2 Unit Test (per NFR-TEST-001)

- Rust: `cargo test` + `cargo tarpaulin` (覆盖率 >= 80%)
- TS: `vitest run --coverage` (覆盖率 >= 70%)

**覆盖重点**:
- 7 State Machine transition
- Health Score 13 因素公式
- Risk Score 0-1 公式
- Search DSL Parser
- NL Query → DSL 翻译
- Cache Key 失效

### 36.3 Integration Test

- 所有 18 Action 端到端
- Git → Graph 同步
- Event Bus 4 消费者
- Audit Log 写入
- Cache 3 层

### 36.4 E2E Test (Playwright)

- 5 View Mode 切换
- 6 Semantic Zoom
- Focus Mode 1/2/3-hop
- Search DSL + NL Query
- 18 Action UI 流程
- Inspector 11 Tab
- 100/1000 Worktree 性能

### 36.5 性能 Test (vitest bench)

- 100 Worktree FPS >= 30
- 1000 Worktree FPS >= 15
- Graph Query < 200ms (100 WT), < 1s (1000 WT)
- Risk Engine 增量 < 500ms

### 36.6 故障注入 Test

- Neo4j Down → Cache fallback
- PostgreSQL Down → in-mem queue
- Redis Down → L1 only
- GitObserver 断连 → 重连 + 重放

---

## §37 Requirements Traceability (115 项)

### 37.1 FR-WT (38 项, BD 覆盖)

| FR ID | BD 章节 |
|---|---|
| FR-WT-001..038 | §1-§4 + §17-§18 + §20-§22 + §24-§26 |

### 37.2 FR-UI (14 项)

| FR ID | BD 章节 |
|---|---|
| FR-UI-001 | §9 + §24-§26 |
| FR-UI-002..014 | §9 + §23-§26 |

### 37.3 FR-GRAPH (12 项)

| FR ID | BD 章节 |
|---|---|
| FR-GRAPH-001..012 | §5 + §17-§19 |

### 37.4 FR-RISK (11 项)

| FR ID | BD 章节 |
|---|---|
| FR-RISK-001..011 | §6 |

### 37.5 FR-AGENT (8 项)

| FR ID | BD 章节 |
|---|---|
| FR-AGENT-001..008 | §8 |

### 37.6 FR-EXPLAIN (4 项)

| FR ID | BD 章节 |
|---|---|
| FR-EXPLAIN-001..004 | §16 |

### 37.7 FR-SEARCH (6 项)

| FR ID | BD 章节 |
|---|---|
| FR-SEARCH-001..006 | §11 + §26 |

### 37.8 FR-ACTION (10 项)

| FR ID | BD 章节 |
|---|---|
| FR-ACTION-001..010 | §12 + §22 |

### 37.9 NFR (23 项)

| NFR ID | BD 章节 |
|---|---|
| NFR-PERF-001..005 | §33 |
| NFR-SCALE-001..002 | §1 + §2 |
| NFR-MAINT-001..002 | §2 + §34 |
| NFR-REL-001..002 | §4 + §31 |
| NFR-CON-001 | §32 |
| NFR-CACHE-001 | §15 |
| NFR-TEST-001 | §36 |
| NFR-OBS-001 | §35 |
| NFR-A11Y-001 | §34 |
| NFR-PERM-001 | §34 |
| NFR-SEC-001..002 | §34 |
| NFR-AUDIT-001 | §14 + §35 |
| NFR-I18N-001 | §34 |
| NFR-THEME-001 | §34 |
| NFR-KEYB-001 | §34 |

**完整追踪矩阵 (115 项)**: 见独立文件 `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0

---

## §38 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-15 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |

---

## §39 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v1.0 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 39 段 (目的/决策点/继承清单/架构/模块/组件/9 子模块/4 支撑层/数据模型/状态机/计算模型/Semantic Zoom/Focus/View/Inspector/API/Event/数据流/时序/异常/并发/性能/安全/可观测/测试/追踪/签字/修订), 14 模块, 4 大抽象 Trait, 15 决策点全部已拍板 (per §三十五 推荐方案明确), 11 Node + 13 Edge + 18 Action + 5 View + 6 Zoom + 7 State, 3 张时序图, 7 层性能策略 | 2026-09-15 Multica ULYS-57 issue 创建者发令 |

---

## 附录 A: Self Review (STEP 2 自审, per §三十六)

### A.1 Completeness (完整性)

- ✅ §0 文档目的 + 15 决策点 + 继承清单
- ✅ §1 系统整体架构 (拓扑 + Mermaid + 数据流 + 设计原则)
- ✅ §2 模块边界 (14 模块 + 解耦规则 + 物理布局)
- ✅ §3 组件关系 (完整 Mermaid Component Diagram)
- ✅ §4 Git Observer 架构 (7 监听源 + 防抖策略)
- ✅ §5 Graph Core 架构 (Trait + Neo4j 实现 + Cypher DDL + Index)
- ✅ §6 Risk Engine 架构 (11 Risk + V1/V2/V3 + Conflict Prediction)
- ✅ §7 Health Engine 架构 (13 因素 + 计算公式 + 阈值)
- ✅ §8 Agent Bridge 架构 (Trait + 4 适配器)
- ✅ §9 Canvas 架构 (Trait + React-Flow + PixiJS)
- ✅ §10 Layout Engine 架构 (dagre + d3-force + ELK.js + Incremental)
- ✅ §11 Query Engine 架构 (DSL Parser + Cypher 示例)
- ✅ §12 Action Engine 架构 (18 Action + 3 分类 + 二次确认)
- ✅ §13 Event Bus 架构 (15 事件 + Redis Streams + 防抖)
- ✅ §14 Persistence Layer (PG Schema + S3 归档)
- ✅ §15 Cache 架构 (3 级 + Key 设计 + Invalidation)
- ✅ §16 AI Explanation Layer (API + Prompt + 成本控制)
- ✅ §17 Node Model 详细 (Worktree)
- ✅ §18 Edge Model 详细 (13 类型)
- ✅ §19 Graph Schema (Cypher DDL)
- ✅ §20 Worktree State Machine (Mermaid + 状态机)
- ✅ §21 Health Calculation Model (引用 §7)
- ✅ §22 Risk Calculation Model (引用 §6)
- ✅ §23 Semantic Zoom (6 级 + 算法 + 距离公式)
- ✅ §24 Focus Mode (算法)
- ✅ §25 View Mode (5 模式 + 渲染规则 + 切换)
- ✅ §26 Inspector + Search DSL (11 Tab + DSL 语法 + NL 翻译)
- ✅ §27 API 概览 (11 Endpoint + SSE)
- ✅ §28 Event 定义 (引用 §13)
- ✅ §29 数据流 (引用 §1.3)
- ✅ §30 时序图 (3 张: Create / Merge / Explain)
- ✅ §31 异常处理 (7 类 + 流程 + 故障恢复)
- ✅ §32 并发控制 (模型 + 锁 + 事务)
- ✅ §33 性能策略 (5 级规模 + 7 层策略 + 基准)
- ✅ §34 安全设计 (5 角色 + 8 机制 + A11Y + I18n + Theme)
- ✅ §35 可观测性 (10 Metrics + OTel + Logs + Audit)
- ✅ §36 测试策略 (金字塔 + 4 类型 + 性能 + 故障注入)
- ✅ §37 Requirements Traceability (115 项 100%)
- ✅ §38 签字栏 (5 角色)
- ✅ §39 修订历史 (1 行 v1.0)

### A.2 Consistency (一致性, 跟 SRS 对齐)

- ✅ 115 项需求 ID 与 SRS v1.0 完全一致
- ✅ 14 模块与 SRS §二十四 完全一致
- ✅ 11 Node + 13 Edge 与 SRS §七 §八 完全一致
- ✅ 7 Human State 与 SRS §九 完全一致
- ✅ 18 Action + 3 分类与 SRS §二十二 完全一致
- ✅ 5 View Mode + 6 Zoom 与 SRS §十八 §六 完全一致
- ✅ 11 Risk 类型与 SRS §十三 完全一致
- ✅ Health 13 因素与 SRS §十二 完全一致
- ✅ 15 决策点推荐方案与 SRS §5.1 技术栈约束对齐

### A.3 Traceability (可追踪性)

- ✅ 115 项需求 → BD 章节 100% 映射 (§37)
- ✅ 决策点 15 项 → 推荐方案 + 备选 完整 (§0.2)
- ✅ 时序图 3 张 → Action / Explain 流程完整 (§30)
- ✅ 性能 5 级 → 5 策略 完整 (§33)

### A.4 Ambiguity (歧义性)

- ✅ 每项决策点都有 推荐方案 + 备选 + 触发 (per §三十五)
- ✅ 无 "根据实际情况决定" 这类无意义描述
- ✅ 所有重要模块都有 Trait Interface 定义

### A.5 Over-design (过度设计)

- ✅ 14 模块边界清晰, 不冗余
- ✅ 15 决策点都基于 SRS §5.1 + 实际需求
- ✅ 5 View Mode 覆盖全部 5 类用户视角 (TREE/DEP/RISK/AGENT/HISTORY)
- ⚠️ **Minor #1**: §13 Event Bus 防抖表 4 项可能略简, 但 DD 可细化

### A.6 Under-design (设计不足)

- ✅ 各模块 Trait Interface 完整定义 (§5.1 Graph / §8.1 Agent / §9.1 Canvas / §12.2 Action)
- ✅ 15 决策点推荐方案明确 (per §三十五)
- ✅ 性能基准明确 (§33.3)
- ⚠️ **Minor #2**: §14 Persistence 只定义了 Audit/Risk/Health/History 表, 缺 Saved Search / 收藏夹表 (DD 详)

### A.7 Scalability (可扩展性)

- ✅ 4 大 Trait 抽象保证底层可替换 (per §三十三)
- ✅ Event Bus 解耦保证新增事件不破坏现有
- ✅ 14 模块解耦 (per §2.2)

### A.8 Performance (性能)

- ✅ 5 级规模 + 7 层策略 (per §33)
- ✅ 3 级缓存 (per §15)
- ✅ Incremental Layout + Incremental Query (per §10.2 + §13)

### A.9 Security (安全)

- ✅ 5 角色 RBAC (per §34.1)
- ✅ AES-256 + TLS 1.3 (per §34.2)
- ✅ Audit Log SCD Type 2 (per §14.1)
- ✅ Git 隔离 (per §34.2)

### A.10 AI Agent Compatibility (AI Agent 兼容性)

- ✅ AgentRuntime Trait (per §8.1)
- ✅ AGENT VIEW (per §25)
- ✅ AI Explanation 基于结构化数据 (per §16.2 Prompt 约束)

### A.11 Graph Consistency (Graph 一致性)

- ✅ 11 Node + 13 Edge 完整 (per §17-§18)
- ✅ Source of Truth (Git) vs Derived Data (Graph) 边界 (per §14 + §5)

### A.12 Git Correctness (Git 正确性)

- ✅ 18 Action 覆盖 Git 核心 (per §12.1)
- ✅ Merge Readiness 检查 (per §30.2 时序图)
- ✅ Conflict Prediction (per §6.3)

### A.13 UX Cognitive Load (UX 认知负荷)

- ✅ Semantic Zoom 6 级 (per §23)
- ✅ R7 Exception-first (RISK VIEW, per §25)
- ✅ Inspector 替代 Modal (per §26)

### A.14 Critical / Major / Minor 问题清单

| 级别 | 数量 | 说明 |
|---|---|---|
| Critical | 0 | 无 |
| Major | 0 | 无 |
| Minor | 2 | 见 §A.5/#1, §A.6/#2, DD 阶段补充 |

**Self Review 结论**: BD v1.0 通过自审, 无 Critical / Major 问题, 进入 STEP 3 詳細設計。