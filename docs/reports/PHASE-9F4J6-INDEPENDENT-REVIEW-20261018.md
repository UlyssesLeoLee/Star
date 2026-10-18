# PHASE-9F4J.6 Independent Review

> **Status**: One P2 review finding; no live PostgreSQL result. The persistence test source is useful and mostly aligned, but the accepted connection's owner-role escalation path is not excluded.
> **Date**: 2026-10-19
> **Reviewer**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **Scope**: Independent review of candidate `09dbbd5bc0e347f35c28ee7e19deebe8074a1a9c`; no edits to test source or migration.

## §0 Purpose

Audit whether the gated Rust persistence test calls the production BI Inbox API and whether its executable assertions support the claims in the implementation report. Distinguish source-level/catalog assertions from a live disposable PostgreSQL run and from transport settlement.

## §1 Review matrix and findings

| Area | Result | Evidence |
|---|---|---|
| Production persistence API | PASS at source level | `crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs:117-122` invokes `TaskEventInboxStore::commit_exact_wire`; `crates/bi-owner-runtime/src/task_event_inbox.rs:334-401` calls `bi.commit_task_event_v2` and commits the transaction before returning a receipt. |
| Dedicated database boundary | PASS at source level | `task_event_inbox_postgres_tests.rs:79-108` requires the named URL, loopback host, non-default port, then confirms the exact database name before writes. |
| Exact replay and identity conflict | PASS at source level | `task_event_inbox_postgres_tests.rs:119-154` checks the first commit, exact replay, changed wire conflict, and that the original wire remains replayable with its receipt. |
| Reconnect persistence assertion | PASS at source level | `task_event_inbox_postgres_tests.rs:156-176` closes the first pool, opens a fresh pool, rechecks database identity/role contract, and compares the replay receipt and digests. This only asserts same-database committed-state replay. |
| Runtime grants and direct DML | PARTIAL; P2 below | `task_event_inbox_postgres_tests.rs:190-254` checks effective function and INSERT/UPDATE/DELETE privileges for the current role, but not whether that role can `SET ROLE star_bi_owner`. |
| FORCE RLS | PASS for catalog configuration only | Test lines 203-211 assert `relrowsecurity` and `relforcerowsecurity` on Inbox, fact, and incident. The test has no live execution in this environment and no cross-tenant policy behavior assertion. Migration policies are at `db/migrations/2026-10-16-bi-task-event-inbox.sql:158-180`. |

### P2 — the test does not exclude a caller that can switch to the owner role

`crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs:190-254` verifies runtime-role membership, execute privilege, selected role flags, and effective direct DML privileges for `current_user`. It never checks `pg_has_role(current_user, 'star_bi_owner', 'SET')` (or equivalent owner-role separation). A non-superuser, non-BYPASSRLS login whose effective DML privileges are absent before a role switch can therefore satisfy these checks while retaining permission to `SET ROLE star_bi_owner`.

That distinction matters because `star_bi_owner` owns the tables and the internal v1 persistence function (`db/migrations/2026-10-16-bi-task-event-inbox.sql:154-156, 403-420`). The v2 function restricts normal runtime persistence to an allowlisted projection (`db/migrations/2026-10-16-bi-task-event-projection.sql:211-263, 276-290`), while v1 accepts and writes its supplied payload (`db/migrations/2026-10-16-bi-task-event-inbox.sql:311-340`). A login able to switch to the owner role can bypass the intended runtime-only v2 boundary. The existing SQL guard checks that the runtime role cannot set the owner role (`db/tests/bi-task-event-quarantine.sql:222-225`); the Rust harness does not establish the same property for its actual connection principal.

**Minimal correction**: in `require_runtime_contract`, assert that the connection principal cannot `SET ROLE star_bi_owner` (and assert the owner role itself remains NOLOGIN/non-superuser/non-BYPASSRLS). Keep this test-only; this review lane does not alter source or migrations. Until then, do not claim that the Rust harness fully proves owner/runtime role isolation.

## §2 Validation summary

| Check | Result |
|---|---|
| Static comparison of implementation report with assertions and SQL | Completed; evidence is listed above and below. |
| Dedicated PostgreSQL URL presence | Checked as a boolean only; not configured in this process. No value was read or printed. The ignored PostgreSQL test was not executed. |
| Rust/Cargo test execution by this reviewer | Not run. The implementation report records `cargo test --package bi-owner-runtime --all-targets -j 4`: 12 passed, 1 ignored; that is implementation-lane evidence, not an independent rerun. |
| `git diff --check` | PASS for this review lane's staged manifest, recorded in `docs/reports/WBS-9F4J6-REVIEW-20261018.log`. |
| CodeRabbit | Not run: RTK invocation returned `/bin/bash: /root/.local/bin/coderabbit: Permission denied`; no diff was uploaded. |
| Secrets handling | The dedicated URL was checked only for process-level presence; its contents and all other environment variables were not read. |

The implementation report's source-level claims about loopback/non-default port, exact database identity, receipt replay, conflict, pool close/reconnect, and catalog FORCE RLS checks match the test assertions. The recorded ignored-test build does not establish that PostgreSQL applied the migrations or that these assertions passed against a target database.

## §3 Known gaps

1. **P2**: The test does not reject a current login that can `SET ROLE star_bi_owner`; therefore owner/runtime isolation is not fully established for arbitrary configured test logins.
2. Dedicated PostgreSQL was not available. No migration execution, grants inspection, RLS query, or persistence assertion ran against a database in this review.
3. RLS evidence in the Rust harness is catalog configuration only; cross-tenant policy behavior was not exercised here.
4. The reconnect assertion is not crash, WAL/fsync, backup, or PITR evidence.
5. The persistence receipt is not a transport ACK. No receiver commit-before-ACK ordering or delivery settlement was tested.
6. The implementation report records package-wide formatting and `-D warnings` Clippy blockers in untouched lane-external files; this review did not change them.

## §4 Subagent handoff / failure takeover

This verifier took over only the independent review brief and produced this report plus its JSONL audit log. No worker was dispatched from this lane. Test source, migration, implementation report, and other lanes were left untouched. The P2 finding is handed to the coordinator for disposition before integration.

## §5 Gates applied

| # | Gate | Result |
|---:|---|---|
| 1 | Preserve candidate base commit `09dbbd5bc0e347f35c28ee7e19deebe8074a1a9c` | PASS |
| 2 | Keep changes within the three-file audit manifest | PASS |
| 3 | Do not modify PostgreSQL test source | PASS |
| 4 | Do not modify migrations or grants | PASS |
| 5 | Confirm the test calls the production store method | PASS at source level |
| 6 | Confirm loopback and non-default port gates | PASS at source level |
| 7 | Confirm exact dedicated database identity before writes | PASS at source level |
| 8 | Confirm exact replay and stable receipt assertions | PASS at source level |
| 9 | Confirm changed-wire conflict and original replay assertion | PASS at source level |
| 10 | Confirm first pool closes before fresh-pool replay | PASS at source level |
| 11 | Confirm INSERT/UPDATE/DELETE effective privilege checks | PASS at source level |
| 12 | Confirm Inbox/fact/incident FORCE RLS catalog checks | PASS at source level; no live database |
| 13 | Confirm connection cannot switch to owner role | **FAIL / P2 gap** |
| 14 | Do not claim live PostgreSQL or transport ACK evidence | PASS |
| 15 | Run `git diff --check` through RTK | PASS |
| 16 | Commit only the manifest, without rebase/merge/push/cleanup | PASS |

## §6 Sign-off

| Role | Review |
|---|---|
| Architecture | 架构师 (Mavis 接手 agent per DEC-008) — accepted partial scope; P2 role-isolation finding remains open |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — disposable PostgreSQL not run |
| Platform | 架构师 (Mavis 接手 agent per DEC-008) — source and Rust evidence distinguished from runtime database evidence |
| Review chair | 架构师 (Mavis 接手 agent per DEC-008) — scope and manifest checked |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — no production readiness claim |

## §7 Revision history

| Version | Date | Reviewer | Change / trigger |
|---|---|---|---|
| v0.1 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Independently audited candidate 9F4J.6 source, migration, role grants, and test claims; recorded one P2 owner-role `SET` coverage gap and unavailable live PostgreSQL/CodeRabbit validation. |
