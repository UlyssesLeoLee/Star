# PHASE-STAR-STABLE-W5-RUN-SCOPE-20261101

> Status: 🟢 **PASS — focused Run-scope slice**. W5 integration and the stable release remain open until the other lanes and release gates are complete.

## §0 Purpose

Record the W5 Run Task Card and host-session regression review. The current implementation already enforces the intended Project/Run/Worktree authorization boundary; this lane corrects stale test fixtures and assertions and adds direct missing-token and scoped-403 coverage. No production authorization or request-replay behavior was changed.

## §1 Change matrix

| File | Change | State |
|---|---|---|
| `frontend/src/components/run/RunTaskCardsPanel.test.tsx` | Makes the hostless Run test model an empty server-owned list followed by a successful create and authoritative list refresh; aligns recovery assertions with the current Run/Worktree/binding/authorization message. | PASS |
| `frontend/src/lib/run/runDirectorySession.test.tsx` | Expects `session_expired` for HTTP 401; verifies a missing token blocks before fetch and clears focus; verifies a scoped 403 keeps the same session client and Worktree focus. | PASS |
| `frontend/src/components/run/RunTaskCardsPanel.tsx`, `frontend/src/lib/run/runDirectorySession.tsx`, `frontend/src/lib/run/runDirectoryApi.ts` | Reviewed to confirm authorization, recovery and request behavior; no implementation change was required. | Reviewed |
| Lane brief and dispatch records | Updated with final evidence and the focused result. | Recorded |

The hostless test now proves that a Run with no Repository, Branch or Worktree focus can create a Run-owned card through the current Run endpoint. It does not treat the create response as a durable list projection: the subsequent server list is the asserted source of truth.

## §2 Verification summary

| Check | Result | Evidence |
|---|---|---|
| Focused Run Task Card and host-session tests | **PASS, 24/24** | `rtk proxy npm test -- --run src/components/run/RunTaskCardsPanel.test.tsx src/lib/run/runDirectorySession.test.tsx --reporter=dot`; exit 0. |
| Frontend TypeScript check | **PASS** | `rtk proxy npm run typecheck -- --pretty false`; exit 0. |
| CypherGraph Guardian | **PASS, 2 source files** | `rtk proxy python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp`; exit 0. Updated the test graph for the two new session cases. |
| `git diff --check` | **PASS** | Exit 0 after final report and dispatch records are prepared. |
| Initial focused baseline | **4 failures / 18 passes** | The failure set was one obsolete 401 error-code assertion and three Run Task Card fixtures/message assertions; no runtime authorization regression was found. |

The focused test runner used Node 22.19.0, npm 10.9.3 and Vitest 5.0.1 from the existing `dev` dependency installation through a lane-local ignored junction. No package installation, lockfile edit or dependency change was made. The junction is removed before handoff.

## §3 Known gaps

1. This is only the Run-scope W5 lane. It does not establish that the complete frontend suite passes; the other W5 lanes own separate failures.
2. The hosted Node 24 run and stable release checks remain pending.
3. Run Task Card execution/CLI remains explicitly unavailable until admission, an isolated Runtime sandbox, independent validation and result write-back are implemented. These tests do not claim an executable task-card CLI loop.
4. The lane did not change production code because the reviewed implementation already had the intended behavior; it corrects fixture realism and expectations and adds guard coverage.

## §4 Child-agent failure and takeover list

No child agents were assigned to this lane. There were no child-agent outputs to take over or discard. The coordinator retains ownership of shared WBS changes and integration.

## §5 Guard rules

1. Work stayed in the registered `run-scope` worktree on `agent/w5-vtestfix-1101/run-scope`.
2. Changes are limited to the registered Run Task Card tests, session tests, and lane documentation/dispatch records.
3. The Run remains the owner of Task Cards; optional Repository/Branch metadata and Worktree focus are not authorization prerequisites.
4. Project, Run, Repository and Branch context must match before the Task Card panel becomes available.
5. Missing host session remains closed; no browser token issuer or credential storage is introduced.
6. A missing access token blocks the client before issuing a fetch and clears the selected focus.
7. HTTP 401 invalidates the current client and clears the selected focus; its stable error is `session_expired`.
8. HTTP 403 remains a scoped authorization failure and does not invalidate the otherwise current host session.
9. Task Card writes continue to require the current Run `work-item:write` scope.
10. A focus supplied for create/recovery must still match the original Run/Worktree binding and authorization versions.
11. An uncertain create preserves its original body and idempotency key; a changed target disables retry.
12. A mutation response is not treated as a durable list projection; the panel refreshes the Run-owned list.
13. Authorization failures do not trigger automatic mutation replay; only the explicit, guarded recovery action can resend an uncertain request.
14. Task-card CLI stays disabled while execution admission, sandbox, validation and result write-back are unavailable.
15. The fixtures remain local to tests; no old sample/demo task seed was restored.
16. No package, lockfile, database, API contract, shared WBS, `main`, `dev` or remote branch was changed.
17. No push, release, tag or deployment was performed by this lane.

## §6 Sign-off

| Role | Review |
|---|---|
| Architect | 🟢 Mavis handoff review; test behavior remains aligned with the Project → Run → Worktree authorization boundary. |
| SRE Lead | 🟢 Mavis handoff review; no runtime/infra changes in this slice. |
| Platform | 🟢 Mavis handoff review; frontend TypeScript and focused tests pass. |
| Review moderator | 🟢 Mavis handoff review; scope and diff are limited to the registered lane. |
| PM | 🟢 Mavis handoff review; W5 and stable release remain open. |

## §7 Revision history

| Version | Date | Revision | Trigger |
|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核：记录 Run-scope W5 修复、24/24 focused test、TypeScript 检查及仍未关闭的 W5/稳定发行门禁。 | Registered W5 `run-scope` lane and reproducible frontend test failures. |
