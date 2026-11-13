# Brief: STAR-DELIVERY-CLEANUP-CONTINUE-20261031

**Agent**: codex
**Phase**: WBS-STAR-DELIVERY-CLEANUP-20261030
**Created**: 2026-10-31 01:04:56

---

Resume the existing Star-only delivery cleanup lane from its current dirty state. Do not reset, overwrite, or drop any existing lane changes. Own only the delivery files already in this worktree. Ensure active image publishing, K3s manifest, Kargo GitOps, ops scripts, and deployment guide identify only Star-supported services: canvas-engine, domain-canvas, canvas-realtime, plus star-api-rest as a separate API service and Envoy ingress. Remove retired canvas-game/game image, deployments, routes, secrets, and subscriptions. Multipass guest VMs with K3s are the supported local/CI direction; do not add Docker Desktop or WSL prerequisites and do not create/delete VMs. Require an explicit kube context/kubeconfig. Run static YAML and shell/PowerShell syntax checks; no live deployment. Commit only after evidence-based review. Report exact files, checks, risks, and SHA.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
