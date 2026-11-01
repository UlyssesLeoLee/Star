# PHASE-STAR-STABLE-W7-CANVAS-BUILDX-20261101

**Version:** v0.1  
**Date:** 2026-10-31  
**Lane:** `agent/stable-release-wave4/canvas-buildx`  
**Base:** `dev@2a57fa0d329be830735d6657677236e195673362`  
**Status:** Implementation and focused static checks complete; lane commit created; coordinator integration pending.

## §0 Purpose

Pin the Buildx binary used by the Star Canvas release workflow and serialize release jobs per release ref, commit SHA, and service image. The lane changes release controls only. It does not establish that the stable release is ready or prove a hosted image build.

## §1 Change matrix

| Area | Change | Result |
|---|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | Pin Buildx to `v0.38.1` through `docker/setup-buildx-action` input `version`; the action itself remains pinned to commit `8d2750c68a42422c14e847fe6c8ac0403b4cbd6f`. | Removes dependence on whichever Buildx version happens to be preinstalled on a GitHub runner. |
| `.github/workflows/publish-star-canvas-services.yml` | Add job concurrency group `star-canvas-publish-${{ github.workflow }}-${{ github.ref }}-${{ matrix.image }}`, `queue: max`, and `cancel-in-progress: false`. | Every run for the same release ref/tag and service image shares a group, independent of commit SHA. Running jobs are not canceled; multiple pending jobs are retained within the documented queue limit. |
| Existing release workflow | Retain the three-image allowlist, per-service filtered Cargo contexts, non-Canvas/game exclusion checks, release confirmation gate, and `sha256` digest validation. | Existing isolation and fail-closed checks remain in place. |
| `docs/reports/PHASE-STAR-STABLE-W7-CANVAS-BUILDX-20261101.md` | Record sources, static checks, limitations, and handoff. | This lane’s evidence is explicit; no hosted result is claimed. |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W7-CANVAS-BUILDX-20261101.jsonl` | Append the focused static-validation result using the dispatcher audit schema. | Captures the validation outcome for this task. |

## §2 Verification summary

### Official source review

Checked on 2026-10-31:

- Docker’s official [Buildx v0.38.1 release](https://github.com/docker/buildx/releases/tag/v0.38.1) labels it the latest release and records release date 2026-10-20. Its release notes identify the updated Go runtime security fix.
- The exact pinned [`setup-buildx-action` `action.yml`](https://github.com/docker/setup-buildx-action/blob/8d2750c68a42422c14e847fe6c8ac0403b4cbd6f/action.yml) declares the optional `version` input.
- [Docker’s version-pinning documentation](https://docs.docker.com/build/ci/github-actions/configure-builder/#version-pinning) documents `version` as the input for selecting a fixed Buildx release.
- [GitHub Actions concurrency documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency) permits `matrix` expressions in job concurrency groups, allows at most one running job in a group, and documents `queue: max`: up to 100 jobs/runs may be pending; further arrivals are canceled. It also specifies FIFO by the time each run starts waiting, so dispatch order itself is not guaranteed, and `queue: max` cannot be combined with `cancel-in-progress: true`.

### Local static checks

| Check | Command / method | Result |
|---|---|---|
| YAML parse and targeted assertions | PowerShell here-string piped to `rtk proxy python -`; PyYAML parsed the workflow and assertions checked the Buildx pin, exact setup action SHA, three service names, ref/tag and matrix-image concurrency key (no SHA identity), `queue: max`, `cancel-in-progress: false`, filtered-context guard strings, and immutable digest guard. | PASS |
| Whitespace/error-marker check | `rtk proxy git diff --check` | PASS, exit 0 |
| Action-specific linter availability | `Get-Command actionlint, yamllint, node -ErrorAction SilentlyContinue` | `actionlint` and `yamllint` were unavailable; `node` was present. No substitute workflow linter was run. |

No Cargo commands were needed because no Rust source or manifest changed. No workflow was dispatched, no image was built, no GHCR login or publication was performed, and no remote result is claimed.

## §3 Known gaps

1. No GitHub-hosted workflow run has validated runner behavior, the pinned Buildx download, filtered context assembly, provenance, SBOM, or registry publication.
2. YAML parsing and targeted assertions do not validate GitHub Actions expression semantics. `actionlint` was unavailable in this environment.
3. The concurrency key uses the stable tag/ref name rather than SHA, so multiple runs for the same tag and matrix image serialize even if the resolved commit SHA differs. Concurrency does not enforce tag immutability: if a tag is retargeted, a later run could publish a different commit to the same semantic-version tag after the earlier run. Repository governance must prevent tag retargeting.
4. `queue: max` retains up to 100 pending jobs/runs in each group in addition to the single running job; additional arrivals are canceled when the pending queue is full. GitHub documents FIFO based on when each item begins waiting, not strictly on dispatch time. This bounds pending retention while avoiding default single-pending replacement.
5. The separate `deploy/k3s-local/Dockerfile` and its broad build context remain outside this lane and require the assigned D2 review before claiming all Docker artifacts are Star-only.
6. This is one release-control slice; it does not close the stable-release WBS or establish a release candidate.

## §4 Subagent handoff

- No child agents were dispatched; the lane was a bounded workflow and evidence task.
- D2 owns the overlapping `deploy/k3s-local/Dockerfile`; it was not edited or included in this change.
- Coordinator review should confirm the workflow diff against the registered W7 brief, validate the tag immutability limitation against release governance, then decide whether to integrate this isolated lane into `dev`.
- The lane commit contains the four registered files and remains local. Recommended next action: coordinator review, then integrate this commit into `dev` only if accepted.

## §5 Guard rules

1. Preserve the three-image allowlist and per-service filtered Cargo contexts.
2. Keep the explicit stable release confirmation and digest-format validation fail-closed.
3. Keep the setup-buildx-action pinned to its existing reviewed commit and pin the Buildx release through its supported `version` input.
4. Keep job-level concurrency scoped to workflow, stable release ref/tag, and matrix image; do not use SHA as the identity and do not enable cancellation of a running release job.
5. Keep `queue: max` so duplicate release dispatches for one tag/image remain pending rather than replacing the previous pending item; disclose that the documented limit is 100 pending per group and later arrivals are canceled.
6. Do not build, authenticate to GHCR, publish images, change GitHub repository settings, or modify `deploy/k3s-local/Dockerfile` within this lane.
7. Do not describe local static checks as hosted release evidence or treat this lane as full release approval.

## §6 Sign-off

| Role | Status |
|---|---|
| Implementer | Complete for the registered lane and committed locally. |
| Coordinator review | Pending. |
| Accountable release owner | Release approval remains with the user; no approval or tag is asserted here. |

## §7 Revision history

| Version | Date | Revision | Author / review |
|---|---|---|---|
| v0.1 | 2026-10-31 | Added Buildx version pin, per-ref/SHA/image non-canceling concurrency, static evidence, and explicit release limitations. | Implemented by lane agent; coordinator review pending. |
| v0.2 | 2026-10-31 | Keyed concurrency to workflow/ref/image so a tag stays serialized across SHA changes; enabled the documented `queue: max` behavior and recorded its 100-pending bound and ordering semantics. | Implemented by lane agent; coordinator review pending. |
