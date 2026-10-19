# PHASE-9F4J.1 Native BI Event Adapter Report

**Date:** 2026-10-19
**Branch:** `codex/wbs-9f4j-native-event`
**Base:** `59fbdacbb106898fded81f5d0455b53d43dbaff3`
**Adapter source commit already in dev:** `4ed8e6f891e4e1fcfdb1b5d86e48d678e26889fa`
**Scope:** Rebase review and independent revalidation of the isolated 9F4J.1 adapter slice; this report does not close 9F4J or enable production ingress.

The private lane started at `3c7fa58c8bce4b5083460ee24bbffa6307acd7d1`. Immediately before rebase, local `dev` was `59fbdacbb106898fded81f5d0455b53d43dbaff3`. `git rebase refs/heads/dev` reported that the lane commit was previously applied and skipped; the matching adapter source commit `4ed8e6f891e4e1fcfdb1b5d86e48d678e26889fa` is an ancestor of `dev`. The private branch therefore landed on the `dev` base without a duplicate source commit. This follow-up changes only the lane report, its WBS evidence, and the assigned rebase brief.

## §0 Purpose

Record the implementation review and post-rebase validation evidence for the bounded native gRPC Event adapter, with the explicit limits required by `docs/briefs/WBS-9F4J1-REBASE-20261019.md`, the original lane brief, and WBS section 4, phase 9F4J.1.

## §1 Change Matrix

| Manifest path | Result |
|---|---|
| `crates/platform-messaging-runtime/src/native_grpc.rs` | Reviewed current dev source from `4ed8e6f891e4e1fcfdb1b5d86e48d678e26889fa`: opt-in Event `PullOne`, `VerifyOrigin`, and lease-fenced `AcknowledgeOne`; provider identity uses provider ID/version, profile digest, and endpoint identity; request/frame limits and no background prefetch. No source edit was needed in this revalidation. |
| `crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs` | Reviewed current dev tests for scope validation, profile/sender/Event/lease pins, malformed route and identity, digest mismatch, 16 KiB wire boundary, exact-wire origin verification, and ACK fencing. Re-ran the package library tests after rebase. |
| `docs/implementation-plans/WBS-PLATFORM-RUNTIME-DELTA-20261018.md` | Updated 9F4J.1 provenance, validation results, identity and ACK boundary limits, and remaining gates. |
| `docs/reports/PHASE-9F4J1-NATIVE-EVENT-REPORT.md` | This post-rebase review, evidence, and limitation report. |
| `docs/briefs/WBS-9F4J1-REBASE-20261019.md` | Assigned rebase/revalidation brief with this lane's recorded outcome. |
| `docs/briefs/wbs-9f4j-native-event-20261018.md` | Original lane brief retained as the source acceptance manifest. |

The pre-existing untracked `docs/briefs/wbs-platform-runtime-20261018.md` was preserved and is not part of this lane's changes. No other agent's files were included.

## §2 Validation Summary

Commands were run through RTK with `E:\DevCache\cargo\bin` on `PATH`. The selected toolchain was Rust 1.99.0 (`cargo 1.99.0 (5f94df478 2026-08-27)`, `rustc 1.99.0 (b940084d7 2026-09-28)`, `rustfmt 1.10.0-stable`).

| Gate | Command | Result |
|---|---|---|
| Target-file format | `rustfmt +1.99.0 --edition 2021 --check crates/platform-messaging-runtime/src/native_grpc.rs crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs` | Exit 0. |
| Crate format | `cargo +1.99.0 fmt -p platform-messaging-runtime -- --check` | Exit 1 because rustfmt reports existing changes in five out-of-manifest files: `admission_delivery.rs`, `admission_receive.rs`, `native_grpc_server.rs`, `transport_profile_pg.rs`, and `tests/native_origin_verification_pg.rs`. Those files were not edited. The two authorized Rust files pass the direct rustfmt check above. |
| Focused check | `cargo +1.99.0 check -p platform-messaging-runtime --no-default-features --features native-grpc-adapter --lib -j 4` | Exit 0. Two existing dead-code warnings remain in `transport_profile.rs` (`TransportMessageKind::from_database`, `TransportProfileLifecycle::from_database`). |
| Focused unit tests | `cargo +1.99.0 test -p platform-messaging-runtime --no-default-features --features native-grpc-adapter --lib -j 4` | Exit 0: 50 passed, 0 failed, 0 ignored. |
| Clippy | `cargo +1.99.0 clippy -p platform-messaging-runtime --no-default-features --features native-grpc-adapter,durable-quarantine --all-targets -j 4` | Exit 0. The extra feature is needed for the crate's existing PostgreSQL integration-test imports during `--all-targets`; no database integration test was run. Warnings remain in untouched `star-context`, `domain-automation`, and `transport_endpoint.rs`; no Clippy warning was reported in the two lane Rust files. |
| Cypher header | `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | This follow-up: exit 0, `PASS (0 source file(s) checked)` because only docs changed. The original source commit separately recorded checks for two changed Rust files. No CodeGraph repository index was present, so neither result is graph-index or coverage verification. |
| Whitespace | `git diff --check` | This follow-up: exit 0 with no whitespace errors in the documentation diff. The original source commit recorded a CRLF-to-LF advisory for `native_grpc.rs`. |
| Rebase | `git rebase refs/heads/dev` | Exit 0. Git skipped original lane commit `3c7fa58c` as previously applied; source-equivalent commit `4ed8e6f891e4e1fcfdb1b5d86e48d678e26889fa` is already an ancestor of the `59fbdacbb106898fded81f5d0455b53d43dbaff3` local dev base. The private branch was not written to `dev`. |

The crate-wide format failure is an existing out-of-scope drift and remains open. It was not resolved by formatting files outside this lane manifest.

## §3 Known Gaps

- This slice provides a transport adapter and unit tests only. It does not provide BI consumer composition, owner Inbox/fact transaction, idempotent receipt persistence, checkpoint/replay recovery, quarantine flow, or Run BI/Benchmark projection.
- The adapter cannot prove owner commit-before-ACK. The caller contract requires the owner to commit its inbox before it invokes ACK; that owner lifecycle is outside this phase.
- Commit-before-ACK is a documented caller convention, not an invariant enforced by the adapter API: `acknowledge()` can be called without a typed owner-commit receipt or a preceding origin-verification result. A production caller must supply that ordering, and it remains unproven here.
- `receiver_service_id` is retained in the local route scope, but `PullOneRequest` does not transmit it; the server derives the receiver from the authenticated client-certificate fingerprint. This lane does not prove that the trusted resolver selected a certificate registered to the local receiver ID. Production composition must bind those identities and verify the server-side mapping before enabling ingress.
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
2. The 2026-10-19 rebase brief authorizes rebasing this private lane onto the inspected local `dev`; do not modify `dev`, push, clean, or change another checkout from this worker lane.
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
| v0.2 | 2026-10-19 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the private rebase onto current local `dev`, identify the already-present adapter source commit, re-run focused Rust 1.99.0 gates, and clarify that receiver/certificate binding and owner commit-before-ACK remain composition gates. |
