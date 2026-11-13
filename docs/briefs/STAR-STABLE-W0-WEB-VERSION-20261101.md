# Brief: STAR-STABLE-W0-WEB-VERSION-20261101

**Agent**: codex
**Phase**: STAR-STABLE-W0-WEB-VERSION
**Created**: 2026-11-01 03:33:30

---

# W0 Web release version binding

## Goal
Make the stable Web build-evidence workflow derive its artifact identity and recorded version from the exact canonical Git tag. The next planned release tag is v1.0.6; Git tags remain the only version source. This workflow is build evidence only and is not a Vercel production deployment.

## Scope and ownership
Only edit:
- .github/workflows/star-stable-web.yml
- this brief, the phase report, and this lane's WBS dispatcher audit

Do not edit frontend/package.json, package-lock.json, Vercel settings, other workflows, root Cargo metadata, or other lanes.

## Required behavior
- Resolve the exact checked-out stable tag with scripts/release/resolve_star_version.py after checkout and before build output is generated.
- Carry the resolver's tag, version, and exact source SHA into build-info.txt, the evidence bundle identity/name, and any version environment needed by the static Web build.
- Keep the existing explicit approval input, clean checkout requirement, pinned actions, lock-driven npm ci, SPDX generation, provenance attestations, and upload failure behavior.
- Preserve clear wording that the artifact is build evidence and does not prove a Vercel deployment.
- Do not change the package manifest's 0.1.0 metadata into a competing canonical version source.
- Add static/self-check assertions in the owned workflow if useful. Update the phase report with exact behavior, validation commands/results, and the remaining Vercel production deployment/consumer verification gap.

## Acceptance
- Only a stable version tag can produce the evidence bundle, and it is resolved by the shared tag resolver.
- Bundle metadata and artifact name carry the same tag, semantic version, and source SHA.
- Existing lock, SPDX, provenance, and approval controls remain intact.
- No Vercel deployment, tag, release, or remote branch is created.
- Run focused static checks that are available; do not treat a workflow file review as a hosted run.

## Integration boundary
Work only on this lane from dev@6c394465635d86894312700ed79eb583c79bd804. Commit the lane result and report. Do not rebase, merge, push, create release tags, or alter any other branch/worktree; the coordinator will review and integrate accepted commits.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
