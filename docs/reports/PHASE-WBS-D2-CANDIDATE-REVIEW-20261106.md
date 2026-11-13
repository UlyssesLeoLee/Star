# PHASE-WBS-D2-CANDIDATE-REVIEW-20261106

Status: **Review rebased to `dev@59c63d1b`; selective integration recommended. Latest dev has a filtered Cargo context generator and fixture tests, but all three deployment-document areas need target-aware adaptation and runtime artifact evidence remains open.** This is not a deployment or release acceptance.

## §0 目的

Independently review the committed D2 Multipass/K3s maintenance candidate against current local dev and the Star delivery constraints. Preserve the active D2 source worktree and its uncommitted WBS/report files.

## §1 改动矩阵

| Area | Result | Decision |
|---|---|---|
| Eight PowerShell/BAT entrypoints | Six PowerShell files parse; retired K3s/backend and start-dev entrypoints now emit errors and exit 2; frontend scripts receive explanatory wording only. | Keep for selective integration. |
| `deploy/k3s-local/README.md` | Candidate retires the old root-context Docker/WSL path; latest dev calls a filtered Cargo-context generator from the Multipass helper. | Adapt to describe the filtered helper as local development/test only; no runtime/release proof. |
| `deploy/multipass/README.md` | Adds the two-instance and no-resume boundary and removes a Docker Desktop resource reference. | Adapt “Stage all creates a guest”: provider lines 349–376 skip launch for existing names and lines 387–405 can probe for up to 25 minutes; stopped guests are not resumed. Keep 2026-10-05 host evidence historical. |
| `docs/deployment/star-canvas-services-deploy.md` | Canvas guide remains limited to Canvas services. Candidate line 19 broadly calls all `deploy/k3s-local` API build material retired; current-dev line 19 still names `deploy/k3s-local/Dockerfile` as the API build path, without mentioning the filtered-context helper. | Adapt both statements: keep this guide explicitly Canvas-only, describe `deploy/k3s-local/build-and-deploy.sh` only as a local development/test helper with generated filtered Cargo context, and leave production publishing/runtime acceptance open. |
| Candidate brief | Added as task provenance. | Keep as provenance only. |

**WBS correction:** `git show --name-status` confirms 12 candidate paths: 8 executable entrypoints (six `maintenance/`, two `scripts/`), 3 changed deployment documents, and 1 new brief. Correct “9 maintenance files” to 8 executable entrypoints. Latest-dev comparison makes all three deployment-document areas Adapt. For the previously Keep Canvas guide, candidate line 19 says all `deploy/k3s-local` API build material is retired, but current-dev line 19 names the Dockerfile as the API build path without the filtered-helper step. Preserve the guide's Canvas-only scope, name the filtered helper only as local development/test support, and keep publishing/runtime acceptance open. The review matrix above enumerates each entrypoint and current-target document disposition.

## §2 验证摘要

- Candidate commit: `56d5c9763d0d85e2f93aee501298e2d670835486`; target: `dev@59c63d1b272258e7820e2fdf71c62c206afd686a`. Latest dev has 212 commits absent from candidate; candidate has 1 commit absent from dev. Whole-branch fast-forward is unavailable.
- `rtk git diff --check dev...56d5c976`: passed (exit 0); after rebasing this review lane, `rtk git diff --check dev...HEAD`: passed.
- PowerShell parser: all 6 changed PowerShell script files in the D2 candidate worktree (including the `.ps1.deprecated` file) parsed without errors.
- Static scan of the retired PowerShell stubs found no WSL, Docker, Helm, kubectl, Multipass, process-start, process-stop, deletion, or git-pull command lines. The two BAT stubs were read and contain only error messages plus exit code 2.
- Stale candidate evidence: `56d5c976:deploy/k3s-local/build-and-deploy.sh:21–30` copies all of `crates/`; its Dockerfile lines 4–6 copies that whole directory. This is not the current dev build path.
- Current dev evidence: `deploy/k3s-local/build-and-deploy.sh:31–35` invokes `scripts/automation/star_k3s_context.py` with an explicit repo and Multipass instance. `deploy/k3s-local/Dockerfile:5–10` has generated per-crate copy markers instead of `COPY crates ./crates`. Static source inspection shows the generator selects the resolved `star-api-rest` Cargo closure, rejects `game`/`legacy` packages/paths, writes an explicit workspace-member set, validates the file allowlist, and computes/rechecks a streaming context digest (`star_k3s_context.py:165–178, 241–321, 549–617, 658–789, 729–742, 965–970`).
- `rtk python -m unittest scripts.automation.test_star_k3s_context -v`: 20 tests, 19 passed and 1 skipped because symlink creation is not permitted in this Windows environment (`WinError 1314`). Fixture tests cover closure filtering, game-dependency rejection, path/reparse guards, stable output, tamper detection, and explicit selected-guest contract. They mock Cargo/Multipass/Docker/K3s interactions.
- `rtk bash -n deploy/k3s-local/build-and-deploy.sh`: passed. The current target `deploy/k3s-local/README.md:43–64` still describes the old direct root-context Docker path and needs adaptation.
- Read-only inspection of the current Multipass provider confirmed a two-instance constant, exclusive file lock, inventory JSON parsing with fail-closed errors, handling of a returned `Deleted` item, and no explicit start/recover/delete/purge call. A stopped existing instance skips launch and is then probed with `multipass exec`.
- Canonical documentation lookup supports the documented `multipass list` format and legacy Deleted-state context, but does not establish 1.17.0-rc1 behavior on this host.
- CodeRabbit CLI review was not run: the available wrapper resolved to `/root/.local/bin/coderabbit` and returned permission denied. No external review result is claimed.
- Coordinator follow-up on `dev@59c63d1b` generated a temporary Linux Cargo context: 333 files, 33 local crates, SHA-256 `2480ec80558c17f8ab50031e158162f460ad1c3004266138d150326d65f88b86`, `forbidden_paths=[]`. Temporary output was automatically cleaned; this was not a build from candidate `56d5c976`.
- No retained context archive, Docker/Multipass image build, final guest image digest, SBOM/provenance, consumer verification, live guest/K3s deployment, rollout, or rollback was produced.

## §3 已知缺口

1. The broad context applies to the stale candidate only; latest dev has a filtered generator, fixture tests, and a coordinator-run current-dev context-generation pass (333 files / 33 local crates / no forbidden paths). That temporary context was auto-cleaned and is not an archived artifact. Final guest image contents/digest, SBOM/provenance, and consumer verification remain unverified. Do not report the stale path as current dev behavior or fixture/static checks as release acceptance.
2. No exact-host evidence proves `multipass list --format json` includes legacy Deleted records on the pinned version. The provider logic only proves it counts such records if returned.
3. `-Stage all` is not a resume operation. Provider lines 349–376 skip launch if the configured name already exists; lines 387–405 may continue probing cloud-init for 25 minutes via `multipass exec`. A stopped guest is not started. Do not describe `-Stage all` as creating a new guest unconditionally or as recovering an existing guest.
4. The Multipass README includes dated host observations. Treat them as historical and do not use them as current resource-capacity or release evidence.
5. Actual VM, K3s readiness, final image digest, SBOM/provenance, consumer, and rollback evidence remain NOT RUN.
6. The D2 source branch is 212 commits behind latest dev. Its worktree contains separate dirty/untracked WBS/report content; this review did not read or change those files. Review that content separately before any cleanup.

## §4 子代理失败接手清单

- No delegated child worker was used inside this review lane; the coordinator delegated this bounded audit to one worker.
- No product code was changed. The coordinator owns the integration decision and any follow-up documentation adaptation.
- D2 source worktree and its dirty/untracked files remain untouched.

## §5 守门规则

1. Revalidate dev HEAD and clean status immediately before integration.
2. Integrate only the reviewed candidate commit/file set; do not merge the full stale branch wholesale.
3. Preserve the D2 owner worktree and its dirty/untracked reports until independently archived/reviewed.
4. Do not start, stop, recover, delete, or purge guests through a deprecated entrypoint.
5. Multipass remains the VM provider; Docker Desktop is not a prerequisite.
6. Keep the test-phase inventory limit at two instances.
7. Count stopped instances; require direct evidence before claiming legacy Deleted records are included by the host listing.
8. Unknown or malformed inventory must fail closed.
9. Keep the cross-process lock around inventory check and VM launch.
10. Never auto-purge or treat a deployment helper as permission to destroy a guest.
11. Only deploy to an explicitly selected, already provisioned, reachable K3s context.
12. Do not expose unrestricted host shell, daemon/socket, or host mounts to agents/tasks.
13. Keep the current dev filter implementation distinct from the stale candidate's broad `crates/` path.
14. Fixture tests and static allowlist checks do not prove a generated full-repo context, final image, SBOM/provenance, or rollout.
15. This lane does not modify main, push, tag, publish, or deploy.

## §6 签字栏

| 角色 | 决定 |
|---|---|
| 架构 | Mavis review: accept the fail-closed retirement subset; adapt the local K3s README for stale root-context instructions, the Multipass README for stopped-guest semantics, and the Canvas guide for conflicting API build descriptions. |
| SRE Lead | Open pending real inventory, two-instance, guest readiness, and recovery evidence. |
| 平台 | Static filter and fixture tests pass; coordinator generated the current-dev temporary context. Retained context archive, image digest, SBOM/provenance, consumer, and deployment verification remain open. |
| 评审主持 | Mavis review per DEC-008; source worktree remains protected. |
| PM | Partial; no WBS or stable-release gate closed by this review alone. |

## §7 修订历史

| Version | Date | Author | Revision | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Reviewed D2 candidate file-by-file; corrected the executable-file count and separated static review from host/runtime acceptance. | User-requested WBS parallel integration review. |
| v0.2 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Rebased review onto dev@59c63d1b; separated stale candidate-wide-crates context from current filtered-context implementation; recorded 20 fixture tests (19 pass, 1 skipped) and the coordinator's temporary 333-file / 33-crate context generation. Kept retained-context archive, image/SBOM/consumer runtime evidence open. | Coordinator requested latest-dev recheck and supplied context-generation evidence. |
| v0.3 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Clarified the Canvas deployment-guide adaptation: candidate line 19 broadly retires all API build material, while current dev line 19 names the Dockerfile without the filtered-helper step; documented the exact local-dev/test-only wording needed. | Coordinator requested concrete per-document Adapt reasons against current dev. |

\n
