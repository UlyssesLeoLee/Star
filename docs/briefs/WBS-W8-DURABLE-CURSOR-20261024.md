# Brief: WBS-W8-DURABLE-CURSOR-20261024

**Agent**: worker
**Phase**: WBS-W8-DURABLE-CURSOR-20261024
**Created**: 2026-10-24 09:23:48

---

## Objective
Close the W8 P2 durable-source-cursor finding in the existing W8 lane. Work only in `C:\Users\leo19\.codex\worktrees\wbs-w8-bi-native-adapter-20261024\Star`; do not edit D:\Star, the dev checkout, or the WLD1 worktree.

## Baseline and current evidence
- Lane HEAD: `b660fa52a23d352539123eafc9ef6b005a978d71` (detached); dirty/untracked work is existing task-owned work and must be preserved.
- Independent review found `crates/bi-native-event-adapter/src/lib.rs` maps native origin (including sequence) to `TaskEventReceiveOrigin`, but `crates/bi-owner-runtime/src/task_event_receiver.rs` calls `commit_exact_wire` without origin. `TaskEventInboxReceipt` and `bi.commit_task_event_v2` storage preserve route/profile identity and wire digest but not the exact source log cursor. After delivery state is released, BI cannot recover that cursor.
- CodeGraph on D:\Star showed the base call path and blast radius: `TaskEventReceiver` -> `TaskEventInboxCommitPort::commit_exact_wire` -> `TaskEventInboxStore::commit_exact_wire` -> `TaskEventInboxReceipt`; it found no covering tests. This worktree has no CodeGraph index, so inspect its current lane files directly before edits and do not assume the D:\Star index contains lane changes.
- Existing independent review also found the W8 detailed design verification status stale relative to the phase report. PostgreSQL/mTLS/restart runtime acceptance remains open.

## Required work
1. Carry a provider-neutral durable source origin across the receiver/inbox commit boundary. Preserve provider identity, source log identity/generation, and the exact opaque cursor without converting it into a provider-specific public contract. For the current native adapter, retain the original sequence exactly.
2. Persist the origin in the BI owner transaction together with the Inbox/fact commit. Include it in exact-replay consistency checks so event identity cannot replay with a different source origin. Keep commit-before-ACK ordering and fail-closed behavior.
3. Add/adjust migration, Rust receipt/DTO, owner SQL call, and tests as required. Add focused regression coverage proving the durable receipt and exact replay retain/match the cursor; conflicting origin must not be acknowledged as exact replay. Preserve tenant isolation/RLS and owner boundaries.
4. Update `docs/design/DD-BI-NATIVE-RECEIVER-001.md` and `docs/reports/PHASE-WBS-W8-BI-NATIVE-ADAPTER-20261024.md` with the independent-review finding, actual focused gates, and open PostgreSQL/mTLS/restart gates. Do not claim runtime acceptance from source/tests or target compilation.
5. Run the available focused gates using Rust 1.99.0, RTK, and `-j 4`; report exact commands/results. Do not modify unrelated rustfmt files or normalize broad Cargo.lock changes.

## Ownership and integration boundary
- You own only the W8 worktree listed above. WLD1 remediation is running in a separate lane and owns its own files; do not touch it.
- Do not commit, branch-switch, rebase, merge, delete files, or clean ignored data. The parent integrator owns review, branch creation, rebase to current dev, and serialized merge.
- Before finishing, report changed paths, tests/checks, failures or unavailable runtime gates, and the exact lane HEAD. Keep the W8 worktree available for parent review.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
