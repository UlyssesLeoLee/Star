# F08 Agent 关系图、Agent 视图与 Agent 画布

> 状态摘要: 前端已有 Agent 画布 (Canvas 详情页「Agent 画布」标签) 与关系图 (「关系图」标签) 的渲染代码, 但两者的数据源在默认状态下均为空: Zustand store 的 `agentSessions` 初始为空数组, ARG 的 REST 端点未被任何服务挂载, Next.js 侧只为 `/api/arg/edges` 提供占位路由。后端 ARG 数据层 (Memgraph 客户端、EdgeOps 写路径、图查询) 为 stub, 4 维协作影响与同步桥未接入运行时, 多人协作 (A12) 只有骨架与 BFF 占位 handler。

> 代码基线: HEAD 1643559f (the single baseline for every feature doc); 核对方法: path/symbol grep plus file reads of the working tree (本文引用的 F08 文件在会话开始时的 git status 快照中未列为已修改; 未逐文件与 HEAD 做 diff 比对); 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

在 Star 平台的无限画布中呈现三类视图: (a) Agent 画布, 以当前工作 Agent 为中心展示其关联 Worktree 与 Work Item; (b) 关系图, 展示 Agent 之间的 10 类关系边; (c) Agent 成就墙与关系编辑器。关系需要持久化到图数据库 (Memgraph) 并能影响 dispatch、上下文、信任度与产出评估。

### 1.2 范围内 / 范围外

- 范围内: Agent 画布 (AgentCanvasView、AgentFilter、派生 layout/selectors)、用户注释层 (sticky/text/shape/path/connector、undo/redo、框选、橡皮擦)、关系图 (RelationshipView)、ARG REST/WS 契约、ARG 后端 crate (`crates/arg`、`crates/arg-effect`、`crates/arg-bridge`)、Agent 域 REST 契约 (`crates/api/src/agent`)、A12 多人协作的 BFF 契约 (`bff/src/collaboration`) 与 canvas-collab 骨架。
- 范围外: 通用白板画布 (`components/CanvasView.tsx` 的画板元素) 不在本文范围; 真实 LLM 调用、跨组织联邦、关系自动发现 (见 SRS-AGENT-RELATIONSHIP-001 §1.4)。

## 2. 用户与场景

| 角色 | 主要场景 | 对应能力 |
|---|---|---|
| PM / 5 域 Lead | 在 Canvas「Agent 画布」标签查看当前工作 Agent 的 worktree 与任务 | Agent 画布派生视图 (FR-AGV-001/002/003) |
| PM / Dev | 在「关系图」标签查看 Agent 间关系与边权重 | RelationshipView (FR-AG-REL-02) |
| Dev / PM | 在 Agent 画布上加注释、框选、撤销 | 注释层 (FR-AGV-101~108) |
| Dev | 拖拽建边、1-click 模板实例化 | RelationshipEditor、TemplateGallery (已孤立, 见 §5) |
| SRE | 查看 ARG 同步状态与成就 | SyncStatus、AchievementWall (已孤立, 见 §5) |

## 3. 需求

编号按本文统一编排 (FR-08-NN); 来源见 §8。状态以代码为准。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-08-01 | Canvas 详情页提供「Agent 画布」「关系图」标签, 支持 `?tab=agent` / `?tab=relation` 深链 | P0 | ✅ | `frontend/src/app/canvas/[id]/page.tsx:54-72`, `:214-234` |
| FR-08-02 | 将 active Agent 识别为 11 个非终态之一 | P0 | ✅ | `frontend/src/lib/agent-view/selectors.ts:20-37` |
| FR-08-03 | 默认选择 active 中 started_at 最新的 Agent, 无 active 时回退全部 | P0 | ✅ | `frontend/src/lib/agent-view/selectors.ts:46-57`, `:110-112` |
| FR-08-04 | URL `?agent=` 覆盖默认选择并写回 URL | P0 | ✅ | `frontend/src/components/agent-view/AgentViewPage.tsx:92`, `:105-107`, `:189-210` |
| FR-08-05 | 自由散开布局: agent 中心, worktree 右侧 80px, work item 双环 (8 + 12) | P0 | ✅ | `frontend/src/lib/agent-view/layout.ts:26-31`, `:85`, `:113-137` |
| FR-08-06 | 派生 viewport fit-to-content, zoom 夹取 | P0 | ✅ | `frontend/src/lib/agent-view/layout.ts:200-220` |
| FR-08-07 | 节点渲染 (agent / worktree / work_item) 与 bezier 连线 | P0 | 🟡 | `frontend/src/components/agent-view/AgentCanvasView.tsx:424`, `:491`, `:545`, `:552-563`; 无 token/cost/role/avatar 字段, 见 §6 |
| FR-08-08 | pan / zoom / 键盘快捷键 (V/H/+/-/1) | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:153-159`, `:263`, `:367-370`, `:877-884` |
| FR-08-09 | 双击节点跳转详情 (agent / worktree / work item) | P1 | 🟡 | `frontend/src/components/agent-view/AgentCanvasView.tsx:381-388`; `/work-item` 为 legacy 重定向, 见 §6 |
| FR-08-10 | 空状态: 无 agent 时提示并链接 /agents | P1 | ✅ | `frontend/src/components/agent-view/AgentViewPage.tsx:214-245` |
| FR-08-11 | 用户注释: sticky / text / shape / brush / connector 创建 | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:80`, `:1030-1034`, `:1103-1120`, `:637` |
| FR-08-12 | 注释文本内联编辑 (Enter 提交、Esc 取消、Shift+Enter 换行) | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:675-678`, `:719-722` |
| FR-08-13 | 注释拖拽移动 (仅位置变化时提交) | P0 | ✅ | `frontend/src/components/agent-view/AgentViewPage.tsx:379-382`; `AgentCanvasView.tsx:263-267` |
| FR-08-14 | 框选 (marquee) 与 Shift 加选、批量删除 | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:95-98`, `:263-283`, `:334-339`, `:1079-1089`; `AgentViewPage.tsx:383-388` |
| FR-08-15 | 橡皮擦删除注释, 级联删除 free connector | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:969-972`; `AgentViewPage.tsx:365-369` |
| FR-08-16 | undo / redo, 上限 50 步 | P0 | ✅ | `frontend/src/lib/agent-view/annotationHistory.ts:42`, `:72`; `AgentViewPage.tsx:312-342` |
| FR-08-17 | 注释按 agent 维度写入 localStorage, 键前缀 `star-agent-view-annotations:` | P0 | ✅ | `frontend/src/lib/agent-view/annotationApi.ts:14-18`, `:26-63`; `AgentViewPage.tsx:137-161` |
| FR-08-18 | minimap 显示注释信标 (sticky 圆、text 空心、shape 框、path 点、connector 线) | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:1128`, `:1159-1192` |
| FR-08-19 | 关系图渲染 SVG 节点与 10 类关系边 (颜色、权重决定线宽、archived 虚线) | P0 | 🟡 | `frontend/src/app/(app)/agent-relationships/view/RelationshipView.tsx:43-54`, `:151-248`; 数据源见 §5 |
| FR-08-20 | 关系图节点点击显示 NodeDetail | P0 | ✅ | `RelationshipView.tsx:116-124`, `:251-252`; `frontend/src/app/(app)/agent-relationships/view/NodeDetail.tsx:27` |
| FR-08-21 | 关系图 WebSocket 状态显示 (connecting / connected / disconnected / mock) | P1 | 🟡 | `RelationshipView.tsx:137-148`; `frontend/src/lib/arg/ws.ts:184-206`; mock 回退见 §5 |
| FR-08-22 | 拖拽建边 (RelationshipEditor) | P0 | ⬜ | `frontend/src/app/(app)/agent-relationships/editor/RelationshipEditor.tsx:174-193`, `:241`; 组件无 importer, 见 §5 |
| FR-08-23 | 10 类关系选择器 | P0 | 🟡 | `frontend/src/app/(app)/agent-relationships/editor/EdgeTypeSelector.tsx:19-77` (客户端枚举); 仅被孤立的 RelationshipEditor 引用 |
| FR-08-24 | 5 团队模板 1-click 实例化 | P1 | ⬜ | `frontend/src/lib/arg/store.ts:169-230` (静态列表); `frontend/src/app/(app)/agent-relationships/templates/TemplateGallery.tsx` 无 importer; 后端 `crates/api/src/arg/controller.rs:77`, `:346-369` 调用 `TemplateOps::instantiate` 写边, 写路径不可用 |
| FR-08-25 | 成就墙 20 项 + 稀有度/类别筛选 | P1 | 🟡 | 定义: `crates/arg/src/models/achievement.rs:97-398`, `:418`; 前端: `frontend/src/app/(app)/agent-relationships/achievements/AchievementWall.tsx:46-64`, `:148-165`; 无可达入口 (见 §5) |
| FR-08-26 | 成就解锁 SSE 推送 | P2 | ⬜ | `crates/api/src/arg/controller.rs:429-442` 返回空结果; `frontend/src/lib/arg/ws.ts:280-300` 为 mock |
| FR-08-27 | ARG 关系 CRUD REST 契约 (`/api/arg/*`) | P0 | 🟡 | 路由注册 `crates/api/src/arg/controller.rs:67-86`; 未被挂载, 见 §5 |
| FR-08-28 | ARG 写入 Memgraph (1 事务 + audit 双写) | P0 | ⬜ | `crates/arg/src/ops/edge_ops.rs:100-130` 调用 `execute_write`, `crates/arg/src/client/memgraph.rs:143-170` 返回 `ARGError::Other` |
| FR-08-29 | 关系查询 (outgoing / incoming / find_edge) | P0 | ⬜ | `crates/arg/src/ops/edge_ops.rs:201-237` 返回 stub 错误 |
| FR-08-30 | 关系列表读取 | P0 | 🟡 | `crates/arg/src/ops/edge_ops.rs:171-183` 无 v2_sink 时静默返回空 Vec |
| FR-08-31 | 4 维协作影响 (dispatch / 上下文 / 信任 / 产出) | P0 | 🟡 | 逻辑存在于 `crates/arg-effect/src/{dispatch_router,context_injector,trust_engine,output_evaluator}.rs`; 读路径为 stub (`dispatch_router.rs:6`, `trust_engine.rs:12`); 无运行时依赖者 |
| FR-08-32 | ARG 同步桥 (Memgraph ↔ in-process StateGraph) | P1 | ⬜ | `crates/arg-bridge/src/memgraph_listener.rs` 9 处 stub 标记; `crates/arg-bridge/src/canvas_sync_bridge.rs:8-10` 声明 0 业务方法 |
| FR-08-33 | Agent 域 REST 13 端点 (A1-A10) | P0 | ⬜ | `crates/api/src/agent/controller.rs:69-85` 注册路由; 处理器全部返回 Default 占位 (`:124-445`), crate 无依赖者 |
| FR-08-34 | Agent 域业务模块 (handoff / topology / status_sync 等) | P0 | 🟡 | `crates/agent-domain/src/lib.rs:20-45`; 缺 `agent_node` / `context_menu` / `monitor` 模块 (DD 所列), 见 §6 |
| FR-08-35 | A12 多人编辑: 元素增删改、cursor、评论、Follow、3 级权限、audit | P0/P1 | 🟡 | BFF 路由 `bff/src/collaboration/controller.rs:63-76`; handler 含权限与 audit 但返回 Default (`:117-139`); canvas-collab 仅模型 `crates/canvas-collab/src/lib.rs:12-25`; 前端 A12 目录不存在 |
| FR-08-36 | A12 WebSocket 推送 (elements / presence / comments / follow) | P0 | 🟡 | 端点与 hub `bff/src/collaboration/wss_hub.rs:96-161`; hub 有 broadcast 但无生产者调用 `publish` |
| FR-08-37 | 派生只读 (节点不可拖动、不写入 store) | P0 | ✅ | `frontend/src/components/agent-view/AgentCanvasView.tsx:559-563` (仅 mouseDown 选中、双击跳转); `AgentViewPage.tsx:95-97` (只读订阅) |
| FR-08-38 | 派生结果确定性 (同输入同输出) | P0 | ✅ | `frontend/src/lib/agent-view/layout.ts:166-172`; `selectors.ts:104-112`; 单测见 §7 |

## 4. 设计

### 4.1 概念模型

- Agent (`AgentSession`): 前端来自 Zustand store 的 `agentSessions`, 字段包括 id、status、started_at、current_step、worktree_id、agent_kind。
- Worktree、WorkItem: 同样来自 store (`worktrees`、`workItems`)。Agent 与 Worktree 为 1:1 (按 `worktree_id`), Worktree 与 WorkItem 为 1:N (按 `worktree_id`)。
- AgentCanvas (派生, 不入 store): `agentId`、`nodes`、`connectors`、`viewport`、`derivedAt`。类型见 `frontend/src/lib/agent-view/types.ts:25-140`。
- 注释 (annotation): kind 为 `sticky_note | text | shape | path`, 加 free connector; 仅存本地 localStorage (`frontend/src/lib/agent-view/types.ts:44-95`)。
- ARG 图: Agent 节点 (archetype、domain、status、trust_score) 与 10 类有向/无向边 (4 核心 + 6 扩展), 定义于 `crates/arg/src/models/edge.rs:26-50` 与前端 `frontend/src/lib/arg/types.ts:71-96`。模板 5 类 (`crates/arg/src/models/template.rs:16-26`), 成就 20 项 (`crates/arg/src/models/achievement.rs:418`)。

### 4.2 数据模型

| 实体 | 定义位置 | 实现状态 |
|---|---|---|
| Agent / Edge / Template / Achievement (Rust 模型) | `crates/arg/src/models/{agent,edge,template,achievement}.rs` | ✅ 结构定义存在 |
| Memgraph 节点与边 (Cypher schema) | `crates/arg/src/client/memgraph.rs:50-106` 仅有连接参数 | ⬜ 无写入实现 |
| `agents`、`agent_relationship_edges`、`agent_relationship_edges_audit`、`team_template_instances`、`achievements`、`achievement_unlocks`、`relationship_events` (SQL 表) | SRS-AGENT-RELATIONSHIP-001 §7.1; BD-AGENT-RELATIONSHIP-001 §4.4 | ⬜ 未发现迁移或 DDL 文件 (grep 无结果) |
| Trust score 审计 | `crates/arg/src/models/trust_audit.rs`, `crates/arg/src/ops/trust_audit.rs` | 🟡 内存结构, 无持久化 |
| 注释数据 | `frontend/src/lib/agent-view/annotationApi.ts` (localStorage) | ✅ 客户端持久化 |
| A12 元素、cursor、评论、权限、audit 模型 | `crates/canvas-collab/src/models/{element,presence,comment,permission,audit}.rs` | 🟡 仅模型 |

### 4.3 接口

#### 4.3.1 前端 REST 客户端 (ARG)

`frontend/src/lib/arg/api.ts` 声明 13 个 REST 函数, 基址取 `NEXT_PUBLIC_ARG_API_BASE`, 默认同源 (`api.ts:58-67`), 请求统一经 `request()` (`api.ts:122-164`):

| 方法 | 路径 | 前端函数 | 后端注册 | 运行时可达 |
|---|---|---|---|---|
| POST | /api/arg/agents | createAgent (`api.ts:169-171`) | `controller.rs:69` | ⬜ 无 Next 路由, 未挂载 |
| GET | /api/arg/agents | listAgents (`api.ts:173-185`) | `controller.rs:69` | ⬜ |
| GET | /api/arg/agents/{id} | getAgent (`api.ts:187-190`) | `controller.rs:70` | ⬜ |
| PATCH | /api/arg/agents/{id} | updateAgent (`api.ts:192-199`) | `controller.rs:70` | ⬜ |
| POST | /api/arg/edges | createEdge (`api.ts:201-204`) | `controller.rs:71` | 🟡 Next 路由 `frontend/src/app/api/arg/edges/route.ts:18-21` 回显 body 并返回 201, 不持久化 |
| GET | /api/arg/edges | listEdges (`api.ts:206-212`) | `controller.rs:71` | 🟡 Next 路由 `route.ts:14-16` 固定返回 `[]` |
| GET | /api/arg/edges/{id} | getEdge (`api.ts:214-217`) | `controller.rs:72-75` | 🟡 Next 路由 `frontend/src/app/api/arg/edges/[id]/route.ts:13-16` 固定 404 |
| PATCH | /api/arg/edges/{id} | updateEdge (`api.ts:219-226`) | `controller.rs:72-75` | 🟡 Next 路由 `[id]/route.ts:18-22` 回显 body |
| DELETE | /api/arg/edges/{id} | archiveEdge (`api.ts:228-231`) | `controller.rs:72-75` | 🟡 Next 路由 `[id]/route.ts:24-27` 返回 archived:true |
| GET | /api/arg/graph | getGraph (`api.ts:233-239`) | `controller.rs:76` | ⬜ 后端固定空图 (`controller.rs:337-338`) |
| POST | /api/arg/templates/instantiate | instantiateTemplate | `controller.rs:77` | ⬜ |
| GET | /api/arg/achievements | listAchievements | `controller.rs:78` | ⬜ |
| GET | /api/arg/achievements/me | myUnlocks | `controller.rs:79` | ⬜ 后端返回空 (`controller.rs:404-424` 依赖 EventWriter 快照) |
| POST | /api/arg/achievements/evaluate | (无前端函数) | `controller.rs:80-83` | ⬜ 后端返回空结果 (`:429-442`) |
| WS | /ws/arg/events | ArgWebSocketClient (`frontend/src/lib/arg/ws.ts:137`) | `controller.rs:84`, `sse_hub.rs` | ⬜ 无服务挂载; 客户端在 3 次重连失败后切换 mock (`ws.ts:184-186`) |

Next.js 的 `rewrites` 仅代理 `/api/v1/:path*` 到 star-api-rest (`frontend/next.config.js:53-54`), `/api/arg/*` 与 `/ws/arg/*` 无代理。

#### 4.3.2 后端 ARG 路由

`crates/api/src/arg/controller.rs:67-86` 注册 14 个路由 (13 REST + 1 WS)。`crates/api/src/arg/mod.rs:108-110` 导出 `build_router`。全工作区无其他 crate 以 path 依赖 `crates/api` (`grep path = "../api"` 无结果), 也无 bin 调用 `arg_routes`/`build_router`, 故路由未对外提供服务。

#### 4.3.3 Agent 域 REST

`crates/api/src/agent/controller.rs:68-85` 注册 13 个路由 (`/api/v1/agents`、`/{id}` 系列、`handoff`、`topology`、`parent`、`pipeline`、`state`、`worktree`、`work-items`、`start`、`stop`)。所有处理器返回 `Default` 占位响应 (`:124-445`)。`crates/star-api-rest/src/routes/agents.rs:82-84` 的 `GET /api/v1/agents` 是另一套实现, 返回 3 个内存样例常量, 注释称需宿主运行时替换。

#### 4.3.4 A12 BFF 接口

`bff/src/collaboration/controller.rs:63-76`:
- REST: `POST /bff/v1/canvas/elements`、`PATCH /bff/v1/canvas/elements/{id}`、`POST /bff/v1/canvas/elements/{id}/comments`、`GET /bff/v1/canvas/{id}/presence`、`POST /bff/v1/canvas/{id}/follow`。
- WS: `GET /bff/v1/ws/canvas/{elements,presence,comments,follow}` (`wss_hub.rs:96-161`)。

每个 handler 执行 RLS 租户检查与 3 级权限 (View / Comment / Edit), `create_element` 写 audit 事件 (`controller.rs:117-139`), 但返回 `ElementResponse::default()` 且注释明确标注元素持久化待实装 (`:137-138`)。`get_presence` 返回 `PresenceListResponse::default()` (`:208-224`)。前端无调用方 (`grep /bff/v1` 在 frontend/src 无命中)。

#### 4.3.5 页面路由

| 路径 | 文件 | 说明 |
|---|---|---|
| `/canvas/[id]` | `frontend/src/app/canvas/[id]/page.tsx` | 「Agent 画布」「关系图」标签 (`:54`, `:214-234`) |
| `/agent-relationships` | `frontend/src/app/(app)/agent-relationships/page.tsx:17-38` | 占位页, 文本「Agent 拓扑图 (待实装…)」 |
| `/agent-view` | 无目录 (已删除) | 导航 `frontend/src/lib/nav/registry.ts:353-361`、`:390` 指向 `/canvas/canvas-001?tab=agent` |
| `/agent?selected=` | `frontend/src/app/agent/page.tsx` | 双击 agent 节点的目标 (`AgentCanvasView.tsx:383`) |
| `/worktree?selected=` | `frontend/src/app/worktree/page.tsx` | 双击 worktree 节点的目标 (`:385`) |
| `/work-item?selected=` | `frontend/src/app/work-item/page.tsx` 与 `redirects.ts:92-96` | 双击 work item 的目标 (`:387`); 规则重定向到 `/sprint?view=list`, 是否保留 `selected` 未核实 (❓) |

### 4.4 状态机与关键流程

- Agent active 集合 (11 个): queued、spawning、initializing、compiling_context、planning、executing、awaiting_feedback、awaiting_human、awaiting_tool、validating、paused (`selectors.ts:20-33`); 终态 completed/failed/cancelled 不计入。
- 默认选择: active 中按 started_at 降序、id 升序取第一; 全部终态时取全部中最新 (`selectors.ts:46-57`)。
- URL 覆盖: `?agent=` 命中则 `auto=false`; 未命中回退默认并 `auto=true` (`AgentViewPage.tsx:105-107`; 规则见 SRS-AGENT-VIEW-001 §5.1.2)。
- 画布派生流水线: `resolveCurrentAgent` → `pickAgentWorktree` → `pickAgentWorkItems` → `layoutAgentCanvas` → `fitToContentViewport` (`AgentViewPage.tsx:92-107` 为组合入口)。
- 注释历史: `commit` 入 past、清空 future; `undo`/`redo` 交换栈; 同引用提交为 no-op (`annotationHistory.ts:47-75`)。
- 关系边生命周期 (设计, ⬜): created → updated (version+1) → archived 终态 (SRS-AGENT-RELATIONSHIP-001 §5.4.1); 后端 `EdgeOps::archive` 为 stub (`edge_ops.rs:194-198`)。
- 信任分档 5 级 (设计, 🟡): `TrustScoreTier` 类型由 `crates/agent-domain/src/lib.rs:35-38` 自 `models::agent` 导出; 与 arg-effect `trust_engine.rs` 的 +0.01/-0.05 规则在运行时未被调用。

### 4.5 权限与安全

- 前端 ARG 请求每次附带 `X-Tenant-Id` (当 tenantId 存在时, `api.ts:134`)。
- 后端 `ARGPermission::check_tenant` 与角色检查 (`crates/api/src/arg/controller.rs:130-133`、`:352-355`、`:436`) 存在, 但路由未挂载, 不可达。
- A12 BFF: 3 级权限 (`bff/src/collaboration/permission.rs`) 与 RLS 租户检查 (`controller.rs:122`、`:182`、`:212`、`:238`) 存在。
- 密钥: Memgraph 连接参数读取环境变量 (`crates/arg/src/client/memgraph.rs:76-88`), 不打印密码 (注释声明)。
- 注释数据仅在浏览器 localStorage, 无后端鉴权 (`annotationApi.ts`)。
- LLM: `crates/arg/src/llm.rs:55-74` 的 `MockLLMClient` 固定返回 mock 文本, 未接入外部 API。

### 4.6 依赖关系

- 前端 Agent 画布依赖: `@/lib/store` (agentSessions、worktrees、workItems)、`@/lib/agent-view/*`、`StatusPill` (`AgentCanvasView.tsx:` 引用 `@/components/StatusPill`)、`next/navigation` (`AgentViewPage.tsx:51`)。
- 关系图依赖: `@/lib/arg/store` (`RelationshipView.tsx:14`) → `@/lib/arg/api` → `/api/arg/*` 与 `@/lib/arg/ws` → `/ws/arg/events`。
- Canvas 页依赖 `RelationshipView` 与 `AgentViewPage` (`canvas/[id]/page.tsx:46-47`)。
- Rust 依赖: `crates/arg` ← `crates/arg-bridge`、`crates/arg-effect` (`Cargo.toml` 见 `crates/arg-effect/Cargo.toml:15-16`); `crates/api` → `star-arg`、`agent-domain`、`canvas-collab` (`crates/api/Cargo.toml:39-45`); `bff` → `canvas-collab` (`bff/Cargo.toml:98`); `crates/star-api-rest` → `agent-domain` (`crates/star-api-rest/Cargo.toml:109`)。
- 依赖方向注释: `crates/arg-effect` 与 `crates/arg-bridge` 无任何 crate 依赖它们 (运行时孤立)。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| Canvas 详情页标签 | `frontend/src/app/canvas/[id]/page.tsx` | ✅ 已接入 | `:54-72`, `:214-234` |
| Agent 画布容器 | `frontend/src/components/agent-view/AgentViewPage.tsx` | ✅ (canvas-only 变体可达) | `:92-107`, `:214-245`, `:283-296`, `:351-389`; `relationships` 模式无 UI 入口 (`:189-191`, `:290`, `:397-399`) |
| Agent 画布视图 | `frontend/src/components/agent-view/AgentCanvasView.tsx` | ✅ 渲染与交互 | `:66`, `:381-388`, `:847`, `:1128` |
| Agent 筛选下拉 | `frontend/src/components/agent-view/AgentFilter.tsx` | ✅ | `:28-36` |
| 派生 layout | `frontend/src/lib/agent-view/layout.ts` | ✅ 纯函数 | `:26-31`, `:62`, `:85`, `:113-137`, `:200-220` |
| 派生 selectors | `frontend/src/lib/agent-view/selectors.ts` | ✅ 纯函数 | `:20-37`, `:46-57`, `:60`, `:81`, `:98` |
| 注释持久化 | `frontend/src/lib/agent-view/annotationApi.ts` | ✅ localStorage | `:14-63`, `:66-76` |
| 注释历史 | `frontend/src/lib/agent-view/annotationHistory.ts` | ✅ | `:42`, `:47-102` |
| 关系图 | `frontend/src/app/(app)/agent-relationships/view/RelationshipView.tsx` | 🟡 渲染可达, 数据默认为空 | `:43-54`, `:60-104`, `:151-260` |
| 节点详情 | `frontend/src/app/(app)/agent-relationships/view/NodeDetail.tsx` | ✅ (经 RelationshipView 可达) | `RelationshipView.tsx:16`, `:252` |
| 关系编辑器 | `frontend/src/app/(app)/agent-relationships/editor/RelationshipEditor.tsx` | ⬜ 孤立 (无 importer) | `:174-193`, `:241` |
| 边类型选择 | `frontend/src/app/(app)/agent-relationships/editor/EdgeTypeSelector.tsx` | 🟡 孤立 | `:19-77` |
| 模板库 | `frontend/src/app/(app)/agent-relationships/templates/TemplateGallery.tsx` | ⬜ 孤立 | 无 import 命中 |
| 成就墙 | `frontend/src/app/(app)/agent-relationships/achievements/AchievementWall.tsx` | 🟡 孤立 (仅 AgentViewTab 引用, 其入口不可达) | `AgentViewTab.tsx:18-19`, `:75` |
| Agent 视图 Tab 容器 | `frontend/src/app/(app)/agent-relationships/AgentViewTab.tsx` | ⬜ 不可达 | `AgentViewPage.tsx:397-399` |
| ARG 占位页 | `frontend/src/app/(app)/agent-relationships/page.tsx` | 🟡 占位 | `:17-38` |
| ARG store | `frontend/src/lib/arg/store.ts` | 🟡 请求失败只写 error 字段 | `:136-149`, `:152-165`, `:169-230` (静态模板), `:317-344`, `:347-380` |
| ARG REST 客户端 | `frontend/src/lib/arg/api.ts` | 🟡 客户端完整, 服务端缺失 | `:58-67`, `:122-164`, `:166-238` |
| ARG WS 客户端 | `frontend/src/lib/arg/ws.ts` | 🟡 mock 回退默认开启 | `:89`, `:137`, `:184-206`, `:218-245` |
| Next ARG 路由 | `frontend/src/app/api/arg/edges/route.ts`; `[id]/route.ts` | 🟡 占位 | 见 §4.3.1 |
| 导航 | `frontend/src/lib/nav/registry.ts` | 🟡 `agent-view` 硬编码 canvas-001 | `:353-361`, `:365-368`, `:390`, `:767` |
| 旧路由重定向 | `frontend/src/lib/redirects.ts` | ✅ 注释声明 `/agent-view` 已移出固定重定向 | `:21-25`, `:98-102` |
| 占位 E2E | `frontend/e2e/arg-relationships.spec.ts` | 🟡 陈旧 | `:42`, `:62`, `:71-73`, `:76-80` (期望 `agent-relationships-tabs` 与 `relationship-editor`, 源码无这些 testid) |
| ARG 数据层 | `crates/arg/src/` | 🟡 模型可用, I/O 为 stub | `ops/edge_ops.rs:129`, `:171-183`, `:186-236`; `client/memgraph.rs:143-170` |
| ARG 效果层 | `crates/arg-effect/src/` | 🟡 逻辑存在, 读路径 stub, 无依赖者 | `dispatch_router.rs:6`, `:66`; `trust_engine.rs:12`; `achievement_engine/mod.rs:78-107` |
| ARG 桥接层 | `crates/arg-bridge/src/` | ⬜ stub, 无依赖者 | `memgraph_listener.rs` (9 处 stub), `period_flush.rs` (8 处), `canvas_sync_bridge.rs:8-10` |
| ARG REST/WS 路由 | `crates/api/src/arg/controller.rs` | 🟡 注册完整, handler 部分 stub, 未挂载 | `:67-86`, `:325-340` (graph 固定空), `:404-424`, `:429-442` (evaluate 空) |
| Agent REST 路由 | `crates/api/src/agent/controller.rs` | ⬜ 占位 handler, 未挂载 | `:68-85`, `:124-445` |
| 内存 Agent 列表 | `crates/star-api-rest/src/routes/agents.rs` | 🟡 内存样例 | `:28`, `:82-84` |
| Agent 域模型 | `crates/agent-domain/src/` | 🟡 部分模块, 缺 3 个 DD 所列模块 | `lib.rs:20-45`; `models/agent.rs` 44 个测试标记 |
| canvas-collab | `crates/canvas-collab/src/` | ⬜ 仅模型, 0 业务方法 | `lib.rs:1-25` |
| A12 BFF | `bff/src/collaboration/` | 🟡 权限与 audit 实装, 元素与 presence 返回占位, 无生产者 | `controller.rs:117-139`, `:208-224`; `wss_hub.rs:82` (send 无调用) |
| canvas-realtime / canvas-engine | `crates/canvas-realtime/`、`crates/canvas-engine/` | ❓ 未核实 (本次未读取) | 名称出现在 Cargo.toml 依赖清单 |
| Python 自动化测试 | `scripts/automation/arg_e2e_test.py`、`scripts/automation/arg_dispatch_test.py` | 🟡 模拟与静态检查, 未运行 | `arg_e2e_test.py:166`, `:181` (StubEdge/StubAgent); `:225-608` (it_1..it_10); `arg_dispatch_test.py:86-368` (cargo 与源码检查) |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| Agent 画布默认无数据: `agentSessions` 初始为 `[]` (`frontend/src/lib/store.ts:412`), 无 writer 把 seed 注入 store (`store.ts:394-395` 注释声明不再喂 seed); 写入路径仅在 `worktree`、`agent` 等页面读取 | 默认打开「Agent 画布」显示空状态 (`AgentViewPage.tsx:214-227`) | 确定数据来源 (后端 `/api/v1/agents` 或用户显式注入), 并补一条加载 action |
| ARG REST 未挂载, Next 侧只有占位 | 关系图永远无 agent/edge (store 请求失败, `store.ts:143-147`, `:159-163`), 编辑器写入假成功 (`route.ts:18-21`) | 在服务进程挂载 `arg_routes`; Next 路由改为代理或删除 |
| 写入与读取 stub | `EdgeOps::create` 失败 (`edge_ops.rs:129`); `list` 静默空 (`:171-183`) | 实装 Memgraph Bolt 客户端 (`memgraph.rs` 的 stub 路径), 去掉 list 的静默空 |
| WebSocket 回退 mock | 关系图 WS 显示 `mock` 并每 7 秒注入伪事件 (`ws.ts:195-206`) | 生产环境关闭 mock 回退, 或仅开发模式启用 |
| 孤立组件 | RelationshipEditor、TemplateGallery、AchievementWall 无可达入口 | 在关系图标签或独立路由挂载, 或删除 |
| `/agent-relationships` 占位页 | 导航直达占位文本 (`page.tsx:31`) | 渲染关系图或删除导航项 |
| `/agent-view` 历史描述与代码不一致 | SRS/BD 声称 `/agent-view` 307 → `/canvas/canvas-001?tab=agent` (commit 7b129089, 2026-11-14); 当前 `redirects.ts:98-102` 注释说明已移出固定重定向 | 以代码为准; 导航 `registry.ts:358`、`:390` 仍硬编码 canvas-001, 改为动态解析 |
| `/work-item?selected=` 参数未保留 | 双击 work item 落到 `/sprint?view=list` (`redirects.ts:92-96`), 选中态可能丢失 | 核实 Next 重定向查询串行为; 必要时改为直达 `/sprint` 并传 id |
| 节点字段缺失 | Agent 节点仅显示 kind/id/current_step/status (`AgentCanvasView.tsx:524-547`); 缺 token、cost、role、avatar、started_at | 对齐 SRS-CANVAS-AGENT-001 A1.1 字段, 或修订需求 |
| Agent 状态 i18n 缺失 | `StatusPill` 对 agent 状态无字典 (`AgentCanvasView.tsx:545`) | 补充 agent 状态字典 |
| 关系编辑无错误横幅 | `RelationshipEditor.tsx:187-188` 注释说明经 banner 呈现, 实际仅 `console.warn` | 增加错误提示 |
| Memgraph 与 SQL 表未建 | 设计 7 张表无迁移 (§4.2) | 建立 schema 迁移并对齐 W/T/M 分类 |
| 4 维效果与同步桥未接入 LangGraph | 无运行时依赖者 (`crates/arg-effect`、`crates/arg-bridge`); Python 侧仅有测试脚本 | 将 dispatch 路由接入 L0 dispatch_node, 同步桥接入 StateGraph |
| Agent 域 REST 为占位 | 13 端点返回空默认值 (`crates/api/src/agent/controller.rs:124-445`) | 实装 handler 或从路由表移除 |
| agent-domain 缺模块 | DD 所列 `agent_node`、`context_menu`、`monitor` 不存在 (`crates/agent-domain/src/lib.rs:20-30`) | 补模块或更新 DD |
| A12 元素、presence、评论未持久化 | 前端无 A12 目录, BFF handler 返回占位 (`bff/src/collaboration/controller.rs:137-139`, `:208-224`) | 实装 canvas-collab 业务方法并接线 BFF |
| WSS 无生产者 | `wss_hub.rs:82` 的 `publish` 无调用 | 在 REST 写路径后发布事件 |
| E2E 与实现脱节 | `frontend/e2e/arg-relationships.spec.ts:62`, `:76-80` 期望的 testid 在源码中不存在 | 重写或删除该 spec |
| Python 测试未覆盖真实栈 | `arg_e2e_test.py:166`, `:181` 使用 Stub 类 | 改为对真实服务的集成测试 |

## 7. 验收与测试

### 7.1 已存在的测试

| 测试 | 位置 | 数量 | 覆盖 |
|---|---|---|---|
| 派生 layout | `frontend/src/lib/agent-view/layout.test.ts` | 11 | 布局、确定性、fit-to-content |
| 派生 selectors | `frontend/src/lib/agent-view/selectors.test.ts` | 14 | active 判定、默认选择、URL 覆盖 |
| 注释存储 | `frontend/src/lib/agent-view/annotationApi.test.ts` | 6 | 键隔离、降级、清空、列表 |
| 注释历史 | `frontend/src/lib/agent-view/annotationHistory.test.ts` | 9 | commit、undo/redo、上限 50、no-op |
| Agent 画布组件 | `frontend/src/components/agent-view/AgentCanvasView.test.tsx` | 12 | 渲染、注释层、工具、minimap |
| ARG crate | `crates/arg/src/**` 与 `crates/arg/tests/` | 66 | 模型、trust 审计、集成 (`crates/arg/tests/integration.rs` 15) |
| ARG effect | `crates/arg-effect/src/**` 与 `tests/` | 85 | dispatch、trust、output、achievement |
| ARG bridge | `crates/arg-bridge/src/**` 与 `tests/` | 34 | listener、flush、offline |
| Agent 域 | `crates/agent-domain/src/**` | 101 | 各 connector 与 `models/agent.rs` (44) |
| ARG API | `crates/api/src/arg/` | 23 | 路由注册 (`mod.rs` `build_router_registers_14_routes`)、DTO、权限 |
| Agent API | `crates/api/src/agent/` | 17 | 权限与 DTO |
| A12 BFF | `bff/src/collaboration/` | 27 | 权限、wss smoke |

### 7.2 已验证 (代码层面)

- 派生函数为纯函数, 同输入同输出 (`layout.ts:166-172`; `selectors.ts:104-112`)。
- 注释历史上限 50 (`annotationHistory.ts:42`)。
- ARG 路由注册数量为 14 (`crates/api/src/arg/mod.rs` 测试)。

### 7.3 未验证

- 本次未运行任何测试。上表数量为源码中 `#[test]` / `it(` / `test(` 标记计数。
- 未验证 `frontend/e2e/*` 的运行结果; `arg-relationships.spec.ts` 已知与源码不符 (§5)。
- 未验证 Canvas 页在浏览器中的实际渲染。
- 未验证 Memgraph 写入、同步桥、4 维效果在真实环境的行为 (均为 stub)。
- 未验证 `?tab=agent` 深链在导航中的实际落点 (导航硬编码 canvas-001)。
- `canvas-realtime` 与 `canvas-engine` 在 Cargo 依赖清单中出现, 本次未读取其源码, 与 A12 的关系未核实 (❓)。
- `/work-item?selected=` 查询串在 Next 重定向中的保留行为未核实 (❓)。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/requirements/SRS-AGENT-RELATIONSHIP-001.md | ARG 关系类型、Memgraph 图模型、4 维效果、模板、成就、REST/WS 契约 | 声称 Memgraph 与同步桥为设计目标; 代码为 stub; 未发现 SQL DDL | 代码为准; 设计意图保留并标 ⬜/🟡 |
| docs/design/BD-AGENT-RELATIONSHIP-001.md | 5-tier 架构、13 REST 端点、5 个 SQL 表、同步桥、组件清单 | 组件表 (C-1..C-24) 与代码模块不一一对应; 路由、表结构与 SRS 不完全一致 | 以代码为准; 组件映射见 §5 (仅读取前 904 行, 共 1089 行; 其余未读) |
| docs/design/DD-AGENT-RELATIONSHIP-001.md | 类签名、状态机、时序、测试计划 | 类与方法多数为 `todo!`/stub 或未实现; 测试计划 (UT/IT/E2E) 与 `scripts/automation/arg_*_test.py` 名称不一致 | 作为设计参考, 实现状态按代码标注 (仅读取前 1071 行, 共 2285 行; 其余未读) |
| docs/requirements/SRS-AGENT-VIEW-001.md | Agent 视图功能需求、派生规则、注释层 §13 | 顶部纠正注记称 /agent-view 于 2026-11-14 (commit 7b129089) 307 到 canvas-001; 当前 `redirects.ts:98-102` 已移除固定重定向 (2026-11-16 注释) | 以 HEAD 代码为准; 纠正注记中的路由描述视为历史 |
| docs/design/BD-AGENT-VIEW-001.md | 派生层组件、5 view、NFR、§12.5 注释层 | 同 SRS-AGENT-VIEW-001 的路由纠正; 组件已迁入 Canvas 宿主 | 同上; 组件结构与代码一致 (§5) |
| docs/requirements/SRS-CANVAS-AGENT-001.md | A1-A12 共 46 项, 含 A12 多人协作与 Run-scoped 契约 | 多数 A1-A8 与 A11/A12 未实装; §1.5 与 v1.4 文本为目标设计 | 作为需求来源; 状态按代码标注 (仅读取前 326 行, 共 1980 行; 其余未读) |
| docs/design/BD-CANVAS-AGENT-001.md | A1-A12 分层设计、14 张表、23 API、5 WS、§1.5 Run-scoped 契约 | 表与 API 设计未落地; §1.5 明确旧 REST/WSS 路由为历史提案 | 以 §1.5 声明为当前规范; 路由与表以代码为准 (仅读取前 655 行, 共 1750 行; 其余未读) |
| docs/design/DD-CANVAS-AGENT-001.md | 类签名、状态与 WSS 时序、测试计划 | 多数为设计草图; 与 BD 的 crate 布局部分一致 | 作为设计参考; 实现状态按代码标注 (仅读取前 727 行, 共 2601 行; 其余未读) |
| frontend/src/app/canvas/[id]/page.tsx | 标签结构、深链、关系图 tenant 取值 | — | 代码为准 |
| frontend/src/components/agent-view/AgentViewPage.tsx | 容器、模式切换、注释状态、空状态 | `relationships` 模式无入口 | 代码为准 |
| frontend/src/app/(app)/agent-relationships/page.tsx | 占位文本 | 与 e2e spec 期望不符 | 代码为准 |
| frontend/src/lib/redirects.ts | `/agent-view` 重定向移出 (2026-11-16 注释) | 与 SRS 纠正注记冲突 | 以代码为准 |
| frontend/src/app/api/arg/edges/route.ts 与 [id]/route.ts | 占位 GET/POST/PATCH/DELETE 行为 | 注释称 "与 MSW mock 同数据源", 实际为静态回显 | 代码为准; 占位行为列为缺口 |
| frontend/e2e/arg-relationships.spec.ts | 8 个 E2E 用例名称 | 期望 testid 与页面不符 | 视为陈旧, 不作为验收依据 |
| scripts/automation/arg_e2e_test.py | 10 个 IT 计划与 Stub 实现 | 使用 StubEdge/StubAgent, 非真实栈 | 视为模拟, 不作为集成证据 |
| scripts/automation/arg_dispatch_test.py | 18 个 UT 计划 | 实际函数为 cargo 与源码静态检查, 命名与 docstring 不一致 | 视为静态检查 |
| git 提交 7b129089 | 2026-11-14 合并 Agent 中心到 Canvas 的说明 | 后续 2026-11-16 变更移除固定重定向 | 作为历史记录 |

