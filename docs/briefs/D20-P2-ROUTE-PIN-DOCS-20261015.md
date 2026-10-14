# Brief: D20-P2-ROUTE-PIN-DOCS-20261015

**Agent**: worker
**Phase**: 9F4C-D20-P2
**Created**: 2026-10-14 22:34:40

---

Create a documentation-only D20 P2 lane based on dev@6b23c29888ad7de1588adca60ddd83cb75bc61f9. This lane is independent from the active code lane `codex/d20-route-pin-ops`; do not edit code, migrations, SQL fixtures, or the implementation lane's files.

Goal: make requirements, basic design, detailed design, implementation plan, and phase status accurately define the still-open stale immutable route-pin mismatch observability/recovery capability. Inspect the current docs before editing and change only the narrowest applicable D20 contract/design/status docs (likely docs/requirements.md, docs/basic-design.md, the D20 Infrastructure/Schedule detailed design, docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md, docs/plan/master-implementation-plan.md, and a focused D20 status/report if needed).

Required design invariants:
- A stale route-pin mismatch must not be silently discarded or counted as an empty successful batch; distinguish zero eligible rows from rows blocked by route mismatch.
- Expose bounded operator-visible counters/status scoped by tenant and event/route identity, without secrets or payload leakage.
- Persist blocked/quarantine evidence durably and append-only with owner authorization, idempotency, audit identity, and an explicit recovery/disposition path.
- Never mutate/retarget an immutable route pin, dual-publish, or fallback to the other transport. Recovery may retry only on the originally pinned route after the pinned endpoint/profile is restored and revalidated; otherwise require an explicit owner-governed terminal disposition/new business event with a new immutable route.
- Preserve attempt/lease fencing and owner-local transaction boundaries. Keep production wiring, deployment grants, TLS, BI/Benchmark, performance, and Schedule/Run execution marked open unless evidence in this branch proves them.
- Do not claim implementation or tests passed. This lane only defines requirements/design and marks D20 P2 as in progress/open pending the code lane.

Include a concise change matrix, known gaps, exact source references, and revision history. Avoid history claims that lack git evidence. Run only documentation consistency/registry checks needed for the changed docs. Commit with author Ulysses <ulysses@mavis.local> and a clear D20 P2 docs message. Send the commit SHA and verification facts to the parent; do not rebase, merge, push, delete branches, or modify other worktrees.

CodeGraph context from parent: projectPath `C:\Users\leo19\orca\workspaces\Star\dev-converge-20261007`, generation/coverage API unavailable in current tools; two explore calls completed. Candidate source showed `InfrastructureOwnerNativeResultPublisher::publish_due_for_tenant` and the exact-append path; previous plan/review establishes claim-time route mismatch filtering and missing operator blocked count/recovery. Parent fallback read found existing docs/requirements AC-D20-INFRA-RUN-RESULT-PUBLISH-05, basic-design.md D20 follow-on status, implementation-plan 搂6.138, and docs/reports/PHASE-9F4C-D20-NATIVE-PUBLISHER-REVIEW.md 搂P2. Existing P1 fix is commit 8506766e. Do not repeat the previous general D20 doc sync; add the missing P2 contract/status narrowly.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
## 7. Documentation lane outcome

### Change matrix

| Layer | Update |
|---|---|
| Requirements | Added separate empty-eligible and route-mismatch outcomes, cross-provider diagnostics, authenticated operator visibility, append-only evidence, same-route exact-wire retry and explicit terminal-disposition acceptance. |
| Basic design | Defined bounded aggregate and paginated per-event status, operator surface and immutable-route recovery; status remains open. |
| Detailed design | Added D20 P2 owner transaction/evidence model, cross-provider state discovery, recovery transitions, source references, acceptance IDs and pending verification cases. |
| Implementation plan / master plan | Added the implementation sequence and retained D20 P2, production wiring, BI/performance, NATS rollback and execution gates as open. |
| Review report | Replaced the informal P2 recommendation with the accepted contract and kept the finding unresolved. |

### Exact source references

- crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::claim_native_due filters claims by immutable current-route match.
- crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs::publish_due_for_tenant reports claim count, which cannot distinguish no candidate from mismatch-blocked rows.
- The native claim path only exposes the issue when that publisher is called; JetStream safely skips native-pinned events, leaving a cross-provider diagnostic gap.
- docs/reports/PHASE-9F4C-D20-NATIVE-PUBLISHER-REVIEW.md §2 P2 and §8 status row document the baseline finding.
- crates/star-api-rest/src/auth/oauth/handlers.rs::handle_client_credentials (256–295) accepts requested scope and issue_tokens (339–360) persists it into token claims/records; crates/star-api-rest/src/group_api.rs::require_scope (627–637) checks exact tokens without validating permission provenance.

### Known gaps

- This lane changes documentation only. It does not implement or verify cross-provider mismatch diagnostics, the authorized operator API/UI, dedicated least-privilege capabilities, verified actor-to-tenant resolution, durable blocked/quarantine records, operator recovery or terminal disposition. The current star-api-rest has no Infrastructure operator route/scope. Token issuance copies requested scope and exact-token scope checks do not prove role authorization; operator permissions must come from verified client/user role provisioning or a separately authenticated internal operator identity.
- The parallel code lane reports an in-progress quarantine table plus bounded RLS-protected read seam. This is partial work only; P2 remains open without cross-provider diagnostics, an authenticated operator surface, role-bound permissions, separate read/recover/dispose capabilities and verified actor-to-tenant resolution.
- No Rust, SQL, PostgreSQL, TLS, BI/Benchmark, performance, NATS rollback or Schedule/Run execution evidence was produced here.
- CodeGraph coverage validation is unavailable in the current tool surface; source references are focused symbol evidence and are not an exhaustive source inventory.
- The generated filename suffix 20261015 is the dispatcher task ID, while the brief creation time and all revision dates are 2026-10-14; renaming it would break the dispatcher audit reference.
- rtk was not installed or resolvable in this worktree; documentation checks used PowerShell, rg, Git diff checks and the script registry checker.

### Revision history

| Version | Date | Revision | Trigger |
|---|---|---|---|
| v0.1 | 2026-10-14 | Ulysses — Mavis review | Dispatch brief created for the documentation-only D20 P2 contract lane. |
| v0.2 | 2026-10-14 | Ulysses — Mavis review | Synchronized requirements, basic/detailed design, implementation plan and D20 P2 review status without claiming implementation. |
