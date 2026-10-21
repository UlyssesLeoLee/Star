# Brief: UI0-SCREEN-STATE-20261022

**Agent**: codex
**Phase**: UI0
**Created**: 2026-10-22 03:27:39

---

# UI-0 Screen and State Package

## Target
Using the current local dev baseline at 17b3d06539f0281b8a5f9e7a1576864487b26337, produce the UI-0 screen/state and API-owner package at docs/implementation-plans/UI0-SCREEN-STATE-20261022.md. Inspect the current UI implementation and relevant owner/API contracts, then distinguish implemented behavior, partial behavior, and planned behavior. Do not claim a capability is implemented unless source and acceptance evidence support it.

## Change
Define the screen and state model for the next Worktree-first UI release:
- Project selection and Project → Engineering Run → Worktree → local Branch/ref navigation.
- Project Worktree Index as a cross-Run summary that does not replace the navigation hierarchy.
- No-focus Run overview, Worktree selection, authorized focus, and focus switching.
- Focused Run peer app tabs, including Work Items, Task Cards, Canvas, Workflow/Schedule, BI/Benchmark, and authorized Plugin Apps.
- Task Card detail with an in-card CLI panel that remains bound to Task, Run, Worktree, Session, authorization, and policy.
- Explicit WORKTREE/GLOBAL chat scope and its authorization implications.
- Loading, empty, denied, unknown, stale, conflict, unavailable, cancellation, and reconciliation states.
- Keyboard, screen-reader, focus order, paging, and accessible naming flows.

For each screen/state, map facts and mutations to the owning API/domain, scope and identity required, version/freshness evidence, loading/cancellation behavior, and safe user-facing recovery. Include a compact state transition diagram or table and explicit focus-switch ordering. Record where current implementation differs from target.

## Constraints
Preserve Project → Engineering Run → Worktree → Branch. Only a validated Worktree focus may mount that Run’s apps. GLOBAL chat does not broaden authorization. Use owner APIs as business truth; UI caches are bounded projections. Missing providers, permissions, binding, freshness, durable receipts, sandbox or runtime evidence must stay visibly unavailable or fail-closed. Do not use product-comparison copy or competitor product names. Do not touch source code, AGENTS.md, the shared WBS, or the main UI plan.

## Ownership
Write only docs/implementation-plans/UI0-SCREEN-STATE-20261022.md. This is a documentation-only design artifact. Do not run implementation changes.

## Observable acceptance
The document contains a screen inventory, state dictionary, owner/API map, focus and cancellation sequence, keyboard/screen-reader flows, current-vs-target gap table, and verifiable UI-0 acceptance criteria. Every fact is labeled with its evidence source or as an explicit unknown. No mock/seed or preview is represented as production behavior.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
