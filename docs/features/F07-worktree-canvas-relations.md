# F07 Worktree 画布、关系与风险视图

> 状态摘要: 领域层已有较完整的类型与内存实现（13 类 Edge、11 类 Node、11 类 Risk、V1/V2/V3 检测函数、13 因素 Health 计算），但运行时没有可用的 Worktree 画布、关系或风险数据通道。`/worktree-canvas` 路由不存在；`components/worktree-canvas/` 下的 RiskView、DependencyView 等视图未被任何页面挂载；BFF `worktree_canvas` 处理器为阶段 1 桩且没有二进制引用该模块。当前 `/canvas/worktree-graph` 页的“风险”“依赖”标签基于 Work Item 派生，不是 Worktree 关系图。
>
> 代码基线: HEAD 047b9c06; 核对方法: path/symbol grep plus file reads（引用的路径在 cdd00a78..047b9c06 间无变更）; 核对日期 2026-11-16.

## 1. 目标与边界

### 1.1 目标

为 AI Agent 并行开发的 Worktree 世界提供一个低认知负荷的监督画布：以 Worktree 为核心节点，展示派生树、依赖与阻塞、冲突与重叠等关系边，并以风险视图把异常节点前置（R7 Exception-first）。画布只读消费 Git 与 Agent 的事实数据，高风险操作经 Action 流程执行（BR-10、R9）。

### 1.2 范围内 / 范围外

| 范围内 | 范围外（归属其他功能点或平行专题） |
|---|---|
| Worktree 画布主视图、视图模式切换（TREE / DEPENDENCY / RISK / AGENT / HISTORY）、缩放与焦点（Focus Mode 1/2/3-hop） | Health Score 计算与趋势（仅消费其结果） |
| 13 类 Edge 与 11 类 Node 的数据模型与渲染约定 | Search DSL 与 NL Query（仅在画布中引用其结果） |
| 依赖关系设置与解除（DEPENDS_ON / BLOCKS / SUPERSEDES），依赖对 Merge 的阻断 | AI Explanation 的 LLM 实现 |
| Risk Engine V1/V2/V3、Conflict Prediction、Risk 事件与审计 | Git 写操作执行（Sync / Rebase / Merge 的 Git 层实现） |
| Relations 与 Risk 的 Inspector 展示 | Agent 执行引擎（仅消费 AgentSession 数据） |
| 与 Run 工作区的深链边界（Worktree 为 focus / CLI target） | `domain-relation` 的 WorkItem 关系（见 §8，属于另一功能点） |

## 2. 用户与场景

| 角色 | 主要故事 | 来源 |
|---|---|---|
| 项目经理（PM） | 一眼看到 Worktree 全景与 5 状态计数；查看 Agent 与 Task 的对应；查看 DERIVED_FROM / SUPERSEDES / BLOCKS 关系 | SRS US-1、US-5、US-12、US-31 |
| 开发者（Dev） | 点击 Worktree 看 Ahead/Behind；快速对比两个 Worktree 的重叠；双击打开 Inspector；设置依赖；Hover 冲突边看 shared_files | SRS US-3、US-6、US-13、US-17、US-20 |
| SRE | 高亮 Conflict / Stale 节点；Destructive 操作二次确认；风险节点高亮（R7） | SRS US-4、US-11、US-21 |
| 5 域 Lead（真人未到位，由 Mavis 临时代签） | 风险视图只突出有风险的 Worktree；大量节点时画布不卡顿 | SRS US-14、US-27 |

## 3. 需求

优先级沿用 SRS（P0/P1/P2）。状态列仅使用 ✅ 已实现、🟡 部分实现、⬜ 未实现、🗑 已退役、❓ 未核实。“🟡”在本文中包括“代码存在但未被任何页面或服务调用”的情形，具体缺口见 §6。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-07-01 | Worktree 节点卡片渲染（名称、分支、状态、Ahead/Behind、Dirty、Health、Risk 计数等字段）（FR-WT-001、FR-WT-003） | P0 | 🟡 | 字段结构见 `crates/graph-core/src/node.rs:191`（`WorktreeNode`）；卡片组件 `frontend/src/components/worktree-canvas/WorktreeCard.tsx` 未被挂载；页面只渲染名称与 ID（`frontend/src/app/canvas/worktree-graph/page.tsx:134-149`） |
| FR-07-02 | 5 视图模式切换，切换不重算数据（FR-WT-019、FR-UI-007、BR-20） | P0 | 🟡 | 页面 5 个标签 `page.tsx:32`、`page.tsx:287-303`，切换只改 `activeView` 状态（`page.tsx:37`）；域枚举 `crates/canvas-renderer/src/view_mode.rs:17`（5 变体）；标签语义与 SRS 不一致，见 §6 |
| FR-07-03 | TREE VIEW：以 DERIVED_FROM 为骨架的派生树（FR-WT-020） | P0 | 🟡 | 页面“树”为 Project → LANE → Worktree 平铺列表（`page.tsx:110-155`），未遍历 DERIVED_FROM；Cypher 派生查询仅在设计中（SRS/BD §11.2） |
| FR-07-04 | DEPENDENCY VIEW：突出 DEPENDS_ON / BLOCKS 边（FR-WT-021） | P1 | 🟡 | `frontend/src/components/worktree-canvas/DependencyView.tsx:10-12` 过滤边；组件未被挂载；页面“依赖”标签列出的是 Work Item（`page.tsx:156-192`） |
| FR-07-05 | RISK VIEW：正常节点弱化，高风险节点强调（FR-WT-022、R7） | P1 | 🟡 | `frontend/src/components/worktree-canvas/RiskView.tsx:13`（CONFLICTS_WITH 过滤）、`:31`（无风险节点 `opacity 0.4`，SRS 要求 20%）；组件未挂载；页面“风险”标签基于 Work Item 的 blocked / p0 / p1（`page.tsx:194-220`） |
| FR-07-06 | AGENT VIEW：按 Agent 聚类，Task → Agent → Worktree 拓扑（FR-WT-023） | P1 | 🟡 | 页面“Agent”标签列出 AgentSession 并链接到 Worktree（`page.tsx:221-249`）；无 Task → Agent → Worktree 的分组拓扑 |
| FR-07-07 | HISTORY VIEW：已合并 Worktree 与 SUPERSEDES 历史（FR-WT-024） | P2 | 🟡 | 页面“历史”标签列出最近更新的 Work Item（`page.tsx:250-269`），未列出 MERGED Worktree 或 SUPERSEDES 关系 |
| FR-07-08 | 画布缩放与信息粒度（Semantic Zoom，SRS §6 六级 L0–L5，US-16） | P0 | 🟡 | 前端类型 `frontend/src/components/worktree-canvas/types.ts:130`（`ZoomLevel` L0–L5）；Rust `LodLevel` 只有 3 级（`crates/canvas-renderer/src/lod.rs:20`）；页面缩放仅为 CSS `scale`（`page.tsx:316-318`） |
| FR-07-09 | 无限画布：平移、自动布局、节点拖动、框选（FR-UI-001、FR-UI-010、FR-UI-012） | P0/P1 | ⬜ | 页面不是无限画布，是可滚动容器加 CSS 缩放（`page.tsx:276-321`）；未发现拖动或框选实现 |
| FR-07-10 | Focus Mode：当前节点突出，1/2/3-hop 邻居高亮，其余降透明度（FR-WT-038、FR-UI-005） | P0 | 🟡 | N-hop 遍历 `crates/relationship-engine/src/n_hop.rs:31`（`NHopQuery`）、`:35`（`execute`）、`:56`（`bfs_local`）；UI 组件 `frontend/src/components/worktree-canvas/FocusMode.tsx` 未挂载；无 API 路由 |
| FR-07-11 | Minimap 与 Fit to View（FR-UI-003、FR-UI-004） | P1/P0 | ⬜ | 组件 `frontend/src/components/worktree-canvas/Minimap.tsx` 未挂载；页面无 Minimap 与 Fit 操作 |
| FR-07-12 | 图层控制：显示或隐藏 Node / Edge 类型（FR-UI-006） | P1 | ⬜ | 页面无图层面板；未发现实现 |
| FR-07-13 | Edge 状态过滤与类型过滤（FR-UI-008、FR-UI-009） | P0/P1 | ⬜ | 页面无过滤控件；`RiskView.tsx` 与 `DependencyView.tsx` 内的 `filter` 为硬编码，不是用户过滤 |
| FR-07-14 | 节点折叠与展开，L3 及以上默认折叠（FR-UI-011、FR-GRAPH-010） | P0/P1 | ⬜ | 未发现折叠实现；Cluster 节点无实现（FR-GRAPH-009） |
| FR-07-15 | 13 类 Edge 类型（FR-GRAPH-003、INV-WC-04） | P0 | ✅ | `crates/graph-core/src/edge.rs:64`（`EdgeKind`，13 变体）；前端 `frontend/src/components/worktree-canvas/types.ts:90`（`EdgeKind`） |
| FR-07-16 | 11 类 Node 类型（FR-GRAPH-002、INV-WC-05） | P0 | ✅ | `crates/graph-core/src/node.rs:64`（`NodeKind`，11 变体） |
| FR-07-17 | CONFLICTS_WITH 边字段：risk_score、shared_files、shared_symbols、detected_at、reason、confidence（FR-GRAPH-006） | P0 | 🟡 | `crates/risk-engine/src/risk.rs:83`（`RiskEdgeMetadata`，含 `risk_score :85`、`shared_files :87`、`reason :93`）；Graph Edge 表 `db/migrations/2026-09-16-worktree-canvas-graph-edge.sql:12` 将 Edge 内容放在 `payload JSONB`，没有独立字段列 |
| FR-07-18 | GraphRepository 抽象（FR-GRAPH-001） | P0 | 🟡 | 内存实现 `crates/graph-core/src/memory_repo.rs:77`；Neo4j 实现为骨架，未连接真实库（`crates/graph-core/src/neo4j_repo.rs:1-20`、`:47` 读取 `NEO4J_URI` 默认值，不建立连接） |
| FR-07-19 | Graph 增量更新：通过 Event Bus 增删改节点与边（FR-GRAPH-011） | P0 | 🟡 | 15 个事件类型 `crates/event-bus/src/events.rs:34`（`EventKind`）与 `:239`（`CanvasEvent`）；未发现消费这些事件并写入 Graph 的代码 |
| FR-07-20 | Graph 查询 Cypher 子集与 Index（FR-GRAPH-007、FR-GRAPH-008） | P0 | ⬜ | Cypher DDL 与索引仅在设计文档中；`neo4j_repo.rs` 为骨架；PG 中仅有 `worktree_canvas_*` 表（见 §4.2） |
| FR-07-21 | N-hop 邻居查询 1/2/3-hop（FR-GRAPH-012） | P0 | 🟡 | `crates/relationship-engine/src/n_hop.rs:31-56`；无对外路由，`bfs_local` 仅在内存图上运行 |
| FR-07-22 | Risk 类型 11 种（FR-RISK-001） | P0 | ✅ | `crates/risk-engine/src/risk.rs:15`（`RiskType`，11 变体）；前端 `frontend/src/components/worktree-canvas/types.ts:28` |
| FR-07-23 | Risk Engine V1：Git Status、Ahead/Behind、File overlap（FR-RISK-002） | P0 | 🟡 | 纯函数 `crates/risk-engine/src/v1_git_status.rs:33`（`detect`），输入由调用方提供（`V1Input`，`:19`）；无 Git Observer 接入 |
| FR-07-24 | Risk Engine V2：Diff overlap 与 Line overlap（FR-RISK-003） | P1 | 🟡 | 纯函数 `crates/risk-engine/src/v2_diff_overlap.rs:29`（`detect`），输入 `V2Input` 由调用方提供；无三方合并预演实现（DD §13 Conflict Prediction 算法在代码中未见） |
| FR-07-25 | Risk Engine V3：Symbol overlap 与 AST（FR-RISK-004） | P2 | 🟡 | `crates/risk-engine/src/v3_symbol_overlap.rs:25`（`detect`）消费调用方给出的 `shared_symbols`（`:15-21`）；本 crate 内不做 tree-sitter 解析 |
| FR-07-26 | Conflict Prediction：Worktree 之间的前置冲突检测，输出 CONFLICTS_WITH（FR-RISK-005） | P0 | 🟡 | 结构 `crates/risk-engine/src/engine.rs:24`（`ConflictPrediction`）；内存引擎 `engine.rs:135`（`InMemoryRiskEngine`）；无端到端调用 Git 的链路 |
| FR-07-27 | Risk Score ∈ [0, 1]（FR-RISK-006、BR-8） | P0 | 🟡 | 评分在 V1 中 `min(1.0)` 截断（`v1_git_status.rs:48`、`:68`、`:85`）；公式与 BD §6.1 不同（见 §6） |
| FR-07-28 | Risk reason 字段，用于可解释展示（FR-RISK-007、R8） | P0 | 🟡 | `risk.rs:72`（`Risk.reason`）、`risk.rs:93`（`RiskEdgeMetadata.reason`）；UI 只显示 `risk: N` 计数（`RiskView.tsx:57`） |
| FR-07-29 | Risk 实时检测：Git 变化触发增量重算（FR-RISK-008） | P0 | ⬜ | `crates/git-observer` 与 `crates/risk-engine` 之间无调用关系；`star-api-rest`、`api`、`star-cli` 的依赖中均不含风险或观察者 crate（见 §4.6） |
| FR-07-30 | Risk 解决跟踪：RiskResolved 事件与状态更新（FR-RISK-009） | P1 | 🟡 | 事件变体存在（`crates/event-bus/src/events.rs:34` 的 `RiskResolved`）；未发现状态存储与更新逻辑 |
| FR-07-31 | Risk 审计日志，SCD Type 2（FR-RISK-011） | P1 | ⬜ | `db/migrations/` 中的 `worktree_canvas_*` 五张表不含风险审计表 |
| FR-07-32 | 依赖设置与解除：DEPENDS_ON 的创建与删除（FR-WT-031、FR-WT-032） | P1 | 🟡 | 高风险 Edge 校验 `crates/relationship-engine/src/engine.rs:147-150` 要求 `high_risk_metadata`；Action 变体 `RemoveDependency`（`crates/action-engine/src/action.rs:81`）存在，无 SetDependency 变体；无路由 |
| FR-07-33 | 依赖阻断 Merge：DEPENDS_ON 未满足时禁止 Merge 并给出原因（FR-WT-031、FR-WT-018） | P1 | 🟡 | 检查函数 `crates/action-engine/src/validator.rs:28-34`（`check_dependency_met`）；仅在单元测试中调用（`validator.rs:114`），Merge 执行路径未调用 |
| FR-07-34 | 标记 Superseded（FR-WT-015） | P2 | 🟡 | `ActionType::MarkSuperseded` 变体（`crates/action-engine/src/action.rs:42` 起）与 SUPERSEDES Edge（`edge.rs:64`）存在；无路由、无 UI 动作 |
| FR-07-35 | Merged Worktree 折叠到 Recently Merged，支持 24h / 7d / All 过滤（FR-WT-025） | P1 | ⬜ | 页面历史标签只列 Work Item（`page.tsx:250-269`）；未发现 MERGED 过滤 |
| FR-07-36 | Inspector 右侧固定面板，11 个标签（FR-UI-014、FR-WT-026、US-13） | P0 | 🟡 | 11 个标签组件存在（`frontend/src/components/worktree-canvas/Inspector*.tsx`，含 `InspectorRelations.tsx`、`InspectorRisk.tsx`）但未挂载；页面没有 Inspector |
| FR-07-37 | Ahead/Behind 的空间距离表达，视觉距离 `log(n+1)`（FR-WT-005） | P0 | 🟡 | 公式 `crates/layout-engine/src/visual_distance.rs:31`，常量 `:15`（`VISUAL_DISTANCE_SCALE = 30.0`）；调用方未在 UI 中使用 |
| FR-07-38 | 布局算法选择：TREE 用 dagre，关系图用 d3-force，复杂图用 ELK（D-LAYOUT-001） | P1 | 🟡 | 算法选择 `crates/layout-engine/src/engine.rs:184`（`pick_algorithm`）；前端 `frontend/src/lib/worktree-canvas/layout.ts` 未核实（❓） |

## 4. 设计

### 4.1 概念模型

- **Worktree**：Git worktree，画布核心节点。人类态（HumanState）为 7 种：Running、Waiting、Ready、Diverged、Conflict、Merged、Stale，状态转移规则写在 `crates/worktree-service/src/lifecycle.rs:3-12` 的模块注释中；转移函数的逐条实现未逐一核对（❓）。
- **Edge**：13 类语义边（`crates/graph-core/src/edge.rs:64`）。分为派生骨架（DERIVED_FROM、BASED_ON、USES_BRANCH、MERGED_INTO）、依赖与阻断（DEPENDS_ON、BLOCKS、SUPERSEDES）、风险与重叠（CONFLICTS_WITH、OVERLAPS_WITH）、以及 Agent 与 Task 关联（WORKS_ON、IMPLEMENTED_IN、MODIFIES、MODIFIES_SYMBOL）。
- **Risk**：11 类风险（`crates/risk-engine/src/risk.rs:15`），高风险 Edge 为 CONFLICTS_WITH、OVERLAPS_WITH、DEPENDS_ON、BLOCKS 四类（`crates/relationship-engine/src/edge_ops.rs:17`）。
- **Health Score**：0–100，13 个扣分因素（`crates/health-engine/src/score.rs:183` 的 `compute`，扣分点 `:190-340`）。本功能点只消费其结果。
- **视图（ViewMode）**：TREE、DEPENDENCY、RISK、AGENT、HISTORY，只切换渲染参数，不重算数据（`crates/canvas-renderer/src/view_mode.rs:17`）。

### 4.2 数据模型

| 实体 | 定义位置 | 说明 |
|---|---|---|
| WorktreeNode | `crates/graph-core/src/node.rs:191` | 字段 id、repo_id、name、branch、path、head_sha、ahead、behind 等，是 Rust 侧的节点结构 |
| Risk / RiskEdgeMetadata | `crates/risk-engine/src/risk.rs:62`、`:83` | 风险与高风险边元数据 |
| worktree_canvas_worktree | `db/migrations/2026-09-16-worktree-canvas-worktree.sql` | Worktree 表，被 Group API 读写（`crates/star-api-rest/src/group_api/worktrees.rs:337,511,843,874,1005`） |
| worktree_canvas_graph_node | `db/migrations/2026-09-16-worktree-canvas-graph-node.sql` | 图节点表 |
| worktree_canvas_graph_edge | `db/migrations/2026-09-16-worktree-canvas-graph-edge.sql:12` | 图边表，字段 from_id、to_id、repo_id、edge_kind、payload JSONB、weight、created_at；注释要求 append-only（`:33`）；启用 RLS 与 FORCE RLS，策略名 `wc_graph_edge_*` |
| worktree_canvas_graph_fingerprint | `db/migrations/2026-09-16-worktree-canvas-graph-fingerprint.sql` | 图指纹表 |
| worktree_canvas_idempotency | `db/migrations/2026-09-16-worktree-canvas-idempotency.sql` | Action 幂等键表 |
| 前端类型 | `frontend/src/components/worktree-canvas/types.ts:28`（RiskType）、`:90`（EdgeKind）、`:115`（ViewMode）、`:130`（ZoomLevel） | TypeScript 侧枚举，与 Rust 侧命名一致 |

设计文档要求的 `worktree.worktree_status_observed`、`worktree.worktree_conflict`、`worktree.worktree_action_audit` 三张表在 `db/migrations/` 中不存在（见 §6）。

### 4.3 接口

**当前存在的路由**

| 路由 | 文件 | 状态 |
|---|---|---|
| `GET/POST /api/v1/worktrees/{worktree_id}/canvases`、elements、document、events、work-items 等（8 条） | `crates/star-api-rest/src/group_api/canvas.rs:405-436`；挂载于 `crates/star-api-rest/src/group_api.rs:814` | ✅ 已挂载。这是 Worktree 范围的 Group Canvas 兼容切片（SRS §1.6），不提供 Risk、Relation、Graph 或 Health 数据 |
| `GET /api/v1/projects/{project_id}/worktrees` 等 Worktree 列表、导入路由 | `crates/star-api-rest/src/group_api/worktrees.rs`（路由表） | 🟡 存在，但仅覆盖 Worktree 列表与导入，不返回关系或风险 |
| `/v1/worktree-canvas/*`（repositories、worktrees CRUD、graph、actions、risks、health、search、nl-query） | `bff/src/worktree_canvas/controller.rs:51-73`；WebSocket `bff/src/worktree_canvas/ws.rs:32`；SSE `bff/src/worktree_canvas/sse.rs:99,103` | 🟡 路由已定义，但 handler 为阶段 1 桩（见下文），且 bff 不被任何二进制引用 |
| `/api/worktree-canvas/*`（MSW 模拟） | `frontend/src/mocks/handlers/worktree-canvas.ts:41-88` | 🟡 仅开发与测试时的模拟，前缀与 BFF 的 `/v1/worktree-canvas` 不一致；无非 mock 的前端调用方 |

**BFF 处理器的当前行为**

- `list_worktrees` 固定返回空列表（`bff/src/worktree_canvas/controller.rs:107-118`）。
- `list_risks` 固定返回空数组（`controller.rs:298-304`）。
- `get_health` 固定返回 `overall_score: 100` 及全零计数（`controller.rs:306-320`）。
- `search` 只校验 query 非空，返回空结果（`controller.rs:321-340`），注释标明“阶段 1: 0 真实 DSL 解析”。
- `nl_query` 原样回显问题并返回占位说明（`controller.rs:341-360`）。

**领域层接口**

- `RelationshipEngine`（`crates/relationship-engine/src/engine.rs:21`）：内存实现 `InMemoryRelationshipEngine`（`engine.rs:146`），`create_edge` 对高风险边要求元数据（`engine.rs:147-150`）。
- `RiskEngine`（`crates/risk-engine/src/engine.rs:39`）：内存实现 `InMemoryRiskEngine`（`engine.rs:135`）。
- `GraphRepository`：内存实现 `crates/graph-core/src/memory_repo.rs:77`；Neo4j 骨架 `crates/graph-core/src/neo4j_repo.rs:175`（未连接，见 §6）。
- `ActionEngine` 的动作枚举 `crates/action-engine/src/action.rs:42`，当前 21 个变体（设计为 18 个，见 §6）。
- `EventKind`（`crates/event-bus/src/events.rs:34`）：15 个事件，其中与本功能点直接相关的是 WorktreeChanged、WorktreeMerged、RiskDetected、RiskResolved、RelationCreated、RelationRemoved。

### 4.4 状态机与关键流程

- **Worktree 人类态**：7 种状态（`crates/worktree-service/src/lifecycle.rs:3-12`）。Ready 的条件为 health ≥ 80、无风险、ahead 为 0、测试通过；Conflict 由 ConflictDetected 进入、ConflictResolved 退出（同文件注释）。
- **风险检测流程（设计）**：Git Observer 事件 → Risk Engine V1/V2/V3 → CONFLICTS_WITH 边 → Health 重算 → 事件推送（DD §13、BD §4.3）。当前只有各检测函数的纯函数实现，事件链路未接通（FR-07-29）。
- **依赖阻断（设计）**：Merge 前检查所有 DEPENDS_ON 的目标是否已 Merged；未满足则拒绝并给出原因（FR-WT-031）。当前检查函数存在，但执行路径不调用（FR-07-33）。
- **Destructive 确认**：Merge、Delete、Cleanup 需要二次确认并写 Audit（SRS BR-13、spec INV-WC-12）。页面与 BFF 均未实现确认流程。

### 4.5 权限与安全

- 设计角色为 tenant_admin、project_admin、developer、viewer、agent（DD §44、spec §6.1），Action 的权限字符串 `worktree:*`（spec §6.2）。
- BFF 处理器调用 `state.permission.require(...)`（View 或 Edit 级别）与 `require_action(...)`（`controller.rs:111`、`:172`、`:185`、`:200`、`:301`、`:309`、`:325`、`:345`），删除与执行动作使用 `require_action`（`controller.rs:212`、`:239`）。Action 的 RBAC 规则定义在 `bff/src/worktree_canvas/permission.rs`，未逐条核对（❓）。这些处理器的业务分支仍为桩。
- 数据层：`worktree_canvas_graph_edge` 启用 RLS 与 FORCE RLS，策略按 tenant 与 workspace 隔离（`db/migrations/2026-09-16-worktree-canvas-graph-edge.sql`）。其他表的 RLS 策略见各迁移文件，未逐一核对（❓）。
- 不得由 LLM 修改 Source of Truth（INV-WC-10）：当前 NL 翻译为模板与关键词匹配（`crates/query-engine/src/nl_translator.rs:1-4`），不调用 LLM，因此不存在越权写入路径。

### 4.6 依赖关系

- 领域 crate：`graph-core`、`git-adapter`、`git-observer`、`worktree-service`、`risk-engine`、`relationship-engine`、`health-engine`、`agent-bridge`、`canvas-renderer`、`layout-engine`、`query-engine`、`action-engine`、`event-bus`，均为根 `Cargo.toml` 的 workspace 成员（`Cargo.toml:27`、`:151` 等）。
- 运行时依赖：`crates/star-api-rest`、`crates/api` 的 Cargo.toml 不依赖上述风险或画布 crate；`crates/star-cli` 仅依赖 `git-adapter`（`crates/star-cli/Cargo.toml:33`）。`bff` 位于根 workspace 之外（根 `Cargo.toml` 无 `bff` 成员），在 `bff/Cargo.toml` 中以 path 依赖引用 `graph-core`、`git-adapter`、`git-observer`。
- WASM 包装：`crates/relationship-engine-wasm/src/lib.rs:56`（`edge_kind_count`）、`crates/query-engine-wasm`，仅被 `frontend/src/app/wasm-demo/page.tsx` 与 `frontend/src/lib/wasm/` 使用。
- 与 WorkItem 关系（`domain-relation`）：独立功能点，见 §8。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| Worktree 图页面（/canvas/worktree-graph） | `frontend/src/app/canvas/worktree-graph/page.tsx` | 🟡 | 5 个标签 `page.tsx:32`；树与历史基于 store，风险与依赖基于 Work Item（`page.tsx:156-269`）；宿主未接入、未授权、空数据三态（`page.tsx:116-117`、`:59-60`） |
| 旧路由 /worktree-canvas | `frontend/src/app/`（无 `worktree-canvas` 目录） | ⬜ | `frontend/src/app/canvas/worktree-graph/not-found.tsx:10` 指向 `/worktree-canvas`，该路由不存在 |
| RiskView 组件 | `frontend/src/components/worktree-canvas/RiskView.tsx` | 🟡 | 按 `riskCount` 排序（`:10-12`）；未被挂载（全仓无 import） |
| DependencyView 组件 | `frontend/src/components/worktree-canvas/DependencyView.tsx` | 🟡 | 过滤 DEPENDS_ON、BLOCKS、SUPERSEDES（`:10-13`）；未被挂载 |
| TreeView、AgentView、HistoryView、FocusMode、Minimap、ZoomL0–L5、WorktreeCard、Inspector 系列、AIDialog、ConfirmationModal、SearchBar、StatusBadge、RiskBadge、HealthBadge | `frontend/src/components/worktree-canvas/` | 🟡 | 组件存在，全仓无页面引用 |
| 画布状态 store | `frontend/src/stores/worktreeCanvasStore.ts:79`（`create`）、`:119`（默认 `viewMode: "TREE"`）、`:121`（`zoomLevel: "L1"`） | 🟡 | 仅组件与测试引用；`frontend/src/lib/hooks/use-annotations.ts` 出现该名称，是否为 import 未核实（❓） |
| 主 store 中的 worktrees | `frontend/src/lib/store.ts:411` | 🟡 | 默认空数组；数据来源未在本功能点内核实 |
| 前端 API 客户端 | `frontend/src/lib/group/worktreeGroupApi.ts` | 🟡 | 已覆盖 Group Canvas、Worktree 列表与导入；无 risk、relation、graph、health 端点 |
| MSW 模拟 | `frontend/src/mocks/handlers/worktree-canvas.ts:41-88` | 🟡 | 阶段 1 桩，固定数据；`risks` 返回空数组（`:72`），`health` 固定值（`:73-75`） |
| BFF worktree_canvas 模块 | `bff/src/worktree_canvas/controller.rs`、`mod.rs`、`sse.rs`、`ws.rs` | 🟡 | feature 门控 `bff/src/lib.rs:29-30`；handler 为桩（§4.3）；无二进制引用 |
| Group Canvas 路由 | `crates/star-api-rest/src/group_api/canvas.rs:405-436` | 🟡 | 已挂载；Canvas 元素与事件，不含风险或关系 |
| worktree-canvas facade | `crates/worktree-canvas/src/lib.rs` | 🟡 | 重新导出 14 个子 crate（`lib.rs` 的 `pub use`）；仅有 1 个冒烟测试；无消费者 |
| graph-core | `crates/graph-core/src/` | 🟡 | `EdgeKind` 13 变体（`edge.rs:64`）、`NodeKind` 11 变体（`node.rs:64`）、内存实现（`memory_repo.rs:77`）；Neo4j 骨架（`neo4j_repo.rs:1-20`） |
| risk-engine | `crates/risk-engine/src/` | 🟡 | `RiskType` 11 变体（`risk.rs:15`）；V1/V2/V3 纯函数（`v1_git_status.rs:33`、`v2_diff_overlap.rs:29`、`v3_symbol_overlap.rs:25`）；内存引擎（`engine.rs:135`） |
| relationship-engine | `crates/relationship-engine/src/` | 🟡 | `HighRiskKind` 4 变体（`edge_ops.rs:17`）；内存引擎（`engine.rs:146`）；N-hop（`n_hop.rs:31`） |
| health-engine | `crates/health-engine/src/score.rs` | 🟡 | 13 因素扣分（`score.rs:190-340`）；本功能点只消费结果 |
| action-engine | `crates/action-engine/src/action.rs:42`、`validator.rs:28-34` | 🟡 | 21 个动作变体；依赖检查函数仅单元测试调用（`validator.rs:114`） |
| canvas-renderer | `crates/canvas-renderer/src/view_mode.rs:17`、`lod.rs:20`、`renderer.rs:121` | 🟡 | ViewMode 5 个；LodLevel 3 个；`StandardCanvasRenderer` 为 Rust 侧状态层 |
| layout-engine | `crates/layout-engine/src/visual_distance.rs:15,31`、`engine.rs:184` | 🟡 | 视觉距离公式与布局选择函数存在；UI 未调用 |
| query-engine | `crates/query-engine/src/dsl_parser.rs:146`、`nl_translator.rs:30` | 🟡 | DSL 解析与模板式 NL 翻译；无 LLM 调用 |
| event-bus | `crates/event-bus/src/events.rs:34,239` | 🟡 | 事件枚举定义；无消费者写入 Graph 或风险状态 |
| git-adapter | `crates/git-adapter/src/libgit2_provider.rs:41`、`cli_provider.rs:99` | 🟡 | 两个 Provider 实现；风险或画布 crate 不调用 |
| git-observer | `crates/git-observer/src/` | 🟡 | 16 个测试，无运行时消费者（§4.6） |
| worktree-service | `crates/worktree-service/src/pg_worktree_service.rs:145`、`service_impl.rs:240`、`lifecycle.rs` | 🟡 | PG 与内存两种实现；模块内 TODO 标记 18 处 |
| 数据库迁移 | `db/migrations/2026-09-16-worktree-canvas-*.sql`（5 个） | 🟡 | 表与 RLS 存在；风险审计表与设计中的三张 worktree.* 表缺失 |
| WASM 演示 | `crates/relationship-engine-wasm/src/lib.rs`、`frontend/src/app/wasm-demo/page.tsx` | 🟡 | 仅演示页面使用 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| 页面风险与依赖标签基于 Work Item，不是 Worktree 关系 | 用户看到的“风险”与 Worktree 冲突无关，容易误判 | 将 RiskView、DependencyView 挂入页面，并让 `page.tsx` 的风险与依赖标签读取 Worktree 图数据；或在页面上标注数据来源 |
| 无服务端路由提供 risks、relations、graph、health | 前端无法获取 CONFLICTS_WITH、DEPENDS_ON 等数据（FR-07-17、FR-07-26、FR-07-32） | 在 Group API 或 BFF 中新增只读路由，调用 relationship-engine 与 risk-engine；先实现 GET 查询，再实现 Action |
| BFF 处理器为阶段 1 桩，且 bff 无二进制引用 | `/v1/worktree-canvas/*` 不可达；返回值为空或固定值，存在误导 | 决定 bff 的归属（并入 star-api-rest 或独立部署），接入真实服务后移除固定返回 |
| 领域 crate 只有内存实现，无运行时消费者 | 数据不持久，重启丢失；Risk 与 Relation 无法跨请求共享 | 为 relationship-engine 与 risk-engine 实现 PG 适配器（基于 `worktree_canvas_graph_edge`、`worktree_canvas_graph_node`）；在 star-api-rest 中引入依赖 |
| Neo4j 实现为骨架（`neo4j_repo.rs:1-20`） | 设计选型 D-GRAPH-001 未落地 | 维持 PG 作为 MVP 存储（与 `worktree_canvas_graph_*` 表一致），或在 ADR 中重新裁决 Graph DB 选型后再实现 |
| 无 Git Observer 到 Risk Engine 的链路（FR-07-29） | 风险不会随 Git 变化更新 | 实现 git-observer 事件到 event-bus 的桥接，并由 risk-engine 订阅 |
| V1 评分公式与 BD §6.1 不一致（`v1_git_status.rs:48,68,85`） | 风险数值无法与设计对齐 | 以 BD §6.1 为设计依据，核对后修改公式，或在 BD 中记录采用的新公式 |
| V2 无三方合并预演；V3 不做 AST 解析（`v3_symbol_overlap.rs:25`） | 只能检测调用方给出的重叠，无真实冲突预测 | V2 实现 diff3 预演；V3 在 star-treesitter 或等价组件中解析后，把符号交给 risk-engine |
| Action 枚举 21 项与设计 18 项不一致（`action.rs:42`） | 存在 AiChat、AiCompletion、AiComposerEdit 等不在设计中的项；缺 Create、Lock、Unlock、SetDependency | 对齐枚举与 SRS §12 的 18 项动作，或更新设计 |
| 依赖阻断只在单元测试中调用（`validator.rs:114`） | DEPENDS_ON 不会阻止 Merge | 在 Merge 执行路径中调用 `check_dependency_met` 并返回原因 |
| 缩放层级不一致：Rust 3 级（`lod.rs:20`）、TS 6 级（`types.ts:130`） | 远近信息粒度无法统一 | 以 SRS §6 的 L0–L5 为准，扩展 LodLevel 并接入 UI |
| 无限画布、Minimap、Focus、折叠、框选、图层控制均未挂载 | 画布交互只有缩放按钮（`page.tsx:306-311`） | 按 P0 优先级逐项接入已有组件，或重写为 React-Flow 等渲染引擎 |
| 设计表（worktree.*）与实际表（worktree_canvas_*）不一致 | 文档与迁移脚本不一致，数据模型难以对齐 | 以已有迁移为准更新 BD §5.2；补充 RLS 与索引 |
| 风险审计表缺失（FR-07-31） | 无法追溯风险的检测与解决 | 新增 T 类审计表（append-only），依 W/T/M 分类规则归类 |
| E2E 全部为 `test.skip`（`frontend/e2e/worktree-canvas.spec.ts:26-56`） | 视图、Action、SSE 无端到端覆盖 | 在页面挂载后替换为真实断言 |
| 旧路由 `/worktree-canvas` 在 not-found 页被引用 | 用户访问后得到 404 | 改为指向 `/canvas/worktree-graph` |

## 7. 验收与测试

### 7.1 现有测试

| 测试 | 位置 | 覆盖内容 | 状态 |
|---|---|---|---|
| 前端组件单测 | `frontend/src/components/worktree-canvas/__tests__/`（ConfirmationModal、HealthBadge、RiskBadge、SearchBar、StatusBadge、TreeView、types、worktreeCanvasStore） | 未挂载组件的渲染与 store 行为 | 🟡：有测试，但组件未挂载；RiskView 与 DependencyView 无测试 |
| 页面相关测试 | `frontend/src/lib/group/__tests__/`（groupProjection、worktreeGroupApi） | Group 投影与 API 客户端 | 🟡：不覆盖风险或依赖数据 |
| E2E | `frontend/e2e/worktree-canvas.spec.ts:26-56` | wc-01 至 wc-08（视图、缩放、Inspector、搜索、NL、确认、SSE） | 🟡：8 项全部 `test.skip`，未运行 |
| Rust 单元测试（#[test] 与 #[tokio::test] 标注数，src 与 tests 合计） | graph-core 72、worktree-service 72、action-engine 58、canvas-renderer 25、query-engine 20、agent-bridge 30、git-observer 16、git-adapter 24、event-bus 13、risk-engine 12、health-engine 7、layout-engine 10、relationship-engine 4、worktree-canvas 1 | 各子 crate 的纯函数与内存实现 | 🟡：覆盖领域逻辑，不覆盖路由、持久化或 UI |
| BFF 单测 | `bff/src/worktree_canvas/controller.rs` 测试模块（`bff_11_rest_endpoints_rbac_enforced` 等） | 桩 handler 的状态码与 RBAC | 🟡：仅验证桩行为，不是真实服务 |
| PG 集成测试 | `crates/star-api-rest/tests/canvas_event_stream_pg.rs`、`canvas_device_auth_pg.rs` | Group Canvas 事件流与设备授权 | 🟡：不涉及风险或关系 |

### 7.2 已验证与未验证

- 已验证（代码结构层）：13 类 Edge、11 类 Node、11 类 Risk、5 类 ViewMode、15 类事件、13 个 Health 扣分因素在代码中存在（见 §3 证据列）。
- 未验证（运行时层）：页面挂载后的渲染、视图切换与风险展示；BFF 与 star-api-rest 的风险或关系路由；领域 crate 的持久化；Git Observer 到 Risk Engine 的链路；Merge 的依赖阻断。
- 测试设计书（TEST-DESIGN-WORKTREE-CANVAS-001）中的通过数字（如 52/52、10/10、8/8）为设计基线，未重跑；其中列出的测试名（如 `merge_destructive_idempotency_audit_e2e`）未逐一检索（❓）。

### 7.3 验收标准（目标）

以下验收项在 SRS §12 与测试设计书中定义，当前均未达成：

- AC-F-7：5 个视图切换不重算数据，网络上无全量重新加载。
- AC-F-8：6 级缩放信息粒度正确。
- AC-F-9：Search DSL 过滤结果正确（归属搜索功能点）。
- AC-F-11：Focus Mode 1/2/3-hop 透明度分别为 100%、80%、20%。
- AC-F-13：Destructive 操作二次确认并写 Audit。
- AC-P-2：1000 个 Worktree 下 FPS ≥ 15（LOD 与虚拟化）。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-WORKTREE-CANVAS-001.md`（读取 §1–§4.4 与 §1.6） | 用户故事、FR-WT / FR-UI / FR-GRAPH / FR-RISK 需求编号与优先级；视图与缩放定义 | 文件头部版本号自相矛盾（v1.10、v1.11 并存）；FR-UI-008、FR-UI-009 为状态过滤与 Edge 过滤，而 `RiskView.tsx:26`、`DependencyView.tsx:29` 把视图标注为 FR-UI-009、FR-UI-008，与 SRS 视图编号（FR-WT-021、FR-WT-022）不一致 | 需求编号以 SRS 为准；视图需求按 FR-WT-019 至 FR-WT-024 对应；组件注释中的编号视为错误 |
| `docs/design/BD-WORKTREE-CANVAS-001.md`（读取 §0–§12） | 系统拓扑、4 个 Trait、Risk 公式（§6.1）、Health 因素（§7.1）、视觉距离、Cypher DDL（§5.3）、Action 分类（§12.1） | Neo4j 为推荐选型，代码为骨架；V1 公式与代码不同；视觉距离常数 BD 与 spec 为 60，代码为 30（`visual_distance.rs:15`）；§5.2 的 `worktree.*` 表与代码不一致；前端路径写为 `(worktree-canvas)`，代码为 `canvas/worktree-graph` | 设计意图保留并标注 ⬜ 或 🟡；公式、常数与表名以代码为现状，差异列入 §6；Neo4j 选型保留为设计，实现以 PG 为现状 |
| `docs/design/DD-WORKTREE-CANVAS-001.md`（读取 §0–§10） | Edge 字段（§8）、Cypher 查询（§9）、Action 列表（§12）、Crate 与前端文件布局（§1.1） | §1.1 的 crate 布局与代码基本一致；前端布局路径与实际不一致；Action 列表与代码枚举不一致（§6） | 字段与布局以代码为准，Action 列表以代码和 SRS §12 的 18 项为对照，差异列入 §6 |
| `docs/specs/worktree-canvas-spec.md`（读取 §0–§9 前半） | 不变量 INV-WC-01 至 12、Action 分类（§4.4）、权限字符串（§6.2）、Event payload（§4.5） | §4.4 中 A-07 Merge 为 Destructive，与 FR-ACTION-002 的 Merge 分类存在已标注的冲突（spec 自注）；WC-* 错误码在代码中未检索（❓） | 以 spec 的 Merge 为 Destructive 为准；错误码列为 ❓ |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md`（读取 §0–§4.2） | 测试矩阵、IT 与 E2E 列表、验收 AC 编号 | 通过数字为设计基线；E2E 实际全部 `test.skip`（`frontend/e2e/worktree-canvas.spec.ts`） | 测试设计保留为目标；通过数字不计为现状 |
| `docs/specs/domain-relation-spec.md`（读取全文） | WorkItem 关系（blocks、relates_to、duplicates、clones）、循环依赖检测、Gantt | 该 spec 的 Relation 聚合根以 WorkItem 为端点，与本功能点的 Worktree 关系不同；代码在 `crates/domain-relation/src/lib.rs`（未核实内容） | 不吸收进 F07；仅记录名称冲突。Worktree 关系与 WorkItem 关系归属不同功能点 |
| `frontend/src/app/canvas/worktree-graph/page.tsx` 文件头注释（代码内文档） | 页面的 5 视图与三态（未接入、未授权、空数据）定义 | 注释称视图由 store 驱动，风险与依赖标签实际由 Work Item 派生（`page.tsx:194-196`） | 以代码为准；注释描述的标签语义列入 §6 |
| `crates/worktree-canvas/src/lib.rs` 文件头注释 | 14 个子 crate 的职责与不变量映射 | 注释中的 `visualDistance = log(n+1)*30` 与 BD 的 60 不同，与代码一致 | 以代码为准 |
| `docs/arch-live/05-worktree-canvas.html` | 未读正文；检索中出现 | 不影响本文判定 | 未用，仅作检索命中 |
| `docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md` | 未读正文；被 spec 与 DD 引用 | 实施计划的任务分解未逐项核对 | 未用；任务状态以代码为准 |
| `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` | 未读正文；被多份文档引用 | 追踪矩阵与本文需求编号对应未逐条核对 | 未用；编号以 SRS 为准 |
| `docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md` 与 `BD/DD-W16-PAYLOAD-SECURITY-001.md` | 未读正文；在测试设计中作为补充引用 | 与协作载荷安全相关，不属于 F07 范围 | 未吸收 |
| `docs/architecture/2026-09-03-treesitter-worktree-graph/*` 与对应 wiki 页 | 未读正文；检索中出现 | 涉及 tree-sitter 与 Worktree 图，可能影响 V3 设计 | 未用；V3 以代码为准，列入 §6 |
| 代码：`bff/src/worktree_canvas/`、`crates/*`（§4–§5 引用的文件） | 实现现状与证据 | 与设计多处不一致，已在 §6 列出 | 代码为现状裁决依据 |
