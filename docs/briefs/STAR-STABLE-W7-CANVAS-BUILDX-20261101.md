# Brief: STAR-STABLE-W7-CANVAS-BUILDX-20261101

**Agent**: worker
**Phase**: PHASE-STAR-STABLE-W7-CANVAS-BUILDX-20261101
**Created**: 2026-10-31 23:42:36

---

Implement the W7 Canvas Buildx release-control hardening in this registered lane only.

Base: dev@2a57fa0d329be830735d6657677236e195673362
Owned product file: .github/workflows/publish-star-canvas-services.yml
Registered report files:
- docs/briefs/STAR-STABLE-W7-CANVAS-BUILDX-20261101.md
- docs/reports/PHASE-STAR-STABLE-W7-CANVAS-BUILDX-20261101.md
- docs/reports/WBS-DISPATCH-STAR-STABLE-W7-CANVAS-BUILDX-20261101.jsonl

Verify the Buildx version and setup-buildx-action input against official Docker sources. If evidence supports an exact Buildx release, pin it in this workflow and add job-level concurrency keyed by the workflow, release tag/ref name, and matrix image, with cancel-in-progress false so a later dispatch cannot cancel a release already publishing. Tag immutability remains the responsibility of repository governance; this concurrency key does not prevent a tag from being retargeted. Preserve the existing three-service allowlist, filtered Cargo contexts, digest placeholders, and fail-closed verification. Do not run a build, authenticate to GHCR, publish images, change external GitHub protection settings, or touch deploy/k3s-local/Dockerfile (D2/W7 owner overlap).

If current official sources do not establish an appropriate fixed Buildx release or safe concurrency key, leave the workflow unchanged and document the blocker. Validate only with available static tooling and git diff --check; no hosted evidence. The report must include the repository-required sections: purpose, change matrix, verification, known gaps, subagent handoff, guard rules, sign-off, revision history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
