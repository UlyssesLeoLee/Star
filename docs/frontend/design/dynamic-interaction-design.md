# Star Frontend — 动态交互与多人协同设计书

> **状态**: Draft v0.1
> **日期**: 2026-08-28
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **触发**: Ulysses 2026-08-28 18:14 JST 反馈"看板的动态交互和多人共同操作功能要参考 E:/AssetsLake 的做法 + 甘特图/日历/工作流等动态交互"

---

## §1 目的与范围

参考 `E:/AssetsLake` 项目 (`frontend/src/plugin-groups/production/`) 的 4 个核心模块实现,补全 Star frontend (D:\Star\frontend) 缺失的:

1. **Kanban 动态拖动** (HTML5 DragEvent + 列重排)
2. **多人协同编辑** (board sync polling + presence cursor + operation notice)
3. **Gantt 时间轴** (work-item 跨项目时间线 + 拖动改 due date)
4. **Calendar 月/周视图** (sprint / milestone 日历)
5. **Workflow 节点编辑器** (state machine 可视化 + 节点拖动连接)
6. **跨模块联动** (board 拖动 → work-item 状态机 → audit log → notification)

适用: D:\Star\frontend 25 domain page, 重点 board / work-item / canvas / planning / workflow / integration 6 个核心可视化页.

---

## §2 设计原则 (per AssetsLake 经验 + Ulysses 偏好)

### §2.1 三层动态交互
- **L1 视觉层**: Tailwind CSS transitions + Framer Motion / CSS keyframes (200-300ms)
- **L2 行为层**: React state + zustand + optimistic update
- **L3 数据层**: TanStack Query + zustand persist + cache invalidation

### §2.2 多人协同不引入 WebSocket
- **原则**: 复用现有 star-mcp stdio + Streamable HTTP 通道, **不**单独部署 Socket.io/WebSocket
- **实现**: TanStack Query `refetchInterval: 2_000` 轮询 `boardSync` API + `since` cursor (per AssetsLake `useIssueBoardSync`)
- **Fallback**: SSE 推送未来 Phase I+ (per spec/services/02)

### §2.3 状态机 + 拖动一致性
- **每拖动 1 次**: 调 `transitionWorkItem(id, toStatus)` → 触发 INV-PM-01~05 → 写 audit log → 通知订阅者
- **拖动 = 一次 transition**: 不允许"拖到中间态再拖出"
- **拖动取消**: 释放到非列区域, 静默 revert (不写 audit)

### §2.4 5 域 + 性能
- **token-OLU 优先**: 拖动交互 AI 协同 ≤ 0.5M tokens / module
- **不引入新重依赖**: 不引 dnd-kit / react-dnd (AssetsLake 用 HTML5 native, 我们对齐)
- **Framer Motion 可选**: 已有 React 18 + Tailwind, 软动效用 CSS + Tailwind transition 已够

---

## §3 Kanban 看板动态拖动 (board/page.tsx)

### §3.1 现状
- 现有 `frontend/src/app/board/page.tsx` (from `frontend/src/app/board/`) 用 `useStore()` 读 mock `board` 数据
- 缺拖动, 缺多人协同, 缺列宽拖动, 缺卡片拖动排序

### §3.2 需求 (per AssetsLake `KanbanBoardPage.tsx`)
| 验收 | 引用 |
|---|---|
| 卡片 `draggable=true` + `onDragStart` 写入 `text/issue-id` | AssetsLake line 402 |
| 列 `onDragOver` (preventDefault) + `onDrop` 触发 transition | AssetsLake line 347/354 |
| 拖动到非列区域 = revert (无副作用) | self define |
| 跨列拖动 → `transitionWorkItem(id, toStatus)` 走状态机 INV-PM-01~05 | B.2.5 worktree transition pattern |
| 拖动期间显示 drop zone 高亮 (蓝边) + ghost 卡片 | self define |
| `useBoardSync(since)` TanStack Query 2s 轮询, 检测他人改动 → toast 提示 | AssetsLake line 102-110 |

### §3.3 数据流
```
[用户拖动] 
  → onDragStart: dataTransfer.text = issue_id
  → onDragOver: preventDefault + dropEffect='move'
  → onDrop: 调 transitionWorkItem(id, to_status)
  → 状态机校验 (INV-PM-01~05)
  → zustand store.update(id, to_status) 乐观更新
  → POST /api/work-items/{id}/transition (mock 暂记 audit log)
  → 其他人 2s 内通过 useBoardSync 拉新数据
```

### §3.4 实现
- 新建 `frontend/src/components/board/KanbanBoard.tsx` (列容器 + 拖动逻辑)
- 新建 `frontend/src/components/board/KanbanCard.tsx` (卡片 + 拖动起点)
- `frontend/src/app/board/page.tsx` 用新组件
- 复用 `useStore()` (zustand) + 加 `transitionWorkItem` (B.2.5 已有)
- 复用 `StatusPill` + `StateMachineDiagram` (frontend/src/components/)

---

## §4 Gantt 时间轴 (planning/page.tsx + 跨 project)

### §4.1 现状
- `frontend/src/app/planning/page.tsx` 现有 mock (sprint + milestone)
- 无时间轴拖动改 due date

### §4.2 需求 (per AssetsLake `GanttPage.tsx`)
| 验收 | 引用 |
|---|---|
| 时间轴 X 轴 = 日期, Y 轴 = sprint/milestone | AssetsLake line 6-7 |
| 拖动 milestone 条左右 = 改 due_date | self define |
| 拖动 sprint 条左右 = 改 start/end_date | self define |
| 跨 sprint 拖 work-item = 改 sprint_id | self define |
| 缩放 (week / month / quarter) | self define |
| 关键路径标识 (critical path 红色) | self define |

### §4.3 实现
- 新建 `frontend/src/components/gantt/GanttChart.tsx` (SVG / CSS grid 渲染)
- 新建 `frontend/src/components/gantt/GanttBar.tsx` (单条拖动)
- 用 HTML5 native drag (与 Kanban 一致)
- `transitionMilestone(id, newDueDate)` store action
- `transitionSprint(id, newStart, newEnd)` store action

---

## §5 Calendar 月/周视图 (planning/page.tsx 子视图)

### §5.1 现状
- 现有 mock planning page
- 无月/周切换

### §5.2 需求
| 验收 | 引用 |
|---|---|
| 月视图: 7x6 网格, 每格显示 due work-item 数 | self define |
| 周视图: 7 天横排, 每 work-item 占一行 | self define |
| 拖 work-item 到不同日期 = 改 due_date | self define |
| 月份切换 (前/后/今天) 按钮 | self define |
| 时区显示 (UTC + user TZ) | self define |

### §5.3 实现
- 新建 `frontend/src/components/calendar/MonthView.tsx` + `WeekView.tsx`
- 共用 `GanttBar` 组件 (CSS 复用)
- 拖动同 §3.4

---

## §6 Workflow 节点编辑器 (workflow/page.tsx + canvas)

### §6.1 现状
- `frontend/src/app/workflow/page.tsx` 现有 mock
- `frontend/src/components/CanvasView.tsx` 已有 Miro 风格画布

### §6.2 需求 (per AssetsLake `WorkflowAutomationPrimitives.tsx`)
| 验收 | 引用 |
|---|---|
| 节点拖动 (state 节点) 改变位置 | self define |
| 节点连接 (transition 边) 拖动出连接线 | self define |
| 双击节点 = 编辑 transition 列表 | self define |
| 保存 workflow schema 到 store + mock 后端 | self define |
| 节点合法性校验 (无 orphan state, 无 self-loop) | self define |

### §6.3 实现
- 复用 `frontend/src/components/CanvasView.tsx` + 加 dnd 扩展
- 新建 `frontend/src/components/workflow/WorkflowNode.tsx` (state 节点)
- 新建 `frontend/src/components/workflow/WorkflowEdge.tsx` (transition 边)
- `saveWorkflow(id, nodes, edges)` store action

---

## §7 跨模块联动

### §7.1 联动矩阵
| 触发 | 影响 | 引用 |
|---|---|---|
| Kanban 拖动 work-item 到 done | work-item page status 同步 + canvas 节点颜色 + audit log 写 | self define |
| Gantt 拖 milestone due_date | planning page 同步 + Calendar 同步 | self define |
| Workflow 节点连接 | work-item 状态机更新 (transition table) | self define |
| Calendar 拖 due_date | Gantt 同步 + work-item 列表排序 | self define |
| Canvas 拖节点 | workflow page 同步 | self define |

### §7.2 实现
- zustand store 单一数据源, 各 module 订阅
- 每次 mutation 走同一个 `store.update()` → 触发所有订阅者 re-render
- 跨 page 联动: 通过 `useRouter().push` 跳 + URL `?highlight=id` 参数 (已有 pattern in work-item page line 127-132)

---

## §8 多人协同 (per AssetsLake `useIssueBoardSync`)

### §8.1 同步策略
- **轮询**: TanStack Query `refetchInterval: 2_000` (per AssetsLake)
- **Cursor**: `since` 时间戳参数, 服务端只返增量
- **本地状态**: zustand store 乐观更新 + 后台 sync verify
- **冲突解决**: last-write-wins (per AssetLake pattern, 不引入 CRDT)
- **降级**: 离线时本地操作, 重连后 sync 拉差异

### §8.2 presence (可选 Phase I+)
- 暂用 toast 提示 "Board updated in another session" (per AssetsLake line 161-164)
- Phase I+ 加 WebSocket 时, 同步 presence cursor

### §8.3 实现
- 新建 `frontend/src/hooks/useBoardSync.ts` (TanStack Query 2s polling)
- 新建 `frontend/src/hooks/useWorkItemSync.ts`
- `useStore` 加 `applyRemoteChange(snapshot)` action
- `toast` 通知 (用 react-hot-toast, 已用)
- **不**引入 WebSocket 库 (per §2.2)

---

## §9 mock 数据持久化 (z-index)

### §9.1 现状
- `frontend/src/lib/store.ts` 用 zustand
- 重启浏览器数据丢失

### §9.2 改进
- 引入 `zustand/middleware` 的 `persist` + `localStorage`
- 数据键: `star-store:v1`
- 容量: 估算 5-10 MB (25 domain × 20 mock records)
- zustand persist 已成熟, 0 token 风险

---

## §10 守门 + 已知缺口

### §10.1 与 AssetsLake 对齐差距
| 项 | AssetsLake | Star 现状 | 差距 |
|---|---|---|---|
| DragEvent API | ✅ HTML5 native | ❌ 无拖动 | 补 3-5 个新组件 |
| TanStack Query | ✅ 轮询 2s | ❌ 用 zustand | 补 1 个 hook |
| react-hot-toast | ✅ | ❌ | 补 toast provider |
| useIssueBoardSync | ✅ since cursor | ❌ 无多人协同 | 补 1 hook + 1 store action |
| 拖动状态机 transition | ✅ 自动触发 | ❌ 手动按钮 | 复用 B.2.5 worktree 模式 |
| CalendarWorkloadModel | ✅ 域模型 | ❌ 纯 mock seed | 补 1 model |

### §10.2 守门 (per AGENTS.md §4)
1. **0 unsafe** (前端无 unsafe)
2. **不沿用 bc23d6c 叙事** (per 8/27 11:09)
3. **代签规则** (per 8/27 19:39/21:59): commit author = Mavis 接手 agent
4. **缺标比错标**: 显式列本节 §10.1 6 项差距
5. **5 域 Lead 独立** (per 8/21): 4-5 个子代理 = 4-5 域, 不兼任
6. **token-OLU**: 0.4-0.6M tokens / module (per 8/21 1 人·周 ≈ 1M)

### §10.3 已知缺口 (per 缺标比错标)
1. **WebSocket 实时同步** — 暂用 2s 轮询; Phase I+ 加 SSE (per spec/services/02)
2. **CRDT 协同** — 暂用 last-write-wins; 严格场景待 Yjs/automerge
3. **离线编辑** — 暂用 localStorage 缓存; 真离线 PWA 待 Phase II+
4. **presence cursor** — 暂用 toast 通知; 真实光标共享待 Phase I+
5. **移动端拖动** — HTML5 native + touch events; 待 Mobile Phase 验证
6. **accessibility** — 键盘拖动 (a11y) 未实现, 需 ARIA live region

---

## §11 子代理 worktree 并行实装 (per 8/27 19:39/21:59 + 4-5 模式)

### §11.1 5 子代理分工 (per §2 5 域 + 守门 3)
| Worker | 模块 | 文件 | 工作量 |
|---|---|---|---|
| W1 | Kanban dnd + 多人协同 | `components/board/*` + `hooks/useBoardSync.ts` + `app/board/page.tsx` | 0.5M tokens |
| W2 | Gantt 时间轴 | `components/gantt/*` + `app/planning/page.tsx` 加 gantt tab | 0.5M |
| W3 | Calendar 月/周视图 | `components/calendar/*` + `app/planning/page.tsx` 加 calendar tab | 0.4M |
| W4 | Workflow 节点编辑器 | `components/workflow/*` + `app/workflow/page.tsx` + `CanvasView` 扩展 | 0.5M |
| W5 | cross-module 联动 + store persist + toast provider | `lib/store.ts` 加 persist + 跨模块 router push + `app/layout.tsx` 加 Toaster | 0.4M |

### §11.2 顺序
1. **W5 先跑** (因为 1-4 依赖 store 升级 + toast provider), 但 W5 不动其他文件, 只升级 store + layout
2. **W1-W4 并发** 在 W5 完成后, 4 个 worktree 各自隔离

### §11.3 测试基线
- 4 个 module 各 3 测试 (per spec/integration/01 §3.1): drag 1 + write 1 + permission 1
- 总 12-15 测试, production build 必须 pass
- E2E smoke: 启 prod server, curl 4 路由全 200, 拿 HTML 验证有 draggable

### §11.4 验证后
- merge 5 wt → main
- production rebuild + restart (PID 39536 kill + new start)
- 26 路由全 200 + 新 4 模块 (board / gantt / calendar / workflow) 真实有拖动

---

## §12 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-28 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 5 模块动态交互 + 多人协同 + 跨模块联动 + 5 子代理 worktree 并行实装 | Ulysses 2026-08-28 18:14 JST 反馈"看板动态拖动 + 多人协同" |

---

## §13 引用

- `E:/AssetsLake/frontend/src/plugin-groups/production/KanbanBoardPage.tsx` (line 171-210 drag logic + line 102-110 board sync + line 402 onDragStart)
- `E:/AssetsLake/frontend/src/plugin-groups/production/GanttPage.tsx` (timeline + GanttModel)
- `E:/AssetsLake/frontend/src/plugin-groups/production/CalendarPage.tsx` (month/week + CalendarWorkloadModel)
- `E:/AssetsLake/frontend/src/plugin-groups/production/WorkflowPage.tsx` (state machine + WorkflowAutomationPrimitives)
- `E:/AssetsLake/frontend/src/hooks/useProduction.ts` (TanStack Query pattern + 2s polling)
- `D:\Star\frontend\src\app\board\page.tsx` (现有 mock)
- `D:\Star\frontend\src\app\work-item\page.tsx` (B.2.5 已实装 transition 按钮)
- `D:\Star\frontend\src\components\StateMachineDiagram.tsx` (状态机可视化)
- `D:\Star\frontend\src\components\Sidebar.tsx` (导航)
- `D:\Star\AGENTS.md` §0/§1/§3/§4 (代签 + 7 段报告 + 12 项守门)
- `D:\Star\docs\architecture\2026-08-26-upgrade\spec\integration\01-22-domain-integration-spec.md` (3 测试 / crate 基线)
