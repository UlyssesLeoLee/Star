# Brief: WLD5-AUTH-SOURCE-REVIEW-20261020

**Agent**: codex-worker
**Phase**: WLD5-AUTH
**Created**: 2026-10-20 08:00:42

---

## Target
Trace the real authenticated session path for Advanced Settings -> Hooks and Group API calls. Inspect only the existing provider/session and Hook page/auth plumbing, including:
- frontend/src/app/providers.tsx and its actual call sites
- frontend/src/lib/run/runDirectorySession.tsx
- frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx
- frontend/src/app/api/group/[...path]/route.ts
- narrowly related tests and the existing PHASE-WLD5 host-session report

## Change
Find whether the shipped root app currently supplies a trusted user/session token callback and non-secret session generation. If a real issuer/provider already exists, wire HookPolicyPage/Group to that provider and add focused tests for session invalidation, 401/403 handling and no preview fallback. Keep bearer credentials server-side or in the existing callback boundary; do not invent a token, mock issuer, cookie name, URL, or permissive fallback.
If no real issuer is wired, make no product-code change. Add docs/reports/WLD5-AUTH-SOURCE-REVIEW-20261020.md with exact path-level evidence, what is and is not wired, and the minimal prerequisite owner/API contract; update no WBS status beyond recording the finding. Use the repository's seven-section report format when adding a report.

## Constraints and ownership
You are not alone in the repo. WLD0 owns docs/basic-design.md, docs/design/DD-WORKTREE-GROUP-001.md, and the WBS; do not edit those. Do not revert or overwrite other lanes. No secrets, tokens, or actual cookie values in output. Preserve fail-closed behavior when provider/session is absent; do not use preview data as a production substitute.

## Acceptance
Use exact source evidence and rtk proxy rg only for literal discovery; inspect bounded line ranges. Run focused tests/typecheck only if you changed frontend code, plus git diff --check and the CypherGraph guard for changed source files. Commit only owned paths and reference docs/briefs/WLD5-AUTH-SOURCE-REVIEW-20261020.md in the commit message. Report unavailable graph coverage tooling rather than claiming a complete graph audit.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
