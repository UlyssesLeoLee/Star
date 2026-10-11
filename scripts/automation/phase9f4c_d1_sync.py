"""Synchronize the Phase 9F4C-D1 admission lifecycle design and evidence."""

# @cypher schema=1 source_sha256=50f1740a0d6e64f90d5060a4d8d16db0f41a014b4e233db182b2ee4e85d60ea1
# MERGE (self:File {path:"scripts/automation/phase9f4c_d1_sync.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::main",kind:"function"})
# MERGE (sync_file:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::sync_file",kind:"function"})
# MERGE (line_offsets:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::_line_offsets",kind:"function"})
# MERGE (upsert_section:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::upsert_section",kind:"function"})
# MERGE (upsert_table_row:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::upsert_table_row",kind:"function"})
# MERGE (remove_once:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::remove_once",kind:"function"})
# MERGE (keep_one_exact_line:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::keep_one_exact_line",kind:"function"})
# MERGE (insert_before_last:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::insert_before_last",kind:"function"})
# MERGE (insert_after_prefix:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::insert_after_prefix",kind:"function"})
# MERGE (write_new:Symbol {id:"scripts/automation/phase9f4c_d1_sync.py::write_new",kind:"function"})
# MERGE (requirements:Config {id:"docs/requirements.md"})
# MERGE (basic:Config {id:"docs/basic-design.md"})
# MERGE (srs:Config {id:"docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md"})
# MERGE (bd:Config {id:"docs/design/BD-AUTOMATION-SCHEDULE-API-001.md"})
# MERGE (dd:Config {id:"docs/design/DD-AUTOMATION-SCHEDULE-API-001.md"})
# MERGE (data:Config {id:"docs/data-design.md"})
# MERGE (plan:Config {id:"docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md"})
# MERGE (automation:Config {id:"docs/automation-design.md"})
# MERGE (registry:Config {id:"scripts/automation/registry.md"})
# MERGE (report:Config {id:"docs/reports/PHASE-9F4C-D1-ADMISSION-LIFECYCLE-REPORT.md"})
# MERGE (wtm:Config {id:"docs/data-design/phase9f4c-d-run-admission-wtm.md"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(sync_file)
# MERGE (self)-[:DEFINES]->(line_offsets)
# MERGE (self)-[:DEFINES]->(upsert_section)
# MERGE (self)-[:DEFINES]->(upsert_table_row)
# MERGE (self)-[:DEFINES]->(remove_once)
# MERGE (self)-[:DEFINES]->(keep_one_exact_line)
# MERGE (self)-[:DEFINES]->(insert_before_last)
# MERGE (self)-[:DEFINES]->(insert_after_prefix)
# MERGE (self)-[:DEFINES]->(write_new)
# MERGE (main)-[:CALLS]->(sync_file)
# MERGE (main)-[:CALLS]->(upsert_section)
# MERGE (main)-[:CALLS]->(upsert_table_row)
# MERGE (main)-[:CALLS]->(remove_once)
# MERGE (main)-[:CALLS]->(keep_one_exact_line)
# MERGE (main)-[:CALLS]->(insert_before_last)
# MERGE (main)-[:CALLS]->(insert_after_prefix)
# MERGE (main)-[:CALLS]->(write_new)
# MERGE (upsert_section)-[:CALLS]->(line_offsets)
# MERGE (sync_file)-[:UPDATES]->(requirements)
# MERGE (sync_file)-[:UPDATES]->(basic)
# MERGE (sync_file)-[:UPDATES]->(srs)
# MERGE (sync_file)-[:UPDATES]->(bd)
# MERGE (sync_file)-[:UPDATES]->(dd)
# MERGE (sync_file)-[:UPDATES]->(data)
# MERGE (sync_file)-[:UPDATES]->(plan)
# MERGE (sync_file)-[:UPDATES]->(automation)
# MERGE (sync_file)-[:UPDATES]->(registry)
# MERGE (sync_file)-[:WRITES]->(report)
# MERGE (sync_file)-[:WRITES]->(wtm)
# @endcypher

from __future__ import annotations

import argparse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]

REQUIREMENT = """### 50.8Z Phase 9F4C-D1 Schedule admission attempt lifecycle

Schedule-originated TaskExecutionRun rows distinguish provisional admission from execution start. A logical attempt has a positive `automation_admission_attempt` ordinal unique within its tenant and occurrence; legacy null ordinals map to attempt 1 for uniqueness without rewriting immutable Run facts. Provisional rows receive `created_at` while `started_at` remains NULL. `started_at` is written only when execution actually starts. Non-Schedule Run creation continues to require an actual start timestamp.

Run list/detail projections expose the logical attempt ordinal, `created_at`, nullable `started_at`, and lifecycle state derived from append-only RunEvents: provisional, confirmed, compensated, or started. Confirmation requires the Run-owner `schedule_admission_confirmed` event; the Schedule-side `schedule_occurrence_linked` fact records a link and alone does not prove the confirmation was consumed. Pagination orders by `COALESCE(started_at, created_at)` and accepts the legacy `cursor_started_at` alias while returning the explicit `cursor_ordered_at` field. Confirmation and compensation events are separate from `run_started`.

This slice changes the persisted lifecycle model and read projection only. It does not implement the Schedule/Run Inbox consumer, confirmation writer, compensation consumer, Infrastructure-demand gate, production owner grants/identity, or an execution path. Schedule execution remains fail-closed until owner-local transactions and all authorization, reservations, Multipass/K3s readiness, sandbox, Hook and independent validation gates are verified.

| Acceptance ID | Acceptance criterion |
|---|---|
| AC-SCHED-API-031 | Each provisional Schedule Run MUST persist a positive attempt ordinal unique per tenant/occurrence, keep `started_at` NULL, and retain `created_at`; legacy NULL ordinal is interpreted as attempt 1 without rewriting old Run rows. Actual execution start is represented separately by `run_started`/`started_at`. The API MUST expose attempt, admission state and both timestamps, and paginate by `COALESCE(started_at, created_at)` with backward-compatible cursor parsing. Reapplying the migration and duplicate/missing/legacy attempt cases MUST be covered. No owner consumer, production execution, VM, or Multipass inventory mutation is implied. |

"""

SRS_ROW = "| SCHED-API-031 | A Schedule provisional Run MUST record a positive logical admission attempt ordinal unique within tenant/occurrence, `created_at`, and `started_at = NULL`; the actual start is recorded separately. Legacy null ordinals represent attempt 1 without mutating immutable rows. Run list/detail MUST expose attempt, admission state and nullable timestamps, and use a stable `COALESCE(started_at, created_at)` cursor while accepting the old cursor alias. Only the Run-owner `schedule_admission_confirmed` event confirms admission; `schedule_occurrence_linked` alone does not. | Disposable PostgreSQL tests cover repeat migration, attempt uniqueness, legacy-null collision, invalid provisional/non-Schedule rows, and lifecycle event vocabulary; focused Rust tests cover API cursor compatibility and state projection. Owner consumers, production grants/identity and execution remain fail-closed. |\n"

SRS_STATUS = "Phase 9F4C-D1 verifies the attempt-lifecycle schema and read projection only; Schedule/Run consumers, production identities/grants, Infrastructure demand confirmation and execution remain fail-closed.\n\n"

BASIC_MAPPING = "| AC-SCHED-API-031 | Requirements §50.8Z; Schedule SRS §SCHED-API-031; Schedule BD §6.16; Schedule DD §6.18. Persist logical Schedule attempt ordinals and provisional Run timestamps, keep actual `run_started` separate, expose lifecycle state and stable ordered pagination. PostgreSQL schema/negative cases and focused API cursor tests are implementation evidence only; owner consumers and execution remain fail-closed. |\n"

BASIC_SECTION = """### 16.51 Phase 9F4C-D1 Schedule admission attempt lifecycle

Schedule Run attempts are persisted as distinct immutable Run identities. `automation_admission_attempt` is positive for provisional Schedule admissions and unique with tenant plus occurrence; a legacy NULL maps to attempt 1 in the unique expression index. Provisional admissions set `created_at` and leave `started_at` NULL. Actual execution start remains a later RunEvent and timestamp. Manual/CLI Run creation keeps an explicit start time.

Run list/detail derives `provisional`, `confirmed`, `compensated`, or `started` from RunEvents and distinguishes creation time from execution start. Only the Run-owner `schedule_admission_confirmed` event confirms admission; Schedule-side `schedule_occurrence_linked` is a link fact and cannot stand in for consuming that confirmation. Pagination sorts on `COALESCE(started_at, created_at)`, accepts `cursor_started_at` for older clients, and emits the ordered cursor alias. The D1 migration changes no owner grants and adds no consumer; same-transaction owner writes, Schedule confirmation, demand publication and all execution gates remain later work.

"""

BD_SECTION = """## 6.16 9F4C-D1 Schedule attempt and provisional Run persistence

The Run persistence schema MUST distinguish durable admission from execution. `created_at` is the row creation time; nullable `started_at` is the actual start time. For new non-Schedule Run rows, `started_at` remains required and writers MUST set it explicitly. A provisional Schedule row MUST have `started_at IS NULL` and a positive `automation_admission_attempt`; its first persisted record sets `created_at`.

Logical attempt uniqueness is `(tenant_id, automation_occurrence_id, COALESCE(automation_admission_attempt, 1))`, preserving legacy rows without updating them. A transport retry of the same logical attempt replays the same Run identity; a later attempt uses the next ordinal and a fresh request identity only after the previous attempt is terminal. Append-only RunEvents distinguish Run-owner Schedule confirmation, compensation and actual `run_started`; the Schedule-side `schedule_occurrence_linked` fact alone is not confirmation.

The authenticated Run API returns the attempt ordinal, derived admission state, creation timestamp and nullable start timestamp. List ordering and cursor comparison use `COALESCE(started_at, created_at)` plus `run_id`; the old `cursor_started_at` query name is accepted and the response also carries `cursor_ordered_at`. CLI inserts explicitly record their actual start to preserve the tightened timestamp invariant.

PostgreSQL 18.6 disposable verification applies this migration twice, runs the attempt/legacy/duplicate/event SQL cases, and asserts owner grants/RLS; the task-run API's focused cursor tests and `star-api-rest --all-targets` compile verify the read path. This does not implement a Schedule admission Inbox, Run confirmation/compensation consumer, production DB grants/identity, BI event pipeline, Infrastructure demand gate, or executor. Multipass inventory remains untouched and capped at two test records.

"""

DD_SECTION = """## 6.18 9F4C-D1 admission attempt schema and API projection

The migration extends the existing `multica.task_execution_run` Transaction with `created_at` and `automation_admission_attempt`; it adds no table, owner role, or runtime grant. Existing Run rows are not backfilled or rewritten. New `created_at` values default to `clock_timestamp()`. `started_at` becomes nullable with no implicit default: non-Schedule rows still require an explicit start, while a provisional Schedule row requires a positive attempt ordinal and a NULL start.

`uq_task_execution_run_schedule_occurrence_attempt` enforces one Run per tenant/occurrence/logical ordinal and maps a legacy NULL ordinal to one. The existing time indexes are rebuilt on `COALESCE(started_at, created_at)`. The append-only event allowlist gains `schedule_admission_confirmed` and `schedule_admission_compensated`; actual `run_started` remains an independent event.

The API's list and detail read paths project `admission_attempt`, `created_at`, nullable `started_at`, and admission state from Run origin plus append-only events. Only `schedule_admission_confirmed` confirms Schedule admission; `schedule_occurrence_linked` is a Schedule-side link fact and does not prove the Run owner consumed confirmation. Cursor ordering and SQL comparison use the coalesced timestamp and Run ID; parser compatibility accepts the legacy `cursor_started_at` and the new `cursor_ordered_at`, rejecting conflicting aliases. The UI labels provisional/confirmed/compensated states and displays actual start only after start evidence.

The disposable PostgreSQL 18.6 gate re-applies the migration and validates provisional attempts 1/2, duplicate ordinal rejection, legacy NULL-as-one collision, missing ordinal rejection, non-Schedule start requirement, and confirmation/compensation event acceptance. Runtime grants remain unchanged. This migration/API slice does not add the Schedule or Run owner consumers, confirmation receipt, Infrastructure demand writer, target-database rollout, BI/Benchmark, provider or execution; production stays fail-closed.

"""

WTM_TEXT = """# Phase 9F4C-D1 Run admission data classification

Date: 2026-10-09
Scope: `2026-10-09-task-run-admission-attempt-lifecycle.sql`

## Transaction facts

| Existing table | Classification | D1 change | Retention / mutability |
|---|---|---|---|
| `multica.task_execution_run` | Transaction | Add `created_at` and `automation_admission_attempt`; keep each attempt as an immutable Run identity; actual start is separate from provisional admission. | Append/update rules remain those of the existing Run owner; no new retention or deletion policy is introduced. Existing rows are not backfilled. |
| `multica.task_execution_run_event` | Transaction | Allow `schedule_admission_confirmed` and `schedule_admission_compensated` in the existing append-only event stream; only `schedule_admission_confirmed` proves Run-owner confirmation. | Append-only; WORM/audit policy unchanged. |

## Work and Master coverage

D1 creates no Work table and no Master table. It does not add an idempotency cache, mutable retry row, or profile revision. Schedule rules remain existing Master data; broker delivery state and lease Work retention are unchanged. The attempt ordinal is part of immutable Run identity and is therefore classified with the Run Transaction, not as mutable Work state.

## Verification boundary

The schema runner applies the migration twice and exercises uniqueness, legacy ordinal, provisional timestamp, non-Schedule start, and event-vocabulary constraints on disposable PostgreSQL 18.6. No target DB grant changes, production deployment, NATS, Multipass VM, or inventory mutation is part of this slice.
"""

PLAN_SECTION = """### 6.114 Phase 9F4C-D1 admission attempt lifecycle（2026-10-09）

| 工作项 | 本阶段实现/证据 | 状态与边界 |
|---|---|---|
| Persisted attempt identity | Add `created_at` and `automation_admission_attempt` to the existing Run Transaction; enforce per-occurrence attempt uniqueness with legacy NULL interpreted as attempt 1; require provisional Schedule rows to retain NULL `started_at`. | 🟢 Disposable PostgreSQL 18.6 repeated migration and SQL invariant gate passed; no owner grants changed. |
| API and UI projection | Return attempt, derived admission state, creation time and nullable actual start; order pages by coalesced creation/start time; retain legacy cursor alias; display provisional/confirmed/compensated states. Confirmation requires the explicit Run-owner confirmation event. | 🟢 Focused API cursor and lifecycle projection tests passed; workspace package all-target compile passed. Frontend Vitest was not run because `frontend/node_modules` is absent. |
| Automation/evidence | Extend `phase9f4d_run_owner_consumer.py` with the D1 migration and lifecycle invariant test; add exact-anchor cross-document sync script and report. | 🟢 PostgreSQL schema-only runner passed after the migration idempotency fix; it created zero VMs and did not modify Multipass inventory. |
| Open gates | Schedule/Run Inbox consumer, confirmation/compensation writers, target DB identity/grants, demand gate, BI/Benchmark integration, provider/readiness producer, sandbox, Hooks and executor. | 🔒 Remain fail-closed; D1 is a schema/read-projection prerequisite, not owner-consumer completion. |

"""

AUTOMATION_SECTION = """### 4.72 Phase 9F4C-D1 admission lifecycle schema gate

**自动化档：[P]** — `scripts/automation/phase9f4d_run_owner_consumer.py` now includes the D1 attempt-lifecycle migration and SQL invariant fixture in its disposable PostgreSQL path. `--schema-only` requires no NATS server, applies the migration twice, checks owner grants/RLS and executes the lifecycle SQL cases. Pass `--cargo` explicitly when Cargo is not on PATH; the schema-only gate checks the declared Rust toolchain version but does not run the NATS/Rust integration target.

`scripts/automation/phase9f4c_d1_sync.py --apply` synchronizes requirements, basic/detailed design, W/T/M, implementation plan, automation index and the phase report using exact anchors. Running it twice must make the second pass a no-op. PostgreSQL evidence is scoped to disposable schema/invariant verification; it does not prove production owner consumers, target database grants, Multipass/K3s providers, execution, BI or Benchmark.

"""

REPORT = """# PHASE-9F4C-D1 — Schedule Admission Attempt Lifecycle

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
"""


def read_text(path: Path) -> tuple[str, str]:
    with path.open("r", encoding="utf-8", newline="") as stream:
        text = stream.read()
    return text, "\r\n" if "\r\n" in text else "\n"


def write_text(path: Path, text: str) -> None:
    with path.open("w", encoding="utf-8", newline="") as stream:
        stream.write(text)


def replace_once(path: Path, old: str, new: str) -> None:
    text, newline = read_text(path)
    old = old.replace("\n", newline)
    new = new.replace("\n", newline)
    if new in text:
        return
    if old not in text:
        raise RuntimeError(f"missing exact anchor in {path.relative_to(ROOT)}: {old[:100]!r}")
    write_text(path, text.replace(old, new, 1))


def remove_once(path: Path, line: str) -> None:
    text, newline = read_text(path)
    line = line.replace("\n", newline)
    if line in text:
        write_text(path, text.replace(line, "", 1))


def keep_one_exact_line(path: Path, line: str) -> None:
    text, newline = read_text(path)
    line = line.replace("\n", newline)
    first = text.find(line)
    if first < 0:
        return
    tail = text[first + len(line) :].replace(line, "")
    if tail != text[first + len(line) :]:
        write_text(path, text[: first + len(line)] + tail)


def _line_offsets(text: str) -> list[tuple[int, str]]:
    offsets: list[tuple[int, str]] = []
    offset = 0
    for line in text.splitlines(keepends=True):
        offsets.append((offset, line))
        offset += len(line)
    return offsets


def upsert_section(path: Path, heading_prefix: str, end_marker: str, block: str) -> None:
    text, newline = read_text(path)
    block = block.replace("\n", newline)
    starts = [offset for offset, line in _line_offsets(text) if line.startswith(heading_prefix)]
    if starts:
        start = starts[0]
        end = text.find(end_marker, start)
        if end < 0:
            raise RuntimeError(f"missing section boundary in {path.relative_to(ROOT)}: {end_marker!r}")
        updated = text[:start] + block + text[end:]
        if updated != text:
            write_text(path, updated)
        return
    marker = text.rfind(end_marker)
    if marker < 0:
        raise RuntimeError(f"missing section insertion marker in {path.relative_to(ROOT)}: {end_marker!r}")
    write_text(path, text[:marker] + block + text[marker:])


def upsert_table_row(path: Path, row_prefix: str, before_marker: str, row: str) -> None:
    text, newline = read_text(path)
    row = row.replace("\n", newline)
    text = "".join(line for line in text.splitlines(keepends=True) if not line.startswith(row_prefix))
    marker = text.rfind(before_marker)
    if marker < 0:
        raise RuntimeError(f"missing table insertion marker in {path.relative_to(ROOT)}: {before_marker!r}")
    write_text(path, text[:marker] + row + text[marker:])


def insert_before_last(path: Path, marker: str, block: str) -> None:
    text, newline = read_text(path)
    marker = marker.replace("\n", newline)
    block = block.replace("\n", newline)
    if block.strip() in text:
        return
    offset = text.rfind(marker)
    if offset < 0:
        raise RuntimeError(f"missing insertion marker in {path.relative_to(ROOT)}: {marker[:100]!r}")
    write_text(path, text[:offset] + block + text[offset:])


def insert_after_prefix(path: Path, prefix: str, block: str) -> None:
    text, newline = read_text(path)
    block = block.replace("\n", newline)
    if block.strip() in text:
        return
    lines = text.splitlines(keepends=True)
    for index, line in enumerate(lines):
        if line.startswith(prefix):
            lines.insert(index + 1, block)
            write_text(path, "".join(lines))
            return
    raise RuntimeError(f"missing row prefix in {path.relative_to(ROOT)}: {prefix!r}")


def write_new(path: Path, body: str) -> None:
    if path.exists():
        existing, _ = read_text(path)
        if existing == body:
            return
        raise RuntimeError(f"refusing to overwrite existing file: {path.relative_to(ROOT)}")
    path.parent.mkdir(parents=True, exist_ok=True)
    write_text(path, body)


def sync() -> None:
    req = ROOT / "docs/requirements.md"
    srs = ROOT / "docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md"
    basic = ROOT / "docs/basic-design.md"
    bd = ROOT / "docs/design/BD-AUTOMATION-SCHEDULE-API-001.md"
    dd = ROOT / "docs/design/DD-AUTOMATION-SCHEDULE-API-001.md"
    data = ROOT / "docs/data-design.md"
    plan = ROOT / "docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md"
    automation = ROOT / "docs/automation-design.md"
    registry = ROOT / "scripts/automation/registry.md"

    replace_once(req, "要件定义书（统合扩展版 v6.11）", "要件定义书（统合扩展版 v6.12）")
    upsert_section(req, "### 50.8Z ", "### 50.9 追溯与后续专题同步", REQUIREMENT)
    insert_after_prefix(req, "| v6.11 |", "| v6.12 | 2026-10-09 | Mavis review | Add §50.8Z / AC-SCHED-API-031 for immutable logical Schedule admission attempts, provisional Run creation/start-time separation, lifecycle event projection and compatible ordered pagination. Record the disposable PostgreSQL 18.6 repeated migration/RLS/SQL invariant gate and focused Rust evidence; keep owner consumers, target grants and execution fail-closed. | The typed owner protocol requires persisted attempt identity without misreporting provisional Runs as started. |\n")

    replace_once(srs, "> Version: 0.32 | Date: 2026-10-09", "> Version: 0.33 | Date: 2026-10-09")
    upsert_table_row(srs, "| SCHED-API-031 |", "## 3. Verification status", SRS_ROW)
    insert_after_prefix(srs, "## 3. Verification status", SRS_STATUS)
    insert_after_prefix(srs, "| v0.32 |", "| v0.33 | 2026-10-09 | Add SCHED-API-031 for immutable attempt ordinals, provisional versus started timestamps, lifecycle projection and compatible ordered pagination; record the repeated PostgreSQL lifecycle schema gate and focused API evidence without enabling owner consumers or execution. | Phase 9F4C-D1 separates Run admission from actual execution start. |\n")

    replace_once(basic, "> **文档版本**: v6.08 (2026-10-09)", "> **文档版本**: v6.09 (2026-10-09)")
    replace_once(basic, "> **上游要件定义书**: docs/requirements.md v6.11", "> **上游要件定义书**: docs/requirements.md v6.12")
    insert_after_prefix(basic, "| AC-SCHED-API-030 |", BASIC_MAPPING)
    upsert_section(basic, "### 16.51 ", "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |", BASIC_SECTION)
    insert_after_prefix(basic, "| v6.08 |", "| v6.09 | 2026-10-09 | Mavis review | Add §16.51 / AC-SCHED-API-031 for the Schedule attempt lifecycle schema and API/UI projection. Align requirements v6.12, Schedule SRS/BD/DD v0.33/v0.32/v0.32 and Data Design v2.2; record PostgreSQL 18.6 and focused Rust evidence, with no owner consumer or execution enabled. | Prevent provisional admission from being presented as execution start. |\n")

    replace_once(bd, "> Version: 0.31 | Date: 2026-10-09", "> Version: 0.32 | Date: 2026-10-09")
    upsert_section(bd, "## 6.16 ", "## 7. Revision history", BD_SECTION)
    insert_after_prefix(bd, "| v0.31 |", "| v0.32 | 2026-10-09 | Add §6.16 for the persisted attempt ordinal, provisional/start timestamp boundary, append-only lifecycle events and ordered Run API cursor; record disposable PostgreSQL and focused Rust evidence while keeping Schedule/Run consumers and execution fail-closed. | Phase 9F4C-D1 lifecycle implementation. |\n")

    replace_once(dd, "> Version: 0.31 | Date: 2026-10-09", "> Version: 0.32 | Date: 2026-10-09")
    upsert_section(dd, "## 6.18 ", "## 7. Revision history", DD_SECTION)
    insert_after_prefix(dd, "| v0.31 |", "| v0.32 | 2026-10-09 | Add §6.18 for attempt lifecycle DDL, legacy ordinal mapping, event vocabulary, API cursor and Task Run UI; record repeated PostgreSQL 18.6 migration/invariant evidence and retain all production consumer, grants and execution gates. | Phase 9F4C-D1 implementation/design reconciliation. |\n")

    replace_once(data, "> **文档版本**: v2.1 (2026-10-09)", "> **文档版本**: v2.2 (2026-10-09)")
    replace_once(data, "Link the Phase 9F4C-D1 W/T/M supplement", "Link `docs/data-design/phase9f4c-d-run-admission-wtm.md`")
    remove_once(
        data,
        "> | v2.2 | 2026-10-09 | Classify the existing TaskExecutionRun and RunEvent lifecycle additions as append-oriented Transaction facts; specify no new Work/Master table, retention change, or role grant. Link the Phase 9F4C-D1 W/T/M supplement and record disposable PostgreSQL evidence only. | Schedule admission attempts need immutable Run identity and separately auditable lifecycle facts. |\n",
    )
    keep_one_exact_line(
        data,
        "> | v2.2 | 2026-10-09 | Classify the existing TaskExecutionRun and RunEvent lifecycle additions as append-oriented Transaction facts; specify no new Work/Master table, retention change, or role grant. Link `docs/data-design/phase9f4c-d-run-admission-wtm.md` and record disposable PostgreSQL evidence only. | Schedule admission attempts need immutable Run identity and separately auditable lifecycle facts. |\n",
    )
    insert_after_prefix(data, "> | v2.1 |", "> | v2.2 | 2026-10-09 | Classify the existing TaskExecutionRun and RunEvent lifecycle additions as append-oriented Transaction facts; specify no new Work/Master table, retention change, or role grant. Link `docs/data-design/phase9f4c-d-run-admission-wtm.md` and record disposable PostgreSQL evidence only. | Schedule admission attempts need immutable Run identity and separately auditable lifecycle facts. |\n")

    replace_once(plan, "渡口 Project / Cloud Branch / Engineering Run 架构与实施计划 v6.46", "渡口 Project / Cloud Branch / Engineering Run 架构与实施计划 v6.47")
    upsert_section(plan, "### 6.114 ", "## 修订履历", PLAN_SECTION)
    insert_after_prefix(plan, "| v6.46 |", "| v6.47 | 2026-10-09 | Add §6.114 for Phase 9F4C-D1 attempt lifecycle persistence and Task Run projection. Record PostgreSQL 18.6 schema gate, Rust verification limits, exact-sync script and fail-closed owner-consumer/VM/execution gaps. | D1 schema/API/UI implementation aligned to the active owner-local design. |\n")

    replace_once(automation, "> **文档版本**: v2.30 (2026-10-09)", "> **文档版本**: v2.31 (2026-10-09)")
    upsert_section(automation, "### 4.72 ", "| Version | Date | Change |", AUTOMATION_SECTION)
    insert_after_prefix(automation, "| v2.30 |", "| v2.31 | 2026-10-09 | Add §4.72 for the D1 lifecycle schema-only gate and exact-anchor idempotent cross-document synchronizer. Record repeated PostgreSQL 18.6 migration and owner/RLS invariant evidence; keep owner consumers, target DB identity/grants and execution fail-closed. | Phase 9F4C-D1 lifecycle schema/API projection. |\n")

    replace_once(registry, "> **文档版本**: v0.77 (2026-10-09)", "> **文档版本**: v0.78 (2026-10-09)")
    replace_once(registry, "docs/automation-design.md` v2.30", "docs/automation-design.md` v2.31")
    replace_once(registry, "§4.71 owner-message contract", "§4.71 owner-message contract, §4.72 admission lifecycle schema gate")
    insert_after_prefix(registry, "| `scripts/automation/phase9f4c_d_owner_protocol.py`", "| `scripts/automation/phase9f4c_d1_sync.py` | Idempotent exact-anchor synchronizer for 9F4C-D1 requirements/basic/Schedule design/Data Design W-T-M/plan/automation registry and phase report; safe to run twice. | Phase 9F4C-D1; automation-design §4.72; plan §6.114 | This phase | 🟢 Second pass is a no-op; owner consumers and execution remain fail-closed. |\n")
    replace_once(registry, "schema/RLS and Run-owner result/quarantine integration runner; applies the existing quarantine migration/assertions", "schema/RLS, D1 Run-attempt lifecycle, and Run-owner result/quarantine integration runner; applies the D1 migration twice and checks its SQL invariants alongside existing quarantine assertions")
    replace_once(registry, "Phase 9F4D-3D5/3D7; automation-design §4.65/§4.70", "Phase 9F4D-3D5/3D7 and 9F4C-D1; automation-design §4.65/§4.70/§4.72")
    insert_after_prefix(registry, "| **v0.77** |", "| **v0.78** | **2026-10-09** | **Mavis review** | Register the idempotent D1 documentation synchronizer and extend the PostgreSQL schema runner index with the repeated Run-attempt lifecycle migration/SQL invariant gate. Record PostgreSQL 18.6 evidence, zero VM/inventory mutation and open owner-consumer/execution gates. | Phase 9F4C-D1 lifecycle implementation and design reconciliation. |\n")

    wtm_path = ROOT / "docs/data-design/phase9f4c-d-run-admission-wtm.md"
    if wtm_path.exists():
        replace_once(
            wtm_path,
            "Allow `schedule_admission_confirmed` and `schedule_admission_compensated` in the existing append-only event stream.",
            "Allow `schedule_admission_confirmed` and `schedule_admission_compensated` in the existing append-only event stream; only `schedule_admission_confirmed` proves Run-owner confirmation.",
        )
    else:
        write_new(wtm_path, WTM_TEXT)
    report_path = ROOT / "docs/reports/PHASE-9F4C-D1-ADMISSION-LIFECYCLE-REPORT.md"
    if report_path.exists():
        replace_once(
            report_path,
            "| API/UI | Project attempt/state/timestamps and paginate by coalesced start/creation time; show lifecycle labels in Task Run history. | Focused Rust cursor tests and package all-target compile. |",
            "| API/UI | Project attempt/state/timestamps and paginate by coalesced start/creation time; show lifecycle labels in Task Run history. | Focused Rust cursor and lifecycle-state tests plus package all-target compile. |",
        )
        replace_once(
            report_path,
            "| `cargo test -p star-api-rest --lib task_runs::tests -j 4` | Passed 4/4. |",
            "| `cargo test -p star-api-rest --lib task_runs::tests -j 4` | Passed 5/5. |",
        )
        replace_once(
            report_path,
            "8. Preserve dirty root state and all unrelated worktrees/branches.\n",
            "8. Preserve dirty root state and all unrelated worktrees/branches.\n9. The Schedule-side `schedule_occurrence_linked` fact does not confirm Run-owner admission; project `confirmed` only from `schedule_admission_confirmed`.\n",
        )
        replace_once(
            report_path,
            "| Frontend Vitest | Not run; `frontend/node_modules` is absent. |\n| `cargo fmt --all -- --check` | Repository-wide check reports unrelated existing formatting differences; no workspace-wide format was applied. Touched Rust files are checked separately. |",
            "| `python scripts/automation/registry_check.py` | Exit 0, zero errors; the repository-wide registry checker emitted 197 existing unregistered-script warnings. |\n| Frontend Vitest | Not run; `frontend/node_modules` is absent. |\n| Rustfmt checks | Workspace/package checks report formatting differences only in unchanged Rust files; no touched Rust file appears in those diagnostics, and no workspace-wide formatting was applied. |",
        )
    else:
        write_new(report_path, REPORT)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="apply the exact-anchor documentation synchronization")
    args = parser.parse_args()
    if not args.apply:
        print("Preview only. Re-run with --apply after checking the current document anchors.")
        return 0
    sync()
    print("Phase 9F4C-D1 documents synchronized.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
