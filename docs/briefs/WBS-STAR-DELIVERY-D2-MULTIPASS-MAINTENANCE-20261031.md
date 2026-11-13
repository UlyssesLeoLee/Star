# Brief: WBS-STAR-DELIVERY-D2-MULTIPASS-MAINTENANCE-20261031

**Agent**: worker
**Phase**: WBS-STAR-DELIVERY-D2-MULTIPASS-MAINTENANCE-20261031
**Created**: 2026-10-31 03:19:35

---

Use this isolated lane at D:/orcaWork/Star/star-delivery-d2-multipass-maintenance-20261031, branch UlyssesLeoLee/star-delivery-d2-multipass-maintenance-20261031, based on dev@f34866b8658ad6a226c648a6f88c745d7195ab74.

Goal: remediate the active legacy K3s maintenance entrypoints found by D4 so Star development uses the supported Multipass-managed Linux guest K3s path. Inspect the actual scripts and deployment guide first. Make the old WSL/Docker Desktop/old Helm 8-service/rustgameserver paths impossible to run accidentally: either safely route them to an already-supported Multipass operation or deprecate them with a clear, accurate fail-closed message and supported next step. Do not invent a new deployment flow if the current provider does not support it.

Preserve the current two-Multipass-inventory-instance limit, global lock, inventory counting, fail-closed behavior, and no-auto-purge. Do not create, start, stop, delete, or purge any Multipass instance; do not invoke Docker or Kubernetes operations; do not read or print credentials. Do not use or require Docker Desktop. Do not remove historical game requirements/ADRs; only clean active operational paths. Keep image/service names Star-only.

Owned manifest only: the WBS, D4 report, this brief, this phase report, this dispatcher audit, the four maintenance K3s batch/PowerShell entrypoints, the two frontend scripts that point at the old updater, both deprecated start-dev PowerShell files, the k3s-local and Multipass READMEs, the current Canvas deployment guide, and scripts/automation/multipass_k3s_gitops.ps1. Do not touch files outside the registered manifest. Keep the Multipass provider unchanged unless source review proves a minimal correction is necessary; if you change it, record why and preserve all cap/lock invariants.

Run bounded static checks only: PowerShell parser for changed .ps1 files, batch text/reference checks, documentation/active-path scan for the old registry, WSL, Docker Desktop, stale eight-service and game deployment strings, dispatcher registry check only if registry files are changed, and git diff --check. Do not claim K3s runtime, image build/publish, Cargo build, or release readiness. Update D2/D4 WBS and phase documents to exactly match observed results and leave D4 open for build/runtime gates. Commit only reviewed in-scope changes; commit message must reference docs/briefs/WBS-STAR-DELIVERY-D2-MULTIPASS-MAINTENANCE-20261031.md; do not merge or clean the worktree.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
