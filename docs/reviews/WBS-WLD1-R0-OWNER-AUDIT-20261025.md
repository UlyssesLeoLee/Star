# WBS-WLD1-R0 ProjectRepository Owner and Source Audit

> **Status:** Static, bounded source audit complete. WLD1-R0 is not ready for a persistence/API implementation until the Project identity and authorization writer are established. No code or database changes were made; no tests were run.
> **Baseline:** dev `bf374449cdc848c3c6aac7ad251da38affa4dedb` (2026-10-25).
> **Scope:** Project, SCM Repository identity, Project permission, Run Directory, and Worktree lifecycle ownership relevant to ProjectRepository links and migration-before-unlink.

## 0. Purpose

Determine whether the current source identifies a canonical writer for Project-to-Repository 0..N links, distinguish that relation from existing SCM, permission, Run, and Worktree relations, and state the next safe implementation gate.

## 1. Finding and readiness

**No current durable ProjectRepository association writer was found in the bounded source examined.** The Project domain exposes an abstract repository port, but the concrete `ProjectRepository` implementation and `InMemoryProjectService` are in-memory. The current migrations provide a versioned SCM Repository master, project membership facts, Run/Branch bindings, and Project/Worktree bindings; none is a Project-to-Repository association owned by a durable Project service.

The recommended future canonical owner is the **Project bounded context**, because the link answers which repositories belong to a Project and its optional default-repository UI preference. This is an architectural recommendation, not an existing implementation fact. Before coding the link table/API, WLD1-R0 must identify or establish the durable Project identity authority and the authoritative Project authorization writer. The inspected code does not establish either one. Do not place Project membership in the SCM Repository master, Run Directory, or Worktree binding table to work around that missing authority.

| Concern | Current evidence | Ownership conclusion |
|---|---|---|
| Project identity and persistence | `Project` has tenant/workspace identity fields; `ProjectRepository` exposes project CRUD and policy/template operations, while `InMemoryProjectRepository` is the concrete implementation. `InMemoryProjectService::new` constructs it. [`crates/domain-project/src/lib.rs:95-130`](../../crates/domain-project/src/lib.rs), [`crates/domain-project/src/lib.rs:408-432`](../../crates/domain-project/src/lib.rs), [`crates/domain-project/src/lib.rs:445-453`](../../crates/domain-project/src/lib.rs), [`crates/domain-project/src/lib.rs:679-704`](../../crates/domain-project/src/lib.rs) | Project identity's durable owner is not established by this code. The in-memory port is not a production source of truth. |
| Project authorization | `permission.project_role_binding` stores tenant/project/user role and SCD2 validity. Its tenant RLS policies scope access. The Run Directory owner receives read/lock access to this authorization fact, not authority to mutate it. [`db/migrations/2026-09-29-worktree-group-phase-2b.sql:10-45`](../../db/migrations/2026-09-29-worktree-group-phase-2b.sql), [`db/migrations/2026-10-20-engineering-directory-owner-role.sql:86-111`](../../db/migrations/2026-10-20-engineering-directory-owner-role.sql) | Authorization is a separate permission fact. The migration set does not identify the canonical writer that grants/revokes Project roles or prove that a Project exists. |
| Repository identity | `scm.repository_master` holds tenant-scoped stable RepositoryId metadata with versioned history and audit. Its own comment explicitly says it has no Project/Branch FK and does not provide Project/Run authorization. [`db/migrations/2026-10-23-scm-repository-master.sql:31-39`](../../db/migrations/2026-10-23-scm-repository-master.sql), [`db/migrations/2026-10-23-scm-repository-master.sql:70-75`](../../db/migrations/2026-10-23-scm-repository-master.sql) | SCM owns Repository identity and metadata, not which Projects associate to that identity. |
| Remote Branch | `scm.cloud_branch` contains `project_id`, `repository_id`, and a required remote identity. `multica.engineering_run` initially references that Branch tuple. [`db/migrations/2026-10-01-engineering-run-directory.sql:70-82`](../../db/migrations/2026-10-01-engineering-run-directory.sql), [`db/migrations/2026-10-01-engineering-run-directory.sql:126-141`](../../db/migrations/2026-10-01-engineering-run-directory.sql) | This is remote Branch registration and cannot represent the Project's repository collection or hostless Runs. |
| Engineering Run and Run↔Worktree | `engineering_run` and `engineering_run_worktree_binding` are Run Directory facts. The host-independent migration makes Run repository/branch optional; the Directory owner has its own restricted role, idempotency ledger, Outbox, and scoped grants. [`db/migrations/2026-10-15-run-host-independent-directory.sql:23-34`](../../db/migrations/2026-10-15-run-host-independent-directory.sql), [`db/migrations/2026-10-15-run-host-independent-directory.sql:212-245`](../../db/migrations/2026-10-15-run-host-independent-directory.sql), [`db/migrations/2026-10-20-engineering-directory-owner-role.sql:21-27`](../../db/migrations/2026-10-20-engineering-directory-owner-role.sql), [`db/migrations/2026-10-20-engineering-directory-owner-role.sql:169-193`](../../db/migrations/2026-10-20-engineering-directory-owner-role.sql) | Run Directory owns Run-local bindings and must receive migration commands. It must not become the canonical ProjectRepository owner. |
| Project↔Worktree | `multica.worktree_project_binding` is explicitly documented as the authoritative Project-to-Worktree SCD2 relation; `worktree_canvas_worktree.project_id` is a read projection. [`db/migrations/2026-09-29-worktree-group-phase-2b.sql:67-85`](../../db/migrations/2026-09-29-worktree-group-phase-2b.sql), [`db/migrations/2026-09-29-worktree-group-phase-2b.sql:102-109`](../../db/migrations/2026-09-29-worktree-group-phase-2b.sql) | This relation identifies Project-owned Worktrees, not ProjectRepository membership. Do not overload it. |
| Worktree lifecycle provider | The trusted provider must recheck Project/Run grants and Run repository binding immediately before Git effects, persist its own idempotent operation/audit/Outbox, and must not write the Run↔Worktree owner's tables directly. [`crates/star-api-rest/src/group_api/worktree_lifecycle.rs:327-342`](../../crates/star-api-rest/src/group_api/worktree_lifecycle.rs) | Host/Worktree checkout identity and local lifecycle belong to that provider/owner boundary; it is not the Project link writer. A durable Host-root authority is still an open WBS dependency. |

The bounded migration search for canonical Project identity looked for `CREATE SCHEMA ... project`, `CREATE TABLE ... project`, `project_master`, and `project_id` declarations under `db/migrations`; it found Project role/binding and domain tables, but no canonical Project identity table. The repository search for `ProjectRepository`, `PgProject`, and Project association terms across `crates`, `db/migrations`, and `frontend/src` found the in-memory domain port, the Worktree lifecycle projection, SCM Repository identity, and Run/Worktree bindings, but no dedicated durable ProjectRepository writer. This is bounded evidence, not proof that no external service or unsearched integration exists.

## 2. Recommended owner boundaries

1. **Project owner (future canonical writer):** owns ProjectRepository membership and its active/retiring/closed state, current link revision, optional default-repository preference, and the Project-side retirement operation. It must first be backed by an authoritative durable Project identity and current authorization boundary.
2. **Permission owner:** owns membership/grant changes. Project/Run mutation APIs must use current authorization facts, not UI role snapshots. The exact writer for `permission.project_role_binding` remains unverified in this repository.
3. **SCM owner:** owns stable `RepositoryId`, Repository master revisions, and provider metadata. Project membership references the stable RepositoryId without modifying SCM facts. Repository-master revisions and Project links are distinct histories.
4. **Run Directory owner:** owns Engineering Run repository bindings and Run↔Worktree references. It receives idempotent migration/retirement commands and commits changes only in its own transaction and Outbox.
5. **Worktree/Host owner:** owns local checkout identity, trusted root/path resolution, Worktree↔Project binding, and actual checkout migration or retirement. A replacement link is acceptable only after the Host owner verifies the checkout's repository identity. It receives its own command and emits its own durable receipt.

Do not use one shared database transaction or direct cross-owner table writes. The current `RunWorktreeLifecycleProvider` contract already requires the provider to use the binding owner and forbids direct writes to its tables; the same owner boundary must apply to ProjectRepository retirement.

## 3. Migration-before-unlink contract

The user-approved policy is **0..N repositories per Project**, an optional default that is only a UI preference, and migration-before-unlink. The implementation contract should preserve that decision:

1. The Project owner CAS-transitions the selected active link to `retiring` and advances a link revision. New Run bindings, Worktree create/import, and other owner references to that link must be fenced before migration begins. No default selection may silently create a Run or Worktree binding.
2. The Project owner issues a stable `retirement_operation_id` to each owner that can hold an active reference. The Run Directory and Worktree/Host owner each migrate references to a different active Repository link in the same Project, or explicitly retire the reference. Each owner validates its own authorization and binding/checkout facts, then commits its change, audit and Outbox locally.
3. Each owner returns an immutable, replayable receipt bound to the retirement operation, Project/Repository link revision, owner identity and owner-local committed revision. Duplicate delivery returns the original receipt; mismatched operation/payload or stale owner revision fails closed.
4. The Project owner completes unlink only after all expected owners acknowledge their fence, every migration/retirement receipt is successful, and a fresh owner-revision-aware reconciliation finds zero active references. If any owner is unavailable, stale, unauthorized, or has an unknown reference count, keep the link `retiring` and retry/reconcile. Historical Run/task/audit facts remain immutable; only active references move or retire.

The final zero-reference check must be serialized against the same owner fences that reject new bindings. A one-time count before owners have fenced admission is not proof: a late Run/Worktree bind could appear after the count. If the Project identity/permission owner cannot provide an authoritative fence and authorization check, unlink remains disabled.

## 4. Default preference and data boundaries

- Store an optional default only within the future Project owner's authority. A non-null value must point to an active ProjectRepository link in the same tenant and Project; enforce this with owner-side CAS/constraints and current authorization.
- Clearing or changing the default is an explicit Project preference update. Retiring a default does not auto-select another repository, and a default never acts as an implicit Run binding.
- Candidate relation data belongs to the Project owner as versioned Master/SCD2 history; owner commands/idempotency leases are bounded Work data; append-only audit and Outbox events are Transaction facts. Do not put payloads or short-lived retry state into the SCM Repository master.
- A cross-tenant link, unknown Project, unknown Repository, archived/retiring target, or incomplete authorization must fail closed. Do not invent a FK to a Project table that does not exist in the examined migrations. Before relying on relational FK enforcement, establish the authoritative Project identity store and a tenant-scoped reference to stable RepositoryId; otherwise use a versioned owner command/API with durable validation and receipt.

## 5. Next WBS gate and acceptance evidence

WBS v0.45 §13 already marks WLD1-R0..R2 open and says source audit precedes owner/schema freeze; this audit confirms that ordering. [`docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:361-385`](../implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md)

Recommended split before WLD1-R1:

- **WLD1-R0a — owner/source decision:** name the durable Project identity writer, ProjectRepository authorization writer, and read/write APIs. If another service outside this repository is the authority, add its versioned interface and current authorization contract. Record which role owns each migration and receipt.
- **WLD1-R0b — relation foundation:** only after R0a, add the Project-owned 0..N link schema/API with CAS, optional preference, audit and same-owner Outbox/idempotency. Preserve tenant isolation and do not expose a write path until grants and database behavior are verified.
- **WLD1-R1 — owner migrations:** add Run Directory and Worktree/Host owner-local freeze/migrate/retire commands with recoverable receipts. Keep each owner writer separate.
- **WLD1-R2 — final unlink:** prove serialized no-new-reference fencing, idempotent replay, failure/retry, zero active references, and final unlink under the target PostgreSQL role/RLS configuration.

Required tests/evidence (not run by this audit):

- Target PostgreSQL migration replay, owner role grants, FORCE RLS, cross-tenant isolation, revoked Project role, and audit/outbox atomicity.
- 0..N links; duplicate/replayed add; CAS races for add/default/retire; default unset/change; rejection of default values not linked to that Project.
- New Run/Worktree reference racing with `retiring`: exactly one ordered result, with no reference accepted after the owner fence.
- Each owner command: exact replay returns the original receipt; changed payload, stale revision, revoked access, missing source/target, and owner outage fail closed; retries resume without duplicate effects.
- Replacement Repository must be an active association in the same Project; Worktree migration must verify actual local checkout identity/root. Explicit retirement handles references that cannot safely migrate.
- Unlink remains `retiring` on partial/unknown owner state; only fresh, all-owner receipts plus serialized zero-reference reconciliation permit closure. Historical event/Run identities are retained.
- Hostless Run remains supported; removing a ProjectRepository link cannot introduce a remote Branch prerequisite.

## 6. Limitations and revision record

- This report is a source/migration audit only. It did not run Rust tests, PostgreSQL, RLS/grant checks, migration replay, runtime worktree operations, or host-root verification.
- CodeGraph evidence and coverage limitation are recorded in the committed audit brief. The parent session had an up-to-date index and task-directed source snippets; this worker used the supplied evidence and targeted source/migration searches. No `check_index_coverage` tool was available to this session. Search findings are therefore scoped, not exhaustive.
- No external Project authority was available in the checked repository state. The recommended Project owner is a design recommendation; the actual canonical owner remains OPEN pending R0a evidence.
- **Revision 1 · 2026-10-25:** Initial WLD1-R0 owner/source audit from dev `bf374449`. No runtime or database completion is claimed.
