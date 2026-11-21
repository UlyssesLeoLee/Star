# F02 Worktree 分支工作区与功能标签页

> 状态摘要: 工作区外壳（左侧 Project / LANE / Worktree / Branch 树、LANE 应用页签、右侧栏与 WORKTREE / GLOBAL 对话范围）已在 `/worktree`、`/worktree/[id]/group` 与 `/projects/.../worktrees/[worktree_id]` 三条路由及全局 Sidebar 上有代码；应用页签存在 5、8、6 项三套集合并存，Lane 集合没有写入路径，聊天发送未接通。默认部署未挂载宿主会话 provider，群组页停留在预览态，读取的客户端 store 初始为空。

> 代码基线: 637ed19d; 核对方法: 对基线提交做 git show、git grep、git ls-tree 只读检索（路径与符号），逐条证据回读行号; 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

以 Worktree（本地 checkout）为工作单元，在同一外壳内完成三件事：

- 选定 Lane、Worktree 与 Branch，使导航、应用区、面包屑与 checkout 焦点处于同一授权上下文；
- 在 LANE 级应用页签（Backlog / Sprint、画布、Workflow / LangGraph、BI / Benchmark、Plugins）之间切换；
- 在右侧栏查看聊天、Worktree 状态与 Team，并在 WORKTREE 与 GLOBAL 对话范围之间选择。

授权数据与本地预览必须以可见标识区分；未授权或未接入时，界面不以示例数据冒充真实管理事实。

### 1.2 范围内 / 范围外

范围内:

- 三条工作区路由的外壳，以及旧 Branch 前置路由的重定向。
- 左侧树：群组页导航树与全局 Sidebar 中的 ProjectRunTree。
- LANE 应用页签集合、`app` 查询参数与页签深链。
- 右侧栏（群组页右侧面板与全局 RightChatRail）与对话范围选择。
- 顶栏功能页签（AppHeader primary tabs）中与 Branch 功能相关的部分。
- Lane 与 Worktree 导航状态的数据模型。

范围外（本文只描述挂载点）:

- 任务卡的生命周期、评审与 CLI 会话内部实现。
- 画布文档与元素编辑（见 `docs/features/F06-unified-canvas.md`）。
- Kanban / Sprint 看板内部实现（见 `docs/features/F04-task-board.md`）。
- Workflow 运行时、Agent 运行与聊天传输。
- Project Worktree Index（`/worktree` 中 UUID 形式 project_id 的分支）、Git 绑定、Hook 策略编辑页、Lane 创建与设置页。
- `/worktree/chat` 聊天页。

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 项目成员 / 开发者 | 选定 Worktree 与 Branch，切换 LANE 应用页签，切换对话范围 |
| 任务负责人 | 在导航树与 Backlog 中按工作流状态查看任务，打开任务子窗口 |
| 评审人 | 通过底部状态筛选查看"评审"状态的任务 |
| 管理员 | 经 Worktree 操作入口进入 Worktree Index（范围外） |

用户故事:

- US-02-1 开发者在导航树点选 Branch，工作区显示该 Worktree 的名称、状态与 Lane 绑定。
- US-02-2 开发者切换 LANE 应用页签，并可经 URL 回到同一页签。
- US-02-3 开发者用底部状态筛选同时过滤导航树与 AI 更新。
- US-02-4 开发者在 WORKTREE 与 GLOBAL 之间切换；GLOBAL 只能选择已授权的 Worktree。
- US-02-5 无授权时，界面明确标识"未接入"或"本地预览"。

## 3. 需求

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-02-01 | 导航按 Project → Lane → Worktree → Branch 分层；Branch 下挂按工作流状态分组的任务；无 Lane 的 Worktree 归入默认分组 | P0 | 🟡 | 全局 live 树 `frontend/src/components/run/ProjectRunTree.tsx:503-534`（Branch 为非链接叶，:533-534）；fallback 树仅以文本"Worktree · {branch}"显示 Branch，无独立 Branch 叶 `ProjectRunTree.tsx:240-249`；群组树 `frontend/src/app/worktree/[id]/group/page.tsx:2636-2686`，任务挂在 Branch 下 `page.tsx:2662-2680` |
| FR-02-02 | 选择 Worktree 后，导航、应用区、面包屑与 checkout 焦点使用同一授权上下文；焦点与授权 RunContext 不一致时不设置焦点 | P0 | 🟡 | 规范外壳校验 `frontend/src/components/run/RunWorkspaceShell.tsx:127-132`；URL 参数同步到导航状态 `frontend/src/app/worktree/page.tsx:530-536`；群组面包屑为固定文案 `page.tsx:2607`；BranchWorkspace 直接读取 store，无焦点校验 `frontend/src/components/run/BranchWorkspace.tsx:49-60` |
| FR-02-03 | LANE 级应用集为 Backlog / Sprint、Canvas、Workflow / LangGraph、BI / Benchmark、Plugins 五项；Agent 能力并入 Canvas | P0 | 🟡 | BranchWorkspace 五项 `BranchWorkspace.tsx:34-40`；群组页八项 `page.tsx:624-638`；规范外壳六项，仅两项可用 `RunWorkspaceShell.tsx:75, 218` |
| FR-02-04 | 群组页应用深链 `?app=`；旧别名 task-card / jira / work-items 规范为 backlog，multica 规范为 task-flow；未知值回退 backlog | P1 | ✅ | `page.tsx:604-608`（resolveAppId）；`page.tsx:831-841`（currentApp 与 router.replace 规范化） |
| FR-02-05 | BranchWorkspace 与规范外壳的应用页签可经 URL 深链恢复 | P2 | ⬜ | 页签为组件内 state `BranchWorkspace.tsx:55`；规范外壳 `activeApp` 为组件 state，解析时重置为 "Task Cards" `RunWorkspaceShell.tsx:110, 121` |
| FR-02-06 | Branch 任务按工作流状态分组：待办、进行中、评审、阻塞、完成、不处理 | P0 | ✅ | 分组定义 `page.tsx:2529-2536`；列视图 `page.tsx:2546-2548`；仅授权投影产生任务数据 `page.tsx:1030` |
| FR-02-07 | 底部状态筛选同时作用于导航树与 AI 更新 | P0 | 🟡 | 筛选控件 `page.tsx:2701-2711`；树与列表使用筛选结果 `page.tsx:1031-1035, 2662-2663`；AI 更新面板不消费筛选 `page.tsx:2688-2697` |
| FR-02-08 | AI 更新页签按时间倒序展示 AI 更新 | P1 | 🟡 | 页签 `page.tsx:2627-2634`；面板仅为空态占位，无数据源 `page.tsx:2693-2697` |
| FR-02-09 | 任务卡在可关闭子窗口中打开；导航树与 Backlog 共用同一子窗口；Escape 关闭 | P1 | 🟡 | 群组页子窗口 `page.tsx:2926-2932`；Escape `page.tsx:1288-1300`；BranchWorkspace 另用 WorkItemDetailDrawer `BranchWorkspace.tsx:217-225`，为第二套实现 |
| FR-02-10 | Backlog 与 Sprint 应用 | P1 | 🟡 | 群组页 Backlog 仅授权投影显示任务，"新建任务"禁用 `page.tsx:2732-2753`；Sprint 应用 `page.tsx:2755`；BranchWorkspace 使用 store 数据的 SprintBoardView，按 worktree 过滤 `BranchWorkspace.tsx:126-134`，`frontend/src/components/sprint/SprintBoardView.tsx:505-507` |
| FR-02-11 | 画布应用以当前 Worktree 上下文进入 | P1 | 🟡 | BranchWorkspace 链接写死 `canvas-001` 并附带 `worktree` 参数 `BranchWorkspace.tsx:138-143`；群组页画布应用 `page.tsx:3126` |
| FR-02-12 | Workflow / LangGraph 应用 | P1 | 🟡 | 群组页为静态说明，明确不创建 Flow、不运行 Agent `page.tsx:3258-3270`；BranchWorkspace 列出 store 中的工作流并链接 `/workflow` `BranchWorkspace.tsx:156-178`；规范外壳禁用 `RunWorkspaceShell.tsx:218` |
| FR-02-13 | BI / Benchmark 应用 | P1 | 🟡 | BranchWorkspace 仅链接 `/analytics`，统计结果计算后未渲染 `BranchWorkspace.tsx:67-74, 180-193`；规范外壳启用 RunBiPanel `RunWorkspaceShell.tsx:223`；群组页说明卡 `page.tsx:2782` |
| FR-02-14 | Plugins 应用只展示服务端授权的 registry 条目，导航不授予执行能力 | P1 | 🟡 | 群组页 registry 投影 `page.tsx:842-852`；执行界面未安装说明 `page.tsx:3334`；BranchWorkspace 读取 store 的 integrations `BranchWorkspace.tsx:195-214` |
| FR-02-15 | 右侧栏含 Chat、Worktree、Team 三页签；Worktree 状态详情默认折叠 | P1 | ✅ | 群组页右侧面板 `page.tsx:3340-3381`，页签切换 `page.tsx:3383-3393`；详情默认关闭 `page.tsx:714, 3362-3363` |
| FR-02-16 | 对话范围 WORKTREE / GLOBAL；GLOBAL 目标逐个选择且最多 20 个；选择本身不授予权限 | P1 | 🟡 | 群组页范围选择 `page.tsx:3405-3414`；目标列表仅在授权投影下读取 `page.tsx:1004-1026`，上限 `page.tsx:3419, 3434`；全局栏范围按钮 `frontend/src/components/chat/RightChatRail.tsx:222-260`；默认 WORKTREE `frontend/src/lib/chat/chatStore.ts:146`；提交未接通 |
| FR-02-17 | 聊天记录与消息发送 | P1 | 🟡 | 记录区为空态 `page.tsx:3343-3352`；输入与发送禁用 `page.tsx:3456-3465`；客户端 `submitScopedChatMessage` 存在 `frontend/src/lib/group/worktreeGroupApi.ts:1137` 但前端无调用；后端路由已注册 `crates/star-api-rest/src/group_api/scoped_chat.rs:172` |
| FR-02-18 | 数据来源标识：授权投影与本地预览分开展示，预览不混入真实视图 | P0 | 🟡 | 群组页头部徽标 `page.tsx:2612-2614`；Backlog 预览提示 `page.tsx:2745-2749`；BranchWorkspace 全文没有来源标识（`BranchWorkspace.tsx`） |
| FR-02-19 | 会话缺失或授权失败时 fail closed，不以 seed 或本地数据冒充授权数据 | P0 | 🟡 | 投影失效进入错误态 `frontend/src/lib/group/groupProjection.tsx:239-243`；群组错误态 `page.tsx:2421-2431`；但 preview 下群组页读取 store `page.tsx:996-997`，BranchWorkspace 直接读 store `BranchWorkspace.tsx:49-54` |
| FR-02-20 | Lane 绑定状态显式展示：未绑定远端、绑定信息缺失时必须可见 | P1 | 🟡 | "未绑定"徽标 `ProjectRunTree.tsx:195-197`、`page.tsx:2646-2648`、`BranchWorkspace.tsx:92`；群组页绑定文案为固定文本 `page.tsx:2607, 3367`；群组投影 Worktree 视图无 lane_id `groupProjection.tsx:112-118` |
| FR-02-21 | 规范路由 `/projects/{project_id}/runs/{engineering_run_id}/worktrees/{worktree_id}` 解析并校验 Project、Run 与 Worktree 绑定，校验通过后设置焦点 | P0 | ✅ | 路由页 `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx:18-22`；校验与设置焦点 `RunWorkspaceShell.tsx:118-138` |
| FR-02-22 | 旧 Branch 前置路由重定向到规范路由，且不以旧 `branch_id` 授权 | P1 | ✅ | `frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx:21-31` |
| FR-02-23 | 规范外壳仅在焦点验证通过后开放 LANE 应用页签 | P0 | ✅ | `RunWorkspaceShell.tsx:217-224`；测试 `frontend/src/components/run/RunWorkspaceShell.test.tsx:249` |
| FR-02-24 | Lane 数据模型与导航状态：Lane 类型、Worktree.lane_id 可选、导航状态含 Lane / Worktree / Branch 并与 URL 同步 | P0 | 🟡 | 类型 `frontend/src/types/ids.ts:437-447, 466-467`；导航状态 `frontend/src/lib/nav/navStore.ts:55-59, 245-246`；URL 同步 `worktree/page.tsx:530-536`；store 中 lanes 无写入动作 `frontend/src/lib/store.ts:177, 434` |
| FR-02-25 | Lane 可在本地创建、授权与列出，无远端身份时同样可用 | P0 | ⬜ | Lane 集合无创建动作 `store.ts:177, 434`；群组页无"新建 LANE"入口，仅有 Worktree 管理链接 `page.tsx:2712-2714` |
| FR-02-26 | 侧栏树虚拟化与有界加载 | P0 | 🟡 | live 树虚拟窗口 `ProjectRunTree.tsx:88-90, 297-301`；fallback 树为普通列表 `ProjectRunTree.tsx:155`；规范外壳 Worktree 选择器有行数上限 `RunWorkspaceShell.tsx:74` |
| FR-02-27 | Hook 策略入口位于"高级设置 → Hooks"，不作为 Worktree 树节点 | P2 | ✅ | 入口 `RunWorkspaceShell.tsx:229-232`，页面 `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx`；有效策略投影待接入 `RunWorkspaceShell.tsx:230` |
| FR-02-28 | 顶栏功能页签可增删；默认集合为任务、画布、协同、沙盒、原型 | P2 | 🟡 | 顶栏页签 `frontend/src/components/AppHeader.tsx:102-125`；默认集合 `frontend/src/lib/nav/navStore.ts:94`；页签链接为全局模块地址，不带 Lane / Worktree 上下文 `AppHeader.tsx:219`；默认"协同"项为 `nema`（`frontend/src/lib/nav/registry.ts:574-577`），非 collaboration（`registry.ts:645-648`） |
| FR-02-29 | 顶栏分支切换下拉（⎇ 分支 ▾） | P2 | ⬜ | 顶栏仅有写死文案的 workspace-switcher `AppHeader.tsx:71-81`（文案 "ACME Studio" 于 :78）；设计稿按钮 `docs/prototypes/2 · Branch 工作区 · 统一画布-html/Canvas.dc.html:27` 无对应代码控件 |
| FR-02-30 | 全局右侧聊天栏默认展开为聊天 | P1 | ⬜ | 默认 `open=false` `frontend/src/components/chat/RightChatRail.tsx:53`；仅 localStorage `star-chat-rail-open` 可覆盖 `RightChatRail.tsx:56-60` |
| FR-02-31 | 群组页右侧面板与全局右侧聊天栏职责不重复 | P1 | 🟡 | 两者均挂载：全局栏经 `frontend/src/components/AppShell.tsx:77`，群组页自有 aside `page.tsx:3340` 与底栏 `page.tsx:3398-3401`；视觉叠加未在本次静态核对中渲染确认 |
| FR-02-32 | Team 页签：Runtime / Daemon 与 Worker / Router 小队在一处展示，控制动作在通道接入后启用 | P2 | 🟡 | 群组页 Team 页签 `page.tsx:3375-3379`；TeamPanel 动作禁用原因 `frontend/src/components/team/TeamPanel.tsx:100-104, 209-214` |

## 4. 设计

### 4.1 概念模型

- Project：租户内的项目，左侧树第一层。
- Lane：Project 下的管理层级。对外名称为 Lane；内部 engineering_run 契约与路由保持不变（`frontend/src/types/ids.ts:433-436`，`frontend/src/components/Sidebar.tsx:38`）。无远端身份时 Lane 仍可创建、授权与列出，远端仓库信息为可选元数据（`ids.ts:442-445`）。
- Worktree：Lane 管理的本地 checkout，`lane_id` 可选；历史数据无 lane_id 时归入默认分组（`ids.ts:466-467`，`ProjectRunTree.tsx:223-256`）。
- Branch：Worktree 的本地 checkout ref，不是远端 Branch。
- 任务卡：Branch 的子项，按工作流状态分组，点选后打开任务子窗口。
- LANE 应用：Lane 级中央应用页签。Agent 能力并入 Canvas，不作为独立页签。
- 对话范围：WORKTREE（当前 Worktree）或 GLOBAL（显式选择的已授权目标）。
- 数据来源：授权投影（live）或本地预览（preview），界面须显式标识。

### 4.2 数据模型

本功能不新增数据库表。下表列出前端数据结构与持久化位置；服务端 Lane 的持久化与 owner 不在本文核对范围内（❓）。

| 实体 / 状态 | 字段 | 来源与持久化 | 证据 |
|---|---|---|---|
| Lane | id, tenant_id, project_id, name, git_remote_url?, git_provider?, default_branch?, created_at | 客户端类型；store 集合初始为空，无写入动作 | `frontend/src/types/ids.ts:437-447`；`frontend/src/lib/store.ts:177, 434` |
| Worktree | id, project_id, lane_id?, name, branch, base_branch, status（17 个取值）, lock_version | store 集合初始为空；仅 `transitionWorktree` 修改状态 | `ids.ts:452-476`；`store.ts:411, 469` |
| GroupWorktreeView | id, project_id, name, branch, status（无 lane_id） | 服务端 group-context 投影 | `frontend/src/lib/group/groupProjection.tsx:112-118` |
| GroupWorkItemView | id, project_id, worktree_id, key, title, status, lifecycle_status, review_state, labels | 服务端 group-context 投影 | `groupProjection.tsx:120-136` |
| 投影模式 | preview / loading / error / live | 客户端派生 | `groupProjection.tsx:160-164` |
| 导航状态 | selectedProjectId, selectedLaneId, selectedWorktreeId, selectedBranch | localStorage `star-nav-store:v2` | `frontend/src/lib/nav/navStore.ts:55-59, 259` |
| 对话范围 | WORKTREE / GLOBAL，默认 WORKTREE | chatStore 持久化字段 | `frontend/src/lib/chat/chatStore.ts:99, 146, 302` |
| 右侧栏展开态 | open | localStorage `star-chat-rail-open` | `RightChatRail.tsx:56-60` |
| 群组页本地状态 | leftPanelTab, rightPanelTab, rightPanelOpen, workflowFilter, workspaceSearch | 组件 state，不持久化 | `frontend/src/app/worktree/[id]/group/page.tsx:705-713` |
| BranchWorkspace 本地状态 | 当前页签、任务抽屉模式 | 组件 state，不随 URL 变化 | `frontend/src/components/run/BranchWorkspace.tsx:55, 57` |

### 4.3 接口

前端路由：

| 路由 | 查询参数 | 证据 |
|---|---|---|
| `/worktree` | project_id, worktree_id, branch, lane_id | `frontend/src/app/worktree/page.tsx:393-396` |
| `/worktree/[id]/group` | app, scope, work_item_id, cli, canvas_id, highlight | `frontend/src/app/worktree/[id]/group/page.tsx:704, 814-815, 1039-1041, 1275, 3233` |
| `/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]` | 无（路径参数） | `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx:18-22` |
| `/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]` | 忽略 branch_id，重定向 | `frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx:21-31` |

群组 API 客户端与后端路由：

| 方法与路径 | 客户端 | 后端 | 前端调用 |
|---|---|---|---|
| GET `/api/v1/worktrees/{worktree_id}/group-context` | `worktreeGroupApi.ts:860` | `crates/star-api-rest/src/group_api.rs:808-811`，handler `:791-804`，resolver `:507-520` | `groupProjection.tsx:166-245` 经 `loadProjection` |
| GET `/api/v1/worktrees/{worktree_id}/group-apps` | `worktreeGroupApi.ts:864` | `crates/star-api-rest/src/group_api/group_apps.rs:268` | 群组页 registry 状态 `page.tsx:816-822` |
| GET `/api/v1/worktrees/{worktree_id}/chat/targets` | `worktreeGroupApi.ts:1152-1162` | `crates/star-api-rest/src/group_api/scoped_chat.rs:168` | `page.tsx:1004-1026`、`page.tsx:1339` |
| POST `/api/v1/worktrees/{worktree_id}/chat/messages` | `worktreeGroupApi.ts:1137-1143` | `crates/star-api-rest/src/group_api/scoped_chat.rs:172` | 无前端调用 |

路由挂载：`crates/star-api-rest/src/main.rs:141-155` 组装 GroupApiState 并挂载 `build_group_router`。

客户端状态接口：`useNavStore` 的 `setLaneContext(laneId, worktreeId, branch)`（`frontend/src/lib/nav/navStore.ts:83, 245-246`）；`useStore` 的 `lanes`、`worktrees`、`workItems`、`sprints`、`workflows`、`integrations`（`frontend/src/components/run/BranchWorkspace.tsx:49-54`）。

### 4.4 状态机与关键流程

投影状态（群组页）:

- 无 API client：preview（`groupProjection.tsx:239-242`）。
- 会话代际失效：error，消息来自会话状态（`groupProjection.tsx:239-241`）。
- client 存在：进入 loading，加载成功为 live，失败为 error（`groupProjection.tsx:199-213`；错误映射 `:207-210`）。
- 画布订阅返回 401 或 403：error（`groupProjection.tsx:227-228`）。

规范外壳焦点流程（`RunWorkspaceShell.tsx:118-138`）:

1. 清空焦点，并把对话范围置为 GLOBAL。
2. 读取 RunContext。
3. 校验 project_id、engineering_run_id 与焦点 worktree_id 一致；不一致则抛出 `scope_mismatch`，状态进入 error，焦点不设置（`:127-135`）。
4. 一致则设置焦点，并把对话范围置为 WORKTREE（`:131-132`）。

群组页应用切换：`app` 参数决定 currentApp，未知值回退 backlog（`page.tsx:831-832`）；旧别名经 `router.replace` 规范化（`page.tsx:834-841`）。

任务子窗口：打开即写入 `work_item_id`（`page.tsx:1039`）；关闭时删除 `work_item_id` 与 `cli` 并将 app 置为 backlog（`page.tsx:1280-1286`）；Escape 执行同样的关闭（`page.tsx:1288-1300`）。

对话范围：切换写入 `scope` 查询参数（`page.tsx:1307-1312`）；切到 GLOBAL 清空已选目标（`page.tsx:1308`）；目标仅在授权投影下读取（`page.tsx:1004-1026`），选择上限 20（`page.tsx:3434`）。

右侧面板：默认关闭，仅在视口宽度至少 1536px 时由 matchMedia 开启（`page.tsx:707-710`）；点击当前页签的导航按钮收起面板（`page.tsx:3389`）。

URL 到导航状态：`/worktree` 的 project_id、lane_id、worktree_id、branch 在 Project 解析后写入 navStore（`frontend/src/app/worktree/page.tsx:530-536`）。

### 4.5 权限与安全

- 群组 API 需要宿主会话提供的 access token；provider 仅在 `runDirectorySession` 存在时挂载（`frontend/src/app/providers.tsx:51-59`，根布局 `frontend/src/app/layout.tsx:75` 未传入会话）。
- 服务端 group-context 先校验 actor，再要求 `worktree:read` 范围（`crates/star-api-rest/src/group_api.rs:795-797`），resolver 在查询前设置租户上下文（`group_api.rs:526`）。
- GLOBAL 目标列表的说明写明"提交时服务端会再次逐个授权"（`page.tsx:3418`）；提交路径未接通，授权复验不可由前端验证。
- 插件导航不授予执行能力（`page.tsx:3334`）；规范外壳的插件页签禁用（`RunWorkspaceShell.tsx:218`）。
- Hook 策略入口位于"高级设置 → Hooks"，不作为 Worktree 树节点（`RunWorkspaceShell.tsx:229-232`）。
- BranchWorkspace 的任务子窗口写死 `reporterId="usr-002"`（`BranchWorkspace.tsx:223`），身份未从会话读取；画布链接写死 `canvas-001`（`BranchWorkspace.tsx:139`）。
- 测试覆盖：投影在无会话时不回退 seed（`frontend/src/lib/group/__tests__/groupProjection.test.tsx:247`）。

### 4.6 依赖关系

- 外壳：`AppShell`（`frontend/src/components/AppShell.tsx:31-80`）包含 AppHeader（`:49`）与全局 RightChatRail（`:77`）。`/worktree` 经 `frontend/src/app/worktree/layout.tsx:17-19` 挂载，`(app)` 路由经 `frontend/src/app/(app)/layout.tsx:43-49` 挂载。
- 左侧导航：Sidebar 挂载 ProjectRunTree（`frontend/src/components/Sidebar.tsx:210`）；根布局渲染 Sidebar（`frontend/src/app/layout.tsx:78`）。
- 群组 API：`Providers` → `WorktreeGroupApiProvider` → `useWorktreeGroupApi`（`frontend/src/lib/group/groupProjection.tsx:64-106`）。
- BranchWorkspace 依赖 SprintBoardView（`BranchWorkspace.tsx:29, 128`）、WorkItemDetailDrawer（`:30, 218`）与 store（`:49-54`）。
- RunWorkspaceShell 依赖 RunTaskCardsPanel、RunBiPanel（`RunWorkspaceShell.tsx:222-223`）、`useRunDirectory`（`:102`）与 `frontend/src/lib/run/runDirectoryApi.ts`。
- 群组页依赖 TeamPanel（`page.tsx:3377`）与 TaskRunHistoryPanel（`page.tsx:3114`）。
- 相邻功能文件存在于基线（`docs/features/F01-nav-project-lane.md`、`F03-worktree-domain.md`、`F04-task-board.md`、`F06-unified-canvas.md`），边界以本节为准。

### 4.7 页面设计

#### 4.7.1 /worktree

文件: `frontend/src/app/worktree/page.tsx`（1296 行）。

布局:

- 顶栏: AppHeader，由 `frontend/src/components/AppShell.tsx:49` 提供。
- 侧栏: 全局 Sidebar，`frontend/src/app/layout.tsx:78`。
- 主区: `AppShell.tsx:61-73`；页面内容 `worktree/page.tsx:613-620`。
- 右侧栏: 全局 RightChatRail，`AppShell.tsx:77`。
- 底栏: 本页无页面级底栏。
- 外壳入口: `frontend/src/app/worktree/layout.tsx:17-19`。

组件树:

- WorktreePage `worktree/page.tsx:356-362`，Suspense 包裹 WorktreePageInner `:377`。
- 无群组 API 分支 `:578-611`：BranchWorkspace（有参数时）`:582-584`，MagosLanePanel `:590`，GitBindingPanel `:593`。
- UUID project 分支 `:574-576`：AuthenticatedWorktreeIndex，此分支不渲染 BranchWorkspace。
- 群组 API 分支 `:613-620`：BranchWorkspace（有参数时）`:616-619`，PageHeader `:621-627`，项目选择器 `:662`。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 群组 API 可用，project_id 非 UUID，且同时带 worktree_id 与 branch | BranchWorkspace 渲染于页面顶部 `page.tsx:616-619` | 🟡 BranchWorkspace 数据来自 store，未经授权投影 |
| 加载 | Suspense 未完成 | 骨架屏 `page.tsx:364-375` | ✅ |
| 空 | 缺少 worktree_id 或 branch | 不渲染 BranchWorkspace，显示 Index 标题 `page.tsx:621-627` | ✅ |
| 错误 | 所选 Project 不可用 | 警告卡片 `page.tsx:687-692` | 🟡 文案存在，授权错误码展示未核对 |
| 未授权 | UUID project_id | 进入 AuthenticatedWorktreeIndex `page.tsx:574-576` | ❓ 401 或 403 在索引组件内的处理未在本页核对 |
| 未接入 | 无群组 API provider | "worktree-directory-unavailable" 区块 `page.tsx:578-611` | ✅ |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换 Project | 授权目录已加载 | 写入 project_id 并 replace URL `page.tsx:524-528` | URL、navStore | ✅ |
| 点选 Worktree（导航） | 同一 Lane 下的 Worktree | 写入 lane_id、worktree_id、branch 并进入本页 | URL、navStore（`ProjectRunTree.tsx:206-210`） | ✅ |
| 切换 BranchWorkspace 页签 | 参数齐全 | 组件内切换 `BranchWorkspace.tsx:101-123` | 组件 state | 🟡 不写入 URL |
| 打开任务卡 | 点选 Backlog 任务 | 打开 WorkItemDetailDrawer `BranchWorkspace.tsx:131, 217-225` | store | 🟡 与群组页子窗口为两套实现 |

数据: 客户端 store 的 projects、worktrees、agentSessions、localRuntimes（`page.tsx:382`）与 navStore（`:386-391`）；UUID 分支使用群组 API 索引。

与文档设计的差距:

- FR-02-02：BranchWorkspace 不校验焦点，直接以 store 查找 Worktree（`BranchWorkspace.tsx:59`）。
- FR-02-19：BranchWorkspace 无来源标识，preview 与 live 不区分（`BranchWorkspace.tsx` 全文）。
- FR-02-05：页签不支持 URL 深链（`BranchWorkspace.tsx:55`）。
- FR-02-01：BranchWorkspace 的数据入口只在非 UUID project_id 下可达（`worktree/page.tsx:574-576`），与 store 中的 project 标识方式并存。

#### 4.7.2 /worktree/[id]/group

文件: `frontend/src/app/worktree/[id]/group/page.tsx`（3522 行）。该路由无独立 layout，由 `frontend/src/app/worktree/layout.tsx:17-19` 提供 AppShell。

布局:

- 顶栏: AppHeader（`AppShell.tsx:49`）。
- 页内头部: 58px 工作区头部，`page.tsx:2600-2623`。
- 左侧: 导航与 AI 更新 aside，`page.tsx:2626-2716`。
- 主区: 应用导航 `page.tsx:2719-2730`，应用内容 `page.tsx:2731-3300`。
- 右侧栏: 页内 aside `page.tsx:3340-3381`，页签导航 `page.tsx:3383-3393`；全局 RightChatRail 同时挂载（`AppShell.tsx:77`）。
- 底栏: 固定定位的群组聊天条 `page.tsx:3398-3470`。

组件树:

- GroupWorkspacePage `page.tsx:700`。
- 任务子窗口（role=dialog）`page.tsx:2926-2932`，内含 CLI 面板与任务操作 `page.tsx:2945-2983`。
- 右侧 TeamPanel `page.tsx:3377`；任务运行历史 TaskRunHistoryPanel `page.tsx:3114`。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常（授权投影） | groupProjection 为 live | 头部徽标"授权投影" `page.tsx:2612-2614`；任务按状态分组 `page.tsx:2546-2548` | ✅ 仅在宿主会话 provider 存在时可达 |
| 加载 | groupProjection 为 loading | "正在读取已授权的 Worktree Group 数据…" `page.tsx:2417-2419` | ✅ |
| 空 | live 且无任务，或筛选后为空 | "当前筛选下没有任务。" `page.tsx:2682`；AI 更新空态 `page.tsx:2693-2697`；聊天空态 `page.tsx:3348-3352` | ✅（AI 更新与聊天为占位） |
| 错误 | 投影失败或会话失效 | role=alert 区块，明示"不会回退显示本地 seed 数据" `page.tsx:2421-2431` | ✅ |
| 找不到 | store 中无该 Worktree（预览态） | "找不到 Worktree" `page.tsx:2434-2444` | ✅ |
| 未授权 | 401 或 403 | 与错误态同路径；画布订阅 401/403 进入 error（`groupProjection.tsx:227-228`） | 🟡 普通投影的 401 与 403 不区分 |
| 未接入（预览） | 无 provider | 头部"本地预览 · seed 任务已隐藏"`page.tsx:2612-2614`；Backlog 提示 `page.tsx:2745-2749`；聊天"未连接" `page.tsx:3346` | ✅ |
| 禁用 | 无授权目标或未接通发送 | "新建任务"禁用 `page.tsx:2742`；输入与发送禁用 `page.tsx:3456-3465` | ✅ |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换导航 / AI 更新 | 无 | 切换左侧页签 `page.tsx:2627-2634` | 组件 state | ✅ |
| 状态筛选 | 无 | 过滤树与列表 `page.tsx:2701-2711, 1031-1035` | 组件 state | ✅ 不作用于 AI 更新 |
| 切换应用 | 无 | 写入 `app` 并跳转 `page.tsx:2725, 1302-1305` | URL | ✅ |
| 打开任务 | 任务存在于投影 | 写入 work_item_id，打开子窗口 `page.tsx:1274-1278, 2926` | URL | ✅ |
| 关闭任务子窗口 | 子窗口已打开 | 删除查询参数，返回 backlog `page.tsx:2932, 1280-1286` | URL | ✅ |
| 打开 CLI 面板 | 任务属于当前 Worktree | 写入 `cli=1` `page.tsx:2977-2979` | URL | 🟡 CLI 内部见其他功能 |
| 切换对话范围 | 无 | 写入 scope `page.tsx:3408, 1307-1312` | URL | ✅ |
| 选择 GLOBAL 目标 | 授权投影下 GLOBAL | 勾选目标，最多 20 个 `page.tsx:3430-3436` | 组件 state | ✅ 选择不提交 |
| 发送聊天消息 | 无 | 按钮禁用 `page.tsx:3465` | 无 | ⬜ 未接通 |
| 切换右侧页签 | 无 | 打开或切换或收起面板 `page.tsx:3389` | 组件 state | ✅ |

数据: `useWorktreeGroupProjection` `page.tsx:995`；store 的 worktrees、projects、canvases、canvasElements、canvasConnectors `page.tsx:806-810`；Lane 来自 store lanes `page.tsx:1000-1003`；GLOBAL 目标来自 `listGlobalChatTargets` `page.tsx:1010`。

与文档设计的差距:

- FR-02-01：群组树在 Branch 下挂任务，与 FR-02-01 的叶节点描述不同；本文以 WBS 的分组设计为目标（见第 8 节）。
- FR-02-02：面包屑为固定文本 `page.tsx:2607`，不随 Lane 绑定变化。
- FR-02-07：筛选不作用于 AI 更新 `page.tsx:2688-2697`。
- FR-02-08：AI 更新无数据源 `page.tsx:2693-2697`。
- FR-02-09：任务子窗口为 dialog，与 BranchWorkspace 的 WorkItemDetailDrawer 为两套实现。
- FR-02-17：聊天发送未接通 `page.tsx:3456-3465`。
- FR-02-18、FR-02-19：预览态读取 store `page.tsx:996-997`，与"预览不混入真实视图"的要求不完全一致。
- FR-02-20：绑定文案固定 `page.tsx:2607, 3367`。
- FR-02-25：无 Lane 创建入口（`page.tsx:2712-2714`）。
- FR-02-31：群组页右侧面板与全局右侧栏同时挂载。

#### 4.7.3 /projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]

文件: `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx`（22 行，直接渲染 `RunWorkspaceShell`，`frontend/src/components/run/RunWorkspaceShell.tsx:101`）。

布局:

- 顶栏: AppHeader，经 `frontend/src/app/(app)/layout.tsx:43-49` 的 AppShell。
- 侧栏: 全局 Sidebar 与 ProjectRunTree（`frontend/src/components/Sidebar.tsx:210`）。
- 主区: 页头 `RunWorkspaceShell.tsx:211-216`，LANE 应用页签 `:217-219`，应用内容 `:220-224`，Worktree 选择区 `:233-259`，RunContext 详情 `:261`。
- 右侧栏: 全局 RightChatRail（`AppShell.tsx:77`）。
- 底栏: 页内 footer 聊天范围与输入 `RunWorkspaceShell.tsx:262-267`。

组件树:

- RunWorktreePage `page.tsx:20-22` → RunWorkspaceShell `RunWorkspaceShell.tsx:101`。
- RunTaskCardsPanel `:222`；RunBiPanel `:223`；RunContextDetails `:261`。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 焦点验证通过 | 页头显示 Checkout 焦点，LANE 应用页签可用（Task Cards、BI） `RunWorkspaceShell.tsx:210-224` | ✅ |
| 无效路径 | 路径参数无法构造 href | "LANE 路径无效" `:202` | ✅ |
| 加载 | 会话或上下文未就绪 | "正在验证 Project 与 LANE 授权…" `:203-204` | ✅ |
| 错误 | RunContext 读取失败或 scope_mismatch | role=alert "LANE 工作区暂不可用" `:205`；错误来源 `:129, 135` | ✅ |
| 无焦点 | 仅 Project 与 Run 有效 | 显示 Worktree 选择区，不挂载 LANE 应用 `:233-259` | ✅ |
| 空 | 当前 LANE 无授权 Worktree | "此 LANE 当前没有…" `:241` | ✅ |
| 未接入 | 服务端未开放 Task Cards 读取 | 页签仍可见但内容禁用，文案说明不回退演示数据 `:226` | ✅ |
| 禁用 | 非 Task Cards 与 BI 的页签 | disabled 并标注"待迁移" `:218` | ✅ |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 选择 Worktree | 无焦点 | 导航到带 worktree 的规范路由 `:248` | 路由 | ✅ |
| 搜索 Worktree | 已加载行 | 过滤已加载行 `:244-245` | 组件 state | ✅ |
| 加载更多 Worktree | 有 next_cursor | 追加分页 `:257` | `listWorktrees` | ✅ 有界 |
| 切换 LANE 应用 | 焦点已设置 | 切换 Task Cards 或 BI `:218` | 组件 state | 🟡 其余页签禁用 |
| 切换聊天范围 | 焦点已设置时可选 WORKTREE | 切换按钮状态 `:264` | 组件 state | 🟡 输入与发送禁用 |
| 打开 Hook 设置 | 无 | 跳转 `/settings/advanced/hooks` `:231` | 路由 | ✅ 有效策略投影待接入 |

数据: `useRunDirectory` 提供 client 与 status `RunWorkspaceShell.tsx:102`；RunContext 来自 `client.getRunContext`（`:124`）；Worktree 列表来自 `listWorktrees`（`lib/run/runDirectoryApi.ts`）。

与文档设计的差距:

- FR-02-03：Inbox、Canvas、Workflow、Plugin 页签仅显示"待迁移"且禁用 `RunWorkspaceShell.tsx:75, 218`。
- FR-02-05：页签无 URL 深链，初值固定为 Task Cards（`:110, 121`）。
- FR-02-20：Lane 绑定信息未在页头呈现。
- FR-02-28、FR-02-31：聊天范围按钮只改变展示态，不接入聊天 runtime（`:266`）。

#### 4.7.4 /projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]

文件: `frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx`（32 行，仅重定向）。

布局: 无页面 UI；重定向前由 `(app)/layout.tsx` 的外壳约束（无可见输出）。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 有效 | canonicalRunWorktreeHref 构造成功 | 重定向到规范路由 `page.tsx:25-31` | ✅ |
| 无效 | 构造失败 | notFound `page.tsx:26-30` | ✅ |

交互: 无。

数据: `canonicalRunWorktreeHref`（`frontend/src/lib/run/runDirectoryApi.ts:945-947`）；不使用 branch_id 参与授权。

与文档设计的差距: 无（FR-02-22 已满足）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| /worktree 入口与 BranchWorkspace 挂载 | `frontend/src/app/worktree/page.tsx` | 🟡 | `:356-362, 574-576, 582-584, 616-619` |
| BranchWorkspace（5 页签） | `frontend/src/components/run/BranchWorkspace.tsx` | 🟡 | `:34-40, 101-214`；无测试文件 |
| 群组工作区（8 应用、树、右栏、聊天条） | `frontend/src/app/worktree/[id]/group/page.tsx` | 🟡 | `:624-638, 2600-2731, 3340-3470`；无测试文件 |
| 规范 LANE 外壳（6 页签，2 可用） | `frontend/src/components/run/RunWorkspaceShell.tsx` | 🟡 | `:75, 101-270`；测试 `RunWorkspaceShell.test.tsx` |
| 规范路由页 | `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx` | ✅ | `:18-22` |
| 旧 Branch 路由重定向 | `frontend/src/app/(app)/projects/[project_id]/branches/[branch_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx` | ✅ | `:21-31` |
| 全局树（live） | `frontend/src/components/run/ProjectRunTree.tsx` | 🟡 | `:92-100, 273-534` |
| 全局树（fallback） | `frontend/src/components/run/ProjectRunTree.tsx` | 🟡 | `:110-271`；非虚拟化 `:155` |
| Lane 类型与 Worktree.lane_id | `frontend/src/types/ids.ts` | ✅ | `:437-447, 466-467` |
| Lane 集合写入路径 | `frontend/src/lib/store.ts` | ⬜ | `:177, 434`，无写入动作 |
| 导航状态（lane / worktree / branch） | `frontend/src/lib/nav/navStore.ts` | ✅ | `:55-59, 83, 245-246, 259`；测试 `frontend/src/lib/nav/__tests__/navStore.test.ts` |
| URL 到导航状态同步 | `frontend/src/app/worktree/page.tsx` | ✅ | `:530-536` |
| 顶栏功能页签 | `frontend/src/components/AppHeader.tsx` | 🟡 | `:52-59, 102-125, 219`；默认集合 `navStore.ts:94` |
| 顶栏分支切换 | `frontend/src/components/AppHeader.tsx` | ⬜ | `:71-81` 仅有写死的 workspace-switcher |
| 全局右侧聊天栏 | `frontend/src/components/chat/RightChatRail.tsx` | 🟡 | `:29-60, 222-260`；默认折叠 `:53`；`frontend/src/lib/chat/chatStore.ts:99, 146` |
| 群组右侧面板（Chat / Worktree / Team） | `frontend/src/app/worktree/[id]/group/page.tsx` | ✅ | `:3340-3393` |
| 群组聊天条与对话范围 | `frontend/src/app/worktree/[id]/group/page.tsx` | 🟡 | `:3398-3470`；发送禁用 `:3465` |
| 任务子窗口（群组） | `frontend/src/app/worktree/[id]/group/page.tsx` | ✅ | `:2926-2932`；Escape `:1288-1300` |
| 任务抽屉（BranchWorkspace） | `frontend/src/components/board/WorkItemDetailDrawer.tsx` | 🟡 | 第二套实现，`BranchWorkspace.tsx:217-225` |
| 群组 API 客户端 | `frontend/src/lib/group/worktreeGroupApi.ts` | ✅ | `:860, 864, 1137-1143, 1152-1162` |
| 群组投影 provider 与 hook | `frontend/src/lib/group/groupProjection.tsx` | ✅ | `:64-106, 166-245` |
| 群组 API provider 挂载 | `frontend/src/app/providers.tsx`，`frontend/src/app/layout.tsx` | 🟡 | 仅会话存在时挂载 `providers.tsx:51-59`；根布局未传入 `layout.tsx:75` |
| 后端 group-context | `crates/star-api-rest/src/group_api.rs` | ✅ | `:507-520, 791-804, 806-829`；挂载 `crates/star-api-rest/src/main.rs:155` |
| 后端 group-apps | `crates/star-api-rest/src/group_api/group_apps.rs` | ✅ | 路由 `:268`，registry 提供方 `main.rs:138-143` |
| 后端 chat 路由 | `crates/star-api-rest/src/group_api/scoped_chat.rs` | 🟡 | 路由 `:168-172` 已注册；前端无发送调用 |
| 群组页 Team 面板 | `frontend/src/components/team/TeamPanel.tsx` | 🟡 | 动作禁用 `:100-104, 209-214` |
| Hook 入口 | `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` | ✅ | 入口由 `RunWorkspaceShell.tsx:231` 链接 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 三套 LANE 应用集（BranchWorkspace 5、群组页 8、规范外壳 6）并存 | 同一 Lane 在不同入口看到不同功能，用户无法确定权威入口 | 以 FR-02-03 的五项为唯一集合；群组页与规范外壳收敛到同一组件；删除或映射多余页签 |
| Lane 集合无写入路径（`store.ts:177, 434`），也无创建 UI | 无法在本地创建、授权或列出 Lane，Worktree 无法稳定归属 Lane | 增加 Lane 的 store action 或服务端 Lane owner 接口；群组页与导航接入同一来源 |
| 群组页绑定文案为固定文本，群组投影无 lane_id（`groupProjection.tsx:112-118`） | 绑定状态不真实，"未绑定"无法依据服务端事实显示 | 在 group-context 投影中加入 Lane 绑定字段，并替换 `page.tsx:2607, 3367` 的固定文案 |
| BranchWorkspace 直接读 store，无来源标识、无焦点校验，写死 reporterId 与 canvas-001（`BranchWorkspace.tsx:59, 139, 223`） | 预览数据可能被当作授权数据；任务报告人与画布目标不随会话变化 | 改为读取授权投影或显式标注预览；reporterId 来自会话；画布目标来自 Lane 上下文 |
| 聊天发送未接通（`page.tsx:3456-3465`；`submitScopedChatMessage` 无调用） | 对话范围与目标选择无法产生效果 | 接入 `submitScopedChatMessage` 与 transcript 读取，并在提交时复验授权 |
| AI 更新与聊天记录无数据源（`page.tsx:2693-2697, 3348-3352`） | AI 更新页签与右侧聊天为占位 | 接入事件流或 transcript；筛选同时作用于 AI 更新 |
| 任务子窗口两套实现（group dialog 与 WorkItemDetailDrawer） | 行为与键盘交互不一致；维护成本高 | 统一为 WorkItemDetailDrawer 形态的同一子窗口（见第 8 节裁决） |
| 全局右侧聊天栏默认折叠（`RightChatRail.tsx:53`） | 与 WBS 的"默认展开聊天"目标不一致 | 按第 8 节裁决改为默认展开，或订正目标 |
| 群组页右侧面板与全局右侧栏同时挂载（`AppShell.tsx:77`，`page.tsx:3340`） | 视觉与快捷键重复，聊天入口分散 | 明确群组页隐藏全局栏或合并为一个右栏，并做渲染核对 |
| 默认部署不挂载群组 API provider（`app/layout.tsx:75`） | 群组页只能预览，BranchWorkspace 读取通常为空的 store | 明确 web 版会话注入方式；未接入时以显式空态替代 store 回退 |
| BI 统计计算未渲染（`BranchWorkspace.tsx:67-74`） | 计算成本白费，BI 页签信息量不足 | 渲染统计卡片或删除计算 |
| 规范外壳的 Hook 有效策略投影待接入（`RunWorkspaceShell.tsx:230`） | 用户无法看到有效策略与执行结果 | 接入 Hook 投影的只读展示，保留深链 |
| 顶栏无分支切换与 Branch 上下文（`AppHeader.tsx:71-81, 219`） | 顶栏页签不随 Worktree 变化，"⎇ 分支 ▾"无对应控件 | 增加分支切换与上下文参数；页签链接携带 lane / worktree |
| 顶栏"协同"默认指向 `nema`（`navStore.ts:94`，`registry.ts:574-577`） | 标签名称与目标模块不一致 | 默认集合改为 collaboration 或更名，与设计稿一致 |
| 无 Lane 归类文案不统一（群组页"未归类" `page.tsx:2645`；fallback "默认 LANE"，`zh-CN.ts:49`） | 同一状态两种文案 | 统一为第 8 节裁决的文案 |
| `/work-item` 路由残留（`redirects.ts:93-96` 仅重定向精确路径，`app/(app)/work-item/[id]/page.tsx` 存在） | 任务卡存在第二入口 | 在任务子窗口统一后移除入口，保留重定向 |

## 7. 验收与测试

已有测试与位置:

- `frontend/src/lib/group/__tests__/groupProjection.test.tsx`：无 provider 时仅预览（`:25-30`）；代际替换后的迟到 401 不覆盖新会话（`:84`）；画布 Outbox 订阅（`:191`）；无会话时拒绝回退 seed（`:247`）；拒绝跨 Project 的 GroupContext（`:260`）。
- `frontend/src/components/run/RunWorkspaceShell.test.tsx`：未聚焦时只显示目录选择、不挂载 LANE 应用（`:154`）；Worktree 选择器的分页、去重与行数预算（`:175, 206, 223, 238`）；验证焦点后显示 LANE 应用（`:249`）。
- `frontend/src/components/run/ProjectRunTree.test.tsx`：选择 Project 清除 Worktree 焦点（`:114`）；展开与键盘导航（`:128, 146`）。
- `frontend/src/lib/nav/__tests__/navStore.test.ts`：页签与侧栏初始化、持久化键与 v3 迁移（`:21, 37, 148, 192`），拒绝将 worktree 模块加入侧栏（`:218`）。
- `frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx`：Worktree 控制中心位于 Home 区（`:100`）；Worktree 行的生命周期与绑定事实（`:132`）。
- `frontend/src/app/providers.test.tsx`：会话代际切换（`:74-93`）。

已核对可验证的内容（静态）:

- 路由与重定向、焦点门控、Lane 类型与导航状态字段、页签集合与默认值、投影状态机、群组 API 的路由注册与认证范围。

未覆盖或未核实:

- BranchWorkspace、群组页的页签与导航、聊天范围与右侧栏：无单元测试，无 e2e（`frontend/e2e/` 中的 worktree 规格面向画布与 MSW 接口，`worktree-canvas.spec.ts`、`worktree-creation-flow.spec.ts`）。
- 群组页在宿主会话存在时的端到端行为（❓）。
- 全局右侧栏与群组页右栏的视觉叠加（❓）。
- group-context 的专门后端测试（❓；仅核对到路由与 resolver 实现）。
- 授权失败与 Lane 绑定字段在服务端的语义（❓）。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/prototypes/2 · Branch 工作区 · 统一画布-html/README.md` | 原型为参考稿，实现应复刻视觉值而非照搬标记；原型目录与运行方式 | 无 | 作为设计参考，不作为实现依据 |
| `docs/prototypes/2 · Branch 工作区 · 统一画布-html/Canvas.dc.html`（最后变更 af0da2a9，2026-11-15） | 顶栏"Branch 功能"页签：任务、画布、协同、沙盒、原型，加"添加页签"；顶栏分支按钮（`:27-34`） | "协同"链接指向 Docs 原型，代码默认集合为 `nema`；分支按钮无代码控件；画布主体属于 F06 | 顶栏功能页签采用本稿的五项为目标；分支按钮作为目标；"协同"映射在第 6 节列为差距 |
| `docs/frontend/design/mockups/branch-workspace-v1.svg`（最后变更 71709f62，2026-11-13） | 左树止于 Branch；顶栏分支选择器；九项顶栏页签；任务主区按状态分列；任务详情右侧子窗口；右栏聊天与 Team；底部 WORKTREE / GLOBAL | 树叶子与 WBS 的任务分组不同；九项页签与 WBS 五项不同；代码无分支选择器 | 树的任务挂载采用 WBS（见下）；页签集合采用 WBS 五项；顶栏九项为旧稿，不作为目标 |
| `docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md`（v1.5，最后变更 5d25d914，2026-11-15） | Lane 模型与命名；五项 LANE 应用；群组左侧双页签与分组；右栏三页签与 WORKTREE / GLOBAL；统一授权上下文；Agent 并入 Canvas；内部 engineering_run 命名保留 | §0 写"RightChatRail 默认展开"，代码默认折叠；WBS 2.3 写"/work-item 不再作入口"，代码仍有页面与少量入口；WBS 状态为计划中的自述 | 作为目标设计的主要来源；代码为当前状态的依据 |
| `docs/features/F02-branch-workspace.md`（基线中的既有版本，声明的核对基线为 cdd00a78） | 需求候选与证据线索 | 其行号与声明基线不适用于 637ed19d；多条主张需要重新核对 | 需求编号重新定义；仅保留经 637ed19d 行号核对的主张 |
| `docs/reports/PHASE-DELIVERY-SYNC-20261118.md:124` | "外壳已有；应用页签三套并存；Lane 无写入路径；聊天未接通" | 与代码一致 | 交叉核对，不作为设计来源 |
| 代码（637ed19d）：第 3 节证据所列文件 | 当前实现与状态 | 无 | 作为当前状态的依据 |
| `docs/features/F01-nav-project-lane.md`、`F03-worktree-domain.md`、`F04-task-board.md`、`F06-unified-canvas.md`、`F07`、`F08`、`F10`、`F12`–`F17` | 仅确认文件存在，作为边界参照 | 未读取正文 | 不纳入本文裁决 |
| `frontend/src/app/worktree/chat/page.tsx` | 仅读取文件头注释（任务卡聊天与全局聊天页签） | 聊天页属于聊天功能 | 不纳入本文范围 |
| `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx` | 仅确认存在 | Lane 设置属于 Lane 管理 | 不纳入本文范围 |
| `crates/star-api-rest/src/group_api*` | 路由注册、group-context resolver、chat 路由、group-apps 挂载 | 具体业务语义属于后端功能文档 | 只采用路由与认证范围 |

裁决:

1. 左侧树的任务挂载：采用 WBS（任务挂在 Branch 下，按工作流状态分组）。理由：WBS 为 2026-11-15 版本，晚于 2026-11-13 的 SVG；群组页已按此实现（`page.tsx:2662-2680`）。
2. 中央 LANE 应用集：采用 WBS 的五项（Backlog / Sprint、Canvas、Workflow / LangGraph、BI / Benchmark、Plugins）。理由：WBS 2.1 已在 BranchWorkspace 落地（`BranchWorkspace.tsx:34-40`）；SVG 的九项与群组页的八项、规范外壳的六项均作为收敛对象。
3. 顶栏功能页签：采用原型 `Canvas.dc.html` 的五项与 navStore 的默认集合（`navStore.ts:94`）。理由：原型于 2026-11-15 与 WBS 同期对齐，晚于 SVG；SVG 的九项页签不作为目标。
4. 任务子窗口形态：采用同一个可关闭子窗口，以 WorkItemDetailDrawer 为目标实现。理由：WBS 2.3（提交 3cc5b6b7）与 BranchWorkspace 注释（`BranchWorkspace.tsx:12`）均指向该组件；群组页 dialog 列为收敛对象。
5. 右侧聊天栏默认：采用 WBS 的"默认展开聊天"为目标；代码默认折叠列为差距（FR-02-30）。理由：WBS 是被接受的目标设计；代码状态以第 4.7 节为准。
6. 无 Lane 归类文案：采用"未归类"。理由：群组页（主工作区）与原需求用语一致；"默认 LANE"仅出现在 fallback 的 i18n 文案（`zh-CN.ts:49`），列为统一对象。
7. 对外名称：采用 Lane，不再使用 Engineering Run；内部 engineering_run 契约与路由保持。理由：WBS §2 与 `ids.ts:433-436` 一致。
8. 全局聊天栏与群组页右栏：两者保留并列为目标，但职责区分：全局栏承载会话传输入口，群组页右栏承载 Worktree 上下文面板；重复挂载的处理列为差距（FR-02-31）。理由：WBS 5.1 与 5.2 分别描述两者，未给出合并决定。
