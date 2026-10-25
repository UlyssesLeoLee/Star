# Brief: VCD0-TEST-DESIGN-20261024

**Agent**: worker
**Phase**: VCD0-DESIGN
**Created**: 2026-10-24 19:02:08

---

Create the canonical Vercel test design for WBS VCD-0, using only current dev SRS v0.4 and review reports VCD0-BASIC-ADR-COMPATIBILITY-20261024.md, VCD0-DD-COMPATIBILITY-20261024.md, VCD0-TD-COMPATIBILITY-20261024.md. Do not copy or read unapproved D:\Star candidate designs.

Worktree: C:\Users\leo19\.codex\worktrees\wbs-vcd-test\Star
Branch: codex/wbs-vcd0-test-review-20261024
Current dev base: a2313a2581e973e30de7f9047df7c73e613c4a39
Exclusive file: docs/test-design/TD-STAR-VERCEL-DEPLOY-001.md. Do not edit any other file, including SRS/WBS/ADR/BD/DD/reviews.

Rebase this private lane onto local dev before writing (its existing review commit is already an ancestor of dev). The design is unapproved and not evidence tests ran: every test is NOT_RUN. Trace all current SRS FR-VC-001..021, NFR-VC-01..08, AC-VC-001..013 and GAP-01..11. Do not invent IDs/thresholds. Cover Preview secret/data isolation, protected access/promotion, real host-session auth, cross-Run denial, missing config/build failures, no production Run/Task/Schedule/Outbox mutation or Cron effect. Rust proof is isolated VCD-2; verify exact account/plan/runtime/MSRV and actual limits; general WebSocket GA is not Rust support. Keep terminal/Canvas real-time on Star. Cron cases must cover missed/never-arrives, failed/timeout without auto-retry, duplicates, overlap, later durable catch-up, one owner receipt, run_as_actor_id revoked before retry/resume, exact rollback target deployment definitions and Disable Cron Jobs setting reconciliation. Validate against actual authoritative Star Owner state/receipts, not mocks or memory. If Multipass is used: max two records including retained stopped/deleted, locked guarded launcher, unknown/full fails closed, never purge, no Docker Desktop; never create a third guest. Include security, resilience, resource/perf and operations evidence needs; all NOT_RUN. No alternate Owner/DB/scheduler, no competitor product names in product-facing text.

Review exact diff; run RTK git diff --check; commit only this file with author per AGENTS.md. Report SHA, path, validation and unresolved issues; do not merge or push.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
