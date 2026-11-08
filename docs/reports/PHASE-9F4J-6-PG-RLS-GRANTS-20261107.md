# Phase 9F4J-6 PostgreSQL RLS and Grants Report v0.4

> **状态**：🟡 本地实现与聚焦验证完成；目标 PostgreSQL 验收 NOT_RUN；等待协调者集成审查
>
> **日期**：2026-11-07
>
> **基点 commit**：632c5e50d5a6352b95c87baa1c26340f52866b31
>
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（范围限于本地代码、验证记录和已披露缺口）

## 0. 报告目的

新增独立的 opt-in PostgreSQL integration test，核对 BI Task Event 三函数 runtime allowlist、owner/runtime/login role 属性、直接表写权限、FORCE RLS、commit 函数安全配置、拒绝 SET ROLE、租户错配拒绝，以及事务回滚。测试保持 #[ignore]，只允许连接到专用 disposable database。

本次跟进增加一个前向迁移，撤销两个 trigger-only guard function 的 PUBLIC EXECUTE，并扩充 catalog test 直接检查这两项 ACL，防止仅依赖运行角色的间接权限探测遗漏 PUBLIC 授权。

runtime allowlist 使用迁移与现有 adapter 的完整签名：

| 函数 | 固定签名 |
|---|---|
| bi.commit_task_event_v2 | uuid,uuid,uuid,uuid,uuid,varchar,integer,varchar,integer,uuid,uuid,timestamptz,jsonb,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,bytea,bytea |
| bi.list_task_event_page_v1 | uuid,uuid,uuid,timestamptz,uuid,integer |
| bi.record_task_event_quarantine_v1 | uuid,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,varchar,varchar,uuid,bigint,uuid,bytea,integer,varchar |

## 1. 改动矩阵

| 文件 | 状态 | 内容 |
|---|---|---|
| db/migrations/2026-11-07-bi-task-event-trigger-exec-revoke.sql | 新增 | 只撤销 `bi.reject_task_event_mutation()` 与 `bi.reject_task_event_quarantine_mutation()` 对 PUBLIC 的 EXECUTE；不改 owner 或触发器 |
| crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs | 更新 | 保留三函数 runtime allowlist，并对两项 trigger-only 函数以 `aclexplode` 检查 PUBLIC ACL，按 `regprocedure` 报告意外授权 |
| docs/briefs/9F4J-6-PG-RLS-GRANTS-20261107.md | 更新 | 记录 follow-up 范围与验证限制 |
| docs/reports/PHASE-9F4J-6-PG-RLS-GRANTS-20261107.md | 更新 | 记录 follow-up 实现、验证和未关闭验收门 |
| docs/reports/WBS-DISPATCH-9F4J-6-PG-RLS-GRANTS-20261107.jsonl | 更新 | 记录 dispatcher、实现及验证结果 |

保护中的 W8 Rust 源文件、迁移、Cargo manifest/lockfile 和 Group WBS 未改。

## 2. 验证摘要

- `cargo +1.99.1 test -p bi-owner-runtime --tests --no-run -j 4`：通过，编译 library tests 与 ignored grants integration test；有 4 条既有 `async_fn_in_trait` warning，均来自未改动的 `task_event_receiver.rs`。
- `rustfmt +1.99.1 --edition 2021 --check crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs`：通过。
- 静态 SQL/test 断言：通过，确认恰有两条指定 PUBLIC EXECUTE 撤销语句，测试查询检查 `aclexplode` 的 PUBLIC grantee 并覆盖同两项函数身份。
- `git diff --check`：文档和实现更新后复跑通过。
- Cypher Graph Guardian `--changed --stamp`：通过，检查并刷新 1 个 Rust source header；该工具未将 `.sql` migration 纳入 source set，因此 migration header 另按 SQL 现有格式人工核对。
- 全 crate `cargo fmt --check` 未在本次 follow-up 重跑；此前记录的非本 lane 差异 `crates/bi-owner-runtime/src/task_event_quarantine.rs:65` 不在登记范围，保持原样。
- CodeRabbit review 未运行：此前本地 CLI 入口执行时报 `/root/.local/bin/coderabbit Permission denied`，Ubuntu WSL 中找不到 `coderabbit`；未外发 diff，协调者须独立复核本 lane。
- `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` 没有在本 lane 配置；目标 PostgreSQL acceptance：**NOT_RUN**。未创建、启动或更改数据库/VM，也未执行 ignored 数据库测试。

## 3. 已知缺口

1. runtime allowlist 固定为 bi.commit_task_event_v2、bi.list_task_event_page_v1 与 bi.record_task_event_quarantine_v1 的迁移签名。两项 reader/quarantine migrations 都显式 REVOKE PUBLIC 并 GRANT star_bi_runtime；该迁移事实不替代数据库 ACL 实测。
2. 新增的前向迁移显式撤销 `bi.reject_task_event_mutation()` 与 `bi.reject_task_event_quarantine_mutation()` 的 PUBLIC EXECUTE；test 另以 `aclexplode` 查询 grantee OID 0 并报告精确 `regprocedure` identity。迁移尚未在目标 PostgreSQL 执行，数据库 ACL 是否已收敛仍为 **NOT_RUN**。
3. v2 commit migration 配置 SECURITY DEFINER、owner star_bi_owner、search_path=pg_catalog 和 row_security=on；未配置函数级 statement_timeout 或 lock_timeout。Rust adapter 在事务内设置 statement_timeout=10s、lock_timeout=5s。
4. 本次只有编译证据，没有目标 PostgreSQL 上的 grants、RLS、租户拒绝或回滚实测证据。数据库验收必须保持 NOT_RUN，直到显式提供受控 disposable DB。

## 4. 子代理失败接手清单

- 本次 trigger ACL follow-up 在独立注册 lane 中完成；无子代理失败或散落产物。协调者集成审查仍待完成。
- 目标数据库实测待具备 STAR_BI_TASK_EVENT_TEST_DATABASE_URL 后另行执行，当前不作验收通过声明。

## 5. 守门规则

| 守门 | 状态 |
|---|---|
| 测试显式 #[ignore]，不自动运行 | ✅ |
| 连接仅读取 STAR_BI_TASK_EVENT_TEST_DATABASE_URL，预期库名固定为 star_bi_task_event_test | ✅ |
| 连接必须是 loopback 且端口不是 5432 | ✅ |
| current_database() 身份确认先于目录检查和事务探针 | ✅ |
| runtime allowlist 固定为 commit_task_event_v2、list_task_event_page_v1、record_task_event_quarantine_v1 的精确签名 | ✅ 编译；数据库结果 NOT_RUN |
| runtime role 必须能执行 allowlist 全部三函数 | ✅ 编译；数据库结果 NOT_RUN |
| 任一额外有效 BI EXECUTE 权限以具体 regprocedure identity 列出 | ✅ 编译；数据库结果 NOT_RUN |
| PUBLIC 对 allowlist 三函数均无 EXECUTE | ✅ 编译；数据库结果 NOT_RUN |
| PUBLIC 对两项 trigger-only guard function 均无 EXECUTE | ✅ 前向迁移与 catalog assertion 已编译/静态检查；数据库结果 NOT_RUN |
| current_user 与 session_user 均满足受限属性检查 | ✅ 编译；数据库结果 NOT_RUN |
| star_bi_runtime 与 star_bi_owner 均为锁定 NOLOGIN role | ✅ 编译；数据库结果 NOT_RUN |
| login principal 无法 SET ROLE 至 owner，目录检查与实际 SET ROLE 均覆盖 | ✅ 编译；数据库结果 NOT_RUN |
| current_user/session_user 对 Inbox、fact、incident、quarantine 均无直接写/删/截断权限 | ✅ 编译；数据库结果 NOT_RUN |
| 四张 BI Task Event 表均启用并 FORCE RLS | ✅ 编译；数据库结果 NOT_RUN |
| commit v2 检查 SECURITY DEFINER、owner、固定 search_path 与 row_security；仅验证迁移定义的值 | ✅ 编译；数据库结果 NOT_RUN |
| tenant mismatch 以 SQLSTATE 42501 拒绝；savepoint rollback 与最终事务 rollback 均覆盖 | ✅ 编译；数据库结果 NOT_RUN |
| 未创建数据库/VM、未输出连接串或凭据；PG acceptance 保持 NOT_RUN | ✅ |

## 6. 签字栏

| # | 角色 | 审核人 | 日期 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 本地实现符合 9F4J.6 权限边界；目标 PostgreSQL 验收 NOT_RUN，集成审查待协调者 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 迁移可前向收敛 PUBLIC EXECUTE；未声称 live grants/RLS 已验证 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 新 migration 与 catalog assertion 聚焦于两项 trigger-only 函数；未改 W8 migrations |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 本 lane 自审及 Rust 1.99.1 no-run/rustfmt/static checks 完成；协调者独立复核待完成 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-11-07 | 🟢 五个登记文件范围内完成；未集成、未 push；目标 PostgreSQL NOT_RUN |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建独立 ignored PG grants 测试，记录 package test compile 结果、NOT_RUN 数据库状态及 trigger function PUBLIC EXECUTE 缺口 | WBS-9F4J-6 brief 与登记范围 |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据独立复核改为三函数 allowlist，加入非 allowlist BI 函数 identity 诊断、session_user 检查及 quarantine 表 DML/RLS；扩充 §5 为 16 条守门项 | 固定 commit 6c1b31b0 的 REVISE 复核要求 |
| v0.3 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 复点 §5 表格确有 16 项，统一修正守门项计数；目标 PostgreSQL acceptance 仍为 NOT_RUN | 独立复审发现报告与 dispatch 计数不一致 |
| v0.4 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增前向 migration 撤销两项 trigger-only 函数的 PUBLIC EXECUTE；PG grants test 增加 ACL grantee 0 断言；记录 Rust 1.99.1 no-run/rustfmt/static 验证与目标 PostgreSQL NOT_RUN | 当前 dev 报告发现默认 PUBLIC EXECUTE 安全缺口 |
