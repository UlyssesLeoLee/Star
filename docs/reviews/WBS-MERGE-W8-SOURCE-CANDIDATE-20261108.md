# W8 Source-Position Candidate Integration Review

**Decision: HOLD — no candidate commit is approved for selective integration.**
**Review baseline:** local `dev@6ad1a00f746234d5e296f942cc67f20f917d1609`
**Candidate:** `UlyssesLeoLee/w8-source-position-claim-fix-20261025@fceda75fb554069450214a2a9c0cef8c98a16c4c`
**Reviewer:** Mavis (independent source and contract review)

## 1. Scope and observed state

The candidate has 9 commits not in current local `dev`; `rtk git rev-list --left-right --count dev...candidate` returned `428 9`. Its diff against `dev` spans 18 files, approximately 4,767 insertions and 183 deletions. Review used local refs only; no fetch was performed, so remote freshness remains unknown.

The original candidate worktree at `D:\orcaWork\Star\w8-source-position-claim-fix-20261025` was read-only during this review. Its tracked files were clean and it had the existing untracked `docs/briefs/WBS-W8-DECODE-FIX-20261026.md`. The branch retains its active MAGOS claim. Neither the claim nor any file, ignored state, branch, or worktree was changed.

The current Group WBS states that dev uses the v2 Inbox API and that the candidate v3 source-origin/source-position work depends on a matching return schema, shared claim migration, Inbox/quarantine arbitration, permissions, and PostgreSQL tests; it records no independently safe content (WBS §2 W8 row and §23.14). This review confirms those dependencies still apply to the current local baseline.

## 2. Candidate-only commits and file/dependency matrix

| Commit | Subject | Candidate-only paths | Dependency / review finding |
|---|---|---|---|
| `0ae310440e446f530a210ec413efb526cb038d8f` | `fix(w8): isolate durable source identities and restore restricted grant` | `Cargo.lock`; `Cargo.toml`; `crates/bi-native-event-adapter/Cargo.toml`; `crates/bi-native-event-adapter/src/lib.rs`; `crates/bi-owner-runtime/src/task_event_inbox.rs`; `crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs`; `crates/bi-owner-runtime/src/task_event_quarantine.rs`; `crates/bi-owner-runtime/src/task_event_receiver.rs`; `crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs`; `db/migrations/2026-10-24-bi-task-event-source-origin.sql`; `db/migrations/2026-10-24-platform-native-transport-lease-order.sql`; `docs/briefs/WBS-W8-TRANSPORT-REMEDIATION-20261025.md`; `docs/reports/PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025.md` | Establishes the adapter, source-origin receipt, Inbox v3 function call, quarantine v2 call, and schema/grant migrations together. The code cannot be selected without both migrations and current grant review. |
| `07a896e3561affb885594b9fb53b65b199921fa6` | `fix(bi): arbitrate source positions across terminal receipts` | `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs`; `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql`; `docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md` | Adds cross-table source-position uniqueness; depends on the v3 source-origin and quarantine schema introduced by the first commit. |
| `fd0472ffdf2ad5fe6385b071abbbf4b6024198f2` | `fix(bi): block cursorless source generations` | The shared-position migration, its PostgreSQL test, and source-claim phase report | Alters the shared claim/mode barrier and corresponding test assertions; not independent of the shared-position migration. |
| `7f96a066dc227ae42731884304135760c687ffc5` | `fix(bi): enforce symmetric source cursor mode barrier` | The shared-position migration, its PostgreSQL test, and source-claim phase report | Adds symmetric sequence/cursor mode checks; depends on the same shared claim ledger and receipt contract. |
| `824c76d017af50795023f7e75313a4da30e0addf` | `fix(bi): gate source claims on read committed` | The shared-position migration, its PostgreSQL test, and source-claim phase report | Adds an isolation-level fail-closed guard; correctness depends on PostgreSQL transaction semantics and the shared claim operation. |
| `cd89b8944b8f842fbaf5f11ada3f1646aad0c49c` | `docs(w8): correct exact replay isolation scope` | `docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md` | Documentation describes the coupled implementation and is misleading if brought over without that implementation and its verification. |
| `bd90aecc1c39bdf7e647cd9768d36d7db1d55d0c` | `test(bi): record W8 PostgreSQL decode failure` | The shared-position PostgreSQL test and source-claim phase report | Records a failure from the candidate’s database test path; it does not supply an independent fix. |
| `5fe3ea9372242d7d72f9dd416f6437475c870904` | `fix(bi): decode source sequence in inbox receipts` | `crates/bi-owner-runtime/src/task_event_inbox.rs`; the shared-position PostgreSQL test and source-claim phase report | Fixes the source-sequence receipt decode after the earlier reported missing-column failure; requires the v3 SQL return columns. |
| `fceda75fb554069450214a2a9c0cef8c98a16c4c` | `fix(w8): high-isolation quarantine replay [brief: docs/briefs/WBS-W8-CLAIM-REPLAY-HIGH-ISOLATION-FIX-20261107.md]` | The shared-position migration; `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs`; the source-claim phase report; `docs/briefs/WBS-W8-CLAIM-REPLAY-HIGH-ISOLATION-FIX-20261107.md`; `docs/reports/WBS-DISPATCH-W8-CLAIM-REPLAY-HIGH-ISOLATION-FIX-20261107.jsonl` | Adds high-isolation replay behavior against the same claim schema and test contract; requires the preceding migration and Inbox/quarantine changes. |

## 3. Compatibility and merge blockers

### 3.1 Inbox API and schema are one coupled migration

Current dev's `TaskEventInboxStore` calls `bi.commit_task_event_v2` and decodes its v2 result. The candidate changes the Rust caller to `bi.commit_task_event_v3`, adds source-origin fields to the receipt, and introduces a migration defining the v3 return contract. Quarantine similarly changes from `bi.record_task_event_quarantine_v1` to v2 and adds opaque cursor/sequence fields. Cherry-picking the Rust changes, adapter, tests, or schema alone leaves caller and database contracts mismatched.

### 3.2 Shared source-position arbitration is not an independent patch

The candidate's shared-position migration serializes Inbox and quarantine terminal writes, backfills and checks historical origins, applies uniqueness to provider/log/generation plus sequence/cursor, and requires a fail-closed path for unsupported isolation levels. Both Inbox and quarantine writers, their durable receipts, and the PostgreSQL concurrency/high-isolation tests depend on this shared ledger. The follow-up commits edit the same migration and test repeatedly; none can be extracted safely as a standalone commit.

### 3.3 P1 authorization issue in the native receive migration

In `db/migrations/2026-10-24-platform-native-transport-lease-order.sql`, `platform_transport.pull_one` is a `SECURITY DEFINER` function with a fixed search path. The migration first revokes execution from `PUBLIC` and `star_platform_transport_runtime`, then grants execution back to `star_platform_transport_runtime`. The function reads caller-controlled `app.platform_deployment_id`, `app.platform_tenant_id`, and `app.platform_service_id` settings. Direct runtime execution therefore bypasses the authenticated wrapper boundary and lets the caller supply the identity settings used by the definer function. The candidate's static/opt-in test `pull_one_execute_is_regranted_only_to_the_runtime_role` asserts this grant, so the test encodes the unsafe boundary rather than rejecting it.

The current WBS requires Platform runtime access through authenticated wrappers (WBS §2 W8 row). This migration is new to current dev; applying it to a database where the file has not yet been recorded can grant the direct privilege regardless of its earlier date in the filename. The candidate must remove the direct runtime grant, retain wrapper-only grants, and update the test to prove direct calls fail while authenticated wrapper calls succeed. PostgreSQL catalog/runtime verification remains required.

## 4. Verification evidence and limits

This review ran no tests, Cargo commands, SQL, migration, database setup, or deployment.

The candidate phase reports record Rust 1.99.1 formatting/check/compile evidence and source-level assertions. They also state that dedicated PostgreSQL endpoints were unavailable and database acceptance remained `NOT_RUN`. The source-claim report records an earlier ignored PostgreSQL run failing with `ColumnNotFound("source_sequence")`; the later `5fe3ea93` commit updates the Rust selection, but the final database path was not rerun in the candidate reports. Reported compile/static checks do not establish migration correctness, grants/RLS behavior, transaction isolation, cross-table arbitration, or durable ACK behavior.

The candidate's own report marks W8 open. No evidence here changes that status.

## 5. Integration decision

**No commit is safe to cherry-pick into current dev.** The first implementation commit owns the source-origin API, adapter, Inbox/quarantine callers, migrations, and grant change. The later source-position commits depend on that contract and shared migration. Documentation/test-only commits are tied to this unintegrated implementation and should not be merged alone.

Keep the candidate branch, active claim, untracked decode-fix brief, ignored state, and worktree intact. Do not merge the 9-commit branch wholesale and do not delete or rewrite its history.

## 6. Smallest next lane

If W8 proceeds, create a fresh lane from the then-current clean `dev` after resolving the overlapping active claim through the approved MAGOS process. In one owned manifest, freeze the v2-versus-v3 contract and implement/test the complete atomic seam:

1. Specify the Inbox receipt and quarantine source-origin schema/API, including sequence, opaque cursor, provider/log/generation identity, exact replay, legacy-origin failure behavior, and migration ordering.
2. Keep source-position arbitration consistent across Inbox and quarantine; test duplicates, mismatches, concurrent claims, cursorless history, supported/unsupported isolation levels, and recovery.
3. Restrict direct `pull_one` execution to the authenticated wrapper boundary; include negative direct-call and positive wrapper-call tests with the intended database roles.
4. Run focused Rust checks and the disposable PostgreSQL migration/grants/RLS/concurrency suite against a dedicated endpoint; report target-production acceptance separately as open until actually run.
5. Update the Group WBS only after independent review and verification, without claiming execution, recovery, or production readiness that was not measured.

## 7. Revision history

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.1 | 2026-11-08 | Mavis | Added current-dev, commit-level W8 candidate review; recorded the coupled Inbox/source-position contract, direct runtime grant blocker, verification limits, and HOLD decision. |
