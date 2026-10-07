# Brief: ERUN-P3-9F4D-3D6-QUARANTINE-AUDIT

**Agent**: explorer
**Phase**: 9F4D-3D6
**Created**: 2026-10-08 00:18:15

---

Read-only bounded design audit for Phase 9F4D-3D6 durable poison quarantine. Repository candidate: D:/orcaWork/Star/phase9f4d3d6-poison-quarantine, branch UlyssesLeoLee/phase9f4d3d6-poison-quarantine, base dev HEAD 588c743aa0d9c78bdc4115ba64249e842cae2444. Do not modify files, run tests, spawn agents, or touch D:/Star (it has user changes).

Source evidence already inspected: crates/run-owner-runtime/src/capacity_result.rs parses before persistence and then calls Message::double_ack; db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql defines tenant-scoped FORCE-RLS Run-owner inbox/outbox only; SRS-AUTOMATION-SCHEDULE-API-001.md SCHED-API-026 and DD-AUTOMATION-SCHEDULE-API-001.md §6.12 explicitly keep malformed/unmatched messages fail-closed and prohibit production enablement without durable quarantine; implementation plan §6.100 names poison quarantine as open. async-nats 0.50.0 docs show broker Message::info() exposes stream, consumer, stream_sequence; Message::double_ack_with(AckKind::Term) confirms server receipt; Term stops redelivery without treating the message as successfully processed.

Review the source and recommend a safe ownership/data contract for malformed or unmatched messages. Never trust tenant/run fields from the bad envelope as authority. Consider stable dedupe from trusted JetStream metadata, persistence-before-Term ordering, bounded bytes, W/T/M and privacy/retention, RLS/owner boundaries, crash/replay behavior, and whether full raw bytes or metadata+digest are needed. Return a concise recommendation, exact paths/symbols, and unresolved decisions. Treat the missing .codegraph/ directory and unavailable codegraph MCP as a disclosed graph limitation; source-level PowerShell/ripgrep fallback is the evidence path. No historical claims without git evidence.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
