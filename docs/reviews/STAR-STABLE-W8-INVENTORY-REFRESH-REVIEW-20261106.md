# Review — Star Stable W8 Multipass Inventory Refresh

**Review date:** 2026-11-06
**Reviewed target:** `dev@a01c66ab93d9884891d0ea885bf74117a5d22d56`
**Reviewed report:** [W8 inventory refresh](../reports/PHASE-STAR-STABLE-W8-INVENTORY-REFRESH-20261106.md)
**Disposition:** ACCEPT WITH LIMITATIONS for the host snapshot only

## Scope

Checked the report against its registered read-only scope, the earlier [W8 capacity review](STAR-STABLE-W8-MULTIPASS-CAPACITY-RECHECK-20261101.md), and the W8 release gates. No VM, branch, or WBS file was changed.

## Findings

1. The recorded current client and daemon versions, `1.16.4+win`, match the observed command output.
2. The JSON inventory contains one visible stopped Ubuntu 26.04 LTS record. Omitting its identifier does not alter that count; because stopped records count, it consumes a slot. The CLI output does not prove the absence of separate retained/deleted ledger rows.
3. The report does not claim the guest has K3s or that it is a valid target for the W8 test. No guest command or cluster operation was run.
4. The earlier `{}` parser/fail-closed issue remains open because this refresh did not inspect or test that script behavior.
5. No production or release gate is closed by this snapshot. The two-record controlled-inventory gate remains unproven. W8 remains OPEN and the stable release remains NO-GO.

## Verification boundary

This is a review of the new read-only host evidence and its wording. It is not a second Multipass runtime session, a guest inspection, a K3s test, an image build, or a performance result. No automated tests were requested or run.

## Recommendation

Accept the report as a dated inventory refresh. Keep W8 open until the supported Multipass version is installed and verified, the controlled inventory gate is exercised, and the two-instance K3s/runtime requirements receive their own evidence.
