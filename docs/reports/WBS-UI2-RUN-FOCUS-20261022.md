# WBS UI-2：Run 未选 Worktree 焦点

日期：2026-10-22
状态：实现已集成到本地 dev（`ff3b142d`）；聚焦 Vitest 与 TypeScript 检查通过。此状态不表示完整 UI-2 或生产 Run App 已验收。

## §0 目的

修正规范 Run 路由在尚未验证 Worktree 焦点时直接显示任务卡的问题。无焦点时只呈现 Run 摘要、授权 Worktree 选择入口和 GLOBAL 聊天范围；收到并验证具体 Worktree 的 RunContext 后，才呈现 Run App tabs 及其面板。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `frontend/src/components/run/RunWorkspaceShell.tsx` | 无焦点时通过现有 Run Directory API 读取该 Run 的 Worktree 目录页，提供关闭预取的规范 Worktree 链接；不渲染 App tabs、Task Cards、BI 或 App 状态说明。错误状态提供重试；空目录和分页情况给出明确说明。WORKTREE 聊天范围只在匹配的 RunContext 确认焦点后启用。 | 完成 |
| `frontend/src/components/run/RunWorkspaceShell.test.tsx` | 覆盖无焦点时不挂载 Task Cards/BI、不出现 App tabs、仅请求 Worktree 目录及显示 GLOBAL；覆盖匹配 Worktree 焦点验证后显示现有 App 面板。 | 2/2 通过 |
| `docs/reports/WBS-UI2-RUN-FOCUS-20261022.md` | 记录实现、验证结果和未完成门禁。 | 完成 |

Worktree 选择只调用授权目录端点，不加载 App 数据；点击链接后由目标规范路由重新取得并验证 Worktree RunContext，不在列表点击时乐观设置焦点。没有增加 API、授权范围或生产 seed/mock。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk git diff --check` | PASS，退出码 0 |
| `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | PASS，2 个变更源码文件通过并刷新 Cypher freshness stamp |
| `rtk pnpm install --offline --frozen-lockfile` | PASS；使用本机包缓存完成隔离 Worktree 依赖安装，没有改动锁文件 |
| `rtk proxy pnpm exec vitest run src/components/run/RunWorkspaceShell.test.tsx`（工作目录 `frontend/`） | PASS，1 个文件 / 2 个测试 |
| `rtk proxy pnpm exec tsc --noEmit -p tsconfig.json`（工作目录 `frontend/`） | PASS，退出码 0 |
| `npm ci --cache ...`（隔离 Worktree） | 首次尝试因 registry ECONNRESET 失败；随后离线 pnpm 安装成功，未保留依赖清单改动 |

未安装依赖，也未修改依赖清单。不能将未运行的 Vitest 或 TypeScript 检查标记为通过。

## §3 已知缺口

1. 验证覆盖的是本次 Run Shell 聚焦测试及前端全量 TypeScript 检查；未运行全量 Vitest、构建或浏览器 E2E。
2. Worktree 选择器显示服务端返回的首个分页；有 `next_cursor` 时提示用户到左侧 Project 目录继续加载。该实现不改变目录分页预算。
3. 当前页面的 GLOBAL 聊天范围可选，但聊天 Runtime 尚未装配，输入和发送仍禁用；本阶段没有宣称聊天发送闭环完成。
4. UI-2 不完成其余 Run App、Worktree 生命周期、任务执行或整体 WBS 的验收。

## §4 子代理失败接手清单

本阶段由 `/root/wbs_ui2_no_focus` 子代理在本报告所述独立 Worktree 内实施；主协调者负责差异复核、集成与阶段文档更新。没有嵌套子代理。

## §5 守门与边界

- 路由仍按 Project → Engineering Run → Worktree → Branch 规范身份处理。
- Worktree ID 必须与服务端返回的焦点一致才会设置会话焦点。
- 未验证焦点时不挂载 Task Cards/BI，不呈现 Run App tabs；只读取授权 Worktree 目录。
- Worktree 路由链接禁用预取，目标页面再次执行焦点授权验证。
- 保留请求取消和授权错误关闭行为；没有新增第三方依赖或生产 mock 数据。
- Cypher 头部已按当前调用与类型关系更新并通过 guard。

## §6 审核状态

实现 worker 自查完成；主协调者复核并复跑聚焦 Vitest、TypeScript 与 Cypher guard。UI-2 no-focus 源码提交 `ff3b142d` 已包含在本地 dev，报告与当前 dev 历史一致。该集成不关闭右侧完整 App 工作区、分页搜索、聊天 Runtime/send、浏览器 Back/Forward、Run 生命周期 provider 或生产 owner 验收。

## §7 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.3 | 2026-10-22 | 将集成状态更新为 UI-2 no-focus 提交 `ff3b142d` 已进入本地 dev；明确仍未关闭的完整 UI 与生产 Run App 门禁。 |
| v0.2 | 2026-10-22 | 主协调者使用冻结锁文件的离线 pnpm 安装复跑聚焦 Vitest（2/2）与 TypeScript 检查（退出码 0），更新子代理执行及审核状态；未运行全量 UI/E2E 构建。 |
| v0.1 | 2026-10-22 | 实施 UI-2 无 Worktree 焦点时的 App 挂载门禁、授权目录选择器、GLOBAL/WORKTREE 范围门控和聚焦测试；记录依赖缺失导致的验证阻塞。 |
