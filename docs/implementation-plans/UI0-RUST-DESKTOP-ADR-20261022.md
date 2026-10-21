# UI-0 ADR：Rust 桌面前端、IPC 与资源边界

**版本：** v0.1
**日期：** 2026-10-22
**设计基线：** 本地 dev 源码提交 17b3d06539f0281b8a5f9e7a1576864487b26337
**状态：** 架构边界已定；渲染器与目标机性能门仍开放
**修订人：** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## 0. 决策摘要

产品目标是 Rust 原生桌面 UI；具体 Rust renderer crate 和绘制后端在目标机对比前不锁定。Rust Desktop Host 负责有界调度、授权上下文传递、取消/drain、数据投影与渲染器适配；业务事实仍由对应的 Project、Run、Worktree、Task、Canvas、Hook、BI 和 Infrastructure owner 持久化与授权。桌面缓存只用于呈现，不能成为第二套业务事实来源。

本 ADR 不选定具体 Rust UI crate 或渲染后端。候选方案必须在目标设备上用同一语义和确定数据集比较启动、导航、滚动、终端、Canvas、多 Agent 并发、内存和无障碍。现有 Tauri + React/Web 仅作为对照基线；它不构成 Rust 原生 UI 的实现证明。

## 1. 范围与约束

适用范围：

- Project → Engineering Run → Worktree → Branch 导航，以及 Worktree focus 后的 Run 级同层 App。
- Task Card 内 CLI、Canvas、Workflow/Schedule、BI/Benchmark、Plugin Apps、底栏 WORKTREE/GLOBAL 聊天范围的状态协调。
- 桌面进程的 API 请求、订阅、任务调度、缓存、输出缓冲、焦点切换、取消和资源观测。
- Rust 原生 fail-closed Hook 与可视化 Hook 设置的桌面交互边界。

不在本 ADR 内定案：

- 具体渲染 crate、GPU 后端、终端模拟器或 Canvas 绘制库。
- 业务数据库 schema、Run/Task/Worktree lifecycle owner、Agent sandbox、Rust server transport。
- 无目标设备测量依据的 RSS、延迟、并发数或队列阈值。

固定产品门：

1. 导航恒为 Project → Engineering Run → Worktree → Branch，不随宿主 SCM 连接改变。
2. Project 或 Run 的选择本身不加载 Run Apps；只有授权、binding 和版本复验通过的 Worktree 才能获得焦点并挂载其 Run 工作区。
3. Task Card 和 Canvas 在 Run 工作区同级；CLI 绑定 Task、Run、Worktree 和 Session。
4. 任一 focus/scope 变更都必须阻止旧 generation 的迟到响应污染新工作区。
5. 缺少授权、owner/provider、durable receipt、sandbox 或数据新鲜度证明时，界面显示 unavailable/denied/stale/conflict，不模拟成功。

## 2. 组件职责与边界

| 层 | 职责 | 禁止事项 |
|---|---|---|
| Rust Desktop Host | 启动与窗口生命周期；管理 scope generation；按 Project/Run/Worktree/Agent/Plugin 分配有界工作额度；取消与 drain；提供受控 IPC；记录本机资源指标。 | 不持有业务主数据副本；不把命令入队、子进程启动或 socket 写入当作业务操作持久成功。 |
| UI Renderer | 绘制当前导航、Run tabs、Task/Canvas/BI 等投影；发出有类型的用户意图；呈现 unknown、stale、denied、pending 与 receipt。 | 不自行拼接宿主 shell 命令；不缓存 token/secret；不绕过 Rust policy/Hook gate；不推断 owner 状态。 |
| Owner API / Provider | 对其实体做实时授权、版本检查、状态迁移、持久化、幂等、审计及 Outbox/receipt。 | 不接受 UI 本地状态作为授权或执行成功证明。 |
| Plugin Host | 按版本化 capability 清单执行经授权的插件工作，支持撤销、停止新调用和 drain。 | 主进程不加载任意 native plugin code；插件不能获得任意宿主路径或 shell。 |
| Native Hook Evaluator | 以 Rust 执行强约束 Hook；统一读取版本化 HookSet；在拒绝、超时、配置缺失或身份不明时 fail closed。 | 插件或 UI 不得替代、减弱或伪造安全决策。 |

业务事件、Outbox/Inbox、幂等身份、租约、恢复与 BI 契约保持 transport-neutral；桌面 IPC 只是受限命令/查询边界，不替代 owner 之间的持久消息语义。

## 3. IPC 与状态模型

每个 IPC 请求/事件携带明确的契约字段：

- request_id 与操作幂等身份（仅对 owner 支持的可变操作使用）。
- project_id、run_id、可选的 worktree_id、实体类型与实体 ID。
- 当前授权/RunContext 版本、策略或 HookSet 版本、scope generation。
- capability、截止时间、取消 token、分页游标或流 cursor。
- 响应中的 owner、实体版本、observed_at、freshness、operation receipt 和可恢复错误分类。

Renderer 只接收被允许字段的投影；秘密与长期凭证保留在 Rust/宿主授权边界中。IPC 输入做 schema、长度、范围、scope 和 capability 校验；不能把 renderer 提供的 Project/Run ID 当作权限本身。

状态分为四类：

1. **事实投影**：来自 owner API，包含来源、版本和观测时间。
2. **本地交互状态**：展开、选择、滚动位置、未发送草稿，不能写回业务 owner。
3. **操作状态**：preview → submitted → accepted/pending → owner receipt → reconciled；缺少最终 receipt 时不得显示完成。
4. **运行时健康**：provider、连接、权限、队列和 freshness。UI 只消费观测结果，不据此推断业务事实。

## 4. 低内存并行与取消

采用按需加载、分页、虚拟化和分层缓存。Project/Run 首屏仅载入摘要；Worktree/Task 列表分页；Canvas 按 viewport 请求；BI 延迟加载；终端/对话输出采用按字节限制的环形缓冲，不保留无限历史。缓存键包含授权 scope、实体版本与 generation，切换 scope 后旧缓存不可被新 scope 复用。

每个后台任务登记其 owner scope、优先级、工作类别、估算/实际字节、取消句柄和 drain 完成状态。队列同时限制条数与字节；按 Project/Run 做公平份额；当前键盘输入、焦点验证和用户操作高于 BI 聚合、缩略图和后台刷新。进度事件可按实体合并，但 owner 事实、授权拒绝、receipt 和审计关联不得静默丢弃。饱和时施加背压并显式报告暂停、拒绝或需重新同步。

切换 Project/Run/Worktree/Chat scope 时：

1. 增加 generation 并立即使旧回调失效。
2. 取消旧 scope 的 API、文件/Git 观察、订阅、渲染任务和插件调用。
3. 清除不再授权的视图模型和本地敏感缓存。
4. 等待 drain 或记录仍未确认停止的外部进程状态。
5. 重新取 RunContext、权限、binding/version 和 freshness。
6. 仅在校验成功后建立新 focus 并按需挂载 Run App。

取消 UI 请求不等于 owner 操作、CLI 子进程、虚拟机或远端 Agent 已停止；这些需要各自 owner 的可观察终态和 durable receipt。

## 5. 性能决策与基准协议

需分别测量三个实现轨道，禁止合并为一个成绩：

1. 现有 Tauri + React/Vite 壳的参考基线。
2. 共享 Web 组件的参考基线。
3. Rust 原生 renderer 的等语义原型。

共同 fixture 通过固定 manifest 和 hash 版本化，至少覆盖：

- Project/Run/Worktree 树：展开、筛选、翻页、键盘导航、unknown/stale/dirty/lock 等状态。
- Task/CLI：多 pane、UTF-8/ANSI 长输出、突发与持续流、重连、cancel/drain。
- Canvas：viewport 内外元素、连线、缩放与拖动。
- 多 Agent：最大配置并发下同时操作目录、CLI、Canvas 和 BI 后台任务。
- 生命周期和插件：权限撤销、focus 往返、操作 pending/receipt、plugin revoke/drain。

每次报告记录设备型号、CPU/RAM、OS/build、显示器/DPI、commit、renderer/runtime/toolchain、debug/release、fixture hash、冷/热状态和重复次数。指标至少包括：

- 应用进程树稳定/峰值 RSS；注明是否包含 renderer/WebView 子进程且避免重复计数。
- Rust allocator、缓存、队列、终端和 Canvas 的分项字节/高水位。
- 冷启动到 Project 可交互、输入反馈、树滚动/展开、focus 到可见、终端 render、Canvas pan/zoom、BI 后台与 UI 竞争的 p50/p95。
- 队列最大条数/字节/年龄、拒绝/合并/重同步次数、取消到 drain 时间和取消后的遗留句柄。
- 键盘、读屏、输入法、窗口缩放/多显示器的结果及阻断项。

目标设备与负载分布确认后，由产品验收者设定可观察阈值；本 ADR 不编造跨设备硬阈值。选型需说明工作负载结果、无障碍覆盖、跨平台风险、内存归因和维护成本。

## 6. 插件、Hook 与安全门

插件 UI 使用 capability-driven descriptor：名称、版本、可用性、所需权限、Run scope、数据来源、撤销状态与 owner health。安装/启用/撤权不得绕开 capability 授权；撤权应阻止新调用并等待现有调用 drain，展示明确结果。插件运行时实现需单独选择和安全审查，不在本 ADR 锁定 WASM、子进程或其他执行后端。

安全 Hook 由 Rust Host 原生评估，支持高级设置内可视化的 typed 编辑器和发布/回滚流程。编辑器负责构造可验证策略与展示评估结果；最终授权、命令放行、生命周期与执行前门仍由原生 evaluator 决定。HookSet 的 Project/Worktree 继承、Run 快照、版本和 BI 决策事件必须一致。

任务卡内 CLI 的界面只有在 Task/Run/Worktree/Session 绑定、当前权限、预算预留、sandbox、Hook decision、控制/取消、独立验证、审计输出和结果回写全部成立后才可启用；否则显示具体缺少的 owner/provider 门禁。

## 7. 尚未决策与关闭条件

| 决策项 | 当前状态 | 关闭条件 |
|---|---|---|
| Rust renderer / drawing backend | 未定 | 同目标设备、同 fixture、同语义基准 + 键盘/读屏/IME 检查；记录候选与结果后单独 ADR 决定。 |
| IPC 序列化和调用框架 | 边界已定、具体 crate 未定 | 用类型规模、跨版本兼容、校验成本、内存拷贝、取消和 tracing 原型比较。 |
| Plugin runtime | capability 与撤销边界已定、执行后端未定 | 威胁模型、隔离验证、资源配额、崩溃恢复和撤销/drain 测试通过。 |
| 资源绝对上限 | 未定 | 采集目标设备和真实负载基线后评审并版本化，不以猜测值固化。 |
| Accessibility parity | 要求已定、设备实测未完成 | 完成键盘、屏幕阅读、输入法、缩放和多窗口测试并记录可接受缺口。 |

本 ADR 的完成不表示 UI renderer 已选择、Rust 原生 UI 已交付、桌面性能已达标或 CLI/插件/Hook Runtime 已生产可用。

## 8. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Rust 桌面职责、类型化 IPC scope/version/cancellation 边界、有界并行策略、安全 Hook/Plugin 边界及可复测性能协议；renderer/runtime/阈值保持待测，不宣称实现或性能验收完成。 | 用户要求当前轮结束后规划新版 UI，并要求优先降低 Rust 桌面内存消耗、保障并行与扩展能力。 |
