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


## 6. Addendum — source-position claim lane review as of 2026-11-07

- **Review status:** not ready to integrate; W8 remains OPEN.
- **Current dev baseline:** `996ae3ae7d6e4c9c075715f3799f5186734d32f0`.
- **Candidate:** `UlyssesLeoLee/w8-source-position-claim-fix-20261025` at `5fe3ea9372242d7d72f9dd416f6437475c870904`.
- **Divergence:** `git rev-list --left-right --count dev...candidate` returned `363 8`; the candidate contains eight commits absent from current dev. The tracked diff touches `Cargo.toml`/`Cargo.lock`, the native adapter and BI owner runtime, three SQL migrations, PG tests, and two phase reports. The candidate worktree also has untracked `docs/briefs/WBS-W8-DECODE-FIX-20261026.md`, which was not read or modified for this review.
- **Method:** read-only, bounded source and diff review of the candidate-side commits and relevant migrations/tests. No CodeGraph index exists for this old candidate path; the current dev index was up to date at 1,925 files, but no `check_index_coverage` tool was available. No exhaustive absence claim is made.

### Findings

#### P2 — Quarantine exact replay is rejected outside READ COMMITTED

**Affected paths:**

- `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:193-195, 341-346`
- `db/migrations/2026-10-24-bi-task-event-source-origin.sql:434-479, 482-519`

The claim trigger rejects a transaction whose isolation is not `read committed` before it tries to find or reuse a claim (`shared-position-claim.sql:193-195`). That trigger is installed `BEFORE INSERT` on `bi.task_event_quarantine` (`:344-346`). The quarantine function attempts `INSERT ... ON CONFLICT DO NOTHING` before checking for an exact stored receipt (`source-origin.sql:458-479`, then `482-519`). PostgreSQL runs the row `BEFORE INSERT` trigger before the conflict action, so a fully matching quarantine replay in `REPEATABLE READ` or `SERIALIZABLE` fails at the trigger and never reaches the exact-replay branch.

The Inbox path differs: `commit_task_event_v3` checks a matching source-origin before invoking the Inbox function and only inserts a source-origin for a new committed event (`source-origin.sql:164-205, 226-237`), so a fully matching Inbox replay can avoid the claim trigger. The phase report says exact replays in higher isolation do not create claims and remain eligible for the original disposition; that statement does not hold for quarantine replay.

**Impact:** fail closed with no duplicate receipt or ACK, but a valid exact quarantine retry cannot complete under the documented higher isolation levels. This is an availability/contract mismatch, not evidence of a second receipt. Before claiming the exact-replay contract, add a pre-insert exact-match path protected by appropriate source-position locks, or narrow the contract to require READ COMMITTED for quarantine retries too. Add an ignored PostgreSQL regression for exact quarantine replay under both higher isolation levels and correct the phase/design text to match the chosen behavior.

### Prior findings rechecked against this candidate

- **Transport `pull_one` grant:** the old missing-regrant finding is addressed at source level. The replacement migration revokes from `PUBLIC` and the runtime role, then restores execute only to `star_platform_transport_runtime` (`db/migrations/2026-10-24-platform-native-transport-lease-order.sql:106-111`). PostgreSQL catalog/role behavior was not tested in this review.
- **Legacy cursorless quarantine sequence:** the earlier different-message-id bypass is addressed at source level. The quarantine function detects an existing cursorless legacy row at the same tenant/consumer/provider/log/generation/sequence without comparing message ID and returns `conflict` before insert (`db/migrations/2026-10-24-bi-task-event-source-origin.sql:444-455`).
- **Inbox/quarantine shared source-position identity:** the candidate adds an owner-local claim table with sequence primary key and partial unique cursor index, historical duplicate checks/backfill, FORCE RLS, and owner-only grants (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:48-162`). A SECURITY DEFINER trigger checks tenant scope, maps both terminal paths to the same identity, requires READ COMMITTED, serializes cursor-mode transitions, inserts or validates the immutable claim, and rejects mismatched disposition, sequence, cursor, message ID, or digest (`:164-320`). The same claim trigger is attached to source-origin and quarantine inserts (`:341-346`). This source-level design addresses the earlier cross-table duplicate-receipt P1 for the reviewed paths; database behavior remains unverified.
- **Cursorless history / cursor-only collision:** non-null cursors have their own unique index; the trigger detects a cursor matching an existing sequence or a sequence matching an existing cursor and rejects non-identical claims (`:122-131, 245-290`). New known-cursor and cursorless claims are prevented from mixing within a generation using shared/exclusive advisory locks and symmetric history checks (`:233-243, 292-317`). Migration backfill can preserve an already mixed generation, but only exact historical replays pass; unmatched new claims fail closed. The source-position PG test includes same cursor/different sequence, both disposition orders, a concurrent Inbox/quarantine race, both cursor-mode directions, mixed legacy replay, and READ-COMMITTED isolation checks (`crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs:213-221, 227-309, 368-581, 583-876`).
- **Inbox sequence decode follow-up:** the final candidate commit adds `source_sequence` to the `commit_task_event_v3` result projection and reconstructs the durable receipt from that column (`crates/bi-owner-runtime/src/task_event_inbox.rs:493-554`). Its regression asserts the first Inbox receipt returns sequence 17 (`source_position_arbitration_pg.rs:247-255`).

### Validation and integration recommendation

The source-position PostgreSQL test is explicitly ignored unless the dedicated database and all migrations/runtime grants exist (`source_position_arbitration_pg.rs:213-221`). The candidate phase report records that the prior ignored run failed at first Inbox decode with `ColumnNotFound("source_sequence")`; commit `5fe3ea93` fixes the projection and adds the receipt assertion, but the report says the dedicated endpoints were unset and the ignored suite was not rerun after that fix. That proves neither migration/RLS/grant behavior nor the new cross-disposition, concurrency, cursor-mode, and ACK assertions. No Cargo or database tests were run for this review. Candidate-reported Rust 1.99.1 checks are historical claims, not independently reproduced here. `git diff --check dev...candidate` passed on the tracked diff.

Do not integrate the eight-commit branch as-is. The review found one P2 behavior mismatch, and 363 dev-only commits plus shared manifest/source paths require a fresh rebase and complete post-rebase review. After the quarantine replay contract is corrected or explicitly narrowed, run the dedicated PostgreSQL suite under the documented owner/runtime roles, then review only the necessary commits and reconcile `Cargo.lock` with current dev. W8/WLD6 still need migration/catalog/RLS, provider identity/mTLS, cancellation/restart, settlement, worker lifecycle, BI/Benchmark, and performance evidence; no production acceptance is established.

| Review item | Result |
|---|---|
| Candidate edits | None; candidate was read-only |
| Test commands run by reviewer | None |
| Read-only validation | Current diff `--check` passed |
| Rebase / merge / push / cleanup | None |
| Decision | Hold for P2 contract fix and dedicated PostgreSQL evidence; keep W8 OPEN |
