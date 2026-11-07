# W8 Source Position Integration Audit

**Review date:** 2026-11-06
**Target:** local `dev@323c5ac83186acaba8dbf38c21566a532795074d`
**Candidate:** `UlyssesLeoLee/w8-source-position-claim-fix-20261025@5fe3ea9372242d7d72f9dd416f6437475c870904`
**Disposition:** **HOLD — do not integrate the four-path manifest or cherry-pick the tip commit.**

## §0 Purpose

Determine whether the currently registered four-path W8 source-position lane forms a coherent, reviewable change against current `dev`, and record the bounded Rust/SQL findings before any integration decision. This audit is read-only against the candidate. It does not authorize expanding the manifest, merging candidate commits, modifying either conflicting W8 worktree, changing `main`, pushing, or cleaning branches.

## §1 Change and dependency matrix

| Area | Evidence | Decision |
|---|---|---|
| Source position SELECT fix | Candidate Inbox calls `bi.commit_task_event_v3` and reads `source_sequence` (`crates/bi-owner-runtime/src/task_event_inbox.rs:495–501, 547–554`). Current `dev` calls v2 and decodes only the older receipt (`:351–356, 385–387`). | **Not standalone.** The latest fix depends on the candidate's source-origin contract. |
| Required migration chain | `2026-10-24-bi-task-event-source-origin.sql` defines `bi.task_event_source_origin`, `quarantine.source_cursor`, and v3; the shared-position migration reads those facts (`:33–78, 124–150, 207–237, 244–295`; shared migration `:137–162`). Neither migration is in current `dev`. | **P1 scope gap.** The registered manifest omits a prerequisite migration. |
| Receiver and quarantine support | Candidate source-origin data flows through Inbox, quarantine, and receiver; receiver commits Inbox/quarantine and rechecks authorization before acknowledging (`task_event_receiver.rs:384–470`). | **Required dependency.** A source-only Inbox patch would not provide a closed path. |
| MAGOS manifest | Current registration has four entries: shared-position migration, arbitration PG test, phase report, and `task_event_inbox.rs`. The manifest records 13 additional candidate paths as drift, including the source-origin migration, receiver/quarantine support, workspace/adapter files, and test support. | **Does not describe the candidate's dependency closure.** The override `94d8b953e690` does not authorize accepting drift or changing the other W8 sources. |
| Transaction and tenant controls | Migrations are transaction-scoped, use tenant RLS, restrict direct runtime table access, and serialize claim arbitration under `READ COMMITTED` (`2026-10-24` migration `:33–78, 124–162, 281–295`; shared migration `:46–97, 99–162, 164–320`). | **Structurally coherent, behavior unverified.** Static review is not a database result. |
| PG audit identity | The documented audit URL is expected to log in as `star_bi_owner`, while its role is `NOLOGIN` (`2026-10-16-bi-task-event-inbox.sql:31–45`). The ignored test connects directly and asserts that current user (`source_position_arbitration_pg.rs:214–217, 882–912`) without a role switch. | **P2 test setup mismatch.** Define an explicit login-capable test identity and role switch, or correct the documented provisioning. |
| Early tenant rejection | `commit_task_event_v3` acquires source advisory locks before checking the supplied tenant against the active tenant (`2026-10-24` migration `:128–162`). V2 performs the tenant check and forced RLS remains tenant-bound. | **P2 hardening.** No cross-tenant read/write path was established by this static review; reject mismatched tenant IDs before taking tenant-derived locks. |
| Native transport adapter | The candidate adds a native adapter and its own transport/migration/test dependencies. Its `receive_one` path is caller-driven; workspace inclusion alone does not establish a production worker call site. | **Separate integration question.** Do not pull adapter-only files into the source-position slice without a distinct acceptance scope. |

## §2 Verification summary

- Two independent read-only reviews examined the Rust wiring and SQL/migration boundaries. Both recommended HOLD; neither changed candidate files.
- Candidate history is eight commits ahead of current `dev`; its aggregate branch delta is 16 paths (`+4,484/-183`). All eight candidate-only commits are unique relative to `dev` in `git cherry`.
- The candidate's phase report records formatting, focused Rust compile/link, and no-run checks after the SELECT edit. The ignored PostgreSQL test was not rerun after that edit because the dedicated database endpoints were unavailable.
- The report records an earlier PostgreSQL run failing on missing `source_sequence`; that failure predates the SELECT fix and is not evidence that the fixed path works.
- This audit ran no tests, migration, database, Rust build, hosted workflow, or deployment. The current database behavior, RLS, grants, concurrency, replay, durable receipt, and commit-before-ACK remain unverified.
- The candidate worktree's pre-existing untracked `docs/briefs/WBS-W8-DECODE-FIX-20261026.md` remains untouched. The two conflicting W8 worktrees remain untouched.

## §3 Known gaps and required next work

1. Reconcile the MAGOS manifest against a coherent source-position dependency closure before any merge. At minimum, review the missing source-origin migration, shared migration, Inbox/quarantine/receiver changes, PG setup/support, arbitration test, and report together.
2. The four-path override is insufficient evidence to accept the candidate's recorded drift. Any broader scope requires a new explicit user-authorized MAGOS plan and a fresh conflict review; preserve both conflicting W8 lanes.
3. Resolve the `star_bi_owner NOLOGIN` test-connection mismatch before relying on the two-connection PG test setup.
4. Add early tenant validation before advisory-lock acquisition as defense in depth, then re-review transaction and RLS behavior.
5. Run the ignored regression test after the SELECT change against the dedicated disposable PostgreSQL endpoints, including the documented login/role setup. Record actual migrations, grants, RLS, collision/concurrency, replay, and ACK observations.
6. Keep BI event ingestion and any dependent execution capability fail-closed until the database behavior and durable ACK chain have evidence.

## §4 Sub-agent handoff status

The Rust wiring and SQL/migration audits completed independently and returned consistent HOLD recommendations. No handoff failed. There was no implementation delegation in this review; no source lane was edited or merged.

## §5 Integration gates

1. Keep `main`, remote refs, and unrelated user worktrees unchanged.
2. Do not merge `5fe3ea93` by itself.
3. Do not infer approval from the prior four-path manifest override.
4. Do not accept unregistered drift into an integration commit.
5. Include the source-origin migration before any v3 Inbox caller.
6. Include and review the Inbox, quarantine, and receiver contract as one dependency chain.
7. Keep native transport adapter adoption outside this slice unless its call path and acceptance are separately reviewed.
8. Make the PG audit login role and any `SET ROLE` operation explicit and reproducible.
9. Reject a tenant mismatch before tenant-derived lock acquisition.
10. Preserve transaction-local tenant context, owner-only table access, and forced tenant RLS.
11. Acknowledge only after durable owner-transaction commit.
12. Run the ignored PostgreSQL regression after the final SQL/Rust changes.
13. Treat `fmt`, compile, and `--no-run` as build evidence, not database behavior evidence.
14. Keep concurrency, replay, recovery, grants, and commit-before-ACK open until observed in the target disposable database.
15. Keep BI ingestion and dependent execution disabled when any identity, migration, database, or durable receipt fact is unknown.

## §6 Review sign-off

| Role | Result |
|---|---|
| Architecture | Mavis 接手审核 — HOLD; dependency closure is incomplete in the registered manifest. |
| SRE Lead | Pending dedicated PostgreSQL and role-provisioning evidence. |
| Platform | Pending fresh MAGOS manifest and source-owner review. |
| Review主持 | Two independent static audits agree; no integration approval issued. |
| PM | User-directed target remains local `dev`; this report does not broaden prior authorization. |

## §7 Revision history

| Version | Date | 修订人 | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the exact four-path manifest gap, dependent source-origin chain, PG login mismatch, and unverified database behavior; recommend HOLD without changing candidate scope. | Independent W8 Rust and SQL audits against `dev@323c5ac8`. |
