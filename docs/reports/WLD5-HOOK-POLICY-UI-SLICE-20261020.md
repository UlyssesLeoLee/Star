# WLD5 Hook Policy UI Slice Report

**Status:** The bounded UI change is implemented and focused frontend checks pass. WLD-5 remains open.
**Date:** 2026-10-21
**Scope:** Advanced Settings → Hooks draft handling only; no shared design/WBS or backend changes.

## §0 Purpose

Close one usability and data-loss gap in the visual Hook policy editor. Existing authorized typed editing remains in place; this slice makes unsaved local edits visible and prevents ordinary scope changes or refreshes from silently discarding them. Local changes remain separate from the server draft and active policy until explicitly saved and published through the existing authenticated API.

## §1 Change matrix

| File | Change | Evidence |
|---|---|---|
| `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` | Compare the editor document with the loaded typed server policy/draft; show an unsaved-change status and explicit discard action; confirm before Project/Worktree/scope changes or policy refresh; use `beforeunload` protection; disable publish and rollback while local edits are unsaved. | Component uses `HookPolicyDocument` / `HookPolicyResponse` from the existing API client. No new endpoint, token source, local sample policy, or provider was added. |
| `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx` | Add a focused regression case proving canceled scope change preserves edits, browser leave is blocked while dirty, and explicit discard returns to the server-loaded document. | The test uses an authenticated test provider and a typed-shaped API response. |
| `docs/reports/WLD5-HOOK-POLICY-UI-SLICE-20261020.md` | Record implementation, checks, limitations, and remaining WLD-5 gates. | This report. |

## §2 Validation summary

| Check | Result |
|---|---|
| Dependency setup | `rtk pnpm install --frozen-lockfile` completed successfully; lockfiles were not edited. |
| Focused test | `rtk pnpm exec vitest run 'src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx'` passed: 1 file, 17 tests. |
| TypeScript | `rtk pnpm typecheck` passed: no errors. |
| Broad frontend test attempt | `rtk pnpm test -- 'src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx'` ran the unfiltered suite due to the script argument separator. It ended with 15 failed files / 17 failed tests, 1,020 passing tests, and 4 Playwright suites imported by Vitest. The Hook test's first-run failure was an ambiguous `getByRole("status")` assertion; it was corrected, and the focused Hook suite then passed. The other failures were not investigated or attributed to this change. The suite also changed one snapshot during that broad run; that generated change was restored. |
| CodeGraph | The assigned checkout did not have a usable indexed graph, and graph coverage tools were unavailable. No graph-completeness claim is made. |
| Cypher Graph Guardian | `cgg_guard.py --changed --stamp` passed for 2 source files after semantic header reconciliation. |
| Diff whitespace | `rtk git diff --check` passed. |
| CodeRabbit | CLI invocation was blocked: `/bin/bash: line 1: /root/.local/bin/coderabbit: Permission denied`; no remote review was performed. Direct source and diff self-review was completed. |

## §3 Known gaps and remaining WLD-5 gates

1. The production embedding host still has no verified authentication issuer/session callback. The page remains unavailable without the existing authenticated Group API client; authentication expiry continues to clear protected UI state by design.
2. This slice does not persist unsaved edits in browser storage or intercept every App Router navigation. The follow-up in `WLD5-GLOBAL-ROUTE-GUARD-20261021.md` now guards the audited app-shell `Link` routes and CommandBar search/recent `router.push` transitions. Ordinary page-content links/imperative transitions outside that shell and browser Back/Forward are not intercepted. Top-level unload/reload and same-tab external navigation rely on `beforeunload` where supported; new-tab links do not leave the current editor. Save the server draft before leaving the app or before the session expires. No policy content is copied across sessions.
3. No target database grants/RLS behavior, production Hook producer, native Rust enforcement at Run/tool/review/cleanup boundaries, live authorization, or lifecycle-wide Audit/BI linkage was exercised here.
4. Publish and rollback remain server-authorized operations using existing typed endpoints. The UI does not claim that this frontend itself enforces security-critical Hook decisions.
5. WLD-5 stays open in the WBS; this report does not mark the visual policy acceptance matrix or native enforcement phase complete.

## §4 Subagent failure handoff

- No child agents were dispatched by this bounded worker; no child output or failed subagent work remains to hand off.
- The first full-suite invocation was unfiltered and exposed unrelated failing files. Those failures are recorded above and were not changed in this ownership slice.
- The pre-fix ambiguous status assertion was corrected before the focused 17-test pass.

## §5 Slice-specific gates

| Gate | Status |
|---|---|
| Advanced Settings → Hooks remains the configuration surface | Preserved. |
| Policy editor consumes only the authenticated typed Group API response | Preserved; no fallback policy facts were added. |
| Local draft edits are visibly distinguished from persisted draft/active policy | Implemented and covered by a focused test. |
| Scope changes and policy refresh do not silently discard local edits | Implemented with explicit confirmation. |
| Browser leave is guarded while local edits are unsaved | App-shell Link/CommandBar routes use an explicit confirmation; document unload uses `beforeunload`. Browser-specific dialog rendering, app-router Back/Forward, and non-shell page navigation were not runtime-tested or intercepted. |
| Publish/rollback cannot apply while there are unsaved local edits | Disabled in the UI and guarded in handlers. Server authorization remains authoritative. |
| Native Rust Hook decisions, target DB and production auth | Not covered by this slice; remain fail-closed/open. |

## §6 Review and sign-off

| Role | Review |
|---|---|
| Architecture | Worker self-review completed against the brief. Coordinator review is still required before integration. |
| SRE Lead | Not part of this bounded UI slice. |
| Platform | Not part of this bounded UI slice. |
| Review facilitator | Not part of this bounded UI slice. |
| PM | Not part of this bounded UI slice. |

This report records a source/UI slice only. It does not approve production Hook enforcement or enable any execution capability.

## §7 Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Recorded local unsaved-draft protection, focused test/typecheck results, broad-suite limitations, and remaining WLD-5 gates. | `docs/briefs/WLD5-HOOK-POLICY-UI-20261020.md` |
| v0.2 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Clarified the follow-up app-shell Link and CommandBar guard coverage, with page-content navigation and browser Back/Forward explicitly out of scope. | `docs/briefs/wld5-global-hook-navigation-20261021.md` |
