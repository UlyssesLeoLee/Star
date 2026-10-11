# PHASE-9F4D-3B1 — Infrastructure Owner Runtime Grants Report

> **报告版本**：v0.1 · 2026-10-07
> **状态**：🟢 Restricted role/grant contract and disposable PostgreSQL gate passed; target DB identity/provisioning and production owner execution remain open and fail-closed
> **范围**：Infrastructure-owner NOLOGIN group role、逐表 W/T privilege matrix、分离的应用登录身份与可重放 disposable DB 验证
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

记录 9F4D-3B1 在 9F4D-3A 六表 Infrastructure-owner schema substrate 之上增加的最小运行权限 contract。生产应用登录身份和 credential 必须由部署系统单独管理；应用 membership 与 transaction-scoped `SET LOCAL ROLE` 还需在目标环境验证。此阶段没有创建生产 owner writer/consumer，也没有打开 Schedule 或 Agent executor。

本阶段使用 loopback-only disposable PostgreSQL 18.6；没有启动、创建或清理任何 Multipass VM。当前测试阶段最多登记 2 个 Multipass instance，Stopped/Deleted/可恢复记录也计数；该上限不因本阶段验证而增加。

## §1 改动矩阵

| 区域 | 完成内容 | 状态与边界 |
|---|---|---|
| Grant migration | 新增 `2026-10-08-infrastructure-owner-runtime-grants.sql`；幂等创建/收敛 `star_infrastructure_owner` 为 NOLOGIN、NOSUPERUSER、NOCREATEDB、NOCREATEROLE、NOREPLICATION、NOBYPASSRLS、NOINHERIT；撤回旧/PUBLIC grants 后只授目标权限 | 🟢 migration 在 disposable PostgreSQL 重复应用；尚无目标 DB migration caller |
| W/T 权限 | Schema 仅授 USAGE；3 张 Work 表授 SELECT/INSERT/UPDATE/DELETE；3 张 Transaction 表授 SELECT/INSERT；不授 CREATE、TRUNCATE、REFERENCES、TRIGGER | 🟢 全部逐表 catalog assertions 通过 |
| 身份分离 | NOLOGIN 权限组与部署管理的 app login/secret 分离；测试 probe 经 membership 与事务内 role activation 验证 | 🟡 真正 app login、secret provisioning、membership 与连接池 rollback/reset 尚未实现/验收 |
| WORM 负向探针 | 独立 overprivileged test-only role 用于证明 DB trigger 拒绝 Transaction UPDATE/DELETE/TRUNCATE | 🟢 仅 disposable fixture；禁止用于生产授权 |
| 自动化与文档 | 扩展 9F4D owner schema runner；更新需求/基本设计/Schedule DD/Infrastructure DD/Group DD/方向指引/实施计划/W-T/automation/registry 并新增本报告 | 🟢 9F4D-3B1 增量同步完成；两实例测试上限保留 |

## §2 验证摘要

复核本阶段 runner 产物 `target/phase9f4d-infrastructure-owner-grants-final/audit.jsonl`：PostgreSQL 18.6 disposable cluster 的 14/14 步均为 `passed`：

| 验证 | 结果 |
|---|---|
| `postgres-init` / `postgres-start` | 通过 |
| `migration-apply-1` / `migration-apply-2` | 通过 |
| `runtime-grants-migration-apply-1` / `runtime-grants-migration-apply-2` | 通过 |
| `owner-runtime-probe-role` / `owner-runtime-privilege-matrix` | 通过 |
| `runtime-role` / `runtime-fixture` | 通过 |
| `runtime-tenant-isolation` | 通过 |
| `immutable-and-retention-guards` | 通过 |
| `force-rls-count`（6/6） | 通过 |
| `postgres-stop` | 通过 |

这是 disposable DB contract evidence；不证明目标数据库已部署、真实应用身份配置正确、连接池 role reset 安全，或 owner API/消费事务已实现。此次文档同步没有重跑 Rust tests、Cargo gates 或 Multipass VM 测试。

## §3 已知缺口

1. 目标 DB 尚未应用 grants migration；生产 migration provisioning caller 未配置。
2. 真实 app login/secret、group membership、权限目录与数据库角色映射未验收。
3. `SET LOCAL ROLE` 的真实连接池成功/异常路径与 rollback/reset 尚未验收。
4. Infrastructure owner Inbox consumer、幂等 command、Host+精确 cluster 双 ledger 原子 reservation、Outbox consumer/reaper/reconciliation 均未实现。
5. Run/Project 当前权限与 profile/binding placement resolver、并发公平性及容量快照仍未生产验收。
6. Multipass Manager/provider、K3s lifecycle/readiness producer、Agent sandbox、Worktree workspace bridge、BI/Benchmark 联动和 Schedule executor 未完成；Schedule 执行保持 fail-closed。
7. 本阶段不启动 VM；测试 inventory 上限仍为 2 个注册实例，包含停止或可恢复项，并且不得自动 purge。

## §4 子代理失败接手清单

本阶段未派发子代理，无失败或遗留子代理产物；实现、证据核对与文档整合均在当前隔离 lane 完成。后续 owner writer/consumer 应各自明确文件 ownership、事务边界和交接证据；Run owner 与 Infrastructure owner 不得共享写事务。

## §5 守门规则

| # | 本阶段守门 |
|---|---|
| 1 | 仅记录可从当前 migration、runner、审计日志或设计文件复核的事实，不编造历史。 |
| 2 | 保留活动 worktree 与用户数据；此次仅在隔离 lane 修改。 |
| 3 | 不把 disposable role、probe 或 mock 标成生产身份/执行能力。 |
| 4 | 权限缺失、identity 未知或目标 DB 未部署时 fail closed。 |
| 5 | 测试阶段 Multipass inventory 最多 2 条记录，停止/可恢复记录也计数。 |
| 6 | inventory 无效、不可读或达到上限时拒绝创建，不自动删除或 purge。 |
| 7 | 所有 live test launch 继续使用 inventory 检查与启动共持有 host-local lock 的受控脚本。 |
| 8 | Infrastructure role 必须 NOLOGIN、NOSUPERUSER、NOBYPASSRLS、NOINHERIT。 |
| 9 | 应用 login/secret 与权限组分离，credential 不进入仓库、审计或日志。 |
| 10 | W/T grants 保持逐表最小权限；不授 schema CREATE 或 TRUNCATE/REFERENCES/TRIGGER。 |
| 11 | Transaction facts 只允许 SELECT/INSERT；数据库 WORM guards 继续拒绝变更与截断。 |
| 12 | test-only overprivileged WORM probe 不得加入部署清单或生产 membership。 |
| 13 | 每次应用迁移需保留 FORCE RLS 与 tenant scope；不能由角色属性推断授权通过。 |
| 14 | Run owner 与 Infrastructure owner 只通过版本化 Inbox/Outbox contract 异步协作，不跨 owner 写表或共享事务。 |
| 15 | 容量、Run binding、当前授权任一未知/过期/不匹配时，不接受 reservation 或启动 executor。 |
| 16 | Host/cluster reservations、provider readiness、独立 Agent sandbox、workspace bridge 和 Hook 门均满足前，不启用 Schedule execution。 |
| 17 | 实现状态逐项同步需求、基本设计、详细设计、计划与阶段报告；不得将本次 isolated gate 表述为生产闭环。 |

## §6 签字栏

| 角色 | 审核者 | 结论 | 范围 |
|---|---|---|---|
| 架构 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 🟢 | owner boundary、应用身份与生产 fail-closed 状态 |
| SRE Lead | Mavis 接手审核 | 🟢 | disposable PG 生命周期与无 VM/无自动 purge 边界；生产 rollout 仍开放 |
| 平台 | Mavis 接手审核 | 🟢 | migration、NOLOGIN role、逐表 grant matrix 与 test-only probe |
| 评审主持 | Mavis 接手审核 | 🟢 | evidence scope、已知缺口与文档版本链 |
| PM | Mavis 接手审核 | 🟢 | 测试阶段最多 2 个 Multipass registered instances 的约束保留 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 9F4D-3B1 restricted NOLOGIN grant group、W/T least-privilege matrix、14/14 disposable PostgreSQL 18.6 evidence、真实应用身份和 owner execution 缺口；保留两实例测试上限。 | Infrastructure-owner runtime grants migration 及其 isolated role/privilege gate 完成。 |
