# PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT

## 0. 目的

为 Multipass 管理的多 Linux guest / K3s 架构增加可信 Infrastructure owner 身份目录持久化基础：分别登记 HostEnvironment、Multipass instance、K3s cluster、cluster-instance membership 和 Kubernetes Node，并以不可变生命周期事实记录注册、状态变化和退休。本报告只关闭 INFRA-2C1 schema / isolated PostgreSQL slice，不代表 Multipass provider、Run 执行或 Worktree-first 全部阶段完成。

## 1. 改动矩阵与验收

| 区域 | 改动 | 证据与边界 |
|---|---|---|
| Resource Master | 新增 `host_environment_catalog`、`multipass_instance_catalog`、`k3s_cluster_catalog`、`k3s_cluster_instance_membership`、`kubernetes_node_catalog` 五张全局 SCD2 Master 表。 | ID 和 generation 以复合外键固定 Host、guest、cluster、membership、node 归属；provider instance name 只是属性，不是所有权凭据。 |
| Lifecycle Transaction | 新增 append-only `resource_catalog_event`，只允许 registered/active、active↔draining 和 retired/retired 生命周期。 | 状态转移用资源类型、ID 和 generation 的事务级 advisory lock 串行化；退休代次才能一次性闭合 `valid_to`，retired 代次为终态。 |
| 一致性约束 | 限制一个 Multipass guest 同时最多属于一个 current cluster membership；同一精确 cluster generation 的 current Kubernetes node name 唯一；每个新 Master generation 必须与 `registered/active` Transaction 同事务提交。 | Deferred constraint trigger 在提交阶段拒绝孤儿 Master；双 cluster membership、重复 node name、错 Host、未知资源/状态负例按预期拒绝。 |
| 数据库安全 | 六张 catalog 表无 `tenant_id`，均有 ENABLE + FORCE RLS 与 `star_infrastructure_owner` owner-only policy。 | Master 仅 SELECT/INSERT/guarded UPDATE；Transaction 仅 SELECT/INSERT；物理删除、WORM mutation、未授权读取均拒绝。 |
| 自动化 | 新增 `phase_infra2c_resource_catalog.py` disposable PostgreSQL runner 和 `phase_infra2c_resource_catalog_sync.py` exact-anchor 文档同步脚本。 | Runner 自有 PostgreSQL 与日志/临时目录，不访问 Multipass CLI、不创建或清理 VM；文档 sync 第二次运行无改动。 |
| 文档 | 同步 requirements、basic design、Infrastructure DD、Group DD、W/T/M、implementation plan、automation design、registry 和本报告。 | 当前累积分类为 16 张表：4 W / 6 T / 6 M；把既有阶段快照与当前总表数区分。 |
| 磁盘边界 | 延续测试期最多 2 条已登记 Multipass instance（含 Stopped/Deleted）。 | 本阶段使用 0 VM，不读取、不改变、不自动 purge 用户的 Multipass inventory；两条上限仍是测试约束，不是产品拓扑上限。 |

## 2. 验证摘要

| 命令 / 门禁 | 结果 |
|---|---|
| PostgreSQL 18.6 disposable loopback runner | **29/29 steps 通过**。3A schema、3B1 runtime grants、3C3 Provider Profile、3D1 global physical ledger、INFRA-2C1 resource catalog 五个 migration 各应用两次。 |
| FORCE RLS 与权限 | **16/16 Infrastructure tables** 启用并强制 RLS；catalog owner-only policy、Master/Transaction 最小 grants 与 unprivileged read rejection 通过。 |
| 数据一致性与 WORM | 正向 catalog fixture、retired 后 SCD2 generation replacement 通过；无 `registered` 事实的 Master 提交、重复注册、未知 resource、cross-host membership、guest 绑定多个 cluster、cluster 内 duplicate current node name、Master update/delete、Transaction update/truncate 均按预期拒绝。 |
| Runner 清理 | PostgreSQL stop 通过；只移除 runner 自己唯一命名的数据目录，最终 `postgres-*` 临时目录数量为 0。Runner audit 在 `target/phase-infra2c-resource-catalog/audit.jsonl`。 |
| Python 语法 | `python -m py_compile` 覆盖两个新 automation scripts；最终结果记录于提交前验证。 |
| 文档同步 | `phase_infra2c_resource_catalog_sync.py` 已重跑；最近一轮有变更的重跑之后再次运行 `changed=0 files`，确认 no-op。 |
| Rust / Cargo | 本切片没有 `.rs` 改动，因此未运行 Cargo。Rust 1.99.0 继续作为工作区工具链基线；本阶段不据此宣称 Rust provider、桌面 RSS 或性能已验证。 |
| Multipass / VM | 未调用 Multipass CLI，VM 创建数为 0，用户 inventory 未改变。 |
| 本地工具限制 | `rtk` 不在此环境 PATH；Shell 命令由 PowerShell 直接调用并记录。`D:\Star` 根工作区的既有修改没有触碰。 |

## 3. 已知缺口

- 未部署到目标数据库；production migration owner、runtime login/membership、secret 和连接池 role activation/reset 均未验收。
- 目前没有 Rust catalog owner writer、受信 Multipass inventory reconciliation 或 Host/cluster/Node 注册命令；现有只读 CLI observation 和 provider name 不构成 owner registration。
- 未实现 current Project/Run authorization 与 `RunInfrastructureBinding` resolver、global capacity admission、trusted Manager IPC/operation receipt、Multipass lifecycle 或 provider profile resolver。
- VM state、K3s API/node Ready、CRI、独立 Agent Runtime sandbox 和 Worktree guest bridge 仍须独立生成、验证并绑定精确 generation；目录存在不表示资源可用。
- Run BI/Benchmark、Schedule worker 和任务卡 CLI 没有消费本目录；执行保持 fail-closed。
- 本阶段未运行 Multipass live test、K3s、桌面 RSS/CPU/IO、并行 p95、workspace Cargo gates 或生产故障恢复验证。

## 4. 子代理失败接手清单

- 本阶段由 root agent 在 `codex/infra-2c-host-inventory` 隔离 worktree 完成，没有派发子代理；无子代理提交或失败产物需要接手。
- 曾遇到 SQL dollar-quote 被脚本替换破坏、PostgreSQL assertion 中 `table_name` 歧义以及文档同步重复插入/旧计数问题；逐项修正后保留最终审计结果，不把失败运行算作通过。
- 已确认原 runner 有 28 步；本次增加孤儿 Master 的 deferred-constraint 负向用例后共 29 步。重新跑完完整 gate 后，需求、设计、计划、自动化索引均以 audit JSONL 的 29/29 为准。

## 5. 守门规则

1. Multipass 只由受限可信 Rust Host Infrastructure Manager 经 allowlist adapter 操作；本目录和任务卡 CLI 不暴露 Multipass daemon/socket 或任意宿主 shell。
2. HostEnvironment、Multipass instance、K3s cluster、membership、Kubernetes Node、Project、Branch、Engineering Run、Worktree 和 TaskExecutionRun 是不同身份。
3. 一个 Host 可以有多个 guest 与 K3s cluster；Run binding 必须 pin 精确 cluster ID/generation，不能只保存 provider name。
4. 只读 Multipass inventory 是观察输入，不证明 owner registration、容量、readiness、授权或运行资格。
5. Global catalog 不含 tenant 身份；读取和写入限于受限 Infrastructure owner role 与 owner-only FORCE RLS。
6. Master 业务字段不可原地改写；只允许在对应 retired Transaction fact 后单次闭合当前 generation。
7. Transaction facts 禁止 UPDATE/DELETE/TRUNCATE；生命周期状态变更按资源 identity/generation 串行化。
8. 当前 Multipass guest 不能同时绑定多个当前 cluster membership；精确 cluster generation 内 current Kubernetes node name 不得重复。
9. 五个 migration 必须在隔离 PostgreSQL 中各应用两次，并检查完整 16 张表的 FORCE RLS 与当前 migration grants。
10. Runner 仅在 loopback 启动它自己的 disposable PostgreSQL；必须确认 PostgreSQL 已停止后才能移除唯一数据目录。
11. 测试期总计最多登记 2 条 Multipass inventory record，包括 Stopped/Deleted；inventory 不可读、计数不明或已满时 fail closed，禁止自动 purge。
12. 不把数据库 schema gate 描述为 owner writer、生产部署、容量 reservation、provider readiness、BI、Benchmark 或 CLI 执行闭环。
13. Task Card CLI 继续关闭，直至当前 Project/Run/Worktree 授权、Host 和精确 cluster capacity、Manager/guest/K3s/CRI、独立 Runtime sandbox、取消恢复和独立验证都形成证据。
14. Rust workspace/toolchain/CI 继续固定 Rust 1.99.0；此迁移阶段无 Rust 源文件，不借用旧版本地 toolchain 声称完成 Cargo 验证。
15. 修改需求、基本设计、详细设计、W/T/M、实施计划与自动化索引时，执行 guarded sync 两次并保留第二次 no-op 证据。

## 6. 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis review — 资源身份拆分、Run 精确 cluster generation 与 owner 边界已复核。 |
| SRE Lead | Mavis review — loopback PostgreSQL 生命周期、29/29 runner 与清理证据已记录；目标 DB 仍未验收。 |
| Platform | Mavis review — FORCE RLS、最小 grants、WORM 和零 VM 验证范围已复核。 |
| 评审主持 | Mavis review — 仅将 schema substrate 标记为完成；provider、readiness、BI 和执行门保持打开。 |
| PM | Mavis review — 测试 inventory 两条上限、无 auto-purge 与产品拓扑无上限均在文档中区分。 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 INFRA-2C1 的五张 SCD2 Master、append-only Transaction、16 表累计 W/T/M 与 RLS、29/29 PostgreSQL evidence、两条 Multipass test cap 和 production gates。 | Host/Multipass/K3s identity catalog schema 完成隔离数据库验收。 |
