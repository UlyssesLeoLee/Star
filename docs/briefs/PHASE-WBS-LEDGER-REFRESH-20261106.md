# Brief: WBS ledger refresh after naming remediation

## Task

Refresh the WBS integration ledger against the current `dev` tip after the
Star product-naming web and CLI remediation lanes have been reviewed and
integrated.

## Ownership

- Lane: `codex/wbs-ledger-refresh-20261106`
- Base: `dev` at lane creation
- Owned files: this brief, the phase report, and
  `docs/reports/PHASE-WBS-INTEGRATION-LEDGER-20261105.md`
- Do not touch source code, other documentation, user-dirty files, `main`,
  remote refs, or any other worktree.

## Required result

1. Update the ledger target SHA and integration rows to the actual current
   `dev` history.
2. Record the accepted web remediation (`d9684285`) and CLI remediation
   (rebased/integrated `72fecc30`) with their file scopes and validation facts.
3. Preserve unresolved naming audit items (compatibility IDs, demo assets,
   package descriptions) as explicit TODOs; do not claim global closure.
4. Produce a phase report with the required §0–§7 sections, including exact
   commands and results, known gaps, and cleanup status.

## Acceptance

- `git diff --check` passes.
- Ledger/report links resolve to committed files.
- No external product brand appears in Star user-facing copy introduced by
  this lane; audit-only historical references remain explicitly scoped.
- The report distinguishes verified integration from unverified remote or
  release state.
