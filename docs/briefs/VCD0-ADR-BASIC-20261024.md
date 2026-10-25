# Brief: VCD0-ADR-BASIC-20261024

**Agent**: worker
**Phase**: VCD0-DESIGN
**Created**: 2026-10-24 19:02:08

---

Create the canonical Vercel decision record and basic design for WBS VCD-0, using only current dev SRS v0.4 and review reports VCD0-BASIC-ADR-COMPATIBILITY-20261024.md, VCD0-DD-COMPATIBILITY-20261024.md, VCD0-TD-COMPATIBILITY-20261024.md. Do not copy or read unapproved D:\Star candidate designs.

Worktree: C:\Users\leo19\.codex\worktrees\wbs-vcd-basic\Star
Branch: codex/wbs-vcd0-basic-review-20261024
Current dev base: a2313a2581e973e30de7f9047df7c73e613c4a39
Exclusive files: docs/adr/ADR-STAR-VERCEL-DEPLOY-001.md and docs/basic-design/BD-STAR-VERCEL-DEPLOY-001.md. Do not edit any other file, including SRS/WBS/reviews.

Rebase this private lane onto local dev before writing (its existing review commit is already an ancestor of dev). Keep this as one optional hosting path: Preview/Production frontend and same-origin BFF only; existing Star Owners remain the sole authority for authorization, durable facts, Schedule, Agent/task execution, Hooks, Canvas/realtime, Worktree filesystem and Multipass/K3s. No second Owner/DB/migration authority/worker/scheduler/VM execution plane. Rust HTTP remains a VCD-2 gated feasibility candidate conditioned on WLD-0/WLD-6 and verified Star Owner contracts. Preview secrets/data isolated; no production Owner/Schedule/Outbox side effects. General Functions WebSocket GA does not prove Rust support; Rust Runtime remains Beta/uncertain. Cron is best-effort wake-up/reconciliation only. Multipass is future VM provider; maximum 2 inventory records, guarded lock, fail closed on unknown/full, never purge automatically, no Docker Desktop prerequisite. Do not invent SRS IDs or acceptance thresholds. Product-facing text must not mention competitor product names.

Write complete concise Draft/unapproved documents, 0 implemented and NOT_RUN for missing evidence. Include decision owner, alternatives/consequences, scope, diagrams/flows if useful, mapping to SRS FR/NFR/GAP/AC and WBS VCD-0..4, open decisions and revision history. Do not close VCD-0. Review exact diff; run RTK git diff --check; commit only the two owned files with author per AGENTS.md. Report SHA, paths, verification, and unresolved items; do not merge or push.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
