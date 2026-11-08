# Brief: 9F4J.3 External Login Identity Guard

**Task:** WBS-9F4J-3 external-login identity follow-up
**Owner:** assigned implementation agent
**Base:** local `dev@c00a3784fb0464b1f1f13208a4e0a339d2226ea5`
**Scope:** PostgreSQL grants acceptance harness only; no production grants or schema changes.

## Objective

Close the self-role false-positive risk recorded in current WBS §23.21. The cross-domain acceptance test can be run by itself; its `pg_has_role(..., 'star_bi_runtime', 'MEMBER')` assertion must not allow a connection that authenticates directly as the `star_bi_runtime` group role to stand in for a restricted login principal.

## Required change

- Make the cross-domain test independently validate that `session_user` is a restricted LOGIN principal distinct from `star_bi_runtime`, that `current_user` has not been switched to the group role, and that `star_bi_runtime` exists with `NOLOGIN` and the expected runtime attributes.
- Keep the required BI runtime membership check, non-allowlisted `star_*` role membership/SET checks, and exact transport function EXECUTE allowlist checks.
- Return actionable, identity-only failure messages; never include the database URL, password, certificate contents, or other credentials.
- Preserve `#[ignore]`, exact disposable database identity, loopback host, non-default port, and fail-closed behavior. Do not create or start a database or VM. The configured database URL is absent at lane start, so live PostgreSQL acceptance remains `NOT_RUN`.
- Update this brief, a follow-up phase report, and the dispatch JSONL. Keep the report's §0–§7 structure and describe this as a test-harness fix, not production grant proof.
- Update the existing Rust Cypher header according to the repository's graph-header rule.

## Validation

Use Rust 1.99.1 and `-j 4` for the focused test-target `--no-run` build; run rustfmt on only the changed Rust file, the documented Cypher Graph Guardian check, and `git diff --check`. Do not execute the ignored PostgreSQL test without a configured, identity-checked disposable database.

## Ownership boundary

Only edit the four registered files: this Rust integration test, this brief, `docs/reports/PHASE-9F4J-3-EXTERNAL-LOGIN-IDENTITY-20261108.md`, and `docs/reports/WBS-DISPATCH-9F4J-3-EXTERNAL-LOGIN-IDENTITY-20261108.jsonl`. Do not modify the canonical WBS, migrations, Cargo files, production source, `main`, remotes, D:\Star user files, or other worktrees. Commit only this manifest and leave local-dev integration to the coordinator after independent review.
