# Brief: STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102

**Agent**: codex
**Phase**: STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102
**Created**: 2026-11-02 03:42:37

---

## Objective
Produce a narrow, evidence-backed W4 update for the cargo-audit install gate, then update the stable-release WBS. Do not change the blocked workflow or any other source.

## Ownership and manifest
Only write the registered files:
- docs/briefs/STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102.md (dispatcher-generated; do not edit after creation)
- docs/reports/PHASE-STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102.md
- docs/reports/WBS-DISPATCH-STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102.jsonl
- docs/reviews/WBS-STAR-STABLE-W4-UPSTREAM-ASSET-REVIEW-20261102.md
- docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md

Do not edit .github/workflows/license-gate.yml: MAGOS denied overlap with origin/dependabot/github_actions/dtolnay/rust-toolchain-1.120.0. Do not edit, rebase, delete, or cherry-pick that source. Preserve D:\Star user changes, main, all other worktrees, and remote refs. No tag/release, build/publish/deploy, or push.

## Verified starting evidence to independently check
- Current target: dev@2b3aab3366f82f0b5e12603d351e32128db5fe41.
- Workflow currently pins cargo-audit version 0.22.2 but keeps CARGO_AUDIT_SHA256 as REPLACE_WITH_UPSTREAM_SHA256_FOR_0.22.2. Its constructed download filename is cargo-audit-v0.22.2-x86_64-unknown-linux-musl.tar.gz.
- Official GitHub release tag is cargo-audit/v0.22.2. The official asset name is cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz, size 6,648,968, asset id 439292222, reported digest sha256:7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428. A direct download hash was independently measured as the same value. Verify using official release/API metadata and a fresh direct-download hash; do not execute the downloaded archive.
- Remote source commit f6bede93 changes dtolnay/rust-toolchain action refs in 8 workflows from @1.99.0 to @1.120.0. Official action documentation says the @rev selects the Rust toolchain version, so that change is incompatible with this repo's Rust 1.99.0 baseline. Keep this branch untouched and unmerged; record the source-overlap reason accurately.
- The current workflow URL/asset filename mismatch and placeholder checksum are separate fail-closed defects. Do not claim the gate is fixed without an authorized source edit and a workflow run.

## Deliverables
- PHASE report with the required sections 0-7 and evidence limitations.
- WBS v1.38 dated 2026-11-02: correct W4 with exact upstream asset identity, filename mismatch, hash verification, and the MAGOS overlap. Keep W4 OPEN/BLOCKED and the overall v1.0.6 release NO-GO; preserve all unrelated WBS statuses and historical entries.
- Review note that distinguishes API metadata and direct download hash from hosted workflow execution or artifact attestation.
- Add JSONL dispatch/validation events using the existing record format.

## Validation
Check official asset id/name/digest/size; hash the exact download and compare. Parse JSONL, ensure the W4 row remains unique and WBS revision is v1.38, confirm report sections 0-7, run git diff --check, and list the changed paths. No tests/builds are required because this is a documentation-only evidence lane.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
