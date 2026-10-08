#!/usr/bin/env python3
"""Idempotently synchronize the Phase 9F4D-3D7 verification across design artifacts."""

# @cypher schema=1 source_sha256=5196ca996beac3c3a06cf730da7a207b62aaeb47bee2bb06182acb3625fa4105
# MERGE (self:File {path:"scripts/automation/phase9f4d3d7_sync.py"})
# MERGE (mutate:Symbol {id:"scripts/automation/phase9f4d3d7_sync.py::mutate",kind:"function"})
# MERGE (replace_once:Symbol {id:"scripts/automation/phase9f4d3d7_sync.py::replace_once",kind:"function"})
# MERGE (insert_before:Symbol {id:"scripts/automation/phase9f4d3d7_sync.py::insert_before",kind:"function"})
# MERGE (insert_after_prefix:Symbol {id:"scripts/automation/phase9f4d3d7_sync.py::insert_after_prefix",kind:"function"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4d3d7_sync.py::main",kind:"function"})
# MERGE (requirements:File {path:"docs/requirements.md"})
# MERGE (basic:File {path:"docs/basic-design.md"})
# MERGE (schedule_srs:File {path:"docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md"})
# MERGE (schedule_bd:File {path:"docs/design/BD-AUTOMATION-SCHEDULE-API-001.md"})
# MERGE (schedule_dd:File {path:"docs/design/DD-AUTOMATION-SCHEDULE-API-001.md"})
# MERGE (group_dd:File {path:"docs/design/DD-WORKTREE-GROUP-001.md"})
# MERGE (infra_dd:File {path:"docs/design/DD-LOCAL-INFRASTRUCTURE-001.md"})
# MERGE (automation:File {path:"docs/automation-design.md"})
# MERGE (plan:File {path:"docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md"})
# MERGE (registry:File {path:"scripts/automation/registry.md"})
# MERGE (self)-[:DEFINES]->(mutate)
# MERGE (self)-[:DEFINES]->(replace_once)
# MERGE (self)-[:DEFINES]->(insert_before)
# MERGE (self)-[:DEFINES]->(insert_after_prefix)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (main)-[:CALLS]->(replace_once)
# MERGE (main)-[:CALLS]->(insert_before)
# MERGE (main)-[:CALLS]->(insert_after_prefix)
# MERGE (replace_once)-[:CALLS]->(mutate)
# MERGE (insert_before)-[:CALLS]->(mutate)
# MERGE (insert_after_prefix)-[:CALLS]->(mutate)
# MERGE (mutate)-[:WRITES]->(requirements)
# MERGE (mutate)-[:WRITES]->(basic)
# MERGE (mutate)-[:WRITES]->(schedule_srs)
# MERGE (mutate)-[:WRITES]->(schedule_bd)
# MERGE (mutate)-[:WRITES]->(schedule_dd)
# MERGE (mutate)-[:WRITES]->(group_dd)
# MERGE (mutate)-[:WRITES]->(infra_dd)
# MERGE (mutate)-[:WRITES]->(automation)
# MERGE (mutate)-[:WRITES]->(plan)
# MERGE (mutate)-[:WRITES]->(registry)
# @endcypher

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def mutate(relative_path: str, transform) -> bool:
    path = ROOT / relative_path
    raw = path.read_bytes()
    newline = b"\r\n" if b"\r\n" in raw else b"\n"
    original = raw.decode("utf-8").replace("\r\n", "\n")
    updated = transform(original)
    if updated == original:
        return False
    path.write_bytes(updated.replace("\n", newline.decode("ascii")).encode("utf-8"))
    return True


def replace_once(relative_path: str, old: str, new: str, marker: str) -> bool:
    def transform(text: str) -> str:
        if marker in text:
            return text
        count = text.count(old)
        if count != 1:
            raise ValueError(f"{relative_path}: expected one source anchor, found {count}: {old!r}")
        return text.replace(old, new, 1)

    return mutate(relative_path, transform)


def insert_before(relative_path: str, anchor: str, addition: str, marker: str) -> bool:
    def transform(text: str) -> str:
        if marker in text:
            marker_index = text.index(marker)
            anchor_index = text.find(anchor, marker_index + len(marker))
            if anchor_index < 0:
                raise ValueError(f"{relative_path}: inserted section lost its trailing anchor {anchor!r}")
            prefix = text[:marker_index].rstrip("\n")
            section = text[marker_index:anchor_index].rstrip("\n")
            suffix = text[anchor_index:].lstrip("\n")
            return prefix + "\n\n" + section + "\n\n" + suffix
        count = text.count(anchor)
        if count != 1:
            raise ValueError(f"{relative_path}: expected one insertion anchor, found {count}: {anchor!r}")
        anchor_index = text.index(anchor)
        prefix = text[:anchor_index].rstrip("\n")
        suffix = text[anchor_index:].lstrip("\n")
        return prefix + "\n\n" + addition.rstrip() + "\n\n" + suffix

    return mutate(relative_path, transform)


def insert_after_prefix(relative_path: str, prefix: str, addition: str, marker: str) -> bool:
    def transform(text: str) -> str:
        if marker in text:
            return text
        lines = text.splitlines(keepends=True)
        matches = [index for index, line in enumerate(lines) if line.startswith(prefix)]
        if len(matches) != 1:
            raise ValueError(f"{relative_path}: expected one line prefix, found {len(matches)}: {prefix!r}")
        index = matches[0]
        ending = "\n" if lines[index].endswith("\n") else ""
        lines.insert(index + 1, addition.rstrip() + ending)
        return "".join(lines)

    return mutate(relative_path, transform)


def main() -> int:
    changed = 0
    changes = [
        ("docs/requirements.md", "# Vibe Coding Work Management SaaS 要件定义书（统合扩展版 v6.06）", "# Vibe Coding Work Management SaaS 要件定义书（统合扩展版 v6.07）", "统合扩展版 v6.07"),
        ("docs/basic-design.md", "> **文档版本**: v6.03 (2026-10-08)", "> **文档版本**: v6.04 (2026-10-08)", "> **文档版本**: v6.04"),
        ("docs/basic-design.md", "> **上游要件定义书**: docs/requirements.md v6.06", "> **上游要件定义书**: docs/requirements.md v6.07", "requirements.md v6.07"),
        ("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "> Version: 0.29 | Date: 2026-10-08", "> Version: 0.30 | Date: 2026-10-08", "> Version: 0.30"),
        ("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "the 3D6 quarantine persistence/consumer hook are verified", "the 3D6 quarantine persistence/consumer hook and 3D7 real JetStream redelivery verification are verified", "3D7 real JetStream redelivery verification are verified"),
        ("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "> Version: 0.28 | Date: 2026-10-08", "> Version: 0.29 | Date: 2026-10-08", "> Version: 0.29"),
        ("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "bounded JetStream publication and real PubAck integration are verified in focused/disposable gates", "bounded JetStream publication, real PubAck integration, and 3D7 redelivery/duplicate-ACK integration are verified in focused/disposable gates", "3D7 redelivery/duplicate-ACK integration are verified"),
        ("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "> Version: 0.28 | Date: 2026-10-08", "> Version: 0.29 | Date: 2026-10-08", "> Version: 0.29"),
        ("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "and 3D6 quarantine persistence/consumer hook are verified", "3D6 quarantine persistence/consumer hook and 3D7 real JetStream redelivery/duplicate ACK are verified", "3D7 real JetStream redelivery/duplicate ACK are verified"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "> **渡口 Project / Branch / Engineering Run / Worktree 与 Run Apps 详细设计 v4.72**", "> **渡口 Project / Branch / Engineering Run / Worktree 与 Run Apps 详细设计 v4.73**", "详细设计 v4.73"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "docs/requirements.md`](../requirements.md) v6.06", "docs/requirements.md`](../requirements.md) v6.07", "requirements.md) v6.07"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "docs/basic-design.md`](../basic-design.md) v6.03", "docs/basic-design.md`](../basic-design.md) v6.04", "basic-design.md) v6.04"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "[`DD-AUTOMATION-SCHEDULE-API-001.md`](DD-AUTOMATION-SCHEDULE-API-001.md) v0.20", "[`DD-AUTOMATION-SCHEDULE-API-001.md`](DD-AUTOMATION-SCHEDULE-API-001.md) v0.29", "DD-AUTOMATION-SCHEDULE-API-001.md) v0.29"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "[`DD-LOCAL-INFRASTRUCTURE-001.md`](DD-LOCAL-INFRASTRUCTURE-001.md) v0.31", "[`DD-LOCAL-INFRASTRUCTURE-001.md`](DD-LOCAL-INFRASTRUCTURE-001.md) v0.32", "DD-LOCAL-INFRASTRUCTURE-001.md) v0.32"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "实施计划 §6.107 v6.39", "实施计划 §6.108 v6.40", "实施计划 §6.108 v6.40"),
        ("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "> v0.31 · 2026-10-08", "> v0.32 · 2026-10-08", "> v0.32 · 2026-10-08"),
        ("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "requirements v6.06 §50.8G-50.8W / basic-design v6.03 §8.1、§16.21-16.47 / Schedule SRS v0.28 / Schedule BD v0.27 / Schedule DD v0.27 / Automation Design v2.28 / W/T/M v0.10 / 实施计划 §6.83-6.107。", "requirements v6.07 §50.8G-50.8X / basic-design v6.04 §8.1、§16.21-16.48 / Schedule SRS v0.30 / Schedule BD v0.29 / Schedule DD v0.29 / Automation Design v2.29 / W/T/M v0.10 / 实施计划 §6.83-6.108。", "实施计划 §6.83-6.108。"),
        ("docs/automation-design.md", "> **文档版本**: v2.28 (2026-10-08)", "> **文档版本**: v2.29 (2026-10-08)", "> **文档版本**: v2.29"),
        ("docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md", "> **渡口 Project / Cloud Branch / Engineering Run 架构与实施计划 v6.39**", "> **渡口 Project / Cloud Branch / Engineering Run 架构与实施计划 v6.40**", "实施计划 v6.40"),
        ("scripts/automation/registry.md", "> **文档版本**: v0.75 (2026-10-08)", "> **文档版本**: v0.76 (2026-10-08)", "> **文档版本**: v0.76"),
    ]
    for args in changes:
        changed += int(replace_once(*args))

    requirement_section = """### 50.8X Phase 9F4D-3D7 JetStream quarantine redelivery verification

The Phase 9F4D-3D7 gate runs the existing Run-owner result consumer against a disposable PostgreSQL 18.6 database and loopback NATS JetStream 2.15.0 server. It configures a trusted deployment/stream generation and explicit metadata-only retention. A deterministic receipt-insert failure must leave the malformed delivery pending with no receipt; after removing the fault, the actual redelivery must commit one receipt before a confirmed Term ACK and must not be redelivered again. A second case commits a receipt, drops the unacknowledged Message and fetch handles to model consumer loss, then verifies exact redelivery is recognized as a duplicate, Term-acked, and not delivered again. The test does not kill or restart an OS process. Shared-subject tests run serially.

This is integration evidence for the existing Platform quarantine owner/Run-owner hook only. It creates no schema or migration, does not access Multipass, and does not create or modify VM inventory. It does not provision production database/NATS identities, KMS, operator replay, production consumers, Schedule/Task Card runs, reservations, readiness, Run BI, or Benchmark; production execution remains disabled and fail-closed.

| Acceptance ID | Acceptance criterion |
|---|---|
| AC-SCHED-API-028 | Disposable PostgreSQL/NATS integration verifies storage failure leaves the real delivery pending, successful redelivery commits one metadata-only receipt before confirmed Term ACK, and dropping the unacknowledged Message/fetch handles after commit replays as an exact duplicate (consumer-loss model; no OS process restart); neither case is delivered again. Rust target passes 2/2; cumulative PostgreSQL/NATS runner passes 22/22 steps; zero VM and unchanged Multipass inventory. Production consumer and execution remain disabled. |

"""
    changed += int(insert_before("docs/requirements.md", "### 50.9 追溯与后续专题同步", requirement_section, "### 50.8X Phase 9F4D-3D7"))

    basic_section = """### 16.48 Phase 9F4D-3D7 JetStream quarantine integration gate

The existing Platform quarantine persistence and opt-in Run-owner hook are verified end to end for two recovery paths using a disposable PostgreSQL 18.6 database and loopback JetStream 2.15.0. Explicit metadata-only policy is used: injected receipt-write failure leaves the source delivery pending until the actual redelivery commits; the consumer then sends and awaits confirmed Term ACK. After a separately committed receipt, the test drops the unacknowledged Message and fetch handles and verifies exact redelivery is recognized as a duplicate and Term-acked. This models consumer loss without killing or restarting an OS process. Both paths assert no further redelivery.

The shared-subject integration target is serialized to avoid stream overlap. The cumulative runner passed 22/22 steps and the ignored Rust target passed 2/2. The gate creates zero VMs and does not change Multipass inventory or schema. Production DB/NATS identities, real KMS, operator replay, production consumer composition, Schedule/Task CLI, readiness, Run BI and Benchmark remain closed.

| Acceptance ID | Design mapping and boundary |
|---|---|
| AC-SCHED-API-028 | Requirements §50.8X; Schedule DD §6.15; database failure and actual redelivery, confirmed commit-before-Term ACK, exact duplicate after dropping unacknowledged Message/fetch handles (consumer-loss model; no OS process restart), and no subsequent delivery; metadata-only, zero VM and no inventory change. Production consumers and execution remain fail-closed. |

"""
    changed += int(insert_before("docs/basic-design.md", "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |", basic_section, "### 16.48 Phase 9F4D-3D7"))

    schedule_srs_row = "| SCHED-API-028 | A disposable PostgreSQL/NATS integration MUST prove that receipt storage failure leaves the malformed source delivery pending; successful real redelivery commits exactly one metadata-only receipt before confirmed Term ACK; after receipt commit, dropping the unacknowledged Message and fetch handles (consumer-loss model; no OS process kill/restart) is recognized as an exact duplicate after redelivery and Term-acked; no further redelivery occurs. The shared-subject Rust test target runs serially. This verifies no production consumer, operator replay, KMS, Schedule execution, Run/BI mutation, reservation, readiness, or Task Card CLI. |"
    changed += int(insert_after_prefix("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "| SCHED-API-027 |", schedule_srs_row, "| SCHED-API-028 |"))

    schedule_bd_section = """### 6.13 Phase 9F4D-3D7 redelivery and confirmed-Term verification

The existing quarantine consumer hook is tested against real loopback JetStream and disposable PostgreSQL, not a mock broker. The first case injects a receipt INSERT failure and proves no receipt is committed while the source delivery remains ack-pending; after removing the fault, JetStream redelivers it and the Platform owner commits exactly one metadata-only receipt before the consumer awaits confirmed Term ACK. The second case records the receipt, drops the unacknowledged Message and fetch handles, and verifies exact duplicate recognition and Term ACK after the real redelivery. This models consumer loss; it does not kill or restart an OS process. Each case checks zero later redelivery.

The two tests share a subject and therefore run with `--test-threads=1`; the disposable runner passed 22/22 steps and the ignored Rust target passed 2/2. No migration, VM, or Multipass inventory change is part of this gate. Production broker/DB identity, KMS, operator replay and consumer enablement remain separate gates; Schedule and Task Card execution remain fail-closed.

"""
    changed += int(insert_before("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "## 7. Revision history", schedule_bd_section, "### 6.13 Phase 9F4D-3D7"))

    schedule_dd_section = """### 6.15 Phase 9F4D-3D7 JetStream quarantine redelivery gate

This gate exercises the existing `RunOwnerCapacityResultConsumer::with_quarantine` and `PlatformPoisonQuarantine` against a disposable PostgreSQL 18.6 database and real loopback JetStream 2.15.0. The configured source identity is trusted by the fixture and the retention policy is explicitly `MetadataOnly`; payload Work rows must remain absent.

The fault-injection case installs a temporary trigger that rejects receipt insertion. The malformed source message remains ack-pending and no receipt exists. After removing the trigger and waiting past the short test AckWait, the actual redelivery stores exactly one receipt, leaves no payload row, waits for confirmed `double_ack_with(AckKind::Term)`, and is not delivered again. The pre-ACK handle-drop case directly commits the immutable receipt, drops the unacknowledged Message and fetch handles, waits past AckWait, and has real redelivery resolve as the exact duplicate outcome; confirmed Term ACK ends delivery and the receipt count remains one. This models consumer loss but does not kill or restart an OS process.

The Rust integration target is serialized because both tests use the same JetStream subject. The focused target passed 2/2; the cumulative disposable PostgreSQL/NATS runner passed 22/22 steps, stopped both services, and removed its owned PostgreSQL data. It created zero VM and left Multipass inventory unchanged. No schema was added by this phase. This proves neither production database/NATS identity activation, real KMS, operator replay, nor production consumer rollout; Schedule/Task execution and all Run/BI/reservation/readiness changes remain disabled.

"""
    changed += int(insert_before("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "## 7. Revision history", schedule_dd_section, "### 6.15 Phase 9F4D-3D7"))

    group_section = """### 8.21 Phase 9F4D-3D7 quarantine recovery evidence

The disposable JetStream/PostgreSQL redelivery gate belongs to the Platform Messaging/Ingress and Run-owner reliability boundary. It does not add a Worktree/Run navigation node, change Project → Cloud Branch → Engineering Run → local Worktree ownership, or make a selected Worktree the owner of quarantine facts. The future Run Apps continue to use authorized RunContext projections; Worktree selection remains focus/CLI checkout context.

The gate verified receipt failure backpressure, commit-before-confirmed-Term ACK, exact duplicate handling after dropping the unacknowledged Message/fetch handles (consumer-loss model; no OS process restart), and no later delivery. Its 22/22 runner steps and 2/2 Rust integration tests used metadata-only retention, zero VMs and unchanged Multipass inventory. It does not establish production consumer/provider wiring, KMS, operator App, BI/Benchmark, Run readiness, Schedule or Task Card execution; those remain closed until their owner and authorization gates pass.

| Acceptance ID | Group design boundary |
|---|---|
| AC-SCHED-API-028 | Schedule DD §6.15; verification evidence only for the existing Platform quarantine owner and Run-owner hook. Preserve Run/App ownership and Worktree focus semantics; no production execution or BI state is implied. |

"""
    changed += int(insert_before("docs/design/DD-WORKTREE-GROUP-001.md", "## §9 跨 App 事件与一致性", group_section, "### 8.21 Phase 9F4D-3D7"))

    infra_section = """### §7.18 Phase 9F4D-3D7 external broker reliability gate

The Schedule/Platform quarantine slice uses a disposable PostgreSQL control-plane fixture and a loopback NATS JetStream test server to verify receipt failure backpressure, real redelivery, exact duplicate replay after dropping the unacknowledged Message/fetch handles (consumer-loss model; no OS process restart), and confirmed Term ACK. This is an external messaging reliability test, not a Multipass or K3s provider operation. It creates no VM, reads or mutates no Multipass inventory, adds no Infrastructure catalog table/event, and does not prove Host/cluster capacity, target K3s health, Run binding, Agent sandbox, BI, or readiness.

The Rust target passed 2/2 integration tests and the cumulative runner passed 22/22 steps. See Schedule DD §6.15 for the source identity and ack/receipt sequence. Production control-plane DB identity, broker identity, KMS and consumer deployment remain open and fail-closed.

"""
    changed += int(insert_before("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "## §8 审阅与修订历史", infra_section, "### §7.18 Phase 9F4D-3D7"))

    automation_section = """### 4.70 Phase 9F4D-3D7 JetStream quarantine redelivery gate

Run `python scripts/automation/phase9f4d_run_owner_consumer.py` with PostgreSQL 18.6, Rust/Cargo 1.99.0 and NATS Server 2.15.0. The runner applies the existing quarantine migration and schema assertions, runs the existing Run-owner RLS/result checks, then executes the ignored JetStream integration target. The two tests share a subject, so the runner uses `--test-threads=1` and writes audit/log artifacts to a new unique directory under `target/`.

The 2026-10-08 gate passed 22/22 steps and both Rust integration tests. It verified receipt-insert-failure redelivery and exact duplicate redelivery after dropping unacknowledged Message/fetch handles (consumer-loss model; no OS process restart), using explicit metadata-only retention; NATS and PostgreSQL stopped successfully, owned PostgreSQL data was removed, and zero VMs/Multipass inventory changes were recorded. This is test-only evidence: no schema or migration was added, and production identity/KMS/operator/consumer/Schedule/Task execution remains disabled.

- Phase report: [`PHASE-9F4D-3D7-QUARANTINE-JETSTREAM-REPORT.md`](reports/PHASE-9F4D-3D7-QUARANTINE-JETSTREAM-REPORT.md).

"""
    changed += int(insert_before("docs/automation-design.md", "## 5. 守门基线", automation_section, "### 4.70 Phase 9F4D-3D7"))

    plan_section = """### 6.108 Phase 9F4D-3D7 JetStream quarantine redelivery gate（2026-10-08）

| Work item | Implementation result | Current gate/status |
|---|---|---|
| Failure/redelivery ordering | A disposable PostgreSQL trigger injects receipt INSERT failure; the malformed message stays pending and no receipt commits. Removing the fault allows actual JetStream redelivery to persist one metadata-only receipt before the confirmed Term ACK. | No later source delivery; payload Work table remains empty. |
| Pre-ACK handle-drop replay | Commit the immutable receipt, then drop the unacknowledged Message and fetch handles (consumer-loss model; no OS process is killed or restarted) and allow real AckWait redelivery. The Run-owner hook recognizes the exact receipt as duplicate and sends confirmed Term ACK. | Receipt remains exactly one; no later source delivery. |
| Verification | Rust 1.99.0 focused ignored integration target passed 2/2. PostgreSQL 18.6 + NATS Server 2.15.0 disposable runner passed 22/22 steps, stopped both processes and removed its owned PostgreSQL data. | Zero VM creation; Multipass inventory unchanged; no schema/migration added. |
| Cross-layer sync | requirements v6.07 §50.8X / AC-SCHED-API-028; basic design v6.04 §16.48; Schedule SRS v0.30 / BD v0.29 / DD v0.29 §6.15; Group DD v4.73 §8.21; Infrastructure DD v0.32 §7.18; automation-design v2.29 / registry v0.76. | Production KMS, DB/NATS identities, operator replay, production consumer, admission/provider/readiness, BI/Benchmark and Schedule/Task Card execution remain open; execution stays fail-closed. |

阶段报告：[PHASE-9F4D-3D7-QUARANTINE-JETSTREAM-REPORT.md](../reports/PHASE-9F4D-3D7-QUARANTINE-JETSTREAM-REPORT.md)。

"""
    changed += int(insert_before("docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md", "## 修订履历", plan_section, "### 6.108 Phase 9F4D-3D7"))

    registry_runner = "| `scripts/automation/phase9f4d_run_owner_consumer.py` | Disposable PostgreSQL 18.6 + loopback NATS JetStream schema/RLS and Run-owner result/quarantine integration runner; applies the existing quarantine migration/assertions, serializes the shared-subject Rust tests, owns/stops only its processes and temp data, creates no Multipass VM. | Phase 9F4D-3D5/3D7; automation-design §4.65/§4.70; plan §6.100/§6.108; phase reports | This commit | 3D7: 22/22 runner steps, Rust integration 2/2 (failure/redelivery and post-commit/pre-ACK duplicate); 0 VM, Multipass unchanged; production consumer/execution remains disabled. |"
    changed += int(replace_once("scripts/automation/registry.md", "| `scripts/automation/phase9f4d_run_owner_consumer.py` | Disposable PostgreSQL 18.6 + loopback NATS JetStream schema/RLS and Run-owner result Inbox integration runner; owns/stops only its processes and temp data; creates no Multipass VM | Phase 9F4D-3D5; automation-design §4.65; implementation plan §6.100; phase report | This commit | 🟢 20/20 runner steps and 1/1 real integration test passed; inventory unchanged; production consumer remains disabled |", registry_runner, "3D7: 22/22 runner steps"))
    registry_sync = "| `scripts/automation/phase9f4d3d7_sync.py` | Exact-anchor, idempotent 3D7 requirements/basic/Schedule/Group/Infrastructure/automation/plan/registry and phase-report index synchronization; second pass is a no-op. | Phase 9F4D-3D7; automation-design §4.70; plan §6.108 | This commit | 🟢 Ran twice; second pass made no changes. |"
    changed += int(insert_after_prefix("scripts/automation/registry.md", "| `scripts/automation/phase9f4d_run_owner_consumer.py` |", registry_sync, "| `scripts/automation/phase9f4d3d7_sync.py` |"))

    rows = [
        ("docs/requirements.md", "| v6.06 |", "| v6.07 | 2026-10-08 | Mavis review | Add §50.8X / AC-SCHED-API-028 for real JetStream quarantine failure/redelivery and exact duplicate after pre-ACK handle drop; record confirmed commit-before-Term ordering, 22/22 disposable PostgreSQL/NATS runner steps, 2/2 Rust integration tests, zero VM and unchanged inventory. Keep production KMS, identities, consumer, operator access and execution fail-closed. | Phase 9F4D-3D7 closes the bounded broker redelivery evidence gap. |", "| v6.07 |"),
        ("docs/basic-design.md", "| v6.03 |", "| v6.04 | 2026-10-08 | Mavis review | Add §16.48 / AC-SCHED-API-028 for database failure backpressure, real JetStream redelivery, confirmed Term ACK, exact duplicate replay after pre-ACK handle drop and no later delivery. Record 22/22 runner steps, 2/2 Rust integration tests, zero VM and unchanged Multipass inventory; production consumers and execution remain disabled. | 9F4D-3D7 verifies the existing quarantine seam without schema or navigation change. |", "| v6.04 |"),
        ("docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md", "| v0.29 |", "| v0.30 | 2026-10-08 | Record SCHED-API-028 and verified real JetStream delivery-pending, commit-before-confirmed-Term, exact duplicate replay after pre-ACK handle drop, no-redelivery and serial shared-subject test semantics; record 22/22 runner steps and 2/2 Rust integration tests. Keep production identity/KMS/consumer/Schedule execution fail-closed. | Phase 9F4D-3D7 closes the bounded quarantine delivery reliability gate. |", "| v0.30 |"),
        ("docs/design/BD-AUTOMATION-SCHEDULE-API-001.md", "| v0.28 |", "| v0.29 | 2026-10-08 | Add §6.13 for verified receipt-failure backpressure, actual JetStream redelivery, exact duplicate replay after pre-ACK handle drop, confirmed Term ACK and serial shared-subject tests; record 22/22 disposable runner steps and 2/2 Rust integration tests while leaving production consumer/KMS/identities and execution closed. | Phase 9F4D-3D7 verifies the existing Schedule/Platform boundary. |", "| v0.29 |"),
        ("docs/design/DD-AUTOMATION-SCHEDULE-API-001.md", "| v0.28 |", "| v0.29 | 2026-10-08 | Add §6.15 for real PostgreSQL/NATS failure, redelivery, committed-receipt duplicate and confirmed Term-ACK integration; specify metadata-only retention, no later delivery, serialized shared-subject tests, 22/22 runner steps and 2/2 integration tests. Keep production identity/KMS/operator/consumer and Schedule/Task execution disabled. | Phase 9F4D-3D7 closes only the disposable redelivery evidence gap. |", "| v0.29 |"),
        ("docs/design/DD-WORKTREE-GROUP-001.md", "| v4.72 |", "| v4.73 | 2026-10-08 | Mavis review | Add §8.21 / AC-SCHED-API-028 for the Platform/Run-owner quarantine recovery test boundary; preserve Project→Branch→Engineering Run→Worktree and Run App ownership, and state explicitly that broker reliability evidence adds no navigation node or execution/BI claim. Align requirements v6.07/basic v6.04/Schedule DD v0.29/Infrastructure DD v0.32/plan v6.40; record 22/22 runner steps, 2/2 Rust tests and zero VM. | 9F4D-3D7 verifies a cross-owner reliability seam without changing Group ownership. |", "| v4.73 |"),
        ("docs/design/DD-LOCAL-INFRASTRUCTURE-001.md", "| v0.31 |", "| v0.32 | 2026-10-08 | Mavis review | Add §7.18 to classify 9F4D-3D7 as external broker/PostgreSQL reliability evidence only; record 22/22 runner steps, 2/2 integration tests, zero VM and unchanged Multipass inventory. Preserve the two-record/no-purge rule and state that no Host/cluster capacity, K3s readiness, Run binding, BI or execution is proved. Align requirements/basic/Schedule/Group/plan version references. | Phase 9F4D-3D7 adds no Infrastructure schema or provider operation. |", "| v0.32 |"),
        ("docs/automation-design.md", "| v2.28 |", "| v2.29 | 2026-10-08 | Add §4.70 for the serialized shared-subject JetStream quarantine integration; register the idempotent cross-document sync script and record 22/22 runner steps, 2/2 Rust integration tests, successful service stop and owned PG cleanup, zero VM and unchanged Multipass inventory. Keep all production execution gates closed. | Phase 9F4D-3D7 verifies the broker recovery seam. |", "| v2.29 |"),
        ("docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md", "| v6.39 |", "| v6.40 | 2026-10-08 | Mavis review | Add §6.108 for Phase 9F4D-3D7; synchronize requirements v6.07, basic v6.04, Schedule SRS/BD/DD v0.30/v0.29/v0.29, Group DD v4.73, Infrastructure DD v0.32, automation v2.29 and registry v0.76. Record real PostgreSQL/NATS 22/22 runner steps, 2/2 Rust integration tests, zero VM and unchanged inventory; no schema change. Keep production identities/KMS/operator/consumer and all Run/Schedule/Task execution gates open. | 3D7 closes the real JetStream redelivery/ACK-loss evidence gap only. |", "| v6.40 |"),
        ("scripts/automation/registry.md", "| **v0.75** |", "| **v0.76** | **2026-10-08** | **Mavis review** | Register the exact-anchor 3D7 documentation sync script and update the disposable PostgreSQL/NATS runner entry for serial shared-subject integration; record 22/22 steps, 2/2 Rust tests, confirmed service stop/owned data cleanup, zero VMs and unchanged Multipass inventory. Preserve production KMS/identity/operator/consumer/execution gates. | Phase 9F4D-3D7 verifies failure redelivery and exact duplicate Term ACK. |", "| **v0.76** |"),
    ]
    for relative_path, prefix, addition, marker in rows:
        changed += int(insert_after_prefix(relative_path, prefix, addition, marker))

    handle_loss_updates = [
        (
            "docs/requirements.md",
            "A second case commits a receipt, drops the source message before ACK to model process loss, then verifies exact redelivery is recognized as a duplicate, Term-acked, and not redelivered again.",
            "A second case commits a receipt, then drops the unacknowledged Message and fetch handles to model consumer loss. The test does not kill or restart an OS process; it verifies exact JetStream redelivery is recognized as a duplicate, Term-acked, and not delivered again.",
            "drops the unacknowledged Message and fetch handles to model consumer loss",
        ),
        (
            "docs/requirements.md",
            "simulated post-commit/pre-ACK loss replays as an exact duplicate",
            "the pre-ACK Message/fetch-handle-drop model replays as an exact duplicate (no OS process is killed or restarted)",
            "pre-ACK Message/fetch-handle-drop model",
        ),
        (
            "docs/requirements.md",
            "exact duplicate after simulated process loss",
            "exact duplicate after the explicit pre-ACK handle-drop consumer-loss model (no OS process restart)",
            "explicit pre-ACK handle-drop consumer-loss model",
        ),
        (
            "docs/basic-design.md",
            "A separately committed receipt followed by a dropped pre-ACK source message is replayed as the exact duplicate and Term-acked.",
            "After committing the receipt, the test drops the unacknowledged Message and fetch handles and verifies exact redelivery is recognized as a duplicate and Term-acked. This models consumer loss without killing or restarting an OS process.",
            "models consumer loss without killing or restarting an OS process",
        ),
        (
            "docs/basic-design.md",
            "exact duplicate after simulated process loss",
            "exact duplicate after the pre-ACK handle-drop consumer-loss model (no OS process restart)",
            "pre-ACK handle-drop consumer-loss model",
        ),
        (
            "docs/basic-design.md",
            "exact duplicate replay after simulated process loss",
            "exact duplicate replay after pre-ACK handle drop",
            "exact duplicate replay after pre-ACK handle drop",
        ),
        (
            "docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md",
            "a receipt committed before simulated process loss is recognized as an exact duplicate after redelivery",
            "after the receipt commits, the test drops the unacknowledged Message and fetch handles (consumer-loss model; no OS process kill/restart) and requires exact duplicate recognition after redelivery",
            "consumer-loss model; no OS process kill/restart",
        ),
        (
            "docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md",
            "exact duplicate replay after simulated process loss",
            "exact duplicate replay after the pre-ACK handle-drop consumer-loss model (no OS process restart)",
            "pre-ACK handle-drop consumer-loss model",
        ),
        (
            "docs/design/BD-AUTOMATION-SCHEDULE-API-001.md",
            "The second case records the receipt, drops the unacknowledged message, and verifies exact duplicate recognition and Term ACK after the real redelivery.",
            "The second case records the receipt, drops the unacknowledged Message and fetch handles, and verifies exact duplicate recognition and Term ACK after real redelivery. This models consumer loss; it does not kill or restart an OS process.",
            "does not kill or restart an OS process",
        ),
        (
            "docs/design/BD-AUTOMATION-SCHEDULE-API-001.md",
            "exact duplicate replay after process-loss simulation",
            "exact duplicate replay after the pre-ACK handle-drop consumer-loss model (no OS process restart)",
            "pre-ACK handle-drop consumer-loss model",
        ),
        (
            "docs/design/DD-AUTOMATION-SCHEDULE-API-001.md",
            "The process-loss case directly commits the immutable receipt, drops the source message before ACK, waits past AckWait, and has the real redelivery resolve as the exact duplicate outcome; confirmed Term ACK ends delivery and the receipt count remains one.",
            "The pre-ACK handle-drop case directly commits the immutable receipt, drops the unacknowledged Message and fetch handles, waits past AckWait, and has real redelivery resolve as the exact duplicate outcome; confirmed Term ACK ends delivery and the receipt count remains one. This models consumer loss but does not kill or restart an OS process.",
            "does not kill or restart an OS process",
        ),
        (
            "docs/design/DD-WORKTREE-GROUP-001.md",
            "exact duplicate handling after simulated process loss",
            "exact duplicate handling after the pre-ACK handle-drop consumer-loss model (no OS process restart)",
            "pre-ACK handle-drop consumer-loss model",
        ),
        (
            "docs/design/DD-LOCAL-INFRASTRUCTURE-001.md",
            "exact duplicate replay after a simulated pre-ACK process loss",
            "exact duplicate replay after dropping the unacknowledged Message/fetch handles (consumer-loss model; no OS process restart)",
            "consumer-loss model; no OS process restart",
        ),
        (
            "docs/automation-design.md",
            "exact duplicate redelivery after a simulated process loss",
            "exact duplicate redelivery after dropping the unacknowledged Message/fetch handles (consumer-loss model; no OS process restart)",
            "consumer-loss model; no OS process restart",
        ),
        (
            "docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md",
            "| Process-loss replay | Commit the immutable receipt, drop the source message before ACK, and allow real AckWait redelivery. The Run-owner hook recognizes the exact receipt as duplicate and sends confirmed Term ACK. | Receipt remains exactly one; no later source delivery. |",
            "| Pre-ACK handle-drop replay | Commit the immutable receipt, then drop the unacknowledged Message and fetch handles (consumer-loss model; no OS process is killed or restarted) and allow real AckWait redelivery. The Run-owner hook recognizes the exact receipt as duplicate and sends confirmed Term ACK. | Receipt remains exactly one; no later source delivery. |",
            "Pre-ACK handle-drop replay",
        ),
    ]
    for args in handle_loss_updates:
        changed += int(replace_once(*args))

    print(
        f"phase9f4d3d7-sync: changed_operations={changed}\n"
        "Cross-layer docs updated; Data Design W/T/M unchanged because this phase adds no schema or persisted fact."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
