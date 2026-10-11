# PHASE-INFRA-2C4-INVENTORY-RECONCILIATION-REPORT

## §0 目的

记录 INFRA-2C4 的有界 Multipass inventory 只读对账切片、验证结果及生产边界。该阶段不接入其他分支的大型改动，也不宣称 Multipass/K3s 执行闭环已完成。

## §1 改动矩阵

| 范围 | 改动 | 结果 |
|---|---|---|
| Adapter | 暴露有界 Multipass JSON parser；合法空 list 作为完整的零实例观察。 | malformed/duplicate/over-limit 仍 fail closed。 |
| Infrastructure owner | 增加精确 Host ID/generation 的只读目录对账器。 | 分类 ManagedPresent / ManagedMissing / ExternalObserved；不写 Master、event、capacity 或 BI。 |
| PostgreSQL | 沿用现有 17 表迁移链，没有新增迁移或表。 | 17/17 FORCE RLS；W/T/M 仍为 4/7/6。 |
| 文档 | 同步 requirements、basic design、Infrastructure DD、Group DD、implementation plan、automation design 与 registry。 | 标明 control-plane PostgreSQL 独立于目标 K3s 集群；生产部署尚未验证。 |

## §2 验证摘要

- cargo fmt -p infrastructure -p multipass-adapter -- --check：通过；cargo fmt --all --check 未通过，差异出现在本次未改动的其他 workspace 文件。
- cargo test -p infrastructure -p multipass-adapter --lib --offline -j 4：通过；infrastructure 62/62，multipass-adapter 19/19。
- cargo clippy -p infrastructure -p multipass-adapter --all-targets --no-deps --offline -j 4 -- -D warnings：通过。
- cargo check -p infrastructure -p multipass-adapter --all-targets --offline -j 4：通过。
- Rust SQLx PostgreSQL integration：1/1 通过；两个 fixture 为 Stopped managed 与 Deleted external，验证 empty snapshot/missing、stale Host generation fail-closed 及 event count 不变。
- PostgreSQL 18.6 累积 runner：28/28 步通过；17/17 FORCE RLS。临时数据库干净停止；没有执行 Multipass 命令或创建 VM。

## §3 已知缺口

- 工作区级 rustfmt 仍有本次未改动文件的差异；C4 两个 package 的格式检查通过，未对其他模块做格式化改写。
- Infrastructure control-plane PostgreSQL 生产部署须位于目标 K3s 集群之外并独立可用；本阶段只记录架构门槛，未验证生产部署。
- 真实 Multipass 进程、Host/Manager 身份 provisioning、生产目标 DB 登录/授权和 API/UI wiring 未完成。
- 对账结果不等于资源 ownership、cluster capacity、K3s/CRI/runtime readiness，也不产生 BI/Benchmark 或执行授权。
- 多集群 provider、RunInfrastructureBinding、Host/cluster 双层 reservation、agent sandbox、原生 Hook、取消/恢复、schedule worker 和任务卡 CLI 闭环仍未完成并保持 fail closed。
- 测试最多登记两条 Multipass inventory 记录（含 Stopped/Deleted），不自动 purge；本阶段未读取或修改真实 inventory。

## §4 子代理失败接手清单

本阶段没有派发子代理；没有子代理结果需要接手。

## §5 守门规则

1. 只合入本阶段 C4 的有界只读对账代码和同步文档，不把其他分支大型重构纳入本次集成。
2. 不完整、无效、过量或身份不匹配输入均 fail closed；数据库错误不能当作空 inventory。
3. Multipass instance、HostEnvironment、K3s cluster、Kubernetes node 是不同身份。
4. 对账不得注册/退休资源，变更事件、容量、readiness、BI 或执行状态。
5. control-plane PostgreSQL 必须在目标 K3s 集群外并可独立启动；生产拓扑需另行验收。
6. 测试阶段最多两个保留 Multipass inventory 实例，不自动 purge，不创建 VM。
7. 未闭合 Manager/provider、数据库身份、capacity、readiness、sandbox、Hook、Run 与 executor 门前，Schedule/Task Card 执行保持关闭。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 C4 有界 Multipass inventory parser 与 Infrastructure owner 只读对账；同步需求、基本/详细设计、计划和自动化索引；记录 focused Rust、SQLx、PostgreSQL 18.6 evidence 与未完成的生产 DB/Manager/provider/Run/readiness/BI/execution 门。 | 本对话确认 Multipass/Multi-K3s 架构并要求只选择性整合符合设计的已验证成果。 |
