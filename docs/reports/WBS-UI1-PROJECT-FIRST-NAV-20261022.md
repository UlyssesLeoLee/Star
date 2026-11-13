# WBS-UI1-PROJECT-FIRST-NAV-20261022

| 项目 | 内容 |
|---|---|
| 版本 / 状态 | v0.4 / UI-1 桌面与移动导航切片完成；阶段总体验收仍开放 |
| 日期 | 2026-10-22 |
| 分支 / 基线 | `codex/ui-plan-v0-7-20261022`，rebase onto UI-0 `4e8a8268`；fast-forwarded local dev at `4dcf567d1e89aa4c648fd06a297554166da0f173` |
| 范围 | Project-first 左侧导航、移动目录抽屉、Project Worktree Index fail-closed 呈现、相关测试及 WBS/详细设计同步 |

## §0 目的

完成 WBS UI-1 的主导航交付切片，并验证移动窄屏操作与焦点行为。验收只覆盖 UI 显示和导航边界；授权目录、Worktree owner facts、生命周期操作和生产闭环仍由对应 owner 与 WLD 阶段验收。

## §1 改动矩阵

| 路径 | 改动 | 状态 |
|---|---|---|
| `frontend/src/components/Sidebar.tsx` | 固定 Project → Engineering Run → Worktree → Branch 为主导航，移除 Main/Project scope 切换；未迁移页面保留在可折叠次级入口；移除静态成功状态。主导航标签使用当前语言。 | 完成 |
| `frontend/src/components/MobileBottomNav.tsx` | 以 Project 层级目录替代旧模块快捷栏；路由变化后收起；支持 Escape 关闭、触发按钮焦点恢复；可访问标签和提示支持英语、简体中文、日语。 | 完成 |
| `frontend/src/app/worktree/page.tsx` | 无授权 directory client 时只显示 unavailable/retry，不以本地 preview 或 seed 作为管理事实；移除旧状态机大图。 | 完成；owner 数据未装配 |
| `frontend/src/lib/i18n/dictionary.ts`、`en.ts`、`zh-CN.ts`、`ja.ts` | 增加 Project 主导航与移动目录抽屉的类型和三语文案。 | 完成 |
| Sidebar、Project Worktree Navigation、ProjectRunTree、RunWorkspaceShell、AppShell、Worktree selector/control 测试 | 覆盖主导航、次级页面入口、树展开/选择、fail-closed 路径、移动抽屉路由收起、语言文案、Escape 焦点恢复及相邻 Shell/Worktree 行为；ProjectRunTree 直接验证 Project 选择、独立展开和 ArrowRight。 | 8 个 focused suites，35 项通过 |
| `docs/implementation-plans/WBS-UI-PLAN-NEXT-20261022.md`、`UI-PLAN-RUN-WORKTREE-NEXT-20261021.md` | 更新 UI-1/UI-2 实测状态、WBS 下一工作、修订记录和未关闭门禁。 | 完成 |

## §2 验证摘要

| 验证 | 结果 |
|---|---|
| 定向 Vitest | 8 个文件、35/35 项通过。 |
| TypeScript | `pnpm run typecheck` 通过。 |
| 浏览器响应式检查 | `localhost:3111/worktree`，390×844 viewport；`documentElement.scrollWidth` 与 `body.scrollWidth` 均为 390，没有横向溢出。 |
| 移动键盘操作 | 打开 Project 目录后焦点到达“收起”；Escape 关闭抽屉后焦点回到 Project 目录触发按钮。 |
| 浏览器 AX | 显示 Project 导航层级、移动目录抽屉和“授权目录暂不可用”状态；未显示 mock Worktree。 |
| 差异检查 | `git diff --check` exit 0；Git 给出 MobileBottomNav LF→CRLF 提示，没有 whitespace error。 |
| CypherGraph Guardian | `cgg_guard.py --changed --stamp` PASS，12 个源文件已核对并刷新 freshness stamp。 |
| 独立 CodeRabbit 审阅 | 未通过：Windows shim 调用 WSL 时 `/root/.local/bin/coderabbit` 返回 `Permission denied`；此项不视为审阅通过。 |

## §3 已知缺口

1. 当前浏览器没有可信宿主授权会话，实际 Project/Run/Worktree 树数据、完整选择流和 focus transition 不能验收；页面正确保持 unavailable。
2. 未使用独立屏幕阅读器验证语音顺序与状态播报；浏览器 AX 树不是屏幕阅读器验收的替代品。
3. Worktree owner 尚未提供全部 Agent/Session、Branch freshness、dirty/ahead/behind、lock/lease 等事实；UI 必须继续显示 unknown/unavailable。
4. production build 曾在可用内存约 0.81 GB 时因 Node OOM/exit 134 失败；本轮未重跑，不能计为通过。
5. UI-0 harness 已经父级复核并集成到本地 dev（`4e8a8268`），其标准库测试 16/16 通过；没有任何真实 renderer driver 样本，阈值和渲染器选择继续开放。CodeRabbit 因 WSL 权限错误未能提供独立审阅。

## §4 子代理失败接手清单

本轮未创建子代理；无子代理失败或待接手项。UI-0 harness 来自既有独立 lane，父级已重跑其 16 项测试、完成源文件审阅并将独立提交纳入本地 dev；该工作仍缺真实 UI driver 样本。

## §5 守门规则

1. 主导航顺序固定为 Project → Engineering Run → Worktree → Branch；远端宿主身份不改变层级。
2. 选择 Worktree 且重新验证授权/binding 前，不挂载该 Run 的 App tabs。
3. 没有可信 directory client 时 fail closed；缓存、seed 和 preview 不能显示成生产 Worktree。
4. 未迁移页面保留可达入口，但不伪装成已挂接 Run 的应用。
5. 移动目录按当前语言显示可访问名称；Escape 关闭后恢复焦点。
6. 本报告通过测试不代表真实屏幕阅读器、授权目录、生命周期操作或任务执行已验收。
7. UI-0 renderer 与性能判断必须等待相同 workload 下的真实、可比 driver 样本；fixture 不构成性能证据。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008） |
| SRE Lead | Mavis 接手审核：仅确认 UI 资源与运行时边界，未声称生产性能验收 |
| 平台 | Mavis 接手审核：确认无授权时 fail-closed 门保持 |
| 评审主持 | Mavis 接手审核：定向测试与浏览器证据已复核 |
| PM | Mavis 接手审核：后续顺序按 UI WBS 继续，owner 门缺口保持开放 |

修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.4 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 UI-1/UI-2 提交经 rebase 后快进至本地 dev 的 commit `4dcf567d`；35/35 定向测试、typecheck 通过，授权数据、真实读屏和 driver 样本仍是未完成验收门。 | 完成当前 WBS 集成批次 |
| v0.3 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 恢复 ProjectRunTree 对 Project 选择、独立展开和 ArrowRight 键盘展开的直接回归测试；八个 focused suites 35/35、typecheck 和 12 个源码关系检查通过。UI-0 已集成，但读屏、真实授权导航和 driver 样本仍开放。 | UI 自审发现局部测试覆盖被替换 |
| v0.2 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 UI-1/UI-2 相关实测更新为 8 个 focused suites 32/32、typecheck 和 12 个源码关系检查通过；补充 `/worktree` no-provider 浏览器 AX 复验，并记录 UI-0 已集成以及 CodeRabbit WSL 权限限制。读屏、真实授权导航、driver 样本仍开放。 | 按 UI WBS 继续推进并完成提交前自审 |
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Project-first 桌面导航、移动目录、多语言标签、窄屏无溢出、焦点恢复及 UI-0 后续阻塞状态。 | 用户要求优化 UI/UX 并按 WBS 继续推进 |
