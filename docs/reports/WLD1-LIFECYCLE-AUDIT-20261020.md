# WLD-1 Lifecycle Provider Source Audit

**Date:** 2026-10-20
**Audited base:** `dev` at `1136a24c709e944a538d28a2feff7e72b2b29f4c`
**Scope:** Source-only trace of `ProjectWorktreeLifecycleProvider` from API state through startup, persistence, and host Git adapter. No source, SQL, UI, or test changes were made. No target database or live provider was exercised.

## 0. Purpose

Determine whether the Project Worktree lifecycle routes have a trusted provider installed and identify the minimum safe implementation boundary. This audit treats the trait’s documented persistence and host-safety requirements as contracts, not as proof that an implementation exists.

## 1. Findings

| Area | Source evidence | Result |
|---|---|---|
| API contract | `crates/star-api-rest/src/group_api/worktree_lifecycle.rs:70-201` defines repository/candidate queries, create/import commands, receipt, states, errors, and `ProjectWorktreeLifecycleProvider`. | Port exists. The receipt states are `Provisioning` and `Ready`; no implementation or transition worker was found. |
| Router | `group_api.rs:180,721-740` declares and mounts `worktree_lifecycle::router()`. Routes cover repository listing, import-candidate listing, create, and import (`worktree_lifecycle.rs:228-245`). | Routes are present in the server. |
| State/composition | `group_api.rs:313-342` stores the provider as `Option<Arc<dyn ...>>` and initializes it to `None`; `with_worktree_lifecycle_provider` is the only installer (`387-392`). `main.rs:38,79-89` constructs `GroupApiState` with `PgGroupAppRegistryProvider` and optional BI port, then builds the router; it does not install a lifecycle provider. | Current startup composition leaves lifecycle unavailable. The handlers return `worktree_lifecycle_unavailable` when the provider is absent (`worktree_lifecycle.rs:266-267,351-352,419-420,465-466`). This is a source-path conclusion, not a live HTTP observation. |
| Concrete provider search | Repository-wide targeted Rust search for `impl ProjectWorktreeLifecycleProvider` / `ProjectWorktreeLifecycleProvider for` found only the trait, state field/setter, and handler references. | No concrete implementation or installer was found in this checkout. |
| Route authorization | `authorize_project_creator` validates actor, requires `worktree:create`, loads the active Project binding, and accepts only writer roles (`worktree_lifecycle.rs:319-337`). Repository listing separately requires `project:read` and an active Project binding (`248-279`). | API performs request-time Project authorization. It does not establish durable authorization immediately before a later host side effect. |
| Provider contract | Trait documentation at `worktree_lifecycle.rs:176-201` requires the provider to recheck current Project membership and Project-to-Repository binding before Git effects, resolve opaque import candidates against that binding, derive paths from server configuration, and atomically persist operation, Worktree/Project binding, Audit, and Outbox idempotently before returning. The permission snapshot is explicitly not a current grant. | These are requirements on a future provider; the routes only pass the snapshot reference and do not implement these guarantees. |
| Request idempotency | Create/import require an idempotency header and pass a request fingerprint and correlation ID in their commands (`worktree_lifecycle.rs:388-416,433-462,487-520`). | The API builds the idempotency inputs but does not persist, replay, or compare them. Durable enforcement belongs to the absent provider. |
| Receipt checks | `validate_receipt` checks non-nil operation/worktree IDs, Project/Repository/correlation equality, and branch validity (`worktree_lifecycle.rs:557-572`). | It does not prove the receipt was durably committed or that a Git Worktree exists. Handler responses are HTTP 202. |
| Project/Worktree persistence | Existing schema includes `worktree_canvas_worktree` (`db/migrations/2026-09-16-worktree-canvas-worktree.sql:12+`), added Project/runtime/version/retention projection columns in Phase 2B, and an authoritative SCD2 `multica.worktree_project_binding` (`2026-09-29-worktree-group-phase-2b.sql:46-104`). | These are useful existing identity/projection relations, but no lifecycle-operation record was found that joins create/import idempotency, provider state, Audit, and Outbox. |
| Existing owner/audit records | `worktree_owner_assignment`, `worktree_management_plan`, and append-only `worktree_management_audit` are defined in `2026-09-29-worktree-group-phase-2d.sql:9-65+`. The management flow checks the target owner's current Project membership and version before updating (`worktrees.rs:668-756`). | These records support existing owner/archive management; they do not make create/import atomic. The lifecycle create command carries `actor_id`, but no explicit owner assignment rule or owner-write is implemented. |
| Existing idempotency storage | `worktree_canvas_idempotency` is a legacy action table with a globally unique UUID key (`2026-09-16-worktree-canvas-idempotency.sql:12-52`). Targeted source search found no use from the lifecycle route/provider path. | Its existence is not evidence of lifecycle idempotency. Do not treat it as the required operation ledger without validating scope, fingerprint/replay semantics, and transactional integration. |
| Outbox/audit evidence | Archive Hook outcomes are written to `multica.hook_execution_event` and management outcomes to `multica.worktree_management_audit` (`worktrees.rs:1083-1138,795-836`). Other owners have their own outboxes, but no Worktree lifecycle create/import outbox was found. `worktree-service/src/pg_worktree_service.rs:16` describes event delivery as in-memory broadcast with a future Outbox follow-up. | No create/import Audit+Outbox transaction is present. Do not reuse a different owner’s outbox as a shortcut. |
| Host Git adapter | `crates/git-adapter/src/provider.rs:171-191` exposes `GitProvider::list_worktrees/create_worktree/remove_worktree`; `Libgit2Provider` and `CliGitProvider` implement these operations (`libgit2_provider.rs:41,57,125,192`; `cli_provider.rs:99,114,154,188`). | Physical Git operations exist, but these adapters take repository/path inputs and do not perform Project authorization, lifecycle persistence, audit/outbox, or idempotency. They are primitives for a trusted provider, not the provider itself. |
| Separate PG service | `PgWorktreeService::create` writes a Worktree row with `/worktrees/{branch}` and calls `upsert_by_id` (`worktree-service/src/pg_worktree_service.rs:191-221`; repository SQL at `pg_worktree_repository.rs:279-347`). | This path does not create a real Git checkout or implement the lifecycle trait; its synthetic path and in-memory event stream cannot prove host acceptance. |

## 2. Current request and intended provider transitions

Current composed path:

```text
HTTP request
  -> route validates input and current Project membership/role
  -> route supplies idempotency key, fingerprint, actor, scope snapshot, correlation ID
  -> GroupApiState has no lifecycle provider in main.rs
  -> handler returns feature_unavailable (worktree_lifecycle_unavailable)
```

Required path once implemented:

```text
authorized request
  -> provider rechecks current Project membership + Project/Repository binding
  -> idempotently persist operation + Worktree projection + Project binding
     + Audit + Outbox in the provider transaction
  -> durable receipt in Provisioning
  -> trusted host adapter acts using a server-resolved repository root
  -> reconciliation records verified Ready (or a durable failure/retry state)
```

The second path is a design target described by the port contract, not current behavior. There is no source-backed `Provisioning -> Ready` transition or crash-recovery mechanism in this checkout. The current receipt enum only has Provisioning and Ready; durable retry/failure states would need an explicit contract decision.

## 3. Archive readiness and Hook gate reuse

Archive is handled separately by `group_api/worktrees.rs`, not by the lifecycle trait. `confirm_management_plan` (`worktrees.rs:518-740`) applies the existing archive gate when the plan targets `archived = true`:

1. It locks and validates the pending plan/worktree version, releases DB locks while host/runtime observations run, then reacquires and reauthorizes before applying.
2. `prepare_archive_gate_facts` (`942-1054`) requires a runtime ID and configured readiness observer; it observes Git-lock state with a two-second timeout and requires `Unlocked`. Unknown lock state fails closed. It then requires a completed drain, a fresh readiness observation (at most five seconds old), and an admission fence with more than five seconds remaining.
3. `evaluate_archive_gate` (`1056-1081`) loads the verified effective Project/Worktree Hook snapshot and evaluates `BeforeWorktreeArchiveCleanup`. Any result other than Allow records the Hook event and management audit, cancels on Deny, and returns a conflict.
4. Before the update it reauthorizes, checks the expected version and observation freshness again, and records the Hook outcome. The successful path updates the logical `archived` flag, retention expiry, version, and append-only management audit (`worktrees.rs:722-836`).

The existing confirmation path **does not invoke Git removal**; it changes the database archive/retention projection. The lifecycle trait currently has no archive/delete method. Keep this management flow as the archive authorization gate. When physical cleanup is added, route it through this same gate (or extract the gate into a shared internal service used by both callers); do not duplicate Hook evaluation or perform removal before a current Allow decision and fresh drain/lock facts. A future cleanup dispatcher should consume a durably confirmed archive intent and remain fail-closed when runtime/lock/Hook state is unknown.

Current startup also leaves the optional Git-lock and archive-readiness observers unset (`GroupApiState::new`, `group_api.rs:322-342`; composition `main.rs:79-89`). As written, archive cleanup cannot pass its readiness gates without trusted observers being installed. Existing unit tests cover unknown lock, stale/future readiness, stale gate facts, and fence margin (`worktrees.rs:1500-1578`); they are not provider or live-runtime acceptance evidence.

## 4. Smallest safe next implementation slice

First establish the authoritative Project-to-Repository identity and server-side local repository-root registry. The current Worktree-to-Project binding does not identify a host path. `scm.cloud_branch` stores a Project/Repository relation supplied by a trusted SCM producer (`2026-10-01-engineering-run-directory.sql:72-84`), but that is remote branch registration; the product contract allows remote SCM association to be optional. No authoritative local root registry was identified in the scoped source/migration search.

Then implement and install a PostgreSQL-backed provider with a **read-only first slice**:

- Enable only `list_project_repositories` against the selected canonical Project-to-Repository registry, filtered by current tenant and Project membership.
- Keep `list_import_candidates`, `create`, and `import` returning Unavailable until candidate identity/root resolution and durable lifecycle acceptance are implemented. Do not seed repository choices or pass a caller-provided path/URL.
- Add the provider to `main.rs` through `with_worktree_lifecycle_provider`; keep missing/ambiguous registry state fail-closed.

For write-side implementation, add a lifecycle operation model and same-transaction persistence for key+fingerprint replay/conflict, operation state, Project binding, owner rule, append-only Audit, and transactional Outbox. Return only a durable `Provisioning` receipt; dispatch the Git operation through a trusted adapter after current authorization/binding checks; persist `Ready` only after independently verifying the expected path, branch, and HEAD. This split avoids equating a DB row or successful socket/process call with durable host completion.

## 5. Acceptance gates and unresolved state

| Gate | Required evidence before create/import is enabled |
|---|---|
| Identity and ownership | Authoritative tenant/Project/Repository and local-root mapping; explicit creator-versus-owner rule; active membership and Repository binding rechecked immediately before effects; revoke/race cases fail closed. |
| Idempotent durable acceptance | Same key+same fingerprint replays the same operation/receipt; same key+different fingerprint conflicts; concurrent duplicate requests yield one operation; rollback leaves no partial Worktree/binding/audit/outbox records. |
| Outbox and recovery | Operation, Project binding, Audit, and Outbox commit atomically; worker retries/restarts reconcile Provisioning to Ready or durable failure without duplicate Git worktrees. |
| Host isolation | Caller cannot supply filesystem paths or URLs; opaque import candidate is re-resolved against the current binding; root derives from trusted configuration; host adapter output/errors do not disclose paths or secrets. |
| Archive/cleanup | Only existing management confirmation plus a current Allow Hook result, complete drain, fresh readiness, unlocked Git state, valid fence, and rechecked Worktree version can authorize physical cleanup; any unknown/stale result blocks it. |
| Independent acceptance | Provider contract tests, PostgreSQL/RLS transaction tests, host-adapter integration and fault/recovery tests, plus a configured runtime observation. No such provider tests or live PostgreSQL/provider acceptance were performed in this audit. |

Unresolved from repository evidence: canonical local repository-root source; exact creator/owner assignment; lifecycle operation retention/state schema; host adapter configuration and sandbox boundary; outbox consumer/recovery path; target database migration/application state; live runtime readiness. Existing API validation tests do not resolve these gaps.

## 6. Verification and revision history

- Source was checked in the assigned checkout at the audited base using targeted Rust/SQL search and direct reads of the cited files. The coordinator supplied Tier 2 CodeGraph context, but its current generation and coverage check were unavailable; no coverage completeness claim is made.
- No tests, Cargo gates, database connections, or live host operations were run, per audit scope.
- `v0.1 — 2026-10-20 — Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核: Source-only WLD-1 lifecycle audit; provider absence, persistence/host boundaries, archive gate reuse, next slice, and acceptance gaps.`
