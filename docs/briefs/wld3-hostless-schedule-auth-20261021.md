# Brief: wld3-hostless-schedule-auth-20261021

**Agent**: worker
**Phase**: wld3-hostless-schedule-auth-20261021
**Created**: 2026-10-21 22:13:11

---

## Objective
Allow Schedule Rule authorization to address an Engineering Run that exists without a remote-host Branch. Preserve the user's Project → Run → Worktree → Branch hierarchy and the existing fail-closed execution boundary.

## Evidence and boundary
- Current CodeGraph query on the dev worktree found `authorize_schedule_run_as` and `schedule_run_authorization_sql` in `crates/star-api-rest/src/group_api/schedule_rules.rs`. The authorization query joins `scm.cloud_branch` and its active revision, and returns a `RunTaskScope` with `branch_id`; Schedule create/revise fixes `run_as_actor_id` and checks it at rule write time.
- Current WBS says WLD-3 Schedule worker/admission remains open. This task is only the hostless identity/authorization slice; do not implement or enable a Schedule executor/worker.
- CodeGraph coverage-check tools are not exposed in this session. Use targeted source inspection for every affected file and report this limitation; do not make exhaustive claims from graph evidence alone.

## File ownership
Own only the smallest necessary Schedule authorization/API code, its migrations and focused tests, plus a task-specific report. Likely starting point: `crates/star-api-rest/src/group_api/schedule_rules.rs` and relevant Schedule authorization/migration tests. Do not edit the shared WBS or UI plan; the coordinator will reconcile those after integration. Do not overwrite or revert other work.

## Required behavior
1. A valid hostless Engineering Run can be listed/read and can have a Schedule Rule created/revised when the current Project and Run grants allow it. Authorization must not require a `scm.cloud_branch` row or invent a Branch identifier.
2. Keep the Run active-state check, tenant scope, current Project+Run writer checks, current authorization serialization/locking, idempotency, audit, and Outbox behavior. Preserve existing branch-backed Runs.
3. Keep `run_as_actor_id` immutable across revisions. This slice does not replace per-trigger/retry/resume authorization; document that runtime-time reauthorization remains a WLD-3 gate.
4. Preserve all target bindings that genuinely apply (Task, Worktree, Profile, HookSet). Do not weaken those checks or substitute mock/seed data. If the existing data contract makes hostless scheduling impossible without a broader schema change, implement the narrowest compatible schema/API change and clearly record remaining blockers.
5. No executor, shell, task session, or infrastructure demand may start from this API slice. Missing runtime identity/provider/capacity/readiness remains fail-closed.

## Verification and documents
- Add focused tests covering hostless authorized Run, unauthorized/revoked Project or Run permission, archived Run, and continued branch-backed behavior. Exercise PostgreSQL/target migration behavior when the disposable DB harness is available; otherwise report exactly which checks were not run.
- Compare code behavior with requirements, basic design, Group detailed design and the WLD plan. Update only the affected canonical design section(s) if they still require a remote Branch for Run authorization; include a revision note.
- Write `docs/reports/WLD3-HOSTLESS-SCHEDULE-AUTH-20261021.md` with changed files, tests and limitations.
- Before handoff: inspect your own diff, run the required focused Rust formatting/build/tests available on this machine, confirm the new migration is replay-safe, and commit a single logical change. Commit message must reference `docs/briefs/wld3-hostless-schedule-auth-20261021.md`.

## Product/repo constraints
Follow the current `AGENTS.md`: Rust 1.99.0 baseline; no competitor comparison/imitative product copy; owner-local transactions and Outbox; no cross-owner direct writes; bounded work; preserve dirty user data; no false production-ready claim. You are not alone in the repository: other workers have separate file ownership. Do not revert their edits. Use the assigned worktree only.

## Graph/source evidence supplied by coordinator
- `schedule_rules.rs`: `create_rule`/`revise_rule` preserve creator identity; `authorize_schedule_run_as` delegates to `authorize_run_principal(..., require_active_run=true)`; the SQL currently requires active cloud Branch and grants.
- Verify exact current source in the assigned worktree; this brief is not a substitute for reading the affected migration/test code.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
