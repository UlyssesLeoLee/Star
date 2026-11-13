# PHASE — Star Stable W8 Multipass Inventory Refresh

**Date:** 2026-11-06
**Target:** local `dev@a01c66ab93d9884891d0ea885bf74117a5d22d56`
**Scope:** read-only host version and instance inventory snapshot
**Disposition:** evidence refresh only; W8 remains OPEN and release remains NO-GO

## §0 Purpose

Refresh the local Multipass host snapshot used by stable-release W8. The lane performs no VM lifecycle operation and does not claim a K3s or Star infrastructure runtime acceptance result.

## §1 Change and evidence matrix

| Item | Evidence | Result / boundary |
|---|---|---|
| Multipass client and daemon | `multipass version` | Both report `1.16.4+win`; this remains below the W8 report's stated v1.17.0 minimum. |
| Full local inventory | `multipass list --format json` | One current CLI row: state `Stopped`, image `Ubuntu 26.04 LTS`. The instance identifier is intentionally omitted. This CLI view does not establish whether a separate controlled ledger retains deleted records. |
| W8 cap | Current inventory count | One of the two permitted records is visible; the stopped row consumes a slot. The controlled total, including any retained/deleted records, is not proven by this CLI snapshot. |
| Prior evidence | [2026-11-01 capacity review](../reviews/STAR-STABLE-W8-MULTIPASS-CAPACITY-RECHECK-20261101.md) | It reported the same client version and one stopped Ubuntu 26.04 record; it also recorded an unknown-empty-inventory parser gap. This refresh did not re-audit the script or test that gap. |
| WBS status | Stable-release WBS W8 | No WBS edit is part of this manifest. W8 remains OPEN. |

## §2 Verification summary

- Through RTK, ran `multipass version` and `multipass list --format json`; both exited successfully.
- Reviewed the prior W8 capacity review and its stated limitations.
- Did not run any VM lifecycle command, guest command, K3s command, container build, database action, or product test.
- The official Multipass list reference documents that `multipass list` lists created instances and accepts JSON output: [Multipass list command reference](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/list.md).

## §3 Known gaps

1. The local client is still 1.16.4; this snapshot does not upgrade or configure it.
2. The sole listed guest is stopped. Its owner, K3s membership, cluster identity/generation, node readiness, workloads, and reservation state are not established here.
3. No guest build, final image, SBOM/provenance, cluster rollout, upgrade, recovery, or rollback was run.
4. The earlier review's `{}` unknown-empty-inventory fail-open finding remains untested and unresolved in this lane.
5. The CLI snapshot does not independently reconcile the controlled inventory ledger or any retained/deleted records. It does not prove that a future two-instance test is safe or ready.

## §4 Sub-agent failure and handoff

No subagents were dispatched from this lane and no handoff failed. The parent integrator should update the relevant WBS only after its own manifest and ownership gate permits that file.

## §5 Integration and operating gates

1. Count every inventory record, including stopped records, toward the two-record test limit.
2. If inventory cannot be parsed completely, fail closed; do not launch or purge automatically.
3. Use the repository's controlled, lock-protected Multipass script for any future creation; this lane does not invoke it.
4. Keep Multipass instance, HostEnvironment, K3s cluster, and Kubernetes node identities distinct.
5. Require a pinned infrastructure profile, exact cluster identity/generation, host and cluster reservations, and independent runtime readiness before accepting an execution task.
6. Do not treat a stopped guest, source-level script check, or image build as proof of the other gates.
7. Preserve existing guest contents and user worktrees; this lane performs no cleanup.

## §6 Review sign-off

| Role | Result |
|---|---|
| Architecture | Refresh is limited to observed host facts; W8 remains OPEN. |
| SRE Lead | No guest or cluster behavior was exercised; runtime gate remains pending. |
| Platform | Multipass version and inventory were observed without mutation; provisioning remains pending. |
| Review | Accept with limitations as a host inventory snapshot only. |
| PM | No release decision changed; v1.0.6 remains NO-GO. |

## §7 Revision history

| Version | Date | Revision | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Record the fresh Multipass version and one stopped Ubuntu 26.04 inventory record; keep W8 and release gates open. | Current W8 inventory refresh |
