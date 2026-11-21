# F09 低代码流程与工作流模板

> 状态摘要: 领域层已有状态流转工作流（`Workflow` / `Transition` / `Guard`，含单测）和自动化规则的内存实现，前端 `/workflow` 与 `/automation` 页面可渲染但数据源从未写入；低代码自动化流程（Flow）、内置工作流模板、看板 17 列模板、模板选择器与 REST 接口均未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

本功能覆盖三类相互区分的"流程"对象，并给出各自的入口与状态:

1. **状态流转工作流（Status-Transition Workflow）**: 定义一个 Project 内 WorkItem 的状态机（state / transition / guard），入口为 `/workflow`，领域实现位于 `crates/domain-workflow`（SRS-CANVAS-WORKFLOW-001 §2.4 表第 1 行）。
2. **自动化流程（Automation Flow，即 Flow）**: n8n 式可执行节点图，编排"触发 → 动作/条件/循环 → 执行历史"，并支持标签绑定派生任务卡；入口规划为 `/automation` 页的 Flow 与 Flow Tags 标签页（SRS-CANVAS-WORKFLOW-001 §8.4 行 1306，W1–W15）。
3. **模板**: 三个相互独立的模板目录——(a) 状态流转工作流的 4 套内置模板（workflow-templates-spec §6）；(b) Flow 的 3 套内置模板（SRS-CANVAS-WORKFLOW-001 §4.14，W14）；(c) 看板 17 列模板（SRS-WORKFLOW-TEMPLATE-001 WT-1..WT-8）。

### 1.2 范围内 / 范围外

**范围内**

- `/workflow` 页面、`/automation` 页面及其导航项（`frontend/src/lib/nav/registry.ts:376-385`、`:501-509`）。
- `crates/domain-workflow`、`crates/domain-automation` 中与状态流转工作流、自动化规则相关的领域代码。
- Store 中 `workflows`、`automationRules`、`savedWorkflows` 切片（`frontend/src/lib/store.ts`）。
- 三类模板的设计要求与实现状态（第 3 节 FR-09-07 至 FR-09-11、FR-09-27 至 FR-09-34）。

**范围外（只在本文描述依赖，不裁定）**

- 画布本体与 `CanvasView` 元素模型，归 F06。`/canvas/[id]` 的"流程"标签页挂载的 `WorkflowBuilder`（`frontend/src/app/canvas/[id]/page.tsx:235-239`）在 §4、§5 描述其现状，但其所属画布由 F06 负责。
- 看板本体（列渲染、拖拽、WorkItem 卡片），归 F04。WT-4 只覆盖"从模板应用到看板"的入口与数据。
- `/review` 页面（`frontend/src/app/(app)/review/page.tsx`，前端 mock）属于审核域，本文只把其中的 `ReviewKind` 作为旁证引用。
- Group Shell 共享底部聊天栏（W15.1），归群组外壳功能；本文只记录 W15 各条的 Flow 侧缺口。

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 项目管理者（PM） | 为 Project 选择状态流转模板（如瀑布式），查看各状态迁移与守卫；把自动化规则或 Flow 绑定到标签，派生任务卡进入 Backlog。 |
| 开发者 | 在 `/automation` 查看规则与触发条件；手动运行一次 Flow；查看执行历史并从失败节点重跑。 |
| 美术 / 创作者 | 选择"视频创作""3D 美术创作"等内置模板，套用后在看板上使用模板列，标注自审与交叉审核。 |
| 项目管理员（project_admin） | 执行需要 `RequireApproval` 的迁移；管理自定义模板与启用开关。 |

## 3. 需求

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-09-01 | 状态流转工作流的实体: `Workflow`（tenant_id、states、transitions、default_initial_state、version）、`WorkflowState`、`Transition`、`WorkflowInstance`、`StateChange`（SRS-CANVAS §2.4；domain-workflow-spec §2） | P0（未标注） | 🟡 | `crates/domain-workflow/src/lib.rs:162-243`；字段与 spec 的 `WorkflowDefinition` 命名不一致（见 §8） |
| FR-09-02 | 迁移严格表查找、终态不可迁出、迁移写入历史（actor + at）、跨租户拒绝 | P0（未标注） | 🟡 | `lib.rs:401-420`（check_transition）、`lib.rs:609-665`（transition）、`lib.rs:614-622`（跨租户）、`lib.rs:656-661`（history）；单测 `lib.rs:855-1478`；仅内存实现，无生产调用方（`crates/` 内无 `InMemoryWorkflowService` 引用） |
| FR-09-03 | Workflow 持久化（PostgreSQL repository）与 REST 接口 | P0（未标注） | ⬜ | `lib.rs:377-393` 仅 trait；`lib.rs:696-730` 仅 `InMemoryWorkflowRepository`；`crates/star-api-rest/src` 无 workflow 路由（grep 无命中）；`crates/` 无 `workflow_instance` 等建表语句 |
| FR-09-04 | `/workflow` 页面展示项目的状态流转工作流（状态流、迁移表、default 标记） | P0（未标注） | 🟡 | `frontend/src/app/workflow/page.tsx:20-95`；数据源 `store.workflows` 从未写入（`frontend/src/lib/store.ts:409`，无 setter） |
| FR-09-05 | 系统默认三态（TODO / IN_PROGRESS / DONE，只读）由 system default 提供 | P0（未标注） | ⬜ | domain-workflow-spec §1 行 17、§4 行 113 `get_system_default`；`crates/domain-workflow` 无对应符号。`crates/star-workflow/src/lib.rs:171-195` 的 `default_workflow()` 是另一个 crate 的 5 态任务流，无前端调用方 |
| FR-09-06 | 命令端口扩展: `update_workflow`、`delete_workflow`、`add_state`、`add_transition`；查询端口 `list_states`、`list_transitions`、`validate_transition` | P1（未标注） | ⬜ | domain-workflow-spec §4 行 70–113；代码只有 `WorkflowCommandPort { create_workflow, start_instance, transition }`（`lib.rs:337-359`）与 `WorkflowQueryPort { get, list_by_tenant }`（`lib.rs:362-374`）；校验为自由函数 `check_transition`（`lib.rs:401`） |
| FR-09-07 | 状态流转工作流的 4 套内置模板（编程瀑布式、文学创作、视频创作、3D 美术），每套为 `CreateWorkflowCommand` 参数集 | P0（未标注） | ⬜ | workflow-templates-spec §6（行 97–159）；`crates/` 无 Programming / Literary / Video / Art3D 符号（grep 无命中） |
| FR-09-08 | `/workflow` 模板选择器（4 张卡片 + 流程图预览，复用 `visualize.rs`） | P1（未标注） | ⬜ | workflow-templates-spec §7.1（行 163–165）；`frontend/src/app/workflow/page.tsx` 无选择器；`crates/domain-workflow/src/visualize.rs:55-168` 存在可视化能力但无前端消费者 |
| FR-09-09 | Worktree 自审 / 交叉审核标注面板与状态徽标 | P1（未标注） | ⬜ | workflow-templates-spec §7.2（行 167–174）；无 Worktree 详情面板；`(app)/review/page.tsx:37` 为前端 mock 列表，非该面板 |
| FR-09-10 | `Transition.required_review_kind` 与 `ReviewKind`（SelfReview / CrossReview / AgentAssistedReview）、Guard 三条求值规则 | P0（未标注） | 🟡 | 后端 ⬜：`crates/` 无 `ReviewKind`、`ReviewRecord`、`DesignArtifact`（grep 无命中），`Transition` 缺该字段（`lib.rs:200-212`）。前端 🟡：`frontend/src/lib/workitem-guard.ts:64` `checkAllArtifactsApproved` 为纯函数，未接入任何迁移求值 |
| FR-09-11 | 模板套用领域事件 `star.events.workflow.template.applied.v1` 与 `WorkflowTemplateCatalog` 只读目录 | P1（未标注） | ⬜ | workflow-templates-spec §8（行 178–182）、§9（行 184–205）；grep 无 `template.applied` 与 `WorkflowTemplateCatalog` |
| FR-09-12 | W1 流程节点类型与编辑入口: 4 类流程节点 element kind、Flow Edge、工具栏"+ Flow"入口、双击展开详情 | P0 | 🟡 | 节点图编辑器存在于 `frontend/src/components/agent-workflow/WorkflowBuilder.tsx:133`（画布"流程"标签页 `canvas/[id]/page.tsx:235-239`），其节点与连线模型为 `lib/agent-workflow/types.ts:17-57`，并非 `workflow_*_node` element kind；无 "+ Flow" 入口（`frontend/src` 无命中） |
| FR-09-13 | W2 触发: 手动触发（▶ 运行）、定时触发（复用 `schedule_cron`）、Webhook 触发、画布事件触发 | P0 | 🟡 | 定时触发常量 `frontend/src/types/ids.ts:988` 与图标 `frontend/src/app/automation/page.tsx:13`；后端定时规则 REST 在 `crates/star-api-rest/src/group_api/schedule_rules.rs:339,343`（Run 作用域，前端无调用方，`frontend/src` 无 `automation/schedule-rules` 引用）；手动、Webhook、canvas_event 无代码（`TRIGGER_ICON` 键见 `automation/page.tsx:9-16`） |
| FR-09-14 | W3 动作节点: 复用 6 种 `AutomationActionKind`、HTTP 请求与数据转换、顺序与并行编排 | P0 | 🟡 | 前端 6 种 `frontend/src/types/ids.ts:991-993`，只读展示 `automation/page.tsx:18-25、88-98`；后端 `ActionType`（`crates/domain-automation/src/lib.rs:190-200`）与 `RuleExecutor`（`lib.rs:774-776`，`InMemoryAutomationService` 实现于 `lib.rs:1100`）为内存实现且枚举不同；无 http_request、transform_data |
| FR-09-15 | W4 分支与汇合: IF、Switch（N 个 case + default）、Merge（race / all） | P0 | 🟡 | 编辑器节点定义 `frontend/src/lib/agent-workflow/workflowLib.ts:63-110`（if-else、switch 静态输出 `rule-1` 与 `fallback` 见 `:88-99`，merge 固定 2 输入见 `:101-110`）；规则面板 `WorkflowBuilder.tsx:809-812`；无执行引擎，无动态分支数 |
| FR-09-16 | W5 循环与批处理: Loop 节点、并发限制（上限 20） | P1 | 🟡 | for-loop 节点 `workflowLib.ts:76-86` 与面板 `WorkflowBuilder.tsx:766-808`；无并发配置与执行 |
| FR-09-17 | W6 变量与表达式: 节点间数据映射 `{{node.<id>.output.<field>}}`、流程变量、CEL 表达式 | P0 | ⬜ | `condition_expr` 仅以文本显示（`automation/page.tsx:83-84`）；`crates/domain-automation/src` 无 CEL 求值（grep 无 `cel` 命中）；`AutomationCondition::evaluate` 为 `ConditionOperator` 枚举比较（`lib.rs:165-177`、`:332`） |
| FR-09-18 | W7 子流程引用节点与流程模板库（"PR 合并后通知"等 3–5 个预置） | P1 | ⬜ | SRS-CANVAS §4.7（FR-W7.1 行 591–606、FR-W7.2 行 607–620）；无 `workflow_subworkflow_node`、`flow_template` 符号 |
| FR-09-19 | W8 错误处理: 节点级重试策略、`on_error` 分支、流程失败通知 | P0 | ⬜ | SRS-CANVAS §4.8；grep `retry_policy` 仅命中 `crates/domain-automation/src/schedule.rs:377` 与 batch 等无关模块，无 Flow 节点重试；无 `on_error` |
| FR-09-20 | W9 执行历史（`execution_history`、`execution_step`，append-only）、从失败节点重跑、单节点调试面板 | P0 | ⬜ | SRS-CANVAS §4.9；grep `execution_history|execution_step` 无前端或 crate 命中 |
| FR-09-21 | W10 启用开关、版本历史（SCD Type 2）、回滚 | P0 | 🟡 | 启用状态只读展示 `automation/page.tsx:55、64-68`，无开关；`crates/domain-workflow/src/lib.rs:177-178` 有 `version` 字段，版本递增由调用方完成（测试 `lib.rs:1306-1313` 手动 `version += 1`）；无版本表 |
| FR-09-22 | W11 流程与标签绑定（`tag_binding_expr`）、派生任务卡规则、标签变更传播、AND/OR/NOT 表达式 | P0 | ⬜ | SRS-CANVAS §4.11；grep `tag_binding` 无命中 |
| FR-09-23 | W12 派生任务卡默认落 Backlog、Sprint 联动、人工覆盖保护、"Flow Tags" 标签页 | P0 | ⬜ | SRS-CANVAS §4.12；`/automation` 无 Flow Tags 标签页（`automation/page.tsx` 仅单一 `SectionTitle "Rules"`，`:50`）；SRS-CANVAS 行 892、1306 |
| FR-09-24 | W13 执行状态与 `WorkItem.status` 的可选映射 | P1 | ⬜ | SRS-CANVAS §4.13；无映射配置代码 |
| FR-09-25 | W14 默认模板选择器（3 套：AAA 游戏资产管线、编程 spec 式、superpowers 式）、套用即实例化节点与边、另存为自定义模板、模板库管理 | P1 | ⬜ | SRS-CANVAS §4.14（行 951–1067）；API-WF-06/07（BD-CANVAS §5.1 行 660–661）无后端路由 |
| FR-09-26 | W15 Flow 与 LangGraph 智能控制: 共享底部聊天栏、NL 生成 Flow、`routing_mode: dynamic_agent`、聊天会话集成、产物复用 Flow schema | P0 | ⬜ | SRS-CANVAS §4.15；前端无群组聊天栏（`frontend/src` 无 GroupShell / ChatBar 命中）；后端 `crates/star-api-rest/src/group_api/scoped_chat.rs` 存在范围聊天工作流 trait，但不产生 Flow 草稿 |
| FR-09-27 | WT-1 模板数据模型与持久化（WorkflowTemplate、17 列结构、版本与 SCD Type 2、软删除） | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 81、213–286；grep `WorkflowTemplate` 在 `frontend/src` 与 `crates` 无命中 |
| FR-09-28 | WT-2 17 列结构（Todo / 作废 / P1–P9 与 P6.1–P6.4 / 审核 / 完成）与不可违反规则 | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 82、287–300；BD-WORKFLOW-TEMPLATE §2.5（行 305–330）。现有项目弹窗的默认是 12 段 IPA 阶段（`CreateProjectDialog.tsx:33-46`），与 17 列不一致（见 §8） |
| FR-09-29 | WT-3 模板 CRUD 界面（`/workflow-templates`，创建、查看、重命名、删除、复制） | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 83、301–312；`frontend/src/app` 无 `workflow-templates` 路由（ls-tree 无命中）；`frontend/src/components/workflow-templates/` 不存在 |
| FR-09-30 | WT-4 模板应用到看板（选择模板 → 模板化看板 → WorkItem.status 与模板列 1:1 映射） | P1（未标注） | 🟡 | 选择界面存在：`frontend/src/components/project/CreateProjectDialog.tsx:48-70`（3 个 `KANBAN_TEMPLATES`）、`:316-340`（卡片与 `data-testid="create-project-template-*"`）、`:122-133`（提交 `kanbanTemplate`、`initialPhaseCount`）；但 `frontend/src/lib/store.ts:501-530`（`addProject`）只读取 name、key、visibility、owner_id、member_count，未创建看板，模板选择被丢弃；路由 `/workflow-templates/{id}/board` 不存在 |
| FR-09-31 | WT-5 模板列可编辑边界（增删列、改列宽、重排、重命名；Todo 不可删，作废列宽固定，顺序约束） | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 85、323–336；无对应约束代码 |
| FR-09-32 | WT-6 模板变更的实时同步（SSE 与 5 秒轮询兜底） | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 86、337–344；BD-WORKFLOW-TEMPLATE §5.1 行 702 SSE 端点无后端路由 |
| FR-09-33 | WT-7 写锁（5 分钟超时）、乐观锁 version、回滚到历史版本 | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 87、345–352；无写锁代码 |
| FR-09-34 | WT-8 用户菜单第 5 个入口 `/workflow-templates` 与当前激活模板指示 | P1（未标注） | ⬜ | SRS-WORKFLOW-TEMPLATE-001 行 88、353–358；`frontend/src/components/UserMenu.tsx` 无相关入口 |
| FR-09-35 | 既有 `/automation` 规则只读视图（基线，非新增需求） | P0（基线） | 🟡 | `frontend/src/app/automation/page.tsx:27-105`；数据源 `store.automationRules`（`frontend/src/lib/store.ts:437` 初值 `[]`，无 setter）；后端无 AutomationRule REST（`crates/star-api-rest/src` 无命中） |

## 4. 设计

### 4.1 概念模型

| 概念 | 定义 | 所在层 | 状态 |
|---|---|---|---|
| Workflow（状态流转工作流） | 一个 Project 的 WorkItem 状态机，含 states、transitions、guard | domain-workflow `lib.rs:162-212` | 🟡 |
| WorkflowInstance | 绑定某个 WorkItem 的运行实例，持有 current_state 与历史 | `lib.rs:215-243` | 🟡 |
| Guard | RequireRole / RequireValidation / RequireApproval 三态守卫 | `lib.rs:137-144` | 🟡 |
| ReviewKind / required_review_kind | 审核类型，与 Guard::RequireApproval 组合 | workflow-templates-spec §4 | ⬜ |
| Automation Flow（Flow） | `automation_flow` 节点图，含 flow_node、flow_edge、版本与执行历史 | SRS-CANVAS §4.1–4.10；BD-CANVAS §4.1 表（`automation_flow`、`flow_node`、`flow_edge`、`execution_history`、`execution_step` 等 8 表） | ⬜ |
| AutomationRule | 触发器 + 条件 + 动作列表 | domain-automation `lib.rs:394-`；frontend `ids.ts:995-1010` | 🟡 |
| 节点图编辑器草稿 | 客户端的 WorkflowNode / WorkflowWire，持久化在浏览器 | `lib/agent-workflow/types.ts:44-57` | 🟡 |
| WorkflowTemplate（看板模板） | 17 列结构的模板实体，支持版本与软删除 | SRS-WORKFLOW-TEMPLATE WT-1/WT-2 | ⬜ |

三个模板目录分别属于不同聚合，互不共享实体：状态流转模板产出 `CreateWorkflowCommand` 参数集；Flow 模板产出 `automation_flow` 节点与边；看板模板产出列结构。

### 4.2 数据模型

**已实现的数据形态**

- 领域层 `Workflow`（`crates/domain-workflow/src/lib.rs:162-183`）、`WorkflowState`（`:186-198`，含 `StateCategory` `:90-97`）、`Transition`（`:200-212`，字段 `id/from/to/trigger/guard`）、`WorkflowInstance`（`:215-229`）、`StateChange`（`:232-243`）。仅存在于内存，`InMemoryWorkflowRepository`（`:696-730`）是唯一仓库实现。
- 领域层 `AutomationRule`（`crates/domain-automation/src/lib.rs:394-`）含 `trigger`、`conditions`、`actions`；内存实现 `InMemoryAutomationService`（`:1100`）。
- 前端类型: `Workflow`（`frontend/src/types/ids.ts:402-410`）、`WorkflowState`（`:386-393`，字段 `kind`、`category`、`position`）、`WorkflowTransition`（`:395-400`，字段 `trigger`、`guard` 为字符串）、`AutomationRule`（`:995-1010`）、`AutomationTriggerKind`（`:986-989`，6 值）、`AutomationActionKind`（`:991-993`，6 值）。
- 前端草稿: `SavedWorkflow`（`frontend/src/lib/store.ts:50`）存放于 `savedWorkflows`（`:192`），由 persist 中间件写入 `localStorage` 键 `star-store:v1`（`store.ts:1671-1675`，存储适配器 `:106-131`）。

**未实现的数据形态（设计）**

- BD-CANVAS-WORKFLOW-001 §4.1 规划 8 表：`automation_flow`（Master）、`automation_flow_versions`（Master，SCD Type 2）、`flow_node`、`flow_edge`（Master）、`execution_history`、`execution_step`（Transaction，append-only）、`flow_template`（Master）、`chat_session`（Transaction）。
- BD-WORKFLOW-TEMPLATE-001 §4.1 规划 7 表（看板模板）。
- 无任何 DDL 或迁移文件在 `637ed19d` 中出现（grep `workflow_instance|workflow_state` 仅命中 star-saga 中的字符串引用）。

### 4.3 接口 routes, APIs, ports

**REST（设计，未实现）**

| 设计 ID | 方法与路径 | 来源 | 实现 |
|---|---|---|---|
| API-WF-01 | `/v1/collaboration/flows` CRUD | BD-CANVAS §5.1（行 655） | ⬜ |
| API-WF-02 | `/v1/collaboration/flows/{id}/nodes`、`/edges` | BD-CANVAS §5.1（行 656） | ⬜ |
| API-WF-03 | `/v1/collaboration/flows/{id}/executions` | BD-CANVAS §5.1（行 657） | ⬜ |
| API-WF-04 | `/v1/collaboration/flows/{id}/executions/{exec_id}/resume` | BD-CANVAS §5.1（行 658） | ⬜ |
| API-WF-05 | `/v1/collaboration/flows/{id}/webhook/{token}` | BD-CANVAS §5.1（行 659） | ⬜ |
| API-WF-06 | `/v1/collaboration/flow-templates` CRUD | BD-CANVAS §5.1（行 660） | ⬜ |
| API-WF-07 | `/v1/collaboration/flow-templates/{id}/instantiate` | BD-CANVAS §5.1（行 661） | ⬜ |
| API-WF-08 | `/v1/collaboration/chat-sessions/{id}/messages` | BD-CANVAS §5.1（行 662） | ⬜ |
| WT-API | `/api/workflow-templates` 列表、详情、创建、更新、删除、锁、SSE | BD-WORKFLOW-TEMPLATE §5.1（行 694–702） | ⬜ |

**已存在的后端路由（相邻，非本功能）**

- `GET/POST /api/v1/projects/{project_id}/engineering-runs/{run_id}/automation/schedule-rules` 与 `GET/PUT .../schedule-rules/{rule_id}`，定义于 `crates/star-api-rest/src/group_api/schedule_rules.rs:336-348`，经 `group_api.rs:821` 合并。该接口为 Run 作用域定时规则，`docs/requirements` 中的 SRS-AUTOMATION-SCHEDULE-API-001 为其规格；它不是 Flow 接口。

**领域端口**

- `WorkflowCommandPort`：`create_workflow`、`start_instance`、`transition`（`crates/domain-workflow/src/lib.rs:337-359`）。
- `WorkflowQueryPort`：`get`、`list_by_tenant`（`:362-374`）。
- `WorkflowRepository`（`:376-393`）：仅内存实现。
- `RuleExecutor`（`crates/domain-automation/src/lib.rs:774-776`）：唯一实现 `InMemoryAutomationService`（`:1100`）。

**前端 store 动作**

- `saveWorkflowDraft(name, nodes, wires)`、`deleteSavedWorkflow(id)`（`frontend/src/lib/store.ts:193-194`，实现 `:451-463`，同名覆盖）。
- `addProject(input)`（签名 `store.ts:245-256` 含 `kanbanTemplate`、`initialPhaseCount`；实现 `:501-530` 不使用这两个字段）。

### 4.4 状态机与关键流程

**状态流转（已实现，领域层）**

`transition(cmd, actor)`（`crates/domain-workflow/src/lib.rs:609-665`）的顺序：

1. `ensure_cross_tenant(actor, cmd.tenant_id)`（`:614`）。
2. 读取 WorkflowInstance 与 Workflow；校验 Workflow 租户（`:617-622`）。
3. `check_transition(wf, current_state, to)`（`:624`，`:401-420`）：终态不可迁出（INV-WF-04），不在转换表则拒绝（INV-WF-03）。
4. 查找对应 Transition 的 guard：`RequireRole(role)` 需 `actor.has_role(role)`（`:633-640`）；`RequireApproval` 目前硬编码要求 `project_admin` 角色（`:641-648`）；`RequireValidation` 为空操作（`:649-651`）。
5. 追加 `StateChange`（`:656-661`），更新 `current_state`，运行不变量 `run_invariants`（`:663`），持久化并写入内存映射（`:664-665`）。

**设计态流程（未实现）**

- workflow-templates-spec §6 的四套模板状态机，例如编程模板 `DETAILED_DESIGN → IN_PROGRESS` 需要 `RequireApproval` 且 `required_review_kind = None`（DesignArtifact 语义）。
- workflow-templates-spec §5 的 Guard 求值三条规则（DesignArtifact 空集合放行、ReviewRecord 全部满足才放行、Agent-Assisted Reject 不阻断）。
- SRS-CANVAS §4.9 的 Execution 生命周期（运行、失败、从失败节点重跑）。

### 4.5 权限与安全

- 角色检查基于 `ActorContext`：`has_role`（`crates/domain-workflow/src/lib.rs:634、643`）。
- 跨租户拒绝：`ensure_cross_tenant`（`:614`）与 `CrossTenantDenied`（`:618-621`）。
- 迁移历史记录 actor 与时间（`:656-661`，INV-WF-05）。
- 前端无权限判断：`/workflow` 与 `/automation` 不做任何租户或角色检查（`frontend/src/app/workflow/page.tsx`、`frontend/src/app/automation/page.tsx` 无 `useAuth` 或等价调用）。
- 设计要求（未实现）: Webhook 入口使用 token 校验（BD-CANVAS §8.2）；模板内置项只读，后端 403 双重保护（SCR-WF-02）；写操作需 `workflow:create` 权限（workflow-templates-spec §10）。

### 4.6 依赖关系

- `crates/domain-workflow` 使用 `ActorContext` 与 `TenantId`（`lib.rs:343`、`:614`）；`visualize.rs` 为独立模块，提供 SVG、Mermaid、DOT 输出（`crates/domain-workflow/src/visualize.rs:55-168`），无调用方。
- `crates/star-workflow` 为独立的 Jira 风格任务流 crate（`Cargo.toml:145-150` 注释与成员声明），无其他 crate 依赖，前端无调用。
- `frontend/src/app/workflow/page.tsx` 依赖 `useStore`、`StatusPill`、lucide 图标；`frontend/src/app/automation/page.tsx` 依赖 `useStore`、`PageHeader`、`SectionTitle`、`Stat`、`useTranslation`。
- `WorkflowBuilder`（`frontend/src/components/agent-workflow/WorkflowBuilder.tsx:25`）依赖 `useStore`、`lib/agent-workflow/workflowLib.ts`、`lib/agent-workflow/types.ts`。
- Kanban 创建表单读取 `store.workflows` 作为"关联工作流"下拉（`frontend/src/app/(app)/kanban/page.tsx:97、312-315`），因数据源为空，仅剩"不关联工作流"选项。
- `BranchWorkspace` 链接到 `/workflow`（`frontend/src/components/run/BranchWorkspace.tsx:156-163`），列表数据同样为空（`:164-165`）。

### 4.7 页面设计

#### 4.7.1 /workflow

**文件**: `frontend/src/app/workflow/page.tsx`

**布局**

- 根布局 `frontend/src/app/layout.tsx:76-94`: 左侧 `Sidebar`（`:78`）、主列（`:80-94`）、`MobileHeader`（`:92`）、`children`（`:93`）、`MobileBottomNav`（`:99`）。
- 该页面位于根路由，不属于 `(app)` 路由组，因此不经过 `(app)/layout.tsx:20、49` 的 `AppShell`。结论: 无顶栏（AppHeader）、无右侧栏、无底栏。
- 页面自身头部为 `<header>`（`workflow/page.tsx:25-28`），标题"低代码流程"。

**组件树**

- `WorkflowPage`（`workflow/page.tsx:20`）
  - `header`（`:25-28`）
  - 空状态段落（`:35-37`）
  - 工作流卡片列表（`:39-95`）：状态流（`:55-68`）、迁移表（`:72-92`，`StatusPill` 用于 from/to）
  - `StatusPill`（import `:5`，来自 `components/StatusPill`）

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | `store.workflows.length > 0` | 工作流卡片、状态流、迁移表（`:39-95`） | ❓ 不可达：`store.workflows` 无写入路径（`frontend/src/lib/store.ts:409` 初值 `[]`，全仓无 setter）；需要后端接入或 seed 写入才能验证 |
| 加载 | 异步拉取 | 无 | ⬜ 无加载态，同步读 store |
| 空 | `workflows.length === 0` | "暂无工作流定义"（`:36`） | 🟡 当前唯一可见状态，但同一文案覆盖"未接入"与"确实为空" |
| 错误 | 拉取失败 | 无 | ⬜ |
| 未授权 | 无项目权限 | 无 | ⬜ |
| 未接入 | 后端 workflow 接口未实现 | 无专门提示 | ⬜ 与空态混用 |
| 禁用 | 无 | 无 | 不适用 |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 查看状态流 | 有数据 | 只读展示 | store | 🟡 代码存在，不可达 |
| 查看迁移表 | 有数据 | 只读展示 from、to 类别、trigger、guard（`:72-92`） | store | 🟡 |
| 查看 default 标记 | `is_default` 为真 | 显示 "default" 徽标（`:47`） | store | 🟡 |
| 新建工作流 / 编辑状态 / 迁移 | — | 无按钮 | 无 | ⬜（FR-09-06） |
| 应用模板 | — | 无选择器 | 无 | ⬜（FR-09-08） |
| 进入画布流程编辑 | — | 无链接 | 无 | ⬜ |

**数据**

- 来源: `useStore((s) => s.workflows)`（`workflow/page.tsx:21`）。
- 字段: `id`、`name`、`project_id`、`is_default`、`states[]`（`id`、`name`、`kind`、`category`）、`transitions[]`（`from_state_id`、`to_state_id`、`trigger`、`guard`）。
- 无 REST 调用，无 MSW mock handler（`frontend/src/mocks` 无 workflow 命中）。

**与文档设计的差距**

- FR-09-04：数据源未接入，页面无法展示任何真实工作流。
- FR-09-06：页面无编辑能力，spec §4 的命令端口缺失。
- FR-09-05：system default 三态未展示。
- FR-09-07、FR-09-08：模板选择器与 4 套内置模板缺失（workflow-templates-spec §7.1）。
- FR-09-10：`RequireApproval` 与 `required_review_kind` 未在迁移表中展示。
- FR-09-21：版本信息未展示。

#### 4.7.2 /automation

**文件**: `frontend/src/app/automation/page.tsx`

**布局**

- 同 4.7.1: 根布局 `frontend/src/app/layout.tsx:76-94`，不经 `AppShell`。
- 页面头部为 `PageHeader`（`automation/page.tsx:35-41`，`track="E"`，`count={rules.length}`）。
- 无标签页（整页仅一个 `SectionTitle "Rules"`，`:50`）。

**组件树**

- `AutomationPage`（`automation/page.tsx:27`）
  - `PageHeader`（`:35-41`，来自 `components/PageHeader`）
  - 统计行 `Stat` × 4（`:43-48`）：Rules、Enabled、Disabled、Fired (24h)
  - `SectionTitle`（`:50`）
  - 规则卡片列表（`:52-104`）：图标映射 `TRIGGER_ICON`（`:9-16`）与 `ACTION_ICON`（`:18-25`）；启用或禁用徽标（`:64-68`）；触发、条件（`:83-84`）、动作（`:88-98`）三栏

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | `automationRules.length > 0` | 统计与规则卡片 | ❓ 不可达：`store.automationRules` 初值 `[]`（`frontend/src/lib/store.ts:437`），无 setter |
| 加载 | 异步拉取 | 无 | ⬜ |
| 空 | 列表为空 | 统计均为 0，规则区无内容，无文案（`:50-52`） | 🟡 呈现为空白区，没有空态说明 |
| 错误 | 拉取失败 | 无 | ⬜ |
| 未授权 | 无项目权限 | 无 | ⬜ |
| 未接入 | 后端 AutomationRule 接口未实现 | 无提示 | ⬜ |
| 禁用 | `enabled === false` | 卡片透明度 60%（`:55`）与"disabled"徽标（`:67`） | ✅ 仅展示，有代码 |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 查看规则 | 有数据 | 只读 | store | 🟡 |
| 切换启用 | — | 无控件 | 无 | ⬜（FR-09-21） |
| 手动运行 | — | 无"▶ 运行"按钮；`TRIGGER_ICON` 无 `manual` 键（`:9-16`） | 无 | ⬜（FR-09-13） |
| 新建规则 / Flow | — | 无 | 无 | ⬜ |
| 切换到 Flow 标签页 | — | 无标签页 | 无 | ⬜（FR-09-12、FR-09-23） |
| 切换到 Flow Tags 标签页 | — | 无标签页 | 无 | ⬜（FR-09-23） |

**数据**

- 来源: `useStore((s) => s.automationRules)`（`automation/page.tsx:29`）。
- 字段: `id`、`name`、`enabled`、`trigger_kind`、`trigger_filter`、`condition_expr`、`actions[]`（`kind`、`config`）、`execution_count_24h`、`last_fired_at`（`ids.ts:995-1010`）。
- 无 REST，无 mock handler。

**与文档设计的差距**

- FR-09-13：缺"▶ 运行"手动触发（SRS-CANVAS FR-W2.1，P0；SRS-MULTICA-AUTOPILOT-001 FR-3 同样要求 "Run now"）。
- FR-09-21：缺启用开关（FR-W10.1）。
- FR-09-23：缺 Flow 与 Flow Tags 两个标签页（SRS-CANVAS 行 892、1306；BD-CANVAS §3.2 SCR-WF-04、SCR-WF-05）。
- FR-09-12：缺 Flow 列表与编辑入口（SCR-WF-01）。
- FR-09-14：只读动作展示，动作类型与后端 `ActionType` 不一致（见 §8）。
- FR-09-17：`condition_expr` 无 CEL 求值。

#### 4.7.3 /workflow-templates（设计存在，路由未实现）

**文件**: 无。`frontend/src/app/workflow-templates` 与 `frontend/src/components/workflow-templates/` 在 `637ed19d` 中不存在。

**布局**: 未实现，设计要求为 WT-3 列表页与 WT-4 模板化看板（SRS-WORKFLOW-TEMPLATE §8.2，行 495–503，四个组件 `TemplateList`、`TemplateDetail`、`TemplateBoard`、`TemplateColumnEditor`）。

**组件树**: 无。

**状态矩阵**: 正常、加载、空、错误、未授权、未接入、禁用均为 ⬜。

**交互**: 创建、查看、重命名、删除、复制、应用、列编辑均为 ⬜（FR-09-29、FR-09-30、FR-09-31）。

**数据**: 设计为 `GET/POST/PATCH/DELETE /api/workflow-templates`（BD-WORKFLOW-TEMPLATE §5.1），无后端或 mock 实现。

**与文档设计的差距**: 全部 WT-1..WT-8 条目（FR-09-27 至 FR-09-34）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| domain-workflow（状态机与守卫） | `crates/domain-workflow/src/lib.rs` | 🟡 | 实体与端口 `:162-393`，迁移 `:401-665`，内存仓库 `:696-730`，单测 `:790-1478` |
| domain-workflow 可视化 | `crates/domain-workflow/src/visualize.rs` | 🟡 | `:55-168` 输出 SVG、Mermaid、DOT；测试 `:202`、`:250`、`:259`；无前端调用 |
| star-workflow（Jira 风格任务流） | `crates/star-workflow/src/lib.rs` | 🟡 | `default_workflow()` `:171-195`，测试 `:606`、`:619`；无调用方 |
| domain-automation（规则与执行器） | `crates/domain-automation/src/lib.rs` | 🟡 | `TriggerType` `:132-140`、`ActionType` `:190-200`、`AutomationRule` `:394-`、`RuleExecutor` `:774-776`、内存实现 `:1100`；测试文件 1 处 `#[cfg(test)]`；无 REST 接口 |
| Run 作用域定时规则 REST | `crates/star-api-rest/src/group_api/schedule_rules.rs` | 🟡 | 路由 `:336-348`，合并 `group_api.rs:821`；前端无调用方 |
| 群组聊天工作流 trait | `crates/star-api-rest/src/group_api/scoped_chat.rs` | 🟡 | 存在 trait 与路由，不产生 Flow 草稿（与 FR-09-26 无关联） |
| `/workflow` 页面 | `frontend/src/app/workflow/page.tsx` | 🟡 | `:20-95`，数据源为空 |
| `/automation` 页面 | `frontend/src/app/automation/page.tsx` | 🟡 | `:27-105`，数据源为空，无标签页与操作 |
| 导航项 | `frontend/src/lib/nav/registry.ts` | ✅ | `workflow` `:376-385`、`automation` `:501-509` |
| 重定向规则 | `frontend/src/lib/redirects.ts` | ✅ | `:75-77` 注释确认 `/workflow` 无重定向；`/automation` 无重定向 |
| Store 切片 | `frontend/src/lib/store.ts` | 🟡 | `workflows` `:153、409`，`automationRules` `:180、437`，`savedWorkflows` `:192-194、448-463`，持久化 `:1671-1675` |
| 节点图编辑器 | `frontend/src/components/agent-workflow/WorkflowBuilder.tsx` | 🟡 | 画布"流程"标签页 `canvas/[id]/page.tsx:235-239`，Agent 视图 `AgentViewPage.tsx:393-396`；草稿保存加载删除 `:143-163`、`:375-401`；无运行或校验 |
| 节点类型库 | `frontend/src/lib/agent-workflow/workflowLib.ts` | 🟡 | `CONTROL_NODE_LIBRARY` `:53-134`；switch 静态输出 `:88-99`；merge 固定 2 输入 `:101-110` |
| 节点类型与连线模型 | `frontend/src/lib/agent-workflow/types.ts` | 🟡 | `:17-57` |
| 前端类型 | `frontend/src/types/ids.ts` | 🟡 | `Workflow` `:386-410`，`AutomationRule` `:986-1010` |
| 审批守卫纯函数 | `frontend/src/lib/workitem-guard.ts` | 🟡 | `checkAllArtifactsApproved` `:64`；测试 `workitem-guard.test.ts` |
| 项目模板选择 | `frontend/src/components/project/CreateProjectDialog.tsx` | 🟡 | 选择卡片 `:316-340`，提交 `:122-133`；store 不应用（见 FR-09-30） |
| 审核列表（mock） | `frontend/src/app/(app)/review/page.tsx` | 🟡 | `ReviewKind` `:20`，`ReviewRecord` `:22-34`，mock `:36-37`；属于审核域，不计入本功能 |
| 看板创建表单的工作流下拉 | `frontend/src/app/(app)/kanban/page.tsx` | 🟡 | `:97`、`:312-315`，数据源为空 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| `store.workflows` 与 `store.automationRules` 无写入路径（`store.ts:409、437`） | `/workflow`、`/automation`、看板下拉、BranchWorkspace 全部只显示空态，无法验证任何展示逻辑 | 增加 REST 拉取（或 seed 写入）并为两个切片增加 setter；同时为空态与未接入态给出不同文案 |
| 无 workflow REST 路由（`crates/star-api-rest/src` 无命中） | 前端无法读写状态流转工作流 | 按 BD-CANVAS §5.1 与 domain-workflow-spec §4 实现 CRUD，先覆盖 `create_workflow`、`get`、`list_by_tenant`、`transition` |
| Workflow 仅内存仓库（`lib.rs:696-730`），无 PostgreSQL 实现 | 重启丢失数据，不满足持久化要求 | 实现 `WorkflowRepository` 的 PostgreSQL 版本与迁移脚本，并覆盖租户隔离 |
| domain-workflow-spec §4 的命令与查询端口缺失（`lib.rs:337-374`） | 无法编辑状态与迁移，无系统默认三态 | 补 `update_workflow`、`delete_workflow`、`add_state`、`add_transition`、`get_system_default` |
| `RequireApproval` 仅检查 `project_admin` 角色（`lib.rs:641-648`），与 `required_review_kind` 设计不一致 | 审批语义与 workflow-templates-spec §5 不符 | 先在 domain 增加 `ReviewKind` 与 `Transition.required_review_kind`，再接入 DesignArtifact 与 ReviewRecord 查询 |
| `RequireValidation` 为空操作（`lib.rs:649-651`） | 构建或测试门禁不生效 | 接入 ValidationResult 查询，按 `Guard::RequireValidation(name)` 求值 |
| workitem-guard 纯函数未接入迁移（`frontend/src/lib/workitem-guard.ts:64`） | 设计审批门禁不生效 | 在迁移求值路径调用 `checkAllArtifactsApproved` 或其后端等价实现 |
| 4 套内置状态流模板缺失（workflow-templates-spec §6） | 新建 Project 只能从零拼状态机 | 增加 `WorkflowTemplateCatalog`，以只读 seed 数据提供 4 套模板 |
| `/workflow` 模板选择器缺失（workflow-templates-spec §7.1） | 用户无法选择模板 | 在 `workflow/page.tsx` 增加 4 卡片选择器，复用 `visualize.rs` 做预览 |
| Worktree 自审与交叉审核面板缺失（workflow-templates-spec §7.2） | 审核标注无处操作 | 在 Worktree 详情页增加面板；依赖 ReviewRecord 后端 |
| 模板选择被 store 丢弃（`store.ts:501-530` vs `CreateProjectDialog.tsx:122-133`） | 用户选择的看板模板无效，项目创建后没有看板 | `addProject` 读取 `kanbanTemplate`、`initialPhaseCount` 并创建 Board 列 |
| 17 列模板（WT-2）与 12 段 IPA 默认（`CreateProjectDialog.tsx:33-46`）冲突 | 列结构定义不唯一 | 由 Ulysses 裁定列数与列集合，见 §8 裁决 4 |
| `/workflow-templates` 路由与组件不存在（ls-tree 无命中） | WT-3、WT-6、WT-8 无法验证 | 按 SRS-WORKFLOW-TEMPLATE §8.2 创建组件与路由，先实现 WT-3 列表页 |
| 无 Flow 聚合、节点、边、执行历史（无符号，DDL 无） | W1–W15 整体不可用 | 按 BD-CANVAS §4.2 建 8 表；先落 `automation_flow`、`flow_node`、`flow_edge` 与 W2.1 手动运行 |
| `/automation` 缺 Flow、Flow Tags 标签页与手动运行（`automation/page.tsx:9-16、50`） | SRS-CANVAS 行 892、1306 的入口缺失 | 增加标签页框架与"▶ 运行"按钮（W2.1 为 P0） |
| 前端 `AutomationTriggerKind`、`AutomationActionKind`（`ids.ts:986-993`）与后端 `TriggerType`、`ActionType`（`domain-automation/src/lib.rs:132-200`）不一致 | 前后端规则模型无法互通 | 以 SRS-CANVAS W3.1 指定的 6 种动作为基线，统一后端枚举并提供 REST |
| CEL 表达式无求值（`domain-automation` 无 `cel`） | W6.3 与 W11.5 无法执行 | 选定 CEL 库并接入 `AutomationCondition` |
| 模板目录三套互不关联 | 用户难以区分三类模板 | 按 §8 裁决 3 分别落位，并在 UI 标注所属对象 |
| `redirects.spec.ts:37` 注释称 `/workflow → /projects?tab=workflow`（与 `redirects.ts:75-77` 及 `redirects.spec.ts:105-106` 矛盾） | 文档误导 | 以代码为准，删除该注释中的旧映射 |

## 7. 验收与测试

**已存在的测试**

| 范围 | 位置 | 验证内容 | 未验证内容 |
|---|---|---|---|
| domain-workflow 单测 | `crates/domain-workflow/src/lib.rs:855-1478`（12 个 async 测试，模块 `:790`） | 三态完整迁移、跳状态拒绝、终态不可迁出、非法迁移拒绝、多状态与阻塞、默认初始状态、跨租户拒绝、version 字段（`:1300-1313`，版本由测试手动递增）、历史记录 actor 与时间、RequireRole 拒绝非审批人、创建与查询、多实例独立 | Guard 的 RequireApproval 与 RequireValidation 语义；持久化；REST |
| star-workflow 单测 | `crates/star-workflow/src/lib.rs:606-619` | `default_workflow_has_4_transitions` | 与前端无关 |
| domain-workflow 可视化单测 | `crates/domain-workflow/src/visualize.rs:202-259` | 输出函数（`#[test]` 两个） | 前端渲染 |
| domain-automation 单测 | `crates/domain-automation/src/lib.rs`（`#[cfg(test)]` 1 处） | 内存规则与执行 | REST；CEL |
| workflowPersistence 测试 | `frontend/src/lib/workflowPersistence.test.ts:26-55` | 草稿保存返回 id、同名覆盖、空名不保存、删除、localStorage 持久化 | `/workflow` 与 `/automation` 页面渲染 |
| 审批守卫纯函数测试 | `frontend/src/lib/workitem-guard.test.ts` | `checkAllArtifactsApproved` 的四种 reason | 迁移接入 |
| 重定向 e2e | `frontend/e2e/redirects.spec.ts:105-106` | `/workflow` 无重定向，独立页可达 | 页面内容 |

**未覆盖**

- `/workflow` 与 `/automation` 没有页面级单测或 e2e（`frontend/e2e` 中仅 `redirects.spec.ts` 命中 `/workflow`）。
- 无 REST 集成测试；无 workflow 或 automation 相关 PostgreSQL 测试。
- 四套状态流模板、看板模板、Flow 执行、模板选择器均无测试。
- `frontend/src/lib/store.test.ts` 与 `AgentCanvasView.test.tsx` 中的 `workflow_id: "wf-default"` 仅为夹具字段，不是工作流行为测试。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-CANVAS-WORKFLOW-001.md`（v1.2） | W1–W15 的 54 条 FR、§2.4 术语消歧（行 232–239）、§8.4 联动接口（行 1306）、FR-W12.4 Flow Tags（行 892） | §2.4 行 237 写 Flow 位于"画布（`/board` 或 `/canvas/automation`，待定）"，而 §8.4 行 1306 写在 `/automation` 页 | 采用 §8.4 的 `/automation` 标签页作为 Flow 入口，理由：§8.4 是后期联动清单，且行 237 明确标注"待定"；Flow 编辑器本体的最终位置仍待定 |
| `docs/design/BD-CANVAS-WORKFLOW-001.md` | 屏幕 ID（SCR-WF-01..06，行 290–373）、REST 8 端点（行 651–662）、8 表（§4.1，行 395–414） | 与代码全部不一致（代码无对应路由与表） | 作为设计目标记录，状态按代码标 ⬜ |
| `docs/design/DD-CANVAS-WORKFLOW-001.md` | 模块划分 MOD-WF-001..011（§7.0，行 335–357）、API 内部设计（§9，行 1305–1620）、数据 SQL（§10，行 1622–2000） | 仅参考结构，未逐条核对实现（代码中无对应符号） | 作为设计目标记录，不引用其内部细节作为状态依据 |
| `docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md`（v0.1，2026-09-14） | WT-1..WT-8、33 条 FR（行 213–358）、§8.2 四个组件（行 495–503）、§2.4 三词消歧（行 156–169） | 17 列与 12 段 IPA 默认不一致（`CreateProjectDialog.tsx:33-46`） | 17 列结构保留为 WT-2 的设计目标；12 段默认作为现有项目弹窗的当前行为，两者的冲突列为未裁决项（裁决 4） |
| `docs/design/BD-WORKFLOW-TEMPLATE-001.md` | 17 列预置（§2.5，行 305–330）、6 端点（§5.1，行 692–704）、屏幕（§3.2） | 代码无对应实现 | 设计目标 |
| `docs/design/DD-WORKFLOW-TEMPLATE-001.md` | 接口协议（§7，行 1003–1191）、7 表 DDL（§9，行 1388–1669）、测试（§10，行 1670–1756） | 仅读取目录结构，未逐条核对 | 设计目标；不作为状态依据 |
| `docs/specs/workflow-templates-spec.md`（Draft v0.1，2026-08-31） | 4 套模板状态表（§6，行 97–159）、`required_review_kind` 与 Guard 语义（§4–§5，行 60–95）、模板选择器（§7.1，行 163–165）、审核面板（§7.2，行 167–174）、事件与目录（§8–§9，行 176–205）、验收（§11，行 211–236）、开放问题（§13，行 251–256） | 与代码全部不一致；§3 声称的 `lib.rs:624` 附近 RequireApproval 实现与当前行号不同（当前为 `:641-648`） | 采用 §6 作为模板内容的目标；采用 §4–§5 作为守卫语义的目标；§3 的行号以代码为准 |
| `docs/specs/domain-workflow-spec.md`（Draft v0.1，2026-08-25） | 职责边界（§1，行 15–27）、端口签名（§4，行 63–116）、实体（§2） | 端口方法名与代码不一致（见 FR-09-06）；spec 使用 `WorkflowDefinition`，代码使用 `Workflow`；spec 声明 system default 由本 crate 提供，代码无 | 以代码为现状，spec 为设计目标；命名以代码为准，文档中注明别名 |
| `docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md`（版本 0.67，2026-10-24） | 作为 `schedule_rules.rs:336-348` 的规格来源，用于说明 Run 作用域定时规则的边界 | 不是 Flow 规格；其 `schedule_rules` 无前端调用方 | 仅作为相邻接口的说明，不计入 FR-09 的实现；定时触发 FR-09-13 另行以 SRS-CANVAS W2.2 为准 |
| `docs/requirements/SRS-MULTICA-AUTOPILOT-001.md`（行 128–136） | FR-3 手动触发"Run now"作为旁证 | 该文档属于 Autopilot 范围，触发对象不是 Flow | 手动触发以 SRS-CANVAS FR-W2.1（P0）为准，Autopilot 仅作旁证 |
| `frontend/src/lib/redirects.ts`、`frontend/e2e/redirects.spec.ts` | 重定向规则与测试 | `redirects.spec.ts:37` 注释与其自身第 105 行及 `redirects.ts:75-77` 矛盾 | 以代码为准：`/workflow` 无重定向 |
| `frontend/src/app/(app)/review/page.tsx` | `ReviewKind` 类型与 mock 列表作为旁证 | 为前端 mock，不代表 ReviewRecord 后端 | 归审核域；本文只引用 `ReviewKind` 枚举名称作为一致性参考 |
| 提交 `e946449d`（基线之后）及其后续提交 | 未采用 | 不在 `637ed19d` 基线内，动态 switch 与 merge 端口、validate、simulate 均不在基线代码中（`workflowLib.ts:88-110` 为静态端口，`WorkflowBuilder.tsx` 无 run、validate、simulate 逻辑） | 不计入任何状态 |
| `docs/features/F04-task-board.md`、`F06-unified-canvas.md` | 未读取全文 | 与本功能的看板与画布边界相邻 | 边界以 §1.2 为准；不引用其内容作为证据 |

**裁决记录**

1. **F09 负责的"工作流"范围**：负责状态流转工作流（`/workflow` 与 `domain-workflow`）、Flow（`/automation` 的 Flow 标签页）与三类模板；画布"流程"标签页的 `WorkflowBuilder` 在 §4、§5 描述但归 F06。理由：SRS-CANVAS §2.4 表（行 234–237）按对象区分三种"工作流"，且 `WorkflowBuilder` 的数据模型（`WorkflowNode`、`WorkflowWire`）不是 Flow 聚合。
2. **Flow 编辑器的入口**：采用 `/automation` 的 Flow 标签页，不采用 `/board` 或 `/canvas/automation`。理由：SRS-CANVAS §8.4 行 1306 是最新的联动清单，§2.4 行 237 的路径标注为"待定"。Flow 编辑器本体的最终形态仍待 Ulysses 确认。
3. **三套模板目录分别落位**：状态流转的 4 套模板（workflow-templates-spec §6）放在 `/workflow` 的选择器；Flow 的 3 套模板（SRS-CANVAS W14）放在 Flow 的"+ Flow"入口旁；看板的 17 列模板（WT）放在 `/workflow-templates`。理由：三者分别产出 `CreateWorkflowCommand`、`automation_flow` 节点与边、看板列结构，聚合不同，不能共用一个选择器。
4. **17 列与 12 段 IPA 默认**：未裁决。当前代码的项目弹窗使用 12 段 IPA 阶段（`CreateProjectDialog.tsx:13-17` 注释记载"per 2026-10-01 用户发令"），WT-2 的设计为 17 列（SRS-WORKFLOW-TEMPLATE 2026-09-14）。代码中的 12 段是较新的用户指令，但 17 列是唯一定义模板结构的设计。两者都保留在文档中，列为 §6 差距，等待 Ulysses 裁定列数与列集合。
5. **`RequireApproval` 的语义**：以 workflow-templates-spec §4–§5 的 `required_review_kind` 为目标设计；当前代码的 `project_admin` 检查（`lib.rs:641-648`）作为现状记录为 🟡。理由：代码中没有 ReviewRecord，spec 是唯一完整定义，状态以代码为准。
6. **自动化动作与触发的基线**：以 SRS-CANVAS W3.1 指定的 6 种 `AutomationActionKind`（`ids.ts:991-993`）为基线，后端 `ActionType`（`lib.rs:190-200`）的枚举差异列为 §6 差距。理由：W3.1 文本直接列出了这 6 种，是需求层的显式约定。
7. **定时触发与 Run 作用域规则**：Flow 的 `schedule_cron`（W2.2）与 `schedule_rules` 接口（SRS-AUTOMATION-SCHEDULE-API-001）视为不同对象，不互相代替。理由：后者绑定 Engineering Run、Worktree 与 Task，而 Flow 绑定 `automation_flow`。
8. **手动触发**：以 SRS-CANVAS FR-W2.1（P0）为准；SRS-MULTICA-AUTOPILOT-001 FR-3 作为旁证。理由：后者属于 Autopilot 的独立范围。
9. **Workflow 命名**：采用代码中的 `Workflow`，文档中注明 spec 的 `WorkflowDefinition` 为别名。理由：代码是当前状态，spec 命名仅作为设计参考。
10. **端口方法**：spec §4 列出的方法未实现，状态按代码标 ⬜，设计目标保留。理由：spec 是设计，代码是现状。
11. **`/workflow` 重定向**：以代码为准，`/workflow` 无重定向；`redirects.spec.ts:37` 的旧注释视为过期。理由：`redirects.ts:75-77` 与同一测试文件第 105–106 行一致。
