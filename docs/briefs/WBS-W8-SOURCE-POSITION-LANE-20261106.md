# Brief: WBS-W8-SOURCE-POSITION-LANE-20261106

**Agent**: worker
**Phase**: WBS-W8-SOURCE-POSITION-LANE-20261106
**Created**: 2026-11-06 18:13:47

---

Task: Complete the existing W8 source-position claim-fix WBS lane, preserving the recorded scope.
Owner/worktree: W8 worker; use only D:\orcaWork\Star\w8-source-position-claim-fix-20261025.
Observed state: branch UlyssesLeoLee/w8-source-position-claim-fix-20261025 at bd90aecc; seven branch-only commits relative to dev@9945c644. Orca reported no live terminal/agent. Current uncommitted tracked changes are limited to crates/bi-owner-runtime/src/task_event_inbox.rs and crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs. There is also an untracked docs/briefs/WBS-W8-DECODE-FIX-20261026.md; preserve it byte-for-byte and do not commit/edit/delete it.
Allowed changes only to the lane's registered manifest: db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql, crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs, docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md, and crates/bi-owner-runtime/src/task_event_inbox.rs. Read the W8 candidate review and transport remediation for context without modifying those worktrees.
Finish the source-position arbitration behavior and lane evidence against current requirements: preserve first-writer atomic claim, cursor-bearing/cursorless generation isolation, exact replay behavior, tenant/source scoping, and fail-closed transactional semantics. Do not broaden architecture or change NATS/provider identities outside the claim. Run focused tests only if their dependencies/endpoints are available; never fabricate PostgreSQL evidence or modify the user's DB/VM. Commit only scoped finished code/evidence. Do not rebase/merge; coordinator will review, rebase to dev, validate, and integrate. Report commit hashes, files, exact checks, and blockers.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
