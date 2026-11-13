# W3 Desktop Consumer Verification Review

- **Status:** ACCEPTED AS A PARTIAL LANE; W3 remains OPEN/BLOCKED
- **Review basis:** codex/star-stable-w3-desktop-consumer-20261103 at cffb4eef4793f4e5057bb4e9f85ccdfb26050493
- **Target baseline:** dev@391ac9029662735e13741ee92464fade1acb0667
- **Reviewer:** Architecture review by Mavis

## §0 Purpose

Independently review the consumer gate for Desktop installers and the correction made after the coordinator found a mismatch between uploaded artifact layout and search depth.

## §1 Change matrix

| Area | Finding |
|---|---|
| Desktop workflow | Adds a separate approved-tag consumer job that downloads the Linux, Windows, and macOS artifacts; checks resolver identity; verifies SLSA provenance and SPDX 2.3 attestations for each installer; requires the expected Rust and frontend SBOM roots. |
| Artifact layout correction | Upload paths share the crates/star-desktop ancestor and retain src-tauri/target subdirectories. The consumer now recursively searches below each artifact root; the former top-level-only search would have found no installers. |
| Lane records | Phase report and dispatch receipt record the review correction and the remaining hosted-evidence gates. |
| Review outcome | Accept as partial W3 progress. The consumer has not run against a real hosted release artifact or attestation. |

## §2 Verification summary

- The registered-base Git diff check passed with no whitespace findings.
- Static review confirmed all installer types are searched recursively and that an empty platform subject set fails closed.
- The multi-path layout behavior is documented in the official [upload-artifact v7 implementation reference](https://github.com/actions/upload-artifact/blob/main/_autodocs/api-reference/shared-search.md); least-common-ancestor paths are stripped while deeper relative directories remain.
- GitHub CLI flags and JSON output constraints were checked against the [official attestation verification reference](https://cli.github.com/manual/gh_attestation_verify).
- No workflow parser/actionlint, hosted workflow, real release artifact, GitHub attestation, install, or rollback was run.

## §3 Known gaps

- SBOM predicate identity, installer digest, SPDX version, non-empty package/relationship lists, and both expected roots are checked statically; the consumer has not been exercised against real generated statements.
- SBOM package contents are not independently reconciled against every transitive dependency.
- W3 remains OPEN/BLOCKED; v1.0.6 remains NO-GO.

## §4 Subagent and handoff record

The initial worker's terminal session disconnected after its implementation; the coordinator continued in the same registered worktree and preserved existing lane files. Independent review identified the nested artifact-path defect, which was corrected in the lane before this review. No active worker remains in this lane.

## §5 Guard rules

Accept only the registered Desktop workflow and lane records. Do not infer release readiness from static checks. Keep W3 open until hosted provenance and SBOM consumers pass for real v1.0.6 artifacts and the remaining signature/install/rollback gates are closed. Do not publish, deploy, tag, push, or modify main.

## §6 Sign-off

| Role | Review |
|---|---|
| Architecture | Mavis: accepted as partial progress; the recursive search matches the uploaded artifact layout. |
| Platform / SRE | Mavis: consumer is read-only and fails closed on absent or mismatched installer subjects. Hosted execution remains unverified. |
| Release owner | Codex/assistant: no release approval implied; v1.0.6 remains NO-GO. |

## §7 Revision history

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v0.1 | 2026-11-03 | Mavis | Added independent review, accepted recursive subject discovery, and preserved hosted and SBOM-content gaps. |
