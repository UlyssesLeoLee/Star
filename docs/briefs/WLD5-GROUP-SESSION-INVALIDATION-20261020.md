# Brief: WLD5-GROUP-SESSION-INVALIDATION-20261020

**Agent**: orca-codex
**Phase**: WLD-5
**Created**: 2026-10-20 22:35:58

---

## Target
Current local dev baseline only. Inspect and change the Group client/session boundary in frontend/src/lib/group/worktreeGroupApi.ts, frontend/src/lib/group/groupProjection.tsx, frontend/src/app/providers.tsx and directly related focused tests. Write evidence to docs/reports/WLD5-GROUP-SESSION-INVALIDATION-20261020.md. Do not edit shared requirements, basic design, detailed design, WBS, or backend lifecycle files.

## Change
Close the bounded WLD-5 host-session lifecycle slice recommended by the existing audit. When a real RunDirectoryHostSession is supplied to Providers, pass the same live getAccessToken callback and non-secret generation to both Run Directory and Group clients. Group HTTP 401/session expiry must invalidate only the still-current session generation, clear protected Group policy/draft/projection state, cancel or ignore old-generation reads, and keep writes disabled until a replacement session is provided. Preserve ordinary 403 as a scoped denial. Never retry a failed mutation under a replacement actor/session.

## Constraints
- There is no verified host token issuer in this repository. Do not invent one, read browser cookies, add demo/seed credentials, or make the root application appear authenticated. RootLayout must remain fail-closed until a real host supplies the session.
- Do not persist bearer tokens in browser storage or logs. Request a current token for each API call.
- Preserve existing Advanced Settings → Hooks placement and the visual policy editor behavior.
- No scope broadening, mock production provider, or shared database write.
- Keep all changes within the ownership list; ask the coordinator through Orca if a required file falls outside it.

## Ownership
This lane owns only the listed frontend session/client/provider files, directly related frontend tests, and the named report. Do not modify dispatcher briefs or any shared design/WBS document.

## Observable acceptance
- Tests prove shared callback/generation wiring when a host session is injected, 401 invalidation and protected-state clearing, late old-generation response suppression, no mutation replay, missing-token fail closed, and 403 remains a scoped authorization error.
- Run the focused frontend tests and typecheck available on this host. Report exact commands/results and do not claim a real host-auth integration.
- Report changed files, verified behavior, remaining issuer/runtime/target DB gates, and whether the slice was implemented or blocked.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
