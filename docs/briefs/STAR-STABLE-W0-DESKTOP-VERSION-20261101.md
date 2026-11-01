# Brief: STAR-STABLE-W0-DESKTOP-VERSION-20261101

**Agent**: codex
**Phase**: STAR-STABLE-W0-DESKTOP-VERSION
**Created**: 2026-11-01 03:33:06

---

# W0 Desktop release version binding

## Goal
Make the desktop stable build derive its installer version, runtime-reported version, and artifact identity from the exact canonical Git tag. The next planned release tag is v1.0.6, and Git tags remain the only version source. The already-integrated resolver is scripts/release/resolve_star_version.py.

## Scope and ownership
Only edit:
- .github/workflows/star-stable-desktop.yml
- crates/star-desktop/src-tauri/src/lib.rs
- this brief, the phase report, and this lane's WBS dispatcher audit

Do not edit root Cargo.toml, Cargo.lock, tauri.conf.json, package manifests, other workflows, or other lanes. Use a build-time Tauri configuration override for the version if needed; do not create a second committed version source.

## Required behavior
- Resolve the exact checked-out stable tag with scripts/release/resolve_star_version.py after checkout and before generated files make the checkout dirty.
- Pass the resolver's version, exact tag, and source SHA through the desktop build and evidence/artifact names.
- Ensure the Tauri installer version and Rust get_app_version() result use the same resolved version. Keep normal non-release development behavior by falling back to the crate version only outside the stable tagged build.
- Preserve the existing approval gate and tag-only behavior. Do not create/push v1.0.6, publish an artifact, or claim signing/notarization/installation verification.
- Add focused assertions where the owned files support them. Update the phase report with changed behavior, validation commands/results, remaining blockers, and W0 status.

## Acceptance
- Static workflow inspection shows the resolver is the only stable release version source and runs before build mutation.
- Desktop bundle version, runtime version, artifact identity, and recorded source SHA are tied to the same tag resolution.
- Existing lock, signing, provenance, and platform build controls remain intact.
- Run focused checks that are available. The known Rust 1.99.0 toolchain issue must be reported as a blocker if it prevents compilation; do not substitute another toolchain as release evidence.

## Integration boundary
Work only on this lane from dev@6c394465635d86894312700ed79eb583c79bd804. Commit the lane result and report. Do not rebase, merge, push, create release tags, or alter any other branch/worktree; the coordinator will review and integrate accepted commits.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
