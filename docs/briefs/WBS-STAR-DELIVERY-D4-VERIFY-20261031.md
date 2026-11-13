# Brief: WBS-STAR-DELIVERY-D4-VERIFY-20261031

**Agent**: worker
**Phase**: WBS-STAR-DELIVERY-D4-VERIFY-20261031
**Created**: 2026-10-31 02:27:32

---

## Objective
Execute the static portion of WBS D4 for Star-only container and deployment delivery on baseline dev@c6afc36f09b4c5e41de1d31a8b9b35574be8b6ad. Confirm active Docker, GHCR workflow, K3s/Helm/Kargo manifests, startup/check scripts, and operator docs all describe only current Star services and use Star-scoped image identities. Correct confirmed active legacy references only within the registered 21-path manifest.

## WBS acceptance scope
- Verify the 3 active Canvas backend image names are Star-prefixed and match the GHCR workflow matrix, Dockerfile paths, K3s manifest, Helm values, Kargo image updater and release docs.
- Scan active workflow/deployment/startup surfaces for retired game service names, images, routes, workloads, registries, and stale release instructions. Distinguish active executable configuration from explicitly historical comments.
- Run bounded static validation available locally: YAML parse for touched deployment/workflow files, script syntax checks for touched scripts, and deterministic image/service inventory checks. Do not build or publish images.
- Update WBS D4 only with evidence actually obtained. Keep Rust workspace build, OCI image build/publish, K3s endpoint/runtime and deployment gates OPEN if not verified. No Docker Desktop, no Multipass instance creation/start, and do not exceed the two-instance test cap.

## Owned files
Only these MAGOS-registered paths may be edited:
docs/implementation-plans/WBS-STAR-DELIVERY-CLEANUP-20261030.md
docs/briefs/WBS-STAR-DELIVERY-D4-VERIFY-20261031.md
docs/reports/PHASE-STAR-DELIVERY-D4-VERIFY-20261031.md
docs/reports/WBS-DISPATCH-STAR-DELIVERY-D4-VERIFY-20261031.jsonl
.github/workflows/publish-star-canvas-services.yml
deploy/star-canvas-services-k3s.yaml
deploy/k3s-local/Dockerfile
deploy/k3s-local/build-and-deploy.sh
deploy/k3s-local/star-api-rest-deploy.yaml
deploy/k3s-local/kustomization.yaml
deploy/k3s-local/README.md
deploy/gitops/kargo/kargo-dev-staging.yaml
deploy/helm/star/Chart.yaml
deploy/helm/star/values.yaml
crates/canvas-engine/Dockerfile
crates/domain-canvas/Dockerfile
crates/canvas-realtime/Dockerfile
maintenance/update-backend-k3s.bat
scripts/start-star-backend.ps1
scripts/verify-star-services.sh
docs/deployment/star-canvas-services-deploy.md

## Required boundaries
Work only in this Orca worktree at D:\orcaWork\Star\wbs-star-delivery-d4-verify-20261031 on branch codex/wbs-star-delivery-d4-verify-20261031. Inspect each file before changing it; do not overwrite existing work. Do not read/write D:\Star or any other worktree. Do not touch main, push, or merge. If a defect requires an unregistered path, stop and report the exact path for a new MAGOS plan. Do not broaden the report beyond verified static checks; no code tests or runtime claims.

## Handoff
Record changed paths, commands and exact results, unresolved D4 gates, and the commit SHA. Commit only this lane's accepted changes after diff review. The coordinator will independently review and rebase/fast-forward into local dev.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
