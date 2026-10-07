# Phase 9F4D-3A — Infrastructure owner W/T/M classification

> Version: v0.4 · 2026-10-07
> Scope: additive schemas in `db/migrations/2026-10-07-infrastructure-capacity-owner-schema.sql` and `db/migrations/2026-10-09-infrastructure-provider-profile-master.sql`
> Boundary: preserve the original six-table 3A classification and add the two 3C3 profile tables; this extension does not rewrite the historical 100-table index or reclassify Run/Execution-owned data.

## §0 Purpose and ownership

Classify every physical table introduced by the 9F4D-3A migration. The Infrastructure owner receives versioned capacity demand from the Run/Execution owner, admits both Host and exact-cluster capacity in its own transaction, and publishes immutable owner facts. No cross-owner write or distributed database transaction is permitted. The schema is not a production writer, consumer, provider, or executor.

## §1 Complete table classification

| Physical table | Class | Row purpose | Retention / deletion | Tenant isolation |
|---|---|---|---|---|
| `infrastructure.capacity_demand_inbox` | Work (W) | Idempotent source-event/occurrence receipt, bounded retry state and processing lease | Starts `received`; may transition under the guard to leased/retry/rejected/accepted; terminal transition must be current and carries a 24–25 hour retention window; DELETE is denied before expiry | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |
| `infrastructure.capacity_reservation_bundle` | Work (W) | One atomic owner-local row pairing distinct Host and exact K3s cluster reservation IDs, generations, budget and lease facts | Starts `held`; only held→released/expired terminal transition; a current terminal timestamp and 24–25 hour retention window are required before DELETE | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |
| `infrastructure.capacity_outbox_delivery` | Work (W) | Independent pending/lease/retry/delivered/dead state for an immutable owner Outbox event | Starts `pending`; valid pending→leased→retry/delivered/dead flow; a current terminal timestamp and 24–25 hour retention window are required before DELETE | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |
| `infrastructure.capacity_demand_receipt` | Transaction (T) | Durable idempotency fact for the final accepted/rejected result of a source demand | Append-only; UPDATE, DELETE and TRUNCATE rejected by database triggers | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |
| `infrastructure.capacity_reservation_event` | Transaction (T) | Append-only held/released/expired fact binding both reservation IDs and owner scope | Append-only; UPDATE, DELETE and TRUNCATE rejected by database triggers | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |
| `infrastructure.capacity_owner_outbox` | Transaction (T) | Immutable versioned owner-to-owner acceptance/rejection/release/expiry event | Append-only; UPDATE, DELETE and TRUNCATE rejected by database triggers; delivery retry state is in the separate Work table above | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |

**3A coverage:** The original six-table migration has exactly one business class per table: 3 W, 3 T, 0 M. No table is mixed-class. This statement describes only the 3A six-table slice; the 3C3 Provider Profile Master and revision-event Transaction tables are classified in §1.1. HostEnvironment/provider/instance/cluster/node catalogs remain future Master work. Existing Project/Run reservations remain owned and classified by their existing migrations.

### §1.1 Phase 9F4D-3C3 profile Master and revision-event Transaction

| Physical table | Class | Row purpose | Retention / deletion | Tenant isolation |
|---|---|---|---|---|
| `infrastructure.provider_profile_revision` | Master (M) | SCD2 provider/profile revision with fixed provider, placement, Host/cluster capacity vectors, canonical digest, validity interval and actor | Physical DELETE is prohibited. Business fields are immutable; only the current row may be closed once by a tenant-scoped actor when a successor revision is prepared. | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |
| `infrastructure.provider_profile_revision_event` | Transaction (T) | Append-only `created` and `superseded` facts for each profile revision | UPDATE, DELETE and TRUNCATE rejected by database triggers; no TTL purge | `tenant_id`, ENABLE + FORCE RLS, `app.tenant_id` policy |

**Cumulative coverage:** 8/8 tables across the 3A and 3C3 migrations have one primary class: 3 W, 4 T, 1 M. No physical table is mixed-class. The 3C3 Infrastructure owner receives SELECT-only access to both profile tables; profile mutations belong to the profile-management boundary, which is not implemented by this phase.

## §2 Constraints and owner boundary

- The reservation bundle is one row to avoid a half-created Host/cluster pair; its two reservation IDs must be distinct and it pins both target generations.
- The Infrastructure owner validates demand idempotency and fresh dual-capacity admission in its own transaction. A schema row alone does not prove current ACL, Run binding, profile placement, capacity, provider readiness, or admission.
- Work rows can only be inserted in their initial state. Terminal timestamps cannot be backdated or materially future-dated. Retention must be between 24 and 25 hours from a current terminal transition, and cleanup is rejected until expiry.
- Transaction facts are immutable through UPDATE/DELETE row triggers and TRUNCATE statement triggers. No credential, source file, or arbitrary provider output belongs in these payloads.
- All eight tables across 3A and 3C3 use tenant FORCE RLS. FORCE does not substitute for a non-superuser, `NOBYPASSRLS` production role and explicit target-environment grants; neither production role provisioning nor target DB deployment is included here.

## §3 Verification evidence and open items

**Historical 3A/3B1 gate:** `scripts/automation/phase9f4d_infrastructure_owner_schema.py --pg-bin <PostgreSQL-18-bin>` was run with PostgreSQL 18.6. At that phase it applied the original schema and grants migrations twice to an isolated loopback-only cluster; checked the NOLOGIN/NOSUPERUSER/NOBYPASSRLS/NOINHERIT role attributes, schema USAGE, and exact W/T table privileges; verified positive/negative visibility across the six then-existing tables, cross-tenant write rejection, append-only UPDATE/DELETE/TRUNCATE rejection, Work lifecycle/retention guards, and 6/6 FORCE RLS. The grant contract gave the 3 Work tables SELECT/INSERT/UPDATE/DELETE and the 3 Transaction tables SELECT/INSERT only, with no schema CREATE or table TRUNCATE/REFERENCES/TRIGGER. A separate overprivileged disposable role was used only to prove WORM triggers reject forbidden mutations; it was not the production role. The temporary database stopped successfully and its unique temporary data directory was removed. The 3B1 audit recorded 14/14 steps passed. No Multipass instance was started; that gate consumed zero of the two registered-instance test cap.

The runner invokes `psql` directly. The 3C3 profile migration adds two tables and brings the verified Infrastructure FORCE RLS count to 8/8. The profile Master stores a deterministic digest, allows only one-time closure of a current revision, and records append-only creation/supersession events. The admission adapter verifies the persisted digest before deriving the resource request; the PostgreSQL SQLx case proves a digest mismatch produces a terminal receipt without a reservation. The grants migration has not been deployed to the target DB and has no current production migration-provisioning caller. The restricted NOLOGIN group is separate from the application login/secret; real membership and transaction-scoped `SET LOCAL ROLE` activation/reset remain unverified. Open items include target schema-owner/runtime provisioning, profile editor/current resolver, trusted Run binding/provider observation, cross-tenant allocator, actual Inbox consumer and owner Outbox delivery, release/reaper/reconciliation worker, Manager/provider/readiness producer, production BI/Benchmark linkage, sandbox/workspace bridge, and Schedule execution. Keep executor launch disabled until those gates pass.

## §4 Revision history

| Version | Date | Revision | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-07 | Mavis review | Classify the six 9F4D-3A Infrastructure-owner tables and record PostgreSQL 18.6 isolated evidence; retain production ownership, grants, provider and execution gaps. |
| v0.2 | 2026-10-07 | Mavis review | Add the 9F4D-3B1 least-privilege W/T grant matrix and separate NOLOGIN group/application identity boundary; record 14/14 isolated PostgreSQL evidence and keep target DB provisioning and runtime activation open. |
| v0.3 | 2026-10-07 | Mavis review | Add 9F4D-3C3 Provider Profile Master and append-only revision-event Transaction classification; record cumulative 8-table classification, 8/8 FORCE RLS and digest-mismatch rejection evidence; keep profile editing/current resolution and production owner/provider/execution gates open. |
| v0.4 | 2026-10-07 | Mavis review | Correct 3A-only six-table statements in §§1–3 to distinguish them from cumulative 3C3 schema; classify cumulative 8/8 table RLS and profile read-only owner boundary. Preserve original 3A/3B1 gate evidence as historical. |
