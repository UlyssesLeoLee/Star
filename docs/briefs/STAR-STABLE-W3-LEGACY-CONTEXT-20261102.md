# Brief: STAR-STABLE-W3-LEGACY-CONTEXT-20261102

**Agent**: worker
**Phase**: W3-LEGACY-CONTEXT-20261102
**Created**: 2026-11-02 05:33:30

---

## Objective
Harden the Star Canvas image release context against non-Star legacy payloads. The current publish workflow has an allowlisted service/package context and rejects paths containing `game`, but its path/package checks do not consistently reject legacy names.

## Required behavior
- Keep the existing per-image/service allowlist, resolved Cargo dependency closure, lockfile identity/checksum verification, and current release tag/version gates.
- Reject any generated source path or Cargo package identity containing a forbidden game or legacy term, case-insensitively, before invoking the image build.
- Keep the rule deterministic and fail closed; do not remove the existing game checks or broaden the package allowlist.
- Add focused tests for forbidden game and legacy paths/package names, accepted Star Canvas names, and the workflow's use of the shared validation.
- Do not build/publish/deploy images or claim SBOM, provenance, digest, consumer, or rollback evidence unless those operations actually run.

## Scope and acceptance
- Own only the MAGOS-registered workflow, the new Star Canvas context validation helper and its focused tests, this brief, the W3 phase report, its dispatch JSONL, and the W3 review.
- Do not modify the stable-release WBS in this parallel lane because the W7 lane owns that shared file. Record the exact proposed W3 status and evidence in the phase report; the coordinator will synchronize the WBS after both code lanes are reviewed.
- Keep the release NO-GO. This change only closes one static context filter gap; it does not prove that all other build paths or final images contain only approved payloads.
- Run focused validation supported by the current environment, report exact outputs/results, and commit with this brief path in the message.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
