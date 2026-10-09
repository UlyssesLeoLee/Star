# PHASE-9F4C-D4 — Native transactional currentness

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> Status: 🟡 Directory/binding prerequisite verified; remaining admission/consumer/execution gates open.

## §0 Purpose

Reauthorize the fixed occurrence creator and keep canonical directory/ACL/binding facts current through the same Run-owner admission transaction. Remove the separate-transaction binding read gap. This does not perform complete admission or enable execution.

## §1 Change matrix

| Area | Implementation | Boundary |
|---|---|---|
| Rust admission | `revalidate_directory_and_binding` sets actor GUC, verifies exact active Project/Branch/Run/repository grants and compares the full current binding/digest. | Task/Profile/Hook/quota/Run writer remain separate gates. |
| Native capability | Fixed qualified SQL, SECURITY DEFINER/search_path=pg_catalog, Project advisory lock plus FOR SHARE on all joined canonical rows, PUBLIC execute revoked, Run owner execute only. | Source credential authentication and target deployment function owner review unverified. Runtime gains no canonical directory write. |
| Binding store | New transaction-local locked resolver; standalone resolver delegates and commits its own short read transaction. | No new table, profile/catalog authority or Infrastructure owner reservation. |
| Verification | Extend actual SQLx test with missing/current/paused/revoked/drift and concurrent row-close cases; migration applied twice. | Admin seeding/mutation bypasses lifecycle triggers only in disposable fixture. |
| Documents | Requirements6.17/basic6.14, Schedule SRS0.38/BD-DD0.37, Data2.6, plan6.53, automation2.36/registry0.83. | Historic results preserved, latest scope explicitly partial. |

## §2 Verification summary

| Gate | Actual result |
|---|---|
| Cumulative PostgreSQL18.6/Rust1.99.0 | 36/36 runner steps; SQLx binding/admission targets2/2. Audit `target/phase9f4d-run-owner-consumer/e516c503-832e-40b8-8d5e-a02bebb44f75/audit.jsonl`. |
| Final schema/privilege additions | 34/34; audit `target/phase9f4d-run-owner-consumer/66c2abdb-8bbb-472d-a440-28ce7d2a55db/audit.jsonl`. Checks function fixed search_path/security definer, no PUBLIC execute, Run execute and no runtime directory/ACL mutation grants. |
| Currentness cases | Current authorization passes; absent/revoked Project grant and paused Run reject; binding successor rejects old pinned command. Concurrent ACL/binding closures return SQLSTATE55P03 while admission holds shared locks. |
| Library regression | Rust1.99.0 package12/12. |
| Package strict Clippy | Passed all-targets/no-deps/-D warnings; domain-kms three existing dependency deprecation warnings disclosed. |
| Format/Python/Cypher/diff | Final package rustfmt, py_compile, graph stamp/guard and diff check before commit; no workspace/release/visual/e2e pass claimed. |
| External reviewer | Installed CodeRabbit shim remained Permission denied in the preceding phase; local review required, no independent pass claimed. |
| Resource cleanup | Disposable PostgreSQL stopped/removed, zero NATS and zero Multipass inventory changes. |

First rebuilding the binding target hit Windows LNK1104. No live matching test process was observed; the subsequent isolated rerun passed. No root cause is inferred from this transient file-open error. All failed evidence is retained. Legacy Audit no-op remains disposable-only prerequisite, not production Audit evidence.

## §3 Known gaps

1. Real source service credentials/tenant authorization, target DB/login identity and migration function ownership/read-lock privileges require production verification.
2. Current Task/Profile/HookSet and resource policy checks, quota and atomic Run/reservation/event writer are not yet connected to this transaction.
3. This method is a prerequisite API, not complete admission authority; persistence commit still cannot prove a caller performed every required gate.
4. Schedule/Run producers, result/fence/confirmation consumers, delivery, compensation, cancellation and retry/resume gates remain open.
5. Infrastructure currentness, physical Host/cluster reservations, Multipass/K3s Manager, sandbox/bridge/readiness, Hooks/independent validation, BI/Benchmark and actual task CLI execution remain closed.
6. Overall Group App phases and full workspace/release/visual/runtime acceptance are incomplete.

## §4 Subagent takeover

No subagent dispatched. Root owns this candidate lane; unrelated worktrees/changes are preserved.

## §5 Guard rules

| # | Rule |
|---|---|
| 1 | Navigation remains Project → Branch → Engineering Run → local Worktree. |
| 2 | Run owner writes only its own facts, never Schedule/Infrastructure rows. |
| 3 | Fixed creator/run-as must be checked for each fresh logical attempt. |
| 4 | Trusted service credentials precede injected source construction. |
| 5 | Tenant/actor GUCs must match native capability parameters. |
| 6 | Native capability is static qualified SQL with fixed search_path and no PUBLIC execute. |
| 7 | Project ACL advisory lock precedes current canonical reads. |
| 8 | All joined identity/revision/grant rows stay share-locked until owner commit/rollback. |
| 9 | Run and Branch must be active and exact repository/branch/run scope must match. |
| 10 | Exact V2 binding/digest is held current in the same transaction. |
| 11 | Missing/stale facts reject, DB/integrity errors are never authorization success. |
| 12 | Runtime receives no direct directory/ACL mutation permission. |
| 13 | Task/Profile/Hook/quota/reservation/confirmation/readiness still gate execution. |
| 14 | Tests create zero VM; Multipass inventory cap two/no auto-purge persists. |
| 15 | Selectively fast-forward reviewed content into local dev; preserve existing indexed files and all unrelated lanes. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: native capability and same-owner transaction scope. |
| SRE Lead | Mavis review: 36/36 plus34/34 isolated checks; production identity/ownership open. |
| Platform | Mavis review: lock/statement bounds inherited from admission; no runtime directory mutation. |
| Review chair | Mavis review: real lock/revocation/drift cases; no independent tool pass. |
| PM | Mavis review: synchronize documents and selectively integrate; complete execution remains unproven. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Verify same-transaction native directory/binding prerequisite and preserve remaining gates. | Current binding resolver previously opened another transaction. |
