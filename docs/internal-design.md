# Star 平台《Internal Design》(前端组件级详细设计)

> **文档版本**: v0.2 (2026-08-26)
> **修订历史**:
>
> | 版本 | 日期 | 变更 | 审批者 |
> |---|---|---|---|
> | v0.1 | 2026-08-25 | 初始版本 | — |
> | v0.2 | 2026-08-26 | 同步 basic-design 5f1ea5b(新增 ScheduleTriggerForm / InboxFilter / CostSummaryTab / SquadGroupView 占位组件) | — |
> **上游**: `docs/requirements.md` v2.0,`docs/basic-design.md` v0.1,`docs/external-design.md` v0.1,`docs/api-design.md` v0.1
> **下游**: Implementation(React + Vite + TypeScript)
> **文档定位**: External Design 展开为内部组件,定义 React 组件树、状态管理、路由、API 调用层。**仍然不写完整 React 代码**,只到组件级、状态形状、API 调用契约级别。

---

## 上游同步 2026-08-26(继承 basic-design 5f1ea5b)

> 本设计书跟随《基本設計書》5f1ea5b 同步,新增以下占位组件。**不**改 React 组件树主结构 / 状态管理选型:
>
> | 同步项 | 占位组件(Props / State / API 契约) |
> |---|---|
> | **S1** REQ-AUTO-002(Schedule Trigger) | `ScheduleTriggerForm`:Props=`{ rule, onChange }`,State=`{ kind: 'Schedule' \| 'Cron', expression: string }`,API=无(本地解析) |
> | **S2** REQ-NOTIF-002(Inbox 噪声抑制) | `InboxFilter`:Props=`{ defaultScope: 'human' }`,State=`{ audience_scope, requires_human_decision }`,API=`GET /v1/notifications?audience_scope=human` |
> | **S4** AgentSession `cost_summary` | `CostSummaryTab`:Props=`{ agentSessionId }`,State=`{ tokenUsage, costSummary, range }`,API=`GET /v1/agent-sessions/{id}`(取 cost_summary 字段) |
> | **S5** Squad 分组视图(Future) | `SquadGroupView`:Props=`{ projectId }`,State=`{ groupBy: 'squad' }`(只读),API=待定(Future 占位) |
>
> **不变量保留**:React 组件树主结构 / 状态管理选型 / 路由结构全部不动;占位组件按需实现。

---

## 0. 文档说明

### 0.1 文档目的与定位

本文档是《External Design》的"代码组织视角"展开,产出:

- 前端技术选型(React + Vite + TS + 状态管理 + 路由 + UI 库)
- 目录结构
- 关键模块组件树(Worktree Control Center / Detail / Chat / Inbox / Board)
- 状态管理(Server State / Client State / Realtime)
- 路由结构(嵌套路由 + 守卫 + URL 参数)
- API 调用层(React Query Hooks 命名 + Error 处理 + Optimistic Update)
- 共享组件库
- 性能预算(LCP / INP / CLS)
- 测试策略(Vitest + RTL + Playwright)

**范围**:
- ✅ Web SPA(主入口)
- ❌ Local Daemon 桌面 UI(不在范围)
- ❌ 移动 App(V2)

### 0.2 与 External Design 的区分

| 维度 | External Design | Internal Design(本文) |
|---|---|---|
| **受众** | 产品 / 设计 / 架构 | 前端工程师 |
| **抽象层级** | UX / 信息架构 | 代码组织 |
| **产出** | 页面结构 / 用户流程 | 组件树 / 状态 / API 契约 |
| **不产出** | 代码 | 完整生产代码 |

### 0.3 命名约定

- **Feature**:业务功能模块(Worktree / Feedback / Board)
- **Component**:React 组件(可复用)
- **Hook**:React Hook(逻辑复用)
- **Slice**:Zustand 状态切片
- **Query**:TanStack Query 数据查询
- **Mutation**:TanStack Query 数据变更
- **Route**:React Router 路由

### 0.4 引用规则

- `§N` 引用《Requirements》v2.0 章节号(最大 §47)
- 引用《Basic Design》使用 `《Basic Design》§X`
- 引用《External Design》使用 `《External Design》§X`
- 引用《API Design》使用 `《API Design》§X`

---

## 1. 前端技术选型

### 1.1 核心技术栈

| 维度 | 选型 | 理由 |
|---|---|---|
| **构建工具** | Vite 5.x | 启动快,HMR 优秀,生产构建快 |
| **框架** | React 18.x | Concurrent Mode / Suspense 成熟,生态完善 |
| **语言** | TypeScript 5.x(严格模式) | 类型安全,IDE 智能提示 |
| **包管理** | pnpm | 速度快,workspace 支持 |
| **Monorepo** | pnpm workspace | 前端 + 设计系统 + shared libs |
| **测试** | Vitest + React Testing Library + Playwright | Vitest 与 Vite 同源,快;RTL 是社区标准;Playwright E2E |
| **Linting** | ESLint + Prettier + TypeScript-ESLint | 强制规范 |

### 1.2 状态管理选型

#### 1.2.1 候选对比

| 方案 | 适用 | 不适用 |
|---|---|---|
| **TanStack Query v5** | Server State(API 缓存 + 同步) | Client State(UI 状态) |
| **Zustand** | 简单 Client State(slice 形式) | 大型复杂状态(不易调试) |
| **Redux Toolkit** | 大型复杂 Client State + 严格可调试性 | 简单状态(过度) |
| **Jotai** | Atomic 状态 | 共享大型对象 |
| **Context + useReducer** | 跨组件树共享小状态 | 性能敏感(无 selector) |

**最终选型**(推荐):

```text
Server State:        TanStack Query v5(数据缓存 + invalidation + optimistic)
Client State:        Zustand(slice 模式,轻量 + 强类型)
Realtime State:      TanStack Query + WebSocket Subscriber Bridge
Form State:          React Hook Form + Zod
URL State:           React Router 7 useSearchParams
Component-local:     useState / useReducer
```

**理由**:

- **TanStack Query**:与 API 强相关(数据缓存 / stale-while-revalidate / refetch on focus / infinite query)
- **Zustand**:比 Redux Toolkit 简单 70%,slice 模式够用,DevTools 支持
- **不用 Redux**:除非遇到需要时间旅行调试的复杂场景,默认 Zustand

### 1.3 路由选型

**选型**:React Router v7(declarative mode + data router)

**理由**:

- 数据加载 API(`loader` / `action`)
- 嵌套路由原生支持
- TypeScript 类型推断完善
- 社区生态成熟

**不使用**:

- Next.js:不需要 SSR / SSG(本应用是登录后 SaaS)
- Remix:同上
- TanStack Router:生态较小,API 复杂

### 1.4 UI 库选型

| 方案 | 适用 | 评价 |
|---|---|---|
| **MUI v5** | 完整企业 UI(组件齐全 + 主题系统) | 重,但功能多 |
| **Tailwind CSS** | 高度自定义 + 设计 Token | 灵活,需自建组件 |
| **Mantine** | 现代 + 完整 | 适中 |
| **Ant Design** | 中后台 | 偏中国化,设计语言固定 |
| **自建 + Headless UI** | 完全控制 | 工作量大 |

**最终选型**(推荐):

```text
基础样式:    Tailwind CSS 3.x + CSS Variables(主题)
Headless:    Radix UI(Accessible primitives)
Icon:        Lucide React
富文本/MD:   TipTap / MDXEditor
Code Editor: Monaco Editor(VS Code 内核)
Date:        date-fns + react-day-picker
Virtual List: @tanstack/react-virtual
```

**理由**:

- **Tailwind**:Utility-first + 设计 Token 直接映射 CSS 变量
- **Radix UI**:无障碍(headless)+ 可访问,符合 WCAG 2.1 AA(继承《External Design》§7.3)
- **Monaco**:VS Code 同款,代码体验最佳
- **不选 MUI**:包体积大 + 主题定制受限

### 1.5 测试选型

| 层级 | 工具 | 理由 |
|---|---|---|
| **单元** | Vitest | 与 Vite 同源,启动快 |
| **组件** | React Testing Library | 用户视角测试 |
| **E2E** | Playwright | 跨浏览器 + 等待策略智能 |
| **Visual Regression** | Chromatic / Loki(Storybook) | 视觉回归 |
| **A11y** | axe-playwright | 自动 a11y 检测 |

**覆盖目标**(继承《Test Design》):

- 单元:Domain Logic Hooks ≥ 80%
- 组件:Shared Components ≥ 80%
- E2E:关键用户流程 100%(《External Design》§4 的 6 个流程)

### 1.6 监控 / 错误追踪

| 工具 | 用途 |
|---|---|
| **Sentry** | Error Tracking + Source Map |
| **OpenTelemetry JS** | RUM(Real User Monitoring) |
| **PostHog / 自建** | 用户行为分析(可选) |
| **Web Vitals** | LCP / INP / CLS(继承 §8) |

---

## 2. 目录结构

### 2.1 Monorepo 划分

```text
frontend/
├── apps/
│   ├── web/                    # 主 Web SPA
│   │   ├── src/
│   │   ├── public/
│   │   ├── tests/
│   │   ├── index.html
│   │   ├── vite.config.ts
│   │   ├── package.json
│   │   └── tsconfig.json
│   └── storybook/              # 组件库预览
│       ├── src/
│       └── ...
├── packages/
│   ├── ui/                     # 共享组件库(WorktreeCard, AgentStatusBadge 等)
│   │   ├── src/
│   │   └── package.json
│   ├── api-client/             # API Client(OpenAPI 生成)
│   │   ├── src/
│   │   └── package.json
│   ├── realtime/               # WebSocket 客户端
│   │   ├── src/
│   │   └── package.json
│   ├── i18n/                   # i18n 配置 + 翻译
│   │   ├── locales/
│   │   └── package.json
│   ├── auth/                   # 认证 + 权限
│   │   ├── src/
│   │   └── package.json
│   ├── hooks/                  # 通用 Hooks(useDebounce, useLocalStorage 等)
│   │   ├── src/
│   │   └── package.json
│   ├── utils/                  # 通用工具
│   │   ├── src/
│   │   └── package.json
│   ├── config/                 # 共享配置
│   │   ├── eslint/
│   │   ├── tsconfig/
│   │   └── tailwind/
│   └── testing/                # 测试工具
│       ├── src/
│       └── package.json
├── pnpm-workspace.yaml
├── package.json
└── tsconfig.base.json
```

### 2.2 apps/web 详细结构

```text
apps/web/src/
├── app/                        # 应用入口
│   ├── App.tsx                 # 根组件
│   ├── router.tsx              # 路由配置
│   ├── providers.tsx           # 全局 Provider(QueryClient, Auth, Theme, I18n)
│   └── error-boundary.tsx      # 顶层 Error Boundary
├── features/                   # 业务功能模块
│   ├── auth/
│   │   ├── components/
│   │   │   ├── LoginForm.tsx
│   │   │   ├── TenantPicker.tsx
│   │   │   └── BootstrapCodeInput.tsx
│   │   ├── hooks/
│   │   │   ├── useAuth.ts
│   │   │   ├── useTenant.ts
│   │   │   └── useBootstrap.ts
│   │   ├── api/
│   │   │   └── auth-api.ts
│   │   ├── store/
│   │   │   └── auth-slice.ts
│   │   └── routes/
│   │       ├── LoginPage.tsx
│   │       └── TenantPickerPage.tsx
│   ├── worktree/               # 核心模块
│   │   ├── components/
│   │   │   ├── WorktreeControlCenter/
│   │   │   │   ├── WorktreeControlCenter.tsx
│   │   │   │   ├── WorktreeTable.tsx
│   │   │   │   ├── WorktreeCard.tsx
│   │   │   │   ├── WorktreeHeatmap.tsx
│   │   │   │   ├── WorktreeFilterBar.tsx
│   │   │   │   ├── WorktreeStatusBadge.tsx
│   │   │   │   └── index.ts
│   │   │   ├── WorktreeDetail/
│   │   │   │   ├── WorktreeDetail.tsx
│   │   │   │   ├── WorktreeOverviewTab.tsx
│   │   │   │   ├── WorktreeDiffTab.tsx
│   │   │   │   ├── WorktreeTestsTab.tsx
│   │   │   │   ├── WorktreeFeedbackTab.tsx
│   │   │   │   ├── WorktreeActivityTab.tsx
│   │   │   │   └── index.ts
│   │   │   ├── CreateWorktreeDialog.tsx
│   │   │   ├── AssignAgentDialog.tsx
│   │   │   └── ConflictResolver.tsx
│   │   ├── hooks/
│   │   │   ├── useWorktreeList.ts
│   │   │   ├── useWorktree.ts
│   │   │   ├── useCreateWorktree.ts
│   │   │   ├── useAssignAgent.ts
│   │   │   ├── useStopAgent.ts
│   │   │   ├── useWorktreeStatus.ts        # Realtime
│   │   │   └── useConflictDetection.ts
│   │   ├── api/
│   │   │   └── worktree-api.ts
│   │   ├── store/
│   │   │   └── worktree-filters-slice.ts
│   │   ├── types/
│   │   │   └── worktree.ts
│   │   └── routes/
│   │       ├── WorktreeControlCenterPage.tsx
│   │       └── WorktreeDetailPage.tsx
│   ├── feedback/
│   │   ├── components/
│   │   │   ├── FeedbackInbox.tsx
│   │   │   ├── FeedbackItem.tsx
│   │   │   ├── FeedbackForm.tsx              # 5 段式
│   │   │   └── InterventionQueue.tsx
│   │   ├── hooks/
│   │   │   ├── useFeedbackList.ts
│   │   │   ├── useSubmitFeedback.ts
│   │   │   └── useResolveFeedback.ts
│   │   ├── api/
│   │   │   └── feedback-api.ts
│   │   └── routes/
│   │       ├── FeedbackInboxPage.tsx
│   │       └── InterventionQueuePage.tsx
│   ├── agent/
│   │   ├── components/
│   │   │   ├── AgentStatusBadge.tsx
│   │   │   ├── AgentChat.tsx
│   │   │   ├── ChatMessage.tsx
│   │   │   └── HandoffDialog.tsx
│   │   ├── hooks/
│   │   │   ├── useAgentSession.ts
│   │   │   └── useAgentChat.ts
│   │   ├── api/
│   │   │   └── agent-api.ts
│   │   └── routes/
│   │       └── AgentChatPage.tsx
│   ├── workitem/
│   │   ├── components/
│   │   │   ├── WorkItemDetail.tsx
│   │   │   ├── AcceptanceCriteriaList.tsx
│   │   │   ├── WorkItemTreeList.tsx
│   │   │   └── WorkItemActivity.tsx
│   │   ├── hooks/
│   │   │   ├── useWorkItem.ts
│   │   │   └── useWorkItemTree.ts
│   │   ├── api/
│   │   │   └── workitem-api.ts
│   │   └── routes/
│   │       └── WorkItemDetailPage.tsx
│   ├── board/
│   │   ├── components/
│   │   │   ├── Board.tsx
│   │   │   ├── BoardColumn.tsx
│   │   │   ├── BoardCard.tsx
│   │   │   ├── Swimlane.tsx
│   │   │   └── WipLimit.tsx
│   │   ├── hooks/
│   │   │   ├── useBoard.ts
│   │   │   └── useMoveCard.ts
│   │   ├── api/
│   │   │   └── board-api.ts
│   │   └── routes/
│   │       └── BoardPage.tsx
│   ├── planning/                # Sprint / Backlog / Roadmap
│   │   ├── components/
│   │   │   ├── SprintPlanning.tsx
│   │   │   ├── BurndownChart.tsx
│   │   │   ├── GanttChart.tsx
│   │   │   └── Backlog.tsx
│   │   ├── hooks/
│   │   │   ├── useSprint.ts
│   │   │   └── useBacklog.ts
│   │   └── routes/
│   │       ├── SprintPage.tsx
│   │       ├── BacklogPage.tsx
│   │       └── RoadmapPage.tsx
│   ├── settings/
│   │   ├── components/
│   │   │   ├── ProfileTab.tsx
│   │   │   ├── ProjectPolicyTab.tsx
│   │   │   ├── AgentPolicyTab.tsx
│   │   │   ├── ProviderDataBoundaryTab.tsx
│   │   │   ├── IntegrationsTab.tsx
│   │   │   ├── MembersTab.tsx
│   │   │   ├── AuditTab.tsx
│   │   │   └── LocalRuntimeTab.tsx
│   │   ├── hooks/
│   │   │   ├── useProjectPolicy.ts
│   │   │   └── useAgentPolicy.ts
│   │   └── routes/
│   │       └── SettingsPage.tsx
│   ├── search/
│   │   ├── components/
│   │   │   ├── CommandPalette.tsx
│   │   │   └── GlobalSearch.tsx
│   │   ├── hooks/
│   │   │   └── useGlobalSearch.ts
│   │   └── api/
│   │       └── search-api.ts
│   └── notifications/
│       ├── components/
│       │   ├── NotificationBell.tsx
│       │   ├── NotificationCenter.tsx
│       │   └── NotificationItem.tsx
│       ├── hooks/
│       │   └── useNotifications.ts
│       └── store/
│           └── notification-slice.ts
├── components/                  # 跨 feature 共享组件
│   ├── layout/
│   │   ├── AppShell.tsx
│   │   ├── TopBar.tsx
│   │   ├── SideNav.tsx
│   │   ├── StatusBar.tsx
│   │   └── TenantSwitcher.tsx
│   ├── data/
│   │   ├── DataTable.tsx
│   │   ├── FilterBar.tsx
│   │   ├── Pagination.tsx
│   │   ├── EmptyState.tsx
│   │   └── ErrorBoundary.tsx
│   ├── feedback/
│   │   ├── Toast.tsx
│   │   ├── ConfirmDialog.tsx
│   │   ├── ProgressBar.tsx
│   │   └── Spinner.tsx
│   ├── display/
│   │   ├── StatusPill.tsx
│   │   ├── PriorityBadge.tsx
│   │   ├── Tag.tsx
│   │   ├── Avatar.tsx
│   │   └── CodeBlock.tsx
│   └── forms/
│       ├── TextField.tsx
│       ├── SelectField.tsx
│       ├── MultiSelectField.tsx
│       ├── DatePickerField.tsx
│       └── FileUploadField.tsx
├── hooks/                       # 全局通用 Hooks
│   ├── useDebounce.ts
│   ├── useLocalStorage.ts
│   ├── useMediaQuery.ts
│   ├── useDocumentTitle.ts
│   ├── useKeyboardShortcut.ts
│   ├── useCopyToClipboard.ts
│   └── useInterval.ts
├── lib/                         # 工具库
│   ├── api/
│   │   ├── client.ts            # Axios / Fetch 封装
│   │   ├── errors.ts            # 错误处理
│   │   └── interceptors.ts      # 401 重定向, CSRF 等
│   ├── realtime/
│   │   ├── client.ts            # WebSocket Client
│   │   ├── subscriber.ts        # 订阅 Pattern
│   │   └── reconnect.ts         # 重连策略
│   ├── auth/
│   │   ├── token.ts             # Token 管理
│   │   └── permissions.ts       # 权限检查
│   ├── i18n/
│   │   └── config.ts
│   ├── theme/
│   │   └── tokens.ts            # 设计 Token 转 Tailwind Config
│   └── utils/
│       ├── format.ts            # 数字 / 日期 / 相对时间
│       ├── classnames.ts
│       └── url.ts
├── store/                       # 全局 Zustand Store
│   ├── index.ts                 # 根 Store
│   ├── slices/
│   │   ├── ui-slice.ts          # 全局 UI 状态
│   │   ├── theme-slice.ts
│   │   └── selection-slice.ts   # 多选状态
├── pages/                       # 顶层页面(很少,主要是 redirect)
│   ├── NotFoundPage.tsx
│   ├── ForbiddenPage.tsx
│   └── ServerErrorPage.tsx
├── styles/                      # 全局样式
│   ├── globals.css
│   ├── tailwind.css
│   └── tokens.css
├── types/                       # 全局类型
│   ├── api.ts
│   ├── domain.ts
│   └── env.d.ts
├── test/                        # 测试工具
│   ├── setup.ts
│   ├── mocks/
│   │   ├── server.ts            # MSW(Mock Service Worker)
│   │   └── handlers.ts
│   └── factories/               # 数据工厂
│       ├── worktree.ts
│       └── workitem.ts
├── main.tsx                     # 入口
└── vite-env.d.ts
```

### 2.3 packages/ui 共享组件库

```text
packages/ui/src/
├── components/
│   ├── WorktreeCard/
│   │   ├── WorktreeCard.tsx
│   │   ├── WorktreeCard.test.tsx
│   │   ├── WorktreeCard.stories.tsx
│   │   └── index.ts
│   ├── AgentStatusBadge/
│   ├── DiffViewer/
│   ├── TestResultList/
│   ├── ConflictHeatmap/
│   ├── FeedbackItem/
│   ├── StatusPill/
│   ├── PriorityBadge/
│   ├── TenantSwitcher/
│   ├── CommandPalette/
│   ├── DataTable/
│   └── ...
├── hooks/
│   └── ...
├── tokens/                      # 设计 Token 转 Tailwind Preset
│   ├── colors.ts
│   ├── spacing.ts
│   ├── typography.ts
│   └── index.ts
├── utils/
│   └── ...
├── theme/
│   └── presets.ts
└── index.ts                     # Barrel Export
```

---

## 3. 关键模块组件树

### 3.1 Worktree Control Center 子树

```mermaid
flowchart TB
    Page[WorktreeControlCenterPage<br/>URL: /worktrees]
    Page --> Header[PageHeader<br/>(title, actions, refresh)]
    Page --> Filter[FilterBar<br/>(status, agent, project, repo, search)]
    Page --> Group[GroupSelector<br/>(project / agent / workitem / status)]
    Page --> View[ViewSwitcher<br/>(table / card / heatmap)]
    Page --> Body[Body]

    Body --> TableView[WorktreeTableView]
    Body --> CardView[WorktreeCardGridView]
    Body --> HeatmapView[WorktreeHeatmapView]

    TableView --> Table[DataTable<br/>(virtual scroll)]
    Table --> Row[WorktreeRow]
    Row --> StatusCol[StatusPill]
    Row --> AgentCol[AgentStatusBadge]
    Row --> ActionCol[RowActions<br/>(view, stop, feedback)]

    CardView --> Grid[VirtualGrid]
    Grid --> Card[WorktreeCard]
    Card --> Status[StatusPill]
    Card --> Agent[AgentStatusBadge]
    Card --> Tests[TestSummary]
    Card --> Conflict[ConflictIndicator]
    Card --> Actions[CardActions]

    HeatmapView --> Heatmap[ConflictHeatmap]
    Heatmap --> Cell[HeatmapCell<br/>(file/symbol)]
    Cell --> Tooltip[OverlapTooltip]

    Page --> Pagination[Pagination]
    Page --> StatusBar[StatusBar<br/>(connection, daemon, agent count)]
```

**关键设计**:

- 3 个 View 共享 `WorktreeListData`(TanStack Query)
- `WorktreeRow` / `WorktreeCard` / `HeatmapCell` 共享 `WorktreeSummary` 类型
- View 切换不重新请求数据

### 3.2 Worktree Detail 子树

```mermaid
flowchart TB
    Page[WorktreeDetailPage<br/>URL: /worktrees/:id]
    Page --> Header[PageHeader<br/>(branch, status, actions)]
    Header --> Status[StatusPill]
    Header --> Actions[HeaderActions<br/>(view workitem, open IDE, stop, more)]

    Page --> Tabs[Tabs]
    Tabs --> Overview[OverviewTab]
    Tabs --> Diff[DiffTab]
    Tabs --> Tests[TestsTab]
    Tabs --> Feedback[FeedbackTab]
    Tabs --> Activity[ActivityTab]

    Overview --> Summary[StatusSummary]
    Overview --> Agent[AgentSessionPanel]
    Overview --> PR[PullRequestPanel]
    Overview --> Conflict[ConflictPanel]
    Overview --> QuickActions[QuickActionButtons]

    Diff --> DiffViewer[DiffViewer<br/>(Monaco or unified)]
    Diff --> FileTree[FileTree]
    DiffViewer --> SymbolSidebar[SymbolSidebar]

    Tests --> TestList[TestResultList]
    TestList --> TestItem[TestItem]
    TestItem --> StackTrace[StackTrace<br/>(collapsible)]

    Feedback --> FeedbackList[FeedbackList]
    FeedbackList --> FeedbackItem[FeedbackItem]
    Feedback --> NewFeedback[NewFeedbackButton]

    Activity --> Timeline[ActivityTimeline]
    Timeline --> EventItem[EventItem<br/>(virtual scroll)]

    Page --> SideDrawer[SideDrawer<br/>(chat, conflict resolver, handoff)]
```

**关键设计**:

- Tabs 切换保留已加载数据(不重新请求)
- Diff 走 Monaco Editor + Object Storage(继承《External Design》§6.3)
- Activity 走虚拟列表(>1000 events)

### 3.3 Agent Chat 子树

```mermaid
flowchart TB
    Page[AgentChatPage<br/>URL: /worktrees/:id/chat]
    Page --> Header[ChatHeader<br/>(agent type, handoff button)]
    Page --> Messages[MessageList<br/>(virtual scroll)]
    Messages --> UserMsg[UserMessage]
    Messages --> AgentMsg[AgentMessage]
    Messages --> ToolCall[ToolCallBlock]
    Messages --> DecisionCard[DecisionCard<br/>(when active)]
    AgentMsg --> Actions[MessageActions<br/>(apply as decision, apply as feedback, cite symbol)]
    Page --> Input[ChatInput<br/>(autocomplete, mention, file attach)]
```

**关键设计**:

- 消息走 Realtime(WS 推送,继承《API Design》§4)
- 输入框支持 @ 提及、文件引用、Symbol 引用
- 消息 Actions 把 Chat 内容升格为 Decision / Feedback

### 3.4 Feedback Inbox 子树

```mermaid
flowchart TB
    Page[FeedbackInboxPage<br/>URL: /inbox/feedback]
    Page --> Filter[FilterBar]
    Page --> Sort[SortSelector]
    Page --> Group[GroupBySelector]
    Page --> List[FeedbackList<br/>(virtual scroll)]
    List --> Group[GroupHeader]
    List --> Item[FeedbackItem<br/>(compact or detailed)]
    Item --> Actions[QuickActions<br/>(view, resolve, edit, supersede)]
    Page --> EmptyState[EmptyState<br/>("All caught up!")]
    Page --> Pagination[Pagination]
```

### 3.5 Board 子树

```mermaid
flowchart TB
    Page[BoardPage<br/>URL: /board]
    Page --> BoardHeader[BoardHeader<br/>(sprint picker, view mode)]
    Page --> Filter[BoardFilterBar]
    Page --> Board[Board<br/>(kanban or scrum)]
    Board --> Column[BoardColumn<br/>(status)]
    Column --> WipLimit[WipLimitIndicator]
    Column --> CardList[CardList<br/>(drag and drop)]
    CardList --> Card[BoardCard]
    Card --> WorkItemSummary[WorkItemSummary]
    Card --> Assignee[Assignee]
    Card --> Priority[PriorityBadge]
    Card --> Points[StoryPoints]
    Page --> Swimlane[Swimlane<br/>(assignee / epic / priority)]
```

**关键设计**:

- 拖动用 `@dnd-kit/core`(无障碍,触屏支持)
- WIP Limit 超出时,Column 红色高亮 + 提示
- 状态变更走 Optimistic Update

### 3.6 前端模块 ↔ 22 domain 协作映射 (v0.16 模块间协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26 Worktree Orchestration 跨 12 domain 协作范围](../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),本节定义前端 5 大模块与 22 domain 的协作入口。

| 前端模块 | 涉及 22 domain (核心 5) | API 入口 | Realtime 通道 (per [basic-design v0.16 §4.13](../../../basic-design.md)) | 状态机交互 |
|---|---|---|---|---|
| **Worktree Control Center** (§3.1) | worktree (主) + work-item + agent + validation + collaboration | `GET /v1/worktrees?filter=` + `GET /v1/worktrees/{id}/observed_state` | `/ws/feed` (高频 observed state 推送) | WorktreeStatusObserved 事件流 |
| **Worktree Detail** (§3.2) | worktree (主) + agent + feedback + scm + validation | `GET /v1/worktrees/{id}` + `GET /v1/changesets?worktree_id=` + `GET /v1/feedback?target=worktree:{id}` | `/ws/feed` + `/ws/notif` (ValidationFailed 降噪触发) | Worktree 状态机 6 转换 (per [basic-design v0.16 §4.1.3](../../../basic-design.md)) |
| **Agent Chat** (§3.3) | agent (主) + worktree + context + feedback | `POST /v1/agent_sessions/{id}/messages` + `GET /v1/context_packets?worktree_id=` | `/ws/feed` (Agent 实时消息) | AgentSessionStarted / Completed 事件流 |
| **Feedback Inbox** (§3.4) | feedback (主) + work-item + worktree + agent + notification | `GET /v1/feedback?status=open&assignee=me` + `PATCH /v1/feedback/{id}` | `/ws/notif` (FeedbackCreated 降噪触发) | FeedbackCreated / Acknowledged / Applied / Verified 4 状态 (per [basic-design v0.16 §4.12.1](../../../basic-design.md)) |
| **Board** (§3.5) | work-item (主) + workflow + planning + project + comment | `GET /v1/boards/{project_id}/columns` + `PATCH /v1/work_items/{id}` (transition) | `/ws/notif` (WorkItem StateChanged 触发) | WorkItem 状态机 + Workflow Guard 校验 (per REQ-WF-003) |

**前端模块依赖的 22 domain 关键 Port** (per [basic-design v0.16 §3.2 contact face 表](../../../basic-design.md)):
- 所有模块走 `domain-permission` PermissionChecker Port 鉴权 (per [basic-design v0.16 §3.2.8 permission 横切](../../../basic-design.md))
- 实时性模块走 `domain-collaboration` + `star-sse` Realtime 推送 (per §D29 Realtime 3 通道)
- 写操作走 Application Service (单 PG 事务,per [basic-design v0.16 §2.4 跨域事务](../../../basic-design.md)),不走 Event Chain
- 跨域写走 Saga (per [spec/saga/01 v0.2 §4 Worktree Orchestration Saga 8 步](../architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md))

---

## 4. 状态管理

### 4.1 Server State(TanStack Query)

#### 4.1.1 QueryClient 配置

```typescript
// apps/web/src/app/providers.tsx
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,            // 30s 内不重新 fetch
      cacheTime: 5 * 60_000,        // 5min 缓存
      refetchOnWindowFocus: true,
      refetchOnReconnect: true,
      retry: (failureCount, error) => {
        if (error instanceof AuthError) return false;
        return failureCount < 3;
      },
    },
    mutations: {
      retry: 0,                     // Mutation 不重试
    },
  },
});
```

#### 4.1.2 Query Key 命名规范

**统一格式**:`[scope, entity, action, params...]`

```typescript
// 例子
['worktree', 'list', { filter, sort, group }]
['worktree', 'detail', worktreeId]
['worktree', 'status', worktreeId]  // Realtime
['feedback', 'inbox', { filter, sort }]
['workitem', 'detail', workItemId]
['agent', 'session', sessionId]
['board', { projectId, sprintId }]
['search', 'global', { query }]
```

**前缀**:

- `['worktree', ...]`:Worktree 数据
- `['feedback', ...]`:Feedback 数据
- `['workitem', ...]`:WorkItem 数据
- `['agent', ...]`:Agent 数据
- `['realtime', ...]`:Realtime 状态

#### 4.1.3 Invalidation 策略

| Mutation | 失效 Query Keys |
|---|---|
| `useCreateWorktree` | `['worktree', 'list', ...]` |
| `useAssignAgent` | `['worktree', 'detail', id]`, `['agent', 'session', ...]` |
| `useStopAgent` | `['agent', 'session', id]`, `['worktree', 'detail', id]` |
| `useSubmitFeedback` | `['feedback', 'inbox', ...]`, `['worktree', 'detail', id]` |
| `useResolveFeedback` | `['feedback', 'inbox', ...]`, `['worktree', 'detail', id]` |
| `useMoveCard` (Board) | `['board', ...]`, `['workitem', 'detail', id]` |

### 4.2 Client State(Zustand Slices)

#### 4.2.1 全局 Slices

```typescript
// store/slices/ui-slice.ts
interface UISlice {
  // Sidebar
  sidebar_collapsed: boolean;
  toggle_sidebar: () => void;
  // Command Palette
  command_palette_open: boolean;
  open_command_palette: () => void;
  close_command_palette: () => void;
  // Theme
  theme: 'light' | 'dark' | 'system';
  set_theme: (t: 'light' | 'dark' | 'system') => void;
  // Modal
  active_modal: string | null;
  open_modal: (id: string, props?: any) => void;
  close_modal: () => void;
}

// store/slices/selection-slice.ts
interface SelectionSlice {
  // Worktree multi-select
  selected_worktree_ids: Set<string>;
  toggle_worktree_selection: (id: string) => void;
  clear_selection: () => void;
  // Bulk action
  active_bulk_action: string | null;
  start_bulk_action: (action: string) => void;
  end_bulk_action: () => void;
}
```

#### 4.2.2 Feature-local Slices

- `worktree-filters-slice.ts`:Worktree Control Center 过滤器状态(可与 URL 同步)
- `chat-draft-slice.ts`:Chat 草稿持久化

#### 4.2.3 持久化

| Slice | 持久化 | 方式 |
|---|---|---|
| `theme-slice` | ✅ | LocalStorage(用户偏好) |
| `ui-slice`(sidebar) | ✅ | LocalStorage |
| `selection-slice` | ❌ | 内存(刷新清除) |
| `chat-draft-slice` | ✅ | LocalStorage(草稿保留) |
| `worktree-filters-slice` | ❌(用 URL) | URL Search Params |

### 4.3 Realtime 集成(WS 订阅,继承《API Design》§4)

#### 4.3.1 WS Client 架构

```text
┌────────────────────────────────────────────┐
│ RealtimeClient (Singleton)                  │
│ - WebSocket connection + auto-reconnect     │
│ - Auth (token)                              │
│ - Subscription registry                     │
└────────────────────────────────────────────┘
                    │
                    │ subscribe(topic, handler)
                    ▼
┌────────────────────────────────────────────┐
│ QueryCacheBridge                            │
│ - on message → invalidate Query Keys       │
│ - on message → setQueryData                │
└────────────────────────────────────────────┘
                    │
                    │ useQuery(['worktree', 'detail', id])
                    ▼
┌────────────────────────────────────────────┐
│ React Component (auto re-render)           │
└────────────────────────────────────────────┘
```

#### 4.3.2 订阅 Pattern

```typescript
// hooks/useWorktreeStatus.ts
function useWorktreeStatus(worktreeId: string) {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unsubscribe = realtimeClient.subscribe(
      `worktree.${worktreeId}.status`,
      (event) => {
        // 方式 1: setQueryData 直接更新
        queryClient.setQueryData(['worktree', 'detail', worktreeId], (prev) => ({
          ...prev,
          status: event.status,
          last_observed_at: event.timestamp,
        }));

        // 方式 2: invalidate 触发 refetch
        // queryClient.invalidateQueries(['worktree', 'list']);
      }
    );
    return unsubscribe;
  }, [worktreeId, queryClient]);

  return useQuery({
    queryKey: ['worktree', 'detail', worktreeId],
    queryFn: () => worktreeApi.getWorktree(worktreeId),
  });
}
```

#### 4.3.3 WS Topic 列表(继承《API Design》§4.2)

```text
worktree.{id}.status          (status 变更)
worktree.{id}.observation     (high-freq 观察)
agent.{session_id}.status     (14 状态变更)
agent.{session_id}.event      (tool call, message)
feedback.{id}.status          (Feedback 状态)
workitem.{id}.update          (字段更新)
project.{id}.board            (Board 状态)
tenant.{id}.notification      (通知)
user.{id}.notification        (个人通知)
```

#### 4.3.4 Stale Handling(继承《Requirements》§23.4)

- 客户端记录 `last_observed_at`
- UI 显式标注"Possibly Stale"(若 > 5min 无新事件)
- 显示连接状态(Online / Reconnecting / Offline)

### 4.4 Form State(React Hook Form + Zod)

```typescript
// 使用 React Hook Form + Zod 做表单验证
const schema = z.object({
  branch: z.string().min(1).max(100),
  agent_type: z.enum(['codex', 'claude_code', 'gemini_cli']),
  policy: z.object({
    max_runtime_seconds: z.number().min(60).max(86400),
    require_test: z.boolean(),
    require_review: z.boolean(),
  }),
});

const { register, handleSubmit, formState: { errors } } = useForm({
  resolver: zodResolver(schema),
});
```

### 4.5 Optimistic Update 模式

**典型例子**:Board Card 拖动

```typescript
function useMoveCard() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ cardId, toColumn, toIndex }) =>
      boardApi.moveCard({ cardId, toColumn, toIndex }),

    // Optimistic Update
    onMutate: async ({ cardId, toColumn, toIndex }) => {
      // 1. 取消相关 refetch
      await queryClient.cancelQueries(['board']);

      // 2. 保存旧状态
      const previous = queryClient.getQueryData(['board']);

      // 3. 乐观更新
      queryClient.setQueryData(['board'], (old) => {
        return moveCardInBoard(old, cardId, toColumn, toIndex);
      });

      return { previous };
    },

    // 失败回滚
    onError: (err, vars, context) => {
      queryClient.setQueryData(['board'], context.previous);
    },

    // 成功后 invalidate
    onSettled: () => {
      queryClient.invalidateQueries(['board']);
    },
  });
}
```

---

## 5. 路由结构

### 5.1 路由配置(React Router v7 Data Router)

```typescript
// app/router.tsx
const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/tenant-pick',
    element: <TenantPickerPage />,
  },
  {
    path: '/',
    element: <ProtectedRoute><AppShell /></ProtectedRoute>,
    children: [
      { index: true, element: <Navigate to="/worktrees" replace /> },
      {
        path: 'worktrees',
        children: [
          { index: true, element: <WorktreeControlCenterPage /> },
          { path: ':id', element: <WorktreeDetailPage /> },
          { path: ':id/chat', element: <AgentChatPage /> },
          { path: ':id/diff/:file?', element: <DiffViewerPage /> },
        ],
      },
      {
        path: 'inbox',
        children: [
          { path: 'feedback', element: <FeedbackInboxPage /> },
          { path: 'intervention', element: <InterventionQueuePage /> },
        ],
      },
      {
        path: 'board',
        element: <BoardPage />,
      },
      {
        path: 'backlog',
        element: <BacklogPage />,
      },
      {
        path: 'sprint/:id',
        children: [
          { index: true, element: <SprintPage /> },
          { path: 'planning', element: <SprintPlanningPage /> },
        ],
      },
      {
        path: 'roadmap',
        children: [
          { index: true, element: <RoadmapPage /> },
          { path: 'gantt/:projectId', element: <GanttPage /> },
        ],
      },
      {
        path: 'workitems/:id',
        element: <WorkItemDetailPage />,
      },
      {
        path: 'settings',
        children: [
          { index: true, element: <Navigate to="profile" replace /> },
          { path: ':tab', element: <SettingsPage /> },
        ],
      },
    ],
  },
  {
    path: '*',
    element: <NotFoundPage />,
  },
]);
```

### 5.2 路由守卫

#### 5.2.1 Auth Guard

```typescript
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();

  if (isLoading) return <FullPageSpinner />;

  if (!user) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  if (!user.tenant_id) {
    return <Navigate to="/tenant-pick" replace />;
  }

  return <>{children}</>;
}
```

#### 5.2.2 Permission Guard

```typescript
function RequirePermission({
  permission,
  children,
  fallback,
}: {
  permission: Permission;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}) {
  const { hasPermission } = usePermissions();

  if (!hasPermission(permission)) {
    return fallback ?? <ForbiddenPage />;
  }

  return <>{children}</>;
}

// 用法
<Route
  path="settings/audit"
  element={
    <RequirePermission permission="audit.read">
      <AuditTab />
    </RequirePermission>
  }
/>
```

### 5.3 URL 参数设计

**Filter / Sort 状态走 URL**(可分享 / 可后退):

```text
/worktrees
  ?status=running,blocked
  &agent=codex
  &project=acme
  &sort=created_at:desc
  &group=project
  &page=1
  &page_size=50
```

**读取 / 写入**:

```typescript
// 用 React Router 的 useSearchParams
const [searchParams, setSearchParams] = useSearchParams();

const filters = useMemo(() => ({
  status: searchParams.get('status')?.split(',') ?? [],
  agent: searchParams.get('agent') ?? null,
  // ...
}), [searchParams]);

const updateFilter = (key: string, value: string | null) => {
  setSearchParams(prev => {
    if (value === null) prev.delete(key);
    else prev.set(key, value);
    return prev;
  });
};
```

---

## 6. API 调用层

### 6.1 React Query Hooks 命名规范

**统一格式**:`use{Entity}{Action}`

| Hook | 用途 | HTTP Method |
|---|---|---|
| `useWorktreeList` | 列表 | GET |
| `useWorktree` | 单个 | GET |
| `useCreateWorktree` | 创建 | POST |
| `useUpdateWorktree` | 更新 | PATCH |
| `useDeleteWorktree` | 删除 | DELETE |
| `useAssignAgent` | 分配 | POST |
| `useStopAgent` | 停止 | POST |
| `useFeedbackList` | 列表 | GET |
| `useFeedback` | 单个 | GET |
| `useSubmitFeedback` | 提交 | POST |
| `useResolveFeedback` | 解决 | POST |
| `useWorkItem` | 单个 | GET |
| `useCreateWorkItem` | 创建 | POST |
| `useMoveCard` | 移动(Board) | PATCH |
| `useAgentSession` | Session | GET |
| `useStartAgentSession` | 启动 | POST |
| `useAgentChat` | Chat 历史 | GET |
| `useSendMessage` | 发送消息 | POST |

### 6.2 API Client 封装

```typescript
// lib/api/client.ts
import { OpenAPIClient } from '@star/api-client';  // 从 OpenAPI 自动生成

const baseURL = import.meta.env.VITE_API_BASE_URL;

export const apiClient = new OpenAPIClient({
  baseURL,
  headers: {
    'X-Client': 'star-web',
    'X-Version': import.meta.env.VITE_APP_VERSION,
  },
  interceptors: {
    request: [
      // 1. 添加 CSRF Token
      (config) => {
        const csrf = getCsrfToken();
        if (csrf) config.headers['X-CSRF-Token'] = csrf;
        return config;
      },
      // 2. 添加 Tenant ID
      (config) => {
        const tenantId = getCurrentTenantId();
        if (tenantId) config.headers['X-Tenant-Id'] = tenantId;
        return config;
      },
    ],
    response: [
      // 1. 401 重定向到 login
      (error) => {
        if (error.response?.status === 401) {
          redirectToLogin();
        }
        return Promise.reject(error);
      },
    ],
  },
});
```

### 6.3 Error 处理(继承《API Design》§8 错误码)

**错误分类**:

```typescript
class ApiError extends Error {
  constructor(
    public code: string,        // E_AUTH_INVALID / E_WORKTREE_NOT_FOUND / ...
    public status: number,
    public details?: Record<string, any>,
    public trace_id?: string,
  ) {
    super(`${code}: ${details?.message ?? 'Unknown error'}`);
  }
}

class AuthError extends ApiError {}         // 401, 重定向
class PermissionError extends ApiError {}   // 403, Forbidden Page
class NotFoundError extends ApiError {}     // 404
class ValidationError extends ApiError {}   // 422, 表单错误
class ConflictError extends ApiError {}     // 409
class RateLimitError extends ApiError {}    // 429
class ServerError extends ApiError {}       // 5xx, 重试 + 通知
class NetworkError extends Error {}         // 无响应
```

**Query Hook Error 处理**:

```typescript
function useWorktree(id: string) {
  return useQuery({
    queryKey: ['worktree', 'detail', id],
    queryFn: () => worktreeApi.getWorktree(id),
    onError: (error) => {
      if (error instanceof NotFoundError) {
        // 跳 404
        navigate('/404');
      } else if (error instanceof ServerError) {
        // Toast + Sentry
        toast.error('Failed to load worktree');
        Sentry.captureException(error);
      }
    },
  });
}
```

### 6.4 Optimistic Update 通用模式

(见 §4.5 例子)

### 6.5 缓存策略

| 数据 | staleTime | cacheTime | refetchOnFocus |
|---|---|---|---|
| Worktree 列表 | 30s | 5min | ✅ |
| Worktree 详情 | 10s | 5min | ✅ |
| WorkItem 详情 | 30s | 5min | ✅ |
| Feedback Inbox | 10s | 5min | ✅ |
| Board | 30s | 5min | ✅ |
| User Profile | 5min | 30min | ❌(稳定) |
| Search Result | 0 | 1min | ❌ |
| Project Policy | 5min | 30min | ❌ |

---

## 7. 共享组件库(packages/ui)

### 7.1 组件 API 设计原则

- ✅ 强类型 Props(TS interface)
- ✅ 默认值合理
- ✅ 受控 / 非受控双模式(关键组件)
- ✅ a11y 属性透传
- ✅ forwardRef 支持
- ✅ as / className 透传
- ✅ Storybook + Chromatic

### 7.2 WorktreeCard API

```typescript
interface WorktreeCardProps {
  // 核心数据
  worktree: WorktreeSummary;
  agentSession?: AgentSessionSummary;
  workItem: WorkItemSummary;
  validationSummary?: ValidationSummary;
  conflictReport?: ConflictReport;

  // 事件
  onClick?: () => void;
  onAction?: (action: WorktreeAction) => void;

  // 显示
  variant?: 'compact' | 'detailed';  // default: 'detailed'
  className?: string;

  // 状态
  isSelected?: boolean;
  isStale?: boolean;
  lastObservedAt?: string;
}

type WorktreeAction =
  | 'view'
  | 'view_workitem'
  | 'open_ide'
  | 'stop_agent'
  | 'run_test'
  | 'create_pr'
  | 'resolve_conflict'
  | 'submit_feedback';
```

### 7.3 AgentStatusBadge API

```typescript
interface AgentStatusBadgeProps {
  status: AgentSessionStatus;  // 14 状态枚举
  pid?: number;                // 进程 PID(debug 模式)
  elapsedSeconds?: number;     // 已运行时长
  showLabel?: boolean;        // 显示状态文字
  size?: 'sm' | 'md' | 'lg';
  onClick?: () => void;
  isStale?: boolean;
}
```

### 7.4 DiffViewer API

```typescript
interface DiffViewerProps {
  diffHandle: DiffHandle;
  fileFilter?: string[];
  symbolFilter?: SymbolRef[];
  viewMode?: 'unified' | 'split' | 'inline';
  highlightFeedbackIds?: FeedbackId[];
  showSymbols?: boolean;
  onSymbolClick?: (symbol: SymbolRef) => void;
  onFileClick?: (path: string) => void;
  maxHeight?: number;          // 像素
}
```

### 7.5 TestResultList API

```typescript
interface TestResultListProps {
  validationResult: ValidationResult;
  testReports: TestReport[];
  showEvidence?: boolean;
  onTestClick?: (testId: string) => void;
  groupBy?: 'file' | 'status' | 'duration';
}
```

### 7.6 ConflictHeatmap API

```typescript
interface ConflictHeatmapProps {
  repositoryId: RepositoryId;
  worktreeIds?: WorktreeId[];
  granularity?: 'file' | 'symbol';
  onCellClick?: (
    worktreeA: WorktreeId,
    worktreeB: WorktreeId,
    overlapTarget: string
  ) => void;
  showLegend?: boolean;
}
```

### 7.7 FeedbackItem API

```typescript
interface FeedbackItemProps {
  feedback: Feedback;
  variant?: 'inbox' | 'inline' | 'compact';
  showTarget?: boolean;
  showProvenance?: boolean;
  onResolve?: () => void;
  onEdit?: () => void;
  onSupersede?: () => void;
  onView?: () => void;
}
```

---

## 8. 性能预算

### 8.1 Core Web Vitals 目标

**所有目标标记 `TBD-MEASURE`**(继承《Requirements》§36):

| 指标 | 目标 | 测量方法 |
|---|---|---|
| **LCP**(Largest Contentful Paint) | TBD-MEASURE < 2.5s | RUM(Web Vitals) |
| **INP**(Interaction to Next Paint) | TBD-MEASURE < 200ms | RUM |
| **CLS**(Cumulative Layout Shift) | TBD-MEASURE < 0.1 | RUM |
| **TTFB**(Time to First Byte) | TBD-MEASURE < 600ms | RUM |
| **FCP**(First Contentful Paint) | TBD-MEASURE < 1.8s | RUM |
| **TBT**(Total Blocking Time) | TBD-MEASURE < 200ms | Lighthouse |

### 8.2 Bundle Size 目标

| 资源 | 目标 |
|---|---|
| Initial JS Bundle(gzip) | TBD-MEASURE < 250KB |
| Initial CSS(gzip) | TBD-MEASURE < 30KB |
| Per-Route JS Chunk(gzip) | TBD-MEASURE < 100KB |
| Image Max | TBD-MEASURE < 200KB |
| Total Page(gzip) | TBD-MEASURE < 500KB |

### 8.3 Code Splitting 策略

**路由级**:

```typescript
// router.tsx - 用 lazy import
const WorktreeDetailPage = lazy(() => import('@features/worktree/routes/WorktreeDetailPage'));
const AgentChatPage = lazy(() => import('@features/agent/routes/AgentChatPage'));
```

**组件级**(大组件 / 重组件):

```typescript
const MonacoEditor = lazy(() => import('@monaco-editor/react'));
const DiffViewer = lazy(() => import('@ui/DiffViewer'));
const GanttChart = lazy(() => import('@features/planning/components/GanttChart'));
```

**库级**(按需 import):

```typescript
// 不要 import 整个 lodash
import debounce from 'lodash/debounce';

// 不要 import 整个 @mui/material
import Button from '@mui/material/Button';
```

### 8.4 性能优化策略

| 策略 | 应用 |
|---|---|
| **虚拟列表** | Worktree Table / Card / Heatmap / Activity / Chat History |
| **Memoization** | `React.memo` 列表项 / `useMemo` 复杂计算 / `useCallback` 事件 |
| **Selector 优化** | Zustand 用 `shallow` 比较 |
| **Image Lazy** | `loading="lazy"` + Intersection Observer |
| **Prefetch** | hover 卡片时 prefetch 详情 |
| **Service Worker** | 静态资源缓存(PWA,V1) |
| **Code Splitting** | 路由 / 重组件 |
| **Tree Shaking** | 启用 sideEffects: false |
| **Web Worker** | 大数据计算(Diff 渲染 / Heatmap 计算) |

### 8.5 长任务处理

| 操作 | 策略 |
|---|---|
| 1000+ Worktree 列表 | 虚拟列表 + 分页 |
| 大 Diff 渲染(> 1MB) | Object Storage 拉取 + Monaco 分块 |
| Heatmap 计算(50+ Worktree) | Web Worker + 进度条 |
| Symbol Index 搜索 | IndexedDB 缓存 + 后端兜底 |
| Gantt 渲染 | 虚拟 + 时间窗口 |

---

## 9. 测试策略

### 9.1 单元测试(Vitest)

**覆盖目标**:`hooks` / `lib` / `utils` ≥ 80%

**例子**:

```typescript
// hooks/useWorktreeFilters.test.ts
describe('useWorktreeFilters', () => {
  it('should parse URL search params to filter object', () => {
    // ...
  });
  it('should update URL when filter changes', () => {
    // ...
  });
});
```

### 9.2 组件测试(React Testing Library)

**覆盖目标**:Shared Components ≥ 80%

**例子**:

```typescript
// components/WorktreeCard.test.tsx
describe('WorktreeCard', () => {
  it('renders all required info', () => {
    render(<WorktreeCard worktree={mock} workItem={mockItem} />);
    expect(screen.getByText('star/WT-001')).toBeInTheDocument();
    expect(screen.getByText('Login API')).toBeInTheDocument();
  });

  it('shows conflict indicator when conflict present', () => {
    // ...
  });

  it('calls onAction when action button clicked', () => {
    // ...
  });
});
```

### 9.3 E2E 测试(Playwright,继承《Test Design》§5)

**覆盖目标**:关键用户流程 100%

**6 个关键流程**(继承《External Design》§4):

1. **从 WorkItem 创建 Worktree**
2. **分配 Worktree 给 Agent**
3. **Agent 修改后 Review + 提交 Feedback**
4. **处理 Feedback Inbox(Resolve / Supersede)**
5. **处理 Conflict(Rebase / Merge)**
6. **Merge PR**

**E2E 例子**:

```typescript
// tests/e2e/create-worktree.spec.ts
test('User can create Worktree from WorkItem', async ({ page }) => {
  // 1. Login
  await page.goto('/login');
  // ...

  // 2. Navigate to WorkItem
  await page.goto('/workitems/WI-123');
  await page.click('button:has-text("Create Worktree")');

  // 3. Fill dialog
  await page.fill('input[name="branch"]', 'feature/WI-123');
  await page.selectOption('select[name="agent_type"]', 'codex');
  await page.click('button:has-text("Create")');

  // 4. Verify
  await expect(page).toHaveURL(/\/worktrees\/[a-f0-9-]+/);
  await expect(page.locator('text=AGENT_RUNNING')).toBeVisible();
});
```

### 9.4 Visual Regression(Storybook + Chromatic)

每个 Shared Component 写 Storybook story,Chromatic 自动截图比对。

### 9.5 A11y 测试(axe-playwright)

```typescript
test('Worktree Control Center is accessible', async ({ page }) => {
  await page.goto('/worktrees');
  const accessibilityScanResults = await new AxeBuilder({ page }).analyze();
  expect(accessibilityScanResults.violations).toEqual([]);
});
```

---

## 10. 给下游契约

### 10.1 给 Implementation 任务分解

**关键任务**(P0):

```text
1. apps/web 项目脚手架(Vite + TS + pnpm workspace)
2. UI Kit 基础组件(Button, Input, Select, Dialog, etc.)
3. Layout(AppShell, TopBar, SideNav, StatusBar)
4. Routing(React Router v7 + Guards)
5. Auth Feature(Login, TenantPicker, Bootstrap)
6. Worktree Feature
   - Control Center(Table + Card + Heatmap)
   - Detail(5 Tabs)
   - Realtime Status
7. Feedback Feature
   - Inbox
   - Form(5 段式)
   - Intervention Queue
8. WorkItem Feature
9. Board Feature(Kanban + 拖动)
10. Settings(Profile + Project Policy + Agent Policy)
11. Realtime(WebSocket Client + Subscription Bridge)
12. Error Handling(Global Boundary + Toast)
13. Testing Setup(Vitest + RTL + Playwright + MSW)
14. CI/CD(Lint + Test + Build + Deploy)
```

**V1 任务**:Heatmap / Sprint Planning / Gantt / Settings 完整 / Audit Tab

**V2 任务**:移动端 / 离线 / 实时协作光标

### 10.2 与 Backend / API 的契约

- 所有 API 端点走《API Design》v0.1 规范
- OpenAPI 3.1 文档由 Backend 维护
- 前端用 OpenAPI Generator 自动生成 TypeScript Client
- 任何 Breaking Change 必须走 RFC,Versioning 走 `/v1/` 路径

### 10.3 与 Design System 的契约

- Design Token 由 Design + Frontend 共同维护
- Storybook 是 Design System 单一来源
- 新组件必须先在 Storybook 实现 + 设计走查,再用于业务

---

## 11. Open Issues(继承上游 + 新增 Internal-J.x)

### 11.1 继承自《External Design》Open Issues

- External-J.5(键盘快捷键):Implementation 阶段实现,Internal Design 需列快捷键清单
- External-J.7(虚拟列表):§8.4 已确定策略
- External-J.8(批量操作):§4.2.1 selection-slice 已设计

### 11.2 本设计新增

- **Internal-J.1**:是否用 Monorepo(pnpm workspace)还是 Polyrepo?本设计默认 Monorepo,优势大。**已决定**。
- **Internal-J.2**:是否用 Vite 还是 Next.js?本设计用 Vite SPA(无 SSR 需求)。**已决定**。
- **Internal-J.3**:状态管理是否上 Redux Toolkit?本设计用 Zustand。**已决定**(除非遇到极端场景再调整)。
- **Internal-J.4**:WebSocket 客户端是否用 Socket.io?本设计用原生 WebSocket + 自封装(避免 Socket.io 后端依赖)。**已决定**。
- **Internal-J.5**:是否用 GraphQL 替代 REST?本设计 REST(与 Backend 现状一致)。**否**。
- **Internal-J.6**:是否上 PWA(Service Worker)?**V1 候选**。
- **Internal-J.7**:是否上 React Native(移动)?**V2 候选**。
- **Internal-J.8**:Error Tracking 用 Sentry 还是自建?Sentry 商业,自建 Open Source 选 OpenObserve / GlitchTip。**V1 候选**。
- **Internal-J.9**:Visual Regression 工具是 Chromatic(商业)还是 Loki(自建)?**V1 候选**。
- **Internal-J.10**:i18n 库是 i18next 还是 react-intl?本设计选 i18next(简单 + 灵活)。**已决定**。

---

## 12. 接口稳定承诺(给 Implementation)

以下接口在本设计冻结后,**不**因 Implementation 阶段而变更:

1. **技术栈选型**(§1):React 18 + Vite 5 + TS 5 + TanStack Query v5 + Zustand + React Router v7
2. **状态管理分工**(§4):Server = TanStack Query / Client = Zustand / Form = RHF
3. **目录结构**(§2):pnpm workspace + apps/packages
4. **Query Key 命名规范**(§4.1.2):`[scope, entity, action, params]`
5. **Hook 命名规范**(§6.1):`use{Entity}{Action}`
6. **路由结构**(§5.1):URL Path 设计
7. **Route Guard 模式**(§5.2):ProtectedRoute / RequirePermission
8. **错误处理分类**(§6.3):ApiError / AuthError / PermissionError / NotFoundError 等
9. **Optimistic Update 模式**(§4.5):标准模板
10. **共享组件 Prop Interface**(§7.2-§7.7):WorktreeCard / AgentStatusBadge / DiffViewer / TestResultList / ConflictHeatmap / FeedbackItem
11. **Cache staleTime 配置**(§6.5)
12. **Bundle 预算**(§8.2)
13. **测试目标**(§9):单元 ≥ 80% / 组件 ≥ 80% / E2E 关键流程 100%
14. **a11y 目标**(继承《External Design》§7.3):WCAG 2.1 AA
15. **i18n 语言清单**(继承《External Design》§8.1):en / zh-CN / ja

**变更流程**:任何对上述接口的修改,需走 RFC + 重新冻结本设计。

---

## 13. 文档元信息

- **章节数**:0~12 主章
- **mermaid 图数**:5(§3.1, §3.2, §3.3, §3.4, §3.5)
- **目标行数**:1000~2000
- **目标大小**:30~70KB
- **下游契约**:Implementation(React + Vite + TypeScript 应用)
- **关联设计**:《External Design》(直接上游) + 《API Design》(API 契约) + 《Basic Design》(信息架构)
- **覆盖 25 Module**:本设计主要涉及 domain-work-item(§3.5 + §2.2 features/workitem)、domain-worktree(§3.1 + §3.2 + §2.2 features/worktree)、domain-workflow(§3.5 Board 状态)、domain-board(§3.5 features/board)、domain-planning(§3.6 features/planning)、domain-relation(§2.2 + §3.5 WorkItem 关联)、domain-comment(§2.2 + §3.5 WorkItem Comments)、domain-feedback(§3.4 features/feedback + 5 段式表单)、domain-context(§3.3 features/agent + Chat)、domain-agent(§3.3 features/agent + Agent Status + Chat)、domain-scm(§3.2 PR 关联 + §2.2 worktree-api 调 SCM Adapter)、domain-development(§3.2 Diff 渲染 + §3.4 ChangeSet)、domain-validation(§3.2 Tests + §3.5 Validation)、domain-tenant(§5.1 全局路由 + §3.1 Login + TenantPicker)、domain-workspace(§5.1 全局 + Tenant Picker 选 Workspace)、domain-project(§3.7 Settings Project Policy)、domain-permission(§5.2 Route Guard + §3.7 Members)、domain-identity(§3.1 Login + §3.7 Local Runtime)、domain-notification(§2.2 features/notifications + 通知中心)、domain-audit(§3.7 Audit Tab + 9 问必答查询)、domain-automation(§3.7 Project Policy 自动化规则 + §3.7 Members)、domain-integration(§3.7 Integrations Tab + §2.2 worktree-api 调 SCM/Agent Adapter)、domain-collaboration(§3.3 Chat 多人协作 + Realtime + §4.3 WS 订阅)、domain-search(§2.2 features/search + Cmd+K)、domain-local-runtime(§3.7 Local Runtime Tab + §2.1 Status Bar Daemon 状态)。**全部 25 Module 至少出现 1 次**。
- **13 类 tenant_id 必带对象**:Worktree(§3.1 + §3.2 #3)、AgentSession(§3.1 + §3.2 + §3.3 #4)、ContextPacket(§3.3 Agent Chat 引用 #5)、Feedback(§3.4 + §3.6 #6)、AI Prompt(§3.3 Chat 走 #7)、AI Response(§3.3 #8)、Diff(§3.2 DiffViewer #9)、Build Log(§3.2 TestResultList #10)、Test Log(同上 #11)、PR Content(§3.2 PR Panel #12)、Symbol Index(§3.2 DiffViewer + §3.1 Heatmap #13)、Repository Credential(§3.7 Integrations Tab #1)、Local Runtime(§3.7 Local Runtime Tab + §2.1 Status Bar #2)。**全部 13 类必带对象至少出现 1 次**。

---

**END of Internal Design v0.1**
