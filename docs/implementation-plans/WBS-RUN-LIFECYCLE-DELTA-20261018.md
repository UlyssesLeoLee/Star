# WBS Run Lifecycle Delta — 2026-10-18

> Status: **partial as of 2026-10-19**. This delta records the isolated 9F4J.6 BI owner Inbox persistence test slice and P2 source correction. It does not close Phase 9F4J or enable production Task-event consumption.
> Lane: `codex/wbs-9f4j-inbox-pg`; rebased onto local dev `59fbdacbb106898fded81f5d0455b53d43dbaff3` on 2026-10-19. Git skipped the original `09dbbd5b` patch as already applied; the private branch initially ended exactly at the checked dev SHA.

## 1. 9F4J.6 scope

Add an opt-in PostgreSQL persistence test for the BI owner Inbox boundary. The test is ignored by default and may write only to a loopback PostgreSQL instance on a non-default port, after confirming the database name is `star_bi_task_event_test`. It requires the explicit `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` variable. The value is never printed by the test or this report.

The reviewed database contract is defined by `db/migrations/2026-10-16-bi-task-event-inbox.sql` and `db/migrations/2026-10-16-bi-task-event-projection.sql`: `bi.commit_task_event_v2` receives the exact 25-argument signature in the test constant, owns the receipt/fact/conflict writes, and is granted to `star_bi_runtime`; the three Transaction tables use FORCE RLS. Static source inspection confirms that the Rust adapter calls this function inside a tenant-scoped transaction and returns a receipt only after `tx.commit()`.

## 2. Acceptance evidence

| Check | Evidence in this lane | Result |
|---|---|---|
| Schema/function contract | `COMMIT_FUNCTION` in `crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs` matches the declaration and grants in the two migrations named above. The opt-in test checks the exact function privilege and required table/role properties at runtime. | Source signature cross-check complete; database assertion not executed. |
| Exact replay | `pg_runtime_commit_is_durable_idempotent_and_conflict_safe` compares outcome, receipt ID, commit timestamp, wire digest, and route-pin digest on immediate replay. | Test compiled; not executed against PostgreSQL. |
| Identity/wire conflict | The test reuses the same tenant/event ID with changed lifecycle bytes, expects `IdentityConflict`, then replays the original and compares the stable receipt. | Test compiled; not executed against PostgreSQL. |
| Commit survives pool close | The test closes the original pool, creates a new pool, rechecks database/runtime identity, and expects the same receipt from an exact replay. This is a new-connection persistence check, not crash/WAL/PITR proof. | Test compiled; not executed against PostgreSQL. |
| Runtime privilege boundary | The test checks restricted login attributes, membership in the locked `star_bi_runtime` role, exact commit-function EXECUTE, FORCE RLS on Inbox/fact/incident, absence of direct table INSERT/UPDATE/DELETE, that authenticated `session_user` cannot `SET ROLE star_bi_owner`, and that the owner role is NOLOGIN with elevated attributes disabled. | P2 source correction independently reviewed; test compiled; not executed against PostgreSQL. |
| Network commit-before-ACK | No transport ACK or production receiver is exercised by this test. | Not covered; remains a separate 9F4J gate. |

## 3. Validation state

- The first check in this follow-up used a stale process PATH and resolved Rust `1.98.1`, so the default MSRV check was rejected. After prepending the already-installed `E:\DevCache\cargo\bin`, Rust `1.99.0` was selected; no toolchain was installed or changed.
- Required focused check with Rust 1.99.0, `cargo check --package bi-owner-runtime --all-targets -j 4`, passed with 0 errors and 4 existing `async_fn_in_trait` warnings in untouched `task_event_receiver.rs`.
- Required focused test with Rust 1.99.0, `cargo test --package bi-owner-runtime --all-targets -j 4`, passed: **12 passed, 0 failed, 1 ignored**. The ignored PostgreSQL test was compiled but not executed.
- Earlier supplemental all-target check/test with `--ignore-rust-version` on Rust 1.98.1 also passed (check: 0 errors, 4 warnings; test: 12 passed, 1 ignored); this is extra evidence only and is not used in place of the verified 1.99.0 run.
- Rust 1.99.0 `rustfmt --check --edition 2021 --config skip_children=true` on both Inbox Rust files passed. No Rust source was changed in this rebase follow-up.
- Independent source/migration review found no unresolved P0/P1. SQLx 0.8.6's locked local source maps `hostaddr` to the same host field returned by `get_host()`, so the loopback check sees that override. Context7 was unavailable during this review.
- Package-wide formatting and Clippy results above are prior lane evidence, not rerun in this follow-up. They report existing differences in untouched `task_event_quarantine.rs:65` and `task_event_receiver.rs:253,265,279,293` respectively.
- A presence-only check found `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` absent. No PostgreSQL test, migration apply, target DB grant/RLS check, rollback test, concurrency/replay run, or network ACK test was run.
- Existing Cypher Graph Guardian evidence covers the unchanged Rust source. The follow-up changed documentation only; the final staged candidate `git diff --cached --check` passed.

## 4. Remaining gates

1. Provide and verify a dedicated disposable `star_bi_task_event_test` database using loopback and a non-default port, apply both BI migrations and runtime grants there, then run only the ignored test with that explicit URL. Do not print the URL.
2. Record the actual PostgreSQL version and observed runtime privilege/RLS results. This local fixture must not be described as target-database deployment evidence.
3. Separately prove the receiver's durable Inbox commit occurs before transport ACK using the actual selected Rust transport adapter; the persistence test does not model a network or ACK.
4. Keep target DB deployment, production login memberships, migration rollout, backup/WAL recovery, consumer recovery, and production readiness open until independently verified.
5. Resolve the pre-existing package format and Clippy findings in their owning workstream; they are not changed by this slice.
6. The owner-role `SET` gap is closed in source only. Do not mark the live privilege assertion complete until the dedicated PostgreSQL test executes successfully.
7. Rust 1.99.0 focused package check/test passed; this does not establish workspace-wide gates.
8. The current disposable database URL is absent; PostgreSQL grants/RLS/rollback/concurrency and transport commit-before-ACK evidence remain open.

## 5. State transition

| Item | Before lane | After lane |
|---|---|---|
| PostgreSQL persistence test source | Inherited draft, path/format not validated | Integrated under the crate test configuration and compiled with Rust 1.99.0 |
| Reconnect durability assertion | Not present | Added; compiles but awaits disposable PostgreSQL execution |
| Runtime contract assertions | Present in ignored test source | Extended to cover authenticated-principal `SET ROLE` denial and locked owner-role attributes; compiled and independently reviewed, not observed against a live test database |
| Production BI Task-event consumer | Disabled/fail-closed | Still disabled/fail-closed |

## 6. Revision history

| Version | Date | Change |
|---|---|---|
| v0.2 | 2026-10-19 | Added the independently reviewed owner-role `SET` denial and role-attribute assertions; recorded all-targets check/test/fmt evidence and preserved the unrun PostgreSQL gate. |
| v0.3 | 2026-10-19 | Recorded the private-lane rebase onto dev `59fbdacb` (duplicate patch skipped), Rust 1.99.0 focused check/test and rustfmt rerun, and absent disposable DB URL; kept live database gates open. |
