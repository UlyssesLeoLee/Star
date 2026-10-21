# WLD3 Occurrence Branch Snapshot — Implementation Report

**Date:** 2026-10-22
**Lane:** `wbs-wld3-occurrence-branch-20261022`
**Baseline:** `adacb3d34217c36f6563c9c36ca77bc5841a490f`
**Status:** Forward migration and regression gates authored; Rust and PostgreSQL acceptance remain open.

## §0 Purpose

Close the persistence gap between hostless Schedule Rule configuration and occurrence materialization: a Rule revision may omit remote Branch context, but every newly inserted occurrence snapshot must carry a valid, non-nil Branch UUID at `rule_snapshot.target.branch_id`. This slice does not add a worker, materializer, admission consumer, or executor.

## §1 Change matrix

| Area | Change | State |
|---|---|---|
| Database contract | Added `2026-10-22-schedule-occurrence-branch-snapshot.sql`. Its conditional constraint checks the nested JSON path, requires a string in standard hyphenated UUID form using a case-insensitive regex, then rejects the nil UUID. The UUID cast is inside a `CASE` arm reached only after the shape check. | Authored; not applied to PostgreSQL |
| Upgrade behavior | Adds the CHECK as `NOT VALID` inside a transaction, preserving pre-existing occurrence rows that cannot be safely backfilled. PostgreSQL enforces it for new or changed rows. A `pg_constraint` guard and repeat-run harness make the migration forward-only and idempotent. | Static contract checked; database replay not run |
| Branchless Rule behavior | The isolated PostgreSQL harness verifies `schedule_rule_revision.branch_id` remains nullable and inserts a full copied Rule row with `branch_id = NULL`; it aligns `changed_by` with the immutable `run_as_actor_id` so the initial-revision identity trigger is exercised correctly. | Harness authored; not run |
| Regression harness | Added valid lowercase and uppercase UUID inserts plus rejection cases for missing, JSON null, empty, malformed, and nil UUID values. `phase9f3_schedule.py` applies the migration in both its first and repeat passes before running this SQL assertion. | Harness authored; not run |
| Existing PG fixtures | Updated occurrence snapshots in `schedule_postgres.rs` so the existing durable integration cases seed the required nested Branch value. | Source updated; Rust/DB test not run |
| Domain contract | `AutomationOccurrenceSnapshotV1::validate` already requires `branch_id.is_some()` and the target validator rejects nil UUID. No duplicate Domain check was added. | Audited; unchanged |

Changed implementation/test files: `db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql`, `scripts/automation/phase9f3_schedule.py`, `scripts/automation/test_schedule_occurrence_branch_snapshot.py`, and `crates/star-pg-adapter/tests/schedule_postgres.rs`.

## §2 Verification summary

| Command/check | Result |
|---|---|
| `rtk run python scripts/automation/test_schedule_occurrence_branch_snapshot.py` | Passed: 3 static contract tests |
| `rtk run python -m py_compile scripts/automation/phase9f3_schedule.py scripts/automation/test_schedule_occurrence_branch_snapshot.py` | Passed |
| `rtk git diff --check` | Passed |
| CypherGraph Guardian `--changed` | Passed: 3 source files checked; the new SQL header freshness stamp was verified separately with the Guardian’s `source_digest` implementation because SQL is not included in its changed-source count |
| CodeRabbit review | Not available: the installed Windows shim launches `/root/.local/bin/coderabbit` in WSL, which returned `Permission denied`; manual diff review continued |
| `cargo fmt`, Rust compile/tests, and Clippy | Not run: `cargo` and `rustfmt` are unavailable on PATH |
| Migration replay and SQL positive/negative insert assertions | Not run: `psql` and `initdb` are unavailable; no target database was touched |

No indexed `.codegraph/` project is available in this Worktree. Impact review used targeted searches and exact inspection of the Domain validator, migration chain, PostgreSQL harness, and fixture insertion points.

## §3 Known gaps

1. The migration has not been executed against PostgreSQL, including its second application. The disposable harness now performs both applications and then tests valid/invalid Branch snapshots, but those database outcomes are not yet verified.
2. The SQL harness branchless Rule insert is a full-row clone with only a new Rule identity/version/timestamps and `branch_id = NULL`; actual table constraints and the run-as insert trigger have not been exercised because PostgreSQL is unavailable.
3. Existing occurrence snapshots that lack Branch remain in place under `NOT VALID`; they are not backfilled with fabricated Branch identities. They cannot be treated as valid execution snapshots. New or changed rows must satisfy the constraint.
4. Rust formatting, compilation, integration tests, and Clippy remain unverified. The updated PG fixtures are source-reviewed only.
5. Target-database migration owner, deployed grants/RLS, deployment ordering, and rollback remain separate release gates.
6. Schedule worker/materializer, occurrence admission, infrastructure reservation/readiness, and execution remain outside this slice and stay disabled.

## §4 Subagent failure / takeover

No child agents were dispatched for this bounded lane. No delegated output required takeover.

## §5 Guardrails

| # | Guardrail | Evidence/state |
|---|---|---|
| 1 | Rule configuration may omit remote Branch context. | Harness checks nullable column and a persisted branchless Rule row; DB execution pending. |
| 2 | Occurrence snapshot must include `rule_snapshot.target.branch_id`. | Migration CHECK and static test authored. |
| 3 | UUID spelling accepts uppercase and lowercase hex in standard hyphenated form. | `~*` expression; harness includes both cases. |
| 4 | Missing key and JSON null fail closed. | Harness rejection cases authored. |
| 5 | Empty and malformed UUID strings fail closed. | Harness rejection cases authored. |
| 6 | Nil UUID fails closed, matching the existing Domain target validator. | SQL and harness cases authored. |
| 7 | Invalid text is not cast before UUID shape validation. | Cast is scoped to the `THEN` branch of `CASE`; malformed-input DB execution pending. |
| 8 | Existing historical snapshots are not rewritten or assigned synthetic Branch IDs. | `NOT VALID`; no backfill/update statement. |
| 9 | Future occurrence inserts and updates are constrained. | PostgreSQL CHECK semantics; live DB verification pending. |
| 10 | Migration reruns do not add duplicate constraints. | `pg_constraint` existence guard plus first/repeat migration plan; live DB verification pending. |
| 11 | The guard is scoped to `automation.occurrence`, not Rule persistence. | Migration only alters occurrence; Rule storage assertion is in the disposable harness. |
| 12 | Occurrence target snapshot remains required and bounded by the base schema. | Existing `NOT NULL`, object, and size checks are unchanged. |
| 13 | Existing PG integration fixtures carry the required Branch snapshot. | Updated adapter seed fixtures; compile/test pending. |
| 14 | Existing Domain checks remain the application-level validation authority. | Domain validator inspected and unchanged. |
| 15 | This migration does not enable Schedule execution. | No worker/materializer/executor code or runtime flag added. |
| 16 | Validation uses only a disposable test cluster; never the target database. | No database command was run because PostgreSQL binaries are unavailable. |
| 17 | No changes outside the assigned migration, narrow harness/fixture, and report scope. | Diff review pending coordinator integration. |

## §6 Sign-off

| Role | State |
|---|---|
| Architecture | Scoped to the Branch-required occurrence boundary while retaining branchless Rule configuration; coordinator review pending |
| SRE Lead | Disposable PostgreSQL replay and constraint behavior pending |
| Platform | Rust 1.99.0 formatting/build/test pending because toolchain commands are unavailable |
| Review host | Static tests, Python syntax, diff whitespace, and Cypher checks passed; CodeRabbit unavailable as recorded above |
| PM | Not production-ready; execution remains disabled |

## §7 Revision history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-22 | Add an idempotent occurrence snapshot Branch CHECK, disposable DB positive/negative cases, branchless Rule persistence assertion, and truthful verification limits. | `docs/briefs/WBS-WLD3-OCCURRENCE-BRANCH-SNAPSHOT-20261022.md` |
