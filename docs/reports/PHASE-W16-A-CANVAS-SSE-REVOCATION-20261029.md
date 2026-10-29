# PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029

> Status: 🟡 Partial. The PostgreSQL acceptance test is implemented and the local safety gate is in place. The OAuth lane provided a shared disposable PostgreSQL 18 runner, which is currently stopped with its run data preserved; this Canvas stream-revocation case remains **NOT_RUN**. This report does not claim W16-A complete.

## §0 Purpose

Record a focused acceptance test for an already-open Worktree Canvas SSE stream after its persisted device binding is revoked. The test uses the production group router, `CanvasEventWakeHub`, PostgreSQL Outbox notification, the W16 device-binding trigger, and the route's authorization recheck. The five-second assertion is a test bound, not a measured production service-level objective.

## §1 Change matrix

| File | Change | State |
|---|---|---|
| `crates/star-api-rest/tests/canvas_device_auth_pg.rs` | Added an ignored PostgreSQL case that seeds persisted Project/Branch/Engineering Run/Worktree/device authorization, confirms a live SSE Canvas wake, then commits device revocation and requires the open stream body to end within five seconds. Added a loopback/local-socket guard before any migration or fixture writes. | Implemented; focused compile **INCOMPLETE**; DB acceptance **NOT_RUN** |
| `docs/briefs/W16-A-CANVAS-SSE-REVOCATION-20261029.md` | Dispatcher brief. | Preserved |
| `docs/reports/WBS-DISPATCH-W16-A-CANVAS-SSE-REVOCATION-20261029.jsonl` | Dispatcher event. Renamed from the helper-created filename after verifying its SHA-256 remained unchanged. | Preserved and registered |
| `docs/reports/PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029.md` | This bounded implementation and evidence report. | Current |

## §2 Verification summary

- Rust 1.99.0 formatting for the changed test module: **PASS**.
- `git diff --check`: **PASS**.
- CypherGraph Guardian `--changed --stamp`: **PASS**, one changed source file.
- MAGOS manifest correction: **PASS**. The registered claim now contains the four present lane artifacts. The absent `scripts/automation/w16a_canvas_sse_pg_acceptance.py` and `scripts/automation/test_w16a_canvas_sse_pg_acceptance.py` paths were removed because this acceptance is a direct Rust integration test and does not need a Python runner or wrapper. The official guard returned `decision=allow`, `code=OK`, `4 entries`, `exit_code=0` for:

  ```powershell
  $repo = 'C:\Users\leo19\.codex\worktrees\w16a-canvas-sse-pg-acceptance-20261029\Star'
  $guard = 'C:\Users\leo19\.codex\plugins\cache\magos\magos\1.0.8\scripts\magos_guard.py'
  py -3 $guard plan --repo $repo --json --lang en --branch 'codex/w16a-canvas-sse-pg-acceptance-20261029' --worktree $repo --paths 'crates/star-api-rest/tests/canvas_device_auth_pg.rs' 'docs/briefs/W16-A-CANVAS-SSE-REVOCATION-20261029.md' 'docs/reports/PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029.md' 'docs/reports/WBS-DISPATCH-W16-A-CANVAS-SSE-REVOCATION-20261029.jsonl' --amend
  ```

  The guard also reported `REMOTE_AS_OF_LAST_FETCH` and `NOT_ARMED`; no claim-store file was edited manually.
- Focused compile: **INCOMPLETE; no pass claimed**. The authorized command `rtk proxy cargo +1.99.0 test -p star-api-rest --test canvas_device_auth_pg --no-run --locked --offline -j 4` began at 3,386 MiB free and emitted dependency warnings before reaching `Compiling star-api-rest`. Free memory then fell to 2,551 MiB, below the 2.5 GiB stop threshold; the command was interrupted and returned exit code 1. No Rust error diagnostic was emitted before interruption, but the target did not finish compiling. The resulting 1.99.0 Cargo/rustc process tree was stopped; no unrelated stable Cargo process was touched.
- CodeRabbit second opinion: **NOT_RUN**. The installed Windows wrapper failed to execute the Ubuntu CLI, reporting `/root/.local/bin/coderabbit: Permission denied`; local source review continues, but this is not a clean CodeRabbit result.
- PostgreSQL stream-revocation acceptance: **NOT_RUN**. The shared disposable local PostgreSQL 18 runner was stopped with its run data preserved before this Canvas SSE case was executed. The case must be run when the shared runner is available; no SSE behavior is claimed verified. The parent lane directed that no Neon, target database, or Multipass inventory be touched.
- The dispatcher JSONL SHA-256 was unchanged by the manifest-filename correction: `B23C89738073D6C26D9C6CF17F89E690C25C1476C54F024C6838E58D81B511C8`.
- The acceptance is invokable against a runner-provisioned disposable local database with:

  ```powershell
  $env:STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL = '<runner-issued local disposable PostgreSQL URL>'
  rtk proxy cargo +1.99.0 test -p star-api-rest --test canvas_device_auth_pg active_canvas_sse_closes_after_device_binding_revocation --locked --offline -j 4 -- --ignored --exact
  ```

  The test rejects non-loopback PostgreSQL endpoints before applying the Canvas Outbox migration or writing fixture rows. It requires the W16 device-auth migration and related Project/Run/Worktree migrations to be present. The focused Cargo test target without `--ignored` can verify collection and run non-DB cases, but will skip this acceptance case.

## §3 Known gaps

1. The live PostgreSQL assertion has not run; stream closure, W16 trigger delivery, and the five-second deadline remain unverified runtime behavior.
2. The test revokes the persisted device binding with a committed SQL update, exercising the installed database trigger and active-stream authorization recheck. It does not exercise an OAuth HTTP revocation endpoint or prove append-only revocation-audit behavior.
3. Unique fixture rows are intended for the disposable local database created for acceptance and remain there until that database is discarded.
4. This is one active-stream/device-revocation lane only. It does not close the broader W16-A acceptance or certify production deployments.

## §4 Subagent failure and takeover

This lane was implemented by the delegated worker `/root/w16a_canvas_sse_acceptance`. The worker did not fail or disconnect, so no takeover or uncollected worker output applies.

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
17. Keep W16-A status partial until the runner executes this test and the remaining W16-A acceptance evidence is reviewed.

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
