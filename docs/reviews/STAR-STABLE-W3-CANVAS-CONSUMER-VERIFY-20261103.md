# W3 Canvas Consumer Verification Review

- **Status:** ACCEPTED AS A PARTIAL LANE; W3 remains OPEN/BLOCKED
- **Review basis:** codex/star-stable-w3-canvas-consumer-20261103 at 1f022fa1b5e5dd503bbcaaa5fafbb45f8d737d89
- **Target baseline:** dev@391ac9029662735e13741ee92464fade1acb0667
- **Reviewer:** Architecture review by Mavis

## §0 Purpose

Independently review the Canvas digest-bound consumer verification change against the W3 release acceptance criteria.

## §1 Change matrix

| Area | Finding |
|---|---|
| Canvas workflow | Validates resolver identity and the three required digest records before calling the existing verifier on each immutable OCI subject; manifest rendering is downstream of those checks. Permissions are read-only and limited to contents, attestations, and packages. |
| Lane records | Brief, phase report, and dispatch receipt describe the implemented scope and remaining release gates. |
| Review outcome | Accept the provenance-consumer addition as partial W3 progress. It does not satisfy the full BOM/attestation acceptance. |

## §2 Verification summary

- The registered-base Git diff check passed.
- Static review confirmed that the job verifies each of the three fixed digest subjects before rendering the manifest.
- GitHub CLI documentation confirms gh attestation verify supports repository, source digest/ref, signer workflow, predicate type, and JSON output constraints: [official command reference](https://cli.github.com/manual/gh_attestation_verify).
- No workflow parser/actionlint, hosted run, real OCI attestation, image, or K3s deployment was executed.

## §3 Known gaps

- The producer sets sbom: true, but this consumer does not independently verify the Buildx SBOM attestation or schema/content of each SBOM.
- No real hosted build, image digest, GitHub attestation, deployment, or rollback evidence exists.
- W3 remains OPEN/BLOCKED; v1.0.6 remains NO-GO.

## §4 Subagent and handoff record

The Canvas lane completed with the four registered implementation/record paths. No uncommitted files remain. This review does not claim a hosted consumer run.

## §5 Guard rules

Keep this lane scoped to Canvas provenance verification. Do not mark W3 complete, publish, deploy, tag, push, or modify main based on this static review. Preserve the independent SBOM consumer gap for follow-up.

## §6 Sign-off

| Role | Review |
|---|---|
| Architecture | Mavis: accepted as partial progress; digest identity checks precede manifest rendering. |
| Platform / SRE | Mavis: read-only consumer permissions; real registry verification remains unobserved. |
| Release owner | Codex/assistant: no release approval implied; v1.0.6 remains NO-GO. |

## §7 Revision history

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.1 | 2026-11-03 | Mavis | Added independent review; retained the SBOM and hosted evidence gaps. |
