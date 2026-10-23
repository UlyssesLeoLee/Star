# Brief: WLD3-PG-CHECKFIX-RESUME-20261023

**Agent**: worker
**Phase**: WBS_CONTINUATION_20261023
**Created**: 2026-10-23 04:54:22

---

Continue the bounded WLD-3 PostgreSQL acceptance lane in C:\Users\leo19\.codex\worktrees\wld3-occurrence-pg-20261022\Star, branch codex/wld3-occurrence-pg-20261022. Fix the checked-in disposable runner startup and the occurrence materialization fixture/constraint mismatch, then rerun the unmodified repeatable harness.

Current evidence: dev=eeb10853b19ae468bfe2c8189d32b948231b23e7; lane=5e99f447c07a15c4d1b396f38c56d53930781bd4 (9 commits behind dev, one unique commit). Unique commit touches scripts/automation/phase9f3_schedule.py, scripts/automation/test_schedule_occurrence_branch_snapshot.py and its retry report. Preserve existing untracked briefs/reports and .cache; check collisions before rebase. Orca reports no active agent/terminal. The dev retry report records: checked-in pg_ctl startup exit 1 after initdb passed; two diagnostic-only runs reached 31/32; schedule_postgres was 4/5 with SQLSTATE 23514 on occurrence_target_snapshot_branch_uuid_check; focused ACL test passed 1/1 twice. Diagnostic wrappers do not count as acceptance. WLD-3 remains open and Schedule execution remains fail-closed.
The failing test is crates/star-pg-adapter/tests/schedule_postgres.rs::materialization_is_idempotent_and_tenant_scoped; migration db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql. Hostless Rule configuration is valid but hostless materialization must remain rejected. Rust 1.99.0 is absent; 1.98.1 is diagnostic only; do not install/change host toolchains.

Inspect the branch and untracked evidence, then rebase onto local dev only after checking untracked collisions; do not drop/stash/delete previous evidence. Trace 23514 to the exact DB constraint and generated JSON snapshot. Fix the fixture/producer to provide the pinned non-nil Branch UUID; do not weaken/bypass the invariant, skip tests, or rely on a modified diagnostic-only runner. Repair Windows-safe startup using a unique disposable data dir and loopback port; clean only resources created by this run, never port 5432/target DBs/Multipass/Docker Desktop or unknown processes.
Run the unmodified runner twice for migration replay and behavioral acceptance including ACL, snapshot, tenant RLS, idempotency, leases/fencing and append-only facts. Run focused Python/static tests, Rust 1.99 tests if available, CodeGraph/Cypher Guardian for changed source, and git diff --check. Keep execution fail-closed; do not add worker, TaskExecutionRun admission, infra demand, executor or BI. Update only lane report and append-only JSONL; no shared WBS/automation-design/registry edits. Commit only lane files, cite this brief path, rebase onto current local dev when clean, and report SHA, exact evidence, unresolved gates and preserved resources. This is an acceptance-harness fix only and does not close WLD-3.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
