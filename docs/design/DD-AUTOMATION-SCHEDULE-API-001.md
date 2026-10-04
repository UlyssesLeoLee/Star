# DD-AUTOMATION-SCHEDULE-API-001 — Detailed Design Addendum

> Status: 🟡 Verified implementation slice; route and target-environment verification remain open.
> Version: 0.7 | Date: 2026-10-04

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
3. Check active Project binding and authorize the Engineering Run and role.
4. Lock a deterministic transaction advisory key derived from tenant, Project, actor, operation, and SHA-256 of the idempotency key.
5. Delete an expired record for the exact key before reuse, then delete at most 64 unrelated expired replay rows visible to that actor/Project. If a live replay exists, return the saved status/body when the request hash matches; otherwise return `409 idempotency_key_reused`.
6. Resolve/lock current Worktree, Task link, Profile, catalog, and HookSet facts.
7. For revision, lock the current rule revision including `run_as_actor_id`, compare `expected_current_version`, and copy that identity unchanged; close the old `valid_to` with transaction `now()` and insert `version+1` with the same boundary. The SCD2 guard rejects other history mutation, and the run-as trigger serializes inserts for the Rule and rejects a changed actor.
8. Insert actor audit facts and a Run-consistent `schedule_rule.created`/`schedule_rule.revised` Outbox record. Save the 24-hour idempotency response.
9. Commit. Any failure before commit rolls back Master, Audit, Outbox, and replay state together.

## 4. Error and execution boundaries

Missing/foreign Project, Run, Worktree, Task, or Rule returns not-found to avoid revealing cross-scope resource existence. Missing scope/role returns forbidden. Version mismatch returns `409 schedule_rule_version_conflict`. Database/provider failures fail closed. The API does not start an occurrence or TaskExecutionRun; consumers and Run admission remain separate phase gates.

`run_as_actor_id` is the fixed identity for future unattended work, not a durable authorization grant. Each occurrence dispatch, retry, and resume must resolve the actor's current Project binding, Run grant, and execution capability. Revocation or ACL lookup failure rejects dispatch without creating a Run. Regranting does not change the creator identity; replacing the owner requires a new Rule. The worker and this authorization transition remain unimplemented.

## 5. Verification required before phase closure

- Re-run focused `rustfmt` and `cargo check --all-targets` after any change to this slice (the 2026-10-04 direct-toolchain checks passed).
- Completed: unauthenticated requests to collection GET/POST and item GET/PUT return 401; signed tokens missing the required scope return 403 before database access; a valid write-scope token with an oversized body returns 413. Rejection responses carry private/no-store headers.
- Still required: role matrix, Run/Project mismatch, target ownership, version conflict, page bounds, and authenticated no-store behavior on success and application errors.
- Idempotency tests for replay, mismatched key reuse, concurrent same-key create, and concurrent CAS revision.
- Completed in the bounded runner with disposable PostgreSQL 18.6: apply the 9F2/9F4A/9F4B migration chain twice; assert all seven Schedule tables use FORCE RLS; exercise tenant isolation, Run-consistent Outbox, append-only mutation rejection, SCD2 boundary, exact-key TTL replay reuse, initial creator/run-as equality, immutable run-as successor behavior, and occurrence-to-pinned-Rule identity equality.
- Compare actual migration privileges with target production database/runtime role grants; until confirmed, keep that release gate open.

## 6. Revision history

| Version | Date | Change |
|---|---|---|
| v0.1 | 2026-10-03 | Define routes, transaction sequence, and production closure gates. |
| v0.2 | 2026-10-04 | Record reviewed cache, TTL, SCD2, and Outbox persistence safety changes. |
| v0.3 | 2026-10-04 | Make the Rule-revision currentness predicate explicit and align validation evidence. |
| v0.4 | 2026-10-04 | Add request-level route-mount/error-header evidence and separate it from valid-auth and PostgreSQL authorization gates. |
| v0.5 | 2026-10-04 | Select the RustCrypto JWT backend and document authenticated scope/body-cap route evidence while retaining database and accepted-access gates. |
| v0.6 | 2026-10-04 | Specify non-client-controlled run-as identity, stable SCD2 inheritance, DB guard behavior, and mandatory per-trigger reauthorization while retaining the unimplemented worker/admission boundary. |
| v0.7 | 2026-10-04 | Specify DB creator binding and occurrence-to-rule identity invariants and record the completed three-migration/five-case PostgreSQL runner; retain production ACL/worker gates. |
