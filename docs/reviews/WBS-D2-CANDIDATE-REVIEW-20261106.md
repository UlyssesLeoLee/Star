# WBS-D2-CANDIDATE-REVIEW-20261106

Status: **Selective integration recommended; two documentation adaptations and one Star-only artifact-boundary gate remain.**

## 固定审查基线

- Target: local `dev@4ae2efb8cc5cf0185a806d1dd13665f51f692b55`
- Candidate: `56d5c9763d0d85e2f93aee501298e2d670835486` — `Retire legacy WSL k3s maintenance entrypoints`
- Candidate ancestry: `dev...56d5c976` reports 211 commits behind and 1 candidate-only commit. A direct fast-forward is not available; any integration must rebase or selectively replay this exact commit after target revalidation.
- Scope: the candidate commit changes 8 executable maintenance/deprecated entrypoints, 3 deployment documents, and adds 1 brief. It does not modify the Multipass provider script, K3s Dockerfile, or retained API build script.

## 文件审查矩阵

| File | Decision | Evidence and rationale |
|---|---|---|
| `maintenance/start-frontend.ps1` | Keep | Comment and relationship metadata clarify that this script starts only the frontend and points backend deployment to a reachable K3s context. Runtime commands are unchanged. |
| `maintenance/start-k3s-backend.bat` | Keep | Replaced the old WSL launcher with an explicit retirement message and exit code 2; no WSL, VM, or cluster command remains. |
| `maintenance/start-k3s-backend.ps1` | Keep | Replaced host/WSL startup logic with an error-only, exit-2 stub. It says this entrypoint cannot create, start, stop, or modify a guest/cluster. |
| `maintenance/update-backend-k3s.bat` | Keep | Replaced the legacy updater wrapper with an error-only, exit-2 stub; no build, push, Helm, or cluster mutation remains. |
| `maintenance/update-backend-k3s.ps1` | Keep | Replaced the legacy updater with an error-only, exit-2 stub. It no longer pulls source, resolves registry tags, or applies a chart. |
| `maintenance/update-frontend.ps1` | Keep | Only its backend-deployment comments and completion guidance changed; frontend update behavior remains unchanged. The new guidance names the current deployment guide and requires a reachable context. |
| `scripts/start-dev.ps1` | Keep | Retired entrypoint now reports the supported frontend path and exits 2; it performs no command execution. |
| `scripts/start-dev.ps1.deprecated` | Keep | Same fail-closed retirement behavior as the legacy entrypoint; it does not start services. |
| `deploy/k3s-local/README.md` | Adapt | Correctly labels the old API image instructions as unsupported and tells users not to run the retained build script. However, `deploy/k3s-local/build-and-deploy.sh` and `deploy/k3s-local/Dockerfile` remain in the tree; the documentation warning alone does not close the Star-only artifact boundary. |
| `deploy/multipass/README.md` | Adapt | The new boundary correctly distinguishes preflight, new-guest provisioning, and deployment to an already reachable cluster, and removes Docker Desktop from the resource rationale. Tighten “Stage all creates a new guest”: provider lines 349–376 skip launch when the configured name already exists; lines 387–405 then probe the guest for up to 25 minutes. A stopped guest is not started or resumed. Date old host/resource observations as historical evidence. |
| `docs/deployment/star-canvas-services-deploy.md` | Keep | Narrows the documented application deployment to a selected, already provisioned and reachable K3s context; says VM/cluster lifecycle is outside this script and the API image path is not supported by this guide. It removes the prior Docker Desktop aside. |
| `docs/briefs/WBS-STAR-DELIVERY-D2-MULTIPASS-MAINTENANCE-20261031.md` | Keep as provenance | This is a brief, not implementation evidence. Its presence does not mean a guest, image, deployment, or rollback was verified. |

## WBS 纠正项与逐文件证据

The WBS description “nine maintenance files are reusable, three deployment documents need adaptation” needs two corrections against the fixed candidate commit:

- `git show --name-only 56d5c9763d0d85e2f93aee501298e2d670835486` lists 12 changed paths: **8 executable entrypoints** (six under `maintenance/`, two under `scripts/`), **3 deployment documents**, and **1 provenance brief**. The exact executable paths are the eight rows at review lines 16–23. Correct the count from nine to eight; “maintenance files” should not aggregate the two `scripts/` paths.
- The three changed deployment documents do not all need adaptation: `deploy/k3s-local/README.md` and `deploy/multipass/README.md` are **Adapt**; `docs/deployment/star-canvas-services-deploy.md` is **Keep**. The latter’s candidate lines 3, 19, 41, and 78 consistently scope the guide to an already provisioned cluster and keep VM lifecycle outside the deployment operation.
- Candidate `deploy/k3s-local/README.md:3,5,20` correctly retires the old API deployment guide and warns against its entrypoint. However, the unchanged `deploy/k3s-local/build-and-deploy.sh:21–30` still copies the entire `crates/` directory into a temporary Docker build context, while the unchanged `deploy/k3s-local/Dockerfile:4–6` copies `crates/` and builds `star-api-rest`. This proves the old build context is not filtered to a Star-only crate allowlist; it does not prove that a game artifact reaches the final image. Keep the Star-only build-context/image gate open.
- Candidate `deploy/multipass/README.md:3,97–101` states the `-Stage all` boundary. The provider at `scripts/automation/multipass_k3s_gitops.ps1:349–376` skips launch for an existing non-Deleted instance, and `:387–405` waits up to 25 minutes for cloud-init through `multipass exec`. A stopped guest therefore is not resumed and can leave provisioning waiting/failing. State the supported behavior precisely: launch when absent; reuse only an already running and usable guest; stopped/unreachable guests need an explicit operator recovery step. The guide’s lines 5 and 136–145 date the prior host deployment evidence to 2026-10-05; retain it as historical evidence, not current readiness proof.

The eight executable changes are limited to retired entrypoints or frontend-only explanatory text. The six retirement stubs return exit code 2 without running host, VM, or cluster operations; the two frontend scripts only clarify the supported deployment boundary.

## Boundary and risk findings

1. The candidate is directionally aligned with the Multipass-first, two-test-instance, no-automatic-purge design. The active provider source uses a cross-process exclusive file lock, a two-instance cap, and a guarded `multipass launch`; the retired startup entrypoints do not expose it as a resume operation.
2. The provider currently reads `multipass list --format json` and rejects malformed inventory. Its code counts a returned `Deleted` record, but the commit does not change that provider or add a test proving the pinned host CLI returns legacy Deleted records in this JSON response. Canonical’s current [list reference](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/list.md) describes listing created instances and documents `--format`; its [removal guide](https://github.com/canonical/multipass/blob/main/docs/how-to-guides/manage-instances/remove-an-instance.md) says Deleted records are a legacy state. This is not runtime evidence for the pinned 1.17.0-rc1 host.
3. The current provider does not call `multipass start`, `recover`, `delete`, or `purge`; an existing stopped guest is not resumed by `-Stage all` and can spend up to 25 minutes in the cloud-init probe. The guide should distinguish “launch if absent” from “reuse running existing guest.”
4. The legacy API Dockerfile/build script remains present and is outside this commit. Its build script stages the entire `crates/` directory and the Dockerfile copies it wholesale; no statement that every Docker context or image is Star-only is supported by this candidate.
5. No live Multipass inventory, guest, K3s API, image, SBOM/provenance, rollout, consumer, or rollback was exercised.

## 审查结论

Keep the **eight executable entrypoint changes** for selective integration after the D2 owner reconciles the WBS and report; adapt the two README files as listed and keep the Canvas deployment guide. No changed candidate file is individually Hold. Hold wholesale integration of the 211-commit-behind source branch; replay only reviewed files/changes against a freshly revalidated dev base. This review does not authorize touching that source worktree, cleaning it, or closing the WBS delivery/artifact gates.

\n