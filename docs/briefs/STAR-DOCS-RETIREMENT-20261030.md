# Brief: STAR-DOCS-RETIREMENT-20261030

**Agent**: worker
**Phase**: STAR-DOCS-RETIREMENT-20261030
**Created**: 2026-10-30 15:21:08

---

Synchronize Star's active architecture documentation and WBS with the decision that shipped container/deployment assets contain Star-supported services only. The retired canvas-game service is not current product scope. Preserve old decision/spec records as historical evidence, clearly label them retired/superseded, and do not erase their history.

Owned files only:
- AGENTS.md (make one concise, durable rule under the existing product/deployment constraints)
- docs/basic-design.md
- docs/requirements/SRS-STAR-CANVAS-GAME-001.md
- docs/design/BD-STAR-CANVAS-GAME-001.md
- docs/design/DD-STAR-CANVAS-GAME-001.md
- docs/implementation-plans/CANVAS-GAME-IMPL-PLAN-001.md
- docs/adr/0027-rust-pivot-agent-game.md
- docs/plans/plan-032-rust-pivot-agent-game.md
- docs/architecture/2026-09-29-upgrade/rust-app-end-research.md
- docs/architecture/2026-09-30-upgrade/star-desktop-p7-e2e-performance-guide.md
- docs/arch-live/01-crate-map.html
- docs/arch-live/05-worktree-canvas.html
- docs/arch-live/07-deploy-topology.html
- docs/arch-live/08-guardrails.html
- docs/worktree-group-guard.md
- new docs/implementation-plans/WBS-STAR-DELIVERY-CLEANUP-20261030.md
- new docs/reports/PHASE-WBS-STAR-DELIVERY-CLEANUP-20261030.md
- this brief and docs/reports/dispatcher/STAR-DOCS-RETIREMENT-20261030.jsonl

Requirements:
1. Update active architecture/index/topology and performance guide claims so the current supported Canvas services are canvas-engine, domain-canvas, and canvas-realtime; keep star-api-rest identified as the Star API. Do not claim retired game service or old five-service counts as current.
2. Mark the old game SRS/basic/detailed design/implementation plan/ADR/plan as retired historical records, not supported product scope. Preserve their contents and paths unless a minimal move is necessary; avoid breaking historical references.
3. Add one concise imperative AGENTS.md rule: Star container/image/CI/deployment/GitOps/operational assets must use Star-owned service identities and exclude retired game/demo components; historical records may retain old terms only when clearly marked as non-current.
4. Create a WBS with requirements, lane ownership, exact acceptance criteria, validation gates, dependencies, and status initially open. The phase report must use the project 7-part report structure, clearly mark pending evidence rather than claiming completion, and leave room for final results.
5. Do not edit the delivery or Rust/tooling files owned by the other lanes. Communicate any stale cross-references you find.
6. Commit only this lane on this worktree with author Ulysses <ulysses@mavis.local>. No push and no direct edits to dev/main.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
