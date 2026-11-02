# Brief: STAR-STABLE-W7-K3S-LOCAL-CONTEXT-20261102

**Agent**: codex
**Phase**: STAR-STABLE-W7-K3S-LOCAL-CONTEXT-20261102
**Created**: 2026-11-02 04:07:14

---

## Objective
Correct the existing local API build-and-deploy path so it uses the already-reviewed Star Cargo-closure context generator instead of staging the whole `crates/` directory. This is a payload-boundary fix only; it does not make the local API path part of the public stable-release matrix or complete multi-cluster infrastructure.

## Baseline evidence
- Current dev is `7f0b993bd9166ea97f0a04e95c359c50fcf1dea9`.
- `deploy/k3s-local/build-and-deploy.sh` directly copies `crates/` into a temporary Docker context and builds with the unexpanded source Dockerfile.
- `scripts/automation/star_k3s_context.py` already resolves the local Cargo closure, creates an explicit per-crate Dockerfile/context, rejects game/legacy paths, and validates the inventory/digest.
- D2 owns an uncommitted `deploy/k3s-local/README.md`; do not read, modify, copy, or clean it. README is intentionally excluded from this manifest.

## Allowed files
Only these registered paths: `deploy/k3s-local/build-and-deploy.sh`, `scripts/automation/test_star_k3s_context.py`, this lane brief/report/review/dispatcher JSONL, and `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md`. Do not edit `deploy/k3s-local/README.md`, the generator or Dockerfile, other worktrees, `D:\Star`, `main`, workflows, tags/releases, or remotes.

## Implementation requirements
1. Have the shell script create an empty child path under its private `mktemp -d` parent and invoke `star_k3s_context.py --repo "$REPO_ROOT" --output "$BUILD_CTX"`; do not copy workspace files itself.
2. Build using the generated context and generated `deploy/k3s-local/Dockerfile`, then preserve the existing `docker save` → K3s containerd `k8s.io` import → manifest apply → rollout flow. Do not introduce Docker Desktop as a prerequisite; do not claim that this path proves the supported Multipass multi-cluster runtime.
3. Add a focused regression test that checks the deploy script invokes the generator, builds from the generated context/Dockerfile, and has no broad `cp -r .../crates` staging. Keep the test deterministic and avoid invoking real Docker or Multipass.
4. Add a phase report and review with AGENTS §§0–§7, update the WBS to v1.39, and append truthful dispatcher audit records. W7 must remain OPEN/BLOCKED and v1.0.6 NO-GO until actual filtered context archive, final image/SBOM/provenance, Multipass/K3s build/deploy and consumer/rollback evidence exist. Record the README owner boundary.

## Validation
Run the focused new Python test, `bash -n deploy/k3s-local/build-and-deploy.sh`, and `rtk proxy git diff --check`. Do not run Docker/Multipass build/deploy, Cargo release builds, or touch inventory. If Python/Cargo dependencies block a check, record exact output. Commit the completed lane and return full SHA plus validation results.

## Boundaries
No changes to `main`; no push, fetch, tag, release, image publish, deployment, or cleanup of other worktrees/branches. Do not bypass MAGOS. Preserve the D2 README exactly.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
