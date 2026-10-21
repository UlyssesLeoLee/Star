# Brief: UI1-DIRECTORY-STATUS-20261021

**Agent**: worker
**Phase**: UI1-20261021
**Created**: 2026-10-21 23:57:05

---

## Objective
Implement only the UI-1 Project → Engineering Run → Worktree → Branch directory-row status slice from WBS `WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` and the existing `UI-PLAN-RUN-WORKTREE-NEXT-20261021.md` in the parent dev tree.

## Ownership
Own `frontend/src/components/run/ProjectRunTree.tsx`, its focused navigation-tree test `frontend/src/components/__tests__/SidebarProjectWorktreeNavigation.test.tsx`, and only the necessary Run Directory translation entries in `frontend/src/lib/i18n/{en,zh-CN,ja,dictionary}.ts`. You are not alone in this codebase; do not revert or overwrite other agent work. Do not edit WBS/UI plan docs, APIs, backend contracts, unrelated tests, or generated files. Do not commit; parent performs final review and commit.

## Required behavior
Use only fields already returned by Run Directory `RunWorktree`: human_state, machine_state, agent_id, agent_session_id, runtime_id, archived, and binding versions. Present human and machine lifecycle states distinctly, with safe friendly labels for known states and a bounded raw/unknown fallback. Present agent assignment and session binding as separate facts; never expose raw UUIDs and never call a binding "active" without liveness evidence. Keep archive state distinct. Continue to label lock, dirty changes, observation time, and branch freshness as unknown/unavailable because this response does not provide those facts. Do not infer Worktree status from a branch name. Preserve current virtualized rendering, keyboard navigation, lazy paging, cache budgets, auth behavior, and focus semantics.

## Acceptance
Add focused assertions for lifecycle-state distinction, assigned/unassigned agent and session binding, archived state, no UUID display, and unknown lock/dirty/freshness facts. Keep the test deterministic and avoid mock/seed being presented as production facts. Run the focused test and `pnpm run typecheck` in this worktree if dependencies exist; report exact commands/results. Run `git diff --check` and the CypherGraph Guardian check for changed source files if available. Do not claim broader completion or WBS UI-1 acceptance.

## Graph evidence and limitations
Parent queried the existing `.codegraph` index for `ProjectRunTree`, `RunWorktree`, `RunDirectoryTree`, and lifecycle fields. It shows the virtualized tree, bounded paging cache, and the API fields above. The graph tool did not provide a coverage checker in this session; do not make exhaustive negative claims from graph results. Inspect the changed files and test path directly before editing.

## Dependency and product constraints
The current main navigation remains Project → Engineering Run → Worktree → Branch, independent of host attachment. No provider, API, permission, Worktree lock, dirty-state, freshness, or runtime-liveness changes are authorized. The Rust-native desktop remains a separate planned delivery; do not characterize this React UI as the Rust desktop implementation.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
