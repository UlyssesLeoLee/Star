# BD-AUTOMATION-SCHEDULE-API-001 — Basic Design Addendum

> Status: 🟡 Verified design/implementation slice; production enablement remains open.
> Version: 0.7 | Date: 2026-10-04
> Parent: `docs/basic-design.md` §16 and `docs/data-design.md` §4.13.

## 1. Placement in the Worktree-first architecture

Schedule Rules are an Automation capability owned by an Engineering Run. The API is nested under Project → Engineering Run. A Schedule Rule references the Run's Branch, Repository, local Worktree, and Task Card; it does not create a separate workspace or bypass Run authorization. The UI may configure rules from the Run's Automation app. Selecting a Worktree remains the focus/CLI checkout decision and is not itself schedule authorization.

## 2. Components and transaction boundary

`star-api-rest::group_api::schedule_rules` is the authenticated REST boundary and is merged into `build_group_router`, which the REST binary uses. The workspace explicitly selects `jsonwebtoken`'s RustCrypto backend for deterministic RS256 signing/verification. Request-level tests run all four methods through the production router: missing credentials return 401, signed JWTs with insufficient scope return 403 before database access, and an oversized write body with a valid write scope returns 413; rejection responses carry private/no-store headers. These tests prove route wiring, token validation, scope rejection, and the body cap, but do not validate accepted Project/Run access, role grants, target ownership, or deployment. The boundary composes Project binding, Engineering Run authorization, Worktree/Task ownership checks, execution Profile resolution, and effective HookSet verification. `domain-automation::schedule` validates the pinned recurrence contract. PostgreSQL stores the immutable rule revision, append-only audit fact, append-only event Outbox record, and short-lived command replay record.

Create and revise each run in one PostgreSQL transaction. The transaction sets tenant/actor context, verifies current Project and Run grants, locks the idempotency key, resolves the target and policy snapshot, writes the Master revision and audit/Outbox facts, saves the replay response, then commits. Any error rolls the whole operation back. No handler launches a worker or creates a TaskExecutionRun.

Create derives immutable `run_as_actor_id` from the authenticated actor after Project/Run write authorization. The initial revision binds that identity to `changed_by`; each successor stores the same principal while `changed_by` separately records the editor. Historical rows backfill from the earliest Rule revision's `changed_by`; database guards reject both an initial mismatch and a changed successor principal. Every occurrence persists the identity in its own immutable column, and an insert guard compares it with the exact pinned Rule revision. The 9F4B runner passed on disposable PostgreSQL 18.6: all three migrations applied twice, seven Schedule tables retained FORCE RLS, and five adapter integration cases passed, including mismatched occurrence identity rejection. The future worker must recheck current Project membership, Run grant, and execution capability on every trigger, retry, and resume; revoked or unavailable ACL facts reject the occurrence without creating a Run.

## 3. W/T/M data classification

| Table | Class | Retention / mutation |
|---|---|---|
| `automation.schedule_rule_revision` | Master | Close-only SCD2 revisions with immutable `run_as_actor_id`; FORCE RLS; no delete/truncate. |
| `automation.schedule_rule_audit` | Transaction | Append-only actor facts; FORCE RLS. |
| `automation.schedule_rule_outbox` | Transaction | Append-only delivery facts written atomically with the revision; a composite foreign key binds its Run to the revision's Run; FORCE RLS. Consumer delivery state is a later bounded Work concern. |
| `automation.schedule_rule_command_idempotency` | Work | 24-hour replay TTL, exact expired-key deletion before reuse plus a 64-row stale cleanup budget, FORCE RLS. |

## 4. Fail-closed execution gate

The current capability is only Rule management. A Rule cannot execute until Phase 9F later stages connect recurrence materialization, fenced leasing, current ACL/target rechecks, capacity reservation, atomic TaskExecutionRun/RunEvent admission, and Outbox consumers. Existing disabled-rule checks remain in both domain materialization and persistence adapters. The implementation now filters future-dated Project bindings, Profile/Catalog entries, HookSets, and Rule revisions from current resolution. Production target database migration and service-role grants remain unverified.

## 5. Revision history

| Version | Date | Change |
|---|---|---|
| v0.1 | 2026-10-03 | Define the initial Run-scoped component and storage boundary. |
| v0.2 | 2026-10-04 | Record the reviewed persistence and cache-safety repair set. |
| v0.3 | 2026-10-04 | Align Rule-revision currentness with the current-fact contract and evidence. |
| v0.4 | 2026-10-04 | Record production REST-router wiring and unauthenticated route test evidence; keep authorization and target-environment gates open. |
| v0.5 | 2026-10-04 | Pin the JWT crypto backend and record signed-token scope rejection and authenticated body-cap request tests; keep accepted-access and database gates open. |
| v0.6 | 2026-10-04 | Add immutable run-as principal storage/backfill/guard and future per-trigger current-ACL recheck; keep worker, Launch Profile provider, target DB, and Run admission gates open. |
| v0.7 | 2026-10-04 | Record the first-revision creator equality and occurrence/pinned-rule identity guards, plus complete 9F4B PostgreSQL 18.6 validation while retaining worker/admission production gates. |
