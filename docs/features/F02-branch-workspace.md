# F02 Worktree 分支工作区与功能标签页

> 状态摘要: 工作区外壳（左侧树、应用页签、右侧栏、对话范围）在三条路由上已有代码，但应用页签有三套集合并存，Lane 集合没有写入路径，聊天发送未接通，群组页在默认部署下只显示本地预览。规范路由 `/projects/.../worktrees/{id}` 的焦点门控已实现，其余应用页签多为占位。
>
> 代码基线: HEAD cdd00a78; 核对方法: path/symbol grep plus file reads; 核对日期 2026-11-16.

## 1. 目标与边界

### 1.1 目标

以 Worktree（本地 checkout）为工作单元，在同一界面内完成三件事：选定 Worktree 与其 Branch；切换该 Lane 的应用页签（任务、Sprint、画布、Workflow、BI、插件）；在右侧栏查看聊天、Worktree 状态与 Team，并选择 WORKTREE 或 GLOBAL 对话范围。授权数据与本地预览必须区分，无授权时不以 seed 数据冒充真实管理事实。

### 1.2 范围内 / 范围外

范围内（工作区外壳与导航）:
- 三条工作区路由的外壳: `/worktree` 中的 BranchWorkspace、`/worktree/[id]/group`（群组页）、`/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]`（规范 LANE 外壳）。
- 左侧 Project → Lane → Worktree → Branch 导航: 群组页内的树，以及全局 Sidebar 中的 ProjectRunTree。
- 应用页签集合、`app` 与 `tab` 参数、旧路由重定向。
- 右侧栏与对话范围选择；Lane/Worktree 导航状态与数据模型。

范围外（由相邻功能承担，本文只描述其在外壳中的挂载点）:
- 任务卡详情、生命周期与评审命令、CLI 会话（子窗口内部与 RunTaskCardsPanel）。
- 画布文档与元素编辑。
- Kanban/Sprint 看板内部实现、Workflow 运行时、聊天传输与 LangGraph。
- Project Worktree Index（AuthenticatedWorktreeIndex）、Git 绑定、Hook 策略编辑、Team 管理动作、Lane 的创建与授权。

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 项目成员 / 开发者 | 选定 Worktree 与 Branch，切换应用页签，切换对话范围 |
| 任务负责人 | 在任务树与 Backlog 中按工作流状态查看任务，打开任务卡子窗口 |
| 评审人 | 通过底部状态筛选查看"评审"状态的任务 |
| 管理员 | 经 Worktree 操作入口进入 Worktree Index（范围外） |

用户故事:
- US-02-1: 开发者在左侧树点选 Branch，工作区显示该 Worktree 的名称、状态与 Lane 绑定。
- US-02-2: 开发者切换应用页签，并可通过 URL 深链回到同一页签。
- US-02-3: 开发者用底部状态筛选同时过滤树与任务列表。
- US-02-4: 开发者在 WORKTREE 与 GLOBAL 之间切换对话范围；GLOBAL 只能选择服务端授权的 Worktree。
- US-02-5: 无授权时，界面明确标识"未接入"或"本地预览"，而不是显示示例数据。

## 3. 需求

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-02-01 | 左侧树按 Project → Lane → Worktree → Branch 分层，Branch 为叶；Lane 缺失时显示"未归类" | P0 | 🟡 部分实现 | 群组页树 `frontend/src/app/worktree/[id]/group/page.tsx:2637-2660`（Branch 下仍挂任务状态分组，叶节点语义不纯）；全局树 `frontend/src/components/run/ProjectRunTree.tsx:121,196,207`；Lane 数据见 FR-02-24 |
| FR-02-02 | 选择 Worktree 后，面包屑、应用区与 checkout 焦点为同一上下文；焦点不匹配时不设置焦点 | P0 | 🟡 | 规范外壳 `frontend/src/components/run/RunWorkspaceShell.tsx`（`scope_mismatch` 校验，焦点门控页签）；群组页面包屑 `page.tsx:2600-2611`；BranchWorkspace 只依赖 query 参数，无焦点校验（`frontend/src/app/worktree/page.tsx:583,618`） |
| FR-02-03 | Branch 功能页签可切换 | P0 | 🟡 | 三套集合并存: BranchWorkspace `frontend/src/components/run/BranchWorkspace.tsx:32-40`（5 项）；群组页 `page.tsx:624-633`（8 项）；规范外壳 `RunWorkspaceShell.tsx:75`（6 项） |
| FR-02-04 | Lane 级应用集: Backlog/Sprint、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins | P0 | 🟡 | 同 FR-02-03；规范外壳仅 Task Cards 与 BI 启用，其余标注"待迁移"并禁用（`RunWorkspaceShell.tsx:217-220`） |
| FR-02-05 | 应用深链 `?app=`；旧别名 task-card/jira/work-items 规范化为 backlog，multica 规范化为 task-flow | P1 | ✅ 已实现 | `page.tsx:604-608`（resolveAppId）；`page.tsx:814-841`（当前应用解析与 `router.replace` 规范化） |
| FR-02-06 | 任务按工作流状态分组（待办、进行中、评审、阻塞、完成、不处理） | P0 | ✅ 已实现（仅授权投影有数据） | 分组定义 `page.tsx:2529-2536`；列表 `page.tsx:2545-2548`；数据仅在 live 模式取得 `page.tsx:1030-1031` |
| FR-02-07 | 底部状态筛选同时作用于导航树与 AI 更新列表 | P0 | 🟡 | 筛选控件 `page.tsx:2702-2710`；筛选结果 `page.tsx:1031`；AI 更新列表无数据源（FR-02-08） |
| FR-02-08 | AI 更新页签，按时间倒序展示更新 | P1 | 🟡 页签存在，更新流未接入 | 页签 `page.tsx:2627-2634`；面板 `page.tsx:2688-2697` 显示"暂无可读取的 AI 回复" |
| FR-02-09 | 任务卡在可关闭的子窗口中打开，导航与 Backlog 共用同一子窗口 | P1 | 🟡 | 群组页子窗口 `page.tsx:2928`（role=dialog），关闭按钮 `page.tsx:2932`；打开参数 `page.tsx:1039-1041`；BranchWorkspace 另用 `WorkItemDetailDrawer`（`BranchWorkspace.tsx:218-225`），为第二套实现 |
| FR-02-10 | Backlog 与 Sprint 应用 | P1 | 🟡 | 群组页 Backlog `page.tsx:2732`（授权投影）；Sprint `page.tsx:2755`，投影不含 Sprint 数据时显示空态（`page.tsx:2761-2775`）；BranchWorkspace 使用 seed 数据的 SprintBoardView（`BranchWorkspace.tsx:126-134`） |
| FR-02-11 | 画布应用，以 Worktree 上下文进入 | P1 | 🟡 | 群组页 Canvas 应用 `page.tsx:3126`；BranchWorkspace 链接硬编码 `canvas-001`（`BranchWorkspace.tsx:139`） |
| FR-02-12 | Workflow / LangGraph 应用 | P1 | 🟡 | 群组页为静态占位，明确不创建 Flow 或运行 Agent（`page.tsx:3258-3270`）；规范外壳禁用（`RunWorkspaceShell.tsx:217-220`）；BranchWorkspace 列出 seed 工作流（`BranchWorkspace.tsx:156-178`） |
| FR-02-13 | BI / Benchmark 应用 | P1 | 🟡 | 群组页说明卡与分析链接 `page.tsx:2782`；规范外壳的 RunBiPanel 显示任务事件表并标注完整性与新鲜度未知（`frontend/src/components/run/RunBiPanel.tsx`）；BranchWorkspace 仅链接 `/analytics`（`BranchWorkspace.tsx:180-193`） |
| FR-02-14 | 插件应用只显示服务端授权 registry 条目，导航不授予执行能力 | P1 | 🟡 | registry 投影 `page.tsx:842-852`；插件视图说明"插件执行界面尚未安装"（`page.tsx:3325-3331`）；规范外壳 Plugin Apps 禁用（`RunWorkspaceShell.tsx:217-220`）；BranchWorkspace 使用 seed integrations（`BranchWorkspace.tsx:195-214`） |
| FR-02-15 | 右侧栏含聊天、Worktree、Team 三页签；Worktree 状态详情默认折叠 | P1 | 🟡 | 默认页签与展开条件 `page.tsx:705-709`（默认 chat；默认展开仅 ≥1536px）；面板 `page.tsx:3338-3394`；默认折叠 `page.tsx:3357-3366`；Team 管理动作未接入（WBS 5.1） |
| FR-02-16 | 对话范围 WORKTREE/GLOBAL；GLOBAL 目标逐项选择，最多 20 个；选择不授予权限 | P1 | 🟡 | 范围选择 `page.tsx:3410-3414`；`changeScope` `page.tsx:1307`；目标读取仅 live 模式 `page.tsx:1005-1026`；目标列表与上限 `page.tsx:3416-3440`；权限说明 `page.tsx:3458-3466` |
| FR-02-17 | 聊天发送与会话记录 | P1 | ⬜ 未实现 | 记录区为空态 `page.tsx:3343-3352`；输入与发送禁用 `page.tsx:3456-3466`；规范外壳同样禁用（`RunWorkspaceShell.tsx:266`）。后端路径 `chat/messages` 已注册（`crates/star-api-rest/src/group_api/scoped_chat.rs:172`），群组页未调用发送路径 |
| FR-02-18 | 数据来源标识：授权投影与本地预览区分，预览不混入真实视图 | P0 | 🟡 | 群组页头部模式徽标 `page.tsx:2612-2618`；Sprint 空态文案 `page.tsx:2761-2775`；规范外壳应用区文案（"服务端尚未开放 Task Cards 读取能力…不会回退到 Worktree 任务或演示数据"）；BranchWorkspace 无任何预览或数据来源标识（整文件无此标记） |
| FR-02-19 | 会话缺失、授权失败或焦点不匹配时 fail closed，不回退 seed | P0 | 🟡 | 投影契约测试 `frontend/src/lib/group/__tests__/groupProjection.test.tsx:247`；规范外壳的错误态与 `scope_mismatch` 校验（`RunWorkspaceShell.tsx`）；BranchWorkspace 直接读取 store 且无门控（`BranchWorkspace.tsx:49-54`），为缺口 |
| FR-02-20 | Lane 绑定状态显式：未绑定远端、绑定信息缺失时须显示 | P1 | 🟡 | 树内"未绑定"徽标 `ProjectRunTree.tsx:196`；群组树 `page.tsx:2640-2643`；BranchWorkspace `BranchWorkspace.tsx:92`。群组页头部文案写死"绑定信息未返回"（`page.tsx:2607`），未读取真实绑定字段；面板同样写死（`page.tsx:3366`） |
| FR-02-21 | 规范路由 `/projects/{project_id}/runs/{engineering_run_id}/worktrees/{worktree_id}` 解析并授权 Project/Run，确认 Worktree 绑定后设置焦点 | P0 | ✅ 已实现 | `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx:1-22`；`RunWorkspaceShell.tsx` 的 `resolveCanonicalRunContext`（获取 RunContext 后校验 project、run、focus，不一致时抛出 `scope_mismatch` 且不设置焦点） |
| FR-02-22 | 旧 Branch 前置路由重定向到规范路由，且不以旧 `branch_id` 授权 | P1 | ✅ 已实现 | `frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx`（`redirect(href)`，忽略 `branch_id`） |
| FR-02-23 | 应用页签支持 `?tab=` 深链 | P2 | ⬜ 未实现 | 规范外壳的 `activeApp` 以内部状态保存，初值固定为 "Task Cards"，解析时重置（`RunWorkspaceShell.tsx:110,121`），未读取查询参数 |
| FR-02-24 | Lane 数据模型与导航状态: Lane 类型、Worktree.lane_id 可选、navStore 选择字段与深链同步 | P0 | 🟡 | 类型 `frontend/src/types/ids.ts:437-447`、`ids.ts:467`；navStore `frontend/src/lib/nav/navStore.ts:55-59,83,245-246,274-277`；store 集合为空且无写入动作 `frontend/src/lib/store.ts:177,434`（§6 缺口 3） |
| FR-02-25 | 侧栏树虚拟化与有界加载 | P0 | ✅ 已实现 | `frontend/src/components/run/ProjectRunTree.tsx:89-90`（VIEWPORT_HEIGHT 288，OVERSCAN 3）、`ProjectRunTree.tsx:301`（窗口切片）；规范外壳 Worktree 选择器有界（`WORKTREE_SELECTION_MAX_ROWS`，`RunWorkspaceShell.tsx`） |
| FR-02-26 | Hook 策略入口位于"高级设置 → Hooks"，不作为 Worktree 树节点 | P2 | ✅ 已实现 | 入口链接 `RunWorkspaceShell.tsx:231`（`/settings/advanced/hooks`）；Hook 页面在 `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` |
| FR-02-27 | 分支切换下拉（`⎇ 分支 ▾`） | P2 | ⬜ 未实现 | 原型顶栏按钮（`docs/prototypes/2 · Branch 工作区 · 统一画布-html/Canvas.dc.html`，顶栏区域）与 `branch-workspace-v1.svg` 顶栏；代码无对应控件，群组页仅以文本显示 Branch（`page.tsx:2607`） |
| FR-02-28 | 顶栏应用页签与"添加页签"入口 | P2 | ⬜ 未实现 | 原型 `Canvas.dc.html` 顶栏导航与"添加页签"按钮；代码中应用页签位于内容区第二行（`page.tsx:2719`），无添加入口 |
| FR-02-29 | 树底部"新建 LANE"入口 | P2 | ⬜ 未实现 | SVG 树底部标注；群组树底部仅有"新建 / 管理 Worktree"链接（`page.tsx:2712`）。i18n 存在"新建 LANE"文案（`frontend/src/lib/i18n/zh-CN.ts:35`），在本外壳中的使用处未核实 |

## 4. 设计

### 4.1 概念模型

- **Project**: 顶层容器，成员角色来自服务端授权（DD-WORKTREE-GROUP-001 §0、§1.1）。
- **Lane**: 对外名称；内部契约仍为 engineering_run（WBS-LANE-WORKSPACE-001 §2）。Lane 承载应用页签的上下文。
- **Worktree**: Lane 管理的本地 checkout，是 Git、CLI、文件操作的目标；`lane_id` 可选（`types/ids.ts:467`）。
- **Branch**: Worktree 当前本地分支的投影叶；远端关联为可选元数据（DD §1.1）。
- **焦点（Focus）**: 规范外壳中，选定 Worktree 并通过授权校验后设置的 checkout 焦点；焦点决定应用页签是否显示（`RunWorkspaceShell.tsx:217`）。
- **应用页签（App）**: Lane 级应用。目标集合采用 WBS 2.1 的五项（Backlog/Sprint、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins），裁决理由见 §8。
- **对话范围（Chat scope）**: WORKTREE 需要焦点；GLOBAL 需要逐项选择授权目标（`page.tsx:3416-3440`）。

### 4.2 数据模型

| 实体 | 字段（代码定义或使用处可见） | 来源 |
|---|---|---|
| Lane | id, tenant_id, project_id, name, git_remote_url?, git_provider?（github/gitlab/gitee）, default_branch?, created_at | `frontend/src/types/ids.ts:437-447` |
| Worktree（相关字段） | id, tenant_id, project_id, lane_id?, name, branch, base_branch, status（WorktreeStatus，17 个取值） | `frontend/src/types/ids.ts:449-467` |
| 导航状态（navStore） | selectedProjectId, selectedLaneId, selectedWorktreeId, selectedBranch；setLaneContext 同步 | `frontend/src/lib/nav/navStore.ts:55-59,83,245-246` |
| 群组投影 | mode（preview / live）、worktree、work_items、refreshError | `frontend/src/lib/group/groupProjection.tsx:112-164` |
| 群组工作项视图 | id, key, title, status, lifecycle_status, review_state, kind, priority, worktree_id；sprint_id 可选 | `page.tsx` 使用处；定义于 `groupProjection.tsx:120-137` |
| 规范 RunContext 信封 | run_context, focus, focus_version, engineering_run, capabilities.task_cards_available, capabilities.execution_admission_available, authorization.project / engineering_run / branch | `RunWorkspaceShell.tsx` 使用处；类型定义于 `frontend/src/lib/run/runDirectoryApi.ts` |
| 规范 RunWorktree | worktree_id, name, branch, machine_state, human_state, archived | `RunWorkspaceShell.tsx` 使用处 |
| 页面查询参数 | 群组页: app, scope, work_item_id, cli, canvas_id, highlight；BranchWorkspace: project_id, worktree_id, branch, lane_id | `page.tsx:704,814-815,1039-1041`；`frontend/src/app/worktree/page.tsx:393-396` |

DD-WORKTREE-GROUP-001 §3 定义了 Index item、RunContext、EntityRef 等 DTO；本文未逐一核对后端持久化表结构（❓）。

### 4.3 接口

**前端路由**

| 路由 | 文件 | 解析与门控 | 状态 |
|---|---|---|---|
| `/worktree?project_id&worktree_id&branch&lane_id` | `frontend/src/app/worktree/page.tsx:575,583,618` | project_id 为 UUID 时改走 AuthenticatedWorktreeIndex（575）；BranchWorkspace 只在非 UUID 路径渲染 | 🟡 |
| `/worktree/[id]/group?app&scope&work_item_id&cli&canvas_id&highlight` | `frontend/src/app/worktree/[id]/group/page.tsx:700-704,814-815,1039-1041` | id 为 worktreeId；数据经投影获取；无 provider 时为预览 | 🟡 |
| `/projects/[project_id]/runs/[engineering_run_id]` | `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/page.tsx` | 无焦点的 Task Cards 页（DD §1.2） | ❓ 路由文件存在，页面内容未核实 |
| `/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]` | 同名 page.tsx（1-22 行）→ `RunWorkspaceShell.tsx` | 获取 RunContext 后校验；无焦点时显示授权 Worktree 选择器 | ✅ |
| 上述路由加 `?tab={app_id}` | — | 未读取查询参数 | ⬜ |
| `/projects/[project_id]/branches/[branch_id]/runs/...` | 同名 page.tsx | 重定向到规范路由 | ✅ |
| `/settings/advanced/hooks?project_id&worktree_id` | `RunWorkspaceShell.tsx:229-231` | 链接 | ✅ |
| `/canvas/worktree-graph`，`/canvas/canvas-001?worktree=` | `BranchWorkspace.tsx:139,145` | 外部链接，目标由画布功能承担 | 🟡 |

**后端 API**（`crates/star-api-rest`，由 `main.rs:62,123` 装配 `build_group_router` 与 PostgreSQL 应用注册表）

| 方法与路径 | 位置 | 授权 | 状态 |
|---|---|---|---|
| GET `/api/v1/worktrees/{worktree_id}/group-context` | `crates/star-api-rest/src/group_api.rs:799`（处理函数 781） | Bearer actor 与 `worktree:read`（781-791） | 🟡 代码存在；部署门状态见 §6 |
| GET `/api/v1/worktrees/{worktree_id}/group-apps` | `crates/star-api-rest/src/group_api/group_apps.rs:268`（`list_group_apps`） | `worktree:read`（276-279） | 🟡 代码存在 |
| GET `/api/v1/worktrees/{worktree_id}/chat/targets` | `crates/star-api-rest/src/group_api/scoped_chat.rs:168` | scoped chat 授权 | 🟡 代码存在 |
| `/api/v1/worktrees/{worktree_id}/chat/messages` | `crates/star-api-rest/src/group_api/scoped_chat.rs:172` | scoped chat 授权 | 🟡 路径已注册，群组页未接入发送 |

**前端客户端**（`frontend/src/lib/group/worktreeGroupApi.ts`）: group-context 调用 `:860`，group-apps `:864`，chat/messages `:1143`，chat/targets `:1162`。客户端代码存在，但默认应用中没有 provider 注入（见 §4.6）。

### 4.4 状态机与关键流程

**A. 规范外壳焦点解析**（`RunWorkspaceShell.tsx`）
1. 初始: 无 load，显示"正在验证 Project 与 LANE 授权…"。
2. 调用 `getRunContext(runId, worktreeId)`；校验 context 的 project_id、engineering_run_id，以及 focus 与路径中 worktreeId 是否一致。不一致则进入错误态（`scope_mismatch`），不设置焦点。
3. 有焦点: `setFocus`，聊天范围为 WORKTREE，显示应用页签与 RunContextDetails。
4. 无焦点: 聊天范围为 GLOBAL；加载授权 Worktree 选择器（分页，单选择器上限 `WORKTREE_SELECTION_MAX_ROWS`）。
5. 错误态提供"重试授权上下文"。

**B. 群组投影**（`frontend/src/lib/group/groupProjection.tsx:166-205`）
- 无 client → `preview`。
- 有 client → 请求键包含 client 身份、worktreeId、refreshKey、canvasId → live（带可选 refreshError）。
- 会话切换时丢弃旧投影（`groupProjection.test.tsx:84` 覆盖）。

**C. 群组页应用解析**（`page.tsx:814-841`）
- 当前应用 = `resolveAppId(requestedApp)`，未知或失效值回落到 backlog（`page.tsx:832`）。
- 别名（task-card、jira、work-items、multica）经 `router.replace` 规范化。

**D. 对话范围**
- 初始 scope 由 URL `scope` 决定（`page.tsx:704`）；`changeScope` 更新 URL（`page.tsx:1307`）。
- GLOBAL 目标列表仅在 live 模式读取（`page.tsx:1005-1026`）。

### 4.5 权限与安全

- 客户端不做授权判定。`WorktreeGroupApiClient` 每个请求取用宿主提供的 token；无 token 时抛出 `session_required` 且不发起请求（DD §1.4；`groupProjection.test.tsx:247`）。
- 服务端对 group-context 与 group-apps 要求 Bearer actor 与 `worktree:read`（`group_api.rs:781-791`；`group_apps.rs:276-279`）。
- 规范外壳焦点不匹配时拒绝设置焦点（`RunWorkspaceShell.tsx` 的 `scope_mismatch`）。
- 插件入口只表示 registry 中存在该条目，不授予能力；每次运行须由后端 capability gateway 重新授权（`page.tsx:3325-3331` 文案）。
- GLOBAL 目标的选择不授予权限，提交时服务端逐个授权（`page.tsx:3458-3466` 文案）。
- URL 参数（app、scope、tab）不授予任何能力。
- 旧 `branch_id` 参数不参与授权（旧路由重定向）。
- Hook 策略入口仅经高级设置（`RunWorkspaceShell.tsx:231`）。

### 4.6 依赖关系

| 依赖 | 方向 | 位置 |
|---|---|---|
| 全局 Sidebar（ProjectRunTree） | 每个页面都挂载 | `frontend/src/app/layout.tsx:78`；`frontend/src/components/Sidebar.tsx:210` |
| AppShell 与全局 RightChatRail | 群组页被 AppShell 包裹，全局右侧聊天栏同时挂载 | `frontend/src/app/worktree/layout.tsx`；`frontend/src/components/AppShell.tsx:77` |
| 宿主会话与 WorktreeGroupApiProvider | Providers 仅在 session 存在时挂载 provider | `frontend/src/app/layout.tsx:75`（未传 session）；`frontend/src/app/providers.tsx:51-59` |
| 画布 | 群组页 Canvas 应用；BranchWorkspace 链接 | `page.tsx:3126`；`BranchWorkspace.tsx:139` |
| 任务卡 | 群组页子窗口；RunTaskCardsPanel；WorkItemDetailDrawer | `page.tsx:2928`；`RunWorkspaceShell.tsx`；`BranchWorkspace.tsx:30` |
| Sprint | SprintBoardView | `BranchWorkspace.tsx:29` |
| 设置、分析、Workflow、集成 | 外部链接 | `RunWorkspaceShell.tsx:231`；`BranchWorkspace.tsx:145,160,186,210` |
| 状态存储 | useStore（worktrees、workItems、lanes 等）；navStore；chatStore（`frontend/src/lib/chat/chatStore.ts:99,146,283`，供全局聊天栏使用，群组页使用自有范围状态） | `frontend/src/lib/store.ts`；`frontend/src/lib/nav/navStore.ts` |
| 后端 | star-api-rest group 路由，PostgreSQL resolver 与应用注册表 | `crates/star-api-rest/src/main.rs:62,123` |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| BranchWorkspace（Lane 应用面板，seed 数据） | `frontend/src/components/run/BranchWorkspace.tsx` | 🟡 | 5 页签 `:32-40`；数据来自 store `:49-54`；无数据来源标识；`stats` 计算 `:68-74` 未被渲染 |
| `/worktree` 页挂载 | `frontend/src/app/worktree/page.tsx` | 🟡 | 分流 `:575`；挂载 `:583`、`:618` |
| 群组页 GroupWorkspacePage | `frontend/src/app/worktree/[id]/group/page.tsx`（3522 行） | 🟡 | 头部与树 `:2600-2660`；应用 `:2732-3280`；右栏 `:3338-3394`；聊天栏 `:3400-3466` |
| 群组页路由布局 | `frontend/src/app/worktree/layout.tsx` | ✅ | 包裹 AppShell |
| 规范 LANE 外壳 | `frontend/src/components/run/RunWorkspaceShell.tsx`（286 行） | 🟡 | 焦点门控与选择器；页签 `:75,217-220`；聊天范围 `:262-268` |
| 规范路由页 | `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx` | ✅ | 1-22 行，渲染 RunWorkspaceShell |
| 旧路由重定向 | `frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx` | ✅ | `redirect(href)` |
| Run BI 面板 | `frontend/src/components/run/RunBiPanel.tsx`（76 行） | 🟡 | 任务事件表，标注完整性与新鲜度未知 |
| 全局项目树 | `frontend/src/components/run/ProjectRunTree.tsx`（536 行） | 🟡 | 虚拟化 `:89-90,301`；两套链接模式 `:207`（/worktree 路径）与 `:511`（规范路径）；演示目录回退 `:153-154` |
| Sidebar 挂载 | `frontend/src/components/Sidebar.tsx:210`；`frontend/src/app/layout.tsx:78` | ✅ | 每页挂载 |
| 导航状态 | `frontend/src/lib/nav/navStore.ts` | ✅ | `:55-59,83,245-246,274-277` |
| Lane 类型与 Worktree.lane_id | `frontend/src/types/ids.ts` | ✅ | `:437-447,467` |
| Lane 集合 | `frontend/src/lib/store.ts` | 🟡 | `:177,434`，空集合，无写入动作 |
| 群组 API 客户端 | `frontend/src/lib/group/worktreeGroupApi.ts` | 🟡 | `:860,864,1143,1162`；默认应用中无 provider |
| 群组投影 hook 与 provider | `frontend/src/lib/group/groupProjection.tsx` | ✅ 代码 / 🟡 默认未装配 | Context `:58-60`；hook `:104-106`、`:166-205` |
| Provider 挂载 | `frontend/src/app/providers.tsx:31,51-59`；`frontend/src/app/layout.tsx:75` | 🟡 | 默认未传 session，provider 不挂载 |
| 全局右侧聊天栏 | `frontend/src/components/AppShell.tsx:77`；`frontend/src/components/chat/RightChatRail.tsx` | 🟡 | 与群组页右栏并存 |
| 后端群组路由 | `crates/star-api-rest/src/group_api.rs:799`；`group_api/group_apps.rs:268`；`group_api/scoped_chat.rs:168,172`；`crates/star-api-rest/src/main.rs:62,123` | 🟡 | 代码存在；部署门状态见 §6 |
| 外部链接目标 | `/nema`、`/sandbox`、`/prototype`、`/workflow`、`/analytics`、`/canvas/worktree-graph` | ✅ 路由目录存在 | `frontend/src/app/(app)/` 下对应目录 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| 1. 宿主会话未装配，WorktreeGroupApiProvider 默认不挂载（`app/layout.tsx:75`） | 群组页与 Index 默认只显示预览，授权视图不可用 | 接入宿主 `runDirectorySession`，并按 DD §1.4 验证 live 路径 |
| 2. 三套应用页签集合并存（5 / 6 / 8 项），与原型的顶栏集合（任务/画布/协同/沙盒/原型）不一致 | 同一 Worktree 在不同路由看到不同功能，导航心智分裂 | 按 §8 裁决统一为一套；群组页的任务生命周期与 V 字模型视图并入 Backlog 的视图选项；其余路由重定向或隐藏 |
| 3. Lane 集合无写入动作（`store.ts:177,434`） | 树中 Lane 名称恒为"未归类"，Lane 绑定徽标没有真实来源 | 经 runDirectory API 加载 Lane 列表并写入 store；或直接以投影中的 Lane 身份渲染 |
| 4. BranchWorkspace 直接读 seed store，且无来源标识（`UI-FIX-WORKSPACE` 简报亦记录该数据边界问题） | 示例数据可能被误认为真实管理事实 | 无授权时不渲染 seed；增加预览标识；与 FR-02-18、FR-02-19 对齐 |
| 5. 绑定信息写死为"绑定信息未返回"（`page.tsx:2607,3366`） | 无法反映真实绑定状态 | 读取投影中的绑定字段，仅在缺失时显示"未返回" |
| 6. 聊天发送未接通；群组页与全局 RightChatRail 并存（`AppShell.tsx:77`） | 同一路由出现两个聊天面，且发送不可用 | 为每条路由确定单一聊天面；接通 `chat/messages`（`scoped_chat.rs:172`） |
| 7. 规范外壳缺 `?tab=` 深链 | 无法分享具体应用页签 | 读取并校验 tab 查询参数（DD §1.2） |
| 8. AI 更新与 Sprint 无数据源（投影不含 Sprint，`page.tsx:2761-2775`） | 两个页签长期为空态 | 扩展 GroupContext 投影，加入 Sprint 标识与更新事件 |
| 9. 画布链接硬编码 `canvas-001`（`BranchWorkspace.tsx:139`） | 其它 Worktree 的画布入口指向错误对象 | 按 Worktree 解析画布列表后再链接 |
| 10. 分支切换下拉与顶栏页签（原型）未实现 | 与原型不一致 | §8 裁决后实现或从设计中删除 |
| 11. 双树（全局 Sidebar 的 ProjectRunTree 与群组页树）并存；ProjectRunTree 两套链接（`:207` 与 `:511`） | 同一路由两套导航树；链接模式不统一 | 决定保留位置；将 `:207` 收敛为规范 href |
| 12. 无群组页与 BranchWorkspace 的组件测试 | 页签、门控与范围切换无回归保护 | 见 §7.3 |
| 13. BranchWorkspace 的 `stats` 计算未渲染（`:68-74`） | 死代码 | 渲染 BI 摘要或删除 |
| 14. DD 与 ui-3pane 仍使用 Engineering Run/Run 命名 | 文档与界面术语不一致 | 以 §8 裁决的 Lane 术语统一文档 |
| 15. 后端群组路由的部署状态未核实（❓） | 代码存在但能否在目标环境提供服务不确定 | 核对目标数据库与运行角色，执行 group-context、group-apps 的端到端调用 |

## 7. 验收与测试

### 7.1 现有测试

| 文件 | 用例（节选） | 覆盖 |
|---|---|---|
| `frontend/src/components/run/RunWorkspaceShell.test.tsx:153-260` | 6 项: 未聚焦时只显示目录选择且不挂载应用；选择器分页与搜索；拒绝重复或超限页；聚焦后显示应用 | 焦点门控与分页上限（仅规范外壳） |
| `frontend/src/components/run/ProjectRunTree.test.tsx:103-146` | 3 项: 选择项目清除焦点；展开不选中；键盘展开 | 树交互 |
| `frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx:99-132` | 2 项: 核心区导航；Worktree 行区分生命周期与绑定 | 侧栏 |
| `frontend/src/lib/group/__tests__/groupProjection.test.tsx:24-260` | 6 项: 无 provider 为 preview；token 注入；陈旧 401 忽略；Canvas poller；缺会话 fail closed；拒绝跨 Project 的 GroupContext | 投影契约 |
| `frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts` | 存在 | 客户端（❓ 未逐条核对） |
| `frontend/src/app/worktree/AuthorizedProjectSelector.test.tsx:38-75` | 3 项 | 目录选择器 |
| `frontend/src/app/worktree/GitBindingPanel.test.tsx`，`ProjectWorktreeLifecycleControls.test.tsx` | 存在 | 相邻功能，未读 |
| `crates/star-api-rest/src/group_api/worktrees.rs` | 16 个测试属性（grep 计数） | 相邻 worktree 路由 |
| `crates/star-api-rest/src/group_api/schedule_rules.rs:1838`，`task_runs.rs:777` | 构造 `build_group_router` 的集成测试 | 群组路由装配，不针对 group-apps |

group_api.rs 与 group_apps.rs 中直接测试计数为 0。frontend/e2e 中未发现引用 `group-app-navigation`、`lane-app-tab`、`right-chat-panel`、`group-chat-scope` 等选择器的规格。

### 7.2 已验证与未验证

- **已验证（代码阅读与单测存在）**: 规范外壳的焦点门控与分页上限；投影的 preview 与 fail-closed 契约；树的虚拟化与选择清除焦点。
- **未验证**: 群组页与 BranchWorkspace 的页签渲染和切换；`?app=` 规范化在浏览器中的行为；右栏默认状态与范围切换；聊天禁用状态；live 模式下的渲染；后端 group-apps 与 group-context 在目标库的运行结果；WBS 8.1 声称的 tsc 与 vitest 通过（本文未重跑）。

### 7.3 建议补充的验收用例（⬜ 未实现）

- 对三条路由断言页签集合、`aria-selected` 状态与 `app` 规范化。
- 无宿主会话时，BranchWorkspace 与群组页不渲染 seed 数据，并显示未接入或预览标识。
- 未知 `app`、`tab` 值回落到默认页签。
- GLOBAL 目标在 live 模式下最多 20 个，选择不触发写操作。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/prototypes/2 · Branch 工作区 · 统一画布-html/README.md` | 说明该目录为参考 mockup，取值自 `Canvas.dc.html` 的内联样式 | 无设计值 | 仅作入口说明 |
| `docs/prototypes/2 · Branch 工作区 · 统一画布-html/Canvas.dc.html`（读顶栏 `:20-40` 与标签统计） | 顶栏为 任务/画布/协同/沙盒/原型 五项，协同、沙盒、原型指向独立页面；含"添加页签"按钮 | 与 SVG v1（九项）、WBS（五项 Lane 应用）、代码（5/6/8 项）均不一致 | 顶栏五项作为待决策项（FR-02-27/28）；协同、沙盒、原型保留为全局路由（目录已存在） |
| `docs/frontend/design/mockups/branch-workspace-v1.svg`（整读） | 布局：左侧树止于 Branch；右栏 Worktree 状态默认折叠；WORKTREE/GLOBAL 切换；可关闭子窗口；标注 1–6（按位置对应树叶、顶栏、顶栏末端、子窗口关闭、右栏状态行、范围切换；SVG 未附说明文字） | 应用页签为九项；树底部有"新建 LANE"；任务列为四列（待办、进行中、评审、完成），代码为六列 | 采用布局与区域划分；不采用九项页签；列集合以代码为准（六列，待决策） |
| `docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md`（整读） | Lane 层级、Phase 1–8 的实施状态与提交记录、Lane 级五项应用、Group 双页签、Worktree 状态默认折叠、Lane 本地实体 | 1.2 称 Lane "本地可建/授权/列出"，代码无写入动作；8.1 的 gate 声明未重跑 | 采用其五项应用集作为 Worktree 内应用目标；状态以代码为准 |
| `docs/design/DD-WORKTREE-GROUP-001.md`（读 §0、§1.1–§1.5、版本头；§4.2、§5、§8.2 仅读标题与摘要，未读全文） | 完成判定；产品树；规范路由表；provider 与 Bearer 契约；BFF 契约；当前 UI 与目标状态表；生产门状态 | 使用 Engineering Run 命名；App 列表为七项（含 Inbox、Task Card Index、Relation、Planning）；`?tab=` 路由；版本头称生产门未关闭 | 路由与契约作为目标设计采用；命名以 WBS（更近期的 Lane 命名）为准；App 列表超出 WBS 的部分列为待决策 |
| `docs/frontend/design/ui-3pane-arch.md`（读 §0、§0.1、§0.2、§1.1–§1.2 片段，其余 grep） | 导航层级 Project → Lane → Worktree → Branch；右侧抽屉思路 | §0.2 称 Branch 功能集为顶栏五项（任务/画布/协同/沙盒/原型），与 WBS 2.1 不同；正文 §1 已被 §0.2 标注为现状以外 | 仅采用导航层级；Branch 功能集以 WBS 为准（见下条裁决） |
| 裁决（顶栏功能集） | — | 原型与 ui-3pane §0.2 同为五项顶栏；WBS 2.1 为五项 Lane 应用 | Worktree 内应用集采用 WBS 2.1 五项；理由：WBS 是逐阶段实施并带提交证据的清单，且 ui-3pane 的五项是全局路由而非 Worktree 内应用。该裁决列入待确认项 |
| `docs/prototypes/INDEX.md`（整读） | 一功能一权威 SRS 的规则；映射表 | 映射表无 "2 · Branch 工作区" 行，故本功能没有权威 SRS | 记为缺口；本文作为功能文档的替代，待确认是否新增 SRS |
| `docs/worktree-group-guard.md`（读标题） | 无 | 讲的是 Git worktree 分组的开发守门（守门 #32–#34），与 UI 功能无关 | 排除 |
| `docs/briefs/UI-FIX-WORKSPACE.md`（整读，13 行） | 要求为 BranchWorkspace 与画布的数据边界故障准备补丁 | 仅为简报，无设计内容 | 作为 FR-02-18/19 与缺口 4 的佐证 |
| `docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md` | 无 | 未读（2350 行） | 未读，见 open_items |
| `docs/frontend/design/ui-redesign-multica-style.md`、`ui-detailed-design.md` | 无 | 未读，仅 grep 命中 | 未读，见 open_items |
| `docs/requirements/SRS-KANBAN-001.md`、`SRS-PROTOTYPE-001.md`、`SRS-SANDBOX-001.md`、`docs/requirements.md` | 无 | 未读，仅 grep 命中 | 未读，见 open_items |
| `docs/refactor/ui-audit-20261113.md`、`docs/arch-live/02-navigation-model.html`、`docs/reports/WBS-UI1-PROJECT-RUN-SELECTION-20261022.md`、`docs/prototypes/0 · 页面合并图-html/Sitemap.dc.html` | 无 | 未读，仅 grep 命中 | 未读，见 open_items |
| `docs/architecture/file-session-management/*`、`docs/architecture/2026-08-26-upgrade/adr/0052-file-session-management.md` | 无 | 未读，仅 grep 命中，主题为会话管理 | 排除（相邻主题） |
| 代码（组合）: BranchWorkspace.tsx、RunWorkspaceShell.tsx、规范页与旧重定向页、worktree/layout.tsx（整读）；群组页、worktree/page.tsx、groupProjection.tsx、providers.tsx、layout.tsx、types/ids.ts、navStore.ts、store.ts、AppShell.tsx、RightChatRail、ProjectRunTree、Sidebar、star-api-rest 群组路由（分段读取或 grep） | 本文 §3–§5 的全部实现证据 | 无 | 作为状态的最终依据 |
