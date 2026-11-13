# Brief: STAR-STABLE-W6-P1E-NPM-20261103

**Agent**: worker
**Phase**: STAR-STABLE-W6-P1E-NPM-20261103
**Created**: 2026-11-03 05:44:47

---

Migrate the last P1E integration workflow frontend consumer from pnpm to the already validated npm lockfile and maintained Node runtime.

Baseline: local dev@67755e31b5693e59678679c60277648fd478cd09.

Own exactly:
1. .github/workflows/p1e-integration.yml
   - Both frontend jobs must use actions/setup-node with exact Node 24.22.2, npm cache, and cache-dependency-path frontend/package-lock.json.
   - Remove pnpm/action-setup and pnpm cache/lockfile references from these jobs.
   - Use npm ci for frozen installation and npm exec for Next/Playwright commands.
   - Preserve the existing jobs, test selectors, retry/failure behavior, and other workflow steps.
2. docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md
   - Update only W6 evidence and append exactly one revision row.
   - State that the P1E workflow is moved to npm, but W6 remains ADAPT/OPEN because frontend/package.json has no packageManager field, pnpm-lock.yaml remains, hosted Node 24/Vercel/Neon/Owner/collaboration/consumer/rollback evidence is missing, and no hosted workflow ran.
3. docs/reports/PHASE-STAR-STABLE-W6-P1E-NPM-20261103.md
   - Create the required seven-section phase report with change matrix, validation, known gaps, agent status, guard rules, five signoff roles and revision history.
   - Record that previous local npm ci acceptance used Node 22.19.0/npm 10.9.3; do not claim Node 24/npm 11.19.1 was tested locally or on hosted runners.

Do not edit frontend/package.json, package-lock.json, pnpm-lock.yaml, PR #292 or its branch; the remote PR remains HOLD. Do not change main or any remote. Do not alter other worktrees. Do not run product tests or hosted workflows. Before commit, parse YAML if PyYAML is available; assert the workflow has exactly two setup-node consumers at Node 24.22.2, npm cache points to frontend/package-lock.json, no pnpm commands/actions/references remain in the workflow, and npm ci/npm exec are used; run git diff --check and inspect exact paths. Mention actionlint availability. Commit only your three owned files with author Ulysses <ulysses@mavis.local>; commit message must reference docs/briefs/STAR-STABLE-W6-P1E-NPM-20261103.md. No overclaim: this lane does not resolve dual lockfiles or close W6.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
