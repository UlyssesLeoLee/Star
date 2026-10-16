# Brief: erun-p3-hostless-owner-migration

**Agent**: worker
**Phase**: ERUN-P3.2-HOSTLESS-TASK-OWNER
**Created**: 2026-10-15 19:55:35

---

Implement one additive PostgreSQL migration for canonical Run Task Card ownership by (tenant_id, project_id, engineering_run_id), with repository_id and branch_id nullable optional host context. Preserve legacy unassigned rows without guessed backfill; new current task owner requires Project+Run. Replace old 5-column Run FKs/checks/metadata-worktree-run-binding guards coherently; allow optional Worktree association and Run-scoped lifecycle audit evidence without mutating historical rows. Evolve task_run_outbox to a new bounded schema version with optional host context while retaining append-only behavior and existing RLS; inspect and explicitly set only necessary grants if role identities are provable. Add graph manifest for new SQL migration and refresh checksum only if the repository guard supports it. Include W/T/M table classification in comments. Do not touch Rust/API/UI or design docs; no tests/DB claims. Work only in C:\Users\leo19\.codex\worktrees\run-task-command-port\Star. No commit. Preserve every unrelated change; other workers own separate paths.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
