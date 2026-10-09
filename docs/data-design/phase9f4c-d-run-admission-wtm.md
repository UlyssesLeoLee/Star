# Phase 9F4C-D Run admission data classification

Date: 2026-10-10
Scope: D1 `2026-10-09-task-run-admission-attempt-lifecycle.sql`; D2 `2026-10-10-multica-run-infrastructure-binding-master.sql`.

## Transaction facts

| Existing table | Classification | D1 change | Retention / mutability |
|---|---|---|---|
| `multica.task_execution_run` | Transaction | Add `created_at` and `automation_admission_attempt`; keep each attempt as an immutable Run identity; actual start is separate from provisional admission. | Append/update rules remain those of the existing Run owner; no new retention or deletion policy is introduced. Existing rows are not backfilled. |
| `multica.task_execution_run_event` | Transaction | Allow `schedule_admission_confirmed` and `schedule_admission_compensated` in the existing append-only event stream; only `schedule_admission_confirmed` proves Run-owner confirmation. | Append-only; WORM/audit policy unchanged. |

## Work and Master coverage

D1 creates no Work table and no Master table. It does not add an idempotency cache, mutable retry row, or profile revision. Schedule rules remain existing Master data; broker delivery state and lease Work retention are unchanged. The attempt ordinal is part of immutable Run identity and is therefore classified with the Run Transaction, not as mutable Work state.

## Verification boundary

The schema runner applies the migration twice and exercises uniqueness, legacy ordinal, provisional timestamp, non-Schedule start, and event-vocabulary constraints on disposable PostgreSQL 18.6. No target DB grant changes, production deployment, NATS, Multipass VM, or inventory mutation is part of this slice.

## D2 Run Infrastructure Binding Master

| New table | Classification | Owner / purpose | Mutability and retention |
|---|---|---|---|
| `multica.run_infrastructure_binding_revision` | Master | Multica Run owner stores current and prior bounded `RunInfrastructureBindingSnapshotV2` revisions for one canonical tenant/Project/Branch/Engineering Run. | SCD2: prior rows may be closed once through `valid_to`; snapshot, scope, binding identity, revision, digest and actor are immutable. No physical deletion. |
| `multica.run_infrastructure_binding_event` | Transaction | Trigger-appended `created` / `superseded` facts for binding revisions. | Append-only/WORM; application INSERT, update, delete and truncate are denied; no retention expiry. |

D2 adds no Work table. The revision Master is tenant-scoped and FORCE RLS protected; the NOLOGIN Run-owner role can read/insert revisions and update only `valid_to`, and can read events without direct insert permission. A fixed-path SECURITY DEFINER trigger appends event facts only after guarded Master lifecycle mutations. A composite FK ties each record to the canonical Engineering Run directory. Snapshot JSON is bounded to 16 KiB and stored with explicit scope, stable binding ID, monotonic CAS revision, schema version and digest. The Rust adapter validates the V2 contract before writes and recomputes the digest after strict read/deserialization. Infrastructure Profile/Host/Cluster Master data stays owned by Infrastructure: this Run-owned snapshot is a versioned reference and does not establish that referenced resources are currently valid.

The PostgreSQL 18.6 disposable runner applied D2 twice and passed the binding lifecycle, least-privilege grants, FORCE RLS, tenant isolation, WORM and direct-event-insert denial assertions. The complete Run-owner schema runner passed 29/29 steps and removed its disposable cluster; one no-op legacy audit function was installed only as a test prerequisite for older Worktree DDL and does not validate production Audit behavior. Rust 1.99.0 focused unit tests passed 12/12; focused rustfmt and package all-target warnings-as-errors Clippy passed. Broader strict Clippy remains blocked by two existing `star-context` lints. No production database grant/deployment, NATS consumer, Multipass VM or inventory mutation is claimed. D2 creates no run-as authorization, owner-message consumer, capacity reservation, provider/readiness evidence, BI/Benchmark event or execution capability.
