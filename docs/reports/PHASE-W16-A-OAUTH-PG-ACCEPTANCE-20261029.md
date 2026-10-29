# PHASE-W16-A-OAUTH-PG-ACCEPTANCE-20261029

**Phase:** W16-A OAuth PostgreSQL transaction acceptance
**Base:** `dev@8704466be335d2a3e9c12ba7a8d7da44e9e6767c`
**Status:** **PARTIAL — the three OAuth repository tests pass against a disposable local PostgreSQL 18.6 cluster. This does not claim W16-A complete.**
**Scope:** The registered OAuth PostgreSQL test, isolated runner and its unit tests, this report, and the dispatcher audit. The existing dispatcher brief is retained unchanged.

## §0 Purpose

Provide a repeatable, fail-closed way to execute the three ignored OAuth authorization-code and refresh-token repository tests against a disposable local PostgreSQL 18 cluster. The runner now selects only `oauth_device_auth_pg`; it does not offer Canvas or workspace suite selection. It runs the exact ignored tests, requires all three named tests and a clean `3 passed; 0 failed; 0 ignored` result, and writes complete command logs plus a JSON summary beneath the invocation's `.cache` directory.

The database is created in a fresh PostgreSQL cluster bound to IPv4 loopback. The runner removes inherited PostgreSQL/database URL variables before setting its own local test URL. It uses the disposable cluster superuser; this checks PostgreSQL transaction behavior and does not establish production role-grant behavior.

## §1 Change matrix

| Registered file | Change | Evidence state |
|---|---|---|
| `crates/star-api-rest/tests/oauth_device_auth_pg.rs` | Authorization-code redemption releases 24 contenders through a barrier, asserts exactly one winner, checks the code is consumed in owner scope, verifies one persisted access/refresh pair, and rejects replay. Refresh consumption and rotation also release 24 contenders and verify one winner, one successor/access pair, and revocation of the original refresh row. | **PASS** in the latest real PostgreSQL run: all three ignored tests passed. |
| `crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs` | Makes `consume_if_active` compare the resolved token snapshot (tenant, client, user, scope, device id, and device generation) in the atomic update, fixing the observed SQL bind-count mismatch and rejecting a stale snapshot. | **PASS** in the real PostgreSQL tests, including a stale-snapshot mismatch assertion. |
| `scripts/automation/w16a_oauth_pg_acceptance.py` | Adds a PostgreSQL 18 disposable-cluster runner; installs the OAuth base and W16 migration after test-only prerequisites; runs only `oauth_device_auth_pg` through `rtk proxy cargo`; streams output to files and parses only a bounded tail; preserves all invocation data and logs. | **PASS**; latest run completed the exact three-test acceptance and stopped its owned database cleanly. |
| `scripts/automation/test_w16a_oauth_pg_acceptance.py` | Covers inherited database-target sanitization, exact test-output proof, RTK proxy plus `-j 4` command construction, bounded log-tail reads, and private-path checks. | 5/5 unit checks pass; they are runner checks, not database acceptance. |
| `docs/briefs/W16-A-OAUTH-PG-ACCEPTANCE-20261029.md` | Existing dispatcher brief retained. | No edits in this lane. |
| `docs/reports/PHASE-W16-A-OAUTH-PG-ACCEPTANCE-20261029.md` | Records the implementation, actual run outcomes, limits, and open acceptance gate. | This report. |
| `docs/reports/WBS-DISPATCH-W16-A-OAUTH-PG-ACCEPTANCE-20261029.jsonl` | Preserves the original brief event under the registered filename; the run result is recorded separately. | Dispatch record retained and extended. |

The runner accepts only `--postgres-bin`, `--cargo`, and `--rtk`; no suite switch exists. The successful invocation ran the current RTK-wrapped, file-backed runner. It constructed `rtk proxy cargo +1.99.0 test -j 4 --locked -p star-api-rest --test oauth_device_auth_pg -- --ignored --test-threads=1`. The successful run and its full logs are preserved under `.cache/w16a-oauth-pg-acceptance/runs/1b4673269a04435daa1fab5c93333133/`.

## §2 Validation summary

| Check | Result |
|---|---|
| `py -3 -m unittest -v scripts.automation.test_w16a_oauth_pg_acceptance` | **PASS, 5/5**, rerun after the OAuth-only, RTK-proxy, bounded-tail, and PostgreSQL transaction-assertion changes. |
| `py -3 -m py_compile scripts\automation\w16a_oauth_pg_acceptance.py scripts\automation\test_w16a_oauth_pg_acceptance.py` | **PASS**, rerun after the final Python changes. |
| `py -3 scripts\automation\w16a_oauth_pg_acceptance.py --help` | **PASS**, rerun; help lists only PostgreSQL-bin, Cargo, and RTK options. |
| CypherGraph Guardian `cgg_guard.py --changed --stamp` | **PASS**, 5 changed source files checked after final code formatting. |
| Local PostgreSQL version check | **PASS**, installed server reports PostgreSQL 18.6. |
| First disposable cluster attempt, run `ddc1cf717259486088b8274e31a86bfc` | Initialization, loopback startup/readiness, test database creation, prerequisites, and OAuth base migration passed. W16 migration exited 3 because the minimal migration sequence lacked the pre-existing `star_engineering_directory_owner` role. No Rust tests ran. The summary is preserved at `.cache/w16a-oauth-pg-acceptance/runs/ddc1cf717259486088b8274e31a86bfc/acceptance-summary.json` (`blocked`, 9 steps). |
| Interrupted disposable cluster attempt, run `a138ae8dbab945679a9b4271ba471c76` | Test-only role and both migrations succeeded. Cargo compilation was interrupted under host-memory pressure before any test result; **0/3 tests completed**. Its logs/data remain under `.cache/w16a-oauth-pg-acceptance/runs/a138ae8dbab945679a9b4271ba471c76/`. The owned cluster was stopped by the guarded helper; unrelated processes were left untouched. |
| Defect-discovery run, `a57cbecbb7664c1399ff874f18c90d5c` | PostgreSQL setup/migrations succeeded; two tests passed and the refresh-consume contender failed because the SQL statement had eight placeholders while the repository call supplied three arguments. This real failure led to changing the method to accept and compare the resolved token snapshot. Full output is preserved in its run directory. |
| Compile-fix run, `c7d89fa142ba469881680eeb0a7f94b1` | PostgreSQL setup/migrations succeeded; test compilation exposed an `E0425` test-fixture reference (`bundles` not in scope). The test was corrected to use the refresh repository; no tests ran in this attempt. Full output is preserved in its run directory. |
| Rust formatting check | **PASS**, Rust 1.99.0 `rustfmt --check --edition 2024` on the three changed Rust files after formatting. |
| Latest real PostgreSQL acceptance, run `1b4673269a04435daa1fab5c93333133` | **PASS**: PostgreSQL 18.6, disposable loopback cluster, both migrations successful, all three named ignored tests passed (`3 passed; 0 failed; 0 ignored; 5 filtered out`), and the owned server stopped with exit code 0. Summary: `.cache/w16a-oauth-pg-acceptance/runs/1b4673269a04435daa1fab5c93333133/acceptance-summary.json`. |

The test prerequisite role is local to the disposable cluster. The runner also supplies minimal audit-table and no-op SCD2-function prerequisites so the OAuth migrations install; those do not prove the production audit or SCD2 implementations. The successful test process compiled the current Rust integration target and ran the exact ignored tests; it is not a workspace-wide Cargo result.

## §3 Known gaps

1. This lane proves the repository transaction slice against disposable PostgreSQL 18.6 only. The tests connect as the disposable cluster superuser, so they do not prove production grants, least-privilege separation, or RLS behavior under a runtime login.
2. The test schema supplies a non-login Directory-role placeholder, an audit table, and a no-op SCD2 function only to install the selected migrations. These fixtures are not production authorization, audit, or SCD2 evidence.
3. No OAuth HTTP endpoint or production authorization flow was exercised. Neon/Vercel, Multipass/K3s, and deployment behavior are outside this lane.
4. The Canvas active-stream revocation test is a separate lane and remains uncompiled/unrun against PostgreSQL at this report revision.
5. W16-A and broader W16 acceptance remain open; this local OAuth transaction result closes only the three named repository tests.

## §4 Subagent failure and takeover

The delegated OAuth worker `/root/w16a_oauth_acceptance` disconnected during tool transport; the coordinator took over final inspection and verification in the registered lane. Four unsuccessful attempts were retained: missing test-only Directory role; a memory-interrupted compile; a real PostgreSQL failure exposing the eight-placeholder/three-argument refresh-consume mismatch; and a Rust test compile error from an out-of-scope fixture variable. After correcting the repository method and test reference, the final PostgreSQL 18.6 run passed all three tests and stopped its owned server cleanly. No unrelated PostgreSQL process was changed. The latest successful evidence is run `1b4673269a04435daa1fab5c93333133`; earlier failures remain preserved rather than overwritten.

## §5 Guardrails

1. Use only a fresh disposable PostgreSQL 18 cluster for this runner.
2. Bind the test server to `127.0.0.1`; never accept a caller-supplied database URL.
3. Strip inherited `PG*`, `DATABASE_URL`, Star database URL, and `SQLX_OFFLINE` settings before creating the private test URL.
4. Do not use Docker Desktop, Multipass, Neon, or another target database for this lane.
5. Apply only the explicit OAuth base and W16 device-auth migrations after test-only prerequisites.
6. Keep the Directory owner placeholder non-login and without superuser, bypass-RLS, database-creation, role-creation, or replication privileges.
7. Treat the audit table and no-op SCD2 function as disposable migration prerequisites only.
8. Keep the documented database-role limitation attached to every result.
9. Run only the three named ignored OAuth tests; do not turn mocks into acceptance evidence.
10. Require every expected test name and a clean summary with three passed, zero failed, and zero ignored.
11. Keep Cargo behind `rtk proxy`, pin Rust 1.99.0, and cap it at four jobs.
12. Stream command output to files and parse only a bounded tail.
13. Stop only the uniquely owned local PostgreSQL cluster; preserve its data and logs for review.
14. Preserve all `.cache` run artifacts and keep them out of the source commit.
15. Do not start another Cargo/PostgreSQL run until the coordinator confirms a safe build window.
16. Mark only this OAuth repository slice accepted after all three named real PostgreSQL tests pass; keep W16-A open until the remaining W16 evidence is complete.

## §6 Signoff

| Role | Review |
|---|---|
| Architecture | Mavis 接手审核 — the OAuth repository slice passes; W16-A remains open. |
| SRE Lead | PostgreSQL 18.6 migrations and all three focused tests passed in the final disposable run; test-only role limitations remain. |
| Platform | Test-only role and audit/SCD2 prerequisites are explicitly limited to the disposable cluster. |
| Review chair | 5/5 Python runner checks, focused PostgreSQL tests, Rust formatting, and CypherGraph Guardian pass; CodeRabbit was not run. |
| Product manager | This local transaction slice does not close W16-A or broader W16 acceptance. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the OAuth-only runner, bounded and RTK-wrapped Cargo execution, PostgreSQL setup outcomes, interrupted test run, preserved evidence, and open acceptance gate. |
| v0.2 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Recorded the final PostgreSQL 18.6 pass (3/3), the refresh-consume SQL defect and fix, the compile correction, current Rust formatting/graph checks, and the remaining production/W16-A limits; triggered by final lane verification. |
