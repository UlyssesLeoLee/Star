# Brief: WLD1-R0-OWNER-AUDIT-20261025

**Agent**: worker
**Phase**: WBS
**Created**: 2026-10-25 13:37:32

---

# WLD1-R0 ProjectRepository owner and source audit

## Objective
Audit the latest dev source and design to identify the canonical owner/writer for Project-to-Repository 0..N associations and the smallest safe implementation boundary for WLD1-R0. This is an evidence report only: do not change code, migrations, WBS, existing design files, or existing user briefs. Write only docs/reviews/WBS-WLD1-R0-OWNER-AUDIT-20261025.md and commit that report on this lane.

## Authoritative decisions and baseline
- User-approved: one Project may associate with 0..N repositories; default repository is optional UI preference only.
- User-approved: unlink is migration-first. Freeze new references; Run Directory and Worktree/Host owners migrate or explicitly retire their own references using idempotent owner commands and receipts; Project owner unlinks only after all receipts succeed and a fresh serialized zero-active-reference check. Failure/unknown remains retiring/fail-closed.
- Current dev baseline: 3dd01fcc11f771fac95a74d9e6ea4fe978bb1e65.
- Current WBS: docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md v0.45, §13, WLD1-R0..R2 still open.
- Do not use competitor product names in product copy. Keep actual service-provider labels only where they are real connector identities.

## Parent CodeGraph evidence (Verify, bounded)
- Project: C:\Users\leo19\orca\workspaces\Star\dev-converge-20261007; index 1,905 files / 37,350 nodes / 117,596 edges, reported up-to-date at this baseline.
- Queries: ProjectRepository in crates/domain-project/src/lib.rs; ProjectRepository trait/InMemoryProjectRepository; ProjectRepositoryEnvelope and validateProjectRepositoryEnvelope in frontend/src/app/worktree/ProjectWorktreeLifecycleControls.tsx.
- Exact source found: ProjectRepository currently exposes Project CRUD/list and policy/template persistence methods; InMemoryProjectRepository implements it. The frontend consumes a ProjectWorktreeRepository projection for list/create/import operations.
- Graph blast radius includes Project callers (30 indexed callers) and ProjectWorktreeLifecycleControls, called from worktree/page.tsx. The graph did not identify a persistent ProjectRepository association writer.
- MCP in this session exposes codegraph_explore but not check_index_coverage. Bounded fallback search was run with rtk rg across crates, db/migrations, frontend/src for project_repository/repository_authorization/repository_id+project_id. It found the domain trait and lifecycle API plus SCM repository_master and Run/Worktree binding usage, but no dedicated ProjectRepository association writer. This is a bounded search, not proof of absence.
- Existing runtime surfaces found by fallback: scm.repository_master stores repository identity; multica.engineering_run and Run/Worktree binding tables pin repository IDs; star-api-rest worktree_lifecycle API reads a Project repository projection and receipt validates Run/Project/Repository/binding. Explain how these should stay owner-separated.

## Audit questions
1. Which bounded context owns the ProjectRepository relation and canonical authorization, based on current code and migrations? Distinguish Project identity owner, SCM repository identity owner, Run Directory, and Host/Worktree owner. If code cannot establish this, state the exact missing source-of-truth and recommend the minimal ADR/contract needed before implementation.
2. Is there a canonical Project aggregate database table/service or only in-memory ProjectRepository? Identify exact paths and evidence.
3. What FK/tenant boundary and authorization fact can safely support 0..N links without inventing a Project table or cross-owner writes?
4. How should optional default selection be persisted, versioned, and authorized so it never becomes a hidden Run binding?
5. Specify minimal owner commands/outbox/inbox/idempotency/receipt identity and retiring state transitions consistent with user decision; name each owner and what it may write.
6. Give implementation readiness verdict: safe to implement WLD1-R0 now, or blocked pending explicit source-owner decision. Include exact next lane paths and disjointness.
7. Recommend narrowly scoped PostgreSQL/RLS/API tests and failure cases; do not claim tests ran.

## Evidence and safety
Read the relevant source/migrations and cite paths, symbols, and line numbers in the report. Treat user-created files/worktrees as read-only. Do not change the three untracked reviewer briefs or D:\Star. Do not merge or push.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
