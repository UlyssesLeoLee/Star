# Brief: vcd0-basic-review-20261024

**Agent**: reviewer
**Phase**: WBS-VCD0
**Created**: 2026-10-24 18:40:09

---

# Task: review the Vercel basic-design and ADR drafts against current dev

## Purpose
Audit the read-only Vercel drafts in `D:\Star` against the authoritative `dev` baseline and report which claims can safely be carried forward. This is a review lane; do not edit the source drafts or create replacement canonical designs.

## Read-only sources
- `D:\Star\docs\basic-design\BD-STAR-VERCEL-DEPLOY-001.md`
- `D:\Star\docs\adr\ADR-STAR-VERCEL-PARALLEL-001.md`
- In this lane, read `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md`, the Vercel section of `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md`, and `AGENTS.md`.

## Accepted architecture for this review
- Vercel is an optional frontend Preview/production delivery plane. Rust HTTP functions are only a feasibility candidate after isolated proof.
- Star's Rust owners and the Multipass-managed K3s execution plane remain authoritative. Test inventory remains capped at two Multipass records; Docker Desktop is not a prerequisite.
- Do not introduce a second independent Owner, database, worker, scheduler, or VM execution plane through the Vercel proposal. A non-K3s execution provider needs a separate explicit architecture decision and WBS acceptance.
- Vercel Cron is a best-effort wake-up/reconciliation signal; durable Schedule facts and authorization remain with Star owners.
- Preview must be isolated from production credentials, data, and owner side effects.
- Treat Rust Runtime Beta limits and feature support as unknown unless the current official evidence or a target-account test proves them. Do not infer Rust eligibility from Node/Python limits.
- Map recommendations to existing SRS FR/AC/GAP IDs. Do not invent new requirements or claim implementation.

## Deliverable
Write only `docs/reviews/VCD0-BASIC-ADR-COMPATIBILITY-20261024.md` in this lane. Include: review scope and source revisions; aligned content; conflicts with exact SRS/WBS constraints; unsupported claims or missing evidence; a minimal remediation list; and whether the draft can be selectively adapted to the current VCD-0 design set. Distinguish current dev facts from untracked main-worktree draft content. Do not copy/edit either source file.

## Validation
Check internal links/IDs and run `git diff --check`. This is documentation review only; do not run deployment or code tests, and report them as not run. Keep the review concise and evidence-linked.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
