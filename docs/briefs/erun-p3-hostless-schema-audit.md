# Brief: erun-p3-hostless-schema-audit

**Agent**: reviewer
**Phase**: ERUN-P3.2-HOSTLESS-TASK-OWNER
**Created**: 2026-10-15 19:50:58

---

Read-only migration and authorization impact audit for moving Task Card canonical ownership to (tenant_id, project_id, engineering_run_id), with repository_id and branch_id as optional context only. Current candidate is d3f0e0ed on codex/run-task-command-port; dev is an ancestor. Inspect exact current source and report: all task metadata constraints/FKs/triggers, Worktree association guards, append-only Outbox payload/FKs/RLS/grants, and schedule/CLI currentness functions that still require remote Branch; a minimal additive migration that preserves legacy rows without guessed backfill; owner-local transaction and lock-order risks; precise operations that must remain fail-closed until broader migration. W/T/M classify every touched table. Do not edit files, run tests/DB commands, or claim production readiness. CodeGraph has no index in this worktree; use focused source inspection. Evidence starting points: db/migrations/2026-09-29-worktree-group-phase-2c.sql, 2026-10-01-engineering-run-directory.sql, 2026-10-02-task-run-owner.sql, 2026-10-02-task-run-engineering-run-binding.sql, 2026-10-10-run-owner-task-currentness.sql, 2026-10-10-run-owner-schedule-preparation.sql, 2026-10-10-run-owner-schedule-confirmation.sql, crates/infrastructure/src/run_task_card_pg.rs, crates/star-api-rest/src/group_api/work_items.rs.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
