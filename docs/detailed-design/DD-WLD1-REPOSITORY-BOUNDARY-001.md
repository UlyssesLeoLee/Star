# DD-WLD1-REPOSITORY-BOUNDARY-001

> 状态：Draft v0.4；hostless Run、V3 placement 和 Schedule/Manual origin 详细设计目标，未实现、未迁移、未通过运行验收。
> 上游：`SRS-WLD1-REPOSITORY-BOUNDARY-001.md` v0.4、`BD-WLD1-REPOSITORY-BOUNDARY-001.md` v0.4。
> 本设计不启用 Worktree 写操作。

## §0 实体与版本

| Logical aggregate | Owner | Identity / version | State |
|---|---|---|---|
| `Project` + `ProjectMemberAuthorization` | Star-native Project owner (`domain-project` boundary) | `tenant_id, project_id, principal_id, grant_revision` | active / revoked; Master history |
| `ProjectRepositoryAssociation` | Star-native Project owner | `tenant_id, project_id, repository_id, association_revision` | active / retiring / revoked; existing binding stays resolvable during retirement |
| `ProjectDefaultRepositoryPreference` | Star-native Project owner | `tenant_id, project_id, repository_id?, preference_revision` | optional; points only to active same-Project association |
| `EngineeringRun` | Engineering Run owner | `tenant_id, project_id, run_id, run_revision` | identity/create/authorize/list do not require host Branch or placement |
| `RunRepositoryBinding` | Engineering Run | `tenant_id, run_id, binding_id, repository_id, association_revision, binding_revision` | active / revoked |
| `RunInfrastructureBinding` | Engineering Run | `tenant_id, project_id, run_id, binding_revision, profile_id, profile_revision, profile_digest, host_environment_id, host_generation, cluster_id, cluster_generation, namespace, runtime_profile, budget` | optional; exists only after explicit placement selection |
| `HostRepositoryRoot` | Host Infrastructure Manager | `host_id, root_id, repository_id, root_generation` | active / stale / disabled |
| `WorktreeDiscoveryPage` | Read-only provider projection | binding/root generation + opaque snapshot cursor | fresh / expired / unavailable |

Association/binding revisions are monotonically increasing owner versions. An API client may submit expected version and idempotency key, but may not select actor/tenant identity, modify owner version, choose a host path, or assert current authorization.

## §1 API contracts

Logical routes are mounted under existing Project/Engineering Run API groups. Exact route names must be reconciled with current route conventions during implementation; they must preserve the command semantics below.

| Operation | Input | Owner transaction / read | Output |
|---|---|---|---|
| List Project repositories | Project ID, cursor, page size | current Project authorization; return active associations and retirement state without exposing cross-tenant existence | paginated opaque Repository identities, labels and association revisions |
| Read Project authorization | Project ID, actor from authenticated middleware | Star-native Project owner current membership/capability read | current grant revision and allowed capability projection; no client-supplied role |
| Associate Repository | Project ID, Repository ID, expected association revision, idempotency key | Project owner validates Project Manager and same-tenant Repository master; insert/update current association + audit + outbox + idempotency receipt | association ID/revision |
| Begin association retirement | Project ID, Repository ID, expected revision, idempotency key | Project owner CAS to `retiring`; emit fence event and audit/outbox; existing bindings remain valid | retirement operation ID/revision |
| Migrate or retire a Run binding | Run ID, binding ID, target association/root generation or explicit retirement reason, expected binding revision, idempotency key | Run owner revalidates current Project/Run grants and target association; owner-local CAS + audit/outbox | migrated/revoked binding receipt |
| Reconcile source references | Project ID, source association ID, operation ID, bounded cursor | Project owner retirement coordinator gathers complete revision-pinned receipts from Run and Worktree/Host owners | inventory revision, page cursor and completeness marker |
| Finalize association unlink | Project ID, Repository ID, operation ID, expected association revision, reconciliation proof, idempotency key | Project owner validates zero active references and complete fresh evidence; CAS `retiring`→`revoked` + audit/outbox | final revoked revision |
| List Run bindings | Run ID | fresh Project + Run authorization and owner read | only explicitly bound Repository summaries and binding revision; existing source binding may resolve while its association is retiring |
| Bind Run Repository | Run ID, Repository ID, root ID, expected binding revision, idempotency key | Run owner obtains current Project association revision and verifies Host/root identity before commit; CAS + audit + outbox | binding ID/revision, root generation, no path |
| Unbind Run Repository | Run ID, binding ID, expected revision, idempotency key | Run owner CAS + audit + outbox; revoke binding | revoked revision |
| Discover Worktrees | Run ID, binding ID, page cursor, bounded page size | authorize Project + Run; new bindings require active association; existing binding may use retiring source only until migration completes; verify binding, Host principal and root generation | sanitized page + opaque cursor + observed generation |
| Read Worktree details | Run ID, binding ID, Worktree opaque ID | same fresh authorization and binding checks | sanitized status and local ref only |
| Create/List/authorize Engineering Run | Project ID; authenticated actor; pagination for list | Run owner checks Project + Run grants without querying remote Branch or Infrastructure placement | Run identity and current authorization projection |
| Set/replace Run Infrastructure placement | Run ID, optional placement profile/target, expected binding revision, idempotency key | Run owner reauthorizes Project/Run, resolves profile and exact Host/cluster generations, validates capability and budget; owner-local CAS + audit/Outbox | versioned binding receipt; absence is represented by no binding, never a sentinel UUID/Branch |
| Submit infrastructure demand | V3 `trigger_origin` tagged variant + TaskExecutionRun attempt identity/number + current binding snapshot | Run owner creates demand only for execution requiring placement; Schedule and Manual variants each validate their own actor/origin, grant, binding/profile digest and scope; V3 Outbox insert shares owner transaction | `HostInfrastructureDemandV3`; no `branch_id` field or Schedule sentinel |
| Receive Infrastructure result | V3 event identity + demand reference | Run owner validates source demand and current execution scope, inserts V3 Inbox fact in same owner transaction, then acknowledges transport | durable owner-local receipt; no cross-owner transaction |

Bind/unlink operations are separate owner commands: Star-native Project owner owns Project identity, member grants and association truth; Run owner owns Run binding truth; Host Manager owns local root truth. No operation opens a distributed transaction across owners. If owner read is stale/unavailable, the mutation is rejected and retried from a fresh authorized read.

Run identity and optional infrastructure placement are separate from `RunRepositoryBinding`: remote Repository/provider/Branch metadata is optional provenance only and is not a parent, authorization input, or substitute for Worktree local checkout ref.

## §2 Project owner transaction

```text
BEGIN (Project owner)
  resolve ActorContext from authenticated middleware
  lock Project scope + expected association revision
  authorize Project manager capability
  validate Repository master exists and tenant_id matches
  compute request digest; reserve exact idempotency identity
  CAS association current projection
  append Audit fact + Project Outbox event
  persist idempotent result receipt
COMMIT
```

- Scope key: `(tenant_id, project_id, repository_id)`; history has unique revision per association.
- Active uniqueness prevents duplicate active link. Retiring is an admission fence: no new Run binding may reference it, while an existing binding remains resolvable until migration/retirement is verified. Revoked relation is preserved as history; re-association creates a new revision/event chain.
- `ProjectDefaultRepositoryPreference` is a separate Master fact, nullable and non-authorizing; its target must be an active association in the same Project. It is never copied into a Run binding.
- The Project owner never writes `scm.repository_master`; it references the stable ID and current master revision for validation. Reuse the existing `domain-project` command/query and `ProjectRepository` port boundary; provide a durable adapter for production. The current in-memory adapter is not production persistence. Do not create a second Project owner. The SCM master permits nullable provider/remote fields, so a local-only Repository does not require a remote Branch.
- The actor cannot write or read relations by tenant membership alone. `ProjectRepositoryAssociationChanged` includes identity/revision/status/correlation only, no path or repository content.

## §2.1 Project identity and membership authorization

- Principal identity and authentication are resolved by the identity/authentication owner; the Star-native Project owner stores Project identity and Project-scoped member/capability grants.
- Project creation and creator membership must commit atomically within the Project owner. Membership role/capability changes increment a monotonic grant revision and append durable audit/outbox facts.
- Every Project query or command resolves current membership from the owner transaction/API; stale browser state, cached role lists, tenant membership, Repository creator, or Run membership is insufficient.
- A Project member revoke invalidates subsequent Project-scoped Repository and Run-binding authorization checks; consumers may cache presentation data only, never the permission decision.

## §2.2 Association retirement and unlink algorithm

1. Begin with the current source `association_revision`; Project owner CASes active→retiring and writes an admission-fence event, audit, Outbox and idempotent receipt in one local transaction. Existing bindings can still resolve against this retiring source, but new bindings are denied.
2. Enumerate active Run bindings from their authoritative owner with a revision-pinned, bounded cursor. Missing owner, incomplete page, stale revision or timeout invalidates the reconciliation attempt.
3. For each reference, Run owner migrates it to another active association in the same Project for which the actor and Run remain authorized, and verifies the target Host/root generation; alternatively it explicitly retires a binding only after the relevant lifecycle owner confirms no checkout is still needed.
4. Each reference owner commits its own binding change, audit, Outbox and durable operation receipt. Retries use the same idempotency identity and return the original receipt. No shared cross-owner transaction or direct table write is allowed.
5. After all owner receipts are complete, Project owner performs a fresh revision-aware reconciliation proving zero active source references; final unlink uses expected source revision CAS, appends audit/outbox and returns a final receipt.
6. Any failure retains the retiring source association as valid for existing bindings and retains all unverified old bindings; no final revoke occurs. Recovery resumes from durable receipts and repeats reconciliation.

## §3 Run owner transaction

```text
BEGIN (Engineering Run owner)
  resolve authenticated ActorContext
  lock Run + binding identity; validate expected binding revision
  authorize current Project role AND current Run capability
  obtain Project owner current association (tenant, project, repository, revision)
  obtain Host Manager current (host, root, repository, generation) assertion
  ensure Project association is active for a new binding; retiring sources are accepted only by existing bindings under the unlink flow; enforce same tenant and current Project/Run grants
  CAS RunRepositoryBinding + append audit/outbox + idempotency receipt
COMMIT
```

- Run may have zero bindings; `RunRepositoryBinding` supports multiple explicit bindings, each to one exact Host root generation.
- The event references the association revision and root generation consumed at bind time. Before every discovery, currentness is re-read; binding-time validation is not a durable authorization grant.
- `retiring` blocks new binding commands but does not invalidate existing binding reads during migration; final `revoked` is allowed only after no active references remain. Host/root generation change still invalidates affected bindings. Consumers mark projections stale, while online owner validation is the correctness fence.
- Unbind does not delete history. Repeating an identical idempotency key returns the stored receipt; same key with different request digest is rejected.

### §3.1 Optional Infrastructure placement and V3 demand

Run can be created, authorized, listed and managed with only `(tenant_id, project_id, run_id)`. An Infrastructure binding is absent until an authorized Run operator explicitly chooses a placement. When present, its immutable revision pins the exact Profile revision/digest, HostEnvironment ID+generation, K3s cluster ID+generation, namespace, runtime profile and budget. Worktree local checkout ref remains Worktree-scoped. No Cloud Branch, null/nil UUID or fabricated Branch is accepted as a hostless sentinel.

For an execution that requires a specific environment, the Run owner rechecks current Project/Run grants and the full binding on admission and every resume/retry. It commits its own execution facts and V3 demand Outbox atomically; this records only a demand, not Host/cluster capacity reservation or runtime readiness. Infrastructure owner independently reserves Host and cluster capacity, then emits a result through its own Outbox; Run owner commits its Inbox receipt before transport acknowledgement. No placement or no confirmed readiness means the execution remains non-executable (fail closed). A Run operation that does not require placement remains available without it.

`HostInfrastructureDemandV3` scope is `(tenant_id, project_id, engineering_run_id, work_item_id, task_execution_run_id, attempt_number, trigger_origin)` and has no Branch field. `trigger_origin` is the mutually exclusive tagged union `schedule { schedule_rule_id, schedule_rule_revision, occurrence_id } | manual { manual_command_id, schedule_attribution? }`. Each schedule origin requires nonzero rule/revision/occurrence identities; a manual origin requires nonzero command identity. Absent fields belong to the other variant and are omitted, never replaced with nil UUIDs. `schedule_attribution` is an optional, owner-verified historical causal reference on a manual command; it does not change the trigger kind, run-as actor, authorization scope or idempotency origin key.

Each `task_execution_run_id` identifies one immutable execution attempt and `attempt_number` is positive and unique within its root origin. V3 also carries owner-derived, immutable `execution_actor_id`: Schedule resolves it from `run_as_actor_id` in the fixed Rule revision; Manual stamps it from the authenticated actor creating the command. The Run owner assigns and persists `manual_command_id` from that authenticated command request (or maps its request idempotency token to an owner identity); the client cannot set the effective actor. Schedule root identity is `(tenant_id, schedule_rule_id, schedule_rule_revision, occurrence_id)`; duplicate trigger delivery returns the existing root/initial attempt. Manual root identity is `(tenant_id, manual_command_id)`; replaying that command returns the original command result, while an intentional new manual start gets a new command ID. A retry/new attempt gets a new `task_execution_run_id` and unused attempt number while retaining the frozen root origin and, for Schedule, rule revision and occurrence. Redelivery/resume of the same attempt preserves the same attempt ID, event/idempotency key and request digest; same key with a different digest conflicts. New V3 idempotency scope includes schema version and a digest of the tagged root origin plus attempt number. The demand canonical digest also covers `execution_actor_id`, `schedule_attribution` presence/content, remote-provenance presence/content, placement presence/content and all Profile/Host/cluster generations.

For Schedule, the rule's immutable revision and resolved `run_as_actor_id` are reauthorized at trigger, retry and resume. For Manual, the actor stamped from authenticated command context is fixed in `execution_actor_id` and its current Run execution capability is revalidated at command admission and each retry/resume. Schedule attribution on a manual run is evidence only and never authorizes it. Optional remote provenance is likewise non-authorizing. The placement snapshot carries Profile identity/revision/digest, HostEnvironment identity/generation, K3s cluster identity/generation, namespace, runtime-profile identity/digest and bounded budget. New V3 writes use only V3 tables/domain/namespace.

### §3.2 Additive V2-to-V3 storage and replay boundary

Existing V2 demand/result records remain immutable historical facts with their exact stored `branch_id`, `schema_version=2` constraints, V2 decoder, canonicalizer and digest domain. The V2 Run owner contract is Schedule-only: `TaskExecutionRun.run_origin` must equal `schedule`; its current physical provenance fields are `automation_rule_id`, `automation_rule_version` and `automation_occurrence_id` (the domain-facing equivalent of Schedule rule ID/revision/occurrence), which the validator checks against the admission scope. Replay of a V2 key uses only that V2 contract. It must never clear/infer Branch, rewrite the event, or re-hash the same payload as V3. A Manual TaskExecutionRun cannot be emitted or admitted through V2 and must fail closed until a compatible V3 reader/writer is deployed. The additive migration introduces version-separated V3 demand Outbox/Inbox and Infrastructure inbox/receipt storage (or an equivalently isolated versioned relation); it does not weaken V2 `NOT NULL`/version checks or rewrite history. New writes use V3 after compatible V3 readers and migration are deployed.

Idempotency keys and dedupe indexes are schema-version scoped. For V3, root-origin key is `schedule(rule_id, rule_revision, occurrence_id)` or `manual(command_id)`; origin kind is included in the key digest so identical UUID bytes in the two variants cannot collide. Schedule occurrence uniqueness is `(tenant, rule_id, rule_revision, occurrence_id)`; manual command uniqueness is `(tenant, manual_command_id)`. Attempt uniqueness is `(tenant, origin_key_digest, attempt_number)` plus unique `task_execution_run_id`. The exact same attempt/key and canonical digest returns the stored receipt; a different digest conflicts; a new attempt requires a new TaskExecutionRun ID and unused attempt number. A key presented against another wire version is not an assumed replay; V2 and V3 do not share dedupe windows or acknowledgements. The full V3 digest includes the discriminant, required origin identity, optional schedule attribution presence/content, attempt identity/number and placement. The migration is auditable and forward-compatible: copy only remote metadata and grants proven by authoritative Project/Repository owners, preserve the original V2 payload/digest, and quarantine ambiguous, revoked or unverifiable records without inventing identity. Rollout order is additive migration → dual-version readers/replay tests → V3 writer enablement → measured V2 retirement decision. No SQL migration is part of this document-only slice.

## §4 Host root registry and path safety

Host root registry is a Host-local durable protected store controlled only by the trusted Rust Host Infrastructure Manager. It stores an opaque `root_id`, stable `host_id`, Repository ID, current generation, canonical root handle/reference, file identity, health state and registration audit. The absolute path and any OS access credential remain in the local protected store and are not serialized into server DTOs or event records.

Registration/update flow:

1. Local operator selects a root through a trusted Host UI/API that is not reachable to Agent/Plugin/Task CLI.
2. Manager resolves and opens the root using platform-native safe APIs and obtains stable filesystem identity. If the Repository has a trusted credential-free remote identity, it may validate that identity without persisting credentials. If the Repository is local-only, the operator must hold an active Project manager capability and explicitly attest `(RepositoryId, HostId, root_id)`; the attestation records actor, time, Project scope and correlation. Directory name or same-tenant membership never proves Repository identity.
3. Manager atomically increments generation and invalidates previous handles when the root moves, object identity changes, access is revoked, or Host identity rotates.
4. Discovery revalidates handle, identity, generation, and repository metadata immediately before use. Path replacement, symlink traversal outside the selected root, mount substitution, permission drift or generation mismatch rejects the operation.
5. If platform APIs cannot establish safe root identity or cancellation, that host provider is unavailable; no raw path fallback is allowed.

A filesystem identity proves that the selected object has not silently changed; it does not by itself prove that its content is the same remote repository. The trusted registration/attestation is the provenance for a local-only mapping. The trust boundary does not pass through application-supplied `PathBuf`, shell string, URL, plugin payload, or environment variable. Server-to-Host messages contain only opaque root ID + generation and a signed/authorized operation envelope.

## §5 Read-only Git provider

The provider API is capability-separated:

```text
ReadOnlyWorktreeProvider:
  list(binding_identity, page_request, deadline, cancellation) -> sanitized page
  get(binding_identity, worktree_id, deadline, cancellation) -> sanitized status
```

It does not expose `create`, `import`, `remove`, `archive`, `clean`, `checkout`, `merge`, `reset`, arbitrary command execution, or write-capable filesystem handles. Existing mixed `RunWorktreeLifecycleProvider`/legacy import adapters are not production implementations of this capability.

Execution rules:

- resolve a registered absolute Git executable at trusted installation; invoke with fixed argv through process API, never shell interpolation;
- use only the verified root handle/generation as cwd; close stdin; clear inherited environment and pass a small reviewed allow-list;
- one discovery subprocess per binding at a time; use configured deadline, cancellation token, row count and stdout/stderr byte caps;
- parse the entire output under a strict schema; malformed/duplicate/over-cap output fails closed; do not include raw output in error text;
- return one page with opaque IDs, state flags, local ref label and cursor. Absolute worktree paths and provider credentials are withheld;
- retain a projection only under a short explicit TTL keyed by current tenant/Project/Run/binding/Host/root generation. Cache cannot authorize reads or survive version/revocation changes.

Output fields are versioned DTO data. Detached HEAD, locked/prunable/missing state are explicit enum values, not fabricated branch names. A local ref string is display data, escaped and length-bounded; it is never treated as an API path or command argument.

## §6 RLS, role grants and owner isolation

- The physical Project association table belongs to the Project owner role; the Run binding table belongs to the Engineering Run owner role; the Repository master stays SCM-owned.
- FORCE RLS is tenant-scoped and, where applicable, also restricts Project/Run scope through trusted session context or owner-only procedures. Project identity/member grant master rows and Project association tables have Project-owner-only writes; client-supplied `tenant_id` is ignored.
- API app role receives only endpoint-specific execute/select permissions or minimal views; it cannot directly mutate owner tables. Project owner cannot write Run/SCM/Host records; Run owner cannot write Project association/SCM/Host records; Host Manager cannot change server Project/Run grants.
- Host local registry uses least-privilege OS ACL and protected storage; no app DB grant exposes path data to other services.
- Migration replay is idempotent and schema ownership/signature/search path are fixed. Grant fingerprint and RLS probes are part of target DB acceptance, not inferred from DDL text.

## §7 Event and idempotency envelopes

Owner Outbox payload minimum:

```text
event_id, schema_version, tenant_id, owner_type, owner_id,
aggregate_id, aggregate_revision, event_type,
correlation_id, causation_id, idempotency_key_digest,
association_or_binding_ref, host_id?, root_id?, root_generation?, occurred_at
```

No absolute path, Git URL credential, token, whole Repository object, raw process output or Worktree file content appears in the event. Relay is at-least-once; consumer Inbox deduplicates by immutable `event_id` and validates tenant/owner/schema before applying local projection. Ack occurs after consumer transaction commits. Event transport/provider can change without changing owner facts or consumer idempotency keys.

For Infrastructure demand, V2 and V3 are distinct wire contracts. V2 preserves its required `branch_id`, `schema_version=2` and Schedule-only origin validation. V3 declares `schema_version=3`, `tenant_id`, `project_id`, `engineering_run_id`, `work_item_id`, `task_execution_run_id`, positive `attempt_number`, owner-derived `execution_actor_id`, tagged `trigger_origin`, Schedule rule/revision/occurrence or Manual command identity, optional Manual `schedule_attribution`, `profile_id/revision/digest`, `host_environment_id/generation`, `cluster_id/generation`, `namespace`, `runtime_profile_digest`, `resource_budget`, optional normalized `remote_provenance`, and `canonical_digest`. V3 omits `branch_id` entirely. The digest uses a new domain label such as `star.infrastructure-demand.v3`; canonical JSON/CBOR encoding, field order, string normalization, omitted-vs-null rules, mutually exclusive variant fields and integer bounds must be frozen before implementation. Raw URLs, credentials and local refs are excluded.

## §8 Error contract

| Code | Meaning | Retry / security behavior |
|---|---|---|
| `PROJECT_SCOPE_DENIED` | Actor lacks active Project permission | do not reveal whether unrelated Repo exists |
| `RUN_SCOPE_DENIED` | Actor lacks Run capability | fail closed; no Host call |
| `ASSOCIATION_STALE` | Project relation revoked or revision changed | reject; refresh authorized choices |
| `BINDING_STALE` | expected Run binding revision is outdated | return current authorized revision summary |
| `HOST_UNAVAILABLE` | Host absent/offline/disabled | no fallback to local process or other host |
| `ROOT_GENERATION_STALE` | root moved/replaced/disabled | reject old binding and cache |
| `PROVIDER_UNAVAILABLE` | read-only adapter not installed | explicit unavailable; never enable mutation provider |
| `DISCOVERY_TIMEOUT` / `CANCELLED` | bounded read did not finish | terminate child; retain no partial page |
| `OUTPUT_LIMIT` / `INVALID_GIT_RESPONSE` | output exceeds profile or fails schema | abort; raw bytes remain local and are discarded |
| `CAPACITY_EXCEEDED` | Host/Run concurrency budget full | bounded retry guidance; no queue growth |

## §9 Sequence flows

### 9.1 Hostless Run

```text
Client → Run owner: Create Run(ProjectId, no Repo/Host)
Run owner: authorize Project membership → persist host-independent Run → Outbox
Client ← Run owner: Run created; bindings=[]
Client → Run owner: List Worktrees
Run owner: authorize current Project + Run → return unbound state
```

### 9.2 Worktree discovery

```text
Client → API: Discover(RunId, BindingId, cursor)
API → Project owner: current association authorization/revision
API → Run owner: current actor grant + binding revision
Run owner → Host Manager: authorized opaque root/generation + deadline/cancel
Host Manager: verify root handle/generation → fixed read-only Git list
Host Manager → Run owner: sanitized bounded page
Run owner → Client: revalidate binding versions → sanitized projection + opaque cursor
```

Any failed check short-circuits subsequent calls. No provider can query all Project roots then filter results in the client.

### §9.3 Deployment database profiles

| Profile | Authoritative Project database | Runtime rule | Evidence boundary |
|---|---|---|---|
| Vercel Web | Neon PostgreSQL | Only a server-side Project owner runtime may connect; browser and UI BFF have no direct database credentials or table-write access. The Star API/owner placement and Neon connection proxy/pool budget remain explicit deployment decisions. | A Neon-target migration, owner-role/RLS/grant and concurrent authorization test is required; a local PostgreSQL pass does not substitute. |
| Multipass/K3s | PostgreSQL inside a Multipass-managed Linux VM | Star API/owner runtime uses a fixed trusted endpoint and bounded pool; VM identity is distinct from K3s cluster/node identity. The test inventory cap is at most two Multipass instances; do not auto-create or purge. | Run the same schema/API/RLS/concurrency suite against the VM database independently; K3s readiness alone is not DB acceptance. |

The profiles do not share credentials, mutable data, transaction state or inferred authorization cache. Schema migrations are versioned and replayable in each profile. DB major version, Neon branch/region/pooler, VM-to-cluster placement, capacity, backup/restore and failover are OPEN until separately decided and tested.

## §10 没有完成的实现事实

该详细设计不声称 durable Project writer/API、关联迁移、Host local registry、只读 provider、Neon/Multipass 数据库接线、RLS/grants 或目标 PostgreSQL 验收已完成。当前 Rust/API/SQL demand 与 result 仍是要求 `branch_id` 且只接受 Schedule `run_origin` 的 V2 契约；手动 TaskExecutionRun 尚无 V3 demand 支持。V3 tagged origin/attempt storage、canonicalizer、读写、迁移与 replay/quarantine 行为尚未实现或验证。目标路由/provider 缺失时必须 unavailable；任何 in-memory adapter、random placeholder ID、mixed scan/import trait 或 process-local event channel都不能充当生产事实。`TD-WLD1`、`DD-LOCAL-INFRASTRUCTURE-001` 与 WBS 尚未同步，W11-A 保持 partial/open。

## §11 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.1 | 2026-10-25 | 细化 owner API/事务、Host root 安全、只读 Git capability、RLS/grants、事件和失败合同。 |
| v0.2 | 2026-10-26 | 绑定 Star 原生 Project owner 与现有 Project domain port；设计 member grant 持久边界、同 Project 迁移/退役、retiring fence、Neon/Multipass VM 数据库 profile。 |
| v0.3 | 2026-10-27 | 明确 Run identity 不含远端 Branch；细化可选 Infrastructure binding、V3 demand fields/digest 域、additive V2 replay/quarantine 与 owner-local Inbox/Outbox；标记未实现与 WBS/DD-LOCAL/TD 同步门。 |
| v0.4 | 2026-10-27 | 定义 V3 Schedule/Manual trigger-origin tagged union、owner-derived execution actor、optional schedule attribution、attempt uniqueness 与 digest/replay；记录 V2 要求 `run_origin=schedule` 和 Schedule provenance，手动来源仍 fail closed。 |
