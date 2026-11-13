# Brief: 9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION-20261107

**Agent**: worker
**Phase**: WBS-9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION-20261107
**Created**: 2026-11-07 22:14:55

---

## Objective

Make native event receive authorization failures explicit so the service can stop in its existing fail-closed worker path instead of retrying a permanent authorization denial as transport noise.

## Owned source scope

Edit only crates/platform-messaging-runtime/src/native_grpc.rs. Preserve unrelated transport paths and existing provider/profile identity behavior.

Map tonic Unauthenticated and PermissionDenied statuses from native event receive RPCs to NativeGrpcEventReceiveError::Unauthorized. Preserve every other status as Transport so ambiguous/transient outcomes remain retryable. Apply consistently to event pull, origin verification, and ACK RPCs. Do not include status text, metadata, event payload, or credentials in errors.

Add focused unit coverage in this same Rust file for Unauthenticated and PermissionDenied becoming Unauthorized, and a transient status such as Unavailable remaining Transport. Update the Cypher header from actual code relationships and refresh its source hash with the project CypherGraph Guardian after editing.

## Coordinator-owned adapter extension

After the worker commit, source inspection found that the application adapter in `crates/star-api-rest/src/bi_task_event_ingestion.rs` discarded the native error class for pull and ACK (mapping every failure to `Transport`) and mapped every origin-verification failure to `Unauthorized`. That would still retry permanent authorization denials and would block the consumer on retryable verification transport failures. The coordinator amended the registered manifest to include this file and owns a shared native-error mapping plus focused tests; the worker's single-file scope and commit remain unchanged.

## Boundaries

The worker owns only `crates/platform-messaging-runtime/src/native_grpc.rs`; the coordinator owns the application adapter extension and lane report/dispatch. Do not modify any W8/Dependabot branch or worktree, D:\Star user files, main, or remotes. Do not change Cargo manifests or Cargo.lock. Keep resource handling bounded and preserve cancellation behavior.

## Validation and handoff

Use Rust 1.99.1. Run the focused crate test covering the mapping, then report the exact command and observed result truthfully. If a build or test cannot run, record it as NOT_RUN with the reason. Report changed symbols and Cypher Guardian result. Commit only this lane when the code and header agree, then hand back the commit SHA for independent review.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
