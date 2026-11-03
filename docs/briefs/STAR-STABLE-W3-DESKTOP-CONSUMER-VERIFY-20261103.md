# Brief: STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103

**Agent**: worker/w3-desktop-consumer-verification
**Phase**: STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103
**Created**: 2026-11-03 06:13:52

---

## Objective
Close the local implementation gap in W3 for Desktop artifacts: the workflow currently creates attestations but has no separate consumer job that downloads the produced installer packages and verifies those attestations. Add a read-only consumer-verification job after all three platform attestation jobs.

## Ownership and base
- Branch: codex/star-stable-w3-desktop-consumer-20261103
- Worktree: D:/orcaWork/Star/star-stable-w3-desktop-consumer-20261103
- Base: dev@391ac9029662735e13741ee92464fade1acb0667
- This recovery lane owns only .github/workflows/star-stable-desktop.yml, this brief, docs/reports/PHASE-STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103.md, and docs/reports/WBS-DISPATCH-STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103.jsonl. The coordinator owns docs/reviews/STAR-STABLE-W3-DESKTOP-CONSUMER-VERIFY-20261103.md and the central WBS update after all three W3 lanes are reviewed.
- This is an independent lane. Do not revert or overwrite other lanes' work.

## Required behavior
- Add an independent downstream job that requires the successful Linux, Windows, and macOS build + attestation jobs, and is gated by the existing approved version-tag condition.
- Download the same-run Linux, Windows, and macOS artifacts using the existing pinned artifact action and exact resolver-derived artifact names.
- Verify every release subject (Linux .deb and AppImage, Windows .msi, macOS .dmg) using the existing scripts/release/verify_attested_release.py. Pass exact repository, v-tag, source commit, file subject and SHA-256. The existing verifier constrains signer workflow, source ref/digest and SLSA provenance predicate; preserve its fail-closed behavior.
- For every installer, also run `gh attestation verify` with exact repository, source digest, `refs/tags/<tag>`, signer workflow `.github/workflows/star-stable-desktop.yml`, and SPDX 2.3 predicate URI `https://spdx.dev/Document/v2.3`. Parse its `--format json` output and fail closed on malformed/empty output, missing or wrong predicate, wrong subject SHA-256, non-SPDX-2.3 documents, empty package/relationship arrays, or absence of both described roots `star-desktop` and `star-desktop-frontend`.
- Give the verifier job only contents: read and attestations: read, plus GH_TOKEN from the workflow token. Do not grant id-token or write permissions to the consumer job.
- Fail if an expected artifact is missing or empty, if the input tag/source identities diverge, or any consumer verification fails. Do not convert the existing operator-facing guidance into a claim that verification ran.
- Keep CI checks separate from the manual stable release approval. Do not change signing, notarization, tag policy, or publish/release behavior. A successful local diff-check does not prove the hosted consumers ran.

## Constraints
- Update this workflow's existing Cypher relationship header and source hash to describe the verifier job.
- Add the PHASE report with sections §0 purpose, §1 change matrix, §2 actual validation (be explicit that no tests/hosted workflow were run; git diff --check only), §3 known gaps, §4 subagent failures (none), §5 guard rules, §6 five-role signoff, §7 revision history. Keep W5 OPEN and v1.0.6 NO-GO; consumer verification code is not evidence that a hosted release ran.
- Record a JSONL dispatch receipt in the lane's registered dispatch path. Reference this brief in the commit message.
- Do not add or run tests, do not edit the central WBS, do not modify main, push, create tags/releases, deploy artifacts, or touch other worktrees.
- Before commit, inspect the workflow diff manually and run only rtk proxy git diff --check. Report exact commands/results and all unrun gates.
- Commit only the listed lane files; use the repository's configured Ulysses author identity.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
