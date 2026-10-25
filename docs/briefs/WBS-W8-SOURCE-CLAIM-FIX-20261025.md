# Brief: WBS-W8-SOURCE-CLAIM-FIX-20261025

**Agent**: worker
**Phase**: WBS
**Created**: 2026-10-25 13:59:08

---

## Objective
Fix the independently reviewed W8 P1 defect: BI Inbox and metadata quarantine currently arbitrate a transport source position in separate uniqueness domains, so concurrent or reordered deliveries can produce two terminal outcomes for one source position.

## Evidence and baseline
- W8 candidate source commit: 0ae310440e446f530a210ec413efb526cb038d8f on codex/wbs-w8-transport-remediation-20261025.
- Independent review: docs/reviews/WBS-W8-REMEDIATION-REVIEW-20261025.md in the review lane; its P1 requires one owner-local claim across Inbox and quarantine, plus PostgreSQL regression for both write orders.
- Parent CodeGraph on the current dev checkout was queried for TaskEventReceiver, TaskEventInboxStore, TaskEventQuarantineStore, and SubagentDispatcher. It confirms Inbox ACK follows durable commit, quarantine records before terminal acknowledgment, and the Rust adapters call PostgreSQL owner functions. The W8 candidate checkout has no .codegraph index, so no CodeGraph claims are made for its changed source; use focused RTK search/read there.
- No live PostgreSQL, RLS, mTLS, restart-recovery, or settlement verification has yet been evidenced. Do not mark W8 closed or merge its candidate based only on Rust tests.

## Required outcome
Implement an atomic, durable, owner-local source-position claim that is shared by the Inbox and quarantine terminal paths. It must bind the complete relevant source scope (tenant, consumer/route, provider identity, generation, log identity, sequence/cursor and immutable event/message identity as supported by existing contracts) so a position cannot be claimed under a second terminal disposition.
- Exact same-position/same-content retry is idempotent.
- Any different identity/content attempting the claimed position fails closed and cannot create a second terminal receipt/fact or authorize ACK.
- Verify both orderings: Inbox A then quarantine B; quarantine A then Inbox B.
- Preserve the existing wire privacy boundary: quarantine stores bounded metadata/digest, never raw wire.
- Use an append-only migration. Classify each table as Work/Transaction/Master, define RLS/ownership/grants and migration-time duplicate detection; fail migration if existing cross-table collisions make the invariant ambiguous.
- Keep producer/provider transport details behind the adapter. No NATS-specific business identity or ACK dependency.

## Owned paths
Only these pre-registered lane paths may be changed:
1. db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql
2. crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs
3. docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md

Do not edit the existing W8 candidate's source files, migration, tests, docs, any other lane's files, or D:\Star. If the owned scope proves insufficient, stop and report required scope; do not widen the manifest.

## Test requirements
Add an actual disposable-PostgreSQL regression using the repository's existing test harness. Assert both cross-disposition orderings, exact replay, identity/content conflict, one terminal receipt at most, and no second ACK authorization. Check database catalog constraints/functions and RLS when supported by the harness. Run Rust 1.99.0 focused formatting and tests; report each exact command and observed result.
If PostgreSQL is unavailable, report the test as blocked/unrun. Never substitute an in-memory model and never claim PostgreSQL validation passed.

## Handoff
Before marking ready, commit only owned paths, report the exact HEAD/changed files/tests/limitations, and leave W8 overall OPEN pending independent review and remaining PostgreSQL/RLS/mTLS/recovery/settlement gates.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
