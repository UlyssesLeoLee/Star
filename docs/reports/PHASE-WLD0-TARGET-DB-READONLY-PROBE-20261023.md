# PHASE-WLD0-TARGET-DB-READONLY-PROBE-20261023

## §0 Purpose

Add a fail-closed, read-only PostgreSQL probe for future WLD-0 target-database evidence. The probe does not identify or connect to a target without explicit libpq service and four matching identity pins.

## §1 Change matrix

| Path | Change | Acceptance boundary |
|---|---|---|
| `scripts/automation/wld0_target_database_probe.py` | New standard-library-only CLI; fixed psql script; pinned identity; bounded ledger, role-membership/ownership, direct grant, table-privilege, relation, RLS and lock-grant catalog snapshot | Read-only target observation only; no migration application or acceptance claim |
| `scripts/automation/test_wld0_target_database_probe.py` | New mocked-subprocess unit coverage for validation, command shape, error secrecy, timeouts and unknown/blocking states | Tests exercise local command/error handling, not PostgreSQL |
| `docs/automation-design.md` | Register §4.101 contract and open gates | Documents behavior and limitations |
| `scripts/automation/registry.md` | Register v1.09 entry | Index only |
| `docs/reports/PHASE-WLD0-TARGET-DB-READONLY-PROBE-20261023.md` | This phase report | Does not replace WBS or target deployment evidence |

## §2 Verification summary

- `python scripts/automation/test_wld0_target_database_probe.py -v`: 15/15 passed. All simulated psql responses came from mocks; this verifies local validation, command construction and fail-closed response handling only.
- Python AST parse of `scripts/automation/wld0_target_database_probe.py` and `scripts/automation/test_wld0_target_database_probe.py`: 2/2 passed.
- `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp`: PASS, 2 source files checked.
- `git diff --check`: PASS. `git status --short` shows exactly five modified/untracked paths from the manifest; its brief is already present in the lane's base commit.
- `Get-Command psql` and `Get-Command pg_isready`: both missing. `python scripts/automation/wld0_target_database_probe.py` returned `{"reason":"missing_or_invalid_service_name","status":"BLOCKED"}` before process discovery, confirming that an unconfigured invocation does not try a connection.
- CodeGraph coverage tools are unavailable in this lane; the existing engineering directory grant/RLS acceptance SQL and migration definitions were read directly as a narrow source fallback. This was not treated as a completion gate.
- No target DB connection was attempted. The dispatch environment check found no configured libpq service or pinned target identity; the current host check confirms `psql` and `pg_isready` are missing. The no-argument smoke invocation returned `BLOCKED` with `missing_or_invalid_service_name`, before psql discovery.
- No migration, target schema, Run API, WBS row, VM, or database service was changed or started.

## §3 Known gaps

1. No production target identity or libpq service has been provided; this host also lacks `psql` and `pg_isready`. No actual target catalog, role, grant, ledger, or RLS result exists, and no connection was attempted.
2. The known `schema_migrations` format records migration names and timestamps but not source checksums. The probe does not establish checksum equivalence, a complete expected predecessor/successor chain, or a complete schema fingerprint. Migration replay and ambiguous-commit classification remain `BLOCKED`.
3. Catalog privileges and the read-only role switch do not establish authenticated HTTP bind/reassign/detach behavior, transaction rollback, concurrent CAS, audit/Outbox deltas, API deployment membership, or tenant-scoped RLS behavior.
4. A disposable PostgreSQL fixture or mocked subprocess result is not target/production evidence.
5. The required psql/libpq service configuration and target identity must be supplied by an authorized operator before any future connection run.

## §4 Subagent failure handoff

No subagent failure or orphaned output is recorded. If continuation is needed, use the committed probe, focused test, design entry (§4.101), registry v1.09, and this report. Revalidate the checkout and target identity before any future probe; do not infer target configuration from environment variables.

## §5 Guard rules

- Keep the probe read-only: fixed catalog SQL, `READ ONLY` transaction, explicit service name, pinned database/login/IP/port, bounded statement/process timeouts and output, and no arbitrary command/SQL/URL/migration input.
- Send psql stderr to `DEVNULL`; do not print credentials, inherited psql environment, raw stderr, or full command arguments. Psql errors must remain sanitized `BLOCKED` results.
- Missing psql/service/identity, unknown ledger format, truncated evidence, or unavailable RLS fields fail closed as `BLOCKED` or `UNKNOWN`.
- Do not interpret local fixtures, mocks, catalogs, or role switching as authenticated binding acceptance or migration-replay safety.
- Preserve the additive lane boundary: no edits to the existing migration runner, migrations, APIs, WBS, or other worktrees; no VM/database startup, rebase, merge, push, or cleanup.
- Use only the standard library. Keep both new Python source files' CypherGraph headers reconciled and stamped.
- Commit author is `Ulysses <ulysses@mavis.local>` per repository authorization. Commit is not integration or remote publication.

## §6 Review/signature roles

| Role | Review status |
|---|---|
| Architecture | Mavis接手审核; scope is additive and production gates remain open |
| SRE Lead | Mavis接手审核 for this bounded probe; no independent production operation was performed |
| Platform | Mavis接手审核; no target runtime or database was changed |
| Review host | Mavis接手审核; focused local checks only |
| PM | Mavis接手审核; WLD-0 remains open |

## §7 Revision history

| Version | Date | Revision author | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add the read-only target probe, bounded tests and explicit target/replay/authenticated-binding gaps | WLD-0 target database evidence remained uncollected |
