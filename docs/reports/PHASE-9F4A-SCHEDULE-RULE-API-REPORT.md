# PHASE-9F4A-SCHEDULE-RULE-API-REPORT

> Status: 🟡 IN PROGRESS — implementation files are present, but this phase is not closed.
> Date: 2026-10-04 | Revision: v0.4

## §0 Purpose

Add an authenticated, Run-scoped Schedule Rule management surface and atomic database write boundary without enabling scheduled execution before the later 9F4 admission/worker gates are complete.

## §1 Change matrix

| Area | Current change | State |
|---|---|---|
| REST API | `schedule_rules` routes for bounded list/read/create/CAS revision; Run, Project, Rule revision, target, Profile, Catalog, and HookSet are resolved server-side with future-dated facts excluded. | Focused all-targets compile and three unit tests pass; route integration tests remain open. |
| Storage | Additive migration adds display name, append-only Run-consistent rule Outbox, 24-hour idempotency Work table, and tenant FORCE RLS. | Disposable PostgreSQL 18.6 applied 9F2/9F4A twice and exercised the new constraints/RLS. |
| Docs | SRS/BD/DD API addenda are cross-linked from requirements v5.58, basic design v5.55, Group DD v4.37, Data Design v0.8, and implementation plan v5.80. | Synced with reviewed code and targeted verification; production gates remain explicit. |
| Worker/BI | No clock worker, occurrence producer, TaskExecutionRun admission, reservation, RunEvent consumer, or BI projection is enabled. | Open by design for later 9F4 stages. |

## §2 Verification summary

The `rtk` launcher remains unavailable, so the cached Rust 1.98.1 toolchain was invoked directly with an isolated target directory. `rustfmt --edition 2021 --check` passed on the four owned Rust files, and `cargo --offline check --locked -p star-api-rest --all-targets -j 4` exited 0. The compiled `star_api_rest` test binary then ran the three `group_api::schedule_rules` tests directly: 3 passed, 0 failed. Existing warnings in unrelated dependency crates were emitted; no focused Rust error occurred.

An independently created disposable PostgreSQL 18.6 container applied the 9F2 and 9F4A migrations twice to a fresh database. A non-superuser role then verified tenant isolation; the verification SQL rejected a Run-mismatched Outbox insert through the five-column foreign key, rejected Outbox mutation, reused an expired matching idempotency key after 65 unrelated expired records, and asserted equality of the SCD2 close/successor boundary. The database was not a target environment and no runtime service grants were claimed.

`cgg_guard.py --stamp` passed for the four owned Rust sources. A whole-tree changed-file Guardian scan remains blocked by legacy non-`@cypher` manifests in `execution_profile_admin.rs` and `execution_profiles.rs`, which are outside this 9F4A ownership; this report claims only the targeted pass.

## §3 Known gaps

1. Route integration tests for authorization, target binding/currentness, replay, concurrent CAS, request bounds, and cross-scope error headers are not yet added/run.
2. API-role database grants and target production DB migration are not installed or verified.
3. The local migration check validates the two 9F4A tables and their dependencies, not deployment against the target database or the complete seven-table Schedule contract.
4. Outbox delivery state/consumer, long-running clock/worker, occurrence-to-Run admission, resource reservation, BI/Benchmark joins, and global fairness remain later Phase 9F work.
5. `rtk` remains unavailable; repeatable direct-toolchain evidence is recorded here, and all changed Rust files must be rechecked before subsequent integration.
6. The unrelated legacy Cypher headers named above need their own owner to complete a whole-tree Guardian pass.

## §4 Sub-agent failure/ownership record

The worker first performed a read-only independent review of the current API/migration diff, then received a scoped writable brief for the same lane. It found and repaired the missing Outbox `engineering_run_id`, SCD2 timestamp skew, expired idempotency-key reuse, future-dated current-fact, cache-header, and Run-FK issues. The sole writable lane is `codex/schedule-production-9f4-20261003`.

## §5 Guard gates

1. Project→Run authorization is checked on every request.
2. Write roles are restricted to tenant/project administrators and developers.
3. Target Worktree/Task ownership is locked and checked against the route Run.
4. Current execution Profile and effective HookSet are resolved by server-owned code.
5. Parser and timezone identities are pinned by the server.
6. Recurrence materialization remains bounded and disabled rules remain non-executable.
7. SCD2 only closes the prior revision and appends a successor.
8. Audit and Outbox are in the same transaction as the rule revision.
9. Outbox rows are append-only and tenant FORCE RLS protected.
10. Idempotency records store only key/request hashes and expire after 24 hours.
11. The matching expired key is removed before reuse and unrelated TTL cleanup is capped at 64 rows.
12. List size is capped at 100 and request body at 16 KiB.
13. Authenticated responses are not cacheable.
14. No Rule API path directly invokes an agent or shell.
15. No worker/TaskExecutionRun producer is enabled by this phase.
16. Phase closure requires route integration and target-environment verification beyond the completed focused Rust and isolated PostgreSQL checks.

## §6 Sign-off

| Role | Review state |
|---|---|
| Architecture | Mavis reviewed focused diff and verification; phase closure open |
| SRE Lead | Disposable migration evidence recorded; target environment open |
| Platform | API/database grant review open |
| Review facilitator | Route integration review open |
| Product | Schedule execution/BI scope remains open |

## §7 Revision history

| Version | Date | Editor | Change |
|---|---|---|---|
| v0.1 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record 9F4A API/migration/design slice and explicitly retain all unavailable verification gates. |
| v0.2 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Synchronize parent requirements/basic/detailed/data design references and clarify that only documentation sync is complete while Rust/API/PostgreSQL verification remains open. |
| v0.3 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Repair independent-review findings; record focused Rust and isolated PostgreSQL evidence; retain route, target DB/grant, worker, admission, and BI closure gates. |
| v0.4 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Re-run direct all-targets/test-binary and fresh PostgreSQL evidence; make Rule-revision currentness and reviewer-to-writer handoff explicit. |
