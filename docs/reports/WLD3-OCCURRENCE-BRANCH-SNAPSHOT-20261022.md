# WLD3 Occurrence Branch Snapshot — Implementation Report

**Date:** 2026-10-22
**Lane:** `wbs-wld3-occurrence-branch-20261022`
**Baseline:** `adacb3d34217c36f6563c9c36ca77bc5841a490f`
**Acceptance retry base:** `df1c22f66d8d0c3b48baac7e348494ba3466569b` (worktree HEAD before this retry commit)
**Status:** Disposable PostgreSQL migration/assertion checks passed; pinned Rust gates are partial. WLD3 remains open and Schedule execution stays disabled.

## §0 Purpose

Close the persistence gap between hostless Schedule Rule configuration and occurrence materialization: a Rule revision may omit remote Branch context, but every newly inserted occurrence snapshot must carry a valid, non-nil Branch UUID at `rule_snapshot.target.branch_id`. This slice does not add a worker, materializer, admission consumer, or executor.

## §1 Change matrix

| Area | Change | State |
|---|---|---|
| Database contract | Added the conditional occurrence CHECK requiring a valid, non-nil Branch UUID at rule_snapshot.target.branch_id; the isolated PostgreSQL 18.6 run applied it successfully. | Disposable PostgreSQL: passed |
| Upgrade behavior | The CHECK was installed as NOT VALID; first and repeat migration passes both succeeded without backfilling historical rows. | Disposable PostgreSQL: passed |
| Branchless Rule behavior | The full branchless Rule revision insert, including immutable run-as identity trigger behavior, succeeded. | Disposable PostgreSQL: passed |
| Regression harness | Lowercase/uppercase valid UUID cases and missing/null/empty/malformed/nil rejection cases executed against PostgreSQL. | Disposable PostgreSQL: passed |
| Existing PG fixtures | The schedule_postgres integration test target compiled with Rust 1.99.0; no DB-backed adapter test was run in this retry. | Compile: passed; execution: not run |
| Domain contract | `AutomationOccurrenceSnapshotV1::validate` already requires `branch_id.is_some()` and the target validator rejects nil UUID. No duplicate Domain check was added. | Audited; unchanged |

Changed implementation/test files: `db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql`, `scripts/automation/phase9f3_schedule.py`, `scripts/automation/test_schedule_occurrence_branch_snapshot.py`, and `crates/star-pg-adapter/tests/schedule_postgres.rs`.

## §2 Verification summary

| Command/check | Result |
|---|---|
| rtk proxy python -m unittest scripts.automation.test_schedule_occurrence_branch_snapshot -v | Passed: 3/3 static contract tests. |
| rtk proxy python -m py_compile scripts/automation/phase9f3_schedule.py scripts/automation/test_schedule_occurrence_branch_snapshot.py | Passed. |
| rtk proxy git diff --check | Passed after the report update. |
| CypherGraph Guardian, scoped to the two changed Python files | Passed; exact command and source checks recorded in the retry log. |
| Disposable PostgreSQL 18.6 acceptance | 29/30 runner steps passed, including first/repeat migrations, nullable branchless Rule persistence, occurrence UUID positive/negative assertions, FORCE RLS catalog and runtime/ACL fixtures. The sole failed step was the embedded Rust ACL test (exit 101); the disposable cluster was stopped and its data directory removed. System PostgreSQL on port 5432 was not touched. |
| Fresh Rust 1.99.0 cargo test --locked -p star-pg-adapter --test schedule_postgres --no-run -j 4 | Passed (exit 0); produced the schedule_postgres test executable in the isolated target. |
| Fresh Rust 1.99.0 cargo test --locked -p domain-automation --lib -j 4 | Passed: 30/30 tests (exit 0). |
| Fresh Rust 1.99.0 focused run-as API test | Blocked before test execution by E0631 at crates/star-api-rest/src/group_api/worktree_lifecycle.rs:815 (Option<Uuid>::is_some_and(Uuid::is_nil) signature mismatch); exit 101. This file is outside the lane manifest. |
| Fresh Rust 1.99.0 domain Clippy | Failed with clippy::unnecessary_map_or at crates/domain-automation/src/lib.rs:1040; exit 101. This file is outside the lane manifest. |
| Rustfmt checks from the runner | The broad Schedule source check and API check exit 1 on existing formatting diffs in domain/API/repository files; no autoformatting was applied. The schedule_postgres.rs fixture was not among the reported diff files. |
| Prior embedded PostgreSQL Rust test (mixed target/toolchain) | E0514/exit 101 was caused by reusing artifacts from Rust 1.99.0 under rustc 1.99.1. The fresh 1.99.0 isolated target did not reproduce E0514; its focused API test instead reached the separate E0631 baseline compile error. |
| CodeGraph evidence | Coverage unverified in this Worktree; exact source inspection used. |
| CodeRabbit review | Unavailable: the existing WSL shim returned permission denied; manual scoped review continued. |

## §3 Known gaps

1. The run-as authorization Rust test is still blocked before its body by the out-of-scope E0631 compile error at worktree_lifecycle.rs:815; this lane does not edit that source.
2. domain-automation Clippy remains blocked by unnecessary_map_or at domain-automation/src/lib.rs:1040; that file is outside this lane.
3. The runner's broad rustfmt checks report existing formatting diffs in domain-automation, star-api-rest/src/group_api/schedule_rules.rs, and star-pg-adapter/src/repository; no unrelated formatting was applied.
4. The combined runner recorded 29 passing PostgreSQL steps and one failed embedded Rust step. Its E0514 target/toolchain mismatch was not reproduced after isolating the target and pinning Rust 1.99.0, but the embedded authorization test remains unexecuted because compilation now stops at E0631.
5. The disposable database proves local migration and assertion behavior only. Migration ownership, deployed grants/RLS, deployment ordering, and rollback remain release gates.
6. Existing occurrence snapshots without Branch remain under NOT VALID and are not backfilled with fabricated identities; they cannot be treated as valid execution snapshots.
7. Schedule worker/materializer, occurrence admission, infrastructure reservation/readiness, and execution remain outside this slice and stay disabled.

## §4 Subagent failure / takeover

No child agents were dispatched for this bounded lane. No delegated output required takeover.

## §5 Guardrails

| # | Guardrail | Evidence/state |
|---|---|---|
| 1 | Rule configuration may omit remote Branch context. | Branchless Rule row persisted successfully in the isolated PostgreSQL run. |
| 2 | Occurrence snapshot must include rule_snapshot.target.branch_id. | Migration CHECK and valid/invalid SQL assertions passed against disposable PostgreSQL. |
| 3 | UUID spelling accepts uppercase and lowercase hex in standard hyphenated form. | Both valid forms passed in PostgreSQL. |
| 4 | Missing key and JSON null fail closed. | Both rejection cases passed in PostgreSQL. |
| 5 | Empty and malformed UUID strings fail closed. | Both rejection cases passed in PostgreSQL. |
| 6 | Nil UUID fails closed, matching the existing Domain target validator. | SQL rejection case passed in PostgreSQL; Domain unit suite passed 30/30. |
| 7 | Invalid text is not cast before UUID shape validation. | Malformed-input rejection passed without a cast error escaping the CHECK. |
| 8 | Existing historical snapshots are not rewritten or assigned synthetic Branch IDs. | `NOT VALID`; no backfill/update statement. |
| 9 | Future occurrence inserts and updates are constrained. | Constraint behavior exercised by the disposable positive/negative SQL assertions. |
| 10 | Migration reruns do not add duplicate constraints. | First and repeat migration passes succeeded. |
| 11 | The guard is scoped to `automation.occurrence`, not Rule persistence. | Migration only alters occurrence; Rule storage assertion is in the disposable harness. |
| 12 | Occurrence target snapshot remains required and bounded by the base schema. | Existing `NOT NULL`, object, and size checks are unchanged. |
| 13 | Existing PG integration fixtures carry the required Branch snapshot. | The schedule_postgres target compiled under fresh Rust 1.99.0. |
| 14 | Existing Domain checks remain the application-level validation authority. | Domain validator inspected and unchanged. |
| 15 | This migration does not enable Schedule execution. | No worker/materializer/executor code or runtime flag added. |
| 16 | Validation uses only a disposable test cluster; never the target database. | Isolated random loopback port 39781; cluster stopped and removed; port 5432 untouched. |
| 17 | No changes outside the assigned migration, narrow harness/fixture, and report scope. | Retry kept changes within the manifest; existing debug scripts, logs, ignored cache, and other lane evidence were preserved. |

## §6 Sign-off

| Role | State |
|---|---|
| Architecture | Scoped to the Branch-required occurrence boundary while retaining branchless Rule configuration; coordinator review pending |
| SRE Lead | Disposable PostgreSQL 18.6 migration/assertion run passed; deployment-database gates remain open. |
| Platform | Pinned Rust 1.99.0 checks are partial: domain tests and integration-target compile pass; run-as API and Clippy fail on documented out-of-scope baseline errors. |
| Review host | Static tests, Python syntax, diff whitespace, and Cypher checks passed; CodeRabbit unavailable as recorded above |
| PM | Not production-ready; execution remains disabled |

## §7 Revision history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-22 | Add an idempotent occurrence snapshot Branch CHECK, disposable DB positive/negative cases, branchless Rule persistence assertion, and truthful verification limits. | `docs/briefs/WBS-WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022.md` |
| v0.2 | 2026-10-22 | Record 29/30 disposable PostgreSQL runner steps, fresh-target Rust 1.99.0 outcomes, E0514 non-reproduction and remaining E0631/Clippy/rustfmt blockers; retain disabled execution. | docs/briefs/WLD3-PG-ACCEPTANCE-RETRY-20261022.md |
