# PHASE-9F4D-3D3-OUTBOX-DELIVERY-REPORT

> 日期：2026-10-07
> 状态：🟡 Infrastructure owner 的 Outbox delivery state machine 已验证；broker publisher、Run/Execution consumer、生产 worker 与执行闭环仍未完成
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

实现 Infrastructure owner 对不可变 `capacity_owner_outbox` 事实的有界 PostgreSQL delivery 状态机：并发领取、lease fencing、ack、retry 与 dead-letter。此阶段不发送 broker 消息，不消费 Run/Execution owner Inbox，不启动 Agent 或任务执行器。

Multipass 仍是本地/CI Linux guest VM 的管理 provider，K3s 可部署在一个或多个 guest 中；Docker Desktop 不是前置。由于测试主机磁盘有限，测试 inventory 总计最多保留 2 条 Multipass 实例记录，Stopped/Deleted 记录也计数。本阶段的 Rust 与 PostgreSQL 验证均使用 0 个 Multipass VM。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

| 范围 | 改动或证据 | 状态 |
|---|---|---|
| PostgreSQL delivery store | 新增 tenant-scoped `claim_due`、`mark_delivered` 与 `record_failure`；使用 Infrastructure owner role、数据库时钟和 `FOR UPDATE SKIP LOCKED`。 | 🟢 实现 |
| 并行与内存边界 | Claim 限制 1..64 条、lease 限制 1..300 秒；payload schema 上限 16 KiB，claim 返回原始 JSON 文本而不创建 JSON 对象图。 | 🟢 序列化 payload bytes 最多 1 MiB/批；heap/RSS 另行预算 |
| Lease fence 与失败路径 | 每次 claim/reclaim 单调增加 attempt；ack/retry 必须匹配 tenant、event、owner、attempt 和有效 lease。Transient 采用有界指数退避；permanent 或第 12 次失败 dead-letter；已耗尽的崩溃 lease 不会产生第 13 次发送。 | 🟢 实现 |
| 数据与所有权 | 复用现有 Work delivery row 和 Transaction Outbox；不新增迁移或 W/T/M 分类；raw error 不写入 delivery row，Infrastructure 不跨 owner 更新 Run/Schedule 数据。 | 🟢 对账 |
| 集成用例 | 覆盖并发唯一领取、过期 lease reclaim、旧 lease ack 拒绝、transient retry、permanent dead-letter 与最后一次 lease 崩溃后的终止处理。 | 🟢 PostgreSQL 集成通过 |
| 架构与文档 | 同步 requirements、basic design、Schedule SRS/BD/DD、Infrastructure DD、W/T/M、automation design、implementation plan、registry 和本报告；保留 Multipass 多 guest/K3s 设计及两条测试 inventory 上限。 | 🟢 已同步 |

## §2 验证摘要（实际执行）

- `scripts/automation/phase9f4d_infrastructure_contract.py`：rustfmt、all-target `cargo check`、warnings-as-errors Clippy、package all-target tests 共 4/4 gates 通过；`infrastructure` crate 49/49 unit tests 通过。第一次链接尝试因 MSVC LNK1104 无法打开 E 盘 target 下的测试 exe 失败，换用短路径 `C:\t\star9f4d3d3-final5` 后 4/4 通过。最终日志位于 `E:\DevCache\phase9f4d3d3-contract-final5`。
- `scripts/automation/phase9f4d_infrastructure_owner_schema.py`：21/21 runner steps 通过，10/10 Infrastructure 表启用 FORCE RLS，10/10 ignored SQLx cases 通过。临时 PostgreSQL 已停止，唯一临时数据目录已确认清理。日志位于 `E:\DevCache\phase9f4d3d3-pg-final2`。
- `python -m py_compile scripts/automation/phase9f4d_infrastructure_contract.py`：通过。
- `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp`：4 个源码文件检查通过。当前分支没有仓库级 `.codegraph/` 索引，也没有可用 graph MCP 工具。
- `python scripts/automation/registry_check.py`：exit 0，0 errors、197 warnings；检查覆盖整个脚本目录，warning 计数不作为本阶段验收项。本阶段 owner-schema runner 已登记并链接到本报告。
- `git diff --check`：通过。
- CodeRabbit CLI：本机 `coderabbit.cmd` shim 返回 exit 126，提示 `/root/.local/bin/coderabbit: Permission denied`；版本和认证状态不可用，因此没有将 CodeRabbit 标记为通过。已对当前 diff、迁移契约与状态机完成直接本地自审。
- 子代理：本轮未派发子代理。

## §3 已知缺口

1. 此状态机是 PostgreSQL store，不是轮询/调度 worker；生产 worker、leader/fence、drain 与可观测性仍未实现。
2. 没有 broker publisher，也没有 Run/Execution owner 的 Outbox consumer/Inbox；事件仍留在数据库中，不能据此推进 occurrence、TaskExecutionRun 或 RunEvent。
3. Target DB schema owner、应用 login/secret、生产 role membership 与 `SET LOCAL ROLE` 生命周期尚未验收。
4. 生产 run-as ACL、当前 binding/profile resolver、跨租户全局容量分配及 Host/cluster 实时 observation producer 尚未闭环。
5. Rust Host Infrastructure Manager、受限 Multipass adapter、指定 K3s cluster 生命周期与 aggregate readiness 仍未形成生产 provider 闭环。
6. VM/K3s Ready 不等于 Agent sandbox、Worktree bridge、原生 Hook、取消/恢复与独立验证通过；Task Card CLI、Schedule 和 Agent 执行继续 fail-closed。
7. 多 provider 性能/RSS、并发压力、跨平台实机与发行 SBOM 验收不在本阶段范围。

## §4 子代理失败接手清单

本阶段未派发子代理；没有失败或遗留的子代理产出需要接手。

## §5 守门规则

1. Infrastructure Outbox 是不可变 Transaction 事实；delivery 状态只写入独立 Work row。
2. 每个数据库事务必须设置受限 owner role 与 tenant scope，并受 FORCE RLS 约束。
3. Nil tenant/worker identity、非法 batch 或 lease 参数必须在打开事务前拒绝。
4. 单次 claim 不得超过 64 条，lease 不得超过 300 秒。
5. Claim 使用数据库时钟、稳定排序与 `SKIP LOCKED`；不得以应用时钟判定 due 或 lease expiry。
6. 每次领取或重领都必须递增 attempt fence。
7. Ack/failure 必须匹配 tenant、event、lease owner、attempt 和未过期 lease；旧 worker 必须 fail closed。
8. 最多 12 次尝试；第 12 次 worker 崩溃后，下次扫描直接 dead-letter，不发起第 13 次。
9. Transient retry 使用指数退避，首个间隔 1 秒、上限 3600 秒。
10. Permanent failure 直接 dead-letter；raw transport error 和 secret 不写入 delivery row。
11. 单 event 原始 JSON 文本不超过 16 KiB，单次 claim 的序列化 payload bytes 上界 1 MiB；这不等于 Rust heap/RSS 上限，未来 consumer 必须单独预算解码和并行度。
12. Terminal Work row 必须满足数据库的 24–25 小时保留 guard 后才可清理。
13. 本阶段不声称 Kafka、Fluvio、NATS 或其他 broker 已发布消息。
14. Infrastructure owner 不得跨 owner 修改 Run、Schedule、TaskExecutionRun 或 RunEvent。
15. 未验证 target DB 身份、生产 grants 与真实 owner worker 前，不得将本地 disposable PostgreSQL 结果标成生产部署。
16. 测试阶段最多登记 2 条 Multipass 实例（含 Stopped/Deleted），inventory 无效或达到上限时 fail closed，不自动 purge；本阶段创建 0 VM。
17. 未验证 broker/consumer、Host 与目标 cluster reservation、Multipass/K3s readiness、独立 Agent sandbox、native Hook 和独立验证前，Schedule/Agent/Task Card CLI 执行保持关闭。

## §6 签字栏（Mavis 审核）

| 角色 | 审核 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008） |
| SRE Lead | Mavis 接手审核 |
| 平台 | Mavis 接手审核 |
| 评审主持 | Mavis 接手审核 |
| PM | Mavis 接手审核 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 9F4D-3D3 阶段报告；记录 Outbox delivery state machine、Rust/PostgreSQL 验证、两条 Multipass 测试 inventory 上限、0 VM 与尚未开放的生产 owner/broker/execution gates。 | Infrastructure owner Outbox delivery primitive 实现并完成 disposable PostgreSQL 验证。 |
