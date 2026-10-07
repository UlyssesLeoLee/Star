# PHASE-9F4D-3D4-JETSTREAM-PUBLISHER-REPORT

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
