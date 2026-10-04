# PHASE-9F4C Schedule Admission and Run-as Authorization Report

> 状态：🟡 9F4C-A 数据持久化与 9F4C-B API enable-time 授权切片已实现并验证；生产 Schedule execution 未开放。
> 日期：2026-10-05
> 版本：v0.4
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

落实 Run-scoped Schedule Rule 的 occurrence→TaskExecutionRun 持久化边界，并采纳用户确认的身份策略：Rule 创建者固定为 `run-as`；无人值守每次触发重新检查其 Project/Run 权限；撤权或无法确认当前授权时 fail closed。

本阶段包含 9F4C-A Postgres 持久化约束与 9F4C-B Rule create/enable API 授权门，并记录可重复的隔离验证。它没有创建生产 occurrence worker 或 application admission writer，因此不能宣称调度执行闭环已完成。

## §1 改动矩阵与验收追溯

| 范围 | 结果 | 验收/文档 |
|---|---|---|
| Dispatch → Run | 新增 nullable `occurrence_dispatch.admitted_run_id`，tenant/run 复合 FK；只允许 `leased → admitted` 时绑定，绑定不可清空/替换；数据库核对 schedule-origin 与 occurrence 一致 | requirements AC-LOOP-009；Schedule API SRS/BD/DD v0.10 |
| Schedule Run Outbox | 新增 tenant-scoped append-only Outbox；复合 FK 固定 Rule revision、occurrence、Run/Task/Project；检查 schedule origin、agent channel、run-as 与 dispatch link；每 occurrence/event type 唯一 | Data Design v1.2 §4.13.5；Group DD v4.45 §8.10/§10.2 |
| Rule enable-time run-as authorization | editor 与执行身份分离；enabled create/revision 在写事务复验固定 run-as actor 的当前 Project/Run writer grant 与 active Run，保留当前 Branch binding 检查；管理员撤权后仍可停用 | requirements v5.66/SRS v0.10、basic v5.63/BD v0.10、Group DD v4.45、API source |
| Runtime DB boundary | 新增表启用并强制 tenant RLS；runner 以非 superuser runtime role 验证 tenant 隔离 | disposable PostgreSQL 18.6 |
| Automation runner | 将 admission migration 纳入四 migration chain，增加合法关联、run-as mismatch、Run link immutable、Outbox UPDATE refusal 与 tenant isolation assertions；将 9F4C-B creator-selection 和角色授权 helper 两个 Rust API tests 纳入 runner | `scripts/automation/phase9f3_schedule.py`；automation-design v2.4 §4.46/§4.48；registry v0.54 |
| 设计与计划 | 同步主需求、基本设计、详细设计、数据设计、Schedule API addenda、Worktree Group 实施计划及阶段报告 | requirements v5.66；basic v5.63；Group DD v4.45；Data Design v1.2；plan v5.88；Schedule API SRS/BD/DD v0.10 |

## §2 验证摘要

| 验证 | 结果 |
|---|---|
| `python -m py_compile scripts/automation/phase9f3_schedule.py` | exit 0 |
| `scripts/automation/phase9f3_schedule.py` 完整 runner（显式 Rust 1.98.1 cargo/rustfmt 与 Docker PostgreSQL 18.6） | 修正后的最终完整执行 overall `passed`；focused rustfmt、domain tests、API/adapter all-target checks、targeted Clippy 与 PostgreSQL integration harness compile 通过；四 migration chain 在同一 disposable DB 中双次应用 |
| PostgreSQL migration chain | 9F2、9F4A、9F4B、9F4C-A 完整链重复应用；8 张 Schedule 表均启用 `FORCE ROW LEVEL SECURITY` |
| DB admission assertions | 合法 schedule-origin Run/dispatch/Outbox 通过；run-as mismatch、已绑定 Run 替换与 Outbox UPDATE 被拒。SQL guard 也拒绝 Outbox DELETE/TRUNCATE 和其它 source/channel mismatch，但当前 runner 未单独覆盖这些负例 |
| runtime tenant isolation | `NOSUPERUSER NOBYPASSRLS` fixture role 仅读取自己的 tenant Outbox row |
| 既有 adapter cases | 五个 ignored PostgreSQL adapter scenarios 全通过 |
| 应用层 admission | 未实现、未验证；没有 production worker/writer，也未测试 reservation transaction rollback/concurrent admission |
| 9F4C-B API authorization | `cargo check --locked -p star-api-rest --all-targets -j 4` 与 focused rustfmt 通过；`cargo test --locked -p star-api-rest --lib task_execution_rules_do_not_grant_agent_role_schedule_authority -j 4 --target-dir E:\DevCache\cargo\target-schedule-auth-20261005` 1 passed/130 filtered。新 ACL SQL 没有 canonical Project/Branch/Run grant disposable PostgreSQL fixture，不能据此声称成功/撤权 SQL 运行路径已验证 |
| 可复跑 API 授权测试 | `phase9f3_schedule.py` 已登记 creator-selection 与 role-policy helper 两个 Cargo test steps；本次独立 target 下 role-policy test 1/1 通过。完整 runner 未为本次复核再次运行，且现有 runner不加载 canonical Directory ACL fixture |
| 链接与追溯守门 | 最终 focused API helper test 使用独立 Cargo target 链接成功；`registry_check.py` exit 0、0 errors、191 warnings；`git diff --check`、focused rustfmt 与 CGG freshness check 通过 |
| 目标环境 | 未连接；目标 migration principal、database grants 与生产 RLS 尚未验收 |

Docker runner 使用 disposable PostgreSQL 18.6、loopback-only port mapping 和本次创建的精确容器名；容器由 runner 清理。它不修改目标数据库。
9F4C-B focused test 首次使用共享 Cargo target 时遇到 `LNK1104`，独立 `--target-dir` 重跑后 1/1 通过；报告只将最终隔离 target 成功计为测试证据。纯角色策略 helper 单测不执行新增授权 SQL。


自审发现原 run-as mismatch fixture 与合法 Outbox 共用 occurrence/event 唯一键，可能只触发唯一约束；已将负例移至合法 Outbox 插入前，并要求命中精确的 trigger error。随后首次重跑暴露容器内 `psql` 走默认 socket 的 runner 连接错误；改为容器内 `127.0.0.1` TCP 后，完整 runner 最终通过。报告只计入修正后的最终成功证据。

## §3 已知缺口

1. 没有 occurrence scheduler/worker；Schedule execution capability 保持关闭。
2. 没有 application admission writer；当前数据库 trigger 只要求 `leased → admitted` 并检查 dispatch 状态/fencing/attempt 序列，不知道 worker 提交的 lease owner/generation。
3. 未来 writer 必须通过条件更新复验当前 lease owner、generation、expiry 和 occurrence deadline。
4. 每次新触发、retry、resume 都未通过真实服务重新校验固定 `run_as_actor_id` 的 Project binding 与 Engineering Run grant；撤权和授权目录不可用时必须 fail closed。
5. 当前没有权威 target、Launch Profile、Agent Profile、HookSet 或 provider/grant resolver 的 admission-time 接线。
6. Reservation、TaskExecutionRun、RunEvent、dispatch link、occurrence event 与 Schedule Run Outbox 尚无 application-level 同事务提交与 rollback proof。
7. 未验证并发同 occurrence admission、stale worker 被拒、Outbox delivery/replay 或 consumer 幂等。
8. 目标环境 migration owner、runtime role、schema/table grants、RLS 与权限矩阵未验证。
9. BI/Benchmark 还未消费 admission Outbox，也未验证 coverage、拒绝原因、misfire 或 schedule success 的正式投影。
10. 旧 dispatch 行不回填猜测的 Run；无精确证据时 `admitted_run_id` 保持 NULL，历史状态不被伪造成已关联。
11. 9F4C-B 新增授权 SQL 尚无带 canonical `permission.project_role_binding`、`permission.cloud_branch_role_binding`、`permission.engineering_run_role_binding` 的 disposable fixture；success/revoked/paused/concurrent grant-change SQL path 未验证。

## §4 子代理失败接手清单

本阶段没有派发子代理，因而没有子代理 RPC、brief、输出收集或接手项。全部变更由当前实施 lane 完成；下阶段 worker/writer 仍须按仓库要求先登记 automation brief 并有界验证。

## §5 守门规则（16 项）

| # | 守门规则 | 本阶段状态 |
|---:|---|---|
| 1 | 创建时 `run_as_actor_id` 来自已授权创建者，调用方不得自选 | 9F4B 已建立，9F4C 沿用 |
| 2 | 每次 schedule trigger/retry/resume 复验该主体当前 Project membership 与 Run grant | 未实现；worker 缺失时保持关闭 |
| 3 | 撤权、身份/授权服务不可用或事实不完整均 fail closed | 需求/设计已固定，生产 worker 未验收 |
| 4 | Worktree、Task、target、Profile、HookSet 与当前 Run binding 在 admission 前重新核对 | 尚未实现 |
| 5 | Run 固定到 occurrence 和精确 Rule revision；不得以新规则改写旧 occurrence | DB 关联约束已验证 |
| 6 | Dispatch 必须以条件更新校验 lease owner、generation、expiry、deadline | 必须由未来 writer 实现；DB trigger 不校验提交者 lease token |
| 7 | Dispatch 只能在 `leased → admitted` 时写 admitted Run link | DB 正反例已验证 |
| 8 | Admitted Run 必须是 schedule-origin，且引用相同 occurrence | DB 正反例已验证 |
| 9 | Outbox Run 必须匹配 tenant/Project/Engineering Run/Task、Rule revision、agent channel 与 run-as | DB FK/trigger 已建立；runner 验证正向关联与 run-as mismatch，其它 source/channel mismatch 尚未单独测试 |
| 10 | `admitted_run_id` 设置后不可替换或清空 | DB mutation 拒绝已验证 |
| 11 | Schedule Run Outbox append-only，并按 occurrence/event type 幂等唯一 | DB trigger 与唯一键已建立；runner 验证 UPDATE 拒绝，DELETE/TRUNCATE 负例尚未单独执行 |
| 12 | Schedule 数据以 tenant FORCE RLS 隔离，runtime role 不得是 superuser/BYPASSRLS | 8 表 RLS 与 disposable non-superuser 隔离已验证 |
| 13 | 生产授权不得继承 migration owner 或超权角色 | 目标 grants 未知，尚未验收 |
| 14 | worker/queue 必须有界、可取消、可恢复并尊重资源预算 | 本阶段未创建 worker，后续阶段必验 |
| 15 | Outbox consumer 与 BI 必须幂等且显式报告 partial/unknown coverage | consumer/BI 未实现 |
| 16 | enabled Rule 写入只验证当下固定 run-as 权限，不能替代每次 trigger/retry/resume 重新授权 | API enable gate 已实现；纯角色 helper 1/1；SQL ACL 集成未验证，worker per-trigger gate 未实现 |

## §6 签字栏

| 角色 | 结论 | 审核者 |
|---|---|---|
| 架构 | 通过已验证的 DB persistence 与 API enable-time authorization 切片审查；SQL ACL 集成和 per-trigger reauthorization 未关闭，不批准生产 execution capability | 架构师（Mavis 接手 agent per DEC-008） |
| SRE Lead | PostgreSQL 18.6 persistence evidence 与 Rust role-helper 1/1 通过；API ACL SQL fixture、目标 DB/grants 仍开放 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
| 平台 | 4 migration chain、8 表 FORCE RLS 隔离 fixture 与 focused API helper test 通过；API ACL SQL fixture 缺失 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
| 评审主持 | 运行态边界与未实现项已分开记录，未把 DB guard 误报为 lease-owner admission | 架构师（Mavis 接手 agent per DEC-008） |
| PM | 本阶段可进入后续 writer/worker 阶段；Schedule 对用户仍不可用 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9F4C-A persistence migration、runner/isolated PostgreSQL 证据、设计追溯与未关闭生产闭环门槛 | 用户确认 immutable run-as 撤权 fail-closed 并要求继续 Schedule admission 工作 |
| v0.2 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正 run-as mismatch fixture 的唯一键误判风险，改为精确验证 trigger 错误；将 Docker `psql` 改为容器内 loopback TCP；记录修正后的最终全量 runner 通过，早期失效/不完整证据不计为成功 | 提交前自审发现负例被唯一键遮蔽并复跑发现 Docker socket 连接问题 |
| v0.3 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 9F4C-B enable-time run-as reauthorization、纯角色 helper 1/1 链接测试与独立 target 证据；明确新增 ACL SQL fixture 和 per-trigger worker/admission 仍缺，Schedule execution 不开放 | 用户确认创建者固定为 run-as、每次触发重验且撤权 fail closed；完成 API enable gate 实装及最终验证 |
| v0.4 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将已通过的 9F4C-B role-policy helper test 纳入 Schedule 自动化 runner；同步 runner、registry、automation-design 与实施计划证据，明确完整 runner 未在本次复核重跑 | 提交前自审发现 focused role-policy test 尚未纳入 [P] runner |
