# Brief: WBS-W8-SOURCE-CLAIM-REVIEW-20261025

**Agent**: reviewer
**Phase**: WBS
**Created**: 2026-10-25 14:24:44

---

## Review objective
Independently review W8 source-position arbitration fix commit 07a896e3561affb885594b9fb53b65b199921fa6 against the original P1 finding and determine whether its migration and regression test close the cross-disposition uniqueness defect at source level.

## Required review scope
Inspect only the commit diff:
- db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql
- crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs
- docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md
Compare to parent 0ae310440e446f530a210ec413efb526cb038d8f and the original review report docs/reviews/WBS-W8-REMEDIATION-REVIEW-20261025.md.

Check specifically:
1. Inbox source-origin and quarantine insert triggers create a single owner-local claim inside their existing receipt transactions, and trigger order sees the Inbox fact before source-origin insertion.
2. Shared sequence and non-null cursor uniqueness serialize both race directions; same-position replays are accepted only for the exact same disposition, sequence, cursor, message identity, and digest.
3. On conflict the transaction rolls back, receiver has no path to a second terminal ACK or receipt.
4. Migration collision detection, locking, backfill, W/T/M classification, RLS FORCE, grants, SECURITY DEFINER/search_path, and append-only guard are correct and do not leak raw wire.
5. PG test uses separate runtime and owner audit connections, verifies both connect to the same dedicated loopback database, scopes owner reads with tenant RLS, and actually exercises both orderings, replays, conflict, and race behavior.

## Evidence boundaries
The implementation report states Rust 1.99 fmt, test target compile/no-run and Cypher Guardian passed; actual PostgreSQL execution is blocked/unrun because no DB URLs or psql are available. Do not claim migration/catalog/RLS/concurrency runtime proof. Do not execute external code review services or share repository contents externally.

## Output
Append a clearly dated addendum to the existing owned report docs/reviews/WBS-W8-REMEDIATION-REVIEW-20261025.md, preserving the original review record. Give findings first with severity, exact paths/lines and reasoning. State whether P1 is source-level closed, source-level blocked, or still open; separately state the real PG gate remains open. Do not edit implementation files. Do not merge, rebase, or push. Report exact review HEAD, changed path, and whether the review worktree is clean.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
