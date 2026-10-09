# PHASE-9F4C-D2 Run Infrastructure Binding 阶段报告

> 版本：v0.2（2026-10-10）
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 状态：🟡 Run-owned binding Master/schema slice 已快进集成到本地 `dev`；SQLx 隔离数据库往返/CAS/回滚已验证；生产 currentness/consumer/execution gates 仍未关闭

## §0 目的

建立一个归 Run/Execution owner 管理的版本化 Infrastructure Binding，作为后续 Schedule admission、Run reservation、Provider readiness 与 executor 的持久化前置来源。该阶段固定 canonical Engineering Run scope、精确 HostEnvironment/K3s cluster generation、Profile/Runtime revision 与 digest、namespace 和资源预算；它不证明资源仍有效，不授权 Run，也不启用任务执行。

## §1 改动矩阵

| 面向 | 交付 | 状态与边界 |
|---|---|---|
| Run binding Master | 新增 `multica.run_infrastructure_binding_revision`，以 SCD2 保存受限 V2 snapshot；canonical Engineering Run composite FK、current-row uniqueness、16 KiB 上限和一次性 `valid_to` close。 | 🟢 隔离 PostgreSQL 18.6 双次 migration 和生命周期断言通过；不代表目标 DB rollout。 |
| Lifecycle Transaction | 新增 append-only `multica.run_infrastructure_binding_event`；应用 Run role 只读，固定 `search_path=pg_catalog` 的 SECURITY DEFINER trigger 从 guarded Master insert/supersede 追加事实，直接 INSERT 被拒绝。 | 🟢 直接伪造事件、更新/删除/截断等负例被拦截；Security-definer 只用于静态、限定的 trigger insert。 |
| Rust owner adapter | 新增 `PostgresRunInfrastructureBindingStore`，写入前校验 V2 contract，按 expected revision CAS 关闭旧版本并插入 successor；读取时严格解码、核对 scope/id/revision 并重算 digest。 | 🟡 Rust 1.99.0 `run-owner-runtime --lib` 单元测试 12/12；本阶段未运行 Rust SQLx save/resolve PostgreSQL round-trip。 |
| Schema runner | 扩展 `phase9f4d_run_owner_consumer.py`：应用 canonical Engineering Run prerequisites、双次 D2 migration、运行 D2 W/T/M/RLS/WORM fixture；增加仅在 disposable DB 使用的 legacy Audit no-op prerequisite。 | 🟢 PostgreSQL 18.6 共 29/29 steps；server 停止且本 runner 自有数据目录已移除；0 Multipass VM、inventory 未变、无 NATS。 |
| 文档 | 同步总需求、基本设计、数据设计、Schedule SRS/BD/DD、实施计划、自动化设计和 registry；显式记录 Rust SQLx round-trip、target DB identity/grants 和后续 consumers 均未关闭。 | 🟢 历史版本行保留，仅追加版本；阶段报告在本文件。 |

## §2 验证摘要

| 检查 | 结果 | 证据与范围 |
|---|---|---|
| PostgreSQL 18.6 schema runner | 通过，29/29，0 failed | `python scripts/automation/phase9f4d_run_owner_consumer.py --pg-bin 'D:\PostgreSQL\18\bin' --schema-only --toolchain-bin 'E:\DevCache\rustup\toolchains\1.99.0-x86_64-pc-windows-msvc\bin'`; audit：`target/phase9f4d-run-owner-consumer/fb618326-7019-4fd8-913a-e2da986cedfd/audit.jsonl`。包括迁移双次应用、FORCE RLS、scope/grant、事件直写拒绝、revision lifecycle、WORM、tenant isolation 与既有 owner assertions。 |
| Rust SQLx binding integration | 通过，1/1（0.54s），runner 30/30 | `--schema-only --binding-tests`；无 superuser/BYPASSRLS 登录角色；真实存取、租户隔离、并发 CAS、回滚/重试和伪造 digest 拒绝；证据见本节 follow-up。 |
| Rust 1.99.0 package tests | 通过，12/12 | `cargo test --offline --locked --ignore-rust-version -p run-owner-runtime --lib -j 4`；library unit tests only，不包含 SQLx 数据库往返。 |
| Rust package rustfmt | 通过 | `cargo fmt --package run-owner-runtime -- --check`。 |
| Rust package all-target Clippy | 通过，`-D warnings` | `cargo clippy --offline --locked --ignore-rust-version -p run-owner-runtime --all-targets --no-deps -- -D warnings`。`domain-kms` 的既有 deprecated API warnings 仍存在于依赖编译输出，不属于此 package 的 warnings-as-errors pass。 |
| 更广的严格 Clippy | 未通过，阻断来自已有文件 | 未加 `--no-deps` 的严格检查在 `crates/star-context/src/actor.rs` 报 `unnecessary_map_or` 与 `field_reassign_with_default` 两项；本阶段未改写该不相关文件。 |
| Python runner | 通过 | `python -m py_compile scripts/automation/phase9f4d_run_owner_consumer.py`。 |
| CypherGraph Guardian | 通过 | 新 Rust header 已改为 canonical `@cypher … @endcypher`，symbol 使用 repo-relative ids；`--changed --stamp` 与随后 `--changed` 均通过。 |
| 文档/差异守门 | 通过 | `git diff --check` exit 0；`registry_check.py`：244 scripts、0 errors、197 warnings（现有未登记脚本警告）；Git 发出 registry 文件 CRLF→LF 提醒，不是 whitespace error。 |
| CodeRabbit | 未完成，不视为通过 | 已安装的 Windows shim 调用 `/root/.local/bin/coderabbit` 时返回 `/bin/bash: Permission denied`，exit 1；人工 self-review 不替代自动审查结果。 |
| 资源边界 | 通过 | runner 报告 `multipass_instances_created=0`、`multipass_inventory_modified=false`；无 NATS server；临时 PostgreSQL 已停止并清理。 |

SQLx follow-up (2026-10-10): D2 SQLx follow-up: Rust 1.99.0 real PostgreSQL 18.6 binding test passed 1/1; the disposable runner passed 30/30 steps. It uses a LOGIN role with NOINHERIT, NOSUPERUSER and NOBYPASSRLS, explicitly activates star_multica_run_owner, and proves save/resolve digest roundtrip, wrong-tenant NotFound, stale revision rejection, single-winner concurrent CAS, insert-failure rollback of Master closure and audit facts, successful retry, and forged-digest rejection. No NATS or Multipass instance was started. Audit: target/phase9f4d-run-owner-consumer/cf849f3d-c7e8-4ea1-b1bd-a813121088c5/audit.jsonl. Production target DB/login provisioning, current authorization and Infrastructure currentness, owner consumers, reservations, sandbox/readiness and execution remain open/fail-closed.

## §3 已知缺口

1. 隔离 SQLx 数据库验证已补齐（1/1，runner 30/30）；目标生产登录角色、canonical parent ingest 与环境 currentness 仍未验收，不能据此标为生产执行闭环。
2. 目标数据库版本、migration identity、应用登录角色到 `star_multica_run_owner` 的授权/role activation 与生产 grants 尚未确认。
3. Run 每次 Schedule trigger 的 `run_as` 权限复验、Schedule/Run owner Inbox/Outbox consumers、幂等/补偿以及原子 Run reservation 仍未连接。
4. Infrastructure owner 对当前 Provider Profile、HostEnvironment、精确 K3s cluster identity/generation 的事务内复验与 Host/cluster 双层容量 reservation 尚未接通；本 Master 仅固定所选 binding snapshot。
5. Multipass/K3s Manager、测试实例总数最多 2 的受控 bootstrap、guest/Agent sandbox、workspace bridge、readiness、Hook gates、取消/drain、BI/Benchmark、Schedule loop worker 与执行仍未验收，执行保持 fail-closed。
6. 更广的严格 Clippy 被 `star-context` 两项既存 lint 阻断；CodeRabbit 未运行完成。两项均不得写成通过。
7. disposable runner 临时提供的 `public.audit_trigger_func()` 是 no-op，只为满足旧 Worktree DDL；不证明生产 Audit trigger。无生产数据库、NATS 或 Multipass 环境被修改。

## §4 子代理失败接手清单

本集成切片未派发子代理；没有待接管或丢失的子代理产物。候选 worktree 仍保留供后续 owner consumer 阶段续作。

## §5 守门规则

| # | 守门要求 |
|---|---|
| 1 | `RunInfrastructureBindingSnapshotV2` 必须由 Run/Execution owner 作为 canonical Run scope 下的唯一当前 revision 管理；基础设施 catalog 继续归 Infrastructure owner。 |
| 2 | binding successor 必须维持 stable binding ID、单调 CAS revision，并在一个 Run-owner transaction 中关闭旧版本及插入新版本。 |
| 3 | Tenant FORCE RLS、受限 Run role 和 column-scoped `valid_to` 更新必须在目标数据库重验；基础 schema fixture 不等于生产 grants。 |
| 4 | 应用角色可读取事件，但不能直接 INSERT/UPDATE/DELETE/TRUNCATE；生命周期事件仅由 guarded Master trigger 追加。 |
| 5 | Rust resolver 必须 strict-deserialize、核对 canonical scope/id/revision 并重算 digest；隔离数据库往返已验证，生产部署与当前基础设施复验仍是 gate。 |
| 6 | Run binding snapshot 不等于 run-as 授权、当前 Provider/Host/K3s 事实、资源 reservation 或 readiness。 |
| 7 | Host reservation 与 exact cluster reservation 必须分别由 Infrastructure owner 在其自身事务办理；不跨 owner 共享写事务。 |
| 8 | Multipass 测试阶段 inventory 总记录最多 2，包含 Stopped/Deleted；不自动 purge，D2 schema/test 不启动 VM。 |
| 9 | target DB identity、Schedule/Run consumer、profile currentness、双 reservation、sandbox/readiness、BI/Benchmark 与 executor 任一缺失时，任务执行继续 fail-closed。 |
| 10 | 只快进本阶段审阅完成的候选提交；不导入别的分支大改动、不清理无关 worktree、不 push。 |
| 11 | SQLx 集成使用无 superuser/BYPASSRLS 的 disposable LOGIN，经显式 role activation 访问 Run-owned 表。 |
| 12 | 并发 CAS 必须只有一个 writer 成功，失败者不得追加 revision。 |
| 13 | successor INSERT 失败必须回滚旧 Master closure 与同事务生命周期事实，并允许正常重试。 |
| 14 | 错 tenant 不得读取 binding；伪造 persisted digest 必须被 Rust resolver 拒绝。 |
| 15 | `--binding-tests` 是 opt-in；与 `--schema-only` 组合不启动 NATS，临时资源只清理由该 runner 创建的实例。 |

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | Mavis 接手审核：Run binding 与 Infrastructure catalog 分属 owner；不跨 owner 直接写表。 |
| SRE Lead | Mavis 接手审核：PostgreSQL 18.6 disposable runner 29/29 并完成自有资源清理；目标 DB 与生产角色未验收。 |
| 平台 | Mavis 接手审核：Rust 1.99.0 focused tests、rustfmt、package Clippy 与 Cypher guard 的范围明确；更广 Clippy 两项既存错误已披露。 |
| 评审主持 | Mavis 接手审核：直接事件伪造风险已通过权限与 trigger 边界收紧；CodeRabbit 因 shim permission denied 未完成。 |
| PM | Mavis 接手审核：实现提交 `87929aa2` 已从候选 worktree fast-forward 到本地 `dev`，随后只同步阶段报告/计划；保留 `.codegraph` 三文件；未 push、未清理其他 worktree/分支；SQLx follow-up 已验证，随本次选择性集成交付；其余 9F4 生产闭环继续排在后续阶段。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Run binding Master/Transaction schema、触发器审计边界、Rust adapter、29/29 disposable PostgreSQL、Rust 1.99.0 package gates、已知缺口及 fail-closed 范围。 | Phase 9F4C-D2 集成前自审完成。 |
| v0.1.1 | 2026-10-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录实现提交 `87929aa2` 已快进合入本地 `dev`；保留 `.codegraph` 与其他分支/worktree，不 push，并维持 Rust SQLx round-trip 和生产运行门为未关闭。 | 完成用户授权的选择性本地 dev 集成。 |
| v0.2 | 2026-10-10 | Ulysses — Mavis 接手审核 | 新增真实 SQLx 存取、租户隔离、并发 CAS、回滚/重试及伪造 digest 拒绝验证；runner 30/30，保留生产门。 | D2 数据库验收补齐。 |
