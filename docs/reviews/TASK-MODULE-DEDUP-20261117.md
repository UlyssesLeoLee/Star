# 待办 / Jira 性质模块 — 重复模块矩阵与去重报告

日期: 2026-11-17
分支: dev
范围: `frontend/` (不碰 `crates/`, Rust lane 另有人负责)
依据发令: "待办等 jira 功能性质的模块应该具有应有的功能，完全和其他重复的模块请去除，降低认知负荷。优先复用以往代码，因为那时为了做 jira 相关功能已经做了很多。"

## 1. 盘点: 任务/待办/工单/Jira 性质模块清单

| 模块 | 路由 | 入口 (registry id) | 渲染内容 (代码证据) | 数据源 |
|---|---|---|---|---|
| 任务 IPA 看板 | `/kanban` | `kanban` (顶栏 tab, DEFAULT_HEADER_TABS[0]) | 6 视图 tabs: Sprint / 任务看板 / IPA 流程图 / 按文件开卡 / 时间线 / 日历 (`kanban/page.tsx`) | store.workItems / board / sprints / nemaStore |
| Sprint 主面板 | `/sprint` | `issues` (id 保留兼容, label "Sprint") | 3 视图: sprint / list / tree + 详情侧栏 + 搜索 (`sprint/page.tsx`) | store.workItems / sprints / board |
| 项目工作区 | `/projects` | `projects` | 6 tabs: kanban / timeline / backlog / agents / worktrees / ipa (`ProjectsClient.tsx`) | store (per project_id 过滤) |
| 旧任务卡详情页 | `/work-item/[id]` | 无 (registry 未注册) | 任务卡只读详情 (`work-item/[id]/page.tsx`) | store.workItems |
| 任务卡子窗口 | (组件, 非路由) | — | `WorkItemDetailDrawer` (kanban/projects/ipa 接线) | store.workItems |
| Planning Hub | `/planning` | `planning` | **被 next.config redirect 307 拦截 → 永不可达**, page.tsx 本体有 Sprints 容量/Milestones/Calendar 4 tabs | store.sprints / milestones |
| Review | `/review` | `review` (isCore) | ReviewRecord mock 列表按 Status 分组 (`review/page.tsx`) | 硬编码 MOCK_REVIEWS |
| 分析 | `/analytics` | `analytics` | 19 BI tabs (燃尽/速度/Sprint 报告/甘特图等) | store 聚合 |
| Worktree Group Jira 视图 | `/worktree/[id]/group` | 无 (Branch 功能集) | V字模型 12 阶段 × 5 状态槽 (`group/page.tsx` VMODEL_PHASES) | worktree 范围 WorkItem |

## 2. 重复模块矩阵

| # | 模块 A | 模块 B | 是否真重复 | 证据 (渲染的数据源 + 操作) | 处置 |
|---|---|---|---|---|---|
| P1 | `/kanban` ?view=Sprint | `/sprint` ?view=sprint | **真重复** | 同一组件 `SprintBoardView` (kanban/page.tsx L53 与 sprint/page.tsx L40 同 import); 同一 store.sprints + store.workItems (均全局); 同一组操作 (Backlog 拖入 Sprint / 创建 / 启动 / 完成 / 删除 / 改名 sprint)。/sprint 严格更全: 外加 list/tree 视图、320px 详情侧栏、搜索。 | **删** /kanban 的 Sprint 视图渲染 → 入口卡深链 /sprint (沿用页面内"时间线→/analytics"入口卡既有模式)。/kanban 默认视图改"任务看板"。 |
| P2 | `/projects` ?tab=kanban | `/kanban` ?view=任务看板 | **真重复** | 同一组件 `KanbanBoard` (ProjectsClient.tsx L666 与 kanban/page.tsx L54 同 import); 同一组操作 (transitionWorkItem / 列增删改重排 / 列内新建 / 打开 Drawer); 数据仅差 project_id 过滤 (projects 侧过滤, kanban 侧全局)。registry.integrity.test.ts 已编码 2026-11-14 拍板"kanban 移出 /projects, 仅 backlog/timeline 保留子 tab"。 | **合并进 /kanban**: 删 /projects kanban tab; /kanban 任务看板接管 project 范围 (navStore.selectedProjectId 过滤) + 接上孤儿组件 KanbanFilters (kind/assignee/priority); `/board` 307 → `/kanban`; 老 `/projects?tab=kanban` 客户端 redirect → `/kanban`; /projects 默认 tab 改 backlog。 |
| P3 | `/planning` (被 redirect 遮蔽) | `/projects` ?tab=timeline | 不是重复, 是**功能被吞** | redirects.ts L61-65 `{ source: "/planning", destination: "/projects?tab=timeline" }` 在 next.config 层抢先拦截 → `planning/page.tsx` (237 行真实实现: Sprint 容量/committed/completed SP 汇总 + Milestone 进度 + Calendar 拖拽) 永不可达 — 与 /workflow 同病史 (redirects.ts L76-78 注释记载 /workflow 因同样原因删除 redirect)。 | **恢复 /planning 独立页**: 删 redirects 条目 (沿用 /workflow 先例), planning 模块恢复应有功能; 补预览数据诚实徽标。 |
| P4 | `/work-item/[id]` 子页面 | `WorkItemDetailDrawer` 子窗口 | 重复但需用户拍板 | 同一 workItem 详情数据; Drawer 是用户三次拍板的统一形态 (居中子窗口 + CLI + 聊天接线), 子页面是 sprint 侧栏"打开在旧 work-item 详情页 (向后兼容)"链接的沉没目标。 | **保留不动** (老书签/深链兼容)。删除需 307 但会丢 workItemId 上下文, 无对等目标, 列"需用户拍板"。 |
| P5 | `/sprint` ?view=list | `/projects` ?tab=backlog | 近似, **不删** | 都是 workItems 表, 但: list 有 kind/SP 列 + 320px 详情侧栏 (全局数据); backlog 有 project_id 过滤 + FilterBar (状态/优先级 chips + 搜索) + 诚实三态。列组与操作不同组。 | 保留两个, 报告标注"需用户拍板是否后续收敛"。 |
| P6 | `/review` | — | 不重复 | ReviewRecord mock 列表, 无同质模块。 | 保留 (mock 数据, 后端 ReviewRecord 域待接)。 |
| P7 | worktree group Jira 视图 | `/kanban` | 不重复 | V字模型 12 阶段 × 5 状态槽是 worktree 范围的 Branch 功能集视图 (group/page.tsx), 数据与操作都是 worktree 隔离语义, 与全局看板不同域。 | 保留。 |

## 3. 实施记录

### 3.1 去重 (真重复)

- **P1**: `kanban/page.tsx` Sprint 视图 → 入口卡 (testid `kanban-sprint-entry`) 深链 `/sprint?view=sprint`; 默认视图 "任务看板"。
- **P2**: `ProjectsClient.tsx` 删除 kanban tab (TAB_ITEMS + 渲染块 + board 专用 handler); `cookies.ts` 默认 tab kanban → backlog (kanban 保留为 legacy 合法值用于 redirect 检测); `subNavRegistry.ts` /projects 组删除 kanban 项; `redirects.ts` `/board` → `/kanban`; `/kanban` 任务看板加 project 范围 + KanbanFilters。
- **P3**: `redirects.ts` + shim 删除 `/planning` redirect (planning 独立页恢复可达); planning 页补预览数据徽标。

### 3.2 功能补齐 (复用既有代码)

- **F1 epic→story 层级**: `sprint/page.tsx` buildTree 从"kind 推断"改为 **parent_id 真实层级** (WorkItem.parent_id 自 2026-11-11 addSubTask 已写入; 无 parent_id 的旧数据按 kind 归组兜底), 导出 + 单测。
- **F2 Kanban 筛选接 UI**: 孤儿组件 `KanbanFilters.tsx` (kind/assignee/priority, 从未被任何页面使用) 接入 `/kanban` 任务看板; 新增纯函数 `applyBoardFilters` + 单测。
- **F3 Sprint 项目范围**: `SprintBoardView` 已声明的 `projectId` prop 从未接线 → 实装过滤 (sprints/workItems 按 project_id; createSprint 传入 project_id); `/sprint` 页传 navStore.selectedProjectId。
- **F4 Backlog 优先级排序**: `ProjectsFilterBar.tsx` 新增纯函数 `sortBacklogItems` (status 主序 + priority p0→p3 次序), ProjectsClient backlog tab 接线 + 单测。

### 3.3 未动 (需用户拍板)

- P4 `/work-item/[id]` 子页面 vs Drawer (见矩阵)。
- P5 `/sprint?view=list` vs `/projects?tab=backlog` 后续收敛方向。
- e2e playwright 三个 spec 断言 `/projects` Kanban tab 的 `social-domain-marker` (collaboration-5d-lead-coordination / five-domain-feedback-loop / uat-business-acceptance) 需随 P2 更新 (vitest 不跑 playwright, 未在本 lane 验证)。

## 4. 验证

见本文件提交同 commit 的验证结果 (tsc 0 error + vitest 全量 + registry.integrity.test.ts 绿)。
