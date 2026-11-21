# F46 架构图谱查看器

> 状态摘要: 看板卡片的 🕸 Arch 按钮、ArchGraphModal 弹窗与 `/api/graph/*` 的 MSW mock 已有前端代码; 其唯一挂载点已在 bb6b860d 中删除, 当前看板页不向卡片传入 onArchClick, 用户界面无法打开弹窗。后端端点、LLM worker、幂等与排他、memgraph 与 graph.* 表 DDL 均未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

在看板任务卡上打开一个居中弹窗, 显示该 work_item 在架构图中的位置: 当前节点以 cyan 高亮, 1-hop 邻居按节点类型着色, 2-hop 代码侧节点 (cratemodule / symbol) 以 20% 不透明度弱化。图由后台分析本地或 git 代码后写入 memgraph; 同一 work_item 的重复请求按 fingerprint 幂等处理, 并发写入按 per-work_item advisory lock 排他 (源: `docs/architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md` §0; `docs/requirements.md` §48.2 REQ-ARCH-001~005)。

### 1.2 范围

- 范围内: 卡片按钮及其透传; 弹窗渲染与交互; 前端投影类型; `/api/graph/*` 的 MSW 端点与示例 fixture; 三个端点的契约; 后端、存储与 worker 的设计项 (多数为 ⬜)。
- 范围外 (源: ADR §1.3 NG-001~005; requirements §48.6): 节点点击跳转到 IDE、worktree 或 symbol; git push webhook 自动重建; 多 monorepo 跨仓分析; 节点与边的手动编辑; 导出 PNG / SVG / JSON。真实 memgraph 接入 (NG-006) 列为 Phase 3 设计项, 见 FR-46-27。

## 2. 用户与场景

| 角色 | 主要场景 | 来源 |
|---|---|---|
| 架构查看者 | 在看板卡片点击 🕸 Arch, 查看任务在 25 类节点、24 类边组成的图中的位置 | requirements §48.2 REQ-ARCH-001, REQ-ARCH-002 |
| 多名查看者 | 同一任务被多人同时打开时只触发一次分析, 结果共享 | REQ-ARCH-004 AC-1, AC-3 |
| 本地开发者 | 以 `source: "local"` 分析当前 worktree | REQ-ARCH-005 AC-1 |
| 远程或 CI 使用者 | 以 `source: "git"` 分析远程仓指定分支与 SHA | REQ-ARCH-005 AC-2 |

主要用户故事: US-1 打开弹窗并阅读邻居; US-2 点击刷新重新确认数据; US-3 加载失败后重试。

## 3. 需求

优先级为推导值: P0 = requirements §48 中标注为必须的需求与其 AC; P1 = AC 支撑项与 spec 必备契约; P2 = ADR 视觉细则与性能目标。源文档未给出优先级。弹窗与卡片按钮的内部逻辑已存在但未挂载, 对用户不可达, 按 🟡 记录, 缺失环节见 FR-46-02。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-46-01 | 看板卡片第 4 行显示 🕸 Arch 按钮, 仅当父组件传入 onArchClick 时渲染 (REQ-ARCH-001) | P0 | 🟡 | `frontend/src/components/board/KanbanCard.tsx:172-184` (条件渲染于 :173); `KanbanBoard.tsx:91-93, 139, 524, 548` 透传; 无调用方 (见 §5) |
| FR-46-02 | 看板页面向看板传入打开回调, 用户可从卡片打开弹窗 (REQ-ARCH-001 AC-1) | P0 | 🗑 | 挂载于 `frontend/src/app/(app)/projects/ProjectsClient.tsx` 的 import、`useArchGraphTrigger` 调用、`onArchClick` 与 `<ArchGraphModal>` 挂载块, 于 commit bb6b860d 删除 (首次加入见 commit 742d3777); `frontend/src/app/(app)/kanban/page.tsx:412-433` 的 KanbanBoard 不传 onArchClick; `git grep onArchClick -- frontend/src/app` 无命中 |
| FR-46-03 | 点击 🕸 按钮时 `stopPropagation`, 不触发卡片 onClick 跳转 (REQ-ARCH-001 AC-2) | P0 | 🟡 | `KanbanCard.tsx:104-109` (stopPropagation 于 :106); 测试 `KanbanCard.test.tsx:122-142` (未执行, 见 §7) |
| FR-46-04 | 弹窗居中、尺寸 80vw × 80vh、z-50; 支持 Esc、背景点击与 X 关闭 (REQ-ARCH-002) | P1 | 🟡 | 居中与层级 `ArchGraphModal.tsx:405-416`; Esc `:381-388`; 背景 `:411-414`; X `:454-462`; 尺寸上限 `:421-422` 为 min(80vw, 1400px) × min(80vh, 900px), 与需求的无上限尺寸不一致 |
| FR-46-05 | 打开时调用 `POST /api/graph/ensure-fresh`; 200 直接渲染; 202 回退到 `POST /api/graph/cypher` (REQ-ARCH-002 §48.4) | P0 | 🟡 | `ArchGraphModal.tsx:233-250` (请求), `:274-307` (分支); `frontend/src/mocks/handlers/graph.ts:54-87`; 仅 MSW mock, 后端见 FR-46-16 |
| FR-46-06 | 使用 cytoscape.js 与 cose-bilkent 布局渲染图 (REQ-ARCH-002 §48.2) | P0 | 🟡 | `ArchGraphModal.tsx:82-92` (动态加载与注册), `:352-371` (创建实例、布局 cose、ready); 依赖 `frontend/package.json:30-31`; 运行时渲染未经测试 (见 §7) |
| FR-46-07 | 当前 work_item 节点以 cyan `#00f0ff`、64px 高亮 (REQ-ARCH-002 AC-2) | P0 | 🟡 | 常量 `frontend/src/types/graph.ts:187-189`; 选择器 `ArchGraphModal.tsx:129-144` (`node[is_current = "true"]`) |
| FR-46-08 | 1-hop 邻居按节点 kind 着色与设形 (REQ-ARCH-002 AC-3) | P0 | 🟡 | `frontend/src/types/graph.ts:158-185` (NODE_STYLE); `ArchGraphModal.tsx:189-201` |
| FR-46-09 | 1-hop 边为 cyan 2px 实线 (ADR §2.3.3) | P1 | 🟡 | `ArchGraphModal.tsx:153-171`; `frontend/src/types/graph.ts:192` (hop1) |
| FR-46-10 | 2-hop 代码侧节点 (cratemodule / symbol) 以 20% 不透明度弱化 (REQ-ARCH-002 AC-4) | P1 | 🟡 | 样式 `ArchGraphModal.tsx:146-151`; 默认路径不产生 2-hop 节点, 见 `frontend/src/mocks/handlers/graph.ts:71` 与 `frontend/src/mocks/data/graph.ts:262-266` |
| FR-46-11 | 2-hop 边为灰色 `#475569`、1px 点线、30% 不透明度 (ADR §2.3.3) | P2 | 🟡 | `ArchGraphModal.tsx:173-182` (opacity 0.3 于 :179); `frontend/src/types/graph.ts:193` |
| FR-46-12 | 当前 work_item 自身的边以 cyan 3px 全亮显示 (ADR §2.1 边视觉编码) | P2 | ⬜ | `EDGE_STYLE.current` 已定义 (`frontend/src/types/graph.ts:195`), 全仓无引用 |
| FR-46-13 | 非当前 work_item 的 1-hop 边为 dashed (ADR §2.1) | P2 | ⬜ | 1-hop 边固定 solid (`ArchGraphModal.tsx:159` 取 `EDGE_STYLE.hop1.dash`) |
| FR-46-14 | 加载、错误 (含 Retry)、手动刷新; footer 显示节点数、边数与 fingerprint (ADR §2.3.2) | P1 | 🟡 | 加载覆盖层 `ArchGraphModal.tsx:469-481`; 错误与 Retry `:484-507`; 刷新 `:444-453`; footer `:521-538` |
| FR-46-15 | 客户端按 fingerprint 去重渲染, 并以 30 秒 staleTime 缓存 cypher 查询 (spec §5.1 L1) | P2 | 🟡 | `ArchGraphModal.tsx:231`, `:338-339`, `:270` |
| FR-46-16 | ensure-fresh 后端幂等: fingerprint 命中则跳过 agent 并返回已有图 (REQ-ARCH-003 AC-1, spec §5.1) | P0 | ⬜ | 无后端实现: `crates/star-api-rest/src` 无 graph 路由; `crates/star-graph-agent` 目录不存在 |
| FR-46-17 | 同一 work_item 的排他: advisory lock (5 分钟 TTL), in-process coalesce, 202 + Retry-After 与前端轮询 (REQ-ARCH-004, spec §5.2) | P0 | ⬜ | 无后端实现; 前端 202 只做一次 cypher 请求, 无轮询 (`ArchGraphModal.tsx:289-307`) |
| FR-46-18 | cypher 与 health 端点契约 (spec §2.2, §2.3) | P1 | 🟡 | MSW: `frontend/src/mocks/handlers/graph.ts:77-97`; health 无调用方 (`git grep "graph/health" -- frontend/src` 仅 mock 与类型注释); 后端 ⬜ |
| FR-46-19 | source 取值 local 或 git; 默认值依据 ActorContext.local_runtime_id 判定 (REQ-ARCH-005 §48.2) | P1 | 🟡 | 类型 `frontend/src/types/graph.ts:106`; 参数 `ArchGraphModal.tsx:55`; 默认硬编码为 `"local"` (`:223`), hook 不传 source (`:596-600`); git 路径 ⬜ |
| FR-46-20 | 每个请求携带 tenant_id; 13 类 RLS 强制 (REQ-ARCH-003 §48.5, REQ-SEC-001) | P0 | 🟡 | 请求携带 tenant_id (ensure-fresh 由 `ArchGraphModal.tsx:317` 传入并于 `:243` 发送; cypher 见 `:257`); 回退值硬编码 (`:596`); 服务端 RLS ⬜ (无表 DDL, 见 FR-46-27) |
| FR-46-21 | 未接通授权数据时显示未接入状态, 不以示例数据冒充 (`AGENTS.md:22`) | P0 | ⬜ | 弹窗只有加载、错误两类覆盖层 (`ArchGraphModal.tsx:469-507`), 无未接入分支; MSW 启用时 `frontend/src/mocks/data/graph.ts:262-288` 把固定邻居图挂到任意 work_item 上 |
| FR-46-22 | Cypher MERGE 写入幂等 (REQ-ARCH-003 §48.2, ADR §2.2.3) | P1 | ⬜ | 无 memgraph 写入代码 |
| FR-46-23 | LLM 推断确定性 (temperature=0, top_p=0.1, seed) (ADR §2.2.3) | P2 | ⬜ | 无 worker 代码 |
| FR-46-24 | 本地与 git 两种源的 AST 提取、LLM 推断与 25 kind 投影 (ADR §2.2.2, REQ-ARCH-005 AC-1/AC-2) | P0 | ⬜ | 无 `crates/star-graph-agent`; 无 AST 或 git 取数代码 |
| FR-46-25 | 25 类节点与 24 类边的类型投影 (ADR §2.1) | P1 | ✅ | `frontend/src/types/graph.ts:22-33` (GraphNodeKind), `:63-69` (GraphEdgeKind) |
| FR-46-26 | 图的 TypeScript 载荷结构: payload、节点、边、统计与 fingerprint (spec §2.3.5 / ADR §2.3.5) | P1 | ✅ | `frontend/src/types/graph.ts:44-99` |
| FR-46-27 | 图数据落 memgraph 与 `graph.graph_node`、`graph.graph_edge`、`graph.graph_fingerprint` 三表, 含 13 类 RLS (requirements §48.3, spec §3) | P1 | ⬜ | 设计文档存在: `docs/data-design/ipa-detail/tables/graph_graph_node.md` (T-NEW-001), `graph_graph_edge.md` (T-NEW-002), `graph_graph_fingerprint.md` (T-NEW-003); 无 DDL (`git grep "graph\.graph_" -- db` 无命中); memgraph 无真实客户端 (Phase 3) |
| FR-46-28 | 性能目标: modal 启动 < 1s, cytoscape 绘制 < 500ms, ensure-fresh P95 < 1s (spec §6, REQ-ARCH-002 AC-1) | P2 | ❓ | 未测量; 确认方法: 在可运行的前端构建中以 MSW 启用模式记录时间 |

## 4. 设计

### 4.1 概念模型

- 工作项 (work_item): 当前分析的 Kanban 任务卡, 在图中是 kind 为 `work_item`、`is_current = true` 的节点 (`frontend/src/types/graph.ts:50-51`)。
- 1-hop 邻居: 与 work_item 直接有边相连的节点, 设计上为 11 类 (spec §0.3, ADR §2.1 表)。
- 2-hop 代码侧节点: 从 1-hop 的 `cratemodule` 或 `symbol` 再跳一步得到, 仅限代码侧 (`types/graph.ts:52-53`; spec §2.2.4)。
- 节点类型 (25 类): tenant, project, identity, workspace, work_item, comment, permission_scheme, workflow, change_set, worktree, agent_session, feedback, validation_case, local_runtime, scm_repository, pull_request, design_artifact, context_packet, audit_event, automation_rule, notification, incident_record, integration_webhook, cratemodule, symbol (`frontend/src/types/graph.ts:22-33`)。
- 边类型 (24 类): ASSIGNED_TO 至 FOLLOWING_WORKFLOW (`frontend/src/types/graph.ts:63-69`)。
- 数据源: `GraphSourceKind = "local" | "git"` (`frontend/src/types/graph.ts:106`)。
- 指纹 (fingerprint): 设计为 `sha256(work_item_id + worktree_branch + worktree_sha + source_kind + project_id)` (spec §0.3, ADR §2.2.3); 前端只把它当作字符串使用。

### 4.2 数据模型

| 对象 | 字段 (代码) | 落点 | 状态 |
|---|---|---|---|
| GraphNode | id, kind, label, is_current, hop_level (1 或 2), properties, style? | `frontend/src/types/graph.ts:44-58` | ✅ 类型 |
| GraphEdge | id, kind, source, target, hop_level, properties? | `frontend/src/types/graph.ts:71-80` | ✅ 类型 |
| GraphPayload | work_item_id, tenant_id, fingerprint, nodes, edges, stats {node_count, edge_count, kind_breakdown}, generated_at | `frontend/src/types/graph.ts:85-99` | ✅ 类型 |
| 节点视觉常量 | NODE_STYLE (25 类), CURRENT_WI_STYLE, EDGE_STYLE | `frontend/src/types/graph.ts:158-196` | ✅ (EDGE_STYLE.current 未用, FR-46-12) |
| graph.graph_node (Master, SCD Type 2) | 设计字段: id UUID, tenant_id, kind, label, external_ref, properties JSONB, fingerprint, valid_from/valid_to 等 | 设计: `docs/data-design/ipa-detail/tables/graph_graph_node.md`; DDL 未见 | ⬜ |
| graph.graph_edge (Master, SCD Type 2) | 设计字段: id, tenant_id, kind, source_id, target_id, hop_level, valid_from/valid_to | 设计: `graph_graph_edge.md`; DDL 未见 | ⬜ |
| graph.graph_fingerprint (Transaction, append-only) | 设计字段: id, tenant_id, work_item_id, fingerprint, source, phase, started_at, ended_at, error_message | 设计: `graph_graph_fingerprint.md`; DDL 未见 | ⬜ |
| 示例图 | 固定 13 节点、13 边 (1-hop) 与 17/17 (2-hop) | `frontend/src/mocks/data/graph.ts:31-243` | 🟡 示例数据 |

数据库三类分类按 spec §3.6 与 requirements §48.3 设计为: 两张 Master 表, 一张 Transaction 表, 无 Work 类表。

### 4.3 接口

| 端点 | 请求 | 响应 | 代码位置 | 状态 |
|---|---|---|---|---|
| POST /api/graph/ensure-fresh | `{work_item_id, tenant_id, source}` (`types/graph.ts:108-113`) | 200 `{status: "fresh", graph}`; 202 `{status: "running", retry_after_ms, phase?}`; 400 | 客户端 `ArchGraphModal.tsx:240-248`; MSW `handlers/graph.ts:54-74` | 🟡 MSW; 后端 ⬜ |
| POST /api/graph/cypher | `{work_item_id, tenant_id, max_hop: 1 \| 2}` (`types/graph.ts:130-135`) | 200 GraphPayload; 400 | 客户端 `ArchGraphModal.tsx:256-266` (固定 max_hop: 2); MSW `handlers/graph.ts:77-87` | 🟡 MSW; 后端 ⬜ |
| GET /api/graph/health | 无 | `{memgraph, agent_runtime, last_successful_run, queue_depth}` (`types/graph.ts:141-146`) | MSW `handlers/graph.ts:90-97` (静态返回); 无调用方 | 🟡 MSW; 后端 ⬜ |
| 错误码 401, 403, 404, 409, 429, 500, 503 | spec §2.1.4 与 §2.3.2 | 无 UI 文案; 客户端只读取 HTTP 状态码 (`ArchGraphModal.tsx:245-247`) | — | ⬜ |
| 鉴权头 Authorization: Bearer | basic-design §11.7 | 客户端只设置 Content-Type (`ArchGraphModal.tsx:242`, `:263`) | — | ⬜ |
| Cypher 查询形状 | spec §2.2.4 (1-hop 与 2-hop) | 无 Cypher 执行代码 | — | ⬜ |
| 后端 ports | spec §4: GraphServicePort, LlmAgentWorkerPort, AdvisoryLockPort | 无 crate 定义 | — | ⬜ |
| 网关代理 | 设计依据 ADR-0027 (spec §1.1 Layer 2) | `frontend/next.config.js:48-57` 仅代理 `/api/v1/:path*` 到 star-api-rest; `/api/graph/*` 未代理 | — | ⬜ |

路径裁决: 采用 `/api/graph/*` (spec §2, requirements §48.4, 代码均使用此前缀), 不采用 ADR §2.3.4 的 `/graph/*`。

### 4.4 状态机与关键流程

弹窗 phase 状态 (`ArchGraphModal.tsx:228`):

| 状态 | 进入条件 | 呈现 | 退出 |
|---|---|---|---|
| idle | open 为真时重置 (`:310-318`) | 加载覆盖层 "Initializing..." (`:475`) | ensure-fresh 结果 |
| ensuring | ensure-fresh 处于 pending (`:276-277`) | "Agent is analyzing code context..." (`:473`) | 返回 fresh 或 running 或错误 |
| loading | ensure-fresh 返回 running (`:289-291`) | "Agent is running, loading 1-hop graph..." (`:474`); 同时启用 cypher 查询 (`:269`) | cypher 成功或失败 |
| rendering | ensure-fresh 返回 fresh (`:286-288`) 或 cypher 成功 (`:304-306`) | 画布 (`:515`) 尚未标记 ready | cytoscape 实例化完成 (`:352-371`) |
| ready | cytoscape 实例就绪 (`:371`) | 画布可见; footer 统计 (`:523-538`) | 关闭或刷新 |
| error | ensure-fresh 或 cypher 抛错 (`:280-283`, `:299-302`) | 错误覆盖层与 Retry (`:484-507`) | Retry 回到 idle |

关键流程与注意点:

- 打开时总是触发一次 ensure-fresh (`ArchGraphModal.tsx:310-328`); 关闭时销毁 cytoscape 实例 (`:322-325`)。
- 渲染数据选择: fresh 时取 ensure-fresh 的 graph, 否则取 cypher 的数据 (`:333-335`, `:401-403`)。在默认 mock 路径中, ensure-fresh 返回 1-hop 图, 因此 2-hop 样式在默认路径中不会命中 (FR-46-10)。
- 刷新按钮在重新触发 ensure-fresh 的同时调用 `cypher.refetch()` (`:447`), fresh 路径下 cypher 结果不参与渲染。
- 同一 fingerprint 再次进入 rendering 时, 渲染 effect 直接返回, phase 停留在 rendering 而不进入 ready (`:338-339`); 画布保留旧实例。
- 后端流程 (设计, spec §5.3 与 ADR §2.2.2): 计算 fingerprint → 查历史命中则返回 → 取得 advisory lock → spawn agent → memgraph MERGE → 标记 fresh → 释放锁。全部 ⬜ (FR-46-16, FR-46-17, FR-46-24)。

### 4.5 权限与安全

| 控制点 | 设计要求 | 现状 | 状态 |
|---|---|---|---|
| tenant_id 来源 | 与 ActorContext 一致, 不一致则 403 tenant_mismatch (spec §2.1.4) | 前端从 `workItem.tenant_id` 取值, 缺失时回退到硬编码 `"tenant-physis-corp"` (`ArchGraphModal.tsx:596`) | 🟡 |
| 读取权限 | `read:graph` (spec §7) | 全仓无 `read:graph` 字符串 | ⬜ |
| 鉴权头 | Bearer JWT (basic-design §11.7) | 客户端未发送 Authorization | ⬜ |
| LLM 凭证 | 经 agent.credential_broker 注入, 不进入 agent 本体 (spec §7) | 无 agent 代码 | ⬜ |
| 多租户 RLS | 3 表 13 类策略 (spec §3.5) | 无 DDL | ⬜ |
| PII | 节点 properties 不含 email (spec §7, requirements §48.5) | 示例 fixture 的 identity 节点 properties 含 `email` (`frontend/src/mocks/data/graph.ts:68`, `:76`) | 🟡 示例数据 |
| 示例数据冒充 | 未接通授权数据时显示未接入态 (`AGENTS.md:22`) | 无未接入分支; MSW 启用时显示固定图 | ⬜ |
| 审计 | graph_fingerprint 记录每次执行 (spec §7, REQ-AUDIT-002) | 无 | ⬜ |

### 4.6 依赖关系

| 依赖 | 位置 | 用途 | 状态 |
|---|---|---|---|
| cytoscape ^3.35.0 | `frontend/package.json:30` | 图渲染 | ✅ |
| cytoscape-cose-bilkent ^4.1.0 | `frontend/package.json:31` | cose-bilkent 布局注册 (`ArchGraphModal.tsx:86`, `:91`) | ✅ |
| @types/cytoscape ^3.31.0 | `frontend/package.json:55`; `frontend/package-lock.json:2145` | 类型; 包声明为弃用的存根 | 🟡 |
| cytoscape-cose-bilkent 类型声明 | `frontend/src/types/cytoscape-ext.d.ts:12-16` | ambient module 声明 | ✅ |
| @tanstack/react-query | `ArchGraphModal.tsx:32` | ensure-fresh 与 cypher 请求 | ✅ |
| react-dom createPortal | `ArchGraphModal.tsx:31` | 挂载到 body | ✅ |
| 全局 store identities | `frontend/src/lib/store.ts:147`, `:403`; `ArchGraphModal.tsx:42`, `:597-599` | 解析 assignee 显示名 | ✅ |
| 宿主看板 | `frontend/src/components/board/KanbanCard.tsx`, `KanbanBoard.tsx`; 页面见 F04 | 按钮与打开回调 | 🟡 (宿主未接入回调, FR-46-02) |
| MSW 运行时 | `frontend/src/mocks/client.ts:20`; `frontend/src/instrumentation.ts:9-23` | 仅 `NEXT_PUBLIC_API_MOCKING=enabled` 时启动 | ✅ |
| 后端 REST | `crates/star-api-rest/src` | 无 graph 路由 | ⬜ |
| LLM worker | `crates/star-graph-agent` | 目录不存在 | ⬜ |
| memgraph 客户端 | `crates/arg/src/client/memgraph.rs` | 文件头标注为 stub 模式 (`:9-12`); 属 ARG 域 | ❓ 复用可行性未核实 |

4.7 页面设计: 本功能无独立页面，入口见 F04 任务卡与 IPA 看板.

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 弹窗组件 ArchGraphModal | `frontend/src/components/board/ArchGraphModal.tsx:222-560` | 🟡 | 组件完整 (请求、状态机、渲染、关闭); 无调用方 |
| 挂载 hook useArchGraphTrigger | `ArchGraphModal.tsx:587-616` | 🟡 | 仅出现在文件自身的注释 (`:583-586`) 中, 无 import |
| 看板卡 🕸 按钮 | `frontend/src/components/board/KanbanCard.tsx:172-184` | 🟡 | 仅当 onArchClick 存在时渲染 (`:173`) |
| 看板透传 | `frontend/src/components/board/KanbanBoard.tsx:91-93, 139, 524, 548` | 🟡 | 透传正常, 但上游无调用方 |
| 看板页 /kanban | `frontend/src/app/(app)/kanban/page.tsx:412-433` | ⬜ | KanbanBoard 无 onArchClick |
| 看板页 /board | `frontend/src/app/board/page.tsx:230` 起 | ⬜ | 调用 KanbanBoard 未传 onArchClick (`git grep onArchClick -- frontend/src/app` 无命中) |
| 历史挂载 | `frontend/src/app/(app)/projects/ProjectsClient.tsx` | 🗑 | 742d3777 首次加入, bb6b860d 删除 import、hook、onArchClick 与挂载块 |
| 类型投影 | `frontend/src/types/graph.ts` | ✅ | 25 类节点 (`:22-33`), 24 类边 (`:63-69`), payload (`:85-99`) |
| cytoscape 类型声明 | `frontend/src/types/cytoscape-ext.d.ts` | ✅ | `:12-16` |
| MSW 端点 | `frontend/src/mocks/handlers/graph.ts` | 🟡 | 三个端点, 仅 mock |
| MSW 注册 | `frontend/src/mocks/handlers/index.ts:25, 46, 69` | ✅ | `graphHandlers` 展开进 handlers 数组 |
| MSW 启动门控 | `frontend/src/instrumentation.ts:9-23`; `frontend/src/mocks/client.ts:20` | ✅ | `NEXT_PUBLIC_API_MOCKING === "enabled"` 才启动 |
| 示例 fixture | `frontend/src/mocks/data/graph.ts` | 🟡 | 13+4 节点固定图; `buildMockGraphForWorkItem` (`:262-288`) 只替换当前节点的 id 与 label |
| 空图 fixture | `frontend/src/mocks/data/graph.ts:246-254` (MOCK_GRAPH_EMPTY) | 🟡 | 存在但无引用 |
| 同名卡片副本 | `crates/star-desktop/frontend/src/components/KanbanCard.tsx:28, 53, 77, 137` | 🟡 | Tauri 前端中的按钮代码; 无弹窗; 无调用方传入 onArchClick |
| 后端 REST 图路由 | `crates/star-api-rest/src` | ⬜ | 无 `/api/graph` 路由; graph 相关命中仅为 `group_api/scoped_chat_store.rs:302` 与 `routes/integrations.rs:7, 172, 175` |
| 网关代理 | `frontend/next.config.js:48-57` | ⬜ | 仅 `/api/v1` |
| LLM worker crate | `crates/star-graph-agent` | ⬜ | 目录不存在 |
| graph.* 表 DDL | `db/` | ⬜ | 无 DDL; db 下仅有 worktree_canvas_graph_* 表 (见 §8) |
| memgraph 客户端 | `crates/arg/src/client/memgraph.rs` | ❓ | stub 模式, 复用可行性待核实 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 弹窗无可达入口: 唯一挂载在 bb6b860d 删除, 当前无 onArchClick 调用方 | REQ-ARCH-001 AC-1 与 AC-2 不成立; 用户无法打开功能 | 在 `kanban/page.tsx` 调用 `useArchGraphTrigger`, 渲染 `<ArchGraphModal>`, 把 `arch.open` 传给 KanbanBoard; 增加 Playwright 冒烟 (点击 `kanban-card-arch-*` 后出现 `arch-graph-modal`) |
| 文档与代码状态不一致: ARCH-AGENT-GRAPH-001-REPORT §1 第 11 项与 basic-design §11.6 称 ProjectsClient 已挂载; ADR §5.5、spec 头部、basic-design §11.1 与 §11.9 称 Phase 1 完成 | 状态误判, 后续工作基于错误前提 | 以挂载是否恢复为准更新上述文档; 本文件的状态与代码一致 |
| 默认路径不产生 2-hop 节点: ensure-fresh mock 返回 1-hop 图, 弹窗在 fresh 路径直接渲染它; 2-hop 样式只在 202 分支 (max_hop: 2) 生效 | REQ-ARCH-002 AC-4 在默认路径不可观察; 文案 "loading 1-hop graph" 与请求 max_hop=2 不一致 (`ArchGraphModal.tsx:258, 474`) | 统一 ensure-fresh 与 cypher 的 max_hop 语义, 让 fresh 路径也携带 2-hop 数据, 或按 spec 只保留 1-hop 并移除 2-hop 样式承诺 |
| 后端缺失: star-api-rest 无 graph 路由, `crates/star-graph-agent` 不存在, Next rewrites 不代理 `/api/graph/*` | 去掉 MSW 后端调用失败 (404) | 在 star-api-rest 实装三端点; 在 `next.config.js` 增加代理, 或把路径统一到 `/api/v1/graph/*` |
| 示例数据可能被当作真实架构: MSW 启用时任意 work_item 都显示固定 PHYSIS 邻居图 | 违反 `AGENTS.md:22` 规则, 误导架构判断 | 增加未接入态 (ensure-fresh 返回 404 或 503 时显示未接入, 不渲染 fixture); 把 fixture 限定为测试或显式 demo 标识 |
| 幂等与排他未实现; 前端 202 无轮询 | 并发时重复 agent 运行与 memgraph 写冲突 (REQ-ARCH-003/004) | Phase 2: fingerprint 表 + advisory lock; 前端按 retry_after_ms 轮询, 上限 30 秒 |
| source 默认值硬编码为 local | REQ-ARCH-005 的默认规则不成立 | 读取 ActorContext.local_runtime_id 决定默认值; hook 显式传入 source |
| tenant 回退硬编码 `"tenant-physis-corp"` (`ArchGraphModal.tsx:596`) | 无 tenant 时可能向错误租户请求 (REQ-SEC-001) | 无 tenant_id 时不发请求并显示未授权态; 服务端以 ActorContext 为准 |
| 错误码未区分: UI 只显示 HTTP 状态码 | 401、403、404、409、429、503 的处置无法区分 (spec §2.1.4) | 按错误码渲染文案与操作 |
| 当前边 3px 全亮与非当前 1-hop 边 dashed 未实现; `EDGE_STYLE.current` 无引用 | FR-46-12 与 FR-46-13 缺失 | 按 ADR §2.1 边视觉编码补充样式分支 |
| 尺寸上限 1400×900 与需求 80vw × 80vh 不一致 | 大屏下弹窗小于需求 | 去掉上限, 或修订需求并记录原因 |
| 示例 fixture 含 email 字段 | 与 spec §7 PII 规则不一致 (即使仅 mock) | 删除 `email` 字段, 保留 display_name |
| 无鉴权头、无 read:graph 检查 | 后端实装后无法区分身份 | 客户端附带 Bearer 令牌; 后端校验 `read:graph` |
| handler 注释与实现不符: `handlers/graph.ts:10` 声称有 maybeReal 头, 文件中实际无 | 注释误导后续维护 | 删除注释或补实现 (参照 `handlers/cli.ts:25`) |
| 测试缺口: 无 ArchGraphModal 组件测试; hop 测试只检查 fixture 形状, 不调用 handler 分派 (`graph.handlers.hop.test.ts:43-95`) | 状态机与样式分支无回归保护 | 为 ArchGraphModal 各 phase 写组件测试; 为 handler 的 max_hop 分派写直接调用测试 |
| Playwright 无 arch 用例 (`frontend/e2e` 无命中) | 入口断开时无告警 | 增加冒烟用例 (见第 1 行) |
| Phase 2 与 Phase 3 未实现 | 功能只能展示 mock | 按 ADR §3 实施: star-graph-agent、memgraph 客户端 (复用可行性先核实 `crates/arg/src/client/memgraph.rs`)、三表 DDL 与 RLS |
| health 端点无调用方 | 错误态无诊断信息 (spec §2.3) | 错误态时调用 `/api/graph/health` 并展示 memgraph 与 agent_runtime 状态 |

## 7. 验收与测试

### 7.1 现有测试

| 文件 | 位置 | 断言 | 执行状态 |
|---|---|---|---|
| `frontend/src/components/board/KanbanCard.test.tsx` | `:102-119` | 不传 onArchClick 时无按钮; 传入后按钮存在, aria-label 含 work item key | ❓ 未执行 |
| 同上 | `:122-142` | 点击按钮调用 onArchClick 一次, 不触发 onClick | ❓ 未执行 |
| `frontend/src/mocks/__tests__/graph.test.ts` | `:26-62` (6 个 it) | graphHandlers 数量为 3; 1-hop 为 13 节点、13 边; 恰有一个 is_current 节点; 覆盖 11 类 kind; 2-hop 为 17/17; 无孤立边 | ❓ 未执行 |
| `frontend/src/mocks/__tests__/graph.handlers.hop.test.ts` | `:28-95` (7 个 it) | 2 个 POST 与 1 个 GET; cypher 路径存在; 1-hop 与 2-hop fixture 的计数与边类型 | ❓ 未执行; 不调用 handler 分派 |

### 7.2 已覆盖与未覆盖

- 有测试文本覆盖的行为 (执行状态待确认): 卡片按钮的显示条件与 stopPropagation (FR-46-01, FR-46-03); MSW 端点注册数量与 fixture 形状 (FR-46-05 的 mock 部分, FR-46-26)。
- 无任何测试覆盖: ArchGraphModal 的渲染与 phase 转换; 202 分支; Esc、背景点击与 Retry; cytoscape 样式 (FR-46-07 至 FR-46-13); 错误码; 后端端点; graph.* 表; 端到端入口。
- 历史提交 742d3777 的说明称 vitest 全量通过 320 项, 核对时未重新验证 (❓)。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md` (种子, 已读) | 目标、25 类节点与 24 类边、1-hop 查询形状、节点与边视觉编码、幂等与排他分层、前端契约、3 阶段拆解 | §0 称 "其余节点 20% opacity" 与 §2.3.3 及 REQ-ARCH-002 (仅 hop 2) 不一致; §2.3.4 路径 `/graph/*` 与其余源及代码 `/api/graph/*` 不一致; §5.5 称落地位置为 star-taskgraph 且 "无 commit", 代码位于 frontend cytoscape, 且存在提交 742d3777 | 弱化范围取 §2.3.3 与 REQ-ARCH-002; 路径取 `/api/graph/*`; 落地位置以 §2.3 与代码为准 (frontend) |
| `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` (种子, 已读) | ensure-fresh、cypher、health 契约; 错误码表; 3 表概要与 RLS; 端口 trait; 幂等与排他流程; 性能目标; 安全与 PII 条目; 缺口清单 | 头部称 "Phase 1 完了"; §2.3 称 health 用于弹窗诊断但无调用方; §7 禁止 email 与 fixture 冲突; §1.1 "API Gateway" 与 next.config 仅代理 `/api/v1` 不符 | 设计以 spec 为准; 实现状态以代码为准 |
| `docs/requirements.md` §48 (REQ-ARCH-001~005, §48.3~§48.9) (已读) | 业务需求与 AC; 高亮规则; 三表类别; 非目标 NG-001~006; 阶段计划 | §48.2 REQ-ARCH-005 的默认 source 规则与代码硬编码不符; §48.8 称 Phase 1 完成 | 需求 ID 作为 FR 来源; 默认 source 规则取需求, 偏差记为 🟡 |
| `AGENTS.md:22` (已读) | "未接通授权数据的功能须显示未接入状态，不得以示例数据冒充" | 弹窗无未接入态 | 规则有效; 记为 FR-46-21 与 §6 |
| `docs/basic-design.md` §11 (行 3937 至 4070) (已读) | 3 层架构、节点与边表、三类分类、5 层幂等与 5 层排他、阶段计划、模块配置 (Phase 2) | §11.1 与 §11.9 称 Phase 1 完成; §11.6 路径 `frontend/src/app/projects/ProjectsClient.tsx` 已过时且挂载已删除; §11.7 称 Bearer JWT 而客户端无鉴权头 | 同 spec; 实现状态以代码为准 |
| `docs/reports/ARCH-AGENT-GRAPH-001-REPORT.md` (已读) | 改动矩阵、缺口清单、tsc 与 vitest 记录 | §1 第 11 项称 ProjectsClient 挂载 ArchGraphModal, 与代码不符 (挂载已由 bb6b860d 删除); 测试数字核对时未执行 | 挂载以代码为准 (🗑); 测试数字记为 ❓ |
| `docs/data-design/ipa-detail/tables/graph_graph_node.md` (T-NEW-001), `graph_graph_edge.md` (T-NEW-002), `graph_graph_fingerprint.md` (T-NEW-003) (头部已读) | 表 ID、类别、一次出典 | 无 DDL 文件, 数据库中未实现 | 设计 ✅ 文档存在; 实现 ⬜ |
| `docs/features/F04-task-board.md` (标题已读) | 宿主看板功能 | — | 仅用于 §4.7 的入口引用 |
| `docs/features/F08-agent-graph-canvas.md` (头部已读) | — | 描述 Agent 画布与关系图; 与弹窗无共享代码 | 舍弃, 不同功能 |
| `docs/architecture/2026-08-26-upgrade/adr/0031-context-graph.md`; `adr/0046-langgraph-task-management-operations.md`; `docs/architecture/2026-09-03-langgraph/*`; `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` (仅关键字命中, 未通读) | — | 分别为 context graph、TMO 与 LangGraph 编排、worktree 画布; 命中仅因 "graph" 关键字 | 舍弃, 不引用其状态 |
| `db/migrations/2026-09-16-worktree-canvas-graph-{node,edge,fingerprint}.sql` (grep 命中) | — | 表名为 worktree_canvas_graph_*, 归属 worktree 画布 | 舍弃, 不计入 graph.* 表 |
| `crates/graph-core` (`Cargo.toml:151-152`; action-engine 引用) | — | worktree 画布的 WorktreeId、UserId 等类型, 非架构图模型 | 舍弃 |
| `crates/arg`, `crates/arg-bridge`, `crates/arg-effect` (`Cargo.toml:133-138`); `crates/arg/src/client/memgraph.rs` 文件头 (已读) | — | ARG 域; memgraph 客户端为 stub 模式 | 不计入本功能实现; Phase 3 复用可行性记为 ❓ |
| `crates/star-taskgraph` (`Cargo.toml:100-101`) | — | Phase H.6 的 TaskCard 与 react-flow JSON 渲染, 无 ensure-fresh 或弹窗引用 | 舍弃; 与 ADR §5.5 称谓不符 |
| `crates/star-desktop/frontend` (`src/components/KanbanCard.tsx:28, 53, 77, 137`; `tests/KanbanCard.test.tsx:63-78`) | 同名卡片按钮 | Tauri 前端副本, 无弹窗 | 记为 🟡 副本, 不计入 frontend 主线 |
| `frontend/next.config.js:44-60`; `frontend/src/instrumentation.ts`; `frontend/src/mocks/client.ts` (已读) | 网关 rewrite 范围; MSW 启动门控 | — | 用于 §4.3、§4.6、§5 |
| commit 742d3777 (已读: 文件统计与 pickaxe) | 首次加入 ArchGraphModal、组件、mock 与 ProjectsClient 挂载 | — | 历史证据 |
| commit bb6b860d (已读: diff 与提交说明) | 删除 ProjectsClient 中的 import、hook、onArchClick 与挂载块 | 提交说明只描述看板去重, 未提及弹窗入口 | 🗑 证据 (FR-46-02) |
| 代码: `frontend/src/types/graph.ts`, `frontend/src/mocks/handlers/graph.ts`, `frontend/src/mocks/data/graph.ts`, `frontend/src/components/board/ArchGraphModal.tsx`, `KanbanCard.tsx`, `KanbanBoard.tsx`, `frontend/src/app/(app)/kanban/page.tsx`, `frontend/src/app/board/page.tsx`, `frontend/src/mocks/handlers/index.ts`, `frontend/package.json`, `crates/star-api-rest/src` (grep) | 实现依据 | — | 实现状态以此为准 |
