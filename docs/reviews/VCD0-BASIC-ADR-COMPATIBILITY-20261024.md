# VCD-0 Basic Design / ADR Compatibility Review

**Review date:** 2026-10-24
**Scope:** Read-only review of the Vercel basic-design draft and parallel-deployment ADR against current dev SRS v0.3, WBS v4.17, and `AGENTS.md`. No implementation or deployment claim is made.

## 1. Evidence and source status

| Input | Revision/state observed | Authority for this review |
|---|---|---|
| Current dev SRS | `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md`, FR-VC-001..021, NFR-VC-01..08, AC-VC-001..013, GAP-01..11 | Authoritative requirement baseline |
| Current dev WBS | `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` v4.17, VCD-0..VCD-4 | Authoritative phase/dependency/acceptance baseline |
| `D:\Star\docs\basic-design\BD-STAR-VERCEL-DEPLOY-001.md` | Untracked in `D:\Star` main worktree at `94fdc837`; header says upstream SRS v2.0 and cites source commit `5a051bfd…` | Unapproved draft only; the cited source revision was not established as current dev |
| `D:\Star\docs\adr\ADR-STAR-VERCEL-PARALLEL-001.md` | Untracked in `D:\Star` main worktree at `94fdc837`; header says SRS/BD v2.0 and cites the same source commit | Unapproved proposal only |
| `D:\Star\docs\reviews\REVIEW-STAR-VERCEL-ADVERSARIAL-002.md` | Untracked review evidence in the same main worktree | Findings are leads, not approved requirements; platform claims were checked against current official pages below |

The review checkout was at `03db1d13380cb0bcab609135e50af604687827cc`. The main worktree contains unrelated dirty and untracked content; this review did not modify it. The drafts themselves state that they are unapproved and unimplemented (BD lines 8, 792-795; ADR lines 3, 40).

## 2. Content compatible with current dev

- A protected frontend Preview/production path, same-origin BFF, no-mock behavior, explicit capability gating, and separation of deployment success from business success are compatible with SRS §3.1-3.3 and FR-VC-005..008/016/021. See BD §0, lines 18, 22-24; §3.6 and §5.1-5.3.
- Keeping PTY, local Worktree access, Agent execution, native Hooks, infrastructure management, and long-lived workers on Star's authoritative runtime is compatible with SRS §3.1-3.2 and FR-VC-013/015/016; it also preserves the Agent/Task CLI fail-closed rule in `AGENTS.md` lines 34-37. See BD lines 96-102, 365-385, 561-571.
- Treating Cron as a best-effort wake-up that asks the durable Star scheduler/owner to reconcile work, with current `run_as_actor_id` authorization, idempotency, and no function-local worker, is compatible with SRS FR-VC-014/015 and WBS VCD-4. See BD §3.12, lines 412-430.
- Protected Preview isolation and promotion controls align with FR-VC-021 and AC-VC-013. The draft correctly keeps these as gates rather than reporting them as implemented.
- General Functions WebSocket availability is now GA, as the official changelog states, while Rust-runtime-specific streaming/WebSocket support remains unproven. The distinction in BD §10 line 782 is sound; current dev SRS/WBS wording needs a platform-fact correction described below.

## 3. Conflicts with the current design baseline

| Finding | Draft evidence | Current baseline | Required correction |
|---|---|---|---|
| **The independent second execution/owner plane is made the default.** BD-D02 and §0.1 define B-Independent with its own Rust Host/Gateway/Owner/DB/scheduler; its diagram shows a separate B database and owner (BD lines 19, 29-44), and the component table assigns a B Owner and scheduler (lines 96-101). ADR likewise defines an independent Owner, DB, and execution domain as B-Independent and makes it one P1B option (ADR lines 8-12, 29, 36, 40). | SRS §3.1 lines 113-116 makes Vercel only a presentation plane and a *conditional stateless adapter* candidate; Star Owner retains authorization, writes, durable facts, and Outbox. FR-VC-009..015 constrain database, migration, schedule, and worker ownership. WBS VCD-0 line 56 requires those planes remain separated; VCD-3 line 59 explicitly reuses Star authorization, owner transactions and durable Outbox/Inbox. | Remove B-Independent as a Vercel deployment option from this design set. Keep Vercel as optional frontend Preview/production; only consider thin Rust HTTP adapters after VCD-2 and WLD gates, with Star owners and their durable state authoritative. A second Owner/DB/worker/scheduler requires a separately approved architecture decision and WBS, not this draft. |
| **The draft opens a second host/VM execution provider.** BD §4.8 permits a B-owned Rust Host and calls for a possible VM-only provider (lines 593-597); ADR asks whether a Non-K3s provider is approved (line 40). | `AGENTS.md` lines 28-30 mandates Multipass-managed K3s for local/CI guests, no Docker Desktop prerequisite, at most two retained Multipass inventory records in the test phase, and fail-closed capacity/binding. SRS §3.2 lines 122-128 and FR-VC-016 exclude Multipass/K3s management, local Worktree filesystem and execution from Vercel. WBS VCD-3 line 59 keeps these in Star's authoritative runtime. | Remove the B Host/VM execution proposal from VCD-0. Vercel functions must not manage Multipass, K3s, Worktrees, or Agent sandboxes. Do not loosen the two-record cap or introduce an alternate provider through Vercel. |
| **New requirement IDs and acceptance scope are not in the authoritative SRS.** BD §2 claims FR-VC-001..040 (lines 120-168); §8.1 adds AC-VC-023..034 (lines 745-752), and §10 claims NFR-VC-01..12/GAP-01..17 (lines 776-777). Several mappings also refer to AC-014/015/018..022. | Current dev SRS §0 line 20 and its actual tables define FR-VC-001..021, NFR-VC-01..08, AC-VC-001..013 and GAP-01..11. WBS VCD-0 line 56 requires traceability from the existing FR/NFR/GAP/AC set. | Do not import these IDs or assert their traceability. Map the useful checks to existing IDs (especially AC-VC-003/005/006/013, FR-VC-014/015/016/021 and relevant GAPs); if scope expansion is later approved, revise the SRS first and update WBS references together. |
| **Cron rollback is described as an unresolved official-document contradiction.** BD §5.5 line 678 and ADR §5 line 49 say the official pages conflict; the untracked adversarial review repeats this at §1 AR-13 and §3 lines 59-60. | Current dev SRS §2.2 line 101, FR-VC-014 line 177 and WBS VCD-4 line 60 already require reconciliation when rollback changes Cron definitions. As reviewed on 2026-10-24, the official [Cron management page](https://vercel.com/docs/cron-jobs/manage-cron-jobs#rollbacks-with-cron-jobs) says rollback restores the target deployment's Cron definitions, and the [Instant Rollback page](https://vercel.com/docs/instant-rollback) says the same. | Remove the “official docs conflict” assertion. Retain the stronger operational requirement: inspect the active configuration after rollback and reconcile it with durable Star Schedule state. Cron remains best-effort and never becomes Schedule truth. |
| **Current dev's general WebSocket platform status is stale.** SRS §2.2 line 100 and GAP-02 line 247 call Functions WebSocket Public Beta; WBS VCD-0 and VCD-2 lines 56, 58 repeat that claim. | The official [WebSocket changelog](https://vercel.com/changelog/websocket-support-is-now-generally-available) says Functions WebSocket support became generally available on 2026-10-20. The same announcement is referenced by BD §10 line 782 and the untracked review §3 line 58. | Correct the SRS/WBS general Functions status to GA. Keep Rust-specific compatibility, duration, memory, concurrency, and streaming support unknown until the Rust feature matrix or target-account test proves them (SRS GAP-02/GAP-11; WBS VCD-2). General GA does not establish Rust-runtime support. |

## 4. Unsupported or unverified draft assertions

- The source headers cite `5a051bfd…`, but the read-only main worktree is now at `94fdc837…` and both named drafts are untracked there. The cited source snapshot and claimed SRS/BD v2.0 lineage do not establish compatibility with current dev SRS v0.3/WBS v4.17.
- A/B release, database, credential, network and failure-domain isolation are presented as target architecture, not proven account/IAM/network behavior (BD §0.1 and §5.5; ADR §§0-2). VCD-1/VCD-4 acceptance still requires actual protected Preview and account-level evidence.
- The B-independent Rust Host, Owner, database and scheduler are proposed facilities, not current Star capabilities. The draft itself says no Vercel-specific feature is implemented (BD line 8), and its test table marks tests NOT_RUN (lines 719-752). No review statement should upgrade them to implementation or production readiness.
- The untracked adversarial report's Cron-doc-conflict assertion is contradicted by the currently opened official pages. The report's general WebSocket-GA assertion is supported, but says nothing about Rust Runtime compatibility.

## 5. Minimal remediation and disposition

1. Rewrite BD/ADR around the current VCD boundary: Vercel serves protected Preview/production web and its constrained BFF; Star remains the sole business Owner/execution authority. Keep Rust functions as isolated VCD-2 feasibility only; VCD-3 adapters stay thin and owner-backed.
2. Delete the independent Owner/DB/scheduler/worker/host topology and the Non-K3s VM-provider path from these documents. Preserve `AGENTS.md` Multipass/K3s identity, two-inventory-record test cap, and fail-closed execution gates.
3. Align every FR/NFR/GAP/AC reference with current SRS IDs. Put any genuinely new requirement through SRS revision and WBS traceability before it appears in a design or test baseline.
4. Correct the general WebSocket GA status in dev SRS/WBS while keeping Rust capability unknown. Replace the Cron “documentation conflict” statement with active-configuration observation/reconciliation after rollback.
5. Retain the drafts' compatible frontend/BFF, Preview isolation, no-mock, identity, Owner-backed Cron wake-up and negative-test ideas only after the above changes; leave deployment and test statuses open until evidenced.

**Disposition:** The draft is **selectively adaptable, not compatible as-is**. Its frontend/BFF, environment isolation, capability gating, and Cron reconciliation sections can seed VCD-0. Its core dual-plane B-Independent decision, added IDs, and alternate host/provider model must not enter the current dev design baseline. VCD-0 remains open until the compatible revisions are made in the canonical SRS/BD/DD/test set and the WBS is synchronized.

## 6. Validation

- Documentation/link and ID cross-check: source paths and cited current SRS/WBS/AGENTS identifiers were inspected; deployment/test evidence remains unrun by design.
- `git diff --check` via RTK: **passed**.
- Code tests, deployment tests, and live Vercel account tests: **not run**, as required for this documentation-only review.
