# Brief: STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103

**Agent**: worker/w3-web-consumer-verification
**Phase**: STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103
**Created**: 2026-11-03 06:13:52

---

## Objective
Close the local implementation gap in W3 for Web/Vercel evidence: the workflow currently attests the build-evidence archive and prints consumer instructions, but does not perform an independent consumer verification. Add a downstream consumer job that re-downloads the run artifact and verifies its attestation.

## Ownership and base
- Branch: codex/star-stable-w3-web-consumer-20261103
- Worktree: D:/orcaWork/Star/star-stable-w3-web-consumer-20261103
- Base: dev@391ac9029662735e13741ee92464fade1acb0667
- You own only .github/workflows/star-stable-web.yml and docs/reports/PHASE-STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103.md. The coordinator owns docs/reviews/STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103.md and the central WBS update after all three W3 lanes are reviewed.
- This is an independent lane. Do not revert or overwrite other lanes' work.

## Required behavior
- Add a separate downstream job after the existing build-and-attest job; keep the current manual APPROVED + version-tag gate.
- Re-download the exact build-evidence archive from the current workflow run using the existing pinned artifact download action. Verify that the expected archive and SHA file are present and non-empty.
- Use scripts/release/verify_attested_release.py with kind=web, exact repository, version tag, source commit and archive path. Compute and pass the archive SHA-256. The verifier constrains signer workflow, source ref/digest and SLSA predicate; preserve fail-closed behavior.
- Consumer job token must have contents: read and attestations: read only; provide GH_TOKEN from github.token. Do not grant id-token or write permissions.
- Check tag/source identity against the exact tag target before verification. Fail on missing artifacts, digest mismatch or failed attestation verification.
- Keep the user-facing summary precise: this verifies a run-produced build-evidence archive and does not deploy to Vercel or prove a hosted production deployment.

## Constraints
- Update this workflow's existing Cypher relationship header and source hash to describe the consumer job.
- Add the PHASE report with sections §0 purpose, §1 change matrix, §2 actual validation (be explicit that no tests/hosted workflow were run; git diff --check only), §3 known gaps, §4 subagent failures (none), §5 guard rules, §6 five-role signoff, §7 revision history. Keep W3 OPEN/BLOCKED and v1.0.6 NO-GO.
- Record a JSONL dispatch receipt in the lane's registered dispatch path. Reference this brief in the commit message.
- Do not add or run tests, do not edit the central WBS, do not modify main, push, create tags/releases, deploy artifacts, or touch other worktrees.
- Before commit, inspect the workflow diff manually and run only rtk proxy git diff --check. Report exact commands/results and all unrun gates.
- Commit only the listed lane files; use the repository's configured Ulysses author identity.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
