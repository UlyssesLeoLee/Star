# Brief: WLD1-LIFECYCLE-OP-OUTBOX-20261020

**Agent**: orca-codex
**Phase**: WLD-1
**Created**: 2026-10-20 22:35:58

---

## Target
Current local dev baseline only. Inspect and, if the owner boundary supports it, change the logical archive/restore management command in crates/star-api-rest/src/group_api/worktrees.rs, its owner-local migration if required, and focused tests. Write evidence to docs/reports/WLD1-LIFECYCLE-OP-OUTBOX-20261020.md. Do not edit shared requirements, basic design, detailed design, WBS, or other UI/backend domains.

## Change
Close one bounded WLD-1 source slice: make successful logical archive/restore state transitions leave an idempotent durable operation receipt and a versioned owner-local Outbox fact in the same owner transaction as the state change and audit. First inspect existing storage and ownership contracts. Reuse an existing compatible owner Outbox/receipt where available; do not create duplicate ledgers. The existing Hook execution event is not automatically an Outbox. If the actual owner/table boundary cannot support this without a cross-owner write or an invented contract, do not code around it: record the evidence and exact blocker in the report.

## Constraints
- Archive remains a logical index state transition. Do not delete, move, or restore a Git checkout and do not claim a production lifecycle provider is installed.
- Preserve the current authorization, Hook, lease/drain, freshness, retention-lock, CAS, tenant RLS, and fail-closed behavior. Unknown/stale state must not permit mutation.
- Keep Worktree lifecycle and Run binding owner writes separate; no cross-owner transaction.
- Do not introduce mocks, seeded success, memory-only queues, arbitrary host paths, or host shell access.
- Keep the Outbox transport-neutral, append-only, bounded, and free of secrets, prompts, and raw logs.
- If a migration is added, include the repository’s replay/freshness checks and enforce tenant scope/RLS and append-only protections. Do not touch user data or the main worktree.

## Ownership
This lane owns only crates/star-api-rest/src/group_api/worktrees.rs, a narrowly scoped new migration if the reviewed contract requires one, directly related tests, and the named report. Do not modify dispatcher briefs or any shared design/WBS document. Before adding another file, check with the coordinator through the Orca ask command.

## Observable acceptance
- Focused tests cover exact idempotent replay, changed-request conflict, audit/Outbox atomicity, stale version/fence, authorization failure, and fail-closed unknown signals as applicable to the implemented command.
- Run the relevant Rust format/check/test gates available on this host and any existing migration replay harness for the changed migration. Report exact commands and results; do not claim Rust 1.99 or target DB validation if unavailable.
- Report lists changed files, source/database evidence, remaining production provider/target DB gaps, and whether the slice was implemented or blocked.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
