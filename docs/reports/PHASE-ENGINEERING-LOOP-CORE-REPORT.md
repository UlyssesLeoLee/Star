# Phase 9F1 — Run-local Engineering Loop Controller Core Report

> Status: 🟡 Core slice implemented; production Engineering Loop remains open
> Date: 2026-10-02
> Author / review: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> Scope: `domain-agent::engineering_loop` and synchronized requirements/design documents

## §0 Purpose

This report records the bounded Run-local Rust Engineering Loop controller slice, its direct validation, and the production dependencies that remain open. This phase does not claim a complete Agent execution or Schedule-to-Loop production path.

## §1 Change Matrix

| Area | Change | Evidence / state |
|---|---|---|
| Domain controller | Added immutable Run/Task/Worktree and Profile/Contract/Acceptance/HookSet/Validation identity binding, boundary snapshot checks, bounded fingerprints, digest-only phase receipt, stop reasons and review gate | `crates/domain-agent/src/engineering_loop.rs`; crate unit tests |
| Resource controls | Added iteration/runtime/CPU/RSS/child/provider/output/event ceilings, atomic per-Run tool permits, no-waiter backpressure and bounded drain receipt | Implemented in the pure domain controller; live OS/process sampler and global fair scheduler remain open |
| Validation | Requires an independent validation provider and exact pinned suite/toolchain identity; success transitions only to `AwaitingReview` | Controller does not mutate Task/Run owner state |
| Requirements/design | Updated root requirements v5.52, basic design v5.49, Task SRS v0.8, Task DD v1.21 and implementation plan v5.68 | Each document describes both the delivered slice and remaining gates |
| Legacy mock cleanup baseline | The previously authorized old Task mock retirement is already present on `dev` | Baseline commit `2f7ac2a3`; this report does not expand deletion to test fixtures or unknown persisted rows |

## §2 Validation Summary

| Gate | Result |
|---|---|
| `cargo test -p domain-agent --lib -j 4` | Passed: 142/142 tests |
| `rustfmt --edition 2021 --check crates/domain-agent/src/engineering_loop.rs` | Passed |
| `cargo clippy -p domain-agent --all-targets --no-deps -j 4 -- -D warnings -A clippy::new_without_default -A clippy::collapsible_if -A clippy::derivable_impls` | Passed; the three explicit allowances are existing crate-wide warning classes, not newly introduced code |
| Strict dependency-inclusive Clippy | Blocked by pre-existing warnings in `star-context` and `domain-llm`; no claim of a clean workspace Clippy run |
| `cargo test --locked` | Blocked because the committed workspace lock does not resolve the current workspace dependency graph; online package test resolved dependencies successfully. Generated lockfile churn is excluded from the change |
| Workspace / desktop / target DB / runtime integration | Not run; no claim of passing |
| CodeRabbit self-review | Attempted `coderabbit review --agent -t uncommitted`; launcher delegated to WSL `/root/.local/bin/coderabbit`, which returned `Permission denied`, so independent review did not complete |

## §3 Known Gaps

1. No production Run admission or current actor/grant reauthorization is wired to the controller.
2. No production Agent provider, Task-card CLI adapter, OS process tree, sandbox, terminal evidence path or cancellation/recovery adapter is wired.
3. The controller has no durable checkpoint, crash recovery, database writer, Outbox event or resume path.
4. Schedule Rule/Occurrence dispatch, worker lease/fencing, overlap policy and retry/backoff are not implemented here.
5. Profile v1 has no cumulative monetary/call-cost cap; current provider-call count is only a bounded count.
6. Run resource measurements are caller-supplied snapshots; no live CPU/RSS sampler or Project-wide quota allocator is attached.
7. Per-Run tool concurrency is bounded, but there is no cross-Run fair scheduler or starvation policy.
8. BI/Benchmark event consumers, target database/RLS deployment and production operating evidence remain open.

## §4 Subagent Failure / Takeover

No subagents were dispatched for this phase; therefore there are no remote worker statuses or uncollected outputs. Changes and test evidence were produced and checked in the current feature worktree.

## §5 Guard Rules Applied

1. Keep safety-critical Hook and authorization decisions under the Rust core; a plugin or model cannot weaken them.
2. Do not claim a production Agent/CLI closure without authorized Run binding, controlled execution, cancellation, independent validation, bounded evidence and result write-back.
3. Fail closed when Profile, scope, snapshot identity, provider, validator or drain evidence is missing or inconsistent.
4. Use fixed-size or explicitly capped histories, receipts, captured output, event buffers and concurrency permits.
5. Do not create an unbounded waiting queue when tool capacity is exhausted.
6. Bind each iteration to the immutable Profile, Contract, acceptance, HookSet and Validation identities.
7. A successful validator result only opens `AwaitingReview`; the Task/Run owner workflow controls final lifecycle transitions.
8. A drain is complete only when observed child processes equal confirmed releases; timeouts and mismatches remain incomplete.
9. Never persist prompts, private reasoning or unbounded logs in Loop receipts.
10. Keep Schedule occurrence ownership in `domain-automation`; the Loop controller must not create a second schedule authority.
11. Keep the separate Validation provider identity and pinned suite/toolchain check.
12. Do not infer target-database, host-auth, OS-sandbox, desktop or BI acceptance from domain unit tests.
13. Reconcile requirements, basic design, detailed design, plan and report with actual implementation.
14. Run the Cypher Graph Guardian against every changed code relationship before finalizing.
15. Restore resolver-generated `Cargo.lock` churn when it is not part of the intended change.

## §6 Signature

| Role | Sign-off |
|---|---|
| Architecture | 架构师（Mavis 接手 agent per DEC-008） |
| SRE Lead | 架构与实施风险已记录；生产运行验收未完成 |
| Platform | Rust domain slice only; production Runtime/provider integration remains open |
| Review chair | Mavis 接手审核 |
| PM | Phase 9F1 core slice delivered; overall Phase 9 remains open |

## §7 Revision History

| Version | Date | Author | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Initial report for Run-local bounded Engineering Loop controller and validation boundaries | Phase 9F1 implementation and document reconciliation |
