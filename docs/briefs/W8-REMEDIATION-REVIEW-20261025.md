# Brief: W8-REMEDIATION-REVIEW-20261025

**Agent**: worker
**Phase**: WBS
**Created**: 2026-10-25 13:37:34

---

# W8 remediation independent review

## Objective
Independently review candidate commit 0ae310440e446f530a210ec413efb526cb038d8f against its parent and the latest dev base. This lane is review-only: do not edit implementation/migrations or the source branch. Write only docs/reviews/WBS-W8-REMEDIATION-REVIEW-20261025.md and commit the report on this lane.

## Baselines and scope
- Latest dev baseline: 3dd01fcc11f771fac95a74d9e6ea4fe978bb1e65.
- Candidate source branch: codex/wbs-w8-transport-remediation-20261025 at commit 0ae310440e446f530a210ec413efb526cb038d8f. Review its complete diff against its parent; candidate is not yet merged.
- Changed scope includes crates/bi-native-event-adapter, bi-owner-runtime TaskEvent Inbox/quarantine/receiver, PostgreSQL source-origin verification tests, and two SQL migrations plus W8 implementation report.
- Prior reports flagged two defects to verify precisely: runtime grant/order around platform_transport.pull_one; quarantine uniqueness/correlation for a repeated durable source cursor with a different message identity. Check the full constraints, grants, owner/RLS boundaries, conflict/replay semantics, and whether the tests actually exercise the regression.
- Parent CodeGraph status at latest dev: 1,905 files / 37,350 nodes / 117,596 edges, index up to date. Query: TaskEventInbox TaskEventQuarantine TaskEventReceiver NativeTaskEventAdapter. Relevant call paths: TaskEventReceiver calls origin verification before persistence and disposition; metadata-only quarantine occurs before terminal ack; TaskEventInboxStore.commit_exact_wire commits exact wire/profile route pin through owner DB function bi.commit_task_event_v2. CodeGraph reports no covering tests for TaskEventReceiver symbols; inspect existing targeted test files and migration harness.
- MCP session did not expose check_index_coverage. Treat that as an evidence limitation. Do not make exhaustive negative claims.

## Review tasks
1. Review every changed migration and security-sensitive source hunk. Look for revokes/grants ordering, role grants in deploy manifests, trigger/RLS invariants, idempotency, source identity, exact wire digest and immutable receipt behavior.
2. Compare the new native adapter to the stable transport contract: no JetStream-specific business dependency, no in-memory durability claims, bounded payload/logging, no secrets in reports.
3. Review Rust and PostgreSQL test coverage and inspect the prior W8 phase report for what was actually run. Separate passed checks from unrun PostgreSQL/mTLS/runtime/settlement checks.
4. Return a clear verdict: merge-safe as a partial remediation, or block with actionable findings. W8 must remain OPEN unless its own DB/RLS/mTLS/recovery/settlement evidence is complete. Do not overstate closure.
5. Report exact file/line references, severity, rationale, and concrete fix. If merge-safe, explicitly say whether rebase to current dev is expected to be clean and any integration gate.

## Boundaries
Only create/commit docs/reviews/WBS-W8-REMEDIATION-REVIEW-20261025.md. Do not touch the implementation lane, existing user briefs, D:\Star, or unrelated branches. Do not merge or push.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
