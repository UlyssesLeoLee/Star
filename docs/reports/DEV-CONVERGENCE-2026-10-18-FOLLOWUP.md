# DEV Convergence Follow-up — 2026-10-18

## 0. Purpose

This follow-up records the requested parallel review and selective convergence against local `dev`. The target was `1b138e6362fc354b4d528e13363da4656ba653b3`; the isolated candidate was based on that exact SHA. Review found that all nine existing feature-lane tips were already contained in `dev`. The only additional tip, `main` at `8dcb65f7cc5602d4f7c6b681c368d5097bc12a22`, changes one navigation paragraph in `AGENTS.md`; its intended hierarchy and stronger constraints are already present in `dev` and the design documents. No product-code cherry-pick or merge was needed.

## 1. Branch and change matrix

| Source | Tip | `dev...source` left/right | Decision |
|---|---|---:|---|
| `codex/bi-task-event-projection` | `203b93c73d82db08f1eaa389f8df669fa6acbd10` | 8 / 0 | Already contained; no source-only commit |
| `codex/d20-cross-provider-diagnostics` | `d15951f9d2c720127899ebdd03df868a205be968` | 35 / 0 | Already contained; no source-only commit |
| `codex/d20-p2-docs` | `8e973f9cd08941fa7d5a12e03b40822eca4affda` | 34 / 0 | Already contained; no source-only commit |
| `codex/d20-pg-receipts` | `27fc6f2a85497b3cf098eceed71a82ea96f83162` | 43 / 0 | Already contained; no source-only commit |
| `codex/d20-route-pin-ops` | `2877821974ef0bec1dc3aec7f6fc831c113d34a5` | 38 / 0 | Already contained; no source-only commit |
| `codex/phase1-gap-audit` | `de31aaa7de0cee38efc27d0a14acb40f4f830ee9` | 36 / 0 | Already contained; no source-only commit |
| `codex/phase9f4g-bi-query` | `94d2c82c3be2678e5cc959dd38598cabdcc0779c` | 2 / 0 | Already contained; no source-only commit |
| `codex/run-task-command-port` | `90996791d0efc27cdcd7f09699e3caf7eda827fc` | 12 / 0 | Already contained; no source-only commit |
| `codex/task-outbox-dispatcher` | `d72b64e6e66576e7c9a6fe2d9c1bb556ce06eebf` | 11 / 0 | Already contained; no source-only commit |
| `main` | `8dcb65f7cc5602d4f7c6b681c368d5097bc12a22` | 117 / 1 | Reviewed; do not cherry-pick because `dev` already has the full intent and additional host-independent Run, naming, and local-Branch constraints |

The `main`-only commit has parent and merge-base `608de5f1ce1a090179700afb350af5560bb88a69`, and changes only one `AGENTS.md` paragraph. The `dev` paragraph already fixes Project → Engineering Run → Worktree → Branch, treats the Branch as the local checkout ref/state, keeps remote host association optional, and adds the product naming boundary. The requirements, basic design, and detailed design on `dev` also state this hierarchy. Replaying the shorter `main` paragraph would add no required behavior and could weaken adjacent policy.

## 2. Validation summary

- Rechecked all recorded source tips against `refs/heads/dev`; each feature-lane right-side count is zero.
- Orca reported the `dev` checkout clean, with no active terminal, PTY, or agent at review time.
- All source worktrees had no tracked or untracked changes. Their ignored files were separately inventoried.
- `git diff --check` passed for the convergence report and current candidate diff.
- No code changed, so Cargo, frontend, database, and runtime tests were not run.

## 3. Known gaps and retained data

- Local `origin/dev` is `4a383cdd5dd2871e3fcb84de104da0dbbcd3aca8`; it is three commits behind local `dev`. No fetch or push was performed, so live remote freshness is unknown.
- All nine source worktrees contain ignored files (from 3 to 145,393 ignored entries). They remain linked and their branches remain present. This pass did not remove branches or worktrees.
- Ignored contents include reports and validation evidence, build outputs, shared dependency Junctions, and PostgreSQL data. The task-event PostgreSQL directory reported no running server during inspection, but its data remains preserved. Shared dependency targets were not touched.
- The temporary integration candidate also contains the dispatcher audit log and Python cache files. It remains available for inspection.
- `main` was not moved, rewritten, or modified.

## 4. Parallel review and handoff

| Reviewer | Scope | Outcome |
|---|---|---|
| Topology auditor | All nine feature lanes, `main`, exact ancestry | Confirmed all feature tips are contained; only one `main`-unique documentation commit exists and its intent is already covered on `dev` |
| Architecture reviewer | `8dcb65f7` navigation paragraph and design alignment | Confirmed the current `dev` rule is more complete; recommended importing no source commit |
| Worktree auditor | Orca/Codex ownership, ignored data, cleanup safety | Confirmed no live Orca activity; advised retaining the data-bearing worktrees |

No reviewer edited a branch or checkout. Review briefs are committed under `docs/briefs/DEV-CONVERGE-20261018-*.md`.

## 5. Integration gates

- Keep `dev` as the only integration target; integrate only reviewed, necessary source changes.
- Since all feature-lane commits are already in `dev`, do not replay them.
- Do not cherry-pick the reduced `main` paragraph over the richer `dev` rule.
- Fast-forward `dev` only from the isolated candidate after rechecking target HEAD, cleanliness, and Orca activity.
- Preserve `main`, remote refs, and all ignored worktree data. Do not push or delete.

## 6. Review record

The orchestrator reconciled the local Git graph and current worktree inventory. Independent subagents completed the topology, architecture, and worktree reviews. No source branch was merged because no source branch had unique feature commits. The candidate carries only this audit record and the three review briefs.

**Review sign-off:** Architecture review — Mavis (reviewed); topology review — completed; worktree/data review — completed; runtime/infrastructure validation — not applicable because no runtime change was made; product-owner direction — user-provided Project → Run → Worktree → Branch design.

## 7. Revision history

| Date | Revision | Change |
|---|---|---|
| 2026-10-18 | v1 | Recorded current branch ancestry, reviewed the main-only navigation-policy change, captured parallel review results, and documented ignored-data retention gates. |
