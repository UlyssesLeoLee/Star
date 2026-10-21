# Brief: UI0-RUST-DESKTOP-ADR-20261022

**Agent**: codex
**Phase**: UI0
**Created**: 2026-10-22 03:27:39

---

# UI-0 Rust Desktop ADR and Benchmark Protocol

## Target
Using the current local dev baseline at 17b3d06539f0281b8a5f9e7a1576864487b26337, produce the UI-0 renderer/IPC/plugin-isolation architecture decision and reproducible performance protocol at docs/implementation-plans/UI0-RUST-DESKTOP-ADR-20261022.md. This is a design and evidence plan, not implementation.

## Change
Evaluate the current Tauri plus Web UI shell against Rust-native desktop rendering candidates and document an ADR that:
- Separates current verified architecture from proposed direction and open decisions.
- Defines typed, versioned, scope-aware IPC for Project, Run, Worktree, Branch, Task, Canvas, Agent, Schedule, BI, Hook and Infrastructure projections.
- Specifies bounded queues, paging/virtualization, lazy loading, cache limits and eviction, backpressure, fairness, cancellation and drain.
- Keeps security-critical Hooks in the Rust core and isolates plugins behind versioned capabilities; never load arbitrary native plugin code in the desktop main process.
- Defines how UI state avoids duplicating owner truth and how stale generations, focus switches, late callbacks and large logs are fenced.
- Provides a reproducible Tauri/Web-reference/Rust-native benchmark protocol covering startup, navigation, Worktree directory, Task Card, Canvas viewport, streaming output, accessibility, cancellation and idle behavior.
- Measures process-tree RSS, per-process memory, cold/warm startup, input-to-visible latency, p50/p95 interaction latency, frame-time/jank, queue depth, cache bytes, retained output bytes, paging work, cancellation latency, drain completion and idle resource use.
- Uses deterministic fixtures and recorded host/runtime/toolchain details. Separate hard invariants from target-device thresholds; do not invent machine-independent memory or latency limits. Identify the target hardware inventory and thresholds as a release gate if unavailable.
- Gives a staged recommendation and decision criteria rather than implying a library is chosen without evidence. When evaluating a specific Rust UI framework/library, use its current official documentation and record the source and version.

## Constraints
Honor Rust MSRV 1.99.0, low-memory desktop use, parallel Agent workloads and the existing fail-closed execution model. Preserve Project → Engineering Run → Worktree → Branch and Worktree-only app focus. Do not make code changes, modify AGENTS.md, the shared WBS, or the shared UI plan. Do not use product-comparison copy or competitor product names.

## Ownership
Write only docs/implementation-plans/UI0-RUST-DESKTOP-ADR-20261022.md. This is documentation-only. Do not run implementation changes.

## Observable acceptance
The ADR separates decision, alternatives, rationale, trade-offs, unresolved decisions, and revision history. The benchmark procedure can be repeated by another developer and yields comparable measurements from identical fixtures. All measurements and thresholds not yet collected are marked pending rather than asserted.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
