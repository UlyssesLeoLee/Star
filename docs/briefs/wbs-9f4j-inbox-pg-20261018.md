# Brief: wbs-9f4j-inbox-pg-20261018

**Agent**: worker
**Phase**: WBS-9F4J-PARALLEL
**Created**: 2026-10-18 22:59:19

---

Task: Complete and review the existing 9F4J.6 BI owner Inbox PostgreSQL persistence test slice.
Owner: Assigned worker in this lane only.
Branch / Worktree: codex/wbs-9f4j-inbox-pg / this worktree.
Base: dev at 6f8d5b8f2d00b1515928c9a25e0762a45cf30e4e.
Manifest: crates/bi-owner-runtime/src/task_event_inbox.rs; crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs; docs/implementation-plans/WBS-RUN-LIFECYCLE-DELTA-20261018.md; this brief and its phase report only.
The existing source and test edits are inherited work; inspect and preserve them, do not revert them. This task covers only persistence boundary evidence and must not claim network commit-before-ACK, target database grants, or production RLS.
Acceptance: satisfy D:\Star/docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md section 4 Phase 9F4J.6 only to the extent proven by an actual disposable PostgreSQL database. Check schema/function signature, exact replay, identity/wire conflict, transaction durability and runtime privilege boundary. If no isolated database is available, keep the test explicitly gated/ignored and report that blocker. Never print/copy environment secrets. Create the missing lane delta with evidence and remaining gates.
Validation: use RTK-prefixed commands; Rust 1.99.0, format check and focused crate test/build with -j 4 as feasible. Run the ignored PostgreSQL test only if its dedicated DB is present and verified disposable. Reconcile Cypher headers and run the repository cgg guard if present. Report exact commands/results; never claim unrun tests.
Commit one reviewable lane commit only after validation. Do not rebase, merge, push, clean, or alter dev. The coordinator owns integration. This task is part of parallel work; do not revert or overwrite the platform lane or unrelated changes.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
