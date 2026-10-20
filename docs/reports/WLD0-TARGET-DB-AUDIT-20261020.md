# WLD-0 Target PostgreSQL and Run-to-Worktree Binding Audit

## §0 Purpose

Record a bounded review of Run-to-Worktree binding source and migration evidence at the assigned `dev` baseline. This report distinguishes source contracts and a disposable schema-harness result from real target PostgreSQL acceptance. No production or target database was connected to or changed. A coordinator follow-up ran the existing isolated PostgreSQL DDL harness; it applied only the harness's base/directory migration chain and did not exercise application-role behavior or binding writes. Graph tooling exposed no applicable indexed-project/coverage check for the binding handlers, so the review used focused source reads and literal searches of the brief's files; this is not a repository-wide completeness claim.

## §1 Change and acceptance matrix

| Area | Evidence | Assessment |
|---|---|---|
| Bind, reassign, detach and actor authorization | The API registers authenticated `PUT` and `DELETE` routes (`crates/star-api-rest/src/group_api/run_worktree_bindings.rs:225-233,235-273`). Each mutation requires actor validation, `project:write`, `worktree:manage`, a current Project manager binding, and manager authorization for the target and, on reassign/detach, current Run (`:286-305,344-359`; Run authorization resolves current Project and Run grants in `crates/star-api-rest/src/group_api/engineering_runs.rs:360-384`). | Source path exists. The route integration test only proves unauthenticated requests return 401 and private cache headers (`crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs:29-68`); authenticated database-backed authorization is not proven. |
| Hostless Run binding | Run scope carries optional repository/Branch identity; target validation rejects a mismatching repository or Branch without a repository but accepts absent remote metadata (`run_worktree_bindings.rs:539-550,1099-1117`). Binding writes retain the local Worktree repository and optional Run Branch (`:819-848`). | Source supports hostless Run metadata while preserving local Worktree repository identity. Database enforcement is unverified. |
| CAS and concurrent writes | The request must include the expected Project-binding version and exact current binding ID/version (`run_worktree_bindings.rs:443-470,473-536`). A tenant/Worktree transaction advisory lock serializes binding changes; row locks and an exact conditional close provide further protection (`:318-342,693-705,731-770,794-816`). A partial unique index limits each tenant/Worktree to one current row (`2026-10-01-engineering-run-directory.sql:213-220`). | Concurrency design is present in source/DDL. The only PostgreSQL lock test is ignored unless given an explicitly disposable `STAR_WLD0_BINDING_TEST_DATABASE_URL` (`run_worktree_bindings.rs:1161-1217`). No live CAS race or duplicate-current-row rejection was tested. |
| Idempotency and replay | A transaction-scoped advisory lock is keyed by tenant, actor and idempotency key; a matching unexpired request hash returns its stored response after current target-Run authorization (`run_worktree_bindings.rs:294-315,592-624`). New mutation, receipt, Outbox event and binding changes commit in one transaction (`:302-419`). The receipt key is actor-scoped, has a 24-hour expiry bound, and is protected by FORCE RLS (`2026-10-15-run-host-independent-directory.sql:212-232`). | Atomic replay path is source-backed. No PostgreSQL replay, hash-conflict, rollback, or expiry test was run. |
| Audit and Outbox | SCD2 source writes invoke the directory audit trigger, which inserts an audit fact in the same transaction (`2026-10-01-engineering-run-directory.sql:416-442,468-475`). Binding events identify bind/reassign/detach, previous/current binding snapshots, actor, correlation and `execution_admission_available: false`; they are inserted before the transaction commits (`run_worktree_bindings.rs:389-419,851-903`). Migration `2026-10-19-run-worktree-bindings.sql:11-29` adds the three lifecycle event names to the existing Outbox allowlist. | Transactional source intent exists. The Outbox delivery worker, database persistence, deployed event constraint and failure rollback are not established here. |
| RLS and grants | The base directory migration enables and forces tenant RLS for eight directory tables and installs tenant-scoped policies/triggers (`2026-10-01-engineering-run-directory.sql:444-478`). The host-independent migration adds actor-scoped FORCE RLS for idempotency and tenant-scoped FORCE RLS for Outbox (`2026-10-15-run-host-independent-directory.sql:212-255`). | Policy DDL is present in source. The reviewed migrations contain no application API role creation or GRANT/REVOKE contract for the binding, Worktree, Project/Run grant, idempotency and Outbox accesses. Role membership, least-privilege DML, target RLS behavior and transaction-local context under that role remain unknown. Other owner-role grants for infrastructure binding do not establish API-role access (`2026-10-10-multica-run-infrastructure-binding-master.sql:222-264`). |
| Migration replay and rollback | Each reviewed migration wraps its DDL in `BEGIN`/`COMMIT`. The host-independent migration adds named constraints without first dropping/guarding all of them (`2026-10-15-run-host-independent-directory.sql:35-39,56-76`), so replay after a successful commit is not idempotent for those additions. The lifecycle migration drops its prior allowlist constraints before adding the replacement (`2026-10-19-run-worktree-bindings.sql:14-27`). | Transactional DDL is source-backed; no down migration or rollback exercise was found in the reviewed scope. Deployment-ledger behavior is not verified. Reapplying 2026-10-15 should not be treated as safe. |
| Disposable runner coverage | `scripts/automation/engineering_run_directory.py:198-247` creates a fresh loopback PostgreSQL cluster, applies the 2026-09-16 Worktree base, 2026-09-29 Project binding, and 2026-10-01 directory migration twice, then checks FORCE RLS catalog flags for eight tables. A coordinator follow-up ran it successfully (exit 0) in the WLD-0 audit worktree. Its legacy audit function is a no-op fixture (`:220-225`) and the runner explicitly says role behavior/write workflows remain unverified (`:237-247`). | This is bounded synthetic-schema evidence only. The runner does not apply 2026-10-15 or 2026-10-19, test application-role grants or binding behavior, or prove target state. It cannot close this WLD-0 DB gate. |

## §2 Validation summary

- **Source audit:** focused source and migration review only; no Rust test, production-role probe, or target/production PostgreSQL connection was made.
- **Available source-level checks:** `rtk proxy cargo test -p star-api-rest --lib group_api::run_worktree_bindings::tests -j 4` and `rtk proxy cargo test -p star-api-rest --test engineering_run_worktree_binding_routes -j 4`. They were not run in this read-only audit. The route test is unauthenticated-only; unit tests do not establish database behavior.
- **Disposable schema runner:** coordinator follow-up ran `rtk proxy py -3 scripts/automation/engineering_run_directory.py --postgres-ddl --postgres-port 57453 --output-dir .cache/wld0-db-audit` from the WLD-0 audit worktree. It exited 0, created and cleanly stopped a fresh loopback PostgreSQL 18 cluster, applied 2026-09-16 / 2026-09-29 / 2026-10-01 (the last twice), and checked FORCE RLS catalog flags for eight tables. The run's summary/logs are under `.cache/wld0-db-audit`; its temporary cluster data was removed after shutdown. The runner's legacy audit trigger is a no-op fixture, and the run did not apply 2026-10-15 or 2026-10-19, probe application grants/RLS as the runtime role, or execute binding workflows. This does not establish the target database state.
- **PostgreSQL concurrency case:** the ignored advisory-lock test requires an explicitly supplied disposable DB URL and tests lock acquisition only, not end-to-end binding mutation (`run_worktree_bindings.rs:1161-1217`). It was not run because no target/disposable database was verified for this audit.
- The source audit agent ran `rtk proxy git diff --check` for the original report; the coordinator reran it after v0.2 and it passed. The follow-up schema runner was the only PostgreSQL execution.

## §3 Known gaps and WLD-0 closure gates

1. Identify the intended target database and its migration ledger/state without exposing credentials; verify that migrations 2026-10-01, 2026-10-15 and 2026-10-19 are applied in the expected order.
2. Define and provision the actual application DB login/group role and explicit least-privilege grants for all tables/functions used by this API. Verify no `BYPASSRLS`, superuser, or unintended owner path; verify role membership and transaction-local tenant/actor/correlation settings.
3. Extend the isolated runner beyond its passing 2026-09-16 / 2026-09-29 / 2026-10-01 schema chain. Define safe apply/replay policy for 2026-10-15 and 2026-10-19, then exercise catalog constraints, FORCE RLS, cross-tenant isolation and grants as the same restricted runtime role. Do not infer target deployment from a synthetic schema-only fixture.
4. Exercise authenticated bind, reassign, detach and exact replay; same-key/different-digest conflict; current grant revocation; Project/Run/repository mismatch; stale Project/binding versions; concurrent same-Worktree attempts; and one-current-row enforcement.
5. Inject a failure between binding close/insert, audit/Outbox insert, and idempotency receipt insert; prove the transaction leaves no partial history or event. Verify outbox event constraints and the actual persistent rows after commit.
6. Add an explicit rollback/recovery plan for the forward schema change and verify it against a disposable copy. The 2026-10-15 migration is not safe for blind replay after a successful application.
7. Keep Task/Agent/CLI/Schedule execution disabled. The emitted binding event explicitly sets `execution_admission_available` false (`run_worktree_bindings.rs:886`); no target DB, application role or lifecycle delivery evidence here justifies changing that state.

## §4 Worker failure and coordinator takeover

No worker failure occurred. The assigned audit was completed as a report, then the coordinator ran the existing disposable schema harness to strengthen the evidence. No product source, migration, runner, WBS, or test file was changed. The harness still leaves the application-role, target DB, and end-to-end binding gates open.

## §5 Guardrails

- Source code, SQL DDL, unit tests and catalog-only fixtures are separate evidence tiers; none alone proves target deployment acceptance.
- Only a disposable database whose identity and isolation are confirmed may be used for write, rollback, replay or concurrency scenarios.
- Do not inspect or print environment secrets; obtain database identity through approved non-secret metadata.
- Require the real restricted application role for RLS/grant acceptance; a migration owner, superuser or hand-built synthetic grant is not equivalent.
- Project and Run authorization is rechecked in the API transaction; DB RLS remains an additional boundary, not a substitute for those checks.
- Keep execution admission closed until persisted binding, current authorization, Outbox durability/delivery and independent Runtime/sandbox gates are accepted.
- Reconcile this report with requirements, basic design and detailed design before marking WLD-0 complete.

## §6 Signoff

| Role | Review state |
|---|---|
| Architecture | Mavis review of bounded source/DDL evidence; target DB acceptance remains open. |
| SRE Lead | Mavis review; the isolated PostgreSQL 18 schema runner exited 0 for its bounded migration list; target migration, rollback and recovery evidence remain open. |
| Platform | Mavis review; application role/grants and target RLS behavior remain open. |
| Review chair | Mavis review; focused reads, isolated DDL harness, and report diff check only. |
| Product manager | Mavis review; execution admission remains closed. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the bounded binding API, hostless schema, authorization, idempotency, Outbox, RLS/grant, migration replay and disposable-runner evidence; retain target PostgreSQL gates. |
| v0.2 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the coordinator's successful isolated PostgreSQL 18 DDL runner for the 2026-09-16 / 2026-09-29 / 2026-10-01 chain, its eight FORCE RLS catalog checks and no-op audit fixture; correct the earlier no-run statement and keep 2026-10-15/19, app grants, binding writes and target DB acceptance open. |
