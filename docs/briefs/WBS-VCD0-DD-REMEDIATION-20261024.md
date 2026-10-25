# Brief: WBS-VCD0-DD-REMEDIATION-20261024

**Agent**: worker
**Phase**: wbs-vcd0-dd-remediation
**Created**: 2026-10-24 23:39:51

---

## Objective
Repair the detailed design based on the independent R2 review. Change only `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md`.

## Required changes
1. Close the R2 P1 gap: explicitly map SRS FR-VC-009..012 to already-existing Star-owned data/authorization/migration owners. Do not create a Vercel-owned database authority, new Star Owner, or new migrator. Keep the provider/owner mapping unresolved unless the current repository documents it. If no explicit approved mapping exists, state fail-closed behavior: no Vercel database connection, deployment capability, or execution is enabled.
2. Close the R2 P2 cross-document gap: include the TD T-049 secure Gateway handshake, Preview Origin allow/deny, current authorization, revocation, protocol-specific data flow, and chat SSE as explicit prerequisites in the relevant architecture/data-flow and traceability sections. Do not invent an issuer, token format, handshake, Origin policy value, or wire protocol. Reference the current TD T-049 and state the gate remains unresolved until its protocol contract is approved.
3. Preserve Draft, VCD-0 OPEN, and all execution/deployment/runtime evidence as NOT_RUN. Update only the DD revision/header as needed; do not claim implementation or acceptance.
4. Preserve existing user content and unrelated changes. Do not modify SRS, BD, TD, WBS, code, or other files. Commit the DD-only change on the current candidate branch, and report exact diff/whitespace validation. Do not merge, rebase, push, or clean any worktree.

## Review evidence
The R2 independent DD report is `docs/reports/PHASE-VCD0-DD-REVIEW-R2-20261024.md` on branch `codex/wbs-vcd0-detailed-review-20261024`. Its exact findings are: (P1) DD §5.2 lacks explicit mapping of FR-VC-009..012 to the existing Star Owner/data plane without adding a new authority; (P2) DD §7/§11 omits TD T-049 Gateway handshake/Preview Origin/current auth/revocation/protocol flow and chat SSE.

The target candidate is currently based on `c7065825a987090be1801eff6120c3dc8ec616b1`; branch `codex/vcd0-canonical-detail-test-20261024`. Recheck the live file and status before editing.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
