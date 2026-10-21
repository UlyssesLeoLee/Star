# Brief: WLD1-LIFECYCLE-PROVIDER-20261022

**Agent**: codex
**Phase**: WLD1
**Created**: 2026-10-22 02:33:10

---

## Goal
Implement the next reviewable WLD-1 Worktree lifecycle provider slice from the current dev baseline. This task runs on an isolated branch based on dev and may change only the files registered in the MAGOS lane manifest:
- crates/star-api-rest/src/group_api/worktree_lifecycle.rs
- crates/star-api-rest/src/group_api.rs
- docs/reports/WLD1-LIFECYCLE-PROVIDER-20261022.md
- docs/reports/WLD1.log (dispatcher audit)
- docs/briefs/WLD1-LIFECYCLE-PROVIDER-20261022.md (this brief)

## Parent graph evidence and bounds
The indexed dev CodeGraph status was up to date (1,897 files, 37,232 nodes, 116,949 edges). MCP CodeGraph returned Transport closed; the available MCP did not expose check_index_coverage. Targeted CLI graph exploration found ProjectWorktreeLifecycleProvider at crates/star-api-rest/src/group_api/worktree_lifecycle.rs:182 and its group_api.rs wiring. CodeGraph reported no covering tests for this provider. A separate ExternalWorktreeImport / RealExternalWorktreeImport exists in crates/worktree-shared-dir/src/external_worktree_import.rs, but that file is outside this lane; do not edit it. The WBS v3.9 §2 and §5 say Worktree lifecycle closure is still open: provider, trusted repository root, authorization/retention, lifecycle receipts, reconciliation, cleanup/drain and target DB acceptance remain incomplete.

## Required design constraints
- Keep identity and authorization Project/Engineering Run scoped, hostless Run supported, and Worktree optional. Do not make a cloud Branch a prerequisite or infer remote identity from a local branch.
- Only use an established trusted repository-root/provider boundary. Never accept arbitrary paths or commands from request payloads. Do not invoke arbitrary host shell, expose Multipass, or bypass the native Hook/authorization boundary.
- Fail closed if the provider, Run binding, current grant, expected version, retention lock, lease/claim/PTY/process drain or reconciliation evidence is missing or ambiguous.
- Do not implement successful mock, seed, in-memory or preview behavior as a production provider. Do not label a source slice production-ready.
- Destructive cleanup must remain unavailable unless the fresh retention/lease/claim/PTY/process/Runtime drain facts are atomically verified and the operation is recoverable. Do not physically delete checkouts or purge VMs.
- Preserve the codebase Cypher header and synchronize comments with actual behavior.
- Stay within the five manifest paths. If a correct implementation requires another path, stop and report the exact dependency; do not edit around the manifest.

## Deliverable
Review the exact trait, route wiring, source contracts and existing acceptance tests; implement the smallest real, safe lifecycle capability that can be supported by the current architecture, with tests in an already-authorized source file. If no safe real provider can be built because trusted host/repository identity or lifecycle owner is absent, make the API fail closed, add precise tests for the refusal behavior, and document the missing provider contract instead of inventing one. Add docs/reports/WLD1-LIFECYCLE-PROVIDER-20261022.md using the repository seven-section report structure. Distinguish source/test evidence from target DB, host provider and runtime acceptance.

## Validation and integration
Inspect the current files before editing. Use focused tests or static checks that are available; record exact commands/results. The parent found cargo, rustc, psql, pg_ctl and initdb absent from PATH, while Python 3.13.7, RTK 0.50.0 and Multipass 1.16.4 are available; verify local availability before relying on them. Do not create VMs or databases. Do not add dependencies or install tools. Run RTK-wrapped Git commands and report diff/status. Commit the lane only when the reviewable slice and report are ready, using the repository author convention. Do not rebase or merge; parent owns integration.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
