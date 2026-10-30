# Brief: W16-A-CANVAS-PG-STARTUP-CANCEL-FIX-20261030

**Agent**: reviewer
**Phase**: W16-A Canvas PostgreSQL runner startup cancellation follow-up review
**Created**: 2026-10-30 07:20:38

---

Independently review the cancellation edge found in the startup-pipe fix. The runner now uses `capture_output=False` only for detached PostgreSQL startup, redirects stdout/stderr to DEVNULL, and on any `BaseException` from `process.wait()` best-effort kills and waits for the direct child before re-raising. A new regression test injects `KeyboardInterrupt` and verifies kill plus a second wait.

Review only the latest runner and Python test diff in this candidate worktree. Focus on preserving the original interruption, bounded cleanup, readiness/fail-closed behavior, whether the test really exercises the exception branch, and CypherGraph header coverage. The coordinator reports 25/25 focused Python tests, py_compile, Guardian, and diff check passing after this patch.

Read-only. Do not edit/commit; do not run Cargo, PostgreSQL, or inspect run data. Report findings, remaining limits, and whether the coordinator can perform the controlled PostgreSQL rerun. Do not inspect or clean preserved failed runs.

## Review result

- The reviewer confirmed that any `BaseException` from `process.wait()` triggers best-effort direct-child kill, bounded wait, and re-raises the original exception; the new test exercises `KeyboardInterrupt`, checks kill and the second wait, and confirms DEVNULL routing.
- No high-severity blocker remains for the controlled rerun. Residual limits: no Windows process-tree/job-object termination guarantee; the outer cleanup attempts `pg_ctl stop` and preserves the data directory unless shutdown is confirmed; a second Ctrl+C during the timeout wait may interrupt after kill was issued.
- Review was read-only. No tests, Cargo, PostgreSQL, or run-data inspection were performed by the reviewer.

## Coordinator acceptance follow-up

The controlled rerun passed in run `49b512fbfe334f0e98e46fe7eae7ea39` on PostgreSQL 18.6. All 41 recorded steps exited 0; the exact Canvas device-revocation case passed 1/1 with 0 failed and 0 ignored. PostgreSQL stopped cleanly and the run-owned data directory was removed after shutdown confirmation. This verifies the bounded cancellation/startup fix in this accepted run, but not Windows process-tree termination, production grants/RLS, multi-instance recovery, production p99, or W16-A closure.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
