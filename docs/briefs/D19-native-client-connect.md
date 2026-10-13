# Brief: D19-native-client-connect

**Agent**: worker
**Phase**: PHASE-9F4C-D19-ENDPOINT-WIRING
**Created**: 2026-10-14 04:25:01

---

## Task
Close the D19 native-client wiring gap by requiring profile-bound `ResolvedTransportEndpoint` values for both native gRPC send and receive connection construction.

## Base and evidence
- Base branch: `dev` at `717c1130e68480b150943c3efa90dbc53c3d52a3`.
- CodeGraph status for this checkout: index is up to date (1,837 files, 35,174 nodes, 104,154 edges). The MCP graph result was noisy and `check_index_coverage` is not exposed; parent inspected the exact target source directly and does not rely on graph results for completeness.
- Relevant source: `NativeGrpcClientConfig`, `NativeGrpcReceiveClientConfig`, private `connect_native_client`, and both `connect` methods in `crates/platform-messaging-runtime/src/native_grpc.rs`; `ResolvedTransportEndpoint` getters in `src/transport_endpoint.rs`.
- Current limitation: both client configs accept caller-provided URI, TLS server name, and endpoint identity even though a trusted resolver now exists.
- Tonic dependency is 0.14.6. Context7 official Tonic docs confirmed the `Endpoint` + `ClientTlsConfig` construction surface; the fetched docs did not document HTTP redirect behavior, so do not make an unsupported claim about 3xx handling.
- Baseline: native gRPC all-targets check passed with Rust 1.98.1 plus `--ignore-rust-version`; Rust 1.99.0 is absent.

## Owned files
- Own only `crates/platform-messaging-runtime/src/native_grpc.rs` and tests within that file.
- Do not edit `transport_endpoint.rs`, `transport_profile_pg.rs`, `lib.rs`, Cargo manifests/lock, migrations, or shared design documents.
- Do not revert or overwrite any other lane's work.

## Required behavior
- Make every native gRPC connect path consume a `ResolvedTransportEndpoint`; remove raw caller-controlled URI/TLS-name/endpoint-identity fields from public connect configs.
- Validate the resolved endpoint's profile digest and endpoint identity against the immutable `AdmissionTransportProfile` before opening a channel. Build the Tonic endpoint only from the typed resolver getters and preserve mandatory TLS, bounded timeout, message size, concurrency, and fail-closed behavior.
- Keep credential loading outside this module's trust decision; do not read opaque secret-reference contents or log credential material. Do not add a redirect-following HTTP client or arbitrary endpoint override.
- Keep the feature opt-in/default-off; do not route production traffic or enable Schedule/Run execution.

## Validation and handoff
- Add focused tests for mismatched profile digest/endpoint identity and existing endpoint/TLS rejection behavior.
- Follow CypherGraph Guardian headers. Run focused formatting and package checks/tests using the installed toolchain; Rust 1.98.1 diagnostics do not satisfy the formal Rust 1.99.0 gate.
- Commit only owned files and cite `docs/briefs/D19-native-client-connect.md` in the commit message.
- Do not rebase, merge, push, delete branches, or clean worktrees.
- Report exact files, validation commands/results, commit SHA, known gaps, and clean status.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
