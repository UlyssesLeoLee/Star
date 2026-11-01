# W7 — Filtered local K3s build context

## Goal

Prevent the local K3s API image from receiving the broad repository `crates/` tree. Build only the `star-api-rest` package and its resolved normal/build dependency closure, with every local package path explicitly listed in the generated Dockerfile.

## Scope

- Modify only `deploy/k3s-local/Dockerfile`, the context generator, focused tests, and this lane's brief/report/dispatch records.
- Keep `star-game`, game/legacy packages, unrelated workspace members, dev dependencies, and files outside the selected local crate closure out of the generated Cargo/Docker context.
- Resolve the Linux x86_64 normal/build dependency graph with `cargo metadata --locked --offline`; fail closed on missing nodes, path escapes, symlinks, forbidden package/path names, unsupported workspace settings, or changed generated contexts.
- In the isolated output only, let Cargo regenerate the lockfile offline from the source lock as its seed; verify every resulting registry identity/checksum against the source lock and every local package against the selected closure or manifest-required optional path dependencies.
- The explicit guest-build option transfers only the generated context archive and runs the image build with `multipass exec` inside the named VM. Do not run a container build on the Windows host, publish or push an image, create a VM, or change Multipass inventory.
- Do not modify D2-owned maintenance scripts or deployment documentation.

## Implementation and acceptance

1. Resolve `star-api-rest` and traverse only normal/build dependency edges for `x86_64-unknown-linux-gnu`; omit dev edges.
2. Generate a minimal workspace membership list and package manifests without dev-dependency sections. Copy manifest-required optional local path packages outside the active workspace member list. Have Cargo regenerate the output Cargo.lock offline, then verify its registry identities/checksums and local package identities against approved source-lock/context inputs.
3. Expand Dockerfile markers into one explicit `COPY crates/<member>/ ...` directive per selected local crate. The checked-in Dockerfile is a template; direct broad-context builds are not supported.
4. Revalidate the generated context immediately before transfer: require expected root files, compare crate manifests against the workspace allowlist, reject game/legacy names and symlinks, and reject unexpanded markers or broad crate copies.
5. Unit tests cover selected closure, dev-only game exclusion, game dependency rejection, missing graph node/manifest, repository path escape, deterministic output, source-lock identity/checksum drift rejection, tampering rejection, and the `multipass exec` build command.

## Run path

After the host `cargo metadata` gate is functional, use:

```powershell
py -3 scripts/automation/star_k3s_context.py --repo <Star-checkout> --build-in-multipass <running-instance> --image-tag local-verify
```

The command does not create or purge Multipass instances. The build runs only inside the named guest and creates a local `star-api-rest:<tag>` image there.

## Evidence boundary

Fixture tests and a mocked Multipass command check do not prove a container build or K3s rollout. A real source-workspace Cargo metadata pass and generated-context locked metadata pass were later completed with Rust 1.99.1 as a diagnostic because the installed Rust 1.99.0 host toolchain cannot start `rustc -vV`. These results do not satisfy the pinned 1.99.0 gate. The Docker image and W7 remain OPEN until the pinned toolchain gate is resolved and the image is built/verified inside the approved Multipass guest.

## CLI reference

- [Multipass transfer command](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/transfer.md)
- [Multipass exec command](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/exec.md)
