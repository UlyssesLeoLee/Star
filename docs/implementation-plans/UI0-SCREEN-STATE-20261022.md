# UI-0：Worktree-first 界面与状态设计

**版本：** v0.1
**日期：** 2026-10-22
**设计基线：** 本地 dev 源码提交 17b3d06539f0281b8a5f9e7a1576864487b26337
**状态：** 设计交付；页面实现和生产闭环仍未完成
**修订人：** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## 0. 目的和结论

本设计把多 Agent 并行开发中的 Worktree 归属、活动状态、checkout 变化和生命周期风险放到 Project 的主导航中。导航顺序固定为 **Project → Engineering Run → Worktree → Branch**。Project 与 Run 是选择/展开层；Worktree 是执行和协作焦点；Branch 是 Worktree 当前本地 checkout 的末级观察。远端 SCM Branch 仅作为可选关联元数据。

Run 是跨 App 的授权和协作范围。用户只有在 Worktree 身份、Run 绑定、授权与版本复验成功后，才进入该 Run 的工作区。Task Card、Canvas、Inbox、Workflow/Schedule、BI/Benchmark 与授权 Plugin Apps 在 Run 内同级；Task Card 中的 CLI 绑定到具体 Task、Run、Worktree 和 Session。固定底栏明确区分 WORKTREE 与 GLOBAL，全局范围不扩大权限。

本设计区分已在源码出现的行为、尚未连接的设计入口和未验证的业务事实。UI-0 不解除 Run 生命周期、CLI、聊天发送、Canvas 写入或 BI 的 fail-closed 门。

## 1. 基线证据与范围

本设计审查的主要源码证据：

| 证据 | 观察 |
|---|---|
| frontend/src/components/run/RunWorkspaceShell.tsx | 规范化 Project/Run/Worktree 路由；读取 RunContext 并校验路径范围；清理焦点并重置 Chat/App 状态；无 Worktree focus 时显示授权 Worktree 选择区；有 focus 时显示同级 App tab。 |
| frontend/src/lib/run/runDirectoryApi.ts | Run Directory 的 Worktree、RunContext、授权与版本投影类型，以及读取 API 客户端。此设计不把前端类型当作持久化 owner 证明。 |
| frontend/src/lib/run/runDirectorySession.tsx | Run Directory 会话与当前授权边界的前端载体。需要在后续验收中核对旧会话、401 和授权代次。 |
| frontend/src/components/run/RunTaskCardsPanel.tsx | Run Task Card 面板和服务端 capability 门。具体 Agent Runtime、取消/恢复、验证和结果回写仍须由各自 owner 验收。 |
| frontend/src/components/run/RunBiPanel.tsx | 当前 BI 入口仍以任务事件投影为主，不能据此宣称生产聚合或完整 Benchmark。 |
| bff/src/routes/chat.rs | 当前通用聊天发送路径仍是内存 transcript/stub；Run Shell 的聊天输入与发送按钮也保持禁用。 |
| docs/implementation-plans/UI-PLAN-RUN-WORKTREE-NEXT-20261021.md §8.4 与 WBS-UI-PLAN-NEXT-20261022.md | 记录当前 Worktree/Run binding、provider、Runtime、DB/RLS、Hook 与 BI 的未关闭门。 |

本地 CodeGraph CLI 报告该 checkout 的索引已更新；CodeGraph MCP 在本会话返回 Transport closed，没有可调用的 check_index_coverage 接口。因此源码结论仅覆盖上表列出的文件和明确引用的行为，不作全仓无遗漏或所有调用点均已核实的声明。Rust 桌面端候选框架的 Context7 查询也未能连接；本阶段不锁定具体 UI crate，不将未查验的框架特性写成事实。

基线中可直接观察到：

- RunWorkspaceShell 先清除旧焦点和 scoped UI 状态，再请求授权 RunContext；路由身份与 RunContext 不一致时拒绝设置焦点。
- Run 无焦点时读取授权 Worktree 首页面，不挂载 Run App；如有下一页，当前界面提示从左侧目录继续。
- 有焦点时显示 App tabs，但当前只有 Task Cards 与 BI 面板有明确组件接入；Canvas、Inbox、Workflow 和 Plugin Apps 仍标为待迁移。
- Task Card 的可读/创建入口受服务端 capability 控制；CLI Runtime、执行准入、取消/恢复、独立验证和结果回写未形成闭环。
- Chat scope 选择已存在局部界面，但聊天输入和发送禁用；这不是可用的聊天 Runtime。
- 目录状态投影中出现的名字、Branch、machine state、Agent/Session 与版本不等于完整可信的 lock、dirty、ahead/behind、时间戳、freshness 或生命周期回执。

## 2. 屏幕清单

| 屏幕/表面 | 进入条件 | 展示内容 | 未满足条件时 |
|---|---|---|---|
| Project 选择与目录 | 用户有可读 Project 列表 | Project 身份、授权摘要、Engineering Run 树；Project 名称只取 owner 投影 | 名称缺失时显示稳定 ID；授权失败时不加载子级 |
| Project Worktree Index | Project 范围授权；跨 Run 行含可信 Run ID 与 binding/version | 汇总 Worktree、归属、Agent/Session、Work Item、Branch/ref、freshness 与风险筛选 | 缺 Run 身份或版本时标为部分/不可用；不推断归属、不允许跨 Run 生命周期操作 |
| Run 概览（无 focus） | RunContext 通过 Project/Run 授权检查，且 focus 为空 | Run 摘要、授权 Worktree 列表、选择 Worktree 的明确入口 | 仅显示已证实的目录事实；不挂载 App tab 或读取 App 数据 |
| Worktree 验证 | 用户选择具体 Run Worktree | 身份、binding、授权、版本和 freshness 的验证进度 | loading/error/denied/stale/conflict 独立显示；不乐观进入工作区 |
| Worktree 工作区（focus） | 当前 Project/Run/Worktree/版本复验通过 | Worktree 标题和 Branch 摘要；Run 级同级 App tabs；按需加载当前 tab | 当前 tab 不可用时显示 owner 原因；切换 focus 时先撤销旧上下文 |
| Branch 详情 | 当前 Worktree 下有本地 Branch/ref 观察 | 本地 branch 名、HEAD、相对基线差异和观测时间（仅在 Git owner 提供时） | 本地与远端身份分开展示；未知值不从远端关联推断 |
| Task Card 与卡内 CLI | Run Task owner capability 开放；Task 绑定当前 Run/Worktree | Task 状态、分配与 Session 证据；CLI 输出、取消/恢复和验证证据区 | provisioner、sandbox、Hook、预算、Session 或 durable receipt 缺失时禁用执行并说明原因 |
| Canvas | Run focus 通过；Canvas owner 查询成功 | 视口内图形、布局、typed Worktree/Task/Branch 引用和对象深链 | 未解析引用明确标注；Canvas 不能代替引用对象的 owner API 写业务字段 |
| Workflow / Schedule | Run focus 和对应 owner capability 通过 | Rule 配置、Occurrence、Run admission、执行、取消和恢复分态 | 配置已保存不代表 trigger 或执行；capacity/授权/owner 事实未知时显示等待/不可用 |
| BI / Benchmark | BI owner 查询通过并返回定义、coverage、样本与 freshness | 聚合结果、来源版本、范围、coverage 和更新时间 | 事件列表不称为 BI；数据缺口和过期状态必须可见 |
| Plugin Apps | Run 授权和插件 capability/consent 通过 | 插件名称、版本、允许能力、运行状态与资源预算 | 撤权后阻止新调用并取消/排空现有工作；native code 不进主进程 |
| 固定聊天栏 | 所在 Run/Worktree 身份明确 | GLOBAL 或 WORKTREE 范围、目标 ID、授权/Runtime 状态 | 无 focus 时不能选择 WORKTREE；Runtime/send 未验收前输入禁用 |

## 3. 状态字典

| 状态 | 含义 | UI 行为 |
|---|---|---|
| loading | owner 查询仍在进行 | 显示进度；允许取消/切换，不显示上次身份下的内容 |
| ready | 当前身份和版本已验证 | 只显示该 scope 的 owner 投影；写操作仍需单独 capability/receipt |
| empty | owner 明确返回空集 | 显示有证据的空态和可用的下一步 |
| unavailable | provider/API/capability 未提供 | 禁止动作，说明缺失的 owner/capability |
| denied | 当前 actor 没有权限 | 不显示超范围数据；重新授权后重新取数 |
| unknown | owner 未提供该事实 | 显示“未知”，不推导锁、dirty、完成或健康状态 |
| stale | 版本/观察时间过期或校验失败 | 禁止危险动作，显示 last observed 与刷新路径 |
| conflict | 当前对象版本与预期版本不符 | 保留用户输入，刷新 owner 事实，要求重新预览 |
| cancelling | 正在停止旧请求/订阅/任务 | 保留安全状态，不启动同一 scope 的新写操作 |
| draining | owner 正在排空 stream/PTY/plugin/runtime | 不把“取消请求已发送”显示为进程已停止 |
| accepted | owner 返回可验证的持久回执 | 显示回执、事件 ID、实体版本和审计/深链 |
| failed | owner 明确返回失败结果 | 显示可恢复原因和安全重试入口；重试使用原幂等身份或新确认 |

## 4. UI 与 owner/API 对照

| UI 数据/操作 | 权威 owner | 必要范围/证明 | UI 边界 |
|---|---|---|---|
| Project 列表、Project 授权 | Project/Authorization owner | project_id、actor、permission version | Project 名称不能来自 seed；scope 变化时清空后代缓存 |
| Run 身份、授权、capabilities | Run Directory / Run owner | project_id、engineering_run_id、context version、authorization roles/versions | 无效 RunContext 时不给子 App 发请求 |
| Worktree 列表与 Run 绑定 | Worktree Directory / Binding owner | worktree_id、engineering_run_id、binding ID/version、validity interval | Project 级 Worktree 投影没有 Run ID 时不进入 Run Index |
| Git 分支、HEAD、dirty/ahead/behind | Git/checkout observation owner | checkout identity、local path identity、base ref、observed-at | checkout ref 和远端 Branch 元数据不混成一个对象 |
| Agent/Session/Runtime | Agent Session / Runtime owner | agent_id、session_id、runtime_id、current state/version | PTY 存在不等于 Agent 在运行；session 结束不等于 Task 完成 |
| Task 与分配 | Run Task owner | Task ID、Run ID、Worktree ID、assignment/version、Actor permission | UI 显示 owner 事实和 receipt；不可本地写出“已分配”假状态 |
| 卡内 CLI | Agent Runtime / Execution / Hook / Budget owners | Task/Run/Worktree/Session、policy snapshot、budget reservation、sandbox proof | 输入只在所有 gate 通过后启用；输出限字节并可审计 |
| Canvas 布局与对象引用 | Canvas owner | Canvas ID、Run ID、typed EntityRef、Canvas version | 布局/连接可以由 Canvas 管；被引用业务实体经其 owner API 变更 |
| Rule/Occurrence/Execution | Schedule / Run Execution owner | immutable run-as actor、Rule version、occurrence ID、Run binding | Rule 配置、trigger、admission、executor 与 outcome 分开显示 |
| Hook 策略与决策 | Rust Hook evaluator / HookSet owner | Project/Worktree inheritance、Run snapshot、HookSet version/digest、event ID | 高级设置 → Hooks 是编辑入口；Run/BI 只读有效策略和决策 |
| BI / Benchmark | BI definition 与 event-ingress owner | definition/version、scope、sample count、coverage、freshness、replay evidence | 仅对有持久 owner intake 的数据显示聚合结果 |
| Plugin | Plugin host / capability registry | plugin ID/version、capability grant、consent、resource budget | 插件不能改变 Host 权限或取代 native Hook；撤权后 cancel/drain |
| Chat | Chat session/runtime owner | scope、Project/Run/Worktree 目标、actor permission、session generation | GLOBAL 仍受 actor/Project 权限限制；当前未连接 Runtime 时禁用发送 |

## 5. Focus 与取消顺序

Worktree focus 是一个受验证的运行上下文，不是前端选中行的别名。状态转换按以下顺序执行：

1. 用户选择 Worktree；先建立 pending_focus，不挂载 Run Apps。
2. 取消旧 Worktree 的 API 请求、事件订阅、Canvas viewport、终端输出与插件任务；对可取消工作请求 cancel 并等待 owner 的 drain/receipt。
3. 增加 focus_generation；旧 generation 回调即使迟到也丢弃。
4. 清空上一 Worktree 的 Task、Canvas selection、Inspector、Chat messages、CLI buffer 和受限缓存；GLOBAL 的 Project/Run 身份保持单独 scope。
5. 通过 owner 获取当前 RunContext、Worktree binding、授权/政策版本与 checkout 观察；校验 Project/Run/Worktree 完全匹配且没有过期/冲突。
6. 验证成功后将 focus 写入会话，并挂载当前 Run tab；默认仅请求用户当前打开 App 所需的数据。
7. 验证失败时保留在 Run overview，焦点为空；显示 denied/stale/conflict/unavailable 的 owner 原因。
8. 用户切换 App tab 时取消并排空旧 tab 的工作；不卸载/重建整个 Run 的无关数据，也不把旧 App 的迟到事件合并到新 tab。

当前 Web Shell 已执行 RunContext 获取、路由范围校验、焦点设置/清理和请求 AbortController；本节中的 stream drain、跨 App generation fence、完整缓存清理与 owner side-effect recovery 是目标合同，尚未实现验收。

## 6. 键盘、焦点和屏幕阅读

### 6.1 Project 树

- 树节点声明层级、展开状态、选择状态与 disabled 状态；Expand/Collapse 与 Select/Open 是分开的可访问动作。
- Up/Down 移动可见节点；Left 收起或回到父级；Right 展开或进入第一个子级；Home/End 移动到当前可见集合首尾；输入字符支持有界 typeahead。
- Project、Run、Worktree、Branch 的可访问名称包含稳定显示名和必要的类型/状态，不把 UUID 作为唯一读屏文本。
- lazy load 显示 Loading children 状态；失败后提供可聚焦重试按钮；分页追加后焦点不跳回树根。
- 行状态除了颜色还使用文字、图标名称或 renderer 对接的平台无障碍公告机制；unknown 与 stale 需明确读出。

### 6.2 Run Apps 与详情操作

- Tab 组件遵循所选 renderer 的无障碍键盘模式并暴露可访问的选中态；只渲染可见 active panel，保留用户离开的 App 选择但不保留已失效身份的数据。
- 进入工作区后焦点位于页面标题或有标签的 App tablist；关闭 Inspector/对话框后焦点返回发起控件。
- 危险动作预览列出目标 Run/Worktree、当前状态、版本、将执行操作、阻塞项及可恢复后果；提交控件必须能由键盘到达。
- 聊天范围控件具有状态语义、scope 名称和目标身份；无 Worktree focus 时 WORKTREE 不在可选集合中。
- 对长时间操作公告进度和可取消状态；取消确认后公告“请求已提交”与 owner “已停止”这两类事实的区别。

## 7. 可用性门与当前差距

- 只选 Project/Run 不请求 Task、Canvas、BI 或 Plugin 内容；只加载导航所需的 Run/Worktree 目录摘要。
- Worktree 选择不会乐观挂载；授权、binding/version 或当前会话不确定时 focus 为空。
- Run Worktree 目录必须支持 next_cursor、搜索与明确的 loading/error/retry；当前 Run Shell 只呈现首批结果，尚无右侧完整分页选择器。
- Run App tabs 的外观不代表实现；当前 Task Cards/BI 面板与其他待迁移 tab 分开呈现。
- 生命周期按钮在真实 Run provider、Repository binding、目标 DB/RLS 与 durable owner receipt 通过前必须 disabled。
- CLI 输入在 Session provisioner、隔离 sandbox、Rust native Hooks、资源预算、取消/drain、独立验证与结果写回未满足前 disabled。
- Canvas、BI、Schedule、Plugin 的局部页面/fixture/事件列表不能称为同级 Run App 生产闭环。
- Project Worktree Index 只接收有可信 Run ID 与 binding/version 的记录，不按 branch 名、路径或相似名称猜测 Run。

## 8. UI-0 验收清单

设计资产齐备后，下一实现批次应逐项验收：

1. 以代表性 Project → Run → Worktree → Branch 树验证导航顺序、选择/展开分离、键盘和读屏；未提供 Project display name 时不造名称。
2. 在 Run 无 focus 状态验证零 Run App mount、零 Run App data request；允许受限的 Run/Worktree 目录请求。
3. Worktree focus 需要在服务端重新授权和核对 binding/version；冲突、过期和撤权不允许挂载 App。
4. 运行快速连续切换，验证旧请求取消、订阅/输出 drain、旧 generation 回调丢弃、旧 scope 数据清空。
5. 对每个字段和动作，提供 owner/API、actor/scope、version/freshness、回执或明确缺失状态。
6. 检验 Task Card、Canvas、Workflow、BI、Plugin 的 tab 内容与 owner contract 一一对应；不允许用 UI tab 名声称业务已可用。
7. 键盘可达、焦点恢复、live status、颜色以外的状态标识和屏幕阅读顺序通过可复测的辅助技术检查。
8. 日志、列表、Canvas viewport、事件订阅和缓存采用确定的上限；没有实际 target-device measurements 前不得设定绝对内存/延迟通过结论。

## 9. 已知缺口

- Rust UI 的具体 renderer crate 与插件 runtime 还未选定；Context7 当前不可用，本设计不编造框架性能或无障碍结论。
- 当前 target machine 硬件/显示器/内存库存和实际负载尚未记录，RSS、启动时间、交互延迟阈值待基准后审定。
- 图数据库索引覆盖检查端点当前不可用；源码证据边界见 §1，不作全仓 exhaustive claim。
- UI-1 到 UI-7 的界面/业务实现及 owner API 验收仍按 WBS 分阶段执行；本文件本身不表示这些能力已落地。

## 10. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 UI-0 界面/状态包、owner/API 对照、焦点与取消流程、无障碍行为、fail-closed 门和当前实现差距；不声称 UI 或生产能力已完成。 | 用户要求当前开发轮完成后，为新版本开展 UI 规划 |
