# F07 Worktree 画布、关系与风险视图

> 状态摘要: `/canvas/worktree-graph` 页面已存在，但只是基于全局 store 与宿主 Group API 可用性的五视图占位，不是 SRS 描述的无限画布；图模型（11 类 Node、13 类 Edge）、内存图仓储、风险 V1/V2 检测、N-hop 与 DSL 解析作为 Rust crate 存在，BFF REST/SSE/WS、Action 执行、Neo4j 后端、实时事件链路、画布交互、Inspector 与搜索 UI 尚未接入。
>
> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

面向 Project 级 Repository，把 AI Agent 并行产生的多个 Worktree 呈现为一张可检视的图：每个 Worktree 的 Git 状态、人类可读状态、风险与依赖关系一眼可辨，异常节点优先突出，并支持以某个 Worktree 为中心查看 1/2/3-hop 邻域。画布定位为 Project 级 Worktree Overview Graph，不拥有 Run 内任务或 CLI 状态机；Run 工作区内的 Group Canvas 是同级应用，两者通过类型化引用深链（SRS-WORKTREE-CANVAS-001 §1.1、§1.6）。

WorkItem 之间的显式关系（阻塞、重复、引用等）由 `domain-relation` 承载，与 Worktree 间的 Edge 图是两个不同的关系域（domain-relation-spec §1）。本功能同时覆盖这两者的展示入口。

### 1.2 范围内 / 范围外

范围内:

- 画布页面与路由、五视图（TREE / DEPENDENCY / RISK / AGENT / HISTORY）切换、缩放、Focus N-hop、搜索与自然语言入口。
- 图数据模型：11 类 Node、13 类 Edge、Worktree 与 AgentSession 节点字段、CONFLICTS_WITH 边字段。
- 风险引擎 V1/V2/V3、冲突预测、风险解决与广播、健康度推荐。
- 图持久化 DDL（`worktree_canvas_graph_node` / `_edge` / `_fingerprint`）与 RLS。
- BFF `/v1/worktree-canvas/*` 路由与 MSW mock 路径。
- WorkItem 关系领域（`domain-relation`）与 `/relation` 只读列表页。

范围外:

- Git 底层执行（libgit2 / git CLI）与 Git 状态采集的 Adapter 细节（SRS-WORKTREE-CANVAS-001 §1.4）。
- AI Agent 执行引擎与 Agent 运行时（同上）。
- CRDT 多人协同算法（同上）。
- L5 Symbol 级 AST 分析（MVP 外，SRS §1.4）。
- Notebook、Dashboard 与历史时间轴等其他画布（同上）。
- Run-owned schema 与历史归属迁移（SRS §1.6，Phase 2E 另列）。
- PR Review 评论系统（属 ARG 平行专题）。

## 2. 用户与场景

| 角色 | 主要场景 | 对应需求 |
|---|---|---|
| 项目经理（PM） | 打开即看到 Worktree 全景与状态计数；查看哪个 Agent 处理哪个任务；用自然语言查询未合并且落后主干的 Worktree | US-1、US-5、US-9 |
| 开发者（Dev） | 过滤 `show:conflict`、`behind:>20`；点击节点进入 1-hop Focus；双击打开右侧 Inspector；对比两个 Worktree 的文件与 Symbol 重叠 | US-3、US-7、US-10、US-13、US-23 |
| SRE | 高亮 Conflict / Stale 节点并一键 Sync Main 或 Merge；按 Health Score 扣分明细排查；Lock / Unlock Worktree；Cleanup 长期无活动的 Worktree | US-4、US-8、US-19、US-24 |
| 5 域 Lead（真人未到位，由 Mavis 代签） | 只看负责域的 Worktree；RISK VIEW 只突出有风险的节点 | US-2、US-14 |

## 3. 需求

源需求编号来自 SRS-WORKTREE-CANVAS-001 §4，本表以 FR-07-NN 重新编号，并在需求列标注源编号。状态依据代码基线 637ed19d 的实际实现判定。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-07-01 | 画布页面入口与路由（源 SRS §1.3、§1.6）。SRS 写作 `/worktree-canvas`，实现路由为 `/canvas/worktree-graph`，由 Canvas「关系图」页签进入 | P0 | 🟡 部分实现 | `frontend/src/app/canvas/worktree-graph/page.tsx:36`；`frontend/src/app/canvas/[id]/page.tsx:54`（子页签「关系图」）、`:225`（跳转链接）。未找到 `app/worktree-canvas` 目录或对应 redirect（`git grep "worktree-canvas" -- frontend`） |
| FR-07-02 | 无限画布：平移、缩放、自动布局、节点手动拖动、多选与框选（源 FR-UI-001、FR-UI-010、FR-UI-012） | P0 | ⬜ 未实现 | `frontend/src/app/canvas/worktree-graph/page.tsx:315-321` 为普通滚动容器加 CSS scale，无拖动、框选或多选事件处理；页面未引用 `canvas-renderer` 产物 |
| FR-07-03 | 缩放控制与 Fit to View / 自动居中（源 FR-UI-004） | P0 | 🟡 部分实现 | 缩放按钮与 0.1–3 范围：`page.tsx:306-311`；scale 应用：`page.tsx:316-318`。无 Fit、无 Auto Center、无快捷键 |
| FR-07-04 | 五视图切换且不重算数据（源 FR-WT-019、FR-UI-007） | P0 | 🟡 部分实现 | 五个页签：`page.tsx:32`、`:287-303`；切换仅更新本地 state：`page.tsx:37`、`:292`。页签标签为中文「树 / 依赖 / 风险 / Agent / 历史」 |
| FR-07-05 | TREE 视图以 Main → Worktree 派生关系（DERIVED_FROM）为骨架（源 FR-WT-020） | P0 | 🟡 部分实现 | 树视图渲染 Project → LANE → Worktree 列表，截取前 6 个：`page.tsx:110-155`，`:134`。未读取 DERIVED_FROM；边类型存在于 `crates/graph-core/src/edge.rs:164`（`DerivedFromEdge`） |
| FR-07-06 | DEPENDENCY 视图以粗线突出 DEPENDS_ON / BLOCKS 边，TREE 边降透明度（源 FR-WT-021） | P1 | 🟡 部分实现 | 依赖页签以 Worktree 为 hub 分组列出任务：`page.tsx:156-193`，无边绘制、无透明度。`HighRiskKind` 已包含 `DependsOn`、`Blocks`（`crates/relationship-engine/src/edge_ops.rs:17-30`） |
| FR-07-07 | RISK 视图：正常节点透明度 20%，高风险节点 100% 且边框发光（源 FR-WT-022） | P1 | 🟡 部分实现 | 风险页签列出 status 为 blocked 或 priority 为 p0/p1 的任务：`page.tsx:196`、`:200-220`。风险对象是任务而非 Worktree，无透明度与发光样式 |
| FR-07-08 | AGENT 视图按 Agent 聚类，形成 Task → Agent → Worktree 树（源 FR-WT-023） | P1 | 🟡 部分实现 | 页签为扁平 Agent 会话列表，每条链接到其 Worktree：`page.tsx:221-249`，`:229`、`:238-242`。无 Task 层、无树形结构 |
| FR-07-09 | HISTORY 视图显示已合并 Worktree 与 SUPERSEDES 历史（源 FR-WT-024） | P2 | 🟡 部分实现 | 页签列出按 `updated_at` 倒序的前 10 个任务：`page.tsx:250-270`，排序 `:252`。无 merged 过滤、无 SUPERSEDES 展示 |
| FR-07-10 | Focus Mode 1/2/3-hop 邻域：中心节点突出，邻居高亮（源 FR-WT-038） | P0 | 🟡 部分实现 | hop 枚举与解析：`crates/canvas-renderer/src/focus.rs:21-56`；页面不调用。Store 中 `focusNodeId` / `focusHop` 仅被未挂载组件引用：`frontend/src/stores/worktreeCanvasStore.ts` |
| FR-07-11 | Focus 透明度：当前 100%、1-hop 80%、其他 20%（源 FR-UI-005） | P0 | ⬜ 未实现 | `focus.rs:1-7` 声明透明度为 hop1 0.8、hop2 0.6、hop3 0.4，与 SRS 数值不同；页面无 Focus UI |
| FR-07-12 | 按 Human State、Edge 类型过滤与图层控制（源 FR-UI-006、FR-UI-008、FR-UI-009） | P0 | ⬜ 未实现 | `page.tsx` 无过滤控件；`frontend/src/components/worktree-canvas/` 中相关组件未被页面导入 |
| FR-07-13 | 搜索 DSL 与自然语言定位，命中后居中高亮（源 FR-UI-002、FR-SEARCH-001 至 003） | P0 | 🟡 部分实现 | DSL 6 个关键字：`crates/query-engine/src/keywords.rs:39-43`；NL 模板匹配：`crates/query-engine/src/nl_translator.rs:30-35`；执行为空结果 stub：`crates/query-engine/src/engine.rs:70-79`；BFF `search` / `nl-query` 为占位：`bff/src/worktree_canvas/controller.rs:321-352`。页面无搜索框 |
| FR-07-14 | Minimap 显示视口矩形与节点缩略并可点击跳转（源 FR-UI-003） | P1 | ⬜ 未实现 | `frontend/src/components/worktree-canvas/Minimap.tsx` 存在，未被任何页面导入 |
| FR-07-15 | 右侧固定 Inspector，11 个页签，禁止大量 Modal（源 FR-UI-014、FR-WT-026） | P0 | ⬜ 未实现 | 11 值的 `InspectorTab` 类型：`frontend/src/components/worktree-canvas/types.ts`；Inspector 组件未挂载；页面无右侧面板 |
| FR-07-16 | 键盘快捷键全操作（源 FR-UI-013） | P1 | ⬜ 未实现 | `page.tsx` 全文无键盘事件处理 |
| FR-07-17 | 节点折叠 / 展开，远距离 Cluster 聚合（源 FR-UI-011、FR-GRAPH-009、FR-GRAPH-010） | P0 / P1 | ⬜ 未实现 | `crates/canvas-renderer/src/viewport.rs:3-8` 注明 Cluster 为 P2 stub，MVP 总是展开 |
| FR-07-18 | 危险动作二次确认（源 FR-ACTION-003） | P0 | 🟡 部分实现 | BFF 对 Destructive 动作要求 `confirm=true`：`bff/src/worktree_canvas/controller.rs:249-261`；前端 `ConfirmationModal` 为未挂载组件 |
| FR-07-19 | 设置依赖 Set Dependency（源 FR-WT-031） | P1 | ⬜ 未实现 | `crates/action-engine/src/action.rs:42` 起的 `ActionType` 中无 `SetDependency` 变体，仅注释提及（`action.rs:18`） |
| FR-07-20 | 移除依赖 Remove Dependency（源 FR-WT-032） | P1 | 🟡 部分实现 | `ActionType::RemoveDependency` 存在（`action.rs:82`、`:119`）；执行为 stub，不删除边（`crates/action-engine/src/engine.rs:230-246`）。`RelationshipEngine` trait 声明 `delete_edge`（`crates/relationship-engine/src/engine.rs:21` 起） |
| FR-07-21 | GraphRepository Trait，含 upsert / delete / query / traverse 等核心方法（源 FR-GRAPH-001） | P0 | 🟡 部分实现 | Trait 定义 11 个 async 方法：`crates/graph-core/src/repository.rs:99-160`，包括 `upsert_node`、`upsert_edge`、`delete_node`、`delete_edge`、`n_hop`、`shortest_path`；无同名 `query` / `traverse` |
| FR-07-22 | 11 类 Node（源 FR-GRAPH-002） | P0 | ✅ 已实现 | `crates/graph-core/src/node.rs:64`（`NodeKind`，11 个变体） |
| FR-07-23 | 13 类 Edge（源 FR-GRAPH-003） | P0 | ✅ 已实现 | `crates/graph-core/src/edge.rs:64`（`EdgeKind`，13 个变体）；`validate_kind` 位于 `crates/relationship-engine/src/edge_ops.rs:95` |
| FR-07-24 | Worktree Node 字段（源 FR-GRAPH-004） | P0 | 🟡 部分实现 | `WorktreeNode` 含 `id`、`repo_id`、`name`、`branch`、`path`、`head_sha`、`ahead`、`behind`、`dirty`、`last_activity`、`created_at`、`merged_at`（`crates/graph-core/src/node.rs:191-214`）。缺 `human_state`、`health_score`、`agent_id`、`task_id`、`risk_count`、`test_state`、`locked`、`archived`；human state 为独立枚举（`crates/graph-core/src/state.rs:29`） |
| FR-07-25 | AgentSession Node 字段（源 FR-GRAPH-005） | P0 | 🟡 部分实现 | `AgentSessionNode` 含 `id`、`repo_id`、`agent_type`、`model`、`status`、`task_id`、`worktree_id`、`started_at`、`last_activity`、`token_usage`、`tool_calls`（`crates/graph-core/src/node.rs:270-282`）。缺 `result_state` |
| FR-07-26 | CONFLICTS_WITH 边字段（源 FR-GRAPH-006） | P0 | ✅ 已实现 | `ConflictsWithEdge` 含 `risk_score`、`shared_files`、`shared_symbols`、`detected_at`、`reason`、`confidence`（`crates/graph-core/src/edge.rs:231-242`）；创建分支 `crates/relationship-engine/src/engine.rs:169` 起 |
| FR-07-27 | Cypher 子集查询，适配 4 个备选 Graph DB（源 FR-GRAPH-007） | P0 | 🟡 部分实现 | DSL 到 Cypher 生成：`crates/query-engine/src/engine.rs:63-66`；Cypher guard：`crates/query-engine/src/cypher_guard.rs`；执行 stub：`engine.rs:70-79`；Neo4j 实现全部返回 `GRAPH.NEO4J_NOT_IMPLEMENTED`：`crates/graph-core/src/neo4j_repo.rs:170-184`；内存实现：`crates/graph-core/src/memory_repo.rs:77` |
| FR-07-28 | Graph 索引：Worktree 与 Risk 上的查询加速索引（源 FR-GRAPH-008） | P0 | 🟡 部分实现 | Edge 表有 from / to / kind 索引：`db/migrations/2026-09-16-worktree-canvas-graph-edge.sql:84-88`；Node 表 DDL 中无索引（`db/migrations/2026-09-16-worktree-canvas-graph-node.sql:17-44`）；无 Risk 表 |
| FR-07-29 | Event Bus 增量更新 Graph，不每次全量重算（源 FR-GRAPH-011、FR-RISK-008） | P0 | 🟡 部分实现 | 15 类事件：`crates/event-bus/src/events.rs:34-64`；100ms 防抖：`crates/event-bus/src/debouncer.rs:6-38`；派生 Risk Edge 仅占位：`crates/event-bus/src/delta.rs:89`；Git 事件订阅返回空流：`crates/git-observer/src/watcher.rs:161-165` |
| FR-07-30 | N-hop 邻域查询，1/2/3-hop（源 FR-GRAPH-012） | P0 | 🟡 部分实现 | 内存仓储实现：`crates/graph-core/src/memory_repo.rs:236`，有测试 `:606`；`crates/relationship-engine/src/n_hop.rs:35-70` 与本地 BFS `:56`；BFF `GET /v1/worktree-canvas/worktrees/{id}/graph` 返回空结果：`bff/src/worktree_canvas/controller.rs:181-192` |
| FR-07-31 | 风险类型 11 种（源 FR-RISK-001） | P0 | ✅ 已实现 | `RiskType` 含 MergeConflict、FileOverlap、SymbolOverlap、Stale、Divergence、Dependency、TestFailure、BuildFailure、AgentIncomplete、ReviewMissing、Superseded（`crates/risk-engine/src/risk.rs:15-60`） |
| FR-07-32 | 风险引擎 V1：Git Status 与 Ahead/Behind、File overlap（源 FR-RISK-002） | P0 | 🟡 部分实现 | 已读取的检测：MergeConflict（`crates/risk-engine/src/v1_git_status.rs:41-59`）与 FileOverlap（`:61-79`，阈值 5 个修改文件）；Ahead/Behind 类信号未在已读范围内核实 |
| FR-07-33 | 风险引擎 V2：Diff overlap 与 Line overlap（源 FR-RISK-003） | P1 | 🟡 部分实现 | 检测函数：`crates/risk-engine/src/v2_diff_overlap.rs:29`（共享文件及行重叠）；由 `InMemoryRiskEngine::detect_with_v2` 调用（`crates/risk-engine/src/engine.rs:112-116`），无 API 路由调用 |
| FR-07-34 | 风险引擎 V3：Symbol overlap 与 AST（源 FR-RISK-004） | P2 | 🟡 部分实现 | `crates/risk-engine/src/v3_symbol_overlap.rs:25-48` 对调用方传入的共享符号列表打分；`InMemoryRiskEngine` 未调用（`engine.rs:105-116`）；默认开关关闭（`v3_symbol_overlap.rs:57-59` 测试） |
| FR-07-35 | 冲突预测：合并前发现潜在冲突并输出 CONFLICTS_WITH（源 FR-RISK-005） | P0 | 🟡 部分实现 | `predict_conflict` 从已存储的风险推导最大分数：`crates/risk-engine/src/engine.rs:153-170`；未调用 Git 做合并预检 |
| FR-07-36 | 风险评分 0–1（源 FR-RISK-006） | P0 | 🟡 部分实现 | V1 与 V3 检测器对分数取 `min(1.0)`（`v1_git_status.rs:48`、`:68`；`v3_symbol_overlap.rs:34`）；`confidence` 未参与评分 |
| FR-07-37 | 每条风险生成可解释 reason（源 FR-RISK-007） | P0 | 🟡 部分实现 | reason 字符串：`v1_git_status.rs:54`、`:74`；`v3_symbol_overlap.rs:40`；无 AI 解释接口，BFF `GET /v1/worktree-canvas/risks` 返回空列表：`controller.rs:297-303` |
| FR-07-38 | Git Observer 检测到变化后增量重算风险（源 FR-RISK-008） | P0 | ⬜ 未实现 | `detect_risks` 仅返回已存储风险：`crates/risk-engine/src/engine.rs:136-151`；Git 事件流为空：`crates/git-observer/src/watcher.rs:161-165` |
| FR-07-39 | 风险解决跟踪，Resolved 事件（源 FR-RISK-009） | P1 | 🟡 部分实现 | `resolve_risk` 设置 `resolved_at` 并广播 `RiskEvent::Resolved`：`crates/risk-engine/src/engine.rs:187-200`；无对外 API |
| FR-07-40 | 根据 Risk 与 Health 推荐下一步（源 FR-RISK-010） | P1 | 🟡 部分实现 | `recommend` 依据最大扣分项给出建议：`crates/health-engine/src/score.rs:358-379`；不含 Risk 维度 |
| FR-07-41 | 风险检测与解决写入审计日志，SCD Type 2（源 FR-RISK-011） | P1 | ⬜ 未实现 | risk-engine 与 relationship-engine 中无审计写入（`git grep -i audit` 仅命中错误上下文注释，`crates/risk-engine/src/error.rs:20`） |
| FR-07-42 | WorkItem 关系类型与实体（源 domain-relation-spec §2、REQ-COLLAB-002） | P1 | 🟡 部分实现 | `RelationType` 含 BLOCKS、DUPLICATES、CLOSES、IMPLEMENTS、REFERENCES、DEPENDS、SIBLING（`crates/domain-relation/src/lib.rs:142-170`）；`Relation` 聚合根 `:205`；创建 / 删除命令 `:334`、`:353`。规格列出的 5 类词表与此不同 |
| FR-07-43 | 关系不变量：自关系禁止、重复禁止、循环检测（源 domain-relation-spec INV-R-01 至 03） | P1 | 🟡 部分实现 | 自关系与重复的测试：`crates/domain-relation/src/lib.rs:1271`、`:1310`；`RelationError::CycleDetected` 与错误码 `RELATION_CYCLE_DETECTED` 已定义（`:299-300`、`:320`），未检出构造该错误的调用点 |
| FR-07-44 | 跨实体关系只读列表页（源 `/relation` 页面，无对应 SRS 编号） | P1 | 🟡 部分实现 | `frontend/src/app/relation/page.tsx:1-76` 为只读表格，列 From / Kind / To / Created（`:39`）；按 `blocks`、`duplicates`、`cloned_from` 着色（`:53-55`），与 `domain-relation` 词表不一致 |

## 4. 设计

### 4.1 概念模型

- **Worktree Node**：Git 事实（分支、HEAD、ahead/behind、dirty、合并时间）加上人类可读状态。Rust 侧结构为 `WorktreeNode`（`crates/graph-core/src/node.rs:191`）；状态枚举 `HumanState` 目前包含 12 个变体，把人类态与机器态合在一起（`crates/graph-core/src/state.rs:29`），与 SRS 的 7 个人类态（RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE）不一致。
- **Edge**：13 类，分为派生关系（DERIVED_FROM、BASED_ON、USES_BRANCH、MERGED_INTO）、执行关系（WORKS_ON、IMPLEMENTED_IN）、代码关系（MODIFIES、MODIFIES_SYMBOL）与风险关系（CONFLICTS_WITH、OVERLAPS_WITH、DEPENDS_ON、BLOCKS、SUPERSEDES）（`crates/graph-core/src/edge.rs:64`）。`HighRiskKind` 声明四类高风险边须携带 metadata（`crates/relationship-engine/src/edge_ops.rs:17-30`）。
- **Risk**：11 类风险，带 `risk_score`（0–1）、`reason`、`related_worktrees`，可解决（`crates/risk-engine/src/risk.rs:62`）。
- **View Mode**：TREE、DEPENDENCY、RISK、AGENT、HISTORY 五种（`crates/canvas-renderer/src/view_mode.rs:17-27`）。页面实现的是同名五页签，但每个页签的数据对象与 SRS 不同（见 4.7）。
- **Semantic Zoom**：Rust 侧 LOD 为 3 级，L0 Project、L1 Repo + Worktree、L2 Task + Agent，阈值 0.3 与 0.7（`crates/canvas-renderer/src/lod.rs:20-26`、`:64-78`）。SRS 定义为 6 级 L0–L5，二者不一致。
- **Action**：`ActionType` 共 21 个变体，按 Safe / Warning / Destructive 分类（`crates/action-engine/src/action.rs:30`、`:42`）。SRS 定义为 18 个核心动作。

### 4.2 数据模型

| 实体 | 位置 | 状态 | 说明 |
|---|---|---|---|
| WorktreeNode | `crates/graph-core/src/node.rs:191-214` | 🟡 部分实现 | 12 个字段，缺 SRS FR-GRAPH-004 列出的人类态、健康度、风险计数、测试状态、锁与归档字段 |
| AgentSessionNode | `crates/graph-core/src/node.rs:270-282` | 🟡 部分实现 | 11 个字段，缺 `result_state` |
| ConflictsWithEdge | `crates/graph-core/src/edge.rs:231-242` | ✅ 已实现 | 字段与 FR-GRAPH-006 对齐 |
| DerivedFromEdge | `crates/graph-core/src/edge.rs:164` | ✅ 已实现 | 含 `merge_base_sha` 可选字段，`crates/relationship-engine/src/engine.rs` 的创建分支构造该类型 |
| Risk / RiskEdgeMetadata | `crates/risk-engine/src/risk.rs:62`、`:83` | ✅ 已实现 | 风险对象与边元数据结构 |
| `worktree_canvas_graph_node` | `db/migrations/2026-09-16-worktree-canvas-graph-node.sql:17-44` | 🟡 部分实现 | 表定义、`ENABLE` 与 `FORCE` RLS（`:44-45`）已有；无 Rust 代码读写该表（`crates/worktree-service/src/pg_worktree_repository.rs:34` 仅为注释中的外键说明） |
| `worktree_canvas_graph_edge` | `db/migrations/2026-09-16-worktree-canvas-graph-edge.sql:12-44` | 🟡 部分实现 | 表定义，唯一约束 `(from_id, to_id, edge_kind)`，RLS 与 tenant / workspace 策略（`:45-61`），索引（`:84-88`）；无读写路径 |
| `worktree_canvas_graph_fingerprint` | `db/migrations/2026-09-16-worktree-canvas-graph-fingerprint.sql` | ❓ 未核实 | DDL 文件存在，字段与策略未核对 |
| WorktreeNodeData / EdgeData | `frontend/src/components/worktree-canvas/types.ts` | 🟡 部分实现 | 前端类型镜像，仅被未挂载的组件与 store 引用 |
| Relation（WorkItem 关系） | `crates/domain-relation/src/lib.rs:205` | 🟡 部分实现 | 聚合根存在，与 `/relation` 页面数据源未直接连通 |

### 4.3 接口

| 接口 | 位置 | 状态 | 说明 |
|---|---|---|---|
| BFF REST 11 路由（`/v1/worktree-canvas/...`） | `bff/src/worktree_canvas/controller.rs:49-75` | 🟡 部分实现 | 路由已挂载；列表类返回空集合（`:82-88`、`:107-117`、`:298-303`），单个资源返回 404 占位（`:168-178`、`:195-217`），健康度返回固定 100（`:306-318`），搜索与 NL 为占位（`:321-352`） |
| BFF `GET /v1/worktree-canvas/bff-health` | `bff/src/worktree_canvas/controller.rs:361`；`bff/src/worktree_canvas/mod.rs:149-150` | ✅ 已实现 | 公开的实例健康检查，不依赖 state |
| BFF SSE 16 路由 | `bff/src/worktree_canvas/sse.rs:216`（`SSE_ROUTE_COUNT`）、`:93`、`:108-110` | 🟡 部分实现 | 每个路由返回带 keepalive 的空流，阶段 1 无业务事件（`sse.rs:5`、`:33`、`:92`） |
| BFF WebSocket 1 路由 | `bff/src/worktree_canvas/ws.rs:30` | 🟡 部分实现 | 有 heartbeat 与消息结构，无业务过滤与取消订阅追踪（`ws.rs:68`） |
| BFF worktree_canvas 功能开关 | `bff/Cargo.toml:42-43`；`bff/src/lib.rs:29-30` | ✅ 已实现 | `worktree_canvas` 为默认开启的 feature |
| 前端 MSW mock `/api/worktree-canvas/*` | `frontend/src/mocks/handlers/worktree-canvas.ts:41-86`；注册于 `frontend/src/mocks/handlers/index.ts:73` | 🟡 部分实现 | 仅 mock。路径前缀 `/api` 与 BFF 的 `/v1` 不一致，`git grep "v1/worktree-canvas"` 在前端无调用，无桥接路由 |
| 前端 MSW mock `/api/graph/*` | `frontend/src/mocks/handlers/graph.ts:54`（ensure-fresh）、`:77`（cypher）、`:90`（health）；注册于 `frontend/src/mocks/handlers/index.ts:69` | 🟡 部分实现 | 仅 mock；`frontend/src/app/api/` 下无对应 route handler；BFF 中无 `/graph` 路由 |
| `GraphRepository` trait | `crates/graph-core/src/repository.rs:99-160` | 🟡 部分实现 | Trait 完整，内存实现可用（`memory_repo.rs:77`），Neo4j 占位（`neo4j_repo.rs:170-184`） |
| `RiskEngine` trait | `crates/risk-engine/src/engine.rs:39-61` | 🟡 部分实现 | 内存实现 `InMemoryRiskEngine`（`engine.rs:82`）；`detect_risks` 仅返回已存风险（`:136-151`） |
| `RelationshipEngine` trait | `crates/relationship-engine/src/engine.rs:21` | 🟡 部分实现 | 内存实现（`engine.rs:89`、`:146`）；创建边时校验高风险 metadata（`:150`） |
| `QueryEngine` | `crates/query-engine/src/engine.rs:63-79` | 🟡 部分实现 | DSL 转 Cypher 可用；`execute_dsl` 返回空结果 |
| `LayoutEngine` | `crates/layout-engine/src/engine.rs:210-270` | 🟡 部分实现 | `compute` 为 MVP 网格布局，三种算法 Dagre / D3Force / Elk 仅按列数区分（`engine.rs:231-258`）；`visual_distance` 公式存在（`crates/layout-engine/src/visual_distance.rs:30-32`） |
| `ActionEngine` | `crates/action-engine/src/engine.rs:198-199`、`:230-246` | 🟡 部分实现 | 动作注册与元数据存在；执行为 `execute_stub`，返回成功与预设状态，不产生副作用 |
| `EventKind` / `CanvasEvent` | `crates/event-bus/src/events.rs:34-64`、`:239-311` | 🟡 部分实现 | 15 类事件定义与防抖存在，无订阅方在链路中消费 |
| `DefaultRiskProvider` | `crates/worktree-service/src/service_impl.rs:125-133` | 🟡 部分实现 | 默认风险提供者返回空结果（注释标注为占位） |

### 4.4 状态机与关键流程

1. **页面渲染判定**（`frontend/src/app/canvas/worktree-graph/page.tsx`）：每个视图按固定顺序判定，先未授权，再未接入，再空数据，最后有数据。
   - 未授权：Group API 会话状态为 `invalidated`（`page.tsx:59-60`）。
   - 未接入：`useWorktreeGroupApi()` 返回 `null`（`page.tsx:48-49`）。默认 context 为 `null`（`frontend/src/lib/group/groupProjection.tsx:58`），默认会话状态为 `unavailable`（`:60`）。
   - 空数据：依赖各视图对应的 store 数组为空（`page.tsx:119-121`、`:160-162`、`:199`、`:225`、`:255`）。
   - 有数据：渲染对应视图。
2. **风险生命周期**（`crates/risk-engine/src/engine.rs`）：检测结果经 `store_risks` 写入内存并广播 `RiskEvent::Detected`（`:118-125`）；`resolve_risk` 设置 `resolved_at` 并广播 `RiskEvent::Resolved`（`:187-200`）。检测入口 `detect_with_v1`、`detect_with_v2`（`:105-116`）目前无外部触发源。
3. **冲突预测**：从已存储风险中取 a ↔ b 之间最大的 `FileOverlap` 分数（`engine.rs:153-170`）。
4. **危险动作**：BFF 先做 RBAC，再检查幂等键缓存，再对 Destructive 动作要求 `confirm=true`（`controller.rs:238-261`），最后写入审计（`:281-292`）；动作本身为占位（`:263-279`）。
5. **N-hop 查询**：`GraphRepository::n_hop`（`memory_repo.rs:236`）或 `NHopQuery::execute`（`crates/relationship-engine/src/n_hop.rs:35-51`）。

### 4.5 权限与安全

- **BFF RBAC**：每个 REST handler 调用 `state.permission.require(PermissionLevel::View)` 或 `require_action`（`controller.rs:85`、`:111`、`:172`、`:200`、`:212`、`:239`）。模块文档声明 5 个角色与 18 个动作的权限矩阵（`bff/src/worktree_canvas/mod.rs:9`）。
- **幂等**：创建与动作执行先查询幂等缓存（`controller.rs:129`、`:242`）。
- **审计**：Destructive、Warning 等动作调用 `WorktreeAudit::is_required` 后写入事件（`controller.rs:281-292`）。
- **图存储隔离**：三张图表启用并强制 RLS（`worktree-canvas-graph-node.sql:44-45`、`worktree-canvas-graph-edge.sql:45-46`），edge 表定义 tenant 与 workspace 策略（`:48-61`）。由于没有读写代码，RLS 策略尚未被执行路径验证。
- **前端可见性**：页面不做逐视图权限判断，仅依赖宿主 Group API 会话是否存在且有效（`page.tsx:48-60`）。宿主 Provider 仅在存在 run-directory 会话时挂载（`frontend/src/app/providers.tsx:52-59`）。
- **密钥**：页面与组件中无凭证处理。

### 4.6 依赖关系

- **Rust crate**：`graph-core`（节点、边、仓储）、`risk-engine`（依赖 `graph-core`、`git-adapter` 的 `Diff`）、`relationship-engine`、`health-engine`、`canvas-renderer`、`layout-engine`、`query-engine`、`action-engine`、`event-bus`、`git-observer`、`worktree-service`、`worktree-canvas`（facade，`crates/worktree-canvas/src/lib.rs:49-62`）。
- **V3 依赖**：tree-sitter 符号解析由 `crates/star-treesitter` 提供（根 `Cargo.toml` 注释；`crates/risk-engine/src/v3_symbol_overlap.rs:3`）。当前 V3 检测器不直接调用它。
- **域边界**：`domain-relation` 为 WorkItem 关系域，与 Worktree 图的 `relationship-engine` 不共享数据模型。
- **前端**：全局 store（`frontend/src/lib/store.ts`，初始数组为空：`:402-412`、`:434-435`）、Group API（`frontend/src/lib/group/groupProjection.tsx`）、App Shell（`frontend/src/components/AppShell.tsx`）、`worktreeCanvasStore`（仅被未挂载组件引用）。
- **外部服务**：Neo4j（未实现）、Git（事件流未接通）。

### 4.7 页面设计

本功能拥有两个页面路由：`/canvas/worktree-graph`（主页面）与 `/relation`（关系列表，归属待确认）。SRS 中的 `/worktree-canvas` 路由在代码基线中不存在页面。

### 4.7.1 /canvas/worktree-graph

文件: `frontend/src/app/canvas/worktree-graph/page.tsx`（页面组件 `WorktreeGraphPage`，`:36`）。

布局:

- 根 Provider：`frontend/src/app/providers.tsx:52-59`（宿主 Group API Provider，仅在存在 run-directory 会话时挂载）。
- 路由段布局：`frontend/src/app/canvas/worktree-graph/layout.tsx:7-12`（`h-screen flex-col` 容器）。
- 画布外层：`frontend/src/app/canvas/layout.tsx:17-18`（`AppShell fullBleed`）。
- 顶栏：`frontend/src/components/AppShell.tsx:48-49`（`AppHeader`）。
- 主区：`frontend/src/components/AppShell.tsx:61-74`（`main`，fullBleed 模式）。
- 右侧栏：`frontend/src/components/AppShell.tsx:77`（`RightChatRail`）。
- 侧栏与底栏：AppShell 中未挂载侧栏或底栏元素。
- 页面内顶部视图切换条与缩放控件：`page.tsx:278-312`；图区域：`page.tsx:315-321`。

组件树:

- `WorktreeGraphPage`（`page.tsx:36`）
  - 顶部切换条（`:278-312`）
    - 返回画布链接 `/canvas/canvas-001`（`:280-286`）
    - 五个视图页签（`:287-303`）
    - 缩放控件（`:306-311`）
  - 图区域 `main[data-testid=wtg-canvas]`（`:315-321`）
    - `renderView()`（`:108-274`）
      - 树视图（`:110-155`）
      - 依赖视图（`:156-193`）
      - 风险视图（`:194-220`）
      - Agent 视图（`:221-249`）
      - 历史视图（`:250-270`）
- 段级文件：`loading.tsx:6`（加载占位）、`error.tsx:13-16`（错误边界）、`not-found.tsx:7-12`（404 页，回链指向不存在的 `/worktree-canvas`）。

状态矩阵（适用于五个视图）:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 宿主已接入，且 store 中有对应数据 | 树：`wtg-view-tree`；依赖：`wtg-view-deps`；风险：`wtg-view-risk`；Agent：`wtg-view-agent`；历史：`wtg-view-history`（`page.tsx:123`、`:164`、`:201`、`:227`、`:257`） | 🟡 部分实现：呈现的对象与 SRS 不同（见 4.7.1 差距） |
| 加载 | 路由段加载 | `worktree-canvas-loading`（`loading.tsx:6`） | 🟡 部分实现：仅路由段占位，页面自身无数据加载状态（页面不发起请求） |
| 空 | 对应数组为空 | `wtg-empty-<view>`，文案「空数据 — …」（`page.tsx:97-106`） | ✅ 已实现 |
| 错误 | 渲染期异常 | `worktree-canvas-error`（`error.tsx:16`） | 🟡 部分实现：仅 React 错误边界，无数据请求错误展示 |
| 未授权 | 会话状态为 `invalidated`（HTTP 401 或缺 token） | `wtg-unauthorized-<view>`（`page.tsx:82-94`） | ✅ 已实现（依赖 Group API 失效信号，`groupProjection.tsx:88-100`） |
| 未接入 | `useWorktreeGroupApi()` 为 `null`，即网页预览模式 | `wtg-disconnected-<view>`（`page.tsx:67-79`） | ✅ 已实现 |
| 禁用 | 无 | 无禁用态 | ⬜ 未实现：页签与缩放按钮无禁用条件 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换视图页签 | 无 | 更新 `activeView`，重渲染对应视图（`page.tsx:37`、`:292`） | 无请求 | 🟡 部分实现：不重算数据，但视图内容为占位 |
| 缩小 / 放大 / 100% | 无 | 调整 `zoom` 并以 CSS scale 应用，范围 0.1–3（`page.tsx:38`、`:308-310`、`:316-318`） | 无 | 🟡 部分实现：无 LOD 切换，无 Fit |
| 返回画布 | 无 | 跳转 `/canvas/canvas-001`（`page.tsx:280-282`） | 路由跳转 | ✅ 已实现 |
| 打开任务（依赖、风险视图） | 数据已加载 | 跳转 `/work-item/<id>`（`page.tsx:175-181`、`:205-208`） | 路由跳转 | ❓ 未核实：目标页面 `frontend/src/app/(app)/work-item/[id]/page.tsx` 存在，未核对其数据读取 |
| 打开 Worktree 分组（Agent 视图） | 数据已加载 | 跳转 `/worktree/<id>/group`（`page.tsx:238-242`） | 路由跳转 | ✅ 已实现（目标页面 `frontend/src/app/worktree/[id]/group/page.tsx` 存在） |
| 选中节点 / 双击打开 Inspector | 无 | 无 | 无 | ⬜ 未实现 |
| 拖动、框选、折叠、键盘快捷键、搜索 | 无 | 无 | 无 | ⬜ 未实现 |

数据:

- 来源：全局 store `useStore`（`page.tsx:39-43`），字段 `worktrees`、`workItems`、`agentSessions`、`projects`、`lanes`。初始值为空数组（`frontend/src/lib/store.ts:402-412`、`:434`）。
- 宿主接入信号：`useWorktreeGroupApi`、`useWorktreeGroupSessionState`（`page.tsx:30`、`:48`、`:59`）。页面不调用这些 client 的任何方法获取数据。
- 字段使用：Worktree 的 `id`、`name`、`status`（`page.tsx:134-141`、`:230`）；WorkItem 的 `id`、`key`、`title`、`status`、`priority`、`updated_at`、`worktree_id`（`:146`、`:172-180`、`:196`、`:252`）；AgentSession 的 `id`、`name`、`agent_kind`、`status`、`worktree_id`（`:229-236`）；Project 的 `name`（`:126`）；Lane 的 `name`（`:131`）。
- REST 与 SSE：页面不调用 BFF 或 MSW mock 接口。

与文档设计的差距:

- FR-07-01：路由与 SRS 不一致，not-found 回链指向不存在的路由。
- FR-07-02、FR-07-03：无限画布与 Fit 未实现。
- FR-07-04：五视图切换存在，但数据对象与 SRS 不符。
- FR-07-05：TREE 未读取 DERIVED_FROM。
- FR-07-06、FR-07-07、FR-07-08、FR-07-09：DEPENDENCY、RISK、AGENT、HISTORY 的对象与 SRS 不同（任务代替边与 Worktree，无透明度与发光，无 Task → Agent → Worktree 树，无 merged 与 SUPERSEDES）。
- FR-07-10、FR-07-11：Focus 未接入页面，透明度数值与 SRS 不同。
- FR-07-12、FR-07-13、FR-07-14、FR-07-15、FR-07-16、FR-07-17：过滤、搜索、Minimap、Inspector、键盘与折叠未接入。
- FR-07-18：确认对话框组件未挂载。

### 4.7.2 /relation

文件: `frontend/src/app/relation/page.tsx`（`RelationPage`，`:1-76`）。

布局: 代码基线中未找到 `frontend/src/app/relation/layout.tsx`，外层布局为 app 根布局（❓ 未核实）。

组件树: `RelationPage` → `PageHeader`（标题与副标题，`:27-34`）→ 只读表格（`:36-76`）。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | store 中 `relations` 非空 | 表格逐行显示 From（`StatusPill` 与名称）、Kind 标签、To、Created | 🟡 部分实现：颜色仅覆盖 blocks、duplicates、cloned_from |
| 空 | `relations` 为空 | 表格无数据行，计数为 0 | ⬜ 未实现：无空态文案 |
| 加载、错误、未授权、未接入、禁用 | 无 | 无 | ⬜ 未实现 |

交互: 无。页面无创建、删除或编辑操作（全文 76 行，无按钮或表单）。

数据: store `relations`、`worktrees`、`workItems`、`agentSessions`、`changeSets`（`frontend/src/app/relation/page.tsx:11-15`）。`lookup` 按 kind 解析名称（`:17-23`）。`relations` 的写入路径经 store 持久化快照恢复（`frontend/src/lib/store.ts:1749`），与 `domain-relation` 无直接连接。

与文档设计的差距:

- FR-07-42：页面词表（blocks、duplicates、cloned_from）与 `domain-relation` 词表及 domain-relation-spec 的 5 类词表均不一致。
- FR-07-43：无关系创建与循环检测的 UI 入口。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 画布页面（五视图占位） | `frontend/src/app/canvas/worktree-graph/` | 🟡 部分实现 | `page.tsx:36-324` |
| 画布组件库（未挂载） | `frontend/src/components/worktree-canvas/` | 🟡 部分实现 | 组件存在，`git grep` 显示无页面以 `@/components/worktree-canvas/` 路径导入（types 与 store 除外） |
| Worktree 图 store | `frontend/src/stores/worktreeCanvasStore.ts` | 🟡 部分实现 | 仅被 `components/worktree-canvas/` 内组件引用 |
| MSW mock（worktree-canvas、graph） | `frontend/src/mocks/handlers/worktree-canvas.ts`、`graph.ts` | ✅ 已实现（仅 mock） | 仅在 `NEXT_PUBLIC_API_MOCKING=enabled` 时启动（`frontend/src/instrumentation.ts:10-17`） |
| 关系列表页 | `frontend/src/app/relation/page.tsx` | 🟡 部分实现 | `:1-76` 只读表格 |
| BFF worktree_canvas 模块 | `bff/src/worktree_canvas/` | 🟡 部分实现 | 路由挂载完整，handler 为占位（`controller.rs:82-352`） |
| BFF SSE / WS | `bff/src/worktree_canvas/sse.rs`、`ws.rs` | 🟡 部分实现 | 空流与占位（`sse.rs:5`、`:108-110`；`ws.rs:68`） |
| 门面 crate | `crates/worktree-canvas/src/lib.rs` | ✅ 已实现 | 14 个子 crate 的 re-export（`:49-62`）与 smoke 测试（`:74-95`） |
| graph-core 类型 | `crates/graph-core/src/node.rs`、`edge.rs`、`state.rs` | ✅ 已实现 | `NodeKind` 11 个、`EdgeKind` 13 个（`node.rs:64`、`edge.rs:64`） |
| graph-core 内存仓储 | `crates/graph-core/src/memory_repo.rs` | ✅ 已实现 | `:77`、`n_hop` `:236`、测试 `:606` |
| graph-core Neo4j 仓储 | `crates/graph-core/src/neo4j_repo.rs` | ⬜ 未实现 | 所有方法返回 `GRAPH.NEO4J_NOT_IMPLEMENTED`（`:170-184`） |
| risk-engine | `crates/risk-engine/src/` | 🟡 部分实现 | V1、V2 可调用；V3 未接入引擎；无审计；无 Git 事件驱动（`engine.rs:105-200`） |
| relationship-engine | `crates/relationship-engine/src/` | 🟡 部分实现 | 内存引擎与 `create_edge`（`engine.rs:146-180`）；N-hop（`n_hop.rs:35-70`） |
| health-engine | `crates/health-engine/src/score.rs` | 🟡 部分实现 | 13 项扣分计算（`:183-351`）与推荐（`:358-379`）；无 API 消费 |
| canvas-renderer | `crates/canvas-renderer/src/` | 🟡 部分实现 | 视图模式（`view_mode.rs:17-27`）、3 级 LOD（`lod.rs:20-26`）、focus（`focus.rs`）可用；Cluster stub（`viewport.rs:3-8`）；React Flow 适配器未被页面使用 |
| layout-engine | `crates/layout-engine/src/` | 🟡 部分实现 | `visual_distance`（`visual_distance.rs:30-32`）与增量更新（`incremental.rs:19`）可用；`compute` 为 MVP 网格（`engine.rs:231-258`）；仅 `/wasm-demo` 页面使用 WASM 包装 |
| query-engine | `crates/query-engine/src/` | 🟡 部分实现 | DSL 关键字与 Cypher 生成可用（`keywords.rs:39-43`、`engine.rs:63-66`）；执行为 stub（`engine.rs:70-79`）；NL 为模板（`nl_translator.rs:30-35`） |
| action-engine | `crates/action-engine/src/` | 🟡 部分实现 | 21 个 `ActionType`（`action.rs:42`）；执行为 stub（`engine.rs:230-246`） |
| event-bus | `crates/event-bus/src/` | 🟡 部分实现 | 15 类事件（`events.rs:34-64`）与 100ms 防抖（`debouncer.rs:6-38`）；`delta.rs:89` 占位 |
| git-observer | `crates/git-observer/src/watcher.rs` | 🟡 部分实现 | 订阅返回空流（`:161-165`） |
| worktree-service 风险提供者 | `crates/worktree-service/src/service_impl.rs` | 🟡 部分实现 | 默认提供者返回空（`:125-133`）；`NoRisk` 占位事件（`:218`） |
| domain-relation | `crates/domain-relation/src/lib.rs` | 🟡 部分实现 | 聚合、命令与 16 个 `#[test]` / `#[tokio::test]`（`:142-384`、`:1243-1383`）；循环检测未接入调用点 |
| 图持久化 DDL | `db/migrations/2026-09-16-worktree-canvas-graph-*.sql` | 🟡 部分实现 | 三张表、RLS、edge 索引；无读写代码 |
| 端到端测试 | `frontend/e2e/worktree-canvas.spec.ts` | ⬜ 未实现 | 8 个用例，全部 `test.skip`，注释标明实跑落在后续子任务 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 路由名不一致：SRS 为 `/worktree-canvas`，页面为 `/canvas/worktree-graph`，not-found 回链指向不存在的路由 | 404 回链；文档与入口不一致 | 确定正式路由；补 redirect 或修正 `not-found.tsx:10` 的链接；同步 SRS |
| 无限画布与 Fit、拖动、框选、折叠、键盘未接入（FR-07-02、03、16、17） | 页面不具备画布能力，只能看占位列表 | 把 `frontend/src/components/worktree-canvas/` 的组件（或 `canvas-renderer` 产物）挂入页面，接入视口与交互 |
| 页面数据只来自 store 与宿主 presence，没有 REST 或 SSE 调用 | 数据在独立部署下无来源；`/api` 与 `/v1` 路径不一致 | 统一路径，让页面通过 BFF `/v1/worktree-canvas/*` 或 Group API 读取 Worktree 图数据；移除对 `/api` mock 的依赖 |
| BFF 全部 handler 为占位（列表空、单资源 404、动作无副作用） | 后端能力不可验证 | 接入 `worktree-service` 与 `graph-core` 的真实仓储；为 handler 增加集成测试 |
| Neo4j 仓储未实现（FR-07-27） | 图查询只有内存实现 | 实现 Bolt 驱动，或明确阶段 2 边界并以 PG 表作为持久化 |
| 图表 DDL 无读写路径（FR-07-28） | RLS 与索引未被执行路径验证 | 实现 PG repository 对 `worktree_canvas_graph_node` / `_edge` 的读写，并补 RLS 测试 |
| Git 事件流为空（`git-observer` 的 `subscribe`），Event Bus delta 占位（FR-07-29、FR-07-38） | 风险与图无法实时更新 | 实现 watcher 的事件流与广播订阅；把 `delta.rs:89` 的风险边派生接入 risk-engine |
| V3 未接入引擎，V2 与冲突预测无 API 入口（FR-07-33 至 35） | 风险结果不能从 UI 获取 | 在 `InMemoryRiskEngine` 中调度 V1–V3；通过 BFF `risks` 与 `search` 暴露 |
| 风险审计未写入（FR-07-41） | 不满足 SCD Type 2 审计要求 | 为 `detect_with_*` 与 `resolve_risk` 增加审计写入 |
| Action 为 stub，缺 Set Dependency（FR-07-19、20） | 依赖与合并操作无实际效果 | 为 `RemoveDependency`、`SetDependency` 与 Merge 实现执行器，经 action-engine 调用 |
| Rust LOD 为 3 级、Focus 透明度为 0.8/0.6/0.4，与 SRS 的 6 级与 100/80/20 不一致（FR-07-11、4.1） | 设计与实现不对齐 | 以 SRS 为准修订 crate，或修订 SRS 并记录决定 |
| `HumanState` 为 12 个变体，SRS 为 7 个人类态（FR-07-24） | 状态语义不一致 | 拆分人类态与机器态，以 SRS §九 为准 |
| `/relation` 与 `domain-relation` 词表不一致；无循环检测调用点（FR-07-42、43） | 关系类型语义混乱，INV-R-03 无法保证 | 统一词表；在创建路径调用循环检测并产生 `CycleDetected` |
| Node 与 AgentSession 字段缺失（FR-07-24、25） | 视图与检查器无法展示 SRS 列出的字段 | 补齐字段并同步 `WorktreeNode`、`AgentSessionNode` 与 DDL |
| Run 归属迁移未做（SRS §1.6） | 当前 Worktree 级接口只是兼容切片 | 按 Phase 2E 设计 Run-scoped schema 与 API |
| 测试设计与实施不一致：E2E 8 个全部 skip，页面无测试 | 无法证明页面行为 | 实现 `wc-01` 至 `wc-08`，并为五视图状态矩阵补组件测试 |
| 规格版本漂移：spec v0.1 引用 SRS v1.1、BD v1.0、DD v1.0；INV-WC-05 与 INV-WC-06 指向的 `NodeRenderer.tsx`、`ZoomManager.ts` 不存在 | 实施者按 spec 找不到文件 | 以 SRS v1.11、BD v1.12、DD v1.4 为当前基线更新 spec 的引用与路径 |

## 7. 验收与测试

已存在的测试（位置均在代码基线 637ed19d）:

- Rust 单元测试（in-source）：`crates/risk-engine/src/v1_git_status.rs`、`v2_diff_overlap.rs`、`v3_symbol_overlap.rs`（V3 默认关闭的测试在 `:57` 起）、`crates/relationship-engine/src/edge_ops.rs:159-165`、`crates/canvas-renderer/src/view_mode.rs:120-161`、`lod.rs`、`focus.rs`、`crates/layout-engine/src/visual_distance.rs`、`crates/query-engine/src/keywords.rs:121-128`、`engine.rs:204`、`crates/event-bus/src/debouncer.rs`、`crates/graph-core/src/memory_repo.rs:606`、`crates/domain-relation/src/lib.rs`（16 个测试）。
- Rust 集成测试：`crates/graph-core/tests/repository_behavior.rs`、`serde_and_mapping.rs`；`crates/action-engine/tests/engine_integration.rs`；`crates/worktree-service/tests/pg_worktree_service_integration.rs`、`start_from_picker_integration.rs`、`external_worktree_import.rs`。
- BFF：`bff/src/lib.rs:113-115` 检查 feature 开启时导出 worktree_canvas。BFF 的 `controller.rs`、`mod.rs` 内有模块级测试（未逐条核对）。
- 前端单元测试：`frontend/src/components/worktree-canvas/__tests__/`（ConfirmationModal、HealthBadge、RiskBadge、SearchBar、StatusBadge、TreeView、types、worktreeCanvasStore）；`frontend/src/mocks/__tests__/graph.test.ts`、`graph.handlers.hop.test.ts`（hop 1 与 2 的 fixture 节点数与差集）。上述组件测试覆盖的是未挂载的组件，不覆盖页面。
- 前端端到端：`frontend/e2e/worktree-canvas.spec.ts`，8 个用例（wc-01 至 wc-08）全部 `test.skip`。
- 页面测试：未找到 `/canvas/worktree-graph` 的页面级测试。

已验证的内容: 枚举变体数量（NodeKind 11、EdgeKind 13、RiskType 11、EventKind 15、ActionType 21、LOD 3、ViewMode 5）、页面的五视图状态分支、风险引擎 V1/V2 的检测条件、内存图仓储的 N-hop、ConflictsWithEdge 字段。

未验证的内容:

- 页面在真实宿主会话下的行为，包括未授权与失效会话的切换。
- BFF 路由与前端 `/api` mock 的一致性（路径不同，无桥接）。
- RLS 策略在执行路径中的生效情况（无读写代码）。
- V1 检测中 Ahead/Behind 类信号的完整覆盖。
- `worktree-service` 中 Worktree 状态机迁移表（归属另一功能的核对范围）。
- 测试设计书 TEST-DESIGN-WORKTREE-CANVAS-001 §6.2 中 AC-001 至 AC-043 的逐条实施状态。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-WORKTREE-CANVAS-001.md`（文件头 v1.11，2026-09-15） | 需求编号与优先级（FR-WT-019 至 024、031、032、038；FR-UI-001 至 014；FR-GRAPH-001 至 012；FR-RISK-001 至 011）；用户故事 US-1 至 US-39；渡口边界 §1.6 | 路由名 `/worktree-canvas` 与代码 `/canvas/worktree-graph` 不一致；LOD、Focus 透明度、HumanState、Action 数量与代码不一致 | 作为需求编号与优先级的基线；状态以代码为准；不一致项列入第 6 节 |
| `docs/design/BD-WORKTREE-CANVAS-001.md`（文件头 v1.12） | 架构目标、组件边界、Risk / Health / Action 分层设计（§0.2、§5、§6、§18、§19 标题） | 设计中的 6 级 LOD、18 个动作与代码的 3 级、21 个变体不同 | 采用为设计意图；代码未矛盾之处保留为设计意图，并以状态标记注明实现情况 |
| `docs/design/DD-WORKTREE-CANVAS-001.md`（文件头 v1.4） | Edge 字段（§8）、风险与健康接口（§13、§14）、Focus 与虚拟化（§30、§37）、Action 接口（§18） | 接口名与代码基本一致，实现程度见第 5 节 | 采用为接口命名与字段依据 |
| `docs/specs/worktree-canvas-spec.md`（文件头 v0.1，2026-09-15） | 不变量 INV-WC-01 至 12（§3）、接口稳定承诺与模块边界（§1、§4） | 头部引用 SRS v1.1、BD v1.0、DD v1.0，已落后；INV-WC-05 与 INV-WC-06 的文件路径不存在 | 采用不变量编号；版本与路径以 SRS、BD、DD 的当前头部为准，差异见第 6 节 |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md`（文件头 v2.8） | 五级测试分层、E2E 用例名 wc-01 至 wc-08（§4.2）、AC 验收映射（§6.2） | 头部引用 SRS v1.0、BD v1.0；E2E 全部为 skip | 采用用例命名；实施状态以第 7 节为准 |
| `docs/specs/domain-relation-spec.md`（Draft v0.1，2026-08-25） | WorkItem 关系实体、INV-R-01 至 06、删除不级联（§2、§3） | 规格的 relation_type 为 5 类（blocks、blocked_by、relates_to、duplicates、clones），代码为 7 类大写词表；规格描述的是 WorkItem 关系，不是 Worktree Edge | 作为 WorkItem 关系域的依据，与 Worktree 图分开记录；词表差异列入第 6 节 |
| `docs/requirements/SRS-WORKTREE-CANVAS-001.md` §1.6 与其引用的 Run 边界 | 双 Canvas 消歧：Project 级 Overview 与 Run 级 Group Canvas 分开 | 当前 Worktree 级接口是兼容切片 | 采用为范围边界；Run-scoped schema 迁移列为后续 |
| 代码基线中 `frontend/src/components/worktree-canvas/`、`crates/`、`bff/src/worktree_canvas/`、`db/migrations/2026-09-16-worktree-canvas-*.sql` | 实现状态与证据 | 与 SRS 的差异见第 6 节 | 作为当前状态的唯一判定依据 |
| `docs/arch-live/05-worktree-canvas.html`、`docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md` | 未读取 | 不属于本功能的 seed 列表 | 未采用，不作为状态依据 |
