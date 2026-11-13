# Phase 9F4D Run-owner W/T/M Classification

> Scope: Run/Execution owner capacity-demand and Infrastructure-result facts introduced by Phase 9F4D-3D5.
> Source migration: db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql
> Parent: docs/data-design.md §4.13.9 remains v1.8; this appendix's local design supplement is v1.10.

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

## W11-A design target — V3 hostless Run demand (not yet physical schema)

The preceding two-table classification describes the existing 9F4D-3D5 migration only. It remains two Transaction tables; the V3 target below is not implemented and does not change the physical count or any prior verification result.

| Proposed record | Target class | Owner and scope | Target fields and invariants |
|---|---|---|---|
| `multica.infrastructure_capacity_demand_outbox_v3` | Transaction (T) | Run/Execution owner; tenant + Project + Engineering Run + Work Item + trigger origin + TaskExecutionRun attempt | Append-only V3 demand with `schema_version=3`, tagged Schedule/Manual origin, owner-derived `execution_actor_id`, positive `attempt_number`, profile revision/digest, exact HostEnvironment ID+generation, K3s cluster ID+generation, namespace, runtime-profile digest, bounded budget, optional schedule attribution/remote provenance and canonical digest. No `branch_id` field. |
| `multica.infrastructure_capacity_result_inbox_v3` | Transaction (T) | Run/Execution owner; same exact source demand/attempt identity | Append-only result receipt and V3 event dedupe, FK/reference only to the V3 demand identity, current source revalidation, commit before transport ACK. |

No-placement Runs write no Infrastructure demand. A TaskExecutionRun requiring a placement remains non-executable until the Run owner revalidates current grants and binding and receives the Infrastructure owner's separate capacity/readiness result. The result Inbox is owner-local; Infrastructure must not write Run-owned tables.

V2 rows keep their required `branch_id`, `schema_version=2`, digest and source identity, and remain Schedule-only: the validator requires `run_origin=schedule` and checks the TaskExecutionRun's canonical rule/revision/occurrence facts (currently projected as `automation_rule_id`, `automation_rule_version`, `automation_occurrence_id`). Manual demand is rejected by V2. V3 supports both tagged origins: Schedule requires `(schedule_rule_id, schedule_rule_revision, occurrence_id)` and resolves its `execution_actor_id` from that immutable Rule revision's `run_as_actor_id`; Manual requires `manual_command_id` and stamps `execution_actor_id` from authenticated command context. Manual may carry optional historical `schedule_attribution` that is causal evidence only. The effective actor is owner-derived, included in the digest and reauthorized on every attempt; clients cannot set it. No nil Schedule UUID is used. Root origin keys are `(tenant, schedule_rule_id, schedule_rule_revision, occurrence_id)` for Schedule and `(tenant, manual_command_id)` for Manual. Attempt identity is unique by `(tenant, origin_key_digest, attempt_number)` and by `task_execution_run_id`; a positive attempt number is never reused within an origin. Exact redelivery of the same attempt and digest replays its receipt; a different digest conflicts; a legitimate retry uses a new TaskExecutionRun attempt identity and number while retaining the original trigger revision/provenance. The canonical digest includes the origin discriminant and required identity, effective actor, optional attribution presence/content, attempt number/identity, placement and profile generations. A future additive migration must add the V3 owner relations and audit/RLS/grants without changing or rewriting V2 facts; ambiguous, revoked or unverifiable rows are quarantined. This proposal is design-only: no DDL, migration, role grant, DB/RLS test, or runtime verification changed in W11-A.

## Revision history

| Version | Date | Revision |
|---|---|---|
| v1.10 | 2026-10-27 | Extend the design-only V3 contract to Schedule and Manual tagged origins, owner-derived execution actor, optional schedule attribution, unique execution attempts, origin/digest/replay semantics; keep V2 Schedule-only and preserve the two existing T classifications. |
| v1.9 | 2026-10-27 | Add design-only V3 hostless Run demand and result Inbox contracts; preserve the two existing T classifications and exact V2 replay boundary. |
