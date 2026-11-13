# Brief: STAR-STABLE-W7-MULTIPASS-DEPLOY-20261102

**Agent**: worker
**Phase**: W7-MULTIPASS-DEPLOY-20261102
**Created**: 2026-11-02 05:26:30

---

## Objective
Resolve the W7 local deployment helper mismatch: it currently builds with host Docker and invokes the host K3s binary, while supported K3s clusters run inside Multipass-managed Linux guests. Make the helper operate on one explicitly selected, already-existing Multipass guest with K3s installed, without requiring Docker Desktop.

## Required behavior
- Require an explicit Multipass instance name; missing, stopped, or unresolvable target must fail closed before build/deploy side effects.
- Use the existing Star-only Cargo context generator for all build inputs. Keep its path, package, lockfile, Dockerfile, symlink, digest, and forbidden game/legacy checks.
- Route guest operations through the supported Multipass CLI. Build the image in the selected guest using an already available build engine, import it into that guest's K3s containerd `k8s.io` namespace, then apply and verify the deployment through that guest's K3s kubectl.
- Before transfer or build, require at least one K3s node and require every reported node to have `Ready=True`; missing, malformed, or unready node status must fail closed.
- Do not create, start, stop, delete, or purge Multipass instances as part of this helper. The current test inventory cap is two records total; do not weaken or replace its existing locked launcher.
- Use fixed argument arrays and validated instance/tag/path values; do not expose arbitrary host or guest shell execution.
- If the selected guest lacks the required build tooling, stop and report the missing prerequisite. Do not install tooling, fall back to host K3s, assume Docker Desktop, or claim a successful deployment.

## Scope and acceptance
- Own only the files in the MAGOS manifest: deploy/k3s-local/build-and-deploy.sh; scripts/automation/star_k3s_context.py; scripts/automation/test_star_k3s_context.py; this brief; the W7 phase report; its dispatch JSONL; the W7 review; and the stable-release WBS.
- Add focused tests proving the explicit-instance fail-closed path, Ready-node admission, filtered-context-only build, and that image import/deploy commands target the selected guest rather than host K3s. Keep existing Star-only and tamper checks.
- Update the phase report and WBS with exact commands/results and clearly state that no live Multipass/K3s deployment, Rust 1.99.0 build, image digest/SBOM/provenance, consumer verification, or rollback was proven unless actually run.
- Keep W7 OPEN/BLOCKED and v1.0.6 NO-GO until the full release gates pass. The helper's guest-level K3s node preflight is a local deployment check; it does not implement Star-native Infrastructure owner reservations, cluster identity binding, Infrastructure readiness admission, or production deployment.
- Run only focused checks supported by the current environment; do not claim tests that were not run. Commit the completed lane with this brief path in the commit message and report the commit hash.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
