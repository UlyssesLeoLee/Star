# VCD-0 Candidate Revalidation Against Current Dev

**Review date:** 2026-11-07
**Target:** local `dev@6634a455bc475a66128390d36d956c47b659fe9a`
**Candidate:** `codex/vcd0-canonical-detail-test-20261024@4f0ceae9818b7bd2b3b25d9164d6e43303121689`
**Disposition:** **ADAPT — do not merge the stale candidate commits.**

## §0 Purpose

Revalidate the VCD-0 candidate against current `dev` and record what remains open for the stable-release W6 lane. The review uses the current canonical requirements/design and the previous VCD-0 review as evidence; it does not change the candidate, launch tests, perform a Vercel or database operation, or authorize a release.

## §1 Candidate and current-design comparison

| Review area | Current evidence | Decision |
|---|---|---|
| Candidate freshness | `dev` is 392 commits ahead of the candidate and the candidate has 5 commits absent from `dev`. The existing 2026-11-06 review records that the candidate carries Vercel DD/TD documents and those candidate documents identify as v0.1; this lane did not reopen candidate source files. Current dev uses SRS v0.4, ADR v0.2, BD v0.3, DD v0.3, and TD v0.4. | **Not a drop-in.** Do not cherry-pick the candidate's five commits. Adapt only individually reviewed ideas to current canonical documents. |
| Project and resource ownership | Current decisions keep Star as the Project/Run authority, allow 0..N repositories, support Runs without a host Branch, keep provider association optional, and require migrate-before-unlink for an active repository. | The candidate does not fully trace these states into the Vercel authorization and acceptance paths. Add the existing SRS/BD/DD identifiers to the VCD-0 trace; do not create a second owner or make a host identity a prerequisite. |
| Remote interaction boundary | Current tests distinguish the terminal/PTY path, remote file-browser, noVNC, Canvas channels, and chat SSE. T-049 is a transport/session check and requires an authenticated principal without target Run capability to see no protected data. | The candidate's route and acceptance text is not aligned with the latest route/security decisions. Resolve Gateway versus route-handler placement and specify fresh authorization, Origin/session checks, revocation, cancel, disconnect, and bounded reconnect recovery before marking the path ready. |
| Multi-user collaboration | The current collaboration contract requires owner-committed push events, bounded wake-up delivery, durable cursor catch-up only after a gap/reconnect, both users' saved drafts, shared conflict identity, and participant-authorized counterpart diffs. | The candidate is not end-to-end collaboration evidence. Keep the existing WLD/W16 acceptance as the source and do not use a successful SSE handshake as proof of durable collaboration. |
| Neon and persistence | The user selected Neon for the Vercel profile. Runtime project/account/region, Star Owner placement, roles/grants, transaction-local tenant context, RLS, pool budget, and migration identity remain unverified. | Record Neon as the target choice and those items as unknown gates. Do not authorize direct Vercel business writes or DDL without an approved Owner/database path and runtime evidence. |
| Stable release | The candidate does not prove a tag-derived v1.0.6 build, hosted Vercel deployment, consumer verification, artifact provenance, or rollback. | W6 stays **ADAPT/OPEN** and v1.0.6 remains **NO-GO**. |

The broader historical comparison is retained in [WBS-VCD0-CURRENT-DEV-INTEGRATION-REVIEW-20261101.md](WBS-VCD0-CURRENT-DEV-INTEGRATION-REVIEW-20261101.md). That report's old target SHA remains a historical snapshot; this report records the current target SHA and confirms that no candidate commit is accepted.

## §2 Verification summary

- Read-only Git ancestry comparison on 2026-11-07 confirmed target `dev@6634a455bc475a66128390d36d956c47b659fe9a`, candidate `4f0ceae9818b7bd2b3b25d9164d6e43303121689`, and a 392/5 divergence using Git metadata only. Candidate source files were not reopened; the candidate DD/TD freshness statements remain attributed to the existing 2026-11-06 review.
- Current canonical Vercel document headers were read directly: SRS v0.4, ADR v0.2, BD v0.3, DD v0.3, TD v0.4. The candidate DD/TD v0.1 finding is inherited from the existing 2026-11-06 review and was not reverified by opening candidate files.
- The W6 row on this current dev base and the existing VCD-0 review were compared. The row keeps multi-repository/hostless Run, optional provider association, Neon/Owner/RLS, collaboration, hosted deployment, consumer, and rollback gates open; this revalidation adds a current-baseline review link.
- No Node/Rust tests, database migration, PostgreSQL/RLS test, Vercel deployment, Multipass/K3s operation, hosted workflow, artifact build, consumer check, or rollback was run in this review.

## §3 Known gaps and next work

1. Keep the candidate branch and worktree intact; do not merge its stale commits. Any implementation must start from a fresh `dev` base and receive a separate exact-file plan and review.
2. Update VCD-0 acceptance to trace the existing 0..N repository, hostless Run, optional provider association, and migrate-before-unlink decisions through the Owner API boundary.
3. Decide the production SSE route and document Origin/session validation, per-request authorization, revocation, cancellation, disconnect, event limits, and replay/cursor behavior.
4. Link push-collaboration acceptance to durable owner transaction, Outbox/Inbox, both-party drafts, common conflict identity, authorized counterpart diffs, and reconnect catch-up. Keep fixed-interval polling out of the normal path.
5. Identify the Neon runtime project and region, Star Owner API location, app and migration roles, grants, RLS transaction context, pool limits, and deployment/rollback identities. Unknown facts keep execution disabled.
6. Run target-environment security, collaboration, build, provenance, consumer, and rollback gates before closing W6 or the v1.0.6 release.

## §4 Sub-agent handoff status

The prior bounded VCD-0 review was read-only and completed. This revalidation was performed against current document headers and Git ancestry; no implementation handoff was issued and no review source lane was edited.

## §5 Review gates

1. Preserve `main`, remote refs, and user-owned worktrees.
2. Do not merge the stale five-commit candidate.
3. Treat the current dev SHA as the only baseline for follow-up work.
4. Keep SRS/ADR/BD/DD/TD versions and trace links synchronized.
5. Keep Project and Run authorization owned by Star.
6. Keep repository association optional and support hostless Runs.
7. Migrate or retire active Worktree bindings before repository unlink.
8. Keep provider-specific access separately authorized.
9. Require no protected data for principals lacking target Run capability.
10. Specify the deployed SSE route and enforce Origin/session checks.
11. Reauthorize before every protected operation and after resume/reconnect.
12. Treat disconnect recovery as bounded authorized cursor catch-up.
13. Require owner-committed durable events for collaboration claims.
14. Keep Vercel, Neon, build, consumer, provenance, and rollback claims evidence-based.
15. Keep W6 ADAPT/OPEN and v1.0.6 NO-GO until hosted and runtime gates pass.

## §6 Review sign-off

| Role | Result |
|---|---|
| Architecture | Mavis 接手审核 — ADAPT; current authority and collaboration contracts remain controlling. |
| SRE Lead | Pending Neon project/role and hosted deployment evidence. |
| Platform | Pending route, session, deployment, and runtime review. |
| Review主持 | Candidate is stale; no candidate commit accepted. |
| PM | Keep W6 open; v1.0.6 remains NO-GO. |

## §7 Revision history

| Version | Date | 修订人 | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Revalidate VCD-0 candidate freshness against `dev@d95fbfe3`; retain ADAPT, reject stale commits, and preserve hosted/runtime release gates. | Stable-release W6 continuation and current-dev comparison. |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Updated the review target to local dev@fb6fbb7 and refreshed Git ancestry to 384 commits ahead / 5 candidate-only. Kept the stale candidate rejected; candidate file claims remain attributed to v0.1 and were not re-read. W6 stays ADAPT/OPEN and v1.0.6 stays NO-GO. | Current-dev WBS revalidation on the exact assigned base; read-only Git ancestry check |
| v0.3 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Updated the target to current local dev@6634a455 and refreshed read-only ancestry to 392 commits ahead / 5 candidate-only. Kept all candidate-only commits rejected without opening candidate files; W6 remains ADAPT/OPEN and v1.0.6 remains NO-GO. | W8/W16 integration advanced current dev after the v0.2 snapshot; metadata-only revalidation |
