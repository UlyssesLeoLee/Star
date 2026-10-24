# PHASE-WLD5-GLOBAL-HOOK-NAVIGATION-20261024

## §0 目的

核验未保存 Hook 策略草稿在当前主应用导航中的保护边界，补齐一个已存在实现路径的测试覆盖，并将验证结果与未覆盖导航类别同步到自动化设计和索引。此报告只说明前端保护能力，不代表生产身份认证或所有路由方式均已受保护。

## §1 改动矩阵

| 范围 | 改动/现状 | 结果 |
|---|---|---|
| `frontend/src/lib/navigation/UnsavedNavigationGuard.tsx` | 审阅现有文档级捕获、取消/放行、同源判定和 `router.push` 路径；本切片无代码变更 | 行为由既有实现提供 |
| `frontend/src/components/navigation/GuardedLink.tsx` | 审阅 Next `onNavigate` 确认和取消路径；本切片无代码变更 | 行为由既有实现提供 |
| `frontend/src/components/navigation/GuardedLink.test.tsx` | 新增嵌套内容元素触发包含它的同源锚点保护测试 | 新增 1 项覆盖 |
| `docs/automation-design.md` | 更新 §4.99 状态、命令结果与明确边界 | 与当前证据同步 |
| `scripts/automation/registry.md` | 记录本 lane brief、报告、dispatch 审计日志并保留先前 dispatch 记录 | 索引同步 |
| `docs/reports/WBS-DISPATCH-WLD5-GLOBAL-HOOK-NAVIGATION-20261024.jsonl` | 已由 `SubagentDispatcher.brief(...)` 创建并记录 brief 事件 | 调度记录可追溯 |

## §2 验证摘要

- `pnpm install --frozen-lockfile --offline`（在 `frontend` 下）成功；从本地 store 重用 330 个依赖，无网络下载。
- `pnpm --dir frontend exec vitest run 'src/components/navigation/GuardedLink.test.tsx' 'src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx'`：2 个测试文件通过，24/24 测试通过。
- `pnpm --dir frontend typecheck`：`tsc --noEmit` 通过。
- Rust/Cargo gate 不适用：本切片没有 Rust 代码改动。
- `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp`：PASS，检查并刷新 1 个变更 TSX 文件的图关系头。
- `python scripts/automation/registry_check.py`：exit 0，`errors=0`，报告 203 条 warning。

## §3 已知缺口

1. 浏览器 Back/Forward 触发的 SPA `popstate` 未由当前公共 App Router API 实现拦截。`beforeunload` 不证明 SPA 历史导航已受保护。
2. 任意程序化 `router.push` 只有在调用方显式执行 `confirmNavigation()` 时才受保护。当前共享命令栏显式调用该守卫、用户菜单使用 `GuardedLink`、普通锚点由文档捕获处理；本切片未审计或改造每个页面级路由调用点。
3. 本切片未增加真实浏览器 E2E/手工运行时证据，Vitest 与 TypeScript 结果不能替代它们。
4. 生产宿主身份/认证仍属独立未完成项；本切片没有实现或宣称该能力。

## §4 子代理失败接手清单

本 lane 未再派生子代理。当前 worker 自行完成源码/测试审阅、文档同步和本报告；无失败子代理产物需要接手。父任务已完成独立复核、验证和 dev 集成。

## §5 守门规则

1. 只声明测试直接证明的链接类别；不把有限测试概括为全导航覆盖。
2. 取消操作须阻止默认导航与后续点击副作用；放行操作须只调用一次应用路由。
3. dirty draft guard 在草稿清除或所有者卸载后必须清理。
4. 外链、修改键、新标签、下载及同路径同查询导航按既定豁免行为处理。
5. Back/Forward、未显式确认的程序化路由、浏览器 E2E 与生产宿主认证继续列为未验收项。
6. 仅本报告中的实际命令结果可作为已通过证据；不以 mock 证明生产认证或路由运行时。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 父任务独立复核完成；提交 `08ce3139` 已快进合入 `dev` |
| SRE Lead | 不适用：本切片未改动部署、消息传输或运行时基础设施 |
| 平台 | 不适用：本切片未改动服务端、数据库或 Rust 平台代码 |
| 评审主持 | 父任务独立审阅完成；合入后聚焦验证通过 |
| PM | WLD5 的未覆盖范围已显式列出；是否扩大为全路由保护由后续 WBS 决策 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核（worker 起草，父任务复核待完成） | 增加嵌套锚点测试；记录现有支持路径、实际 Vitest/typecheck/Guardian/registry-check 结果，以及 Back/Forward、未显式确认的程序化导航和生产宿主认证缺口 | WLD5 lane implementation audit |
| v0.2 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录父任务独立复核完成、合入 `dev@08ce3139` 及合入后的 24/24 聚焦测试、前端类型检查和 diff 检查结果；保留未覆盖导航与认证边界 | WLD5 review and integration closeout |
