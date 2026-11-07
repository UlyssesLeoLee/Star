# Brief: 9F4J-6-PG-RLS-GRANTS-20261107

**Agent**: worker
**Phase**: WBS-9F4J-6
**Created**: 2026-11-07 11:17:22

---

## Objective
Add a separate opt-in PostgreSQL integration test for the BI Task Event runtime privilege boundary, without touching the protected W8 PostgreSQL test or migration.

## File ownership
Only edit the new crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs, this brief, its phase report, and this dispatch JSONL. Do not modify task_event_inbox_postgres_tests.rs, task_event_inbox.rs, any migration, Cargo manifests/lockfile, the Group WBS, or another lane.

## Acceptance
- Use an explicitly named dedicated disposable BI Task Event database and prove database identity before reads that may write.
- Verify the restricted authenticated runtime role can execute exactly this fixed three-function allowlist: bi.commit_task_event_v2(uuid,uuid,uuid,uuid,uuid,varchar,integer,varchar,integer,uuid,uuid,timestamptz,jsonb,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,bytea,bytea), bi.list_task_event_page_v1(uuid,uuid,uuid,timestamptz,uuid,integer), and bi.record_task_event_quarantine_v1(uuid,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,varchar,varchar,uuid,bigint,uuid,bytea,integer,varchar). Any additional effective BI EXECUTE privilege must report the offending function identity.
- Verify the restricted authenticated runtime and login roles have no direct Inbox/fact/incident/quarantine DML and cannot SET ROLE to the owner.
- Verify PUBLIC has no EXECUTE on the three allowlisted functions; owner/runtime roles and the authenticated login have the expected restricted attributes; all four BI tables FORCE RLS; and the commit function carries the expected fixed search_path/timeout settings if its migration defines them.
- Include negative assertions that keep tenant mismatch and transaction rollback fail-closed where safely expressible without changing schema.
- If a missing grant/config is discovered, document it as a gap and leave the overlapping migration/owner function untouched.

## Verification limits
STAR_BI_TASK_EVENT_TEST_DATABASE_URL is currently unset. Compile the ignored integration test with package tests under Rust 1.99.1; do not invent or create a VM/database, do not claim target grants/RLS or runtime Postgres acceptance, and do not modify the existing W8 files. Retain the explicit ignored-test safety boundary.

## Reporting
Update only this registered phase report and dispatch/brief files. Include the required seven report sections, exact test/build results, NOT_RUN target database status, known gaps, and revision history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## Implementation findings (2026-11-07)

- Added the registered ignored integration target. It guards the exact database name, loopback host, and non-default port, then confirms current_database() before catalog checks or transactional probes.
- The runtime adapters call the three allowlisted functions with the signatures above. Their migrations revoke PUBLIC EXECUTE and grant each function to star_bi_runtime.
- Static migration inspection found default PUBLIC EXECUTE on trigger-only bi.reject_task_event_mutation() and bi.reject_task_event_quarantine_mutation(); neither function has an explicit PUBLIC revoke. The runtime allowlist assertion reports any additional effective BI function identity, including these triggers.
- The v2 commit migration pins search_path=pg_catalog and row_security=on; it does not define function-level statement_timeout or lock_timeout. The Rust adapter sets transaction-local values of 10s and 5s respectively.
- Package test compilation passed under Rust 1.99.1. PostgreSQL acceptance remains NOT_RUN because STAR_BI_TASK_EVENT_TEST_DATABASE_URL is unset; no database or VM was created.
