# PHASE-9F4C-D3 — Run-owned admission persistence report

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> Status: 🟡 Persistence primitive verified; production admission/consumer/execution gates open.

## §0 Purpose

Persist processed Schedule commands and Run admission results within one Run-owner transaction, allowing exact transport recovery before creating new Run facts. Preserve Project → Branch → Engineering Run → local Worktree navigation and independent Schedule/Infrastructure owners. This phase adds no executable task CLI path or production consumer.

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Transaction data | Add `multica.schedule_admission_inbox` and `multica.schedule_admission_result_outbox`; bounded exact wire, request/attempt/result identities, service source and fixed run-as, paired FK, append-only/FORCE RLS. | Work 0 / Transaction 2 / Master 0; no TTL or physical deletion. |
| Rust API | `PostgresScheduleAdmissionStore.begin` returns exact Replay or a new locked `ScheduleAdmissionTransaction`; `commit_result` pairs immutable receipt/result with caller's Run-owned transaction. | No authorization, quota/reservation, Run writer or executor implementation is claimed. |
| Scope guard | Accepted result must match Task/Worktree/run-as/rule/occurrence/attempt/correlation and a non-started Run with no start/execution-state event. | Pre-existing Run is seeded by disposable admin solely for this storage test. |
| Performance | Decode max 16 KiB before copying, transaction request lock, lock timeout 5s/statement timeout 10s; no per-message spawned loop or unbounded buffer. | Production pool, queue and fairness configuration remains a composition gate. |
| Dependencies | Add existing workspace `domain-automation` path dependency; Cargo.lock changes one package dependency entry. | No new external dependency/license family or Pi/Node runtime is introduced. |
| Documentation | Requirements v6.16, basic v6.13, Schedule SRS/BD/DD v0.37/v0.36/v0.36, Data Design v2.5, plan v6.52, automation v2.35 and registry v0.82. | Historic evidence preserved; current primitive/production boundaries distinguished. |

## §2 Verification summary

| Gate | Result / evidence |
|---|---|
| Cumulative PostgreSQL18.6/Rust1.99.0 runner | 33/33 passed; audit `target/phase9f4d-run-owner-consumer/40a8dc35-d9a2-4a76-a944-bd39474d51ef/audit.jsonl`. D3 migration applies twice; role/grants/RLS/paired-FK/WORM checks included. |
| New admission SQLx target | 1/1 passed, 0.52s. Actual restricted LOGIN/role activation, tenant/service scope, changed exact-byte replay, concurrent new/replay serialization, drop recovery, receipt/result rollback, orphan commit refusal, duplicate attempt, accepted provisional fixture, prior-start denial and WORM covered. |
| Existing binding SQLx target | 1/1 passed in the cumulative runner. |
| Rust library regression | 12/12 passed using pinned Rust1.99.0; focused package only. |
| Package strict Clippy | Passed `--all-targets --no-deps -D warnings`; dependency domain-kms retains three pre-existing deprecation warnings. Broader strict dependency lint findings from previous phases remain open. |
| Formatting/Python/Cypher/diff | Passed package rustfmt, Python py_compile, Cypher --changed --stamp then --changed (4 source files), and git diff --check; no claim of full workspace/release/e2e verification. |
| Independent review | CodeRabbit remains unavailable through the installed shim (`Permission denied` in this phase review attempt); local source/diff review performed. |
| Resources | Owned PostgreSQL stopped/removed, no NATS, zero Multipass VM/inventory changes. Test inventory cap remains two records, including Stopped/Deleted; no auto-purge. |

The first cumulative attempt correctly exposed an obsolete assertion that denied the newly required start/attempt read columns. The assertion now verifies narrow read columns while retaining no task snapshot access or Run/RunEvent writes. A fixture helper extraction caused a Rust reference mismatch and was fixed. Later Windows LNK1104 prevented rebuilding test executables; no live matching test process was found, and the final rerun above passed. Root cause of that transient file-open failure is not proven. Failed logs are retained; they are not counted as passing. The legacy no-op Audit function remains disposable-only prerequisite evidence, not a production Audit implementation.

## §3 Known gaps

1. Real Schedule service credential authentication, tenant authorization and source-principal activation are not composed. `ScheduleAdmissionSource` is injected scope, not proof of authentication.
2. Current fixed creator/run-as ACL, Task/Profile/HookSet/binding validation, Project/Run quota and atomic Run/reservation/events writer still need to use the new locked transaction.
3. Producer, bounded Outbox delivery Work state, polling, broker ACK, Schedule result/fence consumer, Run confirmation, retry/compensation and cancellation are not connected. Replay of a historical admission result never authorizes current execution/resume.
4. Target DB migration identity, real LOGIN provisioning, current Run/Infrastructure sources, paired physical Host/cluster reservations and deployment verification remain open.
5. Multipass/K3s Manager, Runtime sandbox/workspace bridge/readiness, native Hooks/independent verification, CLI execution/output/return path, BI/Benchmark, loop worker and all other Group App phase gates remain open/fail-closed.
6. Package/focused tests are not complete workspace/release/visual/runtime acceptance. CodeRabbit was not completed.

## §4 Subagent failure takeover

No subagent was dispatched in this slice. Root owns the candidate lane. No unrelated branch/worktree content was imported or removed.

## §5 Guard rules

| # | Rule |
|---|---|
| 1 | Navigation remains Project → Branch → Engineering Run → Worktree; App tabs belong to Run and CLI originates in Task Card. |
| 2 | Inbox/result are Run-owned Transaction facts, never Schedule table mutations. |
| 3 | Real service authentication must precede source scope construction in production. |
| 4 | Fixed run-as is separately reauthorized for each fresh logical attempt. |
| 5 | Request lock precedes all new owner facts; different logical attempts use new request identity. |
| 6 | Exact bytes and original service principal identify transport replay; changed wire/source conflicts. |
| 7 | Historical replay returns historical result only and grants no executor/resume permission. |
| 8 | Deferred result pairing and occurrence/attempt uniqueness must hold at commit. |
| 9 | Receipt/result wire is at most 16 KiB; queues and production pools require bounded composition. |
| 10 | Accepted storage references matching Run with no actual start evidence; fixture is not production admission. |
| 11 | Current authorization, policies, quotas and Run-owned reservation must be checked within the locked transaction. |
| 12 | Infrastructure owner separately reserves Host and exact cluster; no distributed write transaction. |
| 13 | Multipass inventory cap two and no auto-purge remain; this phase creates zero VM. |
| 14 | Missing auth/reservation/confirmation/readiness/Hook/verification provider keeps execution closed. |
| 15 | Selectively fast-forward only reviewed commits to local dev; preserve unrelated changes, indexed files and worktrees. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: owner-local receipt/result transaction and strict production boundary. |
| SRE Lead | Mavis review: disposable PostgreSQL33/33 and owned-resource cleanup; target production identity unverified. |
| Platform | Mavis review: bounded payload/lock/statement time; Rust1.99.0 package gates; production composition open. |
| Review chair | Mavis review: concurrent replay, rollback, source scope and prior-start tests; independent tool unavailable. |
| PM | Mavis review: deliver D3 primitive and synced documents selectively to local dev; overall phases remain incomplete. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Record owner-local admission persistence,33/33 evidence, boundaries and open phases. | Next approved Schedule → Run owner implementation step. |
