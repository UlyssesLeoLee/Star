# Phase 9F4C-D1 Run admission data classification

Date: 2026-10-09
Scope: `2026-10-09-task-run-admission-attempt-lifecycle.sql`

## Transaction facts

| Existing table | Classification | D1 change | Retention / mutability |
|---|---|---|---|
| `multica.task_execution_run` | Transaction | Add `created_at` and `automation_admission_attempt`; keep each attempt as an immutable Run identity; actual start is separate from provisional admission. | Append/update rules remain those of the existing Run owner; no new retention or deletion policy is introduced. Existing rows are not backfilled. |
| `multica.task_execution_run_event` | Transaction | Allow `schedule_admission_confirmed` and `schedule_admission_compensated` in the existing append-only event stream; only `schedule_admission_confirmed` proves Run-owner confirmation. | Append-only; WORM/audit policy unchanged. |

## Work and Master coverage

D1 creates no Work table and no Master table. It does not add an idempotency cache, mutable retry row, or profile revision. Schedule rules remain existing Master data; broker delivery state and lease Work retention are unchanged. The attempt ordinal is part of immutable Run identity and is therefore classified with the Run Transaction, not as mutable Work state.

## Verification boundary

The schema runner applies the migration twice and exercises uniqueness, legacy ordinal, provisional timestamp, non-Schedule start, and event-vocabulary constraints on disposable PostgreSQL 18.6. No target DB grant changes, production deployment, NATS, Multipass VM, or inventory mutation is part of this slice.
