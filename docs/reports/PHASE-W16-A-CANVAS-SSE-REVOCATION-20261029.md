# PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029

> Status: 🟡 Partial. The focused Rust 1.99.0 test target compiled and the exact Canvas stream-revocation case passed on a fresh disposable PostgreSQL 18.6 loopback cluster. This closes only this single-case acceptance slice; it does not claim W16-A complete.

## §0 Purpose

Record a focused acceptance test for an already-open Worktree Canvas SSE stream after its persisted device binding is revoked. The test uses the production group router, `CanvasEventWakeHub`, PostgreSQL Outbox notification, the W16 device-binding trigger, and the route's authorization recheck. The five-second assertion is a test bound, not a measured production service-level objective.

## §1 Change matrix

| File | Change | State |
|---|---|---|
| `crates/star-api-rest/tests/canvas_device_auth_pg.rs` | Added an ignored PostgreSQL case that seeds persisted Project/Branch/Engineering Run/Worktree/device authorization, confirms a live SSE Canvas wake, then commits device revocation and requires the open stream body to end within five seconds. Added a loopback/local-socket guard before any migration or fixture writes. | Implemented; focused compile **PASS**; exact PG18 case **PASS** |
| `scripts/automation/w16a_canvas_sse_pg_acceptance.py` and `scripts/automation/test_w16a_canvas_sse_pg_acceptance.py` | Added a bounded, fail-closed disposable PostgreSQL 18 runner and focused runner tests; it verifies the exact prebuilt binary and unsigned build-attestation claims before starting a private loopback cluster. | Implemented; Python tests **25/25 PASS** |
| `docs/briefs/W16-A-CANVAS-SSE-REVOCATION-20261029.md` | Dispatcher brief. | Preserved |
| `docs/reports/WBS-DISPATCH-W16-A-CANVAS-SSE-REVOCATION-20261029.jsonl` | Dispatcher event. Renamed from the helper-created filename after verifying its SHA-256 remained unchanged. | Preserved and registered |
| `docs/briefs/W16-A-CANVAS-PG-RUNNER-20261029.md` and follow-up briefs | Record the runner contract, hardening, startup-pipe fix, and cancellation review. | Updated to include the accepted run and residual limits |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` | Updates the Canvas SSE revocation PostgreSQL case to PASS with exact evidence and remaining boundaries; full E2E tests stay open. | Updated to v2.1 |
| `docs/reports/PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029.md` | This bounded implementation and evidence report. | Current |

## §2 Verification summary

- Rust 1.99.0 formatting for the changed Rust test module: **PASS**.
- Focused binary build: **PASS** at candidate HEAD `d1269a7928c9323f94403f3f1bb7888048c1dbac`, Rust `1.99.0 (b940084d7 2026-09-28)`, exit 0 in 0.46 seconds. The exact command was `rtk proxy cargo +1.99.0 test -p star-api-rest --test canvas_device_auth_pg --no-run --locked --offline -j 1`. Source SHA-256: `66F46A17FAA287E2D350CA6B24D1B00076EADDC0FCF8F11FA8D0E7C311614F37`; binary SHA-256: `EBCE813E71459B0E871913C25728676F4284ED85A245C2B8F9FDA2E51BEA0F6C`.
- Focused Python runner suite: **25/25 PASS**; `py_compile`: **PASS**; CypherGraph Guardian: **PASS**, two changed source files.
- PostgreSQL 18.6 disposable acceptance: **PASS**, run `49b512fbfe334f0e98e46fe7eae7ea39`, all 41 recorded runner steps exited 0. The exact test `active_canvas_sse_closes_after_device_binding_revocation` reported `1 passed; 0 failed; 0 ignored; 3 filtered out; finished in 0.17s`. The database URL was not recorded, the replay-offset migration was not applied, the stop step exited 0 and reported `server stopped`, and the run-owned data directory was absent after confirmed shutdown. The retained acceptance summary is at `C:\\Users\\leo19\\AppData\\Local\\Temp\\star-w16a-canvas-sse-pg-acceptance\\runs\\49b512fbfe334f0e98e46fe7eae7ea39\\acceptance-summary.json`.
- Build attestation is an **unsigned operator assertion** bound to HEAD and recomputed source/binary SHA-256 values; signature verification and CI-proof verification are false. The Cargo command, toolchain, exit status, and elapsed time are not signed provenance.
- The acceptance used the disposable cluster's `postgres` superuser and test-only no-op audit/SCD2 prerequisites. It does not prove runtime `NOBYPASSRLS`, production grants, RLS behavior, or production audit/SCD2 semantics.
- `git diff --check`: **PASS** after final documentation synchronization; the two existing startup review briefs emit only CRLF-to-LF normalization warnings.
- MAGOS manifest: **PASS**, 17 registered paths. The official guard returned `decision=allow`, `code=OK`, `17 entries`, `exit_code=0`; it also reported `REMOTE_AS_OF_LAST_FETCH` and `NOT_ARMED`. No claim-store file was edited manually.
- Historical compile attempt: the earlier `-j 4` attempt fell below the 2.5 GiB memory stop threshold at 2,551 MiB and was interrupted before the target finished. A subsequent isolated `-j 1` compile at the current candidate HEAD passed; the earlier resource interruption is retained as history, not the current gate result.
- CodeRabbit second opinion: **NOT_RUN**. The installed Windows wrapper failed to execute the Ubuntu CLI, reporting `/root/.local/bin/coderabbit: Permission denied`; local source review continues, but this is not a clean CodeRabbit result.
- PostgreSQL stream-revocation acceptance: **PASS** for the one exact test in run `49b512fbfe334f0e98e46fe7eae7ea39`. Earlier runner failures and the startup-pipe defect remain recorded in their historical run evidence and follow-up briefs; they are not acceptance results and were not deleted.
- The dispatcher JSONL SHA-256 was unchanged by the manifest-filename correction: `B23C89738073D6C26D9C6CF17F89E690C25C1476C54F024C6838E58D81B511C8`.
- The acceptance is invokable against a runner-provisioned disposable local database with:

  ```powershell
  $env:STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL = '<runner-issued local disposable PostgreSQL URL>'
  rtk proxy cargo +1.99.0 test -p star-api-rest --test canvas_device_auth_pg active_canvas_sse_closes_after_device_binding_revocation --locked --offline -j 4 -- --ignored --exact
  ```

  The test rejects non-loopback PostgreSQL endpoints before applying the Canvas Outbox migration or writing fixture rows. It requires the W16 device-auth migration and related Project/Run/Worktree migrations to be present. The focused Cargo test target without `--ignored` can verify collection and run non-DB cases, but will skip this acceptance case.

## §3 Known gaps

1. The test revokes a persisted device binding with a committed SQL update, exercising the installed database trigger and active-stream authorization recheck through the production GET SSE route/hub. It does not exercise an OAuth HTTP revocation endpoint or prove append-only revocation-audit behavior.
2. The disposable PostgreSQL pool uses the `postgres` superuser and test-only no-op audit/SCD2 prerequisites. Production runtime-role grants, `NOBYPASSRLS`, behavioral RLS, and production audit/SCD2 are not proven.
3. This case does not apply the replay-offset migration or exercise multi-instance delivery, listener loss/reconnect, client catch-up, or production-like deployment.
4. Five seconds is only the assertion deadline for this test. No production latency distribution or p99/SLO is measured.
5. This is one active-stream/device-revocation case only. OAuth HTTP revocation, other shared read/subscribe/write paths, E2E-COLLAB acceptance, target Neon/Vercel or Multipass/K3s deployment, and the broader W16-A phase remain **OPEN / PARTIAL / NOT_RUN** as applicable.

## §4 Subagent failure and takeover

The source and runner work were completed by their assigned workers, and the startup fixes received independent read-only review. The final evidence review independently checked the accepted run summary; no failed worker or takeover applies.

## §5 Guard rules

1. Run only against a runner-issued disposable local PostgreSQL database.
2. Fail before writes when the server address is not loopback or a local socket.
3. Do not connect this acceptance fixture to Neon, another remote endpoint, or an unapproved database.
4. Do not use Multipass or Docker as an improvised database source for this lane.
5. Keep the database URL out of logs and reports.
6. Require the explicit `STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL` variable; do not silently substitute another URL.
7. Require the current Canvas Outbox wake migration and the W16 device-binding wake trigger.
8. Require persisted Project, Branch, Engineering Run, Run grants, Worktree, Project-to-Worktree binding, Run-to-Worktree binding, and device facts.
9. Keep each fixture's tenant, actor, Project, Run, Worktree, and device identities unique.
10. Use a device-bound JWT whose generation matches the persisted active device row before revocation.
11. Open the actual GET SSE route through `build_group_router` with the process-scoped `CanvasEventWakeHub`.
12. Confirm a committed Worktree-scoped Canvas Outbox wake is delivered before revoking the device.
13. Do not use arbitrary sleeps, polling loops, or in-memory authorization doubles.
14. Commit device revocation before awaiting stream termination so PostgreSQL emits the trigger notification.
15. Require the active SSE body to terminate without another frame within five seconds.
16. Treat a skipped/ignored or unrun PostgreSQL case as NOT_RUN, never as a pass.
17. A passing single case closes only this Canvas SSE acceptance slice. Keep W16-A Partial until the remaining authentication, production-like role/RLS, endpoint, recovery, and environment acceptance evidence is reviewed.

## §6 Review and approval

| Role | State |
|---|---|
| Architecture | Mavis final review in the parent lane |
| SRE Lead | Mavis final review in the parent lane |
| Platform | Mavis final review in the parent lane |
| Review host | Mavis final review in the parent lane |
| PM | Mavis final review in the parent lane |

## §7 Revision history

| Version | Date | Revised by | Change and trigger |
|---|---|---|---|
| v0.1 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added the bounded Canvas SSE device-revocation acceptance lane report and recorded the database-runner limitation; triggered by W16-A Canvas SSE acceptance dispatch. |
| v0.2 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Recorded the official MAGOS amendment removing two absent, unnecessary Python paths and corrected the PostgreSQL runner status; triggered by the manifest audit. |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Recorded the incomplete focused compile stopped at the memory threshold, with PostgreSQL acceptance still NOT_RUN; triggered by the bounded Cargo compile attempt. |
| v0.4 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the successful Rust 1.99.0 focused build and exact PostgreSQL 18.6 revocation case, correct the current runner status, and retain the unsigned-attestation, test-only-role, production and full-phase gates; triggered by accepted run 49b512fbfe334f0e98e46fe7eae7ea39. |
