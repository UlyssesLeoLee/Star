# BD-WLD1-REPOSITORY-BOUNDARY-001

> 状态：Draft v0.4；补充 Schedule/手动触发来源和 attempt 幂等设计，尚未落库或部署。
> 需求：`SRS-WLD1-REPOSITORY-BOUNDARY-001.md` v0.4。
> 详细设计：`DD-WLD1-REPOSITORY-BOUNDARY-001.md` v0.4；验收：`TD-WLD1-REPOSITORY-BOUNDARY-001.md` v0.2（待同步）。

## §0 设计目标和约束

建立 Project 对 Repository 的授权关联、host-independent Run identity、可选 Run Repository binding、可选 Infrastructure placement、Host 本地 root registry 和只读 Worktree discovery。目标导航依然为 **Project → Engineering Run → Worktree → Branch**；Repository 不作为主导航父层，Branch 是该 Worktree 的 local ref projection。

Run identity 仅由 tenant、Project 与 Run 构成；创建、授权、列表及基本管理不读取远端宿主 Branch。远端 Repository/provider/Branch provenance 是可选版本化元数据，不是 Run 父级或授权前提。Repository master 只证明身份；Project association 才表达 Project scope；RunRepositoryBinding 表达可选源码发现资源；RunInfrastructureBinding 仅在显式选择 placement 时表达环境选择。任一层的身份不能替代其他层的授权。

**本阶段只更新授权、只读发现与跨 owner capacity demand 设计，不启用执行或 Worktree 写入。** `create/import/archive/delete/cleanup` 及任意 Git mutation 继续关闭。

## §1 逻辑上下文

```text
Star-native Project Owner (reuse the existing Project domain boundary)
  ├── Project identity + durable membership authorization (M; versioned history)
  └── ProjectRepositoryAssociation (T; 0..N links + append-only audit/outbox)
          │ authorized owner query / versioned association reference
          ▼
Engineering Run Owner
  ├── Run identity (tenant + Project + Run; no host Branch dependency)
  ├── optional RunRepositoryBinding (T; explicit Repo+HostRoot generation refs)
          │ opaque Run binding + current grant proof
          ▼
Read-only Run Worktree Provider ──authorized command──> Host Infrastructure Manager
                                                           └── HostRootRegistry (M)
                                                           └── restricted Git list adapter
  └── optional RunInfrastructureBinding (T; Profile + exact HostEnvironment/K3s generations)
          │ only for executions requiring this placement
          ▼
      Infrastructure owner (owner-local Inbox/Outbox; capacity/readiness gates)
          │ sanitized page DTO; no path/credential
          ▼
Run UI: Worktree rows → local checkout Branch/ref state
```

异步关系变更与 BI 只通过各事实 owner 的 Transactional Outbox 和消费方 Inbox 传播；不跨 owner direct SQL、不共享跨 owner write transaction。Outbox/delivery adapter 为 transport-neutral 契约，业务事实不依赖特定 broker subject 或 ACK API。Project Authority 已确定为 Star 原生 Project owner；实现复用现有 `domain-project` Project 领域端口，不创建第二个业务 owner。现有 `ProjectRepository` 持久化抽象可作为适配边界；生产 PostgreSQL writer/API 必须真实持久化，不能使用 in-memory adapter 充当业务真相。

## §2 事实所有权

| 事实 | 唯一 write owner | 数据分类 | 授权 / 边界 |
|---|---|---|---|
| Repository stable identity/provider metadata | SCM Repository master owner | Master | tenant scoped；不授权 Project、Run 或 Host |
| Project identity 与成员授权关系 | Star-native Project owner | Master | principal ID 由身份 owner 提供；Project owner 持有 membership/capability 与可追溯 revision；RLS/SCD2 |
| Project↔Repository current association 与历史变更 | Star-native Project owner | Transaction | Project manager grant；CAS、审计、Outbox、0..N 关系；retiring fence 保持既有 binding 有效 |
| Project default Repository UI preference | Star-native Project owner | Master | 可空；只引用 active same-Project association；只用于 UI 排序/预选，不参与授权或 Run binding |
| Run↔Repository/HostRoot 当前 binding 与版本 | Engineering Run owner | Transaction | Project + Run grant；新绑定只选 active association；retiring 时既有 binding 只可按迁移流程继续解析 |
| Run↔Infrastructure placement 与版本 | Engineering Run owner | Transaction | 可选；仅用户显式选择时存在；固定 Profile digest 与 HostEnvironment/K3s cluster ID+generation；没有 placement 不代表创建失败 |
| Host↔Repository root/generation 映射 | Trusted Host Infrastructure Manager | Master | Host-local protected store；路径不离开 Host 信任区；generation SCD2 |
| 受限 Worktree discovery snapshot/cache | Read-only provider | Work | 按 binding/root generation 有界短 TTL；可重建，不具授权意义 |
| Audit 与 Outbox/Inbox 交付事实 | 各源/消费 owner | Transaction | append-only、幂等、最小 payload、owner-only grants |

> 分类遵循 W/T/M 主分类规则：Master 不物理删除并保留 generation 变更历史；Transaction 不物理删除业务/审计事实；Work 明确短 TTL 且可重建。实体部署前需将每张物理表 100% 映射至 W/T/M 并补齐 RLS、retention 与 owner grants。

## §3 命令与读取流程

### 3.1 Project association

1. Project Manager 提交 Repository ID、expected Project association revision、idempotency key。
2. Project owner 验证 actor/tenant/Project grant 与 Repository master tenant identity。
3. Attach 在同一短事务创建 association current projection、append-only audit、Outbox 与幂等 receipt；unlink 先 CAS 为 retiring，最终 revoke 必须走 §3.4 零引用门。
4. 重复同键同 request digest 回放同一 receipt；同键不同 digest 拒绝；CAS stale 返回当前 revision 摘要。
5. Run owner 的 binding 命令读取 Project owner 给出的当前 association revision 事实；不得共享事务。

### 3.2 Run binding

1. Run 可仅以 tenant + Project + Run identity 创建、授权、列表和基本管理；远端 Branch 不参与这些操作。
2. 需要 Worktree discovery 时，Run Owner 再显式选择 Project 已关联 Repository 与目标 Host root，形成独立 RunRepositoryBinding。
3. 需要固定执行环境时，Run Owner 单独选择 Infrastructure Profile 和 placement，形成 RunInfrastructureBinding；它与 Repository binding 不互相推导。
4. Run owner 在线复验 actor 的 Project 与 Run grant、association/root 或 Profile/HostEnvironment/K3s generations；以各自 expected revision 执行 CAS，并在同一 owner 事务写审计、幂等 receipt 和 Outbox。
5. 每次 discovery 或 execution admission/resume 再验证相应 binding。没有 placement 的 Run 不发 Infrastructure demand；要求特定环境的执行缺少有效 placement/readiness 时 fail closed。
6. 无 Repository/Host 的 Run 返回“尚未绑定 Repository/Host”状态，不从 Project 自动选择或列举私有 root；无 placement 是合法状态。

### 3.3 Host root registration and read-only discovery

1. 可信 Host Manager 由本机 operator flow 获得 root handle，不接受远端 path 字符串。
2. Manager canonicalize root 并检查文件系统对象身份。若 Repository 有受信 remote identity，可校验该身份；local-only Repository 则必须取得 Project 授权操作者对 `(RepositoryId, HostId, root)` 的明确确认。只把确认来源、RepositoryId、root opaque id 与 generation 写入本机 registry；不得根据目录名或同租户推断关联。root relocation/replacement 递增 generation。
3. Run provider 把已授权 binding 的 opaque HostId/RepositoryId/root id/generation 发给 Host Manager，命令带 correlation/idempotency/deadline/cancel。
4. Host Manager 精确匹配 generation，验证持有 root handle 的目录身份仍一致，再执行受限只读 Git worktree listing。
5. 只回传分页、限字段的 sanitized DTO：Worktree opaque ID、state/lock/detached 标记、本地 ref label、ahead/behind 若已在授权合同内可读、更新时间与 cursor。绝对路径、远端 URL 中的敏感部分、原始 stdout/stderr 不出 Host。
6. root/host unavailable、输出/条数超限、timeout、cancel 或 schema 错误时返回原因码并结束操作；不扫描其他根、不创建/删除任何内容。

### 3.4 ProjectRepository unlink / reference migration

1. Project owner 以当前 association revision 建立 `retiring` admission fence，拒绝新增 Run binding；旧 association 对已有 binding 仍保持有效。
2. Run owner 按各自事务列举 source binding，并逐项迁移到同一 Project 内当前 active 且已授权的目标 association，或在明确不再需要 checkout 后由对应 owner 显式退役 binding。Project owner 不跨 owner 写表。
3. 每个 owner 提交本地 binding/audit/Outbox 与幂等完成 receipt；Host/Worktree owner 验证目标 root generation 或确认 checkout 已按生命周期规则退役。
4. Project owner 只接受新鲜完整的 owner receipts 和 revision-aware reconciliation；确认零活动 source reference 后，使用 CAS 将 association 从 retiring 转为 revoked。
5. 任一迁移失败、超时、撤权、owner 响应缺失或 revision 变化都保留 source association 与既有 binding；不得报告完成或 unlink。

### 3.5 V3 trigger origin and execution attempts

V3 capacity demand 明确覆盖 Schedule 和手动启动的 TaskExecutionRun。`trigger_origin` 是互斥 tagged union：`schedule` 要求 `schedule_rule_id + schedule_rule_revision + occurrence_id`；`manual` 要求 Run owner 基于已认证命令分配并固化的 `manual_command_id`。Manual 可选携带已证明的 `schedule_attribution` 以表示从哪次 Schedule execution 发起重跑；该 provenance 只作因果记录，不能改变 Manual 身份、授权主体或 run-as 策略。缺少字段用 variant 结构表达，不写 nil/全零 Schedule UUID。

Schedule origin 以 `(tenant, schedule_rule_id, schedule_rule_revision, occurrence_id)` 固定触发身份；调度重复投递回放同一触发。Manual origin 以 `(tenant, manual_command_id)` 固定显式命令，重复提交同一 command 回放原结果；新的人工启动使用新的 command ID。`task_execution_run_id` 对每个执行 attempt 唯一，`attempt_number` 为该 origin 下正数且唯一的递增编号；同一 attempt 的 transport retry/resume 复用 attempt 身份与幂等键，创建新 attempt 必须分配新 TaskExecutionRun ID 和 attempt number。

Schedule attempt 从固定 Rule revision 解析 `run_as_actor_id`；Manual command 从创建时的认证上下文固定 actor。Run owner 写入不可由客户端选择的 `execution_actor_id`，并在每次触发、准入、重试/恢复时复验该 actor 当前 Project/Run 权限。Manual 中的 schedule attribution 不替代任何授权事实。当前 V2 consumer 只接受 `run_origin=schedule`；未迁移到 V3 前，Manual infrastructure demand 必须拒绝。

## §4 授权与撤权传播

- API 请求由 Project/Run owner 分别在自己的事务边界授权；Host Manager 只接受带完整授权 binding/version 的受信 command，不信任客户端主体 ID。
- Project association 进入 retiring 后拒绝新的 Run bind；既有 binding 在 Project/Run 当前授权仍有效时保持可解析，直至完成迁移或 owner 退役。最终转为 revoked 后所有关联 binding 读取拒绝。撤权事件唤醒相关授权缓存失效；最终正确性由 owner 在线复验保障，不依赖异步缓存时效。
- Run grant revoke、Host disable、root generation change 同样使旧 binding 失效。终止活跃订阅/发现请求，拒绝旧 cursor replay 与缓存结果。
- Worktree row 的 opaque ID 与 cursor 只定位对象/页面，不是 credential。单个 row 的 details/action 再次授权，写操作路由在本阶段 unavailable。
- 任何 owner 无法访问授权事实、授权版本不一致或无法验证签名/Host 身份均 fail closed。

## §5 持久化与事件契约

| Owner | 目标记录 | 分类 | 关键约束 |
|---|---|---|---|
| Star-native Project owner | Project identity + Project membership/capability history | M | principal identity 外部引用；SCD2、tenant/Project RLS、当前授权查询 |
| Star-native Project owner | `project_repository_association` current rows + append-only association audit/outbox | T | tenant/project/repository composite scope；expected revision；active/retiring/revoked；撤销保留历史 |
| Star-native Project owner | optional `project_default_repository_preference` | M | 可空、只能指向 same-Project active association；不派生 Run binding |
| Engineering Run | `run_repository_binding` + append-only audit/outbox/idempotency receipt | T | Run 允许零绑定；多个 repo 绑定显式列出；每项含 Project association revision 与 Host/root generation |
| Engineering Run | `run_infrastructure_binding` + append-only audit/outbox/idempotency receipt | T | 仅显式 placement 时有记录；包含 tenant/Project/Run、Profile revision+digest、HostEnvironment ID+generation、K3s cluster ID+generation、namespace、runtime profile 与预算 |
| Run/Execution owner | V3 demand + result Inbox | T | Schedule/Manual tagged origin + owner-derived `execution_actor_id`；可选非授权 schedule attribution；每个 `task_execution_run_id` 唯一标识一个 attempt，V2 不重写 |
| Host Infrastructure Manager | `host_repository_root` generation history + local protected path handle | M | local protected store；opaque root id；不在中央事件持久化路径 |
| Read-only provider | paged snapshot/cache + cursor | W | 明确 TTL、size cap；按授权版本/root generation 隔离；可完全重建 |
| Consumer owner | inbox/processed-event receipt | T | 接收事务先提交后确认；`event_id + schema_version + tenant + owner + sequence` 幂等 |

生产 DDL 由后续实现 lane 根据实际 owner crate、schema、role 与迁移链补齐；当前表名是逻辑目标，不代表已部署。

### §5.1 部署数据库拓扑

| Deployment profile | Project owner database | Boundary |
|---|---|---|
| Vercel Web profile | Neon PostgreSQL | Project owner server-side runtime 访问 Neon；浏览器与 Web BFF 不直接持有 DB 凭据或写表权限。具体 owner runtime 的部署位置与连接代理仍待实现设计确认。 |
| Multipass/K3s profile | Multipass 管理的 Linux VM 内 PostgreSQL | Star API/owner runtime 经受控网络访问；测试阶段 Multipass inventory 总实例数最多 2，建实例前必须检查 inventory，禁止自动 purge。 |

两种 profile 使用同一版本化业务契约与迁移源，但数据库实例、凭据、审计和数据集相互隔离；不存在隐式复制、跨部署事务或跨库 RLS 证明。Neon 项目/分支、区域、数据库版本/连接池，以及 K3s VM 放置、容量/备份与实例分配均保持 OPEN。

## §6 资源、可靠性和可观测性

- 每个 Host/Run binding 有独立并发预算、队列上限、公平调度、deadline、取消与 drain；默认一项 binding 同时最多一个 discovery 子进程。
- Infrastructure demand 与幂等身份按 wire/schema version 隔离。V3 将 trigger kind/variant、owner-derived `execution_actor_id`、attempt 身份、optional schedule attribution/remote provenance 的规范化存在性与完整 placement 纳入新 digest 域；V2 replay 继续使用原 V2 canonicalizer，不能跨版本重哈希。
- Schedule origin 固定 rule ID/revision/occurrence；Manual origin 固定 command ID，可带非授权 schedule provenance。相同 origin+attempt 同 digest 重放原 receipt，不同 digest 冲突；新 attempt 使用新 TaskExecutionRun ID 和 attempt number。
- 页大小、单次 git 输出 bytes、实例数、缓存 TTL、最大并发 Host command 由版本化 Host Discovery Profile 固定，实测前不声称容量；超过 cap 返回明确 `capacity_exceeded` 并保留 UI 状态。
- 缓存键含 tenant/Project/Run/binding revision/Host identity/root generation；权限撤销或任何版本变更立即使旧缓存不可见；不得以用户可写本地 cache 作为授权真相。
- API 与 host RPC 使用结构化原因码、trace/correlation；遥测只汇总数量、耗时、输出字节、拒绝码和 generation，不记录路径、credential 或源 stdout/stderr。
- Worktree list 是即时观察/短 TTL projection，不是业务事实库；业务状态变更只能由后续 Worktree owner commands 产生并持久化。

## §7 错误状态和 UI

| 条件 | Run UI 状态 | 服务端行为 |
|---|---|---|
| Run 无 Repository binding | 尚未绑定；提供经过授权的 Project Repo 选择入口 | 空结果，不触碰 Host |
| Run 有 Repo、无可用 Host root | Host 未配置/离线 | 返回 unavailable，不暴露候选路径 |
| binding/association/root generation stale | 需要重新绑定/权限已变化 | 409/403/503 typed error；拒绝旧 cursor/cache |
| Host provider 未安装 | 发现能力不可用 | 503；没有写侧 fallback |
| Git list timeout/cancel/cap overflow | 更新失败/容量受限 | 终止子进程，保持 projection 不变，提供可重试原因码 |
| repository has no worktree / detached HEAD | 正常空/特殊 local ref 状态 | 如实表示，不伪造远端 Branch |

## §8 决策记录与尚未闭合

- Star 原生 Project owner 是 Project identity、durable member authorization 和 0..N Repository association 的唯一业务真相；独立的可空 default Repository preference 仅作 UI 排序/预选。Project principal identity 由身份 owner 提供。
- 每个 Run 可零个或多个显式 Repo binding；每个 binding 固定一项 Host/root generation。SCM master 的 remote/provider 字段可以为空；local-only identity 由 SCM owner 管理并通过本机授权确认绑定，不要求远端 Branch。
- Run 的 Infrastructure placement 是可选独立关系。无 placement 的 Run 正常存在；需要 placement 的执行必须持有当前 Profile digest、HostEnvironment 与 K3s cluster 精确 generation 及 readiness。
- V2 demand/result schema 保持只读兼容；后续 additive migration 为 V3 提供独立存储/索引与 writer/reader，不放宽 V2 的非空 Branch 约束，不重写历史事实。模糊或撤权的旧关联隔离，不合成 remote identity。
- V2 Run owner demand contract 仍是 Schedule-only；其 Schedule rule/revision/occurrence 需对照 TaskExecutionRun 的当前存储事实校验。V3 才开放 Manual tagged origin；不得把 V2 的 `run_origin=schedule` 约束旁路为手动模式。
- V3 origin key 分别为 `(tenant, schedule_rule_id, schedule_rule_revision, occurrence_id)` 与 `(tenant, manual_command_id)`；attempt number 与 TaskExecutionRun ID 唯一绑定。完整 canonical digest 还纳入 tag、origin variant、optional attribution 存在标记/内容、attempt、remote provenance 与 placement generations。
- Repository association 不进入主导航树；Worktree 仍直接属于 Run；Branch 仅反映 Worktree local ref。
- Root 的绝对路径由 Host local protected store 持有；远端保存/交换 opaque identity。若实现阶段不能提供满足此约束的安全 Host store，provider 保持 unavailable。
- Worktree 写操作在本阶段不开放；read-only profile 和 mutation capability 分离。
- Vercel 使用 Neon；K3s 版使用 Multipass VM 内数据库。目标 schema/RLS/grants、owner adapters、部署接线、Host root store、Worktree provider、cross-owner outbox 和端到端运行证据均未实现；目标选择不构成运行验收。
- 活动引用只能迁往同 Project 内已授权的 active association，或由 owner 显式退役不再需要 checkout 的 binding；迁移目标选择、权威引用清点、receipt/evidence 和 CAS/事务细节仍待详细设计冻结。
- V3 owner storage、wire contracts、canonicalizer、version-scoped idempotency、additive migration 与目标 DB/RLS/concurrency 验收均未实现；`TD-WLD1`、`DD-LOCAL-INFRASTRUCTURE-001` 与 WBS 尚未完成同步，W11-A 保持 partial/open。

## §9 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.1 | 2026-10-25 | 建立事实所有权、绑定与发现流程、W/T/M 逻辑映射、权限撤销、低资源与失败状态设计。 |
| v0.2 | 2026-10-26 | 指定 Star 原生 Project owner 与 Neon/Multipass VM 数据库拓扑；补充引用 retiring fence、同 Project 迁移/显式退役和失败保持原绑定的流程。 |
| v0.3 | 2026-10-27 | 明确 Run identity 不依赖远端 Branch；分离可选 Repository 与 Infrastructure bindings；规定 V3 placement/digest 与 additive V2 replay 兼容目标，标记 TD、DD-LOCAL、WBS 和运行验收仍未同步/完成。 |
| v0.4 | 2026-10-27 | 定义 V3 Schedule/Manual tagged trigger origin、schedule attribution、attempt 唯一性与授权复验；明确 V2 仍只接受 Schedule demand。 |
