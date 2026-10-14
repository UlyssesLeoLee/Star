# Brief: P1-RUN-SESSION-TESTS-20261015

**Agent**: worker
**Phase**: PHASE-1-RUN-SESSION
**Created**: 2026-10-15 00:12:03

---

## Objective
Add focused frontend coverage for the existing Run Directory host-session fail-closed contract, as a bounded Phase 1 quality slice.

## Scope
Own only frontend test files directly covering `RunDirectoryProvider` and `RunDirectoryHostSession` (prefer `frontend/src/lib/run/runDirectorySession.test.tsx` if it matches repository conventions). Cover missing session, valid host session readiness, auth invalidation clearing Run focus, and generation/session replacement disposing the old client without retaining stale focus. Use the existing test framework and stable public behavior.

## Non-goals
Do not invent or stub a production identity provider, store browser credentials, loosen authorization, or claim Phase 1 production auth is complete. Do not edit the production implementation unless a concrete correctness bug is found; report the evidence and stop for coordinator review. Do not edit shared requirements/basic/detailed design or the implementation plan; coordinator owns document synchronization. Do not touch Infrastructure/D20 source files.

## Acceptance
1. Tests exercise only the host-session boundary and prove fail-closed behavior.
2. Targeted frontend tests and typecheck pass if the configured dependencies are available; report exact commands/results and any environment block.
3. `git diff --check` passes.
4. CypherGraph Guardian is reconciled for changed code/test files if required by project skills.
5. Commit references `docs/briefs/P1-RUN-SESSION-TESTS-20261015.md` and report commit SHA, changed paths, and remaining Phase 1 production blockers.

## Context
The current dev plan says Phase 1 has canonical Branch/Engineering Run schema, authorization/read index/context shell, while host session integration, trusted ingest/grants writes, and Run Apps migration remain incomplete. CodeGraph on this worktree returned `RunDirectoryHostSession` and `RunDirectorySessionState` in `frontend/src/lib/run/runDirectorySession.tsx` with no covering tests; existing implementation explicitly fails closed when no host session is supplied. The CodeGraph MCP exposed `codegraph_explore` only; `index_repository` and `check_index_coverage` were unavailable, so treat that graph result as provisional and verify exact test/file scope locally before editing.

Read COMMENTS before acting. This lane is independent of the D20 infrastructure publisher worker; do not revert its edits or change its worktree.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
