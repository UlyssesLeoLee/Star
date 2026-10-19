# WLD5 Authentication Source Review

**Status:** No production root authentication issuer or session callback is wired into the application. Hooks and protected Group operations remain fail-closed.
**Audit base:** `2a27ced47f4b3be2d680f9e8e573e9c54bec01f1`
**Date:** 2026-10-20

## §0 Purpose

Trace the shipped application root, `RunDirectoryHostSession`, Group API client and proxy, and Advanced Settings → Hooks page to determine whether a real trusted authentication source can be wired in this phase. This is a bounded source review. No credential, application source, session issuer, or WBS status was changed.

## §1 Evidence and findings matrix

| Concern | Current source evidence | Finding |
|---|---|---|
| Actual app-root call site | `frontend/src/app/layout.tsx:74-76` renders `<Providers>` without a session prop. Focused `rg` over `frontend/src/app` found the only non-test root call at this location; other matches are the provider definition and test fixtures. | The shipped root supplies no session object. |
| Provider composition | `frontend/src/app/providers.tsx:28` defaults `runDirectorySession` to `null`; lines 47-56 pass it to `RunDirectoryProvider` and mount `WorktreeGroupApiProvider` only when it is non-null. | The Group API composition exists, but is conditional on an absent root input. |
| Session contract and absence behavior | `frontend/src/lib/run/runDirectorySession.tsx:32-39` says a real host supplies the contract and that no browser token issuer is installed; it defines non-secret `generation` and `getAccessToken`. Lines 65-68 return no client when callback or generation is missing. | This is an injection contract, not an issuer or login/session provider. |
| Group client auth boundary | `frontend/src/lib/group/groupProjection.tsx:64-92` requires `getAccessToken` and a non-secret `sessionKey`; `frontend/src/lib/group/worktreeGroupApi.ts:766-809` invokes the callback per request, sends Bearer authorization with `credentials: "omit"`, and invalidates its client on 401. | A real callback and generation must be supplied by a trusted host. Tests or a cookie alone do not satisfy this callback contract. |
| Hooks page behavior | `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx:109-110` obtains the Group client from context; lines 463-470 render an authentication-required state when it is missing and state that policy samples and writes are disabled. | Hooks correctly stays fail-closed without root session composition. |
| Server-side Group BFF | `frontend/src/app/api/group/[...path]/route.ts:117-123` reads a configured/default cookie on the server and returns 401 if absent; lines 172-177 build the upstream Bearer header from that cookie. The design documents specify that the host-issued cookie must be HttpOnly. A repository literal search found the cookie name/config reader in this route and no cookie setter in `frontend/src`. | A BFF reader exists, but it does not issue a cookie or provide a browser token callback. Deployment-side issuance remains unproven. |
| Documented separation | `docs/design/DD-WORKTREE-GROUP-001.md:94-98` documents the per-request host callback for `WorktreeGroupApiClient`; lines 118-120 document the distinct cookie-backed BFF for the compatibility Project Index. `docs/basic-design.md:4947-4949` says the BFF issuer/config are not yet verified and do not connect Group Shell/Run Apps. | The two auth paths are intentionally distinct; the BFF must not be treated as proof that the Group App root is authenticated. |
| Test-only session injection | `frontend/src/app/providers.test.tsx:63-95` constructs a callback and session in test code, tests replacement on generation change, and verifies the no-session state. `HookPolicyPage.test.tsx:40-48` verifies no-provider fail-closed rendering. | These tests cover composition and denial behavior, not a production issuer or root wiring. |

Focused source searches covered `frontend/src` for `RunDirectoryHostSession`, root `Providers` call sites, `runDirectorySession`, token/session provider identifiers, cookie configuration/name, and cookie setters. They found no production session factory or cookie writer in the searched frontend source. This does not establish that an external embedding host has no issuer; its identity and integration contract are not present in this repository evidence.

## §2 Validation summary

| Check | Result |
|---|---|
| CodeGraph | The current isolated worktree did not match an indexed graph project. The `check_index_coverage` tool is unavailable in this session. No graph-completeness claim is made. |
| Source trace | Focused `rtk proxy rg` searches plus bounded line-numbered reads of the exact files listed above. |
| Tests/typecheck | Not run: this audit changed no application source and the brief requires focused frontend tests/typecheck only if frontend code changes. Existing test source was inspected as evidence, not executed. |
| `rtk proxy git diff --cached --check` | Passed for the staged report. |
| CypherGraph Guardian | Not required: no source file changed. |

## §3 Known gaps

1. No concrete trusted issuer, login callback, token refresh owner, or root session factory is supplied by the repository frontend. The external host owner must identify and expose the real source before this can be wired.
2. The existing `RunDirectoryHostSession` contract can carry a trusted callback and non-secret generation, but the actual application root does not receive it.
3. The cookie-backed BFF reads a host-issued HttpOnly cookie, but its issuer and deployment configuration are not present in the reviewed frontend. It is a separate compatibility path and does not satisfy the Group client callback contract.
4. Do not create a development token, synthetic issuer, browser cookie reader, or permissive fallback to close this gap. Keep policy reads and writes disabled until real host identity, authorization, and expiry behavior are verifiable.
5. No WBS item was moved to complete; this review records an authentication-source blocker only.

## §4 Subagent failure handoff

No child agents were dispatched from this bounded review. No child-agent output was used. The parent-supplied graph notes were treated as provisional orientation; current source conclusions rely on the exact reads and searches recorded in §1 and §2.

## §5 Acceptance gates and minimal dependency contract

The authentication owner must provide the following before production wiring is accepted:

1. A named, trusted host authentication owner and an actual login/refresh/logout lifecycle. The source must return the authenticated user’s current access token through an asynchronous callback; the callback must not expose refresh credentials to the browser application.
2. A non-secret `generation` value that changes on login, logout, principal, tenant, or authority changes. The real session must be passed at the client host boundary into the root `Providers`, then shared by `RunDirectoryProvider` and `WorktreeGroupApiProvider`.
3. An explicit API-origin and transport contract compatible with existing HTTPS/trusted-origin validation, plus an authoritative principal/permission model. The server must still reauthorize every Project, Run, and Worktree operation.
4. A deliberate boundary for the cookie-backed Project Index BFF: if retained, its trusted host issuer must set the HttpOnly cookie server-side, and deployment must prove cookie scope, Secure/SameSite policy, rotation, logout/revocation, and target API configuration. It must remain distinct from the Group client’s callback flow unless a reviewed adapter explicitly unifies them.
5. Focused tests using an injected real-session-shaped fixture for generation replacement, missing/expired token, HTTP 401 invalidation, scoped 403 denial, no stale projection after identity change, and no preview/seed fallback. Then run frontend typecheck and the relevant tests, and perform a runtime check against the configured host/API.
6. Preserve fail-closed behavior while any owner, issuer, authorization, or target-runtime fact is missing. Do not claim a production Hooks/Group auth loop from provider-only tests.

## §6 Review and sign-off

| Role | Review |
|---|---|
| Architecture | Mavis 接手审核：source-only finding accepted within the bounded scope; external host ownership remains unknown. |
| SRE Lead | Not part of this source-only review. |
| Platform | Not part of this source-only review. |
| Review facilitator | Not part of this source-only review. |
| PM | Not part of this source-only review. |

This review approves only the reported source finding. It does not approve a production authentication integration or enable protected UI operations.

## §7 Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added exact root/session/Group/BFF/Hook source evidence and the prerequisite host-auth contract; left product code and WBS status unchanged. | `docs/briefs/WLD5-AUTH-SOURCE-REVIEW-20261020.md` |
