# Dev Branch Convergence Report — 2026-10-18

## 0. Purpose

Integrate only reviewed, still-useful content into `dev`. The integration base was `94d2c82c3be2678e5cc959dd38598cabdcc0779c`; `main` remained unchanged.

## 1. Change and branch matrix

| Scope | Finding | Action |
|---|---|---|
| Existing local branches | Ten branch tips are ancestors of or equal to the integration base. | No duplicate merge. |
| `codex/neutral-product-copy` | Tip `0bb81f6c` is 26 commits behind `dev` and has three commits outside `dev` ancestry; one commit's patch is already present in `dev` as `6cd18fa5`. Forty-six of 56 compared paths are identical. The remaining changes include stale route/App-ID aliases, missing `runDirectory` translations, and a restored mock connector. | Do not merge the branch or its commits wholesale. |
| Credential examples | The only selected source changes are six neutral placeholder strings in `en.ts`, `ja.ts`, and `zh-CN.ts`. | Change the display-name examples to a generic API service account and the base URL example to `provider.example.invalid`. |
| Worktrees | Most clean checkouts contain ignored build, report, or validation data; two contain shared dependency junctions, and one contains a PostgreSQL data directory whose use is unverified. | Preserve them pending data-specific retention decisions. `task-outbox-dispatcher` is the sole archive candidate; its archive request remains queued pending confirmation. |

Audited branch tips: `codex/bi-task-event-projection` `203b93c7`, `codex/d20-cross-provider-diagnostics` `d15951f9`, `codex/d20-p2-docs` `8e973f9c`, `codex/d20-pg-receipts` `27fc6f2a`, `codex/d20-route-pin-ops` `28778219`, `codex/phase1-gap-audit` `de31aaa7`, `codex/run-task-command-port` `90996791`, `codex/task-outbox-dispatcher` `d72b64e6`, `codex/phase9f4g-bi-query` `94d2c82c`, and `dev` `94d2c82c`. The review candidate was also based on `94d2c82c`.

## 2. Validation summary

- `git diff --check`: passed for the selected locale edits.
- CypherGraph Guardian `cgg_guard.py --changed --stamp`: passed for all three changed source files.
- Read-only review found no prohibited competitor-comparison copy in tracked frontend strings. Remaining branded matches were compatibility identifiers, a legacy URL alias, an actual connector type, comments, or tests.
- Frontend tests/build were not run; this slice changes only placeholder text. No push or remote-ref refresh was performed.

## 3. Known gaps

- Static frontend inspection does not cover backend-provided labels, user-authored content, or runtime injection.
- Cleanup is incomplete because ignored user data, shared `node_modules` junctions, and a PostgreSQL directory cannot be safely removed based on the available evidence.
- The current local `origin/dev` tracking ref was one commit behind the inspected local `dev`; remote freshness has not been refreshed.

## 4. Parallel review outcomes

- Product-copy review: rejected whole-branch integration; identified the six selected locale strings and the stale behavior changes to exclude.
- Branch-topology review: confirmed the ten other source branch tips are already integrated by ancestry; `codex/neutral-product-copy` is the only branch with unique commits.
- Worktree review: found no live Orca terminals or agents, but identified ignored-data and ownership-retention blockers for bulk cleanup.

## 5. Integration gates

- Integrate from an isolated candidate based on the recorded `dev` SHA; recheck `dev` cleanliness and SHA before fast-forward.
- Do not move `main`, force-update history, push, or delete remote branches.
- Archive only through Codex Worktree management after ignored data and owner safety are established.
- Keep all product navigation, Run authorization, task CLI, routing, and seed behavior from the current `dev` implementation.

## 6. Review record

Reviewed by the coordinating agent with parallel read-only branch, product-copy, and Worktree audits. No user-authored historical claims were added. The report records locally observed state only.

## 7. Revision history

| Date | Revision | Change |
|---|---|---|
| 2026-10-18 | v1 | Recorded selective locale integration, branch ancestry audit, validation, and cleanup blockers. |
