# PHASE-9F4J.6 — BI Inbox PostgreSQL Persistence Test Report

> **状态**：🟡 P2 源码断言已修复并独立复核；本次 Rust 1.99.0 focused check/test/fmt 通过；数据库执行门未关闭，生产 consumer 保持 fail-closed
> **日期**：2026-10-19
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **审批者**：架构师 (Mavis 接手 agent per DEC-008)
> **范围**：只覆盖 BI owner Inbox PostgreSQL persistence test slice；不代表 Phase 9F4J、BI/Benchmark 或消息闭环完成

## §0 目的

记录 9F4J.6 的隔离 PostgreSQL 持久化测试源、准确验证边界和当前工具门状态。测试必须显式启用，并只允许使用命名为 `star_bi_task_event_test`、loopback、非默认端口的专用 disposable database。当前没有配置该数据库，因此报告只记录测试编译，不宣称数据库、target grants、生产 RLS 或 network commit-before-ACK 通过。

## §1 改动矩阵

| 文件 | 改动 | 结果 |
|---|---|---|
| `crates/bi-owner-runtime/src/task_event_inbox.rs` | 将 PostgreSQL test module 放在根模块的显式路径下；将共享生命周期 fixture 限定为 `pub(super)`；刷新测试模块 Cypher 关系。 | Rust 1.99.0 编译及变更文件格式检查通过。 |
| `crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs` | 保留 ignored-by-default、loopback/port/database-name gate 和角色/函数/RLS/DML 检查；补充 pool close/reconnect exact replay；按 P2 review 增加认证 `session_user` 不能 `SET ROLE star_bi_owner`、owner role 不能登录且不得拥有高权限属性的 fail-closed 断言，并更新 Cypher 关系头。 | Rust 1.99.0 all-targets check/test 与变更文件 rustfmt 通过；独立静态复核确认断言符合迁移合同。没有连接 PostgreSQL。 |
| `docs/implementation-plans/WBS-RUN-LIFECYCLE-DELTA-20261018.md` | 新增 9F4J.6 acceptance 证据矩阵与未关闭门禁。 | 明确区分源码/编译证据与数据库运行证据。 |
| `docs/briefs/WBS-9F4J6-REBASE-20261019.md` | 记录本次私有 lane rebase、核验命令和审计边界。 | 不触及本地 `dev` checkout 或其它 lane。 |
| 本报告 | 记录验证结果、限制和 fail-closed 状态。 | phase partial。 |

## §2 验证摘要

| 验证 | 实测结果 |
|---|---|
| Rust 1.99.0 baseline | 本次先用旧进程 PATH 运行默认 check 时解析到 Rust 1.98.1 并被 MSRV 拒绝；随后发现已安装的工具链通过 `E:\DevCache\cargo\bin` 暴露。将该目录前置 PATH 后，`cargo check --package bi-owner-runtime --all-targets -j 4` — PASS，0 errors、4 条既有 `async_fn_in_trait` warnings。没有安装或修改 toolchain。 |
| Rust 1.99.0 focused test | 前置 `E:\DevCache\cargo\bin` 后，`cargo test --package bi-owner-runtime --all-targets -j 4` — PASS：12 passed、0 failed、1 ignored。Ignored PostgreSQL 用例参与编译但未执行。 |
| 当前补充 check | Rust 1.98.1 `cargo check --package bi-owner-runtime --all-targets -j 4 --ignore-rust-version` — PASS，0 errors、4 条既有 `async_fn_in_trait` warnings 位于未修改的 `task_event_receiver.rs`。此结果不满足 MSRV gate。 |
| 当前补充 test | Rust 1.98.1 `cargo test --package bi-owner-runtime --all-targets -j 4 --ignore-rust-version` — PASS：12 passed、0 failed、1 ignored。Ignored PostgreSQL 用例参与编译但未执行；此结果不满足 MSRV gate。 |
| Changed-file formatting | Rust 1.99.0 `rustfmt --check --edition 2021 --config skip_children=true crates/bi-owner-runtime/src/task_event_inbox.rs crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs` — PASS。 |
| P2 修正独立复核 | 静态复核 — PASS：认证 principal 检查使用 `session_user`，owner role flags 与 migration 创建/ALTER 合同一致。没有运行数据库；Context7 请求失败，SQLx 连接主机解析通过锁定版本 0.8.6 的本地源码复核。 |
| Package-wide format / Clippy | 以下为既有 lane 记录，未在本次重放：package fmt 在未修改的 `src/task_event_quarantine.rs:65` 失败；Clippy `-D warnings` 在未修改的 `src/task_event_receiver.rs:253,265,279,293` 失败；advisory Clippy 曾以 0 errors、4 warnings 通过。 |
| Dedicated PostgreSQL | `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` 未配置（只检查是否存在，未读取或显示值）；未运行 ignored test、migration apply 或 grants/RLS live query。 |
| Network commit-before-ACK | 未运行；本测试只验证 BI owner database boundary，不构造 transport receipt/ACK 时序。 |
| Cypher Graph Guardian | 两个 Rust source files checked 为既有 lane 记录，本次未重跑；本次只改文档。CodeGraph 未索引且 graph MCP 不可用，按仓库指令未索引。 |
| Git whitespace | `git diff --cached --check` — PASS：最终 brief/WBS/report 3-file staged candidate 无 whitespace errors。 |

## §3 已知缺口

1. 没有专用 disposable PostgreSQL，因此 exact replay、changed-wire conflict、pool reconnect、function privilege、`SET ROLE` denial、owner role flags 和 FORCE RLS 断言仅完成源码审查/编译，尚未获得 live DB 证据。
2. Test 的 reconnect 检查证明目标 test database 上新连接能重放 receipt 时，才可作为 database persistence evidence；它不是数据库进程崩溃、WAL fsync、备份恢复或 PITR 证明。
3. 本测试不验证任何 transport ACK，更不能证明 receiver 在确认消息前已完成 PostgreSQL commit；network commit-before-ACK 继续是独立阻断门。
4. target DB migration、真实 app login membership/effective grants、生产 RLS、审计、恢复和数据保留仍未验收。
5. Package-wide fmt 和 `-D warnings` Clippy 仍受本 lane 外既存差异阻断；没有越界修改这些文件。
6. 原独立 review 记录的 P2（未排除连接身份切换到 owner role）已由上述断言在源码层修复并经只读复核；只有专用 PostgreSQL 运行结果可关闭实际权限断言门。
7. BI consumer、跨 owner delivery accepted settlement、recovery/retry/fan-out、BI/Benchmark 完整投影和生产启用仍未实现或未验收。此切片不得解锁任务执行或 Schedule/Run delivery。

## §4 子代理失败接手清单

本 lane 接手了 brief 指定的 inherited Inbox source/test draft，修正 Rust test module 路径并补充 pool close/reconnect 断言。原独立 review 提出 owner-role `SET` P2 后，协调 lane 仅在 PostgreSQL test source 增加两个 fail-closed assertion；只读复核确认修复符合 migration 合同。首次私有分支 rebase 到 dev `59fbdacbb106898fded81f5d0455b53d43dbaff3` 时，Git 跳过原提交 `09dbbd5b`（补丁已在 dev）。dev 后续前进到 `15fa97c2aaf2ed335e8d0f771ba90d5acfd83068`，本 lane 的文档提交已再次 rebase 到该 ref，未触及 dev checkout。候选只含 brief、WBS 和本报告。Rust 1.99.0 focused check/test 通过；数据库运行仍未执行。其他 lane 的平台/runtime 文件不属于本报告。

## §5 守门规则

| # | 守门项 |
|---:|---|
| 1 | PostgreSQL 测试默认 ignored；只有明确配置专用 disposable DB URL 且连接后确认精确 DB identity 才可执行。 |
| 2 | 不打印或复制 `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` 内容。 |
| 3 | `bi.commit_task_event_v2` 的调用签名、授权、Inbox/fact/incident 和 RLS 断言必须与迁移一致。 |
| 4 | exact replay 必须复用 receipt ID、commit time、wire digest 和 route digest。 |
| 5 | 相同身份不同 wire 必须返回 conflict，之后原 wire 仍复用初次 receipt。 |
| 6 | pool close/reconnect 仅证明 test DB 可重放已提交状态，不外推为 crash/WAL/PITR。 |
| 7 | Runtime 仅能执行固定 owner function；不得直接改写三张 Transaction 表。 |
| 8 | FORCE RLS 必须覆盖 Inbox、fact 与 integrity incident；disposable DB 结果不代替 target DB 门禁。 |
| 9 | PostgreSQL function commit receipt 不是 transport ACK；没有真实 receiver 时序证据就保持 delivery 未结算。 |
| 10 | 当前未运行 ignored PostgreSQL test，不得标注 PostgreSQL assertion passed。 |
| 11 | Rust 1.99.0 focused test 的 12 passed/1 ignored 不等于 workspace/all-targets gates 完成。 |
| 12 | Package fmt/Clippy 的既存 lane 外问题在报告中列明，不为通过本切片而修改不归属文件。 |
| 13 | 生产 BI consumer 和 Task/Schedule 执行保持 fail-closed，直到权限、传输、取消恢复与独立验证门全部满足。 |
| 14 | 只提交本 brief manifest 中文件；此 lane 获准 rebase 自己的私有分支；不得修改 dev、merge、push 或清理其它 checkout。 |
| 15 | 当前测试源要求认证 `session_user` 不能 `SET ROLE star_bi_owner`，且 owner role 为 NOLOGIN 并关闭 superuser/createdb/createrole/replication/BYPASSRLS/INHERIT；编译或静态复核不代替 live PostgreSQL 结果。 |

## §6 签字栏

| 角色 | 审阅 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — P2 源码缺口已修复并独立复核；live database gate 仍开放 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — disposable PostgreSQL 尚未运行 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — Rust 1.99.0 focused evidence 已记录；全 crate fmt/Clippy 有既存问题 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — P2 修正、Rust 1.99.0 focused gates 与最终三文档 diff whitespace check 已复核 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 不改变生产可用状态 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-18 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 9F4J.6 persistence test、pool reconnect durability assertion、Rust 1.99.0 focused gates 与明确未运行的 disposable DB/network 门禁；生产 consumer 保持 fail-closed。 | 9F4J.6 lane brief 完成与核验 |
| v0.2 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 处置独立 review 的 owner-role `SET` P2：增加认证 principal 与 owner-role 属性断言；Rust all-targets check/test、变更文件格式和只读独立复核通过；明确 PostgreSQL 仍未运行。 | P2 发现的修复与源码复核完成 |
| v0.3 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录私有分支 rebase 到 dev、重复代码提交被跳过，确认代码已在 dev；通过显式 PATH 找到已安装的 Rust 1.99.0 并重跑 focused check/test 与 rustfmt；更新候选文件和门禁状态。 | WBS-9F4J6-REBASE-20261019 brief 执行 |
| v0.4 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 dev 前进后将文档候选再次 rebase 到 `15fa97c2`；未修改 dev、其它 checkout 或预存 untracked brief。 | 与 root 协调确认最新 dev 基线 |
