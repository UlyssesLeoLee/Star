# TD-WLD1-REPOSITORY-BOUNDARY-001

> 状态：Draft v0.2；目标测试设计，测试尚未实现或执行。
> 需求：`SRS-WLD1-REPOSITORY-BOUNDARY-001.md` v0.2。
> 设计：`BD-WLD1-REPOSITORY-BOUNDARY-001.md` v0.2、`DD-WLD1-REPOSITORY-BOUNDARY-001.md` v0.2。

## §0 验收原则

测试必须分辨 ProjectRepository authorization、RunRepository binding、Host/root identity 和只读 Worktree projection。单元测试或 mock provider 不能证明目标数据库 RLS/grants、Host path 防护、跨 owner 事务边界或实际文件系统/Git 行为。没有真实目标角色和 disposable PostgreSQL 证据时，RLS/迁移门保持 OPEN；没有受控 Host integration 时，路径安全与 discovery 运行门保持 OPEN。

测试不得使用生产数据或用户工作树。Host adapter 使用测试临时目录与确定性 Git fixtures；数据库运行必须有独立 disposable target identity、非 superuser owner/app role，不自动连接默认 DB，不清理不属于当前测试的资源。Neon 与 Multipass VM 是两个独立目标 profile，必须分别执行目标迁移和 owner/RLS/grants 验收；Multipass 测试先核对 inventory，最多保留 2 个实例，不自动创建或 purge。

## §1 需求追踪矩阵

| Requirement | Tests | Evidence |
|---|---|---|
| FR-WLD1-01/02, AC-WLD1-01 | UT-PROJECT-01..06, PG-PROJECT-01..08 | Project owner CAS/idempotency/tenant grants + target PG rows/audit/outbox |
| FR-WLD1-11, AC-WLD1-09 | UT-HOST-08, HOST-INT-02 | local-only identity with null remote fields and explicit trusted root attestation |
| FR-WLD1-03, AC-WLD1-02 | API-RUN-01..05 | hostless create/list/authorization independent of repo/branch/host |
| FR-WLD1-04/05, AC-WLD1-03 | UT-RUN-01..08, PG-RUN-01..08 | binding revision and live revoke/currentness behavior |
| FR-WLD1-06/07, AC-WLD1-04 | UT-HOST-01..08, HOST-INT-01..09 | registration provenance, root generation, path isolation, filesystem identity |
| FR-WLD1-08, AC-WLD1-05/07 | UT-RO-01..12, HOST-INT-10..21 | bounded read-only provider on real disposable repositories |
| FR-WLD1-09, AC-WLD1-08 | UI-WLD1-01..08 | navigation, empty, detached, stale and unavailable state |
| FR-WLD1-10 | EVT-WLD1-01..08 | owner outbox/inbox replay, duplicate, reorder and transport replacement |
| FR-WLD1-12/13/14, AC-WLD1-10/11/12 | UT-PROJECT-07..12, PG-PROJECT-09..15, DEP-WLD1-01..04 | canonical Project authority, current member grants, deployment-specific DB acceptance, migration-before-unlink and fail-closed recovery |
| NFR path/privacy/resource | SEC-WLD1-01..12, PERF-WLD1-01..07 | no path leakage, bounded children/output/queue/cache, cancellation and RSS/CPU |

## §2 Unit 与 contract tests

### 2.1 Project identity, membership and Repository association

| ID | 场景 | 预期 |
|---|---|---|
| UT-PROJECT-01 | Project manager attach same-tenant Repository | 单一 active relation，association revision 增加，产出 audit/outbox receipt |
| UT-PROJECT-02 | Project manager attaches a second distinct Repo | 两项都能分页读取，不互相覆盖 |
| UT-PROJECT-03 | 普通 Project member 添加/撤销 | 拒绝，数据库无变化、无 Outbox |
| UT-PROJECT-04 | Repository tenant 与 Project tenant 不同 | 不可读写并返回不泄露存在性的 scoped error |
| UT-PROJECT-05 | 重放相同 idempotency key + digest | 原 receipt 精确回放，只存在一个业务事件 |
| UT-PROJECT-06 | 相同 key 不同 digest / stale expected revision | 拒绝，不静默覆盖新 revision |
| UT-PROJECT-07 | 未授权 actor 读取或改写 Project membership/association | 拒绝且不泄露资源是否存在 |
| UT-PROJECT-08 | Project 创建时 creator membership 持久化失败 | Project 与 creator grant 同事务回滚，不留下半创建 Project |
| UT-PROJECT-09 | Project member revoke 后重放旧 grant revision | 拒绝，当前授权状态不被缓存或旧 revision 复活 |
| UT-PROJECT-10 | 设置/清空 default Repository preference | 只接受 same-Project active association；不生成 Run binding |
| UT-PROJECT-11 | association 进入 retiring | 新 Run bind 被拒；已授权的既有 binding 保持可解析 |
| UT-PROJECT-12 | association 最终 revoked 或当前 Project member 被撤权 | 所有关联读取复验后拒绝；无缓存授权窗口 |

### 2.2 Run binding

- UT-RUN-01：无 Host/Repo/Branch 创建 Run 并授权成功；没有 Worktree provider call。
- UT-RUN-02：Run 未绑定时，Project 可见的 Repository 不会自动出现在 Run bound list。
- UT-RUN-03：绑定另一 Project 或另一 tenant 的 Repository 被拒绝。
- UT-RUN-04：一个 Run 多个明确的 Repository/Host root binding 可稳定分页，返回值不交叉混淆。
- UT-RUN-05：Run Owner 与 Project Manager capability 区分；未经授权 actor 不可 bind/unbind。
- UT-RUN-06：CAS stale、重复请求、idempotency digest conflict 与 revision 回放行为稳定。
- UT-RUN-07：Project relation revoke 和 Run grant revoke 后，新 discover/list detail 失败，即使 binding row 和 cache 仍存在。
- UT-RUN-08：Host/root generation 更新使旧 binding stale；新 binding 必须显式确认新代次。

### 2.3 Host registry 与只读 provider

| ID | 输入/故障 | 预期 |
|---|---|---|
| UT-HOST-01 | Agent/Plugin/API payload 带 absolute path | DTO/schema 拒绝该字段；Host manager 不受调用 |
| UT-HOST-02 | 相同 root 正常 re-register | idempotent 或明确返回相同 generation，不能复制 root |
| UT-HOST-03 | root move/replace/permission revoke | generation 递增并立即废止旧 handle/cache |
| UT-HOST-04 | symlink 指向已选 root 外目录 | 拒绝；不输出被指向路径 |
| UT-HOST-05 | root 目录被替换或 mount/file identity 漂移 | 拒绝并标 stale |
| UT-HOST-06 | 两个 Host 为同一 Repo 注册不同根 | identity 以 Host + Repo + generation 区分，不串用 root |
| UT-HOST-07 | Host authentication/identity 未知或撤销 | 不执行 Git child process |
| UT-HOST-08 | Local-only Repository with no provider/remote/default branch | SCM identity remains stable; only a Project-authorized operator attestation may map a Host root |
| UT-RO-01 | 合法 root 含普通/locked/detached worktree | 返回有界规范化 rows，ref state 正确 |
| UT-RO-02 | root 无 worktree/非预期 repository metadata | 明确 empty/error，不产生 placeholder ID |
| UT-RO-03 | malformed, duplicate, invalid UTF-8/oversized stdout | fail closed，不回显原始输出 |
| UT-RO-04 | stdout/stderr/row/page cap 超限 | 终止/拒绝；无部分成功列表 |
| UT-RO-05 | timeout/cancel during process | child 终止、资源释放、无写 side effect |
| UT-RO-06 | shell metacharacters in local ref label | 显示字符串被长度/字符约束；从不拼进 shell command |
| UT-RO-07 | provider capability introspection | 只暴露 list/get；无法调用 create/import/remove/cleanup |
| UT-RO-08 | provider absent | 503/typed unavailable；没有旧 mixed provider fallback |
| UT-RO-09 | concurrent list on same binding | 不超过 profile 的 in-flight cap；公平处理其他 Run |
| UT-RO-10 | cache hit then revoke/generation change | 旧缓存被拒绝，不以 TTL 延长权限 |
| UT-RO-11 | cursor replay with different Run/binding/tenant | cursor scope mismatch 拒绝 |
| UT-RO-12 | pagination large fixture | 不加载全量 records；稳定 cursor、bounded memory |

## §3 PostgreSQL / owner API integration

使用目标 PostgreSQL 版本与正式迁移链创建 disposable database。至少以 Project owner、Run owner、SCM owner 和 API app 独立 login role 验证，拒绝 superuser/BYPASSRLS 代替应用角色测试。

| ID | 验收 |
|---|---|
| PG-PROJECT-01 | 新 schema/migration 在空库应用两次都成功且无 drift |
| PG-PROJECT-02 | 仅 Project owner 可写关联事实；SCM/Run/API role 直接写入被拒 |
| PG-PROJECT-03 | FORCE RLS 隔离不同 tenant 和同租户不同 Project；设置可信 actor/tenant context 才能读写 |
| PG-PROJECT-04 | Project association active unique、retiring fence、revoked history、revision CAS 和并发 attach/revoke 约束正确 |
| PG-PROJECT-05 | transaction rollback 同时撤销 association、audit、outbox 与幂等记录 |
| PG-PROJECT-06 | 同一 key replay 返回一致 receipt；key/digest mismatch 被拒 |
| PG-PROJECT-07 | Outbox event 不含 root path、credential、URL secret 或 repository file data |
| PG-PROJECT-08 | grant fingerprint 与 migration owner/search_path/signature 符合预期 |
| PG-RUN-01 | Run owner 独立拥有 binding 表；Project/SCM owner 无法直写 |
| PG-RUN-02 | RLS tenant + Project + Run scope 阻断 cross-scope select/update/delete |
| PG-RUN-03 | 多 Repo binding key/unique/revision，Run 零 binding 状态合法 |
| PG-RUN-04 | Project revoke 与 Run bind 并发后，discovery 当前性复验拒绝 stale relation |
| PG-RUN-05 | Run grant revoke 与 discovery race 不能产生授权读 |
| PG-RUN-06 | outbox/audit/idempotency 原子提交或一并 rollback |
| PG-RUN-07 | owner roles 仅可调用批准命令，不跨 owner direct SQL |
| PG-RUN-08 | 事务锁序避免跨 owner query、解绑、撤权形成死锁；deadlock retry 有界 |

| PG-PROJECT-09 | Project identity and member grants are written only by Star-native Project owner; identity-owner/Run/SCM/API roles cannot mutate them |
| PG-PROJECT-10 | Project revoke or grant revision change invalidates subsequent association/binding operations; stale cached authorization is rejected |
| PG-PROJECT-11 | Retiring fence blocks new Run bindings while source remains valid for existing bindings |
| PG-PROJECT-12 | Migration to a different Project or inactive/unauthorized association is rejected without changing source bindings |
| PG-PROJECT-13 | Explicit binding retirement is accepted only with lifecycle-owner evidence that checkout is no longer needed; no implicit delete occurs |
| PG-PROJECT-14 | Missing owner receipt, stale inventory cursor, incomplete page or failed target root verification prevents zero-reference proof and final unlink |
| PG-PROJECT-15 | Only complete fresh zero-reference reconciliation plus expected-revision CAS can finalize unlink; every failure preserves source association and old bindings |

### §3.1 Deployment-profile database integration

| ID | Target | Acceptance |
|---|---|---|
| DEP-WLD1-01 | Vercel/Neon disposable branch or database | Apply canonical migrations twice; verify owner/app roles, RLS/grants, idempotency, rollback and current authorization against the Neon target. |
| DEP-WLD1-02 | Multipass VM PostgreSQL disposable database | Apply the same migrations and owner tests; verify the VM endpoint identity is pinned and no test exceeds the two-instance inventory cap. |
| DEP-WLD1-03 | Both targets independently | Verify there is no browser DB access, cross-profile credential reuse, shared mutable DB assumption or reuse of one target pass as the other target’s evidence. |
| DEP-WLD1-04 | Multipass inventory preparation | If inventory is unknown, full, or identity cannot be classified, fail closed; never auto-create or purge an instance. |

## §4 Host controlled integration

使用单独临时 Host profile 和一次性 Git repository fixtures，不接触用户仓库。每个场景使用固定 ID，记录结果、脱敏日志和资源回收证据。

| ID | 场景 | 验收 |
|---|---|---|
| HOST-INT-01 | 可信管理界面注册 root，重启 Manager | generation 和受保护引用从持久 registry 恢复；启动期间不扫描整盘 |
| HOST-INT-02 | Local-only Repository 没有 remote URL/provider/default branch | 普通 tenant member 的 root 声明被拒；仅显式 Project-authorized operator attestation 可建立 RepositoryId 到 Host root 的映射 |
| HOST-INT-03 | 通过 opaque root ID 读取 | 绝对路径不出现在 API、错误、日志、trace、BI 或 Outbox |
| HOST-INT-04 | 在 discovery 前替换 root 目录、symlink 或 mount target | 旧 generation 与旧 handle 均被拒绝，不读取替换后的越界内容 |
| HOST-INT-05 | root 含多个普通、detached、locked、prunable 或路径缺失的 worktree | 每页结果受限；本地 Branch/ref 状态正确，不伪造远端 Branch |
| HOST-INT-06 | 检查 Git 子进程 | 只启动获批 Git executable 与固定 argv；无 shell、额外子进程或写副作用 |
| HOST-INT-07 | 注入慢、挂起、超量或损坏的 Git 输出 | deadline、cancel、byte/row cap 和 worker cleanup 生效，不发生部分成功 |
| HOST-INT-08 | 同一 Repository 在两个 Host 注册 | Run binding 只返回所选 Host 的 root generation，不串用另一个 Host |
| HOST-INT-09 | 未授权、Host revoked 或 provider missing | 在 child process 启动前失败；返回明确且脱敏的 unavailable/authorization 状态 |
| HOST-INT-10 | 合法 root 含普通、locked、detached worktree | 返回有界规范化 rows，ref state 正确 |
| HOST-INT-11 | root 无 worktree 或出现非预期 repository metadata | 返回明确 empty/error，不产生 placeholder ID |
| HOST-INT-12 | malformed、duplicate、invalid UTF-8 或 oversized stdout | fail closed，不回显原始输出 |
| HOST-INT-13 | stdout/stderr/row/page cap 超限 | 终止或拒绝；没有部分成功列表 |
| HOST-INT-14 | process timeout/cancel | child terminated，资源释放，无写副作用 |
| HOST-INT-15 | local ref label 含 shell metacharacters | 显示值通过长度/字符约束；从不拼入 shell command |
| HOST-INT-16 | provider capability introspection | 只暴露 list/get；不能调用 create/import/remove/cleanup |
| HOST-INT-17 | provider absent | 返回 typed unavailable/503；不回退旧 mixed provider |
| HOST-INT-18 | 同一 binding 并发 list | 遵守 profile in-flight cap，并公平服务其他 Run |
| HOST-INT-19 | cache 命中后发生 revoke 或 generation change | 旧缓存立即不可见，不由 TTL 延长授权 |
| HOST-INT-20 | cursor 用于不同 Run/binding/tenant | scope mismatch 被拒绝 |
| HOST-INT-21 | 大型分页 fixture | 不加载全量记录；cursor 稳定，memory 有界 |

## §5 API / UI / event E2E

| ID | 操作 | 预期 |
|---|---|---|
| API-RUN-01 | 建立 hostless Run | 成功；没有自动 Repo 或 Branch |
| API-RUN-02 | Project manager associate Repo → Run manager bind Repo/Host | 当前 revisions 显式可见且可审计 |
| API-RUN-03 | 未授权 User 请求另一个 Project/Run binding ID | 统一拒绝，不泄漏 Repository/Host/root existence |
| API-RUN-04 | revoke Project link / Run grant / root generation then retry list | 新请求拒绝，活跃 discovery/cursor/cache 失效 |
| API-RUN-05 | Worktree write API missing provider / feature flags | 仍 unavailable，read-only binding 不扩大 capability |
| UI-WLD1-01 | 项目设置关联多个 Repo | 只在设置/选择流程呈现，不插入 Worktree 导航父层 |
| UI-WLD1-02 | hostless Run | Worktree 面板显示可解释未绑定态 |
| UI-WLD1-03 | Run 有两个 repo binding | Worktree rows 显示明确 Repo 标签并稳定分页 |
| UI-WLD1-04 | discovery row 处于 detached HEAD | Branch 子节点清楚显示 detached/ref state，不造远端 Branch |
| UI-WLD1-05 | Host offline/root stale/provider unavailable | 明确状态与可操作重试/绑定提示，不显示本地路径或错误堆栈 |
| UI-WLD1-06 | 权限撤销时当前页面打开 | 页面清除已加载 Worktree，关闭订阅，后续读取拒绝 |
| UI-WLD1-07 | 直接 deep link 到 Worktree row | 服务器重新授权后才返回 details |
| UI-WLD1-08 | 按 `Project→Run→Worktree→Branch` 逐级导航 | Repository 不改变主层级；Branch 保持 Worktree local ref 子项 |
| EVT-WLD1-01..08 | owner outbox 重复、乱序、延迟 ACK、消费者重启、provider 替换 | inbox 幂等；接收事务提交后 ACK；不丢关联/绑定审计事实；event payload 不含敏感路径 |

## §6 性能 / 资源基准

`PERF-WLD1` 必须固定版本化 Host Discovery Profile ID/revision/digest 和实际 workload、Host OS/文件系统、Git 版本、repo/worktree 条数、并发 Run/binding 数、页面大小与 root registry store。报告 p50/p95/p99 延迟、Host Manager RSS/CPU、child process 峰值数、最大 stdout/stderr、分页 memory、取消收敛时间、队列等待与缓存命中。任何数字在基线冻结前均为待测，不作为产品容量承诺。

| ID | 负载点 | 门槛 |
|---|---|---|
| PERF-WLD1-01 | 小/大 Worktree list，配置的最大 row/page | 不扫描无关 Repository/Project；RSS 与输出遵守 Profile byte/row cap |
| PERF-WLD1-02 | 并发多个 Run 对不同 Host binding | bounded queue、公平调度、每 binding 单 in-flight；无单 Run 饥饿 |
| PERF-WLD1-03 | 同一 binding 多客户端同时刷新 | 合并或拒绝重复 active child，不能形成 N 个等价 Git 进程 |
| PERF-WLD1-04 | timeout/cancel/revocation | 子进程按 Profile deadline 终止；资源在有界时间内归还 |
| PERF-WLD1-05 | 缓存命中与 generation change | 命中减少 Git 扫描；generation/revoke 后旧结果零可见 |
| PERF-WLD1-06 | large output and slow client | provider 队列与页面内存受限；慢消费者取消/分页，不积累完整列表 |
| PERF-WLD1-07 | root registry warm restart | 启动期间不全量扫描本地磁盘；按需恢复 registry 并校验 generation |

## §7 Exit gates / 当前状态

WLD1 设计阶段只能在 SRS/BD/DD/TD 交叉检查与兼容评审通过后关闭。实现阶段需依赖 owner relation、Host registry、Run binding、readonly provider 按 WBS 顺序执行。只有实际目标 PostgreSQL 角色/RLS、受控 Host Git integration、API撤权、资源基准和 `Project→Run→Worktree→Branch` E2E 均有结果时才可标记对应 gate PASS。

当前状态：上述测试是计划，尚未实现或运行。没有数据库、Host、Cargo、API、UI 或性能结果被宣称通过。

## §8 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.1 | 2026-10-25 | 建立 Project/Run/Host boundary、数据库/Host/E2E/性能测试矩阵与未验证门。 |
| v0.2 | 2026-10-26 | 补充 Project owner/member 授权真相、Neon 与 Multipass VM 独立验收、解绑迁移/fence/失败恢复测试；所有测试仍未运行。 |
