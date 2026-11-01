# Brief: STAR-STABLE-W5-RUN-SCOPE-20261101

**Agent**: worker
**Phase**: STAR-STABLE-W5-RUN-SCOPE-20261101
**Created**: 2026-11-01 09:13:46

---

Task: Repair Run Task Card and invalidated-session test failures without weakening the Run security boundary.
Owner: Codex worker in the registered isolated run-scope lane.
Base: dev@41f84d407e0c245af88ab29938e8dda57c67af94.
Scope: frontend/src/components/run/RunTaskCardsPanel.tsx and test; frontend/src/lib/run/runDirectorySession.tsx and test; frontend/src/lib/run/runDirectoryApi.ts; plus this lane's brief, phase report, dispatcher JSONL, and dispatcher audit log only.
Baseline: Focused tests initially had four failures. The 401 code expectation was stale (session_expired is current); three Run Task Card checks used a stale create/list fixture or old Worktree recovery text.
Outcome: No production authorization behavior needed changing. The hostless test now models an empty Run-owned server list and a later authoritative list refresh. Recovery assertions match the existing fail-closed UI and verify retry stays disabled when Worktree binding or authorization version changes. Session tests verify missing token blocks before fetch and clears focus, HTTP 401 blocks the directory with session_expired, and scoped 403 keeps the current client and focus.
Validation: rtk proxy npm test -- --run src/components/run/RunTaskCardsPanel.test.tsx src/lib/run/runDirectorySession.test.tsx --reporter=dot passed 24/24. rtk proxy npm run typecheck -- --pretty false passed. CypherGraph Guardian checked two changed source files successfully. Full frontend/W5, hosted Node 24, and stable-release gates remain open.
Security boundaries preserved: no token issuer/storage; no automatic mutation replay after authorization failure; uncertain retry continues to require the same idempotency key/body and a matching Run/Worktree/auth target; task CLI remains disabled without sandboxed admission, independent validation and result write-back.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
