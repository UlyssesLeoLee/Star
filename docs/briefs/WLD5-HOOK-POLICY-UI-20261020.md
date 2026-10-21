# Brief: WLD5-HOOK-POLICY-UI-20261020

**Agent**: worker
**Phase**: WLD5-HOOK-UI-20261020
**Created**: 2026-10-21 00:51:52

---

## Objective
Deliver one bounded, production-safe WLD-5 UI slice for the visual Hook policy surface.

## Ownership
You own only:
- frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx
- frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx
- docs/reports/WLD5-HOOK-POLICY-UI-SLICE-20261020.md

Other agents are working in separate worktrees on WLD-1 and Task CLI. Do not edit or revert their files. Do not edit the shared WBS, requirements, Basic Design, Group DD, or global navigation.

## Requirements
- Preserve Advanced Settings → Hooks as a tab.
- Keep safety-critical decisions Rust-native and fail closed. The page must never create sample policy facts or imply production auth/data when a host session/API is absent.
- Inspect current HookPolicyPage, its tests, current Group Hook API types, and WLD-5 status in docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md.
- Implement the highest-value UI gap that can be closed using the existing typed API contract. Prefer typed/no-code authoring or a clear authorization/provider-unavailable state with stable draft handling. Do not invent backend endpoints, auth tokens, mock providers, or enable publish without real authenticated capability.
- Add focused tests for the behavior you change.
- Write the report with exact changed files, behavior, validation results, and remaining host-auth/database/runtime/Hook enforcement gaps. Do not mark WLD-5 complete.
- Run focused tests/typecheck if available. If blocked, report exact command and reason.

## Commit
If you commit, include this brief path in the commit message: docs/briefs/WLD5-HOOK-POLICY-UI-20261020.md.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
