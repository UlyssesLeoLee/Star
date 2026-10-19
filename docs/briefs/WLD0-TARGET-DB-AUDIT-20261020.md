# Brief: WLD0-TARGET-DB-AUDIT-20261020

**Agent**: codex-worker
**Phase**: WLD0-DB
**Created**: 2026-10-20 08:48:55

---

## Objective
Perform a bounded, read-only review of the WLD-0 Run/Worktree binding database and authorization evidence. Identify what is proven by source or tests and what still needs a real target PostgreSQL run.

## Owned deliverable
Create only docs/reports/WLD0-TARGET-DB-AUDIT-20261020.md. Follow the repository's seven-section report format. Do not edit the WBS or product source.

## Inspect
- crates/star-api-rest/src/group_api/run_worktree_bindings.rs
- crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs
- db/migrations/2026-10-01-engineering-run-directory.sql
- db/migrations/2026-10-15-run-host-independent-directory.sql
- db/migrations/2026-10-19-run-worktree-bindings.sql
- directly referenced migration/role-grant runners and tests only as needed
Trace attach/reassign/detach, transaction/Outbox/idempotency, RLS/FORCE RLS, application-role grants, rollback/replay, concurrent CAS and duplicate-current-row handling.

## Constraints
- You are not alone in the repository. Do not revert or overwrite other work.
- No database writes, migrations, or tests against an unverified database. Never inspect or print secret environment values.
- Do not claim production or target-database acceptance from fixtures or source alone.
- Use CodeGraph first if an applicable index is available. In the parent review the graph results did not cover the binding handlers; the coverage-check tool was unavailable. Fall back to focused exact-source reads and literal searches, and state these limits.
- Run rtk proxy git diff --check for the owned report. Do not commit; the coordinator reviews and integrates it.

## Acceptance
Report the exact file/line evidence, available focused safe test command(s), whether an application DB role/grant contract is present, and the remaining WLD-0 gates. Explicitly keep execution closed if target authorization/database facts remain unknown.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
