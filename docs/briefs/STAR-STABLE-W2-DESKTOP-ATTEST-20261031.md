# Brief: STAR-STABLE-W2-DESKTOP-ATTEST-20261031

**Agent**: worker
**Phase**: STABLE-RELEASE-W2-DESKTOP-ATTEST
**Created**: 2026-10-31 15:06:51

---

Implement the W3 Desktop artifact SBOM and provenance slice for this stable-release WBS.

Ownership is exclusive to `.github/workflows/star-stable-desktop.yml` and the registered lane brief, report, and dispatch JSONL. Do not modify other workflows, package manifests, lockfiles, Cargo files, WBS, main, remotes, or another worktree. Do not create tags, publish assets, deploy, or push.

Baseline evidence: dev@4e6336391f1eec8c6425b2eea9f8543a4fce50da already builds unsigned Linux .deb/AppImage, Windows MSI, and macOS universal DMG; Node 24.22.2, Rust 1.99.0, action SHAs, frontend Vitest, and missing-artifact failures are present. This workflow currently does not produce artifact SBOMs or GitHub build attestations. User-confirmed release artifacts include Desktop, Web/Vercel, and three Canvas images. Stable release remains OPEN.

Use current official GitHub artifact-attestation documentation and verify any action pin against its official upstream repository. Attest each real bundle with its actual digest/subject path and generate an SPDX SBOM tied to the artifact/dependency closure. Keep permissions minimal at job scope, pin actions by full commit SHA, retain the existing unsigned status, and fail when SBOM/attestation inputs are missing. Distinguish provenance attestation from code-signing/notarization and do not claim hosted consumer verification without a run.

Run focused YAML/workflow/static checks and `git diff --check`; run tests only if the workflow change requires them. Record exact results and external gates in the phase report. Commit only the manifest files with author Ulysses <ulysses@mavis.local> after self-review; do not merge or clean anything.

Graph evidence: CodeGraph broad WBS queries returned irrelevant code symbols; there is no `check_index_coverage` tool available. This lane is workflow configuration, so inspect only its exact workflow and official GitHub docs. Context7 documentation consulted: GitHub attestations use `actions/attest@v4`; binary subjects use `subject-path`, with required `attestations: write` and `id-token: write` permissions. Verify exact current action SHA before use.

## Implementation decisions

- Produce separate SPDX 2.3 dependency documents for the selected `star-desktop` Cargo package and the locked Desktop frontend package. The Cargo generator is scoped by package name; npm reads only the committed package lock. Each document is attached to the exact platform bundle subjects with an SBOM attestation.
- Keep build/test jobs at read-only repository permissions. Move OIDC and attestation write permissions to three tag-dispatch-only jobs that download the already-built bundles and SBOMs, so build scripts and PR code do not receive those capabilities.
- Attest Linux `.deb`/AppImage, Windows MSI, and macOS universal DMG individually through their actual paths. These attestations establish provenance and SBOM predicates; they do not sign installers or notarize macOS output.
- The SBOM inventory is split by package ecosystem, and the npm lockfile view includes development/build dependencies. License/NOTICE closure and consumer-side verification remain release gates.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
