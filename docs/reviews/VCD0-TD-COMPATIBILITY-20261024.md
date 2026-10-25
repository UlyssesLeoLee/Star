# VCD-0 Test Design Compatibility Review — R3, 2026-10-25

## 1. Scope and disposition

Independent document review of the current dev baseline in this worktree: Vercel SRS v0.4, ADR v0.2, Basic Design v0.3, Detailed Design v0.3, Test Design v0.4, and the VCD-0 WBS entries. Review was limited to the Gateway/browser connection contract and T-049 coverage. No tests, deployment, or runtime checks were run; all test/runtime evidence remains **NOT_RUN**.

**Disposition: VCD-0 remains OPEN.** TD v0.4 has materially improved T-049 and correctly refuses to invent gateway wire details. It is a draft test plan, not an approved protocol contract or proof of behavior. Remaining issues below must be resolved before VCD-0 can close.

## 2. Historical R2 findings versus current evidence

- TD v0.4 now gives noVNC its own authorized Gateway row and includes positive fixture flow, Origin/identity rejection, revocation, outage/recovery, and trace evidence (§4.7, lines 167–177). The earlier document-level finding that T-049 omitted noVNC is resolved in the test-design text. This is not runtime evidence.
- T-049 now has a positive fixture-data assertion and no longer claims duplicate-side-effect proof for a fixture-only test (§4.7, lines 163–177). Those R2 test-text findings are resolved at document level.
- The real Star Gateway credential handshake and Origin allow/deny contract are still absent. TD correctly marks them as execution prerequisites; they remain open architecture inputs, not assumptions to fill in from this review.
- The earlier removal of remote file-browser coverage from T-049 is not a complete scope decision: the current SRS repository inventory still lists a remote file-browser WebSocket, while the current Vercel requirements and TD do not classify its Vercel behavior. See R3-01.

R2 reports based on earlier SRS revisions are historical evidence only. The findings below use the current paths and versions named in §1.

## 3. Prioritized findings

### R3-01 — P1: Reconcile the current browser-channel inventory, including remote file-browser

**Evidence.** SRS §0, line 35, inventories four existing browser long connections as terminal WS, remote file-browser WS, ARG WS, and chat SSE. SRS §4.4 / FR-VC-013, line 176, instead places terminal/PTy, noVNC, Canvas WSS, and ARG WS on the execution plane, with chat SSE as a separate permitted route; its acceptance text says “4 类长连接.” Basic Design §5, lines 137–138, likewise names PTY/noVNC, Canvas, ARG, and chat SSE. TD v0.4 §4.7, lines 163 and 167–175, tests four Gateway channels plus chat SSE, but explicitly excludes remote file-browser.

The “four Gateway channels plus SSE” count is internally understandable: chat SSE is separate from the four Gateway channels. The unresolved mismatch is the product inventory: remote file-browser appears in the SRS’s current-repository inventory but has no Vercel plane/availability decision or test, while noVNC and Canvas appear in the Vercel requirements but not in that inventory. Remote file-browser is also distinct from the local Worktree filesystem that TD assigns to T-018.

**WBS impact.** Keep the VCD-0 endpoint-inventory gate open in `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` §VCD-0 (line 56) and the summary in `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` (line 325). Refine it to reconcile the browser inventory and explicitly decide remote file-browser handling; do not treat noVNC’s new TD row alone as closure.

**Acceptance condition.** Establish one canonical inventory that maps terminal/PTy, noVNC, ARG, Canvas real-time, chat SSE, and remote file-browser to exactly one Vercel behavior each: approved Star Gateway route, approved same-origin route where applicable, or explicitly unavailable/fail-closed. Trace every enabled feature to its SRS requirement, design plane, and positive plus negative test. If remote file-browser is unavailable on Vercel, test that it is disabled without attempting a local-filesystem fallback.

### R3-02 — P1: Add an authenticated out-of-scope principal/data-isolation assertion

**Evidence.** TD §4.7, lines 167 and 171–175, checks valid fixture delivery, missing/invalid identity, forged actor/Run headers, Origin allow/deny, and revocation. It does not explicitly connect a second otherwise-authenticated principal lacking the fixture Run/capability and assert that this principal receives zero fixture events/frames. Rejecting spoofed identity headers is not equivalent to proving that a valid but out-of-scope session cannot read another Run’s stream.

**WBS impact.** Add this as a required VCD-0 T-049 acceptance case before the VCD-1 hosted Preview smoke gate. Keep VCD-0 OPEN until its outcome is represented in the trace matrix.

**Acceptance condition.** In an isolated fixture, prove both directions: a principal authorized for Run A receives only Run A’s expected fixture data; a valid principal without Run A access/capability receives no Run A data and cannot subscribe or reconnect into that stream. Include the negative assertion in captured evidence for each applicable Gateway channel and the selected chat SSE route, without using production Owner data or creating business side effects.

### R3-03 — P1: Gateway credential, Origin, authorization-freshness and revocation contracts remain execution blockers

**Evidence.** SRS FR-VC-013 (§4.4, line 176) establishes the execution-plane boundary but does not define a browser credential exchange. ADR §6, line 57, keeps chat SSE routing and real session/security contracts unselected. Basic Design §5, lines 137–138, and Detailed Design §2.3, lines 64–66, explicitly defer real Gateway handshake, Preview Origin allow/deny, current Project/Run/capability authorization, revocation, protocol data and lifecycle details to an approved contract. DD §7, lines 168–170, requires fresh authorization on initial connection/reconnection and protection of subsequent data/actions. TD §4.7, lines 163 and 177, correctly makes the missing contract a hard precondition and keeps T-049 NOT_RUN.

This is currently fail-closed by design. It is not evidence that the handshake or Origin protection works. The design does not yet provide a testable source for the credential issuer/audience and browser exchange, the exact allowed and denied Preview Origin rules, or the maximum revocation-propagation behavior.

**WBS impact.** Preserve the VCD-0 contract gate in the parent WBS §VCD-0 and VCD-1 prerequisite. Do not promote a review of the conditional test text into a Gateway security approval.

**Acceptance condition.** A canonical Star Gateway/session contract must define, per enabled channel, the trusted credential source and safe browser exchange, Origin allow/deny rules (including missing or untrusted Origin behavior), Project/Run/capability checks, credential expiry, and revocation behavior with an observable, measurable upper bound. T-049 must then exercise one allowed and one rejected Origin, current authorization, and a post-revocation protected read/action or reconnect. No reusable credential may appear in URL, browser storage, logs, traces, or evidence.

### R3-04 — P1: Fix a chat SSE route and define route-specific stream recovery before T-049 is executable

**Evidence.** SRS FR-VC-013 allows chat SSE through a Next route (§4.4, line 176); ADR §6, line 57, leaves its final route unselected; Basic Design §5, line 138, and Detailed Design §2.3, lines 66 and 170, require a canonical route decision; TD §4.7, lines 163 and 167, correctly tests only the ultimately approved route. T-049 covers a Gateway outage followed by a fresh-session reconnection, but it does not define route-specific streaming behavior or assert retry policy, cancellation on client disconnect, reauthorization after 401/403, or event gap/replay/duplicate behavior for SSE.

**WBS impact.** VCD-0 must choose and synchronize the route across SRS/ADR/BD/DD/TD. VCD-1 must retain the real route’s hosted Preview evidence; do not enable the UI while the route is unresolved.

**Acceptance condition.** Select either the already-permitted Next Route Handler passthrough or another explicitly supported Star route, then document its authentication and Origin enforcement, streaming/backpressure behavior, heartbeat and termination semantics, client-disconnect cancellation, reconnect rules, event resume/gap/duplicate behavior, and handling of authorization failures. T-049 must test a transient interruption and recovery under that contract, verify no stale credential is retried after authorization failure, and verify the defined event continuity semantics. Do not invent timing or retry-count thresholds without a measured/approved contract.

### R3-05 — P2: Refresh the SRS downstream-document status line

**Evidence.** SRS header line 11 says the detailed and test designs have not yet been created in the current dev document tree, but the current baseline includes `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md` v0.3 and `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md` v0.4. This stale header weakens baseline traceability even though the documents themselves cite the current SRS v0.4.

**WBS impact.** Add a small VCD-0 documentation-sync item; it is not runtime evidence and does not change the OPEN gate.

**Acceptance condition.** Update the SRS header to name the current downstream documents and their draft/approval state, then verify the SRS/ADR/BD/DD/TD revision references agree.

## 4. Testability and status

T-049 is substantially more testable than the historical R2 version: it now requires positive per-channel fixture flow, explicit negative Origin/CSRF coverage, identity rejection, revocation, outage and recovery, and redacted evidence. Its fail-closed rule is appropriate while the Gateway contract is missing. It still cannot be executed until R3-01 through R3-04 are closed. Do not infer that adding a row for noVNC proves noVNC protocol compatibility or runtime support.

| Item | Current evidence status |
|---|---|
| SRS / ADR / BD / DD / TD document review | Reviewed at the versions in §1 |
| T-049 and other tests | NOT_RUN; explicitly not executed per assignment |
| Gateway handshake / Origin / revocation / route behavior | UNKNOWN / no approved contract cited |
| Vercel Preview, deployment, runtime, and performance | NOT_RUN |
| VCD-0 | OPEN |

## 5. Review history

| Revision | Date | Change |
|---|---|---|
| R2 | 2026-10-24 | Historical compatibility review against the prior SRS baseline; superseded for current findings. |
| R3 | 2026-10-25 | Re-reviewed current SRS v0.4, ADR/BD/DD v0.2/v0.3, TD v0.4, and WBS; recorded remaining inventory, isolation-test, Gateway-contract, SSE-route/recovery, and SRS-header gaps. No tests or runtime checks run. |
