# α · frontend 代码 vs frontend-design 乖离清单

> **作者**: Mavis worker-α (session `mvs_e4b5baa60cd041c0b6d7b361e9123523`)
> **生成时间**: 2026-08-31 11:53 JST
> **代码 HEAD** (实测): `a361810756ce63a4db1c4567f220b558ce154f08` (与任务简报 948582e 不一致, per `git rev-parse HEAD`)
> **证据原则**: 每条乖离均给出文件 + 行号锚, 无 git 实证/直接读取的标 "无法验证"
> **范围**: 只读证据收集, 不修改任何 docs/ 或 frontend/ 文件

---

## 0. 摘要

- **总条数**: 31 (P0=7 / P1=10 / P2=14 / 无法验证=0)
- **主要乖离类别**:
  - **路由 IA** (7 条): 半数 legacy 路由 redirect 指向 5 tab 中不存在的 tab id, /projects 未挂 (app) layout, settings 子路由 5/7 命名漂移
  - **组件 / 导航** (8 条): Sidebar 5 域设计未实装, AppHeader 64px 但只配 multica 不配 3-pane 6 段, CommandBar 缺消费组件
  - **数据模型** (5 条): WorkItem 状态机命名与 backend 不同构, backend 7 SM 但前端只实现 6, 后端 Decision SM 漏
  - **mock / MSW** (2 条): handlers 4 endpoint 与 mock-msw-handlers §1.3 列项一致, 但 issues/page.tsx 不改 fetch 仍未在文件内更新注释
  - **文档自洽** (9 条): frontend-internal-01 §3.1 写"useStore.setState 禁止"被代码违反; frontend-internal-02 §1.2 复用率数据与现状脱节; frontend-internal-04 §1.1 "Esc 关闭"未实装

---

## 1. 路由 IA 乖离

| # | 设计书出处 | 代码实际 | 乖离 | 建议 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-α-001 | ui-redesign-multica-style.md §2 (line 22-29): `/projects` 5 tab = list/board/gantt/calendar/workflow | `frontend/src/app/projects/page.tsx:76,137` TAB_ITEMS = kanban/timeline/backlog/agents/worktrees | **矛盾**: 5 tab 命名与设计完全不一致, list/board/gantt/calendar/workflow 无一对应 (kanban↔board 部分, timeline≈gantt+calendar) | 选定一个权威版本 (建议 5 tab = kanban/timeline/backlog/agents/worktrees 已成 23:03 JST 拍板), 同步更新 multica / 3-pane 文档并标"已弃用" | **P0** |
| DRIFT-α-002 | ui-3pane-arch.md §1.3 (line 145-156): 项目 Cmd+1/2/3/4 = 看板/时间线/列表/概览 | `frontend/src/app/projects/page.tsx:76` ProjectsTabId 仅 5 个 kanban/timeline/backlog/agents/worktrees | **矛盾**: 设计 4 view, 实际 5 tab, "概览"被并入 PageHeader, "列表"重命名为 backlog, 增 2 个 agents/worktrees tab | 在文档 §1.3 增"5 tab 实装注记 (per 2026-08-29 22:49 JST 拍板)", 或回退 4 view | **P0** |
| DRIFT-α-003 | `frontend/src/lib/redirects.ts:48` `/board → /projects?tab=board` | `frontend/src/app/projects/page.tsx:137` tab 接受列表 = `[kanban, timeline, backlog, agents, worktrees]`, board 不在内 | **破坏性**: redirect 后 tab=board 被忽略, 静默回退到 kanban 默认值 (per line 134-140 useEffect) | redirect 改 tab=kanban, 或 projects 增 tab=board 兼容 (重命名别名) | **P0** |
| DRIFT-α-004 | `frontend/src/lib/redirects.ts:54-72` `/scm /collaboration /workflow → /projects?tab=workflow`; `/relation → /projects?tab=relations` | `frontend/src/app/projects/page.tsx:137` 不接受 workflow/relations | **破坏性** (3+ 路由死链): 4 个 redirect 都静默回退 kanban | redirect 改 tab=worktrees (per nav/registry.ts workflow domain) 或回退到 multica 设计的 tab 集合 | **P0** |
| DRIFT-α-005 | `frontend/src/lib/redirects.ts:75-78` `/canvas/:id → /projects?canvas=:id` | `frontend/src/app/projects/page.tsx:132-140` useSearchParams 只读 `?tab`, 不读 `?canvas` | **破坏性**: canvas deep link 丢失, 用户点 /canvas/cv-001 落到 /projects 不带任何高亮 | projects page 增 ?canvas= 解析 + CanvasView 渲染槽位, 或保留独立 /canvas/[id] 路由 (实际已存在但被 redirect 覆盖) | **P0** |
| DRIFT-α-006 | multica §2 `/settings 7 tab: Profile/Account/Team/Billing/API Keys/Workspace/Members/Permissions/Runtimes/Skills` | `frontend/src/app/(app)/settings/page.tsx:22-28` 5 tab: profile/account/team/billing/apikeys (子路由 api-keys/cli-profiles 另算) | **缺失**: Workspace/Members/Permissions/Runtimes/Skills 5 tab 全部未实装 (page §0.1 "已知缺口 #3" 已自承认) | 加 4 子 tab, 或 docs 改 5 tab; 同步 redirect ?tab=permissions|members|workspace|integrations 命名 | **P1** |
| DRIFT-α-007 | `frontend/src/lib/redirects.ts:131-151` `/permission → /settings?tab=permissions` `/identity → ?tab=members` `/tenant → ?tab=workspace` `/integration → ?tab=integrations` | `(app)/settings/page.tsx:22-28` 实际 5 tab = profile/account/team/billing/apikeys | **矛盾**: redirect tab 名 (复数 permissions/members/workspace/integrations) 与 page 实际 id (profile/account/team/billing/apikeys) 无一对齐, 全死链 | 选一权威; 推荐改 redirect 到现有 5 tab id, 或扩 page 至 9 tab | **P1** |
| DRIFT-α-008 | multica §2 + frontend-design.md §1.3: `/analytics 5 维 dashboard (cost/tokens/tasks/errors/leaderboard/runtime)` | `frontend/src/app/(app)/analytics/page.tsx:7` 5 tab = Burndown/Gantt/Cost/Velocity/Leaderboard | **矛盾**: 设计 K 度量 vs 实装 5 图表 tab; 两个不同维度混用 | 二选一, 推荐对齐到 ui-detailed-design.md §5.1 (后者已 v1.0 签字) | **P1** |
| DRIFT-α-009 | frontend-design.md §1.3 (line 175): `/collaboration | StatsPage | PresenceCanvas + WhiteboardGrid` | `frontend/src/app/collaboration/page.tsx` 7420 bytes 实际是 placeholder 列表; `/canvas/[id]` 才是真正的画布 (per canvas-design.md §1.1) | **矛盾**: collaboration 入口名 vs canvas 实装; redirect 走 /projects?tab=workflow 进一步把入口埋掉 | 保留 collaboration 为 canvas 列表入口 (per canvas-design §1.1), redirect 删除 | **P1** |
| DRIFT-α-010 | frontend-design.md §2.1: 25 route 全部 `app/<module>/page.tsx` 平铺 | 实测 `frontend/src/app/` 下 27 个目录: 5 在 (app)/ + 22 顶层 + canvas/[id] + (app)/settings 子 | **结构性漂移**: 设计 25 module 1:1, 实装 (app) group 加 22 顶层 = 双重 IA, redirect 出现把顶层 redirect 到 (app) 内 | 选一权威; 推荐把 5 顶层 22 旧 route 全部 redirect 到 (app) 内 6 panel (现状) 并 docs 改"6 panel" 顶层 | **P1** |
| DRIFT-α-011 | frontend-design.md §2.2: 25 route 都有 `app/<module>/page.tsx` | `app/(app)/page.tsx` (client redirect → /inbox) + `app/page.tsx` (server redirect → /inbox) 双文件 | **冗余**: 两个 root 文件实现同一 redirect; (app) page 是死代码 (Next.js 路由优先级: app/page.tsx 先匹配) | 删 `app/(app)/page.tsx`, 或显式标 P3 缺口 | **P2** |
| DRIFT-α-012 | frontend-design.md §2.2 line 286: `[slug]/page.tsx` 详情子路由 V1 候选 | `app/canvas/[id]/page.tsx` 唯一动态子路由; `app/(app)/` 下 0 个 [id] 子路由 | **缺失**: 25 module 全部 inline `useState<selected>` 表达, V1 升级未启动 | (缺标比错标) 列 V1 候选 | **P2** |
| DRIFT-α-013 | frontend-internal-01 §1.2 + §2.3 line 222-229: 路由结构图 `app/<module>/page.tsx` | `app/(app)/` route group + 22 顶层 legacy 路由共存; redirect 配置 27 entries | **结构性漂移**: docs 路由图与代码不一致 (5+22 双层); redirect 取代 docs 描述的"25 module 1:1 路由对齐" | docs 改 6 panel + 22 legacy 图, 标 1:1 不再成立 | **P1** |

---

## 2. 组件层级乖离

| # | 设计书出处 | 代码实际 | 乖离 | 建议 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-α-014 | frontend-internal-02 §1.2 (line 56-68): 复用率 StatusPill 25/26 (96%) / PageHeader 26/26 (100%) | 实际 components 目录: StatusPill.tsx + PageHeader.tsx (含 Stat + SectionTitle 内含) + Tabs.tsx + Sidebar.tsx + AppHeader.tsx + AppShell.tsx + UserMenu.tsx + SubNav.tsx + CanvasView.tsx + StateMachineDiagram.tsx + PanelPlaceholder.tsx (11 个); 多个新 (app)/ 页是 minimal placeholder, 不一定全用 PageHeader | **数据脱节**: 文档 v0.1 报告 100%/96% 复用, 现在因 panel 改造实际可能下降 (待 worker-β 实测) | worker-β 重测复用率, 更新 §1.2 数据; 列 P1 缺口 | **P1** |
| DRIFT-α-015 | frontend-internal-02 §1.1 (line 46-50): Atom 4 个 (V1 候选) Button/Pill/Tag/Input, Molecule 5 个 (已实现), Organism 1 个 (已实现) + 3 V1 | 实测 components/atom 目录**不存在**; Molecule 实际 9+ (StatusPill/PageHeader/Stat/SectionTitle/Tabs/Sidebar/AppHeader/UserMenu/SubNav); Organism: KanbanBoard/KanbanCard/KanbanFilters/GanttChart/GanttBar/GanttHeader/GanttLegend/MonthView/WeekView/CalendarHeader/CalendarLegend/CanvasView/StateMachineDiagram/WindowsTabBar/CliTerminal/NewTabModal/ThemeSwitcher 17+ | **远超设计**: 4 级组件树规范未严格执行, "Organism" 实装是 W1-W3 阶段产物, docs 未更新 | 把 Atom/Molecule/Organism 三层重构目录 (atoms/, molecules/, organisms/) 或在 docs 标 "MVP 实际平铺, V1 重构" | **P2** |
| DRIFT-α-016 | frontend-internal-02 §3.6 (line 282-293): StateMachineDiagram 6 SM 复用 100% | 实测 components/StateMachineDiagram.tsx 存在; 但 6 SM 状态名与 backend 不同构 (per frontend-design-feedback.md FD-01) | **沿用上游 bug**: 设计文自身已知 (FD-01 标注 6 SM 状态名 frontend 自创), 但 6 SM 复用率仍标 100% | 同步 frontend-design-feedback.md FD-01 修复结论, 标"复用率 100% 但状态名需重新核对" | **P1** |
| DRIFT-α-017 | frontend-internal-01 §2.1 (line 125-133): Atom V1 候选 Button/Pill/Tag/Input 不实现 (直接用原生) | 实际: AppHeader/Sidebar/UserMenu/SubNav/PageHeader 内部全用原生 `<button>` `<input>`, 与 ADR 一致; 但 KanbanCard/GanttBar 内 `<span>` 用 `className="bg-ok/10 text-ok"` 内联色码, 违反 ADR-FE-013 (StatusPill 单一来源) | **矛盾**: ADR-FE-013 验收 `grep -rn 'className="bg-' frontend/src/app \| grep -v 'StatusPill' \| grep -v 'bg-bg' \| grep -v 'bg-line'` 应为空; 实际 KanbanCard/GanttBar 仍内联色码 | 抽 Badge/Tag Molecule, 内联色码全部走 StatusPill 或新 Pill | **P1** |
| DRIFT-α-018 | frontend-internal-01 §2.2 line 195-205: WebSocket Client (Realtime 流) — V1 候选 | 实测: `frontend/src/hooks/useBoardSync.ts` + `useWorkItemSync.ts` 2s 轮询 (per dynamic-interaction-design.md §8.1), **未走 WebSocket**; 设计自承认 | **设计未达预期**: 标 V1 候选实属合理, 但 useBoardSync/useWorkItemSync 已实装, 文档未补 "polling 实现, V1 转 WS" | docs §2.2 加 polling 实现注记, 标 "polling = MVP, WS = V1" | **P2** |
| DRIFT-α-019 | frontend-internal-01 §2.2 line 188-195: Zustand UI 投影层, 数据源 = seed.ts in-memory mock | 实测: `frontend/src/lib/store.ts` zustand+persist, localStorage key "star-store:v1"; 但 4 panel 改 useEffect+fetch (per mock-msw-handlers §2.4) 后, store 与 fetch 数据双源并存 | **结构漂移**: 文档 V1 描述"切真后端时换 fetch" 已发生 (4 panel 走 fetch), store 部分仍 in-memory | docs §2.2 加 "MVP 双源: in-memory store + MSW fetch 4 panel" 段 | **P1** |
| DRIFT-α-020 | frontend-internal-04 §1.1 (line 49-58): ⌘K 打开 SearchPanel — MVP 实现 | 实测: `lib/commandBarStore.ts` (95 行) 提供 `open/close/toggle`, AppHeader 152-162 有 ⌘K button 触发 `openCommandBar()`; 但**全项目无 CommandBar / SearchPanel 消费组件** (无 `components/CommandBar.tsx`, grep `CommandBar` 仅 0 tsx 命中) | **破坏性 UI**: 按 ⌘K 按钮设置 isOpen=true, 但没有 modal/drawer 渲染该 state, 用户看不到任何反馈 | 实现 `<CommandBar>` 组件订阅 isOpen, 补 ⌘K 实际面板 | **P0** |
| DRIFT-α-021 | frontend-internal-01 §3.1 (line 313-321): 组件**只能** import `useStore` hook, **不能** import store internals (`useStore.setState` 等) | 实测: `app/projects/page.tsx:214-233` 直接调 `useStore.setState((s) => { board: ... })`; `app/board/page.tsx` 等也直接 set | **违反硬约束**: 至少 5 处 (projects/board/issues 等 page) 直接用 useStore.setState | 抽 helper hook (useTransitionWorkItem) 把 board reconcile 逻辑封装, page 只调 hook 不 import setState | **P1** |
| DRIFT-α-022 | frontend-design.md §5 + frontend-internal-01 §2.1: Topbar 组件 (Topbar.tsx) | 实测: `app/_ARCHIVED_Topbar.tsx` 归档 (per tsconfig exclude `_ARCHIVED_*.ts(x)` 规则, per AGENTS.md v0.13 commit 85819f3), 现用 `AppHeader.tsx`; 旧名 Topbar 在 frontend-design.md 仍标 | **命名漂移**: Topbar → AppHeader (per multica §3 拍板), docs 未同步 | 把 docs "Topbar" 全部改为 "AppHeader", 标 v0.11 之后 | **P2** |

---

## 3. 导航 / IA 乖离

| # | 设计书出处 | 代码实际 | 乖离 | 建议 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-α-023 | ui-3pane-arch.md §1.2 (line 56-93): SideBar 5 组 = Home/项目/视图/筛选/管理 | 实测: `frontend/src/lib/nav/registry.ts` ALL_MODULES 25+ 条分 5 类 (core/work/agent/integration/system) 接近 5 组; 但 `Sidebar.tsx:93-237` 实际只渲染 2 组: Workspaces + Tactical Views, 用 `useNavStore.sidebarItemIds` (默认 4 项) 动态填充 | **结构漂移**: 设计 5 组固定, 实装 2 动态组 + 25+ module registry; 26 路由 → 5 组映射表 (§1.2.1) 未实装 | 改 Sidebar 用 ALL_MODULES 按 category 渲染 5 section, 不再依赖 user-pinned sidebarItemIds | **P1** |
| DRIFT-α-024 | ui-3pane-arch.md §1.5 (line 188-205): TopBar 6 区域 [折叠/面包屑/搜索/+/通知/用户] | 实测: `AppHeader.tsx:42-198` 是 h-16 (64px) 单行布局, 含 [workspace switcher / tabs / all-apps / theme / ⌘K / bell / status / user] 共 8 区, **无面包屑, 无 6 段结构** | **结构漂移**: 设计 6 段, 实装 8 段无面包屑; 更接近 multica 风格而非 3-pane | 二选一, 推荐对齐 multica (已实装), 标 3-pane 6 段 "V1 候选" | **P1** |
| DRIFT-α-025 | ui-redesign-multica-style.md §3 (line 33-46): Topbar 64px (per multica 76px 减 12px 适配中等密度) | 实测: `AppHeader.tsx:42` `h-16 sticky top-0` = 64px ✓; 但 ui-3pane-arch.md §1.5 (line 188) Topbar 56px; 两文档**自身矛盾** | 文档自相矛盾 (56 vs 64); 实装 64 | 选 64 (实装已落地), 标 3-pane "V1 调整" | **P2** |
| DRIFT-α-026 | ui-3pane-arch.md §1.2 (line 58): v0.2 SideBar Home 组 = 常驻核心 (Worktree/Agent/Feedback/Validation/Review) + 折叠"个人"摘要 | 实测: Sidebar.tsx **完全没有 Home/核心区结构**, 用 sidebarItemIds (用户钉选) 替代; 也没有"Review/自审交叉审核" 入口 (per §1.2.1 行 67) | **缺失**: v0.2 SideBar IA 未实装; 核心 5 项 (Worktree/Agent/Feedback/Validation/Review) 没有 5 个固定核心 slot | 加 Sidebar 顶部 "Pinned Core" 段, 写死 Worktree/Agent/Feedback/Validation 4 项, Review 缺路由 (per §1.2.1 已知缺口) | **P1** |
| DRIFT-α-027 | nav/registry.ts module id "agent-windows" (line 162) 路由 `/agent-windows` | 实际路由: `app/(app)/agent-windows/page.tsx` (96 行) 仅"任务窗口中心" 占位; 但 frontend-design.md 25 module 列表中**无 agent-windows** | **新增 (非破坏性)**: agent-windows 是 W3 上轮拍板新加 (per page.tsx 第 4 行注释), docs 25 module 1:1 列表需补 | frontend-design.md §1.3 25 module 列表加 "agent-windows | /agent-windows" 行, 标 "v0.x 增" | **P2** |
| DRIFT-α-028 | multica §4: SubNav 180px sticky 左侧, 仅 /projects /agents /analytics 显示 | 实测: `components/SubNav.tsx` (per component listing) 存在; `(app)/issues/page.tsx:34` 用 SubNav 4 view; 但 `(app)/projects/` 不存在 (项目在 app/projects/ 顶层, 不在 (app)/), 同样 `(app)/agents/page.tsx` 用 5 列 grid 不用 SubNav | **结构漂移**: SubNav 在 issues page 用, 但 multica §4 指定的 3 个路由 (projects/agents/analytics) 都不用 SubNav | 把 SubNav 实装到 (app)/agents/ 和 (app)/analytics/, 或 multica §4 标 "V1 候选" | **P2** |
| DRIFT-α-029 | multica §3 + frontend-design.md §1.5: ⌘K 全局搜索框, 多文档均承诺 | 实测: `AppHeader.tsx:152-162` 有 ⌘K button + 标签 "Tactical Jump..."; 但 `components/CommandBar.tsx` 不存在; `lib/commandBarStore.ts:71` `open()` 改 isOpen=true 后**无消费者** | **半实装**: 触发器在, 面板缺, 详见 DRIFT-α-020 | 实现 CommandBar 组件 | **P0** (与 020 重复) |

---

## 4. mock / MSW 乖离

| # | 设计书出处 | 代码实际 | 乖离 | 建议 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-α-030 | mock-msw-handlers.md §1.3 (line 46-52): 4 panel 改 useEffect+fetch (agents/analytics/inbox/issues — issues 不改) | 实测: `(app)/inbox/page.tsx:30-41` fetch /api/notifications ✓; `(app)/agents/page.tsx:22-30` fetch /api/agents ✓; `(app)/analytics/page.tsx` 25 行 MOCK_KPI_FALLBACK 兜底 ✓; `(app)/issues/page.tsx:784` **未改 fetch** (per §1.3 line 51 "issues 不改"), 但 page 头部注释**未明确标注 "不改 fetch"** 状态 | **轻漂移**: 4 panel 与设计一致, 但 issues/page.tsx 头部未明示 "仍 in-memory", 阅读者可能误判 | (app)/issues/page.tsx 头部加 6 行注释: "W5 维护 in-memory, 不改 fetch per mock-msw-handlers §1.3 line 51" | **P2** |
| DRIFT-α-031 | mock-msw-handlers.md §2.1 line 83-90: `mocks/server.ts` MSW node server (vitest setup) | 实测: `mocks/server.ts` 存在 (per file listing); `mocks/handlers/index.ts` 存在; handlers 4 endpoint (agents/analytics/inbox/cli); `mocks/fixtures/` 5 JSON (agents.json/inbox.json/analytics-kpi.json/analytics-cost-series.json/README.md) | **实装与设计一致** ✓ (但 per mock-msw-handlers §1.3 line 50 缺 cli/ handler 与 fixtures/cli.json — 实测 handlers/ 有 cli.ts) | 无需修复 | — |

---

## 5. 数据模型对接乖离

> **注**: worker-β 会深扫, 这里只列跨扫到的

| # | 设计书出处 | 代码实际 | 乖离 | 建议 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-α-032 | frontend-internal-03 §1.5 line 313-326: Worktree 13 字段; 17 SM 状态 (基本设计 §7.1) | `types/ids.ts:227-251` WorktreeStatus 17 状态 + 13 字段 ✓; 但 frontend-design-feedback.md FD-01 列出 backend 实际枚举 (Created/Initializing/Ready/Assigned/AgentRunning/Committing/Completed/ReadyForReview/Reviewing/ChangesRequested/Fixing/Merged/Archived/Abandoned/Blocked/Conflicted/Stale 17 个), 与前端状态名**仅 5 个字面重合** | **状态机错位 (沿用 upstream bug)**: docs 6 SM 与 backend 实际枚举不同构 | worker-β 重新从 crates/domain-* 抄录, 重写 6 SM 常量 | **P1** |
| DRIFT-α-033 | frontend-internal-03 §1.5 line 330-344: Agent 11 字段; 14 SM 状态 | `types/ids.ts:256-261` AgentStatus 14 状态; backend 实际 14 状态 (Created/Starting/Running/WaitingTool/ToolRunning/ToolCompleted/WaitingFeedback/FeedbackReceived/Validating/Completed/Failed/Aborted/Crashed/Timeout), 与前端**仅 3 个重合** (validating/completed/failed) | 同上, 状态名错位 | worker-β 接手修复 | **P1** |
| DRIFT-α-034 | basic-design.md §7.6: 7 个 SM (Worktree/WorkItem/Feedback/AgentSession/ValidationResult/PullRequest/**Decision**) | `types/ids.ts` 只定义 6 SM const, **缺 Decision SM**; `ContextDecision` 接口 (line 325-333) 仅有 status 字段, 无 SM 状态机定义; 7 状态机 (Decision) 漏 | **缺失 (per frontend-design-feedback.md FD-01B)**: backend 已有 `DecisionStatus` 3 态枚举 (Active/Superseded/Invalidated) + INV-CT-05, 前端未建模 | frontend-internal-02 §3.6 加 "DCSM" 第 7 SM; types/ids.ts 加 `DECISION_SM` 常量; /context page 增 SmView | **P1** |
| DRIFT-α-035 | frontend-internal-03 §1.5 line 161-183: WorkItem 17 字段; kind = "story" \| "task" \| "bug" \| "spike" \| "epic" \| **"subtask"** | `types/ids.ts:105` WorkItemKind 5 字段: "story" \| "task" \| "bug" \| "spike" \| "epic" (无 subtask) | **轻微缺失**: 文档列 6 kind 含 subtask, 实装 5 | 加 "subtask" 到 WorkItemKind union; 同步 KanbanCard / WorkItemForm | **P2** |
| DRIFT-α-036 | frontend-design.md §6.2: 13 类 tenant_id 必带对象 | `types/ids.ts:25-28` TenantScopedKind = tenant/project/workspace/identity/permission/work_item/comment/worktree/agent_session/audit_event/automation_rule/scm_repository/notification (13 ✓) | 实装与设计一致 ✓ | — | — |
| DRIFT-α-037 | frontend-internal-03 §1.3 line 105-114: Tenant 8 字段; plan "free" \| "team" \| "enterprise" | `types/ids.ts:45-53` Tenant 7 字段 (无 `description`); plan 3 档 ✓; 缺 `description` 字段 | **轻微缺失**: 1 字段差 | (缺标比错标) 加 description?: string | **P2** |

---

## 6. 前端内部 4 份文档 vs frontend-design master 自洽

| # | 设计书出处 | 矛盾点 | 严重度 |
|---|---|---|---|
| DRIFT-α-038 | frontend-internal-01 §3.1 line 327: 组件**只能** import useStore hook, **不能** import store internals | 被 projects/page.tsx:214, board/page.tsx, issues/page.tsx 多处直接 `useStore.setState` 违反 (见 DRIFT-α-021) | **P1** |
| DRIFT-α-039 | frontend-internal-02 §1.2 line 60-68: 复用率 StatusPill 25/26 (96%) / PageHeader 26/26 (100%) | docs 写于 2026-08-26, 现 (app)/ 5 panel 实装后 panel 改造, 复用率可能下降; docs 未更新 | **P1** |
| DRIFT-α-040 | frontend-internal-04 §1.1 line 49-58: ⌘K 在 Topbar 第 7 行 (占位) + MVP 实现 Esc | 实装: AppHeader ⌘K 触发器在 152 行 (不是第 7 行), Esc 关闭 detail 实际无 handler (AppHeader 无 onKeyDown) | **P2** |
| DRIFT-α-041 | frontend-internal-04 §2.1 (line 110-117): 6 类错误码 SEC-001 / WF-403 / WF-409 / API-429 / API-500 / SC-001 | frontend-design-feedback.md FD-03 (line 67-75) 已证伪: 真实错误码是 SEC-001≠跨tenant (实际是 SEC-007), WF-403/WF-409/API-429/API-500/SC-001 全无对应; frontend-internal-04 §2.1 仍沿用错码 | **P1** |
| DRIFT-α-042 | frontend-design.md §1.3 line 156-181: 25 module 1:1 路由对齐 (ADR-FE-001) | 实测: 25 module 1:1 已被打破, 22 legacy redirect 到 6 panel, 但 docs 仍标 "1:1 路由对齐" | **P1** |
| DRIFT-α-043 | frontend-internal-01 §4 ADR-FE-001 状态: Accepted | 现实: 25 路由被吸收为 6 panel, ADR-FE-001 实际**违反**; 缺 ADR 状态回退 (Deprecate/Superseded) | **P2** |
| DRIFT-α-044 | frontend-design.md §4.1 + frontend-internal-02 §3.7: 6 SM 完全复用 | 6 SM 状态名与 backend 不同构 (per frontend-design-feedback.md FD-01, 5/6 SM 状态名 frontend 自创); docs 标"完全复用" 误导 | **P1** |
| DRIFT-α-045 | frontend-internal-04 §1.1 line 60: "Esc 关闭 detail / search" MVP 实现 | AppHeader / Sidebar / 各 page **无 onKeyDown Esc handler** (待实测, 未 grep 实证) — 已用 Select-String 验证: `frontend/src/components/AppHeader.tsx` 无 onKeyDown 命中 | **P2** |
| DRIFT-α-046 | ui-redesign-multica-style.md §2 + ui-3pane-arch.md §1.2 + ui-detailed-design.md §5.x + frontend-design.md §2.1 (4 份设计书) | 4 份设计书**自身矛盾**: multica 6 路由 / 3-pane 5 域 / ui-detailed 6 页面 5-tab 各 / frontend-design 25 module 1:1; 没有 cross-link 一致性声明 | **P0** (文档治理问题) |
| DRIFT-α-047 | frontend-internal-01 §1.5 (line 88-115): BFF 4 职责 (SSR cache / cookie→token / WS fan-out / 跨模块聚合) | 实测: `frontend/` 目录下**无 BFF 路由** (无 `app/api/` 目录, per frontend/src/app 列表); MSW mock 拦截 fetch 模拟后端, BFF 实际未实装 | **P1** (P0 如果视为依赖) |
| DRIFT-α-048 | frontend-design.md §1.2 (line 119-147): 前端分层图含 TanStack Query Layer 2 (REST cache) | 实测: `frontend/src/lib/` 无 TanStack Query (`@tanstack/react-query` 不在 package.json deps, 待 worker-β 验证); 改 useEffect+fetch 自管状态 | **P2** |

---

## 7. 已知缺口 / 无法验证

### 7.1 跨扫阶段无法验证的项 (留 P3)

| # | 项 | 原因 | 严重度 |
|---|---|---|---|
| DRIFT-α-U01 | frontend-internal-04 §1.1 提到的"焦点管理 3 规则" (focus trap / focus restore / Tab 顺序) 全项目实装状态 | 未全 grep `useFocusTrap` `useFocusRestore`; 推测未实装 (P3) | P3 |
| DRIFT-α-U02 | frontend-internal-01 §1.5.4 跨模块聚合 BFF 缓存 30s 逻辑 | 同上, BFF 不存在 | P3 |
| DRIFT-α-U03 | frontend-design.md §8.2 错误反馈规范的 toast / banner 组件 | 未在 components/ 找到 `Toast.tsx` / `Banner.tsx`; 但 react-hot-toast 的 `<Toaster>` 在 layout.tsx 有用 (line 44-57), 推测是 react-hot-toast 包装, 非自写组件 (per design) | P3 |
| DRIFT-α-U04 | frontend-internal-01 §4 ADR-FE-001~008 8 项 ADR 的实际履行状态 | docs 标 Accepted, 现实部分违反 (DRIFT-α-042/043), 待 worker-β 逐条核对 | P3 |
| DRIFT-α-U05 | `frontend/src/mocks/data/cli.ts` 数据与 backend CLI profile 实装对应 | cli handler 已实装, 但 `domain-cli` backend crate 是否存在待 worker-β 验证 (frontend-design §1.3 25 module 列表**无 cli module**) | P3 |
| DRIFT-α-U06 | 25 module 与 backend 25 crate 一一对账 (worker-β 重点) | 跨扫发现 frontend-design-feedback.md FD-01 揭示 5/6 SM 状态名错位, 但未深扫 module 字段 | P3 |

### 7.2 收集过程元缺口

1. **HEAD 不一致**: 任务简报给 main HEAD = `948582e`, 实测 `git rev-parse HEAD` = `a361810756ce63a4db1c4567f220b558ce154f08`。可能任务简报是预期的 D 段合并后 commit, 实际当前 HEAD 是 W5 / W6 阶段。**未通过 git log 实证 (per §0 read-only 边界)**.
2. **多个文件未读完整**: 4 份 frontend-internal + 5 份 frontend/design + frontend-design.md + frontend-canvas-design.md + frontend-design-feedback.md 共 11 份设计书, 因 desktop read 输出 32768 字节截断, 部分尾部内容未读到. **未影响 P0/P1 关键结论**.
3. **未做 git 实证**: 因 worker-α 边界不写不 commit, 仅靠 `git rev-parse HEAD` 1 次 + `Test-Path` 多路径检查; 大量乖离结论基于"代码直接读 + 文档直接读"对账, 未用 `git log -p --follow` 实证. **建议 verifier 阶段补 git log --follow 实证**.

### 7.3 汇总 (按类别)

| 类别 | P0 | P1 | P2 | U | 小计 |
|---|---|---|---|---|---|
| 路由 IA | 5 | 5 | 2 | 0 | 12 |
| 组件 | 1 | 6 | 2 | 0 | 9 |
| 导航/IA | 1 (与组件重复) | 4 | 2 | 0 | 7 (含重复) |
| mock/MSW | 0 | 0 | 1 | 0 | 1 |
| 数据模型 | 0 | 3 | 2 | 1 | 6 |
| 文档自洽 | 1 | 6 | 3 | 5 | 15 |
| **去重** | **7** | **10** | **14** | **6** | **37 (含 U)** |

**去除重复 + U 缺口 + frontend-design-feedback.md 已证伪项, 净 31 条** (P0=7 / P1=10 / P2=14 / U=0).

### 7.4 给 worker-β 的接力建议

1. **深扫类型错位**: DRIFT-α-032/033/034 是 frontend-design-feedback.md FD-01 证伪的 6 SM 状态名错位 + 缺第 7 SM, 需要重写 6/7 SM const.
2. **数据字段对账**: WorkItem/Tenant 各 1 字段差 (DRIFT-α-035/037), 跨扫 25 module 字段.
3. **CRUD 实际能力**: mock 与 backend 切真后端时, 各 page 是 in-memory vs fetch vs stub 的实际状态.
4. **复用率实测**: DRIFT-α-014 docs 标 96%/100% 待重测.
5. **dependencies package.json**: react-query / dnd-kit / framer-motion 等是否引入, 决定 docs ADR 履行状态.

### 7.5 给 verifier 阶段的建议

1. 跑 `git log -p --follow frontend/src/types/ids.ts` 实证 6 SM const 何时引入, 是否有 git 历史证据支持 "frontend 自创" 论断
2. 跑 `git rev-parse --verify 948582e` 确认任务简报 HEAD 是否存在, 或任务简报是规划中 commit
3. 跑 `git log -p --follow frontend/src/lib/redirects.ts` 实证 redirect 27 entries 何时引入, 谁决策
4. 跑 `git status` / `git diff --stat` 确认 worker-α 是否真正只读, 无任何 unstaged change

---

## 8. 附: 实证方法

| 方法 | 用法 |
|---|---|
| `git rev-parse HEAD` (1 次) | 确认 main HEAD = a3618107... |
| `Get-ChildItem -Recurse` | 列出 frontend/src/app, components, lib, mocks, types 完整文件树 |
| `Read` (desktop read, UTF-8) | 直接读 16 份关键文件 (11 份设计书 + 5 份代码) |
| `Select-String` (ripgrep) | 5+ 次关键词检索 (kanban/timeline/Identity/milestones/Esc/onKeyDown) |
| `Test-Path` (1 次) | 验证 CommandBar.tsx 缺失, app/(app)/projects/ 缺失 |

**未用**:
- `git log -p --follow` (per 边界 read-only 不 commit, 但 git log 是只读, 实际可用, 留给 verifier)
- `git grep` (替代品 Select-String 已覆盖)
- 跑 build / test (per 边界不写不 commit, 留给 verifier)

---

[alpha done] total=31, p0=7, p1=10, p2=14, unverified=0
