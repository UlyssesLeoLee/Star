# WBS UI-2：Run Worktree 选择器分页与搜索

## §0 目的

从 dev/main/缓存 origin/dev 共同基线 5a051bfd8f76fbfbe8ff49a994579c60166d6ec6 改善 Run 未聚焦状态的 Worktree 选择体验。导航固定为 Project → Engineering Run → Worktree → Branch；选择器只展示当前 Run 已授权的 Worktree，点击后由规范路由重新取得 RunContext 并验证焦点。

本阶段只实现 Web Run Shell 的有界分页与已加载条目搜索，不改变 API、授权、持久化或生产能力门。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| frontend/src/components/run/RunWorkspaceShell.tsx | 增加按 next_cursor 加载更多、名称/Branch/状态/ID 本地搜索、重复身份/游标与页大小校验、错误刷新、加载状态与明确结果计数；首屏和后续页最多 50 条，总共最多保留 4 页/200 条。搜索范围只覆盖已加载记录。无焦点时不挂载 Task Cards、BI 或 Run App tabs。 | 已实现；focused Vitest 6/6 与 TypeScript typecheck 通过；production build 尝试后 Node OOM/exit 134；浏览器验收未运行 |
| frontend/src/components/run/RunWorkspaceShell.test.tsx | 覆盖 no-focus gate、focus App gate、已加载条目搜索、游标分页与规范 Worktree 链接，以及页内重复 ID、首屏/后续页超量响应。 | focused Vitest 6/6 通过 |
| frontend/src/components/run/ProjectRunTree.tsx | Worktree 元数据未知时添加可见提示标记，并在 hover/accessibility 描述中保留未知字段说明。 | 已实现 |
| docs/implementation-plans/WBS-UI-PLAN-NEXT-20261022.md | 基线更新到 5a051bfd，更新 UI-2 状态与 UI-0 下一步，保留 owner/运行门禁。 | 完成 |
| docs/implementation-plans/UI-PLAN-RUN-WORKTREE-NEXT-20261021.md | 更新 UI-2 当前交互、搜索范围、200 条上限及剩余缺口。 | 完成 |
| docs/implementation-plans/UI0-SCREEN-STATE-20261022.md | 同步实现边界与尚未关闭的全目录搜索、辅助技术/视觉验收。 | 完成 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 基线 | 本地 main/dev 和缓存 origin/dev 在 5a051bfd；本轮未刷新远端 |
| CypherGraph Guardian | PASS：cgg_guard.py --changed --stamp（3 个源码文件） |
| focused Vitest | PASS：RunWorkspaceShell.test.tsx，6/6；覆盖页内重复 ID、首屏与后续页超量响应 |
| TypeScript | PASS：pnpm run typecheck |
| git diff --check | PASS；仅有文档行尾提示，没有 whitespace error |
| Production build | 未通过：Next.js build 以 Node OOM/exit 134 退出；同一时点系统可用内存约 0.81 GB，不视为代码构建通过 |
| Browser / e2e | 未运行 |
| CodeRabbit | 已尝试 review 未提交差异；CLI 在 Windows WSL 启动时返回 Wsl/Service/E_UNEXPECTED，无审查结果；不视为干净审查 |

## §3 已知缺口

1. 搜索仅覆盖已加载的最多 200 条记录；超出后用户需在左侧 Project 目录继续分页。跨全目录搜索需要 owner/API 查询合同。
2. 本轮没有做浏览器视觉、键盘、屏幕阅读器或窄视口验收。
3. 右侧完整 Run App、聊天 Runtime/send、Run Worktree 生产生命周期、Task CLI 执行及 Rust 原生桌面端仍按 WBS 保持未完成/关闭门状态。
4. 不声明真实 owner-backed 行为、production build 成功或运行时成功；本次 build 失败需在足够可用内存的环境中重试。

## §4 子代理失败接手清单

本 UI-2 切片没有派发子代理；没有子代理产出需要接手。本轮后续 UI-0 benchmark harness 将作为独立 lane 派发。

## §5 守门规则

- 只有 Worktree 授权焦点通过规范 RunContext 校验后才显示 Run Apps。
- 选择器仅请求当前 Run 的 Worktree 目录；不通过 Branch 名、路径或本地缓存推断归属。
- 每页沿用 Run Directory 客户端的 50 条上限；最多保留 4 页/200 条。
- 搜索明确显示已加载记录范围，不宣称服务器全目录搜索。
- 选择条目只导航到规范 Worktree 路由；目标路由仍会重新授权和校验。
- 取消旧 scope 目录请求；检测重复 Worktree 和重复游标；失败时提供刷新目录入口。
- 保持 GLOBAL/WORKTREE 聊天范围约束和现有 Run App fail-closed 行为。
- 不启用尚未通过 owner/provider/sandbox/receipt 验收的生命周期、CLI、Canvas、BI 或 Plugin 功能。

## §6 审核状态

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手完成本地代码/文档差异复核；focused 测试和类型检查通过 |
| SRE Lead | 未涉及基础设施执行；不声明通过 |
| 平台 | production build 因 Node OOM 未通过；浏览器验证未运行 |
| 评审主持 | 本地自检完成；CodeRabbit 因 WSL 启动错误未产出审查结果 |
| PM | 已按 UI-2 WBS 更新范围与残余项 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.4 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 6/6 focused tests、typecheck 与 CypherGraph Guardian 通过；production build 实际尝试但因 Node OOM/exit 134 失败，浏览器与无障碍验收仍开放。 | 完成 UI-2 分页边界自审与构建尝试 |
| v0.3 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 自审补齐首屏/后续页统一预算验证、页内重复 ID 检测和超量响应拒绝；focused Vitest 6/6、typecheck 通过，生产 build 与浏览器/无障碍验收仍开放。 | 按 WBS 继续实施并修复分页边界 |
| v0.2 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加分页搜索回归测试，focused Vitest 3/3、TypeScript typecheck、CypherGraph 与差异检查通过；记录浏览器/e2e/无障碍仍开放。 | 修复依赖链接后继续 WBS 验证 |
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 实现 Run 未聚焦时的有界 Worktree 分页与已加载项搜索，并同步 UI 规划；最初验证尚未运行。 | 用户要求以最新 dev 优化 UI/UX 后继续 WBS |
