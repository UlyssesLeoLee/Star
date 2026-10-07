"""Idempotently synchronize the INFRA-2C1 resource-catalog design chain."""

# @cypher schema=1 source_sha256=040fa6a3c70b5b3b6edfcaec1d7084b5ce91ec50e076bc8f42b88ddf468caded
# MERGE (self:File {path:"scripts/automation/phase_infra2c_resource_catalog_sync.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog_sync.py::main",kind:"function"})
# MERGE (transform:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog_sync.py::transform",kind:"function"})
# MERGE (apply:Symbol {id:"scripts/automation/phase_infra2c_resource_catalog_sync.py::apply",kind:"function"})
# MERGE (requirements:Document {id:"docs/requirements.md"})
# MERGE (basic:Document {id:"docs/basic-design.md"})
# MERGE (group:Document {id:"docs/design/DD-WORKTREE-GROUP-001.md"})
# MERGE (infra:Document {id:"docs/design/DD-LOCAL-INFRASTRUCTURE-001.md"})
# MERGE (wtm:Document {id:"docs/data-design/phase9f4d-infrastructure-owner-wtm.md"})
# MERGE (plan:Document {id:"docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md"})
# MERGE (automation:Document {id:"docs/automation-design.md"})
# MERGE (registry:Document {id:"scripts/automation/registry.md"})
# MERGE (report:Document {id:"docs/reports/PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT.md"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (main)-[:CALLS]->(transform)
# MERGE (main)-[:CALLS]->(apply)
# MERGE (main)-[:UPDATES]->(requirements)
# MERGE (main)-[:UPDATES]->(basic)
# MERGE (main)-[:UPDATES]->(group)
# MERGE (main)-[:UPDATES]->(infra)
# MERGE (main)-[:UPDATES]->(wtm)
# MERGE (main)-[:UPDATES]->(plan)
# MERGE (main)-[:UPDATES]->(automation)
# MERGE (main)-[:UPDATES]->(registry)
# MERGE (main)-[:UPDATES]->(report)
# @endcypher

from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]


def replace_once(text: str, old: str, new: str) -> str:
    if old not in text:
        if new in text:
            return text
        raise ValueError(f"missing exact anchor: {old[:100]!r}")
    if text.count(old) != 1:
        raise ValueError(f"expected one exact anchor, found {text.count(old)}: {old[:100]!r}")
    return text.replace(old, new, 1)


def replace_all(text: str, old: str, new: str) -> str:
    if old not in text:
        if new in text:
            return text
        raise ValueError(f"missing exact replacement text: {old[:100]!r}")
    return text.replace(old, new)


def insert_before(text: str, anchor: str, marker: str, block: str) -> str:
    if marker in text:
        return text
    if text.count(anchor) != 1:
        raise ValueError(f"expected one insertion anchor, found {text.count(anchor)}: {anchor!r}")
    return text.replace(anchor, f"{block.rstrip()}\n\n{anchor}", 1)


def insert_after_prefix(text: str, prefix: str, marker: str, block: str) -> str:
    if marker in text:
        return text
    lines = text.splitlines(keepends=True)
    indexes = [index for index, line in enumerate(lines) if line.startswith(prefix)]
    if len(indexes) != 1:
        raise ValueError(f"expected one line prefix, found {len(indexes)}: {prefix!r}")
    index = indexes[0]
    ending = "\r\n" if lines[index].endswith("\r\n") else "\n"
    lines.insert(index + 1, block.rstrip() + ending)
    return "".join(lines)


def transform(path: str, operations: list[tuple[str, str, str, str | None]]) -> bool:
    target = ROOT / path
    original = target.read_text(encoding="utf-8")
    updated = original
    for operation in operations:
        mode, anchor, marker, value = operation[:4]
        if mode == "replace":
            assert value is not None
            updated = replace_once(updated, anchor, value)
        elif mode == "replace-all":
            assert value is not None
            updated = replace_all(updated, anchor, value)
        elif mode == "before":
            assert value is not None
            updated = insert_before(updated, anchor, marker, value)
        elif mode == "after-prefix":
            assert value is not None
            updated = insert_after_prefix(updated, anchor, marker, value)
        else:
            raise ValueError(f"unsupported transform operation: {mode}")
    if updated != original:
        target.write_text(updated, encoding="utf-8")
        return True
    return False


REQUIREMENTS = """### 50.8T Phase INFRA-2C1 Host / Multipass / K3s resource identity catalog

The Infrastructure owner MUST persist distinct, generation-pinned identities for HostEnvironment, Multipass instance, K3s cluster, cluster-to-instance membership, and Kubernetes node. A K3s cluster MAY contain multiple registered guest instances/nodes, while each current Multipass guest MUST belong to at most one current cluster membership. Project, Branch, Engineering Run, Worktree, HostEnvironment, VM, cluster, and Kubernetes node identities MUST NOT be conflated. `RunInfrastructureBinding` continues to pin the exact cluster identity and generation; provider instance names and read-only CLI inventory are descriptive inputs, never ownership proof.

The five catalog tables are global Infrastructure-owner Master (M) rows with SCD2 generations. The append-only `resource_catalog_event` is Transaction (T) evidence for registration, active/draining transitions, and terminal retirement. Master insertion and its `registered/active` Transaction fact MUST commit atomically; a deferred constraint trigger MUST reject a Master generation committed without that registration fact. Only the Infrastructure owner can read/write the global catalog through a NOLOGIN role, explicit least-privilege grants, ENABLE + FORCE RLS, and owner-only policies; tenants and Plugins cannot read it directly. Master rows are immutable except for a one-time `valid_to` close after a retired Transaction fact. Event state transitions serialize by exact resource identity. Duplicate current guest membership and duplicate current node name within one cluster MUST be rejected.

This slice is persistence substrate only. It MUST NOT infer inventory ownership, capacity, daemon/guest/K3s/CRI/Runtime readiness, authorization, or execution eligibility; it MUST NOT call Multipass or start a VM. It does not implement inventory reconciliation, production owner commands/writer, provider resolver, target database provisioning, Manager/lifecycle, Run binding, BI/Benchmark, or Task Card CLI. The test environment remains capped at two registered Multipass records total (including Stopped/Deleted) by the controlled fail-closed launcher; this is not a customer topology limit, and no test auto-purges instances. Validation applies the current 16-table Infrastructure schema chain twice in disposable PostgreSQL, checks all 16 FORCE RLS tables and catalog owner grants/negative cases, and creates zero Multipass VMs.

| Acceptance ID | Acceptance criterion |
|---|---|
| AC-INFRA-002-T | Given the 9F4D Infrastructure owner schema and grants, migration MUST add exactly five global SCD2 Master tables and one append-only Transaction table, with no tenant identity or user-controlled host path/credential/raw provider output. It MUST enforce composite Host/cluster/instance/membership/node identity, at most one current cluster membership per Multipass instance, unique current node names per exact cluster generation, atomic Master plus `registered/active` Transaction commit via deferred constraint trigger, retired-event-before-SCD2-close, per-resource serialized event transitions, owner-only FORCE RLS, least-privilege table grants, and WORM Transaction facts. Migration reapplication MUST be safe. The disposable runner MUST apply the cumulative 16-table schema chain twice, reject orphan Master and invalid scope/state/privilege/mutation cases, pass all 16 FORCE RLS checks, stop and clean only its own PostgreSQL data, and create zero VMs. This acceptance does not claim production target DB deployment, authorized owner writer/reconciliation, capacity/readiness/provider/Manager/lifecycle, BI, or execution. |"""

BASIC_DESIGN = """### 16.44 Phase INFRA-2C1 Host / Multipass / K3s identity catalog

`infrastructure.host_environment_catalog`, `multipass_instance_catalog`, `k3s_cluster_catalog`, `k3s_cluster_instance_membership`, and `kubernetes_node_catalog` are global SCD2 Master tables. `resource_catalog_event` is append-only Transaction evidence. Each Master identity uses `(id, generation)`; references carry exact generations and composite foreign keys prevent a cluster, VM, Host, membership, or node from being cross-wired. A deferred constraint trigger requires each inserted Master generation and its `registered/active` Transaction fact to commit together; a disposable negative case confirms that it rejects an orphan Master at commit. One current guest can map to only one current cluster membership, and an exact cluster generation cannot have duplicate current Kubernetes node names.

The catalog has no `tenant_id`; all six tables use ENABLE + FORCE RLS with a policy for `star_infrastructure_owner`. Master grants are SELECT/INSERT/UPDATE (UPDATE only for the one-time valid-to close guarded by a retired event); Transaction grants are SELECT/INSERT with UPDATE/DELETE/TRUNCATE rejected independently by triggers. Deferred registration checks prevent a Master row from committing without its registration fact. Per-resource advisory locks serialize lifecycle event transitions. The 16-table W/T/M appendix distinguishes this catalog from the existing tenant-scoped admission/profile tables and global physical-capacity ledger.

This remains an owner persistence substrate: read-only `multipass list` observations and provider instance names do not establish registered ownership. No Rust writer/reconciler, deployment role mapping, target DB migration, Run binding resolver, live capacity/readiness provider, Manager, Multipass lifecycle operation, BI/Benchmark producer, or Task Card CLI is included. The PostgreSQL 18.6 runner verifies the five-migration chain twice, 16/16 FORCE RLS, least-privilege catalog grants, orphan-Master rejection, WORM, identity/state negative cases, and SCD2 replacement; it starts no VM. The live Multipass test cap remains two registered records including Stopped/Deleted, enforced by the separate locked launcher without automatic purge. `Rust 1.99.0` stays the workspace/toolchain baseline; this migration-only slice adds no Rust source.

| Acceptance ID | Design mapping and boundary |
|---|---|
| AC-INFRA-002-T | §16.44 and Infrastructure DD §7.14; verify cumulative schema reapplication, 16 FORCE RLS tables, exact catalog grants, owner-only visibility, composite identities, serialized lifecycle events, deferred-trigger orphan-Master rejection, SCD2 close and append-only events. No Multipass or VM operation; production owner/reconciliation and execution remain closed. |"""

INFRA_DD = """### §7.14 INFRA-2C1：Host / Multipass / K3s resource identity catalog

#### 7.14.1 Schema and ownership

Migration `2026-10-08-infrastructure-resource-master-catalog.sql` adds five global Infrastructure-owner Master tables: `host_environment_catalog`, `multipass_instance_catalog`, `k3s_cluster_catalog`, `k3s_cluster_instance_membership`, and `kubernetes_node_catalog`; it adds the append-only Transaction table `resource_catalog_event`. Each Master row is keyed by `(resource_id, generation)` with a one-current-generation index. Exact-generation composite foreign keys connect Multipass instances and clusters to their HostEnvironment, memberships to both exact cluster and instance on the same Host, and Kubernetes nodes to one exact membership. A deferred constraint trigger requires each inserted Master generation and its `registered/active` event to commit atomically. The constraints permit at most one current cluster membership per guest and one current node name per exact cluster generation.

The catalog deliberately has no tenant identity. All six tables use ENABLE + FORCE RLS and an owner-only policy for the restricted `star_infrastructure_owner` NOLOGIN role. Master privileges are SELECT/INSERT/UPDATE, with a trigger restricting UPDATE to a single `valid_to` closure after the corresponding generation has a committed `retired` event; DELETE/TRUNCATE are denied. The Transaction role can SELECT/INSERT only, and row/statement triggers reject UPDATE/DELETE/TRUNCATE. Event transitions use a transaction-scoped advisory lock derived from resource kind, identity and generation before reading the prior state, so concurrent lifecycle commands cannot both validate from a stale predecessor.

#### 7.14.2 Authority and temporal rules

Only a trusted Infrastructure owner command may register catalog identity. `multipass list` results, CLI names and provider facts remain observations; they cannot create or prove owner registration. Each inserted Master generation must commit atomically with its `registered/active` Transaction fact; a deferred constraint trigger fails the transaction if the fact is absent. Master changes create a new generation after retirement and one-time closure of the prior row. Events start with `registered/active`, allow only active↔draining state changes, and end terminally at `retired/retired`. A retired generation cannot receive another event. Catalog identity is separate from Host/cluster capacity ledger facts, `RunInfrastructureBinding`, guest/K3s/CRI/Runtime readiness, and execution authorization.

#### 7.14.3 Verification and open gates

`phase_infra2c_resource_catalog.py --pg-bin <PostgreSQL-18-bin>` applies the existing 3A schema, 3B1 grants, 3C3 profile, 3D1 physical ledger, and INFRA-2C1 catalog migrations twice each to a private loopback PostgreSQL 18.6 cluster. It verifies 16/16 FORCE RLS tables; catalog-only owner policies and grants; orphan-Master, unprivileged-read, wrong-host, duplicate-membership, duplicate-node-name, invalid-transition and WORM rejection; successful generation replacement; and expected audit facts. The database stops before the runner removes only its uniquely created temporary data directory. The gate creates zero Multipass instances.

This phase does not deploy to a target DB and does not add a Rust owner writer, inventory-to-identity reconciler, production owner login/membership, cross-tenant global allocator, current Run binding resolver, Manager/Multipass lifecycle, guest/K3s readiness, independent Agent Runtime sandbox, BI/Benchmark projection, or Schedule/Task Card executor. The test inventory remains capped at two registered Multipass records total including Stopped/Deleted; no auto-purge is permitted. Those conditions stay fail-closed until their own authorization, capacity, recovery, and end-to-end gates pass.

| Requirement | Detailed design and verification |
|---|---|
| AC-INFRA-002-T | §7.14.1–7.14.3; five M + one T catalog tables, exact generation references, deferred atomic registration, RLS/grants/WORM, per-resource event serialization, two-apply cumulative migration and no-VM PostgreSQL evidence. |"""

GROUP_DD = """### 8.17 Phase INFRA-2C1 Resource identity catalog

HostEnvironment、Multipass guest、K3s cluster、cluster membership 与 Kubernetes Node 是不同的 Infrastructure-owner 身份，持久化为全局 Master SCD2 rows 和 append-only Transaction events。每个新 Master generation 必须与 `registered/active` Transaction 在同一事务提交，延迟约束触发器拒绝缺少注册事实的孤儿 Master。一个 Host 可以容纳多个 Multipass instance 和多个独立 K3s cluster；每个当前 guest membership 固定到一个精确 cluster generation，RunInfrastructureBinding 仍固定 cluster identity/generation。目录行和 provider name 不能由 Run、插件、任务卡 CLI 或只读 `multipass list` 自行创建，也不能充当 ownership/readiness/capacity proof。

本 slice 仅提供经过 owner-only FORCE RLS 的 PostgreSQL schema/grant/state guards。它不改 Project → Branch → Engineering Run → local Worktree 导航，不将 Host/VM/cluster 加成左侧 Run App，也不创建跨 owner 写事务。只有之后的 Infrastructure owner command、授权 resolver、Run binding、Host+精确 cluster reservation、Manager 操作证据、guest/K3s/CRI readiness、独立 Agent Runtime sandbox 和 Worktree bridge 全部成立，Run Task Card CLI 才可启用。当前 BI/Benchmark 不消费这些 Master/Transaction rows；必须由版本化 owner event projection 与覆盖率审计后才能展示。验证在 disposable PostgreSQL 18.6 中两次应用累计 16 表迁移、16/16 FORCE RLS，不启动 Multipass；live 测试记录仍最多 2 条，含 Stopped/Deleted。

| Acceptance ID | Group boundary |
|---|---|
| AC-INFRA-002-T | Infrastructure DD §7.14；验证 catalog identity、generation、owner RLS/WORM、deferred-trigger orphan-Master rejection 与数据库边界。此阶段不创建 Run/Task/BI 事实、不启动 VM，Task Card CLI 保持关闭。 |"""

WTM = """### §1.3 Phase INFRA-2C1 global resource Master and Transaction

| Physical table | Class | Row purpose | Retention / deletion | Tenant isolation |
|---|---|---|---|---|
| `infrastructure.host_environment_catalog` | Master (M) | Stable HostEnvironment identity and provider/OS/architecture attributes with exact generation | SCD2; immutable business fields, one-time close after a retired event; physical DELETE prohibited | No `tenant_id`; ENABLE + FORCE RLS; owner-only `star_infrastructure_owner` policy |
| `infrastructure.multipass_instance_catalog` | Master (M) | Multipass instance UUID, exact Host generation and provider instance name | SCD2; instance name is not ownership proof; physical DELETE prohibited | No `tenant_id`; ENABLE + FORCE RLS; owner-only `star_infrastructure_owner` policy |
| `infrastructure.k3s_cluster_catalog` | Master (M) | K3s cluster UUID/generation, Host identity, provider kind, identity digest and pinned K3s version | SCD2; exact cluster generation is what Run binding must pin; physical DELETE prohibited | No `tenant_id`; ENABLE + FORCE RLS; owner-only `star_infrastructure_owner` policy |
| `infrastructure.k3s_cluster_instance_membership` | Master (M) | Exact cluster-to-Multipass guest membership and server/agent role | SCD2; each current guest maps to at most one current cluster; physical DELETE prohibited | No `tenant_id`; ENABLE + FORCE RLS; owner-only `star_infrastructure_owner` policy |
| `infrastructure.kubernetes_node_catalog` | Master (M) | Kubernetes Node UID/name and exact membership, cluster, instance and Host generations | SCD2; unique current node name per exact cluster generation; physical DELETE prohibited | No `tenant_id`; ENABLE + FORCE RLS; owner-only `star_infrastructure_owner` policy |
| `infrastructure.resource_catalog_event` | Transaction (T) | Immutable register, active/draining transition, and retirement facts keyed by resource kind/identity/generation | Append-only; UPDATE/DELETE/TRUNCATE rejected; no TTL purge | No `tenant_id`; ENABLE + FORCE RLS; owner-only `star_infrastructure_owner` policy |

**Current cumulative classification:** 16/16 tables across 9F4D-3A, 3C3, 3D1 and INFRA-2C1 have exactly one primary class: 4 Work (W), 6 Transaction (T), and 6 Master (M). Eight tenant-owned rows retain tenant FORCE RLS; eight global rows have no tenant column and are limited by owner-only RLS. The 3D2 expiry and 3D3 delivery state reuse existing Work/Transaction tables and add no classification. The INFRA-2C1 runner applied all five migrations in this cumulative schema chain twice and verified 16/16 FORCE RLS tables; its negative case confirmed that a deferred trigger rejects a Master generation without its registered event at commit. It did not create a Multipass VM.

**Owner and scope:** only trusted Infrastructure owner commands can populate or change the global catalog. Every new Master generation and its `registered/active` Transaction fact must commit in the same transaction; the deferred constraint trigger rejects orphan-Master commits. This appendix does not equate VM provider names with ownership, or catalog identity with capacity, Run binding, readiness, authorization, BI completeness, or execution. The live/test launcher continues to fail closed at two registered Multipass records including Stopped/Deleted, with no auto-purge.
"""

PLAN = """### 6.104 INFRA-2C1 Host / Multipass / K3s resource identity catalog（2026-10-08）

| 工作项 | 实施结果 | 当前门/状态 |
|---|---|---|
| Resource Master catalog | 新增五张 global SCD2 Master：HostEnvironment、Multipass instance、K3s cluster、cluster-instance membership、Kubernetes node；精确 generation FK 固定 Host/cluster/guest/node 归属。增加 append-only `resource_catalog_event` Transaction，并要求 Master 与 registered 事实同事务提交。 | Catalog 不含 tenant_id；仅受限 Infrastructure owner NOLOGIN role 可读写。CLI inventory/name 仍非所有权凭据。 |
| Integrity/security | Master 只允许退休事件后的单次 `valid_to` 闭合；事件状态按资源 advisory lock 串行化；每个 guest 至多一个 current cluster membership，每个精确 cluster generation 的 node name 唯一；六表 ENABLE + FORCE RLS。 | Owner Master SELECT/INSERT/guarded UPDATE，Transaction SELECT/INSERT；物理删除、WORM mutation、未授权读取均拒绝。 |
| Verification | PostgreSQL 18.6 disposable loopback runner 将 3A schema、3B1 grants、3C3 profile、3D1 physical ledger、2C1 catalog 五 migration 各应用两次；通过 29/29 runner steps、16/16 FORCE RLS、least-privilege catalog grants、deferred orphan-Master commit rejection plus invalid scope, identity, and WORM negative cases 与 SCD2 replacement。 | PostgreSQL 正常停止并仅删除 runner 自己的数据目录；0 Multipass VM，未接触用户 test inventory。 |
| Cross-layer sync | requirements v6.02 §50.8T / AC-INFRA-002-T；basic design v5.99 §16.44；Infrastructure DD v0.28 §7.14；Group DD v4.68 §8.17；W/T/M v0.8；本计划 v6.33。 | 此 slice 无 Rust source、无 live provider、无 Product UI、无容量/Run/readiness/BI/执行状态变更。 |
| Remaining INFRA-2 | Owner command/writer 与 inventory reconciliation、production login/membership/target grants、Run binding resolver、global allocator、Manager IPC/receipts、provider lifecycle/readiness、Agent Runtime sandbox/Worktree bridge、install/license UX、BI/Benchmark、RSS/并行与 Schedule/Task CLI e2e 均未关闭。 | TaskExecutionRun remains non-executable; Multipass 测试实例仍最多 2 条记录（含 Stopped/Deleted），禁止自动 purge。 |

阶段报告：[`PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT.md`](../reports/PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT.md)。
"""

AUTOMATION = """### 4.66 Phase INFRA-2C1 resource identity catalog

**自动化档：[P]** — `phase_infra2c_resource_catalog_sync.py` performs guarded, idempotent design synchronization; `phase_infra2c_resource_catalog.py` owns only a disposable loopback PostgreSQL cluster and its logs/data.

- Runner applies 9F4D-3A schema, 3B1 runtime grants, 3C3 profile, 3D1 global physical ledger and INFRA-2C1 resource catalog migrations twice each. PostgreSQL 18.6 evidence: 29/29 steps, 16/16 FORCE RLS, owner-only catalog policies, exact catalog grants, atomic Master registration, orphan-Master/identity/state rejection, WORM rejection and valid SCD2 generation replacement.
- Runner uses a unique randomly named private database directory, loopback-only listener and file-backed logs; it stops PostgreSQL before removing only its own directory. It creates zero Multipass VMs and does not read, mutate or purge live Multipass inventory.
- Migration adds five global Master tables plus one Transaction table. It proves only the catalog persistence boundary, not owner command/reconciliation, production target DB, manager, provider readiness, capacity admission, BI, or execution.
- Current live/test Multipass inventory limit remains two registered records total including Stopped/Deleted, enforced by the separate locked launcher. This test limit is not a customer topology limit and no automatic purge is permitted.
- Documentation sync script is exact-anchor guarded and idempotent; run it twice and confirm the second pass reports no file changes. Index both scripts and the phase report in `scripts/automation/registry.md`.
- Phase report: [`PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT.md`](reports/PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT.md).
"""

REGISTRY_RUNNER = "| `scripts/automation/phase_infra2c_resource_catalog.py` | PostgreSQL 18.6 loopback disposable runner for 3A schema + 3B1 grants + 3C3 profile + 3D1 physical ledger + INFRA-2C1 resource catalog; applies each twice, checks 16 FORCE RLS tables, deferred orphan-Master rejection, catalog grants/RLS/WORM/identity/SCD2 and owns only temporary DB/logs; creates zero VMs. | INFRA-2C1; automation-design §4.66; plan §6.104; phase report | This commit | 🟢 29/29 runner steps; PostgreSQL stopped and owned data removed; inventory unchanged. |"
REGISTRY_SYNC = "| `scripts/automation/phase_infra2c_resource_catalog_sync.py` | Exact-anchor guarded idempotent sync of INFRA-2C1 requirement/basic/detail/Group/W-T-M/plan/automation/registry versions and sections. | INFRA-2C1; automation-design §4.66; plan §6.104 | This commit | 🟢 Ran twice; second pass no-op. |"


def main() -> int:
    updates = {
        "docs/requirements.md": [
            ("replace", "# Vibe Coding Work Management SaaS 要件定义书（统合扩展版 v6.01）", "", "# Vibe Coding Work Management SaaS 要件定义书（统合扩展版 v6.02）"),
            ("replace", "Master rows cannot be physically deleted or rewritten, and a generation can close only after its retired Transaction fact.", "", "Master rows are immutable except for a one-time `valid_to` close after a retired Transaction fact."),
            ("replace", "unique current node names per exact cluster generation, retired-event-before-SCD2-close", "", "unique current node names per exact cluster generation, atomic Master plus `registered/active` Transaction commit via deferred constraint trigger, retired-event-before-SCD2-close"),
            ("replace", "28/28 isolated PostgreSQL steps and zero VM use.", "", "29/29 isolated PostgreSQL steps and zero VM use."),
            ("replace", "twice, reject invalid scope/state/privilege/mutation cases", "", "twice, reject orphan Master at commit and invalid scope/state/privilege/mutation cases"),
            ("before", "### 50.9 追溯与后续专题同步", "### 50.8T Phase INFRA-2C1", REQUIREMENTS),
            ("after-prefix", "| v6.01 | 2026-10-08 |", "| v6.02 |", "| v6.02 | 2026-10-08 | Mavis review | Add §50.8T / AC-INFRA-002-T for the Infrastructure-owner Host/Multipass/K3s identity catalog; define five SCD2 Master rows, append-only event state facts, exact-generation relationships, owner-only FORCE RLS and least privilege; record the 16-table migration chain, 29/29 isolated PostgreSQL steps and zero VM use. Preserve the two-record live test cap, no-auto-purge policy and fail-closed production execution boundary. | INFRA-2C1 adds only trusted resource identity persistence, not provider/Run execution. |"),
        ],
        "docs/basic-design.md": [
            ("replace", "> **文档版本**: v5.98 (2026-10-08)", "", "> **文档版本**: v5.99 (2026-10-08)"),
            ("replace", "> **上游要件定义书**: docs/requirements.md v6.01", "", "> **上游要件定义书**: docs/requirements.md v6.02"),
            ("replace", "cross-wired. One current guest", "", "cross-wired. A deferred constraint trigger requires each inserted Master generation and its `registered/active` Transaction fact to commit together; a disposable negative case confirms the deferred trigger rejects an orphan Master at commit. One current guest"),
            ("replace", "serialized lifecycle events, SCD2 close and append-only events.", "", "serialized lifecycle events, deferred-trigger orphan-Master rejection, SCD2 close and append-only events."),
            ("replace-all", "DD-LOCAL-INFRASTRUCTURE-001 v0.20", "", "DD-LOCAL-INFRASTRUCTURE-001 v0.28"),
            ("replace", "record 28/28 disposable PostgreSQL steps and 16/16 FORCE RLS.", "", "record 29/29 disposable PostgreSQL steps and 16/16 FORCE RLS."),
            ("before", "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |", "### 16.44 Phase INFRA-2C1", BASIC_DESIGN),
            ("after-prefix", "| v5.98 | 2026-10-08 | Mavis review |", "| v5.99 |", "| v5.99 | 2026-10-08 | Mavis review | Add §16.44 for INFRA-2C1's five global SCD2 Master identities and append-only Transaction catalog; define exact-generation relationships, per-resource event locks, owner-only FORCE RLS/grants and two-record VM test boundary. Align requirements v6.02, Group DD v4.68, Infrastructure DD v0.28, W/T/M v0.8 and plan v6.33; record 29/29 disposable PostgreSQL steps and 16/16 FORCE RLS. Keep owner writer, provider/readiness, BI and Task CLI gates open. | The infrastructure identity catalog schema passed its cumulative isolated database gate. |"),
        ],
        "docs/design/DD-LOCAL-INFRASTRUCTURE-001.md": [
            ("replace", "append-only Transaction catalog。累计 16 张表", "", "append-only Transaction catalog，并由延迟约束触发器强制 Master 与 registered Transaction 同事务提交。累计 16 张表"),
            ("replace", "one append-only Transaction event table, composite generation FKs", "", "one append-only Transaction event table, deferred atomic registration, composite generation FKs"),
            ("replace", "28/28 runner steps 通过", "", "29/29 runner steps 通过"),
            ("replace", "record 28/28 PG18.6 steps", "", "record 29/29 PG18.6 steps"),
            ("replace", "A guest instance can have only one current cluster membership; a cluster generation can have only one current node of a given name.", "", "The constraints permit at most one current cluster membership per guest and one current node name per exact cluster generation."),
            ("replace", "catalog-only owner policies and grants; unprivileged-read", "", "catalog-only owner policies and grants; orphan-Master commit rejection by the deferred trigger; unprivileged-read"),
            ("replace", "> 上游：requirements v6.01 §50.8G-50.8S / basic-design v5.98 §8.1、§16.21-16.43 / Schedule SRS v0.28 / Schedule BD v0.27 / Schedule DD v0.27 / Automation Design v2.24 / 实施计划 §6.83-6.103。", "", "> 上游：requirements v6.02 §50.8G-50.8T / basic-design v5.99 §8.1、§16.21-16.44 / Schedule SRS v0.28 / Schedule BD v0.27 / Schedule DD v0.27 / Automation Design v2.25 / W/T/M v0.8 / 实施计划 §6.83-6.104。"),
            ("before", "## §8 审阅与修订历史", "### §7.14 INFRA-2C1", INFRA_DD),
            ("after-prefix", "| v0.27 | 2026-10-08 | Mavis review |", "| v0.28 |", "| v0.28 | 2026-10-08 | Mavis review | Add §7.14 for the cumulative global Host/Multipass/K3s identity catalog: five SCD2 Master tables, one append-only Transaction event table, composite generation FKs, one-current-membership/node-name constraints, serialized events and owner-only FORCE RLS/grants. Align requirements v6.02, basic design v5.99, Group DD v4.68, W/T/M v0.8 and plan v6.33; record 29/29 PG18.6 steps, 16/16 FORCE RLS and zero VM use. Preserve owner writer/reconciliation, target deployment, Manager/provider/readiness/BI and execution gaps. | INFRA-2C1 added and verified only the resource identity persistence substrate. |"),
        ],
        "docs/design/DD-WORKTREE-GROUP-001.md": [
            ("replace", "> **渡口 Project / Branch / Engineering Run / Worktree 与 Run Apps 详细设计 v4.67**", "", "> **渡口 Project / Branch / Engineering Run / Worktree 与 Run Apps 详细设计 v4.68**"),
            ("replace", "> - 上位需求：[`docs/requirements.md`](../requirements.md) v6.01 §50", "", "> - 上位需求：[`docs/requirements.md`](../requirements.md) v6.02 §50"),
            ("replace", "> - 上位基本设计：[`docs/basic-design.md`](../basic-design.md) v5.98 §16", "", "> - 上位基本设计：[`docs/basic-design.md`](../basic-design.md) v5.99 §16"),
            ("replace", "[`DD-LOCAL-INFRASTRUCTURE-001.md`](DD-LOCAL-INFRASTRUCTURE-001.md) v0.27", "", "[`DD-LOCAL-INFRASTRUCTURE-001.md`](DD-LOCAL-INFRASTRUCTURE-001.md) v0.28"),
            ("replace", "实施计划 §6.103 v6.30", "", "实施计划 §6.104 v6.33"),
            ("replace", "append-only Transaction events。一个 Host", "", "append-only Transaction events；每个新 Master generation 必须与 `registered/active` Transaction 在同一事务提交。一个 Host"),
            ("replace", "Record 28/28 PostgreSQL steps", "", "Record 29/29 PostgreSQL steps"),
            ("replace", "AC-INFRA-002-T | Infrastructure DD §7.14；验证 catalog identity、generation、owner RLS/WORM 与数据库边界。", "", "AC-INFRA-002-T | Infrastructure DD §7.14；验证 catalog identity、generation、owner RLS/WORM、deferred-trigger orphan-Master rejection 与数据库边界。"),
            ("before", "## §9 跨 App 事件与一致性", "### 8.17 Phase INFRA-2C1", GROUP_DD),
            ("after-prefix", "| AC-INFRA-001 | §8.8 / DD-LOCAL-INFRASTRUCTURE-001 |", "| AC-INFRA-002-T |", "| AC-INFRA-002-T | §8.17 / DD-LOCAL-INFRASTRUCTURE-001 §7.14 | Host/guest/cluster/node 精确身份 catalog 与 owner-only schema；不把目录存在误判为 capacity/readiness/Run binding/CLI authority |"),
            ("after-prefix", "| v4.67 | 2026-10-08 | Mavis review |", "| v4.68 |", "| v4.68 | 2026-10-08 | Mavis review | Add §8.17 for INFRA-2C1 global resource identity catalog and cross-link requirements v6.02/basic design v5.99/Infrastructure DD v0.28/W-T-M v0.8/plan v6.33; preserve Project→Branch→Run→Worktree navigation and exact Run cluster binding; keep owner commands, Manager/readiness, BI and Task Card CLI gated. Record 29/29 PostgreSQL steps, 16/16 FORCE RLS and zero VM use; retain two registered Multipass records for live tests and no auto-purge. | Infrastructure identity persistence is verified without claiming a usable VM provider or execution loop. |"),
        ],
        "docs/data-design/phase9f4d-infrastructure-owner-wtm.md": [
            ("replace", "> Version: v0.7 · 2026-10-07", "", "> Version: v0.8 · 2026-10-08"),
            ("replace", "> Scope: additive schemas in `db/migrations/2026-10-07-infrastructure-capacity-owner-schema.sql`, `db/migrations/2026-10-09-infrastructure-provider-profile-master.sql`, and `db/migrations/2026-10-10-infrastructure-physical-capacity-ledger.sql`", "", "> Scope: cumulative Infrastructure-owner schemas in 3A, 3C3, 3D1, and `db/migrations/2026-10-08-infrastructure-resource-master-catalog.sql`"),
            ("replace", "> Boundary: preserve the original six-table 3A classification, add the two 3C3 profile tables and two 3D1 global physical ledger tables; 3D2 expiry and 3D3 Outbox delivery reuse existing Work/Transaction tables and triggers, add no schema objects, and do not rewrite the historical 100-table index or reclassify Run/Execution-owned data.", "", "> Boundary: preserve phase-scoped 3A/3C3/3D1 snapshots and add the INFRA-2C1 global resource identity catalog; 3D2 expiry and 3D3 delivery reuse existing tables. Do not rewrite the historical 100-table index or reclassify Run/Execution-owned data."),
            ("replace", "Classify every physical table introduced by the 9F4D-3A migration. The Infrastructure owner receives versioned capacity demand from the Run/Execution owner, admits both Host and exact-cluster capacity in its own transaction, and publishes immutable owner facts. No cross-owner write or distributed database transaction is permitted. The schema is not a production writer, consumer, provider, or executor.", "", "Classify every physical table in the cumulative Infrastructure-owner persistence chain. Run/Execution and Infrastructure retain independent owners and transactions; cross-owner facts flow through versioned Outbox/Inbox events, not direct writes or a distributed transaction. The schemas are persistence substrates, not proof of production deployment or execution."),
            ("replace", "**Current cumulative coverage:** 10/10 tables across 3A, 3C3 and 3D1 have one primary class: 4 W, 5 T, 1 M.", "", "**3D1 cumulative snapshot:** 10/10 tables across 3A, 3C3 and 3D1 had one primary class: 4 W, 5 T, 1 M."),
            ("replace", "- The eight tenant-owned tables across 3A and 3C3 use tenant FORCE RLS; the two 3D1 global physical tables use a role-only owner policy and deliberately have no tenant column. All ten tables use FORCE RLS. FORCE does not substitute for a non-superuser, `NOBYPASSRLS` production role and explicit target-environment grants; neither production role provisioning nor target DB deployment is included here.", "", "- At the 3D1 snapshot, eight tenant-owned tables across 3A/3C3 and two global physical tables had FORCE RLS. INFRA-2C1 adds six global owner-only catalog tables; the current 16-table classification and isolation boundaries are listed in §1.3. FORCE RLS does not substitute for a non-superuser, `NOBYPASSRLS` production role and target-environment grants; production identity provisioning and target DB deployment remain open."),
            ("replace", "five global SCD2 Master tables plus one append-only Transaction table; classify 16 total tables", "", "five global SCD2 Master tables plus one append-only Transaction table; require deferred atomic registration; classify 16 total tables"),
            ("replace", "It passed 28/28 runner steps and 16/16 FORCE RLS.", "", "It passed 29/29 runner steps and 16/16 FORCE RLS."),
            ("replace", "and verified 16/16 FORCE RLS tables; it did not create", "", "and verified 16/16 FORCE RLS tables, including deferred orphan-Master rejection at transaction completion; it did not create"),
            ("replace", "**Owner and scope:** only trusted Infrastructure owner commands can populate or change the global catalog. This appendix does not equate VM provider names with ownership, or catalog identity with capacity, Run binding, readiness, authorization, BI completeness, or execution.", "", "**Owner and scope:** only trusted Infrastructure owner commands can populate or change the global catalog. Every new Master generation and its `registered/active` Transaction fact must commit in the same transaction; the deferred constraint trigger rejects orphan-Master commits. This appendix does not equate VM provider names with ownership, or catalog identity with capacity, Run binding, readiness, authorization, BI completeness, or execution."),
            ("replace", "record 28/28 PG18.6 steps, 16/16 FORCE RLS", "", "record 29/29 PG18.6 steps, 16/16 FORCE RLS"),
            ("before", "## §2 Constraints and owner boundary", "### §1.3 Phase INFRA-2C1", WTM),
            ("before", "## §4 Revision history", "**INFRA-2C1 current evidence:**", "**INFRA-2C1 current evidence:** `phase_infra2c_resource_catalog.py` applied the 3A schema, 3B1 grants, 3C3 Provider Profile, 3D1 physical-capacity ledger and INFRA-2C1 catalog migrations twice each in disposable PostgreSQL 18.6. It passed 29/29 runner steps and 16/16 FORCE RLS. The catalog Master tables are global owner-only; existing capacity inbox/profile tables retain tenant or global policies from their own migrations. The runner does not deploy to a target database, create a Multipass VM, or verify production identity/owner commands. The test inventory cap remains two registered Multipass records including Stopped/Deleted, with no automatic purge.\n\n"),
            ("after-prefix", "| v0.7 | 2026-10-07 | Mavis review |", "| v0.8 |", "| v0.8 | 2026-10-08 | Mavis review | Add §1.3 and current cumulative coverage for the six-table INFRA-2C1 resource catalog: five global SCD2 Master tables plus one append-only Transaction table; classify 16 total tables as 4 W / 6 T / 6 M, preserve earlier phase snapshots, and record 29/29 PG18.6 steps, 16/16 FORCE RLS, two-instance test cap and zero VM use. | INFRA-2C1 resource identity schema and cumulative runner passed. |"),
        ],
        "docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md": [
            ("replace", "> **渡口 Project / Cloud Branch / Engineering Run 架构与实施计划 v6.32**", "", "> **渡口 Project / Cloud Branch / Engineering Run 架构与实施计划 v6.33**"),
            ("replace", "新增五张 global SCD2 Master：HostEnvironment、Multipass instance、K3s cluster、cluster-instance membership、Kubernetes node；精确 generation FK 固定 Host/cluster/guest/node 归属。增加 append-only `resource_catalog_event` Transaction。", "", "新增五张 global SCD2 Master：HostEnvironment、Multipass instance、K3s cluster、cluster-instance membership、Kubernetes node；精确 generation FK 固定 Host/cluster/guest/node 归属。增加 append-only `resource_catalog_event` Transaction，并要求 Master 与 registered 事实同事务提交。"),
            ("replace", "通过 28/28 runner steps", "", "通过 29/29 runner steps"),
            ("replace", "orphan-Master/orphan-Master/orphan-Master/invalid-state/identity/WORM negative cases 与 SCD2 replacement", "", "deferred orphan-Master commit rejection plus invalid scope, identity, and WORM negative cases 与 SCD2 replacement"),
            ("replace", "Record 28/28 disposable PostgreSQL steps", "", "Record 29/29 disposable PostgreSQL steps"),
            ("before", "## 修订履历", "### 6.104 INFRA-2C1", PLAN),
            ("after-prefix", "| v6.32 | 2026-10-08 | Mavis review |", "| v6.33 |", "| v6.33 | 2026-10-08 | Mavis review | Add §6.104 for INFRA-2C1's Host/Multipass/K3s identity catalog; sync requirements v6.02, basic design v5.99, Infrastructure DD v0.28, Group DD v4.68, W/T/M v0.8, automation-design v2.25 and registry v0.72. Record 29/29 disposable PostgreSQL steps, 16/16 FORCE RLS and zero VM use; preserve the two-record registered-instance cap, no-auto-purge rule and all production owner/provider/readiness/BI/execution gates. | The cumulative resource identity persistence slice passed its isolated database gate. |"),
        ],
        "docs/automation-design.md": [
            ("replace", "> **文档版本**: v2.24 (2026-10-07)", "", "> **文档版本**: v2.25 (2026-10-08)"),
            ("replace", "PostgreSQL 18.6 evidence: 28/28 steps", "", "PostgreSQL 18.6 evidence: 29/29 steps"),
            ("replace", "evidence: 28/28 steps", "", "evidence: 29/29 steps"),
            ("replace", "record 28/28 runner steps", "", "record 29/29 runner steps"),
            ("replace", "exact catalog grants, identity/state rejection", "", "exact catalog grants, atomic Master registration, orphan-Master/identity/state rejection"),
            ("before", "## 5. 守门基线", "### 4.66 Phase INFRA-2C1", AUTOMATION),
            ("after-prefix", "| v2.24 | 2026-10-07 |", "| v2.25 |", "| v2.25 | 2026-10-08 | Add §4.66 and register guarded resource-catalog documentation sync plus disposable PostgreSQL runner; apply the 3A/3B1/3C3/3D1/INFRA-2C1 chain twice, record 29/29 runner steps and 16/16 FORCE RLS, and keep live Multipass inventory unchanged at the two-record cap. | INFRA-2C1 adds owner-scoped Host/cluster identity persistence without a VM or production executor. |"),
        ],
        "scripts/automation/registry.md": [
            ("replace", "> **文档版本**: v0.70 (2026-10-07)", "", "> **文档版本**: v0.72 (2026-10-08)"),
            ("replace", "🟢 28/28 runner steps; PostgreSQL stopped and owned data removed; inventory unchanged.", "", "🟢 29/29 runner steps including orphan-Master rejection; PostgreSQL stopped and owned data removed; inventory unchanged."),
            ("replace", "checks 16 FORCE RLS tables, catalog grants/RLS/WORM/identity/SCD2", "", "checks 16 FORCE RLS tables, deferred orphan-Master rejection, catalog grants/RLS/WORM/identity/SCD2"),
            ("replace", "five SCD2 Master identities plus one append-only Transaction table, 28/28 runner steps, 16/16 FORCE RLS", "", "five SCD2 Master identities plus one append-only Transaction table, 29/29 runner steps including orphan-Master rejection, 16/16 FORCE RLS"),
            ("after-prefix", "| `scripts/automation/phase9f4d3d5_sync.py`", "scripts/automation/phase_infra2c_resource_catalog.py", REGISTRY_RUNNER + "\n" + REGISTRY_SYNC),
            ("after-prefix", "| **v0.71** | **2026-10-07** |", "| **v0.72** |", "| **v0.72** | **2026-10-08** | **Mavis review** | Register the guarded INFRA-2C1 requirement/design sync and cumulative PostgreSQL runner; document five SCD2 Master identities plus one append-only Transaction table, 29/29 runner steps, 16/16 FORCE RLS, no VM use, and the unchanged two-record live Multipass cap. | The resource identity catalog schema passed its isolated PostgreSQL gate. |"),
        ],
        "docs/reports/PHASE-INFRA-2C1-RESOURCE-CATALOG-REPORT.md": [
            ("replace", "loopback PostgreSQL 生命周期、28/28 runner 与清理证据", "", "loopback PostgreSQL 生命周期、29/29 runner 与清理证据"),
            ("replace", "16 表累计 W/T/M 与 RLS、28/28 PostgreSQL evidence", "", "16 表累计 W/T/M 与 RLS、29/29 PostgreSQL evidence"),
        ],
    }

    changed: list[str] = []
    try:
        for path, operations in updates.items():
            if transform(path, operations):
                changed.append(path)
    except (OSError, ValueError) as error:
        print(f"sync failed: {error}", file=sys.stderr)
        return 1
    print(f"changed={len(changed)} files")
    for path in changed:
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
