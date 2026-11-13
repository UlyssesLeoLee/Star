# PHASE-WLD7-COLLAB-CROSSREF-SOURCE-SYNC-20261107

## §0 Purpose

Reconcile the active version references between the WLD-7 collaboration spec and owner-local data design after the independently merged citation/link lanes.

## §1 Change matrix

| File | Change | Status |
|---|---|---|
| `docs/specs/domain-collaboration-spec.md` | Current owner-local data design reference v0.4 → v0.5; document version v1.3 → v1.4 | Reviewed; ready to integrate |
| `docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md` | Current collaboration spec reference v1.2 → v1.3; document version v0.5 → v0.6 | Reviewed; ready to integrate |
| Group WBS | Not in this manifest because MAGOS found an overlapping registered candidate | Deferred; candidate untouched |

## §2 Validation summary

- No runtime, application, database, or migration tests are applicable to this citation-only change.
- `git diff --check`: PASS.
- The two current source headers reference each other as spec v1.4/data design v0.6; both continue to cite test design v2.6: PASS.
- Three repaired relative links resolve to existing files: PASS 3/3.
- Exact changed-path manifest: PASS 5/5. No application or database tests were run because this lane changes documentation references only.

## §3 Known gaps

- The Group WBS §23.17 still reflects its earlier `dev@6634a455` snapshot and describes the v1.7 citations and source links as pending. The source-document lanes that corrected those references are integrated as `4859b573` and `914d4a67`, with cross-references now reconciled in this lane; a current WBS snapshot still needs a separately permitted manifest.
- WLD-7 owner-local schema remains logical-only: migrations, grants/RLS, target PostgreSQL acceptance, event transport, runtime collaboration, and performance gates remain open/not run.
- The current headers now reconcile at data design v0.6/spec v1.4; WLD-7 remains logical-design-only with implementation and target acceptance open.

## §4 Subagent failure and handoff

- No subagent was dispatched: this is a bounded two-line source-reference reconciliation with no independent subtask.
- No worker failure or uncommitted worker output is present.

## §5 Guardrails

- Preserve prior revision history; add only append-only citation revision entries.
- Do not alter schema semantics, runtime claims, migrations, code, or database state.
- Do not read or modify the overlapping candidate worktree or branch.
- Do not modify `main`, remote refs, or user changes in the main worktree; do not push, tag, release, or deploy.
- Integrate only after exact-manifest review and static checks.

## §6 Sign-off

| Role | Result |
|---|---|
| Architecture | Mavis review complete for the citation-only scope |
| SRE Lead | Not applicable to citation-only scope |
| Platform | Not applicable to citation-only scope |
| Review facilitator | Mavis review complete for the citation-only scope |
| PM | Mavis review complete for the citation-only scope |

## §7 Revision history

| Version | Date | Editor | Change |
|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses — Mavis review | Initial bounded cross-reference reconciliation record. |
| v0.2 | 2026-11-07 | Ulysses — Mavis review | Reconciled current references; static checks and scope review passed. |
