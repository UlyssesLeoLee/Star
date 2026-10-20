# WLD-0 Migration Replay and Disposable Runner Audit

## §0 Purpose and scope

Record a bounded, read-only source audit of migrations `2026-10-15-run-host-independent-directory.sql`, `2026-10-15-run-host-independent-task-owner.sql`, and `2026-10-19-run-worktree-bindings.sql`, their schema prerequisites, and `scripts/automation/engineering_run_directory.py`. No migration, database, test, application role, or target environment was executed or changed by this audit. Line references below refer to the assigned `dev` baseline at `6e3e31ce34ae7eb6d6ee1de7066f693de48c172e`.

This report distinguishes source-defined behavior from the prior bounded disposable PostgreSQL result recorded in [WLD0-TARGET-DB-AUDIT-20261020.md](WLD0-TARGET-DB-AUDIT-20261020.md). That earlier result is evidence only for the runner's listed fresh-cluster chain and FORCE RLS catalog check; it is not evidence for the two October 15 migrations, October 19 migration, application-role behavior, or a deployed target database.

## §1 Migration prerequisites and change matrix

| Migration | Required schema lineage | DDL/data effect | Assessment |
|---|---|---|---|
| `2026-10-15-run-host-independent-directory.sql` | Worktree table from `2026-09-16-worktree-canvas-worktree.sql`; Project/Worktree binding from `2026-09-29-worktree-group-phase-2b.sql`; Branch/Run directory, scope/audit functions, and Run binding tables from `2026-10-01-engineering-run-directory.sql`. The base migration explicitly describes its 2B dependency (`2026-10-01-engineering-run-directory.sql:57-68`) and references the Project binding and Worktree tuple (`:184-220`). | Makes Run repository/remote Branch nullable, makes binding/event Branch nullable, adds Project+Run identity uniqueness and ownership FKs/checks, replaces the directory write guard, creates actor-scoped idempotency and Run Outbox tables with FORCE RLS. It performs no explicit `INSERT`, `UPDATE`, `DELETE`, or backfill. See `2026-10-15-run-host-independent-directory.sql:23-76,78-210,212-269`. | First application only after exact schema and data preflight. Do not blindly replay. |
| `2026-10-15-run-host-independent-task-owner.sql` | `task_metadata`, `work_item_worktree`, and `task_lifecycle_audit` from `2026-09-29-worktree-group-phase-2c.sql` (`:11-37,106-124`); `engineering_run` and `engineering_run_worktree_binding` from the October 1/15 directory chain; `task_run_outbox` from `2026-10-02-task-run-owner.sql:46-68`. The October 15 directory migration must precede this migration because this migration adds an FK to the new `(tenant_id, project_id, engineering_run_id)` Run identity and relies on the Project+Run binding shape (`2026-10-15-run-host-independent-directory.sql:35-39,56-60`; task-owner migration `:30-37,93-121`). | Makes task workspace/host context optional; switches canonical Task ownership to Project+Run; adds Run/Worktree consistency guards; permits null Worktree on newly appended lifecycle audit; evolves Task Outbox to schema v2 while retaining historical rows. Comments explicitly say no Worktree/Branch inference or backfill and that CLI/Schedule schemas remain unchanged (`2026-10-15-run-host-independent-task-owner.sql:5-18,21-54,281-297,372-490`). | First application requires the existing Task and Run schema lineage and its actual data. It does not enable CLI or Schedule admission. |
| `2026-10-19-run-worktree-bindings.sql` | `multica.engineering_run_outbox` from `2026-10-15-run-host-independent-directory.sql:234-255`. | In one transaction, removes the previous named allowlist constraints and adds an allowlist for `engineering_run.created`, `worktree_bound`, `worktree_reassigned`, and `worktree_detached`; no table or data rewrite (`2026-10-19-run-worktree-bindings.sql:11-29`). | Source appears replayable when all stored event types satisfy the new allowlist and no out-of-band schema drift exists. Verify catalog and stored values before deployment/replay. |

### Dependency order for the WLD-0 target slice

For the Run-to-Worktree directory slice, the order is:

1. `2026-09-16-worktree-canvas-worktree.sql`.
2. `2026-09-29-worktree-group-phase-2b.sql`.
3. `2026-10-01-engineering-run-directory.sql`.
4. `2026-10-15-run-host-independent-directory.sql`.
5. `2026-10-19-run-worktree-bindings.sql`.

For Task-owner acceptance, first establish `2026-09-29-worktree-group-phase-2c.sql` and `2026-10-02-task-run-owner.sql` (which creates `task_run_outbox`), in addition to the Run-directory chain above; then apply `2026-10-15-run-host-independent-task-owner.sql`. If validating later lifecycle Outbox behavior, `2026-10-16-task-lifecycle-outbox.sql` consumes that evolved Task Outbox and must follow it (`2026-10-16-task-lifecycle-outbox.sql:5-48`). The October 2 CLI execution identity migration is an independent execution schema dependency, not a prerequisite that makes this Task-owner migration authorize execution (`2026-10-02-task-run-engineering-run-binding.sql:7-10,17-22,91-92`).

The September 16 Worktree base contains `DROP TABLE IF EXISTS worktree_canvas_worktree CASCADE` (`2026-09-16-worktree-canvas-worktree.sql:8-12`); it is suitable only for an empty disposable fixture and must never be replayed on a populated target.

## §2 Transaction, lock, replay, and partial-failure findings

| Area | Evidence and result |
|---|---|
| Transaction boundary | Each of the three target files has one explicit `BEGIN`/`COMMIT` (`2026-10-15-run-host-independent-directory.sql:24,269`; `2026-10-15-run-host-independent-task-owner.sql:19,492`; `2026-10-19-run-worktree-bindings.sql:14,29`). A statement failure before commit should abort that migration's transactional DDL when executed as written; this is not a tested deployment/recovery guarantee. A connection lost during commit leaves commit outcome ambiguous and requires catalog/ledger reconciliation before any retry. |
| 15 October directory replay | Not idempotent after successful application. It adds named constraints without first dropping or checking them (`:35-37,56-76`); a second full run reaches duplicate constraint names and fails. It drops/recreates the binding-to-Run FK dynamically (`:43-60`), but that does not make the whole file replayable. If data violates a new check/FK or an index cannot be created, the transaction aborts. If the client cannot confirm commit outcome, first inspect a schema fingerprint and migration ledger; never blindly rerun. |
| 15 October Task-owner replay | Not idempotent after successful application. The migration adds a named Task check (`2026-10-15-run-host-independent-task-owner.sql:26-37`), appends the `engineering_run_id` audit column (`:283-292`), and adds named Task Outbox checks/FK (`:395-407`) without guarding every addition. Re-running after commit can fail on the already-added constraint/column. Earlier `DROP ... IF EXISTS` and `CREATE OR REPLACE` statements do not make the entire transaction replay-safe. |
| 19 October replay | The file deliberately drops both known old/new allowlist constraint names before adding its replacement inside the same transaction (`2026-10-19-run-worktree-bindings.sql:14-27`). Repeating this exact file is structurally repeatable when existing rows satisfy the new check. A different constraint name, changed event taxonomy, or failed/unknown commit still requires preflight rather than a blind retry. |
| DDL locks | The files use transactional `ALTER TABLE`, constraint creation and index creation; none declares an explicit migration lock timeout, statement timeout, lock budget, or online/concurrent index strategy. Operational lock duration and impact on populated production tables are therefore unproven. The runtime guard uses `FOR SHARE` on current Project membership and Project/Worktree rows (`2026-10-15-run-host-independent-directory.sql:137-143,189-205`); the Task-owner triggers also lock binding, relation, and metadata rows (`2026-10-15-run-host-independent-task-owner.sql:83-104,160-194,215-236,309-355`). The comments expressly say call paths still need lock-order review and this is not a deadlock proof (`:11-14`). |
| Existing-data checks | Before either October 15 file, preflight that all affected constraints and composite FKs validate, that expected unique indexes have the exact definitions, and that no duplicate/current or orphaned identities exist. The directory change adds an event-scope check and Run FKs (`2026-10-15-run-host-independent-directory.sql:62-76`); Task-owner adds Project/Run FKs and a v1/v2 Outbox contract (`2026-10-15-run-host-independent-task-owner.sql:30-37,395-407`). The migrations contain no data repair/backfill path. |

## §3 Disposable runner and acceptance evidence

The runner starts a unique PostgreSQL data directory and binds its server to `127.0.0.1`; it uses a trust-auth bootstrap role named `erun_validation` and invokes `psql` with `ON_ERROR_STOP=1` (`scripts/automation/engineering_run_directory.py:198-219`). It installs a local stand-in for the external `public.audit_trigger_func` and explicitly labels it DDL-only evidence (`:220-227`). The applied chain is exactly September 16 Worktree base, September 29 Phase 2B Project binding, and October 1 Engineering Run directory twice (`:228-236`). The runner counts eight catalog entries with both RLS flags and explicitly says role behavior/write workflows remain unverified (`:237-247`).

Optional `--task-run-ddl` applies Phase 2C, September 30 TaskExecutionRun, inserts one synthetic legacy row, replays October 2 Task Run identity migration twice, and checks only its forward guards (`:248-272`). Neither mode applies the October 15 or October 19 migrations. On successful shutdown the temporary data directory is removed; on stop failure it is preserved (`:273-284`). These file contents describe runner behavior; this audit did not run it. The earlier isolated run and exact limits are separately recorded in [WLD0-TARGET-DB-AUDIT-20261020.md](WLD0-TARGET-DB-AUDIT-20261020.md) §1-§3.

**Evidence boundary:** the source and earlier synthetic run demonstrate a fresh-cluster DDL slice only. The fixture substitutes a no-op audit function and uses the bootstrap/superuser path; the catalog query does not test RLS as a restricted API role. `FORCE ROW LEVEL SECURITY` catalog flags are not proof of least-privilege grants, transaction-local tenant/actor behavior, successful authenticated bind/reassign/detach, Outbox persistence, or target deployment.

## §4 Fail-closed extension and recovery recommendation

1. Keep the existing fresh-loopback cluster creation and destruction boundary. Before any migration step, assert the database endpoint is loopback, its data directory is unique and disposable, its version is expected, and the process cannot accept a target URL. Keep the no-op audit fixture clearly labeled as DDL-only.
2. Add an explicit ordered migration manifest and expected schema fingerprint for the audited slice. For the directory test, extend only after the current 09-16 → 09-29 2B → 10-01 chain. For the Task-owner test, add 09-29 2C and the 10-02 task-run-owner migration (which creates the Outbox), then apply the 10-15 directory and Task-owner migrations in that order. If a separate execution-schema test is intended, add the 09-30 TaskExecutionRun base and 10-02 binding identity migration there; they are not prerequisites that authorize Task ownership. Apply 10-19 only after the 10-15 directory Outbox exists. If the harness is intended to represent the complete October 19 migration state, include and review intervening 10-16 migrations in their own dependency order rather than skipping them.
3. Run each migration in an isolated, recorded transaction step; capture exit status and schema fingerprint after each commit. Stop at the first error. Do not restart from the failed SQL statement or rerun October 15 based only on process exit; reconcile whether that migration committed, compare the catalog with the expected fingerprint, then use a reviewed forward-repair or restore the disposable database from scratch.
4. Add preflight assertions for exact table/constraint/index definitions and relevant row invariants before October 15, and assert the 10-19 event allowlist against existing values before adding its constraint. Avoid `IF NOT EXISTS` as proof of correctness: a same-name, wrong-definition object must fail the fingerprint check.
5. Provision a distinct restricted runtime role in the isolated fixture and test no superuser/BYPASSRLS path, explicit grants, forced tenant RLS, transaction-local tenant/actor/correlation values, and cross-tenant denial. Keep a separate opt-in integration test target whose identity and owner are confirmed disposable; no production or developer target DB URL may be accepted implicitly.
6. Add rollback/recovery evidence for both October 15 files before target rollout. Their transactions protect against ordinary mid-file SQL failure, but no down migration or verified rollback was found in this scope. A successful commit must be rolled forward or restored from a tested backup; do not use blind reverse DDL on append-only history.

Do not mark the WLD-0 migration/DB gate complete until the extended disposable harness, restricted-role/RLS tests, migration replay/partial-failure cases, authenticated API persistence cases, and target migration-ledger review have evidence. Task execution admission remains closed until its separate owner, authorization, infrastructure, sandbox, and verification gates are accepted.

## §5 Known gaps

- The actual target database identity, migration ledger, October 15/19 deployment status, and schema fingerprint were not supplied or observed in this source audit.
- The runner has no October 15/19 steps, no deployment migration ledger, and no actual application-role grants/RLS behavioral probe.
- October 15's non-idempotent constraints/column additions have no tested recovery path; October 19's replayability has not been exercised against existing Outbox values.
- No live target DB, authenticated bind/reassign/detach flow, rollback, partial failure, same-key replay, or concurrency scenario was executed by this worker.
- Source review and the earlier disposable result do not verify end-to-end audit/Outbox durability or delivery.

## §6 Review and guardrails

- Audit level: bounded source verification. The assigned worktree has no CodeGraph index; conclusions are limited to the cited files and lines.
- No application code, SQL migration, runner, tests, or database was modified or executed. No production or target credentials were read.
- Only the existing disposable PostgreSQL result cited in §3 is treated as historical execution evidence, with its fixture limitations preserved.
- Continue to fail closed on missing migration history, role grants, RLS proof, or ambiguous commit state.

| Role | Review state |
|---|---|
| Architecture | Mavis review of source-level migration/replay findings; target acceptance remains open. |
| SRE Lead | Mavis review; the existing isolated runner covers only its recorded migration list. |
| Platform | Mavis review; application-role grants and behavioral RLS remain unverified. |
| Review chair | Mavis review; no migration/DB/test was run by this audit worker. |
| Product manager | Mavis review; execution admission remains closed. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Audit October 15/19 migration prerequisites, DDL, transaction/replay properties, runner coverage, RLS/grant gaps, and a fail-closed extension/recovery order. |
