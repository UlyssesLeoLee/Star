# PHASE-9F4C-D20 Infrastructure Native Publisher PostgreSQL 验收报告

> **状态**：PostgreSQL 18.6 schema / owner-store SQLx 接受验证通过；真实 native publisher 调用、Rust 服务端 durable append、Rust 1.99.0 正式门禁仍未验收。SQLx 用例在迁移双次应用后的 disposable 数据库中直接验证 owner store，并提供 fixture receipt；它不调用 publisher 或真实传输。
>
> **分支 / 基线**：`codex/d20-pg-receipts`，基线 `61ee9c0f69313d45f6259cfd565df20c8fe688ef`。
>
> **范围**：`scripts/automation/phase9f4d_run_owner_consumer.py`、`crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs` 与本报告。适配租户绑定修复后的 route API 并启用显式 test-support feature；未修改 D20 migration 或生产 Rust publisher/provider。

## §0 目的

对 `infrastructure.capacity_outbox_transport_pin` 与 `infrastructure.capacity_native_append_receipt` 做真实、一次性 PostgreSQL 18.6 接受验证，检查双次 migration、tenant FORCE RLS、受限 owner grants、route conflict、exact replay、provider / wire digest、delivery attempt fence、WORM 和事务回滚。验证只在 worktree 内创建可丢弃的临时 cluster；不连接或修改本机 PostgreSQL service 的数据目录。

## §1 改动与完成矩阵

| 区域 | 改动 / 验收 | 状态 |
|---|---|---|
| `scripts/automation/phase9f4d_run_owner_consumer.py` | 新增 `--native-result-tests`；schema-only 模式下双次应用 D20 migration，再执行专用 SQLx target。函数权限拒绝断言改为校验 SQLSTATE `42501` 和被拒绝的函数名，避免依赖 PostgreSQL 本地化错误文案。Cypher 头同步 runner、migration、SQLx target 与拒绝断言调用关系。 | 已完成，Python 语法通过 |
| `crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs` | 新增被 runner 显式调用的 ignored SQLx 测试；只使用生产 owner store 和真实 PG18.6。由 trusted endpoint resolver fixture 生成 route，再直接测试 lease、wire pin、retry、route conflict、receipt 与事务回滚；fixture receipt 不代表已执行 native append。 | 已完成，1/1 实际执行通过 |
| `db/migrations/2026-10-14-infrastructure-native-result-delivery.sql` | 未改动；隔离 cluster 上连续应用两次。 | 两次均通过 |
| 本报告 | 记录验收范围、证据与缺口。 | 已完成 |

## §2 验证摘要

最终通过的命令：

```powershell
python scripts\automation\phase9f4d_run_owner_consumer.py `
  --pg-bin D:\PostgreSQL\18\bin `
  --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin `
  --cargo C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin\cargo.exe `
  --target-dir target\d20-cargo-isolated `
  --schema-only --native-result-tests --ignore-rust-version `
  --output-dir target\phase9f4d-run-owner-consumer\d20-pg-receipts-rebased-retry
```

结果：runner `result=passed`；**93/93 steps passed**；本地 PostgreSQL 版本为 **18.6**；D20 migration apply #1/#2 均通过；SQLx 测试 `1 passed, 0 failed`；PostgreSQL `pg_ctl stop` 通过。临时 cluster 绑定随机 `127.0.0.1` 端口、禁用 Unix socket，唯一 `.cache` 数据目录在成功停止后已删除。`multipass_instances_created=0`，`multipass_inventory_modified=false`。本轮只连接 runner 创建的 disposable cluster，未连接系统 PostgreSQL service 的数据目录。audit SHA-256：`9163FBFA29D8867D7E264B4BA6B600023D55C0A15F2E3671BC6915170CB6DA71`。

SQLx 用例实际覆盖：

- transient failure 后重试复用完全相同的 `message_id` 和已 pin wire bytes；测试将 fixture `exact_replay` receipt 交给 owner store，验证第二次 attempt 的持久化 fence。没有调用远端 append。
- 同一当前租约换用不同 endpoint identity 得到 `RouteConflict`；NATS claim 不会接管已经 native-pinned 的事件。
- 两个 tenant 的 route pin / receipt 在 owner role 下按 RLS 隔离；同 tenant 数据可见，跨 tenant 数据不可见。
- provider identity 或 wire SHA-256 不匹配时拒绝写入 receipt；过期 lease 被重新 claim 后，旧 attempt 无法完成且不会留下 receipt。
- 两张表启用并强制 RLS、各有 tenant `USING` / `WITH CHECK` 策略；owner role 为非 login、非 superuser、无 `BYPASSRLS`，只具备所需 SELECT/INSERT grants。
- owner role 的 UPDATE/DELETE 被 grants 拒绝；以测试管理员角色尝试 UPDATE/DELETE/TRUNCATE 时，WORM guards 拒绝操作。
- 注入 finalize fault 后，receipt insert 与 delivery completion 同事务回滚；删除 fault 后当前 attempt 可成功完成。

其他已通过的定向门禁：

- `cargo fmt --package infrastructure -- --check`
- `python -m py_compile scripts\automation\phase9f4d_run_owner_consumer.py`
- `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp`

最终 audit 与 SQLx 输出位于 `target/phase9f4d-run-owner-consumer/d20-pg-receipts-rebased-retry/`。它是 ignored 的本机运行证据，不纳入 commit。

### 已遇到并修复的 runner / 环境问题

1. 初次运行中 migration 双次应用通过，但旧权限断言因临时 PostgreSQL 的中文错误文本与 runner 的英文子串不匹配而误报失败。runner 已改为开启 psql verbose 输出，确认 SQLSTATE `42501` 且 stderr 指向目标函数；修复后 verify 与 acknowledge 两项拒绝探针均通过。
2. 使用共享 Cargo target 的复跑遇到 Windows linker `LNK1104`，无法打开测试 exe；切换至本 lane 独立 `target/d20-cargo-isolated` 后完整 runner 通过。
3. 首次 `cargo fmt --package infrastructure -- --check` 指出新测试文件格式差异；已格式化并在最终状态重跑 check 通过。
4. rebased SQLx test 首跑发现 PostgreSQL 因 receipt FK 优先拒绝不带 `CASCADE` 的 TRUNCATE，测试错误地期待 WORM trigger 文案。测试已改为 `TRUNCATE ... CASCADE`，并从 `pg_trigger.tgtype` 独立核对两个表都安装 statement-level TRUNCATE trigger；第二次完整 runner 93/93 通过。

## §3 已知缺口

- 本轮只验证真实 PostgreSQL owner tables 与生产 owner-store 事务路径；测试使用 resolver/profile fixture 和合成 receipt，不调用 publisher，也未连接真实 native server。因此不证明 native client/server durable append、网络/TLS、跨进程 exact replay、崩溃恢复或线上部署行为。
- 独立审查已发现的 P1 tenant-route mismatch 已由 `8506766e` 修复：route tenant/deployment/profile 来自 resolver-produced endpoint，publisher 不再接收独立 tenant 参数；当前 Infrastructure library 76/76 tests 通过。P2 stale native route pin mismatch 仍会被 claim 过滤，尚无 operator-visible blocked/mismatch 计数；生产装配前仍须补可观测的 profile revision recovery/quarantine 流程。
- publisher 仍 opt-in；生产 startup/Profile resolver 未接，执行保持 fail closed。此 lane 不证明可投入生产的 Infrastructure→Run 投递闭环。
- 本机仅发现 Rust 1.98.1，而 workspace MSRV 是 1.99.0。本次 Cargo 命令带 `--ignore-rust-version`，属于诊断性实际 PG18.6 SQLx 执行，不满足正式 1.99.0 工具链门禁。
- 未执行 NATS 或 Multipass；未执行 live native server、BI / Benchmark、性能 / 内存、容灾、TLS 或多实例测试。
- CodeRabbit CLI 审查未运行成功：Windows CLI wrapper 调用 `/root/.local/bin/coderabbit` 返回 Permission denied；已完成人工 diff/source review。此限制不等于 CodeRabbit clean review。
- CodeGraph MCP 表示当前 worktree 无可查询索引；按仓库规则未生成索引。Cypher headers 由 `cgg_guard.py --changed --stamp` 校验通过。

## §4 子代理失败接手清单

本 lane 未再派生子代理。接手并修复的失败包括本地化 SQLSTATE 断言、共享 Cargo target 链接争用、Rust 格式差异，以及 FK 截断优先级导致的 WORM 测试误报；最终 migration 与 SQLx 行为全部通过。CodeRabbit 权限阻断已在 §3 保留为审查缺口。

## §5 守门规则

1. 本 lane 仅负责指定 runner、D20 SQLx test 和独立报告。
2. 不改生产 publisher/provider，也不改变 production startup 接线。
3. D20 migration 不需要 schema 修正，因此保持原样。
4. PostgreSQL 必须是一次性 disposable cluster；不得连本机服务数据。
5. 使用随机 loopback 端口，并禁用 Unix socket。
6. 数据目录仅由本次 runner 在 worktree `.cache` 创建；启动前要求唯一新路径。
7. 集群成功停止后才删除本次目录；停止失败则保留目录并报错。
8. 不使用 Docker Desktop 或 Multipass。
9. migration 必须双次应用通过后才继续验收。
10. 每张结果表必须验证 RLS 与 FORCE RLS。
11. 每张结果表必须验证 tenant-scoped `USING` 与 `WITH CHECK` policy。
12. Infrastructure owner 必须保持受限角色，且不得获得 UPDATE / DELETE / TRUNCATE grants。
13. 验收 route pin / receipt 的插入 fencing、WORM 和跨 tenant 隔离。
14. 验收 route conflict、exact replay、provider / digest mismatch 与 attempt fencing。
15. receipt 和 delivery completion 必须在同一事务；故障时无半提交事实。
16. 合成 receipt / test-support route 不能作为 publisher 调用或 native server durable append 的证明。
17. 未使用 Rust 1.99.0 时必须标注 `--ignore-rust-version` 为诊断，且不得关闭生产执行门。

## §6 签字栏

| 角色 | 审阅状态 |
|---|---|
| 架构师 | 架构范围与 fail-closed 限制已由 Mavis 接手审核 |
| SRE Lead | 本 lane disposable PostgreSQL 生命周期和 cleanup 证据已记录 |
| 平台 | PostgreSQL schema、owner grants、RLS 与 WORM SQLx 证据已记录 |
| 评审主持 | CodeRabbit 因 CLI 权限不足未执行；人工自审已完成，独立代码审查仍待后续补充 |
| PM | store/database 阶段验收通过；不表示 D20 生产交付完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 SQLx 验收缩小为 owner-store/数据库事务证明，不再依赖 fake publisher；记录 P1 route tenant binding 修复、93/93 disposable PG18.6 runner、WORM TRUNCATE 断言修正和 P2 mismatch 可观测性缺口 | P1 安全复核后将 PG SQLx lane rebase 到安全 route API 并重跑 |
| v0.1 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D20 native-result route / receipt PG18.6 双次 migration 与 SQLx 验收、runner SQLSTATE 修复、isolated Cargo target 复跑及正式工具链/真实 native server 缺口。 | D20 两张 Infrastructure native result 表的数据库接受验证 lane |
