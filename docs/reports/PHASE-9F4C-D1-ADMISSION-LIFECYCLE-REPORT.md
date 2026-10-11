# PHASE-9F4C-D1 — Schedule Admission Attempt Lifecycle

## 0. Purpose

Implement and document the smallest safe lifecycle prerequisite for Schedule→Run admission: persist logical attempt ordinals and provisional timestamps, expose their state in authenticated Run history, and keep execution closed until owner consumers exist.

## 1. Change matrix

| Area | Change | Result |
|---|---|---|
| Database | Add Run creation/attempt fields, unique per-occurrence logical attempt index, separate actual start invariant, and confirmation/compensation event vocabulary. | Repeatable migration and disposable SQL invariant gate. |
| API/UI | Project attempt/state/timestamps and paginate by coalesced start/creation time; show lifecycle labels in Task Run history. | Focused Rust cursor and lifecycle-state tests plus package all-target compile. |
| Runner/docs | Extend the PostgreSQL schema runner and synchronize requirement, basic design, detailed design, W/T/M, plan, automation index, and this report. | Sync script is exact-anchor guarded and idempotent. |

## 2. Verification summary

| Gate | Result |
|---|---|
| Rust 1.99.0 `cargo check -p star-api-rest --all-targets -j 4` | Passed; existing warnings remain. |
| `cargo test -p star-api-rest --lib task_runs::tests -j 4` | Passed 5/5. |
| PostgreSQL 18.6 schema-only runner | Passed after migration idempotency repair; migration applied twice, grants/RLS and D1 SQL invariants passed; 0 VM and unchanged Multipass inventory. |
| Rust 1.99.0 `cargo test --locked -p run-owner-runtime --test capacity_result_jetstream_pg --no-run -j 4` | Passed; compiles the ignored PostgreSQL/NATS target only. No live NATS integration was enabled. |
| `python scripts/automation/registry_check.py` | Exit 0, zero errors; the repository-wide registry checker emitted 197 existing unregistered-script warnings. |
| Frontend Vitest | Not run; `frontend/node_modules` is absent. |
| Rustfmt checks | Workspace/package checks report formatting differences only in unchanged Rust files; no touched Rust file appears in those diagnostics, and no workspace-wide formatting was applied. |
| `git diff --check` / Cypher Guardian | Recorded after final synchronization and stamping. |

The disposable PostgreSQL audit artifact is `target/phase9f4c-d1-admission-lifecycle-retry2/audit.jsonl`. The initial DDL attempt exposed non-idempotent index creation; the migration now uses `CREATE UNIQUE INDEX IF NOT EXISTS`, and the full retry passed.

## 3. Known gaps

- Schedule admission Inbox and Run confirmation/compensation consumers are not implemented by this slice.
- Production service identity, target DB migration/grants, current-ACL recheck, Project/Run quota reservation and Infrastructure-demand confirmation remain open.
- NATS-backed owner message delivery, provider/readiness producer, Multipass/K3s operation, sandbox readiness, Hook enforcement, BI/Benchmark, and task execution remain disabled and fail-closed.
- Frontend dependency installation and Vitest were not run. Repository-wide formatting has unrelated baseline differences.

## 4. Subagent failure handoff

No implementation subagent was dispatched for this slice. The existing read-only owner-consumer audit brief completed its bounded review; there is no failed worker output to hand off.

## 5. Guardrails

1. Run, Schedule and Infrastructure write only owner-local state.
2. No cross-owner transaction or direct cross-owner table write.
3. Provisional Run creation is not execution start; `started_at` is never inferred from row creation.
4. Infrastructure demand requires Schedule confirmation and remains unwired here.
5. Rust 1.99.0 is the required toolchain; use `-j 4` and bounded disposable gates.
6. Multipass is not invoked; the test inventory limit remains two records and no automatic purge is used.
7. Do not claim a production consumer, BI/Benchmark integration, CLI execution, provider readiness or closed phase.
8. Preserve dirty root state and all unrelated worktrees/branches.
9. The Schedule-side `schedule_occurrence_linked` fact does not confirm Run-owner admission; project `confirmed` only from `schedule_admission_confirmed`.

## 6. Review / approval

| Role | Review status |
|---|---|
| Architecture | Mavis review: owner boundary and fail-closed status retained. |
| SRE Lead | Mavis review: disposable database cleanup and zero-VM evidence required. |
| Platform | Mavis review: no target grants or provider operations are claimed. |
| Review chair | Mavis review: migration/API/docs cross-check recorded. |
| PM | Mavis review: D1 is a bounded prerequisite; follow-on consumer phase remains open. |

## 7. Revision history

| Version | Date | Revision | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-09 | Record D1 lifecycle schema, API/UI projection, validation boundary and open consumer gates. | Phase 9F4C-D1 implementation review. |
