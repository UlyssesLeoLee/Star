# Brief: WBS-WLD7-CANDIDATE-AUDIT-20261027

**Agent**: codex
**Phase**: wbs-progress-20261027
**Created**: 2026-10-27 06:50:05

---

Task: Produce a bounded, read-only review of the old WLD-7 Canvas documentation candidate against current dev so a fresh docs lane can selectively reuse valid content.

Owner/worktree: This worker owns only C:\Users\leo19\.codex\worktrees\Star-wbs-progress-20261027-canvas-audit. Base: dev@9494e842f06206c9112148f722bdbcb9756761fb.

Exact writable manifest:
- docs/briefs/WBS-WLD7-CANDIDATE-AUDIT-20261027.md (dispatcher-generated; do not edit)
- docs/reviews/WBS-WLD7-CANVAS-CANDIDATE-AUDIT-20261027.md
- docs/reports/WBS-DISPATCH-WLD7-CANDIDATE-AUDIT-20261027.jsonl (dispatcher audit; do not hand-edit)

Read-only source scope:
- Current dev docs: docs/requirements/SRS-CANVAS-AGENT-001.md, docs/design/BD-CANVAS-AGENT-001.md, docs/design/DD-CANVAS-AGENT-001.md, docs/test-design/TD-WLD1-REPOSITORY-BOUNDARY-001.md, current WBS §23.2-§23.4, and the integrated W16 audit report.
- Compare with the committed candidate branch codex/wld7-canvas-a12-alignment-20261025, commit 5e5463bb12cf34a53d779e63f7ddc1b139236b76. Do not write to that branch or its worktree. Its merge base with dev is 6b7543e17fd0f64f23f94ddea9807293167843aa. Observed doc-only diff summary against dev before this lane: SRS +117/-274, BD +61/-37, DD +113/-38, total +291/-349. Treat the sizable rewrite as candidate material, never as approved wholesale content.
- Product sources may be searched read-only only where needed. Parent CodeGraph evidence: group API has CanvasOutboxPoller and pollOnce callers in groupProjection; the prior W16 report gives scoped implementation findings. CodeGraph coverage validation is unavailable in this session; limit negative/exhaustive claims and identify every inspected path.

Review requirements:
1. Create a claim-by-claim matrix: candidate text/section, current dev/user architecture evidence, decision Keep / Adapt / Reject, reason, and the exact new-doc section where the accepted adaptation belongs.
2. Enforce current constraints: Project → Engineering Run → Worktree → local Branch; Run can be hostless; remote provider links are optional metadata; Canvas and task/work-item capabilities are siblings under the current Worktree/Run scope; Worktree/Run authorization remains authoritative; no product text may use competitor brands or imitation/benchmark claims.
3. Separate design targets from implemented behavior. Keep V2/V3 Run/placement contracts and Multipass/K3s resource constraints consistent with current WBS; do not infer implementation or test success.
4. Identify large deletions, renamed requirements, compatibility breaks, and any candidate content that conflicts with current WBS/AGENTS direction. Recommend the smallest coherent new SRS/BD/DD edits; do not copy files or edit SRS/BD/DD in this audit lane.
5. Do not read, copy, edit, stage, clean or overwrite D:\Star user-owned changed/moved/deleted files. Preserve both old WLD-7 worktrees and candidate branches.

Report sections: §0 scope, §1 change summary, §2 keep/adapt/reject matrix, §3 evidence and limits, §4 recommended doc changes and dependency order, §5 known gaps, §6 review/sign-off, §7 revision history. No tests/build/database/VM/runtime validation. Avoid competitor product brand names in the review text.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
