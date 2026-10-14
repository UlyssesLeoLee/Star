# Brief: D20-P2-CROSS-PROVIDER-20261015

**Agent**: worker
**Phase**: 9F4C-D20-P2
**Created**: 2026-10-15 00:04:55

---

## Objective
Implement the next bounded D20 P2 slice: make stale immutable native route pins observable even when the JetStream publisher is the caller and safely skips native-pinned events. This is an implementation task, not an architecture-only mock.

## Baseline and ownership
- Base: local `dev` at `1ef0e56477daa9092bd735f95e437e852ba7b6cd`.
- Your exclusive files: `crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs`, `crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs`, and focused Infrastructure tests/migration only if required by the design. The coordinator will handle requirements/basic/detailed design and plan synchronization after reviewing the implementation.
- Do not edit `dev` or other lanes. You are not alone in the repository; do not revert others' changes. Accommodate changes from other lanes.

## Required behavior
1. JetStream's publish result must not make a tenant look idle when due native-pinned work exists or route-pin diagnostics cannot be resolved. Distinguish: no eligible work; native-pinned work observed; stale native pin durably quarantined; diagnostics unavailable. Keep results bounded (count and bounded page; never event IDs as metric labels).
2. Compare a pin only against a trusted current route snapshot produced by the existing resolver-bound route type. Never synthesize endpoint/profile identity from tenant ID, caller scope, or arbitrary route strings. If no trusted snapshot is available, report unavailable/fail closed rather than zero blocked.
3. Preserve exactly-one-provider delivery. JetStream must continue skipping native-pinned items, must not publish them, must not retarget/rewrite a route pin, and must not dual-publish or fallback.
4. Keep all writes in the Infrastructure owner's transaction with tenant scope, immutable append-only quarantine, active-lease/receipt fencing, and bounded batch/page limits. Add a focused PostgreSQL regression that invokes the JetStream-side claim/outcome path without first invoking the native publisher, creates a stale pin, and proves the blocked diagnostic is returned and no message is claimed for JetStream. Also cover empty/no-stale and active lease/receipt safety where feasible.
5. Do not implement operator authentication, recovery, terminal disposition, production startup, BI dashboards, or NATS cutover in this slice. Keep those gates open and fail closed.

## Quality and repository rules
- Read local `AGENTS.md`, relevant headers, tests, and referenced D20 requirements before editing. CodeGraph index at this lane's base was reported fresh, but available graph query returned unrelated symbols; use exact source search as fallback and state coverage limitations.
- Keep memory/parallelism bounded; no background poller, unbounded queue, or tenant-wide unbounded scan. Preserve Rust 1.99.0 baseline and test only with safe disposable PostgreSQL preflight.
- Reconcile Cypher headers from code evidence, then run the guardian script. Do not claim unavailable DB/runtime evidence.
- Run focused Rust formatting/check/tests and PostgreSQL test only if a safe disposable test DB is available; report exact commands/results and blockers.
- As a [M] bounded task, do not change automation registry unless the repository's task classifier says it must be [P].
- Commit the lane with author Ulysses <ulysses@mavis.local>; include this brief path in the commit message.

## Acceptance
A test demonstrates JetStream-side visibility of a native stale route-pin mismatch even when native publisher is never invoked, and the publisher never sends that event through JetStream. Existing route pin immutability, lease fence, tenant scope, and exact-wire tests continue to pass. Design docs remain coordinator-owned until code review.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
