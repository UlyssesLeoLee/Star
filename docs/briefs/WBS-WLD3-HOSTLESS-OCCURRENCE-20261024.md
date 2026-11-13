# Brief: WBS-WLD3-HOSTLESS-OCCURRENCE-20261024

**Agent**: rust-schedule-implementation-worker
**Phase**: WBS-WLD3-HOSTLESS-OCCURRENCE-20261024
**Created**: 2026-10-24 14:59:33

---

# WLD-3 hostless Schedule occurrence lane

## Task
Implement the next bounded WLD-3 slice: canonical hostless Engineering Run Schedule Rules must be eligible for deterministic occurrence materialization and durable idempotent persistence when no remote Branch identity exists. The required identity remains tenant + Project + Engineering Run + bound Worktree + Task + pinned Profile/HookSet versions; remote Branch is optional metadata only.

This slice does not enable executor launch. Keep Run admission/execution fail-closed unless current Run authorization, owner-owned execution identity, InfrastructureBinding with exact cluster identity/generation, capacity reservations, Runtime sandbox, Hook gates, and readiness are all independently proven. Update the WBS wording to distinguish occurrence materialization from admission/execution; do not preserve a remote-Branch requirement as a hidden prerequisite.

## Owner
Implementation worker for this isolated lane. The parent coordinates review and integration.

## Lane
Branch: codex/wbs-wld3-hostless-occurrence-20261024
Worktree: C:\Users\leo19\.codex\worktrees\wbs-wld3-hostless-occurrence-20261024\Star
Base: dev @ 25659752f28e4e1a6446915b36f2ce8a387ca1c1
Integration target: dev
MAGOS manifest: C:\Users\leo19\.codex\worktree-manifests\Star-WLD3-hostless-occurrence-20261024.txt

## Required implementation
- In domain validation/materialization, accept branch_id=None when the canonical Run/Worktree/Task/Profile/HookSet identities are valid; continue rejecting missing or mismatched required identities and preserve branchful compatibility.
- In PostgreSQL persistence and persisted-rule comparison, use null-safe branch matching and persist the canonical target snapshot, including explicit null remote metadata, without weakening tenant/RLS/idempotency checks.
- Add one forward-only migration that relaxes only the remote-Branch snapshot constraint while preserving required canonical local identities, update the migration runner, and add positive branchless + negative malformed identity coverage in unit/PG-backed or migration-catalog tests.
- Keep Schedule execution closed: no bypass of run-as reauthorization, admission owner saga, InfrastructureBinding, or runtime readiness.
- Synchronize requirements, Basic Design, Schedule SRS/BD/DD, Worktree Group Detailed Design and this WBS plan; write a seven-section phase report with verified results and explicit unverified gates. Use neutral product terms only.
- Preserve CypherGraph relationship headers in edited Rust files and run its Guardian workflow for every changed code file.

## Validation
Run changed-file formatting and diff checks; focused Rust unit/integration targets with Rust 1.99.0 and -j 4; the repository Schedule runner against a disposable PostgreSQL fixture only if its required configuration is present. Report exact command, result and remaining target-DB/runtime gates. Do not claim runtime/production closure from static or fixture checks.

## Current lane status

- Implemented the bounded source and synchronized the seven primary design documents plus WBS/report records within the 16-path manifest.
- Static migration-catalog tests passed 3/3; Python AST parsing passed; Cypher Guardian checked 5 changed source files and the SQL migration header was stamped/validated with its digest helper; `git diff --check` passed.
- Rust/Cargo 1.99.0 and local disposable PostgreSQL acceptance completed: the phase runner passed 47/47; domain tests 31/31, two focused Schedule Rule API tests 1/1 each, and Schedule PostgreSQL integration 6/6. This is isolated local evidence; target DB, production authorization, Schedule admission/executor, dev integration, push and cleanup remain outside this brief.
## Handoff
Before handing back, report branch/worktree/base/current HEAD, exact changed files, implementation decisions, tests actually run, remaining blockers, and whether the lane is ready for independent review. Commit the bounded lane after its own checks pass; do not rebase/merge, push, or clean up this worktree.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
