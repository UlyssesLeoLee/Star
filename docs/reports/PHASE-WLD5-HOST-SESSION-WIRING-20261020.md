# PHASE-WLD5 — Host Session Composition Report

**Status:** Frontend session composition slice implemented and focused checks passed. Production authentication remains unavailable until the embedding host supplies a real session.
**Baseline:** assigned `dev` baseline `944b29d5`
**Owner:** WLD5 host-session lane
**Date:** 2026-10-20

## §0 Purpose

Compose the host-provided `RunDirectoryHostSession` into the Group API provider so Run and Group calls use the same injected token callback and non-secret generation. A missing token or Group `401` must invalidate the Group client and discard its UI scope. A `403` remains scoped to the denied operation. The change does not install an authentication source.

## §1 Change and acceptance matrix

| Requirement | Implementation | Evidence |
|---|---|---|
| Share host token callback and session generation | Root `Providers` supplies `getAccessToken` and `generation` to `WorktreeGroupApiProvider` only when a host session exists | `frontend/src/app/providers.tsx`; `providers.test.tsx` |
| Keep service origins independent | Group keeps its existing default `/api/v1` route and does not inherit Run `baseUrl` or trusted origins | Provider composition test asserts the Group path; no Group base URL is copied |
| Resolve tokens per request | `WorktreeGroupApiClient.request` continues to call the injected callback for every request | Existing fresh-token test and provider composition test |
| Invalidate missing-token and HTTP 401 sessions | The Group client invokes its authorization invalidation callback once; the provider clears the client exposed to consumers | `worktreeGroupApi.test.ts`; `groupProjection.tsx` |
| Reject use of an invalidated client | Future calls on the old Group client fail locally with `session_invalidated` and do not issue another fetch | `worktreeGroupApi.test.ts` |
| Keep 403 scoped | Authorization invalidation only handles status 401; a 403 remains a request error | `worktreeGroupApi.test.ts` |
| Clear stale UI state | The Hooks page receives no Group client after invalidation; the projection distinguishes invalidated session from absent provider and returns an error instead of preview | `HookPolicyPage.test.tsx`; `groupProjection.test.tsx` |
| Replace clients on generation change | Group client memoization includes the non-secret generation; replacement reloads through the new client | `providers.test.tsx`; `groupProjection.test.tsx` |

## §2 Validation summary

| Check | Result |
|---|---|
| Focused Vitest: `providers.test.tsx`, `HookPolicyPage.test.tsx`, `worktreeGroupApi.test.ts`, `groupProjection.test.tsx` | PASS — 4 files, 49 tests |
| Frontend TypeScript: `rtk npm run typecheck` | PASS — `tsc --noEmit` |
| Cypher graph freshness and relationships: `cgg_guard.py --changed --stamp` | PASS — 6 source files |
| `rtk proxy git diff --check` | PASS |
| `npm ci` | Completed to provision this isolated worktree; 329 packages installed. npm reported 5 dependency advisories and engine warnings because the host Node is 22.19.0 while some locked packages request 22.22.2+. It rewrote `frontend/public/mockServiceWorker.js`; that install-generated change was restored and is absent from the final diff. This is environment evidence, not a change to dependency versions. |

## §3 Known gaps

1. The application embedding host still supplies no concrete authenticated session/token issuer. Without it, the root Group client remains unavailable and protected features stay fail-closed.
2. No production authentication, database, or live runtime acceptance was performed in this frontend slice.
3. Group API base URL/routing is not proven to be identical to Run Directory routing. The provider therefore forwards neither Run `baseUrl` nor `trustedOrigins`.
4. A 401 or missing token invalidates this Group client. A 403 does not log out the user or invalidate the session.
5. `npm ci` surfaced 5 dependency advisories (2 moderate, 2 high, 1 critical) and Node engine warnings. This task did not change or audit the dependency graph.

## §4 Subagent failure handoff

No child agents were dispatched from this implementation lane. No subagent output was relied upon. The implementation was reviewed against the assigned WLD5 brief and the source contracts it names.

## §5 Acceptance gates

1. The embedding host is the only source of the token callback.
2. The frontend does not create, persist, or infer credentials.
3. Session generation is non-secret and changes when the host session changes.
4. Group resolves the token callback for each API request.
5. Missing token fails closed before fetch.
6. Missing token invalidates the Group client.
7. HTTP 401 invalidates the Group client once.
8. HTTP 403 remains request-scoped and does not invalidate the session.
9. Session replacement creates a distinct Group client.
10. Group does not inherit an unverified Run API origin.
11. No-provider and invalidated-session states are distinguishable.
12. Invalidated Group projection returns an error state rather than preview data.
13. Hooks UI stops rendering old policy and execution-event state after client invalidation.
14. Protected requests retain `credentials: omit` and `cache: no-store`.
15. Focused tests cover composition, generation replacement, 401, missing token, 403, and Hooks state clearing.
16. Reuse of an invalidated Group client fails locally without another fetch.
17. Typecheck, diff whitespace checks, and Cypher header checks pass before commit.

## §6 Review and sign-off

| Role | Review |
|---|---|
| Architecture | Mavis 接手审核 |
| SRE Lead | Mavis 接手审核 |
| Platform | Mavis 接手审核 |
| Review host | Mavis 接手审核 |
| PM | Mavis 接手审核 |

The sign-off covers only this frontend composition slice. It does not certify the absent embedding-host authentication implementation or production runtime.

## §7 Revision history

| Version | Date | Editor | Change |
|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record shared host-session composition, Group invalidation behavior, focused checks, and remaining production gaps. |
