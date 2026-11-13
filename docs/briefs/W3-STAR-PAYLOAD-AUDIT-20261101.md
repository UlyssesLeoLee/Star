# Brief: W3-STAR-PAYLOAD-AUDIT-20261101

**Agent**: reviewer
**Phase**: W3-STAR-PAYLOAD-AUDIT-20261101
**Created**: 2026-11-01 02:52:54

---

## Objective
Run a read-only W3 release-payload boundary audit for all Star container/build-context paths. Confirm that Docker-related product artifacts contain only approved Star components and their declared build/runtime dependencies, and identify evidence still required before release.

## Approved Stable Release scope
- Desktop application
- Web/Vercel
- Three Star Canvas service images
- MIT license identity; release version derives only from Git tag `v1.0.6`

## Review baselines
- Current dev: `a272e9794e1a1975781720c9b3775c914f47f546`
- W7 context-filter candidate branch `codex/star-stable-w7-k3s-context-20261031`, HEAD `c960b9193cc960ae3d85711472a0d4f31fedd0b3`; this is read-only and may be active elsewhere.
- A prior read-only audit reported that `.github/workflows/publish-star-canvas-services.yml` allowlists three Canvas images, builds filtered Cargo contexts, rejects non-allowlisted local workspace dependencies and paths containing `game`, and emits SBOM/provenance; Canvas Dockerfiles copy one crate plus generated lock/workspace manifest and LICENSE. It also reported `deploy/k3s-local/Dockerfile` uses broad `COPY crates ./crates`, `.dockerignore` only excludes `star-game` and `game-legacy`, and `deploy/helm/star` is deprecated/empty while active digest-pinned deployment uses `deploy/star-canvas-services-k3s.yaml`. Verify each item against current commits and candidate; treat this as a hypothesis, not evidence.

## Owned output files
- `docs/briefs/W3-STAR-PAYLOAD-AUDIT-20261101.md` (this brief)
- `docs/reviews/W3-STAR-RELEASE-PAYLOAD-AUDIT-20261101.md`
- `docs/reports/PHASE-STAR-STABLE-W3-PAYLOAD-AUDIT-20261101.md`
- `docs/reports/WBS-DISPATCH-W3-STAR-PAYLOAD-AUDIT-20261101.jsonl`

## Audit scope and method
- Inspect Dockerfiles, `.dockerignore`, image workflows, BuildKit contexts/filters, SBOM/provenance config, Helm/K3s manifests, and relevant scripts in current dev. Read the W7 candidate only through RTK-mediated Git, and compare its exact diff to current dev.
- Inventory every image/build context and classify it as approved release, development-only, or deprecated. Check whether context contents are actually narrowed; `.dockerignore` declarations alone do not prove final image contents.
- Verify the three Canvas service image allowlist and every local workspace dependency. Check if `deploy/k3s-local` is Star-only or should be explicitly excluded/reclassified; do not assume.
- Provide exact file/line evidence, findings by severity, decision per image/context, blockers, and precise remediation/acceptance evidence. Distinguish static review from an actual isolated build-context archive and final image/SBOM inspection; do not claim artifact verification unless performed.
- Do not build, publish, deploy, or change any Dockerfile, workflow, manifest, WBS main document, candidate branch, or other worktree.
- The phase report must contain the repository-required 7 sections, exact commands and results, known gaps, and must not mark W3 closed when real artifact/SBOM/attestation evidence is absent. Commit only owned files and include this dispatcher brief path in the commit subject.

## CodeGraph / source context
This is a configuration/document audit; CodeGraph does not cover the relevant Dockerfile/workflow content. The current session's CodeGraph tool reports 1,907 indexed files but exposes no generation or `check_index_coverage` API. Use targeted RTK `rg`/Git reads for non-code files; do not claim exhaustive graph coverage.

## Isolation
Work only in `C:\Users\leo19\.codex\worktrees\star-stable-w3-payload-audit-20261101\Star` on branch `codex/star-stable-w3-payload-audit-20261101`. You are one of several agents; do not revert other agents’ changes. Keep only to owned files. Do not touch `D:\Star` user files, other branches/worktrees, `main`, remote refs, tags, or push/deploy/release anything.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
