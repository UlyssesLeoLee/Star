# Phase 9F4D Run-owner W/T/M Classification

> Scope: Run/Execution owner capacity-demand and Infrastructure-result facts introduced by Phase 9F4D-3D5.
> Source migration: db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql
> Parent: docs/data-design.md §4.13.9; W/T/M version v1.8.

## Classification

| Table | Primary class | Why | Retention and mutation |
|---|---|---|---|
| multica.infrastructure_capacity_demand_outbox | Transaction (T) | Immutable domain/integration fact that Run owner requested infrastructure capacity for one execution attempt and occurrence. | Append-only; UPDATE/DELETE/TRUNCATE blocked; no automatic TTL or deletion. |
| multica.infrastructure_capacity_result_inbox | Transaction (T) | Immutable owner-received fact and durable global event_id dedupe record for the corresponding demand. | Append-only; UPDATE/DELETE/TRUNCATE blocked; no automatic TTL or deletion. |

Work (W): none. Master (M): none. The word Outbox does not make the demand a delivery Work row: Infrastructure owns its separate mutable delivery lease/state table. This migration has no short-lived job state.

## Owner and access matrix

Both tables enable and FORCE tenant RLS on tenant_id. The NOLOGIN, NOSUPERUSER, NOBYPASSRLS star_multica_run_owner role has schema USAGE and SELECT/INSERT on the two facts only; it has SELECT-only access to TaskExecutionRun origin, Schedule Rule ID/revision, occurrence, correlation and key scope columns so the owner can validate the typed demand inside the caller transaction. UPDATE, DELETE, TRUNCATE and access from other owner roles are revoked. The result's composite foreign key binds event identity and canonical scope to the demand fact; a global event_id unique constraint prevents cross-tenant reuse.

The Run-owner receive contract is provider-neutral: derive canonical Project, Branch, Engineering Run, Work Item, TaskExecutionRun and occurrence scope from stored demand, write one immutable result fact in the Run owner transaction, re-verify the exact source, then acknowledge using the selected transport adapter. JetStream double-ack is specific to its compatibility adapter; native receive remains opt-in and uses the same Inbox-commit-before-ACK boundary. Infrastructure append receipt is not this Run-owned Inbox fact. No TaskExecutionRun/Schedule update or BI event occurs. No physical cleanup is authorized until a separate audit/legal retention rule and immutable export design are reviewed.

## Evidence boundary

The existing schema/integration runner uses disposable PostgreSQL and loopback NATS, creates no Multipass VM and keeps the registered Multipass inventory cap at two, including stopped/deleted records. D20 also authors Infrastructure-owned route-pin and native-append-receipt Transaction tables in a separate migration; those facts do not change this Run-owner two-table W/T/M classification. Schema/RLS/idempotency evidence does not prove target database grants, production provider/Profile resolution, production native receive or publisher wiring, live TLS/revocation, durable replay/quarantine/recovery, BI/Benchmark, RSS/fairness, Schedule/Run execution, or NATS rollback. capacity_released is disabled on both JetStream and native until a typed release contract and consumer lifecycle semantics are implemented.
