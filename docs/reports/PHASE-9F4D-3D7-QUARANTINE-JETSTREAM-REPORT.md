# PHASE-9F4D-3D7 — JetStream Quarantine Redelivery Verification Report

> Version: v0.1 · Date: 2026-10-08 · Status: 🟢 isolated redelivery gate verified; production consumer and execution remain disabled
> Revision owner: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 Purpose

Close the bounded Phase 9F4D-3D6 verification gap by exercising the existing Platform quarantine receipt writer and Run-owner opt-in consumer hook against real disposable PostgreSQL 18.6 and loopback NATS JetStream 2.15.0. This slice adds only a focused integration test and its serialized runner invocation. It adds no schema, migration, Multipass operation, production consumer, Run navigation node, or execution capability.

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Run-owner Rust integration | Added receipt-write failure → actual redelivery → metadata-only receipt commit → confirmed Term ACK → no later delivery assertions. Added committed receipt + dropped unacknowledged Message/fetch handles → exact duplicate replay → confirmed Term ACK → no later delivery assertions. | The handle drop models consumer loss; the test does not kill/restart an OS process or enable production composition. |
| Test runner | Applies the existing quarantine migration/schema assertions and runs the shared-subject JetStream target with `--test-threads=1`. Logs are placed in a unique `target/` output directory. | Prevents integration tests racing over the same JetStream subject. The runner starts/stops only its disposable services and removes only its owned PostgreSQL data. |
| Cross-layer documents | Updated Requirements, Basic Design, Schedule SRS/BD/DD, Group DD, Infrastructure DD, Automation Design, implementation plan, and automation registry. Added this report and an idempotent documentation-sync script. | Preserves Project → Cloud Branch → Engineering Run → local Worktree navigation and Run App ownership. Data Design W/T/M is unchanged because no schema or persisted fact was added. |

## §2 Verification summary

| Gate | Result |
|---|---|
| Rust 1.99.0 focused integration target compile (`cargo test --no-run`) | Passed. |
| Rust formatting (`cargo fmt --package run-owner-runtime -- --check`) | Passed. |
| Run-owner library unit tests | 10/10 passed. |
| Targeted strict Clippy with the three recorded baseline lint categories allowed | Passed for `run-owner-runtime --test capacity_result_jetstream_pg`; unfiltered strict Clippy remains blocked as noted below. |
| PostgreSQL 18.6 + NATS JetStream 2.15.0 disposable runner | 22/22 steps passed. Both the pre-existing capacity-result test and new quarantine test passed: 2/2 integration tests. |
| Quarantine integration | Verified no receipt and one ACK-pending delivery while the injected INSERT fault exists; actual redelivery stores exactly one metadata-only receipt, no payload Work row, awaits confirmed Term ACK, and is not delivered again. The committed-receipt/pre-ACK handle-drop replay returns the exact duplicate outcome and is likewise Term-acked without another delivery. |
| Disposable-resource boundary | PostgreSQL and NATS stopped successfully; the runner removed its owned PostgreSQL data directory. Audit reports zero Multipass instances created and no inventory modification. |
| Documentation sync | `phase9f4d3d7_sync.py` ran twice; the second run reported `changed_operations=0`. Data Design W/T/M was not changed. |
| Python syntax / registry | The sync and runner scripts passed `py_compile`; `registry_check.py` reported 0 errors and 197 repository-wide legacy warnings. Both changed scripts were registered with no warnings. Its log is under the unique `target/` evidence directory. |
| CypherGraph Guardian | `cgg_guard.py --changed --stamp` passed for all 3 changed source files. |
| Diff whitespace | `git diff --check` passed. |
| Strict Clippy | The strict gate is blocked by existing dependency warnings outside this slice: `star-context` (`unnecessary_map_or`, `field_reassign_with_default`) and `domain-kms` (three deprecated `Nonce::from_slice` calls). No changes were made to those unrelated crates. |

The runner audit is retained at `target/phase9f4d3d7-quarantine/root-final-20261008-2/audit.jsonl` with step logs beside it. The earlier parallel-test attempt exposed the shared-subject race; after adding serial test execution, the fresh run passed. The failed attempt used a separate evidence directory and did not replace the passing run.

## §3 Known gaps

- Production Platform DB/NATS identities, trusted deployment/stream provisioning, KMS-backed encryption, key lifecycle, and target database rollout are not verified.
- The duplicate replay case drops the unacknowledged JetStream `Message` and fetch handles to model consumer loss; it does not terminate or restart an operating-system process.
- Operator quarantine inspection/replay UI, production consumer composition, operational alerting, and deployment/rollback procedures remain open.
- Schedule worker admission, Infrastructure provider lifecycle, Host/cluster reservations, K3s readiness, Agent sandbox, Task Card CLI execution, Run BI, and Benchmark remain separate gates. Schedule and Task execution stay fail-closed.
- No Multipass provider operation was performed; the test-only two-record inventory cap and no-auto-purge rule remain unchanged.
- Strict Clippy remains blocked by the three pre-existing dependency lint categories recorded in §2; this phase did not modify those dependencies.

## §4 Subagent failure and takeover

No subagent was dispatched for this phase. There is no child worktree or subagent output to accept or take over. The integration test, runner adjustment, cross-layer document sync, and verification were performed in the isolated 3D7 worktree.

## §5 Guard rules

1. Use `dev@b410f8955f09eefa33fbd3a17599e09c8a6ec706` as the exact integration baseline; do not import unrelated changes from another branch.
2. Keep this phase scoped to the already-designed quarantine contract and its actual JetStream failure/recovery evidence.
3. Use the trusted configured deployment/stream identity and explicit `MetadataOnly`; message tenant/Run claims are not authority.
4. Persist the Platform receipt before confirmed Term ACK. Any write failure leaves the source pending.
5. Exact duplicate replay must be idempotent; conflicting/incomplete facts must fail closed.
6. The serial runner is required because the existing and new integration tests share a JetStream subject.
7. Do not add a migration, alter the W/T/M inventory, or change Multipass inventory in this phase.
8. Do not convert test evidence into production consumer, readiness, BI, Benchmark, reservation, or execution claims.
9. Keep `target/` logs and disposable service data scoped to unique owned paths; stop services before cleaning their owned data.
10. Preserve the two-record Multipass test inventory cap, including stopped/deleted records, and do not auto-purge.
11. Keep production KMS, DB/NATS identities, operator access, worker/provider/readiness, and Schedule/Task execution gates open.
12. Do not edit unrelated Clippy dependency warnings as part of this narrow reliability slice.

## §6 Signoff

| Role | Reviewer | Decision |
|---|---|---|
| Architecture | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 3D7 scope and owner boundary reviewed; overall 9F4D remains open. |
| SRE | SRE Lead (Mavis 接手审核) | 🟢 Disposable PostgreSQL/NATS start-stop and owned-data cleanup verified. |
| Platform | Platform owner (Mavis 接手审核) | 🟢 Receipt-before-Term and exact duplicate recovery verified; production identity/KMS remain open. |
| Review | 评审主持 (Mavis 接手审核) | 🟢 Focused diff and current-design alignment reviewed. |
| Product | PM (Mavis 接手审核) | 🟢 No navigation, Run App ownership, or execution-status overclaim. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the 3D7 integration test, serialized shared-subject runner, 22/22 PostgreSQL/NATS steps, 2/2 Rust integration tests, zero-VM result, document sync and remaining production gates. | Phase 9F4D-3D7 closes the bounded real redelivery/ACK-loss verification gap. |
