# Brief: W16-PAYLOAD-SECURITY-TEST-DESIGN-20261028

**Agent**: Codex W16 test-design worker
**Phase**: WBS-DISPATCH-W16-PAYLOAD-TEST-20261028
**Created**: 2026-10-28 04:59:07

---

Task: Add security-focused W16 test-design acceptance to the canonical Canvas test design.
Owner: This worker owns only docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md and docs/reviews/W16-PAYLOAD-SECURITY-TEST-DESIGN-20261028.md. The dispatcher owns the brief and JSONL audit file.
Branch / Worktree: codex/w16-payload-security-test-design-20261028 / C:\Users\leo19\.codex\worktrees\w16-payload-security-test-design\Star
Base branch / Base SHA: dependency child of codex/w16-payload-security-design-20261028 at 42948810; this parent lane contains SRS-W16-PAYLOAD-SECURITY-001, BD-W16-PAYLOAD-SECURITY-001, DD-W16-PAYLOAD-SECURITY-001, and WBS v0.69. Do not rebase/merge; parent integrates first, coordinator will rebase and integrate this lane after it.
Dependencies: W16 companion design at 42948810; existing PT-09 semantics must remain unchanged.
Files changed: only the two owned paths above; do not change SRS/BD/DD/WBS, source code, lockfiles, user main worktree, or other lanes.
Acceptance criteria:
1. Bump test design metadata/revision from v1.7 to v1.8, preserving all existing cases and honest OPEN/NOT_RUN status.
2. Expand the WLD-7 E2E plan with E2E-COLLAB-12 through E2E-COLLAB-19 covering (12) no trusted device claim/forged client device id is rejected and remains local_only; (13) authenticated enrollment plus device revocation blocks new reads/writes and closes or boundedly expires active SSE; (14) concurrent authorization-code and refresh-token replay yields exactly one successful consumer; (15) KMS/provider unavailable, denied, or unknown version fails closed without persisted plaintext, canonical revision, success receipt, or content-bearing Outbox, and no LocalMockKms fallback; (16) an authorized fresh server process can read an old envelope after restart using its pinned provider/profile/key version; (17) cross-tenant/Project/Run/resource/revision AAD substitution and ciphertext/tag tamper fail without plaintext; (18) key rotation/rewrap preserves authorized old-envelope reads until all live/backup references migrate and records auditable idempotent progress; (19) DB/blob/provider backup restore allows an authorized read while revoked identity remains denied.
3. Add unit/integration test design rows for claim mapping, atomic OAuth single-use consumption, and protector envelope/AAD boundaries. Design tests only; do not implement or run them.
4. Add PT-10 for bounded envelope/KMS overhead under a fixed versioned Collaboration Profile; capture KMS calls per coalesced checkpoint, RSS/CPU, latency percentiles, queue/buffer bounds, cancellation and outage recovery. Do not invent thresholds or workload sizes; keep NOT_RUN until W14 profile and workload are approved. Preserve PT-09 commit-to-read p99 semantics.
5. State the target environments accurately: Neon for the Vercel path; Multipass-managed K3s for self-hosted tests, at most 2 inventory instances; no Docker Desktop prerequisite. Two-node Raft is not HA.
6. Add a concise review report documenting files, case-to-requirement trace, open dependencies, and that no tests ran. Never say code, provider, DB, or security behavior is implemented or verified.
Validation: run only rtk proxy git diff --check; no tests/builds. Commit only the two owned paths; author may follow repository policy. Report exact commit and final worktree status to coordinator.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
