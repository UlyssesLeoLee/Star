# WLD3 Hostless Schedule Rule Authorization — Implementation Report

**Date:** 2026-10-21
**Lane:** `wld3-hostless-schedule-auth-20261021`
**Status:** Source and design slice implemented; Rust and PostgreSQL acceptance gates remain open.

## §0 Purpose

Allow an active Engineering Run with no remote host Branch to list, read, configure, and revise its Schedule Rule when the current Project and Run grants authorize it. Preserve Branch-backed behavior and keep Occurrence creation, admission, and execution fail-closed until a Branch-bound execution path is implemented and verified.

## §1 Change matrix

| Area | Change | State |
|---|---|---|
| Schedule API | Authorization now uses Project/Run grants for all Runs and requires active Branch metadata/grant only when the Run has a Branch. It retains active Run checks, writer checks, the Project advisory lock, immutable run-as identity, idempotency, audit, and same-transaction Outbox. Optional Rule Branch identity comes from the authorized Run scope, never from the Worktree binding; target validation still binds the current Worktree, Task, local repository, Profile, and HookSet. | Implemented; not Rust/DB tested |
| Persistence | Added a forward migration that makes only `automation.schedule_rule_revision.branch_id` nullable and updates the run-as authorization lock helper. Occurrences retain their non-null `target_snapshot` JSONB; there is no standalone occurrence `branch_id` column or SQL JSON-key constraint, and this migration does not alter Occurrence schema. | Implemented; migration replay not executed |
| Domain/runtime boundary | `ScheduleTargetV1.branch_id` is optional for configured Rules. A bounded parser/timezone recurrence check can validate configuration without creating an occurrence. Occurrence snapshots, actual materialization, admission command validation, binding checks, policy reads, and receipt/demand writes still require a real Branch and reject absence through the Rust domain/owner path. No synthetic Branch is created. | Implemented; focused Rust tests not executed |
| Fixtures/runner | Added hostless authorized, Project-revoked, Run-revoked, and archived Run ACL fixtures; the runner includes the host-independent directory and new Schedule migration in its repeat-apply sequence. Branch-backed fixtures remain. Runtime PostgreSQL test target explicitly uses a Branch. | Implemented; DB harness not executed |
| Canonical design | Updated Schedule SRS v0.64, Schedule API DD v0.76, Group DD v5.16 §8.11, and Basic Design v6.93 §§16.25–16.26 to distinguish hostless Rule configuration from Branch-required occurrence/admission/runtime. | Updated |

Changed code and fixture files: `crates/star-api-rest/src/group_api/schedule_rules.rs`, `crates/domain-automation/src/schedule.rs`, `crates/run-owner-runtime/src/schedule_admission.rs`, `crates/run-owner-runtime/src/schedule_policy.rs`, `crates/run-owner-runtime/tests/schedule_admission_pg.rs`, `scripts/automation/phase9f3_schedule.py`, and `db/migrations/2026-10-21-schedule-hostless-run-authorization.sql`. The Group DD cross-reference now points to the current Basic Design and Schedule API DD versions.

## §2 Verification summary

| Command/check | Result |
|---|---|
| `rtk run python -m py_compile scripts/automation/phase9f3_schedule.py` | Passed (exit 0) |
| `rtk git diff --check` | Passed (exit 0) |
| `rtk run python .../cgg_guard.py --changed --stamp` | Passed; 6 source files checked |
| `rtk cargo fmt --check` | Not run: RTK could not find/spawn `cargo` on PATH |
| `rtk cargo test -p domain-automation hostless_rule_can_be_configured_but_cannot_be_materialized -j 1` | Not run: Cargo unavailable |
| `rtk cargo test -p run-owner-runtime schedule_admission_requires_remote_branch_for_runtime_scope -j 1` | Not run: Cargo unavailable |
| PostgreSQL 18.6 migration replay and focused DB tests | Not run: PostgreSQL 18 binaries are absent from the checked standard installation path; no target database acceptance is claimed |
| CodeRabbit second review | Unavailable: CLI launch failed with `/bin/bash: line 1: /root/.local/bin/coderabbit: Permission denied`; local diff review continued |

No indexed `.codegraph/` project was available for this lane. The impact check used targeted source and migration searches, exact call-site inspection, and the passing Cypher header guard; this is not an exhaustive repository graph audit.

## §3 Known gaps

1. Rust 1.99.0 formatting, compilation, unit tests, all-target checks, and Clippy remain unverified because Cargo/rustfmt are unavailable in this environment.
2. The new migration has not been applied twice on PostgreSQL 18.6. Hostless authorization, Rule `branch_id` nullability, the unchanged non-null Occurrence `target_snapshot`, ACL revocation, and lock-race cases are fixture/code assertions only until the disposable database harness runs; PostgreSQL does not independently constrain the Branch key inside the JSON snapshot.
3. Target database migration owner, application roles/grants, FORCE RLS, rollback, and deployment order remain separate acceptance gates.
4. The Schedule worker, per-trigger/retry/resume authorization service, admission consumer/writer, infrastructure reservations/readiness, executor, task CLI, and BI integration are not implemented or enabled by this slice.
5. Schedule materialization and all execution-facing runtime paths remain Branch-required; a hostless configured Rule cannot produce an Occurrence or start work.

## §4 Subagent failure / takeover

No child agents were dispatched in this lane. No delegated output required takeover.

## §5 Guardrails

- `run_as_actor_id` remains immutable across Rule revisions. Each later unattended trigger/retry/resume must reauthorize the same actor against current Project/Run grants; that runtime service remains unimplemented.
- The shared Project lock continues to serialize Project/Run ACL mutation with API authorization. Branch-backed Runs still require a current active Branch row, active revision, and current Branch grant.
- Optional `branch_id` applies only to the configured Rule target and Rule revision storage/read model. Occurrence storage retains a non-null JSON target snapshot, with Branch presence checked by the Rust domain/materializer path; the DB has no standalone occurrence Branch column or JSON-key constraint. Occurrence snapshots, admission, binding revalidation, policy lookup, demand creation, and result receipt still fail closed when Branch is absent.
- The API still validates current Task/Worktree/local repository/Profile/HookSet bindings; no validation was removed to permit hostless configuration.
- Run-owner admission checks a missing Branch as `BindingUnavailable` before opening its owner transaction; current admission and PostgreSQL test fixtures explicitly carry a real Branch.
- This report covers only hostless Schedule Rule configuration/authorization. It does not claim worker, admission, or executor readiness.

## §6 Sign-off

| Role | State |
|---|---|
| Architecture | Implementation follows the approved hostless Rule / Branch-required execution boundary; coordinator integration review pending |
| SRE Lead | Target PostgreSQL, grants, migration replay, and operational acceptance pending |
| Platform | Rust 1.99.0 build/test and runtime acceptance pending |
| Review host | Local self-review completed; independent CodeRabbit review unavailable as recorded above |
| PM | Scope and non-production state recorded; end-user/production acceptance pending |

## §7 Revision history

| Version | Date | Change | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-21 | Record the hostless Schedule Rule authorization/configuration slice, preserved Branch-required execution boundary, design synchronization, and unrun Rust/DB gates. | `docs/briefs/wld3-hostless-schedule-auth-20261021.md` |
