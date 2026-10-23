# PHASE-WLD0 Binding HTTP Acceptance Report

## §0 Purpose

This report records the bounded Run-to-Worktree binding HTTP acceptance lane, originally branched from `dev@8578d5048e6d6d10095d3f8797168471cfd632b8`. The target passed on `dev@e48dbb8aa2a0eaf85bd6412713b55e5f715539b8`; the lane was subsequently rebased onto `dev@5f01e096f4cca4f4a35919d2e9ced6e9f8469783` after dev advanced by a WLD3 brief-only commit that did not touch this lane's files. No code changed in that last rebase. The lane does not claim authenticated database mutation or close WLD-0.

## §1 Change matrix

| Area | Change | Status |
|---|---|---|
| Test JWT | Reuse the existing fixed route-test RSA fixture and public `issue_token` API for signed bearer tokens. | Implemented; exercised by the passing Rust 1.99.0 focused target. |
| HTTP scope boundary | Valid-JWT PUT/DELETE cases each omit one of `project:write` or `worktree:manage`; assert 403 and private response headers. Handler source checks both scopes before `begin_write`; requests use a lazy pool and do not reach database access. | Passed: all four missing-scope requests returned 403 with required headers. This is HTTP precondition acceptance only. |
| Production API and migration | This lane did not change production routes, schema, migrations, dependencies, or authorization behavior. The rebased dev base includes the coordinator's two-line Rust compatibility correction in `worktree_lifecycle.rs`; this lane did not edit it. | No production changes owned by this lane. |
| Database acceptance | No actual bind/replay/conflict request was sent to PostgreSQL. The existing owner-role DDL fixture is migration/privilege acceptance and does not seed the binding route or execute this API as its runtime role. | Open; no DB was started or contacted. |

## §2 Validation summary

- `rtk proxy C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin\rustfmt.exe --check --edition 2021 crates\star-api-rest\tests\engineering_run_worktree_binding_routes.rs` passed.
- `rtk proxy 'E:\DevCache\cargo\bin\rustfmt.exe' +1.99.0 --check --edition 2021 crates\star-api-rest\tests\engineering_run_worktree_binding_routes.rs` passed (exit 0).
- Diagnostic Rust 1.98.1 command: `$env:RUSTC='C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin\rustc.exe'; rtk proxy C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin\cargo.exe test --package star-api-rest --test engineering_run_worktree_binding_routes -j 4 --ignore-rust-version`. It stopped before tests on unchanged base code with E0631 at `crates/star-api-rest/src/group_api/worktree_lifecycle.rs:815`; the session API did not return a numeric status for this diagnostic run.
- On the original base, Rust 1.99.0 returned exit 1 before tests on E0631 at `worktree_lifecycle.rs:815`; this historical blocker was in source outside the lane manifest and was corrected on dev by the coordinator.
- After rebasing onto `dev@e48dbb8aa2a0eaf85bd6412713b55e5f715539b8`, Cargo reported `cargo 1.99.0 (5f94df478 2026-08-27)` and rustc reported `rustc 1.99.0 (b940084d7 2026-09-28)`. `rtk proxy 'E:\DevCache\cargo\bin\cargo.exe' +1.99.0 test --package star-api-rest --test engineering_run_worktree_binding_routes -j 4` exited 0. Exact target result: `running 2 tests`; both `run_worktree_binding_mutations_require_authentication` and `run_worktree_binding_mutations_require_both_scopes_before_database_access` passed; `2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (tests finished in 0.08s; build profile finished in 2m 14s). Cargo emitted non-fatal warnings in shared crates; no warning was promoted to an error.
- The lane was then rebased to `dev@5f01e096f4cca4f4a35919d2e9ced6e9f8469783`; the only intervening dev commit adds `docs/briefs/WLD3-RUST-GATE-UNBLOCK-20261022.md` (verified by `git diff --stat e48dbb8..5f01e096`). The focused target was not rerun after this documentation-only commit because no source or claimed lane file changed.
- CypherGraph Guardian passed for the one changed source file. `rtk proxy git diff --check` passed (Git emitted only the existing CRLF-to-LF normalization warning). Final worktree status is recorded in the append-only log.
- PostgreSQL and target DB checks were not run. The focused pass proves only that authenticated HTTP requests missing a required scope are rejected before database access; it does not prove authenticated database writes, RLS, grants, replay, or transaction behavior.

## §3 Known gaps

1. No disposable fixture in this lane covers authenticated bind, exact replay, idempotency-key conflict, rollback, CAS races, audit/Outbox atomicity, or target-equivalent runtime-role RLS through the HTTP router. The available WLD-0 DDL owner-role harness does not execute those API mutations.
2. Target DB grants/RLS, migration ledger, history reconciliation, rollback/replay/races, and all execution-readiness gates remain open. Task execution remains fail closed.
3. The two-test target does not exercise the valid-scope path beyond the pre-database authorization gate. Rust 1.98.1 output and the earlier Rust 1.99.0 E0631 are historical diagnostics; the latter is resolved on the rebased dev base and the latest Rust 1.99.0 focused target passes.

## §4 Worker failure and coordinator takeover

No worker failure or coordinator takeover occurred in this lane. The worker remains responsible only for the files named in the assigned brief and preserves the isolated worktree for parent review.

## §5 Guardrails

- Scope tests use valid signed JWTs and exercise both write methods against the registered Group router.
- Both valid signed-token cases missing one required scope returned 403 before the production handler calls `begin_write`; the lazy pool makes database access unnecessary for this test. This does not exercise the authenticated mutation path.
- No target/system database URL was read, used, or modified; no PostgreSQL instance was started.
- No production route, authorization implementation, migration, target data, or execution-admission state was changed.
- Passing source checks cannot be promoted to database acceptance, target role/RLS acceptance, or production readiness.
- Worktree remains isolated and must be integrated only after parent review and applicable gates.

## §6 Signoff

| Role | Review state |
|---|---|
| Architecture | Mavis review of bounded scope; WLD-0 remains open. |
| SRE Lead | Mavis review; no disposable or target database was exercised. |
| Platform | Mavis review; application-role grants and target RLS remain open. |
| Review chair | Rust 1.99.0 focused HTTP target passed 2/2; coordinator review remains pending. Database acceptance remains open. |
| Product manager | Mavis review; no production mutation or execution enablement is inferred. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Add signed-JWT scope-boundary test design and record the blocked Rust 1.98.1 diagnostic build and remaining database acceptance gaps. |
| v0.2 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the Rust 1.99.0 focused command and its exit 1 on the unchanged E0631 compile blocker; record CypherGraph Guardian and `git diff --check` passing, with the HTTP assertions still unexecuted. |
| v0.3 | 2026-10-23 | Ulysses (DEC-008; Mavis review) | Record Rust 1.99.0 focused HTTP target passing 2/2 on dev@e48dbb8, then rebase onto documentation-only dev@5f01e096; database and RLS acceptance remain open. |
