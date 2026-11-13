# PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST

> **Status:** Lane implementation complete; awaiting coordinator review and local `dev` integration. Stable release remains **OPEN / NOT RELEASE-READY**.
> **Owner:** canvas-web-attest
> **Branch:** `agent/stable-release-wave2/canvas-web-attest`
> **Base:** `dev@4e6336391f1eec8c6425b2eea9f8543a4fce50da`

## §0 Purpose

Implement the W3 supply-chain evidence slice for the three Star Canvas service images and the Web build. Preserve the existing explicit release confirmation and `star-stable-release` environment gate. This lane does not publish, deploy, create tags, or assert that a workflow configuration proves a hosted run.

## §1 Change Matrix

| File | Change | Evidence boundary |
|---|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | Adds strict SHA-256 digest validation and a registry-published GitHub attestation for each image produced by the approved version-tag dispatch. Grants `attestations:write` and `id-token:write` only to the build job. Retains Buildx `sbom: true` and `provenance: mode=max`. | Attests the digest returned by the image build step. It does not prove an image was built, published, or independently verified. |
| `.github/workflows/star-stable-web.yml` | Adds an approved, version-tag-only Web evidence workflow: locked `npm ci`, unit tests, Next build, SPDX SBOM validation, build-evidence archive, SHA-256 file, artifact upload, and separate provenance/SBOM attestations. | The archive is a build-evidence bundle, not a deployable Vercel artifact or an actual hosted deployment. Archive metadata is normalized; bit-for-bit reproducibility has not been demonstrated. |
| `docs/briefs/STAR-STABLE-W2-CANVAS-WEB-ATTEST-20261031.md` | Records the bounded lane assignment and acceptance conditions. | Created and logged through the repository dispatcher before implementation. |
| `docs/reports/PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST-20261031.md` | Records implementation, checks, unresolved release gates, and review boundary. | This report. |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W2-CANVAS-WEB-ATTEST-20261031.jsonl` | Append-only dispatcher record for the lane brief. | No manual or synthetic dispatch record was added. |

The attestation action is pinned to immutable release commit `1e69f48acb82d1966a394da916b4c1698aa569d6` (`v4.2.2`). The official repository API resolves the immutable `v4.2.2` tag to that commit. Both workflows pass the actual image digest or archive path as the attestation subject; the Web SBOM attestation also supplies the generated SPDX document.

## §2 Verification Summary

| Check | Result |
|---|---|
| YAML parse of both workflows with PyYAML 6.0.3 | PASS |
| Bash syntax check for 10 workflow `run` scripts | PASS (`bash -n`; existing PowerShell steps were not parsed as Bash) |
| Required workflow triggers, confirmation/environment gates, least-privilege attestation permissions, and SHA-pinned third-party actions | PASS (static assertions) |
| Canvas image subject name omits tags; digest is validated as `sha256:` plus 64 lowercase hex characters | PASS (static assertions) |
| Web artifact path, lockfile-driven install, SPDX output validation, artifact upload, and provenance/SBOM subject binding | PASS (static assertions) |
| `git diff --check` and whitespace check over all lane files | PASS |
| Hosted CI, Buildx image build/publish, Vercel deployment, external environment reviewer/branch configuration, and consumer-side `gh attestation verify` | NOT RUN |

GitHub's [container attestation documentation](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations) specifies an untagged fully qualified image name, SHA-256 subject digest, `push-to-registry: true`, and `attestations:write` plus `id-token:write` permissions. The pinned [actions/attest v4.2.2 release](https://github.com/actions/attest/releases/tag/v4.2.2) resolves to commit `1e69f48acb82d1966a394da916b4c1698aa569d6` in the official repository API. npm documents that [`npm ci`](https://docs.npmjs.com/cli/v11/commands/npm-ci) validates package-lock/package manifest consistency and that [`npm sbom`](https://docs.npmjs.com/cli/v11/commands/npm-sbom) emits a dependency SBOM. These facts support workflow syntax only; they are not evidence of a successful run.

## §3 Known Gaps

1. No hosted workflow was run. No Docker image, Web archive, or attestation was generated in this lane.
2. `actionlint` was unavailable locally; YAML parsing, explicit structural checks, and Bash syntax checks do not replace GitHub-hosted workflow validation.
3. The GitHub `star-stable-release` environment exists in workflow configuration, but its reviewers and allowed-ref rules were not inspected or proven.
4. No Vercel project build/deployment, consumer-side digest check, signing, or rollback was performed. The Web archive is evidence of a lockfile-driven CI build only.
5. `npm sbom` records the npm dependency tree; it is not a complete legal-license review and does not inventory the runner OS, Rust, or all non-npm build inputs. Canvas Buildx SBOM/provenance settings likewise do not close per-dependency license/NOTICE obligations.
6. The frontend currently has game-named agent-view source imports (including `frontend/src/app/(app)/agent-view/page.tsx` and `frontend/src/components/agent-game/`). This lane cannot change frontend source. The Web evidence workflow does not prove those modules are absent from the release UI. The product-scope owner must resolve this before treating the Web payload as satisfying the user's exclusion of legacy game content.
7. The project-level MIT choice is confirmed, but Cargo workspace metadata still disagrees and is held by conflicting W8/dependency owners. Do not treat the Chart's MIT field or this workflow as closing W1 or third-party redistribution duties.

## §4 Subagent Failure Takeover

No nested agent was dispatched by this lane. The parent coordinator owns independent review, readiness recording, integration, and cleanup. No worker claim is being used as evidence in place of the checks above.

## §5 Guard Rules

1. Only the five files in this lane's registered manifest were changed.
2. No user-owned dirty worktree, W8 source, or dependency branch was modified.
3. No tag, release, publish, Vercel deploy, image push, or Git push was performed.
4. Canvas publishing still requires the explicit `APPROVED` dispatch input and a `v*` version tag.
5. The existing release environment gate remains in place.
6. Image attestations use the exact digest emitted by Buildx; no tag is used as the attestation subject.
7. Missing or malformed image digest causes the job to fail before attestation.
8. Web artifact creation fails when the Next build output or SBOM is missing or malformed.
9. Third-party actions added here use full commit SHA pins.
10. Attestation permissions are scoped to the jobs that create attestations.
11. The Web workflow uses `npm ci`; it does not rewrite or claim ownership of Web lockfiles.
12. The Web archive is named as evidence and does not claim to be a Vercel deployment.
13. No dependency license has been inferred from an SBOM field.
14. No successful hosted run or consumer verification is claimed.
15. The WBS remains OPEN until the outstanding production gates have direct evidence.

## §6 Signoff

| Role | Signoff / boundary |
|---|---|
| Architecture | Mavis lane self-review recorded; coordinator integration review remains separate. |
| SRE lead | No production or hosted-run evidence in this lane. |
| Platform | Static workflow configuration reviewed by lane owner; external environment configuration remains unverified. |
| Review facilitator | Parent coordinator review required before integration. |
| Product | Scope follows the user's confirmed Desktop, Web/Vercel, and three Canvas image release surface; no release approval is implied. |

## §7 Revision History

| Version | Date | Revision owner | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses (Mavis lane owner; coordinator review required) | Added digest-bound Canvas image attestations and a gated Web build/SBOM evidence workflow; kept hosted release and deployment claims open. | Stable release WBS wave 2 |
