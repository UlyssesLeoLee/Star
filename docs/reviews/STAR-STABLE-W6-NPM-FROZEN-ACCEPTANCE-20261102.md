# Review: STAR-STABLE-W6-NPM-FROZEN-ACCEPTANCE-20261102

## Scope

Reviewed the evidence-only npm acceptance lane against its registered brief. The permitted deliverables are the lane brief, phase report, review, and dispatch JSONL. No implementation, lockfile, workflow, WBS, main, or remote change is part of this review.

## Findings

| Severity | Finding | Disposition |
|---|---|---|
| Informational | `npm ci --no-audit --no-fund` succeeded and package-lock SHA-256 was unchanged. | Accepted as local frozen-install evidence. |
| Informational | `npm run typecheck` passed. | Accepted as local typecheck evidence. |
| Medium | Unconstrained `npm test` exited 1 after 109 files / 936 tests, with two unhandled Vitest worker errors including a V8 allocation failure. | Correctly retained; not presented as a pass. |
| Informational | `npm test -- --maxWorkers=4` passed 111 files / 950 tests. | Accepted as bounded local rerun; does not erase the default-run failure. |
| Medium | `frontend/package.json` has no `packageManager`; pnpm lockfile and active pnpm consumers remain in `p1e-integration.yml`. | W6 remains open pending a package-manager decision and consumer alignment. |
| Medium | Node 22.19.0 is below the declared engine range for three installed packages. | Installation succeeded with warnings; target Node alignment/retest remains open. |
| Medium | E2E, production build, hosted Vercel/Neon/Project-owner acceptance were not run. | No hosted or full release claim is made. |

## Review decision

**Accept as an evidence-only lane record, with W6 and stable release gates OPEN.** The phase report distinguishes the default test failure from the constrained passing retry and enumerates remaining package-manager and hosted acceptance gaps. Before integration, the coordinator should verify the four-file manifest and ensure no out-of-scope changes entered the commit.

## Review history

| Version | Date | Reviewer | Result |
|---|---|---|---|
| v0.1 | 2026-11-02 | 架构师（Mavis 接手 agent per DEC-008）— lane self-review | Accept as evidence-only; W6 remains open |
