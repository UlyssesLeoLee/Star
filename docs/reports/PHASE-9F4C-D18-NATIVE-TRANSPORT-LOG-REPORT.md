# PHASE-9F4C-D18 — Native transport durable-log storage report

## §0 Purpose

Record the D18 PostgreSQL storage foundation for Star's future Rust-owned message transport. D18 establishes a bounded, tenant-scoped durable-log contract only. It does not deliver a Rust gRPC service or replace the existing NATS provider.

## §1 Change and acceptance matrix

| Area | Change | Result |
|---|---|---|
| PostgreSQL schema | Added `platform_transport` Master, Work, receipt and Outbox tables, fixed-path append/pull/ACK/purge routines, FORCE RLS, WORM guards and least-privilege roles. | Implemented; migration applied twice in disposable PostgreSQL. |
| Exact idempotency | Serialized the producer key across profile/log generations; exact digest+log+generation replay returns the original cursor; conflict fails closed. | Sequential and concurrent SQL assertions passed. |
| Bounded resources | Raw wire ≤16 KiB; bounded payload budget, lease duration, attempts and purge batch; TTL removes terminal payload only. | SQL contract passed, including explicit NULL purge-limit rejection. |
| Durable facts | Accepted/acked receipts are append-only and permanent; receipt Outbox is metadata-only and emitted in the same transaction. | WORM, receipt conservation and Outbox assertions passed. |
| Parallel safety | Runner starts two independent PostgreSQL client sessions with the same producer identity and exact wire. | Both calls succeeded; one allocated sequence 1, the other returned exact replay. Final state contained one Work row, one accepted receipt, one Outbox fact and one cursor/byte accounting update. |
| Design synchronization | Updated requirements, basic design, Schedule SRS/DD, data design/W-T-M, automation design, registry, reuse review and master implementation plan. | Updated to identify D18 as storage-only and D19+ as future provider/service work. |
| Source reuse | Kept E:/AssetsLake and other local Rust servers eligible for direct module reference/adaptation/reuse subject to per-file review. | No outside source copied or modified. |

## §2 Verification summary

- `py -3 -m py_compile scripts/automation/phase9f4d_run_owner_consumer.py`: passed.
- Disposable PostgreSQL 18.6 schema-only runner: **77/77 steps passed**, including both applications of `2026-10-12-platform-native-transport-log.sql`, full D18 contract, two concurrent appends and postcondition SQL.
- PostgreSQL stop: passed. Multipass inventory: unchanged; no VM created. NATS: not started.
- Evidence: `target/phase9f4d-run-owner-consumer/245a8441-2924-4a7f-806b-1237215bec0c/audit.jsonl`.
- `git diff --check`: passed. `scripts/automation/registry_check.py`: 0 errors; 197 pre-existing warnings for other scripts without registry entries.
- CypherGraph Guardian passed for the changed Python source. Its built-in source-extension set excludes SQL, so the three SQL Cypher blocks were stamped and individually passed the same validator functions; source graph semantics were reconciled manually.
- CodeRabbit was unavailable: its CLI wrapper resolved to `/root/.local/bin/coderabbit` and failed with `Permission denied`; no diff was sent to the external reviewer. Direct local review and runtime verification were completed.
- The runner checked Cargo's version only. No Rust compilation, Rust test, Rust 1.99.0 gate, target database identity, or production deployment was executed.
- CodeGraph was queried for the existing runner source. The environment did not expose `index_repository` or `check_index_coverage`; `.codegraph/` was preserved. Graph coverage is therefore not claimed.

## §3 Known gaps

1. Rust-native gRPC server/client, service mTLS, production identity verifier and credential lifecycle are not implemented. `app.platform_*` session GUC values are caller context only and cannot authenticate a sender; production DB grants must restrict the runtime role to the verified Rust server workload.
2. Transport Profile to durable-log provisioning, production database grants/role mapping, receiver consumer, BI/Benchmark projection, cleanup scheduling and operational observability remain open.
3. The future transport must compute SHA-256 over exact wire bytes. The SQL boundary validates digest length only; it cannot prove a caller-supplied hash matches payload bytes.
4. Only a two-session exact-duplicate case was exercised. Throughput, contention fairness, RSS, large fan-out, crash recovery, disk-full, network partition, restore and operational failover are not measured.
5. NATS provider dependencies and current consumers remain. D18 does not implement provider cutover or NATS retirement.
6. Schedule/Run task execution, native Hook/runtime/sandbox readiness and independent task verification remain fail closed.
7. Installed Rust 1.99.0 was unavailable in this environment; the D18 runner did not compile Rust. Disposable PostgreSQL evidence is not production target DB evidence.
8. E:/AssetsLake and other local source projects were not modified; this stage did not recopy or re-audit all source modules. Per-file source hash, rights/license/NOTICE and dependency-closure review still applies to any reuse.

## §4 Subagent failure and takeover

No subagent was dispatched for D18. There is no worker RPC failure, uncommitted child patch, or lost worktree output to take over. Existing source projects remained read-only.

## §5 Required gates and result

| # | Gate | D18 result |
|---:|---|---|
| 1 | Use isolated worktree and retain existing user data/untracked index. | Pass; `.codegraph/` preserved. |
| 2 | Apply the migration twice without error. | Pass. |
| 3 | Owner/runtime roles have no login, superuser, role-create, replication or RLS-bypass capability. | Pass. |
| 4 | SECURITY DEFINER routines use fixed search paths and deny PUBLIC execution. | Pass. |
| 5 | Every transport table enables and forces RLS. | Pass. |
| 6 | Runtime has no direct payload/table DML grants. | Pass. |
| 7 | Deployment and tenant scopes fence route and payload access. | Pass. |
| 8 | Receipt Outbox contains metadata and no raw wire or credential. | Pass. |
| 9 | Wire and hash have strict 1..16 KiB and 32-byte bounds. | Pass. |
| 10 | Trusted-service SHA-256 computation is documented as a future service responsibility. | Pass in design; service not yet implemented. |
| 11 | Exact replay returns original cursor; conflicting digest rejects. | Pass. |
| 12 | Two concurrent sessions with one exact producer identity allocate one receipt/work/outbox/cursor. | Pass. |
| 13 | Pull returns one item with bounded lease and uses `SKIP LOCKED`. | Pass. |
| 14 | Attempt cap, expiry re-lease and stale ACK fence are enforced. | Pass. |
| 15 | Purge rejects NULL/out-of-range limit and removes only TTL-expired terminal wire. | Pass. |
| 16 | Permanent receipt/outbox facts survive payload purge and preserve exact replay. | Pass. |
| 17 | Owned PostgreSQL process stops and temporary resources are cleaned; no Multipass inventory mutation. | Pass. |

## §6 Sign-off roles

| Role | Review |
|---|---|
| Architecture | Mavis: storage ownership stays within Platform Messaging; provider cursor remains separate from Run/business identity. |
| SRE Lead | Mavis: disposable PostgreSQL stop passed; production capacity, disaster recovery and load gates remain open. |
| Platform | Mavis: fixed-path capabilities, RLS, append-only facts and concurrent exact idempotency verified in the isolated database. |
| Review Chair | Mavis: documents distinguish storage acceptance from an implemented Rust service or NATS replacement. |
| PM | Mavis: D18 closes this bounded storage slice only; D19–D21 remain planned and overall Worktree-first phases continue. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-12 | Ulysses — Mavis 接手审核 | Record D18 durable transport log schema, parallel exact-idempotency verification, synchronized design and remaining production gates. | D18 PostgreSQL 18.6 runner passed 77/77. |
