# PHASE-WLD0 Run-to-Worktree Binding API Implementation Report

## §0 Purpose

Record the reviewed WLD-0 source implementation and its current validation boundary. This report does not close database acceptance, production deployment, or the broader Worktree lifecycle phase.

## §1 Change matrix

| Area | Change | Status |
|---|---|---|
| API | Added authenticated Project/Run-scoped Worktree bind, reassign, and detach routes with current authorization, CAS, idempotency, per-Worktree serialization, SCD2, audit, and same-transaction Outbox. | Source implemented; target DB not accepted. |
| Hostless Run | Remote Run repository and Branch metadata may be absent; the binding still records the local Worktree repository identity. | Covered by unit-level transition/compatibility checks. |
| Migration | Added the forward event-type constraint migration for bind/reassign/detach Outbox events. | Source only; not applied to a target database. |
| Tests | Added unauthenticated route boundary coverage and unit coverage for transitions, hostless metadata, idempotency scope, lock identity, and advisory-lock SQLSTATE mapping. | Targeted route/unit checks run; database-backed path remains open. |
| Documentation | Synchronized requirements v6.87, basic design v6.86, Group DD v5.09, and WBS snapshots. | Candidate changes; integrate with the source lane. |

## §2 Validation summary

- Rust 1.99.0 `cargo check --package star-api-rest --all-targets -j 4` passed after the lock-error mapping change and final test assertion; existing warnings are in other modules/crates.
- On the rebased candidate branch, `cargo test --package star-api-rest --all-targets -j 4` passed after the final lock-error mapping fix: 261 passed, 0 failed, 2 ignored. The ignored cases require a disposable PostgreSQL advisory-lock fixture and the Schedule canonical-directory ACL fixture.
- The focused advisory-lock regression test also passed after the fix: 1 passed, 0 failed, 146 filtered out.
- Integration lineage: API source commit `745cab56288b4fa3105dcf620b082bc1a8794b31` was cherry-picked as candidate commit `3bb6037b`; the candidate is based on the current `dev` commit `6f8d5b8f`. The shared `dev` checkout has not yet been advanced.
- Rust 1.99 rustfmt check for the changed Rust files passed with child-module traversal disabled for `group_api.rs`; the package-wide `cargo fmt --check` found pre-existing formatting differences in unchanged `group_api/work_items.rs`. That file was not reformatted.
- CypherGraph Guardian passed for 3 changed source files. `git diff --check` passed.
- CodeRabbit CLI review could not run: its Windows wrapper invoked `/root/.local/bin/coderabbit` through Ubuntu and failed with `Permission denied`. A separate static review found no P0/P1 issue and identified the lock-timeout classification issue; that P2 issue was corrected and the focused regression test passed.
- No PostgreSQL connection was used. The migration, application-role grants/FORCE RLS, authenticated writes, rollback, durable idempotency replay, concurrent writers, and ignored advisory-lock PostgreSQL test were not run.

## §3 Known gaps

1. The forward migration has not been applied to the deployment database; schema constraint compatibility, runtime grants, RLS policy behavior, and trigger behavior remain unverified there.
2. Database-backed authenticated writes, rollback, exact replay persistence, and concurrent bind/reassign/detach execution have not been validated.
3. The current source slice establishes a binding record only. It does not reserve Host or cluster capacity, prove K3s/Runtime/sandbox readiness, or enable Task CLI, Agent, Schedule, or execution admission. Responses and Outbox facts keep `execution_admission_available` false.
4. Historical binding reconciliation and the impact of stale or duplicated pre-existing ownership rows still require the WLD-0 acceptance pass.
5. The post-fix candidate all-targets test is a source-level test result; the two ignored fixture-dependent cases remain open. No target PostgreSQL or production identity evidence is inferred from the passing suite.

## §4 Worker failure and coordinator takeover

The assigned worker reached its usage limit before final self-review and commit. Its uncommitted changes remained in the isolated `codex/wld0-binding-api` worktree. The coordinator checked the source, schema, hostless semantics, CypherGraph headers, and test evidence; corrected the advisory-lock SQLSTATE mapping; received independent static review; ran focused post-fix validation; and owns the eventual integration. No worker commit was assumed from a reported status.

## §5 Guardrails

- Navigation and ownership remain Project → Engineering Run → Worktree → Branch; a remote host/Branch is optional metadata for a hostless Run.
- Project/Run authorization, binding version, Project-binding version, Worktree ownership, idempotency, audit, and owner Outbox are checked in the binding owner path.
- Do not treat a successful source-level mutation as production DB acceptance or execution readiness.
- Do not enable Agent/CLI/Schedule execution from this binding slice; admission remains false until the independent reservation, sandbox, Hook, cancellation, and validation gates pass.
- Preserve the two-record Multipass test inventory cap, including stopped/deleted records, in later infrastructure acceptance.
- Merge only after reviewing the final candidate diff and rechecking dev HEAD, working tree, and in-progress Git operations.

## §6 Signoff

| Role | Review state |
|---|---|
| Architecture | Mavis review of this source slice; broader WLD closure remains open. |
| SRE Lead | Mavis review; target DB deployment and recovery evidence remain open. |
| Platform | Mavis review; grants/RLS and runtime wiring remain open. |
| Review chair | Mavis review; focused test and static review recorded above. |
| Product manager | Mavis review; product execution remains disabled. |

## §7 Revision history

| Version | Date | Revision author | Change |
|---|---|---|---|
| v0.1 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record WLD-0 API source implementation, review, exact validation boundaries, coordinator takeover, and remaining DB/production gates. |
| v0.2 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record candidate rebase to dev, post-fix all-targets check and test (261 passed, 2 ignored), and keep PostgreSQL/RLS/production gates open. |
