# Brief: STAR-STABLE-W0-CANVAS-VERSION-20261101

**Agent**: codex
**Phase**: STAR-STABLE-W0-CANVAS-VERSION
**Created**: 2026-11-01 03:33:21

---

# W0 Canvas release version binding

## Goal
Bind all three Canvas service runtime versions, published image identifiers, and the active K3s release manifest to the same exact canonical Git tag. The next planned release tag is v1.0.6; Git tags remain the only version source. Reuse scripts/release/resolve_star_version.py already present on dev.

## Scope and ownership
Only edit:
- .github/workflows/publish-star-canvas-services.yml
- crates/canvas-engine/src/lib.rs and crates/canvas-engine/tests/smoke.rs
- crates/domain-canvas/src/lib.rs and crates/domain-canvas/tests/smoke.rs
- crates/canvas-realtime/src/lib.rs and crates/canvas-realtime/tests/smoke.rs
- deploy/star-canvas-services-k3s.yaml
- scripts/release/render_star_canvas_manifest.py
- scripts/release/tests/test_render_star_canvas_manifest.py
- this brief, the phase report, and this lane's WBS dispatcher audit

Do not edit Cargo.toml, Cargo.lock, Dockerfiles, unrelated deployment paths, other workflows, or other lanes. Do not touch deploy/k3s-local or the active D2/W7 owner drafts.

## Required behavior
- Resolve the exact checked-out stable tag with the existing release resolver before generating or modifying build inputs.
- Pass the resolved version into all three Canvas Rust builds so ServiceMetadata reports the tag-derived version while ordinary non-release builds retain a development fallback.
- Keep OCI tag, source revision, runtime-reported version, and the per-service digest records tied to the same resolver output.
- Make deploy/star-canvas-services-k3s.yaml a clearly non-deployable template with a version placeholder instead of v0.1 labels. Add a deterministic renderer that accepts the resolver output and the three immutable image digests, rejects missing/malformed values, and produces a manifest whose version and image digests agree.
- Wire the gated Canvas release workflow to render and upload the final manifest artifact from the three produced image digests. Do not claim a real publish or rollout was executed.
- Preserve the existing per-service allowlist, filtered contexts, license gate, pinned Buildx, queue: max, no-cancel behavior, SBOM/provenance, and approval gate. Do not edit root Cargo license metadata; W1 owns that blocked issue.
- Add focused renderer tests and update the phase report with exact changed behavior, validation results, remaining W0/W3/W7 blockers, and honest status.

## Acceptance
- Workflow resolves one exact stable tag and carries its version/source SHA into runtime builds, OCI references, and evidence.
- Renderer tests prove deterministic output and fail closed for malformed tag/version, missing or mutable image references, or placeholder remnants.
- Rendered manifest carries the same release version and three digests.
- Existing filter and release-control assertions remain intact.
- Run focused checks that are available. If the known Rust 1.99.0 toolchain prevents Cargo validation, report it; do not substitute another toolchain as release evidence.
- No release tag, image, deployment, or remote branch is created.

## Integration boundary
Work only on this lane from dev@6c394465635d86894312700ed79eb583c79bd804. Commit the lane result and report. Do not rebase, merge, push, create release tags, or alter any other branch/worktree; the coordinator will review and integrate accepted commits.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
