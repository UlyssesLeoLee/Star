# Brief: WBS-W8-DURABLE-CURSOR-REVIEW-20261024

**Agent**: verifier
**Phase**: WBS-W8-DURABLE-CURSOR-REVIEW-20261024
**Created**: 2026-10-24 09:26:47

---

## Assignment
Perform a fresh independent read-only review of the W8 durable source-origin/cursor fix after the W8 implementation owner reports ready and the parent confirms their writes have stopped. Review only `C:\Users\leo19\.codex\worktrees\wbs-w8-bi-native-adapter-20261024\Star`; do not edit or integrate anything.

## Contract and prior finding
- WBS requirement baseline in dev `docs/requirements.md` §50 native BI receive contract: sequence is route pin → validate provider/source origin and destination generation → validate event → commit owner Inbox/fact → reverify source/current authorization → ACK. Exact cursor in quarantine identity is route + provider/log generation/log/sequence; conflicting digest/fault fails closed. See current lines around 3714–3716, re-open in dev if needed.
- Prior W8 review found `TaskEventReceiveOrigin` carried the native sequence to validation but it was dropped before `TaskEventInboxCommitPort::commit_exact_wire`; durable `TaskEventInboxReceipt` and `bi.commit_task_event_v2` did not retain the source cursor. It also found DD verification status stale compared with phase report.
- CodeGraph from D:\Star showed the base flow `TaskEventReceiver` → commit port → `TaskEventInboxStore` → `TaskEventInboxReceipt`; the lane itself has no CodeGraph index, so inspect its exact current diff/source directly. Do not assume D:\Star index covers it.

## Review checks
1. Source identity persisted is provider-neutral: provider/source log identity, generation, and exact opaque cursor are retained without leaking native-specific cursor semantics into unrelated owners. Current native sequence must remain exact.
2. Origin is passed into the owner-local transaction and stored atomically with Inbox/fact; it is included in exact replay identity/consistency. A replay with different origin cannot produce a successful exact replay or ACK.
3. Source auth is checked before commit and rechecked before ACK; ACK still follows durable owner commit; unknown commit result/cancel/revocation remains unconfirmed; no cross-owner writes or shared transaction were added.
4. Migration ownership, W/T/M classification, RLS/grants, WORM/append-only constraints, SQL function arity/types, migration repeatability, and tests align with existing DB contracts.
5. Tests cover persisted exact cursor, exact replay preservation, conflicting origin, and commit-before-ACK as actually implemented. Distinguish unit/target compile from PostgreSQL runtime/mTLS/restart acceptance.
6. DD and phase report reflect independent review, exact focused gates, and unresolved runtime gates; no mock/source test is described as production acceptance.

## Constraints and report
- Read only; no file edits, staging, commits, branch switch, rebase, merge, or cleanup.
- Re-read the current HEAD/status and changed diff before review. Report exact inspected HEAD, findings with severity/path/line and concrete fix, paths checked, and gates actually executed (if any). A clean review does not close PG/mTLS/restart or broader WBS gates.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
