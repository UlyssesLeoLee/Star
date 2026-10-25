# WBS-W8-CANDIDATE-REVIEW-20261025

**Status:** Not ready to integrate as-is; keep W8/WLD6 OPEN.
**Candidate:** `C:\Users\leo19\.codex\worktrees\wbs-w8-bi-native-adapter-20261024\Star`, HEAD `b660fa52a23d352539123eafc9ef6b005a978d71`.
**Current dev comparison:** `2fda1e16632470312ef07b18fb5bf81066822466`.
**Review mode:** read-only candidate review; only this review file was written in the review lane.

## 0. Summary

The adapter and owner changes form a coherent source-level bridge: the route/profile pins are checked before connecting; one bounded delivery retains the native origin; the BI owner persists that origin alongside the Inbox/fact; and the owner receiver controls commit-before-ACK and quarantine-before-terminal-disposition. The candidate also adds a durable per-log lease-order guard.

Two migration issues prevent integration as-is:

1. The lease-order migration revokes `star_platform_transport_runtime`'s execute grant on `pull_one` without restoring it, so the native receiver cannot pull after migration.
2. The quarantine migration weakens the prior unique source-sequence protection for legacy rows when a delivery at the same old sequence has a different message ID. It can insert a new cursor-keyed quarantine row instead of returning conflict.

PostgreSQL migration/role behavior, mTLS, restart/recovery, and owner-to-owner settlement remain unverified. No PostgreSQL or mTLS/runtime completion is claimed.

## 1. Findings

### P1 — Lease-order migration removes runtime permission to pull

**Path:** `db/migrations/2026-10-24-platform-native-transport-lease-order.sql:104-108`

The migration replaces `platform_transport.pull_one`, assigns the owner, then executes `REVOKE ALL ... FROM PUBLIC, star_platform_transport_runtime`; it has no following `GRANT EXECUTE` for the runtime role. The baseline migration grants this exact function to `star_platform_transport_runtime` at `db/migrations/2026-10-12-platform-native-transport-log.sql:628`. Because `CREATE OR REPLACE FUNCTION` retains existing ACLs and this migration explicitly revokes the runtime grant, applying it removes the permission the native receiver needs.

**Trigger and impact:** after applying the migration, a native receive request running as `star_platform_transport_runtime` invokes `pull_one` and is denied before it can lease a message. The new W8 receiver bridge therefore cannot operate against the migrated database.

**Required before integration:** explicitly restore `GRANT EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER) TO star_platform_transport_runtime` after the revoke. Verify `has_function_privilege` with the runtime role against the migrated target DB and include a pull assertion in the disposable-PostgreSQL acceptance run.

### P2 — Legacy quarantine rows no longer conflict on the old source sequence when message identity differs

**Paths:**

- New guard: `db/migrations/2026-10-24-bi-task-event-source-origin.sql:318-340`
- New cursor uniqueness: `db/migrations/2026-10-24-bi-task-event-source-origin.sql:235-243`
- Prior source-sequence uniqueness: `db/migrations/2026-10-17-bi-task-event-quarantine.sql:50-57`
- Native cursor encoding: `crates/bi-native-event-adapter/src/lib.rs:347-353`

The old quarantine table's uniqueness key identifies a source position by route/log identity plus `source_sequence`, independent of `message_id`. The new migration makes `source_sequence` nullable and adds a different unique key only for non-null `source_cursor`. Its legacy-row check matches a cursor-less row only when `legacy.message_id = p_message_id` (lines 318-336). A conflicting delivery for the same old source position but a different message ID bypasses that check; its new row has `source_sequence = NULL` and is not covered by the old unique constraint, while the new cursor index has no key for the old row. The function can then return `recorded` and permit the receiver's later terminal disposition.

The adapter's current provider serializes its native sequence to eight-byte big-endian cursor bytes, so this edge case is statically identifiable for the current provider. The BI contract intentionally treats cursor bytes as opaque, however; the migration must not silently assume every provider uses this encoding.

**Required before integration:** retain fail-closed conflict behavior for a legacy cursor-less row at the same source position, including when the message ID differs. Choose an explicit compatibility strategy that respects the opaque-cursor boundary (for example, provider-specific legacy conversion outside BI or a conservative conflict disposition until the legacy origin can be resolved). Add a database regression with the same legacy route/sequence and a different message ID; assert no new quarantine receipt is recorded and no terminal ACK becomes eligible.

## 2. Positive source-level evidence

- `crates/bi-native-event-adapter/src/lib.rs:242-264, 337-365, 376-402` validates the pinned binding before connecting, rejects an origin outside the expected scope/provider/log/generation and wire-size bound, and retains the exact native origin for subsequent verification.
- `crates/bi-native-event-adapter/src/lib.rs:421-435` maps the protocol's terminal ACK only through the owner receiver's post-commit disposition call and clears the single pending-origin slot when delivery is dropped.
- `crates/bi-owner-runtime/src/task_event_receiver.rs:389-425` passes the same origin into Inbox commit and performs post-commit origin verification before ACK. The new `SourceOriginUnverifiable` path is covered in the source tests by `crates/bi-owner-runtime/src/task_event_receiver.rs:807-824` and the adapter boundary test at `crates/bi-native-event-adapter/src/lib.rs:753-807`.
- `crates/bi-owner-runtime/src/task_event_inbox.rs:445-539` verifies that the source origin matches the pinned provider/log/generation and event identity, calls `commit_task_event_v3`, compares the stored origin, and commits the owner transaction before returning a receipt.
- `crates/bi-owner-runtime/src/task_event_quarantine.rs:69-108, 139-212` checks provider/log/generation against the route and passes opaque cursor bytes to the owner function.
- `db/migrations/2026-10-24-platform-native-transport-lease-order.sql:52-86` adds per-destination claim serialization and blocks another pull while an unexpired lease exists; the index predicate matches the active `leased` guard at lines 18-21 and 66-75.
- The two-message active-lease assertion is present at `crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:253-294`; the static SQL predicate check is at lines 502-513, and the database catalog assertion is at lines 515-532. These are source evidence only here; the target is ignored and was not executed.
- The phase report and detailed design keep the runtime gates open: `docs/reports/PHASE-WBS-W8-BI-NATIVE-ADAPTER-20261024.md:64-85, 116-143` and `docs/design/DD-BI-NATIVE-RECEIVER-001.md:88-116, 135-151`.

## 3. Integration and rebase assessment

Do not merge this dirty candidate yet. Fix both findings and reconcile the report before integration. The candidate is based on `b660fa52`; current dev is `2fda1e166`. Comparing the candidate's changed paths to current dev showed no intervening changes in the scoped source/migration/design paths except the lockfile. `Cargo.lock` has current-dev state that removes `domain-scm` from the `bi-owner-runtime` dependency list; preserve that newer dev state and regenerate/reconcile the lockfile after rebasing rather than taking the candidate lockfile wholesale.

After the fixes, rebase the reviewed candidate onto current dev, inspect the complete post-rebase diff, and integrate only the necessary adapter, owner, migration, focused-test, and design/report files. The candidate's historical review briefs and dispatch JSONL are not runtime product inputs; include them only if repository traceability policy requires them. Update the WBS from the integration checkout, not from this stale base.

## 4. Evidence actually collected in this review

- Read the complete tracked diff and all untracked candidate source, migrations, design/report, briefs, and dispatch record. The candidate was not edited, formatted, tested, committed, rebased, merged, or cleaned.
- Ran `rtk git diff --check` in the candidate; it passed for the tracked diff. This does not validate untracked-file whitespace or migration execution.
- Compared the original `pull_one` grant in the baseline migration with the candidate's replacement migration.
- Used the current dev CodeGraph only as baseline call-flow context; the candidate's dirty modifications were read directly. No candidate coverage/completeness claim is made.
- Did not run Cargo, PostgreSQL, mTLS, service, restart/recovery, or owner-to-owner settlement checks. The candidate phase report records focused Rust 1.99.0 commands as previously passing, but those historical claims were not independently re-executed in this review.

## 5. Remaining W8/WLD6 gates

- Apply both migrations to a dedicated disposable PostgreSQL database under the production-equivalent owner/runtime roles; verify function ACLs, FORCE RLS, grants, direct-DML denial, exact Inbox/source-origin replay, cursor-conflict quarantine, active-lease blocking, and catalog/index behavior.
- Exercise the native service with mTLS identities, profile revocation, route mismatch, ACK uncertainty, cancellation, process restart, lease expiry, and exact replay.
- Implement and verify source-owner accepted settlement and bounded worker lifecycle; the adapter itself does not close owner-to-owner settlement.
- Keep W8/WLD6 OPEN for other owner event streams, durable checkpoints/completeness, recovery, BI query coverage, and versioned Benchmark projections.
