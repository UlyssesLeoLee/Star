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

## 7. Addendum — cursorless arbitration follow-up (2026-10-25)

- **Implementation reviewed:** `fd0472ffdf2ad5fe6385b071abbbf4b6024198f2` against parent `07a896e3561affb885594b9fb53b65b199921fa6`
- **Review checkout HEAD before this addendum:** `e098fe4061e23045049aa08c0c2bef75cec6561a`
- **Scope:** manual source review of the candidate migration/test/report delta and the original cross-disposition finding plus prior shared-claim addendum. No source changes, test execution, PostgreSQL connection, RLS run, rebase, merge, push, or external review service.

### P1 — the cursorless barrier is one-way and accepts a known-to-unknown transition

**Affected paths:**

- `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:221-244, 280-294`
- `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:51-93, 128-147`
- `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs:557-648, 889-944`

The advisory lock serializes a known-cursor writer (shared lock) against a cursorless writer (exclusive lock), but the post-insert barrier is only evaluated when `source_cursor IS NOT NULL` (`...sql:280-289`). Therefore this legal READ COMMITTED ordering remains possible: a known delivery at sequence 32 takes the shared lock, inserts its non-null claim, sees no null claim, and commits; a delayed cursorless quarantine at the earlier sequence 31 then obtains the exclusive lock, inserts a distinct sequence key, skips the `ELSIF` body because its cursor is NULL, and commits. The lock orders transactions; it does not order source sequences. This leaves one generation containing known and unknown cursor history even though BI cannot determine whether sequence 31's missing cursor equals a cursor already claimed at another sequence.

The migration's historical prechecks reject duplicate sequences and duplicate *known* cursors, but do not reject a generation containing both cursorless and non-null rows (`...sql:51-93`). Its backfill preserves null quarantine cursors (`...sql:143-147`), so an already mixed legacy generation is also accepted into the new ledger. The new null-history test seeds a null row only after migration and then tests a later non-null sequence (`source_position_arbitration_pg.rs:585, 889-944`); it does not cover known-then-null or a mixed pre-migration backfill.

The regular Rust quarantine adapter currently passes a non-empty cursor and the prior SQL receiver rejects a NULL parameter (`db/migrations/2026-10-24-bi-task-event-source-origin.sql:416`), which narrows the live application path. It does not establish the database invariant: the new trigger and nullable quarantine schema explicitly support cursorless inserts, the integration fixture uses that path, and legacy null rows are backfilled. This review therefore treats the missing reverse barrier as a source-integrity defect, not as a PostgreSQL runtime finding.

**Minimum fail-closed fix:** when a *new* null-cursor claim is inserted under the exclusive generation lock, reject and roll back if that scope already contains any non-null claim. Keep exact replay on the existing `inserted_count = 0` equality-check path. Separately make migration preflight reject (or persist an explicit blocked-generation fact for) any pre-existing scope containing both NULL and non-NULL claims; do not erase append-only receipts to reconcile it. Add regression coverage for known sequence 32 followed by cursorless sequence 31, cursorless-first followed by known, exact replay of both existing claims, and migration from a pre-existing mixed scope. Re-review the new commit before treating the original P2 as closed.

**Merge effect:** block this commit as a completed source-position safety fix until the reverse transition and mixed-history migration policy are closed. A later non-null claim after cursorless history is rejected source-side, but that alone does not make the policy symmetric.

### P2 — the new tests do not exercise the lock race or migration-time legacy backfill

The candidate adds valuable sequential checks for the same non-null cursor at two sequences (`source_position_arbitration_pg.rs:509-555`) and for cursorless history followed by a new known cursor (`:557-648`). Its only concurrent test races two known-cursor dispositions at one sequence (`:451-507`); it never races the exclusive cursorless path against a shared known-cursor path. The cursorless fixture writes a post-migration row through the trigger (`:889-944`) rather than creating a legacy quarantine row before migration and verifying the migration backfill. These gaps are material because the safety claim depends on the advisory-lock ordering and the migration's historical state.

The test is explicitly ignored until a dedicated PostgreSQL database is available (`source_position_arbitration_pg.rs:191-195`); the candidate report records `0 passed, 1 ignored` and says migration, RLS, mTLS, recovery, and settlement were not run (`docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md:35-36`). Static inspection supports the lock ordering under the stated READ COMMITTED operating assumption, but this is not runtime or concurrency evidence. Add two-session database tests for both lock winner orders and an actual pre-migration cursorless backfill fixture.

### P2 — migration/test database role requirements remain unproven

The earlier BI migration declares `star_bi_owner` and `star_bi_runtime` as `NOLOGIN` and `NOBYPASSRLS` (`db/migrations/2026-10-16-bi-task-event-inbox.sql:31-45`). The phase report describes the test URLs as logging in directly as those roles (`docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md:40`), while the test connects using the URL as supplied and only verifies `current_user` (`source_position_arbitration_pg.rs:654-683`); it does not perform `SET ROLE`. A separate login role plus explicit role switching/startup configuration is needed unless the deployment has another tested setup. Also, this migration sets the claim function owner and revokes PUBLIC/runtime EXECUTE before creating the three triggers (`...sql:299-322`); the exact migration principal must have the required RLS-bypass and trigger/function privileges. The report names the RLS-bypass requirement but no actual migration identity or grant proof. This is an unresolved deployment/test setup gate, not a claim that a PostgreSQL migration was attempted or failed.

**Concrete closure:** document the supported migration principal and test login-to-effective-role setup, then apply the migration and run the ignored test under those production-equivalent roles. Confirm trigger creation succeeds after the function revocations, migration backfill sees all tenants under FORCE RLS, and runtime still cannot access the claim table/function directly.

## 8. Verdict and gate status

**Verdict: BLOCK this commit as the completed cursorless-history fix.** The known-cursor cross-sequence collision path added in this commit is represented in the test source and the previous known-cursor P1 remains source-level closed. The one-way barrier still permits a cursorless claim to follow a known claim, including at a lower sequence; the migration also admits historical mixed generations. The source-level P2 is therefore not fully closed.

**Runtime status: OPEN / NOT_RUN.** No PostgreSQL migration, RLS, advisory-lock concurrency, mTLS, recovery, or settlement test was run by this reviewer. W8 remains OPEN. The candidate report's compile/format results were read as reported evidence only; I did not reproduce them.

| Review item | Result |
|---|---|
| Candidate / parent | `fd0472ffdf2ad5fe6385b071abbbf4b6024198f2` / `07a896e3561affb885594b9fb53b65b199921fa6` |
| New P1 | Cursorless insertion can follow known claims and make generation history ambiguous; migration also backfills mixed scopes without preflight rejection |
| New P2 | Missing reverse-order lock and pre-migration backfill integration cases; migration/test effective-role setup unproven |
| PostgreSQL / RLS / concurrency / mTLS / recovery / settlement | OPEN / NOT_RUN |
| Reviewer tests / implementation edits | None / None |
| Rebase / merge / push / external review | None / None / None / None |

## 9. Addendum — symmetric barrier and isolation-gate review (2026-10-25)

- **Implementation reviewed:** `824c76d017af50795023f7e75313a4da30e0addf` against parent `7f96a066dc227ae42731884304135760c687ffc5`; ancestry includes `fd0472ffdf2ad5fe6385b071abbbf4b6024198f2` and `07a896e3561affb885594b9fb53b65b199921fa6`.
- **Review checkout HEAD before this addendum:** `e361235fa7b99f7da58789b7171a73e722026775`.
- **Scope:** manual source review of the new migration/test/report delta plus the reviewed receiver and stored-function paths. No implementation edits, PostgreSQL execution, RLS or concurrency run, rebase, merge, push, or external review service.

### Source-level P1 — symmetric mode barrier closes for new unmatched claims under READ COMMITTED

The prior known-first→cursorless-later finding is closed at source level for new unmatched claims on the trigger-backed write paths. The trigger takes an exclusive generation lock for a cursorless claim and a shared generation lock for a known-cursor claim (`db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql:233-243`), inserts the claim, then checks for the opposite cursor mode in either direction (`:292-317`). A new cursorless row after known history fails, and a new known row after unknown history fails. Both exceptions abort the same receipt transaction before the receiver can reach terminal acknowledgement. An earlier sequence arriving later is therefore rejected by the symmetric check; lock acquisition order no longer permits the mixed-mode transition identified in §7.

On `inserted_count = 0`, replay is accepted only if exactly one scoped claim matches and its disposition, sequence, cursor, source message ID, and wire digest all equal the incoming identity (`...sql:256-291`). Scope equality includes tenant, consumer, provider, source log, and generation. The Inbox and quarantine owner functions additionally compare their persisted receipt/content and route fields. The Inbox path compares tenant/project/run/work item, aggregate and event identity/version, schema, sender/receiver, transport provider/version, profile ID/revision/digest, endpoint, destination/generation, route-pin SHA, Inbox/fact wire SHA, actor, correlation, occurrence time, and payload (`db/migrations/2026-10-16-bi-task-event-inbox.sql:354-380`); `commit_task_event_v3` separately compares source event/provider/log/generation/sequence/cursor/message identity (`db/migrations/2026-10-24-bi-task-event-source-origin.sql:184-194, 257-268`). Quarantine compares source identity and sequence/cursor/message, sender/receiver, provider/version, profile ID/revision/digest, endpoint, destination/generation, wire digest/length, and fault (`...sql:491-518`). The exact known-cursor quarantine replay fixture checks the same receipt ID and then reaches the receiver's quarantine ACK path (`crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs:756-773`).

The migration deliberately preserves historical mixed-mode rows through backfill. The trigger permits an exact replay of one existing row but rejects any new unmatched claim in either mode (`...sql:128-147, 256-317`). This is a source-level fail-closed policy for future writes; it does not prove or repair the historical relationship between a cursorless receipt and a known cursor. The candidate's mixed-generation fixture simulates the post-backfill ledger/receipt state by direct owner inserts (`source_position_arbitration_pg.rs:705-753, 1186-1290`); it does not execute the migration against pre-existing rows.

The isolation gate is appropriate in principle. PostgreSQL documents that READ COMMITTED takes a statement snapshot, while REPEATABLE READ retains a transaction snapshot from its first non-transaction-control statement; VOLATILE functions obtain fresh snapshots for their internal queries, but that does not replace the transaction-isolation contract. See [PostgreSQL transaction isolation](https://www.postgresql.org/docs/current/transaction-iso.html) and [function volatility](https://www.postgresql.org/docs/current/xfunc-volatility.html). The trigger rejects other isolation levels with SQLSTATE `0A000` before its source-row digest read, generation lock, or claim write (`...sql:189-196`). This supports the symmetric mode-check reasoning for trigger-backed new claims under READ COMMITTED. The PostgreSQL behavior has not been executed here.

### P2 — the trigger-only isolation gate misses Inbox exact replay and runs after outer reads/locks

The candidate requires non-READ-COMMITTED transactions to fail with `0A000`, including exact replay, but the gate exists only in `bi.claim_task_event_source_position()` (`...sql:193-196`). `bi.commit_task_event_v3` acquires its source-position advisory locks and reads `task_event_source_origin` before the trigger (`db/migrations/2026-10-24-bi-task-event-source-origin.sql:153-183`). For an exact Inbox replay it calls `commit_task_event_v2`, verifies the saved origin, and returns `exact_replay` at lines 240-268 without inserting `task_event_source_origin`; the new BEFORE INSERT trigger therefore never runs. When the existing origin/receipt committed before the REPEATABLE READ/SERIALIZABLE transaction snapshot, this path can return the persisted receipt ID: `commit_task_event_v2` exact-matches the saved Inbox/fact fields (`db/migrations/2026-10-16-bi-task-event-inbox.sql:344-380`), `TaskEventInboxStore` commits the transaction and returns that receipt (`crates/bi-owner-runtime/src/task_event_inbox.rs:536-562`), and `TaskEventReceiver` re-verifies then calls `acknowledge()` (`crates/bi-owner-runtime/src/task_event_receiver.rs:425-435`). Therefore **an ACK can occur and the prior receipt is returned, but no new receipt or source-position claim is inserted**. The path bypasses the new claim trigger/ledger lookup, while still checking the immutable source-origin row and full Inbox receipt/content identity; it is not evidence of a duplicate write. If the relevant commit is newer than the transaction snapshot, the replay may instead fail closed, but that does not eliminate the stable-snapshot success case. The quarantine stored function also takes its source locks and reads legacy quarantine history before the trigger (`...sql:434-456`); new writes still abort at the trigger, but the gate is not actually before all source reads or lock acquisition as the brief requires.

**Disposition:** this is a mismatch with the stated isolation-gate scope, not a demonstrated source-integrity defect on the exact Inbox replay path. That path returns a previously committed, immutable receipt only after the full source-origin and Inbox/fact identity checks above; it creates no new receipt or source-position claim. Under the existing append-only/atomic-claim invariant, allowing this exact replay to ACK under REPEATABLE READ/SERIALIZABLE is safe and preserves idempotent delivery recovery. I recommend revising the isolation policy to allow only a fully matched read-only exact-replay fast path under any isolation level, while requiring READ COMMITTED before taking source-position locks or attempting any new/unmatched claim. The fast path should read from the transaction snapshot and return only when every persisted source-origin/receipt field matches; if it finds no exact row under REPEATABLE READ/SERIALIZABLE, fail closed with `0A000` before source-position locks or writes. This handles a concurrent commit newer than the snapshot without retrying into an ambiguous result. If the product deliberately retains the brief's stricter `0A000` requirement for every exact replay, then gate both exposed entry functions (`bi.commit_task_event_v3` and `bi.record_task_event_quarantine_v2`) and test exact Inbox/quarantine replays explicitly; that is stricter than needed for source-position integrity and makes replay availability depend on transaction isolation.

Whichever policy is selected must be made consistent across Inbox and quarantine. Add the corresponding non-READ-COMMITTED exact replay test: either assert the same persisted receipt and a single receiver ACK with no new row under the recommended policy, or assert `0A000`, no ACK and no new receipt if the strict policy is retained. Keep the existing new-claim isolation tests and place the non-READ-COMMITTED gate before source-position locks/writes on that path.

This is a P2 policy/entry-boundary gap, not evidence that a second source claim can currently be committed: trigger-backed new claims abort outside READ COMMITTED. The Inbox exact replay path is read-only with respect to source claims and verifies stable persisted identity, but it returns the previous receipt and authorizes an ACK outside READ COMMITTED, contradicting the specified all-path `0A000` rejection contract. For clarity, the behavior is presently asymmetric: this Inbox path can succeed, while a quarantine exact replay reaches the trigger and is rejected outside READ COMMITTED.

### P2 — mode-race and migration tests distinguish database rows from transport ACKs only partially

The candidate includes direct sequential tests for both mode orders (`source_position_arbitration_pg.rs:576-623, 625-683`), a mixed-history replay/new-claim fixture (`:705-816`), and a known-versus-cursorless race (`:818-869`). However, the cursorless participant in the known-first and concurrent scenarios is `seed_cursorless_quarantine_history`, which directly inserts through the owner audit pool rather than using `TaskEventReceiver`/`PgDelivery` (`:608-620, 840-848, 1110-1183`). The test can assert that a database insert failed or that a receipt row exists, but that fixture has no transport ACK counter. In the race, `known_mode_counts` covers only the receiver side; assertions at lines 851-869 prove one successful database path and one durable receipt, not exactly one terminal ACK across both paths. The phase report acknowledges that the direct fixture does not prove transport ACK behavior (`docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md:55`).

The isolation test also inserts a new cursorless row under REPEATABLE READ/SERIALIZABLE (`source_position_arbitration_pg.rs:685-703`); it does not test the exact Inbox replay path that bypasses the trigger. The isolation helper correctly executes `SET TRANSACTION ISOLATION LEVEL` before setting tenant scope or inserting (`:1122-1138`). The mixed-history fixture is valuable for trigger semantics but cannot prove migration precheck/backfill behavior because it manually inserts claim and receipt rows after the migration is assumed present.

**Concrete closure:** test both mode outcomes through a receiver-backed delivery (or explicitly label direct owner fixtures as receipt-only evidence); assert ACK counters for every transport-backed contender; add non-READ-COMMITTED exact-replay cases for Inbox and quarantine; and run an actual pre-existing-row migration fixture. The test remains ignored and has not been executed against PostgreSQL.

## 10. Verdict and gate status

**Source-level result:** the prior P1 known-first→cursorless-later claim insertion defect is closed for new unmatched trigger-backed claims when the transaction is READ COMMITTED. Exact claim matching includes disposition, sequence, cursor, message ID, digest and scoped source identity; normal Inbox/quarantine replay functions add persisted route/content checks. Mixed historical generations are preserved and are fail-closed for unmatched writes by source inspection.

**Review verdict:** partial source-level remediation; the new brief is not fully satisfied until the isolation gate covers all exposed write entry points and Inbox exact replay, with tests for those paths and accurate ACK evidence. The earlier §7 deployment-role and migration-identity questions remain unresolved. Do not mark W8 complete.

**Runtime status: OPEN / NOT_RUN.** This review did not execute PostgreSQL migration, FORCE RLS/role checks, shared/exclusive advisory-lock concurrency, mTLS, recovery, settlement, or the ignored integration test. Candidate-reported Rust format/check/no-run results were not independently reproduced.

| Review item | Result |
|---|---|
| Candidate / parent | `824c76d017af50795023f7e75313a4da30e0addf` / `7f96a066dc227ae42731884304135760c687ffc5` |
| Prior symmetric-mode P1 | Closed at source level for new unmatched trigger-backed claims under READ COMMITTED |
| New P2 | Inbox exact replay bypasses trigger-only isolation gate; entry gate runs after outer source reads/locks |
| Test evidence P2 | Cursorless direct-owner fixtures prove receipt effects, not transport ACK; no non-RC exact Inbox replay or actual migration backfill run |
| PostgreSQL / RLS / concurrency / mTLS / recovery / settlement | OPEN / NOT_RUN |
| Reviewer tests / implementation edits | None / None |
| Rebase / merge / push / external review | None / None / None / None |
