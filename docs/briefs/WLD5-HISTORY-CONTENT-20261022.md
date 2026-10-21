# Brief: WLD5-HISTORY-CONTENT-20261022

**Agent**: worker
**Phase**: WLD5-HISTORY-20261022
**Created**: 2026-10-22 00:03:14

---

## Objective
Close the remaining WLD-5 Hook unsaved-draft navigation coverage identified in WBS v3.2: browser Back/Forward and arbitrary same-origin page-content links. Existing shell-wide GuardedLink and UserMenu route-approval work is already in dev; do not duplicate or regress it.

## Ownership
Own `frontend/src/lib/navigation/UnsavedNavigationGuard.tsx`, `frontend/src/components/navigation/GuardedLink.tsx`, narrowly scoped shared navigation helpers if needed, and focused tests in their existing navigation/HookPolicyPage test files. Do not edit Hook API/backend, menus/sidebar, WBS/UI plan docs, or unrelated routes. You are not alone in the codebase; do not revert other agent edits. Do not commit; parent will review and commit with the authorized project author.

## Existing source facts
The provider currently stores one active guard and exposes `confirmNavigation()` using `window.confirm`. HookPolicyPage registers the guard while a local Hook policy draft differs from the loaded server document, and `beforeunload` covers document exits. GuardedLink invokes confirm through Next App Router `Link.onNavigate`. The current focused Hook test covers local scope changes, one GuardedLink route, beforeunload, save/discard and unmount; it does not cover history or arbitrary content anchors. Preserve the current prompt and exact Leave/Stay semantics.

## Framework documentation
Parent queried current official Context7 Next.js docs: App Router `Link.onNavigate` can cancel client-side same-origin Link navigation with `preventDefault`; modifier clicks, external links and download links do not call it. The public `useRouter` API provides `push`, `replace`, `back`, `forward`, `refresh`, but no public `beforePopState` or `router.events` hook. Do not depend on private Next.js internals or a global History API patch that can corrupt/erase forward entries. If a proposed history approach needs a capability unsupported by the product's target browser/WebView, provide exact evidence and keep the feature clearly reported as incomplete; do not claim browser Back/Forward coverage from `beforeunload` alone.

## Acceptance
1. A dirty Hook draft prompts before any supported same-origin page-content link leaves; Stay cancels and retains draft/focus/menus/recent-navigation state; Leave navigates once.
2. Browser Back and Forward are guarded if a stable, supportable solution is available without corrupting the history stack; tests must cover Stay and Leave on the actual implemented history mechanism. If not safely possible with public APIs, explicitly report the framework limitation and don't add brittle private API or history mutation.
3. External, modified/new-tab, download, same-document hash-only links and harmless same-route query normalization remain unblocked as appropriate.
4. Existing GuardedLink/UserMenu route behavior and Hook tests remain passing; no double prompts or menu close before approval.
5. Keep `beforeunload`; run focused navigation/Hook tests, `pnpm run typecheck`, `git diff --check`, and changed-file CypherGraph Guardian reconciliation if runtime is available. Report exact commands/results and unresolved limits.

## Graph/source limitations
The parent attempted targeted CodeGraph lookups, but only irrelevant symbols were returned and the graph coverage checker is unavailable in this session; its per-project explore budget is exhausted. Parent directly inspected the current provider, GuardedLink, HookPolicyPage and current Hook test references. You must inspect exact source and existing test harness in this worktree before editing, and make no exhaustive negative claim based solely on graph output.

## Product constraints
Keep the UI language capability-neutral. No API/authorization/backend changes. Leave/Stay must not itself publish or discard a server draft. The Rust-native desktop remains separate; this slice only covers the current Web App Router client.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
