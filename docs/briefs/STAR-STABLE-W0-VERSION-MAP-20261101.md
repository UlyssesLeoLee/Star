# Brief: STAR-STABLE-W0-VERSION-MAP-20261101

**Agent**: worker
**Phase**: STAR-STABLE-W0-VERSION-MAP-20261101
**Created**: 2026-11-01 00:09:48

---

## Objective
Audit how the user-selected Git tag v1.0.6 can remain the single canonical release version across Desktop, Web/Vercel, and the three Star Canvas service images. Produce a current-state mapping and a concrete, minimal implementation/acceptance plan. This lane is an audit; do not change product version files or create the tag.

## Owned files
- docs/reports/PHASE-STAR-STABLE-W0-VERSION-MAP-20261101.md
- docs/reports/WBS-DISPATCH-STAR-STABLE-W0-VERSION-MAP-20261101.jsonl
The coordinator owns docs/briefs/STAR-STABLE-W0-VERSION-MAP-20261101.md.

## Requirements
1. Inspect current dev@2a57fa0d read-only. Identify every release-facing version surface: root Cargo packages/workspace version, BFF/ACI crates where included, Web package/app version, Tauri bundle/product metadata and artifact names, Canvas OCI tags/labels, K3s manifest image references, and any active release workflow inputs.
2. Inspect local Git tags and history relevant to v1.0.5/v1.0.6; do not create or move tags. Do not refresh remote refs.
3. For each surface, report actual value/source, whether it derives from Git tag, and the smallest change needed to make v1.0.6 authoritative without a second canonical version source.
4. Separate active release surfaces from deprecated placeholders and CI-only/development artifacts; do not count empty deprecated Helm metadata as a product payload.
5. Define final-source commit and release procedure checks, including immutable-tag control and evidence needed before publishing.
6. Verify the report cites exact repository paths/lines and captures command/toolchain and limitations; no production implementation is made in this lane.

## Boundaries
- Only write this report and this dispatch JSONL. Do not edit Cargo.toml, package manifests/lockfiles, workflow files, Tauri config, deploy manifests, or any other lane's files.
- Do not create/move tags, build/publish/deploy artifacts, push, or touch D:\Star, main, D2, or other worktrees.
- The report must use the project's 7-section PHASE structure and keep W0 PARTIAL until actual mapping and final source commit are implemented and verified.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
