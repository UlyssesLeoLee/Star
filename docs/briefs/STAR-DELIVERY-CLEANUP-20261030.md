# Brief: STAR-DELIVERY-CLEANUP-20261030

**Agent**: worker
**Phase**: STAR-DELIVERY-CLEANUP-20261030
**Created**: 2026-10-30 15:20:50

---

Scope: Make Star's active image-publishing and Kubernetes deployment artifacts Star-only, removing the retired canvas game service. Preserve the real Star Canvas services: canvas-engine, domain-canvas, and canvas-realtime. Keep the existing Star API image build path (deploy/k3s-local/Dockerfile) intact; do not invent registry package names or publish an API image without an existing contract.

Owned files only:
- .github/workflows/publish-canvas-game.yml (rename to a Star-named workflow)
- deploy/canvas-game-k3s.yaml (rename to a Star-named manifest)
- deploy/gitops/kargo/kargo-dev-staging.yaml
- scripts/start-k3s-backend.ps1 (rename to a Star-named script)
- scripts/verify-5-services.sh (rename to a Star-named script)
- docs/deployment/canvas-game-deploy.md (replace with a Star-named deployment guide)
- this brief and docs/reports/dispatcher/STAR-DELIVERY-CLEANUP-20261030.jsonl

Requirements:
1. Remove the game image from CI test/build matrices, Kargo subscriptions, K3s resources/routes/monitoring, secrets/config, and scripts. Keep only the three actual Star Canvas backend services and existing ingress resources in this Canvas deployment. Do not claim star-api-rest is part of this manifest if it remains separately deployed.
2. Rename active workflow, manifest, script, and deployment-guide paths so their names identify Star/Star Canvas. Update all references within owned files. Tell the coordinator any remaining references in files owned by another lane.
3. Keep image paths under the existing Star GHCR namespace and preserve credentials/trigger/tag behavior unless a concrete issue makes a narrow correction necessary. Never expose credential values.
4. Keep the standalone deploy/k3s-local/Dockerfile for star-api-rest; do not add Docker Desktop assumptions or deploy to any live cluster.

Acceptance:
- Static scan of owned active deployment files finds no canvas-game/game-service image, deployment, route, or old active file path.
- Workflow/build matrix lists only the three retained Canvas crates.
- Kubernetes manifest is syntactically valid by an available local parser; if none is available, report that limit and run structural checks.
- Run relevant existing script/config checks without changing unrelated behavior. Do not push or edit dev/main directly.
- Commit the complete lane on this worktree. Use commit author Ulysses <ulysses@mavis.local> per repository instructions. Report changed paths, validation commands/results, and any remaining cross-lane references.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
