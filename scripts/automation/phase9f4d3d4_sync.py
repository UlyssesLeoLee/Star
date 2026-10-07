from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# @cypher schema=1 source_sha256=106f1a1c1faaaf4d0a850d1130a03aae9b3b377dfeec189f0c7414ab0c39bf2a
# MERGE (self:File {path:"scripts/automation/phase9f4d3d4_sync.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4d3d4_sync.py::main",kind:"function"})
# MERGE (sync:Symbol {id:"scripts/automation/phase9f4d3d4_sync.py::sync",kind:"function"})
# MERGE (main)-[:CALLS]->(sync)
# @endcypher

def read(path):
    raw = path.read_bytes()
    nl = "\r\n" if b"\r\n" in raw else "\n"
    return raw.decode("utf-8").replace("\r\n", "\n"), nl


def write(path, text, nl):
    path.write_bytes(text.replace("\n", nl).encode("utf-8"))


def replace(path, old, new):
    text, nl = read(path)
    if new in text:
        return 0
    if text.count(old) != 1:
        raise RuntimeError(f"expected one source anchor in {path}: {old[:100]!r}")
    write(path, text.replace(old, new, 1), nl)
    return 1


def patch(path, old, new):
    return replace(path, old, new)

def insert(path, anchor, block):
    text, nl = read(path)
    if block.strip() in text:
        return 0
    if text.count(anchor) != 1:
        raise RuntimeError(f"expected one insertion anchor in {path}: {anchor[:100]!r}")
    write(path, text.replace(anchor, block.rstrip() + "\n\n" + anchor, 1), nl)
    return 1


def header(path, prefix, old, new):
    text, nl = read(path)
    lines = text.splitlines()
    hits = [i for i, line in enumerate(lines) if line.startswith(prefix)]
    if not hits and old in prefix:
        applied_prefix = prefix.replace(old, new, 1)
        if any(line.startswith(applied_prefix) for line in lines):
            return 0
    if len(hits) != 1:
        raise RuntimeError(f"expected one header in {path}: {prefix!r}")
    i = hits[0]
    if new in lines[i]:
        return 0
    if old not in lines[i]:
        raise RuntimeError(f"version anchor missing in {path}: {old}")
    lines[i] = lines[i].replace(old, new, 1)
    write(path, "\n".join(lines) + "\n", nl)
    return 1


def patch_line(path, prefix, old, new):
    text, nl = read(path)
    lines = text.splitlines()
    hits = [i for i, line in enumerate(lines) if line.startswith(prefix)]
    if len(hits) != 1:
        raise RuntimeError(f"expected one line in {path}: {prefix!r}")
    i = hits[0]
    if new in lines[i]:
        return 0
    if old not in lines[i]:
        raise RuntimeError(f"line anchor missing in {path}: {old!r}")
    lines[i] = lines[i].replace(old, new, 1)
    write(path, "\n".join(lines) + "\n", nl)
    return 1



def set_line(path, prefix, new_line):
    text, nl = read(path)
    lines = text.splitlines()
    hits = [i for i, line in enumerate(lines) if line.startswith(prefix)]
    if len(hits) != 1:
        raise RuntimeError(f"expected one line in {path}: {prefix!r}")
    if lines[hits[0]] == new_line:
        return 0
    lines[hits[0]] = new_line
    write(path, "\n".join(lines) + "\n", nl)
    return 1

def revision(path, anchor, row, version):
    text, nl = read(path)
    lines = text.splitlines()
    if any(line.startswith((f"| {version} |", f"| **{version}** |")) for line in lines):
        return 0
    hits = [i for i, line in enumerate(lines) if line.startswith(anchor)]
    if len(hits) != 1:
        raise RuntimeError(f"expected one revision anchor in {path}: {anchor!r}")
    lines.insert(hits[0] + 1, row)
    write(path, "\n".join(lines) + "\n", nl)
    return 1


def sync():
    changed = 0
    req = ROOT / "docs/requirements.md"
    changed += header(req, "# Vibe Coding Work Management SaaS", "v5.96", "v5.97")
    changed += insert(req, "### 50.9", """### 50.8O Phase 9F4D-3D4 bounded JetStream Outbox publisher

Infrastructure Outbox delivery rows may be published to NATS JetStream only by an explicitly invoked Rust publisher receiving an already authenticated JetStream context and the tenant-scoped Infrastructure owner store. It MUST NOT discover tenants, start a polling worker, configure credentials or streams, or enable TaskExecutionRun execution. The publisher leases at most 8 rows per tenant, uses a 120..300 second lease, and serially waits for each PubAck.

Only capacity_accepted, capacity_rejected, capacity_released, and capacity_expired schema-version-2 facts may be published under the fixed star.infrastructure.v2 subject prefix. The publisher MUST reject malformed/non-object JSON and payloads over 16 KiB, bound the serialized envelope to 128 KiB, set Nats-Msg-Id to the immutable event UUID, and mark PostgreSQL delivery as delivered only after PubAck. A timeout is ambiguous and MUST be retried with the same message ID. The Run/Execution owner MUST deduplicate event_id in its durable Inbox beyond the broker duplicate window.

This slice verifies the Rust transport against loopback NATS Server 2.15.0 and the disposable PostgreSQL owner schema. It does not provision a production broker, add a poller, establish target DB roles, implement the Run/Execution consumer, or enable execution. It creates no Multipass VM; the test inventory remains capped at two registered records total, including stopped/deleted entries, with no automatic purge.

| Acceptance ID | Acceptance criterion |
|---|---|
| AC-INFRA-002-O | Given a bounded tenant-scoped Infrastructure Outbox claim, the Rust adapter MUST publish only allowlisted schema-v2 capacity events using immutable event_id as Nats-Msg-Id, wait for PubAck before marking delivered, record ambiguous failures through bounded retry/dead-letter, and preserve durable Run-owner Inbox dedupe. It MUST use an injected authenticated context, remain serial and bounded, and MUST NOT create a worker, tenant scheduler, production broker configuration, target DB identity, consumer, or execution path. |""")
    changed += revision(req, "| v5.96 |", "| v5.97 | 2026-10-07 | Mavis review | Add section 50.8O and AC-INFRA-002-O for bounded Rust JetStream publication; pin immutable event_id as Nats-Msg-Id, require PubAck before PostgreSQL delivered state, serial claims and durable Run Inbox dedupe; record real loopback NATS/PostgreSQL integration and Rust 1.99.0, zero Multipass VMs and open production consumer/worker/provider/execution gates. | The Infrastructure Outbox state machine gained a bounded transport adapter. |", "v5.97")

    basic = ROOT / "docs/basic-design.md"
    changed += header(basic, "> **文档版本**:", "v5.93", "v5.94")
    changed += patch(basic, "> **上游要件定义书**: docs/requirements.md v5.96", "> **上游要件定义书**: docs/requirements.md v5.97")
    changed += insert(basic, "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |", """### 16.39 Phase 9F4D-3D4 bounded Infrastructure Outbox JetStream publisher

The Infrastructure owner exposes a Rust adapter receiving an authenticated async-nats JetStream context and the existing PostgreSQL Outbox delivery store. It claims 1..8 rows for one tenant with a 120..300 second lease and publishes serially while awaiting each PubAck. It accepts only the four schema-v2 capacity facts, uses the fixed star.infrastructure.v2 subject namespace, bounds each event JSON to 16 KiB and each serialized envelope to 128 KiB, and sets Nats-Msg-Id to the immutable event UUID.

The PostgreSQL delivery row is marked delivered only after PubAck. Timeout or transport errors are ambiguous and follow the existing transient retry/dead-letter policy with the same message ID; delivery is at least once. The Run/Execution owner needs a durable Inbox keyed by event_id because broker duplicate suppression has a finite window. This directly invoked API does not discover tenants, poll, configure the broker, provision production identities, consume events, or enable execution. No migration, W/T/M change, or Multipass VM is part of this slice.

Verification used Rust 1.99.0, PostgreSQL 18.6, and official NATS Server 2.15.0 bound to loopback. The phase report records the exact focused gates and unrelated baseline workspace-format limitation.""")
    changed += revision(basic, "| v5.93 |", "| v5.94 | 2026-10-07 | Mavis review | Add section 16.39 for injected-context, serial bounded JetStream publication; specify event allowlist, payload/envelope bounds, stable Nats-Msg-Id, PubAck-before-ack and at-least-once Run Inbox dedupe; synchronize requirements v5.97 and Schedule/Infrastructure design; record Rust 1.99.0 and loopback NATS/PostgreSQL evidence while preserving open production gates and the two-instance, zero-VM test limit. | Phase 9F4D-3D4 verified transport publication without changing owner persistence or execution boundaries. |", "v5.94")

    srs = ROOT / "docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md"
    changed += header(srs, "> Version:", "0.25", "0.26")
    changed += set_line(srs, "> Status:", "> Status: 🟡 Rule API, v2 multi-cluster contracts, isolated owner schema/grants, Inbox receive, profile-derived admission, the PostgreSQL Outbox delivery state machine, and the bounded JetStream publisher are verified; trusted resolver, cross-tenant allocator, target DB identity/provisioning, Run-owner consumer, production worker, provider evidence producer, and execution remain open and fail-closed.")
    changed += insert(srs, "## 3. Verification status", """| SCHED-API-025 | Infrastructure owner MAY publish claimed Outbox facts through an explicitly invoked Rust JetStream adapter using an injected authenticated context. Claims MUST be tenant-scoped, bounded to 1..8 and a 120..300 second lease, and published serially. Only allowlisted schema-v2 capacity facts with object payloads at most 16 KiB may be emitted under the fixed subject prefix; the serialized envelope MUST be at most 128 KiB and Nats-Msg-Id MUST equal immutable event_id. PostgreSQL delivery state MUST become delivered only after PubAck. Ambiguous publish failures MUST use bounded retry with the same message ID, and Run/Execution MUST deduplicate event_id in its durable Inbox beyond the broker duplicate window. This MUST NOT imply a worker, tenant scheduler, target DB identity, Run consumer, or execution readiness. |

""")
    changed += insert(srs, "## 4. Revision history", """9F4D-3D4 verification: Rust 1.99.0 package formatting, infrastructure all-target check, 53 library tests, targeted Clippy and dependency license/advisory gate passed. The disposable PostgreSQL 18.6 plus loopback NATS Server 2.15.0 runner passed 23/23 steps, 10/10 FORCE RLS checks and 11/11 ignored SQLx cases, including a real JetStream PubAck and delivered-state assertion. The runner created zero Multipass VMs. Production credentials/stream configuration, polling, target DB identity, Run-owner consumer, provider/Manager and executor remain unverified; durable Run Inbox dedupe is required beyond the broker duplicate window.

""")
    changed += revision(srs, "| v0.25 |", "| v0.26 | 2026-10-07 | Add SCHED-API-025 for injected-context bounded JetStream publication; specify serial claims, payload/envelope bounds, stable event ID, PubAck-before-delivered and durable Run Inbox dedupe. Record Rust 1.99.0 loopback NATS/PostgreSQL evidence and zero-VM test; keep production worker, Run consumer, provider and execution fail-closed. | The PostgreSQL Outbox delivery store gained a bounded transport adapter. |", "v0.26")

    bd = ROOT / "docs/design/BD-AUTOMATION-SCHEDULE-API-001.md"
    changed += header(bd, "> Version:", "0.24", "0.25")
    changed += set_line(bd, "> Status:", "> Status: 🟡 Rule API, typed v2 multi-cluster contracts, Inbox receive, profile-derived tenant-scoped admission, persisted Provider Profile source, Infrastructure Outbox delivery-state primitive, bounded JetStream publication and real PubAck integration are verified in focused/disposable gates; trusted resolver, production broker configuration/poller, production consumers and execution remain fail-closed.")
    changed += insert(bd, "## 7. Revision history", """### 6.10 9F4D-3D4 bounded JetStream publisher

The publisher accepts an established authenticated JetStream context and tenant-scoped Infrastructure delivery store. It leases 1..8 claims for one tenant with a 120..300 second lease and serially awaits PubAck. The allowlist is capacity_accepted, capacity_rejected, capacity_released, and capacity_expired schema v2; each JSON object is at most 16 KiB, the envelope at most 128 KiB, the subject is under star.infrastructure.v2, and Nats-Msg-Id is immutable event_id.

Only successful PubAck permits the PostgreSQL row to transition to delivered. Timeout and transport errors are ambiguous and use same-ID retry; delivery is at least once, so the future Run/Execution consumer must deduplicate with its durable Inbox. No broker setup, tenant poller, production DB identity, consumer, provider, or executor is included. The test runner uses an official NATS Server binary on loopback and creates no VM.""")
    changed += revision(bd, "| v0.24 |", "| v0.25 | 2026-10-07 | Add section 6.10 for serial bounded JetStream publication, injected authenticated context, allowlisted event contract, payload/envelope limits, stable Nats-Msg-Id, PubAck-before-delivered and durable Run Inbox dedupe. Record 9F4D-3D4 loopback NATS/PostgreSQL evidence and preserve production worker/consumer/execution gates. | The owner Outbox delivery state machine gained a bounded transport adapter. |", "v0.25")

    dd = ROOT / "docs/design/DD-AUTOMATION-SCHEDULE-API-001.md"
    changed += header(dd, "> Version:", "0.24", "0.25")
    changed += set_line(dd, "> Status:", "> Status: 🟡 Rule API, typed binding/demand/readiness, isolated owner schema/grants, Inbox receive, profile-derived tenant-scoped admission, persisted Provider Profile source, Outbox delivery state store, and bounded JetStream publisher are verified; trusted resolver, cross-tenant allocator, target migration/app identity, Run-owner Inbox consumer, production worker, provider evidence producer, and execution remain open.")
    changed += insert(dd, "## 7. Revision history", """## 6.11 9F4D-3D4 bounded JetStream publisher

Construct the publisher from an already authenticated async-nats JetStream context and PostgresInfrastructureOwnerOutboxDelivery. It does not own connection establishment, credentials, tenant discovery, stream provisioning, timers, or scheduling. publish_due_for_tenant validates non-nil tenant and lease owner IDs, batch size 1..8, and lease duration 120..300 seconds before its tenant-scoped database claim.

For each claim, validate schema version 2, the four capacity event types, object-shaped JSON, a maximum 16 KiB source payload, and a maximum 128 KiB serialized envelope. Publish serially, set Nats-Msg-Id to the immutable event UUID, and wait up to 10 seconds for PubAck. Only a successful PubAck calls mark_delivered. Timeout or failure calls the existing transient failure path; because the server may already have persisted the message, retry keeps the same message ID. The durable Run/Execution Inbox remains authoritative for dedupe beyond JetStream's duplicate window.

The focused integration launches NATS Server 2.15.0 on loopback with JetStream and the disposable PostgreSQL owner schema, publishes a real message, observes PubAck, verifies the database row is delivered, and verifies it is no longer claimable. The runner stops only the process it started and cleans its unique temporary store after stop. This is not production topology, failover, throughput, or end-to-end consumer evidence.

Rust 1.99.0 focused gates passed: package fmt, infrastructure all-target check, 53 library tests, and targeted Clippy. The extended disposable PostgreSQL/NATS runner passed 23/23 steps, 10/10 FORCE RLS and 11/11 SQLx cases. Workspace-wide formatting reports unrelated baseline files and was not mass-formatted. No Multipass VMs were created; current test inventory stays at two registered instances total.""")
    changed += revision(dd, "| v0.24 |", "| v0.25 | 2026-10-07 | Add section 6.11 for the 9F4D-3D4 publisher API, bounds, event allowlist, PubAck/timeout semantics and real loopback NATS/PostgreSQL integration. Clarify at-least-once delivery and durable Run Inbox dedupe; keep polling, target DB, Run consumer, provider, production broker configuration and execution open. | The bounded JetStream publisher passed its focused transport integration gate. |", "v0.25")

    infra = ROOT / "docs/design/DD-LOCAL-INFRASTRUCTURE-001.md"
    changed += header(infra, "> v0.23", "v0.23", "v0.24")
    changed += patch_line(infra, "> v0.24", "3D3 Outbox delivery state machine 已通过 focused/PG18.6 隔离验证。production resolver/provider、broker publisher/Run-owner Inbox consumer/scheduled reaper", "3D3 Outbox delivery state machine 与 3D4 bounded JetStream publisher 已通过 focused/PG18.6 隔离验证。production resolver/provider、Run-owner Inbox consumer/scheduled reaper")
    changed += replace(infra, "requirements v5.96 §50.8G-50.8N / basic-design v5.93 §8.1、§16.21-16.38 / Schedule SRS v0.25 / Schedule BD v0.24 / Schedule DD v0.24 / 实施计划 §6.83-6.98。", "requirements v5.97 §50.8G-50.8O / basic-design v5.94 §8.1、§16.21-16.39 / Schedule SRS v0.26 / Schedule BD v0.25 / Schedule DD v0.25 / Automation Design v2.23 / 实施计划 §6.83-6.99。")
    changed += insert(infra, "## §8 审阅与修订历史", """### §7.10 Phase 9F4D-3D4：Infrastructure Outbox 到 JetStream 的有界发布

Rust publisher 从调用方接收已认证的 JetStream Context 与 Infrastructure owner delivery store，不负责建立连接、保管生产凭据、创建 stream、发现 tenant 或运行 polling loop。每次只处理一个 tenant 的 1..8 条 claim，lease 为 120..300 秒，并逐条等待 PubAck。

发布契约只允许 schema v2 的 capacity_accepted、capacity_rejected、capacity_released、capacity_expired；主题限定在 star.infrastructure.v2，原始 JSON object 不超过 16 KiB，序列化 envelope 不超过 128 KiB，Nats-Msg-Id 固定为不可变 event_id。收到 PubAck 后才把 delivery Work row 标成 delivered。超时结果可能已被 broker 接收，因此走同 ID 重试；Run/Execution owner 仍必须通过持久化 Inbox 做长期去重。

验证在 loopback NATS Server 2.15.0 与临时 PostgreSQL 18.6 上进行，检查真实 PubAck、数据库 delivered 状态以及后续 claim 不再返回该事件。该切片没有 production broker provisioning、服务循环、目标数据库身份、Run consumer、provider/Manager 或 executor，也不创建 Multipass VM。测试环境的 Multipass 注册实例上限仍为两条记录，包含 Stopped/Deleted 且禁止自动 purge。Rust 工具链基线为 1.99.0。

""")
    changed += revision(infra, "| v0.23 |", "| v0.24 | 2026-10-07 | Mavis review | Add section 7.10 for bounded Infrastructure Outbox JetStream publication; pin stable event ID, PubAck-before-delivery, same-ID retry and durable Run Inbox dedupe; record loopback NATS/PostgreSQL evidence and Rust 1.99.0 with zero VM use. Preserve the two-record Multipass cap and open production worker/consumer/provider/execution gates. | Phase 9F4D-3D4 added verified transport publication without enabling execution. |", "v0.24")

    auto = ROOT / "docs/automation-design.md"
    changed += header(auto, "> **文档版本**:", "v2.22", "v2.23")
    changed += insert(auto, "## 11. 引用文档", """### 4.64 Phase 9F4D-3D4 Infrastructure Outbox JetStream publisher

**自动化档：[P]** — Rust 1.99.0 focused gates plus the disposable PostgreSQL owner-schema runner with an explicitly supplied NATS Server binary.

- The publisher is a directly invoked Rust adapter. It receives an authenticated async-nats JetStream context and the existing owner delivery store; it does not configure credentials/streams, enumerate tenants, or start a background worker.
- Each call claims one tenant's bounded 1..8 due events using a 120..300 second lease and publishes serially. Only schema-v2 capacity accepted/rejected/released/expired facts are accepted; body size is at most 16 KiB, envelope size at most 128 KiB, subject prefix is fixed, and Nats-Msg-Id equals immutable event_id.
- PostgreSQL delivery is acknowledged only after PubAck. Timeout/transport failure is ambiguous, reuses the same ID on retry, and remains at-least-once. The Run/Execution consumer must keep durable event_id Inbox dedupe beyond the broker duplicate window.
- The repeatable owner-schema runner requires --nats-server, starts only that supplied NATS binary on loopback with a unique temporary JetStream store, runs the real ignored SQLx publication case, stops the process it owns, and removes the store only after stop.
- Verification: Rust 1.99.0 package fmt, infrastructure all-target check, 53 library tests and targeted Clippy passed; disposable PostgreSQL 18.6/NATS Server 2.15.0 passed 23/23 steps, 10/10 FORCE RLS and 11/11 ignored SQLx tests. No Multipass VM was created; inventory cap remains two.
- This does not prove production broker credentials/streams, polling/tenant scheduling, target DB identity, Run-owner consumer, provider/Manager, failover, throughput, or execution readiness. Schedule and Task Card execution remain fail-closed.
- Phase evidence: PHASE-9F4D-3D4-JETSTREAM-PUBLISHER-REPORT.md.
""")
    changed += revision(auto, "| v2.22 |", "| v2.23 | 2026-10-07 | Add section 4.64 for bounded Rust JetStream publication and the repeatable loopback PostgreSQL/NATS runner; record Rust 1.99.0 focused gates, real PubAck/delivered-state integration, zero Multipass VMs, durable Run Inbox dedupe and open production consumer/worker/execution gates. | Phase 9F4D-3D4 completed the transport publisher slice. |", "v2.23")

    plan = ROOT / "docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md"
    changed += header(plan, "> **渡口 Project / Cloud Branch / Engineering Run 架构与实施计划", "v6.24", "v6.26")
    changed += insert(plan, "> - 本轮设计更新：", "> - Phase 9F4D-3D4 更新：新增有界 Rust JetStream publisher，按 tenant 串行 claim 1..8 条并等待 PubAck 后回写 delivered；固定四种 schema-v2 capacity event、16 KiB payload/128 KiB envelope、event_id 作为 Nats-Msg-Id。Rust 1.99.0 focused gates 与真实 loopback NATS/PostgreSQL 集成通过；无 worker、Run consumer、生产 broker identity、executor 或 Multipass VM。测试阶段最多保留两条注册 Multipass inventory 记录（含 Stopped/Deleted），不自动 purge。\n\n")
    changed += insert(plan, "## 修订履历", """### 6.99 Phase 9F4D-3D4 Infrastructure Outbox JetStream publisher（2026-10-07）

| 工作项 | 本阶段实现 | 验收与未关闭门 | 状态 |
|---|---|---|---|
| Bounded JetStream publication | Rust publisher receives an injected authenticated JetStream context and PostgreSQL delivery store; 1..8 tenant claim, 120..300 second lease, serial PubAck waits, four schema-v2 capacity events, 16 KiB payload/128 KiB envelope bounds and stable event_id as Nats-Msg-Id. | Delivered state is written only after PubAck. Ambiguous timeout uses same-ID retry; durable Run Inbox dedupe remains required past the broker duplicate window. | Focused slice |
| Real integration | NATS Server 2.15.0 JetStream on loopback plus PostgreSQL 18.6; runner asserts PubAck, delivered state and non-reclaim. | 23/23 runner steps, 10/10 FORCE RLS, 11/11 SQLx cases; Rust 1.99.0 package fmt/check/tests/Clippy passed. | Verified |
| Architecture boundary | No migration, worker, tenant scheduler, production broker credentials/stream, target DB role, Run consumer, provider/Manager, or executor. | Production agent execution remains fail-closed. No Multipass VM created; inventory cap is two registered records including Stopped/Deleted, no auto-purge. | Open |
| Documentation and tooling | Synchronized requirements v5.97, basic design v5.94, Schedule SRS v0.26/BD-DD v0.25, Infrastructure DD v0.24, Automation Design v2.23 and registry. Workspace minimum toolchain is Rust 1.99.0. | W/T/M unchanged; phase report records exact evidence and known gaps. | Synced |

Phase report: PHASE-9F4D-3D4-JETSTREAM-PUBLISHER-REPORT.md.

""")
    changed += revision(plan, "| v6.25 |", "| v6.26 | 2026-10-07 | Mavis review | Add section 6.99 for bounded JetStream publication; record real loopback NATS/PostgreSQL PubAck evidence, Rust 1.99.0 gates, documentation synchronization, zero Multipass VMs and the two-record cap. Keep production polling, target DB identity, Run Inbox consumer, provider/Manager and execution fail-closed. | The Infrastructure Outbox state machine gained its verified bounded transport adapter. |", "v6.26")

    registry = ROOT / "scripts/automation/registry.md"
    changed += header(registry, "> **文档版本**:", "v0.68", "v0.70")
    changed += patch(registry, "docs/automation-design.md" + chr(96) + " v2.16", "docs/automation-design.md" + chr(96) + " v2.23")
    text, nl = read(registry)
    lines = text.splitlines()
    prefix = "| " + chr(96) + "scripts/automation/phase9f4d_infrastructure_owner_schema.py" + chr(96) + " |"
    hits = [i for i, line in enumerate(lines) if line.startswith(prefix)]
    if len(hits) != 1:
        raise RuntimeError(f"expected one schema runner row, got {len(hits)}")
    if "real JetStream publication/PubAck" not in lines[hits[0]]:
        lines[hits[0]] = "| " + chr(96) + "scripts/automation/phase9f4d_infrastructure_owner_schema.py" + chr(96) + " | Phase 9F4D-3A/3B1/3C1/3C2/3C3/3D1/3D2/3D3/3D4 disposable PostgreSQL and JetStream runner: explicit --pg-bin and --nats-server; loopback-only PostgreSQL 18.6 and supplied NATS binary with unique stores; repeatable migrations/grants/profile/global-ledger assertions; restricted owner role, cumulative ten-table W/T/M checks, tenant/global RLS, WORM, lifecycle and retention. Runs every ignored SQLx owner case, including real JetStream publication/PubAck, delivered state and non-reclaim. It never calls Multipass or creates a VM. | Schedule Phase 9F4D-3A through 9F4D-3D4; automation-design §4.55/§4.57/§4.59-§4.64; implementation plan §6.90-§6.99; phase reports | 本次提交（见 Git） | 23/23 runner steps, 10/10 FORCE RLS and 11/11 ignored SQLx cases passed with PostgreSQL 18.6 and official NATS Server 2.15.0 on loopback; no VM. Production poller, Run consumer, target DB identity, provider/Manager and execution remain open; Multipass cap remains two. |"
        write(registry, "\n".join(lines) + "\n", nl)
        changed += 1
    path_cell = "| " + chr(96) + "scripts/automation/phase9f4d3d4_sync.py" + chr(96) + " |"
    row = path_cell + " Idempotent guarded synchronization for the 9F4D-3D4 requirement, design, automation, plan, report and registry entries; validates exact anchors before writing. | automation-design §4.64; implementation plan §6.99 | 本次提交（见 Git） | Re-runnable; second execution is a no-op. |\n\n"
    changed += insert(registry, "## 2. 子代理任务索引", row)
    changed += revision(registry, "| **v0.69** |", "| **v0.70** | **2026-10-07** | Mavis review | Register the 9F4D-3D4 documentation sync and extend the Infrastructure owner runner with explicit loopback NATS integration; synchronize design pointers, preserve Rust 1.99.0, record 23/23 runner steps, 10/10 FORCE RLS, 11/11 SQLx cases and zero VM use. | Bounded JetStream publisher and real PubAck integration passed. |", "v0.70")

    agents = ROOT / "AGENTS.md"
    changed += insert(agents, "**多代理并行与 Rust 桌面性能硬约束**", """**Rust toolchain baseline**：STAR workspace minimum supported Rust version is 1.99.0. Keep rust-toolchain.toml, Cargo.toml rust-version, CI toolchain pins, and dependency MSRV requirements aligned; run focused gates with 1.99.0 before raising the baseline.

""")

    report = ROOT / "docs/reports/PHASE-9F4D-3D4-JETSTREAM-PUBLISHER-REPORT.md"
    body = """# PHASE-9F4D-3D4-JETSTREAM-PUBLISHER-REPORT

## 0. Purpose

Implement and verify the bounded transport publisher connecting the Infrastructure-owner Outbox delivery state machine to NATS JetStream. This completes the 9F4D-3D4 transport slice only; it does not close the Infrastructure/Run owner boundary or enable agent execution.

## 1. Change and acceptance matrix

| Area | Change | Evidence and result |
|---|---|---|
| Rust toolchain | Set workspace minimum, rust-toolchain and pinned CI jobs to Rust 1.99.0; add async-nats 0.50.0 with JetStream and ring features. | Rust 1.99.0 focused infrastructure gates passed. |
| Publisher | Add tenant-scoped Rust adapter with injected authenticated JetStream context, serial PubAck waits, 1..8 claim bound and 120..300 second lease. | 53 infrastructure library tests and real JetStream integration passed. |
| Event contract | Allow four schema-v2 capacity events; cap source payload at 16 KiB and envelope at 128 KiB; use immutable event_id for Nats-Msg-Id. | Invalid event/payload tests pass; actual publication uses the same ID. |
| Delivery state | Mark delivered only after PubAck. Treat timeout/transport failure as ambiguous and retry with the same message ID through the bounded owner state machine. | Integration asserted PubAck, PostgreSQL delivered state and no subsequent claim. |
| Runner | Require explicit NATS binary; start it on loopback with unique temporary JetStream store; stop/clean only the process/store it owns. | 23/23 runner steps, 10/10 FORCE RLS and 11/11 ignored SQLx tests passed. |
| Product boundary | No production worker, tenant scheduler, broker credentials/stream provisioning, target DB identity, Run consumer, provider/Manager, or executor. | These remain open gates; Schedule and Task Card execution remain fail-closed. |
| Disk constraint | Create no Multipass VMs; preserve two-record inventory cap including stopped/deleted entries; no auto-purge. | Runner used PostgreSQL and one temporary local NATS process only. |

## 2. Verification summary

| Command/gate | Result |
|---|---|
| cargo fmt --package infrastructure -- --check | Passed |
| cargo check --locked -p infrastructure --all-targets -j 4 | Passed |
| cargo test --locked -p infrastructure --lib -j 4 | 53/53 passed |
| cargo clippy --locked -p infrastructure --all-targets --no-deps -j 4 -- -D warnings -A clippy::unwrap_or_default | Passed; allowance covers a pre-existing lint class outside this slice. |
| cargo-deny all-features check | Passed |
| Disposable PostgreSQL 18.6 + loopback NATS Server 2.15.0 runner | 23/23 steps, 10/10 FORCE RLS, 11/11 ignored SQLx tests passed, including real JetStream PubAck and PostgreSQL delivered-state checks. |
| cargo fmt --all -- --check | Not clean because unrelated baseline files already have formatting differences under Rust 1.99.0. They were not mass-formatted. |
| Multipass | No VM created; inventory remained untouched. |

The runner used the official NATS Server 2.15.0 Windows release archive. Archive SHA-256: 2D0861DC2CA3567D17F7D19611D7B346CC2CCBD7086A0A77CC957A2C71663200. Executable SHA-256: 4872E458BD2268ABFF4C8C2DAFADBB1D14AF255B3B935D53A1C88FF2D049CBC4. The binary remained outside the repository.

## 3. Known gaps

- No tenant polling/scheduling service or production broker configuration/secret provisioning exists.
- No target database login/membership migration or production deployment was verified.
- No Run/Execution owner consumer or durable Inbox implementation was added in this slice. That consumer must deduplicate immutable event_id beyond JetStream's finite duplicate window.
- No end-to-end provider/Manager, Multipass/K3s lifecycle, Agent sandbox, TaskExecutionRun executor, cancellation/recovery, or schedule-loop closure is claimed.
- The bounded batch controls in-flight serialized work; this phase does not establish a product-wide heap/RSS budget or benchmark throughput/failover.
- Workspace-wide formatting has unrelated baseline differences. Focused package formatting is clean.

## 4. Subagent failure and takeover

No subagents were dispatched for this phase. No unverified subagent output was integrated.

## 5. Guard rules

1. Rust 1.99.0 is the workspace minimum; Cargo, rust-toolchain, CI and dependency MSRV stay aligned.
2. Keep publication tenant-scoped, serial and bounded; never convert it into an implicit unbounded worker.
3. Treat delivery as at least once. PubAck gates delivered state; durable Run Inbox event_id dedupe remains mandatory.
4. Do not mark production broker, consumer, target DB, provider, or execution ready from this local integration.
5. The test phase uses at most two registered Multipass records total, counts stopped/deleted entries, creates zero VMs here, and never auto-purges.
6. Preserve owner separation: Infrastructure writes only its owner state; Run/Execution writes Run/Schedule outcomes in its own transaction.
7. Keep Schedule and Task Card execution fail-closed until target identity, consumer, binding/authorization checks, capacity reservation and independent Runtime readiness are verified.

## 6. Sign-off

| Role | Review |
|---|---|
| Architect | Mavis review — bounded adapter and owner boundary reviewed. |
| SRE Lead | Mavis review — local PostgreSQL/NATS evidence recorded; production topology remains open. |
| Platform | Mavis review — Rust 1.99.0 and runner lifecycle recorded; no VM use. |
| Review host | Mavis review — focused gates and unrelated workspace-format limitation reported. |
| PM | Mavis review — 9F4D-3D4 is a transport slice, not overall infrastructure completion. |

## 7. Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses — Mavis review | Record bounded JetStream publisher, real loopback PubAck/PostgreSQL evidence, Rust 1.99.0 baseline, open production gates and two-record Multipass limit. | Phase 9F4D-3D4 implementation and integration completed. |
"""
    if report.exists():
        if report.read_text(encoding="utf-8") != body:
            raise RuntimeError(f"report already exists with different content: {report}")
    else:
        report.write_text(body, encoding="utf-8", newline="\n")
        changed += 1

    print(f"9F4D-3D4 synchronization complete; changed={changed}")


if __name__ == "__main__":
    sync()
