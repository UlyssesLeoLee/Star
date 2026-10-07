# PHASE-9F4D-3C1-INFRASTRUCTURE-OWNER-INBOX-REPORT

> 日期：2026-10-07
> 状态：🟡 仅完成 Inbox receive adapter；admission 与执行仍 fail-closed
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

交付 Run/Execution owner demand 到 Infrastructure owner PostgreSQL Inbox 的接收切片，验证类型化需求、tenant RLS、幂等和终态回执重放。该阶段不等于基础设施准入或 TaskExecutionRun 执行闭环。

## §1 改动矩阵

| 域 | 改动 | 状态/边界 |
|---|---|---|
| Rust adapter | 新增 PostgresInfrastructureOwnerInbox::receive_demand；校验 V2 demand、外部传入的当前 binding digest、正数 Host generation、deadline；以事务级 owner role 与 tenant RLS 写入 Inbox | 完成 adapter slice；trusted resolver 尚未接线 |
| 幂等 | 区分新入队、同内容 pending replay、已存在 immutable terminal receipt；不同 payload 重用 event/occurrence key 时拒绝 | 完成 |
| PostgreSQL integration | disposable PostgreSQL 18.6 中实际跑 SQLx ignored test，覆盖首次写入、重复、payload 冲突、terminal receipt replay | 完成隔离验证 |
| Schedule/Runtime | 未写容量 reservation、terminal receipt、Infrastructure Outbox；未创建 consumer/worker/provider/executor | 明确保持关闭 |
| 文档 | 同步 requirements、basic/Schedule SRS-BD-DD、Infrastructure DD、Group DD、方向、实施计划、automation registry 与 phase report | 完成 |

## §2 验证摘要

- python scripts/automation/phase9f4d_infrastructure_contract.py --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin --target-dir C:\t\star9f4d-20261007 --output-dir C:\t\star9f4d-20261007\infrastructure-contract-gate-final3：format、check、Clippy 与 tests 全部通过；范围为 domain-infrastructure + infrastructure，Cargo tests 使用 -j 1。
- python scripts/automation/phase9f4d_infrastructure_owner_schema.py --pg-bin D:\PostgreSQL\18\bin --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin --target-dir C:\t\star9f4d-20261007 --output-dir C:\t\star9f4d-20261007\owner-inbox-postgres-gate-final：exit 0；schema/grants gates 与真实 SQLx Inbox 集成测试通过，临时 PostgreSQL 正常停止并清理自己的临时目录。
- CypherGraph Guardian：5 个源文件检查通过。没有启动 Multipass VM；测试阶段最多保留 2 条已登记实例（包括 Stopped/Deleted），本阶段使用 0 条。
- Shell rtk 当前不可用；Rust 工具链由显式 toolchain 路径提供。不得把未执行的 workspace-wide gate 记为通过。

## §3 已知缺口

1. CurrentInfrastructureBindingV2 由调用方提供；生产可信 resolver 与防止读取后 binding/generation 变化的串行化门未接入。
2. 本切片不验证 Run-as actor/Project/Run 权限，也不执行 Schedule occurrence admission。
3. Inbox 尚无生产 Outbox delivery、consumer lease/retry/reaper、capacity admission command、Host reservation、精确 K3s cluster reservation、terminal receipt producer 或 capacity bundle。
4. Migration provisioning、目标数据库应用角色 membership/SET LOCAL ROLE 池化行为、目标 DB RLS grants 仍未验收。
5. Rust Host Infrastructure Manager、受限 Multipass provider、K3s/CRI readiness producer、独立 Agent sandbox、Worktree bridge、BI 与取消/恢复闭环均未实现；Task Card CLI 与 Schedule 执行必须保持禁用。
6. 本阶段未做 RSS/并行压力或生产环境测试；不据此声称资源/商业可用目标已满足。

## §4 子代理失败接手清单

本阶段没有派发子代理；因此没有未接手的子代理产出或失败 lane。

## §5 守门规则

1. Multipass 只由可信 Rust Host Infrastructure Manager 使用受限 adapter 管理。
2. Docker Desktop 不作为 K3s 前置条件。
3. HostEnvironment、Multipass instance、K3s cluster 和 Kubernetes node 保持不同身份。
4. Run binding 固定 cluster ID 与 generation。
5. Placement 由版本化 Infrastructure Profile 决定，不默认单共享集群或每 Run 独占。
6. Project/Run 配额与 Host/cluster 物理容量分别核算。
7. Inbox 入队不等于 capacity accepted 或 reserved。
8. Inbox 与 owner-local 事务隔离；不跨 owner 写表或共享分布式事务。
9. 事务中只激活 star_infrastructure_owner 受限角色并设置 tenant RLS scope。
10. terminal receipts append-only；接收 adapter 只读回执。
11. event/occurrence 键的 payload 冲突必须 fail closed。
12. binding digest 不是授权事实。
13. resolver 缺失、binding stale 或 generation 不合法时不得开放调用路径。
14. 任一 Host/cluster capacity 未确认时不得启动 executor。
15. K3s API/node/CRI readiness 不能替代独立 Agent Runtime sandbox。
16. 测试期间 Multipass inventory 上限为 2，Stopped/Deleted 项也计数；不得自动 purge。
17. VM/K3s gate 不能替代 native Hook、取消恢复、独立验证或 BI evidence。
18. 本阶段不启动任何 Multipass VM，不能宣称 provider/商业产品闭环验收。

## §6 签字栏（Mavis 接手审核）

| 角色 | 审核 |
|---|---|
| 架构 | Mavis 接手审核：owner Inbox receive 与 admission 明确分层 |
| SRE Lead | Mavis 接手审核：只记录实际 PostgreSQL disposable runner 证据 |
| 平台 | Mavis 接手审核：受限 role/RLS，未开放 Host/provider 能力 |
| 评审主持 | Mavis 接手审核：幂等 replay/conflict 与 known gaps 对账 |
| PM | Mavis 接手审核：phase slice 可复核，后续工作保持开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初版记录 9F4D-3C1 adapter、PostgreSQL 集成证据、差距与后续阶段门 | owner Inbox receive adapter 完成 focused verification |
