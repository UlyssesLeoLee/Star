# PHASE-WLD3-PG-EXECUTION-RETRY-20261023

**状态**: 未通过，WLD-3 保持打开
**基线**: `dev` / `e3a59c60b727649500fb4a2d42ac6aa3d16516ea`
**范围**: 当前 dev 的 occurrence Branch 快照、Schedule run-as 授权 PostgreSQL/Rust 验收；只更新本验收 lane 的证据文件
**环境**: Multipass `informed-impala`，PostgreSQL 18.6，Rust/Cargo 1.99.0

## §0 目的

记录在当前 dev 基线上进行的 WLD-3 PostgreSQL occurrence 和 Schedule run-as 验收。该报告只覆盖本 phase 的临时数据库与 focused Rust gate；不表示 Schedule worker、生产运行、完整崩溃恢复或 WLD-3 整体已验收。

## §1 验收矩阵

| 项目 | 结果 | 证据 |
|---|---|---|
| Rust 1.99.0 编译 `schedule_postgres` 集成测试目标 | 通过，exit 0 | `cargo test --locked -p star-pg-adapter --test schedule_postgres --no-run -j 4` |
| 未修改的 disposable PostgreSQL runner 启动 | 失败，`initdb` exit 0，`pg_ctl start` exit 1 | 随机 loopback port 38265；`pg_ctl` 只报告无法启动，测试步骤失败关闭 |
| 长路径 private socket 诊断 | 失败 | 给 `pg_ctl` 加 `-k <data_dir>` 后，PostgreSQL 报 Unix socket 路径超过 107 bytes |
| private short-path + private socket 诊断运行 1 | 31/32 通过 | port 55845；唯一失败为 `schedule_postgres` 的 materialization 测试，exit 101 |
| private short-path + private socket 诊断运行 2 | 31/32 通过 | port 41225；相同唯一失败，exit 101 |
| 目录授权 Rust 测试 | 两次均通过，每次 1/1 | `schedule_run_as_authorization_rechecks_canonical_directory_grants`；155 filtered |
| occurrence/Run schema migrations、RLS/ACL SQL、Outbox 租户隔离 | 两次均通过 | harness 首次与重复迁移、约束和 ACL 步骤均 exit 0 |
| `schedule_postgres` 数据库测试 | 两次均 4/5 | 失败为 `materialization_is_idempotent_and_tenant_scoped`；SQLSTATE 23514，违反 `occurrence_target_snapshot_branch_uuid_check`，`schedule_postgres.rs:327:10` |
| cleanup 与进程检查 | 通过 | 两个唯一 PG data directory 均不存在；guest 中无匹配本轮 runner/test/PG 进程 |

私有 wrapper 只用于定位 PostgreSQL socket 启动问题，动态修改内存中的 `pg_ctl` 参数，没有改当前 checkout 的源文件。其通过的子步骤是诊断证据，不能视为未修改的 checked-in harness 通过。

## §2 验证摘要

- 目标编译使用 Multipass guest 中 Rust/Cargo 1.99.0，退出码 0。
- ACL focused test 在两个独立 disposable PG 运行里均为 `1 passed; 0 failed; 0 ignored; 155 filtered out`。
- 两个数据库运行均将现行 migrations 应用一次并重复应用；migration、RLS catalog、canonical directory ACL、tenant Outbox ACL 和授权测试通过。
- 两个 `schedule_postgres` 运行均为 4 passed / 1 failed。失败项在 occurrence materialization 时插入违反 Branch UUID CHECK 的快照行，数据库返回 SQLSTATE `23514`。因此 occurrence 端到端验收未通过。
- 未修改 runner 在随机端口 38265 启动失败。加入私有 socket 参数的长路径诊断仍因 Unix socket 107-byte 路径上限失败；将源码副本放在短路径 `/tmp/w3r1` 后可完成 PG harness 子步骤并抵达 Rust/DB 测试。
- 每次运行用独立 UUID data directory 与随机 loopback 端口 55845、41225；runner 已删除这些目录。后续 guest 进程查询未发现本轮 runner、测试可执行文件或 `/tmp/w3r1` 进程。
- 没有访问或触碰 5432；没有创建、停止、删除或 purge Multipass VM；没有修改目标 source/WBS 或旧 WLD3 worktree。

## §3 已知缺口

1. checked-in runner 缺少能在当前 guest 权限和路径限制下可靠启动 PostgreSQL 的私有 Unix socket 目录。未修改 runner 仍然失败。短路径 + `-k` wrapper 不是可接受的正式 runner 修复；需要另一个明确登记、审查的源码 lane 增加短且私有的 socket directory，并补充路径长度验收。
2. `materialization_is_idempotent_and_tenant_scoped` 测试创建的 occurrence 快照被数据库约束拒绝。需要在另一个登记的代码 lane 核对 fixture、Rust materializer 和 Branch UUID 约束的契约一致性，再运行原 harness；本 evidence-only lane 不改变这些源码。
3. 本 phase 未覆盖 Schedule worker/admission 消费者的真实执行、进程重启/崩溃恢复矩阵、生产数据库角色授权和 RLS 的部署验证、资源 reservation、Saga/outbox consumer 或 Runtime sandbox。WLD-3 overall remains open；所有 Schedule execution gates 继续 fail closed。
4. CodeGraph freshness/coverage 工具在本 lane 不可用；brief 中记录了本次已读取的 dev graph 结果及直接源文件核对，不能由此推断全仓无其他调用关系。

## §4 子代理失败接手清单

本 lane 未派发子代理。之前停止的 worker 留下的运行状态由本次 guest 日志、JSON 结果、process 清单复核；两轮验收实际终态和 exit code 已在报告与 JSONL 留档。dispatcher brief 创建记录中 `error` 为 null。

## §5 守门规则

- 只在已存在的 Multipass `informed-impala` guest 执行；未创建或清理 Multipass 实例。
- 仅运行 loopback 随机非 5432 端口的 disposable PostgreSQL；不连接系统 PostgreSQL、不停止既有数据库服务。
- 只清理由本次 runner 创建并以 UUID 标识的临时数据目录；核验两个目录均已删除，且无本轮进程残留。
- 私有 wrapper 的诊断通过不冒充未修改 runner 的验收通过；migration pass 不冒充 occurrence materialization pass。
- 数据库约束拒绝异常快照时保留失败结果，不跳过约束或把执行状态放宽。
- 本 lane 不修改生产源码、WBS、旧 WLD3 checkout 或共享设计文档；Schedule execution 保持禁用。
- Rust baseline 使用 1.99.0；没有以其他 Rust 版本的结果替代该 gate。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | 验收记录由 WLD3 worker 形成；Mavis 接手审核尚未执行 |
| SRE Lead | 未作验收签批 |
| 平台 | 未作验收签批 |
| 评审主持 | 未作验收签批 |
| PM | 未作验收签批 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.0 编译、未修改 runner 启动失败、两次私有 PG diagnostic runs、ACL 通过、occurrence materialization 失败及 cleanup 证据 | WLD3 acceptance retry brief |
