# WBS-WLD1 Project→Repository 与 Host Worktree 边界审查

- **审查日期**：2026-10-25
- **审查基线**：`dev` 候选基线 `2fda1e16632470312ef07b18fb5bf81066822466`
- **审查类型**：只读、定向源码与迁移静态审查；不代表运行时部署、目标 PostgreSQL 或产品验收
- **结论**：WLD1 仍未闭合。Repository tenant master 与 Project/Run 授权各有局部实现，但 Project→Repository 规范关系、Host-owned trusted root registry、Run→Repository 管理命令和独立只读 Worktree provider 仍缺少生产闭环。继续保持 Worktree 写操作 fail closed。

## §0 目的与边界

核实 WBS 指定的 Project→Repository 授权 / 主关系、受信 Host 根目录注册、Run 范围只读发现，以及目标 PostgreSQL/RLS/应用角色证据。审查仅限本 lane 的分配文件，不改代码、设计文档或 WBS，不运行测试，不提交、rebase、merge 或清理 worktree。

CodeGraph 对本 worktree 返回“无 `.codegraph/` 索引”；按审查 brief 的有界源码回退规则，仅检查 `group_api/worktrees.rs`、Run/lifecycle API、domain-scm、worktree-shared-dir、star-pg-adapter、相关迁移和 WLD1 报告。此范围外未命中不能作为“全仓不存在”的证明。

## §1 已证实

| 边界 | 当前证据 | 结论 |
|---|---|---|
| WBS 对 WLD1 的状态 | `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:324,330` 将 Repository master slice `ab2a69c` 记为已在 dev 历史中，同时明确 Project→Repository、trusted Host root registry、read-only provider 与目标 PostgreSQL/RLS 仍 open；Worktree 写保持关闭。 | 本审查与当前 WBS 状态一致；不能把 master 子切片等同为 WLD1 完成。 |
| Repository master | `db/migrations/2026-10-23-scm-repository-master.sql:22-23,31-36,51-75` 定义 tenant scoped SCD2 `scm.repository_master` 和 append-only event，强制 tenant RLS，提供 SCM owner 最小表/列授权；迁移注释明确 Project authorization links、Run bindings 和 host paths 在其他 owner 边界，表注释明确不含 Project/Branch FK 或授权。 | 稳定 Repository identity / 元数据已落源码；它本身不授权 Project、Run 或 Host。 |
| Project 与 Run 授权 | `crates/star-api-rest/src/group_api/engineering_runs.rs:376-400` 读取时同时复验当前 Project role binding 与 Run role binding，并以 `FOR SHARE` 锁定。`crates/star-api-rest/src/group_api/worktrees.rs:232-293` 列授权 Project，`295-404` 经 `active_binding` 列 Project 下 Worktree。 | 已存在 Project / Run 授权事实；这不证明 Project 对某 Repository 有授权关系。Project Worktree projection 也不是 Repository root 注册事实。 |
| Project→Repository 规范关系 | master DDL 没有 Project FK；定向搜索的候选 modules 未找到 `ProjectRepository` 持久关系。唯一明显的 Project 与 Repository 联结之一是 `scm.cloud_branch` 的远端 Branch registration（`db/migrations/2026-10-01-engineering-run-directory.sql:72-104`），不能覆盖无远端 Branch 或无 Branch 的 Repository。 | **未闭合**。不能用 Branch 行、Repository 创建者、tenant 同属或 Project membership 推导仓库授权。 |
| Run 与 Repository 当前性 | `db/migrations/2026-10-15-run-host-independent-directory.sql:23-42` 允许 `engineering_run.repository_id`、`branch_id` 为空，并保留 hostless Run；`engineering_runs.rs:608-640` 新建 Run 时明确写入 `(repository_id, branch_id) = (NULL, NULL)`。`worktree_lifecycle.rs:517-525` 写操作会拒绝 Run 未绑定 Repository。 | hostless Run 是有效设计；但当前检查发现创建 Run 后的 Run→Repository 绑定 owner command 尚无证据。不可把 Project 中可见仓库隐式当成 Run 已绑定仓库。 |
| Run→Worktree binding 与 Repo binding 区分 | `crates/star-api-rest/src/group_api/run_worktree_bindings.rs:320-360` 的 mutation 绑定/解除的是 Worktree，先校验 Project scope 与 Worktree 当前 Project binding，再校验 Run 权限；它不是给 Run 选 Repository 的命令。 | 两种关系不能混称。实现 Run→Repository assignment 前须定义其独立 owner、CAS、幂等与撤权语义。 |
| 只读发现 API seam | `crates/star-api-rest/src/group_api/worktree_lifecycle.rs:327-358` 的 `RunWorktreeLifecycleProvider` 将 `list_project_repositories`、`list_import_candidates` 与 `create`、`import` 放在同一个 trait；`430-453` 的 Run route 先授权再交给 provider；`517-532` 对未绑定 Run 的 mutation fail closed、provider 缺失返回 unavailable。`group_api.rs:338-359` 默认 provider 为 `None`。 | API seam 有授权与 fail-closed 基线；没有已安装的生产只读 provider。读与写 capability 混在 trait 中，建议本阶段拆开 read-only capability，防止装入只读 provider 时一并开放写能力。 |
| 旧 ExternalWorktreeImport 注册表 | `crates/worktree-shared-dir/src/external_worktree_import.rs:531-587` 的 `ExternalWorktreeImportRegistry` 是 `RwLock<HashMap<Uuid, PathBuf>>`，公开 `register(repo_id, path)` 直接收路径，且仅进程内保存。该文件 `625-633` 的真实 Git adapter 同样持有它；`694-735` 的 `scan` 只读 Git worktree list，但 `737-760` 的 `import` 仅把路径写入进程内 `managed` 集合并返回随机 placeholder ID。trait 同时含 `scan/import/cleanup_stale`（`193-209`）。 | 这是旧导入接口与内存状态，不是持久、Project/Run 授权的 Host-owned root registry，也不是可用于生产管理的 read-only provider；不得复用为授权或耐久状态证明。 |
| 数据库运行证据 | `crates/star-pg-adapter/tests/scm_repository_master_postgres.rs:482-485` 的 PG 断言为 ignored，要求 disposable PostgreSQL 与非 superuser owner member。WBS `:324` 也记 PostgreSQL runtime、target RLS/grants 未验证。 | 迁移静态定义了部分 tenant FORCE RLS/SCM owner grant；没有当前目标库的 schema、应用 role membership、RLS、权限漂移、回放与并发执行证据。 |
| 相关工作区 / lane 观察 | 本次只读 `rtk git worktree list` 中，除本审查 lane 与 dev checkout 外未见名称含 WLD1 的其它本地 lane；可见的 W8 候选审查 worktree 与本审查目标不同。 | 只说明当时本地 worktree-list 视图；不能代替 MAGOS manifest / 活动 owner 复核，也不能证明没有外部或远端 lane。 |

## §2 未证实与运行时未知

以下项目当前没有足够证据，不能标为通过：

1. 目标 PostgreSQL 是否已应用 master / directory 迁移、迁移重复执行行为、真实登录 role 的成员关系及非 superuser 运行时权限。
2. 新建 Project→Repository owner 边界后，其 RLS 是否阻止跨 tenant / Project 读取；API app role 是否仅能经 owner 命令访问；owner 是否不能写其他 owner 的表。
3. Run 未绑定仓库时，Run 详情的“可选 Repository picker”与“该 Run 已绑定 Repository 列表”是否在 API / UI / 授权上清楚区分。当前 route 的 `repository_id: Option<Uuid>` 仅是 provider 输入，不构成绑定。
4. 任意本地 Host 的根路径是否经 canonicalization、软链接 / mount / symlink 替换、目录身份漂移和 host generation 校验；是否保证路径不来自 Agent、Plugin、CLI 或请求输入。
5. 只读 Git provider 的资源上限、取消、超时、进程参数、脱敏 DTO、输出截断和多用户并发行为。
6. owner 事务的 CAS、请求幂等重放、撤权竞态、Outbox 交付 / 恢复与 BI 事实的目标 DB 证据。
7. 当前 API app-role grants 是否与静态迁移保持一致；SCM owner 权限是否发生非 SCM schema / table privilege drift。

## §3 差距与安全边界

- **P0 — 缺 Project→Repository canonical relation**：建立 Project 范围内的持久关联事实前，必须先拍板权威关系的基数和变更语义。当前代码的 `ProjectPolicy.default_repository_id: Option<RepositoryId>` 只表达可空的默认 Repository 值，现有已批准文档没有把它定义为完整的 Project 授权关系，也没有批准每个 Project 只能有一个关联。`scm.repository_master` 只负责 tenant-scoped Repository identity / 元数据，不携带 Project 关系或授权语义。以下两种方案及其代价、待拍板问题见 §7.1；本审查不替产品决定基数或撤销策略。
- **P0 — 缺 Run→Repository binding command**：由 Run owner 实施单独的绑定 / 解除命令，校验目标 ProjectRepository 关系并记录 binding revision。绑定需 CAS + 幂等；每次 Worktree read / mutation 前重新检查当前 Project 与 Run grants 以及当前关联。撤销 ProjectRepository 后既有 Run 不能继续访问 root。跨 owner 不共享写事务、不直接写对方表；使用 owner API / 命令及可核验的关联版本引用。
- **P0 — 缺 Host-owned trusted root registry**：由可信 Rust Host/Manager 管理稳定 `HostId + RepositoryId + generation` 到本机 canonical root 的映射。路径不可由 API、Agent、Plugin 或任务卡 CLI 提供/改写；API 与事件仅传 opaque repository/root identity 和 generation，不返回绝对路径或凭证。root 过期、重复、解析不一致、Host 离线或 generation 不符均 fail closed。
- **P1 — 缺隔离只读 Worktree provider**：新增只读 trait / adapter，只实现受限的 discovery/list；Run scope 必须有唯一当前 Repository binding 后才可查询对应 root。provider 仅运行有界、可取消的 Git worktree listing，并返回路径脱敏、字段限量的投影。将它与 `create/import/archive/cleanup` 等写能力分开，provider 缺失继续 503；本阶段不得开启任何写侧 lifecycle。
- **P1 — PostgreSQL / RLS / grants 没有部署验收**：静态 `ENABLE/FORCE ROW LEVEL SECURITY` 不足以证明 target role 正确。需要目标环境 disposable DB 上的 owner/app 登录角色、成员关系、跨 tenant 与跨 Project 探针、grant fingerprint、回放/并发和事务回滚证据；当前审查不连接 DB。
- **P1 — master slice 有已知审查债务需再核对**：`PHASE-WLD1-REPOSITORY-MASTER-REVIEW-20261024.md:41,44-48,66-69` 记录旧候选的单独关闭 / role privilege drift / disposable test target guard / `Cargo.lock` 不匹配问题。当前 WBS 已记录 master 实现进 dev，但这份历史审查基于旧候选基线；集成前须在最新 dev 对每项复查是否已修复，不能直接把旧 findings 当成现状，也不能当作已解决。

## §4 推荐 owner / 数据责任矩阵

| 事实或操作 | 唯一写 owner | 读取与授权要求 | 禁止边界 |
|---|---|---|---|
| Repository identity / provider metadata | SCM Repository master owner | tenant scope；该事实本身不授权用户 | 不把 Project/Run/Host path 写进 SCM master |
| Project↔Repository active association | 建议由 Project Directory / Project authority owner 管理，仍需明确拍板 | 当前 Project membership / manager grant；独立关联 revision 与事件 | SCM owner、Run owner、Host manager 不可直接写此关系；`default_repository_id` 是否同时承担此事实尚未决定 |
| Run↔Repository current binding | Engineering Run owner | 写时验证 active Project↔Repository；读 / Git 操作前复验 Project + Run grants 与绑定 currentness | 不复用 Run↔Worktree binding 表表示仓库绑定；不跨 owner 写表 |
| Host↔Repository canonical local root | Rust Host Infrastructure Manager owner | `HostId + RepositoryId + generation` 精确 currentness；本机 capability 限制 | Agent / Plugin / CLI 不获得任意 host path、shell、daemon/socket |
| Worktree discovery projection | 独立 Read-only Host/Run Worktree provider | 只接受已授权的 Run + 已绑定 Repository opaque identity；受限 Git list 操作 | 不创建、导入、删除、归档 Worktree；不暴露本机绝对路径 |
| 异步 owner 通知 / BI | 各 owner 的 transactional Outbox；对应 consumer 是自己的 Inbox owner | 事件只携 identity、revision、correlation / idempotency 信息；接收事务先提交再确认 | 不把跨 owner direct SQL 或单个共享事务当作集成 |

## §5 最小依赖 DAG 与互斥文件建议

以下为下一轮 dispatch 建议，路径为候选归属而非已预留。派发前由协调者先做 MAGOS manifest / 当前 owner / 路径冲突校验；共享入口文件与 workspace 配置必须串行集成。

```text
WLD1-0 边界合同与 owner 决定
   ├── WLD1-A Project↔Repository canonical association owner
   ├── WLD1-B Host root registry (stable Repository identity only)
   └── (A 完成后) WLD1-C Run↔Repository binding owner command
          └── (A+B+C 完成后) WLD1-D 独立只读 Worktree discovery provider
                  └── WLD1-E target PostgreSQL/RLS/app-role + route evidence
```

| Lane | 依赖 | 候选文件所有权（互斥设计） | 主要交付 |
|---|---|---|---|
| WLD1-0 Contract | 无 | 仅 WLD1 SRS / basic design / detailed design / owner matrix 新增或指定章节；不改迁移与 Rust 源码 | 定义 association semantics、hostless Run picker 语义、owner/Outbox、root identity/generation、只读/写分界、RLS/app role 与验收；冻结后再开代码 lane。 |
| WLD1-A ProjectRepository | WLD1-0 | Project authority owner 的新 relation/domain/API/PG adapter 文件 + 一份新的 dated migration + 一份独立 DB adapter 集成测试；不碰 Host registry、Run lifecycle provider 或共享 router 注册 | SCD2/active link、权限版本、审计/Outbox、明确授权 CRUD/query；Repository master 只作 FK identity 引用。Owner crate 路径需先由 DDD 当前映射决定，勿凭空新建第二个 Project owner。 |
| WLD1-B Host root catalog | WLD1-0；只依赖稳定 RepositoryId 契约 | Host Manager owner 的独立 root-catalog 模块 / 本机安全存储 adapter / migration（如采用同一 PG）/ 自有测试文件；不触碰 ProjectRepository 或 Run API 文件 | root identity/generation、canonicalization、无请求路径输入、最小 DB/file ACL、无敏感输出；可以与 A 并行，因为只引用 Repository identity，不拥有 Project 关联。 |
| WLD1-C RunRepository binding | WLD1-A；Run owner 与 A 的跨 owner 读取合同冻结 | 独立新增 `group_api/run_repository_binding.rs` / Run owner port / 独立 migration 与 adapter 文件；不修改 A 的关系 owner 文件，也不改 `run_worktree_bindings.rs` 的 Worktree 绑定语义 | Run repository bind/unbind CAS、幂等、事件、Run grant & ProjectRepository 当前关联校验；hostless Run 可保持无绑定状态。 |
| WLD1-D Read-only provider | A+B+C；若 provider 输出不需绑定则需额外显式 UX/API contract | 新增 `run_worktree_read_provider.rs`、Host read adapter、自有 DTO/测试；不直接扩写 `worktree_lifecycle.rs` 混合 trait，不开放写 provider | 以唯一当前 Run binding 为输入做受限只读扫描，路径脱敏、限量、超时、取消；无 provider/root/binding 均 fail closed。 |
| WLD1-E Integration gate | A+B+C+D | 独立 target-PG 测试文件 / environment manifest / 审计报告；共享 route registration、Cargo workspace / lock 和最终文档由协调者串行集成 | 使用 disposable 非 superuser owner/app roles，验证 migration replay、RLS、授权撤销、跨租户/Project 隔离、路径泄露、provider 503、恢复/超时；实际运行结果单独登记，不用 fixture 代替部署证明。 |

**共享文件串行约束**：如需注册新 crate、在 `group_api.rs`/router 聚合入口挂载新 router、更新 `Cargo.toml`/`Cargo.lock` 或同步总 WBS/主设计索引，应由协调者按 A→B→C→D 的依赖顺序集成；并行 lane 不编辑这些共享文件。任何路径重叠须在 MAGOS manifest 中重新分配，不用临时 cherry-pick 来掩盖所有权冲突。

## §6 验收矩阵

| 门 | 正向验收 | 反向 / 失败关闭验收 | 证据等级 |
|---|---|---|---|
| ProjectRepository | 产品先确认 §7.1 的关联模型；随后验证 Project manager 可按已批准模型添加/读取/移除关联、active association 指向现有 Repository master revision、无 Branch Repository 按模型列举 | 普通成员不能管理关联；关联撤销后 API 不再授权；跨 tenant / Project 关联不可读写；重复请求精确回放、重用 idempotency key 更改内容冲突；Run 已绑定时按已批准的阻止或撤权规则处理 | 当前是待定契约，不是通过项；后续需 owner 单测 + PG owner/app role/RLS route integration，target DB 实证未取得前不得关闭 |
| RunRepository | hostless Run 可以无仓库存在；Project manager 绑定当前已授权 Repo 后形成新的 binding revision；读写 Run grant 正确分开 | 非 ProjectRepository 的 Repo、撤销 grant、Run grant 撤销、expected revision 过期均拒绝；一个 Run 不会被 Project 的其它 Repo 隐式查询 | Run owner command + PG CAS/RLS + Run route integration |
| Host root registry | 受信 Host Manager 注册稳定 Host/Repo/generation；合法 Repo root 能以 opaque identity 解引用 | Agent/Plugin/用户直接提交 path 一律拒绝；symlink/canonical root mismatch、generation stale、路径不存在、Host offline 返回 unavailable；API/event/log 不泄露路径或 credential | Host adapter security tests + target role/ACL probe + sanitized API evidence |
| Read-only Worktree discovery | 已授权且唯一绑定的 Run 返回有界 Git worktree DTO | 无 binding/授权撤销/root stale/provider absent 超时或取消时无文件写/无 shell 副作用；调用者无权读取其他 Project 的 worktrees；响应不含 absolute path | provider contract + controlled host integration + auth/revoke route tests |
| DB & operational readiness | 两次迁移、owner transaction、Outbox/Inbox 和目标角色实际行为可复现 | migration 非 disposable/身份不符时 preflight fail；OWNER 不可跨 schema 写；app role 不能绕 owner；rollback 后无半完成 binding/root 注册 | 当前未运行。必须记录目标实例 identity、role grants、schema diff 和实际 query probe 才能关闭 |
| Worktree write safety | 本阶段不接受写侧 Worktree 可用声明 | 未验收 lifecycle provider / Host action fencing 前，create/import/archive/delete 始终 unavailable | API 当前默认 provider `None`，但实际部署行为仍需 runtime route evidence |

## §7 验证摘要、已知缺口与修订历史

- 本次执行的是定向静态源码、DDL、既有 WLD1 报告与只读 worktree 清单检查；未运行测试、编译、格式化、migration 或 PostgreSQL/RLS 命令。
- CodeGraph 在本 lane 不可用（没有索引）；源码范围和负向结论均按 §0 限定。`rg` 的 bounded module 查询不是全仓 absence proof。
- 当前阻断项：ProjectRepository association owner/API/DDL；Run→Repository command；Host-owned durable root registry；独立只读 provider；target PostgreSQL / RLS / app-role 运行证据；共享入口的合并门禁。
- **集成建议**：仅将这份审查作为下一 WLD1 实施 lane 的输入；不据此关闭 WLD1，不安装 provider，也不打开 Worktree 写操作。
- **修订历史**：v0.1（2026-10-25）— 按 WBS-WLD1-BOUNDARY-REVIEW brief 对 dev 候选基线进行静态边界审查，补充 verified/unknown 矩阵、owner 数据责任、互斥实施 DAG 与验收矩阵。
- **修订历史**：v0.2（2026-10-25）— 根据 WLD1 Project→Repository 只读审查补充 `default_repository_id` 与 `scm.repository_master` 的语义边界、关联策略影响及明确待拍板问题；未新增 schema/API，也未决定 cardinality。

### §7.1 Project→Repository 关联策略与待拍板问题

本补充核验基于隔离 lane 的 dev 基线 `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10`；它扩展当前 dev 已跟踪的 `docs/reviews/WBS-WLD1-BOUNDARY-REVIEW-20261025.md`，不另建重复审查文件。

**已核实的现状**：`crates/domain-project/src/lib.rs:156-180,206` 中 `ProjectPolicy` 有 `default_repository_id: Option<RepositoryId>`，且默认值为 `None`；`domain-project-spec.md:46` 也只把该项列为可选 SCM 默认值。`ProjectRepository` trait（`crates/domain-project/src/lib.rs:408`）负责 Project 记录 / policy 持久化，不是单独的 Project↔SCM Repository 授权关系。`db/migrations/2026-10-23-scm-repository-master.sql:22,31,74` 将 `scm.repository_master` 定义为 tenant-scoped Repository identity 与元数据，并明确 Project authorization links 在其他 owner 边界、没有 Project FK，且 actor provenance 不授予访问权限。因此，不能从字段名、同 tenant、创建者或 Repository master 推导当前 Project 对 Repository 的可见或使用授权。

| 方案 | 关系基数与现有字段解释 | 主要影响 / 风险 |
|---|---|---|
| A. 将 `ProjectPolicy.default_repository_id` 作为唯一 Project↔Repository 关联 | 每个 Project 为 0 或 1 个 Repository；该字段同时成为 canonical association 与默认选择。 | 不需要新增关联事实，但会把“默认配置”升级为“成员关系 / 授权事实”：修改 policy 即可能授予或撤销仓库访问，必须纳入相应授权、审计、revision、tenant/Repository identity 完整性及 Run 绑定并发规则。无法表达一个 Project 管理多个 Repository 的情形。若字段仅被当作 picker 默认值，则它仍不能提供授权允许集合。 |
| B. 建立由 Project owner 管理的显式关联，并保留 `default_repository_id` 为默认选择 | 可批准为每 Project 0..N 个关联，或另行确定其他基数；`default_repository_id` 仅指向当前关联集合中的一个默认 Repository，不作为授权真相。 | 明确分离默认值与授权成员关系，能支持多仓 Project；需要独立关联事实及其 owner-local revision、审计、授权、RLS、幂等与 Outbox，并定义默认值必须属于 active association 的一致性规则。`scm.repository_master` 仍只保留 SCM identity/metadata，不能由它直接承载 Project 授权。 |

**待产品拍板（实现前置条件）**：

1. 一个 Project 的 Repository 关联应为 **0..1**，并让 `ProjectPolicy.default_repository_id` 同时承担唯一关联和默认选择；还是允许 **0..N**，另建 Project-owned canonical association，而保留该字段只作默认选择？当前源码与已批准文档不足以替产品选择其中一个。
2. 若 Repository 已被一个或多个 Run 绑定，Project 管理员移除其关联时，应 **阻止移除，直到相关 Run 先解除或重绑**；还是 **允许立即撤销关联授权**，并保留历史 Run binding 作为审计事实、令依赖该关联的后续 Worktree discovery 与所有 mutation 立即 fail closed？两种策略都必须处理并发读取/撤销并保留历史证据，不能静默继续访问。
3. 是否确认 Project Directory / Project authority owner 是 Project↔Repository active association 的唯一写入 owner，而 SCM owner 仅管理 `scm.repository_master` identity/metadata、Run owner 独立管理 Run↔Repository binding？现有边界报告建议此职责拆分，但当前仍应由 WLD1 contract 明确批准。

在上述问题获得决定前，不新增 association schema/API，不实现 Project Repository authorization/list provider，也不将 `default_repository_id` 解释为仓库访问许可；只读候选列表若没有独立、明确授权的数据事实，同样必须 fail closed。
