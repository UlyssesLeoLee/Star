# Brief: phase9f4h-bi-panel

**Agent**: merge_docs_review
**Phase**: 9F4H
**Created**: 2026-10-17 11:13:56

---

Own frontend/src/components/run/RunBiPanel.tsx, RunWorkspaceShell.tsx and relevant tests only. Add lazy BI tab panel using RunDirectoryApiClient.listRunTaskEvents(run,cursor?,signal?) and exported RunBiTaskEventCursor/Item/Page types. Tab appears/enables only with existing selected Worktree focus, owner remains Run. One current page50, at most100 cursors, abort on switch/unmount, no polling, no retaining pages. Display unknown completeness/freshness and empty as no observed events, not zero-complete. No mock/seed, no competitor names or internals in UI copy. Root owns docs; other agent owns client. No commits or reverting others. Graph current candidate Tier2 targeted source, coverage tool unavailable. No invented history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
