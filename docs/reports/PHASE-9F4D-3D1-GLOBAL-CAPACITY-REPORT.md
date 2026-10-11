# PHASE-9F4D-3D1-GLOBAL-CAPACITY-REPORT

> 日期：2026-10-07
> 状态：🟡 跨租户物理容量账本与准入事务切片已验证；生产 provider、owner worker 和执行闭环仍 fail-closed
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

在同一 Infrastructure owner 内增加跨 tenant 的物理 Host 与精确 K3s cluster 容量台账，避免只按 tenant 汇总 reservation 时，同一物理资源被多个 tenant 并发超额接受。本阶段以全局 reservation、排序去重的租户/物理 advisory locks 和 tenant/global 双台账原子写入提供可验证的 admission primitive；不把该切片描述为生产 allocator 或启用任务执行。

测试主机磁盘空间有限，Multipass 测试期最多保留两条已登记 inventory 记录，Stopped/Deleted 记录也计入。该阶段没有创建 Multipass VM。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

| 域 | 改动与证据 | 状态/边界 |
|---|---|---|
| 全局物理容量 schema | 新增无 `tenant_id` 的 `capacity_physical_reservation` Work 与 append-only `capacity_physical_reservation_event` Transaction；两表启用 FORCE RLS 且仅授予受限 Infrastructure owner policy | 🟢 PostgreSQL 18.6 migration 可重复 apply；全量累计 10/10 Infrastructure 表 FORCE RLS |
| 并发锁与余量 | admission 同时取得 tenant Host/cluster 与全局 Host/cluster 锁；锁 key 排序、去重后逐一取得；held 且 lease 未过期的全局 reservation 计入物理资源余量 | 🟢 helper 单测覆盖 key 重复与排序；跨租户共享 Host、跨 Host 共享精确 cluster 的 SQLx 并发用例各为 1 Accepted + 1 Retry |
| 双账本与审计 | 一次 Infrastructure owner transaction 写 global reservation/event 和既有 tenant reservation/receipt/event/Outbox；物理 reservation 终态转换由数据库 trigger 同事务追加 actor-bound event | 🟢 schema、权限、隔离、WORM 与转换事件验证通过；没有跨 owner 写事务 |
| 自动化与 PostgreSQL | disposable loopback-only PostgreSQL runner 重复应用各迁移并执行 RLS/权限/租户隔离/WORM/retention/transition 检查，再执行 ignored SQLx owner cases | 🟢 21/21 runner steps、10/10 FORCE RLS tables、6/6 SQLx cases 通过；临时 PostgreSQL 正常停止并清理 |
| 文档同步 | requirements v5.94、basic design v5.91、Infrastructure DD v0.21、W/T/M v0.5、implementation plan v6.23、automation design v2.20、registry v0.67 | 🟢 已同步；保留生产执行 fail-closed 与两条 inventory 磁盘上限 |
| Multipass/K3s | 本阶段没有创建 Multipass VM 或部署 K3s；VM 启动仍通过带跨进程互斥锁的受控 bootstrap，测试 inventory 满额/无效时 fail-closed 且不自动 purge | 🔒 仅保留测试资源策略；未验证 Star 原生多集群 provider |

## §2 验证摘要（实际执行）

- PostgreSQL/Rust 自动化 runner：`python scripts/automation/phase9f4d_infrastructure_owner_schema.py --pg-bin D:\PostgreSQL\18\bin`。21/21 runner steps 通过；迁移重复 apply、owner-only FORCE RLS、tenant role 不可读写、WORM、retention 与状态转换事件均通过；10/10 表启用 FORCE RLS；6/6 ignored SQLx Inbox/admission cases 通过。runner 创建的 loopback-only 临时 PostgreSQL 已停止并清理，本阶段 VM 创建数为 0。
- SQLx 并发覆盖：共享物理 Host 与跨 Host 共享精确 K3s cluster 两种跨 tenant 竞争情形各自只接受一个 reservation，另一条返回 retry。
- Rust unit tests：`cargo test --locked -p infrastructure --lib -j 4` 通过，48/48；新增的锁 key 去重单测通过。命令使用 Rust 1.98 MSVC toolchain，目标目录为 `E:\DevCache\cargo\target`。
- Rust format：本阶段变更的 Rust 文件已定向 `rustfmt --check` 通过。`cargo fmt --all -- --check` 未通过，原因是仓库含有大量本阶段未修改的既有格式差异；未对全仓运行 formatter。
- Python：`python -m py_compile scripts/automation/phase9f4d_infrastructure_owner_schema.py` 通过。
- Diff：`git diff --check` 通过。
- CypherGraph Guardian：本阶段 3 个变更源文件的 `cgg_guard.py --changed --stamp` 检查通过。
- 当前 `AGENTS.md` 已规定测试阶段最多两条 Multipass inventory 记录，Stopped/Deleted 也计入且不自动 purge；requirements、design、automation runner 与本报告均保持相同边界。本阶段没有新增重复规则，也没有操作现有实例。

## §3 已知缺口

1. 已完成的是 Infrastructure owner admission/schema primitive；目标环境迁移部署、应用身份配置、生产 role membership 与连接池角色重置未验收。
2. 当前代码仍没有可信 current Run binding/profile resolver、生产容量 observation/provider，故测试容量快照不能替代生产事实 freshness。
3. Infrastructure Inbox consumer、owner Outbox delivery/retry、对账与 release/expiry reaper 尚未形成生产运行闭环；需要确保 tenant/global reservation 生命周期一致。
4. Rust Host Infrastructure Manager、受限 Multipass provider、K3s API/node/CRI readiness 与 VM 生命周期尚未实现及验收；直接 Multipass CLI 仍可绕开测试 bootstrap guard。
5. Agent sandbox、Worktree workspace bridge、资源预算/取消恢复、native Hook 与独立验证尚未闭环，Schedule 和 Task Card CLI 执行保持 fail-closed。
6. Run BI/Benchmark 联动、性能/RSS 压测、真实多 agent 并行压力、商业分发 SBOM 与部署验收不属于本阶段证据。

## §4 子代理失败接手清单

本阶段没有派发子代理；没有子代理产出、失败 lane 或接手任务。

## §5 守门规则（17 项）

1. 全局物理 Host/cluster reservation 属 Infrastructure owner；tenant Inbox、bundle、receipt、event 与 Outbox 继续保持各自 tenant scope。
2. 两张 global physical 表不得添加 `tenant_id` 或 Run payload，且都必须启用并 FORCE RLS。
3. Global physical policy 仅允许受限 `star_infrastructure_owner` 角色访问；tenant-scoped NOBYPASSRLS role 不可读取或写入。
4. 生产 owner role 必须为非 superuser、NOBYPASSRLS，并由受控 migration 配置最小权限。
5. 所有物理资源身份固定 Host/cluster UUID 与 generation；cluster 必须精确匹配 Run binding，不能仅按 namespace 推断。
6. Admission 同时获取 tenant Host/cluster 与 global Host/cluster 锁，先排序去重，再按统一顺序持有全部不同 key。
7. 所有竞争 admission 使用同一 global resource key 空间，确保跨 tenant 的 Host/cluster reservation 串行化。
8. 余量由 Infrastructure owner 在持锁事务内汇总该物理资源所有 held 且未过期的全局 reservation。
9. 接受 demand 时 tenant/global reservation 及对应事件、receipt、审计与 Outbox 必须在一个 Infrastructure owner transaction 内原子提交。
10. 不得通过跨 owner 直接写表或分布式双写来同步 Run/Execution owner 与 Infrastructure owner。
11. Global physical reservation 的资源身份与容量向量不可变；状态只允许 held 到 released/expired 的单向转换。
12. 每次 terminal state transition 必须在同一数据库事务中追加 actor-bound append-only event；缺失 actor 时 fail closed。
13. Transaction event 禁止 UPDATE/DELETE/TRUNCATE；Work terminal row 仅可在明确 retention 到期后删除。
14. Provider observation 过期、binding/profile identity 不匹配或容量未知时，不得接受 demand 或启动 executor。
15. PostgreSQL schema/RLS gate 只证明持久化边界，不证明 production worker、provider、K3s readiness 或 Schedule executor 可用。
16. 测试阶段最多保留两条 Multipass inventory 记录，Stopped/Deleted 也计入；inventory 无效或达到上限时拒绝创建且禁止自动 purge。
17. Multipass guest/K3s readiness 不能替代独立 Agent sandbox、Worktree bridge、native Hook、取消恢复、独立验证与资源预算验收；未闭环前 Schedule/CLI 保持 fail-closed。

## §6 签字栏（Mavis 接手审核）

| 角色 | 审核 |
|---|---|
| 架构 | Mavis 接手审核：全局 Host/精确 cluster reservation 与 tenant scope 分离；双账本只在 Infrastructure owner 事务中写入 |
| SRE Lead | Mavis 接手审核：仅记录实际 21/21 PostgreSQL runner、10/10 FORCE RLS、6/6 SQLx 结果；未声称 VM 或生产部署验收 |
| 平台 | Mavis 接手审核：排序去重锁、owner-only RLS 与 actor-bound transition event 边界已审阅 |
| 评审主持 | Mavis 接手审核：已明确生产 binding/provider/worker/release reaper 尚未闭环，执行保持禁用 |
| PM | Mavis 接手审核：需求/基本设计/详细设计与实施计划同步；两条 Multipass 测试 inventory 上限保留 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 9F4D-3D1 跨 tenant 物理容量账本、admission 双账本事务、owner-only RLS、并发与迁移验证证据，并记录生产缺口及两条 Multipass inventory 上限 | 完成全局 Host/cluster reservation admission slice 的实现与阶段文档同步 |
