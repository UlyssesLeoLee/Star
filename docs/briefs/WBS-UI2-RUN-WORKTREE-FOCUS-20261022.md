# Brief: WBS-UI2-RUN-WORKTREE-FOCUS-20261022

**Agent**: Codex subagent
**Phase**: WBS-20261022-UI2
**Created**: 2026-10-22 00:47:09

---

# Target
Implement the WBS UI-2 correction in this isolated Worktree.

# Change
When the canonical Run route has no verified Worktree focus, render only a Run summary and a clear Worktree-selection affordance. Do not mount Task Cards, BI, any Run App tab list, or App data requests until a specific Worktree focus has been verified. Keep GLOBAL chat scope available; WORKTREE scope is available only after focus verification. Reuse the existing Project/Run navigation and canonical route contract; do not add fake/seed Worktree rows or widen authorization.

# Constraints and ownership
Own only `frontend/src/components/run/RunWorkspaceShell.tsx`, focused tests for that shell, and `docs/reports/WBS-UI2-RUN-FOCUS-20261022.md`. Do not edit the central WBS, UI plan, backend APIs, unrelated components, or user-owned D:\Star checkout. Preserve loading/error/cancellation behavior, canonical identities, accessibility, and responsive behavior. Do not use competitor product names in UI copy. Read the existing Cypher headers and reconcile them with actual code; CodeGraph index is absent, so do not create one.

# Observable acceptance
Add focused tests proving an unfocused Run shows no app tabs or Task/BI panel, and a verified Worktree focus reveals only the existing permitted behavior. Run the focused Vitest file and frontend typecheck if the local dependencies permit; run `python .agents/skills/cypher-graph-guardian/scripts/cgg_guard.py --changed --stamp` from this Worktree if available. Report exact commands/results and any missing tooling. Do not commit. The report must distinguish this slice from full product closure and state residual gaps. The eventual commit message will cite this brief path.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
