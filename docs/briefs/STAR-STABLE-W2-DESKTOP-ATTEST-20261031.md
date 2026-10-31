# Brief: STAR-STABLE-W2-DESKTOP-ATTEST-20261031

**Agent**: desktop_sbom_fix
**Phase**: STABLE-RELEASE-WAVE2
**Created**: 2026-10-31 16:27:07

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
- Require the `workflow_dispatch` choice input `release_confirmation` to equal `APPROVED` before any attestation job can run; default to `NO` for routine validation. Each attestation job references `star-stable-release`; repository settings must configure required reviewers on that environment for the approval gate to apply before its OIDC token can be requested.
- Add a shared tag-release gate that compares root `[workspace.package].license` and the first-line `LICENSE` heading against the user-selected MIT license. The workspace currently says Apache-2.0 while `LICENSE` says MIT License, so the gate must block attestation until the responsible owners resolve that mismatch; this lane does not rewrite shared license metadata.
- Attest Linux `.deb`/AppImage, Windows MSI, and macOS universal DMG individually through their actual paths. These attestations establish provenance and SBOM predicates; they do not sign installers or notarize macOS output.
- The SBOM inventory is split by package ecosystem, and the npm lockfile view includes development/build dependencies. License/NOTICE closure and consumer-side verification remain release gates.
- All six build/attestation validators reject empty SPDX package and relationship arrays, require the expected root package and document DESCRIBES relationship, and enforce the manually checked npm lock floor of 225 packages. Rust validation requires only the selected star-desktop root (minimum 1); no Rust transitive package count is asserted because local rustc is unavailable. Hosted Rust SBOM generation and validation remain open.


## Review follow-up: shared SBOM validator
Replace the six oversized inline validators with concise workflow validation in the already-claimed workflow file; do not add a helper file or expand manifest paths. All six validation sites must reject empty SPDX package and relationship arrays, require the expected root package and its SPDX DESCRIBES link, and enforce the manually audited 225-package npm lock floor. Local Rust transitive enumeration is blocked by the unavailable rustc, so do not invent a Rust count; require the selected star-desktop root and report the gap. Run mutation/control tests against the exact validation logic (empty packages rejected, empty relationships rejected, valid Rust/frontend controls pass, 224 npm packages rejected, 225-package control passes), then update the phase report and WBS dispatch JSONL with actual commands/results. Preserve the current MIT fail-closed gate and all existing scope. Run CypherGraph Guardian and diff checks; commit only the four registered files. Do not merge, push, tag, release or clean the lane.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
