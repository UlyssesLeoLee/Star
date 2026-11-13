# Brief: STAR-RUNTIME-REMOVAL-20261030

**Agent**: worker
**Phase**: STAR-RUNTIME-REMOVAL-20261030
**Created**: 2026-10-30 15:23:43

---

Remove the retired canvas-game Rust service, stale game-labeled desktop demo row, and old game-specific UI/route/docs references embedded in the supported Star Canvas source. Keep Star API, Canvas engine, domain-canvas, and canvas-realtime intact.

Owned files only:
- Cargo.toml and Cargo.lock
- crates/canvas-game/** (remove this placeholder crate)
- scripts/automation/group_guard.py
- scripts/automation/__tests__/test_group_guard.py
- scripts/automation/arch_live_doc_check.py
- crates/star-desktop/src-tauri/src/db_adapter.rs
- crates/star-desktop/src-tauri/src/lib.rs
- crates/canvas-engine/Cargo.toml
- crates/canvas-engine/src/lib.rs
- crates/canvas-engine/src/main.rs
- crates/domain-canvas/Cargo.toml
- crates/domain-canvas/src/lib.rs
- crates/canvas-realtime/Cargo.toml
- crates/canvas-realtime/src/lib.rs
- crates/canvas-realtime/src/main.rs
- this brief and docs/reports/dispatcher/STAR-RUNTIME-REMOVAL-20261030.jsonl

Requirements:
1. Verify there are no production imports or required dependents before deleting the crate. Remove only this game service scaffold, its workspace member/lock entry, Dockerfile, placeholder endpoints and smoke test. Keep other canvas crates and any unrelated game/math library outside this crate unchanged.
2. Remove game-only paths from the automation group allowlist and architecture live checker. Update the corresponding existing guard tests/branch fixture so active tooling no longer treats the retired service as supported.
3. Inspect the desktop adapter fixture named wt-canvas-game. It is a stale demo/test fixture; remove it from any product-visible seed/demo output, or replace it with a Star worktree fixture only if the existing tests need a representative row. Preserve useful test-only fixture coverage and avoid unrelated mock-system refactors.
4. In the eight explicitly listed retained Canvas source files, remove only clearly game-specific titles, buttons, the /api/v1/gameplay placeholder route, and game-only design/deployment references. Preserve Canvas platform, API, persistence, real-time collaboration, and existing service behavior. Update old manifest-path references to deploy/star-canvas-services-k3s.yaml, the delivery lane's planned replacement. Do not alter other business behavior or introduce new game functionality.
5. Minimize Cargo.lock churn. Use Rust 1.99.0. Use --locked for check/test verification; if removal requires Cargo to prune the workspace package entry, first run Cargo once without --locked to make only that minimal lock change, inspect the complete lockfile diff for version churn, then rerun the verification with --locked. Do not run a lockfile-wide version update.
6. Do not edit CI/deployment/docs files owned by other lanes. Report any additional active source references outside this explicit manifest to the coordinator before touching them.
7. Commit the lane in this worktree with author Ulysses <ulysses@mavis.local>. Report changed paths, checks and results. No push; do not modify dev or main.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
