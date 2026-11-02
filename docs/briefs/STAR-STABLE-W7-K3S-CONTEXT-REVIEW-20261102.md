# Brief: STAR-STABLE-W7-K3S-CONTEXT-REVIEW-20261102

**Agent**: reviewer
**Phase**: STAR-STABLE-W7-K3S-CONTEXT-REVIEW-20261102
**Created**: 2026-11-02 12:20:48

---

# WBS task: W7 local K3s filtered-context independent review
Owner: reviewer
Base: dev at the current coordinated checkout; this lane is documentation-only.
Scope: independently review the current Star local K3s image build/deploy path against the requirement that build context, cache, and image inputs contain Star-only payloads. Inspect the generated-context builder, Dockerfile, deployment helper, focused tests, and current WBS evidence. Run only focused static/context tests supported by the current environment. Do not create/start/stop/purge Multipass instances, deploy to K3s, or touch the D2 Multipass worktree. Separately verify package-tree traversal against a Windows junction using an isolated temporary fixture and the actual copy function; do not write repository source. Distinguish source-level filtering from actual build-context archive, final image, digest, SBOM/provenance, consumer, rollout, and rollback proof. If a code gap appears, record it as a blocker and do not change code in this documentation-only lane.
Do not touch WBS, main, remotes, or D:\Star user files. Output only the four files in the registered manifest, include exact evidence and commands, and keep W7/release OPEN unless all acceptance evidence exists. If a generator defect is confirmed, recommend a separate implementation + regression lane; this lane remains docs-only. Finish with git diff --check and one commit whose subject references this brief path.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
