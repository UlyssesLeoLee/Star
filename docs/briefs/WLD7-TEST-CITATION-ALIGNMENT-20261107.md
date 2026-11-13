# Brief: WLD7-TEST-CITATION-ALIGNMENT-20261107

**Agent**: codex
**Phase**: WLD7 test-design citation alignment
**Created**: 2026-11-07 10:28:03

---

## Task
Compare the WLD-7-specific test-design anchors in TEST-DESIGN-WORKTREE-CANVAS-001.md v2.8 against the active citations in domain-collaboration-spec.md v1.5 and WLD-7-COLLABORATION-DATA-DESIGN.md v0.7, which currently cite test-design v2.6.

## Current baseline
Branch codex/wld7-test-citation-alignment-20261107, based on dev@22b92756ad762f27297c493760d95435ef98f2d1.

## Scope
Only these six manifest paths may be written:
- docs/specs/domain-collaboration-spec.md
- docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md
- docs/briefs/WLD7-TEST-CITATION-ALIGNMENT-20261107.md
- docs/reports/PHASE-WLD7-TEST-CITATION-ALIGNMENT-20261107.md
- docs/reviews/WLD7-TEST-CITATION-ALIGNMENT-20261107.md
- docs/reports/WBS-DISPATCH-WLD7-TEST-CITATION-ALIGNMENT-20261107.jsonl

## Required work
1. Compare the WLD-7 cases and source anchors in v2.8 with the cited v2.6 semantics. Focus on the WLD-7 portions of §4.8, AC-COLLAB mapping, and §9.5; do not infer semantic change from the document version number alone.
2. If the WLD-7 semantics and anchors are equivalent, update only the current spec/data citations to v2.8 and append one accurate revision-history entry to each edited source document. Preserve prior revisions.
3. If they differ, leave source citations unchanged and record the exact affected cases/anchors and recommended next step. Do not edit the test-design document: it is owned by the active W16 lane.
4. Keep all runtime, migration, PostgreSQL, provider, deployment, and performance states OPEN / PARTIAL / NOT_RUN as evidenced. This is a citation-only document change.

## Boundaries
Do not modify the WBS, SRS, Basic Design, Detailed Design, test-design, code, main, remote refs, or any other lane/worktree. Do not inspect, copy, alter, or clean the protected Rust candidate worktree. Do not run tests, Cargo, database, VM, cloud, build, or deployment commands. Do not merge or clean up. Commit only changes within the manifest and report the diff/check results.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
## Findings and decision

- Compared the v2.6 test-design snapshot at 6634a455 with the current v2.8 document at the assigned base. WLD-7 cases E2E-COLLAB-01..11, the nine AC-COLLAB-01..09 mapping rows, and PT-09 are text-identical.
- v2.8 increases the §4.8 total from 23 to 24 by adding E2E-COLLAB-24 for W16. Its status paragraph records one PG18.6 device-revocation/SSE case as partial evidence and explicitly says this is not complete WLD-7 multi-user revision-push acceptance.
- The §9.5 source list still names collaboration spec v1.0 and data design v0.2 in both snapshots. This is a separate stale source-version cross-reference; the test-design file is outside this lane and was not changed.
- Decision: update the active citations in the collaboration spec and owner-local data design to v2.8, while preserving WLD-7 OPEN/NOT_RUN and all implementation/database/performance gaps. No test, build, database, VM, cloud, or deployment command was run.
