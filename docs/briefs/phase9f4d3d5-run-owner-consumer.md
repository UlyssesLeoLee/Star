# Brief: phase9f4d3d5-run-owner-consumer

**Agent**: worker
**Phase**: PHASE-9F4D-3D5
**Created**: 2026-10-07 19:28:50

---

Implement the Run/Execution-owner bounded JetStream capacity-result consumer for Phase 9F4D-3D5. Own only `crates/run-owner-runtime/**`, the root workspace membership/dependency lines in `Cargo.toml`, and `Cargo.lock`. Do not edit migrations, other crates, docs, or the automation runner. Other agents/root are editing those; do not revert or overwrite their work. Base is local dev merge `886e1168431dfb6d1efef7b5add4801a1f9b4962`; feature branch is `codex/phase9f4d3d5-run-owner-inbox`.

Repo has no `.codegraph/`; no `index_repository` tool is available, so use only brief-scoped direct source/API inspection and disclose the bounded scope. Current source evidence: Infra publisher is `crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs` and emits envelope_version=1, schema_version=2, tenant_id, event_id, event_type, idempotency_key, correlation_id, object payload, subject `star.infrastructure.v2.<type>`, Nats-Msg-Id=event_id. Allowed types: capacity_accepted/rejected/released/expired. Payload max 16 KiB, envelope max 128 KiB. Rust baseline/async-nats are already pinned to 1.99.0/0.50.0. Existing `PostgresInfrastructureOwnerInbox` is the opposite Run→Infrastructure direction and must not be reused for Run state writes.

Parent is adding migration `db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql` with append-only Run-owned tables:
- `multica.infrastructure_capacity_demand_outbox`: event_id, tenant_id, project_id, work_item_id, task_execution_run_id, occurrence_id, schema_version, idempotency_key, correlation_id, payload, occurred_at.
- `multica.infrastructure_capacity_result_inbox`: event_id, tenant_id, source_event_id, project_id, work_item_id, task_execution_run_id, occurrence_id, event_type, schema_version, idempotency_key, correlation_id, payload_digest, payload, received_at. Composite FK validates source demand event and exact Run scope.
The consumer must read/write only via Run-owner store; it must not write `infrastructure.*` or `automation.*`. Do not mutate task_execution_run state or declare Run ready. Capacity facts are recorded only; later projection/readiness is out of scope. The Run demand Outbox append API must accept a caller-owned SQLx transaction so future Run admission can atomically append it, but there is no admission caller today.

Implement strict envelope/subject/message-id/event type/version/payload validation. Compare result scope (source_event_id, occurrence, TaskExecutionRun, binding digest, and reservation identity when present) against the stored typed `HostInfrastructureDemandV2` in the demand Outbox before persistence. Existing Infra producer payload and `InfrastructureCapacityReservationAcceptanceV2` are not yet field-identical; do not deserialize the producer payload as that full readiness type. Do not claim accepted capacity means Runtime/executor ready. Persist result facts keyed by immutable Infra event_id; exact same ID+digest is idempotent, same ID with changed scope/content is a hard conflict.

Use async-nats 0.50.0 durable PullConsumer supplied by caller (no NATS credentials, stream creation, or production durable setup); one message per fetch, byte ceiling just above 128 KiB envelope+headers, bounded expiry, AckPolicy::Explicit expectation, and call `double_ack()` strictly after the Run DB transaction commits. On DB failure, do not ACK. Cancellation stops starting new fetches; do not call shared Client::drain(). No unbounded queues or parallel payload buffering. Unknown/malformed inputs must not mutate Run facts; do not TERM without durable quarantine. `capacity_released` has an allowlist entry but no producer in current source; keep that fact explicit and avoid inventing release ordering/projection semantics.

Add focused unit tests and an ignored disposable PG+NATS test named/documented so the parent can wire it into the runner. Test ack-after-commit, exact duplicate/replay, payload conflict, DB rollback/no ack, and strict bounds as feasible. Do not run live production services or Multipass; no VM use. Keep Schedule/TaskExecutionRun executor fail-closed. Include Cypher relationship headers on generated code and run Guardian for your owned changed files. No commits/merges. Report exact file list and actual gates run; do not claim unrun tests.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
