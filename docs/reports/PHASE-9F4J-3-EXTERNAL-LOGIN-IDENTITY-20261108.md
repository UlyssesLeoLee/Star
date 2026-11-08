# Phase Report: 9F4J.3 External Login Identity Guard v1.0

> **Status:** Local test-harness change, focused validation, and independent coordinator review complete; not integrated; PostgreSQL acceptance **NOT_RUN**.
>
> **Date:** 2026-11-08
>
> **Base:** `dev@c00a3784fb0464b1f1f13208a4e0a339d2226ea5`

## §0 Purpose

Close the standalone cross-domain grants test's self-role false-positive risk. A positive `pg_has_role(..., 'star_bi_runtime', 'MEMBER')` result must not let the group role itself stand in for a dedicated BI login. This is a test-harness change only; it does not change production grants or database schema.

## §1 Change Matrix

| Registered path | Change |
|---|---|
| `crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs` | The cross-domain test now invokes the runtime-principal guard. The guard requires `current_user = session_user`, both to be distinct from `star_bi_runtime`, validates the session principal is a restricted LOGIN, and validates the BI group is a restricted NOLOGIN role. |
| `docs/briefs/9F4J-3-EXTERNAL-LOGIN-IDENTITY-20261108.md` | Records scope, acceptance criteria, and safety boundaries. |
| This report | Records implementation, exact verification state, and remaining runtime gates. |
| `docs/reports/WBS-DISPATCH-9F4J-3-EXTERNAL-LOGIN-IDENTITY-20261108.jsonl` | Records dispatch and validation receipts. |

The query still checks BI runtime membership, non-allowlisted `star_*` membership/SET paths, and EXECUTE on the seven exact transport signatures. The test remains opt-in and ignored.

## §2 Verification Summary

| Check | Result | Evidence / limits |
|---|---|---|
| Rust toolchain | PASS | Rust 1.99.1 requested explicitly. |
| Focused test-target compile | PASS | `cargo +1.99.1 test -p bi-owner-runtime --test task_event_inbox_runtime_grants_pg --no-run -j 4 --locked`; exit 0, test executable produced in 15.15 s. Four existing `async_fn_in_trait` warnings come from untouched `task_event_receiver.rs`. |
| Targeted rustfmt | PASS | `rustfmt +1.99.1 --edition 2021 --check crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs`; exit 0 after the final source/header edit. |
| Cypher Graph Guardian | PASS | `cgg_guard.py --changed --stamp`; one Rust source file checked and header hash refreshed. |
| `git diff --check` | PASS | Exit 0 on the final lane changes. |
| PostgreSQL grants acceptance | **NOT_RUN** | The dedicated database URL is unset. The ignored test was not executed; no database or VM was created or started. |
| Integration | NOT_INTEGRATED | Independent coordinator review and local-dev integration remain pending. |

## §3 Known Gaps

1. No live PostgreSQL session verified the identity assertions or role grants. The test remains `NOT_RUN` until a dedicated database with the required identity and migrations is available.
2. This lane does not prove production grants, RLS, migrations, transport delivery, or runtime behavior.

## §4 Subagent Failure / Takeover

- The implementation was performed in the registered isolated lane; there was no nested subagent dispatch or failure requiring takeover.
- Independent coordinator review of the registered Rust change and evidence found no remaining issue. The lane remains not integrated; live PostgreSQL acceptance remains unverified.

## §5 Guardrails

1. Only the four MAGOS-registered paths are in scope.
2. The database test remains `#[ignore]`; no connection secret, certificate, or URL is printed.
3. The cross-domain test independently validates the authenticated login and group-role identity before privilege assertions.
4. The exact function allowlist and existing membership/SET denial checks remain intact.
5. No PostgreSQL, VM, migration, Cargo manifest/lockfile, production code, canonical WBS, main, remote, or user file was changed.
6. Compile-only evidence is not database acceptance; PostgreSQL remains `NOT_RUN`.
7. This lane is local and unintegrated; no push, release, or deployment occurred.

## §6 Sign-Off

| Role | State |
|---|---|
| Architecture | Implementation self-review complete; live role matrix remains unverified. |
| SRE Lead | No database or VM was started; acceptance remains `NOT_RUN`. |
| Platform | Test harness identity guard is scoped to the existing BI runtime contract. |
| Review Chair | Independent coordinator review passed on the registered Rust diff; no findings. Integration remains pending. |
| PM | Local lane only; not integrated; PostgreSQL acceptance remains NOT_RUN. |

Revision author: Ulysses (12-role company per DEC-008) — Mavis review complete. No human approval or deployment acceptance is claimed.

## §7 Revision History

| Version | Date | Reviser | Change | Trigger |
|---|---|---|---|---|
| v1.0 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis handoff review pending | Created the follow-up report for the external-login identity guard; recorded compile as in progress and live PostgreSQL acceptance as NOT_RUN. | WBS §23.21 external grant-harness self-role risk |
| v1.1 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis handoff review pending | Recorded the focused Rust 1.99.1 test-target compile, final formatting/graph/whitespace checks, and retained live PostgreSQL as NOT_RUN. | Focused validation completed |
| v1.2 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis review | Independently reviewed the exact registered source diff, confirmed the external LOGIN identity guard and retained the NOT_RUN PostgreSQL boundary; no findings. Integration remains pending. | Coordinator review |
