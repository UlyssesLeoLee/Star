# Brief: WBS-WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022

**Agent**: Codex subagent
**Phase**: WBS-20261022-WLD3
**Created**: 2026-10-22 00:47:10

---

# Status
**OPEN — local disposable PostgreSQL 18.6 acceptance passed 35/35 steps with Rust/Cargo 1.99.0.** The unchanged 2026-10-22 migration → 2026-10-23 correction → repeat sequence passed, including flattened Branch UUID assertions, ACL/RLS checks and 5/5 Schedule PostgreSQL tests. Two focused Schedule API authorization unit tests also passed. Schedule execution remains fail-closed; this local harness does not close worker/owner Saga, target-environment deployment, infrastructure reservation/readiness, or runtime sandbox gates.

# Target
Close the WLD-3 persistence gap that Schedule occurrences must independently require the target Branch snapshot even though hostless Run Rules may be configured without one. The persisted occurrence `target_snapshot` is the flattened serialized `ScheduleTargetV1`; therefore the required UUID is at top-level `target_snapshot.branch_id`.

# Change
Audit the existing AutomationOccurrenceSnapshotV1 / ScheduleTargetV1 contract and `automation.occurrence.target_snapshot` schema, then add a forward-only correction migration that replaces only the exact `occurrence_target_snapshot_branch_uuid_check` constraint with a top-level `branch_id` check. Keep the already committed `2026-10-22-schedule-occurrence-branch-snapshot.sql` unchanged: it is historical migration state and can already be recorded in `schema_migrations`, so editing it would not repair deployed databases. Apply the 2026-10-22 historical migration before the 2026-10-23 correction in the disposable harness, then replay both passes to verify the correction's DROP/ADD transaction is idempotent. Preserve branchless Rule configuration and do not create a worker, materializer, Run, or executor. Add focused flat-shape valid/invalid/nil checks; never use or mutate a target DB. Do not duplicate the already-present Domain-level occurrence validation unless the audit proves a missing executable path.

# Acceptance result and remaining WBS

The disposable local PostgreSQL 18.6 run used loopback port `58997` and UUID data directory `postgres-ae840a34-d899-4f5e-b638-8ec3b12c41b4`; all 35 steps passed, `pg_ctl stop` exited 0, and the owned data directory was removed. Raw result and step logs remain under `.cache/phase9f3-schedule/` and are intentionally not committed. The complete phase CLI stopped before the DB phase at a fail-fast rustfmt check on unrelated baseline files; the checked-in isolated PostgreSQL helper was then run directly and passed. This is local acceptance evidence, not a production target-environment or full WLD3 sign-off.

Still open: implement and test the Schedule worker/materializer and admission consumer; complete the Run/Execution-owner to Infrastructure-owner demand/reservation Saga with idempotent Outbox/Inbox and recovery; validate current authorization, RunInfrastructureBinding, host/cluster capacity and K3s/Agent Runtime sandbox readiness on the intended target environment; and test cancellation, restart/recovery, deployment grants/RLS, and migration rollback. Keep Schedule execution disabled until those gates pass.

# Constraints and ownership
Own only the forward correction migration, the narrowest test/harness files needed for the constraint, and `docs/reports/WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022.md`. Do not rewrite the committed 2026-10-22 migration. Avoid editing broad Schedule API/domain files unless required by concrete evidence. Preserve RLS/WORM and migration runner compatibility. Inspect the existing Cypher header conventions in SQL. CodeGraph index is absent; do not create one. Do not use competitor product names. Do not edit the central WBS or UI plan; parent will synchronize them after review.

# Observable acceptance
Demonstrate that a valid canonical top-level `branch_id` remains accepted and a null/missing/malformed/nil UUID is rejected after the historical migration and forward correction are applied in order, including a repeated correction application. Confirm branchless Rule storage remains valid. If PostgreSQL tooling is unavailable, run explicit static/harness checks and record the database limitation; do not claim DB acceptance. Run `rtk`-wrapped SQL/Rust checks available in this environment and record unavailable tools truthfully. No Schedule execution becomes enabled. The coordinator later authorized one lane commit after acceptance: commit only the reviewed WLD3 manifest, do not include `.cache` or raw logs, and do not rebase, merge, or clean this worktree.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
