# F04 任务卡与 IPA 看板

> 状态摘要: 前端 `/kanban` 的六视图外壳与看板交互已实现, 但看板数据仅来自本地 Zustand store, 创建与移动不写服务端; 服务端授权任务卡 (RunTaskCard, six-state) 的 PG 持久化与 REST 端点已实现, 其前端调用集中在 Run "Task Cards" 面板与 Group Workspace, 不在 `/kanban` 中。两条路径的状态模型互不统一。

> 代码基线: HEAD 1643559f (单一基线); 核对方法: 路径与符号 grep + 工作树文件读取, 并对所有引用文件执行 `git diff --stat 1643559f -- <paths>` (结果为空, 工作树与 HEAD 一致); 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

在 Star 中提供任务卡 (WorkItem / Task Card) 的全生命周期管理, 并以 IPA 技能流程与看板两种视角组织任务:

- 看板视图 (任务看板) 按状态分列, 支持拖拽移动、列增删改排序、列内新建。
- IPA 流程视图把 IPA 技能工作流拆为阶段栏位, 每个技能可直接建卡。
- 服务端对任务卡的状态迁移与评审提供并发控制 (expected_version) 与幂等 (Idempotency-Key) 保护。

### 1.2 范围内 / 范围外

| 范围 | 内容 |
|---|---|
| 范围内 | `/kanban` 页面及其组件; store 中的 WorkItem / Board / Sprint 动作; 服务端 RunTaskCard 与 group API 任务卡端点; `domain-work-item` 与 `domain-board` crate 中与任务卡、看板直接相关的类型与端口 |
| 范围外 | Sprint 面板 (`/sprint`, 属其他功能文档, 本页仅保留入口卡); 甘特图与燃尽图 (`/analytics`); 日历组件的内部实现 (`CalendarView`); nema 文件侧车的内部实现; CLI 会话与 Agent 运行; `/work-item` 旧列表页 (已重定向到 `/issues`, 见 `frontend/src/app/work-item/page.tsx`) |

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 项目成员 (developer) | 在看板上新建任务卡、拖动卡片改变状态、在卡片详情中编辑字段 |
| Run 操作者 / Agent | 在 Run 的 "Task Cards" 面板领取 (claimed)、开始处理 (in_progress)、标记完成 (completed)、提交与评审 (review) |
| Group Workspace 使用者 | 在 Worktree 的 Backlog 中对任务卡执行生命周期迁移与评审 |
| 项目管理者 (project_admin) | 配置看板列 (增删、重命名、排序、WIP 显示) |

主要用户故事:

1. 作为项目成员, 我在 `/kanban` 的创建栏输入标题、选择 IPA 流程、估点, 点击 "开卡"。 (当前仅写入本地 store, 见 FR-F04-02 的状态说明。)
2. 作为 Run 操作者, 我在 Run 的 "Task Cards" 面板领取一张 pending 任务卡到当前 Worktree, 完成后标记 completed。 (服务端路径, 已接线。)
3. 作为 Group Workspace 使用者, 我在 Backlog 中把 live 任务卡从 in_progress 迁移到 review 并提交评审。 (服务端路径, 已接线。)

## 3. 需求

优先级: P0 = 核心闭环; P1 = 重要; P2 = 增强。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-F04-01 | `/kanban` 提供六个视图切换 (Sprint / 任务看板 / IPA 流程图 / 按文件开卡 / 时间线 / 日历) | P0 | ✅ | `frontend/src/app/(app)/kanban/page.tsx:68` (VIEW_TABS), `:379`/`:402`/`:437`/`:448`/`:494`/`:508` (各视图渲染分支) |
| FR-F04-02 | 创建栏六个控件 (标题、IPA 流程、工作流、关联文件、估点、开卡按钮) | P0 | 🟡 | 控件存在: `kanban/page.tsx:279` (标题), `:291` (IPA), `:304` (工作流), `:317` (文件), `:331` (估点), `:343` (开卡); 写入仅到本地 store `:116-127` (tenant_id/project_id/sprint_id/reporter_id 为硬编码常量), 未调用服务端 |
| FR-F04-03 | 每列 "+ Add task" 打开详情抽屉 (新建模式, status 预填) | P0 | 🟡 | `frontend/src/components/board/KanbanBoard.tsx:56` (props `onRequestNewWorkItem`), `kanban/page.tsx:412-422` (接入), `frontend/src/components/board/WorkItemDetailDrawer.tsx:77-79` (模式类型), `:267-285` (`handleCreate` 调用 `addWorkItem`, 仅本地 store) |
| FR-F04-04 | 看板列增加、删除、重命名、拖动重排; 兜底列 (todo) 不可删 | P1 | ✅ | `frontend/src/lib/store.ts:984` (addBoardColumn), `:1000-1007` (removeBoardColumn 拒绝删除兜底列), `:1025` (renameBoardColumn), `:1034` (reorderBoardColumns); `frontend/src/components/board/constants.ts` (TODO_FALLBACK_STATUS); UI 在 `KanbanBoard.tsx:386-444` |
| FR-F04-05 | 卡片拖拽在列间移动并调用状态迁移 | P0 | 🟡 | 拖拽与 drop 在 `KanbanBoard.tsx:170`, `:188`, `:229`, `:341-359`; 迁移动作为本地 `transitionWorkItem` (`store.ts:483-495`), 不调用服务端 lifecycle 端点 |
| FR-F04-06 | 冲刺启动与完成 | P1 | ✅ (位于 /sprint) | store 动作 `frontend/src/lib/store.ts:875` (startSprint), `:899` (completeSprint); UI 调用 `frontend/src/components/sprint/SprintBoardView.tsx:620`, `:698`; `/kanban` 的 Sprint 标签仅为入口卡 (`kanban/page.tsx:379-400`) |
| FR-F04-07 | Backlog 快捷建卡 (从 IPA 流程 / 按文件开卡) | P1 | 🟡 | "按文件开卡" 标签存在 (`kanban/page.tsx:448-491`, 读取 nemaStore); "从 IPA 流程建卡" 按钮在 `/kanban` 中未找到, 原 Backlog 区已随 Sprint 收敛移除 |
| FR-F04-08 | 预览数据标识 (seed 数据不冒充正式视图) | P0 | 🟡 | 横幅 `kanban/page.tsx:271-275` 无条件显示 (不随 `hostConnected` 切换), 文案称 "宿主会话接入后自动切换为授权投影", 但代码中无该切换路径 (见 §4.4、§6) |
| FR-F04-09 | 未授权 / 未接入 / 空数据三态诚实展示, 未接入时禁止写入 | P0 | ✅ | `kanban/page.tsx:104-108` (groupApi / unauthorized / createDisabled), `:424-432` (unavailableState 传入); `KanbanBoard.tsx:104` (类型), `:266-271` (StatusNotice 渲染) |
| FR-F04-10 | IPA 流程图: 按阶段展示 IPA 技能, 技能可关联任务卡 | P1 | 🟡 | `frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.tsx` (存在, 5 个测试); 数据来自 store 与静态目录 `frontend/src/lib/ipa-skills/catalog.ts`; store 中的技能关联更新位于 `store.ts:655-690` (`ipa_skill_ids` 的增删); 无服务端写入 |
| FR-F04-11 | 时间线视图 | P2 | 🗑 (不在 /kanban) | `kanban/page.tsx:494-506` 仅为跳转卡到 `/analytics`; 甘特组件 `frontend/src/components/gantt/GanttView.tsx` 在 `frontend/src/app/(app)/analytics/page.tsx:35`, `:219` 中挂载 |
| FR-F04-12 | 日历视图 | P2 | ✅ | `kanban/page.tsx:508-519` 渲染 `CalendarView` (来自 `frontend/src/components/calendar/`) |
| FR-F04-13 | 任务卡卡片: 点击打开详情, 聊天入口, 拖拽源 | P1 | ✅ | `frontend/src/components/board/KanbanCard.tsx:116` (draggable), `:163-166` (聊天入口链接) |
| FR-F04-14 | 服务端任务卡创建 (按 Worktree 或 Run 作用域, Idempotency-Key 必需) | P0 | ✅ | `crates/star-api-rest/src/group_api/work_items.rs:242-271` (路由), `:274-319` (create_work_item, scope `work-item:write`); 前端 Run 路径 `frontend/src/lib/run/runDirectoryApi.ts:709`, `:748` |
| FR-F04-15 | 服务端生命周期迁移 (six-state, expected_version 乐观并发, 幂等) | P0 | ✅ | `crates/star-api-rest/src/group_api/work_items.rs:589-630`, `:632-675`; 状态机 `crates/domain-work-item/src/run_task_card.rs:212-222`; PG 校验 `crates/infrastructure/src/run_task_card_pg.rs:529`; 前端 `frontend/src/lib/run/runDirectoryApi.ts:777-801`, `frontend/src/app/worktree/[id]/group/page.tsx:2259-2285` |
| FR-F04-16 | 服务端评审 (submit / accept / reject) | P1 | ✅ | `crates/star-api-rest/src/group_api/work_items.rs:677-` (review_work_item), `crates/domain-work-item/src/run_task_card.rs:245-260` (动作), `:385-411` (命令校验); 前端 `runDirectoryApi.ts:824-847`, `group/page.tsx:2331` |
| FR-F04-17 | 服务端任务卡列表 (按 Worktree 与 Run) | P0 | ✅ | `group_api/work_items.rs:321` (list_work_items); `frontend/src/lib/group/worktreeGroupApi.ts:1000-1001`; `frontend/src/lib/group-api/group-context.ts:207`; `frontend/src/lib/run/runDirectoryApi.ts:695-701` |
| FR-F04-18 | 看板 (kanban 页) 读取服务端任务卡 | P0 | ⬜ | `/kanban` 仅读取 `useStore((s) => s.workItems)` (`kanban/page.tsx:171`); 无 store 写入方来自 group API (见 §4.4) |
| FR-F04-19 | 看板 Board 配置服务端持久化 (列与 WIP) | P1 | ⬜ | `crates/domain-board` 仅有内存实现 (`lib.rs:680`, `:1084`), 无 PG 适配器, 无 REST 路由 (`crates/star-api-rest/src` 无 board 路径) |
| FR-F04-20 | WIP 限制为软告警 (不阻止流转) | P2 | 🟡 | 前端软告警 `KanbanBoard.tsx:320-321`, `:471`; 领域层硬拒绝 `crates/domain-board/src/lib.rs:922-924` (与 domain-board-spec INV-B-05 相反) |
| FR-F04-21 | 生命周期事件发布 (Outbox) | P2 | ⬜ | `crates/infrastructure/src/run_task_card_pg.rs:4-5` 注释明确 "does not publish lifecycle Outbox events" |

## 4. 设计

### 4.1 概念模型

存在三套并存的任务模型, 各自的状态集合不同:

| 层 | 模型 | 状态集合 | 证据 |
|---|---|---|---|
| 前端 store (UI) | `WorkItem` (kind 5 种, status 6 种) | `todo / in_progress / review / blocked / done / wontfix` | `frontend/src/types/ids.ts:207-208` (WorkItemStatus), `:241` (WorkItem), `:209` (WorkItemKind) |
| 前端默认列模板 | `KANBAN_COLUMNS` | 5 列 (无 wontfix) | `frontend/src/mocks/data/kanban.ts:7-13` |
| 前端状态机常量 | `WORKITEM_SM` | 3 状态 `todo / in_progress / done` | `frontend/src/types/ids.ts` 约 1141-1152 |
| 领域 WorkItem (spec) | `WorkItem` | 3 状态 `Todo / InProgress / Done` (可回退/重开) | `crates/domain-work-item/src/lib.rs:198` (WorkItemStatus), `:213-219` (as_str), `:1111-1114` (测试断言回退与重开) |
| 领域 RunTaskCard (服务端) | `RunTaskCardStatus` | 6 状态 `pending / claimed / in_progress / completed / failed / cancelled` | `crates/domain-work-item/src/run_task_card.rs:183-209` |
| 领域 RunTaskCard 评审 | `RunTaskCardReviewState` | `none / pending_review / accepted / rejected` | `run_task_card.rs:281-300` |
| 领域看板 | `BoardColumn` | 默认 3 列 `Todo / InProgress / Done`, 列名为自由字符串 | `crates/domain-board/src/lib.rs:206-230` |
| 领域看板 (spec) | Board/Column/Swimlane | 列绑定 Workflow State | `docs/specs/domain-board-spec.md` §2 |

四层并未统一: UI 使用 6 态, 服务端 RunTaskCard 使用另外 6 态 (名称不同, 含 claimed/failed, 缺 review/blocked/wontfix), 领域 WorkItem 与 WORKITEM_SM 为 3 态。`/kanban` 中的状态写入不会映射到服务端任务卡。

### 4.2 数据模型

**服务端 (PG, 迁移 `db/migrations/2026-09-29-worktree-group-phase-2c.sql`):**

| 表 | 行 | 用途 |
|---|---|---|
| `multica.task_metadata` | 11 | 任务卡元数据 (标题、类型、优先级、标签、reporter) |
| `multica.work_item_worktree` | 37 | 任务卡与 Worktree 的关联 |
| `multica.task_lifecycle_current` | 58 | 当前生命周期投影 (status, review_state, active_worktree_id, version, claimed_by, claim_expires_at) |
| `multica.task_command_idempotency` | 92 | 幂等回执 (request_hash, response_body) |

另有审计表 `multica.task_lifecycle_audit` 与 outbox `multica.task_run_outbox` (由 `run_task_card_pg.rs:1059`, `:33-42` 注释引用); 其中 lifecycle outbox 未发布 (见 FR-F04-21)。

**前端 store (Zustand, `frontend/src/lib/store.ts`):** `workItems: WorkItem[]` (`:149`), `board: Board` (`:176`, 初始为空 `columns: []`, 见 `:433`), `sprints`, `identities`。`WorkItem` 字段见 `ids.ts:241-` (含 `key, kind, status, priority, assignee_id, reporter_id, story_points, labels, sprint_id, workflow_id, due_date`)。`Board` (`ids.ts:925-935`) 为 `columns: {status, name?, work_item_ids, wip_limit?}[]`。

**领域 WorkItem (spec 与代码):** `WorkItemType` 含 Epic/Story/Task/Bug/Subtask/AITask 六类 (`crates/domain-work-item/src/lib.rs:167-182`); RunTaskCard 的 `RunTaskCardItemType` 同样为六类 (`run_task_card.rs:95-120`); 前端 `WorkItemKind` 为 story/task/bug/spike/epic 五类, 不含 subtask/ai_task/spike 对应后端 (`ids.ts:209`)。

### 4.3 接口

**服务端 REST (group API, 生产路由由 `build_group_router` 挂载, 见 `crates/star-api-rest/src/main.rs:127-133`):**

| 方法与路径 | 处理函数 | 文件:行 | scope | 说明 |
|---|---|---|---|---|
| GET/POST `/api/v1/worktrees/{worktree_id}/work-items` | list_work_items / create_work_item | `group_api/work_items.rs:242-247`, `:274`, `:321` | `work-item:read` / `work-item:write` | 按 Worktree 列表与创建 |
| GET `/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}` | get_work_item | `:260-263`, `:558` | read | 单卡读取 |
| POST `/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/lifecycle` | transition_work_item | `:264-267`, `:589-630` | write | 迁移, body `target_status / expected_version / correlation_id / reason / focus_worktree_id` (`:178-185`) |
| POST `.../review` | review_work_item | `:268-271`, `:677` | write | 评审, body `action / expected_version` (`:187-195`) |
| GET/POST `/api/v1/engineering-runs/{run_id}/work-items` | list_run_work_items / create_run_work_item | `:248-251` | read / write | Run 作用域 |
| POST `/api/v1/engineering-runs/{run_id}/work-items/{work_item_id}/lifecycle` | transition_run_work_item | `:252-255`, `:632-675` | write | Run 作用域迁移 |
| POST `/api/v1/engineering-runs/{run_id}/work-items/{work_item_id}/review` | review_run_work_item | `:256-259` | write | Run 作用域评审 |

创建与迁移请求必须携带 `Idempotency-Key` 头 (`work_items.rs:285`, `:602`, `:1012`)。

**旧版内存路由 (非生产):** `crates/star-api-rest/src/lib.rs:96-100` 与 `:261-265` 注册 `/work-items` 系列, 绑定 `routes::work_items` (内存服务 `InMemoryWorkItemService`)。`main.rs:52` 注释说明 `build_router` 仅保留给测试与本地路由, 生产二进制使用 `build_group_router`。

**前端客户端:**

- `frontend/src/lib/run/runDirectoryApi.ts:695` (listRunWorkItems), `:709` (createRunTaskCard), `:748` (POST engineering-runs/.../work-items), `:777` (transitionRunTaskCard), `:824` (reviewRunTaskCard); 状态白名单 `:787`。
- `frontend/src/lib/group/worktreeGroupApi.ts:1000` (listWorkItems), `:1044` (lifecycle), `:1173` (review), `:1345` (createWorkItemOnCanvas, 对应端点未在 `group_api/canvas.rs` 的路由表中找到, 见 §6)。
- `frontend/src/lib/group-api/group-context.ts:207` (listWorkItems)。
- 无非测试代码调用 `worktreeGroupApi` 的 POST `/worktrees/{id}/work-items` 创建方法。

**领域端口 (代码):** `RunTaskCardCommandPort` 含 `create / transition / review` (`crates/domain-work-item/src/run_task_card.rs:513-534`); `WorkItemCommandPort` / `WorkItemQueryPort` / `WorkItemRepository` (`crates/domain-work-item/src/lib.rs:547`, `:586`, `:604`); `BoardCommandPort` / `BoardQueryPort` / `BoardRepository` (`crates/domain-board/src/lib.rs:567`, `:606`, `:627`)。

**spec 中的 REST 设计 (未实现):** `docs/specs/domain-work-item-spec.md` §4 中的 `/v1/work-items` 与 `docs/specs/domain-board-spec.md` §4 中的 `/v1/projects/{P}/board` 在代码中无对应路由 (`crates/star-api-rest/src` 中无 `/v1/`、无 board 路径)。

### 4.4 状态机与关键流程

**服务端 RunTaskCard 生命周期** (`crates/domain-work-item/src/run_task_card.rs:212-222`, `allows`):

```
pending  -> claimed | cancelled
claimed  -> in_progress | cancelled
in_progress -> completed | failed | cancelled
failed   -> pending
```

PG 适配器在事务内再次校验 (`crates/infrastructure/src/run_task_card_pg.rs:529`), 并允许过期 claim 的重新排队 (`expired_claim_requeue`)。`claimed` 与 `in_progress` 需要 focus worktree (`frontend/src/lib/run/runDirectoryApi.ts:790`)。

**前端写入流程 (Group Workspace, live 模式):** `frontend/src/app/worktree/[id]/group/page.tsx:2259-2285` 仅在 `groupProjection.mode === "live"` 且 `groupApi` 非空时发起迁移, 携带 `expected_version` 与 UUID `Idempotency-Key`; 评审见 `:2331`。读取经 `frontend/src/lib/group/groupProjection.tsx:254-256` 调用 `client.listWorkItems`。

**`/kanban` 写入流程 (实际):**

1. `kanban/page.tsx:104-108` 计算 `groupApi`、`unauthorized`、`createDisabled`。
2. `handleCreateTaskCard` (`:110`) 在 `createDisabled` 为假时调用 `addWorkItem`, 写入 `tenant_id: "tn-001"`, `project_id: "prj-physis"`, `sprint_id: "sp-12"`, `reporter_id: "usr-002"` (`:117-125`)。宿主已连接时同样只写本地 store。
3. 卡片拖拽与抽屉保存调用 `transitionWorkItem` (`store.ts:483`) 与 `updateWorkItemField`; 均不调用服务端。
4. `store.createWorkItem` (`store.ts:732`) 为 agent 指派时的 TMO M-N8 分支, 代码中无 UI 调用点 (grep 非测试代码无调用)。

**期望中的看板流程 (未实现):** 宿主接入后, `/kanban` 应从 group API 读取授权任务卡投影并把写操作路由到 lifecycle 端点。横幅 `kanban/page.tsx:274` 的文案描述了此切换, 代码中无对应实现。

**看板列与状态同步:** store 的 `reconcileBoard` (`store.ts:80-100`) 保证 `board.columns[].work_item_ids` 为 `workItems[].status` 的派生视图; 列删除时卡片回落到 `todo` (`constants.ts`)。

### 4.5 权限与安全

| 机制 | 位置 | 状态 |
|---|---|---|
| 服务端 scope 校验 | `work_items.rs:282`, `:328`, `:597`, `:640` (`require_scope`, `work-item:read` / `work-item:write`) | ✅ |
| 租户与 Worktree 作用域 | `run_task_card_context` (`work_items.rs`) 与 PG 适配器的 `authorize_run_task_scope` (`run_task_card_pg.rs:33-49` 注释列出的 RLS 表与绑定表) | ✅ |
| 幂等键 | `work_items.rs:1012` (`Idempotency-Key`), 表 `multica.task_command_idempotency` (迁移 `:92`) | ✅ |
| 乐观并发 | `expected_version` (`run_task_card.rs:354-` 与 `:385-`; 前端 `group/page.tsx:2262`) | ✅ |
| 前端未授权禁写 | `kanban/page.tsx:108` (`createDisabled`), `:288`, `:296`, `:309` (输入禁用) | ✅ (仅 /kanban 创建栏) |
| 服务端 RLS | spec 要求 13 类表 RLS; 迁移文件中的 RLS 策略未核对 (需读取 RLS 语句并与 13 类表清单比对) | ❓ |

### 4.6 依赖关系

| 依赖 | 方向 | 证据 |
|---|---|---|
| `domain-workflow` (状态定义) | 设计依赖, 代码未接入 | `domain-work-item-spec.md` §1; `frontend/src/types/ids.ts:1142` 注释称三态来自 `domain-work-item` 而非 workflow |
| `infrastructure` (PG 适配器) | 运行时依赖 | `crates/infrastructure/src/run_task_card_pg.rs:149` (`PgRunTaskCardAdapter`), `crates/star-api-rest/src/group_api.rs:374` |
| `domain-audit` | 设计依赖 (审计记录) | `run_task_card_pg.rs:1059` (`task_lifecycle_audit` 写入) |
| `star-mcp` 工具 | 使用内存服务 | `crates/star-mcp/src/handlers/work_item.rs` 引用 `domain_work_item` |
| Group Workspace 与 Run 面板 | 前端消费 | `frontend/src/components/run/RunWorkspaceShell.tsx:222`, `frontend/src/app/worktree/[id]/group/page.tsx` |
| Analytics 甘特图 | `/kanban` 的时间线为跳转 | `frontend/src/app/(app)/analytics/page.tsx:35`, `:219` |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 看板页壳 (六视图、创建栏、横幅、禁写态) | `frontend/src/app/(app)/kanban/page.tsx` (531 行) | 🟡 | `:68`, `:104-108`, `:271-275`, `:277-353` |
| 看板列容器与拖拽、列操作、未接入态 | `frontend/src/components/board/KanbanBoard.tsx` (580 行) | ✅ (数据仍为本地 store) | `:56`, `:97`, `:266-271`, `:386-444` |
| 单卡渲染与聊天入口 | `frontend/src/components/board/KanbanCard.tsx` (234 行) | ✅ | `:116`, `:163-166` |
| 详情抽屉 (新建 / 查看) | `frontend/src/components/board/WorkItemDetailDrawer.tsx` (1033 行) | 🟡 (仅本地 store) | `:77-79`, `:267`, `:287` |
| 看板筛选 | `frontend/src/components/board/KanbanFilters.tsx` (121 行) | ✅ | `kanban/page.tsx:58`, `:405-410` |
| IPA 流程图 | `frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.tsx` (268 行) | 🟡 | `kanban/page.tsx:437-447`; `frontend/src/lib/ipa-skills/catalog.ts` |
| 时间线 (甘特) | `frontend/src/components/gantt/GanttView.tsx` (100 行) | 🗑 (不在 /kanban; 挂载于 /analytics) | `analytics/page.tsx:35`, `:219` |
| Sprint 面板 | `frontend/src/components/sprint/SprintBoardView.tsx` (714 行) | ✅ (属 /sprint) | `kanban/page.tsx:379-400` 入口卡 |
| Store 任务卡与看板动作 | `frontend/src/lib/store.ts` (1794 行) | 🟡 (本地) | `:483`, `:536`, `:732`, `:875`, `:899`, `:984-1034` |
| 共享类型 | `frontend/src/types/ids.ts` | ✅ | `:207-241`, `:925` |
| 列模板 | `frontend/src/mocks/data/kanban.ts` | 🟡 (5 列, 无 wontfix) | `:7-13` |
| 看板横幅 / 生命周期条 | `kanban/page.tsx:271-275` | 🟡 (横幅无条件显示) | 见 FR-F04-08 |
| 宿主同步 Hook | `frontend/src/hooks/useBoardSync.ts` | 🗑 (唯一调用方为已重定向的 `/board`) | `frontend/src/app/board/page.tsx:93`; `lib/redirects.ts:61` (`/board` → `/kanban`); `frontend/next.config.js:5-7` |
| 工作流守卫 | `frontend/src/lib/workitem-guard.ts` | 🟡 (有测试 13 个, 无生产调用方) | 测试 `workitem-guard.test.ts` |
| 旧 /work-item 页 | `frontend/src/app/work-item/page.tsx` | 🗑 (重定向到 /issues) | 文件头注释 |
| 服务端 RunTaskCard 领域 | `crates/domain-work-item/src/run_task_card.rs` (534 行) | ✅ | `:183-222`, `:513-534` |
| 服务端 PG 适配器 | `crates/infrastructure/src/run_task_card_pg.rs` (1295 行) | ✅ (无 lifecycle outbox) | `:149`, `:178`, `:467-`, `:529`, `:584-` |
| group API 任务卡端点 | `crates/star-api-rest/src/group_api/work_items.rs` (1238 行) | ✅ | `:242-271`, `:274-675` |
| group API 状态注入 | `crates/star-api-rest/src/group_api.rs` | ✅ | `:356-384` (`GroupApiState` 注入 PG 适配器) |
| 生产路由装配 | `crates/star-api-rest/src/main.rs` | ✅ | `:127-133` |
| 旧内存 work-items 路由 | `crates/star-api-rest/src/routes/work_items.rs` | 🗑 (非生产, 仅测试与本地) | `lib.rs:96-100`, `main.rs:52` |
| 领域 WorkItem (spec 实现) | `crates/domain-work-item/src/lib.rs` (1931 行) | 🟡 (仅内存 `InMemoryWorkItemService` `:629`, `InMemoryWorkItemRepository` `:1024`; 无 PG、无 REST) | `:198`, `:547-604` |
| 领域看板 | `crates/domain-board/src/lib.rs` (2153 行) | 🟡 (仅内存 `:680`, `:1084`; 无 PG、无 REST; 默认 3 列 `:206`; WIP 硬拒绝 `:922-924`) | `:567-627` |
| 迁移 (任务卡表) | `db/migrations/2026-09-29-worktree-group-phase-2c.sql` | ✅ | `:11`, `:37`, `:58`, `:92` |
| 共享 DTO (后端) | `crates/star-dto/src/lib.rs` | ❓ (未核对内容) | 未读取 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| `/kanban` 不读取服务端任务卡, 横幅承诺的 "宿主接入后自动切换为授权投影" 未实现 | 已连接用户看到的是 seed 数据; 创建与移动只改本地 store, 无持久化 | 在 `/kanban` 中以 `listRunWorkItems` / `listWorkItems` 替换 store 读取, 并把创建与迁移路由到 lifecycle 端点; 同步收敛状态集合 (见下一行) |
| 四层状态模型未统一 (UI 6 态 / RunTaskCard 6 态不同名 / 领域与 WORKITEM_SM 3 态) | 看板列与服务端状态无法一一映射, 迁移结果无法回写 | 以 RunTaskCard 六态为权威 (已有 PG 与 REST), 定义 UI 列与六态的映射表, 或在领域层补充 review/blocked/wontfix 的扩展机制 (spec 允许 ProjectPolicy 扩展) |
| `/kanban` 创建栏写死 tenant/project/sprint/reporter (`kanban/page.tsx:117-125`) | 多租户与多项目数据错误归属 | 从当前 project 与 identity 上下文读取, 并在未接入时保持禁写 |
| 领域看板 WIP 硬拒绝 (`domain-board/src/lib.rs:922-924`) 与 spec INV-B-05 软告警相反 | 与前端软告警行为不一致 | 按 spec 改为告警, 或更新 spec (以 spec 与代码哪个为准需产品决策) |
| 领域看板与领域 WorkItem 无 PG 适配器、无 REST 路由 | 看板配置不持久化; spec 中 `/v1/*` 端点不存在 | 为 `BoardRepository` 与 `WorkItemRepository` 增加 PG 实现, 并以 group API 或新路由挂载 |
| 生命周期事件未发布 Outbox (`run_task_card_pg.rs:4-5`) | 订阅方 (audit, search, collaboration) 无法收到迁移事件 | 在 migration 中增加 lifecycle outbox 写入与发布 |
| `workitem-guard` 无生产调用 | 守卫规则未生效 | 接入 `transitionWorkItem` 前置校验, 或删除 |
| `useBoardSync` 无存活页面调用方 | 轮询同步代码无效; 若恢复, 2 秒轮询与仓库实时协作约束 (AGENTS.md §4 "实时协作与冲突保护") 不一致 | 删除或改为事件推送 |
| `createWorkItemOnCanvas` (`worktreeGroupApi.ts:1345`) 指向的端点未在 `group_api/canvas.rs` 路由表中找到 | 前端调用可能 404 | 核对 canvas 路由并补齐或删除调用 |
| 无 POST `/worktrees/{id}/work-items` 的前端调用 | 旧 Worktree 作用域创建路径无 UI 入口 | 明确 Run 作用域为唯一入口, 或补充 UI |
| 无 `/kanban` 的 e2e 测试 | 创建、拖拽、禁写态无端到端覆盖 | 为 `/kanban` 增加 Playwright 用例 |
| 时间线与 IPA 从 IPA 流程建卡的快捷入口缺失 | FR-F04-07 不完整 | 恢复 Backlog 快捷入口或在 Sprint 面板提供 |
| `KANBAN_COLUMNS` 为 5 列, UI 类型为 6 态 | wontfix 列在默认模板中缺失 | 统一为单一来源常量 |

## 7. 验收与测试

| 测试 | 位置 | 覆盖 | 未覆盖 |
|---|---|---|---|
| KanbanBoard 组件 | `frontend/src/components/board/KanbanBoard.test.tsx` (11 个) | 列渲染、未接入态 | 服务端写入 |
| KanbanCard | `frontend/src/components/board/KanbanCard.test.tsx` (4 个) | 卡片渲染 | 拖拽后服务端状态 |
| IPA 流程图 | `frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.test.tsx` (5 个) | 阶段渲染 | 服务端关联 |
| Kanban mock | `frontend/src/mocks/__tests__/kanban.test.ts` (3 个) | 列数据 | 页面集成 |
| 工作流守卫 | `frontend/src/lib/workitem-guard.test.ts` (13 个) | 守卫规则 | 生产接线 (守卫未被调用) |
| Run 任务卡面板 | `frontend/src/components/run/RunTaskCardsPanel.test.tsx` (19 个) | 领取、开始、完成、评审、创建、恢复 | `/kanban` 面板 |
| 宿主同步 Hook | `frontend/src/hooks/useBoardSync.test.tsx` (3 个) | Hook 行为 | 页面挂载 |
| Sprint 页面 | `frontend/src/app/(app)/sprint/page.test.tsx` (13 个) | 冲刺动作 | 任务看板 |
| Run 目录 API | `frontend/src/lib/run/runDirectoryApi.test.ts` 与 `runDirectoryApi.bi.test.ts` | 任务卡端点调用 | 与 `/kanban` 的关系 |
| group API 任务卡 | `crates/star-api-rest/src/group_api/work_items.rs` 内 3 个测试 | 路由焦点、旧请求体哈希形状 | 完整 PG 集成 |
| PG 适配器 | `crates/infrastructure/src/run_task_card_pg.rs` 内 3 个测试 | 待核对 | 完整事务路径 |
| 领域看板 | `crates/domain-board/src/lib.rs` 内 12 个测试 | 内存服务、WIP 拒绝 | PG |
| 领域 WorkItem | `crates/domain-work-item/src/lib.rs` 内 16 个测试 | 三态迁移、回退与重开 | PG 与 REST |
| RunTaskCard 领域 | `crates/domain-work-item/src/run_task_card.rs` 无 `#[test]` | 无 (由 PG 与 group API 测试间接覆盖, 未核对) | 直接单元测试 |

**已验证 (依据上述测试与代码阅读):** 六视图切换、创建栏控件、列操作、服务端迁移状态机与校验。

**未验证:** `/kanban` 与服务端的端到端行为; PG 迁移的 RLS 策略; WIP 行为的前后端一致性; 页面横幅与宿主接入切换。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-KANBAN-001.md` | 六视图、创建栏六控件、列操作、FR 清单与文件映射 | (a) 自称 "唯一权威文档", 但其 §1 列定义 (6 列) 与代码默认模板 (5 列) 不符; (b) 修订履历引用 `c0f149f2` 为 v1.0 建档提交, 但该哈希实为 "feat(settings): Mod 目录面板" (`git log` 可核); (c) 引用 `9187f43d` 与 `cf48593b` 与提交说明相符; (d) §2 FR-KB-005 与 FR-KB-009 指向的位置已迁至 `/sprint` 与 `/analytics` | 采纳其视图清单与控件清单 (与代码一致); 状态以代码为准, 本文件重新标注; 修订履历中的 `c0f149f2` 不予采纳 |
| `docs/specs/domain-work-item-spec.md` | WorkItem 聚合、三态默认状态机、INV-WI-01~09、REST 端点、错误码 | 三态与代码一致 (`lib.rs:198`); REST `/v1/work-items` 未实现; 6 类型与代码一致 (`lib.rs:167`); 状态迁移与 WorkflowDefinition 未接入 | 采纳其领域规则作为设计意图 (⬜ 标注); 端点与 Workflow 依赖标注为未实现 |
| `docs/specs/domain-board-spec.md` | Board/Column/Swimlane 实体、INV-B-01~05、WIP 软告警 | 默认列 3 列 (`domain-board/src/lib.rs:206-230`), 而非 spec 所述列绑定 Workflow; WIP 在代码中为硬拒绝 (`:922-924`); 无 REST 路由 | WIP 行为以 spec 为设计意图 (FR-F04-20 标 🟡); 默认列以代码为准 |
| `docs/prototypes/1b · 任务 · IPA 看板-html/README.md` | 仅说明 `TasksIpa.dc.html` 为参考原型 | README 本身不含设计内容, 为指针文件 | 不作为需求来源, 原型文件见下一行 |
| `docs/prototypes/1b · 任务 · IPA 看板-html/TasksIpa.dc.html` | 六视图标签 (`TasksIpa.dc.html:648`), 创建栏控件 (`:120-124`), Backlog 快捷建卡 (`:114-115`), 工作流与 Sprint 选择 (`:517-518`) | 原型状态集合为 `backlog / todo / doing / review / done / archived` (`:642`), 与 SRS (6 列) 和代码 (6 态 / 5 列) 三者各不相同; 原型包含 Backlog 区、Sprint 选择与 CLI 会话菜单, 代码均未实现于 `/kanban` | 视觉与控件结构采纳为设计意图; 状态集合以代码为准, 原型状态集合列为 ⬜ 设计意图 |
| `docs/frontend/design/ui-detailed-design.md` | Tabs/StatusPill 组件规范、`/work-item` 页签设计 (§5.3) | §5.3 的 `/work-item` 页签已由 `frontend/src/app/work-item/page.tsx` 重定向到 `/issues` (非 `/kanban`); §4.2 WorkItemStatus 六值与代码一致 | 仅采纳 StatusPill 六值作为展示约定; §5.3 不采纳 (页面已迁移) |
| `AGENTS.md` (§0 产品导航约束) | 任务卡是 Branch 的子项, 在 Backlog 内以子窗口打开; Group Workspace 为规范入口 | 与 `/kanban` 作为独立 Project 级看板的实现不同 | 规范入口视为 Group Workspace Backlog; `/kanban` 的定位在本文件 §1.2 按现状标注 |
| `frontend/src/app/board/page.tsx` 与 `lib/redirects.ts` | `/board` 历史入口 | `lib/redirects.ts:61` 将 `/board` 重定向到 `/kanban`, 经 `frontend/next.config.js:5-7` 接入; 页面文件仍存在 | 视为已退役入口 (🗑), 其 `useBoardSync` 调用不计入存活功能 |
| `frontend/src/app/worktree/[id]/group/page.tsx` | Group Workspace 的 live 迁移与评审 | 与 `/kanban` 的本地 store 写入并存于同一代码库 | 作为服务端写入的权威前端路径 |
| `db/migrations/2026-09-29-worktree-group-phase-2c.sql` | 任务卡四张表定义 | 与 PG 适配器引用的表一致 (`run_task_card_pg.rs:33-49`) | 采纳 |

### 8.1 未决项

1. 四层状态模型以哪一层为权威 (RunTaskCard 六态 / UI 六态 / 领域三态), 需产品决策。
2. WIP 行为以 domain-board-spec (软告警) 还是代码 (硬拒绝) 为准, 需产品决策。
3. `/kanban` 是否应成为 Project 级看板 (保留本地 store 原型) 还是迁移到 Group Workspace Backlog, 需依据 AGENTS.md §0 决定。
4. `createWorkItemOnCanvas` 目标端点是否存在, 需核对 canvas 路由。
5. `crates/star-dto/src/lib.rs` 与迁移中的 RLS 策略, 需单独核对。
