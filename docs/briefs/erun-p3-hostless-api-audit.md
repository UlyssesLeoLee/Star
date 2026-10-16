# Brief: erun-p3-hostless-api-audit

**Agent**: reviewer
**Phase**: ERUN-P3.2-HOSTLESS-TASK-OWNER
**Created**: 2026-10-15 19:50:58

---

Read-only API/UI contract audit for Run-owned Task Cards that remain usable without a remote host Branch. Inspect current Group API routes, RunTaskCardsPanel, runDirectoryApi client, and tests. Report the smallest coherent API/context change for Project/Run identity with optional Worktree binding; exact current Branch-dependent UI gates and API calls; pagination/create/idempotency/recovery invariants to preserve; current error/status behavior; frontend and Rust files/tests affected; which CLI/execution actions must remain closed. Do not edit files, run tests, or claim completion. CodeGraph has no index in this worktree; use focused source inspection. Evidence starting points: crates/star-api-rest/src/group_api/work_items.rs, engineering_runs.rs, crates/infrastructure/src/run_task_card_pg.rs, crates/domain-work-item/src/run_task_card.rs, frontend/src/components/run/RunTaskCardsPanel.tsx and tests, frontend/src/lib/run/runDirectoryApi.ts and tests.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
