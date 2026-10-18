# PHASE-9F4J.1 Native BI Event Adapter Report

**Date:** 2026-10-18
**Branch:** `codex/wbs-9f4j-native-event`
**Base:** `6f8d5b8f2d00b1515928c9a25e0762a45cf30e4e`
**Scope:** Isolated 9F4J.1 adapter slice only; this report does not close 9F4J or enable production ingress.

## §0 Purpose

Record the implementation and validation evidence for the bounded native gRPC Event adapter in this lane, with the explicit limits required by `docs/briefs/wbs-9f4j-native-event-20261018.md` and WBS section 4, phase 9F4J.1.

## §1 Change Matrix

| Manifest path | Result |
|---|---|
| `crates/platform-messaging-runtime/src/native_grpc.rs` | Adds/finishes opt-in Event `PullOne`, `VerifyOrigin`, and lease-fenced `AcknowledgeOne` adapter behavior; pins provider identity to provider ID/version, profile digest and endpoint identity; keeps bounded requests and no background prefetch. Cypher header now records the verification-pin relationship and retains the existing Rust-server source attribution. |
| `crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs` | Adds focused tests for scope validation, profile/sender/Event/lease pins, malformed route and identity, digest mismatch, 16 KiB wire boundary, exact-wire origin verification and ACK fencing. |
| `docs/implementation-plans/WBS-PLATFORM-RUNTIME-DELTA-20261018.md` | Records 9F4J.1 source/test evidence, validation and remaining gates; explicitly keeps the larger BI and execution paths fail closed. |
| `docs/reports/PHASE-9F4J1-NATIVE-EVENT-REPORT.md` | This phase evidence and limitation report. |
| `docs/briefs/wbs-9f4j-native-event-20261018.md` | Lane brief retained as the acceptance manifest. |

No other agent's files or unrelated brief were included.

## §2 Validation Summary

Commands were run through RTK with `E:\DevCache\cargo\bin` on `PATH`. The selected toolchain was Rust 1.99.0 (`cargo 1.99.0 (5f94df478 2026-08-27)`, `rustc 1.99.0 (b940084d7 2026-09-28)`, `rustfmt 1.10.0-stable`).

| Gate | Command | Result |
|---|---|---|
| Target-file format | `rustfmt +1.99.0 --edition 2021 --check crates/platform-messaging-runtime/src/native_grpc.rs crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs` | Exit 0. |
| Crate format | `cargo +1.99.0 fmt -p platform-messaging-runtime -- --check` | Exit 1 because rustfmt reports existing changes in five out-of-manifest files: `admission_delivery.rs`, `admission_receive.rs`, `native_grpc_server.rs`, `transport_profile_pg.rs`, and `tests/native_origin_verification_pg.rs`. Those files were not edited. The two authorized Rust files pass the direct rustfmt check above. |
| Focused check | `cargo +1.99.0 check -p platform-messaging-runtime --no-default-features --features native-grpc-adapter --lib -j 4` | Exit 0. Two existing dead-code warnings remain in `transport_profile.rs` (`TransportMessageKind::from_database`, `TransportProfileLifecycle::from_database`). |
| Focused unit tests | `cargo +1.99.0 test -p platform-messaging-runtime --no-default-features --features native-grpc-adapter --lib -j 4` | Exit 0: 50 passed, 0 failed, 0 ignored. |
| Clippy | `cargo +1.99.0 clippy -p platform-messaging-runtime --no-default-features --features native-grpc-adapter,durable-quarantine --all-targets -j 4` | Exit 0. The extra feature is needed for the crate's existing PostgreSQL integration-test imports during `--all-targets`; no database integration test was run. Warnings remain in untouched `star-context`, `domain-automation`, and `transport_endpoint.rs`; no Clippy warning was reported in the two lane Rust files. |
| Cypher header | `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | Exit 0: 2 changed source files checked. No CodeGraph repository index was present, so this is header-guard validation only, not graph-index or coverage verification. |
| Whitespace | `git diff --check` | Exit 0; Git emitted only a CRLF-to-LF advisory for `native_grpc.rs`. |

The crate-wide format failure is an existing out-of-scope drift and remains open. It was not resolved by formatting files outside this lane manifest.

## §3 Known Gaps

- This slice provides a transport adapter and unit tests only. It does not provide BI consumer composition, owner Inbox/fact transaction, idempotent receipt persistence, checkpoint/replay recovery, quarantine flow, or Run BI/Benchmark projection.
- The adapter cannot prove owner commit-before-ACK. The caller contract requires the owner to commit its inbox before it invokes ACK; that owner lifecycle is outside this phase.
- Production Profile loading, production service identity and certificate rotation, target database migrations/RLS/grants, real network interoperability, process crash recovery, RSS/latency/fairness measurements, and production readiness were not validated.
- `durable-quarantine` was enabled for Clippy's all-target compilation only. This did not execute the PostgreSQL integration tests or validate a database.
- Crate-wide formatting remains blocked by the five files listed in §2. The required target-file formatting gate passed.
- CodeRabbit's Windows shim delegates to `/root/.local/bin/coderabbit` in WSL; running it through RTK returned `Permission denied`. An independent CodeRabbit review was therefore unavailable; this lane received local source/diff review only.
- 9F4J.2–9F4J.6 and the full 9F4J exit gate remain open. BI ingress and task execution remain fail closed.

## §4 Subagent Failure Handoff

No delegated subagent failure occurred in this lane. The native adapter edits were inherited, then reviewed and completed within the assigned manifest. The lifecycle lane remains separately owned and was not inspected or changed by this report's implementation work.

## §5 Applicable Guardrails

The following are the phase's acceptance guardrails, not claims that other WBS phases have passed:

1. Stay within the lane manifest; do not alter `D:\Star`, shared `dev`, or another agent's checkout.
2. Do not rebase, merge, push, clean, or modify the shared integration target from this worker lane.
3. Route shell/Git/Cargo checks through RTK.
4. Use the workspace Rust 1.99.0 baseline for the reported acceptance gates.
5. Report crate-wide formatting separately from direct formatting of authorized files.
6. Do not expand the scope to unrelated files merely to make the crate-wide formatter pass.
7. Keep Event delivery opt-in and fail closed.
8. Bind every operation to the fixed tenant, sender, receiver, Event kind and route scope.
9. Bind provider identity to provider ID/version, profile digest and endpoint identity.
10. Validate log ID/generation, positive sequence, message ID, lease ID and attempt metadata before accepting a delivery.
11. Hash and verify the exact wire bytes; reject digest mismatch and wire larger than 16 KiB.
12. Bound the gRPC frame to 32 KiB and RPC deadlines to the configured finite maximum.
13. Permit at most one outstanding delivery per adapter and do not add background prefetch.
14. Fence ACK with the same sender, Event kind, log generation, sequence, lease, message and wire digest.
15. Do not represent transport ACK as proof of owner Inbox commit-before-ACK.
16. Do not claim consumer composition, database/RLS/grants, recovery or production readiness from unit tests.
17. Preserve every unrelated dirty/untracked path and leave integration ownership to the coordinator.

## §6 Sign-off

| Role | Review record |
|---|---|
| Architect | Mavis review scope: 9F4J.1 adapter evidence only; no full BI ingress approval. |
| SRE Lead | No production operations or readiness claim is made by this slice. |
| Platform | Focused Rust 1.99.0 checks recorded in §2; coordinator integration review remains separate. |
| Review Chair | Scope and known gaps are recorded; the crate-wide formatter drift remains open. |
| PM | 9F4J remains in progress; only the isolated 9F4J.1 source slice is recorded here. |

## §7 Revision History

| Version | Date | Editor | Change |
|---|---|---|---|
| v0.1 | 2026-10-18 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record 9F4J.1 adapter evidence, exact validation outcomes, out-of-scope formatter drift, and open production gates. |
