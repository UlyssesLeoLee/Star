# F01 项目·Lane·Worktree 导航与层级

> 状态摘要: 导航骨架已在代码中落地 (navStore 四层选择状态、侧栏两段结构、Lane 类型与"未绑定"标识、深链恢复、Lane 级中央应用页签); Lane 的创建与授权、宿主模式目录的点选写入、面包屑与顶栏上下文绑定、降级目录的缓存隔离尚未实现, 整体 🟡。

> 代码基线: HEAD 1643559f (single baseline for every feature doc); 核对方法: 路径与符号 grep 加工作树文件读取, 所引文件的工作树内容与 HEAD 1643559f 一致 (路径限定 git diff 为空); 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

- 以 Project 为根, 在侧栏与工作区中呈现 Project → Lane → Worktree → Branch 四层层级; 选中后, Lane 级功能共享同一导航上下文。
- Lane 是本地管理层级, 无远端 Branch 身份时仍可创建、授权、列出与管理; 远端仓库与分支绑定只是可选元数据 (WBS §2)。
- 选择结果可通过 URL 深链恢复; 未绑定远端、未授权等状态显式可见, 不以示例数据冒充真实状态。

### 1.2 范围内 / 范围外

| 范围 | 内容 |
|---|---|
| 范围内 | 侧栏容器与层级树 (Sidebar.tsx, ProjectRunTree.tsx); 导航状态 store 与深链 (navStore.ts, /worktree 查询参数); Lane 实体的前端类型与 store; Lane 级中央应用页签 (BranchWorkspace.tsx); Worktree 页与 Group 页的导航结构; 顶栏页签与模块添加入口 (AppHeader.tsx, AppMatrixDrawer.tsx, registry.ts) |
| 范围外 | 任务卡内容与状态机 (仅记录其导航入口, 即 BranchWorkspace 的子窗口); Canvas 内部关系图与 Agent 画布; 宿主认证与令牌管理 (runDirectorySession 仅作为依赖); 后端 domain-project / domain-workspace 的业务不变量 (仅作设计依据引用); 聊天栏 RightChatRail 与 WORKTREE/GLOBAL 范围 |

## 2. 用户与场景

角色 (取自 domain-project-spec §7 的 Permission 与内置 Role): tenant_admin、project_admin、developer、viewer。前端导航树不按角色隐藏节点; 仅在树节点上显示 Project 的 role 文本 (ProjectRunTree.tsx:444)。

| 场景 | 说明 | 当前入口 |
|---|---|---|
| S1 选择 Project 并展开 | 在侧栏选中 Project, 展开其 Lane 与 Worktree | ProjectRunTree.tsx:153-271 (降级树), :273-406 (宿主树) |
| S2 进入 Worktree 工作台 | 点选 Worktree, 在 Lane 级页签 (Backlog/Sprint、Canvas、Workflow、BI、Plugins) 之间切换 | worktree/page.tsx:583, 618; BranchWorkspace.tsx:101-123 |
| S3 无远端 Lane 下工作 | Lane 无远端绑定时显式标注"未绑定", 仍可进入其下 Worktree | ProjectRunTree.tsx:195-197; BranchWorkspace.tsx:89-93 |
| S4 分享与恢复视图 | 通过 /worktree?project_id&lane_id&worktree_id&branch 恢复导航上下文 | worktree/page.tsx:393-396, 530-536 |
| S5 窄屏访问目录 | 小屏通过底部导航抽屉打开项目目录 | MobileBottomNav.tsx:79-122 |

## 3. 需求

优先级: P0 = WBS Phase 1 及其阻塞项; P1 = WBS Phase 2 至 6; P2 = Phase 7 或体验项。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-01-01 | 侧栏以 Project 为根, 展示 Project → Lane → Worktree → Branch 层级树 | P0 | 🟡 | 降级树四层: ProjectRunTree.tsx:153-271; 宿主树节点: :443-451, :477-535; 宿主树默认未挂载, 见 FR-01-27 |
| FR-01-02 | 对外术语为 Lane, 不使用 Engineering Run | P0 | ✅ | 运行节点标签 `LANE · ${state}` ProjectRunTree.tsx:445; 侧栏注释 Sidebar.tsx:36-38; 用户可见文本中无 "Engineering Run" (仅 types/ids.ts:434 注释); 内部 engineering_run 路由与 ID 保留 runDirectoryApi.ts:941-947 |
| FR-01-03 | Lane 实体: id、tenant_id、project_id、name, 可选远端字段 git_remote_url、git_provider、default_branch | P0 | ✅ | types/ids.ts:437-447 |
| FR-01-04 | Worktree 可选归属 Lane (lane_id); 未归类 Worktree 显示为"未归类" | P0 | 🟡 | 类型 types/ids.ts:462-478 (lane_id 在 :467); 未归类分组 ProjectRunTree.tsx:223-256 (仅降级树); 宿主树不按 Lane 分组; 后端无 lane_id (见 §5) |
| FR-01-05 | 无远端身份时可创建 Lane | P0 | ⬜ | store 中 lanes 只有类型与空数组 (store.ts:177, 434), 无写入 action; frontend/src/app/api 下无 lane 路由; crates 中无 Lane 实体 |
| FR-01-06 | 列出 Project 下的 Lane | P0 | 🟡 | 降级树 ProjectRunTree.tsx:180 从 store.lanes 过滤; 宿主树的 LANE 节点来自 engineering-runs 接口, 与 store.lanes 不是同一数据源 |
| FR-01-07 | Lane 级授权 (授权范围覆盖 Lane 下资源) | P0 | ⬜ | 授权只在 Project 与宿主会话层实现 (runDirectorySession.tsx:66-80), 无 Lane 维度 |
| FR-01-08 | 未绑定远端的 Lane 与 Worktree 显式显示"未绑定" | P0 | ✅ | ProjectRunTree.tsx:195-197; BranchWorkspace.tsx:89-93; worktree/[id]/group/page.tsx:2646-2648 |
| FR-01-09 | 导航选择状态 (selectedProjectId、selectedLaneId、selectedWorktreeId、selectedBranch) 持久化 | P0 | ✅ | navStore.ts:55-59; 持久化名与版本 navStore.ts:258-262; partialize navStore.ts:268-278 |
| FR-01-10 | 切换 Project 时清空 Lane、Worktree、Branch | P0 | ✅ | navStore.ts:237-244; 单测缺失 (见 §7) |
| FR-01-11 | 点选 Lane 或 Worktree 写入统一导航上下文 | P0 | 🟡 | 降级树写入: ProjectRunTree.tsx:141-147, 189, 209, 243; 宿主树的 selectLane、selectWorktree 定义于 :308-314 但无调用点, DirectoryRow 也未接收 (:452-456) |
| FR-01-12 | URL 深链 (project_id、lane_id、worktree_id、branch) 恢复导航上下文 | P1 | 🟡 | URL 写入 store: worktree/page.tsx:393-396, 530-536; 反向仅经链接 ProjectRunTree.tsx:207, 241; 无页面级单测 |
| FR-01-13 | 选中 Worktree 后中央应用按 worktreeId、branch、laneId 渲染 | P1 | ✅ | worktree/page.tsx:583, 618 (参数来自 URL, 非 navStore); BranchWorkspace.tsx:42-65 |
| FR-01-14 | Lane 级中央应用 5 个页签: Backlog/Sprint、Canvas、Workflow、BI、Plugins | P1 | ✅ | LANE_APP_TABS BranchWorkspace.tsx:34-40; 渲染 :126-214 (Canvas、BI 为跳转链接, 见 §6) |
| FR-01-15 | 面包屑显示当前 Project、Lane、Worktree、Branch, 并随选择变化 | P1 | ⬜ | kanban/page.tsx:205-223 为字面量 "LANE · auth-refactor"、"wt-relation-bfs"、"feat/relation-bfs"; 全部 src 中无其他面包屑 |
| FR-01-16 | 顶栏显示当前 Worktree 标识, 随选择变化 | P2 | ⬜ | AppHeader.tsx:36-43 读取构建期环境变量 NEXT_PUBLIC_WORKTREE_ID 等, 不读 navStore (AppHeader.tsx:82-98) |
| FR-01-17 | 选中 Worktree 时设置 CLI checkout 焦点 | P1 | 🟡 | setFocus 仅在 run 工作区调用: RunWorkspaceShell.tsx:132; 实现 runDirectorySession.tsx:97-100; 导航树点选不设置焦点 |
| FR-01-18 | 任务详情为同一可关闭子窗口, 并绑定当前 Worktree 上下文 | P1 | 🟡 | 可关闭子窗口 BranchWorkspace.tsx:56-57, 217-225; reporterId 写死为 "usr-002" (BranchWorkspace.tsx:223) |
| FR-01-19 | 侧栏展开 (256px) 与折叠 (64px), Ctrl/⌘+B 切换 | P1 | ✅ | Sidebar.tsx:42-45, 55-62, 85-95; Sidebar.test.tsx:91 |
| FR-01-20 | 侧栏两段结构: 层级树 + 底部全局组 (收件箱、Agent、分析) | P1 | ✅ | Sidebar.tsx:208-220; registry.ts:689; Sidebar.test.tsx:64; SidebarProjectWorktreeNavigation.test.tsx:121-130 |
| FR-01-21 | ui-3pane §1.2 五组侧栏 (Home 核心区、项目、视图、管理) | P1 | 🗑 | 提交 59cfe5b8 移除项目、视图、管理、其他入口组; Sidebar.tsx:64-67 注释; Sidebar.test.tsx:116-124 断言不渲染 |
| FR-01-22 | 模块添加入口: 6 个模块分组, 可增删顶栏页签 | P1 | ✅ | registry.ts:755-798; AppMatrixDrawer.tsx:229-256; AppHeader.tsx:117-125; navStore.ts:207-216 |
| FR-01-23 | 顶栏默认 Branch 功能页签: 任务、画布、协同、沙盒、原型 | P1 | ✅ | navStore.ts:94; AppHeader.tsx:52-59, 102-114; 默认集可被用户移除 (navStore.ts:212-216) |
| FR-01-24 | worktree 模块不进入侧栏与钉选 (由 Project Worktree Index 承载) | P2 | ✅ | navStore.ts:97, 175, 196; navStore.test.ts:218 |
| FR-01-25 | Project Worktree Index 页: 跨 Lane、Worktree、Branch 的汇总入口, 含授权 Project 选择 | P2 | ✅ | worktree/page.tsx:621-623; 授权选择 :636-645 (worktree/AuthorizedProjectSelector.tsx) |
| FR-01-26 | Group 页左侧双页签 (导航、AI 更新); 底部工作流状态筛选同时作用于两页签 | P1 | 🟡 | 页签 worktree/[id]/group/page.tsx:2627-2634; 导航树按状态分组 :2662-2680; AI 更新无数据源 :2688-2698; 共享筛选 :2701-2704 |
| FR-01-27 | 宿主会话下的目录: Project、LANE、Worktree、Branch 来自服务端 | P0 | 🟡 | 客户端 runDirectoryApi.ts:625-680; 服务端路由见 §4.3; 前端装配点 providers.tsx:31, 51; 根布局未传入 (layout.tsx:75), web 默认状态为 missing (runDirectorySession.tsx:49-52) |
| FR-01-28 | 会话缺失或失效时的目录降级, 不展示缓存或示例数据 | P0 | 🟡 | 降级树读取 store (ProjectRunTree.tsx:112-121), store 整体持久化 (store.ts:1674-1681); blocked 状态同样落入降级树 (ProjectRunTree.tsx:99-100); 文案要求不显示缓存 (zh-CN.ts:46) |
| FR-01-29 | 数据来源标识: 演示、未接入、授权投影三态 | P1 | 🟡 | 演示标识文案 zh-CN.ts:48 (与实际数据不符, 见 §6); 未授权 fail-closed worktree/page.tsx:578-602; 未授权任务投影提示 group/page.tsx:2681 |
| FR-01-30 | 授权 Project 切换 (宿主模式) | P1 | ✅ | worktree/AuthorizedProjectSelector.tsx; worktree/page.tsx:636-645; 服务端 GET /api/v1/projects (group_api/worktrees.rs:213) |
| FR-01-31 | Workspace 层级: Project 归属 Workspace, 顶栏切换 | P2 | ⬜ | AppHeader.tsx:78 硬编码 "ACME Studio"; 类型存在 (types/ids.ts:193) 但无切换与接线; 设计依据 domain-workspace-spec INV-W-01 |
| FR-01-32 | 树的键盘操作 (方向键展开收起、Home、End) | P2 | 🟡 | 宿主树 moveTreeFocus ProjectRunTree.tsx:315-342; 降级树无键盘处理 |
| FR-01-33 | 宿主树可见行窗口化 | P2 | ✅ | 切片 ProjectRunTree.tsx:300-301; ROW_HEIGHT、VIEWPORT_HEIGHT、OVERSCAN 定义于 :88-90; 降级树为普通列表 (:155) |
| FR-01-34 | 窄屏 (<768px) 通过抽屉访问项目目录 | P2 | ✅ | MobileBottomNav.tsx:79-122; 侧栏窄屏隐藏 Sidebar.tsx:114 |
| FR-01-35 | 侧栏范围切换 (主范围、项目范围) | P2 | ⬜ | 状态字段存在 (navStore.ts:53), 写入 ProjectRunTree.tsx:304; 无读取与切换 UI (Sidebar.tsx:198-199 注释无对应渲染) |
| FR-01-36 | 侧栏项目范围读取 SubNav 注册表 | P2 | 🟡 | 注册表 subNavRegistry.ts:78-118 已定义; 无消费者; subNavRegistry.ts:11 声明由侧栏读取 |
| FR-01-37 | 侧栏模块徽章显示真实计数 | P2 | 🟡 | 徽章取自静态 registry (registry.ts:180, inbox count 5); 渲染 Sidebar.tsx:253-257 |
| FR-01-38 | Canvas 范围选择 Project → Lane → Worktree 与导航状态一致 | P1 | 🟡 | 选择器 CanvasScopeSelector.tsx:45-101 写入 navStore; 未归类编码与其他位置不一致 (见 §6) |

## 4. 设计

### 4.1 概念模型

- 层级: Workspace (上层, 可选) → Project → Lane (管理层级) → Worktree (Lane 管理的本地 checkout) → Branch (Worktree 的本地 checkout ref 与状态)。依据: WBS §2; types/ids.ts:433-436、:462-467; domain-project-spec §1 与 INV-P-01 (Project 必属一个 Workspace)。
- Lane 与 engineering run: 宿主侧 engineering run 承载运行时事实, 宿主树以 "LANE" 标签呈现 (ProjectRunTree.tsx:445); 前端 Lane 实体与宿主 engineering run 的映射未定义, WBS §2 将其列为 backlog。
- 导航上下文: 四元组 (projectId, laneId, worktreeId, branch), 空字符串表示未选择。上层切换清空下层 (navStore.ts:237-244)。
- 视图入口: Lane 级中央应用 5 个页签 (BranchWorkspace.tsx:34-40); Project 视图页签 kanban、timeline、backlog、agents、worktrees 经 /projects?tab= 进入 (registry.ts:282, 292; ProjectsClient.tsx:132); 看板独立为 /kanban (registry.ts:269-272)。

### 4.2 数据模型

| 实体 | 字段 (代码定义) | 来源 | 持久化 |
|---|---|---|---|
| Lane | id、tenant_id、project_id、name、git_remote_url?、git_provider? (github / gitlab / gitee)、default_branch?、created_at | types/ids.ts:437-447 | store.lanes, 随 star-store:v1 持久化 (store.ts:1675-1681) |
| Worktree (前端) | id、tenant_id、project_id、lane_id?、name、branch、base_branch、status (17 态 WorktreeStatus)、local_runtime_id?、agent_session_id?、pr_id?、lock_version、last_event_at、created_at | types/ids.ts:452-478 | store.worktrees |
| Project (前端) | 类型存在 | types/ids.ts:153 | store.projects |
| Workspace (前端) | 类型存在, 无导航接线 | types/ids.ts:193 | store.workspaces |
| 导航状态 NavState | sidebarItemIds、pinnedViewIds、headerTabIds、isMatrixOpen、sidebarFold (expanded / collapsed)、sidebarScope (main / project)、selectedProjectId、selectedLaneId、selectedWorktreeId、selectedBranch | navStore.ts:29-59 | localStorage star-nav-store:v2, version 9 (navStore.ts:258-262) |
| 宿主目录条目 | AuthorizedRunProject、EngineeringRun、RunWorktree、RunContextEnvelope | runDirectoryApi.ts (类型); 使用见 ProjectRunTree.tsx:12, 80 | 内存, 不持久化 |
| Project 域设计字段 | project_id、tenant_id、workspace_id、slug、status 等 | domain-project-spec §2 | 设计依据; 后端接线 ❓ (本文未核实) |

### 4.3 接口

前端路由:

| 路由 | 文件 | 用途 | 状态 |
|---|---|---|---|
| /worktree | frontend/src/app/worktree/page.tsx | Project Worktree Index; 接收 project_id、lane_id、worktree_id、branch | ✅ |
| /worktree/[id]/group | frontend/src/app/worktree/[id]/group/page.tsx | Group Workspace, 左侧双页签 | 🟡 |
| /projects/[project_id]/runs/[engineering_run_id] | frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/page.tsx | 宿主 run 概览 (RunWorkspaceShell) | ✅ |
| .../worktrees/[worktree_id] | frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/worktrees/[worktree_id]/page.tsx | run 内 Worktree 页 | ✅ |
| /kanban | frontend/src/app/(app)/kanban/page.tsx | 看板; 面包屑为字面量 (:205-223) | ✅ |
| /projects?tab= | frontend/src/app/(app)/projects/ProjectsClient.tsx:132 | Project 视图页签 | ✅ |
| /canvas/[id]、/canvas/worktree-graph | frontend/src/app/canvas/[id]/page.tsx、frontend/src/app/canvas/worktree-graph/page.tsx | 画布; 作用域由 navStore 解析 (canvas/[id]/page.tsx:111-112) | 🟡 |
| 旧路由重定向 | frontend/src/lib/redirects.ts:61 (/board → /kanban)、:66-82 (/scm、/collaboration、/relation → /projects?tab=worktrees) | 兼容旧入口 | ✅ |

宿主 HTTP 接口 (客户端 frontend/src/lib/run/runDirectoryApi.ts; 服务端 crates/star-api-rest/src/group_api/):

| 方法与路径 | 客户端 | 服务端 |
|---|---|---|
| GET /api/v1/projects | runDirectoryApi.ts:625-626 | group_api/worktrees.rs:213 |
| GET /api/v1/projects/{id}/worktrees | worktree/page.tsx:440 (listProjectWorktrees) | group_api/worktrees.rs:214-217 |
| GET /api/v1/projects/{id}/members | worktree/page.tsx:494 | group_api/worktrees.rs:218-221 |
| GET /api/v1/projects/{id}/branches | runDirectoryApi.ts:663 | group_api/engineering_runs.rs:276 |
| GET、POST /api/v1/projects/{id}/engineering-runs | runDirectoryApi.ts:628-630, 637-650 | group_api/engineering_runs.rs:277-280 |
| GET /api/v1/branches/{id}/engineering-runs | runDirectoryApi.ts:669 | group_api/engineering_runs.rs:282-284 |
| GET /api/v1/engineering-runs/{id}/worktrees | runDirectoryApi.ts:676-677 | group_api/engineering_runs.rs:286-288 |
| GET /api/v1/engineering-runs/{id}/context | runDirectoryApi.ts:923-927 | group_api/engineering_runs.rs:290-292 |
| GET /api/v1/worktrees/{id}/run-context | runDirectoryApi.ts:935 | group_api/engineering_runs.rs:294-296 |

路由挂载: crates/star-api-rest/src/group_api.rs:796-817 组合各子路由; crates/star-api-rest/src/main.rs:133 合并进服务; main.rs:87-89 启动时创建 PostgreSQL 连接池。

Lane 的创建、列出、授权接口: 无 (⬜)。

导航 store 动作 (navStore.ts:61-85): setSelectedProjectId (:81)、setLaneContext (:83)、setSidebarScope (:80)、toggleSidebarFold (:78)、setSidebarFold (:79)、addHeaderTab 与 removeHeaderTab (:73-75)。

### 4.4 状态机与关键流程

导航上下文转移:

| 事件 | 触发位置 | 结果 |
|---|---|---|
| 选择 Project (降级树) | ProjectRunTree.tsx:135-138 | selectedProjectId 设为该 Project; sidebarScope 为 project; 若 Project 变化则清空 lane、worktree、branch (navStore.ts:240-244) |
| 选择 Project (宿主树) | ProjectRunTree.tsx:302-305 | 同上 |
| 点选 Lane (降级树) | ProjectRunTree.tsx:141-143, 189 | setLaneContext(lane, "", "") |
| 点选 Worktree (降级树) | ProjectRunTree.tsx:145-147, 209, 243 | 链接带 lane、worktree、branch 参数, 并 setLaneContext |
| 点选 Worktree (宿主树) | ProjectRunTree.tsx:507-510 | 仅 clearFocus; 不写入 navStore |
| 打开 /worktree?... | worktree/page.tsx:530-536 | URL 参数覆盖 store 中的 lane、worktree、branch |
| 切换 Lane 页签 | BranchWorkspace.tsx:55, 111 | 本地 useState, 不写入 store, 刷新后重置 |

宿主会话状态: missing、ready、blocked (runDirectorySession.tsx:41-42, 102-103)。宿主树仅在 ready 且存在 client 时渲染 (ProjectRunTree.tsx:99-100), 其余状态渲染降级树。

侧栏折叠: expanded (256px) 与 collapsed (64px) 互切 (navStore.ts:29; Sidebar.tsx:42-45, 85-95)。

### 4.5 权限与安全

- 宿主会话: 访问令牌经 getAccessToken 注入 (runDirectorySession.tsx:66-72); 授权失败触发 onAuthorizationError, 并清空选中焦点 (runDirectorySession.tsx:73-76, 97-107); generation 做格式校验 (runDirectorySession.tsx:67-68)。
- 导航状态不作为授权依据: 授权结果来自服务端过滤。Canvas 作用域明示"请求后端权限复验", 本地预览未接入后端时如实标注 (canvas/[id]/page.tsx:149)。
- 持久化: 导航选择与整个 store (含 projects、worktrees、lanes) 写入 localStorage, 未按用户或租户隔离 (navStore.ts:263-278; store.ts:1675-1681)。共享浏览器会残留他人目录的缓存, 与 FR-01-28 相关。
- 写操作: 宿主树的 "+" 新建 run 按钮 (ProjectRunTree.tsx:489-491) 仅依赖宿主 ready 状态; 无 Lane 维度的权限判断。

### 4.6 依赖关系

- 框架: Next.js App Router (usePathname、useSearchParams: Sidebar.tsx:21, worktree/page.tsx:380); zustand persist (navStore.ts:20-21; store.ts:22)。
- 导航守卫: GuardedLink 包装未保存状态确认 (GuardedLink.tsx:22-37)。
- 宿主会话: RunDirectoryProvider (runDirectorySession.tsx:56), 由 Providers 注入 (providers.tsx:31, 51)。
- 数据 store: projects、worktrees、lanes 在 store.ts:177, 402-411, 434 定义为空初始值; lib/seed.ts 仅被 dmStore.ts、globalSearch.ts 及测试引用, store 不引入。
- 视图: BranchWorkspace 依赖 SprintBoardView 与 WorkItemDetailDrawer (BranchWorkspace.tsx:29-30); 画布作用域依赖 CanvasScopeSelector (CanvasScopeSelector.tsx:45)。
- 注册表: registry.ts (ALL_MODULES、NAV_GROUPS、MODULE_GROUPS)、activeMatch.ts 的 moduleHrefMatches (activeMatch.ts:15-26)。
- 后端: star-api-rest group_api, 启动依赖 PostgreSQL (main.rs:87-89)。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 导航 store | frontend/src/lib/nav/navStore.ts | ✅ | 选择状态 :55-59; 清空规则 :237-244; 持久化 :258-278 |
| 侧栏容器 | frontend/src/components/Sidebar.tsx | 🟡 | 两段结构 :208-220; 折叠 :42-95; 未渲染的历史组件 :274-569 |
| 降级层级树 | frontend/src/components/run/ProjectRunTree.tsx (FallbackProjectRunTree) | 🟡 | 四层渲染 :153-271; 数据来自 store :112-121 |
| 宿主层级树 | frontend/src/components/run/ProjectRunTree.tsx (AuthenticatedProjectRunTree) | 🟡 | :273-406; 节点 :443-535; 点选 Lane 与 Worktree 未接线 :308-314 |
| 宿主运行目录客户端 | frontend/src/lib/run/runDirectoryApi.ts | ✅ | 接口 :625-680; 规范 href :941-947 |
| 宿主会话 provider | frontend/src/lib/run/runDirectorySession.tsx | ✅ | 状态 :41-48, 102-103; 装配点 providers.tsx:31, 51 |
| Lane 类型与 store | frontend/src/types/ids.ts; frontend/src/lib/store.ts | 🟡 | 类型 types/ids.ts:437-447; store.ts:177, 434; 无创建 action |
| Worktree 类型 | frontend/src/types/ids.ts | ✅ | :462-478 |
| Worktree 页 | frontend/src/app/worktree/page.tsx | 🟡 | 深链同步 :530-536; 不可用分支 :578-611; 主视图 :613-637 |
| Group 页 | frontend/src/app/worktree/[id]/group/page.tsx | 🟡 | Lane 取自 worktree.lane_id :999-1003; 双页签 :2627-2698 |
| Lane 级中央应用 | frontend/src/components/run/BranchWorkspace.tsx | ✅ | 5 页签 :34-214; 子窗口 :217-225 |
| 顶栏 | frontend/src/components/AppHeader.tsx | 🟡 | 页签来自 store :52-59; Worktree 标识读环境变量 :36-43, :82-98; 工作区硬编码 :78 |
| 模块注册表 | frontend/src/lib/nav/registry.ts | ✅ | ALL_MODULES; NAV_GROUPS :684-714; MODULE_GROUPS :755-798 |
| 模块添加抽屉 | frontend/src/components/nav/AppMatrixDrawer.tsx | ✅ | 分组映射 :229-231; 页签切换 :256 |
| 路由 active 判定 | frontend/src/lib/nav/activeMatch.ts | ✅ | :15-26 |
| 子导航注册表 | frontend/src/lib/nav/subNavRegistry.ts | 🟡 | 注册 :78-118; 无消费者 |
| 画布范围选择 | frontend/src/components/canvas/CanvasScopeSelector.tsx | 🟡 | :45-101; 未归类编码 :32 |
| 看板面包屑 | frontend/src/app/(app)/kanban/page.tsx | ⬜ | 字面量 :205-223 |
| 移动端目录 | frontend/src/components/MobileBottomNav.tsx | ✅ | :79-122 |
| 旧路由重定向 | frontend/src/lib/redirects.ts | ✅ | :61, :66-82 |
| 服务端目录接口 | crates/star-api-rest/src/group_api/engineering_runs.rs; worktrees.rs | ✅ | 路由 engineering_runs.rs:276-296; worktrees.rs:213-221; 挂载 main.rs:133 |
| Lane 后端实体与接口 | crates/ | ⬜ | 无 Lane 相关 Rust 类型或路由 (检索 lane_id、Lane 无命中) |
| 后端 domain-project、domain-workspace | crates/domain-project、crates/domain-workspace | ❓ | 目录存在; 与导航的接线与不变量实现未核实 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| Lane 无创建、授权、持久化的写入路径 (FR-01-05、FR-01-07) | 无远端身份的 Lane 无法被创建与管理, 不满足 WBS §1 Phase 1 与需求一.2 | 决定 Lane 的归属域 (domain-project 或 domain-worktree); 提供创建与列出接口; 在 store 中补写入 action 并持久化 |
| 宿主树的 Lane、Worktree 点选不写入导航上下文 (ProjectRunTree.tsx:308-314 未接线) | 宿主模式下选择不驱动中央应用、画布与面包屑 | 把 selectLane、selectWorktree 接入 DirectoryRow 的 onNavigate; 补单测 |
| /worktree 页的 URL 到 store 同步在依赖变化时执行 (worktree/page.tsx:530-537, 依赖含 selectedLaneId 等) | 静态推断: 在该页直接点选 Lane 会被回写为 URL 中的值, 点选结果不保留 (❓ 未在浏览器运行核实) | 在浏览器或 e2e 中复现; 将 URL 同步改为仅随查询参数变化触发, 或让 Lane 点选同时更新 URL |
| 降级树读取持久化的 store, 且 blocked 状态也落入降级树 (ProjectRunTree.tsx:99-100; store.ts:1674-1681) | 会话失效后仍显示缓存目录与可点击链接, 与 runDirectorySession.tsx:103 的"目录已清除"文案不一致 | blocked 状态渲染 fail-closed 面板而非降级树; 降级树只用于 missing 的演示态; 授权类数据不进入持久化 |
| 降级树文案称"显示 seed 层级" (zh-CN.ts:48; en.ts:37; ja.ts:38), 但 store 初始为空 (store.ts:402-411) 且 seed 不被引入 (提交 5864582b 起) | 默认安装下降级树为空, 与文案不符 | 二选一: 以 mocks 明确注入演示数据并标注, 或修改文案为空态提示 |
| 面包屑为字面量 (kanban/page.tsx:205-223) | 面包屑与真实选择不一致, 违反 WBS §1.5 | 绑定 navStore 选择器, 显示 Lane 与 Worktree 名称 |
| 顶栏 Worktree 标识读构建期环境变量 (AppHeader.tsx:36-43) 且工作区硬编码 "ACME Studio" (AppHeader.tsx:78) | 标识与选择不同步; 无 Workspace 层级 | Worktree 标识绑定 navStore; 增加 Workspace 选择并接线 domain-workspace 数据 |
| CLI checkout 焦点只在 run 工作区设置 (RunWorkspaceShell.tsx:132) | 导航选择与 CLI 焦点可能不一致 | 定义焦点规则: 导航点选是否调用 setFocus, 并补测试 |
| 未归类 Lane 的编码不统一: 空字符串 (ProjectRunTree.tsx:243; canvas/[id]/page.tsx:112) 与 "__unclassified" (CanvasScopeSelector.tsx:32) | 同一 Worktree 在不同组件中选中状态不同 | 在导航模块导出单一常量并统一使用 |
| Group 页 Lane 取自 worktree.lane_id (group/page.tsx:999-1003), 侧栏取自 navStore | 同页存在两个 Lane 来源, 优先级未定义 | 明确优先级: 以 worktree 的 lane_id 为准, 并让侧栏同步 |
| Group 页左侧树与侧栏层级树为两套实现 (group/page.tsx:2636-2686 与 ProjectRunTree.tsx) | 视觉与行为易漂移 | 抽取共享组件, 或在设计中明确两者职责 |
| 子导航注册表无消费者 (subNavRegistry.ts:11 声明由侧栏读取) | 注释与实际不符; 项目范围切换未实现 | 删除注册表, 或接入侧栏项目范围 (FR-01-35、FR-01-36 一并决定) |
| 侧栏范围字段无读写 UI (navStore.ts:53; Sidebar.tsx:198-199) | 死状态 | 决定保留并实现切换, 或删除字段 |
| Sidebar.tsx 中未渲染的历史组件 (NavModuleList、NavGroupSection、SidebarRow、EmptyProjectState, :274-569) | 维护负担, 误导阅读者 | 删除未使用的组件 |
| 侧栏徽章为静态值 (registry.ts:180) | 徽章不反映真实计数 (ui-3pane §1.2 要求真实计数) | 绑定各模块的真实计数来源 |
| 宿主树默认未挂载: 根布局不传入 runDirectorySession (layout.tsx:75; providers.tsx:31) | web 默认只能看到降级树; FR-01-27 无法在默认环境验证 | 明确宿主装配方式 (桌面宿主或受信来源), 并在文档中标注 |
| BranchWorkspace 的 BI 与 Canvas 页签为跳转链接 (BranchWorkspace.tsx:138-149, 180-193) | 与"Lane 级应用"的内嵌设计不一致 | 决定内嵌或保留跳转, 并在设计中记录 |
| BranchWorkspace 的任务详情写死 reporterId "usr-002" (BranchWorkspace.tsx:223) | 操作人身份与登录态不一致, 影响授权上下文 | 改为取自宿主会话的身份 |
| AI 更新页签无数据源 (group/page.tsx:2693-2696) | 页签为占位 | 接入 Group Chat transcript 后再验收 (本文范围外的数据源) |

## 7. 验收与测试

| 测试 | 位置 | 覆盖 | 未覆盖 |
|---|---|---|---|
| ProjectRunTree 行为与本地化 | frontend/src/components/run/ProjectRunTree.test.tsx:114, 128, 146, 160-174 | 选择 Project 写入项目范围并清除焦点; 展开 Project 与 Lane 不选中; 方向键展开; 三语降级树渲染 | 点选 Lane 与 Worktree 写入; 宿主树 401 行为; 降级树数据来源 |
| 侧栏结构与折叠 | frontend/src/components/__tests__/Sidebar.test.tsx:64, 91, 103, 116, 129 | 两段结构; 折叠控件; 全局组恒定; 不渲染钉选与视图模块; 精确路径激活 | Lane 与 Worktree 树节点 |
| 侧栏与 Worktree 导航 | frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx:100, 132 | 全局组; 宿主 Worktree 行的生命周期与绑定标签 | 第 100 行名称声称"Worktree 控制中心位于核心区", 断言实际覆盖全局组, 名称与断言不一致 |
| 导航 store | frontend/src/lib/nav/__tests__/navStore.test.ts:21-218 (16 个用例) | 默认值; 侧栏与顶栏增删; 折叠与范围; 持久化与迁移; 拒绝 worktree 进入侧栏 | Lane 与 Worktree 上下文写入; Project 切换清空下层 |
| 模块注册表 | frontend/src/lib/nav/__tests__/registry.test.ts:22-141 (13 个用例) | 域色卡; 模块覆盖; MODULE_MAP 唯一性 | NAV_GROUPS 与 MODULE_GROUPS 的映射 |
| 画布范围选择 | frontend/src/components/canvas/CanvasScopeSelector.test.tsx:29-168 | Project、Lane、Worktree 选择写入; 未归类 Lane 分组 | 与宿主树的一致性 |
| 顶栏与外壳 | frontend/src/components/__tests__/AppHeader.test.tsx; AppShell.test.tsx | 顶栏与外壳渲染 | Worktree 标识随选择变化 |
| 路由重定向 e2e | frontend/e2e/redirects.spec.ts:110 | 旧路由重定向与画布上下文解析 | 导航树交互 |

- 未覆盖的关键验收点: 深链同步在 /worktree 页的行为 (无页面级测试, 无 app/worktree/page.test 文件); Lane 级中央应用页签 (无 BranchWorkspace 测试文件); 宿主树的 Lane 与 Worktree 点选; 降级态与 401 态的数据隔离; Group 页的双页签与筛选。
- 本文未运行任何测试或构建, 通过状态 ❓; WBS §0 中的测试数字 (vitest 128·1064) 为声明值, 未复核。
- 关键词检索 frontend/e2e 下仅 redirects.spec.ts 与 remote-mobile.spec.ts 命中导航相关关键字, 未发现覆盖导航树点选的端到端用例。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md | 分阶段状态 (§1 Phase 1 至 8); 关键决策 Lane 本地实体与 engineering_run 内部保留 (§2); 调查结论 (§0) | §0 写 /worktree 页 1279 行, 当前 1296 行; Phase 1.4、1.5 与 2.5 标注已完成, 但宿主树点选、面包屑与切换验证未实现 (见 FR-01-11、FR-01-15); Phase 打钩表示历史完成声明 | 状态以代码核对为准; 打钩项按代码结果重判为 🟡 或 ⬜; §2 的命名与兼容决策采用 |
| docs/architecture/2026-11-15-lane-workspace/REVIEW-2026-11-14-IMPLEMENTATION.md | §1.5 中 Lane Phase 1 与布局整合的闭环声明 | 审查基线为 cfbfe4c5, 未对当前代码逐项复核; 其余条目属于其他 feature | 仅作历史证据, 不作为状态来源; 与 F01 相关的只有 §1.5 声明 |
| docs/specs/domain-project-spec.md | Project 聚合标识与字段 (§2); INV-P-01 Project 必属 Workspace (§3); 角色与 Permission (§7) | 规范描述后端 crate 与 API, 前端导航未接线; 本文未核实 domain-project 的实现 | 采用为设计依据 (FR-01-31、§4.1、§4.2); 后端状态标 ❓ |
| docs/specs/domain-workspace-spec.md | Workspace 层级与成员 (§2); INV-W-01 (§3) | AppHeader 硬编码 "ACME Studio" (AppHeader.tsx:78); 无 Workspace 切换 | 采用为设计依据; Workspace 切换标 ⬜ (FR-01-31) |
| docs/frontend/design/ui-3pane-arch.md | 三栏布局 (§1.1); 侧栏两段结构 (§0.2 第 3 条); 顶栏固定页签 (§0.2 第 2 条); 视图族 (§1.3); 面板 (§1.4-1.5) | §1.2 五组侧栏与 §1.2.1 路由映射与代码不符 (由提交 59cfe5b8 取代); §0.2 引用的 frontend/src/app/projects/page.tsx:78 已迁至 (app)/projects/ProjectsClient.tsx:132; "顶栏固定页签" 与代码中用户可增删的实现不同 | 采用 §0.2 (2026-11-15 更新) 作为当前 IA; §1.2、§1.2.1 标 🗑 (FR-01-21); 顶栏采用默认集 + 可删除 (FR-01-23), 理由: 代码为状态权威, AppHeader.tsx:101 注释明确用户可增删 |
| docs/frontend/design/ui-redesign-multica-style.md | §0.1 五页签与 4 处重定向修复; §3 顶栏 | §0.1 的 /board 目标写为 /projects?tab=kanban, 当前为 /kanban (redirects.ts:61, 2026-11-17 去重); §2 的 6 路由与 §4 的 180px SubNav 设计未被当前侧栏采用 | 重定向以 redirects.ts 为准; 6 路由与 SubNav 设计标 🗑 (未采用) |
| docs/frontend/design/ui-detailed-design.md | §3 AppShell 结构 (Sidebar 宽 220px) | 代码为 w-64 (256px, Sidebar.tsx:43) | 采用代码值 256px; §5 页面规格属于其他 feature, 未采用 |
| git 历史 (非文档): 462260e8、59cfe5b8、5864582b、55728d52 | 462260e8 引入 Lane 类型、store.lanes、四层选择状态与深链; 59cfe5b8 将侧栏收敛为 LANE 树与全局组; 5864582b 将 store 初始态改为空并保留 seed 为离线 fixture; 55728d52 切换 Project 清空下层与严格 lane 过滤 | 无 | 作为状态变化的时间依据 (FR-01-21、§6 seed 文案项) |
| frontend/src/lib/i18n/zh-CN.ts、en.ts、ja.ts (文案, 非设计文档) | 降级与不可用文案 (zh-CN.ts:46, 48) | 文案称显示 seed 层级, 与 store 空初始态不符 | 文案标为待修正 (§6); 代码行为为准 |

源文档均已读取。REVIEW-2026-11-14 中与 F01 无关的条目 (star-api-rest 测试计数、OPS-CONSOLE 等) 未采用。
