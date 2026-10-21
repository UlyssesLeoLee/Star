# Brief: wld5-global-hook-navigation-20261021

**Agent**: worker
**Phase**: wld5-global-hook-navigation-20261021
**Created**: 2026-10-21 22:13:11

---

## Objective
Protect unsaved Hook policy drafts from supported in-app navigation paths across the main application shell, completing the known global-navigation gap while preserving the existing visual Hook editor and browser-unload warning.

## Evidence and framework contract
- `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` already tracks unsaved drafts, protects selected scope changes locally, and registers `beforeunload`; its test covers these behaviors. The WLD-5 report says global app-router navigation protection is not included.
- Official Next.js documentation for the App Router (Context7 `/vercel/next.js/v16.2.9`; repository uses `next ^16.3.5`) documents `Link.onNavigate` with `preventDefault()` for client-side navigation and suggests shared React context for multiple links. The Pages Router `beforePopState` API is not the App Router API. Do not monkeypatch private router internals or globally patch `history`.
- The CodeGraph coverage-check tool is not exposed in this session. Search targeted app-shell/navigation sources and report the evidence scope; do not claim all navigation paths are covered unless audited.

## File ownership
Own the smallest necessary shared navigation guard, the Hook editor registration, relevant App Router navigation links, focused tests and a task-specific report. Search the existing app layout/sidebar/provider before adding a context. Do not edit the shared WBS or UI plan; the coordinator will reconcile those after integration. Do not overwrite or revert other work.

## Required behavior
1. When Hook policy edits are dirty, supported internal Link-based navigation from the app shell must give the user a clear Stay / Leave choice and must cancel navigation on Stay. Leaving must not silently discard the draft; require explicit discard or a completed save.
2. Keep the existing beforeunload warning for full document exits and keep same-editor query/scope changes on their current save/discard protection path without double prompts.
3. Cover imperative navigation owned by the app shell only through explicit, public, testable helpers; preserve route semantics and query/hash behavior. Document any App Router browser-back/forward limitation that cannot be covered safely with public APIs. Do not use Pages Router APIs in an App Router app.
4. Keep the guard inactive when no dirty Hook draft is registered. Unregister on editor unmount, successful save and explicit discard. Avoid unbounded/global state retention.
5. Preserve the Advanced Settings → Hooks visual surface and capability-neutral product wording; do not add demo/mock Hook state.

## Verification and documents
- Add focused tests for dirty registration/unregistration, cancelled internal navigation, confirmed leave, clean-state navigation, save/discard, and beforeunload. Run the targeted Hook/navigation tests and frontend typecheck if available; report any broader pre-existing failures separately.
- Write `docs/reports/WLD5-GLOBAL-ROUTE-GUARD-20261021.md` with supported route classes, exact test results, and limitations.
- Compare behavior with requirements/basic/detailed design; update a canonical design paragraph only if it materially disagrees. Do not update the shared WBS/UI plan in this lane.
- Inspect your diff, keep it scoped, and commit one logical change. Commit message must reference `docs/briefs/wld5-global-hook-navigation-20261021.md`.

## Product/repo constraints
Follow the current `AGENTS.md`: Rust 1.99.0 baseline; no competitor comparison/imitative product copy; bounded state; accessible prompts/focus; preserve dirty user data; no false completion claim. You are not alone in the repository: other workers have separate file ownership. Do not revert their edits. Use the assigned worktree only.

## Graph/source evidence supplied by coordinator
- Hook editor has a `beforeunload` effect and local draft banner; existing focused tests cover local scope selection and browser unload.
- Validate exact current sidebar/layout Link components and app-router semantics in the assigned worktree before editing.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
