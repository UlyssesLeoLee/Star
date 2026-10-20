# WLD2 Task Card CLI UI 安全切片报告

## §0 目的

落实 `docs/briefs/WLD2-TASK-CLI-UI-20261020.md` 的有限 UI 安全切片：在现有 Task/Run/Worktree API 边界内，避免本地预览看起来像可用终端，并在 Session 状态不确定、断开或终止时停止显示可输入的终端。

本报告只覆盖这次 UI 切片；**WLD-2 未完成，Task CLI 不得标为生产闭环可用。**

## §1 改动矩阵

| 文件 | 改动 | 范围 |
|---|---|---|
| `frontend/src/app/worktree/[id]/group/page.tsx` | 移除无授权 Session 时的仿终端预览；只在当前 Task/Worktree 匹配且状态允许运行时挂载终端；启动回执丢失、Session 状态检查失败、恢复失败或取消失败时将状态标为待确认并卸载终端；终止状态明确与 Task 完成分开；不再声称空 Session 列表证明 provisioner 未配置。 | 只改现有页面调用与展示，不改 API、认证、授权或后端行为。 |
| `frontend/src/lib/group/taskCliPresentation.ts` | 为 UI 定义 `unavailable`、`ready`、`reattach`、`not-running`、`ended`、`uncertain` 六种呈现状态。 | 由已验证 Session 回执、当前状态和不确定标记共同决定终端是否显示。 |
| `frontend/src/lib/group/taskCliPresentation.test.ts` | 覆盖无授权回执、可用运行态、断线/待重接、不确定、启动/取消中与各类终态。 | 5 个 Vitest 测试。 |
| `docs/reports/WLD2-TASK-CLI-UI-SLICE-20261020.md` | 记录行为、验证证据和未完成的生产门禁。 | 不更新共享 WBS 或设计文件。 |

## §2 验证摘要

- `rtk proxy git diff --check`：通过。
- Cypher Graph Guardian：`PASS (3 source file(s) checked)`；检查并刷新页面、状态 helper、测试文件的图头。
- 使用现有 `D:\Star\frontend\node_modules\esbuild` 对上述 3 个 TypeScript/TSX 源文件做语法转换：通过；此项只验证语法，不做类型检查。
- 定向 Vitest：5/5 通过。隔离 Worktree 没有 `frontend/node_modules`；为运行该测试，使用现有 `D:\Star\frontend` Vitest 安装及一个运行后删除的临时最小配置，目标测试和源码均来自本 Worktree。这是定向测试，不代表完整前端环境已安装。
- Worktree TypeScript 检查未能完成：执行 `rtk proxy node D:\Star\frontend\node_modules\typescript\bin\tsc --noEmit --project C:\Users\leo19\.codex\worktrees\wbs-wld2-task-cli\Star\frontend\tsconfig.json`，因本 Worktree 缺少依赖而在大量文件上出现 `TS2307`（包括 `react`、`next/navigation`、`vitest`、`@playwright/test`），并连带 JSX/隐式 any 错误。因此不能据此确认页面编译通过；也不能将该依赖缺失错误归因到本次改动。
- 未运行整套前端测试、浏览器/E2E、服务端执行、目标 PostgreSQL/RLS/grants 或实际 Task CLI。
- CodeRabbit 独立审查未运行：Windows `coderabbit.cmd` shim 调用 `/root/.local/bin/coderabbit` 后被系统以 `Permission denied` 拒绝；已完成定向本地 diff 与行为审查。

## §3 已知缺口

1. 仓库审计记录默认 API composition 未装配生产 `TaskCliSessionProvisioner`；本切片没有创建或假造 provisioner，也未证明受控生产执行器存在。
2. 审计记录现有 Task CLI binding resolver 仍依赖 cloud Branch 身份，尚未满足 hostless Run 下的 Task/Run/Worktree 绑定闭环。
3. OS sandbox、撤权后进程树终止、取消与恢复的持久化/ drain、独立验证、证据写入和结果回写仍未验收。
4. Target PostgreSQL migration 执行、应用角色 grants、FORCE RLS 实际结果、回滚与并发行为未连接验证。
5. 本次 TypeScript 全项目检查因隔离 Worktree 缺少依赖而不可判定；需要安装/提供一致的前端依赖环境后重跑。
6. 本次 helper 测试验证 UI 状态映射，不替代页面级浏览器交互、WebSocket 连接、安全沙箱或生产验收。
7. 本次未修改 WBS、需求、基本设计、详细设计；WBS 中 WLD-2 仍保持开放。

## §4 子代理失败接手清单

- 本切片由当前 worker 按限定 brief 实施；未派生其他 worker，无失败的子代理产出需要接手。
- 根协调者负责在集成前复核本报告、当前 diff 及验证限制；不得把本 UI 切片或现有 REST/API 源码等同于 WLD-2 闭环完成。

## §5 守门规则

1. 终端只接受现有服务端 API 返回并通过当前 Task/Worktree 对齐及 ticket 校验的 Session。
2. 本地预览数据不渲染命令行、光标、假 Session 或可输入终端；不增加任意 Shell、mock provisioner 或本地 seed 执行回退。
3. Session 状态检查、取消或恢复失败时按不确定处理并卸载终端；终态不暗示 Task Card 已完成。
4. 继续使用当前 Run API、Task 身份、Worktree 绑定和已有 Profile/API 契约，不增加 endpoint、不放宽授权。
5. 独立验证、evidence、Hook、sandbox、授权事实、资源预算与结果回写未形成闭环前，执行能力仍受服务端 fail-closed 门禁控制。
6. `D:\Star` 中的依赖仅供这次测试运行读取；本报告不声称在该目录执行了代码修改或完整项目验证。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | Mavis 接手审核：仅批准 UI fail-closed 切片表述；WLD-2 仍开放。 |
| SRE Lead | 未进行真实 provisioner、sandbox、撤权终止或生产运行验收。 |
| 平台 | 未进行目标 PostgreSQL 应用角色授权/RLS 验收。 |
| 评审主持 | 未进行独立页面/E2E 审查。 |
| PM | 本切片不是 Task CLI 生产可用声明。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 WLD2 Task Card CLI UI 安全切片报告，记录实现、测试方式和未关闭门禁。 | `docs/briefs/WLD2-TASK-CLI-UI-20261020.md` |
