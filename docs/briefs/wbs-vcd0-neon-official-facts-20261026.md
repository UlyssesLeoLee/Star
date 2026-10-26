# Brief: wbs-vcd0-neon-official-facts-20261026

**Agent**: researcher
**Phase**: WBS-VCD0-NEON-OFFICIAL-FACTS-20261026
**Created**: 2026-10-26 16:11:02

---

## Task
Produce a bounded, source-backed research report for the Vercel deployment design's open Neon platform-fact gap (SRS GAP-06).

## Scope and ownership
You own only:
- docs/reports/WBS-VCD0-NEON-OFFICIAL-FACTS-20261026.md
- docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md
- docs/automation-design.md
- scripts/automation/registry.md

Do not edit the canonical Vercel SRS, ADR, basic design, detailed design, or test design in this lane. WBS §22.4 requires WLD-7 Canvas design synchronization before Vercel design synchronization; this work is preparatory fact verification only and must not mark VCD-0 closed or change that order.

## Evidence
Read current dev copies of:
- docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md (v0.4)
- docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md
- docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md
- docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md
- docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md
- docs/reviews/WBS-VCD0-CONTRACT-GAPS-20261026.md
- docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md §§22.3–22.4

Use Neon primary documentation only for provider facts. Cite direct page URLs and distinguish documented facts from architectural inference. Cover, where official docs support it: pooled endpoint / transaction pooling behavior; direct endpoint use for migration tools requiring session state; available role/connection-string handling; branch-scoped connection strings; any documented connection limits or compute behavior relevant to the design. If a fact is not documented in an authoritative page, mark it UNKNOWN and do not infer it.

## Required report
Create a concise matrix with: claim, source page and URL, supported fact, affected Star document/section, design consequence, remaining verification. Explicitly state Neon does not establish Star Project/Run identity or authorization, Star-owned RLS context, owner transactions/outbox, chat SSE/Gateway, or target account/deployment acceptance. Preserve all NOT_RUN and OPEN statuses.

Update the WBS with a dated note that this is only preparatory provider-fact research and that Vercel design synchronization remains after WLD-7. Update docs/automation-design.md with one new task entry and scripts/automation/registry.md with one matching version row, following local formatting. Do not fabricate a custom automation runner.

## Validation
Run diff/whitespace checks only. Do not run tests, connect to an account/database, create cloud resources, expose credentials, or claim runtime acceptance. Commit your changes on this lane with a summary; leave all unrelated branches and worktrees untouched.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
