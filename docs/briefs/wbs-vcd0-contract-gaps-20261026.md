# Brief: wbs-vcd0-contract-gaps-20261026

**Agent**: Codex subagent
**Phase**: wbs-vcd0-contract-gaps-20261026
**Created**: 2026-10-26 12:45:20

---

# WBS VCD-0 unresolved contract gaps audit

## Objective
Audit current canonical design and traceability for the hosted web deployment track, focusing on unresolved cross-layer contracts already listed in the active WBS. Produce a decision-ready gap report only; do not implement, merge or modify canonical design documents.

## Baseline
- Your lane is based on dev at 7a82c098fe1dc021c8702e468768d68e8b0ee03e.
- WBS: docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md, especially W0/W1/W2/W4/W8/W13/W16 and sections 18/21.
- Review the current canonical Vercel deployment SRS/ADR/basic/detailed/test design and related collaboration/identity data contracts found on dev. Check existing reports before deciding whether a gap is already closed.
- User decisions already recorded: Project has 0..N repositories; active Run/Worktree refs must be migrated before unlink; Star-native Project owns identity and membership; hostedless Runs remain supported; server push/event fan-out replaces fixed client polling; conflicts retain both drafts and allow reading the other proposal; Multipass/K3s is the local/CI deployment track, not Docker Desktop.

## Audit scope
1. Trace each deployment-specific requirement through SRS -> ADR/basic design -> detailed design -> test design -> WBS status.
2. Audit Project/Repository multi-binding and safe unbind/migration behavior, including Star-owned identity and authorization.
3. Audit session, owner, database/RLS, BFF/Gateway, and real-time delivery boundaries: avoid client polling; cover owner-commit wake-up, cross-instance delivery/recovery, authorization/revocation, origin/session handling and bounded resource use.
4. Audit collaboration conflict behavior: both durable drafts, visibility of the other proposal, explicit user resolution, owner revision/CAS, and the <=1000ms arrival-trigger trigger. Do not redefine this as commit-time spacing.
5. Separate documented design from implemented/validated behavior; identify contradictions, missing IDs, stale references, and evidence needed to close each.
6. Do not mention competing product names in product-facing copy; keep the report neutral.

## Output and ownership
- Write only this brief and docs/reviews/WBS-VCD0-CONTRACT-GAPS-20261026.md.
- Cite exact source paths, headings/line numbers, requirement/acceptance/test IDs, and existing evidence reports.
- Present a prioritized gap matrix (severity, trace chain, evidence, minimal next action, owner/dependency, acceptance evidence).
- Mark source/runtime/database tests as NOT_RUN unless directly observed; no broad claims based on document presence.
- Do not edit canonical SRS/ADR/BD/DD/TD, WBS, code, prototypes or other reports.
- Commit the brief and report on this lane.
- Another lane is auditing Canvas A12 in separate files; do not modify its files or revert anyone else's work.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
