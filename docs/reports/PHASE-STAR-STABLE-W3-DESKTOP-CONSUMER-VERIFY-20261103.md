# Phase Report: W3 Desktop Consumer Verification

**Phase:** STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103  
**Branch:** `codex/star-stable-w3-desktop-consumer-20261103`  
**Base:** `dev@391ac9029662735e13741ee92464fade1acb0667`  
**Status:** Local workflow implementation complete; hosted verification not run. W3 remains OPEN/BLOCKED; W5 remains OPEN; v1.0.6 remains NO-GO.

## §0 Purpose

Add an independent read-only consumer job to the Desktop stable workflow. The job waits for Linux, Windows, and macOS bundle builds and their attestation jobs, downloads each same-run artifact, checks the tag/source identity, verifies SLSA provenance for every non-empty `.deb`, `.AppImage`, `.msi`, and `.dmg` using the repository's existing `scripts/release/verify_attested_release.py`, then independently verifies both Rust and frontend SPDX 2.3 SBOM attestations for each installer with constrained `gh attestation verify` calls and fail-closed JSON checks.

This change creates a consumer-verification step in the workflow definition. It does not prove that a hosted release run, artifact, provenance bundle, or verification has succeeded.

## §1 Change matrix

| File | Change | Result |
|---|---|---|
| `.github/workflows/star-stable-desktop.yml` | Added a separate approved-tag consumer job; downloads all three platform artifacts, requires matching resolver outputs and exact tag checkout, hashes every installer subject, verifies SLSA provenance through the existing release verifier, then runs a constrained SPDX 2.3 consumer query and parses its JSON. For each subject, require its matching digest and the two SPDX roots `star-desktop` and `star-desktop-frontend`, with non-empty package/relationship arrays. Updated the Cypher relationship header. | Implemented locally; hosted execution not run. |
| `docs/briefs/STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103.md` | Records lane scope and constraints. | Created before worker dispatch. |
| `docs/reports/PHASE-STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103.md` | Records implementation, evidence, remaining gates, and revision. | This report. |

The job uses `contents: read` and `attestations: read` only. It does not receive `id-token` or write permission. The existing manual `APPROVED` and version-tag conditions remain in force. The SPDX consumer constrains repository, source digest, exact `refs/tags/<tag>`, signer workflow, and predicate type `https://spdx.dev/Document/v2.3`; it parses the successful `gh --format json` response and fails if any verified record lacks the expected installer digest or a valid SPDX 2.3 root.

## §2 Validation summary

- `rtk proxy git diff --check` — passed; Git emitted only the existing CRLF-to-LF normalization warning for the workflow file.
- Automated tests, actionlint, local workflow simulation, GitHub-hosted runs, and release artifact generation were not run.
- The existing verifier constrains the repository, tag ref, source commit, expected artifact digest, signer workflow, and SLSA provenance predicate. The new workflow invokes that verifier for every installer subject.
- The additional GitHub CLI call constrains the same repository, source digest, tag ref, and signer workflow with the SPDX 2.3 predicate URI; parsed JSON must contain an attestation result, matching subject digest, SPDX 2.3 document, non-empty packages/relationships, and both expected described SBOM roots.
- GitHub documents `attestations: read` as a distinct read permission; the verifier job does not inherit the producer jobs' attestation-write permissions. [Workflow permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
- GitHub's official verifier documents source digest, source ref, signer workflow, predicate type, and JSON output containing verified attestation statements. [GitHub CLI attestation verification](https://cli.github.com/manual/gh_attestation_verify) · [GitHub SBOM predicate verification](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations#verifying-an-attestation-for-sboms)

## §3 Known gaps

1. The workflow has not run on GitHub-hosted runners. Availability and behavior of the artifact, token, and attestation APIs remain unverified at runtime.
2. The consumer job verifies SPDX 2.3 SBOM attestations and the expected Rust/frontend root documents, but it does not independently reconcile every SBOM package/license entry against Cargo.lock, package-lock.json, or upstream advisories, nor run a separate SPDX semantic validator.
3. No stable-tag artifact or v1.0.6 release exists as a result of this lane. The workflow's license metadata gate, exact Rust 1.99.0 build, Node 24 hosted build, signing/notarization, installation, upgrade/rollback, and final release approval remain separate open gates.
4. This lane does not verify Web/Vercel evidence, Canvas image attestations, Multipass/K3s deployment, or production rollout.
5. W3 remains OPEN/BLOCKED and W5 remains OPEN. The stable release remains NO-GO.

## §4 Subagent failure and handoff

- No nested subagent was dispatched by this lane.
- The original worker terminally disconnected twice. This recovery continued in the already-existing worktree, preserving its dirty workflow and untracked lane records without recreating, reverting, or overwriting them.
- No nested subagent was dispatched during recovery. The coordinator owns the independent review and central WBS synchronization.

## §5 Guard rules

1. Consumer verification runs only from an explicitly approved workflow dispatch on a version tag.
2. The verifier job depends on all three platform artifact jobs and all three platform attestation jobs.
3. The tag, version, and source commit emitted by Linux, Windows, and macOS must match exactly.
4. The workflow checkout must be the exact peeled commit of the resolved tag.
5. All downloads are named from resolver outputs, not mutable development labels.
6. Linux verification requires at least one non-empty Debian package.
7. Linux verification requires at least one non-empty AppImage package.
8. Windows verification requires at least one non-empty MSI package.
9. macOS verification requires at least one non-empty DMG package.
10. Every matching installer file is hashed and verified; a missing or empty subject fails closed.
11. Attestation verification binds the artifact digest to the exact source commit and tag ref.
12. SLSA provenance verification restricts the signer to the approved Desktop workflow and the SLSA provenance predicate.
13. SPDX SBOM verification binds the same installer SHA-256 to repository, exact tag ref, source commit, approved Desktop signer workflow, and the `https://spdx.dev/Document/v2.3` predicate.
14. Parsed SBOM JSON must include verified attestation records, match the installer digest, contain SPDX-2.3 with non-empty packages/relationships, and describe both `star-desktop` and `star-desktop-frontend` roots.
15. The consumer token is limited to repository contents read and attestation read; it receives no identity-token or write permission.
16. A skipped or failed producer/attestation job prevents consumer verification from succeeding.
17. A successful workflow definition is not hosted-run, artifact, signature, or release evidence; W3/W5 stay open until those gates produce evidence.

## §6 Signoff

| Role | Reviewer | Finding |
|---|---|---|
| Architecture | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Uses the existing release verifier and preserves the approved-tag release boundary. |
| SRE Lead | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Consumer job is read-only and fails closed on missing or mismatched installer subjects. Hosted execution remains unverified. |
| Platform | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | All four installer types require SLSA provenance and both SPDX 2.3 SBOM roots; hosted execution remains unverified. |
| Review Host | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Independent coordinator review is pending. |
| PM / Release Owner | Codex/assistant is accountable release owner | No release approval is implied; v1.0.6 remains NO-GO. |

## §7 Revision history

| Version | Date | Revised by | Change |
|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added an independent Desktop artifact-attestation consumer job. No tests or hosted workflow were run. |
| v0.2 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added digest/source/tag/signer-bound SPDX 2.3 SBOM attestation verification for each installer; parsed the verified statement JSON fail-closed for expected subject digest and Rust/frontend roots; recorded recovery after two terminal worker disconnects. W3/W5 remain open and v1.0.6 remains NO-GO. |
