# PHASE — Star Rust 1.99.1 baseline alignment

## §0 目的

将活动构建、CI、容器和验收入口统一到 Rust 1.99.1，并明确历史 1.99.0 证据不可回写，避免把旧版本验证误报为当前基线通过。

## §1 改动矩阵

| Area | Change | Status |
|---|---|---|
| Toolchain | `rust-toolchain.toml`, root/BFF/independent package MSRV → 1.99.1 | Implemented |
| CI/release | Supported workflows use `dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8` (master commit “Add 1.99.1 patch release”) with explicit `toolchain: 1.99.1`; stable desktop labels and test fixtures also align | Implemented |
| Containers | Canvas `bookworm` image index `sha256:cfcbed05398d0f6f81b7c3a24579475c567f97007ae7837ab421f9ca3e66333d`; k3s-local `slim-bookworm` image index `sha256:7d53bccd41fde2f509a276b37a8ef541f8e75fc3e1f85b059230386f512ed6c1` | Implemented; source digests verified with Docker registry inspection |
| Runners/templates | Active acceptance defaults and generated Cargo fixtures → 1.99.1 | Implemented |
| Docs | Requirements/basic/detailed design, AGENTS, stable WBS, Vercel SRS/ADR/BD/DD/TD, Infrastructure/Schedule/UI plans and current runner instructions synchronized | Implemented |
| Historical records | Earlier phase evidence retained as historical text | Intentional |

## §2 验证摘要

- Official release source reviewed: Rust 1.99.1 was published on 2026-10-15 and fixes the listed 1.99 regressions: <https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/>.
- `rustc +1.99.1 --version` → `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Docker registry inspection resolved the pinned multi-architecture image-index digests recorded above.
- GitHub API inspection resolved the action SHA `683b588c487a69d01f6c6ddf40f036ff682cc2c8` as the current `dtolnay/rust-toolchain` master commit for the 1.99.1 patch release; every supported workflow passes the toolchain explicitly rather than relying on a nonexistent `@1.99.1` action tag.
- `rtk cargo +1.99.1 metadata --no-deps --format-version 1 --locked` passed (exit 0).
- `rtk cargo +1.99.1 check -p star-api-rest --all-targets --locked -j 4` passed (exit 0, 2m10s); existing warnings remain in unrelated crates/tests.
- Python acceptance suites passed: `scripts.automation.test_w16a_device_revocation_http_pg_acceptance` plus `scripts.release.tests.test_star_canvas_context_policy` = 51/51; K3s context suite = 20 run, 1 platform-permission skip.
- `rtk python -m compileall` for the changed runner/template files and `rtk git diff --check` passed.
- PyYAML parsed all 14 workflow files; a structural assertion found 19 Rust action steps, all pinned to the audited SHA and explicit 1.99.1 input.
- Active Vercel design/test documents and current planning/runner instructions were re-audited; remaining 1.99.0 matches are retained only in dated historical evidence, fixtures, or revision history.
- `rustup component add rustfmt clippy --toolchain 1.99.1` completed; `rtk cargo +1.99.1 fmt --all -- --check` still reports pre-existing repository-wide formatting drift (no files were rewritten).
- No workspace-wide or production gate is inferred from this version bump.

## §3 已知缺口

- Full workspace all-targets/release/doc/bench gates, hosted CI, target PostgreSQL/RLS, Vercel/Neon, Multipass/K3s rollout, final image/SBOM/provenance and release approval remain open.
- Historical reports intentionally retain 1.99.0/1.98.x strings; those entries are not current baseline evidence.
- The published Docker digests establish immutable base-image identity, not a successful Star image build or runtime rollout.

## §4 子代理失败接手清单

No sub-agent implementation was used for this lane. The coordinator owns the isolated worktree, static review, validation and integration decision.

## §5 守门规则

1. Do not modify `main`, remote refs, or dirty/unowned worktrees.
2. Use only active configuration files for mechanical version changes; preserve historical evidence.
3. Keep the Rust baseline aligned across Cargo, rust-toolchain, CI, containers and runner defaults.
4. Run focused checks with `-j 4`; do not claim workspace success from a package-only result.
5. Keep execution and release gates fail closed until their independent evidence exists.
6. Do not use the version bump to bypass MAGOS overlap or merge ownership checks.

## §6 签字栏

| Role | Decision |
|---|---|
| Architecture | Mavis 接手审核 — current-baseline alignment is coherent |
| SRE Lead | Pending focused gate output |
| Platform | Pending container/CI review |
| Review主持 | Pending candidate integration review |
| PM | User-directed Rust 1.99.1 baseline |

## §7 修订历史

| Version | Date | 修订人 | Change |
|---|---|---|---|
| v0.1 | 2026-11-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Align active Star toolchain/configuration and document the Rust 1.99.1 baseline; retain historical evidence and open all unrelated release gates. |
| v0.2 | 2026-11-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Extend the current-baseline synchronization to the Vercel SRS/ADR/BD/DD/TD set, Infrastructure/Schedule/UI planning docs, and active runner instructions; preserve dated 1.99.0 evidence as historical. |
