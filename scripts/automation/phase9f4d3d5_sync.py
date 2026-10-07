"""Idempotently synchronize the Phase 9F4D-3D5 requirements and design documents."""

# @cypher schema=1 source_sha256=f3d578d60b13ecbd3e1c28d8ef0bd9f8de0dc41a1387b30fcb16484aec50bc37
# MERGE (self:File {path:"scripts/automation/phase9f4d3d5_sync.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4d3d5_sync.py::main",kind:"function"})
# MERGE (replace_once:Symbol {id:"scripts/automation/phase9f4d3d5_sync.py::replace_once",kind:"function"})
# MERGE (insert_before:Symbol {id:"scripts/automation/phase9f4d3d5_sync.py::insert_before",kind:"function"})
# MERGE (insert_after_prefix:Symbol {id:"scripts/automation/phase9f4d3d5_sync.py::insert_after_prefix",kind:"function"})
# MERGE (write:Symbol {id:"scripts/automation/phase9f4d3d5_sync.py::write",kind:"function"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(replace_once)
# MERGE (self)-[:DEFINES]->(insert_before)
# MERGE (self)-[:DEFINES]->(insert_after_prefix)
# MERGE (self)-[:DEFINES]->(write)
# MERGE (main)-[:CALLS]->(replace_once)
# MERGE (main)-[:CALLS]->(insert_before)
# MERGE (main)-[:CALLS]->(insert_after_prefix)
# MERGE (main)-[:CALLS]->(write)
# @endcypher

from __future__ import annotations

from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[2]
CHANGES = 0


def read(relative_path: str) -> tuple[Path, str]:
    path = ROOT / relative_path
    return path, path.read_text(encoding="utf-8")


def write(path: Path, original: str, updated: str) -> None:
    global CHANGES
    if updated != original:
        path.write_text(updated, encoding="utf-8", newline="\n")
        CHANGES += 1


def replace_once(relative_path: str, old: str, new: str) -> None:
    path, content = read(relative_path)
    if new in content:
        return
    if content.count(old) != 1:
        raise RuntimeError(f"expected exactly one anchor in {relative_path}: {old[:80]!r}")
    write(path, content, content.replace(old, new, 1))


def insert_before(relative_path: str, anchor: str, block: str, marker: str) -> None:
    path, content = read(relative_path)
    if marker in content:
        return
    if content.count(anchor) != 1:
        raise RuntimeError(f"expected exactly one insertion anchor in {relative_path}: {anchor!r}")
    write(path, content, content.replace(anchor, f"{block.rstrip()}\n\n{anchor}", 1))


def insert_before_prefix(relative_path: str, prefix: str, block: str, marker: str) -> None:
    path, content = read(relative_path)
    if marker in content:
        return
    matches = list(re.finditer(r"^" + re.escape(prefix) + r".*$", content, re.MULTILINE))
    if len(matches) != 1:
        raise RuntimeError(f"expected exactly one insertion row in {relative_path}: {prefix!r}")
    start = matches[0].start()
    write(path, content, content[:start] + block.rstrip() + "\n\n" + content[start:])


def move_section_before_table(relative_path: str, section_marker: str, table_marker: str) -> None:
    path, content = read(relative_path)
    section_at = content.find(section_marker)
    table_at = content.find(table_marker)
    if section_at < 0 or table_at < 0:
        raise RuntimeError(f"missing section/table marker in {relative_path}")
    if section_at < table_at:
        return
    section_end = content.find("\n\n| v5.94 |", section_at)
    if section_end < 0:
        raise RuntimeError(f"cannot find end of misplaced section in {relative_path}")
    section = content[section_at:section_end].rstrip()
    without_section = content[:section_at] + content[section_end + 2:]
    table_at = without_section.find(table_marker)
    if table_at < 0:
        raise RuntimeError(f"cannot find revision table after section removal in {relative_path}")
    updated = without_section[:table_at] + section + "\n\n" + without_section[table_at:]
    write(path, content, updated)


def insert_after_prefix(relative_path: str, prefix: str, block: str, marker: str) -> None:
    path, content = read(relative_path)
    if marker in content:
        return
    matches = list(re.finditer(r"^" + re.escape(prefix) + r".*$", content, re.MULTILINE))
    if len(matches) != 1:
        raise RuntimeError(f"expected exactly one revision row in {relative_path}: {prefix!r}")
    end = matches[0].end()
    write(path, content, content[:end] + "\n" + block + content[end:])


def main() -> None:
    global CHANGES
    # User-selected minimum toolchain remains Rust 1.99.0; this phase adds no toolchain change.
    replace_once("docs/requirements.md", "统合扩展版 v5.97", "统合扩展版 v5.98")
    replace_once("docs/basic-design.md", "**文档版本**: v5.94 (2026-10-07)", "**文档版本**: v5.95 (2026-10-07)")
    replace_once("docs/basic-design.md", "docs/requirements.md v5.97", "docs/requirements.md v5.98")
    replace_once("docs/design/DD-WORKTREE-GROUP-001.md", "Worktree 与 Run Apps 详细设计 v4.63", "Worktree 与 Run Apps 详细设计 v4.64")
    replace_once("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "> Version: 0.26 | Date: 2026-10-07", "> Version: 0.27 | Date: 2026-10-07")
    replace_once("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "> Version: 0.25 | Date: 2026-10-07", "> Version: 0.26 | Date: 2026-10-07")
    replace_once("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "> Version: 0.25 | Date: 2026-10-07", "> Version: 0.26 | Date: 2026-10-07")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "> v0.24 · 2026-10-07", "> v0.25 · 2026-10-07")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "requirements v5.97", "requirements v5.98")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "§50.8G-50.8O", "§50.8G-50.8P")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "basic-design v5.94", "basic-design v5.95")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "16.21-16.39", "16.21-16.40")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "Schedule SRS v0.26 / Schedule BD v0.25 / Schedule DD v0.25 / Automation Design v2.23", "Schedule SRS v0.27 / Schedule BD v0.26 / Schedule DD v0.26 / Automation Design v2.24")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "6.83-6.99", "6.83-6.100")
    replace_once("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "Run-owner Inbox consumer/scheduled reaper", "Run-owner consumer provisioning/quarantine/scheduled reaper")
    replace_once("docs/data-design.md", "**文档版本**: v1.7 (2026-10-06)", "**文档版本**: v1.8 (2026-10-07)")
    replace_once("docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md", "架构与实施计划 v6.26", "架构与实施计划 v6.27")
    replace_once("docs/automation-design.md", "**文档版本**: v2.23 (2026-10-07)", "**文档版本**: v2.24 (2026-10-07)")
    replace_once("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "Run-owner consumer, production worker", "production consumer provisioning/quarantine, production worker")
    replace_once("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "Run-owner Inbox consumer, production worker", "production Run-owner consumer provisioning/quarantine, production worker")

    requirement = """### 50.8P Phase 9F4D-3D5 Run-owner capacity result consumer

Before appending a scheduled capacity demand to the Run-owner Outbox, the Run/Execution owner MUST verify in the caller-owned transaction that the referenced TaskExecutionRun has `run_origin='schedule'` and the same Schedule Rule ID/revision, occurrence ID, and correlation ID as the typed demand. A mismatch MUST abort the append; it MUST NOT create an Outbox fact.

The Run/Execution owner may consume Infrastructure capacity results only through an explicitly invoked Rust adapter supplied with an existing durable JetStream pull consumer using explicit acknowledgements. One pull request is limited to one message, a 132 KiB byte ceiling, and a 2-second expiry. The consumer validates the exact subject, envelope version, schema version, event identity, headers, payload contract, and event type; it derives Project/Branch/Engineering Run/Work Item/TaskExecutionRun/occurrence scope from the exact immutable demand Outbox row rather than trusting scope absent from the wire envelope.

The Run owner stores the accepted/rejected/expired result as an append-only Transaction Inbox fact under its own tenant-scoped owner transaction. The database commit completes before JetStream double-ack. A duplicate event is acknowledged only after its immutable scope and contents match the stored fact; a database or validation error never acknowledges the message. The released event remains fail-closed because the Infrastructure owner has no verified producer contract for it. Invalid or unmatched messages remain unacknowledged; durable poison-message quarantine is not implemented, so this consumer is not production-enabled.

This slice does not mutate TaskExecutionRun, Schedule, RunEvent, BI, reservations, readiness, or execution state. A stored capacity_accepted fact is not proof that both reservations remain current or that K3s/Agent Runtime is ready. Verification uses isolated PostgreSQL and loopback NATS only, creates zero Multipass VMs, and preserves the two-record inventory cap including stopped/deleted entries.

Traceability is synchronized with requirements v5.98, Schedule API SRS v0.27, Schedule BD/DD v0.26, Infrastructure DD v0.25, and Data Design v1.8.

| Acceptance ID | Acceptance criterion |
|---|---|
| AC-INFRA-002-P | Before appending a scheduled demand, Run-owner Rust MUST verify in the caller-owned transaction that TaskExecutionRun is schedule-origin and its rule ID/revision, occurrence ID and correlation ID exactly match the typed demand; a mismatch MUST create no Outbox fact. The consumer MUST accept only an explicitly acknowledged durable PullConsumer and MUST bound each pull to one message, at most 132 KiB, and 2 seconds. It MUST bind the event to the immutable Run demand row, write one append-only tenant-scoped Inbox fact, and acknowledge only after commit; exact duplicate replay is idempotent, conflicting replay and malformed scope fail closed, and database errors leave the message unacknowledged. Unproduced release events and poison-message cases MUST remain non-production-enabled until a versioned producer contract and durable quarantine exist. The consumer MUST NOT mutate Run/Schedule/readiness or enable execution. Tests MUST use disposable PostgreSQL and loopback NATS, create zero VMs, and preserve the two-record Multipass cap. |"""
    insert_before("docs/requirements.md", "### 50.9 追溯与后续专题同步", requirement, "### 50.8P Phase 9F4D-3D5")

    basic = """### 16.40 Phase 9F4D-3D5 Run-owner capacity result consumer

The Run-owner demand Outbox append receives the caller's existing Run transaction and validates that the referenced TaskExecutionRun is schedule-origin with matching rule ID/revision, occurrence ID and correlation ID before it appends the typed demand. A mismatch creates no Outbox fact.

Run/Execution owner receives Infrastructure capacity events through a supplied, provisioned durable JetStream pull consumer. The Rust consumer is deliberately not a broker/credential/consumer provisioner: it validates explicit-ack configuration and performs a single-message pull bounded to 132 KiB and 2 seconds. It checks the wire contract and resolves canonical Project, Branch, Engineering Run, Work Item, TaskExecutionRun, and occurrence identity by matching the exact stored Run demand fact.

Within the Run-owner database boundary, it appends one result Inbox fact and commits before double-ack. Duplicate event_id is accepted only when the previously persisted immutable scope and content match. No TaskExecutionRun/Schedule transition, RunEvent/BI projection, reservation release, readiness, or executor start occurs here. The upstream release producer contract and durable poison-message quarantine are absent; malformed messages remain unacknowledged, which blocks production deployment of this consumer. The exact rust-version is 1.99.0; disposable PostgreSQL/NATS verification creates no Multipass guest and keeps the two-record test cap.

| 验收 ID | 对应设计与当前边界 |
|---|---|
| AC-INFRA-002-P | §16.40；caller transaction 内校验 TaskExecutionRun 的 schedule origin、Rule ID/revision、occurrence 和 correlation 后写 demand Outbox；durable explicit-ack PullConsumer，单条/132 KiB/2 秒；以不可变 demand row 派生结果 scope；Run-owner Transaction Inbox commit 后 double-ack；严格 duplicate replay；invalid/unmatched 不 ACK；no quarantine/producer contract 时 production disabled；不改变 Run/Schedule/readiness/execution。 |"""
    insert_before_prefix("docs/basic-design.md", "| v5.94 |", basic, "### 16.40 Phase 9F4D-3D5")
    move_section_before_table("docs/basic-design.md", "### 16.40 Phase 9F4D-3D5", "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |")

    group_dd = """### 8.14 Run-owner capacity result Inbox（Phase 9F4D-3D5）

Infrastructure capacity results enter the Run/Execution owner through its own durable explicit-ack JetStream consumer and PostgreSQL transaction. The consumer bounds each fetch to one message, 132 KiB, and 2 seconds; canonical Project/Branch/Engineering Run/Work Item/TaskExecutionRun/occurrence scope is loaded from the immutable demand fact because those fields are not all present in the result envelope. It appends a Run-owned result Inbox fact, commits, then double-acks. A repeated event_id is acknowledged only after scope, digest, and content equality are confirmed.

The demand Outbox append also validates the referenced TaskExecutionRun inside the caller-owned transaction: it must be schedule-origin and its rule ID/revision, occurrence ID and correlation ID must match the typed demand before any Outbox fact is written.

This Inbox is a durable event fact, not a TaskExecutionRun/Schedule state projection and not a BI/RunEvent producer. Accepted capacity does not mean current reservations or K3s/Runtime readiness. The upstream capacity_released producer is unavailable, and invalid/unmatched messages are left unacknowledged without a durable quarantine table. Therefore the consumer remains production-disabled until producer, quarantine, consumer provisioning/identity, authorization/binding revalidation, and owner projection contracts are completed. The test uses no VM; Multipass inventory remains capped at two registered records. This section aligns requirements v5.98, basic design v5.95, Schedule API v0.27/.26, Infrastructure DD v0.25, and Data Design v1.8.

"""
    insert_before("docs/design/DD-WORKTREE-GROUP-001.md", "## §9 跨 App 事件与一致性", group_dd, "### 8.14 Run-owner capacity result Inbox")

    sched_requirement = "| SCHED-API-026 | Before a scheduled demand is appended, Run/Execution owner MUST validate in the caller-owned transaction that TaskExecutionRun is schedule-origin and its rule ID/revision, occurrence ID and correlation ID match the typed demand; mismatch MUST produce no Outbox fact. The owner MAY consume Infrastructure capacity results through an explicitly invoked Rust adapter wrapping a pre-provisioned durable JetStream PullConsumer with explicit acknowledgements. Each fetch MUST be limited to one message, 132 KiB and 2 seconds. The adapter MUST validate subject/envelope/header/payload identity, hydrate canonical scope from the immutable demand Outbox, append one tenant-scoped Run-owner result Inbox fact, commit before double-ack, and verify immutable equality on duplicate event_id. It MUST NOT mutate TaskExecutionRun, Schedule, RunEvent/BI, reservations, readiness or execution. Unproduced capacity_released and malformed/unmatched messages remain fail-closed; absent durable poison quarantine means the consumer is not production-enabled. |"
    insert_before("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "## 3. Verification status", sched_requirement, "SCHED-API-026")

    bd_schedule = """### 6.11 Phase 9F4D-3D5 Run-owner result consumer

The Run-owner demand Outbox append accepts the caller's transaction and validates the referenced TaskExecutionRun's schedule origin, Rule ID/revision, occurrence and correlation before insertion. The result consumer uses an existing provisioned durable explicit-ack pull consumer. Rust performs one bounded fetch (one message, 132 KiB, 2 seconds), validates the event contract and derives canonical scope from the exact immutable demand Outbox row. It records only an append-only result Inbox fact in the Run-owner transaction, then double-acks after commit. Exact duplicate replay is verified against stored scope/content; conflicts and DB failures are not acknowledged.

No Run/Schedule state update, RunEvent/BI emission, reservation mutation, readiness, or execution is part of this slice. Because invalid messages have no durable quarantine and the release producer is unavailable, this consumer is implemented as a fail-closed adapter and is not production deployable. Rust minimum is 1.99.0; isolated PG/NATS verification uses zero Multipass instances and retains the two-entry cap.

"""
    insert_before("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "## 7. Revision history", bd_schedule, "### 6.11 Phase 9F4D-3D5")

    dd_schedule = """### 6.12 Run-owner capacity result consumer

The Run-owner demand append adapter accepts the caller's existing transaction and validates the referenced TaskExecutionRun as schedule-origin with matching rule ID/revision, occurrence ID and correlation ID before writing the typed demand Outbox fact. A mismatch aborts without an Outbox insert. The result adapter wraps a pre-created durable JetStream PullConsumer with AckPolicy::Explicit; it does not create streams, consumers, credentials, workers, or tenant polling. A pull uses max_messages=1, max_bytes=132 KiB, and expires=2 seconds. The consumer validates schema-v2 result envelopes and payloads, then hydrates the complete domain scope from the immutable demand row before inserting the Run-owner Inbox fact.

The SQL transaction owns tenant RLS scope, deduplication and immutable replay comparison. Commit precedes Message::double_ack(); an error before commit leaves the delivery unacknowledged. Only accepted/rejected/expired producer contracts are consumed; released is explicitly unsupported. No TaskExecutionRun/Schedule mutation or BI event is emitted. Bad messages remain unacknowledged until a separately durable quarantine contract is implemented; production consumer deployment stays disabled.

"""
    insert_before("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "## 7. Revision history", dd_schedule, "### 6.12 Run-owner capacity result consumer")

    infra = """### 7.11 9F4D-3D5 Run-owner result consumer

The Run/Execution owner accepts results through a supplied durable JetStream PullConsumer configured for explicit acknowledgements. Fetch concurrency is fixed at one message; request size is at most 132 KiB and request expiry is 2 seconds. Rust validates the message subject, event ID, schema/envelope, allowlisted type, idempotency and typed payload, then binds it to the matching immutable demand row to obtain canonical Project/Branch/Engineering Run/Task/occurrence scope.

Before the caller-owned Run transaction appends a scheduled demand, the Run-owner store verifies that the referenced TaskExecutionRun is schedule-origin and matches the demand's Schedule Rule ID/revision, occurrence ID and correlation ID. A mismatch fails before the Outbox insert.

Run-owned persistence is a separate append-only Inbox transaction under tenant FORCE RLS and the least-privilege Run-owner role. The adapter commits before double-ack and compares complete immutable facts on duplicate event ID. It records no mutable reservation projection, Run/Schedule transition, readiness, BI event, or execution authorization. capacity_released lacks an upstream producer contract. Malformed/unmatched deliveries are not acknowledged and no quarantine fact is persisted, so the consumer remains production-disabled. Isolated PostgreSQL 18.6 plus loopback NATS tests create zero Multipass VMs; the global test inventory cap remains two registered records including stopped/deleted instances.

"""
    insert_before("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "## §8 审阅与修订历史", infra, "### 7.11 9F4D-3D5")

    data_design = """### 4.13.9 Phase 9F4D-3D5 Run-owner capacity facts（Transaction）

Migration: db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql.

| Table | W/T/M primary class | Owner and purpose | Retention / mutation | RLS and runtime access |
|---|---|---|---|---|
| multica.infrastructure_capacity_demand_outbox | T — Transaction | Run/Execution owner; immutable, versioned infrastructure demand and canonical scope | Append-only; UPDATE/DELETE/TRUNCATE rejected; no TTL or physical deletion | ENABLE + FORCE RLS on tenant_id; star_multica_run_owner SELECT/INSERT only |
| multica.infrastructure_capacity_result_inbox | T — Transaction | Run/Execution owner; immutable Infrastructure result, replay digest and event_id deduplication fact | Append-only; UPDATE/DELETE/TRUNCATE rejected; no TTL or physical deletion | ENABLE + FORCE RLS on tenant_id; star_multica_run_owner SELECT/INSERT only |

W contains no table in this migration; M contains no table in this migration. The Outbox is an immutable business/integration fact, not a short-lived delivery lease table. Broker delivery state remains owned by Infrastructure. Before appending a scheduled demand, Rust verifies the existing TaskExecutionRun's origin, Rule ID/revision, occurrence and correlation within the caller-owned transaction. The FK ties each result to the exact source demand and canonical scope; the global event_id unique constraint prevents cross-tenant identity reuse while RLS restricts visibility. Rust verifies stored demand JSON against typed V2 demand before accepting results. Runtime code does not mutate TaskExecutionRun/Schedule or release reservations. Both tables have no cleanup policy until a reviewed legal/audit retention rule exists.

"""
    insert_before("docs/data-design.md", "### 4.14 Module: domain-identity(`identity` schema)", data_design, "### 4.13.9 Phase 9F4D-3D5")
    insert_after_prefix("docs/data-design.md", "> | v1.7 |", "> | v1.8 | 2026-10-07 | Add §4.13.9 and the Phase 9F4D-3D5 W/T/M appendix: classify the Run demand Outbox and result Inbox as append-only Transaction facts, define tenant FORCE RLS/least privilege and Run-owner schedule demand identity checks, preserve global event identity, and prohibit cleanup absent an approved retention rule. | The Run owner now stores the infrastructure result fact in its own schema boundary. |", "> | v1.8 |")

    plan = """### 6.100 Phase 9F4D-3D5 Run-owner capacity result consumer（2026-10-07）

| 工作项 | 本阶段实现 | 验收与未关闭门 | 状态 |
|---|---|---|---|
| Run-owner consumer | Rust wraps a pre-provisioned durable explicit-ack PullConsumer; fetch is one message, <=132 KiB and 2s. It validates schema-v2 results, hydrates canonical scope from immutable demand, appends the Run-owned Inbox fact, commits, then double-acks. | PostgreSQL/NATS disposable integration checks insert, exact duplicate replay, DB rollback/no-ack, bad identity/scope, ack retry and message/fetch bounds. Exact gate numbers are in the phase report. | Verified only after gates |
| Owner schema / WTM | Demand Outbox and result Inbox are append-only Transaction facts under tenant FORCE RLS; Run-owner is SELECT/INSERT only. | No Work or Master rows; no TTL/physical deletion. No TaskExecutionRun/Schedule/BI projection or reservation mutation. | Verified only after schema gate |
| Production boundary | No stream/consumer provisioning, broker identity deployment, poller, poison quarantine, release producer, provider/Manager, readiness or executor. | Bad messages remain unacknowledged. Capacity acceptance is not readiness. Zero Multipass VMs; two-record inventory cap remains. | Open |
| Documents/tooling | Synchronize requirements v5.98, basic design v5.95, Group DD v4.64, Schedule SRS v0.27/BD-DD v0.26, Infrastructure DD v0.25, Data Design v1.8, Automation Design v2.24 and registry. | Rust minimum remains 1.99.0. The idempotent documentation sync script is registered and rerun as a no-op. | Synced |

Phase report: PHASE-9F4D-3D5-RUN-OWNER-CONSUMER-REPORT.md.

"""
    insert_before("docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md", "## 修订履历", plan, "### 6.100 Phase 9F4D-3D5")

    automation = """### 4.65 Phase 9F4D-3D5 Run-owner capacity result consumer

**自动化档：[P]** — a guarded documentation synchronization script plus the disposable PostgreSQL/NATS real consumer runner.

- Run-owner Rust adapter wraps a supplied durable explicit-ack PullConsumer and fetches at most one message, 132 KiB, with 2-second expiry. It validates the versioned event contract and hydrates canonical scope from the exact stored demand row.
- The result Inbox is a Run-owner append-only Transaction fact protected by tenant FORCE RLS. The transaction commits before double-ack; duplicates are accepted only on exact immutable replay. No TaskExecutionRun/Schedule state, BI/RunEvent, reservation, readiness or execution is changed.
- The schema and integration runner use disposable PostgreSQL 18.6 and a loopback NATS Server; the runner owns and cleans only its own processes/data. No Multipass VM is created and the test inventory cap remains two registered instances.
- Malformed/unmatched events are not acknowledged and no durable quarantine exists; capacity_released has no verified producer. The consumer is not production deployable, and production worker/credentials/stream/provisioning/provider/Manager/execution gates remain open.
- Documentation automation: scripts/automation/phase9f4d3d5_sync.py is idempotent, guarded by exact anchors, registered in scripts/automation/registry.md, and must be run twice with the second pass producing no changes.
- Phase report: docs/reports/PHASE-9F4D-3D5-RUN-OWNER-CONSUMER-REPORT.md.

"""
    insert_before("docs/automation-design.md", "## 5. 守门基线", automation, "### 4.65 Phase 9F4D-3D5")

    registry_script = "| `scripts/automation/phase9f4d3d5_sync.py` | Idempotently sync Phase 9F4D-3D5 requirements/basic/detailed design, W/T/M appendix, and phase plan; exact-anchor guarded and safe to rerun | Phase 9F4D-3D5; automation-design §4.65 | This commit | 🟢 Ran twice; second pass no-op |"
    registry_runner = "| `scripts/automation/phase9f4d_run_owner_consumer.py` | Disposable PostgreSQL 18.6 + loopback NATS JetStream schema/RLS and Run-owner result Inbox integration runner; owns/stops only its processes and temp data; creates no Multipass VM | Phase 9F4D-3D5; automation-design §4.65; implementation plan §6.100; phase report | This commit | 🟢 20/20 runner steps and 1/1 real integration test passed; inventory unchanged; production consumer remains disabled |"
    registry_header = "| 脚本路径 | 用途 | 调用方 | 末次 commit | 状态 |\n|---|---|---|---|---|\n"
    replace_once("scripts/automation/registry.md", registry_header, registry_header + registry_script + "\n" + registry_runner + "\n")

    insert_after_prefix("docs/requirements.md", "| v5.97 |", "| v5.98 | 2026-10-07 | Mavis review | Add §50.8P and AC-INFRA-002-P for the bounded Run-owner JetStream result consumer and caller-transaction validation of scheduled demand against TaskExecutionRun rule/revision/occurrence/correlation; record immutable Inbox, commit-before-ack, fail-closed poison/release contracts, disposable PostgreSQL/NATS evidence, Rust 1.99.0, zero VMs, two-record cap and open production gates. | Phase 9F4D-3D5 adds the result-side owner Inbox without enabling execution. |", "| v5.98 |")
    insert_after_prefix("docs/basic-design.md", "| v5.94 |", "| v5.95 | 2026-10-07 | Mavis review | Add §16.40 for the Run-owner result Inbox and validate scheduled demand against TaskExecutionRun rule/revision/occurrence/correlation within its caller transaction; synchronize requirements v5.98, Group DD v4.64, Schedule contracts, Infrastructure DD v0.25 and Data Design v1.8. Record exact demand-derived result scope, commit-before-double-ack, no state projection, fail-closed poison/release gaps, Rust 1.99.0 and zero-VM/two-record test boundary. | Phase 9F4D-3D5 implements the result consumer slice. |", "| v5.95 |")
    insert_after_prefix("docs/design/DD-WORKTREE-GROUP-001.md", "| v4.63 |", "| v4.64 | 2026-10-07 | Mavis review | Add §8.14 for the Run-owner result Inbox and caller-transaction validation of scheduled demand against TaskExecutionRun rule/revision/occurrence/correlation; record exact result scope, bounded durable pull and commit-before-ack. Preserve no TaskExecutionRun/BI mutation and production-disabled poison/release boundaries; align requirements/basic design and phase evidence. | Phase 9F4D-3D5 adds the result-side Run-owner contract. |", "| v4.64 |")
    insert_after_prefix("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "| v0.26 |", "| v0.27 | 2026-10-07 | Add SCHED-API-026 for caller-transaction validation of scheduled demand against TaskExecutionRun rule/revision/occurrence/correlation and bounded Run-owner result consumption, exact demand binding, append-only Inbox, commit-before-double-ack and fail-closed poison/release handling; record zero-VM PostgreSQL/NATS evidence while keeping production consumer disabled. | Phase 9F4D-3D5 adds the Run-side result Inbox. |", "| v0.27 |")
    insert_after_prefix("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "| v0.25 |", "| v0.26 | 2026-10-07 | Add section 6.11 for caller-transaction TaskExecutionRun scope validation and the Run-owner result consumer's transaction/ack/fail-closed boundary; synchronize SRS v0.27, DD v0.26, requirements v5.98 and basic design v5.95. | Phase 9F4D-3D5 adds a verified result Inbox slice without enabling production execution. |", "| v0.26 |")
    insert_after_prefix("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "| v0.25 |", "| v0.26 | 2026-10-07 | Add section 6.12 for caller-transaction TaskExecutionRun rule/revision/occurrence/correlation validation, durable explicit-ack pull, one-message byte/time bounds, immutable demand matching, Run-owner transaction, exact replay and the no-quarantine production block. | Phase 9F4D-3D5 verifies Run-owned result persistence and acknowledgement ordering. |", "| v0.26 |")
    insert_after_prefix("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "| v0.24 |", "| v0.25 | 2026-10-07 | Add §7.11 for TaskExecutionRun-validated scheduled demand append, the Run-owner capacity result consumer and append-only tenant Inbox; record exact demand-derived scope, commit-before-double-ack, no mutable Run projection, no quarantine/release contract, zero-VM test and two-record cap. | Phase 9F4D-3D5 adds the result-side owner boundary. |", "| v0.25 |")
    insert_after_prefix("docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md", "| v6.26 |", "| v6.27 | 2026-10-07 | Mavis review | Add §6.100 for Phase 9F4D-3D5, including bounded consumer, append-only Run Inbox, owner W/T/M classification, integration evidence and remaining production gates; align the requirements and design chain. | The Run owner now has an explicit result-side Inbox slice. |", "| v6.27 |")
    insert_after_prefix("docs/automation-design.md", "| v2.23 |", "| v2.24 | 2026-10-07 | Add §4.65 and register the idempotent guarded documentation sync; record Phase 9F4D-3D5 result-consumer bounds, caller-transaction TaskExecutionRun scope check, transaction/ack order, fail-closed quarantine/release boundary, Rust 1.99.0, zero Multipass VMs and two-record cap. | The Run-owner result Inbox consumer slice reached its focused verification gate. |", "| v2.24 |")
    insert_after_prefix("scripts/automation/registry.md", "| **v0.70** |", "| **v0.71** | **2026-10-07** | **Mavis review** | Register the Phase 9F4D-3D5 requirements/design synchronization script and record the result-consumer doc chain; keep the second-run no-op check in the phase gate. | Run-owner result Inbox and its documentation are synchronized. |", "| **v0.71** |")

    wt_path = ROOT / "docs" / "data-design" / "phase9f4d-run-owner-wtm.md"
    wt_text = """# Phase 9F4D Run-owner W/T/M Classification

> Scope: Run/Execution owner capacity-demand and Infrastructure-result facts introduced by Phase 9F4D-3D5.
> Source migration: db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql
> Parent: docs/data-design.md §4.13.9; W/T/M version v1.8.

## Classification

| Table | Primary class | Why | Retention and mutation |
|---|---|---|---|
| multica.infrastructure_capacity_demand_outbox | Transaction (T) | Immutable domain/integration fact that Run owner requested infrastructure capacity for one execution attempt and occurrence. | Append-only; UPDATE/DELETE/TRUNCATE blocked; no automatic TTL or deletion. |
| multica.infrastructure_capacity_result_inbox | Transaction (T) | Immutable owner-received fact and durable global event_id dedupe record for the corresponding demand. | Append-only; UPDATE/DELETE/TRUNCATE blocked; no automatic TTL or deletion. |

Work (W): none. Master (M): none. The word Outbox does not make the demand a delivery Work row: Infrastructure owns its separate mutable delivery lease/state table. This migration has no short-lived job state.

## Owner and access matrix

Both tables enable and FORCE tenant RLS on tenant_id. The NOLOGIN, NOSUPERUSER, NOBYPASSRLS star_multica_run_owner role has schema USAGE and SELECT/INSERT on the two facts only; it has SELECT-only access to TaskExecutionRun origin, Schedule Rule ID/revision, occurrence, correlation and key scope columns so the owner can validate the typed demand inside the caller transaction. UPDATE, DELETE, TRUNCATE and access from other owner roles are revoked. The result's composite foreign key binds event identity and canonical scope to the demand fact; a global event_id unique constraint prevents cross-tenant reuse.

The Rust consumer derives canonical Project, Branch, Engineering Run, Work Item, TaskExecutionRun and occurrence scope from stored demand, writes one result fact in the Run owner transaction, then double-acks JetStream. No TaskExecutionRun/Schedule update or BI event occurs. No physical cleanup is authorized until a separate audit/legal retention rule and immutable export design are reviewed.

## Evidence boundary

The schema/integration runner uses disposable PostgreSQL and loopback NATS, creates no Multipass VM and keeps the registered Multipass inventory cap at two, including stopped/deleted records. Isolated schema/RLS/idempotency evidence does not prove target database grants, production broker provisioning, poison-message quarantine, release producer, or execution readiness.
"""
    if wt_path.exists():
        if wt_path.read_text(encoding="utf-8") != wt_text:
            raise RuntimeError(f"existing W/T/M document differs from expected: {wt_path}")
    else:
        wt_path.parent.mkdir(parents=True, exist_ok=True)
        wt_path.write_text(wt_text, encoding="utf-8", newline="\n")
        CHANGES += 1

    print(f"Phase 9F4D-3D5 document sync complete; changed_files={CHANGES}")


if __name__ == "__main__":
    main()
