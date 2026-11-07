# Phase 9F4J-6 PostgreSQL RLS and Grants Report v0.1

> **状态**：🟡 草案 v0.1；目标 PostgreSQL 验收 NOT_RUN
>
> **日期**：2026-11-07
>
> **基点 commit**：e22d92bcbefaf009e18d043ab2c8a762c150fc01
>
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（范围限于本地代码、验证记录和已披露缺口）

## 0. 报告目的

新增独立的 opt-in PostgreSQL integration test，核对 BI Task Event runtime grants、owner/runtime role 属性、直接表写权限、FORCE RLS、函数安全配置、拒绝 SET ROLE、租户错配拒绝，以及事务回滚。测试保持 #[ignore]，只允许连接到专用 disposable database。

## 1. 改动矩阵

| 文件 | 状态 | 内容 |
|---|---|---|
| crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs | 新建 | 独立 ignored 测试；先校验数据库名、loopback 地址和非默认端口，再读目录或执行事务探针 |
| docs/briefs/9F4J-6-PG-RLS-GRANTS-20261107.md | 更新 | 记录实现与迁移检查发现 |
| docs/reports/PHASE-9F4J-6-PG-RLS-GRANTS-20261107.md | 新建 | 本阶段报告 |
| docs/reports/WBS-DISPATCH-9F4J-6-PG-RLS-GRANTS-20261107.jsonl | 更新 | 记录实现、验证和数据库验收状态 |

保护中的 W8 Rust 源文件、迁移、Cargo manifest/lockfile 和 Group WBS 未改。

## 2. 验证摘要

- cargo +1.99.1 test -p bi-owner-runtime --tests --no-run -j 4：通过；编译了 package tests 与新 ignored integration target。Rust 报告的 4 条 async_fn_in_trait warning 来自未改动的 task_event_receiver.rs。
- rustfmt +1.99.1 --edition 2021 crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs：通过；仅格式化本 lane 新文件。
- CypherGraph Guardian --changed --stamp：通过；检查 1 个新增 Rust source file。
- cargo +1.99.1 fmt --manifest-path crates/bi-owner-runtime/Cargo.toml -- --check：未通过；另有既有差异 crates/bi-owner-runtime/src/task_event_quarantine.rs:65。该非注册文件保持原样。
- CodeRabbit review 未运行：本地 CLI 入口执行时报 /root/.local/bin/coderabbit Permission denied，Ubuntu WSL 中找不到 coderabbit；已完成人工自审，未外发 diff。
- STAR_BI_TASK_EVENT_TEST_DATABASE_URL：未设置。目标 PostgreSQL acceptance：**NOT_RUN**。未创建、启动或更改数据库/VM，也未执行 ignored 数据库测试。

## 3. 已知缺口

1. 迁移 2026-10-16-bi-task-event-projection.sql 为 bi.commit_task_event_v2 配置 SECURITY DEFINER、owner star_bi_owner、search_path=pg_catalog 和 row_security=on；迁移没有设置函数级 statement_timeout 或 lock_timeout。Rust adapter 在事务内设置 statement_timeout=10s、lock_timeout=5s。
2. 同一 BI schema 中的 trigger-only bi.reject_task_event_mutation() 在迁移中没有显式撤销默认 PUBLIC EXECUTE。按本任务“runtime 只能执行固定 commit function”的验收，目录 ACL 探针会把该额外权限报为失败；未在本 lane 修改迁移或函数。
3. 本次只有编译证据，没有目标 PostgreSQL 上的 grants、RLS、租户拒绝或回滚实测证据。数据库验收必须保持 NOT_RUN，直到显式提供受控 disposable DB。

## 4. 子代理失败接手清单

- 无子代理失败、无散落子代理产物；本阶段仅由当前注册 lane 修改并审查四个授权文件。
- 目标数据库实测待具备 STAR_BI_TASK_EVENT_TEST_DATABASE_URL 后另行执行，当前不作验收通过声明。

## 5. 守门规则

| 守门 | 状态 |
|---|---|
| 测试显式 #[ignore]，只接受名为 star_bi_task_event_test 的数据库 | ✅ |
| 连接必须是 loopback 且端口不是 5432 | ✅ |
| 数据库身份确认先于目录检查和事务探针 | ✅ |
| 测试覆盖 PUBLIC/其他函数 EXECUTE、角色锁定、无直接表变更权限、FORCE RLS、固定函数安全配置与 SET ROLE 拒绝 | ✅ 编译；数据库结果 NOT_RUN |
| 租户错配必须以 SQLSTATE 42501 拒绝；savepoint rollback 后相同 event 仍能首次提交 | ✅ 编译；数据库结果 NOT_RUN |
| 未创建数据库/VM，未输出连接串或凭据 | ✅ |
| 未改 W8 冲突源、迁移、manifest/lockfile、Group WBS；不 push、不集成 | ✅ |
| 0 unsafe；0 新依赖 | ✅ |

## 6. 签字栏

| # | 角色 | 审核人 | 日期 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 本地实现与范围审核通过；目标 PostgreSQL 验收 NOT_RUN |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 安全边界与未执行项如实记录；数据库验收仍待实测 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 迁移设置与权限缺口已记录；未改重叠迁移 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 自审完成；编译通过；格式检查另有未改文件差异 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 四个注册文件范围内完成；未 push、未集成 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建独立 ignored PG grants 测试，记录 package test compile 结果、NOT_RUN 数据库状态及 trigger function PUBLIC EXECUTE 缺口 | WBS-9F4J-6 brief 与登记范围 |
