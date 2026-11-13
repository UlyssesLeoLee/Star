# Brief: STAR-STABLE-W5-VITEST-DISCOVERY-20261101

**Agent**: worker
**Phase**: STAR-STABLE-W5-VITEST-DISCOVERY-20261101
**Created**: 2026-11-01 09:00:09

---

Task: Exclude Playwright end-to-end files from Vitest discovery.
Owner: Assigned worker; this lane is isolated and other agents are changing separate worktrees.
Base: dev@41f84d407e0c245af88ab29938e8dda57c67af94.
Scope: frontend/vitest.config.ts plus this lane's brief, phase report, dispatcher JSONL, and dispatcher audit log only.
Baseline: from frontend/, rtk proxy npm test on Node 22.19.0 / npm 10.9.3 / Vitest 5.0.1 ran 69.66s: 947 tests, 929 passed, 18 failed; 115 files, 100 passed and 15 failed. Four frontend/e2e/*.spec.ts files failed collection because Playwright test.describe() ran outside its own runner.
Goal: Make Vitest include only intended Vitest tests and exclude Playwright E2E specs. Keep Playwright responsible for those specs. Do not broadly exclude *.spec.ts, skip failures, or reduce unit-test coverage.
Validation: run the targeted collection/config check, then npm test if feasible. Other known failing assertions are owned by other lanes, so do not claim the full suite passes. Record exact commands/results, limitations, and current status in a phase report with the repository's required sections. Do not edit shared WBS, lockfiles, other lanes, main, or dev. Commit only manifest files with the brief path in the commit message.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
