# D20 Brief — Infrastructure-owned native Result publication

## Purpose

Implement the missing native Infrastructure→Run capacity-result path for Star's durable Rust transport without sharing owner writes or depending on NATS-specific ACK semantics. This brief records the next implementation boundary; it does not claim the publisher or production route exists.

## Owner boundaries

- **Infrastructure owner** creates immutable capacity-result facts in its own transaction and owns its Outbox, delivery claims, retry state, lease fence, transport Profile pin, append receipt, and delivery completion.
- **Platform Messaging** authenticates the Infrastructure service identity, appends exact bytes idempotently to the pinned native log, and returns a durable receipt containing the native source identity needed for later verification.
- **Run owner** derives canonical tenant/Project/Cloud Branch/Run and demand scope from its persisted demand, commits its own Inbox/processing fact, and ACKs only after commit plus origin revalidation.
- **Platform quarantine owner** owns permanent malformed/content-conflict receipts. Its commit and exact-source revalidation must precede terminal disposition.

No owner may claim or mutate another owner's tables. No shared DB transaction spans Infrastructure, Run, or Platform Messaging.

## Immutable publication identity

The Infrastructure Outbox event fixes its event ID, schema, target Run route, source demand/correlation IDs, payload bytes and digest, Transport Profile ID/revision/digest, and trusted endpoint identity. Retries use the exact same bytes and pinned identity. A changed Profile, revoked identity, route mismatch, stale generation, expired attempt fence, or unknown append/receipt state must fail closed; the publisher must not redirect to a newly resolved endpoint or silently dual-publish through NATS.

The native append contract is idempotent for the authenticated producer identity plus immutable event identity and exact bytes. Replaying identical bytes returns the durable original receipt. Reusing the identity with different bytes is a permanent conflict that is quarantined; it must not overwrite the original receipt.

Infrastructure marks its Outbox delivery complete only after it persists the returned durable receipt in its own transaction under the current attempt fence. A timeout or ambiguous RPC result schedules bounded retry of the exact same event ID, bytes, Profile and endpoint. A socket write or RPC response alone is not durable acceptance.

## Event scope

`capacity_accepted`, `capacity_rejected`, and `capacity_expired` can enter the publisher allowlist only when their typed result DTO, Infra-owned fact, Run-owner Inbox, BI projection and replay/quarantine behavior agree on the same versioned contract. `capacity_released` stays disabled: the current Run-owner parser returns `ReleasedContractUnavailable`, so neither the native nor JetStream publisher may emit it until typed lifecycle semantics and the downstream consumer/BI contract are implemented end to end.

## Acceptance gates

1. Infrastructure claims are tenant/run scoped, bounded, lease-fenced, serialized or otherwise concurrency-bounded, cancellable and fair under load.
2. The profile/endpoint tuple is resolved from trusted deployment configuration and fixed for the Outbox event; agents, plugins and task-card CLI cannot supply or override it.
3. Exact duplicate append returns the same durable receipt; same identity with different bytes is rejected and quarantined.
4. Infrastructure receipt persistence precedes delivery completion; stale fence, missing receipt store, DB failure, or ambiguous append never marks delivery complete.
5. Run-owner event-ID/content conflict commits a durable quarantine receipt before terminal disposition; quarantine/source-verification failure leaves the delivery unsettled.
6. Run Inbox facts commit before ACK. Transport receipt never writes Run state or proves Host/cluster capacity, readiness, BI or execution.
7. PostgreSQL owner roles, FORCE RLS, migration replay, grants and receipt W/T/M classification are verified. Native transport TLS identity, revoke/rotate behavior, recovery/replay and NATS rollback are exercised.
8. BI/Benchmark reconciles immutable event facts, append receipts, Run Inbox facts, quarantine and retry outcomes without treating transport status as business completion.
9. Measure bounded memory, queue depth, throughput, tail latency and fairness at planned parallel Run/Agent load. Tests use at most two registered Multipass inventory instances, including stopped/deleted records.
10. Keep the feature and production startup route disabled until every above gate passes; do not enable Schedule/Run execution as a side effect of message transport readiness.

## Explicit non-goals

This phase must not add a global polling worker, automatic Profile retargeting, cross-owner SQL writes, host shell/Kubernetes access, direct BI writes from the transport adapter, or NATS/native dual-publish. It does not retire NATS; route-by-route parity and rollback evidence are a separate cutover gate.
