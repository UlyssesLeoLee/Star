# Brief: record static naming remediation in the WBS ledger

## Scope

Update the convergence ledger after `f27c3e6f` so it records the static demo,
delivery-page, generator, and non-published package-description remediation.
The target is the current local `dev`; no source or release artifacts may be
changed by this lane.

## Owned files

- `docs/reports/PHASE-WBS-INTEGRATION-LEDGER-20261105.md`
- this brief
- `docs/reports/PHASE-WBS-STATIC-REMEDIATION-LEDGER-20261106.md`

Do not touch `main`, remotes, user-dirty worktrees, or any other path.

## Acceptance

- Ledger target is the exact current `dev` SHA.
- The static remediation row links to its phase report and records its 97/97,
  109-package, and seven-package `publish=[]` evidence.
- Remaining browser, image, SBOM, signature, deployment, and compatibility
  gaps remain explicitly open.
- `git diff --check` and link/path checks pass; report uses §0–§7.
