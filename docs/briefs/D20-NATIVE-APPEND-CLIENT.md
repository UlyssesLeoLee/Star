# Brief: D20-NATIVE-APPEND-CLIENT

**Agent**: worker
**Phase**: D20-NATIVE-RESULT-PUBLISHER
**Created**: 2026-10-14 08:58:35

---

Implement only the provider-neutral low-level exact append transport primitive needed by Infrastructure capacity-result publishing.

Ownership: crates/platform-messaging-runtime/src/native_grpc.rs, crates/platform-messaging-runtime/src/lib.rs, and a new crates/platform-messaging-runtime/src/native_append.rs if useful. Do not edit Infrastructure files, DB migrations, or docs.

Base and evidence: start from dev commit bc6dd8332d7812bbbd3c84e694495fc7c0055197. The current native gRPC Append RPC already accepts profile digest, recipient service id, message kind, message id, exact wire bytes and SHA-256, and returns log id, generation, sequence, wire SHA and exact_replay. Existing NativeGrpcAdmissionTransport::publish in native_grpc.rs (around line 901) implements the mechanics but is tied to AdmissionDeliveryOwner. Do not route Infrastructure through AdmissionDeliveryOwner::Run or change admission persistence semantics.

Required result: add a small explicit low-level client/interface for exact append using the already-authenticated/pinned endpoint and TLS configuration, returning a typed receipt that preserves exact_replay plus profile/endpoint and exact wire digest. Bound request bytes and concurrent RPCs; validate recipient/message id/kind, response sequence/log id/generation/SHA; keep errors redacted. Prefer reuse of existing connection and profile validation helpers, minimal code, and static dispatch. Do not create background workers, production startup wiring, a second RPC, NATS fallback, or dual-publish behavior.

Tests/gates: add focused unit coverage for malformed append request/receipt validation and byte digest/replay receipt where feasible; run focused checks if Rust 1.99 is available, otherwise report toolchain limitation honestly. Keep Cypher headers accurate and run the repository's Cypher graph guard if available. Commit is optional; return exact changed paths, contract/API, validation results, and any blockers.

Do not revert other agents' work. This agent owns only the files listed above.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
