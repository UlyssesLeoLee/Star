# Phase 9F4C-D Run admission data classification

Date: 2026-10-10
Scope: D3 `2026-10-10-run-owner-schedule-admission.sql`; D1 `2026-10-09-task-run-admission-attempt-lifecycle.sql`; D2 `2026-10-10-multica-run-infrastructure-binding-master.sql`.

## Transaction facts

| Existing table | Classification | D1 change | Retention / mutability |
|---|---|---|---|
| `multica.task_execution_run` | Transaction | Add `created_at` and `automation_admission_attempt`; keep each attempt as an immutable Run identity; actual start is separate from provisional admission. | Append/update rules remain those of the existing Run owner; no new retention or deletion policy is introduced. Existing rows are not backfilled. |
| `multica.task_execution_run_event` | Transaction | Allow `schedule_admission_confirmed` and `schedule_admission_compensated` in the existing append-only event stream; only `schedule_admission_confirmed` proves Run-owner confirmation. | Append-only; WORM/audit policy unchanged. |

## Work and Master coverage

D1 creates no Work table and no Master table. It does not add an idempotency cache, mutable retry row, or profile revision. Schedule rules remain existing Master data; broker delivery state and lease Work retention are unchanged. The attempt ordinal is part of immutable Run identity and is therefore classified with the Run Transaction, not as mutable Work state.

## Verification boundary

The schema runner applies the migration twice and exercises uniqueness, legacy ordinal, provisional timestamp, non-Schedule start, and event-vocabulary constraints on disposable PostgreSQL 18.6. No target DB grant changes, production deployment, NATS, Multipass VM, or inventory mutation is part of this slice.

## D2 Run Infrastructure Binding Master

| New table | Classification | Owner / purpose | Mutability and retention |
|---|---|---|---|
| `multica.run_infrastructure_binding_revision` | Master | Multica Run owner stores current and prior bounded `RunInfrastructureBindingSnapshotV2` revisions for one canonical tenant/Project/Branch/Engineering Run. | SCD2: prior rows may be closed once through `valid_to`; snapshot, scope, binding identity, revision, digest and actor are immutable. No physical deletion. |
| `multica.run_infrastructure_binding_event` | Transaction | Trigger-appended `created` / `superseded` facts for binding revisions. | Append-only/WORM; application INSERT, update, delete and truncate are denied; no retention expiry. |

D2 adds no Work table. The revision Master is tenant-scoped and FORCE RLS protected; the NOLOGIN Run-owner role can read/insert revisions and update only `valid_to`, and can read events without direct insert permission. A fixed-path SECURITY DEFINER trigger appends event facts only after guarded Master lifecycle mutations. A composite FK ties each record to the canonical Engineering Run directory. Snapshot JSON is bounded to 16 KiB and stored with explicit scope, stable binding ID, monotonic CAS revision, schema version and digest. The Rust adapter validates the V2 contract before writes and recomputes the digest after strict read/deserialization. Infrastructure Profile/Host/Cluster Master data stays owned by Infrastructure: this Run-owned snapshot is a versioned reference and does not establish that referenced resources are currently valid.

The PostgreSQL 18.6 disposable runner applied D2 twice and passed the binding lifecycle, least-privilege grants, FORCE RLS, tenant isolation, WORM and direct-event-insert denial assertions. The complete Run-owner schema runner passed 29/29 steps and removed its disposable cluster; one no-op legacy audit function was installed only as a test prerequisite for older Worktree DDL and does not validate production Audit behavior. Rust 1.99.0 focused unit tests passed 12/12; focused rustfmt and package all-target warnings-as-errors Clippy passed. Broader strict Clippy remains blocked by two existing `star-context` lints. No production database grant/deployment, NATS consumer, Multipass VM or inventory mutation is claimed. D2 creates no run-as authorization, owner-message consumer, capacity reservation, provider/readiness evidence, BI/Benchmark event or execution capability.


## D3 classification (100% of newly introduced tables)

| Table | Class | Owner / audit | Mutability / retention |
|---|---|---|---|
| `multica.schedule_admission_inbox` | Transaction | Run owner processed command receipt; independent source Schedule service, fixed run-as, exact bounded wire/SHA-256, attempt and processed timestamp. | Append-only, tenant FORCE RLS; paired result required at commit; no TTL/physical deletion. |
| `multica.schedule_admission_result_outbox` | Transaction | Run owner accepted/rejected immutable result and recorded timestamp; acceptance references matching non-executing Run. | Append-only, tenant FORCE RLS; globally unique result and scoped request; no TTL/physical deletion. |

D3 introduces Work 0 / Transaction 2 / Master 0. Both Transaction rows themselves are immutable audit facts. Delivery checkpoints/leases are not represented by mutating these facts; a future owner-local bounded Work delivery state must be defined before a production poller is enabled. D3 grants only SELECT/INSERT on these facts and narrow identity/start/execution read columns on existing Run/RunEvent tables. It adds no Run or RunEvent INSERT grant and no Schedule/Infrastructure owner table access.
Phase 9F4C-D3 adds Run-owned processed admission Inbox and immutable result Outbox persistence, not a production consumer. `PostgresScheduleAdmissionStore.begin` strict-decodes at most 16 KiB, checks injected tenant/service scope, takes a transaction advisory request lock with 5-second lock and 10-second statement bounds, and returns an exact persisted result before any new Run writes. Changed exact bytes, digest or authenticated source service identity conflict. A new transaction is exposed only for Run-owned authorization, currentness, quota, Run/reservation/event work; dropping it rolls back. `commit_result` validates the typed result and atomically commits the Inbox/result pair with those caller-owned facts. A deferred FK forbids committed orphan receipts; unique tenant/occurrence/attempt prevents different requests claiming one attempt. Acceptance requires a matching schedule-origin, run-as, Task/Worktree/rule/occurrence/ordinal/correlation Run with NULL started_at and no run_started or execution-state event. Both relations are append-only Transaction facts with FORCE tenant RLS, bounded immutable bytes and no physical deletion; no Work or Master table is added. The trusted service source is injected by composition, not deserialized from the body, and its type constructor is not authentication evidence.

Evidence: PostgreSQL 18.6 cumulative runner 33/33; binding SQLx 1/1 and admission SQLx 1/1 (0.52s), Rust 1.99.0 library regression 12/12 and focused strict package Clippy. Audit: `target/phase9f4d-run-owner-consumer/40a8dc35-d9a2-4a76-a944-bd39474d51ef/audit.jsonl`. Tests prove rollback/drop recovery, exact replay, altered-byte/scope/source rejection, concurrent new/replay serialization, different-request same-attempt rejection, result-insert rollback, missing paired-result commit rejection, accepted-result scope guard, prior-start rejection and append-only guard. The accepted Run is seeded only by the disposable admin to exercise storage: it does not prove a production Run writer, authorization or resource reservation. Initial verification exposed the deliberately narrow old grant assertion; it was extended only for four Run identity/start columns and four RunEvent identity/execution columns. An extracted fixture helper reference error was fixed; transient Windows LNK1104 occurred before the final successful isolated rerun. All owned PostgreSQL instances were stopped/removed; zero NATS and zero Multipass VMs were used.

Open gates: real Schedule service authentication and credential lifecycle; target DB/login provisioning; current creator/run-as ACL, Task/Profile/Hook/binding authorization in this transaction; atomic Project/Run quota and Run/reservation writer; Schedule command producer and Run/Schedule/result/confirmation consumers; durable delivery/compensation; Infrastructure currentness and paired physical reservations; Multipass/K3s provider, sandbox, readiness, native Hook/verification, cancellation, BI/Benchmark and execution. No production Run INSERT or event INSERT grant was added. No cross-owner tables are read/written by this new storage module. Execution stays fail-closed.


## D4 scope/classification

D4 introduces no tables (Work 0 / Transaction 0 / Master 0); it adds a native execute-only authorization capability and transaction row locks over existing canonical Master/identity relations. Existing W/T/M classes, audit and retention remain unchanged. Runtime receives no direct directory/ACL write grants. Current Run binding remains the D2 SCD2 Master and is read under FOR SHARE through the caller transaction.

D4 adds `ScheduleAdmissionTransaction.revalidate_directory_and_binding` inside the already locked admission transaction. It sets the fixed occurrence creator as transaction-local actor, invokes the native fixed-path `multica.verify_schedule_run_as_current` capability, and reads the current binding through `resolve_current_binding_locked` with FOR SHARE. The native capability checks tenant/actor GUCs, reuses the canonical Project ACL advisory lock, requires current Project/Branch/Run grants and exact active Branch/Run/repository scope, and holds row locks on identities, revisions and grants. Project/Run writer roles follow the existing Schedule policy; Branch requires membership. The full pinned V2 binding and digest must equal the current locked Master. Missing authorization/paused scope yields RunAsUnauthorized, missing/drifted binding yields BindingUnavailable; storage/integrity errors remain errors and must not be treated as authorization.

The capability uses SECURITY DEFINER with search_path=pg_catalog and schema-qualified static SQL, PUBLIC execute revoked, and execute granted only to the Run owner. It grants no directory/ACL mutation to runtime. Explicit scope checks remain mandatory even when the migration owner bypasses RLS in a disposable test; target deployment must review function ownership, underlying read/row-lock permissions and policies. This capability is not broker identity authentication or complete admission authority. Current Task/Profile/HookSet, quota/reservation, Run writer, confirmation/delivery/compensation and execution remain open/fail-closed.

Evidence: cumulative PostgreSQL18.6/Rust1.99.0 runner 36/36, binding/admission SQLx targets 2/2; audit `target/phase9f4d-run-owner-consumer/e516c503-832e-40b8-8d5e-a02bebb44f75/audit.jsonl`. Admission assertions prove missing directory, current scope success, 55P03 timeout for concurrent ACL and binding closure while the transaction holds locks, revoked Project grant rejection, paused Run rejection and exact binding generation/revision drift rejection. Dedicated final schema run 34/34 checks fixed search_path, SECURITY DEFINER, no PUBLIC execute and no runtime directory/ACL write grants; audit `target/phase9f4d-run-owner-consumer/66c2abdb-8bbb-472d-a440-28ce7d2a55db/audit.jsonl`. Fixture seeding and mutation bypass lifecycle triggers only in disposable admin transactions; row-lock behavior remains real PostgreSQL. A transient MSVC LNK1104 occurred before the successful rerun, with no live matching test process observed; root cause is not proven. No NATS, VM or production database was changed.


## D5 — W/T/M 增量与锁边界

新增表 Work0/Transaction0/Master0（100% 新表覆盖）；只新增 native reader，沿用现有五张 canonical 表，不改变其分类与保留策略。

Schedule 新尝试必须在 Run-owner 同一事务内复验 Task/Worktree 的当前 canonical 归属：tenant、Project、repository、Branch、Engineering Run、Task、Worktree 精确一致；Worktree 未归档且有效，Project binding、Run binding、Task metadata、Task–Worktree relation 均为当前有效行。`revalidate_task_binding` 先复验固定创建者及锁定的 infrastructure binding，再调用固定路径 native SQL capability，持有五张目标表的 FOR SHARE 直到事务结束。缺失、归档或解绑返回 TargetUnavailable；权限/binding 拒绝保留原 typed code，数据库错误不得放行。

仅新增受限 EXECUTE 能力；SECURITY DEFINER 使用 search_path=pg_catalog、静态限定 schema，PUBLIC execute 撤销，runtime 无目标表写权限。目标环境 function owner/RLS/底层行锁权限仍须验收。Profile、原生 HookSet、任务生命周期可执行状态、预算/reservation、非执行态 Run writer、owner delivery/consumer、sandbox 与真正执行闭环仍未完成；primitive commit_result 不是完整准入入口，不启用执行。

验证：PostgreSQL18.6/Rust1.99.0 runner 40/40，SQLx binding/admission targets2/2；库12/12、包级 all-targets/no-deps Clippy -D warnings、格式/Python/图头/diff 通过。实际覆盖目标缺失、Task 身份不匹配、当前有效、Worktree 归档、关联关闭以及五类目标行并发关闭 SQLSTATE55P03。Audit：`target/phase9f4d-run-owner-consumer/857a6fbe-6bc7-4978-b070-bab871067cf7/audit.jsonl`。隔离 admin fixture 跳过生命周期触发器，不作为生产授权写入/审计证据。零 VM；生产数据库未变。CodeRabbit 因 /root/.local/bin/coderabbit: Permission denied 无独立通过结果。


## D6 — W/T/M 与 policy currentness

新表 Work0/Transaction0/Master0，覆盖100%；既有 agent_execution_profile 与 hook_policy_set 保持 SCD2 Master、其 lifecycle audit 保持 append-only Transaction。Native reader/publication lock 不改变表分类或 retention。

D6 为 Schedule 增加 `revalidate_profile_and_hooks` 与私有字段 `SchedulePolicySnapshot`：沿用 D4/D5 同事务授权、Binding 与任务归属门，原生固定路径 SQL reader 至多读取三份 72 KiB envelope；Rust 复用 domain-agent 的 Profile 校验和 domain-hook 的原生策略校验。精确核对 Profile ID/version/digest/schema/scope，Project baseline 与 Worktree overlay 的 ID、继承父版本和规则、有效 HookSet ID/version/digest，以及 Profile 与 RunInfrastructureBinding 的全部八项资源预算。缺失、版本/摘要/继承漂移或文档篡改返回 TargetUnavailable；预算与 binding 不一致返回 BindingUnavailable；存储异常不得放行。

Profile/Hook 当前事实保持 FOR SHARE 至事务结束；Project-scoped Hook 共享 advisory read fence 与 BEFORE publication 独占锁保护“当前没有 overlay”的读空隙，trigger 名称排序在现有继承 guard 之前。Native reader SECURITY DEFINER/search_path=pg_catalog，PUBLIC execute 撤销、仅 Run owner EXECUTE，不授予目录/任务/Profile/Hook 表写权限；目标部署须复核 function owner、RLS、底层读取/行锁权限与 migration 装配。此快照不是执行许可证：仍须当前 Provider/Skill/Grant catalog、任务生命周期、原生 Hook RunAdmission 求值与事实生产者、Project/Run quota/reservation、非执行态 Run writer、owner delivery/confirmation、Runtime sandbox 与独立验证。commit_result primitive 仍不是完整准入入口；执行保持关闭。

验证：Rust1.99.0 包级 all-targets check、库12/12、all-targets/no-deps Clippy -D warnings、fmt/Python/图头/diff；隔离 PostgreSQL18.6 runner44/44、SQLx binding/admission targets2/2。Audit `target/phase9f4d-run-owner-consumer/2a610d58-4b81-43d6-bc8a-4e8dc0c3e793/audit.jsonl`。实际验证 baseline 成功、缺失 overlay 的正常发布被 55P03 阻挡、事务结束后的 overlay 发布使旧 pin 失效、新 overlay pin 成功、Profile 版本漂移、预算 mismatch 和摘要篡改拒绝。Fixtures 的初始注册与 Profile 修订由隔离 admin/replica 完成，不证明生产授权 publisher；overlay 并发 INSERT 使用正常 publication trigger。未创建 VM、未改变 Multipass inventory/生产 DB。


## D7 — W/T/M 与 current catalog fence

新表 Work0/Transaction0/Master0，覆盖100%；沿用七张既有 catalog/capability/Grant Master及其Transaction audit分类，catalog_revision保持既有并发Master projection定义；不改变SCD2/immutable/审计/retention契约。

D7 增加 `ScheduleAdmissionTransaction.revalidate_execution_catalogs` 与私有 `ScheduleExecutionSnapshot`。同一 Run-owner 事务先通过 D4–D6 门，再由 native reader 仅读取 Profile 引用的当前 Provider/Skill 和当前 GrantSet：Provider≤5、Skill≤128、每项 capability≤64，Skill capability 预聚合预算384 KiB、返回 envelope≤512 KiB；Rust 复用 CurrentExecutionCatalogSnapshot/ExecutionProfileResolver，核对当前版本、摘要、availability、capability、完整 GrantSet 身份/版本/标签/到期时间，持有5秒目录 fence并在使用时要求至少1秒剩余。SQL 使用 C collation 对齐 Rust 的标识符排序。缺失、漂移、计数不一致、Grant变化或过期、fence到期均返回 TargetUnavailable，数据库失败不放行。

Native read 与 Run-owner 授权复验使用共享 Project advisory locks；目录 parent publication 和 canonical ACL mutation 仍使用相同 scope 的独占写锁，并持有当前目录/Grant row locks直到事务结束。reader SECURITY DEFINER/search_path=pg_catalog、PUBLIC execute 撤销、仅 Run owner EXECUTE，无目录直接写权限。发布 trigger 排序早于既有 SCD2 guard；capability 保持既有 same-transaction birth/immutable/count 契约。Snapshot 的 lifetime 校验不能替代原 owner 事务存活或执行前重新读取。目标 DB/function owner/RLS/应用身份装配仍未验收；任务 lifecycle、Project/Run预算预留、非执行态Run writer、owner delivery/confirmation、Runtime sandbox与独立验证仍开放，执行保持关闭。

调用时点须遵守已批准的基础设施准备模型：先创建等待基础设施预留的非执行态 Run，结果接受只表示准备请求已登记；实际启动/恢复 executor 前，必须复验 Host/cluster reservations与binding、当前权限/目录/策略，并以可信 Runtime health/readiness 和任务生命周期事实执行 Rust-native BeforeRunAdmission Hook。不得默认 health=true，不得以 UI health、VM/K3s就绪或仅文档校验代替 native Hook求值，也不新增弱化原生安全守门的 phase。当前本阶段未实现或声称通过该实际执行门。

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner47/47，SQLx binding/admission targets2/2；包级all-targets check、库12/12、all-targets/no-deps Clippy -D warnings、包级fmt/Python/图头/diff。Audit `target/phase9f4d-run-owner-consumer/ee331ea1-5cde-4005-8f86-80cc630c1dce/audit.jsonl`。实际覆盖缺失目录、精确4 Provider/1 Skill/Grant、同Project两个不同Task/Worktree预检并行、正常ACL和目录publication55P03、Provider unavailable、Skill digest drift、Grant expiry drift、capability count mismatch及真实5秒fence到期拒绝。仅验证预检并行，不证明Run/executor并行吞吐；未测最大负载RSS/延迟。Fixtures初始注册/变异为隔离admin/replica，不证明生产publisher授权。零VM与生产DB变更。


## D8 — Native Project capacity observation

reader SECURITY DEFINER固定search_path=pg_catalog，PUBLIC execute撤销，仅Run owner受限EXECUTE；不授予runtime直接quota/epoch/reservation写权限。仅现有Work allocation epoch在本事务中变化，drop回滚；没有写入Run、reservation或容量审计事实，也不表示Project容量已预留。新增W/T/M表0/0/0；现有epoch保留30天TTL。未来Run/reservation writer必须在该仍存活事务内再次确认fence并原子写入Run、reservation及成对审计/Run事件和Infrastructure demand Outbox；Host/cluster容量仍由Infrastructure owner自己的事务预留。原生执行Hook、Runtime健康/沙箱、取消恢复与独立验证仍为实际executor门，不制造健康事实。

| Category | Tables | Change |
|---|---|---|
| W | project_execution_resource_admission_lock | Same existing epoch/30-day TTL; transactional upsert; drop rolls back. |
| M | project_execution_resource_quota | Current active revision locked, no mutation. |
| W | project_execution_resource_reservation | Read active and unexpired pending; no lifecycle write. |
| T | Existing quota/reservation/Run events | No facts generated by observation; writer remains open. |


## D9 — Preparation write set

Project准备reservation的lease由数据库取min(occurrence deadline,当前时间+2分钟)，必须与目录5秒fence区分。等待确认期间预算已占用；超时/取消的补偿、release/expiry worker及后续激活仍开放，不把lease过期视作VM/Runtime已清理。新增W/T/M表0/0/0：现有reservation为W，Run/reservation events/RunEvent/Inbox/results为T，TaskContract与binding为M只读。新请求关联有deferred FK，native Run插入有deferred accepted-result/pending-reservation配对门，阻止孤立准备事实提交。SECURITY DEFINER固定search_path，PUBLIC execute撤销，仅可信Rust Run-owner角色EXECUTE，仍无直接Run/event/reservation/Task写权限。原生Hook实际求值、健康/sandbox/readiness、独立验证与executor门保持关闭；未把REST/UI或fixture作为生产闭环。

| Category | Existing table | D9 operation |
|---|---|---|
| W | project_execution_resource_admission_lock | Existing epoch write with30-day TTL. |
| W | project_execution_resource_reservation | pending insert with DB deadline≤2min lease. |
| T | task_execution_run | Immutable preparation insert; request FK + binding snapshot/digest. |
| T | project_execution_resource_reservation_event / task_execution_run_event | Atomic paired reserved event, same identity and details; no execution state. |
| T | schedule_admission_inbox / schedule_admission_result_outbox | Atomic processed receipt/result pair. |
| M | task_metadata / task_contract / directory / profiles / quota / binding revisions | Current facts locked/read; no mutation. |
| W | task_lifecycle_current | Current eligible state locked/read; Task owner retains writes. |

## D10 compensation write set

| Class | Table | Retention / ownership |
|---|---|---|
| W | project_execution_resource_reservation | Existing pending to released transition; no physical deletion here. |
| W | project_execution_resource_admission_lock | Existing allocation epoch, 30-day expiry; no cleanup here. |
| T | schedule_compensation_inbox | New exact immutable receipt, no TTL/deletion, tenant FORCE RLS. |
| T | reservation_event / task_execution_run_event | Paired append-only release audit and compensation fact. |
| M | Task/policy/binding | No mutation. |

New tables W/T/M=0/1/0. Infrastructure reservations are outside this write set.

## D11 confirmation write set

| Class | Table | Ownership / retention |
|---|---|---|
| W | project_execution_resource_admission_lock | Existing 30-day allocation epoch. |
| W | project_execution_resource_reservation | Existing pending row locked, not activated or extended. |
| T | schedule_confirmation_inbox | New immutable exact-byte receipt; FK to own admission and demand; no TTL/deletion; FORCE tenant RLS. |
| T | task_execution_run_event / infrastructure_capacity_demand_outbox | Atomic confirmation plus versioned demand; no execution event. |
| M | Task/Profile/Hook/binding/catalog/quota | Current facts read and locked, no mutation. |

New tables W/T/M=0/1/0. Host and cluster reservations are outside this owner transaction.

## D18 — Native transport log W/T/M classification

| Class | Table | Ownership / retention |
|---|---|---|
| W | `platform_transport.log_state` | Mutable sequence and retained-payload byte counters; scoped to deployment/tenant/log/generation. |
| W | `platform_transport.message_work` | Bounded raw wire, lease and attempt state; only pending/leased work remains; purge terminal wire after configured retention (24–8760 hours). |
| T | `platform_transport.delivery_receipt` | Permanent accepted/acked receipt and idempotency evidence; append-only, no payload and no retention purge. |
| T | `platform_transport.receipt_outbox` | Same-transaction metadata fact keyed by receipt; append-only, no raw payload, credential, or secret. |
| M | `platform_transport.log_master` | Immutable route/generation, payload budget, retention and attempt policy with guarded active→draining→disabled lifecycle. |

D18 adds exactly five tables: Work 2 / Transaction 2 / Master 1 (100% classified). All five require deployment+tenant FORCE RLS. Runtime receives no direct table DML; fixed-path owner routines are its only capabilities. `payload_sha256` is generated by the trusted Rust transport from exact wire bytes before calling the SQL routine. D18 adds no Rust transport service, production identity, profile provisioning or business-owner writes. The durable receipt/outbox belongs to Platform Messaging; Schedule/Run/Infrastructure continue to own their own Transaction facts and communicate through versioned events.
