# W14 UI-1 独立静态复核（2026-10-27）

## 0. 目的与范围

独立复核 `docs/prototypes/w14-ui1-flow-20261026/index.html` 的静态状态与界面文案，并对照既有复核 `docs/reviews/WBS-W14-UI1-FLOW-REVIEW-20261026.md`、WBS §16–19 和 UI-0 导航设计 `docs/reviews/WBS-W14-NAVIGATION-IA-20261026.md`。检查范围包括 Project → Engineering Run → Worktree → Branch、Worktree 焦点与授权状态、Run Apps、Task Card CLI、Canvas/Task Card 层级、GLOBAL/WORKTREE 草稿、演示数据标识及状态泄漏。

本报告只复核静态源码和文档，没有运行浏览器、产品、测试、构建或性能采样；不把原型表现当作生产能力或授权证明。

## 1. 结果摘要

| 检查项 | 结果 | 依据 |
|---|---|---|
| Project → Engineering Run → Worktree → Branch 顺序 | 通过（静态结构） | 原型面包屑 `index.html:306-307`，侧栏树 `:351-375`，聚焦布局步骤 `:449`；Branch 明确标成本地值。 |
| 仅在有效 Worktree 焦点后显示 Run Apps | 部分通过 | `render()` 使用 `focusReady` 控制 tabs（`:347-350`, `:388-396`），但冷启动默认直接设为授权通过（发现 I-1）；变体 B 的 Project Index 也绕开 Worktree 状态面板（发现 I-2）。 |
| Task Card CLI 未接入时禁用 | 通过（原型态） | `:419` 明示受控 CLI 尚未接入且按钮 `disabled`；同处列出执行前置条件。 |
| Canvas 与 Task Cards 是 Run Apps 同级项 | 部分通过 | 两者并列为 tabs，Canvas 可打开 canonical Task Card（`:388-396`, `:431-436`）；Task Card 文案又将任务称为 Worktree-bound（发现 I-4），混淆 Run 级任务归属与 Worktree 执行焦点。 |
| GLOBAL / WORKTREE 草稿隔离 | 通过（仅内存原型） | 状态按 GLOBAL 与 Worktree ID 分区（`:331-333`），重绘时恢复相应草稿（`:468-469`），输入时按当前 scope 写回（`:540-544`）。没有持久化或会话恢复。 |
| checking / denied / revoked 旧内容清理 | 部分通过 | 主工作区在非 ready 状态用状态面板替换 Apps（`:409-412`）；变体 B 的索引仍显示所有样例 Worktree/Branch（发现 I-2）。 |
| mock 与非生产边界 | 通过（页面级说明） | 页面顶部明确为本地交互原型、样例目录、不连接服务或执行命令（`:293-298`）；未连接 App 另有说明（`:437-439`）。 |

## 2. 发现

### I-1 — 冷启动默认绕过 Worktree 授权状态门

**严重度：Warning（阻断 UI-1 授权流程验收）**

- **证据：** `docs/prototypes/w14-ui1-flow-20261026/index.html:323-334` 将初始状态设为 `focus: true`、`authorization: "ready"`；`:347-350` 据此立即将 `showApps` 设为真；`:549` 在页面载入时直接调用 `render()`。因此首次打开页面无需选择 Worktree 或触发模拟授权，就会显示已验证焦点和 Run Apps。
- **设计依据：** `docs/reviews/WBS-W14-NAVIGATION-IA-20261026.md:56,64-65` 要求直接路由、历史恢复和目标切换统一经过授权；授权上下文、焦点与 generation 匹配前不得挂载 App。
- **既有复核：** 未捕获。既有报告 `docs/reviews/WBS-W14-UI1-FLOW-REVIEW-20261026.md:12,22-28` 覆盖点击切换、深链、拒绝与撤权，但没有指出首屏默认处于 ready。
- **最小修正：** 初始状态设为无有效 Worktree 焦点且未授权；先显示 Worktree 选择/状态页，只有显式模拟授权通过后才进入 ready。若保留“已验证示例”快捷状态，必须将其变成明确的用户操作，而不能作为默认首屏。

### I-2 — 变体 B 在授权检查、拒绝或撤权时仍保留样例 Worktree 与 Branch 行

**严重度：Warning（授权边界表现不完整；仅涉及静态样例）**

- **证据：** `indexMarkup` 在 `docs/prototypes/w14-ui1-flow-20261026/index.html:397-405` 无条件包含两个 Worktree、Branch 和“可用”状态；变体 B 在 `:442-445` 总是把 `indexMarkup` 放入工作区。与此同时 `:409-414` 只根据授权状态替换右侧 `appMarkup`。所以进入 checking、模拟 denied 或 revoked 时，B 仍显示索引中的所有 Worktree/Branch 行，而 A/C 的主工作区不显示该焦点的 Branch。
- **设计依据：** UI-0 `docs/reviews/WBS-W14-NAVIGATION-IA-20261026.md:31,65,67` 允许 Project 级 Index，但要求其数据按各自授权范围提供；越权对象不得返回名称、计数或内容。WBS §17 `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:490` 也要求真实 Index 按 actor 权限过滤，并要求授权失败时隐藏旧焦点内容。
- **既有复核：** 部分记录了目标规则但漏掉当前实现。既有复核 `docs/reviews/WBS-W14-UI1-FLOW-REVIEW-20261026.md:13,45` 说明 Index 应按当前用户授权过滤，也认为拒绝/撤权隐藏旧 Branch；未指出变体 B 的固定样例行不受该状态门影响。
- **最小修正：** 将 Project Index 的列表权限与单个 Worktree 焦点授权分别显式建模，并只渲染索引授权允许的行；如果没有可确认的列表授权，则隐藏受保护行并显示安全空状态。若设计决定拒绝单个 Worktree 后仍可列出该 Worktree 的名称/Branch，应在原型中标出该独立授权语义，避免被理解为旧焦点泄漏。

### I-3 — 变体 B 进入 Project Worktree Index 时重复绘制同一索引

**严重度：Info（原型布局缺陷）**

- **证据：** `indexMarkup` 定义在 `docs/prototypes/w14-ui1-flow-20261026/index.html:397-407`；变体 B 的固定左侧列在 `:442-445` 始终显示一次。点击 `show-index` 会令 `state.view` 变成 `index`（`:511-514`），之后 `:413-415` 又将同一份 `indexMarkup` 放进右侧焦点区，形成重复表格。
- **既有复核：** 未捕获。既有复核 `docs/reviews/WBS-W14-UI1-FLOW-REVIEW-20261026.md:13` 记录了 Index 作为 Project 入口，没有比较变体 B 点击后的双列输出。
- **最小修正：** 对变体 B 的索引视图只保留一个索引实例；右侧显示空选择提示或所选 Worktree 的授权状态，而不要重复表格。

### I-4 — Task Card 文案混淆 Run 级任务与 Worktree 执行上下文

**严重度：Warning（领域层级表达不清）**

- **证据：** 任务详情标题写作 `Task Card · Worktree bound`，并称“此任务绑定当前 Run、Worktree 与本地 Branch”（`docs/prototypes/w14-ui1-flow-20261026/index.html:418-419`）；列表又标为 `Run App · Worktree scope`（`:424`）。这容易让用户理解为任务事实归 Worktree，而非归 Run、并由当前 Worktree 提供 CLI checkout 执行焦点。
- **设计依据：** UI-0 `docs/reviews/WBS-W14-NAVIGATION-IA-20261026.md:28,32,37-38` 将任务纳入 Run 的工作/Agent/任务边界，规定 Task Cards 与 Canvas 是同级 Run Apps；CLI 请求携带 Task、Run 与当前 Worktree，并把 checkout 固定到 Worktree。
- **既有复核：** 部分捕获。既有复核 `docs/reviews/WBS-W14-UI1-FLOW-REVIEW-20261026.md:14-15` 确认 tabs 同级且 CLI 禁用，但未审查任务归属与执行上下文是否被文案混为一谈。
- **最小修正：** 将任务详情标为 Run 级 Task Card；另设“CLI 执行上下文”区域，说明启用后使用当前已授权 Worktree/Branch。保持 CLI 禁用态和当前绑定约束说明。

## 3. 限制与已知缺口

- 此复核是源码与设计文档静态比对，不验证浏览器中的布局、交互时序、键盘/读屏支持、权限 API、取消旧请求或迟到响应处理。
- 原型不含真实授权、Worktree owner、Project Index 过滤、任务数据 API、CLI Runtime、Canvas owner 或持久化；I-1 至 I-4 是原型模型/文案缺口，不证明生产产品存在相同缺陷。
- 页面级 demo 声明足够清楚；索引行上的“可用”仍可能在局部截图中被误读，后续如会裁剪原型横幅，建议把“样例”同时标在行状态旁。
- CodeGraph 本轮没有返回 HTML 文件，且当前会话没有 coverage 工具；因此直接检查了 HTML 源码，未声称图覆盖完整，也未作全仓穷尽结论。
- 没有运行测试、构建、浏览器/运行时或性能检查，不能据此验收 UI-1 运行态，也不能关闭 W14。

## 4. 结论与后续门

原型已经表达 Project → Run → Worktree → Branch、并列 Run Apps、禁用 CLI、双 scope 草稿和非生产边界，但冷启动授权默认值、变体 B 的 Index 授权状态表现，以及 Task Card 的 Worktree 归属文案需要修正后再将授权/层级流程视作静态验收通过。变体 B 的索引重复也应一并修复。该结论不改变 WBS 中 UI-1 仅完成原型、W14 保持 OPEN 的状态。

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-27 | W14 UI-1 independent review lane | 独立静态复核 UI-1 原型，记录四项未被既有复核发现或未充分覆盖的问题，并明确验证边界。 | 补齐 WBS §17 要求的独立复核 |
