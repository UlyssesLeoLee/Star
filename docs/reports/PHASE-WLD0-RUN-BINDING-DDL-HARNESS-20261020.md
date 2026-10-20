# PHASE-WLD0 — Run/Worktree Binding DDL Harness

- **Date:** 2026-10-20
- **Baseline:** local `dev` after `9c988ad3`
- **Revision:** v0.1
- **Reviewer:** Mavis review; target-database acceptance remains open

## §0 Purpose

Extend the existing Engineering Run directory automation with an opt-in, fresh-cluster validation path for the 2026-10-15 host-independent Run directory migration and the 2026-10-19 Run/Worktree Outbox event migration. The harness must apply the non-replay-safe 2026-10-15 migration only once, stop at the first failed step, and preserve a clear boundary between disposable schema evidence and production acceptance.

## §1 Change matrix and acceptance

| Area | Change | Acceptance result |
|---|---|---|
| Migration plan | `directory_migration_plan(run_binding=True)` defines 2026-09-16 → 2026-09-29 2B → 2026-10-01 → safe 2026-10-01 repeat → 2026-10-15 directory → 2026-10-19 Run/Worktree bindings. | October 15 and 19 each occur once and in dependency order. |
| Fail-stop execution | `apply_migration_plan` fingerprints after each successful migration and does not start later migrations after an apply or fingerprint failure. | Focused unit coverage passes. |
| Catalog checks | Run binding mode checks FORCE RLS catalog flags for all ten directory/Run binding tables and the exact four-event Outbox constraint. | Disposable PostgreSQL catalog checks pass. This is not behavioral RLS evidence. |
| Runner tests | `--runner-tests` invokes only the focused standard-library unit tests. | 4/4 tests pass; no database is opened by this mode. |
| Documentation | Automation design §4.96, automation registry, and this phase report describe the new scope and its evidence limits. | Documentation records DDL-only status and keeps WLD-0 open. |

## §2 Validation summary

- `python scripts/automation/engineering_run_directory.py --runner-tests`: exit 0, 4 tests passed.
- `registry_check.py --output-log .cache/wld0-run-binding-ddl/registry-check.log`: exit 0, 0 errors, 198 existing index warnings; the new runner test entry was recognized.
- `python scripts/automation/engineering_run_directory.py --run-binding-ddl`: exit 0; all 18 runner steps passed against local PostgreSQL 18.6 in a fresh unique data directory listening only on `127.0.0.1`.
- Migration order was 2026-09-16 Worktree base, 2026-09-29 Phase 2B, 2026-10-01 directory, 2026-10-01 repeat, 2026-10-15 host-independent directory, and 2026-10-19 Run/Worktree bindings. Each successful migration was followed by a catalog fingerprint. The 2026-10-01 first and repeat fingerprints matched (`5a2a31d8de2167094d77f3914411ca62`); fingerprints after 2026-10-15 and 2026-10-19 were `8ab447c764b73249ec9aa3efc46d57a6` and `4a7936f21bb21c8f4f9b681214a03bf6` respectively. The MD5 values are drift-comparison aids, not security attestations.
- The final catalog guard reported ten expected tables with both RLS flags and confirmed the exact allowed Run Outbox lifecycle event names. PostgreSQL stopped successfully and the invocation's unique data directory was removed.
- `git diff --check` passed; CypherGraph Guardian passed for both changed Python source files after source/header reconciliation.

## §3 Known gaps

1. This was a fresh disposable schema run. It did not inspect a deployed database, migration ledger, target schema fingerprint, or target grants.
2. The runner uses its bootstrap `erun_validation` role with trust authentication. It does not prove the restricted API runtime role, effective grants, `NOBYPASSRLS`, or tenant/actor RLS behavior.
3. The external legacy `public.audit_trigger_func` is represented by an explicit no-op DDL fixture. No audit behavior is proven.
4. The Outbox table is empty in this fixture. The catalog guard verifies constraint shape and stored-value compatibility for the empty table; it does not exercise authenticated Outbox inserts or owner delivery.
5. The 2026-10-15 migration is not replayed by the harness and remains unsafe for blind replay after successful application. Ambiguous target commit outcomes require ledger/catalog reconciliation.
6. No authenticated bind/reassign/detach API write, exact idempotency replay, permission revocation, cross-tenant RLS denial, concurrent CAS, injected rollback, or duplicate-current-row test was run.
7. No target-database migration/replay protocol, runtime role grants, authenticated PostgreSQL API acceptance, or production deployment evidence is included. WLD-0 remains open.
8. Task/CLI execution admission, Schedule worker admission, Runtime sandbox, BI delivery and Benchmark remain separate gates and stay fail closed.

## §4 Subagent failure takeover

| Work item | Result | Coordinator action |
|---|---|---|
| WLD-0 migration replay audit | The first audit attempt hit the worker usage limit before producing a deliverable. | The retry completed the bounded source audit; its report was integrated at `c1dc25f4`. No incomplete code was adopted. |
| WLD-0 API acceptance audit | Read-only audit completed; report commit `9c988ad3` is in local `dev`. | Reviewed the report and used its PG-0–PG-9 gaps as the acceptance boundary for this DDL-only runner. |
| Runner implementation | Coordinated in its isolated worktree. | The coordinator owns final review, Cypher reconciliation, integration and cleanup. |

## §5 Guard rules

1. Only create a fresh, unique disposable PostgreSQL data directory for each run.
2. Bind PostgreSQL to `127.0.0.1`; never accept a target database URL.
3. Apply the 2026-09-16 `DROP TABLE ... CASCADE` migration only inside that fresh disposable cluster.
4. Keep the legacy audit-trigger stub explicit and treat it as DDL-only scaffolding.
5. Preserve the declared migration dependency order.
6. Apply the non-replay-safe 2026-10-15 directory migration exactly once per fresh cluster.
7. Apply the 2026-10-19 event-allowlist migration only after the 2026-10-15 Outbox exists.
8. Do not use `IF NOT EXISTS` or a matching object name as proof that a target schema is correct.
9. Capture a catalog fingerprint after each successful migration commit.
10. Stop the migration chain on the first SQL or fingerprint error.
11. Never restart from a failed statement or blindly replay an October 15 migration after an ambiguous commit.
12. Stop PostgreSQL before deleting the runner's data directory.
13. Preserve the data directory if shutdown fails.
14. FORCE RLS catalog flags do not prove restricted-role or policy behavior.
15. A superuser/bootstrap DDL fixture cannot prove application-role grants or tenant isolation.
16. Empty-Run Outbox DDL checks do not prove durable authenticated writes or message delivery.
17. Keep Run execution, Task CLI, and Schedule admission closed until their independent authorization, sandbox, readiness and recovery gates pass.

## §6 Review signatures

| Role | Reviewer | State |
|---|---|---|
| Architecture | Mavis | Reviewed the runner's evidence boundary; target acceptance remains open. |
| SRE Lead | Mavis | Reviewed disposable-cluster lifecycle and cleanup behavior. |
| Platform | Mavis | Runtime-role grants and behavioral RLS remain unverified. |
| Review chair | Mavis | Focused unit and disposable DDL evidence recorded. |
| Product manager | Mavis | No user-facing execution capability is promoted by this harness. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add opt-in October 15/19 Run binding DDL chain, migration fingerprints, fail-stop tests and exact catalog checks; preserve target DB/RLS/API acceptance gaps. |
