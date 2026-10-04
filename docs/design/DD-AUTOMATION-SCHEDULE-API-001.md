# DD-AUTOMATION-SCHEDULE-API-001 — Detailed Design Addendum

> Status: 🟡 Verified implementation slice; route and target-environment verification remain open.
> Version: 0.3 | Date: 2026-10-04

## 1. Routes

| Method | Path | Scope | Behavior |
|---|---|---|---|
| GET | `/api/v1/projects/{project_id}/engineering-runs/{run_id}/automation/schedule-rules` | `work-item:read` | Keyset page of current revisions, `limit` 1–100. |
| POST | same collection | `work-item:write` | Create version 1; requires `Idempotency-Key`. |
| GET | collection + `/{rule_id}` | `work-item:read` | Read current revision only. |
| PUT | collection + `/{rule_id}` | `work-item:write` | CAS successor revision; requires `expected_current_version` and `Idempotency-Key`. |

All route handlers cap JSON bodies at 16 KiB and return authenticated JSON with `Cache-Control: private, no-store` and `Vary: Authorization`. No DELETE route exists: disabling a rule is a new immutable revision with `enabled=false`.

## 2. Write contract

The strict body uses `deny_unknown_fields` and includes `display_name`, `enabled`, cron expression, IANA timezone, DST gap/fold policies, overlap and misfire policies, pause policy, bounded retry policy, deadline, and target Worktree/Task/Profile IDs. Create rejects an `expected_current_version`; revise requires a positive value equal to the locked current version.

The server supplies parser/tzdb versions. It verifies the Run belongs to the route Project; then checks the active Worktree→Project/Run binding and active Task↔Worktree link. Every current-version predicate requires `valid_from <= now()` and `valid_to IS NULL`, including the Rule revision, Worktree Project binding, Profile, Provider, Skill, GrantSet, and HookSet. The server loads the current Worktree-scoped execution admission snapshot, including the verified Profile identity and effective HookSet digest. Caller-supplied digests or versions are never trusted.

The fully resolved `AutomationScheduleRuleRevisionV1` passes its domain validation and bounded materializer parser probe before persistence. Disabled input is validated by a temporary enabled clone; only the original disabled revision is persisted. Invalid parser/timezone/policy input returns `422 invalid_schedule_recurrence` or `422 invalid_schedule_rule`.

## 3. Transaction sequence

1. Validate Actor, scope, bounded request, route UUIDs, and idempotency header.
2. Begin a PostgreSQL transaction; set `app.tenant_id` and `app.actor_id`; apply statement and lock timeouts.
3. Check active Project binding and authorize the Engineering Run and role.
4. Lock a deterministic transaction advisory key derived from tenant, Project, actor, operation, and SHA-256 of the idempotency key.
5. Delete an expired record for the exact key before reuse, then delete at most 64 unrelated expired replay rows visible to that actor/Project. If a live replay exists, return the saved status/body when the request hash matches; otherwise return `409 idempotency_key_reused`.
6. Resolve/lock current Worktree, Task link, Profile, catalog, and HookSet facts.
7. For revision, lock the current rule revision and compare `expected_current_version`; close the old `valid_to` with transaction `now()` and insert `version+1` with the same boundary. The SCD2 guard rejects other history mutation.
8. Insert actor audit facts and a Run-consistent `schedule_rule.created`/`schedule_rule.revised` Outbox record. Save the 24-hour idempotency response.
9. Commit. Any failure before commit rolls back Master, Audit, Outbox, and replay state together.

## 4. Error and execution boundaries

Missing/foreign Project, Run, Worktree, Task, or Rule returns not-found to avoid revealing cross-scope resource existence. Missing scope/role returns forbidden. Version mismatch returns `409 schedule_rule_version_conflict`. Database/provider failures fail closed. The API does not start an occurrence or TaskExecutionRun; consumers and Run admission remain separate phase gates.

## 5. Verification required before phase closure

- Re-run focused `rustfmt` and `cargo check --all-targets` after any change to this slice (the 2026-10-04 direct-toolchain checks passed).
- Route tests for scope/role, Run/Project mismatch, target ownership, version conflict, request bound, and no-store headers.
- Idempotency tests for replay, mismatched key reuse, concurrent same-key create, and concurrent CAS revision.
- Disposable PostgreSQL repeat application of the 9F2 and 9F4A migrations; assert the two new API tables use FORCE RLS; exercise tenant isolation, Run-consistent Outbox, append-only mutation rejection, SCD2 boundary, and exact-key TTL replay reuse.
- Compare actual migration privileges with target production database/runtime role grants; until confirmed, keep that release gate open.

## 6. Revision history

| Version | Date | Change |
|---|---|---|
| v0.1 | 2026-10-03 | Define routes, transaction sequence, and production closure gates. |
| v0.2 | 2026-10-04 | Record reviewed cache, TTL, SCD2, and Outbox persistence safety changes. |
| v0.3 | 2026-10-04 | Make the Rule-revision currentness predicate explicit and align validation evidence. |
