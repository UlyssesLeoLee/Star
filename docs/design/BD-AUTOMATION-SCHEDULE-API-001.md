# BD-AUTOMATION-SCHEDULE-API-001 — Basic Design Addendum

> Status: 🟡 D11 Run-owner confirmation/demand verified in isolated PostgreSQL (56/56); full execution remains open. Production identity/deployment, confirmation/compensation/delivery, Task assignment, infrastructure/runtime/native Hook and complete execution remain open.
> Version: 0.44 | Date: 2026-10-10
> Parent: `docs/basic-design.md` §16 and `docs/data-design.md` §4.13.

## 1. Placement in the Worktree-first architecture
The immutable v2 binding explicitly stores ClusterPlacementPolicyV1 (shared_pool, run_dedicated, or pinned_cluster), and the deterministic digest covers that choice together with the resolved cluster ID/generation. The owner acceptance and readiness evidence use separate host_capacity_reservation_id and cluster_capacity_reservation_id; neither may stand in for the other.

The D2 disposable PostgreSQL runner passed 29/29 steps, including repeated binding migration, canonical Run FK, revision lifecycle, grants, FORCE RLS, tenant isolation and WORM checks. It installed a no-op legacy Audit function only for older Worktree migration prerequisites; that fixture is not production Audit evidence. Rust `run-owner-runtime --lib` tests passed 12/12 with 1.98.1, which does not meet the required 1.99.0 gate.


Schedule Rules are an Automation capability owned by an Engineering Run. The API is nested under Project → Engineering Run. A Schedule Rule references the Run's Branch, Repository, local Worktree, and Task Card; it does not create a separate workspace or bypass Run authorization. The UI may configure rules from the Run's Automation app. Selecting a Worktree remains the focus/CLI checkout decision and is not itself schedule authorization.

## 2. Components and transaction boundary

`star-api-rest::group_api::schedule_rules` is the authenticated REST boundary and is merged into `build_group_router`, which the REST binary uses. The workspace explicitly selects `jsonwebtoken`'s RustCrypto backend for deterministic RS256 signing/verification. Request-level tests run all four methods through the production router: missing credentials return 401, signed JWTs with insufficient scope return 403 before database access, and an oversized write body with a valid write scope returns 413; rejection responses carry private/no-store headers. These tests prove route wiring, token validation, scope rejection, and the body cap, but do not validate accepted Project/Run access, role grants, target ownership, or deployment. The boundary composes Project binding, Engineering Run authorization, Worktree/Task ownership checks, execution Profile resolution, and effective HookSet verification. `domain-automation::schedule` validates the pinned recurrence contract. PostgreSQL stores the immutable rule revision, append-only audit fact, append-only event Outbox record, and short-lived command replay record.

Create and revise each run in one PostgreSQL transaction. The transaction sets tenant/actor context, verifies current Project and Run grants, locks the idempotency key, resolves the target and policy snapshot, writes the Master revision and audit/Outbox facts, saves the replay response, then commits. Any error rolls the whole operation back. No handler launches a worker or creates a TaskExecutionRun.

Create derives immutable `run_as_actor_id` from the authenticated actor after Project/Run write authorization. The initial revision binds that identity to `changed_by`; each successor stores the same principal while `changed_by` separately records the editor. Historical rows backfill from the earliest Rule revision's `changed_by`; database guards reject both an initial mismatch and a changed successor principal. Every occurrence persists the identity in its own immutable column, and an insert guard compares it with the exact pinned Rule revision. The 9F4B runner passed on disposable PostgreSQL 18.6: all three migrations applied twice, seven Schedule tables retained FORCE RLS, and five adapter integration cases passed, including mismatched occurrence identity rejection. The future worker must recheck current Project membership, Run grant, and execution capability on every trigger, retry, and resume; revoked or unavailable ACL facts reject the occurrence without creating a Run.

## 3. W/T/M data classification

| Table | Class | Retention / mutation |
|---|---|---|
| `automation.schedule_rule_revision` | Master | Close-only SCD2 revisions with immutable `run_as_actor_id`; FORCE RLS; no delete/truncate. |
| `automation.schedule_rule_audit` | Transaction | Append-only actor facts; FORCE RLS. |
| `automation.schedule_rule_outbox` | Transaction | Append-only delivery facts written atomically with the revision; a composite foreign key binds its Run to the revision's Run; FORCE RLS. Consumer delivery state is a later bounded Work concern. |
| `automation.schedule_rule_command_idempotency` | Work | 24-hour replay TTL, exact expired-key deletion before reuse plus a 64-row stale cleanup budget, FORCE RLS. |

## 4. Fail-closed execution gate

The current capability is only Rule management. A Rule cannot execute until Phase 9F later stages connect recurrence materialization, fenced leasing, current ACL/target rechecks, capacity reservation, atomic TaskExecutionRun/RunEvent admission, and Outbox consumers. Existing disabled-rule checks remain in both domain materialization and persistence adapters. The implementation now filters future-dated Project bindings, Profile/Catalog entries, HookSets, and Rule revisions from current resolution. Production target database migration and service-role grants remain unverified.

## 5. 9F4C-A admission persistence foundation

`2026-10-04-schedule-run-admission.sql` adds nullable `automation.occurrence_dispatch.admitted_run_id` with a tenant-scoped FK to the canonical TaskExecutionRun. Dispatch history may set this field only in the `leased → admitted` transition; the DB guard confirms that the referenced Run is `run_origin='schedule'` and names the same occurrence. Once set, the Run identity cannot change. A dedicated append-only `automation.schedule_run_outbox` binds Rule revision, occurrence, Engineering Run, Task, and TaskExecutionRun through composite FKs; its insert trigger checks `execution_channel='agent'`, schedule origin/rule/version/occurrence, and `initiated_by=run_as_actor_id`. This Outbox is separate from Rule-change and Task-metadata Outboxes.

The migration does not create a dispatcher, authorize the stored run-as actor, reserve quota, write a RunEvent, or admit a TaskExecutionRun. Its composite relationships validate an already committed Run; they do not define a shared Schedule/Run transaction. Schedule owner claim/command, Run-owner provisional Run/reservation/events/result, Schedule-owner fenced link/outbox/confirmation, and Run-owner confirmation/demand are separate owner-local transactions. Historical dispatches without a provable matching Run remain nullable/unresolved; migration does not infer or backfill a Run identity. The chain requires the canonical TaskExecutionRun migration to be installed first. PostgreSQL 18.6 disposable validation applied the Schedule chain twice and tested positive/negative linkage, immutable identity, Outbox append-only behavior, tenant FORCE RLS, and non-superuser tenant isolation; it does not verify target DB grants or production owner consumers.

## 6. 9F4C-B Rule enable authorization

Rule management identity and unattended execution identity are separate. Each write first authorizes the authenticated editor against current Project/Run writer grants. Creating an enabled Rule then checks its derived creator run-as actor; revising an enabled Rule checks the immutable run-as actor loaded from the locked current revision. The check requires current Project and Run writer grants, the current Branch binding, and an active Engineering Run, and it shares the rule-write transaction so a failed check cannot persist the enabled revision. A manager can disable a Rule after creator revocation, provided the manager still has current rule-write access. The authorization helper verifies the tenant/actor transaction context and canonical Run/Project/grants, then obtains a tenant+Project transaction advisory lock before the API re-reads current ACL facts in a subsequent SQL statement. BEFORE triggers on Project, Branch, and Engineering Run grant INSERT/UPDATE/DELETE acquire the same lock; updates moving a binding between Projects acquire both Project locks in deterministic UUID order. Disposable PostgreSQL 18.6 tests with a SELECT-only directory runtime role prove active grants authorize, revoked Project/Branch/Run grants and a paused Run fail closed, and each of the three revocations waits for the overlapping authorization transaction and becomes visible to the next authorization after commit. Each trigger, retry, and resume still requires a current-state recheck in the Run owner's provisional-admission transaction.

The lock is transaction-scoped and Project-local; a hash collision may add waiting but cannot grant access. ACL mutation triggers resolve the canonical Project from the affected binding/Branch/Run, and fail closed if that scope cannot be resolved. Runtime API roles do not gain directory ACL UPDATE privileges. Disposable test-role helper execution grants must be provisioned after the role exists; production target role/schema grants remain an independent deployment gate.

### 6.1 9F4D Schedule admission inherits the Run infrastructure binding

Schedule Rules do not own or pin a Multipass instance. For each trigger, retry, or resume, Run/Execution re-resolves and authorizes the current RunInfrastructureBinding and provider/profile placement revision. The Run owner transaction rechecks the run-as actor, target/Profile/HookSet, binding and Project/Run quota, then records a provisional non-executing TaskExecutionRun, Project/Run resource reservation, Run events and an admission-result Outbox. The Schedule owner separately rechecks the current occurrence lease/fence/deadline before writing the occurrence link, Schedule Run Outbox and confirmation command. Only after the Run owner consumes that confirmation may a Run-owner transaction emit the idempotent Infrastructure demand. The Run reservation is not a Host or cluster reservation.

The Infrastructure owner consumes the demand through its own idempotent Inbox and transaction, resolves the explicit placement policy, and checks/reserves both HostEnvironment and exact selected-cluster capacity in Infrastructure-owned state. A Run binding pins the cluster ID/generation; a placement policy may select a shared pool, Run-dedicated cluster, or explicitly pinned cluster. It then delegates the chosen guest/cluster lifecycle work asynchronously to the Rust Host Infrastructure Manager. There are no cross-owner writes or distributed transactions. The Schedule worker and Task/Agent runtime never invoke Multipass, its daemon/socket, host shell, or guest exec.

### 6.2 Rust v2 multi-cluster binding, demand, and capacity contracts

The `domain-infrastructure` workspace crate defines V2 `RunInfrastructureBindingSnapshotV2` and `HostInfrastructureDemandV2` plus shared typed provider/resource/namespace values. The binding pins Tenant/Project/Branch/Engineering Run, HostEnvironment, exact K3s cluster ID/generation, binding and profile revisions/digests, namespace/service account, placement result and resource-budget snapshot. The demand pins Run ownership scope, TaskExecutionRun, Schedule Rule revision, occurrence, budget, and deadline; its consumer key is `(tenant_id, occurrence_id)`. A deterministic SHA-256 digest includes the pinned cluster identity. Separate capacity snapshot/request/candidate values calculate Host and selected-cluster headroom; assessment is advisory to the owner transaction and does not itself reserve capacity. Validation rejects incomplete identities, invalid labels/revisions/budgets/deadlines, unknown or stale capacity, and demand-to-binding scope mismatch.

This is a compile-time/data-shape foundation only. It has no ACL or placement resolver, database adapter, persistent Host/cluster reservations, inbox/outbox consumer, Multipass Manager, or execution capability. The fixed Rule creator remains the `run_as_actor_id`; every later admission must recheck the creator's current Project/Run grants and fail closed when authorization facts are missing. No production V1 consumer is present in this slice; any previously persisted V1 payload requires an explicit version migration before V2 rollout.

Implementation evidence and focused gate results: [`PHASE-9F4D-0-INFRASTRUCTURE-CONTRACT-REPORT.md`](../reports/PHASE-9F4D-0-INFRASTRUCTURE-CONTRACT-REPORT.md).

### 6.3 Provider-aware infrastructure readiness gate

要求 Host 与精确目标 cluster reservations、Schedule deadline、scope/binding digest、Host generation 和 cluster ID/generation 均有效，并有不早于 acceptance、不晚于当前时刻且不超过 30 秒的新鲜 observation。Multipass 默认路径需有受信任 Manager receipt、Multipass daemon 与组成目标 cluster 的全部 owned guest aggregate Ready 事实；Remote K3s 对 daemon/guest 两层显式标记 NotApplicable。两种 provider 都必须有 K3s API、profile 所需 Ready node 数、CRI、独立 Agent Runtime sandbox 和 Worktree workspace bridge 证据。此纯函数不授权 run-as、不持久化 owner reservation、不调用 Multipass/K3s、不启动 executor。Run owner 在 start/resume 事务仍须复验固定 creator 的实时 Project/Run ACL、lease/fence、quota 和 current binding；Infrastructure owner 在自己的 store 持久化 acceptance/evidence。Missing 或 stale evidence keeps TaskExecutionRun non-executing。当前测试阶段最多登记 2 个 Multipass 实例，Stopped/Deleted 记录也计入；bootstrap 以 host-local exclusive lock 串行化完整 inventory 检查和 launch，达到上限或无法解析时 fail closed，不自动 purge；所有 live test launch 必须经受控路径。契约测试不创建 VM。

### 6.4 Infrastructure-owner runtime role separation

The migration 2026-10-08-infrastructure-owner-runtime-grants.sql creates or repairs a star_infrastructure_owner NOLOGIN privilege group. It has no elevated role attributes and receives schema USAGE, Work-table SELECT/INSERT/UPDATE/DELETE, and Transaction-table SELECT/INSERT only. Schema CREATE, TRUNCATE, REFERENCES, and TRIGGER are denied. The application login is a separate principal provisioned by deployment secrets/bootstrap; membership and SET LOCAL ROLE activation must be verified in the production pool transaction. No password or login is created by this SQL migration.

Disposable PostgreSQL 18.6 verified duplicate apply and the complete role/privilege matrix. The migration-provisioning callsite, target DB role membership/activation, Infrastructure owner writer/consumer, and production admission remain open. The test cap is two registered Multipass instances including stopped/recoverable entries; this role-contract test does not create a VM.

### 6.5 9F4D-3C1 Infrastructure-owner Inbox receive boundary

The Infrastructure owner stores the incoming versioned demand in its own PostgreSQL Inbox. The adapter receives typed V2 demand plus a current binding snapshot supplied by the trusted owner boundary, checks digest/generation/deadline, and writes under transaction-scoped star_infrastructure_owner role plus tenant RLS. It returns only Enqueued, DuplicatePending, or AlreadyTerminal; an event/occurrence collision with different content is an idempotency conflict.

This is an Inbox receive adapter, not the consumer/admission transaction implemented in the later 3C2 slice. At the 3C1 boundary it did not resolve source identity, reserve Host or cluster capacity, append a terminal receipt, publish an owner Outbox event, invoke Multipass/K3s Manager, or start TaskExecutionRun. The separate owner-admission primitive is specified below; trusted resolver, production consumer, worker, readiness and execution gates remain independently required and fail-closed. The PostgreSQL gate does not consume a Multipass instance; the test inventory cap remains two registered instances total because of host disk capacity.

### 6.6 9F4D-3C2 Infrastructure-owner paired-capacity admission

The internal owner-store primitive accepts a previously persisted Inbox item, a fresh provider observation and a profile-derived resource request. It starts one Infrastructure-owner transaction, applies tenant RLS, acquires deterministic tenant+Host and tenant+exact-cluster advisory locks, and then locks/revalidates the Inbox identity, canonical demand payload, deadline and provider generation. It recomputes active reservations from the tenant's own ledger using `held` rows whose `lease_expires_at` is later than the transaction time; provider-supplied active-reservation totals are not authoritative. The V2 evaluator then checks exact scope, binding digest, freshness and both resource vectors.

On acceptance, one transaction writes distinct Host/cluster reservation IDs, the paired bundle, append-only lifecycle and demand receipt facts, terminal Inbox state, immutable owner Outbox and separate delivery Work row. Unknown/stale/insufficient observations schedule bounded retry without a reservation. Permanent scope/generation mismatches produce a terminal rejection. Disposable PostgreSQL tests verify one winner and one retry under contention, exact replay, generation rejection and capacity reuse after a lease expires.

This implementation is scoped to the tenant RLS ledger. It cannot account for another tenant's claims on a physical Host or cluster; cross-tenant shared placement must remain disabled until a global allocator and ledger exist. Provider/profile resolution, current ACL/binding serialization, Inbox/Outbox consumer, target DB role provisioning, Manager/provider, readiness and task executor are still absent. This slice starts no VM and leaves Schedule/CLI execution disabled.

### 6.7 9F4D-3C3 Persisted Provider Profile source for capacity admission

`infrastructure.provider_profile_revision` is the tenant-scoped SCD2 Master for the exact provider, placement policy and Host/cluster resource vectors pinned by a RunInfrastructureBinding. Its deterministic `profile_digest` covers the schema version, identity, provider, placement and resource dimensions. Infrastructure owner admission reads the exact `(tenant_id, profile_id, revision)` under its owner transaction, recomputes and compares the stored digest, then derives the capacity request from the stored row; no caller-supplied vector is accepted. A mismatch or unsupported record is terminally rejected before reservation. `provider_profile_revision_event` records creation and supersession as append-only Transaction facts. Both tables use FORCE RLS; the Infrastructure owner has read-only grants.

The disposable PostgreSQL runner applies the profile migration twice, checks both tables' tenant isolation, owner read-only privileges, one-time current-revision closure and immutable events, and executes five SQLx admission cases including digest-mismatch rejection without reservation. This adds a persisted source primitive, not a production profile editor/resolver or currentness fence. Cross-tenant shared capacity, target DB provisioning, consumer/delivery, provider/Manager and Schedule execution remain fail-closed. The test phase remains limited to two registered Multipass instances total, including stopped/deleted records; this gate starts zero VMs.

### 6.8 9F4D-3D2 reservation expiry and Schedule owner boundary

`PostgresInfrastructureOwnerAdmission::expire_due_reservations` scans only a caller-supplied tenant scope, rejects batches outside 1..64, orders candidates by PostgreSQL `clock_timestamp()` and processes each pair in its own transaction. The transaction takes the same sorted/deduplicated tenant and physical Host/cluster locks as admission, reloads both ledgers `FOR UPDATE`, and compares the complete reservation identity, generations, resource vectors, digests, observation time, lease and state. A changed or renewed candidate is an idempotent no-op; a mismatched pair aborts without partial mutation.

For an expired held pair, both Work rows transition to `expired`; the tenant T lifecycle fact, trigger-written global physical T fact, immutable `capacity_expired` Infrastructure Outbox event and pending delivery row commit atomically. The database clock is authoritative for both owner admission deadline checks and expiry. The primitive consumes no Schedule occurrence lease and does not renew/cancel an active Agent Runtime lease.

Infrastructure does not write Schedule occurrence, TaskExecutionRun or Run tables. The Run/Execution owner must consume the Outbox event through its own Inbox and then record its own result; Schedule retry/resume remains subject to the pinned run-as actor, current Project/Run authorization, occurrence fence and a fresh Run-owner provisional-admission transaction. The 9F4D-3D2 method is not a scheduled worker or production consumer; active execution remains fail-closed until those owner paths are deployed and verified. Rust and disposable PostgreSQL gates use zero Multipass VMs, and the test inventory cap remains two registered instances total including Stopped/Deleted.

### 6.9 9F4D-3D3 Infrastructure Outbox delivery state machine

Infrastructure owner exposes a bounded PostgreSQL store API over immutable owner Outbox facts and their separate Work delivery rows. Each call activates the restricted owner role and tenant RLS scope. Claims use database time and `FOR UPDATE SKIP LOCKED`, cap the batch at 64 rows and leases at 300 seconds, and return a typed event envelope with a monotonically increasing attempt fence. Stale workers cannot ack or write failure outcomes after lease expiry or a later claim.

Transient delivery failure is retried with exponential delay starting at one second and capped at one hour. Permanent failure and attempt exhaustion transition to `dead`; if the final lease expires after a worker crash, a later claim call terminalizes it instead of reissuing. Delivered/dead rows keep the schema's 24–25 hour cleanup retention. Claim returns the schema-bounded payload as raw UTF-8 JSON text (≤16 KiB/event), not an expanded JSON object graph; serialized payload bytes are at most 1 MiB per maximum batch. This does not claim a 1 MiB heap/RSS bound; a future consumer must budget parsing and parallelism separately.

This implementation does not publish to a transport, run a polling loop, provision production credentials, consume the event in Run/Execution, or enable Schedule execution. It only makes the delivery row transitions durable and concurrency-safe. The PostgreSQL gate uses no Multipass VM; registered inventory remains capped at two entries including stopped/deleted records.

### 6.10 9F4D-3D4 bounded JetStream publisher

The publisher accepts an established authenticated JetStream context and tenant-scoped Infrastructure delivery store. It leases 1..8 claims for one tenant with a 120..300 second lease and serially awaits PubAck. The allowlist is capacity_accepted, capacity_rejected, capacity_released, and capacity_expired schema v2; each JSON object is at most 16 KiB, the envelope at most 128 KiB, the subject is under star.infrastructure.v2, and Nats-Msg-Id is immutable event_id.

Only successful PubAck permits the PostgreSQL row to transition to delivered. Timeout and transport errors are ambiguous and use same-ID retry; delivery is at least once, so the future Run/Execution consumer must deduplicate with its durable Inbox. No broker setup, tenant poller, production DB identity, consumer, provider, or executor is included. The test runner uses an official NATS Server binary on loopback and creates no VM.

### 6.11 Phase 9F4D-3D5 Run-owner result consumer

The Run-owner demand Outbox append accepts the caller's transaction and validates the referenced TaskExecutionRun's schedule origin, Rule ID/revision, occurrence and correlation before insertion. The result consumer uses an existing provisioned durable explicit-ack pull consumer. Rust performs one bounded fetch (one message, 132 KiB, 2 seconds), validates the event contract and derives canonical scope from the exact immutable demand Outbox row. It records only an append-only result Inbox fact in the Run-owner transaction, then double-acks after commit. Exact duplicate replay is verified against stored scope/content; conflicts and DB failures are not acknowledged.

No Run/Schedule state update, RunEvent/BI emission, reservation mutation, readiness, or execution is part of this slice. Because invalid messages have no durable quarantine and the release producer is unavailable, this consumer is implemented as a fail-closed adapter and is not production deployable. Rust minimum is 1.99.0; isolated PG/NATS verification uses zero Multipass instances and retains the two-entry cap.

### 6.12 Phase 9F4D-3D6 Platform ingress quarantine

Malformed capacity-result messages cross the Run-owner boundary only to a separate Platform Messaging/Ingress owner. That owner uses a platform-scoped transaction and dedicated least-privilege role; it does not accept tenant or Run identity from the message for authorization or RLS. The stable quarantine key is the configured JetStream deployment identity and stream generation plus server stream name and stream sequence. `Nats-Msg-Id` and payload event IDs are bounded diagnostics, not authority.

The append-only quarantine receipt is Transaction data containing the trusted source key, allowlisted fault code and parser/schema version, exact wire byte length and SHA-256, bounded observed subject/broker metadata, and receipt time. If exact bytes are retained, they are a separate encrypted, size-bounded Work record with explicit TTL, quota and access policy. Each deployment MUST explicitly configure metadata-only or encrypted 24-hour retention; the service has no default policy.

For encrypted replay, the envelope copies at most 16 `Nats-Msg-Id` values and at most 8 KiB of aggregate value bytes. Either bound being exceeded leaves the source delivery pending; this caps allocations even when malformed input repeats empty headers.

The consumer may send `double_ack_with(AckKind::Term)` only after the receipt and selected payload policy commit. Exact duplicate source key plus equal digest/fault metadata is idempotent; a conflicting digest is an integrity alarm and remains pending. Database, quota, key, unknown-fault, and other transient/uncertain errors leave the delivery unacknowledged. `InvalidStoredDemand` is an owner-data integrity error and stays unacknowledged. Run/Schedule, BI, reservation, readiness and execution state remain untouched.

The implemented substrate adds `platform_messaging.poison_quarantine_receipt` (Transaction), `platform_messaging.poison_quarantine_payload_work` (Work), and `platform_messaging.poison_quarantine_payload_purge_audit` (Transaction), with FORCE RLS, separate ingress/purger NOLOGIN roles, immutable guards, an exact-wire digest and audited 24-hour purge. Encrypted payloads are limited to 200 KiB each and 4 MiB per deployment by both Rust preflight and a database trigger. Encrypted replay headers are bounded to 16 `Nats-Msg-Id` values/8 KiB aggregate. The Run-owner result consumer has an opt-in quarantine service and routes only three explicitly classified permanent faults; it commits quarantine before sending and awaiting confirmation of `double_ack_with(AckKind::Term)`, and leaves all other errors unacknowledged. Metadata-only and encrypted modes are explicit configurations; no deployment choice is presumed.

Focused Rust checks pass (16/16 unit tests, format and targeted Clippy). PostgreSQL 18.6 applied the migration twice and passed the RLS, WORM, TTL, purge-audit and over-quota SQL checks in a disposable loopback database. This does not prove production login membership/role activation, stream/deployment provisioning, target DB rollout, real KMS encryption (the current KMS implementation is LocalMock-only), operator access/replay, full NATS redelivery integration, or production consumer enablement. No Multipass instance or VM is required for these gates; test inventory remains capped at two registered instances including Stopped/Deleted.

### 6.13 Phase 9F4D-3D7 redelivery and confirmed-Term verification

The existing quarantine consumer hook is tested against real loopback JetStream and disposable PostgreSQL, not a mock broker. The first case injects a receipt INSERT failure and proves no receipt is committed while the source delivery remains ack-pending; after removing the fault, JetStream redelivers it and the Platform owner commits exactly one metadata-only receipt before the consumer awaits confirmed Term ACK. The second case records the receipt, drops the unacknowledged Message and fetch handles, and verifies exact duplicate recognition and Term ACK after real redelivery. This models consumer loss; it does not kill or restart an OS process. Each case checks zero later redelivery.

The two tests share a subject and therefore run with `--test-threads=1`; the disposable runner passed 22/22 steps and the ignored Rust target passed 2/2. No migration, VM, or Multipass inventory change is part of this gate. Production broker/DB identity, KMS, operator replay and consumer enablement remain separate gates; Schedule and Task Card execution remain fail-closed.

### 6.14 9F4C-D owner-local Schedule→Run admission protocol

The Schedule owner owns occurrence materialization, claim/reclaim leases, fencing generations, dispatch state, occurrence events and `automation.schedule_run_outbox`. It publishes a versioned admission command from its own Outbox. One logical attempt has a stable `request_id` and positive ordinal; `occurrence_id` groups attempts but is not the Inbox idempotency key. The command carries the pinned Rule revision, target tuple, fixed `run_as_actor_id`, immutable infrastructure binding and bounded deadline. Schedule lease owner/generation/expiry remain owner-local and are never trusted from the message payload. The receiving Run owner must authenticate the Schedule service identity out of band and reject malformed or conflicting scope.

The Run/Execution owner consumes the command through its own Inbox and transaction. It rechecks current Project/Branch/Engineering Run authorization, Worktree/Task/Profile/HookSet, current RunInfrastructureBinding/placement and Project/Run quota; it creates at most one provisional non-executing TaskExecutionRun, reservation, RunEvents, and an admission-result Outbox. Same `request_id` and exact payload returns the same Run ID; a different payload under that request is a conflict. A later logical attempt uses a new `request_id`, while broker redelivery reuses the exact persisted bytes. The Run owner does not update Schedule tables or claim that the Schedule lease remains current.

The Schedule owner consumes the result in its own Inbox/transaction and performs a conditional update against the current lease owner, generation, expiry and occurrence deadline. On success it links the admitted Run, writes its append-only Schedule Run Outbox, appends the occurrence event, and emits a confirmation command. The Run owner consumes that confirmation in its own transaction before writing the Infrastructure demand Outbox. A lost result/confirmation is replayed with the same identity; a stale or cancelled occurrence receives no confirmation and is compensated through idempotent owner commands. A process cannot become executable before confirmation and the existing capacity/readiness gates.

The command authentication and confirmation transport remain design inputs until the trusted production consumers, target DB grants and current-owner service identity are deployed. Existing same-database composite foreign keys only validate already committed facts; a physical database split uses typed owner receipts rather than cross-store foreign keys. No distributed transaction or cross-owner table write is permitted, and Schedule execution stays disabled until the complete flow is implemented and verified.

### 6.15 9F4C-D typed owner-message contract

`domain-automation` exposes schema-versioned `ScheduleRunAdmissionCommandV1`, `ScheduleRunAdmissionResultV1` and `ScheduleRunAdmissionResolutionV1`. The Command embeds `AutomationOccurrenceSnapshotV1` and `RunInfrastructureBindingSnapshotV2`; validation checks tenant, Project, Cloud Branch and Engineering Run consistency and uses the canonical binding digest. The Result echoes the logical request, attempt ordinal, correlation, occurrence and digest; accepted results carry one non-nil provisional Run ID and no rejection code, while rejected results carry a stable rejection code and no Run ID. Resolution accepts only an exactly matching accepted result and is either Confirm or Compensate with a required typed reason.

JSON decoding rejects unknown fields and bodies above 16 KiB. The command payload excludes Schedule lease owner, fencing generation and expiry: those remain Schedule-owned state checked by CAS when linking. `request_id` is stable for one logical attempt; Outbox retries must reuse the persisted bytes exactly. New logical attempts get a fresh ID and incremented ordinal after a terminal prior attempt. The payload's tenant is a claim, not authentication; the consumer derives its tenant/RLS scope from authenticated owner transport identity.

The code slice is a typed domain contract only. It does not persist or publish messages, authenticate broker identity, consume an Inbox, link a Run, process compensation, or enable execution. Those remain future owner-local implementation gates.

## 6.16 9F4C-D1 Schedule attempt and provisional Run persistence

The Run persistence schema MUST distinguish durable admission from execution. `created_at` is the row creation time; nullable `started_at` is the actual start time. For new non-Schedule Run rows, `started_at` remains required and writers MUST set it explicitly. A provisional Schedule row MUST have `started_at IS NULL` and a positive `automation_admission_attempt`; its first persisted record sets `created_at`.

Logical attempt uniqueness is `(tenant_id, automation_occurrence_id, COALESCE(automation_admission_attempt, 1))`, preserving legacy rows without updating them. A transport retry of the same logical attempt replays the same Run identity; a later attempt uses the next ordinal and a fresh request identity only after the previous attempt is terminal. Append-only RunEvents distinguish Run-owner Schedule confirmation, compensation and actual `run_started`; the Schedule-side `schedule_occurrence_linked` fact alone is not confirmation.

The authenticated Run API returns the attempt ordinal, derived admission state, creation timestamp and nullable start timestamp. List ordering and cursor comparison use `COALESCE(started_at, created_at)` plus `run_id`; the old `cursor_started_at` query name is accepted and the response also carries `cursor_ordered_at`. CLI inserts explicitly record their actual start to preserve the tightened timestamp invariant.

PostgreSQL 18.6 disposable verification applies this migration twice, runs the attempt/legacy/duplicate/event SQL cases, and asserts owner grants/RLS; the task-run API's focused cursor tests and `star-api-rest --all-targets` compile verify the read path. This does not implement a Schedule admission Inbox, Run confirmation/compensation consumer, production DB grants/identity, BI event pipeline, Infrastructure demand gate, or executor. Multipass inventory remains untouched and capped at two test records.

### 6.19 9F4C-D2 Run-owner binding source

The Run/Execution owner persists exactly one current infrastructure binding per canonical tenant/Project/Branch/Engineering Run. Each immutable `RunInfrastructureBindingSnapshotV2` revision pins the exact HostEnvironment and K3s cluster identity/generation, provider and runtime profile revisions/digests, namespace/service account, placement policy, and resource budget. Rust recomputes the versioned canonical digest when resolving a stored row and fails closed on malformed JSON, scope drift, binding ID/revision mismatch, or digest mismatch.

The Run owner uses one transaction to compare the expected current revision, close it once and insert the next revision. The Master is SCD2 and deletion is forbidden; create/supersede facts are append-only Transaction events. Tenant FORCE RLS applies to both. The restricted NOLOGIN Run-owner role receives SELECT/INSERT plus UPDATE(valid_to) only on the Master and SELECT-only on lifecycle events. A fixed-search-path SECURITY DEFINER trigger appends events only after a guarded Master insert or supersession, and direct application INSERT is denied. Infrastructure and Automation owners receive no grants. A composite FK binds the stored tuple to `multica.engineering_run`.

This resolver verifies stored Run-owner content integrity and the canonical Run scope; it does not prove run-as authorization or that Infrastructure-owned facts remain current. Infrastructure must independently load and verify the exact Provider Profile digest and Host/K3s identity generation in its own transaction before reserving capacity. No owner shares a write transaction. Rust SQLx save/resolve round-trip, Schedule/Run command consumers, target production DB role, reservations, readiness, Multipass/K3s Manager, BI/Benchmark and execution remain fail-closed.

## 7. Revision history

| Version | Date | Change |
|---|---|---|
| v0.1 | 2026-10-03 | Define the initial Run-scoped component and storage boundary. |
| v0.2 | 2026-10-04 | Record the reviewed persistence and cache-safety repair set. |
| v0.3 | 2026-10-04 | Align Rule-revision currentness with the current-fact contract and evidence. |
| v0.4 | 2026-10-04 | Record production REST-router wiring and unauthenticated route test evidence; keep authorization and target-environment gates open. |
| v0.5 | 2026-10-04 | Pin the JWT crypto backend and record signed-token scope rejection and authenticated body-cap request tests; keep accepted-access and database gates open. |
| v0.6 | 2026-10-04 | Add immutable run-as principal storage/backfill/guard and future per-trigger current-ACL recheck; keep worker, Launch Profile provider, target DB, and Run admission gates open. |
| v0.7 | 2026-10-04 | Record the first-revision creator equality and occurrence/pinned-rule identity guards, plus complete 9F4B PostgreSQL 18.6 validation while retaining worker/admission production gates. |
| v0.8 | 2026-10-05 | Define 9F4C-A's fenced occurrence-to-Run persistence link and separate append-only Schedule Run Outbox; record repeated disposable PostgreSQL validation and retain all authorization/atomic-writer/worker gates. |
| v0.9 | 2026-10-05 | Define create/enable-time run-as reauthorization separately from editor authorization; permit authorized managers to disable after creator revocation; retain the per-trigger worker gate and disclose missing ACL query integration coverage. |
| v0.10 | 2026-10-05 | Record the 1/1 pure role-policy helper test and clarify that it is not database ACL integration evidence; keep SQL fixture and per-trigger authorization gates open. |
| v0.11 | 2026-10-05 | Add disposable PostgreSQL canonical ACL fixture results under a read-only directory runtime role; retain concurrent revocation serialization, full route transaction, worker/admission, and target DB gates. |
| v0.12 | 2026-10-05 | Define Project-scoped advisory-lock ordering and canonical ACL mutation triggers; record Project/Branch/Run concurrent revoke serialization while retaining worker/admission and target database gates. |
| v0.13 | 2026-10-06 | Separate Run-owned Project/Run resource reservation from Infrastructure-owned Host reservation, and define the asynchronous Inbox/Outbox handoff and non-executing state while Host capacity/readiness is pending. |
| v0.14 | 2026-10-06 | Add the implemented Rust v1 binding/demand contract and record that it is a pure typed foundation without persistence, current-ACL resolution, provider execution, or production admission. |
| v0.15 | 2026-10-06 | Add the typed Host reservation/readiness gate and map evidence to the Worktree-first Infrastructure owner boundary; retain current ACL reauthorization and all persistence/worker/provider execution gates. |
| v0.16 | 2026-10-07 | Model multiple Multipass-managed K3s clusters, explicit profile placement, exact Run cluster generation, dual Host/cluster capacity, V2 typed contracts, and the temporary two-instance test limit; retain production fail-closed gates. |
| v0.17 | 2026-10-07 | Make the binding placement strategy part of the digest and name the distinct Host/cluster reservation IDs required by readiness; record 27/27 focused Rust tests while retaining owner/provider production gates. |
| v0.18 | 2026-10-07 | Define the test limit over all registered Multipass instances, including stopped/recoverable records, and record the locked fail-closed bootstrap creation guard without changing production topology. |
| v0.19 | 2026-10-07 | Add the Infrastructure-owner NOLOGIN role, least-privilege W/T grants and separate login/role-activation boundary; record the isolated PostgreSQL 18.6 grant-matrix evidence and keep production identity, migration caller, writer/consumer and execution gates open. |
| v0.20 | 2026-10-07 | Add the 9F4D-3C1 owner Inbox receive design and idempotency outcomes; distinguish receiving from the later capacity consumer/reservation transaction and keep execution disabled. |
| v0.21 | 2026-10-07 | Add the 9F4D-3C2 Infrastructure-owner admission primitive, deterministic dual-resource lock order, recomputed unexpired reservation ledger, atomic paired reservation/receipt/Outbox behavior, PostgreSQL concurrency/expiry evidence, and explicit tenant-only accounting boundary; keep resolver, consumer, provider and execution fail-closed. |
| v0.22 | 2026-10-07 | Add the 9F4D-3C3 tenant-scoped SCD2 Provider Profile Master and append-only revision events; require persisted digest verification and derive capacity vectors inside owner admission; record 17/17 disposable PostgreSQL steps and 5/5 SQLx cases while preserving target DB, production resolver, cross-tenant allocator, provider and execution gates. |
| v0.23 | 2026-10-07 | Add §6.8 for the database-clock, bounded paired-reservation expiry primitive and atomic tenant/global lifecycle plus `capacity_expired` Outbox facts; define Run/Execution consumer ownership and forbid expiry-driven implicit Schedule retry. Record 9F4D-3D2 focused Rust/PostgreSQL evidence while retaining production worker, active Runtime lease, provider and execution gates. Preserve the two-record Multipass inventory cap. |
| v0.24 | 2026-10-07 | Add §6.9 for tenant-scoped Infrastructure Outbox claims, lease fencing, bounded retry/dead-letter behavior and terminal retention; distinguish the verified state machine from broker publishing, production polling, Run-owner consumption and executor readiness. Preserve the two-instance disk cap and fail-closed execution. |
| v0.25 | 2026-10-07 | Add section 6.10 for serial bounded JetStream publication, injected authenticated context, allowlisted event contract, payload/envelope limits, stable Nats-Msg-Id, PubAck-before-delivered and durable Run Inbox dedupe. Record 9F4D-3D4 loopback NATS/PostgreSQL evidence and preserve production worker/consumer/execution gates. | The owner Outbox delivery state machine gained a bounded transport adapter. |
| v0.26 | 2026-10-07 | Add section 6.11 for caller-transaction TaskExecutionRun scope validation and the Run-owner result consumer's transaction/ack/fail-closed boundary; synchronize SRS v0.27, DD v0.26, requirements v5.98 and basic design v5.95. | Phase 9F4D-3D5 adds a verified result Inbox slice without enabling production execution. |
| v0.27 | 2026-10-08 | Add section 6.12 for Platform Messaging/Ingress quarantine ownership, trusted JetStream source identity, receipt and optional Work payload classification, and commit-before-Term semantics. Record that retention selection and all storage/consumer implementation remain open. | Phase 9F4D-3D6 establishes the design boundary without enabling poison-message ACKs. |
| v0.28 | 2026-10-08 | Record the implemented 9F4D-3D6 schema, RLS/role boundaries, explicit retention, database-enforced 200 KiB/4 MiB bounds, 16-value/8-KiB replay-header cap, confirmed Term acknowledgement, 24-hour purge audit and opt-in Run-owner hook; report focused Rust/PostgreSQL evidence and open production KMS, identity, stream, UI and rollout gates. Preserve the zero-VM/two-record Multipass test boundary. | Quarantine implementation, bounded-memory review and SQL schema/quota gates passed. |
| v0.29 | 2026-10-08 | Add §6.13 for verified receipt-failure backpressure, actual JetStream redelivery, exact duplicate replay after the pre-ACK handle-drop consumer-loss model (no OS process restart), confirmed Term ACK and serial shared-subject tests; record 22/22 disposable runner steps and 2/2 Rust integration tests while leaving production consumer/KMS/identities and execution closed. | Phase 9F4D-3D7 verifies the existing Schedule/Platform boundary. |
| v0.30 | 2026-10-09 | Add §6.14 and correct §6.1/§6.2 to separate the Schedule claim/command, Run provisional admission/result, Schedule fenced link/confirmation, and post-confirmation Run demand into owner-local transactions. Define stable idempotent recovery, authenticated service identity, stale-fence compensation and explicit fail-closed implementation gates; no production consumer is claimed. | Architecture review found that the previous Run transaction also wrote Schedule-owned occurrence and Outbox state. |
| v0.31 | 2026-10-09 | Add §6.15 with strict typed Command/Result/Resolution contracts, 16 KiB body bound, canonical infrastructure-binding digest checks, per-logical-attempt request identity, exact-byte broker replay, and out-of-band tenant authentication. Keep worker/consumer/execution disabled. | Freeze the owner-message contract before consumer implementation. |
| v0.32 | 2026-10-09 | Add §6.16 for the persisted attempt ordinal, provisional/start timestamp boundary, append-only lifecycle events and ordered Run API cursor; record disposable PostgreSQL and focused Rust evidence while keeping Schedule/Run consumers and execution fail-closed. | Phase 9F4C-D1 lifecycle implementation. |
| v0.33 | 2026-10-10 | Add §6.19 for Run-owner SCD2 binding persistence/resolution, digest validation, CAS revision, FORCE RLS and the explicit Infrastructure revalidation boundary; record PostgreSQL 18.6 29/29 and Rust 1.98.1 diagnostic tests 12/12; keep Rust 1.99.0, production identity, consumers, reservations, readiness and execution fail-closed. | The owner-message DTO had no authoritative persisted Run binding source. |
| v0.34 | 2026-10-10 | Harden §6.19 audit writes so the application role cannot directly insert lifecycle events; record PostgreSQL 18.6 29/29 and Rust 1.99.0 package tests/rustfmt/targeted Clippy while retaining target DB, production identity, consumers, reservations, readiness and execution gates. | Self-review found that direct event-table INSERT could forge lifecycle facts. |


### 6.20 D2 SQLx verification boundary

D2 SQLx follow-up: Rust 1.99.0 real PostgreSQL 18.6 binding test passed 1/1; the disposable runner passed 30/30 steps. It uses a LOGIN role with NOINHERIT, NOSUPERUSER and NOBYPASSRLS, explicitly activates star_multica_run_owner, and proves save/resolve digest roundtrip, wrong-tenant NotFound, stale revision rejection, single-winner concurrent CAS, insert-failure rollback of Master closure and audit facts, successful retry, and forged-digest rejection. No NATS or Multipass instance was started. Audit: target/phase9f4d-run-owner-consumer/cf849f3d-c7e8-4ea1-b1bd-a813121088c5/audit.jsonl. Production target DB/login provisioning, current authorization and Infrastructure currentness, owner consumers, reservations, sandbox/readiness and execution remain open/fail-closed.

Run `scripts/automation/phase9f4d_run_owner_consumer.py --schema-only --binding-tests` with the PostgreSQL bin path and pinned Rust 1.99.0 toolchain. The new flag is opt-in; existing schema-only and NATS consumer modes retain their behavior. Canonical parent rows and insert fault are disposable test fixtures only; no production trigger is weakened.

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.35 | 2026-10-10 | Ulysses — Mavis review | Close disposable SQLx binding verification gap; preserve every production admission/execution gate. |


### 6.21 D3 admission transaction boundary

Phase 9F4C-D3 adds Run-owned processed admission Inbox and immutable result Outbox persistence, not a production consumer. `PostgresScheduleAdmissionStore.begin` strict-decodes at most 16 KiB, checks injected tenant/service scope, takes a transaction advisory request lock with 5-second lock and 10-second statement bounds, and returns an exact persisted result before any new Run writes. Changed exact bytes, digest or authenticated source service identity conflict. A new transaction is exposed only for Run-owned authorization, currentness, quota, Run/reservation/event work; dropping it rolls back. `commit_result` validates the typed result and atomically commits the Inbox/result pair with those caller-owned facts. A deferred FK forbids committed orphan receipts; unique tenant/occurrence/attempt prevents different requests claiming one attempt. Acceptance requires a matching schedule-origin, run-as, Task/Worktree/rule/occurrence/ordinal/correlation Run with NULL started_at and no run_started or execution-state event. Both relations are append-only Transaction facts with FORCE tenant RLS, bounded immutable bytes and no physical deletion; no Work or Master table is added. The trusted service source is injected by composition, not deserialized from the body, and its type constructor is not authentication evidence.

Evidence: PostgreSQL 18.6 cumulative runner 33/33; binding SQLx 1/1 and admission SQLx 1/1 (0.52s), Rust 1.99.0 library regression 12/12 and focused strict package Clippy. Audit: `target/phase9f4d-run-owner-consumer/40a8dc35-d9a2-4a76-a944-bd39474d51ef/audit.jsonl`. Tests prove rollback/drop recovery, exact replay, altered-byte/scope/source rejection, concurrent new/replay serialization, different-request same-attempt rejection, result-insert rollback, missing paired-result commit rejection, accepted-result scope guard, prior-start rejection and append-only guard. The accepted Run is seeded only by the disposable admin to exercise storage: it does not prove a production Run writer, authorization or resource reservation. Initial verification exposed the deliberately narrow old grant assertion; it was extended only for four Run identity/start columns and four RunEvent identity/execution columns. An extracted fixture helper reference error was fixed; transient Windows LNK1104 occurred before the final successful isolated rerun. All owned PostgreSQL instances were stopped/removed; zero NATS and zero Multipass VMs were used.

Open gates: real Schedule service authentication and credential lifecycle; target DB/login provisioning; current creator/run-as ACL, Task/Profile/Hook/binding authorization in this transaction; atomic Project/Run quota and Run/reservation writer; Schedule command producer and Run/Schedule/result/confirmation consumers; durable delivery/compensation; Infrastructure currentness and paired physical reservations; Multipass/K3s provider, sandbox, readiness, native Hook/verification, cancellation, BI/Benchmark and execution. No production Run INSERT or event INSERT grant was added. No cross-owner tables are read/written by this new storage module. Execution stays fail-closed.

Gate command: `scripts/automation/phase9f4d_run_owner_consumer.py --schema-only --binding-tests --admission-tests`, with PostgreSQL bin and pinned Rust toolchain/target arguments. [Phase report](../reports/PHASE-9F4C-D3-ADMISSION-PERSISTENCE-REPORT.md).

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.36 | 2026-10-10 | Ulysses — Mavis review | Verify owner-local admission persistence; retain every production authorization/reservation/consumer/execution gate. |


### 6.22 D4 currentness capability

D4 adds `ScheduleAdmissionTransaction.revalidate_directory_and_binding` inside the already locked admission transaction. It sets the fixed occurrence creator as transaction-local actor, invokes the native fixed-path `multica.verify_schedule_run_as_current` capability, and reads the current binding through `resolve_current_binding_locked` with FOR SHARE. The native capability checks tenant/actor GUCs, reuses the canonical Project ACL advisory lock, requires current Project/Branch/Run grants and exact active Branch/Run/repository scope, and holds row locks on identities, revisions and grants. Project/Run writer roles follow the existing Schedule policy; Branch requires membership. The full pinned V2 binding and digest must equal the current locked Master. Missing authorization/paused scope yields RunAsUnauthorized, missing/drifted binding yields BindingUnavailable; storage/integrity errors remain errors and must not be treated as authorization.

The capability uses SECURITY DEFINER with search_path=pg_catalog and schema-qualified static SQL, PUBLIC execute revoked, and execute granted only to the Run owner. It grants no directory/ACL mutation to runtime. Explicit scope checks remain mandatory even when the migration owner bypasses RLS in a disposable test; target deployment must review function ownership, underlying read/row-lock permissions and policies. This capability is not broker identity authentication or complete admission authority. Current Task/Profile/HookSet, quota/reservation, Run writer, confirmation/delivery/compensation and execution remain open/fail-closed.

Evidence: cumulative PostgreSQL18.6/Rust1.99.0 runner 36/36, binding/admission SQLx targets 2/2; audit `target/phase9f4d-run-owner-consumer/e516c503-832e-40b8-8d5e-a02bebb44f75/audit.jsonl`. Admission assertions prove missing directory, current scope success, 55P03 timeout for concurrent ACL and binding closure while the transaction holds locks, revoked Project grant rejection, paused Run rejection and exact binding generation/revision drift rejection. Dedicated final schema run 34/34 checks fixed search_path, SECURITY DEFINER, no PUBLIC execute and no runtime directory/ACL write grants; audit `target/phase9f4d-run-owner-consumer/66c2abdb-8bbb-472d-a440-28ce7d2a55db/audit.jsonl`. Fixture seeding and mutation bypass lifecycle triggers only in disposable admin transactions; row-lock behavior remains real PostgreSQL. A transient MSVC LNK1104 occurred before the successful rerun, with no live matching test process observed; root cause is not proven. No NATS, VM or production database was changed.

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.37 | 2026-10-10 | Ulysses — Mavis review | Add verified transactional directory/binding prerequisite; preserve remaining admission/execution gates. |


## 6.23 D5 — Task/Worktree canonical capability

Schedule 新尝试必须在 Run-owner 同一事务内复验 Task/Worktree 的当前 canonical 归属：tenant、Project、repository、Branch、Engineering Run、Task、Worktree 精确一致；Worktree 未归档且有效，Project binding、Run binding、Task metadata、Task–Worktree relation 均为当前有效行。`revalidate_task_binding` 先复验固定创建者及锁定的 infrastructure binding，再调用固定路径 native SQL capability，持有五张目标表的 FOR SHARE 直到事务结束。缺失、归档或解绑返回 TargetUnavailable；权限/binding 拒绝保留原 typed code，数据库错误不得放行。

仅新增受限 EXECUTE 能力；SECURITY DEFINER 使用 search_path=pg_catalog、静态限定 schema，PUBLIC execute 撤销，runtime 无目标表写权限。目标环境 function owner/RLS/底层行锁权限仍须验收。Profile、原生 HookSet、任务生命周期可执行状态、预算/reservation、非执行态 Run writer、owner delivery/consumer、sandbox 与真正执行闭环仍未完成；primitive commit_result 不是完整准入入口，不启用执行。

验证：PostgreSQL18.6/Rust1.99.0 runner 40/40，SQLx binding/admission targets2/2；库12/12、包级 all-targets/no-deps Clippy -D warnings、格式/Python/图头/diff 通过。实际覆盖目标缺失、Task 身份不匹配、当前有效、Worktree 归档、关联关闭以及五类目标行并发关闭 SQLSTATE55P03。Audit：`target/phase9f4d-run-owner-consumer/857a6fbe-6bc7-4978-b070-bab871067cf7/audit.jsonl`。隔离 admin fixture 跳过生命周期触发器，不作为生产授权写入/审计证据。零 VM；生产数据库未变。CodeRabbit 因 /root/.local/bin/coderabbit: Permission denied 无独立通过结果。

关联脚本：`scripts/automation/phase9f4d_run_owner_consumer.py`；报告：[D5](../reports/PHASE-9F4C-D5-TASK-CURRENTNESS-REPORT.md)。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| Version: 0.38 | 2026-10-10 | Ulysses — Mavis 接手审核 | 同步已验证的事务内任务归属门与剩余缺口。 | D5 实现与实测。 |


## 6.24 D6 — bounded policy reader 与 publication fence

D6 为 Schedule 增加 `revalidate_profile_and_hooks` 与私有字段 `SchedulePolicySnapshot`：沿用 D4/D5 同事务授权、Binding 与任务归属门，原生固定路径 SQL reader 至多读取三份 72 KiB envelope；Rust 复用 domain-agent 的 Profile 校验和 domain-hook 的原生策略校验。精确核对 Profile ID/version/digest/schema/scope，Project baseline 与 Worktree overlay 的 ID、继承父版本和规则、有效 HookSet ID/version/digest，以及 Profile 与 RunInfrastructureBinding 的全部八项资源预算。缺失、版本/摘要/继承漂移或文档篡改返回 TargetUnavailable；预算与 binding 不一致返回 BindingUnavailable；存储异常不得放行。

Profile/Hook 当前事实保持 FOR SHARE 至事务结束；Project-scoped Hook 共享 advisory read fence 与 BEFORE publication 独占锁保护“当前没有 overlay”的读空隙，trigger 名称排序在现有继承 guard 之前。Native reader SECURITY DEFINER/search_path=pg_catalog，PUBLIC execute 撤销、仅 Run owner EXECUTE，不授予目录/任务/Profile/Hook 表写权限；目标部署须复核 function owner、RLS、底层读取/行锁权限与 migration 装配。此快照不是执行许可证：仍须当前 Provider/Skill/Grant catalog、任务生命周期、原生 Hook RunAdmission 求值与事实生产者、Project/Run quota/reservation、非执行态 Run writer、owner delivery/confirmation、Runtime sandbox 与独立验证。commit_result primitive 仍不是完整准入入口；执行保持关闭。

验证：Rust1.99.0 包级 all-targets check、库12/12、all-targets/no-deps Clippy -D warnings、fmt/Python/图头/diff；隔离 PostgreSQL18.6 runner44/44、SQLx binding/admission targets2/2。Audit `target/phase9f4d-run-owner-consumer/2a610d58-4b81-43d6-bc8a-4e8dc0c3e793/audit.jsonl`。实际验证 baseline 成功、缺失 overlay 的正常发布被 55P03 阻挡、事务结束后的 overlay 发布使旧 pin 失效、新 overlay pin 成功、Profile 版本漂移、预算 mismatch 和摘要篡改拒绝。Fixtures 的初始注册与 Profile 修订由隔离 admin/replica 完成，不证明生产授权 publisher；overlay 并发 INSERT 使用正常 publication trigger。未创建 VM、未改变 Multipass inventory/生产 DB。

复用既有 Rust 领域类型，仅增加 workspace path 依赖 domain-agent/domain-hook，无第三方版本变化；不创建 LLM session、Provider 或无界缓存，端到端 RSS/吞吐尚未实测。脚本 `scripts/automation/phase9f4d_run_owner_consumer.py`；[D6 report](../reports/PHASE-9F4C-D6-POLICY-CURRENTNESS-REPORT.md)。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| Version: 0.39 | 2026-10-10 | Ulysses — Mavis 接手审核 | 同步已验证 Profile/Hook/budget 门与仍开放的执行门禁。 | D6 实现与隔离验证。 |


## 6.25 D7 — 并行预检与目录发布隔离

D7 增加 `ScheduleAdmissionTransaction.revalidate_execution_catalogs` 与私有 `ScheduleExecutionSnapshot`。同一 Run-owner 事务先通过 D4–D6 门，再由 native reader 仅读取 Profile 引用的当前 Provider/Skill 和当前 GrantSet：Provider≤5、Skill≤128、每项 capability≤64，Skill capability 预聚合预算384 KiB、返回 envelope≤512 KiB；Rust 复用 CurrentExecutionCatalogSnapshot/ExecutionProfileResolver，核对当前版本、摘要、availability、capability、完整 GrantSet 身份/版本/标签/到期时间，持有5秒目录 fence并在使用时要求至少1秒剩余。SQL 使用 C collation 对齐 Rust 的标识符排序。缺失、漂移、计数不一致、Grant变化或过期、fence到期均返回 TargetUnavailable，数据库失败不放行。

Native read 与 Run-owner 授权复验使用共享 Project advisory locks；目录 parent publication 和 canonical ACL mutation 仍使用相同 scope 的独占写锁，并持有当前目录/Grant row locks直到事务结束。reader SECURITY DEFINER/search_path=pg_catalog、PUBLIC execute 撤销、仅 Run owner EXECUTE，无目录直接写权限。发布 trigger 排序早于既有 SCD2 guard；capability 保持既有 same-transaction birth/immutable/count 契约。Snapshot 的 lifetime 校验不能替代原 owner 事务存活或执行前重新读取。目标 DB/function owner/RLS/应用身份装配仍未验收；任务 lifecycle、Project/Run预算预留、非执行态Run writer、owner delivery/confirmation、Runtime sandbox与独立验证仍开放，执行保持关闭。

调用时点须遵守已批准的基础设施准备模型：先创建等待基础设施预留的非执行态 Run，结果接受只表示准备请求已登记；实际启动/恢复 executor 前，必须复验 Host/cluster reservations与binding、当前权限/目录/策略，并以可信 Runtime health/readiness 和任务生命周期事实执行 Rust-native BeforeRunAdmission Hook。不得默认 health=true，不得以 UI health、VM/K3s就绪或仅文档校验代替 native Hook求值，也不新增弱化原生安全守门的 phase。当前本阶段未实现或声称通过该实际执行门。

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner47/47，SQLx binding/admission targets2/2；包级all-targets check、库12/12、all-targets/no-deps Clippy -D warnings、包级fmt/Python/图头/diff。Audit `target/phase9f4d-run-owner-consumer/ee331ea1-5cde-4005-8f86-80cc630c1dce/audit.jsonl`。实际覆盖缺失目录、精确4 Provider/1 Skill/Grant、同Project两个不同Task/Worktree预检并行、正常ACL和目录publication55P03、Provider unavailable、Skill digest drift、Grant expiry drift、capability count mismatch及真实5秒fence到期拒绝。仅验证预检并行，不证明Run/executor并行吞吐；未测最大负载RSS/延迟。Fixtures初始注册/变异为隔离admin/replica，不证明生产publisher授权。零VM与生产DB变更。

脚本 `scripts/automation/phase9f4d_run_owner_consumer.py`；[D7 report](../reports/PHASE-9F4C-D7-CATALOG-CURRENTNESS-REPORT.md)。没有新增依赖或第三方版本。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| Version: 0.40 | 2026-10-10 | Ulysses — Mavis 接手审核 | 对账有界当前目录与同Project预检并行；明确准备/实际执行Hook时点。 | D7实现与隔离并发验证。 |


## 6.26 D8 — Allocation epoch and current capacity

Schedule owner 必须在同一 owner 事务内完成固定 run-as 与 canonical Task/Worktree、Profile/Hook、Provider/Skill/Grant 及绑定复验，再比较当前 Project 配额与 Run 的全部八项 ResourceBudget 上限。不得以成功预检代替实际 reservation、非执行态 Run 创建或 executor 启动许可。缺失/禁用/未来生效配额、预算超限、计数/算术溢出与过期目录 fence 必须 fail closed。

`revalidate_project_capacity` 使用 native `read_schedule_project_capacity_current`，复用手动 Run 分配路径的 Project allocation epoch（UPSERT 实际写入），串行同 Project 的容量分配。读取当前有效 quota，持有 quota publication 同键共享 advisory lock及quota行锁；REPEATABLE READ 下过时 epoch 必须产生40001而不能用旧 reservation 视图放行。统计同 Project 所有 active reservation，以及 lease 未过期的 pending reservation；过期 active 仍计数，released与过期 pending不计数。CPU/运行时长按单Run ceiling核对，RSS/子进程/并行工具/provider calls/输出/event buffer 按共享剩余总量核对，另核对Run数量。SQL仅返回一个≤4KiB envelope，Rust checked-add核算且在等待容量锁后重验5秒目录fence；无整表载入Rust缓存。数据库SUM仍需扫描Project相关ledger，未声称已验证最大规模查询/RSS性能。

reader SECURITY DEFINER固定search_path=pg_catalog，PUBLIC execute撤销，仅Run owner受限EXECUTE；不授予runtime直接quota/epoch/reservation写权限。仅现有Work allocation epoch在本事务中变化，drop回滚；没有写入Run、reservation或容量审计事实，也不表示Project容量已预留。新增W/T/M表0/0/0；现有epoch保留30天TTL。未来Run/reservation writer必须在该仍存活事务内再次确认fence并原子写入Run、reservation及成对审计/Run事件和Infrastructure demand Outbox；Host/cluster容量仍由Infrastructure owner自己的事务预留。原生执行Hook、Runtime健康/沙箱、取消恢复与独立验证仍为实际executor门，不制造健康事实。

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner49/49，binding/admission SQLx targets2/2、库15/15、包级all-targets check/Clippy与fmt、Python编译、图头和diff检查。Audit `target/phase9f4d-run-owner-consumer/d1e13219-1fbf-4f85-b7c1-e3604fa548d7/audit.jsonl`。实际覆盖缺失/禁用quota、精确预算匹配、同Project容量预检与正常手动epoch写入锁互斥（55P03）、quota关闭等待、drop回滚epoch、REPEATABLE READ过时allocation snapshot拒绝40001，以及未过期pending/过期pending/过期active/released计数。初始quota和reservation状态使用隔离admin/replica fixtures，不证明生产quota publisher、Run/reservation writer或真正executor并发。零VM/productionDB变更。

脚本 `scripts/automation/phase9f4d_run_owner_consumer.py`；[D8 report](../reports/PHASE-9F4C-D8-PROJECT-CAPACITY-REPORT.md)。本阶段无新增第三方依赖。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| 0.41 | 2026-10-10 | Ulysses — Mavis接手审核 | 接入Schedule当前Project容量核算，保留原子Run/reservation写入与执行缺口。 | D8实现与隔离容量验证。 |


## 6.27 D9 — Atomic Run-owner preparation

Schedule owner的fresh admission须经D4–D8当前授权、绑定、策略、目录和容量复验后，在Run owner本地事务原子创建非执行态TaskExecutionRun、pending Project reservation、成对reservation审计/RunEvent及处理后Inbox/admission-result Outbox。必须从canonical Task metadata、lifecycle和当前非空TaskContract读取有界完整快照，不接受调用者伪造任务内容。`started_at`与execution_state保持空，不产生run_started、VM请求或Task lifecycle写入；accepted只表示准备已登记。

`ScheduleAdmissionTransaction.prepare_and_commit`消耗原事务，并复用`commit_result`的exact replay/pair持久化。当前准入拒绝可提交不可变rejected result；存储错误、写入后fence/deadline失效则整笔回滚，不把已暂存Run与rejected结果一起提交。受限native `prepare_schedule_execution_run`复验scope/当前Profile和binding、再次串行核算Project容量并拒绝同Task已有active/未过期pending reservation。Task eligibility与既有CLI一致：in_progress、固定run-as持有同Worktree且非pending_review；当前TaskContract必须存在且验收项非空。Task/验收快照各≤32KiB，描述≤16KiB、labels≤64、dependencies/验收项≤128；超限拒绝，不截断。完整typed Engineering Run snapshot保留全部25个identity/version/role/ref字段；Run另持久化不可变Infrastructure binding snapshot/digest和request关联。

准备阶段的Outbox是Run→Schedule admission-result，不提前发布Infrastructure demand。Schedule owner必须消费结果、复验本owner lease/generation/deadline/cancellation，提交链接及confirmation；Run owner在下一独立事务消费可信confirmation、复验当前授权/绑定/Project reservation后才发布自己的Infrastructure demand Outbox。Host/cluster容量由Infrastructure owner自己的事务预留。这维持DD §6.2既有两owner确认协议，并明确D8“原子Run/reservation与Outbox”中的Outbox时点，不能用准备accepted绕过confirmation。Task owner的自动分配/claim/in_progress生产者仍需接入，Schedule不可跨owner篡改lifecycle。

Project准备reservation的lease由数据库取min(occurrence deadline,当前时间+2分钟)，必须与目录5秒fence区分。等待确认期间预算已占用；超时/取消的补偿、release/expiry worker及后续激活仍开放，不把lease过期视作VM/Runtime已清理。新增W/T/M表0/0/0：现有reservation为W，Run/reservation events/RunEvent/Inbox/results为T，TaskContract与binding为M只读。新请求关联有deferred FK，native Run插入有deferred accepted-result/pending-reservation配对门，阻止孤立准备事实提交。SECURITY DEFINER固定search_path，PUBLIC execute撤销，仅可信Rust Run-owner角色EXECUTE，仍无直接Run/event/reservation/Task写权限。原生Hook实际求值、健康/sandbox/readiness、独立验证与executor门保持关闭；未把REST/UI或fixture作为生产闭环。

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner 52/52，binding/admission SQLx targets 2/2，库测试15/15，包级all-targets check、Clippy、fmt、Python编译、Cypher图头及diff检查通过。Audit `target/phase9f4d-run-owner-consumer/00ff7b51-16c4-4a62-a776-121f6ef33a47/audit.jsonl`。实际覆盖受限owner角色写入、生命周期/TaskContract拒绝、deadline拒绝、延迟配对门拒绝孤立提交、写入后目录fence失效整笔回滚、结果持久化失败回滚及同命令重试、exact replay和容量占用；未产生execution事件或Infrastructure demand。隔离PostgreSQL已停止，零Multipass inventory和生产DB变更。未完成workspace/release/doc/bench、生产交付或性能验收。

脚本 `scripts/automation/phase9f4d_run_owner_consumer.py`；[D9 report](../reports/PHASE-9F4C-D9-SCHEDULE-PREPARATION-REPORT.md)。本阶段无新增第三方依赖或VM操作。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| 0.42 | 2026-10-10 | Ulysses — Mavis接手审核 | 对账真实Run-owner准备写入；保持Schedule确认后才发布Infrastructure demand。 | D9实现及隔离SQLx事务验证。 |


## 6.28 — Run-owner pre-infrastructure compensation

Run owner提供`PostgresScheduleAdmissionStore.compensate_admission`，消费有界严格`ScheduleRunAdmissionResolutionV1`补偿命令。必须绑定原始Schedule service identity、tenant及不可变accepted result的request/attempt/result/correlation/occurrence/Run/binding digest；错误来源、跨租户、伪造digest、Confirm误投及冲突重放均拒绝。可信transport认证仍须由部署组合，调用者传入UUID不是认证证据。

Native `compensate_schedule_preparation`在自身事务中串行原request及Project allocation epoch，仅对未确认、未执行、未发布Infrastructure demand且reservation仍pending的D9准备Run释放Project预算。它原子写入released状态、配对reservation/Run release audit、schedule_admission_compensated及exact-byte不可变receipt。最后receipt写入失败须回滚释放与全部事件。清理以原始owner事实为依据，不要求已撤权run-as重新授权；此权限不能创建或启动工作。历史accepted replay不改变released reservation。

新`guard_prepared_schedule_demand`对带D9 preparation lineage的Run复用同Project epoch和Run行锁，要求确认事件、未补偿和有效pending reservation，避免低层Outbox primitive绕过确认或与补偿竞争；旧无preparation lineage的Run保持既有契约。REPEATABLE READ过时epoch必须由数据库拒绝，未以Rust无界缓存观察竞争状态。

W/T/M新增0/1/0：`multica.schedule_compensation_inbox`为T（不可变exact-wire、无TTL/物理删除、FORCE tenant RLS）；既有reservation和allocation epoch为W，release/Run events及receipt为T；M只读。仅受限SECURITY DEFINER固定search_path capability执行写入，Run role无直接receipt/reservation/Task mutation。新表索引为tenant/request主键与唯一resolution身份，wire≤16KiB；函数有5秒lock/10秒statement timeout。

验证：隔离PostgreSQL18.6/Rust1.99.0 runner 54/54，binding/admission SQLx targets 2/2，库15/15及包级all-targets check、strict Clippy、fmt、Python编译、Cypher图头与diff检查通过。Audit `target/phase9f4d-run-owner-consumer/67cccbc7-e28b-4984-bd14-920afbc329f5/audit.jsonl`。真实owner角色验证释放/配对审计/receipt、exact replay、冲突重放、来源/tenant/digest误配、Confirm误投、active/confirmed拒绝、最终receipt失败整笔回滚、跨tenant RLS与补偿前后demand拒绝。Active/confirmed输入为隔离admin/replica fixtures，不证明生产确认/激活producer；未验证新的跨进程并发/过时snapshot竞争或最大规模性能。隔离PostgreSQL已停止，零VM/生产DB操作。

Schedule result/link/confirmation producer、confirmation消费与Infrastructure demand真正发布、全量超时release worker、跨Infrastructure owner补偿、生产service身份/DB grants、native Hook求值、Host/cluster reservations、Manager/sandbox/readiness、取消恢复及独立验证仍开放；本阶段不可宣称Agent/CLI闭环可用。零新依赖、零VM/生产DB操作。

脚本 `scripts/automation/phase9f4d_run_owner_consumer.py`；阶段报告 `docs/reports/PHASE-9F4C-D10-SCHEDULE-COMPENSATION-REPORT.md`。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| 0.43 | 2026-10-10 | Ulysses — Mavis接手审核 | 接入准备Run补偿和demand确认隔离门。 | D10实现与验证。 |


## 6.29 — Run-owner confirmation transaction

`PostgresScheduleAdmissionStore.confirm_admission`严格解码有界Schedule确认命令，核验原始Schedule service identity、tenant、accepted result及request/attempt/result/correlation/occurrence/Run/binding digest。它在同Run-owner事务复验当前Project/Branch/Run授权、Worktree/Task绑定、当前Infrastructure binding、Profile/Hook、Provider/Skill/Grant目录及5秒目录fence。确认不是executor许可证。部署必须独立认证service；注入UUID不是transport认证证据。

受限native `confirm_schedule_preparation`串行request和既有Project allocation epoch；复核当前quota及全Project已占用总量，准备reservation已在总量中，不能重复加算其预算。它锁定未确认、未补偿、未执行Run与有效pending reservation，检查Task lifecycle仍in_progress、run-as/Worktree一致、非pending_review，当前metadata/TaskContract版本与准备快照相同，以及当前binding/Profile版本与有效Hook文档不变。原生Hook的实际执行、Runtime健康/sandbox与独立验证仍为后续门。

确认RunEvent、typed V2 Infrastructure demand和不可变confirmation receipt在一个Run-owner事务提交。demand截止时间由DB钳制为min(occurrence deadline,Project reservation lease)，payload/receipt wire≤16KiB，Profile envelope≤72KiB；late fence/deadline或任何写入失败回滚全部事实。receipt引用其demand，tenant FORCE RLS且append-only，无直接receipt/Run/event/reservation写权限。请求精确重放返回同一历史demand/event identity，不重建demand或激活预算；不同resolution bytes拒绝。已确认/demand发布后不能再走D10本地补偿，后续必须通过跨owner释放协议处理。

新增W/T/M表0/1/0：`multica.schedule_confirmation_inbox`为T，无TTL/物理删除；既有Project allocation epoch/reservation为W，RunEvent/demand为T；当前Task/Profile/Hook/binding/catalog为M只读。单事务复用有界快照和数据库锁，无跨owner写事务，无VM创建或宿主shell权限扩展；Host/cluster容量由Infrastructure owner另行核验/预留。

验证：隔离PostgreSQL18.6/Rust1.99.0 runner 56/56，binding/admission SQLx targets 2/2，库15/15及包级all-targets check、strict Clippy、fmt、Python编译、Cypher图头和diff检查通过。Audit `target/phase9f4d-run-owner-consumer/b0a6e67a-47c9-4f05-86ce-aad4ae1292a8/audit.jsonl`。真实Run-owner路径验证当前授权撤销/预留过期/quota禁用拒绝、最后receipt失败整笔回滚、写入后目录fence失效回滚、成功确认及同一demand精确重放、冲突resolution拒绝、跨tenant RLS、确认后D10补偿拒绝，以及demand deadline钳制至reservation lease；只有一条demand且零执行事件。权限/quota/时钟失效来源仍为隔离admin/replica fixtures；未证明真实Schedule producer、认证transport、跨进程竞争或最大规模性能。隔离PostgreSQL已停止，零Multipass inventory/生产DB变更。

本阶段关闭Run-owner本地确认+demand写入，不关闭Schedule owner的result Inbox、lease/generation/deadline/cancellation复验与link/confirmation producer、生产认证/角色配置、可靠Outbox交付、过期/取消/跨owner补偿、Host Manager/sandbox/readiness/native Hook求值/独立验证、Agent/任务卡CLI完整闭环及BI/Benchmark。仍无生产executor、无完整workspace/release/doc/bench或规模性能验收。

脚本 `scripts/automation/phase9f4d_run_owner_consumer.py`；报告 `docs/reports/PHASE-9F4C-D11-SCHEDULE-CONFIRMATION-REPORT.md`。

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| 0.44 | 2026-10-10 | Ulysses — Mavis接手审核 | Run-owner确认及demand事务写入对账。 | D11实现及隔离数据库验证。 |
