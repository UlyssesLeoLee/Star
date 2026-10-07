# PHASE-9F4D-3D2-RESERVATION-EXPIRY-REPORT

> 日期：2026-10-07
> 状态：🟡 Infrastructure owner 的 paired admission reservation 过期原语已验证；生产扫描 worker、owner consumers、Multipass/K3s provider 与执行闭环仍未完成
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

实现一个 tenant-scoped、有界、可重入的 Infrastructure reservation expiry primitive。每个 Host + 精确 K3s cluster reservation pair 在单独的 owner transaction 中验证并过期，写出可供后续 owner 消费的事实；本阶段不启用 worker 或 Agent 执行。整体基础设施继续以 Multipass 管理多个 Linux guest、guest 内运行 K3s 为基线，不依赖 Docker Desktop。

测试主机的 Multipass 盘空间有限，测试阶段登记实例总数不得超过 2；Stopped/Deleted 记录也计入。所有本阶段的 Rust 与 PostgreSQL 验证使用 0 个 Multipass VM。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

| 范围 | 改动或证据 | 状态 |
|---|---|---|
| Rust owner primitive | `expire_due_reservations` 限定 tenant 与 1..64 批次；admission deadline 和 expiry 均以 PostgreSQL 时钟判定；每个 pair 使用排序去重的 tenant/physical Host/cluster locks 与独立 transaction；完整复核双账本。 | 🟢 实现 |
| Paired lifecycle | 仅一致且仍 held/已到期的 pair 才在同一 transaction 转为 expired；同时写 tenant lifecycle fact、数据库 trigger 写 global physical fact、`capacity_expired` 不可变 Outbox 与 pending delivery。 | 🟢 PostgreSQL 集成用例通过 |
| Schedule / Run ownership | Infrastructure 不直接修改 occurrence 或 TaskExecutionRun。Outbox 由 Run/Execution owner 后续消费；过期本身不触发 Schedule retry/resume。 | 🟡 已冻结边界；consumer 尚未实现 |
| Data classification | 重用已有 10 张表与 transition trigger；无 migration、无 W/T/M 数量变化。 | 🟢 对账 |
| Multipass/K3s | 不创建 VM，不操作 Multipass inventory；测试登记上限继续为 2 条，包含 Stopped/Deleted，失败关闭且不自动 purge。 | 🟢 策略保持 |
| 文档 | requirements v5.95 / basic design v5.92 / Schedule SRS v0.24 / Schedule BD v0.23 / Infrastructure DD v0.22 / W-T-M v0.6 / implementation plan v6.24 / automation design v2.21 / registry v0.68。 | 🟢 已同步 |

## §2 验证摘要（实际执行）

- `python scripts\automation\phase9f4d_infrastructure_contract.py --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98-x86_64-pc-windows-msvc\bin --target-dir E:\DevCache\s9f4f --output-dir E:\DevCache\phase9f4d3d2-contract-final3`：format、all-target check、targeted Clippy、package all-target tests 共 4/4 gates 通过；infrastructure crate 49/49 unit tests 通过。
- `python scripts\automation\phase9f4d_infrastructure_owner_schema.py --pg-bin D:\PostgreSQL\18\bin --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98-x86_64-pc-windows-msvc\bin --target-dir E:\DevCache\s9f4f --output-dir E:\DevCache\phase9f4d3d2-pg-final3`：21/21 runner steps 通过；10/10 Infrastructure 表启用 FORCE RLS；7/7 ignored SQLx 集成用例通过，其中新增用例验证 live lease 不会被回收、过期 pair 双侧原子转换、重复调用不重复记账。PostgreSQL stop gate 通过，runner 的临时数据与空临时目录已清理。
- `python -m py_compile scripts\automation\phase9f4d_infrastructure_contract.py scripts\automation\phase9f4d_infrastructure_owner_schema.py`：通过。
- `python scripts\automation\registry_check.py`：exit 0、0 errors、197 warnings；该检查遍历整个脚本库，warning 数量不用于本阶段通过判定。
- `git diff --check`：通过。
- CodeRabbit CLI 可执行文件解析到 Windows shim，但执行返回 `/bin/bash: line 1: /root/.local/bin/coderabbit: Permission denied`；版本与认证状态无法验证，因此没有把它报告为通过。已完成本地 diff 与代码边界自审。

## §3 已知缺口

1. `expire_due_reservations` 是 owner-store primitive；尚无生产 tenant sweep scheduler、跨 tenant 分片调度或运行时 leader/fencing。
2. Outbox delivery worker/retry 与 Run/Execution owner 的 `capacity_expired` Inbox consumer 尚未实现；Schedule occurrence、TaskExecutionRun 和 RunEvent 不会由本阶段自动推进。
3. Target DB migration provisioning、真实应用登录身份/role activation/reset、生产 ACL/current binding/profile resolver 与新鲜 Host/cluster observation producer 仍未验收。
4. Rust Host Infrastructure Manager、受限 Multipass adapter、K3s lifecycle/readiness producer、sandbox/workspace bridge、native Hook、独立 validation 与 Task Card CLI 生产执行仍未闭环；执行必须 fail closed。
5. 当前 reaper 只处理 admission hold，不代表 active Agent Runtime lease renewal，也不停止/删除 VM 或 K3s cluster。两类 lease/lifecycle 不得混用。
6. 多平台 provider capability、性能/RSS、并行压力与商业发行 SBOM 验收不在本阶段证据范围内。

## §4 子代理失败接手清单

本阶段未派发子代理；没有失败或遗留的子代理产出需要接手。

## §5 守门规则

1. 代码与实际数据库行为是事实来源；文档不得把未接线能力写成生产可用。
2. 过期 primitive 必须使用受信 tenant scope，并执行 tenant RLS。
3. 批次必须限制在 1..64，拒绝无效输入时不得触碰数据库。
4. Lease 扫描与转换复核统一使用 PostgreSQL `clock_timestamp()`。
5. 每个 reservation pair 使用独立 transaction；批次后续失败不回滚已完成的其他 pair。
6. tenant、物理 Host、精确 K3s cluster locks 必须全排序、去重后取得。
7. 持锁后必须重新读取并锁定 tenant/global rows，不依赖候选扫描快照。
8. 过期前必须比较两侧 reservation IDs、资源身份、generation、capacity vector、digest、observation、lease 与 state。
9. 任一 pair 不一致时回滚该 pair，禁止部分释放或猜测修复。
10. Infrastructure owner 只能在自身 transaction 写 Infrastructure 表，不得跨 owner 更新 Run/Schedule 数据。
11. Tenant lifecycle 与 global physical lifecycle facts 必须随双 Work 状态转换原子提交。
12. `capacity_expired` Outbox event 必须不可变、带稳定幂等键与原 correlation。
13. Outbox pending delivery row 必须与事件在同一 transaction 写入。
14. 已 renewed/changed 的候选必须幂等跳过；已过期 pair 重入不得重复事件。
15. Admission reservation expiry 不等同于 active Runtime lease renewal 或 VM/K3s 销毁。
16. 没有生产 scheduler、Outbox/Run consumer、provider/readiness 与 sandbox 证据时，Schedule 与 Task Card CLI 不得执行。
17. 测试阶段最多保留 2 条登记 Multipass instance（含 Stopped/Deleted）；本阶段创建 0 VM，且禁止自动 purge。

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
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 9F4D-3D2 reservation expiry 阶段报告；记录 paired-ledger 原子过期、数据库时钟、7/7 SQLx、10/10 FORCE RLS、双实例磁盘边界与 production gaps。 | Infrastructure owner admission reservation expiry primitive 完成实现与隔离验证。 |
