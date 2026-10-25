# Brief: VCD0-DETAILED-DESIGN-20261024

**Agent**: worker
**Phase**: VCD0-DESIGN
**Created**: 2026-10-24 19:02:08

---

Create the canonical Vercel detailed design for WBS VCD-0, using only current dev SRS v0.4 and review reports VCD0-BASIC-ADR-COMPATIBILITY-20261024.md, VCD0-DD-COMPATIBILITY-20261024.md, VCD0-TD-COMPATIBILITY-20261024.md. Do not copy or read unapproved D:\Star candidate designs.

Worktree: C:\Users\leo19\.codex\worktrees\wbs-vcd-detail\Star
Branch: codex/wbs-vcd0-detailed-review-20261024
Current dev base: a2313a2581e973e30de7f9047df7c73e613c4a39
Exclusive file: docs/detailed-design/DD-STAR-VERCEL-DEPLOY-001.md. Do not edit any other file, including SRS/WBS/ADR/BD/reviews.

Rebase this private lane onto local dev before writing (its existing review commit is already an ancestor of dev). Design one optional Vercel frontend/BFF hosting path. Existing Star Owners remain authoritative for Project/Run/Worktree, current authorization, durable facts, Schedule, BI, task/Agent execution, native Hooks, Canvas/realtime, local checkout and Multipass/K3s. No second Owner, database, migrator, queue/worker, scheduler, VM-only provider, or execution plane. Preview credentials/data are isolated and cannot cause production Owner/Schedule/Outbox side effects. Production BFF uses the real host-session contract, same-origin allowlist and current Run authorization; never infer identity from headers. Ambiguous mutation status remains state_unknown until the same Owner/idempotency key proves status. Detail owner-local transactions, idempotency, audit and Outbox/Inbox only where mapped to current Star contracts; do not invent APIs/tables. Rust endpoints are conditional on VCD-2 and WLD-0/WLD-6; Rust Runtime is Beta and Rust-specific feature limits/WebSocket remain unknown. General WebSocket GA is not Rust evidence. Cron is best-effort wake-up only; specify missed/duplicate/overlap/failure/catch-up/rollback reconciliation, immutable run_as_actor_id recheck and current RunInfrastructureBinding resolution, never capacity reservation or Schedule truth. Keep PTY/noVNC/Canvas WSS, CLI, hooks, long-lived consumers, Multipass manager and local Worktree FS on Star runtimes. Multipass tests max 2 retained inventory records, locked launcher, fail closed on unknown/full, no purge, no Docker Desktop. No new IDs or unsupported thresholds; no competitor product names in product-facing text.

Write complete Draft/unapproved DD with owner boundaries, flows, environment separation, auth, endpoint allowlist, state/failure semantics, Cron, deployment/rollback, security, resource budgets, observability, traceability, gaps and revision history. Mark 0 implemented/NOT_RUN and do not close VCD-0. Review exact diff; run RTK git diff --check; commit only this file with author per AGENTS.md. Report SHA, path, validation, unresolved items; do not merge or push.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
