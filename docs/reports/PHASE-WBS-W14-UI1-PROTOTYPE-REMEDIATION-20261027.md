# PHASE-WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027

## 0. 目的与范围

按已合入 dev 的独立复核报告 `docs/reviews/WBS-W14-UI1-INDEPENDENT-REVIEW-20261027.md` 修复 UI-1 静态导航原型的 I-1–I-4，并将实际状态同步到 WBS。范围仅限 dispatcher brief 指定的四个文件；不改产品代码、API、数据库或其他设计/复核文档。本报告记录静态原型修正，不代表 UI-1 运行验收或 W14 关闭。

## 1. 改动矩阵

| 文件 | 改动 | 结果 |
|---|---|---|
| `docs/prototypes/w14-ui1-flow-20261026/index.html` | 移除冷启动的默认 Worktree 焦点和授权；将 Project Index 列表授权与单 Worktree 焦点授权拆成独立状态；授权检查/拒绝/撤权时分别控制列表行与焦点内容；保存 Index 复验前的有效 Worktree 页面，取消/拒绝/撤权时恢复；检查中提供返回焦点并取消本地模拟检查的入口；避免变体 B 重复绘制 Index；明确 Task Card 的 Run 归属与 CLI 执行上下文；CLI 保持禁用。 | 完成静态原型修正。授权操作只演示同步本地状态，不连接身份服务。 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | 更新 §16/§17 与 v0.63 修订历史，链接独立复核和修正报告，记录 I-1–I-4、独立授权语义、Index 页面返回路径、左树受保护行处理及未运行门禁。 | W14 保持 OPEN；未把静态修正写成生产验收。 |
| `docs/briefs/WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027.md` | 补充 Index 列表复验不撤销仍有效 Worktree 焦点及其聊天 scope，并明确保存/恢复视图和返回路径。 | Brief 与实际原型/授权边界一致。 |
| `docs/reports/PHASE-WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027.md` | 本报告。 | 记录范围、结果、验证与限制。 |

## 2. 验证摘要

- 对 HTML 的状态转换与 A/B/C 静态树作了定向源码检查：Index 行只在当前 Index 授权 generation 有效时出现；Index 授权 checking/denied/revoked 时收起受保护的 Worktree/Branch 列表行；单 Worktree 焦点失效时清除当前 Branch、Run Apps、任务详情和 WORKTREE scope。若 Index 列表授权有效，焦点 checking/denied/revoked 时保留获准的 Index 行；若焦点授权仍有效，Index 复验/拒绝/撤权不会连带撤销该焦点或改变其 chat scope。Index 复验开始时记录焦点页；取消、拒绝、撤权路径恢复仍有效的 Worktree 页面；Index 检查中可以显式返回当前焦点并取消本地模拟请求，若焦点已失效则回到无焦点 workspace。
- 检查变体 B：Index 表只由固定 Index 面板渲染一次；右侧只呈现焦点/选择状态。
- 检查 Task Card 与 Canvas：Task Card 归属 Run，Worktree/本地 Branch 只表示 CLI 执行上下文；CLI 与发送操作保持禁用，Canvas 与 Task Cards 同级。
- 检查底栏草稿按 GLOBAL 与 Worktree ID 分区；演示数据/非生产说明及无任意 Shell 的边界仍保留。
- WBS §16/§17 与 brief 已对照原型同步。Cypher 头更新为规范化的 `File`/`Symbol` 关系并刷新源码摘要；当前 Guardian 的源码扩展名清单不含 `.html`，因此其零源文件结果不能视作 HTML 关系覆盖证明。
- 未运行测试、构建、浏览器、产品运行时、辅助技术、API/数据库或性能测量。静态检查不证明 DOM/JavaScript 实际运行正确。

## 3. 已知缺口

1. 页面仍是内存中的静态演示；Index 授权、Worktree 授权、撤权和 generation fence 不会调用真实授权 owner，也没有真实异步请求/迟到响应，不能证明生产竞态安全。
2. 尚未验证浏览器视觉、窄屏布局、键盘操作、读屏器、焦点顺序、颜色对比或可访问性。
3. Project、Run、Worktree、Branch、Task Card、Canvas、Run Apps 和草稿均为本地样例；没有 owner API、数据库、持久化、事件推送或真实数据。
4. 受控 Agent Runtime、隔离 sandbox、原生 Hook、预算、取消/恢复、独立验证及结果回写未接入；CLI 与发送仍禁用。
5. 未进行 RSS、CPU、帧时间、队列、缓存、并行负载或其他性能测量。W14 的目标设备、workload 与测量门继续开放。

## 4. 子代理失败接手清单

- 本 lane 未派发其他子代理；没有子代理失败或待接手产物。
- 交由 root/Mavis 按已登记文件清单审阅本 lane 的四文件差异。该审阅/集成由父任务处理；本 lane 不提交、不 rebase、不合并。

## 5. 守门规则

1. 只修改 dispatcher brief 指定的 HTML、WBS、brief 和本报告四个文件；不改独立复核报告。
2. 不触碰 `D:\Star` 根工作树、dev checkout 或其他 Worktree/branch 的文件、用户数据和状态。
3. 不修改产品实现、API、数据库、migration、runtime、owner contract 或其他设计文档。
4. Project Index 列表授权与单个 Worktree 焦点授权是两项独立授权，不得从一项推导另一项。
5. Index 授权为 idle/checking/denied/revoked 时，A/B/C 左树及 Index 面板隐藏所有受保护的 Worktree/Branch 行、对象名称、Branch 值和计数。
6. Index 授权通过只允许显示 Project 范围列表；它本身不授予选择或打开任一 Worktree 的焦点。
7. 从列表选择 Worktree 或显式深链都必须先进入新的焦点 checking；旧焦点在验证期间立即清除。
8. Worktree 焦点 denied/revoked/checking 时，清除旧 Worktree、Branch、Run Apps、任务详情和 WORKTREE 聊天 scope；回退到 GLOBAL。
9. Index 复验、拒绝或撤权只控制列表可见性，不撤销仍有效的 Worktree 焦点或改写其 chat scope；取消/拒绝/撤权恢复仍有效的焦点页面，检查期间只允许有效焦点通过显式入口返回并取消本地模拟检查。
10. 左树可保留不含对象标识的 Project/Run 导航容器；Branch 叶节点只在其 Worktree 焦点有效时出现。
11. 保持 Project → Engineering Run → Worktree → Branch；Worktree 下的 Branch 表示本地 checkout ref/state，不冒充远端 Branch。
12. Run Apps 只在当前 Worktree 焦点授权有效时打开；收件箱、Task Cards、Canvas、Workflow、BI/Benchmark 和 Plugin Apps 保持 Run 同级。
13. Canvas 节点仅引用 canonical Task Card；Task Card 归属 Run，当前 Worktree/本地 Branch 只作为 CLI 执行上下文。
14. CLI、聊天发送与未连接 App 保持禁用/未接入；不添加任意 Shell，也不把 UI/API 契约或模拟按钮标成可执行能力。
15. 底部聊天明确区分 GLOBAL/WORKTREE；WORKTREE 仅在焦点授权有效时启用，草稿按 GLOBAL 与 Worktree ID 隔离。
16. 所有资源/任务/状态均须有显式 demo/non-production 标记；本地样例不得被描述为真实 Project、授权、任务或性能事实。
17. generation 比较仅是同步原型状态的表现，不构成生产异步回调安全证明；浏览器、键盘/读屏/窄屏、API/数据库、生产执行和性能测量均未运行，W14 保持 OPEN。

## 6. 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`：静态原型与既定导航/授权边界对齐；不表示生产架构验收。 |
| SRE Lead | `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`：可靠性、运行态撤权与恢复未验证，维持 OPEN。 |
| 平台 | `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`：API、数据库、桌面 runtime、性能均未验证。 |
| 评审主持 | `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`：I-1–I-4 已逐项记录并映射到原型与 WBS。 |
| PM | `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`：仅接受静态修正结果；UI-1 运行门及 W14 不关闭。 |

以上为本 agent 按项目代审规则记录的文档审查结论，不代表真实人员或生产环境批准。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 UI-1 原型 I-1–I-4；进一步保持 Index 列表和 Worktree 焦点权限独立；同步 WBS/brief，记录静态验证边界与未运行门禁。 | 已合入 dev 的独立静态复核报告及 dispatcher brief。 |
| v0.2 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Index 复验前的有效焦点页面记录；取消/拒绝/撤权自动恢复焦点；检查中增加返回当前 Worktree 并取消本地模拟请求的显式入口；同步 WBS/brief，继续声明真实异步授权安全未验证。 | 独立复审发现 Index 状态转换可能暂时困住仍获授权的 Worktree 页面。 |
