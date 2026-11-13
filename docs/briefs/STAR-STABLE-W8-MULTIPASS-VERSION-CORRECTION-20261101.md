# Brief: STAR-STABLE-W8-MULTIPASS-VERSION-CORRECTION-20261101

**Agent**: verifier
**Phase**: WBS-W8-MULTIPASS-VERSION-CORRECTION-20261101
**Created**: 2026-11-01 01:27:46

---

## Assignment

Correct the W8 provenance report's Multipass release chronology using current primary sources, then commit the corrected report and the dispatcher-created follow-up brief/log on this existing isolated lane.

## Finding to resolve

The current report incorrectly says 1.16.4 is the latest stable release and treats 1.17.0 as a prerelease. The official Canonical GitHub release page now lists 1.17.0 as Latest and records its release on 2026-10-19. Reconcile with these primary sources:
- https://github.com/canonical/multipass/releases
- https://canonical.com/multipass/docs/latest/reference/release-notes/

The existing WBS external-source section also records 1.17.0 GA. Do not repeat the stale '1.16.4 latest stable' claim.

## Required correction

1. Correct §1's upstream Multipass stable/maintenance row and all dependent report text/assessment. State that 1.17.0 is stable, HCS is the Windows driver direction, and the user's local host remains Multipass 1.16.4+win on Hyper-V. The host/launcher mismatch remains, but the target version is no longer prerelease.
2. Correct §2 source-review evidence, §3 blocker wording, §5 guard text, and §7 revision history. Preserve the snapshot date and clearly distinguish upstream stable version from locally installed version.
3. Keep W8 and stable release OPEN. Do not start, migrate, upgrade, inspect through guest login, delete, purge, or otherwise operate any Multipass instance. Do not change product scripts, Dockerfiles, workflows, the WBS, or any other lane.
4. Add a short correction note with the official source links and date basis; do not claim a verified host upgrade path or successful VM migration.
5. Commit only the registered W8 report, this follow-up brief, and the new W8 correction dispatcher JSONL. Do not amend or rewrite the earlier commit. Keep the original W8 brief and dispatch log intact.

## Verification

Check only the report/brief/log: exact version statements are consistent; the report still has §0–§7; dispatcher JSONL parses; git diff --check passes. No product tests/builds are required. Report the commit SHA and remaining limits.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
