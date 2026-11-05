# Brief — Star Rust 1.99.1 baseline alignment

## Objective

Move the active Star build, validation, container, and release-toolchain defaults from Rust 1.99.0 to the current Rust 1.99.1 point release while preserving historical evidence and keeping all unrelated worktree data untouched.

## Scope

- `rust-toolchain.toml`, workspace/BFF/independent crate `rust-version` declarations.
- Supported CI workflows (using `dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8` with explicit `toolchain: 1.99.1`), release test fixtures, acceptance-runner defaults, and scaffolding templates.
- Star Canvas and k3s-local Rust builder images, pinned to verified Rust 1.99.1 image-index digests.
- Requirements, basic design, detailed design, AGENTS guidance, and stable-release WBS synchronization.

## Non-goals

- Rewriting historical phase reports or changing the meaning of evidence recorded against Rust 1.99.0.
- Closing workspace, target database, hosted CI, Vercel/Neon, Multipass/K3s, publication, or execution gates.
- Modifying `main`, remote branches, existing dirty worktrees, or unrelated W8/D2 contents.

## Acceptance

1. Active configuration resolves to Rust 1.99.1 without stale current defaults.
2. Docker builder references use verified immutable Rust 1.99.1 digests.
3. Focused metadata, formatting/static checks, and selected package checks run with `+1.99.1`; failures remain explicit.
4. Historical 1.99.0 strings remain only where they document historical evidence or a prior phase snapshot.
5. Changes are independently reviewed before fast-forward integration to local `dev`.

## Evidence source

Rust release: <https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/>.
