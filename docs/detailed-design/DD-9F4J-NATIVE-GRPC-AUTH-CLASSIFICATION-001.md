# DD-9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION-001 — Native event authorization error classification

> **Status: design synchronized; implementation slice PARTIAL.** Native and application error mapping is present in local dev. Focused source-lane tests, all-target compilation, formatting checks and independent source review passed. PostgreSQL/RLS/grants, live mTLS/RPC, commit-before-ACK network proof, Clippy and production identity acceptance remain NOT RUN.

## 1. Purpose

Preserve the distinction between a permanent authorization denial and a transport failure throughout the BI Task Event receive path. An authorization denial must stop the consumer in its existing fail-closed state. It must not be retried as network noise or converted into a successful terminal disposition.

This contract classifies errors only. It does not create identity, grant permissions, refresh a stale Profile, move ownership, or change the immutable delivery route.

## 2. Scope and ownership

- The Platform Messaging native gRPC adapter owns the mapping from tonic Status to the transport-neutral error class.
- The Star API composition adapter owns the mapping from native route errors into application ingestion errors.
- The Task Event consumer owns the stop/retry decision and durable owner transaction ordering.
- Identity, certificate, Profile, route-pin, destination-generation and event authorization remain authoritative in their existing owners. The error mapper cannot substitute for reauthorization.
- No table, migration, stored procedure, event payload or owner transaction is added by this change.

## 3. Classification contract

| Source result | Native error | Application error | Consumer disposition |
|---|---|---|---|
| gRPC Unauthenticated | Unauthorized | Unauthorized | Mark blocked and stop; do not retry as transport; do not ACK or terminal-dispose |
| gRPC PermissionDenied | Unauthorized | Unauthorized | Same as Unauthenticated |
| Any other gRPC status | Sanitized Transport | Transport | Use existing bounded attempt/backoff and lease policy |
| Route Contract, Transport, or Busy | Not applicable | Currently normalized to Transport | Keep existing bounded transport policy; do not introduce an unbounded retry |

The native mapper is shared by PullOne, VerifyOrigin, and AcknowledgeOne; the application mapper is shared by pull, origin verification, and ACK composition. The original server status message, details, metadata and source error chain are not part of the returned error.

The current normalization of Contract and Busy to application Transport is an explicit implementation constraint. If future evidence requires different retryability, introduce a separately reviewed typed classification rather than inferring it from free-form text.

## 4. Consumer state and recovery

1. **Authorization denied before a delivery is leased:** mark the consumer Blocked, stop the current receive loop, and do not acknowledge or create a BI fact.
2. **Authorization denied after pull or during origin verification:** retain the unconfirmed source delivery; do not persist unverified data, quarantine a terminal outcome, or ACK it. Let the existing lease/redelivery contract recover it.
3. **Authorization denied during ACK after owner commit:** stop the consumer and leave source confirmation unresolved. After authorization is restored, re-read the current certificate/profile/route/destination authorization. Redelivery must hit the BI owner's exact-replay receipt and return the original result without creating a second fact.
4. **Sanitized transport failure:** use only the current bounded attempt, deadline, backoff and lease policy. Unknown outcome must not be guessed as commit or rollback; owner exact replay and source redelivery reconcile it.
5. **Resume:** no stale positive authorization result may be reused. Resume requires a fresh authoritative check of identity, current Profile revision/digest, route direction, event kind and destination generation. If any fact is unavailable or mismatched, keep the consumer stopped.

This error mapping does not alter owner commit-before-ACK: the Inbox/fact commit remains the durable business receipt, while transport ACK only confirms source delivery.

## 5. Privacy, logging and resource bounds

- Public/application error text is a stable category and safe operation context only. Do not include tonic status details, response metadata, certificates, credentials, tenant/event identifiers, exact wire bytes or payload.
- Metrics may use bounded labels for error class and operation stage. Do not use unbounded tenant, Run, event, correlation or certificate identifiers.
- Keep existing one-in-flight delivery, payload/frame limits, deadlines, queue limits, retry budget and cancellation behavior. Error mapping must not allocate or retain a second payload copy.
- Authorization denial is a control-plane state transition, not a retry timer. A repeated denied request must not create a high-rate retry loop.

## 6. Acceptance matrix

| Check | Evidence/status |
|---|---|
| Native status mapping test native_event_status_mapping_is_fail_closed | PASS, 1 test on Rust 1.99.1 source lane |
| Application preservation test native_event_receive_errors_preserve_auth_and_retryable_transport | PASS, 1 test on Rust 1.99.1 source lane; first attempt hit a transient Windows linker lock, exact command retry passed |
| star-api-rest --all-targets | PASS on the source lane, with pre-existing warnings |
| Changed Rust files rustfmt; CypherGraph Guardian; git diff --check | PASS on the source lane |
| Independent source/spec review | PASS with no P0/P1; reviewer did not run tests |
| PostgreSQL roles/grants/RLS and migration acceptance | NOT RUN |
| Live mTLS, real native RPC, revoke/rotate and commit-before-ACK latch | NOT RUN |
| Clippy and post-replay Rust gates on local dev | NOT RUN |

## 7. Open gates

This document and the source slice do not close 9F4J. Before enabling the BI consumer, separately prove target database roles/grants/FORCE RLS, production certificate and Profile provisioning/rotation, real mTLS RPC behavior, commit-before-ACK with an independent database observer, exact replay after process restart, cancellation/revoke behavior, bounded recovery and operator visibility. Clippy and the applicable Rust 1.99.1 gates must also pass on the integrated target. Until then BI ingress and all Schedule/Agent execution paths remain fail closed.

Task-specific SRS/Basic/Detailed Design and implementation-plan references outside this lane's registered manifest remain a follow-on synchronization item. The parent requirements §50.8AX, Basic Design §16.79, this DD and Group WBS §23.21 are synchronized in this slice.

## 8. Revision history

| Version | Date | Change |
|---|---|---|
| v1.0 | 2026-11-08 | Defined the native/application authorization error mapping, consumer stop and bounded retry contract, privacy constraints, source evidence and open runtime gates. |
