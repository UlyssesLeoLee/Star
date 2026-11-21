# F01 项目·Lane·Worktree 导航与层级

> 状态摘要: 导航上下文模型 (`navStore` 的 Project / Lane / Worktree / Branch 字段)、Lane 与 Worktree 类型、侧栏 Project→Lane→Worktree 树的 seed 降级渲染、URL 深链同步与 Canvas 范围选择器已在代码中实现。认证路径下的目录树仍把 engineering run 显示为 "LANE"、不渲染 `lanes` 数据; Lane 的创建动作、后端持久化与授权、看板面包屑的动态数据未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 建立统一的导航层级 Project → Lane → Worktree → Branch, 使用户在侧栏选择 Project 后可按 Lane 展开, 再选择 Worktree 进入同一个授权上下文。
- Lane 是 Project 下的管理层级, 本地可创建、授权、列出和管理; 远端仓库、分支、provider 绑定是可选元数据, 缺失时在界面显式显示"未绑定"。
- 面包屑、中央应用、任务卡、Canvas 范围与 CLI checkout 焦点共享同一份导航上下文 (`navStore`), 切换 Project 时清空下级选择。
- 对外界面使用 "Lane" 命名; 内部 engineering run 契约 (路由、ID、API) 在迁移完成前保留, 界面不得据此假定迁移已完成。

### 1.2 范围内 / 范围外

| 范围内 | 范围外 (由其他功能拥有) |
|---|---|
| 侧栏导航树 (Project / Lane / Worktree / Branch 四层) 与折叠态 | Lane 内中央应用的 Backlog、Canvas、Workflow、BI、Plugins 页签 (F02/F06 等) |
| `navStore` 导航上下文字段、持久化与切换规则 | 任务卡子窗口与看板内容 (看板相关功能) |
| `/worktree` 页面的 URL 深链与上下文同步 | Canvas 画布内容与关系图 (F06/F07) |
| Lane 设置页的路径配置、统计、Git 批量入口的结构 | Git 操作的执行后端与冲突处理 (F03 等) |
| Project/Lane/Worktree 的前端类型与 seed 数据 | 后端 Project/Workspace 聚合与 RLS (见 §4.2 与 §6) |

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 开发者 / Agent 操作者 | 在侧栏展开 Project, 选择 Lane, 再点击 Worktree 进入其 Branch 工作区, 查看状态与 Agent 会话。 |
| 项目负责人 | 在 Lane 设置页查看 Lane 下的 Worktree 数量、统计, 并对 Lane 下的 Worktree 发起批量 Git 操作入口。 |
| 任何用户 | 通过深链 (`/worktree?project_id=…&lane_id=…&worktree_id=…&branch=…`) 或刷新页面恢复上下文; 切换 Project 后下级选择被清空。 |
| 运维 / 无宿主会话用户 | 在无宿主连接时看到 seed 降级目录, 且被明确告知这是降级视图。 |

## 3. 需求

优先级: P0 = 导航层级与上下文的最小闭环; P1 = 授权与一致性; P2 = 可读性与性能增强。状态取值见文首约定。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-01-01 | 侧栏渲染 Project → Lane → Worktree → Branch 四层树 | P0 | 🟡 | seed 降级路径渲染 Project、Lane、Worktree 三层 (`frontend/src/components/run/ProjectRunTree.tsx:180-222`); 认证路径的 Branch 行 (`:533-534`) 为只读, Lane 不作为独立层渲染 (`:518-532`, `:445`) |
| FR-01-02 | Lane 实体: id、tenant_id、project_id、name, 远端字段可选 | P0 | ✅ | `frontend/src/types/ids.ts:437-447`; store 集合 `frontend/src/lib/store.ts:177, 434` |
| FR-01-03 | Lane 无远端绑定时可创建、授权、列出、管理 | P0 | 🟡 | 列出与"未绑定"标记 ✅ (`ProjectRunTree.tsx:194-197`); 创建动作 ⬜ (store 无 `addLane`/`setLanes` 动作, 仅 seed 写入 `frontend/src/lib/seed.ts:199-201`); 授权 ⬜ |
| FR-01-04 | Worktree 可选引用 lane_id; 未归类 Worktree 归入"未归类"分组 | P0 | ✅ | `frontend/src/types/ids.ts:467`; 未归类分组 `ProjectRunTree.tsx:223-256`; seed 中 wt-001 等 `frontend/src/lib/seed.ts:205` |
| FR-01-05 | 导航上下文字段 selectedProjectId / selectedLaneId / selectedWorktreeId / selectedBranch 并持久化 | P0 | ✅ | `frontend/src/lib/nav/navStore.ts:55-59`; 持久化 `:259-278` |
| FR-01-06 | 切换 Project 时清空 Lane、Worktree、Branch | P0 | ✅ | `frontend/src/lib/nav/navStore.ts:240-243`; 测试 `frontend/src/components/canvas/CanvasScopeSelector.test.tsx` 用例 5 |
| FR-01-07 | 点选 Lane 清空 Worktree 与 Branch | P0 | ✅ | `frontend/src/components/run/ProjectRunTree.tsx:141-143` (fallback), `:308-310` (认证路径); 测试 `CanvasScopeSelector.test.tsx` 用例 7 |
| FR-01-08 | 点选 Worktree 一次性设置 Lane、Worktree、Branch | P0 | ✅ | `ProjectRunTree.tsx:145-147, 312-314`; 链接构造 `:207` |
| FR-01-09 | URL 深链 lane_id、worktree_id、branch 与导航上下文双向同步 | P0 | 🟡 | 读取与写入 `frontend/src/app/worktree/page.tsx:396, 530-537`; 深链 `ProjectRunTree.tsx:207, 241`; 其他页面 (看板面包屑) 不读取 URL 上下文 |
| FR-01-10 | 面包屑、中央应用、任务详情、CLI checkout 焦点同一上下文 | P0 | 🟡 | Canvas 读取 `frontend/src/app/canvas/page.tsx:35-36, 62-72, 106-109`; 看板面包屑硬编码 `frontend/src/app/(app)/kanban/page.tsx:208, 212`; CLI 焦点未在 `navStore` 中出现 (❓ 需在 CLI 模块核对) |
| FR-01-11 | 绑定信息缺失时显式显示"未绑定"状态 | P0 | ✅ | `ProjectRunTree.tsx:195-197` |
| FR-01-12 | Lane 设置页: 路径配置、Lane 统计、Git 批量操作 | P1 | 🟡 | 页面与标签 `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx:481`; 批量按钮 `:602-611`; 路径配置为空占位, 不持久化 `:344-348` |
| FR-01-13 | 侧栏底部全局组与顶部项目树分区 | P1 | ✅ | `frontend/src/components/Sidebar.tsx:208-219`; `data-testid="sidebar-group-global"` `:213` |
| FR-01-14 | 侧栏折叠态仅显示图标, 展开态 256px | P2 | ✅ | `frontend/src/components/run/ProjectRunTree.tsx:95`; `frontend/src/app/globals.css:56, 60` (`--sidebar-w`); `navStore.ts:162` |
| FR-01-15 | Project/Lane/Worktree 后端聚合、RLS 与授权复验 | P1 | ⬜ | 后端 crate 中无 Lane 实体与 Lane 路由 (见 §4.3 与 §5); spec 中 Project 与 Workspace 不含 Lane (`docs/specs/domain-project-spec.md` §2, `docs/specs/domain-workspace-spec.md` §2) |
| FR-01-16 | 数据来源标识: 降级 seed 视图明确标注非真实数据 | P1 | 🟡 | 降级提示 `ProjectRunTree.tsx:154` (`t.runDirectory.fallbackHint`) 与空 Lane 占位 `:257-261`; 认证路径无对应标注 |
| FR-01-17 | 侧栏 scope (main / project) 切换 | P2 | 🟡 | 状态 `navStore.ts:53, 80, 235` 与 `ProjectRunTree.tsx:137`; 注释标注"Scope Toggle"的分区 `Sidebar.tsx:198-199`, 其后未见对应控件 (`:201-219`) |
| FR-01-18 | 认证路径 engineering run 以 Lane 语义标注, 与内部契约保持兼容 | P1 | 🟡 | 标签 `ProjectRunTree.tsx:445`; 内部路由保留 `:519` (`canonicalRunHref`) 与 `frontend/src/app/(app)/projects/[project_id]/runs/[engineering_run_id]/page.tsx`; 兼容映射 (WBS §2) 未在代码中单独实现 |

## 4. 设计

### 4.1 概念模型

```
Tenant
 └─ Project            (navStore.selectedProjectId; types/ids.ts:153)
     ├─ Lane           (管理层级, 可无远端绑定; types/ids.ts:437)
     │    └─ Worktree  (本地 checkout, lane_id 可选; types/ids.ts:462-467)
     │         └─ Branch (Worktree 的本地 checkout ref; 不冒充远端 Branch)
     └─ 未归类 Worktree (lane_id 缺失, 显示在"未归类"分组)
```

- 界面层级名称固定为 Project → Lane → Worktree → Branch。Lane 不是远端仓库的别名, 远端字段 `git_remote_url`、`git_provider`、`default_branch` 为可选元数据 (`types/ids.ts:443-445`)。
- Worktree 属于 Lane 管理的本地 checkout。Worktree 的 `lane_id` 为可选, 历史数据缺失时归入未归类分组。
- 内部 engineering run 与 Lane 的映射是界面层标签 (`ProjectRunTree.tsx:445`), 不是数据迁移结果。

### 4.2 数据模型

| 实体 | 字段 (前端类型) | 来源 | 持久化 |
|---|---|---|---|
| Lane | `id, tenant_id, project_id, name, git_remote_url?, git_provider?, default_branch?, created_at` (`frontend/src/types/ids.ts:437-447`) | seed 3 条 (`frontend/src/lib/seed.ts:199-201`) | 前端 store 内存 (`store.ts:177, 434`); 无后端表 |
| Worktree | `id, tenant_id, project_id, lane_id?, name, branch, base_branch, status, local_runtime_id?, agent_session_id?, pr_id?, lock_version, …` (`ids.ts:462-475`) | seed (`seed.ts:205-216`) | 前端 store |
| 导航上下文 | `selectedProjectId, selectedLaneId, selectedWorktreeId, selectedBranch`, 另含 `sidebarFold, sidebarScope` (`navStore.ts:39-59`) | 用户操作与 URL | localStorage, key `star-nav-store:v2`, version 9 (`navStore.ts:259-261`) |

后端 Project、Workspace、Worktree 的领域 spec 与 crate 中未出现 Lane 实体 (`docs/specs/domain-project-spec.md` §2、`docs/specs/domain-workspace-spec.md` §2; crate 检索无 lane 命中)。Lane 的表结构、RLS 与索引不存在于代码中。

### 4.3 接口 (routes, APIs, ports)

| 类别 | 名称 | 位置 | 说明 |
|---|---|---|---|
| 页面路由 | `/worktree` | `frontend/src/app/worktree/page.tsx` | 读取 `project_id`、`lane_id`、`worktree_id`、`branch` 查询参数 (`:394-396`) |
| 页面路由 | `/lane/[laneId]/settings` | `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx` | Lane 设置 |
| 页面路由 | `/projects` (Project 视图的 tab) | `frontend/src/app/(app)/projects/page.tsx` | 由 `ProjectRunTree.tsx:156-170` 的链接进入 |
| REST | 无 Lane 端点 | `crates/star-api-rest/src` 检索无 lane 路由 | ⬜ |
| REST | engineering run 相关端点 | `crates/star-api-rest/src/group_api.rs:813` (`engineering_runs::router()`); `group_api/bi_analytics.rs:63` (`/api/v1/engineering-runs/{engineering_run_id}/bi/task-events`) | 内部契约, 仍按 run 命名 |
| 前端 port | `RunDirectoryApiClient` (`createEngineeringRun`, `canonicalRunHref`) | `frontend/src/lib/run/runDirectoryApi.ts`, 使用于 `ProjectRunTree.tsx:80, 367` | 认证路径的目录数据来源 |
| 前端 store | `useNavStore`: `setSelectedProjectId`, `setLaneContext(laneId, worktreeId, branch)`, `setSidebarScope` | `frontend/src/lib/nav/navStore.ts:81-83, 237-246` | 导航上下文 |
| 前端 store | `useStore`: `lanes`, `worktrees`, `projects` | `frontend/src/lib/store.ts:177` | seed 与内存数据 |

### 4.4 状态机与关键流程

**导航上下文转换** (`navStore.ts`):

| 当前状态 | 事件 | 结果 | 证据 |
|---|---|---|---|
| 任意 | `setSelectedProjectId(id)` 且 id 不同 | 清空 selectedLaneId、selectedWorktreeId、selectedBranch | `navStore.ts:240-243` |
| 任意 | `setLaneContext(lane, "", "")` (点选 Lane) | 仅保留 Lane | `ProjectRunTree.tsx:141-143` |
| 任意 | `setLaneContext(lane, wt, branch)` (点选 Worktree) | 三者同时设置 | `ProjectRunTree.tsx:145-147`, `:312-314` |
| URL 查询参数变化 | `/worktree` 页 effect | 若与 store 不同则调用 `setLaneContext` 同步 | `worktree/page.tsx:530-537` |
| 页面挂载后 | persist rehydrate | `migratePersistedNavState` 修正缺失字段为空串 | `navStore.ts:112-153` |

**侧栏树数据流**:

1. `Sidebar` 在折叠态与展开态渲染 `ProjectRunTree` (`Sidebar.tsx:210`)。
2. `ProjectRunTree` 读取 `useRunDirectory()` 状态; `ready` 且有 client 时渲染认证路径, 否则渲染 seed 降级路径 (`ProjectRunTree.tsx:99-100`)。
3. 降级路径按 `projects` → `lanes` (按 project_id 过滤) → `worktrees` (按 lane_id 过滤) 展开; 无 lane_id 的 Worktree 进入未归类分组 (`:180-256`)。
4. 认证路径按 RunDirectoryTree 的 project/run/worktree/branch 节点渲染 (`:477-535`); Lane 节点未作为独立类型渲染。

**Canvas 范围选择**:

- `CanvasScopeSelector` 以 `UNCLASSIFIED_LANE_ID` 表示未归类 Worktree 的伪 Lane (`frontend/src/components/canvas/CanvasScopeSelector.tsx:31`), 切换 Lane 时清空 Worktree (`:93`), 切换 Worktree 时回填 Branch (`:99`)。
- Canvas 页面依据 `selectedWorktreeId` 与 `selectedLaneId` 解析画布 (`frontend/src/app/canvas/page.tsx:35-36, 62-72`)。

### 4.5 权限与安全

| 项 | 设计要求 (来源) | 代码现状 |
|---|---|---|
| Lane 授权 | Lane 可授权管理, 授权前提不依赖远端 (WBS §2 Lane 本地实体先行) | 无 Lane 授权检查; 前端 store 无授权字段 (`ids.ts:437-447`) |
| 认证路径 | 目录数据来自会话 client (`ProjectRunTree.tsx:99`) | 认证失败时降级为 seed, 且标注降级 (`:154`) |
| Canvas 范围 | 点击入口时把 project/lane/worktree 范围交给后端做 RLS 复验 (`canvas/[id]/page.tsx:128-132`) | 本地预览不接入后端, 页面明确提示"本地预览未接入后端" (`canvas/[id]/page.tsx:149`) |
| 深链 | URL 参数不得作为授权依据 | URL 参数直接写入导航上下文 (`worktree/page.tsx:530-537`); 授权由后端承担 (未实现) |
| 环境 | 不依赖前端状态推断执行权限 (WBS Phase 6.4) | ❓ 后端执行权限复验未在代码中核对到; 当前前端仅做显示 |

### 4.6 依赖关系

| 依赖方 | 被依赖 | 位置 |
|---|---|---|
| `navStore` | `registry` (模块定义) | `frontend/src/lib/nav/navStore.ts:22` |
| `Sidebar` | `ProjectRunTree`, `registry` | `frontend/src/components/Sidebar.tsx:31, 33` |
| `ProjectRunTree` | `navStore`, `store`, `runDirectorySession`, `runDirectoryApi`, `GuardedLink`, i18n | `frontend/src/components/run/ProjectRunTree.tsx:76-86` |
| 根布局 | `Sidebar` | `frontend/src/app/layout.tsx:9, 78` |
| `(app)` 布局 | `AppShell` (含 `AppHeader`、`RightChatRail`) | `frontend/src/app/(app)/layout.tsx:49`; `frontend/src/components/AppShell.tsx:49, 77` |
| `/worktree` 页 | `worktree/layout.tsx` 的 `AppShell` | `frontend/src/app/worktree/layout.tsx:18` |
| `CanvasScopeSelector` | `navStore`, `store` | `frontend/src/components/canvas/CanvasScopeSelector.tsx:51-55` |
| Lane 设置页 | `store` (lanes, worktrees) | `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx:330-340` |

### 4.7 页面设计

本功能拥有两个页面路由, 以及一个作为壳层组件出现在所有 (app) 路由上的侧栏。侧栏的导航树与底部全局组在 §4.7.1 的布局中说明。

### 4.7.1 /worktree

**文件**: `frontend/src/app/worktree/page.tsx`

**布局**:

- 根布局在 `frontend/src/app/layout.tsx:78` 挂载 `<Sidebar />` (左侧栏, 含 ProjectRunTree 与底部全局组)。
- `frontend/src/app/worktree/layout.tsx:18` 使用 `<AppShell>`; `frontend/src/components/AppShell.tsx:49` 提供顶栏 `AppHeader`, `:77` 提供右侧 `RightChatRail`。
- 主区为页面内容。

**组件树** (顶层):

| 组件 | 位置 |
|---|---|
| `Sidebar` | `frontend/src/components/Sidebar.tsx:47` |
| `ProjectRunTree` | `frontend/src/components/run/ProjectRunTree.tsx:92` |
| `AppShell` | `frontend/src/components/AppShell.tsx` |
| `AppHeader` | `frontend/src/components/AppHeader.tsx` (由 AppShell 引用, `AppShell.tsx:49`) |
| `RightChatRail` | `AppShell.tsx:77` 引用 |
| 页面主体 `WorktreePage` 相关组件 | `frontend/src/app/worktree/page.tsx` (`:388-391` 读取导航上下文) |

**状态矩阵**:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 有选中 Project 与 Worktree | 面包屑与中央内容随上下文切换 | 🟡 上下文同步存在 (`:530-537`), 面包屑未逐项核对 |
| 加载 | 挂载前 (`hasMounted` 为假) | 选中 Project 按 `hasMounted` 取值 (`:397`) | ✅ 有条件分支 |
| 空 | 无 Project 选中 | 侧栏显示空 Project 占位 (`Sidebar.tsx:401-413`, `projectDisabledHint`) | ✅ 代码存在 |
| 错误 | 深链参数与 store 不一致 | 以 URL 为准回写 store (`:534-535`) | 🟡 无用户可见错误提示 |
| 未授权 | 认证失败 | 树降级为 seed (`ProjectRunTree.tsx:99-100`), 降级提示 `:154` | 🟡 降级但未区分 401 与 403 |
| 未接入 | 无宿主会话 | 同上降级路径 | ✅ 有降级提示 |
| 禁用 | 无 Lane 时 Lane 选择器禁用 | `CanvasScopeSelector.test.tsx` 用例 1 描述 | ✅ 仅在 Canvas 选择器中实现 |

**交互**:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 点击 Project 行 | 无 | 设置 selectedProjectId, sidebarScope=project, 跳转 `/worktree?project_id=` | `navStore` (localStorage) | ✅ `ProjectRunTree.tsx:135-138, 169-170` (fallback), `:302-305, 486-488` (认证) |
| 点击 Lane 行 | fallback 路径, lanes 存在 | 清空 Worktree/Branch, 仅保留 Lane | `setLaneContext` | ✅ `:141-143, 189` |
| 点击 Worktree 行 | 有 lane_id 或未归类 | 设置 Lane、Worktree、Branch, 跳转深链 | `setLaneContext` + URL | ✅ `:145-147, 206-210, 240-244` |
| 切换项目 | selectedProjectId 变化 | 清空下级 | `navStore.ts:240-243` | ✅ |
| 刷新目录 | 认证路径 | 调用 `model.refresh()` | RunDirectoryTree | ✅ `ProjectRunTree.tsx:379` |
| 新建 run | 认证路径, 项目行 | 打开创建对话框, 调用 `createEngineeringRun` | `RunDirectoryApiClient` | ✅ `:343-377, 489-491` |
| 折叠侧栏 | 任意 | `sidebarFold` 切换 | `navStore.ts:227-231` | ✅ |
| 点击认证路径的 Lane (run 行) | 认证路径 | 仅跳转 run 页面, 不设置 Lane | `:526-530` | 🟡 Lane 语义缺失 |

**数据**:

- 来源: `useStore` (`projects`, `worktrees`, `lanes`) 与 `useNavStore`; 认证路径来自 `useRunDirectory()` 会话。
- 字段: Project 的 `id`、`name`、`key`; Lane 的 `id`、`name`、`project_id`、`git_remote_url`; Worktree 的 `id`、`name`、`branch`、`lane_id`、`status`。
- seed 数据为本地 deterministic mock (`frontend/src/lib/seed.ts:199-216`)。

**与文档设计的差距**:

- FR-01-03: Lane 创建与授权动作缺失。
- FR-01-10: 看板面包屑的 Lane 与 Project 名称硬编码 (`kanban/page.tsx:208, 212`), 未与导航上下文统一。
- FR-01-18: 认证路径未把 Lane 作为层级渲染 (`ProjectRunTree.tsx:518-532`)。
- FR-01-16: 认证路径无数据来源标识。
- FR-01-17: 侧栏 scope 切换控件缺失 (`Sidebar.tsx:198-201`)。
- ui-3pane-arch §1.2 Home 核心区 (Worktree 控制中心、Agent、Feedback、Validation、Review): 在 `Sidebar.tsx:220` 的 `home-personal` 分组之外未核对到对应核心区, ❓ 需在 Sidebar 模块核对。

### 4.7.2 /lane/[laneId]/settings

**文件**: `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx`

**布局**:

- `(app)` 布局 `frontend/src/app/(app)/layout.tsx:49` 使用 `<AppShell>`, 侧栏由根布局 `frontend/src/app/layout.tsx:78` 提供。
- 页面内以顶部标题与标签页 (`:481`, 角色 tablist) 组织主区。

**组件树** (顶层):

| 组件 | 位置 |
|---|---|
| `LaneSettingsPage` | `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx:323` |
| 路径配置行 | `:121-123` (`onUpdate`) |
| Worktree 表格 | `WorktreeTable` 引用 `:626` |
| 统计区 | `useMemo` 计算 `LaneStats` `:354-380` |

**状态矩阵**:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | lane 存在 | 标题、标签页、Worktree 表格 | ✅ `:474-477, 481` |
| 加载 | 无 | 无加载态 | ⬜ |
| 空 | 该 Lane 下无 Worktree | "该 LANE 下暂无 Worktree" | ✅ `:234` |
| 错误 | lane 不存在 | "找不到 LANE" 与 ID | ✅ `:438-444` |
| 未授权 | 无 | 无授权检查 | ⬜ |
| 未接入 | 无后端 | 路径配置为空占位, Git 按钮调用 `executeGitOp` | 🟡 执行结果来源 ❓ |
| 禁用 | 无 Worktree 时批量按钮禁用 | `disabled={laneWorktrees.length === 0 || gitLoading}` | ✅ `:602-611` |

**交互**:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换标签 paths / stats / git | 无 | 切换 `activeTab` | 本地 state (`:343`) | ✅ |
| 编辑路径 | 无 | 更新 `pathConfigs` | 本地 state (`:344-348`), 不持久化 | 🟡 |
| 批量 fetch / pull / push / status | 至少 1 个 Worktree, 非加载中 | 对每个 Worktree 调用 `executeGitOp` | ❓ 需核对 `executeGitOp` 后端 | 🟡 |
| 单 Worktree Git 操作 | WorktreeTable 行 | 打开 gitModal | 本地 state (`:349`) | 🟡 |

**数据**:

- 来源: `useStore` 的 `lanes`、`worktrees`、`projects`、`agentSessions` (`page.tsx:330-340`)。
- 字段: Lane 的 `name`、`project_id`; Worktree 的 `lane_id`、`status`、`branch`; 统计 `LaneStats` (`:71`)。
- 路径配置 (`LanePathConfig`, `:63-69`) 无 seed 值, 默认三行空占位 (`:344-348`)。

**与文档设计的差距**:

- FR-01-12: 路径配置未持久化, Git 批量结果来源未在本页核对。
- FR-01-03: 页面未提供 Lane 创建或授权入口。
- FR-01-15: 无后端授权与 RLS 复验。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 导航上下文 store | `frontend/src/lib/nav/navStore.ts` | ✅ | `:55-59, 83, 237-246, 259-278` |
| Lane / Worktree 类型 | `frontend/src/types/ids.ts` | ✅ | `:437-447, 462-467` |
| 全局 store 的 lanes 集合 | `frontend/src/lib/store.ts` | ✅ | `:177, 434` |
| seed 数据 (Lane 与 Worktree) | `frontend/src/lib/seed.ts` | ✅ | `:199-216` |
| 侧栏目录树 (seed 降级路径) | `frontend/src/components/run/ProjectRunTree.tsx` (`FallbackProjectRunTree`) | 🟡 | `:110-271` |
| 侧栏目录树 (认证路径) | `frontend/src/components/run/ProjectRunTree.tsx` (`AuthenticatedProjectRunTree`) | 🟡 | `:273-389, 477-535` |
| 侧栏壳层 | `frontend/src/components/Sidebar.tsx` | ✅ | `:33, 198-219` |
| 根布局挂载侧栏 | `frontend/src/app/layout.tsx` | ✅ | `:9, 78` |
| `/worktree` 页面上下文同步 | `frontend/src/app/worktree/page.tsx` | 🟡 | `:388-396, 530-537` |
| Canvas 范围选择器 | `frontend/src/components/canvas/CanvasScopeSelector.tsx` | ✅ | `:31, 51-99` |
| Canvas 页面解析 | `frontend/src/app/canvas/page.tsx` | ✅ | `:35-72` |
| Canvas 入口范围 | `frontend/src/app/canvas/[id]/page.tsx` | 🟡 | `:110-112, 139-149` (本地预览) |
| 看板面包屑 | `frontend/src/app/(app)/kanban/page.tsx` | 🟡 | `:205-213` (硬编码) |
| Lane 设置页 | `frontend/src/app/(app)/lane/[laneId]/settings/page.tsx` | 🟡 | `:323-626` |
| Worktree 组视图的 Lane 回退 | `frontend/src/app/worktree/[id]/group/page.tsx` | ✅ | `:999-1002` (无 lane_id 时取项目首个 lane) |
| 后端 Lane 实体与路由 | `crates/` (domain-project, domain-workspace, domain-worktree, star-api-rest) | ⬜ | 检索无 Lane 命中 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| Lane 创建动作缺失 (FR-01-03) | 用户无法在界面新建 Lane; Lane 只能来自 seed | 在 store 增加 `addLane` 动作与创建 UI, 并让 `ProjectRunTree` 与 Lane 设置页共用 |
| 认证路径不渲染 Lane 层 (FR-01-01, FR-01-18) | 真实会话下侧栏缺少 Lane 层级, Lane 语义与 run 不一致 | 在 RunDirectoryTree 中引入 lane 节点, 或由 run 与 lane 的映射函数提供 lane_id |
| 点击认证路径的 run 行不设置 Lane 上下文 (`ProjectRunTree.tsx:526-530`) | 深链与面包屑上下文不一致 | 认证路径的 run 行调用 `setLaneContext` 并标注 lane 来源 |
| 后端 Lane 实体、RLS、授权缺失 (FR-01-15) | Lane 授权只能是前端显示, 不能作为执行依据 | 在 domain-project 或新 crate 中增加 Lane 聚合与 RLS 策略, 并在 star-api-rest 增加路由 |
| 看板面包屑硬编码 (FR-01-10, `kanban/page.tsx:208, 212`) | 看板上下文与导航上下文不一致 | 面包屑读取 `useNavStore` 的 selectedProjectId 与 selectedLaneId |
| URL 参数直接写入导航上下文 (`worktree/page.tsx:530-537`) | 深链可被构造, 不经授权复验 | 深链仅作为请求参数, 上下文在后端复验后回写 |
| Lane 设置页路径配置未持久化 (FR-01-12) | 路径配置刷新即丢失 | 为 LanePathConfig 增加存储与后端接口 |
| 侧栏 scope 切换控件缺失 (FR-01-17) | 用户无法在 main 与 project 之间切换 | 在 `Sidebar.tsx:198` 注释对应位置补充控件 |
| 降级提示未区分未授权与未接入 (`ProjectRunTree.tsx:154`) | 401 与 403 的用户指引相同 | 按状态码分别给出处理提示 |
| ui-3pane-arch Home 核心区未核对 | 设计中的核心导航项缺少对照 | ❓ 在 Sidebar 模块核对 `home-personal` 之外的核心分组 |

## 7. 验收与测试

已存在的测试 (只读核对, 未运行):

| 测试文件 | 覆盖内容 | 位置 |
|---|---|---|
| `frontend/src/lib/nav/__tests__/navStore.test.ts` | 默认值、侧栏与页签增删、fold / scope / selectedProjectId、持久化键 `star-nav-store:v2`、v3 迁移 | 用例名见 `:21-218` |
| `frontend/src/components/canvas/CanvasScopeSelector.test.tsx` | 三级范围选择器: 默认值、联动过滤、切换 Project 清空、未归类 Worktree 归入默认 LANE、脏持久化值回退 | 用例 1-8, `:50-165` |
| `frontend/src/components/run/ProjectRunTree.test.tsx` | 切换 Project 清空 Worktree focus; 展开 Project 与 LANE 不选中; ArrowRight 展开 | `:114-146` |
| `frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx` | Worktree 控制中心位于 Home 核心区 (`:100`); Run Worktree 行的生命周期与绑定信息 (`:132`) | `:99-132` |

**已验证 (由测试名称与断言描述支持)**:

- 导航上下文字段与切换规则 (FR-01-05 至 FR-01-08)。
- Canvas 范围选择器的级联规则 (FR-01-06, FR-01-07, 未归类分组)。
- 侧栏 Project 与 Worktree 行的状态描述。

**未验证**:

- 认证路径下 Lane 层级的渲染 (无对应测试)。
- Lane 创建、授权、列出的端到端行为 (无对应测试)。
- URL 深链与 store 同步的端到端 (`worktree/page.tsx:530-537` 无专门测试)。
- Lane 设置页的 Git 批量操作与路径配置持久化。
- 后端 Lane 的 RLS 与授权 (无后端实现)。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md` | §0 现状表、§1 Phase 1 导航上下文、§2 Lane 本地实体先行与内部 Run 命名保留、Phase 2 Project / Lane 布局 | §0 称"`Worktree` 无 `lane_id`; 无 `Lane` interface; store 无 `lanes` 集合"。代码在 637ed19d 已有 (`ids.ts:437-467`, `store.ts:177`) | 以代码为准。WBS 的 §0 差距行与代码不一致, 本文档按代码记录 |
| `docs/architecture/2026-11-15-lane-workspace/REVIEW-2026-11-14-IMPLEMENTATION.md` | 审查方法; Lane 相关的提交 (462260e8, cfbfe4c5) 标为已闭环 | 审查文档以 2026-11-14 提交为主, 仅在 §1.5 与 Lane Phase 1 相关。其 §3 指出的 dev 分支前移问题属于分支管理, 不是产品事实 | 仅采纳 Lane Phase 1 的闭环状态作为参考; 其余内容不纳入本功能 |
| `docs/specs/domain-project-spec.md` | Project 聚合、ProjectPolicy、RLS 与 Project 权限字符串 | spec 不含 Lane 实体, 与 WBS 的 Lane 本地实体不一致 | 后端 Project 聚合以 spec 为边界; Lane 的后端归属在 spec 中未定, 列为 §6 差距 |
| `docs/specs/domain-workspace-spec.md` | Workspace 与成员关系、RLS 与删除级联 | 不涉及 Lane | 仅作为 Project 上游边界引用 |
| `docs/frontend/design/ui-3pane-arch.md` | §0.2 现状纠正注记 (导航层级 Project→Lane→Worktree→Branch; 侧栏两段); §1.2 SideBar 分组; §1.2.1 路由映射; §3 认知负荷规则 | §1.2 的 Home 核心区 (Worktree 控制中心、Agent、Feedback、Validation、Review) 在代码的 `Sidebar.tsx:198-220` 中未核对到对应实现; §1.1 布局图侧栏宽 240px, 代码为 256px (`globals.css:56`) | 侧栏宽度以代码 256px 为准; Home 核心区列为 ❓ |
| `docs/frontend/design/ui-redesign-multica-style.md` | §0.1 5 tab 与 4 redirect 实装注记; §4 SubNav 设计 | §1-§5 描述 6 路由与 22 路由聚合, 与当前导航树 (Project→Lane→Worktree→Branch) 不同; §4 的 SubNav 180px 与代码 256px 不同 | 6 路由聚合与 SubNav 设计已被后续导航树取代, 不作为本功能需求; 仅采纳 §0.1 的 tab 命名事实 |
| `docs/frontend/design/ui-detailed-design.md` | §1 降低认知负荷原则; §2 颜色 Token; §3 AppShell 布局 (侧栏 220px) | §3 的侧栏 220px 与 `globals.css:56` 的 256px 不同; §5 页面规格以 `/analytics`、`/planning` 为主, 与导航树无直接对应 | 颜色与原则采纳; 侧栏宽度以代码为准; 页面规格不纳入本功能 |
| `frontend/src/lib/nav/navStore.ts` | 导航上下文字段、切换规则、持久化 | 与 WBS §0 描述一致 (字段已存在) | 采纳 |
| `frontend/src/components/run/ProjectRunTree.tsx` | 双路径渲染、Lane 点选、Worktree 点选、未归类分组 | 认证路径不渲染 Lane 层 | 采纳并以代码为准记录差距 |
| `frontend/src/components/Sidebar.tsx` | 侧栏分区与 ProjectRunTree 挂载 | 注释 `:198` 的 Scope Toggle 无对应控件 | 以代码为准; 控件缺失列为差距 |
| 代码检索 (`lane`, `Lane`, `lane_id`) | 后端无 Lane 实体与路由 | 无 | 以代码为准 |
| 测试文件 (4 个) | 用例名称与断言范围 | 未运行 | 作为已验证证据的来源, 不代表当前通过 |
