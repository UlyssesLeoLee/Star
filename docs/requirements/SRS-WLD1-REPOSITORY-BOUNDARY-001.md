# SRS-WLD1-REPOSITORY-BOUNDARY-001

> 状态：Draft v0.4；需求已纳入 Project owner、部署选择、hostless Run、可选 infrastructure placement 及 Schedule/手动触发来源契约，尚未实施或通过运行验收。
> 基线：dev `63d97ca5b00a1476e0beeaad0e88032c85b0f1b6`。
> W11-A 设计增补基线：dev `4f629ce5103fb4a9fb67a775341475ef66bbf2ea`；仅文档决策，待同步 `DD-LOCAL-INFRASTRUCTURE-001` 与 WBS。
> 配套文档：`BD-WLD1-REPOSITORY-BOUNDARY-001.md`、`DD-WLD1-REPOSITORY-BOUNDARY-001.md`、`TD-WLD1-REPOSITORY-BOUNDARY-001.md`。

## §0 文档信息

| 项 | 内容 |
|---|---|
| 文档 ID | SRS-WLD1-REPOSITORY-BOUNDARY-001 |
| 主题 | Project–Repository 授权关联、host-independent Run 绑定、可信 Host root 与只读 Worktree 发现 |
| 范围 | WLD1-0 领域契约；不包含创建、导入、合并或清理 Worktree |
| 修订 | v0.4 · 2026-10-27 · 明确 V3 demand 同时支持 Schedule 与手动触发、attempt 唯一性和 origin digest/replay；V2 保持 Schedule-only |
| 上游 | `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` WLD1；现有 Repository master 与 Engineering Run 目录设计 |

## §1 目的与产品结构

WLD1 为 Run 管理本地 Worktree 建立明确的授权和机器边界，避免把 Project 同租户、远端 Branch 注册或 Repository 创建者身份误当授权。Engineering Run 的 identity 与创建、授权、列出不依赖远端宿主 Branch；其核心范围为 tenant + Project + Run。产品主导航固定为 **Project → Engineering Run → Worktree → Branch**。Repository binding 与 Infrastructure placement 是两个独立的可选 Run 关系；Repository 是 Project 设置及 Run 绑定所引用的资源身份，不是 Worktree 的父导航层；Branch 节点显示 Worktree 的本地 checkout ref/state，不代表远端 Branch。

本设计先建立可验证的 Repository 关系、Host root 身份及只读发现。生产 Worktree 写操作继续关闭，直至后续独立阶段具备授权、持久化、审计、隔离执行及运行验收。

## §2 参与者与用户目标

| 参与者 | 目标 |
|---|---|
| Project Manager | 在 Project 内明确关联一个或多个 Repository；可撤销关联并查看变更记录 |
| Run Owner / Operator | 可在不选择 Repository、Host、远端 Branch 或 infrastructure placement 时创建并管理 Run；需要 Worktree 发现或特定执行环境时，再分别显式选择对应 binding |
| Project Member / Run Member | 只读取自身当前有权 Project、Run 的资源和脱敏 Worktree 发现结果 |
| Trusted Host Infrastructure Manager | 以本机受信方式注册/更新 Repository root；维护 Host 与 root generation |
| Agent / Plugin / Task CLI | 只能通过已授权 Run API 使用不带绝对路径的受限发现结果；不得注册或选择宿主路径 |

## §3 词汇

- **Repository master**：tenant 内稳定的 Repository 身份与非授权元数据；不蕴含 Project、Run 或 Host 授权。
- **ProjectRepository association**：Project owner 管理的 Project 与 Repository 之间可审计、可撤销关系。
- **RunRepository binding**：Engineering Run owner 对已关联 Repository 的显式选择及版本化引用；可为空，不要求远端 Branch。
- **RunInfrastructureBinding**：Engineering Run owner 管理的可选基础设施 placement revision；只有 Run 显式选择 K3s/其他受支持 placement 时才存在，固定 Profile digest、Host/cluster identity 与 generation，不构成 Run identity。
- **Remote provenance metadata**：可选的 Repository/provider/remote-branch 来源信息；只作 provenance，不参与 Run 创建、actor 授权或 Worktree local checkout-ref 推导。
- **Host root registry**：可信 Host Manager 管理的 `(HostId, RepositoryId, generation)` 到本机 canonical root 的受保护映射。
- **Worktree discovery**：只读观察一个精确绑定 root 下 Git worktree 元数据的投影；不是导入、管理或本地注册动作。
- **Local checkout ref**：`branch`、detached HEAD 或 unborn state 等 Worktree 当前本地 ref 信息，不是远端 Branch 对象。

## §4 范围与排除

### 4.1 本阶段范围

1. Star 原生 Project owner 唯一持有 Project identity、Project membership authorization facts 与 ProjectRepository associations。一个 Project 可关联 0..N 个同 tenant Repository；可选 default Repository 仅作界面偏好，不参与授权或隐式 Run binding。
2. Run 可无 Repository、无 Host、无远端 Branch、无 Infrastructure placement 创建、授权、列出和管理；RunRepositoryBinding 与 RunInfrastructureBinding 是独立、显式且可撤销的后续选择。
3. Worktree discovery 需要独立的 RunRepositoryBinding 与 Host/root generation；选择 K3s execution placement 则使用独立的 RunInfrastructureBinding。任一 binding 均不要求 Cloud Branch，也不把本地 checkout ref提升为 Run identity。
4. 受信 Host Manager 持有 canonical root；服务端和前端只交换 opaque identity/generation 与脱敏 DTO。
5. 单独的只读 provider 根据有效 Run binding 发现 Worktree，并返回分页/限量的 Worktree 与 local checkout ref 投影。
6. 变更由事实 owner 原子写入 Audit 与 Outbox；跨 owner 用版本化命令、API 和事件交互。
7. Vercel 部署的 Project owner 使用 Neon；K3s 部署使用 Multipass VM 内的数据库。两种部署分别持有自己的连接、数据与迁移验收，不推定共享或复制数据。

### 4.2 明确排除

- Worktree create/import/archive/delete/cleanup、任意 Git 写命令、合并或分支重写。
- Agent/Plugin/任务卡 CLI 直接访问 Host shell、Multipass socket、文件系统 root 或绝对路径。
- 以远端 Branch、仓库 URL、Git provider 连接状态作为 Run 的创建、授权或可见性前置条件。
- 以 Repository master、Project 同租户、creator、Task link 或 Worktree list projection 代替 Project/Run authorization。
- 将只读 provider 与带写能力的生命周期 provider 合并为一个默认启用的 capability。

## §5 功能需求

| ID | 需求 |
|---|---|
| FR-WLD1-01 | Project Manager 只能在当前授权 Project 中关联 Repository master；Repository 必须属于同一 tenant。Project 可关联多个 Repo；相同有效关系不得重复创建。 |
| FR-WLD1-02 | ProjectRepository 关联新增、retiring fence、最终撤销和重放必须由 Star 原生 Project owner 以 expected revision + idempotency key 执行并写 Audit/Outbox；同键不同请求内容返回冲突。可选 default Repository 只引用同一 Project 的 active association，不创建 Run binding。 |
| FR-WLD1-03 | Run 创建、授权、列出和基本管理必须在没有 Host、Repository、远端 Branch 或 infrastructure placement 时成功。无 RunRepositoryBinding 不返回任何 Host Worktree，也不隐式继承 Project 的全部 Repo；无 RunInfrastructureBinding 不接受要求该 placement 的执行请求。 |
| FR-WLD1-04 | Run Owner 只能绑定该 Project 当前有效关联的 Repository；绑定记录关联 revision、HostId、root id/generation，并采用 CAS、幂等、审计和 Outbox。Run 可以显式绑定多个 Repo；每个 binding 独立识别目标 Host root，Worktree 结果不可跨 binding 混淆。远端 Branch 不得成为此 binding 的必填字段。 |
| FR-WLD1-05 | 每次读取或发现前复验当前 Project member/manager、Run grant、ProjectRepository state/revision、RunRepository binding revision 和 Host root generation。新 binding 只接受 active association；retiring association 可仅供已有 binding 在迁移完成前继续解析；revoked、撤权、过期或未知状态 fail closed。 |
| FR-WLD1-06 | Host root 只能由可信 Host Infrastructure Manager 经本机受控选择/注册流程建立。API、Agent、Plugin、Task CLI、Schedule 或用户请求不得传入或改写绝对路径；若 Repo 没有 remote identity，则必须以 Project 授权的本机操作者确认 Host root 与 RepositoryId 的关联，并记录该信任来源。 |
| FR-WLD1-07 | Host registry 记录稳定 Host identity、Repository identity、generation、健康状态及受保护的本机 root handle/reference。路径和凭证不出现在远端 API、事件、日志、BI 或普通前端 DTO。 |
| FR-WLD1-08 | 只读 Worktree provider 只接受已授权 Run binding，执行受限、可取消、超时且输出有界的 Git worktree discovery；provider 缺失、root stale、Host 离线或输出异常时返回明确 unavailable/error，绝不降级为写操作。 |
| FR-WLD1-09 | Worktree UI 位于 Run 下；Branch 子节点是本地 ref/state。无绑定、无 Host、detached HEAD、root stale、provider unavailable 与空列表都必须有可区分状态。 |
| FR-WLD1-10 | 异步事件只包含 opaque IDs、revision/generation、状态、correlation/idempotency 与最小摘要；消费方在本地事务完成后确认，并支持重复/乱序投递。 |
| FR-WLD1-11 | Repository master 可为 local-only：`provider_key`、`remote_identity`、`remote_url` 与 default branch 可为空；稳定 RepositoryId 由 SCM owner 分配，Host root 关联由可信操作者确认，不要求远端 Branch。 |
| FR-WLD1-12 | Star 原生 Project owner 是 Project identity、持久成员授权事实和 0..N Repository association 的唯一业务真相；principal/authentication identity 仍由其身份 owner 提供。当前授权必须通过版本化 owner API/命令校验，缓存或 UI 状态不能替代。 |
| FR-WLD1-13 | Vercel 使用 Neon；K3s 部署使用 Multipass VM 内数据库。浏览器不得直连数据库；两部署的数据、凭据、迁移状态与授权域相互隔离，未经版本化迁移不得假定共享或复制。 |
| FR-WLD1-14 | 解绑引用只可迁移到同一 Project 内当前 active 且已授权的 Repository association，或由 Project owner 显式退役已不再需要 checkout 的引用。迁移核验前旧 association 与既有 binding 保持有效；缺少新鲜、完整的零引用证明或 owner receipt 时必须拒绝最终解绑。 |
| FR-WLD1-15 | `RunInfrastructureBinding` 仅在 Run 显式选择基础设施 placement 时创建或更新；有效绑定固定 tenant/Project/Run scope、Profile/version/digest、HostEnvironment ID+generation、目标 K3s cluster ID+generation、namespace、runtime profile 与预算。未选择 placement 的 hostless Run 不得以 nil UUID、占位 Branch 或推测 provider 伪造 binding。 |
| FR-WLD1-16 | 要求指定基础设施的 TaskExecutionRun 必须在准入和每次 resume 时重新授权并复验当前 binding、Profile digest、Host/cluster generation 与 readiness；缺失、撤权、漂移或未知状态 fail closed。无需该 placement 的 Run 管理/API 操作不得被强制要求远端 Branch。 |
| FR-WLD1-17 | Infra demand 新写使用 `RunInfrastructureBindingSnapshotV3` / `HostInfrastructureDemandV3`。V3 scope 不含必填 Branch；V2 历史 replay 仍按原 V2 canonicalizer/branch_id 校验，不能转写为 V3、补造/清空 Branch 或跨版本复用幂等 receipt。 |
| FR-WLD1-18 | V3 demand 必须携带判别式 `trigger_origin`：Schedule variant 要求有效 `schedule_rule_id`、创建时冻结的 `schedule_rule_revision` 和 `occurrence_id`；Manual variant 要求 Run owner 基于认证命令分配并固化的唯一 `manual_command_id`，不得以 nil Schedule UUID 表示手动触发。Manual variant 可带可选、已证明来源的 `schedule_attribution`，仅作因果溯源，不改变其 Manual 身份或授权判断。 |
| FR-WLD1-19 | 每个 V3 `TaskExecutionRun` 标识一个不可变 execution attempt，并带正数 `attempt_number`；同一 attempt 的投递重试沿用 `task_execution_run_id`、origin、attempt 与 event/idempotency identity。一次新的执行 attempt 必须取得新 `task_execution_run_id` 和该 origin 下未使用的 attempt number。Schedule occurrence 以 `(tenant, schedule_rule_id, schedule_rule_revision, occurrence_id)` 唯一识别；Manual command 以 `(tenant, manual_command_id)` 唯一识别；各 origin 下 attempts 另行区分。 |
| FR-WLD1-20 | V2 demand/consumer 保持 Schedule-only：执行来源必须是 `run_origin=schedule`，且旧 Schedule rule/revision/occurrence 校验通过；Manual demand 在 V2 必须拒绝。V3 的 origin tag、optional schedule attribution、attempt number、binding/placement 和其它安全敏感字段须进入新 V3 canonical digest；同 origin+attempt 同 digest 回放原 receipt，不同 digest 冲突，新 attempt 以新 attempt identity 写入。 |
| FR-WLD1-21 | V3 每个 origin 必须绑定由 owner 从权威授权事实解析的不可变 `execution_actor_id`：Schedule 使用对应 Rule revision 的 `run_as_actor_id`，Manual 使用创建 command 时由认证上下文盖章的 actor ID；不得由客户端设置。每次 trigger/admission/retry/resume 都按该 actor 实时复验当前 Project/Run execution authorization。Manual 的 `schedule_attribution` 不得改变 actor 或授权结论。 |

## §6 非功能、安全与资源需求

- **租户隔离**：所有 server-side relation 的 key、查询、唯一约束和 FORCE RLS 均包含 tenant scope；Project 授权不可从 tenant 相同推导。
- **对象授权**：每个 API 以当前 actor context 重新验证 Project 与 Run 权限；opaque ID 不构成授权凭据。
- **路径隔离**：路径只存在于 Host 信任边界。API response、错误、日志、trace、Audit、Outbox、BI 与插件事件均不得输出 absolute root/worktree path、credential 或原始 Git stdout/stderr。
- **路径健全性**：root 解析应验证 canonical directory 与文件身份/generation；symlink、mount、目录替换、generation 漂移、重复 root 与竞态均 fail closed。
- **进程边界**：禁止 shell string；只用绝对可信 Git executable + argv；限定 cwd、环境、stdin、输出字节、工作量、并发、deadline 与取消；超限终止子进程并丢弃原始输出。
- **低资源**：按 Run/Host 并发预算公平排队；同一 binding 至多一个发现请求在途；列表分页、缓存有界且以 binding/root generation 为键；不得后台定时扫描全部仓库。
- **可审计性**：关联、撤销、Run bind/unbind、Host root 注册/换代和失败拒绝均记录 actor、scope、expected/actual revision、原因码、correlation 与幂等身份。
- **版本与回放**：V3 digest 必须绑定 schema/domain version、tenant/Project/Run/task/attempt scope、trigger-origin kind 与完整 variant、optional schedule attribution/remote-provenance 的存在标记和规范化内容、完整 placement/Profile/Host/cluster generation；V2/V3 使用分离的 wire decoder、digest domain 与 idempotency namespace。
- **可用性边界**：当前阶段只允许 read-only capability。Worktree 写入能力必须由后续版本化 Host action profile 和独立 acceptance gate 启用。

## §7 验收条件

| ID | 可验证结果 |
|---|---|
| AC-WLD1-01 | Project 可关联 0..N 个 Repo，default preference 可空且不隐式绑定 Run；attach/retire/finalize 遵守 CAS、幂等与审计，跨 tenant 或无 Project 权限被拒绝。 |
| AC-WLD1-02 | hostless Run 在无 Repository、Host、远端 Branch 时可创建、列出和授权；Run 不隐式取得 ProjectRepository 内容。 |
| AC-WLD1-03 | 新 binding 只能指向当前 active ProjectRepository；retiring 时既有 binding 在当前用户授权仍有效的条件下可继续解析至迁移完成；final revoke、Run revoke、Host/root generation 变更后读取拒绝。 |
| AC-WLD1-04 | Agent/API 不能注入路径；Host root canonicalization、symlink/mount 替换及 stale generation 均 fail closed；远端响应/日志无绝对路径。 |
| AC-WLD1-05 | 只读 discovery 对合法 Git root 返回稳定、分页、脱敏的 Worktree/local-ref DTO；不存在任何文件写入或任意 shell 行为。 |
| AC-WLD1-06 | 目标 PostgreSQL 迁移可重复应用；owner/app 登录角色、FORCE RLS、精确 grants、跨 tenant/project 探针、事务回滚、Outbox replay 和撤权竞争通过。 |
| AC-WLD1-07 | provider 未装配、超时、取消、输出溢出、损坏输出、root/host unavailable 时返回明确错误且不泄露原始内容、不触发写操作。 |
| AC-WLD1-08 | `Project→Run→Worktree→Branch` 导航契约保持；Repository 只作为 Project/Run 绑定资料，Branch 不表示远端父节点。 |
| AC-WLD1-09 | 无 provider/remote URL/default branch 的 local-only Repository 可被 SCM owner 稳定识别、由 Project 显式关联并经可信 Host 注册；整个流程不要求远端 Branch。 |
| AC-WLD1-10 | Project identity、成员授权和 Repository associations 均由 Star 原生 Project owner 持久化；每次 API 操作复验当前授权，不能由 UI/缓存或 SCM/Run owner 写入替代。 |
| AC-WLD1-11 | 同一数据库迁移与 owner/RLS/grants 验收分别通过 Vercel→Neon 和 K3s→Multipass VM 数据库配置；任一部署失败不能被另一部署的结果替代。 |
| AC-WLD1-12 | unlink 仅在同 Project 合法迁移或显式退役、各 owner 新鲜完成回执、完整 revision-aware reconciliation 证明源关联零活动引用并通过 CAS 后成功；任一故障保留旧 association 与 binding。 |
| AC-WLD1-13 | 以无 Repository/Host/远端 Branch/Infrastructure placement 创建的 Run 可被授权和列出；Run identity、membership 检查与列表不查询 Cloud Branch。对该 Run 的 Worktree discovery 只返回未绑定状态，不枚举 Project 私有 root。 |
| AC-WLD1-14 | 有 placement 的 Run 绑定及 demand 固定精确 Profile/Host/cluster generation 与预算；没有 placement 的请求不能启动要求该环境的 execution。V2 fixture 可按旧 hash 回放，V3 只由新 schema writer 产生；同 key 跨版本不得误判为相同请求。 |
| AC-WLD1-15 | V3 Schedule 与 Manual demand 均可通过有判别式 origin variant 验证；Schedule 要求 rule ID/revision/occurrence，Manual 要求 command ID；Manual 可选 schedule attribution 是明确对象或 absent，不使用 nil/空 Schedule UUID。 |
| AC-WLD1-16 | 相同 Schedule occurrence 的重复触发只回放同一 logical origin；Manual command 重复提交只回放原始命令；同一 attempt 的 redelivery 使用相同 `task_execution_run_id`/`attempt_number` 和 digest。新 attempt 使用唯一 ID/attempt number，origin revision 与因果 provenance 保持冻结。V2 手动来源拒绝，V2 Schedule replay 保持原摘要。 |
| AC-WLD1-17 | Schedule 每次 trigger/retry/resume 都按当前权限复验 Rule revision 固定的 `run_as_actor_id`；Manual command 创建时从认证上下文固定 `execution_actor_id`，之后每次 admission/retry/resume 都复验该 actor 的当前权限。客户端不能指定该字段；改变或伪造 `schedule_attribution` 不得改变 Manual 授权身份或绕过撤权。 |

## §8 已知缺口与状态

- Project owner 选择已关闭：Star 原生 Project owner 负责 identity、durable member authorization 与 association。`domain-project` 是现有 Project 领域复用边界；当前持久 writer/API/schema/部署接线仍未实现，不能把 in-memory adapter 当生产事实。
- local-only Repository 与 trusted Host root 的 identity confirmation 尚未实现。
- RunRepository owner binding command 尚未实现。
- 当前 Rust/API/SQL RunInfrastructureBinding 与 demand 仍为 V2，强制 `branch_id`；本版本 V3 字段与迁移仅为设计目标，未改代码、schema 或目标数据库。
- 当前 V2 demand consumer 还要求 Schedule origin 并校验 rule/revision/occurrence；当前持久字段使用 `automation_rule_id`、`automation_rule_version`、`automation_occurrence_id`。V3 增加 Manual tagged origin 是设计目标，尚未实现，不能暗示当前可直接以 Manual Run 启动 Infrastructure demand。
- 持久/受保护的 Host root registry 与独立 read-only provider 尚未实现。
- `DD-LOCAL-INFRASTRUCTURE-001` 的 branch scope 尚未同步：原 docs/design 文件在用户 main 工作树处于未提交移动/删除，遵守保留门禁；WBS 状态也待另一 lane 合并后同步。故 W11-A 整体保持 partial/open。
- Neon 与 Multipass VM 数据库的版本、区域、连接池/代理、备份恢复和部署凭据尚未冻结；两套目标 PostgreSQL 的应用角色 grants、RLS、并发撤权、跨 owner Outbox、路径隔离及 Host adapter 均未运行验收。
- 当前 Worktree 写操作继续 fail closed；本 SRS 是设计目标，不代表可用功能。

## §9 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.1 | 2026-10-25 | 建立 Project↔Repository、host-independent Run、Run binding、Host root generation、只读 Worktree discovery、隐私/资源约束与验收基线。 |
| v0.2 | 2026-10-26 | 关闭 Star 原生 Project owner 与 Vercel→Neon、K3s→Multipass VM 数据库选择；规定同 Project 迁移/显式退役后才能 CAS 解绑，并同步验收条件。 |
| v0.3 | 2026-10-27 | 分离 host-independent Run identity、可选 RunRepositoryBinding 与可选 Infrastructure placement；定义 V3 schema/digest/replay 需求并记录实现仍停留在 V2 的缺口。 |
| v0.4 | 2026-10-27 | 定义 V3 Schedule/Manual tagged trigger origin、可选 schedule attribution、TaskExecutionRun attempt 唯一性与 digest/replay；明确 V2 仍是 Schedule-only 且 Manual 需求 fail closed。 |
