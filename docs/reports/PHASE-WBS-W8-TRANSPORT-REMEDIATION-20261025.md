# PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025

**Status:** bounded source-level transport remediation implemented; focused Rust gates pass; PostgreSQL runtime acceptance remains open
**Base:** `dev` at `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10`
**Branch:** `codex/wbs-w8-transport-remediation-20261025`
**Scope:** native BI Task-event receive adapter, durable source-origin identity, quarantine conflict isolation, and least-privilege native receive grant

## 0. Purpose

Address the two W8 blockers identified in the dirty, read-only candidate: `platform_transport.pull_one` must remain callable only by its restricted runtime role, and source-position identity must isolate a sequence/cursor reused with a different message identity. The candidate was reviewed file by file; only code and migrations compatible with the current dev owner-local durable transport contract were selected. This phase does not close W8 or the broader BI/Benchmark work.

## 1. Change matrix

| Path | Change | Evidence boundary |
|---|---|---|
| `Cargo.toml`, `Cargo.lock` | Register the bounded native BI event adapter and resolve its workspace dependencies | Package-specific Rust tests compile and pass |
| `crates/bi-native-event-adapter/` | Add a profile-pinned native gRPC bridge into the existing BI owner receiver; preserve exact origin correlation and defer terminal disposition until owner persistence and origin verification | 4 adapter tests pass; no live mTLS or PostgreSQL evidence |
| `crates/bi-owner-runtime/src/task_event_inbox.rs` | Persist provider-neutral sequence and opaque cursor with Inbox receipts; reject mismatched source identity; return a distinct fail-closed result for legacy Inbox rows without verifiable source origin | Owner unit/static migration tests pass; database test remains ignored |
| `crates/bi-owner-runtime/src/task_event_receiver.rs` | Carry source origin through commit/quarantine and preserve commit, authorization, and disposition ordering | Owner and adapter tests pass |
| `crates/bi-owner-runtime/src/task_event_quarantine.rs` | Bind quarantine writes to provider/log/generation and retain sequence plus exact cursor | Unit/static migration tests pass; DB function is unexecuted |
| `db/migrations/2026-10-24-bi-task-event-source-origin.sql` | Add durable Inbox origin; serialize source position checks; return stable exact receipts; enforce quarantine sequence/cursor uniqueness at the logical provider/log/generation scope; fail migration on historical duplicate origins instead of choosing a winner; retain legacy sequence as NOT NULL | Static SQL assertions pass; migration has not been applied to PostgreSQL |
| `db/migrations/2026-10-24-platform-native-transport-lease-order.sql` | Revoke broad execution then grant `pull_one` only to `star_platform_transport_runtime` | Static test passes; catalog/role behavior not run on PostgreSQL |
| `crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs` | Add source-level assertions for the pull grant and retained lease-order contract; database catalog test remains opt-in | Targeted source test passes; ignored DB tests remain unrun |
| `docs/briefs/WBS-W8-TRANSPORT-REMEDIATION-20261025.md` | Preserve the lane brief and its ownership boundary | Documentation only |
| `docs/reports/PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025.md` | Record this phase, exact validation, and unresolved runtime gates | This report |

The dirty candidate remains unchanged. Candidate behavior that weakened or broadened source-position uniqueness was not retained. No source-sequence `DROP NOT NULL`, broad `PUBLIC` grant, or whole-branch merge was used.

## 2. Verification summary

### Rust and source-level gates

- `rtk cargo +1.99.0 test -p bi-owner-runtime --all-targets -j 4`: passed, 19 passed, 0 failed, 1 ignored. The ignored test requires the dedicated `star_bi_task_event_test` PostgreSQL database.
- `rtk cargo +1.99.0 test -p bi-native-event-adapter --all-targets -j 4`: passed, 4 passed, 0 failed, 0 ignored. The run waited on another Cargo build lock before completing.
- `rtk cargo +1.99.0 test -p platform-messaging-runtime --test native_origin_verification_pg pull_one_execute_is_regranted_only_to_the_runtime_role -j 4`: passed, 1 passed, 0 failed, 2 filtered. It waited for another active Cargo build; no other process was terminated.
- `rtk cargo +1.99.0 fmt -p bi-owner-runtime -p bi-native-event-adapter -- --check`: passed after formatting the changed owner package.
- `rtk run rustfmt +1.99.0 --check --edition 2021 crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs`: passed.
- `rtk git diff --check`: passed after the final quarantine-scope correction and report update.
- CypherGraph Guardian `cgg_guard.py --changed --stamp`: passed for all 6 changed Rust source files after the final source edit. The Guardian script excludes `.sql` from source-file validation; both migration freshness digests were independently recomputed with the Guardian's canonical algorithm and matched their headers.

The adapter test's first final rerun encountered a transient Windows `LNK1104` on the shared Cargo target while another Cargo process was active. After that process exited, a retry waited for the shared cache lock and passed. The linker issue was transient build-target contention, not a code fix; the Cargo target may contend again while other lanes build concurrently.

One owner all-target run initially failed because a static assertion matched SQL indentation literally. The assertion was made whitespace-insensitive and the full owner all-target run then passed with the results above.

### Not passed or not executed

- `rtk cargo fmt --all -- --check` reports formatting drift in unrelated repository files; lane-scoped formatting checks pass.
- CodeRabbit could not run because the installed executable returned `/bin/bash: /root/.local/bin/coderabbit: Permission denied` for its version/auth checks. Local source review was performed; no independent CodeRabbit result is claimed.
- No PostgreSQL connection is available to this lane (`psql` and `sqlfluff` are absent; the dedicated BI and platform PostgreSQL test URL variables are unset). No SQL migration, grants, RLS, index catalog/EXPLAIN, concurrency, or stored-function behavior has been runtime-verified.

## 3. Known gaps

1. Apply both migrations to a dedicated disposable PostgreSQL instance and test least-privilege grants, FORCE RLS, append-only triggers, exact replay receipt stability, same sequence with a different message ID, same cursor with a different sequence, concurrent duplicate attempts, and migration reapplication.
2. Verify native mTLS, server identity, source revocation, crash/restart recovery, and settlement against the actual runtime.
3. Complete W8 worker lifecycle, source-owner settlement, owner Inbox/Outbox integration, checkpoints/completeness, BI query coverage, and Benchmark projections.
4. Keep execution and production acceptance fail-closed until runtime evidence covers the gaps above. No in-memory queue or socket-write success is treated as durable acceptance.

## 4. Subagent failure and handoff

- No child agents were dispatched from this lane.
- The old candidate worktree remained read-only; no stage, commit, archive, cleanup, or file write occurred there.
- Parent integration owner: apply this lane only after reviewing its commit and keeping PostgreSQL/runtime acceptance open. This commit is not a dev merge and does not update shared WBS/AGENTS files.

## 5. Guard rules

1. Keep BI persistence owner-local and transport-neutral; provider ACK APIs never define business truth.
2. Pin provider, log, log generation, source sequence, opaque cursor, and message identity as one immutable delivery origin.
3. Scope source-position uniqueness by tenant, consumer, provider, log, and generation; route/profile revision must not weaken it.
4. A reused sequence or cursor with any different message identity is a conflict and cannot create a second receipt.
5. Exact replay returns the original durable receipt identity and timestamp only when origin, route, wire digest/length, and disposition facts match.
6. Missing legacy origin data is unverifiable and must fail closed before terminal ACK.
7. Check and lock the source identity before owner Inbox/quarantine writes; database unique indexes remain the final collision guard.
8. Quarantine stores bounded metadata and digests only, never event bytes.
9. Preserve `source_sequence NOT NULL`; fail migration on historical duplicate source origins instead of silently deleting or selecting rows.
10. `platform_transport.pull_one` is executable only by `star_platform_transport_runtime`; never restore `PUBLIC` execution.
11. Keep SECURITY DEFINER functions on a fixed search path and preserve tenant scope/RLS checks.
12. Owner commit or durable quarantine must complete before terminal transport disposition.
13. Keep wire and cursor lengths bounded at the Rust and database boundaries.
14. Revalidate the immutable route/profile and original native origin before disposition.
15. Apply SQL and role assertions only against a dedicated disposable database; source-string tests do not prove catalog behavior.
16. Do not mark W8 production-complete until PostgreSQL, mTLS, concurrency, restart/recovery, and settlement gates have real evidence.

## 6. Signoff

| Role | Signoff |
|---|---|
| Architecture | Mavis handoff review: bounded source-level change; W8 remains open |
| SRE Lead | Runtime database, migration, recovery, and settlement evidence pending |
| Platform | Least-privilege migration and native adapter source reviewed; PostgreSQL catalog behavior pending |
| Review Host | Local self-review completed; CodeRabbit unavailable with permission error |
| PM | Phase scope and explicit open gates recorded; parent integration review pending |

## 7. Revision history

| Version | Date | Editor | Revision | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-25 | Ulysses — Mavis handoff | Selectively implement W8 durable source identity and restricted pull grant; record source-level gates and runtime gaps | WBS-W8 transport remediation brief |
