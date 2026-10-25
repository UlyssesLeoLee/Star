# WBS-W8-REMEDIATION-REVIEW-20261025

- **Review status:** BLOCK — one P1 source-identity integrity defect remains in the Inbox/quarantine boundary
- **Candidate:** `0ae310440e446f530a210ec413efb526cb038d8f`
- **Candidate parent:** `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10`
- **Latest dev base reviewed:** `bf374449cdc848c3c6aac7ad251da38affa4dedb`
- **Review type:** bounded manual source review; no external review service or runtime tests used

## 0. Scope and method

Reviewed the candidate's complete commit summary and changed source, migration, test, and phase-report files against its parent. Focused on `platform_transport.pull_one` ACL ordering, source identity persistence, Inbox/quarantine replay and collision semantics, owner/RLS boundaries, receiver disposition ordering, and whether tests exercise the stated regressions. The parent-provided CodeGraph evidence was Verify-tier for the named receiver/store symbols; `check_index_coverage` was unavailable, so this is not an exhaustive repository audit.

The candidate phase report records passing focused Rust gates and source assertions, but says PostgreSQL migration/catalog/RLS/concurrency behavior, mTLS, recovery, and settlement were not run (`docs/reports/PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025.md:31-49`). I did not rerun tests because this lane is manual-review-only.

## 1. Findings

### P1 — Inbox and quarantine do not share a source-position uniqueness boundary

**Affected paths:**

- `db/migrations/2026-10-24-bi-task-event-source-origin.sql:153-173, 226-233`
- `db/migrations/2026-10-24-bi-task-event-source-origin.sql:434-456, 458-520`
- `crates/bi-owner-runtime/src/task_event_receiver.rs:437-468`

`bi.commit_task_event_v3` takes `bi-task-origin-*` advisory locks and searches only `bi.task_event_source_origin` before committing an Inbox/fact receipt. `bi.record_task_event_quarantine_v2` takes separate `bi-task-quarantine-*` advisory locks and searches only `bi.task_event_quarantine`. The unique indexes also apply independently to each table: the source-origin indexes are at migration lines 63-72; quarantine indexes are at lines 347-356. Neither writer checks the other table, and the two writers do not serialize on the same source-position lock keys.

This permits two inconsistent outcomes for the same tenant/consumer/provider/log/generation/sequence/cursor with different message IDs:

1. If a valid Inbox origin already exists, `commit_task_event_v3` returns `conflict` for the changed message. The receiver routes that error to quarantine, where a new quarantine row can be recorded because that source position is not yet in the quarantine table. The receiver then re-verifies and terminally acknowledges the delivery (`task_event_receiver.rs:437-468`). This creates a second durable receipt for the same source position.
2. If a message was quarantined first, a later valid delivery with a different message ID is checked only against `task_event_source_origin`; absent an Inbox origin, it can be committed as a BI fact while the conflicting quarantine receipt remains.

**Required fix:** enforce one owner-local source-position claim shared by Inbox and quarantine. Prefer a single immutable source-origin registry with an outcome/receipt reference, written atomically with either the Inbox/fact or quarantine receipt. If retaining two tables, both SECURITY DEFINER writers must acquire the same deterministic sequence and cursor locks and check both tables before writing, with a fail-closed rule for conflicting or already-terminal quarantine origins. Add PostgreSQL regression cases for both write orders and prove the second identity cannot create a second receipt or become ACK-eligible.

**Merge effect:** this is a blocker for the candidate as submitted. The `pull_one` ACL repair is independently correct at source level, but the combined commit is not merge-safe until this integrity boundary is fixed and re-reviewed.

## 2. Prior findings checked

### `pull_one` grant ordering — source-level fix is correct

The migration assigns the function to `star_platform_transport_owner`, revokes execution from `PUBLIC` and `star_platform_transport_runtime`, then restores execution only to `star_platform_transport_runtime` (`db/migrations/2026-10-24-platform-native-transport-lease-order.sql:106-111`). This resolves the prior missing-regrant defect. The new Rust test checks migration text (`crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:522-536`); it does not establish the deployed PostgreSQL ACL. Catalog verification under the production-equivalent roles remains required.

### Legacy quarantine source-sequence collision — source-level fix is present

`source_sequence` remains required and the new quarantine function returns `conflict` when a matching legacy row has no cursor, regardless of message ID (`db/migrations/2026-10-24-bi-task-event-source-origin.sql:343-356, 444-455`). The new unique source-sequence index also preserves the legacy position key. This addresses the previously documented case where a different message ID at an old cursor-less sequence could have created a new quarantine receipt. The existing Rust regression is a source-string assertion (`crates/bi-owner-runtime/src/task_event_quarantine.rs:221-271`), not an executed database test.

## 3. Validation and evidence limits

- Candidate report states `bi-owner-runtime` all-target tests passed (19 passed, 1 ignored), native adapter tests passed (4), and the targeted grant assertion passed (1); these are candidate-reported results, not independently reproduced here (`docs/reports/PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025.md:33-35`).
- The database test in `native_origin_verification_pg.rs` is explicitly ignored until a dedicated PostgreSQL database and migrations are available (`crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:190-195`). The migration and cross-table collision behavior therefore lack runtime evidence.
- The source-level quarantine regression checks migration text, not an actual collision transaction. The missing cross-table cases are not covered by the reviewed tests.
- Candidate report states no PostgreSQL grants/RLS/index/concurrency behavior, live mTLS, restart recovery, or settlement was verified (`docs/reports/PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025.md:49-56`).
- A read-only comparison found merge base `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10`; no scoped candidate paths changed between that base and latest dev. Rebase is therefore expected to be clean in the reviewed scope, but no rebase was performed.
- `git diff --check` for the candidate commit completed without reported whitespace errors. No claim is made about full build, SQL execution, or runtime readiness.

## 4. Verdict and W8 state

**Verdict: BLOCK this candidate commit as a whole until P1 is fixed.** The lease-order grant repair and legacy-sequence safeguard are source-level improvements, but the two durable receipt stores do not enforce a single source-position identity across their boundary.

**W8 remains OPEN.** Even after the source fix, closeout requires disposable PostgreSQL migration/catalog/RLS and concurrency evidence, mTLS and provider-identity verification, crash/restart recovery, owner settlement, plus the remaining W8 worker, Inbox/Outbox, BI, and Benchmark acceptance work. Keep execution and production acceptance fail-closed until those gates have evidence.

## 5. Review record

| Item | Result |
|---|---|
| Implementation edits | None |
| External diff sharing | None |
| Tests run by reviewer | None; manual source review only |
| Rebase / merge / push | None |
| Report author | Ulysses — Mavis handoff review |

## 6. Addendum — independent shared-claim review (2026-10-25)

- **Implementation reviewed:** `07a896e3561affb885594b9fb53b65b199921fa6` against parent `0ae310440e446f530a210ec413efb526cb038d8f`
- **Review checkout HEAD before this addendum:** `f83d1dbf08f54456ea29b40f98b98ddc3332a70b`
- **Scope:** only the implementation commit's migration, PostgreSQL test, and phase report, plus the original finding above. This review did not execute PostgreSQL or tests.

### P2 — Legacy cursorless quarantine claims cannot arbitrate a cursor-only collision

**Affected paths:**

- `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:85-132, 218-251`
- `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs:785-799`

The new claim ledger permits `source_cursor` to be `NULL` for compatibility with legacy quarantine rows (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:91-112, 128-132`). Sequence uniqueness still protects every such row, including a later claim at the same sequence. For claims with known cursors, the partial unique cursor index and trigger conflict lookup reject a reused cursor at a different sequence.

However, an old quarantine row whose cursor is unknown is backfilled with `NULL`. A future claim at a different sequence with the same opaque cursor cannot match that legacy row: the unique index excludes `NULL`, and the trigger lookup compares cursors only when the stored row has an equal non-null cursor. Because BI must treat provider cursors as opaque, it cannot infer the missing cursor from sequence. The historical sequence key prevents same-sequence duplication, but this migration cannot establish cursor uniqueness against an unknown legacy cursor across different sequences.

The PostgreSQL test helper also sets every cursor to `sequence.to_be_bytes()` (`crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs:785-799`). It therefore cannot exercise the cursor-only unique-index path where two records have different sequences and the same non-null cursor.

**Required before claiming global cursor uniqueness:** either resolve legacy cursorless rows with a trusted provider-specific backfill, or fail closed for affected provider/log generations until those rows are reconciled. Add a PostgreSQL case with two different sequences sharing one cursor, and a legacy cursorless case that verifies the selected conservative policy. Do not derive a cursor from the sequence in BI.

### Original P1 — source-level cross-disposition arbitration is closed for the sequence and known-cursor keys

The new migration locks both receipt tables before duplicate detection and backfill (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:38-39, 42-79`), creates one append-only claim ledger with a scoped sequence primary key and a partial unique index for non-null cursor (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:85-112`), and backfills both dispositions before enabling writes through triggers (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:113-132, 272-279`). It rejects existing cross-disposition sequence or known-cursor collisions before installing the ledger rather than choosing a winner.

The two `BEFORE INSERT` triggers claim inside the existing Inbox/source-origin or quarantine transaction. The Inbox trigger reads the already-created Inbox receipt digest before source-origin insertion (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:172-190`), matching the v3 order. On an insert conflict, the trigger accepts only one existing row with identical disposition, sequence, cursor, message identity, and digest; otherwise it raises and aborts the surrounding receipt transaction (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:205-251`). Thus Inbox→quarantine and quarantine→Inbox conflicts at the same sequence cannot create a second receipt or reach terminal ACK. The WORM trigger, forced tenant RLS, owner policy, runtime revocations, and `SECURITY DEFINER`/fixed `search_path` boundary are present (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:134-149, 258-279`). The claim stores identity and digest metadata only.

The added integration test covers exact replay, changed content, both cross-disposition write orders, a concurrent Inbox/quarantine race, failed retries, terminal receipt counts, runtime versus owner-audit roles, same-database loopback checks, and tenant-scoped owner reads (`crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs:183-190, 197-499, 505-730`). The trigger/unique-index assertions are source-supported; the integration test is still ignored and was not run.

**P1 result:** source-level CLOSED for the original same-sequence cross-disposition defect and for collisions involving known non-null cursors. The legacy cursorless limitation above remains an explicit P2 rollout constraint; it prevents claiming complete historical cursor uniqueness until reconciled.

**PostgreSQL gate:** OPEN / UNRUN. The phase report records `0 passed, 1 ignored` and says migrations, FORCE RLS, role behavior, and concurrency were not executed (`docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md:32-34`). This source review is not database evidence. W8 remains OPEN for this gate and the previously listed mTLS, recovery, settlement, and remaining W8 acceptance work.

| Review item | Result |
|---|---|
| Source-level original P1 | Closed for sequence and known non-null cursor arbitration |
| P2 legacy cursor-only limitation | Remains; requires explicit resolution or fail-closed rollout policy |
| PostgreSQL migration / RLS / concurrency | Unrun; acceptance gate remains open |
| Tests run by reviewer | None; manual source review only |
| Implementation edits / rebase / merge / push | None |
