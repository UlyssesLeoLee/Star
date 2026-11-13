# WLD7-COLLAB-CROSSREF-SOURCE-SYNC-20261107

## Objective

Reconcile the current version references between the WLD-7 collaboration spec and its owner-local data design after their separately reviewed citation updates landed in `dev`.

## Baseline and scope

- Base: local `dev@914d4a67b31dce7ed364a19134b7a8d694816ce9`.
- Source commits already in the base: `4859b573` (spec links/test citation) and `914d4a67` (data-design test citation).
- Update only the current references: spec → data design v0.5; data design → spec v1.3. Preserve historical revision entries and all implementation-status language.
- This lane does not edit the Group WBS because MAGOS reports an overlapping registered WBS candidate. No content from that candidate is read or changed.

## Acceptance

1. Current document versions are spec v1.4 and data design v0.6; their active references point to the counterpart versions v0.5 and v1.3 respectively.
2. Append citation-only revision entries v1.4 and v0.6; prior revision history remains intact.
3. Test design remains v2.6 in both documents.
4. `git diff --check` passes, and the manifest contains exactly the two source documents, this brief, its dispatch record, and the phase report.
5. No code, migration, database, runtime, or product behavior is claimed as implemented or verified.
