# Brief: ERUN-P2-HOOK-DEEP-LINK

**Agent**: worker
**Phase**: ERUN-P2
**Created**: 2026-10-02 07:29:32

---

Make the existing advanced visual Hook policy editor consume the Run shell deep link safely.

Base: origin/dev at 3966dab74f6af157e25b12f205f50bbd8966f861. Private independent worktree; rebase onto latest origin/dev before handing off. Read AGENTS.md. CodeGraph MCP/index tooling and .codegraph are unavailable; use exact source reads/rg and make no graph-coverage claim.

Ownership: only frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx. Add/update the Cypher structural manifest in that file. Do not edit Run tree/shell, Providers, API schemas, backend, advanced navigation or docs; another lane owns CLI and root will reconcile docs after both changes.

Context: RunWorkspaceShell links to /settings/advanced/hooks?project_id=<authorized project>&worktree_id=<focused checkout>. HookPolicyPage currently ignores the query and silently defaults to the first authorized project/worktree. Make this deep link useful without allowing route values to become authority: parse/validate IDs as hints, resolve them against the current authenticated API's authorized Project directory and that Project's Worktree directory, and only then select the project/worktree policy view. A valid Worktree deep link should bring the editor to that Worktree scope and visibly show its Project/Worktree context. If either hint is malformed, absent from its authorized parent list, not yet loaded, or unavailable because session/API failed, display an explicit blocked/needs-selection/error state and do not silently switch into another project's writable policy editor. Ordinary navigation without hints must retain existing behavior. Never auto-publish or change policy content because of a deep link; deep links only select scope/read state, and edits remain explicit user actions. Preserve current Senior Settings → Hooks tab, typed visual editor, fail-closed authorization API use and cancellation/late-response protections. Do not add run_id or query parameters to writes.

Acceptance: lint/type-consistent URL parsing, no unauthorized or ambiguous default on invalid Run deep links, accessible explanatory state, minimal file scope, detailed explanation for root's doc update. Commit only the owned page, cite this brief plus scripts/automation/engineering_run_directory.py and docs/automation-design.md §4.36. Run static review/compile-only frontend typecheck if the independent checkout has dependencies cached; do not run tests. Report commit, review evidence, exact checks, limits. Do not force integrations.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
