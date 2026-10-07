# PHASE-9F4D-3D6-QUARANTINE-IMPLEMENTATION-REPORT

> Status: 🟡 Platform quarantine schema and opt-in Run-owner hook implemented and verified; production consumer and Schedule execution remain disabled.
> Date: 2026-10-08
> Pre-merge base: local `dev` at `5b7f86968117c6ab75435fe042df0ef1e7e16a6c`
> Source commit: `b421c2951f9d13771354206f46080b6bd853da3e`
> Local `dev` integration: no-FF merge `79574a1b5aaa97f97656b9e5939436adcd20647a`; clean after merge, no push, `main` unchanged.
> Lane: `UlyssesLeoLee/phase9f4d3d6-quarantine`

## 0. Purpose

Implement the 9F4D-3D6 quarantine substrate for permanently malformed capacity-result messages. Keep untrusted message scope out of Run-owner RLS, persist the quarantine receipt and any selected encrypted Work in a separate Platform owner, and allow terminal acknowledgement only after the owner transaction commits. This report covers a verified substrate, not production consumer enablement or an end-to-end Schedule execution loop.

## 1. Change matrix

| Area | Implementation | Status |
|---|---|---|
| Rust Platform owner | Added `platform-messaging-runtime`: trusted JetStream source identity, exact wire digest, explicit metadata-only/encrypted-24h policy, immutable receipt, optional encrypted replay Work, scoped DB sessions, idempotency checks and bounded audited purge. Encrypted replay copies no more than 16 `Nats-Msg-Id` values or 8 KiB total. | 6/6 platform tests passed. |
| Run-owner hook | `RunOwnerCapacityResultConsumer::with_quarantine` routes only invalid event, malformed JSON and event-id conflict; after quarantine commit it sends confirmed `double_ack_with(AckKind::Term)`. Unknown/owner-data/transient errors stay unacknowledged. | 10/10 Run-owner tests passed. |
| Database schema | Added receipt, payload Work and purge-audit tables with separate NOLOGIN roles, FORCE RLS, append-only/retention guards, per-item 200 KiB cap, per-deployment 4 MiB quota trigger and 24-hour TTL. | PostgreSQL 18.6 double-apply and schema/quota tests passed. |
| Design synchronization | Updated main requirements/basic design, Schedule SRS/basic/detailed design, Group DD, Data Design W/T/M, implementation plan and phase report. | Version pointers aligned; design-only report preserved as historical. |
| Multipass / Rust | Preserved the Multipass Linux-guest/K3s architecture, the two-record test inventory cap including Stopped/Deleted records, and workspace Rust 1.99.0. This phase started zero VMs. | No inventory/lifecycle command was used. |

## 2. Verification summary

- Rust toolchain: `rustc 1.99.0`, `cargo 1.99.0` via the repository-pinned `rust-toolchain.toml`.
- `cargo fmt --manifest-path Cargo.toml --package platform-messaging-runtime --package run-owner-runtime -- --check`: passed.
- `cargo check -p platform-messaging-runtime -p run-owner-runtime --lib -j 4 --offline --locked`: passed. It reports three existing `domain-kms` deprecation warnings for `Nonce::from_slice`.
- `cargo test -p platform-messaging-runtime -p run-owner-runtime --lib -j 4 --offline --locked`: passed, 6/6 + 10/10 tests.
- Targeted `cargo clippy -p platform-messaging-runtime -p run-owner-runtime --lib -j 4 --offline --locked --no-deps -- -D warnings`: passed; same dependency deprecation warnings are outside the changed crates.
- Local `dev` integration used a no-FF merge from the recorded source commit; conflict prediction and merge succeeded, and `git diff --check HEAD^1 HEAD` passed. Post-merge Rust gates could not be rerun in the current environment because `cargo` is unavailable in PowerShell and the Ubuntu WSL distribution; the Rust results above are pre-merge source-lane evidence, not post-merge evidence. The `dev` worktree is clean and is 40 commits ahead of `origin/dev`; nothing was pushed and `main` was not changed.
- PostgreSQL 18.6 disposable loopback cluster on port 55312: the migration applied twice; `db/tests/platform-poison-quarantine-schema.sql` passed FORCE RLS, least privilege, WORM, metadata-only, TTL/purge audit and quota-overflow assertions. This used synthetic ciphertext bytes and is not evidence of real KMS encryption.
- PostgreSQL server was stopped and `pg_ctl status` confirmed no server running. Its 42 MiB owned data directory remains under this lane's `target/phase9f4d3d6-pg-3c11e76e8c18415dace3c215435dbc00/data`; recursive cleanup was rejected by the environment policy, so no bypass was attempted.
- No Multipass VM, NATS server, target database, or Docker Desktop was used for this slice. NATS redelivery was not exercised in this turn.

## 3. Known gaps

1. The deployment must choose `metadata_only` or `encrypted_24h`; the service has no implicit default.
2. The current KMS implementation is LocalMock-only. Encrypted retention fails closed for production until a real KMS provider and key lifecycle are available.
3. Production DB login membership, role activation, migration provisioning and grants are not configured or verified.
4. Trusted deployment identity, stream generation and durable-consumer provisioning remain deployment-owned configuration and are not wired to a production consumer.
5. Full NATS redelivery, ACK loss, database/key/quota outage integration, and consumer lifecycle were not exercised.
6. Operator inspection/replay UI and access audit are not implemented; quarantine payload is not exposed to Run Apps, Task Card CLI, Agent, Plugin, or Canvas.
7. `InvalidStoredDemand` remains unacknowledged; only three stable permanent fault codes enter quarantine.
8. No target database rollout, performance/RSS benchmark, BI/Benchmark projection, provider readiness, task execution, or end-to-end Schedule acceptance was performed.
9. The local 42 MiB PostgreSQL fixture is stopped but retained because recursive cleanup was blocked; this directory is not part of Multipass inventory.

## 4. Subagent results and takeover

- No subagent was dispatched for this implementation slice. The earlier read-only design audit remains a source of architecture input; no subagent edits or commits were imported.
- The first SQL quota attempt exposed a test fixture omission (`expires_at`); the fixture now supplies the exact 24-hour window and the full schema/quota test passes.
- The first psql connection attempts used incorrect roles. The isolated cluster's `quarantine_test_admin` role was identified through its single-user mode, after which the migration and test were rerun successfully. The live PostgreSQL service on port 5555 was not touched.

## 5. Guard rules

1. Never derive Platform quarantine authority or RLS scope from message tenant, Run, event, subject or `Nats-Msg-Id` claims.
2. Use configured deployment identity and stream generation plus server stream name/sequence as the immutable receipt identity.
3. Keep receipt and purge audit append-only Transaction facts; keep encrypted bytes in separate bounded Work.
4. Require explicit retention configuration; do not infer or default metadata-only versus encrypted 24-hour storage.
5. Enforce the 200 KiB item cap and 4 MiB per-deployment encrypted payload quota in both Rust admission and PostgreSQL.
6. Count expired Work against quota until its purge audit and delete commit atomically.
7. Use separate NOLOGIN ingress and purger roles with FORCE RLS and least privilege.
8. Do not expose Multipass daemon/socket, host shell, or host mounts to this owner, Agent, Plugin, Task Card CLI or Schedule worker.
9. Term-ack only after the Platform receipt and selected payload action commit, using a confirmed `double_ack_with(AckKind::Term)`.
10. Preserve idempotent identical replay; a trusted-key digest/fault conflict stays unacknowledged and raises an integrity signal.
11. Keep `InvalidStoredDemand`, unknown classification, DB/KMS/quota errors and other transient errors unacknowledged.
12. Never store malformed capacity results in Run-owner Inbox or write Run/Infrastructure-owned facts from the Platform transaction.
13. Do not mutate TaskExecutionRun, Schedule, RunEvent/BI, reservations, readiness or execution from quarantine.
14. Do not claim schema fixtures with synthetic ciphertext prove KMS encryption or production key management.
15. Preserve the test-stage maximum of two registered Multipass records including Stopped/Deleted; DB/NATS contract tests start zero VMs.
16. Use Rust 1.99.0 for focused Cargo gates; retain `-j 4 --offline --locked` and report dependency warnings precisely.
17. Bound encrypted replay headers by both count (16) and aggregate `Nats-Msg-Id` bytes (8 KiB), so empty duplicate values cannot bypass the memory cap.
18. Keep production consumer and Task CLI disabled until identity/provisioning, KMS, redelivery, operator audit and independent execution gates pass.

## 6. Review roles

| Role | Review |
|---|---|
| Architect | Mavis review — owner split, trusted identity and commit-before-Term preserved. |
| SRE Lead | Mavis review — quota, TTL, replay/backpressure and purge bounds verified in the disposable schema gate; production monitoring remains open. |
| Platform | Mavis review — Multipass two-record cap and Rust 1.99.0 baseline retained; no VM started. |
| Review host | Mavis review — focused Rust, SQL and cross-document status checked; production limitations stated. |
| PM | Mavis review — phase remains partial until production wiring and execution gates close. |

## 7. Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record Platform quarantine schema/runtime implementation, database-enforced quota, 16-value/8-KiB replay-header bound, confirmed commit-before-Term hook, focused Rust/PostgreSQL evidence, local `dev` merge identity, post-merge Cargo limitation, remaining production gates and the retained stopped disposable data directory. | Phase 9F4D-3D6 implementation, bounded-memory review and subsequent local `dev` convergence. |
