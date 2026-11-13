# PHASE-WLD3-PG-EXECUTION-RETRY-20261023

**状态**: 本轮完整 `phase9f3_schedule.py` runner 已终止并通过，45/45 steps 均为 `passed` / exit 0；WLD3 整体仍 OPEN，Schedule execution 保持 fail-closed。
**测试 HEAD**: 本轮完整 runner 基于 `dev@182dba2a69b5b8c82d9b7215a86a0b800e85dc79`。先前 PostgreSQL helper acceptance 的记录 HEAD 为 `52872fd58e748c505f3d61b1011b8bedcecf3960`；历史格式门基线为 `74cb3d90eee02647873d2eb17ef91cc7b66a6da4`。按集成边界本 lane 暂不 rebase/merge，集成前由协调者复核目标差异。
**范围**: occurrence Branch 快照 correction、Schedule run-as 授权 API 单测，以及本地 PostgreSQL/Rust 验收
**本轮环境**: Windows 本机 PostgreSQL 18.6 tools、Rust/Cargo 1.99.0、worktree-local ignored Cargo target；启动前 D 盘可用空间 110.59 GB；不使用 Multipass guest、Docker 或系统 PostgreSQL

## §0 目的

验证 immutable historical migration → forward correction → repeat replay、扁平 ScheduleTargetV1 Branch 快照、Run-as ACL/RLS 和 Schedule adapter 集成行为。本报告仅说明这些隔离的本地验收结果；不表示 Schedule worker、跨 owner 执行 Saga、生产部署或 WLD3 整体已验收。

## §1 先前诊断运行（历史记录）

| 项目 | 历史结果 | 说明 |
|---|---|---|
| 初始 checked-in PostgreSQL runner | pg_ctl start 曾失败，exit 1 | 当时定位到 Unix socket 路径长度问题；后续修改 runner 为使用独立短 socket 路径。 |
| 两次私有短路径诊断运行 | 每轮 31/32 | 唯一失败为 materialization 插入，SQLSTATE 23514；历史 migration 对应嵌套 JSON path，而 producer 存储扁平 ScheduleTargetV1。 |
| 早期 runner 执行 | 29/30 | embedded Rust ACL 测试曾失败；后续使用 Rust 1.99.0 新 target 的测试通过。 |
| 历史修正 follow-up | PostgreSQL 尚未执行 | 旧 2026-10-22 migration 被保留，新增 2026-10-23 correction；静态测试和编译先通过。 |
| 当前状态 | 由本报告 §1.2 的完整 runner 45/45 取代 | 历史失败及前轮 35/35 helper 仍保留为诊断证据，不代表本轮完整入口结果。 |

## §1.1 前轮独立本机 PostgreSQL helper acceptance（历史）

前轮完整入口脚本因 Schedule rustfmt gate 未通过而停止，随后直接调用同一 checked-in 脚本中的 `run_isolated_postgres` helper 完成 35/35 数据库 acceptance。其后一次完整 runner 曾在 run-as API 测试的 fresh dependency 编译期间被协调者停止。以上均为历史诊断记录；本轮完整 runner 终态见 §1.2。

| 项目 | 结果 | 证据 |
|---|---|---|
| 临时数据库启动 | 通过，exit 0 | 随机 loopback 端口 58997；唯一 data directory UUID 为 ae840a34-d899-4f5e-b638-8ec3b12c41b4。 |
| migration 首次应用 | 全部通过 | 旧 2026-10-22 migration 在前，2026-10-23 correction 在后。 |
| migration 重放 | 全部通过 | 同一顺序再次应用；纠正 migration 的精确 DROP/ADD 可重复运行。 |
| occurrence Branch CHECK | 通过 | 扁平 target_snapshot.branch_id 接受有效 lowercase/uppercase UUID，拒绝 missing、null、empty、malformed 和 nil。 |
| Branchless Rule | 通过 | Rule 配置可不含远端 Branch 身份，run-as 身份约束仍生效。 |
| RLS / ACL / Outbox tenant isolation | 通过 | FORCE RLS catalog、runtime role、canonical directory ACL 与 Outbox tenant RLS steps 全部通过。 |
| run-as ACL Rust 测试 | 通过，1/1 | schedule_run_as_authorization_rechecks_canonical_directory_grants。 |
| Schedule PostgreSQL integration tests | 通过，5/5 | 包含 materialization 幂等与 tenant scope、retry、deadline、lease fencing、run-as 不可变性。 |
| stop / owned directory cleanup | 通过 | postgres-stop exit 0；UUID data directory 已删除；随机端口不再监听。 |
| 总计 | 35/35 helper steps passed | 前轮独立数据库 helper 结果；本报告 §1.2 记录本轮完整入口的 45/45。 |

### 前轮 Rust 格式 / 完整入口复验（历史）

| 检查 | 结果 |
|---|---|
| 五个 Schedule domain/adapter Rust 文件，rustfmt `--edition 2024 --check` | 通过；仅应用定向 rustfmt，未运行 `cargo fmt --all`。 |
| `crates/star-api-rest/src/group_api/schedule_rules.rs`，rustfmt `--edition 2021 --check` | 通过；该文件属于本轮经 MAGOS 扩展后的格式门范围。 |
| `cargo test --locked -p domain-automation --lib -j 4` | 通过，30/30；使用本 worktree 的 `.cache/phase9f3-schedule/cargo-target`。 |
| `cargo check --locked -p star-pg-adapter --all-targets -j 4` | 通过；出现 `domain-kms` 三条 `Nonce::from_slice` deprecated warning，不影响 exit 0。 |
| 完整 `phase9f3_schedule.py` runner，显式 Rust 1.99.0 toolchain Cargo 与 `RUSTC`，隔离 `CARGO_TARGET_DIR` | 五文件 rustfmt、API rustfmt、domain tests、adapter all-targets check、API all-targets check 均通过。进入 `schedule-rule-api-run-as-test` 后在独立 target 编译依赖；超过三分钟未完成且仍有编译输出，按协调者指示停止。该 step 无最终 exit/status，后续 run-as test、authorization-role test、integration compile、clippy 与 PostgreSQL steps 未由本轮完整入口执行。原始 step logs 保留在 `.cache/phase9f3-schedule/`，不提交。 |
| 工具链参数修正 | runner 将 shim 路径 `E:/DevCache/cargo/bin/cargo.exe` 解析为 `rustup.exe`，所以不能作为 Cargo 子命令直接运行。改传 `E:/DevCache/rustup/toolchains/1.99.0-x86_64-pc-windows-msvc/bin/cargo.exe`；首次 API check 还需显式设置同一 toolchain 的 `RUSTC`，否则 `autocfg` 找不到编译器进程。纠正后的完整 runner 已通过 API all-targets check。 |

前轮 step stdout/stderr 与 phase CLI 输出是历史证据路径；本轮完整结果和分步 logs 保存在 §1.2 指定的 `.cache/phase9f3-schedule/`。所有 `.cache` 日志与 Cargo target 均仅留作本地验收证据，不纳入 lane commit。

## §1.2 本轮完整 `phase9f3_schedule.py` runner

本轮在 lane worktree、`dev@182dba2a69b5b8c82d9b7215a86a0b800e85dc79` 上执行完整入口。该 worktree 起始没有可复用的 `.cache/phase9f3-schedule` target；未删除或复用其他 worktree 的 Cargo target。Rust/Cargo 1.99.0 与 PostgreSQL 18.6 可执行文件均使用报告指定的本机路径。第一次完整执行完成 fresh dependency compile（run-as test profile 编译耗时 6m21s），而后续 warm-cache capture rerun 将完整终端 JSON 保存在 ignored 本地证据目录。两次完整入口均返回 exit 0；保存的终端 JSON 可由 `ConvertFrom-Json` 解析，包含 45 个结果，全部 `passed` / exit 0。

```powershell
$env:RUSTC = "E:/DevCache/rustup/toolchains/1.99.0-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:CARGO_TARGET_DIR = "D:/orcaWork/Star/wld3-full-runner-20261023/.cache/phase9f3-schedule/cargo-target"
rtk proxy python scripts/automation/phase9f3_schedule.py --cargo E:/DevCache/rustup/toolchains/1.99.0-x86_64-pc-windows-msvc/bin/cargo.exe --rustfmt E:/DevCache/cargo/bin/rustfmt.exe --postgres-bin-dir E:/DevCache/star-pg18.6-phase-infra2/unpacked/pgsql/bin
```

完整 runner JSON：`.cache/phase9f3-schedule/wld3-full-runner-terminal-20261023.log`；外层 runner exit code：`.cache/phase9f3-schedule/wld3-full-runner-exit-20261023.txt`。每个 step 的 stdout/stderr 分别保存在同一 ignored 目录，以 step 名命名；日志和 Cargo target 不进入 lane commit。终端汇总为 `status=passed`、`exit_code=0`、`result_count=45`、`passed=45`、`failed=0`、`blocked=0`。

| Runner step | 终态 | Exit |
|---|---|---:|
| `schedule-rustfmt` | passed | 0 |
| `schedule-rule-api-rustfmt` | passed | 0 |
| `schedule-domain-tests` | passed；30/30 | 0 |
| `schedule-adapter-check` | passed | 0 |
| `schedule-rule-api-check` | passed | 0 |
| `schedule-rule-api-run-as-test` | passed；1/1 | 0 |
| `schedule-rule-api-authorization-role-test` | passed；1/1 | 0 |
| `schedule-integration-test-compile` | passed | 0 |
| `schedule-domain-clippy` | passed | 0 |
| `schedule-adapter-clippy` | passed | 0 |
| `postgres-init` | passed | 0 |
| `postgres-start` | passed | 0 |
| `postgres-directory-audit-prerequisite` | passed | 0 |
| `postgres-directory-migration-1` | passed | 0 |
| `postgres-directory-migration-2` | passed | 0 |
| `postgres-directory-migration-3` | passed | 0 |
| `postgres-directory-migration-4` | passed | 0 |
| `postgres-schedule-task-run-schema-fixture` | passed | 0 |
| `postgres-schedule-migration-first-1` | passed | 0 |
| `postgres-schedule-migration-first-2` | passed | 0 |
| `postgres-schedule-run-as-legacy-fixture` | passed | 0 |
| `postgres-schedule-migration-first-3` | passed | 0 |
| `postgres-schedule-run-as-legacy-backfill` | passed | 0 |
| `postgres-schedule-migration-first-4` | passed | 0 |
| `postgres-schedule-migration-first-5` | passed | 0 |
| `postgres-schedule-migration-first-6` | passed | 0 |
| `postgres-schedule-migration-first-7` | passed | 0 |
| `postgres-schedule-migration-first-8` | passed | 0 |
| `postgres-schedule-run-admission-link-fixture` | passed | 0 |
| `postgres-schedule-migration-repeat-1` | passed | 0 |
| `postgres-schedule-migration-repeat-2` | passed | 0 |
| `postgres-schedule-migration-repeat-3` | passed | 0 |
| `postgres-schedule-migration-repeat-4` | passed | 0 |
| `postgres-schedule-migration-repeat-5` | passed | 0 |
| `postgres-schedule-migration-repeat-6` | passed | 0 |
| `postgres-schedule-migration-repeat-7` | passed | 0 |
| `postgres-schedule-migration-repeat-8` | passed | 0 |
| `postgres-schedule-occurrence-branch-snapshot` | passed | 0 |
| `postgres-schedule-force-rls-catalog` | passed | 0 |
| `postgres-runtime-role` | passed | 0 |
| `postgres-schedule-run-as-directory-acl-fixture` | passed | 0 |
| `postgres-schedule-run-as-directory-acl-rust-test` | passed | 0 |
| `postgres-schedule-run-outbox-tenant-rls` | passed | 0 |
| `postgres-schedule-integration-tests` | passed；5/5 | 0 |
| `postgres-stop` | passed | 0 |

本轮隔离 PostgreSQL 使用随机 loopback 端口 `61018` 和 UUID data directory `postgres-abd258a9-f821-4f89-948a-7f960c944c3d`；`postgres-stop` exit 0。完成后实测该 UUID data directory 已不存在，端口 `61018` 与系统端口 `5432` 均无 listener；受保护端口 `5555` 仍为 PID `7460`，未复用或停止。Schedule domain 30/30、两项 API 定向测试各 1/1、PostgreSQL integration 5/5。非致命编译 warning 已保留在分步 stderr logs；所有 45 个 step 的实际结果均为 exit 0。

## §2 验证摘要

| 命令 / 检查 | 结果 |
|---|---|
| 本轮完整 `phase9f3_schedule.py` runner；Rust/Cargo 1.99.0、worktree-local `CARGO_TARGET_DIR`、本机 PostgreSQL 18.6 | `status=passed`、exit 0、45/45 steps `passed` / exit 0；JSON 与每步日志见 §1.2。 |
| 定向 rustfmt：五个 `RUST_FILES`，edition 2024；`schedule_rules.rs`，edition 2021 | 均通过 `--check`；范围只包括 runner 实际检查的六个 Rust 源文件。 |
| `rtk proxy E:/DevCache/rustup/toolchains/1.99.0-x86_64-pc-windows-msvc/bin/cargo.exe test --locked -p domain-automation --lib -j 4`（本轮 lane-local `CARGO_TARGET_DIR`） | 通过，30/30。 |
| `rtk proxy E:/DevCache/rustup/toolchains/1.99.0-x86_64-pc-windows-msvc/bin/cargo.exe check --locked -p star-pg-adapter --all-targets -j 4`（本轮 lane-local `CARGO_TARGET_DIR`） | 通过，exit 0；有三条非致命 deprecated warnings，见上表。 |
| 完整 runner 中的 PostgreSQL 18.6 isolated steps | 35/35 database steps passed；端口、UUID data directory、stop exit 和日志见 §1.2。 |
| `cargo test --locked -p star-api-rest --lib schedule_rule_request_cannot_select_run_as_actor -j 1` | 本轮完整 runner：passed 1/1，Rust 1.99.0。 |
| `cargo test --locked -p star-api-rest --lib task_execution_rules_do_not_grant_agent_role_schedule_authority -j 1` | 本轮完整 runner：passed 1/1，Rust 1.99.0。 |
| `rtk proxy E:/DevCache/cargo/bin/cargo.exe check --locked -p star-pg-adapter --all-targets -j 4` | 先前 v0.3 记录：passed，exit 0；本轮独立 toolchain Cargo 的 all-targets 检查也通过，见本表上方。 |
| `rtk proxy python -m unittest scripts.automation.test_schedule_occurrence_branch_snapshot -v` | 先前 v0.3 独立验收记录：passed，3/3；本轮未重跑。 |
| `rtk proxy python -m py_compile scripts/automation/phase9f3_schedule.py scripts/automation/test_schedule_occurrence_branch_snapshot.py` | 先前 v0.3 独立验收记录：passed，exit 0；本轮未重跑。 |
| `rtk proxy rustfmt --edition 2021 --check crates/star-pg-adapter/tests/schedule_postgres.rs` | 先前 v0.3 记录：passed；本轮该文件已通过 runner 的 edition-2024 gate。 |
| CypherGraph Guardian | 本轮只改报告与 brief，未改 Rust source；前一轮 6 个 Rust 文件的 header 检查结果仅作历史记录，本轮不刷新 header。 |
| `rtk proxy git diff --check` | 本轮 Markdown-only 改动提交前执行，exit 0。 |

本机只读检查确认系统 PostgreSQL 5432 没有被本次 runner 使用；已有受保护监听端口 5555 / PID 7460 未触碰。本轮临时数据库只监听 127.0.0.1:61018，停止后该端口不再监听。没有执行 Multipass / K3s / Docker 命令，也没有改变 VM 生命周期。

## §3 已知缺口

1. 完整本地 phase CLI 已通过 45/45 steps；该结果只覆盖 §1.2 列出的 Rust、migration、RLS/ACL fixture 与 adapter acceptance。生产目标环境连接、权限/grant、部署排序、迁移回滚和线上约束定义仍未验证。
2. Schedule worker/materializer、admission consumer、Run/Execution owner 到 Infrastructure owner 的 demand/reservation Saga、Inbox/Outbox durable delivery 与 crash recovery 仍未实现或未验收。
3. Multipass/K3s 目标环境中的 RunInfrastructureBinding、Host/cluster reservation、K3s API/node/CRI readiness 和独立 Agent Runtime sandbox readiness 未覆盖。
4. Schedule trigger/retry/resume 的完整撤权、取消、进程重启、租约恢复和端到端 failure matrix 未覆盖。
5. 2026-10-22 migration 是不可变历史，correction migration 只替换精确约束名；部署时若约束定义或依赖关系与预期不符必须停止并调查，不能宽泛删除。
6. 没有启用 Schedule execution；WLD3 保持 OPEN。

## §4 子代理失败接手清单

本验收 lane 未派发子代理。WLD0 由协调者并行处理；没有读取、修改或清理 WLD0 文件与数据。

## §5 守门规则

- PostgreSQL 使用独立 UUID data directory 和随机 loopback 端口 61018；不访问、不停止、不改写系统 PostgreSQL 5432。
- 受保护监听 5555 / PID 7460 未复用、停止或修改。
- pg_ctl stop 实际 exit 0；只删除经 UUID 与 lane containment 核验、由本 runner 创建的临时 data directory。
- 历史 migration 保持原样；先运行旧 migration，再运行 exact-name forward correction，并重复相同顺序。
- 不伪造 Branch UUID、不回填 legacy rows、不弱化 CHECK。
- Rust baseline 为 1.99.0；没有以 1.98.x 或 1.99.1 结果替代本轮要求。
- 完整本地 runner 45/45 不冒充生产目标环境、worker/Saga、Runtime sandbox 或 WLD3 全部验收。
- Schedule execution 继续 fail-closed。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | 本地完整 runner 45/45 steps 通过；生产 worker/Saga 与目标环境仍未验收，WLD3 整体仍 OPEN。 |
| SRE Lead | 本机 disposable PostgreSQL 18.6：完整 runner 中 35/35 database steps 通过；stop 与 UUID-owned directory cleanup 通过；生产目标环境未验收。 |
| 平台 | 六个 Rust 文件的 edition rustfmt gates、domain tests 30/30、API 定向 tests 2/2、integration tests 5/5、adapter/API checks、integration compile、两项 clippy 与 PostgreSQL steps 全部通过；完整 runner 为 45/45 / exit 0。 |
| 评审主持 | 本次只变更 Markdown 文档；提交前执行 `git diff --check`，Rust source 未改。 |
| PM | 当前结果不构成商业产品执行闭环；Schedule execution 保持关闭。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.0 编译、旧 runner 启动失败、短路径诊断运行、ACL 和 occurrence CHECK 失败及 cleanup 证据。 | WLD3 acceptance retry brief |
| v0.2 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录保留 10-22 历史 migration、新增 10-23 flattened Branch correction、静态/编译检查和待执行数据库验收。 | integration review of b702fddd |
| v0.3 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 35/35 本地 PostgreSQL 18.6 acceptance、两项 Schedule API 权限单测、adapter all-targets check、runner fail-fast 边界和仍开放的 WLD3 生产/worker/Saga gates。 | 本轮 WLD3 lane 验收 |
| v0.4 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 runner 实际检查的五个 edition-2024 Rust 文件和一个 edition-2021 API 文件的格式差异；记录格式门、domain/adapter/API 编译检查结果及完整 runner 在 API run-as test fresh-target 编译阶段被停止的边界；WLD3 仍 OPEN。 | `docs/briefs/WLD3-RUSTFMT-UNBLOCK-20261023.md` |
| v0.5 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 在 `dev@182dba2a` 使用 Rust/Cargo 1.99.0 与 PostgreSQL 18.6 完成完整 phase runner；45/45 steps passed / exit 0，记录完整终端 JSON、全部分步结果、61018 loopback PostgreSQL cleanup 与仍未关闭的生产 worker/Saga/K3s/Runtime gates；WLD3 保持 OPEN，execution fail-closed。 | `docs/briefs/WLD3-FULL-RUNNER-20261023.md` |
