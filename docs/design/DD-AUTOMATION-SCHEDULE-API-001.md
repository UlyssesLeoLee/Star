# DD-AUTOMATION-SCHEDULE-API-001 — Detailed Design Addendum

> Status: 🟡 Verified implementation slice; route and target-environment verification remain open.
> Version: 0.12 | Date: 2026-10-05

## 1. Routes

| Method | Path | Scope | Behavior |
|---|---|---|---|
| GET | `/api/v1/projects/{project_id}/engineering-runs/{run_id}/automation/schedule-rules` | `work-item:read` | Keyset page of current revisions, `limit` 1–100. |
| POST | same collection | `work-item:write` | Create version 1; requires `Idempotency-Key`. |
| GET | collection + `/{rule_id}` | `work-item:read` | Read current revision only. |
| PUT | collection + `/{rule_id}` | `work-item:write` | CAS successor revision; requires `expected_current_version` and `Idempotency-Key`. |

The router configures a 16 KiB JSON body cap and private response headers (`Cache-Control: private, no-store`, `Vary: Authorization`). The workspace explicitly enables `jsonwebtoken` RustCrypto, providing one deterministic backend for the existing RS256 issuer/verifier. Request-level tests run all four methods through production `build_group_router`: missing credentials produce 401; correctly signed JWTs missing the required read/write scope produce 403 before DB access; and a signed write-scope POST exceeding 16 KiB produces 413. Rejection responses carry both private headers. This is not evidence for accepted role/Project/Run authorization or persistence. No DELETE route exists: disabling a rule is a new immutable revision with `enabled=false`.

## 2. Write contract

The strict body uses `deny_unknown_fields` and includes `display_name`, `enabled`, cron expression, IANA timezone, DST gap/fold policies, overlap and misfire policies, pause policy, bounded retry policy, deadline, and target Worktree/Task/Profile IDs. Create rejects an `expected_current_version`; revise requires a positive value equal to the locked current version. The body does not accept `run_as_actor_id`: create derives it from the authenticated actor after current authorization, and revise reads it from the locked current row. The database trigger also requires the initial revision's `run_as_actor_id` to equal its `changed_by`, protecting the creator binding from direct database writers outside the REST API.

The server supplies parser/tzdb versions. It verifies the Run belongs to the route Project; then checks the active Worktree→Project/Run binding and active Task↔Worktree link. Every current-version predicate requires `valid_from <= now()` and `valid_to IS NULL`, including the Rule revision, Worktree Project binding, Profile, Provider, Skill, GrantSet, and HookSet. The server loads the current Worktree-scoped execution admission snapshot, including the verified Profile identity and effective HookSet digest. Caller-supplied digests or versions are never trusted.

The fully resolved `AutomationScheduleRuleRevisionV1` passes its domain validation and bounded materializer parser probe before persistence. Disabled input is validated by a temporary enabled clone; only the original disabled revision is persisted. Invalid parser/timezone/policy input returns `422 invalid_schedule_recurrence` or `422 invalid_schedule_rule`. Materialization copies the Rule principal into `AutomationOccurrenceSnapshotV1.run_as_actor_id`; domain validation requires equality with the nested Rule snapshot, the adapter writes the dedicated occurrence column, and a PostgreSQL trigger rejects identity drift from the exact referenced Rule version.

## 3. Transaction sequence

1. Validate Actor, scope, bounded request, route UUIDs, and idempotency header.
2. Begin a PostgreSQL transaction; set `app.tenant_id` and `app.actor_id`; apply statement and lock timeouts.
3. Authorize the authenticated editor against current Project and Engineering Run writer grants; the current Branch binding must also exist.
4. Lock a deterministic transaction advisory key derived from tenant, Project, actor, operation, and SHA-256 of the idempotency key.
5. Delete an expired record for the exact key before reuse, then delete at most 64 unrelated expired replay rows visible to that actor/Project. If a live replay exists, return the saved status/body when the request hash matches; otherwise return `409 idempotency_key_reused`.
6. For an enabled create, call `multica.lock_schedule_run_as_authorization(tenant_id, run_as_actor_id, project_id, run_id)`. For an enabled revision, first lock the current Rule row and compare `expected_current_version`, then call the helper with the immutable actor from that row. The helper verifies transaction tenant/actor GUCs and canonical Project/Branch/Run plus current grants, then takes a tenant+Project transaction advisory lock. Only after that call returns does the API issue the current ACL query as a separate SQL statement, requiring current Project/Run writer grants, current Branch binding, and active Run. `READ COMMITTED` therefore observes a revocation that acquired the lock first and committed while the helper waited. ACL BEFORE triggers for Project, Branch, and Run grant INSERT/UPDATE/DELETE take the same lock; moving a grant across Projects acquires both locks in UUID order. The runtime role has SELECT, but no UPDATE privilege, on the canonical ACL tables; the runner provisions EXECUTE on the helper after creating its runtime role. A disabled revision skips the creator recheck so an authorized manager can stop a rule after creator revocation.
7. Resolve/lock current Worktree, Task link, Profile, catalog, and HookSet facts. For revision, copy the locked `run_as_actor_id` unchanged; close the old `valid_to` with transaction `now()` and insert `version+1` with the same boundary. The SCD2 guard rejects other history mutation, and the run-as trigger serializes inserts for the Rule and rejects a changed actor.
8. Insert actor audit facts and a Run-consistent `schedule_rule.created`/`schedule_rule.revised` Outbox record. Save the 24-hour idempotency response.
9. Commit. Any failure before commit rolls back Master, Audit, Outbox, and replay state together.

## 4. Error and execution boundaries

Missing/foreign Project, Run, Worktree, Task, or Rule returns not-found to avoid revealing cross-scope resource existence. Missing scope/role returns forbidden. Version mismatch returns `409 schedule_rule_version_conflict`. Database/provider failures fail closed. The API does not start an occurrence or TaskExecutionRun; consumers and Run admission remain separate phase gates.

`run_as_actor_id` is the fixed identity for future unattended work, not a durable authorization grant. The create/enable API gate checks the actor's current Project/Run writer grants and active Run state; it does not enable a worker. Its exact SQL helper passes a disposable PostgreSQL 18.6 directory fixture: active grants authorize; revoked Project, Branch, or Run grants and a paused Run fail closed. The runtime role has no UPDATE privilege on canonical Directory ACL tables. Project, Branch, and Run grant-revocation races are serialized against the API authorization transaction by the shared Project lock; full API create/revise rollback remains untested. Each occurrence dispatch, retry, and resume must repeat the authorization check in the admission transaction and resolve current target/profile/HookSet/quota and execution capability. Revocation or ACL lookup failure rejects dispatch without creating a Run. Regranting does not change the creator identity; replacing the owner requires a new Rule. The worker and admission transition remain unimplemented.

## 5. Verification required before phase closure

- Re-run focused `rustfmt` and `cargo check --all-targets` after any change to this slice (the 2026-10-04 direct-toolchain checks passed).
- Completed: unauthenticated requests to collection GET/POST and item GET/PUT return 401; signed tokens missing the required scope return 403 before database access; a valid write-scope token with an oversized body returns 413. Rejection responses carry private/no-store headers.
- Completed after 9F4C-B: the pure role-policy helper matrix test passed 1/1 for accepted writer roles and viewer/agent/empty-role denial. A separate disposable PostgreSQL 18.6 test invokes the exact canonical Project/Branch/Run authorization helper as a read-only directory runtime role and proves an active developer grant succeeds while each revoked grant and a paused Run fail closed.
- Still required: SQL-backed coverage for the full accepted/denied role matrix, Run/Project mismatch, target ownership, version conflict, page bounds, create/revise rollback, concurrent CAS/replay, and authenticated no-store behavior on success and application errors.
- Idempotency tests for replay, mismatched key reuse, concurrent same-key create, and concurrent CAS revision.
- Completed in the bounded runner with disposable PostgreSQL 18.6: apply the 9F2/9F4A/9F4B/9F4C-A/9F4C-C migration chain twice; assert all eight Schedule tables use FORCE RLS; exercise tenant isolation, Run-consistent Outbox, append-only mutation rejection, SCD2 boundary, exact-key TTL replay reuse, initial creator/run-as equality, immutable run-as successor behavior, occurrence-to-pinned-Rule identity equality, and three concurrent ACL-revocation lock cases.
- Compare actual migration privileges with target production database/runtime role grants; until confirmed, keep that release gate open.

## 6. 9F4C-A persistence invariants

The new migration has an explicit prerequisite: the canonical `multica.task_execution_run` schema, including its tenant/run and tenant/project/task/run unique keys, must already be installed. It adds `admitted_run_id` to the Work-class dispatch row and an FK to `(tenant_id, run_id)`. The dispatch guard requires a non-null Run link for a new `leased → admitted` transition, checks the referenced TaskExecutionRun uses `run_origin='schedule'` and the same `automation_occurrence_id`, and makes the link immutable. Legacy admitted/terminal rows may remain null when no exact Run can be proven; no identity is guessed.

`automation.schedule_run_outbox` is a Transaction table with tenant FORCE RLS, append-only UPDATE/DELETE/TRUNCATE guards, and unique `(tenant_id, occurrence_id, event_type)`. Composite FKs pin its Rule revision and Engineering Run, occurrence and Project, and TaskExecutionRun and Task. An insert trigger verifies the Run is schedule-origin/agent-channel, has the same Engineering Run, rule/version/occurrence and `initiated_by`, and that the occurrence's immutable `run_as_actor_id` matches. The Outbox is distinct from `schedule_rule_outbox` (Rule changes) and `multica.task_run_outbox` (Task metadata changes).

The database layer permits setting the link only in the fenced state transition, but it does not itself know the worker's supplied lease owner/generation. The future application transaction must include both values and require current owner, generation, lease expiry, and occurrence deadline in its conditional update. It must also reauthorize the stored run-as identity and current Run/Worktree/Task/Profile/HookSet facts before writing Run, resource reservation, `run_started`/`schedule_occurrence_linked` RunEvents, occurrence `run_admitted` event, dispatch transition, and Schedule Run Outbox in one transaction. No production capability is connected yet.

The updated bounded runner passed on disposable PostgreSQL 18.6: the five Schedule migrations applied twice, eight Schedule tables had FORCE RLS, the canonical ACL fixture passed active/revoked/paused authorization cases with a runtime role that has no UPDATE privilege on directory ACL tables, and Project, Branch, and Run grant revocations each waited for a concurrent authorization transaction's Project lock. Once authorization committed, revocation completed and a later authorization failed closed. An admitted Run/Outbox pair passed identity constraints, mismatched run-as and Run-link mutation were rejected, Outbox UPDATE was rejected, and a `NOSUPERUSER NOBYPASSRLS` role observed only its tenant's row. This does not exercise target production DB grants, full REST write rollback, worker reauthorization, quota reservation, or a Rust admission service.

## 6.1 9F4C-C authorization and ACL mutation lock protocol

`multica.lock_schedule_run_as_authorization` checks non-null scope and equality with `app.tenant_id`/`app.actor_id`, then resolves the canonical Engineering Run, Branch, and current Project/Branch/Run grants before taking `pg_advisory_xact_lock(hashtextextended('star.schedule.run-as.authorization:project:' || tenant_id || ':' || project_id, 741391))`. The API performs its authoritative active-Run and writer-grant query in the next SQL statement, so `READ COMMITTED` obtains a fresh statement snapshot after any lock wait. The Project/Branch/Run ACL mutation triggers derive all affected canonical Project IDs from OLD/NEW rows, sort UUIDs, then take the same lock for each Project before the row mutation proceeds. Unresolvable mutation scope raises an exception. The transaction lock disappears automatically at commit/rollback; it adds no persistent table or global Project serialization. A `hashtextextended` collision can only cause an unrelated transaction to wait.

The runtime role has SELECT on canonical ACL tables and EXECUTE on the checked helper only; PUBLIC execution is revoked. The ACL mutation triggers run as the table's trigger execution context and do not grant the Schedule runtime role ACL writes. The runner grants helper EXECUTE only after creating its disposable role. Target database owner, trigger deployment, function ownership, and runtime grants remain to be verified. The race test proves API enable-time authorization versus grant mutation ordering; it does not test a future worker/admission transaction.

## 7. Revision history

| Version | Date | Change |
|---|---|---|
| v0.1 | 2026-10-03 | Define routes, transaction sequence, and production closure gates. |
| v0.2 | 2026-10-04 | Record reviewed cache, TTL, SCD2, and Outbox persistence safety changes. |
| v0.3 | 2026-10-04 | Make the Rule-revision currentness predicate explicit and align validation evidence. |
| v0.4 | 2026-10-04 | Add request-level route-mount/error-header evidence and separate it from valid-auth and PostgreSQL authorization gates. |
| v0.5 | 2026-10-04 | Select the RustCrypto JWT backend and document authenticated scope/body-cap route evidence while retaining database and accepted-access gates. |
| v0.6 | 2026-10-04 | Specify non-client-controlled run-as identity, stable SCD2 inheritance, DB guard behavior, and mandatory per-trigger reauthorization while retaining the unimplemented worker/admission boundary. |
| v0.7 | 2026-10-04 | Specify DB creator binding and occurrence-to-rule identity invariants and record the completed three-migration/five-case PostgreSQL runner; retain production ACL/worker gates. |
| v0.8 | 2026-10-05 | Specify fenced Run linkage and Schedule Run Outbox DB invariants; record repeated disposable PostgreSQL checks and keep worker authorization, quota, atomic writer, target DB and BI gates open. |
| v0.9 | 2026-10-05 | Specify editor/run-as separation, transactional authorization on enabled Rule writes, disable-after-revocation behavior, and the still-open per-trigger worker gate. |
| v0.10 | 2026-10-05 | Record the pure role-policy helper test separately from route/SQL authorization evidence; keep canonical ACL fixtures and per-trigger worker/admission verification open. |
| v0.11 | 2026-10-05 | Record disposable PostgreSQL Project/Branch/Run ACL cases under a read-only directory runtime role; keep full API transaction, concurrent revocation serialization, worker, and target-environment gates open. |
| v0.12 | 2026-10-05 | Specify the shared Project lock and separate-statement ACL reread; record five-migration repeated application, and Project/Branch/Run revoke-wait tests while preserving worker/admission and target-grant gates. |
