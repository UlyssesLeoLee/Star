# PHASE-9F4D-MULTICLUSTER-CAPACITY-REPORT — V2 multi-cluster contract closeout

> Status: 🟡 Rust V2 value contracts verified; Infrastructure owner persistence, provider execution, and Schedule production execution remain open.
> Date: 2026-10-07
> Revision: v0.5
> Reviewer: Ulysses（一人公司 12 角色 per DEC-008）— Mavis review

## §0 Purpose

Record the Phase 9F4D multi-cluster contract correction for the user's Multipass-managed Linux guests and K3s architecture. The current test phase permits at most two registered Multipass instances in total, including stopped/recoverable entries, because retained guest disks still consume host storage. The bootstrap uses a host-local exclusive lock across inventory check and launch. This report does not create, delete, recover, or change a VM.

## §1 Change matrix

| Area | Delivered | State |
|---|---|---|
| Run binding placement | Add typed `ClusterPlacementPolicyV1` values for shared pool, Run-dedicated, and pinned-cluster placement; include the policy wire value in the binding digest. | 🟢 Implemented and covered by a digest test. |
| Capacity ownership | Keep HostEnvironment and exact K3s cluster capacity as separate ledgers and reservations. | 🟢 Pure Rust candidate evaluation remains advisory; owner transaction/persistence is open. |
| Reservation evidence | Give acceptance and readiness evidence distinct `host_capacity_reservation_id` and `cluster_capacity_reservation_id` fields; reject nil or mismatched IDs. | 🟢 Implemented and covered by readiness tests. |
| Test resource cap | Limit the full registered Multipass inventory in this phase to two entries; stopped and recoverable/deleted records count. Contract tests do not launch VMs. The bootstrap script serializes inventory check plus launch, rejects a third instance or an unparseable inventory, and never auto-purges. | 🟢 Documented and guarded as a temporary test constraint, not a product topology limit. |
| Design synchronization | Update requirements, basic design, Group DD, Schedule SRS/BD/DD, Infrastructure DD, implementation plan, automation design, registry, and this report. | 🟢 Current versions and evidence links synchronized. |

## §2 Verification summary

Focused runner: `scripts/automation/phase9f4d_infrastructure_contract.py`, using Rust 1.98.1 and target directory `C:\t\star9f4d-placement-reservations-target-20261007`.

| Gate | Result |
|---|---|
| rustfmt check for `lib.rs`, `readiness.rs`, and `capacity.rs` | 🟢 PASS |
| `cargo check --locked -p domain-infrastructure --all-targets -j 4` | 🟢 PASS |
| Clippy with `-D warnings` | 🟢 PASS |
| `cargo test --locked -p domain-infrastructure --all-targets -j 4` | 🟢 PASS — 27/27 tests |
| Multipass bootstrap launch guard | 🟢 PASS — offline PowerShell smoke covered both supported JSON shapes, malformed inventory fail-closed, refusal at two registered instances, allow at one existing instance, Deleted-target refusal, and cross-process lock contention (a second PowerShell process cannot acquire the file lock until release); all Multipass calls were mocked. |

The final isolated-target runner output is in `C:\t\star9f4d-placement-reservations-rerun-20261007`. An earlier isolated test link attempt returned MSVC `LNK1104`; rerunning the complete four-gate runner with a separate target directory passed. One earlier check attempt also caught and fixed a missing test import and rustfmt difference. No Multipass instance was started or modified.

The Rust and PowerShell source CypherGraph Guardian check passed for all five changed source files. `git diff --check` passed. The PowerShell guard was verified only against mock inventory output and was not exercised against the user's Multipass daemon. Its lock coordinates concurrent runs of this bootstrap script; direct Multipass CLI or unrelated launchers can bypass it, so all live test instance creation in this phase must use the guarded script.

## §3 Known gaps

- The authoritative Infrastructure Profile/placement resolver and current ACL/Run-binding recheck are not implemented by this contract crate.
- Host and cluster reservation ledgers, transactional owner stores, Inbox/Outbox consumers, release/reconciliation workers, and the Schedule admission writer remain open.
- The Rust Host Infrastructure Manager, Multipass adapter, guest/K3s readiness producer, and production provider lifecycle remain open.
- Runtime sandbox enforcement, target database/RLS grants, BI/Benchmark event production, and desktop RSS/fairness measurements remain open.
- No production consumer currently makes the V2 wire contract an executable Schedule feature; Run execution remains fail closed.
- Two registered Multipass instances total, including stopped/recoverable entries, is only the temporary cap for this test phase; it does not constrain customer topology.
- The script guard does not intercept arbitrary external Multipass CLI use; live tests must launch through the guarded bootstrap until the Star-native provider enforces the same cap.

## §4 Subagent failures and handoff

No subagents were dispatched for this bounded closeout. There are no uncollected child-agent outputs.

## §5 Guard rules

1. Do not treat a capacity candidate as a persisted reservation or an authorization decision.
2. Host and exact-cluster reservation IDs and leases must be independently present and matched.
3. Unknown, stale, mismatched, expired, or insufficient capacity/readiness fails closed.
4. Keep the full registered Multipass test inventory at no more than two instances for this phase, including stopped/recoverable entries; route every live test launch through the guarded bootstrap; Rust contract tests stay VM-free.
5. Never expose Multipass daemon/socket, arbitrary host shell, or unrestricted guest exec to Task CLI, Agent, or Plugin.
6. Do not mark Schedule execution, VM provisioning, sandbox, BI, or production provider lifecycle as complete based on this Rust value-contract slice.

## §6 Review roles

| Role | Review state | Scope |
|---|---|---|
| Architecture | 🟢 Mavis review | Multi-cluster identity, placement and owner boundaries. |
| SRE Lead | 🟡 Open for provider rollout | Capacity operations, reconciliation, VM/K3s lifecycle and observability. |
| Platform | 🟡 Open for production integration | Rust Manager/provider, DB/RLS, Runtime sandbox and target environment. |
| Review host | 🟢 Mavis review | V2 binding digest, dual reservation identity, tests and fail-closed wording. |
| Product | 🟢 User constraint recorded | Temporary two-instance maximum during the current test phase. |

## §7 Revision history

| Version | Date | Revision | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-07 | Record the multi-cluster V2 contract, verification evidence, temporary two-instance test cap, and remaining production gates. | User limited the test phase to two Multipass instances; closeout review found missing typed placement digest and dual reservation identity details. |
| v0.2 | 2026-10-07 | Clarify the disk cap as two total registered instances, guard the bootstrap path against a third or unverifiable inventory, and record the offline mocked PowerShell checks and final review gates. | User clarified that stopped instances also consume disk. |
| v0.3 | 2026-10-07 | Serialize the bootstrap inventory check and launch with a host-local exclusive lock; verify contention behavior; disclose that direct CLI use can bypass the script guard and route live tests through the controlled path. | Review found concurrent bootstrap invocations could race the count check. |
| v0.4 | 2026-10-07 | Execute and record the offline mocked PowerShell launch-guard smoke, including independent-process lock contention; align active requirements, basic-design, Group DD, DUKOU direction, and implementation-plan pointers. | Final evidence and version-reference audit. |
| v0.5 | 2026-10-07 | Record the correction of active Group DD/basic-design infrastructure paragraphs and final current-document references; preserve historical snapshots and the open production gates. | Cross-layer review found stale active shared-control-plane wording. |
