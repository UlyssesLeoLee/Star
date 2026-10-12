# PHASE-9F4C-D19 — Native server authentication slice report

> Status: 🟡 Partial implementation and targeted gates passed; production transport and Schedule/Run execution remain closed.
> Date: 2026-10-12 · Revision: v0.1 · Reviewer: Ulysses — Mavis接手审核

## §0 Purpose

Record the D19 Star-owned Rust gRPC server and mTLS-to-service/Profile/log authorization slice, its targeted verification evidence, and the remaining gates before production traffic or execution may use it.

## §1 Change matrix

| Area | Change | Evidence / state |
|---|---|---|
| Rust server | Added optional Tonic `DurableTransport` handlers for Append, PullOne and AcknowledgeOne; mandatory client-certificate TLS setup; bounded frames, payload, concurrency and DB deadlines. | Rust 1.99.0 target check and Clippy pass; native-feature lib tests 35/35. |
| Identity and routing | Added certificate fingerprint identity/revocation facts and immutable Profile revision/route/log-generation bindings; authenticated wrappers recheck current state and call the D18 durable-log functions. | PostgreSQL 18.6 contract test passes; target DB role/grant deployment and product provisioning API remain open. |
| Database hardening | Runtime role loses direct access to storage routines/tables and receives only authenticated wrapper EXECUTE; audit rows are append-only and RLS/FORCE RLS applies. | Migration re-applied twice; grants/RLS/WORM assertions passed in disposable DB. |
| Local module reuse | Retained the separately reviewed RustGameServer TLS helper adaptation; D19 server and auth logic are Star-written. AssetsLake remains read-only and no relay code was copied. | Source attribution and reuse boundaries documented in `DD-RUST-SERVER-REUSE-001.md`. |
| Documentation | Updated requirements, basic design, detailed design, reuse review, implementation plan, automation contract and registry. | This report and linked design changes are in the current isolated branch. |

## §2 Verification summary

- `cargo check -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4` — passed with Rust 1.99.0.
- `cargo clippy -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4` — passed; output contains warnings from unchanged `star-context`, `domain-automation` and `domain-kms`, with no D19 warnings.
- `cargo test -p platform-messaging-runtime --features native-grpc-adapter --lib -j 1` — 35/35 passed.
- Targeted `rustfmt --edition 2024 --config skip_children=true --check` for the changed Rust files and Cypher Graph Guardian `--changed --stamp` — passed.
- Disposable PostgreSQL 18.6 runner — 80/80 steps passed. D19 auth migration applied twice; positive exact-wire-hash append, exact replay, Pull lease, ACK, unknown/revoked certificate rejection, audit and role assertions passed; PostgreSQL stopped.
- Final SQL audit: `target/phase9f4d-run-owner-consumer/d19-server-auth-20261012-r3/audit.jsonl`. Multipass inventory was not modified and no Multipass instance was created.
- `cargo fmt --all -- --check` and complete package tests were not run in this slice. The previous package test invocation hit Windows `LNK1104`; this is not a passing package-test result. No real gRPC TLS handshake/e2e was run.
- Independent CodeRabbit review was unavailable: the CLI entry failed with `/root/.local/bin/coderabbit: Permission denied`. Local source, migration, tests and documentation review was completed; the CLI failure is not treated as a clean independent review.

## §3 Known gaps

1. No typed production control-plane API for certificate registration, revocation, rotation or Profile→log provisioning; current SQL setup is a disposable-test owner operation.
2. Profile `endpoint_identity` has no trusted resolver to a concrete HTTPS URI and TLS server-name pair.
3. Target database runtime credentials/grants and deployed certificate lifecycle are unverified.
4. No live gRPC client/server TLS e2e, production peer identity mapping, dual-provider contract suite or rollback-tested NATS cutover.
5. No production Run owner consumer committing Inbox/receipt before transport ACK; quarantine/replay and crash/recovery remain open.
6. No BI/Benchmark parity, throughput/tail-latency/fairness/RSS or full Cargo workspace/release/doc/bench gate.
7. The auth SQL fixture uses a fixed 32-byte Profile digest to test exact pin lookup; it does not recompute the canonical digest. D17's typed resolver unit test covers canonical digest construction separately.
8. No D19 SBOM/NOTICE verification for the complete shipped dependency closure. Native feature remains opt-in; do not route production traffic or enable Schedule/Run execution.

## §4 Subagent failure handoff

No subagents were dispatched for this slice; there is no subagent output to accept or recover. All code and verification statements above were checked against the current worktree and its audit artifacts.

## §5 Guard rules

| # | Guard | Result |
|---|---|---|
| 1 | Rust workspace/toolchain minimum is Rust 1.99.0; targeted package all-targets check uses `-j 4`. | ✅ Targeted check passed on Rust 1.99.0; this is not a workspace-wide check. |
| 2 | Format only changed Rust files; do not reformat unrelated files. | ✅ Targeted rustfmt check passed; full workspace fmt remains outside this report. |
| 3 | Run target crate Clippy and inspect warnings. | ✅ Passed; listed warnings belong to unchanged dependencies. |
| 4 | Run focused Rust unit tests for changed behavior. | ✅ Native-feature library tests 35/35. |
| 5 | Full package/integration tests are a separate gate. | ⚠️ Not passed; previous link attempt failed with Windows `LNK1104`, current slice did not rerun it. |
| 6 | Exercise database authorization under the disposable PostgreSQL chain. | ✅ Runner 80/80; exact report and logs retained. |
| 7 | Reapply schema migrations to prove rerun behavior. | ✅ D19 auth migration applied twice. |
| 8 | Verify tenant/deployment RLS and append-only audit boundaries. | ✅ Disposable SQL contract passed. |
| 9 | Runtime DB identity may call only its least-privilege authenticated surface. | ✅ Raw storage routines/tables denied; wrappers granted in disposable test. |
| 10 | TLS peer identity must be mandatory and fail closed. | ✅ Code requires peer certificate and server TLS config requires client auth; live network handshake still open. |
| 11 | Derive service identity from certificate registry, never message claims. | ✅ Handler submits peer fingerprint; SQL resolves active identity. |
| 12 | Fix Profile digest, route direction and exact log generation per request. | ✅ Wrapper rechecks active Profile and exact binding under transaction locks. |
| 13 | Bound RPC concurrency, DB wait, payload and frame sizes. | ✅ Code bounds 8 RPCs, 10-second DB operation, 16 KiB wire and 32 KiB gRPC frame; stress/RSS not tested. |
| 14 | Preserve source project state and reuse/license evidence. | ✅ Source repositories remain read-only; only authorized TLS helper adaptation is retained. |
| 15 | Keep NATS until provider parity and operational rollback are accepted. | ✅ NATS adapter retained; no cutover performed. |
| 16 | Keep Schedule/Run execution closed until identity, sandbox, consumer and validation gates pass. | ✅ Native provider remains feature-gated and execution fail closed. |
| 17 | Sync requirements/basic/detailed design and report only verified outcomes. | ✅ Seven design/plan/automation docs plus this report updated with partial status and limitations. |

## §6 Sign-off

| Role | Review |
|---|---|
| Architecture | 架构师 (Mavis 接手 agent per DEC-008) |
| SRE Lead | SRE Lead (Mavis 接手审核 per DEC-008) |
| Platform | 平台 (Mavis 接手审核 per DEC-008) |
| Review chair | 评审主持 (Mavis 接手审核 per DEC-008) |
| PM | PM (Mavis 接手审核 per DEC-008) |

## §7 Revision history

| Version | Date | Revision owner | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the D19 native gRPC server/auth slice, Rust and PostgreSQL evidence, reuse boundary and production blockers. | Continue D19 after user authorized direct module-level reuse of local Rust servers. |
| v0.2 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record that the independent CodeRabbit review could not run because its executable was denied; report local review separately. | Final review and integration readiness check |
