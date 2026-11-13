# Brief: W16-A-CANVAS-PG-STARTUP-PIPE-FIX-20261030

**Agent**: reviewer
**Phase**: W16-A Canvas PostgreSQL runner startup-pipe follow-up review
**Created**: 2026-10-30 07:17:06

---

Independently review the narrow follow-up fix after the live runner showed that PostgreSQL's detached startup descendants kept the captured stdout pipe open. The coordinator stopped only the exact run-owned temporary cluster and preserved that failed run's evidence.

Review only the runner and unit-test changes in this candidate worktree. Check that `capture_output=False` sends both child output streams to the null device, that it is used only for PostgreSQL startup, that timeout/exit handling remains fail-closed, and that startup readiness is still proven by the separate psql check. Check the updated CypherGraph headers and test coverage.

Read-only. Do not edit or commit files; do not run Cargo, PostgreSQL, or inspect any prior run data. Report concrete findings with paths/lines, remaining limits, and whether the fix is ready for the coordinator's rerun. This is part of the existing registered W16-A candidate lane; it does not authorize changes to other worktrees, dev, or main.

## Review result

- The reviewer found no high-severity blocker for a controlled rerun. `capture_output=False` routes `pg_ctl` output to DEVNULL; a separate psql readiness/version step still proves the server is reachable.
- Residual limits are direct-child-only termination (no Windows process-tree/job-object guarantee), possible surviving descendants or failed kill, and a second Ctrl+C interrupting the bounded timeout wait after kill is issued. Main cleanup attempts `pg_ctl stop` using the validated data directory and retains data unless shutdown is confirmed.
- Review was read-only. No Cargo, PostgreSQL, or run-data inspection was performed.

## Coordinator acceptance follow-up

The approved rerun passed in run `49b512fbfe334f0e98e46fe7eae7ea39` on PostgreSQL 18.6. All 41 recorded steps exited 0; the exact Canvas device-revocation case passed 1/1 with 0 failed and 0 ignored. PostgreSQL stopped cleanly and the run-owned data directory was removed after shutdown confirmation. The separate readiness/version check remained the server-start proof. This does not prove process-tree termination on Windows, production grants/RLS, multi-instance recovery, production p99, or W16-A closure.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
