# WBS-W11 / PR-2 Multipass 与 K3s 基础设施差距审计

> 日期：2026-10-26
> 审计范围：WBS W11 与 Platform Runtime Delta PR-2 的验收要求、当前详细设计、Rust 源码切片、数据库验证记录及宿主级 Multipass/K3s PoC。
> 状态：差距审计完成；W11 / PR-2 保持 OPEN，未执行部署或测试，也未更改产品代码、数据库或 VM。
> 审计工作树：codex/wbs-w11-infrastructure-gap-audit-20261026，审计开始时 HEAD 为 a0b8d53f8d5ff930c310da2cc8de62000cc9801e。

## §0 目的与证据边界

本报告判断现有成果能否满足 W11 / PR-2 的产品验收门，并区分四种证据：

- **设计**：规范已定义，但没有相应生产执行证据。
- **源码切片**：Rust 类型、纯逻辑、owner 数据访问或受限只读 adapter 已存在；不代表运行中的 Manager/provider。
- **宿主实验**：本机脚本真实创建并验证过 VM/K3s；不代表 Star 产品服务能创建、管理或授权该环境。
- **生产验证**：真实目标部署身份、数据库 grants、运行中的 Manager/provider、Agent Runtime 和端到端行为均有证据。

CodeGraph Tier-2 结果由协调者从已索引 dev 快照 2074eb7baee702edec7cf56515587666a371a37c 提供；本审计 checkout 没有 .codegraph，连接的 MCP 未暴露 check_index_coverage。随后仅针对给定符号和路径核对了本工作树的精确源码行。未作全仓库负向穷举；“尚无证据”不等于声称仓库中绝不存在任何相关代码。

本轮没有检查 Multipass inventory、VM、数据库、Secrets、其他工作树或部署状态；没有运行测试或脚本。既有验证结果均按文档记录引用，不作为本轮重新实测结果。

## §1 改动矩阵与验收结果

本审计没有修改产品代码。WBS 的当前拆分为 W11（P1，依赖 W2/W4/W8）和 PR-2（P0，依赖现有 Infrastructure contract/profile/reservation、owner identity、target DB/grants 与 PR-4 预算门）。PR-2 的验收条件比 W11 摘要更具体，应作为生产闭环完成门。见 WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:60 与 WBS-PLATFORM-RUNTIME-DELTA-20261018.md:12,30。

| 验收面 | 当前证据及分类 | 结论 |
|---|---|---|
| 测试 inventory 最多 2 条，Stopped/Deleted 仍计数 | 宿主脚本在 multipass_k3s_gitops.ps1:25-26,168,331-360 定义两条上限；在锁内读完整 inventory 并拒绝已满、Deleted 目标和无法读取的情形。Rust adapter 在 multipass-adapter/src/lib.rs:236-237 的解析上限为 512 行，registered_count 只计数，不执行两条 cap。 | **部分满足（host test script only）**。没有证据证明所有 VM 创建入口共享此门，也没有 Rust Manager 的统一 cap enforcement。Rust 512 行解析界限不是 2 条产品上限。 |
| 并发创建的跨进程独占锁、损坏 inventory fail closed、绝不自动 purge | PowerShell 路径通过 FileShare.None 在 LOCALAPPDATA 下持锁，覆盖 inventory 读取与 launch；解析/执行失败拒绝创建；上限提示明确不自动 purge，见脚本:302-360。 | **宿主脚本满足其自身路径**。它不能证明其他脚本、用户直接调用 Multipass 或未来 Manager 的调用也被统一协调。没有运行脚本或验证当前 inventory。 |
| HostEnvironment、Multipass instance、K3s cluster、Kubernetes node 身份分层 | 详细设计 DD-LOCAL-INFRASTRUCTURE-001.md:64,76；INFRA-2C1 catalog 为五类 Master 与 append-only event，精确 generation/FK 关系见 DD:255-271。 | **设计与 schema 源码切片已具备**。catalog 不等于活跃 provider observation、容量、binding、readiness 或 execution 授权。 |
| Run 选择基础设施时固定 cluster ID + generation | RunInfrastructureBindingSnapshotV2 包含精确 k3s_cluster_id/generation、placement/profile digest；validate 拒绝 nil ID/非正 generation 且 digest 纳入两字段，见 domain-infrastructure/src/lib.rs:348-420,422-450。 | **版本化合同切片具备**；target Run owner resolver、生产写入/读取、权限复验与部署身份未验收。该绑定只适用于选择基础设施 placement 的 Run；hostless Run 不应被强制创建该绑定。另有下节所列 Cloud Branch 必填冲突。 |
| Host 与目标 cluster 双 reservation、owner-local 写事务 | Infrastructure admission 在自身 PostgreSQL 事务中锁 inbox/resource scope、检查 persisted profile digest，并从该事务读 Host 与精确 cluster 的活动 reservation；record_accepted 写入不同的 Host/cluster reservation ID、generation、capacity snapshot、receipt、Inbox terminal state 和 Outbox，见 infrastructure_owner_admission.rs:204-230,291-368,1231-1438。 | **Infrastructure owner 源码切片具备**。尚无真实 Run owner 产生 demand → Infrastructure consumer → 接受/回执 → Run owner 消费闭环或 target DB 证明。输入 observed_snapshot 仍须由受信 Manager/resolver 产生并绑定当前世代。 |
| Rust Infrastructure Manager 与 Multipass 创建/生命周期 | Multipass adapter 只允许 Inventory/Version/LocalDriver 三种固定只读命令，见 multipass-adapter/src/lib.rs:246-301。Infra2C2 Rust writer 可验签并持久化 Manager 提交的资源注册/生命周期收据；DD:281-297 明确当前 writer 不实现 provider inventory reconciliation 或周期 reconciler。 | **Manager/provider 未实现为可验收生产路径**。已存在签名 receipt/catalog writer 不等于 Manager 进程、launch/start/stop/delete adapter、密钥部署或运行实例。 |
| Inventory reconciliation | PostgresMultipassInventoryReconciler 在精确 Host generation 下使用 REPEATABLE READ / READ ONLY 事务，比较 ManagedPresent/Missing/ExternalObserved，不注册、retire 或改 capacity/readiness，见 infrastructure_multipass_inventory_reconciliation.rs:223-295；DD:327-343。 | **只读观察源码切片**，不是生命周期管理器或授权证明。 |
| K3s aggregate/API/node/CRI、Agent sandbox 与 workspace bridge | ReadinessEvidenceV2 包含 manager receipt、daemon、所有 owned guest aggregate、exact cluster/generation、K3s API/node 数、CRI、Agent sandbox、Worktree bridge，见 domain-infrastructure/src/readiness.rs:197-261；纯 gate 检查层级完整且新鲜，见 :303-426。 | **纯验证合同/fixture**。没有 readiness producer、真实 Multipass/K3s 获取及签名事实，也没有证据证明失败时生产 executor 确实被门禁阻断。 |
| 无 host shell/daemon/socket/任意 mount 暴露给 Agent/Plugin/CLI | 设计明确要求窄 allowlist 和 Manager 信任边界，DD:76-84,100；Rust adapter 当前只有只读 probe。 | **设计已定义，尚无 Agent/Plugin/Task CLI 集成与隔离验证**。只读 adapter 本身不证明实际调用拓扑不可越权。 |
| 数据库与授权 | INFRA schema、grants 和 RLS 有隔离 PostgreSQL 18.6 runner / SQLx slice 记录；WTM 文档:5,94-116 明确 target DB、生产 owner membership、NOLOGIN/NOBYPASSRLS 角色部署与生产 grants 未验证。 | **隔离测试通过记录不等于目标部署验收**。本轮没有目标数据库身份、Neon 或 Multipass VM 内数据库的连接/grants 证据；也未确认 Infrastructure control-plane PostgreSQL 的生产位置与独立可用性（DD:341）。 |
| 取消、过期、崩溃恢复与资源释放 | 有 bounded expiry/admission/outbox lease primitives；WTM:106-110 明确没有生产 expiry scheduler、跨租户协调、生产 Outbox delivery/Run outcome consumer。DD:80,88-90 是 operation lease/drain/recovery 设计。 | **有限 owner primitive 有源码，Manager operation 生命周期和端到端恢复未完成**。无法证明重启、取消、撤权、容量释放或 VM orphan 对账。 |

### §1.1 宿主 PoC 与产品实现的区分

PHASE-MULTIPASS-K3S-GITOPS-REPORT.md:22-32,50-86 记录了本机 Windows 11 Home / Multipass 1.17.0-rc1/HCS 的一台 4 CPU、8 GiB、28 GiB guest，运行 K3s v1.36.5+k3s1；19 个基础设施/GitOps Pod 均为 Running。该报告明确称宿主基础设施是“未入库，仅本机”（:22-32）。它证明固定设备、驱动和这一组声明式服务的宿主/K3s PoC，不证明 Star Rust Manager、Run placement、Run owner授权、容量预约、任务执行隔离或产品恢复。

该 PoC 报告仍列出 Windows 侧 kubectl 直连未验证、mount 未启用、仓库使用 transfer、rc1 升级路径未验证等事项（:132-143）。不能把一次 K3s/GitOps 就绪外推为 sandbox 或受授权 workspace bridge 已就绪。当前是否仍有 VM、inventory 计数及磁盘状态，本审计未检查（NOT_INSPECTED）。

## §2 验证摘要

| 证据 | 文档所记结果 | 可以证明 | 不能证明 |
|---|---|---|---|
| Rust domain readiness | readiness.rs 有 every_readiness_layer_must_be_ready、readiness_evidence_must_match_both_capacity_reservations 等 fixture tests，见 :664-850 | 纯 validator 对不完整层级、reservation ID/generation 不匹配的拒绝逻辑已编码 | 测试本轮未运行；不证明 readiness producer、provider 状态获取或 executor 实际受控 |
| Multipass read-only adapter | adapter tests 覆盖 Stopped/Deleted 保留、空 inventory、malformed、duplicate 与 parser row limit，见 multipass-adapter/src/lib.rs:795-900；当前 row limit 为 512（:236-237） | parser 和 bounded probe 的源码级单元断言存在 | 本轮未运行；不证明两实例 cap，因为 parser limit 不是 cap |
| INFRA-2C4 | DD:343 记录 Rust 1.99 adapter/infrastructure suites 19/19 与 62/62、SQLx 1/1、PG18.6 runner 28/28、17/17 FORCE RLS；零 VM | 文档记录的 bounded source、SQLx 与 disposable PG gate | 不是目标 PostgreSQL、Neon、Multipass VM 内 DB 或生产 grants 验收 |
| INFRA-2C2 | WTM:114 记录 disposable PostgreSQL 18.6 runner 26/26、17/17 FORCE RLS 和真实 Rust SQLx writer test；零 VM | 签名 receipt/catalog writer 在隔离环境的特定数据库行为 | 不是部署 Manager、生产 key/role、目标 DB grants 或 provider create 生命周期 |
| D20 consumer/publisher | DD:397,405 与 WTM:5,9 记载 source slice、隔离 DB 与测试结果；同时标记 opt-in、无生产 consumer、无 native server/provider 执行 | bounded event/Outbox source slices 的局部合同与隔离验证 | 不证明持续 worker、可信 resolver 生产接线、Run settlement、capacity/readiness 或 Manager |
| Multipass/K3s host PoC | 宿主报告记录一台 guest 与 19/19 Pods Running | 固定本机 PoC 的硬件/driver/runtime 结果 | 不证明两条上限下的并行/压力/恢复；不证明 Star Manager 或 Agent Runtime |

本轮验证结论：**未运行任何测试/编译/脚本（NOT_RUN）**。上述历史验证仅依文档引用，均需与提交版本和后续执行计划一并看待。隔离 PostgreSQL runner 和真实目标数据库必须分别报告。

## §3 已知缺口与设计风险

1. **hostless Run 与 Cloud Branch 必填身份不一致（P0 设计前置）**。W2 验收要求无远端宿主时仍可创建、授权和列出 Run（WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:51）；当前绑定快照则把 branch_id 标为“Cloud Branch”，校验要求其非空并将其放入 digest（domain-infrastructure/src/lib.rs:355-356,399-417,422-435；readiness.rs:209-210）。详细设计也将 Project/Branch/Run 放在一个 Infrastructure binding 上（DD-LOCAL-INFRASTRUCTURE-001.md:64,84）。在接生产 resolver/Manager 前，需将此字段改为符合当前产品语义的可选远端关联，或移出 Run 基础绑定并由 Worktree 的本地 checkout ref 单独表达；不得让远端宿主 Branch 成为 Run 身份或授权前提。需同步需求、基本设计、详细设计、schema/API 以及 digest/replay 兼容契约。
2. **2-record 规则未成为统一 provider 约束**。宿主 bootstrap 脚本覆盖自身 launch 路径；Rust snapshot 只保留/统计所有行，最大解析数量为 512，未见对应两条 cap/并发创建操作。不能据此断言所有创建调用都通过锁。将来 Manager 所有 VM create 路径都必须串行化同一 inventory 的验证与 launch；手动/外部调用的治理策略需明确。
3. **Readiness evidence 没有 producer**。合同把安全层拆得足够清楚，但 manager receipt、guest aggregate、K3s API/node/CRI、sandbox、bridge 的真实、可新鲜重验事件尚未由运行中的 provider 采集与持久化。
4. **两套 owner 的端到端交接不完整**。Infrastructure admission 的单 owner 事务能配对写 Host/cluster 预约事实，但没有完整目标部署链证明 Run/Execution owner demand、Infrastructure inbox/receipt/outbox、Run 侧 settlement 各自提交且可恢复。
5. **目标数据库与角色未验收**。历史 PostgreSQL 18.6 证据使用 disposable loopback runner。真实 Neon 部署与 Multipass VM 数据库均无本报告可确认的身份、grants、RLS、备份和恢复结果；Infrastructure 控制面数据库还要求与目标 K3s cluster 分离并独立可用。
6. **Operator 的生命周期与安全域未闭合**。受限 Rust Manager 的真实进程身份、key provisioning/rotation、argv allowlist、operation lease、cancel/drain、崩溃后的 reconciler、外部实例隔离、拒绝任意 mount，以及 Agent sandbox/Worktree bridge integration 都未在真实产品执行中验证。
7. **WBS 优先级与并行条件需显式对齐**。总计划 W11 是 P1，平台差异 WBS 中 PR-2 是 P0（WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:60；WBS-PLATFORM-RUNTIME-DELTA-20261018.md:30）。这可能是不同层级的优先级，但执行看板应说明其关系。PR-2 依赖 owner identity、target DB/grants、PR-4 资源预算；当前 W2/W4/W8 的授权与 owner/transport seam 未全部生产验收，最终 Agent E2E 还依赖 PR-1/PR-3。

### 最小后续 lane 建议

1. **W11-A：先修 Run binding 语义**。在要求任何 provider 动作前，明确 hostless Run 的 binding 字段；移除 Cloud Branch 必填依赖或改成明确 optional association，并更新三层设计、序列化摘要与迁移/兼容测试。完成门：无宿主 Run 可被持久化/授权；缺远端 provider metadata 不影响其 Project/Run scope；Worktree-local Branch 只在其 own context 中产生。
2. **W11-B：建立受信 resolver 到单 owner admission 的生产接缝**。明确 Infrastructure owner 如何解析当前 Run binding、Profile digest、Host/cluster generation 与 ACL；证明 Host/cluster reservation 在 Infrastructure 自己的事务中幂等写入，通过版本化 Outbox/Inbox 回到 Run owner，不跨 owner 写库。依赖 W2/W4/W8 及 target DB/grants。
3. **W11-C：再实现受限 Rust Manager/provider 生命周期**。统一 inventory lock/cap 与 create/inspect/start/stop/delete owned-resource boundaries；拒绝 malformed、超过 2 条和 owner 不明状态，不自动 purge；以签名 operation receipt 原子更新 catalog。先不暴露任务执行命令。
4. **W11-D：实现实际 readiness producer、双层恢复和桥接**。按精确 cluster generation 聚合 guest/API/node/CRI readiness；资源预约、binding、当前 ACL、native Hook、runtime sandbox、受限 Worktree bridge 全部新鲜匹配之前保持 TaskExecutionRun non-executing；补 cancel/drain/restart/reap 及真实目标 PG/授予身份验收。
5. **W11-E：最后用最多两条 inventory 记录做端到端与资源验证**。分别报告 host manager/VM 实际状态、目标 DB/grants、Agent sandbox、bridge、并发/故障/恢复与 RSS/延迟数据。过不了任一门就继续 fail closed；不运行无人归属 VM 清理。

## §4 子代理失败接手清单

- 本 lane 未派生子代理；没有子代理 RPC/交付失败需要接手。
- 本轮只读审计；没有执行部署、迁移、编译、测试或 VM 操作。
- 需要协调者在合入前复核 §3.1 Cloud Branch 语义和代码/文档基线是否仍一致。

## §5 守门规则

1. W11/PR-2 持续 OPEN；合同、schema、fixture、源代码或 PowerShell PoC 均不能单独标记生产闭环。
2. 测试阶段 Multipass inventory 所有登记记录合计不超过 2；Stopped 与 Deleted 都计数，未知状态不减免名额。
3. 所有允许的 create 入口必须在跨进程独占锁内完成 inventory 验证和创建；无效 inventory、锁不可得或达到上限时 fail closed。
4. 不自动 purge、不自动恢复 Deleted 实例，不对 owner 不明实例 stop/delete。
5. HostEnvironment、Multipass instance、K3s cluster、Kubernetes node 使用独立身份与 generation；选择基础设施 placement 的 Run binding 固定 cluster ID+generation。hostless Run 可不含该绑定，不能因此失去 Project/Run 身份、授权或列表能力。
6. Project/Run 身份、当前 ACL、provider profile 和目标 generation 必须由受信 owner/resolver 获取，wire 中自报 ID 不构成授权。
7. Run/Execution 与 Infrastructure owner 分别在自己的事务中写入 owner facts；禁止跨 owner 直接改表或跨 owner 共享写事务。
8. Rust Manager 是唯一获得受限 Multipass mutation capability 的可信组件；Agent、Plugin、Task CLI 不获得宿主 shell、daemon/socket、凭据或任意 mount。
9. 双 reservation、有效 binding、当前 ACL/Hook 与 K3s/CRI、独立 Runtime sandbox、受限 bridge 证据全部新鲜匹配前，不启动 TaskExecutionRun。
10. 将 disposable PostgreSQL 结果、Rust fixtures、Windows host PoC、目标 DB、Manager 实际执行与 Agent 实际执行分别记录，不能互相代替。
11. 本审计没有查看当前 VM inventory，也没有验证两条名额现状；后续任何真实 VM 操作必须先遵守受控锁与预算流程。
12. 更新实现后须逐条同步需求、基本设计、详细设计和 WBS，标出实际验证门及仍未完成项。

## §6 签批栏

| 角色 | 结论 |
|---|---|
| 架构 | Mavis 接手审阅通过；审计结论：W11/PR-2 未达到生产完成门 |
| SRE Lead | 未进行生产部署/恢复审查；目标环境证据缺失 |
| 平台 | 确认现有 Rust evidence 为受限源码切片，未证明生产 Manager/provider |
| 评审主持 | CodeGraph Tier-2 与定向源码/文档证据已引用；索引覆盖检查工具不可用，未作全仓负向断言 |
| PM | 后续以 §3 最小 lane 和 §5 完成门更新 WBS；不将宿主 PoC 计为产品完成 |

本报告为子代理审计产物，已由集成协调者完成文档与证据边界复核，待提交集成；不代表 W11/PR-2 产品验收完成。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-26 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审计 | 新增 W11/PR-2 的证据分层、验收差距、hostless Run/Cloud Branch 设计冲突和后续实施 lane；未运行测试，保持开放状态 | 协调者 brief 指派 Multipass/K3s gap audit |
