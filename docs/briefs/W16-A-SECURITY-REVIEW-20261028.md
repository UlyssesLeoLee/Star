# Brief: W16-A-SECURITY-REVIEW-20261028

**Agent**: verifier
**Phase**: W16-A-DEVICE-AUTH
**Created**: 2026-10-28 16:56:22

---

# W16-A independent authentication security review

Scope: read-only review of the current staged implementation in this isolated worktree, against docs/briefs/W16-A-DEVICE-AUTH-IMPL-20261028.md and the registered W16-A manifest. Inspect only the device enrollment, OAuth code/refresh consumption, device binding repository and migration, authenticated-user middleware, Canvas mutation authorization, and Canvas SSE invalidation paths.

Prior source evidence: current branch adds verified device claims, Ed25519 device proof, PostgreSQL device binding/generation, atomic OAuth authorization-code consumption and refresh-token rotation, and Canvas device-generation authorization. The A brief identifies required fail-closed revocation, tenant binding, one-winner consumption and real PostgreSQL validation. An attempted CodeGraph query reported that this worktree has no .codegraph index; do not call CodeGraph again here. Codebase-memory graph tools are not available to this reviewer; use exact source inspection.

Return only concrete findings with severity and file:line, including whether each issue is confirmed or needs live PostgreSQL evidence. Check for cross-tenant/device substitution, replay/concurrency, revoke-vs-request races, RLS/migration mismatch, stale JWT/SSE behavior, and any route that accepts legacy sessions for shared writes. Do not edit files, stage or commit, run Cargo, access secrets, or modify any other worktree. If no confirmed blocker is found, state the remaining evidence limits explicitly.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
