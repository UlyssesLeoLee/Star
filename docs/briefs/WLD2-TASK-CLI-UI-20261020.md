# Brief: WLD2-TASK-CLI-UI-20261020

**Agent**: worker
**Phase**: WLD2-TASK-CLI-UI-20261020
**Created**: 2026-10-21 00:51:52

---

## Objective
Deliver one bounded Task Card CLI UI safety/closure slice for WLD-2.

## Ownership
You own only:
- frontend/src/app/worktree/[id]/group/page.tsx
- directly related Task CLI frontend tests under frontend/src
- docs/reports/WLD2-TASK-CLI-UI-SLICE-20261020.md

Other agents are working in separate worktrees on WLD-1 and Hooks. Do not edit or revert their files. Do not edit the shared WBS, requirements, Basic Design, Group DD, or global navigation.

## Requirements
- Inspect the current CLI start/status/cancel/reattach UI and API types plus docs/reports/WLD2-TASK-CLI-CLOSURE-AUDIT-20261020.md and the WLD-2 row in docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md.
- Keep CLI bound to the authenticated Task, current Run and selected Worktree. Do not create arbitrary Shell, mock provisioner, fake session, or local seed fallback.
- Execution must remain visibly unavailable unless the existing server capability/provisioner returns an actual authorized session and single-use attachment ticket. Improve the UI''s handling of unavailable/error/uncertain state and ensure failure never looks like task completion or a usable terminal.
- Use only existing API contracts; do not invent endpoints or alter backend/auth behavior.
- Add focused tests for the behavior you change.
- Write the report with exact changed files, behavior, validation results, and remaining provisioner/sandbox/DB/auth/runtime/independent-validation gaps. Do not mark WLD-2 complete.
- Run focused tests/typecheck if available. If blocked, report exact command and reason.

## Commit
If you commit, include this brief path in the commit message: docs/briefs/WLD2-TASK-CLI-UI-20261020.md.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
