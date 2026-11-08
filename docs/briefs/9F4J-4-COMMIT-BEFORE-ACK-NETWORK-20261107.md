# Brief: 9F4J-4 Commit-Before-ACK Network Proof

**Phase:** WBS-9F4J-4
**Lane:** codex/9f4j4-network-latch-20261107
**Baseline:** local dev@4df294046125571ef90de64fb11891eb5002fd34
**Created:** 2026-11-07

## Objective

Add a focused PostgreSQL integration harness that proves the owner transaction commits before the native messaging service records ACK. The test must use the real native gRPC+mTLS transport, TaskEventReceiver, TaskEventInboxStore, and an independent database observer. It must hold the ACK boundary open while inspecting committed BI facts and source delivery state.

This lane provides a reproducible acceptance harness. It does not claim that a live database or certificate environment is available or that the end-to-end acceptance has passed.

## Required scenario

1. Seed one authorized Event through the authenticated database append function, then pull it over the real native gRPC+mTLS client. The append RPC itself is outside this test's assertion.
2. Let TaskEventReceiver commit the exact wire into the BI owner inbox and fact store.
3. Pause immediately before the real ACK RPC. From a separate observer connection, confirm the BI receipt and fact are committed, the source delivery remains leased/unacked, and no ACK receipt exists.
4. Abort the receiver task while the ACK latch is still held, simulating interruption after BI commit and before the ACK RPC. Wait for the source lease to expire.
5. Pull the exact same wire again through native gRPC+mTLS. Confirm exact replay returns the same receipt, the receiver sends a real ACK, and no second fact is added.

The latch may wrap only the test-side ACK call. It must not replace the production transport, receiver, database transaction, or durable ACK path.

## Scope and ownership

Only the following MAGOS-registered files may change in this lane:

- crates/star-api-rest/tests/bi_task_event_commit_before_ack_pg.rs
- docs/briefs/9F4J-4-COMMIT-BEFORE-ACK-NETWORK-20261107.md
- docs/reports/PHASE-9F4J-4-COMMIT-BEFORE-ACK-NETWORK-20261107.md
- docs/reviews/9F4J-4-COMMIT-BEFORE-ACK-NETWORK-REVIEW-20261107.md
- docs/reports/WBS-DISPATCH-9F4J-4-COMMIT-BEFORE-ACK-NETWORK-20261107.jsonl

No production source, migration, Cargo manifest/lockfile, WBS, protected W8/Dependabot content, main, remote, or user files are in scope.

## Preconditions and evidence

- Dedicated disposable BI and native transport PostgreSQL databases with the required schemas, migrations, roles, grants, profile, certificate identities, Event log binding, and test route. The harness verifies their exact database names before it writes.
- The source route must have the sender certificate fingerprint registered and authorized for the pinned Event profile. The test seeds through `platform_transport.append_authenticated`; it does not exercise sender-side mTLS append. The receiver client certificate and server certificate are exercised by the real native gRPC+mTLS pull, origin-verification, and ACK path.
- Required inputs are `STAR_BI_TASK_EVENT_TEST_DATABASE_URL`, `STAR_BI_TASK_EVENT_TEST_OBSERVER_DATABASE_URL`, `STAR_PLATFORM_PROFILE_TEST_DATABASE_URL`, `STAR_PLATFORM_PROFILE_TEST_OBSERVER_DATABASE_URL`, `STAR_BI_TASK_EVENT_TEST_DEPLOYMENT_ID`, `STAR_BI_TASK_EVENT_TEST_TENANT_ID`, `STAR_BI_TASK_EVENT_TEST_PROFILE_ID`, `STAR_BI_TASK_EVENT_TEST_PROFILE_REVISION`, `STAR_BI_TASK_EVENT_TEST_SENDER_SERVICE_ID`, `STAR_BI_TASK_EVENT_TEST_RECEIVER_SERVICE_ID`, `STAR_BI_TASK_EVENT_TEST_DESTINATION_ID`, `STAR_BI_TASK_EVENT_TEST_DESTINATION_GENERATION`, `STAR_BI_TASK_EVENT_TEST_PROVIDER_VERSION`, `STAR_BI_TASK_EVENT_TEST_ENDPOINT_IDENTITY`, `STAR_BI_TASK_EVENT_TEST_ENDPOINT_URI`, `STAR_BI_TASK_EVENT_TEST_TLS_SERVER_NAME`, `STAR_BI_TASK_EVENT_TEST_LOG_GENERATION_POLICY`, `STAR_BI_TASK_EVENT_TEST_SENDER_CERT_SHA256`, `STAR_BI_TASK_EVENT_TEST_SERVER_CA_PEM`, `STAR_BI_TASK_EVENT_TEST_RECEIVER_CERT_PEM`, and `STAR_BI_TASK_EVENT_TEST_RECEIVER_KEY_PEM`. `STAR_BI_TASK_EVENT_TEST_PROFILE_SECRET_REFERENCE` is optional. Private values must not be printed or stored in this lane.
- PostgreSQL may run on loopback or an IP-literal private Multipass guest address; public and unresolved database hosts are rejected before the harness opens any database connection. Test database names and the dedicated, otherwise idle test profile/log route remain mandatory.
- A PostgreSQL transaction-scoped advisory lock serializes acceptance processes sharing the source test database and prevents them from interleaving on the fixed Event route.
- Rust 1.99.1 focused all-targets compile and test-build evidence.
- A live run against the designated disposable databases, endpoint, and certificate set is required before marking network acceptance PASS.

If any prerequisite is missing, the ignored test cannot be enabled; compile success and pre-seeded fixtures are not network acceptance. This test proves native receiver-side pull/origin-verification/ACK ordering and exact replay; it does not test sender-side mTLS append or append-over-gRPC.

## Guardrails

- Preserve the immutable provider/profile/endpoint and exact-wire identity for retry.
- Never ACK before the BI owner transaction commits.
- Keep the receiver bounded to one in-flight delivery and preserve cancellation/deadline handling.
- Do not treat ACK transport success as BI persistence, or BI persistence as production readiness.
- Keep Schedule, Task execution, accepted settlement, fan-out, and production ingress disabled pending their independent gates.

## Handoff

The implementation worker owns only the registered Rust test file. The coordinator owns the four registered documentation files, performs independent source/diff review, and records exact validation results. Integration into local dev is allowed only after the candidate is reviewed and focused checks complete; PostgreSQL/mTLS acceptance remains NOT_RUN until it is actually exercised.
