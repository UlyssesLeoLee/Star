# Brief: WBS-VCD0-TD-REMEDIATION-R3-20261024

**Agent**: worker
**Phase**: WBS-VCD0
**Created**: 2026-10-24 23:24:39

---

Work only in the canonical detail/test candidate lane at C:/Users/leo19/.codex/worktrees/vcd0-canonical-detail-test-20261024/Star and only edit docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md. Preserve the existing uncommitted T-049 work; do not reset or overwrite it. Read the independent review report docs/reports/PHASE-VCD0-TD-REVIEW-R2-20261024.md in this reviewer lane and the exact candidate T-049 before editing.

Resolve the four findings conservatively:
1. Remove remote file-browser WS from T-049. It is only a repository inventory fact, has no approved requirement/owner scope in this Vercel design, and must not be implied as available. Keep FR-VC-016 local Worktree filesystem disabled/fail-closed in Vercel.
2. Make the Star Gateway browser credential handshake and allowed Preview Origin an explicit prerequisite/design gate. Do not invent a protocol or accept actor/Run headers as identity. The future contract must be consistent with SRS FR-VC-008: no reusable bearer, cookie, or connection credential in URL, browser storage, logs, traces, terminal, or evidence. Specify that T-049 remains NOT_RUN and its positive path is ineligible until SRS/BD/DD record a real Star-authorized handshake, origin validation, current Run/capability authorization, and revocation behavior. Retain safe negative cases for missing/invalid identity and disallowed Origin only where the deployed contract can be tested; do not claim a test was executed.
3. For the remaining transports (terminal/session WS, ARG WS, Canvas WS, chat SSE), define protocol-specific positive assertions using the real Star Gateway and isolated no-production fixtures: an actual authorized non-mutating event/frame/heartbeat is observed, and any bidirectional channel completes the documented safe round trip. Use lifecycle/heartbeat limits only from the real Gateway contract; do not invent duration or latency thresholds. If the contract has no observable positive data event or lifecycle target, state that as a prerequisite gap and keep acceptance NOT_RUN.
4. Narrow recovery assertions to fresh authorization plus connection/recovery evidence. Remove the claim that no command/business side effect was duplicated unless the test actually triggers a specific allowed idempotent operation and checks its durable receipt; do not add such an operation in this bounded TD-only task.

Keep all cases initially NOT_RUN; keep the document Draft, VCD-0 OPEN, and all deployment/runtime claims unverified. Update only traceability and revision history in the owned file; do not create new SRS/AC identifiers. Check focused diff and whitespace only; do not run tests, deploy, access a Vercel account, or modify SRS/ADR/BD/DD/WBS/review reports. Do not rebase, merge, push, or delete worktrees. Commit only the owned TD file. Parent will review and integrate. You are not alone in the repository; accommodate other lanes and never revert their changes.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
