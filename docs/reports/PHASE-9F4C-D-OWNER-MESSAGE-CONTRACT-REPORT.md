# PHASE-9F4C-D Owner Message Contract Report

> Status: 🟢 Domain contract slice verified; local `dev` integration pending at report creation.
> Version: 0.1 | Date: 2026-10-09
> Scope: Schedule→Run owner-message DTO contract only. This is not completion of Phase 9F4C-D consumers or production Schedule execution.

## §0 Purpose

Freeze the bounded, versioned message contract needed to connect Schedule and Run/Execution through owner-local Outbox/Inbox transactions. The contract follows this conversation's Project → Cloud Branch → Engineering Run → Worktree architecture and the two-owner transaction boundary. The scope adds no shared transaction and does not enable a worker, consumer or executor.

## §1 Change matrix

| Area | Change | State |
|---|---|---|
| Rust domain contract | Add strict v1 Schedule Admission Command, Result, Resolution, typed outcomes/rejection/compensation reasons, identity/scope validators, canonical infrastructure-binding digest match and 16 KiB encode/decode bound. | Implemented |
| Idempotency | Distinguish one logical attempt (`request_id` + positive attempt ordinal) from transport redelivery; occurrence ID groups attempts. A new logical attempt uses a new ID after terminal resolution; transport redelivery reuses persisted payload bytes exactly. | Contract specified; Outbox persistence not implemented |
| Trust boundary | Treat envelope `tenant_id` as a scope claim only. Receiving owner authenticates service identity out of band and establishes tenant/RLS scope independently. Schedule lease owner/generation/expiry remain Schedule-owned state and are not copied into the replay payload. | Contract/design specified; production identity mapping not implemented |
| Branch intake | Rechecked local branch heads and the relevant active lanes. No committed unique candidate commit was found. The dirty 9F4D consumer lane was left intact; no whole-branch overlay or large refactor was copied. | Reviewed; no external lane content integrated |
| Documentation | Updated requirements v6.11, basic design v6.08, Schedule SRS v0.32, BD v0.31, DD v0.31, automation design v2.30, registry v0.77, and plan v6.45. | Synchronized |

## §2 Verification summary

- Runner: `scripts/automation/phase9f4c_d_owner_protocol.py`
- Run ID: `20261008T192724Z-80898675`
- Evidence directory: `target/phase9f4c-d-owner-protocol/20261008T192724Z-80898675/`
- Toolchain: Rust 1.99.0; direct Cargo executable used because `rtk` was unavailable on PATH.

| Gate | Result |
|---|---|
| Python runner syntax (`python -m py_compile`) | Pass |
| `cargo fmt --check` for `domain-automation` | Pass |
| `cargo check --all-targets -j 4` for `domain-automation` | Pass |
| Focused admission contract tests | 3/3 pass |
| Full `domain-automation` package tests | 29/29 pass |
| Focused Clippy | Pass with four named pre-existing lint categories allowed: `derivable_impls`, `unnecessary_map_or`, `unnecessary_sort_by`, `bool_assert_comparison` |
| `git diff --check` | Pass |
| Runner total | 6/6 gates pass |

An initial test-link attempt using a shared Cargo target failed with MSVC `LNK1104`. The complete runner passed after using the lane-unique short target `E:\DevCache\phase9f4c-d-owner-contract`. No workspace-wide build/test, database, broker, VM or Multipass operation was run as part of this slice.

## §3 Known gaps

- No Schedule command producer/Outbox persistence or exact-byte durable replay implementation.
- No authenticated Schedule/Run production transport, trusted identity mapping, target database app role/grants or tenant-RLS consumer context.
- No Run-owner admission Inbox, atomic provisional TaskExecutionRun/reservation/RunEvent/result writer, or Schedule result Inbox/link CAS/confirmation consumer.
- No Run-owner confirmation receipt, Infrastructure demand publication gate, owner-command compensation processor, or cancellation/reconciliation worker.
- No trusted current binding/profile resolver, Infrastructure owner reservations in a live flow, Multipass/K3s lifecycle, Manager/provider readiness, Agent Runtime sandbox, or scoped Worktree bridge.
- No BI/Benchmark, Run Task Card CLI, or production Schedule execution. All remain fail-closed.
- Verification is package-scoped; full workspace checks and PostgreSQL/NATS integration were not run for this DTO-only change.

## §4 Sub-agent failure and handoff

No sub-agent was dispatched for this slice. No child-agent output was incorporated. No child failure or handoff remains outstanding.

## §5 Guard rules

| # | Guard | Evidence/status |
|---:|---|---|
| 1 | Follow Project → Branch → Engineering Run → Worktree scope. | Message validates Project/Branch/Run against the pinned binding. |
| 2 | Preserve owner-local writes; no distributed transaction. | DTO layer only; no persistence added. |
| 3 | Keep Schedule's lease fence inside Schedule ownership. | Owner/generation/expiry excluded from payload; future link must CAS current fence. |
| 4 | Fix `run_as_actor_id`; reauthorize on every trigger. | Inherited occurrence snapshot; consumer still open. |
| 5 | Separate logical attempt from transport retry. | `request_id` + positive ordinal; transport redelivery reuses exact bytes. |
| 6 | Reject changed payload under the same request. | Protocol matching tests cover identity/scope/digest; durable Inbox conflict check remains open. |
| 7 | Authenticate owner identity out of band. | Documented; body tenant is not proof; production mapping remains open. |
| 8 | Pin exact Infrastructure binding identity and digest. | Uses `RunInfrastructureBindingSnapshotV2`; command/result/resolution matching implemented. |
| 9 | Bound payload and parsing memory. | Strict JSON, unknown-field rejection, 16 KiB maximum. |
| 10 | Keep accepted Run non-executing pending confirmation. | Result contract calls it provisional; no executor path added. |
| 11 | Publish Infrastructure demand only after Schedule confirmation. | Design constraint documented; consumer not implemented. |
| 12 | Require owner-command compensation for stale/cancelled work. | Typed reasons defined; processor remains open. |
| 13 | Avoid unrelated branch overlays and user data. | Candidate commits are already in `dev`; dirty worktrees were not modified or cleaned. |
| 14 | Record actual toolchain, lint exceptions and test scope. | Rust 1.99.0, direct Cargo path, four baseline lint exceptions and package-only evidence recorded. |
| 15 | Do not claim production, VM, broker or workspace completion. | No DB/NATS/Multipass activity; remaining gates listed in §3. |

## §6 Review sign-off

| Role | Reviewer | Decision |
|---|---|---|
| Architecture | Mavis 接手审核 | 🟢 Owner and scope boundaries match the approved design |
| SRE Lead | Mavis 接手审核 | 🟢 Bounded payload and fail-closed limits recorded |
| Platform | Mavis 接手审核 | 🟢 Out-of-band identity and owner-local messaging boundary recorded |
| Review Facilitator | Mavis 接手审核 | 🟢 Branch intake kept selective; active worktrees preserved |
| PM | Mavis 接手审核 | 🟢 Incomplete production capabilities are explicitly listed |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| 0.1 | 2026-10-09 | Mavis review | Add the 9F4C-D typed owner-message contract report, focused verification evidence, branch intake decision, and open production gates. | Freeze the Schedule→Run contract before implementing owner consumers. |
