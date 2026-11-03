# W3 Web Consumer Verification Review

- **Status:** ACCEPTED AS A PARTIAL LANE; W3 remains OPEN/BLOCKED
- **Review basis:** codex/star-stable-w3-web-consumer-20261103 at c88fa95bc8e4bec2e11df34866c160ee2a44ac6e
- **Target baseline:** dev@391ac9029662735e13741ee92464fade1acb0667
- **Reviewer:** Architecture review by Mavis

## §0 Purpose

Independently review the Web build-evidence consumer gate against the W3 acceptance criteria and preserve the boundary between a build bundle and a live Web deployment.

## §1 Change matrix

| Area | Finding |
|---|---|
| Web workflow | The consumer re-resolves the release identity, requires exact tag/source equality, downloads the exact same-run bundle, checks its SHA-256 file, runs the bounded SLSA verifier, and requires a successfully verified SPDX 2.3 attestation. |
| Identity constraints | gh attestation verify is constrained by repository, source digest, tag ref, signer workflow, and SPDX predicate type; the JSON assertion fails closed when it contains no verified SPDX 2.3 record. |
| Lane records | Brief, phase report, and dispatch receipt preserve the W3 state and hosted/deployment gaps. |
| Review outcome | Accept the consumer implementation as partial W3 progress; it does not prove Vercel deployment or a real release. |

## §2 Verification summary

- The registered-base Git diff check passed.
- Producer and consumer paths match: the producer writes the checksum beside the archive; the consumer checks that basename from the downloaded artifact directory.
- Static review confirmed the exact archive name, digest, release tag, source commit, signer workflow, and SPDX predicate are checked.
- GitHub CLI flag and JSON-result semantics were checked against the [official command reference](https://cli.github.com/manual/gh_attestation_verify).
- No workflow parser/actionlint, hosted workflow, actual attestation, Vercel deployment, or rollback was run.

## §3 Known gaps

- The SBOM attestation is checked for a verified SPDX 2.3 predicate; this lane does not independently reconcile the package inventory against the full Web build dependency closure.
- The workflow explicitly does not deploy to Vercel. A live deployment, hosted artifact, installation/rollback, and external CI evidence remain unverified.
- W3 remains OPEN/BLOCKED; v1.0.6 remains NO-GO.

## §4 Subagent and handoff record

The Web lane completed with its four registered implementation/record paths. No uncommitted files remain. The worker's static validation is recorded in its phase report and was independently rechecked here.

## §5 Guard rules

Accept this lane only as a build-evidence consumer. Do not describe it as a production deployment, close W3, publish, deploy, tag, push, or modify main from this review.

## §6 Sign-off

| Role | Review |
|---|---|
| Architecture | Mavis: accepted as partial progress; release identity and consumer checks are ordered correctly. |
| Platform / SRE | Mavis: verifier is constrained and the checksum path matches the producer. |
| Release owner | Codex/assistant: no release approval implied; v1.0.6 remains NO-GO. |

## §7 Revision history

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.1 | 2026-11-03 | Mavis | Added independent review and preserved hosted Vercel/deployment gaps. |
