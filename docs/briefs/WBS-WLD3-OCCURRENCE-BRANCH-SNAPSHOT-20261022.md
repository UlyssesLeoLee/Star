# Brief: WBS-WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022

**Agent**: Codex subagent
**Phase**: WBS-20261022-WLD3
**Created**: 2026-10-22 00:47:10

---

# Target
Close the WLD-3 persistence gap that Schedule occurrences must independently require the target Branch snapshot even though hostless Run Rules may be configured without one.

# Change
Audit the existing AutomationOccurrenceSnapshotV1 / ScheduleTargetV1 contract and `automation.occurrence.target_snapshot` schema, then add a minimal forward-only, idempotent PostgreSQL migration constraint that rejects occurrence snapshots without a valid non-null Branch UUID. Preserve branchless Rule configuration and do not create a worker, materializer, Run, or executor. Add a focused regression check in the existing migration/API harness where one exists; never use or mutate a target DB. Do not duplicate the already-present Domain-level occurrence validation unless the audit proves a missing executable path.

# Constraints and ownership
Own only the new migration, the narrowest test/harness files needed for the constraint, and `docs/reports/WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022.md`. Avoid editing broad Schedule API/domain files unless required by concrete evidence. Preserve RLS/WORM and migration runner compatibility. Inspect the existing Cypher header conventions in SQL. CodeGraph index is absent; do not create one. Do not use competitor product names. Do not edit the central WBS or UI plan; parent will synchronize them after review.

# Observable acceptance
Demonstrate that a valid canonical target Branch UUID remains accepted and a null/missing/malformed UUID is rejected at the database boundary or, when DB tooling is absent, by an explicit existing static/harness gate with that limitation recorded. Check exact migration re-run behavior and compatibility with prior migrations. Run `rtk`-wrapped SQL/Rust checks available in this environment and record unavailable PostgreSQL/Cargo tools truthfully. No Schedule execution becomes enabled. Do not commit. The eventual commit message will cite this brief path.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
