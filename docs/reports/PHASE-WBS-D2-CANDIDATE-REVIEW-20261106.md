# PHASE-WBS-D2-CANDIDATE-REVIEW-20261106

Status: **Review complete / selective integration recommended with two documentation adaptations and one open Star-only artifact-boundary gate.** This is not a deployment or release acceptance.

## §0 目的

Independently review the committed D2 Multipass/K3s maintenance candidate against current local dev and the Star delivery constraints. Preserve the active D2 source worktree and its uncommitted WBS/report files.

## §1 改动矩阵

| Area | Result | Decision |
|---|---|---|
| Eight PowerShell/BAT entrypoints | Six PowerShell files parse; retired K3s/backend and start-dev entrypoints now emit errors and exit 2; frontend scripts receive explanatory wording only. | Keep for selective integration. |
| `deploy/k3s-local/README.md` | Marks the API path unsupported and warns against the retained build script. The Dockerfile and script remain present. | Adapt; warning does not close artifact boundary. |
| `deploy/multipass/README.md` | Adds the two-instance and no-resume boundary and removes a Docker Desktop resource reference. | Adapt “Stage all creates a guest”: provider lines 349–376 skip launch for existing names and lines 387–405 can probe for up to 25 minutes; stopped guests are not resumed. Keep 2026-10-05 host evidence historical. |
| `docs/deployment/star-canvas-services-deploy.md` | Restricts deployment to an already provisioned/reachable context and disclaims API build/deployment support. | Keep. |
| Candidate brief | Added as task provenance. | Keep as provenance only. |

**WBS correction:** `git show --name-only` confirms 12 candidate paths: 8 executable entrypoints (six `maintenance/`, two `scripts/`), 3 changed deployment documents, and 1 provenance brief. Correct “9 maintenance files” to 8 executable entrypoints. Of the three changed deployment documents, only two need adaptation (`deploy/k3s-local/README.md` and `deploy/multipass/README.md`); `docs/deployment/star-canvas-services-deploy.md` is Keep. The review matrix above enumerates each entrypoint and document disposition.

## §2 验证摘要

- Candidate commit: `56d5c9763d0d85e2f93aee501298e2d670835486`; target: `dev@4ae2efb8cc5cf0185a806d1dd13665f51f692b55`. Ancestry is 211 behind / 1 ahead; fast-forward is unavailable without rebasing or replaying the candidate commit.
- `rtk git diff --check dev...56d5c976`: passed (exit 0).
- PowerShell parser: all 6 changed PowerShell script files in the D2 candidate worktree (including the `.ps1.deprecated` file) parsed without errors.
- Static scan of the retired PowerShell stubs found no WSL, Docker, Helm, kubectl, Multipass, process-start, process-stop, deletion, or git-pull command lines. The two BAT stubs were read and contain only error messages plus exit code 2.
- Pointer/file existence checks passed for `scripts/start-star-backend.ps1`, the retained API build script and Dockerfile, and the Canvas manifest. In the candidate source, `deploy/k3s-local/build-and-deploy.sh:21–30` stages the entire `crates/` directory and `deploy/k3s-local/Dockerfile:4–6` copies it wholesale; this leaves the build-context payload gate open (it does not establish final image contents).
- Read-only inspection of the current Multipass provider confirmed a two-instance constant, exclusive file lock, inventory JSON parsing with fail-closed errors, handling of a returned `Deleted` item, and no explicit start/recover/delete/purge call. A stopped existing instance skips launch and is then probed with `multipass exec`.
- Canonical documentation lookup supports the documented `multipass list` format and legacy Deleted-state context, but does not establish 1.17.0-rc1 behavior on this host.
- CodeRabbit CLI review was not run: the available wrapper resolved to `/root/.local/bin/coderabbit` and returned permission denied. No external review result is claimed.
- No user request or evidence authorized a live Multipass/K3s run, so none was performed.

## §3 已知缺口

1. `deploy/k3s-local/build-and-deploy.sh` and `deploy/k3s-local/Dockerfile` still exist. The former stages all of `crates/` (`:21–30`) and the latter copies it (`:4–6`); filtering plus actual filtered-context/final-image evidence remain separate Star-only release gates. No claim about game payload in the final image was verified.
2. No exact-host evidence proves `multipass list --format json` includes legacy Deleted records on the pinned version. The provider logic only proves it counts such records if returned.
3. `-Stage all` is not a resume operation. Provider lines 349–376 skip launch if the configured name already exists; lines 387–405 may continue probing cloud-init for 25 minutes via `multipass exec`. A stopped guest is not started. Do not describe `-Stage all` as creating a new guest unconditionally or as recovering an existing guest.
4. The Multipass README includes dated host observations. Treat them as historical and do not use them as current resource-capacity or release evidence.
5. Actual VM, K3s readiness, final image digest, SBOM/provenance, consumer, and rollback evidence remain NOT RUN.
6. The D2 source branch is 211 commits behind dev. Its worktree contains separate dirty/untracked WBS/report content; this review did not read or change those files. Review that content separately before any cleanup.

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
13. A README warning does not prove the retained API Docker context is Star-only.
14. Do not mark runtime/deployment/SBOM/provenance/rollback gates complete from static review.
15. This lane does not modify main, push, tag, publish, or deploy.

## §6 签字栏

| 角色 | 决定 |
|---|---|
| 架构 | Mavis review: accept the fail-closed retirement subset; documentation and artifact-boundary follow-ups remain open. |
| SRE Lead | Open pending real inventory, two-instance, guest readiness, and recovery evidence. |
| 平台 | Open pending filtered context, image digest, SBOM/provenance, and deployment verification. |
| 评审主持 | Mavis review per DEC-008; source worktree remains protected. |
| PM | Partial; no WBS or stable-release gate closed by this review alone. |

## §7 修订历史

| Version | Date | Author | Revision | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Reviewed D2 candidate file-by-file; corrected the executable-file count and separated static review from host/runtime acceptance. | User-requested WBS parallel integration review. |

\n