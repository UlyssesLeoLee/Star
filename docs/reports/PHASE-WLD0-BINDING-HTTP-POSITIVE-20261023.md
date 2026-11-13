# PHASE-WLD0-BINDING-HTTP-POSITIVE-20261023

## §0 Purpose

This report records the positive authenticated Run-to-Worktree binding acceptance added after the earlier scope-boundary-only report `PHASE-WLD0-BINDING-HTTP-ACCEPTANCE-20261022.md` v0.3. The code commit `3e67b3fb3f36c5e2524695abcefdc61edff83c20` was rebased onto the then-current local `dev@89edea2b85d0433b96446546bfb28b9d1b89d27a` and fast-forwarded into `dev`. This is disposable PostgreSQL acceptance evidence; it does not establish production target-database readiness or close WLD-0.

## §1 Change matrix

| Area | Change | Evidence/status |
|---|---|---|
| Lock-only RLS policy | Added a forward migration that permits the owner role to see/lock the five source rows needed by `SELECT ... FOR SHARE` (`USING (true)`) while retaining `WITH CHECK (false)` for UPDATE. | Disposable PostgreSQL catalog checks passed; direct UPDATE probes on all five policies returned SQLSTATE `42501`, and source rows remained active. |
| Authenticated HTTP path | Added a real-router test using signed test JWTs and a non-superuser runtime database login; each mutation transaction selects the owner role. | Bind, detach and injected Outbox-failure phases passed against isolated PostgreSQL. |
| Concurrency and idempotency | Added concurrent compare-and-swap, exact replay, changed-payload key reuse and stale-version checks. | One concurrent writer won and one conflicted; exact replay returned the saved receipt; changed request and stale version conflicted. |
| Scope and RLS | Added same-tenant non-owner and cross-tenant requests plus an owner-role visibility probe. | Unauthorized and cross-tenant requests returned not-found; the other tenant saw zero binding rows. |
| Rollback and durable facts | Added snapshots for binding history, audit, Outbox and idempotency facts; injects a detach Outbox failure. | Failed detach returned 500 and left all measured facts unchanged; successful detach and exact replay produced one closed binding, one detach Outbox event and one close audit fact. |
| Reproducible acceptance | Added an isolated PostgreSQL 18 runner, static runner tests and a policy catalog regression assertion. | The runner used a unique data directory and random loopback port, retained logs, and removed only its own successful disposable database after a clean stop. |

## §2 Validation summary

- On the rebased candidate, `rtk proxy python scripts/automation/phase9f2_binding_http_acceptance.py --run-unit-tests` exited 0. The final run completed 50/50 steps with zero failures on PostgreSQL 18.6, loopback port `58508`, runtime login `erun_directory_api_test`; the run summary recorded `target_database_touched=false`. The final `pg_ctl stop` exited 0 and the successful disposable data directory was removed.
- The three Rust 1.99.0 phases (`bind-race`, `rollback-detach`, `detach`) each exited 0. The runner invokes Cargo with `-j 4 --locked` and the ignored test is enabled only by this isolated runner.
- `python scripts/automation/test_engineering_run_directory.py`: 6/6 passed. `python -m unittest scripts.automation.test_phase9f2_binding_http_acceptance`: 8/8 passed. `python -m py_compile` for the four changed Python files exited 0.
- The lane worker reported changed-file CypherGraph Guardian success. `rtk proxy git diff --cached --check` exited 0; only Git's CRLF/LF normalization notices appeared.
- The run, earlier failed-attempt logs and PostgreSQL diagnostic evidence were copied outside the checkout and SHA-256 verified. `C:\Users\leo19\.codex\worktree-evidence\Star\wld0-binding-http-positive-20261023\manifest.sha256` covers 5,007 files / 134,106,567 bytes. Generated Cargo target caches were excluded from evidence preservation.
- CodeRabbit could not run: `/bin/bash` reported `/root/.local/bin/coderabbit: Permission denied`. Local source, migration, runner and test review was completed; this is not a CodeRabbit pass.
- No target/system database was used, no fetch or push was performed, and no production execution was enabled.

## §3 Known gaps

1. The acceptance uses a disposable PostgreSQL fixture. Target database app-role membership, deployed grants/FORCE RLS, migration-ledger state, deployment ordering, production replay/rollback and historical binding reconciliation remain unverified.
2. Hostless Run create/list/open, all production binding lifecycle paths, duplicate-current-row repair and full target-scope authorization still require their own acceptance.
3. This result does not prove a production Agent runtime, independent sandbox, Task CLI, Schedule worker, Infrastructure binding/readiness, or production BI consumer. Execution admission remains fail-closed.

## §4 Sub-agent failure and coordinator handoff

The WLD0 acceptance worker completed the assigned scope. No worker failure or recovery takeover occurred. The coordinator reviewed the changed implementation and evidence, created commit `2ba5d7cd`, rebased it onto current `dev` as `3e67b3fb`, reran the isolated acceptance, and fast-forwarded it into the clean inactive `dev` checkout. No worker-owned data was discarded.

## §5 Guardrails

- The runner strips inherited PostgreSQL/database overrides, chooses a random IPv4 loopback port, uses a unique private data directory and never targets the system PostgreSQL instance.
- Runtime login identity was asserted before requests; the production handler selects the narrowly scoped owner role inside each mutation transaction.
- UPDATE denial was asserted by exact SQLSTATE `42501` and unchanged active source rows, not by catalog shape alone.
- Database snapshots prove only the disposable fixture's binding/audit/Outbox/idempotency deltas. Production grants and migration state are not inferred.
- The runner removed its own data directory only after all acceptance steps and `pg_ctl stop` succeeded. Failed attempts remain preserved in the external evidence backup.
- The code commit was integrated locally only; no fetch, push, target-database deployment or execution enablement occurred.

## §6 Signoff

| Role | Review state |
|---|---|
| Architecture | Mavis review: bounded positive HTTP/DB acceptance is integrated; WLD-0 remains open. |
| SRE Lead | Mavis review: disposable PostgreSQL 18.6 passed; production database and deployment remain unverified. |
| Platform | Mavis review: runtime-role lock and UPDATE-denial behavior passed only in the isolated fixture. |
| Review chair | Local review and focused validation passed; CodeRabbit was unavailable due permission denied. |
| Product manager | Mavis review: no production readiness or execution capability is claimed. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the rebased, database-backed positive binding HTTP acceptance, its 50/50 disposable PostgreSQL run, local integration and remaining target-database/product gates. |