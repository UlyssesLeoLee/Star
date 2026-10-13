# PHASE-9F4D-3D5-RUN-OWNER-CONSUMER-REPORT

## 0. Purpose

Implement and verify the Run/Execution-owner side of the Infrastructure capacity event boundary: a typed demand Outbox append primitive and a bounded, durable-JetStream result consumer that writes only Run-owned immutable facts. This report closes the 9F4D-3D5 implementation slice only. It does not claim a production Schedule worker, provider, executor, or the overall Worktree-first architecture is complete.

## 1. Change and acceptance matrix

| Area | Change | Evidence and result |
|---|---|---|
| Demand Outbox | Added `PostgresRunOwnerCapacityStore::append_capacity_demand` for a caller-owned Run transaction. Before insert, it validates the typed demand and verifies that TaskExecutionRun is schedule-origin with matching rule ID/revision, occurrence ID, and correlation ID. | Disposable PG/NATS integration rejects an occurrence mismatch without an Outbox insert; insert and exact replay pass. |
| Result consumer | Added a Rust adapter around a caller-supplied durable explicit-ack PullConsumer. It requests one message, sets async-nats `max_bytes=132 KiB` as a client-side buffering target, and uses a 2-second expiry. | Unit tests cover one-message/time bounds and reject parser payloads above 128 KiB after receipt; the real JetStream test covers message identity, stored-demand scope, persistence and ACK ordering. These tests did not prove broker/stream ingress limits or peak RSS. |
| Owner persistence | Added append-only demand Outbox and result Inbox Transaction facts under tenant FORCE RLS and the least-privilege `star_multica_run_owner` role. Result scope is hydrated from the exact stored demand. | Migration applied twice; schema, role/privilege matrix, RLS isolation, and cross-tenant global `event_id` collision checks passed on disposable PostgreSQL. |
| Commit and acknowledgement | The Run-owner transaction commits before JetStream `double_ack()`. Exact `event_id` replay is acknowledged only after immutable scope/content equality; store failure leaves the delivery unacknowledged. | Unit and real JetStream/PostgreSQL integration checks passed, including rollback followed by redelivery. |
| Owner boundary | No TaskExecutionRun/Schedule projection, BI/RunEvent, reservation mutation, readiness claim, or executor launch was added. | Capacity acceptance remains a fact only; production execution stays disabled. |
| Docs and automation | Synchronized requirements, basic design, Schedule SRS/BD/DD, Group DD, Infrastructure DD, Data Design W/T/M, implementation plan, automation design and registry. Added an exact-anchor idempotent sync script. | Two consecutive sync runs reported `changed_files=0`; requirement and design text now includes the TaskExecutionRun identity check. |
| Multipass disk limit | This verification used disposable local PostgreSQL and NATS only. | Runner reported `multipass_instances_created=0` and `multipass_inventory_modified=false`; the registered-instance cap remains two, including stopped/deleted records. |

## 2. Verification summary

| Command / gate | Result |
|---|---|
| `cargo fmt -p run-owner-runtime -- --check` (Rust 1.99.0) | Passed. |
| `cargo clippy --offline --locked -p run-owner-runtime --all-targets -j 1 -- -D warnings` | Passed. |
| `cargo test --offline --locked -p run-owner-runtime --lib -j 1` | 9/9 passed. |
| `cargo test --offline --locked -p run-owner-runtime --test capacity_result_jetstream_pg --no-run -j 1` | Passed; integration test target linked. |
| Disposable PostgreSQL 18.6 + NATS Server 2.15.0 runner | 20/20 runner steps passed; the real JetStream/PostgreSQL integration test passed 1/1. Migrations were applied twice, RLS and owner privileges were checked, tenant isolation and the expected cross-tenant `event_id` unique conflict were verified, and both owned services stopped cleanly. |
| `python -m py_compile scripts/automation/phase9f4d_run_owner_consumer.py scripts/automation/phase9f4d3d5_sync.py` | Passed. |
| Documentation sync script | Two runs passed with zero changed files each. |
| `python scripts/automation/registry_check.py` | Exit 0; 238 scripts scanned, 0 errors, 197 existing/unregistered-script warnings. Both new Phase 9F4D-3D5 scripts are listed as `[OK]`. |
| `cargo fmt --all -- --check` | Not clean because unrelated workspace files have existing formatting differences under Rust 1.99.0; those files were not mass-formatted. The new crate's formatting gate is clean. |
| Workspace-wide `cargo check` / test matrix | Not run for this bounded slice. The verified scope is the new crate plus the real disposable PostgreSQL/NATS runner. |
| CodeRabbit | Not run: the installed `coderabbit.cmd` shim resolved to `/root/.local/bin/coderabbit` and exited 126 with `Permission denied`. This is an unavailable review result, not a clean CodeRabbit review. |
| `rtk` command wrapper | Not resolvable in this environment; Cargo was invoked by its explicit Rust 1.99.0 toolchain path. |

One intermediate runner attempt encountered MSVC `LNK1104` opening its test executable. The integration target then linked successfully with `--no-run`, and the final full runner passed. The initial linker failure is retained here rather than treated as a code-test result.

Final runner audit: `target/phase9f4d-run-owner-consumer/root-final-20261007-3/audit.jsonl`.

## 3. Known gaps

- No Schedule admission writer currently calls the new demand append API. The append primitive is verified; it is not a production Schedule workflow.
- No application login/membership provisioning or target-database deployment was verified. PostgreSQL evidence is disposable and local.
- No production JetStream stream/durable provisioning, credentials, tenant poller, or worker lifecycle is implemented.
- Malformed, unmatched, or conflicting events remain unacknowledged unless an eligible permanent poison fault is explicitly routed to the separate Platform Messaging quarantine owner, which commits a trusted-source receipt before confirmed Term ACK. The quarantine hook exists, but trusted production consumer provisioning, size enforcement, KMS/operator controls, and peak-RSS evidence are not complete; the consumer is not production deployable.
- The Infrastructure `capacity_released` producer contract is unavailable and remains fail-closed.
- No result projection updates TaskExecutionRun, Schedule, BI/RunEvent, or reservations. A stored `capacity_accepted` event does not prove current reservations, Multipass/K3s readiness, or Agent Runtime sandbox readiness.
- No native Multipass manager/provider, RunInfrastructureBinding currentness revalidation, independent Runtime sandbox, cancellation/recovery worker, or task executor is closed by this phase.
- No workspace-wide build/test gate, RSS benchmark, failure-load benchmark, or production environment gate was run.

## 4. Subagent failure and takeover

- `jetstream_consumer_audit` supplied a read-only review of the publisher contract, async-nats 0.50 consumer behavior, ACK ordering, bounded fetch design, and failure-test recommendations.
- `run_owner_consumer` delivered the Rust crate, workspace dependency changes, tests, and CypherGraph header update. It did not run Cargo after handoff; the root agent ran the final Rust gates.
- `run_owner_schema_runner` delivered the migration and disposable runner. Its first combined PostgreSQL/NATS integration attempt failed with `InvalidStoredDemand`; the root agent retained that failure as non-passing evidence and reran the final source against the complete runner. Final result: 20/20 runner steps and 1/1 real integration test passed.
- No worker commit was imported. The root agent reviewed and verified the integration-lane changes before the targeted local-dev merge.

## 5. Guard rules

1. Rust 1.99.0 remains the selected workspace minimum; keep Cargo, toolchain, CI, and dependency requirements aligned.
2. Project, cloud Branch, Engineering Run, TaskExecutionRun, occurrence, and Work Item remain distinct identities. Validate the scheduled demand against its TaskExecutionRun inside the caller-owned transaction.
3. Run and Infrastructure owners write only their own facts. Do not share a cross-owner transaction or write another owner's tables.
4. Keep all event facts append-only, tenant-scoped, FORCE-RLS protected, and least-privilege. Preserve immutable replay equality and the global result `event_id` constraint.
5. Request one message, set `max_bytes=132 KiB` for async-nats client buffering, and use a 2-second expiry. Do not treat `max_bytes` or the post-receive parser guard as ingress enforcement. Before production, enforce and verify NATS `max_payload` and JetStream Stream `MaxMsgSize` at no more than 128 KiB and measure peak consumer RSS. Commit the Run-owner transaction before `double_ack()`; database or validation failure must not ACK.
6. Do not TERM or silently discard poison messages without durable quarantine. Keep release unsupported until its producer contract exists.
7. Do not infer Schedule/TaskExecutionRun readiness, reservation validity, BI state, or execution authorization from a stored capacity result.
8. Keep production deployment and Schedule/Task Card execution fail-closed until consumer identity/provisioning, quarantine, current authorization/binding checks, both owner reservations, provider readiness, and the independent Agent Runtime sandbox are verified.
9. Test with at most two registered Multipass inventory records total, including stopped/deleted entries. This phase created zero VMs and never purges inventory.
10. Run the idempotent doc sync twice and preserve the requirements → basic design → detailed design → implementation plan trace.

## 6. Sign-off

| Role | Review |
|---|---|
| Architect | Mavis review — demand identity check, owner boundary, and result Inbox scope reviewed. |
| SRE Lead | Mavis review — disposable PostgreSQL/NATS lifecycle and failure/redelivery evidence recorded; production broker remains open. |
| Platform | Mavis review — Rust 1.99.0, one-message resource bounds, and zero-VM verification recorded. |
| Review host | Mavis review — focused format, Clippy, unit, integration, migration/RLS, and documentation gates reviewed; workspace format limitation and unavailable CodeRabbit disclosed. |
| PM | Mavis review — 9F4D-3D5 is a verified Run-owner consumer slice, not production execution readiness or overall architecture completion. |

## 7. Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses — Mavis review | Record the Run-owner demand/result facts, caller-transaction TaskExecutionRun validation, bounded JetStream consumer, final PG/NATS evidence, documentation synchronization, production gaps, and the two-record Multipass limit. | Phase 9F4D-3D5 implementation and final verification. |
| v0.2 | 2026-10-13 | Ulysses — Mavis接手审核 | Clarify client-side `max_bytes` buffering semantics, the post-materialization 128 KiB parser guard, and the separate server/stream admission and peak-RSS gates | Review Run-owner consumer memory claims against current async-nats and NATS documentation |
