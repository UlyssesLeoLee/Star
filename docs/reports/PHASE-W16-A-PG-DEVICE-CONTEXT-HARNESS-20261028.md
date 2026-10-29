# PHASE-W16-A-PG-DEVICE-CONTEXT-HARNESS-20261028

**Phase:** W16-A-PG-DEVICE-CONTEXT-HARNESS  
**Base:** W16-A `eeeda771bd16c6b3e608766064d9b91e8f310e45`  
**Target:** `dev` at `06466f7522f558cb3a27d108ec56b7d2e3caac83`  
**Scope:** PostgreSQL acceptance fixture and its disposable-cluster assertions only; no Rust, migration, Cargo, or WBS implementation edits.

## §0 Purpose

Prepare and exercise the existing authenticated Run-to-Worktree PostgreSQL runner against its isolated local PostgreSQL 18 cluster. The runner installs the OAuth and W16 device schema in dependency order, seeds active device identities for route-test actors, and proves the Directory lock role can take `FOR SHARE` while its restrictive UPDATE policy leaves `generation` unchanged. It also proves the refresh-token device/generation pair rejects a device ID with NULL generation using an admin transaction that is rolled back. For upgrade coverage, the fixture creates the historical weak same-name device/generation CHECK before applying the current W16 migration, exercising the migration's replacement branch. This validates only the local database/API boundary; it does not validate Neon/Vercel, K3s, or Multipass deployment.

## §1 Change matrix

| File / area | Change | Evidence state |
|---|---|---|
| `scripts/automation/phase9f2_binding_http_acceptance.py` | Adds test-only audit/SCD2 prerequisites, applies OAuth and Run Directory migrations, installs the historical weak same-name refresh device/generation CHECK before W16, then applies the current migration; seeds three device rows and probes ACL/RLS, FOR SHARE, denied UPDATE, and NULL-pair rejection. | Runner/static checks pass; legacy-constraint replacement, device PG tests, and HTTP acceptance await the rebase onto the source fix. |
| Device authorization fixture | Uses device UUIDs `a0000000-0000-0000-0000-000000000001`, `...0002`, and `...0003`, generation 1, for actor A/tenant A, developer B/tenant A, and actor C/tenant B respectively. Distinct IDs satisfy the W16 `(tenant_id, device_id)` primary key while retaining the actors' current Project/Run authorization assertions. | Source fixture implemented. The local run stopped at the reader gate before inserting these rows. |
| Directory lock privilege gate | Requires table-level `SELECT`, exactly one non-grantable column-level `UPDATE(generation)`, no other table/column ACLs, and the named restrictive UPDATE policy with false `USING` and `WITH CHECK`. Runtime/PUBLIC receive no direct OAuth ACL. The behavior probe uses the runtime login with `SET LOCAL ROLE`, expects `SELECT generation ... FOR SHARE` to succeed, then expects direct UPDATE to affect zero rows and leave generation unchanged. | Static assertions implemented. Real PostgreSQL execution awaits the backend fix commit. |
| Device-generation pair | Checks the applied constraint explicitly includes `device_generation IS NOT NULL`; as admin, attempts a refresh-token insert with non-null `device_id` and NULL generation inside a transaction, requires `check_violation`, and rolls the transaction back. No runtime privilege is added. | Static assertions implemented. Real PostgreSQL execution awaits the backend fix commit. |
| OAuth owner ACL and refresh isolation | Retains the exact OAuth owner table/column allowlist, direct-runtime/PUBLIC ACL rejection, and the three-row refresh-token tenant+actor SELECT/UPDATE isolation probe. | Static assertions retained; real PostgreSQL execution awaits the backend fix commit. |
| Audit and SCD2 prerequisites | Provides disposable test definitions needed to install the OAuth migration on a fresh cluster. The SCD2 function is intentionally a no-op because this runner does not test OAuth client SCD2 updates. | Fixture-only; not production audit or SCD2 evidence. |
| W16 report | Records the local PostgreSQL run, its failing gate, preserved evidence, and known limits. | Updated. |

The backend reader contract supplied for integration is `SET LOCAL ROLE star_engineering_directory_owner` in the short device-read transaction, with tenant and actor scope applied before the query. The runtime remains `NOINHERIT` and receives no direct device-table grant. The route-test lane owns the JWT helper changes; this harness seeds the matching identities.

## §2 Validation summary

The initial database run below is historical evidence from base `eeeda771`, before the backend owner-SELECT grant and the current review fixes were integrated. It must not be read as a result for the revised harness or accepted backend migration.

| Check | Result |
|---|---|
| Historical isolated PostgreSQL 18 initialization, startup, readiness and shutdown | PASS; loopback-only port `64637`; shutdown exit 0. |
| Historical OAuth base, Run Directory migration chain/fingerprints, and W16 migration | PASS; every recorded migration/fingerprint step exited 0. |
| Historical Directory owner role setup and existing lock-policy catalog check | PASS. |
| Historical `postgres-device-reader-role-membership-check` | PASS; runtime is a member of `star_engineering_directory_owner`. |
| Historical `postgres-device-reader-no-direct-acl-check` | PASS; explicit-OID query found no direct `SELECT` ACL for runtime login or `PUBLIC`. |
| Historical `postgres-device-reader-owner-select-check` | FAIL; owner role lacked `SELECT` on `public.oauth_device_bindings` in the pre-integration migration. |
| Historical authenticated HTTP phases | NOT RUN; the reader gate stopped the old runner before device seeding and HTTP calls. |
| Current Python AST syntax parse | PASS |
| Legacy weak same-name refresh pair constraint upgrade fixture | Implemented before W16 migration; real PostgreSQL replacement-path result NOT RUN. |
| Current runner `--help` dry run | PASS |
| Current runner unit tests | PASS; 8 tests. |
| Current `rtk proxy git diff --check` | PASS |
| Current CypherGraph Guardian `--changed --stamp` | PASS; one source file checked. |
| Current PostgreSQL Directory lock and denied-UPDATE behavior probe | NOT RUN; waiting for backend fix commit before rebase/DB run. |
| Current admin invalid-pair constraint and rollback probe | NOT RUN; waiting for backend fix commit before rebase/DB run. |
| Current device repository PostgreSQL tests and authenticated HTTP phases | NOT RUN; waiting for backend fix commit before rebase/DB run. |

The historical full command was `py -3 scripts/automation/phase9f2_binding_http_acceptance.py --run-unit-tests`; it returned exit 1 at `postgres-device-reader-owner-select-check`. PostgreSQL ran only on `127.0.0.1:64637`. The summary records `target_database_touched: false` and `multipass_or_docker_used: false`. The runner stopped the server successfully and preserved the failed cluster and logs under `.cache/wld0-binding-http-positive-20261023/runs/34be06a94028437c94bf02c5dea94fa3/`; the acceptance summary is in its `logs/a7748f6524094744ad5104264e22a11d/` directory. The role-membership and no-direct-ACL steps each have their own exit-0 result. The owner-role SELECT step failed with `engineering directory owner is missing scoped device-binding SELECT`. This run is on base `eeeda771`, whose checked-out W16 migration predates the separate code lane's owner SELECT grant; it is not a result against final integrated code. The unknown stopped Multipass instance was not touched.

Current static checks were run on the revised runner after adding the two review regressions. The harness remains uncommitted on its private lane; no rebase or PostgreSQL process was started for this revision.

## §3 Known gaps

1. In base `eeeda771`, the runtime membership predicate passed, and the separate direct-ACL query found no `SELECT` grant to the runtime login or `PUBLIC`.
2. The accepted lock-role ACL must be table-level `SELECT` plus column-level `UPDATE(generation)` only; the false restrictive UPDATE policy must still ensure zero affected rows. The revised real PostgreSQL probe is pending.
3. The old local run stopped before device rows were inserted and before authenticated HTTP phases. The revised full run remains pending the backend fix commit.
4. The legacy weak same-name CHECK replacement path, new device-generation pair catalog, and admin rollback checks are statically present but have not run against PostgreSQL; they do not grant extra runtime privileges.
5. The audit and SCD2 objects are local prerequisites; they do not establish the production platform audit schema or SCD2 behavior.
6. Missing, stale, and revoked device-token cases remain the route-test lane's responsibility. This fixture seeds only the active identities required by the successful and authorization-negative actor paths.
7. The local run did not inspect Neon/Vercel, K3s, or Multipass deployment state. Rust gates and execution-admission readiness were not evaluated; execution remains closed.

## §4 Subagent failure and takeover

The earlier harness worker disconnected before returning; its uncommitted runner changes, brief/report/WBS dispatch record, and `.cache` evidence were preserved. The coordinator added the revised PostgreSQL lock/denied-update and NULL-pair probes plus a pre-migration legacy weak CHECK fixture that exercises the replacement path; static checks were rerun. The historical host-local run completed migrations, role setup, membership, and direct-ACL predicates, then stopped at the old owner-role SELECT gate; the server shut down cleanly and its evidence remains preserved. The coordinator owns backend migration review; rebase and rerun only after the backend fix commit arrives. Revised HTTP acceptance remains NOT RUN.

## §5 Guardrails

1. Do not connect this fixture to production or an unapproved database.
2. Do not start, stop, or repurpose the unknown stopped Multipass instance.
3. Keep the audit schema and helper functions limited to the disposable test cluster.
4. Keep OAuth client SCD2 behavior out of the claims for this runner.
5. Keep the application runtime login non-superuser and `NOINHERIT`.
6. Never grant OAuth device/refresh-table access directly to the runtime login or `PUBLIC`.
7. Grant Directory owner table-level `SELECT` plus only column-level `UPDATE(generation)`; the restrictive false UPDATE policy must keep direct update rowcount at zero.
8. Do not add a broad table UPDATE grant to make row locking work.
9. Keep the read path on the Directory owner `SET LOCAL ROLE` boundary and verify `FOR SHARE` succeeds.
10. Keep tenant and actor context transaction-local before the device lookup.
11. Seed only the three explicit tenant/actor/device tuples required by the route phases.
12. Use distinct device UUIDs for actors sharing a tenant, as required by the table primary key.
13. Keep all seeded devices active, generation 1, and without revocation timestamps.
14. Use synthetic public-key fingerprints only; never place private key or proof material in the fixture.
15. Require the device/generation constraint to reject a non-null device ID with NULL generation; perform the negative insert as admin inside a rolled-back transaction.
16. Treat catalog assertions and source review as weaker evidence than the actual authenticated PostgreSQL route run.
17. Keep authorization-negative assertions intact and do not replace current Project/Run checks with device presence.
18. Do not infer production audit, RLS, or migration deployment from a disposable local cluster.
19. Keep execution admission closed until its independent runtime, sandbox, persistence, and authorization gates pass.

## §6 Signoff

| Role | Review |
|---|---|
| Architecture | 🟢 Mavis 接手终审 — device identity remains actor/tenant scoped; execution remains closed. |
| SRE Lead | 🟢 Mavis 接手终审 — historical local PostgreSQL stopped cleanly; revised database and HTTP acceptance remain NOT RUN. |
| Platform | 🟢 Mavis 接手终审 — static ACL/policy and new behavioral probes are recorded; actual backend migration execution is pending. |
| Review chair | 🟢 Mavis 接手终审 — AST, help, 8 unit tests, diff-check and CGG pass; historical pre-integration owner-SELECT failure is retained. |
| Product manager | 🟢 Mavis 接手终审 — this interim harness result does not mark the full acceptance phase complete. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the OAuth/W16 runner migration order, scoped device fixtures, least-privilege catalog gate, validation performed, and unrun PostgreSQL acceptance. |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Replace the endpoint-based NOT RUN status with the requested local PostgreSQL 18 run; record 8 passing unit tests, successful migrations/role setup, the pre-integration owner SELECT failure, HTTP phases not reached, and preserved evidence paths. |
| v0.3 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Split the reader catalog gate into independent membership, direct-ACL, and owner-SELECT steps; record individual exits, the updated local run and preserved evidence path, and keep HTTP phases NOT RUN. |
| v0.4 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add exact Directory table/column ACL, false restrictive UPDATE, real `FOR SHARE` plus zero-row unchanged-generation behavior, and admin rollback rejection of the invalid device/generation pair; preserve the historical failing run and report current static checks while awaiting backend commit. |
| v0.5 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add a legacy weak same-name CHECK before applying the current W16 migration to exercise replacement logic; rerun AST, --help, 8 unit tests, CGG, and diff checks. Real migration/HTTP behavior remains NOT_RUN until the source fix is rebased. |