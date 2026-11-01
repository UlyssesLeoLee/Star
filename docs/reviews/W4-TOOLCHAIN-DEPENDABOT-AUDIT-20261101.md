# W4 Toolchain Dependabot Candidate Review — 2026-11-01

**Disposition: HOLD — do not accept candidate `f6bede9301572ddd249ffd8c2baa39af4f106c0e`.** It requests Rust 1.120.0 across the workflow set, while Star's declared and tested minimum is 1.99.0. The official Rust release index currently reaches 1.99.1 (2026-10-15); it provides no evidence that 1.120.0 is an available Rust release. This review does not alter workflows or close W4.

## Scope and fixed evidence

- Baseline: `dev@4ee0a7670dcba4e24b5df23eef18e90fa353e8e0`.
- Candidate: local remote-tracking ref `origin/dependabot/github_actions/dtolnay/rust-toolchain-1.120.0`, commit `f6bede9301572ddd249ffd8c2baa39af4f106c0e` (2026-10-19), parent `8dcb65f7cc5602d4f7c6b681c368d5097bc12a22`.
- Review is limited to that commit's patch. The commit changes eight workflow files, with 19 toolchain-reference replacements. The broader baseline-to-candidate tree comparison includes unrelated branch divergence and is not treated as this commit's patch.
- Project contract at the baseline: `rust-toolchain.toml` selects `1.99.0`; root `Cargo.toml` declares `rust-version = "1.99.0"`.

## Changed workflow inventory

| Workflow | Candidate change | Review finding |
|---|---|---|
| `.github/workflows/ci.yml` | Six `@1.99.0` refs become `@1.120.0` | Default-selected toolchain no longer validates the declared minimum. |
| `.github/workflows/coverage.yml` | `@1.99.0` → `@1.120.0` | Same MSRV coverage regression. |
| `.github/workflows/license-gate.yml` | Three refs change to `@1.120.0` | Conflicts with the inline requirement that the action toolchain match `rust-toolchain.toml`; likely breaks the intended metadata/toolchain consistency. |
| `.github/workflows/p1e-integration.yml` | Two action refs change; explicit `with.toolchain: 1.99.0` remains | The override retains 1.99.0, but the action ref is now separately pinned to an unverified `1.120.0` ref. Upstream documentation says an explicit `toolchain` input should be paired with `@master` as the action revision. |
| `.github/workflows/publish-canvas-game.yml` | `@1.99.0` → `@1.120.0` | Same MSRV coverage regression. |
| `.github/workflows/spawn-e2e.yml` | Two refs change to `@1.120.0` | Same MSRV coverage regression. |
| `.github/workflows/star-desktop-build.yml` | Three refs change to `@1.120.0` | Same MSRV coverage regression across supported desktop targets. |
| `.github/workflows/wasm-build.yml` | `@1.99.0` → `@1.120.0` | Same MSRV coverage regression. |

## Upstream and release semantics

The [`dtolnay/rust-toolchain` README](https://github.com/dtolnay/rust-toolchain/blob/master/README.md) describes this as an action that installs a Rust toolchain using rustup. It explicitly says the requested `@rev` selects the Rust toolchain (for example, `@1.89.0` selects Rust 1.89.0). Its optional `toolchain` input defaults to matching `@rev`; when an explicit `toolchain` input is provided, the README says to use `@master` as the action revision. It also documents full-length commit SHA references and requires the chosen SHA to remain in `master` history.

The [official Rust release announcements index](https://blog.rust-lang.org/releases/) lists Rust 1.99.1 on 2026-10-15 and 1.99.0 on 2026-10-01 as the latest releases at the review date. The individual official announcements are [Rust 1.99.1](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/) and [Rust 1.99.0](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/). There is no official evidence in that release index for Rust 1.120.0. Because the action revision itself selects a toolchain by default, this candidate cannot be treated as an ordinary action implementation update.

## Recommendation

Do not merge this candidate. Keep CI aligned with the declared Rust 1.99.0 baseline until a deliberate, separately reviewed MSRV change updates `rust-toolchain.toml`, Cargo `rust-version`, CI pins, and the required focused gates together. If W4 also requires supply-chain hardening, handle that as a separate change: select a verified `dtolnay/rust-toolchain` full commit SHA in `master` history and explicitly set the intended 1.99.0 toolchain. The current dev already demonstrates a SHA-pinned pattern in `.github/workflows/publish-star-canvas-services.yml` (`ff7f6a3374ce205788da40198ec918bd5d67aea7`, annotated `1.99.0`); this is local precedent, not proof that the candidate is acceptable.

## Limits

This review used the local baseline and local remote-tracking candidate only; it did not fetch or verify the live upstream branch, check the candidate action ref's remote existence, run GitHub Actions, or exercise Cargo. The exact remote PR status is unknown. No claim is made that the candidate was merged or that W4 is closed.
