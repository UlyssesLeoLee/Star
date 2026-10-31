# PHASE-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030

> Status: 🟡 **PARTIAL**. The four authenticated HTTP acceptance cases passed against a runner-owned disposable PostgreSQL 18 instance with Rust 1.99.1. Exact baseline Rust 1.99.0 compatibility and all production/full-W16 gates remain open; this does not close W16-A.

## §0 Purpose

Record the bounded test design and evidence for the authenticated device-revocation HTTP route. The test freezes the current repeat behavior as first revoke `204 No Content`, repeated revoke `400 InvalidGrant`, and no duplicate audit fact. The intended runner owns a disposable PostgreSQL 18 loopback cluster, uses a constrained application login for the actual router/repository path, and preserves a machine-readable per-run summary. Production readiness is outside this slice.

## §1 Change matrix

| File | Change | State |
|---|---|---|
| `crates/star-api-rest/tests/device_revocation_http_pg.rs` | Adds four ignored cases for unauthenticated rejection; owner-scoped device and refresh-token revoke plus audit; repeat/concurrent semantics; audit-failure rollback. | Four named cases **PASS** on disposable PostgreSQL 18 with Rust 1.99.1 |
| `scripts/automation/w16a_device_revocation_http_pg_acceptance.py` | Adds a unique PostgreSQL 18 loopback runner, environment target sanitization, explicit migrations/test-only prerequisites, constrained app role, exact test-output gate, run-owned cleanup, and Rust toolchain preflight. | Implemented; selected toolchain is checked before database startup; default remains 1.99.0 |
| `scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py` | Covers external-target sanitization, loopback identity, named test-output matching, command bounds, cleanup safety, Rust preflight, BLOCKED summary output, and bounded subprocess logs. | 15/15 Python unit checks **PASS** |
| `docs/automation-design.md`, `scripts/automation/registry.md` | Register the runner, its status and evidence boundaries. | Updated |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` | Adds `IT-W16-DEVICE-REVOKE-HTTP-PG-01`; preserves full W16, production and deployment gates as open. | Updated to v2.2 |
| `docs/briefs/W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md` | Preserves the original brief and adds the latest execution evidence through `SubagentDispatcher.brief`. | Updated with the 1.99.1 PASS and remaining 1.99.0/production gates |
| `docs/reports/WBS-DISPATCH-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.jsonl` | Dispatcher audit for the brief update. | Original brief event preserved; execution-update event appended by the dispatcher |
| `docs/reports/PHASE-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md` | This phase record. | Updated with the successful rerun and scoped limitations |

## §2 Verification summary

| Check | Result | Evidence |
|---|---|---|
| Focused Python runner tests | **PASS**, 15/15 | `rtk proxy py -3 -m unittest scripts.automation.test_w16a_device_revocation_http_pg_acceptance`; exit 0. Includes external-target sanitization, loopback, cleanup, selected-toolchain preflight, exact case matching, fail-closed BLOCKED summary behavior, and a 2 MiB-per-stream subprocess output test proving 1 MiB on-disk caps. |
| Full runner acceptance | **PASS**, 4/4 named Rust tests | Run `20261031T034802-cda7dbb9b0`: disposable PostgreSQL 18 on `127.0.0.1`; all 10 runner steps and exact four-test output accepted; PostgreSQL stopped and only the run-owned data directory was removed. |
| Rust toolchain used | **PASS**, Rust/Cargo 1.99.1 | Runner summary records `rustc 1.99.1 (58cf8f9c3 2026-10-13)` and `cargo 1.99.1 (5f94df478 2026-08-27)`, selected as `stable` for explicit request `1.99.1`. The workspace baseline remains 1.99.0; exact 1.99.0 compatibility is **NOT_RUN**. |
| Exact Rust acceptance tests | **PASS**, 4/4 | `unauthenticated_revoke_is_rejected`, `authenticated_revoke_is_owner_scoped_and_audited`, `concurrent_revoke_has_one_winner_and_one_audit_fact`, and `audit_failure_rolls_back_device_and_refresh_revocation`; run summary has `acceptance_output_valid=true` and no missing cases. |
| Rust formatting | **PASS** | `rustfmt +stable --edition 2021 --check --config skip_children=true crates/star-api-rest/tests/device_revocation_http_pg.rs`; exit 0. |
| Automation registry check | **PASS with warnings** | `rtk proxy py -3 scripts/automation/registry_check.py` exited 0 with 0 errors and 205 repository-wide warnings; the two new runner files were reported as `[OK]`. |
| `git diff --cached --check` | **PASS** after final document/brief synchronization | Exit 0 for the nine registered files staged in this lane. |
| CodeGraph | **LIMITED** | This worktree has no `.codegraph/` index; repository instructions therefore skip CodeGraph. Relevant route, repository, migration and test files were located by targeted `rg` and source reads. No complete graph coverage is claimed. |
| CypherGraph Guardian | **PASS** | `cgg_guard.py --changed --stamp`; three changed source files checked and their source hashes refreshed. |
| `git diff --check` | **PASS** | Exit 0; a CRLF-to-LF normalization notice appeared for the dispatcher-generated brief. |

The accepted Rust command used `rtk proxy cargo +stable test -j 4 --locked -p star-api-rest --test device_revocation_http_pg -- --ignored --test-threads=1`. The runner was explicitly asked for Rust 1.99.1 and records that both `rustc` and Cargo resolved to 1.99.1. This confirms the HTTP slice on that patch release only; it does not establish the workspace's exact Rust 1.99.0 minimum compatibility. The runner now drains both child streams while retaining only their newest 1 MiB each; a regression test emits 2 MiB to stdout and stderr and verifies both saved files stay within that bound.

Prior `.cache/w16-device-revocation-http-pg/runs/` summaries were retained unchanged. Seven earlier summaries record linker `LNK1104` failures; they are historical attempts and are not represented as this successful run's cause. The new run stopped its own PostgreSQL instance and removed its run-owned database directory. Other PostgreSQL processes observed on the host were created on 2026-10-29, before this lane's run; this lane did not stop or modify them.

## §3 Known gaps

1. The four HTTP cases pass only in a local disposable PostgreSQL fixture; exact Rust 1.99.0 compatibility remains **NOT_RUN**.
2. The local fixture applies the real OAuth base and W16 device migration, but installs test-only `audit_audit_event` and no-op SCD2 prerequisites. It does not verify production/global audit or SCD2 behavior.
3. The test uses a disposable administrative pool for fixture setup and observation and a constrained `LOGIN`/`NOINHERIT` application pool for the HTTP repository calls. This is not evidence of target-environment grants, production RLS, or deployment readiness.
4. No production database, Neon deployment, Vercel deployment, Multipass/K3s environment, backup/restore, multi-instance recovery, end-to-end identity chain, KMS behavior, or performance/p99 was exercised.
5. W16-A requirements outside this one revoke endpoint slice, including subscription/read/write/resolve enforcement, remain **OPEN / PARTIAL / NOT_RUN** as applicable.
6. Run the focused gate on the exact workspace baseline Rust 1.99.0 when its compiler is available; the passing Rust 1.99.1 run does not substitute for that compatibility check.
7. Existing `MockOAuth2Bundles` supplies unrelated OAuth router support state; the device persistence dependency under test is replaced with the real `PgDeviceBindingRepository` over the constrained disposable app login. The helper does not stand in for the revoke route's persistence or serve as production evidence for other OAuth flows.

## §4 Subagent failure and takeover

The assigned worker's original stream disconnected before completion. The parent coordinator resumed the lane, reran the final renamed runner and its 14 unit checks, updated registered documentation, and requested a separate read-only code review. This report preserves the earlier 1.99.0 toolchain failure as history while recording the later successful 1.99.1 acceptance. No historical evidence directory or unrelated PostgreSQL process was cleaned up.

## §5 Guard rules

1. Only the runner-owned disposable PostgreSQL 18 loopback cluster may be used by this acceptance.
2. Inherited PostgreSQL/database-target variables are removed before test execution; no caller-supplied database URL is accepted.
3. The runner creates a new timestamp/UUID run directory and never reuses or overwrites prior evidence.
4. Every data-directory operation must remain strictly inside this runner's `.cache` root.
5. On uncertain PostgreSQL status, retain the run data; remove it only after the runner confirms it is stopped.
6. The runner launches PostgreSQL only on `127.0.0.1` and records no database URL.
7. Apply only the OAuth base and W16 device migration, after clearly identified test-only global audit/SCD2 prerequisites.
8. Seed fixture data only in the owned disposable database; never use fixtures as production evidence.
9. Run only the four explicitly named ignored tests; require all four pass with zero failed/ignored tests before reporting acceptance PASS.
10. Authentication must come from a signed test bearer and the actual `build_oauth_router` path.
11. Revoke persistence must use `PgDeviceBindingRepository` through the constrained app login, not direct table updates.
12. The app login must be non-superuser, non-bypass-RLS, non-inheriting, and unable to read device rows directly outside the owner transaction.
13. The app login may assume only the OAuth credential owner role; verify this from the live disposable database session.
14. First revoke expects 204; repeat expects 400 `InvalidGrant` under the current route contract and must not append another audit fact.
15. Two concurrent revokes expect one 204, one 400, and exactly one device-revocation audit fact.
16. An injected W16 device-audit insert error must prove device and refresh-token changes roll back together; the injected trigger is test-only.
17. Any missing compiler, ignored/unrun test, failed cleanup, or failed prerequisite leaves the phase gate open; no local disposable result establishes production RLS, audit, deployment or complete W16-A readiness.

## §6 Review and approval

| Role | State |
|---|---|
| Architecture | Mavis parent review pending |
| SRE Lead | Mavis parent review pending |
| Platform | Mavis parent review pending |
| Review host | Mavis parent review pending |
| PM | Mavis parent review pending |

## §7 Revision history

| Version | Date | Revised by | Change and trigger |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the authenticated device-revocation HTTP test contract, the initial Rust 1.99.0 resolution failure, runner-unit evidence, and the still-open production/full-W16 gates. |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the first successful 4/4 HTTP/PG acceptance and 14/14 runner-unit checks on Rust 1.99.1; keep exact 1.99.0 compatibility and production/full-W16 gates open. |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Cap runner child logs at 1 MiB per stream, verify the cap with 2 MiB output in 15/15 unit tests, rerun 10/10 runner steps and 4/4 HTTP cases, and retain the baseline/production gates as open. |
