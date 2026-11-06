# WBS-D2-CANDIDATE-REVIEW-20261106

Status: **Review rebased to `dev@59c63d1b`; selective integration remains recommended. The current dev has a filtered Star API build-context implementation and fixture tests, but three documents need target-aware adaptation and live artifact evidence remains open.**

## 固定审查基线

- Target: local `dev@59c63d1b272258e7820e2fdf71c62c206afd686a`
- Candidate: `56d5c9763d0d85e2f93aee501298e2d670835486` — `Retire legacy WSL k3s maintenance entrypoints`
- Candidate ancestry: latest dev has 212 commits absent from `56d5c976`, while the candidate has 1 commit absent from dev. A direct fast-forward is not available; integrate only reviewed changes after target revalidation.
- Scope: the candidate commit changes 8 executable maintenance/deprecated entrypoints, 3 deployment documents, and adds 1 brief. The candidate does not change the Multipass provider or the Cargo-context generator later present on latest dev.

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
| `deploy/k3s-local/README.md` | Adapt | The candidate retires the stale root-context Docker/WSL instructions, but its blanket “no supported API image flow” statement does not match `dev@59c63d1b`: the current helper calls the filtered Cargo-context generator. The target README still contains the old direct-root Docker instructions at lines 43–64. Adapt the document to distinguish the old path from the filtered Multipass guest development helper, and state that the latter is not runtime/release validated. |
| `deploy/multipass/README.md` | Adapt | The new boundary correctly distinguishes preflight, new-guest provisioning, and deployment to an already reachable cluster, and removes Docker Desktop from the resource rationale. Tighten “Stage all creates a new guest”: provider lines 349–376 skip launch when the configured name already exists; lines 387–405 then probe the guest for up to 25 minutes. A stopped guest is not started or resumed. Date old host/resource observations as historical evidence. |
| `docs/deployment/star-canvas-services-deploy.md` | Adapt | The guide remains scoped to the three Canvas services and an already reachable K3s context. Candidate line 19 says all API build/deployment material under `deploy/k3s-local` is retired; latest dev has a filtered local API helper. The current-dev line 19 also calls `deploy/k3s-local/Dockerfile` the API image build path, which omits the helper's context-generation/filtering step and can be read as the old direct Docker context path. Adapt both statements: say this Canvas guide does not build/deploy `star-api-rest`, refer to `deploy/k3s-local/build-and-deploy.sh` only as a local development/test helper that generates a filtered Cargo context, and state that it has no production publishing or runtime acceptance. |
| `docs/briefs/WBS-STAR-DELIVERY-D2-MULTIPASS-MAINTENANCE-20261031.md` | Keep as provenance | This is a brief, not implementation evidence. Its presence does not mean a guest, image, deployment, or rollback was verified. |

## WBS correction and candidate-to-dev comparison

`git show --name-status 56d5c9763d0d85e2f93aee501298e2d670835486` lists 12 changed paths: **8 executable entrypoints** (six under `maintenance/`, two under `scripts/`), **3 deployment documents**, and **1 new provenance brief**. Correct the WBS count from nine to eight; the two `scripts/` entries should not be miscounted as files under `maintenance/`. The eight executable paths are enumerated in review rows 16–23. The candidate commit is still 212 target-only commits behind latest dev.

The candidate snapshot’s build path is stale relative to the target. In commit `56d5c976`, `deploy/k3s-local/build-and-deploy.sh:21–30` copies all of `crates/` into a temporary context, and `deploy/k3s-local/Dockerfile:4–6` copies that directory wholesale. This accurately describes the old candidate snapshot only; it is not the current `dev@59c63d1b` build path.

On current dev, `deploy/k3s-local/build-and-deploy.sh:31–35` invokes `scripts/automation/star_k3s_context.py` with an explicit repository and Multipass instance. The current Dockerfile uses local-crate copy markers (`deploy/k3s-local/Dockerfile:5–10`); the generator replaces them with per-crate `COPY` lines. Static source inspection found that `star_k3s_context.py:241–321` resolves `star-api-rest`'s active Cargo dependency closure, `:165–178` rejects `game`/`legacy` path/package terms, `:549–617` writes an explicit workspace-member list, and `:658–789` copies and validates only the selected crate closure plus required build files. It computes a streaming digest (`:729–742`) and revalidates the context before transfer/build (`:745–789`, `:965–970`).

The current target `deploy/k3s-local/README.md:43–64` still contains the old direct root-context instructions, so all **three** candidate deployment-document areas need adaptation against latest dev: update the local K3s README to describe the filtered helper as development/test-only; update the Multipass README’s Stage-all and stopped-guest wording; and update the Canvas deployment guide. In that guide, candidate line 19 broadly retires all `deploy/k3s-local` API build material, while current-dev line 19 identifies `deploy/k3s-local/Dockerfile` as the API image build path without mentioning the new filter helper. Rewrite this to preserve the guide's Canvas-only scope, identify `deploy/k3s-local/build-and-deploy.sh` as a local development/test helper using generated filtered Cargo context, and keep publishing/runtime acceptance explicitly open. The WBS’s “three deployment documents need adaptation” is therefore correct for current-dev integration, although the earlier candidate-only disposition of the Canvas guide as Keep is superseded by the target comparison.

The changed executable code remains limited to retired entrypoints or frontend-only explanatory text. Six retirement stubs return exit code 2 without running host, VM, or cluster operations; the two frontend scripts only clarify the supported deployment boundary.

## Validation boundary

- `rtk python -m unittest scripts.automation.test_star_k3s_context -v`: **20 tests run; 19 passed and 1 skipped** because this Windows environment lacks permission to create the symlink required by `test_rejects_symlinked_root_input` (`WinError 1314`). The tests use fixture Cargo graphs and mocked Cargo/Multipass/Docker/K3s interactions; they verify closure selection, rejection of game dependencies, path/reparse checks, stable output, tamper detection, and the explicit-guest script contract.
- `rtk bash -n deploy/k3s-local/build-and-deploy.sh`: passed.
- `rtk git diff --check dev...HEAD`: passed for this review lane.
- Coordinator follow-up on `dev@59c63d1b` generated the current full-repository Cargo context in a temporary Linux target: **333 files**, **33 local crates**, SHA-256 `2480ec80558c17f8ab50031e158162f460ad1c3004266138d150326d65f88b86`, and `forbidden_paths=[]`. The temporary output was automatically cleaned; this is current-dev evidence, not a build from candidate `56d5c976`.
- The WBS release gate still needs a retained/archived generated context and digest, guest image digest, SBOM/provenance, and consumer verification. No Docker/Multipass image build, live guest/K3s deployment, rollout, or rollback was exercised; keep those gates open.

## Boundary and risk findings

1. The candidate is directionally aligned with the Multipass-first, two-test-instance, no-automatic-purge design. The active provider source uses a cross-process exclusive file lock, a two-instance cap, and a guarded `multipass launch`; the retired startup entrypoints do not expose it as a resume operation.
2. The provider currently reads `multipass list --format json` and rejects malformed inventory. Its code counts a returned `Deleted` record, but the commit does not change that provider or add a test proving the pinned host CLI returns legacy Deleted records in this JSON response. Canonical’s current [list reference](https://github.com/canonical/multipass/blob/main/docs/reference/command-line-interface/list.md) describes listing created instances and documents `--format`; its [removal guide](https://github.com/canonical/multipass/blob/main/docs/how-to-guides/manage-instances/remove-an-instance.md) says Deleted records are a legacy state. This is not runtime evidence for the pinned 1.17.0-rc1 host.
3. The current provider does not call `multipass start`, `recover`, `delete`, or `purge`; an existing stopped guest is not resumed by `-Stage all` and can spend up to 25 minutes in the cloud-init probe. The guide should distinguish “launch if absent” from “reuse running existing guest.”
4. The stale candidate’s API Dockerfile/build script stages `crates/` wholesale, while current dev has a filtered context generator with fixture tests. Do not carry the stale broad path forward or mistake fixture tests for proof about a real generated context/final image/SBOM.
5. No live Multipass inventory, guest, K3s API, image, SBOM/provenance, rollout, consumer, or rollback was exercised.

## 审查结论

Keep the **eight executable entrypoint changes** for selective integration after the D2 owner reconciles the WBS and report; adapt all three deployment documents for their distinct reasons: the local K3s README retains old direct root-context instructions, the Multipass README overstates `Stage all`/stopped-guest recovery, and the Canvas guide's retirement/Dockerfile descriptions conflict with the filtered helper now in current dev. No changed candidate file is individually Hold. Hold wholesale integration of the 212-commit-behind source branch; replay only reviewed files/changes against a freshly revalidated dev base. This review does not authorize touching that source worktree, cleaning it, or closing the WBS delivery/artifact gates.

\n
