# WLD-5 Hooks Session Integration Audit

Date: 2026-10-20
Branch: `codex/wld5-hook-session-audit-20261020`
Source base: `1136a24c709e944a538d28a2feff7e72b2b29f4c` (matches the brief's dev base)

## 0. Scope and evidence

This is a source audit only. No application source, credentials, or tests were changed/run. The brief was read from the integration-candidate worktree because it was absent in this audit worktree; the requested report path was also absent and is created here.

The coordinator's CodeGraph index/generation coverage API was unavailable in this worktree. I used focused `rg` searches and inspected the cited files directly.

## 1. Current route and authentication source

- `frontend/src/app/layout.tsx:74-76` renders `<Providers>` without a session prop. `frontend/src/app/providers.tsx:16,32-36` defaults `runDirectorySession` to `null` and mounts only `RunDirectoryProvider`; it does not mount `WorktreeGroupApiProvider`.
- `frontend/src/app/(app)/settings/advanced/layout.tsx:29` keeps Hooks as an Advanced Settings sibling tab. `frontend/src/app/(app)/settings/advanced/[tab]/page.tsx:41` mounts `HookPolicyPage` at the Hooks route. `HookPolicyPage.tsx:109-110` reads `useWorktreeGroupApi()`; without a provider it renders the fail-closed authentication-required state at lines 465-470.
- `frontend/src/lib/run/runDirectorySession.tsx:32-39` defines a host-supplied `generation` and `getAccessToken` contract, and explicitly says no browser credential storage or token issuer is installed here. `Providers` accepts this contract, but RootLayout does not supply it.
- The only repository reference found for `star_access_token` is the server-side reader in `frontend/src/app/api/group/[...path]/route.ts:117-123`; the route injects it as an upstream Bearer token at lines 172-177. Repository search found no cookie setter/token issuer or client auth-session provider. This is a cookie-backed server proxy, not a client token callback or session generation source.
- `WorktreeGroupApiClient` obtains a Bearer token from `getAccessToken` for each request and sets `credentials: "omit"` (`frontend/src/lib/group/worktreeGroupApi.ts:766-791`). Do not read the server cookie from browser code.

## 2. Comparison with Run-directory session handling

`RunDirectoryHostSession` is the closest existing contract. `RunDirectoryProvider` creates a client for the supplied generation, disposes the old client and clears Run focus on generation replacement, and invalidates on missing token or HTTP 401 (`runDirectorySession.tsx:56-108`; `runDirectoryApi.ts:590-612`). HTTP 403 remains a scoped permission denial (`runDirectoryApi.ts:605-612`).

`WorktreeGroupApiProvider` requires `getAccessToken` and a non-empty `sessionKey`, and recreates its client when either changes (`frontend/src/lib/group/groupProjection.tsx:49-70`). It has no authorization-invalidation callback. Its client throws on missing token/API errors but does not retire the client on 401 (`worktreeGroupApi.ts:766-791`).

Existing tests provide partial coverage:

- `runDirectorySession.test.tsx:124-185`: missing session, 401 invalidation, generation replacement/disposal; API tests distinguish 403.
- `groupProjection.test.tsx:29-80,139-150`: new token after session-key change and missing-token fail closed.
- `HookPolicyPage.test.tsx:25-32,267-285`: no-provider state and deep-link revalidation after a test generation change. Test tokens do not prove root application auth wiring.

## 3. Minimal safe wiring and expiry/revocation behavior

Use one real host session contract for both Run and Group clients. At the client provider boundary, pass its live `getAccessToken` callback to `WorktreeGroupApiProvider` and the same non-secret `generation` as `sessionKey`. Mount the Group provider only when the real session exists; preserve the current no-provider/no-data/no-write state when it does not. Do not introduce seeded/demo credentials or expose the server cookie to JavaScript.

There is no concrete in-repository host auth provider to supply to RootLayout, so the actual token source must be provided by the embedding host/auth owner before this wiring can be completed. If deployment auth is cookie-only, keep the JWT in the Group server proxy and adapt the Group client to use that same-origin proxy with cookie credentials; the current Group client instead uses Bearer auth with omitted cookies, so this needs an explicit transport change.

Required lifecycle behavior:

1. Request the current token on each API call. Rotate the non-secret generation on login/logout, actor/tenant switch, or authority/session replacement.
2. Missing token, HTTP 401/session-expired, or explicit session-revoked response invalidates the Group client: cancel/ignore old-generation reads, clear Project/Worktree and policy/draft state, disable save/publish/rollback, and require a new generation.
3. Treat ordinary 403 as an authorization denial, not automatic logout. Refresh the affected authorized directory and clear data no longer in scope; invalidate the principal only on an explicit session-revocation signal.
4. Never replay a failed mutation under a newly selected actor/session. Revalidate scope and current server authorization before a later write.

## 4. Focused tests to add or extend

Test recommendations only; none were run.

- `frontend/src/app/providers.test.tsx` (or current provider test location): no session means no Group request and Hooks remains fail closed; an injected real host-session contract feeds both providers the same callback and generation.
- `frontend/src/lib/group/__tests__/groupProjection.test.tsx` and `worktreeGroupApi.test.ts`: current token requested on every call; missing/expired token sends no fetch; 401 invalidates and clears the client; 403 stays a scoped authorization error; generation replacement suppresses late old-session results.
- `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx`: parameterize `saveDraft`, `publishDraft`, `rollback` across expiry/revocation; assert no follow-up write, stale policy/draft is cleared, and controls remain disabled until replacement-session scope revalidation. Extend the current deep-link replacement test to cover policy/event reads too.

## 5. Remaining Rust-native Hook and database gates

The typed Rust policy API/evaluator and migration sources exist; that does not prove production runtime readiness.

1. **Phase producers/adapter:** `crates/domain-hook/src/lib.rs:96-101` currently declares `BeforeRunAdmission` and `BeforeWorktreeArchiveCleanup`. Requirements call for additional tool, validation, review, and Worktree lifecycle phases (`docs/requirements/SRS-MULTICA-HOOK-001.md:149-153`). The same SRS says no production `TaskCliSessionProvisioner` adapter is installed, so Run-admission capability remains disabled (`SRS-MULTICA-HOOK-001.md:145`). Complete the real readiness/fence integration and fail closed on missing or stale authority/policy.
2. **Target DB/runtime roles:** policy and execution-event migrations declare SCD2/append-only guards and tenant FORCE RLS (`db/migrations/2026-09-30-multica-hook-policy.sql:29-31,75-76,121-122,240-277`; `db/migrations/2026-10-01-multica-hook-execution-event.sql:20,72-89`). The SRS separates source/isolated validation from target deployment and lists target DB/RLS/grants as a production gate (`SRS-MULTICA-HOOK-001.md:491-493`). Verify target migration application, runtime-role grants, cross-tenant isolation, draft expiry, SCD2 restrictions, and append-only audit/event behavior.
3. **Atomic Run admission/evidence:** acceptance requires readiness outside the DB transaction, fresh scope/lifecycle/fence checks, Allow atomically writing Run + HookSet snapshot + Hook ledger + shared-ID RunEvent, and Deny creating no Run (`SRS-MULTICA-HOOK-001.md:513-514`). Detailed design says no real Runtime adapter or target-DB end-to-end acceptance exists; Outbox delivery and production auth remain open (`docs/detailed-design/DD-MULTICA-HOOK-001.md:1575`).
4. **BI/consumer coverage:** keep coverage partial/unknown until applicable producers and RunEvent/outcome joins are verified. Full BI/cohort and Run Detail / Project Quality & Improvement consumers remain open (`DD-MULTICA-HOOK-001.md:1575`); do not infer full coverage from the policy API, event schema, or summary UI.
5. **Worktree lifecycle proof:** verify the real Runtime/cleanup provider and destructive command's in-transaction reauthorization/CAS against target DB/runtime roles. Requirements keep the Domain Command as the final atomic gate and require actor, lease, claim, drain, and fresh-lock checks before archive/binding removal/cleanup (`SRS-MULTICA-HOOK-001.md:149-151`).

## 6. Audit result

Hooks is mounted in the correct Advanced Settings tab and fails closed without its Group API provider. No in-repository client session provider supplies a real token callback or generation, and the Group client lacks the Run-directory 401 invalidation behavior. The smallest safe next step is a real host session bridge shared by both clients, Group-client expiry/revocation invalidation, and the focused tests above. Until the host auth source, runtime adapter, and target DB gates are proven, capability must remain fail-closed; this audit provides no production-readiness claim.

## 7. Revision history

| Version | Date | Editor | Change |
|---|---|---|---|
| v0.1 | 2026-10-20 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Source audit of host session wiring, Hooks mount, expiry/revocation, focused tests, and remaining Rust Hook/DB gates. |
