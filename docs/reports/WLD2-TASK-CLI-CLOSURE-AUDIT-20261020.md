# WLD-2 Task Card 与任务绑定 CLI 闭环审计

**日期：** 2026-10-20  
**审计范围：** Run-scoped Task Card API/UI、TaskExecutionRun、Worktree 绑定、任务 CLI session/PTY、权限与 sandbox、取消恢复、验证与证据、Audit/Outbox、数据库 RLS/grants。  
**基线：** `codex/wld2-task-cli-audit-20261020` @ `35a52c972a58b339cb790ad30ed07ee4c56a797a`。  
**结论：** Task Card 的数据/API 与受控 CLI admission 源码切片已经存在；Run Directory 内的 CLI 明确禁用。当前仓库尚未形成可证明的、Run 绑定的生产执行闭环：默认 API composition 没有 provisioner，bounded `crates/` 搜索未找到生产实现；现有 CLI start context 仍要求远端 Branch 身份；PTY 只是 Local Runtime 的低层进程/IO 原语，未连接到该 API、OS sandbox、独立验证和执行结果写回。

## 1. §0 目的与边界 / §1 改动矩阵

本报告按 WLD-2 brief 对现有 source 与 migration 作只读核对。标记“存在”仅表示所引源码定义了对应接口/逻辑，不等于生产服务已装配、目标数据库已运行迁移或端到端验收通过。CodeGraph Tier-2 线索由主代理提供；本工作区没有 `check_index_coverage` 工具，因此对相关实现进行了直接源码、migration 和测试搜索；否定性结论仅限本文注明的文件/搜索范围。

| 审计项 | 当前源码证据与状态 |
|---|---|
| Run Task Card API/UI | Run Directory 的 Task Card 列表/创建由 [`RunTaskCardsPanel.tsx`](../../frontend/src/components/run/RunTaskCardsPanel.tsx#L201) 走 Run-scoped client；CLI 按钮在 [`RunTaskCardsPanel.tsx`](../../frontend/src/components/run/RunTaskCardsPanel.tsx#L454) 明确 disabled，提示 admission、Runtime sandbox、独立验证与结果回写未装配。旧 Worktree Group 页面另有“CLI 预览”及受门控的 live request：[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2634)、[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2674)、[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2703)。这是旧 Worktree 页面，不代表 Run Directory Task Card 已接入 CLI。 |
| TaskExecutionRun owner/persistence | CLI API 有授权、readiness、profile/Hook 检查、idempotency 与 provisioner 调用顺序：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L514)。`record_cli_task_run` 持久化不可变 Run snapshot、资源预留、`run_started` 初始 event 和幂等键：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1575)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1659)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1729)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1746)。后续 provisioner 的返回/状态 event 在单独路径追加：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L881)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1793)。它证明有数据库 admission/persistence 逻辑，不证明 provisioner 实际生产运行或跨系统原子提交。 |
| Task—Worktree—Run 绑定 | API 在创建任务 Run 时调用当前目录 binding 解析；没有唯一当前 binding 会失败：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L999)。解析器查询 Run/Worktree binding 并联接 `scm.cloud_branch`、active revision 与相关授权：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1111)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1135)。因此该 CLI 起动路径仍要求 cloud Branch 身份，与当前 WBS 的 hostless Run 方向不一致；WBS 已将 hostless Task CLI 列为不可用：[`WBS`](../implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md#L27)、[`WBS`](../implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md#L79)。 |
| Session admission 与 PTY 调用链 | `TaskCliSessionProvisioner` 提供 start/list/status/cancel/reattach 能力接口；安全 readiness/capability 默认 false：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L254)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L256)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L275)。默认 `GroupApiState::new` 将 provisioner 设为 `None`，只有显式 builder 注入；availability 需要 admission、catalog、profile-bound 全部满足：[`group_api.rs`](../../crates/star-api-rest/src/group_api.rs#L337)、[`group_api.rs`](../../crates/star-api-rest/src/group_api.rs#L348)、[`group_api.rs`](../../crates/star-api-rest/src/group_api.rs#L404)。bounded `rg` 搜索 `crates/` 中 `impl TaskCliSessionProvisioner` 仅发现 trait/引用，没有生产实现。PTY 的 `PreparedTaskCliExecution` 先校验固定绝对 command、Worktree 路径、profile 与一次性 fence；源码说明它只准备/校验，不负责 spawn：[`task_execution.rs`](../../crates/domain-local-runtime/src/task_execution.rs#L254)、[`task_execution.rs`](../../crates/domain-local-runtime/src/task_execution.rs#L391)、[`task_execution.rs`](../../crates/domain-local-runtime/src/task_execution.rs#L473)、[`task_execution.rs`](../../crates/domain-local-runtime/src/task_execution.rs#L552)。`TaskPtySession` 是独立子进程与 PTY IO 原语；文件说明它不做 sandbox 或 ACL recheck：[`task_pty.rs`](../../crates/domain-local-runtime/src/task_pty.rs#L30)、[`task_pty.rs`](../../crates/domain-local-runtime/src/task_pty.rs#L35)。在 `crates/` 的 `TaskPtyManager`/`task_pty::` 搜索没有找到 CLI API 的生产调用方。故不能把低层 PTY 支持描述成 Task Card 到命令 spawn 的完整调用链。 |
| Sandbox、权限与撤权 | CLI start 在 API 边界校验当前 actor/scope、任务/Worktree 上下文、profile/Hook/readiness 与执行预算；`TaskCliSessionProvisioner` 未安装或 readiness 不满足时 fail closed：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L514)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L904)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L922)。读取/取消/重接 session 会重新解析父级 Task/Worktree/Project/Run 绑定与当前 writer 权限：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L2007)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L2065)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L2090)。但 PTY 原语明确要求调用者自行提供 ACL、Runtime、grant、audit 与 sandbox；本审计路径未发现可装配的 OS sandbox provider，也没有证明撤权能终止已运行的 OS 进程树。 |
| Cancel、终止与恢复 | API 暴露 session list/status/cancel/reattach，且先验证父级访问：[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L471)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1937)、[`cli_sessions.rs`](../../crates/star-api-rest/src/group_api/cli_sessions.rs#L1967)。旧 UI 展示 session 历史及取消/重接入口：[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L1073)、[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2731)。这些功能依赖未安装的 provisioner。PTY manager 可 kill/wait 当前直接子进程：[`task_pty.rs`](../../crates/domain-local-runtime/src/task_pty.rs#L242)、[`task_pty.rs`](../../crates/domain-local-runtime/src/task_pty.rs#L341)；未证明子进程树清理、跨进程持久化 session registry、进程重启后恢复、可靠 drain 或输出持久化。 |
| 独立验证、证据与结果回写 | TaskRun 详情 API 可读取 event/evidence projection：[`task_runs.rs`](../../crates/star-api-rest/src/group_api/task_runs.rs#L394)、[`task_runs.rs`](../../crates/star-api-rest/src/group_api/task_runs.rs#L425)。在 bounded Rust 搜索中，`task_execution_evidence` 仅发现读取路径，没有发现 CLI 侧证据写入器或独立 validator/result writer。migration 定义 append-only evidence metadata：[`2026-09-30-worktree-task-execution-run.sql`](../../db/migrations/2026-09-30-worktree-task-execution-run.sql#L238)、[`2026-09-30-worktree-task-execution-run.sql`](../../db/migrations/2026-09-30-worktree-task-execution-run.sql#L301)。数据模型存在不等于验证产物或任务结果已经写回。 |
| Audit、Outbox、RLS 与实际 DB grants | Execution Run、event、evidence、idempotency 有 append-only/immutability trigger 与 FORCE RLS/policy：[`2026-09-30-worktree-task-execution-run.sql`](../../db/migrations/2026-09-30-worktree-task-execution-run.sql#L286)、[`2026-09-30-worktree-task-execution-run.sql`](../../db/migrations/2026-09-30-worktree-task-execution-run.sql#L306)、[`2026-09-30-worktree-task-execution-run.sql`](../../db/migrations/2026-09-30-worktree-task-execution-run.sql#L338)。Task metadata/lifecycle 有独立 Run Outbox trigger，同 owner 事务写入 lifecycle 事实：[`2026-10-02-task-run-owner.sql`](../../db/migrations/2026-10-02-task-run-owner.sql#L46)、[`2026-10-16-task-lifecycle-outbox.sql`](../../db/migrations/2026-10-16-task-lifecycle-outbox.sql#L87)、[`2026-10-16-task-lifecycle-outbox.sql`](../../db/migrations/2026-10-16-task-lifecycle-outbox.sql#L135)。在 `cli_sessions.rs` 中，执行状态直接追加到 `task_execution_run_event`；bounded 搜索未发现同一路径写 `task_run_outbox`。目标 PostgreSQL 的 migration 执行状态、API 应用角色 grants、实际 RLS 结果和 rollback 均未连接验证；静态 migration 不能证明生产部署权限。 |
| Mock/disabled 状态 | Run Directory Task Card 的 CLI 明确禁用，UI 解释缺少的四个闭环门：[`RunTaskCardsPanel.tsx`](../../frontend/src/components/run/RunTaskCardsPanel.tsx#L454)。旧 Worktree Group 的“预览”仍标为 mock preview；live start 仅允许走已授权的 Local Runtime 请求并要求 Approved Launch Profile/Agent Execution Profile：[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2634)、[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2674)、[`page.tsx`](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2703)。因此不能把预览当生产执行，也不应把已存在的 Task Card API/TaskRun persistence 一概说成 mock。 |

## 2. §2 验证摘要

- 本次仅作只读审计；按 brief 只运行 `rtk proxy git diff --check`，结果记录在本报告提交前的集成回执中。没有运行 Rust/frontend tests、build、clippy、target DB migration、PostgreSQL RLS/grants 或 live CLI execution。
- CodeGraph 的有界 Tier-2 线索用于定位 RunDirectory session/client、prepared execution 和 PTY primitive；`check_index_coverage` 不可用。对关键正向和否定结论使用直接源码、migration、测试引用和限定范围的 `rtk proxy rg` 搜索回退。无 coverage clean/completeness 声明。
- 已查看相关测试的静态覆盖范围：`cli_sessions.rs` 包含 binding drift/revoked grant、missing/ambiguous/moved binding、bounded list/receipt/readiness/fence 测试；`task_runs.rs` 的路由测试只核对挂载/auth 门；前端 `worktreeGroupApi.test.ts` 核对 endpoint/method 映射。这些测试均未在本审计中运行，也不构成生产 Runtime、sandbox 或 target PostgreSQL 验收。
- 报告不把旧 Worktree 页面路径、测试 fixture、migration schema 或 REST 接口定义等同于完整 Task Card→CLI→验证→结果闭环。

## 3. §3 已知缺口与最小下一步

1. **缺少生产执行器装配。** `GroupApiState` 默认没有 provisioner；bounded 搜索未发现 `TaskCliSessionProvisioner` 的生产实现。提供具体、受控的 Rust provider 并在 production composition 注册前，UI 保持 disabled。
2. **hostless Run 上下文尚未贯通 CLI。** 当前 CLI 的 binding resolver 要求 cloud Branch 与 revision；按当前 hostless Run 设计，应使用 Run/Worktree/Task 本地权威 binding，远端宿主仅为可选元数据，并覆盖撤权、重绑、Branch 变化与无宿主身份情况。
3. **准备器与 PTY 未形成受控 spawn 链。** 需把签名/一次性 fence、profile digest、Hook snapshot、当前权限复验、预算 reservation、OS sandbox readiness 和 spawn/failed-spawn reconciliation 收敛到单一授权入口；PTY 本身不得接受任意 command/environment、工作目录或宿主挂载。
4. **执行生命周期未闭合。** 需实现可中断取消、进程树终止、崩溃后的可恢复 registry/状态重建、bounded output 持久化与 drain；验证授权撤销/预算到期后执行会停止或保持 fail closed。
5. **验证和写回缺失。** 增加独立 validator、版本化结果/证据写入器和有限大小/摘要的 evidence；执行输出不得自行作为通过证据。Task lifecycle 结果、Audit 与 owner Outbox 必须按各自 owner 的事务边界写入，不能伪造跨 owner 原子性。
6. **数据库部署证明缺失。** 在目标 PostgreSQL 上跑 migration、应用 API 最小权限、FORCE RLS、隔离/撤权/重复请求/并发/rollback 试验；确认 API execution role grants 后才能放行。

**建议的最小实现切片：** 先完成一个 hostless Run→Worktree→Task 的生产 admission/provider 纵切面，包含当前权限复验、Profile/Hook/budget/fence、受限 sandbox 接口、TaskExecutionRun 事实与失败补偿；起初仍保持面向用户的执行按钮关闭，直到目标 DB grants/RLS、进程生命周期、独立验证和结果 Outbox 全部有可重复证据。不要通过 shell fallback 或把 `supports_profile_bound_run_admission` 提前置 true 来缩短切片。

## 4. §4 子代理失败接手清单

本次没有派生子代理，故没有子代理失败或未接手产物。该项不代表其他 WLD lanes 已验收；本报告只覆盖 WLD-2 Task Card/CLI 范围。

## 5. §5 守门规则

1. 默认 composition 缺 provisioner 时，创建 CLI session 必须失败关闭，不能写成已执行，也不能退化为任意 shell。
2. Task Card CLI 只能在当前 Project/Run/Worktree/Task 绑定、actor 权限、Profile/Hook 版本与执行预算均重新验证后启动；hostless Run 不得被 cloud Branch 身份暗中强制依赖。
3. 低层 PTY、签名 grant、UI 控件或 REST contract 均不足以单独称为 CLI 闭环。只有 production provider、授权 spawn、隔离 sandbox、取消/恢复、独立验证、受限证据与 owner 结果写回均打通，才能启用 Run Directory CLI。
4. PTY 的 per-session IO 限额不等于全局并发/RSS 限额；生产 admission 需有全局 session ceiling、排队/背压、公平性、预算释放、超时、终止和 shutdown drain 证据。
5. `task_execution_run_event` 与 `task_run_outbox` 是不同事实/owner 入口。执行事件不能被假定自动发布成任务 lifecycle 或 BI 事件；各 owner 使用本地事务 + Outbox/Inbox、幂等键与持久回执。
6. evidence/read projection 和 migration 的 FORCE RLS 声明不能代替目标部署中的 role grants/RLS 测试；任何授权、binding、profile、Hook、DB 或 Runtime 状态未知时都不接受执行。
7. mock preview 和测试 fixture 必须与生产状态显著分离；禁止把 preview/session UI 显示成已执行成功。

## 6. §6 签字栏

| 角色 | 审阅状态 |
|---|---|
| 架构 | Mavis 接手审核：本报告仅作有界源码审计；production CLI 仍未闭环。 |
| SRE Lead | 未作独立 SRE/目标环境验收；生产恢复、drain、RSS/并发与 DB 可用性仍待证据。 |
| 平台 | 未作独立 Runtime/sandbox/应用角色授权验收。 |
| 评审主持 | 未进行跨域设计评审；本报告不替代 WLD-2 implementation review。 |
| PM | 未进行产品启用审批；Run Task Card CLI 继续保持不可用状态。 |

## 7. §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 WLD-2 Task Card/CLI 闭环只读审计；逐项记录 UI/API、TaskExecutionRun、Worktree binding、PTY、权限/sandbox、取消恢复、验证/Outbox 与 DB 限制。 | `docs/briefs/WLD2-TASK-CLI-CLOSURE-AUDIT-20261020.md` |
