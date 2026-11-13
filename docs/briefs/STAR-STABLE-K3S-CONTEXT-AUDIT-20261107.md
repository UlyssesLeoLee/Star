# Brief: STAR-STABLE-K3S-CONTEXT-AUDIT-20261107

**Agent**: codex
**Phase**: Star stable K3s build-context audit
**Created**: 2026-11-07 10:28:05

---

## Task
Independently audit the current K3s local image build path against the requirement that Star Docker/build artifacts contain only Star and explicitly selected runtime/build inputs, with no game or unrelated legacy payload.

## Current baseline
Branch codex/star-stable-k3s-context-audit-20261107, based on dev@22b92756ad762f27297c493760d95435ef98f2d1.

## Scope
Only these four manifest paths may be written:
- docs/briefs/STAR-STABLE-K3S-CONTEXT-AUDIT-20261107.md
- docs/reports/PHASE-STAR-STABLE-K3S-CONTEXT-AUDIT-20261107.md
- docs/reviews/STAR-STABLE-K3S-CONTEXT-AUDIT-20261107.md
- docs/reports/WBS-DISPATCH-STAR-STABLE-K3S-CONTEXT-AUDIT-20261107.jsonl

## Required work
1. Inspect the current deploy/k3s-local/Dockerfile template, .dockerignore, scripts/automation/star_k3s_context.py, its focused test definitions, and deploy/k3s-local/build-and-deploy.sh. Report file/line evidence for the generated workspace/crate allowlist, rejection of game/legacy paths and unlisted files, build invocation inside the Multipass guest, and any remaining payload gaps.
2. Distinguish static source assertions from a generated-context inventory and a final-image inventory. Do not claim artifact-level exclusion unless that exact context/image was built and inspected. No tests or builds are to be run in this lane.
3. State accurately that the repo's path uses Multipass/K3s, not Docker Desktop. Keep the test inventory limit at two retained Multipass instances in any future runtime acceptance plan.
4. Write a phase report with the required §0–§7 structure, a separate review report, and a brief that proposes the precise WBS follow-up. Do not claim the WBS itself was updated in this lane because its path is not in the manifest and overlaps a protected candidate claim.

## Boundaries
Do not edit application code, Dockerfiles, .dockerignore, test files, WBS files, main, remote refs, or other worktrees. Do not inspect, copy, alter, or clean the protected Rust candidate worktree. Do not run tests, Docker/Podman, Cargo, database, VM, cloud, build, release, or deployment commands. Do not merge or clean up. Commit only the four manifest paths and report static checks.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## Audit findings and exact WBS follow-up

Static source review found that `scripts/automation/star_k3s_context.py` derives a filtered Cargo workspace from the active non-dev dependency closure, writes per-crate Dockerfile `COPY` entries, and validates the generated file set before transfer to the selected Multipass guest. Source also declares checks for forbidden game/legacy names, symlinks, unexpected root entries/files, and context digest/archive inventory drift. The test file defines assertions for these behaviors, but this lane did not run those tests.

This is source-level evidence only. No context was generated here, so there is no fresh generated-context file inventory or digest. No guest build/image inspection was run, so there is no final-image layer/file inventory, SBOM, or provenance evidence from this lane. The current helper's static path targets an existing Multipass guest: the host shell uses the Multipass CLI, while the selected Linux guest must already provide a Docker build engine and K3s; it does not use Docker Desktop or manage VM lifecycle.

**Proposed WBS follow-up (W7 local API context filter row; WBS itself is outside this lane's write manifest):** append a dated static-audit entry linking this phase report and review, identify the source guard/test definitions as reviewed but not executed, set generated-context inventory and final-image inventory to `NOT RUN`, and keep W7 `OPEN`. Require a separately authorized acceptance lane to (1) regenerate the context with the pinned Rust 1.99.1 toolchain and retain the sorted inventory plus digest, (2) build only inside an explicitly selected Multipass guest while keeping the retained Multipass inventory at no more than two instances, (3) inspect the resulting image filesystem/layers and produce/verify digest, SBOM, and provenance, then (4) record K3s rollout, health, and rollback evidence. Keep `deploy/k3s-local` development-only and outside the approved Canvas release image matrix unless the release owner changes that scope.
