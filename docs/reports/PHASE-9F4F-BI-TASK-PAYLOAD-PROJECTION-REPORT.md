# Phase 9F4F — BI Task Payload Projection Report

> **Version**: v0.1
> **Status**: 🟡 Isolated implementation and database fixture complete; production activation gates remain open
> **Date**: 2026-10-16
> **Reviewer**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **Scope**: BI owner fact data minimization; no transport, analytics query API, or execution enablement

## §0 Purpose

Phase 9F4F removes free-form Task payload duplication from the live BI fact table. The BI owner database creates a strict allowlist projection from a bounded source payload and stores it in the same transaction as the durable Inbox receipt and fact. Historical live facts are converted through the same projector in an atomic migration. This report records the implementation boundary and only the disposable PostgreSQL evidence actually obtained.

## §1 Change Matrix

| Area | Change | Evidence |
|---|---|---|
| Rust BI owner | Calls the fixed v2 database entry point and rejects lifecycle/review values outside the explicit allowlists before opening a transaction | Rust 1.99.0 rustfmt, package all-target tests (4/4) and strict all-target Clippy passed |
| BI schema | Adds a database projector, v2 commit wrapper, least-privilege grants, and a transactional replacement-table backfill that drops the live raw-payload table | Migration source and disposable runner |
| Database tests | Covers legacy raw sentinel removal, projection field retention, exact replay, missing lifecycle status rejection, runtime grants, FORCE RLS and WORM | PostgreSQL 18.6 disposable loopback run |
| Automation | Extends the isolated runner for both base and projection migrations, repeated application, legacy data seeding and privacy checks | Runner exit code 0 |
| Documents | Synchronizes total requirements, Task SRS, basic/detail design, Group detail design, data design, implementation plan, automation design and registry | Cross-reference and stale-status audit |

No navigation node, route, Task executor, receiver, dispatcher or analytics API was added. BI remains a Run App; execution remains fail closed.

## §2 Validation Summary

| Gate | Result | Boundary |
|---|---|---|
| PostgreSQL 18.6 disposable runner | PASS, exit 0: “9F4E/9F4F BI Task Inbox and safe projection PostgreSQL gates passed” | Isolated loopback fixture only; not the target database |
| Base migration repeatability | PASS, applied twice before seeding | Does not validate production migration orchestration |
| Historical projection migration | PASS, applied repeatedly; legacy sentinel absent from the live fact | Existing WAL, replicas and backups were not inspected or purged |
| New v2 write/replay and malformed-state rejection | PASS, stable receipt replayed; missing lifecycle status rejected | No network receiver or trusted production identity exercised |
| Runtime privilege/RLS/WORM checks | PASS in disposable fixture | Target role provisioning and grants remain unverified |
| Rust 1.99.0 focused crate gates | PASS: rustfmt `--check`; `cargo test -p bi-owner-runtime --all-targets -j 4 --offline` passed 4/4; strict `cargo clippy --all-targets -D warnings` passed | Focused package only; no workspace-wide result is claimed |
| Workspace-wide Rust 1.99.0 gates | NOT RUN | This phase validated only `bi-owner-runtime`; unrelated workspace checks remain open |

The runner stops and removes only its uniquely owned temporary PostgreSQL data directory after confirming the process stopped.

## §3 Known Gaps

1. The crate-private authenticated principal still has no trusted transport factory; there is no production receiver or network dispatcher.
2. No source-side accepted settlement verifies the receiver's durable receipt. No fan-out, per-consumer projection store or Run-fair scheduler is enabled.
3. There is no BI/Benchmark query API, UI or materialized analytics view. The stored allowlist projection is a safe event source, not an analytics product.
4. Target database deployment, production role provisioning, grants, RLS and migration rollback have not been verified.
5. The migration removes raw payload from the current fact table only. Existing WAL, replicas, PITR archives and backups need a separate retention/restore policy. Task source Outbox retention remains owned by its source service.
6. Concurrent duplicate delivery, crash recovery, restore/replay, load/RSS and fairness have not been tested.
7. Focused `bi-owner-runtime` Rust 1.99.0 format/test/strict-Clippy gates passed; workspace-wide Rust checks remain open. No production receiver or Task execution capability is enabled.

## §4 Subagent Failure Handoff

No subagents were dispatched for this slice. There are no failed or uncollected subagent outputs.

## §5 Guard Rules

1. Raw input is bounded before database projection; only the owner database function defines persisted BI fields.
2. Metadata projection retains approved IDs, versions, event type and owner-change state only.
3. Lifecycle projection retains approved IDs, versions, event type, lifecycle/review states and optional transition states only.
4. Unknown schema/event/scope or unknown or missing lifecycle/review values fail closed.
5. Runtime executes v2 only; raw v1 persistence and the helper are not granted to runtime.
6. Inbox receipt and fact commit remain one BI-owner transaction; exact replay returns the stable receipt.
7. Legacy projection failure rolls back the complete replacement migration, leaving the original live table intact.
8. The three BI tables remain Transaction tables with tenant FORCE RLS and append-only guards.
9. Dropping the live raw table does not claim erasure from WAL, replicas, backups or source Outbox storage.
10. The disposable fixture is not production target DB, trusted identity, consumer fan-out, BI query API, or execution proof.
11. Project → Engineering Run → Worktree → Branch navigation and BI Run App placement are unchanged.
12. Keep receiver, BI ingestion, accepted delivery and Task execution disabled until their independent production gates pass.

## §6 Sign-off

| Role | Review |
|---|---|
| Architect | Mavis review: database-owned allowlist projection and data-owner boundary accepted for this implementation slice |
| SRE Lead | Mavis review: disposable migration gate accepted; backup/WAL/PITR and recovery remain open |
| Platform | Mavis review: fixed-function runtime grant boundary accepted in fixture; target deployment remains open |
| Review Host | Mavis review: changes stay within the existing Run BI app and do not alter navigation |
| PM | Mavis review: phase status is limited to the verified schema slice; production readiness is not claimed |

## §7 Revision History

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record DB-generated BI projection, atomic legacy conversion, PostgreSQL fixture evidence and remaining production gates | 9F4F implementation and requirements/basic/detail design reconciliation |
