# Phase 9F4C-D13 — Owner-local Admission Delivery

> v0.1 — 2026-10-11 — owner-local persistence/publisher slice; full phase and production execution remain open.

## §0 Purpose

Deliver Schedule request/resolution and Run result Outbox messages using each owner's own credentials and transaction boundary. Preserve original bytes, bounded recovery and immutable receipts. Broker PubAck does not prove downstream processing, infrastructure admission or task execution.

## §1 Change matrix

| Area | Candidate implementation | Status |
|---|---|---|
| PostgreSQL | Two owner-local Work queues and two immutable Transaction receipts; W/T/M 2/2/0; FORCE tenant RLS and native-only mutations | Idempotent migration and permission/RLS probes passed on final candidate |
| Rust owner library | Typed wire validation, at most eight serial publications, ten-second PubAck timeout, fixed owner, stream/domain verification and stable message ID | all-targets check and strict Clippy passed |
| Recovery | Lease fencing, twelve-attempt limit, seven-day exact retention, bounded reconciliation and permanent terminal receipts | Native retention/exhaustion/WORM and reconcile-cap tests passed; injected receipt rollback not covered |
| Broker | Real request crash replay/dedup, Run result publication and wrong-stream retry | Request/result/resolution and bounded retry/replay tests passed |
| Runner | Owned child-tree timeout cleanup, retained timeout logs, early flag checks, compile -j4/test threads 1 | Runner syntax and owned timeout-tree cleanup probe passed |
| Documentation | Requirements/basic/detailed/data design, plan and automation registry | Requirements/basic/detailed/data/plan and native replacement guidance synchronized |

## §2 Verification summary

Rust 1.99.0 all-targets `cargo check --offline --locked -p platform-messaging-runtime -p run-owner-runtime -j 4` passed (45.95 seconds). Existing domain-kms deprecation warnings remain. First native run timed out at 900 seconds while compiling the binding test; test assertions were not executed. Its isolated PostgreSQL stopped successfully; subsequent process inspection found no surviving Cargo/rustc from that run. Evidence: `target/phase9f4d-run-owner-consumer/da22528f-2218-4a40-b5fa-8aa67f32fcbd/audit.jsonl`.

Official NATS Server v2.12.14 test tool was downloaded with Python's default HTTPS certificate validation and matched the GitHub release API SHA-256 digest before guarded extraction. Version probe passed. Earlier EOF, connection timeout and Schannel revocation-service failures are retained as failed acquisition attempts. No certificate verification bypass, product dependency change or Multipass inventory mutation.

Final transport-port native/broker run: `target/phase9f4d-run-owner-consumer/ee749003-f308-4a2c-bba5-eb18ccf09e85/audit.jsonl` (67/67 passed steps; two actual Rust/PG integration tests). Both isolated services stopped normally. Request, result and actual Schedule-fence compensation resolution published; stable-ID crash replay matched the original broker sequence and retained one message within the dedupe window. Wrong-stream retry and pre-cancel were exercised. Metadata-only admin fixtures verified native retention/exhaustion/WORM/cap checks; these fixtures are not production identity or authorization evidence. All-target check, 23/23 library unit tests and strict Clippy --no-deps for platform-messaging-runtime/run-owner-runtime passed. Fresh port check took 11.99s and strict Clippy 6.31s. Owned timeout-tree probe passed (`d13-timeout-c57ab196-e0c9-4b6d-a52e-afd7b2875565/summary.json`). Two intermediate builds failed LNK1104 opening existing test exe paths; no live matching processes were found, disk had free space, target write probe passed; serial reruns succeeded. No cache deletion/security exception; exact root cause not established. No workspace/release/doc/bench or independent review pass claim. CodeRabbit version probe returned `/root/.local/bin/coderabbit: Permission denied`; independent review unavailable.

### Rust-native replacement preparation

`AdmissionTransport` and `AdmissionPublisher<T>` isolate owner policy from JetStream. The transitional adapter maps logical identity/routing to NATS APIs and verifies PubAck. Static dispatch avoids per-message future boxing. No native server provider or authenticated production deployment is implemented. The user's Rust server is the future replacement; its protocol, identity, durable proof, recovery and receiver commit-before-ack must pass equivalent gates. Current v1 proof has bounded log ID and positive durable cursor; opaque native receipts require versioned mapping/migration rather than emulating JetStream. Existing quarantine/capacity consumer and Cargo dependency isolation remain work before removing NATS from the complete product.

## §3 Known gaps

Authenticated owner broker provisioning and production composition, request/resolution/result receivers, Inbox-before-double-ACK, poison handling and receiver recovery remain open. The isolated broker test has no production authentication proof. Final transport-port regression verifies request/result/resolution broker publication, native retention/exhaustion and reconciliation capacity rejection. Outbox-enqueue-at-cap rollback, inserted-receipt transaction failure, concurrent contention and scale/performance still require separate acceptance. Host/cluster reservations, Multipass provider readiness, native hooks, runtime sandbox, task-card CLI closure, independent validation and BI/Benchmark remain open. Reviewed slice is intended for local dev; actual integration must be established by Git refs and the integration audit, not this report. Remote/main remain unchanged in this slice. Full objective remains active.

## §4 Subagent failures and takeover

No new subagent dispatched. Root owns this candidate lane. Root/user worktrees and dev CodeGraph files are preserved.

## §5 Guard rules

1. Each owner publishes with its own pool/credentials.
2. No cross-owner write transaction.
3. Fixed schema/role allowlist.
4. FORCE tenant RLS.
5. Original immutable wire and typed identity validation.
6. At most eight messages per serial batch.
7. Ten-second broker timeout and bounded lease.
8. Stable owner/kind/tenant/message ID.
9. Positive matching stream/domain PubAck only.
10. Twelve-attempt retry ceiling.
11. Exact seven-day Work retention.
12. Permanent audited terminal receipt before Work removal.
13. Queue cap failure rolls back original Outbox transaction.
14. Cancellation and stale-worker fencing.
15. Delivery does not enable execution or prove receiver commit.

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: owner-local boundaries; verified publisher slice; full phase open |
| SRE Lead | Mavis review: bounded queues/retries and honest timeout evidence |
| Platform | Mavis review: role pinning and tenant RLS; identity composition open |
| Review chair | Mavis review: actual broker tool verified; native/broker tests passed; independent review unavailable |
| PM | Mavis review: full product phases remain active |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-11 | Ulysses — Mavis接手审核 | Verified owner-local delivery and replaceable transport port; Rust-native future provider and honest gaps | Continue D12, official NATS tool acquisition, user native-server replacement direction |
