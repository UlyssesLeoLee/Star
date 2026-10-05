# PHASE-9F4C Schedule Admission and Run-as Authorization Report

> 状态：🟡 9F4C-A 数据持久化、9F4C-B API enable-time 授权与 9F4C-C ACL 撤权竞态排序切片已实现并验证；生产 Schedule execution 未开放。
> 日期：2026-10-05
> 版本：v0.6
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

落实 Run-scoped Schedule Rule 的 occurrence→TaskExecutionRun 持久化边界，并采纳用户确认的身份策略：Rule 创建者固定为 `run-as`；无人值守每次触发重新检查其 Project/Run 权限；撤权或无法确认当前授权时 fail closed。

本阶段包含 9F4C-A PostgreSQL 持久化约束、9F4C-B Rule create/enable API 授权门，以及 9F4C-C run-as 授权与 Project/Branch/Run ACL mutation 的事务锁排序，并记录可重复的隔离验证。它没有创建生产 occurrence worker 或 application admission writer，因此不能宣称调度执行闭环已完成。

## §1 改动矩阵与验收追溯

| 范围 | 结果 | 验收/文档 |
|---|---|---|
| Dispatch → Run | 新增 nullable `occurrence_dispatch.admitted_run_id`，tenant/run 复合 FK；只允许 `leased → admitted` 时绑定，绑定不可清空/替换；数据库核对 schedule-origin 与 occurrence 一致 | requirements AC-LOOP-009；Schedule API SRS/BD/DD v0.12 |
| Schedule Run Outbox | 新增 tenant-scoped append-only Outbox；复合 FK 固定 Rule revision、occurrence、Run/Task/Project；检查 schedule origin、agent channel、run-as 与 dispatch link；每 occurrence/event type 唯一 | Data Design v1.4 §4.13.5；Group DD v4.46 §8.10/§10.2 |
| Rule enable-time run-as authorization | editor 与执行身份分离；enabled create/revision 在写事务复验固定 run-as actor 的当前 Project/Run writer grant 与 active Run，保留当前 Branch binding 检查；管理员撤权后仍可停用 | requirements v5.68/SRS v0.12、basic v5.65/BD v0.12、DD v0.12、API source |
| ACL revoke concurrency | authorization helper 与 Project/Branch/Run grant mutation triggers 共用 tenant+Project transaction advisory lock；API 获锁后以独立 SQL statement 重读 ACL，目标 runtime 不获 ACL UPDATE | 9F4C-C migration；Project/Branch/Run 三组双连接撤权等待与 post-commit fail-closed 测试 |
| Runtime DB boundary | 新增表启用并强制 tenant RLS；runner 以非 superuser runtime role 验证 tenant 隔离 | disposable PostgreSQL 18.6 |
| Automation runner | 将 9F4C-C ACL lock migration 纳入五个 Schedule migration chain；加入 Project、Branch、Run 三类真实并发撤权序列测试，保留现有 occurrence/Outbox/RLS/tenant assertions | `scripts/automation/phase9f3_schedule.py`；automation-design v2.6 §4.50；registry v0.56 |
| 设计与计划 | 同步主需求、基本设计、详细设计、数据设计、Schedule API addenda、Worktree Group 实施计划及阶段报告 | requirements v5.68；basic v5.65；Data Design v1.4；plan v5.91 §6.82；Schedule API SRS/BD/DD v0.12 |

## §2 验证摘要

| 验证 | 结果 |
|---|---|
| `python -m py_compile scripts/automation/phase9f3_schedule.py` | exit 0 |
| `scripts/automation/phase9f3_schedule.py` 完整 runner（显式 Rust 1.98.1 cargo/rustfmt 与 Docker PostgreSQL 18.6） | 最终执行 overall `passed`；focused rustfmt、domain tests、API/adapter all-target checks、targeted Clippy 与 PostgreSQL integration harness compile 通过；五 migration chain 在同一 disposable DB 中双次应用 |
| PostgreSQL migration chain | 9F2、9F4A、9F4B、9F4C-A、9F4C-C 完整链重复应用；8 张 Schedule 表均启用 `FORCE ROW LEVEL SECURITY` |
| DB admission assertions | 合法 schedule-origin Run/dispatch/Outbox 通过；run-as mismatch、已绑定 Run 替换与 Outbox UPDATE 被拒。SQL guard 也拒绝 Outbox DELETE/TRUNCATE 和其它 source/channel mismatch，但当前 runner 未单独覆盖这些负例 |
| runtime tenant isolation | `NOSUPERUSER NOBYPASSRLS` fixture role 仅读取自己的 tenant Outbox row |
| 既有 adapter cases | 五个 ignored PostgreSQL adapter scenarios 全通过 |
| 应用层 admission | 未实现、未验证；没有 production worker/writer，也未测试 reservation transaction rollback/concurrent admission |
| 9F4C-B API authorization | Full runner 编译并执行授权 helper；canonical disposable PostgreSQL fixture 1/1 通过，覆盖 active developer grants、Project/Branch/Run 分别撤权及 paused Run 拒绝 |
| 授权 SQL 权限边界 | canonical ACL helper 以 directory ACL 表只读 runtime role 通过 active/revoked/paused 场景；helper 校验 scope 后取得 tenant+Project transaction advisory lock，API 后续 statement 读取 ACL；Project/Branch/Run 三种撤权均等待锁持有者提交，随后撤权成功且后续授权拒绝 |
| 链接与追溯守门 | `registry_check.py` exit 0、0 errors、191 warnings；最终 `git diff --check`、focused rustfmt 与 CGG freshness check 通过 |
| 目标环境 | 未连接；目标 migration principal、database grants 与生产 RLS 尚未验收 |

Docker runner 使用 disposable PostgreSQL 18.6、loopback-only port mapping 和本次创建的精确容器名；容器由 runner 清理。它不修改目标数据库。
9F4C-B focused test 首次使用共享 Cargo target 时遇到 `LNK1104`，曾使用独立 target 隔离验证。该早期测试不执行授权 SQL；本轮最终完整 runner 已另行通过 canonical PostgreSQL ACL fixture，报告以该数据库用例作为授权 SQL 证据。


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
11. 尚未验证完整 REST create/revise 事务回滚、Run/Project mismatch、完整角色矩阵、target/profile 当前性、并发 CAS/replay、成功响应缓存头及生产目标 grants；本轮已单独验证 enable-time authorization 与 Project/Branch/Run grant mutation 的锁排序。生产 worker 触发、retry、resume reauthorization 仍未实现。

## §4 子代理失败接手清单

本阶段没有派发子代理，因而没有子代理 RPC、brief、输出收集或接手项。全部变更由当前实施 lane 完成；下阶段 worker/writer 仍须按仓库要求先登记 automation brief 并有界验证。

## §5 守门规则（17 项）

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
| 16 | enabled Rule 写入只验证当下固定 run-as 权限，不能替代每次 trigger/retry/resume 重新授权 | API enable gate、active/revoked/paused SQL、ACL mutation serialization 已验证；worker per-trigger gate 未实现 |
| 17 | Project/Branch/Run grant mutation 与 API run-as authorization 必须共用 tenant+Project transaction lock，API 获锁后需在后续 statement 复读当前授权 | PostgreSQL 18.6 Project/Branch/Run 三类双连接撤权均观察到等待；授权提交后撤权完成，后续授权 fail closed |

## §6 签字栏

| 角色 | 结论 | 审核者 |
|---|---|---|
| 架构 | 通过已验证的 DB persistence、API enable-time authorization 与 ACL revoke ordering 切片审查；完整 REST 事务和 per-trigger reauthorization 未关闭，不批准生产 execution capability | 架构师（Mavis 接手 agent per DEC-008） |
| SRE Lead | PostgreSQL 18.6 migration/RLS、canonical ACL active/revoked/paused 与三类撤权锁排序 evidence 通过；目标 DB/grants 和写事务仍开放 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
| 平台 | 5 migration chain、8 表 FORCE RLS、canonical directory ACL fixture、三类撤权 race 与 focused runner 通过；生产 grants 和 Schedule worker 未验收 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
| 评审主持 | 运行态边界与未实现项已分开记录，未把 DB guard 误报为 lease-owner admission | 架构师（Mavis 接手 agent per DEC-008） |
| PM | 本阶段可进入后续 writer/worker 阶段；Schedule 对用户仍不可用 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9F4C-A persistence migration、runner/isolated PostgreSQL 证据、设计追溯与未关闭生产闭环门槛 | 用户确认 immutable run-as 撤权 fail-closed 并要求继续 Schedule admission 工作 |
| v0.2 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正 run-as mismatch fixture 的唯一键误判风险，改为精确验证 trigger 错误；将 Docker `psql` 改为容器内 loopback TCP；记录修正后的最终全量 runner 通过，早期失效/不完整证据不计为成功 | 提交前自审发现负例被唯一键遮蔽并复跑发现 Docker socket 连接问题 |
| v0.3 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 9F4C-B enable-time run-as reauthorization、纯角色 helper 1/1 链接测试与独立 target 证据；明确新增 ACL SQL fixture 和 per-trigger worker/admission 仍缺，Schedule execution 不开放 | 用户确认创建者固定为 run-as、每次触发重验且撤权 fail closed；完成 API enable gate 实装及最终验证 |
| v0.4 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将已通过的 9F4C-B role-policy helper test 纳入 Schedule 自动化 runner；同步 runner、registry、automation-design 与实施计划证据，明确完整 runner 未在本次复核重跑 | 提交前自审发现 focused role-policy test 尚未纳入 [P] runner |
| v0.5 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将只读 ACL runtime role 下 canonical PostgreSQL Project/Branch/Run active/revoked/paused 结果与最终完整 runner 证据纳入报告；明确授权读取未串行化并发撤权，仍将 API 全事务/竞态、worker/admission、consumer/BI 与目标 grants 列为未完成 | ACL SQL runner 修复后以 disposable PostgreSQL 18.6 完成最终验证 |
| v0.6 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 9F4C-C shared Project transaction lock、ACL mutation triggers 与三类 revoke wait/post-commit fail-closed 双连接证据；同步 SRS/BD/DD、Data Design、实施计划及自动化索引；保留完整 REST rollback、worker/admission、consumer/BI 与生产 grants 门 | 用户确认 run-as 创建者固定不变、每次触发复验权限并撤权 fail closed，完成授权/撤权竞态序列化实现 |
