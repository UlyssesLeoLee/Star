# Brief: VCD0-CANONICAL-DD-TD-20261024

**Agent**: worker
**Phase**: VCD0-CANONICAL-DESIGN-20261024
**Created**: 2026-10-24 20:50:59

---

Create the canonical Vercel detailed design and test design for WBS VCD-0. This is documentation-only.

Authoritative source: latest dev SRS v0.4 and the three compatibility reports in docs/reviews: VCD0-BASIC-ADR-COMPATIBILITY-20261024.md, VCD0-DD-COMPATIBILITY-20261024.md, and VCD0-TD-COMPATIBILITY-20261024.md. The user's untracked D:/Star Vercel drafts are user-owned and read-only. The user asked for their review; use them only for issues already dispositioned in the compatibility reports. Do not copy their alternate dual-owner, second database, independent scheduler/worker, or alternate VM execution architecture. Do not modify any D:/Star file.

Worktree: C:/Users/leo19/.codex/worktrees/vcd0-canonical-detail-test-20261024/Star
Branch: codex/vcd0-canonical-detail-test-20261024
Base: current local dev at dispatch time.
Exclusive files:
- docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md
- docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md
Do not edit other files, including SRS, WBS, reports, briefs, or existing draft paths.

Design one optional Vercel web presentation and same-origin BFF path. Existing Star Owners remain authoritative for identity, current authorization, durable facts, Schedule, BI, Agent/Task execution, native Hooks, Canvas/realtime, local checkout and Multipass/K3s. No second Owner/database/migrator/queue/worker/scheduler/VM-only execution plane. Preview has isolated secrets/data and zero production Owner/Schedule/Outbox side effects. BFF must use the real embedding-host session contract and current Project/Run authorization; never infer identity from client headers. Ambiguous mutation status stays unknown until the same Star Owner/idempotency key proves its outcome. Do not invent API/table names. Rust endpoints are conditional on VCD-2 and WLD-0/WLD-6; Rust Runtime Beta/feature support remains unknown. General Functions WebSocket GA is not Rust evidence. Cron is best-effort wake-up only; cover missed, duplicate, overlap, timeout/failure, durable catch-up, rollback/active-definition reconciliation, immutable run_as_actor_id revalidation on each trigger/retry/resume, and current RunInfrastructureBinding resolution. Cron never reserves capacity or owns Schedule truth. Keep PTY/CLI, native Hooks, long-lived consumers, Canvas realtime, Multipass manager and local Worktree filesystem on Star runtimes. Multipass tests must count no more than two retained inventory records; locked launcher; unknown/full fail closed; no auto-purge; no Docker Desktop.

In the DD, specify owner boundaries, flows, environment separation, authorization, bounded resource/cost budgets only where backed by existing SRS, failure/recovery semantics, deployment/rollback, security, observability/BI, traceability, open questions and revision history.
In the TD, trace every current SRS FR-VC-001..021, NFR-VC-01..08, AC-VC-001..013 and GAP-01..11 without inventing IDs. Include Preview isolation, real session auth, cross-Run denial, missing configuration/build failures, no production side effects, Rust VCD-2 feasibility, Cron failure/reconciliation cases, resource/security/recovery evidence. All unexecuted cases are NOT_RUN; the design is not implementation evidence. Product-facing text must not mention competitor product names.

Review exact diff. Run rtk git diff --check; no tests are needed for docs-only work. Commit only the two owned files with author per AGENTS.md. Report commit SHA, paths, validation, and unresolved issues. Do not merge or push.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
