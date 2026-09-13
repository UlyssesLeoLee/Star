# BD-CANVAS-WORKFLOW-001

> **无限画布 — 自动化流程域 (Automation Flow Domain) 基本設計書 v1.0.2** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟢 v1.0.2 — **已签字** (2026-09-13 初版落档; §11 全部 5 角色已由项目所有者 Say世意（D-Boy）本人确认签字, 非代签; §9.3 TBD 追踪矩阵 35 项仍待详细设计阶段逐项拍板, 不因签字栏完成而自动裁决)
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-CANVAS-WORKFLOW-001.md`](../requirements/SRS-CANVAS-WORKFLOW-001.md) **v1.1** (54 项 FR = W1-W13 42 项 v1.0 + W14-W15 12 项 v1.1 新增, 34 用户故事, 8 张新增/扩展表)
> - **重要溯源说明 (per 守门 #1 禁回溯叙事)**: SRS v1.1 于 2026-09-12 在分支 `agent/sonnet/ulys-15` (issue ULYS-15) 落档, **未合并至 `main`, 未推送远端**。本 BD 所在分支 `agent/sonnet/ulys-28` 通过 `git merge agent/sonnet/ulys-15` (commit `8023a34`) 从同一本地仓库 (`D:\Star`) 引入该版本, 而非从 `main` 读取 — `main`/其他 worktree 此时仍只有 v1.0 (42 项 FR)。后续任何读者若在别处只看到 v1.0, 请先确认所在分支是否已合并 `agent/sonnet/ulys-15`, 避免重复此前子代理 (GPT5.6terra) 曾发生的"按 v1.0 误判本文档不合规"问题。
> - 上位总册: [`docs/requirements/SRS-CANVAS-001.md`](../requirements/SRS-CANVAS-001.md) (总册, 索引级, 待总册后续版本正式收录本专题为"三核心"第 3 核心) + [`docs/design/BD-CANVAS-001.md`](./BD-CANVAS-001.md) (root 写总册 BD, 本 BD 专题聚焦 W1-W15 详细设计)
> - 平行专题 BD: [`docs/design/BD-CANVAS-AGENT-001.md`](./BD-CANVAS-AGENT-001.md) (专题 1: agent 管理 46 项) + [`docs/design/BD-CANVAS-GAMIFY-001.md`](./BD-CANVAS-GAMIFY-001.md) (专题 2: 游戏化 32 项) — 本 BD 为**专题 3: 自动化流程**, 与前两者并列, **不属于**总册 `SRS-CANVAS-001` 现行"双核心"功能分类, 仅做索引级交叉引用 (per 本 issue 明确指示, 不强行归类)
> - 关联架构决策: [`docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md`](../architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) (L0/L1 + TMO 架构, `TopAgentState`/`SubAgentState`, `/api/tmo/*` 8 端点, W15 LangGraph 智能控制/聊天栏对接此架构而非另起一套)
> - 关联 V0.1 实装: `frontend/src/components/CanvasView.tsx` (element 渲染基座) + `frontend/src/lib/store.ts` line 565 (`actor_session_id` L0 聊天栏会话预留点) + `frontend/src/components/CommandBar.tsx` (⌘K 命令面板, 与 W15 底部聊天栏是两个独立组件)
> - 撰写者: Sonnet (agent, per Multica ULYS-28 assignment)
> - 修订人/审批: 待 §11 签字栏拍板 (5 角色均 Draft/待拍板状态, 沿用 SRS v1.1 相同签字栏状态)
> - 日期: 2026-09-13
> - 受众: 詳細設計エンジニア / 実装エンジニア / UI/UX 设计师 / アーキテクト / SRE / 5 域 Lead
> - **dual-use 提醒**: 本 BD 不重复总册 BD (`BD-CANVAS-001.md`) 跨域共享部分, 聚焦 W1-W15 54 项详细设计; 本 BD 亦不裁决 SRS §10 已列的 11 项已知风险与各 FR "已知缺口" — 详见 §9 TBD 追踪矩阵, 一律标注待拍板, 不自行假设

---

## §0 目的 (Purpose)

本文档基于 [`SRS-CANVAS-WORKFLOW-001.md`](../requirements/SRS-CANVAS-WORKFLOW-001.md) v1.1 的需求, 定义 **无限画布 — 自动化流程域 (Automation Flow Domain)** 的基本設計:

- 系统架构 (UI / API / 数据流 / LangGraph 路由层) 覆盖 W1-W15 54 项
- 组件一览 + 画面设计 (Flow 编辑器 / 模板选择器 / 底部聊天栏 / 执行历史面板 / 标签绑定管理面板)
- 数据设计 (8 张新增/扩展表定义, W/T/M 分类, SCD/RLS/审计策略)
- 接口设计 (REST + WebSocket 端点, 认证/超时/重试策略)
- 5 View 详细 (机能/データ/動作/モジュール/ネットワーク) 覆盖 54 项 FR
- NFR (性能/可用性/RTO-RPO/安全/运维, 量化指标, 基本设计提案值待拍板)
- 安全设计 (Agent 占位激活前置校验、RBAC、Webhook 鉴权)
- 守门合规 + **TBD/已知缺口追踪矩阵** (11 项 SRS 已知风险 + 逐 FR 已知缺口, 全部显式标注待拍板, 不自行裁决)
- 需求→设计→测试→验收追溯矩阵 (54 项 FR 全覆盖)
- 5 角色签字栏 + 修订履历

**关于 SRS 中【已知缺口】/【待定】/【风险 #1-#11】的处理原则 (per issue 明确指示 + 守门 #11 缺标比错标)**: 本 BD **不**对 SRS 标注为"待 Design Doc 决策"的事项自行拍板 (例如: Flow 编辑器是否复用主画布 viewport、join 超时策略、webhook 签名轮换细节、detached 是否需人工确认、动态路由可复现性要求等)。本 BD 在相应章节给出**候选方案**并明确标注为**【TBD】**, 附影响面说明, 汇总于 §9.2; 最终选型需 5 域 Lead / Ulysses 拍板后回填。

---

## §1 适用范围 (Scope)

### 1.1 包含 (In-Scope, W1-W15 15 子能力, 54 项 FR 逐条对应)

#### 1.1.1 W1 流程节点类型体系 (4 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W1.1 | Dev/PM (画布用户) | 已进入 Flow 编辑模式, `automation_flow_id` 已创建 | 节点 kind 选择 (trigger/action/condition/subflow) | 画布新增 1 个 `flow_node` element (SVG `<g>`, 按 kind 区分图标) | kind 非法值 → 前端拒绝渲染, 不写库 | `flow_node` 记录归属唯一 `automation_flow_id` | P0 |
| FR-WORKFLOW-W1.2 | Dev/PM | 画布上已存在 ≥2 个 `flow_node` | 拖拽连线操作 (from/to 节点) | 新增 `flow_edge` 记录, 条件节点边显示分支标签 | 起止节点相同 (自环) → 前端拒绝创建边 | 边持久化, 参与后续执行图遍历 | P0 |
| FR-WORKFLOW-W1.3 | Dev/PM | 已进入画布, 有 Flow 创建权限 | 点击工具栏 "+ Flow" 按钮 | 新建 1 条空 `automation_flow` 记录并进入编辑态 | 创建请求失败 (网络/权限) → 提示错误, 不进入编辑态 | 用户获得 1 个可编辑的空 Flow | P0 |
| FR-WORKFLOW-W1.4 | Dev/PM | 画布上存在 ≥1 个 `flow_node` | 双击目标节点 | 侧栏展开该节点完整配置 (trigger_kind/condition_expr/action config 等) | 节点数据加载失败 → 侧栏显示错误占位 | 侧栏进入可编辑状态, 修改经校验后写回 | P1 |

#### 1.1.2 W2 触发节点 (4 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W2.1 | Dev/PM | Flow 起点为 `trigger_kind="manual"` | 点击"运行"按钮 | 触发 1 次 Execution, 写 1 条 `execution_history` | 并发重复点击 → 幂等生成多条独立 Execution (per 设计: 手动触发不去重, 【TBD】是否需要防抖) | Execution 状态进入 `running` | P0 |
| FR-WORKFLOW-W2.2 | 系统 (cron 调度器) | Flow `enabled=true` 且节点配置 `schedule_cron` | cron 表达式到点 | 自动触发 1 次 Execution | cron 表达式非法 → 保存时前端/后端双重校验拒绝 | 同上 | P0 |
| FR-WORKFLOW-W2.3 | 外部系统 (webhook 调用方) | Flow `enabled=true`, webhook 节点已生成 `webhook_token` | POST `/v1/collaboration/flows/{flow_id}/webhook/{token}` | 触发 1 次 Execution, 请求体作为触发数据可被下游节点引用 | token 不匹配/已吊销 → 401 拒绝, 不触发; 请求体超限 → 413 | 同上 | P1 |
| FR-WORKFLOW-W2.4 | 系统 (画布事件流) | Flow `enabled=true`, 触发节点配置 `canvas_event` 过滤条件 | 画布 element 增删改/标签变更事件 | 命中过滤条件时触发 Execution | 事件风暴 (高频操作) → 【TBD】是否需 debounce, 见 SRS 已知缺口 | 同上 | P1 |

#### 1.1.3 W3 动作节点 (3 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W3.1 | 系统 (RuleExecutor) | 节点挂载 6 种既有 `AutomationActionKind` 之一 | 上游节点输出 / 静态配置 | 动作执行结果写入 `execution_step.output` | 动作执行失败 → 转入 W8 重试/错误分支 | 节点状态转 `succeeded`/`failed` | P0 |
| FR-WORKFLOW-W3.2 | 系统 (RuleExecutor) | 节点 kind = `http_request` 或 `transform_data` | HTTP 请求参数 / JSONPath 表达式 | HTTP 响应体 / 整形后数据, 供下游引用 | HTTP 超时/5xx → 计入重试策略 (W8.1); JSONPath 语法错误 → 保存时前端校验拒绝 | 同上 | P1 |
| FR-WORKFLOW-W3.3 | Dev/PM | 已有 ≥2 个动作节点 | 顺序/并行连线操作 | 执行时按图遍历顺序/并行调度 | 无 | 并行分支各自独立推进, 顺序分支等前置完成 | P1 |

#### 1.1.4 W4 分支与条件 (3 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W4.1 | 系统 (CEL 引擎) | 节点配置 `condition_expr` (IF 模式) | 上游数据 + CEL 表达式 | true/false 二选一分支继续执行 | 表达式求值报错 → 节点标记 `failed`, 走 W8 错误分支 | 恰好 1 条分支被激活 | P0 |
| FR-WORKFLOW-W4.2 | 系统 (CEL 引擎) | 节点配置 Switch 模式 (取值表达式 + N case) | 上游数据 | 命中 case 对应分支, 或 `default` 分支 | 表达式求值报错 → 同 W4.1 | 恰好 1 条分支被激活 | P1 |
| FR-WORKFLOW-W4.3 | 系统 (执行引擎) | ≥2 条 inbound edge 汇入 Merge 节点 | 各 inbound 分支的到达事件 | race: 首个到达即继续; join: 全部到达才继续 | join 模式部分分支永不到达 → 【TBD】超时策略未定, 见 SRS 已知缺口 | 汇合后继续 1 条 outbound | P1 |

#### 1.1.5 W5 循环与批处理 (2 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W5.1 | 系统 (执行引擎) | 节点配置 `loop_source_expr` 求值为数组 | 数组输入 | 循环体子图对每个元素执行 1 次, 全部完成后继续下游 | `loop_source_expr` 求值非数组 → 节点 `failed` | 循环体执行次数 = 数组长度 | P1 |
| FR-WORKFLOW-W5.2 | 系统 (执行引擎) | Loop 节点配置 `concurrency` (1-20) | 同上 | 并发度 = min(concurrency, 剩余元素数) 的循环体实例同时运行 | `concurrency` 超出 1-20 范围 → 保存时前端校验拒绝 | 同 W5.1 | P2 |

#### 1.1.6 W6 变量与表达式传递 (3 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W6.1 | Dev/PM + 系统 | 上游节点已产生输出 | `{{node.<id>.output.<field>}}` 引用表达式 | 执行时替换为上游实际输出值 | 引用不存在的 node_id/field → 节点 `failed`, 报错信息指明缺失引用 | 下游节点获得替换后的实际值 | P0 |
| FR-WORKFLOW-W6.2 | Dev/PM | Flow 已定义 `variables` | `{{flow.variables.<key>}}` 引用 | 执行时替换为当前全局变量值 | 引用不存在的 key → 节点 `failed` | 已完成的历史 Execution 不受后续变量修改影响 | P2 |
| FR-WORKFLOW-W6.3 | 系统 (CEL parser) | 表达式语法符合既有 CEL 语法 | CEL 表达式字符串 | 求值结果 (与 `/automation` 页规则一致) | 语法错误 → 保存时前端/后端双重校验拒绝 | 同一表达式在两处求值结果一致 | P0 |

#### 1.1.7 W7 子流程与复用 (2 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W7.1 | 系统 (执行引擎) | 节点配置 `referenced_flow_id`, 引用深度 ≤5 (BR-W-6) | 触发数据 | 同步等待子流程执行完成, 子流程末端输出作为本节点输出 | 引用深度 >5 或存在环 → 保存时拒绝 (循环检测算法 【TBD】, 见 SRS 已知缺口) | 子流程 1 次 Execution 记录关联主流程 | P1 |
| FR-WORKFLOW-W7.2 | Dev/PM | "+ Flow" 入口可见模板选择器 | 选择预置模板 | 生成预配置 Flow 图 | 无 | 用户可直接改参数使用 (**具体化落地见 W14**) | P2 |

#### 1.1.8 W8 错误处理与重试 (3 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W8.1 | 系统 (执行引擎) | 节点配置 `retry_policy` | 节点执行失败事件 | 按 `max_retries`/`backoff`/`delay_ms` 自动重试, 每次重试写 1 条 `execution_step` | 重试耗尽仍失败 → 转 W8.2 错误分支判定 | 节点最终状态为 `succeeded` 或 `failed` | P0 |
| FR-WORKFLOW-W8.2 | 系统 (执行引擎) | 节点重试耗尽且存在 `on_error` 边 | 节点最终失败事件 | 流程转入 `on_error` 分支 | 无 `on_error` 边 → 整体 Execution 标记 `failed` | Execution 不因单节点失败而无法收敛 | P0 |
| FR-WORKFLOW-W8.3 | 系统 (notification 域) | Execution 最终 `failed` 且无 `on_error` 兜底 | Execution 失败事件 | 30s 内通知 Flow 所有者 | 通知发送失败 → 复用既有 notification 域重试策略, 不阻塞 Execution 状态 | Flow 所有者收到通知 | P1 |

#### 1.1.9 W9 执行历史与调试 (3 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W9.1 | 系统 (执行引擎) | Execution 已启动 | 每次运行 / 每节点执行事件 | 1 条 `execution_history` + N 条 `execution_step` (均 append-only) | 写入失败 → 【TBD, 属 Design Doc 落地细节】需保证至少一次写入 (at-least-once), 执行引擎需要重试写入 | 历史记录不可修改/删除 | P0 |
| FR-WORKFLOW-W9.2 | Dev/PM/SRE | 存在 1 条 `failed` 的 Execution | "从失败节点重跑"操作 | 新 Execution (`resumed_from_execution_id` 指向原记录), 只重跑失败节点及下游 | 上游为非幂等动作 (如 `create_worktree`) 时复用历史输出的安全性 → **【TBD】留 Design Doc 评估幂等性标记机制, 见 SRS 风险 #3** | 已成功节点不重复执行 | P0 |
| FR-WORKFLOW-W9.3 | Dev/PM/SRE | 存在 ≥1 条 `execution_step` | 双击目标 execution_step | 侧栏展开完整输入/输出 JSON (格式化/可折叠) | JSON 过大 → 【TBD】是否分页/截断, Design Doc 阶段定 | 无副作用 (只读) | P1 |

#### 1.1.10 W10 激活状态与版本管理 (3 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W10.1 | Dev/PM/5域Lead | 无 `is_placeholder=true` 且未绑定 `agent_id` 的节点 | 切换 `enabled` 开关 | `enabled=true` 时触发节点响应真实事件; `false` 时仅手动可跑 | 存在未绑定占位节点时尝试激活 → **拒绝, 提示具体节点** (per W14.5 激活前置校验, 详见 §8 安全设计) | Flow 激活状态变更生效 | P0 |
| FR-WORKFLOW-W10.2 | 系统 (版本引擎) | Flow 定义发生变更 | 变更后的节点/边集合 | 新增 1 条 `automation_flow_versions` 记录 (SCD2) | 并发编辑冲突 → 【TBD】乐观锁/CRDT 依赖 SRS 风险 #6 (A12 CRDT 选型未拍板), 本 BD 不重复设计 | 历史版本只读可查 diff | P1 |
| FR-WORKFLOW-W10.3 | Dev/PM | 存在 ≥2 个历史版本 | 选择目标历史版本 + 回滚操作 | 生成新版本记录 (内容=所选历史版本), 版本号递增 | 所选版本已被物理清理 (不应发生, SCD2 append-only) → 系统异常, 记录审计日志 | 当前生效版本 = 所选历史版本内容 | P1 |

#### 1.1.11 W11 标签绑定任务卡 (5 项, issue 核心诉求)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W11.1 | PM/5域Lead | Flow 已存在 | `tags` + `tag_binding_expr` 配置 | 系统持续监控命中状态 | 表达式语法错误 → 保存前校验拒绝 (per §7.3) | Flow 进入受监控状态 | P0 |
| FR-WORKFLOW-W11.2 | 系统 (标签求值引擎) | `tags` 命中 `tag_binding_expr` | 命中事件 | 新建 1 条 `WorkItem` (`kind=automation_flow_card`, `source_flow_id` 回指) | 并发多次命中同一表达式 → 幂等判定 (per BR-W-1, 已有卡片时不重复创建) | Backlog 出现新卡 | P0 |
| FR-WORKFLOW-W11.3 | 系统/PM/5域Lead | 派生 WorkItem 已存在 | 用户对派生卡的读写操作 (Sprint 拖拽/字段编辑/图表统计) | 派生卡与人工卡完全同权 (真实持久化, 非投影) | 无 (设计决策本身即缺口澄清) | 派生卡参与既有全部 Sprint/图表逻辑 | P0 |
| FR-WORKFLOW-W11.4 | 系统 (事件流) | Flow `tags`/`tag_binding_expr` 变更 | canvas_event 事件 | 按 BR-W-3 三分支处理 (硬删/移回删/detached) | 事件处理中途失败 → 【TBD, Design Doc】重试/补偿机制未定 | 派生卡状态与命中结果最终一致 | P0 |
| FR-WORKFLOW-W11.5 | PM/5域Lead | 无 | AND/OR/NOT (含括号) 布尔表达式字符串 | 求值为 true/false | 语法错误 → 保存前校验拒绝 | 表达式可正确参与 W11.1 求值 | P1 |

#### 1.1.12 W12 Backlog / Sprint 联动 (5 项, issue 核心诉求)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W12.1 | 系统 | 派生 WorkItem 新建 | `sprint_id` 留空 | 卡片出现在 Backlog 列 | 无 | 复用既有 `!w.sprint_id` 判定, 零改动 | P0 |
| FR-WORKFLOW-W12.2 | PM | 派生卡在 Backlog | 拖拽操作 (`@dnd-kit`) | `sprint_id` 写入目标 Sprint | 拖拽目标 Sprint 已锁定/已完成 → 复用既有拖拽规则拒绝 | 派生卡进入目标 Sprint | P0 |
| FR-WORKFLOW-W12.3 | 系统 (事件流) | 派生卡所在 Sprint 状态为 `active`, 标签解绑 | 解绑事件 | `tag_binding_status=detached`, 红色角标提示, 不移除不删除 | 无 | 待 Sprint 完成/取消回 Backlog 后再走硬删规则 | P0 |
| FR-WORKFLOW-W12.4 | PM/5域Lead | 存在 ≥1 条标签绑定 Flow | 访问 "Flow Tags" 管理面板 | 表格展示"标签表达式→Flow→派生任务卡"三级绑定 | 数据加载失败 → 空态提示 | 支持在此页直接增删 Flow 的 tags | P1 |
| FR-WORKFLOW-W12.5 | PM | 用户手动编辑过派生卡某字段 | 字段编辑操作 | `user_edited_fields` 记录被接管字段, 该字段后续不再被系统同步覆盖 | 无 | 系统字段(`title`/`description`/`labels`)与用户字段分区生效 | P1 |

#### 1.1.13 W13 数据一致性与状态映射 (2 项)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W13.1 | PM/5域Lead | 无 (默认解耦) | 可选 `status_mapping` 配置 | 未配置时 Execution 状态不影响 WorkItem.status; 配置后按规则联动 | 映射规则引用不存在的 WorkItemStatus → 保存前校验拒绝 | 状态联动行为可预期、可配置 | P1 |
| FR-WORKFLOW-W13.2 | 系统 | 派生卡与人工卡共用 `sprint_id` 单值字段 | 无 (约束性 FR) | 派生卡天然满足 "1 task 1 sprint" | 无 | 无需例外分支 | P0 |

#### 1.1.14 W14 默认工作流模板库 (7 项, v1.1 新增)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W14.1 | 美术/TA, Dev, PM | 点击 "+ Flow" 入口 | 选择 3 套内置模板之一 | 模板选择器展示简介 + 缩略节点图预览 | 模板数据加载失败 → 空态提示, 不阻塞手动创建路径 | 用户可预览后决定是否套用 | P1 |
| FR-WORKFLOW-W14.2 | 美术/TA | 已选中 AAA 游戏资产管线模板 | 套用操作 | 生成 5 串联占位节点 + 1 组并行贴图占位 + 1 Merge, 数据流预映射 | 无 | 全部节点 `is_placeholder=true` | P1 |
| FR-WORKFLOW-W14.3 | Dev | 已选中 spec 式模板 | 套用操作 | 生成 5 占位节点(含 1 条件节点) + 1 条评审不通过回指边 | 用户搭建出无法退出的循环 → **不做自动检测** (per SRS 风险 #11, 【TBD】是否需 Flow 级最大循环次数硬限制) | 同上 | P1 |
| FR-WORKFLOW-W14.4 | Dev | 已选中 superpowers 式模板 | 套用操作 | 生成 5 占位节点(含 1 条件节点) + 1 条评审不通过回指边 | 同 W14.3 | 阶段占位名称不强绑定具体 agent 技能包 (per SRS 已知缺口) | P1 |
| FR-WORKFLOW-W14.5 | Dev/PM/美术TA | 已选中任一模板 | 套用确认操作 | 一次性实例化模板定义的全部节点+边+data_mapping | 尝试激活含 `is_placeholder=true` 且未绑定 `agent_id` 的 Flow → **拒绝并提示具体节点** (详见 §8) | 占位节点可保存但不可激活运行 | P0 |
| FR-WORKFLOW-W14.6 | Dev/PM/美术TA | 已套用并调整内置模板 | "另存为自定义模板"操作 | 新增 1 条 `is_builtin=false` 的 `flow_template` 记录 | 无 | 内置模板本身不受影响 | P1 |
| FR-WORKFLOW-W14.7 | Dev/PM/美术TA | 存在自定义模板 | 改名/删除/搜索操作 | 自定义模板 CRUD 生效, 内置模板无删除入口 | 尝试删除内置模板 → 前端不提供入口, 后端 403 兜底 | 模板库列表反映最新状态 | P2 |

#### 1.1.15 W15 LangGraph 智能控制 + 底部聊天栏 (5 项, v1.1 新增)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WORKFLOW-W15.1 | Dev/PM/美术TA | 已进入画布/Flow 编辑视图 | 无 (UI 常驻) | 底部常驻聊天栏渲染, `actor_session_id` 从 `undefined` 变为有效会话 | L0 会话建立失败 → 聊天栏显示离线态, 不阻塞画布其他操作 | 用户可开始对话 | P0 |
| FR-WORKFLOW-W15.2 | Dev/PM/美术TA | 聊天栏会话已建立 | 自然语言描述文本 | `chat_session.parsed_flow_draft` 生成草稿节点图, 落入编辑视图供确认 | mock 规则未命中任何句式 → 提示"未能解析", 不生成草稿 (**mock 规则覆盖范围【TBD】, 见 SRS 已知缺口**) | 草稿需用户显式确认才正式保存 (不静默落库) | P1 |
| FR-WORKFLOW-W15.3 | 系统 (L0) | 条件节点 `routing_mode="dynamic_agent"` | 当前 `TopAgentState` 上下文 | L0 动态决定分支走向, 决策依据记录到 `execution_step` | L0 不可达/超时 → **【TBD, Design Doc】降级策略未定** (回退 static_cel 默认分支? 整体失败?) | 分支决策可事后审计 (**决策可复现性 【TBD】, 见 SRS 风险 #9**) | P1 |
| FR-WORKFLOW-W15.4 | 系统 | 聊天栏消息已发送 | 消息内容 + `actor_session_id` | `execution_history.origin_chat_session_id` 回链 | 会话已过期 → 复用既有 TopAgentState 会话生命周期规则拒绝 | 由聊天栏发起的 Execution 可溯源至对话 | P0 |
| FR-WORKFLOW-W15.5 | Dev/PM/美术TA | 聊天栏已生成草稿 | 用户在编辑视图中的后续操作 | 草稿节点与手动创建节点行为完全一致 (可拖动/改参数/删除) | 无 | 不引入专属"聊天栏 Flow"数据结构, 复用 `flow_node`/`flow_edge` | P1 |

**In-Scope 合计**: 54 项 FR (P0=21 / P1=25 / P2=8), 34 用户故事 (US-W1 ~ US-W34, 覆盖率 34/54=63.0% ≥ 60% 门槛)。

### 1.2 不包含 (Out-of-Scope, 与 SRS §1.4 完全对齐, 不重复裁决)

| 排除项 | 说明 | 边界依据 |
|---|---|---|
| 真实第三方 LLM/NLU API 调用 (W15.2 NL→Flow 解析, W15.3 动态路由决策) | v1 一律 mock/规则化实现 | 守门 #23 v2 (禁真实第三方 AI API 调用), 真实接入时间点待 P2 阶段拍板 — **本 BD 不预先设计真实 LLM 集成方案** |
| 真实 LangGraph Python 后端 (`/api/tmo/*`) 网络对接 | 本 BD 只定义前端契约 + mock 桩 | 真实对接属 Agent Runtime 范畴, 待专门设计 |
| 真实第三方连接器市场 / 可视化表达式构建器 / 节点级混合标签 | 均为 P2+ 观察项 | per SRS §1.4, 留 P2+ |
| 后端持久化 API 具体实现 (DDL / route handler 代码) | 本 BD §4/§5 定义数据/接口**契约**, 不含可运行代码 | 详细设计 (DD-CANVAS-WORKFLOW-001, 待创建) 阶段落地 |
| A12 多人协同编辑 CRDT 冲突解决 | 依赖 `SRS-CANVAS-AGENT-001` A12 既有基线 | 本 BD 不重新设计, 只声明依赖 (SRS 风险 #6) |
| superpowers 模板各阶段真实技能自动触发 | v1 仅占位节点图, 无真实调度契约 | 属 Agent Runtime 范畴 |
| "图生高模"/"绑骨骼"等具体 AI 生成能力的真实执行逻辑 | v1 仅占位 | 待专门 SRS |

### 1.3 文档结构 (per issue 章节指示 + `BD-CANVAS-AGENT-001.md` 10 段模板折中)

本 BD 共 12 段 + 1 附录, 在 `BD-CANVAS-AGENT-001.md`/`BD-CANVAS-GAMIFY-001.md` 既有 10 段模板基础上, 按本 issue 的明确指示新增 2 个独立顶层段落 (§8 安全设计单列成段, 而非像姊妹 BD 那样折入 NFR 小节; §10 追溯矩阵单列成段), 以匹配 `ipa-security-design` / 追溯矩阵的 issue 要求; §3 沿用姊妹 BD"组件一览"命名并入画面设计内容 (而非拆出独立"画面设计"顶层段), 因为本域新增画面(聊天栏/模板选择器)与新增组件是 1:1 对应关系, 拆分会造成重复表述:

§0 目的 / §1 适用范围 / §2 系统架构 / §3 组件一览与画面设计 / §4 数据设计 / §5 接口设计 / §6 5 View 详细 / §7 NFR / §8 安全设计 / §9 守门合规 + TBD 追踪矩阵 / §10 追溯矩阵 / §11 签字栏 / §12 修订履历 / 附录 A 跨专题引用清单

### 1.4 派生映射

本 BD 100% 派生自 `SRS-CANVAS-WORKFLOW-001` v1.1 §4 (W1-W15, 54 项), 不新增 SRS 未提及的功能范围; W14/W15 (v1.1 新增 12 项) 是本次 issue 的重点验收对象, 在 §3/§4/§5/§8/§10 均标注 **(v1.1 新增)** 以便追溯。

---

## §2 系统架构 (System Architecture)

### 2.1 总体架构图 (5-tier, 对齐 `BD-CANVAS-AGENT-001` §2.1 tier 划分)

```
┌─────────────────────────────────────────────────────────────────┐
│ UI Tier (前端, frontend/src/)                                    │
│  - CanvasView.tsx: 新增 6 个 flow element kind 渲染分支            │
│  - Flow 编辑器组件 (页面路径待定,【TBD】复用主画布 viewport 或独立子画布) │
│  - 模板选择器组件 (W14.1, 挂载于 "+ Flow" 入口旁)                    │
│  - 底部聊天栏组件 (W15.1, 独立于 CommandBar.tsx)                    │
│  - 执行历史面板 / 标签绑定管理面板 ("/automation" 页扩展 tab)         │
├─────────────────────────────────────────────────────────────────┤
│ API Tier (BFF REST + WebSocket, per §5)                          │
│  - Flow/Node/Edge CRUD, Execution 触发/查询/重跑, Webhook 入口     │
│  - Flow Template CRUD (v1.1), Chat Session 消息 (v1.1)            │
├─────────────────────────────────────────────────────────────────┤
│ LangGraph 路由层 (v1.1 新增, 对接既有 ADR-0046, 不新起并行架构)      │
│  - L0 TopAgentState: 接收 chat_session 消息 + dynamic_agent 路由请求│
│  - "L1↔L1 通信禁止": Flow 内 Agent 占位节点间不允许直接通信决策,     │
│    动态路由决策统一经 L0 做出 (per 守门 #13a 派生)                   │
│  - /api/tmo/* 既有 8 端点复用, 不新增并行会话管理端点                │
├─────────────────────────────────────────────────────────────────┤
│ Application Tier (执行引擎, per §7 数据设计)                        │
│  - RuleExecutor 扩展: 触发调度 / CEL 求值 / 重试策略 / Merge-Loop   │
│  - 标签求值引擎: canvas_event → tag_binding_expr 命中判定 → BR-W-3  │
├─────────────────────────────────────────────────────────────────┤
│ Data Tier (per §4)                                                │
│  - automation_flow(_versions) / flow_node / flow_edge (Master)    │
│  - execution_history / execution_step / chat_session (Transaction)│
│  - flow_template (Master, v1.1) / WorkItem (Master, 扩展字段)     │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 UI/API/数据流概述

- **UI → API**: Flow 编辑器/聊天栏/模板选择器均通过 §5.2 REST 端点持久化, 无本地专属存储 (per §5.3 Zustand slice 仅做前端缓存, 非唯一真源)。
- **API → LangGraph 路由层**: 仅 W15.3 (`routing_mode="dynamic_agent"`) 与 W15.1/W15.2/W15.4 (聊天栏) 场景调用 L0; 其余 52 项 FR 的执行路径不经过 LangGraph 路由层, 走既有 `RuleExecutor` 同步/异步调度。
- **数据流**: 详见 §6.2 データ view 对 §7.4 (SRS) 标签绑定数据流的复用与扩展。

### 2.3 数据流场景 (Flow 执行 + 标签绑定, 两条主链路)

```
[场景 A: Flow 执行]
触发事件 (manual/cron/webhook/canvas_event)
  → RuleExecutor 建立 execution_history (running)
  → 按 flow_edge 图遍历: action/condition/merge/loop/subflow 节点逐一求值
  → 每节点写 1 条 execution_step (含 retry_policy 重试记录)
  → dynamic_agent 条件节点 → 经 L0 TopAgentState 决策 (per §2.1 LangGraph 路由层)
  → 全部节点完成或走 on_error 分支 → execution_history 状态收敛 (succeeded/failed)

[场景 B: 标签绑定任务卡]
Flow.tags 变更 (含 W15 聊天栏/模板套用间接触发)
  → canvas_event 触发 (FR-W2.4)
  → 标签绑定重新求值 (FR-W11.1/W11.4)
  → 命中变化判定 → 创建/detach/硬删 WorkItem (BR-W-3 三分支)
  → user_edited_fields 保护检查, 系统字段同步跳过已接管字段
```

---

## §3 组件一览与画面设计 (Component List & Screen Design)

> per `ipa-screen-design` skill: 以下画面均分配唯一 Screen ID, 未确认的文案/尺寸/交互细节一律标注【TBD】, 不编造。

### 3.1 组件总览 (新增组件, 对齐姊妹 BD 组件表格式)

| 组件 | 说明 | 对应 FR |
|---|---|---|
| Flow 编辑器 (新组件, 页面路径待定) | 节点拖拽画布, 复用或独立于 `CanvasView.tsx` viewport 【TBD, 见 SRS 风险 #1】 | W1-W10 |
| 模板选择器 (新组件) | "+ Flow" 入口旁弹出, 展示 3 套内置模板 + 自定义模板 | W14 |
| 底部聊天栏 (新组件) | 画布/Flow 编辑视图底部常驻, 落地 `store.ts:565` `actor_session_id` 预留点 | W15.1/W15.4 |
| 执行历史面板 (新组件) | 独立面板 (非画布 element), 类比 `/automation` 页 | W9 |
| Flow Tags 管理面板 (新 tab, 扩展 `/automation` 页) | 标签表达式→Flow→派生任务卡 三级绑定关系表 | W12.4 |
| 节点配置侧栏 (扩展既有侧栏模式) | 双击节点后展开, 复用 V0.1 联动 2 模式 | W1.4, W9.3 |

### 3.2 Screen ID 一览

| Screen ID | 画面名称 | 目的 | 主要 Actor |
|---|---|---|---|
| SCR-WF-01 | Flow 编辑器主画面 | 拖拽搭建/编辑节点图 | Dev/PM/美术TA |
| SCR-WF-02 | 模板选择器 (弹层/侧栏) | 选择内置/自定义模板一键套用 | Dev/PM/美术TA |
| SCR-WF-03 | 底部聊天栏 | 自然语言创建/编辑 Flow, L0 会话入口 | Dev/PM/美术TA |
| SCR-WF-04 | 执行历史面板 | 查看/重跑/调试 Execution | Dev/PM/SRE |
| SCR-WF-05 | Flow Tags 管理面板 | 管理标签绑定关系, 增删 Flow tags | PM/5域Lead |
| SCR-WF-06 | 节点配置侧栏 | 编辑单节点参数 (trigger/action/condition/retry 等) | Dev/PM/美术TA |

#### SCR-WF-01 Flow 编辑器主画面

| 项目 | 内容 |
|---|---|
| 显示条件 | 用户点击工具栏 "+ Flow" (FR-W1.3) 或从 Flow 列表进入已有 Flow |
| 画面构成 | 顶部工具栏 (Active/Inactive 开关 FR-W10.1, 版本下拉 FR-W10.2, "+ Flow" 入口) + 中央节点画布 (复用或独立于 `CanvasView.tsx` viewport, 【TBD】见 SRS 风险 #1) + 右侧节点配置侧栏 (双击呼出, SCR-WF-06) + 底部聊天栏 (SCR-WF-03) |
| 输入项目 | 节点拖拽位置 / connector 连线 / 双击节点触发详情 |
| 权限控制 (前端显示 vs 后端) | 前端: 无 Flow 编辑权限时隐藏"+ Flow"/编辑操作按钮; 后端: RBAC 校验写操作 (per §8.2), 前端隐藏不能替代后端校验 |
| 画面迁移 | 工具栏 "+ Flow" → 新建空 Flow (本画面); 模板选择器确认套用 (SCR-WF-02) → 返回本画面 (已预置节点); 执行历史 tab → SCR-WF-04 |
| 关联 API | `/v1/collaboration/flows`, `/v1/collaboration/flows/{id}/nodes`, `/edges` (per §5.2) |
| 异常表现 | 加载失败 → 画布区域显示错误占位 + 重试按钮; 保存冲突 (SCD2 并发写) → 【TBD, 见 SRS 风险 #6/#10.2】 |

#### SCR-WF-02 模板选择器

| 项目 | 内容 |
|---|---|
| 显示条件 | "+ Flow" 入口旁触发 (FR-W14.1) |
| 画面构成 | 3 套内置模板卡片 (AAA 游戏资产管线 / spec 式 / superpowers 式, 各附简介+缩略节点图预览) + 自定义模板列表 (`is_builtin=false`) + 空白创建入口 |
| 输入项目 | 选中某模板 → 确认套用按钮 |
| 必填/校验 | 无表单输入, 仅单选 |
| 按钮行为 | "套用" → 一次性实例化节点/边/data_mapping (FR-W14.5), 跳转 SCR-WF-01; "取消" → 关闭弹层 |
| 权限控制 | 内置模板只读 (无编辑/删除入口, 前端隐藏 + 后端 403 双重保护); 自定义模板增删改需 Flow 写权限 |
| 关联 API | `/v1/collaboration/flow-templates` (GET 列表, POST 套用后新建 Flow) |
| 异常表现 | 模板列表加载失败 → 空态 + 重试, 不阻塞"空白创建"路径 |

#### SCR-WF-03 底部聊天栏

| 项目 | 内容 |
|---|---|
| 显示条件 | 画布/Flow 编辑视图始终可见 (常驻), 【TBD】是否需跨页面持久悬浮 (per SRS 已知缺口) |
| 画面构成 | 输入框 + 发送按钮 + 消息历史区 (per `chat_session`) + 草稿预览态 (解析出 `parsed_flow_draft` 后展示"确认应用"/"取消"按钮) |
| 输入项目 | 自然语言文本 (无长度上限【TBD】) |
| Validation | mock 规则匹配失败 → 提示"未能解析"消息, 不生成草稿, 不报错崩溃 |
| 按钮行为 | "发送" → 调用 §5.2 聊天消息端点; "确认应用"草稿 → 正式写入 `flow_node`/`flow_edge` (非静默保存); "取消" → 丢弃草稿 |
| 权限控制 | 需登录会话 (`actor_session_id`), 与既有 L0 TopAgentState 会话生命周期一致 |
| 关联 API | `POST /v1/collaboration/chat-sessions/{id}/messages` |
| 异常表现 | L0 会话不可达 → 聊天栏显示"离线"提示, 不阻塞画布其他操作 (per FR-W15.1 异常处理) |

#### SCR-WF-04 执行历史面板

| 项目 | 内容 |
|---|---|
| 显示条件 | Flow 编辑视图内 "执行历史" tab, 或独立 `/automation` 扩展页 |
| 画面构成 | Execution 列表 (状态/耗时/触发方式) + 展开后节点级 `execution_step` 明细 (输入/输出 JSON, 可折叠) |
| 输入项目 | 无 (只读列表) + "重跑"按钮 (仅 failed 状态可见) |
| 按钮行为 | "从失败节点重跑" → POST resume 端点 (FR-W9.2) |
| 权限控制 | 查看: Flow 只读权限即可; 重跑: 需 Flow 写权限 |
| 关联 API | `GET /v1/collaboration/flows/{id}/executions`, `POST .../resume` |
| 异常表现 | JSON 过大 →【TBD】是否分页/截断 |

#### SCR-WF-05 Flow Tags 管理面板

| 项目 | 内容 |
|---|---|
| 显示条件 | `/automation` 页新增 "Flow Tags" tab (FR-W12.4) |
| 画面构成 | 表格: 标签表达式 / 关联 Flow / 派生任务卡链接 / 当前命中状态 (bound/unbound/detached) |
| 输入项目 | 表格内联编辑 Flow 的 `tags` |
| 权限控制 | 需 Flow 写权限方可编辑 tags; 只读用户可查看命中状态 |
| 关联 API | 复用 Flow CRUD (`PATCH /v1/collaboration/flows/{id}`) |
| 异常表现 | 数据加载失败 → 空态提示 |

#### SCR-WF-06 节点配置侧栏

| 项目 | 内容 |
|---|---|
| 显示条件 | 双击 SCR-WF-01 画布上任一节点 |
| 画面构成 | 按节点 kind 动态渲染表单 (trigger_kind / condition_expr / action config / retry_policy / routing_mode 等) |
| 必填/Validation | CEL 表达式语法校验 (W6.3) / retry_policy 数值范围 / concurrency 1-20 (W5.2) |
| 按钮行为 | "保存" → PATCH 节点; "取消" → 丢弃未保存修改 |
| 权限控制 | 需 Flow 写权限 |
| 关联 API | `PATCH /v1/collaboration/flows/{id}/nodes/{node_id}` |
| 异常表现 | 校验失败 → 表单内联错误提示, 不提交 |

### 3.3 画面迁移图

```
工具栏 "+ Flow" ──→ SCR-WF-02 模板选择器 ──(套用/空白)──→ SCR-WF-01 Flow 编辑器
                                                              │
                          ┌───────────────────────────────────┼───────────────────┐
                          ▼                                   ▼                   ▼
                  SCR-WF-06 节点配置侧栏(双击节点)    SCR-WF-04 执行历史面板   SCR-WF-05 Flow Tags 面板
                          │                                   │
                          ▼                                   ▼
                    (保存/取消回 SCR-WF-01)          (重跑生成新 Execution)

SCR-WF-03 底部聊天栏 (画布内常驻, 独立生命周期) ──(确认应用草稿)──→ SCR-WF-01 (草稿节点落入编辑视图)
```

---

## §4 数据设计 (Data Design)

> per `ipa-database-design` skill: 以下字段均取自 SRS §7 已声明的 schema 增项, 不新增未在 SRS 中出现的业务字段; 索引/约束为**基本设计阶段建议**, 非最终定案, 详细设计阶段可调整。

### 4.1 数据模型总览 (8 张表, W/T/M 覆盖)

| 表 | 分类 | 说明 |
|---|---|---|
| `automation_flow` | Master | Flow 聚合根, 持有当前生效指针 |
| `automation_flow_versions` | Master (SCD Type 2) | 版本历史, 物理删除禁止 |
| `flow_node` | Master | 流程节点, 随版本演进 |
| `flow_edge` | Master | 流程边 |
| `execution_history` | Transaction | 单次执行记录, append-only |
| `execution_step` | Transaction | 单节点执行记录, append-only |
| `flow_template` (**v1.1 新增**) | Master | 模板库主体 (内置只读 + 自定义可写) |
| `chat_session` (**v1.1 新增**) | Transaction | 底部聊天栏会话/消息, append-only |

**W/T/M 三类横展 (per 守门 #13, 100% 表覆盖声明)**:

- Master 5/8 (62.5%): `automation_flow` / `automation_flow_versions` / `flow_node` / `flow_edge` / `flow_template`
- Transaction 3/8 (37.5%): `execution_history` / `execution_step` / `chat_session`
- **Work 0/8 (0%)** — 显式声明而非遗漏 (per 守门 #11 缺标比错标): 本域没有 session-bound/TTL 临时数据表。聊天栏会话看似像"session"概念, 但 SRS FR-W15.4 明确要求其执行记录需可回链审计 (`execution_history.origin_chat_session_id`), 故归类 **Transaction (append-only 永久保留)** 而非 Work (TTL 过期清理); 若详细设计阶段引入短期编辑锁/在线协作状态等临时数据, 届时可能新增 Work 类表, 本 BD 不预先假设该需求存在。
- **总计 8/8 = 100% ✓**

### 4.2 表定义 (DDL 建议, 详细设计阶段可调整)

#### 4.2.1 `automation_flow` (Master, 物理删除禁止)

```sql
CREATE TABLE automation_flow (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  tags TEXT[] NOT NULL DEFAULT '{}',
  tag_binding_expr TEXT,  -- AND/OR/NOT 布尔表达式, per FR-W11.5
  enabled BOOLEAN NOT NULL DEFAULT false,  -- FR-W10.1
  variables JSONB,  -- flow 级全局变量, per FR-W6.2
  status_mapping JSONB,  -- 可选 Execution状态→WorkItem.status 映射, per FR-W13.1
  current_version_id UUID,  -- FK → automation_flow_versions(id), 当前生效版本指针
  tenant_id UUID NOT NULL,  -- RLS 13 类 per 守门 #13
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by UUID NOT NULL
);

CREATE INDEX idx_flow_tenant ON automation_flow(tenant_id);
CREATE INDEX idx_flow_enabled ON automation_flow(enabled);
CREATE INDEX idx_flow_tags ON automation_flow USING GIN(tags);

ALTER TABLE automation_flow ENABLE ROW LEVEL SECURITY;
CREATE POLICY flow_tenant_isolation ON automation_flow
  USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

#### 4.2.2 `automation_flow_versions` (Master, SCD Type 2, 物理删除禁止)

```sql
CREATE TABLE automation_flow_versions (
  id UUID PRIMARY KEY,
  automation_flow_id UUID NOT NULL REFERENCES automation_flow(id),
  version_no INTEGER NOT NULL,
  definition JSONB NOT NULL,  -- nodes + edges 快照, per FR-W10.2
  valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
  valid_to TIMESTAMPTZ,
  is_current BOOLEAN NOT NULL DEFAULT true,
  rolled_back_from_version_no INTEGER,  -- 回滚来源, per FR-W10.3
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by UUID NOT NULL,
  UNIQUE(automation_flow_id, version_no)
);

CREATE INDEX idx_flowver_flow ON automation_flow_versions(automation_flow_id);
CREATE INDEX idx_flowver_current ON automation_flow_versions(automation_flow_id, is_current) WHERE is_current = true;
CREATE INDEX idx_flowver_tenant ON automation_flow_versions(tenant_id);
```

#### 4.2.3 `flow_node` (Master)

```sql
CREATE TABLE flow_node (
  id UUID PRIMARY KEY,
  automation_flow_id UUID NOT NULL REFERENCES automation_flow(id),
  kind VARCHAR(30) NOT NULL,  -- trigger/action/condition/merge/loop/subworkflow
  trigger_kind VARCHAR(30),  -- manual/schedule_cron/webhook/canvas_event
  action_kind VARCHAR(30),  -- 既有 6 种 + http_request + transform_data + dispatch_agent
  condition_expr TEXT,  -- CEL, per FR-W4.1/W4.2/W6.3
  merge_mode VARCHAR(10),  -- race/join, per FR-W4.3
  loop_source_expr TEXT,  -- per FR-W5.1
  loop_body_node_ids UUID[],
  concurrency INTEGER DEFAULT 1 CHECK (concurrency BETWEEN 1 AND 20),  -- per FR-W5.2
  input_bindings JSONB,  -- {{node.<id>.output.<field>}} 映射, per FR-W6.1
  retry_policy JSONB,  -- {max_retries, backoff, delay_ms}, per FR-W8.1
  referenced_flow_id UUID REFERENCES automation_flow(id),  -- per FR-W7.1
  webhook_token VARCHAR(64),  -- per FR-W2.3
  routing_mode VARCHAR(20) NOT NULL DEFAULT 'static_cel',  -- static_cel/dynamic_agent, per FR-W15.3 (v1.1 新增)
  is_placeholder BOOLEAN NOT NULL DEFAULT false,  -- per FR-W14.5 (v1.1 新增)
  placeholder_role_hint VARCHAR(100),  -- per FR-W14.5 (v1.1 新增)
  position_x REAL, position_y REAL,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_node_flow ON flow_node(automation_flow_id);
CREATE INDEX idx_node_tenant ON flow_node(tenant_id);
CREATE INDEX idx_node_placeholder ON flow_node(automation_flow_id, is_placeholder) WHERE is_placeholder = true;
CREATE UNIQUE INDEX idx_node_webhook_token ON flow_node(webhook_token) WHERE webhook_token IS NOT NULL;
```

#### 4.2.4 `flow_edge` (Master)

```sql
CREATE TABLE flow_edge (
  id UUID PRIMARY KEY,
  automation_flow_id UUID NOT NULL REFERENCES automation_flow(id),
  from_node_id UUID NOT NULL REFERENCES flow_node(id),
  to_node_id UUID NOT NULL REFERENCES flow_node(id),
  edge_label VARCHAR(50),  -- true/false/case 值
  edge_kind VARCHAR(10) NOT NULL DEFAULT 'normal',  -- normal/on_error, per FR-W8.2
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (from_node_id <> to_node_id)  -- 禁止自环, per FR-W1.2 异常处理
);

CREATE INDEX idx_edge_flow ON flow_edge(automation_flow_id);
CREATE INDEX idx_edge_from ON flow_edge(from_node_id);
CREATE INDEX idx_edge_to ON flow_edge(to_node_id);
```

#### 4.2.5 `execution_history` (Transaction, append-only, 物理删除禁止, **不**级联删除随 Flow 删除)

```sql
CREATE TABLE execution_history (
  id UUID PRIMARY KEY,
  automation_flow_id UUID NOT NULL,  -- 有意不加 REFERENCES ... ON DELETE CASCADE, per §7.3 数据完整性约束
  automation_flow_version_id UUID,
  status VARCHAR(20) NOT NULL,  -- running/succeeded/failed
  trigger_kind VARCHAR(30) NOT NULL,
  resumed_from_execution_id UUID,  -- per FR-W9.2
  origin_chat_session_id UUID,  -- per FR-W15.4 (v1.1 新增)
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ended_at TIMESTAMPTZ,
  tenant_id UUID NOT NULL
);

CREATE INDEX idx_exec_flow ON execution_history(automation_flow_id);
CREATE INDEX idx_exec_status ON execution_history(status);
CREATE INDEX idx_exec_chat_session ON execution_history(origin_chat_session_id) WHERE origin_chat_session_id IS NOT NULL;
CREATE INDEX idx_exec_tenant ON execution_history(tenant_id);
```

#### 4.2.6 `execution_step` (Transaction, append-only, 物理删除禁止)

```sql
CREATE TABLE execution_step (
  id UUID PRIMARY KEY,
  execution_id UUID NOT NULL REFERENCES execution_history(id),
  flow_node_id UUID NOT NULL,
  status VARCHAR(20) NOT NULL,  -- succeeded/failed/skipped
  input JSONB,
  output JSONB,
  retry_count INTEGER NOT NULL DEFAULT 0,
  routing_decision JSONB,  -- L0 动态路由决策依据, per FR-W15.3 (v1.1 新增, 粒度【TBD】)
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ended_at TIMESTAMPTZ,
  tenant_id UUID NOT NULL
);

CREATE INDEX idx_step_exec ON execution_step(execution_id);
CREATE INDEX idx_step_node ON execution_step(flow_node_id);
```

#### 4.2.7 `flow_template` (Master, **v1.1 新增**, per W14)

```sql
CREATE TABLE flow_template (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  is_builtin BOOLEAN NOT NULL DEFAULT false,
  definition JSONB NOT NULL,  -- { nodes: FlowNode[], edges: FlowEdge[] }
  created_by UUID NOT NULL,
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_template_tenant ON flow_template(tenant_id);
CREATE INDEX idx_template_builtin ON flow_template(is_builtin);

ALTER TABLE flow_template ENABLE ROW LEVEL SECURITY;
CREATE POLICY template_tenant_isolation ON flow_template
  USING (is_builtin = true OR tenant_id = current_setting('app.tenant_id')::UUID);
-- 内置模板 (is_builtin=true) 跨租户可读, 自定义模板受租户隔离; 写操作另受 §8 RBAC 校验 (内置只读)
```

#### 4.2.8 `chat_session` (Transaction, **v1.1 新增**, per W15, append-only)

```sql
CREATE TABLE chat_session (
  id UUID PRIMARY KEY,
  actor_session_id UUID NOT NULL,  -- 对应 store.ts:565 TODO, 与 L0 TopAgentState 会话对齐
  message TEXT NOT NULL,
  parsed_flow_draft JSONB,  -- mock 解析出的草稿节点图, per FR-W15.2
  applied BOOLEAN NOT NULL DEFAULT false,  -- 草稿是否已被用户确认应用
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by UUID NOT NULL
);

CREATE INDEX idx_chat_actor_session ON chat_session(actor_session_id);
CREATE INDEX idx_chat_tenant ON chat_session(tenant_id);
```

### 4.3 既有表字段扩展

| 表 | 新增字段 | 说明 |
|---|---|---|
| `WorkItem` | `source_kind VARCHAR(30)` | 标记派生来源 (`workflow_tag_binding`), per FR-W11.2 |
| `WorkItem` | `source_flow_id UUID` (nullable, **不**级联删除, per §7.3) | 指回源 Flow, Flow 删除时置空保留卡片 |
| `WorkItem` | `tag_binding_status VARCHAR(10)` | `bound`/`detached`/`null`, per FR-W12.3 |
| `WorkItem` | `user_edited_fields TEXT[]` | 系统字段/用户字段分区, per FR-W12.5 |

### 4.4 枚举扩展

| 枚举 | 新增值 | 说明 |
|---|---|---|
| `AutomationTriggerKind` | `webhook`, `canvas_event` | per FR-W2.3/W2.4 |
| `AutomationActionKind` | `http_request`, `transform_data` | per FR-W3.2 (既有 `dispatch_agent` 复用, 不新增) |
| `WorkItemKind` | `automation_flow_card` | per FR-W11.2 |

### 4.5 SCD / RLS / 审计策略

- **SCD Type 2**: 仅 `automation_flow_versions` 采用 (`valid_from`/`valid_to`/`is_current`), 与姊妹 BD (`agents`/`agent_relationship_edges` 直接在主表做 SCD2) 不同设计选择 — 本域将"当前生效版本"拆到独立版本表, 因为 Flow 的"定义"(节点图) 变更频率与幅度显著高于 `automation_flow` 主表的元数据字段 (name/tags/enabled), 拆分可避免每次拖节点都产生 1 条完整主表新版本。**此设计选择在详细设计阶段仍可调整, 标记为建议而非定案**。
- **RLS**: `tenant_id` 贯穿全部 8 张新表 (per 守门 #13), 沿用姊妹 BD `current_setting('app.tenant_id')` 模式; `flow_template` 对内置模板放宽为跨租户可读 (§4.2.7)。
- **审计**: `execution_history`/`execution_step`/`chat_session` 本身即 append-only 审计记录, 不再额外建审计表 (对齐 SRS §7.2 "类比既有 AuditEvent 设计"); `automation_flow`/`flow_node`/`flow_edge` 的变更审计依赖 `automation_flow_versions` 的版本对比 (diff), 不单独建 audit 表 — **【TBD, Design Doc】**是否需要独立 `automation_flow_audit` 表记录"谁在何时改了哪个字段"(细粒度审计), 当前设计假设版本 diff 已足够, 待安全/合规评审确认。

### 4.6 数据完整性约束 (与 SRS §7.3 对齐)

- `flow_node.automation_flow_id` 外键必填, 级联删除 (Flow 删除 → 节点/边级联删除)
- `execution_history`/`execution_step` **不**级联删除, 保留历史审计 (per §4.2.5 注释)
- `WorkItem.source_flow_id` 指向的 Flow 被删除时置空并保留卡片, 不级联删除 WorkItem
- `automation_flow.tag_binding_expr` 保存前必须通过语法校验 (前端 + 后端双重, per §6.2 异常场景)
- `flow_node.concurrency` CHECK (1-20), `flow_edge` CHECK 禁止自环 (from ≠ to)

## §5 接口设计 (Interface Design)

> per `ipa-interface-design` skill: 端点路径取自 SRS §8.2 已声明契约, 未声明的具体超时秒数/重试次数一律引用总册既有基线或标注【TBD】, 不自行编造数值。

### 5.1 REST API (8 端点, BFF)

| API ID | Method | Path | 说明 | Auth | RLS |
|---|---|---|---|---|---|
| API-WF-01 | `GET/POST/PATCH/DELETE` | `/v1/collaboration/flows` | Flow CRUD, 复用总册 §6.2 Canvas CRUD 模式 | Session token (既有) | ✓ tenant_id |
| API-WF-02 | `GET/POST/PATCH/DELETE` | `/v1/collaboration/flows/{id}/nodes`, `/edges` | Node/Edge CRUD | 同上 | ✓ |
| API-WF-03 | `POST/GET` | `/v1/collaboration/flows/{id}/executions` | 手动触发 Execution + 历史查询 | 同上 | ✓ |
| API-WF-04 | `POST` | `/v1/collaboration/flows/{id}/executions/{exec_id}/resume` | 从失败节点重跑 (per FR-W9.2) | 同上, 需 Flow 写权限 | ✓ |
| API-WF-05 | `POST` | `/v1/collaboration/flows/{id}/webhook/{token}` | Webhook 触发入口 (per FR-W2.3) | **token 校验** (非 session, 详见下) | ✓ |
| API-WF-06 (**v1.1 新增**) | `GET/POST/PATCH/DELETE` | `/v1/collaboration/flow-templates` | 模板 CRUD, 内置模板 (`is_builtin=true`) 只读, DELETE 仅对自定义模板开放 | Session token | ✓ |
| API-WF-07 (**v1.1 新增**) | `POST` | `/v1/collaboration/flow-templates/{id}/instantiate` | 套用模板一次性实例化节点/边/data_mapping (per FR-W14.5) | 同上 | ✓ |
| API-WF-08 (**v1.1 新增**) | `POST` | `/v1/collaboration/chat-sessions/{id}/messages` | 聊天栏消息 + NL→Flow 解析 (per FR-W15.1/W15.2), v1 后端为 mock 规则化解析, 复用 `/api/tmo/*` 做会话管理 | Session token + `actor_session_id` | ✓ |

### 5.2 API 详细契约 (示例: 核心端点, 详细设计阶段补齐余下端点的完整 schema)

#### API-WF-03 `POST /v1/collaboration/flows/{id}/executions`

| 项 | 内容 |
|---|---|
| Direction | UI → BFF |
| Encoding | JSON, UTF-8 |
| Request | `{ trigger_kind: "manual", input?: Record<string, unknown> }` |
| Response 200 | `{ execution_id: UUID, status: "running" }` |
| Response 4xx | `400` Flow 定义非法 (存在未绑定 `agent_id` 的 `is_placeholder` 节点且 Flow `enabled=true` 时同样拒绝, 见 §8) / `403` 无 Flow 执行权限 / `404` Flow 不存在 |
| Timeout | 复用总册 §5.6 错误处理基线 (网络层), 具体秒数【TBD, 待总册 §5.6 数值统一确认后引用, 本 BD 不新定义独立于总册的超时数值】 |
| Retry | 网络传输层最多 2 次 (per 总册 §5.6), 与 FR-W8.1 节点级业务重试 (`retry_policy`) 相互独立分层, 不叠加计数 |
| Idempotency | 手动触发 v1 **不去重** (每次调用产生 1 条新 Execution), 与 Webhook/cron 触发行为一致, 由调用方自行避免重复点击造成的重复执行 |
| Ordering | 单 Flow 允许并发多个 Execution 同时运行, 不做全局串行化 (per SRS 未声明串行约束) |

#### API-WF-05 `POST /v1/collaboration/flows/{id}/webhook/{token}`

| 项 | 内容 |
|---|---|
| Direction | 外部系统 → BFF (唯一对外无需登录会话的入口) |
| Authentication | **path token 校验** (`flow_node.webhook_token`), 非 Session Cookie/Bearer — 因为调用方是外部系统而非登录用户 |
| Authorization | token 与 `webhook_token` 精确匹配 + 所属 Flow `enabled=true` 方可触发 |
| Request | 任意 JSON body (透传作为触发数据, 供下游节点 `input_bindings` 引用) |
| Response 200 | `{ execution_id: UUID }` |
| Response 4xx | `401` token 不匹配/已吊销 / `413` body 超出大小限制【TBD, 具体字节数待 Design Doc 结合总册请求体上限统一确认】 |
| **签名/token 轮换策略** | **【TBD, 见 SRS §10 风险相关已知缺口】** — 当前设计仅有静态 `webhook_token` 字符串比对, 无 HMAC 签名校验、无 token 定期轮换机制。安全评审前不建议接入外部生产系统调用, 详见 §8 安全设计 |
| Retry (调用方侧) | 由外部系统自行决定, BFF 不对 webhook 请求做特殊去重 (幂等性依赖外部系统请求体自带的业务幂等键, 本 BD 不假设外部系统行为) |

#### API-WF-08 `POST /v1/collaboration/chat-sessions/{id}/messages`

| 项 | 内容 |
|---|---|
| Direction | UI (底部聊天栏) → BFF → (mock 规则引擎, 非真实 LLM) |
| Request | `{ message: string, actor_session_id: UUID }` |
| Response 200 | `{ chat_session_id: UUID, parsed_flow_draft: FlowDraft \| null, matched: boolean }` |
| Response 4xx | `401` `actor_session_id` 已过期/不存在 |
| 业务规则 | v1 解析层为 mock/规则化实现 (关键词+模板匹配), 非真实 LLM API 调用 (per 守门 #23 v2); 解析失败返回 `matched: false`, 不视为错误 |
| 已知缺口 | mock 规则覆盖范围 (能正确解析哪些句式) 待 Design Doc 列出具体规则表 (per SRS §10 已知缺口); 真实 NLU/LLM 接入时间点待 P2 阶段拍板, 本 BD 不预先设计真实集成方案 |

### 5.3 实时更新: 复用既有 Canvas WebSocket 通道 (不新增独立 WS 端点)

SRS §8 未声明本域专属 WebSocket 端点; Flow 节点/边本身是画布 element 的扩展 kind (per §2.1 6 个新 element kind), 节点位置/执行状态的实时同步**复用**总册既有 `wss://canvas-collab/canvases/[id]` 通道 (per `BD-CANVAS-AGENT-001` §5.2, `element.update` 事件), 不新增并行的 WebSocket 端点。Execution 运行中的节点级状态高频刷新需求 (如 SCR-WF-04 面板是否需要秒级刷新而非轮询) — **【TBD, Design Doc】**, 若确认需要, 届时在既有 `canvas-collab` 通道上扩展 `execution_step.status_changed` 事件类型, 而非另起新 WebSocket 端点。

### 5.4 与既有 25 module 联动接口

复用总册 §6.3 联动矩阵, 新增 1 行 (per SRS §8.4):

| 25 module | 画布表现 | 本 BD 扩展 |
|---|---|---|
| automation | `automation_node` 单规则 | 升级为 Flow 图 (W1-W10), 既有 `AutomationRule` 视为 1 trigger + N action 的退化 2 层 Flow, 向后兼容 |
| work-item | 拖 WorkItem → 画布 element | 标签绑定自动生成 WorkItem (W11-W12) |
| notification | `send_notification` 动作 | Flow 失败通知复用 (FR-W8.3) |
| L0/TMO (ADR-0046) | `/api/tmo/*` 8 端点 | 聊天栏会话 + 动态路由决策复用, 不新增并行会话端点 (v1.1 新增) |

### 5.5 错误处理总则

复用总册 §5.6 错误处理基线 (网络层重试), 节点级 `retry_policy` (FR-W8.1) 是业务层重试, 两层重试独立计数、互不干扰 (per SRS §8.5)。所有 REST 端点的 4xx 错误响应体统一沿用既有 BFF 错误结构 `{ error_code, message }` (per总册既有约定), 本 BD 不新定义独立的错误响应格式。

## §6 5-View 详细设计 (機能・データ・動作・モジュール・ネットワーク)

> 与 `BD-CANVAS-AGENT-001` §6 同一 5-View 体系, 逐 View 覆盖全部 54 项 FR (W1-W15)。

### 6.1 機能 View (Functional)

| 功能块 | 对应 FR 组 | 核心机能 |
|---|---|---|
| 节点体系与画布编排 | W1-W7 (FR-W1.1~W7.2, 20 项) | 节点类型/触发/动作/分支/循环/变量/子流程 — 图结构编辑与静态校验 |
| 执行引擎 | W8-W9 (FR-W8.1~W9.3, 6 项) | 错误重试、执行历史、单步调试 |
| 生命周期治理 | W10 (FR-W10.1~W10.3, 3 项) | 激活状态机、版本 SCD、Agent 占位符预检 |
| 任务卡联动 (issue 核心诉求) | W11-W12 (FR-W11.1~W12.5, 10 项) | 标签绑定表达式 → 自动生成 WorkItem, Backlog/Sprint 三分支回收 |
| 一致性 | W13 (FR-W13.1~W13.2, 2 项) | Flow 状态与 WorkItem 状态映射一致性校验 |
| 模板库 (v1.1) | W14 (FR-W14.1~W14.7, 7 项) | 内置模板浏览/预览/一键实例化, Agent 占位符节点 |
| 智能控制/聊天栏 (v1.1) | W15 (FR-W15.1~W15.5, 5 项) | 底部聊天栏、mock NL→Flow 解析、`routing_mode` 静态/动态双模式 |

54 项 FR 的逐条 Actor/输入/输出/异常映射已在 §1.1.1-§1.1.15 给出, 本节不重复罗列, 仅做功能块级归类以支撑 §10 追溯矩阵按块索引。

### 6.2 データ View (Data)

复用 §4 全部内容 (8 张表, W/T/M = 5/3/0, 见 §4.1)。跨 View 补充: Flow 定义 (`automation_flow`+`flow_node`+`flow_edge`) 是**静态图数据**, Execution 相关表 (`execution_history`+`execution_step`) 是**运行时快照数据**, 两者生命周期独立 — 删除/停用 Flow 不级联删除历史 Execution 记录 (审计要求, per SRS §7 数据需求)。`chat_session` (v1.1) 是短生命周期会话数据, 与 Flow 数据无外键强耦合 (仅通过 `parsed_flow_draft` JSON 字段弱引用候选 Flow, 未落库前不产生实际 FK)。

### 6.3 動作 View (Behavior / State Machine)

#### 6.3.1 Flow 生命周期状态机 (FR-W10.1)

```
草稿(draft) --发布--> 已激活(enabled) --停用--> 已停用(disabled)
                          |                          |
                          +---------手动重新激活------+
```

激活前置校验 (per FR-W10.1 + FR-W14.5 v1.1 新增约束): 图中若存在 `is_placeholder=true` 且未绑定 `agent_id` 的节点, 禁止置为 `enabled`, API-WF-01 PATCH 返回 `400`（占位符未完成绑定）。

#### 6.3.2 Execution 状态机 (FR-W9.1-W9.3)

```
running --成功--> succeeded
running --节点失败且无重试余量--> failed --手动 resume (API-WF-04)--> running (从失败节点起)
running --超时/取消--> cancelled
```

#### 6.3.3 WorkItem 联动三分支 (BR-W-3, W12)

Flow 标签解绑或 Flow 删除时, 对应自动生成的 WorkItem 按其所在容器三分支处理: Backlog 中 → 硬删除; 已排期 Sprint（未开始）→ 先移回 Backlog 再硬删除; 进行中 Sprint → 仅标记 "detached"（保留任务卡, 解除来源追溯), 不做删除 — 因为进行中 Sprint 的任务卡可能已产生下游工时/评论等业务数据, 删除会破坏审计链。

### 6.4 モジュール View (Module)

复用总册 §1.1.4 25-module 矩阵, 本 BD 新增/扩展的模块边界:

| 模块 | 关系 |
|---|---|
| `workflow-engine` (新增) | 承载 W1-W10 图结构定义与执行引擎, 是本 BD 的核心新模块 |
| `automation` (既有, 扩展) | 既有 `AutomationRule` 作为 Flow 的退化 2 层特例, 向后兼容, 不破坏既有数据 |
| `work-item` (既有, 扩展) | 新增标签绑定生成/回收逻辑 (W11-W12), 复用既有 WorkItem CRUD, 仅扩展字段 (见 §4.3) |
| `flow-template-library` (新增, v1.1) | W14 模板 CRUD + 实例化, 与 `workflow-engine` 强依赖（实例化产物是普通 Flow） |
| `chat-bar` (新增, v1.1) | W15 聊天会话 + mock 解析, 依赖 `/api/tmo/*` (ADR-0046) 做会话管理, **不**直接调用 L1 Agent（per "L1↔L1 通信禁止" 派生约束, 聊天栏发起的动作经由既有 L0/TMO 编排层, 不新增旁路） |

### 6.5 ネットワーク View (Network / Deployment)

复用 `BD-CANVAS-AGENT-001` §2.1 既定 5-tier 部署拓扑（UI / BFF / Domain Service / DB / 既有 L0-TMO 服务), 本 BD 不引入新的网络拓扑层或新部署单元。Webhook 入口 (API-WF-05) 需要对外暴露, 部署侧是否需要独立的 API Gateway 限流/WAF 规则 — **【TBD, Design Doc, 见 §8 安全设计 §8.4】**。

## §7 非功能要件 (NFR)

> per `ipa-nonfunctional-requirements` skill: 以下数值为**基本设计提案值（待拍板）**, 非项目已批准基线; SRS 未量化处一律标【TBD】而非编造。验收方法逐条给出。

| NFR ID | 类别 | 要求（提案值, 待拍板） | 验收方法 |
|---|---|---|---|
| NFR-WF-01 | 性能 | Flow 图 CRUD (API-WF-01/02) P95 响应 < 500ms（沿用总册既有 Canvas CRUD 基线, 非本 BD 新定标准） | 性能测试脚本 + APM 采样 |
| NFR-WF-02 | 性能 | 单 Execution 节点间调度延迟 【TBD, SRS 未给出具体数值, 待与总册 §7 性能基线协调后拍板】 | — |
| NFR-WF-03 | 容量 | 单 Flow 节点数上限 【TBD, SRS §7 未声明, 需产品侧确认避免超大图导致渲染/执行性能劣化】 | 边界测试 |
| NFR-WF-04 | 容量 | `execution_history`/`execution_step` 保留周期 【TBD, 见 §4.5 审计策略同一缺口, 影响存储容量规划】 | — |
| NFR-WF-05 | 可用性 | 沿用总册既有可用性基线, 本 BD 不新增独立于总册的 SLA 承诺 | 沿用总册监控 |
| NFR-WF-06 | 可靠性 | 节点级重试 `retry_policy`（FR-W8.1）最大重试次数与退避策略 【TBD, SRS 未给出默认值, Design Doc 阶段需与产品确认默认 policy】 | 单元测试覆盖重试路径 |
| NFR-WF-07 | 扩展性 | Flow 图新增节点 kind 需可插拔扩展（W1 节点类型体系设计目标）, 不要求本版本预留具体扩展点数量 | 代码评审 |
| NFR-WF-08 | 安全性 | 见 §8, 本节不重复 | — |
| NFR-WF-09 | 运维性/监控 | Execution 失败需可观测（沿用总册日志/监控管线), 是否需要专属告警规则 【TBD, Design Doc】 | — |
| NFR-WF-10 | 兼容性 | 既有 `AutomationRule` 数据向后兼容, 迁移脚本需保证 0 数据丢失 | 迁移前后行数/字段比对测试 |
| NFR-WF-11 | 灾备 | 沿用总册既有备份策略, 本 BD 不新增独立于总册的 RTO/RPO 数值 | 沿用总册灾备演练 |

## §8 安全设计 (Security Design)

> per `ipa-security-design` skill: 按实际攻击面逐项列出, 服务端权限校验与前端显示隐藏严格区分; 资料不足处标记【安全确认必要】而非假设已有防护。

### 8.1 认证与授权

- **认证 (Authentication)**: 所有 UI 发起的 API-WF-01~04/06~08 复用既有 Session Token 机制, 本 BD 不新增独立认证方式。
- **授权 (Authorization)**: 复用既有 RBAC, Flow/Template 的读写权限与所属 Canvas/Workspace 权限一致 (per §4.2 RLS `tenant_id` 策略); 前端画面按钮的显示/隐藏（如 SCR-WF-01 编辑按钮）**仅为体验优化**, 真正的写权限校验必须在 BFF 层复核, 不得仅依赖前端隐藏。

### 8.2 Webhook 入口 (API-WF-05) — 攻击面重点

- 现设计仅有静态 `webhook_token` 字符串精确比对, **无 HMAC 签名校验、无 token 定期轮换机制、无请求体大小上限**（后者见 §5.2 已标 TBD）。
- **【安全确认必要】** 该入口是系统对外暴露、无需登录会话即可触发执行的唯一端点, 建议安全评审给出: (a) 是否要求 HMAC-SHA256 签名头, (b) token 轮换周期, (c) 请求体大小上限, (d) 是否需要来源 IP allowlist。本 BD 不预先假设评审结论, 上述四项在评审前均按【TBD】处理, 不建议在评审完成前接入外部生产系统。

### 8.3 Agent 占位符激活预检 (FR-W14.5/W10.1 派生)

- 服务端强制校验: Flow 中存在 `is_placeholder=true` 且 `agent_id IS NULL` 的节点时, 禁止 `enabled=true`（见 §6.3.1）。此校验必须在 BFF/Domain Service 层强制执行, 不得仅由前端 UI 阻止提交, 防止绕过前端直接调用 API-WF-01 PATCH。

### 8.4 输入校验与常见攻击面

| 攻击面 | 现状 | 备注 |
|---|---|---|
| CEL 条件表达式 (`condition_expr`, W4) | 复用既有 `AutomationRule.condition_expr` 沙箱执行环境, 不新增独立 CEL 执行器 | 沙箱逃逸风险由既有 CEL 引擎既定边界承担, 本 BD 不重新评估既有引擎安全性 |
| `{{node.<id>.output.<field>}}` 数据映射语法 (W6) | 需防止映射表达式被用于越权读取其他 Flow/其他租户的数据 | **【安全确认必要】**: 映射解析器是否严格限定在同一 Execution 上下文内取值, 需 Design Doc 明确并附单元测试证据 |
| 聊天栏输入 (W15, mock 解析) | v1 为规则/关键词匹配, 非真实 LLM, 无 Prompt Injection 攻击面（因无真实模型推理） | 待未来接入真实 NLU/LLM 时需重新评估 Prompt Injection, 本版本不适用 |
| Webhook 外部输入 body | 透传给下游节点 `input_bindings`, 需防止注入内容通过节点动作（如通知发送）产生二次注入 | **【安全确认必要】**, 具体转义/校验规则待 Design Doc 补齐 |

### 8.5 审计与敏感数据

- `execution_history`/`execution_step` 记录执行输入/输出, 若节点涉及敏感字段（如 WorkItem 中的隐私数据）, 是否需要脱敏存储 — **【TBD, Design Doc, 见 §4.5 同一缺口】**。
- Secret/密钥管理（如未来 webhook HMAC 密钥的存储与轮换）复用总册既有密钥管理机制, 本 BD 不新建独立密钥库。

## §9 守门合规 (Guard Compliance) + TBD 追踪矩阵

### 9.1 守门 #13 W/T/M 三分类声明

已在 §4.1 声明: Master 5 张 / Transaction 3 张 / **Work 0 张**。

**Work 类 0 张理由**: SRS 对本域 8 张表均未提出"临时性、会话级、需 TTL 自动清理"的业务需求 — Flow 定义类表（Master）需长期保留并支持版本回溯, Execution 类表（Transaction）需长期保留供审计, `chat_session`（v1.1 新增）虽是会话级数据但 SRS 未要求自动过期清理机制（仅要求关联 `actor_session_id`), 故暂归入 Transaction 而非 Work。若后续确认 `chat_session` 需要 TTL 自动清理策略, 则应重新分类为 Work — **本项分类前提标记【TBD, Design Doc 确认 `chat_session` 保留策略后可能改变本分类】**。

### 9.2 守门 #13a L1↔L1 通信禁止派生约束

`chat-bar` 模块 (W15) 不直接调用 L1 Agent, 经由既有 `/api/tmo/*` (ADR-0046) L0/TMO 编排层转发, 见 §6.4。

### 9.3 TBD 追踪矩阵 (全量汇总, 按来源分类)

> 本表汇总本 BD 全文出现的所有【TBD】/【安全确认必要】标记, 以及 SRS §10 已知风险 #1-#11 的继承状态。任何一项在本 BD 中均未被擅自假设或裁决。

| # | 来源 | 内容 | 影响 | 归属阶段 |
|---|---|---|---|---|
| T-01 | SRS 风险#1 | Flow 编辑器复用主画布 viewport 还是独立子画布 (FR-W1.3) | 影响 W1-W10 全部 UI 实装路径, 本 BD §3 画面设计按"独立子画布"假设草拟, 待拍板后可能需重画 SCR-WF-01~06 | Design Doc 阶段拍板 |
| T-02 | SRS 风险#2 | 后端真实持久化引擎缺失, v1 为前端 mock | Execution 无法跨会话可靠恢复, 真实生产使用前必须补齐; 本 BD §4 DDL 按"真实后端"设计, 但落地时序取决于此风险解决时间点 | P0 阻塞项, 需真实后端设计 |
| T-03 | SRS 风险#3 | "从失败节点重跑"(API-WF-04) 对非幂等动作 (`create_worktree`) 的安全性未澄清 | 可能导致重复创建 worktree | Design Doc 评估幂等性标记机制 |
| T-04 | SRS 风险#4 | 两个 Flow 标签表达式重叠命中同一标签组合 | 用户可能得到 2 张语义重复的任务卡 | v1 不做去重, P1 观察后决定 |
| T-05 | SRS 风险#5 | detached 状态 (FR-W12.3) 是否需要 Lead 人工确认交互 | 影响 5 域 Lead 实际处理体验 | v1 只读角标提示, 待真人反馈细化 |
| T-06 | SRS 风险#6 | A12 多人协同编辑 CRDT 选型未拍板 | 若 Flow 编辑器需多人协同则依赖此项 | 依赖外部 `SRS-CANVAS-AGENT-001`, 本 BD 不重复设计 |
| T-07 | SRS 风险#7 | 总册 `SRS-CANVAS-001` 尚未正式收录本专题为"三核心" | 总册措辞仍以"双核心"为主 | 本次 wrap-up 仅做最小索引同步（见 §附录/wrap-up 说明), 完整总册改写留后续 |
| T-08 | SRS 风险#8 (v1.1) | W15.2/15.3 v1 均为 mock 规则化实现, 与用户对"LangGraph 智能控制"预期可能有落差 | 真实使用体验可能不及预期 | v1 交付 mock 版本, 真实 LLM 接入时间点待 P2 拍板 |
| T-09 | SRS 风险#9 (v1.1) | `routing_mode="dynamic_agent"` 决策可复现性/确定性未澄清 | 影响 e2e 测试稳定性与执行历史可审计性 | Design Doc 阶段明确是否要求确定性 mock |
| T-10 | SRS 风险#10 (v1.1) | AAA/spec/superpowers 3 套默认模板节点清单为 agent 合理推断, 未经真人逐节点确认 | 实际落地节点清单可能需调整 | §11 签字栏标记 Draft, 评审时逐节点确认 |
| T-11 | SRS 风险#11 (v1.1) | spec 模板"评审不通过"回指边循环不做自动死循环检测 | 用户可能手动搭建出无法退出的循环, 消耗执行资源 | v1 依赖节点级重试上限, Flow 级最大循环次数硬限制留 P2 评估 |
| T-12 | 本 BD §4.5 | 是否需要独立于 `automation_flow_versions` 的专属审计表 | 影响审计数据模型是否需要扩展 | Design Doc |
| T-13 | 本 BD §5.2 | API-WF-03 超时秒数无总册统一数值可引用 | 影响客户端超时/重试策略实装 | 待总册 §5.6 数值统一确认 |
| T-14 | 本 BD §5.2 | Webhook 请求体大小上限未定义 | 影响 API-WF-05 输入校验实装 | Design Doc 结合总册请求体上限确认 |
| T-15 | 本 BD §5.2/§8.2 | Webhook 签名/token 轮换策略缺失（仅静态 token 比对） | 安全评审前不建议接入外部生产系统 | 安全评审阶段, 见 §8.2 |
| T-16 | 本 BD §5.3 | Execution 节点级状态是否需要秒级实时刷新（而非轮询） | 影响是否需要扩展既有 WebSocket 事件类型 | Design Doc |
| T-17 | 本 BD §7 | NFR-WF-02/03/04/06/09 具体量化数值（调度延迟/节点数上限/保留周期/重试默认值/告警规则） | 影响容量规划与运维实装 | Design Doc, 需产品/SRE 协同拍板 |
| T-18 | 本 BD §8.4 | `{{node.<id>.output.<field>}}` 映射解析器是否严格限定同 Execution 上下文取值 | 若无严格限定, 存在跨 Flow/跨租户越权读取风险 | 【安全确认必要】, Design Doc + 单元测试证据 |
| T-19 | 本 BD §8.4 | Webhook 外部输入 body 透传下游节点的二次注入防护规则 | 若节点动作含通知发送等场景, 可能产生二次注入 | 【安全确认必要】, Design Doc |
| T-20 | 本 BD §8.5 | Execution 记录中敏感字段是否需脱敏存储 | 影响审计数据的隐私合规性 | Design Doc |
| T-21 | 本 BD §9.1 | `chat_session` 是否需要 TTL 自动清理, 影响 W/T/M 分类是否需从 Transaction 改判 Work | 影响 §4.1 分类结论的稳定性 | Design Doc 确认后可能需修订本 BD §4.1 |
| T-22 | 本 BD §6.1(FR-W2.1) | 手动触发重复点击是否需要防抖(去重) | 影响并发点击时 Execution 是否重复生成 | Design Doc |
| T-23 | 本 BD §6.1(FR-W2.4) | 画布事件触发风暴(高频操作)是否需要 debounce | 影响事件驱动 Execution 生成频率与系统负载 | SRS 已知缺口, Design Doc |
| T-24 | 本 BD §6.3.1(FR-W4.3) | Merge 节点 join 模式下部分 inbound 分支永不到达时的超时策略未定 | 影响 join 型 Merge 节点是否会无限等待 | SRS 已知缺口, Design Doc |
| T-25 | 本 BD §6.3.1(FR-W7.1) | 子流程引用环检测算法未定 (仅约束引用深度 ≤5) | 若循环检测算法缺失, 深度限制外仍可能构成执行期死循环 | SRS 已知缺口, Design Doc |
| T-26 | 本 BD §6.3.2(FR-W9.1) | `execution_history`/`execution_step` 写入失败时的 at-least-once 重试机制未定 | 若不保证至少一次写入, 执行历史可能出现记录缺失 | Design Doc 落地细节 |
| T-27 | 本 BD §3.2(SCR-WF-04)/§6.3.2(FR-W9.3) | `execution_step` 输入/输出 JSON 过大时是否分页/截断未定 | 影响侧栏详情面板渲染性能与可用性 | Design Doc |
| T-28 | 本 BD §6.3.3(FR-W11.4) | 标签绑定事件处理中途失败的重试/补偿机制未定 | 影响派生任务卡与命中结果的最终一致性保证 | Design Doc |
| T-29 | 本 BD §6.4(FR-W15.3) | L0 不可达/超时时的降级策略未定(回退 static_cel 默认分支, 或整体失败) | 影响 dynamic_agent 路由模式的可用性与容错行为 | Design Doc |
| T-30 | 本 BD §3.2(SCR-WF-03) | 底部聊天栏是否需跨页面持久悬浮未定 | 影响聊天会话上下文在页面切换时是否保留 | Design Doc |
| T-31 | 本 BD §3.2(SCR-WF-03) | 聊天栏自然语言输入长度上限未定 | 影响输入校验规则与超长文本处理方式 | Design Doc |
| T-32 | 本 BD §4.2(`chat_session.routing_decision`) | L0 动态路由决策依据 JSON 字段粒度未定 | 影响决策可审计性与前端展示细节, 与 T-09 决策可复现性相关但非同一问题 | Design Doc |
| T-33 | 本 BD §8.2 | Webhook 入口是否需要来源 IP allowlist 未定 | 影响 Webhook 攻击面缓解措施完整性 | 【安全确认必要】, 安全评审阶段 |
| T-34 | 本 BD §6.5(网络视图)/§8.4 | Webhook 入口部署侧是否需要独立 API Gateway 限流/WAF 规则 | 影响 Webhook 攻击面在网络层的缓解措施完整性 | Design Doc, 见 §8 安全设计 |
| T-35 | 本 BD §附录 D | 与本 BD 配套的测试设计文档尚未创建 | 影响 §10 追溯矩阵 Test Case 列能否回填 | Design Doc 后续阶段, 依据 `ipa-test-case` skill 产出 |

## §10 追溯矩阵 (Traceability Matrix)

> 覆盖全部 54 项 FR (W1-W15), 而非仅 v1.1 新增 12 项。Design/Test 列为本 BD 交付时点的映射, Test Case ID 留待测试设计阶段（`ipa-test-case` skill）编写后回填。

| FR 组 | FR 数 | 对应设计章节 | 对应表/API/画面 | Test Case (待补) |
|---|---|---|---|---|
| W1 节点类型体系 | 4 | §2.1, §3.2(SCR-WF-01), §4.2(`flow_node`) | `flow_node`, SCR-WF-01 | 【TBD, 测试设计阶段】 |
| W2 触发节点 | 4 | §3.2(SCR-WF-06 节点配置侧栏), §5.1(API-WF-05) | `flow_node`(kind=trigger), API-WF-05 webhook | 【TBD】 |
| W3 动作节点 | 3 | §3.2(SCR-WF-06 节点配置侧栏), §5.4(module 联动) | `flow_node`(kind=action) | 【TBD】 |
| W4 分支与条件 | 3 | §6.3.1, §8.4(CEL) | `flow_edge`(condition_expr) | 【TBD】 |
| W5 循环与批处理 | 2 | §9.3(T-11 循环风险) | `flow_node`(kind=loop) | 【TBD】 |
| W6 变量与表达式传递 | 3 | §8.4(数据映射语法) | `flow_edge`(data_mapping) | 【TBD】 |
| W7 子流程与复用 | 2 | §4.2(`automation_flow`自引用) | `automation_flow` | 【TBD】 |
| W8 错误处理与重试 | 3 | §6.3.2, §7(NFR-WF-06) | `execution_step`(retry_policy) | 【TBD】 |
| W9 执行历史与调试 | 3 | §5.1(API-WF-03), §3.2(SCR-WF-04) | `execution_history`, SCR-WF-04 | 【TBD】 |
| W10 激活状态与版本管理 | 3 | §6.3.1, §4.2(`automation_flow_versions` SCD2) | `automation_flow_versions` | 【TBD】 |
| W11 标签绑定任务卡 | 5 | §6.3.3, §3.2(SCR-WF-05 Flow Tags 管理面板), §4.3(WorkItem 扩展) | `flow_node`(tag_binding_expr), WorkItem, SCR-WF-05 | 【TBD】 |
| W12 Backlog/Sprint 联动 | 5 | §6.3.3(BR-W-3 三分支), §3.2(SCR-WF-05) | WorkItem 状态字段, SCR-WF-05 | 【TBD】 |
| W13 数据一致性 | 2 | §6.2 | 跨表一致性校验逻辑 | 【TBD】 |
| W14 默认工作流模板库 (v1.1) | 7 | §3.2(SCR-WF-02 模板选择器), §4.2(`flow_template`), §5.1(API-WF-06/07) | `flow_template`, SCR-WF-02 | 【TBD】 |
| W15 智能控制+聊天栏 (v1.1) | 5 | §4.2(`chat_session`), §5.1(API-WF-08), §3.2(SCR-WF-03 底部聊天栏), §6.4 | `chat_session`, SCR-WF-03 | 【TBD】 |
| **合计** | **54** | — | — | — |

## §11 签字栏 (Signature Block)

| 角色 | 姓名/代签 | 状态 | 日期 |
|---|---|---|---|
| Ulysses (业务 owner) | Say世意（D-Boy） | 已确认 | 2026-09-13 |
| 5 域 Lead（跨域） | Say世意（D-Boy） | 已确认 | 2026-09-13 |
| PM | Say世意（D-Boy） | 已确认 | 2026-09-13 |
| SRE | Say世意（D-Boy） | 已确认 | 2026-09-13 |
| Dev Lead | Say世意（D-Boy） | 已确认 | 2026-09-13 |

**本文档为 ULYS-28 issue 委托的设计文档交付物, 由 agent Sonnet 撰写。5 个角色（Ulysses/5 域 Lead/PM/SRE/Dev Lead）均已由项目所有者 Say世意（D-Boy）本人确认签字（2026-09-13，非代签）, 状态由 Draft 转为已签字。§9.3 TBD 追踪矩阵中的全部 35 项为设计层面的候选方案标注, 不因签字栏完成而自动裁决, 仍需在详细设计阶段逐项拍板。**

## §12 修订履历 (Revision History)

| 版本 | 日期 | 变更摘要 | 作者 |
|---|---|---|---|
| v1.0 | 2026-09-13 | 初版交付, 覆盖 SRS-CANVAS-WORKFLOW-001 v1.1 全部 54 项 FR (W1-W15), §0-§12 + 附录完整章节结构, 8 张表 W/T/M=5/3/0, 8 个 REST API, 35 项 TBD 追踪矩阵 | Sonnet (agent) |
| v1.0.1 | 2026-09-13 | §11 签字栏: Ulysses（业务 owner）角色由 Say世意（D-Boy）本人确认签字, 其余 4 角色（5 域 Lead/PM/SRE/Dev Lead）仍待拍板; 未改动正文其他章节 | Sonnet (agent), per D-Boy 确认 |
| v1.0.2 | 2026-09-13 | §11 签字栏: 剩余 4 角色（5 域 Lead/PM/SRE/Dev Lead）由 Say世意（D-Boy）本人确认签字, 签字日期统一 2026-09-13, 非代签; 5 角色全部签字完成; 未改动正文其他章节, §9.3 TBD 矩阵 35 项状态不变（仍待详细设计阶段逐项拍板） | Sonnet (agent), per D-Boy 确认 |

## 附录

- 附录 A：本文档所引用的上位文档 —— `docs/requirements/SRS-CANVAS-WORKFLOW-001.md` (v1.1, 分支 `agent/sonnet/ulys-15` 合并入本分支, 详见交付说明)。
- 附录 B：本文档所引用的同级文档 —— `docs/design/BD-CANVAS-AGENT-001.md`（5-tier 架构/5-View 体系/WebSocket 通道均直接复用其既定设计, 本 BD 不重复定义）、`docs/design/BD-CANVAS-GAMIFY-001.md`（章节结构参照）。
- 附录 C：本文档所引用的总册文档 —— `docs/design/BD-CANVAS-001.md`（§1.1.4 模块 view 索引, 本次已同步新增交叉引用, 见交付说明）。
- 附录 D：与本 BD 配套的测试设计文档 —— 【TBD, 尚未创建, 建议依据 §10 追溯矩阵与 `ipa-test-case` skill 后续产出】。
