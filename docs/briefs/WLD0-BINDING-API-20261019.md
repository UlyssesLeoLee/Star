# Brief: WLD0-BINDING-API-20261019

**Agent**: worker
**Phase**: WLD0-BINDING-API
**Created**: 2026-10-19 00:39:28

---

# Task
Implement the WLD-0 Run-to-Worktree binding mutation slice in an isolated worktree.

# Ownership and baseline
- Branch: codex/wld0-binding-api
- Worktree: C:\Users\leo19\.codex\worktrees\wld0-binding-api\Star
- Base branch: codex/wbs-dev-integration at the branch HEAD when this brief is dispatched (currently 6cf09063); this includes the reviewed 9F4J.1, 9F4J.6 and WLD-0 audit commits. The coordinator has a separate pending privilege-test fix; it is independent of this API scope and will rebase/integrate after you finish.
- You own only the paths listed below. Do not edit root D:\Star or any other checkout.

# Files in this lane
1. crates/star-api-rest/src/group_api.rs
2. crates/star-api-rest/src/group_api/run_worktree_bindings.rs
3. crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs
4. db/migrations/2026-10-19-run-worktree-bindings.sql
5. docs/briefs/WLD0-BINDING-API-20261019.md

If repository evidence requires another file, stop before editing it and send the coordinator the exact path, reason, overlap risk, and smallest safe manifest amendment.

# Requirements
- Implement an authenticated, typed API for assigning, detaching, and reassigning a Worktree to a Project-owned Engineering Run using the existing versioned SCD2 binding contract.
- A Run can be hostless: remote repository/Branch metadata is optional and must not be required to bind a local Worktree. Keep the canonical navigation Project -> Engineering Run -> Worktree -> observed local Branch.
- Validate the active Project binding, tenant, Project authorization and Run authorization in the owner transaction. Reject cross-Project/tenant links, stale expected binding versions, stale Project binding versions, invalid state transitions, and missing/uncertain permission facts with fail-closed responses.
- Use durable command idempotency with exact request identity/payload conflict handling. Close the prior active revision and write the new revision, audit fact, and versioned Outbox event in the same owner transaction. Preserve immutable historical revisions and current uniqueness; do not write another bounded-context owner's tables directly.
- Rebind must be atomic within this owner and must not leave two active Run bindings for one Worktree. Detach must create a durable/versioned lifecycle fact. Use only existing supported schema and migration conventions; add a forward migration if needed, never rewrite an applied migration.
- Keep execution disabled. A binding mutation is not evidence of Host/K3s capacity, sandbox readiness, CLI availability, or production deployment.
- Add tests that exercise rejection and concurrency/idempotency boundaries using the repository's real test conventions. Do not use seed/mock success as production evidence. If live PostgreSQL is unavailable, mark DB execution not run; compile-only evidence must be labeled accurately.
- Add valid Cypher relationship headers to all new/changed source and migration files per the repo's conventions; run the canonical CGG guard when available.

# Acceptance and handoff
- Run changed-file rustfmt, the focused star-api-rest all-targets check/tests appropriate to the change, CGG, and git diff checks. Record exact commands and outcomes; do not claim gates you did not run.
- Update only this brief with the final handoff summary: commit(s), paths, tests/gates, database limitations, known gaps, and recommended integration.
- Commit only the listed files, with the repository-authorized Ulysses author identity. Do not rebase, merge, push, delete, or clean any branch/worktree; coordinator owns integration.
- Preserve the current user decisions in product copy: do not use third-party products as imitation/comparison language; real configured connector names are allowed. Keep the Multipass/K3s test inventory capped at two instances in every related acceptance statement.

# Read COMMENTS

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## Final handoff summary

- **状态**：API 源码 lane 可进入候选集成分支；PostgreSQL acceptance 和生产执行仍关闭。执行响应与 Outbox payload 均保持 `execution_admission_available: false`。
- **实现文件**：`crates/star-api-rest/src/group_api.rs`、`crates/star-api-rest/src/group_api/run_worktree_bindings.rs`、`crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs`、`db/migrations/2026-10-19-run-worktree-bindings.sql`。未保留 `work_items.rs` 的纯格式调整。
- **静态审查**：独立审查未发现 P0/P1；修复了 advisory-lock 错误分类：仅 SQLSTATE `55P03`/`57014` 映射为可重试冲突，其他数据库错误仍为内部错误，并加入单元覆盖。Run hostless 时允许远端 repo/Branch 元数据为空，binding 继续记录本地 Worktree repo 身份。
- **验证**：Rust 1.99 `cargo check --package star-api-rest --all-targets -j 4`（P2 修复后）通过；`cargo test --package star-api-rest --all-targets -j 4`（P2 修复前）通过；修复后 advisory-lock 错误分类单测通过；变更 Rust 文件 `rustfmt --edition 2021 --check`、CGG 和 `git diff --check` 通过。包级 `cargo fmt --package star-api-rest -- --check` 因本 lane 外已有格式差异未通过；没有格式化那些文件。
- **数据库限制**：没有连接或修改 PostgreSQL。迁移未执行；RLS、真实认证写入、事务回滚、幂等持久化与并发 advisory-lock 测试未验证。标记为 ignored 的数据库测试未运行。
- **审查和接手**：API worker 在最终自审前达到用量上限，由协调者接手；独立只读审查完成，协调者完成 P2 修复并复测。候选分支集成由协调者另行完成，提交号记入候选集成报告，不在此 brief 自引用。
