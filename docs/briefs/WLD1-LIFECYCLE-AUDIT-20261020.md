# Brief: WLD1-LIFECYCLE-AUDIT-20261020

**Agent**: worker
**Phase**: WLD1-LIFECYCLE-AUDIT-20261020
**Created**: 2026-10-20 06:43:51

---

WLD-1 source audit only. Work only in this worktree and modify only docs/reports/WLD1-LIFECYCLE-AUDIT-20261020.md. Trace ProjectWorktreeLifecycleProvider from GroupApiState through startup/composition, persistence and trusted host provider. Verify whether an implementation exists, what owner/audit/outbox/idempotency/authorization checks exist, and how current group_api/worktrees.rs archive readiness/Hook gates should be reused. Identify the smallest safe next implementation slice, exact source paths/symbols, missing tests and production gates. Do not change Rust/SQL/UI source; do not claim target DB or runtime acceptance. Base is dev 1136a24c709e944a538d28a2feff7e72b2b29f4c. Graph context from coordinator: ProjectWorktreeLifecycleProvider in crates/star-api-rest/src/group_api/worktree_lifecycle.rs:182 exposes repository/candidate reads plus create/import; GroupApiState in crates/star-api-rest/src/group_api.rs:323 defaults the provider to None (initialization around 342; setter around 387). Create/import handlers around 388-474. Existing archive gate is in crates/star-api-rest/src/group_api/worktrees.rs around 600, 960-1030 and tests around 1539. Coordinator used CodeGraph Tier 2, indexed worktree exists (1,891 files; generation/coverage API unavailable), then focused rg fallback; no index coverage tool is available. Read exact current files before conclusions.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
