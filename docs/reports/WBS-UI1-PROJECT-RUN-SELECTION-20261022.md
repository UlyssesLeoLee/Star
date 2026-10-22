# WBS UI-1：Project / Run 选择与展开分离

## §0 目的

在主导航 Project → Engineering Run → Worktree → Branch 中，使 Project/Run 的选择语义与目录展开语义独立。Project 选择进入 Project Worktree Index；Run 选择只进入无 Worktree focus 的 Run 概览；只有经目标路由复验的 Worktree 才能开启 Run Apps。本报告记录 Web 树的一个可审查切片，不关闭完整 UI-1。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| frontend/src/components/run/ProjectRunTree.tsx | Project 与 Run 使用独立 disclosure button 和导航 treeitem；Project 链接携带 project_id 并更新 Project scope，Run 链接打开 canonical Run overview 并清理旧 Worktree focus；移除会误导为直接打开任务面的快捷图标。 | 已实现；最终浏览器交互仍待验收 |
| frontend/src/components/run/ProjectRunTree.test.tsx | 覆盖 Project 导航/Project scope/focus 清理、Project/Run disclosure 不触发选择和 Run no-focus 路径，以及 ArrowRight 键盘展开不触发选择。 | 3/3 通过 |
| docs/implementation-plans/WBS-UI-PLAN-NEXT-20261022.md | 更新 UI-1 状态、下一步和 WBS 修订记录。 | 完成 |
| docs/implementation-plans/UI-PLAN-RUN-WORKTREE-NEXT-20261021.md | 同步实现状态、证据边界和未关闭的浏览器/无障碍验收。 | 完成 |
| docs/implementation-plans/UI0-SCREEN-STATE-20261022.md | 同步实际局部行为与验证结果，不将其写成完整 UI-1。 | 完成 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| UI-1/UI-2 focused Vitest | PASS：两个文件，9/9 tests；ProjectRunTree 为 3/3，RunWorkspaceShell 为 6/6 |
| TypeScript | PASS：`pnpm run typecheck` |
| CypherGraph Guardian | PASS：changed source files stamped and validated |
| `git diff --check` | PASS；仅有 CRLF/LF 行尾提示，无 whitespace error |
| 浏览器 / 键盘 / 屏幕阅读器 | 未运行；不视为通过 |
| Production build | 之前尝试因 Node OOM/exit 134 失败；本切片不重复声称构建通过 |
| CodeRabbit | 未取得独立审查结果：Windows shim 调用失败，WSL 内 `/root/.local/bin/coderabbit` 返回 `Permission denied`，版本与认证状态无法读取；本地差异自审已完成 |

## §3 已知缺口

1. 尚未在真实浏览器验证 Next navigation、Back/Forward、focus 恢复和导航守卫拒绝时的状态。
2. 组件用例验证 tree row 语义与入口，但未覆盖辅助技术、完整 roving focus、窄视口、读屏顺序和颜色外的视觉反馈。
3. Run overview 的 Project/Run 权限、Worktree binding/provider、lock、dirty/ahead/behind、observed-at、Branch freshness 和 receipt 仍依赖 owner/API；缺失事实保持 unknown/unavailable。
4. UI-1 其他目标——跨 Run Index 的可信归属、操作预览/回执、键盘/分页验收——仍开放。完整 Run Apps、CLI、Canvas、BI、插件和基础设施不因本切片启用。

## §4 子代理失败接手清单

本切片未派发子代理，没有子代理产出需要接手。并行 UI-0 benchmark harness 是独立 lane，尚不是 UI-1 验收证据。

## §5 守门规则

- Project 选择和 Run 选择不隐式建立 Worktree focus；旧 focus 在切换 Project/Run 时清除。
- Run overview 不等于 checkout focus，不得因此挂载 Task/Canvas/BI/Plugin App 内容。
- Worktree 目标仍由规范路由复验授权、Project/Run/Worktree 身份及 binding/version。
- Project/Run 展开只请求其目录下级；不能把展示投影当作锁、Git 差异或执行状态事实。
- 缺失/陈旧 owner 事实必须显示 unknown/unavailable；不以本地缓存或 Branch 名推断。
- 只对本切片实际通过的 focused tests、typecheck 和 Cypher 关系校验作通过陈述。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核：切片与 Project → Run → Worktree → Branch 层级及 Worktree focus 门一致 |
| SRE Lead | 不涉及基础设施执行；未作可用性结论 |
| 平台 | focused tests、typecheck、CypherGraph Guardian 通过；浏览器/读屏与 production build 未通过/未验收 |
| 评审主持 | 本地差异复核完成；外部 CodeRabbit 未能启动，未作为通过证据 |
| PM | WBS 已更新；完整 UI-1 仍开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Project/Run 选择与展开分离的 Web 实现、聚焦测试和残余验收边界；不关闭完整 UI-1。 | 按 WBS 继续 UI-1 |
