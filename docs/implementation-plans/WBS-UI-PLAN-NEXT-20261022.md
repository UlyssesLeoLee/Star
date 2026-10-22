# WBS：下一版 Worktree-first UI

**Date:** 2026-10-22
**Revision:** v0.9
**Baseline:** local main, dev, and cached origin/dev were verified at 5a051bfd8f76fbfbe8ff49a994579c60166d6ec6; the WBS implementation lane starts from this shared commit. Remote tracking freshness beyond the cached ref has not been refreshed.
**Design:** [UI-PLAN-RUN-WORKTREE-NEXT-20261021.md](UI-PLAN-RUN-WORKTREE-NEXT-20261021.md) v1.6.
**Status:** UI-0 screen/state and Rust desktop ADR, target-machine static inventory, and hash-pinned workload parameters are recorded. Generator/runner implementation is active on branch `codex/ui0-bench-harness-20261022` in its registered worktree; measured display/DPI/power inventory is now available in the worker lane, while comparable UI tracks and renderer choice remain open. UI-1 now separates Project/Run selection from expansion and routes Project selection to the Project Worktree Index; three focused tree tests cover navigation, disclosure and ArrowRight expansion without selecting. UI-2 picker tests pass 6/6; the combined UI-1/UI-2 focused suite passes 9/9 and TypeScript typecheck passes. Production build was attempted and exited with Node OOM while host free memory measured about 0.81 GB. Browser/screen-reader acceptance remains open. Orca currently reports no live dev terminal/PTY or agent; recheck owner and HEAD immediately before integration.

## §0 Purpose

Convert the v0.8 UI plan into reviewable delivery phases. The product problem is multi-Agent Worktree ownership, lifecycle, concurrency and change visibility inside a Project. Every phase preserves the Project → Engineering Run → Worktree → Branch navigation hierarchy, Worktree-focus gate, Run-level app ownership and Rust desktop resource constraints.

The dev baseline includes the WLD-1 Run-scoped lifecycle API seam, but has no installed trusted provider by default, no complete Run↔Worktree binding writer, and no frontend migration from the disabled Project-scoped paths. Rust 1.99.0 and PostgreSQL validation were unavailable for that source slice. UI-0 now has a screen/state package and Rust desktop ADR; renderer choice and target-device measurements are still open. Consequently all physical lifecycle actions, task execution, chat send and BI ingestion remain fail-closed until their respective owners pass acceptance.

## §1 Fixed product and UI contracts

1. **Navigation:** Project is the first level, followed by Engineering Run, Worktree and local Branch/ref observation. Remote SCM identities are optional metadata and never reorder the tree.
2. **Focus:** selecting or expanding Project/Run does not open a Run app. Only a specific Worktree that passes current authorization and binding/version checks establishes focus and mounts that Run's app tabs.
3. **Run apps:** Inbox, Work Items/planning, Task Cards, Canvas, Workflow/Schedule, BI/Benchmark and authorized Plugin Apps are peers within the focused Run workspace. Task Card and Canvas are sibling tabs; CLI opens inside its Task Card.
4. **Ownership:** each display and mutation is backed by its owning API, current scope, entity/version, observed-at/freshness and operation receipt. UI caches are projections, never another source of business truth.
5. **Fail closed:** provider, session, permission, binding, policy, durable receipt, sandbox or freshness gaps render as unavailable/denied/stale/conflict. No seed, mock, preview store or process-start event may be presented as production success.
6. **Desktop:** Rust is the performance-sensitive desktop target. The existing Tauri+React shell and shared Web UI are comparison baselines, not proof of a Rust-native UI. Native security Hooks remain Rust-owned; plugins stay capability-limited and isolated.

## §2 Ordered delivery phases

| Phase | Deliverables | Dependencies | Acceptance gate | State |
|---|---|---|---|---|
| UI-0 — Design package and performance decision | Screen/state package, owner/API map, focus/keyboard/accessibility flows, Rust desktop IPC/resource/plugin/Hook ADR, target-machine static inventory, and hash-pinned workload parameters are recorded. | WLD owner contracts for displayed facts; target device inventory for measurements. | Design package reviewed; target-machine scenarios collect process-tree RSS, queue/cache/output counters, p50/p95 and cancellation/drain. Renderer and thresholds remain unselected until comparable target-device evidence exists. | Generator/runner implementation active in the registered UI-0 lane; display/DPI/power capture, comparable target-device runs, and renderer decision remain open. |
| UI-1 — Project and Worktree control surface | Project selector/tree; Run expand/select; Worktree directory and cross-Run index; ownership, Agent/Session/Task links, local Branch/HEAD, dirty/ahead/behind, locks/leases and freshness when their owners provide them. | UI-0 state dictionary; WLD-0 relation/binding source. | Project → Run → Worktree → Branch remains stable; the index carries a trusted Run ID/binding; missing facts appear unknown; expand and select are separate; keyboard navigation and paging are accessible. | Partial: Project and Run now have separate disclosure controls and selection links; Project selection sets Project scope and opens its Worktree Index, while Run selection opens the no-focus overview and clears Worktree focus. Three focused tree tests cover navigation, button expansion and ArrowRight expansion without selection. Run ID and binding IDs/versions flow through the projection; unavailable lock, dirty/ahead/behind, observed-at, and Branch freshness remain unknown. Browser, screen-reader and remaining owner facts are open. |
| UI-2 — Focused Run workspace | No-focus Run overview and paged Worktree picker; verified focus transition; same-level app tabs; lazy app data loading; inspector; fixed WORKTREE/GLOBAL Chat Bar scope. | UI-1; current session and RunContext contract. | No focus means zero Run App mounts/requests. Focus change cancels and drains old streams, clears scoped state, then reauthorizes and validates binding/version before mounting. GLOBAL does not broaden authorization. | No-focus gate and bounded picker implemented; single-page and aggregate limits reject oversized responses, page-local/cross-page duplicate IDs and repeated cursors are rejected; focused picker tests 6/6 and TypeScript typecheck pass. Production build failed with Node OOM/exit 134 at about 0.81 GB free host memory. Browser/e2e/accessibility and full shell acceptance remain open. |
| UI-3 — Worktree lifecycle controls | Run-scoped create/import/rebind/archive/restore/cleanup panels; command preview, blocker list, confirmation, idempotency identity, receipt and reconciliation view. | WLD-0 binding owner; WLD-1 provider, target DB/RLS and authenticated mutation acceptance; Run repository binding. | UI calls only Run-scoped contracts. No operation enables until trusted provider, live authorization, owner binding, retention/lease/PTY/Runtime drain and durable receipt gates are proven. Recovery is based on owner facts; no physical deletion on unknown/stale state. | Blocked on WLD-1 production closure; API source seam alone is insufficient. |
| UI-4 — Task Card and in-card CLI | Task detail and assignment evidence; CLI panel; bounded output; cancel/reattach/resume; validation evidence and result writeback. | WLD-2 Run-owned Task/Session provider, sandbox, native Hook and budget contracts; UI-2 focus shell. | The CLI is bound to Task/Run/Worktree/Session and current policy. Cancellation/recovery, independent verification, bounded auditable evidence and owner result writeback are demonstrated. Otherwise input stays disabled. | Planned; no production execution claim. |
| UI-5 — Infinite Canvas | Canvas sibling tab; viewport-based loading/rendering; typed references to Task, Worktree, Branch and other Run objects; safe deep links and selection sync. | Canvas owner persistence/API; UI-2; UI-4 typed Task references. | Canvas stores geometry/layout and typed references only; business mutations use the referenced object's owner API. Viewport work is bounded and cancellable; unresolved references are explicit. | Planned; canonical Run shell integration remains open. |
| UI-6 — Workflow, Hooks, analytics and plugins | Loop/Schedule timeline and config-vs-execution states; visual Hooks settings/deep links; BI/Benchmark coverage and freshness; plugin capability/consent/lifecycle; Infrastructure status. | WLD-3/5/6 and infrastructure owner acceptance; versioned RunContext; native Hook evaluator and transport-neutral events. | Rule configuration is distinct from trigger/occurrence/admission/execution; BI shows coverage/sample/freshness; plugin revoke stops new work and drains; Hooks remain Rust-enforced; Infrastructure view separates Host, Multipass instance, K3s cluster and node. Test inventory stays at two Multipass records maximum. | Planned; source slices are partial. |
| UI-7 — Desktop integration and release gate | Rust-native parity and accessibility validation; upgrade/session recovery; cross-client API contract; release evidence and known-gap screen. | UI-0 through UI-6; Rust 1.99.0 CI and production owners. | No UI-thread I/O; all work queues and caches have limits, backpressure, fairness, cancellation and drain; target-device measurements meet reviewed thresholds; denied/unknown actions remain safe and understandable. | Planned; no release acceptance. |

## §3 Dependency and parallel-work map

```text
UI-0 → UI-1 → UI-2 ──┬→ UI-3 lifecycle
                      ├→ UI-4 Task Card/CLI
                      ├→ UI-5 Canvas
                      └→ UI-6 Workflow/Hooks/BI/Plugin/Infrastructure
UI-3..UI-6 → UI-7 release gate
```

UI-3 through UI-6 may be delivered in separate lanes only after UI-0 freezes shared RunContext, EntityRef, capability, freshness, event and error-state contracts. Before each lane is created, register its exact file manifest; do not let parallel lanes edit shared navigation, RunContext, state dictionaries or WBS concurrently. Parallel execution is not acceptance: each lane must pass owner-specific validation and integrate serially into `dev`.

## §4 Rust desktop performance and resource gates

- **Bounded reads:** Project/Run summaries on entry; paged Worktree and Task lists; Canvas viewport queries; terminal/transcript ring buffers bounded by bytes; no full-history resident cache.
- **Bounded parallelism:** per-Project/Run/Worktree/Agent/Plugin CPU, memory, file-handle and subscription budgets; bounded queues with item and byte caps; Run fairness and backpressure; separate priority for current input/focus from BI/thumbnail/background work.
- **Cancellation:** request generations prevent stale results from repainting; long API, filesystem, Git, render, IPC and plugin tasks can cancel and drain. A canceled UI request does not claim that an external process stopped; owner receipts must prove it.
- **Measurement:** record device/OS/build, renderer, Rust/toolchain, fixture hash, warm/cold run, process-tree RSS, queue/cache/terminal/Canvas counters, input-to-feedback and scroll/focus/Canvas p50/p95, event-to-projection delay and cancel-to-drain. Compare Tauri+React, shared Web reference and Rust parity separately.
- **Infrastructure test cap:** use Multipass-managed Linux guests for K3s scenarios; retain at most two inventory records during this test phase. Do not require Docker Desktop or auto-purge instances to recover disk space.
- **Threshold policy:** collect target-machine baseline first; reviewers approve thresholds after observing the workload. No generic RSS or latency targets are invented in this WBS.

## §5 Cross-owner readiness gates

| Capability | Required before enabling the corresponding UI |
|---|---|
| Worktree lifecycle | Current Project/Run permission, Run repository binding, trusted root resolver/provider, target DB/RLS, durable idempotent receipt, owner audit/Outbox and recoverable reconciliation. |
| Agent / Task CLI | Current Run/task binding, provisioner, isolated sandbox, native Hook decision, budget reservation, cancellation/drain, independent verification and result writeback. |
| Schedule / Loop | Immutable run-as identity, current per-trigger/retry/resume authorization, occurrence materialization, owner-local admission and confirmed infrastructure capacity/readiness. |
| Hooks | Visual typed policy editor in Advanced Settings → Hooks and a Rust-native fail-closed evaluator; selected UI events and BI records share a versioned HookSet contract. |
| BI / Benchmark | Durable owner event intake, explicit definition/version, sample and coverage metadata, freshness and replay/recovery evidence. |
| Plugins / transports | Versioned capabilities, isolated runtime, revocation/drain and transport-neutral owner event/outbox contracts. No arbitrary native plugin loading in the desktop main process. |
| Multipass/K3s | Explicit RunInfrastructureBinding with cluster identity/generation; Host and cluster reservations from their owners; independent Agent sandbox readiness. Guest readiness alone cannot enable task execution. |

## §6 First implementation batch after this plan

1. UI-0 design inputs are recorded in [UI0-SCREEN-STATE-20261022.md](UI0-SCREEN-STATE-20261022.md), [UI0-RUST-DESKTOP-ADR-20261022.md](UI0-RUST-DESKTOP-ADR-20261022.md), and [UI0-BENCH-001](fixtures/ui0-bench-fixture-v0.1.json). The deterministic generator/runner is being implemented in branch `codex/ui0-bench-harness-20261022`; next capture display refresh/DPI/power and execute comparable tracks after the lane passes review. Keep renderer and thresholds open pending evidence.
2. UI-1 selection/expansion split is implemented and focused tests pass 3/3, including ArrowRight disclosure without selection. Next validate actual browser navigation, focus/activation and screen-reader semantics; obtain owner/API facts for observed-at, lock, dirty/ahead/behind, and Branch source/freshness. Keep missing facts unknown.
3. UI-2 targeted checks pass: focused Vitest is 6/6 and TypeScript typecheck passes. Browser/e2e and accessibility acceptance remain open; search covers only loaded pages (up to 200 rows), and server-side search requires an owner/API contract.
4. Start UI-3 only after WLD-1 provider, binding owner, target DB/RLS and receipt acceptance exist; until then document disabled states and recovery copy.
5. Keep Task CLI, Canvas and BI/Benchmark as distinct owner-backed app surfaces with cross-links, and reserve independent implementation lanes only after the shared contracts are frozen.

## §7 Revision history

| Version | Date | Change |
|---|---|---|
| v0.9 | 2026-10-22 | Implemented the UI-1 Project/Run selection and expansion split; Project selection opens the Project Worktree Index and Run selection opens a no-focus overview. Three focused tree tests cover navigation and keyboard disclosure; combined UI suite is 9/9 and typecheck passes. Browser/accessibility, owner facts, UI-0 harness and dev owner gates remain open. | WBS continuation after UI-2 boundaries |
| v0.8 | 2026-10-22 | Started the registered UI-0 generator/runner worktree and recorded its exact lane; confirmed Orca's dev checkout still has one live attached terminal, so final integration remains gated on owner state. UI-2 production build OOM is recorded as a failed check. | WBS continuation with parallel UI-0 work |
| v0.7 | 2026-10-22 | Recorded that the Next.js production build was attempted but exited with Node OOM while only 0.81 GB host memory was available; do not treat this as a passing build. UI-0 generator/runner remains next. | UI-2 build validation |
| v0.6 | 2026-10-22 | Closed UI-2 page-boundary gaps: validate the first and later responses, cap each response at 50 and total choices at 200, reject page-local/cross-page duplicate IDs and cursor loops; focused tests are 6/6. UI-0 generator/runner remains the next lane. | UI-2 boundary review and WBS continuation |
| v0.5 | 2026-10-22 | Recorded focused UI-2 test/typecheck/CypherGraph/diff-check evidence; marked UI-0 generator/runner as the next implementation lane while retaining browser, accessibility, and performance gaps. | WBS continuation and UI-2 verification |
| v0.4 | 2026-10-22 | Recorded target-machine inventory and hash-pinned UI0-BENCH-001 workload parameters; documented missing owner freshness fields and visible unknown metadata. Generator, measurements, display/power details, and renderer decision remain open. | UI-0 and UI-1 audit |
| v0.3 | 2026-10-22 | Updated the implementation baseline to 5a051bfd; recorded bounded picker/search implementation and its open validation/server-side search gaps. | Continue UI-2 from latest dev |
| v0.2 | 2026-10-22 | Updated to the actual dev 17b3d065 baseline and linked UI plan v0.9; recorded the completed screen/state and Rust desktop ADR artifacts while keeping target-device performance and renderer selection open. No UI implementation or production capability is claimed. |
| v0.1 | 2026-10-22 | Turned UI plan v0.8 into ordered UI-0..UI-7 deliverables, dependency/parallel map, Rust resource acceptance gates and explicit WLD owner prerequisites. No UI implementation or production capability is claimed. |
