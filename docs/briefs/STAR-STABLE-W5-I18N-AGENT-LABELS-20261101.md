# Brief: STAR-STABLE-W5-I18N-AGENT-LABELS-20261101

**Agent**: worker
**Phase**: STAR-STABLE-W5-I18N-AGENT-LABELS-20261101
**Created**: 2026-11-01 09:00:15

---

Task: Repair locale and agent-session display test failures against the current UI contract.
Owner: Assigned worker; other agents are changing separate isolated worktrees.
Base: dev@41f84d407e0c245af88ab29938e8dda57c67af94.
Scope: frontend/src/lib/i18n/I18nProvider.tsx and its tests; frontend/src/components/agent/SessionCard.tsx and its tests; plus this lane's brief, phase report, dispatcher JSONL, and dispatcher audit log only.
Baseline failures: three locale tests expect obsolete Tactical Jump labels but the current UI shows workspace-switch labels; SessionCard test expects a raw provider id while the UI displays a human-readable provider label.
Goal: Check existing localization/product copy and component API contracts first. Update implementation or stale assertions based on evidence, not by copying old test expectations. Keep zh-CN/en/ja behavior coherent and keep the selected provider representation consistent.
Validation: run the focused I18nProvider and SessionCard test files; record exact commands/results, review decisions, and remaining risks in a phase report with required sections. Do not edit shared WBS, other lanes, lockfiles, main, or dev. Commit only manifest files with the brief path in the commit message.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
