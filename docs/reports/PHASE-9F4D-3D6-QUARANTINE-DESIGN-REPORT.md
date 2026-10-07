# PHASE-9F4D-3D6-QUARANTINE-DESIGN-REPORT

> Status: 🟡 Design increment recorded; implementation and production enablement are open.
> Date: 2026-10-08
> Base: local `dev` at `5577e3b1358f7f214f8838f17c8faba4ebecc171`
> Lane: `codex/phase9f4d3d6-quarantine-contract`

## 0. Purpose

Define a safe poison-message quarantine boundary for the 9F4D-3D5 Run-owner JetStream consumer. This increment separates malformed-message handling from tenant-scoped Run facts and records trusted broker identity, immutable receipt, conditional payload storage, fault disposition, and terminal-ACK ordering. It is a design checkpoint only: no database migration, writer, ACK handler, or production consumer was implemented, so Phase 9F4D-3D6 is not complete.

## 1. Change matrix

| Area | Change | Current state |
|---|---|---|
| Product requirements | Added AC-INFRA-002-Q and SCHED-API-027 for a Platform Messaging/Ingress quarantine owner, trusted deployment/stream-generation/stream/sequence identity, immutable receipt and commit-before-Term. | Specified; unimplemented. |
| Basic design | Added §16.41 plus Schedule API basic-design §6.12. Quarantine is a platform service, not a Run navigation node; Run/BI/reservation/readiness/execution facts remain unchanged. | Specified; retention and fault policy remain open. |
| Detailed design | Added Schedule API §6.13 and Group DD §8.15 for owner/role isolation, exact-wire digest, payload separation, duplicate/conflict, retry/backpressure, operator signals and acceptance gates. | Specified; no schema or service. |
| Data classification | Added Data Design §4.13.10: receipt is append-only Transaction; optional raw bytes are a separate TTL Work record. Both are conceptual classifications only. | No tables, migrations, grants or purge job. |
| Implementation plan | Added §6.101 and a phase report link; explicitly records the unresolved user retention decision and leaves 3D6 open. | Plan synchronized. |
| Multipass/K3s | Kept Multipass as the Linux guest VM provider, separate Host/instance/cluster/node identities, explicit Run cluster binding, and maximum two registered test instances including Stopped/Deleted. | No VM created; no inventory changed. |
| Runtime code | No code changed. The current 3D5 consumer already hydrates canonical scope from its immutable demand row and remains production-disabled for poison messages. | No Rust, migration, script, or runtime behavior changed. |

## 2. Verification summary

- This increment changed documentation only. No Cargo command, unit/integration test, PostgreSQL migration, NATS runner, or VM command was run.
- Manual cross-reference inspection confirmed the increment is recorded in the main requirements (v5.99), Schedule SRS (v0.28), main Basic Design (v5.96), Schedule Basic/Detail Design (v0.27/v0.27), Group DD (v4.65), Data Design (v1.9), implementation plan (v6.28), and this report.
- The previous Phase 9F4D-3D5 report records its final disposable PostgreSQL/NATS runner as 20/20 steps and the real JetStream integration test as 1/1 passed. This is prior-stage evidence, not a test run in this increment.
- The local `rtk` command is unavailable in this environment. Read-only inspection and document edits used PowerShell tooling directly.
- The two-record Multipass cap applies to total registered inventory, including Stopped/Deleted records; this increment created zero VMs and did not modify inventory.

## 3. Known gaps

1. The user has not selected encrypted raw-payload retention for 24 hours or metadata-plus-digest-only storage. No default is chosen.
2. Permanent versus transient fault taxonomy is not finalized. `InvalidStoredDemand` remains an owner-data integrity error and cannot be Term-acked automatically.
3. Quarantine schema, platform role, RLS/access policy, exact receipt retention, body encryption/key lifecycle, quota, TTL purge, operator UI/access audit and migration privileges are not implemented or verified.
4. No Rust quarantine store/writer, consumer routing, `double_ack_with(AckKind::Term)`, redelivery/backpressure tests, or production broker identity/stream generation configuration exists.
5. Run-owner consumer provisioning, `capacity_released` producer, current authorization/binding revalidation, production worker, provider/Manager, K3s readiness, independent Agent sandbox, BI/Benchmark and task execution remain open and fail-closed.
6. No target database identity/grants, production RLS, performance/RSS benchmark, or end-to-end multi-Run acceptance was performed.

## 4. Subagent results and takeover

- The earlier read-only quarantine audit recommended a separate Platform Messaging/Ingress owner and a trusted source tuple based on configured deployment/stream generation plus JetStream stream/sequence. Its findings were used as design input; it made no repository edits.
- A follow-up to the previous schema runner confirmed the first 3D5 attempt failed at `consume_one()` with `InvalidStoredDemand`, without a Rust backtrace or raw serde/SQLx error. The current consumer derives omitted Project/Branch/Run scope from the immutable demand Outbox row. The 3D5 report records the later final runner as passing 20/20 steps and 1/1 integration case; this was not rerun here.
- No subagent commit was imported and no new worker was dispatched for this documentation-only increment.

## 5. Guard rules

1. Do not store malformed messages in the Run-owner tenant Inbox or write another owner's tables.
2. Never derive quarantine authority or dedupe identity from envelope tenant/Run claims, payload event IDs, subject, or `Nats-Msg-Id`.
3. Use trusted deployment identity, configured stream generation, server stream name and server stream sequence as the receipt key.
4. Keep receipt Transaction data append-only; keep any optional raw body in a separate encrypted, bounded Work store with explicit TTL and purge evidence.
5. Do not choose or implement raw-payload retention until the user selects a policy.
6. Do not Term-ack before the receipt and selected payload action commit.
7. Exact identity replay is idempotent only when digest/fault metadata match; conflicting digest is an integrity incident and stays unacknowledged.
8. Database, quota, key, unknown metadata, unknown fault classification and other transient failures stay unacknowledged with bounded backpressure.
9. Keep `InvalidStoredDemand` non-terminal until a reviewed versioned classifier resolves its owner-data integrity semantics.
10. Quarantine must not mutate TaskExecutionRun, Schedule, RunEvents/BI, reservations, readiness, or execution.
11. Keep payload content out of logs, traces, metrics labels, and BI.
12. Keep the existing single-message, 132 KiB and 2-second pull bounds unless a reviewed versioned contract changes them.
13. Preserve Multipass multi-cluster identity separation; at most two registered test instances total, counting Stopped/Deleted; never auto-purge.
14. Rust workspace minimum remains 1.99.0; this docs-only increment does not claim compiler validation.
15. Production consumer and TaskExecutionRun execution remain fail-closed until the owner, retention, classifier, migrations/grants, runtime wiring and failure gates are complete.

## 6. Review roles

| Role | Review |
|---|---|
| Architect | Mavis review — owner boundaries, trusted source key and acknowledgement order recorded. |
| SRE Lead | Mavis review — transient failure/backpressure and operator signal requirements recorded; not runtime-tested. |
| Platform | Mavis review — Multipass two-record cap, zero-VM test boundary and Rust 1.99.0 baseline preserved. |
| Review host | Mavis review — cross-document versions aligned; code and schema intentionally unchanged. |
| PM | Mavis review — 3D6 remains open pending the user's payload-retention choice and implementation gates. |

## 7. Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the 3D6 quarantine owner, trusted source identity, immutable receipt, optional Work payload, commit-before-Term invariant, unresolved retention/taxonomy, and all unimplemented production gates. | Continue Phase 9F4D after integrating the 3D5 Run-owner consumer. |
