# Brief: WLD2-TASK-CLI-CLOSURE-AUDIT-20261020

**Agent**: codex-worker
**Phase**: WLD2-TASK-CLI-AUDIT
**Created**: 2026-10-20 08:14:49

---

## Target
Read-only, bounded audit of WLD-2 Task Card and task-bound CLI execution closure against the current WBS and canonical Run/Worktree design.

## Graph evidence and limits
Parent CodeGraph exploration found a bounded RunDirectory session/client path and PreparedTaskCliExecution/TaskPtySession support with bounded output/input, but did not establish a Run-bound production executor. Another graph query surfaced legacy REST worktree and merge-request routes that use in-memory services/default actors and fabricate missing identifiers. Treat these as separate legacy routes, not proof of Run task-card behavior. No check_index_coverage tool is available in this session; read exact source and migrations directly before reporting.

## Scope and questions
Inspect Run-scoped task-card API/UI, TaskExecutionRun persistence/owner boundaries, task-card-to-Worktree binding, CLI session preparation and PTY execution, OS sandbox, authorization and revocation, cancellation/termination/recovery, independent validation/evidence/result write-back, audit/outbox, DB grants/RLS, and whether any UI presents a mock/disabled state as usable. Establish the actual call path from task card to command spawn; distinguish low-level PTY primitives from authorized production orchestration.

## Change
Add one seven-section report only at docs/reports/WLD2-TASK-CLI-CLOSURE-AUDIT-20261020.md. Do not edit Rust, SQL, frontend, design documents, or WBS. Do not add or enable arbitrary shell fallback, synthetic identity, seed/mock production provider, or permissive execution. Keep execution fail-closed where gates are missing.

## Acceptance
Include exact file and line evidence; state CodeGraph coverage tooling is unavailable and identify direct source fallback. Run git diff --check only. Commit the report and reference docs/briefs/WLD2-TASK-CLI-CLOSURE-AUDIT-20261020.md in the message. Report remaining dependencies and the next minimal implementation slice.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
