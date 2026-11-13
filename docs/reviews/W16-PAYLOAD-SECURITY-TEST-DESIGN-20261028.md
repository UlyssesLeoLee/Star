# W16 Payload Security Test Design Review

## §0 Purpose

Record the review of the W16 security test-design update. This is a design review only; it does not establish that application code, database migrations, identity providers, or KMS providers implement or pass the specified behavior.

## §1 Change Matrix

| File | Reviewed change | Result |
|---|---|---|
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` | Version v1.8; adds W16 unit/integration test rows, E2E-COLLAB-12..19, PT-10 with W14 profile gate, W16 requirement traceability, topology notes, and revision history. | Scoped to the assigned test-design file; all new cases remain design-only / OPEN / NOT_RUN. |
| `docs/reviews/W16-PAYLOAD-SECURITY-TEST-DESIGN-20261028.md` | This review report, including case-to-requirement mapping and remaining dependencies. | New assigned report. |

## §2 Verification Summary

- Reviewed the dispatcher brief and the W16 SRS/BD companion files on the lane base.
- `rtk proxy git diff --check`: passed with no whitespace errors.
- Tests, builds, database checks, provider calls, and runtime security checks were not run.
- No implementation or security behavior is claimed as verified.

## §3 Open Dependencies

- Production issuer/device claims, enrollment proof, revocation persistence, and bounded SSE invalidation need implementation and target-environment evidence.
- Authorization-code and refresh-token atomic single-use behavior needs the auth-owner transaction implementation and real PostgreSQL concurrency evidence.
- A concrete approved provider/profile, owner persistence, protected envelope read path, and secure logging/Outbox boundaries remain prerequisites for the KMS cases.
- Rotation, retention, and consistent DB/blob/provider backup restore need operational procedures and evidence.
- PT-10 workload, provider baseline, resource budget, and thresholds must be fixed by the W14 Collaboration Profile before measurement; no values were invented here.
- The deployment test path must use Neon for Vercel or Multipass-managed K3s for self-hosted validation. The self-hosted test inventory is capped at two Multipass instances; two-node Raft is not HA, and Docker Desktop is not a prerequisite.

## §4 Subagent Failures

No subagents were dispatched for this bounded document lane. No delegated work failed.

## §5 Guardrails

- E2E-COLLAB-12..19, PT-10, and the added UT/IT rows are planned tests, not executed tests.
- PT-09's existing conflict-commit-to-both-participants-and-first-bounded-diff p99 ≤ 1 second criterion remains unchanged.
- Provider failures, identity uncertainty, AAD mismatch, and tampering must remain fail-closed in future implementation and acceptance evidence.
- Only the two paths listed in §1 are owned by this lane; the brief and dispatcher audit remain outside its commit.

## §6 Sign-off

| Role | Reviewer | Status |
|---|---|---|
| Architecture / test-design review | Mavis (Codex coordinator review) | Reviewed; accepted for integration (design only) |
| Implementation, SRE, database/provider operation | Not performed in this lane | Open |

## §7 Revision History

| Version | Date | Editor | Change |
|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Initial review record for the W16 test-design delta; explicitly records design-only status and open implementation/provider dependencies. |
