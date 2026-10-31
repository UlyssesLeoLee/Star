# Brief: STAR-STABLE-W2-CANVAS-WEB-ATTEST-20261031

**Agent**: worker
**Phase**: STABLE-RELEASE-W2-CANVAS-WEB-ATTEST
**Created**: 2026-10-31 15:11:33

---

Implement the Canvas image attestation and reproducible Web build evidence slice for the stable-release WBS.

Ownership is exclusive to .github/workflows/publish-star-canvas-services.yml, the new .github/workflows/star-stable-web.yml, and the registered brief, report, and dispatch JSONL. Do not modify package manifests, lockfiles, Cargo files, WBS, D2/D4/W8 files, main, remotes, or another worktree. Do not create tags, publish assets, deploy to Vercel, or push.

User-confirmed release artifacts are Desktop, Web/Vercel, and three Star Canvas service images. This lane must not claim that a CI Web artifact proves an actual hosted Vercel deployment. Canvas publishing remains a manually approved action bound to the star-stable-release environment; preserve or strengthen that gate. Attest the actual pushed image digests, and make the Web build generate an SPDX SBOM tied to the produced build artifact. Keep permissions minimal and every third-party action pinned to a verified full commit SHA. The workflow must fail closed when the artifact, digest, SBOM, or attestation input is missing.

Use current official GitHub and npm CLI documentation. Verify action pins against their official upstream repositories. Inspect the exact existing workflows and output semantics before editing. Validate YAML, trigger/approval conditions, per-job permissions, SHA pins, image digest subject binding, Web artifact/SBOM binding, and run git diff --check. Record exact local checks and clearly leave hosted CI, actual Vercel deployment/consumer verification, GHCR publishing, and external environment reviewer/branch protections unverified. Do not claim release readiness.

Commit only the manifest files with author Ulysses <ulysses@mavis.local> after self-review. Do not merge or clean anything.

Graph evidence: broad CodeGraph queries returned unrelated source symbols; workflow configuration is a non-code scope, and no check_index_coverage tool was available. Context7 references consulted: official GitHub artifact attestations require the binary subject path or a container name plus digest, with attestations:write and id-token:write permissions; npm ci enforces lock/package consistency and npm sbom emits SPDX or CycloneDX.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
