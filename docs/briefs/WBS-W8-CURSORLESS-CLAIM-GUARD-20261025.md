# Brief: WBS-W8-CURSORLESS-CLAIM-GUARD-20261025

**Agent**: worker
**Phase**: WBS
**Created**: 2026-10-25 14:31:44

---

## Objective
Resolve the independent W8 review's P2: a historical or newly recorded quarantine claim with NULL source_cursor can currently coexist with a later non-NULL cursor claim at another sequence in the same source log generation. The test fixture also maps cursor bytes from sequence, so cursor uniqueness is not independently exercised.

## Baseline and evidence
- Current W8 fix: 07a896e3561affb885594b9fb53b65b199921fa6 on lane UlyssesLeoLee/w8-source-position-claim-fix-20261025.
- Independent review addendum: e098fe4061e23045049aa08c0c2bef75cec6561a in D:\orcaWork\Star\w8-remediation-review-20261025. The primary sequence-key P1 is source-level closed; PG remains blocked/unrun.
- No PostgreSQL database URL is configured and psql is not on PATH. Do not claim database runtime proof.

## Required outcome
Keep the same registered lane and exact three owned paths:
- db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql
- crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs
- docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md

Implement a provider-neutral fail-closed barrier for unknown cursor history within the same tenant/consumer/provider/log/generation:
- Any existing NULL-cursor claim means a new non-NULL-cursor claim at a different sequence is rejected until the source uses a fresh generation or history is explicitly reconciled.
- Exact replay of an already claimed non-NULL position remains allowed; position conflicts continue through the shared unique claim logic.
- Avoid serializing normal cursor-bearing traffic: use compatible shared/exclusive advisory-lock modes keyed to source-log generation (non-NULL claims share; NULL-cursor claims take exclusive) so a concurrent first cursorless claim cannot race past the unknown-history barrier. Add an indexed NULL-cursor lookup if needed.
- Keep quarantine raw-wire-free, claims append-only, and current tenant RLS/grants/search_path boundaries intact.
- Add an actual PostgreSQL regression for same cursor at different sequence and for a cursorless quarantine fact followed by a different-sequence cursor-bearing delivery; assert no second receipt/ACK. Keep this test ignored unless the dedicated disposable DB is configured.
- Document the conservative generation rollover/reconciliation behavior and validation status in the phase report.

Do not widen the manifest or edit other W8 source/candidate files. If this behavior requires another path, stop and report it rather than editing it. Run focused Rust 1.99 fmt, test-target compile/no-run, default test and Cypher Guardian as available; list PostgreSQL as blocked/unrun. Do not merge, rebase, push, or use external code review. Commit only the three registered paths when complete and leave W8 OPEN until the final independent review and real PG/RLS gates pass.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
