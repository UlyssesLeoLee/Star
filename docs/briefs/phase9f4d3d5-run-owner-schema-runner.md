# Brief: phase9f4d3d5-run-owner-schema-runner

**Agent**: worker
**Phase**: PHASE-9F4D-3D5
**Created**: 2026-10-07 19:28:50

---

Implement only the Phase 9F4D-3D5 PostgreSQL schema and disposable runner integration for a Run-owner infrastructure demand Outbox and capacity-result Inbox. Own exactly `db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql` and `scripts/automation/phase9f4d_run_owner_consumer.py` (you may instead extend the existing owner schema runner only if it avoids overlap with the new runner). Do not edit Rust crates, Cargo files, docs, or reports. Other agents/root are editing those; do not revert or overwrite their work. Base is local dev merge `886e1168431dfb6d1efef7b5add4801a1f9b4962`; feature branch is `codex/phase9f4d3d5-run-owner-inbox`.

Bounded source evidence: `multica.task_execution_run` and schedule tables are in existing migrations; Infrastructure-owned capacity Outbox/Work delivery is `db/migrations/2026-10-07-infrastructure-capacity-owner-schema.sql`. Infrastructure and Run are separate write owners. The current Infrastructure publisher emits event_id, event_type, schema v2 and payload; no Run consumer exists. `capacity_released` currently has no producer. There is no .codegraph index and no index_repository MCP tool; do not initialize one.

Schema design:
1. Add an append-only Transaction `multica.infrastructure_capacity_demand_outbox` with event_id PK; tenant/project/work_item/task_execution_run/occurrence scope; schema_version=2; idempotency key; correlation_id; bounded object JSON payload; occurred_at. Composite FK to exact `multica.task_execution_run` tuple; uniqueness for tenant+event_id and tenant+idempotency key. This is a persistence primitive only; no schedule admission caller is implemented in this phase.
2. Add an append-only Transaction `multica.infrastructure_capacity_result_inbox` keyed by (tenant_id,event_id), source_event_id, exact Run/occurrence scope, allowlisted event type (four capacity events), schema_version=2, correlation/idempotency, SHA-256 payload_digest, bounded object JSON payload, received_at. FK source_event_id and all Run scope columns to the demand Outbox. Exact event replay must be no-op by PK; conflicting-content detection is done by Rust store. No mutable projection/state or ACK delivery Work is added; JetStream durable ack + immutable event table is the dedupe authority.
3. FORCE RLS on both tables; tenant policies; append-only UPDATE/DELETE/TRUNCATE guards; create a NOLOGIN/NOSUPERUSER/NOBYPASSRLS/NOCREATEDB/NOCREATEROLE/NOREPLICATION/NOINHERIT role `star_multica_run_owner`; grant only schema usage + SELECT/INSERT on those two tables and minimal SELECT on `multica.task_execution_run`. Do not create app login credentials or grant writes to Infrastructure/Automation. Use comments and W/T/M classification. Do not broaden all existing multica table grants.
4. The disposable runner must apply the existing prerequisite migrations then this migration twice, verify FORCE RLS and role privilege matrix, and run the new Rust crate's ignored integration test against supplied PostgreSQL and NATS binaries on loopback with unique temporary data. It must stop only processes it starts, clean only its own temporary paths after confirmed stop, create zero Multipass VMs, never auto-purge, and keep total live test Multipass inventory capped at 2. Keep logs/audit within its own target directory. Do not print database URLs or secrets.
5. Preserve an honest boundary: app-login membership/target DB deployment, schedule admission writer, consumer projection, manager/provider/executor remain open/fail-closed. No commit/merge. Include Cypher header where the Python script is generated if this repo convention applies, and run relevant runner static/gates you can actually run. Report exact changed files and actual gates run. No fabricated history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
