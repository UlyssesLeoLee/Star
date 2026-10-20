# PHASE-WLD0 — Engineering Directory owner role and behavioral RLS

- **Date:** 2026-10-20
- **Baseline:** Orca `dev` candidate `ef0a76b6`
- **Revision:** v0.3
- **Reviewer:** Mavis review; production/API identity acceptance remains open

## §0 Purpose

Add a least-privilege PostgreSQL mutation identity for Run-to-Worktree binding commands and verify its lock-only access to other domain facts under FORCE RLS. Keep deployment-specific login membership, target database, authenticated API writes and execution admission separate from disposable acceptance.

## §1 Change matrix and acceptance

| Area | Change | Acceptance result |
|---|---|---|
| Directory role | New `star_engineering_directory_owner` is NOLOGIN, NOSUPERUSER, NOCREATEROLE, NOBYPASSRLS and NOINHERIT. The migration never grants membership to a deployment login; it clears old direct grants and memberships from the owner role, fails closed if it already owns database objects, and preserves deployment API-login membership. | Catalog and non-superuser session checks pass in disposable PostgreSQL. |
| REST transaction | Binding mutation begins a transaction, executes `SET LOCAL ROLE star_engineering_directory_owner`, then sets tenant/actor context. Missing membership fails the command closed. | Source change present; actual API login membership and HTTP mutations are not exercised. |
| Lock-only source access | Narrow UPDATE column grants permit PostgreSQL `FOR SHARE` locks on current Project/Branch/Run revision and grant facts, Project/Worktree bindings, Worktree projection and Task facts. Restrictive UPDATE-deny policies prevent mutation of those source facts. | Disposable session verifies required column grants, row locks, restrictive policy catalog and no-row Project/Task updates. |
| Owner-local writes | Run binding is the only SCD2 fact the role may change. Audit/Event writes are append-only, Run Outbox is INSERT-only, and idempotency access is bounded to replay/expiry cleanup. The role owns no RLS-protected table. | Grant and role-owner checks pass; no API bind/reassign/detach write is claimed. |
| Migration runner | `--run-binding-ddl` now includes the Task/Run prerequisite schema, owner migration, per-migration fingerprints including owner grants and a separate real non-superuser login acceptance session. A drift fixture injects direct/database privileges and outgoing role memberships, then replays the migration; a separate role-owned schema fixture expects migration failure. | 6/6 runner tests and 34/34 PostgreSQL steps pass; normalization, schema fingerprint stability and owner-object fail-closed behavior are verified. |
| Documentation | Requirements v6.90, Basic Design v6.91, Group DD v5.14, Automation Design v2.63, registry v1.06 and WBS v2.7 describe the contract and remaining gates. | Updated with disposable-only evidence, role-drift replay, owner-object rejection, and explicit unverified gates. |

## §2 Verification summary

- `python -m py_compile` for the runner and its focused test module: passed.
- `engineering_run_directory.py --runner-tests`: **6/6 passed**.
- `engineering_run_directory.py --run-binding-ddl` against a fresh loopback PostgreSQL 18.6 cluster: **34/34 steps passed**. The final run applied the ordered Worktree/Project/Task/Run chain, captured a fingerprint after every migration, checked the directory catalog and Outbox allowlist, then verified NOINHERIT, `SET LOCAL ROLE`, NOBYPASSRLS, owner-not-table-owner, required SELECT/`FOR SHARE`, tenant scoped Task reads, missing-tenant denial, and restrictive UPDATE denial against seeded Project/Task/Task-Worktree records. A drift fixture injected direct grants, database privileges and outgoing role memberships, then replayed the owner migration; the resulting role privileges were normalized and schema fingerprint remained stable. A role-owned schema fixture triggered the migration's expected fail-closed error, transaction rollback was observed, and the fixture was removed. PostgreSQL stopped and the runner removed only its unique temporary data directory.
- A first role-acceptance attempt exposed a fixture query that checked a multica relation before the login had schema usage. The assertion was moved after `SET LOCAL ROLE`; the failed run stopped PostgreSQL and cleaned its unique data directory. During final self-review, the API authorization `FOR SHARE` on `multica.engineering_run_revision` exposed a missing lock-only `valid_to` grant; the migration now grants that column with a restrictive UPDATE-deny policy, and the disposable session explicitly checks its row-lock privilege and policy. The complete rerun passed.
- Rust all-targets check: **not verified**. The Cargo shim is absent from PATH; invoking the available Rust 1.98.1 toolchain through `rtk proxy` was rejected because workspace crates require Rust 1.99.0. The 1.99.0 toolchain was not found on this host.
- Target-database migration, app-login provisioning, authenticated API writes, rollback, replay races and production deployment were not run.

## §3 Known gaps

1. No target database was inspected or mutated; target migration ledger/catalog state and ambiguous-commit reconciliation remain unknown.
2. The actual REST deployment login and its `star_engineering_directory_owner` membership are not identified or configured. The runner’s temporary login proves the role mechanism only.
3. No authenticated attach/reassign/detach API write, revocation race, exact/conflicting idempotency replay, rollback injection, duplicate-current-row handling, concurrent CAS/lock race or post-deployment history reconciliation ran.
4. Rust 1.99.0 compile/test/fmt gates remain open because the toolchain is missing. The available 1.98.1 is below the workspace/package MSRV and is not an acceptable substitute.
5. This phase does not enable Task CLI, Agent, Schedule, Runtime, BI or Benchmark execution. Admission remains closed.

## §4 Subagent failure takeover

| Work item | Result | Coordinator action |
|---|---|---|
| Directory owner-role implementation and integration | No subagent was assigned in this continuation slice. | Coordinator reviewed the source, migration, runner and documentation directly; no partial worker output was adopted. |

## §5 Guard rules

1. Keep the owner role NOLOGIN/NOSUPERUSER/NOBYPASSRLS/NOINHERIT; do not bake an API login name into the migration.
2. Switch roles transaction-locally before setting tenant context or querying/mutating Directory facts.
3. Grant only SELECT and the narrow columns needed for `FOR SHARE`; keep restrictive UPDATE-deny policies on facts owned by other domains.
4. Do not grant Run Outbox reads, updates or deletes to the Directory mutation role.
5. Run PostgreSQL only as a fresh loopback disposable cluster; never accept a target DB URL.
6. Stop PostgreSQL before removing its unique data directory; preserve the directory if shutdown fails.
7. Treat this fixture as role/RLS evidence, not production API, target migration or execution proof.
8. Keep Task/Schedule execution and BI ingestion fail-closed until their independent owner, sandbox, readiness and recovery gates pass.

## §6 Review signatures

| Role | Reviewer | State |
|---|---|---|
| Architecture | Mavis | Reviewed role boundary; deployment identity remains open. |
| SRE Lead | Mavis | Reviewed disposable runner lifecycle and cleanup. |
| Platform | Mavis | Disposable role/RLS behavior passed; target grants remain open. |
| Review chair | Mavis | Focused runner and PostgreSQL evidence recorded. |
| Product manager | Mavis | No execution capability is promoted by this phase. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.3 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record 34/34 PostgreSQL steps, injected grant/database privilege/membership drift and migration re-normalization with a stable schema fingerprint; test role-owned schema rejection and preserve target DB/API/Rust production gates. |
| v0.2 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add and verify the missing Run revision lock-only grant and UPDATE-deny policy found during final review; synchronize all design versions and preserve production and Rust 1.99.0 gates. |
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add the NOLOGIN Directory mutation role, transaction-local role switch, lock-only column grants, restrictive RLS denial and disposable PostgreSQL acceptance; preserve production and Rust 1.99.0 gates. |
