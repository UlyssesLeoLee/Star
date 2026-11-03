# Brief: STAR-STABLE-W7-K3S-JUNCTION-GUARD-20261102

**Agent**: worker
**Phase**: STAR-STABLE-W7-K3S-JUNCTION-GUARD-20261102
**Created**: 2026-11-02 12:53:12

---

# WBS task: W7 filtered K3s build-context junction guard and Windows regression

Implement the confirmed security fix in scripts/automation/star_k3s_context.py and scripts/automation/test_star_k3s_context.py. The independent W7 review proved that _copy_local_package follows a Windows directory junction inside a selected package and copies content from outside the repository into the build context. Existing guards use Path.is_symlink(), which returned false for the junction.

Before editing, read the repository AGENTS.md and this brief, inspect the existing source/test Cypher headers, and inspect the actual caller and package path checks. Query CodeGraph for the bounded symbols first. Parent attempted codegraph_explore on the exact two files but the result was unrelated and did not establish source coverage; index freshness/generation and check_index_coverage were unavailable in this session. Parent fallback evidence: _copy_local_package at source lines 590-625 uses os.walk(followlinks=False), tests child.is_symlink and source.is_symlink, then copies files; existing fixture symlink regression is test_rejects_symlinked_root_input around test lines 335-345; no package-child junction regression exists. Parent review ran an isolated real Windows mklink /J experiment against production _copy_local_package and observed is_symlink=False, is_junction=True, outside_payload_copied=True.

Keep this change bounded to the two source/test files and the six registered documentation files plus docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md. Do not edit D2 files, Multipass scripts/inventory, other reports, Cargo files, Docker workflows, main, remote refs, or other worktrees. Preserve the user requirement that Star Docker contexts contain only the approved Star Cargo closure and fail closed on links/reparse points. Reject the junction before descending or copying. Preserve ordinary symlink rejection and ensure inability to classify a Windows reparse point fails closed, including where Path.is_junction is unavailable. Avoid broad refactors or exposing unrestricted host paths.

Add a real Windows junction regression fixture under a selected package directory pointing outside the fixture repository. Place a sentinel only in the outside target, assert the context generation raises ContextError, and assert the sentinel is not copied. Use a safe junction cleanup that removes the link itself and never walks its target. Skip only on non-Windows; on Windows report junction creation failures rather than silently passing. Keep all existing tests.

Run the focused Python tests and relevant syntax checks through RTK. Reconcile Cypher headers with concrete imports/calls/tests and run the CypherGraph Guardian changed/stamp check from the repository root. Do not report any Docker, Multipass, K3s, image, SBOM, provenance, or hosted evidence unless actually run. Update the new phase report, test-design, review, dispatch receipt, and WBS so the junction defect and its verified fix are accurately recorded; W7 and v1.0.6 remain OPEN/NO-GO because real image and deployment gates remain. Keep reports in the required seven-section structure. Commit only files in the registered manifest, with a subject/body referencing this brief. Send the commit SHA, focused test output, Cypher guard result, report state, and clean status for independent review.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
