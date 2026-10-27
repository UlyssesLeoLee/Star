# Brief: WBS-DESIGN-BASELINE-TRACE-AUDIT-20261026

**Agent**: explorer
**Phase**: WBS-DESIGN-BASELINE-TRACE-AUDIT-20261026
**Created**: 2026-10-26 18:36:16

---

## Objective
Perform a read-only WBS W1 design-baseline trace audit against the immutable local dev snapshot at b1aa0f8f3536f5ab49b2a2af3f02ad6b4f707aa4. Write only the assigned audit report. Do not edit canonical requirements/design documents, WBS, source code, tests, or any other file.

## Source scope
Use only this lane's clean checkout at the pinned dev commit. Review the active requirements/basic design and relevant detailed design, implementation-plan, and owner-contract documents necessary to assess the current architecture. Do not read or use D:\\Star's uncommitted content or another active worktree as evidence. The repository has no indexed CodeGraph project; keep this a document-only audit and do not infer implementation facts from source names.

## User-accepted architecture invariants to trace
- Left navigation is Project -> Engineering Run -> Worktree -> Branch regardless of remote host connection. A Run can exist, be authorized, listed, and managed without a remote Branch. Branch under Worktree means observed local checkout ref/state.
- A Project is Star-owned and may associate with zero or more repositories. Active Run/Worktree references must migrate before a repository association is removed.
- Selecting a Run Worktree opens that Run's right-side app tabs and focuses CLI on that checkout. Inbox, task management, Canvas, workflow, BI/Benchmark, Hooks evidence, and plugin apps are Run-scoped siblings; CLI opens from a task card. Bottom chat scope is explicitly WORKTREE or GLOBAL.
- Product UI/help/error/marketing text must avoid competitor product names or imitation/comparison copy; real connector identity may show its provider in connector settings.
- Native Hooks are a required Rust fail-closed security boundary, visually configured under Advanced Settings -> Hooks, inherited/snapshotted across Project/Worktree/Run, and linked to execution and BI evidence.
- Host Infrastructure Manager uses Multipass to manage Linux guests for K3s; Docker Desktop is not a prerequisite. The current test phase has a total two-record Multipass inventory cap, including Stopped/Deleted records, with locking, fail-closed inventory checks, and no automatic purge. Cluster identity/generation is required only when a Run selects infrastructure placement; hostless Runs do not need remote Branch or infrastructure binding.
- Agent/Memory/Skill/Context/Validation/Loop and plugin/LangGraph capabilities require versioned, budgeted, replaceable contracts. Resume rechecks authorization and Hook policy. NATS is a removable transport adapter on the path to the user's Rust server; owner facts remain provider-neutral and durable.
- Rust desktop memory/parallelism needs explicit bounded queues/caches/logs, pagination/virtualization, backpressure, fairness, cancellation/drain, and measured workload evidence. Rust MSRV is 1.99.0.
- Schedule creator is immutable run-as and each trigger/retry/resume rechecks Project/Run execution permission, failing closed on revocation or unavailable authority.

## Deliverable
Create docs/reviews/WBS-DESIGN-BASELINE-TRACE-AUDIT-20261026.md only. Include: purpose and pinned baseline; a requirement-to-document traceability matrix; confirmed aligned statements; concrete contradictions or omissions with exact file/section anchors; unverified implementation/acceptance claims and their proper OPEN/NOT_RUN status; prioritized W1 follow-up edits and dependencies; limitations. Do not claim W1, WLD-7, W11, W8, VCD-0, W14, or product acceptance complete. Do not mention competitor names in the report.

## Validation
This is a document-only read. Verify the report's cited paths/sections exist in this checkout and run a whitespace-only diff check. Do not run tests, databases, VMs, network fetches, or alter repo state beyond the single assigned report file.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
