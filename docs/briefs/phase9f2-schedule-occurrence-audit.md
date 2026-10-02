# Brief: phase9f2-schedule-occurrence-audit

**Agent**: explorer
**Phase**: phase9f2-schedule-audit
**Created**: 2026-10-02 18:46:55

---

## Objective
Read-only audit the current Schedule/Automation occurrence architecture for the active Worktree-first Group Apps goal. Recommend the next production-oriented implementation slice after Phase 9F1, with explicit code/migration/API evidence and blockers.

## Supplied evidence and repository state
- Current `dev` and `origin/dev` are both `e794d834498533176498cb0b649152ef2c64116e` (reconfirm from assigned worktree).
- No `.codegraph/` exists and no codebase-memory graph tools are exposed in the parent session; use focused source reads/rg fallback, do not claim graph coverage.
- `crates/domain-automation/src/lib.rs` defines `AutomationRule`, `AutomationExecution`, and `InMemoryAutomationService`; focused search did not find an AutomationOccurrence type.
- `crates/star-scheduler/src/lib.rs` is a CPM/WBS dependency scheduler, not an occurrence clock/dispatcher.
- `db/migrations/2026-09-30-worktree-task-execution-run.sql` includes nullable rule/version/occurrence fields and a unique `(tenant_id, automation_occurrence_id)` index in TaskExecutionRun; current search did not find a durable AutomationOccurrence owner table.
- Root detailed design §14.10 assigns rule/occurrence ownership to `domain-automation`; the implementation plan Phase 9 row still lists occurrence dispatcher, fencing/idempotency, Run snapshot admission, Loop runtime and fairness as open.

## Questions to answer
1. What exact existing types/ports/migrations and ownership seams should a durable AutomationOccurrence use?
2. Which fields/policies are already modeled on AutomationRule, and which are absent for target scope, timezone, overlap/misfire, retry/deadline, rule-version snapshot, lease/fencing and stable idempotency?
3. Which existing Run admission API can atomically accept occurrence identity and prevent duplicate TaskExecutionRun creation? Identify exact functions/routes/constraints and any missing transaction integration.
4. Recommend one next phase bounded by production correctness, not a mock/in-memory replacement. Split code/migration/API/integration gates and state when identity/target DB availability is genuinely required.
5. Identify the smallest independent work lane, if any, that can safely run in parallel with parent changes.

## Constraints
- Read-only: do not edit, stage, commit, push, install dependencies, or run tests.
- Report exact file paths, symbols and line ranges; label inference separately from direct evidence.
- Do not treat unit tests, nullable schema fields, or in-memory stores as production closure.
- Do not remove tests or persisted records.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
