# W1 Cargo overlap audit — 2026-11-01

**Status: W1 BLOCKED.** The user selected MIT as Star's product license. At the fixed target `dev@4ee0a7670dcba4e24b5df23eef18e90fa353e8e0`, the root workspace still declares `Apache-2.0` (`Cargo.toml:187`) while `LICENSE:1` says `MIT License`. The stable Canvas release workflow explicitly fails this mismatch (`.github/workflows/publish-star-canvas-services.yml:109-117`). This audit changes no source, manifest, lockfile, license, or source ref.

## Scope and evidence boundary

Compared the fixed `dev` tree with the committed refs below. All reads were read-only; no source branch, worktree, `main`, or remote ref was changed. The detached `wbs-w8-bi-native-adapter-20261024` worktree's uncommitted files were not opened. Its current contents and active MAGOS claims remain unverified and preserved. This is a bounded Cargo/ref audit, not a full branch or Docker image audit.

| Ref | Exact observed commit | Evidence / relation to target |
|---|---|---|
| Target | `4ee0a7670dcba4e24b5df23eef18e90fa353e8e0` | Fixed dev baseline and current lane HEAD before documentation changes. |
| W8 transport remediation | `0ae310440e446f530a210ec413efb526cb038d8f` | Commit `fix(w8): isolate durable source identities and restore restricted grant`; adds the native event adapter and owner-side source-origin changes. Its parent is `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10`. |
| W8 source-position claim | `bd90aecc1c39bdf7e647cd9768d36d7db1d55d0c` | Six commits after `0ae31044`; adds only a PostgreSQL arbitration test, migration, and phase report, with no additional Cargo manifest/lockfile changes. |
| Dependabot dependency group | `48f261011a51d14c44ca0182f99ebdade7a2bce5` | Commit `chore(deps): bump the rust-dependencies group across 1 directory with 15 updates`; parent is `5a051bfd8f76fbfbe8ff49a994579c60166d6ec6`. |

## Findings

### MIT metadata already applied to 13 package manifests

Commit `9083742a63306f310976b820faaa02a400a17ab8` (`build: align Star package license metadata to MIT`) changed these 13 package manifests to MIT: `bff/Cargo.toml`; `crates/domain-ai/Cargo.toml`; `crates/domain-agent-windows/Cargo.toml`; `crates/domain-form/Cargo.toml`; `crates/domain-cli/Cargo.toml`; `crates/domain-dashboard/Cargo.toml`; `crates/domain-report/Cargo.toml`; `crates/domain-kms/Cargo.toml`; `crates/domain-theme/Cargo.toml`; `crates/platform-messaging-proto/Cargo.toml`; `crates/star-credential/Cargo.toml`; `crates/star-treesitter/Cargo.toml`; and `crates/star-taskgraph/Cargo.toml`.

Those package changes do not settle the workspace identity: the root still says Apache-2.0. The three Canvas service manifests (`crates/canvas-engine/Cargo.toml`, `crates/domain-canvas/Cargo.toml`, `crates/canvas-realtime/Cargo.toml`) inherit `license.workspace = true`, so their package metadata currently resolves from the mismatched root. The stable release gate checks the root license and LICENSE heading and stops on this state. W1 cannot be called complete or a stable tag approved until the root metadata and all release/package metadata agree with MIT and the gate passes.

### W8 transport ref

At `0ae31044`, the root `Cargo.toml` adds only the workspace member `crates/bi-native-event-adapter`; `Cargo.lock` gains 12 lines. The new package manifest (`crates/bi-native-event-adapter/Cargo.toml`) depends on `bi-owner-runtime`, `platform-messaging-runtime` with `native-grpc-adapter`, `thiserror`, `tokio`, `tokio-util`, and `uuid`. The commit also changes the owner inbox/receiver/quarantine, adds PostgreSQL tests and source-origin/lease-order migrations, and adds W8 report/brief files. These paths align with W8's Rust-native durable receive and source-origin scope; they are not W1 license changes.

**Recommendation: adapt later, do not merge the branch wholesale.** A future W8 integration may consider the exact W8 paths and workspace-member addition after the MIT root metadata is reconciled and the W8 correctness gates pass. The committed W8 tree is based on an older snapshot whose root manifest contains `crates/canvas-game`; comparing the complete tree with target dev shows this legacy package and other historical differences. The present-target workspace member set does not contain it. A branch merge would therefore risk restoring content outside the reviewed W8 commit. Do not import historical tree state as part of W1.

The source-position branch at `bd90aecc` has no Cargo changes beyond the shared `0ae31044` base. Its six later commits add `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs`, `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql`, and `docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md`. That report states the PostgreSQL integration failed on the first Inbox decode because the SQLx query omitted `source_sequence`, and W8 remains OPEN. **Recommendation: hold those W8 commits pending correction and successful PostgreSQL/RLS/concurrency verification; do not represent them as validated.** The detached adapter worktree remains uninspected and untouched.

### Dependabot ref

Commit `48f26101` directly changes root dependency declarations `serial_test` 3→4, `nix` 0.29→0.31, and `wasm-bindgen-test` 0.2→0.3; it changes `crates/canvas-realtime/Cargo.toml` `tokio-tungstenite` 0.24→0.29 and rewrites its description, and updates `Cargo.lock` (910 lines in the commit diff, grouped as 15 dependency updates). The exact ref tree compared with target dev also contains the older `crates/canvas-game` workspace member; that is historical tree state, not a change made by the dependency commit itself.

**Recommendation: reject whole-ref integration; defer the dependency-only commit for a separate compatibility review.** These version updates do not resolve W1's license mismatch. No compile/test or API migration verification was run in this audit, so they are not approved here. The changed `canvas-realtime` description also carries stale legacy deployment references and should be reviewed independently if the dependency updates are later selected.

### Release-payload boundary

The Canvas release workflow is statically scoped: it creates a generated workspace with one allowlisted Canvas crate (`.github/workflows/publish-star-canvas-services.yml:120, 155-164, 207-216`), verifies package identity/build-script/version constraints (`:155-164`), and rejects local crate files outside that service and reports that the context contains no game/foreign workspace sources (`:326-328`). The three service manifests inherit `license.workspace = true`; the generated workspace propagates root package metadata. The workflow checks that root workspace license and LICENSE heading both equal MIT, while the crate checks cover name/build/version and dependency closure. This source inspection is not evidence that an actual image or SBOM was built and inspected.

The local k3s image uses a generated-context template (`deploy/k3s-local/Dockerfile:1-9`; `deploy/k3s-local/build-and-deploy.sh:25-26`), not a literal broad `COPY crates ./crates` in the template. The generated context and final image were not built or audited here. Keep that artifact path open in WBS; do not claim every Docker image/context is Star-only based solely on the Canvas workflow or `.dockerignore`.

## Itemized disposition

| Item | Disposition | Reason / next gate |
|---|---|---|
| Commit `9083742a` and its 13 MIT package manifests | Keep in current dev; no change in this lane | Verified on the fixed target; root workspace metadata remains Apache-2.0. |
| Root `Cargo.toml:187` / `LICENSE:1` mismatch | W1 blocker | Must align to the user's MIT decision; stable release workflow already fails closed. |
| W8 commit `0ae31044` | Adapt later | W8-relevant adapter/member changes; integrate only exact reviewed paths after license reconciliation and W8 verification. |
| W8 source-position commits `07a896e3` through `bd90aecc` | Hold | Report records first PostgreSQL Inbox decode failure; correction and live DB gates remain open. |
| Dependabot commit `48f26101` | Defer; reject full-ref merge | Dependency compatibility unverified and branch tree is stale; unrelated to W1 license resolution. |
| Detached W8 adapter worktree and active source claims | Preserve / unverified | Uncommitted content was not read; no mutation or cleanup authorized in this lane. |
| Canvas generated release context | Static guard verified; artifact unverified | Need actual generated-context, image, and SBOM inspection before release payload claim. |
| k3s-local generated context/image | Open follow-up | Template is generated; inspect generator output and final image separately. |
