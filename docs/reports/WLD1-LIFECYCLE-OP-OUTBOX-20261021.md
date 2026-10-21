# WLD-1 Worktree Lifecycle Operation Outbox Slice

**Date:** 2026-10-21  
**Status:** Bounded logical archive/restore source slice; WLD-1 remains open.  
**Author:** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 Purpose

Persist logical Worktree archive/restore transitions as owner-local facts. This slice connects the existing Worktree management plan to its audit, Outbox event and replay receipt. It does not create a production Host lifecycle provider or enable destructive checkout cleanup.

## §1 Change matrix

| File | Change | Boundary |
|---|---|---|
| `crates/star-api-rest/src/group_api/worktrees.rs` | Atomically persist changed Worktree state/version, management Audit, schema-v1 archive/restore Outbox fact and plan result; serialize same tenant/requester/idempotency-key plan creation; preserve version/retention and skip Hook/event for same-state archive no-op; add source tests. | Worktree management owner only. No cross-owner writes or delivery consumer. |
| `db/migrations/2026-10-20-worktree-management-outbox.sql` | Add append-only Outbox with per-tenant plan/version uniqueness, payload allowlist and one-version continuity constraints, FORCE RLS and UPDATE/DELETE/TRUNCATE guards. | Source migration; deployed target database and runtime grants are not proven. |
| `db/tests/worktree-management-outbox.sql` | Apply migration twice in a fresh database; verify tenant RLS, immutable triggers, archive/restore facts and invalid payload rejection. | Isolated fixture only. |
| `docs/requirements.md`, `docs/basic-design.md`, `docs/design/DD-WORKTREE-GROUP-001.md` | Synchronize exact replay, no-op, payload, W/T/M and remaining gates with the source behavior. | Product and design state remains explicit about unimplemented lifecycle capabilities. |
| `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` | Refresh WLD-1/WLD-2/WLD-5 evidence and acceptance gates. | WLD-0–WLD-7 remain open as stated. |

## §2 Validation summary

| Gate | Result |
|---|---|
| Disposable PostgreSQL | PostgreSQL 18.6 cluster initialized under a unique temporary directory; fixture applied successfully twice. RLS isolation, archive/restore positives, append-only UPDATE/DELETE/TRUNCATE, malformed extra-key/fractional/gapped-version rejects and uniqueness passed. `WLD1_POSTGRES_FIXTURE=PASS`. Cluster stopped after the run. |
| Rust focused tests | `star-api-rest` library test target compiled and passed 9/9 selected `group_api::worktrees::archive_gate_tests` (142 filtered). Used local Rust/Cargo 1.98.1 with `--ignore-rust-version` because 1.99.0 is unavailable; diagnostic only, not the required baseline gate. Existing dependency warnings remain. |
| Rust formatting | Rust 1.98.1 `rustfmt --edition 2021 --check` passed for `worktrees.rs`; not the required 1.99.0 baseline. |
| Cypher Graph Guardian | `cgg_guard.py --changed --stamp` passed: 1 Rust source file checked. SQL files are outside the guard's supported source extensions. |
| Diff check | `rtk git diff --check` passed after all source and documentation updates. |
| Target database and app role | Not tested. Migration application, API login membership, target grants and production FORCE RLS behavior remain open. |
| Host provider and cleanup | Not tested. No physical Git checkout was created, deleted, restored, drained or reconciled. |

## §3 Known gaps

1. This slice handles the existing logical `set_archived` plan only; Worktree create/import, binding/rebinding, provider reconciliation, physical cleanup and restore remain unimplemented or unverified.
2. PostgreSQL 18.6 evidence comes from a fresh disposable bootstrap cluster. It does not establish target database migration safety, runtime application-role grants, authenticated API behavior, deployment identity or historical reconciliation.
3. The local environment has Rust 1.98.1 but not the workspace-required Rust 1.99.0 toolchain. Any diagnostic compile using `--ignore-rust-version` is not baseline acceptance.
4. The source uses a transaction-scoped advisory lock to serialize the idempotency key before lookup. Database concurrency tests for exact replay, differing request hashes, rollback and lock cancellation remain open. The no-op behavior is covered at helper level, not yet through an authenticated API/database integration test.
5. The Outbox has no dispatcher, receiver Inbox, retry/lease consumer, BI/Benchmark projection or delivery acknowledgement. `occurred_at` is not a consumer high-water cursor.
6. Target application grants are absent from this migration slice. The fixture's test role proves only its declared SELECT/INSERT policy surface.
7. Runtime admission, Task CLI, Schedule execution and production BI ingestion remain fail closed pending their independent owner, authorization, sandbox, readiness and persistence gates.

## §4 Subagent failure handoff

- No WLD-1 implementation worker failure occurred in this lane. The coordinator's source review found a same-state archive mutation without an Outbox fact, broad idempotency collision mapping, missing payload version continuity, and fixture coverage gaps; this slice now addresses the state no-op, key serialization, migration constraint and positive/negative SQL cases.
- Remaining concurrency and target-environment gaps are listed in §3 and must stay open in the WBS.

## §5 Guard rules

1. Reauthorize the current Project manager before plan creation and confirmation.
2. Keep Worktree management as the owner of its state, audit, Outbox and plan receipt.
3. Scope idempotency serialization by tenant, requester and key; retain the persistent request hash as the replay authority.
4. Return the stored receipt for an exact replay; reject a reused identity with changed request content.
5. Change Worktree state only under its expected-version CAS; a failed one-row update aborts the owner transaction.
6. Treat an archive request whose target equals current state as a no-op: preserve version and retention expiry, skip archive Hook evaluation, and emit no transition fact.
7. Emit archive/restore events only for real `false → true` and `true → false` transitions.
8. Commit Worktree state, Audit, Outbox and confirmed plan result in one transaction.
9. Keep uniqueness on `(tenant, plan)` and `(tenant, Worktree, version)`.
10. Enforce exact top-level/nested payload allowlists, row identity, event semantics and adjacent integer versions in the database.
11. Exclude host paths, credentials, prompts, terminal output and raw logs from the event payload.
12. Force tenant RLS on the Outbox; missing or cross-tenant scope must not expose rows or accept inserts.
13. Reject UPDATE, DELETE and TRUNCATE on persisted Outbox facts.
14. Do not treat a persisted Outbox row as delivered, acknowledged, projected into BI or used as a cleanup command.
15. Keep Worktree Outbox transactions separate from Engineering Run, Task, Schedule, Infrastructure and BI owner transactions.
16. Require fresh authorization, lease/claim/process drain and retention-lock evidence before any future physical cleanup; unknown blocks destruction.
17. Keep Task/Schedule execution and production BI ingestion closed until their own runtime, identity, database and recovery gates pass.

## §6 Sign-off

| Role | Result |
|---|---|
| Architecture | Mavis 接手审核：approve the bounded source slice based on the recorded disposable PostgreSQL, focused diagnostic test, formatting, CGG and diff evidence; Rust 1.99.0 and production WLD-1 remain open. |
| SRE Lead | No real Host provider, drain, cleanup or recovery acceptance in this slice. |
| Platform | Disposable PostgreSQL only; target database and runtime application role remain unverified. |
| Review facilitator | Source and migration review recorded; concurrency and deployment evidence remain required. |
| PM | This is an owner-fact persistence slice, not a user-ready Worktree lifecycle closure. |

## §7 Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added Worktree archive/restore Outbox source slice, no-op semantics, idempotency serialization, disposable PostgreSQL evidence and open gates. | WLD-1 bounded implementation and coordinator review |
