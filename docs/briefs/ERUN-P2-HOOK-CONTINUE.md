# Brief: ERUN-P2-HOOK-CONTINUE

**Agent**: worker
**Phase**: PHASE-ERUN-P2
**Created**: 2026-10-02 08:57:44

---

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

## Ownership amendment

The continuation lane may update the existing HookPolicyPage test file to mock Next navigation and verify fail-closed invalid hints, current-session scope replacement, and authorized Worktree selection. Commit the exact relevant test file with the page; no dependency/lock changes or unrelated tests. Root owns all other files and serial integration.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)


Continuation: the prior agent handle is missing after interruption. Resume the existing uncommitted draft in C:/Users/leo19/.codex/worktrees/erun-p2-hook-deep-link/Star; do not restart or discard it. Current remote dev is 1068ea40. Finish the owned implementation, review it, run appropriate targeted verification (existing tests permitted now, no broad unnecessary tests), commit owned files with author Ulysses. Do not push or merge; coordinator owns serial rebase/integration. Current AGENTS.md new section 0 includes Project/Branch/Run tree, Native Hook, Rust resource bounds, actual Task CLI closed-loop and documentation consistency; follow these, report genuine production blockers. Graph MCP/index tools are unavailable in parent and no .codegraph; use exact source and qualify graph evidence as unavailable. RTK and Orca CLI are unavailable in this host. Briefs stay owned by root. You are not alone; preserve others changes. No unsupported historic narratives.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
