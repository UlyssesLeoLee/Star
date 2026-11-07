# W16 Implementation Decisions — Pre-merge Review

## Review scope

- Lane: `codex/w16-implementation-decisions-20261107`
- Base: `dev@89f2f5bd2f34a5977fdd843bbd9f2abc93c8ce66`
- Files: W16 SRS/BD/DD v0.9, Canvas Test Design v2.8, brief, phase report, dispatch audit.
- Out of scope: implementation/migrations, protected Rust candidate, root-worktree user changes, `main`, remote refs.

## Checklist

| Check | Result |
|---|---|
| Owner-local transaction and no cross-owner writes | PASS — design boundary only |
| `source_sequence=device_generation` and bootstrap floor | PASS — READY/pre-READY semantics reviewed |
| Snapshot/replay consistency and bounds | PASS — design contract; no runtime implementation claim |
| Authenticated service identity, capability, tenant scoping, and adapter mechanism evidence | PASS — proposal contract is explicit; adapter selection/evidence remains open |
| Lease, continuous checkpoint, gap handling | PASS — design and failure boundaries reviewed |
| W/T/M, append-only retention, RLS/grants | PASS — design classification reviewed; database validation remains NOT_RUN |
| Test mappings and NOT_RUN status | PASS — traceability reviewed; new cases remain NOT_RUN |
| Registered eight-path scope | PASS — coordinator verified the exact registered manifest |

## Findings

The first read-only reviewer pass happened before the phase brief was written and before the v0.9 edits existed. Its sequencing deviation is recorded in the brief and phase report; it is not treated as approval of this diff. A final read-only review of SRS/BD/DD v0.9, Test Design v2.8 and the phase report returned **APPROVE**. It confirmed that pre-READY state changes do not emit events or initialize floors, the first post-READY baseline absorbs old state, READY bindings begin with floor 0 and generation 1 event, and subsequent READY mutations commit the matching event atomically. Static validation found no broken relative links or malformed dispatch JSONL; `git diff --check` passed. Review scope is design consistency only: no Cargo, PostgreSQL, HTTP, Neon, Multipass, or K3s test was run.

## Verdict

**APPROVED FOR LOCAL DEV INTEGRATION.** The eight registered paths, current-text review, Markdown links, JSONL, whitespace check, and final diff are verified. No implementation or runtime validation is claimed; all new runtime acceptance remains NOT_RUN. The local dev fast-forward is a separate integration action.

## Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Created an honest pre-merge checklist; review and integration remain open. | W16 implementation-decision lane. |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录当前 v0.9/v2.8 文档的一致性复审 APPROVE、精确八路径清单与静态检查结果；明确新验收仍 NOT_RUN，待本地 dev fast-forward。 | READY 与 pre-READY 的事件/floor 边界完成最终交叉检查。 |
