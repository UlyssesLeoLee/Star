# WLD5 Global Hook Route Guard Report

**Status:** The bounded navigation guard is implemented and focused frontend checks pass. WLD-5 remains open.
**Date:** 2026-10-21
**Scope:** Keep unsaved local Hook policy edits from being silently discarded by the audited app-shell routes and global CommandBar. No backend, shared WBS, or execution-policy changes.

## §0 Purpose

Hook policy drafts already warned on document unload and on local scope changes, but app-shell links and CommandBar route actions could leave the editor without asking. This slice registers a guard only while the Hook page has local unsaved edits. A canceled navigation keeps the editor and its changes in place; an approved navigation proceeds and the page cleanup removes the guard.

## §1 Change matrix

| File group | Change | Evidence |
|---|---|---|
| `frontend/src/lib/navigation/UnsavedNavigationGuard.tsx`, `frontend/src/components/navigation/GuardedLink.tsx` | Added a root-provided guard context and a `Link.onNavigate` wrapper using public Next.js App Router API. Guard state is held in a ref, registered only while dirty, and token-cleaned on save/discard/unmount. | Focused component tests cover stay/leave, clean state, unmount cleanup, caller navigation callbacks, external URLs, and new tabs. |
| `frontend/src/app/providers.tsx`, app-shell Link consumers | Mounted the context in `Providers`; routed desktop/mobile shell, Run tree, settings tabs, app matrix, user menu and Project Worktree Index entry through `GuardedLink`. `MobileHeader` is included because it is mounted by RootLayout. Navigation side effects now run only after the route is approved. | Source audit of the named shell components and the root layout confirmed the consumers. |
| `frontend/src/components/CommandBar.tsx` | Gated both search-result and recent-item imperative `router.push` transitions before recording recents or closing the command bar. | CommandBar tests cover canceled/approved search and recent-item routes. |
| `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` and focused tests | Registers while `hasUnsavedChanges`; keeps the existing `beforeunload` listener and local scope confirmation. Save, discard and unmount clear protection. | Hook page integration test exercises route stay, no duplicate scope prompt, browser unload event, discard, save, and cleanup. |
| This report and `WLD5-HOOK-POLICY-UI-SLICE-20261020.md` | Records tested scope and remaining route gaps without marking WLD-5 complete. | The prior report now points to this follow-up and states its exact boundary. |

### Guard behavior

| Navigation type | Behavior |
|---|---|
| Same-origin app-shell `Link` using `GuardedLink` | Prompts only while a Hook draft is dirty. “Stay” prevents the route; “Leave” allows it. |
| CommandBar search result or recent item | Prompts before any recent-item write, panel close, or `router.push`. |
| Clean editor, saved/discarded draft, or unmounted Hook page | No custom prompt; the registered guard is inactive or removed. |
| Same-tab external/document navigation or reload | `GuardedLink` does not intercept it. Existing `beforeunload` protection remains and is browser-dependent. |
| New-tab/modifier navigation | App Router `onNavigate` does not run; the current page stays open. |
| Browser Back/Forward and non-shell page-content navigation | Not intercepted. This slice does not patch browser history or private Next.js router internals. |

## §2 Validation summary

| Check | Result |
|---|---|
| Dependency setup | `rtk pnpm install --frozen-lockfile` completed; lockfiles were not changed. |
| Focused tests | `rtk pnpm exec vitest run 'src/components/navigation/GuardedLink.test.tsx' 'src/components/CommandBar.test.tsx' 'src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx'` passed: 3 files, 30 tests. The first attempt exposed a test-only origin mismatch; the mock now uses the active jsdom origin and models Next.js client navigation. |
| TypeScript | `rtk pnpm typecheck` passed: no errors. |
| Cypher Graph Guardian | `rtk python <skill>/scripts/cgg_guard.py --changed --stamp` passed: 17 source files checked. The 17 files match the 14 modified tracked TS/TSX files plus 3 new source/test files in this lane; documentation files are not counted as source files. |
| Diff whitespace | `rtk git diff --check` passed. |
| Code review | Direct source/diff self-review completed. The parent reports the CodeRabbit shim cannot run because WSL returned `permission denied`; no external review is claimed. |
| CodeGraph | No recognized indexed project/generation was available for this isolated checkout. Exact-source fallback covered the RootLayout shell, named link consumers, CommandBar route calls, and related Hook page. No graph completeness claim is made. |

## §3 Known gaps and remaining WLD-5 gates

1. This does not intercept every internal route. Guarded coverage is limited to the audited app-shell Links and CommandBar routes. Ordinary page-content links and feature-local imperative navigation can still leave the Hook page if rendered in a future persistent/parallel route arrangement.
2. Browser Back/Forward is not safely preventable through the public App Router navigation API available to this implementation. The existing `beforeunload` handler protects document unload where the browser permits it, but it does not protect client-side history traversal.
3. External same-tab navigation relies on the existing browser `beforeunload` prompt; new-tab and modifier links do not leave the current editor. Browser dialog rendering was not runtime-tested.
4. Drafts are still not persisted in browser storage. Save to the server draft before session expiry or leaving through an uncovered route.
5. This UI slice does not establish production auth, target database grants/RLS, Hook producers, native Rust enforcement, or lifecycle-wide Audit/BI linkage. Those WLD-5 gates remain open.

## §4 Subagent failure handoff

- No child agents were dispatched by this bounded worker; no child output remains to hand off.
- The first focused test attempt failed because the test mock treated `localhost` and the jsdom `localhost:<port>` origin as different sites. The test harness was corrected, and the repeated focused suite passed.
- A first diff scan found `MobileHeader` still importing `next/link` despite being mounted by RootLayout; it was included before final validation.

## §5 Slice-specific gates

| Gate | Status |
|---|---|
| Advanced Settings → Hooks remains the configuration surface | Preserved. |
| Guard is inactive while the editor is clean | Implemented and tested. |
| Dirty app-shell links prompt and cancel without route side effects | Implemented and tested through the shared guarded-link component and Hook page. |
| CommandBar search and recent routes prompt before changing panel/recents | Implemented and tested. |
| Existing scope-change confirmation remains single-prompt | Preserved and tested. |
| Save/discard/unmount remove the guard; document `beforeunload` stays active while dirty | Implemented and tested. |
| Browser Back/Forward and all arbitrary internal page links | Not covered; no full global guarantee is claimed. |
| Native Hook decisions and production enforcement | Not covered; remain separate WLD-5 gates. |

## §6 Review and sign-off

| Role | Review |
|---|---|
| Architecture | Worker self-review completed against the assigned brief; coordinator review is required before integration. |
| SRE Lead | Not part of this bounded UI slice. |
| Platform | Not part of this bounded UI slice. |
| Review facilitator | Not part of this bounded UI slice. |
| PM | Not part of this bounded UI slice. |

This report records a bounded UI navigation guard. It does not approve production Hook enforcement or mark the UI plan complete.

## §7 Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added app-shell and CommandBar protection for dirty Hook policy drafts, with exact limitations and focused validation. | `docs/briefs/wld5-global-hook-navigation-20261021.md` |
