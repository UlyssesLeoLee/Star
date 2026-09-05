# SRS-AGENT-VIEW-001

> **Agent View 要件定義書 v1.0** (per 日本 IPA SEC 標準 / 要件定義書 テンプレート)
>
> - 状态: Requirements Baseline
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: `9806d3d` (Agent view 実装) + `bfcde68` (実装報告)
> - 关联実装報告: `docs/reports/PHASE-AGENT-VIEW-IMPL-REPORT.md` v0.1
> - 关联基本設計書: `docs/design/BD-AGENT-VIEW-001.md` v0.1 (本 commit 同期落档)
> - 上位要件: `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (STAR Agent Runtime SRS, 9/3 落档)
> - 平行 view: `docs/architecture/2026-09-03-agent-runtime/` (Rust Runtime) + `docs/architecture/2026-09-03-langgraph/` (LangGraph)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-05 JST
> - 受众: 详细設計工程师 / 架构审查者 / UI/UX 设计师 / SRE

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-AGENT-VIEW-001 |
| 文书名 | Agent View 要件定義書 |
| 版本 | v1.0 |
| 作成日 | 2026-09-05 |
| 作成者 | Ulysses — Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) |
| 关联 commit | `9806d3d` (実装) + `bfcde68` (報告) |
| 关联文档 | `BD-AGENT-VIEW-001.md` (本 commit 同期) + `PHASE-AGENT-VIEW-IMPL-REPORT.md` v0.1 |
| 上位文档 | `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 (情報システム等の整備に係る標準的指針) 制定 STAR 平台 **Agent View** 视图的需求规格说明书, 涵盖功能/非功能/数据/接口/约束/场景/验收等维度, 作为后续基本设计 (`BD-AGENT-VIEW-001.md`) / 详细设计 / 实装 / 测试 / 验收的唯一依据。

### 1.2 背景

STAR 平台已落地 22 个 domain-* crate + 47 个 workspace package + LangGraph view + Agent Runtime view (per 2026-09-03 同期落档), 现有视图 (Kanban / Timeline / Backlog / Agents / Worktrees) 各自承担单一视角。**用户痛点** (per 2026-09-05 11:25 JST 用户发令): 单个 Agent Session 关联的 worktree + 多个 work-items + 实时状态, 在多视图间切换成本高, 缺乏一张图整体把握的入口。

### 1.3 包含范围

- `/agent-view` 页面 (路由 + UI + 交互)
- 无限画布 (Miro 风格, 自由散开布局)
- 当前工作 agent 自动筛选 (11 个 active 状态)
- 用户手动覆盖 (顶部 dropdown + URL `?agent=` 参数)
- 3 类节点渲染 (agent / worktree / work_item) + bezier connector
- 跟 Kanban / Worktree 共享 zustand store 实时同步
- V/H/+/-/1 键盘快捷键
- 跳详情联动 (双击 → /agent / /worktree / /work-item)

### 1.4 不包含范围

- 后端 Agent Session 创建 / 启停 / 状态机 (per [SRS-Runtime §0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md))
- 节点拖动编辑 (派生视图, 写不归本视图管, per §3 缺口 #2)
- canvas 持久化 (per §3 缺口 #3)
- agent session 1:N 关联 work-items (per §3 缺口 #4, 当前 schema 缺 `WorkItem.agent_session_id` 字段)
- minimap 点击跳转 (per §3 缺口 #6)
- LangGraph / Agent Runtime Rust 层 (引用平行 view, 不重写)

### 1.5 用户故事

| 编号 | 角色 | 故事 | 优先级 |
|---|---|---|---|
| US-1 | 项目经理 (Ulysses 类) | 作为 PM, 我希望打开应用就看到当前正在工作的 agent 中心视图, 一眼看到它执行的 worktree 和关联的 work-items, 不用切换 4 个 Tab | P0 |
| US-2 | 5 域 Lead (未来真人到位) | 作为 Lead, 我希望切换到我负责的 agent session, 看清其执行进度和瓶颈 work-items | P0 |
| US-3 | SRE | 作为 SRE, 我希望看到 agent 的 token / cost 用量, 跟 budget 对比, 触发预算告警 | P1 |
| US-4 | PM | 作为 PM, 我希望从 Agent View 双击 work-item 跳 Kanban 详情, 不丢上下文 | P1 |
| US-5 | Dev | 作为 Dev, 我希望快速对比多个 agent session 的拓扑, 决定接下来挂哪个 worktree | P2 |

---

## §2 用语定义 (用語集 / Ubiquitous Language)

| 用语 | 定义 | 出处 / 备注 |
|---|---|---|
| Agent Session | 1 个 AI Agent 执行的会话实例, 14 状态机 (queued/spawning/initializing/compiling_context/planning/executing/awaiting_feedback/awaiting_human/awaiting_tool/validating/paused/completed/failed/cancelled) | per [SRS-Runtime §8](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) |
| Active Agent | 处于上述 14 状态中前 11 个 (排除 completed/failed/cancelled 终态) 的 agent session | 本 SRS 新增 (per §4.1.1) |
| 当前工作 Agent | "Active 集合中 started_at 最新的 agent"; 如果没有 active, 取 started_at 最新的任意 agent | 本 SRS 新增 (per §4.1.2) |
| Worktree | Git worktree, 17 状态机 (per basic-design §4.1) | per [SRS-Runtime §8](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) |
| Work Item | 项目任务卡, 6 状态机 (todo/in_progress/review/blocked/done/wontfix) | per [SRS-Runtime §8](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) |
| 派生视图 (Projection) | 从已有数据派生计算的视图, 不是业务事实源, 不可写 | DDD 概念, per [SRS-Runtime §4.5](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) |
| 无限画布 (Infinite Canvas) | Miro 风格画布, 世界坐标 + viewport 转换, 鼠标 pan/zoom, 节点不强制栅格 | per [frontend-canvas-design.md](../../frontend-canvas-design.md) v0.1 |
| 自由散开布局 (Free-form Layout) | 节点按关联性散开, 不强制 swimlane / 网格; agent 居中, worktree 紧邻, work-items 圆周分布 | 本 SRS 新增 (per §4.1.3) |
| Bezier Connector | 三次贝塞尔曲线, 复用 frontend-canvas-design §3 公式 (c1x = fx + dx*0.25, c2x = tx - dx*0.25) | per [frontend-canvas-design.md](../../frontend-canvas-design.md) §3 |
| Fit-to-content | 根据 bbox 自动算 zoom + viewport, 使所有节点刚好填满容器 + 60px padding | 本 SRS 新增 (per §4.1.4) |
| URL 参数 (URL Param) | `?agent=ag-XXX` 用于深链 / 跨页面 share | per [frontend-design.md §6.1](../../frontend-design.md) URL State 规范 |
| Auto Badge | dropdown 触发器右上角 "auto" 角标, 标识当前是自动选而非用户手动覆盖 | 本 SRS 新增 |
| 派生时间戳 (derivedAt) | AgentCanvas 派生计算完成的时间 (ISO 8601), 触发 AgentCanvasView viewport 重置 | 本 SRS 新增 |

---

## §3 业务背景 / 前提条件

### 3.1 业务背景

- STAR 平台 1M logical agents / 1 SRE 单机 / token-OLU 计算 (per [SRS-Runtime §2](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md))
- 现有 12 个 agent session mock seed (per `frontend/src/lib/seed.ts`)
- 现有 12 个 worktree mock seed (per `frontend/src/lib/seed.ts`)
- 现有 30 个 work-item mock seed (per `frontend/src/lib/seed.ts`)
- 5 域 (player/economy/match/social/admin) Lead 真人未到位, Mavis 临时代签 (per [AGENTS.md §4 守门 #3 v2 派生规](../../AGENTS.md))

### 3.2 前提条件

- P-1: zustand store (`@/lib/store`) 已落地 agentSessions / worktrees / workItems 3 个集合
- P-2: StatusPill 组件已落地 60+ 状态色码 (per [frontend-canvas-design §2.3](../../frontend-canvas-design.md))
- P-3: 路由系统已就绪 (Next.js 14.2.5 App Router)
- P-4: i18n 系统已就绪 (3 语言 zh-CN / en / ja)
- P-5: SVG 基础工具已落地 (lucide-react / recharts 等)
- P-6: 不引入新依赖 (复用 lucide-react / StatusPill / useStore)

### 3.3 业务规则 (Business Rules)

- BR-1: 一个 Agent Session 同一时刻只能"当前工作"一次 (互斥)
- BR-2: 一个 Agent Session 通过 `worktree_id` 1:1 关联 1 个 Worktree (per [SRS-Runtime §6.2](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md))
- BR-3: 一个 Worktree 通过 `worktree_id` 1:N 关联多个 Work Items (per ids.ts WorkItem.worktree_id)
- BR-4: Work Item 状态机 (6 态) 决定 connector 颜色 (in_progress=blue / review=amber / blocked=red / todo=ink-dim / done=green / wontfix=ink-mute)
- BR-5: 用户手动覆盖后, dropdown 切换立刻写 URL `?agent=ag-XXX` (no debounce, 立即反映)

---

## §4 业务需求

### 4.1 功能需求 (Functional Requirements)

#### 4.1.1 FR-1 Active Agent 识别

| 项 | 内容 |
|---|---|
| ID | FR-AGV-001 |
| 描述 | 系统应能识别 "active" 状态的 agent session, 用于自动筛选 |
| 输入 | `AgentSession[]` |
| 输出 | `boolean` (isActiveAgent) |
| 业务规则 | 11 个 active 状态: queued / spawning / initializing / compiling_context / planning / executing / awaiting_feedback / awaiting_human / awaiting_tool / validating / paused |
| 反例 | 3 个终态: completed / failed / cancelled (不算 active) |
| 优先级 | P0 |

#### 4.1.2 FR-2 当前工作 Agent 自动选

| 项 | 内容 |
|---|---|
| ID | FR-AGV-002 |
| 描述 | 系统应在没有用户手动覆盖时, 自动选 "当前工作 agent" (active 中 started_at 最新, fallback 全部 started_at 最新) |
| 输入 | `AgentSession[]` |
| 输出 | `AgentSession` (或 null) |
| 业务规则 | BR-1 (互斥), 排序: started_at DESC, tie-breaker id ASC (稳定) |
| 优先级 | P0 |

#### 4.1.3 FR-3 自由散开布局

| 项 | 内容 |
|---|---|
| ID | FR-AGV-003 |
| 描述 | 系统应在无限画布上以"自由散开"方式布局节点 |
| 输入 | agent + worktree + workItems |
| 输出 | `AgentCanvas { nodes, connectors, viewport }` |
| 布局规则 | agent 中心 (0,0) / worktree 右侧 80px gap 居中对齐 / work-items 圆周 (内圈 8 + 外圈 12) |
| 排序规则 | [status_order ASC, due_date ASC, id ASC] (稳定) |
| 确定性 | 同样输入永远出同样输出 (避免 SSR/CSR hydration 漂移) |
| 优先级 | P0 |

#### 4.1.4 FR-4 Fit-to-content Viewport

| 项 | 内容 |
|---|---|
| ID | FR-AGV-004 |
| 描述 | 系统应在画布首次加载时, 自动算 fit-to-content viewport |
| 输入 | bbox (minX/minY/maxX/maxY) + 容器尺寸 (1200x800) + padding (60px) |
| 输出 | `{ x, y, zoom }` |
| 算法 | zoom = min(usableW/bw, usableH/bh, 1.5), 中心对齐; zoom clamp [0.2, 1.5] |
| 优先级 | P0 |

#### 4.1.5 FR-5 节点渲染

| 项 | 内容 |
|---|---|
| ID | FR-AGV-005 |
| 描述 | 系统应渲染 3 类节点 (agent / worktree / work_item), 视觉差异 |
| 视觉规范 | agent: 220x110 圆角矩形, 蓝底 + status pill + token/cost; worktree: 240x80 圆角矩形, 深底 + branch + status pill; work_item: 180x64 卡片, 浅底 + key + title + status pill + priority |
| 优先级 | P0 |

#### 4.1.6 FR-6 Connector 渲染

| 项 | 内容 |
|---|---|
| ID | FR-AGV-006 |
| 描述 | 系统应渲染 agent→worktree + worktree→work_item 两条 bezier connector |
| 视觉规范 | 三次贝塞尔 (c1x = fx + dx*0.25, c2x = tx - dx*0.25), 颜色按 status (per BR-4), 中点 label |
| 优先级 | P0 |

#### 4.1.7 FR-7 Pan / Zoom 交互

| 项 | 内容 |
|---|---|
| ID | FR-AGV-007 |
| 描述 | 系统应支持画布 pan / zoom 交互 |
| 交互规范 | 中键 / pan tool / shift = pan; 滚轮 = zoom (以光标为中心); 工具栏 5 按钮 (select/pan/zoom-in/zoom-out/fit) |
| zoom 范围 | [0.1, 4.0] |
| 键盘快捷键 | V=select, H=pan, +/-=zoom, 1=fit |
| 优先级 | P0 |

#### 4.1.8 FR-8 节点 Hover / Select

| 项 | 内容 |
|---|---|
| ID | FR-AGV-008 |
| 描述 | 系统应支持节点 hover (高亮边框) / select (蓝色边框) 状态 |
| 视觉规范 | 默认 #30363d, hover #2f81f7, select #79c0ff |
| 优先级 | P1 |

#### 4.1.9 FR-9 双击跳详情

| 项 | 内容 |
|---|---|
| ID | FR-AGV-009 |
| 描述 | 系统应支持双击节点跳到对应详情页 |
| 路由 | agent → /agent?selected={id} / worktree → /worktree?selected={id} / work_item → /work-item?selected={id} |
| 优先级 | P1 |

#### 4.1.10 FR-10 顶部 Agent 筛选

| 项 | 内容 |
|---|---|
| ID | FR-AGV-010 |
| 描述 | 系统应在顶部提供 agent 筛选 dropdown |
| 行为 | 触发器显示当前 agent (id + kind) + auto 角标; dropdown 列 [active 优先, started_at desc, id asc] 的 agents; 点选 → onChange 写 URL |
| a11y | aria-haspopup / role=listbox / role=option / aria-selected |
| 优先级 | P0 |

#### 4.1.11 FR-11 URL 参数 Override

| 项 | 内容 |
|---|---|
| ID | FR-AGV-011 |
| 描述 | 系统应支持通过 URL `?agent=ag-XXX` 覆盖默认选择 |
| 行为 | URL 给了且找到了 → 选它, auto=false; URL 给了但找不到 → fallback 默认, auto=true; URL 没给 → auto 默认 |
| 优先级 | P0 |

#### 4.1.12 FR-12 Minimap

| 项 | 内容 |
|---|---|
| ID | FR-AGV-012 |
| 描述 | 系统应在右下角渲染 minimap (viewport 范围 + 节点位置) |
| 视觉规范 | 160x112 px 容器, viewport 矩形 + 节点绿点 |
| 优先级 | P2 |

#### 4.1.13 FR-13 跳 Kanban 联动

| 项 | 内容 |
|---|---|
| ID | FR-AGV-013 |
| 描述 | 系统应在 header 提供跳 Kanban Board 的按钮, 携带 worktree_id filter |
| 路由 | /board?assignee_id=&worktree_id={wt-XXX} |
| 优先级 | P2 |

#### 4.1.14 FR-14 空状态

| 项 | 内容 |
|---|---|
| ID | FR-AGV-014 |
| 描述 | 系统应在 store 无 agent / 无 resolvable agent 时显示空状态 |
| 视觉规范 | warn icon + 提示文案 + 跳 /agents 链接 |
| 优先级 | P1 |

### 4.2 非功能需求 (Non-Functional Requirements)

#### 4.2.1 NFR-1 性能

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-PERF-001 |
| 指标 | 画布首次渲染 ≤ 500ms (mock 12 agent + 12 worktree + 30 wi 场景) |
| 测量 | FCP / LCP, 浏览器 dev tools |
| 优先级 | P1 |

#### 4.2.2 NFR-2 视觉一致性

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-UI-001 |
| 指标 | 跟 StatusPill 配色一致 / 跟现有 canvas-design v0.1 一致 / dark mode 优先 |
| 测量 | 人工 review, 无 P0 视觉缺陷 |
| 优先级 | P0 |

#### 4.2.3 NFR-3 键盘可达

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-A11Y-001 |
| 指标 | 顶部 dropdown 满足 WAI-ARIA listbox 模式; 工具栏按钮含 title; 快捷键不冲突 (跳过 INPUT/TEXTAREA/SELECT) |
| 测量 | axe-core / WAVE / 人工 |
| 优先级 | P1 |

#### 4.2.4 NFR-4 派生确定性

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-DET-001 |
| 指标 | 同样输入永远出同样输出 (SSR/CSR hydration 无漂移) |
| 测量 | vitest (per `layout.test.ts` "排序稳定" / "deterministic" 2 个测试) |
| 优先级 | P0 |

#### 4.2.5 NFR-5 派生纯函数

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-DET-002 |
| 指标 | `layoutAgentCanvas` / `fitToContentViewport` / `resolveCurrentAgent` / `pickAgentWorktree` / `pickAgentWorkItems` / `isActiveAgent` / `pickDefaultAgent` 全部是纯函数, 无副作用, 无 IO, 无 Date.now() |
| 测量 | vitest (per §6.1 单元测试覆盖) |
| 优先级 | P0 |

#### 4.2.6 NFR-6 不污染 Store

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-STATE-001 |
| 指标 | AgentCanvas 不进 zustand store (避免污染 canvasElements 持久化) |
| 派生 | 用 local useState + derivedAt 时间戳触发 AgentCanvasView viewport 重置 |
| 优先级 | P0 |

#### 4.2.7 NFR-7 派生只读

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-STATE-002 |
| 指标 | 节点只读, 不能拖动 (派生视图, 拖动会跟 store 同步冲突) |
| 用户编辑 | 走通用 canvas (`/canvas/[id]`) |
| 优先级 | P0 |

#### 4.2.8 NFR-8 国际化

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-I18N-001 |
| 指标 | 3 语言 (zh-CN / en / ja) 友好, 至少 4 项 i18n key 落 `dictionary.ts` |
| 派生 | StatusPill 走 `useStatusLabel` 翻译, agent / worktree status 当前缺 StatusKind, 走 prettify fallback |
| 优先级 | P2 |

#### 4.2.9 NFR-9 测试覆盖

| 项 | 内容 |
|---|---|
| ID | NFR-AGV-TEST-001 |
| 指标 | 29/29 vitest pass (3 测试文件, layout 11 + selectors 14 + AgentCanvasView 4) |
| 派生 | 29/29 + 0 typecheck err (我新增的 10 个文件) |
| 优先级 | P0 |

---

## §5 约束条件 (Constraints)

### 5.1 技术约束

- TC-1: 不引入新 npm 依赖 (per [AGENTS.md §4 守门 #19 v19+ 累积规](../../AGENTS.md) "不偷偷 commit 子代理产出", 同理 0 新依赖)
- TC-2: 复用 `@/lib/store` (zustand) / `@/components/StatusPill` / `@/components/PageHeader` / lucide-react
- TC-3: 遵循 Next.js 14.2.5 App Router 规范 ("use client" + useSearchParams + useRouter)
- TC-4: 遵循 TypeScript strict mode, 0 `any` (除 fallback / 类型断言)
- TC-5: 遵循 ESLint + Prettier (per `frontend/.eslintrc.json`)

### 5.2 业务约束

- BC-1: 界面名 = "Agent" (per 用户发令, 跟其他视图命名一致: Kanban / Timeline / Backlog / Agents / Worktrees)
- BC-2: 路由 = `/agent-view` (per 用户拍板 #3, 跟 /agent (Agent Sessions) / /agents (Agents 列表) 不冲突)
- BC-3: 跟 Kanban / Worktree 共享 zustand store 实时同步 (per 用户拍板 "数据对应 kanban 等界面的情况")

### 5.3 安全 / 合规约束

- SC-1: 不打印环境变量 (per [AGENTS.md §4 守门 #5](../../AGENTS.md))
- SC-2: 不输出 secret / token (per [AGENTS.md §4 守门 #5](../../AGENTS.md))
- SC-3: 13 租户隔离 (per [SRS-Runtime §43-§47](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) 租户隔离约束), Agent View 通过 store tenant_id 隐式隔离 (本 view 不显式处理, 委托 store)

### 5.4 组织约束

- OC-1: 5 域真人 Lead 到位前 Mavis 临时代签 (per [AGENTS.md §4 守门 #3 v2 派生规](../../AGENTS.md) 8/21 拍板 + 9/3 11:35 JST 反转)
- OC-2: commit author = Ulysses (per [AGENTS.md §4 守门 #10](../../AGENTS.md) + 8/27 19:39 JST 用户授权)
- OC-3: 报告 / 文档 7 段结构 (per [AGENTS.md §3](../../AGENTS.md))

---

## §6 业务场景 (Use Cases / 业务シナリオ)

### 6.1 主要场景

#### UC-1: PM 打开应用直接看当前工作 agent 拓扑

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 浏览器打开 `https://star.example.com/agent-view` | page 读取 store + URL `?agent=` (无) |
| 2 | 系统 | 调 `resolveCurrentAgent(agents, null)` | 返回 `{ agentId: 'ag-005', agent: <executing>, auto: true }` (per FR-2) |
| 3 | 系统 | 调 `pickAgentWorktree` + `pickAgentWorkItems` | 返回 1 wt + 5 wi |
| 4 | 系统 | 调 `layoutAgentCanvas` | 返回 7 nodes (1 agent + 1 wt + 5 wi) + 6 connectors + viewport |
| 5 | 系统 | 调 `fitToContentViewport` | zoom 0.8, 中心对齐 |
| 6 | 系统 | 渲染 AgentCanvasView | 700ms 内 FCP 完成 |
| 7 | PM | 看到 agent 中心, worktree 右侧, 5 wi 圆周散开 | (pass) |

**Acceptance**: 完整链路 ≤ 500ms (NFR-1), 视觉一致 (NFR-2)

#### UC-2: PM 切换到等待决策的 agent

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 点顶部 dropdown 触发器 | dropdown 打开, 列 12 agents (active 优先) |
| 2 | PM | 选 `ag-003 (awaiting_human)` | onChange 触发 |
| 3 | 系统 | 写 URL `?agent=ag-003` | router.replace (per FR-11) |
| 4 | 系统 | 重派生 canvas | ag-003 中心, 关联 wt-003 + 关联 wi |
| 5 | 系统 | 顶部 "auto" 角标消失 | auto=false (用户手动覆盖) |

**Acceptance**: URL 立即更新, auto 角标消失, 画布内容更新 ≤ 200ms

#### UC-3: SRE 双击 worktree 跳 Worktree Manager

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | SRE | 鼠标移到 wt-003 节点 | 边框高亮 (#2f81f7) (per FR-8) |
| 2 | SRE | 双击 wt-003 节点 | onNodeDoubleClick 触发 (per FR-9) |
| 3 | 系统 | `window.location.href = '/worktree?selected=wt-003'` | 路由跳 Worktree Manager |

**Acceptance**: 路由跳成功, Worktree Manager 自动选中 wt-003

#### UC-4: PM 跳 Kanban 看关联 work-items

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 点 header 右上角 "Kanban" 按钮 | `<a href="/board?assignee_id=&worktree_id=wt-005">` |
| 2 | 系统 | 跳 Kanban Board | Board 携带 worktree_id=wt-005 filter, 显示该 wt 关联的 wi |

**Acceptance**: Kanban filter 自动应用, wi 列表匹配 (per BR-3)

### 6.2 异常场景

#### UC-5: store 无 agent session

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | SRE | 浏览器打开 `/agent-view` | page 读取 store, agents.length === 0 |
| 2 | 系统 | 显示空状态 | warn icon + "No agent sessions" + 跳 /agents 链接 (per FR-14) |

**Acceptance**: 不显示空白画布, 给清晰引导

#### UC-6: URL `?agent=ag-XXX` 找不到

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 浏览器打开 `/agent-view?agent=ag-XXX` (URL 残留旧 id) | resolveCurrentAgent 找不到 |
| 2 | 系统 | fallback 默认 (auto=true) | 显示当前 active agent, 顶部 auto 角标在 |

**Acceptance**: 不显示空白, auto 角标提示这是 fallback

---

## §7 数据需求 (Data Requirements)

### 7.1 输入数据 (Store Schema)

复用现有 zustand store (per `frontend/src/lib/store.ts`):

| 集合 | 字段 (相关) | 类型 | 用途 |
|---|---|---|---|
| `agentSessions` | id / worktree_id / agent_kind / status / current_step / token_usage / cost_summary / started_at / ended_at | `AgentSession[]` | 中心节点 + 筛选源 |
| `worktrees` | id / branch / status / agent_session_id | `Worktree[]` | 1:1 关联 + 节点渲染 |
| `workItems` | id / key / title / status / priority / worktree_id / due_date | `WorkItem[]` | 圆周散点 + status 着色 |

### 7.2 派生数据 (Projection Schema, 不进 store)

| 字段 | 类型 | 来源 | 用途 |
|---|---|---|---|
| `AgentCanvas.agentId` | `string` | resolveCurrentAgent | 标识当前 |
| `AgentCanvas.nodes` | `AgentCanvasNode[]` | layoutAgentCanvas | 节点坐标 + 类型 + ref |
| `AgentCanvas.connectors` | `AgentCanvasConnector[]` | layoutAgentCanvas | 边 |
| `AgentCanvas.viewport` | `{ x, y, zoom }` | fitToContentViewport | 初始 viewport |
| `AgentCanvas.derivedAt` | `Iso8601` | `new Date().toISOString()` | 触发 AgentCanvasView viewport 重置 |

**重要**: 派生数据**不**持久化 (per NFR-6), F5 刷新会重派生 (~50ms)

### 7.3 数据完整性约束

- IC-1: `worktree_id` 在 store.worktrees 必须存在 (否则 pickAgentWorktree 返回 null, layout 跳过)
- IC-2: `work_item.worktree_id` 必须等于 `agent.worktree_id` 才能被 pickAgentWorkItems 选中 (per BR-3)
- IC-3: 派生数据不写 store (per NFR-6), 违反 = NFR 违反

### 7.4 数据流 (Data Flow)

```
[store.agentSessions] 
  ↓ resolveCurrentAgent(agents, urlAgentId)
  ↓ (active 优先 → started_at desc → id asc → tie-breaker via internal compareByStartedDescThenIdAsc)
  ↓ 返回 { agentId, agent, auto }
[store.worktrees] 
  ↓ pickAgentWorktree(worktrees, agent)
  ↓ (worktree_id 1:1 关联)
  ↓ 返回 Worktree | null
[store.workItems]
  ↓ pickAgentWorkItems(workItems, agent, worktree)
  ↓ (worktree_id 过滤)
  ↓ 返回 WorkItem[]
[LayoutInput] { agent, worktree, workItems }
  ↓ layoutAgentCanvas
  ↓ (agent (0,0) + worktree 右侧 80px + wi 圆周内 8 外 12)
  ↓ 排序 [status_order, due_date, id] 稳定 (via internal compareWorkItems helper)
  ↓ 返回 { nodes, connectors, bbox }
[bbox]
  ↓ fitToContentViewport(bbox, 1200, 800, 60)
  ↓ (zoom clamp [0.2, 1.5] + 中心对齐)
  ↓ 返回 { x, y, zoom }
[AgentCanvas] { agentId, nodes, connectors, viewport, derivedAt }
  ↓ render
[AgentCanvasView SVG]
```

---

## §8 接口需求 (Interface Requirements)

### 8.1 内部接口 (组件 Props)

```typescript
// 8.1.1 AgentCanvasView props
interface AgentCanvasViewProps {
  canvas: AgentCanvas;          // 派生画布
  agent: AgentSession;          // 中心节点
  worktree: Worktree | null;    // 紧邻节点 (可为 null)
}

// 8.1.2 AgentFilter props
interface AgentFilterProps {
  agents: ReadonlyArray<AgentSession>;  // 候选列表
  selectedId: string;                   // 当前选中
  auto: boolean;                        // 是否 auto 选
  onChange: (agentId: string) => void;  // 切换回调
}

// 8.1.3 layoutAgentCanvas 输入
interface LayoutInput {
  agent: AgentSession;
  worktree: Worktree | null;
  workItems: WorkItem[];
}
```

### 8.2 外部接口 (Route)

| 路径 | 入参 | 出参 | 备注 |
|---|---|---|---|
| `/agent-view` | - | page 渲染 | 主入口 |
| `/agent-view?agent=ag-XXX` | `?agent=ag-XXX` | 选 ag-XXX (auto=false) | 深链 (per FR-11) |
| `/agent?selected=ag-XXX` | `?selected=ag-XXX` | Agent Sessions 详情 (per §1.3 US-4) | 双击 agent 节点 |
| `/worktree?selected=wt-XXX` | `?selected=wt-XXX` | Worktree Manager 详情 | 双击 worktree 节点 |
| `/work-item?selected=wi-XXX` | `?selected=wi-XXX` | Work Item 详情 | 双击 work-item 节点 |
| `/board?worktree_id=wt-XXX` | `?worktree_id=wt-XXX` | Kanban Board (filtered) | header 跳 Kanban (per FR-13) |

### 8.3 Zustand Store 依赖

| Action / Getter | 用途 |
|---|---|
| `useStore((s) => s.agentSessions)` | 选 agent 源 |
| `useStore((s) => s.worktrees)` | 1:1 关联 |
| `useStore((s) => s.workItems)` | 圆周散点 + 实时同步 (per §1.3) |

**重要**: Agent View **不**调任何 action (transitionWorkItem / transitionAgent / addWorkItem), 仅订阅读取 (派生只读, per NFR-7)

---

## §9 验收标准 (受入基準 / Acceptance Criteria)

### 9.1 功能验收 (Functional AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-F-1 | URL 无参, store 有 active agent → 自动选 active 中 started_at 最新 | 手动 / vitest FR-2 测试 |
| AC-F-2 | URL `?agent=ag-XXX` 存在 → 选该 agent, auto=false | 手动 / vitest FR-11 测试 |
| AC-F-3 | URL `?agent=ag-XXX` 不存在 → fallback 默认, auto=true | 手动 / vitest FR-11 测试 |
| AC-F-4 | 顶部 dropdown 点选 → URL 立即更新, auto 角标消失 | 手动 |
| AC-F-5 | 画布首次加载 fit-to-content, zoom clamp [0.2, 1.5] | 手动 / vitest FR-4 测试 |
| AC-F-6 | 鼠标拖空白 (pan) → viewport 平移 | 手动 |
| AC-F-7 | 鼠标滚轮 (zoom) → 以光标为中心缩放 | 手动 |
| AC-F-8 | 工具栏 zoom in/out/fit 5 按钮正常 | 手动 |
| AC-F-9 | V/H/+/-/1 快捷键正常 (不冲突 INPUT/TEXTAREA/SELECT) | 手动 |
| AC-F-10 | 双击 agent 节点 → /agent?selected=ag-XXX | 手动 |
| AC-F-11 | 双击 worktree 节点 → /worktree?selected=wt-XXX | 手动 |
| AC-F-12 | 双击 work_item 节点 → /work-item?selected=wi-XXX | 手动 |
| AC-F-13 | 节点 hover → 边框高亮 | 手动 |
| AC-F-14 | 节点 select → 蓝色边框 | 手动 |
| AC-F-15 | minimap 显示 viewport 矩形 + 节点 | 手动 |
| AC-F-16 | header 跳 Kanban 按钮携带 worktree_id | 手动 |
| AC-F-17 | store 无 agent → 显示空状态 + 跳 /agents 链接 | 手动 |
| AC-F-18 | 跟 Kanban 同步: 在 Kanban 改 worktree 关联, Agent View 立即反映 | 手动 |

### 9.2 质量验收 (Quality AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-Q-1 | vitest 29/29 pass (3 测试文件) | `pnpm test --run src/lib/agent-view src/components/agent-view` |
| AC-Q-2 | typecheck 0 err (我新增的 10 个文件) | `tsc --noEmit` |
| AC-Q-3 | 不引入新依赖 | `package.json` diff (空) |
| AC-Q-4 | commit author = Ulysses | `git log --format='%an <%ae>' HEAD` |
| AC-Q-5 | 7 段报告落档 (per [AGENTS.md §3](../../AGENTS.md)) | `docs/reports/PHASE-AGENT-VIEW-IMPL-REPORT.md` 存在 |
| AC-Q-6 | 派生确定性 (NFR-4) | vitest "排序稳定" / "deterministic" 2 个测试 pass |
| AC-Q-7 | 派生纯函数 (NFR-5) | 7 个函数 vitest pass |

### 9.3 文档验收 (Documentation AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-D-1 | 本 SRS (要件定義書) 落档 | `docs/requirements/SRS-AGENT-VIEW-001.md` 存在 |
| AC-D-2 | BD (基本設計書) 落档 | `docs/design/BD-AGENT-VIEW-001.md` 存在 |
| AC-D-3 | 実装報告 (7 段) 落档 | `docs/reports/PHASE-AGENT-VIEW-IMPL-REPORT.md` 存在 |
| AC-D-4 | self-review (本 SRS + BD) 落地报告 | `docs/reports/PHASE-AGENT-VIEW-SELF-REVIEW.md` 存在 |

---

## §10 已知风险 / 未解決問題 (Known Issues / 缺口)

| # | 风险 / 缺口 | 影响 | 缓解 / 后续 |
|---|---|---|---|
| 1 | mock 数据 (跟全局一致); 真实后端 D.6+ 接入时改 store 即可, 组件不动 | 节点 / connector / status 都是 seed.ts 数据 | D.6+ 接入真实 data plane; 现状不阻塞 UI 演示 |
| 2 | 节点只读, 不能拖动 (派生视图; 拖动会跟 store 冲突) | 跟通用 CanvasView 区分; 用户编辑去 `/canvas/[id]` | Phase 2+ 看 DDD Review 拍板 |
| 3 | 不存到 store.canvasElements (避免污染; 用 derivedAt 时间戳触发重渲染) | F5 刷新会重派生 (~50ms) | 可接受; 用户没要求 persist 派生视图 |
| 4 | 节点只显示 worktree_id 关联的 work-items, 不显示 assignee_id 关联 (per ids.ts schema 缺 `WorkItem.agent_session_id` 字段) | 当前 agent 跟 wi 是 worktree 中介关联; 未来 DDD 加 `WorkItem.agent_session_id` 字段后可精确关联 | DDD Review 拍板; 当前 schema gap |
| 5 | agent / worktree status 走 StatusPill 默认 prettify, 没有 i18n 字典 (StatusKind 只有 workItem / sprint / workItemKind / refactor 4 类) | 英文/日文显示会保留 snake_case (e.g. "awaiting_human" 而不是 "Awaiting Human") | dictionary.ts v0.6+ 加 agent / worktree 状态表 |
| 6 | minimap 不支持点击跳转 (只是 viewport 可视化) | 用户 fit-to-content 用工具栏按钮代替 | P2 优化 |
| 7 | 当前 store 是 in-memory + zustand persist (localStorage); 多用户多 session 共享状态不可见 | 实际跨 session 协同走后端 (D.6+) | 当前 SPA 模式可接受 |
| 8 | 5 域真人 Lead 到位前 Mavis 临时代签 | 真人到位后追溯签字覆盖 | per 9/3 19:35 JST 拍板 D 维持 |

**DDD Review 必查**: 缺口 #4 (schema gap) + #5 (i18n) + #2 (派生只读)

---

## §11 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-05 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per §12 + 9/3 19:35 JST 拍板 D 维持)

---

## §12 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v1.0 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 12 段 (文档信息/目的/用语/前提/业务需求/约束/场景/数据/接口/验收/风险/签字) | 2026-09-05 11:25 JST 用户发令 "需要一个以当前工作 agent 为筛选模式的 view 界面, 形式是无限画布, 这个 agent 会有和它关联的任务, 数据对应 kanban 等界面的情况, 界面名字就是 Agent" + ask_user 拍板 #1/#2/#3 |
| v1.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | self-review fixes: (1) AC-D-3 (DD 詳細設計書) 删除 (用户只要求 2 份, 不在范围), AC-D-4 → AC-D-3, (2) §7.4 数据流图加 internal helper 提示 (compareByStartedDescThenIdAsc / compareWorkItems) | 2026-09-05 self-review [PHASE-AGENT-VIEW-SELF-REVIEW.md](../reports/PHASE-AGENT-VIEW-SELF-REVIEW.md) v0.1 Finding #4 + #5 |
