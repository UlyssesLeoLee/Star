# Brief: STAR-STABLE-W3-CANVAS-CONSUMER-VERIFY-20261103

**Agent**: worker/w3-canvas-consumer-verification
**Phase**: STAR-STABLE-W3-CANVAS-CONSUMER-VERIFY-20261103
**Created**: 2026-11-03 06:13:52

---

## Objective
Close the local implementation gap in W3 for Star Canvas images: the release workflow currently generates image attestations and writes a verification command as guidance, but does not independently verify the digest-pinned OCI subjects. Add verification to the separate manifest-rendering consumer job.

## Ownership and base
- Branch: codex/star-stable-w3-canvas-consumer-20261103
- Worktree: D:/orcaWork/Star/star-stable-w3-canvas-consumer-20261103
- Base: dev@391ac9029662735e13741ee92464fade1acb0667
- You own only .github/workflows/publish-star-canvas-services.yml and docs/reports/PHASE-STAR-STABLE-W3-CANVAS-CONSUMER-VERIFY-20261103.md. The coordinator owns docs/reviews/STAR-STABLE-W3-CANVAS-CONSUMER-VERIFY-20261103.md and the central WBS update after all three W3 lanes are reviewed.
- This is an independent lane. Do not revert or overwrite other lanes' work.

## Required behavior
- Extend the separate render-release-manifest consumer job after it downloads the exact resolver output and all three digest records. Verify all three OCI image subjects before manifest rendering.
- Invoke scripts/release/verify_attested_release.py with kind=canvas for each recorded image_ref and digest. Bind verification to GITHUB_REPOSITORY, resolved tag and resolved source SHA. Use the existing resolver-derived immutable digest record; do not reconstruct mutable tags.
- Require contents: read, attestations: read and packages: read only; give GH_TOKEN from github.token. Do not grant id-token or write permissions to this consumer job.
- Fail if any required digest record is absent, inconsistent or not one of the approved Star Canvas image subjects, or if verification fails. Keep rendering and upload downstream of successful verification.
- Summary must distinguish cryptographic consumer verification of attestations from actual image deployment/rollout; do not claim K3s or Multipass validation.

## Constraints
- Update this workflow's existing Cypher relationship header and source hash to describe the consumer verification step.
- Add the PHASE report with sections §0 purpose, §1 change matrix, §2 actual validation (be explicit that no tests/hosted workflow were run; git diff --check only), §3 known gaps, §4 subagent failures (none), §5 guard rules, §6 five-role signoff, §7 revision history. Keep W3 OPEN/BLOCKED and v1.0.6 NO-GO.
- Record a JSONL dispatch receipt in the lane's registered dispatch path. Reference this brief in the commit message.
- Do not add or run tests, do not edit the central WBS, do not modify main, push, create tags/releases, deploy images, or touch other worktrees.
- Before commit, inspect the workflow diff manually and run only rtk proxy git diff --check. Report exact commands/results and all unrun gates.
- Commit only the listed lane files; use the repository's configured Ulysses author identity.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
