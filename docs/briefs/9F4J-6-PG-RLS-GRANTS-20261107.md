# Brief: 9F4J-6-PG-RLS-GRANTS-20261107

**Agent**: worker
**Phase**: WBS-9F4J-6-TRIGGER-REVOKE
**Created**: 2026-11-07 21:46:12

---

## Objective
Resolve the statically confirmed PUBLIC EXECUTE gap for the two BI Task Event trigger-only functions using a forward-only migration, and strengthen the registered PostgreSQL grants test so the gap cannot silently return.

## Baseline and evidence
- Base branch: dev
- Base commit: 632c5e50d5a6352b95c87baa1c26340f52866b31
- Current report: docs/reports/PHASE-9F4J-6-PG-RLS-GRANTS-20261107.md
- Existing report identifies PUBLIC EXECUTE on bi.reject_task_event_mutation() and bi.reject_task_event_quarantine_mutation() as a static migration gap. Target PostgreSQL acceptance is NOT_RUN.

## Owned files
Only edit these five registered paths:
1. db/migrations/2026-11-07-bi-task-event-trigger-exec-revoke.sql
2. crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs
3. docs/briefs/9F4J-6-PG-RLS-GRANTS-20261107.md
4. docs/reports/PHASE-9F4J-6-PG-RLS-GRANTS-20261107.md
5. docs/reports/WBS-DISPATCH-9F4J-6-PG-RLS-GRANTS-20261107.jsonl

## Implementation
- Add an idempotent forward migration that revokes PUBLIC EXECUTE from exactly the two trigger-only functions; preserve function ownership and trigger behavior.
- Extend the PostgreSQL test to explicitly inspect ACL entries with grantee OID 0 for those trigger-only functions and fail with their exact regprocedure identity if PUBLIC EXECUTE remains.
- Keep the existing fixed allowlist, runtime principal, RLS, and rollback assertions intact.
- Do not edit the protected W8 migrations, source, tests, Cargo manifests/lockfile, Group WBS, main, remote refs, or any other worktree.

## Validation and reporting
- Run the focused Rust 1.99.1 package test compile and scoped formatter, plus static migration/test assertions and diff checks.
- The ignored database test must remain NOT_RUN unless a configured dedicated disposable PostgreSQL target is already provided; do not create or start a database or VM.
- Update the phase report honestly, retaining target PostgreSQL, live RLS/grants, and network acceptance as NOT_RUN.
- Preserve the report's seven-section structure and add a revision-history entry.
- Do not push, tag, release, or deploy.

## Acceptance
The forward migration revokes only the two named PUBLIC grants; the SQL test checks the exact ACL boundary; focused compile/format/static checks pass; changes remain within the five registered paths; and all unexecuted database acceptance remains explicit as NOT_RUN.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## Implementation results (2026-11-07)

- Added the forward migration for the two trigger-only functions and a direct PUBLIC ACL catalog assertion in the ignored grants test.
- `cargo +1.99.1 test -p bi-owner-runtime --tests --no-run -j 4`: PASS; the test target compiled. Four `async_fn_in_trait` warnings remain in untouched `task_event_receiver.rs`.
- Scoped Rust 1.99.1 rustfmt, static migration/test ACL assertions, `git diff --check`, and Cypher Graph Guardian for the changed Rust source: PASS.
- Live PostgreSQL/RLS/grants acceptance remains NOT_RUN. No database or VM was created and the ignored test was not run.
