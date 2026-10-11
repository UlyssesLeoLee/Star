# Brief: phase9f4d3d5-run-owner-schema-audit

**Agent**: explorer
**Phase**: PHASE-9F4D-3D5
**Created**: 2026-10-07 19:18:45

---

Read-only architecture audit for the next planned 9F4D slice. Inspect the current Run/Execution owner schema and code for multica.task_execution_run, automation.occurrence, automation.occurrence_dispatch, automation.schedule_run_outbox, plus the Infrastructure capacity result event contract. Recommend the narrowest safe Run-owned durable Inbox/acknowledgement persistence and state transitions for capacity_accepted, capacity_rejected, capacity_released, and capacity_expired. State exact table owners, keys, FK needs, idempotency rules, append-only Transaction facts vs short-lived Work state (W/T/M), and what must remain fail-closed. No edits. Cite exact repository file paths and line numbers; distinguish observed evidence from inference and unknown target DB/Auth facts. No fabricated history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
