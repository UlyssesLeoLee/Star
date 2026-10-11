# PHASE-9F4C-D12 — Schedule owner admission handoff

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses — Mavis接手审核
> Status: isolated owner handshake verified; production delivery/execution and full phases remain open.

## §0 Purpose

Connect the existing Schedule lease repository to Run preparation, confirmation and compensation through owner-local immutable request/result/resolution facts.

## §1 Change matrix

| Area | Change | Evidence boundary |
|---|---|---|
| Schedule adapter | Persist original request/fence; validate and resolve actual Run result | No broker publisher/receiver or authenticated deployment |
| SQL capability | Own link/event/Outbox/Inbox/resolution in one transaction | No Run or Infrastructure writes |
| Data | Three T tables, paired receipt/resolution FK, append-only, FORCE tenant RLS | Shared-DB read constraints remain; no physical database split |
| Integration | Actual claim → request → Run preparation → result → confirm → Run demand; lost lease → compensate → Run release | Rule/occurrence and clock sources are fixtures |
| Retry | Three typed rejected supplier results, actual Schedule retry/exhaustion | Does not prove rejected broker producer |
| Design | Requirements/basic/detailed/data/phase/automation and W/T/M classification synchronized | Full product goal remains active |

All handoff wires are bounded to 16 KiB. Schedule uses its existing bounded pool with 5-second lock and 10-second statement timeouts. Per-occurrence advisory locking and dispatch row locks serialize handoff state. Exact replay preserves persisted bytes and identity; it does not renew authority or create another logical attempt. Accepted results create one paired confirm/compensate command. Confirmation checks the original live lease/generation, slot/dispatch deadlines, cancellation and link before and after its writes. A failed final Outbox insert rolls back the link, event, Inbox and all Outboxes. Rejected results change current dispatch only under the original live fence, with bounded exponential backoff and one exact terminal/TTL timestamp.

The configured source UUID is injected trusted scope, not authentication. Run separately consumes the resolution and rechecks its current authorization, binding, policy, Task state and Project budget. Infrastructure separately reserves Host/cluster budgets. No executor or VM permission is created by the Schedule transaction.

## §2 Verification summary

PostgreSQL 18.6 / Rust 1.99.0 isolated runner: 62/62 passed. Audit: `target/phase9f4d-run-owner-consumer/0dcfeed4-74a6-433d-bc7f-a4cb4aec8b5c/audit.jsonl`. Binding/admission SQLx targets passed. Verified exact request/result replay, wrong source/digest rejection, final resolution failure rollback, actual Schedule link/confirmation and Run demand, original lease expiry plus real adapter reclaim and Run compensation, three rejected result retries through exhaustion, tenant invisibility and absence of cross-owner mutation grants. Rule/occurrence are canonical supplier fixtures; clock expiry is an admin fixture. Every runner stopped its disposable PostgreSQL; zero Multipass inventory or production DB changes.

隔离PostgreSQL18.6/Rust1.99.0 runner 62/62通过（audit `target/phase9f4d-run-owner-consumer/0dcfeed4-74a6-433d-bc7f-a4cb4aec8b5c/audit.jsonl`）；两个包all-targets check通过，Run-owner strict Clippy通过，库测试Run-owner15/15、star-pg-adapter35/35通过，owned-file fmt、Python编译、Cypher图头及diff检查通过。Star-pg-adapter独立strict Clippy未通过：lib150、lib-test152项诊断来自11个未改动文件，Schedule adapter没有诊断；详见`target/phase9f4d-run-owner-consumer/d12-pg-strict-clippy.log`，不宣称整个包/仓库quality gates全绿。CodeRabbit权限拒绝，不计为独立审核通过。

The first all-target compile failed because the SHA-256 return type does not implement LowerHex; fixed fixture encoding without adding a dependency. Later attempts hit intermittent Windows LNK1104 creating generated test executables; disk capacity and absence of matching live processes were checked, with no cache/security policy deletion. Serial reruns reached the actual tests. A compensation test originally followed the successful case's retained Project reservation and was correctly rejected by the budget gate; moved compensation before positive confirmation, without changing production quota checks. Failed evidence remains in the runner output directories (`d67e4271-db2d-4823-9f77-0eccb6dc8ad3` and the serial/fixed summary files).

## §3 Known gaps

Production identity/role provisioning, authenticated Schedule/Run message transport, request/resolution publisher, result receiver, durable delivery/recovery and cross-owner cancellation/expiry/release remain open. Cancellation/expiry/link-conflict compensation branches have implementation but lack separate D12 branch-specific acceptance. No new cross-process contention or scale/performance proof. Schedule materializer deadline versus slot deadline needs separate reconciliation. Shared-DB FK/trigger reads do not prove physical owner database split. Host/cluster reservations/provider/readiness, native Hook execution, Agent sandbox, independent validation, Task CLI execution closure and BI/Benchmark remain open. No whole-workspace/release/doc/bench claim. Existing star-pg-adapter strict Clippy failures are reported separately; no sweeping refactor. CodeRabbit current probe: `/root/.local/bin/coderabbit: Permission denied`; no independent review pass. RTK and matching graph tools unavailable; bounded source fallback.

## §4 Subagent failures and takeover

No subagent dispatched. Root owns the implementation lane; user/root worktrees and dev CodeGraph files are preserved.

## §5 Guard rules

1. Bounded typed request/result/resolution.
2. Tenant RLS and injected source scope.
3. Canonical occurrence and pinned Rule snapshot.
4. Original owner/generation fence.
5. Lease and occurrence deadline before/after writes.
6. Logical attempt monotonicity and retry bound.
7. No overlap with unreceived or confirmed requests.
8. Exact immutable replay and conflict rejection.
9. No stale worker dispatch rewrite.
10. Cancellation/expiry/link decision before link creation.
11. Paired accepted receipt/resolution.
12. Atomic link/event/Inbox/Outbox rollback.
13. Bounded retry and exact terminal retention timestamp.
14. Owner-local mutation capabilities only.
15. Separate Run/Infrastructure revalidation; no executor/VM authority.

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: owner-local message transactions and existing lease model. |
| SRE Lead | Mavis review: bounded locks, deadlines, immutable replay and failure evidence. |
| Platform | Mavis review: restricted capability, FORCE RLS and no cross-owner writes. |
| Review chair | Mavis review: actual handshake evidence and explicit fixture limits. |
| PM | Mavis review: local persistence slice only; full product phases remain active. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis接手审核 | Schedule owner handoff and isolated two-owner integration. | Continue after D11 dev integration. |
