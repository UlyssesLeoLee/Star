# PHASE-9F4D-3C2-INFRASTRUCTURE-OWNER-ADMISSION-REPORT

> 日期：2026-10-07
> 状态：🟡 Infrastructure owner tenant-scoped admission store slice 已验证；生产消费与执行仍 fail-closed
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

在 9F4D-3C1 已持久化的 Infrastructure demand Inbox 上，完成 owner-local Host + 精确 K3s cluster 双层容量准入事务。目标是验证同一资源范围内并发 admission 不超卖、成功写入双 reservation 和 owner 事实时保持原子、重放稳定、过期 reservation 不继续占用容量。本阶段不接通 production consumer、provider 或 TaskExecutionRun executor。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

| 域 | 改动与证据 | 状态/边界 |
|---|---|---|
| Rust owner store | 新增 `PostgresInfrastructureOwnerAdmission::admit_inbox`；按 tenant+Host 与 tenant+精确 cluster 计算 advisory key 并排序加锁，锁定重读 Inbox，校验 V2 payload、摘要、scope、generation、deadline 与终态回执 | 🟢 内部事务 primitive；无公开 API/consumer |
| 容量与 reservation | 在锁内从当前 tenant ledger 重算未过期 held reservation；容量可用时原子写入 distinct Host/cluster reservation、bundle、append-only event、terminal receipt、Inbox accepted、owner Outbox 和 delivery Work row | 🟢 disposable PostgreSQL 验证；仅当前 tenant 账本 |
| 重试与拒绝 | 容量未知/过期/不足时写有界退避 retry 且不写 reservation；永久 provider scope/generation 错误写终态拒绝；accepted/rejected receipt 按 identity/digest 重放 | 🟢 覆盖并发一胜一重试、重放、generation 拒绝与 lease 到期容量复用 |
| PostgreSQL 与权限 | 重复应用既有 schema/grants migration，验证 6 张 owner 表 FORCE RLS、tenant isolation、privilege matrix 与只读/写权限边界 | 🟢 PG 18.6 disposable loopback 实例；未触碰目标数据库 |
| 文档与自动化 | 更新 requirements、basic design、Schedule SRS/BD/DD、Infrastructure DD、方向、实施计划、automation design/registry，新增本阶段报告并运行 CypherGraph Guardian | 🟢 引用同步；保留未完成能力清单 |
| Multipass / execution | 本阶段未启动 Multipass VM；Schedule/Task Card CLI 未开放执行 | 🔒 继续 fail-closed；测试 inventory 上限为 2 个登记实例 |

## §2 验证摘要（实际执行）

- Rust 定向 runner：`py -3 scripts/automation/phase9f4d_infrastructure_contract.py --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin --target-dir E:\t\s9f4d3c2 --output-dir target\phase9f4d-infrastructure-contract-3c2-e`。format、`cargo check --all-targets`、warnings-as-errors Clippy 与 all-target tests 均通过；`domain-infrastructure` 27 tests、`infrastructure` 47 tests 通过，测试串行运行。Rust runner 中 4 个需 DB 的 SQLx cases 按设计标记 ignored，并由下方 PostgreSQL runner 实际执行。
- PostgreSQL runner：`py -3 scripts/automation/phase9f4d_infrastructure_owner_schema.py --pg-bin D:\PostgreSQL\18\bin --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin --target-dir E:\t\s9f4d3c2 --output-dir target\phase9f4d-owner-admission-3c2`。PostgreSQL 18.6 disposable loopback runner 共 15 steps 全部通过：schema/grants gates、6/6 FORCE RLS、tenant isolation、privilege/WORM/retention checks，以及 4/4 SQLx Inbox/admission tests；临时 PostgreSQL 正常停止并清理自身数据库目录。
- 合并到 `dev` 后重新运行同一 Rust runner：`E:\t\s9f4d3c2-postmerge` 独立 target 下 format/check/Clippy/tests 4 steps 全通过；重复 disposable PostgreSQL runner 15/15 steps 通过，4/4 SQLx cases 通过，临时服务正常停止。第一次在复用 target 的 post-merge 尝试遇到 `link.exe LNK1104`、无法打开测试 exe；没有 Rust source diagnostic。隔离到新 target 后完整复验成功，故以该成功 run 作为集成证据并保留本次环境异常记录。
- CypherGraph Guardian：`cgg_guard.py --changed --stamp`，5 个 Rust 源文件通过。
- `git diff --check`：通过。未运行 workspace-wide Cargo gates；本阶段使用两个受影响 crate 的定向 all-target gates。
- CodeRabbit CLI 检查未能启动：执行环境返回 `/root/.local/bin/coderabbit: Permission denied`；因此没有外发 diff，CodeRabbit review 不构成通过证据。本地代码和文档自查继续完成。
- 未启动 Multipass VM。测试阶段最多保留 2 个 registered inventory records，Stopped/Deleted 也计数；inventory 不可验证或达到上限时 launcher fail-closed，且不自动 purge。

## §3 已知缺口

1. Admission 接口接收 trusted provider snapshot 和 profile-derived request，但 trusted Project/Run binding、placement/profile resolver 与 provider observation producer 尚未接入。
2. 当前 owner ledger 与 advisory lock 均按 tenant 隔离，不能核算多个 tenant 对同一物理 Host/cluster 的竞争；跨租户共享必须等全局 allocator/ledger 后再启用。
3. Infrastructure Inbox/Outbox production consumer、delivery lease/retry/reaper、target DB runtime identity/grants provisioning 与部署仍未完成。
4. Rust Host Infrastructure Manager、受限 Multipass provider、实际 guest/K3s/CRI readiness、独立 Agent sandbox 和 Worktree workspace bridge 未接入。
5. Run owner 与 Infrastructure owner 的跨域状态联动、BI/Benchmark、Hook 联动、取消/恢复/独立验证以及 Schedule/Task Card CLI 执行闭环仍未完成；本切片不使 TaskExecutionRun 进入执行态。
6. 未执行 workspace-wide build/test、真实 Multipass/K3s、多 agent RSS/并行压力或商业发行验收；不可据此宣称完整产品闭环或性能达标。

## §4 子代理失败接手清单

本阶段没有派发子代理；因此没有子代理产出、失败 lane 或未接手任务。

## §5 守门规则（18 项）

1. Infrastructure owner 仅处理已持久化的 tenant-scoped Inbox demand。
2. 事务必须在受限 `star_infrastructure_owner` role 与 tenant RLS scope 下执行。
3. Host 与精确 cluster reservation 必须由同一个 owner transaction 决定。
4. tenant+Host 与 tenant+cluster advisory keys 必须按稳定次序加锁，降低竞争死锁风险。
5. 加锁后必须锁定并重读 immutable Inbox identity/payload，校验列投影、payload digest、binding digest 和 deadline。
6. 调用方/观测 snapshot 中的 active reservation totals 不可作为权威；容量必须从 owner ledger 重新聚合。
7. 只有 held 且 `lease_expires_at > transaction_now` 的 reservation 才占用容量。
8. accepted 时 Host ID 与 cluster ID 必须不同，bundle、receipt、event、Inbox terminal state、Outbox 与 delivery row 原子提交。
9. 任一成功路径写失败时事务整体回滚，不允许半个 reservation bundle。
10. stale/unknown/insufficient capacity 只允许有界 retry，不得在 retry 状态创建 reservation。
11. 永久 scope/generation mismatch 必须记录终态拒绝与 owner 事件。
12. terminal receipt 必须按 tenant、event/occurrence identity 与 digest 精确重放；冲突 fail-closed。
13. RLS 仍是 owner 事务的独立边界；六张 owner 表均启用并 FORCE RLS。
14. 任何 Owner 不得直接写另一个 Owner 的表；跨 Owner 只通过版本化 Outbox/Inbox。
15. Tenant ledger 不可用于跨租户共享资源准入；必须先实现全局 allocator/ledger。
16. 测试阶段最多 2 个 Multipass inventory 条目，包含 Stopped/Deleted；inventory 异常或达到上限时拒绝创建，绝不自动 purge。
17. 本阶段验证使用 disposable PostgreSQL 并启动 0 个 Multipass VM；不得把 VM/K3s 证据等同 Star provider/sandbox 验收。
18. 缺少 resolver、consumer、Manager、readiness、sandbox、Hook、取消恢复与独立验证前，Schedule/Task Card CLI execution 必须关闭。

## §6 签字栏（Mavis 接手审核）

| 角色 | 审核 |
|---|---|
| 架构 | Mavis 接手审核：owner admission 与 Run-owner admission 分属事务边界，跨租户容量明确禁止误用 |
| SRE Lead | Mavis 接手审核：只记录实际 Rust/PG gate；未声称 workspace/VM/生产验收 |
| 平台 | Mavis 接手审核：FORCE RLS、双资源锁与 reservation 原子性验证通过 |
| 评审主持 | Mavis 接手审核：并发、replay、generation reject、expired lease 用例与设计对齐 |
| PM | Mavis 接手审核：phase slice 可复核，production gates 保持开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初版记录 9F4D-3C2 tenant-scoped owner admission、实际 Rust/PG 证据、两实例硬盘上限和未完成生产门 | 定向 Rust 与 disposable PostgreSQL gates 全部完成 |
| v0.2 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 fast-forward 到 dev 后 Rust 与 PostgreSQL 复验结果；透明记录共享 target 上的单次 LNK1104 及隔离 target 重跑成功 | 集成后门禁完成 |
