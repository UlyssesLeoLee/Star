# Brief: WBS-W14-DESKTOP-UI-OPTIONS-20261026

**Agent**: rust-desktop-ui-research
**Phase**: WBS-W14
**Created**: 2026-10-26 06:44:52

---

# WBS W14 research: Rust desktop UI options

## Goal
Prepare a decision-ready, source-backed planning memo for a future Star desktop UI direction. This is a research/design phase only; do not implement or make a binding framework decision.

## Owned file
- Create/edit only `docs/reviews/WBS-W14-RUST-DESKTOP-UI-OPTIONS-20261026.md`.
- The dispatcher-created task brief is `docs/briefs/WBS-W14-DESKTOP-UI-OPTIONS-20261026.md`.
- Do not edit the WBS, SRS/ADR/BD/DD/TD, app source, manifests, lockfiles, or any other files. Do not commit, rebase, merge, push, or clean worktrees.

## Product and engineering constraints
- The left navigation is Project → Engineering Run → Worktree → Branch. Selecting a Worktree scopes the Run's App tabs and CLI checkout focus. Canvas and task cards are siblings under Worktree/Run app scope. Chat has explicit WORKTREE/GLOBAL scope.
- User's central pain is multi-agent Worktree sprawl and lack of control inside a Worktree. Desktop UI must make Worktree ownership, branch/ref state, active agents, runs, conflicts, and resource use legible.
- Target is a high-performance Rust desktop application. Reduce idle and loaded memory, bound retained logs/history, support parallel work without blocking UI, and use paging/virtualization, on-demand loading, bounded caches/queues, cancellation and drain. Do not claim Rust automatically uses less memory or performance without measurements.
- Do not mention or compare the product to competitor brands. Framework/library names are allowed.
- Commercial/open-source review must follow project AGENTS: commercial use is not banned merely by GPL/AGPL/LGPL; inspect linking, bundling, redistribution and source/notice obligations. Exclude non-commercial, field-of-use, source-available, or core-feature paywall restrictions. No licensing legal conclusion beyond evidence.
- Rust MSRV baseline is 1.99.0; do not introduce dependencies or modify toolchain.

## Verified implementation baseline from read-only W14 audit
- Current desktop is Tauri 2 shell with React 19/TypeScript/Vite (`crates/star-desktop/frontend/package.json:13-29`, `crates/star-desktop/README.md:28-39`).
- Desktop Worktree page currently synthesizes nodes in `frontend/src/pages/WorktreeVizPage.tsx:29-43`; layout is JS Map/useMemo (`:45-86`); full SVG node rendering has no viewport virtualization (`:141-173`).
- `KanbanBoard.tsx:193-233` maps full lists without virtualization; `BoardView.tsx:56-68` filters the full work item list.
- Rust `canvas-renderer` has viewport/LOD facilities but Tauri depends on `canvas-engine`, not `canvas-renderer` (`src-tauri/Cargo.toml:32-36`); it is not wired into this UI.
- Worktree/Canvas Tauri commands synchronously return unpaged Vec values and demo DB returns fixed fixtures (`src-tauri/src/lib.rs:154-193`); do not describe this as a production paging contract.
- `useLayoutEngine.ts:7-8,57-94` attempts WASM and falls back to JS; WorktreeVizPage's custom layout still runs in JS.
- No app-level bounded UI queue/backpressure/cancel/drain or RSS/p95/benchmark evidence was found in the bounded audit. No tests/build/runtime/performance measurements were run. These are not proven absent repository-wide; state audit scope and verification limits.

## Research scope
Compare at minimum:
1. Existing Tauri 2 + web UI (React) as a keep-and-optimize baseline.
2. Slint.
3. egui/eframe.
4. iced.
Optionally include Bevy UI only if there is a concrete repo-level integration reason; keep comparison bounded.

Use up-to-date primary sources only: official project docs and the canonical project repositories/releases. Establish evidence of community/maintenance activity within the past 12 months (as of 2026-10-26) using dated official release/changelog/maintenance evidence. Cite direct URLs beside claims. Do not rely on search snippets alone.

For each option, compare:
- Rust/native boundary and migration/integration fit with current Tauri commands, app APIs, auth/RunContext and plugin model;
- renderer/windowing choices and multi-platform impact;
- UI paradigm and state/concurrency/cancellation implications;
- large Worktree tree, Canvas/graph, Task/Kanban virtualization or viewport strategy; if native support is not documented, say it must be built;
- accessibility/input, international text, complex layouts and user customization;
- memory/performance characteristics as design implications only (no unsupported numeric ranking);
- build/deploy/MSRV/maintenance and license plus redistribution obligations;
- risk and a small proof-of-concept/measurement plan.

## Output structure
1. Purpose, scope, evidence date.
2. Current implementation facts and NOT_RUN limits.
3. Comparison matrix with cited sources.
4. Architecture implications for Project→Run→Worktree→Branch navigation and Worktree-first operational UI.
5. Provisional recommendation with explicit assumptions and decision gates; preserve existing app until a measurement-backed migration decision.
6. A phased UI plan (information architecture, interactive prototype, bounded data contracts/virtualization, resource profiling on target Windows hardware, migration decision) with acceptance criteria and dependencies.
7. Sources and evidence limitations.
8. Explicit status: planning/research only; no implementation or performance acceptance.

Do not edit docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md; parent will integrate reviewed result and update the WBS after all lanes converge.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
