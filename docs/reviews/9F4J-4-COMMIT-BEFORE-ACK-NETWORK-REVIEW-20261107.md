# Independent Review: 9F4J.4 Commit-Before-ACK Network Proof

**Review status:** ACCEPTED FOR LOCAL DEV INTEGRATION; LIVE ACCEPTANCE NOT RUN
**Baseline:** dev@4df294046125571ef90de64fb11891eb5002fd34
**Lane:** codex/9f4j4-network-latch-20261107

## Scope

Review the registered test-only proof for the 9F4J.4 requirement in the Group WBS. The required claim is narrow: the BI owner transaction is durable before the transport stores acknowledgement, and exact redelivery does not duplicate the BI receipt or fact.

## Required evidence

The final test and review must establish all of the following through the real system path:

1. Native gRPC+mTLS server and client are active with separately authorized Event identities.
2. The real TaskEventReceiver commits through TaskEventInboxStore before calling the ACK operation.
3. The latch pauses only the test-side ACK invocation; it does not fake transport receipt, owner commit, or ACK persistence.
4. A separate PostgreSQL observer sees the committed inbox receipt and fact while the source record remains leased/unacked and no ACK receipt exists.
5. Releasing the latch causes a real ACK RPC and durable source ACK receipt.
6. Exact redelivery returns the original BI receipt and leaves exactly one BI fact.
7. The test has bounded waits and respects lease timing, authorization/profile pinning, and exact-wire identity.

## Preliminary findings

- The architecture is feasible using the production TaskEventReceiver and native transport seam, with a test-only latch at the ACK boundary.
- The test seeds an Event through the authenticated `platform_transport.append_authenticated` database function. It exercises receiver-side native gRPC+mTLS pull, origin verification, and ACK, but it does not claim sender-side mTLS append.
- The first implementation review found:
  - **P1 — Database host validation ran after opening four connections** (initial test lines 286–297). Fixed by validating all four parsed hosts before the first `connect_pool` call. Re-review must confirm the new ordering.
  - **P2 — Concurrent test processes could interfere on the fixed Event route** (initial test lines 417–425 and 671–695). Fixed by acquiring a PostgreSQL transaction-scoped advisory lock before appending or pulling from the route. Re-review must confirm the lock is held through completion and released with the transaction.
- SPDX header was changed to `MIT` to match the selected project license.
- Focused Rust 1.99.1 test build passed; the pure host-guard test passed (1/1); targeted formatting and Cypher Graph Guardian checks passed. These do not replace a live acceptance run.
- The existing environment has no configured BI/transport test database or reachable local PostgreSQL endpoint and no provisioned test certificate set. Live PostgreSQL+mTLS acceptance remains NOT RUN.
- An observer database/process from another owner or unrelated source log is not interchangeable. All observers must inspect the same source delivery and BI facts consumed by the receiver.

## Independent re-review

- Confirmed all four parsed database hosts are checked before any connection pool is opened.
- Confirmed the transaction-scoped advisory lock is acquired before Event append, held through replay and post-ACK assertions, and committed to release after success. This serializes acceptance processes sharing the source database; a dedicated otherwise-idle route is still required.
- Confirmed `MIT` SPDX, exact test scope, and the documented boundary: receiver/server-side mTLS is exercised; sender-side mTLS append is not.
- Re-ran the pure database host guard test: 1 passed, 0 failed; JSONL entries parse; Rust 1.99.1 focused no-run build and rustfmt pass.
- No new blocking issue found. This is an integration review of the harness, not a live PostgreSQL/mTLS result.

## Review disposition

**ACCEPTED FOR LOCAL DEV INTEGRATION.** The independent re-review confirmed that the first pass's P1/P2 findings are fixed and that the test harness matches its documented scope. PostgreSQL+mTLS acceptance remains NOT RUN until executed against the designated disposable environment. This review does not establish production readiness.

## Change log

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses (12-role company per DEC-008) — Mavis review | Recorded the test protocol, feasibility caveats, environment blockers, and pending implementation review; made no approval claim | 9F4J.4 lane start |
| v0.2 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis review | Recorded first code review findings (P1 database host validation ordering, P2 fixed-route concurrency), their scoped fixes, focused build/static-check evidence, and live-test limitation; approval remains pending independent re-review | Candidate implementation arrived |
| v0.3 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis review | Recorded independent re-review confirmation of the P1/P2 fixes and accepted the registered harness/documentation for local dev integration while preserving live PostgreSQL/mTLS as NOT RUN | Focused fixes re-reviewed |
