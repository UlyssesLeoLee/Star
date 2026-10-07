# PHASE-9F4D-3A — Infrastructure Owner Schema Report

> **报告版本**：v0.1 · 2026-10-07
> **状态**：🟢 六表 PostgreSQL schema/trigger substrate 与 disposable PG18.6 gate 通过；生产 owner writer/consumer、provider 与 Schedule execution 仍开放并 fail closed
> **范围**：Infrastructure demand Inbox、Host+exact-cluster reservation bundle、receipt/lifecycle/Outbox facts、delivery state 与逐表 W/T 分类
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

记录 9F4D-3A 对 Infrastructure owner 持久化边界的首个物理 schema 切片。Run/Execution owner 仍只发布自己的版本化 demand Outbox；新 Infrastructure schema 只用于其独立 Inbox、Host+精确 K3s cluster 双层 capacity bundle、幂等 receipt、不可变生命周期/Outbox facts 和可清理的投递状态。

本阶段没有添加 production writer、consumer、迁移启动调用方、数据库角色 grants、Multipass Manager/provider 或 executor。验证只使用本机 loopback disposable PostgreSQL 18.6，启动 0 个 Multipass instance，未消耗当前测试阶段最多 2 个已登记实例的磁盘预算。

## §1 改动矩阵

| 区域 | 完成内容 | 状态与边界 |
|---|---|---|
| Migration | 新增 `2026-10-07-infrastructure-capacity-owner-schema.sql`，创建独立 `infrastructure` schema 下六表、索引、Work lifecycle trigger、Transaction WORM trigger、tenant RLS/FORCE RLS 与 PUBLIC revoke | 🟢 additive owner schema；未部署目标库、未自动授予 app 角色 |
| Work 数据 | `capacity_demand_inbox`、`capacity_reservation_bundle`、`capacity_outbox_delivery`；reservation bundle 单行固定 Host 和精确 cluster 两个不同 reservation ID 与 generation | 🟢 受限初态/迁移；terminal 必须是当前时间附近且 24–25 小时后才允许物理清理 |
| Transaction 数据 | `capacity_demand_receipt`、`capacity_reservation_event`、`capacity_owner_outbox` | 🟢 每张都有 UPDATE/DELETE row guard 和 TRUNCATE statement guard；delivery retry state 与事实分表 |
| W/T/M | 六张实体表逐张分类，3 Work + 3 Transaction + 0 Master，无混合分类 | 🟢 见 `docs/data-design/phase9f4d-infrastructure-owner-wtm.md`；历史 100-table baseline 未重写 |
| 可重放 runner | 新增 Python runner：显式 PostgreSQL bin，loopback-only temporary cluster，步骤超时和 file-backed logs/JSONL audit；仅在 stop 成功且临时路径仍安全时删除自有数据目录 | 🟢 运行结果可复查；没有 Multipass 或 Docker 依赖 |
| 文档同步 | requirements v5.87、basic design v5.84、Schedule DD v0.19、Infrastructure DD v0.15、W/T addendum v0.1、implementation plan v6.15、automation-design v2.15、registry v0.64 | 🟢 明确 schema substrate 与生产能力间的差异 |

## §2 验证摘要

| 门禁 | 命令/证据 | 结果 |
|---|---|---|
| Python syntax | `py -3 -m py_compile scripts/automation/phase9f4d_infrastructure_owner_schema.py` | 🟢 PASS |
| PostgreSQL version | `D:\PostgreSQL\18\bin\initdb.exe --version` | 🟢 PostgreSQL 18.6 |
| Disposable schema runner | `py -3 scripts/automation/phase9f4d_infrastructure_owner_schema.py --pg-bin D:\PostgreSQL\18\bin` | 🟢 PASS；runner 10/10 步骤，migration 连续应用两次；临时 server stop 成功并删除唯一临时数据目录 |
| Tenant isolation | `NOSUPERUSER NOBYPASSRLS` disposable role 对六张表逐表检查 tenant A 可见、tenant B 不可见，跨 tenant INSERT 被拒绝 | 🟢 PASS |
| Lifecycle/WORM | append-only UPDATE/DELETE/TRUNCATE 拒绝；测试角色显式获 TRUNCATE privilege 以验证 statement guard；Work 初态、有效/无效 transition、24–25 小时 retention、回填过期 timestamp、expiry 前 DELETE 负例 | 🟢 PASS |
| FORCE RLS | 查询 PostgreSQL catalog 统计 `infrastructure` 下 FORCE RLS 实体表 | 🟢 6/6 |
| VM/resource budget | runner 使用本机 PostgreSQL，代码路径不调用 Multipass；未运行任何 Multipass launch | 🟢 使用 0/2 测试实例 |
| Target database/app grants | 本阶段没有访问目标 DB 或改变任何持久生产角色权限 | ⚪ 未验证、保持开放 |

详细 stdout/stderr 与步骤结果位于 worktree `target/phase9f4d-infrastructure-owner-schema/`。此处 `psql` 直接执行 migration，不是应用启动迁移器的部署证明。`star-pg-adapter::apply_migrations` 当前没有本 migration 的调用方，且 target DB owner/runtime grants 未安装/审查。

## §3 已知缺口

1. **生产 migration provisioning 未接线**：该文件仍需进入正式数据库部署/迁移 owner 流程；隔离 runner 直接调用 `psql`。
2. **角色与权限未验收**：目标库 schema owner、运行角色最小 grants、`NOBYPASSRLS`、连接池 tenant context 和部署顺序均未确认。Disposable role 的 `GRANT ALL` 仅用于局部验证，不是生产 grant 建议。
3. **Infrastructure owner command 未实现**：尚无当前授权/profile/Run binding/capacity 事实复验、幂等 Inbox consumer、原子 dual-capacity admission writer 或 concurrent duplicate/conflict test。
4. **Owner event 链未实现**：无 production demand publisher、Inbox consumer、acceptance consumer、Outbox relay/retry、exactly-once claim；语义仍是事务 Outbox + 幂等消费。
5. **生命周期 worker 未实现**：无 lease expiry/reaper、24–25 小时到期清理、capacity reconciliation、process restart/drain 或 owner inventory repair worker。
6. **Provider 未实现**：无 Star Rust Host Infrastructure Manager、Multipass inventory adapter、cluster placement resolver、Manager operation receipt producer、guest/K3s readiness producer 或 remote-provider implementation。
7. **执行 sandbox 和 workspace bridge 未闭环**：没有独立 Agent Runtime sandbox、native Hook 生命周期联动、受控 Worktree transfer、取消/恢复/独立验证。
8. **BI/Benchmark/Loop 未接通**：无 capacity/worker/provider/Hook outcome 的 production event producer、BI/Benchmark read model 或 schedule loop execution evidence。
9. **目标系统与性能未验收**：没有 target DB/RLS/grants、桌面/VM RSS、CPU/IO、1–8 Agent fairness、磁盘满恢复、Linux/macOS/Windows provider compatibility 测量。
10. **不代表 Phase 9F4D 总体完成**：schema slice 通过不能使 Schedule、任务卡 CLI 或任何 Agent executor 进入可用状态；缺失 provider/sandbox/readiness 时继续 fail closed。

## §4 子代理失败接手清单

本阶段没有派发子代理；没有子代理 RPC 状态或未收集产物。

## §5 守门规则（本阶段适用 16 项）

1. Project → Cloud Branch → Engineering Run → 本地 Worktree 主树不变；Infrastructure Environment/owner 不新增导航层。
2. Multipass 仍是本地/CI Linux guest VM 首选路径，K3s 运行在 guest；不引入 Docker Desktop 前置条件。
3. 本测试阶段最多 2 条已登记 Multipass inventory 记录，停止/可恢复记录也计入；不得自动 purge。
4. 本阶段 PostgreSQL runner 使用 0 个 Multipass instance，不能为 schema gate 启动 VM。
5. Run/Execution owner 和 Infrastructure owner 只写自己的事务/表；严禁跨 owner 写表和分布式 DB transaction。
6. Run owner 自有 quota reservation 不等价于 Host 或 cluster reservation。
7. Capacity pair 必须指向当前 binding 精确选择的 Host/cluster ID 与 generation，两个 reservation IDs 必须不同。
8. Capacity snapshot 是候选，不能替代 Infrastructure owner 在 admission transaction 内重算和原子落账。
9. 六张物理表必须恰有一个 W/T/M 主分类；本阶段为 3 Work + 3 Transaction + 0 Master。
10. 全部六表 tenant RLS `ENABLE` + `FORCE`；目标运行角色不得是 superuser/BYPASSRLS，并须在真实 target grants gate 再验。
11. Work 记录从规定初态创建，只有 guard 允许的状态迁移才可写。
12. Work terminal timestamp 必须接近 DB 当前时刻，retention 需在 24–25 小时区间内；expiry 前不允许物理删除。
13. Transaction receipt/event/outbox 是 append-only，拒绝 UPDATE/DELETE/TRUNCATE；不放入密钥、源文件或未界定 provider 输出。
14. schema runner 的 production claim 只覆盖实际 PG18.6 disposable 测试，不扩展到 target DB、API、worker 或 service deployment。
15. temporary data directory 只有 server stop 被确认且 resolve path 仍在 runner 专用目录内才能删除；无法确认 stop 时保留现场。
16. provider readiness、独立 Agent sandbox、native Hook、workspace bridge、独立验证与结果回写未完成前，不得启动 TaskExecutionRun executor。

## §6 签字栏

| 角色 | 审核状态 | 边界说明 |
|---|---|---|
| 架构师 | 🟢 Mavis 接手审核 | 确认双 owner 与 Host/精确 cluster 边界 |
| SRE Lead | 🟡 Mavis 接手审核 | 只确认数据库 schema smoke；未作 target reliability/capacity/SLO 验收 |
| 平台 | 🟢 Mavis 接手审核 | PG18.6 隔离 runner 与 6/6 FORCE RLS 通过；migration deployment/grants/provider 未接通 |
| 评审主持 | 🟡 Mavis 接手审核 | 本地代码/文档自审；未取得独立外部 reviewer 结果 |
| PM | 🟢 Mavis 接手审核 | 只关闭 9F4D-3A schema substrate slice，不关闭总 Phase 或 Schedule Loop |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 9F4D-3A 报告，记录 6-table W/T schema、PG18.6 10/10 runner 结果、双 owner 与测试期 Multipass 2-instance 上限；保持 production migration/grants/writer/consumer/provider/sandbox/BI/Schedule execution 开放 | 用户要求开始完成剩余 phase，同时将测试阶段 Multipass instance 限制在 2 个 |
