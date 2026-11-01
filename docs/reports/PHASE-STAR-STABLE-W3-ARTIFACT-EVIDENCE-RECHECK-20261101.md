# PHASE-STAR-STABLE-W3-ARTIFACT-EVIDENCE-RECHECK-20261101

**Audit date:** 2026-11-01

**Audited local baseline:** dev at 3726c977984c69792dc21f0be97228e7a33e7659

**Mode:** read-only static inspection and GitHub API queries
**Disposition:** W3 remains OPEN; stable release evidence is insufficient.

## §0 Purpose

Recheck actual hosted artifact, SBOM, provenance, signature, and independent-consumer evidence for Desktop packages, Web/Vercel, and the three Star Canvas service images. This audit distinguishes the exact local dev baseline from GitHub-visible dev. Workflow configuration is design evidence only, not proof of execution or artifact existence.

## §1 Evidence matrix

| Surface | Local workflow configuration | Hosted evidence | Assessment |
|---|---|---|---|
| Desktop: Linux .deb/AppImage, Windows MSI, macOS universal DMG | Per-platform bundles and Rust/npm SPDX SBOMs are uploaded. Artifact names end in unsigned. Separate approved-tag jobs configure provenance and bind both SBOMs to each installer subject. See star-stable-desktop.yml lines 201–212, 329–339, 451–461, 465, 499–564, 571–696. | No run for the exact baseline, v1.0.6 tag/release, artifact ID, installer digest, SBOM digest, attestation result, native signature, or consumer verification found. | NOT READY. Attestation jobs are configured but no result is evidenced. Native Authenticode or Apple signing/notarization invocation was not found in the workflow or Tauri config; packages are explicitly unsigned. |
| Web/Vercel | Workflow creates SPDX SBOM, reproducible evidence archive and SHA-256 sidecar, and configures provenance/SBOM attestations. It explicitly says it does not deploy to Vercel. See star-stable-web.yml lines 3–4, 99–105, 134–164, 180–182. | No run/artifact ID, archive digest, attestation result, Vercel deployment identity, deployed asset digest, or independent verification found. | NOT READY. The bundle is not evidence for the approved Web/Vercel production surface. |
| Canvas: canvas-engine | Guarded allowlist; BuildKit provenance and SBOM; immutable digest record; OCI image attestation; digest-pinned manifest configured. See publish-star-canvas-services.yml lines 48–49, 77–90, 138, 355–371, 431–444, 456–506. | No v1.0.6 run/image digest, registry artifact ID, attestation result, manifest artifact ID, or independent pull/verification found. | NOT READY. Configured evidence has no produced artifact result. |
| Canvas: domain-canvas | Same guarded allowlist, BuildKit SBOM/provenance, digest/attestation and manifest pipeline as canvas-engine. | Same missing hosted run, digest, attestation, manifest and consumer verification. | NOT READY. |
| Canvas: canvas-realtime | Same guarded allowlist, BuildKit SBOM/provenance, digest/attestation and manifest pipeline as canvas-engine. | Same missing hosted run, digest, attestation, manifest and consumer verification. | NOT READY. |

GitHub artifact attestations are supply-chain claims, not native installer code signatures. The current Desktop package names explicitly mark the outputs unsigned.

## §2 Validation and hosted evidence

### Local baseline and remote dev

- GitHub Actions query for exact local SHA 3726c977984c69792dc21f0be97228e7a33e7659 returned total_count 0.
- GitHub GET branches/dev returned remote SHA 40a84c1aaf804267ee25e02384e99276c3151906, protected=false, with required status checks disabled. It differs from the audited local baseline.
- Contents API lookups for all three locally audited workflows at remote ref dev returned 404. They are absent from the currently visible remote dev branch.
- The remote dev run list reports 497 historical runs. Its first page includes four runs on remote SHA 40a84c1aaf804267ee25e02384e99276c3151906: CI 40142045777 failed; License & Advisory Gate 40142045790 failed; legacy Canvas publication 40142045772 failed; Coverage 40142045770 succeeded. None is a run for the audited local baseline or its stable workflows.

### Tag, release, artifact, and attestation evidence

- GET git/ref/tags/v1.0.6 and GET releases/tags/v1.0.6 both returned 404.
- The releases page lists v1.0.5 through v1.0.0; all six objects have empty assets arrays. v1.0.5 was published 2026-09-21T21:16:25Z.
- GitHub Fetch rejected repository-wide Actions artifact listing and the user container-package listing with HTTP 400 INVALID_ARGUMENT because these endpoints are outside its allowed public repository/search endpoints. These failures make inventory unavailable through this connector; they do not prove an empty inventory.
- With no exact-baseline run, v1.0.6 tag, or concrete subject digest, no actual artifact/attestation could be independently queried or verified. No independent consumer verification record was found.

### Checks performed

Read the W3 brief, relevant local workflows/config, and dispatcher JSONL. Used targeted RTK-wrapped searches for uploads, SBOM, provenance, attestations, Vercel deployment, and Desktop signing configuration. No builds, product tests, workflow mutation, secret/environment-variable access, publish, deploy, VM operation, or remote write was performed. Documentation checks are recorded in the dispatcher JSONL.

## §3 Known gaps and required evidence

1. Publish the reviewed workflow files to the intended remote branch, then verify branch SHA and workflow contents before relying on hosted runs.
2. Create/confirm v1.0.6 only from the approved commit. Run the gated workflows and retain run IDs, artifact IDs, immutable digests, and verification output.
3. Desktop: obtain each installer and both SBOMs; verify attestations against downloaded bytes; add independently verifiable platform-native signing/notarization and consumer checks.
4. Web/Vercel: produce a real deployment from the approved commit and record deployment identity, source SHA, immutable output digest, SBOM/provenance, signature or equivalent deployment assurance, and independent verification.
5. For each Canvas image, record its pushed OCI digest; retrieve and verify digest-bound SBOM/provenance; verify the digest-pinned manifest; capture an independent pull-and-verify result.
6. Retain consumer verification for every approved surface. Workflow definitions, unrelated green runs, and missing API data do not satisfy this gate.
7. Recheck branch protection and required release checks; visible remote dev currently reports protection disabled.

## §4 Subagent failure and handoff

No child agents were dispatched in this bounded audit. There are no child-agent failures. Hosted workflow executions, artifacts, and independent consumer evidence remain outstanding as described in §3.

## §5 Release gates

| Gate | Result |
|---|---|
| All three Desktop platforms have artifacts tied to approved source | FAIL — no matching run or artifact IDs |
| Desktop SBOM/provenance verifies against downloaded installer bytes | FAIL — no digest or attestation result |
| Desktop native signing/notarization is verifiable | FAIL — outputs named unsigned; no signing invocation found |
| Web/Vercel deployment is tied to approved source and immutable output | FAIL — workflow does not deploy to Vercel |
| Web/Vercel deployment has source-bound signed SBOM/provenance or equivalent assurance | FAIL — no deployment or attestation result |
| Each Canvas image has immutable digest-bound signed SBOM/provenance | FAIL — no digest or attestation result |
| Independent consumer verification exists for every surface | FAIL — no records |
| **Overall W3** | **OPEN — do not mark release-ready or close W3** |

## §6 Sign-off

Evidence disposition only; this is not approval to publish or release.

| Role | Status |
|---|---|
| Architect | Mavis review: hold; local workflows absent from visible remote dev |
| SRE Lead | Mavis review: hold; no hosted evidence or independent consumer verification |
| Platform | Mavis review: hold; no Desktop signatures or Canvas digests |
| Review host | Mavis review: report accepted for evidence accuracy; release gate remains open |
| PM | Mavis review: hold; no v1.0.6 release evidence |

The accountable release owner retains release approval.

## §7 Revision history

| Version | Date | Revision | Reviewer |
|---|---|---|---|
| v0.1 | 2026-11-01 | Initial W3 artifact, SBOM, signature, provenance, remote-state, and consumer-evidence audit; W3 remains open. | Ulysses — Mavis review |
