# Brief: STAR-STABLE-W5-FRONTEND-MISC-20261101

**Agent**: worker
**Phase**: STAR-STABLE-W5-FRONTEND-MISC-20261101
**Created**: 2026-11-01 09:00:17

---

Task: Resolve the remaining search, WebSocket protocol, query fallback, and Gantt test failures.
Owner: Assigned worker; other agents are changing separate isolated worktrees.
Base: dev@41f84d407e0c245af88ab29938e8dda57c67af94.
Scope: frontend/src/lib/search/globalSearch.ts and test; frontend/src/lib/terminal/wsProtocol.ts and test; frontend/src/lib/wasm/query-engine.ts and test; frontend/src/components/gantt/GanttChart.tsx and test; plus this lane's brief, phase report, dispatcher JSONL, and dispatcher audit log only.
Baseline failures: globalSearch relies on a removed example task; unknown WebSocket type test expects a throw; query-engine parity test expects case-sensitive matching; two Gantt drag callbacks fail.
Goal: Reconcile each behavior with current docs and implementation contracts. Never reintroduce removed mock/demo tasks; use test-local fixtures or update stale expectations. Confirm WebSocket unknown-message semantics from its protocol schema; keep Rust/JS query matching parity; verify Gantt updates the intended model.
Validation: run each affected focused test file; record exact commands/results, contract decisions, and unresolved risks in a phase report with required sections. Do not edit shared WBS, other lanes, lockfiles, main, or dev. Commit only manifest files with the brief path in the commit message.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
