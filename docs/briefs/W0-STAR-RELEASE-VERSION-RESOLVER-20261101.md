# Brief: W0-STAR-RELEASE-VERSION-RESOLVER-20261101

**Agent**: worker
**Phase**: W0-STAR-RELEASE-VERSION-RESOLVER-20261101
**Created**: 2026-11-01 02:51:15

---

## Objective
Implement the strict Git-tag-derived Star release version resolver for W0. The only version authority is an exact stable Git tag such as `v1.0.6`; emit the normalized SemVer and the peeled source commit SHA.

## Owned files
- `scripts/release/resolve_star_version.py`
- `scripts/release/tests/test_resolve_star_version.py`
- `docs/briefs/W0-STAR-RELEASE-VERSION-RESOLVER-20261101.md` (this brief)
- `docs/reports/PHASE-STAR-STABLE-W0-RESOLVER-20261101.md`
- `docs/reports/WBS-DISPATCH-W0-STAR-RELEASE-VERSION-RESOLVER-20261101.jsonl`

## Requirements
- Accept only canonical `vMAJOR.MINOR.PATCH` stable tags (reject prerelease/build metadata, missing components, whitespace, and leading zeroes except `0`).
- Resolve the named tag to a commit using safe argument-list subprocess calls, no shell. Confirm the exact tag ref exists, peeled object is a commit, and checked-out `HEAD` equals the resolved commit.
- Emit machine-readable JSON containing tag, normalized release version, and source SHA. Optionally append `STAR_RELEASE_TAG`, `STAR_RELEASE_VERSION`, and `STAR_RELEASE_SOURCE_SHA` to an explicitly supplied GitHub output file only after all checks pass.
- Fail with actionable errors and nonzero exit for malformed tag, missing tag, non-commit target, or HEAD mismatch. Avoid printing secrets or rewriting Git refs.
- Keep dependencies to Python standard library. Add focused tests for valid annotated/lightweight tags and invalid values, missing refs, and mismatched HEAD. Use an isolated temporary Git repo in tests.
- Do not modify any workflow, Cargo manifest/lockfile, K3s template, WBS main document, `main`, or other worktree. W0 remains partial until release surfaces are wired and hosted artifact verification is complete.

## Validation and report
Run the focused resolver tests and `git diff --check` through the repository RTK workflow. The phase report must have the repository-required 7 sections, list exact changed files, actual commands/results, known gaps, and explicitly state that workflow integration/hosted proof remain open. Commit only the owned files; commit subject must mention this dispatcher brief path.

## CodeGraph / source context
Parent queried CodeGraph for release/version resolver and `SubagentDispatcher.brief`; it returned no relevant existing resolver implementation, with 1,907 files indexed, but no project generation or coverage API. Exact configuration/workflow discovery used targeted `rg` fallback. Treat the new resolver path as unindexed; do not claim exhaustive negative coverage. Independent acceptance review and dev integration are owned by the coordinator.

## Isolation
Work only in `C:\Users\leo19\.codex\worktrees\star-stable-w0-release-version-resolver\Star` on branch `codex/star-stable-w0-release-version-resolver-20261101`, based on dev `a272e9794e1a1975781720c9b3775c914f47f546`. Do not touch `D:\Star` user files, other branches/worktrees, remote refs, `main`, or publish/tag/push anything. You are one of several agents; do not revert others’ edits and keep to this file ownership.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
