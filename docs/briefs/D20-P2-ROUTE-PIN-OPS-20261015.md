# Brief: D20-P2-ROUTE-PIN-OPS-20261015

**Agent**: worker
**Phase**: D20-P2
**Created**: 2026-10-14 22:29:14

---

Implement the next D20 phase slice for stale native route-pin operations on this isolated `codex/d20-route-pin-ops` worktree. The approved plan is `docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md` §6.138 and master plan D20: the existing publisher silently filters route-pin mismatches, so an empty batch is ambiguous; P2 requires operator-visible blocked state and a safe recovery/quarantine path.

Ownership: own the Infrastructure native-result delivery diagnostics and durable quarantine implementation, its migration/SQLx acceptance, and only the directly required automation runner wiring. Do not edit requirements/basic/detailed design or the dev branch; the coordinator will synchronize those after review. Avoid unrelated D19/D21 changes.

Hard invariants: never mutate or retarget an immutable provider/Profile/endpoint/wire pin; never dual-publish or fallback NATS/native; require Infrastructure tenant RLS and owner-local transactions; keep diagnostic output bounded and redact payloads/secrets; quarantine must be durable, append-only/WORM, idempotent, and must not override an active lease or delivered receipt; every admin/recovery action must fail closed unless it can reuse an existing trusted authorization contract. If no canonical operator authorization surface exists, do not invent a bypass: implement the safe owner-store seam and document the API integration blocker precisely.

Inspect the existing Postgres store and `infrastructure_native_result_delivery_pg` test. Add tests that distinguish “no due events” from stale-route blocked events and prove quarantine cannot change the pinned route or affect another tenant. Keep the migration replayable and classify every new table as Work/Transaction/Master; apply migrations twice in the disposable PostgreSQL runner when available. Run focused Rust formatting/check/tests and the existing disposable PG lane if its tools are available; record exact Rust version and whether the formal 1.99.0 gate ran. Do not start NATS or Multipass, and do not claim production startup wiring, live native append, BI parity, or execution readiness.

CodeGraph was present in the parent checkout, but two targeted explores returned unrelated symbols; source discovery therefore fell back to focused `rg` and line reads. There is no exposed `check_index_coverage` tool in this session. Relevant paths: `crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs` (`claim_native_due` around 524–679; `record_failure` around 942), `crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs` (`publish_due_for_tenant` around 208), `crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs`, migration `db/migrations/2026-10-14-infrastructure-native-result-delivery.sql`, automation runner `scripts/automation/phase9f4d_run_owner_consumer.py`. P2 review evidence is in `docs/reports/PHASE-9F4C-D20-NATIVE-PUBLISHER-REVIEW.md`.

Before finishing, update CypherGraph Guardian headers for changed code, self-review your diff, provide the exact acceptance results and remaining gates, and commit only your scoped changes on this worktree with the repository-prescribed author. You are not alone in the repository: the coordinator owns the dev docs and later integration; do not revert or modify anyone else’s files.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
