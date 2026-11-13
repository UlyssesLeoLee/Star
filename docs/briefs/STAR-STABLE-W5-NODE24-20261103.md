# Brief: STAR-STABLE-W5-NODE24-20261103

**Agent**: worker
**Phase**: STAR-STABLE-W5-NODE24-20261103
**Created**: 2026-11-03 05:21:48

---

Goal: update the legacy Star desktop build workflow from EOL Node 20 to exact Node 24.22.2, then synchronize the Stable Release WBS and this phase report.

Baseline: local dev@92f98f4a8699990227465722777a9edb9baab670. Node.js 24.22.2 is an official LTS release dated 2026-10-28; Node 20 reached EOL on 2026-03-24. Sources: https://nodejs.org/en/blog/release/v24.22.2/ and https://nodejs.org/en/about/eol. Context7 official actions/setup-node docs confirm node-version accepts an exact SemVer string.

Your write scope is exactly:
1. .github/workflows/star-desktop-build.yml - replace only its two node-version: '20' values with node-version: '24.22.2' and update the corresponding step labels from "Install Node.js 20" to "Install Node.js 24.22.2". Preserve all other workflow content. Do not incorporate or alter the remote Dependabot branch/PR.
2. docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md - update only W5 evidence and append one revision entry. Record static local evidence and keep W5 OPEN plus v1.0.6 NO-GO because hosted Node 24, native package/signature/notarization, BOM/attestation, install/update/rollback and CSP evidence remain missing.
3. docs/reports/PHASE-STAR-STABLE-W5-NODE24-20261103.md - create the required seven-section phase report with sections 0-7, change matrix, validation, known gaps, agent status, 15-17 guard rules, five signoff roles and revision history. State clearly that this lane did not run hosted CI or product tests.

Coordinator owns the dispatcher brief, dispatch JSONL follow-up records and independent review; do not edit those files. No other files may change. Before commit, validate the workflow with a YAML parser if available, assert the two exact pins and absence of Node 20 pins in this workflow, run git diff --check and inspect the exact diff. Mention actionlint availability accurately. Do not run tests or hosted workflows. Commit only your three owned paths with author Ulysses <ulysses@mavis.local>; commit message must reference docs/briefs/STAR-STABLE-W5-NODE24-20261103.md. Keep all remote refs, main and unrelated worktrees untouched.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
