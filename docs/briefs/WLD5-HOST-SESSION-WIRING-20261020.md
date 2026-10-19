# Brief: WLD5-HOST-SESSION-WIRING-20261020

**Agent**: worker
**Phase**: WLD5 host session composition
**Created**: 2026-10-20 07:16:17

---

Implement the smallest production-safe shared session composition slice from WLD5 without inventing authentication. Current dev baseline is 944b29d5. Source evidence: RunDirectoryHostSession already carries a non-secret generation and live getAccessToken callback in frontend/src/lib/run/runDirectorySession.tsx; frontend/src/app/providers.tsx passes it only to RunDirectoryProvider; WorktreeGroupApiProvider in frontend/src/lib/group/groupProjection.tsx accepts getAccessToken and sessionKey; the Hooks page fails closed when no Group client exists. The WLD5 audit states no concrete host auth provider or token issuer is installed. Wire the same injected host session callback and generation into the Group client only when supplied; preserve missing-session fail-closed behavior. Add Group client invalidation on HTTP 401 or missing token, clear/invalidate the provider client so existing Hook page effects discard old policy and scope, and keep ordinary 403 scoped rather than logging out. Never add demo credentials, browser credential storage, or client access to a server cookie. Do not assume Run and Group API base URLs are identical unless existing routing contracts prove it. Add focused tests for provider composition, per-call token use, 401 invalidation, 403 behavior, session-generation replacement, and Hook state clearing using the already existing test files where possible. Add a seven-section PHASE report at docs/reports/PHASE-WLD5-HOST-SESSION-WIRING-20261020.md that explicitly says root runtime remains unavailable until the embedding host supplies a real session. Keep execution and production readiness fail-closed. Own only the files in the registered WLD5 manifest; if more files are required, stop and report the missing ownership before editing. Follow Cypher Graph Guardian for every source edit.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
