# PHASE-ERUN-P3-TASK-CARD-RECOVERY-REPORT

> **状态**：🟡 有界客户端恢复切片完成；生产创建与 Agent CLI 仍关闭
> **日期**：2026-10-09
> **基线**：本地 `dev` `64d6567f`；隔离分支 `codex/taskcard-recovery`
> **范围**：Task Card create 不确定结果在面板重挂载/页面 reload 后的安全恢复；不包含 Task Owner、导航、迁移、Host provider 或其他分支重构。

## §0 目的

将未决的 Task Card create 请求从 React 组件内存提升到有界异步 IndexedDB，使当前用户可以在同一 WebView profile 内恢复原请求 body/key，并且只能在原 Run、Worktree、授权与 binding 快照仍一致时重试。该切片只补客户端恢复；它不改变 Run-owned Task Card 服务端事务/API，也不表示生产 Run Apps capability 已启用。

## §1 改动矩阵

| 范围 | 改动 | 状态 |
|---|---|---|
| IndexedDB store | 新增 schema 校验、tenant/actor/Project/Run scope、每 scope 单条未决命令、全局 32 条上限及精确删除 | 完成 |
| Run Task Cards 面板 | POST 前必须持久化；重挂载恢复原正文/key；authorization/binding/focus 变化或 29 天超期时阻止重放；清除前要求用户确认已核对 Run 列表 | 完成 |
| 错误处理 | 本地恢复存储不可读/不可写时 fail closed；API 授权错误与 IndexedDB 错误分开呈现 | 完成 |
| 测试 | IndexedDB 跨 store instance round-trip、容量上限、面板 remount、actor 隔离、binding/授权漂移、storage failure、显式清除与超期禁止重试 | 完成 |
| 文档 | 同步 requirements v6.09、basic design v6.06、Task SRS/BD/DD v0.14/v0.7/v1.27、Group DD v4.75、plan v6.42 | 完成 |

## §2 验证摘要

| 验证 | 结果 |
|---|---|
| `npm ci` | Exit 0；Node 22.19.0 低于部分依赖声明的 22.22.2，出现 EBADENGINE 警告；依赖审计提示 5 个现有依赖树漏洞，本切片未改变 lockfile 或新增依赖 |
| 定向 Vitest | `npm run test -- src/components/run/RunTaskCardsPanel.test.tsx src/lib/run/taskCardCreateRecoveryStore.test.ts`：2 files、15/15 tests 通过 |
| TypeScript | `npm run typecheck`：`tsc --noEmit` exit 0 |
| 其余验证 | `cgg_guard.py --changed --stamp`：PASS，检查 4 个变更源码文件；`git diff --check`：PASS。CodeRabbit CLI 已安装包装器但其 WSL 二进制 `/root/.local/bin/coderabbit` 返回 `Permission denied`，因此未获得独立 CodeRabbit 审查结论。未运行生产目标 DB/RLS、完整前端 build/e2e 或 Rust workspace gates；本切片没有 Rust、schema 或 owner API 修改 |

## §3 已知缺口

1. 可信 `RunDirectoryHostSession`、Run Apps 服务端 capability、Task Owner 目标数据库 migration/grants/RLS 仍未验收；生产创建继续 fail closed。
2. IndexedDB 仅在本机 WebView/browser profile 保存请求正文与幂等键，此切片未加密；浏览器清理或用户重置会丢失记录。本地记录缺失不能证明原 POST 未成功。
3. 客户端 29 天重试窗口短于服务端 `multica.task_command_idempotency` 30 天保留；超期后必须人工核对 Run Task Cards 并显式清除。该机制不承诺 exactly-once、跨设备恢复或 server-side reconciliation。
4. Run App 其它 tab、任务编辑与任务卡内 CLI 仍未闭环；CLI sandbox、授权、取消/恢复、独立验证与结果回写 gates 保持关闭。
5. npm 使用 Node 22.19.0 时有依赖 engine 警告和 5 项审计告警；未执行完整 build/e2e。

## §4 子代理失败接手清单

本阶段未派发子代理；无失败子任务或接手事项。所有变更集中在单一隔离工作树，没有从其他大分支整体合入内容。

## §5 守门规则（本阶段适用 17 项）

1. Worktree-first 主架构与 Project → Branch → Run → Worktree 归属保持不变。
2. 只修改 Task Card uncertain-create recovery 的最小代码与直接关联文档。
3. 创建入口仍要求当前 Run capability、`work-item:write` 和有效 Worktree focus。
4. 服务端继续推导 canonical Run owner 并复验当前 actor/Project/Run 权限。
5. Task metadata、lifecycle、Worktree association、audit、idempotency 与 Run Outbox 仍由原 owner transaction 管理。
6. 客户端在 POST 前先保存完整 body、correlation ID、Idempotency-Key 与 target 快照。
7. 本地 scope 包含 tenant、actor、Project 与 Run；认证 token/credential 不进入 IndexedDB。
8. 一个 actor/Run scope 最多一条未决创建，整个 store 最多 32 条。
9. 客户端 29 天重放窗口短于服务端 30 天幂等保留期。
10. Worktree、Run、authorization binding ID/version、context/focus version 改变时禁止重放。
11. IndexedDB load/save 失败时创建 fail closed。
12. 错误结果使用相同请求体与相同幂等键重试。
13. 过期或 target 不匹配时先人工检查 Run Task Cards，确认后才可清除记录。
14. 本地任务正文未加密且数据可被 WebView 清除，不宣称 exactly-once 或跨设备恢复。
15. 浏览器/本地 storage 中无记录不得被表述为服务端没有创建任务。
16. 宿主认证、目标 DB/RLS/capability 未验收前不得宣称生产 Task Card create 可用。
17. Task Card CLI、Run execution、Agent Runtime、BI/Benchmark 与 Schedule gate 不因本切片开放。

## §6 签字栏

| 角色 | 审核人 | 结论 |
|---|---|---|
| 架构负责人 | 架构师（Mavis 接手 agent per DEC-008） | 🟢 仅认可本地恢复切片与架构边界 |
| SRE Lead | Mavis 接手审核 | 🟢 保留目标 DB/RLS/宿主生产门 |
| 平台负责人 | Mavis 接手审核 | 🟢 确认未新增运行时或 schema 依赖 |
| 评审主持 | Mavis 接手审核 | 🟢 定向测试、typecheck、Cypher Guardian 与 diff check 通过；CodeRabbit 不可执行及未运行的 gates 已列明 |
| PM | Mavis 接手审核 | 🟢 用户可见的不确定请求恢复有界且明确提示风险 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录有界 IndexedDB recovery 实现、15/15 定向测试、TypeScript typecheck 与未关闭的生产 DB/RLS/Host/CLI 缺口 | 完成 Task Card 跨 remount/reload recovery 切片 |
