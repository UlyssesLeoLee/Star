# WLD-0 Run/Worktree Binding Design Audit

**Audit date:** 2026-10-18 JST
**Baseline:** `6f8d5b8f2d00b1515928c9a25e0762a45cf30e4e` (`codex/wbs-wld0-binding`)
**Scope:** Source-backed review of the canonical Project/Engineering Run/Worktree/Branch binding and its authority, persistence, lifecycle, and event boundaries. This report is an audit deliverable, not an implementation claim.

## 0. Purpose and conclusion

Determine whether one canonical, versioned contract currently covers Project-owned hostless Runs, optional Run-to-Worktree membership, Worktree-local Branch observation, authorization, owner commands, idempotency, stale/cross-scope rejection, lifecycle, RLS/grants, and lifecycle/BI events.

**Conclusion: partial contract exists; end-to-end canonical contract is not complete.** The intended Master relation is `multica.engineering_run_worktree_binding`, with SCD2 versioning, one current Run relation per Worktree, and an exact Project-binding reference. The host-independent migration permits a Run with no remote repository/Branch metadata. `WorktreeFocus` carries binding identifiers and versions. These pieces give a strong data and read-context foundation.

The unresolved pieces are material to a usable binding system: the reviewed REST router exposes Run and Worktree directory reads/context but no bind, detach, or rebind command; Worktree create/import remain Project-scoped; Branch is returned as an unqualified `w.branch` string without observation source/freshness; and the legacy task scope adapter rejects a hostless Run because it requires `branch_id`. The schema’s new idempotency/outbox tables are not represented in the DD’s eight-table W/T/M inventory. Target role grants and production database/runtime behavior remain unverified.

This audit does **not** claim WLD-0 implemented, migrations deployed, or a production Run/Worktree lifecycle available.

## 1. Evidence boundary

| Evidence tier | Inspected sources | What the evidence supports |
|---|---|---|
| Requirements intent | `docs/requirements.md` §50.1–50.3, WTG-001/005/006, AC-WTG-001 (lines 2231–2258, 2288–2295, 2446–2450) | Project → Engineering Run → Worktree → local Branch projection; Project-owned hostless Run; optional remote SCM metadata; selected Run Worktree focuses checkout; Run Apps share Run scope. |
| Basic design intent/status | `docs/basic-design.md` §16.1–16.4 and §16.9–16.12 (lines 4216–4332, 4490–4637) | Same navigation, RunContext and component boundary, event/owner concepts, W/T/M and acceptance trace; status paragraphs still record implementation/migration gaps. |
| Detailed design/schema contract | `docs/design/DD-WORKTREE-GROUP-001.md` §1.1–1.3, §3.2–3.4, §4–6 (lines 28–83, 123–207, 211 onward) | Product tree/routes, canonical context and focus DTOs, directory tables, authorization rules, and current implementation limitations. |
| Persistence source | `db/migrations/2026-10-01-engineering-run-directory.sql`; `db/migrations/2026-10-15-run-host-independent-directory.sql`; `db/migrations/2026-10-15-run-host-independent-task-owner.sql` | SCD2 relation, tuple constraints, tenant RLS policies/triggers, nullable remote Run metadata, idempotency and outbox schema, plus task-owner scope guards. |
| Application source | `crates/star-api-rest/src/group_api/engineering_runs.rs`, `worktree_lifecycle.rs`, `worktrees.rs`, `group_api.rs` | Current HTTP route surface, hostless Run creation/read, Worktree projection and focus verification, Project-scoped Worktree lifecycle, and separate archive management. |
| Test / database / runtime evidence | `docs/reports/PHASE-ERUN-DIRECTORY-P1-REPORT.md` §2 and §3; `docs/design/DD-WORKTREE-GROUP-001.md` header/status and §3.4 | A disposable PostgreSQL DDL exercise is reported for the directory migration. It does not prove target database role grants, the complete current migration chain, the binding command lifecycle, or production runtime behavior. No WLD-0-specific test or target runtime was run in this audit. |

The similarly named `docs/data-design/phase9f4d-run-owner-wtm.md` is specifically about infrastructure capacity demand/result facts (two Transaction tables, lines 1–24); it is not the W/T/M inventory for the directory Run↔Worktree relation. Directory W/T/M intent is currently embedded in DD §3.4 and migration comments.

## 2. Findings by required contract area

| Area | Current evidence | Assessment |
|---|---|---|
| Project-owned hostless Run | Requirements §50.1 and Basic Design §16.1/16.2 state Run ownership is Project/Run based and remote Branch is optional. The host-independent migration drops `NOT NULL` from Run `repository_id` and `branch_id` and adds `branch_id IS NULL OR repository_id IS NOT NULL` (`2026-10-15-run-host-independent-directory.sql:23–39`). `create_project_run` writes both as NULL and creates revision, explicit Run grant, Outbox fact and 24-hour command receipt in its transaction (`engineering_runs.rs:509–666`, especially 590–655). | **Source-backed for directory creation/read.** Does not establish full hostless Run apps/CLI execution. |
| Optional Run↔Worktree relation | DD §3.4 calls `engineering_run_worktree_binding` an SCD2 Master, one current Run per Worktree, tied to exact historical Project binding (`DD-WORKTREE-GROUP-001.md:194–205`). Migration 2026-10-01 uses a partial unique index on `(tenant_id, worktree_id) WHERE valid_to IS NULL` and versioned rows (`:191–220`). A Worktree without a current row is possible; the row is not mandatory on `engineering_run`. | **Schema intent is clear:** 0 or 1 current Run per Worktree; one Run can have many Worktrees. **Command lifecycle is absent from reviewed REST routes.** |
| Remote versus local Branch | Run `repository_id`/`branch_id` are optional remote host metadata in migration comments (`2026-10-15-run-host-independent-directory.sql:256–261`). Worktree directory selects `w.branch` as a string (`engineering_runs.rs:704–715`) and exposes that value in RunWorktreeRow; no local observation timestamp/source/status is in this DTO. The local Worktree model and Project Index also return `w.branch` (`worktrees.rs:267–278`). | The UI hierarchy says local Branch under Worktree, but source does not yet publish a typed Branch observation contract. Remote `branch_id` and local branch ref must remain distinct. Missing/stale observer state must be `unknown`, not inferred from optional remote metadata. |
| Owner commands | DD §2 defines a `DomainOwnerCommandGateway` to call a fact owner and write Audit/Outbox in the owner transaction (`:123–141`), and §4 scopes routes. In source, Run creation is implemented directly in the REST handler using one SQLx transaction (`engineering_runs.rs:509–666`). Worktree create/import use a typed `ProjectWorktreeLifecycleProvider` (`worktree_lifecycle.rs:70–182, 228–246`). | There is no single explicit Run/Worktree binding command owner/port. A future command must define the owner (directory owner), typed request/result, transaction boundary, and how current authorization and binding facts are locked. |
| Actor, grants, and authorization | Run create checks current Project manager role before creating the Run, then writes a Run role grant for the actor (`engineering_runs.rs:519–529, 613–617`). `run_authority` resolves current Project and Run grant (`engineering_runs.rs:360–384`). DD §3.2 says normal Run Apps do not require remote Branch grant; SCM operations do (`:168–186`). Directory migration requires transaction-local tenant/actor/correlation and current Project manager for writes (`2026-10-01-engineering-run-directory.sql:59–63, 343–354`) and forces tenant RLS (`:444–478`). | **Read/create authority has a source path.** No binding command exists to demonstrate per-command actor revalidation or exact target Worktree authorization. Database policies are not a substitute for application role GRANT proof. |
| Idempotency | Run creation requires `Idempotency-Key`, hashes the request, performs actor/tenant-scoped replay/conflict checks, and writes a receipt in the same transaction (`engineering_runs.rs:523–584, 653–660`). Migration adds actor-scoped idempotency with a maximum 24-hour expiry and FORCE RLS (`2026-10-15-run-host-independent-directory.sql:212–232`). | **Run create only.** No binding/detach/rebind command means there is no replay contract for relation mutations. Project Worktree create/import idempotency is a separate command and does not bind a Run (`worktree_lifecycle.rs:228–246, 388–455`). |
| Stale and cross-scope rejection | Run `WORKTREE_SELECT` joins a current Run binding, its exact current Project binding, and the Worktree by tenant/project/repository (`engineering_runs.rs:704–715`). `load_focus` locks those joined rows for share (`:753–765`). Migration FKs tie the binding to `(tenant, project, run)` and exact Project binding; insert trigger checks current Project binding, Worktree repository, optional Run repository/remote Branch consistency (`2026-10-15-run-host-independent-directory.sql:56–60, 189–207`). | **Read-path and insert-time protection exist in source/schema.** There is no API-level expected-version/CAS contract for a mutation, no exercised stale replay or cross-Run conflict test, and no target DB proof. |
| Hostless downstream scope | `RunOwnedTaskScope` preserves optional repository/branch, but `authorize_run_task_scope` converts it to an execution scope requiring repository and non-null host Branch (`engineering_runs.rs:386–406, 408–423`). The host-independent task-owner migration permits nullable `branch_id`, while CLI/session code selects and joins a remote Branch grant (`cli_sessions.rs:974–1195`). | **Important end-to-end mismatch:** hostless Run directory does not yet imply hostless Task CLI/Execution capability. Such execution must remain disabled/fail-closed until an owner/runtime contract no longer treats remote Branch as required. |
| Detach, rebind, archive, restore, recovery | SCD2 Master rows are close-once/append-successor per DD §3.4 (`:194–207`). Run router registers only Run directory list/create, legacy Branch list, Run Worktree GET, and RunContext GET routes (`engineering_runs.rs:266–288`). Project Worktree creation/import are separate (`worktree_lifecycle.rs:228–246`). Worktree archive/restore runs through versioned management plans and changes Worktree archive state (`worktrees.rs:144–164, 381–518, 753–775`); it does not mutate Run binding. Run listing query does not filter archived Worktrees (`engineering_runs.rs:704–715`). | DB history constraints exist, but there is no explicit bind/detach/rebind endpoint or recovery command. Archive currently preserves Run relation and visible archived status by read-path shape; design should say archive is not detach. Recovery and active-agent/CLI drain conditions are not specified for a binding transition. |
| Database roles and RLS | Directory base migration enables/forces tenant RLS and write triggers; hostless migration adds FORCE RLS to idempotency and RLS to Outbox (`2026-10-15-run-host-independent-directory.sql:212–255`). `PHASE-ERUN-DIRECTORY-P1-REPORT.md:22` reports eight directory tables with FORCE RLS in a disposable database; it notes a stand-in for a legacy trigger and does not prove production role behavior. The two directory migrations contain no `GRANT`/`REVOKE` statements (bounded search). | **Policy/trigger DDL exists; role-grant contract remains unproven.** Target application DB roles and least-privilege grants must be explicitly inventoried and tested. |
| Lifecycle / BI events | `engineering_directory_event` is trigger-created append-only audit, not an outbox (`2026-10-01-engineering-run-directory.sql:222–258, 391–439, 494–500`). The hostless migration creates an append-only `engineering_run_outbox`; its table comment explicitly says the claim/retry/delivery worker is not implemented (`2026-10-15-run-host-independent-directory.sql:234–267`). Current Run create writes `engineering_run.created` (`engineering_runs.rs:639–651`). | Directory audit, delivery, and BI facts are distinct. There is no source-backed `WorktreeBound/Detached/Rebound` outbox event, no delivery consumer, and no proven BI/Benchmark projection for these transitions. |

## 3. Contract gaps and design inconsistencies

1. **Partial versus complete binding contract.** SRS and DD define the tree and relation, and SQL enforces versioned tuples, but there is no write API/owner command to create or transition the relation. Read DTO version fields alone do not supply mutation semantics.
2. **Remote Branch versus local Branch ambiguity.** The Run relation’s `branch_id` is a remote SCM identity; the UI leaf is a local checkout observation. The Worktree projection currently exposes only `w.branch` with no freshness/source envelope, so consumers cannot distinguish an authoritative fresh observation from cached/imported text.
3. **Hostless scope ends at the directory boundary.** Run creation succeeds with null remote metadata, but the legacy task scope adapter and CLI authorization still demand a host Branch. The basic/SRS promise must not be interpreted as execution readiness.
4. **W/T/M inventory is stale/incomplete.** DD §3.4 says eight tables, `3T/5M/0W` and full FORCE RLS. The 2026-10-15 hostless migration adds `engineering_run_command_idempotency` (Work, 24-hour receipts) and `engineering_run_outbox` (Transaction, append-only delivery source), but the DD table inventory and W/T/M count do not include them. The existing `phase9f4d-run-owner-wtm.md` covers infrastructure demand/result instead.
5. **Authorization comments diverge across migration versions.** The 2026-10-01 comments say Branch/Run grants are mandatory for reads (`:60–62`) and Run grant consumers must also validate Cloud Branch grant (`:492–493`); 2026-10-15 changes the Run grant comment to say remote Branch grant is not required (`:256–261`). The active semantics should be stated once in DD and migration notes: current Project membership + Run grant for Run Apps; Branch grant only for remote SCM actions.
6. **Binding audit is not BI delivery.** The trigger-only `engineering_directory_event` is durable audit evidence; the separate Outbox schema is not wired for binding transitions. Do not count the trigger audit row as delivered BI data.
7. **Schema evidence is narrower than operational readiness.** A disposable DDL run does not establish current target database migration state, app role grants, cross-owner behavior, runtime freshness, command replay, event delivery, or execution capability.

## 4. Proposed canonical contract for coordinator review

Adopt one versioned `RunWorktreeBinding`/`WorktreeFocus` contract across SRS, Basic Design, DD, DTOs and migration comments:

```text
RunWorktreeBinding v1 {
  tenant_id, project_id, engineering_run_id, worktree_id,
  binding_id, binding_version, valid_from, valid_to?,
  project_binding_id, project_binding_version,
  repository_id, remote_branch_id?,
  actor_id, correlation_id
}

WorktreeBranchObservation v1 {
  worktree_id, repository_id, local_ref?, head_commit_id?,
  observed_at?, source_runtime_id?, observation_version?,
  state: current | stale | unavailable | unknown
}
```

Contract rules:

- Canonical Run owner is `(tenant_id, project_id, engineering_run_id)`. Remote repository/Branch identities are nullable metadata; they neither parent the Run nor authorize ordinary Run Apps.
- A Project Worktree may have zero or one current Run binding. One Run may have multiple Worktrees. A binding must reference the exact current Project binding identity/version and the Worktree’s repository. If remote Branch metadata is supplied, validate its Project/repository and applicable capability separately.
- Run App reads require current Project membership and the current Run grant. Worktree/Git/CLI operations additionally require a current Worktree binding/focus proof and their own capability. Remote SCM actions additionally require Branch-specific authority. Human owner fields never create grants.
- Bind, detach and rebind are typed commands owned by the directory owner and committed with actor, expected Run/Worktree/Project-binding versions, correlation ID, and an actor-scoped idempotency key/fingerprint. Exact replay returns the original receipt; same key with a different request conflicts. Reject stale versions, wrong tenant/project/repository, Worktree bound to another Run, inactive Project binding, revoked authority, and active execution conflicts.
- Detach closes the current SCD2 binding; rebind closes the old row and inserts a successor atomically. Preserve history; do not physically delete or guess-backfill. Archive/restore changes Worktree visibility/state and does not silently detach/rebind. Define explicit drain/restore behavior for running Agent/CLI sessions and interrupted owner commands.
- Local Branch is an observation scoped to a Worktree/repository, not `scm.cloud_branch`. Include source and observation time; stale, missing, or unavailable observations render `unknown`. Do not infer a local branch from Run remote Branch metadata.
- Commit a typed `WorktreeBound`, `WorktreeDetached`, or `WorktreeRebound` event and the owner’s Outbox fact in the same transaction. Keep directory audit, transport receipt and BI facts distinct. BI/Benchmark consumes only delivered, deduplicated owner facts and exposes freshness/lag.
- Complete database role grants and FORCE RLS validation for the production application role. The target DB test must cover tenant isolation, actor attribution, duplicate key replay/conflict, stale CAS, cross-Project/Run/repository rejection, close-once history, and outbox atomicity. Production readiness is a separate gate.

This is a proposal where source semantics are incomplete; it is not implemented by this audit.

## 5. File-by-file design update matrix

| File | Required coordinator update |
|---|---|
| `docs/requirements.md` §50 | Preserve Project → Run → Worktree → Branch. Add the binding cardinality and explicit attach/detach/rebind/archive/restore semantics; distinguish optional remote Branch metadata from local Branch observation; add acceptance cases for hostless CLI fail-closed/current capability, CAS/idempotency, scope rejection and delivered BI facts. Clarify that hostless directory support is not execution readiness. |
| `docs/basic-design.md` §16.1–16.4, §16.9–16.12 | Name the directory binding owner and commands; define WorktreeBranchObservation DTO/freshness, authorization split, event/Outbox flow, and lifecycle state transitions. Update component/data ownership and W/T/M inventory, RLS/grants and acceptance traces. |
| `docs/design/DD-WORKTREE-GROUP-001.md` §1–6 and §3.4 | Publish the versioned contract, API routes/command DTOs, expected-version lock/CAS behavior, error semantics, SCD2 close-successor sequence, archive independence, recovery/drain guard and event contract. Reconcile the old “eight tables / 3T+5M” statement with the two Oct 15 tables. Remove/replace stale Branch-grant-required text where it contradicts hostless Run semantics. |
| `docs/data-design/phase9f4d-run-owner-wtm.md` and directory W/T/M source | Keep the current infrastructure demand/result classification scoped to infrastructure. Add/locate a dedicated directory W/T/M inventory covering all current directory tables, including Work idempotency and Transaction Outbox, retention, RLS and audit obligations. |
| `db/migrations/2026-10-01-engineering-run-directory.sql` / `2026-10-15-run-host-independent-directory.sql` | When implementing, retain tuple/FK/trigger safety; align comments, role grants, binding event/outbox production, command idempotency retention, FORCE RLS and SCD2 constraints with the approved DD. Do not treat a migration file as applied evidence. |
| `crates/star-api-rest/src/group_api/engineering_runs.rs`, `worktree_lifecycle.rs`, `worktrees.rs` | Add binding owner commands only after contract approval; do not infer Run from a Worktree or remote Branch; add read DTO observation freshness; keep archive separate; reconcile hostless CLI path before advertising execution. |
| `docs/reports/PHASE-ERUN-DIRECTORY-P1-REPORT.md` and future WLD reports | Record exact current migration chain, target role/grant tests, binding command test outcomes, delivery/BI behavior, and runtime evidence separately. Keep existing disposable DDL result scoped to what was actually tested. |

## 6. Acceptance gates for WLD-0 closure

- One versioned contract is referenced consistently from SRS → Basic Design → DD → W/T/M inventory and has no remote/local Branch ambiguity.
- Project-hostless Run create/read works with current Project membership and Run grant; absence of remote Branch does not grant SCM authority and does not silently enable CLI execution.
- Run↔Worktree relation is optional, one-current-per-Worktree, project/repository exact, versioned, and only mutable through typed idempotent owner commands with atomic audit/outbox.
- Bind, detach, rebind, archive, restore, actor revoke, stale version, same-key replay/conflict, cross-tenant/project/run/repository, active-agent drain and recovery behaviors have explicit tests.
- Local Branch observation includes provenance/freshness and fails closed to `unknown` when stale/unavailable.
- Target PostgreSQL role grants, FORCE RLS, trigger authority, transition history and outbox atomicity are tested with the actual application role. Disposable schema validation and production target state are reported separately.
- BI/Benchmark receives only deduplicated delivered binding facts, with lag and replay evidence; trigger audit alone is not accepted as BI delivery.

## 7. Review and validation summary

- Reviewed the bounded source paths named in the dispatch brief; `.codegraph` is absent in this worktree and was not initialized.
- No canonical SRS, Basic Design, DD, data-design, production Rust, migration, test fixture, database, or runtime was modified or executed.
- Validation for this audit artifact: `rtk git diff --check`; manifest-only path/status verification; manual self-review against the brief. No Cargo, test, PostgreSQL or runtime success is claimed.
- Coordinator disposition required: approve the proposed contract and file update matrix before WLD-1 implements binding commands or mutates canonical design.
