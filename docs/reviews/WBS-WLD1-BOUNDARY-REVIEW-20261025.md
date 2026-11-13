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

- **P0 — 缺 Project→Repository canonical relation**：建立 Project 范围内的持久关联事实。建议该关系由 Project Directory / Project authority owner 管理；`scm.repository_master` 继续只负责 Repository stable identity 与非敏感元数据，不向其加 Project authorization 语义。明确“一名 Project 可以关联多个 Repository”“移除关联后的 Run 处理”“无 Branch 仓库可见性”。
- **P0 — 缺 Run→Repository binding command**：由 Run owner 实施单独的绑定 / 解除命令，校验目标 ProjectRepository 关系并记录 binding revision。绑定需 CAS + 幂等；每次 Worktree read / mutation 前重新检查当前 Project 与 Run grants 以及当前关联。撤销 ProjectRepository 后既有 Run 不能继续访问 root。跨 owner 不共享写事务、不直接写对方表；使用 owner API / 命令及可核验的关联版本引用。
- **P0 — 缺 Host-owned trusted root registry**：由可信 Rust Host/Manager 管理稳定 `HostId + RepositoryId + generation` 到本机 canonical root 的映射。路径不可由 API、Agent、Plugin 或任务卡 CLI 提供/改写；API 与事件仅传 opaque repository/root identity 和 generation，不返回绝对路径或凭证。root 过期、重复、解析不一致、Host 离线或 generation 不符均 fail closed。
- **P1 — 缺隔离只读 Worktree provider**：新增只读 trait / adapter，只实现受限的 discovery/list；Run scope 必须有唯一当前 Repository binding 后才可查询对应 root。provider 仅运行有界、可取消的 Git worktree listing，并返回路径脱敏、字段限量的投影。将它与 `create/import/archive/cleanup` 等写能力分开，provider 缺失继续 503；本阶段不得开启任何写侧 lifecycle。
- **P1 — PostgreSQL / RLS / grants 没有部署验收**：静态 `ENABLE/FORCE ROW LEVEL SECURITY` 不足以证明 target role 正确。需要目标环境 disposable DB 上的 owner/app 登录角色、成员关系、跨 tenant 与跨 Project 探针、grant fingerprint、回放/并发和事务回滚证据；当前审查不连接 DB。
- **P1 — master slice 有已知审查债务需再核对**：`PHASE-WLD1-REPOSITORY-MASTER-REVIEW-20261024.md:41,44-48,66-69` 记录旧候选的单独关闭 / role privilege drift / disposable test target guard / `Cargo.lock` 不匹配问题。当前 WBS 已记录 master 实现进 dev，但这份历史审查基于旧候选基线；集成前须在最新 dev 对每项复查是否已修复，不能直接把旧 findings 当成现状，也不能当作已解决。

## §4 推荐 owner / 数据责任矩阵

| 事实或操作 | 唯一写 owner | 读取与授权要求 | 禁止边界 |
|---|---|---|---|
| Repository identity / provider metadata | SCM Repository master owner | tenant scope；该事实本身不授权用户 | 不把 Project/Run/Host path 写进 SCM master |
| Project↔Repository active association | Project Directory / Project authority owner（需在 WLD1 contract 明确） | 当前 Project membership / manager grant；独立关联 revision 与事件 | SCM owner、Run owner、Host manager 不可直接写此关系 |
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
| ProjectRepository | Project manager 添加/读取/移除关联；active association 指向现有 Repository master revision；同 Project 多 Repo 与无 Branch Repo 可完整列举 | 普通成员不能管理关联；撤权 / 移除后 API 不再授权；跨 tenant / Project 关联不可读写；重复请求精确回放、重用 idempotency key 更改内容冲突 | owner 单测 + PG owner/app role/RLS route integration；target DB 实证未取得前不得关闭 |
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

## §8 Current-dev continuation audit (2026-11-07)

**Comparison baseline:** local `dev@996ae3ae7d6e4c9c075715f3799f5186734d32f0`. The 2026-10-25 lane baseline and evidence above are historical, not current implementation proof. This is a bounded static audit; I did not edit source/DDL, run tests, run Cargo, or connect to PostgreSQL.

### §8.1 Boundaries observed at dev@996ae3ae (2026-11-07)

| Concern | Current-dev evidence | Assessment |
|---|---|---|
| SCM Repository identity | `db/migrations/2026-10-23-scm-repository-master.sql:22-23,31-39,53-74` defines tenant-scoped `scm.repository_master` plus versioned event history and RLS/owner grants; it explicitly places Project links elsewhere and adds no Project FK. The WBS at `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:368` records the integrated adapter/remediation slice and selected-package tests. | **Present as a bounded SCM master slice.** The WBS says PostgreSQL runtime, full workspace, Clippy, target DB/RLS/grants remain open. This is not Project authorization or a Project repository directory. |
| Project identity and membership authorization | `permission.project_role_binding` is a durable tenant-scoped role table with FORCE RLS in `db/migrations/2026-09-29-worktree-group-phase-2b.sql:10-48`. The existing `ProjectRepository` port and `InMemoryProjectService` are shown at `crates/domain-project/src/lib.rs:406-453`; the in-memory service constructs `InMemoryProjectRepository`. Current Run authorization loads Project binding in `crates/star-api-rest/src/group_api/engineering_runs.rs` (parent CodeGraph snippet identified `active_binding` and the Run authority path). | A role table and current Run-side reads exist, but they do not by themselves prove the selected Star-native Project owner's durable Project identity/member writer or a versioned cross-deployment owner API. A bounded current path scan found no Project PostgreSQL adapter alongside the SCM adapter; this is not an exhaustive repository-wide absence claim. |
| Project↔Repository association | `SRS-WLD1-REPOSITORY-BOUNDARY-001.md:49-55,69-82` and `BD-WLD1-REPOSITORY-BOUNDARY-001.md:18-40,44-56` specify the user-selected Project owner, 0..N active associations, optional default preference, audit/Outbox, and migrate-before-unlink. `scm.repository_master` has no Project association columns. | **Design is explicit; durable association table/API/owner command is not evidenced in the checked implementation.** `ProjectPolicy.default_repository_id` or same-tenant SCM membership cannot stand in for the 0..N authorization relation. |
| Hostless Engineering Run | Current API route is `crates/star-api-rest/src/group_api/engineering_runs.rs:274-295`; `create_project_run` inserts `repository_id=NULL, branch_id=NULL` and creates its Run grant in `:606-640`. The returned representation preserves both values as `null` at `:642-653`. | **Hostless create path exists at source level.** This does not create a separate RunRepository binding or prove production HTTP/target-DB acceptance in this audit. |
| Run↔Worktree binding vs Run↔Repository binding | Parent CodeGraph evidence identifies `RunWorktreeRow` in `engineering_runs.rs:234-253` and `WorktreeProjectState`/current Run binding in `run_worktree_bindings.rs:196-219,772-812`; the binding is a Worktree-to-Run fact and validates the Worktree's repository identity. The WLD1 detailed design defines a separate optional `RunRepositoryBinding` (`DD-WLD1-REPOSITORY-BOUNDARY-001.md:11-18,35-40`). | Do not treat the current Run↔Worktree binding table as the planned independent Run↔Repository CAS owner command. Its runtime and DB gates remain distinct. |
| Host root and legacy import registry | `crates/worktree-shared-dir/src/external_worktree_import.rs:521-565` implements `ExternalWorktreeImportRegistry` as `RwLock<HashMap<Uuid, PathBuf>>`; `register` accepts a `PathBuf`. The current WLD1 contract instead requires trusted Host Manager ownership and opaque root identity/generation (`SRS-WLD1-REPOSITORY-BOUNDARY-001.md:41-53,74-76`; `BD-WLD1-REPOSITORY-BOUNDARY-001.md:28-37,44-55`). | The legacy process-local map is neither a durable Host-owned root registry nor a safe registration boundary. The checked migrations/source do not establish the proposed HostRoot registry; keep root registration and path resolution fail closed. |
| Worktree discovery/provider wiring | `crates/star-api-rest/src/group_api/worktree_lifecycle.rs:327-358` exposes one `RunWorktreeLifecycleProvider` trait containing repository discovery, import-candidate, create, and import operations. `require_lifecycle_provider` returns unavailable at `:528-532`. `GroupApiState` initializes this provider as `None` at `crates/star-api-rest/src/group_api.rs:361-384`; the inspected startup path in `main.rs:95-101` constructs state and installs the BI query port but does not call `.with_run_worktree_lifecycle_provider(...)`. | **No lifecycle provider is installed by the inspected `star-api-rest` startup path.** The API remains fail-closed. The current combined trait also does not match the WLD1 target's independent read-only discovery capability; do not enable the trait or claim provider readiness from its interface alone. |
| Deployment and target DB | WLD1 SRS records Vercel→Neon and K3s→database inside Multipass VM(s) at `:55,81`; TD says Neon and Multipass profiles require independent migration/owner/RLS/grant acceptance and limits test inventory to two VM instances at `docs/test-design/TD-WLD1-REPOSITORY-BOUNDARY-001.md:7-11`. The WBS repository-master row at `:368` says target PostgreSQL runtime was not run. | These are accepted architecture/test constraints, not evidence that Neon or the Multipass-hosted DB is wired or that either target passed migration/RLS tests. |

### §8.2 Snapshot findings and current document cross-check (2026-11-08)

1. **Project owner and association implementation remain open.** The user decisions are closed and WBS v0.85 §13 records the Star-native Project owner, 0..N association, migrate-before-unlink, and deployment split. Remaining work concerns durable owner/API/schema, live authorization, association CAS/idempotency/audit/Outbox, and target-role evidence; do not reopen owner selection.
2. **SCM-master status is already distinguished in current WBS.** The 2026-10-23 directory report is a pre-master snapshot. WBS v0.85 §12-§13 records the integrated SCM identity-master slice and keeps Project association/Host root boundaries separate. Preserve dated WBS §15 history; do not repeat the old “no canonical Repository master” wording in current status or reopen that lane.
3. **WLD1 design-to-test reference drift remains.** Current BD v0.4 cites TD v0.2 as pending sync, while TD is v0.3 and aligns with SRS/BD/DD v0.4. A separate manifest-scoped documentation sync should update the BD reference; TD remains a test design, not executed acceptance.
4. **WBS R0b wording lags the detailed contract.** WBS v0.85 §13 still says association state and transaction boundary await design confirmation, while DD v0.4 specifies active/retiring/revoked states and the Project-owner transaction contract. Update current WBS wording to separate those specified semantics from open reference inventory, admission fence, target selection, proof completeness/revisions, cross-owner coordination, and deployed DB evidence. Preserve the dated v0.2 snapshot in WBS §15.
5. **Source-wiring observations in §8.1 and this section are historical only.** They were not revalidated at dev@6b3151f3; refresh source evidence before using them to decide implementation or provider readiness. Worktree writes remain fail closed until their independent gates are proven.
6. **The current-dev copy of this review still has superseded owner-pending language in §7.1.** WBS v0.85 §13 records the 2026-10-26 selection of the Star-native Project owner. Mark the old recommendation as superseded in a separate current-document sync; do not reopen owner selection.
7. **Correction to the prior lane-state statement:** both dated paths are tracked at this lane HEAD and at dev@6b3151f3. The continuation changes those same paths; the earlier phrase saying the lane held untracked copies was incorrect. Reconcile the reviewed text against current dev rather than replacing its documents blindly.

### §8.3 Minimal dependency DAG and acceptance

```text
WLD1-0: synchronize/freeze the selected owner, 0..N association, root, and deployment contracts
   ├── WLD1-A: Star-native Project identity/member owner + ProjectRepository association
   └── WLD1-B: trusted Host root registry (parallel only after RepositoryId and
              Project-manager authorization contract are frozen)
          └── WLD1-C: separate RunRepository binding command (depends on A+B)
                 └── WLD1-D: dedicated read-only Run Worktree provider
                        └── WLD1-E: serialized Neon and Multipass-VM DB/Host acceptance
                               └── WLD1-R1: migrate or explicitly retire every active source reference
                                      └── WLD1-R2: revision-pinned zero-reference proof and final unlink
```

- A and B may run in parallel only after WLD1-0 freezes stable `RepositoryId`, authenticated Project-manager capability contract, root-generation identity, and path-secrecy rules. C depends on A+B; D depends on A+B+C; E validates the implemented contracts before any reference migration. Shared router/workspace/Cargo/WBS integration is serialized by the coordinator.
- A acceptance: Project-owned 0..N association add/list/retire/revoke, expected revision and exact replay/conflict, same-tenant Repository identity, cross-tenant/unauthorized rejection, optional default constrained to active links, durable audit/Outbox and FORCE-RLS/app-role catalog behavior. The Project identity/member-auth API and its Vercel/Neon and K3s/Multipass-VM persistence must be covered separately.
- B acceptance: only a trusted local Host Manager can register a canonical repository root; root identity/generation is stable, path input cannot come from API/Agent/Plugin/Task CLI, stale/offline/ambiguous roots fail closed, and no absolute path or credential appears in API/event/log/BI.
- C acceptance: hostless Run remains valid; RunRepository binding has its own owner, CAS/idempotency, audit/Outbox and current Project+Run grant checks, independent from Cloud Branch, Worktree binding, and Infrastructure placement.
- D acceptance: discovery is scoped to current Project+Run grants and one current binding; bounded/cancellable Git reads return sanitized paginated results; no mutation occurs. Provider absence returns unavailable.
- E acceptance is run separately against disposable Neon and the DB inside a Multipass VM, with correct owner/app identities, migration replay, RLS/grants, revoke races, cross-tenant/Project isolation and failure recovery. Only after E is accepted may R1 enumerate complete version-pinned Run/Worktree/Host references and migrate them to an eligible same-Project active Repository or explicitly retire unneeded checkout references. Every failure preserves the source association and old bindings. R2 requires complete, fresh evidence of zero active references plus concurrency-safe expected-revision CAS and durable audit/Outbox receipts before unlink. No test was run by this review; current state remains OPEN.

### §8.4 Evidence limits

The parent supplied current-dev CodeGraph status (1,925 files, up-to-date) and task-directed snippets for `EngineeringRun`, `RunWorktreeRow`, `WorktreeProjectState`, and binding call paths. This lane has no `check_index_coverage` tool, so graph evidence does not support exhaustive absence claims. I performed bounded source/DDL/doc searches in the named modules and migrations. No code, migration, compile, test, Neon, or Multipass database operation was executed. WLD1 remains OPEN; Worktree create/import/archive/delete/cleanup remain unavailable unless separately accepted owner/provider gates are proven.
### §8.5 Revision history

- v0.4 (2026-11-07) — Revalidated the bounded findings against current dev@996ae3ae, distinguished the integrated SCM Repository master from the still-missing Project association, updated the accepted Project owner/deployment decisions, identified WBS and BD/TD reference drift, and recorded the minimum WLD1 dependency and acceptance sequence. No runtime claim was added.
- v0.5 (2026-11-08) — Kept the 2026-11-07 source observations explicitly historical, refreshed the WBS/BD/TD cross-check against dev@6b3151f3, corrected the lane-file tracking statement, and clarified which WLD1 decisions are specified versus still open. No source or runtime claim was added.
