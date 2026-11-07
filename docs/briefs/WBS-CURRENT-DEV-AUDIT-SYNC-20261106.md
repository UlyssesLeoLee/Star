# Brief: WBS-CURRENT-DEV-AUDIT-SYNC-20261106

**Agent**: worker
**Phase**: WBS-CURRENT-DEV-AUDIT-SYNC-20261106
**Created**: 2026-11-07 05:35:45

---

## Objective
Update the current-dev WBS documentation from dev@6634a455bc475a66128390d36d956c47b659fe9a, preserving older snapshots and the six-file manifest. The coordinator supplied verified facts; the candidate source/worktree must not be opened.

## Verified facts and required boundaries
- VCD-0 candidate 4f0ceae9818b7bd2b3b25d9164d6e43303121689: read-only Git ancestry is 392 dev-ahead / 5 candidate-only. Use metadata only; reject candidate-only commits.
- W8 commits 08c37448 and 769943db are ancestors of current dev; the focused synthetic parser/fail-closed create-path suite passed 31 assertions. No live Multipass inventory, lock contention, Stopped/Deleted counting, two-instance VM, K3s or disk-capacity verification.
- W16 design commits 37a35feb and 6634a455 are ancestors of current dev; payload SRS/BD/DD v0.8 and Canvas test design v2.6. New UT/IT/E2E remain NOT_RUN. Some older reports/reviews retain pre-integration or pending-review wording; record this status drift in the WBS only.
- W16-A existing HTTP/PG report: four named cases and 10/10 runner steps PASS on disposable PostgreSQL 18 with Rust/Cargo 1.99.1. Exact baseline 1.99.0 compatibility NOT_RUN; test-only audit/SCD2 prerequisites; production grants/RLS/global audit and full W16-A remain PARTIAL/OPEN.
- WLD-7 active set: SRS v1.11, BD v1.12, Group DD v5.33, collaboration spec v1.2, owner-local data v0.4, test design v2.6. Spec/data retain v1.7 citation drift. docs/specs/domain-collaboration-spec.md:303 has three missing links to basic-design.md (title basic-design v0.16), architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md, and architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md. Record these as separately scoped source-document follow-ups; do not modify those sources.
- Stable release: W2 OPEN/BLOCKED; archive metadata hashes only, do not open Cargo.lock. W4 OPEN/BLOCKED; latest record only brief/action=brief. W6 ADAPT/OPEN. W8 OPEN. W16/WLD-7 OPEN/PARTIAL/NOT_RUN. v1.0.6 remains NO-GO.
- W8/W16 worktrees are integrated/claimed released but not cleaned; ignored scripts/automation/__pycache__ Python bytecode remains. Do not claim cleanup complete.

## Required edits
Only the six registered manifest paths: this brief, its dispatcher JSONL, Stable Release WBS, Worktree Group WBS, VCD-0 current-dev review and this phase report. Preserve all prior WBS snapshots and revisions; add the current snapshot. Ensure links point to existing reports/reviews and report the old W8/W16 wording drift without editing unregistered reports.

## Validation
Document/link checks and git diff --check only. Do not run tests, Cargo, database, cloud, VM/K3s, build, performance, release, deployment, fetch or push. Do not rebase, merge or clean; coordinator performs review and integration.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
