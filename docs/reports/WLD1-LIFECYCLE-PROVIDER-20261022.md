# WLD-1 Run-scoped Worktree Lifecycle API Slice

**Date:** 2026-10-22
**Status:** Run-scoped API contract and fail-closed seam only; WLD-1 remains open.
**Author:** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 Purpose

Move the Worktree lifecycle API boundary under Engineering Run while preserving hostless Runs and optional remote Branch identity. This slice does not install a real local Worktree provider, establish a Run↔Worktree owner binding, or enable create/import/cleanup in production.

## §1 Change matrix

| File | Change | Boundary |
|---|---|---|
| `crates/star-api-rest/src/group_api/worktree_lifecycle.rs` | Add Run-scoped repository discovery, import-candidate, create, and import routes. Authorize Project and Run in the existing Run owner transaction; require the current Run repository binding before mutation; preserve explicit provider-unavailable failure; validate Run identity and binding state on receipts; derive a stable correlation ID from tenant, Run, actor, operation, and idempotency key when clients omit one. Add unit cases for hostless Runs without a Branch, missing repository binding, missing provider, stable correlation IDs, and invalid nil IDs. | No Git side effect exists without an injected trusted provider. Legacy Project endpoints remain registered and fail closed with the existing unavailable error for authenticated callers. |
| `crates/star-api-rest/src/group_api.rs` | Replace the Project-scoped provider slot with an optional Run-scoped provider slot; keep it `None` by default and expose explicit trusted-provider injection. | Startup still has no concrete provider. |
| `docs/briefs/WLD1-LIFECYCLE-PROVIDER-20261022.md` | Record the bounded implementation brief and lane constraints. | Dispatcher brief only; not provider acceptance evidence. |
| `docs/reports/WLD1-LIFECYCLE-PROVIDER-20261022.md` | Record this source-slice report and remaining gates. | No requirements/basic/detailed-design or WBS status is closed by this file. |

## §2 Validation summary

| Gate | Result |
|---|---|
| Cypher Graph Guardian | `cgg_guard.py --root . --changed --stamp` passed; 2 changed Rust source files checked. |
| Diff hygiene | `rtk proxy git diff --check` passed. Git reported only line-ending normalization warnings for the two Rust files. |
| Scope and client compatibility | Static search confirmed the existing frontend still calls Project-scoped paths. Compatibility paths remain present and return the existing unavailable response; Run paths are separate. |
| Rust formatting/build/tests | Not run: `cargo` and `rustc` are unavailable in this environment. Added unit cases are source-only evidence, not passing test evidence. |
| PostgreSQL/provider/runtime | Not run. No migration, target database, trusted repository-root provider, host dispatch, Runtime sandbox, or cleanup operation was exercised. |
| CodeRabbit | CLI invocation failed with `Permission denied`; the local source review continued and this is not recorded as a clean CodeRabbit result. |
| CodeGraph coverage | MCP returned `Transport closed`; the available CLI index describes the dev baseline, not this candidate worktree, and exposes no coverage check. Candidate changes were reviewed directly. |

## §3 Known gaps

1. `GroupApiState::new` installs no `RunWorktreeLifecycleProvider`; every lifecycle request remains unavailable until a trusted provider is implemented and deliberately installed.
2. No provider proves trusted repository-root resolution, current Project/Run grants immediately before host effects, current Repository binding, durable idempotency, owner-local audit/Outbox, recovery, or reconciliation.
3. `Ready` receipt validation requires a non-nil Run binding identity and positive revision. No code in this slice creates that binding; it remains owned by the existing Run↔Worktree binding owner.
4. The frontend still uses Project-scoped lifecycle controls. The compatibility routes return unavailable; moving the UI to the Run routes is a separate UI/API integration item.
5. There is no target-database migration or evidence for app-role grants, FORCE RLS behavior, replay, races, rollback, or authenticated route behavior.
6. Hook enforcement, current lease/claim/PTY/process drain, Runtime readiness, physical checkout cleanup and recovery remain unavailable. No checkout or VM was created or deleted.
7. Rust 1.99.0 formatting, compilation and tests remain unverified because the Rust toolchain is unavailable here.

## §4 Subagent failure handoff

- The WLD-1 worker produced a partial Run-scope API diff but did not return a completion report. The coordinator interrupted the worker to stop concurrent edits, reviewed the shared lane, and completed the compatibility, stable-correlation, report and Cypher-header work.
- Review identified and addressed two source-level gaps: legacy Project clients would otherwise receive 404 after route replacement, and server-generated random correlation IDs would make exact idempotency replay receipt validation unstable.
- The worker did not commit or rebase. The coordinator owns the remaining validation, commit and dev integration steps. No worker failure is presented as passing verification.

## §5 Guard rules

1. Run identity and current Project/Run authorization are resolved through the existing Run authority owner; a cloud Branch is not required.
2. Project or Run identity alone never authorizes a physical Worktree operation. Create/import require the Repository currently bound to the Run.
3. A client permission snapshot is context, not current authorization. A future provider must reauthorize immediately before every host side effect.
4. Only a trusted provider may resolve repository roots or perform Git operations. Request payloads contain no local path, URL, shell command or filesystem selector.
5. Missing provider, missing Run repository binding, ambiguous identity, stale authorization, or incomplete receipt fails closed.
6. `Ready` requires the binding owner’s active Run↔Worktree identity and positive revision; provisioning is not described as ready.
7. Exact idempotency replay must preserve the same correlation identity when the caller omits one. Reusing a key with changed request content remains the provider’s conflict decision.
8. Legacy Project routes remain disabled; they cannot create a Project-only Worktree that has no Run owner.
9. Do not treat API shape, a unit case, a 202 receipt, or a static migration as provider, database, host, Runtime, BI or product acceptance.
10. Physical cleanup remains blocked until current authorization, retention, lease/claim, process drain, audit, durable receipt and reconciliation are proven by their owners.

## §6 Sign-off

| Role | Result |
|---|---|
| Architecture | Mavis 接手审核：approve this bounded Run-scope API/fail-closed source slice only; WLD-1 remains open. |
| SRE Lead | Source boundary reviewed; no provider, drain, cleanup or recovery runtime evidence. |
| Platform | Cypher guard and diff hygiene passed; Rust 1.99.0 and target PostgreSQL/app-role gates remain open. |
| Review facilitator | Direct diff review completed; CodeRabbit unavailable due executable permission failure; no independent automated review pass is claimed. |
| PM | Not a user-ready lifecycle capability; Run lifecycle actions remain unavailable until the provider and owner-boundary gates close. |

## §7 Revision history

| Version | Date | Editor | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Added the Run-scoped lifecycle API seam, compatibility fail-closed paths, stable replay correlation, source review evidence and open gates. | WLD-1 provider implementation lane and coordinator review |
