# Brief: WLD1-PROJECT-REPOSITORY-DECISION-20261025

**Agent**: worker
**Phase**: WLD1-PROJECT-REPOSITORY-DECISION-20261025
**Created**: 2026-10-25 14:42:11

---

Update only docs/reviews/WBS-WLD1-R0-OWNER-AUDIT-20261025.md and docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md in the registered WLD1 lane. Incorporate the user decision: Project may link 0..N repositories; unbinding an association requires migrating all active Run/Worktree references first, verifying destination bindings and durable audit/outbox evidence, then unbinding with concurrency-safe revision checking. Do not invent a canonical Project identity or authorization authority: keep R0a blocked/open pending that choice. Align WBS R0b scope and acceptance, explicitly note what code/DB/API and PostgreSQL evidence is not yet implemented or verified. No source code or schema changes. Preserve existing lane history and no fabricated historical claims. Return exact changed paths, commit hash, validation commands/results, and remaining gates.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
