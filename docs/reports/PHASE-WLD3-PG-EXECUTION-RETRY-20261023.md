# PHASE-WLD3-PG-EXECUTION-RETRY-20261023

**状态**: 本地隔离 PostgreSQL / Rust 验收通过；WLD3 整体仍 OPEN，Schedule execution 保持 fail-closed
**测试 HEAD**: 52872fd58e748c505f3d61b1011b8bedcecf3960；协调者记录的当前 dev 为 3fc49919。按集成边界本 lane 暂不 rebase/merge，集成前由协调者复核两者差异。
**范围**: occurrence Branch 快照 correction、Schedule run-as 授权 API 单测，以及本地 PostgreSQL/Rust 验收
**本轮环境**: Windows 本机 PostgreSQL 18.6 tools、Rust/Cargo 1.99.0、lane-local Cargo target；不使用 Multipass guest、Docker 或系统 PostgreSQL

## §0 目的

验证 immutable historical migration → forward correction → repeat replay、扁平 ScheduleTargetV1 Branch 快照、Run-as ACL/RLS 和 Schedule adapter 集成行为。本报告仅说明这些隔离的本地验收结果；不表示 Schedule worker、跨 owner 执行 Saga、生产部署或 WLD3 整体已验收。

## §1 先前诊断运行（历史记录）

| 项目 | 历史结果 | 说明 |
|---|---|---|
| 初始 checked-in PostgreSQL runner | pg_ctl start 曾失败，exit 1 | 当时定位到 Unix socket 路径长度问题；后续修改 runner 为使用独立短 socket 路径。 |
| 两次私有短路径诊断运行 | 每轮 31/32 | 唯一失败为 materialization 插入，SQLSTATE 23514；历史 migration 对应嵌套 JSON path，而 producer 存储扁平 ScheduleTargetV1。 |
| 早期 runner 执行 | 29/30 | embedded Rust ACL 测试曾失败；后续使用 Rust 1.99.0 新 target 的测试通过。 |
| 历史修正 follow-up | PostgreSQL 尚未执行 | 旧 2026-10-22 migration 被保留，新增 2026-10-23 correction；静态测试和编译先通过。 |
| 当前状态 | 由本报告 §1.1 的 35/35 run 取代 | 历史失败仍保留为诊断证据，不再代表最新结果。 |

## §1.1 最新本机 PostgreSQL 18.6 acceptance

完整入口脚本在 rustfmt fail-fast 阶段停止，未进入数据库步骤；随后直接调用同一 checked-in 脚本中的 run_isolated_postgres helper 完成数据库 acceptance。此操作只验证数据库 helper，不把完整 phase CLI 记为通过。

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
| 总计 | 35/35 steps passed | 原始 JSON result：.cache/phase9f3-schedule/wld3-postgres-isolated-20261023.log。 |

原始 step stdout/stderr 保存在 .cache/phase9f3-schedule/postgres-*.stdout.log 与 postgres-*.stderr.log。完整 phase CLI 输出保存在 .cache/phase9f3-schedule/wld3-pg-acceptance-20261023.log。上述 .cache 和日志仅作为本地验收证据保留，不纳入 lane commit。

## §2 验证摘要

| 命令 / 检查 | 结果 |
|---|---|
| rtk proxy python scripts/automation/phase9f3_schedule.py --cargo E:/DevCache/cargo/bin/cargo.exe --rustfmt E:/DevCache/cargo/bin/rustfmt.exe --postgres-bin-dir E:/DevCache/star-pg18.6-phase-infra2/unpacked/pgsql/bin | exit 1，fail-fast 于 schedule-rustfmt；报告的既有差异位于 crates/domain-automation/src/lib.rs 与 crates/domain-automation/src/schedule.rs，数据库阶段未由该完整入口启动。完整命令输出保留于上述 wld3-pg-acceptance log。 |
| phase9f3_schedule.py 中 run_isolated_postgres helper，使用 PostgreSQL 18.6 bin-dir、Rust 1.99.0 与 lane-local Cargo target | 35/35 steps passed；端口、UUID data directory、stop exit 和日志见 §1.1。 |
| cargo test --locked -p star-api-rest --lib schedule_rule_request_cannot_select_run_as_actor -j 1 | passed 1/1，Rust 1.99.0。 |
| cargo test --locked -p star-api-rest --lib task_execution_rules_do_not_grant_agent_role_schedule_authority -j 1 | passed 1/1，Rust 1.99.0。 |
| rtk proxy E:/DevCache/cargo/bin/cargo.exe check --locked -p star-pg-adapter --all-targets -j 4 | passed，exit 0；该命令使用 lane-local CARGO_TARGET_DIR。 |
| rtk proxy python -m unittest scripts.automation.test_schedule_occurrence_branch_snapshot -v | passed，3/3。 |
| rtk proxy python -m py_compile scripts/automation/phase9f3_schedule.py scripts/automation/test_schedule_occurrence_branch_snapshot.py | passed，exit 0。 |
| rtk proxy rustfmt --edition 2021 --check crates/star-pg-adapter/tests/schedule_postgres.rs | passed，exit 0。 |
| CypherGraph Guardian | 修订后执行；结果记录在本次提交前检查。 |
| rtk proxy git diff --check | 修订后执行；结果记录在本次提交前检查。 |

本机初始只读检查确认系统 PostgreSQL 5432 没有被本次 runner 使用；已有受保护监听端口 5555 / PID 7460 未触碰。测试数据库只监听 127.0.0.1:58997。没有执行 Multipass / K3s / Docker 命令，也没有改变 VM 生命周期。

## §3 已知缺口

1. 当前完整 phase CLI 在 schedule-rustfmt fail-fast，尚未获得整个脚本入口一键通过；lane-owned Rust integration test 文件的定向 rustfmt 已通过。
2. 35/35 仅证明本机隔离 PostgreSQL 中 migration、约束、RLS/ACL fixture 和 Schedule adapter tests 的行为；生产目标环境连接、权限/grant、部署排序、迁移回滚和线上约束定义仍未验证。
3. Schedule worker/materializer、admission consumer、Run/Execution owner 到 Infrastructure owner 的 demand/reservation Saga、Inbox/Outbox durable delivery 与 crash recovery 仍未实现或未验收。
4. Multipass/K3s 目标环境中的 RunInfrastructureBinding、Host/cluster reservation、K3s API/node/CRI readiness 和独立 Agent Runtime sandbox readiness 未覆盖。
5. Schedule trigger/retry/resume 的完整撤权、取消、进程重启、租约恢复和端到端 failure matrix 未覆盖。
6. 2026-10-22 migration 是不可变历史，correction migration 只替换精确约束名；部署时若约束定义或依赖关系与预期不符必须停止并调查，不能宽泛删除。
7. 没有启用 Schedule execution；WLD3 保持 OPEN。

## §4 子代理失败接手清单

本验收 lane 未派发子代理。WLD0 由协调者并行处理；没有读取、修改或清理 WLD0 文件与数据。

## §5 守门规则

- PostgreSQL 使用独立 UUID data directory 和随机 loopback 端口 58997；不访问、不停止、不改写系统 PostgreSQL 5432。
- 受保护监听 5555 / PID 7460 未复用、停止或修改。
- pg_ctl stop 实际 exit 0；只删除经 UUID 与 lane containment 核验、由本 runner 创建的临时 data directory。
- 历史 migration 保持原样；先运行旧 migration，再运行 exact-name forward correction，并重复相同顺序。
- 不伪造 Branch UUID、不回填 legacy rows、不弱化 CHECK。
- Rust baseline 为 1.99.0；没有以 1.98.x 或 1.99.1 结果替代本轮要求。
- 本地 helper 35/35 不冒充完整 CLI、生产目标环境、worker/Saga、Runtime sandbox 或 WLD3 全部验收。
- Schedule execution 继续 fail-closed。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | 本地 occurrence/ACL acceptance 范围明确；等待协调者集成审查，WLD3 整体仍 OPEN。 |
| SRE Lead | 本机 disposable PostgreSQL 18.6：35/35，stop 与 owner-directory cleanup 通过；生产目标环境未验收。 |
| 平台 | 两个 focused Schedule API authorization tests 各 1/1；star-pg-adapter all-targets check exit 0；5/5 Schedule PostgreSQL tests 通过。完整 phase CLI rustfmt gate 仍失败。 |
| 评审主持 | diff-check 与 CypherGraph Guardian 在本次提交前运行；结果按最终报告记录。 |
| PM | 当前结果不构成商业产品执行闭环；Schedule execution 保持关闭。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.0 编译、旧 runner 启动失败、短路径诊断运行、ACL 和 occurrence CHECK 失败及 cleanup 证据。 | WLD3 acceptance retry brief |
| v0.2 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录保留 10-22 历史 migration、新增 10-23 flattened Branch correction、静态/编译检查和待执行数据库验收。 | integration review of b702fddd |
| v0.3 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 35/35 本地 PostgreSQL 18.6 acceptance、两项 Schedule API 权限单测、adapter all-targets check、runner fail-fast 边界和仍开放的 WLD3 生产/worker/Saga gates。 | 本轮 WLD3 lane 验收 |
