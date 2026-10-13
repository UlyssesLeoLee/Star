# Brief: D21-endpoint-resolver

**Agent**: endpoint-resolver worker
**Phase**: PHASE-9F4C-D21
**Created**: 2026-10-14 03:03:02

---

## Objective
Close one D19 production-wiring gap by adding a typed, trusted resolver from a pinned Transport Profile endpoint_identity to an HTTPS URI and TLS server-name pair.

## Graph and source evidence (parent, Verify/Tier 2)
- Project: Star dev checkout, C:/Users/leo19/orca/workspaces/Star/dev-converge-20261007, base dev@0c2c466a.
- Existing .codegraph database is present in this checkout. The available codegraph_explore calls found TransportProfileDefinition, TransportProfileRevision, PgTransportProfileResolver, current profile revocation integration coverage, and native_grpc.rs functions client_tls_config, valid_https_endpoint, NativeGrpcAdmissionTransport, and NativeGrpcAdmissionReceiveTransport.
- Graph queries were noisy; targeted disk review confirmed transport_profile.rs stores an opaque endpoint_identity in immutable Profile revisions, while native_grpc.rs takes caller-supplied endpoint URI/TLS name. transport_profile_pg.rs resolves active/revoked Profile revisions but does not map endpoint identity to a trusted URI.
- MCP tools index_repository and check_index_coverage are unavailable in this session; no graph generation/status was returned. Direct file review is the fallback and exhaustive claims are out of scope.

## Owned files
- Primary: crates/platform-messaging-runtime/src/transport_endpoint.rs (new).
- crates/platform-messaging-runtime/src/lib.rs only for module/export wiring.
- Add a lane-specific phase report under docs/reports/PHASE-9F4C-D21-ENDPOINT-REPORT.md.
- Do not edit native_grpc.rs, native_grpc_server.rs, transport_profile.rs, migrations, or shared requirements/basic design files.

## Required behavior
- Resolve only exact deployment-trusted endpoint identities into a validated HTTPS URI and TLS server-name pair; reject unknown, duplicate, disabled/revoked, malformed, or mismatched values.
- Prevent request-provided arbitrary URI/host, redirects, credentials, fragment/query, wildcard/invalid TLS names, and unbounded resolver entries.
- Keep secret material out of the resolver; only opaque secret references may be carried. Do not activate native traffic, change default features, or enable Schedule/Run execution.
- Add focused unit tests for valid resolution and fail-closed cases. Update only the detailed design/report for the implementation; root will reconcile the three top-level design layers after integration.
- Follow CypherGraph Guardian headers for generated/edited code and inspect comments around affected contracts.

## Verification and handoff
- Run formatting/check/tests for the changed crate using Rust 1.99.0 if available, otherwise report the exact toolchain blocker. Do not claim DB behavior from unit tests.
- Commit the lane with a message citing this brief path. Preserve other agents' work; do not revert or edit outside ownership.
- Before reporting done, state changed files, commands/results, remaining production identity/certificate/DB gaps, commit SHA, and worktree cleanliness.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
