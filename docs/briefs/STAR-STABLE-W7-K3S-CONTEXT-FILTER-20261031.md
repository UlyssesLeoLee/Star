# W7 — Filtered local K3s build context

## Goal

Prevent the local K3s API image from receiving the broad repository `crates/` tree. Build only the `star-api-rest` package and its resolved normal/build dependency closure, with every local package path explicitly listed in the generated Dockerfile.

## Scope

- Modify only `deploy/k3s-local/Dockerfile`, the context generator, focused tests, and this lane's brief/report/dispatch records.
- Keep `star-game`, game/legacy packages, unrelated workspace members, dev dependencies, and files outside the selected local crate closure out of the generated Cargo/Docker context.
- Resolve a Linux x86_64 Cargo graph with `--locked --offline`; fail closed on missing nodes, missing lock identities, path escapes, symlinks, forbidden package/path names, unsupported workspace settings, or changed generated contexts.
- The explicit guest-build option transfers only the generated context archive and runs the image build with `multipass exec` inside the named VM. Do not run a container build on the Windows host, publish or push an image, create a VM, or change Multipass inventory.
- Do not modify D2-owned maintenance scripts or deployment documentation.

## Implementation and acceptance

1. Resolve `star-api-rest` and traverse only normal/build dependency edges for `x86_64-unknown-linux-gnu`; omit dev edges.
2. Generate a minimal workspace membership list, filtered Cargo.lock, and package manifests without dev-dependency sections. Validate all selected local path dependencies remain inside the repository and the selected closure.
3. Expand Dockerfile markers into one explicit `COPY crates/<member>/ ...` directive per selected local crate. The checked-in Dockerfile is a template; direct broad-context builds are not supported.
4. Revalidate the generated context immediately before transfer: require expected root files, compare crate manifests against the workspace allowlist, reject game/legacy names and symlinks, and reject unexpanded markers or broad crate copies.
5. Unit tests cover selected closure, dev-only game exclusion, game dependency rejection, missing graph node/manifest, repository path escape, deterministic output, tampering rejection, and the `multipass exec` build command.

## Run path

After the host `cargo metadata` gate is functional, use:

```powershell
py -3 scripts/automation/star_k3s_context.py --repo <Star-checkout> --build-in-multipass <running-instance> --image-tag local-verify
```

The command does not create or purge Multipass instances. The build runs only inside the named guest and creates a local `star-api-rest:<tag>` image there.

## Evidence boundary

Fixture tests and a mocked Multipass command check do not prove the real workspace dependency graph, actual generated context, container build, or K3s rollout. W2 currently records that the installed Rust 1.99.0 Windows toolchain cannot start `rustc -vV`; Cargo metadata therefore could not be run against the real workspace in this lane. The Docker image and W7 remain OPEN until the real filtered context is generated and inspected and the image is built/verified inside the approved Multipass guest.

## CLI reference

- [Multipass transfer command](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/transfer.md)
- [Multipass exec command](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/exec.md)
