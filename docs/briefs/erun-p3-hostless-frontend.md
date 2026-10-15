# Brief: erun-p3-hostless-frontend

**Agent**: worker
**Phase**: ERUN-P3.2-HOSTLESS-TASK-OWNER
**Created**: 2026-10-15 19:55:35

---

Implement host-independent Run Task Cards UI/client support in ONLY frontend/src/components/run/RunTaskCardsPanel.tsx, frontend/src/lib/run/runDirectoryApi.ts, related taskCardCreateRecoveryStore.ts and their focused tests. Contract: Project+Engineering Run owns Task Cards; remote repository/Branch are optional, Worktree focus is optional context; GET/POST use /api/v1/engineering-runs/{run_id}/work-items with POST body focus_worktree_id nullable. Preserve keyset paging, strict Run/Project response validation, idempotency recovery/replay safety and old persisted pending records; do not replay a pending create to a different target. No client-provided actor/tenant auth. Keep Task Card CLI/execution controls disabled and do not enable all Run Apps globally. Add tests for hostless list/create, null/valid/mismatched focus, authorization and legacy recovery. Work only in C:\Users\leo19\.codex\worktrees\run-task-command-port\Star and only these frontend files/tests. No commit. Preserve every unrelated change; other workers own separate paths.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
