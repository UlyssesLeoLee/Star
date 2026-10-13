# Brief: D21-mtls-e2e

**Agent**: mtls-e2e worker
**Phase**: PHASE-9F4C-D21
**Created**: 2026-10-14 03:03:02

---

## Objective
Close one D19 transport acceptance gap by exercising the actual Tonic server/client TLS path over loopback and the real PostgreSQL-backed service, including peer-certificate revocation rejection.

## Graph and source evidence (parent, Verify/Tier 2)
- Project: Star dev checkout, C:/Users/leo19/orca/workspaces/Star/dev-converge-20261007, base dev@0c2c466a.
- Existing .codegraph database is present. codegraph_explore identified NativeGrpcTransportServer, its Append/Pull/Acknowledge/VerifyOrigin RPCs, serve_native_grpc, authenticated_peer, and server_mtls_config; native_grpc.rs owns client/server TLS config and peer fingerprinting.
- Direct source fallback verified server builder and PostgreSQL-backed service in native_grpc_server.rs, TLS helpers and existing native provider in native_grpc.rs, and the current SQLx origin verification fixture tests/native_origin_verification_pg.rs.
- The D19 report says the SQL contract checks revoked certificates, but no live gRPC TLS handshake/e2e was run.
- index_repository and check_index_coverage are unavailable; the graph tool did not return generation/status. No claim of full graph coverage.

## Owned files
- crates/platform-messaging-runtime/src/native_grpc_server.rs only if a minimal test seam is required.
- crates/platform-messaging-runtime/src/native_grpc.rs only if strictly required for handshake/shutdown support.
- crates/platform-messaging-runtime/Cargo.toml only for test-only dependencies, with license/source review.
- New test target crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs.
- Add a lane-specific phase report under docs/reports/PHASE-9F4C-D21-MTLS-E2E-REPORT.md.
- Do not edit transport_endpoint.rs, transport_profile.rs, transport_profile_pg.rs, lib.rs, migrations, or shared requirements/basic design files.

## Required behavior
- Use the real production Tonic TLS config and NativeGrpcTransportServer service against a disposable PostgreSQL database; do not substitute a fake business server or synthetic revocation decision.
- Prove a trusted client reaches a bounded RPC, a client without a certificate is rejected, wrong CA/server name is rejected, and revoking the presented peer certificate prevents a subsequent authenticated operation.
- Keep fixtures test-only and disposable; do not use production credentials/certs or alter Multipass inventory. Preserve default-off native transport and fail-closed Run execution.
- If a live DB/Windows runtime gate is unavailable, keep the DB-backed test opt-in/ignored with its exact environment requirement and report it as unexecuted. Do not label an ignored test as a pass.
- Follow CypherGraph Guardian headers for generated/edited Rust files and inspect existing comments around changed contracts.

## Verification and handoff
- Run changed-file rustfmt and targeted package check/tests with Rust 1.99.0 if available; execute the DB-backed test only against an isolated disposable instance. Record exact command/results and environmental failures.
- Commit the lane with a message citing this brief path. Preserve other agents' work; do not revert or edit outside ownership.
- Before reporting done, list files, commands/results, limitations (especially live DB/runtime proof), commit SHA, and clean status.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
