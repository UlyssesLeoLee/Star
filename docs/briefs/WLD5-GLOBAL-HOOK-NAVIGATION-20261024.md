# Brief: WLD5-GLOBAL-HOOK-NAVIGATION-20261024

**Agent**: codex-worker
**Phase**: WBS-WLD5-2026-10-24
**Created**: 2026-10-24 02:11:38

---

## Objective
Complete the WBS slice for unsaved Hook settings protection across internal navigation. Audit current behavior first; the current dev already contains UnsavedNavigationGuardProvider, GuardedLink and content-anchor interception, so do not duplicate the mechanism. Make only demonstrated fixes and missing tests.

## Baseline and graph evidence
- Target dev HEAD and lane base: 44b027ca09204ce1480667946a37f4900da10691.
- CodeGraph returned current source and relationships for UnsavedNavigationGuard and GuardedLink, showing HookPolicyPage and several draft/command components as consumers. It also found focused tests in HookPolicyPage.test.tsx and GuardedLink.test.tsx, plus CommandBar/UserMenu/provider tests.
- The current guard confirms unsaved changes, preserves managed Next Link handling, routes accepted same-origin anchors through router.push, and skips modified clicks, external targets, downloads and same-route navigation.
- No check_index_coverage or index_repository tool is exposed in this session. Treat graph coverage as bounded; read exact files for any unindexed/missed detail and make no exhaustive absence claims.

## Scope
Use only the registered WLD5 lane manifest: navigation guard, GuardedLink, HookPolicyPage and listed tests, docs/automation-design.md, scripts/automation/registry.md, and the phase report/dispatch log. Do not touch Repository master or unrelated docs.

## Required work
1. Inspect the existing focused tests and identify any real gap in behavior or lifecycle cleanup; keep changes small and consistent with the existing architecture.
2. Add or repair tests for the demonstrated cases, including cancellation and accepted navigation where missing.
3. Update the automation registry/design and a seven-section phase report to match actual behavior; do not mark production host authentication as implemented.
4. Run focused pnpm tests and typecheck if dependencies are available; report exact commands and results.
5. Return exact diff, validation, limitations, and integration recommendation. Do not commit, rebase, merge, push, or edit other worktrees; parent owns review and integration.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
