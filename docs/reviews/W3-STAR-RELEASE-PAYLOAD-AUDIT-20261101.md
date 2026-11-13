# W3 Star Release Payload Boundary Audit

**Audit date:** 2026-11-01

**Current dev baseline:** `a272e9794e1a1975781720c9b3775c914f47f546`

**Read-only W7 candidate:** `codex/star-stable-w7-k3s-context-20261031` at `c960b9193cc960ae3d85711472a0d4f31fedd0b3`

**Decision:** **W3 remains open. Do not treat the audited source state as release-ready.**

## Scope and method

This is a static audit of tracked Dockerfiles, Docker context assembly, container image references, Canvas publish configuration, and related K3s/Kargo/Helm manifests. The approved stable release scope is the desktop app, Web/Vercel, and the three Canvas service images; this review covers the container/build-context paths only. The Canvas release path is statically narrow, but the current dev baseline and W7 candidate both have a license metadata mismatch that makes the stable Canvas release job fail its own gate. The compiled Canvas version also remains `0.1.0` instead of deriving from the approved `v1.0.6` tag.

Reads were mediated through RTK (`rtk read`, `rtk rg`, and `rtk proxy git`). No Docker build, context archive, registry query, publish, deploy, or test was run. The audit therefore does not claim to have inspected a real BuildKit context archive, final OCI image, SBOM, or provenance statement.

## Container and deployment inventory

| Path / image | Classification | Static conclusion |
|---|---|---|
| `crates/canvas-engine/Dockerfile`, `crates/domain-canvas/Dockerfile`, `crates/canvas-realtime/Dockerfile` | Approved stable Canvas image build paths | Each Dockerfile uses digest-pinned Rust and Debian bases, consumes one generated service context, and copies only the built executable plus `LICENSE` into the runtime stage. The workflow allowlists exactly these three image names and generates a separate context for each. This is a static source-level pass only. (`.github/workflows/publish-star-canvas-services.yml:64, 94-128, 234-250, 279-290`; each Canvas Dockerfile lines 4-14.) |
| `deploy/star-canvas-services-k3s.yaml` | Stable release render template | Three Star images use required digest placeholders; the file says it is not directly deployable until successful release digests are supplied. It also pins an Envoy runtime image by digest. No actual Star image digest or release deployment was verified. (`deploy/star-canvas-services-k3s.yaml:1-4, 49, 118, 187, 346`.) |
| `deploy/k3s-local/Dockerfile` plus `deploy/k3s-local/build-and-deploy.sh` | Development/local K3s image; outside the approved stable image set | The script creates a temporary context and copies the whole `crates/` tree; the Dockerfile copies all of it into the builder stage. The root `.dockerignore` is not staged into that temporary context. The runtime stage copies only the `star-api-rest` executable, but the build context and builder input exceed the approved component/dependency closure. (`deploy/k3s-local/build-and-deploy.sh:18, 21-30`; `deploy/k3s-local/Dockerfile:1-13`; `.dockerignore:1-13`.) |
| `deploy/k3s-local/star-api-rest-deploy.yaml`, `deploy/k3s-local/envoy/deployment.yaml`, `deploy/k3s-local/bff-deployment.yaml` | Development/local manifests | They refer to `star-api-rest:local`, Envoy `v1.31.0` / `v1.31-latest`, and placeholder `bff:local`. They are not part of the three-image stable Canvas publishing workflow. (`deploy/k3s-local/star-api-rest-deploy.yaml:30-32`; `deploy/k3s-local/envoy/deployment.yaml:35-38`; `deploy/k3s-local/bff-deployment.yaml:42-45, 139-143`.) |
| `tools/star-flash-mock/k3s/envoy-deployment.yaml` | Mock/test infrastructure | It references a mutable Envoy `v1.32-latest` image and identifies its purpose as serving `tools/star-flash-mock`; it is not a Star stable release build context. (`tools/star-flash-mock/k3s/envoy-deployment.yaml:1-4, 35-38`.) |
| `deploy/gitops/kargo/kargo-dev-staging.yaml` | Dev/staging promotion configuration | Its warehouse tracks the three Canvas image repositories, and its stages are dev/staging promotion configuration. It is not a Docker build context or proof of a stable release. (`deploy/gitops/kargo/kargo-dev-staging.yaml:1-5, 23-56, 126-135`.) |
| `deploy/helm/star` | Deprecated placeholder | `Chart.yaml` identifies a deprecated chart with no active workload templates; it is not the current Canvas deployment path. (`deploy/helm/star/Chart.yaml:1-6`; `deploy/helm/star/values.yaml:1-4`.) |

The tracked-file inventory contains exactly four Dockerfiles and one `.dockerignore`; no tracked `Containerfile` or Docker Compose file was found. The three Canvas images are the only Docker build-and-push matrix entries in the audited release workflow.

## Findings

### P1 — Stable Canvas release is blocked by contradictory license metadata

At the audited current dev baseline, `[workspace.package]` declares `license = "Apache-2.0"` in `Cargo.toml`, while `LICENSE` starts with `MIT License`. The stable Canvas workflow requires the workspace license to be exactly `MIT` and the first license line to be exactly `MIT License`; the current `Cargo.toml` value therefore makes the tag release job throw at its license gate before context assembly. Each Canvas Dockerfile separately labels the output image `org.opencontainers.image.licenses="MIT"`, so the sources and declared image identity disagree. (`Cargo.toml:182-188`; `LICENSE:1`; `.github/workflows/publish-star-canvas-services.yml:68-76`; each Canvas Dockerfile:14.)

The W7 candidate changes neither `Cargo.toml` nor the license gate, so this remains true at candidate `c960b9193cc960ae3d85711472a0d4f31fedd0b3`. No release run was executed. Resolve the license identity across workspace metadata, license text, image labels, and generated SBOM before the release gate can pass.

### P1 — The application version is not derived from the approved Git tag

The root workspace package version is `0.1.0`; the Canvas crate manifests inherit that value. The context generator copies the workspace package metadata into the generated minimal manifest, while the publish workflow separately asks Docker metadata to create an OCI semver tag from the selected Git ref. The three services compile their reported version from `env!("CARGO_PKG_VERSION")`, so the OCI tag may reflect `v1.0.6` while the application version embedded in each binary remains `0.1.0`. (`Cargo.toml:182-188`; `crates/canvas-engine/Cargo.toml:7-12`, `crates/domain-canvas/Cargo.toml:7-12`, `crates/canvas-realtime/Cargo.toml:7-12`; `.github/workflows/publish-star-canvas-services.yml:150-158, 272-275`; each service `src/lib.rs:67-77`.)

The release condition accepts any `refs/tags/v*` when manually dispatched with `release_gate_confirmation=APPROVED`; it does not bind this run to the specified `v1.0.6` ref. (`.github/workflows/publish-star-canvas-services.yml:13-20, 46-64`.) This checkout has no local `v1.0.6` tag ref, so the tag’s exact source commit was not verified here. Make the approved release ref explicit and ensure the version embedded in the binary and OCI metadata both derive from that immutable tag.

### P2 — The local `star-api-rest` context exceeds the component boundary if treated as a release path

`build-and-deploy.sh` copies `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and the entire repository `crates/` directory into a temporary context, then invokes Docker with that temporary directory. It does not copy `.dockerignore`. The Dockerfile then copies all staged crates into the builder image. The denylist in the repository-root `.dockerignore` cannot narrow this external temporary context. The final runtime stage copies only the `star-api-rest` binary, but that does not erase the overbroad context or builder input. (`deploy/k3s-local/build-and-deploy.sh:21-30`; `deploy/k3s-local/Dockerfile:4-10`; `.dockerignore:4-13`.)

This is currently a `star-api-rest:local` manual K3s path and is outside the approved three-image stable release scope, so it is not a blocker for those three images if it remains explicitly excluded. The repository still needs an owner decision and consistent classification. If it is ever proposed as a release image, first replace the broad context with an explicit, dependency-closed context and verify the actual context contents and final image.

### P2 — The legacy local build entrypoint remains broad and incompatible with the W7 template

The W7 candidate does add `scripts/automation/star_k3s_context.py` and focused tests. Static inspection shows the tool selects a Cargo package closure, expands the Dockerfile markers, computes a context digest, and rechecks a temporary tar archive before transferring it to a named Multipass guest. (`W7 c960b9193cc960ae3d85711472a0d4f31fedd0b3:scripts/automation/star_k3s_context.py:601-669, 672-688, 775-817`.) Its brief provides an explicit CLI path and its report says it generated a 333-file context and passed locked metadata checks with Rust 1.99.1; those prior results are self-reported by the candidate and were not rerun in this audit. The candidate report also says pinned Rust 1.99.0, the guest image build, and K3s deployment remain unverified. (`W7 c960b9193cc960ae3d85711472a0d4f31fedd0b3:docs/briefs/STAR-STABLE-W7-K3S-CONTEXT-FILTER-20261031.md:27-30`; `W7 c960b9193cc960ae3d85711472a0d4f31fedd0b3:docs/reports/PHASE-STAR-STABLE-W7-K3S-CONTEXT-FILTER-20261031.md:3, 9, 35-40`.)

However, the existing `deploy/k3s-local/build-and-deploy.sh` remains unchanged: it stages all of `crates/` and directly invokes the checked-in Dockerfile template without running the new generator. `deploy/k3s-local/README.md` still directs users to that old script. The W7 candidate therefore introduces a filtered alternative while leaving the prior documented entrypoint broad and incompatible with its marker template. No build was run. (`deploy/k3s-local/build-and-deploy.sh:21-30`; `deploy/k3s-local/README.md:44`; W7 candidate Dockerfile lines 9-13.)

The W7 report records an unresolved D2-owner worktree scope conflict: it says an uncommitted D2 README marks the API image/build/deploy path retired and unsupported. That status is not independently verified in this audit and the other worktree was left untouched. Treat `deploy/k3s-local` as outside the release scope; do not represent the W7 candidate as a supported product path until the D2 owner resolves that scope conflict. (`W7 c960b9193cc960ae3d85711472a0d4f31fedd0b3:docs/reports/PHASE-STAR-STABLE-W7-K3S-CONTEXT-FILTER-20261031.md:39, 77`.)

### P2 — Build configuration exists, but release artifact evidence is absent

The Canvas workflow requests `provenance: mode=max` and `sbom: true`, validates a returned digest, attests the image, and records the immutable image reference. This proves that the workflow is configured to produce evidence; this audit did not inspect an actual run, context archive, image digest, attached SBOM, or attestation. The context step logs file/package counts rather than retaining a per-file hash manifest. (`.github/workflows/publish-star-canvas-services.yml:244-250, 279-318`.)

The release deployment template also pins an external Envoy image by digest, but this audit did not retrieve its SBOM, license record, or provenance. (`deploy/star-canvas-services-k3s.yaml:344-347`.) W3 must remain open until the approved `v1.0.6` run provides the three exact image digests, retained context inventory with hashes, final image/rootfs review, SBOMs and attestations for each Star image, and evidence for the pinned Envoy runtime dependency.

## W7 candidate delta

Compared with `a272e9794e1a1975781720c9b3775c914f47f546`, the full W7 candidate diff is 30 files (1,358 insertions, 887 deletions). It includes changes to the Canvas and desktop workflows, the local Dockerfile, the new K3s context generator/tests, WBS changes, and deletions/replacements of other stable-release briefs, reports, and dispatch records. The broad WBS/report changes are outside this audit's owned scope and were not reviewed for acceptance; preserve the candidate worktree and select any future integration by reviewed, independently accepted changes only.

Within the container/build paths, candidate `c960b9193cc960ae3d85711472a0d4f31fedd0b3` changes:

- `.github/workflows/publish-star-canvas-services.yml`: removes the per-image concurrency configuration and explicit Buildx version pin. The three-image allowlist, minimal-context assembly, package closure checks, and SBOM/provenance settings are otherwise unchanged.
- `deploy/k3s-local/Dockerfile`: replaces broad `COPY crates` with a template expanded by the added context generator. The existing manual build script and README remain on the old direct-build path.

No Canvas release context filter was removed. The candidate's local context helper is not proof of a built image, and the license/version findings on the stable Canvas workflow remain. This audit makes no edits to the candidate branch or worktree.

## Required closure evidence

1. Align Cargo license metadata, `LICENSE`, OCI labels, and SBOM declarations with the approved MIT identity; demonstrate the stable license gate passes on the exact release tag.
2. Verify the exact `v1.0.6` tag resolves to the approved source commit, and show the runtime-reported version and OCI semver tag both equal the approved release version.
3. For each Canvas image, retain a sorted path-and-SHA-256 manifest of the exact context consumed by BuildKit. Confirm that the manifest contains only the generated workspace files, one allowlisted crate, the lockfile closure, Dockerfile, toolchain file, and LICENSE.
4. Build through the gated workflow and capture each resulting immutable image digest. Inspect the final OCI image filesystem/layers and verify that the attached SBOM and provenance refer to the same digest and source revision.
5. Collect a separate license/SBOM/provenance record for the digest-pinned Envoy runtime dependency.
6. Keep `deploy/k3s-local` and mock/test images outside the stable release deliverable. If W7 is continued, resolve the existing direct-build script/README and D2 scope conflict, then validate the new generator path before treating it as a supported local image workflow.
