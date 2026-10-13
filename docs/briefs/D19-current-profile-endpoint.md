# Brief: D19-current-profile-endpoint

**Agent**: worker
**Phase**: PHASE-9F4C-D19-ENDPOINT-WIRING
**Created**: 2026-10-14 04:25:01

---

## Task
Implement one D19 production-wiring slice that resolves a persisted, current Transport Profile through the existing PostgreSQL resolver and then maps its exact endpoint identity through the deployment-trusted endpoint registry.

## Base and evidence
- Base branch: `dev` at `717c1130e68480b150943c3efa90dbc53c3d52a3`.
- CodeGraph status for this checkout: index is up to date (1,837 files, 35,174 nodes, 104,154 edges). The available MCP graph tool returned unrelated symbols for targeted endpoint queries; `check_index_coverage` is not exposed in this session. Parent fell back to exact source review; negative/exhaustive claims are out of scope.
- Relevant current symbols/files: `PgTransportProfileResolver::resolve` in `crates/platform-messaging-runtime/src/transport_profile_pg.rs`; `TrustedEndpointResolver::resolve` and `ResolvedTransportEndpoint` in `crates/platform-messaging-runtime/src/transport_endpoint.rs`; module export in `src/lib.rs`.
- Current limitation: these two resolvers are not composed; `PgTransportProfileResolver` is uncached and returns an authorized active revision, while the endpoint resolver expects that current revision.
- Baseline: `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --all-targets --features native-grpc-adapter -j 4` passed with Rust 1.98.1. Rust 1.99.0 is not installed; report this as a formal-toolchain blocker.

## Owned files
- Own a new module such as `crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs` and its `src/lib.rs` export.
- Add focused test coverage in a new narrowly scoped test file if required; do not edit `native_grpc.rs`, endpoint resolver code, migrations, or shared design documents.
- Do not revert or overwrite any other lane's work.

## Required behavior
- Expose a typed composition that, on every resolve call, invokes `PgTransportProfileResolver::resolve(session, profile_id, kind)` and passes that newly read revision to `TrustedEndpointResolver::resolve`.
- The deployment-owned endpoint registry must be fixed at construction. Do not accept URI, TLS host, endpoint identity, or redirect destination from request data; do not cache profile revisions or secret material.
- Preserve current authenticated-session, tenant/deployment, profile lifecycle, endpoint identity, digest, and opaque-secret boundaries. Surface bounded errors without leaking endpoint URI or secret references.
- Keep native transport default-off and Schedule/Run execution disabled. Do not provision a database or Multipass instance.

## Validation and handoff
- Follow CypherGraph Guardian headers for every new/changed Rust file.
- Run focused format/check/tests using the installed toolchain if possible; because Rust 1.99.0 is absent, do not claim formal gates. Any DB-dependent test must be guarded by a unique disposable DB identity and remain unrun if that safe target is unavailable.
- Commit only owned files and cite `docs/briefs/D19-current-profile-endpoint.md` in the commit message.
- Do not rebase, merge, push, delete branches, or clean worktrees.
- Report exact changed files, commands/results, commit SHA, known gaps, and clean status.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
