# VCD-0 Test Design Compatibility Review — 2026-10-24

## Scope and conclusion

Read-only review of D:\Star\docs\test-design\TD-STAR-VERCEL-DEPLOY-001.md against the assigned checkout’s current Vercel SRS, platform-runtime WBS, Run-lifecycle WBS, AGENTS.md, and current official platform documentation. The candidate adversarial review was treated as untrusted and checked only where relevant.

**Disposition: do not use this TD as the current approved acceptance baseline until P0 finding VCD-R1 is resolved.** It has useful fail-closed, idempotency, secret-scanning, and NOT_RUN controls, but it describes a larger A/B owner topology than the current dev baseline approves. All reviewed TD cases remain NOT_RUN. This document review did not run tests, deploy services, or establish implementation or runtime behavior.

## Findings

### VCD-R1 — P0: A/B independent/attached owner topology is not the current approved scope

**Evidence.** TD §0 and §1 define A-K3s, B-Independent with its own VM/PostgreSQL, B-Attached, and B-Web preview; §1 adds eight A/B acceptance cases pending approval; §3 expects a B Owner, B database, B WebSocket owner, B Schedule owner, and B Rust service; §5 then makes P1B a release phase with a real Gateway/Owner and writes. See target TD lines 3, 7, 11–22, 49–62, 68–77, and 133–140.

The current dev SRS is v0.3. Its approved sequence is Phase 1 frontend plus same-origin BFF, with a Rust HTTP control-plane only a Phase 2 candidate after VCD-2 and owner/database/identity review; Star retains execution and durable-fact authority. See SRS §0 and §§3.1–3.3, especially lines 11, 24, and 113–137. Current VCD WBS §2 likewise says the Vercel track is optional hosting and preserves Star’s owners and execution plane (VCD-0..4, lines 50–60). The TD labels PAR-01..08 as awaiting approval but uses B-owner behavior as if it were already approved.

**Required WBS follow-up — VCD-0, then VCD-1/VCD-2/VCD-3 as applicable.** Rebase the active test baseline on SRS v0.3: protected frontend/BFF for Phase 1; isolate Rust runtime feasibility under VCD-2; allow any Rust endpoint only under VCD-3 after VCD-2 PASS and WLD-0. Keep PAR-01..08 in a clearly marked, ADR-gated proposal until the architecture and SRS/basic/detail designs approve that topology. Do not make a second Owner, database, worker, scheduler, or VM execution plane a release prerequisite by implication.

### VCD-R2 — P1: Upstream design versions and acceptance IDs do not trace to current dev

**Evidence.** TD line 3 claims upstream SRS/BD/DD v2.0 and ADR-PAR-001. TD §4.2 creates AC-VC-023..034, while §6 line 144 calls AC-VC-001..034 “upstream.” In the required current dev checkout, the Vercel SRS is v0.3 and explicitly says the basic, detailed, and test designs are not yet in the dev document tree (SRS lines 3, 11, 20). It defines FR-VC-001..021, NFR-VC-01..08, and AC-VC-001..013 (lines 20, 222–238). VCD-0 is specifically the WBS item to create and reconcile that design set (WBS lines 54–56). This finding is against the requested dev baseline; it does not assert what may exist in D:\Star outside the reviewed target and candidate files.

**Required WBS follow-up — VCD-0.** Add a trace matrix for every current SRS FR/NFR/GAP/AC to the approved design section and test ID. Mark PAR and AC-VC-014..034 as proposed/derived until the upstream SRS and decision record approve them; do not describe them as upstream requirements. Keep all unexecuted cases NOT_RUN.

### VCD-R3 — P1: WebSocket cases conflate general Functions GA with unproven Rust-runtime support

**Evidence.** TD includes WS authentication and owner-ready expectations in UT-VC-010, IT-VC-003/005, ST-VC-004, AD-19..21, AC-029, and P1B. The current SRS assigns PTY/noVNC and Canvas WSS to Star’s execution plane and permits chat SSE through Next.js; see §§3.1–3.3 and FR-VC-013 (lines 113–137, 176). The TD cannot treat successful general Functions WebSocket behavior as evidence that its Rust-runtime path supports WebSockets.

Official status: general Vercel Functions WebSocket support became GA on 2026-10-20 ([official changelog](https://vercel.com/changelog/websocket-support-is-now-generally-available)). The Rust runtime page still labels Rust Beta; its feature matrix leaves entries blank and does not list WebSocket support ([Rust runtime documentation](https://vercel.com/docs/functions/runtimes/rust)). Rust-specific WebSocket/streaming compatibility is therefore **unresolved**, not disproven. The current dev SRS §2.2 and VCD-0 WBS row still call general Functions WebSocket Beta (SRS line 100; WBS line 56), so that source baseline also needs correction.

**Required WBS follow-up — VCD-0 and VCD-2.** Correct general Functions status to GA while preserving the Rust-runtime uncertainty. Keep PTY/noVNC/Canvas WSS acceptance on Star-owned runtime paths. If a Vercel Rust handler has a real WebSocket requirement, test it only as an isolated VCD-2 feasibility gate on the intended plan/runtime; leave that capability disabled until runtime-specific proof and a design decision exist. Do not infer Rust support from the general GA announcement.

### VCD-R4 — P1: Cron tests do not explicitly prove recovery from missed and failed invocations

**Evidence.** TD IT-VC-007 covers duplicate/late/DST/misfire, AD-24 covers 100 simultaneous ticks, and AD-25 covers rollback (lines 55, 106–107); these are useful partial controls. The listed cases do not explicitly assert that a request which never reaches the function is absent from runtime logs, that a failed invocation is not retried, that a later invocation scans and catches up durable due work, or that a prior long-running invocation overlaps a later interval. “Misfire” and simultaneous duplicate delivery do not independently prove those cases. Current SRS FR-VC-014 requires missed/duplicate/overlap/failure/rollback handling and durable reconciliation (line 177); WBS VCD-4 and WLD-3 require the operational and owner-side recovery gates (VCD WBS line 60; WLD WBS line 38). AGENTS.md also requires live authorization on every trigger/retry/resume (line 37).

Official Cron documentation says delivery is best effort, invocations may be missed or duplicated, a still-running invocation can overlap a later one, and failed invocations are not retried ([Managing Cron Jobs](https://vercel.com/docs/cron-jobs/manage-cron-jobs), especially “Cron job error handling,” “delivery and idempotency,” and “controlling cron job concurrency”).

**Required WBS follow-up — VCD-4 + WLD-3.** Add deterministic cases for a missed request, failed/timeout invocation with no automatic retry, overlapping invocations across intervals, durable catch-up on a later wake-up, one logical occurrence/one owner receipt, and run_as_actor_id revocation before retry/resume. For rollback, assert the exact Cron definitions restored from the target deployment and the project’s Disable Cron Jobs state, then reconcile owner Schedule state; checking only that some Cron is active is insufficient.

### VCD-R5 — P1: Preview isolation lacks an explicit no-side-effect acceptance case

**Evidence.** TD tests secret exposure (ST-VC-008) and a Preview credential attempt against production Owner (AD-03), but no case explicitly asserts that an untrusted Preview cannot create or mutate production Run/Task/Schedule facts, publish an Outbox event, or trigger production Cron. Current SRS FR-VC-021 and AC-VC-013 explicitly require no production Owner side effects and no Preview Schedule authority (lines 189, 238). VCD-1 requires the same gate (WBS line 57).

**Required WBS follow-up — VCD-1.** Add a negative hosted-Preview case that attempts the relevant owner and Schedule writes and then checks authoritative Run/Task/Schedule/Outbox state is unchanged. Exercise untrusted branch Preview, isolated credentials/data, denied production access, protected promotion identity/audit, and confirm no production Cron invocation.

### VCD-R6 — P1, unresolved: VM test-provider and two-record Multipass cap are not specified

**Evidence.** TD §0 and IT-VC-001 require a B-Independent “VM/PG” and Rust VM but name no VM provider or inventory guard (lines 7, 49, 58, 76). It is unclear whether this fixture is intended to consume the local Multipass test inventory. AGENTS.md §0 (line 28) forbids a Docker Desktop prerequisite and caps Multipass inventory at two records, counting stopped/deleted entries, with locked creation, fail-closed inventory checks, and no automatic purge. WBS VCD-2 expects an isolated no-production-data runtime proof (line 58); WLD-4 preserves the two-record limit for infrastructure tests (WLD WBS line 39).

**Required WBS follow-up — VCD-0/VCD-2; WLD-4 if the fixture uses Multipass/K3s.** State whether the VM is externally managed or Multipass-managed. If Multipass-managed, charge it against the two-record inventory and use the guarded locked launcher; fail closed for unknown/full inventory and do not purge. State explicitly that no Docker Desktop setup is required. This is an unresolved test-fixture detail, not a verified cap violation.

## Official evidence and candidate-claim check

- General Functions WebSocket GA date/status is confirmed by the [2026-10-20 official changelog](https://vercel.com/changelog/websocket-support-is-now-generally-available). Rust-runtime WebSocket compatibility remains unresolved in the [Rust runtime page](https://vercel.com/docs/functions/runtimes/rust).
- Cron miss/duplicate/best-effort delivery, no retry, and overlap risks are confirmed by [Managing Cron Jobs](https://vercel.com/docs/cron-jobs/manage-cron-jobs).
- The candidate review §3 claim that the current Instant Rollback and Cron pages conflict was not confirmed. Both [Managing Cron Jobs](https://vercel.com/docs/cron-jobs/manage-cron-jobs) and [Instant Rollback](https://vercel.com/docs/instant-rollback) currently state that rollback restores the Cron definitions associated with the target deployment and preserves the project-level Disable Cron Jobs setting. The TD’s active-state check is prudent, but the test should assert those exact semantics.
- No test, deployment, runtime limit, or production behavior is claimed as validated here.
