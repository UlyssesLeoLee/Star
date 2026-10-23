# WLD3 Occurrence Branch Snapshot — Implementation Report

**Date:** 2026-10-22
**Lane:** `wbs-wld3-occurrence-branch-20261022`
**Baseline:** `adacb3d34217c36f6563c9c36ca77bc5841a490f`
**Acceptance retry base:** `df1c22f66d8d0c3b48baac7e348494ba3466569b` (worktree HEAD before this retry commit)
**Status:** The historical 2026-10-22 migration remains unchanged; the 2026-10-23 forward correction now targets flattened `target_snapshot.branch_id`. Isolated local PostgreSQL 18.6 acceptance passed 35/35 steps, and the focused Rust/API gates recorded below passed. WLD3 remains open because worker/owner Saga, target-environment deployment and Runtime readiness are not covered; Schedule execution stays disabled.

## §0 Purpose

Close the persistence gap between hostless Schedule Rule configuration and occurrence materialization: a Rule revision may omit remote Branch context, but every newly inserted occurrence must carry a valid, non-nil Branch UUID at top-level `target_snapshot.branch_id`. The producer serializes `ScheduleTargetV1` directly, so its object is flattened. The committed 2026-10-22 migration remains unchanged as historical state; the 2026-10-23 correction replaces only its named CHECK in a transaction. This slice does not add a worker, materializer, admission consumer, or executor.

## §1 Change matrix

| Area | Change | State |
|---|---|---|
| Historical migration | The committed 2026-10-22 CHECK targets the nested `rule_snapshot.target.branch_id` path. It is retained unchanged because deployed migration trackers may already record it. | Historical source retained; not the producer contract |
| Forward correction | Added the 2026-10-23 migration to drop only the exact named CHECK and re-add it for flattened top-level `branch_id`, as `NOT VALID`, with no backfill. | PostgreSQL 18.6: passed |
| Upgrade behavior | Harness orders the historical migration before the correction and runs the sequence on both first and repeat passes. | First and repeat migration passes: passed |
| Branchless Rule behavior | The full branchless Rule revision insert, including immutable run-as identity trigger behavior, succeeded. | Disposable PostgreSQL: passed |
| Regression harness | Lowercase/uppercase flat UUID cases and missing/null/empty/malformed/nil rejection cases are present after correction. | Static and PostgreSQL assertions: passed |
| Existing PG fixtures | Schedule PostgreSQL integration suite ran against the isolated database; all 5 tests passed. | 5/5 passed |
| Schedule API authorization tests | Request cannot select another run-as identity; task execution roles do not grant Schedule authority. | 2 focused unit tests passed, 1/1 each |
| Adapter compile | `star-pg-adapter --all-targets` checked under Rust 1.99.0. | Passed, exit 0 |
| Domain contract | `AutomationOccurrenceSnapshotV1::validate` already requires `branch_id.is_some()` and the target validator rejects nil UUID. No duplicate Domain check was added. | Audited; unchanged |

Correction files: `db/migrations/2026-10-23-schedule-occurrence-branch-snapshot-correction.sql`, `scripts/automation/phase9f3_schedule.py`, `scripts/automation/test_schedule_occurrence_branch_snapshot.py`, and `crates/star-pg-adapter/tests/schedule_postgres.rs`. The committed 2026-10-22 migration is unchanged.

## §2 Verification summary

| Command/check | Result |
|---|---|
| rtk proxy python -m unittest scripts.automation.test_schedule_occurrence_branch_snapshot -v | Passed: 3/3 static contract tests. |
| rtk proxy python -m py_compile scripts/automation/phase9f3_schedule.py scripts/automation/test_schedule_occurrence_branch_snapshot.py | Passed. |
| rtk proxy git diff --check | Passed after the report update. |
| CypherGraph Guardian, scoped to the two changed Python files | Passed; exact command and source checks recorded in the retry log. |
| Historical disposable PostgreSQL 18.6 run | 29/30 steps passed against the then-current nested-path CHECK. This was superseded by the corrected 35/35 acceptance run below. Its only failure was the embedded Rust ACL test (exit 101); the disposable cluster was stopped and its data directory removed. |
| Fresh Rust 1.99.0 cargo test --locked -p star-pg-adapter --test schedule_postgres --no-run -j 4 | Passed (exit 0); produced the schedule_postgres test executable in the isolated target. |
| Fresh Rust 1.99.0 cargo test --locked -p domain-automation --lib -j 4 | Passed: 30/30 tests (exit 0). |
| Earlier Rust 1.99.0 focused API attempt | One prior test invocation stopped at an unrelated E0631 compile error; this does not describe the latest narrow Schedule API tests, which passed below. |
| Fresh Rust 1.99.0 domain Clippy | Failed with clippy::unnecessary_map_or at crates/domain-automation/src/lib.rs:1040; exit 101. This file is outside the lane manifest. |
| Rustfmt checks from the runner | The broad Schedule source check and API check exit 1 on existing formatting diffs in domain/API/repository files; no autoformatting was applied. The schedule_postgres.rs fixture was not among the reported diff files. |
| Prior embedded PostgreSQL Rust test (mixed target/toolchain) | E0514/exit 101 was caused by reusing artifacts from Rust 1.99.0 under rustc 1.99.1. The fresh 1.99.0 isolated target did not reproduce E0514; its focused API test instead reached the separate E0631 baseline compile error. |
| CodeGraph evidence | Coverage unverified in this Worktree; exact source inspection used. |
| CodeRabbit review | Unavailable: the existing WSL shim returned permission denied; manual scoped review continued. |

### Current correction follow-up (2026-10-23)

| Command/check | Result |
|---|---|
| `rtk proxy python -m unittest scripts.automation.test_schedule_occurrence_branch_snapshot -v` | Passed: 3/3 static tests; they now assert the immutable historical migration, forward migration order, flat UUID contract, and replay loop. |
| `rtk proxy python -m py_compile scripts/automation/phase9f3_schedule.py scripts/automation/test_schedule_occurrence_branch_snapshot.py` | Passed. |
| Rust 1.99.0 `cargo test -p star-pg-adapter --test schedule_postgres --no-run -j 4` with lane-local target | Passed: integration test executable compiled; no test was run and no database was contacted. |
| CypherGraph Guardian, scoped to the four implementation files | Passed for 3 source files; SQL digest was refreshed through Guardian's `source_digest` helper because `.sql` is outside its source extension set. The repository-wide changed scan also reports preserved untracked `.cache` helper scripts without headers; those artifacts were not changed. |
| `cargo fmt --all -- --check` and targeted `rustfmt --check` | Workspace-wide `cargo fmt --all -- --check` remains blocked by pre-existing formatting differences outside this lane. Targeted `rustfmt --edition 2021 --check crates/star-pg-adapter/tests/schedule_postgres.rs` passes after formatting; format-only changes beyond the two flat fixture updates were reviewed. |
| PostgreSQL correction acceptance | Superseded by the latest isolated PostgreSQL 18.6 run: 35/35 steps passed, including old → correction → repeat migrations, flat valid/invalid/nil assertions, ACL/RLS, and 5/5 Schedule integration tests. |
| Latest Schedule API authorization unit tests | Rust 1.99.0: cargo test --locked -p star-api-rest --lib schedule_rule_request_cannot_select_run_as_actor -j 1 and cargo test --locked -p star-api-rest --lib task_execution_rules_do_not_grant_agent_role_schedule_authority -j 1; both passed 1/1. |
| Latest star-pg-adapter all-targets check | rtk proxy E:\DevCache\cargo\bin\cargo.exe check --locked -p star-pg-adapter --all-targets -j 4: passed, exit 0. |
| Full phase CLI entrypoint | The pinned phase9f3_schedule.py command failed fast before database work at schedule-rustfmt due existing diffs in crates/domain-automation/src/lib.rs and crates/domain-automation/src/schedule.rs. The isolated checked-in run_isolated_postgres helper was then run directly; this is not reported as a full CLI pass. |
| Latest isolated PostgreSQL result | 35/35 steps passed. PostgreSQL 18.6 used 127.0.0.1:58997 and UUID data directory .cache/phase9f3-schedule/postgres-ae840a34-d899-4f5e-b638-8ec3b12c41b4; postgres-stop exit 0 and the owned directory was removed. Result: .cache/phase9f3-schedule/wld3-postgres-isolated-20261023.log; full-entrypoint log: .cache/phase9f3-schedule/wld3-pg-acceptance-20261023.log; raw step stdout/stderr: .cache/phase9f3-schedule/postgres-*.stdout.log / *.stderr.log. |

## §3 Known gaps

1. The checked-in full phase entrypoint currently fails fast at broad rustfmt checks on unrelated baseline files; the direct isolated PostgreSQL helper passed, but not the entire entrypoint. The lane-owned Rust test file passes targeted rustfmt.
2. This database was disposable and local. Production target connection, migration ownership/permissions, deployed grants and RLS, deployment ordering, and rollback remain open.
3. Schedule worker/materializer, admission consumer, Run/Execution-owner to Infrastructure-owner Saga, durable Outbox/Inbox recovery, and execution remain outside this slice and stay disabled.
4. Target-environment RunInfrastructureBinding, host/cluster reservations, K3s API/node/CRI readiness, independent Agent Runtime sandbox, cancellation, restart, and recovery have not been validated.
5. Existing occurrence snapshots without Branch remain under NOT VALID and are not backfilled with fabricated identities; they cannot be treated as valid execution snapshots.
6. The 2026-10-22 migration is immutable history and may be skipped by schema_migrations; only the 2026-10-23 forward correction repairs an already-recorded old constraint. Deployment must verify the exact constraint name/definition and stop if it is unexpected rather than dropping a broader object.
7. The older 29/30 result and prior SQLSTATE 23514 fixture failure are retained as historical diagnostics; the latest corrected local run passed 35/35. Schedule execution remains fail-closed.

## §4 Subagent failure / takeover

No child agents were dispatched for this bounded lane. No delegated output required takeover.

## §5 Guardrails

| # | Guardrail | Evidence/state |
|---|---|---|
| 1 | Rule configuration may omit remote Branch context. | Branchless Rule row persisted successfully in the isolated PostgreSQL run. |
| 2 | Occurrence snapshot must include flattened target_snapshot.branch_id. | Forward correction and actual PostgreSQL insert assertions passed. |
| 3 | UUID spelling accepts uppercase and lowercase hex in standard hyphenated form. | Both flat lowercase and uppercase forms passed PostgreSQL assertions. |
| 4 | Missing key and JSON null fail closed. | Both flat invalid cases were rejected. |
| 5 | Empty and malformed UUID strings fail closed. | Both flat invalid cases were rejected. |
| 6 | Nil UUID fails closed, matching the existing Domain target validator. | Nil UUID was rejected by PostgreSQL; Domain validator is unchanged. |
| 7 | Invalid text is not cast before UUID shape validation. | Malformed flat text was rejected by the SQL assertion without relaxing the constraint. |
| 8 | Existing historical snapshots are not rewritten or assigned synthetic Branch IDs. | Both migrations are NOT VALID; no backfill was added. |
| 9 | Future occurrence inserts and updates are constrained. | Corrected flat CHECK passed positive and negative SQL cases and the 5-test integration suite. |
| 10 | Historical migration remains unchanged; forward correction is ordered after it and safely re-applies its exact DROP/ADD transaction. | Historical → correction and repeat replay both passed. |
| 11 | The guard is scoped to `automation.occurrence`, not Rule persistence. | Migration only alters occurrence; Rule storage assertion is in the disposable harness. |
| 12 | Occurrence target snapshot remains required and bounded by the base schema. | Existing `NOT NULL`, object, and size checks are unchanged. |
| 13 | Existing PG integration fixtures carry the required Branch snapshot. | The schedule_postgres target compiled under fresh Rust 1.99.0. |
| 14 | Existing Domain checks remain the application-level validation authority. | Domain validator inspected and unchanged. |
| 15 | This migration does not enable Schedule execution. | No worker/materializer/executor code or runtime flag added. |
| 16 | Validation uses only a disposable test cluster; never the target database. | Latest run used UUID data directory and loopback port 58997; stop exited 0 and the owned directory was deleted. Port 5432 was not touched; protected port 5555/PID 7460 was not used or modified. |
| 17 | No changes outside the assigned migration, narrow harness/fixture, and lane report/WBS scope authorized by the coordinator. | Implementation stayed within the migration and narrow harness/fixture; only lane-owned report/WBS files were updated. Existing debug scripts, logs, ignored cache, and other lane evidence were preserved and are excluded from the commit. |

## §6 Sign-off

| Role | State |
|---|---|
| Architecture | Scoped to the Branch-required occurrence boundary while retaining branchless Rule configuration; coordinator review pending |
| SRE Lead | Local isolated PostgreSQL 18.6 acceptance passed 35/35, including stop and owned-directory cleanup; production target/deployment gates remain open. |
| Platform | Rust 1.99.0 adapter all-targets check, 5/5 Schedule PostgreSQL integration tests, and two focused API authorization tests passed; full entrypoint rustfmt preflight still fails on unrelated baseline files. |
| Review host | Static tests, Python syntax, diff whitespace, and Cypher checks passed; CodeRabbit unavailable as recorded above |
| PM | Not production-ready; execution remains disabled |

## §7 Revision history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-22 | Add an idempotent occurrence snapshot Branch CHECK, disposable DB positive/negative cases, branchless Rule persistence assertion, and truthful verification limits. | `docs/briefs/WBS-WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022.md` |
| v0.2 | 2026-10-22 | Record 29/30 disposable PostgreSQL runner steps, fresh-target Rust 1.99.0 outcomes, E0514 non-reproduction and remaining E0631/Clippy/rustfmt blockers; retain disabled execution. | docs/briefs/WLD3-PG-ACCEPTANCE-RETRY-20261022.md |
| v0.3 | 2026-10-23 | Preserve the committed nested-path migration as history; add a forward correction for flat `target_snapshot.branch_id`, update harness ordering and static assertions, and mark correction PostgreSQL acceptance pending. | `docs/briefs/WBS-WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022.md`; integration review of commit `b702fddd` |
| v0.4 | 2026-10-23 | Record isolated PostgreSQL 18.6 acceptance 35/35, repeat correction migration, flat UUID SQL cases, ACL/RLS, 5/5 adapter integration tests, two Schedule API authorization tests, cleanup, and remaining WLD3 production/worker/Saga gates. | WLD3 local PostgreSQL acceptance and focused Rust 1.99.0 gates |
