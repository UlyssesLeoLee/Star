# PHASE-WLD0 — Hostless Engineering Run authenticated HTTP acceptance

- **Date:** 2026-10-23
- **Baseline:** local `dev` candidate `ad56a413cf87b55587f6b2c7b15a271e13e8945f`
- **Lane:** `UlyssesLeoLee/wld0-hostless-run-api-20261023`
- **Revision:** v0.1
- **Review state:** Local disposable acceptance passed; integration to `dev` is the next gate.

## §0 Purpose

Verify the Project-scoped hostless Engineering Run API through authenticated HTTP and a fresh disposable PostgreSQL 18 loopback cluster. Close the least-privilege owner-role gap exposed by the route: the Directory owner must be able to insert the Run identity, first revision, and creator grant in the same owner transaction. Keep target-database deployment, production identity, and Task/CLI execution as separate gates.

## §1 Change matrix and acceptance

| Area | Change | Acceptance result |
|---|---|---|
| REST transaction | `begin_read` and `begin_write` switch to `star_engineering_directory_owner` before setting tenant scope. | Authenticated hostless HTTP create, list and RunContext routes pass in the disposable cluster. |
| Owner grants | New forward migration grants only `INSERT` on `multica.engineering_run`, `multica.engineering_run_revision` and `permission.engineering_run_role_binding`; it does not grant these writes to the API login. | Catalog, effective privilege, owner-role drift normalization and schema-fingerprint checks pass. |
| Hostless Run API | Route acceptance covers Project-manager create, exact idempotency replay, Project Run list, RunContext without Worktree focus, and a Project developer without the Run grant. | All positive requests pass; the unauthorized Run is not exposed. |
| Existing binding contract | The same harness retains bind race, injected Outbox rollback and detach phases. | Each HTTP phase passes; final binding/audit/Outbox snapshots match expected facts. |
| Runner and unit gates | Directory migration plan and binding runner apply the new migration and verify effective owner/runtime grants. | Focused Python suites pass 20/20 and 8/8; the end-to-end PostgreSQL runner passes 55/55 steps. |
| Documentation | Requirements, Basic Design, Group DD and WBS record the owner boundary, local evidence and remaining production gates. | Synchronized in this lane; target database and production deployment are explicitly not claimed. |

## §2 Verification summary

- `python -m unittest discover -s scripts/automation -p test_engineering_run_directory.py`: **20 passed**.
- `python -m unittest discover -s scripts/automation -p test_phase9f2_binding_http_acceptance.py`: **8 passed**.
- `scripts/automation/phase9f2_binding_http_acceptance.py` on a fresh PostgreSQL 18 loopback cluster: **55/55 steps passed**. The hostless create/replay/list/context phase, bind race, rollback injection, detach, migration replay/fingerprints, role attributes, and effective owner/runtime grants all returned exit code 0.
- Each Rust HTTP invocation completed its exact ignored integration case successfully: `rust-http-hostless-run`, `rust-http-bind-race`, `rust-http-rollback-detach`, and `rust-http-detach`.
- The runner reported `target_database_touched=false`, stopped PostgreSQL successfully, and removed its unique temporary data directory. No Multipass or Docker instance was used.
- Changed Rust was formatted with Rust 1.99.0 `rustfmt`; CypherGraph Guardian was run on the supported source files. Manual self-review covered the changed Rust routes/tests, owner migration, runner steps and synchronized docs. CodeRabbit was unavailable: the installed Windows `.cmd` delegates to `/root/.local/bin/coderabbit` in WSL, which returned `Permission denied`; no CodeRabbit clean result is claimed.

## §3 Known gaps

1. No target or production database was connected, migrated, or inspected. Deployment API-login membership for `star_engineering_directory_owner` remains environment-specific and unverified.
2. No production authentication/session provider, deployment grants/RLS catalog, post-deployment migration ledger, or historical Run/Worktree reconciliation was tested.
3. This phase does not complete the full Run-to-Worktree lifecycle, Task Card owner migration, hostless Task workflow, CLI provisioner, Runtime sandbox, Schedule worker, or Agent execution. Those capabilities remain fail-closed.
4. The local PostgreSQL runner does not establish production concurrency, multi-instance recovery, service availability, performance, or backup/restore guarantees.
5. Integration and managed-worktree removal are recorded only after the `dev` checkout passes its own preflight and fast-forward gates.

## §4 Subagent failure takeover

| Work item | Result | Coordinator action |
|---|---|---|
| Hostless Run HTTP/API implementation | No subagent was assigned directly to this WLD-0 slice. | Coordinator reviewed the changes, reran the complete disposable acceptance, and owns final Git integration. |
| Other parallel lanes | UI sampler and target-database probe were separate WBS lanes, not dependencies of this acceptance. | Their results were not copied into this lane; only the already-integrated `dev` baseline was used. |

## §5 Guard rules

1. Keep `star_engineering_directory_owner` NOLOGIN/NOSUPERUSER/NOBYPASSRLS/NOINHERIT; do not encode deployment login membership in the migration.
2. Switch role inside each owner transaction before tenant/actor GUCs or Directory reads/writes.
3. Limit new Run creation rights to INSERT on Run identity, first revision, and creator grant; do not widen API-login or cross-owner write privileges.
4. Missing role membership, Project authority, Run grant, or required database fact must fail closed; unauthorized Runs remain indistinguishable from absent Runs.
5. Run PostgreSQL only on the harness-owned loopback cluster; stop it before removing its unique data directory.
6. A disposable acceptance pass does not authorize production migration, Task/CLI execution, Agent admission, Schedule execution, or Runtime activation.

## §6 Review signatures

| Role | Reviewer | State |
|---|---|---|
| Architecture | Mavis | Reviewed owner boundary and hostless Project → Run semantics. |
| SRE Lead | Mavis | Reviewed disposable database lifecycle; target deployment remains open. |
| Platform | Mavis | Authenticated route and effective role-grant acceptance passed locally. |
| Review chair | Mavis | Final source, migration, test and documentation review recorded before integration. |
| Product manager | Mavis | No production or execution capability is promoted by this phase. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add authenticated hostless Run HTTP acceptance, least-privilege owner INSERT migration, 55-step PostgreSQL evidence, and explicit deployment/execution gaps; `dev` integration pending. |
