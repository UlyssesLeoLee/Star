# SRS-CANVAS-WORKFLOW-001

**无限画布 — 专题 3: n8n 式自动化流程 (Automation Flow) + 标签绑定任务卡**

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项 | 内容 |
|---|---|
| 文档 ID | SRS-CANVAS-WORKFLOW-001 |
| 标题 | 无限画布 — 自动化流程 (n8n 式工作流) + 标签绑定任务卡 需求定义书 |
| 所属总册 | `SRS-CANVAS-001` (双核心 → 本文档落地后为 **三核心**) |
| 平行专题 | `SRS-CANVAS-AGENT-001` (专题 1: agent 管理) / `SRS-CANVAS-GAMIFY-001` (专题 2: 游戏化) |
| 触发 issue | ULYS-15 "agent无限画布功能强化" |
| 撰写者 | Sonnet (agent, per Multica ULYS-15 assignment) |
| 版本 | v1.1 |
| 状态 | Draft — 待 5 域 Lead / Ulysses 拍板 |

### 0.2 修订履历 (本 SRS)

| 版本 | 日期 | 变更 | 触发 / 拍板依据 |
|---|---|---|---|
| v1.0 | 2026-09-12 | 初版: W1-W13 子能力 (42 项 FR), 覆盖 n8n 式节点图 + 标签绑定任务卡 + backlog/sprint 联动 | ULYS-15 issue 委托 (人类创建者需求文档委托, 非 Ulysses 直接拍板 — 待 §11 签字栏正式拍板) |
| v1.1 | 2026-09-12 | 新增 W14 (默认工作流模板库: AAA 游戏资产管线 / spec 式 / superpowers 式) + W15 (LangGraph 智能控制 + 底部聊天栏), 共 +12 项 FR (42→54), +6 用户故事 (28→34); 同步更新 §1.3.2/§1.4/§1.5/§7/§8/§9/§10 | ULYS-15 issue 创建者 2 条追评委托: 评论 `01a09566-3f2d-7dc0-81c6-1ff46b89f056` (2026-09-12T11:35:01Z, 三套默认模板 + agent 占位预置) 与评论 `01a09567-5720-7500-a870-9cff243e8102` (2026-09-12T11:36:12Z, LangGraph 智能控制 + 补齐底部聊天栏, 创建者声明为"之前提过的需求") |

### 0.3 撤回记录 (per 守门 #1 禁回溯叙事)

本 SRS **不是** 对 `SRS-CANVAS-001` §1.4 "Miro 12 种 diagram (含 Flowchart / BPMN)" 与 "Miro 通用集成" 砍掉决定的静默重写。明确记录如下反转:

| 撤回时间 | 撤回对象 | 原砍掉理由 (2026-09-10 17:08 JST) | 本次反转理由 (2026-09-12, ULYS-15) | 反转范围 |
|---|---|---|---|---|
| 2026-09-12 (本 SRS v1.0) | `SRS-CANVAS-001` §1.4 "Miro 12 种 diagram" 行中的 **Flowchart** 子项、"Miro 通用集成" 行 | "内容生产用专门工具" / "Star 25 module 已有, 集成超出画布核心" | ULYS-15 人类创建者明确要求"n8n 那种工作流的功能" — 但本 SRS **不是**恢复 Miro 被动绘图型 Flowchart, 而是将其重新定位为**可执行的自动化编排** (execution graph), 是 §4.5.2 已规划的 `automation` module (Rule + Trigger + Condition + Action) 的**图结构扩展**, 而非新增第二套并行绘图引擎 | 仅限"可执行工作流图" 一项; Miro 其余 11 种 diagram / 模板库 / 演示 / 版本 / 移动 / 完整 a11y / 第三方 OAuth 集成等**维持砍掉**, 不在本反转范围内 |

**结论**: 本 SRS 新增的"自动化流程 (Automation Flow)"不是 Miro 式静态 Flowchart 绘图工具的复活, 而是 V0.1 `automation_node` / `AutomationRule` (`frontend/src/types/ids.ts` §25, `frontend/src/app/automation/page.tsx`) 从"1 trigger + 线性 N action" 的**扁平规则**升级为"**N 节点有向图**"的**执行引擎扩展**。

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

按日本 IPA SEC 标准, 为 STAR 平台无限画布定义**专题 3: 自动化流程 (Automation Flow)**, 回应 ULYS-15 issue 的 4 项核心诉求:

1. 无限画布具备 **n8n 式工作流(节点式自动化编排)** 能力
2. 一套自动化流程可按**标签模式**打包为 1 张**任务卡**, 出现在 **Backlog**
3. 该任务卡可**拖拽进 Sprint** (复用现有 Jira 式 Backlog↔Sprint 机制)
4. 面板管理中改变标签后, 任务卡随之**增加 / 消失**, 且该变化规则需与既有 Sprint 生命周期规则 (per `docs/briefs/kanban-sprint-view-001.md` §8 Jira 设计参考) **不冲突**

### 1.2 背景 (用户痛点)

ULYS-15 issue 原文 (issue 创建者, 人类成员, 2026-09-12):

> "无限画布功能除了 miro 的功能点，还要具备 n8n 那种工作流的功能，并且可以把一套工作流按照标签的模式纳入一个任务卡，在 backlog 里出现，并且允许拖拽进冲刺，在面板管理，改变标签后，任务卡爷随之改变，消失或者增加，这一系列的功能要妥善设计，我需要需求文档"

**已确认的现状差距** (per 代码走查, 2026-09-12):

| 现状 | 差距 |
|---|---|
| `frontend/src/app/automation/page.tsx` 已有 Rule + Trigger(6 种) + Condition(CEL) + Action(6 种) + RuleExecutor, `CanvasView.tsx` 已有 `automation_node` element kind (line 236) | **扁平规则**: 1 trigger → 1 condition → N action **线性列表**, 无分支 / 循环 / 子流程 / 多触发节点 / 节点间变量传递 — 不是图结构, 不是 n8n |
| `frontend/src/app/workflow/page.tsx` + `Workflow`/`WorkflowState`/`WorkflowTransition` 类型 (`ids.ts` §8) | 这是 **WorkItem 状态流转状态机** (todo→in_progress→review→done 类), **与 n8n 式自动化流程是完全不同的概念**, 仅因中文/英文命名相同 (\"workflow\") 而易混淆, 本 SRS §2.4 专门消歧 |
| `frontend/src/components/sprint/SprintBoardView.tsx` — Backlog = `workItems.filter(w => !w.sprint_id)`, 拖拽走 `@dnd-kit`, `addToSprint`/`removeFromSprint` 修改 `sprint_id` | 任务卡机制已成熟 (Jira 式 7 动作), 但**没有"由标签自动生成/回收任务卡"的机制** — 目前 WorkItem 都是人工创建 |
| `WorkItem.labels: string[]` (`ids.ts` line 216) | 已有标签字段, 可直接复用, **无需新增 WorkItem 字段**做标签存储 |

#### 1.2.1 追评委托 (v1.1 新增, per 守门 #1 禁回溯叙事 — 显式记录而非静默并入)

issue 创建者在 v1.0 交付后追加 2 条评论, 均在原 issue 范围内, 本次 v1.1 显式吸收:

> 追评 1 (`01a09566-3f2d-7dc0-81c6-1ff46b89f056`, 2026-09-12T11:35:01Z): "默认要一套图生高模到绑骨骼低模化加动作，各类贴图等的 AAA 工作流模板。再加一套编程的 spec 式的和一套 superpowers 式的工作流模板，用户选择后，可直接把 agent 占位和边以及数据流，其他类型的 node 都预设出来，供用户调整和自定义保存"

> 追评 2 (`01a09567-5720-7500-a870-9cff243e8102`, 2026-09-12T11:36:12Z): "这些功能要支持 langgraph 的智能控制，允许通过最下方的聊天栏位创建，如果没有这个聊天栏位，要补上，我之前提过这个需求。"

**已确认的现状差距 (针对追评, per 代码走查 2026-09-12)**:

| 现状 | 差距 |
|---|---|
| `frontend/src/lib/store.ts` line 565 已有 `actor_session_id: undefined, // TODO: 接入 L0 chat bar session` (位于 `assignee_type: "agent"` 分支, per ADR-0046 §6.1) | 证实创建者"之前提过"的说法属实: 代码里已预留 L0 聊天栏会话接入点, 但**聊天栏 UI 本身尚未实现** |
| `frontend/src/components/CommandBar.tsx` | 全仓库唯一含"底部/bottom/chat"相关字样的组件, 实为 ⌘K 式命令面板 (顶部/浮层触发), **不是**画布底部常驻聊天输入栏, 确认聊天栏缺口真实存在, 非误标 |
| `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` (L0/L1 + TMO 7 StateGraph 节点 + `/api/tmo/*` 8 端点) | LangGraph 架构已在 ADR 层面拍板并部分落地 (`TopAgentState`/`SubAgentState`), 但**未接入画布/聊天栏交互层**, 也未覆盖"动态路由控制自动化 Flow 执行"这一新场景 |
| `AutomationActionKind` 已含 `dispatch_agent` (`ids.ts` line 866), `SRS-CANVAS-WORKFLOW-001` v1.0 §4.7.2 FR-W7.2 已有"流程模板库"占位 (P2, 已知缺口"模板数量与清单待拍板") | 追评 1 正是对该已知缺口的具体拍板 (3 套模板 + 具体清单), 本次由 **W14 承接落地**, 不新增第二套模板机制 (详见 §4.7.2 已更新的已知缺口 + §4.14) |

### 1.3 包含范围 (In-Scope)

#### 1.3.1 3 专题 SRS 职责划分 (更新, 双核心 → 三核心)

| SRS | 域 | 子能力 | 项数 |
|---|---|---|---|
| `SRS-CANVAS-001` (总册) | 跨域 / 索引 | 三核心索引 + 优先级 + 跨块接口 | 索引级, 待总册 v1.2 同步收录本专题 |
| `SRS-CANVAS-AGENT-001` | 核心 1: agent 管理 | A1-A12 | 46 项 |
| `SRS-CANVAS-GAMIFY-001` | 核心 2: 游戏化 | G1-G12 | 32 项 |
| **`SRS-CANVAS-WORKFLOW-001` (本 SRS)** | **核心 3: 自动化流程** | **W1-W15 (15 子能力)** | **54 项** |

#### 1.3.2 W1-W15 子能力总览

| 子能力 | 名称 | 项数 | 概述 |
|---|---|---|---|
| W1 | 流程节点类型体系 | 4 | 触发 / 动作 / 条件 / 子流程 4 类节点, 扩展 `automation_node` |
| W2 | 触发节点 | 4 | manual / schedule / webhook / canvas_event |
| W3 | 动作节点 | 3 | 复用 6 `AutomationActionKind` + 通用动作 + 链式编排 |
| W4 | 分支与条件 | 3 | IF / Switch / Merge |
| W5 | 循环与批处理 | 2 | Loop node + 并发限制 |
| W6 | 变量与表达式传递 | 3 | 节点间数据流 + 全局变量 + CEL 表达式 |
| W7 | 子流程与复用 | 2 | 子流程引用 + 模板库 |
| W8 | 错误处理与重试 | 3 | 节点级 retry + 错误分支 + 失败通知 |
| W9 | 执行历史与调试 | 3 | 执行日志 + 从失败节点重跑 + 调试面板 |
| W10 | 激活状态与版本管理 | 3 | active/inactive + 版本历史 (SCD2) + 回滚 |
| W11 | 标签绑定任务卡 | 5 | 流程↔标签绑定 + 派生任务卡生成 + 持久化判定 + 标签变更传播 + 标签表达式 |
| W12 | Backlog / Sprint 联动 | 5 | 默认落 Backlog + 拖拽进 Sprint + **在制 Sprint 标签变更规则** + 面板管理 UI + 手动覆盖 |
| W13 | 数据一致性与状态映射 | 2 | 流程执行状态 ↔ WorkItem.status 解耦声明 + 1 task 1 sprint 维持 |
| W14 (**v1.1 新增**) | 默认工作流模板库 | 7 | AAA 游戏资产管线 / spec 式 / superpowers 式 3 套内置模板 + 选中后自动预置 agent 占位节点/边/数据流 + 自定义另存 |
| W15 (**v1.1 新增**) | LangGraph 智能控制 + 底部聊天栏 | 5 | 补齐底部聊天栏 (自然语言建/改 Flow) + LangGraph 式动态路由 + 与既有 L0/TMO 架构对接 |

**合计: 54 项 FR** (P0 优先, 3 个月内目标; 其中 W1-W13 = 42 项为 v1.0 范围, W14-W15 = 12 项为 v1.1 新增)

#### 1.3.3 n8n 概念 → 本 SRS 映射表 (per 缺标比错标, 逐项落地而非笼统"对标 n8n")

| n8n 概念 | 本 SRS 落地项 | 画布 element / connector kind | 复用 / 新增 |
|---|---|---|---|
| Trigger node (manual/schedule/webhook/event) | W2.1-W2.4 | `workflow_trigger_node` (新增 kind) | 触发种类复用 `AutomationTriggerKind` + 新增 webhook/canvas_event |
| Action node | W3.1-W3.3 | `workflow_action_node` (新增 kind) | 复用 `AutomationActionKind` 6 种 |
| IF / Switch 分支 | W4.1-W4.2 | `workflow_condition_node` + 2/N 条 outbound connector | 复用 CEL (`condition_expr`) |
| Merge 汇合 | W4.3 | `workflow_merge_node` (新增 kind) | 新增 |
| Loop / batch | W5.1-W5.2 | `workflow_loop_node` (新增 kind, 含子图边界 Frame) | 新增, 复用 V0.1 Frame |
| 表达式 / 变量传递 | W6.1-W6.3 | connector 上标注 `data_mapping` | 新增 |
| Credentials (凭据) | — | — | **不含**, 复用 NFR-SEC-1 stdin pipe 既有约束, 不做通用凭据保险箱 (P3+, 见 §1.4) |
| Error handling + retry | W8.1-W8.3 | `workflow_action_node.retry_policy` + 红色 error connector | 新增 |
| Sub-workflow | W7.1 | `workflow_subworkflow_node` (新增 kind, 引用另一 flow_id) | 新增 |
| Execution history + re-run | W9.1-W9.2 | 独立面板 (非画布 element), 类比 `/automation` 页 | 新增 |
| Active/inactive + versioning | W10.1-W10.3 | 工具栏开关 + 版本下拉 | 新增, W/T/M per 守门 #13 |
| Agent 占位节点 (**v1.1 新增**, per 追评 1) | W14.5 | 复用 `workflow_action_node` (`action_kind = "dispatch_agent"`), 不新增 element kind | 复用 W3.1 既有 `dispatch_agent` 动作, 新增 `is_placeholder` / `placeholder_role_hint` 字段, 不扩展 W1 节点类别体系 |
| LangGraph 式动态路由 (**v1.1 新增**, per 追评 2) | W15.3 | `workflow_condition_node.routing_mode = "static_cel" \| "dynamic_agent"` | 新增, 区别于 W4 静态 CEL 分支, 对接既有 L0/TopAgentState (ADR-0046) |
| 底部聊天栏 / NL→Flow (**v1.1 新增**, per 追评 2) | W15.1-W15.2 | 独立于画布 element 的固定 UI 区域 (非画布 element), 类比 §4.9.1 执行历史面板"非画布 element"处理方式 | 新增 |

### 1.4 不包含范围 (Out-of-Scope, per 守门 #11 缺标比错标)

| 不包含类别 | 内容 | 理由 / 转交方 |
|---|---|---|
| 通用凭据保险箱 (Credentials vault) | OAuth token / API key 加密存储管理 | 复用既有 NFR-SEC-1 (stdin pipe, 不落库), 通用凭据管理超出本 SRS, 留 P3+ (per 总册 NFR-SEC-5 已砍第三方 OAuth 集成同理) |
| 真实第三方连接器市场 (n8n 400+ integrations) | Slack / Salesforce / Google Sheets 等具名连接器 | `call_webhook` 动作已可覆盖任意 HTTP 端点, 具名连接器 UI 封装留 P2+ |
| 可视化拖拽式表达式构建器 (n8n Expression Editor UI) | 图形化表达式辅助输入 | v1 仅文本 CEL 编辑框, 图形化辅助留 P2+ |
| 节点级混合标签 (同一 flow 内不同 node 挂不同 tag 参与不同任务卡绑定) | per §4.11.5 已知缺口 | v1 标签绑定在 **flow 整体级别**, 不做 node 级标签继承, 留 P2+ |
| 真正的分布式/持久化执行引擎 (跨进程 durable execution, 类 Temporal) | 执行状态崩溃恢复 | v1 复用现有前端 mock + `automationRules` store 同构模拟, 真实后端执行引擎属于 Agent Runtime / BFF 范畴, per `SRS-STAR-AGENT-RUNTIME-001.md` 边界, 本 SRS 只定义前端画布 + 数据模型 + 交互, 不做后端执行调度设计 |
| Miro 其余 11 种 diagram / 模板库 / 演示 / 版本 / 移动 / 完整 a11y / 通用第三方集成 | per §0.3 反转范围声明 | 维持 `SRS-CANVAS-001` §1.4 砍掉决定, 不在本次反转范围 |
| 现有 `Workflow` 状态机 (`/workflow` 页) 改造 | WorkItem 状态流转配置 | 与本 SRS 是**两个独立概念** (per §2.4 消歧), 不改动、不合并, W13.1 仅定义可选映射接口 |
| ARG (Agent Relationship Graph) 相关能力 | delegates_to / consults 等 10 类关系 | 属于 `SRS-CANVAS-AGENT-001` A11, 本 SRS 不重复 |
| 游戏化 32 项 | reward / 积分 / 投票 / confetti | 属于 `SRS-CANVAS-GAMIFY-001`, 不重复 |
| 后端持久化 API 具体实现 (数据库表 DDL / BFF route handler 代码) | REST/DB 层实现 | 本 SRS 只定义 §7/§8 数据与接口**需求** (schema 增项 + API 契约), 具体实现留 Design Doc (DD-CANVAS-WORKFLOW-001, 待创建) |
| A12 多人协同编辑同一 Automation Flow 画布的 CRDT 冲突解决 | 多人同时拖节点 | 复用 `SRS-CANVAS-AGENT-001` A12 既有多人编辑基线, 本 SRS 不重新设计 CRDT, 只声明依赖 |
| 真实第三方 LLM/NLU API 调用 (聊天栏自然语言解析 W15.2, 动态路由判断 W15.3) | 真实调用 OpenAI/Claude 等外部大模型做 NL→Flow 生成或路由决策 | per 守门 #23 v2 (禁真实第三方 AI API 调用), v1 一律 **mock 接口** (固定/规则化解析, 类比 `SRS-CANVAS-GAMIFY-001` G5 AI 聚类的同款处理), 真实 LLM 接入留 P2, 详见 §10 |
| 真实 LangGraph Python 后端执行引擎接入 (真实 `console_server.py` `/api/tmo/*` 调用) | 聊天栏/动态路由与真实 L0/TMO 服务的网络对接 | 本 SRS 只定义前端契约 (画布 UI + 数据模型 + mock 桩), 复用 §3.2 "v1 前端 zustand mock" 既有约束边界, 真实对接属 Design Doc + Agent Runtime 范畴 |
| superpowers 式模板中各阶段的真实技能自动触发 (如自动调用 brainstorming/TDD 技能执行) | Flow 节点自动驱动真实 agent 技能执行 | v1 该模板仅在画布上预置"阶段占位节点 + 顺序边", 代表方法论阶段划分, 不做真实技能调度对接, 真实调度属 Agent Runtime 范畴 |

### 1.5 用户故事 (≥ 60% = 54 × 0.6 = 32.4 → 满足 ≥ 33, 本 SRS 提供 34 个)

| 编号 | 角色 | 故事 | 子能力 | 优先级 |
|---|---|---|---|---|
| US-W1 | Dev | 作为 Dev, 我希望在画布上拖出一个"手动触发"节点连到"发通知"节点, 不写代码就能搭一个简单自动化 | W1 + W2.1 + W3.1 | P0 |
| US-W2 | SRE | 作为 SRE, 我希望用"定时触发"节点每天 9 点跑一次巡检流程, 复用现有 `schedule_cron` | W2.2 | P0 |
| US-W3 | Dev | 作为 Dev, 我希望用"webhook 触发"节点接收外部系统的 POST 请求启动流程 | W2.3 | P1 |
| US-W4 | PM | 作为 PM, 我希望画布上"元素被移动/创建"这类画布事件也能触发自动化 (canvas_event), 不用切到 `/automation` 页 | W2.4 | P1 |
| US-W5 | Dev | 作为 Dev, 我希望用 IF 节点判断"work item priority == p0"走不同分支, 不用为每个优先级单独建规则 | W4.1 | P0 |
| US-W6 | Dev | 作为 Dev, 我希望用 Switch 节点按 `agent_kind` 值路由到 3 条不同分支, 不用嵌套多层 IF | W4.2 | P1 |
| US-W7 | Dev | 作为 Dev, 我希望两条分支执行完后能汇合到同一个后续节点, 不用复制后续逻辑两份 | W4.3 | P1 |
| US-W8 | SRE | 作为 SRE, 我希望用 Loop 节点对 50 个 work item 逐个跑同一段处理逻辑, 不用建 50 条规则 | W5.1 | P1 |
| US-W9 | SRE | 作为 SRE, 我希望限制 Loop 并发数为 5, 避免瞬间打爆下游 webhook | W5.2 | P2 |
| US-W10 | Dev | 作为 Dev, 我希望把"触发节点"的输出字段 (如 work_item.id) 直接引用到下游动作节点的参数里, 不用手抄 | W6.1 | P0 |
| US-W11 | Dev | 作为 Dev, 我希望定义一个流程级全局变量 (如 `threshold=0.8`), 多个节点都能读, 改一处生效全流程 | W6.2 | P2 |
| US-W12 | Dev | 作为 Dev, 我希望条件节点用跟 automation rule 一样的 CEL 语法, 不用学新的表达式语言 | W6.3 | P0 |
| US-W13 | PM | 作为 PM, 我希望把一段常用逻辑存成"子流程", 在多个主流程里引用, 改一次全生效 | W7.1 | P1 |
| US-W14 | Dev | 作为 Dev, 我希望有几个预置流程模板 (如"PR 合并后通知 + 建 worktree") 一键套用, 不用从零搭 | W7.2 | P2 |
| US-W15 | SRE | 作为 SRE, 我希望某个动作节点失败时自动重试 3 次 (指数退避), 不用人工重跑 | W8.1 | P0 |
| US-W16 | SRE | 作为 SRE, 我希望重试耗尽后走"错误分支"发告警, 而不是让整个流程静默失败 | W8.2 | P0 |
| US-W17 | 5 域 Lead | 作为 Lead, 我希望流程失败时收到通知, 不用主动去查执行历史才发现 | W8.3 | P1 |
| US-W18 | SRE | 作为 SRE, 我希望看到每次执行的完整历史 (哪个节点跑了多久、输出什么), 排障不用猜 | W9.1 | P0 |
| US-W19 | SRE | 作为 SRE, 我希望流程在第 3 个节点失败后, 能只从第 3 个节点重跑, 不用把前 2 个已成功节点重新跑一遍 | W9.2 | P0 |
| US-W20 | Dev | 作为 Dev, 我希望调试时能单独看某个节点这次跑的输入/输出 JSON, 不用翻日志文本 | W9.3 | P1 |
| US-W21 | PM | 作为 PM, 我希望能一键停用某个流程而不删除它, 排查问题时先止血 | W10.1 | P0 |
| US-W22 | PM | 作为 PM, 我希望流程改动后能看到版本历史并按需回滚, 改坏了能一键退回 | W10.2 + W10.3 | P1 |
| US-W23 | PM (issue 创建者场景) | 作为 PM, 我希望给一套自动化流程打上标签 (如 "release-checklist"), 系统自动在 Backlog 生成对应任务卡, 不用手建 | W11.1 + W11.2 | P0 |
| US-W24 | PM | 作为 PM, 我希望把这张自动生成的任务卡跟其他人工任务卡一样, 直接拖进 Sprint, 不需要额外操作 | W12.1 + W12.2 | P0 |
| US-W25 | PM (issue 创建者场景) | 作为 PM, 我希望在标签管理面板里把某个标签从流程上摘掉后, 对应任务卡如果还在 Backlog 就自动消失; 如果已经进了正在进行的 Sprint, 不要被直接删掉, 而是标出"已脱离标签"提醒我处理 | W11.4 + W12.3 | P0 |
| US-W26 | PM | 作为 PM, 我希望反过来给流程新加一个已绑定规则匹配的标签后, Backlog 里自动出现新任务卡, 不用手建 | W11.4 | P0 |
| US-W27 | 5 域 Lead | 作为 Lead, 我希望在面板管理里看到"标签 → 流程 → 任务卡"的绑定关系一览表, 一眼看懂哪些自动化在跑 | W12.4 | P1 |
| US-W28 | SRE | 作为 SRE, 我希望流程的运行状态 (跑成功/失败) 不会自动改动 task 卡的看板状态 (todo/done), 两者互不干扰, 除非我显式配置映射 | W13.1 | P1 |
| US-W29 (**v1.1 新增**) | 美术/TA | 作为美术/TA, 我希望一键套用"AAA 游戏资产管线"模板, 画布自动搭好"图生高模→绑骨骼→低模化→加动作→出贴图"整条 agent 占位链路, 不用从零拖节点 | W14.1 + W14.2 | P1 |
| US-W30 (**v1.1 新增**) | Dev | 作为 Dev, 我希望套用"spec 式"编程工作流模板, 自动搭好"需求 spec 撰写→评审→实现→测试→代码评审"的节点链路 | W14.1 + W14.3 | P1 |
| US-W31 (**v1.1 新增**) | Dev | 作为 Dev, 我希望套用"superpowers 式"工作流模板, 自动搭好"brainstorming→写计划→执行计划→代码评审→收尾"的阶段占位节点链路 | W14.1 + W14.4 | P1 |
| US-W32 (**v1.1 新增**) | PM | 作为 PM, 我希望套用任意默认模板后能直接调整节点参数/占位 agent, 并另存为我自己的自定义模板供团队复用 | W14.5 + W14.6 + W14.7 | P1 |
| US-W33 (**v1.1 新增**, issue 创建者场景) | PM | 作为 PM, 我希望在画布底部的聊天栏里用自然语言描述"每天 9 点检查 P0 缺陷并通知我", 系统直接生成对应的 Flow 节点图, 不用手动拖节点 | W15.1 + W15.2 | P0 |
| US-W34 (**v1.1 新增**) | 5 域 Lead | 作为 Lead, 我希望流程执行到条件节点时能由 LangGraph 式智能判断 (而非只有写死的 CEL 表达式) 决定下一步走哪个 agent, 并且这套判断跟现有 L0/TMO 架构是同一套, 不是另起炉灶 | W15.3 + W15.4 | P1 |

**用户故事覆盖统计**: 34 个 (≥ 32.4 要求, 满足)

---

## §2 用语定义 (用语集 / Ubiquitous Language)

### 2.1 画布基础 (per `frontend-canvas-design.md` v0.1, 沿用总册 §2.1, 不重复定义)

### 2.2 自动化流程域 (本 SRS 新增)

| 术语 | 定义 |
|---|---|
| 自动化流程 (Automation Flow, 简称 **Flow**) | 由 N 个流程节点 + 有向 connector 构成的可执行图, 是 `AutomationRule` 的图结构超集; 1 Flow = 1 `automation_flow` 记录 |
| 流程节点 (Flow Node) | Flow 图中的最小单元, 4 类: 触发 (Trigger) / 动作 (Action) / 条件 (Condition) / 子流程 (Sub-flow); 画布上渲染为新增 element kind |
| 触发节点 (Trigger Node) | Flow 的起点, 决定何时启动执行, 复用 `AutomationTriggerKind` + 新增 webhook / canvas_event |
| 动作节点 (Action Node) | 执行副作用的节点, 复用 `AutomationActionKind` 6 种 |
| 条件节点 (Condition Node) | IF (2 分支) 或 Switch (N 分支), guard 用 CEL, 复用 `condition_expr` |
| 汇合节点 (Merge Node) | N 条分支汇合为 1 条后续路径 |
| 循环节点 (Loop Node) | 对数组批量执行子图, 画布上用 Frame 包裹循环体 |
| 子流程节点 (Sub-flow Node) | 引用另一个 `automation_flow`, 深度 ≤ 5 防止循环引用 |
| 执行 (Execution / Run) | 1 次 Flow 从触发到结束的完整运行记录, `execution_history` 表 (Transaction) |
| 执行步骤 (Execution Step) | 1 次 Execution 中单个节点的运行记录 (输入/输出/耗时/状态) |
| 标签绑定 (Tag Binding) | 1 个 Flow 在整体级别绑定 1 条**标签表达式** (tag AND/OR/NOT), 决定是否生成派生任务卡 |
| 派生任务卡 (Derived Task Card) | 因 Tag Binding 命中而由系统自动创建/更新的 1 条 **真实 WorkItem 记录** (非虚拟投影, per §4.11.3 判定), `source_kind = "workflow_tag_binding"` |
| 脱离状态 (Detached) | 派生任务卡所在 Sprint 已 **active** 且其标签绑定被移除时的过渡状态, 卡片保留但标红提示, 不立即删除 (per BR-W-3) |
| 默认工作流模板 (Default Flow Template, **v1.1 新增**) | 内置只读的一套预配置节点图 (含节点/边/数据流/占位 agent), 选中后一键实例化为可编辑 `automation_flow`; v1.1 内置 3 套: AAA 游戏资产管线 / spec 式编程 / superpowers 式 |
| Agent 占位节点 (Agent Placeholder Node, **v1.1 新增**) | `workflow_action_node` 的一种状态 (`action_kind = "dispatch_agent"` 且 `is_placeholder = true`), 表示"此处需要一个 agent 但尚未绑定具体 agent_id", 供用户后续指派, 不新增 element kind (per W14.5) |
| 底部聊天栏 (Bottom Chat Bar, **v1.1 新增**) | 画布视图底部常驻的自然语言输入区域, 用于对话式创建/修改 Flow, 对接既有 L0 会话 (`actor_session_id`, per ADR-0046), 非画布 element |
| LangGraph 式动态路由 (Dynamic Agent Routing, **v1.1 新增**) | 条件节点的一种路由模式 (`routing_mode = "dynamic_agent"`), 由 agent 在执行时依据上下文动态决定走哪条分支, 区别于 W4 的静态 CEL 表达式路由 (`routing_mode = "static_cel"`) |

### 2.3 跨域共享 (复用总册 §2.4, 不重复)

### 2.4 "工作流" 一词消歧 (**本 SRS 必读**, 防止与既有 `Workflow` 类型混淆)

| 词 | 指代 | 数据类型 | 用途 | 页面 |
|---|---|---|---|---|
| 状态流转工作流 (Status-Transition Workflow, 既有 V0.1) | WorkItem 状态机 (todo→in_progress→review→done) | `Workflow` / `WorkflowState` / `WorkflowTransition` (`ids.ts` §8) | 定义 1 个 project 的 work-item 状态迁移规则 (state.category + transition.trigger/guard) | `/workflow` |
| **自动化流程 (Automation Flow, 本 SRS 新增)** | n8n 式可执行节点图 | 本 SRS 新增 `automation_flow` / `FlowNode` / `FlowEdge` | 编排"当 X 发生, 做 Y, 若 Z 则..." 的自动化逻辑 | 画布 (`/board` 或独立 `/canvas/automation`, per §8.2 待定) |

**规则**: 本 SRS 全文使用"**自动化流程 / Flow**"指代新概念, 从不使用裸词"workflow"以避免歧义; 若中文语境需要, 统一称"自动化流程", 不称"工作流" (后者留给既有状态机概念)。

---

## §3 业务背景 / 前提条件

### 3.1 业务背景

STAR 平台已有 `automation` module (`frontend/src/app/automation/page.tsx`, per 总册 §4.5.2 25 module 联动矩阵第 6 行) 提供 Rule + Trigger + Condition(CEL) + Action + RuleExecutor 6 INV 基线, 且 `CanvasView.tsx` 已有 `automation_node` element kind 可在画布上渲染单条规则并跳转 `/automation` 详情。

`SRS-CANVAS-001` §4.5.2 已预留"`automation` 画布事件触发 rule, rule trigger_kind 加 `canvas_event`"的扩展点, 但截至 ULYS-15 委托时尚未实装图结构编排能力 (仍是线性 1 trigger + N action)。

同时, `SprintBoardView.tsx` 已实装成熟的 Jira 式 Backlog↔Sprint 拖拽机制 (per `docs/briefs/kanban-sprint-view-001.md`), 但当前所有 WorkItem 均为人工创建, 无"系统按规则自动生成/回收任务卡"的先例。

### 3.2 前提条件 (per 守门 #11 缺标比错标)

| 前提 | 状态 | 影响 |
|---|---|---|
| `AutomationRule` / `automation_node` / `/automation` 页已实装 (前端 mock, zustand store) | ✅ 已确认 | W1-W10 在此基础上做图结构扩展, 非从零新建 |
| `WorkItem.labels: string[]` 已有字段 | ✅ 已确认 (`ids.ts` line 216) | W11 标签绑定直接复用, 无需新增 WorkItem 字段存标签 |
| `sprint_id` 缺省 = Backlog 的判定逻辑已实装 (`SprintBoardView.tsx` line 511) | ✅ 已确认 | W12.1 直接复用, 派生任务卡只需正确置空 `sprint_id` 即可出现在 Backlog |
| Backlog↔Sprint 拖拽机制 (`@dnd-kit`, `addToSprint`/`removeFromSprint`) | ✅ 已确认 | W12.2 无需改动拖拽逻辑本身, 派生任务卡是普通 WorkItem, 天然可拖 |
| 后端真实持久化 / 数据库表 | ❌ 未确认 (V0.1 为 localStorage + zustand persist, 派生视图不持久化) | 本 SRS §7 仅定义 schema 需求, 真实后端落地属于 Design Doc + 实装阶段, 是 P0 阻塞前提, 已列入 §10 已知风险 |
| CRDT / 多人协同选型 (Yjs / Automerge / LWW) | ❌ 未拍板 (per `SRS-CANVAS-AGENT-001` A12.6 P0 阻塞) | 本 SRS 画布编辑 Flow 图若需多人协同, 依赖 A12 选型结果, 不重复设计 |

### 3.3 业务规则 (Business Rules, 新增于本 SRS, 编号延续总册惯例)

| 编号 | 规则 |
|---|---|
| BR-W-1 | 1 个 Flow 在**整体级别**绑定 0..1 条标签表达式 (v1 不支持 1 Flow 多条表达式) |
| BR-W-2 | 派生任务卡是**真实 WorkItem** (Master, SCD Type 2), 非只读投影 (per §4.11.3 判定依据) — 因为拖入 Sprint 需要可写的 `sprint_id` 字段, 投影视图无法承载写操作 |
| BR-W-3 (**核心冲突裁决规则**, per issue "改变标签后任务卡随之改变") | 标签绑定变化对派生任务卡的影响按卡片所处生命周期分 3 种: (a) 卡片仍在 **Backlog** (`sprint_id` 为空) → 标签命中新增 tag → 立即生成新卡; 标签解绑 → 立即删除对应卡 (硬删除, 因未进入任何 Sprint, 不影响既有 Sprint 统计); (b) 卡片在 **planned** (未开始) Sprint 中 → 标签解绑 → 卡片自动移出该 Sprint 并回 Backlog (`sprint_id` 置空), 再按 (a) 规则处理, 与既有"Sprint 未开始可自由调整"隐含规则一致; (c) 卡片在 **active** (进行中) Sprint 中 → 标签解绑 → **不自动删除、不自动移出**, 卡片状态置为 `tag_binding_status = "detached"` 并在 Sprint 看板上标红角标"🔗 已脱离标签", 直到该 Sprint `completeSprint` / `cancelSprint` 时按既有"未完成卡片 sprint_id 置空回 Backlog"规则统一处理 (per `kanban-sprint-view-001.md` §8) — **不破坏"Sprint 只能从 Backlog 拉 / 移出 Sprint 状态回 Backlog / 1 task 1 sprint"3 项已拍板规则** |
| BR-W-4 | 1 张派生任务卡任意时刻最多绑定 1 个 Flow (`WorkItem.workflow_tag_binding_id` 1:1), 避免多 Flow 争抢同一张卡的字段更新 |
| BR-W-5 | 派生任务卡的 `title` / `description` / `labels` 由系统按 Flow 元数据 (name + 绑定的标签表达式) 自动生成并在 Flow 变更时同步覆盖; 但用户对该卡做过的**人工编辑** (如改 `assignee_id` / `priority` / `story_points`) 保留, 不被系统覆盖 (per 字段级 CRDT-lite: 系统字段 vs 用户字段分区, 详见 §7.4) |
| BR-W-6 | 子流程引用深度 ≤ 5, 超出报错拒绝保存 (防环 / 防无限递归) |
| BR-W-7 | 流程执行状态 (idle/running/succeeded/failed/waiting) 与 WorkItem.status (todo/in_progress/review/blocked/done/wontfix) **默认不映射**, 除非 W13.1 显式配置映射规则 (per §1.4 已排除自动映射假设) |

---

## §4 业务需求 (54 项 FR, 其中 W1-W13 = 42 项 v1.0 / W14-W15 = 12 项 v1.1 新增)

### 4.1 W1 流程节点类型体系 (4 项)

#### 4.1.1 FR-W1.1 新增 4 类流程节点 element kind

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W1.1 |
| 描述 | `CanvasView.tsx` 新增 4 个 element kind: `workflow_trigger_node` / `workflow_action_node` / `workflow_condition_node` / `workflow_subworkflow_node` (`workflow_merge_node` / `workflow_loop_node` 见 W4.3 / W5.1), 均挂在同一 `automation_flow_id` 下构成 1 张图 |
| 输入 | `automation_flow_id` (UUID) |
| 输出 | SVG `<g>` element, 按 kind 区分图标 (⚡ trigger / ▶ action / ◇ condition / 📦 subflow) + 边框色 |
| 数据 schema 增项 | 新增 `automation_flow` 表 (Master) + `flow_node` 表 (Master, 属于某 flow) |
| 接口依赖 | 复用 V0.1 element 渲染基座 (`CanvasView.tsx` line 156-340 的 switch-case 结构), 新增 4 个 case 分支 |
| 业务规则 | BR-W-1 |
| 优先级 | P0 |

**用户故事**: US-W1

**验收标准**: AC-W1.1 — 画布上可创建 4 类节点, 每类节点有独立图标与色码, 节点属于唯一 1 个 `automation_flow_id`

**已知缺口**: 节点级混合标签不支持 (per §1.4)

#### 4.1.2 FR-W1.2 节点间有向 connector (流程边 / Flow Edge)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W1.2 |
| 描述 | 复用 V0.1 3 种 connector routing (straight/bezier/orthogonal), 新增 `flow_edge` 语义: 每条 edge 有 `from_node_id` / `to_node_id` / `edge_label` (用于条件分支标注 true/false/case值) |
| 数据 schema 增项 | `flow_edge` 表 (Master) |
| 接口依赖 | 复用 V0.1 connector 渲染 |
| 业务规则 | — |
| 优先级 | P0 |

**用户故事**: US-W1

**验收标准**: AC-W1.2 — 节点间可连线, 条件节点连出的边显示分支标签

**已知缺口**: 无

#### 4.1.3 FR-W1.3 画布工具栏新增"自动化流程"创建入口

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W1.3 |
| 描述 | 工具栏 (`CanvasView.tsx` line 376) 新增 "+ Flow" 按钮, 点击后进入 Flow 编辑模式 (子画布或独立 canvas, per §8.2 待定), 可从空白开始拖节点 |
| 接口依赖 | 复用 V0.1 工具栏 UI 组件 |
| 业务规则 | — |
| 优先级 | P0 |

**用户故事**: US-W1

**验收标准**: AC-W1.3 — 点击 "+ Flow" 创建 1 个空 `automation_flow` 记录并进入编辑

**已知缺口**: Flow 编辑是独立子画布还是复用主画布同一 viewport, 待 Design Doc 决策 (见 §10)

#### 4.1.4 FR-W1.4 双击流程节点跳转 / 展开详情

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W1.4 |
| 描述 | 复用 V0.1 `onElementDoubleClick` (line 132-143) 模式, 双击流程节点展开侧栏显示节点配置 (trigger_kind / condition_expr / action config 等) |
| 接口依赖 | 复用 V0.1 联动 2 模式 |
| 业务规则 | — |
| 优先级 | P1 |

**用户故事**: US-W20

**验收标准**: AC-W1.4 — 双击节点打开配置侧栏, 可编辑该节点字段

**已知缺口**: 无

### 4.2 W2 触发节点 (4 项)

#### 4.2.1 FR-W2.1 手动触发 (Manual Trigger)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W2.1 |
| 描述 | Flow 起点为手动触发时, 画布/面板提供"▶ 运行"按钮立即启动 1 次 Execution |
| 数据 schema 增项 | `flow_node.trigger_kind = "manual"` |
| 接口依赖 | 新增 RuleExecutor 扩展方法 |
| 优先级 | P0 |

**用户故事**: US-W1

**验收标准**: AC-W2.1 — 点击"运行"按钮触发 1 次 Execution, 生成 1 条 execution_history 记录

**已知缺口**: 无

#### 4.2.2 FR-W2.2 定时触发 (复用既有 `schedule_cron`)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W2.2 |
| 描述 | 直接复用 `AutomationTriggerKind.schedule_cron`, 无需新增触发类型, 仅需支持挂载到 `flow_node` 而非仅 `AutomationRule` |
| 接口依赖 | 复用既有 `AutomationTriggerKind` 枚举 |
| 优先级 | P0 |

**用户故事**: US-W2

**验收标准**: AC-W2.2 — cron 表达式配置后, 到点自动触发 Execution

**已知缺口**: 无

#### 4.2.3 FR-W2.3 Webhook 触发 (新增)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W2.3 |
| 描述 | 新增 `AutomationTriggerKind` 枚举值 `webhook`, 每个 webhook 触发节点生成唯一 URL (`/v1/collaboration/flows/{flow_id}/webhook/{token}`), 外部 POST 请求触发 Execution |
| 数据 schema 增项 | `AutomationTriggerKind` 枚举扩展 + `flow_node.webhook_token` |
| 接口依赖 | 新增 BFF route (per §8.2) |
| 优先级 | P1 |

**用户故事**: US-W3

**验收标准**: AC-W2.3 — POST 到 webhook URL 触发 Execution, 请求体作为触发数据可被下游节点引用

**已知缺口**: webhook 安全校验 (签名/token 轮换) 细节留 Design Doc

#### 4.2.4 FR-W2.4 画布事件触发 (`canvas_event`, 落地总册 §4.5.2 预留扩展点)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W2.4 |
| 描述 | 新增 `AutomationTriggerKind` 枚举值 `canvas_event`, 支持画布操作 (element 创建/移动/删除/tag 变更) 作为触发源, 落地 `SRS-CANVAS-001` §4.5.2 "rule trigger_kind 加 canvas_event" 预留项 |
| 数据 schema 增项 | `AutomationTriggerKind` 枚举扩展 + `trigger_filter` 支持 `canvas_event_kind` 过滤 (如仅 `element.tag_changed`) |
| 接口依赖 | 复用 V0.1 canvas action dispatch (元素增删改事件流) |
| 业务规则 | 与 W11.4 标签变更传播共用同一事件流 |
| 优先级 | P1 |

**用户故事**: US-W4

**验收标准**: AC-W2.4 — 画布元素标签变更时可触发绑定的 Flow

**已知缺口**: 事件流是否需要节流 (debounce) 防止拖拽过程中高频触发, 留 Design Doc

### 4.3 W3 动作节点 (3 项)

#### 4.3.1 FR-W3.1 复用既有 6 种 `AutomationActionKind`

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W3.1 |
| 描述 | `assign_user` / `set_label` / `send_notification` / `create_worktree` / `call_webhook` / `dispatch_agent` 6 种既有动作直接可挂载为 `workflow_action_node`, 无需重新实现执行逻辑 |
| 接口依赖 | 复用既有 `AutomationActionKind` + `RuleExecutor` 执行逻辑 |
| 优先级 | P0 |

**用户故事**: US-W1

**验收标准**: AC-W3.1 — 6 种既有动作均可作为节点使用, 执行结果与 `/automation` 页规则一致

**已知缺口**: 无

#### 4.3.2 FR-W3.2 新增通用动作: HTTP 请求 / 数据转换

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W3.2 |
| 描述 | 新增 `AutomationActionKind` 枚举值 `http_request` (通用 HTTP 调用, 区别于仅 POST 的 `call_webhook`) 与 `transform_data` (JSONPath/表达式做数据整形, 供下游节点消费) |
| 数据 schema 增项 | `AutomationActionKind` 枚举扩展 |
| 优先级 | P1 |

**用户故事**: US-W1

**验收标准**: AC-W3.2 — `http_request` 支持 GET/POST/PUT/DELETE + 自定义 header; `transform_data` 输出可被下游引用

**已知缺口**: 无

#### 4.3.3 FR-W3.3 动作节点链式编排 (顺序 / 并行)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W3.3 |
| 描述 | N 个动作节点可用 connector 顺序连接 (前一个完成才执行下一个), 或从同一上游节点分出多条边并行执行 (无需 Merge 等待, 各自独立) |
| 业务规则 | 并行分支若需汇合等待, 用 W4.3 Merge 节点 |
| 优先级 | P1 |

**用户故事**: US-W1

**验收标准**: AC-W3.3 — 顺序连接的动作节点按序执行; 并行分出的动作节点同时触发

**已知缺口**: 无

### 4.4 W4 分支与条件 (3 项)

#### 4.4.1 FR-W4.1 IF 分支节点

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W4.1 |
| 描述 | `workflow_condition_node` 支持 IF 模式: 1 条 CEL guard 表达式 (复用既有 `condition_expr`), 2 条 outbound edge 分别标注 `true`/`false` |
| 数据 schema 增项 | `flow_node.condition_expr` (复用 `AutomationRule.condition_expr` 语法) |
| 优先级 | P0 |

**用户故事**: US-W5

**验收标准**: AC-W4.1 — guard 求值为 true 走 true 分支, false 走 false 分支, 二选一, 不重复触发

**已知缺口**: 无

#### 4.4.2 FR-W4.2 Switch 多分支节点

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W4.2 |
| 描述 | `workflow_condition_node` 支持 Switch 模式: 1 条取值表达式 + N 个 case 值, 每个 case 对应 1 条 outbound edge, 另有 1 条 `default` 边兜底 |
| 优先级 | P1 |

**用户故事**: US-W6

**验收标准**: AC-W4.2 — 表达式命中某 case 走对应分支, 无命中走 default

**已知缺口**: 无

#### 4.4.3 FR-W4.3 Merge 汇合节点

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W4.3 |
| 描述 | 新增 element kind `workflow_merge_node`, N 条 inbound edge 汇合为 1 条 outbound edge, 支持 "任一到达即触发" (race) 或 "全部到达才触发" (join) 2 种模式 |
| 数据 schema 增项 | `flow_node.merge_mode: "race" \| "join"` |
| 优先级 | P1 |

**用户故事**: US-W7

**验收标准**: AC-W4.3 — race 模式首个到达即继续, join 模式等待全部到达

**已知缺口**: join 模式超时策略 (等多久算超时) 留 Design Doc

### 4.5 W5 循环与批处理 (2 项)

#### 4.5.1 FR-W5.1 Loop 节点 (数组批处理)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W5.1 |
| 描述 | 新增 element kind `workflow_loop_node`, 接收数组输入 (如 work_item 列表), 对每个元素执行内部子图 (子图用 V0.1 Frame 包裹, 视觉上表示"循环体"), 全部完成后继续下游 |
| 数据 schema 增项 | `flow_node.loop_source_expr` (CEL, 求值出数组) + `flow_node.loop_body_node_ids[]` |
| 接口依赖 | 复用 V0.1 Frame 渲染 |
| 优先级 | P1 |

**用户故事**: US-W8

**验收标准**: AC-W5.1 — 数组有 N 项, 循环体内节点执行 N 次, 每次输入为当前项

**已知缺口**: 无

#### 4.5.2 FR-W5.2 并发限制

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W5.2 |
| 描述 | Loop 节点支持配置 `concurrency` (默认 1, 顺序执行), 大于 1 时并发执行, 上限 20 防止资源耗尽 |
| 数据 schema 增项 | `flow_node.concurrency: number (1-20)` |
| 优先级 | P2 |

**用户故事**: US-W9

**验收标准**: AC-W5.2 — concurrency=5 时同时最多 5 个循环体实例在跑

**已知缺口**: 无

### 4.6 W6 变量与表达式传递 (3 项)

#### 4.6.1 FR-W6.1 节点间数据映射

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W6.1 |
| 描述 | 下游节点参数可用 `{{node.<node_id>.output.<field>}}` 语法引用上游节点输出, 编辑侧栏提供"插入引用"辅助按钮 (非图形化, per §1.4 排除拖拽式表达式构建器) |
| 数据 schema 增项 | `flow_node.input_bindings: Record<string, string>` (字段名 → 引用表达式) |
| 优先级 | P0 |

**用户故事**: US-W10

**验收标准**: AC-W6.1 — 引用表达式在执行时被正确替换为上游实际输出值

**已知缺口**: 无

#### 4.6.2 FR-W6.2 流程级全局变量

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W6.2 |
| 描述 | `automation_flow` 可定义 `variables: Record<string, unknown>` 键值表, 任意节点可通过 `{{flow.variables.<key>}}` 引用 |
| 数据 schema 增项 | `automation_flow.variables` |
| 优先级 | P2 |

**用户故事**: US-W11

**验收标准**: AC-W6.2 — 修改全局变量后, 后续 Execution 使用新值 (不影响已完成的历史执行记录)

**已知缺口**: 无

#### 4.6.3 FR-W6.3 表达式统一走 CEL (复用既有语法, 不新增表达式语言)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W6.3 |
| 描述 | 条件节点 / 循环源表达式 / 数据转换动作均使用与 `AutomationRule.condition_expr` 相同的 CEL 语法, 保证 `/automation` 与 Flow 编辑器学习成本一致 |
| 业务规则 | 与既有 CEL guard 引擎共用 1 套 parser |
| 优先级 | P0 |

**用户故事**: US-W12

**验收标准**: AC-W6.3 — 同一条 CEL 表达式在 `/automation` 规则和 Flow 节点中求值结果一致

**已知缺口**: 无

### 4.7 W7 子流程与复用 (2 项)

#### 4.7.1 FR-W7.1 子流程引用节点

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W7.1 |
| 描述 | 新增 element kind `workflow_subworkflow_node`, 引用另一个 `automation_flow_id`, 执行时同步等待子流程跑完并把子流程末端输出作为本节点输出 |
| 数据 schema 增项 | `flow_node.referenced_flow_id` |
| 业务规则 | BR-W-6 (深度 ≤ 5 防环) |
| 优先级 | P1 |

**用户故事**: US-W13

**验收标准**: AC-W7.1 — 子流程节点执行触发被引用 Flow 的 1 次 Execution, 主流程等待其完成

**已知缺口**: 循环引用检测算法 (DFS 环检测) 留 Design Doc

#### 4.7.2 FR-W7.2 流程模板库

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W7.2 |
| 描述 | 预置 3-5 个常用模板 (如"PR 合并后通知 + 建 worktree"), 类比 `SRS-CANVAS-AGENT-001` A11.4 团队模板 1-click 部署模式, "+ Flow" 入口旁提供模板选择器 |
| 优先级 | P2 |

**用户故事**: US-W14

**验收标准**: AC-W7.2 — 选择模板后生成预配置好的 Flow 图, 可直接改参数使用

**已知缺口**: ~~模板数量与具体清单待 P2 阶段拍板~~ **已由 v1.1 W14 具体拍板并落地** (per 追评 1, `01a09566-3f2d-7dc0-81c6-1ff46b89f056`): 3 套内置模板 (AAA / spec 式 / superpowers 式) + 具体节点清单见 §4.14; 本条目 (W7.2) 保留作为"模板库存在通用机制"的原始声明, 不重复定义, 优先级维持 P2 供本 SRS 版本演进溯源 (per 守门 #1 禁回溯叙事)

### 4.8 W8 错误处理与重试 (3 项)

#### 4.8.1 FR-W8.1 节点级重试策略

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W8.1 |
| 描述 | 每个动作节点可配置 `retry_policy: { max_retries: number, backoff: "fixed" \| "exponential", delay_ms: number }`, 失败自动按策略重试 |
| 数据 schema 增项 | `flow_node.retry_policy` |
| 优先级 | P0 |

**用户故事**: US-W15

**验收标准**: AC-W8.1 — 配置 max_retries=3 后, 节点失败自动重试最多 3 次, 每次记录 1 条 execution_step

**已知缺口**: 无

#### 4.8.2 FR-W8.2 错误分支 (on_error)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W8.2 |
| 描述 | 节点重试耗尽后, 若存在标记为 `on_error` 的 outbound edge (红色, 区别于普通黑色 edge), 流程转入该分支而非整体失败终止 |
| 数据 schema 增项 | `flow_edge.edge_kind: "normal" \| "on_error"` |
| 优先级 | P0 |

**用户故事**: US-W16

**验收标准**: AC-W8.2 — 存在 on_error 分支时, 节点最终失败会走该分支; 不存在时整个 Execution 标记为 failed

**已知缺口**: 无

#### 4.8.3 FR-W8.3 流程失败通知

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W8.3 |
| 描述 | Execution 最终状态为 failed 且无 on_error 分支兜底时, 自动对接 notification 域 (复用既有 `send_notification` 动作语义) 通知 Flow 所有者 |
| 接口依赖 | 复用总册 §4.5.2 25 module 联动矩阵 "notification" 行 |
| 优先级 | P1 |

**用户故事**: US-W17

**验收标准**: AC-W8.3 — Flow 失败后, 所有者在 30s 内收到 1 条通知

**已知缺口**: 无

### 4.9 W9 执行历史与调试 (3 项)

#### 4.9.1 FR-W9.1 执行历史表 (Transaction, append-only)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W9.1 |
| 描述 | 每次 Execution 写 1 条 `execution_history` 记录 (Transaction) + 每个节点写 1 条 `execution_step` 记录 (Transaction), 均 append-only, 不可修改/删除, 类比既有 `AuditEvent` 设计 |
| 数据 schema 增项 | `execution_history` 表 + `execution_step` 表, 均 W/T/M 归类 = Transaction (per 守门 #13) |
| 优先级 | P0 |

**用户故事**: US-W18

**验收标准**: AC-W9.1 — 每次运行后可在历史列表看到 1 条记录, 展开后看到每个节点的输入/输出/耗时/状态

**已知缺口**: 无

#### 4.9.2 FR-W9.2 从失败节点重跑

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W9.2 |
| 描述 | 对于失败的 Execution, 提供"从失败节点重跑"操作: 复用已成功节点的历史输出作为输入, 只重新执行失败节点及其下游, 不重跑已成功节点 |
| 数据 schema 增项 | 新 Execution 记录 `resumed_from_execution_id` (指向原失败记录) |
| 优先级 | P0 |

**用户故事**: US-W19

**验收标准**: AC-W9.2 — 3 节点流程在第 3 节点失败后重跑, 只有第 3 节点重新执行, 前 2 节点直接复用历史输出

**已知缺口**: 若上游节点为非幂等动作 (如 `create_worktree`), 复用历史输出是否总是安全, 留 Design Doc 评估 (标记为 P0 阶段前须澄清项, 见 §10)

#### 4.9.3 FR-W9.3 单节点调试面板

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W9.3 |
| 描述 | 双击某次 Execution 中的某个 execution_step, 侧栏展开该节点的完整输入/输出 JSON (格式化 + 可折叠) |
| 优先级 | P1 |

**用户故事**: US-W20

**验收标准**: AC-W9.3 — 点击节点执行记录, JSON 输入输出可读、可复制

**已知缺口**: 无

### 4.10 W10 激活状态与版本管理 (3 项)

#### 4.10.1 FR-W10.1 Active/Inactive 开关

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W10.1 |
| 描述 | 每个 `automation_flow` 有 `enabled: boolean` 字段 (复用既有 `AutomationRule.enabled` 语义), 仅 `enabled=true` 的 Flow 的触发节点会响应真实触发事件; `enabled=false` 时仍可手动运行调试 |
| 数据 schema 增项 | `automation_flow.enabled` |
| 业务规则 (v1.1 新增, per 守门 #1 禁回溯叙事 — 显式记录本条对 W10.1 原始范围的追加) | 若 Flow 中存在 `is_placeholder=true` 且未绑定真实 `agent_id` 的节点, 禁止将该 Flow 设为 `enabled=true`; 详见 §4.14.5 FR-W14.5 激活前置校验 |
| 优先级 | P0 |

**用户故事**: US-W21, US-W32

**验收标准**: AC-W10.1 — 停用后, schedule/webhook/canvas_event 触发不再生效; 手动触发仍可用于调试; 尝试激活含未绑定 `agent_id` 的占位节点的 Flow 时被拒绝并提示具体节点 (v1.1 新增, 对应 §4.14.5)

**已知缺口**: 无

#### 4.10.2 FR-W10.2 版本历史 (SCD Type 2)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W10.2 |
| 描述 | `automation_flow` 定义变更时, 生成新版本记录而非原地覆盖, 保留完整版本历史, 归类 Master + SCD Type 2 (per 守门 #13) |
| 数据 schema 增项 | `automation_flow_versions` 表 (Master, SCD Type 2: `valid_from` / `valid_to` / `is_current`) |
| 优先级 | P1 |

**用户故事**: US-W22

**验收标准**: AC-W10.2 — 每次保存生成新版本, 历史版本只读可查看 diff

**已知缺口**: 无

#### 4.10.3 FR-W10.3 回滚

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W10.3 |
| 描述 | 从版本历史选择任意历史版本, 一键回滚为当前生效版本 (生成新版本记录, 内容 = 所选历史版本, 而非物理删除中间版本, 保持 SCD2 append-only 语义) |
| 优先级 | P1 |

**用户故事**: US-W22

**验收标准**: AC-W10.3 — 回滚后当前生效定义与所选历史版本一致, 版本号继续递增 (不复用旧版本号)

**已知缺口**: 无

### 4.11 W11 标签绑定任务卡 (5 项, **issue 核心诉求**)

#### 4.11.1 FR-W11.1 流程 ↔ 标签绑定

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W11.1 |
| 描述 | `automation_flow` 新增 `tag_binding_expr: string \| null` 字段, 存储 1 条标签表达式 (语法见 W11.5), 在面板管理界面配置; 绑定后系统持续监控 Flow 自身的 `tags: string[]` 字段是否命中表达式 |
| 数据 schema 增项 | `automation_flow.tags: string[]` (Flow 自身标签) + `automation_flow.tag_binding_expr` |
| 业务规则 | BR-W-1 |
| 优先级 | P0 |

**用户故事**: US-W23

**验收标准**: AC-W11.1 — Flow 详情页可设置 tags 与 tag_binding_expr, 配置后系统开始监控命中状态

**已知缺口**: 无

#### 4.11.2 FR-W11.2 派生任务卡生成规则

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W11.2 |
| 描述 | Flow 的 `tags` 命中 `tag_binding_expr` 时, 系统自动创建 1 条 WorkItem: `title` = Flow 名称, `description` = Flow 摘要 (节点数/触发类型), `labels` = 命中的标签表达式涉及的具体标签值, `kind` 复用既有 `WorkItemKind` 新增枚举值 `automation_flow_card`, `sprint_id` 缺省 (进 Backlog), `source_kind = "workflow_tag_binding"` + `source_flow_id` 指回该 Flow |
| 数据 schema 增项 | `WorkItemKind` 枚举扩展 `automation_flow_card` + `WorkItem.source_kind` + `WorkItem.source_flow_id` |
| 业务规则 | BR-W-2, BR-W-4, BR-W-5 |
| 优先级 | P0 |

**用户故事**: US-W23

**验收标准**: AC-W11.2 — 绑定命中后 Backlog 立即出现 1 张新任务卡, 卡片可点开查看关联的 source Flow

**已知缺口**: 无

#### 4.11.3 FR-W11.3 任务卡持久化身份判定 (**设计决策落地**)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W11.3 |
| 描述 | **判定结论: 派生任务卡是真实持久化 WorkItem (Master, SCD Type 2), 不是只读派生视图/投影。** 理由: (1) 拖入 Sprint 需要写 `sprint_id`, 派生视图无法承载写操作; (2) Sprint burndown / velocity 等既有图表 (`Chart01Burndown` 等) 直接读 `workItems` 数组统计 `story_points`, 若是投影则无法参与统计; (3) 用户可能对派生卡做人工编辑 (指派人/优先级), 需要可写字段承载 (per BR-W-5 系统字段与用户字段分区) |
| 业务规则 | BR-W-2, BR-W-5 |
| 优先级 | P0 |

**用户故事**: US-W23

**验收标准**: AC-W11.3 — 派生任务卡在 Sprint burndown / velocity 图表中与人工创建任务卡同等参与统计

**已知缺口**: 无 (本项即缺口澄清本身)

#### 4.11.4 FR-W11.4 标签变更传播 (增/删任务卡)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W11.4 |
| 描述 | 面板管理界面修改 Flow 的 `tags` 或 `tag_binding_expr` 后, 触发 W2.4 `canvas_event` 同源事件流, 系统重新评估命中状态: 新命中 → 走 FR-W11.2 生成; 原命中变为不命中 → 走 BR-W-3 三段式规则处理 (Backlog 硬删 / planned Sprint 移回 Backlog 再硬删 / active Sprint 标记 detached) |
| 业务规则 | BR-W-3 (核心裁决规则) |
| 优先级 | P0 |

**用户故事**: US-W25, US-W26

**验收标准**: AC-W11.4 — 3 种卡片生命周期状态下的标签解绑行为均符合 BR-W-3 定义, e2e 用例逐一覆盖

**已知缺口**: 无

#### 4.11.5 FR-W11.5 标签表达式语法 (AND/OR/NOT)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W11.5 |
| 描述 | `tag_binding_expr` 支持简单布尔表达式: `tag:"a" AND tag:"b"` / `tag:"a" OR tag:"b"` / `NOT tag:"c"`, 可组合 (如 `tag:"a" AND (tag:"b" OR tag:"c")`), 避免"1 标签 = 1 卡"导致标签爆炸 |
| 优先级 | P1 |

**用户故事**: US-W23

**验收标准**: AC-W11.5 — 3 种布尔算子及括号组合均正确求值命中/不命中

**已知缺口**: v1 表达式作用于 **Flow 整体标签**, 不支持 node 级标签继承 (per §1.4 已排除)

### 4.12 W12 Backlog / Sprint 联动 (5 项, **issue 核心诉求**)

#### 4.12.1 FR-W12.1 派生任务卡默认落 Backlog

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W12.1 |
| 描述 | FR-W11.2 生成的 WorkItem 的 `sprint_id` 字段留空 (`undefined`), 复用既有 `SprintBoardView.tsx` line 511 `backlogItems = filtered.filter(w => !w.sprint_id)` 判定逻辑, **无需任何代码改动即天然出现在 Backlog** |
| 业务规则 | — |
| 优先级 | P0 |

**用户故事**: US-W23

**验收标准**: AC-W12.1 — 新生成的派生任务卡出现在 Sprint 视图左侧 Backlog 列, 与人工卡片混排

**已知缺口**: 无

#### 4.12.2 FR-W12.2 拖拽进 Sprint (复用既有机制, 零改动)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W12.2 |
| 描述 | 派生任务卡是普通 WorkItem (per FR-W11.3), 天然可被既有 `@dnd-kit` 拖拽逻辑 (`addToSprint`) 拖入任意 Sprint 列, 无需为派生卡新增特殊拖拽规则 |
| 业务规则 | — |
| 优先级 | P0 |

**用户故事**: US-W24

**验收标准**: AC-W12.2 — 派生任务卡可像人工卡片一样被拖入 Sprint, `sprint_id` 正确写入

**已知缺口**: 无

#### 4.12.3 FR-W12.3 在制 Sprint 标签变更规则 (**核心冲突裁决落地**)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W12.3 |
| 描述 | 落地 BR-W-3 (c) 分支: 派生任务卡所在 Sprint 状态为 `active` 时, 标签解绑触发 `tag_binding_status = "detached"` 字段写入, Sprint 看板该卡片渲染红色 "🔗 已脱离标签" 角标 (类比既有 `PRIORITY_COLOR` 色码扩展), 且**不**从 Sprint 移除、**不**删除该 WorkItem; 待该 Sprint 通过 `completeSprint` / `cancelSprint` 走既有"未完成卡片 sprint_id 置空回 Backlog"流程后, 若届时仍不命中标签, 再走 FR-W11.4 Backlog 硬删规则 |
| 数据 schema 增项 | `WorkItem.tag_binding_status: "bound" \| "detached" \| null` |
| 业务规则 | BR-W-3 |
| 优先级 | P0 |

**用户故事**: US-W25

**验收标准**: AC-W12.3 — active Sprint 中的派生卡标签解绑后, 卡片仍在原 Sprint 列, 显示 detached 角标; Sprint 完成/取消后按既有规则回 Backlog, 之后立即被 FR-W11.4 硬删

**已知缺口**: `detached` 角标是否需要额外的 Lead 人工"确认接受/恢复绑定"交互, 留 Design Doc 阶段结合 5 域 Lead 反馈细化 (v1 先做只读提示)

#### 4.12.4 FR-W12.4 面板管理 UI (标签绑定关系总览)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W12.4 |
| 描述 | 新增管理面板 (页面路径待定, 建议 `/automation` 页扩展 1 个 "Flow Tags" tab), 表格展示 "标签表达式 → Flow → 派生任务卡" 三级绑定关系, 支持在此页直接增删 Flow 的 tags |
| 接口依赖 | 复用既有 `/automation` 页面 UI 组件 (Stat / 表格) |
| 优先级 | P1 |

**用户故事**: US-W27

**验收标准**: AC-W12.4 — 面板列出所有已绑定 Flow, 每行显示当前命中状态 (bound/unbound) 与对应任务卡链接

**已知缺口**: 无

#### 4.12.5 FR-W12.5 人工编辑覆盖保护 (BR-W-5 落地)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W12.5 |
| 描述 | 派生任务卡的字段分 2 类: **系统字段** (`title`/`description`/`labels`, 每次 Flow 变更同步覆盖) 与 **用户字段** (`assignee_id`/`priority`/`story_points`/`status`/`due_date`, 一旦用户手动改过则永久保留, 不再被系统同步覆盖), 用 `WorkItem.user_edited_fields: string[]` 记录哪些字段已被人工接管 |
| 数据 schema 增项 | `WorkItem.user_edited_fields: string[]` |
| 业务规则 | BR-W-5 |
| 优先级 | P1 |

**用户故事**: US-W24

**验收标准**: AC-W12.5 — 用户改过 `priority` 后, 即便 Flow 名称变了, 该卡 `priority` 不被系统同步覆盖; 未改过的 `title` 仍跟随 Flow 名称同步

**已知缺口**: 无

### 4.13 W13 数据一致性与状态映射 (2 项)

#### 4.13.1 FR-W13.1 执行状态 ↔ WorkItem.status 显式可选映射

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W13.1 |
| 描述 | 默认 (per BR-W-7) Flow 执行状态与派生任务卡的 `status` 完全解耦, 互不影响; 面板管理提供**可选**映射配置 (如 "Execution succeeded → WorkItem.status = done"), 关闭时维持默认解耦 |
| 数据 schema 增项 | `automation_flow.status_mapping: Record<string, WorkItemStatus> \| null` |
| 业务规则 | BR-W-7 |
| 优先级 | P1 |

**用户故事**: US-W28

**验收标准**: AC-W13.1 — 未配置映射时, Flow 跑失败不改变卡片 status; 配置映射后按规则联动

**已知缺口**: 无

#### 4.13.2 FR-W13.2 "1 task 1 sprint" 规则对派生卡同等适用

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W13.2 |
| 描述 | 派生任务卡与人工任务卡共用同一 `WorkItem.sprint_id` 单值字段 (非数组), 天然满足既有 "1 task 1 sprint" 约束, 无需为派生卡新增例外分支 |
| 业务规则 | 复用既有约束, 不新增 |
| 优先级 | P0 |

**用户故事**: US-W24

**验收标准**: AC-W13.2 — 派生任务卡任意时刻只能属于 0 或 1 个 Sprint

**已知缺口**: 无

### 4.14 W14 默认工作流模板库 (7 项, **v1.1 新增**, per 追评 1 `01a09566-3f2d-7dc0-81c6-1ff46b89f056`)

本子能力是 §4.7.2 FR-W7.2 (流程模板库通用机制) 的具体化落地, 不新增第二套模板机制; 关键设计决策: "agent 占位" 复用既有 W3.1 `dispatch_agent` 动作 (不新增 element kind), 仅新增 `is_placeholder` / `placeholder_role_hint` 两个字段, 因此本子能力不改动 W1 的 4 类节点体系。

#### 4.14.1 FR-W14.1 默认模板选择器 (3 套内置模板)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.1 |
| 描述 | "+ Flow" 入口的模板选择器 (per FR-W7.2 已有入口) 内置 3 套只读模板: (a) AAA 游戏资产管线, (b) 编程 spec 式, (c) superpowers 式; 每套模板附简介 + 缩略节点图预览 |
| 数据 schema 增项 | 新增 `flow_template` 表 (Master, `is_builtin: boolean`) |
| 业务规则 | — |
| 优先级 | P1 |

**用户故事**: US-W29 + US-W30 + US-W31

**验收标准**: AC-W14.1 — 模板选择器展示 3 套内置模板, 每套可预览缩略节点图

**已知缺口**: 无

#### 4.14.2 FR-W14.2 AAA 游戏资产管线模板 (具体节点清单)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.2 |
| 描述 | 套用后生成链路: `manual` 触发 → Agent 占位"图生高模" → Agent 占位"绑骨骼 (rigging)" → Agent 占位"高模转低模 (retopology)" → Agent 占位"绑定动作 (animation)" → 并行 N 条 Agent 占位"贴图生成" (diffuse/normal/roughness/AO 各 1 节点, per W3.3 并行编排) → Merge 汇合 (per W4.3) → 结束 |
| 数据 schema 增项 | 复用 `flow_template.definition: { nodes: FlowNode[], edges: FlowEdge[] }` (JSON 快照) |
| 业务规则 | 各 Agent 占位节点默认 `is_placeholder = true`, `placeholder_role_hint` 分别写入 "图生高模" / "绑骨骼" / "低模化" / "动作绑定" / "贴图生成" |
| 优先级 | P1 |

**用户故事**: US-W29

**验收标准**: AC-W14.2 — 套用该模板后画布生成 5 个串联占位节点 + 1 组并行贴图占位节点 + 1 个 Merge 节点, 边与数据流预连好 (上游输出字段已映射到下游输入)

**已知缺口**: "图生高模"/"绑骨骼"/"贴图生成"等具体 AI 生成能力的真实执行逻辑不在本 SRS 范围内 (v1 仅占位, 真实生成能力对接属 Agent Runtime / 具体 3D 生成服务范畴, 待专门 SRS)

#### 4.14.3 FR-W14.3 编程 spec 式工作流模板

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.3 |
| 描述 | 套用后生成链路: `manual` 触发 → Agent 占位"需求 spec 撰写" → Condition (spec 评审是否通过, `routing_mode="static_cel"` 默认) → [通过] Agent 占位"实现开发" → Agent 占位"测试" → Agent 占位"代码评审" → 结束; [不通过] 边回指"需求 spec 撰写"节点形成修订循环 |
| 数据 schema 增项 | 同 W14.2, 复用 `flow_template.definition` |
| 业务规则 | 回指边形成的循环由用户在编辑态自行确认退出条件, 不做自动死循环检测 (与 BR-W-6 子流程环检测是两个不同层面, 此处是普通 Flow 边循环, 留 §10 已知风险) |
| 优先级 | P1 |

**用户故事**: US-W30

**验收标准**: AC-W14.3 — 套用该模板后画布生成 5 个占位节点 (含 1 个条件节点) + 1 条评审不通过回指边

**已知缺口**: 无

#### 4.14.4 FR-W14.4 superpowers 式工作流模板

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.4 |
| 描述 | 套用后生成链路 (阶段划分参照 superpowers 方法论的头脑风暴→写计划→执行→评审→收尾节奏): `manual` 触发 → Agent 占位"brainstorming" → Agent 占位"writing-plans" → Agent 占位"executing-plans (含 TDD)" → Agent 占位"code-review" → Condition (评审是否通过) → [通过] Agent 占位"finishing-branch" → 结束; [不通过] 边回指"executing-plans"节点 |
| 数据 schema 增项 | 同 W14.2 |
| 业务规则 | 本模板仅代表**方法论阶段划分**的占位节点图, 不隐含对任何具体 agent 技能包的调用契约 (per §1.4 已声明的排除项) |
| 优先级 | P1 |

**用户故事**: US-W31

**验收标准**: AC-W14.4 — 套用该模板后画布生成 5 个占位节点 (含 1 个条件节点) + 1 条评审不通过回指边

**已知缺口**: 阶段命名与真实 agent 技能包 (如某具体 skill 命名规范) 的对应关系待 Design Doc 阶段与技能体系维护方对齐, v1 仅用阶段占位名称, 不做强绑定

#### 4.14.5 FR-W14.5 模板套用自动预置节点/边/数据流 (核心诉求)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.5 |
| 描述 | 用户选中任一模板 (内置或自定义) 后, 系统**一次性**在新建的 `automation_flow` 下实例化 `flow_template.definition` 中的全部节点 (含 Agent 占位) + 边 + `data_mapping` (per W6.1 数据流引用), 用户无需手动重建任一节点/连线 |
| 数据 schema 增项 | `flow_node.is_placeholder: boolean` (新增字段) + `flow_node.placeholder_role_hint: string \| null` (新增字段) |
| 接口依赖 | 复用 W1.1-W1.2 节点/边渲染 + W6.1 数据流映射机制, 批量实例化而非单节点创建 |
| 业务规则 | 占位节点在未绑定具体 `agent_id` 前, 该节点可保存但**不可激活运行** (per W10.1 激活前置校验新增一条: 存在 `is_placeholder=true` 节点时禁止 active) |
| 优先级 | P0 |

**用户故事**: US-W32

**验收标准**: AC-W14.5 — 套用模板后, 画布一次性出现模板定义的全部节点与边, 数据流字段已预映射; 尝试激活含未绑定占位节点的 Flow 时被拒绝并提示具体节点

**已知缺口**: 无

#### 4.14.6 FR-W14.6 用户调整后另存为自定义模板

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.6 |
| 描述 | 用户在套用内置模板并调整节点参数/占位 agent 后, 可"另存为自定义模板" (不覆盖内置模板, `is_builtin=false`), 供本人或团队后续复用 |
| 数据 schema 增项 | `flow_template.created_by` / `flow_template.tenant_id` |
| 业务规则 | 内置模板 (`is_builtin=true`) 只读, 不可编辑或删除, 仅可"另存为" |
| 优先级 | P1 |

**用户故事**: US-W32

**验收标准**: AC-W14.6 — 调整后点击"另存为模板", 生成 1 条 `is_builtin=false` 的新模板记录, 内置模板本身不受影响

**已知缺口**: 无

#### 4.14.7 FR-W14.7 模板库管理 (内置只读 + 自定义 CRUD)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W14.7 |
| 描述 | 模板库列表页支持: 查看全部模板 (内置 + 自定义) / 自定义模板改名/删除 / 按名称搜索; 内置模板不提供删除入口 |
| 接口依赖 | per §8.2 新增 `flow_template` CRUD 端点 |
| 业务规则 | — |
| 优先级 | P2 |

**用户故事**: US-W32

**验收标准**: AC-W14.7 — 自定义模板可改名/删除, 内置模板无删除按钮

**已知缺口**: 无

### 4.15 W15 LangGraph 智能控制 + 底部聊天栏 (5 项, **v1.1 新增**, per 追评 2 `01a09567-5720-7500-a870-9cff243e8102`)

本子能力对接既有 `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` L0/L1 + TMO 架构, 不重新设计一套并行的智能控制系统; "L1↔L1 通信禁止" (守门 #13 a 派生, per `AGENTS.md` L0/TMO 架构记述, 非 §4 独立编号条款) 约束同样适用于本 SRS 的动态路由 — 动态路由决策统一经 L0 做出, 不允许 Flow 内不同 Agent 占位节点互相直接通信决策。

#### 4.15.1 FR-W15.1 补齐画布底部聊天栏

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W15.1 |
| 描述 | 画布视图 (`/board` 或 Flow 编辑视图) 底部新增常驻聊天栏 UI (非画布 element, 类比 §4.9.1 执行历史面板的处理方式), 落地 `frontend/src/lib/store.ts` line 565 `// TODO: 接入 L0 chat bar session` 既有预留点; 聊天栏与既有 `CommandBar.tsx` (⌘K 命令面板) 是两个独立组件, 不合并 |
| 数据 schema 增项 | 新增 `chat_session` 表 (Transaction, 对应 `actor_session_id`) |
| 接口依赖 | 复用 ADR-0046 `actor_session_id` 字段与 L0 TopAgentState 会话机制 |
| 业务规则 | — |
| 优先级 | P0 |

**用户故事**: US-W33

**验收标准**: AC-W15.1 — 画布底部出现常驻输入栏; 发送消息后 `store.ts` 的 `actor_session_id` 不再为 `undefined`

**已知缺口**: 聊天栏是否需要跨页面 (画布外) 持久悬浮, 待 Design Doc 决策

#### 4.15.2 FR-W15.2 自然语言创建/编辑 Flow

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W15.2 |
| 描述 | 用户在聊天栏输入自然语言描述 (如"每天 9 点检查 P0 缺陷并通知我"), 系统解析后生成对应的 `flow_node` + `flow_edge` 图, 落入当前 Flow 编辑视图供用户确认/调整, 不直接静默保存 |
| 数据 schema 增项 | `chat_session.parsed_flow_draft: Record<string, unknown> \| null` |
| 接口依赖 | per §8.2 新增 NL→Flow 解析端点 |
| 业务规则 | per 守门 #23 v2, v1 解析层为 **mock/规则化实现** (关键词 + 模板匹配, 非真实 LLM API 调用), 真实 LLM 语义解析留 P2 (类比 `SRS-CANVAS-GAMIFY-001` G5) |
| 优先级 | P1 |

**用户故事**: US-W33

**验收标准**: AC-W15.2 — 输入符合 mock 规则的自然语言描述后, 编辑视图内出现对应草稿节点图, 用户确认后才正式保存

**已知缺口**: mock 规则的覆盖范围 (能正确解析哪些句式) 待 Design Doc 列出具体规则表; 真实 NLU/LLM 接入时间点待 P2 阶段拍板

#### 4.15.3 FR-W15.3 LangGraph 式动态路由

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W15.3 |
| 描述 | 条件节点新增 `routing_mode: "static_cel" \| "dynamic_agent"`; 选择 `dynamic_agent` 时, 分支走向由 L0 依据当前上下文 (`TopAgentState`) 在执行时动态决定, 而非预先写死的 CEL 表达式, 与 W4 静态分支并存, 用户按需选择 |
| 数据 schema 增项 | `flow_node.routing_mode` (新增字段, 默认 `"static_cel"` 保持向后兼容) |
| 接口依赖 | 复用 ADR-0046 `TopAgentState.active_tmo_operation` 语义, 不新增并行状态机 |
| 业务规则 | 动态路由决策必须经 L0 做出, 禁止 Flow 内 Agent 占位节点间直接通信决策 (per 守门 #13 a 派生 "L1↔L1 通信禁止" 同源约束) |
| 优先级 | P1 |

**用户故事**: US-W34

**验收标准**: AC-W15.3 — `routing_mode="dynamic_agent"` 的条件节点在执行历史 (`execution_step`) 中记录 L0 给出的路由决策依据, 便于事后审计

**已知缺口**: "动态路由决策依据"记录粒度 (是否需要记录完整 L0 推理过程还是仅记录最终分支选择) 待 Design Doc 拍板; v1 动态路由决策底层同样为 mock 规则化实现, 真实智能判断依赖 P2 真实 LLM 接入 (per §1.4 排除项)

#### 4.15.4 FR-W15.4 聊天栏与 L0/TMO 会话集成

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W15.4 |
| 描述 | 聊天栏发送的每条消息绑定 1 个 `actor_session_id`, 与既有 L0 TopAgentState 会话生命周期一致 (开启/结束), 聊天栏创建的 Flow 的执行记录 (`execution_history`) 需可回链到发起该 Flow 的 `chat_session` |
| 数据 schema 增项 | `execution_history.origin_chat_session_id: Uuid \| null` |
| 接口依赖 | 复用 `/api/tmo/*` 既有 8 端点, 不新增并行的会话管理端点 |
| 业务规则 | — |
| 优先级 | P0 |

**用户故事**: US-W34

**验收标准**: AC-W15.4 — 由聊天栏创建的 Flow 执行后, 执行历史记录中可追溯到发起对话的 `chat_session_id`

**已知缺口**: 无

#### 4.15.5 FR-W15.5 聊天栏产物复用既有 Flow schema (不建平行系统)

| 项 | 内容 |
|---|---|
| ID | FR-WORKFLOW-W15.5 |
| 描述 | 聊天栏生成的 Flow 草稿必须是标准 `flow_node`/`flow_edge` 记录 (与 W1-W14 手动搭建/模板套用产出的 Flow 完全同构), 可在同一 Flow 编辑器中被手动继续编辑, 不引入专属"聊天栏 Flow"数据结构 |
| 业务规则 | 与 W14.5 "模板批量实例化"复用同一批量写入接口, 语义上"聊天栏"是 Flow 的第 3 种创建入口 (手动拖拽 / 模板套用 / 自然语言), 而非第 3 套数据模型 |
| 优先级 | P1 |

**用户故事**: US-W33

**验收标准**: AC-W15.5 — 聊天栏生成的 Flow 节点在编辑视图中与手动创建节点行为完全一致 (可拖动/改参数/删除)

**已知缺口**: 无

---

## §5 约束条件 (Constraints)

### 5.1 守门合规 (per AGENTS.md §4)

| 守门编号 | 合规点 |
|---|---|
| 守门 #1 禁回溯叙事 | §0.3 显式记录本 SRS 对总册 §1.4 "Miro Flowchart / 通用集成" 砍掉决定的反转, 未静默重写 |
| 守门 #11 缺标比错标 | §1.4 不包含范围 + 每项 FR "已知缺口" 字段 + §10 已知风险均显式列出, 不留隐性假设 |
| 守门 #13 W/T/M 三类横展 | §7.2 逐表标注 Work/Transaction/Master 分类 |
| 守门 #13 a 派生 (L1↔L1 通信禁止, per `AGENTS.md` §6 ADR 索引 + ADR-0046, 非 §4 独立编号条款) | W15.3 动态路由决策显式约束为经 L0 做出, 不允许 Agent 占位节点间直接通信决策 |
| 守门 #14 签字栏 | §11 沿用 5 角色签字结构 |
| 守门 #23 v2 禁真实第三方 AI API 调用 (**v1.1 新增**) | W15.2 NL→Flow 解析、W15.3 动态路由决策底层 v1 均为 mock/规则化实现, 真实 LLM 接入留 P2, 详见 §1.4 / §10 |

### 5.2 技术约束 (per V0.1 既有栈)

沿用总册 §5.2, 不重复; 新增: 本 SRS 的图执行引擎在 v1 阶段为**前端 zustand mock 模拟**, 不涉及真实分布式任务队列。

### 5.3 业务约束

复用总册 §5, 新增 BR-W-1 至 BR-W-7 (per §3.3)。

### 5.4 安全 / 合规约束

Webhook token (FR-W2.3) 与 HTTP 请求动作 (FR-W3.2) 涉及外部网络调用, 复用总册 NFR-SEC-1 (凭据不落库/不打印) 约束, 具体加密存储方案属于 Design Doc 范畴。

### 5.5 组织约束

复用总册 §5.5, 不重复。

---

## §6 业务场景 (Use Cases)

### 6.1 主要场景

**场景 1: PM 用标签把一套发布自动化打包成任务卡并排入冲刺** (对应 issue 原文诉求)

1. PM 在画布上搭建 1 个 Flow: `schedule_cron` 触发 → IF (是否周五) → `call_webhook` 通知发布群 → `create_worktree`
2. PM 给该 Flow 打标签 `tags: ["release-checklist", "phase-2"]`, 并设置 `tag_binding_expr = 'tag:"release-checklist" AND tag:"phase-2"'`
3. 表达式立即命中 (FR-W11.1), 系统在 Backlog 生成 1 张任务卡 "release-checklist Flow" (FR-W11.2)
4. PM 打开 Sprint 视图, 把该卡从 Backlog 拖入本 Sprint (FR-W12.2, 零改动复用既有拖拽)
5. Sprint 进入 active 后, PM 发现该 Flow 已过时, 在面板管理里把 `phase-2` 标签摘掉 (FR-W12.4)
6. 表达式不再命中, 但卡片已在 active Sprint 中 → 卡片**不被删除**, 显示"🔗 已脱离标签"角标 (FR-W12.3, BR-W-3(c))
7. Sprint 结束时, 该未完成卡片按既有规则回 Backlog, 系统随即因标签仍不命中而将其硬删除 (FR-W11.4)

**场景 2: SRE 排查一次失败的自动化执行**

1. SRE 打开执行历史面板 (FR-W9.1), 看到某次 Execution 在第 3 个节点 (`http_request`) 失败
2. 展开该节点执行记录, 看到输入/输出 JSON, 定位到是超时 (FR-W9.3)
3. SRE 直接点击"从失败节点重跑" (FR-W9.2), 前 2 个节点 (触发 + 数据转换) 复用历史输出, 只重跑第 3 个节点

**场景 3 (v1.1 新增): 美术/TA 套用 AAA 模板并绑定占位 agent** (对应追评 1 诉求)

1. 美术/TA 打开模板选择器, 选中"AAA 游戏资产管线"模板 (FR-W14.1)
2. 系统一次性生成 5 个串联占位节点 + 1 组并行贴图占位节点 + 1 个 Merge 节点, 节点间数据流已预映射 (FR-W14.5)
3. 美术/TA 尝试直接激活该 Flow, 系统检测到占位节点未绑定具体 `agent_id`, 拒绝激活并逐个提示 (FR-W14.5 业务规则)
4. 美术/TA 为每个占位节点指派具体 agent, 再次激活成功
5. 美术/TA 微调"贴图生成"节点的分辨率参数, 另存为自定义模板 "AAA-4K 贴图版" 供团队复用 (FR-W14.6)

**场景 4 (v1.1 新增): PM 用聊天栏自然语言创建 Flow 并接受 LangGraph 动态路由** (对应追评 2 诉求)

1. PM 在画布底部聊天栏输入"每天 9 点检查 P0 缺陷并通知我" (FR-W15.1)
2. 系统 (v1 mock 规则化解析) 生成对应草稿: `schedule_cron` 触发 → 数据查询 → IF (是否存在 P0 缺陷) → `send_notification` (FR-W15.2)
3. PM 确认草稿, Flow 正式保存, 与手动搭建的 Flow 完全同构, 可继续手动编辑 (FR-W15.5)
4. PM 把 IF 节点的 `routing_mode` 改为 `dynamic_agent`, 交由 L0 依据当时上下文动态判断是否需要额外升级通知 (FR-W15.3)
5. 该 Flow 每次执行的 `execution_history` 均可回链到发起对话的 `chat_session_id` (FR-W15.4)

### 6.2 异常场景

| 场景 | 处理 |
|---|---|
| 派生任务卡在 Backlog 时被用户手动删除, 之后标签仍命中 | 系统检测到 `source_flow_id` 已无对应存活 WorkItem, 重新生成新卡 (视为用户误删的自愈, 而非"删除即永久排除") — **已知缺口**: 是否需要"用户手动排除某 Flow 不再自动生成卡"的开关, 留 P1 拍板 (per §10) |
| 子流程 A 引用子流程 B, B 又引用 A (循环引用) | 保存时 DFS 环检测, 拒绝保存并报错提示环路径 (per BR-W-6) |
| Loop 节点数组源为空 | 循环体 0 次执行, 直接继续下游, 不报错 |
| 标签表达式语法错误 (如括号不匹配) | 保存时校验拒绝, 提示具体语法错误位置 |
| 两个不同 Flow 的 `tag_binding_expr` 同时命中同一组标签值 | v1 各自独立生成各自的派生卡 (1 Flow 1 卡, per BR-W-1), 不做跨 Flow 去重, 用户需自行避免表达式重叠 (已知缺口, 列入 §10) |

---

## §7 数据需求 (Data Requirements)

### 7.1 输入数据 (Store Schema 增项汇总)

| 表/字段 | 新增 or 扩展 | 说明 |
|---|---|---|
| `automation_flow` | 新增表 | Flow 定义主体 |
| `automation_flow_versions` | 新增表 | SCD2 版本历史 |
| `flow_node` | 新增表 | 流程节点 |
| `flow_edge` | 新增表 | 流程边 |
| `execution_history` | 新增表 | 单次执行记录 |
| `execution_step` | 新增表 | 单节点执行记录 |
| `AutomationTriggerKind` | 枚举扩展 | + `webhook` + `canvas_event` |
| `AutomationActionKind` | 枚举扩展 | + `http_request` + `transform_data` |
| `WorkItemKind` | 枚举扩展 | + `automation_flow_card` |
| `WorkItem.source_kind` | 字段新增 | 标记派生来源 |
| `WorkItem.source_flow_id` | 字段新增 | 指回源 Flow |
| `WorkItem.tag_binding_status` | 字段新增 | `bound`/`detached`/`null` |
| `WorkItem.user_edited_fields` | 字段新增 | 系统字段/用户字段分区 |
| `flow_template` (**v1.1 新增**) | 新增表 | 模板库主体, per W14.1 |
| `flow_node.is_placeholder` / `flow_node.placeholder_role_hint` (**v1.1 新增**) | 字段新增 | Agent 占位标记, per W14.5 |
| `flow_node.routing_mode` (**v1.1 新增**) | 字段新增 | `static_cel` / `dynamic_agent`, per W15.3 |
| `chat_session` (**v1.1 新增**) | 新增表 | 底部聊天栏会话, per W15.1/W15.4 |
| `chat_session.parsed_flow_draft` (**v1.1 新增**) | 字段新增 | NL→Flow 解析草稿, per W15.2 |
| `execution_history.origin_chat_session_id` (**v1.1 新增**) | 字段新增 | 回链发起对话, per W15.4 |

### 7.2 W/T/M 三类横展 (per 守门 #13, 100% 表覆盖)

| 表 | 分类 | 理由 |
|---|---|---|
| `automation_flow` | **Master** | Flow 定义是业务实体主档, 随 SCD2 演进 |
| `automation_flow_versions` | **Master (SCD Type 2)** | 显式版本历史, `valid_from`/`valid_to`/`is_current` |
| `flow_node` / `flow_edge` | **Master** | 附属于某版本 Flow 定义的结构化子实体, 随 Flow 版本联动 |
| `execution_history` | **Transaction** | 每次运行 1 条, append-only, 不可改 |
| `execution_step` | **Transaction** | 每节点每次运行 1 条, append-only |
| `WorkItem` (含派生卡) | **Master** | 复用既有分类, 派生卡不例外 |
| `flow_template` (**v1.1 新增**) | **Master** | 模板定义主档 (内置只读 + 自定义可写), 结构与 `automation_flow` 同源 |
| `chat_session` (**v1.1 新增**) | **Transaction** | 每次聊天会话/消息 append-only 记录, 类比 `execution_history` |

### 7.3 数据完整性约束

- `flow_node.automation_flow_id` 外键必填, 级联删除 (Flow 删除 → 节点/边级联删除, 但 `execution_history` **不**级联删除, 保留历史审计)
- `WorkItem.source_flow_id` 若指向的 Flow 被删除, 派生卡**不**级联删除, `source_flow_id` 置空并保留卡片 (per 缺标比错标: 避免用户误删 Flow 连带丢失 Sprint 中的任务卡)
- `automation_flow.tag_binding_expr` 保存前必须通过语法校验 (per §6.2 异常场景)

### 7.4 数据流 (Data Flow)

```
Flow.tags 变更
   → canvas_event 触发 (FR-W2.4)
   → 标签绑定重新求值 (FR-W11.1)
   → 命中变化判定 (FR-W11.4)
        → 新命中: 创建 WorkItem (系统字段) (FR-W11.2)
        → 失去命中 + Backlog: 硬删 WorkItem (BR-W-3a)
        → 失去命中 + planned Sprint: sprint_id 置空 → 硬删 (BR-W-3b)
        → 失去命中 + active Sprint: tag_binding_status=detached, 不删除 (BR-W-3c)
   → 用户字段保护检查 (user_edited_fields, FR-W12.5) → 系统字段同步 skip 已接管字段
```

---

## §8 接口需求 (Interface Requirements)

### 8.1 内部接口 (组件 Props, 待 Design Doc 精确化)

| 组件 | 新增 Props / 依赖 |
|---|---|
| `CanvasView.tsx` | 新增 4-6 个 element kind 渲染分支 (per W1.1, W4.3, W5.1) |
| `/automation` 页 | 新增 "Flow" tab (列表 + 编辑入口) + "Flow Tags" tab (per W12.4) |
| 新增 Flow 编辑器组件 (页面路径待定) | 节点拖拽画布, 复用 `CanvasView.tsx` viewport/pan/zoom 基座还是独立组件, 待 Design Doc 决策 (见 §10) |
| 新增底部聊天栏组件 (**v1.1 新增**) | 独立于 `CommandBar.tsx`, 落地 `store.ts` line 565 TODO, per W15.1 |
| 新增模板选择器组件 (**v1.1 新增**) | 挂载于 "+ Flow" 入口旁 (per 既有 FR-W7.2 入口), per W14.1 |

### 8.2 外部接口 (BFF Route, 新增)

| API | 方法 | 路径 | 说明 |
|---|---|---|---|
| Flow CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/flows` | 复用总册 §6.2 Canvas CRUD 模式 |
| Flow Node/Edge CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/flows/{id}/nodes`, `/edges` | 同上 |
| Execution 触发/查询 | POST/GET | `/v1/collaboration/flows/{id}/executions` | 手动触发 + 历史查询 |
| Execution 重跑 | POST | `/v1/collaboration/flows/{id}/executions/{exec_id}/resume` | per FR-W9.2 |
| Webhook 入口 | POST | `/v1/collaboration/flows/{id}/webhook/{token}` | per FR-W2.3 |
| 标签绑定求值 (内部) | — | 复用 canvas_event 事件流, 非独立 REST 端点 | per FR-W2.4 + FR-W11.4 |
| Flow Template CRUD (**v1.1 新增**) | GET/POST/PATCH/DELETE | `/v1/collaboration/flow-templates` | per FR-W14.1/W14.6/W14.7, 内置模板 (`is_builtin=true`) 只读, DELETE 仅对自定义模板开放 |
| 聊天栏消息 + NL→Flow 解析 (**v1.1 新增**) | POST | `/v1/collaboration/chat-sessions/{id}/messages` | per FR-W15.1/W15.2, v1 后端为 mock 规则化解析 (per 守门 #23 v2), 复用/对接 `/api/tmo/*` (ADR-0046) 做 `actor_session_id` 会话管理, 不新增并行会话端点 |

### 8.3 Zustand Store 依赖

| Store slice | 新增/扩展 |
|---|---|
| `automationFlows` | 新增 |
| `flowNodes` / `flowEdges` | 新增 |
| `executionHistory` | 新增 |
| `workItems` | 扩展 (新增字段, per §7.1) |
| `flowTemplates` (**v1.1 新增**) | 新增, per W14 |
| `chatSessions` (**v1.1 新增**) | 新增, per W15.1/W15.4, 落地 `store.ts` line 565 `actor_session_id` TODO |

### 8.4 与既有 25 module 联动接口

复用总册 §6.3 联动矩阵, 新增 1 行:

| 25 module | 画布表现 | 本 SRS 扩展 |
|---|---|---|
| automation | `automation_node` 单规则 | **升级为 Flow 图** (W1-W10), 单规则模式保留向后兼容 (existing `AutomationRule` 视为 1 trigger + N action 的退化 2 层 Flow) |
| work-item | 拖 WorkItem → 画布 element | 新增: 标签绑定自动生成 WorkItem (W11-W12) |

### 8.5 错误处理

复用总册 §5.6 错误处理基线 (max 2 retries 网络层), 节点级 retry (FR-W8.1) 是业务层重试, 与网络传输层重试独立分层, 不互相干扰。

---

## §9 验收标准 (Acceptance Criteria)

### 9.1 功能验收 (54 个, 详见 §4 各项 "验收标准" 字段, 此处汇总统计)

| 优先级 | 项数 (v1.0 W1-W13) | 项数 (v1.1 W14-W15 新增) | 合计 |
|---|---|---|---|
| P0 | 18 | 3 | 21 |
| P1 | 17 | 8 | 25 |
| P2 | 7 | 1 | 8 |
| **合计** | **42** | **12** | **54** |

### 9.2 质量验收

| 项 | 标准 |
|---|---|
| e2e 覆盖 | 场景 1 (标签绑定→Backlog→拖入Sprint→active中解绑→Sprint结束回收) 与场景 2 (失败重跑) 各至少 1 条 e2e |
| e2e 覆盖 (**v1.1 新增**) | 场景 3 (套用 AAA 模板→占位节点全部出现→绑定 agent 前禁止激活) 与场景 4 (聊天栏输入→草稿生成→确认保存) 各至少 1 条 e2e |
| 单元测试 | BR-W-3 三分支 (Backlog/planned/active) 各 1 条单测 |
| 类型检查 | 新增/扩展字段通过 TypeScript strict 编译, 0 error |

### 9.3 文档验收

| 项 | 标准 |
|---|---|
| 总册同步 | `SRS-CANVAS-001` 需同步更新为三核心索引 (per本次配套修订, 见 commit) |
| 术语消歧 | §2.4 "工作流" 消歧表在后续所有引用本 SRS 的文档中需被引用, 不得重新引发歧义 |

---

## §10 已知风险 / 未解决问题 (Known Issues, per 守门 #11 缺标比错标)

| 编号 | 风险/问题 | 影响 | 状态 |
|---|---|---|---|
| 风险 #1 | Flow 编辑器是复用主画布 viewport 还是独立子画布 (FR-W1.3) | 影响 W1-W10 全部 UI 实装路径 | 待 Design Doc 阶段拍板 |
| 风险 #2 | 后端真实持久化引擎缺失 (v1 前端 mock, per §3.2) | Execution 无法跨会话可靠恢复, 真实生产使用前必须补齐 | P0 阻塞, 需真实后端设计 |
| 风险 #3 | FR-W9.2 "从失败节点重跑"对非幂等动作 (`create_worktree`) 的安全性未澄清 | 可能导致重复创建 worktree | 待 Design Doc 评估幂等性标记机制 |
| 风险 #4 | 两个 Flow 的标签表达式重叠命中同一标签组合 (per §6.2 异常场景) | 用户可能得到 2 张语义重复的任务卡 | v1 不做去重, 留 P1 观察真实使用情况后决定是否需要冲突检测 |
| 风险 #5 | detached 状态 (FR-W12.3) 是否需要 Lead 人工确认交互 | 影响 5 域 Lead 的实际处理体验 | v1 先做只读角标提示, 待真人 Lead 反馈后细化 |
| 风险 #6 | A12 多人协同编辑 CRDT 选型未拍板 (per `SRS-CANVAS-AGENT-001` A12.6) | 若 Flow 编辑器需要多人协同, 依赖此项 | 依赖外部 SRS, 本 SRS 不重复设计, 只声明依赖 |
| 风险 #7 | 总册 `SRS-CANVAS-001` 尚未正式收录本专题为"三核心"第 3 核心 (仅本次配套修订 §1.3.1) | 总册 §4.1/§4.4 P0 清单等章节仍以"双核心"措辞为主, 需后续完整修订 | 已在总册做最小同步 (见配套 commit), 完整总册 v1.2 全量改写留后续 |
| 风险 #8 (**v1.1 新增**) | W15.2/W15.3 v1 均为 mock 规则化实现 (per 守门 #23 v2), 真实语义理解/智能判断能力有限, 可能与用户对"LangGraph 智能控制"的实际预期 (真正的 LLM 推理) 有落差 | 真实使用体验可能不及预期, 需在交付说明中明确 v1/P2 边界 | v1 先交付 mock 版本验证交互流程, 真实 LLM 接入时间点待 5 域 Lead / Ulysses 在 P2 阶段拍板 |
| 风险 #9 (**v1.1 新增**) | LangGraph 动态路由 (`routing_mode="dynamic_agent"`) 的决策可复现性/确定性未澄清 — 同一输入两次执行是否应产生相同路由结果 | 影响 e2e 测试稳定性与执行历史的可审计性 | 待 Design Doc 阶段明确: 是否要求同输入同输出 (确定性 mock), 或允许非确定性但需记录决策依据 (per FR-W15.3 已知缺口) |
| 风险 #10 (**v1.1 新增**) | AAA/spec/superpowers 3 套默认模板的具体节点清单 (per W14.2-W14.4) 由 agent 撰写本文档时基于 issue 描述与常见方法论合理推断, 尚未经真人 (issue 创建者 / 5 域 Lead) 逐节点确认 | 实际落地时节点清单可能需调整 | 已在 §11 签字栏标记 Draft 待拍板, 建议评审时逐节点过一遍 §4.14.2-§4.14.4 |
| 风险 #11 (**v1.1 新增**) | FR-W14.3 编程 spec 式模板中"评审不通过"回指边形成的循环, 不做自动死循环检测 (与 BR-W-6 子流程环检测是不同层面 — 后者是子流程引用图, 前者是普通 Flow 边循环), 用户可能手动搭建出无法退出的循环 | 若用户反复"不通过"且无人工干预, Flow 可能无限重跑相同节点, 消耗执行资源 | v1 不做检测, 依赖 FR-W8.1 节点级重试策略上限与人工介入; 是否需要 Flow 级最大循环次数硬限制留 P2 评估 |

---

## §11 签字栏 (5 角色 per AGENTS.md §3 7 段结构 + 守门 #14)

| 角色 | 姓名/代签 | 状态 | 日期 |
|---|---|---|---|
| Ulysses (业务owner) | — | 待拍板 | — |
| 5 域 Lead (跨域, 真人未到位 Mavis 临时代签惯例) | — | 待拍板 | — |
| PM | — | 待拍板 | — |
| SRE | — | 待拍板 | — |
| Dev Lead | — | 待拍板 | — |

**本文档为 ULYS-15 issue 委托的需求文档交付物, 由 agent Sonnet 撰写, 尚未经过上述 5 角色正式拍板, 状态为 Draft, 供审阅与后续拍板使用。**

---

## §12 修订历史

| 版本 | 日期 | 变更摘要 | 作者 |
|---|---|---|---|
| v1.0 | 2026-09-12 | 初版交付, 42 项 FR (W1-W13), 28 用户故事, 含 BR-W-3 核心裁决规则 | Sonnet (agent) |
| v1.1 | 2026-09-12 | 追加 W14 (默认工作流模板库: AAA/spec/superpowers 3 套模板 + agent 占位自动预置 + 自定义另存) + W15 (LangGraph 智能控制 + 补齐底部聊天栏), 42→54 项 FR, 28→34 用户故事; 回应 issue 追评 1/2, 新增风险 #8-#10 | Sonnet (agent) |
