# Brief: VCD0-CANONICAL-ADR-BASIC-20261024

**Agent**: worker
**Phase**: VCD0-CANONICAL-DESIGN-20261024
**Created**: 2026-10-24 20:50:59

---

Create the canonical Vercel decision record and basic design for WBS VCD-0. This is documentation-only.

Authoritative source: latest dev SRS v0.4 and the three compatibility reports in docs/reviews: VCD0-BASIC-ADR-COMPATIBILITY-20261024.md, VCD0-DD-COMPATIBILITY-20261024.md, and VCD0-TD-COMPATIBILITY-20261024.md. The user's untracked D:/Star Vercel drafts are user-owned and read-only. The user asked for their review; use them only as evidence for issues already dispositioned in the compatibility reports. Do not copy their alternate dual-owner, second database, independent scheduler/worker, or alternate VM execution architecture into the canonical design. Do not modify any D:/Star file.

Worktree: C:/Users/leo19/.codex/worktrees/vcd0-canonical-basic-20261024/Star
Branch: codex/vcd0-canonical-basic-20261024
Base: current local dev at dispatch time.
Exclusive files:
- docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md
- docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md
Do not edit other files, including SRS, WBS, reports, briefs, or existing draft paths.

Architecture to encode: Vercel is an optional web presentation and same-origin BFF hosting path. Existing Star Owners remain the sole authority for identity/authorization, durable business facts, Schedule, Agent/Task execution, Hooks, BI, Canvas/realtime, local Worktree filesystem, and Multipass/K3s. Do not add a second Owner, database authority, migration authority, worker, scheduler, or VM execution plane. Preview is isolated from production secrets/data and creates no production Owner/Schedule/Outbox side effects. Rust HTTP functions are conditional on a successful isolated VCD-2 proof plus existing WLD-0/WLD-6 contracts; Rust Runtime is Beta and Rust-specific limits/WebSocket support remain unknown. General Functions WebSocket GA is not proof for Rust. Cron is a best-effort wake-up only; durable Schedule facts and reconciliation stay in Star. Preserve the Multipass cap of two retained test inventory records, lock-protected launcher, fail-closed unknown/full state, no auto-purge, and no Docker Desktop prerequisite. Product-facing text must not mention competitor product names.

Write complete but concise Draft/unapproved documents with decision owner and status, alternatives/consequences, scope, architecture/flows, trust and environment boundaries, capability gates, operational/resource constraints, risks, SRS traceability and revision history. Use only existing SRS IDs and approved thresholds; do not invent requirements or evidence. Mark untested behavior NOT_RUN and VCD-0 OPEN. Clearly distinguish platform facts, design decisions, and unresolved account/runtime proof.

Review exact diff. Run rtk git diff --check; no tests are needed for docs-only work. Commit only the two owned files with author per AGENTS.md. Report commit SHA, paths, validation, and unresolved decisions. Do not merge or push.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
