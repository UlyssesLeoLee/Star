# Brief: ERUN-P3.1-task-owner-adapter

**Agent**: worker
**Phase**: ERUN-P3.1
**Created**: 2026-10-15 02:58:51

---

Implement canonical Run Task Card command contract and PostgreSQL adapter in domain-work-item/infrastructure only. Keep legacy three-state service unchanged. Preserve tenant/actor auth, Run/Worktree binding, idempotency, lifecycle row lock/CAS, append-only audit, metadata-trigger Outbox and rollback. Support create/transition/review plus atomic Canvas task creation. Do not claim lifecycle Outbox parity. No REST/docs/migrations. Add Cypher headers and focused tests. Base dev 8e973f9c.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
