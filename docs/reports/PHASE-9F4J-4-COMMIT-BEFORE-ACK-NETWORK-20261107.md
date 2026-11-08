# PHASE-9F4J-4 — Owner Commit-Before-ACK Network Proof

> **Status: IN PROGRESS / LIVE ACCEPTANCE NOT RUN** · Revision v0.2 · 2026-11-08
> Lane: codex/9f4j4-network-latch-20261107 · Baseline: dev@4df294046125571ef90de64fb11891eb5002fd34

## §0 Purpose

Record the test harness and evidence for proving that the BI owner transaction commits before the native messaging service stores ACK. This phase requires a real native gRPC+mTLS path, the production TaskEventReceiver and TaskEventInboxStore, a disposable PostgreSQL database, and an independent observer connection. A source-level harness or successful compile does not establish live network acceptance.

## §1 Change matrix

| Path | Intended change | State |
|---|---|---|
| crates/star-api-rest/tests/bi_task_event_commit_before_ack_pg.rs | Test-only latch immediately before the actual ACK RPC; independent observer checks BI commit and source lease/ACK facts; replay assertion | Implemented; focused build and pure host-guard test pass; independent source review accepts it for local dev integration |
| docs/briefs/9F4J-4-COMMIT-BEFORE-ACK-NETWORK-20261107.md | Scope, scenario, preconditions, guardrails | Updated with exact environment inputs and sender/receiver transport boundary |
| This report | Evidence and remaining gates | Updated to this validation snapshot |
| docs/reviews/9F4J-4-COMMIT-BEFORE-ACK-NETWORK-REVIEW-20261107.md | Independent review record | P1/P2 fixes confirmed; accepted for local dev integration, live acceptance remains open |
| docs/reports/WBS-DISPATCH-9F4J-4-COMMIT-BEFORE-ACK-NETWORK-20261107.jsonl | Dispatch and validation receipts | Focused validation and independent source-review result recorded |

The lane is restricted to its five MAGOS-registered paths. No production code, migration, manifest/lockfile, WBS, main, remote, or user data may be changed.

## §2 Verification summary

| Check | Result | Evidence / limits |
|---|---|---|
| Rust toolchain | PASS | Explicit Rust 1.99.1 used; workspace default 1.99.0 is not used in this environment |
| Focused all-targets compile | NOT RUN | Only the focused integration-test target was built; workspace/all-targets gates are outside this lane |
| Focused test build | PASS | `rtk cargo +1.99.1 test -p star-api-rest --test bi_task_event_commit_before_ack_pg --no-run -j 4`; executable produced |
| Private/loopback host guard test | PASS | `rtk cargo +1.99.1 test -p star-api-rest --test bi_task_event_commit_before_ack_pg disposable_database_host_allows_loopback_and_multipass_private_addresses -j 4`; 1 passed |
| Targeted rustfmt | PASS | `rtk rustfmt --edition 2024 --check crates/star-api-rest/tests/bi_task_event_commit_before_ack_pg.rs` |
| Cypher Graph Guardian | PASS | `rtk python .../cgg_guard.py --changed --stamp`; one Rust source file checked |
| git diff --cached --check | PASS | All five registered paths staged and checked; no whitespace errors |
| PostgreSQL+mTLS commit-before-ACK test | NOT RUN | No designated DB URL or verified certificate set is available in this session; no VM/DB has been started |

## §3 Known gaps

1. The first independent pass found that host validation followed connection establishment and concurrent runs could share the fixed route. Both fixes were confirmed by independent re-review: all host checks precede connection, and a transaction-scoped advisory lock serializes runs sharing the source database.
2. Live PostgreSQL and native gRPC+mTLS acceptance is NOT RUN. The receiver client/server TLS path is exercised by the harness when provisioned; the sender certificate fingerprint authorizes the SQL-seeded Event, but sender-side mTLS append is explicitly outside the assertion.
3. The live run must observe the BI inbox receipt/fact on separate connections while the source delivery is leased and has no ACK receipt, then confirm exact replay preserves the receipt and single fact.
4. The source route must be dedicated and otherwise idle. The test holds a transaction-scoped advisory lock for the fixed route in the source database; environments must point the append fixture and native service at the same source database.
5. PostgreSQL role/grant/RLS, crash recovery beyond this single interruption point, backup/PITR, production identity, workload performance, and deployment readiness remain open.
6. No schema, Cargo manifest, lockfile, production source, or cross-domain acceptance harness change is included in this lane.

## §4 Subagent review and takeover

- The implementation worker is assigned only the registered integration test path. The coordinator owns the four registered documentation paths and will review the resulting test independently.
- Preliminary read-only architecture review found the proof feasible with the real receiver and native transport, using a test-side latch at the ACK boundary and a separate PostgreSQL observer. It also identified missing live DB/certificate prerequisites; no acceptance was executed.
- The independent reviewer confirmed the proof structure, identified the host-ordering and fixed-route concurrency issues, and confirmed both scoped fixes on re-review. The lane is accepted for local dev integration; this does not certify live PostgreSQL/mTLS behavior.
- The source Event is seeded with the existing authenticated database function because the append convenience API does not satisfy the Event-kind boundary. The harness does not claim sender-side mTLS append.
- A separate release-payload audit is tracked in stable-release WBS work and is not part of this lane.

## §5 Guardrails

1. Use the actual native gRPC+mTLS transport and authenticated Event route.
2. Use the production TaskEventReceiver and TaskEventInboxStore commit path.
3. Hold the real ACK RPC immediately before invocation; do not replace it with a fake acknowledgement.
4. Query BI and transport state from an independent observer connection while ACK is held.
5. Prove source remains leased/unacked and no ACK receipt exists before release.
6. Prove source ACK after release and exact replay returns the same durable BI receipt.
7. Assert one receipt and one fact after the duplicate delivery.
8. Keep tenant, sender/receiver, Event kind, route generation, profile pin, and exact-wire identity checks active.
9. Bound the test's waits and in-flight work; honor the minimum lease duration.
10. Secrets and certificate private keys must not appear in logs, reports, or commits.
11. A missing DB or certificate prerequisite must remain NOT_RUN/blocked, not be replaced by an in-memory fake.
12. Compile, fixtures, ignored tests, and static checks do not prove live PostgreSQL+mTLS behavior.
13. Reject public or unresolved database host names before opening any database connection.
14. Serialize test processes that share the fixed source test route; fail before appending when its advisory lock is unavailable.
15. Do not edit files outside the registered five-path manifest.
16. Do not modify W8/Dependabot branches or worktrees, main, remote, or D:\Star user content.
17. Schedule, task execution, accepted settlement, fan-out, and production ingress remain fail closed.

## §6 Sign-off

| Role | Review responsibility | State |
|---|---|---|
| Architecture | Mavis reviews owner/transport boundary and commit-before-ACK ordering | Source review accepted for dev integration; live behavior unverified |
| SRE Lead | Mavis reviews bounded waits, lease handling, and failure reporting | Source review accepted; live environment unavailable |
| Platform | Mavis reviews native gRPC+mTLS and immutable route identity | Source review accepted for harness integration; live handshake not run |
| Review chair | Mavis reviews scope, diff, and test evidence | Independent re-review passed; candidate file check passed |
| PM | Mavis reviews status and WBS traceability | IN PROGRESS; PostgreSQL/mTLS acceptance remains NOT RUN |

Revision author: Ulysses (12-role company per DEC-008) — Mavis review. This is not a claim of human approval or deployment acceptance.

## §7 Revision history

| Version | Date | Reviser | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses (12-role company per DEC-008) — Mavis review | Created the phase record, fixed the registered five-file boundary, specified the real pre-ACK observation sequence, and recorded missing live DB/mTLS prerequisites as NOT RUN | 9F4J.4 lane start |
| v0.2 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis review | Recorded Rust 1.99.1 focused build, pure host-guard test, formatting and graph checks; corrected sender-mTLS scope; added pre-connect database host rejection and fixed-route advisory locking; kept live acceptance NOT RUN pending environment and independent re-review | First review findings and focused validation |
| v0.3 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis review | Recorded independent re-review acceptance for local dev integration, five-path staged whitespace check, and the remaining live PostgreSQL/mTLS acceptance gate | P1/P2 fixes confirmed |
